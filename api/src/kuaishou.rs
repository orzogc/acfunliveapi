use crate::response::*;
use pretend::{pretend, Json, Result};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KsQuery<'a>([(&'a str, &'a str); 6]);

impl<'a> KsQuery<'a> {
    pub(crate) fn visitor(user_id: &'a str, device_id: &'a str, service_token: &'a str) -> Self {
        Self([
            ("subBiz", "mainApp"),
            ("kpn", "ACFUN_APP"),
            ("kpf", "PC_WEB"),
            ("userId", user_id),
            ("did", device_id),
            ("acfun.api.visitor_st", service_token),
        ])
    }

    pub(crate) fn user(user_id: &'a str, device_id: &'a str, service_token: &'a str) -> Self {
        Self([
            ("subBiz", "mainApp"),
            ("kpn", "ACFUN_APP"),
            ("kpf", "PC_WEB"),
            ("userId", user_id),
            ("did", device_id),
            ("acfun.midground.api_st", service_token),
        ])
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KsForm<'a> {
    visitor_id: i64,
    live_id: &'a str,
}

impl<'a> KsForm<'a> {
    #[inline]
    pub(crate) fn new(user_id: i64, live_id: &'a str) -> Self {
        Self {
            visitor_id: user_id,
            live_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartPlayForm<'a> {
    author_id: i64,
    pull_stream_type: &'a str,
}

impl<'a> StartPlayForm<'a> {
    #[inline]
    pub(crate) fn new(liver_uid: i64) -> Self {
        Self {
            author_id: liver_uid,
            pull_stream_type: "FLV",
        }
    }
}

#[pretend]
pub(crate) trait KuaishouZt {
    #[request(method = "POST", path = "/rest/zt/live/web/startPlay")]
    #[header(name = "Referer", value = "https://live.acfun.cn/")]
    async fn start_play(&self, query: &KsQuery, form: &StartPlayForm) -> Result<Json<LiveInfo>>;

    #[request(method = "POST", path = "/rest/zt/live/web/gift/list")]
    #[header(name = "Referer", value = "https://live.acfun.cn/")]
    async fn gift_list(&self, query: &KsQuery, form: &KsForm) -> Result<Json<GiftList>>;

    #[request(method = "POST", path = "/rest/zt/live/web/endSummary")]
    #[header(name = "Referer", value = "https://live.acfun.cn/")]
    async fn end_summary(&self, query: &KsQuery, form: &KsForm) -> Result<Json<Summary>>;
}
