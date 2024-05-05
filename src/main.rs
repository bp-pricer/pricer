use bptf::BackpackTF;
use db::Database;

pub mod bptf;
pub mod db;
pub mod event;
pub mod types;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let db = Database::default();

    let mut bptf = BackpackTF::new(
        std::env::var("BPTF_API_KEY").unwrap(),
        std::env::var("BPTF_USER_KEY").unwrap(),
        db.clone(),
    )
    .unwrap();

    let item_str = match std::env::var("ITEMS") {
        Ok(item_str) => item_str,
        Err(_) => {
            panic!("ITEMS not set in .env");
        }
    };

    let items = item_str.split(',').collect::<Vec<&str>>();
    let items_owned: Vec<String> = items.iter().map(|&x| x.into()).collect();
    let items_ws = items_owned.clone();
    /*tokio::spawn(async move {
        bptf.watch_snapshots(items.clone()).await;
    })
    .await;*/

    let bp_other = bptf.clone();
    /*tokio::spawn(async move {
        bptf.watch_snapshots(items_owned.clone()).await;
    });*/

    tokio::spawn(async move {
        bp_other.watch_websocket(items_ws).await;
    });

    std::thread::sleep(std::time::Duration::from_secs(100));
}
