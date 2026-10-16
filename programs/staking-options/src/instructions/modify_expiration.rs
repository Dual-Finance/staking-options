use anchor_spl::token::{Mint, TokenAccount};

pub use crate::*;

pub fn modify_expiration(
    ctx: Context<ModifyExpiration>,
    new_expiration_unix_sec: u64,
) -> Result<()> {
    // Only 1 strike because strikes are independent.
    require!(
        ctx.accounts.state.strikes.len() == 1,
        SOErrorCode::TooManyStrikes
    );
    // Only allow accelerating expiration.
    assert!(ctx.accounts.state.option_expiration >= new_expiration_unix_sec);

    // Verify the option mint is the real SO mint for this state's single strike.
    // Without this, a decoy zero-supply mint (and a matching empty token
    // account) could be substituted to trivially satisfy the checks below and
    // force-expire someone else's option.
    let strike = ctx.accounts.state.strikes[0];
    check_mint!(ctx, strike, _bump);

    // Only allow accelerating expiration.
    require!(
        ctx.accounts.state.option_expiration >= new_expiration_unix_sec,
        SOErrorCode::InvalidExpiration
    );

    // Require that the authority holds all the outstanding options and no more
    // are issued.
    require!(
        ctx.accounts.user_so_account.owner == *ctx.accounts.authority.key,
        SOErrorCode::IncorrectAuthority
    );
    require!(
        ctx.accounts.user_so_account.amount == ctx.accounts.option_mint.supply,
        SOErrorCode::NotEnoughTokens
    );

    ctx.accounts.state.option_expiration = new_expiration_unix_sec;
    if ctx.accounts.state.subscription_period_end > new_expiration_unix_sec {
        ctx.accounts.state.subscription_period_end = new_expiration_unix_sec;
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(new_expiration_unix_sec: u64)]
pub struct ModifyExpiration<'info> {
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

    /// User must have all the outstanding staking options for the SO mint.
    #[account(constraint = user_so_account.mint == option_mint.key() @ SOErrorCode::WrongMint)]
    pub user_so_account: Box<Account<'info, TokenAccount>>,
    /// Mint is needed to get the number of outstanding options.
    pub option_mint: Box<Account<'info, Mint>>,
}

impl<'info> ModifyExpiration<'info> {
    pub fn validate_accounts(&self) -> Result<()> {
        // Only the SO state authority may modify the expiration. This mirrors
        // the other authority-write instructions (issue, withdraw, ...).
        require_keys_eq!(
            self.authority.key(),
            self.state.authority,
            SOErrorCode::IncorrectAuthority
        );

        Ok(())
    }
}
