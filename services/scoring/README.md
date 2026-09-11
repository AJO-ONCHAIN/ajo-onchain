# `services/scoring` — credit scoring and default early warning

**Status: Tranche 3. Deliberately empty.** There are no Python files here yet
and there should not be until Tranches 1 and 2 are complete.

A scoring model trained on no data is a decoration. This service becomes
useful once real circles have produced real contribution histories, and not
before.

## What it will do

A FastAPI service that reads contribution history — from the Supabase cache
and from `ReputationRegistry` — and produces two advisory outputs:

1. **A credit score** for a member, usable by lenders and by circle founders
   deciding whom to admit.
2. **A default early warning**: circles whose contribution pattern suggests
   someone is about to miss a payment, so a human can reach out before it
   becomes a default on a permanent record.

The second is the more valuable one. A default is recorded on-chain forever
and leaves the rest of the circle short, and most defaults are avoidable —
someone forgot, or is a few days from payday.

## What it will never do

**This service has no write authority over `EscrowVault` or the round state
machine.** It cannot move funds. It cannot flag a default. It cannot change a
phase, admit a member, or influence who receives a payout. There is no
mechanism by which its output can alter contract state.

That is a structural property, not a policy. The service holds no key that
any contract accepts, and the vault has no entry point that would take one.

**A pull request giving this service a path to move funds should be rejected
on sight** — not reviewed and negotiated, rejected. The whole pitch of AJO
Onchain is that no off-chain component has to be honest for members' money to
be safe. A scoring service that can touch funds is a component that has to be
honest, and it is a component running a statistical model whose failure modes
nobody fully understands.

## Its one write path

The service may publish a **signed attestation** to `ReputationRegistry`.

That attestation:

- is stored under a **separate storage key** from contract-enforced history,
  so a consumer can tell them apart without reading our documentation;
- is **labelled as off-chain model output**, not as contract-enforced fact;
- carries the **signing key and model version** that produced it, so a
  consumer can decide whether to trust that model and can stop trusting it
  later.

A score is an opinion. A contribution count is a fact. Storing them together
would let the opinion borrow the authority of the fact, which is precisely
the confusion that makes automated credit scoring harmful when it is wrong.

## Fairness is a requirement, not a later concern

This service will produce numbers that affect whether real people get access
to money. Before it ships:

- Its inputs must be documented, and anything that is a proxy for something
  we would not score on directly must be justified in writing or removed.
- A member must be able to see what their score is and what drove it.
- There must be a route to contest a score that is wrong.

Write these down before the first model, not after the first complaint.
