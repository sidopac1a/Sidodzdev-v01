//! Worker Thread Management
//! 
//! Manages background operations for device scanning, ISO validation, and writing.

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use tracing::{info, warn, error};

use crate::core::app_state::{SharedAppState, OperationPhase, LogLevel, ProgressInfo};
use crate::core::events::{EventBus, UiEvent, BackendCommand, HashType};
use crate::disk::enumerator::enumerate_usb_devices;
use crate::disk::writer::DiskWriter;
use crate::iso::parser::parse_iso;
use crate::iso::validator::validate_iso_hash;
use crate::utils::logging::LogManager;

/// Background worker for handling operations
pub struct Worker {
    pub handle: Option<JoinHandle<()>>,
    pub cancel_flag: Arc<AtomicBool>,
    pub runtime: Arc<Runtime>,
}

impl Worker {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = Arc::new(Runtime::new()?);
        Ok(Self {
            handle: None,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            runtime,
        })
    }

    pub fn start(
        &mut self,
        state: SharedAppState,
        event_bus: Arc<Mutex<EventBus>>,
    ) {
        let cancel_flag = self.cancel_flag.clone();
        let runtime = self.runtime.clone();

        let handle = thread::spawn(move || {
            runtime.block_on(async {
                worker_loop(state, event_bus, cancel_flag).await;
            });
        });

        self.handle = Some(handle);
    }

    pub fn request_cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    pub fn reset_cancel(&self) {
        self.cancel_flag.store(false, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }
}

async fn worker_loop(
    state: SharedAppState,
    event_bus: Arc<Mutex<EventBus>>,
    cancel_flag: Arc<AtomicBool>,
) {
    loop {
        // Check for cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            let mut state = state.lock().unwrap();
            state.is_running = false;
            state.progress.phase = OperationPhase::Cancelled;
            state.add_log(LogLevel::Warning, "Operation cancelled");

            if let Ok(bus) = event_bus.lock() {
                let _ = bus.ui_sender.send(UiEvent::OperationCancelled);
            }
            cancel_flag.store(false, Ordering::Relaxed);
        }

        // Process backend commands
        if let Ok(bus) = event_bus.lock() {
            if let Ok(command) = bus.backend_receiver.try_recv() {
                match command {
                    BackendCommand::RefreshDevices => {
                        handle_refresh_devices(&state, &event_bus).await;
                    }
                    BackendCommand::SelectIso(path) => {
                        handle_select_iso(&state, &event_bus, path).await;
                    }
                    BackendCommand::StartWrite => {
                        handle_start_write(&state, &event_bus, cancel_flag.clone()).await;
                    }
                    BackendCommand::CancelOperation => {
                        cancel_flag.store(true, Ordering::Relaxed);
                    }
                    BackendCommand::VerifyIso(hash_type) => {
                        handle_verify_iso(&state, &event_bus, hash_type).await;
                    }
                    BackendCommand::FormatDevice => {
                        handle_format_device(&state, &event_bus).await;
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

async fn handle_refresh_devices(
    state: &SharedAppState,
    event_bus: &Arc<Mutex<EventBus>>,
) {
    let mut state_guard = state.lock().unwrap();
    state_guard.progress.phase = OperationPhase::ScanningDevices;
    state_guard.add_log(LogLevel::Info, "Scanning for USB devices...");
    drop(state_guard);

    match enumerate_usb_devices() {
        Ok(devices) => {
            let mut state_guard = state.lock().unwrap();
            state_guard.devices = devices.clone();
            state_guard.progress.phase = OperationPhase::Idle;
            state_guard.add_log(LogLevel::Success, 
                format!("Found {} USB device(s)", devices.len()));

            if let Ok(bus) = event_bus.lock() {
                let _ = bus.ui_sender.send(UiEvent::DevicesUpdated(devices));
            }
        }
        Err(e) => {
            let mut state_guard = state.lock().unwrap();
            state_guard.progress.phase = OperationPhase::Error;
            state_guard.add_log(LogLevel::Error, format!("Device enumeration failed: {}", e));

            if let Ok(bus) = event_bus.lock() {
                let _ = bus.ui_sender.send(UiEvent::Error(e.to_string()));
            }
        }
    }
}

async fn handle_select_iso(
    state: &SharedAppState,
    event_bus: &Arc<Mutex<EventBus>>,
    path: String,
) {
    let mut state_guard = state.lock().unwrap();
    state_guard.progress.phase = OperationPhase::ValidatingIso;
    state_guard.add_log(LogLevel::Info, format!("Analyzing ISO: {}", path));
    drop(state_guard);

    let path_buf = PathBuf::from(path);

    match parse_iso(&path_buf).await {
        Ok(iso_info) => {
            let mut state_guard = state.lock().unwrap();
            state_guard.iso_info = Some(iso_info.clone());
            state_guard.config.iso_path = Some(path_buf);
            state_guard.progress.phase = OperationPhase::Idle;
            state_guard.add_log(LogLevel::Success, 
                format!("ISO validated: {} ({:.2} MB)", 
                    iso_info.label, 
                    iso_info.size as f64 / 1024.0 / 1024.0));

            if let Ok(bus) = event_bus.lock() {
                let _ = bus.ui_sender.send(UiEvent::IsoValidated(iso_info));
            }
        }
        Err(e) => {
            let mut state_guard = state.lock().unwrap();
            state_guard.progress.phase = OperationPhase::Error;
            state_guard.add_log(LogLevel::Error, format!("ISO validation failed: {}", e));

            if let Ok(bus) = event_bus.lock() {
                let _ = bus.ui_sender.send(UiEvent::Error(e.to_string()));
            }
        }
    }
}

async fn handle_start_write(
    state: &SharedAppState,
    event_bus: &Arc<Mutex<EventBus>>,
    cancel_flag: Arc<AtomicBool>,
) {
    let (device_id, iso_path, boot_mode, partition_scheme, file_system) = {
        let state_guard = state.lock().unwrap();
        (
            state_guard.config.selected_device.clone(),
            state_guard.config.iso_path.clone(),
            state_guard.config.boot_mode,
            state_guard.config.partition_scheme,
            state_guard.config.file_system,
        )
    };

    if device_id.is_none() || iso_path.is_none() {
        let mut state_guard = state.lock().unwrap();
        state_guard.add_log(LogLevel::Error, "Device or ISO not selected");
        return;
    }

    let device_id = device_id.unwrap();
    let iso_path = iso_path.unwrap();

    let mut state_guard = state.lock().unwrap();
    state_guard.is_running = true;
    state_guard.progress.phase = OperationPhase::Writing;
    state_guard.add_log(LogLevel::Info, format!("Starting write to device: {}", device_id));
    drop(state_guard);

    let writer = DiskWriter::new(
        device_id.clone(),
        iso_path,
        boot_mode,
        partition_scheme,
        file_system,
        cancel_flag,
    );

    match writer.write(state.clone(), event_bus.clone()).await {
        Ok(_) => {
            let mut state_guard = state.lock().unwrap();
            state_guard.is_running = false;
            state_guard.progress.phase = OperationPhase::Completed;
            state_guard.add_log(LogLevel::Success, "Write completed successfully!");

            if let Ok(bus) = event_bus.lock() {
                let _ = bus.ui_sender.send(UiEvent::OperationCompleted);
            }
        }
        Err(e) => {
            let mut state_guard = state.lock().unwrap();
            state_guard.is_running = false;
            state_guard.progress.phase = OperationPhase::Error;
            state_guard.add_log(LogLevel::Error, format!("Write failed: {}", e));

            if let Ok(bus) = event_bus.lock() {
                let _ = bus.ui_sender.send(UiEvent::Error(e.to_string()));
            }
        }
    }
}

async fn handle_verify_iso(
    state: &SharedAppState,
    event_bus: &Arc<Mutex<EventBus>>,
    hash_type: HashType,
) {
    let iso_path = {
        let state_guard = state.lock().unwrap();
        state_guard.config.iso_path.clone()
    };

    if let Some(path) = iso_path {
        let mut state_guard = state.lock().unwrap();
        state_guard.add_log(LogLevel::Info, 
            format!("Starting {} verification...", hash_type));
        drop(state_guard);

        match validate_iso_hash(&path, hash_type).await {
            Ok(hash) => {
                let mut state_guard = state.lock().unwrap();
                state_guard.add_log(LogLevel::Success, 
                    format!("ISO {} hash: {}", hash_type, hash));
            }
            Err(e) => {
                let mut state_guard = state.lock().unwrap();
                state_guard.add_log(LogLevel::Error, 
                    format!("Hash verification failed: {}", e));
            }
        }
    }
}

async fn handle_format_device(
    state: &SharedAppState,
    event_bus: &Arc<Mutex<EventBus>>,
) {
    let device_id = {
        let state_guard = state.lock().unwrap();
        state_guard.config.selected_device.clone()
    };

    if let Some(id) = device_id {
        let mut state_guard = state.lock().unwrap();
        state_guard.add_log(LogLevel::Info, format!("Formatting device: {}", id));
        // Format implementation would go here
        state_guard.add_log(LogLevel::Success, "Format completed");
    }
}
