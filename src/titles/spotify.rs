use anyhow::Result;

use crate::webs::spotify_get;
use crate::webs::Webs;

pub async fn anything<W: Webs>(webs: &W, kind: &str, id: &str) -> Result<String> {
    // lol
    let api_name = format!("{}s", kind);

    let res = spotify_get(
        webs.client(),
        webs.config(),
        webs.state(),
        &format!("{}/{}", api_name, id),
    )
    .await?;
    let v = serde_json::to_string(&res)?;
    Ok(v)
}
