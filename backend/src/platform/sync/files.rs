//! Format-independent helpers for files written to a shared folder: canonical JSON, the
//! `sha256:` checksums every shared file carries, and writes that another computer never sees
//! half done. Both sync streams (the entry engine and the Lab catalog) use these, so their
//! files hash the same way. What each stream writes, and in which JSON layout, stays with it.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

pub(crate) const CHECKSUM_PREFIX: &str = "sha256:";

/// JSON with every object's keys sorted, so the same value always has the same bytes.
pub(crate) fn canonical_json_bytes<T: Serialize>(value: &T) -> serde_json::Result<Vec<u8>> {
    serde_json::to_vec(&canonicalize(serde_json::to_value(value)?))
}

/// Canonical JSON without the `checksum` and `auth` fields, which are computed over the rest.
pub(crate) fn canonical_json_bytes_without_checksum_or_auth<T: Serialize>(
    value: &T,
) -> serde_json::Result<Vec<u8>> {
    let mut value = serde_json::to_value(value)?;
    if let Value::Object(object) = &mut value {
        object.remove("checksum");
        object.remove("auth");
    }
    serde_json::to_vec(&canonicalize(value))
}

/// The `checksum` a shared file stores for its own content.
pub(crate) fn content_checksum<T: Serialize>(value: &T) -> serde_json::Result<String> {
    Ok(format!(
        "{CHECKSUM_PREFIX}{}",
        sha256_hex(&canonical_json_bytes_without_checksum_or_auth(value)?)
    ))
}

pub(crate) fn sha256_digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Lowercase hex SHA-256.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Writes `bytes` to `temp_path`, flushes it to disk, then renames it to `final_path`, so a
/// reader on another computer sees either the whole file or none of it. An existing
/// `final_path` is replaced only when `replace_existing` is set; otherwise it is an error.
/// The temporary file is removed if anything fails.
pub(crate) fn write_file_atomically(
    temp_path: &Path,
    final_path: &Path,
    bytes: &[u8],
    replace_existing: bool,
) -> io::Result<()> {
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        if final_path.exists() {
            if !replace_existing {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("File already exists: {}", final_path.display()),
                ));
            }
            fs::remove_file(final_path)?;
        }
        fs::rename(temp_path, final_path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp_path);
    }
    result
}

fn canonicalize(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize).collect()),
        Value::Object(object) => {
            let mut keys = object.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut sorted = Map::new();
            for key in keys {
                if let Some(value) = object.get(&key) {
                    sorted.insert(key, canonicalize(value.clone()));
                }
            }
            Value::Object(sorted)
        }
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use uuid::Uuid;

    #[test]
    fn sha256_matches_the_standard_test_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(sha256_hex(&[0u8; 1000]).len(), 64);
    }

    #[test]
    fn canonical_json_sorts_keys_at_every_level_and_drops_checksum_and_auth() {
        let value = serde_json::json!({
            "b": 1,
            "a": { "d": [{ "z": 1, "y": 2 }], "c": null },
            "checksum": "sha256:old",
            "auth": "signature",
        });

        assert_eq!(
            String::from_utf8(canonical_json_bytes(&value).unwrap()).unwrap(),
            r#"{"a":{"c":null,"d":[{"y":2,"z":1}]},"auth":"signature","b":1,"checksum":"sha256:old"}"#
        );
        assert_eq!(
            String::from_utf8(canonical_json_bytes_without_checksum_or_auth(&value).unwrap())
                .unwrap(),
            r#"{"a":{"c":null,"d":[{"y":2,"z":1}]},"b":1}"#
        );
    }

    #[test]
    fn atomic_writes_create_or_replace_only_as_asked() {
        let folder = env::temp_dir().join(format!("shared-files-{}", Uuid::new_v4()));
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("file.json");
        let temp = |name: &str| folder.join(format!("{name}.tmp"));

        write_file_atomically(&temp("first"), &path, b"one", false).unwrap();
        assert!(write_file_atomically(&temp("second"), &path, b"two", false).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"one");
        write_file_atomically(&temp("third"), &path, b"three", true).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"three");
        assert_eq!(fs::read_dir(&folder).unwrap().count(), 1);
        let _ = fs::remove_dir_all(&folder);
    }
}
