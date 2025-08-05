#!/bin/bash

# POE Gem Calculator - Development Runner
# This script helps with common development tasks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install Rust first:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    print_success "Rust $(rustc --version) found"
}

# Function to run the application
run_app() {
    print_status "Starting POE Gem Calculator..."

    # Set environment variables for development
    export RUST_LOG=${RUST_LOG:-info}
    export RUST_BACKTRACE=${RUST_BACKTRACE:-1}

    # Create cache directory if it doesn't exist
    mkdir -p cache

    # Run with optional arguments
    if [ $# -eq 0 ]; then
        cargo run -- --log-level debug --port 3000
    else
        cargo run -- "$@"
    fi
}

# Function to run tests
run_tests() {
    print_status "Running tests..."
    cargo test --verbose
    print_success "Tests completed"
}

# Function to check code quality
check_quality() {
    print_status "Checking code quality..."

    # Check formatting
    print_status "Checking code formatting..."
    if cargo fmt --check; then
        print_success "Code formatting is correct"
    else
        print_warning "Code formatting issues found. Run 'cargo fmt' to fix."
    fi

    # Run clippy
    print_status "Running clippy lints..."
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_success "No clippy issues found"
    else
        print_error "Clippy found issues"
        return 1
    fi
}

# Function to build for production
build_release() {
    print_status "Building release version..."
    cargo build --release
    print_success "Release build completed"
    print_status "Binary location: target/release/poe-gem-calculator"
}

# Function to clean build artifacts
clean() {
    print_status "Cleaning build artifacts..."
    cargo clean
    rm -rf cache/*
    print_success "Clean completed"
}

# Function to watch for changes and rebuild
watch() {
    if ! command -v cargo-watch &> /dev/null; then
        print_warning "cargo-watch not found. Installing..."
        cargo install cargo-watch
    fi

    print_status "Starting file watcher..."
    cargo watch -x "run -- --log-level debug --port 3000"
}

# Function to run with Docker
docker_run() {
    print_status "Building and running with Docker..."

    if ! command -v docker &> /dev/null; then
        print_error "Docker not found. Please install Docker first."
        exit 1
    fi

    docker build -t poe-gem-calculator .
    docker run -p 3000:3000 -v "$(pwd)/cache:/app/cache" poe-gem-calculator
}

# Function to show help
show_help() {
    echo "POE Gem Calculator - Development Runner"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  run [ARGS]     Run the application in development mode (default)"
    echo "  test           Run all tests"
    echo "  check          Check code quality (fmt + clippy)"
    echo "  build          Build release version"
    echo "  clean          Clean build artifacts and cache"
    echo "  watch          Run with file watching (auto-restart on changes)"
    echo "  docker         Build and run with Docker"
    echo "  help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                           # Run with default settings"
    echo "  $0 run --port 8080          # Run on port 8080"
    echo "  $0 run --log-level trace    # Run with trace logging"
    echo "  $0 test                     # Run tests"
    echo "  $0 check                    # Check code quality"
    echo "  $0 watch                    # Run with auto-restart"
    echo ""
    echo "Environment Variables:"
    echo "  RUST_LOG        Set log level (debug, info, warn, error)"
    echo "  RUST_BACKTRACE  Show backtraces (0, 1, full)"
    echo "  PORT            Server port (default: 3000)"
    echo ""
}

# Main script logic
main() {
    # Change to script directory
    cd "$(dirname "$0")"

    # Check Rust installation
    check_rust

    # Parse command
    case "${1:-run}" in
        "run")
            shift
            run_app "$@"
            ;;
        "test")
            run_tests
            ;;
        "check")
            check_quality
            ;;
        "build")
            build_release
            ;;
        "clean")
            clean
            ;;
        "watch")
            watch
            ;;
        "docker")
            docker_run
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# Handle script interruption
trap 'echo -e "\n${YELLOW}[INFO]${NC} Script interrupted by user"; exit 130' INT

# Run main function with all arguments
main "$@"
