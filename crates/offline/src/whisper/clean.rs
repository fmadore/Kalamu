//! Whisper's text tidied: the names to spell as its initial prompt, its loops and the subtitle
//! credits it writes over silence cut back, and its lines joined into sentences.

use kalamu_core::Segment;

/// Longest initial prompt: whisper.cpp keeps at most 224 tokens of it, about this much text.
const MAX_PROMPT_CHARS: usize = 600;
/// How many times running a word, or a group of words, is repeated before it counts as a loop.
const LOOP_WORD: usize = 6;
const LOOP_GROUP: usize = 4;

/// The names and words to spell as Whisper's initial prompt: a list in the researcher's order,
/// as much as whisper.cpp keeps, without the NUL a C string cannot hold.
pub(super) fn initial_prompt(terms: &[String]) -> Option<String> {
    let mut kept: Vec<String> = Vec::new();
    let mut length = 0;
    for term in terms {
        let term: String = term.chars().filter(|c| *c != '\0').collect();
        let term = term.trim();
        if term.is_empty() {
            continue;
        }
        length += term.len() + 2;
        if length > MAX_PROMPT_CHARS {
            break;
        }
        kept.push(term.to_owned());
    }
    (!kept.is_empty()).then(|| format!("{}.", kept.join(", ")))
}

/// A word as loops are compared: without case or the punctuation around it.
fn bare(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

/// The text with a word said over and over, or a group of words, kept once: Whisper's loops
/// on noise or silence. Words repeated a few times, as people do, are left.
fn collapse_repeats(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let keys: Vec<String> = words.iter().map(|w| bare(w)).collect();
    let mut out: Vec<&str> = Vec::new();
    let mut i = 0;
    'words: while i < words.len() {
        for n in 1..=8 {
            let needed = if n == 1 { LOOP_WORD } else { LOOP_GROUP };
            if i + n * needed > words.len() || keys[i..i + n].iter().all(String::is_empty) {
                continue;
            }
            let mut times = 1;
            while i + (times + 1) * n <= words.len()
                && keys[i + times * n..i + (times + 1) * n] == keys[i..i + n]
            {
                times += 1;
            }
            if times >= needed {
                out.extend(&words[i..i + n]);
                i += times * n;
                continue 'words;
            }
        }
        out.push(words[i]);
        i += 1;
    }
    out.join(" ")
}

/// Subtitle credits Whisper learnt from its training data and writes over silence or music.
fn is_credit(text: &str) -> bool {
    let text = text.to_lowercase();
    text.contains("amara.org")
        || ["sous-titres réalisés", "sous-titrage st", "subtitles by"]
            .iter()
            .any(|credit| text.starts_with(credit))
}

/// The lines without Whisper's loops: credits dropped, words said over and over kept once, and
/// three or more lines in a row saying the same kept as one, lasting as long as they did.
pub(super) fn without_loops(segments: Vec<Segment>) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    let mut run = 0;
    for mut segment in segments {
        if is_credit(&segment.text) {
            continue;
        }
        segment.text = collapse_repeats(&segment.text);
        if segment.text.is_empty() {
            continue;
        }
        let same = out.last().is_some_and(|last| {
            let key = |t: &str| t.split_whitespace().map(bare).collect::<Vec<_>>();
            key(&last.text) == key(&segment.text)
        });
        run = if same { run + 1 } else { 0 };
        if run >= 2 {
            // A third line saying the same: the second is dropped, the first lasts until now.
            if run == 2 {
                out.pop();
            }
            if let Some(first) = out.last_mut() {
                first.end_ms = first.end_ms.max(segment.end_ms);
            }
            continue;
        }
        out.push(segment);
    }
    out
}

/// The lines as running text: one sentence a line, as the lines of subtitle length are cut
/// inside sentences.
pub(super) fn sentences(segments: &[Segment]) -> String {
    let mut out = String::new();
    for segment in segments {
        let text = segment.text.trim();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push(' ');
        }
        out.push_str(text);
        let end = text.trim_end_matches(|c: char| c.is_whitespace() || "\"'»”’)]".contains(c));
        if end.ends_with(['.', '?', '!', '…']) {
            out.push('\n');
        }
    }
    out.trim_end().to_owned()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn names_to_spell_become_a_prompt_whisper_keeps_whole() {
        let names = vec!["Samory Touré".to_owned(), "Ouaga\0dougou".to_owned()];
        assert_eq!(
            initial_prompt(&names).as_deref(),
            Some("Samory Touré, Ouagadougou.")
        );
        assert_eq!(initial_prompt(&[]), None);
        let many: Vec<String> = (0..500).map(|i| format!("Name {i}")).collect();
        assert!(initial_prompt(&many).unwrap().len() <= MAX_PROMPT_CHARS);
    }

    fn line(start_ms: u64, end_ms: u64, text: &str) -> Segment {
        Segment {
            start_ms,
            end_ms,
            speaker: None,
            text: text.to_owned(),
        }
    }

    #[test]
    fn loops_are_cut_back_and_what_people_say_twice_is_kept() {
        assert_eq!(
            collapse_repeats("Il a dit merci merci merci merci merci merci merci."),
            "Il a dit merci"
        );
        assert_eq!(
            collapse_repeats("on y va, on y va, on y va, on y va, on y va et puis"),
            "on y va, et puis"
        );
        assert_eq!(
            collapse_repeats("non, non, non, je ne crois pas"),
            "non, non, non, je ne crois pas"
        );
        let lines = vec![
            line(0, 1_000, "Bonjour."),
            line(1_000, 2_000, "Oui."),
            line(2_000, 3_000, "Oui."),
            line(3_000, 4_000, "Merci beaucoup."),
            line(4_000, 5_000, "Merci beaucoup."),
            line(5_000, 6_000, "merci beaucoup"),
            line(6_000, 9_000, "Merci beaucoup."),
            line(
                9_000,
                10_000,
                "Sous-titres réalisés par la communauté d'Amara.org",
            ),
            line(10_000, 11_000, "Au revoir."),
        ];
        let kept = without_loops(lines);
        let texts: Vec<_> = kept.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(
            texts,
            ["Bonjour.", "Oui.", "Oui.", "Merci beaucoup.", "Au revoir."]
        );
        assert_eq!((kept[3].start_ms, kept[3].end_ms), (3_000, 9_000));
    }

    #[test]
    fn the_text_runs_one_sentence_a_line() {
        let lines = [
            line(0, 1_000, "Nous sommes arrivés à Bobo-Dioulasso"),
            line(1_000, 2_000, "en 1957, avec mon père."),
            line(2_000, 3_000, "Vous vous souvenez?"),
            line(3_000, 4_000, "« Oui. »"),
        ];
        assert_eq!(
            sentences(&lines),
            "Nous sommes arrivés à Bobo-Dioulasso en 1957, avec mon père.\nVous vous souvenez?\n« Oui. »"
        );
    }
}
