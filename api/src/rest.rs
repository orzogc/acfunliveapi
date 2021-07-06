use crate::{client::ApiClient, response::*, Error, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Rest: Sized {
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync;
}

#[async_trait]
impl Rest for GiftList {
    #[inline]
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        match client.live_id() {
            Some(live_id) => client.get_gift_list(live_id).await,
            None => Err(Error::NotSetLiverUid),
        }
    }
}

#[async_trait]
impl Rest for LiveList {
    #[inline]
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        client.get_live_list(1_000_000, 0).await
    }
}

#[async_trait]
impl Rest for MedalList {
    #[inline]
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        client.get_medal_list().await
    }
}

#[async_trait]
impl Rest for UserLiveInfo {
    #[inline]
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        match client.liver_uid() {
            Some(liver_uid) => client.get_user_live_info(liver_uid).await,
            None => Err(Error::NotSetLiverUid),
        }
    }
}

#[async_trait]
impl Rest for Summary {
    #[inline]
    async fn request<C>(client: &ApiClient<C>) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        match client.live_id() {
            Some(live_id) => client.get_summary(live_id).await,
            None => Err(Error::NotSetLiverUid),
        }
    }
}
