use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

use super::youtube::major_duration_unit;
use crate::webs::spotify_get;
use crate::webs::Context;

lazy_static! {
    static ref URL_STRIPPER: Regex = Regex::new("cid=[a-f0-9]{30,34}").unwrap();
}

pub async fn anything(http: Client, context: Arc<Context>, kind: &str, id: &str) -> Result<String> {
    // lol
    let api_name = format!("{}s", kind);

    let res = spotify_get(&http, context, &format!("{}/{}", api_name, id)).await?;
    Ok(match kind {
        "track" => render_track(&serde_json::from_value(res)?),
        _ => serde_json::to_string(&res)?,
    })
}

fn render_track(track: &SpotifyTrack) -> String {
    let mut msg = format!(
        "{} {} {} - {}",
        major_duration_unit(&Duration::from_millis(track.duration_ms)),
        track.album.release_date,
        artist_names(&track.artists),
        track.name
    );

    if let Some(preview_url) = &track.preview_url {
        msg.push_str(" - ");
        msg.push_str(&strip_url(preview_url));
    }

    msg
}

fn artist_names(artists: &[SpotifyArtist]) -> String {
    artists.iter().map(|a| &a.name).join(", ")
}

fn strip_url(url: &str) -> String {
    let ret = URL_STRIPPER.replace(url, "").to_string();
    assert!(!ret.contains("cid="), "stripping failed");
    ret
}

#[derive(Deserialize)]
struct SpotifyTrack {
    album: SpotifyAlbum,
    artists: Vec<SpotifyArtist>,
    duration_ms: u64,
    name: String,
    preview_url: Option<String>,

    popularity: f64,

    disc_number: u64,
    track_number: u64,
}

#[derive(Deserialize)]
struct SpotifyAlbum {
    artists: Vec<SpotifyArtist>,
    name: String,
    // 2020-05-22
    release_date: String,
    total_tracks: u64,
}

#[derive(Deserialize)]
struct SpotifyArtist {
    name: String,
}

#[cfg(test)]
mod test {
    use super::render_track;
    use super::SpotifyTrack;

    #[test]
    fn json() {
        let track: SpotifyTrack =
            serde_json::from_str(include_str!("../../tests/spotify-track.json")).unwrap();
        assert_eq!(
            concat!(
                "3m 2020-05-22 Major Lazer, Diplo, Marcus Mumford, Lost Frequencies",
                " - Lay Your Head On Me (feat. Marcus Mumford) - Lost Frequencies Remix",
                " - https://p.scdn.co/mp3-preview/92b4abe7d7e721dffdcf7744ee4d062ede55a648?"
            ),
            render_track(&track)
        );
    }

    #[test]
    fn no_preview() {
        let track: SpotifyTrack =
            serde_json::from_str(include_str!("../../tests/spotify-null.json")).unwrap();
        assert_eq!(
            "12m 2009-04-20 The Juan Maclean - Happy House",
            render_track(&track)
        );
    }
}
