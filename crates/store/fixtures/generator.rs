// The fixture generator, appended by `make.sh` to an old release's own store tests: to
// `src/tests/projects.rs` from 1.1.0, which has projects, and to `src/tests.rs` before, with the
// lines between the PROJECTS markers left out. It uses that release's own test helpers (`job`,
// `piece`, `project`, `collection`, `document`), so the records are exactly as it wrote them.
//
// Not compiled here: `src/` does not include it.

#[test]
#[ignore = "writes a fixture for the current store's migration tests"]
fn write_fixture() -> Result<(), StoreError> {
    let Ok(out) = std::env::var("KALAMU_FIXTURE_OUT") else {
        return Ok(());
    };
    let out = std::path::PathBuf::from(out);
    let _ = std::fs::remove_file(&out);
    {
        let store = SqliteStore::open(&out)?;
        let job = job();
        store.create_job(&job)?;
        for index in 0..2u32 {
            store.put_result(&kalamu_core::StoredResult {
                cache_key: format!("key-{index}"),
                provider: kalamu_core::ProviderId::new("gemini"),
                model: "gemini-3.8-flash".into(),
                output: kalamu_core::ProcessOutput {
                    text: format!("Page {}", index + 1),
                    ..Default::default()
                },
                request: Vec::new(),
                cost_usd: Some(0.01),
                duration_ms: 10,
                created_at: kalamu_core::Timestamp(1_790_000_000_000),
            })?;
            store.upsert_piece(&piece(&job, index, kalamu_core::PieceStatus::Done))?;
        }
        store.put_correction(&kalamu_core::Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: job.sources[0].id,
            index: 0,
            text: Some("Corrected page 1".into()),
            fields: None,
            reviewed: true,
            updated_at: kalamu_core::Timestamp(1_790_000_000_500),
        })?;
        store.set_job_status(job.id, kalamu_core::JobStatus::Completed)?;
        // PROJECTS-BEGIN
        let p = project("Mission archives");
        store.put_project(&p)?;
        let letters = collection(&p, "Letters");
        store.put_collection(&letters)?;
        let doc = document(&p, &job);
        store.put_document(&doc)?;
        store.add_member(letters.id, doc.id)?;
        store.put_link(&kalamu_core::DocumentLink {
            document_id: doc.id,
            job_id: job.id,
            source_id: job.sources[0].id,
            linked_at: kalamu_core::Timestamp(4),
        })?;
        store.set_preferred(
            doc.id,
            kalamu_core::Task::TextExtraction,
            Some((job.id, job.setups[0].id)),
        )?;
        // PROJECTS-END
    }
    // Closed, so the write-ahead log is folded in and the fixture is one file.
    assert!(!out.with_extension("sqlite-wal").exists());
    Ok(())
}
