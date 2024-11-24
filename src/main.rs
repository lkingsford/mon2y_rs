mod mon2y;

use crate::mon2y::action_log::{Action, ActionLogEntry};
use clap::Parser;
use log::info;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    info!("Hello, world!");
}
