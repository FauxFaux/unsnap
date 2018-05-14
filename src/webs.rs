use std::collections::HashMap;
use std::io;
use std::io::Read;

use failure::Error;
use failure::ResultExt;
use reqwest::header::Authorization;
use reqwest::Client;
use reqwest::IntoUrl;
use reqwest::Response;
use reqwest::Url;
use serde_json::Value;

use config::Config;

// This is an interface, for generics-based dispatch. I made my decision, aware of the issues.
pub trait Webs {
    fn imgur_get(&self, sub: &str) -> Result<Value, Error>;
    fn youtube_get(&self, url_suffix: &str, body: HashMap<&str, &str>) -> Result<Value, Error>;
    fn raw_get<U: IntoUrl>(&self, url: U) -> Result<Resp, Error>;
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
    fn imgur_get(&self, sub: &str) -> Result<Value, Error> {
        Ok(self.client
            .get(&format!("https://api.imgur.com/3/{}", sub))
            .header(Authorization(format!(
                "Client-ID {}",
                &self.config.keys.imgur_client_id
            )))
            .send()?
            .json()
            .context("bad json from imgur")?)
    }

    fn youtube_get<'s>(
        &self,
        url_suffix: &str,
        mut body: HashMap<&str, &str>,
    ) -> Result<Value, Error> {
        let mut args = hashmap! {"key" => self.config.keys.youtube_developer_key.as_str() };
        args.extend(body);

        let url = Url::parse_with_params(
            &format!("https://www.googleapis.com/youtube/{}", url_suffix),
            args,
        ).unwrap();

        Ok(self.client
            .get(url)
            .send()?
            .json()
            .context("bad json from youtube")?)
    }

    fn raw_get<U: IntoUrl>(&self, url: U) -> Result<Resp, Error> {
        let inner = self.client.get(url).send()?;
        Ok(Resp { inner })
    }
}

impl Read for Resp {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}
