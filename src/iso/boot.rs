//! Boot Record Analysis
//! 
//! Analyzes boot records and bootloaders in ISO images.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};

/// Boot record information
#[derive(Debug, Clone)]
pub struct BootRecordInfo {
    pub boot_type: BootType,
    pub boot_loader: String,
    pub version: String,
    pub architecture: String,
    pub is_efi: bool,
    pub is_legacy: bool,
    pub boot_file_path: String,
    pub partition_guid: Option<String>,
}

/// Boot type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootType {
    Unknown,
    Bios,
    Uefi,
    Hybrid,
    ElTorito,
    Grub,
    Syslinux,
    Isolinux,
    WindowsBootManager,
}

impl std::fmt::Display for BootType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootType::Unknown => write!(f, "Unknown"),
            BootType::Bios => write!(f, "BIOS"),
            BootType::Uefi => write!(f, "UEFI"),
            BootType::Hybrid => write!(f, "Hybrid"),
            BootType::ElTorito => write!(f, "El Torito"),
            BootType::Grub => write!(f, "GRUB"),
            BootType::Syslinux => write!(f, "SYSLINUX"),
            BootType::Isolinux => write!(f, "ISOLINUX"),
            BootType::WindowsBootManager => write!(f, "Windows Boot Manager"),
        }
    }
}

/// Analyze boot records in an ISO image
pub fn analyze_boot_records(path: &Path) -> Result<Vec<BootRecordInfo>> {
    info!("Analyzing boot records in: {:?}", path);

    let mut file = File::open(path)
        .context("Failed to open file for boot analysis")?;

    let mut records = Vec::new();

    // Check for El Torito boot catalog
    if let Ok(record) = analyze_eltorito(&mut file) {
        records.push(record);
    }

    // Check for UEFI boot
    if let Ok(record) = analyze_uefi_boot(&mut file) {
        records.push(record);
    }

    // Check for BIOS boot
    if let Ok(record) = analyze_bios_boot(&mut file) {
        records.push(record);
    }

    // Check for GRUB
    if let Ok(record) = analyze_grub(&mut file) {
        records.push(record);
    }

    // Check for SYSLINUX/ISOLINUX
    if let Ok(record) = analyze_syslinux(&mut file) {
        records.push(record);
    }

    info!("Found {} boot record(s)", records.len());
    Ok(records)
}

/// Analyze El Torito boot record
fn analyze_eltorito(file: &mut File) -> Result<BootRecordInfo> {
    file.seek(SeekFrom::Start(34816))?; // Sector 17
    let mut boot_record = [0u8; 2048];
    file.read_exact(&mut boot_record)?;

    // Check for El Torito signature
    let signature = &boot_record[7..39];
    if signature != b"EL TORITO SPECIFICATION" {
        return Err(anyhow::anyhow!("Not an El Torito boot record"));
    }

    // Extract boot catalog pointer
    let catalog_lba = u32::from_le_bytes([
        boot_record[71], boot_record[72], boot_record[73], boot_record[74]
    ]);

    // Read boot catalog
    file.seek(SeekFrom::Start(catalog_lba as u64 * 2048))?;
    let mut catalog = [0u8; 2048];
    file.read_exact(&mut catalog)?;

    // Determine boot type from catalog
    let boot_indicator = catalog[0x20];
    let boot_type = if boot_indicator == 0x88 {
        BootType::ElTorito
    } else {
        BootType::Unknown
    };

    let platform_id = catalog[0x21];
    let architecture = match platform_id {
        0x00 => "x86",
        0x01 => "PowerPC",
        0x02 => "x86_64",
        0xEF => "EFI",
        _ => "Unknown",
    };

    Ok(BootRecordInfo {
        boot_type,
        boot_loader: "El Torito".to_string(),
        version: "1.0".to_string(),
        architecture: architecture.to_string(),
        is_efi: platform_id == 0xEF,
        is_legacy: platform_id == 0x00,
        boot_file_path: format!("Boot Catalog at LBA {}", catalog_lba),
        partition_guid: None,
    })
}

/// Analyze UEFI boot
fn analyze_uefi_boot(file: &mut File) -> Result<BootRecordInfo> {
    // Check for EFI boot partition in MBR/GPT
    file.seek(SeekFrom::Start(0))?;
    let mut mbr = [0u8; 512];
    file.read_exact(&mut mbr)?;

    // Check for GPT
    if &mbr[512..520] == b"EFI PART" {
        return Ok(BootRecordInfo {
            boot_type: BootType::Uefi,
            boot_loader: "UEFI".to_string(),
            version: "2.0".to_string(),
            architecture: "x86_64".to_string(),
            is_efi: true,
            is_legacy: false,
            boot_file_path: "EFI/BOOT/BOOTX64.EFI".to_string(),
            partition_guid: Some("C12A7328-F81F-11D2-BA4B-00A0C93EC93B".to_string()),
        });
    }

    // Check MBR partition type 0xEF (EFI System Partition)
    let partition_type = mbr[446 + 4];
    if partition_type == 0xEF {
        return Ok(BootRecordInfo {
            boot_type: BootType::Uefi,
            boot_loader: "UEFI".to_string(),
            version: "2.0".to_string(),
            architecture: "x86_64".to_string(),
            is_efi: true,
            is_legacy: false,
            boot_file_path: "EFI/BOOT/BOOTX64.EFI".to_string(),
            partition_guid: None,
        });
    }

    Err(anyhow::anyhow!("No UEFI boot found"))
}

/// Analyze BIOS boot
fn analyze_bios_boot(file: &mut File) -> Result<BootRecordInfo> {
    file.seek(SeekFrom::Start(0))?;
    let mut mbr = [0u8; 512];
    file.read_exact(&mut mbr)?;

    if mbr[510] != 0x55 || mbr[511] != 0xAA {
        return Err(anyhow::anyhow!("No valid MBR boot signature"));
    }

    // Check for active partition
    let has_active_partition = mbr[446] == 0x80;

    if !has_active_partition {
        return Err(anyhow::anyhow!("No active partition found"));
    }

    Ok(BootRecordInfo {
        boot_type: BootType::Bios,
        boot_loader: "MBR".to_string(),
        version: "1.0".to_string(),
        architecture: "x86".to_string(),
        is_efi: false,
        is_legacy: true,
        boot_file_path: "MBR Boot Code".to_string(),
        partition_guid: None,
    })
}

/// Analyze GRUB bootloader
fn analyze_grub(file: &mut File) -> Result<BootRecordInfo> {
    file.seek(SeekFrom::Start(0x8000))?;
    let mut buffer = [0u8; 128];
    file.read_exact(&mut buffer)?;

    let content = String::from_utf8_lossy(&buffer);

    if !content.contains("GRUB") && !content.contains("grub") {
        return Err(anyhow::anyhow!("No GRUB bootloader found"));
    }

    // Extract version if available
    let version = if let Some(pos) = content.find("GRUB ") {
        content[pos..pos+10].to_string()
    } else {
        "Unknown".to_string()
    };

    Ok(BootRecordInfo {
        boot_type: BootType::Grub,
        boot_loader: "GRUB".to_string(),
        version,
        architecture: "x86/x64".to_string(),
        is_efi: content.contains("EFI"),
        is_legacy: true,
        boot_file_path: "boot/grub".to_string(),
        partition_guid: None,
    })
}

/// Analyze SYSLINUX/ISOLINUX
fn analyze_syslinux(file: &mut File) -> Result<BootRecordInfo> {
    file.seek(SeekFrom::Start(0x8000))?;
    let mut buffer = [0u8; 128];
    file.read_exact(&mut buffer)?;

    let content = String::from_utf8_lossy(&buffer);

    let (boot_type, loader) = if content.contains("ISOLINUX") {
        (BootType::Isolinux, "ISOLINUX")
    } else if content.contains("SYSLINUX") {
        (BootType::Syslinux, "SYSLINUX")
    } else {
        return Err(anyhow::anyhow!("No SYSLINUX/ISOLINUX found"));
    };

    Ok(BootRecordInfo {
        boot_type,
        boot_loader: loader.to_string(),
        version: "Unknown".to_string(),
        architecture: "x86".to_string(),
        is_efi: false,
        is_legacy: true,
        boot_file_path: format!("boot/{}", loader.to_lowercase()),
        partition_guid: None,
    })
}

/// Get recommended boot mode based on boot records
pub fn get_recommended_boot_mode(records: &[BootRecordInfo]) -> String {
    let has_uefi = records.iter().any(|r| r.is_efi);
    let has_legacy = records.iter().any(|r| r.is_legacy);

    if has_uefi && has_legacy {
        "UEFI + CSM (Legacy)".to_string()
    } else if has_uefi {
        "UEFI Only".to_string()
    } else if has_legacy {
        "Legacy BIOS Only".to_string()
    } else {
        "Unknown".to_string()
    }
}
