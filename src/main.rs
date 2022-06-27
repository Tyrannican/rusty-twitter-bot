mod twitter;
mod scryfall;
use scryfall::select_card;
use twitter::test_twitter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let card = select_card().await?;
    // card.download_artwork();
    test_twitter().await?;
    Ok(())
}
