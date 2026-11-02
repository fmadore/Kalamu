//! Pipelines: a job's settings saved under a name, so the same kind of job starts again in one
//! click. Everything but the files: the task, each set-up's model and preset, languages, line
//! breaks, formats and where results go. One JSON file each, like user presets.

use std::path::Path;

use kalamu_core::{ExportFormat, JobSettings, OutputDestination, SetupRequest, Task, Timestamp};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::PromptError;
use crate::folder::{Folder, Saved};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Pipeline {
    /// Empty for a new pipeline; given when it is first saved.
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub task: Task,
    pub setups: Vec<SetupRequest>,
    #[serde(default)]
    pub settings: JobSettings,
    #[serde(default)]
    pub formats: Vec<ExportFormat>,
    #[serde(default)]
    pub destination: OutputDestination,
    /// When it was last saved; the interface sends 0 and saving sets it.
    pub updated_at: Timestamp,
}

impl Saved for Pipeline {
    const PREFIX: &'static str = "pipeline-";
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct PipelineLibrary(Folder<Pipeline>);

impl PipelineLibrary {
    /// Loads saved pipelines from `dir`, creating it if needed; an unreadable file is skipped.
    pub fn open(dir: &Path) -> Result<Self, PromptError> {
        Folder::open(dir).map(Self)
    }

    /// Nothing saved; for tests.
    pub fn in_memory() -> Self {
        Self(Folder::in_memory())
    }

    /// By name, ignoring case.
    pub fn list(&self) -> Vec<Pipeline> {
        self.0.list()
    }

    pub fn get(&self, id: &str) -> Result<Pipeline, PromptError> {
        self.0.get(id)
    }

    /// Saves a pipeline under its id, or a new one. Saving under a name already used replaces
    /// that pipeline, as saving a file under its name would.
    pub fn save(&mut self, mut pipeline: Pipeline) -> Result<Pipeline, PromptError> {
        pipeline.name = pipeline.name.trim().to_string();
        if pipeline.name.is_empty() || pipeline.setups.is_empty() {
            return Err(PromptError::IncompletePipeline);
        }
        pipeline.id = self.0.id_for(&pipeline)?;
        pipeline.updated_at = Timestamp::now();
        self.0.put(pipeline)
    }

    pub fn delete(&mut self, id: &str) -> Result<(), PromptError> {
        self.0.delete(id)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use kalamu_core::{ProviderId, SetupOptions};

    use super::*;

    fn pipeline(name: &str) -> Pipeline {
        Pipeline {
            id: String::new(),
            name: name.into(),
            task: Task::Analysis,
            setups: vec![SetupRequest {
                provider: ProviderId::new("gemini"),
                model: "gemini-3.8-flash".into(),
                preset_id: Some("analysis.entities".into()),
                label: None,
                options: SetupOptions::default(),
                merge: None,
            }],
            settings: JobSettings::default(),
            formats: vec![ExportFormat::Csv],
            destination: OutputDestination::default(),
            updated_at: Timestamp(0),
        }
    }

    #[test]
    fn pipelines_are_saved_listed_replaced_by_name_and_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let mut library = PipelineLibrary::open(dir.path()).unwrap();
        let first = library.save(pipeline("Press, people and places")).unwrap();
        library.save(pipeline("Archives")).unwrap();
        let again = library
            .save(pipeline(" press, PEOPLE and places "))
            .unwrap();
        assert_eq!(again.id, first.id, "the same name replaces the pipeline");
        let reopened = PipelineLibrary::open(dir.path()).unwrap();
        let names: Vec<_> = reopened.list().into_iter().map(|p| p.name).collect();
        assert_eq!(names, ["Archives", "press, PEOPLE and places"]);
        library.delete(&first.id).unwrap();
        assert_eq!(PipelineLibrary::open(dir.path()).unwrap().list().len(), 1);
    }

    #[test]
    fn a_pipeline_needs_a_name_and_a_model() {
        let mut library = PipelineLibrary::in_memory();
        assert!(matches!(
            library.save(pipeline("  ")),
            Err(PromptError::IncompletePipeline)
        ));
        let mut empty = pipeline("Empty");
        empty.setups.clear();
        assert!(library.save(empty).is_err());
        let mut forged = pipeline("Forged");
        forged.id = "../settings".into();
        assert!(
            library.save(forged).is_err(),
            "an id never becomes a path outside the folder"
        );
    }
}
