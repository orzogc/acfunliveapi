use acfunliveapi::{
    client::ApiClientBuilder,
    response::{Gift, GiftList},
};
use acfunlivedanmaku::{client::*, danmaku::*, Result};
use futures::StreamExt;
use std::{collections::HashMap, env};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter(Some("acfunliveapi"), log::LevelFilter::Trace)
        .filter(Some("acfunlivedanmaku"), log::LevelFilter::Trace)
        .init();

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

    let mut client = DanmakuClient::default_client(api_client.into()).await?;
    while let Some(result) = client.next().await {
        match result {
            Ok(damaku) => match damaku {
                Danmaku::ActionSignal(action) => handle_action(action, &gifts),
                Danmaku::StateSignal(state) => handle_state(state),
                Danmaku::NotifySignal(notify) => handle_notify(notify),
            },
            Err(e) => println!("error: {}", e),
        }
    }

    Ok(())
}

fn handle_action(action: Vec<ActionSignal>, gifts: &HashMap<i64, Gift>) {
    for action in action {
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
            ActionSignal::Unknown(s) => {
                println!("unknown action signal: {}", String::from_utf8_lossy(&s))
            }
        }
    }
}

fn handle_state(state: Vec<StateSignal>) {
    for state in state {
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
            StateSignal::Unknown(s) => {
                println!("unknown state signal: {}", String::from_utf8_lossy(&s))
            }
        }
    }
}

fn handle_notify(notify: Vec<NotifySignal>) {
    for notify in notify {
        match notify {
            NotifySignal::KickedOut(d) => println!("kicked out: {}", d.reason),
            NotifySignal::ViolationAlert(d) => println!("violation alert: {}", d.violation_content),
            NotifySignal::ManagerState(d) => println!("manager state: {:?}", d.state()),
            NotifySignal::Unknown(s) => {
                println!("unknown notify signal: {}", String::from_utf8_lossy(&s))
            }
        }
    }
}
