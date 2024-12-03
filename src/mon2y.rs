use log::{debug, info, warn};
use std::collections::HashMap;

use rand::Rng;
pub trait Action: Clone + Copy + Eq + std::hash::Hash {
    type StateType: State<ActionType = Self>;
    fn execute(&self, state: &Self::StateType) -> Self::StateType;
}
///
/// An actor is either a player or a game action.
///
/// A player is just an identifier, typically a number between 0 and n-1.
///
/// A game action is a action that the game takes, rather than a player.
pub enum Actor<ActionType> {
    /// A player is just an identifier, typically a number between 0 and n-1.
    Player(u8),
    /// A game action is a action that the game takes (such as rolling a dice, or drawing a card), rather than a player.
    GameAction(ActionType),
}

type Reward = Vec<f64>;

pub trait State {
    type ActionType: Action<StateType = Self>;
    fn permitted_actions(&self) -> Vec<Self::ActionType>;
    fn next_actor(&self) -> Actor<Self::ActionType>;
    fn terminal(&self) -> bool;
    fn reward(&self) -> Vec<f64>;
}

pub enum Node<StateType: State, ActionType: Action<StateType = StateType>> {
    Expanded {
        state: StateType,
        children: HashMap<ActionType, Node<StateType, ActionType>>,
        visit_count: u32,
        value_sum: f64,
    },
    Placeholder,
}

impl<StateType: State, ActionType: Action<StateType = StateType>> Node<StateType, ActionType> {
    fn fully_explored(&self) -> bool {
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

    fn visit_count(&self) -> u32 {
        match self {
            Node::Expanded { visit_count, .. } => *visit_count,
            Node::Placeholder => 0,
        }
    }

    fn value_sum(&self) -> f64 {
        match self {
            Node::Expanded { value_sum, .. } => *value_sum,
            Node::Placeholder => 0.0,
        }
    }

    fn expansion(
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
                        let u: f64 = (self.visit_count() as f64 / child_node.visit_count() as f64)
                            .ln()
                            .sqrt();
                        // Random used to break ties
                        // Todo: Cache the rng
                        let r: f64 = rand::thread_rng().gen::<f64>() * 1e-6;
                        (action.clone(), q + constant * u + r)
                    })
                    .collect();
                ucbs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                ucbs.iter().map(|(action, _)| action.clone()).collect()
            }
            Node::Placeholder => Vec::new(),
        }
    }

    fn state(&self) -> &StateType {
        match self {
            Node::Expanded { state, .. } => state,
            Node::Placeholder => panic!("Placeholder node has no state"),
        }
    }

    fn insert_child(&mut self, action: ActionType, child: Node<StateType, ActionType>) {
        if let Node::Expanded { children, .. } = self {
            children.insert(action, child);
        } else {
            panic!("Inserting child into placeholder");
        }
    }

    fn get_child(&mut self, action: ActionType) -> &mut Node<StateType, ActionType> {
        if let Node::Expanded { children, .. } = self {
            children.get_mut(&action).unwrap()
        } else {
            panic!("Getting child from placeholder");
        }
    }

    fn new_expanded(state: StateType) -> Node<StateType, <StateType as State>::ActionType> {
        let mut children = HashMap::new();
        for action in state.permitted_actions() {
            // This should probably be in node.expansion, but my baby-rust
            // brain can't quite figure it out right now.
            children.insert(action, Node::Placeholder);
        }
        Node::Expanded {
            state,
            children,
            visit_count: 0,
            value_sum: 0.0,
        }
    }
}

pub enum Selection<ActionType: Action> {
    FullyExplored,
    Selection(Vec<ActionType>),
}

pub struct Tree<StateType: State, ActionType: Action<StateType = StateType>> {
    root: Node<StateType, ActionType>,
    constant: f64,
}

impl<StateType: State<ActionType = ActionType>, ActionType: Action<StateType = StateType>>
    Tree<StateType, ActionType>
{
    pub fn new(root: Node<StateType, ActionType>) -> Tree<StateType, ActionType> {
        Tree {
            root,
            constant: 2.0_f64.sqrt(),
        }
    }

    ///
    /// Returns a path to the current selection
    ///
    pub fn selection(&self) -> Selection<ActionType> {
        if self.root.fully_explored() {
            return Selection::FullyExplored;
        }
        if let Node::Placeholder = self.root {
            return Selection::Selection(vec![]);
        }
        let current_selection = &self.root;
        let mut result: Vec<ActionType> = vec![];
        while let Node::Expanded { .. } = current_selection {
            let best_picks = current_selection.best_pick(self.constant);
            let best_pick = best_picks[0].clone();
            // Use `get_mut` to get a mutable reference from `children`
            result.push(best_pick);
        }
        Selection::Selection(result)
    }

    pub fn expansion(&mut self, selection: Selection<ActionType>) {
        let mut cur_node = &mut self.root;

        if let Selection::Selection(selection) = selection {
            for action in &selection {
                // Navigate the tree by taking mutable references
                let parent_state = cur_node.state();
                let mut expanded_child = cur_node.expansion(*action, parent_state);
                cur_node.insert_child(action.clone(), expanded_child);
                cur_node = cur_node.get_child(*action);
            }
        }
    }

    pub fn play_out(&self, selection: Selection<ActionType>) -> Option<Reward> {
        todo!()
    }

    pub fn propagate_reward(&mut self, selection: Selection<ActionType>, reward: Reward) {
        todo!()
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestGameState {
        injected_reward: Vec<f64>,
        injected_terminal: bool,
        injected_permitted_actions: Vec<TestGameAction>
    }

    impl State for TestGameState {
        type ActionType = TestGameAction;
        fn permitted_actions(&self) -> Vec<Self::ActionType> {
            self.injected_permitted_actions.clone()
        }
        fn next_actor(&self) -> Actor<Self::ActionType> {
            Actor::Player(0)
        }
        fn reward(&self) -> Vec<f64> {
            return self.injected_reward.clone()
        }

        fn terminal(&self) -> bool {
            return self.injected_terminal
        }

    }

    #[derive(Hash,Copy,  Clone, Eq, PartialEq)]
    enum TestGameAction{
        Win,
        Pass
    }
    impl Action for TestGameAction {
        type StateType = TestGameState;
        fn execute(&self, state: &Self::StateType) -> Self::StateType {
            state.clone()
        }
    }


    #[test]
    fn test_selection_basic(){
        let root = Node::Expanded(
            ExpandedNode {
            state:TestGameState{
                injected_reward: vec![0.0, 0.0],
                injected_terminal: false,
                injected_permitted_actions: vec![TestGameAction::Pass],
            },
            0},
        )
    }
}
 */
