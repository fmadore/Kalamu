//! The files a project file carries: Kalamu's frozen texts always, the researcher's originals
//! when chosen. Each is written in chunks with its fingerprint checked: on saving, an original
//! changed since it was read is left out, and a frozen text missing or changed is listed so the
//! save fails (#239); on opening, a file whose bytes no longer match is a damaged file.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use kalamu_core::ProjectFileProgress;
use kalamu_core::{MissingText, ProjectFileError};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

use super::selection::Listed;
use super::{CHUNK, ROLE_ORIGINAL, ROLE_SNAPSHOT, read_err, write_err};

/// What writing the files came to.
#[derive(Debug, Default)]
pub(super) struct Written {
    /// Originals carried, and those that could not be (not found, or changed since read).
    pub originals: u32,
    pub originals_left: u32,
    /// The paths of the frozen texts no longer in the data folder, and of those whose bytes
    /// no longer match what the jobs read: either fails the save.
    pub frozen_missing: Vec<String>,
    pub frozen_changed: Vec<String>,
}

/// Writes the files listed into the project file: the frozen texts, and the originals when
/// `with_originals`.
pub(super) fn write_files(
    file: &Connection,
    listed: &[Listed],
    with_originals: bool,
    progress: &dyn Fn(ProjectFileProgress),
) -> Result<Written, ProjectFileError> {
    let wanted = |f: &Listed| f.role == ROLE_SNAPSHOT || with_originals;
    let total: u64 = listed
        .iter()
        .filter(|f| wanted(f))
        .filter_map(|f| std::fs::metadata(&f.path).ok().map(|m| m.len()))
        .sum();
    let mut done = 0;
    progress(ProjectFileProgress { done, total });
    let mut out = Written::default();
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
            if original {
                out.originals_left += 1;
            } else {
                out.frozen_missing.push(f.path.clone());
            }
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
            out.originals += u32::from(original);
        } else {
            tx.execute("DELETE FROM kalamu_file_chunks WHERE file_id = ?1", [id])
                .map_err(write_err)?;
            if original {
                out.originals_left += 1;
            } else {
                out.frozen_changed.push(f.path.clone());
            }
        }
        tx.commit().map_err(write_err)?;
    }
    Ok(out)
}

/// The frozen texts a file lists but does not carry, each with the documents and jobs that
/// read it, read from the file's own rows. A file saved since #239 has none: a save fails
/// rather than leave one out. One saved by 1.10.0 may.
pub(super) fn missing(file: &Connection) -> Result<Vec<MissingText>, ProjectFileError> {
    let mut stmt = file
        .prepare(
            "SELECT path, file_name FROM kalamu_files
             WHERE role = ?1 AND carried = 0 ORDER BY id",
        )
        .map_err(read_err)?;
    let left: Vec<(String, String)> = stmt
        .query_map([ROLE_SNAPSHOT], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(read_err)?
        .collect::<Result<_, _>>()
        .map_err(read_err)?;
    if left.is_empty() {
        return Ok(Vec::new());
    }
    // Each job's name and its sources, read loosely: only their ids and paths matter.
    let mut stmt = file
        .prepare("SELECT id, name, data FROM jobs ORDER BY created_at, id")
        .map_err(read_err)?;
    let jobs: Vec<(String, String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(read_err)?
        .collect::<Result<_, _>>()
        .map_err(read_err)?;
    let jobs: Vec<(String, String, serde_json::Value)> = jobs
        .into_iter()
        .filter_map(|(id, name, data)| Some((id, name, serde_json::from_str(&data).ok()?)))
        .collect();
    let mut titles = file
        .prepare(
            "SELECT DISTINCT d.title FROM document_links l JOIN documents d ON d.id = l.document_id
             WHERE l.job_id = ?1 AND l.source_id = ?2 ORDER BY d.title",
        )
        .map_err(read_err)?;
    let mut out = Vec::new();
    for (path, file_name) in left {
        let mut text = MissingText {
            documents: Vec::new(),
            jobs: Vec::new(),
            file_name,
        };
        for (id, name, job) in &jobs {
            let sources = job["sources"].as_array().map(Vec::as_slice).unwrap_or(&[]);
            let read: Vec<&str> = sources
                .iter()
                .filter(|s| s["path"].as_str() == Some(path.as_str()))
                .filter_map(|s| s["id"].as_str())
                .collect();
            if read.is_empty() {
                continue;
            }
            if !text.jobs.contains(name) {
                text.jobs.push(name.clone());
            }
            for source in read {
                let found = titles
                    .query_map(params![id, source], |row| row.get::<_, String>(0))
                    .map_err(read_err)?;
                for title in found {
                    let title = title.map_err(read_err)?;
                    if !text.documents.contains(&title) {
                        text.documents.push(title);
                    }
                }
            }
        }
        out.push(text);
    }
    Ok(out)
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
