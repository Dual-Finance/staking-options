use anchor_lang::prelude::*;

pub const SO_CONFIG_SEED: &[u8] = b"so-config";
pub const SO_VAULT_SEED: &[u8] = b"so-vault";
pub const SO_MINT_SEED: &[u8] = b"so-mint";
pub const SO_USDC_SEED: &[u8] = b"so-usdc";

pub const NUM_ATOMS_PER_USDC: u64 = 1_000_000;

#[account]
pub struct State {
    // Identifier for this SO. This allows multiple projects to use the same
    // token, which is especially useful for downside SO.
    pub so_name: String,

    // For identifying the SO within the same SO name.
    pub period_num: u64,

    // Authority is required to sign all issuing and withdrawing. Should be a
    // PDA or owner of the project.
    pub authority: Pubkey,

    // Number of tokens available for SOs
    pub options_available: u64,

    // Seconds since unix epoch for options to expire.
    pub option_expiration: u64,

    // Seconds since unix epoch for subscription period end.
    pub subscription_period_end: u64,

    // Number of decimals for the base token as well as SO.
    pub base_decimals: u8,

    // Number of decimals for the quote token.
    pub quote_decimals: u8,

    // Mint of the reward token
    pub base_token_mint: Pubkey,

    // The account that will receive payments on the options.
    pub quote_account: Pubkey,

    // Vector of all strikes for an SO. Limit 100. For monitoring only. A strike
    // is number of quote atoms per full base token.
    pub strikes: Vec<u64>,
}
