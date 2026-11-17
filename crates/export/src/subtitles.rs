use kalamu_core::{ExportDocument, ExportError, ExportFormat, Exporter, Segment, Task};

use crate::timeline::{cues, timeline};

pub struct SrtExporter;
pub struct VttExporter;

fn stamp(ms: u64, separator: char) -> String {
    let (h, m, s, rest) = (ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, ms % 1000);
    format!("{h:02}:{m:02}:{s:02}{separator}{rest:03}")
}

fn refused(message: String) -> ExportError {
    ExportError::Write {
        format: "subtitle",
        message,
    }
}

/// The document's timed lines, on the recording's clock. A transcript's come from its
/// segments or the times its text holds. A translation's are the lines of the transcript it
/// translated, each with the time and speaker of the line it translates (`kalamu_core::aligned`):
/// a translation of any other text has none, and one whose correction joined or split lines
/// has lost them, which is said rather than written with times that no longer fit.
fn lines(doc: &ExportDocument) -> Result<Vec<Segment>, ExportError> {
    if doc.task != Task::Translation {
        return Ok(timeline(doc));
    }
    let changed = &doc.provenance.lines_changed;
    if !changed.is_empty() {
        let which = changed
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let (passage, has) = if changed.len() == 1 {
            ("passage", "has")
        } else {
            ("passages", "have")
        };
        return Err(refused(format!(
            "the correction of {passage} {which} no longer {has} one paragraph per line of the \
             transcript, so the times of its lines are lost; correct it back to one paragraph per \
             line to write subtitles"
        )));
    }
    let lines: Vec<Segment> = doc
        .parts
        .iter()
        .flat_map(|p| p.segments.iter().cloned())
        .collect();
    if lines.is_empty() {
        return Err(refused(
            "this translation has no times: subtitles are written only from a transcript sent on \
             with its times, translated line by line"
                .into(),
        ));
    }
    Ok(lines)
}

fn no_timestamps() -> ExportError {
    refused("the transcription has no timestamps; choose a model or prompt that gives them".into())
}

impl Exporter for SrtExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Srt
    }

    fn supports(&self, task: Task) -> bool {
        matches!(task, Task::AudioTranscription | Task::Translation)
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let cues = cues(&lines(doc)?, doc.options.speaker_labels);
        if cues.is_empty() {
            return Err(no_timestamps());
        }
        let mut out = String::new();
        for (i, cue) in cues.iter().enumerate() {
            out.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
                stamp(cue.start_ms, ','),
                stamp(cue.end_ms, ','),
                cue.text
            ));
        }
        Ok(out.into_bytes())
    }
}

impl Exporter for VttExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Vtt
    }

    fn supports(&self, task: Task) -> bool {
        matches!(task, Task::AudioTranscription | Task::Translation)
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let cues = cues(&lines(doc)?, false);
        if cues.is_empty() {
            return Err(no_timestamps());
        }
        let mut out = String::from("WEBVTT\n\n");
        for cue in &cues {
            let text = match (&cue.speaker, doc.options.speaker_labels) {
                // WebVTT's own voice tag, which players can style or hide.
                (Some(s), true) => format!("<v {s}>{}", cue.text),
                _ => cue.text.clone(),
            };
            out.push_str(&format!(
                "{} --> {}\n{}\n\n",
                stamp(cue.start_ms, '.'),
                stamp(cue.end_ms, '.'),
                text
            ));
        }
        Ok(out.into_bytes())
    }
}
