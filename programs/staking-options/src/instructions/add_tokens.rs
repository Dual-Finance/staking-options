use anchor_spl::token;
use anchor_spl::token::{Token, TokenAccount};
use vipers::prelude::*;

use crate::*;

pub fn add_tokens(ctx: Context<AddTokens>, num_tokens_to_add: u64) -> Result<()> {
    // Move tokens from the depositor to the vault.
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.base_account.to_account_info(),
            to: ctx.accounts.base_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, num_tokens_to_add)?;

    // Update the state to reflect the newly available tokens for options.
    ctx.accounts.state.options_available = unwrap_int!(ctx
        .accounts
        .state
        .options_available
        .checked_add(num_tokens_to_add));

    Ok(())
}

#[derive(Accounts)]
#[instruction(num_tokens_to_add: u64)]
pub struct AddTokens<'info> {
    pub authority: Signer<'info>,

    /// State holding all the data for the intended stake.
    #[account(mut,
        seeds = [
            SO_CONFIG_SEED,
            state.so_name.as_bytes(),
            &state.base_mint.key().to_bytes()
        ],
        bump = state.state_bump
    )]
    pub state: Box<Account<'info, State>>,

    /// Where the base tokens are going to be held. Controlled by this program.
    #[account(mut,
        seeds = [SO_VAULT_SEED, state.so_name.as_bytes(), &state.base_mint.key().to_bytes()],
        bump = state.vault_bump,
    )]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    /// Where the additional tokens are coming from.
    #[account(mut)]
    pub base_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> AddTokens<'info> {
    pub fn validate_accounts(&self, _num_tokens_to_add: u64) -> Result<()> {
        // Do not need to verify num tokens to add is valid because the token
        // program does that.

        // Check that the token type matches the mint in the SO state that is
        // getting credited.
        assert_keys_eq!(self.base_account.mint, self.state.base_mint, WrongMint);

        // Do not allow adding tokens to an SO that is expired already.
        check_not_expired!(self.state.subscription_period_end);

        Ok(())
    }
}
