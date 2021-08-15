use crate::Result;
use std::time::Duration;

pub use pretend_reqwest::Client as HttpClient;

const TIMEOUT: Duration = Duration::from_secs(10);
const IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const KEEPALIVE: Duration = Duration::from_secs(120);

#[inline]
fn default_reqwest_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .gzip(true)
        .timeout(TIMEOUT)
        .pool_idle_timeout(IDLE_TIMEOUT)
        .tcp_keepalive(KEEPALIVE)
        .use_rustls_tls()
        .no_trust_dns()
        .https_only(true)
        .build()?)
}

pub fn new_http_client() -> Result<HttpClient> {
    Ok(HttpClient::new(default_reqwest_client()?))
}
