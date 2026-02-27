use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "watson")]
#[command(version = "0.1.0")]
#[command(about = "Watson - OSINT username and email lookup tool", long_about = None)]
pub struct Cli {
    /// Username to search for
    #[arg(value_name = "USERNAME", short = 'u', long = "username")]
    pub username: Option<String>,

    /// File containing usernames to search (one per line)
    #[arg(long = "file", short = 'F')]
    pub file: Option<String>,

    /// Generate username variations
    #[arg(long = "variations")]
    pub variations: bool,

    /// Scrape found profiles for emails
    #[arg(long = "emails")]
    pub scrape_emails: bool,

    /// Rotate User-Agent to avoid detection
    #[arg(long = "rotate-ua")]
    pub rotate_ua: bool,

    /// Rate limit in milliseconds between requests to same domain
    #[arg(long = "rate-limit")]
    pub rate_limit: Option<u64>,

    /// Email to search for
    #[arg(value_name = "EMAIL", short = 'm', long = "email")]
    pub email: Option<String>,

    /// Output file path
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<String>,

    /// Output format (text, json, csv, html)
    #[arg(short = 'f', long = "format", default_value = "text")]
    pub format: OutputFormat,

    /// Proxy URL (e.g., socks5://127.0.0.1:1080)
    #[arg(short = 'p', long = "proxy")]
    pub proxy: Option<String>,

    /// Use Tor for requests
    #[arg(short = 't', long = "tor")]
    pub tor: bool,

    /// Request timeout in seconds
    #[arg(long = "timeout", default_value = "15")]
    pub timeout: u64,

    /// Maximum concurrent requests
    #[arg(long = "max-concurrent", default_value = "50")]
    pub max_concurrent: usize,

    /// Include NSFW sites in search
    #[arg(long = "nsfw")]
    pub nsfw: bool,

    /// Print all results (including not found)
    #[arg(short = 'a', long = "print-all")]
    pub print_all: bool,

    /// Print only found results
    #[arg(short = 's', long = "print-found")]
    pub print_found: bool,

    /// Use local data file
    #[arg(short = 'l', long = "local")]
    pub local: bool,

    /// Site to search (can be specified multiple times)
    #[arg(long = "site")]
    pub site: Option<Vec<String>>,

    /// Enable verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// List supported sites
    #[arg(long = "list-sites")]
    pub list_sites: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum OutputFormat {
    /// Plain text output
    Text,
    /// JSON output
    Json,
    /// CSV output
    Csv,
    /// HTML report
    Html,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Csv => write!(f, "csv"),
            OutputFormat::Html => write!(f, "html"),
        }
    }
}
