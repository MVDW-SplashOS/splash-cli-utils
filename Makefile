# Makefile for splash-cli-utils
# Builds and installs mkdev and signals utilities

.PHONY: all build install uninstall clean help mkdev signals test

# Default installation directory
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin

# Cargo build profile (release or debug)
PROFILE ?= release
CARGO_FLAGS = --$(PROFILE)

# Color output
GREEN = \033[0;32m
YELLOW = \033[1;33m
RED = \033[0;31m
NC = \033[0m # No Color

all: build

help:
	@echo "$(YELLOW)splash-cli-utils Build System$(NC)"
	@echo ""
	@echo "Available targets:"
	@echo "  $(GREEN)build$(NC)     - Build both mkdev and signals"
	@echo "  $(GREEN)mkdev$(NC)     - Build only mkdev"
	@echo "  $(GREEN)signals$(NC)   - Build only signals"
	@echo "  $(GREEN)install$(NC)   - Install both tools to $(BINDIR)"
	@echo "  $(GREEN)uninstall$(NC) - Remove installed tools"
	@echo "  $(GREEN)clean$(NC)     - Clean build artifacts"
	@echo "  $(GREEN)test$(NC)      - Run tests for both projects"
	@echo "  $(GREEN)help$(NC)      - Show this help message"
	@echo ""
	@echo "Configuration:"
	@echo "  PREFIX=$(PREFIX)"
	@echo "  BINDIR=$(BINDIR)"
	@echo "  PROFILE=$(PROFILE)"
	@echo ""
	@echo "Examples:"
	@echo "  make build              # Build both tools in release mode"
	@echo "  make install            # Install to /usr/local/bin"
	@echo "  make install PREFIX=~   # Install to ~/bin"
	@echo "  make build PROFILE=debug  # Build in debug mode"

build: mkdev signals

mkdev:
	@echo "$(GREEN)Building mkdev...$(NC)"
	@cargo build $(CARGO_FLAGS) --bin mkdev
	@echo "$(GREEN)mkdev built successfully$(NC)"

signals:
	@echo "$(GREEN)Building signals...$(NC)"
	@cargo build $(CARGO_FLAGS) --bin sig
	@echo "$(GREEN)signals built successfully$(NC)"

install: build
	@echo "$(YELLOW)Installing to $(BINDIR)...$(NC)"
	@mkdir -p $(BINDIR)
	@cp target/$(PROFILE)/mkdev $(BINDIR)/mkdev
	@cp target/$(PROFILE)/sig $(BINDIR)/sig
	@chmod +x $(BINDIR)/mkdev $(BINDIR)/sig
	@echo "$(GREEN)Installation complete!$(NC)"
	@echo "$(GREEN)mkdev installed as: $(BINDIR)/mkdev$(NC)"
	@echo "$(GREEN)signals installed as: $(BINDIR)/sig$(NC)"

uninstall:
	@echo "$(YELLOW)Removing installed binaries...$(NC)"
	@rm -f $(BINDIR)/mkdev
	@rm -f $(BINDIR)/sig
	@echo "$(GREEN)Uninstall complete$(NC)"

clean:
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	@cargo clean
	@echo "$(GREEN)Clean complete$(NC)"

test:
	@echo "$(GREEN)Running tests...$(NC)"
	@cargo test --workspace
	@echo "$(GREEN)All tests completed$(NC)"

# Development targets
dev-build: PROFILE = debug
dev-build: build

check:
	@echo "$(GREEN)Checking workspace...$(NC)"
	@cargo check --workspace

fmt:
	@echo "$(GREEN)Formatting code...$(NC)"
	@cargo fmt --all

clippy:
	@echo "$(GREEN)Running clippy...$(NC)"
	@cargo clippy --workspace -- -D warnings

# Package information
info:
	@echo "$(YELLOW)Project Information:$(NC)"
	@echo "Workspace members: $$(cargo metadata --no-deps --format-version 1 | jq -r '.workspace_members[]' | cut -d'#' -f1 | tr '\n' ' ')"
	@echo "Build profile: $(PROFILE)"
	@echo "Install prefix: $(PREFIX)"
