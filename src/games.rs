mod c4;
mod nt;

pub use c4::C4;
pub use nt::NT;

use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum Games {
    C4,
    NT,
}
