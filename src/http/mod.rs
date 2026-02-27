use crate::ua::UserAgentRotator;
use reqwest::{Client, ClientBuilder, Proxy};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    proxy: Option<String>,
    use_tor: bool,
    ua_rotator: Arc<RwLock<UserAgentRotator>>,
    rotate_ua: bool,
}

impl HttpClient {
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
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .user_agent(&default_ua)
            .danger_accept_invalid_certs(false)
            .build()?;

        Ok(Self {
            client,
            proxy: None,
            use_tor: false,
            ua_rotator,
            rotate_ua,
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

    async fn get_user_agent(&self) -> String {
        if self.rotate_ua {
            self.ua_rotator.read().await.get_random()
        } else {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
        }
    }

    fn build_proxy_client_with_ua(&self, ua: &str) -> Option<Client> {
        if let Some(ref proxy_url) = self.proxy {
            if let Ok(proxy) = Proxy::all(proxy_url) {
                if let Ok(client) = ClientBuilder::new()
                    .proxy(proxy)
                    .timeout(Duration::from_secs(15))
                    .connect_timeout(Duration::from_secs(5))
                    .user_agent(ua)
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
        let ua = self.get_user_agent().await;
        
        if let Some(client) = self.build_proxy_client_with_ua(&ua) {
            return client.get(url).send().await;
        }
        
        // Use default client with custom UA
        let custom_client = ClientBuilder::new()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .user_agent(&ua)
            .danger_accept_invalid_certs(false)
            .build()?;
        
        custom_client.get(url).send().await
    }

    pub async fn head(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let ua = self.get_user_agent().await;
        
        if let Some(client) = self.build_proxy_client_with_ua(&ua) {
            return client.head(url).send().await;
        }
        
        let custom_client = ClientBuilder::new()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .user_agent(&ua)
            .danger_accept_invalid_certs(false)
            .build()?;
        
        custom_client.head(url).send().await
    }

    pub async fn post(&self, url: &str, body: Option<String>) -> Result<reqwest::Response, reqwest::Error> {
        let ua = self.get_user_agent().await;
        
        if let Some(client) = self.build_proxy_client_with_ua(&ua) {
            let mut req = client.post(url);
            if let Some(b) = body {
                req = req.body(b);
            }
            return req.send().await;
        }
        
        let custom_client = ClientBuilder::new()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .user_agent(&ua)
            .danger_accept_invalid_certs(false)
            .build()?;
        
        match body {
            Some(b) => custom_client.post(url).body(b).send().await,
            None => custom_client.post(url).send().await,
        }
    }

    pub async fn put(&self, url: &str, body: Option<String>) -> Result<reqwest::Response, reqwest::Error> {
        let ua = self.get_user_agent().await;
        
        if let Some(client) = self.build_proxy_client_with_ua(&ua) {
            let mut req = client.put(url);
            if let Some(b) = body {
                req = req.body(b);
            }
            return req.send().await;
        }
        
        let custom_client = ClientBuilder::new()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .user_agent(&ua)
            .danger_accept_invalid_certs(false)
            .build()?;
        
        match body {
            Some(b) => custom_client.put(url).body(b).send().await,
            None => custom_client.put(url).send().await,
        }
    }

    pub fn is_using_tor(&self) -> bool {
        self.use_tor
    }

    pub fn has_proxy(&self) -> bool {
        self.proxy.is_some()
    }
}
