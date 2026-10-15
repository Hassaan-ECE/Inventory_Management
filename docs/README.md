# Docs index — Inventory Management

**Start here for agents/chats:** [SESSION_START_PROMPT.md](SESSION_START_PROMPT.md) · [SESSION_HANDOFF.md](SESSION_HANDOFF.md) · [CHAT_HANDOFF.md](CHAT_HANDOFF.md) · [planning/DECISIONS.md](planning/DECISIONS.md)

---

# Docs index — lineage note (TE Test Equipment)

## Trust rule

**Verify or do not trust.** Current version, sync defaults, share layout, and release files come from code + live systems (`package.json` / `Cargo.toml` / `tauri.conf.json`, `backend/src/platform/shared_root.rs`, `backend/src/sync` and the per-module sync copies, `S:\...\Inventory_Management_App`, `gh release list`). Markdown is only as good as its last verification date.

## Live (use these)

| Doc | Role |
|-----|------|
| [planning/DECISIONS.md](planning/DECISIONS.md) | Product policy (D-001…); reconcile to code if they diverge |
| [SESSION_HANDOFF.md](SESSION_HANDOFF.md) | Current session state, team install, verified data/sync notes |
| [../README.md](../README.md) | Human project entry |
| [SESSION_START_PROMPT.md](SESSION_START_PROMPT.md) | Paste block for new chats |
| [../AGENTS.md](../AGENTS.md) | Agent workspace rules |
| [planning/IMPORT_PROFILE.md](planning/IMPORT_PROFILE.md) | Excel import profile (re-run dry-run before cutover claims) |
| [engineering/PROJECT_FOLDER_BREAKDOWN.md](engineering/PROJECT_FOLDER_BREAKDOWN.md) | File-by-file map of the code (regenerated 2026-10-15) |
| [engineering/FEOXDB_SYNC_MIGRATION_PLAN.md](engineering/FEOXDB_SYNC_MIGRATION_PLAN.md) | Sync design, shared layout per module, thresholds |
| [engineering/SYNC_RECOVERY_INVARIANTS.md](engineering/SYNC_RECOVERY_INVARIANTS.md) | Startup recovery rules |
| [engineering/UPDATER_AND_RELEASE.md](engineering/UPDATER_AND_RELEASE.md) | Signing keys and release checklist |
| [engineering/AGENT_RUNBOOK.md](engineering/AGENT_RUNBOOK.md) | Build/release traps and pivots |

## Historical / advisory (not runtime authority)

Bannered or archive material. Do **not** use for current version, sync default, or updater behavior:

- `planning/PROJECT_DISCUSSION.md`, `SECOND_OPINION_REVIEW.md`, `ENGINEERING_SUGGESTIONS.md`
- `superpowers/specs/2026-07-13-te-test-equipment-v0.1-design.md` and other TE-scaffold lineage plans unless still cited by DECISIONS
- `engineering/archive/*` (ME-lineage evidence; the active `engineering/` docs were reconciled to this product on 2026-10-15)

## Implemented plans (behavior authority; not “open work”)

- [superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md](superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md) — **IM-011** adaptive TE sync (**implemented 2026-07-20**). Use for regressions and future multi-inventory extension; do not treat as unimplemented backlog.

## Active roadmap (open work)

- [superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md](superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md) — what `bun run desktop` does today vs phases A–D (TE ops, architecture, module ports, ship).
- [superpowers/plans/2026-07-20-platform-module-architecture-extract.md](superpowers/plans/2026-07-20-platform-module-architecture-extract.md) — **IM-012** Phase B extract (implemented).
- [superpowers/plans/2026-07-20-te-lab-components-port.md](superpowers/plans/2026-07-20-te-lab-components-port.md) — **C1** TE Lab Components port (implemented).
- [superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md](superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md) — **IM-014** generalized Lab catalog + grid storage (implementation complete; copied-data/live cutover pending).
- [superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md](superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md) — **IM-015** same-record TE Calibration workspace + guarded roster tooling (implementation complete; live roster cutover pending).
- [superpowers/plans/2026-07-20-first-team-release.md](superpowers/plans/2026-07-20-first-team-release.md) — **Phase D** first team release (copy, product roots, NSIS, optional updater).

Migration operations: [runbooks/te-lab-components-catalog-v2-migration.md](runbooks/te-lab-components-catalog-v2-migration.md).

## Active cutover design (not yet “done”)

- `superpowers/specs/2026-07-15-legacy-inventory-merge-design.md` — legacy supplement design; implementation status is in handoff/data, not assumed complete from the spec alone.
