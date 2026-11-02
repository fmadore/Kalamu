//! Workflows: pipelines run one after another over the same documents, each step reading the
//! files or the corrected text of an earlier step (#49). A saved workflow is a definition;
//! running it freezes a copy, so saving it again (a new version) never changes a run started.
//!
//! Each step is a pipeline, so a saved pipeline becomes a step as it is, and a pipeline on its
//! own is a one-step workflow.

use std::path::Path;

use kalamu_core::workflow::check_steps;
use kalamu_core::{OutputDestination, StepInput, Task, Timestamp};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::folder::{Folder, Saved};
use crate::{Pipeline, PromptError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Workflow {
    /// Empty for a new workflow; given when it is first saved.
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// Raised by one each time it is saved; a run records the version it started from.
    #[serde(default)]
    pub version: u32,
    pub steps: Vec<WorkflowStep>,
    /// When it was last saved; the interface sends 0 and saving sets it.
    pub updated_at: Timestamp,
}

/// One step: a pipeline's settings, what it reads, and whether to stop for a review after it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkflowStep {
    /// Its task, models, prompts, languages and formats; its name names the step.
    pub pipeline: Pipeline,
    #[serde(default)]
    pub input: StepInput,
    /// Wait for the researcher to review this step's results before a later step reads them.
    #[serde(default)]
    pub review: bool,
}

impl Workflow {
    /// A pipeline as a workflow of one step, which reads the files it is run on.
    pub fn from_pipeline(pipeline: Pipeline) -> Self {
        Self {
            id: String::new(),
            name: pipeline.name.clone(),
            version: 0,
            steps: vec![WorkflowStep {
                pipeline,
                input: StepInput::Sources,
                review: false,
            }],
            updated_at: Timestamp(0),
        }
    }

    /// Each step's task and input, in order.
    pub fn shape(&self) -> Vec<(Task, StepInput)> {
        self.steps
            .iter()
            .map(|s| (s.pipeline.task, s.input))
            .collect()
    }

    /// Whether every step can run: a name, a model for each step, and inputs that read text
    /// from a step before.
    pub fn check(&self) -> Result<(), PromptError> {
        if self.name.trim().is_empty() || self.steps.iter().any(|s| s.pipeline.setups.is_empty()) {
            return Err(PromptError::IncompleteWorkflow);
        }
        check_steps(&self.shape()).map_err(PromptError::Workflow)
    }

    /// With the default that replaced each retired one its steps name.
    fn current(mut self) -> Self {
        self.steps = self
            .steps
            .into_iter()
            .map(|step| WorkflowStep {
                pipeline: step.pipeline.current(),
                ..step
            })
            .collect();
        self
    }
}

impl Saved for Workflow {
    const PREFIX: &'static str = "workflow-";
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct WorkflowLibrary(Folder<Workflow>);

impl WorkflowLibrary {
    /// Loads saved workflows from `dir`, creating it if needed; an unreadable file is skipped.
    pub fn open(dir: &Path) -> Result<Self, PromptError> {
        Folder::open(dir).map(Self)
    }

    /// Nothing saved; for tests.
    pub fn in_memory() -> Self {
        Self(Folder::in_memory())
    }

    /// By name, ignoring case.
    pub fn list(&self) -> Vec<Workflow> {
        self.0.list().into_iter().map(Workflow::current).collect()
    }

    pub fn get(&self, id: &str) -> Result<Workflow, PromptError> {
        self.0.get(id).map(Workflow::current)
    }

    /// Saves a workflow under its id, or a new one, as its next version. Saving under a name
    /// already used replaces that workflow, as a pipeline's name does.
    pub fn save(&mut self, mut workflow: Workflow) -> Result<Workflow, PromptError> {
        workflow.name = workflow.name.trim().to_string();
        workflow.check()?;
        workflow.id = self.0.id_for(&workflow)?;
        workflow.version = self.0.saved(&workflow.id).map_or(0, |w| w.version) + 1;
        workflow.updated_at = Timestamp::now();
        self.0.put(workflow)
    }

    /// A copy under a new name, as its own first version.
    pub fn duplicate(&mut self, id: &str, name: &str) -> Result<Workflow, PromptError> {
        let mut copy = self.get(id)?;
        copy.id = String::new();
        copy.name = self.free_name(name);
        self.save(copy)
    }

    pub fn delete(&mut self, id: &str) -> Result<(), PromptError> {
        self.0.delete(id)
    }

    /// `name`, or `name (2)`, `name (3)`… when it is taken: an import or a copy never
    /// replaces a workflow of the same name.
    pub(crate) fn free_name(&self, name: &str) -> String {
        self.0.free_name(name)
    }
}

/// A workflow without what belongs to one computer: each step's output folder.
pub(crate) fn shareable(mut workflow: Workflow) -> Workflow {
    for step in &mut workflow.steps {
        step.pipeline.destination = OutputDestination::default();
    }
    workflow
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod tests {
    use kalamu_core::{ExportFormat, JobSettings, ProviderId, SetupOptions, WorkflowError};

    use super::*;
    use kalamu_core::SetupRequest;

    pub(crate) fn pipeline(name: &str, task: Task) -> Pipeline {
        Pipeline {
            id: String::new(),
            name: name.into(),
            task,
            setups: vec![SetupRequest {
                provider: ProviderId::new("gemini"),
                model: "gemini-3.8-flash".into(),
                preset_id: None,
                label: None,
                options: SetupOptions::default(),
                merge: None,
            }],
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Txt],
            destination: OutputDestination::Folder(r"C:\Users\me\results".into()),
            updated_at: Timestamp(0),
        }
    }

    pub(crate) fn extract_then_analyse(name: &str) -> Workflow {
        let mut workflow = Workflow::from_pipeline(pipeline("Extract", Task::TextExtraction));
        workflow.name = name.into();
        workflow.steps[0].review = true;
        workflow.steps.push(WorkflowStep {
            pipeline: pipeline("People and places", Task::Analysis),
            input: StepInput::Step { step: 0 },
            review: false,
        });
        workflow
    }

    #[test]
    fn saving_again_makes_a_new_version_under_the_same_id() {
        let dir = tempfile::tempdir().unwrap();
        let mut library = WorkflowLibrary::open(dir.path()).unwrap();
        let first = library.save(extract_then_analyse("Letters")).unwrap();
        assert_eq!(first.version, 1);
        let mut edited = first.clone();
        edited.steps[1].review = true;
        let second = library.save(edited).unwrap();
        assert_eq!((second.id.as_str(), second.version), (first.id.as_str(), 2));
        let by_name = library.save(extract_then_analyse(" LETTERS ")).unwrap();
        assert_eq!((by_name.id, by_name.version), (first.id.clone(), 3));

        let copy = library.duplicate(&first.id, "Letters").unwrap();
        assert_ne!(copy.id, first.id);
        assert_eq!((copy.name.as_str(), copy.version), ("Letters (2)", 1));

        let reopened = WorkflowLibrary::open(dir.path()).unwrap();
        assert_eq!(reopened.list().len(), 2);
        assert_eq!(reopened.get(&first.id).unwrap().version, 3);
        library.delete(&copy.id).unwrap();
        assert_eq!(WorkflowLibrary::open(dir.path()).unwrap().list().len(), 1);
    }

    #[test]
    fn a_workflow_that_cannot_run_is_not_saved() {
        let mut library = WorkflowLibrary::in_memory();
        assert!(matches!(
            library.save(extract_then_analyse("  ")),
            Err(PromptError::IncompleteWorkflow)
        ));
        let mut forward = extract_then_analyse("Forward");
        forward.steps[0].input = StepInput::Step { step: 1 };
        assert!(matches!(
            library.save(forward),
            Err(PromptError::Workflow(WorkflowError::NotEarlier { step: 1 }))
        ));
        let mut fields = extract_then_analyse("Fields");
        fields.steps.push(WorkflowStep {
            pipeline: pipeline("Again", Task::Analysis),
            input: StepInput::Step { step: 1 },
            review: false,
        });
        assert!(matches!(
            library.save(fields),
            Err(PromptError::Workflow(WorkflowError::NotText { step: 3 }))
        ));
        let mut forged = extract_then_analyse("Forged");
        forged.id = "../settings".into();
        assert!(library.save(forged).is_err());
    }

    #[test]
    fn a_pipeline_is_a_one_step_workflow_and_shared_without_its_folder() {
        let workflow = Workflow::from_pipeline(pipeline("Extract", Task::TextExtraction));
        assert!(workflow.check().is_ok());
        assert_eq!(
            workflow.shape(),
            vec![(Task::TextExtraction, StepInput::Sources)]
        );
        let shared = shareable(extract_then_analyse("Letters"));
        assert!(
            shared
                .steps
                .iter()
                .all(|s| s.pipeline.destination == OutputDestination::BesideSources)
        );
    }
}
