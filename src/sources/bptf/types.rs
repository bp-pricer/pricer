use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use tf_item_attributes::TFItemAttribute;

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceHistory {
    success: i8,
    history: Vec<PriceHistoryNode>,
}

impl PriceHistory {
    pub fn get_average_past_24h(&self) -> Option<f32> {
        let now = Utc::now();
        let threshold = now - Duration::hours(24);

        let filtered_data = self
            .history
            .iter()
            .filter(|node| node.timestamp as i64 >= threshold.timestamp());

        todo!("Implement average calculation");
        /*  let (sum, count) = filtered_data.fold((0.0, 0), |(acc_sum, acc_count), node| {
            (acc_sum + node.value, acc_count + 1)
        });

        info!("{:?}", self.history);

        if count == 0 {
            return None;
        }

        Some(sum / count as f32) */
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceHistoryRespone {
    response: PriceHistory,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceHistoryNode {
    value: f32,
    value_high: f32,
    currency: String,
    timestamp: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Listing {
    steamid: String,
    offers: u32,
    buyout: i32,
    pub details: String,
    intent: String,
    timestamp: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub price: f32,
    pub item: Item,
    pub bump: u32,
    #[serde(rename = "userAgent")]
    user_agent: Option<UserAgent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAgent {
    #[serde(rename = "lastPulse")]
    last_pulse: u32,
    client: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Item {
    id: Option<u64>,
    original_id: Option<u64>,
    defindex: u64,
    level: Option<u8>,
    quality: u32,
    inventory: Option<u32>,
    quantity: Option<StrIntValue>,
    origin: Option<u32>,
    attributes: Option<Vec<ItemAttribute>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
enum StrIntValue {
    Str(String),
    Int(u64),
    Float(f32),
}

impl Into<u64> for StrIntValue {
    fn into(self) -> u64 {
        match self {
            StrIntValue::Str(s) => s.parse().unwrap(),
            StrIntValue::Int(i) => i,
            StrIntValue::Float(f) => f as u64,
        }
    }
}

impl From<StrIntValue> for f32 {
    fn from(value: StrIntValue) -> f32 {
        match value {
            StrIntValue::Str(s) => s.parse().unwrap(),
            StrIntValue::Int(i) => i as f32,
            StrIntValue::Float(f) => f,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ItemAttribute {
    defindex: Option<StrIntValue>,
    value: Option<StrIntValue>,
    float_value: Option<StrIntValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListingResponse {
    pub listings: Vec<Listing>,
    #[serde(rename = "createdAt")]
    created_at: u32,
}

impl ListingResponse {
    /// Removes all the listings that are not selling the given item
    pub fn filter_not_selling(&mut self) -> &mut Self {
        self.listings = <Vec<Listing> as Clone>::clone(&self.listings)
            .into_iter()
            .filter(|listing| listing.intent == "sell")
            .collect();

        self
    }

    /// Removes all the listings that are not buying the given item
    pub fn filter_not_buying(&mut self) -> &mut Self {
        self.listings = <Vec<Listing> as Clone>::clone(&self.listings)
            .into_iter()
            .filter(|listing| listing.intent == "buy")
            .collect();

        self
    }

    /// Removes all the human listings
    pub fn filter_humans(&mut self) -> &mut Self {
        self.listings = self
            .listings
            .clone()
            .into_iter()
            .filter(|listing| {
                // human = false, bot = true
                let agent = match &listing.user_agent {
                    Some(agent) => agent,
                    // Assume human if no user agent is provided
                    None => return false,
                };

                // Check if lastPulse of agent is at least 20 minutes ago
                let now = Utc::now().timestamp() as u32;
                let last_pulse = agent.last_pulse;
                let twenty_minutes = 20 * 60;

                now - last_pulse < twenty_minutes
            })
            .collect();
        self
    }

    /// Fitlters outliers from the listings
    pub fn filter_outliers(&mut self) -> &mut Self {
        let mut prices: Vec<f32> = self.listings.iter().map(|l| l.price).collect();
        prices.sort_by(|a, b| a.partial_cmp(b).unwrap()); // Sort prices for efficient median calculation
        let median_price = prices[prices.len() / 2]; // Median is the middle element

        // 20% tolerance factor
        let tolerance_factor = 1.2;
        let lower_bound = median_price / tolerance_factor;
        let upper_bound = median_price * tolerance_factor;

        // Filter listings that are outside the tolerance bounds
        self.listings = self
            .listings
            .iter()
            .filter(|l| l.price >= lower_bound && l.price <= upper_bound)
            .cloned()
            .collect();

        self
    }

    pub fn get_average(&self) -> f32 {
        let sum = self
            .listings
            .iter()
            .fold(0.0, |acc, listing| acc + listing.price);
        sum / self.listings.len() as f32
    }

    /// Removes all the listings that have the specified attributes
    /// This is used to remove listings that have attributes that are not
    /// compatible with the item we are looking for
    /// For example, if we are looking for a Strange item, we want to remove
    /// all listings that have attributes that are not compatible with Strange  
    /// items
    pub fn dont_have_attributes(&mut self, attributes: Vec<TFItemAttribute>) {
        // 1. Avoid unnecessary cloning
        let mut listings = self.listings.clone();

        // 2. Filter directly using negation
        listings = listings
            .into_iter()
            .filter_map(|listing| match &listing.item.attributes {
                Some(item_attributes) => {
                    // Check for missing attributes or incompatible ones
                    if !item_attributes.iter().any(|attr| {
                        attributes.iter().any(|a| {
                            let index: u64 = match &attr.defindex {
                                Some(index) => index.clone().into(),
                                None => return true,
                            };

                            a.clone() as u64 == index
                        })
                    }) {
                        Some(listing) // Keep listing if no matching attributes found
                    } else {
                        None // Discard listing if matching attribute found
                    }
                }
                None => Some(listing),
            })
            .collect();

        // 3. Update self.listings after filtering
        self.listings = listings;
    }
}
