//! Explores the tree from the first turn repeatedly, saving the annotations
mod game;
mod games;
mod mon2y;
mod test;

use clap::Parser;
use env_logger::fmt::Formatter;
use game::Game;
use games::Games;
use games::{C4, CS, EBR, NT};
use log::Record;
use mon2y::{calculate_best_turn, BestTurnPolicy};
use std::io;
use std::io::Write;
use std::thread;
use std::time::Instant;

const CHUNK_SIZE: usize = 1000;

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
    #[arg(short, long, default_value = None)]
    reports_folder: Option<String>,
    #[arg(short('c'), long, default_value_t = 2.0_f64.sqrt())]
    exploration_constant: f64,
}

fn run_explore<G: Game>(
    game: G,
    iterations: usize,
    thread_count: usize,
    exploration_constant: f64,
    report_path: &str,
) -> f64 {
    let state = game.init_game();
    let start = Instant::now();
    let (_, annotations) = calculate_best_turn(
        iterations,
        None,
        thread_count,
        state,
        BestTurnPolicy::MostVisits,
        exploration_constant,
        false,
        true,
    );
    let elapsed = start.elapsed();
    let iterations_per_second = iterations as f64 / elapsed.as_secs_f64();
    println!(
        "{} iterations in {:.2} seconds ({:.2} iterations per second)",
        iterations,
        &elapsed.as_secs_f64(),
        iterations_per_second
    );
    let chunks = (annotations.len() as f64 / CHUNK_SIZE as f64).ceil() as usize;
    annotations
        .chunks(CHUNK_SIZE)
        .enumerate()
        .for_each(|(i, chunk)| {
            let filename = format!(
                "{}-{:0>width$}.json",
                report_path,
                i,
                width = chunks.to_string().len()
            );
            let serialized = serde_json::to_string(&chunk).unwrap();
            std::fs::write(filename, serialized).unwrap();
        });
    elapsed.as_secs_f64()
}

fn main() {
    let args = Args::parse();

    let reports_folder = args.reports_folder.unwrap_or_else(|| {
        format!(
            "reports/{:?}/{}",
            args.game,
            chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
        )
    });

    if let Err(err) = std::fs::create_dir_all(&reports_folder) {
        eprintln!(
            "Failed to create reports folder {}: {}",
            reports_folder, err
        );
        std::process::exit(1);
    }

    println!(
        "===\nIterations: {}, Episodes: {}, Threads: {}, Path: {}",
        args.iterations, args.episodes, args.threads, &reports_folder
    );
    println!("---");
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

    (0..args.episodes)
        .map(|iteration| format!("{}/{}", reports_folder, iteration))
        .for_each(|filename| {
            match args.game {
                Games::C4 => run_explore(
                    C4,
                    args.iterations,
                    args.threads,
                    args.exploration_constant,
                    &filename,
                ),
                Games::NT => run_explore(
                    NT {
                        player_count: args.player_count,
                    },
                    args.iterations,
                    args.threads,
                    args.exploration_constant,
                    &filename,
                ),
                Games::CS => run_explore(
                    CS {
                        player_count: args.player_count,
                    },
                    args.iterations,
                    args.threads,
                    args.exploration_constant,
                    &filename,
                ),
                Games::EBR => run_explore(
                    EBR {
                        player_count: args.player_count,
                    },
                    args.iterations,
                    args.threads,
                    args.exploration_constant,
                    &filename,
                ),
            };
        });
}
