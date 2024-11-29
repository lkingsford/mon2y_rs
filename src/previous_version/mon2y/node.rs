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
    Unknown,
}

pub struct ActResponse {
    pub permitted_actions: Vec<Action>,
    pub state: dyn State,
    pub next_player: Option<u8>,
    pub reward: Option<Vec<f64>>,
    pub terminated: bool,
    pub next_act_fn: Option<ActCallable>,
    pub memo: Option<Box<str>>,
}

pub struct Node<'a> {
    pub action: Action,
    pub state: Option<&'a dyn State>,
    pub act_fn: ActCallable,
    pub player_id: Option<u8>,
    pub reward: Reward,
    pub permitted_actions: Option<Vec<Action>>,
    pub next_player: Option<u8>,
    pub next_act_fn: Option<ActCallable>,
    children: HashMap<Action, &Node>,
    visit_count: u64,
    value_sum: f64,
    parent_state: Option<&'a dyn State>,
}

impl Node {
    pub fn new(
        action: Action,
        state: Option<&dyn State>,
        act_fn: ActCallable,
        player_id: Option<u8>,
        reward: Reward,
        permitted_actions: Option<Vec<Action>>,
        next_player: Option<u8>,
        next_act_fn: Option<ActCallable>,
        parent_state: Option<&'a dyn State>,
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
            parent_state,
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
    pub fn best_pick(&self) -> Vec<(Action, f64)> {
        let constant = 2.0_f64.sqrt();
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
        ucbs
    }

    pub fn selection(&mut self) -> Option<&mut Node> {
        if self.fully_explored() {
            return None;
        }
        if self.leaf() {
            return Some(self);
        }
        let mut current_selection = self;
        while !current_selection.leaf() {
            let best_picks = current_selection.best_pick();
            let best_pick = best_picks[0].0.clone();
            // Use `get_mut` to get a mutable reference from `children`
            current_selection = current_selection.children.get_mut(&best_pick).unwrap();
        }
        Some(current_selection)
    }

    pub fn expansion(&mut self) -> () {
        if self.permitted_actions.is_none() | self.state.is_none() {
            let act_response = (self.act_fn)(
                self.parent_state.as_ref().unwrap().as_ref(),
                self.action.clone(),
            );
            self.permitted_actions = Some(act_response.permitted_actions);
            self.state = Some(act_response.state);
            self.next_player = act_response.next_player;
            self.next_act_fn = act_response.next_act_fn;
        }
        for action in self.permitted_actions.as_ref().unwrap() {
            let child = Node::new(
                action.clone(),
                None,
                self.next_act_fn.unwrap(),
                self.next_player,
                Reward::Unknown,
                None,
                None,
                None,
                Some(self.state.as_ref().unwrap().clone()),
            );
            self.children.insert(action.clone(), child);
        }
    }

    pub fn play_out(&self) -> Option<Reward> {
        todo!()
    }

    pub fn back_propogate(&self, reward: Option<Reward>) {}
}
