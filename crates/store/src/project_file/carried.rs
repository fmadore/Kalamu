//! The files a project file carries: Kalamu's frozen texts always, the researcher's originals
//! when chosen. Each is written in chunks with its fingerprint checked: on saving, a file
//! changed since it was read is left out; on opening, a file whose bytes no longer match is a
//! damaged file.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use kalamu_core::ProjectFileError;
use kalamu_core::ProjectFileProgress;
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

use super::selection::Listed;
use super::{CHUNK, ROLE_ORIGINAL, ROLE_SNAPSHOT, read_err, write_err};

/// Writes the files listed into the project file: the frozen texts, and the originals when
/// `with_originals`. Returns how many originals it carries and how many it could not (not found,
/// or changed since they were read).
pub(super) fn write_files(
    file: &Connection,
    listed: &[Listed],
    with_originals: bool,
    progress: &dyn Fn(ProjectFileProgress),
) -> Result<(u32, u32), ProjectFileError> {
    let wanted = |f: &Listed| f.role == ROLE_SNAPSHOT || with_originals;
    let total: u64 = listed
        .iter()
        .filter(|f| wanted(f))
        .filter_map(|f| std::fs::metadata(&f.path).ok().map(|m| m.len()))
        .sum();
    let mut done = 0;
    progress(ProjectFileProgress { done, total });
    let (mut carried, mut left) = (0, 0);
    for f in listed {
        file.execute(
            "INSERT INTO kalamu_files (role, path, file_name, sha256, size, carried)
             VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            params![f.role, f.path, f.file_name, f.sha256, f.size as i64],
        )
        .map_err(write_err)?;
        let id = file.last_insert_rowid();
        let original = f.role == ROLE_ORIGINAL;
        if !wanted(f) {
            continue;
        }
        let Ok(mut reader) = std::fs::File::open(&f.path) else {
            left += u32::from(original);
            continue;
        };
        let tx = file.unchecked_transaction().map_err(write_err)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; CHUNK];
        let mut seq = 0;
        loop {
            let n = fill(&mut reader, &mut buffer).map_err(write_err)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
            tx.execute(
                "INSERT INTO kalamu_file_chunks (file_id, seq, data) VALUES (?1, ?2, ?3)",
                params![id, seq, &buffer[..n]],
            )
            .map_err(write_err)?;
            seq += 1;
            done += n as u64;
            progress(ProjectFileProgress { done, total });
        }
        // A file changed since it was read is not the one the results were read from.
        if hex(hasher.finalize().as_slice()) == f.sha256 {
            tx.execute("UPDATE kalamu_files SET carried = 1 WHERE id = ?1", [id])
                .map_err(write_err)?;
            carried += u32::from(original);
        } else {
            tx.execute("DELETE FROM kalamu_file_chunks WHERE file_id = ?1", [id])
                .map_err(write_err)?;
            left += u32::from(original);
        }
        tx.commit().map_err(write_err)?;
    }
    Ok((carried, left))
}

/// Reads until `buffer` is full or the file ends.
fn fill(reader: &mut impl Read, buffer: &mut [u8]) -> std::io::Result<usize> {
    let mut n = 0;
    while n < buffer.len() {
        match reader.read(&mut buffer[n..])? {
            0 => break,
            read => n += read,
        }
    }
    Ok(n)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// A file the project file carries, as listed in it.
pub(super) struct Carried {
    pub id: i64,
    pub role: String,
    pub path: String,
    pub file_name: String,
    pub sha256: String,
    pub size: u64,
}

pub(super) fn carried(file: &Connection) -> Result<Vec<Carried>, ProjectFileError> {
    let mut stmt = file
        .prepare(
            "SELECT id, role, path, file_name, sha256, size FROM kalamu_files
             WHERE carried = 1 ORDER BY id",
        )
        .map_err(read_err)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Carried {
                id: row.get(0)?,
                role: row.get(1)?,
                path: row.get(2)?,
                file_name: row.get(3)?,
                sha256: row.get(4)?,
                size: row.get::<_, i64>(5)?.max(0) as u64,
            })
        })
        .map_err(read_err)?;
    rows.collect::<Result<_, _>>().map_err(read_err)
}

/// Writes one carried file to `to`, through a part renamed once its bytes match its
/// fingerprint. `done` counts the bytes written so far, across files.
pub(super) fn extract(
    file: &Connection,
    carried: &Carried,
    to: &Path,
    done: &mut u64,
    total: u64,
    progress: &dyn Fn(ProjectFileProgress),
) -> Result<(), ProjectFileError> {
    if let Some(folder) = to.parent() {
        std::fs::create_dir_all(folder).map_err(write_err)?;
    }
    let part = part_of(to);
    let written = (|| {
        let mut out = std::fs::File::create(&part).map_err(write_err)?;
        let mut hasher = Sha256::new();
        let mut stmt = file
            .prepare("SELECT data FROM kalamu_file_chunks WHERE file_id = ?1 ORDER BY seq")
            .map_err(read_err)?;
        let mut rows = stmt.query([carried.id]).map_err(read_err)?;
        while let Some(row) = rows.next().map_err(read_err)? {
            let data: Vec<u8> = row.get(0).map_err(read_err)?;
            hasher.update(&data);
            out.write_all(&data).map_err(write_err)?;
            *done += data.len() as u64;
            progress(ProjectFileProgress { done: *done, total });
        }
        out.sync_all().map_err(write_err)?;
        if hex(hasher.finalize().as_slice()) != carried.sha256 {
            return Err(ProjectFileError::Damaged(format!(
                "{} does not match its fingerprint",
                carried.file_name
            )));
        }
        drop(out);
        std::fs::rename(&part, to).map_err(write_err)
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    written
}

fn part_of(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    path.with_file_name(name)
}
