use kalamu_core::{ExportDocument, ExportError, ExportFormat, Exporter, Task};

use crate::timeline::{cues, timeline};

pub struct SrtExporter;
pub struct VttExporter;

fn stamp(ms: u64, separator: char) -> String {
    let (h, m, s, rest) = (ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, ms % 1000);
    format!("{h:02}:{m:02}:{s:02}{separator}{rest:03}")
}

fn no_timestamps() -> ExportError {
    ExportError::Write {
        format: "subtitle",
        message: "the transcription has no timestamps; choose a model or prompt that gives them"
            .into(),
    }
}

impl Exporter for SrtExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Srt
    }

    fn supports(&self, task: Task) -> bool {
        task == Task::AudioTranscription
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let cues = cues(&timeline(doc), doc.options.speaker_labels);
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
        task == Task::AudioTranscription
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let cues = cues(&timeline(doc), false);
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
