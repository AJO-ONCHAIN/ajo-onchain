/**
 * AJO Onchain indexer — read cache and reminder worker.
 *
 * ## Read-only by construction
 *
 * This process holds no keys and signs nothing. It has no Stellar account, no
 * secret in its environment that could authorise a transaction, and no code
 * path that builds one. Everything it does is: poll contract events, write
 * rows to Supabase, send reminder messages.
 *
 * That is a deliberate architectural property, not an accident of the current
 * feature set. **If this process dies, nobody loses money — the dashboard
 * goes stale.** Circles keep running, because `advance_phase` and
 * `settle_and_pay` are permissionless and anyone can poke a circle whose
 * deadline has passed.
 *
 * Please keep it that way. A pull request that gives the indexer a signing
 * key changes the trust model of the whole project: it would add a component
 * that must be online and honest for members' funds to be safe, and the
 * entire pitch is that no such component exists. If you think the indexer
 * needs to sign something, open a discussion first — the answer is almost
 * always that the contract should do it, or that the user's wallet should.
 *
 * ## Why TypeScript and not Rust
 *
 * The contributor pool this project is recruiting from — the Nigerian tech
 * community, Stellar Wave, OnlyDust — is overwhelmingly TypeScript. Matching
 * the contracts' toolchain would be tidier; being approachable to the people
 * we want contributing matters more, and this process is not consensus-
 * critical so the stakes of that choice are low.
 */

import { handleEvent } from "./handlers/index";
import { loadConfig } from "./lib/config";
import { startReminderWorker } from "./lib/reminders";
import { loadCursor, pollEvents, saveCursor } from "./lib/stellar";

const config = loadConfig();

/** Set false by SIGINT/SIGTERM so the loop finishes its pass and exits. */
let running = true;

/**
 * Poll, dispatch, persist the cursor, repeat.
 *
 * The cursor is saved only after every event in a batch has been handled, so
 * a crash mid-batch replays that batch rather than skipping it. Handlers must
 * therefore be idempotent — see the note in `handlers/index.ts`.
 */
async function main(): Promise<void> {
  console.log(`[indexer] starting against ${config.rpcUrl}`);
  console.log(`[indexer] factory ${config.factoryContractId}`);
  console.log(`[indexer] poll interval ${config.pollIntervalMs}ms`);

  startReminderWorker(config);

  let cursor = await loadCursor();
  console.log(`[indexer] resuming from cursor ${cursor ?? "(none)"}`);

  while (running) {
    try {
      const { events, nextCursor } = await pollEvents(config, cursor);

      for (const event of events) {
        await handleEvent(event);
      }

      if (nextCursor && nextCursor !== cursor) {
        cursor = nextCursor;
        await saveCursor(cursor);
      }

      if (events.length > 0) {
        console.log(`[indexer] handled ${events.length} event(s)`);
      }
    } catch (error) {
      // A failed poll is expected occasionally: RPC nodes restart, networks
      // blip. Log it and keep the loop alive rather than exiting, because a
      // dead indexer is a stale dashboard and nobody is watching this process
      // at 3am. The cursor is not advanced, so nothing is missed.
      console.error("[indexer] poll failed, retrying:", error);
    }

    await sleep(config.pollIntervalMs);
  }

  console.log("[indexer] stopped cleanly");
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function shutdown(signal: string): void {
  console.log(`[indexer] ${signal} received, finishing current pass`);
  running = false;
}

process.on("SIGINT", () => shutdown("SIGINT"));
process.on("SIGTERM", () => shutdown("SIGTERM"));

main().catch((error) => {
  console.error("[indexer] fatal:", error);
  process.exit(1);
});
