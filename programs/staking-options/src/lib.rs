use anchor_lang::prelude::*;
use vipers::prelude::*;

#[macro_use]
mod macros;

mod common;
mod errors;
mod instructions;

pub use crate::common::*;
pub use crate::errors::ErrorCode;
pub use crate::instructions::*;

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "Dual Staking Options",
    project_url: "http://dual.finance",
    contacts: "email:dual-labs@dual.finance",
    policy: "https://github.com/Dual-Finance/staking-options/blob/master/SECURITY.md",

    preferred_languages: "en",
    source_code: "https://github.com/Dual-Finance/staking-options",
    auditors: "None"
}

declare_id!("4yx1NJ4Vqf2zT1oVLk4SySBhhDJXmXFt88ncm4gPxtL7");

#[program]
pub mod staking_options {
    use super::*;

    #[access_control(ctx.accounts.validate_accounts(num_tokens_to_add))]
    pub fn add_tokens(ctx: Context<AddTokens>, num_tokens_to_add: u64) -> Result<()> {
        add_tokens::add_tokens(ctx, num_tokens_to_add)
    }

    #[access_control(ctx.accounts.validate_accounts(option_expiration, subscription_period_end))]
    pub fn config(
        ctx: Context<Config>,
        option_expiration: u64,
        subscription_period_end: u64,
        num_tokens: u64,
        lot_size: u64,
        so_name: String,
    ) -> Result<()> {
        config::config(
            ctx,
            option_expiration,
            subscription_period_end,
            num_tokens,
            lot_size,
            so_name,
        )
    }

    #[access_control(ctx.accounts.validate_accounts(amount, strike))]
    pub fn exercise(ctx: Context<Exercise>, amount: u64, strike: u64) -> Result<()> {
        exercise::exercise(ctx, amount, strike)
    }

    #[access_control(ctx.accounts.validate_accounts(strike))]
    pub fn init_strike(ctx: Context<InitStrike>, strike: u64) -> Result<()> {
        init_strike::init_strike(ctx, strike)
    }

    #[access_control(ctx.accounts.validate_accounts(strike))]
    pub fn init_strike_with_payer(ctx: Context<InitStrikeWithPayer>, strike: u64) -> Result<()> {
        init_strike::init_strike_with_payer(ctx, strike)
    }

    #[access_control(ctx.accounts.validate_accounts(amount))]
    pub fn issue(ctx: Context<Issue>, amount: u64, strike: u64) -> Result<()> {
        issue::issue(ctx, amount, strike)
    }

    pub fn name_token(ctx: Context<NameToken>, strike: u64) -> Result<()> {
        name_token::name_token(ctx, strike)
    }

    #[access_control(ctx.accounts.validate_accounts())]
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        withdraw::withdraw(ctx)
    }
}
