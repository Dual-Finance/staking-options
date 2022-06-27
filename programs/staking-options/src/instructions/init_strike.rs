use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use vipers::prelude::*;

pub use crate::common::*;

pub fn init_strike(
    ctx: Context<InitStrike>,
    strike: u64,
) -> Result<()> {
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
    #[account(mut)]
    pub state: Box<Account<'info, State>>,

    #[account(
        init,
        payer = authority,
        seeds = [SO_MINT_SEED, &state.key().to_bytes(), &strike.to_be_bytes()],
        bump,
        mint::decimals = state.decimals,
        mint::authority = state.authority)]
    pub option_mint: Account<'info, Mint>,

    // TODO: Consider a data account at PDA(mint address) for a reverse lookup
    // so if you have a token, you can remember the strike and project mint.
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl <'info> InitStrike<'info>  {
    pub fn validate_accounts(&self, _strike: u64) -> Result<()> {
        // Verify the state is at the right address
        check_state!(self);

        // Verify the authority to init strike against the state authority
        assert_keys_eq!(self.authority, self.state.authority);

        // Verify that it is not already expired
        check_not_expired!(self.state.subscription_period_end);

        // Make sure there are not too many strikes already.
        invariant!(self.state.strikes.len() < 100);

        Ok(())
    }
}