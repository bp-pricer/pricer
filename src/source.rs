use tf2_price::{Currencies, Currency};
use tf2_sku::SKU;

#[derive(Debug)]
pub enum PricingError {
    ServerError,
    InternalError
}

pub trait PriceSource {
    fn get_lowest_seller(&self, item: SKU) -> impl std::future::Future<Output = Result<Currencies, PricingError>> + Send;
}