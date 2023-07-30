use anchor_lang::prelude::*;

pub const SO_CONFIG_SEED: &[u8] = b"so-config";
pub const SO_VAULT_SEED: &[u8] = b"so-vault";
pub const SO_REVERSE_VAULT_SEED: &[u8] = b"so-reverse-vault";
pub const SO_MINT_SEED: &[u8] = b"so-mint";
pub const SO_REVERSE_MINT_SEED: &[u8] = b"so-reverse-mint";

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

    // Bump for where the quote tokens are saved in a reversible staking option.
    pub quote_vault_bump: u8,

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
}

pub const DUAL_DAO_ADDRESS: &str = "7Z36Efbt7a4nLiV7s5bY7J2e4TJ6V9JEKGccsy2od2bE";
const DUAL_RISK_MANAGER: &str = "CkcJx7Uwgxck5zm3DqUp2N1ikkkoPn2wA8zf7oS4tFSZ";

const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
const DAIPO: &str = "EjmyN6qEC1Tf1JxiG1ae7UTJhUxSwk1TCWNWqxWV4J6o";
const USDH: &str = "USDH1SM1ojwWUga67PGrgFWUHibbjqMvuMaDkRJTgkX";
const CHAI: &str = "3jsFX1tx2Z8ewmamiwSU851GzyzM2DJMq7KWW5DM8Py3";

pub fn is_fee_exempt(user_quote_account_owner: Pubkey) -> bool {
    // Do not charge fee when the DUAL DAO is exercising since that is the recipient of the fee.
    if user_quote_account_owner.to_string() == DUAL_DAO_ADDRESS {
        return true;
    }
    // Do not charge fee when the DUAL DAO is exercising since that is the recipient of the fee.
    if user_quote_account_owner.to_string() == DUAL_RISK_MANAGER {
        return true;
    }

    return false;
}

pub fn is_reduced_fee(base_mint: Pubkey, quote_mint: Pubkey) -> bool {
    let is_base_stable = [
        USDC.to_string(),
        USDT.to_string(),
        DAIPO.to_string(),
        USDH.to_string(),
        CHAI.to_string(),
    ]
    .contains(&base_mint.to_string());
    let is_quote_stable = [
        USDC.to_string(),
        USDT.to_string(),
        DAIPO.to_string(),
        USDH.to_string(),
        CHAI.to_string(),
    ]
    .contains(&quote_mint.to_string());

    // Do not charge fee for swaps of just stable coins.
    return is_base_stable && is_quote_stable;
}
