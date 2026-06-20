//! Logging Utilities
//! 
//! Centralized logging management for the application.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{info, debug, warn, error, Level};
use tracing_subscriber::{fmt, EnvFilter};

/// Log manager for file-based logging
pub struct LogManager {
    log_file: Mutex<std::fs::File>,
    log_level: Level,
}

impl LogManager {
    /// Create a new log manager
    pub fn new(log_dir: PathBuf, level: Level) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&log_dir)?;

        let log_path = log_dir.join(format!("sidozdev_{}.log", 
            chrono::Local::now().format("%Y%m%d_%H%M%S")));

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        Ok(Self {
            log_file: Mutex::new(log_file),
            log_level: level,
        })
    }

    /// Log a message to file
    pub fn log(&self, level: Level, message: &str) {
        if level <= self.log_level {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let log_line = format!("[{}] {:5} - {}\n", timestamp, level, message);

            if let Ok(mut file) = self.log_file.lock() {
                let _ = file.write_all(log_line.as_bytes());
                let _ = file.flush();
            }
        }
    }

    /// Log info message
    pub fn info(&self, message: &str) {
        self.log(Level::INFO, message);
    }

    /// Log warning message
    pub fn warn(&self, message: &str) {
        self.log(Level::WARN, message);
    }

    /// Log error message
    pub fn error(&self, message: &str) {
        self.log(Level::ERROR, message);
    }

    /// Log debug message
    pub fn debug(&self, message: &str) {
        self.log(Level::DEBUG, message);
    }
}

/// Initialize tracing subscriber
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("sidozdev=info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}

/// Get log directory path
pub fn get_log_dir() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("sidozdev");
    path.push("logs");
    path
}
