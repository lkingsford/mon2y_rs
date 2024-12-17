use super::game::{Action, Actor, State};
use super::node::{create_expanded_node, Node};
use super::BestTurnPolicy;
use super::Reward;
use core::panic;
use log::{debug, trace};
use rand::Rng;
use std::sync::{Arc, RwLock};

#[derive(Debug, PartialEq)]
pub enum Selection<ActionType: Action> {
    FullyExplored,
    Selection(Vec<ActionType>),
}

pub struct Tree<StateType: State, ActionType: Action<StateType = StateType>> {
    pub root: Arc<RwLock<Node<StateType, ActionType>>>,
    constant: f64,
}

impl<StateType: State<ActionType = ActionType>, ActionType: Action<StateType = StateType>>
    Tree<StateType, ActionType>
where
    StateType: State<ActionType = ActionType>,
    ActionType: Action<StateType = StateType>,
{
    fn node_ref(root: Node<StateType, ActionType>) -> Arc<RwLock<Node<StateType, ActionType>>> {
        // Only doing this to keep it a little tidier
        Arc::new(RwLock::new(root))
    }

    pub fn new(root: Node<StateType, ActionType>) -> Tree<StateType, ActionType> {
        Tree {
            root: Tree::node_ref(root),
            constant: 2.0_f64.sqrt(),
        }
    }

    ///
    /// Returns a path to the current selection
    ///
    pub fn selection(&self) -> Selection<ActionType> {
        {
            let root = self.root.read().unwrap();
            if root.fully_explored() {
                return Selection::FullyExplored;
            }
            if let Node::Placeholder = *root {
                return Selection::Selection(vec![]);
            }
        }

        let mut current_selection = self.root.clone();
        let mut result: Vec<ActionType> = vec![];

        loop {
            let best_pick = {
                let node = current_selection.read().unwrap();
                if let Node::Expanded { .. } = &*node {
                    let best_picks = node.best_pick(self.constant);
                    best_picks[0].clone()
                } else {
                    break;
                }
            };
            // I don't like the borrow checker right now
            let next_node = {
                let node = current_selection.read().unwrap();
                if let Node::Expanded { children, .. } = &*node {
                    children.get(&best_pick).unwrap().clone()
                } else {
                    break;
                }
            };

            result.push(best_pick);
            current_selection = next_node;
        }

        Selection::Selection(result)
    }

    pub fn expansion(&self, selection: &Selection<ActionType>) {
        trace!("Expansion: Selection: {:#?}", selection);
        let mut cur_node = self.root.clone();

        if let Selection::Selection(selection) = selection {
            for action in selection.iter() {
                {
                    let child_node = {
                        let node = cur_node.read().unwrap();
                        if let Node::Expanded { .. } = &*node {
                            node.get_child(action.clone()).clone()
                        } else {
                            continue;
                        }
                    };

                    {
                        let mut write_node = child_node.write().unwrap();
                        let cur_state = write_node.state();
                        let expanded_child = write_node.expansion(*action, &cur_state);
                        write_node.insert_child(action.clone(), expanded_child);
                    }
                    cur_node = child_node;
                }
            }
        }
    }

    pub fn play_out(&self, selection_path: Vec<ActionType>) -> Vec<Reward> {
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
            trace!("Reward is {:?}", cur_state.reward());
            cur_state.reward()
        } else {
            panic!("Expected an expanded node");
        }
    }

    pub fn propagate_reward(&mut self, selection_path: Vec<ActionType>, reward: Vec<Reward>) {
        let mut cur_node = &mut self.root;
        // Reward doesn't matter for root
        cur_node.visit(0.0);
        for action in selection_path {
            let actor = cur_node.state().next_actor();
            cur_node = cur_node.get_child_mut(action);
            cur_node.visit(match actor {
                Actor::Player(player_id) => *reward.get(player_id as usize).unwrap_or(&0.0),
                _ => 0.0,
            });
        }
    }

    pub fn iterate(&mut self) {
        let selection = self.selection();
        if let Selection::FullyExplored = selection {
            return;
        };
        self.expansion(&selection);
        if let Selection::Selection(selection_path) = selection {
            let reward = self.play_out(selection_path.clone());
            trace!("Before propagate");
            if log::log_enabled!(log::Level::Trace) {
                self.root.best_pick(self.constant);
            }
            self.propagate_reward(selection_path, reward);
            trace!("After propagate");
            if log::log_enabled!(log::Level::Trace) {
                self.root.best_pick(self.constant);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[derive(Clone, Debug)]
    struct TestGameState {
        injected_reward: Vec<f64>,
        injected_terminal: bool,
        injected_permitted_actions: Vec<TestGameAction>,
        player_count: u8,
        next_player_id: u8,
    }

    impl State for TestGameState {
        type ActionType = TestGameAction;
        fn permitted_actions(&self) -> Vec<Self::ActionType> {
            self.injected_permitted_actions.clone()
        }
        fn next_actor(&self) -> Actor<Self::ActionType> {
            Actor::Player(self.next_player_id)
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
            let next_player_id = if let Actor::Player(player_id) = state.next_actor() {
                (player_id + 1) % state.player_count
            } else {
                self::panic!("Not a player");
            };
            match self {
                TestGameAction::NextTurnInjectActionCount(c) => TestGameState {
                    injected_permitted_actions: (0..*c)
                        .map(|i| TestGameAction::WinInXTurns(i))
                        .collect(),
                    next_player_id,
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
                    next_player_id,
                    ..state.clone()
                },
                TestGameAction::Win => TestGameState {
                    injected_terminal: true,
                    injected_reward: vec![1.0],
                    next_player_id,
                    ..state.clone()
                },
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
            player_count: 1,
            next_player_id: 0,
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
            player_count: 1,
            next_player_id: 0,
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
            player_count: 1,
            next_player_id: 0,
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
        tree.expansion(&selection);
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
            player_count: 1,
            next_player_id: 0,
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

        assert_eq!(reward, vec![1.0]);
    }

    #[test]
    fn test_propagate_one_player() {
        let root_state = TestGameState {
            injected_reward: vec![0.0],
            injected_terminal: false,
            injected_permitted_actions: vec![
                TestGameAction::WinInXTurns(2),
                TestGameAction::WinInXTurns(3),
            ],
            player_count: 1,
            next_player_id: 0,
        };

        let explored_state = TestGameAction::WinInXTurns(2).execute(&root_state);
        let mut root = create_expanded_node(root_state);

        let mut explored_node = create_expanded_node(explored_state);

        let mut child_node =
            create_expanded_node(TestGameAction::WinInXTurns(1).execute(&explored_node.state()));

        let grandchild_state = TestGameAction::Win.execute(&child_node.state());
        let grandchild_node = create_expanded_node(grandchild_state);

        child_node.insert_child(TestGameAction::Win, grandchild_node);
        explored_node.insert_child(TestGameAction::WinInXTurns(1), child_node);
        root.insert_child(TestGameAction::WinInXTurns(2), explored_node);
        let mut tree = Tree::new(root);

        let path = vec![
            TestGameAction::WinInXTurns(2),
            TestGameAction::WinInXTurns(1),
            TestGameAction::Win,
        ];
        let check_path = path.clone();
        const REWARD: f64 = 0.8;
        tree.propagate_reward(path, vec![REWARD]);

        for path_i in 1..=check_path.len() {
            let semi_path = check_path[0..path_i].to_vec();
            let node = tree.root.get_node_by_path(semi_path);
            assert_eq!(node.value_sum(), REWARD);
            assert_eq!(node.visit_count(), 1);
        }
    }

    #[test]
    fn test_propagate_two_players() {
        let root_state = TestGameState {
            injected_reward: vec![0.0],
            injected_terminal: false,
            injected_permitted_actions: vec![
                TestGameAction::WinInXTurns(2),
                TestGameAction::WinInXTurns(3),
            ],
            player_count: 2,
            next_player_id: 0,
        };

        let explored_state = TestGameAction::WinInXTurns(2).execute(&root_state);
        let mut root = create_expanded_node(root_state);

        let mut explored_node = create_expanded_node(explored_state);

        let mut child_node =
            create_expanded_node(TestGameAction::WinInXTurns(1).execute(&explored_node.state()));

        let grandchild_state = TestGameAction::Win.execute(&child_node.state());
        let grandchild_node = create_expanded_node(grandchild_state);

        child_node.insert_child(TestGameAction::Win, grandchild_node);
        explored_node.insert_child(TestGameAction::WinInXTurns(1), child_node);
        root.insert_child(TestGameAction::WinInXTurns(2), explored_node);
        let mut tree = Tree::new(root);

        let path = vec![
            TestGameAction::WinInXTurns(2),
            TestGameAction::WinInXTurns(1),
            TestGameAction::Win,
        ];
        let check_path = path.clone();
        // Using slightly unusual rewards to just make more certain that it was actually this reward
        const REWARD: f64 = 0.8;
        const LOSS_REWARD: f64 = -0.6;
        tree.propagate_reward(path, vec![REWARD, LOSS_REWARD]);

        for path_i in 1..=check_path.len() {
            // This isn't the greatest way to do this - maybe we should be just looking it up in a
            // table.
            let semi_path = check_path[0..path_i].to_vec();
            let player_id = (path_i + 1) % 2;
            let node = tree.root.get_node_by_path(semi_path);
            if player_id == 0 {
                assert_eq!(node.value_sum(), REWARD);
                assert_eq!(node.visit_count(), 1);
            } else {
                assert_eq!(node.value_sum(), LOSS_REWARD);
                assert_eq!(node.visit_count(), 1);
            }
        }
    }
}
