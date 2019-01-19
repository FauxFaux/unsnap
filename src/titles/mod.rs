mod html;
mod imgur;
mod reddit;
mod twitter;
mod youtube;

use failure::Error;
use regex::Regex;
use result::ResultOptionExt;
use url::Url;

use crate::webs::Webs;

lazy_static! {
    static ref URL: Regex = Regex::new("https?://[^ ]+").unwrap();
    static ref IMGUR_IMAGE: Regex =
        Regex::new(r"https?://(?:i\.)?imgur\.com/([a-zA-Z0-9]{5,9})\.(?:jpg|mp4|webm|png|gif)")
            .unwrap();
    static ref IMGUR_GALLERY: Regex =
        Regex::new(r"https?://(?:www\.)?imgur\.com/(?:a|gallery)/([a-zA-Z0-9]{5,7})").unwrap();
    static ref REDDIT_VIDEO: Regex = Regex::new(r"https?://v.redd.it/(\w+)/").unwrap();
    static ref TWITTER_TWEET: Regex =
        Regex::new(r"https?://(?:www\.)?twitter.com/(?:[^/]+)/status/(\d{16,25})").unwrap();
    static ref YOUTUBE_VIDEO: Regex = Regex::new(
        r"https?://(?:(?:(?:www\.)?youtube\.com/watch\?v=)|(?:youtu.be/))([a-zA-Z0-9_-]{11})"
    )
    .unwrap();
    static ref CHAINED_NEWLINES: Regex = Regex::new(r"¶(?:\s*¶)+").unwrap();
    static ref REPEATED_SPACE: Regex = Regex::new(r"\s{2,}").unwrap();
}

pub fn titles_for<W: Webs>(webs: &W, line: &str) -> Vec<Result<String, Error>> {
    URL.find_iter(line)
        .filter_map(|url| {
            title_for(webs, url.as_str())
                .map(|maybe| {
                    maybe.map(|title| {
                        format!(
                            "[ {} - {} ]",
                            hostname(url.as_str()),
                            strip_whitespace(&title)
                        )
                    })
                })
                .invert()
        })
        .collect()
}

fn title_for<W: Webs>(webs: &W, url: &str) -> Result<Option<String>, Error> {
    if let Some(m) = IMGUR_IMAGE.captures(url) {
        let id = &m[1];
        return Ok(Some(imgur::image(webs, id)?));
    }

    if let Some(m) = IMGUR_GALLERY.captures(url) {
        let id = &m[1];
        return Ok(Some(imgur::gallery(webs, id)?));
    }

    if let Some(m) = REDDIT_VIDEO.captures(url) {
        let id = &m[1];
        return Ok(Some(reddit::video(webs, id)?));
    }

    if let Some(m) = TWITTER_TWEET.captures(url) {
        let id = &m[1];
        return Ok(Some(twitter::tweet(webs, id)?));
    }

    if let Some(m) = YOUTUBE_VIDEO.captures(url) {
        let id = &m[1];
        return Ok(Some(youtube::video(webs, id)?));
    }

    Ok(html::process(webs, url)
        .map(|s| strip_whitespace(&s))
        .map_err(|e| {
            info!("gave up processing url {:?}: {:?}", url, e);
            return e;
        })
        .ok())
}

fn hostname(url: &str) -> String {
    url.parse::<Url>()
        .ok()
        .and_then(|url| url.host_str().map(|host| host.to_owned()))
        .unwrap_or_else(|| "[invalid url]".to_string())
}

fn show_size(val: f64) -> String {
    use number_prefix::binary_prefix;
    use number_prefix::Prefixed;
    use number_prefix::Standalone;

    match binary_prefix(val) {
        Standalone(bytes) => format!("{} bytes", bytes),
        Prefixed(prefix, n) => format!("{:.1}{}B", n, prefix),
    }
}

fn strip_whitespace(text: &str) -> String {
    let text = text.replace(|c: char| c.is_control() || c.is_whitespace(), " ");
    let text = text.trim();
    REPEATED_SPACE.replace_all(text, " ").to_string()
}

fn cleanup_newlines(text: &str) -> String {
    let text = text.trim();
    let text = text.replace(|c: char| c.is_control(), " ¶ ");
    let text = CHAINED_NEWLINES.replace_all(&text, " ¶ ");
    REPEATED_SPACE.replace_all(&text, " ").to_string()
}

#[cfg(test)]
mod tests {
    use super::IMGUR_IMAGE;

    #[test]
    fn imgur_image() {
        assert_eq!(
            1,
            IMGUR_IMAGE
                .captures("yellow https://imgur.com/ZbIiLa9.mp4 snow")
                .unwrap()
                .len()
                // includes group 0
                - 1
        )
    }

    #[test]
    fn hostname_extraction() {
        use super::hostname;
        assert_eq!("imgur.com", hostname("https://imgur.com/a/foo"));
        assert_eq!("[invalid url]", hostname("/a/foo"));
        assert_eq!("127.0.0.1", hostname("http://127.0.0.1/a/foo"));
        assert_eq!("xn--fent-ipa.re", hostname("https://fenêt.re/"));
    }

    #[test]
    fn new_lines() {
        use super::cleanup_newlines;
        assert_eq!("foo ¶ bar", cleanup_newlines("foo\n \n   bar"));
    }

    #[test]
    fn strip() {
        use super::strip_whitespace;
        assert_eq!("foo bar", strip_whitespace("  \n  foo  bar    \n"));
        assert_eq!("foo bar", strip_whitespace("foo \0 \x06 \u{009f} bar"));
    }
}
