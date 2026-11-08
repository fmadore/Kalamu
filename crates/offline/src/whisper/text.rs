//! The text Whisper writes, made into what Kalamu keeps: the names and words to spell as its
//! prompt, and its known failures cut back — lines it invents over silence, loops, the subtitle
//! credits it learnt from its training data — before the lines are joined into sentences.

use kalamu_core::Segment;

use super::engine::RawSegment;

/// Longest list of names and words offered as the prompt, before the model's tokenizer has its
/// say: whisper.cpp keeps at most 223 tokens, which this much Latin text stays well within.
const MAX_PROMPT_CHARS: usize = 600;

/// How many times running a word, or a group of words, is repeated before it counts as a loop.
const LOOP_WORD: usize = 6;
const LOOP_GROUP: usize = 4;

// whisper.cpp's own defaults (`no_speech_thold` 0.6, `logprob_thold` -1.0, as in OpenAI's
// reference implementation), which it applies to a whole decoding pass, averaging over every
// token in the window: one confident sentence carries a weak trailing line past them, the
// "Thank you." Whisper invents over room noise. Applied to each line, as Live Translation does,
// they drop that line and nothing else: confident speech has a mean log-probability well above
// -1 however noisy the window is judged to be.
const NO_SPEECH_THRESHOLD: f32 = 0.6;
const LOGPROB_THRESHOLD: f32 = -1.0;

/// The terms offered to the prompt: the researcher's list in order, without the NUL a C string
/// cannot hold, as much of it as [`MAX_PROMPT_CHARS`] allows.
fn offered(terms: &[String]) -> Vec<String> {
    let mut kept = Vec::new();
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
    kept
}

/// The prompt as provenance records it before the model is loaded: the terms offered, joined.
/// [`Prompt::fit`] writes the same text when they all fit the model's budget, as a list of this
/// length does in the scripts researchers' names are mostly written in; when one does not, the
/// log says so (`whisper/mod.rs`).
pub(super) fn initial_prompt(terms: &[String]) -> Option<String> {
    let kept = offered(terms);
    (!kept.is_empty()).then(|| format!("{}.", kept.join(", ")))
}

/// Whether whisper.cpp judged the window silent and was unsure of this line: text it invented.
pub(super) fn is_silence(segment: &RawSegment) -> bool {
    segment.no_speech > NO_SPEECH_THRESHOLD && segment.mean_logprob < LOGPROB_THRESHOLD
}

/// The names and words to spell, as Whisper's prompt: tokens rather than text, so the model's
/// own tokenizer decides how many fit. Ported from Live Translation's `whisper/prompt.rs`.
#[derive(Debug, Default)]
pub(super) struct Prompt {
    /// Given to every window with `FullParams::set_tokens`, which, unlike
    /// `set_initial_prompt`, neither tokenizes again per window nor leaks a C string per call.
    pub tokens: Vec<i32>,
    /// The list as kept, for provenance: the terms that fit, joined as the model reads them.
    pub text: String,
    /// The terms that made it in, in order and compared without case or punctuation.
    terms: Vec<String>,
}

impl Prompt {
    /// The first of `terms`, in the researcher's order and within [`MAX_PROMPT_CHARS`], that
    /// fit `budget` tokens as one list, written as [`initial_prompt`] writes it. whisper.cpp
    /// keeps the *last* tokens of a prompt that is too long; stopping at the first term that
    /// does not fit keeps the top of the list instead.
    pub fn fit(
        terms: &[String],
        budget: usize,
        mut tokenize: impl FnMut(&str) -> Option<Vec<i32>>,
    ) -> Self {
        let mut prompt = Self::default();
        let mut list = String::new();
        for term in offered(terms) {
            let candidate = if list.is_empty() {
                term.clone()
            } else {
                format!("{list}, {term}")
            };
            let text = format!("{candidate}.");
            match tokenize(&text) {
                Some(tokens) if tokens.len() <= budget => {
                    prompt.tokens = tokens;
                    prompt.terms.push(bare(&term));
                    prompt.text = text;
                    list = candidate;
                }
                _ => break,
            }
        }
        prompt
    }

    /// How many of the terms offered made it in.
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Whether a line is the prompt read back rather than speech: two or more of its terms in a
    /// row, in the list's order, and nothing else, as Whisper writes over noise or music it
    /// cannot make out. A term said on its own is kept: that is what the list is for.
    pub fn is_echo(&self, text: &str) -> bool {
        let parts: Vec<String> = text
            .split([',', '、', '，', '،'])
            .map(bare)
            .filter(|part| !part.is_empty())
            .collect();
        parts.len() >= 2
            && self
                .terms
                .windows(parts.len())
                .any(|run| run.iter().eq(parts.iter()))
    }
}

/// A word as loops are compared: without case or the punctuation around it.
pub(super) fn bare(word: &str) -> String {
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

    /// One token per word: enough to count a budget by.
    fn words(text: &str) -> Option<Vec<i32>> {
        Some(
            text.split([' ', ','])
                .filter(|word| !word.is_empty())
                .map(|_| 1)
                .collect(),
        )
    }

    fn terms(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn the_top_of_the_list_is_kept_when_it_does_not_all_fit() {
        let list = terms(&["Samory Touré", "Ouaga\0dougou", "Bobo-Dioulasso", "IWAC"]);
        let prompt = Prompt::fit(&list, 4, words);
        assert_eq!(prompt.text, "Samory Touré, Ouagadougou, Bobo-Dioulasso.");
        assert_eq!(prompt.tokens.len(), 4);
        assert_eq!(Prompt::fit(&list, 100, words).len(), 4);
        assert!(Prompt::fit(&[], 100, words).tokens.is_empty());
        // A term the tokenizer refuses ends the list there.
        let refused = Prompt::fit(&list, 100, |t| (!t.contains("Bobo")).then(|| vec![0]));
        assert_eq!(refused.text, "Samory Touré, Ouagadougou.");
    }

    #[test]
    fn provenance_records_the_prompt_the_model_is_given() {
        let names = terms(&["Samory Touré", "Ouaga\0dougou"]);
        let recorded = initial_prompt(&names);
        assert_eq!(recorded.as_deref(), Some("Samory Touré, Ouagadougou."));
        assert_eq!(Some(Prompt::fit(&names, 223, words).text), recorded);
        assert_eq!(initial_prompt(&[]), None);
        let many: Vec<String> = (0..500).map(|i| format!("Name {i}")).collect();
        assert!(initial_prompt(&many).unwrap().len() <= MAX_PROMPT_CHARS);
    }

    #[test]
    fn the_list_read_back_is_an_echo_and_speech_is_not() {
        let prompt = Prompt::fit(&terms(&["IWAC", "Samory Touré", "Bouaké"]), 100, words);
        assert!(prompt.is_echo(" IWAC, Samory Touré, Bouaké."));
        assert!(prompt.is_echo("samory touré, bouaké"));
        assert!(!prompt.is_echo("IWAC."));
        assert!(!prompt.is_echo("Bouaké, IWAC"));
        assert!(!prompt.is_echo("IWAC, which Samory Touré founded"));
        assert!(!Prompt::default().is_echo("IWAC, Bouaké"));
    }

    #[test]
    fn only_unlikely_text_in_a_window_judged_silent_is_dropped() {
        let judged = |no_speech: f32, mean_logprob: f32| RawSegment {
            text: "Thank you.".into(),
            start_cs: 0,
            end_cs: 100,
            no_speech,
            mean_logprob,
        };
        assert!(is_silence(&judged(0.9, -1.4)));
        assert!(!is_silence(&judged(0.9, -0.2)));
        assert!(!is_silence(&judged(0.2, -1.6)));
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
