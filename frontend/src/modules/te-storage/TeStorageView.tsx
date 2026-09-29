import { useEffect, useMemo, useState } from "react";
import { BoxesIcon, MoonIcon, PlusIcon, SearchIcon, SunIcon } from "lucide-react";
import type { DesktopModuleViewProps } from "@/platform/modules/types";
import { InventorySystemSwitcher } from "@/shell/InventorySystemSwitcher";
import { ShellStatusStrip } from "@/shell/ShellStatusStrip";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { getStorageBridge } from "./storageBridge";
import { StorageEntryDialog } from "./StorageEntryDialog";
import { StorageColumnMenu } from "./StorageColumnMenu";
import { STORAGE_COLUMNS, type StorageBridge, type StorageColumn, type StorageEntry } from "./types";
import { useStorageInventory } from "./useStorageInventory";

const VISIBILITY_KEY = "inventory-management:te-storage:columns:v1";
type Visibility = Record<StorageColumn, boolean>;
function defaultVisibility(): Visibility {
  return Object.fromEntries(STORAGE_COLUMNS.map(({ key }) => [key, key !== "pr" && key !== "description"])) as Visibility;
}
function readVisibility(): Visibility {
  const defaults = defaultVisibility();
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(VISIBILITY_KEY) ?? "null");
    if (saved && typeof saved === "object") {
      for (const { key } of STORAGE_COLUMNS) {
        const value = (saved as Record<string, unknown>)[key];
        if (typeof value === "boolean") defaults[key] = value;
      }
    }
    return Object.values(defaults).some(Boolean) ? defaults : defaultVisibility();
  } catch { return defaults; }
}

export function TeStorageView({ active, activeViewId, onViewChange, theme, onThemeToggle, bridge = getStorageBridge() }: DesktopModuleViewProps & { bridge?: StorageBridge | null }) {
  const inventory = useStorageInventory(active, bridge);
  const [visibility, setVisibility] = useState(readVisibility);
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<{ key: StorageColumn; direction: "asc" | "desc" } | null>(null);
  const [editor, setEditor] = useState<{ entry: StorageEntry | null } | null>(null);
  const [columnMenu, setColumnMenu] = useState<{ x: number; y: number } | null>(null);
  const [previousActive, setPreviousActive] = useState(active);
  if (previousActive !== active) {
    setPreviousActive(active);
    setColumnMenu(null);
  }
  useEffect(() => { try { localStorage.setItem(VISIBILITY_KEY, JSON.stringify(visibility)); } catch { /* Preferences are optional. */ } }, [visibility]);
  const columns = STORAGE_COLUMNS.filter(({ key }) => visibility[key]);
  const rows = useMemo(() => {
    const terms = query.toLocaleLowerCase().trim().split(/\s+/).filter(Boolean);
    const filtered = inventory.entries.filter(entry => !entry.archived && terms.every(term => STORAGE_COLUMNS.some(({ key }) => String(entry[key] ?? "").toLocaleLowerCase().includes(term))));
    if (!sort) return filtered;
    return [...filtered].sort((a, b) => {
      const left = a[sort.key]; const right = b[sort.key];
      if (left === null) return right === null ? 0 : 1;
      if (right === null) return -1;
      const order = typeof left === "number" && typeof right === "number" ? left - right : String(left).localeCompare(String(right), undefined, { numeric: true, sensitivity: "base" });
      return sort.direction === "asc" ? order : -order;
    });
  }, [inventory.entries, query, sort]);
  if (!active) return null;
  const mode = !bridge ? "Preview" : !inventory.loaded ? "Loading" : inventory.shared?.enabled && inventory.shared.hasLocalOnlyChanges ? "Pending sync" : inventory.shared?.available ? "Shared" : "Local";
  const message = inventory.error || (!bridge ? "Open the desktop app to add and save items." : inventory.shared?.message ?? "Loading TE Storage inventory…");

  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <header className="relative z-40 flex shrink-0 flex-wrap items-center gap-3 border-b border-border px-3 py-3 sm:px-5">
        <InventorySystemSwitcher value={activeViewId} onChange={onViewChange} />
        <span className="rounded-full border border-border px-2 py-0.5 text-[11px] text-muted-foreground" title={message}>{mode}</span>
        <div className="ml-auto flex items-center gap-2">
          <Button aria-label={theme === "light" ? "Dark Theme" : "Light Theme"} title={theme === "light" ? "Dark Theme" : "Light Theme"} variant="outline" size="icon" onClick={onThemeToggle}>
            {theme === "light" ? <MoonIcon className="size-3.5" /> : <SunIcon className="size-3.5" />}
          </Button>
          <Button disabled={!inventory.canModify} onClick={() => setEditor({ entry: null })}><PlusIcon className="size-4" />Add Entry</Button>
        </div>
      </header>
      <div className="flex min-h-0 flex-1 flex-col gap-2 px-2 py-2 sm:px-3">
        <div className="relative z-30 flex shrink-0 items-center gap-2 rounded-xl border border-border/70 bg-card p-2">
          <div className="relative min-w-0 flex-1">
            <SearchIcon aria-hidden className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input aria-label="Search TE Storage" className="pl-9" placeholder="Search by PN, PR, PO, manufacturer, model, description, location, or notes" value={query} onChange={event => setQuery(event.target.value)} />
          </div>
        </div>
        {inventory.error ? <p role="alert" className="px-2 text-sm text-destructive-foreground">{inventory.error}</p> : null}
        <div className="min-h-0 flex-1 overflow-auto rounded-xl border border-border/70 bg-card/80">
          <table aria-label="TE Storage inventory" className="w-full table-fixed border-collapse text-sm" style={{ minWidth: columns.reduce((width, column) => width + column.width, 0) }}>
            <colgroup>{columns.map(column => <col key={column.key} style={{ width: column.width }} />)}</colgroup>
            <thead className="sticky top-0 z-20 bg-card" onContextMenu={event => {
              event.preventDefault();
              event.stopPropagation();
              setColumnMenu({ x: event.clientX, y: event.clientY });
            }}>
              <tr>{columns.map(({ key, label }) => (
                <th key={key} scope="col" title="Right-click to show or hide columns" aria-sort={sort?.key === key ? sort.direction === "asc" ? "ascending" : "descending" : "none"}
                  className="border-b border-border p-0 text-center text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
                  <button className={`min-h-12 w-full px-2 py-3 hover:bg-accent/35 ${sort?.key === key ? "bg-accent/40 text-foreground" : ""}`}
                    onKeyDown={event => {
                      if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
                        event.preventDefault();
                        const rect = event.currentTarget.getBoundingClientRect();
                        setColumnMenu({ x: rect.left, y: rect.bottom });
                      }
                    }}
                    onClick={() => setSort(current => current?.key === key ? current.direction === "asc" ? { key, direction: "desc" } : null : { key, direction: "asc" })}>{label}</button>
                </th>
              ))}</tr>
            </thead>
            <tbody>
              {rows.map(entry => (
                <tr key={entry.entryUuid} className="border-b border-border/50 hover:bg-accent/25" onDoubleClick={() => { if (inventory.canModify) setEditor({ entry }); }}>
                  {columns.map(({ key }, index) => <td key={key} className="truncate px-3 py-2.5 text-center" title={String(entry[key] ?? "")}>
                    {index === 0 ? <button disabled={!inventory.canModify} aria-label={`Edit item ${entry.pn || entry.model || entry.id}`} className="w-full truncate font-medium hover:underline disabled:no-underline" onClick={() => setEditor({ entry })}>{entry[key] ?? "—"}{entry[key] === "" ? "—" : ""}</button> : entry[key] === null || entry[key] === "" ? <span className="text-muted-foreground">—</span> : entry[key]}
                  </td>)}
                </tr>
              ))}
              {rows.length === 0 ? <tr><td colSpan={columns.length} className="px-4 py-20 text-center">
                <BoxesIcon aria-hidden className="mx-auto mb-3 size-8 text-muted-foreground/60" />
                <p className="font-medium">{bridge && !inventory.loaded ? inventory.error ? "Could not load inventory" : "Loading storage items…" : query ? "No matching items" : "No storage items yet"}</p>
                <p className="mt-1 text-sm text-muted-foreground">{query ? "Try another search or clear the search field." : inventory.canModify ? "Use Add Entry to record your first item." : !bridge ? "Your TE Storage inventory will appear here." : ""}</p>
              </td></tr> : null}
            </tbody>
          </table>
        </div>
      </div>
      <ShellStatusStrip resultsLabel={`${rows.length} ${rows.length === 1 ? "item" : "items"}`} message={message} />
      {columnMenu ? <StorageColumnMenu anchor={columnMenu} visibility={visibility}
        onToggle={key => setVisibility(current => ({ ...current, [key]: !current[key] }))}
        onClose={() => setColumnMenu(null)} /> : null}
      {editor ? <StorageEntryDialog entry={editor.entry} onClose={() => setEditor(null)} onSave={inventory.save} onDelete={inventory.remove} /> : null}
    </section>
  );
}
