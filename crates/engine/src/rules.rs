//! Which sources, models and formats suit each task: decided here once, given to the interface
//! so New job offers only what will run, and checked again when a job is created, so a job from
//! an old pipeline or a stale window cannot ask a model for what it does not do.

use kalamu_catalog::Catalog;
use kalamu_core::{ExportFormat, InputKind, ProviderId, Task, YOUTUBE_LINK};
use kalamu_media::Reading;
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
    /// The reasoning level suggested for the task, for a model that reasons: what New job shows
    /// as "suggested" and sends when the researcher chooses none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub suggested_reasoning: Option<String>,
    /// The app's choice for the task, offered first.
    pub recommended: bool,
    /// Its provider's choice for the task, offered when the researcher picks that provider.
    pub provider_default: bool,
    /// For a model replaced by a newer one or retired, the model offered in its place: New job
    /// offers it only to a set-up that already names it, beside a button for its replacement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub replaced_by: Option<String>,
    /// Its provider no longer serves it: a job naming it is refused before anything is sent.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[ts(optional, as = "Option<bool>")]
    pub retired: bool,
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
    /// Whether that language must be chosen, with no detection to fall back on: a
    /// translation's.
    pub needs_language: bool,
    /// The models that do it, in catalogue order.
    pub models: Vec<ModelRef>,
    /// For a table of contents, the models that can combine its pages; empty otherwise.
    pub merge_models: Vec<ModelRef>,
    /// Its results are a text a later analysis can read, frozen as corrected, and scored
    /// against the researcher's review.
    pub yields_text: bool,
    /// Lines kept as on the page or joined, as chosen.
    pub line_breaks: bool,
    /// A sample of the first pages can be tried.
    pub samples: bool,
    /// Several pages of a PDF can go in one call.
    pub pages_per_call: bool,
    /// Double pages are worth splitting.
    pub splits_spreads: bool,
    /// A PDF's text layer is what it reads, not a sign it need not run.
    pub reads_text_layer: bool,
    /// Several images can be read as the pages of one document.
    pub page_documents: bool,
}

fn refs<'a>(
    catalog: &Catalog,
    task: Task,
    models: impl IntoIterator<Item = &'a kalamu_catalog::ModelInfo>,
) -> Vec<ModelRef> {
    models
        .into_iter()
        .map(|m| ModelRef {
            provider: m.provider.clone(),
            model: m.id.clone(),
            suggested_reasoning: m.tuning(task, None).reasoning,
            recommended: m.is_recommended_for(task),
            provider_default: m.is_provider_default_for(task),
            replaced_by: catalog.replacement(m).map(|r| r.id.clone()),
            retired: m.retired,
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
            needs_language: task.needs_language(),
            yields_text: task.yields_text(),
            line_breaks: task.has_line_breaks(),
            samples: task.takes_samples(),
            pages_per_call: task.groups_pages(),
            splits_spreads: task.splits_spreads(),
            reads_text_layer: task.reads_text_layer(),
            page_documents: task.reads_page_documents(),
            models: refs(catalog, task, catalog.models_for(task)),
            merge_models: if task == Task::Contents {
                refs(catalog, task, catalog.merge_models())
            } else {
                Vec::new()
            },
        })
        .collect()
}

/// The formats a job of these set-ups can write: those of its task, and a searchable PDF and
/// ALTO only when a set-up's model reports where text sits on the page.
pub(crate) fn writable_formats(
    catalog: &Catalog,
    task: Task,
    setups: &[crate::SetupRequest],
    formats: &mut Vec<ExportFormat>,
) {
    let layout = setups.iter().any(|s| {
        catalog
            .model(&s.provider, &s.model)
            .is_some_and(|m| m.layout)
    });
    formats.retain(|f| task.formats().contains(f) && (layout || !f.needs_layout()));
}

impl Engine {
    /// Every task's rules, against the catalogue in use.
    pub fn task_rules(&self) -> Vec<TaskRules> {
        task_rules(&self.current_catalog())
    }

    /// Refuses a job whose sources the task does not read, whose models do not do the task or
    /// read its sources, or name a model its provider no longer serves, and a translation with no
    /// language to translate into. A model the catalogue
    /// does not list is left to its provider, which refuses what it cannot take; one replaced by
    /// a newer model still runs, as the pipeline or comparison that named it asked.
    pub(crate) fn check_job(catalog: &Catalog, new: &NewJob) -> Result<(), EngineError> {
        let unread = |s: &&kalamu_core::SourceFile| {
            !new.task.reads_source(s.kind) || (s.is_page_set() && !new.task.reads_page_documents())
        };
        if let Some(source) = new.sources.iter().find(unread) {
            return Err(EngineError::UnsupportedSource(source.file_name.clone()));
        }
        if new.task.needs_language() && new.settings.languages.codes().is_empty() {
            return Err(EngineError::NoLanguage);
        }
        let links = new.sources.iter().any(|s| s.kind == InputKind::YouTube);
        for request in &new.setups {
            let Some(model) = catalog.model(&request.provider, &request.model) else {
                continue;
            };
            if model.retired {
                return Err(EngineError::RetiredModel(model.name.clone()));
            }
            if !model.does(new.task) || (links && !model.inputs.iter().any(|i| i == YOUTUBE_LINK)) {
                return Err(EngineError::UnsupportedModel(request.model.clone()));
            }
        }
        Ok(())
    }

    /// Refuses a job whose model cannot be given one of its sources at all: images or a whole
    /// PDF to a model that reads text only, a whole PDF to one that reads images but no PDF.
    /// Decided by media's own rule ([`kalamu_media::readable`]) on what the model would be sent
    /// ([`crate::piece::accepted_inputs`]), so the job is refused for exactly what each piece
    /// would have failed on. A model the catalogue does not list is left to its provider.
    pub(crate) fn check_reading(&self, catalog: &Catalog, new: &NewJob) -> Result<(), EngineError> {
        let reading = if new.task == Task::Analysis {
            Reading::Whole
        } else {
            Reading::Pieces
        };
        for request in &new.setups {
            let Some(model) = catalog.model(&request.provider, &request.model) else {
                continue;
            };
            let provider = self.provider(&request.provider)?;
            let accepted = crate::piece::accepted_inputs(provider.as_ref(), Some(model), new.task);
            for source in &new.sources {
                if let Err(e) = kalamu_media::readable(source.kind, reading, &accepted) {
                    return Err(EngineError::CannotRead {
                        model: model.name.clone(),
                        file: source.file_name.clone(),
                        reason: e.to_string(),
                    });
                }
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

    #[test]
    fn a_searchable_pdf_is_written_only_by_a_model_that_reports_layout() {
        let catalog = Catalog::bundled();
        let setup = |provider: &str, model: &str| crate::SetupRequest {
            provider: ProviderId::new(provider),
            model: model.into(),
            preset_id: None,
            label: None,
            options: Default::default(),
            merge: None,
        };
        let asked = vec![ExportFormat::Txt, ExportFormat::SearchablePdf];
        let mut formats = asked.clone();
        let gemini = [setup("gemini", "gemini-3.8-flash")];
        writable_formats(&catalog, Task::TextExtraction, &gemini, &mut formats);
        assert_eq!(formats, [ExportFormat::Txt]);
        let mut formats = asked.clone();
        let both = [
            setup("gemini", "gemini-3.8-flash"),
            setup("mistral", "mistral-ocr-4-1"),
        ];
        writable_formats(&catalog, Task::TextExtraction, &both, &mut formats);
        assert_eq!(formats, asked);
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

    fn fable_5_1_job() -> NewJob {
        NewJob {
            name: "Test".into(),
            task: Task::Analysis,
            sources: Vec::new(),
            setups: vec![crate::SetupRequest {
                provider: ProviderId::new("anthropic"),
                model: "claude-fable-5-1".into(),
                preset_id: None,
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: Default::default(),
            output: Default::default(),
        }
    }

    /// A model replaced by a newer one stays in the rules, marked with its replacement, so New
    /// job can show a pipeline that names it; it still runs.
    #[test]
    fn a_replaced_model_is_marked_and_still_runs() {
        let catalog = Catalog::bundled();
        let rules = task_rules(&catalog);
        let analysis = rules.iter().find(|r| r.task == Task::Analysis);
        let entry = |id: &str| {
            analysis
                .and_then(|r| r.models.iter().find(|m| m.model == id))
                .cloned()
        };
        let old = entry("claude-fable-5-1");
        assert_eq!(
            old.as_ref().and_then(|m| m.replaced_by.as_deref()),
            Some("claude-fable-5-2")
        );
        assert!(old.is_some_and(|m| !m.retired && !m.recommended && !m.provider_default));
        assert!(entry("claude-fable-5-2").is_some_and(|m| m.replaced_by.is_none()));
        assert!(Engine::check_job(&catalog, &fable_5_1_job()).is_ok());
    }

    /// Once its provider no longer serves it, a job naming it is refused before anything is
    /// sent, with a message that says a replacement exists.
    #[test]
    fn a_retired_model_is_refused() {
        let mut catalog = Catalog::bundled();
        for model in catalog.models.iter_mut() {
            if model.id == "claude-fable-5-1" {
                model.retired = true;
            }
        }
        assert!(matches!(
            Engine::check_job(&catalog, &fable_5_1_job()),
            Err(EngineError::RetiredModel(name)) if name == "Claude Fable 5.1"
        ));
        let rules = task_rules(&catalog);
        let retired = rules
            .iter()
            .flat_map(|r| &r.models)
            .find(|m| m.model == "claude-fable-5-1");
        assert!(retired.is_some_and(|m| m.retired && m.replaced_by.is_some()));
    }
}
