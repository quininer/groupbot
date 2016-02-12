extern crate tox;
extern crate toml;
extern crate chrono;

#[macro_use] mod utils;

use std::env::args;
use std::path::Path;
use std::thread::sleep;
use std::fs::OpenOptions;
use std::io::{ Write, Read };
use std::collections::HashMap;
use chrono::UTC;
use tox::core::{
    Event,
    Status, Chat, Listen, FriendManage
};
use tox::core::chat::MessageType;
use tox::core::file::{ hash, FileKind, FileOperate, FileManage };
use tox::core::group::{ GroupCreate, GroupType, PeerChange };
use tox::av::AvGroupCreate;

use utils::{ parse_config, init, save };


fn main() {
    let config = parse_config(args().skip(1).next().unwrap_or("./config.toml".into()));
    let (mut bot, avatar, path) = init(config.get("bot").and_then(|r| r.as_table()).unwrap());
    println!("Tox ID: {}", bot.address());

    let botiter = bot.iterate();
    let mut group = bot.create_group().unwrap();
    let mut leave_time = HashMap::new();
    let mut logfd = log!(open config, UTC::today());
    let mut today = UTC::today();

    'main: loop {
        sleep(bot.interval());
        match botiter.try_recv() {
            Ok(Event::FriendStatusMessage(friend, _)) => {

                if avatar.len() != 0
                    && !check!(keyword config, "off_avatar", friend)
                {
                    friend.transmission(
                        FileKind::AVATAR,
                        "avatar.png",
                        avatar.len() as u64,
                        Some(&hash(&avatar))
                    ).ok();
                }

                if !check!(keyword config, "off_invite", friend) {
                    group.invite(&friend);
                }
            },
            Ok(Event::RequestFriend(pk, msg)) => {
                if check!(master config, "passphrase", k, {
                    msg != try_unwrap!(k.as_str()).as_bytes()
                }) {
                    continue
                };

                if bot.add_friend(pk).is_ok() {
                    save(&path, &bot);
                }
            },
            Ok(Event::FriendFileChunkRequest(_, file, pos, len)) => {
                if pos as usize + len <= avatar.len() {
                    file.send(pos, &avatar[pos as usize .. pos as usize + len]).ok();
                }
            },
            Ok(Event::FriendMessage(friend, ty, message)) => {
                match message.as_slice() {
                    b"/invite" => {
                        if !group.invite(&friend) {
                            friend.say("invite fail.").ok();
                        }
                    },
                    b"/help" => { friend.say("TODO").ok(); },
                    mut msg @ _ => {
                        if msg.starts_with(b"/ ") {
                            msg = &msg[2..];
                        }

                        if check!(keyword config, "open_group", friend) {
                            group.send(ty, format!(
                                "({}) {}",
                                String::from_utf8_lossy(&friend.name().unwrap_or("Unknown".into())),
                                String::from_utf8_lossy(msg)
                            )).ok();
                        }
                    }
                }
            },
            Ok(Event::GroupInvite(friend, ty, token)) => {
                if !check!(master config, "pk", k, {
                    try_loop!(friend.publickey()) == try_loop!(try_unwrap!(k.as_str()).parse())
                }) {
                    continue
                };

                match match ty {
                    GroupType::TEXT => bot.join(&friend, &token),
                    GroupType::AV => bot.join_av(&friend, &token, Box::new(|_,_,_,_,_,_| ()))
                } {
                    Ok(g) => {
                        group.leave();
                        group = g;
                    },
                    Err(_) => { friend.say("join fail.").ok(); }
                };
            },
            Ok(Event::GroupTitle(_, peer_opt, title)) => {
                let msg = format!(
                    "* ({}) title: {}",
                    String::from_utf8_lossy(
                        &peer_opt
                            .and_then(|r| r.name().ok())
                            .unwrap_or("Unknown".into())
                    ),
                    String::from_utf8_lossy(&title)
                );
                log!(write (config, today), logfd, msg);
            },
            Ok(Event::GroupMessage(_, peer, mty, msg)) => {
                let msg = format!(
                    "{}({}) {}",
                    match mty {
                        MessageType::NORMAL => "",
                        MessageType::ACTION => "* "
                    },
                    String::from_utf8_lossy(&peer.name().unwrap_or("Unknown".into())),
                    String::from_utf8_lossy(&msg)
                );
                log!(write (config, today), logfd, msg);

                if !peer.is_ours() {
                    // TODO group
                }
            },
            Ok(Event::GroupPeerChange(_, peer, change)) => {
                let peer_pk = try_loop!(peer.publickey());
                let msg = format!(
                    "* ({}) {}: {}",
                    peer_pk,
                    match change {
                        PeerChange::ADD => "join",
                        PeerChange::DEL => "leave",
                        PeerChange::NAME => "rename"
                    },
                    String::from_utf8_lossy(&peer.name().unwrap_or("Unknown".into()))
                );
                log!(write (config, today), logfd, msg);
                match change {
                    PeerChange::ADD => {
                        if check!(keyword
                            config,
                            "open_offline_message",
                            try_loop!(bot.get_friend(peer_pk))
                        ) {
                            // TODO fake offline message
                        }
                    },
                    PeerChange::DEL => {
                        leave_time.insert(peer_pk, UTC::now());
                    },
                    _ => ()
                };
            },
            Err(_) => (),
            e @ _ => println!("Event: {:?}", e)
        }
    }
}
