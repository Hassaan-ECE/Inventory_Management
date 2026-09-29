# TE Storage Room Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an empty, usable Storage Room inventory with independent saving and team sync.

**Architecture:** Add module-local domain/storage/sync code following the existing Lab inventory engine. Integrate it into the module registry, database handles and command routing. Build a focused React view with its own bridge and the existing adaptive sync controller.

**Tech Stack:** Existing React/TypeScript/Tauri/Rust/FeOxDB; no new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-28-te-storage-room-design.md`

## Global constraints

- Preserve existing product IDs, DB files and TE/Lab shared streams.
- Storage Room uses `te-storage`, `te-storage-room.feox`, `TE_Storage_Room`, and `INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT`.
- No live test records, installer staging, or version bump.

## Review focus

- Offline saves remain queued and converge after reconnect.
- Changes from an old editor preserve fields edited elsewhere.
- Failed saves retain form input; failed loads cannot show mock records.
- Sync finishing after switching modules cannot replace the active module's data.
- Hidden identifiers retain leading zeros and survive editing/restart.

## Tasks

- [x] Backend: adapt the inventory operation engine into `backend/src/modules/te_storage/`, with explicit PN/PR/PO domain fields; extend platform IDs/paths, stores, coordinator and commands. Test empty DB, reopen, two peers, stale edits, deletion, validation and root isolation with `cargo test --manifest-path backend/Cargo.toml te_storage`.
- [x] Frontend: implement `frontend/src/modules/te-storage/{types,storageBridge,useStorageInventory,StorageEntryDialog,TeStorageView}`; register host. Default to seven visible columns. Use search/sort and module-scoped visibility persistence. Run `bun run test -- frontend/tests/te-storage.test.tsx frontend/tests/platform-module-registry.test.ts`.
- [x] Verification: run frontend lint/tests/build and Rust tests/clippy; inspect the running frontend if browser tooling is available. Update session notes with delivery and remaining operational limits.

Execution is inline in the existing checkout, within the user's authorized scope.


## Completion record — 2026-09-28

- Frontend suite: 216 passed, 1 skipped. Final focused Storage Room/registry recheck: 9 passed.
- Rust full suite: 471 passed, 14 intentionally ignored. Final Storage Room recheck after monotonic timestamp refinement: 16 passed.
- `bun run lint`, `bun run build`, `cargo build --manifest-path backend/Cargo.toml`, and Clippy with `--all-targets --all-features -- -D warnings` passed. Vite retains its chunk-size advisory.
- Reviewed source once for isolation and concurrency. Added regression coverage and fixes for merged-field restart recovery, multiple offline edits, invalid stale patches, and post-outbox write failure rollback. Storage Room uses deterministic per-field version serialization and strictly increasing timestamps for successive local edits/deletion. Existing TE/Lab sync implementations were not refactored.
- Browser check against the existing Vite server: exact seven default headers, empty state, column menu defaults, showing PR # in its requested position, and contained horizontal overflow. Screenshot: `.tmp/te-storage-smoke/table.png`.
- Real Tauri desktop smoke used `.tmp/te-storage-smoke/data`, three redirected shared roots, sync disabled, and an isolated WebView profile. Added SMOKE-001 with PR 00045, PO 00087, quantity 3 and location A1. Stopped/reopened the process, verified all values, edited location to B4, then confirmed deletion and empty inventory. The three redirected shared directories remained absent. Smoke processes were stopped.
- Team synchronization was exercised against temporary shared directories with two separate FeOx stores. The live S: product share was unavailable to this terminal and was not used for verification.
- No version bump or installer release. The existing untracked July Lab plan was preserved.

Ruling: Implement in the existing checkout following the user's live-development working agreement and authorization for a working module. A focused fresh review was warranted for the new persistence/sync path; no worktree or incremental commits were introduced.
