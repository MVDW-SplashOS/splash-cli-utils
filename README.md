# splash-cli-utils

An alternative collection of CLI utilities written in Rust, providing modern replacements for common system tools with improved performance, safety, and user experience.

## 🚀 Available Tools

### ✅ Completed Tools

- **mkdev** - Advanced disk image writer with auto-optimization and progress tracking
- **signals** - Process signal management tool (binary: `sig`)

### 📋 Planned Tools

- **mkimg** - Create disk images from devices or folders
- **delete** - Smart file deletion with trash support
- **copy** - Advanced file/folder copying with clipboard integration
- **paste** - Paste files/folders from clipboard
- **cut** - Cut files/folders to clipboard or another location
- **archive** - Universal archive management (zip, tar, etc.)
- **seek** - Fast file/folder search by name or content
- **peek** - Enhanced file content preview (head/tail with extras)

## 📦 Installation

### Quick Install (Recommended)

The easiest way to build and install both tools:

```bash
./install.sh
```

This will install both `mkdev` and `sig` to `/usr/local/bin` by default.

### Custom Installation

```bash
# Install to a custom location
./install.sh --prefix ~/.local

# Build in debug mode
./install.sh --debug

# Non-interactive installation
./install.sh --yes

# Force overwrite existing binaries
./install.sh --force

# See all options
./install.sh --help
```

### Manual Build and Install

If you prefer to use the Makefile directly:

```bash
# Build both tools
make build

# Build individual tools
make mkdev
make signals

# Install (may require sudo for /usr/local)
make install

# Install to custom location
make install PREFIX=~/.local

# Build in debug mode
make build PROFILE=debug

# Clean build artifacts
make clean
```

### Using Cargo Workspace

You can also build using Cargo directly:

```bash
# Build all tools in the workspace
cargo build --release --workspace

# Build individual tools
cargo build --release -p mkdev
cargo build --release -p sig

# Run tests
cargo test --workspace

# Check code
cargo check --workspace
cargo clippy --workspace
cargo fmt --workspace
```

## 🛠️ Tool Usage

### mkdev - Disk Image Writer

A modern replacement for `dd` with automatic buffer optimization and real-time progress tracking.

```bash
# Basic usage
mkdev source.iso /dev/sdX

# With custom buffer size
mkdev source.iso /dev/sdX --buffer-size 32

# Examples
mkdev ubuntu-22.04.iso /dev/sdc
mkdev raspios-lite.img /dev/sdc --buffer-size 64
```

**Features:**
- Auto-detects optimal buffer size for your hardware
- Real-time progress with speed and ETA
- Data integrity with sync operations
- Safe confirmation prompts
- Modern, colorful output

### sig - Signal Management

Send signals to processes by PID or name with advanced matching.

```bash
# Send signal by PID
sig kill 1234
sig term 1234
sig int 1234

# Send signal by process name
sig kill chrome
sig term nginx
sig int python

# Send to all matching processes
sig kill --all chrome
sig term -a python

# Using signal numbers
sig 9 1234
sig 15 chrome --all
```

**Supported signals:**
- `int`, `interrupt`, `sigint` (2)
- `term`, `terminate`, `sigterm` (15)
- `kill`, `sigkill` (9)
- `hup`, `hangup`, `sighup` (1)
- `quit`, `sigquit` (3)
- `usr1`, `sigusr1` (10)
- `usr2`, `sigusr2` (12)
- `stop`, `sigstop` (19)
- `cont`, `sigcont` (18)
- Or any numeric signal

## 📋 Requirements

- **Rust** 1.70+ (install from [rustup.rs](https://rustup.rs/))
- **Linux** (both tools use Linux-specific system calls)
- **libc** (automatically handled by Cargo)

## 🚧 Development

### Project Structure

```
splash-cli-utils/
├── Cargo.toml          # Workspace configuration
├── Makefile           # Build automation
├── install.sh         # Installation script
├── mkdev/             # Disk image writer
│   ├── Cargo.toml
│   └── src/main.rs
└── signals/           # Signal management tool
    ├── Cargo.toml
    └── src/main.rs
```

### Development Commands

```bash
# Format code
make fmt

# Run linter
make clippy

# Check without building
make check

# Run tests
make test

# Development build
make dev-build

# Get project info
make info
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting: `make test && make clippy`
5. Submit a pull request

## 🔧 Uninstallation

```bash
# Using the install script
./install.sh --uninstall

# Using make
make uninstall

# Manual removal
rm /usr/local/bin/mkdev /usr/local/bin/sig
```

## 📄 License

This project is dual-licensed under MIT OR Apache-2.0. See the LICENSE file for details.

## 🤝 Acknowledgments

These tools are inspired by traditional Unix utilities but redesigned with modern safety, performance, and usability in mind.
