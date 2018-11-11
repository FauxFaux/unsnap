use cast::f64;
use failure::Error;
use iowrap::ReadMany;
use twoway;

use crate::titles::show_size;
use crate::webs::Webs;

pub fn process<W: Webs>(webs: &W, url: &str) -> Result<String, Error> {
    let mut resp = webs.raw_get(url)?;
    const PREVIEW_BYTES: usize = 16 * 4096;

    let mut buf = [0u8; PREVIEW_BYTES];
    let found = resp.read_many(&mut buf)?;
    let buf = &buf[..found];

    match parse_html(buf) {
        Ok(title) => Ok(title),
        Err(e) => {
            if buf.len() < PREVIEW_BYTES {
                info!("no title found for {:?} ({}), but we read it all", url, e);
                Ok(no_title_size(f64(buf.len())))
            } else if let Some(len) = resp.content_length() {
                info!("no title found for {:?} ({}), but it had a length", url, e);
                Ok(no_title_size(len))
            } else {
                info!("no title found for {:?} ({}), and no length guess", url, e);
                Err(format_err!("no title and overly long: {}", e))
            }
        }
    }
}

fn no_title_size(size: f64) -> String {
    format!("No title found. Size: {}", show_size(size))
}

fn parse_html(buf: &[u8]) -> Result<String, &'static str> {
    // I'm not parsing HTML with regex.
    // It took me about four hours to write this code.
    // Not in coding time. In hating myself.

    let buf = &buf[twoway::find_bytes(buf, b"<title").ok_or("no title")?..];
    let buf = &buf[find_byte(buf, b'>').ok_or("no title tag terminator")?..];
    if buf.is_empty() {
        return Err("title starts at end of sub-document");
    }

    let buf = &buf[1..];
    let buf = &buf[..find_byte(buf, b'<').ok_or("no title terminator")?];

    Ok(String::from_utf8_lossy(buf).to_string())
}

#[inline]
fn find_byte(buf: &[u8], byte: u8) -> Option<usize> {
    twoway::find_bytes(buf, &[byte])
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
            "Commonwealth meeting: Queen hopes Prince Charles will succeed her - BBC News",
            parse_html(include_bytes!("../../tests/bbc.html"))
                .unwrap()
                .as_str()
        )
    }
}
