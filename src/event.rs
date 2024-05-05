use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;

use super::types::{Item, Listing, StrIntValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event", content = "payload")]
pub enum Event {
    #[serde(rename = "listing-update")]
    ListingUpdate(EventListing),
    #[serde(rename = "listing-delete")]
    ListingDelete(EventListingDeletion),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventListing {
    pub id: String,
    pub steamid: String,
    pub appid: u32,
    pub details: Option<String>,
    pub currencies: BPCurrencies,
    pub intent: String,
    #[serde(rename = "listedAt")]
    pub listed_at: u32,
    #[serde(rename = "bumpedAt")]
    pub bumped_at: u32,
    pub count: u32,
    pub status: String,
    pub source: String,
    pub item: EventItem,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventListingDeletion {
    pub id: String,
    pub item: EventItem,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BPCurrencies {
    pub metal: Option<f32>,
    pub keys: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventItem {
    pub id: StrIntValue,
    #[serde(rename = "originalId")]
    pub original_id: StrIntValue,
    pub name: String,
    pub defindex: u32,
}

/*
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub event: EventType,
    #[serde(flatten)]
    pub listing: Listing,
}*/

/*#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum Event {
}*/
