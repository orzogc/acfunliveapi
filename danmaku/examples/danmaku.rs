use acfunliveapi::{
    client::ApiClientBuilder,
    response::{Gift, GiftList},
};
use acfunlivedanmaku::{client::*, danmaku::*, Result};
use std::{collections::HashMap, env};
use tokio::select;

#[tokio::main]
async fn main() -> Result<()> {
    let liver_uid: i64 = env::var("LIVER_UID")
        .expect("need to set the LIVER_UID environment variable to the liver's uid")
        .parse()
        .expect("LIVER_UID should be an integer");
    let api_client = ApiClientBuilder::default_client()?
        .liver_uid(liver_uid)
        .build()
        .await?;
    let gifts: GiftList = api_client.get().await?;
    let gifts: HashMap<_, _> = gifts
        .data
        .gift_list
        .into_iter()
        .map(|g| (g.gift_id, g))
        .collect();
    let mut client: DanmakuClient<_> = api_client.into();
    let action_rx = client.action_signal();
    let action = async {
        while let Ok(action) = action_rx.recv().await {
            handle_action(action, &gifts);
        }
    };
    let state_rx = client.state_signal();
    let state = async {
        while let Ok(state) = state_rx.recv().await {
            handle_state(state);
        }
    };
    let notify_rx = client.notify_signal();
    let notify = async {
        while let Ok(notify) = notify_rx.recv().await {
            handle_notify(notify);
        }
    };
    select! {
        result = client.danmaku() => {
            result?;
        }
        _ = action => {}
        _ = state => {}
        _ = notify => {}
    }

    Ok(())
}

fn handle_action(action: ActionSignal, gifts: &HashMap<i64, Gift>) {
    match action {
        ActionSignal::Comment(d) => {
            let user_info = d.user_info.unwrap_or_default();
            println!(
                "{} {}({}): {}",
                d.send_time_ms, user_info.nickname, user_info.user_id, d.content
            );
        }
        ActionSignal::Like(d) => {
            let user_info = d.user_info.unwrap_or_default();
            println!(
                "{} {}({}) liked",
                d.send_time_ms, user_info.nickname, user_info.user_id
            );
        }
        ActionSignal::EnterRoom(d) => {
            let user_info = d.user_info.unwrap_or_default();
            println!(
                "{} {}({}) entered the live room",
                d.send_time_ms, user_info.nickname, user_info.user_id
            );
        }
        ActionSignal::FollowAuthor(d) => {
            let user_info = d.user_info.unwrap_or_default();
            println!(
                "{} {}({}) followed the liver",
                d.send_time_ms, user_info.nickname, user_info.user_id
            );
        }
        ActionSignal::ThrowBanana(d) => {
            let user_info = d.visitor.unwrap_or_default();
            println!(
                "{} {}({}) threw {} bananas",
                d.send_time_ms, user_info.name, user_info.user_id, d.count
            );
        }
        ActionSignal::Gift(d) => {
            let user_info = d.user.unwrap_or_default();
            println!(
                "{} {}({}) gave {} * {} (all: {})",
                d.send_time_ms,
                user_info.nickname,
                user_info.user_id,
                d.count,
                gifts.get(&d.gift_id).cloned().unwrap_or_default().gift_name,
                d.count * d.combo
            );
        }
        ActionSignal::RichText(d) => println!("{:?}", d),
        ActionSignal::JoinClub(d) => {
            let fans_info = d.fans_info.unwrap_or_default();
            let uper_info = d.uper_info.unwrap_or_default();
            println!(
                "{} {}({}) joined {}({})'s club",
                d.join_time_ms,
                fans_info.name,
                fans_info.user_id,
                uper_info.name,
                uper_info.user_id
            );
        }
    }
}

fn handle_state(state: StateSignal) {
    match state {
        StateSignal::AcFunDisplayInfo(d) => println!("bananas count: {}", d.banana_count),
        StateSignal::DisplayInfo(d) => println!(
            "viewers count: {}, likes count: {}, likes delta: {}",
            d.watching_count, d.like_count, d.like_delta
        ),
        StateSignal::TopUsers(d) => d.user.into_iter().for_each(|u| {
            let user_info = u.user_info.unwrap_or_default();
            println!(
                "top user: {}({}) (coins: {})",
                user_info.nickname, user_info.user_id, u.display_send_amount
            );
        }),
        StateSignal::RecentComment(d) => d.comment.into_iter().for_each(|c| {
            let user_info = c.user_info.unwrap_or_default();
            println!(
                "recent comment: {} {}({}): {}",
                c.send_time_ms, user_info.nickname, user_info.user_id, c.content
            );
        }),
        StateSignal::RedpackList(d) => println!("{:?}", d),
        StateSignal::ChatCall(d) => println!("{:?}", d),
        StateSignal::ChatAccept(d) => println!("{:?}", d),
        StateSignal::ChatReady(d) => println!("{:?}", d),
        StateSignal::ChatEnd(d) => println!("{:?}", d),
        StateSignal::AuthorChatCall(d) => println!("{:?}", d),
        StateSignal::AuthorChatAccept(d) => println!("{:?}", d),
        StateSignal::AuthorChatReady(d) => println!("{:?}", d),
        StateSignal::AuthorChatEnd(d) => println!("{:?}", d),
        StateSignal::AuthorChatChangeSoundConfig(d) => println!("{:?}", d),
    }
}

fn handle_notify(notify: NotifySignal) {
    match notify {
        NotifySignal::KickedOut(d) => println!("kicked out: {}", d.reason),
        NotifySignal::ViolationAlert(d) => println!("violation alert: {}", d.violation_content),
        NotifySignal::ManagerState(d) => println!("manager state: {:?}", d.state()),
    }
}
