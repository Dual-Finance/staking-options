use anchor_spl::token::{Mint, Token, TokenAccount};

pub use crate::*;

pub fn exercise(ctx: Context<Exercise>, amount: u64, strike: u64) -> Result<()> {
    // Verify the state is at the right PDA
    check_state!(ctx);

    // Verify the vault is correct.
    check_vault!(ctx);

    // Verify the mint is correct.
    check_mint!(ctx, strike);

    // Take the option tokens and burn
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Burn {
            mint: ctx.accounts.option_mint.to_account_info(),
            from: ctx.accounts.user_so_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info().clone(),
        },
    );
    anchor_spl::token::burn(burn_ctx, amount)?;

    // Take the Quote Token payment
    let payment: u64 = unwrap_int!((unwrap_int!(amount.checked_mul(strike)))
        .checked_div(ctx.accounts.state.quote_decimals as u64));

    // 3.5% fee.
    let fee: u64 = unwrap_int!(unwrap_int!(payment.checked_mul(35)).checked_div(1_000));
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.user_quote_account.to_account_info(),
                to: ctx.accounts.project_quote_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info().clone(),
            },
        ),
        unwrap_int!(payment.checked_sub(fee)),
    )?;
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.user_quote_account.to_account_info(),
                to: ctx.accounts.fee_quote_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info().clone(),
            },
        ),
        fee,
    )?;

    // Transfer the base tokens
    let (_so_vault, so_vault_bump) =
        Pubkey::find_program_address(gen_vault_seeds!(ctx), ctx.program_id);
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.base_token_vault.to_account_info(),
                to: ctx.accounts.user_base_token_account.to_account_info(),
                authority: ctx.accounts.base_token_vault.to_account_info(),
            },
            &[&[
                SO_VAULT_SEED,
                &ctx.accounts.state.period_num.to_be_bytes(),
                &ctx.accounts.state.base_token_mint.key().to_bytes(),
                &[so_vault_bump],
            ]],
        ),
        amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, strike: u64)]
pub struct Exercise<'info> {
    pub authority: Signer<'info>,

    /// State holding all the data for the stake that the staker wants to do.
    pub state: Box<Account<'info, State>>,

    /// Where the SO are coming from.
    #[account(mut)]
    pub user_so_account: Box<Account<'info, TokenAccount>>,
    /// Mint is needed to burn the options.
    #[account(mut)]
    pub option_mint: Box<Account<'info, Mint>>,

    /// Where the payment is coming from.
    #[account(mut)]
    pub user_quote_account: Box<Account<'info, TokenAccount>>,

    /// Where the payment is going
    #[account(mut)]
    pub project_quote_account: Box<Account<'info, TokenAccount>>,

    /// Where the fee is going
    #[account(mut)]
    pub fee_quote_account: Box<Account<'info, TokenAccount>>,

    /// The base token location for this SO.
    #[account(mut)]
    pub base_token_vault: Box<Account<'info, TokenAccount>>,

    /// Where the base tokens are going.
    #[account(mut)]
    pub user_base_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Exercise<'info> {
    pub fn validate_accounts(&self, _amount: u64, _strike: u64) -> Result<()> {
        // Verify the address of quote accounts. Because this account matches,
        // the token type will also be verified by the token program.
        assert_keys_eq!(
            self.state.quote_account,
            self.project_quote_account,
            IncorrectFeeAccount
        );

        // Verify that it is owned by DUAL.
        assert_eq!(
            self.fee_quote_account.owner.key().to_string(),
            "A9YWU67LStgTAYJetbXND2AWqEcvk7FqYJM9nF3VmVpv"
        );

        // Verify expiration
        check_not_expired!(self.state.option_expiration);

        Ok(())
    }
}
