#[macro_use]
extern crate log;

mod config;
mod content;
mod danger;
mod titles;
mod webs;

use std::fs;

use anyhow::Context as _;
use anyhow::Result;
use anyhow::format_err;
use futures::prelude::*;
use irc::client::{Sender, prelude as ic};
use reqwest::Client;
use std::sync::Arc;

use crate::webs::Context;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::try_init()?;

    let config: config::Config = toml::from_str(&fs::read_to_string("bot.toml")?)?;

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

    let (http, context) = Context::new(config);
    let context = Arc::new(context);

    let mut client = ic::Client::from_config(irc_config).await?;

    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        let http = http.clone();
        let context = Arc::clone(&context);
        if let Err(e) = handle(http, context, &client, &message).await {
            warn!("processing error: {:?}: {:?}", message, e);
        }
    }

    Ok(())
}

async fn handle(
    http: Client,
    context: Arc<Context>,
    client: &ic::Client,
    message: &ic::Message,
) -> Result<()> {
    info!("<- {:?}", message);

    match message.command {
        ic::Command::PRIVMSG(ref dest, ref msg) => {
            if let Some(nick) = message.source_nickname() {
                tokio::spawn(process_msg_or_log(
                    http,
                    dest.to_string(),
                    client.sender(),
                    context,
                    nick.to_string(),
                    msg.to_string(),
                ));
            }
        }
        _ => (),
    }

    Ok(())
}

async fn process_msg_or_log(
    http: Client,
    dest: String,
    sender: Sender,
    context: Arc<Context>,
    nick: String,
    msg: String,
) -> () {
    if let Err(e) = process_msg(
        http,
        context,
        nick.to_string(),
        msg.to_string(),
        move |message| {
            sender
                .send_privmsg(dest.to_string(), message)
                .with_context(|| format_err!("replying to {:?}", dest))
        },
    )
    .await
    .with_context(|| format_err!("processing < {:?}> {:?}", nick, msg))
    {
        warn!("process_msg failed: {:?}", e)
    }
}

async fn process_msg<F>(
    http: Client,
    context: Arc<Context>,
    nick: String,
    msg: String,
    mut sender: F,
) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    if msg.starts_with("!qalc ") {
        let input = &msg["!qalc".len()..];
        match danger::qalc(input) {
            Ok(resp) => sender(&format!("{}: {}", nick, limit_length(&resp)))?,
            Err(e) => {
                sender(&format!("{}: It did not work.", nick))?;
                error!("qalc {:?} failed: {:?}", input, e);
            }
        }
        return Ok(());
    }

    for title in titles::titles_for(http, context, &msg).await? {
        assert!(!title.contains(|c: char| c.is_control()));
        sender(limit_length(&title))?;
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
