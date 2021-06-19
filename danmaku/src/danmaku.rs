use crate::{acproto, global::*, Error, Result};
use prost::Message;

#[cfg_attr(feature = "_serde", derive(serde::Deserialize, serde::Serialize))]
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

#[cfg_attr(feature = "_serde", derive(serde::Deserialize, serde::Serialize))]
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

#[cfg_attr(feature = "_serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum NotifySignal {
    KickedOut(acproto::CommonNotifySignalKickedOut),
    ViolationAlert(acproto::CommonNotifySignalViolationAlert),
    ManagerState(acproto::CommonNotifySignalLiveManagerState),
}

fn transfer<T>(err: async_channel::TrySendError<T>) -> Result<()> {
    match err {
        async_channel::TrySendError::Full(_) => Ok(()),
        async_channel::TrySendError::Closed(_) => Err(Error::SendDanmakuError),
    }
}

pub(crate) async fn action_signal(
    payload: &[u8],
    action_tx: &Option<async_channel::Sender<ActionSignal>>,
) -> Result<()> {
    let action = acproto::ZtLiveScActionSignal::decode(payload)?;

    for item in action.item {
        for pl in item.payload.iter().rev() {
            match item.signal_type.as_str() {
                COMMENT => {
                    if let Some(tx) = action_tx {
                        let comment = acproto::CommonActionSignalComment::decode(pl.as_slice())?;
                        tx.try_send(ActionSignal::Comment(comment))
                            .or_else(transfer)?;
                    }
                }
                LIKE => {
                    if let Some(tx) = action_tx {
                        let like = acproto::CommonActionSignalLike::decode(pl.as_slice())?;
                        tx.try_send(ActionSignal::Like(like)).or_else(transfer)?;
                    }
                }
                USER_ENTER_ROOM => {
                    if let Some(tx) = action_tx {
                        let enter =
                            acproto::CommonActionSignalUserEnterRoom::decode(pl.as_slice())?;
                        tx.try_send(ActionSignal::EnterRoom(enter))
                            .or_else(transfer)?;
                    }
                }
                FOLLOW_AUTHOR => {
                    if let Some(tx) = action_tx {
                        let follow =
                            acproto::CommonActionSignalUserFollowAuthor::decode(pl.as_slice())?;
                        tx.try_send(ActionSignal::FollowAuthor(follow))
                            .or_else(transfer)?;
                    }
                }
                THROW_BANANA => {
                    if let Some(tx) = action_tx {
                        let banana = acproto::AcfunActionSignalThrowBanana::decode(pl.as_slice())?;
                        tx.try_send(ActionSignal::ThrowBanana(banana))
                            .or_else(transfer)?;
                    }
                }
                GIFT => {
                    if let Some(tx) = action_tx {
                        let gift = acproto::CommonActionSignalGift::decode(pl.as_slice())?;
                        tx.try_send(ActionSignal::Gift(gift)).or_else(transfer)?;
                    }
                }
                RICH_TEXT => {
                    if let Some(tx) = action_tx {
                        let rich_text = acproto::CommonActionSignalRichText::decode(pl.as_slice())?;
                        tx.try_send(ActionSignal::RichText(rich_text))
                            .or_else(transfer)?;
                    }
                }
                JOIN_CLUB => {
                    if let Some(tx) = action_tx {
                        let join = acproto::AcfunActionSignalJoinClub::decode(pl.as_slice())?;
                        tx.try_send(ActionSignal::JoinClub(join))
                            .or_else(transfer)?;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

pub(crate) async fn state_signal(
    payload: &[u8],
    state_tx: &Option<async_channel::Sender<StateSignal>>,
) -> Result<()> {
    let state = acproto::ZtLiveScStateSignal::decode(payload)?;

    for item in state.item.iter().rev() {
        match item.signal_type.as_str() {
            ACFUN_DISPLAY_INFO => {
                if let Some(tx) = state_tx {
                    let banana =
                        acproto::AcfunStateSignalDisplayInfo::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::AcFunDisplayInfo(banana))
                        .or_else(transfer)?;
                }
            }
            DISPLAY_INFO => {
                if let Some(tx) = state_tx {
                    let display =
                        acproto::CommonStateSignalDisplayInfo::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::DisplayInfo(display))
                        .or_else(transfer)?;
                }
            }
            TOP_USERS => {
                if let Some(tx) = state_tx {
                    let top_user =
                        acproto::CommonStateSignalTopUsers::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::TopUsers(top_user))
                        .or_else(transfer)?;
                }
            }
            RECENT_COMMENT => {
                if let Some(tx) = state_tx {
                    let comment =
                        acproto::CommonStateSignalRecentComment::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::RecentComment(comment))
                        .or_else(transfer)?;
                }
            }
            REDPACK_LIST => {
                if let Some(tx) = state_tx {
                    let redpack = acproto::CommonStateSignalCurrentRedpackList::decode(
                        item.payload.as_slice(),
                    )?;
                    tx.try_send(StateSignal::RedpackList(redpack))
                        .or_else(transfer)?;
                }
            }
            CHAT_CALL => {
                if let Some(tx) = state_tx {
                    let chat_call =
                        acproto::CommonStateSignalChatCall::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::ChatCall(chat_call))
                        .or_else(transfer)?;
                }
            }
            CHAT_ACCEPT => {
                if let Some(tx) = state_tx {
                    let chat_accept =
                        acproto::CommonStateSignalChatAccept::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::ChatAccept(chat_accept))
                        .or_else(transfer)?;
                }
            }
            CHAT_READY => {
                if let Some(tx) = state_tx {
                    let chat_ready =
                        acproto::CommonStateSignalChatReady::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::ChatReady(chat_ready))
                        .or_else(transfer)?;
                }
            }
            CHAT_END => {
                if let Some(tx) = state_tx {
                    let chat_end =
                        acproto::CommonStateSignalChatEnd::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::ChatEnd(chat_end))
                        .or_else(transfer)?;
                }
            }
            AUTHOR_CHAT_CALL => {
                if let Some(tx) = state_tx {
                    let chat_call =
                        acproto::CommonStateSignalAuthorChatCall::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::AuthorChatCall(chat_call))
                        .or_else(transfer)?;
                }
            }
            AUTHOR_CHAT_ACCEPT => {
                if let Some(tx) = state_tx {
                    let chat_accept = acproto::CommonStateSignalAuthorChatAccept::decode(
                        item.payload.as_slice(),
                    )?;
                    tx.try_send(StateSignal::AuthorChatAccept(chat_accept))
                        .or_else(transfer)?;
                }
            }
            AUTHOR_CHAT_READY => {
                if let Some(tx) = state_tx {
                    let chat_ready =
                        acproto::CommonStateSignalAuthorChatReady::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::AuthorChatReady(chat_ready))
                        .or_else(transfer)?;
                }
            }
            AUTHOR_CHAT_END => {
                if let Some(tx) = state_tx {
                    let chat_end =
                        acproto::CommonStateSignalAuthorChatEnd::decode(item.payload.as_slice())?;
                    tx.try_send(StateSignal::AuthorChatEnd(chat_end))
                        .or_else(transfer)?;
                }
            }
            SOUND_CONFIG => {
                if let Some(tx) = state_tx {
                    let sound_config =
                        acproto::CommonStateSignalAuthorChatChangeSoundConfig::decode(
                            item.payload.as_slice(),
                        )?;
                    tx.try_send(StateSignal::AuthorChatChangeSoundConfig(sound_config))
                        .or_else(transfer)?;
                }
            }
            LIVE_STATE => {}
            _ => {}
        }
    }

    Ok(())
}

pub(crate) async fn notify_signal(
    payload: &[u8],
    notify_tx: &Option<async_channel::Sender<NotifySignal>>,
) -> Result<()> {
    let notify = acproto::ZtLiveScNotifySignal::decode(payload)?;

    for item in notify.item.iter().rev() {
        match item.signal_type.as_str() {
            KICKED_OUT => {
                if let Some(tx) = notify_tx {
                    let kicked_out =
                        acproto::CommonNotifySignalKickedOut::decode(item.payload.as_slice())?;
                    tx.try_send(NotifySignal::KickedOut(kicked_out))
                        .or_else(transfer)?;
                }
            }
            VIOLATION_ALERT => {
                if let Some(tx) = notify_tx {
                    let alert =
                        acproto::CommonNotifySignalViolationAlert::decode(item.payload.as_slice())?;
                    tx.try_send(NotifySignal::ViolationAlert(alert))
                        .or_else(transfer)?;
                }
            }
            MANAGER_STATE => {
                if let Some(tx) = notify_tx {
                    let state = acproto::CommonNotifySignalLiveManagerState::decode(
                        item.payload.as_slice(),
                    )?;
                    tx.try_send(NotifySignal::ManagerState(state))
                        .or_else(transfer)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
