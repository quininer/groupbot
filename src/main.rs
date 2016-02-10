extern crate tox;
extern crate toml;
extern crate chrono;

#[macro_use] mod utils;

use std::env::args;
use std::thread::sleep;
use std::collections::HashSet;
use chrono::offset::utc::UTC;
use tox::core::{
    Event,
    Status, Chat, Listen
};
use tox::core::status::Connection;
use tox::core::file::{ FileKind, FileOperate, FileManage };
use tox::core::group::{ GroupCreate, GroupType };
use tox::av::AvGroupCreate;

use utils::{ parse_config, init };


fn main() {
    let config = parse_config(args().skip(1).next().unwrap_or("./config.toml".into()));
    let (mut bot, avatar) = init(config.get("bot").and_then(|r| r.as_table()).unwrap());
    println!("Tox ID: {}", bot.address());

    let botiter = bot.iterate();
    // let mut group = bot.create_group_av().unwrap();
    let mut group = bot.create_group().unwrap();
    let mut avatar_sended = HashSet::new();
    let mut today = UTC::today();

    'main: loop {
        sleep(bot.interval());
        match botiter.try_recv() {
            Ok(Event::FriendConnection(friend, connection)) => {
                if UTC::today() != today {
                    avatar_sended = HashSet::new();
                    today = now;
                }
                if connection == Connection::NONE { continue };
                // TODO check friend status
                if !avatar_sended.insert(try_loop!(friend.publickey())) {
                    friend.transmission(FileKind::AVATAR, "avatar.png", avatar.len() as u64, None).ok();
                }

                // TODO check friend status
                group.invite(&friend);
            },
            Ok(Event::FriendFileChunkRequest(_, file, pos, len)) => {
                if pos as usize + len < avatar.len() {
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
                        // TODO check friend status
                        // TODO write log
                        group.send(ty, format!(
                            "({}) {}",
                            String::from_utf8_lossy(&friend.name().unwrap_or("unknown".into())),
                            String::from_utf8_lossy(msg)
                        )).ok();
                    }
                }
            },
            Ok(Event::GroupInvite(friend, ty, token)) => {
                // TODO check master pk
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
            Ok(Event::GroupTitle(_, _, _)) => {
                // write log
                unimplemented!()
            },
            Ok(Event::GroupMessage(_, _, _, _)) => {
                // write log
                unimplemented!()
            },
            Ok(Event::GroupPeerChange(_, _, _)) => {
                // fake offline message & join/leave log
                // unimplemented!()
                ()
            },
            Err(_) => (),
            e @ _ => println!("Event: {:?}", e)
        }
    }
}
