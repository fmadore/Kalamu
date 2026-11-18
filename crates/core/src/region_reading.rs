//! Reading one region of a page again with a model (#84, "Reading a region again"): the box is
//! cut from the original at full resolution, sent alone as a text extraction of an image
//! (the preset chosen, or the default one; the job's languages and line breaks), its answer
//! cached and costed, and offered as a proposal. Only the researcher's acceptance writes it:
//! as the text of a region they drew, or in place of the words an engine's region holds in the
//! page's text, through their corrections. The page's own reading is never replaced.
//!
//! The box is in fractions of the piece as the review shows it, as every region is
//! ([`crate::regions`]).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::proposal::{Lifecycle, Proposal};
use crate::{
    Correction, DrawnRegion, JobId, PageHalf, PromptSnapshot, ProposalStatus, ProviderId,
    RequestSetting, SetupId, SourceId, Timestamp,
};

/// Where an accepted reading goes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum ReadingTarget {
    /// A region the researcher drew (the ask's `region_id` is its id): accepting writes the
    /// reading as its text.
    Region,
    /// An engine's region whose words have a certain place in the text shown
    /// ([`crate::ShownRegion::span`]): accepting puts the reading in place of that stretch in
    /// the page's correction, as accepting a passage does.
    Text {
        setup_id: SetupId,
        /// The piece whose text is shown.
        index: u32,
        /// The stretch, in UTF-16 code units as the interface counts them; `end` is past it.
        start: u32,
        end: u32,
        /// What the text shows there now: accepting is refused once it is something else.
        was: String,
    },
    /// An engine's region with no certain place in the text: the reading can be read and
    /// copied, not written anywhere.
    Nowhere,
}

/// What the researcher asks: one region of a page, read again by the model and prompt chosen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RegionAsk {
    /// Chosen by the interface, so that the request can be cancelled while it waits.
    pub id: String,
    pub job_id: JobId,
    pub source_id: SourceId,
    /// The page's number in the source, from 1.
    pub page: u32,
    /// The half of a split spread the box is on; its fractions are of that half.
    pub half: Option<PageHalf>,
    /// Fractions of the piece as shown, from its top-left corner.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// The region read: a drawn region's id, or an engine's (`r3_l2`).
    pub region_id: String,
    pub target: ReadingTarget,
    pub provider: ProviderId,
    pub model: String,
    /// A text extraction preset; the default one when none is given.
    pub preset_id: Option<String>,
}

/// A region read again, as the store keeps it: the request, the prompt as sent, the answer,
/// what it cost and what the researcher decided. Recorded before the call and after it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RegionReading {
    pub ask: RegionAsk,
    pub status: ProposalStatus,
    /// The text extraction preset's prompt as sent.
    pub prompt: PromptSnapshot,
    /// What the request carried besides the prompt.
    #[serde(default)]
    pub request: Vec<RequestSetting>,
    /// The key of the stored answer: the same for the same picture, model and prompt. Empty when
    /// the reading was stopped before its picture was cut.
    pub cache_key: String,
    /// The model's reading, as a text extraction stores it.
    pub answer: Option<String>,
    /// What this request cost; none when the answer came from the cache or has no price.
    pub cost_usd: Option<f64>,
    /// The answer was one already stored: asking cost nothing.
    #[serde(default)]
    pub cached: bool,
    pub error: Option<String>,
    /// The text written: the answer, or the researcher's edit of it.
    pub accepted: Option<String>,
    pub asked_at: Timestamp,
    pub answered_at: Option<Timestamp>,
    pub decided_at: Option<Timestamp>,
}

impl Proposal for RegionReading {
    fn status(&self) -> ProposalStatus {
        self.status
    }

    fn lifecycle(&mut self) -> Lifecycle<'_> {
        Lifecycle {
            status: &mut self.status,
            request: &mut self.request,
            cost_usd: &mut self.cost_usd,
            cached: &mut self.cached,
            error: &mut self.error,
            accepted: &mut self.accepted,
            answered_at: &mut self.answered_at,
            decided_at: &mut self.decided_at,
        }
    }
}

/// What accepting a reading writes, or, given back by accepting, what *Undo* puts back: the
/// page's correction, or the researcher's region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[ts(export)]
pub enum ReadingEffect {
    Correction(Correction),
    Region(DrawnRegion),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegionReadingError {
    /// The words of the text where the region's reading would go are no longer as they were.
    #[error("The text where this reading would go has changed")]
    Changed,
    /// An engine's region whose words have no certain place in the text.
    #[error("This reading has no certain place in the page's text: copy it instead")]
    Nowhere,
    /// The model found no text in the region.
    #[error("The model found no text in this region")]
    Empty,
    /// A model that does not read a page's text.
    #[error("{0} cannot read the text of a page")]
    CannotRead(String),
    /// The reading is not waiting for a decision (decided already, failed or cancelled).
    #[error("This reading is no longer waiting for a decision")]
    NotWaiting,
}

impl RegionReadingError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Changed => "region_reading.changed",
            Self::Nowhere => "region_reading.nowhere",
            Self::Empty => "region_reading.empty",
            Self::CannotRead(_) => "region_reading.cannot_read",
            Self::NotWaiting => "region_reading.not_waiting",
        }
    }
}

impl RegionAsk {
    /// The ask's box as a region draft, for the checks a drawn region passes
    /// ([`crate::regions::check`]).
    pub fn as_draft(&self) -> crate::RegionDraft {
        crate::RegionDraft {
            id: None,
            job_id: self.job_id,
            source_id: self.source_id,
            page: self.page,
            half: self.half,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            text: String::new(),
            adjusts: None,
        }
    }
}

/// The bytes of `text` that UTF-16 offset `at` falls on; none past its end or inside a
/// character.
fn byte_at(text: &str, at: u32) -> Option<usize> {
    let mut units = 0u32;
    for (byte, c) in text.char_indices() {
        if units == at {
            return Some(byte);
        }
        units += c.len_utf16() as u32;
    }
    (units == at).then_some(text.len())
}

/// Where the stretch `was` stands in `text`, in bytes: at `start` (UTF-16 units) when it is
/// there, or else its only place in the text. `None` when the text there changed.
pub fn locate(text: &str, start: u32, end: u32, was: &str) -> Option<(usize, usize)> {
    if was.is_empty() || end < start {
        return None;
    }
    if let (Some(s), Some(e)) = (byte_at(text, start), byte_at(text, end))
        && text.get(s..e) == Some(was)
    {
        return Some((s, e));
    }
    let mut places = text.match_indices(was).map(|(i, _)| i);
    match (places.next(), places.next()) {
        (Some(only), None) => Some((only, only + was.len())),
        _ => None,
    }
}

/// `text` with the target's stretch replaced by `with`, where [`locate`] finds it; `None` for
/// a target that is not a stretch of the text, or a text that changed there.
pub fn replace(text: &str, target: &ReadingTarget, with: &str) -> Option<String> {
    let ReadingTarget::Text {
        start, end, was, ..
    } = target
    else {
        return None;
    };
    let (s, e) = locate(text, *start, *end, was)?;
    Some(format!("{}{with}{}", &text[..s], &text[e..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_target(start: u32, end: u32, was: &str) -> ReadingTarget {
        ReadingTarget::Text {
            setup_id: SetupId::new(),
            index: 0,
            start,
            end,
            was: was.into(),
        }
    }

    #[test]
    fn a_stretch_is_replaced_where_the_interface_counted_it() {
        // "é" is one UTF-16 unit and two bytes; "𝔄" two units and four bytes.
        let text = "été 𝔄 la fete, puis la fete encore";
        let at = text[..text.find("fete").unwrap_or(0)]
            .encode_utf16()
            .count() as u32;
        let target = text_target(at, at + 4, "fete");
        assert_eq!(
            replace(text, &target, "fête").as_deref(),
            Some("été 𝔄 la fête, puis la fete encore")
        );
        // The second of two identical stretches, by its place.
        let second = text[..text.rfind("fete").unwrap_or(0)]
            .encode_utf16()
            .count() as u32;
        let target = text_target(second, second + 4, "fete");
        assert_eq!(
            replace(text, &target, "fête").as_deref(),
            Some("été 𝔄 la fete, puis la fête encore")
        );
    }

    #[test]
    fn a_stretch_moved_is_found_alone_and_a_changed_one_refused() {
        let text = "Le Gouverneur général a reçu deux cents fidèles.";
        // Moved by an edit before it, and only once in the text: found.
        let moved = text_target(3, 13, "deux cents");
        assert_eq!(
            replace(text, &moved, "200").as_deref(),
            Some("Le Gouverneur général a reçu 200 fidèles.")
        );
        // No longer there: refused.
        let gone = text_target(29, 39, "trois cents");
        assert_eq!(replace(text, &gone, "300"), None);
        // Twice in the text and not at its place: not guessed.
        let twice = "la fete, la fete";
        assert_eq!(replace(twice, &text_target(1, 5, "fete"), "x"), None);
        // Nothing to replace, or a place that is not a stretch.
        assert_eq!(replace(text, &text_target(3, 3, ""), "x"), None);
        assert_eq!(replace(text, &text_target(9, 3, "Gouv"), "x"), None);
        assert_eq!(replace(text, &ReadingTarget::Nowhere, "x"), None);
        assert_eq!(replace(text, &ReadingTarget::Region, "x"), None);
    }

    #[test]
    fn an_offset_inside_a_character_is_no_place() {
        let text = "𝔄bc";
        assert_eq!(byte_at(text, 1), None);
        assert_eq!(byte_at(text, 2), Some(4));
        assert_eq!(byte_at(text, 4), Some(text.len()));
        assert_eq!(byte_at(text, 5), None);
    }

    #[test]
    fn targets_and_effects_are_sent_as_the_interface_reads_them() -> Result<(), serde_json::Error> {
        let json = serde_json::to_value(ReadingTarget::Nowhere)?;
        assert_eq!(json, serde_json::json!({ "kind": "nowhere" }));
        let json = serde_json::to_value(text_target(1, 2, "a"))?;
        assert_eq!(json["kind"], "text");
        assert_eq!(json["was"], "a");
        let errors = [
            RegionReadingError::Changed,
            RegionReadingError::Nowhere,
            RegionReadingError::Empty,
            RegionReadingError::CannotRead("m".into()),
            RegionReadingError::NotWaiting,
        ];
        assert!(
            errors
                .iter()
                .all(|e| e.code().starts_with("region_reading."))
        );
        Ok(())
    }
}
