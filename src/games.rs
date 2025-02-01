pub mod c4;
pub mod cs;
pub mod ebr;
pub mod nt;

pub use c4::C4;
pub use cs::CS;
pub use ebr::EBR;
pub use nt::NT;

use clap::ValueEnum;
use serde::Deserialize;

#[derive(Debug, Clone, ValueEnum, Deserialize)]
pub enum Games {
    C4,
    NT,
    CS,
    EBR,
}
