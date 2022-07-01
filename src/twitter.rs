use crate::scryfall::Card;

use reqwest as requests; // For my own sanity, i _hate_ typing reqwests
use reqwest_oauth1::OAuthClientProvider;
use serde::{Serialize, Deserialize};
use aws_sdk_ssm::{Client, Error};

use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticationToken {
    access_token: String
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadResponse {
    media_id: i128,
    media_id_string: String,
}

async fn get_api_key(ssm_client: &Client, param_name: &str) -> Result<String, Error> {
    let response = ssm_client.get_parameter().name(param_name).send().await?;
    let param = response.parameter()
        .unwrap()
        .value()
        .unwrap_or_default();

    Ok(param.to_string())
}

async fn get_auth_token<'a>() -> Result<reqwest_oauth1::Secrets<'a>, Box<dyn std::error::Error>> {
    let awscfg = aws_config::load_from_env().await;
    let ssm_client = Client::new(&awscfg);
    let api_key = get_api_key(&ssm_client, "TwartbotApiKey").await?;
    let api_secret = get_api_key(&ssm_client, "TwartbotApiSecretKey").await?;
    let access_key = get_api_key(&ssm_client, "TwartbotAccessToken").await?;
    let access_secret = get_api_key(&ssm_client, "TwartbotAccessSecret").await?;

    let secrets = reqwest_oauth1::Secrets::new(api_key, api_secret)
        .token(access_key, access_secret);

    Ok(secrets)
}

async fn upload_image() -> Result<String, Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open("image.jpg")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let client = requests::Client::new();
    let auth_token = get_auth_token().await?;

    let form = requests::multipart::Form::new();
    let part = requests::multipart::Part::bytes(buffer);

    let form = form.part("media", part);

    println!("Uploading artwork...");
    let response = client.oauth1(auth_token)
        .post("https://upload.twitter.com/1.1/media/upload.json")
        .multipart(form)
        .send()
        .await?
        .json::<UploadResponse>()
        .await?;

    Ok(response.media_id_string)
}

pub async fn tweet_card(card: &Card) -> Result<(), Box<dyn std::error::Error>> {
    let media_id = upload_image().await?;
    let auth = get_auth_token().await?;
    
    let name = card.name.as_ref().unwrap();
    let set_name = card.set_name.as_ref().unwrap();
    let flavour = card.flavor_text.as_ref().unwrap();
    
    let update = format!(
        "{} ({}):\n\n{}\n\n#MTG #MagicTheGathering #MTGA #MTGArena",
        name, set_name, flavour
    );
    
    println!("{}", &update);
    let params = [("media_ids", media_id), ("status", update)];
    let client = requests::Client::new();

    client.oauth1(auth)
        .post("https://api.twitter.com/1.1/statuses/update.json")
        .form(&params)
        .send()
        .await?;

    println!("Tweeted successfully!");
    Ok(())
}
