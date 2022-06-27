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

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod staking_options {
    use super::*;

    #[access_control(ctx.accounts.validate_accounts(num_tokens_to_add))]
    pub fn add_tokens(ctx: Context<AddTokens>, num_tokens_to_add: u64) -> Result<()> {
        Ok(())
    }

    #[access_control(ctx.accounts.validate_accounts(period_num, option_expiration, subscription_period_end, num_tokens_in_period))]
    pub fn config(
        ctx: Context<Config>,
        period_num: u64,
        option_expiration: u64,
        subscription_period_end: u64,
        num_tokens_in_period: u64,
    ) -> Result<()> {
        Ok(())
    }

    #[access_control(ctx.accounts.validate_accounts(amount))]
    pub fn exercise(ctx: Context<Exercise>, amount: u64) -> Result<()> {
        Ok(())
    }

    #[access_control(ctx.accounts.validate_accounts(strike))]
    pub fn init_strike(ctx: Context<InitStrike>, strike: u64) -> Result<()> {
        Ok(())
    }

    #[access_control(ctx.accounts.validate_accounts(amount))]
    pub fn issue(ctx: Context<Issue>, amount: u64) -> Result<()> {
        Ok(())
    }

    #[access_control(ctx.accounts.validate_accounts())]
    pub fn rollover(ctx: Context<Rollover>) -> Result<()> {
        Ok(())
    }

    #[access_control(ctx.accounts.validate_accounts())]
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        Ok(())
    }
}
