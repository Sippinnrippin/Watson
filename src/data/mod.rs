use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorType {
    #[serde(rename = "status_code")]
    StatusCode,
    #[serde(rename = "message")]
    Message,
    #[serde(rename = "redirect")]
    Redirect,
    #[serde(rename = "response_url")]
    ResponseUrl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorMessages {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteInfo {
    pub url: String,
    #[serde(rename = "urlMain")]
    pub url_main: String,
    #[serde(rename = "errorType")]
    pub error_type: ErrorType,
    #[serde(rename = "errorMsg", skip_serializing_if = "Option::is_none")]
    pub error_msg: Option<ErrorMessages>,
    #[serde(rename = "errorUrl", skip_serializing_if = "Option::is_none")]
    pub error_url: Option<String>,
    #[serde(rename = "regexCheck", skip_serializing_if = "Option::is_none")]
    pub regex_check: Option<String>,
    #[serde(rename = "username_claimed", skip_serializing_if = "Option::is_none")]
    pub username_claimed: Option<String>,
    #[serde(rename = "request_method", skip_serializing_if = "Option::is_none")]
    pub request_method: Option<String>,
    #[serde(rename = "request_payload", skip_serializing_if = "Option::is_none")]
    pub request_payload: Option<serde_json::Value>,
    #[serde(rename = "urlProbe", skip_serializing_if = "Option::is_none")]
    pub url_probe: Option<String>,
    #[serde(rename = "headers", skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "isNSFW", skip_serializing_if = "Option::is_none")]
    pub is_nsfw: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitesData {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    #[serde(flatten)]
    pub sites: std::collections::HashMap<String, SiteInfo>,
}

impl SitesData {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let data: SitesData = serde_json::from_str(&content)?;
        Ok(data)
    }

    pub fn load_from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data: SitesData = serde_json::from_str(json)?;
        Ok(data)
    }
}
