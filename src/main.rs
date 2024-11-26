mod c4;
mod mon2y;

//use crate::mon2y::action_log::{Action, ActionLogEntry};
use clap::{Parser, ValueEnum};
use mon2y::{action::Action, state::State};

#[derive(Debug, Clone, ValueEnum)]
enum Games {
    C4,
}

fn parse_players(s: &str) -> Result<Vec<PlayerType>, String> {
    s.split(',')
        .map(|part| PlayerType::from_str(part, true))
        .collect()
}

impl Games {
    fn init(&self) -> mon2y::node::ActResponse {
        match self {
            Games::C4 => c4::c4game::init_game(),
        }
    }

    fn get_human_turn(&self, state: &dyn State) -> Action {
        match self {
            Games::C4 => c4::c4game::get_human_turn(state),
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum PlayerType {
    H,
    R,
    M,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg()]
    game: Games,

    /// Players participating in the game
    #[arg(short, long, value_delimiter = ',', value_enum)]
    players: Vec<PlayerType>,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let mut last_response = args.game.init();

    while !last_response.terminated {
        last_response = (last_response.next_act_fn)(
            &*last_response.state,
            args.game.get_human_turn(&*last_response.state),
        );
    }
}
