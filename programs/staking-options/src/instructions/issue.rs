use anchor_spl::token::{Mint, Token, TokenAccount};

pub use crate::*;

pub fn issue(ctx: Context<Issue>, amount: u64, strike: u64) -> Result<()> {
    // TODO: Log the state

    // Verify the state is at the right address
    check_state!(ctx);

    // Verify the mint is at the right address
    check_mint!(ctx, strike);

    // Mint tokens for the user
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::MintTo {
            mint: ctx.accounts.option_mint.to_account_info(),
            to: ctx.accounts.user_so_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info().clone(),
        },
    );
    anchor_spl::token::mint_to(cpi_ctx, amount)?;

    // Update state to reflect the number of available tokens
    ctx.accounts.state.options_available =
        unwrap_int!(ctx.accounts.state.options_available.checked_sub(amount));

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, strike: u64)]
pub struct Issue<'info> {
    pub authority: Signer<'info>,

    // State holding all the data for the stake that the staker wants to do.
    #[account(mut)]
    pub state: Box<Account<'info, State>>,

    #[account(mut)]
    pub option_mint: Account<'info, Mint>,

    /// Where the options will be sent.
    #[account(mut)]
    pub user_so_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Issue<'info> {
    pub fn validate_accounts(&self, amount: u64) -> Result<()> {
        // Verify the authority signer matches state authority
        assert_keys_eq!(self.authority, self.state.authority);

        // Verify subscription period
        check_not_expired!(self.state.subscription_period_end);

        // Make sure there are enough tokens to back the options.
        invariant!(self.state.options_available >= amount, NotEnoughTokens);

        // Do not need to verify the SO mint is at the right address. The
        // authority check is sufficient. If a different mint was somehow
        // assigned the same authority, it is not an issue if the authority
        // issues those tokens.

        Ok(())
    }
}
