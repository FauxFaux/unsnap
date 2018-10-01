extern crate chrono;
#[macro_use]
extern crate failure;
extern crate iowrap;
extern crate irc;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
extern crate number_prefix;
extern crate reqwest;
extern crate result;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate time_parse;
extern crate toml;
extern crate twoway;

mod config;
mod files;
mod titles;
mod webs;

use failure::Error;
use irc::client::prelude::*;

use crate::webs::Webs;

fn main() -> Result<(), Error> {
    let config: config::Config = toml::from_slice(&files::load_bytes("bot.toml")?)?;

    let irc_config = irc::client::prelude::Config {
        nickname: Some(config.server.nick.to_string()),
        server: Some(config.server.hostname.to_string()),
        username: config.server.user.clone(),
        channels: Some(config.server.channels.clone()),
        ..Default::default()
    };

    let webs = webs::Internet::new(config);

    let mut async_bullshit = irc::client::prelude::IrcReactor::new()?;
    let client = async_bullshit.prepare_client_and_connect(&irc_config)?;

    client.identify()?;

    async_bullshit.register_client_with_handler(client, move |client, message| {
        if let Err(e) = handle(&webs, client, &message) {
            eprintln!("processing error: {:?}: {:?}", message, e);
        }
        Ok(())
    });

    async_bullshit.run()?;
    Ok(())
}

fn handle<W: Webs>(webs: &W, client: &IrcClient, message: &Message) -> Result<(), Error> {
    println!("<- {:?}", message);

    match message.command {
        Command::PRIVMSG(ref dest, ref msg) => {
            if let Some(nick) = message.source_nickname() {
                process_msg(webs, nick, &msg, |s| Ok(client.send_notice(dest, s)?))?
            }
        }
        _ => (),
    }

    Ok(())
}

fn process_msg<F, W: Webs>(webs: &W, nick: &str, msg: &str, mut write: F) -> Result<(), Error>
where
    F: FnMut(&str) -> Result<(), Error>,
{
    for title in titles::titles_for(webs, msg) {
        let title = title?;
        write(&format!("{}: {}", nick, title))?;
    }
    Ok(())
}
