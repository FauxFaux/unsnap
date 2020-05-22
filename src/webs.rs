use std::cell::RefCell;
use std::collections::HashMap;
use std::time;

use anyhow::bail;
use anyhow::format_err;
use anyhow::Context;
use anyhow::Result;
use maplit::hashmap;
use reqwest::Client;
use reqwest::Response;
use serde_json::Value;

use crate::config::Config;

pub trait Webs {
    fn client(&self) -> &Client;
    fn config(&self) -> &Config;
    fn state(&self) -> &State;
}

pub struct Internet {
    client: Client,
    config: Config,
    state: State,
}

impl Webs for Internet {
    fn client(&self) -> &Client {
        &self.client
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn state(&self) -> &State {
        &self.state
    }
}

pub struct Explode;

impl Webs for Explode {
    fn client(&self) -> &Client {
        unimplemented!()
    }

    fn config(&self) -> &Config {
        unimplemented!()
    }

    fn state(&self) -> &State {
        unimplemented!()
    }
}

impl Internet {
    pub fn new(config: Config) -> Internet {
        let ua = chrome_ua();
        info!("UA: {}", ua);
        Internet {
            client: Client::new(),
            config,
            state: State::default(),
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

pub fn errors(resp: Response) -> Result<Response> {
    if !resp.status().is_success() {
        bail!("bad response code: {}", resp.status())
    }

    Ok(resp)
}

pub async fn imgur_get(client: &Client, config: &Config, sub: &str) -> Result<Value> {
    let resp = client
        .get(&format!("https://api.imgur.com/3/{}", sub))
        .header(
            "Authorization",
            &format!("Client-ID {}", &config.keys.imgur_client_id),
        )
        .send()
        .await?;
    Ok(resp.json().await.context("bad json from imgur")?)
}

pub async fn twitter_get(
    client: &Client,
    config: &Config,
    state: &State,
    sub: &str,
) -> Result<Value> {
    if state.twitter_token.borrow().is_none() {
        state.update_twitter_token(client, config).await?;
    }
    let resp = errors(
        client
            .get(&format!("https://api.twitter.com/{}", sub))
            .header(
                "Authorization",
                &state.twitter_token.borrow().clone().unwrap(),
            )
            .send()
            .await?,
    )?;
    Ok(resp.json().await.context("bad json from twitter")?)
}

pub async fn youtube_get(
    client: &Client,
    config: &Config,
    url_suffix: &str,
    body: &HashMap<&str, &str>,
) -> Result<Value> {
    let mut args = hashmap! {"key" => config.keys.youtube_developer_key.as_str() };
    args.extend(body);

    let url = url::Url::parse_with_params(
        &format!("https://www.googleapis.com/youtube/{}", url_suffix),
        args,
    )
    .unwrap();

    Ok(errors(client.get(url.as_str()).send().await?)?
        .json()
        .await
        .context("bad json from youtube")?)
}

#[derive(Default)]
pub struct State {
    twitter_token: RefCell<Option<String>>,
}

impl State {
    async fn update_twitter_token(&self, client: &Client, config: &Config) -> Result<()> {
        let token_body: Value = errors(
            client
                .post("https://api.twitter.com/oauth2/token")
                .basic_auth(
                    &config.keys.twitter_app_key,
                    Some(&config.keys.twitter_app_secret),
                )
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body("grant_type=client_credentials")
                .send()
                .await?,
        )?
        .json()
        .await
        .with_context(|| "bad json from twitter auth")?;

        self.twitter_token.replace(Some(format!(
            "Bearer {}",
            extract_token(&token_body)
                .with_context(|| format_err!("processing oauth response: {:?}", token_body))?
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

fn extract_token(token_body: &Value) -> Result<String> {
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

pub fn content_length(resp: &Response) -> Option<f64> {
    resp.headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
}

pub fn content_type(resp: &Response) -> Option<&str> {
    resp.headers()
        .get("Content-Type")
        .and_then(|v| v.to_str().ok())
}

pub async fn read_many(inner: &mut Response, mut buf: &mut [u8]) -> Result<usize> {
    let mut total = 0;
    while let Some(chunk) = inner.chunk().await? {
        let to_put = chunk.len().min(buf.len());
        buf[..to_put].copy_from_slice(&chunk[..to_put]);
        buf = &mut buf[to_put..];
        total += to_put;
    }

    Ok(total)
}
