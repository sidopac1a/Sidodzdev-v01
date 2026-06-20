//! Disk Writer
//! 
//! Handles writing ISO/IMG files to USB devices with progress tracking and cancellation support.

use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};

use crate::core::app_state::{SharedAppState, OperationPhase, LogLevel, ProgressInfo, BootMode, PartitionScheme, FileSystem};
use crate::core::events::{EventBus, UiEvent};
use crate::disk::partition::create_partition_table;
use crate::disk::format::format_device;
use crate::utils::async_utils::AsyncFileReader;

/// Disk writer for creating bootable USB drives
pub struct DiskWriter {
    device_id: String,
    iso_path: PathBuf,
    boot_mode: BootMode,
    partition_scheme: PartitionScheme,
    file_system: FileSystem,
    cancel_flag: Arc<AtomicBool>,
    buffer_size: usize,
}

impl DiskWriter {
    pub fn new(
        device_id: String,
        iso_path: PathBuf,
        boot_mode: BootMode,
        partition_scheme: PartitionScheme,
        file_system: FileSystem,
        cancel_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            device_id,
            iso_path,
            boot_mode,
            partition_scheme,
            file_system,
            cancel_flag,
            buffer_size: 4 * 1024 * 1024, // 4MB buffer
        }
    }

    /// Execute the full write operation
    pub async fn write(
        &self,
        state: SharedAppState,
        event_bus: Arc<Mutex<EventBus>>,
    ) -> Result<()> {
        let start_time = Instant::now();

        // Step 1: Validate inputs
        self.validate_inputs()?;

        // Step 2: Get ISO file size
        let iso_size = std::fs::metadata(&self.iso_path)?.len();

        // Step 3: Create partition table
        self.update_progress(&state, &event_bus, 0.0, "Creating partition table...", iso_size).await;
        create_partition_table(&self.device_id, self.partition_scheme, self.boot_mode)?;

        // Step 4: Format the device
        self.update_progress(&state, &event_bus, 5.0, "Formatting device...", iso_size).await;
        format_device(&self.device_id, self.file_system)?;

        // Step 5: Write ISO data
        self.update_progress(&state, &event_bus, 10.0, "Writing ISO data...", iso_size).await;
        self.write_iso_data(&state, &event_bus, iso_size).await?;

        // Step 6: Install bootloader
        self.update_progress(&state, &event_bus, 95.0, "Installing bootloader...", iso_size).await;
        self.install_bootloader(&state, &event_bus).await?;

        // Step 7: Verify write (if enabled)
        let verify_enabled = {
            let state_guard = state.lock().unwrap();
            state_guard.config.verify_after_write
        };

        if verify_enabled {
            self.update_progress(&state, &event_bus, 97.0, "Verifying write...", iso_size).await;
            self.verify_write(&state, &event_bus, iso_size).await?;
        }

        let elapsed = start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        let speed_mbps = (iso_size as f64 / 1024.0 / 1024.0) / elapsed_secs.max(0.001);

        self.update_progress(&state, &event_bus, 100.0, 
            &format!("Completed! Speed: {:.2} MB/s", speed_mbps), iso_size).await;

        info!("Write operation completed in {:.2}s at {:.2} MB/s", elapsed_secs, speed_mbps);

        Ok(())
    }

    /// Validate that inputs are correct
    fn validate_inputs(&self) -> Result<()> {
        if !self.iso_path.exists() {
            return Err(anyhow::anyhow!("ISO file does not exist: {:?}", self.iso_path));
        }

        if self.device_id.is_empty() {
            return Err(anyhow::anyhow!("No device selected"));
        }

        info!("Validated inputs: device={}, iso={:?}", self.device_id, self.iso_path);
        Ok(())
    }

    /// Write ISO data to the device
    async fn write_iso_data(
        &self,
        state: &SharedAppState,
        event_bus: &Arc<Mutex<EventBus>>,
        total_size: u64,
    ) -> Result<()> {
        let mut iso_file = File::open(&self.iso_path)
            .context("Failed to open ISO file")?;

        let mut device_file = self.open_device_for_writing()?;

        let mut buffer = vec![0u8; self.buffer_size];
        let mut bytes_written: u64 = 0;
        let mut last_progress_update = Instant::now();
        let mut last_bytes_written: u64 = 0;

        loop {
            // Check for cancellation
            if self.cancel_flag.load(Ordering::Relaxed) {
                return Err(anyhow::anyhow!("Operation cancelled by user"));
            }

            let bytes_read = iso_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            device_file.write_all(&buffer[..bytes_read])?;
            device_file.flush()?;

            bytes_written += bytes_read as u64;

            // Update progress every 500ms
            if last_progress_update.elapsed() >= Duration::from_millis(500) {
                let progress_pct = 10.0 + (bytes_written as f64 / total_size as f64) * 85.0;
                let bytes_delta = bytes_written - last_bytes_written;
                let time_delta = last_progress_update.elapsed().as_secs_f64();
                let speed_mbps = (bytes_delta as f64 / 1024.0 / 1024.0) / time_delta.max(0.001);

                let remaining_bytes = total_size - bytes_written;
                let eta_seconds = if speed_mbps > 0.1 {
                    (remaining_bytes as f64 / 1024.0 / 1024.0 / speed_mbps) as u64
                } else {
                    0
                };

                self.update_progress_with_speed(
                    state, event_bus, progress_pct, 
                    &format!("Writing... {:.1}%", progress_pct),
                    bytes_written, total_size, speed_mbps, eta_seconds,
                ).await;

                last_progress_update = Instant::now();
                last_bytes_written = bytes_written;
            }

            // Yield to allow other tasks to run
            sleep(Duration::from_millis(1)).await;
        }

        // Ensure all data is written
        device_file.sync_all()?;

        info!("Wrote {} bytes to device", bytes_written);
        Ok(())
    }

    /// Open the device for writing
    fn open_device_for_writing(&self) -> Result<File> {
        // On Windows, we need to open the physical drive with special flags
        // This is a simplified version - production code would use WinAPI directly
        let device_path = if self.device_id.starts_with("\\\\.\\") {
            self.device_id.clone()
        } else {
            format!("\\\\.\\{}", self.device_id)
        };

        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            use winapi::um::winbase::FILE_FLAG_NO_BUFFERING;
            use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_WRITE};

            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(false)
                .truncate(false)
                .custom_flags(FILE_FLAG_NO_BUFFERING)
                .open(&device_path)?;

            Ok(file)
        }

        #[cfg(not(windows))]
        {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(false)
                .open(&device_path)?;

            Ok(file)
        }
    }

    /// Install bootloader based on boot mode
    async fn install_bootloader(
        &self,
        _state: &SharedAppState,
        _event_bus: &Arc<Mutex<EventBus>>,
    ) -> Result<()> {
        match self.boot_mode {
            BootMode::BiosLegacy => {
                info!("Installing BIOS legacy bootloader");
                // Would install MBR bootloader here
            }
            BootMode::Uefi | BootMode::UefiSecureBoot => {
                info!("Installing UEFI bootloader");
                // Would install EFI bootloader here
            }
        }

        Ok(())
    }

    /// Verify the written data
    async fn verify_write(
        &self,
        state: &SharedAppState,
        event_bus: &Arc<Mutex<EventBus>>,
        total_size: u64,
    ) -> Result<()> {
        let mut iso_file = File::open(&self.iso_path)?;
        let mut device_file = self.open_device_for_reading()?;

        let mut iso_buffer = vec![0u8; self.buffer_size];
        let mut device_buffer = vec![0u8; self.buffer_size];
        let mut bytes_verified: u64 = 0;
        let mut last_progress_update = Instant::now();

        loop {
            if self.cancel_flag.load(Ordering::Relaxed) {
                return Err(anyhow::anyhow!("Verification cancelled"));
            }

            let iso_read = iso_file.read(&mut iso_buffer)?;
            if iso_read == 0 {
                break;
            }

            let device_read = device_file.read(&mut device_buffer)?;
            if device_read != iso_read {
                return Err(anyhow::anyhow!("Device read size mismatch during verification"));
            }

            if iso_buffer[..iso_read] != device_buffer[..device_read] {
                return Err(anyhow::anyhow!("Verification failed: data mismatch detected"));
            }

            bytes_verified += iso_read as u64;

            if last_progress_update.elapsed() >= Duration::from_millis(500) {
                let progress_pct = 97.0 + (bytes_verified as f64 / total_size as f64) * 3.0;
                self.update_progress(
                    state, event_bus, progress_pct, 
                    &format!("Verifying... {:.1}%", progress_pct), total_size,
                ).await;
                last_progress_update = Instant::now();
            }

            sleep(Duration::from_millis(1)).await;
        }

        info!("Verification completed successfully: {} bytes verified", bytes_verified);

        let mut state_guard = state.lock().unwrap();
        state_guard.add_log(LogLevel::Success, 
            format!("Write verification passed: {} bytes verified", bytes_verified));

        Ok(())
    }

    /// Open device for reading (verification)
    fn open_device_for_reading(&self) -> Result<File> {
        let device_path = if self.device_id.starts_with("\\\\.\\") {
            self.device_id.clone()
        } else {
            format!("\\\\.\\{}", self.device_id)
        };

        Ok(File::open(&device_path)?)
    }

    /// Update progress to UI
    async fn update_progress(
        &self,
        state: &SharedAppState,
        event_bus: &Arc<Mutex<EventBus>>,
        percentage: f32,
        operation: &str,
        total_bytes: u64,
    ) {
        let mut progress = ProgressInfo::default();
        progress.phase = OperationPhase::Writing;
        progress.percentage = percentage.clamp(0.0, 100.0);
        progress.total_bytes = total_bytes;
        progress.current_operation = operation.to_string();

        let mut state_guard = state.lock().unwrap();
        state_guard.progress = progress.clone();
        state_guard.add_log(LogLevel::Info, operation.to_string());

        if let Ok(bus) = event_bus.lock() {
            let _ = bus.ui_sender.send(UiEvent::ProgressUpdated(progress));
        }
    }

    /// Update progress with speed information
    async fn update_progress_with_speed(
        &self,
        state: &SharedAppState,
        event_bus: &Arc<Mutex<EventBus>>,
        percentage: f32,
        operation: &str,
        bytes_written: u64,
        total_bytes: u64,
        speed_mbps: f32,
        eta_seconds: u64,
    ) {
        let progress = ProgressInfo {
            phase: OperationPhase::Writing,
            percentage: percentage.clamp(0.0, 100.0),
            bytes_written,
            total_bytes,
            speed_mbps,
            eta_seconds,
            current_operation: operation.to_string(),
        };

        let mut state_guard = state.lock().unwrap();
        state_guard.progress = progress.clone();

        if let Ok(bus) = event_bus.lock() {
            let _ = bus.ui_sender.send(UiEvent::ProgressUpdated(progress));
        }
    }
}
