use super::get_recent_epoch;
use crate::error::ErrorCode;
use crate::states::*;
use anchor_lang::{
    prelude::*,
    system_program::{create_account, CreateAccount},
};
use anchor_spl::token::{self, Token};
use anchor_spl::token_2022::{
    self,
    spl_token_2022::{
        self,
        extension::{
            metadata_pointer,
            transfer_fee::{TransferFeeConfig, MAX_FEE_BASIS_POINTS},
            BaseStateWithExtensions, ExtensionType, StateWithExtensions,
        },
    },
    Token2022,
};
use anchor_spl::token_interface::{initialize_mint2, InitializeMint2, Mint};
use std::collections::HashSet;

//四种白名单的mint:
const MINT_WHITELIST: [&'static str; 4] = [
    "HVbpJAQGNpkgBaYBZQBR1t7yFdvaYVp2vCQQfKKEN4tM", //Pax Dollar
    "Crn4x1Y2HUKko7ox2EZMT6N2t2ZyH7eKtwkBGVnhEq1g", // GMO JPY
    "FrBfWJ4qE5sCzKm3k3JaAtqZcXUh4LvJygDeketsrsH4", // Z.com USD
    "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo", //PayPal USD
];

/// 检查代币是否支持
pub fn is_supported_mint(mint_account: &InterfaceAccount<Mint>) -> Result<bool> {
    let mint_info = mint_account.to_account_info();
    // 如果是标准 SPL Token，直接返回支持
    if *mint_info.owner == Token::id() {
        return Ok(true);
    }
    // 如果mint的地址在白名单中，则返回支持
    let mint_whitelist: HashSet<&str> = MINT_WHITELIST.into_iter().collect();
    if mint_whitelist.contains(mint_account.key().to_string().as_str()) {
        return Ok(true);
    }
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;
    let extensions = mint.get_extension_types()?;
    // 遍历代币的所有扩展功能
    for e in extensions {
        // 如果发现任何一个扩展功能不在这5个允许的扩展功能中，就返回 false
        if e != ExtensionType::TransferFeeConfig
            && e != ExtensionType::MetadataPointer
            && e != ExtensionType::TokenMetadata
            && e != ExtensionType::InterestBearingConfig
            && e != ExtensionType::MintCloseAuthority
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// 计算 Token-2022 的转账反向手续费
pub fn get_transfer_inverse_fee(
    mint_account: Box<InterfaceAccount<Mint>>,
    post_fee_amount: u64,
) -> Result<u64> {
    let mint_info = mint_account.to_account_info();
    if *mint_info.owner == Token::id() {
        return Ok(0); // 如果是普通 SPL Token，没有手续费
    }
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;
    //提取代币的转账手续费配置 如果成功提取到 TransferFeeConfig，则进入 if let 块。
    let fee = if let Ok(transfer_fee_config) = mint.get_extension::<TransferFeeConfig>() {
        // 获取当前的 epoch，用于计算当前的手续费率
        let epoch = get_recent_epoch()?;
        // 从 transfer_fee_config 中获取当前 epoch 的手续费配置
        let transfer_fee = transfer_fee_config.get_epoch_fee(epoch);
        // 检查手续费率是否达到了最大值
        if u16::from(transfer_fee.transfer_fee_basis_points) == MAX_FEE_BASIS_POINTS {
            //如果手续费率达到了最大值，直接使用配置中的最大手续费
            u64::from(transfer_fee.maximum_fee)
        } else {
            // 否则，计算实际需要的手续费
            // 这里使用了 calculate_inverse_epoch_fee 方法，传入当前 epoch 和转账后的金额
            // 这个方法会根据当前的手续费率计算出需要额外支付的手续费，以确保接收方能收到 post_fee_amount
            transfer_fee_config
                .calculate_inverse_epoch_fee(epoch, post_fee_amount)
                .unwrap()
        }
    } else {
        0
    };
    Ok(fee)
}

/// 从资金提供者转账到pool_vault
pub fn transfer_from_user_to_pool_vault<'info>(
    signer: &Signer<'info>,
    from: &AccountInfo<'info>,
    to_vault: &AccountInfo<'info>,
    mint: Option<Box<InterfaceAccount<'info, Mint>>>,
    token_program: &AccountInfo<'info>,
    token_program_2022: Option<AccountInfo<'info>>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    let mut token_program_info = token_program.to_account_info();
    let from_token_info = from.to_account_info();
    match (mint, token_program_2022) {
        (Some(mint), Some(token_program_2022)) => {
            if from_token_info.owner == token_program_2022.key {
                token_program_info = token_program_2022.to_account_info()
            }
            token_2022::transfer_checked(
                CpiContext::new(
                    token_program_info,
                    token_2022::TransferChecked {
                        from: from_token_info,
                        to: to_vault.to_account_info(),
                        authority: signer.to_account_info(),
                        mint: mint.to_account_info(),
                    },
                ),
                amount,
                mint.decimals,
            )
        }
        _ => token::transfer(
            CpiContext::new(
                token_program_info,
                token::Transfer {
                    from: from_token_info,
                    to: to_vault.to_account_info(),
                    authority: signer.to_account_info(),
                },
            ),
            amount,
        ),
    }
}
