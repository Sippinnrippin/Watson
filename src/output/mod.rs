use crate::engine::QueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
    Html,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchReport {
    pub username: String,
    pub total_sites: usize,
    pub claimed_count: usize,
    pub available_count: usize,
    pub error_count: usize,
    pub results: Vec<QueryResult>,
    pub tor_used: bool,
}

impl SearchReport {
    pub fn new(username: String, results: Vec<QueryResult>, tor_used: bool) -> Self {
        let claimed_count = results.iter().filter(|r| r.is_claimed()).count();
        let available_count = results
            .iter()
            .filter(|r| r.status == crate::engine::QueryStatus::Available)
            .count();
        let error_count = results
            .iter()
            .filter(|r| r.status == crate::engine::QueryStatus::Error)
            .count();

        Self {
            username,
            total_sites: results.len(),
            claimed_count,
            available_count,
            error_count,
            results,
            tor_used,
        }
    }

    pub fn to_text(&self) -> String {
        let mut output = format!("\n=== Watson Search Results for '{}' ===\n", self.username);
        output.push_str(&format!("Total sites checked: {}\n", self.total_sites));
        output.push_str(&format!("Found on: {} sites\n", self.claimed_count));
        output.push_str(&format!("Available on: {} sites\n", self.available_count));
        output.push_str(&format!("Errors: {}\n", self.error_count));

        if self.tor_used {
            output.push_str("Using Tor: Yes\n");
        }

        output.push_str("\n--- Found Accounts ---\n");

        for result in &self.results {
            if result.is_claimed() {
                output.push_str(&format!(
                    "[+] {}: {}\n",
                    result.site_name, result.profile_url
                ));
            }
        }

        output.push_str("\n--- Available Accounts ---\n");

        for result in &self.results {
            if result.status == crate::engine::QueryStatus::Available {
                output.push_str(&format!(
                    "[-] {}: {}\n",
                    result.site_name, result.profile_url
                ));
            }
        }

        output
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn to_csv(&self) -> String {
        let mut output =
            String::from("site_name,site_url,profile_url,status,http_status,response_time_ms\n");

        for result in &self.results {
            output.push_str(&format!(
                "{},{},{},{},{},{}\n",
                result.site_name,
                result.site_url,
                result.profile_url,
                format!("{:?}", result.status).to_lowercase(),
                result
                    .http_status
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                result
                    .response_time_ms
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            ));
        }

        output
    }

    pub fn to_html(&self) -> String {
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Watson Search Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        .container { max-width: 900px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        h1 { color: #333; border-bottom: 2px solid #007bff; padding-bottom: 10px; }
        .stats { display: flex; gap: 20px; margin: 20px 0; }
        .stat { padding: 15px 25px; background: #f8f9fa; border-radius: 5px; text-align: center; }
        .stat-value { font-size: 24px; font-weight: bold; color: #007bff; }
        .stat-label { color: #666; font-size: 12px; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background: #007bff; color: white; }
        .claimed { color: #28a745; font-weight: bold; }
        .available { color: #dc3545; }
        .error { color: #ffc107; }
        .site-link { color: #007bff; text-decoration: none; }
        .site-link:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Watson Search Report</h1>
        <p><strong>Username:</strong> #USERNAME#</p>
        <div class="stats">
            <div class="stat">
                <div class="stat-value">#TOTAL#</div>
                <div class="stat-label">Total Sites</div>
            </div>
            <div class="stat">
                <div class="stat-value">#CLAIMED#</div>
                <div class="stat-label">Found</div>
            </div>
            <div class="stat">
                <div class="stat-value">#AVAILABLE#</div>
                <div class="stat-label">Available</div>
            </div>
            <div class="stat">
                <div class="stat-value">#ERRORS#</div>
                <div class="stat-label">Errors</div>
            </div>
        </div>
        <table>
            <thead>
                <tr>
                    <th>Site</th>
                    <th>Profile URL</th>
                    <th>Status</th>
                    <th>HTTP Status</th>
                    <th>Response Time</th>
                </tr>
            </thead>
            <tbody>
"#,
        );

        for result in &self.results {
            let status_class = match result.status {
                crate::engine::QueryStatus::Claimed => "claimed",
                crate::engine::QueryStatus::Available => "available",
                crate::engine::QueryStatus::Error => "error",
                _ => "",
            };

            let status_text = format!("{:?}", result.status);

            html.push_str(&format!(
                r#"                <tr>
                    <td>{}</td>
                    <td><a href="{}" class="site-link" target="_blank">{}</a></td>
                    <td class="{}">{}</td>
                    <td>{}</td>
                    <td>{} ms</td>
                </tr>
"#,
                result.site_name,
                result.profile_url,
                result.profile_url,
                status_class,
                status_text,
                result
                    .http_status
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
                result
                    .response_time_ms
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
            ));
        }

        html.push_str(
            r#"            </tbody>
        </table>
    </div>
</body>
</html>
"#,
        );

        html = html.replace("#USERNAME#", &self.username);
        html = html.replace("#TOTAL#", &self.total_sites.to_string());
        html = html.replace("#CLAIMED#", &self.claimed_count.to_string());
        html = html.replace("#AVAILABLE#", &self.available_count.to_string());
        html = html.replace("#ERRORS#", &self.error_count.to_string());

        html
    }
}
