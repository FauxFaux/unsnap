#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate rustls;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate webpki;
extern crate webpki_roots;

mod comms;
mod config;
mod errors;
mod files;
mod parse;

use errors::*;
use parse::Command;
use parse::Ident;
use parse::Targeted;

quick_main!(run);

#[derive(Copy, Clone, Debug)]
enum State {
    Connecting,
    Connected,
}

fn run() -> Result<()> {
    let config: config::Config = toml::from_slice(&files::load_bytes("bot.toml")?)?;

    let mut state = State::Connecting;

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
                process_msg(&parsed.whom, dest, msg, |s| conn.write_line(s))?
            }
            _ => (),
        }
    }

    Ok(())
}

fn process_msg<F>(whom: &Ident, into: &str, msg: &str, mut write: F) -> Result<()>
where
    F: FnMut(&str) -> Result<()>,
{
    write(&format!("PRIVMSG {} :{}: SHUT UP ABOUT {}", into, whom.nick(), msg))?;
    Ok(())
}
