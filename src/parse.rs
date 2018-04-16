use std::ascii::AsciiExt;

use errors::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message<'m> {
    pub whom: Ident<'m>,
    pub command: Command<'m>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident<'m> {
    inner: &'m str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Targeted<'m> {
    pub dest: &'m str,
    pub msg: &'m str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command<'m> {
    Numeric(u16, &'m str),
    Ping(&'m str),
    Notice(Targeted<'m>),
    PrivMsg(Targeted<'m>),
    Join(&'m str),
    Unknown(&'m str, &'m str),
}

pub fn parse(line: &str) -> Result<Message> {
    let (ident, line) = if line.starts_with(':') {
        line[1..].split_at(line.find(' ').ok_or("malformed leader")?)
    } else {
        ("", line)
    };

    let line = line.trim();

    let (word, line) = match line.find(' ') {
        Some(val) => line.split_at(val),
        None => (line, ""),
    };

    let line = line.trim();

    let command = match word {
        "PING" => Command::Ping(line),
        "PRIVMSG" => Command::PrivMsg(Targeted::new(line)?),
        "NOTICE" => Command::Notice(Targeted::new(line)?),
        other if other.is_ascii_digit() => Command::Numeric(other.parse()?, line),
        other => Command::Unknown(other, line),
    };

    Ok(Message {
        whom: Ident { inner: ident },
        command,
    })
}

impl<'a> Targeted<'a> {
    fn new(from: &'a str) -> Result<Targeted<'a>> {
        let (whom, msg) = from.split_at(from.find(' ').ok_or("invalid targeted")?);
        let msg = msg.trim();
        let msg = if msg.starts_with(':') { &msg[1..] } else { msg };

        Ok(Targeted { dest: whom, msg })
    }
}

impl<'a> Ident<'a> {
    pub fn nick(&self) -> &str {
        &self.inner[..self.inner.find('!').unwrap_or(self.inner.len())]
    }
}