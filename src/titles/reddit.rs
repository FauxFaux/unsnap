use anyhow::Result;

use crate::webs::read_many;
use crate::webs::Webs;

pub async fn video<W: Webs>(webs: &W, id: &str) -> Result<String> {
    let base = format!("https://v.redd.it/{}/", id);
    let html = crate::titles::html::process(webs, &base).await.ok();

    let mut buf = [0u8; 32 * 1024];
    let mut resp = webs
        .client()
        .get(&format!("{}DASHPlaylist.mpd", base))
        .send()
        .await?;
    read_many(&mut resp, &mut buf).await?;
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
