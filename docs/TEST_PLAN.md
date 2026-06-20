# Sidozdev Test Plan

## Test Strategy Overview

This test plan covers all aspects of Sidozdev testing including unit tests, integration tests, system tests, and acceptance tests.

## 1. Unit Tests

### 1.1 Core Module Tests

#### AppState Tests
```rust
#[test]
fn test_app_state_creation() {
    let state = AppState::new();
    assert!(state.devices.is_empty());
    assert!(state.config.selected_device.is_none());
    assert!(state.config.iso_path.is_none());
    assert!(!state.is_running);
    assert_eq!(state.progress.phase, OperationPhase::Idle);
}

#[test]
fn test_add_log() {
    let mut state = AppState::new();
    state.add_log(LogLevel::Info, "Test message");
    assert_eq!(state.logs.len(), 1);
    assert_eq!(state.logs[0].message, "Test message");
    assert_eq!(state.logs[0].level, LogLevel::Info);
}

#[test]
fn test_log_limit() {
    let mut state = AppState::new();
    for i in 0..1005 {
        state.add_log(LogLevel::Info, format!("Message {}", i));
    }
    assert_eq!(state.logs.len(), 1000);
}

#[test]
fn test_is_ready() {
    let mut state = AppState::new();
    assert!(!state.is_ready());

    state.config.selected_device = Some("test_device".to_string());
    assert!(!state.is_ready());

    state.config.iso_path = Some(PathBuf::from("test.iso"));
    assert!(state.is_ready());

    state.is_running = true;
    assert!(!state.is_ready());
}

#[test]
fn test_cancel_request() {
    let mut state = AppState::new();
    state.request_cancel();
    assert!(state.cancel_requested);
    assert_eq!(state.logs[0].level, LogLevel::Warning);
}
```

#### Config Tests
```rust
#[test]
fn test_default_config() {
    let config = OperationConfig::default();
    assert_eq!(config.boot_mode, BootMode::Uefi);
    assert_eq!(config.partition_scheme, PartitionScheme::Gpt);
    assert_eq!(config.file_system, FileSystem::Fat32);
    assert_eq!(config.hash_algorithm, HashAlgorithm::Sha256);
    assert!(config.verify_after_write);
    assert!(config.quick_format);
    assert!(!config.bad_block_check);
}

#[test]
fn test_app_settings_default() {
    let settings = AppSettings::default();
    assert_eq!(settings.theme, Theme::System);
    assert_eq!(settings.language, Language::English);
    assert_eq!(settings.log_level, "info");
}

#[test]
fn test_settings_save_load() {
    let settings = AppSettings::default();
    settings.save().unwrap();

    let loaded = AppSettings::load();
    assert_eq!(loaded.theme, settings.theme);
    assert_eq!(loaded.language, settings.language);
}
```

### 1.2 Disk Module Tests

#### Device Tests
```rust
#[test]
fn test_usb_device_display() {
    let device = UsbDevice {
        device_id: "\\\\.\\PhysicalDrive1".to_string(),
        friendly_name: "Test USB".to_string(),
        vendor: "SanDisk".to_string(),
        product: "Cruzer".to_string(),
        capacity: 16_000_000_000, // 16GB
        free_space: 8_000_000_000,
        interface_type: "USB 3.0".to_string(),
        device_path: "\\\\.\\PhysicalDrive1".to_string(),
        drive_letters: vec!["E:".to_string()],
        serial_number: "1234567890".to_string(),
        is_removable: true,
        is_mounted: true,
        current_file_system: Some("FAT32".to_string()),
        partition_scheme: Some("MBR".to_string()),
        vid_pid: "0781:5567".to_string(),
    };

    assert_eq!(device.capacity_display(), "14.90 GB");
    assert_eq!(device.free_space_display(), "7.45 GB");
    assert!(device.display_summary().contains("Test USB"));
}
```

#### Partition Tests
```rust
#[test]
fn test_mbr_creation() {
    let mbr = MasterBootRecord::new();
    assert_eq!(mbr.boot_signature, [0x55, 0xAA]);

    let bytes = mbr.to_bytes();
    assert_eq!(bytes.len(), 512);
    assert_eq!(bytes[510], 0x55);
    assert_eq!(bytes[511], 0xAA);
}

#[test]
fn test_mbr_partition_entry() {
    let mut entry = PartitionEntry::new();
    entry.bootable = 0x80;
    entry.partition_type = 0x0C;
    entry.start_lba = 2048;
    entry.size_lba = 1000000;

    let bytes = entry.to_bytes();
    assert_eq!(bytes.len(), 16);
    assert_eq!(bytes[0], 0x80);
    assert_eq!(bytes[4], 0x0C);
}

#[test]
fn test_gpt_header() {
    let header = GptHeader::new(32_000_000_000, 512);
    let bytes = header.to_bytes();
    assert_eq!(bytes.len(), 512);
    assert_eq!(&bytes[0..8], b"EFI PART");
}
```

### 1.3 ISO Module Tests

#### Parser Tests
```rust
#[test]
fn test_iso_type_display() {
    assert_eq!(IsoType::Iso9660.to_string(), "ISO 9660");
    assert_eq!(IsoType::HybridIso.to_string(), "Hybrid ISO");
    assert_eq!(IsoType::Img.to_string(), "IMG/Raw Image");
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(512), "512.00 B");
    assert_eq!(format_bytes(1024), "1.00 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(format_bytes(1024u64 * 1024 * 1024 * 1024), "1.00 TB");
}

#[test]
fn test_boot_mode_display() {
    assert_eq!(BootMode::BiosLegacy.to_string(), "BIOS (Legacy)");
    assert_eq!(BootMode::Uefi.to_string(), "UEFI");
    assert_eq!(BootMode::UefiSecureBoot.to_string(), "UEFI + Secure Boot");
}
```

### 1.4 Utils Module Tests

#### Hash Tests
```rust
#[test]
fn test_compare_hashes() {
    assert!(compare_hashes("abc123", "ABC123"));
    assert!(!compare_hashes("abc123", "def456"));
}

#[test]
fn test_format_hash() {
    let hash = "1234567890abcdef1234567890abcdef";
    assert_eq!(format_hash(hash, 10), "123456...cdef");
    assert_eq!(format_hash(hash, 40), hash);
}
```

## 2. Integration Tests

### 2.1 Event System Tests
```rust
#[test]
fn test_event_bus() {
    let bus = EventBus::new();

    // Send event from backend to UI
    bus.ui_sender.send(UiEvent::DevicesUpdated(vec![])).unwrap();
    let event = bus.ui_receiver.recv().unwrap();
    matches!(event, UiEvent::DevicesUpdated(_));

    // Send command from UI to backend
    bus.backend_sender.send(BackendCommand::RefreshDevices).unwrap();
    let cmd = bus.backend_receiver.recv().unwrap();
    matches!(cmd, BackendCommand::RefreshDevices);
}
```

### 2.2 Worker Thread Tests
```rust
#[tokio::test]
async fn test_worker_refresh_devices() {
    let state = create_shared_state();
    let bus = Arc::new(Mutex::new(EventBus::new()));
    let worker = Worker::new().unwrap();

    // Send refresh command
    bus.lock().unwrap().backend_sender
        .send(BackendCommand::RefreshDevices).unwrap();

    // Wait for response
    let event = bus.lock().unwrap().ui_receiver.recv_timeout(Duration::from_secs(5));
    assert!(event.is_ok());
}
```

### 2.3 ISO Validation Tests
```rust
#[tokio::test]
async fn test_iso_validation() {
    let test_iso = create_test_iso(); // Helper function
    let info = parse_iso(&test_iso).await.unwrap();

    assert_eq!(info.iso_type, IsoType::Iso9660);
    assert!(info.size > 0);
    assert!(!info.label.is_empty());
}

#[tokio::test]
async fn test_hash_calculation() {
    let test_file = create_test_file(b"Hello, World!");

    let md5 = validate_iso_hash(&test_file, HashType::Md5).await.unwrap();
    assert_eq!(md5.len(), 32);

    let sha1 = validate_iso_hash(&test_file, HashType::Sha1).await.unwrap();
    assert_eq!(sha1.len(), 40);

    let sha256 = validate_iso_hash(&test_file, HashType::Sha256).await.unwrap();
    assert_eq!(sha256.len(), 64);
}
```

## 3. System Tests

### 3.1 Device Enumeration Test
```
Test: USB Device Enumeration
Preconditions: Multiple USB devices connected
Steps:
  1. Launch Sidozdev
  2. Click "Refresh Devices"
Expected: All USB devices displayed with correct info
  - Name, capacity, interface type, device ID
  - No system drives included
  - Removable devices only
```

### 3.2 ISO Selection Test
```
Test: ISO File Selection
Preconditions: Valid ISO file available
Steps:
  1. Click "Browse..."
  2. Select ISO file
  3. Verify ISO info displayed
Expected:
  - Label shown
  - Size correct
  - Type detected (ISO9660/Hybrid/IMG)
  - Bootable status shown
  - Boot modes listed
```

### 3.3 Write Operation Test
```
Test: Full Write Operation
Preconditions: USB device and ISO selected
Steps:
  1. Select USB device
  2. Select ISO file
  3. Configure options (UEFI, GPT, FAT32)
  4. Click Start
  5. Monitor progress
  6. Wait for completion
Expected:
  - Progress bar advances smoothly
  - Speed displayed (MB/s)
  - ETA updates
  - Completion message shown
  - Device bootable after operation
```

### 3.4 Cancellation Test
```
Test: Operation Cancellation
Preconditions: Write operation in progress
Steps:
  1. Start write operation
  2. Wait for 10-20% progress
  3. Click Cancel
Expected:
  - Operation stops within 5 seconds
  - "Cancelled" status shown
  - No data corruption on device
  - UI returns to ready state
```

### 3.5 Verification Test
```
Test: Write Verification
Preconditions: Write completed with verification enabled
Steps:
  1. Enable "Verify After Write"
  2. Complete write operation
  3. Verify phase completes
Expected:
  - Verification progress shown
  - "Verification passed" message
  - No errors reported
```

### 3.6 Hash Verification Test
```
Test: ISO Hash Calculation
Preconditions: ISO file selected
Steps:
  1. Click MD5 button
  2. Wait for calculation
  3. Click SHA1 button
  4. Click SHA256 button
Expected:
  - Hash values displayed
  - Correct hash lengths
  - Consistent results on recalculation
```

## 4. Performance Tests

### 4.1 Write Speed Benchmark
```
Test: Write Speed Measurement
Devices: USB 2.0, USB 3.0, USB 3.1, USB 3.2
ISO Sizes: 1GB, 4GB, 8GB, 16GB

Expected Minimum Speeds:
  - USB 2.0: 8-15 MB/s
  - USB 3.0: 25-40 MB/s
  - USB 3.1: 40-80 MB/s
  - USB 3.2: 80-150 MB/s
```

### 4.2 Memory Usage Test
```
Test: Memory Usage Monitoring
Steps:
  1. Monitor memory during idle
  2. Monitor memory during device scan
  3. Monitor memory during write
  4. Monitor memory during verification

Expected:
  - Idle: <50 MB
  - Scan: <60 MB
  - Write: <100 MB (with 4MB buffer)
  - Verify: <100 MB
  - No memory leaks (stable after 10 operations)
```

### 4.3 Startup Time Test
```
Test: Application Startup
Steps:
  1. Cold start (first launch)
  2. Warm start (subsequent)

Expected:
  - Cold start: <3 seconds
  - Warm start: <1 second
```

## 5. Compatibility Tests

### 5.1 Windows Version Tests
```
Test Matrix:
  - Windows 10 21H2 (Build 19044)
  - Windows 10 22H2 (Build 19045)
  - Windows 11 21H2 (Build 22000)
  - Windows 11 22H2 (Build 22621)
  - Windows 11 23H2 (Build 22631)

Tests:
  - Installation
  - Device enumeration
  - Write operation
  - UEFI boot
  - Secure Boot
```

### 5.2 USB Device Tests
```
Test Matrix:
  Brands: SanDisk, Kingston, Samsung, Transcend, PNY, HP
  Sizes: 4GB, 8GB, 16GB, 32GB, 64GB, 128GB
  Types: USB 2.0, USB 3.0, USB 3.1, USB-C
  Formats: Pre-formatted FAT32, NTFS, exFAT, raw

Tests:
  - Detection
  - Formatting
  - Write operation
  - Boot verification
```

### 5.3 ISO Compatibility Tests
```
Test ISOs:
  - Windows 10/11 (official Microsoft)
  - Ubuntu 22.04/24.04
  - Fedora 39/40
  - Debian 12
  - Arch Linux
  - Kali Linux
  - Hiren's BootCD
  - GParted Live
  - Clonezilla
  - MemTest86
  - Various custom ISOs

Tests:
  - Detection
  - Parsing
  - Write operation
  - Boot verification (VM test)
```

## 6. Security Tests

### 6.1 Privilege Escalation Test
```
Test: Admin Privilege Check
Steps:
  1. Run without admin
  2. Attempt device write
Expected:
  - Clear error message
  - UAC prompt option
  - Graceful failure
```

### 6.2 System Drive Protection Test
```
Test: System Drive Protection
Steps:
  1. Verify system drives not listed
  2. Attempt to access system drive paths
Expected:
  - No system drives in device list
  - Error on direct system drive access
```

### 6.3 Malformed ISO Test
```
Test: Malformed ISO Handling
Steps:
  1. Test with corrupted ISO
  2. Test with non-ISO file renamed as .iso
  3. Test with truncated ISO
Expected:
  - Graceful error handling
  - No crashes
  - Informative error messages
```

## 7. UI/UX Tests

### 7.1 Theme Switching Test
```
Test: Light/Dark/System Theme
Steps:
  1. Switch to Light theme
  2. Switch to Dark theme
  3. Switch to System theme
Expected:
  - All UI elements update correctly
  - Colors consistent
  - No visual glitches
```

### 7.2 Language Switching Test
```
Test: Arabic/English Language
Steps:
  1. Switch to Arabic
  2. Verify all text translated
  3. Switch to English
  4. Verify all text in English
Expected:
  - Complete translation
  - RTL layout for Arabic
  - No untranslated strings
```

### 7.3 Log Export Test
```
Test: Log Export Functionality
Steps:
  1. Perform operations to generate logs
  2. Click Save Logs
  3. Choose save location
  4. Verify file contents
Expected:
  - File created successfully
  - All logs included
  - Correct format (timestamp, level, message)
```

## 8. Regression Tests

### 8.1 Full Workflow Regression
```
Test: Complete End-to-End Workflow
Steps:
  1. Fresh launch
  2. Refresh devices
  3. Select device
  4. Select ISO
  5. Configure options
  6. Start write
  7. Monitor progress
  8. Verify completion
  9. Check logs
  10. Export logs
  11. Change settings
  12. Restart application
  13. Verify settings persisted
Expected:
  - All steps complete without errors
  - Settings persist across sessions
```

## 9. Test Automation

### CI/CD Pipeline Tests
```yaml
# GitHub Actions workflow
name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all
      - run: cargo test --all --release
      - run: cargo clippy --all-targets --all-features
      - run: cargo fmt --check
```

### Test Coverage Targets
- Unit tests: >80% coverage
- Integration tests: >60% coverage
- Critical paths: 100% coverage

## 10. Manual Test Checklist

### Pre-Release Checklist
- [ ] Fresh install on clean Windows 10
- [ ] Fresh install on clean Windows 11
- [ ] Write Windows 10 ISO to USB 3.0 drive
- [ ] Write Ubuntu ISO to USB 2.0 drive
- [ ] Verify UEFI boot on real hardware
- [ ] Verify BIOS boot on real hardware
- [ ] Test cancellation at 25%, 50%, 75%
- [ ] Test with full verification enabled
- [ ] Test hash calculation (MD5, SHA1, SHA256)
- [ ] Test theme switching during operation
- [ ] Test language switching during operation
- [ ] Test log export
- [ ] Test settings persistence
- [ ] Test error recovery (pull USB during write)
- [ ] Memory leak check (24-hour stress test)
- [ ] Binary size check (<20MB)
- [ ] Virus scan (false positive check)
- [ ] Code signing verification
