use crate::Result;
use async_trait::async_trait;
use pretend::{
    client::{Bytes, Client, Method},
    Error, HeaderMap, Response, Result as PretendResult, Url,
};
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

#[inline]
fn default_reqwest_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .gzip(true)
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(120))
        .use_rustls_tls()
        .no_trust_dns()
        .https_only(true)
        .build()?)
}

#[derive(Clone, Debug)]
pub struct HttpClient(reqwest::Client);

impl HttpClient {
    #[inline]
    pub fn new(client: reqwest::Client) -> Self {
        Self(client)
    }

    #[inline]
    pub fn default_client() -> Result<Self> {
        Ok(Self(default_reqwest_client()?))
    }
}

#[async_trait]
impl Client for HttpClient {
    async fn execute(
        &self,
        method: Method,
        url: Url,
        headers: HeaderMap,
        body: Option<Bytes>,
    ) -> PretendResult<Response<Bytes>> {
        let mut builder = self.request(method, url).headers(headers);
        if let Some(body) = body {
            builder = builder.body(body);
        }
        let response = builder.send().await;
        let mut response = response.map_err(|err| Error::Response(Box::new(err)))?;

        let status = response.status();
        let headers = std::mem::take(response.headers_mut());

        let bytes = response.bytes().await;
        let bytes = bytes.map_err(|err| Error::Body(Box::new(err)))?;

        Ok(Response::new(status, headers, bytes))
    }
}

impl Default for HttpClient {
    #[inline]
    fn default() -> Self {
        Self::default_client().expect("failed to build default reqwest client")
    }
}

impl AsRef<reqwest::Client> for HttpClient {
    #[inline]
    fn as_ref(&self) -> &reqwest::Client {
        &self.0
    }
}

impl AsMut<reqwest::Client> for HttpClient {
    #[inline]
    fn as_mut(&mut self) -> &mut reqwest::Client {
        &mut self.0
    }
}

impl Deref for HttpClient {
    type Target = reqwest::Client;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HttpClient {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
