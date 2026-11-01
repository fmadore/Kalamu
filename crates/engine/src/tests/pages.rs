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
        self.engine
            .create_job(NewJob {
                name: "Registre".into(),
                task: Task::TextExtraction,
                sources: vec![source],
                setups: vec![SetupRequest {
                    provider: ProviderId::new("fake"),
                    model: "fake-model".into(),
                    preset_id: None,
                    label: None,
                    options: Default::default(),
                    merge: None,
                }],
                settings: kalamu_core::JobSettings {
                    pages_per_call: Some(together),
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
    let pieces = f.engine.pieces(job.id).unwrap_or_default();
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
