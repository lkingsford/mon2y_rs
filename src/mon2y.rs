use core::panic;
use log::{debug, info, warn};
use std::{cmp::max, collections::HashMap, path};

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

const MIN_CHILD_VISIT: f64 = 0.00000000001;

type Reward = Vec<f64>;

pub trait State: Clone {
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

    fn visit(&mut self, reward: f64) {
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

    fn get_child_mut(&mut self, action: ActionType) -> &mut Node<StateType, ActionType> {
        if let Node::Expanded { children, .. } = self {
            children.get_mut(&action).unwrap()
        } else {
            panic!("Getting child from placeholder");
        }
    }

    fn get_child<'a>(&'a self, action: ActionType) -> &Node<StateType, ActionType> {
        if let Node::Expanded { children, .. } = self {
            children.get(&action).unwrap()
        } else {
            panic!("Getting child from placeholder");
        }
    }

    fn new_expanded(state: StateType) -> Node<StateType, <StateType as State>::ActionType> {
        create_expanded_node(state)
    }

    fn get_node_by_path(&self, path: Vec<ActionType>) -> &Node<StateType, ActionType> {
        let mut node = self;
        for action in path {
            node = node.get_child(action);
        }
        node
    }
}

fn create_expanded_node<StateType>(state: StateType) -> Node<StateType, StateType::ActionType>
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

#[derive(Debug, PartialEq)]
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
where
    StateType: State<ActionType = ActionType>,
    ActionType: Action<StateType = StateType>,
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
        let mut current_selection = &self.root;
        let mut result: Vec<ActionType> = vec![];
        while let Node::Expanded { children, .. } = current_selection {
            let best_picks = current_selection.best_pick(self.constant);
            let best_pick = best_picks[0].clone();
            result.push(best_pick);
            current_selection = children.get(&best_pick).unwrap();
        }
        Selection::Selection(result)
    }

    pub fn expansion(&mut self, selection: Selection<ActionType>) {
        let mut cur_node = &mut self.root;

        if let Selection::Selection(selection) = selection {
            for action in selection.iter() {
                let cur_node_state = &cur_node.state();

                if let Node::Expanded { .. } = cur_node {
                    let child_node = cur_node.get_child(action.clone());

                    if let Node::Placeholder = child_node {
                        let expanded_child = child_node.expansion(*action, &cur_node_state);
                        cur_node.insert_child(action.clone(), expanded_child);
                    }
                }

                // Move to the child node after the borrow ends
                if let Node::Expanded { children, .. } = cur_node {
                    cur_node = children.get_mut(action).expect("Child must exist");
                } else {
                    panic!("Expected an Expanded node");
                }
            }
        }
    }

    pub fn play_out(&self, selection_path: Vec<ActionType>) -> Option<Reward> {
        let node = self.root.get_node_by_path(selection_path);
        let mut rng = rand::thread_rng();

        if let Node::Expanded { state, .. } = node {
            let mut cur_state = Box::new(state.clone());

            while !cur_state.terminal() {
                let permitted_actions = cur_state.permitted_actions();

                let action: ActionType =
                    permitted_actions[rng.gen_range(0..permitted_actions.len())].clone();
                cur_state = Box::new(action.execute(&cur_state));
            }

            Some(cur_state.reward())
        } else {
            panic!("Expected an expanded node");
        }
    }

    pub fn propagate_reward(&mut self, selection: Selection<ActionType>, reward: Reward) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestGameState {
        injected_reward: Vec<f64>,
        injected_terminal: bool,
        injected_permitted_actions: Vec<TestGameAction>,
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
            return self.injected_reward.clone();
        }

        fn terminal(&self) -> bool {
            return self.injected_terminal;
        }
    }

    #[derive(Hash, Copy, Clone, Eq, PartialEq, Debug)]
    enum TestGameAction {
        Win,
        WinInXTurns(u8),
        NextTurnInjectActionCount(u8),
    }
    impl Action for TestGameAction {
        type StateType = TestGameState;
        fn execute(&self, state: &Self::StateType) -> Self::StateType {
            match self {
                TestGameAction::NextTurnInjectActionCount(c) => TestGameState {
                    injected_permitted_actions: (0..*c)
                        .map(|i| TestGameAction::WinInXTurns(i))
                        .collect(),
                    ..state.clone()
                },
                TestGameAction::WinInXTurns(turns) => TestGameState {
                    injected_permitted_actions: {
                        if (*turns > 0) {
                            vec![TestGameAction::WinInXTurns(turns - 1)]
                        } else {
                            vec![TestGameAction::Win]
                        }
                    },
                    ..state.clone()
                },
                TestGameAction::Win => TestGameState {
                    injected_terminal: true,
                    injected_reward: vec![1.0],
                    ..state.clone()
                },
                _ => state.clone(),
            }
        }
    }

    ///
    /// Test that selection returns the unexplored path at the next node
    ///
    #[test]
    fn test_selection_basic() {
        let root_state = TestGameState {
            injected_reward: vec![0.0],
            injected_terminal: false,
            injected_permitted_actions: vec![
                TestGameAction::WinInXTurns(2),
                TestGameAction::WinInXTurns(3),
            ],
        };

        let explored_state = TestGameAction::WinInXTurns(2).execute(&root_state);
        let mut root = create_expanded_node(root_state);

        let mut explored_node = create_expanded_node(explored_state);
        explored_node.visit(0.0f64);

        root.insert_child(TestGameAction::WinInXTurns(2), explored_node);
        root.insert_child(TestGameAction::WinInXTurns(3), Node::Placeholder);
        root.visit(0.0f64);
        let tree = Tree::new(root);

        assert_eq!(
            tree.selection(),
            Selection::Selection(vec![TestGameAction::WinInXTurns(3)])
        );
    }

    ///
    /// Test that selection returns the unexplored path at the next node
    ///
    #[test]
    fn test_selection_multiple_expanded() {
        let root_state = TestGameState {
            injected_reward: vec![0.0],
            injected_terminal: false,
            injected_permitted_actions: vec![
                TestGameAction::WinInXTurns(2),
                TestGameAction::WinInXTurns(3),
            ],
        };

        let mut explored_state_1 = TestGameAction::WinInXTurns(2).execute(&root_state);
        explored_state_1.injected_permitted_actions = vec![TestGameAction::WinInXTurns(1)];
        let explored_state_2 = TestGameAction::WinInXTurns(3).execute(&root_state);
        let mut root = create_expanded_node(root_state);

        let mut explored_node_1 = create_expanded_node(explored_state_1);
        explored_node_1.visit(0.0f64);
        explored_node_1.insert_child(TestGameAction::WinInXTurns(1), Node::Placeholder);

        let mut explored_node_2 = create_expanded_node(explored_state_2);
        explored_node_2.visit(-1.0f64);
        explored_node_2.visit(0.0f64);

        root.insert_child(TestGameAction::WinInXTurns(2), explored_node_1);
        root.insert_child(TestGameAction::WinInXTurns(3), explored_node_2);
        root.visit(0.0f64);
        root.visit(0.0f64);
        root.visit(0.0f64);
        let tree = Tree::new(root);

        assert_eq!(
            tree.selection(),
            Selection::Selection(vec![
                TestGameAction::WinInXTurns(2),
                TestGameAction::WinInXTurns(1)
            ])
        );
    }

    #[test]
    fn test_expansion_basic() {
        let root_state = TestGameState {
            injected_reward: vec![0.0],
            injected_terminal: false,
            injected_permitted_actions: vec![
                TestGameAction::WinInXTurns(2),
                TestGameAction::WinInXTurns(3),
            ],
        };
        let mut explored_state_1 = TestGameAction::WinInXTurns(2).execute(&root_state);
        explored_state_1.injected_permitted_actions =
            vec![TestGameAction::NextTurnInjectActionCount(5)];

        let explored_state_2 = TestGameAction::WinInXTurns(3).execute(&root_state);
        let mut root = create_expanded_node(root_state);

        let mut explored_node_1 = create_expanded_node(explored_state_1);
        explored_node_1.visit(0.0f64);
        explored_node_1.insert_child(
            TestGameAction::NextTurnInjectActionCount(5),
            Node::Placeholder,
        );

        let mut explored_node_2 = create_expanded_node(explored_state_2);
        explored_node_2.visit(-1.0f64);
        explored_node_2.visit(0.0f64);

        root.insert_child(TestGameAction::WinInXTurns(2), explored_node_1);
        root.insert_child(TestGameAction::WinInXTurns(3), explored_node_2);

        let selection_path = vec![
            TestGameAction::WinInXTurns(2),
            TestGameAction::NextTurnInjectActionCount(5),
        ];
        let selection = Selection::Selection(selection_path.clone());

        let mut tree = Tree::new(root);
        tree.expansion(selection);
        let node = tree.root.get_node_by_path(selection_path);
        if let Node::Expanded { children, .. } = node {
            assert_eq!(children.len(), 5);
        } else {
            self::panic!("Node is not expanded");
        }
    }

    #[test]
    fn test_play_out() {
        let root_state = TestGameState {
            injected_reward: vec![0.0],
            injected_terminal: false,
            injected_permitted_actions: vec![TestGameAction::WinInXTurns(3)],
        };

        let explored_state = TestGameAction::WinInXTurns(2).execute(&root_state);
        let mut root = create_expanded_node(root_state);

        let mut explored_node = create_expanded_node(explored_state);
        explored_node.visit(0.0f64);

        root.insert_child(TestGameAction::WinInXTurns(2), explored_node);
        root.insert_child(TestGameAction::WinInXTurns(3), Node::Placeholder);
        root.visit(0.0f64);
        let tree = Tree::new(root);

        let selection_path = vec![TestGameAction::WinInXTurns(2)];
        let reward = tree.play_out(selection_path);

        assert_eq!(reward, Some(vec![1.0]));
    }
}
