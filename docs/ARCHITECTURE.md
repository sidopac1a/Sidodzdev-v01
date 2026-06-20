# Sidozdev Architecture Diagram

## System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              USER INTERFACE                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐  │
│  │  Device List    │  │  ISO Selector   │  │  Options Panel          │  │
│  │  Component      │  │  Component      │  │  Component              │  │
│  │                 │  │                 │  │                         │  │
│  │ • USB Devices   │  │ • File Browser  │  │ • Boot Mode             │  │
│  │ • Device Info   │  │ • ISO Info      │  │ • Partition Scheme      │  │
│  │ • Selection     │  │ • Validation    │  │ • File System           │  │
│  └────────┬────────┘  └────────┬────────┘  └───────────┬─────────────┘  │
│           │                    │                       │                  │
│  ┌────────┴────────────────────┴───────────────────────┴─────────────┐    │
│  │                    Progress Panel Component                       │    │
│  │  • Real-time Progress Bar  • Speed & ETA  • Start/Cancel         │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│           │                                                             │
│  ┌────────┴─────────────────────────────────────────────────────────┐    │
│  │                    Log Panel Component                            │    │
│  │  • Operation Logs  • Filtering  • Export  • Auto-scroll          │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Settings & About Dialogs                       │    │
│  │  • Language (AR/EN)  • Theme (Light/Dark/System)  • About       │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │
                         ┌─────────┴──────────┐
                         │   Event Bus (MPSC)  │
                         │  UI <-> Backend     │
                         │  Thread-safe comms  │
                         └─────────┬──────────┘
                                   │
┌──────────────────────────────────┴──────────────────────────────────────┐
│                            CORE ENGINE                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐  │
│  │  App State      │  │  Configuration  │  │  Worker Thread          │  │
│  │  Manager        │  │  Manager        │  │  (Async Runtime)        │  │
│  │                 │  │                 │  │                         │  │
│  │ • Shared State  │  │ • Persistent    │  │ • Device Scanning       │  │
│  │ • Thread-safe   │  │   Settings      │  │ • ISO Validation        │  │
│  │ • Progress      │  │ • JSON Storage  │  │ • Write Operations      │  │
│  │ • Log Buffer    │  │ • Auto-save     │  │ • Cancellation          │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────┘  │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │
                    ┌──────────────┼──────────────┐
                    │              │              │
┌───────────────────┴──┐  ┌───────┴──────┐  ┌────┴────────────────────┐
│     DISK MODULE      │  │  ISO MODULE  │  │    UTILITIES MODULE     │
│                      │  │              │  │                         │
│  ┌────────────────┐  │  │  ┌────────┐  │  │  ┌──────────────────┐  │
│  │ USB Enumerator │  │  │  │ Parser │  │  │  │ Hash Calculator  │  │
│  │ (SetupAPI)     │  │  │  │(ISO9660│  │  │  │ (MD5/SHA1/SHA256)│  │
│  │                │  │  │  │Hybrid  │  │  │  │                  │  │
│  │ • Device Info  │  │  │  │IMG/UDF)│  │  │  │ • Async I/O      │  │
│  │ • Capacity     │  │  │  │        │  │  │  │ • Buffer Mgmt    │  │
│  │ • Interface    │  │  │  │• Boot  │  │  │  │ • Progress       │  │
│  │ • VID/PID      │  │  │  │ Record │  │  │  │   Tracking       │  │
│  └────────────────┘  │  │  │• Hybrid│  │  │  └──────────────────┘  │
│  ┌────────────────┐  │  │  │ Detect │  │  │  ┌──────────────────┐  │
│  │ Disk Writer    │  │  │  └────────┘  │  │  │ Logging System   │  │
│  │ (WinAPI)       │  │  │              │  │  │                  │  │
│  │                │  │  │  ┌────────┐  │  │  │ • File Logging   │  │
│  │ • Raw Write    │  │  │  │Boot    │  │  │  │ • Log Levels     │  │
│  │ • Buffer Mgmt  │  │  │  │Record  │  │  │  │ • Export         │  │
│  │ • Cancellation │  │  │  │Analysis│  │  │  └──────────────────┘  │
│  │ • Progress     │  │  │  │        │  │  │  ┌──────────────────┐  │
│  └────────────────┘  │  │  │• El    │  │  │  │ Platform Utils   │  │
│  ┌────────────────┐  │  │  │ Torito │  │  │  │                  │  │
│  │ Write Verifier │  │  │  │• UEFI  │  │  │  │ • Error Codes    │  │
│  │                │  │  │  │• BIOS  │  │  │  │ • Elevation      │  │
│  │ • Full Verify  │  │  │  │• GRUB  │  │  │  │ • Windows Ver    │  │
│  │ • Hash Compare │  │  │  │• Sys-  │  │  │  └──────────────────┘  │
│  │ • Sample Check │  │  │  │ linux  │  │  │                        │
│  └────────────────┘  │  │  └────────┘  │  │                        │
│  ┌────────────────┐  │  │              │  │                        │
│  │ Partition Mgr  │  │  │  ┌────────┐  │  │                        │
│  │                │  │  │  │Hash    │  │  │                        │
│  │ • MBR Creation │  │  │  │Validator│  │  │                        │
│  │ • GPT Creation │  │  │  │        │  │  │                        │
│  │ • Boot Code    │  │  │  │• MD5   │  │  │                        │
│  └────────────────┘  │  │  │• SHA1  │  │  │                        │
│  ┌────────────────┐  │  │  │• SHA256│  │  │                        │
│  │ Format Engine  │  │  │  └────────┘  │  │                        │
│  │                │  │  │              │  │                        │
│  │ • FAT32 Native │  │  │              │  │                        │
│  │ • NTFS (cmd)   │  │  │              │  │                        │
│  │ • exFAT (cmd)  │  │  │              │  │                        │
│  └────────────────┘  │  │              │  │                        │
│                      │  │              │  │                        │
└──────────────────────┘  └──────────────┘  └────────────────────────┘
```

## Data Flow

```
User Action → UI Component → Event Bus → Worker Thread → Disk/ISO Module → Hardware
                                              ↑
                                              │ (Progress Updates)
                                              ↓
UI Component ← Event Bus ← Worker Thread ← Disk/ISO Module
```

## Module Responsibilities

### UI Module (egui)
- **DeviceList**: Renders USB device list, handles selection
- **IsoSelector**: File dialog, ISO info display, hash buttons
- **OptionsPanel**: Boot mode, partition, file system configuration
- **ProgressPanel**: Real-time progress, speed, ETA, start/cancel
- **LogPanel**: Scrollable log view with filtering and export
- **App**: Main controller, event processing, dialog management

### Core Module
- **AppState**: Central state with Mutex, all UI data
- **Config**: Persistent JSON settings (language, theme, defaults)
- **Events**: MPSC channels for UI-backend communication
- **Worker**: Tokio async runtime, processes commands, sends events

### Disk Module
- **Enumerator**: Windows SetupAPI for USB device enumeration
- **Device**: Data structure with all device properties
- **Writer**: Raw disk write with progress, cancellation, verification
- **Verifier**: Post-write verification (full/hash/sampled)
- **Partition**: MBR/GPT creation with boot code
- **Format**: FAT32 native, NTFS/exFAT via Windows format command

### ISO Module
- **Parser**: ISO9660, Hybrid ISO, IMG, UDF detection and parsing
- **Validator**: MD5/SHA1/SHA256 hash calculation
- **Hybrid**: Hybrid ISO detection (ISO+USB bootable)
- **Boot**: Boot record analysis (El Torito, UEFI, BIOS, GRUB, SYSLINUX)

### Utils Module
- **Hash**: Synchronous hash calculation for verification
- **Logging**: File-based logging with rotation
- **Platform**: Windows error codes, elevation, version detection
- **AsyncUtils**: Async file I/O helpers, blocking operations

## Threading Model

```
Main Thread (UI)
    └── egui event loop (60 FPS)
        └── process_events() ← MPSC receiver
        └── render UI components

Worker Thread
    └── tokio runtime (multi-thread)
        └── worker_loop() ← MPSC receiver
            ├── enumerate_usb_devices() [blocking]
            ├── parse_iso() [async]
            ├── write_disk() [async with cancellation]
            └── verify_write() [async]

Event Bus (MPSC Channels)
    ├── ui_sender → ui_receiver (backend → UI)
    └── backend_sender → backend_receiver (UI → backend)
```

## Memory Safety

- **No unsafe** except Windows API FFI calls (minimal, isolated)
- **Arc<Mutex<AppState>>** for shared state
- **MPSC channels** for thread communication (no shared mutable state)
- **Tokio** for async I/O with proper cancellation
- **BufReader/BufWriter** for buffered file operations
- **4MB write buffers** for optimal USB performance

## Error Handling Strategy

- **thiserror/anyhow** for structured errors
- **Result propagation** through all layers
- **Graceful degradation** (e.g., fallback to format command)
- **User-friendly error messages** in UI
- **Detailed error logging** for debugging
