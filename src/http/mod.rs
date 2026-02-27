use reqwest::{Client, ClientBuilder, Proxy};
use std::time::Duration;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    proxy: Option<String>,
    use_tor: bool,
}

impl HttpClient {
    pub fn new(timeout: u64) -> Result<Self, reqwest::Error> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(false)
            .build()?;

        Ok(Self {
            client,
            proxy: None,
            use_tor: false,
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

    fn build_proxy_client(&self) -> Option<Client> {
        if let Some(ref proxy_url) = self.proxy {
            if let Ok(proxy) = Proxy::all(proxy_url) {
                if let Ok(client) = ClientBuilder::new()
                    .proxy(proxy)
                    .timeout(Duration::from_secs(15))
                    .connect_timeout(Duration::from_secs(5))
                    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                    .danger_accept_invalid_certs(false)
                    .build()
                {
                    return Some(client);
                }
            }
        }
        None
    }

    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        if let Some(client) = self.build_proxy_client() {
            return client.get(url).send().await;
        }
        self.client.get(url).send().await
    }

    pub async fn head(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        if let Some(client) = self.build_proxy_client() {
            return client.head(url).send().await;
        }
        self.client.head(url).send().await
    }

    pub async fn post(&self, url: &str, body: Option<String>) -> Result<reqwest::Response, reqwest::Error> {
        if let Some(client) = self.build_proxy_client() {
            let mut req = client.post(url);
            if let Some(b) = body {
                req = req.body(b);
            }
            return req.send().await;
        }
        match body {
            Some(b) => self.client.post(url).body(b).send().await,
            None => self.client.post(url).send().await,
        }
    }

    pub async fn put(&self, url: &str, body: Option<String>) -> Result<reqwest::Response, reqwest::Error> {
        if let Some(client) = self.build_proxy_client() {
            let mut req = client.put(url);
            if let Some(b) = body {
                req = req.body(b);
            }
            return req.send().await;
        }
        match body {
            Some(b) => self.client.put(url).body(b).send().await,
            None => self.client.put(url).send().await,
        }
    }

    pub fn is_using_tor(&self) -> bool {
        self.use_tor
    }

    pub fn has_proxy(&self) -> bool {
        self.proxy.is_some()
    }
}
