//! Hybrid ISO Detection
//! 
//! Detects and handles Hybrid ISO images (ISO + USB bootable).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};

/// Hybrid ISO detection result
#[derive(Debug, Clone)]
pub struct HybridIsoInfo {
    pub is_hybrid: bool,
    pub has_mbr: bool,
    pub has_gpt: bool,
    pub has_isolinux: bool,
    pub has_syslinux: bool,
    pub has_grub: bool,
    pub boot_offset: u64,
    pub iso_offset: u64,
    pub partition_count: u32,
}

impl Default for HybridIsoInfo {
    fn default() -> Self {
        Self {
            is_hybrid: false,
            has_mbr: false,
            has_gpt: false,
            has_isolinux: false,
            has_syslinux: false,
            has_grub: false,
            boot_offset: 0,
            iso_offset: 0,
            partition_count: 0,
        }
    }
}

/// Detect if an ISO is a hybrid image
pub fn detect_hybrid_iso(path: &Path) -> Result<HybridIsoInfo> {
    info!("Detecting hybrid ISO: {:?}", path);

    let mut file = File::open(path)
        .context("Failed to open file for hybrid detection")?;

    let mut info = HybridIsoInfo::default();

    // Check for MBR at offset 0
    file.seek(SeekFrom::Start(0))?;
    let mut mbr_sector = [0u8; 512];
    file.read_exact(&mut mbr_sector)?;

    if mbr_sector[510] == 0x55 && mbr_sector[511] == 0xAA {
        info.has_mbr = true;
        info.is_hybrid = true;

        // Count partitions
        info.partition_count = count_mbr_partitions(&mbr_sector);

        // Check for GPT
        if mbr_sector[446..462] == [0x00, 0x00, 0x01, 0x00, 0xEE, 0x00, 0x00, 0x00, 
                                       0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF] {
            info.has_gpt = true;
        }
    }

    // Check for ISO 9660 signature at offset 32768 (sector 16)
    file.seek(SeekFrom::Start(32768))?;
    let mut iso_signature = [0u8; 5];
    file.read_exact(&mut iso_signature)?;

    if &iso_signature == b"CD001" {
        info.iso_offset = 32768;
    }

    // Check for bootloaders
    info.has_isolinux = check_for_isolinux(&mut file)?;
    info.has_syslinux = check_for_syslinux(&mut file)?;
    info.has_grub = check_for_grub(&mut file)?;

    if info.has_isolinux || info.has_syslinux || info.has_grub {
        info.is_hybrid = true;
    }

    info!("Hybrid detection result: {:?}", info);
    Ok(info)
}

/// Count MBR partitions
fn count_mbr_partitions(mbr: &[u8]) -> u32 {
    let mut count = 0;
    for i in 0..4 {
        let offset = 446 + i * 16;
        if mbr[offset + 4] != 0 { // Partition type != 0
            count += 1;
        }
    }
    count
}

/// Check for ISOLINUX bootloader
fn check_for_isolinux(file: &mut File) -> Result<bool> {
    // ISOLINUX has specific signatures in boot files
    // Check for "ISOLINUX" string in boot sector area
    file.seek(SeekFrom::Start(0x8000))?; // 32KB offset
    let mut buffer = [0u8; 64];
    file.read_exact(&mut buffer)?;

    let content = String::from_utf8_lossy(&buffer);
    Ok(content.contains("ISOLINUX") || content.contains("isolinux"))
}

/// Check for SYSLINUX bootloader
fn check_for_syslinux(file: &mut File) -> Result<bool> {
    file.seek(SeekFrom::Start(0x8000))?;
    let mut buffer = [0u8; 64];
    file.read_exact(&mut buffer)?;

    let content = String::from_utf8_lossy(&buffer);
    Ok(content.contains("SYSLINUX") || content.contains("syslinux"))
}

/// Check for GRUB bootloader
fn check_for_grub(file: &mut File) -> Result<bool> {
    // Check for GRUB stage2 or core.img signatures
    file.seek(SeekFrom::Start(0x8000))?;
    let mut buffer = [0u8; 64];
    file.read_exact(&mut buffer)?;

    let content = String::from_utf8_lossy(&buffer);
    Ok(content.contains("GRUB") || content.contains("grub"))
}

/// Get recommended write mode for hybrid ISO
pub fn get_recommended_write_mode(info: &HybridIsoInfo) -> HybridWriteMode {
    if info.has_mbr && info.has_gpt {
        HybridWriteMode::DdRaw
    } else if info.has_mbr {
        HybridWriteMode::DdRaw
    } else if info.has_isolinux || info.has_syslinux {
        HybridWriteMode::IsoHybrid
    } else {
        HybridWriteMode::StandardIso
    }
}

/// Write mode for hybrid ISOs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridWriteMode {
    /// Standard ISO 9660 copy
    StandardIso,
    /// DD-style raw copy (preserves MBR/boot sectors)
    DdRaw,
    /// ISO hybrid mode (ISOLINUX/SYSLINUX)
    IsoHybrid,
}

impl std::fmt::Display for HybridWriteMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HybridWriteMode::StandardIso => write!(f, "Standard ISO"),
            HybridWriteMode::DdRaw => write!(f, "DD Raw Copy"),
            HybridWriteMode::IsoHybrid => write!(f, "ISO Hybrid"),
        }
    }
}
