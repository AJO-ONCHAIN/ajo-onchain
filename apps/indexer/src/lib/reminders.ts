/**
 * Contribution-deadline reminders over WhatsApp and SMS.
 *
 * ## Why this lives in the indexer and not in the web app
 *
 * This is the part people get wrong, so it is worth being explicit.
 *
 * A reminder has to go out whether or not anybody has the web app open. It is
 * a scheduled job: wake up, find circles whose contribution deadline is
 * close, message the members who have not paid yet.
 *
 * Next.js route handlers cannot do that. They are request-scoped — they exist
 * because someone made an HTTP request, and they are killed when the response
 * is sent. On Vercel they are serverless functions, so there is no process to
 * keep a timer in and no guarantee the same instance is alive a minute later.
 * A `setInterval` in a route handler either never fires or fires on a random
 * subset of instances.
 *
 * The indexer is a long-lived process that is already running for the event
 * loop, already knows the deadlines, and already has the member list. This is
 * the right home.
 *
 * ## Why reminders matter more than they look
 *
 * A missed contribution is not a UX annoyance, it is a default: it is
 * recorded on-chain permanently and it leaves the rest of the circle short.
 * Most of them are forgetfulness, not inability to pay. For the members this
 * product exists for, a WhatsApp message the day before is the difference
 * between a circle that completes and one that collapses — and WhatsApp,
 * not email, is where that message has to arrive.
 */

import type { IndexerConfig } from "./config";

/** How often to scan for approaching deadlines. */
const SCAN_INTERVAL_MS = 60 * 60 * 1000;

/**
 * Start the reminder loop alongside the event poller.
 *
 * TODO(#22): implement the deadline scan.
 *
 * What it needs to do:
 *
 * 1. Query Supabase for circles in `Contributing` whose deadline falls inside
 *    the next 24 hours.
 * 2. For each, find members whose contributed bit is not set.
 * 3. Send each of them one message — see TODO(#23).
 * 4. Record that the reminder was sent, so the next scan an hour later does
 *    not send it again. Without this step the worker messages the same person
 *    twenty-four times, which is how a helpful reminder becomes a reason to
 *    block the number.
 */
export function startReminderWorker(config: IndexerConfig): void {
  void config;
  console.log(
    `[reminders] worker registered (scan every ${SCAN_INTERVAL_MS / 60000}m) — not yet implemented, see TODO(#22)`,
  );
}

/**
 * Send one reminder.
 *
 * TODO(#23): implement against a WhatsApp Business API provider.
 *
 * Notes for whoever picks this up:
 *
 * - WhatsApp requires pre-approved message templates for business-initiated
 *   conversations. The template text has to be submitted and approved before
 *   any of this works, so start that early — it is the long pole, not the
 *   code.
 * - Fall back to SMS when a member has no WhatsApp number on file.
 * - Phone numbers are personal data. They belong in Supabase behind row-level
 *   security, never in an event, never on-chain, and never in a log line.
 */
export async function sendReminder(
  recipientPhone: string,
  circleName: string,
  deadline: Date,
): Promise<void> {
  void recipientPhone;
  void circleName;
  void deadline;
}
