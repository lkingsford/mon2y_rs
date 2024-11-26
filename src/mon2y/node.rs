use rand::{self, Rng};

use super::action::Action;
use super::state::State;
use std::collections::HashMap;
use std::option::Option;

pub type ActCallable = fn(state: &dyn State, action: Action) -> ActResponse;

pub enum Reward {
    Ongoing,
    OngoingBreadcrumb(Vec<f64>),
    Finished(Vec<f64>),
}

pub struct ActResponse {
    pub permitted_actions: Vec<Action>,
    pub state: Box<dyn State>,
    pub next_player: Option<u8>,
    pub reward: Option<Vec<f64>>,
    pub next_act_fn: Box<ActCallable>,
    pub memo: Option<Box<str>>,
}

pub struct Node {
    pub action: Action,
    pub state: Option<Box<dyn State>>,
    pub act_fn: ActCallable,
    pub player_id: Option<u8>,
    pub reward: Reward,
    pub permitted_actions: Option<Vec<Action>>,
    pub next_player: Option<u8>,
    pub next_act_fn: ActCallable,
    children: HashMap<Action, Node>,
    visit_count: u64,
    value_sum: f64,
}

impl Node {
    pub fn new(
        action: Action,
        state: Option<Box<dyn State>>,
        act_fn: ActCallable,
        player_id: Option<u8>,
        reward: Reward,
        permitted_actions: Option<Vec<Action>>,
        next_player: Option<u8>,
        next_act_fn: ActCallable,
    ) -> Node {
        Node {
            action,
            state,
            act_fn,
            player_id,
            reward,
            permitted_actions,
            next_player,
            next_act_fn,
            children: HashMap::new(),
            visit_count: 0,
            value_sum: 0.0,
        }
    }

    fn leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn fully_explored(&self) -> bool {
        self.children.is_empty()
            || self
                .children
                .iter()
                .all(|(_, child)| child.fully_explored())
    }

    fn best_pick(&self, constant: f64) -> Vec<Action> {
        let mut ucbs: Vec<(Action, f64)> = self
            .children
            .iter()
            .map(|(action, child_node)| {
                // UCB formula
                let q: f64 = child_node.value_sum / (1.0 + child_node.visit_count as f64);
                let u: f64 = (self.visit_count as f64 / child_node.visit_count as f64)
                    .ln()
                    .sqrt();
                // Random used to break ties
                let r: f64 = rand::thread_rng().gen::<f64>() * 1e-6;
                (action.clone(), q + constant * u + r)
            })
            .collect();
        ucbs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ucbs.iter().map(|(a, _)| a.clone()).collect()
    }
}
