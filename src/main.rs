extern crate tox;
extern crate toml;
extern crate chrono;
extern crate rustc_serialize;

#[macro_use] mod utils;

use std::env::args;
use std::path::Path;
use std::thread::sleep;
use std::fs::OpenOptions;
use std::io::{ Write, Read };
use std::collections::HashMap;
use chrono::UTC;
use rustc_serialize::base64::{ FromBase64, ToBase64, STANDARD };
use tox::core::{
    Event,
    Status, Chat, Listen, FriendManage
};
use tox::core::status::Connection;
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

                if !check!(keyword config, "off_invite", friend)
                    && !check!(keyword config, "open_group", friend)
                {
                    group.invite(&friend);
                }
            },
            Ok(Event::RequestFriend(pk, _)) => {
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
                    b"/id" => {
                        friend.say(format!("{}", bot.address())).ok();
                    },
                    b"/help" => { friend.say("TODO").ok(); },
                    mut msg @ _ => {
                        if msg.starts_with(b"/ ") {
                            msg = &msg[2..];
                        }

                        if check!(keyword config, "open_group", friend) {
                            let msg = format!(
                                "({}) {}",
                                String::from_utf8_lossy(&friend.name().unwrap_or("Unknown".into())),
                                String::from_utf8_lossy(msg)
                            );
                            group.send(ty, &msg).ok();

                            for f in bot.list_friend() {
                                if check!(keyword config, "open_group", f)
                                    && f.publickey().ok() != friend.publickey().ok()
                                {
                                    f.send(ty, format!("(groupbot) {}", &msg)).ok();
                                }
                            }
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
                    "title ({}): {}",
                    String::from_utf8_lossy(
                        &peer_opt
                            .and_then(|r| r.name().ok())
                            .unwrap_or("Unknown".into())
                    ),
                    String::from_utf8_lossy(&title)
                );
                log!(write (config, today), logfd, &msg);

                for f in bot.list_friend() {
                    if check!(keyword config, "open_group", f) {
                        f.action(&msg).ok();
                    }
                }
            },
            Ok(Event::GroupMessage(_, peer, mty, msg)) => {
                let msg = format!(
                    "({}) {}",
                    String::from_utf8_lossy(&peer.name().unwrap_or("Unknown".into())),
                    String::from_utf8_lossy(&msg)
                );
                log!(write (config, today), logfd, format!(
                    "{}{}",
                    match mty {
                        MessageType::NORMAL => "",
                        MessageType::ACTION => "* "
                    },
                    &msg
                ));

                if !peer.is_ours() {
                    for f in bot.list_friend() {
                        if check!(keyword config, "open_group", f) {
                            f.send(mty, &msg).ok();
                        }
                    }
                }
            },
            Ok(Event::GroupPeerChange(_, peer, change)) => {
                // TODO new groupchat
                if change == PeerChange::DEL { continue };

                let peer_pk = try_loop!(peer.publickey());
                let msg = format!(
                    "{} {}",
                    String::from_utf8_lossy(&peer.name().unwrap_or("Unknown".into())),
                    match change {
                        PeerChange::ADD => "join",
                        PeerChange::NAME => "rename",
                        _ => unreachable!()
                    }
                );

                let f = bot.get_friend(peer_pk);
                if change == PeerChange::ADD && f.is_ok() {
                    let f = f.unwrap();
                    if check!(keyword config, "open_offline_message", f) {
                        for s in log!(read
                            (config, today),
                            leave_time.get(&peer_pk).unwrap_or(&UTC::now()).timestamp(),
                            UTC::now().timestamp()
                        ) {
                            if s.starts_with(b"* ") {
                                f.action(&s[2..]).ok();
                            } else {
                                f.say(&s).ok();
                            };
                        }
                    };
                };

                log!(write (config, today), logfd, format!("* {}", &msg));

                if !peer.is_ours() {
                    for f in bot.list_friend() {
                        if check!(keyword config, "open_group", f) {
                            f.action(&msg).ok();
                        };
                    }
                }


            },
            Ok(Event::FriendConnection(friend, connection)) => {
                let friend_pk = try_loop!(friend.publickey());
                match connection {
                    Connection::NONE => {
                        leave_time.insert(friend_pk, UTC::now());
                    },
                    Connection::TCP | Connection::UDP => {
                        if check!(keyword config, "open_group", friend)
                            && check!(keyword config, "open_offline_message", friend)
                        {
                            for s in log!(read
                                (config, today),
                                leave_time.get(&friend_pk).unwrap_or(&UTC::now()).timestamp(),
                                UTC::now().timestamp()
                            ) {
                                if s.starts_with(b"* ") {
                                    friend.action(&s[2..]).ok();
                                } else {
                                    friend.say(&s).ok();
                                };
                            }
                        };
                    }
                };
            },
            Err(_) => (),
            e @ _ => println!("Event: {:?}", e)
        }
    }
}
