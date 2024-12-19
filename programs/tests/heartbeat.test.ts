import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import type { Programs as Sovereign } from "../target/types/programs";
import SoverignIDL from "../target/idl/programs.json";
import { describe, it } from "bun:test";
const connection = new anchor.web3.Connection(
  "http://localhost:8899",
  "confirmed"
);
const keypair = anchor.web3.Keypair.fromSecretKey(
  Uint8Array.from(await Bun.file("./localnet-keys/admin.json").json())
);

describe("programs", () => {
  const wallet = new anchor.Wallet(keypair);
  const provider = new anchor.AnchorProvider(connection, wallet, {});
  const soverign = new Program<Sovereign>(SoverignIDL as Sovereign, provider);

  it("Is initialized!", async () => {
    const tx = await soverign.methods
      .initialize()
      .accounts({
        authority: wallet.publicKey,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });
});
