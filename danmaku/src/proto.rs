use crate::{client::DanmakuToken, global::*, Error, Result};
use aes::Aes128;
use block_modes::{block_padding::Pkcs7, BlockMode, Cbc};
use prost::Message;
use rand::{distributions::Standard, Rng};
use std::{
    convert::{TryFrom, TryInto},
    time::SystemTime,
};

pub mod acproto {
    include!(concat!(env!("OUT_DIR"), "/acproto.rs"));
}

fn encrypt(plain_text: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let mut iv: Vec<u8> = rand::thread_rng()
        .sample_iter(Standard)
        .take(aes::BLOCK_SIZE)
        .collect();
    let cipher: Cbc<Aes128, Pkcs7> = Cbc::new_from_slices(key, iv.as_slice())?;
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

#[derive(Clone, Debug, Default)]
pub(crate) struct ProtoData {
    user_id: i64,
    security_key: String,
    service_token: String,
    live_id: String,
    enter_room_attach: String,
    pub(crate) tickets: Vec<String>,
    app_id: i32,
    instance_id: i64,
    session_key: Vec<u8>,
    seq_id: i64,
    header_seq_id: i64,
    heartbeat_seq_id: i64,
    pub(crate) ticket_index: usize,
}

impl ProtoData {
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

    fn payload(&self, command: String, paload_data: Option<Vec<u8>>) -> Result<Vec<u8>> {
        let mut pl = acproto::UpstreamPayload {
            command,
            seq_id: self.seq_id,
            retry_count: RETRY_COUNT,
            sub_biz: SUB_BIZ.to_string(),
            ..Default::default()
        };
        if let Some(data) = paload_data {
            pl.payload_data = data;
        }
        let mut buf = Vec::new();
        pl.encode(&mut buf)?;

        Ok(buf)
    }

    fn command(&self, command: String, paload: Option<Vec<u8>>) -> Result<Vec<u8>> {
        let mut cmd = acproto::ZtLiveCsCmd {
            cmd_type: command,
            ticket: self
                .tickets
                .get(self.ticket_index)
                .ok_or(Error::IndexOutOfRange("tickets", self.ticket_index))?
                .clone(),
            live_id: self.live_id.clone(),
            ..Default::default()
        };
        if let Some(data) = paload {
            cmd.payload = data;
        }
        let mut buf = Vec::new();
        cmd.encode(&mut buf)?;

        Ok(buf)
    }

    // https://github.com/wpscott/AcFunDanmaku/tree/master/AcFunDanmu
    fn encode(&self, header: &mut acproto::PacketHeader, payload: &[u8]) -> Result<Vec<u8>> {
        header.decoded_payload_len = u32::try_from(payload.len())?;
        let mut header_buf = Vec::new();
        header.encode(&mut header_buf)?;

        let security_key: Vec<u8>;
        let key = if header.encryption_mode()
            == acproto::packet_header::EncryptionMode::KEncryptionServiceToken
        {
            security_key = base64::decode(self.security_key.as_str())?;
            security_key.as_slice()
        } else {
            self.session_key.as_slice()
        };
        let mut encrypted = encrypt(payload, key)?;

        let mut data = 0xABCD0001u32.to_be_bytes().to_vec();
        data.extend_from_slice(&(u32::try_from(header_buf.len())?).to_be_bytes());
        data.extend_from_slice(&(u32::try_from(encrypted.len())?).to_be_bytes());
        data.append(&mut header_buf);
        data.append(&mut encrypted);

        Ok(data)
    }

    // https://github.com/wpscott/AcFunDanmaku/tree/master/AcFunDanmu
    pub(crate) fn decode(&mut self, data: &[u8]) -> Result<acproto::DownstreamPayload> {
        let header_length = usize::try_from(u32::from_be_bytes(
            data.get(4..8)
                .ok_or_else(|| Error::ProtoDataLengthError("header length", 4, data.len() - 4))?
                .try_into()?,
        ))?;
        let body_length = usize::try_from(u32::from_be_bytes(
            data.get(8..12)
                .ok_or_else(|| Error::ProtoDataLengthError("body length", 4, data.len() - 8))?
                .try_into()?,
        ))?;
        let header =
            acproto::PacketHeader::decode(data.get(12..(12 + header_length)).ok_or_else(
                || Error::ProtoDataLengthError("header", header_length, data.len() - 12),
            )?)?;
        let body = data
            .get((12 + header_length)..(12 + header_length + body_length))
            .ok_or_else(|| {
                Error::ProtoDataLengthError("body", body_length, data.len() - 12 - header_length)
            })?;
        if data.len() != 12 + header_length + body_length {
            return Err(Error::ProtoDataLengthError(
                "data length",
                12 + header_length + body_length,
                data.len(),
            ));
        }

        self.app_id = header.app_id;
        self.header_seq_id = header.seq_id;

        let decrypted: Vec<u8>;
        let payload = match header.encryption_mode() {
            acproto::packet_header::EncryptionMode::KEncryptionNone => body,
            acproto::packet_header::EncryptionMode::KEncryptionServiceToken => {
                decrypted = decrypt(body, base64::decode(self.security_key.as_str())?.as_slice())?;
                decrypted.as_slice()
            }
            acproto::packet_header::EncryptionMode::KEncryptionSessionKey => {
                decrypted = decrypt(body, self.session_key.as_slice())?;
                decrypted.as_slice()
            }
        };
        if payload.len() != usize::try_from(header.decoded_payload_len)? {
            return Err(Error::ProtoDataLengthError(
                "payload length",
                usize::try_from(header.decoded_payload_len)?,
                payload.len(),
            ));
        }

        Ok(acproto::DownstreamPayload::decode(payload)?)
    }

    pub(crate) fn register_response(&mut self, payload: &acproto::DownstreamPayload) -> Result<()> {
        let resp = acproto::RegisterResponse::decode(payload.payload_data.as_slice())?;
        self.instance_id = resp.instance_id;
        self.session_key = resp.sess_key;

        Ok(())
    }
}

impl From<DanmakuToken> for ProtoData {
    #[inline]
    fn from(token: DanmakuToken) -> Self {
        Self {
            user_id: token.user_id,
            security_key: token.security_key,
            service_token: token.service_token,
            live_id: token.live_id,
            enter_room_attach: token.enter_room_attach,
            tickets: token.tickets,
            seq_id: 1,
            header_seq_id: 1,
            ..Default::default()
        }
    }
}

pub(crate) trait Generate {
    fn generate(data: &mut ProtoData) -> Result<Vec<u8>>;
}

impl Generate for acproto::RegisterRequest {
    fn generate(data: &mut ProtoData) -> Result<Vec<u8>> {
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
            instance_id: data.instance_id,
            zt_common_info: Some(acproto::ZtCommonInfo {
                kpn: KPN.into(),
                kpf: KPF.into(),
                uid: data.user_id,
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut buf = Vec::new();
        register.encode(&mut buf)?;
        let payload = data.payload(REGISTER.to_string(), Some(buf))?;
        let mut header = data.header();
        header.encryption_mode =
            acproto::packet_header::EncryptionMode::KEncryptionServiceToken.into();
        header.token_info = Some(acproto::TokenInfo {
            token_type: acproto::token_info::TokenType::KServiceToken.into(),
            token: data.service_token.as_str().into(),
        });
        data.seq_id += 1;

        data.encode(&mut header, payload.as_slice())
    }
}

impl Generate for acproto::UnregisterRequest {
    fn generate(data: &mut ProtoData) -> Result<Vec<u8>> {
        let payload = data.payload(UNREGISTER.to_string(), None)?;
        let mut header = data.header();

        data.encode(&mut header, payload.as_slice())
    }
}

impl Generate for acproto::ZtLiveCsEnterRoom {
    fn generate(data: &mut ProtoData) -> Result<Vec<u8>> {
        let enter = Self {
            enter_room_attach: data.enter_room_attach.clone(),
            client_live_sdk_version: CLIENT_LIVE_SDK_VERSION.to_string(),
            ..Default::default()
        };
        let mut buf = Vec::new();
        enter.encode(&mut buf)?;

        let cmd = data.command(ENTER_ROOM.to_string(), Some(buf))?;
        let payload = data.payload(GLOBAL_CS_CMD.to_string(), Some(cmd))?;
        let mut header = data.header();
        data.seq_id += 1;

        data.encode(&mut header, payload.as_slice())
    }
}

impl Generate for acproto::KeepAliveRequest {
    fn generate(data: &mut ProtoData) -> Result<Vec<u8>> {
        let keep_alive = Self {
            presence_status: acproto::register_request::PresenceStatus::KPresenceOnline.into(),
            app_active_status: acproto::register_request::ActiveStatus::KAppInForeground.into(),
            ..Default::default()
        };
        let mut buf = Vec::new();
        keep_alive.encode(&mut buf)?;

        let payload = data.payload(KEEP_ALIVE.to_string(), Some(buf))?;
        let mut header = data.header();
        data.seq_id += 1;

        data.encode(&mut header, payload.as_slice())
    }
}

impl Generate for acproto::ZtLiveScMessage {
    fn generate(data: &mut ProtoData) -> Result<Vec<u8>> {
        let payload = data.payload(PUSH_MESSAGE.to_string(), None)?;
        let mut header = data.header();
        header.seq_id = data.header_seq_id;

        data.encode(&mut header, payload.as_slice())
    }
}

impl Generate for acproto::ZtLiveCsHeartbeat {
    fn generate(data: &mut ProtoData) -> Result<Vec<u8>> {
        let heartbeat = Self {
            client_timestamp_ms: i64::try_from(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_millis(),
            )?,
            sequence: data.heartbeat_seq_id,
        };
        let mut buf = Vec::new();
        heartbeat.encode(&mut buf)?;

        let cmd = data.command(HEARTBEAT.to_string(), Some(buf))?;
        let payload = data.payload(GLOBAL_CS_CMD.to_string(), Some(cmd))?;
        let mut header = data.header();
        data.heartbeat_seq_id += 1;
        data.seq_id += 1;

        data.encode(&mut header, payload.as_slice())
    }
}

impl Generate for acproto::ZtLiveCsUserExit {
    fn generate(data: &mut ProtoData) -> Result<Vec<u8>> {
        let cmd = data.command(USER_EXIT.to_string(), None)?;
        let payload = data.payload(GLOBAL_CS_CMD.to_string(), Some(cmd))?;
        let mut header = data.header();
        data.seq_id += 1;

        data.encode(&mut header, payload.as_slice())
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
