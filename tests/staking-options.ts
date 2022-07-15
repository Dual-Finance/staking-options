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

  let projectTokenMint: PublicKey;
  let projectTokenAccount: PublicKey;
  let projectTokenVault: PublicKey;
  let state: PublicKey;
  let usdcMint: PublicKey;
  let usdcAccount: PublicKey;
  let userUsdcAccount: PublicKey;
  let optionMint: PublicKey;
  let userSoAccount: PublicKey;
  let feeUsdcAccount: PublicKey;
  let userProjectTokenAccount: PublicKey;

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

    projectTokenMint = await createMint(provider);
    projectTokenAccount = await createTokenAccount(
      provider,
      projectTokenMint,
      provider.wallet.publicKey,
    );
    await mintToAccount(
      provider,
      projectTokenMint,
      projectTokenAccount,
      numTokensInPeriod,
      provider.wallet.publicKey,
    );
    if (!usdcMint) {
      usdcMint = await createMint(provider);
      usdcAccount = await createTokenAccount(
        provider,
        usdcMint,
        provider.wallet.publicKey,
      );
    }

    const [_state, _stateBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_CONFIG_SEED)),
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_NAME)),
          toBeBytes(periodNum),
          projectTokenMint.toBuffer(),
        ],
        program.programId,
      ));
    state = _state;

    const [_projectTokenVault, _projectTokenVaultBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_VAULT_SEED)),
          toBeBytes(periodNum),
          projectTokenMint.toBuffer(),
        ],
        program.programId,
      ));
    projectTokenVault = _projectTokenVault;

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
          projectTokenVault,
          projectTokenAccount,
          usdcAccount,
          projectTokenMint,
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

  async function issue(amount: number) {
    console.log('Issuing');

    userSoAccount = await createTokenAccount(
      provider,
      optionMint,
      provider.wallet.publicKey,
    );

    await program.rpc.issue(
      new anchor.BN(amount),
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
      projectTokenMint,
      projectTokenAccount,
      numTokensInPeriod,
      provider.wallet.publicKey,
    );

    await program.rpc.addTokens(
      new anchor.BN(OPTIONS_AMOUNT),
      {
        accounts: {
          authority: provider.wallet.publicKey,
          state,
          projectTokenVault,
          projectTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      },
    );
  }

  async function exercise(amount: number) {
    console.log('Exercising');

    userUsdcAccount = await createTokenAccount(
      provider,
      usdcMint,
      provider.wallet.publicKey,
    );
    await mintToAccount(
      provider,
      usdcMint,
      userUsdcAccount,
      OPTIONS_AMOUNT * STRIKE * DEFAULT_MINT_DECIMALS,
      provider.wallet.publicKey,
    );
    feeUsdcAccount = await createTokenAccount(
      provider,
      usdcMint,
      new PublicKey('A9YWU67LStgTAYJetbXND2AWqEcvk7FqYJM9nF3VmVpv'),
    );
    userProjectTokenAccount = await createTokenAccount(
      provider,
      projectTokenMint,
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
          userUsdcAccount,
          projectUsdcAccount: usdcAccount,
          feeUsdcAccount,
          projectTokenVault,
          userProjectTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      },
    );
  }

  async function withdraw() {
    console.log('Withdrawing');
    console.log('Sleeping til options expire');
    await new Promise((r) => setTimeout(r, 100000));

    await program.rpc.withdraw(
      {
        accounts: {
          authority: provider.wallet.publicKey,
          state,
          projectTokenVault,
          projectTokenAccount,
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
    assert.equal(stateObj.decimals, DEFAULT_MINT_DECIMALS);
    assert.equal(stateObj.projectTokenMint.toBase58(), projectTokenMint.toBase58());
    assert.equal(stateObj.usdcAccount.toBase58(), usdcAccount.toBase58());
    assert.equal(stateObj.strikes.length, 0);

    // Verify the tokens are stored.
    const projectTokenVaultAccount = await getTokenAccount(
      provider,
      projectTokenVault,
    );
    assert.equal(projectTokenVaultAccount.amount.toNumber(), numTokensInPeriod);
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
    await issue(OPTIONS_AMOUNT);

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

    const projectTokenVaultAccount = await getTokenAccount(
      provider,
      projectTokenVault,
    );
    assert.equal(projectTokenVaultAccount.amount.toNumber(), numTokensInPeriod + OPTIONS_AMOUNT);
  });

  it('Exercise Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await issue(OPTIONS_AMOUNT);
    await exercise(OPTIONS_AMOUNT);

    const userProjectTokenAccountAccount = await getTokenAccount(
      provider,
      userProjectTokenAccount,
    );
    assert.equal(userProjectTokenAccountAccount.amount.toNumber(), OPTIONS_AMOUNT);
  });

  it('Withdraw Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await issue(OPTIONS_AMOUNT);
    await exercise(OPTIONS_AMOUNT);
    await withdraw();

    const userProjectTokenAccountAccount = await getTokenAccount(
      provider,
      projectTokenAccount,
    );
    assert.equal(
      userProjectTokenAccountAccount.amount.toNumber(),
      numTokensInPeriod - OPTIONS_AMOUNT,
    );
  });

  it('Rollover Success', async () => {
    await configureSO();

    // Rollover again
    await mintToAccount(
      provider,
      projectTokenMint,
      projectTokenAccount,
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
          projectTokenMint.toBuffer(),
        ],
        program.programId,
      ));

    const [newProjectTokenVault, _projectTokenVaultBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(SO_VAULT_SEED)),
          toBeBytes(newPeriodNum),
          projectTokenMint.toBuffer(),
        ],
        program.programId,
      ));

    // Wait for the old one to expire.
    await new Promise((r) => setTimeout(r, 100000));

    optionExpiration = Math.floor(Date.now() / 1000 + 100);
    subscriptionPeriodEnd = optionExpiration;

    console.log('Config again');
    await program.rpc.config(
      new anchor.BN(newPeriodNum),
      new anchor.BN(optionExpiration),
      new anchor.BN(subscriptionPeriodEnd),
      new anchor.BN(numTokensInPeriod),
      {
        accounts: {
          authority: provider.wallet.publicKey,
          soAuthority: provider.wallet.publicKey,
          state: newState,
          projectTokenVault: newProjectTokenVault,
          projectTokenAccount,
          usdcAccount,
          projectTokenMint,
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
          oldProjectTokenVault: projectTokenVault,
          newProjectTokenVault,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      },
    );

    const newProjectTokenVaultAccount = await getTokenAccount(
      provider,
      newProjectTokenVault,
    );
    assert.equal(
      newProjectTokenVaultAccount.amount.toNumber(),
      2 * numTokensInPeriod,
    );
  });
});
