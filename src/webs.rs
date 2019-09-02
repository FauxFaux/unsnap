use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::time;

use failure::bail;
use failure::format_err;
use failure::Error;
use failure::ResultExt;
use maplit::hashmap;
use serde_json::Value;
use ureq::Response;

use crate::config::Config;

// This is an interface, for generics-based dispatch. I made my decision, aware of the issues.
pub trait Webs {
    fn imgur_get(&self, sub: &str) -> Result<Value, Error>;
    fn twitter_get(&self, sub: &str) -> Result<Value, Error>;
    fn youtube_get(&self, url_suffix: &str, body: &HashMap<&str, &str>) -> Result<Value, Error>;
    fn raw_get<U: AsRef<str>>(&self, url: U) -> Result<Resp, Error>;
}

pub struct Internet {
    config: Config,
    client: ureq::Agent,
    twitter_token: RefCell<Option<String>>,
}

pub struct Resp {
    inner: Response,
}

impl Internet {
    pub fn new(config: Config) -> Internet {
        let ua = chrome_ua();
        info!("UA: {}", ua);
        Internet {
            config,
            client: ureq::agent().set("User-Agent", &ua).build(),
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

fn errors(resp: ureq::Response) -> Result<Response, Error> {
    if let Some(err) = resp.synthetic_error() {
        // TODO: chainable?
        bail!("http client error: {:?}", err);
    }

    if !resp.ok() {
        bail!("bad response code: {}", resp.status())
    }

    Ok(resp)
}

impl Webs for Internet {
    fn imgur_get(&self, sub: &str) -> Result<Value, Error> {
        Ok(self
            .client
            .get(&format!("https://api.imgur.com/3/{}", sub))
            .set(
                "Authorization",
                &format!("Client-ID {}", &self.config.keys.imgur_client_id),
            )
            .call()
            .into_json()
            .context("bad json from imgur")?)
    }

    fn twitter_get(&self, sub: &str) -> Result<Value, Error> {
        if self.twitter_token.borrow().is_none() {
            self.update_twitter_token()?;
        }
        Ok(errors(
            self.client
                .get(&format!("https://api.twitter.com/{}", sub))
                .set(
                    "Authorization",
                    &self.twitter_token.borrow().clone().unwrap(),
                )
                .call(),
        )?
        .into_json()
        .context("bad json from twitter")?)
    }

    fn youtube_get<'s>(
        &self,
        url_suffix: &str,
        body: &HashMap<&str, &str>,
    ) -> Result<Value, Error> {
        let mut args = hashmap! {"key" => self.config.keys.youtube_developer_key.as_str() };
        args.extend(body);

        let url = url::Url::parse_with_params(
            &format!("https://www.googleapis.com/youtube/{}", url_suffix),
            args,
        )
        .unwrap();

        Ok(errors(self.client.get(url.as_str()).call())?
            .into_json()
            .context("bad json from youtube")?)
    }

    fn raw_get<U: AsRef<str>>(&self, url: U) -> Result<Resp, Error> {
        let inner = errors(self.client.get(url.as_ref()).call())?;
        Ok(Resp { inner })
    }
}

impl Internet {
    fn update_twitter_token(&self) -> Result<(), Error> {
        let token_body: Value = errors(
            self.client
                .post("https://api.twitter.com/oauth2/token")
                .auth(
                    &self.config.keys.twitter_app_key,
                    &self.config.keys.twitter_app_secret,
                )
                .set("Content-Type", "application/x-www-form-urlencoded")
                .send_string("grant_type=client_credentials"),
        )?
        .into_json()
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
            .header("Content-Length")
            .and_then(|v| v.parse().ok())
    }

    pub fn content_type(&self) -> Option<&str> {
        self.inner.header("Content-Type")
    }

    pub fn into_reader(self) -> impl Read {
        self.inner.into_reader()
    }
}
