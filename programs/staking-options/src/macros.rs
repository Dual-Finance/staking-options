macro_rules! check_not_expired {
    ($expiration:expr) => {
        require!(
            Clock::get().unwrap().unix_timestamp as u64 <= $expiration,
            SOErrorCode::Expired
        );
    };
}

macro_rules! check_expired {
    ($expiration:expr) => {
        require!(
            Clock::get().unwrap().unix_timestamp as u64 > $expiration,
            SOErrorCode::NotYetExpired
        );
    };
}

// This check verifies that nobody made a fake SO State at a different address.
macro_rules! check_mint {
    ($ctx:expr, $strike:expr, $bump:ident) => {
        let (expected_mint, mint_bump) = Pubkey::find_program_address(
            &[
                SO_MINT_SEED,
                &$ctx.accounts.state.key().to_bytes(),
                &$strike.to_be_bytes(),
            ],
            $ctx.program_id,
        );

        require_keys_eq!(
            $ctx.accounts.option_mint.key(),
            expected_mint,
            SOErrorCode::InvalidMint
        );
        let $bump = mint_bump;
    };
}

macro_rules! check_reverse_mint {
    ($ctx:expr, $strike:expr, $bump:ident) => {
        let (expected_mint, mint_bump) = Pubkey::find_program_address(
            &[
                SO_REVERSE_MINT_SEED,
                &$ctx.accounts.state.key().to_bytes(),
                &$strike.to_be_bytes(),
            ],
            $ctx.program_id,
        );

        require_keys_eq!(
            $ctx.accounts.reverse_option_mint.key(),
            expected_mint,
            SOErrorCode::InvalidMint
        );
        let $bump = mint_bump;
    };
}
