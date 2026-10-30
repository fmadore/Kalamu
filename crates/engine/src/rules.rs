//! Which sources, models and formats suit each task: decided here once, given to the interface
//! so New job offers only what will run, and checked again when a job is created, so a job from
//! an old pipeline or a stale window cannot ask a model for what it does not do.

use kalamu_catalog::Catalog;
use kalamu_core::{ExportFormat, InputKind, ProviderId, Task, YOUTUBE_LINK};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::setup::NewJob;
use crate::{Engine, EngineError};

/// A model in the catalogue, by provider and id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ModelRef {
    pub provider: ProviderId,
    pub model: String,
}

/// What a task takes and gives, for the interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskRules {
    pub task: Task,
    /// The kinds of source it reads.
    pub sources: Vec<InputKind>,
    /// The formats its results are written in, in the order to offer them.
    pub formats: Vec<ExportFormat>,
    /// Whether the job's language is the one the model answers in, one only.
    pub answers_in_language: bool,
    /// The models that do it, in catalogue order.
    pub models: Vec<ModelRef>,
    /// For a table of contents, the models that can combine its pages; empty otherwise.
    pub merge_models: Vec<ModelRef>,
}

fn refs<'a>(models: impl IntoIterator<Item = &'a kalamu_catalog::ModelInfo>) -> Vec<ModelRef> {
    models
        .into_iter()
        .map(|m| ModelRef {
            provider: m.provider.clone(),
            model: m.id.clone(),
        })
        .collect()
}

/// Every task's rules, against a catalogue.
pub fn task_rules(catalog: &Catalog) -> Vec<TaskRules> {
    Task::ALL
        .into_iter()
        .map(|task| TaskRules {
            task,
            sources: task.source_kinds().to_vec(),
            formats: task.formats().to_vec(),
            answers_in_language: task.answers_in_language(),
            models: refs(catalog.models_for(task)),
            merge_models: if task == Task::Contents {
                refs(catalog.merge_models())
            } else {
                Vec::new()
            },
        })
        .collect()
}

impl Engine {
    /// Every task's rules, against the catalogue in use.
    pub fn task_rules(&self) -> Vec<TaskRules> {
        task_rules(&self.catalog())
    }

    /// Refuses a job whose sources the task does not read, or whose models do not do the task
    /// or read its sources. A model the catalogue does not list is left to its provider, which
    /// refuses what it cannot take.
    pub(crate) fn check_job(catalog: &Catalog, new: &NewJob) -> Result<(), EngineError> {
        if let Some(source) = new.sources.iter().find(|s| !new.task.reads_source(s.kind)) {
            return Err(EngineError::UnsupportedSource(source.file_name.clone()));
        }
        let links = new.sources.iter().any(|s| s.kind == InputKind::YouTube);
        for request in &new.setups {
            let Some(model) = catalog.model(&request.provider, &request.model) else {
                continue;
            };
            if !model.does(new.task) || (links && !model.inputs.iter().any(|i| i == YOUTUBE_LINK)) {
                return Err(EngineError::UnsupportedModel(request.model.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_follow_the_catalogue() {
        let catalog = Catalog::bundled();
        let rules = task_rules(&catalog);
        assert_eq!(rules.len(), Task::ALL.len());
        let contents = rules.iter().find(|r| r.task == Task::Contents);
        assert!(contents.is_some_and(|r| !r.models.is_empty() && !r.merge_models.is_empty()));
        for r in &rules {
            for m in &r.models {
                let info = catalog.model(&m.provider, &m.model);
                assert!(info.is_some_and(|i| i.does(r.task)));
            }
            if r.task != Task::Contents {
                assert!(r.merge_models.is_empty());
            }
        }
        // Whisper transcribes and does nothing else.
        let whisper = |r: &&TaskRules| r.models.iter().any(|m| m.provider.as_str() == "whisper");
        assert!(
            rules
                .iter()
                .filter(whisper)
                .all(|r| r.task == Task::AudioTranscription)
        );
    }

    /// The bundled catalogue's rules, written for the interface's tests and sample data, as
    /// ts-rs writes the types: `cargo test` keeps the file current.
    #[test]
    fn export_rules_for_the_interface() -> Result<(), Box<dyn std::error::Error>> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../src/lib/dev/taskRules.json");
        let json = serde_json::to_string_pretty(&task_rules(&Catalog::bundled()))? + "\n";
        if std::fs::read_to_string(&path).ok().as_deref() != Some(json.as_str()) {
            std::fs::write(&path, json)?;
        }
        Ok(())
    }

    #[test]
    fn a_model_is_refused_a_task_it_does_not_do() {
        let catalog = Catalog::bundled();
        let whisper = catalog
            .models
            .iter()
            .find(|m| m.provider.as_str() == "whisper")
            .map(|m| m.id.clone())
            .unwrap_or_default();
        let new = NewJob {
            name: "Test".into(),
            task: Task::TextExtraction,
            sources: Vec::new(),
            setups: vec![crate::SetupRequest {
                provider: ProviderId::new("whisper"),
                model: whisper.clone(),
                preset_id: None,
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: Default::default(),
            output: Default::default(),
        };
        assert!(matches!(
            Engine::check_job(&catalog, &new),
            Err(EngineError::UnsupportedModel(m)) if m == whisper
        ));
        // The same model transcribes.
        let new = NewJob {
            task: Task::AudioTranscription,
            ..new
        };
        assert!(Engine::check_job(&catalog, &new).is_ok());
    }
}
