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
// Use only when needed since PDA computation is expensive.
macro_rules! check_state {
    ($ctx:expr) => {
        let (so_state, _so_state_bump) = Pubkey::find_program_address(
            &[
                SO_CONFIG_SEED,
                &$ctx.accounts.state.so_name.as_bytes(),
                &$ctx.accounts.state.period_num.to_be_bytes(),
                &$ctx.accounts.state.base_token_mint.key().to_bytes(),
            ],
            $ctx.program_id,
        );

        assert_keys_eq!($ctx.accounts.state.key(), so_state, InvalidState);
    };
}

// This check verifies that nobody made a fake SO State at a different address.
macro_rules! check_vault {
    ($ctx:expr) => {
        let (expected_vault, _expected_vault_bump) = Pubkey::find_program_address(
            &[
                SO_VAULT_SEED,
                &$ctx.accounts.state.period_num.to_be_bytes(),
                &$ctx.accounts.state.base_token_mint.key().to_bytes(),
            ],
            $ctx.program_id,
        );

        assert_keys_eq!(
            $ctx.accounts.base_token_vault.key(),
            expected_vault,
            InvalidVault
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

// Create the seeds for an SO Vault. Needing when signing a transaction for it.
macro_rules! gen_vault_seeds {
    ($ctx:expr) => {
        &[
            SO_VAULT_SEED,
            &$ctx.accounts.state.period_num.to_be_bytes(),
            &$ctx.accounts.state.base_token_mint.key().to_bytes(),
        ]
    };
}
