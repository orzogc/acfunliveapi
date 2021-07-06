use crate::response::*;
use pretend::{header, pretend, request, Json, Response, Result};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct LoginForm<'a> {
    username: &'a str,
    password: &'a str,
    key: &'a str,
    captcha: &'a str,
}

impl<'a> LoginForm<'a> {
    #[inline]
    pub(crate) fn new(username: &'a str, password: &'a str) -> Self {
        Self {
            username,
            password,
            key: "",
            captcha: "",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) enum Sid {
    #[serde(rename = "acfun.api.visitor")]
    Visitor,
    #[serde(rename = "acfun.midground.api")]
    Midground,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct TokenForm {
    pub(crate) sid: Sid,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct LiveListForm {
    pub(crate) count: u32,
    pub(crate) pcursor: u32,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveInfoForm {
    pub(crate) author_id: i64,
}

#[pretend]
pub(crate) trait AcFunId {
    #[request(method = "POST", path = "/rest/web/login/signin")]
    async fn login(&self, form: &LoginForm) -> Result<Response<Json<Login>>>;

    #[request(method = "POST", path = "/rest/app/visitor/login")]
    #[header(name = "Cookie", value = "_did={device_id}")]
    async fn visitor_token(&self, form: TokenForm, device_id: &str) -> Result<Json<VisitorToken>>;

    #[request(method = "POST", path = "/rest/web/token/get")]
    #[header(name = "Cookie", value = "{cookie}")]
    async fn user_token(&self, form: TokenForm, cookie: &str) -> Result<Json<UserToken>>;
}

#[pretend]
pub(crate) trait AcFunLive {
    #[request(method = "GET", path = "/")]
    async fn device_id(&self) -> Result<Response<()>>;

    #[request(method = "POST", path = "/rest/pc-direct/live/channel")]
    #[header(name = "Cookie", value = "{cookie}")]
    async fn live_list(&self, form: LiveListForm, cookie: &str) -> Result<Json<LiveList>>;

    #[request(method = "GET", path = "/rest/pc-direct/fansClub/fans/medal/list")]
    #[header(name = "Cookie", value = "{cookie}")]
    async fn medal_list(&self, cookie: &str) -> Result<Json<MedalList>>;

    #[request(method = "POST", path = "/rest/pc-direct/live/info")]
    async fn live_info(&self, form: LiveInfoForm) -> Result<Json<UserLiveInfo>>;
}
