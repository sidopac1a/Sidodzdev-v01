//! Partition Management
//! 
//! Handles partition table creation (MBR/GPT) and boot record installation.

use std::ptr;
use std::ffi::CString;
use winapi::um::fileapi::*;
use winapi::um::handleapi::*;
use winapi::um::winioctl::*;
use winapi::um::winnt::*;
use winapi::shared::minwindef::*;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};

use crate::core::app_state::{BootMode, PartitionScheme};
use crate::utils::platform::get_last_error_string;

/// Create partition table on the device
pub fn create_partition_table(
    device_id: &str,
    scheme: PartitionScheme,
    boot_mode: BootMode,
) -> Result<()> {
    info!("Creating {:?} partition table on {}", scheme, device_id);

    match scheme {
        PartitionScheme::Mbr => create_mbr_partition_table(device_id, boot_mode),
        PartitionScheme::Gpt => create_gpt_partition_table(device_id, boot_mode),
    }
}

/// Create MBR partition table
fn create_mbr_partition_table(device_id: &str, boot_mode: BootMode) -> Result<()> {
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

    // Clean the first 1MB to remove any existing partition table
    let clean_buffer = vec![0u8; 1024 * 1024];
    let mut bytes_written: DWORD = 0;

    let write_result = unsafe {
        WriteFile(
            handle,
            clean_buffer.as_ptr() as *const _,
            clean_buffer.len() as u32,
            &mut bytes_written,
            ptr::null_mut(),
        )
    };

    if write_result == 0 {
        unsafe { CloseHandle(handle); }
        return Err(anyhow::anyhow!("Failed to clean device: {}", get_last_error_string()));
    }

    // Create MBR structure
    let mut mbr = MasterBootRecord::new();

    // Set boot code based on boot mode
    match boot_mode {
        BootMode::BiosLegacy => {
            mbr.set_boot_code(&BIOS_BOOT_CODE);
        }
        BootMode::Uefi | BootMode::UefiSecureBoot => {
            // For UEFI with MBR, we use protective MBR
            mbr.set_protective_mbr();
        }
    }

    // Create single partition covering the whole disk
    mbr.create_single_partition();

    // Write MBR to disk
    let mbr_bytes = mbr.to_bytes();
    let mut bytes_written: DWORD = 0;

    unsafe {
        SetFilePointer(handle, 0, ptr::null_mut(), FILE_BEGIN);
    }

    let write_result = unsafe {
        WriteFile(
            handle,
            mbr_bytes.as_ptr() as *const _,
            mbr_bytes.len() as u32,
            &mut bytes_written,
            ptr::null_mut(),
        )
    };

    unsafe { CloseHandle(handle); }

    if write_result == 0 {
        return Err(anyhow::anyhow!("Failed to write MBR: {}", get_last_error_string()));
    }

    info!("MBR partition table created successfully");
    Ok(())
}

/// Create GPT partition table
fn create_gpt_partition_table(device_id: &str, boot_mode: BootMode) -> Result<()> {
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

    // Get device size
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

    let device_size = unsafe { disk_geometry.DiskSize } as u64;
    let sector_size = unsafe { disk_geometry.Geometry.BytesPerSector } as u64;

    // Clean first 2MB and last 2MB for GPT headers
    let clean_buffer = vec![0u8; 2 * 1024 * 1024];
    let mut bytes_written: DWORD = 0;

    unsafe {
        WriteFile(
            handle,
            clean_buffer.as_ptr() as *const _,
            clean_buffer.len() as u32,
            &mut bytes_written,
            ptr::null_mut(),
        );
    }

    // Create GPT structures
    let gpt_header = GptHeader::new(device_size, sector_size);
    let gpt_entries = GptEntryArray::new_single_partition(device_size, sector_size);

    // Write protective MBR at LBA 0
    let mut protective_mbr = MasterBootRecord::new();
    protective_mbr.set_protective_mbr();
    let mbr_bytes = protective_mbr.to_bytes();

    unsafe {
        SetFilePointer(handle, 0, ptr::null_mut(), FILE_BEGIN);
        WriteFile(
            handle,
            mbr_bytes.as_ptr() as *const _,
            mbr_bytes.len() as u32,
            &mut bytes_written,
            ptr::null_mut(),
        );
    }

    // Write GPT header at LBA 1
    let gpt_header_bytes = gpt_header.to_bytes();
    let gpt_header_offset = sector_size as i32;

    unsafe {
        SetFilePointer(handle, gpt_header_offset, ptr::null_mut(), FILE_BEGIN);
        WriteFile(
            handle,
            gpt_header_bytes.as_ptr() as *const _,
            gpt_header_bytes.len() as u32,
            &mut bytes_written,
            ptr::null_mut(),
        );
    }

    // Write GPT entries
    let gpt_entries_bytes = gpt_entries.to_bytes();
    let entries_offset = sector_size as i32 * 2;

    unsafe {
        SetFilePointer(handle, entries_offset, ptr::null_mut(), FILE_BEGIN);
        WriteFile(
            handle,
            gpt_entries_bytes.as_ptr() as *const _,
            gpt_entries_bytes.len() as u32,
            &mut bytes_written,
            ptr::null_mut(),
        );
    }

    // Write backup GPT at end of disk
    let backup_gpt_offset = (device_size - sector_size) as i32;

    unsafe {
        SetFilePointer(handle, backup_gpt_offset, ptr::null_mut(), FILE_BEGIN);
        WriteFile(
            handle,
            gpt_header_bytes.as_ptr() as *const _,
            gpt_header_bytes.len() as u32,
            &mut bytes_written,
            ptr::null_mut(),
        );
    }

    unsafe { CloseHandle(handle); }

    info!("GPT partition table created successfully");
    Ok(())
}

/// Master Boot Record structure
#[derive(Debug)]
struct MasterBootRecord {
    boot_code: [u8; 440],
    disk_signature: [u8; 4],
    reserved: [u8; 2],
    partition_table: [PartitionEntry; 4],
    boot_signature: [u8; 2],
}

impl MasterBootRecord {
    fn new() -> Self {
        Self {
            boot_code: [0; 440],
            disk_signature: [0; 4],
            reserved: [0; 2],
            partition_table: [PartitionEntry::new(); 4],
            boot_signature: [0x55, 0xAA],
        }
    }

    fn set_boot_code(&mut self, code: &[u8]) {
        let len = std::cmp::min(code.len(), 440);
        self.boot_code[..len].copy_from_slice(&code[..len]);
    }

    fn set_protective_mbr(&mut self) {
        // Set partition type to 0xEE (GPT protective)
        self.partition_table[0].partition_type = 0xEE;
        self.partition_table[0].start_lba = 1;
        self.partition_table[0].size_lba = 0xFFFFFFFF;
    }

    fn create_single_partition(&mut self) {
        self.partition_table[0].bootable = 0x80;
        self.partition_table[0].partition_type = 0x0C; // FAT32 LBA
        self.partition_table[0].start_lba = 2048; // 1MB alignment
        self.partition_table[0].size_lba = 0; // Will be set based on disk size
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(512);
        bytes.extend_from_slice(&self.boot_code);
        bytes.extend_from_slice(&self.disk_signature);
        bytes.extend_from_slice(&self.reserved);
        for entry in &self.partition_table {
            bytes.extend_from_slice(&entry.to_bytes());
        }
        bytes.extend_from_slice(&self.boot_signature);
        bytes
    }
}

/// Partition entry in MBR
#[derive(Debug, Clone, Copy)]
struct PartitionEntry {
    bootable: u8,
    start_chs: [u8; 3],
    partition_type: u8,
    end_chs: [u8; 3],
    start_lba: u32,
    size_lba: u32,
}

impl PartitionEntry {
    fn new() -> Self {
        Self {
            bootable: 0,
            start_chs: [0; 3],
            partition_type: 0,
            end_chs: [0; 3],
            start_lba: 0,
            size_lba: 0,
        }
    }

    fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0] = self.bootable;
        bytes[1..4].copy_from_slice(&self.start_chs);
        bytes[4] = self.partition_type;
        bytes[5..8].copy_from_slice(&self.end_chs);
        bytes[8..12].copy_from_slice(&self.start_lba.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.size_lba.to_le_bytes());
        bytes
    }
}

/// GPT Header structure
#[derive(Debug)]
struct GptHeader {
    signature: [u8; 8],
    revision: [u8; 4],
    header_size: u32,
    crc32: u32,
    reserved: u32,
    current_lba: u64,
    backup_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    disk_guid: [u8; 16],
    partition_entry_lba: u64,
    num_partition_entries: u32,
    partition_entry_size: u32,
    partition_entry_crc32: u32,
}

impl GptHeader {
    fn new(disk_size: u64, sector_size: u64) -> Self {
        let total_sectors = disk_size / sector_size;

        Self {
            signature: *b"EFI PART",
            revision: [0x00, 0x00, 0x01, 0x00], // GPT revision 1.0
            header_size: 92,
            crc32: 0, // Would need proper CRC32 calculation
            reserved: 0,
            current_lba: 1,
            backup_lba: total_sectors - 1,
            first_usable_lba: 34,
            last_usable_lba: total_sectors - 34,
            disk_guid: generate_guid(),
            partition_entry_lba: 2,
            num_partition_entries: 128,
            partition_entry_size: 128,
            partition_entry_crc32: 0,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(512);
        bytes.extend_from_slice(&self.signature);
        bytes.extend_from_slice(&self.revision);
        bytes.extend_from_slice(&self.header_size.to_le_bytes());
        bytes.extend_from_slice(&self.crc32.to_le_bytes());
        bytes.extend_from_slice(&self.reserved.to_le_bytes());
        bytes.extend_from_slice(&self.current_lba.to_le_bytes());
        bytes.extend_from_slice(&self.backup_lba.to_le_bytes());
        bytes.extend_from_slice(&self.first_usable_lba.to_le_bytes());
        bytes.extend_from_slice(&self.last_usable_lba.to_le_bytes());
        bytes.extend_from_slice(&self.disk_guid);
        bytes.extend_from_slice(&self.partition_entry_lba.to_le_bytes());
        bytes.extend_from_slice(&self.num_partition_entries.to_le_bytes());
        bytes.extend_from_slice(&self.partition_entry_size.to_le_bytes());
        bytes.extend_from_slice(&self.partition_entry_crc32.to_le_bytes());
        bytes.resize(512, 0); // Pad to sector size
        bytes
    }
}

/// GPT Partition Entry
#[derive(Debug)]
struct GptPartitionEntry {
    partition_type_guid: [u8; 16],
    unique_guid: [u8; 16],
    starting_lba: u64,
    ending_lba: u64,
    attributes: u64,
    name: [u16; 36],
}

impl GptPartitionEntry {
    fn new_efi_system_partition(start_lba: u64, end_lba: u64) -> Self {
        // EFI System Partition GUID: C12A7328-F81F-11D2-BA4B-00A0C93EC93B
        let esp_guid = [
            0x28, 0x73, 0x2A, 0xC1,
            0x1F, 0xF8,
            0xD2, 0x11,
            0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B
        ];

        let mut name = [0u16; 36];
        let name_str = "EFI System Partition";
        for (i, c) in name_str.encode_utf16().enumerate() {
            if i < 36 {
                name[i] = c;
            }
        }

        Self {
            partition_type_guid: esp_guid,
            unique_guid: generate_guid(),
            starting_lba: start_lba,
            ending_lba: end_lba,
            attributes: 0,
            name,
        }
    }

    fn to_bytes(&self) -> [u8; 128] {
        let mut bytes = [0u8; 128];
        bytes[0..16].copy_from_slice(&self.partition_type_guid);
        bytes[16..32].copy_from_slice(&self.unique_guid);
        bytes[32..40].copy_from_slice(&self.starting_lba.to_le_bytes());
        bytes[40..48].copy_from_slice(&self.ending_lba.to_le_bytes());
        bytes[48..56].copy_from_slice(&self.attributes.to_le_bytes());
        for (i, &c) in self.name.iter().enumerate() {
            bytes[56 + i * 2..58 + i * 2].copy_from_slice(&c.to_le_bytes());
        }
        bytes
    }
}

/// GPT Entry Array
#[derive(Debug)]
struct GptEntryArray {
    entries: Vec<GptPartitionEntry>,
}

impl GptEntryArray {
    fn new_single_partition(disk_size: u64, sector_size: u64) -> Self {
        let total_sectors = disk_size / sector_size;
        let start_lba = 2048; // 1MB alignment
        let end_lba = total_sectors - 2048;

        Self {
            entries: vec![
                GptPartitionEntry::new_efi_system_partition(start_lba, end_lba),
            ],
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(128 * 128); // 128 entries * 128 bytes each
        for entry in &self.entries {
            bytes.extend_from_slice(&entry.to_bytes());
        }
        // Pad to full sector
        while bytes.len() % 512 != 0 {
            bytes.push(0);
        }
        bytes
    }
}

/// Generate a random GUID
fn generate_guid() -> [u8; 16] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut guid = [0u8; 16];
    rng.fill(&mut guid);
    // Set version (4) and variant bits
    guid[6] = (guid[6] & 0x0F) | 0x40;
    guid[8] = (guid[8] & 0x3F) | 0x80;
    guid
}

/// BIOS boot code (simplified - would be actual boot code in production)
static BIOS_BOOT_CODE: [u8; 440] = {
    let mut code = [0u8; 440];
    // This would contain actual x86 boot code
    // For now, it's a placeholder
    code[0] = 0xEB; // JMP instruction
    code[1] = 0x3C; // Offset
    code
};

// Include the DISK_GEOMETRY_EX structure definition
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
