use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AnalysisSchema, ProviderId, SetupId};

/// The prompt a set-up uses, frozen at job creation so later edits to a preset never change
/// what a job did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PromptSnapshot {
    /// The preset it came from, if any (`text_extraction.printed`, or a user preset's id).
    pub preset_id: Option<String>,
    pub preset_name: String,
    /// The instructions as written, with `{variables}`.
    pub template: String,
    /// The instructions sent to the model (system prompt), variables filled in.
    pub text: String,
    /// The short request sent with each page or segment.
    #[serde(default)]
    pub user_prompt: String,
    /// For an analysis, the fields its answer must have.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub schema: Option<AnalysisSchema>,
}

/// Provider-independent options for one set-up.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupOptions {
    /// Sampling temperature, where the provider supports it.
    pub temperature: Option<f32>,
    /// Ask for speaker labels in transcriptions.
    #[serde(default)]
    pub diarize: bool,
    /// Ask for timestamps in transcriptions.
    #[serde(default)]
    pub timestamps: bool,
    /// Expected number of speakers, if known.
    pub speakers: Option<u32>,
    /// How much the model thinks before it answers, in its provider's words (`low`, `high`).
    /// Chosen in New job, or the suggestion for the task; settled when the job is created, so
    /// the job, its cache keys and provenance hold the level sent. `None` for a model that does
    /// not reason, and in jobs made before levels could be chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reasoning: Option<String>,
    /// Names and words to spell as the researcher writes them (people, places, terms of the
    /// field), one per line or separated by commas, for a transcription model that takes them:
    /// Whisper reads them as its initial prompt. Left out when empty, so cache keys made before
    /// it existed stay valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub vocabulary: Option<String>,
    /// Write the speech in English rather than in the language spoken, for a model that can
    /// (Whisper Large v3, Small and Base; not Turbo). Left out when off.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[ts(optional, as = "Option<bool>")]
    pub translate: bool,
    /// What the researcher's work is about, in their words ("What is your research about?"),
    /// for an analysis: added to the request sent with the document, so the same question
    /// costs nothing twice and another is another analysis. Left out when empty, so cache keys
    /// made before it existed stay valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub question: Option<String>,
    /// The researcher's own themes or interview guide, one per line, for an analysis whose
    /// themes say whether each comes from the guide or emerged from the document: added to the
    /// request after the question, so other themes are another analysis. Left out when empty,
    /// so cache keys made before it existed stay valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub themes: Option<String>,
}

/// Introduces the researcher's question in the request.
pub const QUESTION_INTRO: &str = "The researcher's work, in their own words:";
/// Introduces the researcher's own themes in the request.
pub const THEMES_INTRO: &str = "The researcher's own themes or interview guide, one per line:";

impl SetupOptions {
    /// The request sent with each document with the researcher's question and their own
    /// themes after it, when there are any: what an analysis asks of every document.
    pub fn with_question(&self, request: &str) -> String {
        let mut out = match self.question.as_deref().map(str::trim) {
            Some(question) if !question.is_empty() => {
                format!("{request}\n\n{QUESTION_INTRO} {question}")
            }
            _ => request.to_string(),
        };
        let themes = self.theme_lines();
        if !themes.is_empty() {
            out.push_str(&format!("\n\n{THEMES_INTRO}"));
            for theme in themes {
                out.push_str(&format!("\n- {theme}"));
            }
        }
        out
    }

    /// The researcher's themes, one each, trimmed, without empty lines, list marks or repeats.
    pub fn theme_lines(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for line in self.themes.as_deref().unwrap_or("").lines() {
            let theme = line.trim().trim_start_matches(['-', '*', '•']).trim();
            if !theme.is_empty() && !out.iter().any(|o| o.eq_ignore_ascii_case(theme)) {
                out.push(theme.to_owned());
            }
        }
        out
    }

    /// The names and words to spell, one each, trimmed, without empty ones or repeats.
    pub fn vocabulary_terms(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for term in self
            .vocabulary
            .as_deref()
            .unwrap_or("")
            .split(['\n', ',', ';'])
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            if !out.iter().any(|o| o.eq_ignore_ascii_case(term)) {
                out.push(term.to_owned());
            }
        }
        out
    }
}

/// One provider, model and prompt to run, as chosen in New job or saved in a pipeline, before
/// its prompt is frozen into a [`Setup`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupRequest {
    pub provider: ProviderId,
    pub model: String,
    /// A preset id; the task's default when absent.
    pub preset_id: Option<String>,
    /// Shown in comparison columns and file names; derived from the model when absent.
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub options: SetupOptions,
    /// For a table of contents, the model that combines the pages; the same model as the
    /// pages' when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub merge: Option<MergeModel>,
}

/// A provider, model and prompt combination. A normal job has one; comparison mode has several.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Setup {
    pub id: SetupId,
    /// Shown in comparison columns and export file names (`gemini-flash`).
    pub label: String,
    pub provider: ProviderId,
    pub model: String,
    pub prompt: PromptSnapshot,
    #[serde(default)]
    pub options: SetupOptions,
    /// For a table of contents, the second step: the model that combines the pages' readings,
    /// which may be another provider's (a large model reads the pages, a small one combines).
    /// For an analysis read in parts (`windows`), the step that brings the parts together, on
    /// the set-up's own model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub merge: Option<MergeStep>,
    /// For a text extraction that reads several pages in one call, the most pages this model
    /// can write in one answer: settled when the job is created, so a later catalogue refresh
    /// never regroups its pages nor changes their cache keys. `None` when the job reads one
    /// page at a time, and in jobs made before it was settled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub pages_cap: Option<u32>,
    /// For an analysis that cites a text too long for its model to read at once, how the text
    /// is read in parts of whole turns before `merge` brings them together: settled when the
    /// job is created from the catalogue's limit. `None` when every text fits, and in jobs
    /// made before texts were read in parts, which read each text whole.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub windows: Option<crate::analysis::cited::TurnWindows>,
}

/// A second step, one call per document over the first step's answers: a table of contents'
/// pages combined into entries, or a long text's parts brought together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MergeStep {
    pub provider: ProviderId,
    pub model: String,
    pub prompt: PromptSnapshot,
}

/// The model chosen for a table of contents' second step, before its prompt is frozen: in a
/// new job's request and in a saved pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MergeModel {
    pub provider: ProviderId,
    pub model: String,
}

/// One call's provider, model and prompt: a set-up's, or its merge step's.
#[derive(Debug, Clone, Copy)]
pub struct Step<'a> {
    pub provider: &'a ProviderId,
    pub model: &'a str,
    pub prompt: &'a PromptSnapshot,
}

impl Setup {
    /// The call each page or segment makes.
    pub fn step(&self) -> Step<'_> {
        Step {
            provider: &self.provider,
            model: &self.model,
            prompt: &self.prompt,
        }
    }

    /// The call that combines a document's pages, for a table of contents.
    pub fn merge_step(&self) -> Option<Step<'_>> {
        self.merge.as_ref().map(|m| Step {
            provider: &m.provider,
            model: &m.model,
            prompt: &m.prompt,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_to_spell_are_read_one_each_without_repeats() {
        let options = SetupOptions {
            vocabulary: Some("Samory Touré, Ouagadougou\n\n samory touré ;Bobo-Dioulasso\n".into()),
            ..SetupOptions::default()
        };
        assert_eq!(
            options.vocabulary_terms(),
            ["Samory Touré", "Ouagadougou", "Bobo-Dioulasso"]
        );
        assert!(SetupOptions::default().vocabulary_terms().is_empty());
    }

    #[test]
    fn the_researchers_themes_follow_the_question_one_per_line() {
        let options = SetupOptions {
            question: Some("Le port".into()),
            themes: Some("- Les débuts\n\n  • la tontine \nles débuts\n".into()),
            ..SetupOptions::default()
        };
        assert_eq!(options.theme_lines(), ["Les débuts", "la tontine"]);
        assert_eq!(
            options.with_question("Read it."),
            format!(
                "Read it.\n\n{QUESTION_INTRO} Le port\n\n{THEMES_INTRO}\n- Les débuts\n- la tontine"
            )
        );
        // None typed: the request as it always was.
        assert_eq!(
            SetupOptions::default().with_question("Read it."),
            "Read it."
        );
        let json = serde_json::to_string(&SetupOptions::default()).unwrap_or_default();
        assert!(!json.contains("themes"), "{json}");
    }

    /// Jobs and cache keys made before the names and the translation existed read the same.
    #[test]
    fn options_left_off_are_not_written() {
        let json = serde_json::to_string(&SetupOptions::default()).unwrap_or_default();
        assert!(
            !json.contains("vocabulary") && !json.contains("translate"),
            "{json}"
        );
    }
}
