use failure::Error;
use serde_json::Value;

use webs::Webs;

pub fn tweet<W: Webs>(webs: &W, id: &str) -> Result<String, Error> {
    let resp = webs.twitter_get(&format!("1.1/statuses/show.json?id={}", id))?;

    let text = resp
        .get("text")
        .ok_or_else(|| format_err!("missing text"))?
        .as_str()
        .ok_or_else(|| format_err!("text not text"))?;

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
    use reqwest::IntoUrl;
    use serde_json;
    use serde_json::Value;

    use webs::Resp;
    use webs::Webs;

    struct TwitterTest;

    impl Webs for TwitterTest {
        fn imgur_get(&self, _sub: &str) -> Result<Value, Error> {
            unimplemented!()
        }

        fn twitter_get(&self, sub: &str) -> Result<Value, Error> {
            Ok(match sub {
                "1.1/statuses/show.json?id=210462857140252672" => serde_json::from_str(
                    include_str!("../../tests/twitter-docsample.json"),
                ).unwrap(),
                other => unimplemented!("test bug: {:?}", other),
            })
        }

        fn youtube_get(
            &self,
            url_suffix: &str,
            body: &HashMap<&str, &str>,
        ) -> Result<Value, Error> {
            unimplemented!()
        }

        fn raw_get<U: IntoUrl>(&self, _url: U) -> Result<Resp, Error> {
            unimplemented!()
        }
    }

    #[test]
    fn doc_sample() {
        assert_eq!(
            "Twitter API — Along with our new #Twitterbird, we've also updated our Display Guidelines: https://t.co/Ed4omjYs  ^JC",
            super::tweet(&mut TwitterTest {}, "210462857140252672")
                .unwrap()
                .as_str()
        )
    }
}
