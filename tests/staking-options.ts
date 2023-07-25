import assert from 'assert';
import { PublicKey, Transaction } from '@solana/web3.js';
import {
  AnchorProvider, Program, BN, workspace, web3, utils,
} from '@coral-xyz/anchor';
import { StakingOptions as SO } from '@dual-finance/staking-options';
import {
  createAssociatedTokenAccount,
} from '@project-serum/associated-token';
import { Metaplex } from '@metaplex-foundation/js';
import { getAccount, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { StakingOptions } from '../target/types/staking_options';
import {
  DEFAULT_MINT_DECIMALS,
  createMint,
  createTokenAccount,
  mintToAccount,
  toBeBytes,
} from './utils/utils';

const anchor = require('@coral-xyz/anchor');

describe('staking-options', () => {
  const provider: AnchorProvider = AnchorProvider.local();
  anchor.setProvider(provider);
  const program: Program<StakingOptions> = workspace.StakingOptions as Program<StakingOptions>;

  const so = new SO(provider.connection.rpcEndpoint);

  let baseMint: PublicKey;
  let baseAccount: PublicKey;
  let baseVault: PublicKey;
  let state: PublicKey;
  let quoteMint: PublicKey;
  let quoteAccount: PublicKey;
  let userQuoteAccount: PublicKey;
  let optionMint: PublicKey;
  let userSoAccount: PublicKey;
  let userReverseSoAccount: PublicKey;
  let userBaseAccount: PublicKey;

  let optionExpiration: number;
  let subscriptionPeriodEnd: number;
  const numTokens: number = 1_000_000_000;
  const STRIKE: number = 1_000;
  const OPTIONS_AMOUNT: number = 10_000_000;
  const LOT_SIZE: number = 1_000_000;
  let SO_NAME: string = 'SO_staking_options_SO';
  const OPTION_EXPIRATION_DELAY_SEC = 100;

  async function configureSO() {
    console.log('Configuring SO');

    subscriptionPeriodEnd = Math.floor(Date.now() / 1_000 + OPTION_EXPIRATION_DELAY_SEC / 2);
    optionExpiration = Math.floor(Date.now() / 1_000 + OPTION_EXPIRATION_DELAY_SEC);
    console.log(`subscriptionPeriodEnd: ${subscriptionPeriodEnd}, optionExpiration: ${optionExpiration}`);

    // Use a new BaseMint every run so that there is a new State.
    baseMint = await createMint(provider, undefined);
    baseAccount = await createTokenAccount(
      provider,
      baseMint,
      provider.wallet.publicKey,
    );
    await mintToAccount(
      provider,
      baseMint,
      baseAccount,
      new BN(numTokens),
      provider.wallet.publicKey,
    );
    if (!quoteMint) {
      quoteMint = await createMint(provider, undefined);
      quoteAccount = await createTokenAccount(
        provider,
        quoteMint,
        provider.wallet.publicKey,
      );
    }

    state = await so.state(SO_NAME, baseMint);
    baseVault = await so.baseVault(SO_NAME, baseMint);

    const instr = await so.createConfigInstruction(
      optionExpiration,
      subscriptionPeriodEnd,
      new BN(numTokens),
      new BN(LOT_SIZE),
      SO_NAME,
      provider.wallet.publicKey,
      baseMint,
      baseAccount,
      quoteMint,
      quoteAccount,
    );

    const tx = new Transaction();
    tx.add(instr);
    await provider.sendAndConfirm(tx);
  }

  async function configureV2() {
    console.log('Configuring SO v2');

    subscriptionPeriodEnd = Math.floor(Date.now() / 1_000 + OPTION_EXPIRATION_DELAY_SEC / 2);
    optionExpiration = Math.floor(Date.now() / 1_000 + OPTION_EXPIRATION_DELAY_SEC);
    console.log(`subscriptionPeriodEnd: ${subscriptionPeriodEnd}, optionExpiration: ${optionExpiration}`);

    // Use a new BaseMint every run so that there is a new State.
    baseMint = await createMint(provider, undefined);
    baseAccount = await createTokenAccount(
      provider,
      baseMint,
      provider.wallet.publicKey,
    );
    await mintToAccount(
      provider,
      baseMint,
      baseAccount,
      new BN(numTokens),
      provider.wallet.publicKey,
    );
    if (!quoteMint) {
      quoteMint = await createMint(provider, undefined);
      quoteAccount = await createTokenAccount(
        provider,
        quoteMint,
        provider.wallet.publicKey,
      );
    }

    state = await so.state(SO_NAME, baseMint);
    baseVault = await so.baseVault(SO_NAME, baseMint);

    const instr = program.instruction.configV2(
      new BN(optionExpiration),
      new BN(subscriptionPeriodEnd),
      new BN(numTokens),
      new BN(LOT_SIZE),
      SO_NAME,
      {
        accounts: {
          authority: provider.wallet.publicKey,
          soAuthority: provider.wallet.publicKey,
          issueAuthority: provider.wallet.publicKey,
          state,
          baseVault,
          baseAccount,
          quoteAccount,
          baseMint,
          quoteMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: web3.SystemProgram.programId,
          rent: web3.SYSVAR_RENT_PUBKEY,
        },
      },
    );

    const tx = new Transaction();
    tx.add(instr);
    await provider.sendAndConfirm(tx);
  }

  async function initStrike(strike: number) {
    console.log('Init Strike');

    optionMint = await so.soMint(strike, SO_NAME, baseMint);

    const instr = await so.createInitStrikeInstruction(
      new BN(strike),
      SO_NAME,
      provider.wallet.publicKey,
      baseMint,
    );

    const tx = new Transaction();
    tx.add(instr);
    await provider.sendAndConfirm(tx);
  }

  async function issue(amount: number, strike: number) {
    console.log('Issuing');

    userSoAccount = await createTokenAccount(
      provider,
      optionMint,
      provider.wallet.publicKey,
    );

    const instr = await so.createIssueInstruction(
      new BN(amount),
      new BN(strike),
      SO_NAME,
      provider.wallet.publicKey,
      baseMint,
      userSoAccount,
    );
    const tx = new Transaction();
    tx.add(instr);
    await provider.sendAndConfirm(tx);
  }

  async function addTokens() {
    console.log('Adding tokens');
    await mintToAccount(
      provider,
      baseMint,
      baseAccount,
      new BN(numTokens),
      provider.wallet.publicKey,
    );

    const instr = await so.createAddTokensInstruction(
      new BN(OPTIONS_AMOUNT),
      SO_NAME,
      provider.wallet.publicKey,
      baseAccount,
    );
    const tx = new Transaction();
    tx.add(instr);
    await provider.sendAndConfirm(tx);
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
      new BN(OPTIONS_AMOUNT * STRIKE * DEFAULT_MINT_DECIMALS),
      provider.wallet.publicKey,
    );

    userBaseAccount = await createTokenAccount(
      provider,
      baseMint,
      provider.wallet.publicKey,
    );

    const feeAccount = await SO.getFeeAccount(quoteMint);

    try {
      console.log('Creating ATA', feeAccount.toBase58());
      const ataTx = new Transaction();
      ataTx.add(
        await createAssociatedTokenAccount(
          provider.wallet.publicKey,
          new PublicKey('7Z36Efbt7a4nLiV7s5bY7J2e4TJ6V9JEKGccsy2od2bE'),
          quoteMint,
        ),
      );
      await provider.sendAndConfirm(ataTx);
    } catch (err) {
      console.log(err);
      console.log('Fee account already exists');
    }

    console.log('Creating exercise instruction');
    const instr = await so.createExerciseInstruction(
      new BN(amount),
      new BN(STRIKE),
      SO_NAME,
      provider.wallet.publicKey,
      userSoAccount,
      userQuoteAccount,
      userBaseAccount,
    );
    const tx = new Transaction();
    tx.add(instr);
    await provider.sendAndConfirm(tx);
  }

  async function withdraw(sleep = OPTION_EXPIRATION_DELAY_SEC) {
    console.log('Withdrawing');
    console.log(`Sleeping til options expire: ${Date.now() / 1_000}`);
    await new Promise((r) => setTimeout(r, sleep * 1_000));
    console.log(`Done sleeping: ${Date.now() / 1_000}`);

    const instr = await so.createWithdrawInstruction(
      SO_NAME,
      provider.wallet.publicKey,
      baseAccount,
    );
    const tx = new Transaction();
    tx.add(instr);
    await provider.sendAndConfirm(tx);
  }

  async function nameToken() {
    console.log('Naming token');

    const instr = await so.createNameTokenInstruction(
      new BN(STRIKE),
      SO_NAME,
      provider.wallet.publicKey,
      baseMint,
    );
    const tx = new Transaction();
    tx.add(instr);
    await provider.sendAndConfirm(tx);
  }

  it('Config v2 Success', async () => {
    await configureV2();
    console.log('Done configuring, doing verification');

    // Verify the State.
    const stateObj = await program.account.state.fetch(state);
    assert.equal(
      stateObj.authority.toBase58(),
      provider.wallet.publicKey.toBase58(),
    );
    assert.equal(
      stateObj.issueAuthority.toBase58(),
      provider.wallet.publicKey.toBase58(),
    );
    assert.equal(stateObj.optionsAvailable.toNumber(), numTokens);
    assert.equal(stateObj.optionExpiration.toNumber(), optionExpiration);
    assert.equal(
      stateObj.subscriptionPeriodEnd.toNumber(),
      subscriptionPeriodEnd,
    );
    assert.equal(stateObj.baseDecimals, DEFAULT_MINT_DECIMALS);
    // assert.equal(stateObj.quoteDecimals, DEFAULT_MINT_DECIMALS);
    assert.equal(stateObj.baseMint.toBase58(), baseMint.toBase58());
    assert.equal(stateObj.quoteMint.toBase58(), quoteMint.toBase58());
    assert.equal(stateObj.quoteAccount.toBase58(), quoteAccount.toBase58());
    assert.equal(stateObj.strikes.length, 0);
    assert.equal(stateObj.lotSize, LOT_SIZE);
    assert.equal(stateObj.soName, SO_NAME);

    // Verify the tokens are stored.
    const baseVaultAccount = await getAccount(provider.connection, baseVault);
    assert.equal(Number(baseVaultAccount.amount), numTokens);
  });

  it('Config Success', async () => {
    await configureSO();

    // Verify the State.
    const stateObj = await program.account.state.fetch(state);
    assert.equal(
      stateObj.authority.toBase58(),
      provider.wallet.publicKey.toBase58(),
    );
    assert.equal(stateObj.optionsAvailable.toNumber(), numTokens);
    assert.equal(stateObj.optionExpiration.toNumber(), optionExpiration);
    assert.equal(
      stateObj.subscriptionPeriodEnd.toNumber(),
      subscriptionPeriodEnd,
    );
    assert.equal(stateObj.baseDecimals, DEFAULT_MINT_DECIMALS);
    // assert.equal(stateObj.quoteDecimals, DEFAULT_MINT_DECIMALS);
    assert.equal(stateObj.baseMint.toBase58(), baseMint.toBase58());
    assert.equal(stateObj.quoteMint.toBase58(), quoteMint.toBase58());
    assert.equal(stateObj.quoteAccount.toBase58(), quoteAccount.toBase58());
    assert.equal(stateObj.strikes.length, 0);
    assert.equal(stateObj.lotSize, LOT_SIZE);
    assert.equal(stateObj.soName, SO_NAME);

    // Verify the tokens are stored.
    const baseVaultAccount = await getAccount(provider.connection, baseVault);
    assert.equal(Number(baseVaultAccount.amount), numTokens);
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

    const userSoAccountAccount = await getAccount(provider.connection, userSoAccount);
    assert.equal(Number(userSoAccountAccount.amount), OPTIONS_AMOUNT / LOT_SIZE);
  });

  it('AddTokens Success', async () => {
    await configureSO();
    await initStrike(STRIKE);
    await addTokens();

    const baseVaultAccount = await getAccount(provider.connection, baseVault);
    assert.equal(
      Number(baseVaultAccount.amount),
      numTokens + OPTIONS_AMOUNT,
    );
  });

  it('Exercise Success', async () => {
    try {
      await configureSO();
      await initStrike(STRIKE);
      await issue(OPTIONS_AMOUNT, STRIKE);
      await exercise(OPTIONS_AMOUNT / LOT_SIZE);
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const userBaseAccountAccount = await getAccount(
      provider.connection,
      userBaseAccount,
    );
    assert.equal(
      Number(userBaseAccountAccount.amount),
      OPTIONS_AMOUNT,
    );
  });

  it('Withdraw Success', async () => {
    try {
      await configureSO();
      await initStrike(STRIKE);
      await issue(OPTIONS_AMOUNT, STRIKE);
      await exercise(OPTIONS_AMOUNT / LOT_SIZE);
      await withdraw();
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const userBaseAccountAccount = await getAccount(provider.connection, baseAccount);
    assert.equal(
      Number(userBaseAccountAccount.amount),
      numTokens - OPTIONS_AMOUNT,
    );
  });

  it('Name Token', async () => {
    try {
      await configureSO();
      await initStrike(STRIKE);
      await nameToken();

      const metaplex = new Metaplex(provider.connection);
      const nft = await metaplex.nfts().findByMint({ mintAddress: optionMint });

      // This verifies that the name gets truncated as well as scientific
      // notation for strike in terms of tokens.
      assert.equal(nft.name, 'DUAL-SO_staking_options-1.00e-3');

      assert.equal(nft.symbol, 'DUAL-SO');
      assert.equal(nft.uri, 'https://www.dual.finance/images/token-logos/staking-options.json');
    } catch (err) {
      console.log(err);
      assert(false);
    }
  });

  it('Config Fail Name Too Long', async () => {
    SO_NAME = '123456789012345678901234567890123';
    try {
      await configureSO();
      assert(false);
    } catch (err) {
      assert(true);
    }

    // Reset SO_NAME.
    SO_NAME = 'SO_staking_options_SO';
  });

  it('Partial Withdraw Success', async () => {
    try {
      await configureSO();
      await initStrike(STRIKE);
      await issue(OPTIONS_AMOUNT, STRIKE);

      console.log('Attempting partial withdraw');
      await withdraw(OPTION_EXPIRATION_DELAY_SEC / 2);

      assert.equal(
        Number((await getAccount(provider.connection, baseAccount)).amount),
        numTokens - OPTIONS_AMOUNT,
      );

      console.log('Attempting partial withdraw again');
      await withdraw(0);

      // No change in the number of tokens
      assert.equal(
        Number((await getAccount(provider.connection, baseAccount)).amount),
        numTokens - OPTIONS_AMOUNT,
      );

      console.log('Final withdraw');
      await withdraw(OPTION_EXPIRATION_DELAY_SEC / 2);

      assert.equal(
        Number((await getAccount(provider.connection, baseAccount)).amount),
        numTokens,
      );
    } catch (err) {
      console.log(err);
      assert(false);
    }

    console.log('Verifying state removed');
    assert(await provider.connection.getAccountInfo(state) === null);
  });

  it('E2E Reversible', async () => {
    try {
      subscriptionPeriodEnd = Math.floor(Date.now() / 1_000 + OPTION_EXPIRATION_DELAY_SEC / 2);
      optionExpiration = Math.floor(Date.now() / 1_000 + OPTION_EXPIRATION_DELAY_SEC);
      console.log(`subscriptionPeriodEnd: ${subscriptionPeriodEnd}, optionExpiration: ${optionExpiration}`);

      baseMint = await createMint(provider, undefined);
      baseAccount = await createTokenAccount(
        provider,
        baseMint,
        provider.wallet.publicKey,
      );
      await mintToAccount(
        provider,
        baseMint,
        baseAccount,
        new BN(numTokens),
        provider.wallet.publicKey,
      );
      quoteMint = await createMint(provider, undefined);
      quoteAccount = await createTokenAccount(
        provider,
        quoteMint,
        provider.wallet.publicKey,
      );

      state = await so.state(SO_NAME, baseMint);
      baseVault = await so.baseVault(SO_NAME, baseMint);
      const [quoteVault, _quoteVaultBump] = web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from(utils.bytes.utf8.encode('so-reverse-vault')),
          Buffer.from(utils.bytes.utf8.encode(SO_NAME)),
          baseMint.toBuffer(),
        ],
        program.programId,
      );

      const authority = provider.wallet.publicKey;
      const configReversibleInstr = program.instruction.configV3(
        new BN(optionExpiration),
        new BN(subscriptionPeriodEnd),
        new BN(numTokens),
        new BN(LOT_SIZE),
        SO_NAME,
        {
          accounts: {
            authority,
            soAuthority: authority,
            issueAuthority: authority,
            state,
            baseVault,
            quoteVault,
            baseAccount,
            quoteAccount,
            baseMint,
            quoteMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: web3.SystemProgram.programId,
            rent: web3.SYSVAR_RENT_PUBKEY,
          },
        },
      );

      const configTx = new Transaction();
      configTx.add(configReversibleInstr);
      await provider.sendAndConfirm(configTx);

      optionMint = await so.soMint(STRIKE, SO_NAME, baseMint);
      const [reverseOptionMint, _reverseOptionMintBump] = await web3.PublicKey.findProgramAddress(
        [
          Buffer.from(utils.bytes.utf8.encode('so-reverse-mint')),
          state.toBuffer(),
          toBeBytes(STRIKE),
        ],
        program.programId,
      );

      const initStrikeReversibleInstr = program.instruction.initStrikeReversible(new BN(STRIKE), {
        accounts: {
          authority,
          payer: authority,
          state,
          optionMint,
          reverseOptionMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: web3.SystemProgram.programId,
          rent: web3.SYSVAR_RENT_PUBKEY,
        },
      });

      const initStrikeTx = new Transaction();
      initStrikeTx.add(initStrikeReversibleInstr);
      await provider.sendAndConfirm(initStrikeTx);

      await issue(OPTIONS_AMOUNT, STRIKE);

      userQuoteAccount = await createTokenAccount(
        provider,
        quoteMint,
        provider.wallet.publicKey,
      );
      await mintToAccount(
        provider,
        quoteMint,
        userQuoteAccount,
        new BN(OPTIONS_AMOUNT * STRIKE * DEFAULT_MINT_DECIMALS),
        provider.wallet.publicKey,
      );

      userBaseAccount = await createTokenAccount(
        provider,
        baseMint,
        provider.wallet.publicKey,
      );

      try {
        const ataTx = new Transaction();
        ataTx.add(
          await createAssociatedTokenAccount(
            provider.wallet.publicKey,
            new PublicKey('7Z36Efbt7a4nLiV7s5bY7J2e4TJ6V9JEKGccsy2od2bE'),
            quoteMint,
          ),
        );
        await provider.sendAndConfirm(ataTx);
      } catch (err) {
        console.log(err);
        console.log('Fee account already exists');
      }

      baseVault = await so.baseVault(SO_NAME, baseMint);
      userReverseSoAccount = await createTokenAccount(
        provider,
        reverseOptionMint,
        provider.wallet.publicKey,
      );

      const reversibleExerciseInstr = program.instruction.exerciseReversible(
        new BN(OPTIONS_AMOUNT / LOT_SIZE),
        new BN(STRIKE),
        {
          accounts: {
            authority,
            state,
            userSoAccount,
            optionMint,
            userReverseSoAccount,
            reverseOptionMint,
            userQuoteAccount,
            quoteVault,
            baseVault,
            userBaseAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
        },
      );

      const exerciseTx = new Transaction();
      exerciseTx.add(reversibleExerciseInstr);
      await provider.sendAndConfirm(exerciseTx);

      const reverseExerciseInstr = program.instruction.reverseExercise(
        new BN(OPTIONS_AMOUNT / LOT_SIZE / 2),
        new BN(STRIKE),
        {
          accounts: {
            authority,
            state,
            userSoAccount,
            optionMint,
            userReverseSoAccount,
            reverseOptionMint,
            userQuoteAccount,
            quoteVault,
            baseVault,
            userBaseAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
        },
      );

      const reverseTx = new Transaction();
      reverseTx.add(reverseExerciseInstr);
      await provider.sendAndConfirm(reverseTx);

      console.log(`Sleeping til options expire: ${Date.now() / 1_000}`);
      await new Promise((r) => setTimeout(r, OPTION_EXPIRATION_DELAY_SEC * 1_000));
      console.log(`Done sleeping: ${Date.now() / 1_000}`);

      const feeAccount = await SO.getFeeAccount(quoteMint);

      const withdrawAllInstr = program.instruction.withdrawAll({
        accounts: {
          authority,
          state,
          baseVault,
          baseAccount,
          quoteVault,
          quoteAccount,
          feeQuoteAccount: feeAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: web3.SystemProgram.programId,
        },
      });
      const withdrawTx = new Transaction();
      withdrawTx.add(withdrawAllInstr);
      await provider.sendAndConfirm(withdrawTx);
    } catch (err) {
      console.log(err);
      assert(false);
    }
  });
});
