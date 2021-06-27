pub(crate) const DANMAKU_SERVER: &str = "wss://link.xiatou.com/";

pub(crate) const RETRY_COUNT: u32 = 1;
pub(crate) const SUB_BIZ: &str = "mainApp";
pub(crate) const KPN: &str = "ACFUN_APP";
pub(crate) const KPF: &str = "PC_WEB";
pub(crate) const CLIENT_LIVE_SDK_VERSION: &str = "kwai-acfun-live-link";
pub(crate) const LINK_VERSION: &str = "2.13.8";
pub(crate) const DEVICE_MODEL: &str = "h5";

pub(crate) const REGISTER: &str = "Basic.Register";
pub(crate) const UNREGISTER: &str = "Basic.Unregister";
pub(crate) const KEEP_ALIVE: &str = "Basic.KeepAlive";
pub(crate) const PING: &str = "Basic.Ping";
pub(crate) const ENTER_ROOM: &str = "ZtLiveCsEnterRoom";
pub(crate) const ENTER_ROOM_ACK: &str = "ZtLiveCsEnterRoomAck";
pub(crate) const HEARTBEAT: &str = "ZtLiveCsHeartbeat";
pub(crate) const HEARTBEAT_ACK: &str = "ZtLiveCsHeartbeatAck";
pub(crate) const USER_EXIT: &str = "ZtLiveCsUserExit";
pub(crate) const USER_EXIT_ACK: &str = "ZtLiveCsUserExitAck";
pub(crate) const GLOBAL_CS_CMD: &str = "Global.ZtLiveInteractive.CsCmd";
pub(crate) const PUSH_MESSAGE: &str = "Push.ZtLiveInteractive.Message";
pub(crate) const ACTION_SIGNAL: &str = "ZtLiveScActionSignal";
pub(crate) const STATE_SIGNAL: &str = "ZtLiveScStateSignal";
pub(crate) const NOTIFY_SIGNAL: &str = "ZtLiveScNotifySignal";
pub(crate) const STATUS_CHANGED: &str = "ZtLiveScStatusChanged";
pub(crate) const TICKET_INVALID: &str = "ZtLiveScTicketInvalid";

pub(crate) const COMMENT: &str = "CommonActionSignalComment";
pub(crate) const LIKE: &str = "CommonActionSignalLike";
pub(crate) const USER_ENTER_ROOM: &str = "CommonActionSignalUserEnterRoom";
pub(crate) const FOLLOW_AUTHOR: &str = "CommonActionSignalUserFollowAuthor";
pub(crate) const THROW_BANANA: &str = "AcfunActionSignalThrowBanana";
pub(crate) const GIFT: &str = "CommonActionSignalGift";
pub(crate) const RICH_TEXT: &str = "CommonActionSignalRichText";
pub(crate) const JOIN_CLUB: &str = "AcfunActionSignalJoinClub";

pub(crate) const ACFUN_DISPLAY_INFO: &str = "AcfunStateSignalDisplayInfo";
pub(crate) const DISPLAY_INFO: &str = "CommonStateSignalDisplayInfo";
pub(crate) const TOP_USERS: &str = "CommonStateSignalTopUsers";
pub(crate) const RECENT_COMMENT: &str = "CommonStateSignalRecentComment";
pub(crate) const REDPACK_LIST: &str = "CommonStateSignalCurrentRedpackList";
pub(crate) const CHAT_CALL: &str = "CommonStateSignalChatCall";
pub(crate) const CHAT_ACCEPT: &str = "CommonStateSignalChatAccept";
pub(crate) const CHAT_READY: &str = "CommonStateSignalChatReady";
pub(crate) const CHAT_END: &str = "CommonStateSignalChatEnd";
pub(crate) const AUTHOR_CHAT_CALL: &str = "CommonStateSignalAuthorChatCall";
pub(crate) const AUTHOR_CHAT_ACCEPT: &str = "CommonStateSignalAuthorChatAccept";
pub(crate) const AUTHOR_CHAT_READY: &str = "CommonStateSignalAuthorChatReady";
pub(crate) const AUTHOR_CHAT_END: &str = "CommonStateSignalAuthorChatEnd";
pub(crate) const SOUND_CONFIG: &str = "CommonStateSignalAuthorChatChangeSoundConfig";
pub(crate) const LIVE_STATE: &str = "CommonStateSignalLiveState";

pub(crate) const KICKED_OUT: &str = "CommonNotifySignalKickedOut";
pub(crate) const VIOLATION_ALERT: &str = "CommonNotifySignalViolationAlert";
pub(crate) const MANAGER_STATE: &str = "CommonNotifySignalLiveManagerState";
