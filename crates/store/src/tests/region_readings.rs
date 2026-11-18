//! Regions read again from the review: kept before and after their call, their cost counted in
//! what the job spent, their answers kept from pruning, cancelled by a restart, accepted and
//! undone with their effect (the page's correction, or the researcher's region) in one
//! transaction, and deleted with their job.

use kalamu_core::{
    Correction, DrawnRegion, PromptSnapshot, ProposalStatus, ReadingEffect, ReadingTarget,
    RegionAsk, RegionReading, StoredResult,
};

use super::*;

fn reading(job: &Job, id: &str, status: ProposalStatus, cost: Option<f64>) -> RegionReading {
    RegionReading {
        ask: RegionAsk {
            id: id.into(),
            job_id: job.id,
            source_id: job.sources[0].id,
            page: 1,
            half: None,
            x: 0.1,
            y: 0.2,
            width: 0.3,
            height: 0.1,
            region_id: "d1".into(),
            target: ReadingTarget::Region,
            provider: ProviderId::new("gemini"),
            model: "gemini-3.8-flash".into(),
            preset_id: None,
        },
        status,
        prompt: PromptSnapshot {
            preset_id: Some("text_extraction.printed".into()),
            preset_name: "Printed document".into(),
            template: "Transcribe.".into(),
            text: "Transcribe.".into(),
            user_prompt: "Transcribe this page.".into(),
            schema: None,
        },
        request: Vec::new(),
        cache_key: format!("region-{id}"),
        answer: None,
        cost_usd: cost,
        cached: false,
        error: None,
        accepted: None,
        asked_at: Timestamp::now(),
        answered_at: None,
        decided_at: None,
    }
}

fn region(job: &Job, text: &str) -> DrawnRegion {
    DrawnRegion {
        id: "d1".into(),
        job_id: job.id,
        source_id: job.sources[0].id,
        page: 1,
        half: None,
        x: 0.1,
        y: 0.2,
        width: 0.3,
        height: 0.1,
        text: text.into(),
        adjusts: None,
        source_hash: "abc".into(),
        created_at: Timestamp(1),
        updated_at: Timestamp(1),
    }
}

#[test]
fn a_reading_is_kept_before_and_after_its_call() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let mut asked = reading(&job, "g1", ProposalStatus::Asked, None);
    store.put_region_reading(&asked, None)?;
    assert_eq!(store.region_reading("g1")?.status, ProposalStatus::Asked);
    asked.status = ProposalStatus::Answered;
    asked.answer = Some("note en marge".into());
    asked.cost_usd = Some(0.002);
    store.put_region_reading(&asked, None)?;
    assert_eq!(store.region_reading("g1")?, asked);
    let listed = store.source_region_readings(job.id, job.sources[0].id)?;
    assert_eq!(listed, vec![asked]);
    assert!(
        store
            .source_region_readings(job.id, SourceId::new())?
            .is_empty()
    );
    assert!(matches!(
        store.region_reading("other"),
        Err(StoreError::NotFound)
    ));
    Ok(())
}

#[test]
fn a_reading_s_cost_counts_in_what_its_job_spent() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.put_region_reading(
        &reading(&job, "g1", ProposalStatus::Answered, Some(0.002)),
        None,
    )?;
    // An answer from the cache cost nothing.
    store.put_region_reading(&reading(&job, "g2", ProposalStatus::Answered, None), None)?;
    assert!((store.spent(job.id)? - 0.002).abs() < 1e-12);
    let listed = store.list_jobs()?;
    assert!(
        listed[0]
            .cost_usd
            .is_some_and(|c| (c - 0.002).abs() < 1e-12)
    );
    Ok(())
}

#[test]
fn a_restart_cancels_a_reading_still_waiting() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.put_region_reading(&reading(&job, "g1", ProposalStatus::Asked, None), None)?;
    store.put_region_reading(&reading(&job, "g2", ProposalStatus::Answered, None), None)?;
    store.mark_running_as_interrupted()?;
    assert_eq!(
        store.region_reading("g1")?.status,
        ProposalStatus::Cancelled
    );
    assert_eq!(store.region_reading("g2")?.status, ProposalStatus::Answered);
    Ok(())
}

#[test]
fn an_accepted_reading_is_kept_with_the_page_s_correction() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let mut accepted = reading(&job, "g1", ProposalStatus::Accepted, None);
    accepted.accepted = Some("Gouverneur".into());
    let correction = Correction {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        index: 0,
        text: Some("Le Gouverneur a reçu".into()),
        fields: None,
        reviewed: false,
        updated_at: Timestamp::now(),
    };
    store.put_region_reading(
        &accepted,
        Some(&ReadingEffect::Correction(correction.clone())),
    )?;
    assert_eq!(store.corrections(job.id)?, vec![correction]);
    assert_eq!(store.region_reading("g1")?.status, ProposalStatus::Accepted);
    // The source's files are out of date, as after any correction.
    assert_eq!(store.outdated_files(Some(job.id))?.len(), 1);
    Ok(())
}

#[test]
fn an_accepted_reading_and_its_undoing_write_the_region_s_text_with_it() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let drawn = region(&job, "");
    store.put_region(&drawn)?;
    let mut accepted = reading(&job, "g1", ProposalStatus::Accepted, None);
    accepted.accepted = Some("note en marge".into());
    let written = region(&job, "note en marge");
    store.put_region_reading(&accepted, Some(&ReadingEffect::Region(written.clone())))?;
    assert_eq!(store.region("d1")?, written);
    // Undone: the region as it was, and the reading said undone, together.
    accepted.status = ProposalStatus::Undone;
    store.put_region_reading(&accepted, Some(&ReadingEffect::Region(drawn.clone())))?;
    assert_eq!(store.region("d1")?, drawn);
    assert_eq!(store.region_reading("g1")?.status, ProposalStatus::Undone);
    Ok(())
}

#[test]
fn a_failed_effect_leaves_the_reading_as_it_was() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let answered = reading(&job, "g1", ProposalStatus::Answered, None);
    store.put_region_reading(&answered, None)?;
    // A region of a job that does not exist breaks the transaction: nothing of it is kept.
    let mut accepted = answered.clone();
    accepted.status = ProposalStatus::Accepted;
    let mut stray = region(&job, "x");
    stray.job_id = JobId::new();
    assert!(
        store
            .put_region_reading(&accepted, Some(&ReadingEffect::Region(stray)))
            .is_err()
    );
    assert_eq!(store.region_reading("g1")?, answered);
    assert!(matches!(store.region("d1"), Err(StoreError::NotFound)));
    Ok(())
}

#[test]
fn a_reading_s_answer_is_kept_from_pruning_and_goes_with_its_job() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.put_result(&StoredResult {
        cache_key: "region-g1".into(),
        provider: ProviderId::new("gemini"),
        model: "gemini-3.8-flash".into(),
        output: ProcessOutput::default(),
        request: Vec::new(),
        cost_usd: Some(0.002),
        duration_ms: 10,
        created_at: Timestamp::now(),
    })?;
    store.put_region_reading(
        &reading(&job, "g1", ProposalStatus::Answered, Some(0.002)),
        None,
    )?;
    assert_eq!(store.prune_results()?.results, 0);
    assert!(
        store
            .cached_result(&CacheKey("region-g1".into()))?
            .is_some()
    );
    store.delete_job(job.id)?;
    assert!(matches!(
        store.region_reading("g1"),
        Err(StoreError::NotFound)
    ));
    assert_eq!(store.prune_results()?.results, 1);
    Ok(())
}
