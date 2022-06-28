use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use vipers::prelude::*;

pub use crate::common::*;

pub fn rollover(ctx: Context<Rollover>) -> Result<()> {
    // Verify the state is at the right address
    let (old_so_state, _old_so_state_bump) = Pubkey::find_program_address(
        &[
            SO_CONFIG_SEED,
            &ctx.accounts.old_state.period_num.to_be_bytes(),
            &ctx.accounts.old_state.project_token_mint.key().to_bytes(),
        ],
        ctx.program_id,
    );
    let (new_so_state, _new_so_state_bump) = Pubkey::find_program_address(
        &[
            SO_CONFIG_SEED,
            &ctx.accounts.new_state.period_num.to_be_bytes(),
            &ctx.accounts.new_state.project_token_mint.key().to_bytes(),
        ],
        ctx.program_id,
    );
    assert_keys_eq!(ctx.accounts.old_state.key(), old_so_state, InvalidState);
    assert_keys_eq!(ctx.accounts.new_state.key(), new_so_state, InvalidState);

    // Update the new state with old state tokens
    ctx.accounts.new_state.options_available = unwrap_int!(ctx
        .accounts
        .new_state
        .options_available
        .checked_add(ctx.accounts.old_state.options_available));

    // Update the old state tokens available
    ctx.accounts.old_state.options_available = 0;

    // Move the unallocated tokens
    let (_old_so_vault, old_so_vault_bump) = Pubkey::find_program_address(
        &[
            SO_VAULT_SEED,
            &ctx.accounts.old_state.period_num.to_be_bytes(),
            &ctx.accounts.old_state.project_token_mint.key().to_bytes(),
        ],
        ctx.program_id,
    );
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.old_project_token_vault.to_account_info(),
                to: ctx.accounts.new_project_token_vault.to_account_info(),
                authority: ctx.accounts.old_project_token_vault.to_account_info(),
            },
            &[&[
                SO_VAULT_SEED,
                &ctx.accounts.old_state.period_num.to_be_bytes(),
                &ctx.accounts.old_state.project_token_mint.key().to_bytes(),
                &[old_so_vault_bump],
            ]],
        ),
        ctx.accounts.old_project_token_vault.amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct Rollover<'info> {
    pub authority: Signer<'info>,

    /// State holding all the data for the stake that the staker wants to do.
    #[account(mut)]
    pub old_state: Box<Account<'info, State>>,

    /// State holding all the data for the stake that the staker wants to do.
    #[account(mut)]
    pub new_state: Box<Account<'info, State>>,

    /// The project token location
    #[account(mut)]
    pub old_project_token_vault: Box<Account<'info, TokenAccount>>,

    /// The project token location
    #[account(mut)]
    pub new_project_token_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Rollover<'info> {
    pub fn validate_accounts(&self) -> Result<()> {
        assert_keys_eq!(
            self.new_state.authority.key(),
            self.old_state.authority.key(),
            InvalidState
        );
        assert_keys_eq!(
            self.new_state.project_token_mint.key(),
            self.old_state.project_token_mint.key(),
            InvalidState
        );

        // Verify the authority
        assert_keys_eq!(self.authority, self.old_state.authority);
        assert_keys_eq!(self.authority, self.new_state.authority);

        // Verify that it is expired
        check_expired!(self.old_state.subscription_period_end);
        check_not_expired!(self.new_state.subscription_period_end);

        Ok(())
    }
}
