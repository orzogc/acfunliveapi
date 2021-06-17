use crate::{client::ApiClient, response::*, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Rest: Sized {
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync;
}

#[async_trait]
impl Rest for GiftList {
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        client.get_gift_list(client.live_id()).await
    }
}

#[async_trait]
impl Rest for LiveList {
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        client.get_live_list(1_000_000, 0).await
    }
}

#[async_trait]
impl Rest for MedalList {
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        client.get_medal_list(0).await
    }
}
