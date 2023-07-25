use anchor_lang::AccountsClose;
use anchor_spl::token::{Token, TokenAccount};

pub use crate::common::*;
pub use crate::*;

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    // Allow partial withdraw after the subscription period end.
    let now: u64 = Clock::get().unwrap().unix_timestamp as u64;

    let transfer = anchor_spl::token::Transfer {
        from: ctx.accounts.base_vault.to_account_info(),
        to: ctx.accounts.base_account.to_account_info(),
        authority: ctx.accounts.base_vault.to_account_info(),
    };
    let seeds: &[&[&[u8]]] = &[&[
        SO_VAULT_SEED,
        &ctx.accounts.state.so_name.as_bytes(),
        &ctx.accounts.state.base_mint.key().to_bytes(),
        &[ctx.accounts.state.vault_bump],
    ]];

    if now > ctx.accounts.state.option_expiration {
        // Send base tokens from the vault.
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer,
                seeds,
            ),
            ctx.accounts.base_vault.amount,
        )?;
        // Conditionally close the SOState if it is the final withdraw.
        ctx.accounts
            .state
            .close(ctx.accounts.authority.to_account_info())?;
    } else {
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer,
                seeds,
            ),
            ctx.accounts.state.options_available,
        )?;
        ctx.accounts.state.options_available = 0;
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct Withdraw<'info> {
    #[account(mut)]
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
    pub state: Account<'info, State>,

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
        // Verify the authority to withdraw against the state authority.
        require_keys_eq!(self.authority.key(), self.state.authority);

        // Verify that subscription period has ended.
        check_expired!(self.state.subscription_period_end);

        Ok(())
    }
}

pub fn withdraw_all(ctx: Context<WithdrawAll>) -> Result<()> {
    // Allow partial withdraw after the subscription period end.
    let now: u64 = Clock::get().unwrap().unix_timestamp as u64;

    let base_transfer = anchor_spl::token::Transfer {
        from: ctx.accounts.base_vault.to_account_info(),
        to: ctx.accounts.base_account.to_account_info(),
        authority: ctx.accounts.base_vault.to_account_info(),
    };
    let base_seeds: &[&[&[u8]]] = &[&[
        SO_VAULT_SEED,
        &ctx.accounts.state.so_name.as_bytes(),
        &ctx.accounts.state.base_mint.key().to_bytes(),
        &[ctx.accounts.state.vault_bump],
    ]];

    let quote_transfer = anchor_spl::token::Transfer {
        from: ctx.accounts.quote_vault.to_account_info(),
        to: ctx.accounts.quote_account.to_account_info(),
        authority: ctx.accounts.quote_vault.to_account_info(),
    };
    let quote_fee_transfer = anchor_spl::token::Transfer {
        from: ctx.accounts.quote_vault.to_account_info(),
        to: ctx.accounts.fee_quote_account.to_account_info(),
        authority: ctx.accounts.quote_vault.to_account_info(),
    };
    let quote_seeds: &[&[&[u8]]] = &[&[
        SO_REVERSE_VAULT_SEED,
        &ctx.accounts.state.so_name.as_bytes(),
        &ctx.accounts.state.base_mint.key().to_bytes(),
        &[ctx.accounts.state.quote_vault_bump],
    ]];

    if now > ctx.accounts.state.option_expiration {
        // Send base tokens from the vault.
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                base_transfer,
                base_seeds,
            ),
            ctx.accounts.base_vault.amount,
        )?;

        let total_quote_tokens = ctx.accounts.quote_vault.amount;
        let fee: u64 = total_quote_tokens
            .checked_mul(35)
            .unwrap()
            .checked_div(1_000)
            .unwrap();
        // Send quote tokens from the vault.
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                quote_transfer,
                quote_seeds,
            ),
            total_quote_tokens.checked_sub(fee).unwrap(),
        )?;

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                quote_fee_transfer,
                quote_seeds,
            ),
            fee,
        )?;

        // Close the SOState if it is the final withdraw.
        ctx.accounts
            .state
            .close(ctx.accounts.authority.to_account_info())?;
    } else {
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                base_transfer,
                base_seeds,
            ),
            ctx.accounts.state.options_available,
        )?;
        ctx.accounts.state.options_available = 0;

        // Dont withdraw the quote tokens since there are still reverse options
        // exercisable for them.
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct WithdrawAll<'info> {
    #[account(mut)]
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
    pub state: Account<'info, State>,

    /// The base token location
    #[account(mut,
        seeds = [SO_VAULT_SEED, state.so_name.as_bytes(), &state.base_mint.key().to_bytes()],
        bump = state.vault_bump,
    )]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    /// Where the tokens are getting returned to
    #[account(mut)]
    pub base_account: Box<Account<'info, TokenAccount>>,

    /// The quote token location
    #[account(mut,
        seeds = [SO_REVERSE_VAULT_SEED, state.so_name.as_bytes(), &state.base_mint.key().to_bytes()],
        bump = state.quote_vault_bump,
    )]
    pub quote_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub quote_account: Box<Account<'info, TokenAccount>>,

    /// DUAL DAO owned fee account
    #[account(mut)]
    pub fee_quote_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawAll<'info> {
    pub fn validate_accounts(&self) -> Result<()> {
        // Verify the authority to withdraw against the state authority.
        require_keys_eq!(self.authority.key(), self.state.authority);

        // Verify that subscription period has ended.
        check_expired!(self.state.subscription_period_end);

        Ok(())
    }
}
