use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    pub result: i64,
    pub img: String,
    pub user_id: i64,
    pub username: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitorToken {
    pub result: i64,
    pub ac_security: String,
    pub user_id: i64,
    #[serde(rename = "acfun.api.visitor_st")]
    pub acfun_api_visitor_st: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserToken {
    pub result: i64,
    pub ssecurity: String,
    pub user_id: i64,
    #[serde(rename = "acfun.midground.api_st")]
    pub acfun_midground_api_st: String,
    #[serde(rename = "acfun.midground.api.at")]
    pub acfun_midground_api_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveInfo {
    pub result: i64,
    pub data: LiveInfoData,
    pub host: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveInfoData {
    pub live_id: String,
    pub available_tickets: Vec<String>,
    pub enter_room_attach: String,
    #[serde(skip_serializing)]
    pub(crate) video_play_res: String,
    #[serde(skip_deserializing)]
    pub stream_info: StreamInfo,
    pub caption: String,
    pub ticket_retry_count: i64,
    pub ticket_retry_interval_ms: i64,
    pub notices: Vec<Notice>,
    pub config: LiveInfoConfig,
    pub live_start_time: i64,
    pub panoramic: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub user_id: i64,
    pub user_name: String,
    pub user_gender: String,
    pub notice: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveInfoConfig {
    pub gift_slot_size: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfo {
    pub live_adaptive_manifest: Vec<LiveAdaptiveManifest>,
    #[serde(rename = "liveAdaptiveConfig", skip_serializing)]
    pub(crate) live_adaptive_config_string: String,
    #[serde(skip_deserializing)]
    pub live_adaptive_config: LiveAdaptiveConfig,
    pub stream_name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveAdaptiveManifest {
    pub free_traffic_cdn: bool,
    pub version: String,
    pub r#type: String,
    pub hide_auto: bool,
    pub adaptation_set: AdaptationSet,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptationSet {
    pub gop_duration: i64,
    pub representation: Vec<Representation>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Representation {
    pub id: i64,
    pub url: String,
    pub bitrate: i64,
    pub quality_type: String,
    pub media_type: String,
    pub level: i64,
    pub name: String,
    pub hidden: bool,
    pub enable_adaptive: bool,
    pub default_select: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LiveAdaptiveConfig {
    pub buffer_init: i64,
    pub stable_buffer_switch_up_cnt: i64,
    pub stable_buffer_diff: i64,
    pub stable_buffer_cnt: i64,
    pub last_high_water_mark_in_ms: i64,
    pub speed_down_threshold: i64,
    pub min_state_cycle: i64,
    pub max_switching_time: i64,
    pub initiative_switching_time: i64,
    pub switch_pts_diff: i64,
    pub max_retry_cnt: i64,
    pub normal_config: NormalConfig,
    pub state_config: StateConfig,
    pub liveshow_config: LiveshowConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct NormalConfig {
    pub switch_down_q: i64,
    pub switch_down_bw_frag: i64,
    pub switch_up_bw_frag: i64,
    pub switch_up_q: i64,
    pub switch_time: i64,
    pub continuous_switch_time: i64,
    pub switch_up_bw_frag1: i64,
    pub switch_up_bw_frag2: i64,
    pub switch_up_bw_frag2_cnt: i64,
    pub speed_up_threshold: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StateConfig {
    pub frag_bw_window: i64,
    pub ls_sample_cnt: i64,
    pub ls_steps: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LiveshowConfig {
    pub buffer_init: i64,
    pub mobile_init_index: i64,
    pub speed_up_threshold: i64,
    pub switch_down_q: i64,
    pub switch_down_bw_frag: i64,
    pub switch_up_bw_frag: i64,
    pub switch_up_q: i64,
    pub switch_time: i64,
    pub continuous_switch_time: i64,
    pub switch_up_bw_frag1: i64,
    pub switch_up_bw_frag2: i64,
    pub switch_up_bw_frag2_cnt: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gift {
    pub result: i64,
    pub data: GiftData,
    pub host: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftData {
    pub gift_list: Vec<GiftList>,
    pub external_display_gift_id: i64,
    pub external_display_gift_tips_delay_time: i64,
    pub external_display_gift: ExternalDisplayGift,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftList {
    pub gift_id: i64,
    pub gift_name: String,
    pub ar_live_name: String,
    pub pay_wallet_type: i64,
    pub gift_price: i64,
    pub webp_pic_list: Vec<GiftPicList>,
    pub png_pic_list: Vec<GiftPicList>,
    pub small_png_pic_list: Vec<GiftPicList>,
    pub allow_batch_send_size_list: Vec<i64>,
    pub can_combo: bool,
    pub can_draw: bool,
    pub magic_face_id: i64,
    pub vup_ar_id: i64,
    pub description: String,
    pub redpack_price: i64,
    pub corner_marker_text: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftPicList {
    pub cdn: String,
    pub url: String,
    pub url_pattern: String,
    pub free_traffic: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDisplayGift {
    pub gift_list: Vec<GiftList>,
    pub tips_delay_time: i64,
}
