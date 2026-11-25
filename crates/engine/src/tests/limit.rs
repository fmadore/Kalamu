//! The limit on calls at the same time holds across the job's files (#269): a folder of
//! one-page photographs is read as many at once as a long PDF, and never more.

use super::*;

/// The limit the tests set.
const LIMIT: usize = 4;

/// A provider that counts its calls in flight. Each call waits until the limit has been reached
/// once (or a while has passed), so the most seen at once is what the engine allows.
struct Gate {
    now: AtomicUsize,
    most: AtomicUsize,
    calls: AtomicUsize,
}

#[async_trait]
impl Provider for Gate {
    fn id(&self) -> ProviderId {
        ProviderId::new("fake")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["application/pdf"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let now = self.now.fetch_add(1, Ordering::SeqCst) + 1;
        self.most.fetch_max(now, Ordering::SeqCst);
        let waited = tokio::time::Instant::now();
        while self.most.load(Ordering::SeqCst) < LIMIT
            && waited.elapsed() < Duration::from_millis(1500)
        {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
        self.now.fetch_sub(1, Ordering::SeqCst);
        Ok(ProcessOutput {
            text: format!("{:?}", request.input.piece),
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["fake-model".into()])
    }
}

fn gated() -> (Fixture, Arc<Gate>) {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let gate = Arc::new(Gate {
        now: AtomicUsize::new(0),
        most: AtomicUsize::new(0),
        calls: AtomicUsize::new(0),
    });
    let engine = Engine::new(
        store.clone(),
        vec![gate.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: LIMIT,
            max_attempts: 1,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    let fixture = Fixture {
        engine,
        fake: Fake::new(),
        store,
        dir,
    };
    (fixture, gate)
}

/// A PDF of `pages` pages whose bytes are its own (the `nth` file's), so no answer comes from
/// the cache.
fn distinct_pdf(f: &Fixture, name: &str, nth: usize, pages: usize) -> SourceFile {
    let path = f.dir.path().join(name);
    let sizes: Vec<(f32, f32)> = (0..pages)
        .map(|p| (500.0 + (nth * 10 + p) as f32, 842.0))
        .collect();
    std::fs::write(&path, kalamu_media::sample_pdf(&sizes, false))
        .unwrap_or_else(|_| panic!("write pdf"));
    kalamu_media::inspect(&path, None).unwrap_or_else(|_| panic!("inspect"))
}

/// Every source's text was written, so each was finished once its pieces were stored.
fn written(f: &Fixture, names: &[String]) {
    for name in names {
        let txt = f.dir.path().join(format!("{name}.txt"));
        assert!(txt.is_file(), "{} was not written", txt.display());
    }
}

#[tokio::test]
async fn one_page_files_are_read_as_many_at_once_as_the_limit() {
    let (f, gate) = gated();
    let names: Vec<String> = (1..=10).map(|n| format!("photo-{n:02}")).collect();
    let sources = names
        .iter()
        .enumerate()
        .map(|(i, n)| distinct_pdf(&f, &format!("{n}.pdf"), i, 1))
        .collect();
    let job = f.new_job(sources, &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(gate.calls.load(Ordering::SeqCst), 10);
    assert_eq!(gate.most.load(Ordering::SeqCst), LIMIT);
    written(&f, &names);
}

#[tokio::test]
async fn a_long_file_among_short_ones_never_takes_the_job_over_the_limit() {
    let (f, gate) = gated();
    let names: Vec<String> = ["a", "b", "long", "c", "d", "e"]
        .iter()
        .map(|n| (*n).to_string())
        .collect();
    let sources = names
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let pages = if n == "long" { 7 } else { 1 };
            distinct_pdf(&f, &format!("{n}.pdf"), i, pages)
        })
        .collect();
    let job = f.new_job(sources, &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(gate.calls.load(Ordering::SeqCst), 12);
    assert_eq!(gate.most.load(Ordering::SeqCst), LIMIT);
    written(&f, &names);
    // The long file's pages are written in their order, whatever order they came back in.
    let txt = std::fs::read_to_string(f.dir.path().join("long.txt")).unwrap_or_default();
    let at = |n: u32| txt.find(&format!("--- Page {n} ---"));
    assert!(
        (1..7).all(|n| at(n).is_some() && at(n) < at(n + 1)),
        "{txt}"
    );
}
