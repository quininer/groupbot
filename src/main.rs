extern crate tox;
extern crate toml;

mod utils;

use std::env::args;
use std::thread::sleep;
use tox::core::{
    Tox, Event,
    Status, Chat, Listen
};
use tox::core::group::{
    Group,
    GroupManage, GroupCreate, GroupType
};

use utils::{
    parse_config, init
};


fn main() {
    let config = parse_config(args().next().unwrap_or("./config.toml".into()));
    let (mut bot, avatar) = init(config.get("bot").and_then(|r| r.as_table()).unwrap());
    let botiter = bot.iterate();

    loop {
        sleep(bot.interval());
        match botiter.try_recv() {
            Ok(Event::SelfConnection(_)) => {
                // check group & create
                unimplemented!()
            },
            Ok(Event::FriendConnection(_, _)) => {
                // invite to group
                // send avatar
                unimplemented!()
            },
            Ok(Event::FriendFileChunkRequest(_, _, _, _)) => {
                // send avatar
                unimplemented!()
            },
            Ok(Event::FriendMessage(_, _, _)) => {
                // command & transmit
                unimplemented!()
            },
            Ok(Event::GroupTitle(_, _, _)) | Ok(Event::GroupMessage(_, _, _, _)) => {
                // write log
                unimplemented!()
            },
            Ok(Event::GroupPeerChange(_, _, _)) => {
                // fake offline message & join/leave log
                unimplemented!()
            },
            Err(_) => (),
            e @ _ => println!("Event: {:?}", e)
        }
    }
}
