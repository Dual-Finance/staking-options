import assert from 'assert';
import { PublicKey, Transaction } from '@solana/web3.js';
import { Provider, Program } from '@project-serum/anchor';
import { StakingOptions as SO } from '@dual-finance/staking-options';
import {
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
} from '@project-serum/associated-token';
const { getAccount } = require('@solana/spl-token');
import { StakingOptions } from '../target/types/staking_options';
import {
  DEFAULT_MINT_DECIMALS,
  createMint,
  createTokenAccount,
  mintToAccount,
} from './utils/utils';

const anchor = require('@project-serum/anchor');

describe('staking-options', () => {
  anchor.setProvider(anchor.Provider.env());
  const provider: Provider = anchor.Provider.env();
  const program = anchor.workspace.StakingOptions as Program<StakingOptions>;

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
  let userBaseAccount: PublicKey;

  let optionExpiration: number;
  let subscriptionPeriodEnd: number;
  const numTokens: number = 1_000_000_000;
  const STRIKE: number = 1_000;
  const OPTIONS_AMOUNT: number = 10_000_000;
  const LOT_SIZE: number = 1_000_000;
  const SO_NAME: string = 'SO';

  async function configureSO() {
    console.log('Configuring SO');

    optionExpiration = Math.floor(Date.now() / 1000 + 100);
    subscriptionPeriodEnd = optionExpiration;

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
      numTokens,
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
      numTokens,
      LOT_SIZE,
      SO_NAME,
      provider.wallet.publicKey,
      baseMint,
      baseAccount,
      quoteMint,
      quoteAccount,
    );

    const tx = new Transaction();
    tx.add(instr);
    await provider.send(tx);
  }

  async function initStrike(strike: number) {
    console.log('Init Strike');

    optionMint = await so.soMint(strike, SO_NAME, baseMint);

    const instr = await so.createInitStrikeInstruction(
      strike,
      SO_NAME,
      provider.wallet.publicKey,
      baseMint,
    );

    const tx = new Transaction();
    tx.add(instr);
    await provider.send(tx);
  }

  async function issue(amount: number, strike: number) {
    console.log('Issuing');

    userSoAccount = await createTokenAccount(
      provider,
      optionMint,
      provider.wallet.publicKey,
    );

    const instr = await so.createIssueInstruction(
      amount,
      strike,
      SO_NAME,
      provider.wallet.publicKey,
      baseMint,
      userSoAccount,
    );
    const tx = new Transaction();
    tx.add(instr);
    await provider.send(tx);
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

    const instr = await so.createAddTokensInstruction(
      OPTIONS_AMOUNT,
      SO_NAME,
      provider.wallet.publicKey,
      baseAccount,
    );
    const tx = new Transaction();
    tx.add(instr);
    await provider.send(tx);
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

    userBaseAccount = await createTokenAccount(
      provider,
      baseMint,
      provider.wallet.publicKey,
    );

    // Sleep so the account gets created
    await new Promise((r) => setTimeout(r, 1_000));
    const feeAccount = await getAssociatedTokenAddress(new PublicKey('7Z36Efbt7a4nLiV7s5bY7J2e4TJ6V9JEKGccsy2od2bE'), quoteMint);

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
      await provider.send(ataTx);
      await new Promise((r) => setTimeout(r, 1_000));
    } catch (err) {
      console.log('Fee account already exists');
    }

    const instr = await so.createExerciseInstruction(
      amount,
      STRIKE,
      SO_NAME,
      provider.wallet.publicKey,
      userSoAccount,
      userQuoteAccount,
      userBaseAccount,
    );
    const tx = new Transaction();
    tx.add(instr);
    await provider.send(tx);
  }

  async function withdraw() {
    console.log('Withdrawing');
    console.log('Sleeping til options expire');
    await new Promise((r) => setTimeout(r, 100_000));

    const instr = await so.createWithdrawInstruction(
      SO_NAME,
      provider.wallet.publicKey,
      baseAccount,
    );
    const tx = new Transaction();
    tx.add(instr);
    await provider.send(tx);
  }

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
    assert.equal(stateObj.quoteDecimals, DEFAULT_MINT_DECIMALS);
    assert.equal(stateObj.baseMint.toBase58(), baseMint.toBase58());
    assert.equal(stateObj.quoteAccount.toBase58(), quoteAccount.toBase58());
    assert.equal(stateObj.strikes.length, 0);

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
});
