extern crate tox;
extern crate toml;

use std::thread::sleep;
use tox::core::{
    ToxOptions, Event,
    Network, Status, Chat, Listen
};
use tox::core::group::{ GroupManage, GroupCreate, GroupType };


fn main() {
    let mut bot = init();
    let botiter = bot.iterate();

    loop {
        sleep(bot.interval());
        match botiter.try_recv() {
            Ok(Event::SelfConnection(_)) => {
                // check groupbot is friend.
                unimplemented!()
            },
            Ok(Event::FriendConnection(_, _)) => {
                // invite logbot to group
                // send avatar
                unimplemented!()
            },
            Ok(Event::FriendFileChunkRequest(_, _, _, _)) => {
                // send avatar
                unimplemented!()
            },
            Ok(Event::FriendMessage(_, _, _)) => {
                // command
                unimplemented!()
            },
            Ok(Event::GroupInvite(_, _, _) => {
                // check pk & join
                unimplemented!()
            },
            Ok(Event::GroupTitle(_, _, _, _)) | Ok(Event::GroupMessage(_, _, _, _)) => {
                // write log
                unimplemented!()
            },
            Ok(Event::GroupPeerChange(_, _, _) => {
                // fake offline message & join/leave log
                unimplemented!()
            }
        }
    }
}

fn init() {
    unimplemented!()
}
