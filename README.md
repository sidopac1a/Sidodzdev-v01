# Sidozdev - USB Bootable Media Creator

A professional, open-source USB bootable media creation tool built with Rust. Sidozdev is a modern alternative to Rufus, designed for creating bootable USB drives from ISO images with full support for Windows 10 and Windows 11.

## Features

- **USB Device Detection**: Automatic enumeration of all connected USB storage devices
- **Multi-Format Support**: ISO9660, Hybrid ISO, and IMG files
- **Boot Modes**: BIOS Legacy, UEFI, and UEFI + Secure Boot
- **Partition Schemes**: MBR and GPT
- **File Systems**: FAT32, NTFS, exFAT
- **Hash Verification**: MD5, SHA1, SHA256 for ISO integrity checking
- **Real-time Progress**: Live progress bar with speed and ETA
- **Detailed Logging**: Comprehensive operation logs with export capability
- **Safe Cancellation**: Graceful operation cancellation
- **Write Verification**: Post-write data integrity verification
- **Dark/Light Themes**: Full theme support
- **Multi-language**: Arabic and English support

## Architecture

Sidozdev follows a clean modular architecture with complete separation between backend and UI:

```
sidozdev/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   ├── core/            # Core application logic
│   │   ├── app_state.rs # State management
│   │   ├── config.rs    # Persistent settings
│   │   ├── events.rs    # UI-backend communication
│   │   └── worker.rs    # Background operations
│   ├── disk/            # Disk operations
│   │   ├── enumerator.rs # USB device enumeration
│   │   ├── device.rs    # Device representation
│   │   ├── writer.rs    # Disk writing engine
│   │   ├── verifier.rs  # Write verification
│   │   ├── partition.rs # Partition table creation
│   │   └── format.rs    # File system formatting
│   ├── iso/             # ISO management
│   │   ├── parser.rs    # ISO parsing and detection
│   │   ├── validator.rs # Hash validation
│   │   ├── hybrid.rs    # Hybrid ISO detection
│   │   └── boot.rs      # Boot record analysis
│   ├── ui/              # User interface (egui)
│   │   ├── app.rs       # Main application UI
│   │   ├── styles.rs    # Theme configuration
│   │   ├── i18n.rs      # Internationalization
│   │   └── components/  # UI components
│   └── utils/           # Utilities
│       ├── hash.rs      # Hash calculations
│       ├── logging.rs   # Logging system
│       ├── platform.rs  # Windows platform utilities
│       └── async_utils.rs # Async helpers
```

## Requirements

- **Operating System**: Windows 10 (64-bit) or Windows 11
- **Rust**: Version 1.75 or later
- **Administrator Privileges**: Required for disk access
- **Visual Studio Build Tools**: For Windows API compilation

## Building

### Prerequisites

1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Install Visual Studio Build Tools with C++ workload
3. Install Windows SDK

### Build Steps

```bash
# Clone the repository
git clone https://github.com/sidozdev/sidozdev.git
cd sidozdev

# Build in release mode
cargo build --release

# The executable will be at:
# target/release/sidozdev.exe
```

### Development Build

```bash
# Build in debug mode (faster compilation)
cargo build

# Run with logging
cargo run -- --log-level=debug
```

## Usage

1. **Launch Sidozdev** with administrator privileges
2. **Select a USB device** from the detected devices list
3. **Choose an ISO file** using the file browser
4. **Configure options**:
   - Boot mode (BIOS/UEFI/UEFI+Secure Boot)
   - Partition scheme (MBR/GPT)
   - File system (FAT32/NTFS/exFAT)
5. **Click Start** to begin writing
6. **Monitor progress** in real-time
7. **Verify** the write upon completion

## Safety Warning

**All data on the selected USB device will be permanently erased.** Always double-check the selected device before starting the operation.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to our repository.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- GUI powered by [egui](https://github.com/emilk/egui)
- Inspired by [Rufus](https://rufus.ie/)

## Support

For issues, feature requests, or questions, please visit our [GitHub Issues](https://github.com/sidozdev/sidozdev/issues) page.

---

**Sidozdev** - Professional USB bootable media creation, reimagined in Rust.
