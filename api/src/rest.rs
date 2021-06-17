use crate::{client::ApiClient, response::*, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Rest: Sized {
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync;
}

#[async_trait]
impl Rest for Gift {
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        client.get_gift_list(client.live_id()).await
    }
}
