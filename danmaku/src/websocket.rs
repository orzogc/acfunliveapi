use crate::{Error, Result};
use async_trait::async_trait;
use futures::{
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
};
use std::borrow::Cow;

#[cfg(feature = "default_ws_client")]
use futures::stream::{SplitSink, SplitStream};
#[cfg(feature = "default_ws_client")]
use std::{
    pin::Pin,
    task::{Context, Poll},
};
#[cfg(feature = "default_ws_client")]
use tokio::{net::TcpStream, time::timeout};
#[cfg(feature = "default_ws_client")]
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[cfg(feature = "default_ws_client")]
const WS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[async_trait]
pub trait WebSocketWrite: Sink<Vec<u8>> + Unpin + Send {
    #[inline]
    async fn write<T>(&mut self, message: T) -> Result<()>
    where
        T: Into<Vec<u8>> + Send,
    {
        self.send(message.into())
            .await
            .map_err(|_| Error::WsWriteError)
    }

    #[inline]
    async fn close_connection(&mut self) -> Result<()> {
        self.close().await.map_err(|_| Error::WsCloseError)
    }
}

#[async_trait]
pub trait WebSocketRead: Stream<Item = Result<Vec<u8>>> + Unpin + Send {
    #[inline]
    async fn read(&mut self) -> Result<Vec<u8>> {
        self.next().await.ok_or(Error::WsClosed)?
    }
}

#[async_trait]
pub trait WebSocket {
    type Write: WebSocketWrite;
    type Read: WebSocketRead;

    async fn connect<'a, T>(self, url: T) -> Result<(Self::Write, Self::Read)>
    where
        T: Into<Cow<'a, str>> + Send;
}

#[cfg(feature = "default_ws_client")]
pub struct WsWrite(SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>);

#[cfg(feature = "default_ws_client")]
impl Sink<Vec<u8>> for WsWrite {
    type Error = Error;

    #[inline]
    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.0.poll_ready_unpin(cx).map_err(Into::into)
    }

    #[inline]
    fn start_send(mut self: Pin<&mut Self>, item: Vec<u8>) -> Result<()> {
        self.0
            .start_send_unpin(Message::Binary(item))
            .map_err(Into::into)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.0.poll_flush_unpin(cx).map_err(Into::into)
    }

    #[inline]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.0.poll_close_unpin(cx).map_err(Into::into)
    }
}

#[cfg(feature = "default_ws_client")]
#[async_trait]
impl WebSocketWrite for WsWrite {
    #[inline]
    async fn write<T>(&mut self, message: T) -> Result<()>
    where
        T: Into<Vec<u8>> + Send,
    {
        match timeout(WS_TIMEOUT, self.0.send(Message::binary(message))).await {
            Ok(result) => result.map_err(Into::into),
            Err(_) => Err(Error::WsWriteTimeout),
        }
    }

    #[inline]
    async fn close_connection(&mut self) -> Result<()> {
        match timeout(WS_TIMEOUT, self.0.send(Message::Close(None))).await {
            Ok(result) => result.map_err(Into::into),
            Err(_) => Err(Error::WsCloseTimeout),
        }
    }
}

#[cfg(feature = "default_ws_client")]
pub struct WsRead(SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>);

#[cfg(feature = "default_ws_client")]
impl Stream for WsRead {
    type Item = Result<Vec<u8>>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0
            .poll_next_unpin(cx)
            .map_ok(|m| m.into_data())
            .map_err(Into::into)
    }
}

#[cfg(feature = "default_ws_client")]
#[async_trait]
impl WebSocketRead for WsRead {
    #[inline]
    async fn read(&mut self) -> Result<Vec<u8>> {
        match timeout(WS_TIMEOUT, self.0.next()).await {
            Ok(result) => Ok(result.ok_or(Error::WsClosed)??.into_data()),
            Err(_) => Err(Error::WsReadTimeout),
        }
    }
}

#[cfg(feature = "default_ws_client")]
#[cfg_attr(feature = "_serde", derive(::serde::Deserialize, ::serde::Serialize))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct WsClient;

#[cfg(feature = "default_ws_client")]
#[async_trait]
impl WebSocket for WsClient {
    type Write = WsWrite;
    type Read = WsRead;

    #[inline]
    async fn connect<'a, T>(self, url: T) -> Result<(Self::Write, Self::Read)>
    where
        T: Into<Cow<'a, str>> + Send,
    {
        let url = url.into();
        match timeout(WS_TIMEOUT, connect_async(url.as_ref())).await {
            Ok(result) => {
                let (write, read) = result?.0.split();
                Ok((WsWrite(write), WsRead(read)))
            }
            Err(_) => Err(Error::WsConnectTimeout),
        }
    }
}

#[cfg(feature = "default_ws_client")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::from_utf8;

    #[tokio::test]
    async fn test_websocket() -> Result<()> {
        let client = WsClient {};
        let (mut ws_write, mut ws_read) = client.connect("ws://echo.websocket.org/").await?;
        ws_write.write("hello").await?;
        let msg = ws_read.read().await?;
        let msg = from_utf8(msg.as_slice()).unwrap();
        assert_eq!(msg, "hello");
        ws_write.close().await
    }
}
