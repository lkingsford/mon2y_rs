//! Plays configurations of the MCTS against one another
mod game;
mod games;
mod mon2y;

use clap::Parser;
use env_logger::fmt::Formatter;
use game::Game;
use games::Games;
use games::{C4, NT};
use log::Record;
use mon2y::game::{Action, Actor, State};
use mon2y_rs::mon2y::{calculate_best_turn, BestTurnPolicy};
use rand::Rng;
use serde::Deserialize;
use std::{fs, thread};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg()]
    config_file: String,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Debug, Deserialize)]
struct ArenaSettings {
    game: Games,
    episodes: usize,
    players: Vec<PlayerSettings>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum PlayerSettings {
    Random,
    Mcts(MctsSettings),
}

#[derive(Debug, Deserialize)]
struct MctsSettings {
    policy: BestTurnPolicy,
    exploration_constant: f64,
    iterations: usize,
    time_limit: Option<f32>,
    threads: usize,
}

fn run_episode<G: Game>(game: G, players: Vec<PlayerSettings>) -> Vec<f64> {
    let mut state = game.init_game();
    while !state.terminal() {
        let actor = state.next_actor();
        match actor {
            Actor::Player(player) => {
                let player = &players[player as usize];
                let action: G::ActionType = match player {
                    PlayerSettings::Random => {
                        let permitted_actions = state.permitted_actions();
                        permitted_actions[rand::thread_rng().gen_range(0..permitted_actions.len())]
                    }
                    PlayerSettings::Mcts(mcts_settings) => calculate_best_turn(
                        mcts_settings.iterations,
                        match mcts_settings.time_limit {
                            None => None,
                            Some(time_limit) => {
                                Some(std::time::Duration::from_secs_f32(time_limit))
                            }
                        },
                        mcts_settings.threads,
                        state.clone(),
                        mcts_settings.policy,
                        mcts_settings.exploration_constant,
                        false,
                    ),
                };
                state = action.execute(&state);
            }
            Actor::GameAction(actions) => {
                let action = actions[rand::thread_rng().gen_range(0..actions.len())].0;
                state = action.execute(&state);
            }
        }
    }
    state.reward()
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

    // Open the config file
    let config_file = fs::read_to_string(&args.config_file).expect("Failed to read config file");
    let arena_settings: ArenaSettings =
        serde_json::from_str(&config_file).expect("Failed to parse config file");

    let mut results = vec![0.0; arena_settings.players.len()];
    for episode_count in 0..arena_settings.episodes {
        log::info!("Starting episode {}", episode_count);
        let episode_result = match arena_settings.game {
            Games::C4 => run_episode(games::C4, arena_settings.players),
            Games::NT => run_episode(
                games::NT {
                    player_count: arena_settings.players.len() as u8,
                },
                arena_settings.players,
            ),
        };
        for (i, result) in episode_result.iter().enumerate() {
            results[i] += *result;
        }
    }
}
