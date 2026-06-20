//! USB Device Representation
//! 
//! Defines the data structures for USB storage devices.

use serde::{Serialize, Deserialize};

/// USB storage device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    /// Unique device identifier (e.g., "\\\\.\\PhysicalDrive1")
    pub device_id: String,
    /// User-friendly name
    pub friendly_name: String,
    /// Vendor name
    pub vendor: String,
    /// Product/model name
    pub product: String,
    /// Total capacity in bytes
    pub capacity: u64,
    /// Available free space in bytes
    pub free_space: u64,
    /// Interface type (USB 2.0, USB 3.0, etc.)
    pub interface_type: String,
    /// Device path for I/O operations
    pub device_path: String,
    /// Drive letter(s) if mounted
    pub drive_letters: Vec<String>,
    /// Serial number
    pub serial_number: String,
    /// Whether the device is removable
    pub is_removable: bool,
    /// Whether the device is currently mounted
    pub is_mounted: bool,
    /// Current file system
    pub current_file_system: Option<String>,
    /// Partition scheme (MBR/GPT/Unknown)
    pub partition_scheme: Option<String>,
    /// VID:PID string
    pub vid_pid: String,
}

impl UsbDevice {
    /// Format capacity as human-readable string
    pub fn capacity_display(&self) -> String {
        format_bytes(self.capacity)
    }

    /// Format free space as human-readable string
    pub fn free_space_display(&self) -> String {
        format_bytes(self.free_space)
    }

    /// Get a summary string for display
    pub fn display_summary(&self) -> String {
        format!(
            "{} - {} ({} - {})",
            self.friendly_name,
            self.device_id,
            self.capacity_display(),
            self.interface_type
        )
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Device capability information
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    pub supports_usb_boot: bool,
    pub supports_efi_boot: bool,
    pub max_partition_count: u32,
    pub sector_size: u32,
    pub physical_sector_size: u32,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            supports_usb_boot: true,
            supports_efi_boot: true,
            max_partition_count: 4,
            sector_size: 512,
            physical_sector_size: 512,
        }
    }
}
