use errors::*;
use webs::Webs;

pub fn video<W: Webs>(webs: &W, id: &str) -> Result<String> {
    let resp = webs.youtube_get(
        "v3/videos",
        hashmap!(
        "id" => id,
        "part" => "snippet"
    ),
    )?;
    let data = resp.get("items")
        .ok_or("missing items")?
        .get(0)
        .ok_or("unexpectedly empty items")?;

    Ok(data.get("snippet")
        .and_then(|snippet| snippet.get("title"))
        .and_then(|title| title.as_str())
        .ok_or("no title")?
        .to_string())
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
                aiweechoo => serde_json::from_str(include_str!(
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
            "Platinum Level Circulation (Avicii x Tsukihi Araragi x Nadeko Sengoku)",
            super::video(&mut YoutubeTest {}, "JwhjqdSPw5g")
                .unwrap()
                .as_str()
        )
    }
}
