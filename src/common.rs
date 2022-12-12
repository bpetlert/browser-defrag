use std::{fs::File, io::Read, path::Path};

use anyhow::{Context, Result};
use tracing::trace;

/// Check whether a file is valid sqlite3 or not.
///
/// first 16 bytes => The header string: "SQLite format 3\000"
///
/// https://www.sqlite.org/fileformat.html
pub fn is_sqlite_file(path: &Path) -> Result<bool> {
    let header_size: usize = 16;
    let mut header: Vec<u8> = Vec::with_capacity(header_size);

    let mut file =
        File::open(path).with_context(|| format!("Could not open `{}`", path.display()))?;

    file.by_ref()
        .take(header_size as u64)
        .read_to_end(&mut header)
        .with_context(|| format!("Could not read header string of `{}`", path.display()))?;

    match std::str::from_utf8(&header).with_context(|| {
        format!(
            "Failed to convert string header `{:?}` of `{}` to UTF8",
            header,
            path.display()
        )
    }) {
        Ok(h) => {
            if h == "SQLite format 3\x00" {
                return Ok(true);
            }
        }
        Err(err) => {
            trace!("{err:#}");
            return Ok(false);
        }
    }

    Ok(false)
}
