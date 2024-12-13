use anchor_lang::prelude::*;

use crate::util::get_recent_epoch;

use super::AmmConfig;
use anchor_spl::token_interface::Mint;
pub const REWARD_NUM: usize = 3;

pub const POOL_SEED: &str = "pool";
pub const POOL_VAULT_SEED: &str = "pool_vault";
pub const POOL_REWARD_VAULT_SEED: &str = "pool_reward_vault";
pub const POOL_TICK_ARRAY_BITMAP_SEED: &str = "pool_tick_array_bitmap_extension";

/// PDA of `[POOL_SEED, config, token_mint_0, token_mint_1]`
#[account(zero_copy(unsafe))]
#[repr(C, packed)]
#[derive(Default, Debug)]
pub struct PoolState {
    pub bump: [u8; 1],
    // 归属的pool_config
    pub amm_config: Pubkey,
    // Pool creator
    pub owner: Pubkey,

    /// Token pair of the pool, where token_mint_0 address < token_mint_1 address
    /// 强制要求mint_0_address < mint_1_address 可以保证不同时出现eth/btc btc/eth 两种池子
    pub token_mint_0: Pubkey,
    pub token_mint_1: Pubkey,

    /// Token pair vault
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,

    ///预言机account的地址
    pub observation_key: Pubkey,

    /// mint0 and mint1 decimals
    pub mint_decimals_0: u8,
    pub mint_decimals_1: u8,

    pub tick_spacing: u16,
    /// 整个池子当前价格范围内的总流动性，而不是某个单独 position 的流动性
    pub liquidity: u128,
    /// 当前池子的价格且用Q64.64表示  sqrt(token_1/token_0) Q64.64 value
    pub sqrt_price_x64: u128,
    /// 前池子价格所在的 tick 位置, i.e. according to the last tick transition that was run.
    pub tick_current: i32,

    ///为将来升级保留的padding
    pub padding3: u16,
    pub padding4: u16,

    /// token0和token1在池子中每单位流动性累积的手续费,且这是一个Q64.64
    pub fee_growth_global_0_x64: u128,
    pub fee_growth_global_1_x64: u128,

    /// The amounts of token_0 and token_1 that are owed to the protocol.
    /// 从交易手续费中抽取的一部分，作为协议收入
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,

    /// The amounts in and out of swap token_0 and token_1
    pub swap_in_amount_token_0: u128,
    pub swap_out_amount_token_1: u128,
    pub swap_in_amount_token_1: u128,
    pub swap_out_amount_token_0: u128,

    /// 从右往左的每个bit都代表一种池子的状态
    /// bit0, 1: 禁止添加positon和流动性, 0: normal
    /// bit1, 1: 禁止减少, 0: normal
    /// bit2, 1: 禁止LP收取他们的交易手续费, 0: normal
    /// bit3, 1: 禁止LP收取流动性挖矿奖励(协议方主动提供的额外代币奖励), 0: normal
    /// bit4, 1: 禁止 swap, 0: normal
    pub status: u8,
    /// Leave blank for future use
    pub padding: [u8; 7],

    /// Raydium 的每个池子支持 3 种奖励代币
    /// 三种奖励代币的特殊规则：
    /// 一个LP最多可以获得3种代币奖励,具体由创建的人决定
    /// 第一个奖励槽位 (index = 0):没有特殊限制,可以是任何代币
    /// 第二个奖励槽位 (index = 1):
    /// 如果池子的两个代币都还没被用作奖励，那么这个奖励必须是:
    /// 池子中的 token_0，或 token_1 或是 白名单种的token
    /// 第三个奖励槽位 (index = 2): 需要特殊权限,只能由管理员或经过验证的操作所有者设置
    pub reward_infos: [RewardInfo; REWARD_NUM],

    ///  tick array 被初始化的位图数据结构
    /// 看笔记 序号4
    pub tick_array_bitmap: [u64; 16],

    /// 总手续费（未包含协议费和基金费）
    pub total_fees_token_0: u64,
    /// 已领取的手续费
    pub total_fees_claimed_token_0: u64,
    pub total_fees_token_1: u64,
    pub total_fees_claimed_token_1: u64,

    /// 基金费
    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,

    // 允许swap的开始时间时间戳
    pub open_time: u64,
    // 最近一次更新epoch
    pub recent_epoch: u64,

    // 为将来升级保留的padding
    pub padding1: [u64; 24],
    pub padding2: [u64; 32],
}
impl PoolState {
    pub const LEN: usize = 8
        + 1
        + 32 * 7
        + 1
        + 1
        + 2
        + 16
        + 16
        + 4
        + 2
        + 2
        + 16
        + 16
        + 8
        + 8
        + 16
        + 16
        + 16
        + 16
        + 8
        + RewardInfo::LEN * REWARD_NUM
        + 8 * 16
        + 512;

    pub fn initialize(
        &mut self,
        bump: u8,
        sqrt_price_x64: u128,
        open_time: u64,
        tick: i32,
        pool_creator: Pubkey,
        token_vault_0: Pubkey,
        token_vault_1: Pubkey,
        amm_config: &Account<AmmConfig>,
        token_mint_0: &InterfaceAccount<Mint>,
        token_mint_1: &InterfaceAccount<Mint>,
        observation_state_key: Pubkey,
    ) -> Result<()> {
        self.bump = [bump];
        self.amm_config = amm_config.key();
        self.owner = pool_creator.key();
        self.token_mint_0 = token_mint_0.key();
        self.token_mint_1 = token_mint_1.key();
        self.mint_decimals_0 = token_mint_0.decimals;
        self.mint_decimals_1 = token_mint_1.decimals;
        self.token_vault_0 = token_vault_0;
        self.token_vault_1 = token_vault_1;
        self.tick_spacing = amm_config.tick_spacing;
        self.liquidity = 0;
        self.sqrt_price_x64 = sqrt_price_x64;
        self.tick_current = tick;
        self.padding3 = 0;
        self.padding4 = 0;
        self.reward_infos = [RewardInfo::new(pool_creator); REWARD_NUM];
        self.fee_growth_global_0_x64 = 0;
        self.fee_growth_global_1_x64 = 0;
        self.protocol_fees_token_0 = 0;
        self.protocol_fees_token_1 = 0;
        self.swap_in_amount_token_0 = 0;
        self.swap_out_amount_token_1 = 0;
        self.swap_in_amount_token_1 = 0;
        self.swap_out_amount_token_0 = 0;
        self.status = 0;
        self.padding = [0; 7];
        self.tick_array_bitmap = [0; 16];
        self.total_fees_token_0 = 0;
        self.total_fees_claimed_token_0 = 0;
        self.total_fees_token_1 = 0;
        self.total_fees_claimed_token_1 = 0;
        self.fund_fees_token_0 = 0;
        self.fund_fees_token_1 = 0;
        self.open_time = open_time;
        self.recent_epoch = get_recent_epoch()?;
        self.padding1 = [0; 24];
        self.padding2 = [0; 32];
        self.observation_key = observation_state_key;

        Ok(())
    }
}

/// 奖励状态与下面的u8相对应
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, Debug, PartialEq)]
/// State of reward
pub enum RewardState {
    /// Reward not initialized 0
    Uninitialized,
    /// Reward initialized, but reward time is not start 1
    Initialized,
    /// Reward in progress 2
    Opening,
    /// Reward end, reward time expire or 3
    Ended,
}

#[zero_copy(unsafe)]
#[repr(C, packed)]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct RewardInfo {
    /// 奖励状态只有四种情况,因此u8绰绰有余
    pub reward_state: u8,
    /// Reward open time
    pub open_time: u64,
    /// Reward end time
    pub end_time: u64,
    /// Reward last update time
    pub last_update_time: u64,
    /// Q64.64 number indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// The total amount of reward emissioned
    pub reward_total_emissioned: u64,
    /// The total amount of claimed reward
    pub reward_claimed: u64,
    /// Reward token mint.
    pub token_mint: Pubkey,
    /// Reward vault token account.
    pub token_vault: Pubkey,
    /// The owner that has permission to set reward param
    pub authority: Pubkey,
    ///  Q64.64 定点数格式表示的数值，用来追踪每单位流动性自奖励开始以来赚取的总代币数量
    pub reward_growth_global_x64: u128,
}

impl RewardInfo {
    pub const LEN: usize = 1 + 8 + 8 + 8 + 16 + 8 + 8 + 32 + 32 + 32 + 16;

    /// Creates a new RewardInfo
    pub fn new(authority: Pubkey) -> Self {
        Self {
            authority,
            ..Default::default()
        }
    }
}

/// Emitted when a pool is created and initialized with a starting price
///
#[event]
#[cfg_attr(feature = "client", derive(Debug))]
pub struct PoolCreatedEvent {
    /// The first token of the pool by address sort order
    #[index]
    pub token_mint_0: Pubkey,

    /// The second token of the pool by address sort order
    #[index]
    pub token_mint_1: Pubkey,

    /// The minimum number of ticks between initialized ticks
    pub tick_spacing: u16,

    /// The address of the created pool
    pub pool_state: Pubkey,

    /// The initial sqrt price of the pool, as a Q64.64
    pub sqrt_price_x64: u128,

    /// The initial tick of the pool, i.e. log base 1.0001 of the starting price of the pool
    pub tick: i32,

    /// Vault of token_0
    pub token_vault_0: Pubkey,
    /// Vault of token_1
    pub token_vault_1: Pubkey,
}
