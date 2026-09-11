# Architecture diagrams

## Round lifecycle

One circle, one round. Phases advance exactly one step at a time, and only
once the deadline has passed. Rotational circles skip the two bidding phases
through an explicit arm in the transition table, not by omission — see
`next_phase` in `contracts/escrow-vault/src/lib.rs`.

```mermaid
stateDiagram-v2
    [*] --> Contributing: initialize()

    Contributing --> Settling: advance_phase()<br/>Rotational
    Contributing --> Bidding: advance_phase()<br/>BidBased
    Bidding --> Revealing: advance_phase()
    Revealing --> Settling: advance_phase()
    Settling --> Payout: advance_phase()
    Payout --> Contributing: settle_and_pay()<br/>round += 1

    note right of Contributing
        contribute() accepted here only.
        Leaving this phase flags every
        member who did not pay.
    end note

    note right of Payout
        advance_phase() is REJECTED here.
        Only settle_and_pay() leaves Payout.
        Otherwise the pot would roll into
        the next round unpaid.
    end note

    note left of Settling
        Rotational: recipient is recomputed
        as the lowest-indexed member who has
        not yet won. The caller's argument is
        checked against it, never trusted.
    end note
```

Two properties of this diagram are load-bearing:

- **Every transition is `advance_phase()` with no arguments.** There is no
  edge a caller can name. That is invariant 2.
- **`advance_phase` and `settle_and_pay` are permissionless.** Anyone may
  drive a circle whose deadline has passed, so no single member can stall the
  round and freeze everyone's funds. That is threat T3.

## Trust boundary

```mermaid
flowchart TB
    subgraph trustless["TRUSTLESS — enforced by contract"]
        direction TB
        factory["CircleFactory<br/><i>deploys one vault per circle</i>"]
        vault["EscrowVault<br/><b>holds all funds</b><br/><i>no admin withdrawal</i>"]
        token["SEP-41 token (USDC)<br/><i>the one trusted dependency</i>"]
        registry["ReputationRegistry<br/><i>Tranche 3 — public primitive</i>"]

        factory -->|deploy_v2| vault
        vault <-->|transfer| token
        vault -->|history| registry
    end

    subgraph advisory["ADVISORY — cannot move funds"]
        direction TB
        indexer["apps/indexer<br/><i>read cache + reminders</i><br/><b>holds no keys</b>"]
        supabase[("Supabase<br/><i>cache</i>")]
        scoring["services/scoring<br/><i>Tranche 3 — model output</i>"]
        web["apps/web<br/><i>thin BFF + UI</i>"]
    end

    member(["Circle member"])

    vault -.->|events| indexer
    registry -.->|read| scoring
    indexer --> supabase
    supabase -.->|read| web
    scoring -.->|signed attestation<br/>separate key, labelled| registry

    member -->|signs transactions<br/>with own wallet| vault
    member -->|reads| web

    classDef trust fill:#0b3d2c,stroke:#1f7a5c,color:#e8f5ef
    classDef adv fill:#3d2c0b,stroke:#7a5c1f,color:#f5efe8
    class factory,vault,token,registry trust
    class indexer,supabase,scoring,web adv
```

**Every arrow into the advisory box is dotted, because every one of them is a
read.** There is no solid arrow from advisory back into trustless that moves
value. The single arrow pointing back — the scoring service's attestation —
writes model output under a separate storage key, clearly labelled, and
cannot change a contract-enforced fact.

That is the property the whole pitch rests on — protect it in review.

Concretely, in review: if a pull request adds an arrow from the advisory box
into the trustless box, or gives anything in the advisory box a signing key,
that is not a feature with a security consideration. It is a change to what
the product is, and it needs a discussion before it needs a code review.

## Why the indexer is a separate process

```mermaid
flowchart LR
    subgraph vercel["Next.js on Vercel — request-scoped"]
        route["route handler<br/><i>born on request,<br/>dies on response</i>"]
    end

    subgraph worker["apps/indexer — long-lived process"]
        poll["event poll loop"]
        remind["reminder scheduler"]
    end

    rpc["Soroban RPC"]

    rpc --> poll
    route -.->|"cannot hold<br/>a connection"| rpc

    classDef bad fill:#4a1c1c,stroke:#a33,color:#fbeaea
    classDef good fill:#1c3d4a,stroke:#3a8,color:#eaf5fb
    class route bad
    class poll,remind good
```

A Next.js route handler exists because someone made an HTTP request and is
killed when the response is sent. On Vercel it is a serverless function, so
there is no process to keep a timer in and no guarantee the same instance
exists a minute later.

Event subscription needs a connection that outlives a request. Reminders need
to fire whether or not anyone has the app open. Neither belongs in a route
handler, and putting them there produces a system that appears to work in
development and silently stops in production.
