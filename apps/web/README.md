# `apps/web` — AJO Onchain web app

**Status: not scaffolded.** This directory is a README on purpose.

A half-built UI in the repository is worse than an honest placeholder. It
invites reviewers to judge the project on an unfinished screen instead of on
the contracts, which are the actual work. The UI arrives when Tranche 1's
contracts are done.

## Creating it

```bash
bun create next-app apps/web --typescript --tailwind --app --src-dir --import-alias "@/*"
```

Then add `@stellar/stellar-sdk`, `@supabase/supabase-js`, and a wallet
connector (Freighter, plus a mobile-friendly option — see below).

## Scope discipline — read this before writing a route handler

The contracts are the backend. The web app is a client with a thin
backend-for-frontend attached, and the BFF exists for exactly three things:

1. Session and auth cookies.
2. Proxying the SEP-24 anchor callback, which must not be exposed to the
   browser.
3. Calls that genuinely need a server-held secret.

Everything else is client-side or comes from the read cache.

**Not in route handlers:**

- **No business logic.** Whether a member may contribute, who is owed the
  pot, whether a phase can advance — all of that is decided by the contract
  and only by the contract. A rule implemented in a route handler is a rule a
  malicious client skips by calling the contract directly. Worse, if the two
  ever disagree, the contract wins and the UI has been lying.
- **No long-lived connections.** Route handlers are request-scoped. On Vercel
  they are serverless functions that are killed once the response is sent.
  Event subscription lives in `apps/indexer`.
- **No scheduled work.** Same reason. Contribution reminders live in
  `apps/indexer/src/lib/reminders.ts`, which explains this at length.

**Read circle state straight from Supabase.** It is the cache the indexer
maintains and it is fast. Read from the chain when correctness matters more
than latency — before signing a transaction, and on any screen showing money
about to move — because the cache can be stale or, if the indexer is down,
wrong.

## Mobile-first, and what that actually means here

The target user is on an Android phone on a metered connection in Lagos, not
on a laptop. That has consequences beyond responsive layout:

- Budget the JavaScript. Every unnecessary kilobyte is a real cost to someone.
- The app must be usable on a slow 3G connection.
- Amounts are shown in naira first, with the USDC amount secondary. Members
  save in naira and think in naira — see `audit/known-issues.md` #3, which is
  an open product question, not a solved one.
- Wallet connection has to work on a phone. Freighter is a desktop browser
  extension; a mobile path is required, not a follow-up.

## Verifying what the UI claims

Every screen that asserts something about custody should be checkable. Where
the UI says funds cannot be withdrawn by an admin, link to the contract
source on an explorer. The claim is worth little; the link is the product.
