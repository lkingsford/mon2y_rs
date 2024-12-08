use super::game::{Action, State};
use core::panic;
use log::warn;
use rand::Rng;
use std::collections::HashMap;

const MIN_CHILD_VISIT: f64 = 0.00000000001;
#[derive(Debug)]
pub enum Node<StateType: State, ActionType: Action<StateType = StateType>> {
    Expanded {
        state: StateType,
        children: HashMap<ActionType, Node<StateType, ActionType>>,
        visit_count: u32,
        /// Sum of rewards for this player
        value_sum: f64,
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
        match self {
            Node::Expanded { children, .. } => {
                let mut ucbs: Vec<(ActionType, f64)> = children
                    .iter()
                    .map(|(action, child_node)| {
                        // UCB formula
                        let q: f64 =
                            child_node.value_sum() / (1.0 + child_node.visit_count() as f64);
                        let u: f64 = (f64::max(MIN_CHILD_VISIT, self.visit_count() as f64)
                            / (f64::max(MIN_CHILD_VISIT, child_node.visit_count() as f64)))
                        .ln()
                        .sqrt();
                        // Random used to break ties
                        // Todo: Cache the rng
                        let r: f64 = rand::thread_rng().gen::<f64>() * 1e-6;
                        (action.clone(), q + constant * u + r)
                    })
                    .collect();
                ucbs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
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

    pub fn get_child_mut(&mut self, action: ActionType) -> &mut Node<StateType, ActionType> {
        if let Node::Expanded { children, .. } = self {
            children.get_mut(&action).unwrap()
        } else {
            panic!("Getting child from placeholder");
        }
    }

    pub fn get_child<'a>(&'a self, action: ActionType) -> &Node<StateType, ActionType> {
        if let Node::Expanded { children, .. } = self {
            children.get(&action).unwrap()
        } else {
            panic!("Getting child from placeholder");
        }
    }

    pub fn new_expanded(state: StateType) -> Node<StateType, <StateType as State>::ActionType> {
        create_expanded_node(state)
    }

    pub fn get_node_by_path(&self, path: Vec<ActionType>) -> &Node<StateType, ActionType> {
        let mut node = self;
        for action in path {
            node = node.get_child(action);
        }
        node
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
    }
}
