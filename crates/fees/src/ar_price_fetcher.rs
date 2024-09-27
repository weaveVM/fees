use serde_json::Value;
use std::sync::{Arc, RwLock};

pub const AR_FETCH_INTERVAL_SECONDS: u64 = 120;

pub struct PriceContainer {
    pub price: f64,
    pub base_price_in_winston: i64,
    pub init: bool,
}

impl PriceContainer {
    pub async fn fetch_price() -> Option<f64> {
        let req =
            reqwest::get("https://api.redstone.finance/prices?symbol=AR&provider=redstone&limit=1")
                .await
                .ok()?;
        let json = req.json::<Value>().await.ok()?;
        let item = json.as_array()?;
        if !item.is_empty() {
            let ar_item = item.get(0)?;
            let price = ar_item.get("value")?.as_f64()?;
            Some(price)
        } else {
            None
        }
    }

    pub async fn fetch_base_price_in_winston() -> Option<i64> {
        let req = reqwest::get("https://arweave.net/price/1").await.ok()?;
        let as_text = req.text().await.ok()?;
        Some(as_text.as_str().parse::<i64>().ok()?)
    }

    pub fn update(&mut self, new_price: f64) {
        if !self.init {
            self.init = true;
        }
        self.price = new_price;
    }

    pub fn update_base_winston(&mut self, new_winston_base_fee: i64) {
        self.base_price_in_winston = new_winston_base_fee;
    }
}

pub struct ArPriceFetcher {
    pub current_price: Arc<RwLock<PriceContainer>>,
}

impl ArPriceFetcher {
    pub fn new() -> Self {
        Self {
            current_price: Arc::new(RwLock::new(PriceContainer {
                price: 0.004,
                base_price_in_winston: 185021129i64,
                init: false,
            })),
        }
    }

    pub fn init(&self) {
        let cur_price = self.current_price.clone();
        tokio::spawn(async move {
            let cur_price = cur_price.clone();

            let forever = tokio::task::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                    AR_FETCH_INTERVAL_SECONDS,
                ));

                loop {
                    let cur_price = cur_price.clone();
                    interval.tick().await;
                    let get_price = PriceContainer::fetch_price().await;
                    let base_price_per_winston =
                        PriceContainer::fetch_base_price_in_winston().await;

                    {
                        let mut data = cur_price.write().unwrap();
                        if let Some(price) = get_price {
                            data.update(price);
                        }
                    }

                    {
                        let mut data = cur_price.write().unwrap();
                        if let Some(base_winston_price) = base_price_per_winston {
                            data.update_base_winston(base_winston_price);
                        }
                    }
                }
            });

            forever.await.unwrap();
        });
    }
}
