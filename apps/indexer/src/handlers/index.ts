/**
 * Event handlers — one branch per contract event topic.
 *
 * **`contracts/shared/src/events.rs` is the source of truth for these
 * topic names and payload fields.** Not this file, and not the database
 * schema. If a topic is renamed there and not here, the `default` branch
 * below starts logging unknown events and the dashboard goes stale without
 * anything failing loudly. That is why renaming a topic is a breaking change
 * that must move both sides in one pull request.
 *
 * ## Handlers must be idempotent
 *
 * The cursor advances only after a whole batch is handled, so a crash
 * mid-batch replays the batch. Every write here must therefore be an upsert
 * keyed by something stable — the contract address plus round, or the event's
 * ledger and index — never a blind `insert` or a `count = count + 1`.
 */

import type { ContractEvent } from "../lib/stellar";

/**
 * Dispatch one event to its handler.
 *
 * The exhaustive switch is deliberate: an unrecognised topic is logged rather
 * than ignored, so a contract/indexer version mismatch shows up in the logs
 * on the first event instead of being discovered when someone notices the
 * numbers are wrong.
 */
export async function handleEvent(event: ContractEvent): Promise<void> {
  switch (event.topic) {
    case "created":
      return handleCreated(event);
    case "joined":
      return handleJoined(event);
    case "contrib":
      return handleContrib(event);
    case "phase":
      return handlePhase(event);
    case "payout":
      return handlePayout(event);
    case "default":
      return handleDefault(event);
    case "cycdone":
      return handleCycleDone(event);
    case "deployed":
      return handleDeployed(event);
    default:
      console.warn(
        `[handlers] unknown topic "${event.topic}" from ${event.contractId} — ` +
          "is the indexer older than the deployed contracts?",
      );
  }
}

/**
 * TODO(#14): upsert a `circles` row.
 *
 * Payload: `founder`, `token`, `contribution`. The vault address is
 * `event.contractId`, which is the natural primary key.
 */
async function handleCreated(event: ContractEvent): Promise<void> {
  void event;
}

/**
 * TODO(#15): upsert a `circle_members` row.
 *
 * Payload: `member`, `index`. Key on `(circle, index)` rather than
 * `(circle, member)` — the index is what every contract bitmap refers to, so
 * it is the value the rest of the cache has to agree with.
 */
async function handleJoined(event: ContractEvent): Promise<void> {
  void event;
}

/**
 * TODO(#16): record the contribution and update the circle's pot.
 *
 * Payload: `member`, `amount`, `round`. Key on `(circle, member, round)`,
 * which makes a replayed event a no-op.
 */
async function handleContrib(event: ContractEvent): Promise<void> {
  void event;
}

/**
 * TODO(#17): update the circle's phase and deadline.
 *
 * Payload: `round`, `phase_index`. The index maps to
 * `Contributing | Bidding | Revealing | Settling | Payout` in that order —
 * see `phase_index` in `contracts/escrow-vault/src/lib.rs`. It is a number
 * rather than a name precisely so that reordering the Rust enum cannot
 * change the meaning of events already on the ledger; do not "improve" this
 * by switching on a string.
 */
async function handlePhase(event: ContractEvent): Promise<void> {
  void event;
}

/**
 * TODO(#18): record the payout and mark the recipient as having won.
 *
 * Payload: `recipient`, `net`, `fee`, `round`. This is the row the
 * reputation work in Tranche 3 will eventually read, so store the round and
 * the amounts, not just the fact that someone was paid.
 */
async function handlePayout(event: ContractEvent): Promise<void> {
  void event;
}

/**
 * TODO(#19): flag the member as in default for this round.
 *
 * Payload: `member`, `round`. Defaults are permanent on-chain — the contract
 * never clears the bitmap — so the cache must not clear them either, however
 * tempting it is to "fix" a member who pays late.
 */
async function handleDefault(event: ContractEvent): Promise<void> {
  void event;
}

/**
 * TODO(#20): close out the cycle and reset per-cycle aggregates.
 *
 * Payload: `round`. Every member has now been paid once and the contract has
 * cleared its winners bitmap.
 */
async function handleCycleDone(event: ContractEvent): Promise<void> {
  void event;
}

/**
 * TODO(#21): add the new vault to the watch list.
 *
 * Payload: `vault`, `founder`. Emitted by the factory, not a vault. Until
 * this is stored, a newly deployed circle is invisible to the next poll —
 * see the cold-start note in `lib/stellar.ts`.
 */
async function handleDeployed(event: ContractEvent): Promise<void> {
  void event;
}
