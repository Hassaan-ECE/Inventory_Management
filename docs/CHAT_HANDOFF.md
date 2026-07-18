# Chat handoff — move conversation to Inventory Management

**Purpose:** Continue work in a new agent chat opened on the unified product workspace, not the TE standalone tree.

## Before the new chat

1. Open / focus folder: `C:\Projects\Active\Inventory_Management`
2. Create the new GitHub repository (empty or README-only is fine).
3. Optional: `git init`, first commit, `git remote add origin <url>`, push (or do that in the new chat).
4. Paste the prompt from `docs/SESSION_START_PROMPT.md` (section below the line) into the new chat.
5. Optionally attach or mention:
   - New repo URL
   - Whether adaptive sync (IM-011) or git setup should go first

## Canonical docs in this repo

| Doc | Role |
|-----|------|
| [SESSION_START_PROMPT.md](SESSION_START_PROMPT.md) | Paste into new chats |
| [SESSION_HANDOFF.md](SESSION_HANDOFF.md) | Current state / next slices |
| [planning/DECISIONS.md](planning/DECISIONS.md) | IM-* product decisions |
| [superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md](superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md) | Adaptive sync implementation authority |
| [../AGENTS.md](../AGENTS.md) | Short agent rules |
| [../README.md](../README.md) | Human product overview |

## Legacy (read-only reference)

| Path | Role |
|------|------|
| `C:\Projects\Active\Inventory_Apps\TE\TE_Test_Equipment_Inventory` | Standalone TE app + origin of D-029 plan |
| `C:\Projects\Active\Inventory_Apps\TE\TE_Parts_Inventory` | Standalone components |
| `C:\Projects\Active\Inventory_Apps\ME\ME_Inventory` | Standalone ME |
| `S:\...\InventoryApps\...` | Legacy shared data until cutover |

## Do not

- Continue large redesign work only in the TE standalone folder.
- Archive old GitHub repos before the unified app can replace them for the team.
- Point the new app’s updater at TE’s GitHub release endpoint.
