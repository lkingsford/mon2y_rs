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

        let mut result_stack: Vec<(Option<ActionType>, Arc<RwLock<Node<StateType, ActionType>>>)> =
            vec![(None, self.root.clone())];

        loop {
            log::debug!("Result stack size {}", result_stack.len());
            let current = match result_stack.last() {
                Some(x) => x.clone(),
                None => {
                    log::warn!("Result stack is empty");
                    return Selection::FullyExplored;
                }
            };
            let node = current.1.clone();
            let expanded = {
                let node_read = node.read().unwrap();
                matches!(&*node_read, Node::Expanded { .. })
            };

            let best_pick = if expanded {
                let best_picks = super::node::best_pick(&node, self.constant);
                if best_picks.is_empty() {
                    log::warn!("Best picks is empty");
                    result_stack.pop();
                    continue;
                }
                best_picks[0].clone()
            } else {
                break;
            };
            // I don't like the borrow checker right now
            let next_node = {
                let node = result_stack.last().unwrap().1.read().unwrap();
                if let Node::Expanded { children, .. } = &*node {
                    children.get(&best_pick).unwrap().clone()
                } else {
                    break;
                }
            };

            result_stack.push((Some(best_pick), next_node.clone()));
        }

        Selection::Selection(result_stack.iter().filter_map(|x| x.0.clone()).collect())
    }

    pub fn expansion(
        &self,
        selection: &Selection<ActionType>,
    ) -> Vec<Arc<RwLock<Node<StateType, ActionType>>>> {
        trace!("Expansion: Selection: {:#?}", selection);
        let mut cur_node = self.root.clone();
        // This root is needed as part of the output to ensure that propagate can work
        // It was either here or selection. Could fit in either place.
        // Could also be in iterate, but that was going to result in more memory allocations.
        let mut result: Vec<Arc<RwLock<Node<StateType, ActionType>>>> = vec![self.root.clone()];

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
                        let cur_state = {
                            let node = cur_node.read().unwrap();
                            node.state().clone()
                        };

                        let expanded_child = {
                            let read_node = child_node.read().unwrap();
                            if let Node::Placeholder { .. } = &*read_node {
                                Some(read_node.expansion(*action, &cur_state))
                            } else {
                                None
                            }
                        };

                        if let Some(expanded_child) = expanded_child {
                            cur_node
                                .write()
                                .unwrap()
                                .insert_child(action.clone(), expanded_child);
                        }
                    }

                    result.push(cur_node);
                    cur_node = child_node;
                }
            }
        }
        result
    }

    pub fn play_out(&self, state: StateType) -> Vec<Reward> {
        let mut rng = rand::thread_rng();

        let mut cur_state = Box::new(state.clone());

        while !cur_state.terminal() {
            let permitted_actions = cur_state.permitted_actions();

            let action: ActionType =
                permitted_actions[rng.gen_range(0..permitted_actions.len())].clone();
            cur_state = Box::new(action.execute(&cur_state));
        }
        trace!("Reward is {:?}", cur_state.reward());
        cur_state.reward()
    }

    pub fn propagate_reward(
        &self,
        nodes: Vec<Arc<RwLock<Node<StateType, ActionType>>>>,
        reward: Vec<Reward>,
    ) {
        let mut previous_node = nodes[0].clone();
        for node in nodes[1..].iter() {
            {
                let actor = {
                    let read_previous = previous_node.read().unwrap();
                    if let Node::Expanded { .. } = &*read_previous {
                        read_previous.state().next_actor()
                    } else {
                        panic!("Attempting to propagate to a placeholder node");
                    }
                };

                let mut cur_node = node.write().unwrap();
                cur_node.visit(match actor {
                    Actor::Player(player_id) => *reward.get(player_id as usize).unwrap_or(&0.0),
                    _ => 0.0,
                })
            }
            previous_node = node.clone();
        }
    }

    pub fn iterate(&self) {
        let selection = self.selection();
        if let Selection::FullyExplored = selection {
            log::warn!("Iterate short circuited - fully explored");
            return;
        };
        let expanded_nodes = self.expansion(&selection);
        if let Selection::Selection(..) = selection {
            let reward = {
                self.play_out(
                    expanded_nodes
                        .last()
                        .unwrap()
                        .read()
                        .unwrap()
                        .state()
                        .clone(),
                )
            };
            self.propagate_reward(expanded_nodes, reward);
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
        let node_path = tree.root.clone();
        let node_ref = node_path.read().unwrap().get_node_by_path(selection_path);
        let node = node_ref.read().unwrap();
        if let Node::Expanded { children, .. } = &*node {
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
        let tree = Tree::new(root);
        let reward = tree.play_out(explored_state);

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
        let owned_root = tree.root.clone();
        // Todo: Think about ways to tidy this.
        let nodes = vec![
            tree.root.clone(),
            owned_root
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(2))
                .clone(),
            owned_root
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(2))
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(1))
                .clone(),
            owned_root
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(2))
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(1))
                .read()
                .unwrap()
                .get_child(TestGameAction::Win)
                .clone(),
        ];

        let check_path = path.clone();
        const REWARD: f64 = 0.8;
        tree.propagate_reward(nodes, vec![REWARD]);

        for path_i in 1..=check_path.len() {
            let semi_path = check_path[0..path_i].to_vec();
            let node_ref = tree.root.read().unwrap().get_node_by_path(semi_path);
            let node = node_ref.read().unwrap();
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
        let owned_root = tree.root.clone();
        // Not super pleased with this here either
        let nodes = vec![
            tree.root.clone(),
            owned_root
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(2))
                .clone(),
            owned_root
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(2))
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(1))
                .clone(),
            owned_root
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(2))
                .read()
                .unwrap()
                .get_child(TestGameAction::WinInXTurns(1))
                .read()
                .unwrap()
                .get_child(TestGameAction::Win)
                .clone(),
        ];

        let check_path = path.clone();
        // Using slightly unusual rewards to just make more certain that it was actually this reward
        const REWARD: f64 = 0.8;
        const LOSS_REWARD: f64 = -0.6;
        tree.propagate_reward(nodes, vec![REWARD, LOSS_REWARD]);

        for path_i in 1..=check_path.len() {
            // This isn't the greatest way to do this - maybe we should be just looking it up in a
            // table.
            let semi_path = check_path[0..path_i].to_vec();
            let player_id = (path_i + 1) % 2;
            let node_ref = tree.root.read().unwrap().get_node_by_path(semi_path);
            let node = node_ref.read().unwrap();
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
