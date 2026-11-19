//! Reading the `[hh:mm:ss]` markers a model writes into a transcription's text into timed
//! segments: a shared reading of stored results, used by the review and by the exports.

use crate::Segment;

/// The timed lines of one piece whose model wrote `[hh:mm:ss]` markers into its text, each
/// ending where the next begins, for the review.
pub fn timed_lines(text: &str) -> Vec<Segment> {
    let mut out = segments_from_text(text, 0);
    close_ends(&mut out);
    out
}

/// Closes each segment at the next one's start when the model gave no end.
pub fn close_ends(segments: &mut [Segment]) {
    for i in 0..segments.len() {
        if segments[i].end_ms <= segments[i].start_ms {
            let next = segments.get(i + 1).map(|s| s.start_ms);
            segments[i].end_ms = next
                .filter(|&n| n > segments[i].start_ms)
                .unwrap_or(segments[i].start_ms + 4_000);
        }
    }
}

/// Reads lines like `[00:01:05] Speaker 2: text`. Lines without a timestamp continue the
/// previous segment. Times are shifted by `offset_ms`.
pub fn segments_from_text(text: &str, offset_ms: u64) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((ms, rest)) = parse_stamp(line) {
            let (speaker, body) = split_speaker(rest);
            out.push(Segment {
                start_ms: offset_ms + ms,
                end_ms: 0,
                speaker,
                text: body.to_string(),
            });
        } else if let Some(last) = out.last_mut() {
            last.text.push(' ');
            last.text.push_str(line);
        }
    }
    out
}

pub(crate) fn parse_stamp(line: &str) -> Option<(u64, &str)> {
    let rest = line.strip_prefix('[')?;
    let (stamp, rest) = rest.split_once(']')?;
    let parts: Vec<u64> = stamp
        .split(':')
        .map(|p| p.trim().parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    let secs = match parts.as_slice() {
        [h, m, s] => h * 3600 + m * 60 + s,
        [m, s] => m * 60 + s,
        _ => return None,
    };
    Some((secs * 1000, rest.trim_start()))
}

fn split_speaker(rest: &str) -> (Option<String>, &str) {
    if let Some((label, body)) = rest.split_once(':') {
        let label = label.trim();
        let looks_like_label = !label.is_empty()
            && label.len() <= 40
            && label.split_whitespace().count() <= 4
            && !label.contains('[');
        if looks_like_label {
            return (Some(label.to_string()), body.trim_start());
        }
    }
    (None, rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_in_text_become_segments() {
        let segs = segments_from_text("[00:01:05] Speaker 2: Oui.\ncontinued\n[01:10] Hm.", 1000);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].start_ms, 66_000);
        assert_eq!(segs[0].speaker.as_deref(), Some("Speaker 2"));
        assert_eq!(segs[0].text, "Oui. continued");
        assert_eq!(segs[1].start_ms, 71_000);
        assert_eq!(segs[1].speaker, None);
    }

    #[test]
    fn each_timed_line_ends_where_the_next_begins() {
        let lines = timed_lines("[00:00:02] A: Bonjour.\n[00:00:07] B: Oui.");
        assert_eq!(lines.len(), 2);
        assert_eq!((lines[0].start_ms, lines[0].end_ms), (2_000, 7_000));
        assert_eq!((lines[1].start_ms, lines[1].end_ms), (7_000, 11_000));
    }

    #[test]
    fn text_without_markers_has_no_timed_lines() {
        assert!(timed_lines("Pas de minutage ici.").is_empty());
    }
}
