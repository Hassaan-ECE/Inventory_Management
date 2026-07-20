# Docs index — Inventory Management

**Start here for agents/chats:** [SESSION_START_PROMPT.md](SESSION_START_PROMPT.md) · [SESSION_HANDOFF.md](SESSION_HANDOFF.md) · [CHAT_HANDOFF.md](CHAT_HANDOFF.md) · [planning/DECISIONS.md](planning/DECISIONS.md)

---

# Docs index — lineage note (TE Test Equipment)

## Trust rule

**Verify or do not trust.** Current version, sync defaults, share layout, and release files come from code + live systems (`package.json` / `Cargo.toml` / `tauri.conf.json`, `backend/src/sync`, `S:\...\TE_Test_Equipment_Inventory`, `gh release list`). Markdown is only as good as its last verification date.

## Live (use these)

| Doc | Role |
|-----|------|
| [planning/DECISIONS.md](planning/DECISIONS.md) | Product policy (D-001…); reconcile to code if they diverge |
| [SESSION_HANDOFF.md](SESSION_HANDOFF.md) | Current session state, team install, verified data/sync notes |
| [../README.md](../README.md) | Human project entry |
| [SESSION_START_PROMPT.md](SESSION_START_PROMPT.md) | Paste block for new chats |
| [../AGENTS.md](../AGENTS.md) | Agent workspace rules |
| [planning/IMPORT_PROFILE.md](planning/IMPORT_PROFILE.md) | Excel import profile (re-run dry-run before cutover claims) |

## Historical / advisory (not runtime authority)

Bannered or archive material. Do **not** use for current version, sync default, or updater behavior:

- `planning/PROJECT_DISCUSSION.md`, `SECOND_OPINION_REVIEW.md`, `ENGINEERING_SUGGESTIONS.md`
- `superpowers/specs/2026-07-13-te-test-equipment-v0.1-design.md` and other TE-scaffold lineage plans unless still cited by DECISIONS
- `engineering/archive/*`, ME-shaped notes under `engineering/` until rewritten for this product

## Implemented plans (behavior authority; not “open work”)

- [superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md](superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md) — **IM-011** adaptive TE sync (**implemented 2026-07-20**). Use for regressions and future multi-inventory extension; do not treat as unimplemented backlog.

## Active cutover design (not yet “done”)

- `superpowers/specs/2026-07-15-legacy-inventory-merge-design.md` — legacy supplement design; implementation status is in handoff/data, not assumed complete from the spec alone.
