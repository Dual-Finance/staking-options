use anchor_spl::token::Mint;

pub use crate::*;

pub fn name_token(ctx: Context<NameToken>, strike: u64) -> Result<()> {
    // Verify the mint
    check_mint!(ctx, strike, bump);

    // Token name is
    // DUAL-[soName]-[strike converted to lots]
    let strike_quote_atoms_per_lot_float: f64 = strike as f64;
    let strike_quote_tokens_per_lot_float: f64 = strike_quote_atoms_per_lot_float
        / (u64::pow(10, ctx.accounts.state.quote_decimals as u32) as f64);
    let strike_quote_tokens_per_token_float: f64 = strike_quote_tokens_per_lot_float
        / ctx.accounts.state.lot_size as f64
        * (u64::pow(10, ctx.accounts.state.base_decimals as u32) as f64);

    let token_name: String = format!(
        "DUAL-{:.18}-{:.2e}",
        ctx.accounts.state.so_name, strike_quote_tokens_per_token_float
    );
    let symbol: String = "DUAL-SO".to_string();
    msg!("Naming token {}", token_name);

    msg!("Calling into metaplex for {}", ctx.accounts.state.so_name);
    let ix = mpl_token_metadata::instruction::create_metadata_accounts_v3(
        mpl_token_metadata::ID,
        *ctx.accounts.option_mint_metadata_account.key,
        ctx.accounts.option_mint.key(),
        ctx.accounts.option_mint.key(),
        ctx.accounts.payer.key(),
        ctx.accounts.option_mint.key(),
        token_name,
        symbol,
        "https://www.dual.finance/images/token-logos/staking-options.png".to_string(),
        None,
        0,
        true,
        true,
        None,
        None,
        None,
    );

    solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.option_mint_metadata_account.to_account_info(),
            ctx.accounts.option_mint.to_account_info(),
            ctx.accounts.option_mint.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.option_mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        &[&[
            SO_MINT_SEED,
            &ctx.accounts.state.key().to_bytes(),
            &strike.to_be_bytes(),
            &[bump],
        ]],
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(strike: u64)]
pub struct NameToken<'info> {
    #[account(constraint = authority.key() == state.authority.key())]
    pub authority: Signer<'info>,
    pub payer: Signer<'info>,

    /// State holding all the data for the stake that the staker wants to do.
    pub state: Box<Account<'info, State>>,

    /// Mint of base tokens.
    pub option_mint: Box<Account<'info, Mint>>,

    /// CHECK: This is not dangerous. Checked by metaplex
    #[account(mut)]
    pub option_mint_metadata_account: AccountInfo<'info>,

    /// CHECK: This is the metaplex program
    pub token_metadata_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
