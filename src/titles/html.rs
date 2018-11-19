use cast::f64;
use failure::Error;
use iowrap::ReadMany;
use twoway;

use super::strip_whitespace;
use crate::titles::show_size;
use crate::webs::Webs;

pub fn process<W: Webs>(webs: &W, url: &str) -> Result<String, Error> {
    let mut resp = webs.raw_get(url)?;
    const PREVIEW_BYTES: usize = 64 * 4096;

    let mut buf = [0u8; PREVIEW_BYTES];
    let found = resp.read_many(&mut buf)?;
    let buf = &buf[..found];

    match parse_html(buf) {
        Ok(ref title) if !strip_whitespace(title).is_empty() => return Ok(title.to_owned()),
        Ok(_empty) => (),
        Err(e) => info!("no title found for {:?}: {}", url, e),
    }

    let len = if buf.len() < PREVIEW_BYTES {
        Some(f64(buf.len()))
    } else if let Some(len) = resp.content_length() {
        Some(len)
    } else {
        None
    };

    let content_type = resp.content_type();

    let mut ret = "No title found.".to_string();
    if let Some(content_type) = content_type {
        ret.push_str(&format!(" Content-type: {}.", content_type));
    }

    if let Some(len) = len {
        ret.push_str(&format!(" Size: {}.", show_size(len)));
    }

    Ok(ret)
}

fn parse_html(buf: &[u8]) -> Result<String, &'static str> {
    // I'm not parsing HTML with regex.
    // It took me about four hours to write this code.
    // Not in coding time. In hating myself.

    let buf = &buf[find_string(buf, b"<title").ok_or("no title")?..];
    let buf = &buf[find_byte(buf, b'>').ok_or("no title tag terminator")?..];
    if buf.is_empty() {
        return Err("title starts at end of sub-document");
    }

    let buf = &buf[1..];
    let buf = &buf[..find_byte(buf, b'<').ok_or("no title terminator")?];
    let title = String::from_utf8_lossy(buf);
    Ok(match htmlescape::decode_html(&title) {
        Ok(decoded) => decoded,
        Err(e) => {
            info!("invalid html escape: {:?}: {:?}", title, e);
            title.to_string()
        }
    })
}

#[inline]
fn find_string(buf: &[u8], string: &[u8]) -> Option<usize> {
    if buf.len() < string.len() {
        return None;
    }

    for i in 0..=buf.len() - string.len() {
        if buf[i..][..string.len()].eq_ignore_ascii_case(string) {
            return Some(i);
        }
    }

    None
}

#[inline]
fn find_byte(buf: &[u8], byte: u8) -> Option<usize> {
    twoway::find_bytes(buf, &[byte])
}

#[cfg(test)]
mod tests {
    use super::parse_html;

    use super::find_string;

    #[test]
    fn finder() {
        assert_eq!(None, find_string(b"hello", b"cat"));
        assert_eq!(Some(1), find_string(b"hello", b"ello"));
        assert_eq!(Some(1), find_string(b"hello", b"e"));
        assert_eq!(Some(1), find_string(b"he", b"e"));
        assert_eq!(Some(1), find_string(b"hel", b"el"));
        assert_eq!(Some(4), find_string(b"hello", b"o"));
        assert_eq!(Some(0), find_string(b"hello", b"hello"));
        assert_eq!(Some(0), find_string(b"hello", b"h"));
        assert_eq!(Some(0), find_string(b"h", b"h"));
        assert_eq!(Some(0), find_string(b"HeLLo", b"heLlo"));
    }

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
        )
    }
}
