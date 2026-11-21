//! The step that brings together a long text read in parts (`Setup::windows`), frozen for a job
//! from an analysis prompt whose fields cite the text: its own instructions, then the
//! instructions each part was read with, and the fields it answers in, each field of entries
//! that cite the text answered with the numbers of its quotations
//! ([`kalamu_core::analysis::cited::consolidate`]).

use kalamu_core::analysis::cited::{consolidate, has_cited};
use kalamu_core::{PromptSnapshot, Task};

use crate::{Preset, RenderContext, describe, render};

/// The step's own instructions, before the prompt's.
const INSTRUCTIONS: &str = "# Bringing together a long text read in parts

The text was too long to be read at once, so it was read in parts, each made of whole numbered \
turns (a speaker's turn in a transcript, a passage in a document), each part repeating the \
last turns of the one before. You are given what each part found: its answers, with every \
quotation by its number (`Q1`, `Q2`…), and the quotations themselves with their turn, their \
speaker and time or their page when known, and their words. Write the answer for the whole \
text from them.

- Merge what is the same across parts: a theme found by two parts, or a passage quoted twice \
where two parts overlap, is one entry. Keep what is different apart.
- Order the entries as the instructions below ask, for the whole text, not part by part.
- Back each entry with the numbers of its quotations, taken from the parts. Use only numbers \
that appear in the parts' quotations, never words: the quotations are already checked against \
the text, and Kalamu puts their words back from their numbers.
- Answer a choice for the whole text, as the instructions below say for a text read in parts; \
when they say nothing of it, answer what the whole text shows, never what one part alone shows \
against the others.
- Write prose fields (a summary, a description, a reason) anew for the whole text, in \
{languages}, from what the parts say: add nothing they do not.
- A list that says what was not found (themes of the researcher's guide, say) holds only what \
no part found.
- When some turns could not be read (`turns_not_read`), answer from the parts read, and say \
nothing about what those turns may hold.

## The instructions each part was read with

";

/// The request sent with the parts' findings.
pub const CONSOLIDATION_REQUEST: &str = "Bring together the findings of the parts above into one \
answer for the whole text, following the instructions given. Back each entry only with the \
numbers of quotations the parts give.";

impl Preset {
    /// The step that brings a long text's parts together, frozen for a job: `None` for a prompt
    /// that is not an analysis citing the text.
    pub fn consolidation_snapshot(&self, ctx: &RenderContext) -> Option<PromptSnapshot> {
        let fields = self.schema.as_ref()?;
        if self.task != Task::Analysis || !has_cited(&fields.fields) {
            return None;
        }
        let schema = consolidate::schema(fields);
        let template = format!("{INSTRUCTIONS}{}", self.instructions.trim());
        Some(PromptSnapshot {
            preset_id: Some(self.id.clone()),
            preset_name: self.name.clone(),
            text: render(&format!("{template}{}", describe(&schema)), ctx),
            template,
            user_prompt: CONSOLIDATION_REQUEST.to_string(),
            schema: Some(schema),
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use kalamu_core::{FieldKind, LanguageChoice};

    use crate::PromptLibrary;

    fn ctx() -> crate::RenderContext {
        crate::RenderContext {
            languages: LanguageChoice::Auto,
            language_names: Vec::new(),
            speakers: None,
            line_breaks: None,
            answers: true,
        }
    }

    #[test]
    fn the_interview_prompt_brings_its_parts_together_by_the_numbers_of_its_quotations() {
        let library = PromptLibrary::in_memory();
        let interview = library.get("analysis.interview").unwrap();
        let step = interview.consolidation_snapshot(&ctx()).unwrap();
        assert_eq!(step.preset_id.as_deref(), Some("analysis.interview"));
        assert!(
            step.text
                .starts_with("# Bringing together a long text read in parts")
        );
        // The part's own instructions follow, then the fields described.
        assert!(step.text.contains("# Interview: themes and quotations"));
        let schema = step.schema.unwrap();
        let themes = schema.fields.iter().find(|f| f.key == "themes").unwrap();
        assert!(matches!(themes.kind, FieldKind::Entries { .. }));
        assert!(!step.text.contains("{languages}"));
        // A prompt that cites nothing has no such step.
        let summary = library.get("analysis.summary_research").unwrap();
        assert!(summary.consolidation_snapshot(&ctx()).is_none());
    }
}
