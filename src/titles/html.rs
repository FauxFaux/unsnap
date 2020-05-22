use anyhow::Result;
use cast::f64;
use regex::bytes;

use super::strip_whitespace;
use crate::titles::show_size;
use crate::webs::content_length;
use crate::webs::content_type;
use crate::webs::read_many;
use crate::webs::Webs;

lazy_static::lazy_static! {
    static ref TITLE: bytes::Regex = bytes::Regex::new(r"(?i)<title[^>]*>([^<]*)<").unwrap();
}

pub async fn process<W: Webs>(webs: &W, url: &str) -> Result<String> {
    let mut resp = webs.client().get(url).send().await?;
    const PREVIEW_BYTES: usize = 64 * 4096;

    let content_length = content_length(&resp);
    let content_type = content_type(&resp).map(String::from);

    let mut buf = [0u8; PREVIEW_BYTES];
    let found = read_many(&mut resp, &mut buf).await?;
    let buf = &buf[..found];

    let missing = match parse_html(buf) {
        Ok(ref title) if !strip_whitespace(title).is_empty() => return Ok(title.to_owned()),
        Ok(_empty) => false,
        Err(e) => {
            info!("no title found for {:?}: {}", url, e);
            true
        }
    };

    let len = if buf.len() < PREVIEW_BYTES {
        Some(f64(buf.len()))
    } else if let Some(len) = content_length {
        Some(len)
    } else {
        None
    };

    let ret = if missing {
        "No title found."
    } else {
        "Empty title found."
    };

    let mut ret = ret.to_string();

    if let Some(content_type) = content_type {
        ret.push_str(&format!(" Content-type: {}.", content_type));
    }

    if let Some(len) = len {
        ret.push_str(&format!(" Size: {}.", show_size(len)));
    }

    Ok(ret)
}

fn parse_html(buf: &[u8]) -> Result<String, &'static str> {
    let title = match TITLE.captures_iter(buf).next() {
        Some(cap) => String::from_utf8_lossy(&cap[1]).to_string(),
        None => return Err("no regex match"),
    };

    Ok(match htmlescape::decode_html(&title) {
        Ok(decoded) => decoded,
        Err(e) => {
            info!("invalid html escape: {:?}: {:?}", title, e);
            title.to_string()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::parse_html;

    #[test]
    fn html() {
        assert_eq!(
            "ponies",
            parse_html(b"<html><head><title>ponies</title></head><body></body></html>")
                .unwrap()
                .as_str()
        );

        assert_eq!(
            "ponies",
            parse_html(b"<html><head><TITle>ponies</TITLE></head><body></body></html>")
                .unwrap()
                .as_str()
        );

        assert_eq!(
            "'",
            parse_html(b"<html><head><title>&#x27;</title></head><body></body></html>")
                .unwrap()
                .as_str()
        );

        assert_eq!(
            "Commonwealth meeting: Queen hopes Prince Charles will succeed her - BBC News",
            parse_html(include_bytes!("../../tests/bbc.html"))
                .unwrap()
                .as_str()
        );

        assert_eq!(
            "Look Out for New ‘Find Your Place’ Ads This Summer | StreetEasy",
            parse_html(include_bytes!("../../tests/streeteasy.html"))
                .unwrap()
                .as_str()
        );
    }
}
