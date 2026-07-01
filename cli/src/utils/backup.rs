use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The backup path for a file: the original path with `.bak` appended to its
/// name (e.g. `~/.ssh/authorized_keys` -> `~/.ssh/authorized_keys.bak`).
fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".bak");
    PathBuf::from(name)
}

/// Back up an existing file before it is overwritten, copying it to a sibling
/// `<name>.bak`.
///
/// This is a no-op when the file does not yet exist (nothing to preserve).
/// Returns the backup path when a backup was made, or `None` otherwise.
pub fn backup_existing_file(path: &Path) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    let backup = backup_path(path);
    std::fs::copy(path, &backup).with_context(|| {
        format!(
            "Failed to back up existing file from {} to {}",
            path.display(),
            backup.display()
        )
    })?;

    Ok(Some(backup))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_backup_path_appends_bak() {
        assert_eq!(
            backup_path(Path::new("/home/user/.ssh/authorized_keys")),
            PathBuf::from("/home/user/.ssh/authorized_keys.bak")
        );
        // An existing extension is preserved, not replaced.
        assert_eq!(
            backup_path(Path::new("/tmp/known_hosts.d")),
            PathBuf::from("/tmp/known_hosts.d.bak")
        );
    }

    #[test]
    fn test_backup_existing_file_copies_contents() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("authorized_keys");
        fs::write(&path, "original contents\n").unwrap();

        let backup = backup_existing_file(&path).unwrap();

        let backup = backup.expect("a backup should be made for an existing file");
        assert_eq!(backup, path.with_file_name("authorized_keys.bak"));
        assert_eq!(fs::read_to_string(&backup).unwrap(), "original contents\n");
        // The original file is left in place for the caller to overwrite.
        assert_eq!(fs::read_to_string(&path).unwrap(), "original contents\n");
    }

    #[test]
    fn test_backup_existing_file_noop_when_absent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("does_not_exist");

        let backup = backup_existing_file(&path).unwrap();

        assert!(backup.is_none());
        assert!(!path.with_file_name("does_not_exist.bak").exists());
    }
}
