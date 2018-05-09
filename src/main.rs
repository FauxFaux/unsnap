#[macro_use]
extern crate error_chain;
extern crate iowrap;
extern crate irc;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate number_prefix;
extern crate reqwest;
extern crate result;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate twoway;

mod config;
mod errors;
mod files;
mod titles;
mod webs;

use irc::client::prelude::*;

use errors::*;
use webs::Webs;

quick_main!(run);

fn run() -> Result<()> {
    let config: config::Config = toml::from_slice(&files::load_bytes("bot.toml")?)?;

    let irc_config = irc::client::prelude::Config {
        nickname: Some(config.server.nick.to_string()),
        server: Some(config.server.hostname.to_string()),
        username: config.server.user.clone(),
        channels: Some(config.server.channels.clone()),
        ..Default::default()
    };

    let mut webs = webs::Internet::new(config);

    let mut async_bullshit = irc::client::prelude::IrcReactor::new().map_err(unerr)?;
    let client = async_bullshit
        .prepare_client_and_connect(&irc_config)
        .map_err(unerr)?;

    client.identify().map_err(unerr)?;

    async_bullshit.register_client_with_handler(client, move |client, message| {
        if let Err(e) = handle(&webs, client, &message) {
            eprintln!("processing error: {:?}: {:?}", message, e);
        }
        Ok(())
    });

    async_bullshit.run().map_err(unerr)?;
    Ok(())
}

fn handle<W: Webs>(webs: &W, client: &IrcClient, message: &Message) -> Result<()> {
    println!("<- {:?}", message);

    match message.command {
        Command::PRIVMSG(ref dest, ref msg) => if let Some(nick) = message.source_nickname() {
            process_msg(webs, nick, &msg, |s| {
                client.send_notice(dest, s).map_err(unerr)
            })?
        },
        _ => (),
    }

    Ok(())
}

fn unerr(err: irc::error::IrcError) -> Error {
    format!("irc error: {:?}", err).into()
}

fn process_msg<F, W: Webs>(webs: &W, nick: &str, msg: &str, mut write: F) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    for title in titles::titles_for(webs, msg) {
        let title = title?;
        write(&format!("{}: {}", nick, title))?;
    }
    Ok(())
}
