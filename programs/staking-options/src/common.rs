use anchor_lang::prelude::*;

pub const SO_CONFIG_SEED: &[u8] = b"so-config";
pub const SO_VAULT_SEED: &[u8] = b"so-vault";
pub const SO_MINT_SEED: &[u8] = b"so-mint";

#[account]
pub struct State {
    // Identifier for this SO. This allows multiple projects to use the same
    // token, which is especially useful for downside SO.
    pub so_name: String,

    // Authority is required to sign all issuing and withdrawing. Should be a
    // PDA or owner of the project.
    pub authority: Pubkey,

    // Number of tokens available for SOs. Units are atoms of the base.
    pub options_available: u64,

    // Seconds since unix epoch for options to expire.
    pub option_expiration: u64,

    // Seconds since unix epoch for subscription period end.
    pub subscription_period_end: u64,

    // Number of decimals for the base token as well as SO.
    pub base_decimals: u8,

    // Number of decimals for the quote token.
    pub quote_decimals: u8,

    // Mint of the project token
    pub base_mint: Pubkey,

    // Mint of the quote token
    pub quote_mint: Pubkey,

    // The account that will receive payments on the options.
    pub quote_account: Pubkey,

    // Number of atoms of the base token to be traded per lot.
    pub lot_size: u64,

    pub state_bump: u8,
    pub vault_bump: u8,

    // Vector of all strikes for an SO. Limit 100. For monitoring only.
    // A strike is number of quote atoms per lot.
    pub strikes: Vec<u64>,

    // If present, this issue authority is allowed to issue in addition to the
    // authority above. This is useful in the case where a DAO is the one doing
    // the config, initStrike, withdraw, but a program is doing the issuing.
    pub issue_authority: Pubkey,
    // Padding of variable length.
}
