use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::time;

use failure::Error;
use failure::ResultExt;
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::IntoUrl;
use reqwest::Response;
use reqwest::Url;
use serde_json::Value;

use crate::config::Config;

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
        let mut headers = reqwest::header::HeaderMap::new();
        let ua = chrome_ua();
        info!("UA: {}", ua);
        headers.insert("User-Agent", ua.parse().unwrap());
        Internet {
            config,
            client: ClientBuilder::new()
                .default_headers(headers)
                .build()
                .unwrap(),
            twitter_token: RefCell::default(),
        }
    }
}

fn chrome_ua() -> String {
    let now = epoch_secs();
    let apple = (now - 800_000_000) / 1_400_212;
    let chrome = (now - 1_234_567_890) / 4_400_212;
    format!(
        concat!(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/{apple}.36 ",
            "(KHTML, like Gecko) Chrome/{chrome}.0.3538.110 Safari/{apple}.36"
        ),
        apple = apple,
        chrome = chrome
    )
}

impl Webs for Internet {
    fn imgur_get(&self, sub: &str) -> Result<Value, Error> {
        Ok(self
            .client
            .get(&format!("https://api.imgur.com/3/{}", sub))
            .header(
                AUTHORIZATION,
                format!("Client-ID {}", &self.config.keys.imgur_client_id),
            )
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
            .header(AUTHORIZATION, self.twitter_token.borrow().clone().unwrap())
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
        )
        .unwrap();

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

        self.twitter_token.replace(Some(format!(
            "Bearer {}",
            extract_token(&token_body)
                .with_context(|_| format_err!("processing oauth response: {:?}", token_body))?
        )));

        Ok(())
    }
}

fn epoch_secs() -> u64 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn extract_token(token_body: &Value) -> Result<String, Error> {
    if token_body
        .get("token_type")
        .ok_or_else(|| format_err!("no token_type"))?
        .as_str()
        .ok_or_else(|| format_err!("non-string token_type"))?
        != "bearer"
    {
        bail!("invalid/missing token_type in oauth response");
    }

    Ok(token_body
        .get("access_token")
        .ok_or_else(|| format_err!("no access_token"))?
        .as_str()
        .ok_or_else(|| format_err!("non-string bearer token"))?
        .to_string())
}

impl Resp {
    pub fn content_length(&self) -> Option<f64> {
        self.inner
            .headers()
            .get("Content-Length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
    }

    pub fn content_type(&self) -> Option<&str> {
        self.inner
            .headers()
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
    }
}

impl Read for Resp {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}
