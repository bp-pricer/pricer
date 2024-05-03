use crate::sources::PricingError;
use log::error;
use reqwest::Client;

use self::types::{Listing, ListingResponse};

use super::PriceSource;

const BASE_URL: &str = "https://backpack.tf/api";

pub mod types;

#[derive(Clone)]
pub struct BackpackTF {
    req_client: Client,
    auth_key: String,
    user_token: String,
}

impl BackpackTF {
    pub fn new(auth_key: String, user_token: String) -> Result<Self, ()> {
        let Ok(client) = reqwest::ClientBuilder::new().build() else {
            return Err(());
        };

        Ok(Self {
            req_client: client,
            user_token,
            auth_key,
        })
    }

    /// Requests a snapshot of the given item from Backpack.tf
    /// This is used to get the current listings for the item.
    /// The original item name is used to get the listings.
    pub async fn get_snapshot(&self, item: &str) -> Result<ListingResponse, PricingError> {
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

        match req.json().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("Failed to parse JSON from request: {:?}", e);
                Err(PricingError::InternalError)
            }
        }
    }
}

impl PriceSource for BackpackTF {
    async fn get_lowest_seller(&self, item: &str) -> Result<Listing, PricingError> {
        let mut snapshot = self.get_snapshot(item).await?;

        // removes listings that are not selling the item, and then sorts by price
        snapshot
            .filter_not_selling()
            .listings
            .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

        // lowest one is the first one
        let lowest_price = snapshot.listings.first().unwrap();

        // TODO: Do proper conversion with the key price, this is currently also a loss of data due to the cast
        Ok(lowest_price.clone())
    }

    async fn get_listings(&self, item: &str) -> Result<Vec<Listing>, PricingError> {
        let snapshot = self.get_snapshot(item).await?;

        Ok(snapshot.listings)
    }
}
