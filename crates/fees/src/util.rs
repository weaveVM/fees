pub fn raw_calculate_wvm_base_storage_fee(ar_base_fee: f64, wvm_usd_price: f64) -> f64 {
    // 1 = 1 token
    (ar_base_fee * 1f64) / wvm_usd_price
}
