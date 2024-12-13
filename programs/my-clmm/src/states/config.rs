use anchor_lang::prelude::*;
pub const AMM_CONFIG_SEED: &str = "amm_config";

///是用来计算费率的分母值，设置为 1,000,000（百万）。这意味着费率以百万分之一为单位。
pub const FEE_RATE_DENOMINATOR_VALUE: u32 = 1_000_000;

#[account]
#[derive(Default, Debug)]
pub struct AmmConfig {
    pub bump: u8,
    pub index: u16,
    pub owner: Pubkey,
    /// 协议费率:是从 trade_fee 中抽取的一部分作为协议收入,归 Raydium 协议所有
    /// 是 trade_fee 的一个百分比
    ///
    /// // 例如，假设一笔 1000 USDC 的交易：
    /// trade_fee_rate = 3000      // 0.3% 交易总费率
    /// protocol_fee_rate = 100000 // 10% 从交易费中抽取
    /// fund_fee_rate = 50000      // 5% 从交易费中抽取
    /// 计算过程：
    /// total_fee = 1000 * (3000/1_000_000) = 3 USDC        // 总交易费
    /// protocol_fee = 3 * (100000/1_000_000) = 0.3 USDC    // 给协议的费用
    /// fund_fee = 3 * (50000/1_000_000) = 0.1 //给基金的费用
    pub protocol_fee_rate: u32,
    /// 交易费率:这是给LP的
    pub trade_fee_rate: u32,
    /// tick spacing: 表示两个相邻tick之间的距离,由trade_fee_rate决定
    pub tick_spacing: u16,
    /// 基金费率:这是给基金的,基金可用于生态系统发展
    pub fund_fee_rate: u32,
    /// 基金所有者: 基金的地址
    pub fund_owner: Pubkey,
    /// 给将来账户添加字段预留的空间,因为sol的账户空间在创建时就分配固定大小
    /// 如果将来需要添加字段，则需要预留空间
    /// padding_u32预留 4 字节，可能用于添加新的 u32 类型字段
    pub padding_u32: u32,
    /// padding 预留 24 字节，可能用于添加新的 u64 类型字段
    pub padding: [u64; 3],
}

impl AmmConfig {
    pub const LEN: usize = 8 + 1 + 2 + 32 + 4 + 24;
}

/// 当创建或更新配置时发出的事件
#[event]
#[cfg_attr(feature = "client", derive(Debug))]
pub struct ConfigChangeEvent {
    pub index: u16,
    #[index]
    pub owner: Pubkey,
    pub protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
    pub tick_spacing: u16,
    pub fund_fee_rate: u32,
    pub fund_owner: Pubkey,
}
