use anchor_lang::prelude::*;
const EXTENSION_TICKARRAY_BITMAP_SIZE: usize = 14;

///来扩展价格范围的数据结构。它扩展了池子中可以记录的价格范围（tick array）的数量
/// 假设一个 ETH/USDC 池子：
/// 主账户 tick_array_bitmap:
/// - 能记录价格范围 $1500-$2500
///
/// TickArrayBitmapExtension:
/// - positive_tick_array_bitmap: 记录更高价格范围 $2500-$10000
/// - negative_tick_array_bitmap: 记录更低价格范围 $500-$1500
///
/// 池子主账户的 tick_array_bitmap 空间有限
/// 通过这个扩展账户可以支持更大的价格波动范围
#[account(zero_copy(unsafe))]
#[repr(C, packed)]
#[derive(Debug)]
pub struct TickArrayBitmapExtension {
    pub pool_id: Pubkey,
    /// Packed initialized tick array state for start_tick_index is positive
    pub positive_tick_array_bitmap: [[u64; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE],
    /// Packed initialized tick array state for start_tick_index is negitive
    pub negative_tick_array_bitmap: [[u64; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE],
}

impl Default for TickArrayBitmapExtension {
    #[inline]
    fn default() -> TickArrayBitmapExtension {
        TickArrayBitmapExtension {
            pool_id: Pubkey::default(),
            positive_tick_array_bitmap: [[0; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE],
            negative_tick_array_bitmap: [[0; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE],
        }
    }
}

impl TickArrayBitmapExtension {
    pub const LEN: usize = 8 + 32 + 64 * EXTENSION_TICKARRAY_BITMAP_SIZE * 2;

    pub fn initialize(&mut self, pool_id: Pubkey) {
        self.pool_id = pool_id;
        self.positive_tick_array_bitmap = [[0; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE];
        self.negative_tick_array_bitmap = [[0; 8]; EXTENSION_TICKARRAY_BITMAP_SIZE];
    }
}
