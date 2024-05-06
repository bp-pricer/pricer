use std::collections::HashMap;

use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use log::{debug, error, info};
use reqwest::Client;
use serde_json::json;
use tokio_tungstenite::{
    client_async_tls, connect_async, tungstenite::protocol::frame::coding::Data,
};

use crate::{
    db::Database,
    event::{Event, EventListingDeletion},
    types::{Listing, ListingResponse, PricingError},
};

const BASE_URL: &str = "https://backpack.tf/api";
const WS_URL: &str = "wss://ws.backpack.tf/events";

#[derive(Clone)]
pub struct BackpackTF {
    req_client: Client,
    auth_key: String,
    user_token: String,
    db: Database,
    // TODO: perf: look for faster hashmap implementation
    snapshot_cache: HashMap<String, u32>,
}

impl BackpackTF {
    // TODO: impl Default instead
    pub fn new(auth_key: String, user_token: String, database: Database) -> Result<Self, ()> {
        let Ok(client) = reqwest::ClientBuilder::new().build() else {
            return Err(());
        };

        Ok(Self {
            req_client: client,
            user_token,
            auth_key,
            db: database,
            snapshot_cache: HashMap::new(),
        })
    }

    pub async fn watch_snapshots(&mut self, items: Vec<String>) {
        for item in items {
            let snapshot = self.get_snapshot(&item).await;

            if let Err(e) = snapshot {
                if e == PricingError::IsAlreadyCached {
                    debug!("Snapshot for item {} is already cached", item);
                    continue;
                }
                error!("Failed to get snapshot for item {}: {:?}", item, e);
                continue;
            }

            let listings = snapshot.unwrap().listings;
            let listings_len = listings.len();

            match self.db.update_listings_from_snapshot(listings, &item).await {
                Ok(_) => {
                    //    info!("Stored {} listings for item {}", listings_len, item);
                }
                Err(e) => {
                    error!("Failed to store listings in the database: {:?}", e);
                }
            }
        }
    }

    /// Reads the stream of events from the Backpack.tf websocket in a loop
    ///
    /// This is used later to update the price on demand
    pub async fn watch_websocket(&self, items: Vec<String>) {
        let (ws_stream, _) = match connect_async(WS_URL).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect to Backpack.tf websocket: {:?}", e);
                return;
            }
        };

        let (_, read) = ws_stream.split();

        read.for_each(|msg| async {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to read message from Backpack.tf websocket: {:?}", e);
                    return;
                }
            };

            let msg = match msg.to_text() {
                Ok(msg) => msg,
                Err(e) => {
                    error!(
                        "Failed to parse message from Backpack.tf websocket: {:?}",
                        e
                    );
                    return;
                }
            };

            if msg.is_empty() {
                return;
            }

            let events: Vec<Event> = match serde_json::from_str(msg) {
                Ok(events) => events,
                Err(e) => {
                    error!("Failed to deserialize events: {:?}", e);
                    return;
                }
            };

            let listings = events
                .clone()
                .into_iter()
                .filter_map(|event| match event {
                    Event::ListingUpdate(listing) => {
                        if items.contains(&listing.item.name) {
                            Some(listing)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect();

            let listings_deleted: Vec<EventListingDeletion> = events
                .into_iter()
                .filter_map(|event| match event {
                    Event::ListingDelete(listing) => {
                        if items.contains(&listing.item.name) {
                            Some(listing)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect();

            self.db.handle_delete_events(listings_deleted).await;
            self.db
                .update_listings_from_websocket(listings)
                .await
                .unwrap();
            /*    let listings: Vec<EventListing> = events
                .into_iter()
                .filter_map(|event| match event {
                    Event::ListingUpdate(listing) => Some(listing),
                    _ => None,
                })
                .collect();

            self.db.store_listings(listings.clone()).await;

            info!("Stored {} listings", listings.len());   */
        })
        .await;
    }

    /// Requests a snapshot of the given item from Backpack.tf
    /// This is used to get the current listings for the item.
    /// The original item name is used to get the listings.
    pub async fn get_snapshot(&mut self, item: &str) -> Result<ListingResponse, PricingError> {
        let key = self.snapshot_cache.get(item);

        if let Some(timestamp) = key {
            if Utc::now().timestamp() as u32 - timestamp < 60 {
                return Err(PricingError::IsAlreadyCached);
            }
        }

        let req = match self
            .req_client
            .get(format!("{}/classifieds/listings/snapshot", BASE_URL))
            .query(&[
                ("token", &self.user_token),
                ("appid", &"440".to_owned()),
                ("sku", &item.to_owned()),
            ])
            .send()
            .await
        {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to send request to Backpack.tf: {:?}", e);
                return Err(PricingError::InternalError);
            }
        };

        if req.status().is_server_error() {
            return Err(PricingError::ServerError);
        }

        if req.status() != 200 {
            error!(
                "Failed to get price from Backpack.tf: {:?}, {:?}",
                req.status(),
                req.text().await.unwrap()
            );
            return Err(PricingError::InternalError);
        }

        self.snapshot_cache
            .insert(item.to_owned(), Utc::now().timestamp() as u32);

        match req.json().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("Failed to parse JSON from request: {:?}", e);
                Err(PricingError::InternalError)
            }
        }
    }
}
