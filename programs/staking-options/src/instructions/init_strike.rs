use anchor_spl::token::{Mint, Token};

pub use crate::common::*;
pub use crate::*;

pub fn init_strike(ctx: Context<InitStrike>, strike: u64) -> Result<()> {
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
    #[account(mut,
        seeds = [
            SO_CONFIG_SEED,
            state.so_name.as_bytes(),
            &state.base_mint.key().to_bytes()
        ],
        bump = state.state_bump
    )]
    pub state: Box<Account<'info, State>>,

    #[account(
        init,
        payer = authority,
        seeds = [SO_MINT_SEED, &state.key().to_bytes(), &strike.to_be_bytes()],
        bump,
        mint::decimals = 0,
        mint::authority = option_mint)]
    pub option_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitStrike<'info> {
    pub fn validate_accounts(&self, _strike: u64) -> Result<()> {
        // Verify the authority to init strike against the state authority
        require_keys_eq!(self.authority.key(), self.state.authority);

        // Verify that it is not already expired
        check_not_expired!(self.state.subscription_period_end);

        // Make sure there are not too many strikes already.
        require!(self.state.strikes.len() < 100, SOErrorCode::TooManyStrikes);

        Ok(())
    }
}

pub fn init_strike_with_payer(ctx: Context<InitStrikeWithPayer>, strike: u64) -> Result<()> {
    ctx.accounts.state.strikes.push(strike);

    Ok(())
}

#[derive(Accounts)]
#[instruction(strike: u64)]
pub struct InitStrikeWithPayer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    // State holding all the data for the stake that the staker wants to do.
    // Needs to be updated to reflect the new strike.
    #[account(mut,
        seeds = [
            SO_CONFIG_SEED,
            state.so_name.as_bytes(),
            &state.base_mint.key().to_bytes()
        ],
        bump = state.state_bump
    )]
    pub state: Box<Account<'info, State>>,

    #[account(
        init,
        payer = payer,
        seeds = [SO_MINT_SEED, &state.key().to_bytes(), &strike.to_be_bytes()],
        bump,
        mint::decimals = 0,
        mint::authority = option_mint)]
    pub option_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitStrikeWithPayer<'info> {
    pub fn validate_accounts(&self, _strike: u64) -> Result<()> {
        // Verify the authority to init strike against the state authority
        require_keys_eq!(self.authority.key(), self.state.authority);

        // Verify that it is not already expired
        check_not_expired!(self.state.subscription_period_end);

        // Make sure there are not too many strikes already.
        require!(self.state.strikes.len() < 100, SOErrorCode::TooManyStrikes);

        Ok(())
    }
}

// Same as init strike except this also initializes the reverse
pub fn init_strike_reversible(ctx: Context<InitStrikeReversible>, strike: u64) -> Result<()> {
    ctx.accounts.state.strikes.push(strike);

    Ok(())
}

#[derive(Accounts)]
#[instruction(strike: u64)]
pub struct InitStrikeReversible<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    // State holding all the data for the stake that the staker wants to do.
    // Needs to be updated to reflect the new strike.
    #[account(mut,
        seeds = [
            SO_CONFIG_SEED,
            state.so_name.as_bytes(),
            &state.base_mint.key().to_bytes()
        ],
        bump = state.state_bump
    )]
    pub state: Box<Account<'info, State>>,

    #[account(
        init,
        payer = payer,
        seeds = [SO_REVERSE_MINT_SEED, &state.key().to_bytes(), &strike.to_be_bytes()],
        bump,
        mint::decimals = 0,
        mint::authority = reverse_option_mint)]
    pub reverse_option_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        seeds = [SO_MINT_SEED, &state.key().to_bytes(), &strike.to_be_bytes()],
        bump,
        mint::decimals = 0,
        mint::authority = option_mint)]
    pub option_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitStrikeReversible<'info> {
    pub fn validate_accounts(&self, _strike: u64) -> Result<()> {
        // Verify the authority to init strike against the state authority
        require_keys_eq!(self.authority.key(), self.state.authority);

        // Verify that it is not already expired
        check_not_expired!(self.state.subscription_period_end);

        // Make sure there are not too many strikes already.
        require!(self.state.strikes.len() < 100, SOErrorCode::TooManyStrikes);

        Ok(())
    }
}
