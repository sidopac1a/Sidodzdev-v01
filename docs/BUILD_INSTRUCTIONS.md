# Sidozdev Build Instructions

## Prerequisites

### Required Software

1. **Rust Toolchain** (1.75 or later)
   ```powershell
   # Install from https://rustup.rs/
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   # Or on Windows, download and run rustup-init.exe
   ```

2. **Visual Studio Build Tools** (Windows)
   - Download from: https://visualstudio.microsoft.com/downloads/
   - Select "Desktop development with C++" workload
   - Or install minimal: MSVC v143, Windows 10/11 SDK

3. **Windows SDK** (10.0.19041.0 or later)
   - Included with Visual Studio Build Tools
   - Or standalone from Windows SDK archive

4. **Git** (for cloning)
   ```powershell
   # https://git-scm.com/download/win
   winget install Git.Git
   ```

### System Requirements

- **OS**: Windows 10 (64-bit) version 1903 or later, or Windows 11
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 2GB free space for build artifacts
- **Privileges**: Administrator (for testing disk operations)

## Build Steps

### 1. Clone Repository

```powershell
# Clone the repository
git clone https://github.com/sidozdev/sidozdev.git
cd sidozdev

# Or download and extract ZIP
curl -L -o sidozdev.zip https://github.com/sidozdev/sidozdev/archive/main.zip
Expand-Archive sidozdev.zip -DestinationPath .
cd sidozdev-main
```

### 2. Verify Rust Installation

```powershell
# Check Rust version (must be >= 1.75)
rustc --version
# Expected: rustc 1.75.0 (or later)

# Check Cargo version
cargo --version
# Expected: cargo 1.75.0 (or later)

# Update if needed
rustup update
```

### 3. Install Dependencies

```powershell
# Rust will automatically download and compile dependencies
# No manual dependency installation needed

# Optional: Install cargo tools for development
cargo install cargo-watch      # Auto-rebuild on changes
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-audit      # Security audit
```

### 4. Build Debug Version

```powershell
# Build in debug mode (faster compilation, slower runtime)
cargo build

# Output: target/debug/sidozdev.exe
```

### 5. Build Release Version

```powershell
# Build in release mode (optimized, slower compilation)
cargo build --release

# Output: target/release/sidozdev.exe
```

### 6. Run Tests

```powershell
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_app_state_creation

# Run tests in release mode (slower but more accurate)
cargo test --release
```

### 7. Run the Application

```powershell
# Run debug version
cargo run

# Run with logging
cargo run -- --log-level=debug

# Run release version
cargo run --release

# Run with specific log filter
$env:RUST_LOG="sidozdev=debug,eframe=info"
cargo run --release
```

## Advanced Build Options

### Cross-Compilation (Future)

```powershell
# For Windows ARM64 (future support)
rustup target add aarch64-pc-windows-msvc
cargo build --target aarch64-pc-windows-msvc
```

### Static Linking

The project is configured for static CRT linking in `.cargo/config.toml`:
```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

This ensures the executable doesn't require VC++ redistributables.

### Code Coverage

```powershell
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# View report: tarpaulin-report.html
```

### Security Audit

```powershell
# Install cargo-audit
cargo install cargo-audit

# Check for known vulnerabilities
cargo audit
```

## Development Workflow

### Hot Reload Development

```powershell
# Install cargo-watch
cargo install cargo-watch

# Auto-rebuild on file changes
cargo watch -x run

# Auto-run tests on changes
cargo watch -x test
```

### IDE Setup

#### Visual Studio Code
1. Install extensions:
   - rust-analyzer
   - CodeLLDB (debugger)
   - Better TOML
   - crates

2. Configure `.vscode/settings.json`:
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.procMacro.enable": true
}
```

#### JetBrains RustRover / IntelliJ IDEA
1. Install Rust plugin
2. Import project (Cargo.toml)
3. Configure Run Configuration for `cargo run`

### Debugging

```powershell
# Build with debug symbols
cargo build

# Use VS Code with CodeLLDB extension
# Or use WinDbg for native debugging

# Enable backtrace on panic
$env:RUST_BACKTRACE=1
cargo run
```

## Troubleshooting

### Common Build Errors

#### Error: "linker `link.exe` not found"
**Solution**: Install Visual Studio Build Tools with C++ workload
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --add Microsoft.VisualStudio.Workload.VCTools"
```

#### Error: "Windows SDK not found"
**Solution**: Install Windows SDK
```powershell
winget install Microsoft.WindowsSDK
```

#### Error: "cannot find -lsetupapi"
**Solution**: Ensure Windows SDK is properly installed and LIB environment variable is set
```powershell
# Check environment
$env:LIB
# Should include Windows SDK library paths
```

#### Error: "failed to run custom build command for `winres`"
**Solution**: Ensure icon file exists or disable resource compilation
```powershell
# Create placeholder icon (or copy actual icon)
New-Item -ItemType Directory -Force -Path assets/icons
# Place app.ico in assets/icons/
```

#### Error: "feature `crt-static` is not supported"
**Solution**: Update Rust toolchain
```powershell
rustup update
```

### Performance Issues

#### Slow Compilation
```powershell
# Use sccache for faster rebuilds
cargo install sccache
$env:RUSTC_WRAPPER="sccache"

# Or use mold linker (Linux/Mac, not Windows)
```

#### Large Binary Size
```powershell
# Check binary size
cargo bloat --release

# Strip symbols (already done in release profile)
# Profile is configured with strip = true
```

## Packaging

### Create Installer (MSI)

```powershell
# Install WiX Toolset
winget install WiXToolset.WiXToolset

# Build installer (requires WiX)
# TODO: Add WiX configuration
```

### Create Portable ZIP

```powershell
# Build release
cargo build --release

# Create distribution directory
New-Item -ItemType Directory -Force -Path dist\sidozdev-1.0.0

# Copy executable
Copy-Item target\release\sidozdev.exe dist\sidozdev-1.0.0\

# Copy assets
Copy-Item -Recurse assets dist\sidozdev-1.0.0\

# Copy documentation
Copy-Item README.md dist\sidozdev-1.0.0\
Copy-Item LICENSE dist\sidozdev-1.0.0\

# Create ZIP
Compress-Archive -Path dist\sidozdev-1.0.0 -DestinationPath dist\sidozdev-1.0.0-x64.zip
```

### Code Signing

```powershell
# Sign with EV certificate (requires certificate)
signtool sign /fd sha256 /a /tr http://timestamp.digicert.com /td sha256 target\release\sidozdev.exe

# Verify signature
signtool verify /pa target\release\sidozdev.exe
```

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/build.yml
name: Build and Test

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    runs-on: windows-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        components: rustfmt, clippy

    - name: Cache Dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Check Formatting
      run: cargo fmt -- --check

    - name: Run Clippy
      run: cargo clippy --all-targets --all-features -- -D warnings

    - name: Build
      run: cargo build --release

    - name: Run Tests
      run: cargo test --release

    - name: Upload Artifact
      uses: actions/upload-artifact@v3
      with:
        name: sidozdev-release
        path: target/release/sidozdev.exe
```

## Verification

### Check Build Success

```powershell
# Verify executable exists
Test-Path target\release\sidozdev.exe

# Check file size (should be ~10-15MB)
(Get-Item target\release\sidozdev.exe).Length / 1MB

# Check dependencies (should be minimal)
dumpbin /dependents target\release\sidozdev.exe

# Run executable
.\target\release\sidozdev.exe
```

### Smoke Test

```powershell
# Run with trace logging
$env:RUST_LOG="trace"
.\target\release\sidozdev.exe

# Expected: Application window opens
# Check: Device list, ISO selection, options, progress, logs
```

## Support

For build issues:
1. Check this troubleshooting section
2. Search GitHub Issues
3. Create new issue with:
   - Rust version (`rustc --version`)
   - Windows version (`winver`)
   - Build command used
   - Full error message
   - Build logs

---

**Last Updated**: 2024  
**Build Version**: 1.0.0
