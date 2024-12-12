use log::{debug, trace};

use crate::mon2y::game::Actor;

use super::game::{Action, State};
use super::node::{create_expanded_node, Node};
use super::tree::Tree;
use super::BestTurnPolicy;

pub fn calculate_best_turn<
    StateType: State<ActionType = ActionType>,
    ActionType: Action<StateType = StateType>,
>(
    iterations: usize,
    state: StateType,
    policy: BestTurnPolicy,
) -> <StateType as State>::ActionType
where
    StateType: State<ActionType = ActionType>,
    ActionType: Action<StateType = StateType>,
{
    let mut root_node = create_expanded_node(state);
    let mut tree = Tree::new(root_node);

    for iteration in 0..iterations {
        trace!("Starting iteration {}", iteration);
        &tree.iterate();
    }
    if log::log_enabled!(log::Level::Trace) {
        tree.root.trace_log_children(0);
    }
    match policy {
        BestTurnPolicy::MostVisits => {
            if let Node::Expanded { children, .. } = &tree.root {
                debug!(
                    "Action, Visits, Value: {:?}",
                    children
                        .iter()
                        .map(|(action, node)| (
                            action.clone(),
                            node.visit_count(),
                            node.value_sum()
                        ))
                        .collect::<Vec<_>>()
                );
                // Short circuit on a winning move
                // Implemented because (I think) the UCB formula doesn't end up prioritizing
                // certainly winning moves, because they're already explored. Dunno if this
                // is a cludge though.
                let winning_moves: Vec<ActionType> = children
                    .iter()
                    .filter_map(|(action, node)| {
                        if node.state().terminal() {
                            let actor = tree.root.state().next_actor();
                            if let Actor::Player(player_id) = actor {
                                if let Some((index, _)) =
                                    // Annoying - but necessary because I was dumb enough to use f64
                                    // (otherwise, it'd be max_by_key)
                                    node.state().reward().iter().enumerate().max_by(
                                            |(_, a), (_, b)| {
                                                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)
                                            },
                                        )
                                {
                                    if index == player_id as usize {
                                        return Some(action.clone());
                                    }
                                }
                            }
                        }
                        None
                    })
                    .collect();
                if let Some(action) = winning_moves.first() {
                    return action.clone();
                }

                children
                    .iter()
                    .max_by_key(|(_, node)| node.visit_count())
                    .unwrap()
                    .0
                    .clone()
            } else {
                panic!("Expected root to be an expanded node")
            }
        }
    }
}
