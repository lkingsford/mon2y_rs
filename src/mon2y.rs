pub mod game;
mod mcts;
pub use mcts::calculate_best_turn;
pub mod node;
pub mod tree;

pub type Reward = f64;

#[derive(Debug, Clone)]
pub enum BestTurnPolicy {
    MostVisits,
}
