/// 处理 Q64.64 固定点数的库
/// 用于 sqrt_price_math.rs 和 liquidity_amounts.rs

pub const Q64: u128 = (u64::MAX as u128) + 1; // 2^64
pub const RESOLUTION: u8 = 64;
