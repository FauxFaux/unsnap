use failure::Error;
use iowrap::ReadMany;
use twoway;

use crate::webs::Webs;

pub fn process<W: Webs>(webs: &W, url: &str) -> Result<String, Error> {
    let mut resp = webs.raw_get(url)?;

    let mut buf = [0u8; 16 * 4096];
    let found = resp.read_many(&mut buf)?;
    let buf = &buf[..found];

    parse_html(buf)
}

fn parse_html(buf: &[u8]) -> Result<String, Error> {
    // I'm not parsing HTML with regex.
    // It took me about four hours to write this code.
    // Not in coding time. In hating myself.

    let buf = &buf[twoway::find_bytes(buf, b"<title").ok_or(format_err!("no title"))?..];
    let buf = &buf[find_byte(buf, b'>').ok_or(format_err!("no title tag terminator"))?..];
    ensure!(!buf.is_empty(), "title starts at end of sub-document");
    let buf = &buf[1..];
    let buf = &buf[..find_byte(buf, b'<').ok_or(format_err!("no title terminator"))?];

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
