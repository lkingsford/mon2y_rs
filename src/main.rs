mod c4;
mod game;
mod games;
mod mon2y;

//use crate::mon2y::action_log::{Action, ActionLogEntry};
use c4::C4;
use clap::{Parser, ValueEnum};
use env_logger::{fmt::Formatter, Builder};
use game::Game;
use games::Games;
use log::{Level, Record};
use mon2y::game::{Action, Actor, State};
use mon2y::{calculate_best_turn, BestTurnPolicy};
use std::io::Write;
use std::thread;

use rand::Rng;

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
    #[arg(short, long)]
    iterations: usize,
    #[arg(short, long, default_value_t = 1)]
    episodes: usize,
    #[arg(short, long, default_value_t = 4)]
    threads: usize,
}

fn run_game<G: Game>(game: G, players: Vec<PlayerType>, iterations: usize, threads: usize) {
    let mut state = game.init_game();
    while !state.terminal() {
        let actor = state.next_actor();
        game.visualise_state(&state);
        match actor {
            Actor::Player(player) => {
                let action: G::ActionType = match players.get(player as usize) {
                    Some(PlayerType::H) => game.get_human_turn(&state),
                    Some(PlayerType::R) => {
                        let permitted_actions = state.permitted_actions();
                        permitted_actions[rand::thread_rng().gen_range(0..permitted_actions.len())]
                    }
                    Some(PlayerType::M) => calculate_best_turn(
                        iterations,
                        threads,
                        state.clone(),
                        BestTurnPolicy::MostVisits,
                    ),
                    _ => todo!(),
                };
                log::info!("Player {} plays {:?}", player, action);
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
        .format(|buf: &mut Formatter, record: &Record| {
            let thread_id = thread::current().id();
            let timestamp = buf.timestamp_millis();
            writeln!(
                buf,
                "[{}] [Thread: {:?}] [{}] - {}",
                timestamp,
                thread_id,
                record.level(),
                record.args()
            )
        })
        .filter_level(args.verbose.log_level_filter())
        .init();

    let players = args.players;

    for _ in 0..args.episodes {
        match args.game {
            Games::C4 => {
                run_game(C4, players.clone(), args.iterations, args.threads);
            }
        }
    }
}
