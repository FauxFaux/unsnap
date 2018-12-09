use quick_xml::events::attributes::Attribute;
use quick_xml::events::Event;

pub fn highest_stream(playlist: &str) -> Result<String, &'static str> {
    let mut reader = quick_xml::Reader::from_str(playlist);
    let mut curr_bandwidth = None;
    let mut best = None;
    let mut buf = Vec::new();
    let mut txt = String::new();
    let mut collect_text = false;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Representation" => {
                    curr_bandwidth = e.attributes().find_map(|a| a.ok().and_then(bandwidth));
                }
                b"BaseURL" => {
                    if let Some(curr) = curr_bandwidth {
                        if best.as_ref().map(|(_name, band)| *band).unwrap_or(0) < curr {
                            collect_text = true;
                        }
                    }
                }
                _ => (),
            },
            Ok(Event::Text(ref e)) if collect_text => {
                txt.push_str(&e.unescape_and_decode(&reader).unwrap())
            }
            Ok(Event::End(ref e)) => match e.name() {
                b"Representation" => curr_bandwidth = None,
                b"BaseURL" if collect_text => {
                    collect_text = false;
                    if let Some(curr) = curr_bandwidth {
                        best = Some((txt.to_string(), curr));
                        txt.clear();
                    }
                }
                _ => (),
            },
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
    }

    if let Some((best, _band)) = best {
        Ok(best)
    } else {
        Err("dash playlist parsing failed to find a match")
    }
}

fn bandwidth(a: Attribute) -> Option<u64> {
    if a.key == b"bandwidth" {
        String::from_utf8_lossy(&a.value).parse().ok()
    } else {
        None
    }
}

#[test]
fn highest() {
    assert_eq!(
        Ok("DASH_2_4_M".to_string()),
        highest_stream(include_str!("../../tests/DASHPlaylist.mpd"))
    )
}
