# Contributing to kohaku-rs

Thank you for helping build a **community-maintained** Rust layer on top of the [Ethereum Foundation Kohaku](https://github.com/ethereum/kohaku) SDK.

## Principles

1. **Track upstream, don’t fork blindly** — Prefer wrapping official `ethereum/kohaku` Rust crates over rewrites.
2. **Desktop-first** — APIs must work in Dioxus/Tauri without a browser extension host.
3. **Small, reviewable PRs** — One upstream module (or CLI command) per PR when possible.
4. **Document sync** — Update [KOHAKU_SYNC.md](./KOHAKU_SYNC.md) when you map or port a component.

## Development setup

```bash
rustup update stable
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all
```

## Pull request checklist

- [ ] `cargo test --workspace` passes  
- [ ] `KOHAKU_SYNC.md` updated (status + last reviewed commit)  
- [ ] Public API changes noted in crate `CHANGELOG.md` (when crate exists)  
- [ ] No secrets, RPC keys, or mainnet mnemonics in tests  
- [ ] Link to upstream file you ported (GitHub permalink with commit SHA)

## Keeping in sync with upstream

### Watch these repositories

On GitHub, watch **Releases** and **Discussions** for:

- https://github.com/ethereum/kohaku  
- https://github.com/kassandraoftroy/kohaku-cli  
- https://github.com/ethereum/kohaku-extension (UI-only changes; skim for SDK usage)  
- https://github.com/a16z/helios (light client)

### Weekly maintainer ritual (~30 minutes)

1. Open [KOHAKU_SYNC.md](./KOHAKU_SYNC.md) and note today’s date.  
2. Run the sync script (add to repo root):

   ```bash
   ./scripts/sync-upstream.sh
   ```

3. Triage the generated `sync-reports/YYYY-MM-DD.md`:  
   - New commits in `ethereum/kohaku` since last pin  
   - npm version bumps for `@kohaku-eth/*`  
   - New issues labeled `breaking` or `security`  
4. File GitHub issues for any upstream change that affects our trait boundaries.  
5. Update **Last upstream pin** in `KOHAKU_SYNC.md`.

### Upstream pin policy

Record in `KOHAKU_SYNC.md`:

```text
Last upstream pin: ethereum/kohaku @ <full-sha>
Last kohaku-cli pin: kassandraoftroy/kohaku-cli @ <full-sha>
```

For `kohaku-railgun`, bump the git dependency only after running integration tests.

## Code ownership areas

| Area | Upstream source | Crate |
|------|-----------------|-------|
| Plugin traits | `packages/plugins` | `kohaku-core` |
| Railgun | `crates/railgun`, `crates/railgun-ts` | `kohaku-railgun` |
| Stealth | ERC-5564 + roadmap | `kohaku-stealth` |
| Helios provider | `packages/provider/src/helios` | (future `kohaku-provider`) |
| P2P broadcast | `packages/plugins/src/broadcaster` | `kohaku-p2p` |
| CLI | `kassandraoftroy/kohaku-cli/src/commands/*` | `kohaku-cli` |

## Reporting security issues

Do **not** open public issues for key-handling or cryptography bugs. Email the maintainers listed in the repo profile and coordinate with upstream Kohaku if the bug is in `ethereum/kohaku`.

## Communication

- GitHub Discussions for design questions  
- Tag PRs with `good-first-issue`, `upstream-sync`, `phase-1`  
- Mention kohaku-rs in Ethereum Rust / Kohaku community threads when releasing crates (see README launch plan)
