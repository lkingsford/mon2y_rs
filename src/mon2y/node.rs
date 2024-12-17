use super::game::{Action, State};
use core::panic;
use log::{debug, trace, warn};
use rand::Rng;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub enum Node<StateType: State, ActionType: Action<StateType = StateType>> {
    Expanded {
        state: StateType,
        children: HashMap<ActionType, Arc<RwLock<Node<StateType, ActionType>>>>,
        visit_count: u32,
        /// Sum of rewards for this player
        value_sum: f64,
        cached_ucb: RwLock<Option<f64>>,
    },
    Placeholder,
}

impl<StateType: State, ActionType: Action<StateType = StateType>> Node<StateType, ActionType> {
    pub fn fully_explored(&self) -> bool {
        match self {
            Node::Expanded {
                state, children, ..
            } => {
                children.is_empty()
                    || children.iter().all(|(_, child)| match child {
                        Node::Expanded { .. } => child.fully_explored(),
                        Node::Placeholder => false,
                    })
            }
            Node::Placeholder => false,
        }
    }

    pub fn visit_count(&self) -> u32 {
        match self {
            Node::Expanded { visit_count, .. } => *visit_count,
            Node::Placeholder => 0,
        }
    }

    pub fn value_sum(&self) -> f64 {
        match self {
            Node::Expanded { value_sum, .. } => *value_sum,
            Node::Placeholder => 0.0,
        }
    }

    pub fn visit(&mut self, reward: f64) {
        match self {
            Node::Expanded {
                visit_count,
                value_sum,
                cached_ucb,
                ..
            } => {
                *visit_count += 1;
                *value_sum += reward as f64;
                self.invalidate_cached_ucb(true);
            }
            Node::Placeholder => {
                warn!("Visiting placeholder node");
            }
        }
    }

    pub fn invalidate_cached_ucb(&self, recurse: bool) {
        match self {
            Node::Expanded {
                cached_ucb,
                children,
                ..
            } => {
                let mut cached_ucb_ref = cached_ucb.write().unwrap();
                *cached_ucb_ref = None;
                if !recurse {
                    for child in children.values() {
                        // Only need to invalidate the first level of child: 'parent visits' is part of ucb
                        child.invalidate_cached_ucb(false);
                    }
                }
            }
            Node::Placeholder => {}
        }
    }

    pub fn cache_ucb(&self, ucb: f64) {
        match self {
            Node::Expanded { cached_ucb, .. } => {
                let mut cached_ucb_ref = cached_ucb.write().unwrap();
                *cached_ucb_ref = Some(ucb);
            }
            Node::Placeholder => {}
        }
    }

    pub fn cached_ucb(&self) -> Option<f64> {
        match self {
            Node::Expanded { cached_ucb, .. } => {
                let ucb = cached_ucb.read().unwrap();
                *ucb
            }
            Node::Placeholder => None,
        }
    }

    pub fn expansion(
        &self,
        action: ActionType,
        parent_state: &<ActionType as Action>::StateType,
    ) -> Node<StateType, <StateType as State>::ActionType> {
        if let Node::Expanded { .. } = self {
            panic!("Expanding an expanded node");
        }
        let state = action.execute(parent_state);
        Self::new_expanded(state)
    }
    pub fn best_pick(&self, constant: f64) -> Vec<ActionType> {
        // also fix fror other player rewards
        match self {
            Node::Expanded { children, .. } => {
                let mut ucbs: Vec<(ActionType, f64)> = children
                    .iter()
                    .filter_map(|(action, child_node)| {
                        if child_node.fully_explored() {
                            return None;
                        }
                        let cached_ucb = child_node.cached_ucb();
                        if let Some(ucb) = cached_ucb {
                            return Some((action.clone(), ucb));
                        }
                        let visit_count = child_node.visit_count() as f64;
                        let parent_visits = self.visit_count() as f64;
                        if visit_count == 0.0 {
                            return Some((action.clone(), f64::INFINITY));
                        }
                        let q: f64 = child_node.value_sum() / visit_count;
                        let u: f64 = (parent_visits.ln() / visit_count).sqrt();
                        // Random used to break ties
                        // Todo: Cache the rng
                        let r: f64 = rand::thread_rng().gen::<f64>() * 1e-6;
                        let ucb: f64 = (q + constant * u + r);
                        trace!(
                            "UCB action: {:?}, value_sum: {}, visit_count: {}, parent_visits: {}, q: {}, u: {}, c: {} ucb: {}",
                            action,
                            child_node.value_sum(),
                            child_node.visit_count(),
                            parent_visits,
                            q,
                            u,
                            constant,
                            ucb
                        );
                        Some((action.clone(), ucb))
                    })
                    .collect();
                for (action, ucb) in ucbs.iter_mut() {
                    let node = children.get(action).unwrap();
                    node.cache_ucb(*ucb);
                }
                ucbs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                debug!("UCBS action, ucb: {:?}", ucbs.iter().collect::<Vec<_>>());
                ucbs.iter().map(|(action, _)| action.clone()).collect()
            }
            Node::Placeholder => Vec::new(),
        }
    }

    pub fn state(&self) -> &StateType {
        match self {
            Node::Expanded { state, .. } => state,
            Node::Placeholder => panic!("Placeholder node has no state"),
        }
    }

    pub fn insert_child(&mut self, action: ActionType, child: Node<StateType, ActionType>) {
        if let Node::Expanded { children, .. } = self {
            children.insert(action, child);
        } else {
            panic!("Inserting child into placeholder");
        }
    }

    pub fn get_child(&self, action: ActionType) -> Arc<RwLock<Node<StateType, ActionType>>> {
        if let Node::Expanded { children, .. } = self {
            children.get(&action).unwrap().clone()
        } else {
            panic!("Getting child from placeholder");
        }
    }

    pub fn new_expanded(state: StateType) -> Node<StateType, <StateType as State>::ActionType> {
        create_expanded_node(state)
    }

    pub fn trace_log_children(&self, level: usize) {
        match self {
            Node::Expanded { children, .. } => {
                for (action, child) in children.iter() {
                    match child {
                        Node::Expanded { .. } => {
                            let action_name = format!("{:?}", action);
                            trace!("{} {}", "         |-".repeat(level), action_name);
                            trace!(
                                "{} {:.6} {}",
                                "         | ".repeat(level),
                                child.value_sum(),
                                child.visit_count()
                            );
                            trace!(
                                "{} {:.6}",
                                "         | ".repeat(level),
                                child.value_sum() / (child.visit_count() as f64)
                            );
                            child.trace_log_children(level + 1);
                        }
                        Node::Placeholder => {
                            let action_name = format!("({:?})", action);
                            trace!("{} {}", "         |-".repeat(level), action_name);
                        }
                    }
                }
            }
            Node::Placeholder => return,
        }
    }
}

pub fn create_expanded_node<StateType>(state: StateType) -> Node<StateType, StateType::ActionType>
where
    StateType: State,
{
    // Used here so can be used outside of an instance of Node
    // (I think the Node::new_expanded should be able to work? But my rust brain
    // is still learning and couldn't figure out syntax that the type checker
    // was happy with)
    let mut children = HashMap::new();
    for action in state.permitted_actions() {
        children.insert(action, Node::Placeholder);
    }
    Node::Expanded {
        state,
        children,
        visit_count: 0,
        value_sum: 0.0,
        cached_ucb: RwLock::new(None),
    }
}
