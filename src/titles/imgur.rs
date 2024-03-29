use anyhow::format_err;
use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

use crate::titles::show_size;
use crate::webs::imgur_get;
use crate::webs::Context;

pub async fn image(http: Client, context: Arc<Context>, id: &str) -> Result<String> {
    let resp = imgur_get(&http, &context.config, &format!("image/{}", id)).await?;
    render_image(resp)
}

fn render_image(resp: Value) -> Result<String> {
    let data = resp.get("data").ok_or(format_err!("missing data"))?;

    image_body(data, None)
}

fn image_body(data: &Value, title_hint: Option<&str>) -> Result<String> {
    let mut title = format!(
        "{}×{}",
        data.get("width").ok_or(format_err!("missing width"))?,
        data.get("height").ok_or(format_err!("missing height"))?
    );

    if let Some(size) = preferred_size(data) {
        title.push(' ');
        title.push_str(&show_size(size));
    }

    if let Some(section) = data.get("section").and_then(|s| s.as_str()) {
        title.push(' ');
        title.push_str("/r/");
        title.push_str(section);
    }

    push_sfw(&mut title, data);

    if let Some(post_title) = data.get("title").and_then(|s| s.as_str()) {
        title.push_str(" ፤ ");
        title.push_str(post_title)
    } else if let Some(post_title) = title_hint {
        title.push_str(" ፤ ");
        title.push_str(post_title)
    } else if let Some(desc) = data.get("description").and_then(|s| s.as_str()) {
        title.push_str(" ፤ ");
        title.push_str(desc);
    }

    Ok(title)
}

fn push_sfw(title: &mut String, data: &Value) {
    title.push(' ');
    title.push_str(match data.get("nsfw") {
        Some(v) => match v.as_bool() {
            Some(true) => "NSFW",
            Some(false) => "sfw",
            None => "?fw",
        },
        None => "¿fw",
    });
}

pub async fn gallery(http: Client, context: Arc<Context>, id: &str) -> Result<String> {
    let resp = imgur_get(&http, &context.config, &format!("album/{}", id)).await?;
    render_gallery(resp)
}

fn render_gallery(resp: Value) -> Result<String> {
    let data = resp.get("data").ok_or(format_err!("missing data"))?;

    let gallery_title = data
        .get("title")
        .and_then(|t| t.as_str())
        .ok_or(format_err!("missing title"))?;

    let count = data
        .get("images_count")
        .and_then(|c| c.as_i64())
        .ok_or(format_err!("no image count"))?;

    let images = data
        .get("images")
        .and_then(|v| v.as_array())
        .ok_or(format_err!("no images"))?;

    if 1 == count && !images.is_empty() {
        let image = &images[0];
        return Ok(format!(
            "{} ፤ {}",
            preferred_link(image)?,
            image_body(image, Some(gallery_title))?
        ));
    }

    let mut animated = 0;
    let mut size = 0.;

    for image in images {
        size += preferred_size(image).ok_or(format_err!("album image is missing size"))?;
        if let Some(true) = image.get("animated").and_then(|b| b.as_bool()) {
            animated += 1;
        }
    }

    let mut title = format!("{}/{} animated ፤ ", animated, images.len());
    title.push_str(&show_size(size));

    push_sfw(&mut title, data);

    if let Some(post_title) = data.get("title").and_then(|s| s.as_str()) {
        title.push_str(" ፤ ");
        title.push_str(post_title)
    }

    Ok(title)
}

fn preferred_link(image: &Value) -> Result<&str> {
    Ok(image
        .get("mp4")
        .or_else(|| image.get("link"))
        .and_then(|s| s.as_str())
        .ok_or(format_err!("no link on embedded image"))?)
}

fn preferred_size(data: &Value) -> Option<f64> {
    for key in &["mp4_size", "webm_size", "size"] {
        if let Some(size) = try_f64(data, key) {
            return Some(size);
        }
    }

    None
}

fn try_f64(data: &Value, key: &str) -> Option<f64> {
    data.get(key).and_then(|s| s.as_f64())
}

#[cfg(test)]
mod tests {
    use serde_json;
    use serde_json::Value;

    const STRAIGHT_IMAGE: &str = r#"
        {"data":{"id":"TUgcjTQ","title":null,"description":null,"datetime":1517869892,
        "type":"image\/jpeg","animated":false,"width":470,"height":334,"size":12828,
        "views":443,"bandwidth":5682804,"vote":null,"favorite":false,"nsfw":false,
        "section":null,"account_url":null,"account_id":null,"is_ad":false,
        "in_most_viral":false,"has_sound":false,"tags":[],"ad_type":0,"ad_url":"",
        "in_gallery":false,"link":"https:\/\/i.imgur.com\/TUgcjTQ.jpg"},
        "success":true,"status":200}"#;

    const IMAGE_WITH_SECTION: &str = r#"
        {"data":{"id":"zEG4ULo","title":null,"description":null,"datetime":1523881468,
        "type":"image\/jpeg","animated":false,"width":640,"height":799,"size":99578,
        "views":1029990,"bandwidth":102564344220,"vote":null,"favorite":false,"nsfw":false,
        "section":"pics","account_url":null,"account_id":null,"is_ad":false,
        "in_most_viral":false,"has_sound":false,"tags":[],"ad_type":0,"ad_url":"",
        "in_gallery":false,"link":"https:\/\/i.imgur.com\/zEG4ULo.jpg"},
        "success":true,"status":200}
    "#;

    const IMAGE_WITH_TITLE: &str = r#"
        {"data":{"id":"PmSOx4H",
        "title":"My army is ready, we attack at nightfall",
        "description":null,"datetime":1523954060,
        "type":"image\/jpeg","animated":false,"width":720,"height":540,"size":33292,
        "views":65040,"bandwidth":2165311680,"vote":null,"favorite":false,"nsfw":false,
        "section":"pics","account_url":null,"account_id":null,"is_ad":false,
        "in_most_viral":false,"has_sound":false,"tags":[],"ad_type":0,"ad_url":"",
        "in_gallery":true,"link":"https:\/\/i.imgur.com\/PmSOx4H.jpg"},
        "success":true,"status":200}
    "#;

    const VIDEO_WITH_DESCRIPTION: &str = r##"
        {"data":{"id":"SRup0KZ",
        "title":null,"description":"#dolphinsandshit","datetime":1523933676,
        "type":"image\/gif","animated":true,"width":667,"height":500,"size":68422159,
        "views":920699,"bandwidth":62996213369141,"vote":null,"favorite":false,"nsfw":false,
        "section":"awesomenature","account_url":null,"account_id":null,"is_ad":false,
        "in_most_viral":false,"has_sound":false,"tags":[],"ad_type":0,"ad_url":"",
        "in_gallery":false,"link":"http:\/\/i.imgur.com\/SRup0KZh.gif",
        "mp4":"https:\/\/i.imgur.com\/SRup0KZ.mp4","gifv":"https:\/\/i.imgur.com\/SRup0KZ.gifv",
        "mp4_size":8642558,"looping":true},
        "success":true,"status":200}
    "##;

    const SINGLE_IMAGE_ALBUM: &str = r##"
        {"data":{"id":"rTV6u","title":"Branch manager and Assistant Branch manager",
        "description":null,"datetime":1523747874,
        "cover":"tUulJaV","cover_width":640,"cover_height":770,
        "account_url":null,"account_id":null,"privacy":"hidden","layout":"blog",
        "views":141192,"link":"https:\/\/imgur.com\/a\/rTV6u","favorite":false,"nsfw":null,
        "section":null,"images_count":1,"in_gallery":true,"is_ad":false,
        "images":[{
            "id":"tUulJaV","title":null,"description":null,"datetime":1523747876,
            "type":"image\/jpeg","animated":false,"width":640,"height":770,"size":89545,
            "views":113562,"bandwidth":10168909290,"vote":null,"favorite":false,"nsfw":null,
            "section":null,"account_url":null,"account_id":null,"is_ad":false,
            "in_most_viral":false,"has_sound":false,"tags":[],"ad_type":0,"ad_url":"",
            "in_gallery":false,"link":"https:\/\/i.imgur.com\/tUulJaV.jpg"}]},
        "success":true,"status":200}
    "##;

    const SINGLE_IMAGE_ANIMATED: &str = r##"
        {"data":{"id":"nRuZwtp","title":"Bango cat","description":null,"datetime":1543508604,
        "cover":"KbVOQOm","cover_width":580,"cover_height":580,"account_url":"ctilic12110",
        "account_id":98180053,"privacy":"public","layout":"blog","views":599,
        "link":"https:\/\/imgur.com\/a\/nRuZwtp","favorite":false,"nsfw":null,"section":null,
        "images_count":1,"in_gallery":true,"is_ad":false,"include_album_ads":false,
        "images":[{"id":"KbVOQOm","title":null,"description":null,"datetime":1543508602,
        "type":"image\/gif","animated":true,"width":580,"height":580,"size":1812159,"views":1820,
        "bandwidth":3298129380,"vote":null,"favorite":false,"nsfw":null,"section":null,
        "account_url":null,"account_id":null,"is_ad":false,"in_most_viral":false,"has_sound":false,
        "tags":[],"ad_type":0,"ad_url":"","in_gallery":false,
        "link":"https:\/\/i.imgur.com\/KbVOQOm.gif","mp4":"https:\/\/i.imgur.com\/KbVOQOm.mp4",
        "gifv":"https:\/\/i.imgur.com\/KbVOQOm.gifv","hls":"https:\/\/i.imgur.com\/KbVOQOm.m3u8",
        "mp4_size":1304498,"looping":true,"processing":{"status":"completed"}}]},"success":true,
        "status":200}
    "##;

    const MULTI_IMAGE_ALBUM: &str = r##"
        {"data":{"id":"mk0v7",
        "title":"Transformation Tuesday: went from 6xl to 3xl... still got ways to go. Thanks imgur",
        "description":null,"datetime":1524022042,
        "cover":"EtJ3EyI","cover_width":2048,"cover_height":2048,
        "account_url":null,"account_id":null,"privacy":"hidden","layout":"blog",
        "views":110124,"link":"https:\/\/imgur.com\/a\/mk0v7","favorite":false,"nsfw":null,
        "section":null,"images_count":2,"in_gallery":true,"is_ad":false,
        "images":[{
            "id":"EtJ3EyI","title":null,"description":"#transformation #weight_loss #motivation",
            "datetime":1524021941,"type":"image\/jpeg","animated":false,"width":2048,"height":2048,
            "size":368244,"views":98421,"bandwidth":36242942724,"vote":null,"favorite":false,
            "nsfw":null,"section":null,"account_url":null,"account_id":null,"is_ad":false,
            "in_most_viral":false,"has_sound":false,"tags":[],"ad_type":0,"ad_url":"",
            "in_gallery":false,"link":"https:\/\/i.imgur.com\/EtJ3EyI.jpg"
        },{
            "id":"HOny6VX","title":null,"description":null,"datetime":1524021936,
            "type":"image\/png","animated":false,"width":750,"height":1334,"size":519762,
            "views":97200,"bandwidth":50520866400,"vote":null,"favorite":false,"nsfw":null,
            "section":null,"account_url":null,"account_id":null,"is_ad":false,"in_most_viral":false,
            "has_sound":false,"tags":[],"ad_type":0,"ad_url":"","in_gallery":false,
            "link":"https:\/\/i.imgur.com\/HOny6VX.png"}]},
        "success":true,"status":200}
    "##;

    fn json(val: &'static str) -> Value {
        serde_json::from_str(val).unwrap()
    }

    #[test]
    fn format_image() {
        assert_eq!(
            "470×334 12.5KiB sfw",
            super::render_image(json(STRAIGHT_IMAGE)).unwrap()
        );

        assert_eq!(
            "640×799 97.2KiB /r/pics sfw",
            super::render_image(json(IMAGE_WITH_SECTION)).unwrap()
        );

        assert_eq!(
            "720×540 32.5KiB /r/pics sfw ፤ My army is ready, we attack at nightfall",
            super::render_image(json(IMAGE_WITH_TITLE)).unwrap()
        );

        assert_eq!(
            "667×500 8.2MiB /r/awesomenature sfw ፤ #dolphinsandshit",
            super::render_image(json(VIDEO_WITH_DESCRIPTION)).unwrap()
        );
    }

    #[test]
    fn format_album() {
        assert_eq!(
            "https://i.imgur.com/tUulJaV.jpg ፤ 640×770 87.4KiB ?fw ፤ Branch manager and Assistant Branch manager",
            super::render_gallery(json(SINGLE_IMAGE_ALBUM)).unwrap()
        );

        assert_eq!(
            "https://i.imgur.com/KbVOQOm.mp4 ፤ 580×580 1.2MiB ?fw ፤ Bango cat",
            super::render_gallery(json(SINGLE_IMAGE_ANIMATED)).unwrap()
        );

        assert_eq!(
            "0/2 animated ፤ 867.2KiB ?fw ፤ Transformation Tuesday: went from 6xl to 3xl... still got ways to go. Thanks imgur",
            super::render_gallery(json(MULTI_IMAGE_ALBUM)).unwrap()
        );
    }
}
