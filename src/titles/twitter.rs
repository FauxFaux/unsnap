use failure::format_err;
use failure::Error;

use crate::webs::twitter_get;
use crate::webs::Webs;

pub async fn tweet<W: Webs>(webs: &W, id: &str) -> Result<String, Error> {
    let resp = twitter_get(
        webs.client(),
        webs.config(),
        webs.state(),
        &format!("1.1/statuses/show.json?id={}&tweet_mode=extended", id),
    )
    .await?;

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
    use std::collections::HashMap;

    use failure::Error;
    use reqwest::Client;
    use serde_json;
    use serde_json::Value;

    use crate::webs::Explode;
    use crate::webs::Resp;
    use crate::webs::Webs;

    fn twitter_get_mock(client: &Client, sub: &str) -> Result<Value, Error> {
        Ok(match sub {
            "1.1/statuses/show.json?id=210462857140252672&tweet_mode=extended" => {
                serde_json::from_str(include_str!("../../tests/twitter-docsample.json")).unwrap()
            }
            "1.1/statuses/show.json?id=1015263010703544321&tweet_mode=extended" => {
                serde_json::from_str(include_str!("../../tests/twitter-multiline.json")).unwrap()
            }
            other => unimplemented!("test bug: {:?}", other),
        })
    }

    #[tokio::test]
    async fn doc_sample() {
        assert_eq!(
            "Twitter API — Along with our new #Twitterbird, we've also updated our Display Guidelines: https://t.co/Ed4omjYs ^JC",
            super::tweet(&mut Explode {}, "210462857140252672")
                .await.unwrap()
                .as_str()
        )
    }

    #[tokio::test]
    async fn multiline() {
        assert_eq!(
            "Joel the Forklift! — JIM MORRISON: people are strange, when you’re a stranger ¶ PRODUCER: nice ¶ JIM MORRISON: people are docks, when you’re a doctor ¶ PRODUCER: what ¶ JIM MORRISON: *wiggling fingers* people are ticks, when you’re a tickler ¶ PRODUCER (lips on mic): uh, I think we’re good Jim",
            super::tweet(&mut Explode {}, "1015263010703544321")
                .await.unwrap()
                .as_str()
        )
    }
}
