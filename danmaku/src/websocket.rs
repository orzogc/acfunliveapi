use crate::Error;
use async_trait::async_trait;
use futures::{AsyncRead, AsyncWrite};
use std::borrow::Cow;

#[cfg(feature = "default_ws_client")]
use crate::Result;
#[cfg(feature = "default_ws_client")]
use async_tungstenite::tokio::ConnectStream;
#[cfg(feature = "default_ws_client")]
use futures::{
    io::{IoSlice, IoSliceMut},
    AsyncBufRead,
};
#[cfg(feature = "default_ws_client")]
use std::{
    pin::Pin,
    task::{Context, Poll},
};
#[cfg(feature = "default_ws_client")]
use tokio::time::timeout;
#[cfg(feature = "default_ws_client")]
use ws_stream_tungstenite::WsStream;

#[cfg(feature = "default_ws_client")]
const WS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[async_trait]
pub trait WebSocket: AsyncRead + AsyncWrite + Unpin + Sized {
    type Error: From<Error>;

    async fn connect<'a, T>(url: T) -> std::result::Result<Self, Self::Error>
    where
        T: Into<Cow<'a, str>> + Send;
}

#[cfg(feature = "default_ws_client")]
#[derive(Debug)]
pub struct WebSocketClient(WsStream<ConnectStream>);

#[cfg(feature = "default_ws_client")]
impl AsyncRead for WebSocketClient {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }

    #[inline]
    fn poll_read_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_read_vectored(cx, bufs)
    }
}

#[cfg(feature = "default_ws_client")]
impl AsyncWrite for WebSocketClient {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    #[inline]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }

    #[inline]
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write_vectored(cx, bufs)
    }
}

#[cfg(feature = "default_ws_client")]
impl AsyncBufRead for WebSocketClient {
    #[inline]
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        Pin::new(&mut self.get_mut().0).poll_fill_buf(cx)
    }

    #[inline]
    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.0).consume(amt)
    }
}

#[cfg(feature = "default_ws_client")]
#[async_trait]
impl WebSocket for WebSocketClient {
    type Error = Error;

    #[inline]
    async fn connect<'a, T>(url: T) -> Result<Self>
    where
        T: Into<Cow<'a, str>> + Send,
    {
        let url = url.into();
        match timeout(
            WS_TIMEOUT,
            async_tungstenite::tokio::connect_async(url.as_ref()),
        )
        .await
        {
            Ok(result) => Ok(Self(WsStream::new(result?.0))),
            Err(_) => Err(Error::WsConnectTimeout),
        }
    }
}

#[cfg(feature = "default_ws_client")]
#[cfg(test)]
mod tests {
    use super::*;
    use futures::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_websocket() -> Result<()> {
        let mut client = WebSocketClient::connect("ws://echo.websocket.org/").await?;
        client.write_all(b"hello").await?;
        let mut msg = [0u8; 5];
        client.read_exact(&mut msg).await?;
        assert_eq!(&msg, b"hello");
        client.close().await?;

        Ok(())
    }
}
