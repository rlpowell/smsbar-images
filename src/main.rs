use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::{Tz, US::Pacific};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

use base64::decode;

use std::env;
use std::error::Error;
use std::fs;
use std::str;

#[derive(PartialEq, Clone)]
enum State {
    Between,
    Mms(DateTime<Tz>),
}

fn try_event_name(event: Event) -> Option<String> {
    match event.clone() {
        Event::Start(e) => Some(str::from_utf8(e.name().as_ref()).unwrap().to_owned()),
        Event::End(e) => Some(str::from_utf8(e.name().as_ref()).unwrap().to_owned()),
        Event::Empty(e) => Some(str::from_utf8(e.name().as_ref()).unwrap().to_owned()),
        _ => None,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut days_back: i64 = 9999;

    if args.len() > 3 {
        panic!("Only takes two arguments, XML file and (optional) number of days back to process images");
    }

    if args.len() < 2 {
        panic!("Takes one or two arguments, XML file and (optional) number of days back to process images");
    }

    let xml_filename = args[1].clone();

    if args.len() == 3 {
        days_back = args[2].parse().unwrap();
    }

    let date_since = Utc::now() - Duration::days(days_back);

    // Used to track whether we're currently processing an MMS, and if so what the timestamp on it is
    let mut state = State::Between;

    println!("reading xml file to string");
    let xml_string = fs::read_to_string(xml_filename)?;

    println!("parsing the xml string");

    let mut reader = Reader::from_str(xml_string.as_str());
    reader.trim_text(true);

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        let event = reader.read_event().unwrap_or_else(|e| {
            panic!(
                "Error at file character number {}: {:#?}",
                reader.buffer_position(),
                e
            )
        });

        // Here we match on all the things we care about:
        // the event itself, its name if it has one, and our current state
        match (
            event.clone(),
            try_event_name(event).as_ref().map(String::as_ref),
            state.clone(),
        ) {
            (Event::Eof, _, _) => break,

            (Event::Start(e), Some("mms"), State::Between) => {
                if let Ok(Some(attr)) = e.try_get_attribute("date") {
                    // println!("{:#?}", attr);
                    if let Ok(date_i64) = attr.unescape_value()?.parse::<i64>() {
                        // println!("{:#?}", date_i64);
                        let date_time = Pacific.timestamp_millis_opt(date_i64).unwrap();
                        // println!("{}", date_time.format("%Y-%d-%m %H:%M %Z"));

                        state = State::Mms(date_time);
                    }
                }
            }

            (Event::Start(_), Some("mms"), State::Mms(_)) => {
                panic!("found an mms start tag when we were already processing an mms??")
            }

            // MMS attachments are represented in the XML as "parts";
            // we don't about the "parts" tag, only the "part" tags underneath
            (Event::Empty(e), Some("part"), State::Mms(date_time)) => {
                if date_time < date_since {
                    println!("date {} is too old", date_time.format("%Y-%d-%m %H:%M %Z"));
                } else if let Ok(Some(ct)) = e.try_get_attribute("ct") {
                    let ct = ct.unescape_value()?.to_string();
                    if ct.starts_with("image/") {
                        if let Ok(Some(orig_filename)) = e.try_get_attribute("cl") {
                            let orig_filename = orig_filename.unescape_value()?.to_string();
                            if let Ok(Some(data)) = e.try_get_attribute("data") {
                                let data = data.unescape_value()?.to_string();
                                let binary = decode(data).unwrap();

                                let filename = format!(
                                    "output/{}--IMG_MMS_{}",
                                    date_time.format("%Y-%m-%d_%H-%M-%S"),
                                    orig_filename
                                );
                                println!("Writing {}", filename);
                                fs::write(filename, binary).expect("can't write the file :(");
                            } else {
                                panic!("Image with no data ('data' attr): {:#?}", e)
                            }
                        } else {
                            panic!("Image with no filename ('cl' attr): {:#?}", e)
                        }
                    }
                }
            }
            (Event::Empty(_), Some("part"), State::Between) => {
                panic!("found a part when not inside an mms")
            }
            (Event::End(_), Some("mms"), State::Mms(_)) => {
                state = State::Between;
            }
            (Event::End(_), Some("mms"), State::Between) => {
                panic!("found an mms end when not inside one")
            }
            _ => (),
        }
    }

    Ok(())
}
