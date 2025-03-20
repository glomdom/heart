use bincode::{Decode, Encode, config::standard};
use lz4::{Decoder, EncoderBuilder};
use quick_xml::{
    Reader,
    events::{self, Event},
};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
};

#[derive(Encode, Decode, Debug, Clone)]
pub struct Card {
    pub id: u32,     // 4b
    pub version: u8, // 1b
    pub cost: u32,   // 4b
    pub attack: u32, // 4b
    pub health: u32, // 4b
    pub cardid: String,
    pub name: String,
    pub hand_text: Option<String>,
    pub flavor_text: Option<String>,
}

#[derive(Encode, Decode, Debug)]
pub struct CardCollection {
    pub cards: Vec<Card>,
}

pub fn save_cards_to_file(cards: &CardCollection, file_path: &str) -> std::io::Result<()> {
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    let mut encoder = EncoderBuilder::new().level(4).build(writer)?;
    let encoded = bincode::encode_to_vec(cards, standard()).expect("failed to serialize card data");

    encoder.write_all(&encoded)?;
    let (_output, result) = encoder.finish();

    result
}

pub fn load_cards_from_file(file_path: &str) -> std::io::Result<CardCollection> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut decoder = Decoder::new(reader)?;
    let mut decoded_data = Vec::new();

    decoder.read_to_end(&mut decoded_data)?;
    let (cards, _) = bincode::decode_from_slice(&decoded_data, standard())
        .expect("Failed to deserialize cards.");

    Ok(cards)
}

pub fn parse_carddefs_xml(file_path: &str, locale: &str) -> CardCollection {
    let file = File::open(file_path).expect("failed to open file");
    let reader = BufReader::new(file);

    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut cards = Vec::new();

    let mut current_card = Card {
        cardid: String::new(),
        id: 0,
        version: 0,
        name: String::new(),
        cost: 0,
        attack: 0,
        health: 0,
        hand_text: None,
        flavor_text: None,
    };

    let mut in_entity = false;

    while let Ok(event) = xml_reader.read_event_into(&mut buf) {
        match event {
            Event::Start(ref e) if e.name().as_ref() == b"Entity" => {
                in_entity = true;
                current_card = Card {
                    cardid: String::new(),
                    id: 0,
                    version: 0,
                    name: String::new(),
                    cost: 0,
                    attack: 0,
                    health: 0,
                    hand_text: None,
                    flavor_text: None,
                };

                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"CardID" => {
                            current_card.cardid = String::from_utf8_lossy(&attr.value).to_string();
                        }

                        b"ID" => {
                            current_card.id = String::from_utf8_lossy(&attr.value)
                                .to_string()
                                .parse()
                                .unwrap_or(0)
                        }

                        b"version" => {
                            current_card.version = String::from_utf8_lossy(&attr.value)
                                .to_string()
                                .parse()
                                .unwrap_or(0)
                        }

                        _ => (),
                    }
                }
            }

            Event::Start(ref e) | Event::Empty(ref e)
                if in_entity && e.name().as_ref() == b"Tag" =>
            {
                if let Some(tag_name) = get_tag_name(e) {
                    // special tags
                    match tag_name.as_str() {
                        "CARDNAME" => {
                            if let Some(card_name) =
                                parse_localization(&mut xml_reader, &mut buf, locale)
                            {
                                current_card.name = card_name;
                            }
                        }

                        "CARDTEXT" => {
                            if let Some(card_text) =
                                parse_localization(&mut xml_reader, &mut buf, locale)
                            {
                                current_card.hand_text = Some(card_text);
                            }
                        }

                        "FLAVORTEXT" => {
                            if let Some(flavor_text) =
                                parse_localization(&mut xml_reader, &mut buf, locale)
                            {
                                current_card.flavor_text = Some(flavor_text);
                            }
                        }

                        _ => {
                            handle_tag_attributes(&mut current_card, e);
                        }
                    }
                }
            }

            Event::End(ref e) if e.name().as_ref() == b"Entity" => {
                cards.push(current_card.clone());
                in_entity = false;
            }

            Event::Eof => break,

            _ => {}
        }

        buf.clear();
    }

    CardCollection { cards }
}

fn get_tag_name(e: &events::BytesStart) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"name" {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }

    None
}

fn handle_tag_attributes(card: &mut Card, e: &events::BytesStart) {
    let mut tag_name = String::new();
    let mut tag_value = String::new();

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"name" => tag_name = String::from_utf8_lossy(&attr.value).to_string(),
            b"value" => tag_value = String::from_utf8_lossy(&attr.value).to_string(),

            _ => {}
        }
    }

    match tag_name.as_str() {
        "COST" => card.cost = tag_value.parse().unwrap_or(0),
        "ATK" => card.attack = tag_value.parse().unwrap_or(0),
        "HEALTH" => card.health = tag_value.parse().unwrap_or(0),

        _ => {}
    }
}

pub fn parse_localization<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    locale: &str,
) -> Option<String> {
    while let Ok(event) = reader.read_event_into(buf) {
        match event {
            Event::Start(ref e) if e.name().as_ref() == locale.as_bytes() => {
                if let Ok(Event::Text(text)) = reader.read_event_into(buf) {
                    let raw_text = text.unescape().unwrap_or_default();
                    return Some(raw_text.into_owned());
                }
            }

            Event::End(ref e) if e.name().as_ref() == b"Tag" => break,
            Event::Eof => break,

            _ => {}
        }

        buf.clear();
    }

    None
}
