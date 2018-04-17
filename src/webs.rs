use std::fmt;

use reqwest::header::Authorization;
use reqwest::Client;
use serde_json::Value;

use config::Config;
use errors::*;

// This is an interface, for generics-based dispatch. I made my decision, aware of the issues.
pub trait Webs {
    fn imgur_get(&self, sub: &str) -> Result<Value>;
}

pub struct Internet<'c> {
    config: &'c Config,
    client: Client,
}

#[cfg(never)]
struct ClientId {
    token: String,
}

#[cfg(never)]
impl Scheme for ClientId {
    fn scheme<'a>() -> Option<&'a str> {
        None
    }

    fn fmt_scheme(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client-ID {}", self.token)
    }
}

impl<'c> Internet<'c> {
    pub fn new(config: &Config) -> Internet {
        Internet {
            config,
            client: Client::new(),
        }
    }
}

impl<'c> Webs for Internet<'c> {
    fn imgur_get(&self, sub: &str) -> Result<Value> {
        self.client
            .get(&format!("https://api.imgur.com/3/{}", sub))
            .header(Authorization(format!(
                "Client-ID {}",
                &self.config.keys.imgur_client_id
            )))
            .send()?
            .json()
            .chain_err(|| format!("bad json from imgur"))
    }
}
