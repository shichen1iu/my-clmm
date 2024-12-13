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
