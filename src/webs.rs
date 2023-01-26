use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time;

use anyhow::bail;
use anyhow::format_err;
use anyhow::Context as _;
use anyhow::Result;
use maplit::hashmap;
use reqwest::Client;
use reqwest::Response;
use serde_json::Value;

use crate::config::Config;

pub struct Context {
    pub config: Config,
    pub state: State,
}

impl Context {
    pub fn new(config: Config) -> (Client, Context) {
        let ua = chrome_ua();
        info!("UA: {}", ua);
        let client = reqwest::ClientBuilder::new()
            .user_agent(ua)
            .build()
            .expect("infallible");
        (
            client,
            Context {
                config,
                state: State::default(),
            },
        )
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

pub async fn twitter_get(client: &Client, context: Arc<Context>, sub: &str) -> Result<Value> {
    if context
        .state
        .twitter_token
        .lock()
        .expect("poisoned")
        .is_none()
    {
        context
            .state
            .update_twitter_token(client, &context.config)
            .await?;
    }
    let token = context
        .state
        .twitter_token
        .lock()
        .expect("poisoned")
        .clone()
        .expect("populated above");
    let resp = errors(
        client
            .get(&format!("https://api.twitter.com/{}", sub))
            .header("Authorization", &token)
            .send()
            .await?,
    )?;
    Ok(resp.json().await.context("bad json from twitter")?)
}

pub async fn spotify_get(client: &Client, context: Arc<Context>, sub: &str) -> Result<Value> {
    // I think this is the worst threading code I've ever written.

    for retry in [true, false] {
        let token = maybe_refresh(client, &context).await?;

        let url = format!("https://api.spotify.com/v1/{}", sub);

        let resp = client
            .get(&url)
            .header("Authorization", &token)
            .send()
            .await
            .with_context(|| format_err!("network fetching {:?}", url))?;

        if resp.status().is_success() {
            return Ok(resp.json().await.context("bad json from spotify")?);
        }

        if resp.status().as_u16() == 401 {
            clear_token(&context);
            continue;
        }

        if !retry {
            bail!("spotify replied: {:?}", resp.status());
        }
    }

    unreachable!("out of spotify retries")
}

async fn maybe_refresh(client: &Client, context: &Arc<Context>) -> Result<String> {
    let token = context
        .state
        .spotify_token
        .lock()
        .expect("poisoned")
        .deref()
        .clone();

    Ok(match token {
        None => {
            let new_value = State::fetch_spotify_token(client, &context).await?;
            context
                .state
                .spotify_token
                .lock()
                .expect("poisoned")
                .replace(new_value.clone());
            new_value
        }
        Some(value) => value.clone(),
    })
}

fn clear_token(context: &Arc<Context>) {
    let _ = context
        .state
        .spotify_token
        .lock()
        .expect("poisoned")
        // take() meaning clear()
        .take();
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
    twitter_token: Mutex<Option<String>>,
    spotify_token: Mutex<Option<String>>,
}

async fn oauth_token(client: &Client, url: &str, key: &str, secret: &str) -> Result<String> {
    let token_body: Value = errors(
        client
            .post(url)
            .basic_auth(key, Some(secret))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials")
            .send()
            .await?,
    )?
    .json()
    .await
    .with_context(|| format_err!("bad json from auth: {:?}", url))?;

    Ok(format!(
        "Bearer {}",
        extract_token(&token_body)
            .with_context(|| format_err!("processing oauth response: {:?}", token_body))?
    ))
}

impl State {
    async fn update_twitter_token(&self, client: &Client, config: &Config) -> Result<()> {
        let new_value = oauth_token(
            client,
            "https://api.twitter.com/oauth2/token",
            &config.keys.twitter_app_key,
            &config.keys.twitter_app_secret,
        )
        .await?;
        self.twitter_token
            .lock()
            .expect("poisoned")
            .replace(new_value);

        Ok(())
    }

    async fn fetch_spotify_token(client: &Client, context: &Arc<Context>) -> Result<String> {
        let new_value = oauth_token(
            client,
            "https://accounts.spotify.com/api/token",
            &context.config.keys.spotify_app_key,
            &context.config.keys.spotify_app_secret,
        )
        .await?;

        Ok(new_value)
    }
}

fn epoch_secs() -> u64 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn extract_token(token_body: &Value) -> Result<String> {
    if !token_body
        .get("token_type")
        .ok_or_else(|| format_err!("no token_type"))?
        .as_str()
        .ok_or_else(|| format_err!("non-string token_type"))?
        .eq_ignore_ascii_case("bearer")
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
