//! Files written later: a correction marks its source's files out of date, and they are written
//! after a quiet period, before they are listed or exported, when the app quits and at the next
//! launch.

use std::path::Path;
use std::time::Instant;

use super::*;

/// An engine over a database file in `dir`, as the app opens it, writing files `delay` after
/// the last save.
fn engine_on(store: Arc<SqliteStore>, fake: Arc<Fake>, dir: &Path, delay: Duration) -> Arc<Engine> {
    Engine::new(
        store,
        vec![fake],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.join("listening"),
            snapshots: dir.join("snapshots"),
            write_delay: delay,
            ..EngineConfig::default()
        },
    )
}

fn open(dir: &Path) -> Arc<SqliteStore> {
    Arc::new(SqliteStore::open(&dir.join("kalamu.db")).unwrap_or_else(|e| panic!("store: {e}")))
}

fn on_disk(delay: Duration) -> Fixture {
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let store = open(dir.path());
    let fake = Fake::new();
    let engine = engine_on(store.clone(), fake.clone(), dir.path(), delay);
    Fixture {
        engine,
        fake,
        store,
        dir,
    }
}

impl Fixture {
    /// A finished one-source job of three pages, its files written.
    async fn finished(&self) -> Job {
        let source = self.pdf("bulletin.pdf", 3);
        let job = self.new_job(vec![source], &["fake-model"]);
        assert_eq!(self.run(job.id).await, JobStatus::Completed);
        assert!(self.txt().contains("fake-model page 1"));
        job
    }

    fn correct(&self, job: &Job, index: u32, text: &str) {
        self.engine
            .save_correction(Correction {
                job_id: job.id,
                setup_id: job.setups[0].id,
                source_id: job.sources[0].id,
                index,
                text: Some(text.into()),
                fields: None,
                reviewed: true,
                updated_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
    }

    fn txt(&self) -> String {
        std::fs::read_to_string(self.dir.path().join("bulletin.txt")).unwrap_or_default()
    }

    fn marks(&self) -> usize {
        self.store.outdated_files(None).map_or(0, |m| m.len())
    }

    /// Waits for every mark to be cleared, at most five seconds.
    async fn until_written(&self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.marks() > 0 {
            assert!(Instant::now() < deadline, "files never written");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
}

#[tokio::test]
async fn a_correction_is_written_after_a_quiet_period() {
    let delay = Duration::from_millis(300);
    let f = on_disk(delay);
    let job = f.finished().await;
    let saved = Instant::now();
    f.correct(&job, 0, "Le Bulletin, corrigé");
    assert!(!f.txt().contains("corrigé"), "not written on the spot");
    assert_eq!(f.marks(), 1);

    f.until_written().await;
    assert!(
        saved.elapsed() >= delay,
        "written before the quiet period ended"
    );
    assert!(f.txt().contains("Le Bulletin, corrigé"));
}

#[tokio::test]
async fn several_quick_saves_are_written_once() {
    let f = on_disk(Duration::from_millis(400));
    let job = f.finished().await;
    for i in 0..5 {
        f.correct(&job, i % 3, &format!("version {i}"));
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    assert!(!f.txt().contains("version"));
    f.until_written().await;
    let txt = f.txt();
    assert!(
        txt.contains("version 4") && txt.contains("version 3"),
        "{txt}"
    );
    assert_eq!(f.engine.later.rounds.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn listing_or_exporting_writes_what_is_out_of_date_first() {
    let f = on_disk(Duration::from_secs(60));
    let job = f.finished().await;
    let (setup, source) = (job.setups[0].id, job.sources[0].id);

    f.correct(&job, 0, "exported");
    let written = f
        .engine
        .export(job.id, setup, source)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(written.iter().any(|p| p.ends_with("bulletin.txt")));
    assert!(f.txt().contains("exported"));
    assert_eq!(f.marks(), 0);

    f.correct(&job, 1, "listed");
    let listed = f.engine.outputs(job.id).unwrap_or_else(|e| panic!("{e}"));
    assert!(!listed.is_empty());
    assert!(f.txt().contains("listed"));
    assert_eq!(f.marks(), 0);

    f.correct(&job, 2, "summarised");
    let texts = f
        .engine
        .text_outputs(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(texts.len(), 1);
    assert!(f.txt().contains("summarised"));

    // Quitting writes what is left and leaves nothing for later.
    f.correct(&job, 0, "at quit");
    assert_eq!(
        f.engine.finish_writing(Duration::from_secs(10)).ok(),
        Some(0)
    );
    assert!(f.txt().contains("at quit"));
    f.correct(&job, 0, "after quit");
    assert_eq!(f.marks(), 1, "kept for the next launch");
    assert_eq!(f.engine.later.rounds.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn files_left_out_of_date_by_a_crash_are_written_at_the_next_launch() {
    let f = on_disk(Duration::from_secs(60));
    let job = f.finished().await;
    f.correct(&job, 1, "before the crash");
    let Fixture {
        engine,
        fake,
        store,
        dir,
    } = f;
    drop(engine);
    drop(store);

    let store = open(dir.path());
    let engine = engine_on(store.clone(), fake, dir.path(), Duration::from_secs(60));
    assert_eq!(store.outdated_files(None).map(|m| m.len()).ok(), Some(1));
    assert_eq!(engine.write_outdated().ok(), Some(0));
    let txt = std::fs::read_to_string(dir.path().join("bulletin.txt")).unwrap_or_default();
    assert!(txt.contains("before the crash"), "{txt}");
    assert_eq!(store.outdated_files(None).map(|m| m.len()).ok(), Some(0));
}

#[tokio::test]
async fn a_write_that_fails_keeps_its_mark_until_one_succeeds() {
    let f = on_disk(Duration::from_millis(50));
    let job = f.finished().await;
    // A folder where the text file was: it cannot be replaced.
    let txt = f.dir.path().join("bulletin.txt");
    std::fs::remove_file(&txt).unwrap_or_else(|e| panic!("{e}"));
    std::fs::create_dir(&txt).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(txt.join("inside"), b"x").unwrap_or_else(|e| panic!("{e}"));
    let mut events = f.engine.subscribe();

    f.correct(&job, 0, "kept");
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(f.marks(), 1, "kept after the failure");
    assert_eq!(
        f.engine.later.rounds.load(Ordering::SeqCst),
        1,
        "tried once, not again and again"
    );
    let mut failed = false;
    while let Ok(event) = events.try_recv() {
        failed |= matches!(event, ProgressEvent::SourceFailed { job_id, .. } if job_id == job.id);
    }
    assert!(failed, "the failure is reported");
    assert_eq!(f.engine.write_outdated().ok(), Some(1));
    assert_eq!(f.marks(), 1);

    std::fs::remove_dir_all(&txt).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.engine.write_outdated().ok(), Some(0));
    assert_eq!(f.marks(), 0);
    assert!(f.txt().contains("kept"));
}

#[tokio::test]
async fn deleting_a_job_deletes_its_marks() {
    let f = on_disk(Duration::from_secs(60));
    let job = f.finished().await;
    f.correct(&job, 0, "deleted");
    assert_eq!(f.marks(), 1);
    f.engine
        .delete_job(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.marks(), 0);
    assert_eq!(f.engine.write_outdated().ok(), Some(0));
}
