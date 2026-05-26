# Kohaku upstream sync map

Living document: **official Kohaku module → kohaku-rs crate**. Update on every upstream sync.

## Pin record

| Source | Branch | Last reviewed commit | Date |
|--------|--------|----------------------|------|
| [ethereum/kohaku](https://github.com/ethereum/kohaku) | `master` | _set on first weekly sync_ | — |
| [kassandraoftroy/kohaku-cli](https://github.com/kassandraoftroy/kohaku-cli) | `main` | _set on first weekly sync_ | — |
| [ethereum/kohaku-extension](https://github.com/ethereum/kohaku-extension) | `main` | _optional_ | — |

npm versions (from kohaku-cli lockfile baseline):

| Package | Version |
|---------|---------|
| `@kohaku-eth/railgun` | 0.0.1-alpha.21 |
| `@kohaku-eth/plugins` | 0.0.1-alpha.8 |
| `@kohaku-eth/privacy-pools` | 0.0.2-alpha.9 |

---

## Repository index

| URL | Purpose |
|-----|---------|
| https://github.com/ethereum/kohaku | SDK monorepo (TypeScript + Rust) |
| https://github.com/kassandraoftroy/kohaku-cli | **CLI wallet blueprint** |
| https://github.com/ethereum/kohaku-extension | Reference browser wallet |
| https://github.com/ethereum/kohaku-commons | Shared extension / wallet logic |
| https://github.com/AmbireTech/extension | Ambire upstream (pre-privacy fork) |
| https://github.com/a16z/helios | Helios light client (Rust) |
| https://notes.ethereum.org/@niard/KohakuRoadmap | Roadmap (stealth, P2P, Helios, PQ) |
| https://ethereum.github.io/kohaku/ | Docs site |

> **Note:** `github.com/kohaku-eth/railgun` is **not** a separate repository; Railgun ships inside `ethereum/kohaku` as `crates/railgun-ts` (npm `@kohaku-eth/railgun`).

---

## TypeScript / npm packages → Rust crates

| Upstream path | npm name | kohaku-rs crate | Status | Notes |
|---------------|----------|-----------------|--------|-------|
| `packages/plugins/` | `@kohaku-eth/plugins` | **kohaku-core** | 🟡 | `Host`, `PrivacyPlugin`, errors — see `src/plugin.rs` |
| `packages/plugins/src/host/` | (part of plugins) | **kohaku-core** | 🟡 | `Network`, `Storage`, `Keystore` |
| `packages/plugins/src/broadcaster/` | (part of plugins) | **kohaku-p2p** | 🔴 | Roadmap Phase 2; stub only |
| `crates/railgun-ts/` | `@kohaku-eth/railgun` | **kohaku-railgun** | 🔴 | Wrap `crates/railgun` (native), not TS |
| `crates/railgun/` | — | **kohaku-railgun** | 🔴 | EF native ZK / indexer / transact |
| `crates/userop-kit/` | — | **kohaku-railgun** | 🔴 | ERC-4337 userops for private broadcast |
| `crates/userop-kit-ts/` | — | **kohaku-railgun** | 🔴 | TS bindings; ignore for desktop |
| `packages/privacy-pools/` | `@kohaku-eth/privacy-pools` | _future_ `kohaku-privacy-pools` | 🔴 | CLI already uses pools |
| `packages/provider/` | `@kohaku-eth/provider` | _future_ `kohaku-provider` | 🔴 | Phase 1: Helios |
| `packages/provider/src/helios/` | — | **kohaku-provider** | 🔴 | Port to `a16z/helios` library |
| `packages/provider/src/ethers/` | — | **kohaku-provider** | 🔴 | Fallback RPC |
| `packages/provider/src/viem/` | — | — | — | Use `alloy` in Rust |
| `packages/provider/src/colibri/` | — | **kohaku-provider** | 🔴 | Private state reads |
| `packages/pq-account/` | `@kohaku-eth/pq-account` | _future_ `kohaku-pq` | 🔴 | Phase 3 |
| _(roadmap only)_ | — | **kohaku-stealth** | 🟢 | ERC-5564; not in SDK monorepo yet |
| `crates/common/` | — | **kohaku-core** | 🟡 | Share types with upstream where possible |
| `crates/crypto/` | — | **kohaku-railgun** | 🔴 | Crypto primitives for Railgun |
| `crates/poseidon-rust/` | — | **kohaku-railgun** | 🔴 | Hash function |
| `crates/eip-1193-provider/` | — | **kohaku-provider** | 🔴 | Provider abstraction |

---

## Official Rust crates (reuse, don’t rewrite)

| Upstream crate | Modules (indicative) | kohaku-rs action |
|----------------|----------------------|------------------|
| `crates/railgun` | `account`, `circuit`, `transact`, `indexer`, `merkle_tree`, `poi` | Git dependency + `PrivacyPlugin` adapter |
| `crates/common` | shared types | Re-export or align `kohaku-core::types` |
| `crates/crypto` | symmetric / wallet crypto | Depend from `kohaku-railgun` |
| `crates/userop-kit` | 4337 bundling | Use in `broadcast_private_operation` |

---

## CLI command map (`kassandraoftroy/kohaku-cli` → `kohaku-cli`)

| Upstream file | Command | kohaku-rs status |
|---------------|---------|------------------|
| `src/commands/createWallet.ts` | `create-wallet` | 🔴 |
| `src/commands/listWallets.ts` | `list-wallets` | 🔴 |
| `src/commands/balances.ts` | `balances` | 🔴 |
| `src/commands/nextFreshAddress.ts` | `next-fresh-address` | 🔴 |
| `src/commands/shield.ts` | `shield` | 🔴 |
| `src/commands/unshield.ts` | `unshield` | 🔴 |
| `src/commands/seeDecryptedStorage.ts` | `see-decrypted-storage` | 🔴 |
| `src/host/makeHost.ts` | (host wiring) | 🔴 → `kohaku-core::Host` |
| `src/host/keystore.ts` | (encrypted seed) | 🔴 |
| `src/host/storage.ts` | (plugin state) | 🔴 |
| `src/index.ts` | CLI entry | 🟡 `stealth-meta`, `status` only |

Dependencies in upstream CLI `package.json`: `@kohaku-eth/plugins`, `@kohaku-eth/railgun`, `@kohaku-eth/privacy-pools`, `ethers`.

---

## Extension-only (low priority for kohaku-rs)

| Upstream | Reason to skip |
|----------|----------------|
| `ethereum/kohaku-extension` UI | Browser / React; Vaughan replaces with Dioxus |
| `ethereum/kohaku-commons` | Ambire-specific controllers |
| `AmbireTech/extension` | Upstream wallet chrome |

Still **watch** extension PRs for new SDK usage patterns.

---

## Phase alignment (Kohaku roadmap)

| Roadmap theme | Upstream today | kohaku-rs Phase |
|---------------|----------------|-----------------|
| Helios light client | `packages/provider/src/helios` | **Phase 1** → `kohaku-provider` |
| ERC-5564 stealth | Roadmap / ecosystem libs | **Phase 1** → `kohaku-stealth` ✅ scaffold |
| Railgun + 4337 | `crates/railgun`, `railgun-ts` | **Phase 2** → `kohaku-railgun` |
| P2P broadcast | `plugins/.../broadcaster` | **Phase 2** → `kohaku-p2p` |
| Privacy Pools | `packages/privacy-pools` | **Phase 2** |
| PQ accounts | `packages/pq-account` | **Phase 3** |

---

## Weekly diff workflow

1. `git fetch` pinned clones of `ethereum/kohaku` and `kohaku-cli`.  
2. `git log LAST_PIN..HEAD --oneline` for each.  
3. For each commit touching mapped paths above, add a row to `sync-reports/YYYY-MM-DD.md`.  
4. If `packages/plugins/src/base.ts` or `Host` types change, open a **breaking** issue for `kohaku-core`.  
5. If `@kohaku-eth/railgun` version bumps in kohaku-cli, schedule `kohaku-railgun` integration test.  
6. Update pin table at top of this file.

### Automation (recommended)

Add `.github/workflows/upstream-sync.yml` (weekly cron) that:

- Runs `scripts/sync-upstream.sh`  
- Opens a PR updating the pin table + report  
- Does **not** auto-merge

---

## crates.io naming

| Crate | crates.io |
|-------|-----------|
| `kohaku-core` | Publish first after traits stabilize |
| `kohaku-stealth` | **First public release** (Phase 1) |
| `kohaku-railgun` | After EF `railgun` crate is pinned + tested |
| `kohaku-p2p` | Phase 2 |
| `kohaku-cli` | Optional binary crate; can stay git-only |

Do **not** publish as `kohaku` (taken).
