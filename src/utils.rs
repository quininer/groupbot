use std::path::Path;
use std::fs::File;
use std::io::{ Read, Write };
use toml::{ Parser, Table };

use tox::core::{
    Tox, ToxOptions,
    Network
};

macro_rules! try_loop {
    ( $exp:expr ) => {
        match $exp {
            Ok(out) => out,
            Err(_) => {
                // warn
                continue
            }
        }
    }
}

macro_rules! try_unwrap {
    ( $exp:expr ) => {
        match $exp {
            Some(out) => out,
            None => {
                // warn
                continue
            }
        }
    }
}

macro_rules! check {
    ( $config:expr, $lookup:expr, $k:ident, $pass:block ) => {{
        let mut pass = true;
        if let Some(kk) = $config.get("master").and_then(|r| r.lookup($lookup)) {
            pass = false;
            for $k in try_unwrap!(kk.as_slice()) {
                pass = $pass || pass;
            }
        }
        pass
    }}
}

pub fn parse_config<P: AsRef<Path>>(path: P) -> Table {
    let mut data = String::new();
    File::open(path).unwrap()
        .read_to_string(&mut data).unwrap();
    Parser::new(&data).parse()
        .unwrap()
}

pub fn save(path: &str, bot: &Tox) {
    File::create(path).unwrap()
        .write(&bot.save()).ok();
}

pub fn init(config: &Table) -> (Tox, Vec<u8>, String) {
    let path = config.get("profile")
        .and_then(|r| r.as_str()).unwrap();

    let bot = match File::open(path) {
        Ok(mut fd) => {
            let mut data = Vec::new();
            fd.read_to_end(&mut data).unwrap();
            ToxOptions::new()
                .from(&data)
                .generate().unwrap()
            },
        Err(_) => {
            let bot = ToxOptions::new().generate().unwrap();
            save(path, &bot);
            bot
        }
    };

    bot.set_name(
        config.get("name")
            .and_then(|r| r.as_str())
            .unwrap_or("groupbot")
    ).ok();
    bot.set_status_message(
        config.get("status_message")
            .and_then(|r| r.as_str())
            .unwrap_or("say '/help' to me.")
    ).ok();
    bot.bootstrap(
        config.get("bootstrap_addr").and_then(|r| r.as_str()).unwrap(),
        config.get("bootstrap_pk").and_then(|r| r.as_str()).unwrap()
            .parse().unwrap()
    ).ok();

    let mut avatar_data = Vec::new();
    if let Some(avatar_path) = config.get("avatar").and_then(|r| r.as_str()) {
        if let Ok(mut fd) = File::open(avatar_path) {
            fd.read_to_end(&mut avatar_data).unwrap();
        }
    }

    (bot, avatar_data, path.into())
}

pub fn check_status(status_message: &[u8], keyword: &[u8]) -> bool {
    unimplemented!()
}
