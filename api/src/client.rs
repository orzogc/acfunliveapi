use crate::{acfun::*, kuaishou::*, response::*, Error, Rest, Result};
use cookie::Cookie;
use core::str;
use pretend::{http::header::SET_COOKIE, resolver::UrlResolver, Pretend, Response, Url};
use pretend_reqwest::Client as PRClient;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, time::Duration};

const ACFUN_ID: &str = "https://id.app.acfun.cn/";
const ACFUN_LIVE: &str = "https://live.acfun.cn/";
//const ACFUN_API: &str = "https://api-new.app.acfun.cn/";
//const ACFUN_MEMBER: &str = "https://member.acfun.cn/";
const KUAISHOU_ZT: &str = "https://api.kuaishouzt.com/";

pub type Cookies = String;

#[inline]
fn default_reqwest_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .gzip(true)
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(120))
        .use_rustls_tls()
        .no_trust_dns()
        .https_only(true)
        .build()?)
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum AcFunToken {
    Visitor(VisitorToken),
    User(UserToken),
}

struct Clients<C> {
    acfun_id: Pretend<C, UrlResolver>,
    acfun_live: Pretend<C, UrlResolver>,
    //acfun_api: Pretend<C, UrlResolver>,
    //acfun_member: Pretend<C, UrlResolver>,
    kuaishou_zt: Pretend<C, UrlResolver>,
}

impl<C> Clients<C> {
    fn new(client: C) -> Result<Self>
    where
        C: Clone,
    {
        Ok(Self {
            acfun_id: Pretend::for_client(client.clone()).with_url(Url::parse(ACFUN_ID)?),
            acfun_live: Pretend::for_client(client.clone()).with_url(Url::parse(ACFUN_LIVE)?),
            //acfun_api: Pretend::for_client(client.clone()).with_url(Url::parse(ACFUN_API)?),
            //acfun_member: Pretend::for_client(client.clone()).with_url(Url::parse(ACFUN_MEMBER)?),
            kuaishou_zt: Pretend::for_client(client).with_url(Url::parse(KUAISHOU_ZT)?),
        })
    }
}

impl Clients<PRClient> {
    fn default_clients() -> Result<Self> {
        let client = default_reqwest_client()?;

        Ok(Self {
            acfun_id: Pretend::for_client(PRClient::new(client.clone()))
                .with_url(Url::parse(ACFUN_ID)?),
            acfun_live: Pretend::for_client(PRClient::new(client.clone()))
                .with_url(Url::parse(ACFUN_LIVE)?),
            //acfun_api: Pretend::for_client(PRClient::new(client.clone())).with_url(Url::parse(ACFUN_API)?),
            //acfun_member: Pretend::for_client(PRClient::new(client.clone())).with_url(Url::parse(ACFUN_MEMBER)?),
            kuaishou_zt: Pretend::for_client(PRClient::new(client))
                .with_url(Url::parse(KUAISHOU_ZT)?),
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Token {
    pub user_id: i64,
    pub security_key: String,
    pub service_token: String,
    pub device_id: String,
    pub cookies: Option<Cookies>,
}

impl Token {
    #[inline]
    pub fn is_login(&self) -> bool {
        !(self.user_id == 0
            || self.security_key.is_empty()
            || self.service_token.is_empty()
            || self.device_id.is_empty())
    }

    #[inline]
    pub fn is_visitor(&self) -> bool {
        self.cookies.is_none()
    }

    #[inline]
    pub fn is_user(&self) -> bool {
        self.cookies.is_some()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Stream {
    pub url: String,
    pub bitrate: i64,
    pub quality_type: String,
    pub quality_name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Live {
    pub liver_uid: i64,
    pub live_id: String,
    pub tickets: Vec<String>,
    pub enter_room_attach: String,
    pub title: String,
    pub start_time: i64,
    pub panoramic: bool,
    pub stream_name: String,
    pub stream_list: Vec<Stream>,
}

pub struct Client<C> {
    clients: Clients<C>,
    token: Token,
    live: Live,
    user_id_string: String,
}

impl Client<PRClient> {
    #[inline]
    fn default_client() -> Result<Self> {
        Ok(Self {
            clients: Clients::default_clients()?,
            token: Token::default(),
            live: Live::default(),
            user_id_string: String::new(),
        })
    }
}

impl<C> Client<C> {
    #[inline]
    fn new(client: C) -> Result<Self>
    where
        C: Clone,
    {
        Ok(Self {
            clients: Clients::new(client)?,
            token: Token::default(),
            live: Live::default(),
            user_id_string: String::new(),
        })
    }

    #[inline]
    pub fn set_token(&mut self, token: Token) -> &mut Self {
        self.user_id_string = token.user_id.to_string();
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
    pub fn set_live(&mut self, live: Live) -> &mut Self {
        self.live = live;
        self
    }

    #[inline]
    pub fn live(&self) -> &Live {
        &self.live
    }

    #[inline]
    pub fn live_mut(&mut self) -> &mut Live {
        &mut self.live
    }

    #[inline]
    pub fn is_login(&self) -> bool {
        self.token.is_login()
    }

    #[inline]
    pub fn is_visitor(&self) -> bool {
        self.token.is_visitor()
    }

    #[inline]
    pub fn is_user(&self) -> bool {
        self.token.is_user()
    }

    #[inline]
    pub fn user_id(&self) -> i64 {
        self.token.user_id
    }

    #[inline]
    pub fn liver_uid(&self) -> i64 {
        self.live.liver_uid
    }

    #[inline]
    pub fn live_id(&self) -> &str {
        self.live.live_id.as_str()
    }

    #[inline]
    fn ks_query(&self) -> KsQuery {
        if self.is_visitor() {
            KsQuery::visitor(
                self.user_id_string.as_str(),
                self.token.device_id.as_str(),
                self.token.service_token.as_str(),
            )
        } else {
            KsQuery::user(
                self.user_id_string.as_str(),
                self.token.device_id.as_str(),
                self.token.service_token.as_str(),
            )
        }
    }

    #[inline]
    fn ks_form<'a>(&self, live_id: &'a str) -> KsForm<'a> {
        KsForm::new(self.token.user_id, live_id)
    }
}

impl<C> Client<C>
where
    C: pretend::client::Client + Send + Sync,
{
    pub async fn login(
        &self,
        account: impl Into<Cow<'_, str>>,
        password: impl Into<Cow<'_, str>>,
    ) -> Result<(Login, Cookies)> {
        let resp: Response<_> = self
            .clients
            .acfun_id
            .login(&LoginForm::new(&account.into(), &password.into()))
            .await?;

        let cookies = resp
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .map(|v| {
                v.to_str()
                    .map_err(Error::HeaderToStrError)
                    .and_then(|s| Cookie::parse(s).map_err(Error::ParseCookieError))
                    .map(|c| format!("{}={}", c.name(), c.value()))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?
            .join("; ");
        let login: Login = resp.into_body().value();

        Ok((login, cookies))
    }

    pub async fn get_device_id(&self) -> Result<String> {
        let resp: Response<_> = self.clients.acfun_live.device_id().await?;
        let did = resp
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .map(|v| {
                v.to_str()
                    .map_err(Error::HeaderToStrError)
                    .and_then(|s| Cookie::parse(s).map_err(Error::ParseCookieError))
            })
            .find(|c| {
                if let Ok(c) = c {
                    c.name() == "_did"
                } else {
                    false
                }
            })
            .ok_or(Error::GetDidFailed)??
            .value()
            .to_string();

        Ok(did)
    }

    pub async fn get_acfun_token(&self) -> Result<AcFunToken> {
        let did: String;
        let device_id = if self.token.device_id.is_empty() {
            did = self.get_device_id().await?;
            did.as_str()
        } else {
            self.token.device_id.as_str()
        };

        match &self.token.cookies {
            Some(c) => {
                let token: UserToken = self
                    .clients
                    .acfun_id
                    .user_token(
                        &TokenForm {
                            sid: Sid::Midground,
                        },
                        c.as_str(),
                    )
                    .await?
                    .value();

                Ok(AcFunToken::User(token))
            }
            None => {
                let token: VisitorToken = self
                    .clients
                    .acfun_id
                    .visitor_token(&TokenForm { sid: Sid::Visitor }, device_id)
                    .await?
                    .value();

                Ok(AcFunToken::Visitor(token))
            }
        }
    }

    pub async fn get_live_info(&self, liver_uid: i64) -> Result<LiveInfo> {
        if liver_uid <= 0 {
            return Err(Error::InvalidUid(liver_uid));
        }
        if !self.is_login() {
            return Err(Error::NoLogin);
        }

        let mut info: LiveInfo = self
            .clients
            .kuaishou_zt
            .start_play(&self.ks_query(), &StartPlayForm::new(liver_uid))
            .await?
            .value();
        info.data.stream_info = serde_json::from_str(info.data.video_play_res.as_str())?;
        info.data.stream_info.live_adaptive_config =
            serde_json::from_str(info.data.stream_info.live_adaptive_config_string.as_str())?;

        Ok(info)
    }

    #[inline]
    pub async fn get<T>(&self) -> Result<T>
    where
        T: Rest,
    {
        T::request(self).await
    }

    pub async fn get_gift_list(&self, live_id: impl Into<Cow<'_, str>>) -> Result<Gift> {
        let live_id = live_id.into();
        if live_id.is_empty() {
            Err(Error::EmptyLiveId)
        } else if !self.is_login() {
            Err(Error::NoLogin)
        } else {
            Ok(self
                .clients
                .kuaishou_zt
                .gift_list(&self.ks_query(), &self.ks_form(live_id.as_ref()))
                .await?
                .value())
        }
    }
}

pub struct ClientBuilder<C> {
    client: Client<C>,
    account: String,
    password: String,
    liver_uid: i64,
}

impl ClientBuilder<PRClient> {
    #[inline]
    pub fn default_client() -> Result<Self> {
        Ok(Self {
            client: Client::default_client()?,
            account: String::new(),
            password: String::new(),
            liver_uid: 0,
        })
    }
}

impl<C> ClientBuilder<C> {
    #[inline]
    pub fn new(client: C) -> Result<Self>
    where
        C: Clone,
    {
        Ok(Self {
            client: Client::new(client)?,
            account: String::new(),
            password: String::new(),
            liver_uid: 0,
        })
    }

    #[inline]
    pub fn login<'a>(
        mut self,
        account: impl Into<Cow<'a, str>>,
        password: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.account = account.into().into_owned();
        self.password = password.into().into_owned();

        self
    }

    #[inline]
    pub fn liver_uid(mut self, liver_uid: i64) -> Self {
        self.liver_uid = liver_uid;

        self
    }
}

impl<C> ClientBuilder<C>
where
    C: pretend::client::Client + Send + Sync,
{
    pub async fn build(self) -> Result<Client<C>> {
        let mut client = self.client;
        if !(self.account.is_empty() || self.password.is_empty()) {
            let (login, cookies) = client.login(self.account, self.password).await?;
            client.token.user_id = login.user_id;
            client.user_id_string = login.user_id.to_string();
            client.token.cookies = Some(cookies);
        }
        client.token.device_id = client.get_device_id().await?;
        match client.get_acfun_token().await? {
            AcFunToken::Visitor(token) => {
                client.token.user_id = token.user_id;
                client.user_id_string = token.user_id.to_string();
                client.token.security_key = token.ac_security;
                client.token.service_token = token.acfun_api_visitor_st;
            }
            AcFunToken::User(token) => {
                client.token.security_key = token.ssecurity;
                client.token.service_token = token.acfun_midground_api_st;
            }
        }
        if self.liver_uid != 0 {
            let mut info = client.get_live_info(self.liver_uid).await?;
            client.live = Live {
                liver_uid: self.liver_uid,
                live_id: info.data.live_id,
                tickets: info.data.available_tickets,
                enter_room_attach: info.data.enter_room_attach,
                title: info.data.caption,
                start_time: info.data.live_start_time,
                panoramic: info.data.panoramic,
                stream_name: info.data.stream_info.stream_name,
                stream_list: Vec::new(),
            };
            info.data
                .stream_info
                .live_adaptive_manifest
                .get_mut(0)
                .ok_or(Error::IndexOutOfRange("live_adaptive_manifest", 0))?
                .adaptation_set
                .representation
                .iter_mut()
                .for_each(|r| {
                    client.live.stream_list.push(Stream {
                        url: std::mem::take(&mut r.url),
                        bitrate: r.bitrate,
                        quality_type: std::mem::take(&mut r.quality_type),
                        quality_name: std::mem::take(&mut r.name),
                    })
                });
        }

        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_visitor() -> Result<()> {
        let liver_uid: i64 = env::var("LIVER_UID")
            .expect("need to set the LIVER_UID environment variable to the liver's uid")
            .parse()
            .expect("LIVER_UID should be an integer");
        let client = ClientBuilder::default_client()?
            .liver_uid(liver_uid)
            .build()
            .await?;
        println!("{:?}", client.token());
        //println!("{:?}", client.get_live());
        let _gift: Gift = client.get().await?;
        //println!("{:?}", _gift);

        Ok(())
    }

    #[tokio::test]
    async fn test_user() -> Result<()> {
        let account = env::var("ACCOUNT")
            .expect("need to set the ACCOUNT environment variable to the AcFun account");
        let password = env::var("PASSWORD").expect(
            "need to set the PASSWORD environment variable to the AcFun account's password",
        );
        let liver_uid: i64 = env::var("LIVER_UID")
            .expect("need to set the LIVER_UID environment variable to the liver's uid")
            .parse()
            .expect("LIVER_UID should be an integer");
        let client = ClientBuilder::default_client()?
            .login(account, password)
            .liver_uid(liver_uid)
            .build()
            .await?;
        println!("{:?}", client.token());
        //println!("{:?}", client.get_live());
        let _gift: Gift = client.get().await?;
        //println!("{:?}", _gift);

        Ok(())
    }
}
