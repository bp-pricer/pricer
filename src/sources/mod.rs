use self::bptf::types::Listing;

pub mod bptf;

#[derive(Debug)]
pub enum PricingError {
    ServerError,
    InternalError,
    InvalidConfig,
}

pub trait PriceSource {
    fn get_lowest_seller(
        &self,
        item: &str,
    ) -> impl std::future::Future<Output = Result<Listing, PricingError>> + Send;

    fn get_listings(
        &self,
        item: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Listing>, PricingError>> + Send;
}
