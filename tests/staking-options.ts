import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { StakingOptions } from "../target/types/staking_options";

describe("staking-options", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.StakingOptions as Program<StakingOptions>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
