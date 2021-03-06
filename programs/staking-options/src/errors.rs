use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The mint in the SO state did not match the token type being received")]
    WrongMint,
    #[msg("Expired")]
    Expired,
    #[msg("NotYetExpired")]
    NotYetExpired,
    #[msg("State did not match")]
    InvalidState,
    #[msg("Vault did not match")]
    InvalidVault,
    #[msg("Mint did not match")]
    InvalidMint,
    #[msg("Account receiving fees does not match")]
    IncorrectFeeAccount,
    #[msg("Not enough tokens to issue the SO")]
    NotEnoughTokens,
}
