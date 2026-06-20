//! Configuration Management
//! 
//! Handles persistent application settings and preferences.

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::core::app_state::{Theme, Language, BootMode, PartitionScheme, FileSystem, HashAlgorithm};

/// Persistent application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub language: Language,
    pub default_boot_mode: BootMode,
    pub default_partition_scheme: PartitionScheme,
    pub default_file_system: FileSystem,
    pub default_hash_algorithm: HashAlgorithm,
    pub auto_check_updates: bool,
    pub last_iso_directory: Option<PathBuf>,
    pub show_advanced_options: bool,
    pub log_level: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            language: Language::English,
            default_boot_mode: BootMode::Uefi,
            default_partition_scheme: PartitionScheme::Gpt,
            default_file_system: FileSystem::Fat32,
            default_hash_algorithm: HashAlgorithm::Sha256,
            auto_check_updates: true,
            last_iso_directory: None,
            show_advanced_options: false,
            log_level: "info".to_string(),
        }
    }
}

impl AppSettings {
    /// Get the configuration directory path
    pub fn config_dir() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("sidozdev");
        path
    }

    /// Get the configuration file path
    pub fn config_file() -> PathBuf {
        let mut path = Self::config_dir();
        path.push("config.json");
        path
    }

    /// Load settings from disk
    pub fn load() -> Self {
        let path = Self::config_file();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let path = Self::config_file();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
