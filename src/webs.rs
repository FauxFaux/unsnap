use std::cell::RefCell;
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
    fn twitter_get(&self, sub: &str) -> Result<Value, Error>;
    fn youtube_get(&self, url_suffix: &str, body: &HashMap<&str, &str>) -> Result<Value, Error>;
    fn raw_get<U: IntoUrl>(&self, url: U) -> Result<Resp, Error>;
}

pub struct Internet {
    config: Config,
    client: Client,
    twitter_token: RefCell<Option<String>>,
}

pub struct Resp {
    inner: Response,
}

impl Internet {
    pub fn new(config: Config) -> Internet {
        Internet {
            config,
            client: Client::new(),
            twitter_token: RefCell::default(),
        }
    }
}

impl Webs for Internet {
    fn imgur_get(&self, sub: &str) -> Result<Value, Error> {
        Ok(self
            .client
            .get(&format!("https://api.imgur.com/3/{}", sub))
            .header(Authorization(format!(
                "Client-ID {}",
                &self.config.keys.imgur_client_id
            )))
            .send()?
            .json()
            .context("bad json from imgur")?)
    }

    fn twitter_get(&self, sub: &str) -> Result<Value, Error> {
        if self.twitter_token.borrow().is_none() {
            self.update_twitter_token()?;
        }
        Ok(self
            .client
            .get(&format!("https://api.twitter.com/{}", sub))
            .header(Authorization(self.twitter_token.borrow().clone().unwrap()))
            .send()?
            .json()
            .context("bad json from twitter")?)
    }

    fn youtube_get<'s>(
        &self,
        url_suffix: &str,
        body: &HashMap<&str, &str>,
    ) -> Result<Value, Error> {
        let mut args = hashmap! {"key" => self.config.keys.youtube_developer_key.as_str() };
        args.extend(body);

        let url = Url::parse_with_params(
            &format!("https://www.googleapis.com/youtube/{}", url_suffix),
            args,
        ).unwrap();

        Ok(self
            .client
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

impl Internet {
    fn update_twitter_token(&self) -> Result<(), Error> {
        let token_body: Value = self
            .client
            .post("https://api.twitter.com/oauth2/token")
            .basic_auth(
                self.config.keys.twitter_app_key.to_string(),
                Some(self.config.keys.twitter_app_secret.to_string()),
            )
            .form(&hashmap! {
                "grant_type" => "client_credentials",
            })
            .send()?
            .json()
            .context("bad json from twitter auth")?;

        if let Some(val) = token_body.get("token_type") {
            match val.as_str() {
                Some("bearer") => (),
                other => bail!(
                    "invalid token_type in oauth response: {:?} {:?}",
                    other,
                    token_body
                ),
            }
        } else {
            bail!("no token type in oauth response: {:?}", token_body);
        }

        if let Some(val) = token_body.get("access_token") {
            if let Some(token) = val.as_str() {
                self.twitter_token
                    .replace(Some(format!("Bearer {}", token)));
                Ok(())
            } else {
                bail!(
                    "non-string bearer token in oauth response: {:?}",
                    token_body
                )
            }
        } else {
            bail!("no access_token in oauth response: {:?}", token_body)
        }
    }
}

impl Read for Resp {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}
