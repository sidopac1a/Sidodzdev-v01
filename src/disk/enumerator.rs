//! USB Device Enumeration
//! 
//! Cross-platform USB storage device detection using Windows SetupAPI and WMI.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use winapi::shared::minwindef::{DWORD, TRUE, FALSE, BYTE};
use winapi::shared::guiddef::GUID;
use winapi::um::setupapi::*;
use winapi::um::cfgmgr32::*;
use winapi::um::fileapi::*;
use winapi::um::handleapi::*;
use winapi::um::ioapiset::*;
use winapi::um::winioctl::*;
use winapi::um::winnt::*;
use winapi::um::winbase::*;

use crate::disk::device::UsbDevice;
use crate::utils::platform::get_last_error_string;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};

/// GUID for USB storage devices
const GUID_DEVINTERFACE_DISK: GUID = GUID {
    Data1: 0x53f56307,
    Data2: 0xb6bf,
    Data3: 0x11d0,
    Data4: [0x94, 0xf2, 0x00, 0xa0, 0xc9, 0x1e, 0xfb, 0x8b],
};

/// Enumerate all USB storage devices connected to the system
pub fn enumerate_usb_devices() -> Result<Vec<UsbDevice>> {
    info!("Starting USB device enumeration");

    let mut devices = Vec::new();

    // Get device information set for all disk devices
    let device_info_set = unsafe {
        SetupDiGetClassDevsW(
            &GUID_DEVINTERFACE_DISK,
            ptr::null(),
            ptr::null_mut(),
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        )
    };

    if device_info_set == INVALID_HANDLE_VALUE {
        return Err(anyhow::anyhow!("Failed to get device info set: {}", get_last_error_string()));
    }

    let mut device_index = 0;
    let mut device_interface_data = SP_DEVICE_INTERFACE_DATA {
        cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
        InterfaceClassGuid: GUID::default(),
        Flags: 0,
        Reserved: 0,
    };

    // Enumerate all device interfaces
    while unsafe {
        SetupDiEnumDeviceInterfaces(
            device_info_set,
            ptr::null_mut(),
            &GUID_DEVINTERFACE_DISK,
            device_index,
            &mut device_interface_data,
        )} == TRUE
    {
        // Get required buffer size
        let mut required_size: DWORD = 0;
        unsafe {
            SetupDiGetDeviceInterfaceDetailW(
                device_info_set,
                &device_interface_data,
                ptr::null_mut(),
                0,
                &mut required_size,
                ptr::null_mut(),
            );
        }

        if required_size == 0 {
            device_index += 1;
            continue;
        }

        // Allocate buffer for device interface detail
        let mut detail_data_buffer = vec![0u8; required_size as usize];
        let detail_data = detail_data_buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
        unsafe {
            (*detail_data).cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;
        }

        let mut dev_info_data = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ClassGuid: GUID::default(),
            DevInst: 0,
            Reserved: 0,
        };

        // Get device interface detail
        let success = unsafe {
            SetupDiGetDeviceInterfaceDetailW(
                device_info_set,
                &device_interface_data,
                detail_data,
                required_size,
                ptr::null_mut(),
                &mut dev_info_data,
            )
        };

        if success == TRUE {
            // Extract device path
            let device_path = unsafe {
                let path_ptr = (*detail_data).DevicePath.as_ptr();
                let path_len = wide_string_length(path_ptr);
                let path_slice = std::slice::from_raw_parts(path_ptr, path_len);
                OsString::from_wide(path_slice).to_string_lossy().to_string()
            };

            debug!("Found device path: {}", device_path);

            // Check if this is a USB device by looking at the bus type
            if let Ok(device) = get_device_info(device_info_set, &dev_info_data, &device_path) {
                if is_usb_device(&device_path) {
                    devices.push(device);
                }
            }
        }

        device_index += 1;
    }

    unsafe {
        SetupDiDestroyDeviceInfoList(device_info_set);
    }

    info!("Found {} USB storage device(s)", devices.len());
    Ok(devices)
}

/// Get detailed information about a device
fn get_device_info(
    device_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
    device_path: &str,
) -> Result<UsbDevice> {
    // Get friendly name
    let friendly_name = get_device_property_string(
        device_info_set,
        dev_info_data,
        SPDRP_FRIENDLYNAME,
    ).unwrap_or_else(|| "Unknown Device".to_string());

    // Get device description
    let description = get_device_property_string(
        device_info_set,
        dev_info_data,
        SPDRP_DEVICEDESC,
    ).unwrap_or_else(|| "USB Device".to_string());

    // Get hardware IDs to extract VID/PID
    let hardware_ids = get_device_property_string(
        device_info_set,
        dev_info_data,
        SPDRP_HARDWAREID,
    ).unwrap_or_default();

    let vid_pid = extract_vid_pid(&hardware_ids);

    // Get device capacity and other info by opening the device
    let (capacity, sector_size, physical_sector_size) = get_device_geometry(device_path)?;

    // Get drive letters
    let drive_letters = get_drive_letters_for_device(device_path);

    // Determine interface type
    let interface_type = get_usb_interface_type(&hardware_ids);

    // Extract vendor and product from description
    let (vendor, product) = parse_vendor_product(&description);

    Ok(UsbDevice {
        device_id: device_path.clone(),
        friendly_name: friendly_name.clone(),
        vendor,
        product,
        capacity,
        free_space: capacity, // Will be updated later
        interface_type,
        device_path: device_path.to_string(),
        drive_letters,
        serial_number: extract_serial_number(&hardware_ids),
        is_removable: true,
        is_mounted: !drive_letters.is_empty(),
        current_file_system: None,
        partition_scheme: None,
        vid_pid,
    })
}

/// Get device geometry (capacity, sector size)
fn get_device_geometry(device_path: &str) -> Result<(u64, u32, u32)> {
    let wide_path: Vec<u16> = device_path.encode_utf16().chain(std::iter::once(0)).collect();

    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            0, // No access needed for geometry
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null_mut(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err(anyhow::anyhow!("Failed to open device: {}", get_last_error_string()));
    }

    let mut disk_geometry = DISK_GEOMETRY_EX::default();
    let mut bytes_returned: DWORD = 0;

    let success = unsafe {
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

    unsafe {
        CloseHandle(handle);
    }

    if success == 0 {
        return Err(anyhow::anyhow!("Failed to get device geometry"));
    }

    let capacity = unsafe { disk_geometry.DiskSize };
    let sector_size = unsafe { disk_geometry.Geometry.BytesPerSector };

    Ok((capacity as u64, sector_size, sector_size))
}

/// Get drive letters associated with a physical device
fn get_drive_letters_for_device(device_path: &str) -> Vec<String> {
    // This is a simplified implementation
    // In production, you'd use WMI or volume management APIs
    Vec::new()
}

/// Check if a device is a USB device by its path
fn is_usb_device(device_path: &str) -> bool {
    device_path.to_lowercase().contains("usb") || 
    device_path.to_lowercase().contains("usbstor")
}

/// Get USB interface type from hardware IDs
fn get_usb_interface_type(hardware_ids: &str) -> String {
    if hardware_ids.contains("USB\\VID_") {
        // Check for USB 3.0 indicators
        if hardware_ids.contains("3.0") || hardware_ids.contains("USB30") {
            "USB 3.0".to_string()
        } else if hardware_ids.contains("2.0") || hardware_ids.contains("USB20") {
            "USB 2.0".to_string()
        } else {
            "USB".to_string()
        }
    } else {
        "USB".to_string()
    }
}

/// Extract VID:PID from hardware IDs
fn extract_vid_pid(hardware_ids: &str) -> String {
    // Parse "USB\\VID_1234&PID_5678"
    let lower = hardware_ids.to_lowercase();
    if let Some(vid_start) = lower.find("vid_") {
        if let Some(pid_start) = lower.find("pid_") {
            let vid = &lower[vid_start + 4..vid_start + 8];
            let pid = &lower[pid_start + 4..pid_start + 8];
            return format!("{}:{}", vid, pid);
        }
    }
    "Unknown".to_string()
}

/// Extract serial number from hardware IDs
fn extract_serial_number(hardware_ids: &str) -> String {
    // The serial number is typically after the last \\ in the hardware ID
    if let Some(last_backslash) = hardware_ids.rfind('\\') {
        hardware_ids[last_backslash + 1..].to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Parse vendor and product from description
fn parse_vendor_product(description: &str) -> (String, String) {
    let parts: Vec<&str> = description.splitn(2, ' ').collect();
    if parts.len() >= 2 {
        (parts[0].to_string(), parts[1..].join(" "))
    } else {
        ("Unknown".to_string(), description.to_string())
    }
}

/// Get a string property from device info
fn get_device_property_string(
    device_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
    property: DWORD,
) -> Option<String> {
    let mut buffer = vec![0u8; 512];
    let mut required_size: DWORD = 0;
    let mut property_type: DWORD = 0;

    let success = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            device_info_set,
            dev_info_data as *const _ as *mut _,
            property,
            &mut property_type,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            &mut required_size,
        )
    };

    if success == TRUE && property_type == REG_SZ {
        let wide_slice = unsafe {
            std::slice::from_raw_parts(
                buffer.as_ptr() as *const u16,
                required_size as usize / 2,
            )
        };

        let os_string = OsString::from_wide(wide_slice);
        let string = os_string.to_string_lossy().to_string();
        // Remove null terminator
        string.trim_end_matches('\0').to_string().into()
    } else {
        None
    }
}

/// Calculate length of a wide string
unsafe fn wide_string_length(ptr: *const u16) -> usize {
    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    len
}

/// DISK_GEOMETRY_EX structure
#[repr(C)]
#[derive(Default)]
struct DISK_GEOMETRY_EX {
    Geometry: DISK_GEOMETRY,
    DiskSize: i64,
    Data: [BYTE; 1],
}

/// DISK_GEOMETRY structure
#[repr(C)]
#[derive(Default)]
struct DISK_GEOMETRY {
    Cylinders: i64,
    MediaType: u32,
    TracksPerCylinder: DWORD,
    SectorsPerTrack: DWORD,
    BytesPerSector: DWORD,
}
