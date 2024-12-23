use super::game::{Action, State};
use core::panic;
use log::{debug, trace, warn};
use rand::Rng;
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
struct CachedUcb {
    ucb: f64,
    value_sum: f64,
    visit_count: u32,
    parent_visit_count: u32,
}

#[derive(Debug)]
pub enum Node<StateType: State, ActionType: Action<StateType = StateType>> {
    Expanded {
        state: StateType,
        children: HashMap<ActionType, Arc<RwLock<Node<StateType, ActionType>>>>,
        visit_count: u32,
        /// Sum of rewards for this player
        value_sum: f64,
        cached_ucb: RwLock<Option<CachedUcb>>,
        cached_fully_explored: Option<RwLock<bool>>,
    },
    Placeholder,
}

impl<StateType: State, ActionType: Action<StateType = StateType>> Node<StateType, ActionType> {
    pub fn fully_explored(&self) -> bool {
        match self {
            Node::Expanded {
                children,
                cached_fully_explored,
                ..
            } => {
                if let Some(cached) = cached_fully_explored {
                    let read_lock = cached.read().unwrap();
                    if *read_lock == true {
                        return true;
                    }
                };
                let child_nodes: Vec<Arc<RwLock<Node<StateType, ActionType>>>> =
                    { children.values().cloned().collect() };
                child_nodes.is_empty()
                    || child_nodes.iter().all(|child| {
                        let child = child.clone();
                        let child_node = child.read().unwrap();
                        match *child_node {
                            Node::Expanded { .. } => child_node.fully_explored(),
                            Node::Placeholder => false,
                        }
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
                ..
            } => {
                *visit_count += 1;
                *value_sum += reward as f64;
            }
            Node::Placeholder => {
                warn!("Visiting placeholder node");
            }
        }
    }

    pub fn cache_ucb(&self, ucb: f64, value_sum: f64, visit_count: u32, parent_visit_count: u32) {
        match self {
            Node::Expanded { cached_ucb, .. } => {
                if let Ok(mut cached_ucb_ref) = cached_ucb.try_write() {
                    *cached_ucb_ref = Some(CachedUcb {
                        ucb,
                        value_sum,
                        visit_count,
                        parent_visit_count,
                    });
                }
            }
            Node::Placeholder => {}
        }
    }

    pub fn cached_ucb(
        &self,
        value_sum: f64,
        visit_count: u32,
        parent_visit_count: u32,
    ) -> Option<f64> {
        match self {
            Node::Expanded { cached_ucb, .. } => {
                let ucb = cached_ucb.read().unwrap();
                match *ucb {
                    Some(CachedUcb {
                        ucb: cached_ucb,
                        value_sum: cached_value_sum,
                        visit_count: cached_visit_count,
                        parent_visit_count: cached_parent_visit_count,
                    }) => {
                        if cached_value_sum == value_sum
                            && cached_visit_count == visit_count
                            && cached_parent_visit_count == parent_visit_count
                        {
                            Some(cached_ucb)
                        } else {
                            None
                        }
                    }
                    None => None,
                }
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
    pub fn state(&self) -> &StateType {
        match self {
            Node::Expanded { state, .. } => state,
            Node::Placeholder => panic!("Placeholder node has no state"),
        }
    }

    pub fn insert_child(&mut self, action: ActionType, child: Node<StateType, ActionType>) {
        if let Node::Expanded { children, .. } = self {
            children.insert(action, Arc::new(RwLock::new(child)));
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

    pub fn get_node_by_path(
        &self,
        path: Vec<ActionType>,
    ) -> Arc<RwLock<Node<StateType, ActionType>>> {
        if path.is_empty() {
            panic!("Can't return empty path")
        }
        let mut node = None;
        for action in path {
            if node.is_none() {
                node = Some(self.get_child(action));
            } else {
                node = Some(node.unwrap().read().unwrap().get_child(action).clone());
            }
        }
        node.unwrap()
    }

    pub fn trace_log_children(&self, level: usize) {
        match self {
            Node::Expanded { children, .. } => {
                for (action, child) in children.iter() {
                    let cloned_child = child.clone();
                    let child_node = cloned_child.read().unwrap();
                    match *child_node {
                        Node::Expanded { .. } => {
                            let action_name = format!("{:?}", action);
                            trace!("{} {}", "         |-".repeat(level), action_name);
                            trace!(
                                "{} {:.6} {}",
                                "         | ".repeat(level),
                                child_node.value_sum(),
                                child_node.visit_count()
                            );
                            trace!(
                                "{} {:.6}",
                                "         | ".repeat(level),
                                child_node.value_sum() / (child_node.visit_count() as f64)
                            );
                            child_node.trace_log_children(level + 1);
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

pub fn best_pick<StateType, ActionType>(
    node_lock: &RwLock<Node<StateType, ActionType>>,
    constant: f64,
) -> Vec<ActionType>
where
    StateType: State<ActionType = ActionType>,
    ActionType: Action<StateType = StateType>,
{
    let children: HashMap<ActionType, Arc<RwLock<Node<StateType, ActionType>>>> = {
        let node = node_lock.read().unwrap();
        match &*node {
            Node::Expanded { children, .. } => children
                .iter()
                .map(|(action, child)| (action.clone(), child.clone()))
                .collect(),
            Node::Placeholder => {
                return vec![];
            }
        }
    };
    let parent_visit_count = { node_lock.read().unwrap().visit_count() };

    let mut ucbs: Vec<(ActionType, f64)> = children
                    .iter()
                    .filter_map(|(action, child_node)| {
                        let (visit_count, value_sum) = {
                        let child_ref = child_node.clone();
                        let child_node = child_ref.read().unwrap();
                        if child_node.fully_explored() {
                            return None;
                        }
                        let cached_ucb = child_node.cached_ucb(
                            child_node.value_sum(), child_node.visit_count(), parent_visit_count);
                        if let Some(ucb) = cached_ucb {
                            return Some((action.clone(), ucb));
                        }
                        (child_node.visit_count() as f64, child_node.value_sum())
                    };
                        let parent_visits = parent_visit_count as f64;
                        if visit_count == 0.0 {
                            return Some((action.clone(), f64::INFINITY));
                        }
                        let q: f64 = value_sum / visit_count;
                        let u: f64 = (parent_visits.ln() / visit_count).sqrt();
                        // Random used to break ties
                        // Todo: Cache the rng
                        let r: f64 = rand::thread_rng().gen::<f64>() * 1e-6;
                        let ucb: f64 = (q + constant * u + r);
                        trace!(
                            "UCB action: {:?}, value_sum: {}, visit_count: {}, parent_visits: {}, q: {}, u: {}, c: {} ucb: {}",
                            action,
                            value_sum,
                            visit_count,
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
        let read_node = node.read().unwrap();
        read_node.cache_ucb(
            *ucb,
            read_node.value_sum(),
            read_node.visit_count(),
            parent_visit_count,
        );
    }
    ucbs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    debug!("UCBS action, ucb: {:?}", ucbs.iter().collect::<Vec<_>>());
    ucbs.iter().map(|(action, _)| action.clone()).collect()
}

pub fn create_expanded_node<StateType>(state: StateType) -> Node<StateType, StateType::ActionType>
where
    StateType: State,
{
    // Used here so can be used outside of an instance of Node
    // (I think the Node::new_expanded should be able to work? But my rust brain
    // is still learning and couldn't figure out syntax that the type checker
    // was happy with)
    let mut children: HashMap<
        StateType::ActionType,
        Arc<RwLock<Node<StateType, StateType::ActionType>>>,
    > = HashMap::new();
    for action in state.permitted_actions() {
        children.insert(action, Arc::new(RwLock::new(Node::Placeholder)));
    }
    Node::Expanded {
        state,
        children,
        visit_count: 0,
        value_sum: 0.0,
        cached_ucb: RwLock::new(None),
        cached_fully_explored: None,
    }
}
