use crate::{acproto, global::*, Error, Result};
use futures::channel::mpsc;
use prost::Message;

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
#[derive(Clone, Debug, Default, ::serde::Deserialize, Eq, Hash, PartialEq, ::serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MedalInfo_ {
    medal_info: MedalInfo,
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
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub enum NotifySignal {
    KickedOut(acproto::CommonNotifySignalKickedOut),
    ViolationAlert(acproto::CommonNotifySignalViolationAlert),
    ManagerState(acproto::CommonNotifySignalLiveManagerState),
    #[cfg_attr(feature = "_serde", serde(with = "serde_bytes"))]
    Unknown(Vec<u8>),
}

#[inline]
fn transfer<T>(err: mpsc::TrySendError<T>) -> Result<()> {
    if err.is_full() {
        Ok(())
    } else {
        Err(Error::SendDanmakuError)
    }
}

pub(crate) async fn action_signal(
    payload: &[u8],
    action_tx: &mut mpsc::Sender<Vec<ActionSignal>>,
) -> Result<()> {
    let action = acproto::ZtLiveScActionSignal::decode(payload)?;
    let mut v = Vec::with_capacity(action.item.iter().map(|i| i.payload.len()).sum());
    for item in action.item {
        for pl in item.payload {
            v.push(match item.signal_type.as_str() {
                COMMENT => ActionSignal::Comment(acproto::CommonActionSignalComment::decode(
                    pl.as_slice(),
                )?),
                LIKE => ActionSignal::Like(acproto::CommonActionSignalLike::decode(pl.as_slice())?),
                USER_ENTER_ROOM => ActionSignal::EnterRoom(
                    acproto::CommonActionSignalUserEnterRoom::decode(pl.as_slice())?,
                ),
                FOLLOW_AUTHOR => ActionSignal::FollowAuthor(
                    acproto::CommonActionSignalUserFollowAuthor::decode(pl.as_slice())?,
                ),
                THROW_BANANA => ActionSignal::ThrowBanana(
                    acproto::AcfunActionSignalThrowBanana::decode(pl.as_slice())?,
                ),
                GIFT => ActionSignal::Gift(acproto::CommonActionSignalGift::decode(pl.as_slice())?),
                RICH_TEXT => ActionSignal::RichText(acproto::CommonActionSignalRichText::decode(
                    pl.as_slice(),
                )?),
                JOIN_CLUB => ActionSignal::JoinClub(acproto::AcfunActionSignalJoinClub::decode(
                    pl.as_slice(),
                )?),
                _ => ActionSignal::Unknown(pl),
            })
        }
    }
    v.sort_unstable_by_key(ActionSignal::time);
    action_tx.try_send(v).or_else(transfer)?;

    Ok(())
}

pub(crate) async fn state_signal(
    payload: &[u8],
    state_tx: &mut mpsc::Sender<Vec<StateSignal>>,
) -> Result<()> {
    let state = acproto::ZtLiveScStateSignal::decode(payload)?;
    let mut v = Vec::with_capacity(state.item.len());
    for item in state.item {
        v.push(match item.signal_type.as_str() {
            ACFUN_DISPLAY_INFO => StateSignal::AcFunDisplayInfo(
                acproto::AcfunStateSignalDisplayInfo::decode(item.payload.as_slice())?,
            ),
            DISPLAY_INFO => StateSignal::DisplayInfo(
                acproto::CommonStateSignalDisplayInfo::decode(item.payload.as_slice())?,
            ),
            TOP_USERS => StateSignal::TopUsers(acproto::CommonStateSignalTopUsers::decode(
                item.payload.as_slice(),
            )?),
            RECENT_COMMENT => StateSignal::RecentComment(
                acproto::CommonStateSignalRecentComment::decode(item.payload.as_slice())?,
            ),
            REDPACK_LIST => StateSignal::RedpackList(
                acproto::CommonStateSignalCurrentRedpackList::decode(item.payload.as_slice())?,
            ),
            CHAT_CALL => StateSignal::ChatCall(acproto::CommonStateSignalChatCall::decode(
                item.payload.as_slice(),
            )?),
            CHAT_ACCEPT => StateSignal::ChatAccept(acproto::CommonStateSignalChatAccept::decode(
                item.payload.as_slice(),
            )?),
            CHAT_READY => StateSignal::ChatReady(acproto::CommonStateSignalChatReady::decode(
                item.payload.as_slice(),
            )?),
            CHAT_END => StateSignal::ChatEnd(acproto::CommonStateSignalChatEnd::decode(
                item.payload.as_slice(),
            )?),
            AUTHOR_CHAT_CALL => StateSignal::AuthorChatCall(
                acproto::CommonStateSignalAuthorChatCall::decode(item.payload.as_slice())?,
            ),
            AUTHOR_CHAT_ACCEPT => StateSignal::AuthorChatAccept(
                acproto::CommonStateSignalAuthorChatAccept::decode(item.payload.as_slice())?,
            ),
            AUTHOR_CHAT_READY => StateSignal::AuthorChatReady(
                acproto::CommonStateSignalAuthorChatReady::decode(item.payload.as_slice())?,
            ),
            AUTHOR_CHAT_END => StateSignal::AuthorChatEnd(
                acproto::CommonStateSignalAuthorChatEnd::decode(item.payload.as_slice())?,
            ),
            SOUND_CONFIG => StateSignal::AuthorChatChangeSoundConfig(
                acproto::CommonStateSignalAuthorChatChangeSoundConfig::decode(
                    item.payload.as_slice(),
                )?,
            ),
            LIVE_STATE => StateSignal::Unknown(item.payload),
            _ => StateSignal::Unknown(item.payload),
        })
    }
    state_tx.try_send(v).or_else(transfer)?;

    Ok(())
}

pub(crate) async fn notify_signal(
    payload: &[u8],
    notify_tx: &mut mpsc::Sender<Vec<NotifySignal>>,
) -> Result<()> {
    let notify = acproto::ZtLiveScNotifySignal::decode(payload)?;
    let mut v = Vec::with_capacity(notify.item.len());
    for item in notify.item {
        v.push(match item.signal_type.as_str() {
            KICKED_OUT => NotifySignal::KickedOut(acproto::CommonNotifySignalKickedOut::decode(
                item.payload.as_slice(),
            )?),
            VIOLATION_ALERT => NotifySignal::ViolationAlert(
                acproto::CommonNotifySignalViolationAlert::decode(item.payload.as_slice())?,
            ),
            MANAGER_STATE => NotifySignal::ManagerState(
                acproto::CommonNotifySignalLiveManagerState::decode(item.payload.as_slice())?,
            ),
            _ => NotifySignal::Unknown(item.payload),
        })
    }
    notify_tx.try_send(v).or_else(transfer)?;

    Ok(())
}
