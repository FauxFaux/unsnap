use anyhow::Result;
use anyhow::format_err;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

use crate::webs::Context;
use crate::webs::twitter_get;

pub async fn tweet(http: Client, context: Arc<Context>, id: &str) -> Result<String> {
    let resp = twitter_get(
        &http,
        context,
        &format!("1.1/statuses/show.json?id={}&tweet_mode=extended", id),
    )
    .await?;

    render_tweet(resp)
}

fn render_tweet(resp: Value) -> Result<String> {
    let text = super::cleanup_newlines(
        resp.get("full_text")
            .ok_or_else(|| format_err!("missing text"))?
            .as_str()
            .ok_or_else(|| format_err!("text not text"))?,
    );

    let user = resp
        .get("user")
        .ok_or_else(|| format_err!("no user"))?
        .as_object()
        .ok_or_else(|| format_err!("user not object"))?;

    let name = user
        .get("name")
        .ok_or_else(|| format_err!("user lacks name"))?
        .as_str()
        .ok_or_else(|| format_err!("user name not text"))?;

    Ok(format!("{} — {}", name, text))
}

#[cfg(test)]
mod tests {
    use serde_json;

    #[test]
    fn doc_sample() {
        assert_eq!(
            "Twitter API — Along with our new #Twitterbird, we've also updated our Display Guidelines: https://t.co/Ed4omjYs ^JC",
            super::render_tweet(
                serde_json::from_str(include_str!("../../tests/twitter-docsample.json")).unwrap()
            )
            .unwrap()
            .as_str()
        )
    }

    #[test]
    fn multiline() {
        assert_eq!(
            "Joel the Forklift! — JIM MORRISON: people are strange, when you’re a stranger ¶ PRODUCER: nice ¶ JIM MORRISON: people are docks, when you’re a doctor ¶ PRODUCER: what ¶ JIM MORRISON: *wiggling fingers* people are ticks, when you’re a tickler ¶ PRODUCER (lips on mic): uh, I think we’re good Jim",
            super::render_tweet(
                serde_json::from_str(include_str!("../../tests/twitter-multiline.json")).unwrap()
            )
            .unwrap()
            .as_str()
        )
    }
}
