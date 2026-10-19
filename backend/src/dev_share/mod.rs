//! The developer test share: a folder laid out like the team share (`modules\<inventory>\`)
//! holding fixture records that cover every field and state of every inventory. Three fake
//! computers write the fixtures through the real mutations and sync engine, so the folder
//! holds the same operation files, snapshots, and manifests as the live share, including
//! concurrent edits and deletes.
//!
//! `bun run dev:share` rebuilds `.tmp/dev-share`, the folder developer builds sync with, and
//! moves `.tmp/dev-data` aside so the next developer build starts from the share. Set
//! `INVENTORY_MANAGEMENT_DEV_SHARE_TARGET` to build somewhere else, such as an S: dev folder
//! for two-PC testing.
//!
//! **Every new field or feature adds fixtures here in the same change.** The coverage tests
//! fail when a synced field is never set by any fixture.

mod te_lab_components;
mod te_storage;
mod te_test_equipment;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    modules::{
        te_lab_components::store::InventoryDb as LabDb,
        te_storage::store::InventoryDb as StorageDb, te_test_equipment::store::InventoryDb as TeDb,
    },
    platform::{
        common::{CommandResult, InventorySharedStatus},
        dev_build, module_shared_root, team_share_root, LocalDatabase, ModuleId,
    },
};

const TARGET_ENV: &str = "INVENTORY_MANAGEMENT_DEV_SHARE_TARGET";
/// Written at the root of every dev share. A rebuild only clears a folder that has it (or
/// the developer builds' own `.tmp/dev-share`).
const MARKER_FILE: &str = "DEV-SHARE.md";
/// Each fake computer's database. A fixture run writes a few thousand records.
const COMPUTER_DATABASE_SIZE: u64 = 64 * 1024 * 1024;

/// One fake team member's computer: every inventory in one local database, as in 0.2.
pub(super) struct Computer {
    pub(super) te: TeDb,
    pub(super) lab: LabDb,
    pub(super) storage: StorageDb,
}

impl Computer {
    fn open(folder: &Path, name: &str) -> CommandResult<Self> {
        let database = LocalDatabase::open_file(
            folder.join(name).join("inventory-management.feox"),
            COMPUTER_DATABASE_SIZE,
        )
        .map_err(|error| format!("Could not open {name}'s database: {error}"))?;
        Ok(Self {
            te: TeDb::open_in(&database),
            lab: LabDb::open_in(&database),
            storage: StorageDb::open_in(&database),
        })
    }
}

/// The fake team. Alice, Bob, and Carol each start one inventory, so every share has
/// snapshots and operations from more than one client.
pub(super) struct Team {
    pub(super) alice: Computer,
    pub(super) bob: Computer,
    pub(super) carol: Computer,
    folder: PathBuf,
}

impl Team {
    fn new() -> CommandResult<Self> {
        let folder = env::temp_dir().join(format!("inventory-dev-share-{}", Uuid::new_v4()));
        Ok(Self {
            alice: Computer::open(&folder, "alice")?,
            bob: Computer::open(&folder, "bob")?,
            carol: Computer::open(&folder, "carol")?,
            folder,
        })
    }

    fn everyone(&self) -> [&Computer; 3] {
        [&self.alice, &self.bob, &self.carol]
    }
}

impl Drop for Team {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.folder);
    }
}

/// Each inventory's folder in the share being built.
pub(super) struct ShareRoots {
    pub(super) te: PathBuf,
    pub(super) lab: PathBuf,
    pub(super) storage: PathBuf,
}

impl ShareRoots {
    fn in_share(share: &Path) -> Self {
        Self {
            te: module_shared_root(share, ModuleId::TeTestEquipment),
            lab: module_shared_root(share, ModuleId::TeLabComponents),
            storage: module_shared_root(share, ModuleId::TeStorage),
        }
    }

    fn all(&self) -> [&Path; 3] {
        [&self.te, &self.lab, &self.storage]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevShareSummary {
    pub te_entries: usize,
    pub lab_parts: usize,
    pub lab_placements: usize,
    pub storage_entries: usize,
}

/// Builds a dev share at `target`, replacing a previous dev share there.
pub(crate) fn build_dev_share(target: &Path) -> CommandResult<DevShareSummary> {
    refuse_live_share(target)?;
    clear_target(target)?;
    let roots = ShareRoots::in_share(target);
    for root in roots.all() {
        fs::create_dir_all(root)
            .map_err(|error| format!("Could not create {}: {error}", root.display()))?;
    }

    let team = Team::new()?;
    te_test_equipment::write(&team, &roots.te)?;
    te_storage::write(&team, &roots.storage)?;
    te_lab_components::write(&team, &roots.lab)?;

    let summary = DevShareSummary {
        te_entries: team.alice.te.load_entries()?.len(),
        lab_parts: team.alice.lab.load_parts()?.len(),
        lab_placements: team.alice.lab.load_stock_placements()?.len(),
        storage_entries: team.alice.storage.load_entries()?.len(),
    };
    fs::write(target.join(MARKER_FILE), marker_text(target, &summary))
        .map_err(|error| format!("Could not write {MARKER_FILE}: {error}"))?;
    Ok(summary)
}

/// Fails the build when a sync did not reach the share; fixtures must land there.
pub(super) fn require_synced(shared: &InventorySharedStatus, what: &str) -> CommandResult<()> {
    if !shared.available || shared.has_local_only_changes == Some(true) || shared.warning.is_some()
    {
        return Err(format!(
            "{what} did not sync cleanly: {} {:?}",
            shared.message, shared.warning
        ));
    }
    Ok(())
}

/// The record as JSON with this computer's display ids removed, for comparing computers.
pub(super) fn without_local_ids<T: Serialize>(record: &T) -> Value {
    let mut value = serde_json::to_value(record).expect("records serialize");
    if let Value::Object(fields) = &mut value {
        fields.remove("id");
        fields.remove("databaseId");
    }
    value
}

/// Text of exactly `length` characters that starts with `label`.
pub(super) fn text_of_length(label: &str, length: usize) -> String {
    label
        .chars()
        .chain("·max".chars().cycle())
        .take(length)
        .collect()
}

/// Never build into the live team share, or into a folder that holds it.
fn refuse_live_share(target: &Path) -> CommandResult<()> {
    let target_text = comparable_path(target);
    let live = comparable_path(team_share_root());
    let live_modules = comparable_path(&team_share_root().join("modules"));
    let inside =
        |outer: &str, inner: &str| inner == outer || inner.starts_with(&format!("{outer}\\"));
    if inside(&target_text, &live) || inside(&live_modules, &target_text) {
        return Err(format!(
            "{} is the live team share or holds it. Build the dev share in its own folder, for example {}\\dev-share.",
            target.display(),
            team_share_root().display()
        ));
    }
    if !target.is_absolute() {
        return Err(format!("{} must be an absolute path.", target.display()));
    }
    Ok(())
}

fn comparable_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

/// Clears a previous dev share. Any other non-empty folder is left alone.
fn clear_target(target: &Path) -> CommandResult<()> {
    if !target.exists() {
        return Ok(());
    }
    let is_empty = fs::read_dir(target)
        .map_err(|error| format!("Could not read {}: {error}", target.display()))?
        .next()
        .is_none();
    let is_dev_share = target.join(MARKER_FILE).is_file() || target == dev_build::dev_share_dir();
    if !is_empty && !is_dev_share {
        return Err(format!(
            "{} is not empty and has no {MARKER_FILE}, so it is not a dev share. Choose an empty folder.",
            target.display()
        ));
    }
    fs::remove_dir_all(target).map_err(|error| {
        format!(
            "Could not clear {}: {error}. Close any developer build using it first.",
            target.display()
        )
    })
}

fn marker_text(target: &Path, summary: &DevShareSummary) -> String {
    let root = |module| module_shared_root(target, module).display().to_string();
    format!(
        "# Inventory Management developer test share\n\n\
         Built {built} by `bun run dev:share` (`backend/src/dev_share/`). Fixture data only; \
         never copy it into the live share. Rebuilding clears this folder.\n\n\
         - TE Test Equipment: {te} entries\n\
         - TE Lab Components: {parts} parts in {placements} stock placements\n\
         - EVCS Storage: {storage} entries\n\n\
         Calibration states are relative to the build date, so rebuild to refresh them.\n\n\
         ## Point a developer build at this share\n\n\
         Developer builds use `<repo>/.tmp/dev-share` by default. For any other folder (for \
         example two PCs testing through S:), set before `bun run dev:desktop`:\n\n\
         ```powershell\n\
         $env:INVENTORY_MANAGEMENT_SHARED_ROOT = '{te_root}'\n\
         $env:INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT = '{lab_root}'\n\
         $env:INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT = '{storage_root}'\n\
         ```\n",
        built = crate::platform::common::now_timestamp(),
        te = summary.te_entries,
        parts = summary.lab_parts,
        placements = summary.lab_placements,
        storage = summary.storage_entries,
        te_root = root(ModuleId::TeTestEquipment),
        lab_root = root(ModuleId::TeLabComponents),
        storage_root = root(ModuleId::TeStorage),
    )
}

/// Moves `.tmp/dev-data` aside, since it holds sync state for the share being replaced.
fn set_aside_dev_data() -> CommandResult<Option<PathBuf>> {
    let data = dev_build::dev_local_data_dir();
    if !data.exists() {
        return Ok(None);
    }
    let previous = data.with_file_name("dev-data-previous");
    if previous.exists() {
        fs::remove_dir_all(&previous)
            .map_err(|error| format!("Could not clear {}: {error}", previous.display()))?;
    }
    fs::rename(&data, &previous).map_err(|error| {
        format!(
            "Could not move {} aside: {error}. Close the developer build first.",
            data.display()
        )
    })?;
    Ok(Some(previous))
}

/// `bun run dev:share`. Ignored, so `cargo test` never rewrites the dev share.
#[test]
#[ignore = "rebuilds the developer test share; run it with `bun run dev:share`"]
fn build_the_dev_share() {
    let target = env::var_os(TARGET_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(dev_build::dev_share_dir);
    let set_aside = if target == dev_build::dev_share_dir() {
        set_aside_dev_data().unwrap()
    } else {
        None
    };

    let summary = build_dev_share(&target).unwrap();

    println!("Dev share built at {}: {summary:?}", target.display());
    if let Some(previous) = set_aside {
        println!(
            "The previous developer database moved to {}.",
            previous.display()
        );
    }
}

mod tests {
    use super::*;

    fn scratch_folder() -> PathBuf {
        env::temp_dir().join(format!("dev-share-test-{}", Uuid::new_v4()))
    }

    #[test]
    fn a_new_computer_syncs_every_fixture_from_the_dev_share() {
        let target = scratch_folder();

        let summary = build_dev_share(&target).unwrap();

        assert!(target.join(MARKER_FILE).is_file());
        let roots = ShareRoots::in_share(&target);
        let folder = scratch_folder();
        let dana = Computer::open(&folder, "dana").unwrap();
        te_test_equipment::assert_matches_share(&dana, &roots.te, summary.te_entries);
        te_storage::assert_matches_share(&dana, &roots.storage, summary.storage_entries);
        te_lab_components::assert_matches_share(&dana, &roots.lab, &summary);
        let _ = fs::remove_dir_all(&target);
        let _ = fs::remove_dir_all(&folder);
    }

    #[test]
    fn the_share_holds_snapshots_and_operations_from_several_computers() {
        let target = scratch_folder();
        build_dev_share(&target).unwrap();

        for root in ShareRoots::in_share(&target).all() {
            let shared = root.join("shared");
            let files = walk(&shared);
            assert!(
                files.iter().any(|path| path.ends_with("manifest.json")),
                "{} has no manifest",
                root.display()
            );
            assert!(
                files.iter().any(|path| path.contains("snapshot-")),
                "{} has no snapshot",
                root.display()
            );
            assert!(
                files
                    .iter()
                    .filter(|path| path.ends_with(".json") && path.contains("ops"))
                    .count()
                    > 1,
                "{} has no operations after its snapshot",
                root.display()
            );
        }
        let _ = fs::remove_dir_all(&target);
    }

    #[test]
    fn rebuilding_replaces_a_previous_dev_share_but_never_another_folder() {
        let target = scratch_folder();
        build_dev_share(&target).unwrap();
        build_dev_share(&target).unwrap();

        let other = scratch_folder();
        fs::create_dir_all(&other).unwrap();
        fs::write(other.join("notes.txt"), "keep me").unwrap();
        assert!(build_dev_share(&other).is_err());
        assert!(other.join("notes.txt").is_file());

        let _ = fs::remove_dir_all(&target);
        let _ = fs::remove_dir_all(&other);
    }

    #[test]
    fn the_live_team_share_is_refused() {
        let live = team_share_root();
        for target in [
            live.to_path_buf(),
            live.join("modules"),
            live.join("modules").join("TE_Test_Equipment"),
            live.parent().unwrap().to_path_buf(),
            PathBuf::from(live.to_string_lossy().to_uppercase() + "\\"),
        ] {
            assert!(refuse_live_share(&target).is_err(), "{}", target.display());
        }
        assert!(refuse_live_share(&live.join("dev-share")).is_ok());
    }

    #[test]
    #[should_panic(expected = "No demo fixture sets `notes`")]
    fn a_field_no_fixture_sets_fails_the_coverage_check() {
        let records = [serde_json::json!({ "qty": 0, "notes": "", "archived": false })];
        assert_every_field_is_set("demo", &records, &["qty".into(), "notes".into()]);
    }

    #[test]
    fn text_of_length_is_exact_in_characters() {
        let text = text_of_length("Ω label", 512);
        assert_eq!(text.chars().count(), 512);
        assert!(text.starts_with("Ω label"));
    }

    fn walk(folder: &Path) -> Vec<String> {
        let mut files = Vec::new();
        for entry in fs::read_dir(folder).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk(&path));
            } else {
                files.push(path.to_string_lossy().replace('\\', "/"));
            }
        }
        files
    }
}

/// Checks that every field a fixture can set is set by at least one record. A field counts
/// as set when it is not null, not an empty string, list, or map, and not `false`.
pub(super) fn assert_every_field_is_set(what: &str, records: &[Value], fields: &[String]) {
    for field in fields {
        let set = records.iter().any(|record| match record.get(field) {
            None | Some(Value::Null) | Some(Value::Bool(false)) => false,
            Some(Value::String(text)) => !text.is_empty(),
            Some(Value::Array(items)) => !items.is_empty(),
            Some(Value::Object(fields)) => !fields.is_empty(),
            Some(_) => true,
        });
        assert!(
            set,
            "No {what} fixture sets `{field}`. Add one in backend/src/dev_share/."
        );
    }
}
