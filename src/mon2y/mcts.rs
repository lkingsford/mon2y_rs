use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use log::{debug, trace};

use crate::mon2y::game::Actor;
use crate::mon2y::tree::Selection;

use super::game::{Action, State};
use super::node::{create_expanded_node, Node};
use super::tree::Tree;
use super::BestTurnPolicy;

/// Run multiple iterations of the MCTS algorithm on a state.
pub fn calculate_best_turn<
    'a,
    StateType: State<ActionType = ActionType> + Sync + Send + 'static,
    ActionType: Action<StateType = StateType> + Sync + Send + 'static,
>(
    iterations: usize,
    time_limit: Option<std::time::Duration>,
    thread_count: usize,
    state: StateType,
    policy: BestTurnPolicy,
) -> <StateType as State>::ActionType
where
    StateType: State<ActionType = ActionType>,
    ActionType: Action<StateType = StateType>,
{
    log::info!("Starting next turn");
    let root_node = create_expanded_node(state);
    let tree = Arc::new(Tree::new(root_node));
    let mut threads = vec![];

    let finished_iterations: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    for _ in 0..thread_count {
        let tree_clone = Arc::clone(&tree);
        let finished_iterations_clone: Arc<AtomicUsize> = Arc::clone(&finished_iterations);
        let time_started = std::time::Instant::now();
        threads.push(std::thread::spawn(move || loop {
            {
                trace!(
                    "Starting iteration {}",
                    finished_iterations_clone.load(std::sync::atomic::Ordering::SeqCst)
                );
                let result = tree_clone.iterate();
                let current_iterations =
                    finished_iterations_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                trace!("Finished iteration {}", current_iterations);
                if current_iterations >= iterations
                    || result == Selection::FullyExplored
                    || time_started.elapsed() > time_limit.unwrap_or(std::time::Duration::MAX)
                {
                    break;
                }
            }
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }

    log::info!(
        "Completed {} iterations",
        finished_iterations.load(std::sync::atomic::Ordering::SeqCst)
    );

    if log::log_enabled!(log::Level::Trace) {
        tree.root.clone().read().unwrap().trace_log_children(0);
    }
    let root_ref = tree.root.clone();
    let root = root_ref.read().unwrap();

    match policy {
        BestTurnPolicy::MostVisits => {
            if let Node::Expanded { children, .. } = &*root {
                log::info!(
                    "Action, Visits, Value: {:?}",
                    children
                        .iter()
                        .map(|(action, node)| {
                            let node = node.read().unwrap();
                            (action.clone(), node.visit_count(), node.value_sum())
                        })
                        .collect::<Vec<_>>()
                );
                // Short circuit on a winning move
                // Implemented because (I think) the UCB formula doesn't end up prioritizing
                // certainly winning moves, because they're already explored. Dunno if this
                // is a cludge though.
                let winning_moves: Vec<ActionType> = children
                    .iter()
                    .filter_map(|(action, node)| {
                        let node_ref = node.clone();
                        let node = node_ref.read().unwrap();
                        if let Node::Placeholder = &*node {
                            return None;
                        }
                        if node.state().terminal() {
                            let actor = root.state().next_actor();
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
                    .max_by_key(|(_, node)| node.read().unwrap().visit_count())
                    .unwrap()
                    .0
                    .clone()
            } else {
                panic!("Expected root to be an expanded node")
            }
        }
    }
}
