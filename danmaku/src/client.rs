use crate::{danmaku::*, global::*, proto::*, websocket::*, Error, Result};
use asynchronous_codec::Framed;
use futures::{ready, stream::FusedStream, SinkExt, Stream, StreamExt};
use std::{
    collections::VecDeque,
    convert::TryInto,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, SystemTime},
};

#[cfg(feature = "api")]
use acfunliveapi::{
    client::{ApiClient, ApiClientBuilder, ApiToken, Live},
    pretend,
};
#[cfg(feature = "api")]
use std::{borrow::Cow, convert::TryFrom};

#[cfg_attr(feature = "_serde", derive(::serde::Deserialize, ::serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DanmakuToken {
    pub user_id: i64,
    pub liver_uid: i64,
    pub security_key: String,
    pub service_token: String,
    pub live_id: String,
    pub enter_room_attach: String,
    pub tickets: Vec<String>,
}

impl DanmakuToken {
    #[inline]
    pub fn is_valid(&self) -> bool {
        !(self.user_id == 0
            || self.security_key.is_empty()
            || self.service_token.is_empty()
            || self.live_id.is_empty()
            || self.enter_room_attach.is_empty()
            || self.tickets.is_empty())
    }
}

#[cfg(feature = "api")]
impl DanmakuToken {
    pub async fn visitor(liver_uid: i64) -> Result<Self> {
        let client = ApiClientBuilder::default_client()?
            .liver_uid(liver_uid)
            .build()
            .await?;

        Ok(Self::from_token_live(
            client.token(),
            client.live().ok_or(acfunliveapi::Error::NotSetLiverUid)?,
        ))
    }

    pub async fn user<'a>(
        account: impl Into<Cow<'a, str>>,
        password: impl Into<Cow<'a, str>>,
        liver_uid: i64,
    ) -> Result<Self> {
        let client = ApiClientBuilder::default_client()?
            .user(account, password)
            .liver_uid(liver_uid)
            .build()
            .await?;

        Ok(Self::from_token_live(
            client.token(),
            client.live().ok_or(acfunliveapi::Error::NotSetLiverUid)?,
        ))
    }

    pub async fn from_api_client<C>(client: &ApiClient<C>, liver_uid: i64) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        let info = client.get_live_info(liver_uid).await?;
        let token = client.token();

        Ok(Self {
            user_id: token.user_id,
            liver_uid,
            security_key: token.security_key.clone(),
            service_token: token.service_token.clone(),
            live_id: info.data.live_id,
            enter_room_attach: info.data.enter_room_attach,
            tickets: info.data.available_tickets,
        })
    }

    #[inline]
    pub fn from_token_live(token: &ApiToken, live: &Live) -> Self {
        Self {
            user_id: token.user_id,
            liver_uid: live.liver_uid,
            security_key: token.security_key.clone(),
            service_token: token.service_token.clone(),
            live_id: live.live_id.clone(),
            enter_room_attach: live.enter_room_attach.clone(),
            tickets: live.tickets.clone(),
        }
    }
}

#[cfg(feature = "api")]
impl<C> TryFrom<ApiClient<C>> for DanmakuToken {
    type Error = Error;

    fn try_from(client: ApiClient<C>) -> Result<Self> {
        match client.live() {
            Some(live) => Ok(Self::from_token_live(client.token(), live)),
            None => Err(Error::NoLiveInfo),
        }
    }
}

#[cfg(feature = "api")]
impl<C> TryFrom<&ApiClient<C>> for DanmakuToken {
    type Error = Error;

    fn try_from(client: &ApiClient<C>) -> Result<Self> {
        match client.live() {
            Some(live) => Ok(Self::from_token_live(client.token(), live)),
            None => Err(Error::NoLiveInfo),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ClientState {
    BeforeRegister,
    Registering,
    Registered,
    Closing,
    Closed,
}

#[cfg(feature = "default_ws_client")]
pub type DefaultDanmakuClient = DanmakuClient<WebSocketClient>;

#[derive(Debug)]
pub struct DanmakuClient<W> {
    client: Framed<W, DanmakuProto>,
    state: ClientState,
    message: VecDeque<SendMessage>,
    interval: Option<Duration>,
    time: SystemTime,
    heartbeat_seq_id: i64,
}

impl<W: WebSocket> DanmakuClient<W> {
    #[inline]
    pub async fn new(token: DanmakuToken) -> std::result::Result<Self, W::Error> {
        if token.is_valid() {
            match W::connect(DANMAKU_SERVER).await {
                Ok(client) => Ok(Self {
                    client: Framed::new(client, token.try_into()?),
                    state: ClientState::BeforeRegister,
                    message: VecDeque::new(),
                    interval: None,
                    time: SystemTime::now(),
                    heartbeat_seq_id: 0,
                }),
                Err(e) => Err(e),
            }
        } else {
            Err(Error::InvalidToken.into())
        }
    }

    #[inline]
    pub fn user_id(&self) -> i64 {
        self.client.codec().user_id
    }

    #[inline]
    pub fn liver_uid(&self) -> i64 {
        self.client.codec().liver_uid
    }

    #[inline]
    pub fn live_id(&self) -> &str {
        &self.client.codec().live_id
    }

    #[inline]
    pub async fn close(&mut self) -> Result<()> {
        self.client.close().await
    }
}

#[cfg(feature = "default_ws_client")]
impl DanmakuClient<WebSocketClient> {
    #[inline]
    pub async fn default_client(token: DanmakuToken) -> Result<Self> {
        Self::new(token).await
    }

    #[cfg(feature = "api")]
    #[inline]
    pub async fn visitor(liver_uid: i64) -> Result<Self> {
        Self::default_client(DanmakuToken::visitor(liver_uid).await?).await
    }

    #[cfg(feature = "api")]
    #[inline]
    pub async fn user<'a>(
        account: impl Into<Cow<'a, str>>,
        password: impl Into<Cow<'a, str>>,
        liver_uid: i64,
    ) -> Result<Self> {
        Self::default_client(DanmakuToken::user(account, password, liver_uid).await?).await
    }

    #[cfg(feature = "api")]
    #[inline]
    pub async fn from_api_client<C>(client: &ApiClient<C>, liver_uid: i64) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        Self::default_client(DanmakuToken::from_api_client(client, liver_uid).await?).await
    }
}

impl<W: WebSocket> Stream for DanmakuClient<W> {
    type Item = std::result::Result<Danmaku, W::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.state {
                ClientState::BeforeRegister => {
                    ready!(self.client.poll_ready_unpin(cx))?;
                    self.client.start_send_unpin(SendMessage::RegisterRequest)?;
                    self.state = ClientState::Registering;
                }
                ClientState::Registering => {
                    ready!(self.client.poll_flush_unpin(cx))?;
                    let msg = if let Some(result) = ready!(self.client.poll_next_unpin(cx)) {
                        result?
                    } else {
                        self.state = ClientState::Closed;
                        return Poll::Ready(None);
                    };
                    if msg == ReceiveMessage::RegisterResponse {
                        self.message.push_back(SendMessage::KeepAliveRequest);
                        self.message.push_back(SendMessage::ZtLiveCsEnterRoom);
                        self.state = ClientState::Registered;
                    } else {
                        return Poll::Ready(Some(Err(Error::RegisterError.into())));
                    }
                }
                ClientState::Registered => {
                    if let Some(interval) = self.interval {
                        let now = SystemTime::now();
                        match now.duration_since(self.time) {
                            Ok(t) => {
                                if t >= interval {
                                    self.message.push_back(SendMessage::ZtLiveCsHeartbeat);
                                    self.heartbeat_seq_id += 1;
                                    if self.heartbeat_seq_id % 5 == 4 {
                                        self.message.push_back(SendMessage::KeepAliveRequest);
                                    }
                                    self.time = now;
                                }
                            }
                            Err(e) => {
                                log::trace!("failed to get the interval from SystemTime: {}", e)
                            }
                        }
                    }
                    while !self.message.is_empty() {
                        ready!(self.client.poll_ready_unpin(cx))?;
                        let msg = self
                            .message
                            .pop_front()
                            .expect("the message VecDeque is empty");
                        self.client.start_send_unpin(msg)?;
                    }
                    ready!(self.client.poll_flush_unpin(cx))?;
                    let msg = if let Some(result) = ready!(self.client.poll_next_unpin(cx)) {
                        result?
                    } else {
                        self.state = ClientState::Closed;
                        return Poll::Ready(None);
                    };
                    match msg {
                        ReceiveMessage::Danmaku(danmaku) => return Poll::Ready(Some(Ok(danmaku))),
                        ReceiveMessage::RegisterResponse => {
                            log::trace!("registered more than once");
                        }
                        ReceiveMessage::Interval(interval) => {
                            self.interval = Some(Duration::from_millis(interval));
                        }
                        ReceiveMessage::PushMessage => {
                            self.message.push_back(SendMessage::ZtLiveScMessage);
                        }
                        ReceiveMessage::EnterRoom => {
                            self.message.push_back(SendMessage::ZtLiveScMessage);
                            self.message.push_back(SendMessage::ZtLiveCsEnterRoom);
                        }
                        ReceiveMessage::PushAndStop => {
                            self.message.push_back(SendMessage::ZtLiveScMessage);
                            self.message.push_back(SendMessage::ZtLiveCsUserExit);
                            self.message.push_back(SendMessage::UnregisterRequest);
                            self.state = ClientState::Closing;
                        }
                        ReceiveMessage::Stop => {
                            self.message.push_back(SendMessage::ZtLiveCsUserExit);
                            self.message.push_back(SendMessage::UnregisterRequest);
                            self.state = ClientState::Closing;
                        }
                        ReceiveMessage::Close => {
                            self.state = ClientState::Closing;
                        }
                    }
                }
                ClientState::Closing => {
                    while !self.message.is_empty() {
                        ready!(self.client.poll_ready_unpin(cx))?;
                        let msg = self
                            .message
                            .pop_front()
                            .expect("the message VecDeque is empty");
                        self.client.start_send_unpin(msg)?;
                    }
                    ready!(self.client.poll_close_unpin(cx))?;
                    self.state = ClientState::Closed;
                    return Poll::Ready(None);
                }
                ClientState::Closed => return Poll::Ready(None),
            }
        }
    }
}

impl<W: WebSocket> FusedStream for DanmakuClient<W> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.state == ClientState::Closed
    }
}

#[cfg(all(feature = "api", feature = "default_ws_client"))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::{select, time::sleep};

    #[tokio::test]
    async fn test_danmaku() -> Result<()> {
        let liver_uid: i64 = env::var("LIVER_UID")
            .expect("need to set the LIVER_UID environment variable to the liver's uid")
            .parse()
            .expect("LIVER_UID should be an integer");
        let mut client = DanmakuClient::visitor(liver_uid).await?;
        let danmaku = async {
            while let Some(result) = client.next().await {
                match result {
                    Ok(damaku) => println!("{:?}", damaku),
                    Err(e) => println!("error: {}", e),
                }
            }
        };
        select! {
            _ = danmaku => {}
            _ = sleep(Duration::from_secs(60)) => {}
        }

        Ok(())
    }
}
