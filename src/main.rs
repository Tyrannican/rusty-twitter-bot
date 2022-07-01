mod twitter;
mod scryfall;
use scryfall::select_card;
use twitter::tweet_card;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let card = select_card().await?;
    card.download_artwork().await?;
    
    tweet_card(&card).await?;
    Ok(())
}
