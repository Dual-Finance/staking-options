use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use vipers::prelude::*;

pub use crate::common::*;

pub fn withdraw(
    ctx: Context<Withdraw>,
) -> Result<()> {
    // Send project tokens from the vault.
    let (_so_vault, so_vault_bump) = Pubkey::find_program_address(
        gen_vault_seeds!(ctx),
        ctx.program_id,
    );
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.project_token_vault.to_account_info(),
                to: ctx.accounts.project_token_account.to_account_info(),
                authority: ctx.accounts.project_token_vault.to_account_info(),
            },
            &[&[
                SO_VAULT_SEED,
                &ctx.accounts.state.period_num.to_be_bytes(),
                &ctx.accounts.state.project_token_mint.key().to_bytes(),
                &[so_vault_bump],
            ]],
        ),
        ctx.accounts.project_token_vault.amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    // State holding all the data for the stake that the staker wants to do.
    #[account(mut, close=authority)]
    pub state: Box<Account<'info, State>>,

    /// The project token location
    #[account(mut)]
    pub project_token_vault: Box<Account<'info, TokenAccount>>,

    /// Where the tokens are getting returned to
    #[account(mut)]
    pub project_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl <'info> Withdraw<'info>  {
    pub fn validate_accounts(&self) -> Result<()> {
        // Verify the token types match so you cannot withdraw from a different
        // vault.
        //check_vault!(self);

        // Verify the state is at the right address
        //check_state!(self);

        // Verify the authority to init strike against the state authority
        assert_keys_eq!(self.authority, self.state.authority);

        // Verify that it is not already expired
        check_expired!(self.state.option_expiration);

        Ok(())
    }
}