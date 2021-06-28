use serde::{Deserialize, Serialize};

fn deserialize_stream_info<'de, D>(deserializer: D) -> Result<StreamInfo, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrStruct;

    impl<'de> serde::de::Visitor<'de> for StringOrStruct {
        type Value = StreamInfo;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            serde_json::from_str(s).map_err(E::custom)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct)
}

fn deserialize_live_adaptive_config<'de, D>(deserializer: D) -> Result<LiveAdaptiveConfig, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrStruct;

    impl<'de> serde::de::Visitor<'de> for StringOrStruct {
        type Value = LiveAdaptiveConfig;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            serde_json::from_str(s).map_err(E::custom)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct)
}

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
    #[serde(deserialize_with = "deserialize_stream_info")]
    pub video_play_res: StreamInfo,
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
    #[serde(deserialize_with = "deserialize_live_adaptive_config")]
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
pub struct GiftList {
    pub result: i64,
    pub data: GiftData,
    pub host: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftData {
    pub gift_list: Vec<Gift>,
    pub external_display_gift_id: i64,
    pub external_display_gift_tips_delay_time: i64,
    pub external_display_gift: ExternalDisplayGift,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gift {
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
    pub gift_list: Vec<Gift>,
    pub tips_delay_time: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveList {
    pub result: i64,
    pub request_id: String,
    pub live_list: Vec<UserLiveInfo>,
    pub count: i64,
    pub pcursor: String,
    #[serde(rename = "host-name")]
    pub host_name: String,
    pub total_count: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLiveInfo {
    pub result: Option<i64>,
    #[serde(rename = "host-name")]
    pub host_name: Option<String>,
    pub visitor: Option<i64>,
    pub author_id: i64,
    pub user: UserInfo,
    pub request_id: String,
    pub group_id: Option<String>,
    #[serde(flatten)]
    pub live_data: Option<LiveData>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveData {
    pub action: i64,
    pub href: String,
    pub live_id: String,
    pub stream_name: String,
    pub create_time: i64,
    pub title: String,
    pub cover_urls: Vec<String>,
    #[serde(rename = "type")]
    pub live_type: LiveType,
    pub portrait: bool,
    pub panoramic: bool,
    pub online_count: i64,
    pub format_online_count: String,
    pub like_count: i64,
    pub format_like_count: String,
    pub has_fans_club: bool,
    pub biz_custom_data: String,
    pub cdn_auth_biz: i64,
    pub disable_danmaku_show: bool,
    pub paid_show_user_buy_status: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub action: i64,
    pub href: String,
    pub id: String,
    pub name: String,
    pub name_color: i64,
    pub head_url: String,
    pub user_head_img_info: UserAvatar,
    pub head_cdn_urls: Vec<AvatarCdnUrl>,
    pub avatar_image: String,
    pub avatar_frame: i64,
    pub avatar_frame_mobile_img: String,
    pub avatar_frame_pc_img: String,
    pub is_followed: bool,
    pub is_following: bool,
    pub following_status: i64,
    pub following_count: String,
    pub following_count_value: i64,
    pub contribute_count: String,
    pub contribute_count_value: i64,
    pub fan_count: String,
    pub fan_count_value: i64,
    pub gender: i64,
    pub sex_trend: i64,
    pub verified_type: Option<i64>,
    pub verified_types: Option<Vec<i64>>,
    pub verified_text: Option<String>,
    pub signature: Option<String>,
    pub is_join_up_college: Option<bool>,
    pub come_from: Option<String>,
    pub live_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAvatar {
    pub width: i64,
    pub height: i64,
    pub size: i64,
    pub r#type: i64,
    pub thumbnail_image: ThumbnailImage,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailImage {
    pub cdn_urls: Vec<CdnUrl>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnUrl {
    pub url: String,
    pub free_traffic_cdn: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvatarCdnUrl {
    pub url: String,
    pub free_traffic_cdn: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveType {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub category_name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalList {
    pub result: i64,
    pub medal_list: Vec<Medal>,
    pub rank_index: Option<String>,
    pub live_gift_config: LiveGiftConfig,
    pub medal_degree_limit: MedalDegreeLimit,
    pub club_name: Option<String>,
    pub medal: Option<Medal>,
    #[serde(rename = "host-name")]
    pub host_name: String,
    pub status: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Medal {
    pub uper_id: i64,
    pub friendship_degree: i64,
    pub join_club_time: i64,
    pub club_name: String,
    pub wear_medal: bool,
    pub uper_name: String,
    pub uper_head_url: String,
    pub current_degree_limit: i64,
    pub level: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveGiftConfig {
    pub before_discount_gift_count: i64,
    pub live_gift_id: i64,
    pub after_discount_gift_count: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalDegreeLimit {
    pub uper_id: i64,
    pub gift_degree: i64,
    pub gift_degree_limit: i64,
    pub peach_degree: i64,
    pub peach_degree_limit: i64,
    pub live_watch_degree: i64,
    pub live_watch_degree_limit: i64,
    pub banana_degree: i64,
    pub banana_degree_limit: i64,
}
