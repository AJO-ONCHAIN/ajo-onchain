---
name: Good first issue
about: A scoped, self-contained task suitable for someone new to the repository.
title: "[good first issue] "
labels: ["good first issue", "help wanted"]
assignees: ""
---

<!--
Maintainers: fill every section. The point of this template is that someone
who has never seen this repository can pick the issue up without asking a
follow-up question first. If you cannot fill in the file path and the
definition of done, the issue is not ready to be opened.
-->

## What needs doing

<!--
Written for someone who has never seen the repository. Say what the thing is
and why it matters, not just what to type. Two or three sentences.

Good:  "The vault must reject a second contribution from the same member in
        one round. The check exists in the contract; there is no test proving
        it works, so a refactor could remove it silently."
Bad:   "Add test for double contribution."
-->

## Where

**File:** `path/to/file.rs`
**Marker:** `TODO(#N)` — find it with `grep -rn "TODO(#N)" .`

<!--
Point at the exact place. If the change spans two files, list both and say
which one to start with.
-->

## Definition of done

- [ ] The change is implemented at the marker above.
- [ ] A test covers it, and that test fails if the change is reverted.
- [ ] `task check` passes locally.
- [ ] The `TODO(#N)` marker is removed, and its entry in `docs/ISSUES.md` is
      deleted in the same pull request.

<!-- Add anything specific to this issue. -->

## Skills

<!-- e.g. "Rust basics. No Soroban or blockchain experience needed — the
test harness in contracts/escrow-vault/src/test.rs sets everything up." -->

## Effort

<!-- One of: 1–2 hours / half a day / 1–2 days. Be honest; an underestimate
that leaves someone stuck at midnight costs a contributor. -->

## Before you start

Comment on the issue to claim it — this avoids two people doing the same
work. If you get stuck, ask in the issue. A question is not a failure; a
silently abandoned branch is worse for everyone.

If `task contracts:test` does not pass on a clean clone, that is our bug, not
yours. Open an issue.
