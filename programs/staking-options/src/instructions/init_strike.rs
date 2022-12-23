use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use vipers::prelude::*;

pub use crate::common::*;

pub fn init_strike(ctx: Context<InitStrike>, strike: u64) -> Result<()> {
    ctx.accounts.state.strikes.push(strike);

    Ok(())
}

#[derive(Accounts)]
#[instruction(strike: u64)]
pub struct InitStrike<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    // State holding all the data for the stake that the staker wants to do.
    // Needs to be updated to reflect the new strike.
    #[account(mut,
        seeds = [
            SO_CONFIG_SEED,
            state.so_name.as_bytes(),
            &state.base_mint.key().to_bytes()
        ],
        bump = state.state_bump
    )]
    pub state: Box<Account<'info, State>>,

    #[account(
        init,
        payer = authority,
        seeds = [SO_MINT_SEED, &state.key().to_bytes(), &strike.to_be_bytes()],
        bump,
        mint::decimals = 0,
        mint::authority = option_mint)]
    pub option_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitStrike<'info> {
    pub fn validate_accounts(&self, _strike: u64) -> Result<()> {
        // Verify the authority to init strike against the state authority
        assert_keys_eq!(self.authority, self.state.authority);

        // Verify that it is not already expired
        check_not_expired!(self.state.subscription_period_end);

        // Make sure there are not too many strikes already.
        invariant!(self.state.strikes.len() < 100);

        Ok(())
    }
}

pub fn init_strike_with_payer(ctx: Context<InitStrikeWithPayer>, strike: u64) -> Result<()> {
    ctx.accounts.state.strikes.push(strike);

    Ok(())
}

#[derive(Accounts)]
#[instruction(strike: u64)]
pub struct InitStrikeWithPayer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    // State holding all the data for the stake that the staker wants to do.
    // Needs to be updated to reflect the new strike.
    #[account(mut,
        seeds = [
            SO_CONFIG_SEED,
            state.so_name.as_bytes(),
            &state.base_mint.key().to_bytes()
        ],
        bump = state.state_bump
    )]
    pub state: Box<Account<'info, State>>,

    #[account(
        init,
        payer = payer,
        seeds = [SO_MINT_SEED, &state.key().to_bytes(), &strike.to_be_bytes()],
        bump,
        mint::decimals = 0,
        mint::authority = option_mint)]
    pub option_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitStrikeWithPayer<'info> {
    pub fn validate_accounts(&self, _strike: u64) -> Result<()> {
        // Verify the authority to init strike against the state authority
        assert_keys_eq!(self.authority, self.state.authority);

        // Verify that it is not already expired
        check_not_expired!(self.state.subscription_period_end);

        // Make sure there are not too many strikes already.
        invariant!(self.state.strikes.len() < 100);

        Ok(())
    }
}
