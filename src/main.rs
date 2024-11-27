mod c4;
mod mon2y;

//use crate::mon2y::action_log::{Action, ActionLogEntry};
use clap::{Parser, ValueEnum};
use mon2y::{action::Action, state::State};
use rand::Rng;

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

    let players = args.players;

    while !last_response.terminated {
        if let Some(next_act_fn) = last_response.next_act_fn {
            let state = &*last_response.state;
            let action = if let Some(player_index) = last_response.next_player {
                match players.get(player_index as usize) {
                    Some(PlayerType::H) => args.game.get_human_turn(state),
                    Some(PlayerType::R) => last_response.permitted_actions
                        [rand::thread_rng().gen_range(0..last_response.permitted_actions.len())]
                    .clone(),
                    _ => todo!(),
                }
            } else {
                Action::NoAct
            };
            last_response = next_act_fn(state, action);
        }
    }
}
