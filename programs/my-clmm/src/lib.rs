use anchor_lang::prelude::*;
mod error;
mod instructions;
mod libraries;
mod states;
mod util;
use core as core_;
use instructions::*;
use states::*;
declare_id!("3EJpuBuuaKJH8B6AaFyVinPszYFkYrkiR7FDKDpAzMWh");

pub mod admin {
    use anchor_lang::prelude::declare_id;
    declare_id!("9qMknujRc8eqBZ6gSrypjYyzNNpwiwASxocKdAfg563C");
}

#[program]
pub mod my_clmm {
    use super::*;

    // AMM 协议的配置，包含交易费用和协议费用
    /// # 参数
    ///
    /// * `ctx` - 指令所需的账户
    /// * `index` - AMM 配置的索引，可能存在多个配置
    /// * `tick_spacing` - 与配置绑定的 tick 间距，创建后不能更改
    /// * `trade_fee_rate` - 交易费率，可以更改
    /// * `protocol_fee_rate` - 协议费率，是交易费用中的一部分
    /// * `fund_fee_rate` - 基金费率，是交易费用中的一部分
    pub fn create_amm_config(
        ctx: Context<CreateAmmConfig>,
        index: u16,
        tick_spacing: u16,
        trade_fee_rate: u32,
        protocol_fee_rate: u32,
        fund_fee_rate: u32,
    ) -> Result<()> {
        assert!(trade_fee_rate < FEE_RATE_DENOMINATOR_VALUE);
        assert!(protocol_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        assert!(fund_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        assert!(fund_fee_rate + protocol_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        instructions::create_amm_config(
            ctx,
            index,
            tick_spacing,
            trade_fee_rate,
            protocol_fee_rate,
            fund_fee_rate,
        )
    }

    /// 2.更新 AMM 配置的所有者
    /// 必须由当前所有者或管理员调用
    ///
    /// # 参数
    ///
    /// * `ctx` - 账户上下文
    /// * `trade_fee_rate` - AMM 配置的新交易费率，当 `param` 为 0 时设置
    /// * `protocol_fee_rate` - AMM 配置的新协议费率，当 `param` 为 1 时设置
    /// * `fund_fee_rate` - AMM 配置的新基金费率，当 `param` 为 2 时设置
    /// * `new_owner` - 配置的新所有者，当 `param` 为 3 时设置
    /// * `new_fund_owner` - 配置的新基金所有者，当 `param` 为 4 时设置
    /// * `param` - 取值可以是 0 | 1 | 2 | 3 | 4，其他值将报错
    pub fn update_amm_config(ctx: Context<UpdateAmmConfig>, param: u8, value: u32) -> Result<()> {
        instructions::update_amm_config(ctx, param, value)
    }

    /// 3.为给定的代币对和初始价格创建交易池
    ///
    /// # 参数
    ///
    /// * `ctx` - 账户上下文
    /// * `sqrt_price_x64` - 交易池的初始价格平方根（token1数量/token0数量），以 Q64.64 定点数格式表示
    pub fn create_pool(
        ctx: Context<CreatePool>,
        sqrt_price_x64: u128,
        open_time: u64,
    ) -> Result<()> {
        instructions::create_pool(ctx, sqrt_price_x64, open_time)
    }
}
