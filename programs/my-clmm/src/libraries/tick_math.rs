use crate::{error::ErrorCode, libraries::big_num::U128};

use anchor_lang::require;

/// 最小的 tick
pub const MIN_TICK: i32 = -443636;
/// 最大的 tick
pub const MAX_TICK: i32 = -MIN_TICK;

/// 从 #get_sqrt_price_at_tick 返回的最小值。等价于 get_sqrt_price_at_tick(MIN_TICK)
pub const MIN_SQRT_PRICE_X64: u128 = 4295048016;
/// 从 #get_sqrt_price_at_tick 返回的最大值。等价于 get_sqrt_price_at_tick(MAX_TICK)
pub const MAX_SQRT_PRICE_X64: u128 = 79226673521066979257578248091;

// Number 64, encoded as a U128
const NUM_64: U128 = U128([64, 0]);

const BIT_PRECISION: u32 = 16;

/// 计算 1.0001^(tick/2) 作为一个 U64.64 数字，表示两个资产的比率的平方根 (token_1/token_0)
///
/// 计算结果作为 U64.64
/// 每个魔法因子是 `2^64 / (1.0001^(2^(i - 1)))`，其中 i 在 `[0, 18)` 中。
///
/// 如果 |tick| > MAX_TICK，则抛出异常
///
/// # 参数
/// * `tick` - 价格 tick
///
///
pub fn get_sqrt_price_at_tick(tick: i32) -> Result<u128, anchor_lang::error::Error> {
    let abs_tick = tick.abs() as u32;
    require!(abs_tick <= MAX_TICK as u32, ErrorCode::TickUpperOverflow);

    // i = 0
    let mut ratio = if abs_tick & 0x1 != 0 {
        U128([0xfffcb933bd6fb800, 0])
    } else {
        // 2^64
        U128([0, 1])
    };
    // i = 1
    if abs_tick & 0x2 != 0 {
        ratio = (ratio * U128([0xfff97272373d4000, 0])) >> NUM_64
    };
    // i = 2
    if abs_tick & 0x4 != 0 {
        ratio = (ratio * U128([0xfff2e50f5f657000, 0])) >> NUM_64
    };
    // i = 3
    if abs_tick & 0x8 != 0 {
        ratio = (ratio * U128([0xffe5caca7e10f000, 0])) >> NUM_64
    };
    // i = 4
    if abs_tick & 0x10 != 0 {
        ratio = (ratio * U128([0xffcb9843d60f7000, 0])) >> NUM_64
    };
    // i = 5
    if abs_tick & 0x20 != 0 {
        ratio = (ratio * U128([0xff973b41fa98e800, 0])) >> NUM_64
    };
    // i = 6
    if abs_tick & 0x40 != 0 {
        ratio = (ratio * U128([0xff2ea16466c9b000, 0])) >> NUM_64
    };
    // i = 7
    if abs_tick & 0x80 != 0 {
        ratio = (ratio * U128([0xfe5dee046a9a3800, 0])) >> NUM_64
    };
    // i = 8
    if abs_tick & 0x100 != 0 {
        ratio = (ratio * U128([0xfcbe86c7900bb000, 0])) >> NUM_64
    };
    // i = 9
    if abs_tick & 0x200 != 0 {
        ratio = (ratio * U128([0xf987a7253ac65800, 0])) >> NUM_64
    };
    // i = 10
    if abs_tick & 0x400 != 0 {
        ratio = (ratio * U128([0xf3392b0822bb6000, 0])) >> NUM_64
    };
    // i = 11
    if abs_tick & 0x800 != 0 {
        ratio = (ratio * U128([0xe7159475a2caf000, 0])) >> NUM_64
    };
    // i = 12
    if abs_tick & 0x1000 != 0 {
        ratio = (ratio * U128([0xd097f3bdfd2f2000, 0])) >> NUM_64
    };
    // i = 13
    if abs_tick & 0x2000 != 0 {
        ratio = (ratio * U128([0xa9f746462d9f8000, 0])) >> NUM_64
    };
    // i = 14
    if abs_tick & 0x4000 != 0 {
        ratio = (ratio * U128([0x70d869a156f31c00, 0])) >> NUM_64
    };
    // i = 15
    if abs_tick & 0x8000 != 0 {
        ratio = (ratio * U128([0x31be135f97ed3200, 0])) >> NUM_64
    };
    // i = 16
    if abs_tick & 0x10000 != 0 {
        ratio = (ratio * U128([0x9aa508b5b85a500, 0])) >> NUM_64
    };
    // i = 17
    if abs_tick & 0x20000 != 0 {
        ratio = (ratio * U128([0x5d6af8dedc582c, 0])) >> NUM_64
    };
    // i = 18
    if abs_tick & 0x40000 != 0 {
        ratio = (ratio * U128([0x2216e584f5fa, 0])) >> NUM_64
    }

    // Divide to obtain 1.0001^(2^(i - 1)) * 2^32 in numerator
    if tick > 0 {
        ratio = U128::MAX / ratio;
    }

    Ok(ratio.as_u128())
}

/// 公式: i=log√p(i)/log(√1.0001)
///
/// 输入一个 sqrt price (√P)，返回对应的 tick 值(这个函数是 get_sqrt_price_at_tick 的反函数)
/// 假设一开始的sqrt_price = 23.423 则sqrt_price_x64 = 23.423 * 2^64
pub fn get_tick_at_sqrt_price(sqrt_price_x64: u128) -> Result<i32, anchor_lang::error::Error> {
    // 第二个不等式必须是 <，因为价格永远无法达到最大 tick 的价格
    // 验证价格在合法范围内
    require!(
        sqrt_price_x64 >= MIN_SQRT_PRICE_X64 && sqrt_price_x64 < MAX_SQRT_PRICE_X64,
        ErrorCode::SqrtPriceX64
    );

    //此时leading_zeros = 59,msb=68
    let msb: u32 = 128 - sqrt_price_x64.leading_zeros() - 1;
    //此时log2p_integer_x32 = (68-64) * 2^32 =4 * 2^32
    let log2p_integer_x32 = (msb as i128 - 64) << 32;

    let mut bit: i128 = 0x8000_0000_0000_0000i128; //十六进制,在二进制中表示第一个数为1,其他权威0
    let mut precision = 0;
    let mut log2p_fraction_x64 = 0;

    //这段代码是在对输入的价格(sqrt_price_x64)进行标准化处理(大约在1到2之间）
    let mut r = if msb >= 64 {
        //此时msb=68 >=64
        //所以 sqrt_price_x64 右移 5 个位置
        //这里将sqrt_price_x64 右移 5 个位置的原因是:r *= r 的操作可能会导致数值溢出
        //此时的r = 0.732 * 2^64
        sqrt_price_x64 >> (msb - 63)
    } else {
        // 如果最高位在64位以下，左移使数字变大,使它同样落在0~1之间
        sqrt_price_x64 << (63 - msb)
    };

    while bit > 0 && precision < BIT_PRECISION {
        //第一次循环 r=0.535824×2^128
        r *= r;

        //r >> 127 是在检查 r 的值是否大于或等于 2^127。
        //如果 r 大于或等于 2^127，那么 r >> 127 的结果将是 1，表示 r 超过了 2
        //此时is_r_more_than_two = 1 r大于2^127
        let is_r_more_than_two = r >> 127 as u32;

        //如果 is_r_more_than_two 为 1，表示 r 当前的值大于 2^127，因此需要右移 64 位 (63 + 1)
        //如果 is_r_more_than_two 为 0，表示 r 的值小于 2^127，因此右移 63 位
        //r = 0.535824×2^64
        //is_r_more_than_two = 1
        r >>= 63 + is_r_more_than_two;

        log2p_fraction_x64 += bit * is_r_more_than_two as i128;

        bit >>= 1;

        precision += 1;
    }
    let log2p_fraction_x32 = log2p_fraction_x64 >> 32;
    let log2p_x32 = log2p_integer_x32 + log2p_fraction_x32;

    // 14 位细化给出的误差范围为 2^-14 / log2 (√1.0001) = 0.8461 < 1
    // 由于 tick 是小数，误差小于 1 是可以接受的

    // 基数变化规则: 乘以 2^16 / log2 (√1.0001)
    let log_sqrt_10001_x64 = log2p_x32 * 59543866431248i128;

    // tick - 0.01
    let tick_low = ((log_sqrt_10001_x64 - 184467440737095516i128) >> 64) as i32;

    // tick + (2^-14 / log2(√1.001)) + 0.01
    let tick_high = ((log_sqrt_10001_x64 + 15793534762490258745i128) >> 64) as i32;

    Ok(if tick_low == tick_high {
        tick_low
    } else if get_sqrt_price_at_tick(tick_high).unwrap() <= sqrt_price_x64 {
        tick_high
    } else {
        tick_low
    })
}
