use std::io::Read;

use reqwest as requests; // For my own sanity, i _hate_ typing reqwests
use serde::{Serialize, Deserialize};
use aws_sdk_ssm::{Client, Error};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticationToken {
    access_token: String
}

#[derive(Debug)]
pub struct TwitterClient {
    client: requests::Client,
    auth_token: String,
}

impl TwitterClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let auth_token = get_auth_token().await?;
        let client = requests::Client::new();

        Ok(Self { client, auth_token })
    }

    pub async fn get(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.get(url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?
            .text()
            .await?;

        println!("{}", response);

        Ok(())
    }

    pub async fn tweet(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn upload_image(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

async fn get_api_key(ssm_client: &Client, param_name: &str) -> Result<String, Error> {
    let response = ssm_client.get_parameter().name(param_name).send().await?;
    let param = response.parameter()
        .unwrap()
        .value()
        .unwrap_or_default();

    Ok(param.to_string())
}

async fn get_auth_token() -> Result<String, Box<dyn std::error::Error>> {
    let awscfg = aws_config::load_from_env().await;
    let ssm_client = Client::new(&awscfg);
    let api_key = get_api_key(&ssm_client, "TwartbotApiKey").await?;
    let api_secret = get_api_key(&ssm_client, "TwartbotApiSecretKey").await?;
    
    let http_client = requests::Client::new();
    let bearer_token = http_client.post("https://api.twitter.com/oauth2/token")
        .basic_auth(api_key, Some(api_secret))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?
        .json::<AuthenticationToken>()
        .await?
        .access_token;

    Ok(bearer_token)
}

pub async fn test_twitter() -> Result<(), Box<dyn std::error::Error>> {
    let twitter_client = TwitterClient::new().await?;
    twitter_client.upload_image().await?;
    Ok(())
}
