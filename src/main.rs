#[macro_use]
extern crate log;

mod config;
mod content;
mod danger;
mod files;
mod titles;
mod webs;

use anyhow::format_err;
use anyhow::Context;
use anyhow::Result;
use futures::prelude::*;
use irc::client::prelude as ic;

use crate::webs::Webs;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::try_init()?;

    let config: config::Config = toml::from_slice(&files::load_bytes("bot.toml")?)?;

    let irc_config = ic::Config {
        nickname: Some(config.server.nick.to_string()),
        server: Some(config.server.hostname.to_string()),
        username: config.server.user.clone(),
        channels: config.server.channels.clone(),
        password: config.server.password.clone(),
        nick_password: config.server.nick_password.clone(),

        // freenode takes over 10s to warm up, including hostname verification failure
        ping_timeout: Some(20),
        ..Default::default()
    };

    let webs = webs::Internet::new(config);

    let mut client = ic::Client::from_config(irc_config).await?;

    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        if let Err(e) = handle(&webs, &client, &message).await {
            warn!("processing error: {:?}: {:?}", message, e);
        }
    }

    Ok(())
}

async fn handle<W: Webs>(webs: &W, client: &ic::Client, message: &ic::Message) -> Result<()> {
    info!("<- {:?}", message);

    match message.command {
        ic::Command::PRIVMSG(ref dest, ref msg) => {
            if let Some(nick) = message.source_nickname() {
                process_msg(webs, nick, &msg, |s| {
                    Ok(client
                        .send_privmsg(dest, s)
                        .with_context(|| format_err!("replying to {:?}", dest))?)
                })
                .await
                .with_context(|| format_err!("processing < {:?}> {:?}", nick, msg))?
            }
        }
        _ => (),
    }

    Ok(())
}

async fn process_msg<F, W: Webs>(webs: &W, nick: &str, msg: &str, mut write: F) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
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

    for title in titles::titles_for(webs, msg).await? {
        assert!(!title.contains(|c: char| c.is_control()));
        write(&limit_length(&title))?;
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
