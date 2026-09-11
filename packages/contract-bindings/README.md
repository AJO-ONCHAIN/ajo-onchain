# `packages/contract-bindings` — generated TypeScript clients

Typed clients for the contracts, generated from their WASM specs.

## Generated, not written

```bash
task bindings        # or: ./scripts/gen-bindings.sh
```

Output lands in `src/` and is **gitignored**.

Committing generated bindings guarantees that one day they will disagree with
the contracts they claim to describe, and the disagreement will be found by a
user rather than by CI. Regenerating takes seconds; debugging a stale binding
that silently encodes the wrong argument order does not.

CI regenerates them. If a contract's interface changes, nothing here needs
editing — rerun the task.

## Using them

```ts
import { Client as EscrowVault } from "@ajo/contract-bindings/escrow_vault";
```

The client is read-and-sign only: it builds transactions that a wallet signs.
It holds no key of its own, and it should not grow one.
