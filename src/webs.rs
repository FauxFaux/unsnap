use std::io;
use std::io::Read;

use reqwest::header::Authorization;
use reqwest::Client;
use reqwest::IntoUrl;
use reqwest::Response;
use serde_json::Value;

use config::Config;
use errors::*;

// This is an interface, for generics-based dispatch. I made my decision, aware of the issues.
pub trait Webs {
    fn imgur_get(&self, sub: &str) -> Result<Value>;
    fn raw_get<U: IntoUrl>(&self, url: U) -> Result<Resp>;
}

pub struct Internet {
    config: Config,
    client: Client,
}

pub struct Resp {
    inner: Response,
}

impl Internet {
    pub fn new(config: Config) -> Internet {
        Internet {
            config,
            client: Client::new(),
        }
    }
}

impl Webs for Internet {
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

    fn raw_get<U: IntoUrl>(&self, url: U) -> Result<Resp> {
        let inner = self.client.get(url).send()?;
        Ok(Resp { inner })
    }
}

impl Read for Resp {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}
