use anchor_spl::token::{Mint, Token, TokenAccount};

pub use crate::ErrorCode::IncorrectFeeAccount;
pub use crate::*;

pub fn exercise(ctx: Context<Exercise>, amount_lots: u64, strike: u64) -> Result<()> {
    // Verify the mint is correct.
    check_mint!(ctx, strike, bump);

    // TODO: Store all of the strikes on the state object and their bumps as well as a mapping of token to strike

    // Take the option tokens and burn
    anchor_spl::token::burn(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.option_mint.to_account_info(),
                from: ctx.accounts.user_so_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
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

    // Take the Quote Token payment
    let payment: u64 = amount_lots.checked_mul(strike).unwrap();

    // Charge fee when it is not DUAL DAO is exercising.
    if ctx.accounts.user_quote_account.owner.key().to_string()
        != "7Z36Efbt7a4nLiV7s5bY7J2e4TJ6V9JEKGccsy2od2bE"
    {
        // 3.5% fee.
        let fee: u64 = payment.checked_mul(35).unwrap().checked_div(1_000).unwrap();
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.user_quote_account.to_account_info(),
                    to: ctx.accounts.project_quote_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info().clone(),
                },
            ),
            payment.checked_sub(fee).unwrap(),
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
    } else {
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.user_quote_account.to_account_info(),
                    to: ctx.accounts.project_quote_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info().clone(),
                },
            ),
            payment,
        )?;
    }

    // Transfer the base tokens
    anchor_spl::token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.base_vault.to_account_info(),
                to: ctx.accounts.user_base_account.to_account_info(),
                authority: ctx.accounts.base_vault.to_account_info(),
            },
            &[&[
                SO_VAULT_SEED,
                &ctx.accounts.state.so_name.as_bytes(),
                &ctx.accounts.state.base_mint.key().to_bytes(),
                &[ctx.accounts.state.vault_bump],
            ]],
        ),
        amount_lots.checked_mul(ctx.accounts.state.lot_size).unwrap(),
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, strike: u64)]
pub struct Exercise<'info> {
    pub authority: Signer<'info>,

    /// State holding all the data for the stake that the staker wants to do.
    #[account(mut,
        seeds = [
            SO_CONFIG_SEED,
            state.so_name.as_bytes(),
            &state.base_mint.key().to_bytes()
        ],
        bump = state.state_bump
    )]
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
    #[account(mut,
        seeds = [SO_VAULT_SEED, state.so_name.as_bytes(), &state.base_mint.key().to_bytes()],
        bump = state.vault_bump,
    )]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    /// Where the base tokens are going.
    #[account(mut)]
    pub user_base_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Exercise<'info> {
    pub fn validate_accounts(&self, _amount: u64, _strike: u64) -> Result<()> {
        // Verify the address of quote accounts. Because this account matches,
        // the token type will also be verified by the token program.
        require_keys_eq!(
            self.state.quote_account.key(),
            self.project_quote_account.key(),
            IncorrectFeeAccount
        );

        // Verify that it is owned by DUAL.
        require_eq!(
            self.fee_quote_account.owner.key().to_string(),
            "7Z36Efbt7a4nLiV7s5bY7J2e4TJ6V9JEKGccsy2od2bE"
        );

        // Verify expiration
        check_not_expired!(self.state.option_expiration);

        Ok(())
    }
}
