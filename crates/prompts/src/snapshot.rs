//! A preset frozen for a job: its variables filled in, the fields of its answer described in
//! words beside the schema sent.

use kalamu_core::{PromptSnapshot, Task};

use crate::{Preset, RenderContext, describe, render};

/// The request sent with a table of contents' second step.
pub const MERGE_REQUEST: &str = "Combine the pages above into one entry per article or item, \
following the instructions given.";

/// Appended to a correction's instructions: where the corrected text goes in the answer.
const CORRECTION_ANSWER: &str = "\n\n## Answer\n\nPut the whole corrected text, and nothing \
else, in the field `text`. When the text needs no correction, return it unchanged.";

/// Appended to a translation's instructions: where the translated text goes in the answer.
const TRANSLATION_ANSWER: &str = "\n\n## Answer\n\nPut the whole translated text, and nothing \
else, in the field `text`. When the page or passage holds no text, leave the field empty.";

impl Preset {
    /// Fills in variables and freezes the result for a job.
    pub fn snapshot(&self, ctx: &RenderContext) -> PromptSnapshot {
        // The schema the model answers in: the fields themselves for an analysis, each page's
        // entries for a table of contents, the corrected or translated text for a correction or
        // a translation.
        let schema = match self.task {
            Task::Analysis => self.schema.clone(),
            Task::Contents => self.schema.as_ref().map(kalamu_core::contents::page_schema),
            Task::TextCorrection => Some(kalamu_core::correction::schema()),
            Task::Translation => Some(kalamu_core::correction::translation_schema()),
            Task::TextExtraction | Task::AudioTranscription => None,
        };
        // The fields are described in words as well as sent as a schema: the schema fixes the
        // answer's shape, the words carry what belongs in each field.
        let instructions = match (self.task, &schema) {
            (Task::TextCorrection, _) => {
                format!("{}{CORRECTION_ANSWER}", self.instructions.trim_end())
            }
            (Task::Translation, _) => {
                format!("{}{TRANSLATION_ANSWER}", self.instructions.trim_end())
            }
            (_, Some(schema)) => format!("{}{}", self.instructions.trim_end(), describe(schema)),
            (_, None) => self.instructions.clone(),
        };
        PromptSnapshot {
            preset_id: Some(self.id.clone()),
            preset_name: self.name.clone(),
            template: self.instructions.clone(),
            text: render(&instructions, ctx),
            user_prompt: render(&self.user_prompt, ctx),
            schema,
        }
    }

    /// A table of contents' second step, frozen for a job: the instructions that combine the
    /// pages, with the entry's fields. `None` for every other task. Lines are not its concern.
    pub fn merge_snapshot(&self, ctx: &RenderContext) -> Option<PromptSnapshot> {
        if self.task != Task::Contents {
            return None;
        }
        let template = self.merge_instructions.clone()?;
        let schema = kalamu_core::contents::merge_schema(self.schema.as_ref()?);
        let ctx = RenderContext {
            line_breaks: None,
            ..ctx.clone()
        };
        Some(PromptSnapshot {
            preset_id: Some(self.id.clone()),
            preset_name: self.name.clone(),
            text: render(
                &format!("{}{}", template.trim_end(), describe(&schema)),
                &ctx,
            ),
            template,
            user_prompt: MERGE_REQUEST.to_string(),
            schema: Some(schema),
        })
    }
}
