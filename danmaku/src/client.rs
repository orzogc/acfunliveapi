use crate::{danmaku::*, global::*, proto::*, websocket::*, Error, Result};
use async_compression::futures::bufread::GzipDecoder;
use futures::{
    channel::{mpsc, oneshot},
    io::AsyncReadExt,
    try_join,
    {sink::SinkExt, stream::StreamExt},
};
use futures_timer::Delay;
use prost::Message;
use std::time::Duration;

#[cfg(feature = "api")]
use acfunliveapi::{
    client::{Client as ApiClient, ClientBuilder as ApiClientBuilder, Live, Token as ApiToken},
    pretend,
};

const CHANNEL_SIZE: usize = 100;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

#[cfg_attr(feature = "_serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Token {
    pub user_id: i64,
    pub security_key: String,
    pub service_token: String,
    pub live_id: String,
    pub enter_room_attach: String,
    pub tickets: Vec<String>,
}

#[cfg(feature = "api")]
impl Token {
    pub async fn new(liver_uid: i64) -> Result<Self> {
        let client = ApiClientBuilder::default_client()?
            .liver_uid(liver_uid)
            .build()
            .await?;

        Ok(Self::from_token_live(client.token(), client.live()))
    }

    pub async fn from_api_client<C>(client: &ApiClient<C>, liver_uid: i64) -> Result<Self>
    where
        C: pretend::client::Client + Send + Sync,
    {
        let info = client.get_live_info(liver_uid).await?;
        let token = client.token();

        Ok(Self {
            user_id: token.user_id,
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
            security_key: token.security_key.clone(),
            service_token: token.service_token.clone(),
            live_id: live.live_id.clone(),
            enter_room_attach: live.enter_room_attach.clone(),
            tickets: live.tickets.clone(),
        }
    }
}

#[cfg(feature = "api")]
impl<C> From<ApiClient<C>> for Token {
    #[inline]
    fn from(client: ApiClient<C>) -> Self {
        Self::from_token_live(client.token(), client.live())
    }
}

#[cfg(feature = "api")]
impl<C> From<&ApiClient<C>> for Token {
    #[inline]
    fn from(client: &ApiClient<C>) -> Self {
        Self::from_token_live(client.token(), client.live())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Client<C> {
    token: Token,
    ws_client: C,
    action_tx: Option<async_channel::Sender<ActionSignal>>,
    state_tx: Option<async_channel::Sender<StateSignal>>,
    notify_tx: Option<async_channel::Sender<NotifySignal>>,
}

impl<C> Client<C> {
    #[inline]
    pub fn set_token(&mut self, token: Token) -> &mut Self {
        self.token = token;
        self
    }

    #[inline]
    pub fn token(&self) -> &Token {
        &self.token
    }

    #[inline]
    pub fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }

    #[inline]
    pub fn action_signal(&mut self) -> async_channel::Receiver<ActionSignal> {
        let (tx, rx) = async_channel::bounded(CHANNEL_SIZE);
        self.action_tx = Some(tx);
        rx
    }

    #[inline]
    pub fn state_signal(&mut self) -> async_channel::Receiver<StateSignal> {
        let (tx, rx) = async_channel::bounded(CHANNEL_SIZE);
        self.state_tx = Some(tx);
        rx
    }

    #[inline]
    pub fn notify_signal(&mut self) -> async_channel::Receiver<NotifySignal> {
        let (tx, rx) = async_channel::bounded(CHANNEL_SIZE);
        self.notify_tx = Some(tx);
        rx
    }
}

impl<C: WebSocket> Client<C> {
    pub async fn danmaku(self) -> Result<()> {
        let mut data: ProtoData = self.token.into();
        let (mut ws_write, mut ws_read) = self.ws_client.connect(DANMAKU_SERVER).await?;
        let action_tx = self.action_tx;
        let state_tx = self.state_tx;
        let notify_tx = self.notify_tx;

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

        let (mut ws_tx, mut ws_rx) = mpsc::channel::<Command>(CHANNEL_SIZE);
        let mut read_ws_tx = ws_tx.clone();
        let mut hb_ws_tx = ws_tx.clone();
        let (hb_tx, hb_rx) = oneshot::channel::<u64>();
        let write = async {
            let mut hb_tx = Some(hb_tx);
            while let Some(cmd) = ws_rx.next().await {
                match cmd {
                    Command::Decode(msg) => {
                        let payload = data.decode(msg.as_slice())?;
                        handle(&payload, &mut ws_tx, &action_tx, &state_tx, &notify_tx).await?;
                    }
                    Command::StartHeartbeat(interval) => {
                        if let Some(tx) = hb_tx.take() {
                            tx.send(interval).or(Err(Error::SendOneshotError))?;
                        }
                    }
                    Command::Heartbeat => {
                        ws_write
                            .write(acproto::ZtLiveCsHeartbeat::generate(&mut data)?)
                            .await?
                    }
                    Command::KeepAlive => {
                        ws_write
                            .write(acproto::KeepAliveRequest::generate(&mut data)?)
                            .await?
                    }
                    Command::PushMessage => {
                        ws_write
                            .write(acproto::ZtLiveScMessage::generate(&mut data)?)
                            .await?
                    }
                    Command::TicketInvalid => {
                        data.ticket_index = (data.ticket_index + 1) % data.tickets.len();
                        ws_write
                            .write(acproto::ZtLiveCsEnterRoom::generate(&mut data)?)
                            .await?;
                    }
                    Command::Stop => {
                        ws_write
                            .write(acproto::ZtLiveCsUserExit::generate(&mut data)?)
                            .await?;
                        ws_write
                            .write(acproto::UnregisterRequest::generate(&mut data)?)
                            .await?;
                        ws_write.close().await?;
                        break;
                    }
                    Command::Close => {
                        ws_write.close().await?;
                        break;
                    }
                }
            }

            Result::<()>::Err(Error::StopDanmaku("stop writing"))
        };
        let read = async {
            while let Ok(msg) = ws_read.read().await {
                read_ws_tx.send(Command::Decode(msg)).await?;
            }
            read_ws_tx.send(Command::Close).await?;

            Result::<()>::Err(Error::StopDanmaku("stop reading"))
        };
        let heartbeat = async {
            let interval = hb_rx.await?;
            let mut heartbeat_seq_id: i64 = 0;

            loop {
                hb_ws_tx.send(Command::Heartbeat).await?;
                heartbeat_seq_id += 1;
                if heartbeat_seq_id % 5 == 4 {
                    hb_ws_tx.send(Command::KeepAlive).await?;
                }

                Delay::new(Duration::from_millis(interval)).await;
            }

            #[allow(unreachable_code)]
            Result::<()>::Err(Error::StopDanmaku("stop heartbeat"))
        };
        let _ = try_join!(write, read, heartbeat);
        let _ = ws_write.close().await;

        Ok(())
    }
}

#[cfg(all(feature = "api", feature = "default_ws_client"))]
impl<C> From<ApiClient<C>> for Client<WsClient> {
    #[inline]
    fn from(client: ApiClient<C>) -> Self {
        ClientBuilder::default_client(client.into()).build()
    }
}

#[cfg(all(feature = "api", feature = "default_ws_client"))]
impl<C> From<&ApiClient<C>> for Client<WsClient> {
    #[inline]
    fn from(client: &ApiClient<C>) -> Self {
        ClientBuilder::default_client(client.into()).build()
    }
}

#[cfg_attr(feature = "_serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct ClientBuilder<C> {
    token: Token,
    ws_client: C,
}

#[cfg(feature = "default_ws_client")]
impl ClientBuilder<WsClient> {
    #[inline]
    pub fn default_client(token: Token) -> Self {
        Self {
            token,
            ws_client: WsClient,
        }
    }
}

impl<C> ClientBuilder<C> {
    #[inline]
    pub fn new(token: Token, client: C) -> Self {
        Self {
            token,
            ws_client: client,
        }
    }

    #[inline]
    pub fn build(self) -> Client<C> {
        Client {
            token: self.token,
            ws_client: self.ws_client,
            action_tx: None,
            state_tx: None,
            notify_tx: None,
        }
    }
}

impl<C> From<ClientBuilder<C>> for Client<C> {
    #[inline]
    fn from(builder: ClientBuilder<C>) -> Self {
        builder.build()
    }
}

async fn handle(
    stream: &acproto::DownstreamPayload,
    ws_tx: &mut mpsc::Sender<Command>,
    action_tx: &Option<async_channel::Sender<ActionSignal>>,
    state_tx: &Option<async_channel::Sender<StateSignal>>,
    notify_tx: &Option<async_channel::Sender<NotifySignal>>,
) -> Result<()> {
    match stream.command.as_str() {
        GLOBAL_CS_CMD => {
            let cmd = acproto::ZtLiveCsCmdAck::decode(stream.payload_data.as_slice())?;
            match cmd.cmd_ack_type.as_str() {
                ENTER_ROOM_ACK => {
                    let enter_room = acproto::ZtLiveCsEnterRoomAck::decode(cmd.payload.as_slice())?;
                    let interval = if enter_room.heartbeat_interval_ms > 0 {
                        enter_room.heartbeat_interval_ms as u64
                    } else {
                        10000
                    };
                    ws_tx.send(Command::StartHeartbeat(interval)).await?;
                }
                HEARTBEAT_ACK => {}
                USER_EXIT_ACK => {}
                _ => {}
            }
        }
        KEEP_ALIVE => {}
        PING => {}
        UNREGISTER => {
            ws_tx.send(Command::Close).await?;
        }
        PUSH_MESSAGE => {
            ws_tx.send(Command::PushMessage).await?;
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
                    action_signal(payload.as_slice(), action_tx).await?;
                }
                STATE_SIGNAL => {
                    state_signal(payload.as_slice(), state_tx).await?;
                }
                NOTIFY_SIGNAL => {
                    notify_signal(payload.as_slice(), notify_tx).await?;
                }
                STATUS_CHANGED => {
                    let status = acproto::ZtLiveScStatusChanged::decode(payload.as_slice())?;
                    if status.r#type() == acproto::zt_live_sc_status_changed::Type::LiveClosed
                        || status.r#type() == acproto::zt_live_sc_status_changed::Type::LiveBanned
                    {
                        ws_tx.send(Command::Stop).await?;
                    }
                }
                TICKET_INVALID => {
                    ws_tx.send(Command::TicketInvalid).await?;
                }
                _ => {}
            }
        }
        _ => {
            if stream.error_code == 10018 {
                ws_tx.send(Command::Stop).await?;
            }
        }
    }

    Ok(())
}

#[cfg(all(feature = "api", feature = "default_ws_client"))]
#[cfg(test)]
mod tests {
    use super::*;
    use futures::{future::FutureExt, select};
    use std::env;

    #[tokio::test]
    async fn test_danmaku() -> Result<()> {
        let liver_uid: i64 = env::var("LIVER_UID")
            .expect("need to set the LIVER_UID environment variable to the liver's uid")
            .parse()
            .expect("LIVER_UID should be an integer");
        let mut client = ClientBuilder::default_client(Token::new(liver_uid).await?).build();
        let action_rx = client.action_signal();
        let action = async {
            while let Ok(action) = action_rx.recv().await {
                println!("{:?}", action);
            }
        };
        let state_rx = client.state_signal();
        let state = async {
            while let Ok(state) = state_rx.recv().await {
                println!("{:?}", state);
            }
        };
        let notify_rx = client.notify_signal();
        let notify = async {
            while let Ok(notify) = notify_rx.recv().await {
                println!("{:?}", notify);
            }
        };
        select! {
            result = client.danmaku().fuse() => {
                result?;
            }
            _ = action.fuse() => {}
            _ = state.fuse() => {}
            _ = notify.fuse() => {}
            _ = Delay::new(Duration::from_secs(60)).fuse() => {}
        }

        Ok(())
    }
}
