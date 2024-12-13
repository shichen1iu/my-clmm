use anchor_lang::prelude::*;

use crate::util::get_recent_epoch;

pub const OBSERVATION_NUM: usize = 100;
pub const OBSERVATION_SEED: &str = "observation";

///`ObservationState` 和 `Observation` 是 Raydium 的价格预言机（Oracle）组件
#[zero_copy(unsafe)]
#[repr(C, packed)]
#[derive(Default, Debug)]
pub struct Observation {
    /// 观察记录的区块时间戳
    pub block_timestamp: u32,
    /// 在持续时间内的 tick 累积值
    /// 用于计算时间加权平均价格(TWAP)
    pub tick_cumulative: i64,
    /// 为未来功能更新预留的填充空间
    pub padding: [u64; 4],
}

impl Observation {
    pub const LEN: usize = 4 + 8 + 8 * 4;
}

#[account(zero_copy(unsafe))]
#[repr(C, packed)]
#[cfg_attr(feature = "client", derive(Debug))]
pub struct ObservationState {
    /// 该观察状态是否已初始化
    pub initialized: bool,

    /// 最近更新的epoch
    pub recent_epoch: u64,

    /// observations 数组中最近更新的索引位置
    pub observation_index: u16,

    /// 属于哪个交易池
    pub pool_id: Pubkey,

    /// 观察记录数组，存储历史价格数据
    pub observations: [Observation; OBSERVATION_NUM],

    /// 为未来功能更新预留的填充空间
    pub padding: [u64; 4],
}

impl ObservationState {
    pub const LEN: usize = 8 + 1 + 8 + 2 + 32 + (Observation::LEN * OBSERVATION_NUM) + 8 * 4;

    pub fn initialize(&mut self, pool_id: Pubkey) -> Result<()> {
        self.initialized = false;
        self.recent_epoch = get_recent_epoch()?;
        self.observation_index = 0;
        self.pool_id = pool_id;
        self.observations = [Observation::default(); OBSERVATION_NUM];
        self.padding = [0u64; 4];
        Ok(())
    }
}
