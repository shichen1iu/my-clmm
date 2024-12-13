use anchor_lang::prelude::*;

pub const OPERATION_SEED: &str = "operation";
pub const OPERATION_SIZE_USIZE: usize = 10;
pub const WHITE_MINT_SIZE_USIZE: usize = 100;

/// Holds the current owner of the factory
#[account(zero_copy(unsafe))]
#[repr(C, packed)]
#[derive(Debug)]
pub struct OperationState {
    /// Bump to identify PDA
    pub bump: u8,
    /// Address of the operation owner
    pub operation_owners: [Pubkey; OPERATION_SIZE_USIZE],
    /// The mint address of whitelist to emmit reward
    pub whitelist_mints: [Pubkey; WHITE_MINT_SIZE_USIZE],
}

impl OperationState {
    pub const LEN: usize = 8 + 1 + 32 * OPERATION_SIZE_USIZE + 32 * WHITE_MINT_SIZE_USIZE;
    pub fn initialize(&mut self, bump: u8) {
        self.bump = bump;
        self.operation_owners = [Pubkey::default(); OPERATION_SIZE_USIZE];
        self.whitelist_mints = [Pubkey::default(); WHITE_MINT_SIZE_USIZE];
    }
}
