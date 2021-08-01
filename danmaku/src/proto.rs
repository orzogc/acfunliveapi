use crate::{client::DanmakuToken, danmaku::*, global::*, Error, Result};
use aes::Aes128;
use asynchronous_codec::{BytesMut, Decoder, Encoder};
use block_modes::{block_padding::Pkcs7, BlockMode, Cbc};
use flate2::read::GzDecoder;
use prost::{bytes::Buf, Message};
use rand::{distributions::Standard, Rng};
use std::{
    convert::{TryFrom, TryInto},
    io::Read,
    time::SystemTime,
};

const U32_LENGTH: usize = std::mem::size_of::<u32>();
const PROTO_MAGIC: [u8; 4] = 0xABCD0001u32.to_be_bytes();

pub mod acproto {
    include!(concat!(env!("OUT_DIR"), "/acproto.rs"));
}

fn encrypt(plain_text: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let mut iv: Vec<u8> = rand::thread_rng()
        .sample_iter(Standard)
        .take(aes::BLOCK_SIZE)
        .collect();
    let cipher: Cbc<Aes128, Pkcs7> = Cbc::new_from_slices(key, &iv)?;
    let mut cipher_text = cipher.encrypt_vec(plain_text);
    iv.append(&mut cipher_text);

    Ok(iv)
}

fn decrypt(cipher_text: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if cipher_text.len() <= aes::BLOCK_SIZE {
        return Err(Error::CipherTextTooShort(cipher_text.len()));
    }
    let iv = &cipher_text[..aes::BLOCK_SIZE];
    let cipher_text = &cipher_text[aes::BLOCK_SIZE..];
    let cipher: Cbc<Aes128, Pkcs7> = Cbc::new_from_slices(key, iv)?;

    Ok(cipher.decrypt_vec(cipher_text)?)
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SendMessage {
    RegisterRequest,
    UnregisterRequest,
    ZtLiveCsEnterRoom,
    KeepAliveRequest,
    ZtLiveScMessage,
    ZtLiveCsHeartbeat,
    ZtLiveCsUserExit,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ReceiveMessage {
    Danmaku(Danmaku),
    RegisterResponse,
    Interval(u64),
    PushMessage,
    EnterRoom,
    PushAndStop,
    Stop,
    Close,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DanmakuProto {
    pub(crate) user_id: i64,
    pub(crate) liver_uid: i64,
    security_key: Vec<u8>,
    service_token: String,
    pub(crate) live_id: String,
    enter_room_attach: String,
    tickets: Vec<String>,
    app_id: i32,
    instance_id: i64,
    session_key: Option<Vec<u8>>,
    seq_id: i64,
    header_seq_id: i64,
    heartbeat_seq_id: i64,
    ticket_index: usize,
}

impl DanmakuProto {
    #[inline]
    fn header(&self) -> acproto::PacketHeader {
        acproto::PacketHeader {
            app_id: self.app_id,
            uid: self.user_id,
            instance_id: self.instance_id,
            encryption_mode: acproto::packet_header::EncryptionMode::KEncryptionSessionKey.into(),
            seq_id: self.seq_id,
            kpn: KPN.to_string(),
            ..Default::default()
        }
    }

    #[inline]
    fn payload(&self, command: String, paload_data: Option<Vec<u8>>) -> acproto::UpstreamPayload {
        let mut payload = acproto::UpstreamPayload {
            command,
            seq_id: self.seq_id,
            retry_count: RETRY_COUNT,
            sub_biz: SUB_BIZ.to_string(),
            ..Default::default()
        };
        if let Some(data) = paload_data {
            payload.payload_data = data;
        }

        payload
    }

    #[inline]
    fn command(&self, command: String, paload: Option<Vec<u8>>) -> Vec<u8> {
        let mut cmd = acproto::ZtLiveCsCmd {
            cmd_type: command,
            ticket: self
                .tickets
                .get(self.ticket_index)
                .expect("ticket_index is out of range")
                .clone(),
            live_id: self.live_id.clone(),
            ..Default::default()
        };
        if let Some(data) = paload {
            cmd.payload = data;
        }

        cmd.encode_to_vec()
    }

    #[inline]
    pub(crate) fn register_response(&mut self, payload: &acproto::DownstreamPayload) -> Result<()> {
        let resp = acproto::RegisterResponse::decode(payload.payload_data.as_slice())?;
        self.instance_id = resp.instance_id;
        self.session_key = Some(resp.sess_key);

        Ok(())
    }

    fn danmaku(&mut self, stream: &acproto::DownstreamPayload) -> Result<Option<ReceiveMessage>> {
        match stream.command.as_str() {
            REGISTER => {
                self.register_response(stream)?;
                Ok(Some(ReceiveMessage::RegisterResponse))
            }
            GLOBAL_CS_CMD => {
                let cmd = acproto::ZtLiveCsCmdAck::decode(stream.payload_data.as_slice())?;
                match cmd.cmd_ack_type.as_str() {
                    ENTER_ROOM_ACK => {
                        let enter_room =
                            acproto::ZtLiveCsEnterRoomAck::decode(cmd.payload.as_slice())?;
                        let interval = if enter_room.heartbeat_interval_ms > 0 {
                            u64::try_from(enter_room.heartbeat_interval_ms)?
                        } else {
                            10000
                        };
                        Ok(Some(ReceiveMessage::Interval(interval)))
                    }
                    HEARTBEAT_ACK => Ok(None),
                    USER_EXIT_ACK => Ok(None),
                    _ => {
                        log::trace!("unknown ZtLiveCsCmdAck cmd_ack_type: {}", cmd.cmd_ack_type);
                        Ok(None)
                    }
                }
            }
            KEEP_ALIVE => Ok(None),
            PING => Ok(None),
            UNREGISTER => Ok(Some(ReceiveMessage::Close)),
            PUSH_MESSAGE => {
                let message = acproto::ZtLiveScMessage::decode(stream.payload_data.as_slice())?;
                let payload = if message.compression_type()
                    == acproto::zt_live_sc_message::CompressionType::Gzip
                {
                    let mut reader = GzDecoder::new(message.payload.as_slice());
                    let mut buf = Vec::new();
                    let _ = reader.read_to_end(&mut buf)?;
                    buf
                } else {
                    message.payload
                };
                match message.message_type.as_str() {
                    ACTION_SIGNAL => Ok(Some(ReceiveMessage::Danmaku(
                        action_signal(&payload)?.into(),
                    ))),
                    STATE_SIGNAL => Ok(Some(ReceiveMessage::Danmaku(
                        state_signal(&payload)?.into(),
                    ))),
                    NOTIFY_SIGNAL => Ok(Some(ReceiveMessage::Danmaku(
                        notify_signal(&payload)?.into(),
                    ))),
                    STATUS_CHANGED => {
                        let status = acproto::ZtLiveScStatusChanged::decode(payload.as_slice())?;
                        if status.r#type() == acproto::zt_live_sc_status_changed::Type::LiveClosed
                            || status.r#type()
                                == acproto::zt_live_sc_status_changed::Type::LiveBanned
                        {
                            Ok(Some(ReceiveMessage::PushAndStop))
                        } else {
                            Ok(Some(ReceiveMessage::PushMessage))
                        }
                    }
                    TICKET_INVALID => {
                        log::trace!("danmaku ticket is invalid");
                        self.ticket_index = (self.ticket_index + 1) % self.tickets.len();
                        Ok(Some(ReceiveMessage::EnterRoom))
                    }
                    _ => {
                        log::trace!(
                            "unknown ZtLiveScMessage message_type: {}",
                            message.message_type
                        );
                        Ok(Some(ReceiveMessage::PushMessage))
                    }
                }
            }
            _ => {
                if stream.error_code == 10018 {
                    log::trace!(
                        "DownstreamPayload error: error_code is 10018, stop getting danmaku, error_msg: {}",
                        stream.error_msg
                    );
                    Ok(Some(ReceiveMessage::Stop))
                } else if stream.error_code != 0 {
                    log::trace!(
                        "DownstreamPayload error: error_code: {} , error_msg: {}",
                        stream.error_code,
                        stream.error_msg
                    );
                    Ok(None)
                } else {
                    log::trace!("unknown DownstreamPayload command: {}", stream.command);
                    Ok(None)
                }
            }
        }
    }
}

impl Encoder for DanmakuProto {
    type Item = SendMessage;

    type Error = Error;

    // https://github.com/wpscott/AcFunDanmaku/tree/master/AcFunDanmu
    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<()> {
        let (mut header, payload) = match item {
            SendMessage::RegisterRequest => acproto::RegisterRequest::generate(self),
            SendMessage::UnregisterRequest => acproto::UnregisterRequest::generate(self),
            SendMessage::ZtLiveCsEnterRoom => acproto::ZtLiveCsEnterRoom::generate(self),
            SendMessage::KeepAliveRequest => acproto::KeepAliveRequest::generate(self),
            SendMessage::ZtLiveScMessage => acproto::ZtLiveScMessage::generate(self),
            SendMessage::ZtLiveCsHeartbeat => acproto::ZtLiveCsHeartbeat::generate(self),
            SendMessage::ZtLiveCsUserExit => acproto::ZtLiveCsUserExit::generate(self),
        };

        let payload = payload.encode_to_vec();
        header.decoded_payload_len = u32::try_from(payload.len())?;
        let encrypted = match header.encryption_mode() {
            acproto::packet_header::EncryptionMode::KEncryptionNone => payload,
            acproto::packet_header::EncryptionMode::KEncryptionServiceToken => {
                encrypt(&payload, &self.security_key)?
            }
            acproto::packet_header::EncryptionMode::KEncryptionSessionKey => {
                if let Some(key) = &self.session_key {
                    encrypt(&payload, key)?
                } else {
                    return Err(Error::NoSessionKey);
                }
            }
        };
        let header = header.encode_to_vec();

        dst.reserve(3 * U32_LENGTH + header.len() + encrypted.len());
        dst.extend_from_slice(&PROTO_MAGIC);
        dst.extend_from_slice(&(u32::try_from(header.len())?).to_be_bytes());
        dst.extend_from_slice(&(u32::try_from(encrypted.len())?).to_be_bytes());
        dst.extend_from_slice(&header);
        dst.extend_from_slice(&encrypted);

        Ok(())
    }
}

impl Decoder for DanmakuProto {
    type Item = ReceiveMessage;

    type Error = Error;

    // https://github.com/wpscott/AcFunDanmaku/tree/master/AcFunDanmu
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        if src.len() < 3 * U32_LENGTH {
            return Ok(None);
        }

        let header_length = usize::try_from(u32::from_be_bytes(
            src[U32_LENGTH..2 * U32_LENGTH].try_into()?,
        ))?;
        let body_length = usize::try_from(u32::from_be_bytes(
            src[2 * U32_LENGTH..3 * U32_LENGTH].try_into()?,
        ))?;
        if src.len() < 3 * U32_LENGTH + header_length + body_length {
            src.reserve(3 * U32_LENGTH + header_length + body_length - src.len());
            return Ok(None);
        }

        src.advance(3 * U32_LENGTH);
        let header = acproto::PacketHeader::decode(src.split_to(header_length))?;
        self.app_id = header.app_id;
        self.header_seq_id = header.seq_id;
        let body = src.split_to(body_length);
        let decrypted: Vec<u8>;
        let payload = match header.encryption_mode() {
            acproto::packet_header::EncryptionMode::KEncryptionNone => body.as_ref(),
            acproto::packet_header::EncryptionMode::KEncryptionServiceToken => {
                decrypted = decrypt(&body, &self.security_key)?;
                decrypted.as_slice()
            }
            acproto::packet_header::EncryptionMode::KEncryptionSessionKey => {
                decrypted = if let Some(key) = &self.session_key {
                    decrypt(&body, key)?
                } else {
                    return Err(Error::NoSessionKey);
                };
                decrypted.as_slice()
            }
        };
        let payload_len = usize::try_from(header.decoded_payload_len)?;
        if payload.len() != payload_len {
            return Err(Error::ProtoDataLengthError(
                "payload length",
                payload_len,
                payload.len(),
            ));
        }
        let stream = acproto::DownstreamPayload::decode(payload)?;

        self.danmaku(&stream)
    }
}

impl TryFrom<DanmakuToken> for DanmakuProto {
    type Error = Error;

    #[inline]
    fn try_from(token: DanmakuToken) -> Result<Self> {
        Ok(Self {
            user_id: token.user_id,
            liver_uid: token.liver_uid,
            security_key: base64::decode(&token.security_key)?,
            service_token: token.service_token,
            live_id: token.live_id,
            enter_room_attach: token.enter_room_attach,
            tickets: token.tickets,
            seq_id: 1,
            header_seq_id: 1,
            ..Default::default()
        })
    }
}

pub(crate) trait Generate {
    fn generate(proto: &mut DanmakuProto) -> (acproto::PacketHeader, acproto::UpstreamPayload);
}

impl Generate for acproto::RegisterRequest {
    fn generate(proto: &mut DanmakuProto) -> (acproto::PacketHeader, acproto::UpstreamPayload) {
        let register = Self {
            app_info: Some(acproto::AppInfo {
                sdk_version: CLIENT_LIVE_SDK_VERSION.to_string(),
                link_version: LINK_VERSION.to_string(),
                ..Default::default()
            }),
            device_info: Some(acproto::DeviceInfo {
                platform_type: acproto::device_info::PlatformType::H5Windows.into(),
                device_model: DEVICE_MODEL.to_string(),
                ..Default::default()
            }),
            presence_status: acproto::register_request::PresenceStatus::KPresenceOnline.into(),
            app_active_status: acproto::register_request::ActiveStatus::KAppInForeground.into(),
            instance_id: proto.instance_id,
            zt_common_info: Some(acproto::ZtCommonInfo {
                kpn: KPN.into(),
                kpf: KPF.into(),
                uid: proto.user_id,
                ..Default::default()
            }),
            ..Default::default()
        };

        let payload = proto.payload(REGISTER.to_string(), Some(register.encode_to_vec()));
        let mut header = proto.header();
        header.encryption_mode =
            acproto::packet_header::EncryptionMode::KEncryptionServiceToken.into();
        header.token_info = Some(acproto::TokenInfo {
            token_type: acproto::token_info::TokenType::KServiceToken.into(),
            token: proto.service_token.as_str().into(),
        });
        proto.seq_id += 1;

        (header, payload)
    }
}

impl Generate for acproto::UnregisterRequest {
    #[inline]
    fn generate(proto: &mut DanmakuProto) -> (acproto::PacketHeader, acproto::UpstreamPayload) {
        let payload = proto.payload(UNREGISTER.to_string(), None);
        let header = proto.header();

        (header, payload)
    }
}

impl Generate for acproto::ZtLiveCsEnterRoom {
    fn generate(proto: &mut DanmakuProto) -> (acproto::PacketHeader, acproto::UpstreamPayload) {
        let enter = Self {
            enter_room_attach: proto.enter_room_attach.clone(),
            client_live_sdk_version: CLIENT_LIVE_SDK_VERSION.to_string(),
            ..Default::default()
        };

        let cmd = proto.command(ENTER_ROOM.to_string(), Some(enter.encode_to_vec()));
        let payload = proto.payload(GLOBAL_CS_CMD.to_string(), Some(cmd));
        let header = proto.header();
        proto.seq_id += 1;

        (header, payload)
    }
}

impl Generate for acproto::KeepAliveRequest {
    fn generate(proto: &mut DanmakuProto) -> (acproto::PacketHeader, acproto::UpstreamPayload) {
        let keep_alive = Self {
            presence_status: acproto::register_request::PresenceStatus::KPresenceOnline.into(),
            app_active_status: acproto::register_request::ActiveStatus::KAppInForeground.into(),
            ..Default::default()
        };

        let payload = proto.payload(KEEP_ALIVE.to_string(), Some(keep_alive.encode_to_vec()));
        let header = proto.header();
        proto.seq_id += 1;

        (header, payload)
    }
}

impl Generate for acproto::ZtLiveScMessage {
    #[inline]
    fn generate(proto: &mut DanmakuProto) -> (acproto::PacketHeader, acproto::UpstreamPayload) {
        let payload = proto.payload(PUSH_MESSAGE.to_string(), None);
        let mut header = proto.header();
        header.seq_id = proto.header_seq_id;

        (header, payload)
    }
}

impl Generate for acproto::ZtLiveCsHeartbeat {
    fn generate(proto: &mut DanmakuProto) -> (acproto::PacketHeader, acproto::UpstreamPayload) {
        let heartbeat = Self {
            client_timestamp_ms: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
            sequence: proto.heartbeat_seq_id,
        };

        let cmd = proto.command(HEARTBEAT.to_string(), Some(heartbeat.encode_to_vec()));
        let payload = proto.payload(GLOBAL_CS_CMD.to_string(), Some(cmd));
        let header = proto.header();
        proto.heartbeat_seq_id += 1;
        proto.seq_id += 1;

        (header, payload)
    }
}

impl Generate for acproto::ZtLiveCsUserExit {
    #[inline]
    fn generate(proto: &mut DanmakuProto) -> (acproto::PacketHeader, acproto::UpstreamPayload) {
        let cmd = proto.command(USER_EXIT.to_string(), None);
        let payload = proto.payload(GLOBAL_CS_CMD.to_string(), Some(cmd));
        let header = proto.header();
        proto.seq_id += 1;

        (header, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_and_decrypt() -> Result<()> {
        let key = base64::decode("giEyDh9ECKoxyj6kID4eXg==")?;
        let cipher_text = encrypt(b"hello", &key)?;
        let plain_text = decrypt(cipher_text.as_ref(), &key)?;
        assert_eq!(plain_text, b"hello");

        Ok(())
    }
}
