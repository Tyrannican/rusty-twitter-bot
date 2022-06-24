use reqwest as requests;
use serde::{Serialize, Deserialize};

// TODO: Get API keys from AWS somehow
const API_KEY: &'static str = "";
const API_SECRET: &'static str = "";

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticationToken {
    access_token: String
}

pub async fn test_twitter() -> Result<(), Box<dyn std::error::Error>> {
    let client = requests::Client::new();

    let bearer_token = client.post("https://api.twitter.com/oauth2/token")
        .basic_auth(API_KEY, Some(API_SECRET))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?
        .json::<AuthenticationToken>()
        .await?
        .access_token;
    println!("{:?}", bearer_token);

    Ok(())
}
