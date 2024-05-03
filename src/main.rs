use crate::sources::PriceSource;
use log::info;
use sources::bptf::BackpackTF;
use tf2_sku::{tf2_enum::Quality, SKU};

pub mod sources;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let bptf = BackpackTF::new(
        std::env::var("BPTF_API_KEY").unwrap(),
        std::env::var("BPTF_USER_KEY").unwrap(),
    )
    .unwrap();

    let item = "Mann Co. Supply Crate Key";

    // start timer to measure how long it takes to get the lowest seller
    // let listing = bptf.get_lowest_seller(item).await.unwrap();

    /*  for v in 0..2 {
        let bptf = bptf.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let listing = bptf.get_lowest_seller(item).await.unwrap();
            info!(
                "Found lowest seller in {:?} for {:?} at {:?} ref",
                start.elapsed(),
                item,
                listing.price
            );
            print!("{:?}", listing);
        })
        .await
        .unwrap();
    }*/
    /*info!(
        "Found lowest seller in {:?} for {:?} at {:?} ref",
        start.elapsed(),
        item,
        listing.price
    );*/

    let quality = Quality::Unique;
    let tradable = true;
    let craftable = true;
    let priceindex = "0".to_owned();

    let mut snapshot = bptf.get_snapshot(item).await.unwrap();

    let listings = snapshot
        .filter_not_selling()
        .filter_outliers()
        .filter_humans()
        .listings
        .clone();

    for listing in listings {
        info!("{:?}", listing.details);
    }
    /*    .iter()
    .for_each(|listing| {
        info!("Price: {:?}", listing.price);
    }); */

    /*info!(
        "buying + selling: {:?}, selling: {:?}, buying: {:?}",
        price_history.get_average(),
        price_history
            .is_not_selling()
            .filter_outliers()
            .get_average(),
        history_clone
            .is_not_buying()
            .filter_outliers()
            .get_average(),
    );*/
}
