use failure::Error;
use regex::Regex;

use crate::webs::Webs;

lazy_static! {
    static ref CHAINED_NEWLINES: Regex = Regex::new(r"¶(?:\s*¶)+").unwrap();
    static ref REPEATED_SPACE: Regex = Regex::new(r"\s{2,}").unwrap();
}

pub fn tweet<W: Webs>(webs: &W, id: &str) -> Result<String, Error> {
    let resp = webs.twitter_get(&format!(
        "1.1/statuses/show.json?id={}&tweet_mode=extended",
        id
    ))?;

    let text = cleanup_newlines(
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

fn cleanup_newlines(text: &str) -> String {
    let text = text.replace(|c: char| c.is_ascii_control(), " ¶ ");
    let text = CHAINED_NEWLINES.replace_all(&text, " ¶ ");
    REPEATED_SPACE.replace_all(&text, " ").to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use failure::Error;
    use reqwest::IntoUrl;
    use serde_json;
    use serde_json::Value;

    use crate::webs::Resp;
    use crate::webs::Webs;

    struct TwitterTest;

    impl Webs for TwitterTest {
        fn imgur_get(&self, _sub: &str) -> Result<Value, Error> {
            unimplemented!()
        }

        fn twitter_get(&self, sub: &str) -> Result<Value, Error> {
            Ok(match sub {
                "1.1/statuses/show.json?id=210462857140252672&tweet_mode=extended" => {
                    serde_json::from_str(include_str!("../../tests/twitter-docsample.json"))
                        .unwrap()
                }
                "1.1/statuses/show.json?id=1015263010703544321&tweet_mode=extended" => {
                    serde_json::from_str(include_str!("../../tests/twitter-multiline.json"))
                        .unwrap()
                }
                other => unimplemented!("test bug: {:?}", other),
            })
        }

        fn youtube_get(
            &self,
            _url_suffix: &str,
            _body: &HashMap<&str, &str>,
        ) -> Result<Value, Error> {
            unimplemented!()
        }

        fn raw_get<U: IntoUrl>(&self, _url: U) -> Result<Resp, Error> {
            unimplemented!()
        }
    }

    #[test]
    fn new_lines() {
        use super::cleanup_newlines;
        assert_eq!("foo ¶ bar", cleanup_newlines("foo\n \n   bar"));
    }

    #[test]
    fn doc_sample() {
        assert_eq!(
            "Twitter API — Along with our new #Twitterbird, we've also updated our Display Guidelines: https://t.co/Ed4omjYs ^JC",
            super::tweet(&mut TwitterTest {}, "210462857140252672")
                .unwrap()
                .as_str()
        )
    }

    #[test]
    fn multiline() {
        assert_eq!(
            "Joel the Forklift! — JIM MORRISON: people are strange, when you’re a stranger ¶ PRODUCER: nice ¶ JIM MORRISON: people are docks, when you’re a doctor ¶ PRODUCER: what ¶ JIM MORRISON: *wiggling fingers* people are ticks, when you’re a tickler ¶ PRODUCER (lips on mic): uh, I think we’re good Jim",
            super::tweet(&mut TwitterTest {}, "1015263010703544321")
                .unwrap()
                .as_str()
        )
    }
}
