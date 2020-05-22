use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use chrono::DateTime;
use maplit::hashmap;
use serde_json::Value;
use time_parse::duration;

use crate::webs::youtube_get;
use crate::webs::Webs;

pub async fn video<W: Webs>(webs: &W, id: &str) -> Result<String> {
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

    render_video(resp)
}

fn render_video(resp: Value) -> Result<String> {
    let data = resp
        .get("items")
        .ok_or(anyhow!("missing items"))?
        .get(0)
        .ok_or(anyhow!("unexpectedly empty items"))?;

    let snippet = data.get("snippet").ok_or(anyhow!("snippet missing"))?;

    let title = string(snippet.get("title"))?;
    let channel_title = string(snippet.get("channelTitle"))?;
    let published = DateTime::parse_from_rfc3339(string(snippet.get("publishedAt"))?)?;
    let duration = duration::parse(string(
        data.get("contentDetails")
            .ok_or(anyhow!("no content details"))?
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

fn string(value: Option<&Value>) -> Result<&str> {
    Ok(value
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("expected a string"))?)
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
    use serde_json;

    #[test]
    fn aiweechoo() {
        assert_eq!(
            "5m 2013-03-08 ፤ [shoopfex] ፤ Platinum Level Circulation (Avicii x Tsukihi Araragi x Nadeko Sengoku)",
            super::render_video(serde_json::from_str(include_str!("../../tests/youtube-aiweechoo.json")).unwrap())
                .unwrap()
                .as_str()
        )
    }
}
