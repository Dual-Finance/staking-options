import {
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import assert from 'assert';
import { PublicKey } from '@solana/web3.js';
import { Provider, Program } from '@project-serum/anchor';
import { StakingOptions } from '../target/types/staking_options';
import {
  createMint,
  createTokenAccount,
  getTokenAccount,
  mintToAccount,
  DEFAULT_MINT_DECIMALS,
  toBeBytes,
} from './utils/index';

const anchor = require('@project-serum/anchor');

describe('staking-options', () => {
  anchor.setProvider(anchor.Provider.env());
  const provider: Provider = anchor.Provider.env();
  const program = anchor.workspace.StakingOptions as Program<StakingOptions>;

  const SO_CONFIG_SEED = 'so-config';
  const SO_VAULT_SEED = 'so-vault';
  const SO_MINT_SEED = 'so-mint';

  let baseMint: PublicKey;
  let baseAccount: PublicKey;
  let baseVault: PublicKey;
  let state: PublicKey;
  let quoteMint: PublicKey;
  let quoteAccount: PublicKey;
  let userQuoteAccount: PublicKey;
  let optionMint: PublicKey;
  let userSoAccount: PublicKey;
  let feeQuoteAccount: PublicKey;
  let userBaseAccount: PublicKey;

  let optionExpiration: number;
  let subscriptionPeriodEnd: number;
  const numTokens: number = 1_000_000_000;
  const STRIKE: number = 1_000;
  const OPTIONS_AMOUNT: number = 1_000;
  const LOT_SIZE: number = 1_000_000;
  const SO_NAME: string = 'SO';

  async function configureSO() {
    console.log('Configuring SO');

    optionExpiration = Math.floor(Date.now() / 1000 + 100);
    subscriptionPeriodEnd = optionExpiration;

    baseMint = await createMint(provider);
    baseAccount = await createTokenAccount(
      provider,
      baseMint,
      provider.wallet.publicKey,
    );
    await mintToAccount(
      provider,
      baseMint,
      baseAccount,
      numTokens,
      provider.wallet.publicKey,
    );
    if (!quoteMint) {
      quoteMint = await createMint(provider);
      quoteAccount = await createTokenAccount(
        provider,
        quoteMint,
        provider.wallet.publicKey,
      );
    }

    const [_state, _stateBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_CONFIG_SEED)),
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_NAME)),
          baseMint.toBuffer(),
        ],
        program.programId,
      ));
    state = _state;

    const [_baseVault, _baseVaultBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_VAULT_SEED)),
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_NAME)),
          baseMint.toBuffer(),
        ],
        program.programId,
      ));
    baseVault = _baseVault;

    await program.rpc.config(
      new anchor.BN(optionExpiration),
      new anchor.BN(subscriptionPeriodEnd),
      new anchor.BN(numTokens),
      new anchor.BN(LOT_SIZE),
      SO_NAME,
      {
        accounts: {
          authority: provider.wallet.publicKey,
          soAuthority: provider.wallet.publicKey,
          state,
          baseVault,
          baseAccount,
          quoteAccount,
          quoteMint,
          baseMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
      },
    );
  }

  async function initStrike(strike: number) {
    console.log('Init Strike');

    const [_optionMint, _optionMintBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_MINT_SEED)),
          state.toBuffer(),
          toBeBytes(strike),
        ],
        program.programId,
      ));
    optionMint = _optionMint;

    await program.rpc.initStrike(
      new anchor.BN(strike),
      {
        accounts: {
          authority: provider.wallet.publicKey,
          state,
          optionMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
      },
    );
  }

  async function issue(amount: number, strike: number) {
    console.log('Issuing');

    userSoAccount = await createTokenAccount(
      provider,
      optionMint,
      provider.wallet.publicKey,
    );

    await program.rpc.issue(
      new anchor.BN(amount),
      new anchor.BN(strike),
      {
        accounts: {
          authority: provider.wallet.publicKey,
          state,
          optionMint,
          userSoAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      },
    );
  }

  async function addTokens() {
    console.log('Adding tokens');
    // Top off the number of available tokens for the LSO.
    await mintToAccount(
      provider,
      baseMint,
      baseAccount,
      numTokens,
      provider.wallet.publicKey,
    );

    await program.rpc.addTokens(
      new anchor.BN(OPTIONS_AMOUNT),
      {
        accounts: {
          authority: provider.wallet.publicKey,
          state,
          baseVault,
          baseAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      },
    );
  }

  async function exercise(amount: number) {
    console.log('Exercising');

    userQuoteAccount = await createTokenAccount(
      provider,
      quoteMint,
      provider.wallet.publicKey,
    );
    await mintToAccount(
      provider,
      quoteMint,
      userQuoteAccount,
      OPTIONS_AMOUNT * STRIKE * DEFAULT_MINT_DECIMALS,
      provider.wallet.publicKey,
    );
    feeQuoteAccount = await createTokenAccount(
      provider,
      quoteMint,
      new PublicKey('CZqTD3b3oQw8cDK4CBddpKF6epA1fR36GBbvU5VBt2Dz'),
    );
    userBaseAccount = await createTokenAccount(
      provider,
      baseMint,
      provider.wallet.publicKey,
    );

    await program.rpc.exercise(
      new anchor.BN(amount),
      new anchor.BN(STRIKE),
      {
        accounts: {
          authority: provider.wallet.publicKey,
          state,
          userSoAccount,
          optionMint,
          userQuoteAccount,
          projectQuoteAccount: quoteAccount,
          feeQuoteAccount,
          baseVault,
          userBaseAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      },
    );
  }

  async function withdraw() {
    console.log('Withdrawing');
    console.log('Sleeping til options expire');
    await new Promise((r) => setTimeout(r, 100_000));

    await program.rpc.withdraw(
      {
        accounts: {
          authority: provider.wallet.publicKey,
          state,
          baseVault,
          baseAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      },
    );
  }

  it('Config Success', async () => {
    await configureSO();

    // Verify the State.
    const stateObj = await program.account.state.fetch(state);
    assert.equal(stateObj.authority.toBase58(), provider.wallet.publicKey.toBase58());
    assert.equal(stateObj.optionsAvailable.toNumber(), numTokens);
    assert.equal(stateObj.optionExpiration.toNumber(), optionExpiration);
    assert.equal(stateObj.subscriptionPeriodEnd.toNumber(), subscriptionPeriodEnd);
    assert.equal(stateObj.baseDecimals, DEFAULT_MINT_DECIMALS);
    assert.equal(stateObj.quoteDecimals, DEFAULT_MINT_DECIMALS);
    assert.equal(stateObj.baseMint.toBase58(), baseMint.toBase58());
    assert.equal(stateObj.quoteAccount.toBase58(), quoteAccount.toBase58());
    assert.equal(stateObj.strikes.length, 0);

    // Verify the tokens are stored.
    const baseVaultAccount = await getTokenAccount(
      provider,
      baseVault,
    );
    assert.equal(baseVaultAccount.amount.toNumber(), numTokens);
  });

  it('InitStrike Success', async () => {
    await configureSO();
    await initStrike(STRIKE);

    // Verify the strike exists in the state.
    const stateObj = await program.account.state.fetch(state);
    assert.equal(stateObj.strikes[0].toNumber(), STRIKE);
  });

  it('Issue Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await issue(OPTIONS_AMOUNT, STRIKE);

    const userSoAccountAccount = await getTokenAccount(
      provider,
      userSoAccount,
    );
    assert.equal(userSoAccountAccount.amount.toNumber(), OPTIONS_AMOUNT);
  });

  it('AddTokens Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await addTokens();

    const baseVaultAccount = await getTokenAccount(
      provider,
      baseVault,
    );
    assert.equal(baseVaultAccount.amount.toNumber(), numTokens + OPTIONS_AMOUNT);
  });

  it('Exercise Success', async () => {
    try {
      await configureSO();
      await initStrike(STRIKE);
      await issue(OPTIONS_AMOUNT, STRIKE);
      await exercise(OPTIONS_AMOUNT);
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const userBaseAccountAccount = await getTokenAccount(
      provider,
      userBaseAccount,
    );
    assert.equal(userBaseAccountAccount.amount.toNumber(), OPTIONS_AMOUNT * LOT_SIZE);
  });

  it('Withdraw Success', async () => {
    try {
      await configureSO();
      await initStrike(STRIKE);
      await issue(OPTIONS_AMOUNT, STRIKE);
      await exercise(OPTIONS_AMOUNT);
      await withdraw();
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const userBaseAccountAccount = await getTokenAccount(
      provider,
      baseAccount,
    );
    assert.equal(
      userBaseAccountAccount.amount.toNumber(),
      numTokens - OPTIONS_AMOUNT * LOT_SIZE,
    );
  });
});
