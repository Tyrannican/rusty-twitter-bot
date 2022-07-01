// it's so awkward to type reqwest so type alias ftw
use reqwest as requests;
use rand::{thread_rng, prelude::SliceRandom};
use serde::{Serialize, Deserialize};

use std::collections::HashMap;

const URL: &str = "https://api.scryfall.com/bulk-data";

#[derive(Serialize, Deserialize, Debug)]
struct BulkPageData {
    data: Vec<BulkDataEntry>
}

#[derive(Serialize, Deserialize, Debug)]
struct BulkDataEntry {
    name: String,
    download_uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub name: Option<String>,
    pub image_uris: Option<HashMap<String, String>>,
    pub flavor_text: Option<String>,
    pub set_name: Option<String>
}

impl Card {
    pub async fn download_artwork(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading artwork for {}...", self.name.as_ref().unwrap());
        let artcrop = self.image_uris
            .as_ref()
            .unwrap()
            .get("art_crop")
            .unwrap()
            .to_owned();

        let mut img_file = std::fs::File::create("image.jpg")
            .expect("unable to open file handle");

        let task = tokio::task::spawn_blocking( move || {    
            requests::blocking::get(artcrop)
                .unwrap()
                .copy_to(&mut img_file)
                .unwrap();
            }
        );

        task.await?;

        Ok(())
    }

    pub fn is_invalid(&self) -> bool {
        let setname = self.set_name.as_ref().unwrap();
        match setname.as_str() {
            "Unglued" | "Unhinged" | "Unstable" | "Unsanctioned" => return true,
            _ => {}
        }
        
        if self.flavor_text.is_none() {
            return true;
        }

        if self.image_uris.is_none() {
            return true;
        }

        false
    }
}

pub async fn select_card() -> Result<Card, Box<dyn std::error::Error>> {
    let mut cards = download_cards().await?; 
    let mut valid_cards: Vec<Card> = cards
        .drain(..)
        .filter_map(|c| match c.is_invalid() {
            false => Some(c),
            true => None
        })
        .collect();

    let mut rng = thread_rng();
    valid_cards.shuffle(&mut rng);

    let card = valid_cards.choose(&mut rng).unwrap();
    Ok(card.clone())
}

async fn download_cards() -> Result<Vec<Card>, Box<dyn std::error::Error>> {
    println!("Fetching Scryfall download link...");
    let bulk_data_entries = requests::get(URL)
        .await?
        .json::<BulkPageData>()
        .await?
        .data;

    let oracle_cards: Vec<String> = bulk_data_entries
        .iter()
        .filter(|e| e.name == "Oracle Cards")
        .map(|e| e.download_uri.to_string())
        .collect();

    println!("Downloading Oracle Card data...");
    let download_link = &oracle_cards[0];
    let cards = requests::get(download_link)
        .await?
        .json::<Vec<Card>>()
        .await?;

    Ok(cards)
}
