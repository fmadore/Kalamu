//! Pipelines: a job's settings saved under a name, so the same kind of job starts again in one
//! click. Everything but the files: the task, each set-up's model and preset, languages, line
//! breaks, formats and where results go. One JSON file each, like user presets.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use kalamu_core::{
    ExportFormat, JobSettings, OutputDestination, ProviderId, SetupOptions, Task, Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{PromptError, io, write_atomic};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Pipeline {
    /// Empty for a new pipeline; given when it is first saved.
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub task: Task,
    pub setups: Vec<PipelineSetup>,
    #[serde(default)]
    pub settings: JobSettings,
    #[serde(default)]
    pub formats: Vec<ExportFormat>,
    #[serde(default)]
    pub destination: OutputDestination,
    /// When it was last saved; the interface sends 0 and saving sets it.
    pub updated_at: Timestamp,
}

/// One model and its preset, as New job sets it up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PipelineSetup {
    pub provider: ProviderId,
    pub model: String,
    pub preset_id: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub options: SetupOptions,
}

pub struct PipelineLibrary {
    dir: Option<PathBuf>,
    saved: BTreeMap<String, Pipeline>,
}

impl PipelineLibrary {
    /// Loads saved pipelines from `dir`, creating it if needed; an unreadable file is skipped.
    pub fn open(dir: &Path) -> Result<Self, PromptError> {
        fs::create_dir_all(dir).map_err(io)?;
        let mut saved = BTreeMap::new();
        for entry in fs::read_dir(dir).map_err(io)? {
            let path = entry.map_err(io)?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(pipeline) = fs::read_to_string(&path)
                .map_err(io)
                .and_then(|s| serde_json::from_str::<Pipeline>(&s).map_err(io))
            {
                saved.insert(pipeline.id.clone(), pipeline);
            }
        }
        Ok(Self {
            dir: Some(dir.to_path_buf()),
            saved,
        })
    }

    /// Nothing saved; for tests.
    pub fn in_memory() -> Self {
        Self {
            dir: None,
            saved: BTreeMap::new(),
        }
    }

    /// By name, ignoring case.
    pub fn list(&self) -> Vec<Pipeline> {
        let mut out: Vec<Pipeline> = self.saved.values().cloned().collect();
        out.sort_by_key(|p| p.name.to_lowercase());
        out
    }

    /// Saves a pipeline under its id, or a new one. Saving under a name already used replaces
    /// that pipeline, as saving a file under its name would.
    pub fn save(&mut self, mut pipeline: Pipeline) -> Result<Pipeline, PromptError> {
        pipeline.name = pipeline.name.trim().to_string();
        if pipeline.name.is_empty() || pipeline.setups.is_empty() {
            return Err(PromptError::IncompletePipeline);
        }
        if pipeline.id.is_empty() {
            pipeline.id = self
                .saved
                .values()
                .find(|p| p.name.to_lowercase() == pipeline.name.to_lowercase())
                .map(|p| p.id.clone())
                .unwrap_or_else(|| format!("pipeline-{}", uuid::Uuid::new_v4()));
        }
        if uuid::Uuid::parse_str(pipeline.id.trim_start_matches("pipeline-")).is_err() {
            return Err(PromptError::NotFound(pipeline.id));
        }
        pipeline.updated_at = Timestamp::now();
        if let Some(dir) = &self.dir {
            write_atomic(&dir.join(format!("{}.json", pipeline.id)), &pipeline)?;
        }
        self.saved.insert(pipeline.id.clone(), pipeline.clone());
        Ok(pipeline)
    }

    pub fn delete(&mut self, id: &str) -> Result<(), PromptError> {
        if self.saved.remove(id).is_none() {
            return Err(PromptError::NotFound(id.to_string()));
        }
        if let Some(dir) = &self.dir {
            let path = dir.join(format!("{id}.json"));
            if path.exists() {
                fs::remove_file(path).map_err(io)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn pipeline(name: &str) -> Pipeline {
        Pipeline {
            id: String::new(),
            name: name.into(),
            task: Task::Analysis,
            setups: vec![PipelineSetup {
                provider: ProviderId::new("gemini"),
                model: "gemini-3.8-flash".into(),
                preset_id: Some("analysis.entities".into()),
                label: None,
                options: SetupOptions::default(),
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
