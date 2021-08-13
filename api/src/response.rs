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
    pub result: i32,
    pub img: String,
    pub user_id: i64,
    pub username: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitorToken {
    pub result: i32,
    pub ac_security: String,
    pub user_id: i64,
    #[serde(rename = "acfun.api.visitor_st")]
    pub acfun_api_visitor_st: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserToken {
    pub result: i32,
    pub ssecurity: String,
    pub user_id: i64,
    #[serde(rename = "acfun.midground.api_st")]
    pub acfun_midground_api_st: String,
    #[serde(rename = "acfun.midground.api.at")]
    pub acfun_midground_api_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LiveInfo {
    pub result: i32,
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
    pub ticket_retry_count: i32,
    pub ticket_retry_interval_ms: i32,
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
    pub gift_slot_size: i32,
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
    pub gop_duration: i32,
    pub representation: Vec<Representation>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Representation {
    pub id: i32,
    pub url: String,
    pub bitrate: i32,
    pub quality_type: String,
    pub media_type: String,
    pub level: i32,
    pub name: String,
    pub hidden: bool,
    pub enable_adaptive: bool,
    pub default_select: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LiveAdaptiveConfig {
    pub buffer_init: i32,
    pub stable_buffer_switch_up_cnt: i32,
    pub stable_buffer_diff: i32,
    pub stable_buffer_cnt: i32,
    pub last_high_water_mark_in_ms: i32,
    pub speed_down_threshold: i32,
    pub min_state_cycle: i32,
    pub max_switching_time: i32,
    pub initiative_switching_time: i32,
    pub switch_pts_diff: i32,
    pub max_retry_cnt: i32,
    pub normal_config: NormalConfig,
    pub state_config: StateConfig,
    pub liveshow_config: LiveshowConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct NormalConfig {
    pub switch_down_q: i32,
    pub switch_down_bw_frag: i32,
    pub switch_up_bw_frag: i32,
    pub switch_up_q: i32,
    pub switch_time: i32,
    pub continuous_switch_time: i32,
    pub switch_up_bw_frag1: i32,
    pub switch_up_bw_frag2: i32,
    pub switch_up_bw_frag2_cnt: i32,
    pub speed_up_threshold: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StateConfig {
    pub frag_bw_window: i32,
    pub ls_sample_cnt: i64,
    pub ls_steps: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LiveshowConfig {
    pub buffer_init: i32,
    pub mobile_init_index: i32,
    pub speed_up_threshold: i32,
    pub switch_down_q: i32,
    pub switch_down_bw_frag: i32,
    pub switch_up_bw_frag: i32,
    pub switch_up_q: i32,
    pub switch_time: i32,
    pub continuous_switch_time: i32,
    pub switch_up_bw_frag1: i32,
    pub switch_up_bw_frag2: i32,
    pub switch_up_bw_frag2_cnt: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GiftList {
    pub result: i32,
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
    pub pay_wallet_type: i32,
    pub gift_price: i32,
    pub webp_pic_list: Vec<GiftPicture>,
    pub png_pic_list: Vec<GiftPicture>,
    pub small_png_pic_list: Vec<GiftPicture>,
    pub allow_batch_send_size_list: Vec<i32>,
    pub can_combo: bool,
    pub can_draw: bool,
    pub magic_face_id: i32,
    pub vup_ar_id: i32,
    pub description: String,
    pub redpack_price: i32,
    pub corner_marker_text: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftPicture {
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
    pub channel_list_data: ChannelListData,
    pub total_count: i32,
    //pub channel_data: ChannelData,
    pub live_list: Vec<UserLiveInfo>,
    //pub recommend_authors_data: Vec<::serde_json::Value>,
    pub channel_filters: ChannelFilters,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelListData {
    pub result: i32,
    pub request_id: String,
    pub live_list: Vec<UserLiveInfo>,
    pub count: i64,
    pub pcursor: String,
    #[serde(rename = "host-name")]
    pub host_name: String,
    pub total_count: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLiveInfo {
    pub result: Option<i32>,
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
    pub action: i32,
    pub href: String,
    pub live_id: String,
    pub stream_name: String,
    pub create_time: i64,
    pub title: Option<String>,
    pub cover_urls: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub live_type: Option<LiveType>,
    pub portrait: bool,
    pub panoramic: bool,
    pub online_count: i32,
    pub format_online_count: String,
    pub like_count: i32,
    pub format_like_count: String,
    pub has_fans_club: bool,
    pub biz_custom_data: String,
    pub cdn_auth_biz: i32,
    pub disable_danmaku_show: bool,
    pub paid_show_user_buy_status: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub action: i32,
    pub href: String,
    pub id: String,
    pub name: String,
    pub name_color: i32,
    pub head_url: String,
    pub user_head_img_info: UserAvatar,
    pub head_cdn_urls: Vec<CdnUrl>,
    pub avatar_image: String,
    pub avatar_frame: i32,
    pub avatar_frame_mobile_img: String,
    pub avatar_frame_pc_img: String,
    pub is_following: bool,
    pub is_followed: bool,
    pub following_status: i32,
    pub following_count: String,
    pub following_count_value: i32,
    pub contribute_count: String,
    pub contribute_count_value: i32,
    pub fan_count: String,
    pub fan_count_value: i32,
    pub gender: i32,
    pub sex_trend: i32,
    pub verified_type: Option<i32>,
    pub verified_types: Option<Vec<i32>>,
    pub verified_text: Option<String>,
    pub signature: Option<String>,
    pub is_join_up_college: Option<bool>,
    pub come_from: Option<String>,
    pub live_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAvatar {
    pub width: i32,
    pub height: i32,
    pub size: i32,
    pub r#type: i32,
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
pub struct LiveType {
    pub id: i32,
    pub name: String,
    pub category_id: i32,
    pub category_name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelFilters {
    pub live_channel_display_filters: Vec<LiveChannelDisplayFilter>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveChannelDisplayFilter {
    pub display_filters: Vec<DisplayFilter>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayFilter {
    pub filter_type: i32,
    pub filter_id: i32,
    pub name: String,
    pub cover: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalList {
    pub result: i32,
    pub medal_list: Vec<Medal>,
    #[serde(rename = "host-name")]
    pub host_name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Medal {
    pub uper_id: i64,
    pub uper_name: String,
    pub uper_head_url: String,
    pub club_name: String,
    pub level: i32,
    pub join_club_time: i64,
    pub wear_medal: bool,
    pub friendship_degree: i32,
    pub current_degree_limit: i32,
}

/*
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveGiftConfig {
    pub before_discount_gift_count: i32,
    pub live_gift_id: i64,
    pub after_discount_gift_count: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalDegreeLimit {
    pub uper_id: i64,
    pub gift_degree: i32,
    pub gift_degree_limit: i32,
    pub peach_degree: i32,
    pub peach_degree_limit: i32,
    pub live_watch_degree: i32,
    pub live_watch_degree_limit: i32,
    pub banana_degree: i32,
    pub banana_degree_limit: i32,
}
*/

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Summary {
    pub result: i32,
    pub data: SummaryData,
    pub host: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryData {
    pub live_duration_ms: i64,
    pub like_count: String,
    pub watch_count: String,
    pub pay_wallet_type_to_receive_currency: Option<GiftValue>,
    pub pay_wallet_type_to_receive_count: Option<GiftCount>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GiftValue {
    #[serde(rename = "1")]
    pub paid_gift_value: i32,
    #[serde(rename = "2")]
    pub banana_value: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GiftCount {
    #[serde(rename = "1")]
    pub paid_gift_count: i32,
    #[serde(rename = "2")]
    pub banana_count: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalRankList {
    pub result: i32,
    pub cur_user_rank_index: Option<String>,
    pub friendship_degree_rank: Vec<MedalRank>,
    pub has_fans_club: bool,
    pub fans_total_count: i32,
    pub club_name: String,
    pub cur_user_friendship_degree: Option<i32>,
    #[serde(rename = "host-name")]
    pub host_name: String,
    pub is_in_fans_club: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalRank {
    pub friendship_degree: i32,
    pub user_id: i64,
    pub medal_level: i32,
    pub user_info: UserInfo,
}
