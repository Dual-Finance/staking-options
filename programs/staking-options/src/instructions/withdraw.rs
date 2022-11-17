use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use vipers::prelude::*;

pub use crate::common::*;

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    // Send base tokens from the vault.
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.base_vault.to_account_info(),
                to: ctx.accounts.base_account.to_account_info(),
                authority: ctx.accounts.base_vault.to_account_info(),
            },
            &[&[
                SO_VAULT_SEED,
                &ctx.accounts.state.so_name.as_bytes(),
                &ctx.accounts.state.base_mint.key().to_bytes(),
                &[ctx.accounts.state.vault_bump],
            ]],
        ),
        ctx.accounts.base_vault.amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// State holding all the data for the stake that the staker wants to do.
    #[account(mut,
        close=authority,
        seeds = [
            SO_CONFIG_SEED,
            state.so_name.as_bytes(),
            &state.base_mint.key().to_bytes()
        ],
        bump = state.state_bump
    )]
    pub state: Box<Account<'info, State>>,

    /// The base token location
    #[account(mut,
        seeds = [SO_VAULT_SEED, state.so_name.as_bytes(), &state.base_mint.key().to_bytes()],
        bump = state.vault_bump,
    )]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    /// Where the tokens are getting returned to
    #[account(mut)]
    pub base_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn validate_accounts(&self) -> Result<()> {
        // Verify the authority to init strike against the state authority
        assert_keys_eq!(self.authority, self.state.authority);

        // Verify that it is already expired
        check_expired!(self.state.option_expiration);

        Ok(())
    }
}
