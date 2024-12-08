use super::game::{Action, State};
use super::node::create_expanded_node;
use super::tree::Tree;
use super::BestTurnPolicy;

pub fn calculate_best_turn<StateType>(
    iterations: usize,
    state: StateType,
    policy: BestTurnPolicy,
) -> <StateType as State>::ActionType
where
    StateType: State,
{
    let mut root_node = create_expanded_node(state);
    let mut tree = Tree::new(root_node);
    tree.calculate_best_turn(iterations, policy)
}
