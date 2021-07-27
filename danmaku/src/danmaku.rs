use crate::{acproto, global::*, Result};
use derive_more::From;
use prost::Message;

#[cfg(feature = "_serde")]
use crate::Error;

#[cfg_attr(feature = "_serde", derive(::serde::Deserialize, ::serde::Serialize))]
#[cfg_attr(feature = "_serde", serde(rename_all = "camelCase"))]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct MedalInfo {
    pub uper_id: i64,
    pub user_id: i64,
    pub club_name: String,
    pub level: i32,
}

#[cfg(feature = "_serde")]
#[derive(Clone, Debug, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MedalInfo_ {
    medal_info: MedalInfo,
}

#[cfg(feature = "_serde")]
impl MedalInfo {
    #[inline]
    pub fn new<'a>(badge: impl Into<std::borrow::Cow<'a, str>>) -> Result<Self> {
        badge.into().parse()
    }
}

#[cfg(feature = "_serde")]
impl std::str::FromStr for MedalInfo {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str::<MedalInfo_>(s)?.medal_info)
    }
}

#[cfg_attr(feature = "_serde", derive(::serde::Deserialize, ::serde::Serialize))]
#[derive(Clone, Debug, From, PartialEq)]
pub enum Danmaku {
    ActionSignal(Vec<ActionSignal>),
    StateSignal(Vec<StateSignal>),
    NotifySignal(Vec<NotifySignal>),
}

#[cfg_attr(feature = "_serde", derive(::serde::Deserialize, ::serde::Serialize))]
#[derive(Clone, Debug, From, PartialEq)]
pub enum ActionSignal {
    Comment(acproto::CommonActionSignalComment),
    Like(acproto::CommonActionSignalLike),
    EnterRoom(acproto::CommonActionSignalUserEnterRoom),
    FollowAuthor(acproto::CommonActionSignalUserFollowAuthor),
    ThrowBanana(acproto::AcfunActionSignalThrowBanana),
    Gift(acproto::CommonActionSignalGift),
    RichText(acproto::CommonActionSignalRichText),
    JoinClub(acproto::AcfunActionSignalJoinClub),
    #[cfg_attr(feature = "_serde", serde(with = "serde_bytes"))]
    Unknown(Vec<u8>),
}

impl ActionSignal {
    #[inline]
    pub fn time(&self) -> i64 {
        match self {
            ActionSignal::Comment(s) => s.send_time_ms,
            ActionSignal::Like(s) => s.send_time_ms,
            ActionSignal::EnterRoom(s) => s.send_time_ms,
            ActionSignal::FollowAuthor(s) => s.send_time_ms,
            ActionSignal::ThrowBanana(s) => s.send_time_ms,
            ActionSignal::Gift(s) => s.send_time_ms,
            ActionSignal::RichText(s) => s.send_time_ms,
            ActionSignal::JoinClub(s) => s.join_time_ms,
            ActionSignal::Unknown(_) => 0,
        }
    }
}

#[cfg_attr(feature = "_serde", derive(::serde::Deserialize, ::serde::Serialize))]
#[derive(Clone, Debug, From, PartialEq)]
pub enum StateSignal {
    AcFunDisplayInfo(acproto::AcfunStateSignalDisplayInfo),
    DisplayInfo(acproto::CommonStateSignalDisplayInfo),
    TopUsers(acproto::CommonStateSignalTopUsers),
    RecentComment(acproto::CommonStateSignalRecentComment),
    RedpackList(acproto::CommonStateSignalCurrentRedpackList),
    ChatCall(acproto::CommonStateSignalChatCall),
    ChatAccept(acproto::CommonStateSignalChatAccept),
    ChatReady(acproto::CommonStateSignalChatReady),
    ChatEnd(acproto::CommonStateSignalChatEnd),
    AuthorChatCall(acproto::CommonStateSignalAuthorChatCall),
    AuthorChatAccept(acproto::CommonStateSignalAuthorChatAccept),
    AuthorChatReady(acproto::CommonStateSignalAuthorChatReady),
    AuthorChatEnd(acproto::CommonStateSignalAuthorChatEnd),
    AuthorChatChangeSoundConfig(acproto::CommonStateSignalAuthorChatChangeSoundConfig),
    #[cfg_attr(feature = "_serde", serde(with = "serde_bytes"))]
    Unknown(Vec<u8>),
}

#[cfg_attr(feature = "_serde", derive(::serde::Deserialize, ::serde::Serialize))]
#[derive(Clone, Debug, From, PartialEq)]
pub enum NotifySignal {
    KickedOut(acproto::CommonNotifySignalKickedOut),
    ViolationAlert(acproto::CommonNotifySignalViolationAlert),
    ManagerState(acproto::CommonNotifySignalLiveManagerState),
    #[cfg_attr(feature = "_serde", serde(with = "serde_bytes"))]
    Unknown(Vec<u8>),
}

pub(crate) fn action_signal(payload: &[u8]) -> Result<Vec<ActionSignal>> {
    let action = acproto::ZtLiveScActionSignal::decode(payload)?;
    let mut signals = Vec::with_capacity(action.item.iter().map(|i| i.payload.len()).sum());
    for item in action.item {
        for pl in item.payload {
            signals.push(match item.signal_type.as_str() {
                COMMENT => acproto::CommonActionSignalComment::decode(pl.as_slice())?.into(),
                LIKE => acproto::CommonActionSignalLike::decode(pl.as_slice())?.into(),
                USER_ENTER_ROOM => {
                    acproto::CommonActionSignalUserEnterRoom::decode(pl.as_slice())?.into()
                }
                FOLLOW_AUTHOR => {
                    acproto::CommonActionSignalUserFollowAuthor::decode(pl.as_slice())?.into()
                }
                THROW_BANANA => {
                    acproto::AcfunActionSignalThrowBanana::decode(pl.as_slice())?.into()
                }
                GIFT => acproto::CommonActionSignalGift::decode(pl.as_slice())?.into(),
                RICH_TEXT => acproto::CommonActionSignalRichText::decode(pl.as_slice())?.into(),
                JOIN_CLUB => acproto::AcfunActionSignalJoinClub::decode(pl.as_slice())?.into(),
                _ => {
                    log::trace!("unknown action signal type: {}", item.signal_type);
                    pl.into()
                }
            })
        }
    }
    signals.sort_unstable_by_key(ActionSignal::time);

    Ok(signals)
}

pub(crate) fn state_signal(payload: &[u8]) -> Result<Vec<StateSignal>> {
    let state = acproto::ZtLiveScStateSignal::decode(payload)?;
    let mut signals = Vec::with_capacity(state.item.len());
    for item in state.item {
        signals.push(match item.signal_type.as_str() {
            ACFUN_DISPLAY_INFO => {
                acproto::AcfunStateSignalDisplayInfo::decode(item.payload.as_slice())?.into()
            }
            DISPLAY_INFO => {
                acproto::CommonStateSignalDisplayInfo::decode(item.payload.as_slice())?.into()
            }
            TOP_USERS => {
                acproto::CommonStateSignalTopUsers::decode(item.payload.as_slice())?.into()
            }
            RECENT_COMMENT => {
                acproto::CommonStateSignalRecentComment::decode(item.payload.as_slice())?.into()
            }
            REDPACK_LIST => {
                acproto::CommonStateSignalCurrentRedpackList::decode(item.payload.as_slice())?
                    .into()
            }
            CHAT_CALL => {
                acproto::CommonStateSignalChatCall::decode(item.payload.as_slice())?.into()
            }
            CHAT_ACCEPT => {
                acproto::CommonStateSignalChatAccept::decode(item.payload.as_slice())?.into()
            }
            CHAT_READY => {
                acproto::CommonStateSignalChatReady::decode(item.payload.as_slice())?.into()
            }
            CHAT_END => acproto::CommonStateSignalChatEnd::decode(item.payload.as_slice())?.into(),
            AUTHOR_CHAT_CALL => {
                acproto::CommonStateSignalAuthorChatCall::decode(item.payload.as_slice())?.into()
            }
            AUTHOR_CHAT_ACCEPT => {
                acproto::CommonStateSignalAuthorChatAccept::decode(item.payload.as_slice())?.into()
            }
            AUTHOR_CHAT_READY => {
                acproto::CommonStateSignalAuthorChatReady::decode(item.payload.as_slice())?.into()
            }
            AUTHOR_CHAT_END => {
                acproto::CommonStateSignalAuthorChatEnd::decode(item.payload.as_slice())?.into()
            }
            SOUND_CONFIG => acproto::CommonStateSignalAuthorChatChangeSoundConfig::decode(
                item.payload.as_slice(),
            )?
            .into(),
            LIVE_STATE => {
                log::trace!("unknown state signal type: {}", item.signal_type);
                item.payload.into()
            }
            _ => {
                log::trace!("unknown state signal type: {}", item.signal_type);
                item.payload.into()
            }
        })
    }

    Ok(signals)
}

pub(crate) fn notify_signal(payload: &[u8]) -> Result<Vec<NotifySignal>> {
    let notify = acproto::ZtLiveScNotifySignal::decode(payload)?;
    let mut signals = Vec::with_capacity(notify.item.len());
    for item in notify.item {
        signals.push(match item.signal_type.as_str() {
            KICKED_OUT => {
                acproto::CommonNotifySignalKickedOut::decode(item.payload.as_slice())?.into()
            }
            VIOLATION_ALERT => {
                acproto::CommonNotifySignalViolationAlert::decode(item.payload.as_slice())?.into()
            }
            MANAGER_STATE => {
                acproto::CommonNotifySignalLiveManagerState::decode(item.payload.as_slice())?.into()
            }
            _ => {
                log::trace!("unknown notify signal type: {}", item.signal_type);
                item.payload.into()
            }
        })
    }

    Ok(signals)
}
