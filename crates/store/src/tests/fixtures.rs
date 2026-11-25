//! Databases written by earlier releases (`crates/store/fixtures`), each by that release's own
//! store crate (`fixtures/make.sh`): migrated to the current schema, then their job, its
//! answers, its correction and, from 1.1.0, its project read back whole.

use kalamu_core::{CacheKey, ProjectStore};

use super::*;

/// The releases that changed the schema or what a stored record holds. A release that changes
/// either adds its fixture (`fixtures/make.sh <tag>`) and its version here.
const FIXTURES: &[&str] = &[
    "1.0.0", "1.1.0", "1.2.0", "1.3.0", "1.4.0", "1.5.0", "1.6.0", "1.7.0", "1.8.0", "1.9.0",
    "1.10.0",
];

/// The first release with projects.
const PROJECTS_FROM: &str = "1.1.0";

fn fixture(version: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(format!("jobs-{version}.sqlite"))
}

/// A copy of the fixture to migrate, so the one in the repository is never changed.
fn copied(version: &str) -> Result<(tempfile::TempDir, std::path::PathBuf), StoreError> {
    let dir = tempfile::tempdir().map_err(|e| StoreError::Database(e.to_string()))?;
    let path = dir.path().join("jobs.sqlite");
    std::fs::copy(fixture(version), &path)
        .map_err(|e| StoreError::Database(format!("{version}: {e}")))?;
    Ok((dir, path))
}

#[test]
fn every_release_s_database_is_migrated_and_read_back() -> Result<(), StoreError> {
    for &version in FIXTURES {
        let (_dir, path) = copied(version)?;
        let store = SqliteStore::open(&path)?;
        let applied: i64 = store
            .conn()
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(db)?;
        assert_eq!(applied as usize, migrations::MIGRATIONS.len(), "{version}");

        // The job, whole, with its two pages read.
        assert_eq!(store.unreadable_jobs()?, vec![], "{version}");
        let listed = store.list_jobs()?;
        assert_eq!(listed.len(), 1, "{version}");
        let summary = &listed[0];
        assert_eq!(summary.name, "Letters", "{version}");
        assert_eq!(summary.status, JobStatus::Completed, "{version}");
        assert_eq!(
            (summary.pieces_finished, summary.pieces_total),
            (2, 2),
            "{version}"
        );
        assert!(
            summary.cost_usd.is_some_and(|c| (c - 0.02).abs() < 1e-9),
            "{version}"
        );
        let job = store.job(summary.id)?;
        assert_eq!(job.sources[0].file_name, "letter.pdf", "{version}");
        assert_eq!(job.setups[0].model, "gemini-3.8-flash", "{version}");
        let pieces = store.pieces(job.id)?;
        assert_eq!(pieces.len(), 2, "{version}");
        let answer = store
            .finished_result(&pieces[0])?
            .ok_or(StoreError::NotFound)?;
        assert_eq!(answer.output.text, "Page 1", "{version}");
        assert!(
            store
                .cached_result(&CacheKey("key-1".into()))?
                .is_some_and(|r| r.output.text == "Page 2"),
            "{version}"
        );

        // The researcher's correction, on top of the answer.
        let corrections = store.corrections(job.id)?;
        assert_eq!(corrections.len(), 1, "{version}");
        assert_eq!(
            (corrections[0].text.as_deref(), corrections[0].reviewed),
            (Some("Corrected page 1"), true),
            "{version}"
        );

        // The project, its collection, document, link and chosen result.
        let projects = store.projects()?;
        if version_before(version, PROJECTS_FROM) {
            assert!(projects.is_empty(), "{version}");
            continue;
        }
        assert_eq!(projects.len(), 1, "{version}");
        let project = &projects[0];
        assert_eq!(project.name, "Mission archives", "{version}");
        let documents = store.documents(project.id)?;
        assert_eq!(documents.len(), 1, "{version}");
        assert_eq!(documents[0].title, "Letter of 1915", "{version}");
        assert_eq!(documents[0].tags, vec!["Letters".to_string()], "{version}");
        assert_eq!(
            store.collections(project.id)?[0].name,
            "Letters",
            "{version}"
        );
        assert_eq!(store.members(project.id)?.len(), 1, "{version}");
        let links = store.links(project.id)?;
        assert_eq!(
            links.iter().map(|l| l.job_id).collect::<Vec<_>>(),
            vec![job.id],
            "{version}"
        );
        let preferred = store.preferred(project.id)?;
        assert_eq!(preferred.len(), 1, "{version}");
        assert_eq!(preferred[0].task, Task::TextExtraction, "{version}");
    }
    Ok(())
}

/// Opened again, a migrated database is left as it is, and keeps working: a job added beside
/// the old one, a correction changed.
#[test]
fn a_migrated_database_takes_new_work() -> Result<(), StoreError> {
    let (_dir, path) = copied(FIXTURES[0])?;
    let old = {
        let store = SqliteStore::open(&path)?;
        store.list_jobs()?[0].id
    };
    let store = SqliteStore::open(&path)?;
    let added = job();
    store.create_job(&added)?;
    let mut correction = store.corrections(old)?.remove(0);
    correction.text = Some("Corrected again".into());
    store.put_correction(&correction)?;
    assert_eq!(store.list_jobs()?.len(), 2);
    assert_eq!(
        store.corrections(old)?[0].text.as_deref(),
        Some("Corrected again")
    );
    Ok(())
}

/// Every fixture listed is in the folder, and every one in the folder is listed.
#[test]
fn the_fixtures_listed_are_the_fixtures_kept() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let mut kept: Vec<String> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|e| {
                    let name = e.file_name().into_string().ok()?;
                    Some(
                        name.strip_prefix("jobs-")?
                            .strip_suffix(".sqlite")?
                            .to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    kept.sort_by_key(|v| version_order(v));
    assert_eq!(kept, FIXTURES);
}

fn version_order(version: &str) -> Vec<u32> {
    version.split('.').filter_map(|n| n.parse().ok()).collect()
}

fn version_before(version: &str, other: &str) -> bool {
    version_order(version) < version_order(other)
}
