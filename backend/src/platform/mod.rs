mod module_id;
mod shared_root;

pub(crate) use module_id::ModuleId;
pub(crate) use shared_root::default_shared_root;
#[cfg(test)]
pub(crate) use shared_root::{DEFAULT_LAB_COMPONENTS_SHARED_ROOT, DEFAULT_SHARED_ROOT};
