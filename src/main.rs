use source::PriceSource;
use sources::bptf::BackpackTF;
use tf2_sku::{tf2_enum::Quality, SKU};


pub mod source;
pub mod sources;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let bptf = BackpackTF::new(std::env::var("BPTF_API_KEY").unwrap(), std::env::var("BPTF_USER_KEY").unwrap()).unwrap();

    bptf.get_lowest_seller(SKU::new(513, Quality::Unique)).await.unwrap();
}