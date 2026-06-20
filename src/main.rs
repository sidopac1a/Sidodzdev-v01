//! Sidozdev - Professional USB Bootable Media Creator
//! 
//! A Rust-based, cross-platform alternative to Rufus for creating bootable USB drives.

use sidozdev::run_app;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("sidozdev=info")
        .init();

    // Run the application
    if let Err(e) = run_app() {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}
