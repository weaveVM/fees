use crate::ar_price_fetcher::{ArPriceFetcher, AR_FETCH_INTERVAL_SECONDS};
use crate::util::raw_calculate_wvm_base_storage_fee;
use crate::{UpdatePriceCb, WvmUpdatePriceCb, WVM_USD_PRICE};
use std::sync::Arc;

pub struct WvmFee {
    ar_price_fetcher: Arc<ArPriceFetcher>,
    base_fee_cb: Option<UpdatePriceCb>,
}

pub struct WvmFeeManager {
    wvm_fee: Arc<WvmFee>,
}

impl WvmFeeManager {
    pub fn new(wvm_fee: Arc<WvmFee>) -> Self {
        Self { wvm_fee }
    }

    pub fn init(&self) {
        let curr = self.wvm_fee.clone();
        let base_fee_cb = curr.clone().base_fee_cb.as_ref().map(|i| i.cb.clone());

        if let Some(cb) = base_fee_cb {
            tokio::spawn(async move {
                let cb = cb.clone();
                let forever = tokio::task::spawn(async move {
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                        AR_FETCH_INTERVAL_SECONDS + 10,
                    ));

                    loop {
                        let cb = cb.clone();
                        let wvm_fee = curr.clone();

                        interval.tick().await;
                        let fee = wvm_fee.calculate_wvm_base_storage_fee().await;
                        let fee_to_gas = fee * 1_000_000_000f64;
                        let _ = cb(fee_to_gas as i64);
                    }
                });

                forever.await.unwrap();
            });
        }
    }
}

impl WvmFee {
    pub fn new(base_fee_cb: Option<WvmUpdatePriceCb>) -> Self {
        Self {
            ar_price_fetcher: Arc::new(ArPriceFetcher::new()),
            base_fee_cb: match base_fee_cb {
                None => None,
                Some(func) => Some(UpdatePriceCb { cb: Arc::new(func) }),
            },
        }
    }

    pub fn init(&self) {
        self.ar_price_fetcher.init();
    }

    pub async fn wvm_usd_price(&self) -> f64 {
        WVM_USD_PRICE
    }

    pub async fn arweave_base_usd_fee(&self) -> f64 {
        let ar_price_reader = self.ar_price_fetcher.current_price.read().unwrap();
        let init = ar_price_reader.init;
        if init {
            let base_fee_in_winston = ar_price_reader.base_price_in_winston;
            let one_ar_in_dollars = ar_price_reader.price;
            let one_ar_in_winston = 1000000000000i64;

            // one_ar_in_winston -> one_ar_in_dollars
            // base_fee_in_winston -> x

            (base_fee_in_winston as f64 * one_ar_in_dollars) / one_ar_in_winston as f64
        } else {
            0.004 // Default in case it hasn't been init
        }
    }

    pub(crate) async fn raw_calculate_lowest_possible_gas_price(
        &self,
        base_storage_fee: f64,
        block_gas_limit: u64,
    ) -> f64 {
        (base_storage_fee * 1e21 / block_gas_limit as f64) / 1e9
    }

    pub async fn calculate_wvm_base_storage_fee(&self) -> f64 {
        let ar_base_fee = self.arweave_base_usd_fee().await;
        let wvm_usd_price = self.wvm_usd_price().await;

        raw_calculate_wvm_base_storage_fee(ar_base_fee, wvm_usd_price)
    }

    pub async fn calculate_wvm_base_storage_fee_gwei(&self) -> f64 {
        self.calculate_wvm_base_storage_fee().await * 1_000_000_000f64
    }
}

#[cfg(test)]
mod tests {
    use crate::util::raw_calculate_wvm_base_storage_fee;
    use crate::wvm_fee::{WvmFee, WvmFeeManager};
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    #[tokio::test]
    pub async fn test_wvm_base_fee() {
        let base_fee = raw_calculate_wvm_base_storage_fee(0.004, 12.5);
        assert_eq!(base_fee, 0.00032);
    }

    #[tokio::test]
    pub async fn test_wvm_lowest_possible_gas_price() {
        let base_fee = raw_calculate_wvm_base_storage_fee(0.004, 12.5);

        let lowest_possible_gas_price = WvmFee::new(None)
            .raw_calculate_lowest_possible_gas_price(base_fee, 300_000_000)
            .await;

        assert_eq!(lowest_possible_gas_price, 1.0666666666666667);

        let lowest_possible_gas_price = WvmFee::new(None)
            .raw_calculate_lowest_possible_gas_price(base_fee, 500_000_000)
            .await;

        assert_eq!(lowest_possible_gas_price, 0.64);
    }

    #[tokio::test]
    pub async fn test_wvm_ar_fetcher() {
        let wvm_fee = WvmFee::new(None);
        wvm_fee.init();
        tokio::time::sleep(Duration::from_secs(10)).await;
        let price_container = wvm_fee.ar_price_fetcher.current_price.read().unwrap();
        assert!(price_container.init);
        assert!(price_container.price > 1f64);
        assert!(price_container.base_price_in_winston > 10000);
    }

    #[tokio::test]
    pub async fn test_ar_base_fee_usd() {
        let wvm_fee = WvmFee::new(None);
        let ar_fee = wvm_fee.arweave_base_usd_fee().await;
        assert!(ar_fee >= 0.004);
        assert!(ar_fee <= 0.008);
    }

    #[tokio::test]
    pub async fn test_ar_base_fee_usd_atomic_update() {
        let mut atomic = Arc::new(RwLock::new(0 as i64));
        let at_ref = atomic.clone();
        let wvm_fee = WvmFee::new(Some(Box::new(move |price| {
            let at_ref = atomic.clone();
            let mut w = at_ref.write().unwrap();
            *w += price;
            Ok(())
        })));
        let fee_manager = WvmFeeManager::new(Arc::new(wvm_fee));
        fee_manager.init();
        tokio::time::sleep(Duration::from_secs(10)).await;
        let x = at_ref.read().unwrap();
        assert!((&x as &i64) > &300_000);
        assert!((&x as &i64) < &400_000);
    }
}
