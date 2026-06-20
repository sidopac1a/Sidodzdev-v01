//! Disk Formatting
//! 
//! Handles formatting USB devices with various file systems (FAT32, NTFS, exFAT).

use std::ptr;
use std::ffi::CString;
use std::process::Command;
use winapi::um::fileapi::*;
use winapi::um::handleapi::*;
use winapi::um::winioctl::*;
use winapi::um::winnt::*;
use winapi::shared::minwindef::*;
use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};

use crate::core::app_state::FileSystem;
use crate::utils::platform::get_last_error_string;

/// Format a device with the specified file system
pub fn format_device(device_id: &str, file_system: FileSystem) -> Result<()> {
    info!("Formatting {} with {:?}", device_id, file_system);

    match file_system {
        FileSystem::Fat32 => format_fat32(device_id),
        FileSystem::Ntfs => format_ntfs(device_id),
        FileSystem::ExFat => format_exfat(device_id),
    }
}

/// Format device as FAT32
fn format_fat32(device_id: &str) -> Result<()> {
    info!("Formatting as FAT32");

    let drive_letter = extract_drive_letter(device_id);

    if let Some(letter) = drive_letter {
        let output = Command::new("cmd")
            .args(&[
                "/C",
                "format",
                &format!("{}", letter),
                "/FS:FAT32",
                "/V:SIDOZDEV",
                "/Q",
                "/Y",
            ])
            .output()
            .context("Failed to execute FAT32 format command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("FAT32 format failed: {}", stderr));
        }

        info!("FAT32 format completed successfully");
        Ok(())
    } else {
        native_format_fat32(device_id)
    }
}

/// Format device as NTFS
fn format_ntfs(device_id: &str) -> Result<()> {
    info!("Formatting as NTFS");

    let drive_letter = extract_drive_letter(device_id);

    if let Some(letter) = drive_letter {
        let output = Command::new("cmd")
            .args(&[
                "/C",
                "format",
                &format!("{}", letter),
                "/FS:NTFS",
                "/V:SIDOZDEV",
                "/Q",
                "/Y",
            ])
            .output()
            .context("Failed to execute NTFS format command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("NTFS format failed: {}", stderr));
        }

        info!("NTFS format completed successfully");
        Ok(())
    } else {
        native_format_ntfs(device_id)
    }
}

/// Format device as exFAT
fn format_exfat(device_id: &str) -> Result<()> {
    info!("Formatting as exFAT");

    let drive_letter = extract_drive_letter(device_id);

    if let Some(letter) = drive_letter {
        let output = Command::new("cmd")
            .args(&[
                "/C",
                "format",
                &format!("{}", letter),
                "/FS:EXFAT",
                "/V:SIDOZDEV",
                "/Q",
                "/Y",
            ])
            .output()
            .context("Failed to execute exFAT format command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("exFAT format failed: {}", stderr));
        }

        info!("exFAT format completed successfully");
        Ok(())
    } else {
        native_format_exfat(device_id)
    }
}

/// Extract drive letter from device path
fn extract_drive_letter(device_id: &str) -> Option<String> {
    if device_id.len() == 2 && device_id.ends_with(':') {
        return Some(device_id.to_uppercase());
    }

    if device_id.starts_with("\\\\.\\") && device_id.len() == 6 && device_id.ends_with(':') {
        return Some(device_id[4..6].to_uppercase());
    }

    None
}

/// Native FAT32 formatting implementation
fn native_format_fat32(device_id: &str) -> Result<()> {
    info!("Using native FAT32 formatting");

    let device_path = CString::new(device_id)?;

    let handle = unsafe {
        CreateFileA(
            device_path.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_NO_BUFFERING,
            ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err(anyhow::anyhow!("Failed to open device: {}", get_last_error_string()));
    }

    let mut disk_geometry = DISK_GEOMETRY_EX::default();
    let mut bytes_returned: DWORD = 0;

    let result = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
            ptr::null_mut(),
            0,
            &mut disk_geometry as *mut _ as *mut _,
            std::mem::size_of::<DISK_GEOMETRY_EX>() as u32,
            &mut bytes_returned,
            ptr::null_mut(),
        )
    };

    if result == 0 {
        unsafe { CloseHandle(handle); }
        return Err(anyhow::anyhow!("Failed to get device geometry"));
    }

    let total_sectors = unsafe { disk_geometry.DiskSize } as u64 / 
                        unsafe { disk_geometry.Geometry.BytesPerSector } as u64;
    let sector_size = unsafe { disk_geometry.Geometry.BytesPerSector } as u32;

    let boot_sector = create_fat32_boot_sector(total_sectors, sector_size)?;

    let mut bytes_written: DWORD = 0;
    let write_result = unsafe {
        WriteFile(
            handle,
            boot_sector.as_ptr() as *const _,
            boot_sector.len() as u32,
            &mut bytes_written,
            ptr::null_mut(),
        )
    };

    if write_result == 0 {
        unsafe { CloseHandle(handle); }
        return Err(anyhow::anyhow!("Failed to write FAT32 boot sector"));
    }

    let fat_table = create_fat32_table(total_sectors, sector_size)?;

    let fat1_offset = 32 * sector_size as i32;
    unsafe {
        SetFilePointer(handle, fat1_offset, ptr::null_mut(), FILE_BEGIN);
    }

    let write_result = unsafe {
        WriteFile(
            handle,
            fat_table.as_ptr() as *const _,
            fat_table.len() as u32,
            &mut bytes_written,
            ptr::null_mut(),
        )
    };

    if write_result == 0 {
        unsafe { CloseHandle(handle); }
        return Err(anyhow::anyhow!("Failed to write FAT32 table"));
    }

    unsafe { CloseHandle(handle); }

    info!("Native FAT32 formatting completed");
    Ok(())
}

/// Create FAT32 boot sector
fn create_fat32_boot_sector(total_sectors: u64, sector_size: u32) -> Result<Vec<u8>> {
    let mut boot_sector = vec![0u8; sector_size as usize];

    boot_sector[0] = 0xEB;
    boot_sector[1] = 0x58;
    boot_sector[2] = 0x90;

    boot_sector[3..11].copy_from_slice(b"MSDOS5.0");

    boot_sector[11..13].copy_from_slice(&sector_size.to_le_bytes()[..2]);
    boot_sector[13] = 8;
    boot_sector[14..16].copy_from_slice(&32u16.to_le_bytes());
    boot_sector[16] = 2;
    boot_sector[17..19].copy_from_slice(&0u16.to_le_bytes());
    boot_sector[19..21].copy_from_slice(&0u16.to_le_bytes());
    boot_sector[21] = 0xF8;
    boot_sector[22..24].copy_from_slice(&0u16.to_le_bytes());
    boot_sector[24..26].copy_from_slice(&63u16.to_le_bytes());
    boot_sector[26..28].copy_from_slice(&255u16.to_le_bytes());
    boot_sector[28..32].copy_from_slice(&0u32.to_le_bytes());

    let total_sectors_32 = if total_sectors > 0xFFFFFFFF { 0xFFFFFFFF } else { total_sectors as u32 };
    boot_sector[32..36].copy_from_slice(&total_sectors_32.to_le_bytes());

    let sectors_per_fat = calculate_fat32_sectors_per_fat(total_sectors, sector_size);
    boot_sector[36..40].copy_from_slice(&sectors_per_fat.to_le_bytes());
    boot_sector[40..42].copy_from_slice(&0u16.to_le_bytes());
    boot_sector[42..44].copy_from_slice(&0u16.to_le_bytes());
    boot_sector[44..48].copy_from_slice(&2u32.to_le_bytes());
    boot_sector[48..50].copy_from_slice(&1u16.to_le_bytes());
    boot_sector[50..52].copy_from_slice(&6u16.to_le_bytes());
    boot_sector[52..64].copy_from_slice(&[0; 12]);
    boot_sector[64] = 0x80;
    boot_sector[65] = 0;
    boot_sector[66] = 0x29;
    boot_sector[67..71].copy_from_slice(&generate_volume_serial().to_le_bytes());

    let label = b"SIDOZDEV   ";
    boot_sector[71..82].copy_from_slice(label);
    boot_sector[82..90].copy_from_slice(b"FAT32   ");
    boot_sector[510] = 0x55;
    boot_sector[511] = 0xAA;

    Ok(boot_sector)
}

/// Calculate sectors per FAT for FAT32
fn calculate_fat32_sectors_per_fat(total_sectors: u64, sector_size: u32) -> u32 {
    let sectors_per_cluster = 8u32;
    let total_clusters = (total_sectors as u32) / sectors_per_cluster;
    let fat_entries_per_sector = sector_size / 4;

    (total_clusters + fat_entries_per_sector - 1) / fat_entries_per_sector
}

/// Create FAT32 table
fn create_fat32_table(total_sectors: u64, sector_size: u32) -> Result<Vec<u8>> {
    let sectors_per_fat = calculate_fat32_sectors_per_fat(total_sectors, sector_size);
    let mut fat_table = vec![0u8; (sectors_per_fat * sector_size) as usize];

    fat_table[0..4].copy_from_slice(&0x0FFFFFF8u32.to_le_bytes());
    fat_table[4..8].copy_from_slice(&0x0FFFFFFFu32.to_le_bytes());
    fat_table[8..12].copy_from_slice(&0x0FFFFFFFu32.to_le_bytes());

    Ok(fat_table)
}

/// Generate volume serial number
fn generate_volume_serial() -> u32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen()
}

/// Native NTFS formatting (placeholder)
fn native_format_ntfs(_device_id: &str) -> Result<()> {
    warn!("Native NTFS formatting not fully implemented, using format command");
    Err(anyhow::anyhow!("Native NTFS formatting requires Windows format command"))
}

/// Native exFAT formatting (placeholder)
fn native_format_exfat(_device_id: &str) -> Result<()> {
    warn!("Native exFAT formatting not fully implemented, using format command");
    Err(anyhow::anyhow!("Native exFAT formatting requires Windows format command"))
}

#[repr(C)]
#[derive(Default)]
struct DISK_GEOMETRY_EX {
    Geometry: DISK_GEOMETRY,
    DiskSize: i64,
    Data: [u8; 1],
}

#[repr(C)]
#[derive(Default)]
struct DISK_GEOMETRY {
    Cylinders: i64,
    MediaType: u32,
    TracksPerCylinder: DWORD,
    SectorsPerTrack: DWORD,
    BytesPerSector: DWORD,
}
