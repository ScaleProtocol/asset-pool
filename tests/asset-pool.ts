import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ScaleAssetPool } from "../target/types/asset_pool";

describe("scale-asset-pool", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ScaleAssetPool as Program<ScaleAssetPool>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
