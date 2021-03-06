use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

pub use crate::*;

pub fn config(
    ctx: Context<Config>,
    period_num: u64,
    option_expiration: u64,
    subscription_period_end: u64,
    num_tokens_in_period: u64,
    so_name: String,
) -> Result<()> {
    // Verify the SO name is a reasonable length.
    invariant!(so_name.len() < 64);

    // Fill out the State
    ctx.accounts.state.period_num = period_num;
    ctx.accounts.state.so_name = so_name;
    ctx.accounts.state.authority = ctx.accounts.so_authority.key();
    ctx.accounts.state.options_available = num_tokens_in_period;
    ctx.accounts.state.option_expiration = option_expiration;
    ctx.accounts.state.subscription_period_end = subscription_period_end;
    ctx.accounts.state.base_decimals = ctx.accounts.base_token_mint.decimals;
    ctx.accounts.state.quote_decimals = ctx.accounts.quote_mint.decimals;
    ctx.accounts.state.base_token_mint = ctx.accounts.base_token_mint.key();
    ctx.accounts.state.quote_account = ctx.accounts.quote_account.key();
    // Do not need to initialize strikes as empty vector.

    // Take tokens that will back the options.
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.base_token_account.to_account_info(),
            to: ctx.accounts.base_token_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, num_tokens_in_period)?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(period_num: u64, option_expiration: u64, subscription_period_end: u64, num_tokens_in_period: u64, so_name: String)]
pub struct Config<'info> {
    /// Does not have to match the authority for the SO State, but it can.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The authority that will be required for issuing and withdrawing.
    /// CHECK: Only used for comparing signers. Will be used in later transactions.
    pub so_authority: AccountInfo<'info>,

    /// State holding all the data for the stake that the staker wants to do.
    #[account(
        init,
        payer = authority,
        seeds = [SO_CONFIG_SEED, so_name.as_bytes(), &period_num.to_be_bytes(), &base_token_mint.key().to_bytes()],
        bump,
        space =
          8 +     // discriminator
          64 +    // so_name
          8 +     // period_num
          32 +    // authority
          8 +     // options_available
          8 +     // option_expiration
          8 +     // subscription_period_end
          1 +     // decimals
          32 +    // base_token_mint 
          32 +    // quote_account 
          8 +     // strikes overhead
          8 * 100 // strikes
    )]
    pub state: Box<Account<'info, State>>,

    /// Where the base tokens are going to be held.
    /// This is not an ATA because this should be separate for each period, not
    /// one owned by this program.
    #[account(
        init,
        payer = authority,
        seeds = [SO_VAULT_SEED, &period_num.to_be_bytes(), &base_token_mint.key().to_bytes()],
        bump,
        token::mint = base_token_mint,
        token::authority = base_token_vault)]
    pub base_token_vault: Box<Account<'info, TokenAccount>>,

    /// Where the tokens are coming from.
    #[account(mut)]
    pub base_token_account: Box<Account<'info, TokenAccount>>,

    /// Saved for later. Not used. TokenAccount instead of AccountInfo in order
    /// to get the anchor type checking.
    pub quote_account: Box<Account<'info, TokenAccount>>,

    /// Mint of base tokens.
    pub base_token_mint: Box<Account<'info, Mint>>,
    /// Mint of quote tokens. Needed for storing the number of decimals.
    pub quote_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Config<'info> {
    pub fn validate_accounts(
        &self,
        _period_num: u64,
        option_expiration: u64,
        subscription_period_end: u64,
        _num_tokens_in_period: u64,
    ) -> Result<()> {
        // Verify the type of token matches input
        assert_keys_eq!(self.base_token_mint, self.base_token_account.mint.key());
        assert_keys_eq!(self.quote_mint, self.quote_account.mint.key());

        // period_num should be increasing, but not necessarily required.
        // num_tokens_in_period is verified by the token program doing the transfer.

        // Make sure it is not already expired.
        check_not_expired!(option_expiration);
        check_not_expired!(subscription_period_end);

        // Cannot verify the token type of the quote_account because it could be
        // something else for downside SO.

        Ok(())
    }
}
