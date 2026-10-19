use super::super::files::{
    canonical_json_bytes, canonical_json_bytes_without_checksum_or_auth, content_checksum,
};
use super::super::SyncRecord;
use super::super::{auth, SyncCoreResult, SyncOperationEnvelope};

pub(crate) fn canonical_operation_checksum<R: SyncRecord>(
    operation: &SyncOperationEnvelope<R>,
) -> SyncCoreResult<String> {
    Ok(content_checksum(operation)?)
}
pub(crate) fn canonical_operation_json<R: SyncRecord>(
    operation: &SyncOperationEnvelope<R>,
) -> SyncCoreResult<Vec<u8>> {
    Ok(canonical_json_bytes(operation)?)
}
pub(crate) fn sign_operation_for_configured_trust<R: SyncRecord>(
    operation: &mut SyncOperationEnvelope<R>,
) -> SyncCoreResult<()> {
    operation.auth = None;
    let bytes = canonical_json_bytes_without_checksum_or_auth(operation)?;
    operation.auth = auth::sign_canonical_bytes(R::OPERATION_AUTH_DOMAIN, &bytes)?;
    Ok(())
}
