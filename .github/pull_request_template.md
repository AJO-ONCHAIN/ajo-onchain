## What this changes

<!-- One or two sentences. What is different after this is merged? -->

## Why

<!-- Link the issue: "Closes #12". If there is no issue, say what prompted
this. -->

## How it was tested

<!-- Say what you ran and what you saw. "task check passes" is a fine answer
for a docs change. For a contract change, name the test that would fail if
the change were reverted. -->

---

## Security-sensitive change

- [ ] **This PR touches fund movement, phase transitions, or the winners
      bitmap.**

<!--
If you ticked that box, you MUST explain below. Those three areas are the
audited invariants of this project, and they need to be visible at review
time rather than discovered in the diff.

Explain:
  - which of the three areas this touches,
  - what the change does to the invariant,
  - why it is still enforced afterwards.

An explanation like "small refactor, no behaviour change" is not enough for
these files — say what makes it behaviour-preserving.
-->

**Explanation (required if ticked):**

---

## The three invariants

A PR that weakens any of these will be rejected, however good the reason. If
you believe one must change, open a discussion and add the reasoning to
`audit/known-issues.md` first — the discussion comes before the code.

1. **No privileged withdrawal.** There is no admin withdrawal function
   anywhere in `EscrowVault`. Funds leave only through `settle_and_pay`, and
   only to a member of that circle.
2. **No phase skipping.** Phases advance exactly one step at a time via
   `advance_phase`. No entry point accepts a target phase from the caller.
3. **One win per cycle.** A member may receive at most one payout before
   every member has received one.

- [ ] I have read the three invariants above and this PR does not weaken any
      of them.

## Checklist

- [ ] `task check` passes locally.
- [ ] Commits follow [Conventional Commits](https://www.conventionalcommits.org/).
- [ ] If I resolved a `TODO(#n)`, I removed the marker and its entry in
      `docs/ISSUES.md`.
- [ ] If I changed an event topic or payload in
      `contracts/shared/src/events.rs`, this PR also updates
      `apps/indexer/src/handlers/index.ts` and `docs/contracts/events.md`.
      Those must move together or the indexer silently goes stale.
