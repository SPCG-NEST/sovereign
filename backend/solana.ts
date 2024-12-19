import {
  address,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
} from "@solana/web3.js";
import { AnchorProvider, Program } from "@coral-xyz/anchor";
import type { Idl } from "@coral-xyz/anchor";
import bs58 from "bs58";

import idl from "./idl/programs.json";

// Constants
const PROGRAM_ID = "4oVhv3o16X3XR99UgbFrWNKptoNBkg2hsbNY2nYPpv4a";
const HTTP_ENDPOINT = "http://localhost:8899";
const WS_ENDPOINT = "ws://localhost:8900";

// Setup
const rpc = createSolanaRpc(HTTP_ENDPOINT);
const rpcSubscriptions = createSolanaRpcSubscriptions(WS_ENDPOINT);
const abortController = new AbortController();

const provider = new AnchorProvider(
  { ...rpc, ...rpcSubscriptions } as any,
  {} as any,
  { commitment: "confirmed" },
);
const program = new Program(idl as Idl, provider);

// Monitor
async function setupProgramMonitor() {
  try {
    console.log("Program monitor starting...");

    // Subscribe to program account changes
    const accountTypes = Object.keys(program.account);
    console.log("Subscribing to account types:", accountTypes);

    const accountPromises = accountTypes.map(async (accountType) => {
      const discriminator = program.coder.accounts.memcmp(accountType);
      // Use programNotifications instead of accountNotifications so that we can
      // track newly created accounts that didn't exist prior to the subscription
      const accountsSubscription = await rpcSubscriptions
        .programNotifications(address(PROGRAM_ID), {
          filters: [
            {
              memcmp: {
                offset: BigInt(discriminator.offset),
                bytes: discriminator.bytes,
                encoding: "base58",
              },
            },
          ],
        })
        .subscribe({ abortSignal: abortController.signal });

      // Monitor this account type
      console.log(`Started monitoring ${accountType}`);
      for await (const notification of accountsSubscription) {
        const { account, pubkey } = notification.value;
        console.log(`\n=== Account Update (${accountType}) ===`);
        console.log("Account:", pubkey.toString());
        console.log("Slot:", notification.context.slot.toString());

        try {
          const decodedData = program.coder.accounts.decode(
            accountType,
            Buffer.from(bs58.decode(account.data)),
          );
          console.log("Account data:", decodedData);
        } catch (error) {
          console.error("Account decoding error:", error);
        }
      }
    });

    // Set up program logs monitoring
    const logsSubscription = await rpcSubscriptions
      .logsNotifications(
        { mentions: [address(PROGRAM_ID)] },
        { commitment: "confirmed" },
      )
      .subscribe({ abortSignal: abortController.signal });

    console.log("Started monitoring program logs");
    const logsPromise = (async () => {
      for await (const log of logsSubscription) {
        console.log("\n=== Program Log ===");
        console.log("Logs:", log.value.logs);
      }
    })();

    // Handle cleanup on interrupt
    process.on("SIGINT", () => {
      console.log("\nReceived interrupt signal. Cleaning up...");
      abortController.abort();
      process.exit(0);
    });
  } catch (error) {
    console.error("Program monitor error:", error);
    throw error;
  }
}

setupProgramMonitor()
  .then(() => console.log("Program monitor started"))
  .catch(console.error);
