const anchor = require('@project-serum/anchor');
const serumCmn = require('@project-serum/common');
const { TokenInstructions } = require('@project-serum/serum');
const { PublicKey } = require('@solana/web3.js');

const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
  TokenInstructions.TOKEN_PROGRAM_ID.toString(),
);
const DEFAULT_MINT_DECIMALS = 6;

async function getTokenAccount(provider, addr) {
  return serumCmn.getTokenAccount(provider, addr);
}

async function createMintInstructions(provider, authority, mint) {
  const instructions = [
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: mint,
      space: 82,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
      programId: TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeMint({
      mint,
      decimals: DEFAULT_MINT_DECIMALS,
      mintAuthority: authority,
    }),
  ];
  return instructions;
}

async function createMint(provider, initialAuthority) {
  let authority = initialAuthority;
  if (authority === undefined) {
    authority = provider.wallet.publicKey;
  }
  const mint = anchor.web3.Keypair.generate();
  const instructions = await createMintInstructions(
    provider,
    authority,
    mint.publicKey,
  );

  const tx = new anchor.web3.Transaction();
  tx.add(...instructions);

  await provider.send(tx, [mint]);

  return mint.publicKey;
}

async function createTokenAccountInstrs(
  provider,
  newAccountPubkey,
  mint,
  owner,
  lamportsRequested,
) {
  let lamports = lamportsRequested;
  if (lamports === undefined) {
    lamports = await provider.connection.getMinimumBalanceForRentExemption(165);
  }
  return [
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey,
      space: 165,
      lamports,
      programId: TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeAccount({
      account: newAccountPubkey,
      mint,
      owner,
    }),
  ];
}

async function createTokenAccount(provider, mint, owner) {
  const vault = anchor.web3.Keypair.generate();
  const tx = new anchor.web3.Transaction();
  tx.add(
    ...(await createTokenAccountInstrs(provider, vault.publicKey, mint, owner)),
  );
  await provider.send(tx, [vault]);
  return vault.publicKey;
}

async function createMintToAccountInstrs(
  mint,
  destination,
  amount,
  mintAuthority,
) {
  return [
    TokenInstructions.mintTo({
      mint,
      destination,
      amount,
      mintAuthority,
    }),
  ];
}

async function mintToAccount(
  provider,
  mint,
  destination,
  amount,
  mintAuthority,
) {
  // mint authority is the provider
  const tx = new anchor.web3.Transaction();
  tx.add(
    ...(await createMintToAccountInstrs(
      mint,
      destination,
      amount,
      mintAuthority,
    )),
  );
  await provider.send(tx, []);
}

// https://github.com/nodejs/node/blob/v14.17.0/lib/internal/buffer.js#L129-L145
function readBigInt64LE(buffer, initialOffset = 0) {
  let offset = initialOffset;
  const first = buffer[offset];
  const last = buffer[offset + 7];
  if (first === undefined || last === undefined) {
    throw new RangeError();
  }
  // tslint:disable-next-line:no-bitwise
  const val = buffer[offset + 4]
    + buffer[offset + 5] * 2 ** 8
    + buffer[offset + 6] * 2 ** 16
    + (last << 24); // Overflow
  return (
    (BigInt(val) << BigInt(32)) // tslint:disable-line:no-bitwise
    + BigInt(
      first
        + buffer[++offset] * 2 ** 8
        + buffer[++offset] * 2 ** 16
        + buffer[++offset] * 2 ** 24,
    )
  );
}

// https://github.com/nodejs/node/blob/v14.17.0/lib/internal/buffer.js#L89-L107
function readBigUInt64LE(buffer, initialOffset = 0) {
  let offset = initialOffset;
  const first = buffer[offset];
  const last = buffer[offset + 7];
  if (first === undefined || last === undefined) {
    throw new RangeError();
  }

  const lo = first
    + buffer[++offset] * 2 ** 8
    + buffer[++offset] * 2 ** 16
    + buffer[++offset] * 2 ** 24;

  const hi = buffer[++offset]
    + buffer[++offset] * 2 ** 8
    + buffer[++offset] * 2 ** 16
    + last * 2 ** 24;

  return BigInt(lo) + (BigInt(hi) << BigInt(32)); // tslint:disable-line:no-bitwise
}

function empty32Buffer() {
  return Buffer.alloc(32);
}

function PKorNull(data) {
  return data.equals(empty32Buffer()) ? null : new PublicKey(data);
}

function parsePriceInfo(data, exponent) {
  // aggregate price
  const priceComponent = readBigInt64LE(data, 0);
  const price = Number(priceComponent) * 10 ** exponent;
  // aggregate confidence
  const confidenceComponent = readBigUInt64LE(data, 8);
  const confidence = Number(confidenceComponent) * 10 ** exponent;
  // aggregate status
  const status = data.readUInt32LE(16);
  // aggregate corporate action
  const corporateAction = data.readUInt32LE(20);
  // aggregate publish slot. It is converted to number to be consistent with
  // Solana's library interface (Slot there is number)
  const publishSlot = Number(readBigUInt64LE(data, 24));
  return {
    priceComponent,
    price,
    confidenceComponent,
    confidence,
    status,
    corporateAction,
    publishSlot,
  };
}

function parseEma(data, exponent) {
  // current value of ema
  const valueComponent = readBigInt64LE(data, 0);
  const value = Number(valueComponent) * 10 ** exponent;
  // numerator state for next update
  const numerator = readBigInt64LE(data, 8);
  // denominator state for next update
  const denominator = readBigInt64LE(data, 16);
  return {
    valueComponent, value, numerator, denominator,
  };
}

async function parsePriceData(data, currentSlot) {
  // pyth magic number
  const magic = data.readUInt32LE(0);
  // program version
  const version = data.readUInt32LE(4);
  // account type
  const type = data.readUInt32LE(8);
  // price account size
  const size = data.readUInt32LE(12);
  // price or calculation type
  const priceType = data.readUInt32LE(16);
  // price exponent
  const exponent = data.readInt32LE(20);
  // number of component prices
  const numComponentPrices = data.readUInt32LE(24);
  // number of quoters that make up aggregate
  const numQuoters = data.readUInt32LE(28);
  // slot of last valid (not unknown) aggregate price
  const lastSlot = readBigUInt64LE(data, 32);
  // valid on-chain slot of aggregate price
  const validSlot = readBigUInt64LE(data, 40);
  // exponential moving average price
  const emaPrice = parseEma(data.slice(48, 72), exponent);
  // exponential moving average confidence interval
  const emaConfidence = parseEma(data.slice(72, 96), exponent);
  // space for future derived values
  const drv1Component = readBigInt64LE(data, 96);
  const drv1 = Number(drv1Component) * 10 ** exponent;
  // minimum number of publishers for status to be TRADING
  const minPublishers = data.readUInt8(104);
  // space for future derived values
  const drv2 = data.readInt8(105);
  // space for future derived values
  const drv3 = data.readInt16LE(106);
  // space for future derived values
  const drv4 = data.readInt32LE(108);
  // product id / reference account
  const productAccountKey = new PublicKey(data.slice(112, 144));
  // next price account in list
  const nextPriceAccountKey = PKorNull(data.slice(144, 176));
  // valid slot of previous update
  const previousSlot = readBigUInt64LE(data, 176);
  // aggregate price of previous update
  const previousPriceComponent = readBigInt64LE(data, 184);
  const previousPrice = Number(previousPriceComponent) * 10 ** exponent;
  // confidence interval of previous update
  const previousConfidenceComponent = readBigUInt64LE(data, 192);
  const previousConfidence = Number(previousConfidenceComponent) * 10 ** exponent;
  // space for future derived values
  const drv5Component = readBigInt64LE(data, 200);
  const drv5 = Number(drv5Component) * 10 ** exponent;
  const aggregate = parsePriceInfo(data.slice(208, 240), exponent);

  const { status, price, confidence } = aggregate;

  // price components - up to 32
  const priceComponents = [];
  let offset = 240;
  let shouldContinue = true;
  while (offset < data.length && shouldContinue) {
    const publisher = PKorNull(data.slice(offset, offset + 32));
    offset += 32;
    if (publisher) {
      const componentAggregate = parsePriceInfo(
        data.slice(offset, offset + 32),
        exponent,
      );
      offset += 32;
      const latest = parsePriceInfo(data.slice(offset, offset + 32), exponent);
      offset += 32;
      priceComponents.push({
        publisher,
        aggregate: componentAggregate,
        latest,
      });
    } else {
      shouldContinue = false;
    }
  }

  return {
    magic,
    version,
    type,
    size,
    priceType,
    exponent,
    numComponentPrices,
    numQuoters,
    lastSlot,
    validSlot,
    emaPrice,
    emaConfidence,
    drv1Component,
    drv1,
    minPublishers,
    drv2,
    drv3,
    drv4,
    productAccountKey,
    nextPriceAccountKey,
    previousSlot,
    previousPriceComponent,
    previousPrice,
    previousConfidenceComponent,
    previousConfidence,
    drv5Component,
    drv5,
    aggregate,
    priceComponents,
    price,
    confidence,
    status,
  };
}

function parsePriceAndExpiration(buf) {
  // const overhead = readBigUInt64LE(buf, 0);
  const strikePrice = Number(readBigUInt64LE(buf, 8));
  const expiration = Number(readBigUInt64LE(buf, 16));
  const mintPk = new PublicKey(buf.slice(24, 56));
  return {
    strikePrice,
    expiration,
    spl_mint: mintPk,
  };
}

function toBeBytes(x) {
  const y = Math.floor(x / 2 ** 32);
  return Uint8Array.from(
    [y, y << 8, y << 16, y << 24, x, x << 8, x << 16, x << 24].map(
      (z) => z >>> 24,
    ),
  );
}

async function findAssociatedTokenAddress(walletAddress, tokenMintAddress) {
  const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
    'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
  );
  return (
    await PublicKey.findProgramAddress(
      [
        walletAddress.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        tokenMintAddress.toBuffer(),
      ],
      SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    )
  )[0];
}

module.exports = {
  TOKEN_PROGRAM_ID,
  getTokenAccount,
  createMint,
  createTokenAccount,
  createTokenAccountInstrs,
  mintToAccount,
  parsePriceData,
  parsePriceAndExpiration,
  toBeBytes,
  findAssociatedTokenAddress,
  DEFAULT_MINT_DECIMALS,
};
