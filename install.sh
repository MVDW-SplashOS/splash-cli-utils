#!/bin/bash

# Installation script for splash-cli-utils
# This script builds and installs the utilities

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
PREFIX="/usr/local"
PROFILE="release"
INTERACTIVE=true
FORCE=false

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}$1${NC}"
}

# Function to show help
show_help() {
    cat << EOF
splash-cli-utils Installation Script

USAGE:
    ./install.sh [OPTIONS]

OPTIONS:
    -p, --prefix PATH       Installation prefix (default: /usr/local)
    -d, --debug            Build in debug mode instead of release
    -y, --yes              Non-interactive mode, assume yes to all prompts
    -f, --force            Force installation even if binaries exist
    -u, --uninstall        Uninstall the tools instead of installing
    -h, --help             Show this help message

EXAMPLES:
    ./install.sh                    # Install to /usr/local/bin
    ./install.sh -p ~/.local        # Install to ~/.local/bin
    ./install.sh -d                 # Build in debug mode
    ./install.sh -y -f              # Force install without prompts
    ./install.sh -u                 # Uninstall

Note: You may need sudo privileges for system-wide installation.
EOF
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check dependencies
check_dependencies() {
    print_header "Checking dependencies..."

    if ! command_exists cargo; then
        print_error "Rust/Cargo is not installed!"
        print_error "Please install Rust from https://rustup.rs/ and try again."
        exit 1
    fi

    print_status "Rust/Cargo found: $(cargo --version)"

    # Check if we're in the right directory
    if [[ ! -f "Makefile" ]] || [[ ! -d "mkdev" ]] || [[ ! -d "signals" ]]; then
        print_error "This script must be run from the splash-cli-utils root directory!"
        exit 1
    fi

    print_status "Project structure verified"
}

# Function to build the tools
build_tools() {
    print_header "Building tools..."

    print_status "Building with profile: $PROFILE"

    if ! make build PROFILE="$PROFILE"; then
        print_error "Build failed!"
        exit 1
    fi

    print_status "Build completed successfully"
}

# Function to check if binaries already exist
check_existing() {
    local bindir="$PREFIX/bin"
    local mkdev_exists=false
    local sig_exists=false

    if [[ -f "$bindir/mkdev" ]]; then
        mkdev_exists=true
    fi

    if [[ -f "$bindir/sig" ]]; then
        sig_exists=true
    fi

    if [[ "$mkdev_exists" == true ]] || [[ "$sig_exists" == true ]]; then
        if [[ "$FORCE" == false ]]; then
            print_warning "Some binaries already exist:"
            [[ "$mkdev_exists" == true ]] && print_warning "  - $bindir/mkdev"
            [[ "$sig_exists" == true ]] && print_warning "  - $bindir/sig"

            if [[ "$INTERACTIVE" == true ]]; then
                echo -n "Do you want to overwrite them? (y/N): "
                read -r response
                if [[ ! "$response" =~ ^[Yy]$ ]]; then
                    print_status "Installation cancelled"
                    exit 0
                fi
            else
                print_error "Binaries exist and --force not specified"
                exit 1
            fi
        fi
    fi
}

# Function to install the tools
install_tools() {
    print_header "Installing tools..."

    local bindir="$PREFIX/bin"

    print_status "Installing to: $bindir"

    # Check if we need sudo
    local use_sudo=false
    if [[ ! -w "$(dirname "$bindir")" ]] && [[ ! -w "$bindir" ]] 2>/dev/null; then
        if [[ "$PREFIX" == "/usr/local" ]] || [[ "$PREFIX" =~ ^/usr ]]; then
            use_sudo=true
            print_warning "Root privileges required for installation to $PREFIX"
        else
            print_error "No write permission to $bindir"
            exit 1
        fi
    fi

    # Create bin directory if it doesn't exist
    if [[ "$use_sudo" == true ]]; then
        sudo mkdir -p "$bindir"
    else
        mkdir -p "$bindir"
    fi

    # Install binaries
    local install_cmd="make install PREFIX=\"$PREFIX\" PROFILE=\"$PROFILE\""

    if [[ "$use_sudo" == true ]]; then
        # Find cargo path to pass to sudo environment
        local cargo_path=$(which cargo)
        local cargo_dir=$(dirname "$cargo_path")
        if ! sudo env PATH="$cargo_dir:$PATH" bash -c "$install_cmd"; then
            print_error "Installation failed!"
            exit 1
        fi
    else
        if ! eval "$install_cmd"; then
            print_error "Installation failed!"
            exit 1
        fi
    fi

    print_status "Installation completed successfully!"
    print_status ""
    print_status "Installed tools:"
    print_status "  mkdev -> $bindir/mkdev"
    print_status "  sig   -> $bindir/sig"
    print_status ""
    print_status "You can now use:"
    print_status "  mkdev <source> <target>    # Write disk images to devices"
    print_status "  sig <signal> <process>     # Send signals to processes"
}

# Function to uninstall the tools
uninstall_tools() {
    print_header "Uninstalling tools..."

    local bindir="$PREFIX/bin"
    local use_sudo=false

    # Check if we need sudo
    if [[ ! -w "$bindir" ]] 2>/dev/null; then
        if [[ "$PREFIX" == "/usr/local" ]] || [[ "$PREFIX" =~ ^/usr ]]; then
            use_sudo=true
            print_warning "Root privileges required for uninstallation from $PREFIX"
        fi
    fi

    local uninstall_cmd="make uninstall PREFIX=\"$PREFIX\""

    if [[ "$use_sudo" == true ]]; then
        # Find cargo path to pass to sudo environment (needed for potential rebuilds)
        local cargo_path=$(which cargo)
        local cargo_dir=$(dirname "$cargo_path")
        if ! sudo env PATH="$cargo_dir:$PATH" bash -c "$uninstall_cmd"; then
            print_error "Uninstallation failed!"
            exit 1
        fi
    else
        if ! eval "$uninstall_cmd"; then
            print_error "Uninstallation failed!"
            exit 1
        fi
    fi

    print_status "Uninstallation completed successfully!"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--prefix)
            PREFIX="$2"
            shift 2
            ;;
        -d|--debug)
            PROFILE="debug"
            shift
            ;;
        -y|--yes)
            INTERACTIVE=false
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -u|--uninstall)
            check_dependencies
            uninstall_tools
            exit 0
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main installation process
print_header "splash-cli-utils Installation"
print_status "Configuration:"
print_status "  Install prefix: $PREFIX"
print_status "  Build profile:  $PROFILE"
print_status "  Interactive:    $INTERACTIVE"
print_status ""

# Check dependencies
check_dependencies

# Check for existing binaries
check_existing

# Confirm installation if interactive
if [[ "$INTERACTIVE" == true ]]; then
    echo -n "Proceed with installation? (Y/n): "
    read -r response
    if [[ "$response" =~ ^[Nn]$ ]]; then
        print_status "Installation cancelled"
        exit 0
    fi
fi

# Build and install
build_tools
install_tools

print_header "Installation Complete!"
print_status "Run './install.sh --help' for more options"
