//! Plays configurations of the MCTS against one another
mod game;
mod games;
mod mon2y;

//use crate::mon2y::action_log::{Action, ActionLogEntry};
use clap::{Parser, ValueEnum};
use env_logger::{fmt::Formatter, Builder};
use game::Game;
use games::Games;
use games::{C4, NT};
use log::{Level, Record};
use mon2y::game::{Action, Actor, State};
use mon2y::{calculate_best_turn, BestTurnPolicy};
use rand::Rng;
use serde::Deserialize;
use std::io::Write;
use std::{fs, io, thread};

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

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
enum PlayerSettings {
    Random,
    Mcts(MctsSettings),
}

#[derive(Debug, Deserialize, Clone)]
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
                let action: G::ActionType = match players.get(player as usize) {
                    Some(PlayerSettings::Random) => {
                        let permitted_actions = state.permitted_actions();
                        permitted_actions[rand::thread_rng().gen_range(0..permitted_actions.len())]
                    }
                    Some(PlayerSettings::Mcts(mcts_settings)) => calculate_best_turn(
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
                    _ => todo!(),
                };
                log::debug!("Player {} plays {:?}", player, action);
                state = action.execute(&state);
            }
            Actor::GameAction(actions) => {
                //TODO: Use a weighted random (because the second variable is supposed to be the weight)
                let action = actions[rand::thread_rng().gen_range(0..actions.len())].0;
                state = action.execute(&state);
            }
        }
    }
    game.visualise_state(&state);
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
    for episode in 0..arena_settings.episodes {
        log::info!("Starting episode {}", episode);
        let result = match arena_settings.game {
            Games::C4 => run_episode(C4, arena_settings.players.clone()),
            Games::NT => run_episode(
                NT {
                    player_count: arena_settings.players.len() as u8,
                },
                arena_settings.players.clone(),
            ),
        };
        for (i, r) in result.iter().enumerate() {
            results[i] += *r;
        }
    }
    println!("Player\tResult\tPercentage");
    let total: f64 = results.iter().sum();
    for (i, r) in results.iter().enumerate() {
        println!("{}\t{:?}\t{:>5.2}%", i + 1, r, (100.0 * r) / total);
    }
}
