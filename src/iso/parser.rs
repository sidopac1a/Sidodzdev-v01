//! ISO Image Parser
//! 
//! Parses ISO9660, Hybrid ISO, and IMG files to extract metadata.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use tracing::{info, debug, warn};

/// ISO image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoInfo {
    /// Volume label
    pub label: String,
    /// Total size in bytes
    pub size: u64,
    /// ISO type
    pub iso_type: IsoType,
    /// Bootable status
    pub is_bootable: bool,
    /// Boot mode support
    pub boot_modes: Vec<String>,
    /// File system type
    pub file_system: String,
    /// Creation date
    pub creation_date: String,
    /// Publisher
    pub publisher: String,
    /// Architecture (x86, x64, ARM, etc.)
    pub architecture: String,
    /// Has UEFI boot
    pub has_uefi_boot: bool,
    /// Has BIOS boot
    pub has_bios_boot: bool,
    /// El Torito boot catalog present
    pub has_eltorito: bool,
    /// Hybrid ISO (ISO + USB image)
    pub is_hybrid: bool,
    /// MD5 hash (optional)
    pub md5_hash: Option<String>,
    /// SHA1 hash (optional)
    pub sha1_hash: Option<String>,
    /// SHA256 hash (optional)
    pub sha256_hash: Option<String>,
    /// Estimated write time (seconds)
    pub estimated_write_time: u64,
}

/// ISO image type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsoType {
    Iso9660,
    HybridIso,
    Img,
    Udf,
    Unknown,
}

impl std::fmt::Display for IsoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsoType::Iso9660 => write!(f, "ISO 9660"),
            IsoType::HybridIso => write!(f, "Hybrid ISO"),
            IsoType::Img => write!(f, "IMG/Raw Image"),
            IsoType::Udf => write!(f, "UDF"),
            IsoType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Parse an ISO/IMG file and extract information
pub async fn parse_iso(path: &Path) -> Result<IsoInfo> {
    info!("Parsing ISO file: {:?}", path);

    let mut file = File::open(path)
        .context("Failed to open ISO file")?;

    let size = file.metadata()?.len();

    // Determine ISO type
    let iso_type = detect_iso_type(&mut file)?;

    // Parse based on type
    let info = match iso_type {
        IsoType::Iso9660 | IsoType::HybridIso => parse_iso9660(&mut file, size, iso_type)?,
        IsoType::Img => parse_img(&mut file, size)?,
        IsoType::Udf => parse_udf(&mut file, size)?,
        IsoType::Unknown => parse_unknown(&mut file, size)?,
    };

    info!("ISO parsed successfully: {} ({}, {})", 
        info.label, info.iso_type, info.size);

    Ok(info)
}

/// Detect the type of ISO image
fn detect_iso_type(file: &mut File) -> Result<IsoType> {
    let mut header = [0u8; 32768]; // 32KB - ISO primary volume descriptor offset

    file.seek(SeekFrom::Start(0))?;
    let bytes_read = file.read(&mut header)?;

    if bytes_read < 8 {
        return Ok(IsoType::Unknown);
    }

    // Check for ISO 9660 signature at offset 32768 (sector 16)
    file.seek(SeekFrom::Start(32768))?;
    let mut iso_signature = [0u8; 5];
    file.read_exact(&mut iso_signature)?;

    if &iso_signature == b"CD001" {
        // Check for hybrid ISO (has partition table or boot record)
        file.seek(SeekFrom::Start(0x8000))?;
        let mut hybrid_check = [0u8; 2];
        if file.read(&mut hybrid_check)? == 2 {
            // Check for MBR signature at offset 510-511
            file.seek(SeekFrom::Start(510))?;
            let mut mbr_sig = [0u8; 2];
            if file.read(&mut mbr_sig)? == 2 && mbr_sig == [0x55, 0xAA] {
                return Ok(IsoType::HybridIso);
            }
        }
        return Ok(IsoType::Iso9660);
    }

    // Check for UDF
    file.seek(SeekFrom::Start(0))?;
    let mut udf_check = [0u8; 8];
    if file.read(&mut udf_check)? == 8 {
        // UDF has "BEA01" at start or "NSR" somewhere
        if &udf_check[..5] == b"BEA01" || &udf_check[..3] == b"NSR" {
            return Ok(IsoType::Udf);
        }
    }

    // Check for raw IMG (no recognizable header, but has MBR)
    file.seek(SeekFrom::Start(510))?;
    let mut mbr_sig = [0u8; 2];
    if file.read(&mut mbr_sig)? == 2 && mbr_sig == [0x55, 0xAA] {
        return Ok(IsoType::Img);
    }

    Ok(IsoType::Unknown)
}

/// Parse ISO 9660 file system
fn parse_iso9660(file: &mut File, size: u64, iso_type: IsoType) -> Result<IsoInfo> {
    // Read primary volume descriptor at sector 16 (offset 32768)
    file.seek(SeekFrom::Start(32768))?;

    let mut pvd = [0u8; 2048]; // One sector
    file.read_exact(&mut pvd)?;

    // Extract volume label (bytes 40-71 of PVD)
    let label = String::from_utf8_lossy(&pvd[40..72])
        .trim_end()
        .to_string();

    // Extract system identifier (bytes 8-39)
    let system_id = String::from_utf8_lossy(&pvd[8..40])
        .trim_end()
        .to_string();

    // Extract publisher (bytes 318-446)
    let publisher = String::from_utf8_lossy(&pvd[318..446])
        .trim_end()
        .to_string();

    // Extract creation date (bytes 813-830)
    let creation_date = String::from_utf8_lossy(&pvd[813..831])
        .trim_end()
        .to_string();

    // Check for boot record descriptor
    let mut is_bootable = false;
    let mut has_uefi_boot = false;
    let mut has_bios_boot = false;
    let mut has_eltorito = false;
    let mut boot_modes = Vec::new();

    // Check for El Torito boot catalog
    if check_eltorito_boot(file)? {
        is_bootable = true;
        has_eltorito = true;
        boot_modes.push("El Torito".to_string());
    }

    // Check for UEFI boot files
    if check_uefi_boot(file)? {
        has_uefi_boot = true;
        if !boot_modes.contains(&"UEFI".to_string()) {
            boot_modes.push("UEFI".to_string());
        }
    }

    // Check for BIOS boot
    if check_bios_boot(file)? {
        has_bios_boot = true;
        if !boot_modes.contains(&"BIOS".to_string()) {
            boot_modes.push("BIOS".to_string());
        }
    }

    // Detect architecture from boot files
    let architecture = detect_architecture(file)?;

    // Estimate write time (rough: 10 MB/s for USB 2.0, 30 MB/s for USB 3.0)
    let estimated_write_time = size / (10 * 1024 * 1024);

    Ok(IsoInfo {
        label: if label.is_empty() { "NO_LABEL".to_string() } else { label },
        size,
        iso_type,
        is_bootable,
        boot_modes,
        file_system: "ISO 9660".to_string(),
        creation_date: if creation_date.is_empty() { "Unknown".to_string() } else { creation_date },
        publisher: if publisher.is_empty() { "Unknown".to_string() } else { publisher },
        architecture,
        has_uefi_boot,
        has_bios_boot,
        has_eltorito,
        is_hybrid: iso_type == IsoType::HybridIso,
        md5_hash: None,
        sha1_hash: None,
        sha256_hash: None,
        estimated_write_time,
    })
}

/// Parse raw IMG file
fn parse_img(file: &mut File, size: u64) -> Result<IsoInfo> {
    // IMG files are raw disk images, parse MBR for info
    let mut mbr = [0u8; 512];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut mbr)?;

    // Check for boot signature
    let is_bootable = mbr[510] == 0x55 && mbr[511] == 0xAA;

    let mut boot_modes = Vec::new();
    let mut has_uefi_boot = false;
    let mut has_bios_boot = false;

    if is_bootable {
        boot_modes.push("BIOS".to_string());
        has_bios_boot = true;
    }

    // Check for EFI partition
    if check_efi_partition(file)? {
        has_uefi_boot = true;
        boot_modes.push("UEFI".to_string());
    }

    let estimated_write_time = size / (10 * 1024 * 1024);

    Ok(IsoInfo {
        label: "IMG_IMAGE".to_string(),
        size,
        iso_type: IsoType::Img,
        is_bootable,
        boot_modes,
        file_system: "Raw Image".to_string(),
        creation_date: "Unknown".to_string(),
        publisher: "Unknown".to_string(),
        architecture: "Unknown".to_string(),
        has_uefi_boot,
        has_bios_boot,
        has_eltorito: false,
        is_hybrid: false,
        md5_hash: None,
        sha1_hash: None,
        sha256_hash: None,
        estimated_write_time,
    })
}

/// Parse UDF file system
fn parse_udf(file: &mut File, size: u64) -> Result<IsoInfo> {
    // UDF parsing is simplified
    let estimated_write_time = size / (10 * 1024 * 1024);

    Ok(IsoInfo {
        label: "UDF_VOLUME".to_string(),
        size,
        iso_type: IsoType::Udf,
        is_bootable: false,
        boot_modes: Vec::new(),
        file_system: "UDF".to_string(),
        creation_date: "Unknown".to_string(),
        publisher: "Unknown".to_string(),
        architecture: "Unknown".to_string(),
        has_uefi_boot: false,
        has_bios_boot: false,
        has_eltorito: false,
        is_hybrid: false,
        md5_hash: None,
        sha1_hash: None,
        sha256_hash: None,
        estimated_write_time,
    })
}

/// Parse unknown image type
fn parse_unknown(file: &mut File, size: u64) -> Result<IsoInfo> {
    let estimated_write_time = size / (10 * 1024 * 1024);

    Ok(IsoInfo {
        label: "UNKNOWN".to_string(),
        size,
        iso_type: IsoType::Unknown,
        is_bootable: false,
        boot_modes: Vec::new(),
        file_system: "Unknown".to_string(),
        creation_date: "Unknown".to_string(),
        publisher: "Unknown".to_string(),
        architecture: "Unknown".to_string(),
        has_uefi_boot: false,
        has_bios_boot: false,
        has_eltorito: false,
        is_hybrid: false,
        md5_hash: None,
        sha1_hash: None,
        sha256_hash: None,
        estimated_write_time,
    })
}

/// Check for El Torito boot catalog
fn check_eltorito_boot(file: &mut File) -> Result<bool> {
    // El Torito boot record is at sector 17 (offset 34816)
    file.seek(SeekFrom::Start(34816))?;
    let mut boot_record = [0u8; 2048];
    file.read_exact(&mut boot_record)?;

    // Check for "EL TORITO SPECIFICATION" string
    let signature = &boot_record[7..39];
    Ok(signature == b"EL TORITO SPECIFICATION")
}

/// Check for UEFI boot files
fn check_uefi_boot(file: &mut File) -> Result<bool> {
    // UEFI boot files are typically in /EFI/BOOT/ directory
    // This is a simplified check - in production, we'd parse the file system
    // For now, check for EFI boot partition signature in MBR
    file.seek(SeekFrom::Start(446))?;
    let mut partition_entry = [0u8; 16];
    file.read_exact(&mut partition_entry)?;

    // Check partition type 0xEF (EFI System Partition)
    let partition_type = partition_entry[4];
    Ok(partition_type == 0xEF)
}

/// Check for BIOS boot
fn check_bios_boot(file: &mut File) -> Result<bool> {
    file.seek(SeekFrom::Start(0))?;
    let mut mbr = [0u8; 512];
    file.read_exact(&mut mbr)?;

    // Check for boot signature
    Ok(mbr[510] == 0x55 && mbr[511] == 0xAA)
}

/// Check for EFI partition
fn check_efi_partition(file: &mut File) -> Result<bool> {
    // Check GPT partition table for EFI system partition
    file.seek(SeekFrom::Start(512))?;
    let mut gpt_header = [0u8; 8];
    file.read_exact(&mut gpt_header)?;

    // GPT signature "EFI PART"
    Ok(&gpt_header == b"EFI PART")
}

/// Detect architecture from boot files
fn detect_architecture(file: &mut File) -> Result<String> {
    // Simplified architecture detection
    // In production, we'd scan for boot files and check their PE/ELF headers

    // Check for common architecture indicators in the image
    file.seek(SeekFrom::Start(0))?;
    let mut header = [0u8; 64];
    file.read_exact(&mut header)?;

    // Check for x86 boot code patterns
    if header[0] == 0xEB || header[0] == 0xE9 {
        // x86 jump instruction
        return Ok("x86/x64".to_string());
    }

    // Check for ARM patterns
    if &header[0..4] == &[0x00, 0x00, 0x00, 0xEA] {
        return Ok("ARM".to_string());
    }

    Ok("x86/x64".to_string()) // Default assumption
}

/// Check if file is a valid ISO/IMG
pub fn is_valid_iso(path: &Path) -> bool {
    if let Ok(mut file) = File::open(path) {
        if let Ok(iso_type) = detect_iso_type(&mut file) {
            return iso_type != IsoType::Unknown;
        }
    }
    false
}

/// Get ISO file size display
pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_idx = 0;

    while size_f >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size_f, UNITS[unit_idx])
}
