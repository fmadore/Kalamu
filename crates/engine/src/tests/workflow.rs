//! Workflow runs: extraction then analysis, each step a job; a review or failed pieces holding
//! the next step until the researcher goes on; unchanged steps read again from the cache; at
//! launch nothing is sent without the researcher; definitions that cannot run refused.

use kalamu_core::{Correction, RunId, RunStatus, StepInput, StepState, WorkflowError};
use kalamu_prompts::{Pipeline, PipelineSetup, Workflow, WorkflowStep};

use super::*;

fn pipeline(name: &str, task: Task, model: &str, preset: Option<&str>) -> Pipeline {
    Pipeline {
        id: String::new(),
        name: name.into(),
        task,
        setups: vec![PipelineSetup {
            provider: ProviderId::new("fake"),
            model: model.into(),
            preset_id: preset.map(Into::into),
            label: None,
            options: Default::default(),
            merge: None,
        }],
        settings: Default::default(),
        formats: vec![ExportFormat::Txt, ExportFormat::Json],
        destination: OutputDestination::BesideSources,
        updated_at: Timestamp(0),
    }
}

/// Extract the text, then list the people and places in it.
fn extract_then_analyse(review: bool, analyst: &str) -> Workflow {
    Workflow {
        id: "workflow-1".into(),
        name: "Registers".into(),
        version: 2,
        steps: vec![
            WorkflowStep {
                pipeline: pipeline("Extract", Task::TextExtraction, "fake-model", None),
                input: StepInput::Sources,
                review,
            },
            WorkflowStep {
                pipeline: pipeline(
                    "People and places",
                    Task::Analysis,
                    analyst,
                    Some("analysis.entities"),
                ),
                input: StepInput::Step { step: 0 },
                review: false,
            },
        ],
        updated_at: Timestamp(0),
    }
}

impl Fixture {
    /// Starts a run and waits until it no longer runs on its own: finished, or waiting for the
    /// researcher.
    async fn run_workflow(&self, workflow: Workflow, sources: Vec<SourceFile>) -> RunView {
        let events = self.engine.subscribe();
        let run = self
            .engine
            .start_run(StartRun {
                workflow,
                name: "Registre".into(),
                sources,
                project: None,
                collection: None,
            })
            .unwrap_or_else(|e| panic!("start run: {e}"));
        self.settle(run.id, events).await
    }

    async fn settle(
        &self,
        id: RunId,
        mut events: tokio::sync::broadcast::Receiver<ProgressEvent>,
    ) -> RunView {
        loop {
            let view = self.engine.run_view(id).unwrap_or_else(|e| panic!("{e}"));
            if view.status != RunStatus::Running {
                return view;
            }
            match tokio::time::timeout(Duration::from_secs(10), events.recv()).await {
                Ok(Ok(_)) => {}
                other => panic!("the run did not settle: {other:?} {view:?}"),
            }
        }
    }

    fn provenance(&self, name: &str) -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(self.dir.path().join(name)).unwrap_or_default())
            .unwrap_or_default()
    }
}

fn job_of(view: &RunView, step: usize) -> JobId {
    view.steps[step]
        .job
        .as_ref()
        .map(|j| j.id)
        .unwrap_or_else(|| panic!("step {step} has no job: {view:?}"))
}

fn states(view: &RunView) -> Vec<StepState> {
    view.steps.iter().map(|s| s.state).collect()
}

#[tokio::test]
async fn a_workflow_extracts_then_analyses_the_text_it_extracted() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 2);
    let view = f
        .run_workflow(
            extract_then_analyse(false, "fake-model"),
            vec![scan.clone()],
        )
        .await;
    assert_eq!(view.status, RunStatus::Finished, "{view:?}");
    assert_eq!(states(&view), [StepState::Done, StepState::Done]);
    assert_eq!(f.fake.calls(), 3, "two pages, then one analysis");

    let extraction = f
        .engine
        .job(job_of(&view, 0))
        .unwrap_or_else(|e| panic!("{e}"));
    let analysis = f
        .engine
        .job(job_of(&view, 1))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(extraction.name, "Registre · Extract");
    assert_eq!(extraction.sources[0].id, scan.id);
    let from = analysis.sources[0]
        .derived_from
        .clone()
        .unwrap_or_else(|| panic!("the analysis reads a snapshot"));
    assert_eq!((from.job_id, from.source_id), (extraction.id, scan.id));
    let read = f
        .fake
        .read
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    assert!(read[0].contains("fake-model page 2"), "{read:?}");

    // Each step writes its files, and the analysis's provenance places it in the run.
    assert!(f.dir.path().join("registre.txt").is_file());
    let provenance = f.provenance("registre.analysis.provenance.json");
    let workflow = &provenance["workflow"];
    assert_eq!(workflow["run_id"], view.run.id.to_string());
    assert_eq!(workflow["name"], "Registers");
    assert_eq!(
        (workflow["version"].as_u64(), workflow["step"].as_u64()),
        (Some(2), Some(2))
    );
    assert_eq!(workflow["reads"], 1);
    assert!(workflow.get("approval").is_none(), "nothing was asked");
    assert_eq!(
        provenance["source"]["derived_from"]["job_id"],
        extraction.id.to_string()
    );
    let first = f.provenance("registre.provenance.json");
    assert_eq!(first["workflow"]["step"], 1);
    assert!(first["workflow"].get("reads").is_none());
}

#[tokio::test]
async fn a_review_holds_the_next_step_which_then_reads_the_corrections() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 2);
    let view = f
        .run_workflow(extract_then_analyse(true, "fake-model"), vec![scan])
        .await;
    assert_eq!(view.status, RunStatus::NeedsYou);
    assert_eq!(states(&view), [StepState::Review, StepState::Waiting]);
    assert!(
        view.steps[1].job.is_none(),
        "nothing starts before the review"
    );
    assert!(matches!(
        f.engine.approve_step(view.run.id, 1),
        Err(EngineError::StepNotFinished)
    ));

    let extraction = f
        .engine
        .job(job_of(&view, 0))
        .unwrap_or_else(|e| panic!("{e}"));
    // The job's page knows it waits for its review.
    let step = f
        .engine
        .job_step(extraction.id)
        .unwrap_or_else(|e| panic!("{e}"))
        .unwrap_or_else(|| panic!("a step"));
    assert_eq!(
        (step.step, step.view.steps[0].state),
        (0, StepState::Review)
    );
    f.engine
        .save_correction(Correction {
            job_id: extraction.id,
            setup_id: extraction.setups[0].id,
            source_id: extraction.sources[0].id,
            index: 0,
            text: Some("Registre des baptêmes, 1755.".into()),
            fields: None,
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let events = f.engine.subscribe();
    f.engine
        .approve_step(view.run.id, 0)
        .unwrap_or_else(|e| panic!("{e}"));
    let view = f.settle(view.run.id, events).await;
    assert_eq!(states(&view), [StepState::Done, StepState::Done]);
    let read = f
        .fake
        .read
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    assert!(read[0].contains("Registre des baptêmes, 1755."), "{read:?}");

    let approval = &f.provenance("registre.analysis.provenance.json")["workflow"]["approval"];
    assert_eq!(approval["partial"], false);
    assert!(approval["at"].as_i64().is_some_and(|at| at > 0));
}

#[tokio::test]
async fn pages_not_read_hold_the_next_step_until_the_researcher_goes_on() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 3);
    f.fake
        .fail(2, vec![ProviderError::Blocked("refused".into())]);
    let view = f
        .run_workflow(extract_then_analyse(false, "fake-model"), vec![scan])
        .await;
    assert_eq!(states(&view), [StepState::Attention, StepState::Waiting]);
    let events = f.engine.subscribe();
    f.engine
        .approve_step(view.run.id, 0)
        .unwrap_or_else(|e| panic!("{e}"));
    let view = f.settle(view.run.id, events).await;
    assert_eq!(view.status, RunStatus::Finished);
    assert_eq!(states(&view), [StepState::DoneWithErrors, StepState::Done]);
    let provenance = f.provenance("registre.analysis.provenance.json");
    assert_eq!(provenance["workflow"]["approval"]["partial"], true);
    assert_eq!(
        provenance["source"]["derived_from"]["missing"],
        serde_json::json!([2]),
        "the page not read is named"
    );
}

#[tokio::test]
async fn a_step_that_could_not_start_starts_once_its_input_is_read() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 1);
    f.fake
        .fail(1, vec![ProviderError::Blocked("refused".into())]);
    let view = f
        .run_workflow(extract_then_analyse(false, "fake-model"), vec![scan])
        .await;
    // Going on with no page read: the next step has no text, and says so.
    f.engine
        .approve_step(view.run.id, 0)
        .unwrap_or_else(|e| panic!("{e}"));
    let view = f
        .engine
        .run_view(view.run.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(view.steps[1].job.is_none());
    assert!(view.run.steps[1].progress.error.is_some());

    // The page read on a retry: the step starts after all.
    let events = f.engine.subscribe();
    f.engine
        .retry_failed(job_of(&view, 0))
        .unwrap_or_else(|e| panic!("{e}"));
    let view = f.settle(view.run.id, events).await;
    assert_eq!(view.status, RunStatus::Finished, "{view:?}");
    assert!(view.run.steps[1].progress.error.is_none());
}

#[tokio::test]
async fn a_step_job_is_recorded_with_its_step_or_not_at_all() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 1);
    let view = f
        .run_workflow(extract_then_analyse(true, "fake-model"), vec![scan.clone()])
        .await;
    let job = crate::setup::build_job(
        "Orphan".into(),
        Task::Analysis,
        vec![scan],
        Vec::new(),
        Default::default(),
        Default::default(),
    );
    let progress = kalamu_core::StepProgress {
        job_id: Some(job.id),
        ..Default::default()
    };
    // A step the run does not have: neither the job nor the progress is kept.
    assert!(
        f.store
            .create_step_job(&job, view.run.id, 9, &progress)
            .is_err()
    );
    assert!(f.store.job(job.id).is_err());
}

#[tokio::test]
async fn running_again_pays_only_for_the_steps_that_changed() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 2);
    f.run_workflow(
        extract_then_analyse(false, "fake-model"),
        vec![scan.clone()],
    )
    .await;
    assert_eq!(f.fake.calls(), 3);

    let again = f
        .run_workflow(
            extract_then_analyse(false, "fake-model"),
            vec![scan.clone()],
        )
        .await;
    assert_eq!(again.status, RunStatus::Finished);
    assert_eq!(
        f.fake.calls(),
        3,
        "the same run again is read from the cache"
    );

    let other = f
        .run_workflow(extract_then_analyse(false, "other-model"), vec![scan])
        .await;
    assert_eq!(other.status, RunStatus::Finished);
    assert_eq!(f.fake.calls(), 4, "only the analysis, whose model changed");
}

#[tokio::test]
async fn at_launch_a_step_made_ready_is_not_started_without_the_researcher() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 2);
    let view = f
        .run_workflow(extract_then_analyse(true, "fake-model"), vec![scan])
        .await;
    // The approval was recorded just as the app closed, before the next step was made.
    let mut progress = view.run.steps[0].progress.clone();
    progress.approval = Some(kalamu_core::Approval {
        at: Timestamp::now(),
        partial: false,
    });
    f.store
        .set_step_progress(view.run.id, 0, &progress)
        .unwrap_or_else(|e| panic!("{e}"));

    f.engine.resume_runs().unwrap_or_else(|e| panic!("{e}"));
    let view = f
        .engine
        .run_view(view.run.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(states(&view), [StepState::Done, StepState::Paused]);
    assert_eq!(view.status, RunStatus::NeedsYou);
    let analysis = job_of(&view, 1);
    assert_eq!(f.fake.calls(), 2, "nothing sent at launch");
    f.engine.resume_runs().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        f.engine.run_view(view.run.id).map(|v| job_of(&v, 1)).ok(),
        Some(analysis),
        "a step is made once"
    );

    assert_eq!(f.run(analysis).await, JobStatus::Completed);
    let view = f
        .engine
        .run_view(view.run.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(view.status, RunStatus::Finished);
}

#[tokio::test]
async fn a_workflow_that_cannot_run_is_refused_before_anything_is_sent() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 1);
    let start = |workflow| {
        f.engine.start_run(StartRun {
            workflow,
            name: String::new(),
            sources: vec![scan.clone()],
            project: None,
            collection: None,
        })
    };
    let mut forward = extract_then_analyse(false, "fake-model");
    forward.steps[0].input = StepInput::Step { step: 1 };
    assert!(matches!(
        start(forward),
        Err(EngineError::Workflow(WorkflowError::NotEarlier { step: 1 }))
    ));
    let mut unknown = extract_then_analyse(false, "fake-model");
    unknown.steps[1].pipeline.setups[0].provider = ProviderId::new("nowhere");
    let refused = start(unknown);
    assert!(
        matches!(&refused, Err(EngineError::Step { step: 2, source }) if matches!(**source, EngineError::UnknownProvider(_))),
        "{refused:?}"
    );
    // A transcription does not read scans.
    let mut audio = extract_then_analyse(false, "fake-model");
    audio.steps[0].pipeline.task = Task::AudioTranscription;
    assert!(matches!(
        start(audio),
        Err(EngineError::Step { step: 1, .. })
    ));
    assert!(f.engine.run_views().unwrap_or_default().is_empty());
    assert_eq!(f.fake.calls(), 0);
}

#[tokio::test]
async fn a_cancelled_run_starts_no_further_step() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 1);
    let view = f
        .run_workflow(extract_then_analyse(true, "fake-model"), vec![scan])
        .await;
    f.engine
        .cancel_run(view.run.id)
        .unwrap_or_else(|e| panic!("{e}"));
    f.engine
        .approve_step(view.run.id, 0)
        .unwrap_or_else(|e| panic!("{e}"));
    let view = f
        .engine
        .run_view(view.run.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(view.status, RunStatus::Stopped);
    assert_eq!(view.steps[1].state, StepState::Cancelled);
    assert!(view.steps[1].job.is_none());

    // Forgetting the run keeps its jobs.
    let extraction = job_of(&view, 0);
    f.engine
        .delete_run(view.run.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(f.engine.job(extraction).is_ok());
    assert!(f.engine.run_views().unwrap_or_default().is_empty());
}

#[tokio::test]
async fn a_workflow_runs_on_the_text_of_an_earlier_job() {
    let f = fixture();
    let extraction = f.new_job(vec![f.pdf("registre.pdf", 2)], &["fake-model"]);
    assert_eq!(f.run(extraction.id).await, JobStatus::Completed);
    let texts = f
        .engine
        .snapshot(extraction.id, extraction.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let calls = f.fake.calls();

    // A workflow that reads scans first cannot start on text, and says which step.
    assert!(matches!(
        f.engine.start_run(StartRun {
            workflow: extract_then_analyse(false, "fake-model"),
            name: String::new(),
            sources: texts.clone(),
            project: None,
            collection: None,
        }),
        Err(EngineError::Step { step: 1, .. })
    ));

    let mut analyse = extract_then_analyse(false, "fake-model");
    analyse.steps.remove(0);
    analyse.steps[0].input = StepInput::Sources;
    let view = f.run_workflow(analyse, texts).await;
    assert_eq!(view.status, RunStatus::Finished);
    assert_eq!(f.fake.calls(), calls + 1, "the text is not read again");
    let analysis = f
        .engine
        .job(job_of(&view, 0))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        analysis.sources[0].derived_from.as_ref().map(|d| d.job_id),
        Some(extraction.id)
    );
}
