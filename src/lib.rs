//! Sidozdev Library
//! 
//! Core library for the Sidozdev USB bootable media creator.

pub mod core;
pub mod disk;
pub mod iso;
pub mod ui;
pub mod utils;

use anyhow::Result;
use tracing::info;

/// Run the main application
pub fn run_app() -> Result<()> {
    info!("Starting Sidozdev v{}", env!("CARGO_PKG_VERSION"));
    ui::run()
}
