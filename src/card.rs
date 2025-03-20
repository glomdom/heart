use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Default)]
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

#[derive(Encode, Decode, Debug, Clone)]
pub struct CardCollection {
    pub cards: Vec<Card>,
}