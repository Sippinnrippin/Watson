use crate::data::{ErrorMessages, ErrorType, SiteInfo};
use crate::http::HttpClient;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryStatus {
    #[serde(rename = "claimed")]
    Claimed,
    #[serde(rename = "available")]
    Available,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "illegal")]
    Illegal,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub username: String,
    pub site_name: String,
    pub site_url: String,
    pub profile_url: String,
    pub status: QueryStatus,
    pub http_status: Option<u16>,
    pub error_message: Option<String>,
    pub response_time_ms: Option<u64>,
}

impl QueryResult {
    pub fn new(
        username: &str,
        site_name: &str,
        site_url: &str,
        profile_url: &str,
        status: QueryStatus,
    ) -> Self {
        Self {
            username: username.to_string(),
            site_name: site_name.to_string(),
            site_url: site_url.to_string(),
            profile_url: profile_url.to_string(),
            status,
            http_status: None,
            error_message: None,
            response_time_ms: None,
        }
    }

    pub fn claimed(username: &str, site_name: &str, site_url: &str, profile_url: &str) -> Self {
        Self::new(username, site_name, site_url, profile_url, QueryStatus::Claimed)
    }

    pub fn available(username: &str, site_name: &str, site_url: &str, profile_url: &str) -> Self {
        Self::new(username, site_name, site_url, profile_url, QueryStatus::Available)
    }

    pub fn error(username: &str, site_name: &str, site_url: &str, profile_url: &str, error: &str) -> Self {
        let mut result = Self::new(username, site_name, site_url, profile_url, QueryStatus::Error);
        result.error_message = Some(error.to_string());
        result
    }

    pub fn illegal(username: &str, site_name: &str, site_url: &str) -> Self {
        Self::new(username, site_name, site_url, "", QueryStatus::Illegal)
    }

    pub fn is_claimed(&self) -> bool {
        self.status == QueryStatus::Claimed
    }
}

pub struct SearchEngine {
    http_client: HttpClient,
    timeout: u64,
    max_concurrent: usize,
    include_nsfw: bool,
}

impl SearchEngine {
    pub fn new(timeout: u64, max_concurrent: usize, include_nsfw: bool) -> Result<Self, reqwest::Error> {
        Ok(Self {
            http_client: HttpClient::new(timeout)?,
            timeout,
            max_concurrent,
            include_nsfw,
        })
    }

    pub fn with_proxy(mut self, proxy: String) -> Self {
        self.http_client = self.http_client.with_proxy(proxy);
        self
    }

    pub fn with_tor(mut self) -> Self {
        self.http_client = self.http_client.with_tor();
        self
    }

    pub async fn search_username(
        &self,
        username: &str,
        sites: &HashMap<String, SiteInfo>,
    ) -> Vec<QueryResult> {
        use tokio::sync::Semaphore;
        use std::sync::Arc;

        let sites_to_check: Vec<(String, SiteInfo)> = sites
            .iter()
            .filter(|(_, info)| self.include_nsfw || !info.is_nsfw.unwrap_or(false))
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect();

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut handles = Vec::new();

        for (site_name, site_info) in sites_to_check {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let username = username.to_string();
            let http_client = self.http_client.clone();
            let timeout = self.timeout;

            let handle = tokio::spawn(async move {
                let result = check_site_internal(&http_client, &username, &site_name, &site_info, timeout).await;
                drop(permit);
                result
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(Some(result)) = handle.await {
                results.push(result);
            }
        }

        results
    }

    async fn check_site(
        &self,
        username: &str,
        site_name: &str,
        site_info: &SiteInfo,
    ) -> Option<QueryResult> {
        if let Some(ref regex) = site_info.regex_check {
            if let Ok(re) = Regex::new(regex) {
                if !re.is_match(username) {
                    return Some(QueryResult::illegal(username, site_name, &site_info.url_main));
                }
            }
        }

        let profile_url = site_info.url.replace("{}", username);
        let probe_url = site_info.url_probe.as_ref().unwrap_or(&profile_url).replace("{}", username);

        let start = std::time::Instant::now();

        let result = match site_info.request_method.as_deref() {
            Some("POST") => {
                let body = site_info.request_payload.as_ref().map(|p| p.to_string());
                self.http_client.post(&probe_url, body).await
            }
            Some("PUT") => {
                let body = site_info.request_payload.as_ref().map(|p| p.to_string());
                self.http_client.put(&probe_url, body).await
            }
            Some("HEAD") | None => {
                if site_info.error_type == ErrorType::StatusCode {
                    self.http_client.head(&probe_url).await
                } else {
                    self.http_client.get(&probe_url).await
                }
            }
            _ => self.http_client.get(&probe_url).await,
        };

        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => {
                let status = response.status();
                let http_status = status.as_u16();

                let detected = match site_info.error_type {
                    ErrorType::StatusCode => {
                        status == reqwest::StatusCode::OK
                    }
                    ErrorType::Message => {
                        if let Ok(text) = response.text().await {
                            if let Some(ref error_msgs) = site_info.error_msg {
                                let msg_list: Vec<&str> = match error_msgs {
                                    ErrorMessages::Single(s) => vec![s.as_str()],
                                    ErrorMessages::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
                                };
                                !msg_list.iter().any(|msg| text.contains(msg))
                            } else {
                                status == reqwest::StatusCode::OK
                            }
                        } else {
                            status == reqwest::StatusCode::OK
                        }
                    }
                    ErrorType::Redirect => {
                        status != reqwest::StatusCode::NOT_FOUND
                            && status != reqwest::StatusCode::FORBIDDEN
                    }
                    ErrorType::ResponseUrl => {
                        if let Some(ref error_url) = site_info.error_url {
                            let resp_url = response.url();
                            resp_url.to_string() != *error_url
                        } else {
                            status == reqwest::StatusCode::OK
                        }
                    }
                };

                let query_result = if detected {
                    QueryResult::claimed(username, site_name, &site_info.url_main, &profile_url)
                } else {
                    QueryResult::available(username, site_name, &site_info.url_main, &profile_url)
                };

                Some(QueryResult {
                    http_status: Some(http_status),
                    response_time_ms: Some(elapsed),
                    ..query_result
                })
            }
            Err(e) => {
                Some(QueryResult::error(
                    username,
                    site_name,
                    &site_info.url_main,
                    &profile_url,
                    &e.to_string(),
                ))
            }
        }
    }

    pub fn is_using_tor(&self) -> bool {
        self.http_client.is_using_tor()
    }
}

async fn check_site_internal(
    http_client: &HttpClient,
    username: &str,
    site_name: &str,
    site_info: &SiteInfo,
    timeout: u64,
) -> Option<QueryResult> {
    if let Some(ref regex) = site_info.regex_check {
        if let Ok(re) = Regex::new(regex) {
            if !re.is_match(username) {
                return Some(QueryResult::illegal(username, site_name, &site_info.url_main));
            }
        }
    }

    let profile_url = site_info.url.replace("{}", username);
    let probe_url = site_info.url_probe.as_ref().unwrap_or(&profile_url).replace("{}", username);

    let start = std::time::Instant::now();

    let result = match site_info.request_method.as_deref() {
        Some("POST") => {
            let body = site_info.request_payload.as_ref().map(|p| p.to_string());
            http_client.post(&probe_url, body).await
        }
        Some("PUT") => {
            let body = site_info.request_payload.as_ref().map(|p| p.to_string());
            http_client.put(&probe_url, body).await
        }
        Some("HEAD") | None => {
            if site_info.error_type == ErrorType::StatusCode {
                http_client.head(&probe_url).await
            } else {
                http_client.get(&probe_url).await
            }
        }
        _ => http_client.get(&probe_url).await,
    };

    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(response) => {
            let status = response.status();
            let http_status = status.as_u16();

            let detected = match site_info.error_type {
                ErrorType::StatusCode => {
                    status == reqwest::StatusCode::OK
                }
                ErrorType::Message => {
                    if let Ok(text) = response.text().await {
                        if let Some(ref error_msgs) = site_info.error_msg {
                            let msg_list: Vec<&str> = match error_msgs {
                                ErrorMessages::Single(s) => vec![s.as_str()],
                                ErrorMessages::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
                            };
                            !msg_list.iter().any(|msg| text.contains(msg))
                        } else {
                            status == reqwest::StatusCode::OK
                        }
                    } else {
                        status == reqwest::StatusCode::OK
                    }
                }
                ErrorType::Redirect => {
                    status != reqwest::StatusCode::NOT_FOUND
                        && status != reqwest::StatusCode::FORBIDDEN
                }
                ErrorType::ResponseUrl => {
                    if let Some(ref error_url) = site_info.error_url {
                        let resp_url = response.url();
                        resp_url.to_string() != *error_url
                    } else {
                        status == reqwest::StatusCode::OK
                    }
                }
            };

            let query_result = if detected {
                QueryResult::claimed(username, site_name, &site_info.url_main, &profile_url)
            } else {
                QueryResult::available(username, site_name, &site_info.url_main, &profile_url)
            };

            Some(QueryResult {
                http_status: Some(http_status),
                response_time_ms: Some(elapsed),
                ..query_result
            })
        }
        Err(e) => {
            Some(QueryResult::error(
                username,
                site_name,
                &site_info.url_main,
                &profile_url,
                &e.to_string(),
            ))
        }
    }
}
