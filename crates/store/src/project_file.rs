//! A project saved as one file and opened on another computer (#213): a SQLite database with
//! this store's own schema, holding one project's rows, and three tables of its own (the
//! manifest, the files it carries and their bytes). Its layout is in `docs/formats.md`.
//!
//! Saving reads the project's rows under the store's lock, so the file is one moment of the
//! project, then writes the files it carries without holding it. Opening checks the file,
//! writes the files it carries, then puts every row in one transaction, each identifier given a
//! new one, so the same file opened twice comes in beside itself and an open that fails leaves
//! the store as it was. Checking reads a file as opening would, writing nothing (#246); every
//! save ends with that check of the file written.

use std::path::Path;

use kalamu_core::project_file::{self as pf, ArchivePlaces, Manifest};
use kalamu_core::{
    ProjectArchive, ProjectFileCheck, ProjectFileError, ProjectFileOpened, ProjectFilePlan,
    ProjectFileProgress, ProjectFileSaved, ProjectId,
};
use rusqlite::Connection;

use super::SqliteStore;

mod carried;
mod check;
mod open;
mod remap;
mod rows;
mod save;
mod selection;

/// The file's own tables, beside the store's: the manifest (one row), the files it carries
/// (`role` is `snapshot` for Kalamu's frozen texts, `original` for the researcher's files;
/// `carried` is 0 for a file listed but left out) and their bytes.
const FILE_TABLES: &str = r#"
CREATE TABLE kalamu_project_file (
    format TEXT NOT NULL,
    version INTEGER NOT NULL,
    manifest TEXT NOT NULL
);
CREATE TABLE kalamu_files (
    id INTEGER PRIMARY KEY,
    role TEXT NOT NULL,
    path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size INTEGER NOT NULL,
    carried INTEGER NOT NULL
);
CREATE TABLE kalamu_file_chunks (
    file_id INTEGER NOT NULL REFERENCES kalamu_files(id),
    seq INTEGER NOT NULL,
    data BLOB NOT NULL,
    PRIMARY KEY (file_id, seq)
);
"#;

/// The store's tables a project file carries, in the order they are put back: each after the
/// tables it refers to.
const TABLES: &[&str] = &[
    "projects",
    "collections",
    "documents",
    "collection_documents",
    "jobs",
    "document_links",
    "preferred_results",
    "pieces",
    "results",
    "corrections",
    "speaker_names",
    "charges",
    "units",
    "snapshots",
    "passages",
    "drawn_regions",
    "region_readings",
    "review_samples",
    "review_kept",
    "review_set_aside",
    "dictionary_entries",
    "dictionary_apart",
    "dictionary_requests",
    "dictionary_dismissed",
];

/// The tables carried, for the tests.
#[cfg(test)]
pub(crate) fn tables() -> &'static [&'static str] {
    TABLES
}

/// The tables whose `id` is given a new one when a file is opened, wherever it appears.
const IDENTIFIED: &[&str] = &[
    "projects",
    "collections",
    "documents",
    "jobs",
    "snapshots",
    "units",
    "dictionary_entries",
    "dictionary_requests",
    "passages",
    "drawn_regions",
    "region_readings",
];

/// The models' answers, keyed by their fingerprint: taken as they are, never remapped, an answer
/// already in the store kept.
const RESULTS: &str = "results";

/// The bytes of a carried file are written in chunks of this size.
const CHUNK: usize = 4 << 20;

const ROLE_SNAPSHOT: &str = "snapshot";
const ROLE_ORIGINAL: &str = "original";

fn read_err(e: impl std::fmt::Display) -> ProjectFileError {
    ProjectFileError::Read(e.to_string())
}

fn write_err(e: impl std::fmt::Display) -> ProjectFileError {
    ProjectFileError::Write(e.to_string())
}

/// The schema a new project file starts from: the store's, then the file's own tables.
fn create_schema(conn: &Connection) -> Result<(), ProjectFileError> {
    super::migrate(conn).map_err(ProjectFileError::Store)?;
    conn.execute_batch(FILE_TABLES).map_err(write_err)
}

/// The manifest's schema number: the migrations this store knows.
fn schema() -> u32 {
    super::migrations::MIGRATIONS.len() as u32
}

impl ProjectArchive for SqliteStore {
    fn plan_project_file(
        &self,
        project: ProjectId,
        places: &ArchivePlaces,
    ) -> Result<ProjectFilePlan, ProjectFileError> {
        save::plan(self, project, places)
    }

    fn save_project_file(
        &self,
        project: ProjectId,
        path: &Path,
        with_originals: bool,
        places: &ArchivePlaces,
        progress: &dyn Fn(ProjectFileProgress),
    ) -> Result<ProjectFileSaved, ProjectFileError> {
        save::save(self, project, path, with_originals, places, progress)
    }

    fn open_project_file(
        &self,
        path: &Path,
        places: &ArchivePlaces,
        progress: &dyn Fn(ProjectFileProgress),
    ) -> Result<ProjectFileOpened, ProjectFileError> {
        open::open(self, path, places, progress)
    }

    fn check_project_file(
        &self,
        path: &Path,
        progress: &dyn Fn(ProjectFileProgress),
    ) -> Result<ProjectFileCheck, ProjectFileError> {
        check::check(path, progress)
    }
}

/// The manifest a file is saved with.
fn manifest(
    name: &str,
    documents: u32,
    jobs: u32,
    with_originals: bool,
    places: &ArchivePlaces,
) -> Manifest {
    Manifest {
        format: pf::FORMAT.to_string(),
        version: pf::VERSION,
        schema: schema(),
        app_version: places.app_version.clone(),
        saved_at: kalamu_core::Timestamp::now(),
        project_name: name.to_string(),
        documents,
        jobs,
        with_originals,
    }
}
