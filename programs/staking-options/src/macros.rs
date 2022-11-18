macro_rules! check_not_expired {
    ($expiration:expr) => {
        invariant!(
            Clock::get().unwrap().unix_timestamp as u64 <= $expiration,
            Expired
        );
    };
}

macro_rules! check_expired {
    ($expiration:expr) => {
        invariant!(
            Clock::get().unwrap().unix_timestamp as u64 > $expiration,
            NotYetExpired
        );
    };
}

// This check verifies that nobody made a fake SO State at a different address.
macro_rules! check_mint {
    ($ctx:expr, $strike:expr) => {
        let (expected_mint, _expected_mint_bump) = Pubkey::find_program_address(
            &[
                SO_MINT_SEED,
                &$ctx.accounts.state.key().to_bytes(),
                &$strike.to_be_bytes(),
            ],
            $ctx.program_id,
        );

        assert_keys_eq!($ctx.accounts.option_mint.key(), expected_mint, InvalidMint);
    };
}
