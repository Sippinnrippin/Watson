use crate::ua::UserAgentRotator;
use regex::Regex;
use reqwest::{Client, ClientBuilder};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct EmailScraper {
    client: Client,
    ua_rotator: Arc<RwLock<UserAgentRotator>>,
    rotate_ua: bool,
}

impl EmailScraper {
    pub fn new(timeout: u64, rotate_ua: bool) -> Result<Self, reqwest::Error> {
        let ua_rotator = Arc::new(RwLock::new(UserAgentRotator::new()));
        
        let default_ua = if rotate_ua {
            ua_rotator.blocking_read().get_random()
        } else {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
        };

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .connect_timeout(Duration::from_secs(5))
            .user_agent(&default_ua)
            .build()?;

        Ok(Self {
            client,
            ua_rotator,
            rotate_ua,
        })
    }

    async fn get_user_agent(&self) -> String {
        if self.rotate_ua {
            self.ua_rotator.read().await.get_random()
        } else {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
        }
    }

    pub async fn scrape_profile(&self, url: &str) -> Option<Vec<String>> {
        let ua = self.get_user_agent().await;
        
        let custom_client = ClientBuilder::new()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .user_agent(&ua)
            .build()
            .ok()?;

        let response = custom_client.get(url).send().await.ok()?;

        if !response.status().is_success() {
            return None;
        }

        let text = response.text().await.ok()?;
        let emails = self.extract_emails(&text);

        if emails.is_empty() {
            None
        } else {
            Some(emails)
        }
    }

    fn extract_emails(&self, text: &str) -> Vec<String> {
        let email_regex = match Regex::new(
            r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
        ) {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        let mut emails: Vec<String> = email_regex
            .find_iter(text)
            .map(|m| m.as_str().to_lowercase())
            .collect();

        emails.sort();
        emails.dedup();
        emails
    }
}

pub async fn scrape_emails_from_results(
    profile_urls: Vec<(String, String)>,
    timeout: u64,
    rotate_ua: bool,
) -> Vec<(String, String, Option<Vec<String>>)> {
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let scraper = match EmailScraper::new(timeout, rotate_ua) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let semaphore = Arc::new(Semaphore::new(10));
    let mut handles = vec![];

    for (site_name, profile_url) in profile_urls {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let scraper = EmailScraper::new(timeout, rotate_ua).ok();

        if scraper.is_none() {
            continue;
        }

        let scraper = scraper.unwrap();
        let handle = tokio::spawn(async move {
            let emails = scraper.scrape_profile(&profile_url).await;
            drop(permit);
            (site_name, profile_url, emails)
        });
        handles.push(handle);
    }

    let mut results = vec![];
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    results
}
