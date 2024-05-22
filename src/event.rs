use log::info;
use serde::{Deserialize, Serialize};
use super::types::{Listing, StrIntValue};

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
    #[serde(rename = "value")]
    pub value: ListingValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListingValue {
    raw: f32
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
    pub original_id: Option<StrIntValue>,
    pub name: String,
    pub defindex: u32,
}




/// Universal listing type used to be converted from/to to store in the database
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UniversalListing {
    pub id: Option<String>,
    pub steamid: String,
    pub details: Option<String>,
    pub intent: String,
    pub price: f32,
    pub bumped_at: f32,
    pub item: UniversalItem,
    // TODO: user agent, item, timestamp?
}

impl From<Listing> for UniversalListing {
    fn from(listing: Listing) -> Self {
      //  info!("listing item id: {:?}", listing.item.id);
        Self {
            id: None,
            steamid: listing.steamid,
            details: Some(listing.details),
            intent: listing.intent,
            price: listing.price,
            bumped_at: listing.timestamp as f32,
            item: UniversalItem {
                id: Some(StrIntValue::Int(listing.item.id.unwrap_or(0))),
                defindex: listing.item.defindex as u32,
            },
        }
    }
}

impl From<EventListing> for UniversalListing {
    fn from(event_listing: EventListing) -> Self {
      //  info!("eventlisting item id: {:?}", event_listing.item.id);
        Self {
            id: Some(event_listing.id),
            steamid: event_listing.steamid,
            details: event_listing.details,
            intent: event_listing.intent,
            price: event_listing.value.raw,
            bumped_at: event_listing.bumped_at as f32,
            item: UniversalItem {
                id: Some(StrIntValue::Int(event_listing.item.id.into())),
                defindex: event_listing.item.defindex,
            },
        }
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UniversalItem {
    pub id: Option<StrIntValue>,
    // TODO: check if this is relevant
    //pub original_id: Option<StrIntValue>,
    pub defindex: u32,
}