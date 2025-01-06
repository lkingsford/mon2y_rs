mod c4;
mod nt;

pub use c4::C4;
pub use nt::NT;

use clap::ValueEnum;
use serde::Deserialize;

#[derive(Debug, Clone, ValueEnum, Deserialize)]
pub enum Games {
    C4,
    NT,
}
