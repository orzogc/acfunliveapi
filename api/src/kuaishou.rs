use crate::response::*;
use pretend::{header, pretend, request, Json, Result};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub(crate) struct KsQuery<'a>(HashMap<&'a str, &'a str>);

impl<'a> KsQuery<'a> {
    pub(crate) fn visitor(user_id: &'a str, device_id: &'a str, service_token: &'a str) -> Self {
        let mut query = HashMap::with_capacity(6);
        query.insert("subBiz", "mainApp");
        query.insert("kpn", "ACFUN_APP");
        query.insert("kpf", "PC_WEB");
        query.insert("userId", user_id);
        query.insert("did", device_id);
        query.insert("acfun.api.visitor_st", service_token);

        Self(query)
    }

    pub(crate) fn user(user_id: &'a str, device_id: &'a str, service_token: &'a str) -> Self {
        let mut query = HashMap::with_capacity(6);
        query.insert("subBiz", "mainApp");
        query.insert("kpn", "ACFUN_APP");
        query.insert("kpf", "PC_WEB");
        query.insert("userId", user_id);
        query.insert("did", device_id);
        query.insert("acfun.midground.api_st", service_token);

        Self(query)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KsForm<'a> {
    visitor_id: i64,
    live_id: &'a str,
}

impl<'a> KsForm<'a> {
    pub(crate) fn new(user_id: i64, live_id: &'a str) -> Self {
        Self {
            visitor_id: user_id,
            live_id,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartPlayForm<'a> {
    author_id: i64,
    pull_stream_type: &'a str,
}

impl<'a> StartPlayForm<'a> {
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
}
