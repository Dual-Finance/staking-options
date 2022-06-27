import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import assert from 'assert';
import { PublicKey } from '@solana/web3.js';
import { Provider, Program } from '@project-serum/anchor';
import { StakingOptions } from '../target/types/staking_options';
import {
  getTokenAccount,
  findAssociatedTokenAddress,
  createMint,
  createTokenAccount,
  mintToAccount,
  DEFAULT_MINT_DECIMALS,
  toBeBytes,
} from './utils/index';

const anchor = require('@project-serum/anchor');
const web3 = require('@solana/web3.js');
const process = require('process');

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
      provider.wallet.publicKey,
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
  });

  it('InitStrike Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
  });

  it('Issue Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await issue(OPTIONS_AMOUNT);
  });

  it('AddTokens Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await addTokens();
  });

  it('Exercise Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await issue(OPTIONS_AMOUNT);
    await exercise(OPTIONS_AMOUNT);
  });

  it('Withdraw Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await issue(OPTIONS_AMOUNT);
    await exercise(OPTIONS_AMOUNT);
    try {
      await withdraw();
    } catch (err) {
      console.log(err);
    }
  });
});
