//! Event System
//! 
//! Thread-safe event communication between backend and UI.

use std::sync::mpsc::{channel, Sender, Receiver};
use crate::core::app_state::{ProgressInfo, LogLevel, LogEntry};
use crate::disk::device::UsbDevice;
use crate::iso::parser::IsoInfo;

/// Events sent from backend to UI
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Device list updated
    DevicesUpdated(Vec<UsbDevice>),
    /// ISO validation completed
    IsoValidated(IsoInfo),
    /// Progress update
    ProgressUpdated(ProgressInfo),
    /// Log message
    LogMessage(LogEntry),
    /// Operation completed
    OperationCompleted,
    /// Operation cancelled
    OperationCancelled,
    /// Error occurred
    Error(String),
    /// Device removed
    DeviceRemoved(String),
    /// Device added
    DeviceAdded(UsbDevice),
}

/// Commands sent from UI to backend
#[derive(Debug, Clone)]
pub enum BackendCommand {
    /// Refresh device list
    RefreshDevices,
    /// Select ISO file
    SelectIso(String),
    /// Start writing process
    StartWrite,
    /// Cancel current operation
    CancelOperation,
    /// Verify ISO hash
    VerifyIso(HashType),
    /// Format device
    FormatDevice,
}

/// Hash type for verification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    Md5,
    Sha1,
    Sha256,
}

/// Event bus for UI-backend communication
pub struct EventBus {
    pub ui_sender: Sender<UiEvent>,
    pub ui_receiver: Receiver<UiEvent>,
    pub backend_sender: Sender<BackendCommand>,
    pub backend_receiver: Receiver<BackendCommand>,
}

impl EventBus {
    pub fn new() -> Self {
        let (ui_sender, ui_receiver) = channel();
        let (backend_sender, backend_receiver) = channel();

        Self {
            ui_sender,
            ui_receiver,
            backend_sender,
            backend_receiver,
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
