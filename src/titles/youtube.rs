use std::time::Duration;

use chrono::DateTime;
use failure::format_err;
use failure::Error;
use maplit::hashmap;
use serde_json::Value;
use time_parse::duration;

use crate::webs::youtube_get;
use crate::webs::Webs;

pub async fn video<W: Webs>(webs: &W, id: &str) -> Result<String, Error> {
    let resp = youtube_get(
        webs.client(),
        webs.config(),
        "v3/videos",
        &hashmap!(
            "id" => id,
            "part" => "snippet,contentDetails"
        ),
    )
    .await?;

    let data = resp
        .get("items")
        .ok_or(format_err!("missing items"))?
        .get(0)
        .ok_or(format_err!("unexpectedly empty items"))?;

    let snippet = data.get("snippet").ok_or(format_err!("snippet missing"))?;

    let title = string(snippet.get("title"))?;
    let channel_title = string(snippet.get("channelTitle"))?;
    let published = DateTime::parse_from_rfc3339(string(snippet.get("publishedAt"))?)?;
    let duration = duration::parse(string(
        data.get("contentDetails")
            .ok_or(format_err!("no content details"))?
            .get("duration"),
    )?)?;

    Ok(format!(
        "{} {} ፤ [{}] ፤ {}",
        major_duration_unit(&duration),
        published.date().naive_local(),
        channel_title,
        title
    ))
}

fn string(value: Option<&Value>) -> Result<&str, Error> {
    Ok(value
        .and_then(|v| v.as_str())
        .ok_or(format_err!("expected a string"))?)
}

fn major_duration_unit(duration: &Duration) -> String {
    let mut d = duration.as_secs();
    for (div, name) in &[(60, 's'), (60, 'm')] {
        if d < *div {
            return format!("{}{}", d, name);
        }

        d /= *div;
    }

    format!("{}h", d)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use failure::Error;
    use maplit::hashmap;
    use reqwest::Client;
    use serde_json;
    use serde_json::Value;

    use crate::webs::Explode;
    use crate::webs::Webs;

    fn youtube_get_mock(
        client: &Client,
        url_suffix: &str,
        body: &HashMap<&str, &str>,
    ) -> Result<Value, Error> {
        assert_eq!("v3/videos", url_suffix);
        let aiweechoo = hashmap! { "id" => "JwhjqdSPw5g", "part" => "snippet,contentDetails" };
        Ok(match body {
            val if *val == aiweechoo => {
                serde_json::from_str(include_str!("../../tests/youtube-aiweechoo.json")).unwrap()
            }
            body => unimplemented!("test bug: {:?}", body),
        })
    }

    #[tokio::test]
    async fn aiweechoo() {
        assert_eq!(
            "5m 2013-03-08 ፤ [shoopfex] ፤ Platinum Level Circulation (Avicii x Tsukihi Araragi x Nadeko Sengoku)",
            super::video(&mut Explode {}, "JwhjqdSPw5g")
                .await.unwrap()
                .as_str()
        )
    }
}
