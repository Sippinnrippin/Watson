use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailService {
    pub url: String,
    #[serde(rename = "urlMain")]
    pub url_main: String,
    #[serde(rename = "errorType")]
    pub error_type: EmailErrorType,
    #[serde(rename = "errorMsg", skip_serializing_if = "Option::is_none")]
    pub error_msg: Option<String>,
    #[serde(rename = "requestMethod", skip_serializing_if = "Option::is_none")]
    pub request_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailErrorType {
    #[serde(rename = "status_code")]
    StatusCode,
    #[serde(rename = "message")]
    Message,
}

pub fn get_email_services() -> HashMap<String, EmailService> {
    let mut services = HashMap::new();

    services.insert(
        "Gravatar".to_string(),
        EmailService {
            url: "https://en.gravatar.com/{}.json".to_string(),
            url_main: "https://gravatar.com".to_string(),
            error_type: EmailErrorType::Message,
            error_msg: Some("User not found".to_string()),
            request_method: None,
        },
    );

    services.insert(
        "DuckDuckGo".to_string(),
        EmailService {
            url: "https://duckduckgo.com/?q={}&format=json".to_string(),
            url_main: "https://duckduckgo.com".to_string(),
            error_type: EmailErrorType::StatusCode,
            error_msg: None,
            request_method: None,
        },
    );

    services.insert(
        "HaveIBeenPwned".to_string(),
        EmailService {
            url: "https://haveibeenpwned.com/api/v3/breachedaccount/{}".to_string(),
            url_main: "https://haveibeenpwned.com".to_string(),
            error_type: EmailErrorType::StatusCode,
            error_msg: None,
            request_method: None,
        },
    );

    services
}
