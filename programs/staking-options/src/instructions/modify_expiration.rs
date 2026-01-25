use anchor_spl::token::{Mint, TokenAccount};

pub use crate::common::*;
pub use crate::*;

pub fn modify_expiration(
    ctx: Context<ModifyExpiration>,
    new_expiration_unix_sec: u64,
) -> Result<()> {
    // Only allow accelerating expiration.
    assert!(ctx.accounts.state.option_expiration >= new_expiration_unix_sec);

    // Require that the authority holds all the outstanding options and no more are issued.

    // Only 1 strike because strikes are independent.
    assert!(ctx.accounts.state.strikes.len() == 1);
    assert!(ctx.accounts.user_so_account.owner == *ctx.accounts.authority.key);
    assert!(ctx.accounts.user_so_account.amount == ctx.accounts.option_mint.supply);

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
    pub user_so_account: Box<Account<'info, TokenAccount>>,
    /// Mint is needed to get the number of outstanding options.
    pub option_mint: Box<Account<'info, Mint>>,
}
