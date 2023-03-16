use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

pub use crate::*;

pub fn config_v2(
    ctx: Context<ConfigV2>,
    option_expiration: u64,
    subscription_period_end: u64,
    num_tokens: u64,
    lot_size: u64,
    so_name: String,
) -> Result<()> {
    // Verify the SO name is a reasonable length.
    require!(so_name.len() < 32, SOErrorCode::InvalidName);

    // Fill out the State
    ctx.accounts.state.so_name = so_name;
    ctx.accounts.state.authority = ctx.accounts.so_authority.key();

    let optional_issue_authority = &mut ctx.accounts.issue_authority;
    if let Some(unwrapped_issue_authority) = optional_issue_authority {
        ctx.accounts.state.issue_authority = unwrapped_issue_authority.key();
    }
    ctx.accounts.state.options_available = num_tokens;
    ctx.accounts.state.option_expiration = option_expiration;
    ctx.accounts.state.subscription_period_end = subscription_period_end;
    ctx.accounts.state.base_decimals = ctx.accounts.base_mint.decimals;
    ctx.accounts.state.quote_decimals = ctx.accounts.quote_mint.decimals;
    ctx.accounts.state.base_mint = ctx.accounts.base_mint.key();
    ctx.accounts.state.quote_mint = ctx.accounts.quote_mint.key();
    ctx.accounts.state.quote_account = ctx.accounts.quote_account.key();
    ctx.accounts.state.lot_size = lot_size;
    // Do not need to initialize strikes as empty vector.

    ctx.accounts.state.state_bump = *ctx.bumps.get("state").unwrap();
    ctx.accounts.state.vault_bump = *ctx.bumps.get("base_vault").unwrap();

    // Take tokens that will back the options.
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.base_account.to_account_info(),
            to: ctx.accounts.base_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, num_tokens)?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(option_expiration: u64, subscription_period_end: u64, num_tokens: u64, lot_size: u64, so_name: String)]
pub struct ConfigV2<'info> {
    /// Does not have to match the authority for the SO State, but it can.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The authority that will be required init strike and withdrawing.
    /// CHECK: Only used for comparing signers. Will be used in later transactions.
    pub so_authority: AccountInfo<'info>,

    /// An authority that can be used for issuing tokens. Should be a PDA.
    /// CHECK: Only used for comparing signers. Will be used in later transactions.
    pub issue_authority: Option<AccountInfo<'info>>,

    /// State holding all the data for the stake that the staker wants to do.
    #[account(
        init,
        payer = authority,
        seeds = [SO_CONFIG_SEED, so_name.as_bytes(), &base_mint.key().to_bytes()],
        bump,
        space =
          8 +       // discriminator
          64 +      // so_name
          32 +      // authority
          8 +       // options_available
          8 +       // option_expiration
          8 +       // subscription_period_end
          8 +       // decimals
          32 +      // base_mint 
          32 +      // quote_mint 
          32 +      // quote_account 
          8 +       // lot size
          1 + 1 +   // bumps
          8 +       // strikes overhead
          8 * 100 + // strikes
          32 +      // issue_authority
          68        // unused bytes for future upgrades
    )]
    pub state: Box<Account<'info, State>>,

    /// Where the base tokens are going to be held.
    #[account(
        init,
        payer = authority,
        seeds = [SO_VAULT_SEED, so_name.as_bytes(), &base_mint.key().to_bytes()],
        bump,
        token::mint = base_mint,
        token::authority = base_vault)]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    /// Where the tokens are coming from.
    #[account(mut)]
    pub base_account: Box<Account<'info, TokenAccount>>,

    /// Saved for later. Not used. TokenAccount instead of AccountInfo in order
    /// to get the anchor type checking.
    pub quote_account: Box<Account<'info, TokenAccount>>,

    /// Mint of base tokens.
    pub base_mint: Box<Account<'info, Mint>>,
    /// Mint of quote tokens. Needed for storing the number of decimals.
    pub quote_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> ConfigV2<'info> {
    pub fn validate_accounts(
        &self,
        option_expiration: u64,
        subscription_period_end: u64,
    ) -> Result<()> {
        // Verify the type of token matches input
        require_keys_eq!(self.base_mint.key(), self.base_account.mint.key());
        require_keys_eq!(self.quote_mint.key(), self.quote_account.mint.key());

        // num_tokens is verified by the token program doing the transfer.

        // Make sure it is not already expired.
        check_not_expired!(option_expiration);
        check_not_expired!(subscription_period_end);

        require!(subscription_period_end <= option_expiration, SOErrorCode::InvalidExpiration);

        // Cannot verify the token type of the quote_account because it could be
        // something else for downside SO.

        Ok(())
    }
}

pub fn config(
    ctx: Context<Config>,
    option_expiration: u64,
    subscription_period_end: u64,
    num_tokens: u64,
    lot_size: u64,
    so_name: String,
) -> Result<()> {
    // Verify the SO name is a reasonable length.
    require!(so_name.len() < 32, SOErrorCode::InvalidName);

    // Fill out the State
    ctx.accounts.state.so_name = so_name;
    ctx.accounts.state.authority = ctx.accounts.so_authority.key();

    ctx.accounts.state.options_available = num_tokens;
    ctx.accounts.state.option_expiration = option_expiration;
    ctx.accounts.state.subscription_period_end = subscription_period_end;
    ctx.accounts.state.base_decimals = ctx.accounts.base_mint.decimals;
    ctx.accounts.state.quote_decimals = ctx.accounts.quote_mint.decimals;
    ctx.accounts.state.base_mint = ctx.accounts.base_mint.key();
    ctx.accounts.state.quote_mint = ctx.accounts.quote_mint.key();
    ctx.accounts.state.quote_account = ctx.accounts.quote_account.key();
    ctx.accounts.state.lot_size = lot_size;
    // Do not need to initialize strikes as empty vector.

    ctx.accounts.state.state_bump = *ctx.bumps.get("state").unwrap();
    ctx.accounts.state.vault_bump = *ctx.bumps.get("base_vault").unwrap();

    // Take tokens that will back the options.
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.base_account.to_account_info(),
            to: ctx.accounts.base_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, num_tokens)?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(option_expiration: u64, subscription_period_end: u64, num_tokens: u64, lot_size: u64, so_name: String)]
pub struct Config<'info> {
    /// Does not have to match the authority for the SO State, but it can.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The authority that will be required init strike and withdrawing.
    /// CHECK: Only used for comparing signers. Will be used in later transactions.
    pub so_authority: AccountInfo<'info>,

    /// State holding all the data for the stake that the staker wants to do.
    #[account(
        init,
        payer = authority,
        seeds = [SO_CONFIG_SEED, so_name.as_bytes(), &base_mint.key().to_bytes()],
        bump,
        space =
          8 +       // discriminator
          64 +      // so_name
          32 +      // authority
          8 +       // options_available
          8 +       // option_expiration
          8 +       // subscription_period_end
          8 +       // decimals
          32 +      // base_mint 
          32 +      // quote_mint 
          32 +      // quote_account 
          8 +       // lot size
          1 + 1 +   // bumps
          8 +       // strikes overhead
          8 * 100 + // strikes
          32 +      // issue_authority
          68        // unused bytes for future upgrades
    )]
    pub state: Box<Account<'info, State>>,

    /// Where the base tokens are going to be held.
    #[account(
        init,
        payer = authority,
        seeds = [SO_VAULT_SEED, so_name.as_bytes(), &base_mint.key().to_bytes()],
        bump,
        token::mint = base_mint,
        token::authority = base_vault)]
    pub base_vault: Box<Account<'info, TokenAccount>>,

    /// Where the tokens are coming from.
    #[account(mut)]
    pub base_account: Box<Account<'info, TokenAccount>>,

    /// Saved for later. Not used. TokenAccount instead of AccountInfo in order
    /// to get the anchor type checking.
    pub quote_account: Box<Account<'info, TokenAccount>>,

    /// Mint of base tokens.
    pub base_mint: Box<Account<'info, Mint>>,
    /// Mint of quote tokens. Needed for storing the number of decimals.
    pub quote_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Config<'info> {
    pub fn validate_accounts(
        &self,
        option_expiration: u64,
        subscription_period_end: u64,
    ) -> Result<()> {
        // Verify the type of token matches input
        require_keys_eq!(self.base_mint.key(), self.base_account.mint.key());
        require_keys_eq!(self.quote_mint.key(), self.quote_account.mint.key());

        // num_tokens is verified by the token program doing the transfer.

        // Make sure it is not already expired.
        check_not_expired!(option_expiration);
        check_not_expired!(subscription_period_end);

        require!(subscription_period_end <= option_expiration, SOErrorCode::InvalidExpiration);

        // Cannot verify the token type of the quote_account because it could be
        // something else for downside SO.

        Ok(())
    }
}
