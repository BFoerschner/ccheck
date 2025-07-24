# Default recipe to run when no arguments are provided
default:
    @just --list

test: test-basic test-shells

test-basic:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running integration tests..."

    # Test basic functionality with alpine image
    echo "Testing basic command check..."
    cargo run -- alpine:latest ls cat grep --shell sh

    # Test file input functionality
    echo "Testing file input..."
    echo -e "ls\ncat\ngrep\n# This is a comment\nmkdir" > /tmp/test_commands.txt
    cargo run -- alpine:latest --file /tmp/test_commands.txt --shell sh
    rm -f /tmp/test_commands.txt

    # Test missing commands (should exit with error)
    echo "Testing missing commands detection..."
    if cargo run -- alpine:latest nonexistent_command_12345 --shell sh; then
        echo "ERROR: Should have failed for missing command"
        exit 1
    else
        echo "SUCCESS: Correctly detected missing command"
    fi

    echo "test successful"

test-shells:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running multi-shell tests..."
    
    # Test with different shell entrypoints (all use POSIX scripts internally)
    echo "Testing with bash entrypoint..."
    cargo run -- ubuntu:latest ls cat grep --shell bash
    
    echo "Testing with sh entrypoint..."
    cargo run -- alpine:latest ls cat grep --shell sh
    
    echo "Testing with nu shell entrypoint..."
    cargo run -- hustcer/nushell:latest ls cat grep --shell nu

    echo "test successful"

# Install the binary locally
install:
    cargo install --path .
