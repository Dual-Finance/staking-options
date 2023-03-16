use anchor_spl::token::{Mint, Token, TokenAccount};

pub use crate::*;

pub fn issue(ctx: Context<Issue>, amount: u64, strike: u64) -> Result<()> {
    // Verify the mint is at the right address
    check_mint!(ctx, strike, bump);

    let amount_lots: u64 = amount.checked_div(ctx.accounts.state.lot_size).unwrap();

    anchor_spl::token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.option_mint.to_account_info(),
                to: ctx.accounts.user_so_account.to_account_info(),
                authority: ctx.accounts.option_mint.to_account_info(),
            },
            &[&[
                SO_MINT_SEED,
                &ctx.accounts.state.key().to_bytes(),
                &strike.to_be_bytes(),
                &[bump],
            ]],
        ),
        amount_lots,
    )?;

    // Update state to reflect the number of available tokens
    ctx.accounts.state.options_available = ctx
        .accounts
        .state
        .options_available
        .checked_sub(amount)
        .unwrap();

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, strike: u64)]
pub struct Issue<'info> {
    pub authority: Signer<'info>,

    // State holding all the data for the stake that the staker wants to do.
    #[account(mut,
        seeds = [
            SO_CONFIG_SEED,
            state.so_name.as_bytes(),
            &state.base_mint.key().to_bytes()
        ],
        bump = state.state_bump
    )]
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
        // Verify the authority signer matches state authority. in this case, it
        // can be the issue authority or the so authority.
        require!(
            self.authority.key.to_bytes() == self.state.authority.to_bytes()
                || self.authority.key.to_bytes() == self.state.issue_authority.to_bytes(),
            SOErrorCode::IncorrectAuthority
        );

        // Verify subscription period
        check_not_expired!(self.state.subscription_period_end);

        // Make sure there are enough tokens to back the options.
        require!(
            self.state.options_available >= amount,
            SOErrorCode::NotEnoughTokens
        );

        // Do not need to verify the SO mint is at the right address. The
        // authority check is sufficient. If a different mint was somehow
        // assigned the same authority, it is not an issue if the authority
        // issues those tokens.

        Ok(())
    }
}
