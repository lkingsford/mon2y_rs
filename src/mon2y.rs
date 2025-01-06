pub mod game;
mod mcts;
pub use mcts::calculate_best_turn;
pub mod node;
pub mod tree;
use clap::ValueEnum;
use serde::Deserialize;

pub type Reward = f64;

#[derive(Debug, Clone, Copy, ValueEnum, Deserialize)]
pub enum BestTurnPolicy {
    MostVisits,
    Ucb0,
}

impl std::fmt::Display for BestTurnPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BestTurnPolicy::MostVisits => write!(f, "MostVisits"),
            BestTurnPolicy::Ucb0 => write!(f, "Ucb0"),
        }
    }
}
