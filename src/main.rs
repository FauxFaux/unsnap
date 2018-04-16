#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate hyper;

extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate number_prefix;
extern crate reqwest;
extern crate rustls;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate webpki;
extern crate webpki_roots;

mod comms;
mod config;
mod errors;
mod files;
mod parse;
mod titles;
mod webs;

use errors::*;
use parse::Command;
use parse::Ident;
use parse::Targeted;
use webs::Webs;

quick_main!(run);

#[derive(Copy, Clone, Debug)]
enum State {
    Connecting,
    Connected,
}

fn run() -> Result<()> {
    let config: config::Config = toml::from_slice(&files::load_bytes("bot.toml")?)?;

    let mut state = State::Connecting;
    let mut webs = webs::Internet::new(&config);

    let mut conn = comms::Comm::connect(&config.server.hostname, config.server.port)?;
    conn.write_line(format!(
        "USER {} * * :{}",
        config.server.user.as_ref().unwrap_or(&config.server.nick),
        config
            .server
            .real_name
            .as_ref()
            .unwrap_or(&config.server.nick)
    ))?;
    conn.write_line(format!("NICK {}", &config.server.nick))?;

    loop {
        let line = match conn.read_line()? {
            Some(line) => line,
            None => continue,
        };

        let parsed = parse::parse(&line)?;
        println!("<- {:?}", parsed);

        match parsed.command {
            Command::Ping(token) => conn.write_line(format!("PONG {}", token))?,
            Command::Numeric(1, _) => {
                state = State::Connected;
                conn.write_line("CAP LS 302")?;
                for channel in &config.server.channels {
                    conn.write_line(format!("JOIN {}", channel))?;
                }
            }
            Command::PrivMsg(Targeted { dest, msg }) => {
                process_msg(&mut webs, &parsed.whom, dest, msg, |s| conn.write_line(s))?
            }
            _ => (),
        }
    }

    Ok(())
}

fn process_msg<F, W: Webs>(
    webs: &mut W,
    whom: &Ident,
    into: &str,
    msg: &str,
    mut write: F,
) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    for title in titles::titles_for(webs, msg) {
        let title = title?;
        write(&format!("NOTICE {} :{}: {}", into, whom.nick(), title))?;
    }
    Ok(())
}
