use std::path::Path;
use std::fs::File;
use std::io::{ Read, Write };
use toml::{ Parser, Table };

use tox::core::{
    Tox, ToxOptions,
    Network
};


pub fn parse_config<P: AsRef<Path>>(path: P) -> Table {
    let mut data = String::new();
    File::open(path).unwrap()
        .read_to_string(&mut data).unwrap();
    Parser::new(&data).parse()
        .unwrap()
}

pub fn init(config: &Table) -> (Tox, Vec<u8>) {
    let path = config.get("profile").and_then(|r| r.as_str()).unwrap();

    let mut im = match File::open(path) {
        Ok(mut fd) => {
            let mut data = Vec::new();
            fd.read_to_end(&mut data).unwrap();
            ToxOptions::new()
                .from(&data)
                .generate().unwrap()
            },
        Err(_) => {
            let mut im = ToxOptions::new().generate().unwrap();
            File::create(path).unwrap()
                .write(&im.save()).ok();
            im
        }
    };

    im.set_name(config.get("name").and_then(|r| r.as_str()).unwrap_or("groupbot")).ok();
    im.set_status_message(config.get("status_message").and_then(|r| r.as_str()).unwrap_or("say 'help' to me.")).ok();
    im.bootstrap(
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

    (im, avatar_data)
}
