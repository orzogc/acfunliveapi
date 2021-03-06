use crate::{danmaku::*, global::*, proto::*, websocket::*, Error, Result};
use async_compression::futures::bufread::GzipDecoder;
use async_timer::Interval;
use futures::{
    channel::{mpsc, oneshot},
    io::AsyncReadExt,
    join,
    {sink::SinkExt, stream::StreamExt},
};
use prost::Message;
use std::{convert::TryFrom, time::Duration};

#[cfg(feature = "api")]
use acfunliveapi::{
    client::{ApiClient, ApiClientBuilder, ApiToken, Live},
    pretend,
};
#[cfg(feature = "api")]
use std::borrow::Cow;

const CHANNEL_SIZE: usize = 100;

#[derive(Clone, Debug)]
enum Command {
    Decode(Vec<u8>),
    StartHeartbeat(u64),
    Heartbeat,
    KeepAlive,
    PushMessage,
    TicketInvalid,
    Stop,
    Close,
}

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
impl<C> From<ApiClient<C>> for DanmakuToken {
    #[inline]
    fn from(client: ApiClient<C>) -> Self {
        match client.live() {
            Some(live) => Self::from_token_live(client.token(), live),
            None => Self::from_token_live(client.token(), &Live::default()),
        }
    }
}

#[cfg(feature = "api")]
impl<C> From<&ApiClient<C>> for DanmakuToken {
    #[inline]
    fn from(client: &ApiClient<C>) -> Self {
        match client.live() {
            Some(live) => Self::from_token_live(client.token(), live),
            None => Self::from_token_live(client.token(), &Live::default()),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DanmakuClient<C> {
    token: DanmakuToken,
    ws_client: C,
    action_tx: Option<mpsc::Sender<Vec<ActionSignal>>>,
    state_tx: Option<mpsc::Sender<Vec<StateSignal>>>,
    notify_tx: Option<mpsc::Sender<Vec<NotifySignal>>>,
}

#[cfg(feature = "default_ws_client")]
impl DanmakuClient<WsClient> {
    #[inline]
    pub fn default_client(token: DanmakuToken) -> Self {
        Self {
            token,
            ws_client: WsClient,
            action_tx: None,
            state_tx: None,
            notify_tx: None,
        }
    }

    #[cfg(feature = "api")]
    #[inline]
    pub async fn visitor(liver_uid: i64) -> Result<Self> {
        Ok(Self::default_client(
            DanmakuToken::visitor(liver_uid).await?,
        ))
    }

    #[cfg(feature = "api")]
    #[inline]
    pub async fn user<'a>(
        account: impl Into<Cow<'a, str>>,
        password: impl Into<Cow<'a, str>>,
        liver_uid: i64,
    ) -> Result<Self> {
        Ok(Self::default_client(
            DanmakuToken::user(account, password, liver_uid).await?,
        ))
    }

    #[cfg(feature = "api")]
    #[inline]
    pub async fn from_api_client<C>(client: &ApiClient<C>, liver_uid: i64) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        Ok(Self::default_client(
            DanmakuToken::from_api_client(client, liver_uid).await?,
        ))
    }
}

impl<C> DanmakuClient<C> {
    #[inline]
    pub fn new(token: DanmakuToken, client: C) -> Self {
        Self {
            token,
            ws_client: client,
            action_tx: None,
            state_tx: None,
            notify_tx: None,
        }
    }

    #[inline]
    pub fn set_token(&mut self, token: DanmakuToken) -> &mut Self {
        self.token = token;
        self
    }

    #[inline]
    pub fn token(&self) -> &DanmakuToken {
        &self.token
    }

    #[inline]
    pub fn token_mut(&mut self) -> &mut DanmakuToken {
        &mut self.token
    }

    #[inline]
    pub fn user_id(&self) -> i64 {
        self.token.user_id
    }

    #[inline]
    pub fn liver_uid(&self) -> i64 {
        self.token.liver_uid
    }

    #[inline]
    pub fn live_id(&self) -> &str {
        self.token.live_id.as_str()
    }

    #[inline]
    pub fn action_signal(&mut self) -> mpsc::Receiver<Vec<ActionSignal>> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        self.action_tx = Some(tx);
        rx
    }

    #[inline]
    pub fn state_signal(&mut self) -> mpsc::Receiver<Vec<StateSignal>> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        self.state_tx = Some(tx);
        rx
    }

    #[inline]
    pub fn notify_signal(&mut self) -> mpsc::Receiver<Vec<NotifySignal>> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        self.notify_tx = Some(tx);
        rx
    }
}

impl<C: WebSocket> DanmakuClient<C> {
    async fn send<T: Generate>(
        ws_write: &mut <C as WebSocket>::Write,
        data: &mut ProtoData,
    ) -> Result<()> {
        ws_write.write(T::generate(data)?).await
    }

    pub async fn danmaku(self) -> Result<()> {
        if !self.token.is_valid() {
            return Err(Error::InvalidToken);
        }
        let mut data: ProtoData = self.token.into();
        let (mut ws_write, mut ws_read) = self.ws_client.connect(DANMAKU_SERVER).await?;
        let mut action_tx = self.action_tx;
        let mut state_tx = self.state_tx;
        let mut notify_tx = self.notify_tx;

        ws_write
            .write(acproto::RegisterRequest::generate(&mut data)?)
            .await?;
        let msg = ws_read.read().await?;
        let payload = data.decode(msg.as_slice())?;
        data.register_response(&payload)?;
        ws_write
            .write(acproto::KeepAliveRequest::generate(&mut data)?)
            .await?;
        ws_write
            .write(acproto::ZtLiveCsEnterRoom::generate(&mut data)?)
            .await?;

        let (mut ws_tx, mut ws_rx) = mpsc::unbounded::<Command>();
        let mut read_ws_tx = ws_tx.clone();
        let mut hb_ws_tx = ws_tx.clone();
        let (hb_tx, hb_rx) = oneshot::channel::<Interval>();
        let write = async {
            let mut hb_tx = Some(hb_tx);
            let mut err = None;
            while let Some(cmd) = ws_rx.next().await {
                match cmd {
                    Command::Decode(msg) => {
                        let payload = match data.decode(msg.as_slice()) {
                            Ok(payload) => payload,
                            Err(e) => {
                                err = Some(e);
                                break;
                            }
                        };
                        if let Err(e) = handle(
                            &payload,
                            &mut ws_tx,
                            &mut action_tx,
                            &mut state_tx,
                            &mut notify_tx,
                        )
                        .await
                        {
                            err = Some(e);
                            break;
                        }
                    }
                    Command::StartHeartbeat(interval) => {
                        if let Some(tx) = hb_tx.take() {
                            let timer = async_timer::interval(Duration::from_millis(interval));
                            if tx.send(timer).is_err() {
                                err = Some(Error::SendOneshotError);
                                break;
                            }
                        }
                    }
                    Command::Heartbeat => {
                        if let Err(e) =
                            Self::send::<acproto::ZtLiveCsHeartbeat>(&mut ws_write, &mut data).await
                        {
                            err = Some(e);
                            break;
                        }
                    }
                    Command::KeepAlive => {
                        if let Err(e) =
                            Self::send::<acproto::KeepAliveRequest>(&mut ws_write, &mut data).await
                        {
                            err = Some(e);
                            break;
                        }
                    }
                    Command::PushMessage => {
                        if let Err(e) =
                            Self::send::<acproto::ZtLiveScMessage>(&mut ws_write, &mut data).await
                        {
                            err = Some(e);
                            break;
                        }
                    }
                    Command::TicketInvalid => {
                        data.ticket_index = (data.ticket_index + 1) % data.tickets.len();
                        if let Err(e) =
                            Self::send::<acproto::ZtLiveCsEnterRoom>(&mut ws_write, &mut data).await
                        {
                            err = Some(e);
                            break;
                        }
                    }
                    Command::Stop => {
                        if let Err(e) =
                            Self::send::<acproto::ZtLiveCsUserExit>(&mut ws_write, &mut data).await
                        {
                            err = Some(e);
                            break;
                        }
                        if let Err(e) =
                            Self::send::<acproto::UnregisterRequest>(&mut ws_write, &mut data).await
                        {
                            err = Some(e);
                            break;
                        }
                        let _ = ws_write.close().await;
                        ws_rx.close();
                        ws_tx.close_channel();
                    }
                    Command::Close => {
                        let _ = ws_write.close().await;
                        ws_rx.close();
                        ws_tx.close_channel();
                    }
                }
            }
            if let Some(e) = err {
                let _ = ws_write.close().await;
                ws_rx.close();
                ws_tx.close_channel();
                return Err(e);
            }

            Result::Ok(())
        };
        let read = async {
            while let Ok(msg) = ws_read.read().await {
                if read_ws_tx.send(Command::Decode(msg)).await.is_err() {
                    log::trace!("failed to send message to handle");
                    break;
                }
            }
            let _ = read_ws_tx.send(Command::Close).await;
            read_ws_tx.close_channel();
        };
        let heartbeat = async {
            let mut timer = match hb_rx.await {
                Ok(interval) => interval,
                Err(e) => {
                    hb_ws_tx.close_channel();
                    return Err(e.into());
                }
            };
            let mut heartbeat_seq_id: i64 = 0;
            while hb_ws_tx.send(Command::Heartbeat).await.is_ok() {
                heartbeat_seq_id += 1;
                if heartbeat_seq_id % 5 == 4 && hb_ws_tx.send(Command::KeepAlive).await.is_err() {
                    break;
                }
                timer.as_mut().await;
            }
            hb_ws_tx.close_channel();

            Result::Ok(())
        };
        let (wr, _, hbr) = join!(write, read, heartbeat);
        if let Err(e) = wr {
            log::trace!("writing WebSocket message error: {:?}", e);
        }
        if let Err(e) = hbr {
            log::trace!("heartbeat error: {:?}", e);
        }
        let _ = ws_write.close().await;

        Ok(())
    }
}

#[cfg(feature = "default_ws_client")]
impl From<DanmakuToken> for DanmakuClient<WsClient> {
    #[inline]
    fn from(token: DanmakuToken) -> Self {
        Self::default_client(token)
    }
}

#[cfg(all(feature = "api", feature = "default_ws_client"))]
impl<C> From<ApiClient<C>> for DanmakuClient<WsClient> {
    #[inline]
    fn from(client: ApiClient<C>) -> Self {
        Self::default_client(client.into())
    }
}

#[cfg(all(feature = "api", feature = "default_ws_client"))]
impl<C> From<&ApiClient<C>> for DanmakuClient<WsClient> {
    #[inline]
    fn from(client: &ApiClient<C>) -> Self {
        Self::default_client(client.into())
    }
}

async fn handle(
    stream: &acproto::DownstreamPayload,
    ws_tx: &mut mpsc::UnboundedSender<Command>,
    action_tx: &mut Option<mpsc::Sender<Vec<ActionSignal>>>,
    state_tx: &mut Option<mpsc::Sender<Vec<StateSignal>>>,
    notify_tx: &mut Option<mpsc::Sender<Vec<NotifySignal>>>,
) -> Result<()> {
    match stream.command.as_str() {
        GLOBAL_CS_CMD => {
            let cmd = acproto::ZtLiveCsCmdAck::decode(stream.payload_data.as_slice())?;
            match cmd.cmd_ack_type.as_str() {
                ENTER_ROOM_ACK => {
                    let enter_room = acproto::ZtLiveCsEnterRoomAck::decode(cmd.payload.as_slice())?;
                    let interval = if enter_room.heartbeat_interval_ms > 0 {
                        u64::try_from(enter_room.heartbeat_interval_ms)?
                    } else {
                        10000
                    };
                    let _ = ws_tx.send(Command::StartHeartbeat(interval)).await;
                }
                HEARTBEAT_ACK => {}
                USER_EXIT_ACK => {}
                _ => {}
            }
        }
        KEEP_ALIVE => {}
        PING => {}
        UNREGISTER => {
            let _ = ws_tx.send(Command::Close).await;
        }
        PUSH_MESSAGE => {
            let _ = ws_tx.send(Command::PushMessage).await;
            let message = acproto::ZtLiveScMessage::decode(stream.payload_data.as_slice())?;
            let payload = if message.compression_type()
                == acproto::zt_live_sc_message::CompressionType::Gzip
            {
                let mut reader = GzipDecoder::new(message.payload.as_slice());
                let mut buf = Vec::new();
                let _ = reader.read_to_end(&mut buf).await?;
                buf
            } else {
                message.payload
            };
            match message.message_type.as_str() {
                ACTION_SIGNAL => {
                    if let Some(tx) = action_tx {
                        action_signal(payload.as_slice(), tx).await?;
                    }
                }
                STATE_SIGNAL => {
                    if let Some(tx) = state_tx {
                        state_signal(payload.as_slice(), tx).await?;
                    }
                }
                NOTIFY_SIGNAL => {
                    if let Some(tx) = notify_tx {
                        notify_signal(payload.as_slice(), tx).await?;
                    }
                }
                STATUS_CHANGED => {
                    let status = acproto::ZtLiveScStatusChanged::decode(payload.as_slice())?;
                    if status.r#type() == acproto::zt_live_sc_status_changed::Type::LiveClosed
                        || status.r#type() == acproto::zt_live_sc_status_changed::Type::LiveBanned
                    {
                        let _ = ws_tx.send(Command::Stop).await;
                    }
                }
                TICKET_INVALID => {
                    log::trace!("danmaku ticket invalid");
                    let _ = ws_tx.send(Command::TicketInvalid).await;
                }
                _ => {}
            }
        }
        _ => {
            if stream.error_code == 10018 {
                let _ = ws_tx.send(Command::Stop).await;
            }
        }
    }

    Ok(())
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
        let mut action_rx = client.action_signal();
        let action = async {
            while let Some(action) = action_rx.next().await {
                println!("{:?}", action);
            }
        };
        let mut state_rx = client.state_signal();
        let state = async {
            while let Some(state) = state_rx.next().await {
                println!("{:?}", state);
            }
        };
        let mut notify_rx = client.notify_signal();
        let notify = async {
            while let Some(notify) = notify_rx.next().await {
                println!("{:?}", notify);
            }
        };
        select! {
            result = client.danmaku() => {
                result?;
            }
            _ = action => {}
            _ = state => {}
            _ = notify => {}
            _ = sleep(Duration::from_secs(60)) => {}
        }

        Ok(())
    }
}
