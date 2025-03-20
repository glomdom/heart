use crate::card::CardCollection;
use bincode::config::standard;
use log::info;
use lz4::{Decoder, EncoderBuilder};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

pub fn save_cards_to_file(cards: &CardCollection, file_path: &str) -> std::io::Result<()> {
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    let mut encoder = EncoderBuilder::new().level(4).build(writer)?;
    let encoded = bincode::encode_to_vec(cards, standard()).expect("failed to serialize card data");

    encoder.write_all(&encoded)?;
    let (_output, result) = encoder.finish();

    info!("compressed and saved cards to {}", file_path);

    result
}

pub fn load_cards_from_file(file_path: &str) -> std::io::Result<CardCollection> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut decoder = Decoder::new(reader)?;
    let mut decoded_data = Vec::new();

    decoder.read_to_end(&mut decoded_data)?;
    let (cards, _) =
        bincode::decode_from_slice(&decoded_data, standard()).expect("failed to deserialize cards");

    info!("decompressed and loaded cards from {}", file_path);

    Ok(cards)
}
