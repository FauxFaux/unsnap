mod html;
mod imgur;
mod twitter;
mod youtube;

use failure::Error;
use regex::Regex;
use result::ResultOptionExt;

use crate::webs::Webs;

lazy_static! {
    static ref URL: Regex = Regex::new("https?://[^ ]+").unwrap();
    static ref IMGUR_IMAGE: Regex =
        Regex::new(r"https?://(?:i\.)?imgur\.com/([a-zA-Z0-9]{5,9})\.(?:jpg|mp4|webm|png|gif)")
            .unwrap();
    static ref IMGUR_GALLERY: Regex =
        Regex::new(r"https?://(?:www\.)?imgur\.com/(?:a|gallery)/([a-zA-Z0-9]{5,7})").unwrap();
    static ref TWITTER_TWEET: Regex =
        Regex::new(r"https?://(?:www\.)?twitter.com/(?:[^/]+)/status/(\d{16,25})").unwrap();
    static ref YOUTUBE_VIDEO: Regex = Regex::new(
        r"https?://(?:(?:(?:www\.)?youtube\.com/watch\?v=)|(?:youtu.be/))([a-zA-Z0-9_-]{11})"
    )
    .unwrap();
}

pub fn titles_for<W: Webs>(webs: &W, line: &str) -> Vec<Result<String, Error>> {
    URL.find_iter(line)
        .filter_map(|url| title_for(webs, url.as_str()).invert())
        .collect()
}

pub fn title_for<W: Webs>(webs: &W, url: &str) -> Result<Option<String>, Error> {
    if let Some(m) = IMGUR_IMAGE.captures(url) {
        let id = &m[1];
        return Ok(Some(imgur::image(webs, id)?));
    }

    if let Some(m) = IMGUR_GALLERY.captures(url) {
        let id = &m[1];
        return Ok(Some(imgur::gallery(webs, id)?));
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
        .map_err(|e| {
            info!("gave up processing url {:?}: {:?}", url, e);
            return e;
        })
        .ok())
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
}
