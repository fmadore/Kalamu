//! Correcting a passage the researcher selected in the review, with a model, without a job over
//! the whole document: the passage is sent with a little of the text around it, marked as
//! context the model must not change, through the correction task's request (its presets, its
//! one field); the answer is checked and offered as a proposal, written into the researcher's
//! corrections only when they accept it. The model's own output is never changed.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{JobId, PromptSnapshot, ProviderId, RequestSetting, SetupId, SourceId, Timestamp};

/// The fewest letters or figures a passage may hold: less is not worth a call.
pub const MIN_CHARS: usize = 3;
/// The most characters a passage may hold, about a page and a half of print: the corrected
/// passage must come back whole in one answer, which a long passage risks; a longer stretch is
/// corrected by a job, page by page, with its retries.
pub const MAX_CHARS: usize = 4_000;
/// The characters of context sent on each side of the passage, at most.
pub const CONTEXT_CHARS: usize = 300;

/// Appended to the correction preset's request, so the model corrects the passage alone.
pub const REQUEST: &str = "Only the text between <passage> and </passage> is to be corrected. \
The text between <before> and </before>, and between <after> and </after>, is context, given \
only so that you can read the passage in its place: do not correct it and do not repeat it. Put \
the corrected passage alone, without the tags, in the field `text`, keeping its line breaks \
where they are. When the passage needs no correction, return it unchanged.";

/// What the researcher asks: a passage of one piece's text as the review shows it, the context
/// around it, and the model and prompt to correct it with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PassageAsk {
    /// Chosen by the interface, so that the request can be cancelled while it waits.
    pub id: String,
    pub job_id: JobId,
    /// The set-up whose text was selected; its correction layer takes the accepted passage.
    pub setup_id: SetupId,
    pub source_id: SourceId,
    pub index: u32,
    /// Up to [`CONTEXT_CHARS`] characters before the passage, in the same piece.
    pub before: String,
    pub passage: String,
    /// Up to [`CONTEXT_CHARS`] characters after it, in the same piece.
    pub after: String,
    /// Where the passage starts in the piece's text, in UTF-16 code units as the interface
    /// counts them: which of several identical stretches was meant.
    pub at: u32,
    pub provider: ProviderId,
    pub model: String,
    /// A correction preset; the default one when none is given.
    pub preset_id: Option<String>,
}

/// Where a proposal stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProposalStatus {
    /// Recorded before the call; still waiting for the model.
    Asked,
    /// The model answered and the answer is offered.
    Answered,
    /// The call failed, or its answer was refused.
    Failed,
    /// The researcher stopped waiting, or Kalamu closed while it waited.
    Cancelled,
    /// Written into the researcher's corrections.
    Accepted,
    Discarded,
    /// Accepted, then put back as it was.
    Undone,
}

impl ProposalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asked => "asked",
            Self::Answered => "answered",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Accepted => "accepted",
            Self::Discarded => "discarded",
            Self::Undone => "undone",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::Asked,
            Self::Answered,
            Self::Failed,
            Self::Cancelled,
            Self::Accepted,
            Self::Discarded,
            Self::Undone,
        ]
        .into_iter()
        .find(|v| v.as_str() == s)
    }
}

/// What was done to the model's answer before it was offered, for the researcher to know.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProposalNote {
    /// The model gave back some of the context too; it was taken out.
    ContextRemoved,
    /// The model wrapped the passage in words of its own; they were taken out.
    CommentaryRemoved,
    /// The answer is much longer or shorter than the passage.
    LengthChanged,
    /// The model changed nothing.
    Unchanged,
}

/// A passage asked about, as the store keeps it: the request, the prompt as sent, the answer,
/// what it cost and what the researcher decided. Recorded before the call and after it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PassageProposal {
    pub ask: PassageAsk,
    pub status: ProposalStatus,
    /// The correction preset's prompt as sent, the passage's request included.
    pub prompt: PromptSnapshot,
    /// What the request carried besides the prompt.
    #[serde(default)]
    pub request: Vec<RequestSetting>,
    /// The key of the stored answer, the same for the same passage, context, model and prompt.
    pub cache_key: String,
    /// The corrected passage, once checked; its spaces at either end are the passage's own.
    pub answer: Option<String>,
    #[serde(default)]
    pub notes: Vec<ProposalNote>,
    /// What this request cost; none when the answer came from the cache or has no price.
    pub cost_usd: Option<f64>,
    /// The answer was one already stored: asking cost nothing.
    #[serde(default)]
    pub cached: bool,
    pub error: Option<String>,
    /// The text written into the correction: the answer, or the researcher's edit of it.
    pub accepted: Option<String>,
    pub asked_at: Timestamp,
    pub answered_at: Option<Timestamp>,
    pub decided_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PassageError {
    #[error("Select at least {MIN_CHARS} letters or figures")]
    TooShort,
    #[error("The passage is {0} characters long; at most {MAX_CHARS} are sent at once")]
    TooLong(usize),
    #[error("The context around the passage is longer than {CONTEXT_CHARS} characters")]
    ContextTooLong,
    /// The text under the selection is no longer as it was.
    #[error("The text under the selection has changed")]
    Changed,
    #[error("The model's answer was empty")]
    Empty,
    #[error(
        "The model's answer is {answer} characters long, the passage {passage}: too different to \
         be a correction"
    )]
    TooDifferent { passage: usize, answer: usize },
    /// The proposal is not waiting for a decision (decided already, failed or cancelled).
    #[error("This proposal is no longer waiting for a decision")]
    NotWaiting,
}

impl PassageError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooShort => "passage.too_short",
            Self::TooLong(_) => "passage.too_long",
            Self::ContextTooLong => "passage.context",
            Self::Changed => "passage.changed",
            Self::Empty => "passage.empty",
            Self::TooDifferent { .. } => "passage.too_different",
            Self::NotWaiting => "passage.not_waiting",
        }
    }
}

/// Whether a passage and its context may be sent.
pub fn check(ask: &PassageAsk) -> Result<(), PassageError> {
    let letters = ask.passage.chars().filter(|c| c.is_alphanumeric()).count();
    if letters < MIN_CHARS {
        return Err(PassageError::TooShort);
    }
    let length = ask.passage.chars().count();
    if length > MAX_CHARS {
        return Err(PassageError::TooLong(length));
    }
    // A little slack: the interface cuts the context at a word's edge.
    let most = CONTEXT_CHARS + 40;
    if ask.before.chars().count() > most || ask.after.chars().count() > most {
        return Err(PassageError::ContextTooLong);
    }
    Ok(())
}

/// The text the model reads: the passage between its context, each part marked.
pub fn input(before: &str, passage: &str, after: &str) -> String {
    format!(
        "<before>\n{before}\n</before>\n<passage>\n{passage}\n</passage>\n<after>\n{after}\n</after>"
    )
}

/// Where the passage starts in `text`, in bytes: the stretch of context, passage and context
/// that starts at the passage's place `at` (UTF-16 units), or else the only such stretch.
/// `None` when the text under the selection changed.
pub fn locate(text: &str, ask: &PassageAsk) -> Option<usize> {
    let whole = format!("{}{}{}", ask.before, ask.passage, ask.after);
    let starts: Vec<usize> = text
        .match_indices(whole.as_str())
        .map(|(i, _)| i + ask.before.len())
        .collect();
    let utf16 = |byte: usize| text[..byte].encode_utf16().count();
    starts
        .iter()
        .copied()
        .find(|&s| utf16(s) == ask.at as usize)
        .or_else(|| starts.first().copied().filter(|_| starts.len() == 1))
}

/// `text` with the passage replaced by `with`, where [`locate`] finds it.
pub fn replace(text: &str, ask: &PassageAsk, with: &str) -> Option<String> {
    let start = locate(text, ask)?;
    let end = start + ask.passage.len();
    Some(format!("{}{with}{}", &text[..start], &text[end..]))
}

/// The model's answer made a proposal: tags and context it gave back taken out, a line of its
/// own words before the passage taken out, the passage's own spaces at either end kept.
/// Refused when nothing is left or it is too far from the passage's length to be a correction
/// of it; noted when it is much longer or shorter, or unchanged.
pub fn read_answer(
    before: &str,
    passage: &str,
    after: &str,
    answer: &str,
) -> Result<(String, Vec<ProposalNote>), PassageError> {
    let mut notes = Vec::new();
    let mut text = answer.trim().to_string();
    // A fence of its own around the answer.
    if let Some(inner) = text.strip_prefix("```").and_then(|t| t.strip_suffix("```")) {
        let inner = inner.split_once('\n').map_or(inner, |(first, rest)| {
            if first.trim().chars().all(|c| c.is_ascii_alphanumeric()) {
                rest
            } else {
                inner
            }
        });
        text = inner.trim().to_string();
        notes.push(ProposalNote::CommentaryRemoved);
    }
    // The passage's own tags: what lies between them is the answer.
    if let (Some(open), Some(close)) = (text.find("<passage>"), text.rfind("</passage>"))
        && open < close
    {
        let inner = text[open + "<passage>".len()..close].trim().to_string();
        let outside = format!("{}{}", &text[..open], &text[close + "</passage>".len()..]);
        if !outside.trim().is_empty() {
            notes.push(ProposalNote::ContextRemoved);
        }
        text = inner;
    }
    // The context's tags, with what they hold.
    for (open, close) in [("<before>", "</before>"), ("<after>", "</after>")] {
        while let (Some(o), Some(c)) = (text.find(open), text.find(close)) {
            if c < o {
                break;
            }
            text = format!("{}{}", &text[..o], &text[c + close.len()..])
                .trim()
                .to_string();
            if !notes.contains(&ProposalNote::ContextRemoved) {
                notes.push(ProposalNote::ContextRemoved);
            }
        }
    }
    // The context given back without its tags, at either end: long enough to be sure of.
    let (lead, tail) = (before.trim(), after.trim());
    if lead.chars().count() >= 20
        && let Some(rest) = text.strip_prefix(lead)
    {
        text = rest.trim().to_string();
        notes.push(ProposalNote::ContextRemoved);
    }
    if tail.chars().count() >= 20
        && let Some(rest) = text.strip_suffix(tail)
    {
        text = rest.trim().to_string();
        if !notes.contains(&ProposalNote::ContextRemoved) {
            notes.push(ProposalNote::ContextRemoved);
        }
    }
    // A line of the model's own before the passage ("Here is the corrected passage:").
    if let Some((first, rest)) = text.split_once('\n') {
        let first = first.trim();
        let said = first.ends_with(':') && !passage.trim_start().starts_with(first);
        if said && first.chars().count() <= 80 && !rest.trim().is_empty() {
            text = rest.trim().to_string();
            notes.push(ProposalNote::CommentaryRemoved);
        }
    }
    if text.is_empty() {
        return Err(PassageError::Empty);
    }
    let core = passage.trim();
    let (p, a) = (core.chars().count(), text.chars().count());
    if a > p * 2 + 20 || a * 2 + 20 < p {
        return Err(PassageError::TooDifferent {
            passage: p,
            answer: a,
        });
    }
    if a.abs_diff(p) > 10 && a.abs_diff(p) * 4 > p {
        notes.push(ProposalNote::LengthChanged);
    }
    if text == core {
        notes.push(ProposalNote::Unchanged);
    }
    let start = passage.len() - passage.trim_start().len();
    let end = passage.trim_end().len();
    let corrected = format!("{}{text}{}", &passage[..start], &passage[end.max(start)..]);
    Ok((corrected, notes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(before: &str, passage: &str, after: &str, at: u32) -> PassageAsk {
        PassageAsk {
            id: "p1".into(),
            job_id: JobId::new(),
            setup_id: SetupId::new(),
            source_id: SourceId::new(),
            index: 0,
            before: before.into(),
            passage: passage.into(),
            after: after.into(),
            at,
            provider: ProviderId::new("fake"),
            model: "m".into(),
            preset_id: None,
        }
    }

    #[test]
    fn a_passage_too_short_or_too_long_is_refused() {
        assert_eq!(check(&ask("", " a, ", "", 0)), Err(PassageError::TooShort));
        assert_eq!(check(&ask("", "abc", "", 0)), Ok(()));
        let long = "é".repeat(MAX_CHARS + 1);
        assert_eq!(
            check(&ask("", &long, "", 0)),
            Err(PassageError::TooLong(MAX_CHARS + 1))
        );
        let context = "x".repeat(CONTEXT_CHARS + 41);
        assert_eq!(
            check(&ask(&context, "abc", "", 0)),
            Err(PassageError::ContextTooLong)
        );
    }

    #[test]
    fn the_input_marks_the_context_apart_from_the_passage() {
        let text = input("Le Gouverneur", " a recu ", "ce matin");
        assert_eq!(
            text,
            "<before>\nLe Gouverneur\n</before>\n<passage>\n a recu \n</passage>\n<after>\nce matin\n</after>"
        );
    }

    #[test]
    fn the_passage_is_found_at_its_place_or_alone() {
        // “é” is one UTF-16 unit and two bytes: the place is counted as the interface counts.
        let text = "été la fete, puis la fete encore";
        let first = ask("la ", "fete", ",", 7);
        assert_eq!(locate(text, &first), Some(9));
        let second = ask("la ", "fete", " encore", 23);
        assert_eq!(locate(text, &second), Some(text.rfind("fete").unwrap_or(0)));
        // Two identical stretches and a place that is neither: not guessed.
        let both = ask("la ", "fete", "", 2);
        assert_eq!(locate(text, &both), None);
        // Moved, but only once in the text: found.
        let moved = ask("puis la ", "fete", " encore", 0);
        assert!(locate(text, &moved).is_some());
        let gone = ask("la ", "fête", ",", 7);
        assert_eq!(locate(text, &gone), None);
        assert_eq!(
            replace(text, &first, "fête").as_deref(),
            Some("été la fête, puis la fete encore")
        );
    }

    #[test]
    fn an_answer_is_taken_as_it_is_with_the_passage_spaces() -> Result<(), PassageError> {
        let (text, notes) = read_answer("Le ", " Gouvemeur generaI ", " a", "Gouverneur général")?;
        assert_eq!(text, " Gouverneur général ");
        assert!(notes.is_empty());
        let (same, notes) = read_answer("", "Gouverneur", "", "Gouverneur")?;
        assert_eq!(same, "Gouverneur");
        assert_eq!(notes, vec![ProposalNote::Unchanged]);
        Ok(())
    }

    #[test]
    fn tags_context_and_commentary_given_back_are_taken_out() -> Result<(), PassageError> {
        let before = "Le Gouverneur général a reçu ce matin ";
        let after = " de la communauté musulmane de la ville.";
        let passage = "deux cents fideles";
        let tagged = format!(
            "<before>\n{before}\n</before>\n<passage>\ndeux cents fidèles\n</passage>\n<after>{after}</after>"
        );
        let (text, notes) = read_answer(before, passage, after, &tagged)?;
        assert_eq!(text, "deux cents fidèles");
        assert_eq!(notes, vec![ProposalNote::ContextRemoved]);

        let echoed = format!("{}deux cents fidèles{}", before.trim(), after.trim_end());
        let (text, notes) = read_answer(before, passage, after, &echoed)?;
        assert_eq!(text, "deux cents fidèles");
        assert_eq!(notes, vec![ProposalNote::ContextRemoved]);

        let said = "Here is the corrected passage:\ndeux cents fidèles";
        let (text, notes) = read_answer(before, passage, after, said)?;
        assert_eq!(text, "deux cents fidèles");
        assert_eq!(notes, vec![ProposalNote::CommentaryRemoved]);

        let fenced = "```text\ndeux cents fidèles\n```";
        let (text, _) = read_answer(before, passage, after, fenced)?;
        assert_eq!(text, "deux cents fidèles");
        Ok(())
    }

    #[test]
    fn an_empty_or_far_too_different_answer_is_refused() -> Result<(), PassageError> {
        let passage = "deux cents fideles";
        assert_eq!(read_answer("", passage, "", "  "), Err(PassageError::Empty));
        let essay = "Voici une longue explication. ".repeat(4);
        assert!(matches!(
            read_answer("", passage, "", &essay),
            Err(PassageError::TooDifferent { .. })
        ));
        let (_, notes) = read_answer("", passage, "", "deux cents fidèles de la ville")?;
        assert_eq!(notes, vec![ProposalNote::LengthChanged]);
        Ok(())
    }
}
