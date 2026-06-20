//! Platform Utilities
//! 
//! Windows-specific utilities and error handling.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::FormatMessageW;
use winapi::um::winnt::{LPWSTR, WCHAR};
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::HRESULT;

/// Get the last Windows error as a string
pub fn get_last_error_string() -> String {
    unsafe {
        let error_code = GetLastError();
        format_error_code(error_code)
    }
}

/// Format a Windows error code to string
pub fn format_error_code(error_code: DWORD) -> String {
    const FORMAT_MESSAGE_ALLOCATE_BUFFER: DWORD = 0x00000100;
    const FORMAT_MESSAGE_FROM_SYSTEM: DWORD = 0x00001000;
    const FORMAT_MESSAGE_IGNORE_INSERTS: DWORD = 0x00000200;

    unsafe {
        let mut buffer: LPWSTR = std::ptr::null_mut();

        let length = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            std::ptr::null(),
            error_code,
            0, // Default language
            &mut buffer as *mut LPWSTR as LPWSTR,
            0,
            std::ptr::null_mut(),
        );

        if length == 0 || buffer.is_null() {
            return format!("Error code: {}", error_code);
        }

        let message = std::slice::from_raw_parts(buffer, length as usize);
        let os_string = OsString::from_wide(message);
        let message_string = os_string.to_string_lossy().to_string();

        // Free the allocated buffer
        winapi::um::winbase::LocalFree(buffer as *mut _);

        message_string.trim().to_string()
    }
}

/// Check if running on Windows 10 or later
pub fn is_windows_10_or_later() -> bool {
    use winapi::um::sysinfoapi::GetVersionExW;
    use winapi::um::winnt::OSVERSIONINFOW;

    unsafe {
        let mut os_info = OSVERSIONINFOW {
            dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
            dwMajorVersion: 0,
            dwMinorVersion: 0,
            dwBuildNumber: 0,
            dwPlatformId: 0,
            szCSDVersion: [0; 128],
        };

        if GetVersionExW(&mut os_info) != 0 {
            os_info.dwMajorVersion >= 10
        } else {
            false
        }
    }
}

/// Check if running with administrator privileges
pub fn is_elevated() -> bool {
    use winapi::um::securitybaseapi::CheckTokenMembership;
    use winapi::um::winnt::{PSID, SID_IDENTIFIER_AUTHORITY, SECURITY_NT_AUTHORITY};
    use winapi::shared::minwindef::BOOL;

    unsafe {
        let mut administrators_group: PSID = std::ptr::null_mut();
        let mut nt_authority = SID_IDENTIFIER_AUTHORITY {
            Value: [0, 0, 0, 0, 0, 5],
        };

        let result = winapi::um::securitybaseapi::AllocateAndInitializeSid(
            &mut nt_authority,
            2,
            winapi::um::winnt::SECURITY_BUILTIN_DOMAIN_RID as u32,
            winapi::um::winnt::DOMAIN_ALIAS_RID_ADMINS as u32,
            0, 0, 0, 0, 0, 0,
            &mut administrators_group,
        );

        if result == 0 {
            return false;
        }

        let mut is_member: BOOL = 0;
        let check_result = CheckTokenMembership(
            std::ptr::null_mut(),
            administrators_group,
            &mut is_member,
        );

        winapi::um::securitybaseapi::FreeSid(administrators_group);

        check_result != 0 && is_member != 0
    }
}

/// Request elevation (restart as administrator)
pub fn request_elevation() -> Result<(), String> {
    use std::process::Command;
    use std::env;

    let current_exe = env::current_exe()
        .map_err(|e| format!("Failed to get current executable: {}", e))?;

    let result = Command::new("powershell")
        .args(&[
            "-Command",
            "Start-Process",
            "-FilePath",
            current_exe.to_str().unwrap_or("sidozdev.exe"),
            "-Verb",
            "runAs",
        ])
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to request elevation: {}", e)),
    }
}

/// Get Windows version string
pub fn get_windows_version() -> String {
    use winapi::um::sysinfoapi::GetVersionExW;
    use winapi::um::winnt::OSVERSIONINFOW;

    unsafe {
        let mut os_info = OSVERSIONINFOW {
            dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
            dwMajorVersion: 0,
            dwMinorVersion: 0,
            dwBuildNumber: 0,
            dwPlatformId: 0,
            szCSDVersion: [0; 128],
        };

        if GetVersionExW(&mut os_info) != 0 {
            format!("Windows {}.{} (Build {})", 
                os_info.dwMajorVersion, 
                os_info.dwMinorVersion, 
                os_info.dwBuildNumber)
        } else {
            "Unknown Windows Version".to_string()
        }
    }
}

/// Check if a process is running with elevated privileges
pub fn check_elevation() -> Result<bool, String> {
    Ok(is_elevated())
}
