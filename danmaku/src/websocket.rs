use crate::Result;
use async_trait::async_trait;
use std::borrow::Cow;

#[cfg(feature = "default_ws_client")]
use crate::Error;
#[cfg(feature = "default_ws_client")]
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
#[cfg(feature = "default_ws_client")]
use tokio::net::TcpStream;
#[cfg(feature = "default_ws_client")]
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[async_trait]
pub trait WebSocketWrite {
    async fn write<T>(&mut self, message: T) -> Result<()>
    where
        T: Into<Vec<u8>> + Send;

    async fn close(&mut self) -> Result<()>;
}

#[async_trait]
pub trait WebSocketRead {
    async fn read(&mut self) -> Result<Vec<u8>>;
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
#[async_trait]
impl WebSocketWrite for WsWrite {
    async fn write<T>(&mut self, message: T) -> Result<()>
    where
        T: Into<Vec<u8>> + Send,
    {
        self.0
            .send(Message::binary(message))
            .await
            .map_err(|e| Error::WsWriteError(Box::new(e)))
    }

    async fn close(&mut self) -> Result<()> {
        self.0
            .send(Message::Close(None))
            .await
            .map_err(|e| Error::WsCloseError(Box::new(e)))?;
        self.0
            .close()
            .await
            .map_err(|e| Error::WsCloseError(Box::new(e)))
    }
}

#[cfg(feature = "default_ws_client")]
pub struct WsRead(SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>);

#[cfg(feature = "default_ws_client")]
#[async_trait]
impl WebSocketRead for WsRead {
    async fn read(&mut self) -> Result<Vec<u8>> {
        Ok(self
            .0
            .next()
            .await
            .ok_or(Error::WsClosed)?
            .map_err(|e| Error::WsReadError(Box::new(e)))?
            .into_data())
    }
}

#[cfg(feature = "default_ws_client")]
#[cfg_attr(feature = "_serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct WsClient;

#[cfg(feature = "default_ws_client")]
#[async_trait]
impl WebSocket for WsClient {
    type Write = WsWrite;
    type Read = WsRead;

    async fn connect<'a, T>(self, url: T) -> Result<(Self::Write, Self::Read)>
    where
        T: Into<Cow<'a, str>> + Send,
    {
        let (stream, _) = connect_async(url.into().as_ref())
            .await
            .map_err(|e| Error::WsConnectError(Box::new(e)))?;
        let (write, read) = stream.split();

        Ok((WsWrite(write), WsRead(read)))
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
