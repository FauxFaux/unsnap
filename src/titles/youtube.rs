use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use serde_json::Value;
use time_parse::duration;

use errors::*;
use webs::Webs;

pub fn video<W: Webs>(webs: &W, id: &str) -> Result<String> {
    let resp = webs.youtube_get(
        "v3/videos",
        hashmap!(
            "id" => id,
            "part" => "snippet,contentDetails"
        ),
    )?;

    println!("{:?}", resp);

    let data = resp.get("items")
        .ok_or("missing items")?
        .get(0)
        .ok_or("unexpectedly empty items")?;

    let snippet = data.get("snippet").ok_or("snippet missing")?;

    let title = string(snippet.get("title"))?;
    let channel_title = string(snippet.get("channelTitle"))?;
    let published = DateTime::parse_from_rfc3339(string(snippet.get("publishedAt"))?)?;
    let duration = duration::parse(string(
        data.get("contentDetails")
            .ok_or("no content details")?
            .get("duration"),
    )?).map_err(|e| format!("{:?}", e))?;

    Ok(format!(
        "{} {} ፤ [{}] ፤ {}",
        major_duration_unit(&duration),
        published.date().naive_local(),
        channel_title,
        title
    ))
}

fn string(value: Option<&Value>) -> Result<&str> {
    Ok(value.and_then(|v| v.as_str()).ok_or("expected a string")?)
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

    use reqwest::IntoUrl;
    use serde_json;
    use serde_json::Value;

    use errors::*;
    use webs::Resp;
    use webs::Webs;

    struct YoutubeTest;

    impl Webs for YoutubeTest {
        fn imgur_get(&self, sub: &str) -> Result<Value> {
            unimplemented!()
        }

        fn youtube_get(&self, url_suffix: &str, body: HashMap<&str, &str>) -> Result<Value> {
            assert_eq!("v3/videos", url_suffix);
            let aiweechoo = hashmap! { "key" => "JwhjqdSPw5g", "content" => "snippet" };
            Ok(match body {
                ref val if *val == aiweechoo => serde_json::from_str(include_str!(
                    "../../tests/youtube-aiweechoo.json"
                )).unwrap(),
                body => unimplemented!("test bug: {:?}", body),
            })
        }

        fn raw_get<U: IntoUrl>(&self, url: U) -> Result<Resp> {
            unimplemented!()
        }
    }

    #[test]
    fn aiweechoo() {
        assert_eq!(
            "5m 2013-03-08 ፤ [shoopfex] ፤ Platinum Level Circulation (Avicii x Tsukihi Araragi x Nadeko Sengoku)",
            super::video(&mut YoutubeTest {}, "JwhjqdSPw5g")
                .unwrap()
                .as_str()
        )
    }
}
