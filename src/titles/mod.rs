mod html;
mod imgur;

use regex::Regex;

use errors::*;
use webs::Webs;

lazy_static! {
    static ref URL: Regex = Regex::new("https?://[^ ]+").unwrap();
    static ref IMGUR_IMAGE: Regex =
        Regex::new(r"https?://i\.imgur\.com/([a-zA-Z0-9]{5,9})\.(?:jpg|mp4|webm|png|gif)").unwrap();
    static ref IMGUR_GALLERY: Regex =
        Regex::new(r"https?://(?:www\.)?imgur\.com/(?:a|gallery)/([a-zA-Z0-9]{5,7})").unwrap();
}

pub fn titles_for<W: Webs>(webs: &mut W, line: &str) -> Vec<Result<String>> {
    URL.find_iter(line)
        // FLIP? invert.
        .filter_map(|url| match title_for(webs, url.as_str()) {
            Ok(Some(x)) => Some(Ok(x)),
            Err(e) => Some(Err(e)),
            Ok(None) => None,
        })
        .collect()
}

pub fn title_for<W: Webs>(webs: &mut W, url: &str) -> Result<Option<String>> {
    if let Some(m) = IMGUR_IMAGE.captures(url) {
        let id = &m[1];
        return Ok(Some(imgur::image(webs, id)?));
    }

    if let Some(m) = IMGUR_GALLERY.captures(url) {
        let id = &m[1];
        return Ok(Some(imgur::gallery(webs, id)?));
    }

    Ok(None)
}
