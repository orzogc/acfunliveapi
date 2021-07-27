use crate::{acfun::*, kuaishou::*, response::*, Error, Rest, Result};
use cookie::Cookie;
use core::str;
use pretend::{http::header::SET_COOKIE, resolver::UrlResolver, Pretend, Response, Url};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[cfg(feature = "default_http_client")]
use crate::http::HttpClient;

const ACFUN_ID: &str = "https://id.app.acfun.cn/";
const ACFUN_LIVE: &str = "https://live.acfun.cn/";
const KUAISHOU_ZT: &str = "https://api.kuaishouzt.com/";
//const ACFUN_MEMBER: &str = "https://member.acfun.cn/";

pub type Cookies = String;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum AcFunToken {
    Visitor(VisitorToken),
    User(UserToken),
}

#[derive(Clone, Debug)]
struct Clients<C> {
    acfun_id: Pretend<C, UrlResolver>,
    acfun_live: Pretend<C, UrlResolver>,
    kuaishou_zt: Pretend<C, UrlResolver>,
    //acfun_member: Pretend<C, UrlResolver>,
}

impl<C: Clone> Clients<C> {
    #[inline]
    fn new(client: C) -> Result<Self> {
        Ok(Self {
            acfun_id: Pretend::for_client(client.clone()).with_url(Url::parse(ACFUN_ID)?),
            acfun_live: Pretend::for_client(client.clone()).with_url(Url::parse(ACFUN_LIVE)?),
            kuaishou_zt: Pretend::for_client(client).with_url(Url::parse(KUAISHOU_ZT)?),
            //acfun_member: Pretend::for_client(client.clone()).with_url(Url::parse(ACFUN_MEMBER)?),
        })
    }
}

#[cfg(feature = "default_http_client")]
impl Clients<HttpClient> {
    #[inline]
    fn default_clients() -> Result<Self> {
        Self::new(HttpClient::default_client()?)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ApiToken {
    pub user_id: i64,
    pub security_key: String,
    pub service_token: String,
    pub device_id: String,
    pub cookies: Option<Cookies>,
}

impl ApiToken {
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
    pub bitrate: i32,
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

#[derive(Clone, Debug)]
pub struct ApiClient<C> {
    clients: Clients<C>,
    token: ApiToken,
    live: Option<Live>,
    user_id_string: String,
}

#[cfg(feature = "default_http_client")]
impl ApiClient<HttpClient> {
    #[inline]
    fn default_client() -> Result<Self> {
        Ok(Self {
            clients: Clients::default_clients()?,
            token: ApiToken::default(),
            live: None,
            user_id_string: String::new(),
        })
    }
}

impl<C> ApiClient<C> {
    #[inline]
    fn new(client: C) -> Result<Self>
    where
        C: Clone,
    {
        Ok(Self {
            clients: Clients::new(client)?,
            token: ApiToken::default(),
            live: None,
            user_id_string: String::new(),
        })
    }

    #[inline]
    pub fn set_token(&mut self, token: ApiToken) -> &mut Self {
        self.user_id_string = token.user_id.to_string();
        self.token = token;
        self
    }

    #[inline]
    pub fn token(&self) -> &ApiToken {
        &self.token
    }

    #[inline]
    pub fn token_mut(&mut self) -> &mut ApiToken {
        &mut self.token
    }

    #[inline]
    pub fn set_live(&mut self, live: Live) -> &mut Self {
        self.live = Some(live);
        self
    }

    #[inline]
    pub fn live(&self) -> Option<&Live> {
        self.live.as_ref()
    }

    #[inline]
    pub fn live_mut(&mut self) -> Option<&mut Live> {
        self.live.as_mut()
    }

    #[inline]
    pub fn has_live(&self) -> bool {
        self.live.is_some()
    }

    #[inline]
    pub fn acfun_id(&self) -> &Pretend<C, UrlResolver> {
        &self.clients.acfun_id
    }

    #[inline]
    pub fn acfun_live(&self) -> &Pretend<C, UrlResolver> {
        &self.clients.acfun_live
    }

    #[inline]
    pub fn kuaishou_zt(&self) -> &Pretend<C, UrlResolver> {
        &self.clients.kuaishou_zt
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
    pub fn liver_uid(&self) -> Option<i64> {
        self.live().map(|l| l.liver_uid)
    }

    #[inline]
    pub fn live_id(&self) -> Option<&str> {
        self.live().map(|l| l.live_id.as_str())
    }

    #[inline]
    fn ks_query(&self) -> KsQuery {
        if self.is_visitor() {
            KsQuery::visitor(
                &self.user_id_string,
                &self.token.device_id,
                &self.token.service_token,
            )
        } else {
            KsQuery::user(
                &self.user_id_string,
                &self.token.device_id,
                &self.token.service_token,
            )
        }
    }

    #[inline]
    fn ks_form<'a>(&self, live_id: &'a str) -> KsForm<'a> {
        KsForm::new(self.token.user_id, live_id)
    }
}

impl<C> ApiClient<C>
where
    C: pretend::client::Client + Send + Sync,
{
    pub async fn user(
        &self,
        account: impl Into<Cow<'_, str>>,
        password: impl Into<Cow<'_, str>>,
    ) -> Result<(Login, Cookies)> {
        let resp: Response<_> = self
            .acfun_id()
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
        let resp: Response<_> = self.acfun_live().device_id().await?;
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
                    .acfun_id()
                    .user_token(
                        TokenForm {
                            sid: Sid::Midground,
                        },
                        c,
                    )
                    .await?
                    .value();

                Ok(AcFunToken::User(token))
            }
            None => {
                let token: VisitorToken = self
                    .acfun_id()
                    .visitor_token(TokenForm { sid: Sid::Visitor }, device_id)
                    .await?
                    .value();

                Ok(AcFunToken::Visitor(token))
            }
        }
    }

    #[inline]
    pub async fn get_live_info(&self, liver_uid: i64) -> Result<LiveInfo> {
        if liver_uid <= 0 {
            Err(Error::InvalidUid(liver_uid))
        } else if !self.is_login() {
            Err(Error::VisitorOrUserNotLogin)
        } else {
            Ok(self
                .kuaishou_zt()
                .start_play(&self.ks_query(), &StartPlayForm::new(liver_uid))
                .await?
                .value())
        }
    }

    #[inline]
    pub async fn get<T>(&self) -> Result<T>
    where
        T: Rest,
    {
        T::request(self).await
    }

    #[inline]
    pub async fn get_gift_list(&self, live_id: impl Into<Cow<'_, str>>) -> Result<GiftList> {
        let live_id = live_id.into();
        if live_id.is_empty() {
            Err(Error::EmptyLiveId)
        } else if !self.is_login() {
            Err(Error::VisitorOrUserNotLogin)
        } else {
            Ok(self
                .kuaishou_zt()
                .gift_list(&self.ks_query(), &self.ks_form(live_id.as_ref()))
                .await?
                .value())
        }
    }

    #[inline]
    pub async fn get_live_list(&self, count: u32, page: u32) -> Result<LiveList> {
        Ok(self
            .acfun_live()
            .live_list(
                count,
                page,
                self.token.cookies.as_deref().unwrap_or_default(),
            )
            .await?
            .value())
    }

    #[inline]
    pub async fn get_medal_list(&self) -> Result<MedalList> {
        if !self.is_user() {
            Err(Error::NotUser)
        } else {
            Ok(self
                .acfun_live()
                .medal_list(self.token.cookies.as_deref().unwrap_or_default())
                .await?
                .value())
        }
    }

    #[inline]
    pub async fn get_user_live_info(&self, liver_uid: i64) -> Result<UserLiveInfo> {
        if liver_uid <= 0 {
            Err(Error::InvalidUid(liver_uid))
        } else {
            Ok(self
                .acfun_live()
                .live_info(liver_uid, self.token.cookies.as_deref().unwrap_or_default())
                .await?
                .value())
        }
    }

    #[inline]
    pub async fn get_summary(&self, live_id: impl Into<Cow<'_, str>>) -> Result<Summary> {
        let live_id = live_id.into();
        if live_id.is_empty() {
            Err(Error::EmptyLiveId)
        } else if !self.is_login() {
            Err(Error::VisitorOrUserNotLogin)
        } else {
            Ok(self
                .kuaishou_zt()
                .end_summary(&self.ks_query(), &self.ks_form(live_id.as_ref()))
                .await?
                .value())
        }
    }

    #[inline]
    pub async fn get_medal_rank_list(&self, liver_uid: i64) -> Result<MedalRankList> {
        if liver_uid <= 0 {
            Err(Error::InvalidUid(liver_uid))
        } else {
            Ok(self
                .acfun_live()
                .medal_rank_list(liver_uid, self.token.cookies.as_deref().unwrap_or_default())
                .await?
                .value())
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiClientBuilder<C> {
    client: ApiClient<C>,
    account: Option<String>,
    password: Option<String>,
    liver_uid: Option<i64>,
}

#[cfg(feature = "default_http_client")]
impl ApiClientBuilder<HttpClient> {
    #[inline]
    pub fn default_client() -> Result<Self> {
        Ok(Self {
            client: ApiClient::default_client()?,
            account: None,
            password: None,
            liver_uid: None,
        })
    }
}

impl<C> ApiClientBuilder<C> {
    #[inline]
    pub fn new(client: C) -> Result<Self>
    where
        C: Clone,
    {
        Ok(Self {
            client: ApiClient::new(client)?,
            account: None,
            password: None,
            liver_uid: None,
        })
    }

    #[inline]
    pub fn user<'a>(
        mut self,
        account: impl Into<Cow<'a, str>>,
        password: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.account = Some(account.into().into_owned());
        self.password = Some(password.into().into_owned());

        self
    }

    #[inline]
    pub fn liver_uid(mut self, liver_uid: i64) -> Self {
        self.liver_uid = Some(liver_uid);

        self
    }
}

impl<C> ApiClientBuilder<C>
where
    C: pretend::client::Client + Send + Sync,
{
    pub async fn build(self) -> Result<ApiClient<C>> {
        let mut client = self.client;
        if let Some((account, password)) = self.account.zip(self.password) {
            let (login, cookies) = client.user(account, password).await?;
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
        if let Some(liver_uid) = self.liver_uid {
            let info = client.get_live_info(liver_uid).await?;
            client.live = Some(Live {
                liver_uid,
                live_id: info.data.live_id,
                tickets: info.data.available_tickets,
                enter_room_attach: info.data.enter_room_attach,
                title: info.data.caption,
                start_time: info.data.live_start_time,
                panoramic: info.data.panoramic,
                stream_name: info.data.video_play_res.stream_name,
                stream_list: info
                    .data
                    .video_play_res
                    .live_adaptive_manifest
                    .into_iter()
                    .next()
                    .ok_or(Error::IndexOutOfRange("live_adaptive_manifest", 0))?
                    .adaptation_set
                    .representation
                    .into_iter()
                    .map(|r| Stream {
                        url: r.url,
                        bitrate: r.bitrate,
                        quality_type: r.quality_type,
                        quality_name: r.name,
                    })
                    .collect(),
            });
        }

        Ok(client)
    }
}

#[cfg(feature = "default_http_client")]
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
        let client = ApiClientBuilder::default_client()?
            .liver_uid(liver_uid)
            .build()
            .await?;
        let _gifts: GiftList = client.get().await?;
        let _live_list: LiveList = client.get().await?;
        let _info = client.get_user_live_info(1).await?;
        let _info: UserLiveInfo = client.get().await?;
        let _summary: Summary = client.get().await?;
        let _medal_rank_list: MedalRankList = client.get().await?;

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
        let client = ApiClientBuilder::default_client()?
            .user(account, password)
            .liver_uid(liver_uid)
            .build()
            .await?;
        let _gifts: GiftList = client.get().await?;
        let _live_list: LiveList = client.get().await?;
        let _medal_list: MedalList = client.get().await?;
        let _info = client.get_user_live_info(1).await?;
        let _info: UserLiveInfo = client.get().await?;
        let _summary: Summary = client.get().await?;
        let _medal_rank_list: MedalRankList = client.get().await?;

        Ok(())
    }
}
