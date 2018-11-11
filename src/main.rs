extern crate cast;
extern crate chrono;
#[macro_use]
extern crate failure;
extern crate iowrap;
extern crate irc;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
extern crate number_prefix;
extern crate pretty_env_logger;
extern crate reqwest;
extern crate result;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate subprocess;
extern crate tempfile;
extern crate time_parse;
extern crate toml;
extern crate twoway;

mod config;
mod danger;
mod files;
mod titles;
mod webs;

use failure::Error;
use failure::ResultExt;
use irc::client::prelude::*;

use crate::webs::Webs;

fn main() -> Result<(), Error> {
    pretty_env_logger::try_init()?;

    let config: config::Config = toml::from_slice(&files::load_bytes("bot.toml")?)?;

    let irc_config = irc::client::prelude::Config {
        nickname: Some(config.server.nick.to_string()),
        server: Some(config.server.hostname.to_string()),
        username: config.server.user.clone(),
        channels: Some(config.server.channels.clone()),
        nick_password: config.server.nick_password.clone(),
        ..Default::default()
    };

    let webs = webs::Internet::new(config);

    let mut async_bullshit = irc::client::prelude::IrcReactor::new()?;
    let client = async_bullshit.prepare_client_and_connect(&irc_config)?;

    client.identify()?;

    async_bullshit.register_client_with_handler(client, move |client, message| {
        if let Err(e) = handle(&webs, client, &message) {
            warn!("processing error: {:?}: {:?}", message, e);
        }
        Ok(())
    });

    async_bullshit.run()?;
    Ok(())
}

fn handle<W: Webs>(webs: &W, client: &IrcClient, message: &Message) -> Result<(), Error> {
    trace!("<- {:?}", message);

    match message.command {
        Command::PRIVMSG(ref dest, ref msg) => {
            if let Some(nick) = message.source_nickname() {
                process_msg(webs, nick, &msg, |s| {
                    Ok(client
                        .send_notice(dest, s)
                        .with_context(|_| format_err!("replying to {:?}", dest))?)
                })
                .with_context(|_| format_err!("processing < {:?}> {:?}", nick, msg))?
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
    if msg.starts_with("!qalc ") {
        let input = &msg["!qalc".len()..];
        match danger::qalc(input) {
            Ok(resp) => write(&format!("{}: {}", nick, limit_length(&resp)))?,
            Err(e) => {
                write(&format!("{}: It did not work.", nick))?;
                error!("qalc {:?} failed: {:?}", input, e);
            }
        }
        return Ok(());
    }

    for title in titles::titles_for(webs, msg) {
        let title = title?;
        write(&format!("{}: {}", nick, limit_length(&title)))?;
    }
    Ok(())
}

fn limit_length(val: &str) -> &str {
    for end in 365..400 {
        if val.is_char_boundary(end) {
            return &val[..end];
        }
    }

    val
}
