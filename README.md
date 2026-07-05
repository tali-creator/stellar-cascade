# Cascade

**Automated, streaming funding for the open-source software this ecosystem depends on.**

## The problem

Every protocol, wallet, and app here is built on a stack of open-source libraries, SDKs, and tools — most of it maintained by a small number of unpaid or underpaid contributors. Funding those maintainers today is manual and occasional: a grant application here, a one-off donation there, decided by a small group of reviewers on their own timeline. There is no default, automated way for a projects success to flow back down to the dependencies that made it possible. When a maintainer burns out or moves on, everything built on top of their work inherits that risk silently.

## What Cascade does

Cascade is a funding-splitting protocol built natively on Soroban. A project registers its dependency tree on-chain — declaring what it is built on and how funding should be split across that tree. From then on, whenever money flows in (protocol revenue, a grant, a donation) it cascades automatically down the tree in real time, denominated in USDC or EURC.

Core features:
- **On-chain dependency graphs** — projects declare dependencies and split percentages transparently, and can update them as their stack evolves
- **Continuous micro-streaming** — funds move in small, constant streams rather than lump-sum payouts, giving maintainers a predictable ongoing income instead of a single check
- **Cascading splits** — if Project A depends on Project B, which depends on Project C, funding cascades through the whole tree automatically, not just one layer deep
- **Any funding source** — works whether the incoming money is protocol trading fees, a foundation grant, or a one-time community donation
- **Public, verifiable flows** — anyone can see exactly how funds are moving through the dependency graph, so there is no ambiguity about who is getting paid for what

## Why this model works

Automated, streaming dependency funding has already proven itself as a durable approach: platforms built on this idea have moved tens of millions of dollars to open-source maintainers over the years, and newer epoch-based streaming variants have distributed thousands of units of value to 80+ projects on a predictable recurring cycle, without depleting the underlying treasury, because funding comes from continuous flow rather than one-time drawdown.

Low, predictable transaction costs are a genuine structural advantage for this model — funding can stream in amounts and frequencies that would be economically pointless on a higher-fee network. That makes Cascade a real differentiator, not just a port of an existing idea: a funding mechanism that is only viable because of what this network is good at.

## What this unlocks

- A sustainable, automated income stream for the maintainers every other project here quietly depends on
- Reduced maintainer burnout and turnover, meaning fewer silent points of failure across shared tooling
- A transparent, on-chain alternative to manual grant allocation that scales without needing a review committee to grow in proportion
- A flywheel: as more projects launch and route funding through Cascade, the dependencies they share get stronger, making the next project easier to build

## Status

🚧 Early development. See open issues for current priorities across `/frontend`, `/backend`, and `/contracts`.

## Repo structure

```
frontend/    Next.js/TypeScript app — dependency graph explorer, funding dashboard
backend/     Rust service — indexes on-chain events, serves API for the frontend
contracts/   Soroban smart contracts — dependency registry, streaming/splitting logic
```

## Contributing

Issues are open and welcome contributions across all three layers. See individual issue labels for scope and difficulty.
