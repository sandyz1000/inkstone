#!/bin/bash

# Inkstone Web App - Build and Development Script

set -e

COLOR_RESET='\033[0m'
COLOR_GREEN='\033[0;32m'
COLOR_BLUE='\033[0;34m'
COLOR_YELLOW='\033[1;33m'
COLOR_RED='\033[0;31m'

print_step() {
    echo -e "${COLOR_BLUE}==>${COLOR_RESET} $1"
}

print_success() {
    echo -e "${COLOR_GREEN}✓${COLOR_RESET} $1"
}

print_warning() {
    echo -e "${COLOR_YELLOW}⚠${COLOR_RESET} $1"
}

print_error() {
    echo -e "${COLOR_RED}✗${COLOR_RESET} $1"
}

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install from https://rustup.rs/"
        exit 1
    fi
    print_success "Cargo found: $(cargo --version)"
    
    if ! command -v trunk &> /dev/null; then
        print_warning "Trunk not found. Installing..."
        cargo install trunk
    fi
    print_success "Trunk found: $(trunk --version)"
    
    if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
        print_warning "wasm32-unknown-unknown target not found. Installing..."
        rustup target add wasm32-unknown-unknown
    fi
    print_success "wasm32-unknown-unknown target installed"
}

# Development server
dev() {
    print_step "Starting development server..."
    cd "$(dirname "$0")"
    trunk serve --open
}

# Build for production
build() {
    print_step "Building for production..."
    cd "$(dirname "$0")"
    trunk build --release
    print_success "Build complete! Output in dist/"
}

# Clean build artifacts
clean() {
    print_step "Cleaning build artifacts..."
    cd "$(dirname "$0")"
    trunk clean
    cargo clean
    print_success "Clean complete!"
}

# Run tests
test() {
    print_step "Running tests..."
    cd "$(dirname "$0")"
    cargo test --target wasm32-unknown-unknown
}

# Check code
check() {
    print_step "Checking code..."
    cd "$(dirname "$0")"
    cargo check --target wasm32-unknown-unknown
    print_success "Code check complete!"
}

# Format code
fmt() {
    print_step "Formatting code..."
    cd "$(dirname "$0")"
    cargo fmt
    print_success "Code formatted!"
}

# Show help
help() {
    cat << EOF
Inkstone Web App - Build and Development Script

Usage: $0 <command>

Commands:
    dev         Start development server with hot reload
    build       Build for production (optimized)
    clean       Clean build artifacts
    test        Run tests
    check       Check code for errors
    fmt         Format code
    help        Show this help message

Examples:
    $0 dev              # Start dev server
    $0 build            # Build for production
    $0 clean            # Clean everything

EOF
}

# Main
case "${1:-dev}" in
    dev)
        check_prerequisites
        dev
        ;;
    build)
        check_prerequisites
        build
        ;;
    clean)
        clean
        ;;
    test)
        check_prerequisites
        test
        ;;
    check)
        check_prerequisites
        check
        ;;
    fmt)
        fmt
        ;;
    help|--help|-h)
        help
        ;;
    *)
        print_error "Unknown command: $1"
        help
        exit 1
        ;;
esac
