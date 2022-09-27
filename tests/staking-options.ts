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
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const provider: Provider = anchor.Provider.env();
  const program = anchor.workspace.StakingOptions as Program<StakingOptions>;

  const SO_CONFIG_SEED = 'so-config';
  const SO_VAULT_SEED = 'so-vault';
  const SO_MINT_SEED = 'so-mint';

  let baseTokenMint: PublicKey;
  let baseTokenAccount: PublicKey;
  let baseTokenVault: PublicKey;
  let state: PublicKey;
  let quoteMint: PublicKey;
  let quoteAccount: PublicKey;
  let userQuoteAccount: PublicKey;
  let optionMint: PublicKey;
  let userSoAccount: PublicKey;
  let feeQuoteAccount: PublicKey;
  let userBaseTokenAccount: PublicKey;

  let optionExpiration: number;
  let subscriptionPeriodEnd: number;
  const periodNum: number = 0;
  const numTokensInPeriod: number = 1_000_000_000;
  const STRIKE: number = 1_000;
  const OPTIONS_AMOUNT: number = 1_000;
  const SO_NAME: string = 'SO';

  async function configureSO() {
    console.log('Configuring SO');

    optionExpiration = Math.floor(Date.now() / 1000 + 100);
    subscriptionPeriodEnd = optionExpiration;

    baseTokenMint = await createMint(provider);
    baseTokenAccount = await createTokenAccount(
      provider,
      baseTokenMint,
      provider.wallet.publicKey,
    );
    await mintToAccount(
      provider,
      baseTokenMint,
      baseTokenAccount,
      numTokensInPeriod,
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
          toBeBytes(periodNum),
          baseTokenMint.toBuffer(),
        ],
        program.programId,
      ));
    state = _state;

    const [_baseTokenVault, _baseTokenVaultBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_VAULT_SEED)),
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_NAME)),
          toBeBytes(periodNum),
          baseTokenMint.toBuffer(),
        ],
        program.programId,
      ));
    baseTokenVault = _baseTokenVault;

    await program.rpc.config(
      new anchor.BN(periodNum),
      new anchor.BN(optionExpiration),
      new anchor.BN(subscriptionPeriodEnd),
      new anchor.BN(numTokensInPeriod),
      SO_NAME,
      {
        accounts: {
          authority: provider.wallet.publicKey,
          soAuthority: provider.wallet.publicKey,
          state,
          baseTokenVault,
          baseTokenAccount,
          quoteAccount,
          quoteMint,
          baseTokenMint,
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
      baseTokenMint,
      baseTokenAccount,
      numTokensInPeriod,
      provider.wallet.publicKey,
    );

    await program.rpc.addTokens(
      new anchor.BN(OPTIONS_AMOUNT),
      {
        accounts: {
          authority: provider.wallet.publicKey,
          state,
          baseTokenVault,
          baseTokenAccount,
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
      new PublicKey('A9YWU67LStgTAYJetbXND2AWqEcvk7FqYJM9nF3VmVpv'),
    );
    userBaseTokenAccount = await createTokenAccount(
      provider,
      baseTokenMint,
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
          baseTokenVault,
          userBaseTokenAccount,
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
          baseTokenVault,
          baseTokenAccount,
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
    assert.equal(stateObj.periodNum.toNumber(), 0);
    assert.equal(stateObj.authority.toBase58(), provider.wallet.publicKey.toBase58());
    assert.equal(stateObj.optionsAvailable.toNumber(), numTokensInPeriod);
    assert.equal(stateObj.optionExpiration.toNumber(), optionExpiration);
    assert.equal(stateObj.subscriptionPeriodEnd.toNumber(), subscriptionPeriodEnd);
    assert.equal(stateObj.baseDecimals, DEFAULT_MINT_DECIMALS);
    assert.equal(stateObj.quoteDecimals, DEFAULT_MINT_DECIMALS);
    assert.equal(stateObj.baseTokenMint.toBase58(), baseTokenMint.toBase58());
    assert.equal(stateObj.quoteAccount.toBase58(), quoteAccount.toBase58());
    assert.equal(stateObj.strikes.length, 0);

    // Verify the tokens are stored.
    const baseTokenVaultAccount = await getTokenAccount(
      provider,
      baseTokenVault,
    );
    assert.equal(baseTokenVaultAccount.amount.toNumber(), numTokensInPeriod);
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

    const baseTokenVaultAccount = await getTokenAccount(
      provider,
      baseTokenVault,
    );
    assert.equal(baseTokenVaultAccount.amount.toNumber(), numTokensInPeriod + OPTIONS_AMOUNT);
  });

  it('Exercise Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await issue(OPTIONS_AMOUNT, STRIKE);
    await exercise(OPTIONS_AMOUNT);

    const userBaseTokenAccountAccount = await getTokenAccount(
      provider,
      userBaseTokenAccount,
    );
    assert.equal(userBaseTokenAccountAccount.amount.toNumber(), OPTIONS_AMOUNT);
  });

  it('Withdraw Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await issue(OPTIONS_AMOUNT, STRIKE);
    await exercise(OPTIONS_AMOUNT);
    await withdraw();

    const userBaseTokenAccountAccount = await getTokenAccount(
      provider,
      baseTokenAccount,
    );
    assert.equal(
      userBaseTokenAccountAccount.amount.toNumber(),
      numTokensInPeriod - OPTIONS_AMOUNT,
    );
  });

  it('Rollover Success', async () => {
    await configureSO();

    // Rollover again
    await mintToAccount(
      provider,
      baseTokenMint,
      baseTokenAccount,
      numTokensInPeriod,
      provider.wallet.publicKey,
    );
    const newPeriodNum = 1;

    const [newState, _stateBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_CONFIG_SEED)),
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_NAME)),
          toBeBytes(newPeriodNum),
          baseTokenMint.toBuffer(),
        ],
        program.programId,
      ));

    const [newBaseTokenVault, _baseTokenVaultBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_VAULT_SEED)),
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_NAME)),
          toBeBytes(newPeriodNum),
          baseTokenMint.toBuffer(),
        ],
        program.programId,
      ));

    // Wait for the old one to expire.
    await new Promise((r) => setTimeout(r, 100_000));

    optionExpiration = Math.floor(Date.now() / 1000 + 2000);
    subscriptionPeriodEnd = optionExpiration;

    console.log('Config again');
    await program.rpc.config(
      new anchor.BN(newPeriodNum),
      new anchor.BN(optionExpiration),
      new anchor.BN(subscriptionPeriodEnd),
      new anchor.BN(numTokensInPeriod),
      SO_NAME,
      {
        accounts: {
          authority: provider.wallet.publicKey,
          soAuthority: provider.wallet.publicKey,
          state: newState,
          baseTokenVault: newBaseTokenVault,
          baseTokenAccount,
          quoteAccount,
          quoteMint,
          baseTokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
      },
    );

    console.log('Rolling over');
    await program.rpc.rollover(
      {
        accounts: {
          authority: provider.wallet.publicKey,
          oldState: state,
          newState,
          oldBaseTokenVault: baseTokenVault,
          newBaseTokenVault,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      },
    );

    const newBaseTokenVaultAccount = await getTokenAccount(
      provider,
      newBaseTokenVault,
    );
    assert.equal(
      newBaseTokenVaultAccount.amount.toNumber(),
      2 * numTokensInPeriod,
    );
  });
});
