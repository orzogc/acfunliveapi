use crate::{acproto, global::*, Error, Result};
use futures::channel::mpsc;
use prost::Message;

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
}

#[cfg_attr(feature = "_serde", derive(::serde::Deserialize, ::serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum NotifySignal {
    KickedOut(acproto::CommonNotifySignalKickedOut),
    ViolationAlert(acproto::CommonNotifySignalViolationAlert),
    ManagerState(acproto::CommonNotifySignalLiveManagerState),
}

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
    for item in action.item.iter().rev() {
        for pl in item.payload.iter().rev() {
            match item.signal_type.as_str() {
                COMMENT => v.push(ActionSignal::Comment(
                    acproto::CommonActionSignalComment::decode(pl.as_slice())?,
                )),
                LIKE => v.push(ActionSignal::Like(acproto::CommonActionSignalLike::decode(
                    pl.as_slice(),
                )?)),
                USER_ENTER_ROOM => v.push(ActionSignal::EnterRoom(
                    acproto::CommonActionSignalUserEnterRoom::decode(pl.as_slice())?,
                )),
                FOLLOW_AUTHOR => v.push(ActionSignal::FollowAuthor(
                    acproto::CommonActionSignalUserFollowAuthor::decode(pl.as_slice())?,
                )),
                THROW_BANANA => v.push(ActionSignal::ThrowBanana(
                    acproto::AcfunActionSignalThrowBanana::decode(pl.as_slice())?,
                )),
                GIFT => v.push(ActionSignal::Gift(acproto::CommonActionSignalGift::decode(
                    pl.as_slice(),
                )?)),
                RICH_TEXT => v.push(ActionSignal::RichText(
                    acproto::CommonActionSignalRichText::decode(pl.as_slice())?,
                )),
                JOIN_CLUB => v.push(ActionSignal::JoinClub(
                    acproto::AcfunActionSignalJoinClub::decode(pl.as_slice())?,
                )),
                _ => {}
            }
        }
    }
    action_tx.try_send(v).or_else(transfer)?;

    Ok(())
}

pub(crate) async fn state_signal(
    payload: &[u8],
    state_tx: &mut mpsc::Sender<Vec<StateSignal>>,
) -> Result<()> {
    let state = acproto::ZtLiveScStateSignal::decode(payload)?;
    let mut v = Vec::with_capacity(state.item.len());
    for item in state.item.iter().rev() {
        match item.signal_type.as_str() {
            ACFUN_DISPLAY_INFO => v.push(StateSignal::AcFunDisplayInfo(
                acproto::AcfunStateSignalDisplayInfo::decode(item.payload.as_slice())?,
            )),
            DISPLAY_INFO => v.push(StateSignal::DisplayInfo(
                acproto::CommonStateSignalDisplayInfo::decode(item.payload.as_slice())?,
            )),
            TOP_USERS => v.push(StateSignal::TopUsers(
                acproto::CommonStateSignalTopUsers::decode(item.payload.as_slice())?,
            )),
            RECENT_COMMENT => v.push(StateSignal::RecentComment(
                acproto::CommonStateSignalRecentComment::decode(item.payload.as_slice())?,
            )),
            REDPACK_LIST => v.push(StateSignal::RedpackList(
                acproto::CommonStateSignalCurrentRedpackList::decode(item.payload.as_slice())?,
            )),
            CHAT_CALL => v.push(StateSignal::ChatCall(
                acproto::CommonStateSignalChatCall::decode(item.payload.as_slice())?,
            )),
            CHAT_ACCEPT => v.push(StateSignal::ChatAccept(
                acproto::CommonStateSignalChatAccept::decode(item.payload.as_slice())?,
            )),
            CHAT_READY => v.push(StateSignal::ChatReady(
                acproto::CommonStateSignalChatReady::decode(item.payload.as_slice())?,
            )),
            CHAT_END => v.push(StateSignal::ChatEnd(
                acproto::CommonStateSignalChatEnd::decode(item.payload.as_slice())?,
            )),
            AUTHOR_CHAT_CALL => v.push(StateSignal::AuthorChatCall(
                acproto::CommonStateSignalAuthorChatCall::decode(item.payload.as_slice())?,
            )),
            AUTHOR_CHAT_ACCEPT => v.push(StateSignal::AuthorChatAccept(
                acproto::CommonStateSignalAuthorChatAccept::decode(item.payload.as_slice())?,
            )),
            AUTHOR_CHAT_READY => v.push(StateSignal::AuthorChatReady(
                acproto::CommonStateSignalAuthorChatReady::decode(item.payload.as_slice())?,
            )),
            AUTHOR_CHAT_END => v.push(StateSignal::AuthorChatEnd(
                acproto::CommonStateSignalAuthorChatEnd::decode(item.payload.as_slice())?,
            )),
            SOUND_CONFIG => v.push(StateSignal::AuthorChatChangeSoundConfig(
                acproto::CommonStateSignalAuthorChatChangeSoundConfig::decode(
                    item.payload.as_slice(),
                )?,
            )),
            LIVE_STATE => {}
            _ => {}
        }
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
    for item in notify.item.iter().rev() {
        match item.signal_type.as_str() {
            KICKED_OUT => v.push(NotifySignal::KickedOut(
                acproto::CommonNotifySignalKickedOut::decode(item.payload.as_slice())?,
            )),
            VIOLATION_ALERT => v.push(NotifySignal::ViolationAlert(
                acproto::CommonNotifySignalViolationAlert::decode(item.payload.as_slice())?,
            )),
            MANAGER_STATE => v.push(NotifySignal::ManagerState(
                acproto::CommonNotifySignalLiveManagerState::decode(item.payload.as_slice())?,
            )),
            _ => {}
        }
    }
    notify_tx.try_send(v).or_else(transfer)?;

    Ok(())
}
