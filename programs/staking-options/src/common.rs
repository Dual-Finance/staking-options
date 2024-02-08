use anchor_lang::prelude::*;

pub const SO_CONFIG_SEED: &[u8] = b"so-config";
pub const SO_VAULT_SEED: &[u8] = b"so-vault";
pub const SO_REVERSE_VAULT_SEED: &[u8] = b"so-reverse-vault";
pub const SO_MINT_SEED: &[u8] = b"so-mint";
pub const DUAL_DAO_ADDRESS: &str = "7Z36Efbt7a4nLiV7s5bY7J2e4TJ6V9JEKGccsy2od2bE";
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

const DUAL_RISK_MANAGER: &str = "CkcJx7Uwgxck5zm3DqUp2N1ikkkoPn2wA8zf7oS4tFSZ";

const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
const DAIPO: &str = "EjmyN6qEC1Tf1JxiG1ae7UTJhUxSwk1TCWNWqxWV4J6o";
const USDH: &str = "USDH1SM1ojwWUga67PGrgFWUHibbjqMvuMaDkRJTgkX";
const CHAI: &str = "3jsFX1tx2Z8ewmamiwSU851GzyzM2DJMq7KWW5DM8Py3";

const WBTCPO: &str = "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh";
const TBTC: &str = "6DNSN2BJsaPFdFFc1zP37kkeNe4Usc1Sqkzr9C9vPWcU";
const WSTETHPO: &str = "ZScHuTtqZukUrtZS43teTKGs2VqkKL8k4QCouR2n6Uo";
const RETHPO: &str = "9UV2pC1qPaVMfRv8CF7qhv7ihbzR91pr2LX9y2FDfGLy";
const WETHPO: &str = "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs";
const WSOL: &str = "So11111111111111111111111111111111111111112";

const MNGO: &str = "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac";
const RAY: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
const NOS: &str = "nosXBVoaCTtYdLvKY6Csb4AC8JCdQKKAaWYtx2ZMoo7";

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

pub fn get_fee_bps(base_mint: Pubkey, quote_mint: Pubkey) -> u64 {
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

    // Reduced fee on stables
    if is_base_stable && is_quote_stable {
        return 5;
    }

    let is_base_major = [
        WBTCPO.to_string(),
        TBTC.to_string(),
        WETHPO.to_string(),
        WSTETHPO.to_string(),
        RETHPO.to_string(),
        WSOL.to_string(),
    ]
    .contains(&base_mint.to_string());

    let is_quote_major = [
        WBTCPO.to_string(),
        TBTC.to_string(),
        WETHPO.to_string(),
        WSTETHPO.to_string(),
        RETHPO.to_string(),
        WSOL.to_string(),
    ]
    .contains(&quote_mint.to_string());

    let is_base_partner = [
        MNGO.to_string(),
        RAY.to_string(),
        NOS.to_string(),
    ]
    .contains(&base_mint.to_string());

    let is_quote_partner = [
        MNGO.to_string(),
        RAY.to_string(),
        NOS.to_string(),
    ]
    .contains(&quote_mint.to_string());

    // Charge reduced fees on pairs of majors.
    if (is_base_major && is_quote_stable) || (is_quote_major && is_base_stable) {
        return 25;
    }
    // Charge reduced fees to partners.
    if (is_base_partner && is_quote_stable) || (is_quote_partner && is_base_stable) {
        return 50;
    }
    // Charge lower fee on major/major pairs
    if is_base_major && is_quote_major {
        return 5;
    }

    return 350;
}
