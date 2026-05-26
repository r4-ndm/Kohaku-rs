# kohaku-rs

Community-driven Rust SDK for [Kohaku](https://github.com/ethereum/kohaku)—the Ethereum Foundation’s modular privacy toolkit—aimed at **desktop wallets** (Dioxus, Tauri, native CLI) rather than browser extensions.

> **Unofficial.** This repository is not maintained by the Ethereum Foundation. It tracks upstream Kohaku releases and fills gaps for native Rust consumers such as [Vaughan](https://github.com/r4-ndm/vaughan).

## Why this exists

| Need | Official Kohaku today | kohaku-rs goal |
|------|----------------------|----------------|
| Desktop / Dioxus integration | Extension-first (`kohaku-extension`) | Pure Rust workspace, no Node at runtime |
| Published crates | npm `@kohaku-eth/*`; Rust lives in monorepo | `kohaku-*` on [crates.io](https://crates.io) |
| Stealth (ERC-5564) | Roadmap / not a standalone SDK package yet | `kohaku-stealth` (Phase 1) |
| Helios light client | `packages/provider/src/helios` (TS/WASM) | Native Helios via `a16z/helios` (Phase 1) |
| CLI reference | [`kassandraoftroy/kohaku-cli`](https://github.com/kassandraoftroy/kohaku-cli) | `kohaku-cli` binary |

**Important:** [`ethereum/kohaku`](https://github.com/ethereum/kohaku) already contains substantial **Rust** for Railgun (`crates/railgun`, `crates/crypto`, `userop-kit`, …). kohaku-rs does **not** reimplement that work—it wraps and exposes it behind stable, desktop-friendly APIs.

The crates.io name [`kohaku`](https://crates.io/crates/kohaku) is an unrelated tokenizer; use **`kohaku-core`**, **`kohaku-stealth`**, etc.

## Workspace crates

| Crate | Status | Purpose |
|-------|--------|---------|
| [`kohaku-core`](crates/kohaku-core) | 🟡 scaffolding | Host + `PrivacyPlugin` traits (mirrors `@kohaku-eth/plugins`) |
| [`kohaku-stealth`](crates/kohaku-stealth) | 🟢 Phase 1 | ERC-5564 meta-address generation |
| [`kohaku-railgun`](crates/kohaku-railgun) | 🔴 planned | Wrap `ethereum/kohaku/crates/railgun` |
| [`kohaku-p2p`](crates/kohaku-p2p) | 🔴 planned | Private broadcast (roadmap Phase 2) |
| [`kohaku-cli`](crates/kohaku-cli) | 🟡 scaffolding | Rust port of `kohaku-cli` |

Legend: 🟢 usable · 🟡 WIP · 🔴 not started

## Quick start

```bash
git clone https://github.com/r4-ndm/Kohaku-rs.git
cd kohaku-rs
cargo test
cargo run -p kohaku-cli -- stealth-meta
```

### Use in Vaughan (Dioxus)

```toml
[dependencies]
kohaku-core = "0"
kohaku-stealth = "0"
```

Run wallet logic on a background `tokio` task; call kohaku-rs from async commands. Avoid `wasm-bindgen` unless you deliberately target WASM—desktop builds should use native Helios + RPC.

## Roadmap

### Phase 1 (now) — stable primitives

1. **`kohaku-stealth`** — ERC-5564, publish `0.1.0` to crates.io  
2. **`kohaku-provider`** (future crate) — Helios from [`a16z/helios`](https://github.com/a16z/helios), port ideas from [`packages/provider/src/helios`](https://github.com/ethereum/kohaku/tree/master/packages/provider/src/helios)  
3. **`kohaku-core`** — finalize trait signatures against upstream `packages/plugins`

### Phase 2 — privacy protocols

4. **`kohaku-railgun`** — git-depend on `ethereum/kohaku` `crates/railgun`, implement `PrivacyPlugin`  
5. **Privacy Pools** — track `packages/privacy-pools`  
6. **`kohaku-p2p`** — port `packages/plugins/src/broadcaster`

### Phase 3 — wallet surface

7. **`kohaku-cli`** — parity with [`kohaku-cli` commands](https://github.com/kassandraoftroy/kohaku-cli#commands)  
8. Vaughan integration examples

See [KOHAKU_SYNC.md](./KOHAKU_SYNC.md) for the upstream ↔ Rust module map and [CONTRIBUTING.md](./CONTRIBUTING.md) for sync workflow.

## Upstream repositories

| Repository | Role |
|------------|------|
| https://github.com/ethereum/kohaku | **Primary SDK** (TS packages + Rust crates) |
| https://github.com/kassandraoftroy/kohaku-cli | **CLI blueprint** (shield / unshield / balances) |
| https://github.com/ethereum/kohaku-extension | Reference browser wallet (Ambire fork) |
| https://github.com/ethereum/kohaku-commons | Shared extension logic |
| https://github.com/AmbireTech/extension | Ambire upstream |
| https://github.com/a16z/helios | Light client (Rust) |
| https://notes.ethereum.org/@niard/KohakuRoadmap | Official roadmap |

npm packages (from monorepo): `@kohaku-eth/railgun`, `@kohaku-eth/plugins`, `@kohaku-eth/provider`, `@kohaku-eth/privacy-pools`, `@kohaku-eth/pq-account`.

## Contributing

We welcome contributors maintaining sync with upstream Kohaku. Read [CONTRIBUTING.md](./CONTRIBUTING.md) and update [KOHAKU_SYNC.md](./KOHAKU_SYNC.md) when you port a module.

## License

MIT — see [LICENSE](./LICENSE). Upstream Kohaku components may use other licenses; check each dependency before redistribution.
