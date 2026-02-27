use regex::Regex;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub struct EmailScraper {
    client: Client,
}

impl EmailScraper {
    pub fn new(timeout: u64) -> Result<Self, reqwest::Error> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()?;

        Ok(Self { client })
    }

    pub async fn scrape_profile(&self, url: &str) -> Option<Vec<String>> {
        let response = self.client.get(url).send().await.ok()?;

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
) -> Vec<(String, String, Option<Vec<String>>)> {
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let scraper = match EmailScraper::new(timeout) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let semaphore = Arc::new(Semaphore::new(10));
    let mut handles = vec![];

    for (site_name, profile_url) in profile_urls {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let scraper = EmailScraper::new(timeout).ok();

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
