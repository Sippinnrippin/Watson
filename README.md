# Watson

Blazing fast OSINT tool for finding usernames and emails across 400+ social networks. Built in Rust with concurrent requests, proxy/Tor support, and multiple output formats. Special thanks to the devs over at https://github.com/sherlock-project/sherlock for their site data and inspiration!

## Overview

Watson is a powerful OSINT (Open Source Intelligence) tool for finding usernames across social networks. Inspired by Sherlock, it provides fast and efficient username searching with support for proxies and Tor.

## Features

- **Username Search**: Check 400+ sites for username existence
- **Email Lookup**: Search for emails across services
- **Proxy Support**: HTTP/HTTPS/SOCKS proxy support
- **Tor Support**: Route requests through Tor network
- **Multiple Output Formats**: JSON, CSV, HTML report, text
- **Fast**: Built with Rust for optimal performance
- **Local/Remote Data**: Use local data file or fetch from GitHub

## Installation

### From Source

```bash
git clone https://github.com/Sippinnrippin/Watson.git
cd Watson
cargo build --release
```

The binary will be at `target/release/watson`

### From Binary

Download the latest release from the [Releases](https://github.com/Sippinnrippin/Watson/releases) page.

Pre-built binaries available for:
- **Linux** (x86_64)
- **macOS** (x86_64)
- **Windows** (x86_64.exe)

## Usage

### Basic Usage

```bash
watson -u username
```

### Email Search

```bash
watson -m user@example.com
```

### Search Specific Sites

```bash
watson -u username --site github --site twitter
```

### Using Proxy

```bash
watson -u username --proxy socks5://127.0.0.1:1080
```

### Using Tor

```bash
watson -u username --tor
```

### Output Formats

```bash
# JSON output
watson -u username -f json -o results.json

# CSV output
watson -u username -f csv -o results.csv

# HTML report
watson -u username -f html -o results.html

# Text output (default)
watson -u username -o results.txt
```

### Other Options

```bash
watson --help
```

```
Watson - OSINT username and email lookup tool

Usage: watson [OPTIONS]

Options:
  -u, --username <USERNAME>
          Username to search for

  -m, --email <EMAIL>
          Email to search for

  -o, --output <FILE>
          Output file path

  -f, --format <FORMAT>
          Output format (text, json, csv, html)

          Possible values:
          - text: Plain text output
          - json: JSON output
          - csv:  CSV output
          - html: HTML report
          
          [default: text]

  -p, --proxy <PROXY>
          Proxy URL (e.g., socks5://127.0.0.1:1080)

  -t, --tor
          Use Tor for requests

      --timeout <TIMEOUT>
          Request timeout in seconds
          
          [default: 60]

      --max-concurrent <MAX_CONCURRENT>
          Maximum concurrent requests
          
          [default: 20]

      --nsfw
          Include NSFW sites

  -a, --print-all
          Print all results (including not found)

  -s, --print-found
          Print only found results

  -l, --local
          Use local data file

  -e, --site <SITE>
          Site to search (can be specified multiple times)

  -v, --verbose
          Enable verbose output

      --list-sites
          List supported sites

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Data

Watson uses the same site data as Sherlock, which is fetched from:
https://github.com/sherlock-project/sherlock

You can use the local data file (`data/sites.json`) by passing the `-l` flag.

## Building

### Requirements

- Rust 1.70+
- Cargo

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

### Building for Windows

#### Option 1: Build on Windows (Recommended)

Install Rust on Windows, then run in CMD or PowerShell:

```cmd
# Install Rust (if not installed)
rustup install stable

# Build for Windows
rustup target add x86_64-pc-windows-msvc
cargo build --release
```

The executable will be at `target\release\watson.exe`

#### Option 2: Cross-Compile from Linux/WSL2

```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Install MinGW-w64
sudo apt install mingw-w64

# Cross-compile
cargo build --target x86_64-pc-windows-gnu --release
```

The executable will be at `target/x86_64-pc-windows-gnu/release/watson.exe`

## License

MIT License

## Acknowledgments

- Special thanks to the devs over at https://github.com/sherlock-project/sherlock for their site data and inspiration
