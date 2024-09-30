use crate::ar_price_fetcher::ArPriceFetcher;
use crate::WVM_USD_PRICE;
use std::sync::Arc;

pub struct WvmFee {
    ar_price_fetcher: Arc<ArPriceFetcher>,
}

impl WvmFee {
    pub fn new() -> Self {
        Self {
            ar_price_fetcher: Arc::new(ArPriceFetcher::new()),
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

    pub(crate) async fn raw_calculate_wvm_base_storage_fee(
        &self,
        ar_base_fee: f64,
        wvm_usd_price: f64,
    ) -> f64 {
        // 1 = 1 token
        (ar_base_fee * 1f64) / wvm_usd_price
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

        self.raw_calculate_wvm_base_storage_fee(ar_base_fee, wvm_usd_price)
            .await
    }

    pub async fn calculate_wvm_base_storage_fee_gwei(&self) -> f64 {
        self.calculate_wvm_base_storage_fee().await * 1_000_000_000f64
    }
}

#[cfg(test)]
mod tests {
    use crate::wvm_fee::WvmFee;
    use std::time::Duration;

    #[tokio::test]
    pub async fn test_wvm_base_fee() {
        let base_fee = WvmFee::new()
            .raw_calculate_wvm_base_storage_fee(0.004, 12.5)
            .await;
        assert_eq!(base_fee, 0.00032);
    }

    #[tokio::test]
    pub async fn test_wvm_lowest_possible_gas_price() {
        let base_fee = WvmFee::new()
            .raw_calculate_wvm_base_storage_fee(0.004, 12.5)
            .await;

        let lowest_possible_gas_price = WvmFee::new()
            .raw_calculate_lowest_possible_gas_price(base_fee, 300_000_000)
            .await;

        assert_eq!(lowest_possible_gas_price, 1.0666666666666667);

        let lowest_possible_gas_price = WvmFee::new()
            .raw_calculate_lowest_possible_gas_price(base_fee, 500_000_000)
            .await;

        assert_eq!(lowest_possible_gas_price, 0.64);
    }

    #[tokio::test]
    pub async fn test_wvm_ar_fetcher() {
        let wvm_fee = WvmFee::new();
        wvm_fee.init();
        tokio::time::sleep(Duration::from_secs(10)).await;
        let price_container = wvm_fee.ar_price_fetcher.current_price.read().unwrap();
        assert!(price_container.init);
        assert!(price_container.price > 1f64);
        assert!(price_container.base_price_in_winston > 10000);
    }

    #[tokio::test]
    pub async fn test_ar_base_fee_usd() {
        let wvm_fee = WvmFee::new();
        let ar_fee = wvm_fee.arweave_base_usd_fee().await;
        assert!(ar_fee >= 0.004);
        assert!(ar_fee <= 0.008);
    }
}
