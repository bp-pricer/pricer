use log::{error, info, warn};
use redis::{AsyncCommands, Client, RedisError};
use serde_json::Value;

use crate::{
    event::{EventListing, EventListingDeletion, UniversalItem, UniversalListing},
    types::Listing,
};

#[derive(Clone)]
pub struct Database {
    client: Client,
}

impl Default for Database {
    fn default() -> Self {
        let url = match std::env::var("REDIS_URL") {
            Ok(url) => url,
            Err(_) => {
                panic!("REDIS_URL not set in .env");
            }
        };

        let client = match Client::open(url) {
            Ok(client) => client,
            Err(e) => {
                panic!("Failed to connect to redis: {:?}", e);
            }
        };

        info!("Successfully connected to the redis server");

        Self { client }
    }
}

impl Database {
    pub async fn store_listings(&self, listings: Vec<EventListing>) {
        for listing in listings {
            let mut con = match self.client.get_multiplexed_async_connection().await {
                Ok(con) => con,
                Err(e) => {
                    panic!("Failed to get connection to redis: {:?}", e);
                }
            };

            let id = listing.id.clone();

            let key = format!("listing:{}:{}", listing.item.defindex, listing.id);
            let db_listing: UniversalListing = listing.into();
            let value = serde_json::to_string(&db_listing).unwrap();

            match con.set(key, value).await {
                Ok(()) => {
                    info!("Stored listing with id {}", id);
                }
                Err(e) => {
                    panic!("Failed to store listing with id {}: {:?}", id, e);
                }
            }
        }
    }

    /// Store the item definitions in the database
    /// one definition is a tuple of (name, defindex)
    pub async fn store_item_definitions(
        &self,
        definitions: Vec<(String, u32)>,
    ) -> Result<(), RedisError> {
        let mut con = match self.client.get_multiplexed_async_connection().await {
            Ok(con) => con,
            Err(e) => {
                panic!("Failed to get connection to redis: {:?}", e);
            }
        };

        for (name, defindex) in definitions {
            let key = format!("item:{}", defindex);
            let value = name;

            match con.set(key, value).await {
                Ok(()) => {
                    info!("Stored item definition with defindex {}", defindex);
                }
                Err(e) => {
                    panic!(
                        "Failed to store item definition with defindex {}: {:?}",
                        defindex, e
                    );
                }
            }
        }

        Ok(())
    }

    pub async fn handle_delete_events(&self, listings: Vec<EventListingDeletion>) {
        let mut deleted = 0;

        let mut con = match self.client.get_multiplexed_async_connection().await {
            Ok(con) => con,
            Err(e) => {
                panic!("Failed to get connection to redis: {:?}", e);
            }
        };

        for listing in listings {
            let key = format!("listing:{}:{}", listing.item.defindex, listing.id);

            match con.del(key).await {
                Ok(()) => {
                    deleted += 1;
                    //info!("Deleted listing with id {}", listing.id);
                }
                Err(e) => {
                    panic!("Failed to delete listing with id {}: {:?}", listing.id, e);
                }
            }
        }

        if deleted > 0 {
            info!("Deleted {} listings", deleted);
        }
    }

    pub async fn update_listings_from_websocket(
        &self,
        listings: Vec<EventListing>,
    ) -> Result<(), RedisError> {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                panic!("Failed to get connection to redis: {:?}", e);
            }
        };

        let mut updated = 0;
        let mut created = 0;

        for listing in listings {
            let key = format!("listing:{}:{}", listing.item.defindex, listing.id);
            let db_listing: UniversalListing = listing.into();
            let value = serde_json::to_string(&db_listing).unwrap();

            match conn.exists(key.clone()).await {
                Ok(exists) => {
                    if exists {
                        updated += 1;
                    } else {
                        created += 1;
                    }
                }
                Err(e) => {
                    panic!(
                        "Failed to check if listing with key {} exists: {:?}",
                        key, e
                    );
                }
            }

            match conn.set(key.clone(), value).await {
                Ok(()) => {
                    //info!("Modified listing with key {}", key);
                }
                Err(e) => {
                    panic!("Failed to update listing with key {}: {:?}", key, e);
                }
            }
        }

        if updated > 0 || created > 0 {
            info!(
                "Updated {} listings, created {} listings from websocket",
                updated, created
            );
        }

        Ok(())
    }

    pub async fn scan_for_old_listings(&self) -> Result<(), RedisError> {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                panic!("Failed to get connection to redis: {:?}", e);
            }
        };

        let keys: Vec<String> = match conn.keys("listing:*").await {
            Ok(keys) => keys,
            Err(e) => {
                panic!("Failed to get keys from redis: {:?}", e);
            }
        };

        let mut deleted = 0;

        for key in keys {
            let value: String = match conn.get(&key).await {
                Ok(value) => value,
                Err(e) => {
                    warn!("Failed to get value from redis: {:?} - key: {:?}", e, key);
                    //panic!();
                    continue;
                }
            };

            let db_listing: UniversalListing = match serde_json::from_str(&value) {
                Ok(db_listing) => db_listing,
                Err(e) => {
                    panic!("Failed to deserialize listing: {:?}", e);
                }
            };

            if db_listing.bumped_at < (chrono::Utc::now().timestamp() - 86400) as u32 {
                conn.del::<&str, bool>(&key).await.unwrap();
                deleted += 1;
                info!(
                    "Deleted listing with key {}, reason: too old (bumped {}m ago), {:?}",
                    key,
                    (chrono::Utc::now().timestamp() - db_listing.bumped_at as i64) / 60,
                    db_listing
                );
            }
        }

        if deleted > 0 {
            info!("Deleted {} listings that were too old", deleted);
        }

        Ok(())
    }

    /// Updates all entries in the database from a snapshot by finding already existing entries
    /// or creating new ones
    ///
    /// A listing update does not automatically mean that anything has changed since it always writes the listing to the database
    pub async fn update_listings_from_snapshot(
        &self,
        listings: Vec<Listing>,
        item: &str,
    ) -> Result<(), RedisError> {
        let mut updated = 0;
        let mut created = 0;
        let mut con = match self.client.get_multiplexed_async_connection().await {
            Ok(con) => con,
            Err(e) => {
                panic!("Failed to get connection to redis: {:?}", e);
            }
        };

        for listing in listings {
            let mut key = String::new();

            // skip listings without a user agent aka. not a bot
            if listing.user_agent.is_none() {
                continue;
            }

            if listing.intent == "sell" {
                let item_id = match listing.item.id {
                    Some(id) => id,
                    None => {
                        panic!("Listing with intent sell has no item id");
                    }
                };
                let id = format!("440_{}", item_id);
                key = format!("listing:{}:{}", listing.item.defindex, id);
            } else {
                let id = format!("440_{}_{:x}", listing.steamid, md5::compute(item));
                key = format!("listing:{}:{}", listing.item.defindex, id);
            }

            // TODO: remove this, debug only
            match con.exists(key.clone()).await {
                Ok(exists) => {
                    if exists {
                        updated += 1;
                    } else {
                        created += 1;
                    }
                }
                Err(e) => {
                    panic!(
                        "Failed to check if listing with key {} exists: {:?}",
                        key, e
                    );
                }
            }

            let db_value: UniversalListing = listing.into();
            let value = serde_json::to_string(&db_value).unwrap();

            match con.set(key.clone(), value).await {
                Ok(()) => {
                    //info!("Modified listing with key {}", key);
                }
                Err(e) => {
                    panic!("Failed to update listing with key {}: {:?}", key, e);
                }
            }
        }

        if updated > 0 || created > 0 {
            info!(
                "Updated {} listings, created {} listings from snapshot",
                updated, created
            );
        }
        Ok(())
    }

    /// Get all the listings for a given item defindex
    /// This also deletes all the listings that are older than 7 days
    pub async fn get_item_listings(
        &self,
        defindex: u32,
    ) -> Result<Vec<UniversalListing>, RedisError> {
        let mut con = match self.client.get_multiplexed_async_connection().await {
            Ok(con) => con,
            Err(e) => {
                panic!("Failed to get connection to redis: {:?}", e);
            }
        };

        let key = format!("listing:{}:*", defindex);
        let keys: Vec<String> = match con.keys(key).await {
            Ok(keys) => keys,
            Err(e) => {
                panic!("Failed to get keys from redis: {:?}", e);
            }
        };

        let mut listings = Vec::new();

        for key in keys {
            let value: String = match con.get(&key).await {
                Ok(value) => value,
                Err(e) => {
                    panic!("Failed to get value from redis: {:?}", e);
                }
            };

            let listing: UniversalListing = match serde_json::from_str(&value) {
                Ok(listing) => listing,
                Err(e) => {
                    panic!("Failed to deserialize listing: {:?}", e);
                }
            };

            if listing.bumped_at < (chrono::Utc::now().timestamp() - 259200) as u32 {
                // I have no clue what those args are
                con.del::<&str, bool>(&key).await.unwrap();
                info!("Deleted listing with id {:?}, reason: too old", listing.id);
            }

            listings.push(listing);
        }

        Ok(listings)
    }
}
