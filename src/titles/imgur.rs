use serde_json::Value;

use errors::*;
use webs::Webs;

pub fn image<W: Webs>(webs: &mut W, id: &str) -> Result<String> {
    let resp = webs.imgur_get(&format!("image/{}", id))?;
    let data = resp.get("data").ok_or("missing data")?;

    image_body(data, None)
}

fn image_body(data: &Value, title_hint: Option<&str>) -> Result<String> {
    let mut title = format!(
        "{}×{}",
        data.get("width").ok_or("missing width")?,
        data.get("height").ok_or("missing height")?
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

pub fn gallery<W: Webs>(webs: &mut W, id: &str) -> Result<String> {
    let resp = webs.imgur_get(&format!("album/{}", id))?;
    let data = resp.get("data").ok_or("missing data")?;

    let gallery_title = data.get("title")
        .and_then(|t| t.as_str())
        .ok_or("missing title")?;

    let count = data.get("images_count")
        .and_then(|c| c.as_i64())
        .ok_or("no image count")?;

    let images = data.get("images")
        .and_then(|v| v.as_array())
        .ok_or("no images")?;

    if 1 == count && !images.is_empty() {
        let image = &images[0];
        return Ok(format!(
            "{} ፤ {}",
            image
                .get("link")
                .and_then(|s| s.as_str())
                .ok_or("no link on embedded image")?,
            image_body(image, Some(gallery_title))?
        ));
    }

    let mut animated = 0;
    let mut size = 0.;

    for image in images {
        size += preferred_size(image).ok_or("album image is missing size")?;
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

fn show_size(val: f64) -> String {
    use number_prefix::binary_prefix;
    use number_prefix::Prefixed;
    use number_prefix::Standalone;

    match binary_prefix(val) {
        Standalone(bytes) => format!("{} bytes", bytes),
        Prefixed(prefix, n) => format!("{:.1}{}B", n, prefix),
    }
}

#[cfg(test)]
mod tests {
    use reqwest::IntoUrl;
    use serde_json;
    use serde_json::Value;

    use errors::*;
    use webs::Resp;
    use webs::Webs;

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

    struct ImgurTest;

    impl Webs for ImgurTest {
        fn imgur_get(&self, sub: &str) -> Result<Value> {
            Ok(match sub {
                "album/rTV6u" => serde_json::from_str(SINGLE_IMAGE_ALBUM).unwrap(),
                "album/mk0v7" => serde_json::from_str(MULTI_IMAGE_ALBUM).unwrap(),
                "image/TUgcjTQ" => serde_json::from_str(STRAIGHT_IMAGE).unwrap(),
                "image/PmSOx4H" => serde_json::from_str(IMAGE_WITH_TITLE).unwrap(),
                "image/SRup0KZ" => serde_json::from_str(VIDEO_WITH_DESCRIPTION).unwrap(),
                "image/zEG4ULo" => serde_json::from_str(IMAGE_WITH_SECTION).unwrap(),
                other => unimplemented!(),
            })
        }

        fn raw_get<U: IntoUrl>(&self, url: U) -> Result<Resp> {
            unimplemented!()
        }
    }

    #[test]
    fn format_image() {
        assert_eq!(
            "470×334 12.5KiB sfw",
            super::image(&mut ImgurTest {}, "TUgcjTQ").unwrap()
        );

        assert_eq!(
            "640×799 97.2KiB /r/pics sfw",
            super::image(&mut ImgurTest {}, "zEG4ULo").unwrap()
        );

        assert_eq!(
            "720×540 32.5KiB /r/pics sfw ፤ My army is ready, we attack at nightfall",
            super::image(&mut ImgurTest {}, "PmSOx4H").unwrap()
        );

        assert_eq!(
            "667×500 8.2MiB /r/awesomenature sfw ፤ #dolphinsandshit",
            super::image(&mut ImgurTest {}, "SRup0KZ").unwrap()
        );
    }

    #[test]
    fn format_album() {
        assert_eq!(
            "https://i.imgur.com/tUulJaV.jpg ፤ 640×770 87.4KiB ?fw ፤ Branch manager and Assistant Branch manager",
            super::gallery(&mut ImgurTest {}, "rTV6u").unwrap()
        );

        assert_eq!(
            "0/2 animated ፤ 867.2KiB ?fw ፤ Transformation Tuesday: went from 6xl to 3xl... still got ways to go. Thanks imgur",
            super::gallery(&mut ImgurTest {}, "mk0v7").unwrap()
        );
    }
}
