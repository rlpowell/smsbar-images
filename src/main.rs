use quick_xml::events::Event;
use quick_xml::reader::Reader;
// use regex::Regex;
use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::{Tz, US::Pacific};

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

    // println!("args: {:#?}", args);
    if args.len() > 2 {
        panic!("Only takes one argument, number of days back to process images");
    }
    let mut days_back: i64 = 9999;
    if args.len() == 2 {
        days_back = args[1].parse().unwrap();
    }
    let date_since = Utc::now() - Duration::days(days_back);

    let mut state = State::Between;
    // an unem with whether we're in an mms or not, and the date

    println!("reading xml file to string");
    let xml_string = fs::read_to_string("/tmp/sms.xml")?;
    // xml_string = xml_string.replace("\r\n", "\n");

    // println!("mucking with the string");
    // // Basically we replace the whole XML header, and add a namespace
    // // to the first element because it makes minidom happy
    // let re = Regex::new(r"^(?s).*?<smses ([^>]*)>").unwrap();
    // let fixed_xml_string = re.replace_all(
    //     xml_string.as_str(),
    //     "<?xml version='1.0' encoding='UTF-8' standalone='yes' ?>\n<smses $1 xmlns=\"smses\">",
    // );
    // // println!("{:#?}", fixed_xml_string);

    println!("parsing the xml string");
    // let root: Element = fixed_xml_string.parse().unwrap();

    let mut reader = Reader::from_str(xml_string.as_str());
    reader.trim_text(true);

    // let mut count = 0;
    // let mut txt = Vec::new();
    let mut buf = Vec::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event()`
        let event = reader
            .read_event_into(&mut buf)
            .unwrap_or_else(|e| panic!("Error at position {}: {:#?}", reader.buffer_position(), e));
        match (
            event.clone(),
            try_event_name(event).as_ref().map(String::as_ref),
            state.clone(),
        ) {
            // Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            // exits the loop when reaching end of file
            (Event::Eof, _, _) => break,

            // Ok(e) => println!("{:#?}", e),

            //
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
                // if let Ok(Some(attr)) = e.try_get_attribute("readable_date") {
                //     println!("{:#?}", attr);
                // }

                // for attr in e.attributes() {
                //     let attr = attr.unwrap();
                //     println!("{:#?} -- {:#?}", attr.key, attr.unescape_value())
                // }
                //     println!(
                //     "attributes values: {:?}",
                //     e.attributes() //.map(|a| a.unwrap().value).collect::<Vec<_>>()
                // ),
            }
            (Event::Start(_), Some("mms"), State::Mms(_)) => {
                // We should only end up here if the state is *not* Between, which is bad
                panic!("found an mms when we were already processing one??")
            }
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
                                // println!("{:#?} -- {}", ct, date_time.format("%Y-%d-%m %H:%M %Z"));
                                // println!("{:#?}", orig_filename);
                                // println!("{:#?}", e);
                                // println!("{:#?}", binary);
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
            // Ok(Event::Text(e)) => txt.push(e.unescape().unwrap().into_owned()),

            // There are several other `Event`s we do not consider here
            _ => (),
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    // println!("processing elements");
    // for child in root.children() {
    //     println!("{:#?}", child);
    // }

    Ok(())
}
