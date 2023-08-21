import { AnchorProvider, BN, web3 } from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';
import { TokenInstructions } from '@project-serum/serum';

export const DEFAULT_MINT_DECIMALS = 6;

async function createMintInstructions(
  provider: AnchorProvider,
  authority: PublicKey,
  mint: PublicKey,
) {
  const instructions = [
    web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: mint,
      space: 82,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
      programId: TokenInstructions.TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeMint({
      mint,
      decimals: DEFAULT_MINT_DECIMALS,
      mintAuthority: authority,
    }),
  ];
  return instructions;
}

export async function createMint(
  provider: AnchorProvider,
  initialAuthority: PublicKey | undefined,
) {
  let authority = initialAuthority;
  if (authority === undefined) {
    authority = provider.wallet.publicKey;
  }
  const mint = web3.Keypair.generate();
  const instructions = await createMintInstructions(
    provider,
    authority,
    mint.publicKey,
  );

  const tx = new web3.Transaction();
  tx.add(...instructions);

  await provider.sendAndConfirm(tx, [mint]);

  return mint.publicKey;
}

async function createTokenAccountInstrs(
  provider: AnchorProvider,
  newAccountPubkey: PublicKey,
  mint: PublicKey,
  owner: PublicKey,
  lamportsRequested: number,
) {
  let lamports = lamportsRequested;
  if (lamports === undefined) {
    lamports = await provider.connection.getMinimumBalanceForRentExemption(165);
  }
  return [
    web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey,
      space: 165,
      lamports,
      programId: TokenInstructions.TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeAccount({
      account: newAccountPubkey,
      mint,
      owner,
    }),
  ];
}

export async function createTokenAccount(
  provider: AnchorProvider,
  mint: PublicKey,
  owner: PublicKey,
) {
  const vault = web3.Keypair.generate();
  const tx = new web3.Transaction();
  tx.add(
    ...(await createTokenAccountInstrs(provider, vault.publicKey, mint, owner, undefined)),
  );
  await provider.sendAndConfirm(tx, [vault]);
  return vault.publicKey;
}

async function createMintToAccountInstrs(
  mint: PublicKey,
  destination: PublicKey,
  amount: BN,
  mintAuthority: PublicKey,
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

export async function mintToAccount(
  provider: AnchorProvider,
  mint: PublicKey,
  destination: PublicKey,
  amount: BN,
  mintAuthority: PublicKey,
) {
  const tx = new web3.Transaction();
  tx.add(
    ...(await createMintToAccountInstrs(
      mint,
      destination,
      amount,
      mintAuthority,
    )),
  );
  await provider.sendAndConfirm(tx, []);
}

export function toBeBytes(x: number) {
  const y = Math.floor(x / 2 ** 32);
  return Uint8Array.from(
    [y, y << 8, y << 16, y << 24, x, x << 8, x << 16, x << 24].map(
      (z) => z >>> 24,
    ),
  );
}
