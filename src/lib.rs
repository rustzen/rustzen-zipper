pub mod archive_pack;
pub mod config;
pub mod error;
pub mod inspect;
pub mod unpack;

pub use archive_pack as pack;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
pub enum CompressionMode {
    Stored,
    Deflated,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
pub enum OverwriteMode {
    Overwrite,
    Skip,
    Error,
}
