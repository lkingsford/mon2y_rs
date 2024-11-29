mod c4;
mod game;
mod mon2y;

//use crate::mon2y::action_log::{Action, ActionLogEntry};
use c4::C4;
use clap::{Parser, ValueEnum};
use game::Game;
use mon2y::{Action, Actor, State};
use rand::Rng;

#[derive(Debug, Clone, ValueEnum)]
enum Games {
    C4,
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

fn run_game<G: Game>(game: G, players: Vec<PlayerType>) {
    let mut state = game.init_game();

    while !state.terminal() {
        let actor = state.next_actor();
        match actor {
            Actor::Player(player) => {
                let action: G::ActionType = match players.get(player as usize) {
                    Some(PlayerType::H) => game.get_human_turn(&state),
                    Some(PlayerType::R) => {
                        let permitted_actions = state.permitted_actions();
                        permitted_actions[rand::thread_rng().gen_range(0..permitted_actions.len())]
                    }
                    _ => todo!(),
                };
                state = action.execute(&state);
            }
            Actor::GameAction(action) => {
                state = action.execute(&state);
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let players = args.players;

    match args.game {
        Games::C4 => {
            run_game(C4, players);
        }
    }
}
