use failure::Error;
use iowrap::ReadMany;

use crate::webs::Webs;

pub fn video<W: Webs>(webs: &W, id: &str) -> Result<String, Error> {
    let base = format!("https://v.redd.it/{}/", id);
    let mut resp = webs.raw_get(&format!("{}DASHPlaylist.mpd", base))?;
    let mut buf = [0u8; 32 * 1024];
    resp.read_many(&mut buf)?;
    let dash_playlist = String::from_utf8_lossy(&buf);
    Ok(match crate::content::dash::highest_stream(&dash_playlist) {
        Ok(mp4) => format!("Reddit 'dash' video: {}{}", base, mp4),
        Err(e) => format!("Reddit video link: {} failed with {}", base, e),
    })
}
