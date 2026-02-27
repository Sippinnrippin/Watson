# ⚡ Watson

> Blazing fast OSINT tool for finding usernames and emails across 400+ social networks.

Built in Rust with concurrent requests, proxy/Tor support, and multiple output formats.

---

> ⚠️ **DISCLAIMER**
> 
> **Watson is intended for legitimate security research, penetration testing, and educational purposes only.**
> 
> **You are solely responsible for your actions while using this tool.**
> 
> The creator of Watson does not condone, encourage, or endorse any illegal or unethical use of this software. Using this tool to access accounts without authorization or for malicious purposes may violate laws and regulations, including computer crime laws.
> 
> By using Watson, you agree to:
> - ✅ Use the tool only for lawful purposes
> - ✅ Respect the privacy and consent of others
> - ✅ Comply with all applicable laws and regulations
> - ❌ Not use this tool for unauthorized access or harassment
> 
> **The creator assumes no liability for any damages, legal consequences, or misuse of this tool.**

---

## 📖 Overview

Watson is a powerful OSINT (Open Source Intelligence) tool for finding usernames across social networks. Inspired by [Sherlock](https://github.com/sherlock-project/sherlock), it provides fast and efficient username searching with support for proxies and Tor.

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🔍 **Username Search** | Check 400+ sites for username existence |
| 📧 **Email Lookup** | Search for emails across services |
| 🌐 **Proxy Support** | HTTP/HTTPS/SOCKS proxy support |
| 🧅 **Tor Support** | Route requests through Tor network |
| 📄 **Multiple Outputs** | JSON, CSV, HTML report, text |
| ⚡ **Fast** | Built with Rust for optimal performance |
| 💾 **Local/Remote** | Use local data file or fetch from GitHub |

## 📦 Installation

### From Source

```bash
git clone https://github.com/Sippinnrippin/Watson.git
cd Watson
cargo build --release
```

The binary will be at `target/release/watson`

### From Binary

Download the latest release from the [Releases](https://github.com/Sippinnrippin/Watson/releases) page.

Pre-built binaries:
- 🐧 **Linux** (`watson`)
- 🍎 **macOS** (`watson`)

## 🚀 Running Watson

### Option 1: Run from release folder

```bash
./watson -u username
```

### Option 2: Add to PATH

```bash
export PATH="$PATH:/path/to/Watson/target/release"
watson -u username
```

## 💻 Usage

### Basic Usage

```bash
watson -u username
```

### Performance Tips

For faster searches, use `--local`:

```bash
watson -u username --local
```

To limit specific sites (much faster):

```bash
watson -u username --site github twitter instagram --local
```

### Email Search

```bash
watson -m user@example.com
```

### Scrape Emails from Profiles

```bash
watson -u username --emails
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
# JSON
watson -u username -f json -o results.json

# CSV
watson -u username -f csv -o results.csv

# HTML
watson -u username -f html -o results.html
```

## ⚙️ Other Options

```bash
watson --help
```

```
Watson - OSINT username and email lookup tool

Usage: watson [OPTIONS]

Options:
  -u, --username <USERNAME>    Username to search for
  -m, --email <EMAIL>        Email to search for
  -e, --emails                 Scrape found profiles for emails
  -o, --output <FILE>         Output file path
  -f, --format <FORMAT>       Output format (text, json, csv, html)
  -p, --proxy <PROXY>        Proxy URL
  -t, --tor                    Use Tor for requests
  --timeout <TIMEOUT>          Request timeout (default: 15)
  --max-concurrent <N>         Max concurrent (default: 50)
  --nsfw                       Include NSFW sites
  -a, --print-all             Print all results
  -s, --print-found           Print only found results
  -l, --local                 Use local data file
  -v, --verbose               Verbose output
  --list-sites                List supported sites
  -h, --help                  Print help
```

## 🔨 Building

### Requirements

- Rust 1.70+
- Cargo

### Commands

```bash
# Debug
cargo build

# Release
cargo build --release

# Test
cargo test
```

## 📜 License

MIT License

## 🙏 Acknowledgments

Special thanks to the devs over at [sherlock-project](https://github.com/sherlock-project/sherlock) for their inspiration!

---

*Made with 🔥 by [Sippinnrippin](https://github.com/Sippinnrippin)*
