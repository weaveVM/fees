use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub mod ar_price_fetcher;
pub mod ar_price_fetcher_onchain;
pub mod util;
pub mod wvm_fee;

pub type WvmUpdatePriceCb = Box<dyn Fn(i64) -> Result<(), ()> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct UpdatePriceCb {
    pub cb: Arc<WvmUpdatePriceCb>,
}

impl Debug for UpdatePriceCb {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function pointer")
    }
}

/// This represents the value in dollars of a single WVM token.
pub const WVM_USD_PRICE: f64 = 12.5;

// mod test {
//     use crate::ar_price_fetcher::PriceContainer;

//     #[tokio::test]
//     async fn test_price_fetch() {
//         let price = PriceContainer::fetch_price().await.unwrap();
//         println!("price {}", price);
//         assert!(price > 0f64);
//     }
// }


