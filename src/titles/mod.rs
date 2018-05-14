mod html;
mod imgur;
mod youtube;

use regex::Regex;
use result::ResultOptionExt;

use errors::*;
use webs::Webs;

lazy_static! {
    static ref URL: Regex = Regex::new("https?://[^ ]+").unwrap();
    static ref IMGUR_IMAGE: Regex =
        Regex::new(r"https?://i\.imgur\.com/([a-zA-Z0-9]{5,9})\.(?:jpg|mp4|webm|png|gif)").unwrap();
    static ref IMGUR_GALLERY: Regex =
        Regex::new(r"https?://(?:www\.)?imgur\.com/(?:a|gallery)/([a-zA-Z0-9]{5,7})").unwrap();
    static ref YOUTUBE_VIDEO: Regex = Regex::new(
        r"https?://(?:(?:(?:www\.)?youtube\.com/watch\?v=)|(?:youtu.be/))([a-zA-Z0-9_-]{11})"
    ).unwrap();
}

pub fn titles_for<W: Webs>(webs: &W, line: &str) -> Vec<Result<String>> {
    URL.find_iter(line)
        .filter_map(|url| title_for(webs, url.as_str()).invert())
        .collect()
}

pub fn title_for<W: Webs>(webs: &W, url: &str) -> Result<Option<String>> {
    if let Some(m) = IMGUR_IMAGE.captures(url) {
        let id = &m[1];
        return Ok(Some(imgur::image(webs, id)?));
    }

    if let Some(m) = IMGUR_GALLERY.captures(url) {
        let id = &m[1];
        return Ok(Some(imgur::gallery(webs, id)?));
    }

    if let Some(m) = YOUTUBE_VIDEO.captures(url) {
        let id = &m[1];
        return Ok(Some(youtube::video(webs, id)?));
    }

    Ok(html::process(webs, url).ok())
}
