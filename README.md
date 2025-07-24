# ccheck

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=flat&logo=docker&logoColor=white)](https://www.docker.com/)

A simple CLI tool that checks if commands are available inside Docker containers. Useful for validating Docker images and making sure your containerized apps have all the tools they need.

## Features

- Check multiple commands in a single container run
- Uses templates to support POSIX shells and nu
- Read command lists from files (with comment support)
- Colored output to easily spot missing commands
- Works with any Docker image

## Installation

### From Source (Recommended)

```sh
cargo install --git https://github.com/bfoerschner/ccheck.git
```

## Quick Start

Check if basic commands exist in Alpine Linux:

```bash
ccheck alpine:latest ls cat grep curl
```

Output:
```sh
Checking Docker image: alpine:latest
Checking 4 command(s):

The following commands are missing:

curl 

Summary: 3/4 commands installed
Error: 1 command(s) missing
```

## Usage Examples

### Command Line Arguments

```bash
# Check specific commands
ccheck ubuntu:latest python3 pip git nodejs npm

# Use different shell (default: zsh)
ccheck --shell bash alpine:latest make gcc g++

# Short form
ccheck -s sh debian:latest wget curl
```

### File Input

Create a file with commands to check:

```bash
# commands.txt
python3
pip
git
nodejs
npm
# This is a comment - ignored
docker
kubectl
```

### Check commands from file
```bash
ccheck --file commands.txt ubuntu:latest
```

### Different Shell Entry Points

```bash
# Using bash entrypoint
ccheck --shell bash alpine:latest ls cat grep

# Using nu shell entrypoint (still uses POSIX scripts internally)
ccheck --shell nu hustcer/nushell:latest ls cat grep

# Using zsh entrypoint (default)
ccheck ubuntu:latest zsh git curl
```

## Advanced Usage

### Validating Custom Images

Check that your custom Docker image has all the development tools you need:

```bash
# Create a comprehensive check list
cat > dev-tools.txt << EOF
# Core utilities
ls
cat
grep
find
sed
awk

# Development tools
git
curl
wget
vim
nano

# Build tools
make
gcc
g++

# Package managers
apt-get
pip
npm
EOF
```

# Validate your custom image
```bash
ccheck -f dev-tools.txt mycompany/dev-image:latest
```

### Using in CI/CD

Here's how to use ccheck in your GitHub Actions workflow:

```yaml
# .github/workflows/validate-image.yml
name: Validate Docker Image
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install ccheck
      run: |
        cargo install --git https://github.com/bfoerschner/ccheck.git
    
    - name: Validate production image
      run: |
        ccheck production-image:latest curl jq envsubst
    
    - name: Validate development image  
      run: |
        ccheck -f .github/required-tools.txt dev-image:latest
```

## Development

### Running Tests

We have tests for different shell types:

```bash
# Run all tests
just test

# Run only POSIX shell tests
just test-posix  

# Run only Nu shell tests
just test-nu
```

### Building from Source
```sh
# Debug build
cargo build

# Release build
cargo build --release

# Run without installing
cargo run -- alpine:latest ls cat grep
```

## Contributing

Contributions are welcome! You can help by:

- Reporting bugs and issues
- Suggesting new features
- Improving documentation
- Adding test cases
- Adding support for more shells

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
