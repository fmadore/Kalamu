//! Passages corrected from the review: kept before and after their call, their cost counted in
//! what the job spent, their answers kept from pruning, cancelled by a restart, accepted with
//! their correction in one transaction, and deleted with their job.

use kalamu_core::{
    Correction, PassageAsk, PassageProposal, PromptSnapshot, ProposalStatus, StoredResult,
};

use super::*;

fn proposal(job: &Job, id: &str, status: ProposalStatus, cost: Option<f64>) -> PassageProposal {
    PassageProposal {
        ask: PassageAsk {
            id: id.into(),
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: job.sources[0].id,
            index: 0,
            before: "Le ".into(),
            passage: "Gouvemeur".into(),
            after: " a reçu".into(),
            at: 3,
            provider: ProviderId::new("gemini"),
            model: "gemini-3.8-flash".into(),
            preset_id: None,
        },
        status,
        prompt: PromptSnapshot {
            preset_id: Some("text_correction.general".into()),
            preset_name: "Printed document or newspaper".into(),
            template: "Correct.".into(),
            text: "Correct.".into(),
            user_prompt: "Correct the passage.".into(),
            schema: None,
        },
        request: Vec::new(),
        cache_key: format!("passage-{id}"),
        answer: None,
        notes: Vec::new(),
        cost_usd: cost,
        cached: false,
        error: None,
        accepted: None,
        asked_at: Timestamp::now(),
        answered_at: None,
        decided_at: None,
    }
}

#[test]
fn a_passage_is_kept_before_and_after_its_call() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let mut asked = proposal(&job, "p1", ProposalStatus::Asked, None);
    store.put_passage(&asked, None)?;
    assert_eq!(store.passage("p1")?.status, ProposalStatus::Asked);
    asked.status = ProposalStatus::Answered;
    asked.answer = Some("Gouverneur".into());
    asked.cost_usd = Some(0.002);
    store.put_passage(&asked, None)?;
    assert_eq!(store.passage("p1")?, asked);
    let listed = store.source_passages(job.id, job.setups[0].id, job.sources[0].id)?;
    assert_eq!(listed, vec![asked]);
    assert!(matches!(store.passage("other"), Err(StoreError::NotFound)));
    Ok(())
}

#[test]
fn a_passage_s_cost_counts_in_what_its_job_spent() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.put_passage(
        &proposal(&job, "p1", ProposalStatus::Answered, Some(0.002)),
        None,
    )?;
    // An answer from the cache cost nothing.
    store.put_passage(&proposal(&job, "p2", ProposalStatus::Answered, None), None)?;
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
fn a_restart_cancels_a_passage_still_waiting() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.put_passage(&proposal(&job, "p1", ProposalStatus::Asked, None), None)?;
    store.put_passage(&proposal(&job, "p2", ProposalStatus::Answered, None), None)?;
    store.mark_running_as_interrupted()?;
    assert_eq!(store.passage("p1")?.status, ProposalStatus::Cancelled);
    assert_eq!(store.passage("p2")?.status, ProposalStatus::Answered);
    Ok(())
}

#[test]
fn an_accepted_passage_is_kept_with_its_correction() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let mut accepted = proposal(&job, "p1", ProposalStatus::Accepted, None);
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
    store.put_passage(&accepted, Some(&correction))?;
    assert_eq!(store.corrections(job.id)?, vec![correction]);
    assert_eq!(store.passage("p1")?.status, ProposalStatus::Accepted);
    // The source's files are out of date, as after any correction.
    assert_eq!(store.outdated_files(Some(job.id))?.len(), 1);
    Ok(())
}

#[test]
fn a_passage_s_answer_is_kept_from_pruning_and_goes_with_its_job() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.put_result(&StoredResult {
        cache_key: "passage-p1".into(),
        provider: ProviderId::new("gemini"),
        model: "gemini-3.8-flash".into(),
        output: ProcessOutput::default(),
        request: Vec::new(),
        cost_usd: Some(0.002),
        duration_ms: 10,
        created_at: Timestamp::now(),
    })?;
    store.put_passage(
        &proposal(&job, "p1", ProposalStatus::Answered, Some(0.002)),
        None,
    )?;
    assert_eq!(store.prune_results()?.results, 0);
    assert!(
        store
            .cached_result(&CacheKey("passage-p1".into()))?
            .is_some()
    );
    store.delete_job(job.id)?;
    assert!(matches!(store.passage("p1"), Err(StoreError::NotFound)));
    assert_eq!(store.prune_results()?.results, 1);
    Ok(())
}
