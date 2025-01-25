//! Benchmarks mon2y_rs by just taking the first turn and timing it
mod game;
mod games;
mod mon2y;
mod test;

use clap::Parser;
use game::Game;
use games::Games;
use games::{C4, CS, NT};
use mon2y::{calculate_best_turn, BestTurnPolicy};
use std::time::Instant;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg()]
    game: Games,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    #[arg(short, long, default_value_t = 100000)]
    iterations: usize,
    #[arg(short, long, default_value_t = 8)]
    threads: usize,
    #[arg(short, long, default_value_t = 10)]
    episodes: usize,
    #[arg(short, long, default_value_t = 3)]
    player_count: u8,
}

fn run_benchmark<G: Game>(game: G, iterations: usize, thread_count: usize) -> f64 {
    let state = game.init_game();
    let start = Instant::now();
    calculate_best_turn(
        iterations,
        None,
        thread_count,
        state,
        BestTurnPolicy::MostVisits,
        2.0_f64.sqrt(),
        false,
    );
    let elapsed = start.elapsed();
    let iterations_per_second = iterations as f64 / elapsed.as_secs_f64();
    println!(
        "{} iterations in {:.2} seconds ({:.2} iterations per second)",
        iterations,
        &elapsed.as_secs_f64(),
        iterations_per_second
    );
    elapsed.as_secs_f64()
}

fn main() {
    let args = Args::parse();
    println!(
        "===\nIterations: {}, Episodes: {}, Threads: {}",
        args.iterations, args.episodes, args.threads
    );
    println!("---");
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let durations: Vec<f64> = (0..args.episodes)
        .map(|_| match args.game {
            Games::C4 => run_benchmark(C4, args.iterations, args.threads),
            Games::NT => run_benchmark(
                NT {
                    player_count: args.player_count,
                },
                args.iterations,
                args.threads,
            ),
            Games::CS => run_benchmark(
                CS {
                    player_count: args.player_count,
                },
                args.iterations,
                args.threads,
            ),
        })
        .collect();
    println!("---");
    println!(
        "Average duration: {:.2} seconds",
        durations.iter().sum::<f64>() / durations.len() as f64
    );
    println!(
        "Average iterations per second: {}",
        (args.episodes * args.iterations) as f64 / durations.iter().sum::<f64>()
    );
}
