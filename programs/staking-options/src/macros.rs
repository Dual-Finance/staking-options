macro_rules! check_not_expired {
    ($expiration:expr) => {
        invariant!(
            Clock::get().unwrap().unix_timestamp as u64 <= $expiration,
            NotYetExpired
        );
    };
}

macro_rules! check_expired {
    ($expiration:expr) => {
        invariant!(
            Clock::get().unwrap().unix_timestamp as u64 > $expiration,
            Expired
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
                &$ctx.accounts.state.period_num.to_be_bytes(),
                &$ctx.accounts.state.project_token_mint.key().to_bytes(),
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
                &$ctx.accounts.state.project_token_mint.key().to_bytes(),
            ],
            $ctx.program_id,
        );

        assert_keys_eq!(
            $ctx.accounts.project_token_vault.key(),
            expected_vault,
            InvalidVault
        );
    };
}

// Create the seeds for an SO Vault. Needing when signing a transaction for it.
macro_rules! gen_vault_seeds {
    ($ctx:expr) => {
        &[
            SO_VAULT_SEED,
            &$ctx.accounts.state.period_num.to_be_bytes(),
            &$ctx.accounts.state.project_token_mint.key().to_bytes(),
        ]
    };
}
