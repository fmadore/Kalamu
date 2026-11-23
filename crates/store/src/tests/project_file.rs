//! A project saved to a file and opened again (#213): everything the researcher did comes back,
//! nothing of another project or of the keychain goes, and the same file opened twice comes in
//! beside itself.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use kalamu_core::{
    ArchivePlaces, Collection, CollectionId, DerivedFrom, DescriptionField, DictionaryStore,
    Document, DocumentId, DocumentLink, Entry, EntryId, KeptApart, KeptAside, Project,
    ProjectArchive, ProjectFileError, ProjectFileOpened, ProjectId, ProjectStore, SampleSpec,
    Snapshot, SpeakerNames, StoredResult,
};

use super::*;

mod failures;
mod frozen;
mod paths;
mod statuses;

/// A secret placed everywhere the store could hold one outside the project: none of it may
/// reach the file.
const SECRET: &str = "sk-proj-SECRET-0123456789";

pub(super) struct Fixture {
    pub dir: tempfile::TempDir,
    pub store: SqliteStore,
    pub project: ProjectId,
    pub letter: PathBuf,
    pub job: Job,
    pub derived: Job,
}

impl Fixture {
    pub fn places(&self, name: &str) -> ArchivePlaces {
        places(self.dir.path(), name)
    }

    pub fn saved(&self, with_originals: bool) -> Result<PathBuf, ProjectFileError> {
        let path = self.dir.path().join("Mission letters.kalamu");
        self.store.save_project_file(
            self.project,
            &path,
            with_originals,
            &self.places("here"),
            &|_| {},
        )?;
        Ok(path)
    }
}

pub(super) fn places(root: &Path, name: &str) -> ArchivePlaces {
    ArchivePlaces {
        snapshots: root.join(name).join("snapshots"),
        originals: root.join(name).join("originals"),
        app_version: "1.9.0".into(),
    }
}

fn file_source(path: &Path, bytes: &[u8]) -> Result<SourceFile, StoreError> {
    std::fs::create_dir_all(path.parent().unwrap_or(Path::new("."))).map_err(corrupt)?;
    std::fs::write(path, bytes).map_err(corrupt)?;
    let mut source = job().sources[0].clone();
    source.id = SourceId::new();
    source.path = path.display().to_string();
    source.file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    source.sha256 = kalamu_core::fingerprint::sha256_hex(bytes);
    source.size_bytes = bytes.len() as u64;
    Ok(source)
}

fn project(name: &str) -> Project {
    Project {
        id: ProjectId::new(),
        name: name.into(),
        description: "Letters of the mission, 1912–1960".into(),
        archived: true,
        source_defaults: BTreeMap::from([(DescriptionField::Repository, "ANOM".into())]),
        created_at: Timestamp(1),
        updated_at: Timestamp(1),
    }
}

fn result(key: &str, text: &str) -> StoredResult {
    StoredResult {
        cache_key: key.into(),
        provider: ProviderId::new("gemini"),
        model: "gemini-3.8-flash".into(),
        output: ProcessOutput {
            text: text.into(),
            ..ProcessOutput::default()
        },
        request: Vec::new(),
        cost_usd: Some(0.01),
        duration_ms: 10,
        created_at: Timestamp(5),
    }
}

fn piece_of(job: &Job, source: usize, index: u32, key: &str) -> PieceState {
    PieceState {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[source].id,
        index,
        piece: PieceRef::page(index + 1),
        status: PieceStatus::Done,
        cache_key: Some(key.into()),
        error: None,
        attempts: 1,
        updated_at: Timestamp(6),
    }
}

fn document(project: ProjectId, title: &str, source: SourceFile) -> Document {
    Document {
        id: DocumentId::new(),
        project_id: project,
        title: title.into(),
        source,
        created_at: Timestamp(3),
        tags: vec!["Kept".into(), "Letters".into()],
        source_description: BTreeMap::from([(DescriptionField::Shelfmark, "14 MIOM 2".into())]),
    }
}

/// A project with a document read by a job that also read another project's file, a frozen
/// text read by a second job, corrections, names, choices, a dictionary and review decisions;
/// and another project whose every trace holds the secret.
pub(super) fn fixture() -> Result<Fixture, StoreError> {
    let dir = tempfile::tempdir().map_err(corrupt)?;
    let store = SqliteStore::in_memory()?;
    let here = places(dir.path(), "here");
    let letter = dir.path().join("Archives").join("lettre 1.pdf");
    let mine = file_source(&letter, b"%PDF-1.4 a letter of 1915")?;
    let theirs = file_source(
        &dir.path().join("Other").join("secret.pdf"),
        SECRET.as_bytes(),
    )?;

    let p = project("Mission letters");
    store.put_project(&p)?;
    let other = project(SECRET);
    store.put_project(&other)?;
    let letters = Collection {
        id: CollectionId::new(),
        project_id: p.id,
        name: "Correspondence".into(),
        created_at: Timestamp(2),
    };
    store.put_collection(&letters)?;

    let mut job = job();
    job.sources = vec![mine.clone(), theirs.clone()];
    job.status = JobStatus::Running;
    store.create_job(&job)?;
    let doc = document(p.id, "Letter of 1915", mine);
    store.put_document(&doc)?;
    store.add_member(letters.id, doc.id)?;
    let secret_doc = document(other.id, SECRET, theirs);
    store.put_document(&secret_doc)?;
    for (document, source) in [(doc.id, 0), (secret_doc.id, 1)] {
        store.put_link(&DocumentLink {
            document_id: document,
            job_id: job.id,
            source_id: job.sources[source].id,
            linked_at: Timestamp(4),
        })?;
    }

    store.put_result(&result("key-mine-0", "Mon cher Père,"))?;
    store.put_result(&result("key-mine-1", "le 3 mai 1915"))?;
    store.put_result(&result("key-theirs", SECRET))?;
    store.upsert_piece(&piece_of(&job, 0, 0, "key-mine-0"))?;
    let mut held = piece_of(&job, 0, 1, "key-mine-1");
    held.status = PieceStatus::Submitted;
    store.upsert_piece(&held)?;
    store.upsert_piece(&piece_of(&job, 1, 0, "key-theirs"))?;
    let correction = |source: usize, text: &str| Correction {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[source].id,
        index: 0,
        text: Some(text.into()),
        fields: None,
        reviewed: true,
        updated_at: Timestamp(7),
    };
    store.put_corrections(&[correction(0, "Mon très cher Père,"), correction(1, SECRET)])?;
    store.put_speaker_names(
        job.id,
        &SpeakerNames {
            setup_id: job.setups[0].id,
            source_id: job.sources[0].id,
            names: BTreeMap::from([("Speaker 1".into(), "Père Éric".into())]),
        },
    )?;
    store.add_charge(&Charge {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        index: 0,
        usage: Default::default(),
        cost_usd: Some(0.002),
        created_at: Timestamp(8),
    })?;
    store.set_preferred(
        doc.id,
        Task::TextExtraction,
        Some((job.id, job.setups[0].id)),
    )?;

    // The corrected text frozen, and an analysis of it.
    let text = "Mon très cher Père,";
    let sha = kalamu_core::fingerprint::sha256_hex(text.as_bytes());
    std::fs::create_dir_all(&here.snapshots).map_err(corrupt)?;
    let frozen = here.snapshots.join(format!("{sha}.txt"));
    std::fs::write(&frozen, text).map_err(corrupt)?;
    let snapshot = Snapshot {
        id: Snapshot::new_id(),
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        task: Task::TextExtraction,
        original_path: job.sources[0].path.clone(),
        sha256: sha.clone(),
        parts: Vec::new(),
        missing: Vec::new(),
        created_at: Timestamp(9),
        lines: Vec::new(),
        turns: Vec::new(),
    };
    store.put_snapshot(&snapshot)?;
    let mut derived = self::job();
    derived.task = Task::Analysis;
    derived.sources[0] = SourceFile {
        id: SourceId::new(),
        path: frozen.display().to_string(),
        file_name: "lettre 1.txt".into(),
        sha256: sha,
        size_bytes: text.len() as u64,
        kind: InputKind::Text,
        page_count: None,
        derived_from: Some(DerivedFrom {
            snapshot_id: snapshot.id.clone(),
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: job.sources[0].id,
            original_path: job.sources[0].path.clone(),
            unit: None,
            timed: false,
        }),
        ..derived.sources[0].clone()
    };
    store.create_job(&derived)?;
    store.put_link(&DocumentLink {
        document_id: doc.id,
        job_id: derived.id,
        source_id: derived.sources[0].id,
        linked_at: Timestamp(10),
    })?;
    store.put_result(&result("key-derived", "Summary: a letter."))?;
    store.upsert_piece(&piece_of(&derived, 0, 0, "key-derived"))?;

    // The dictionary and the review's decisions.
    let entry = |project: ProjectId, label: &str| Entry {
        id: EntryId::new(),
        project_id: project,
        namespace: "people".into(),
        label: label.into(),
        forms: vec![label.into()],
        reviewed: true,
        created_at: Timestamp(11),
        updated_at: Timestamp(11),
    };
    let (eric, ouedraogo) = (entry(p.id, "Père Éric"), entry(p.id, "Moussa Ouédraogo"));
    store.put_entry(&eric)?;
    store.put_entry(&ouedraogo)?;
    store.put_apart(p.id, &KeptApart::new(eric.id, ouedraogo.id))?;
    store.put_dismissed(p.id, "people", "eric|moussa")?;
    store.put_entry(&entry(other.id, SECRET))?;
    store.set_review_sample(
        p.id,
        "project",
        Some(&SampleSpec {
            seed: 4242,
            size: 5,
            drawn_at: Timestamp(12),
        }),
    )?;
    store.put_kept_aside(
        p.id,
        &KeptAside {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: job.sources[0].id,
            index: Some(0),
            revision: None,
            note: "Ask Fatou about the date".into(),
            kept_at: Timestamp(13),
        },
    )?;
    store.set_result_aside(
        p.id,
        "project",
        (derived.id, derived.setups[0].id),
        Some(Timestamp(14)),
    )?;
    Ok(Fixture {
        dir,
        store,
        project: p.id,
        letter,
        job,
        derived,
    })
}

/// Opens the fixture's file in `store`.
pub(super) fn opened(
    f: &Fixture,
    store: &SqliteStore,
    path: &Path,
    name: &str,
) -> Result<ProjectFileOpened, ProjectFileError> {
    store.open_project_file(path, &f.places(name), &|_| {})
}

#[test]
fn a_project_comes_back_whole_on_another_computer() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let plan = f.store.plan_project_file(f.project, &f.places("here"))?;
    assert_eq!((plan.documents, plan.jobs, plan.originals), (1, 2, 1));
    assert_eq!(plan.file_name, "Mission letters.kalamu");
    let path = f.saved(true)?;

    let there = SqliteStore::in_memory()?;
    let opened = opened(&f, &there, &path, "there")?;
    assert_eq!(opened.name, "Mission letters");
    assert_eq!((opened.documents, opened.jobs, opened.originals), (1, 2, 1));
    assert!(opened.with_originals);
    let p = opened.project_id;
    assert_ne!(p, f.project, "a new identity");

    let projects = there.projects()?;
    assert_eq!(projects.len(), 1, "nothing of the other project");
    assert_eq!(projects[0].description, "Letters of the mission, 1912–1960");
    assert_eq!(
        projects[0].source_defaults[&DescriptionField::Repository],
        "ANOM"
    );
    assert!(!projects[0].archived, "opened to be seen");
    let documents = there.documents(p)?;
    assert_eq!(documents.len(), 1);
    let doc = &documents[0];
    assert_eq!(doc.title, "Letter of 1915");
    assert_eq!(doc.tags, ["Kept", "Letters"]);
    assert_eq!(
        doc.source_description[&DescriptionField::Shelfmark],
        "14 MIOM 2"
    );
    assert_eq!(there.collections(p)?[0].name, "Correspondence");
    assert_eq!(
        there.members(p)?,
        vec![(there.collections(p)?[0].id, doc.id)]
    );

    // The job, kept to this project's file, its original written here and pointed at.
    let links = there.links(p)?;
    assert_eq!(links.len(), 2);
    let job = there.job(links[0].job_id)?;
    assert_ne!(job.id, f.job.id);
    assert_eq!(job.sources.len(), 1, "the other project's file left out");
    assert_eq!(job.status, JobStatus::Interrupted, "nothing runs in a file");
    let written = PathBuf::from(&job.sources[0].path);
    assert!(written.starts_with(f.places("there").originals.join(p.to_string())));
    assert_eq!(
        std::fs::read(&written).map_err(corrupt)?,
        std::fs::read(&f.letter).map_err(corrupt)?
    );
    assert_eq!(doc.source.path, job.sources[0].path);
    assert_eq!(job.sources[0].sha256, f.job.sources[0].sha256);

    let pieces = there.pieces(job.id)?;
    assert_eq!(pieces.len(), 2);
    assert_eq!(
        pieces[1].status,
        PieceStatus::Pending,
        "a batch stays with its computer"
    );
    let answer = there.cached_result(&CacheKey("key-mine-0".into()))?;
    assert_eq!(
        answer.map(|r| r.output.text).as_deref(),
        Some("Mon cher Père,")
    );
    let corrections = there.corrections(job.id)?;
    assert_eq!(corrections.len(), 1);
    assert_eq!(corrections[0].text.as_deref(), Some("Mon très cher Père,"));
    assert!(corrections[0].reviewed);
    assert_eq!(
        there.speaker_names(job.id)?[0].names["Speaker 1"],
        "Père Éric"
    );
    assert_eq!(there.charges(job.id)?.len(), 1);
    let preferred = there.preferred(p)?;
    assert_eq!(
        (preferred[0].job_id, preferred[0].document_id),
        (job.id, doc.id)
    );

    // The analysis of the frozen text reads it here.
    let derived = there.job(links[1].job_id)?;
    let from = derived.sources[0]
        .derived_from
        .clone()
        .ok_or(StoreError::NotFound)?;
    assert_eq!(
        from.job_id, job.id,
        "its tie to the job follows the job's new identity"
    );
    let snapshot = there.snapshot(&from.snapshot_id)?;
    assert_eq!(snapshot.job_id, job.id);
    assert_eq!(snapshot.original_path, job.sources[0].path);
    let frozen = PathBuf::from(&derived.sources[0].path);
    assert!(frozen.starts_with(&f.places("there").snapshots));
    assert_eq!(
        std::fs::read_to_string(&frozen).map_err(corrupt)?,
        "Mon très cher Père,"
    );
    assert!(
        there
            .cached_result(&CacheKey("key-derived".into()))?
            .is_some()
    );

    // The dictionary and the review's decisions.
    let entries = there.entries(p)?;
    assert_eq!(entries.len(), 2);
    let apart = there.apart(p)?;
    assert_eq!(apart.len(), 1);
    assert!(entries.iter().any(|e| e.id == apart[0].a));
    assert_eq!(
        there.dismissed(p)?,
        vec![("people".into(), "eric|moussa".into())]
    );
    assert_eq!(
        there.review_sample(p, "project")?.map(|s| s.seed),
        Some(4242)
    );
    let kept = there.kept_aside(p)?;
    assert_eq!(
        (kept[0].job_id, kept[0].note.as_str()),
        (job.id, "Ask Fatou about the date")
    );
    assert_eq!(
        there.results_set_aside(p, "project")?,
        vec![(derived.id, derived.setups[0].id)]
    );
    Ok(())
}

#[test]
fn nothing_of_another_project_or_of_the_keychain_is_in_the_file() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    for with_originals in [false, true] {
        let path = f.saved(with_originals)?;
        let bytes = std::fs::read(&path).map_err(corrupt)?;
        assert!(
            !bytes.windows(SECRET.len()).any(|w| w == SECRET.as_bytes()),
            "the secret is in the file"
        );
        assert!(!f.dir.path().join("Mission letters.kalamu.part").exists());
    }
    Ok(())
}

#[test]
fn the_same_file_opened_twice_comes_in_beside_itself() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.saved(false)?;
    let first = opened(&f, &f.store, &path, "here")?;
    let second = opened(&f, &f.store, &path, "here")?;
    let saved = Timestamp::now().utc_date();
    assert_eq!(first.name, format!("Mission letters ({saved})"));
    assert_eq!(second.name, format!("Mission letters ({saved}) 2"));
    assert_eq!(f.store.projects()?.len(), 4);
    // The project it came from is untouched.
    assert_eq!(f.store.job(f.job.id)?.sources.len(), 2);
    assert_eq!(f.store.job(f.derived.id)?, f.derived);
    assert_eq!(f.store.links(f.project)?.len(), 2);
    let ids = |p: ProjectId| -> Result<Vec<JobId>, StoreError> {
        Ok(f.store.links(p)?.iter().map(|l| l.job_id).collect())
    };
    let (a, b, c) = (
        ids(f.project)?,
        ids(first.project_id)?,
        ids(second.project_id)?,
    );
    assert!(a.iter().all(|j| !b.contains(j) && !c.contains(j)));
    assert!(b.iter().all(|j| !c.contains(j)));
    // Without the originals, the document keeps its path and shows as missing until found.
    let doc = &f.store.documents(first.project_id)?[0];
    assert_eq!(doc.source.path, f.letter.display().to_string());
    assert_eq!(first.originals, 0);
    Ok(())
}
