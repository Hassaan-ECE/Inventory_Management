//! TE Lab Components' store: the shared [`ModuleStore`] holding the Lab catalog (parts,
//! storage areas, containers, and stock placements).

use crate::platform::{
    store::{ModuleRecord, ModuleStore},
    ModuleId,
};

mod catalog;
mod keys;
mod metadata;
#[cfg(test)]
mod tests;

pub(crate) use crate::platform::store::SyncKeyspace;

/// Marks the Lab namespace; the catalog has no single entry type.
pub(crate) struct LabCatalog;

impl ModuleRecord for LabCatalog {
    const MODULE: ModuleId = ModuleId::TeLabComponents;
}

pub(crate) type InventoryDb = ModuleStore<LabCatalog>;
