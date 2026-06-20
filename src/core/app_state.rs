//! Application State Management
//! 
//! Central state management for the Sidozdev application.

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::disk::device::UsbDevice;
use crate::iso::parser::IsoInfo;

/// Current phase of the operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationPhase {
    Idle,
    ScanningDevices,
    ValidatingIso,
    Formatting,
    Writing,
    Verifying,
    Completed,
    Cancelled,
    Error,
}

/// Boot mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootMode {
    BiosLegacy,
    Uefi,
    UefiSecureBoot,
}

impl std::fmt::Display for BootMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootMode::BiosLegacy => write!(f, "BIOS (Legacy)"),
            BootMode::Uefi => write!(f, "UEFI"),
            BootMode::UefiSecureBoot => write!(f, "UEFI + Secure Boot"),
        }
    }
}

/// Partition scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartitionScheme {
    Mbr,
    Gpt,
}

impl std::fmt::Display for PartitionScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitionScheme::Mbr => write!(f, "MBR"),
            PartitionScheme::Gpt => write!(f, "GPT"),
        }
    }
}

/// File system type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSystem {
    Fat32,
    Ntfs,
    ExFat,
}

impl std::fmt::Display for FileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileSystem::Fat32 => write!(f, "FAT32"),
            FileSystem::Ntfs => write!(f, "NTFS"),
            FileSystem::ExFat => write!(f, "exFAT"),
        }
    }
}

/// Hash algorithm for ISO verification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
}

impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Md5 => write!(f, "MD5"),
            HashAlgorithm::Sha1 => write!(f, "SHA1"),
            HashAlgorithm::Sha256 => write!(f, "SHA256"),
        }
    }
}

/// Theme preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Language preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Arabic,
    English,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Arabic => write!(f, "العربية"),
            Language::English => write!(f, "English"),
        }
    }
}

/// Progress information
#[derive(Debug, Clone, Default)]
pub struct ProgressInfo {
    pub phase: OperationPhase,
    pub percentage: f32,
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub speed_mbps: f32,
    pub eta_seconds: u64,
    pub current_operation: String,
}

/// Application configuration for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationConfig {
    pub selected_device: Option<String>,
    pub iso_path: Option<PathBuf>,
    pub boot_mode: BootMode,
    pub partition_scheme: PartitionScheme,
    pub file_system: FileSystem,
    pub hash_algorithm: HashAlgorithm,
    pub verify_after_write: bool,
    pub quick_format: bool,
    pub bad_block_check: bool,
}

impl Default for OperationConfig {
    fn default() -> Self {
        Self {
            selected_device: None,
            iso_path: None,
            boot_mode: BootMode::Uefi,
            partition_scheme: PartitionScheme::Gpt,
            file_system: FileSystem::Fat32,
            hash_algorithm: HashAlgorithm::Sha256,
            verify_after_write: true,
            quick_format: true,
            bad_block_check: false,
        }
    }
}

/// Central application state
#[derive(Debug)]
pub struct AppState {
    pub config: OperationConfig,
    pub devices: Vec<UsbDevice>,
    pub iso_info: Option<IsoInfo>,
    pub progress: ProgressInfo,
    pub logs: Vec<LogEntry>,
    pub is_running: bool,
    pub cancel_requested: bool,
    pub theme: Theme,
    pub language: Language,
    pub error_message: Option<String>,
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
    Debug,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: OperationConfig::default(),
            devices: Vec::new(),
            iso_info: None,
            progress: ProgressInfo::default(),
            logs: Vec::new(),
            is_running: false,
            cancel_requested: false,
            theme: Theme::System,
            language: Language::English,
            error_message: None,
        }
    }

    pub fn add_log(&mut self, level: LogLevel, message: impl Into<String>) {
        let timestamp = chrono::Local::now().format("%H:%M:%S.%3f").to_string();
        self.logs.push(LogEntry {
            timestamp,
            level,
            message: message.into(),
        });

        // Keep only last 1000 log entries
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    pub fn request_cancel(&mut self) {
        self.cancel_requested = true;
        self.add_log(LogLevel::Warning, "Cancellation requested by user");
    }

    pub fn reset_cancel(&mut self) {
        self.cancel_requested = false;
    }

    pub fn is_ready(&self) -> bool {
        self.config.selected_device.is_some() 
            && self.config.iso_path.is_some()
            && !self.is_running
    }
}

/// Thread-safe application state wrapper
pub type SharedAppState = Arc<Mutex<AppState>>;

/// Create a new shared application state
pub fn create_shared_state() -> SharedAppState {
    Arc::new(Mutex::new(AppState::new()))
}
