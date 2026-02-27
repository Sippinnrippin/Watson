mod cli;
mod data;
mod email;
mod engine;
mod http;
mod output;
mod ratelimit;
mod scrape;
mod ua;
mod variations;

use clap::Parser;
use cli::{Cli, OutputFormat};
use data::SitesData;
use engine::{QueryResult, SearchEngine};
use output::SearchReport;
use scrape::scrape_emails_from_results;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use tracing::info;
use variations::generate_variations;

fn load_sites_data(local: bool) -> Result<HashMap<String, data::SiteInfo>, Box<dyn std::error::Error>> {
    let data = if local {
        let path = "data/sites.json";
        if std::path::Path::new(path).exists() {
            SitesData::load_from_file(path)?
        } else {
            return Err(format!("Error: Local data file not found: {}\nUse --local with a local sites.json or remove --local to fetch from GitHub.", path).into());
        }
    } else {
        let url = "https://raw.githubusercontent.com/sherlock-project/sherlock/master/sherlock_project/resources/data.json";
        info!("Fetching sites data from GitHub...");
        
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
            
        let response = client.get(url).send()?;
        
        if !response.status().is_success() {
            return Err(format!("Error: Failed to fetch sites data: HTTP {}\nUse --local with a local sites.json.", response.status()).into());
        }
        
        let json = response.text()?;
        SitesData::load_from_json(&json)?
    };
    
    Ok(data.sites)
}

fn handle_output(
    report: &SearchReport,
    format: &OutputFormat,
    output_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = match format {
        OutputFormat::Text => report.to_text(),
        OutputFormat::Json => report.to_json()?,
        OutputFormat::Csv => report.to_csv(),
        OutputFormat::Html => report.to_html(),
    };

    match output_file {
        Some(path) => {
            fs::write(path, &content)?;
            println!("Results saved to: {}", path);
        }
        None => {
            println!("{}", content);
        }
    }

    Ok(())
}

async fn run_email_search(
    email: &str,
    timeout: u64,
    max_concurrent: usize,
    proxy: Option<&str>,
    tor: bool,
    rotate_ua: bool,
) -> Result<Vec<QueryResult>, Box<dyn std::error::Error>> {
    use crate::email::get_email_services;
    use crate::http::HttpClient;
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let services = get_email_services();
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    
    let mut http_client = HttpClient::new(timeout, rotate_ua)?;
    if tor {
        http_client = http_client.with_tor();
    } else if let Some(p) = proxy {
        http_client = http_client.with_proxy(p.to_string());
    }

    let mut handles = Vec::new();

    for (service_name, service_info) in services {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let email = email.to_string();
        let http_client = http_client.clone();

        let handle = tokio::spawn(async move {
            let url = service_info.url.replace("{}", &email);
            
            let start = std::time::Instant::now();
            let result = http_client.get(&url).await;
            let elapsed = start.elapsed().as_millis() as u64;

            match result {
                Ok(response) => {
                    let status = response.status();
                    let http_status = status.as_u16();

                    let claimed = match service_info.error_type {
                        email::EmailErrorType::StatusCode => status == reqwest::StatusCode::OK,
                        email::EmailErrorType::Message => {
                            if let Ok(text) = response.text().await {
                                if let Some(ref err_msg) = service_info.error_msg {
                                    !text.contains(err_msg)
                                } else {
                                    status == reqwest::StatusCode::OK
                                }
                            } else {
                                status == reqwest::StatusCode::OK
                            }
                        }
                    };

                    let query_result = if claimed {
                        QueryResult::claimed(&email, &service_name, &service_info.url_main, &url)
                    } else {
                        QueryResult::available(&email, &service_name, &service_info.url_main, &url)
                    };

                    Some(QueryResult {
                        http_status: Some(http_status),
                        response_time_ms: Some(elapsed),
                        ..query_result
                    })
                }
                Err(e) => {
                    Some(QueryResult::error(
                        &email,
                        &service_name,
                        &service_info.url_main,
                        &url,
                        &e.to_string(),
                    ))
                }
            }
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(Some(result)) = handle.await {
            results.push(result);
        }
    }

    Ok(results)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.timeout < 1 || cli.timeout > 300 {
        eprintln!("Error: Timeout must be between 1 and 300 seconds.");
        return Ok(());
    }

    if cli.max_concurrent < 1 || cli.max_concurrent > 100 {
        eprintln!("Error: Max concurrent must be between 1 and 100.");
        return Ok(());
    }

    if cli.tor && cli.proxy.is_some() {
        eprintln!("Error: Cannot use both --tor and --proxy at the same time.");
        return Ok(());
    }

    if cli.list_sites {
        let sites = load_sites_data(cli.local)?;
        println!("\n=== Supported Sites ({} total) ===\n", sites.len());
        for (name, info) in &sites {
            if !cli.nsfw && info.is_nsfw.unwrap_or(false) {
                continue;
            }
            println!("{} - {}", name, info.url_main);
        }
        return Ok(());
    }

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{} {}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .init();
    }

    if let Some(email) = cli.email {
        println!("\nSearching for email: {}", email);
        
        let results = run_email_search(
            &email,
            cli.timeout,
            cli.max_concurrent,
            cli.proxy.as_deref(),
            cli.tor,
            cli.rotate_ua,
        ).await?;

        let claimed_count = results.iter().filter(|r| r.is_claimed()).count();
        
        if cli.print_found {
            for result in &results {
                if result.is_claimed() {
                    println!("[+] {}: {}", result.site_name, result.profile_url);
                }
            }
        } else if cli.print_all {
            for result in &results {
                let status = if result.is_claimed() { "[+]" } else { "[-]" };
                println!("{} {}: {}", status, result.site_name, result.profile_url);
            }
        } else {
            for result in &results {
                if result.is_claimed() {
                    println!("[+] {}: {}", result.site_name, result.profile_url);
                }
            }
        }

        println!("\nFound {} results for {}", claimed_count, email);

        let report = SearchReport::new(email.clone(), results, cli.tor);
        
        if let Some(ref output) = cli.output {
            handle_output(&report, &cli.format, Some(output))?;
        }

        return Ok(());
    }

    let username = match &cli.username {
        Some(u) => u.clone(),
        None => {
            println!("Watson - OSINT Username & Email Lookup Tool");
            println!();
            println!("Usage:");
            println!("  watson -u <username>        Search for username");
            println!("  watson -m <email>          Search for email");
            println!();
            println!("Options:");
            println!("  -u, --username USERNAME    Username to search for");
            println!("  -m, --email EMAIL          Email to search for");
            println!("  -o, --output FILE          Output file path");
            println!("  -f, --format FORMAT        Output format (text, json, csv, html)");
            println!("  -p, --proxy URL            Proxy URL");
            println!("  -t, --tor                  Use Tor for requests");
            println!("  --timeout SECONDS          Request timeout (default: 60)");
            println!("  --max-concurrent N         Max concurrent requests (default: 20)");
            println!("  --nsfw                     Include NSFW sites");
            println!("  -a, --print-all            Print all results");
            println!("  -s, --print-found          Print only found results");
            println!("  -l, --local                Use local data file");
            println!("  -e, --site NAME            Search specific site");
            println!("  -v, --verbose              Verbose output");
            println!("  --list-sites               List supported sites");
            return Ok(());
        }
    };

    if cli.username.is_none() && cli.email.is_none() && cli.file.is_none() {
        eprintln!("Error: Please specify either --username, --file, or --email");
        eprintln!("Use watson --help for usage information");
        return Ok(());
    }

    if (cli.username.is_some() || cli.file.is_some()) && cli.email.is_some() {
        eprintln!("Error: Cannot use --username/--file and --email at the same time");
        return Ok(());
    }

    // Determine usernames to search
    let mut usernames_to_search: Vec<String> = vec![];

    if let Some(ref file_path) = cli.file {
        // Load usernames from file
        match fs::read_to_string(file_path) {
            Ok(content) => {
                let users: Vec<String> = content
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if users.is_empty() {
                    eprintln!("Error: No usernames found in file");
                    return Ok(());
                }
                usernames_to_search = users;
            }
            Err(e) => {
                eprintln!("Error: Could not read file: {}", e);
                return Ok(());
            }
        }
    } else if let Some(ref username) = cli.username {
        if cli.variations {
            println!("Generating username variations...");
            usernames_to_search = generate_variations(username);
        } else {
            usernames_to_search = vec![username.clone()];
        }
    }

    if let Some(ref username) = cli.username {
        if username.is_empty() || username.len() > 50 {
            eprintln!("Error: Username must be between 1 and 50 characters");
            return Ok(());
        }
    }

    if let Some(ref email) = cli.email {
        if !email.contains('@') {
            eprintln!("Error: Invalid email format");
            return Ok(());
        }
    }

    info!("Loading sites data...");
    let sites = load_sites_data(cli.local)?;
    
    let site_filter = cli.site.clone();
    let filtered_sites: HashMap<String, data::SiteInfo> = if let Some(ref filter) = site_filter {
        sites
            .into_iter()
            .filter(|(name, _)| filter.iter().any(|f| name.to_lowercase().contains(&f.to_lowercase())))
            .collect()
    } else {
        sites
    };

    info!("Found {} sites to check", filtered_sites.len());

    let mut engine = SearchEngine::new(cli.timeout, cli.max_concurrent, cli.nsfw, cli.rotate_ua)?;

    if let Some(rate_limit) = cli.rate_limit {
        if rate_limit > 0 {
            info!("Using rate limiting: {}ms between requests", rate_limit);
            engine = engine.with_rate_limit(rate_limit);
        }
    }

    if cli.tor {
        info!("Using Tor for requests");
        engine = engine.with_tor();
    } else if let Some(ref proxy) = cli.proxy {
        info!("Using proxy: {}", proxy);
        engine = engine.with_proxy(proxy.clone());
    }

    let tor_used = engine.is_using_tor();

    // Search for all usernames
    for username in usernames_to_search {
        println!("\nSearching for username: {}", username);
        
        let results = engine.search_username(&username, &filtered_sites).await;
        
        let report = SearchReport::new(username.clone(), results, tor_used);

    if cli.scrape_emails {
        let claimed_results: Vec<(String, String)> = report.results
            .iter()
            .filter(|r| r.is_claimed())
            .map(|r| (r.site_name.clone(), r.profile_url.clone()))
            .collect();

        if !claimed_results.is_empty() {
            println!("\nScraping profiles for emails...");
            let email_results = scrape_emails_from_results(claimed_results, cli.timeout, cli.rotate_ua).await;

            let mut emails_found = false;
            for (site_name, profile_url, emails) in email_results {
                if let Some(email_list) = emails {
                    if !email_list.is_empty() {
                        emails_found = true;
                        for email in email_list {
                            println!("[+] {}: {} -> Email: {}", site_name, profile_url, email);
                        }
                    }
                }
            }

            if !emails_found {
                println!("No emails found on profiles.");
            }
        }
    }
    
    if cli.print_found {
        for result in &report.results {
            if result.is_claimed() {
                println!("[+] {}: {}", result.site_name, result.profile_url);
            }
        }
    } else if cli.print_all {
        handle_output(&report, &cli.format, cli.output.as_deref())?;
    } else if cli.output.is_some() || cli.format != cli::OutputFormat::Text {
        handle_output(&report, &cli.format, cli.output.as_deref())?;
    } else {
        for result in &report.results {
            if result.is_claimed() {
                println!("[+] {}: {}", result.site_name, result.profile_url);
            }
        }
    }

    println!("\nFound {} results for {}", report.claimed_count, username);
    
    if let Some(ref output) = cli.output {
        handle_output(&report, &cli.format, Some(output))?;
    }
    }

    Ok(())
}
