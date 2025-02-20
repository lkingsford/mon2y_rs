use std::sync::atomic::{self, AtomicUsize};

use log::trace;

use crate::mon2y::annotation::Annotation;
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
    exploration_constant: f64,
    log_children: bool,
    annotate: bool,
) -> (<StateType as State>::ActionType, Vec<Annotation>)
where
    StateType: State<ActionType = ActionType>,
    ActionType: Action<StateType = StateType>,
{
    log::debug!("Starting next turn");
    let root_node = create_expanded_node(state, None);
    if let Node::Expanded { children, .. } = &root_node {
        if children.len() == 1 {
            log::debug!("Short circuited - only one option");
            return( children.keys().next().unwrap().clone(), vec![])
        }
    }

    let tree = Tree::new_with_constant(root_node, exploration_constant);
    let finished_iterations = AtomicUsize::new(0);
    let annotations: Vec<Annotation> = vec![];

    let results =
        std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(thread_count);
        (0..thread_count)
            .map(|_| {
                scope.spawn(|| -> Vec<Annotation> {
                    let mut annotations: Vec<Annotation> = vec![];
                    loop {
                        let time_started = std::time::Instant::now();
                        trace!(
                            "Starting iteration {}",
                            finished_iterations.load(atomic::Ordering::SeqCst)
                        );
                        let (result, annotation) = tree.iterate();
                        if annotate {
                        if let Some(annotation) = annotation {
                            annotations.push(annotation)
                        }};
                        let current_iterations =
                            finished_iterations.fetch_add(1, atomic::Ordering::SeqCst);
                        trace!("Finished iteration {}", current_iterations);
                        if current_iterations >= iterations
                            || result == Selection::FullyExplored
                            || time_started.elapsed()
                                > time_limit.unwrap_or(std::time::Duration::MAX)
                        {
                            break;
                        }
                    }
                    annotations
                })
            })
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    });

    log::debug!(
        "Completed {} iterations",
        finished_iterations.load(std::sync::atomic::Ordering::SeqCst)
    );

    if log::log_enabled!(log::Level::Trace) || log_children {
        tree.root.clone().read().unwrap().log_children(0);
    }
    let root_ref = tree.root.clone();

    match policy {
        BestTurnPolicy::Ucb0 => {
            let node = root_ref.read().unwrap();
            // This bit of logic is reimplemented due to crashing when tree is fully explored
            let mut picks = match &*node {
                Node::Expanded { children, .. } => children
                    .iter()
                    // TODO: Add random factor
                    .map(|(action, child)| {
                        let child = child.read().unwrap();
                        (
                            action.clone(),
                            child.value_sum()
                                / if child.visit_count() > 0 {
                                    child.visit_count() as f64
                                } else {
                                    f64::INFINITY
                                },
                        )
                    })
                    .collect::<Vec<_>>(),
                _ => panic!("Root should be parent"),
            };
            picks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            log::debug!("Action, UCB0: {:?}", picks);
            (picks[0].0.clone(), annotations)
        }

        BestTurnPolicy::MostVisits => {
            let root = root_ref.read().unwrap();
            if let Node::Expanded { children, .. } = &*root {
                log::debug!(
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
                        if let Node::Placeholder { .. } = &*node {
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
                                        return (Some(action.clone()), annotations)
                                    }
                                }
                            }
                        }
                        None
                    })
                    .collect();
                if let Some(action) = winning_moves.first() {
                    return (action.clone(), annotations)
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
