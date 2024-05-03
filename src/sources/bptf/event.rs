use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;

use super::types::{Item, Listing};

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
    id: String,
    appid: u32,
    currencies: BPCurrencies,
    intent: String,
    #[serde(rename = "listedAt")]
    listed_at: u32,
    #[serde(rename = "bumpedAt")]
    bumped_at: u32,
    count: u32,
    status: String,
    source: String,
    item: EventItem,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventListingDeletion {
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BPCurrencies {
    pub metal: Option<f32>,
    pub keys: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventItem {
    name: String,
    defindex: u32,
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
