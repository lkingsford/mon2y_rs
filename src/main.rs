mod c4;
mod mon2y;

//use crate::mon2y::action_log::{Action, ActionLogEntry};
use clap::{Parser, Subcommand, ValueEnum};
use std::str::FromStr;

#[derive(Subcommand, Debug, Clone)]
enum Games {
    C4,
}

impl Games {
    fn init(&self) -> impl mon2y::state::State {
        match self {
            Games::C4 => c4::c4game::init_game(),
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
enum PlayerType {
    Human,
    Random,
    Mcts,
}

impl FromStr for PlayerType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" | "h" => Ok(PlayerType::Human),
            "random" | "r" => Ok(PlayerType::Random),
            "mcts" | "m" => Ok(PlayerType::Mcts),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    game: Games,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[arg(long, value_enum)]
    players: Vec<PlayerType>,
}

fn main() {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
}
