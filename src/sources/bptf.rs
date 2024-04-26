use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tf2_price::Currencies;
use tf2_sku::SKU;

use crate::source::{PriceSource, PricingError};


#[derive(Debug, Serialize, Deserialize)]
pub struct Listing {
    steamid: String,
    offers: u32,
    buyout: i32,
    details: String,
    intent: String,
    timestamp: u32,
    price: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    id: u32,
    original_id: u32,
    defindex: u32,
    level: u32,
    quality: u32,
    inventory: u32,
    quantity: u32,
    origin: u32,
    attributes: Vec<ItemAttributes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemAttributes {
    defindex: u32,
    value: String,
    float_value: Option<f32>,
}

const BASE_URL: &'static str = "https://backpack.tf/api";

pub struct BackpackTF {
    req_client: Client,
    auth_key: String,
    user_token: String
}

impl BackpackTF {
    pub fn new(auth_key: String, user_token: String) -> Result<Self, ()> {
        let Ok(client) = reqwest::ClientBuilder::new().build() else {

            return Err(());
        };

        Ok(Self {
            req_client: client,
            user_token,
            auth_key
        })
    }
}

impl PriceSource for BackpackTF {
    async fn get_lowest_seller(&self, item: SKU) -> Result<Currencies, PricingError> {
        let price = Currencies::new();

        let req = self.req_client.get(format!("{}/classifieds/listings/snapshot", BASE_URL)).query(&[("token", &self.user_token), ("appid", &"440".to_owned()), ("sku",  &"The Original".to_owned())]).send().await.unwrap();
        
        if req.status().is_server_error() {
            return Err(PricingError::ServerError);
        }

        if req.status() != 200 {
            error!("Failed to get price from Backpack.tf: {:?}, {:?}", req.status(), req.text().await.unwrap());
            return Err(PricingError::InternalError);
        }

        Ok(price)   
    }
}