//! What an inventory supplies to use the shared entry sync engine.

use crate::platform::{common::CommandResult, store::StoredEntry};

/// How a concurrent update to the same entry is merged. Each inventory keeps the rule it
/// shipped with in 0.1.4: clients on different versions sync the same shared folder during a
/// rollout, and both must reach the same result from the same operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MergeRule {
    /// TE Test Equipment. An update made from the same base version as the current state,
    /// touching different fields, is merged onto the current entry. Any other concurrent
    /// update is decided whole-entry by the operation order.
    DisjointFieldsFromSameBase,
    /// EVCS Storage. Every field remembers the operation that last set it, and an incoming
    /// update wins field by field. Entry states also keep the merged entry, so recovery can
    /// restore exactly what was applied.
    PerFieldVersions,
}

pub(crate) trait SyncRecord: StoredEntry {
    /// Inventory name used in log and status messages.
    const LABEL: &'static str;
    /// Environment variable that overrides this inventory's shared root.
    const SHARED_ROOT_ENV: &'static str;
    /// `schema_version` written to, and expected in, this inventory's operations and
    /// snapshots.
    const SYNC_SCHEMA_VERSION: u16;
    /// `entity_type` of this inventory's operations.
    const ENTITY_TYPE: &'static str;
    /// HMAC domain for operation files.
    const OPERATION_AUTH_DOMAIN: &'static str;
    const MERGE_RULE: MergeRule;
    /// Field names a create sets, in the spelling `changed_fields` uses after normalizing.
    const ENTRY_FIELDS: &'static [&'static str];

    fn validate(&self) -> CommandResult<()>;

    /// Maps the field names the frontend reports as changed onto the names
    /// [`SyncRecord::copy_fields`] understands; unknown and identity fields are dropped.
    fn normalize_changed_fields(fields: &[String]) -> Vec<String>;

    /// Copies the named fields from `source` onto `self`.
    fn copy_fields(&mut self, source: &Self, fields: &[String]);

    fn set_updated_at(&mut self, updated_at: String);

    /// Gives an incoming copy of an entry this computer's display id for it.
    fn take_local_id_from(&mut self, existing: &Self);

    /// Assigns a new local display id (an incoming entry whose id is taken here).
    fn set_local_id(&mut self, id: i64);
}

impl MergeRule {
    /// Only per-field merging needs the field versions and recovery entry on entry states.
    pub(crate) fn keeps_field_versions(self) -> bool {
        self == Self::PerFieldVersions
    }
}
