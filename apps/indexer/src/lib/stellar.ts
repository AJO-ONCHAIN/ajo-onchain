/**
 * Soroban RPC access: event polling and cursor persistence.
 *
 * Everything here is a read. The `@stellar/stellar-sdk` import is the RPC
 * client only — this module never constructs, signs or submits a transaction.
 */

import type { IndexerConfig } from "./config";

/**
 * One contract event, normalised into the shape the handlers want.
 *
 * `topic` is the first topic of the event, which is the name declared in
 * `contracts/shared/src/events.rs`. `data` is the event body: the
 * `#[contractevent]` macro publishes non-topic fields as a map keyed by field
 * name, so these are the struct's field names.
 */
export interface ContractEvent {
  /** Address of the vault (or factory) that emitted this event. */
  contractId: string;
  /** First topic — the event name. See `events.rs`. */
  topic: string;
  /** Decoded event body, keyed by the Rust struct's field names. */
  data: Record<string, unknown>;
  /** Ledger sequence the event was emitted in. */
  ledger: number;
  /** RPC paging cursor for this event. */
  cursor: string;
}

export interface PollResult {
  events: ContractEvent[];
  /** Cursor to resume from, or `null` if nothing new arrived. */
  nextCursor: string | null;
}

/**
 * Fetch contract events since `cursor`.
 *
 * TODO(#11): implement this against `rpc.Server.getEvents`.
 *
 * What it needs to do:
 *
 * 1. Watch two sets of contracts: the factory at `config.factoryContractId`,
 *    and every vault address the factory has deployed. The vault list comes
 *    from `deployed` events, which means the factory must be polled first on
 *    a cold start or the first few vault events are missed.
 * 2. Page through results. `getEvents` returns a bounded page; keep
 *    requesting until the returned cursor stops advancing.
 * 3. Convert each raw event's XDR topics and value into the `ContractEvent`
 *    shape above using `scValToNative`.
 * 4. Handle the RPC's retention window. A Soroban RPC node keeps only a
 *    limited history of events. If the stored cursor is older than the node's
 *    oldest ledger the call fails, and the indexer must fall back to
 *    reconstructing state by reading contract storage directly rather than
 *    silently skipping the gap.
 *
 * Returning an empty result for now means `bun run indexer:dev` starts, logs,
 * and idles cleanly instead of crashing — which is what you want while you
 * are building the rest.
 */
export async function pollEvents(
  config: IndexerConfig,
  cursor: string | null,
): Promise<PollResult> {
  void config;
  void cursor;
  return { events: [], nextCursor: null };
}

/**
 * Read the saved cursor.
 *
 * TODO(#13): persist the cursor in Supabase rather than in memory.
 *
 * It must survive a restart. An indexer that resumes from nothing will
 * either replay the entire chain history on every deploy or, worse, start at
 * "now" and leave a permanent hole in the cache that nothing ever fills.
 * A single-row `indexer_state` table keyed by network is enough.
 */
export async function loadCursor(): Promise<string | null> {
  return null;
}

/**
 * Persist the cursor. See TODO(#13).
 */
export async function saveCursor(cursor: string): Promise<void> {
  void cursor;
}
