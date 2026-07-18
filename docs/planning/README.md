# Planning docs (product decisions)

**Status:** Active  
**Last updated:** 2026-07-15

Product/architecture decisions for TE Test Equipment Inventory. Origin: docs-only repo [Hassaan-ECE/TE_Lab_Equipment_Inventory](https://github.com/Hassaan-ECE/TE_Lab_Equipment_Inventory), remapped to **Test Equipment** identity.

## Authority order

1. [DECISIONS.md](DECISIONS.md) — **authoritative** policy (reconcile to code if they diverge)  
2. [../SESSION_HANDOFF.md](../SESSION_HANDOFF.md) / [../../README.md](../../README.md) — **current runtime** state (verify versions against package files)  
3. [PROJECT_DISCUSSION.md](PROJECT_DISCUSSION.md), [SECOND_OPINION_REVIEW.md](SECOND_OPINION_REVIEW.md), [ENGINEERING_SUGGESTIONS.md](ENGINEERING_SUGGESTIONS.md) — **historical / advisory** pre-implementation context  

If documents disagree on **policy**, update others to match **DECISIONS.md** (after checking code). If they disagree on **version/share/sync defaults**, code and live systems win.

## Related project entry points

| Doc | Role |
|-----|------|
| [../README.md](../README.md) | Docs index + trust rules |
| [../../README.md](../../README.md) | Project entry |
| [../SESSION_HANDOFF.md](../SESSION_HANDOFF.md) | Cross-session verified state |
| [../../AGENTS.md](../../AGENTS.md) | Short rules for coding agents |

## Identity (do not casually change)

| Item | Value |
|------|--------|
| Display name | TE Test Equipment Inventory |
| Workspace | `C:\Projects\Active\Inventory_Apps\TE\TE_Test_Equipment_Inventory` |
| Tauri identifier | `com.te.test.equipment.inventory` |
| Local DB (accepted) | `%LOCALAPPDATA%\com.te.test.equipment.inventory\inventory.feox` |
| Product share (D-027) | `S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE_Test_Equipment_Inventory` |

Earlier planning used the name “TE Lab Equipment Inventory” / `com.te.lab.equipment.inventory`. That is **historical**; see D-015 in DECISIONS.md.
