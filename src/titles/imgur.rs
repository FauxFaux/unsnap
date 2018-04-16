use errors::*;
use webs::Webs;

pub fn image<W: Webs>(webs: &mut W, id: &str) -> Result<String> {
    let resp = webs.imgur_get(&format!("image/{}", id))?;
    let data = resp.get("data").ok_or("missing data")?;

    let mut title = format!(
        "{}×{}",
        data.get("width").ok_or("missing width")?,
        data.get("height").ok_or("missing height")?
    );

    if let Some(size) = data.get("size").and_then(|s| s.as_f64()) {
        title.push(' ');
        title.push_str(&show_size(size));
    }

    if let Some(section) = data.get("section").and_then(|s| s.as_str()) {
        title.push(' ');
        title.push_str("/r/");
        title.push_str(section);
    }

    title.push(' ');
    title.push_str(match data.get("nsfw") {
        Some(v) => match v.as_bool() {
            Some(true) => "NSFW",
            Some(false) => "sfw",
            None => "?fw",
        },
        None => "¿fw",
    });

    Ok(title)
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
    use serde_json;
    use serde_json::Value;

    use errors::*;
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

    struct ImgurTest;

    impl Webs for ImgurTest {
        fn imgur_get(&self, sub: &str) -> Result<Value> {
            Ok(match sub {
                "image/TUgcjTQ" => serde_json::from_str(STRAIGHT_IMAGE).unwrap(),
                "image/zEG4ULo" => serde_json::from_str(IMAGE_WITH_SECTION).unwrap(),
                other => unimplemented!(),
            })
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
    }
}
