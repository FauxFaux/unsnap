use failure::Error;
use iowrap::ReadMany;

use crate::webs::Webs;

pub async fn video<W: Webs>(webs: &W, id: &str) -> Result<String, Error> {
    let base = format!("https://v.redd.it/{}/", id);
    let html = crate::titles::html::process(webs, &base)
        .await.ok();

    let mut buf = [0u8; 32 * 1024];
    webs
        .raw_get(&format!("{}DASHPlaylist.mpd", base))?
        .read_many(&mut buf)
        .await?;
    let dash_playlist = String::from_utf8_lossy(&buf);
    Ok(match crate::content::dash::highest_stream(&dash_playlist) {
        Ok(mp4) => match html {
            Some(title) => format!("{}{} - {}", base, mp4, title),
            None => format!("Reddit 'dash' link without title: {}{}", base, mp4),
        },
        Err(e) => match html {
            Some(title) => format!("{} [video link failed: {} {}]", title, base, e),
            None => format!("Reddit 'dash' link mega-fail: {} [{}]", base, e),
        },
    })
}
