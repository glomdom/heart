use log::{LevelFilter, info};
use serialization::{load_cards_from_file, save_cards_to_file};
use std::time::Instant;
use xml_parser::parse_carddefs_xml;

mod card;
mod serialization;
mod xml_parser;

fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .init();

    let parse_start = Instant::now();
    let defs = parse_carddefs_xml("hsdata/CardDefs.xml", "enUS");
    let parse_duration = parse_start.elapsed();
    info!(
        "finished parsing {} cards from `hsdata/CardDefs.xml` in {:.2?}",
        defs.cards.len(),
        parse_duration
    );

    let save_start = Instant::now();
    save_cards_to_file(&defs, "cards.dat").expect("failed to save cards to file");
    let save_duration = save_start.elapsed();
    info!("saved compressed cards in {:.2?}", save_duration);

    let load_start = Instant::now();
    let loaded_defs = load_cards_from_file("cards.dat").expect("failed to load cards from file");
    let load_duration = load_start.elapsed();
    info!(
        "read {} cards in {:.2?}",
        loaded_defs.cards.len(),
        load_duration
    );

    info!("parsing took: {:.2?}", parse_duration);
    info!("saving took: {:.2?}", save_duration);
    info!("loading took: {:.2?}", load_duration);
}
