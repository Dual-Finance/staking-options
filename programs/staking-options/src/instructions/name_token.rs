use anchor_spl::token::{Mint, Token};

pub use crate::*;

pub fn name_token(
    ctx: Context<NameToken>,
    strike: u64
) -> Result<()> {
    // Verify the mint
    check_mint!(ctx, strike, bump);

    msg!("Calling into metaplex for {}", ctx.accounts.state.so_name);
    let ix = mpl_token_metadata::instruction::create_metadata_accounts_v3(
        mpl_token_metadata::ID,
        *ctx.accounts.option_mint_metadata_account.key,
        ctx.accounts.option_mint.key(),
        ctx.accounts.option_mint.key(),
        ctx.accounts.authority.key(),
        ctx.accounts.option_mint.key(),
        // TODO: Truncate SO Name and include lot size and baseMint
        "DUAL-SO-".to_string() + &ctx.accounts.state.so_name,
        "DUAL-SO-".to_string() + &ctx.accounts.state.so_name,
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
            ctx.accounts.authority.to_account_info(),
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
    #[account(mut, constraint = authority.key() == state.authority.key())]
    pub authority: Signer<'info>,

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
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
