//! A workflow as a file to share or cite: the definition, and each step's pipeline record with
//! its prompts in full. Like a pipeline's file it leaves out output folders, and it never holds
//! keys, source text or anything a run produced.

use std::fmt::Write as _;

use kalamu_core::provenance::GeneratorInfo;
use kalamu_core::{StepInput, Timestamp};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::record::{import_presets, remap_presets, task_name};
use crate::workflows::shareable;
use crate::{PipelineFile, PromptError, PromptLibrary, Workflow, WorkflowLibrary};

const FILE_FORMAT: &str = "kalamu-workflow";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkflowFile {
    pub format: String,
    pub version: u32,
    pub generator: GeneratorInfo,
    pub exported_at: Timestamp,
    pub workflow: Workflow,
    /// One per step, in order: what each sends, prompts and presets included.
    pub steps: Vec<PipelineFile>,
}

impl WorkflowFile {
    pub fn new(workflow: Workflow, app_version: &str, steps: Vec<PipelineFile>) -> Self {
        Self {
            format: FILE_FORMAT.into(),
            version: 1,
            generator: GeneratorInfo {
                name: "Kalamu".into(),
                version: app_version.into(),
            },
            exported_at: Timestamp::now(),
            workflow: shareable(workflow),
            steps,
        }
    }

    pub fn to_json(&self) -> Result<String, PromptError> {
        serde_json::to_string_pretty(self).map_err(crate::io)
    }

    /// The record as Markdown, for a methods section: the steps in order and what each reads,
    /// then each step's models, settings and prompts.
    pub fn to_markdown(&self) -> String {
        let w = &self.workflow;
        let mut out = String::new();
        let _ = writeln!(out, "# {}\n", w.name);
        let _ = writeln!(
            out,
            "A Kalamu workflow (version {}), exported on {} from {} {}. The JSON file \
             exported with it can be imported into Kalamu to run the same steps.\n",
            w.version,
            self.exported_at.utc_date(),
            self.generator.name,
            self.generator.version
        );
        out.push_str("## Steps\n\n");
        for (i, step) in w.steps.iter().enumerate() {
            let reads = match step.input {
                StepInput::Sources => "reads the files the workflow is run on".to_string(),
                StepInput::Step { step } => {
                    let name = w.steps.get(step as usize).map_or("", |s| &s.pipeline.name);
                    format!(
                        "reads the text of step {} ({name}) as corrected when it starts",
                        step + 1
                    )
                }
            };
            let review = if step.review && w.steps.iter().any(|s| s.input == read(i)) {
                "; the steps that read it wait until its results are reviewed"
            } else {
                ""
            };
            let _ = writeln!(
                out,
                "{}. **{}**: {}, {reads}{review}.",
                i + 1,
                step.pipeline.name,
                task_name(step.pipeline.task)
            );
        }
        for (i, (step, file)) in w.steps.iter().zip(&self.steps).enumerate() {
            let _ = writeln!(out, "\n## Step {}: {}\n", i + 1, step.pipeline.name);
            // Each pipeline's sections, a level down under its step.
            for line in file.sections().lines() {
                if line.starts_with('#') {
                    out.push('#');
                }
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    }
}

fn read(i: usize) -> StepInput {
    StepInput::Step { step: i as u32 }
}

impl WorkflowLibrary {
    /// Adds the workflow in a shared file, with its presets, under a name not yet taken. A
    /// preset identical to one already here is used as it is; any other becomes a user preset.
    pub fn import(
        &mut self,
        json: &str,
        prompts: &mut PromptLibrary,
    ) -> Result<Workflow, PromptError> {
        let file: WorkflowFile =
            serde_json::from_str(json).map_err(|e| PromptError::InvalidImport(e.to_string()))?;
        if file.format != FILE_FORMAT || file.version > 1 {
            return Err(PromptError::InvalidImport(file.format));
        }
        let mut workflow = shareable(file.workflow);
        workflow.check()?;
        let presets = file.steps.into_iter().flat_map(|s| s.presets).collect();
        let ids = import_presets(presets, &workflow.name, prompts)?;
        for step in &mut workflow.steps {
            remap_presets(&mut step.pipeline, &ids);
        }
        workflow.id = String::new();
        workflow.name = self.free_name(&workflow.name);
        self.save(workflow)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use kalamu_core::Task;

    use super::*;
    use crate::workflows::tests::extract_then_analyse;

    fn file(prompts: &PromptLibrary, workflow: Workflow) -> WorkflowFile {
        let steps = workflow
            .steps
            .iter()
            .map(|s| {
                let preset = prompts
                    .list(Some(s.pipeline.task))
                    .into_iter()
                    .next()
                    .unwrap();
                let mut pipeline = s.pipeline.clone();
                pipeline.setups[0].preset_id = Some(preset.id.clone());
                PipelineFile::new(pipeline, "1.0.0", Vec::new(), Vec::new(), vec![preset])
            })
            .collect();
        WorkflowFile::new(workflow, "1.0.0", steps)
    }

    #[test]
    fn a_shared_workflow_is_imported_beside_one_of_the_same_name() {
        let mut prompts = PromptLibrary::in_memory();
        let mut library = WorkflowLibrary::in_memory();
        let saved = library.save(extract_then_analyse("Letters")).unwrap();
        let shared = file(&prompts, saved.clone());
        let json = shared.to_json().unwrap();
        assert!(!json.contains(r"C:\\Users"), "no output folder: {json}");

        let imported = library.import(&json, &mut prompts).unwrap();
        assert_ne!(imported.id, saved.id);
        assert_eq!(imported.name, "Letters (2)");
        assert_eq!(imported.steps.len(), 2);
        assert_eq!(imported.steps[1].input, StepInput::Step { step: 0 });
        assert!(imported.steps[1].pipeline.task == Task::Analysis);

        let markdown = shared.to_markdown();
        assert!(markdown.starts_with("# Letters\n"));
        assert!(markdown.contains(
            "2. **People and places**: analysis, reads the text of step 1 (Extract) as \
             corrected when it starts."
        ));
        assert!(markdown.contains("the steps that read it wait until its results are reviewed"));
        assert!(markdown.contains("## Step 2: People and places\n\n### Task"));

        assert!(matches!(
            library.import("{\"format\":\"kalamu-pipeline\"}", &mut prompts),
            Err(PromptError::InvalidImport(_))
        ));
    }
}
