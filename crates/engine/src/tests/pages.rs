//! Several pages of a PDF read in one call: grouped up to what the model can write, split back
//! into pages at their marks, and any page the answer does not give read again on its own.

use kalamu_core::PagesPerCall;

use super::*;

impl Fixture {
    /// The fake model as the catalogue would list it, reading PDFs and taking instructions,
    /// with this output limit.
    fn listed(&self, max_output: u32) {
        let mut catalog = kalamu_catalog::Catalog::bundled();
        let mut fake = catalog
            .models
            .iter()
            .find(|m| m.id == "gemini-3.8-flash")
            .cloned()
            .unwrap_or_else(|| panic!("model"));
        fake.provider = ProviderId::new("fake");
        fake.id = "fake-model".into();
        fake.limits.max_output_tokens = Some(max_output);
        catalog.models.push(fake);
        self.engine.set_catalog(catalog);
    }

    fn grouped_job(&self, source: SourceFile, together: PagesPerCall) -> Job {
        self.sampled_job(source, together, None)
    }

    fn sampled_job(&self, source: SourceFile, together: PagesPerCall, sample: Option<u32>) -> Job {
        self.job_with(source, together, sample, None)
    }

    /// A job whose pages all go together, read with `preset`.
    fn preset_job(&self, source: SourceFile, preset: &str) -> Job {
        self.job_with(source, PagesPerCall::Whole, None, Some(preset))
    }

    fn job_with(
        &self,
        source: SourceFile,
        together: PagesPerCall,
        sample: Option<u32>,
        preset: Option<&str>,
    ) -> Job {
        self.engine
            .create_job(NewJob {
                name: "Registre".into(),
                task: Task::TextExtraction,
                sources: vec![source],
                setups: vec![SetupRequest {
                    provider: ProviderId::new("fake"),
                    model: "fake-model".into(),
                    preset_id: preset.map(str::to_string),
                    label: None,
                    options: Default::default(),
                    merge: None,
                }],
                settings: kalamu_core::JobSettings {
                    pages_per_call: Some(together),
                    sample_pages: sample,
                    ..Default::default()
                },
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt],
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// The page ranges sent together, in page order (calls run two at a time).
    fn groups(&self) -> Vec<(u32, u32)> {
        let mut groups = self
            .fake
            .groups
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        groups.sort_unstable();
        groups
    }

    fn text(&self, name: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(name)).unwrap_or_default()
    }
}

#[tokio::test]
async fn pages_are_sent_together_and_kept_page_by_page() {
    let f = fixture();
    f.listed(65_536);
    let job = f.grouped_job(f.pdf("registre.pdf", 7), PagesPerCall::Several { pages: 3 });
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.groups(), [(1, 3), (4, 6)]);
    assert_eq!(f.fake.calls(), 3, "two groups, and the last page alone");

    // Every page is its own result, in the file and in the review.
    let pieces = f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(pieces.len(), 7);
    assert!(pieces.iter().all(|p| p.status.is_finished()));
    let txt = f.text("registre.txt");
    for n in 1..=7 {
        assert!(
            txt.contains(&format!("fake-model page {n}")),
            "page {n}: {txt}"
        );
    }
    let provenance: serde_json::Value =
        serde_json::from_str(&f.text("registre.provenance.json")).unwrap_or_default();
    let pages: Vec<&str> = provenance["request"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|s| s["name"] == "pages")
        .filter_map(|s| s["value"].as_str())
        .collect();
    assert_eq!(
        pages,
        ["1-3", "4-6"],
        "provenance says which pages went together"
    );
    // Each call's usage is counted once.
    assert_eq!(provenance["usage"]["output_tokens"], 300);

    // Run again: nothing is sent. The same file page by page is another reading.
    let again = f.grouped_job(f.pdf("registre.pdf", 7), PagesPerCall::Several { pages: 3 });
    assert_eq!(f.run(again.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 3);
    let alone = f.grouped_job(f.pdf("registre.pdf", 7), PagesPerCall::One);
    assert_eq!(f.run(alone.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 10, "seven pages, one by one");
}

#[tokio::test]
async fn a_whole_document_is_cut_to_what_the_model_can_write() {
    let f = fixture();
    // 10,000 tokens of output: three fifths of it, at 1,500 a page, is four pages.
    f.listed(10_000);
    let job = f.grouped_job(f.pdf("registre.pdf", 10), PagesPerCall::Whole);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.groups(), [(1, 4), (5, 8), (9, 10)]);
}

#[tokio::test]
async fn a_page_left_unmarked_or_cut_off_is_read_again_alone() {
    let f = fixture();
    f.listed(65_536);
    f.fake.unmarked.store(2, Ordering::SeqCst);
    let job = f.grouped_job(f.pdf("registre.pdf", 3), PagesPerCall::Whole);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.groups(), [(1, 3)]);
    assert_eq!(f.fake.calls(), 2, "the group, then page 2 alone");
    assert!(f.text("registre.txt").contains("fake-model page 2"));

    // Stopped at the output limit: the last page given may be incomplete, and is read again.
    let f = fixture();
    f.listed(65_536);
    f.fake.cut_groups.store(true, Ordering::SeqCst);
    let job = f.grouped_job(f.pdf("registre.pdf", 3), PagesPerCall::Whole);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 2, "the group, then page 3 alone");
    let provenance: serde_json::Value =
        serde_json::from_str(&f.text("registre.provenance.json")).unwrap_or_default();
    assert!(
        provenance.get("truncated").is_none(),
        "no page kept is cut: {provenance}"
    );
}

#[tokio::test]
async fn a_call_that_gives_no_page_is_still_counted() {
    let f = fixture();
    f.listed(65_536);
    // Page 2 unmarked and page 1 dropped as possibly cut off: nothing is kept from the call.
    f.fake.unmarked.store(2, Ordering::SeqCst);
    f.fake.cut_groups.store(true, Ordering::SeqCst);
    let job = f.grouped_job(f.pdf("registre.pdf", 2), PagesPerCall::Whole);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 3, "the group, then each page alone");
    let provenance: serde_json::Value =
        serde_json::from_str(&f.text("registre.provenance.json")).unwrap_or_default();
    assert_eq!(provenance["refused"]["answers"], 1, "{provenance}");
    assert_eq!(provenance["refused"]["usage"]["output_tokens"], 100);
}

#[tokio::test]
async fn pages_are_grouped_as_when_the_job_was_made() {
    let f = fixture();
    f.listed(10_000);
    let job = f.grouped_job(f.pdf("registre.pdf", 10), PagesPerCall::Whole);
    assert_eq!(job.setups[0].pages_cap, Some(4));
    // The catalogue is refreshed before the job runs, with a larger output limit: the pages
    // go as they would have, and keep their keys.
    let mut catalog = f.engine.catalog().clone();
    for model in catalog.models.iter_mut().filter(|m| m.id == "fake-model") {
        model.limits.max_output_tokens = Some(65_536);
    }
    f.engine.set_catalog(catalog);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.groups(), [(1, 4), (5, 8), (9, 10)]);
}

#[tokio::test]
async fn a_sample_is_read_as_the_whole_run_reads_it() {
    let f = fixture();
    f.listed(10_000);
    let sample = f.sampled_job(f.pdf("registre.pdf", 10), PagesPerCall::Whole, Some(3));
    assert_eq!(f.run(sample.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 1);
    // The whole run finds the sample's three pages in the cache: page 4, the rest of their
    // group, goes alone, and the other groups as they always would.
    let whole = f.grouped_job(f.pdf("registre.pdf", 10), PagesPerCall::Whole);
    assert_eq!(f.run(whole.id).await, JobStatus::Completed);
    assert_eq!(f.groups(), [(1, 3), (5, 8), (9, 10)]);
    assert_eq!(f.fake.calls(), 4);
}

#[tokio::test]
async fn pages_go_one_by_one_where_grouping_does_not_apply() {
    let f = fixture();
    f.listed(65_536);
    // Economy mode sends pages as a batch, one by one.
    let mut job = f.grouped_job(f.pdf("registre.pdf", 3), PagesPerCall::Whole);
    job.settings.pace = kalamu_core::Pace::Economy;
    let setup = job.setups[0].clone();
    assert_eq!(f.engine.pages_per_call(&job, &job.sources[0], &setup), 1);
    // A model the catalogue does not list as reading PDFs with instructions.
    let f = fixture();
    let job = f.grouped_job(f.pdf("registre.pdf", 3), PagesPerCall::Whole);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert!(f.groups().is_empty());
    assert_eq!(f.fake.calls(), 3);
}

/// How many pages go together follows the preset's density hint, never its id or name: with
/// 65,536 tokens of answer, 26 ordinary pages fit, or 9 dense ones.
#[tokio::test]
async fn a_dense_preset_sends_fewer_pages_together_whatever_it_is_called() {
    let f = fixture();
    f.listed(65_536);
    let cap = |preset: &str| f.preset_job(f.pdf("journal.pdf", 30), preset).setups[0].pages_cap;
    assert_eq!(cap("text_extraction.newspaper"), Some(9));
    assert_eq!(cap("text_extraction.printed"), Some(26));

    let own = |name: &str, dense: Option<bool>| {
        f.engine
            .with_prompts(|p| {
                let mut own = p.get("text_extraction.printed")?;
                own.id = String::new();
                own.name = name.into();
                own.dense = dense;
                p.save(own)
            })
            .map(|p| p.id)
            .unwrap_or_else(|e| panic!("{e}"))
    };
    let gazettes = own("Gazettes", Some(true));
    assert_eq!(
        cap(&gazettes),
        Some(9),
        "the researcher's own, marked dense"
    );
    let named = own("Newspaper pages", None);
    assert_eq!(cap(&named), Some(26), "a name says nothing");
    let copy = f
        .engine
        .with_prompts(|p| p.duplicate("text_extraction.newspaper", "Le Réveil"))
        .map(|p| p.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(cap(&copy), Some(9), "a copy of the newspaper default");

    // A job made before the cap was settled works it out again from the preset it names.
    for (preset, pages) in [(gazettes.as_str(), 9), ("text_extraction.printed", 26)] {
        let job = f.preset_job(f.pdf("journal.pdf", 30), preset);
        let mut setup = job.setups[0].clone();
        setup.pages_cap = None;
        assert_eq!(
            f.engine.pages_per_call(&job, &job.sources[0], &setup),
            pages
        );
    }
}
