use reqwest::{Client, ClientBuilder, Proxy};
use std::time::Duration;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    proxy: Option<String>,
    use_tor: bool,
    timeout: u64,
}

impl HttpClient {
    pub fn new(timeout: u64) -> Result<Self, reqwest::Error> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(false)
            .build()?;

        Ok(Self {
            client,
            proxy: None,
            use_tor: false,
            timeout,
        })
    }

    pub fn with_proxy(mut self, proxy: String) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_tor(mut self) -> Self {
        self.use_tor = true;
        self.proxy = Some("socks5://127.0.0.1:9050".to_string());
        self
    }

    pub fn build_with_config(&self) -> Result<Client, reqwest::Error> {
        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(self.timeout))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(false);

        if let Some(ref proxy) = self.proxy {
            if let Ok(proxy) = Proxy::all(proxy) {
                builder = builder.proxy(proxy);
            }
        }

        builder.build()
    }

    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let client = self.build_with_config()?;
        client.get(url).send().await
    }

    pub async fn head(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let client = self.build_with_config()?;
        client.head(url).send().await
    }

    pub async fn post(&self, url: &str, body: Option<String>) -> Result<reqwest::Response, reqwest::Error> {
        let client = self.build_with_config()?;
        match body {
            Some(b) => client.post(url).body(b).send().await,
            None => client.post(url).send().await,
        }
    }

    pub async fn put(&self, url: &str, body: Option<String>) -> Result<reqwest::Response, reqwest::Error> {
        let client = self.build_with_config()?;
        match body {
            Some(b) => client.put(url).body(b).send().await,
            None => client.put(url).send().await,
        }
    }

    pub fn is_using_tor(&self) -> bool {
        self.use_tor
    }

    pub fn has_proxy(&self) -> bool {
        self.proxy.is_some()
    }
}
