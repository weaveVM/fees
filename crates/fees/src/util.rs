pub fn raw_calculate_wvm_base_storage_fee(ar_base_fee: f64, wvm_usd_price: f64) -> f64 {
    // 1 = 1 token
    (ar_base_fee * 1f64) / wvm_usd_price
}

pub fn raw_calculate_lowest_possible_gas_price(base_storage_fee: f64, block_gas_limit: u64) -> f64 {
    (base_storage_fee * 1e21 / block_gas_limit as f64) / 1e9
}
