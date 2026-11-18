//! What the embedding program is told, the file of texts it reads, and the vectors read from
//! what it prints back.

use kalamu_core::{EmbedRole, ProviderError};

use super::CONTEXT;

/// Written between two texts in the file the program reads, which splits it there
/// (`--embd-separator`): its default, a new line, would cut a text of several lines into as
/// many texts. Removed from the texts themselves, so it never splits one.
pub(super) const SEPARATOR: &str = "<#kalamu-text#>";

/// A text as the model reads it: with the task's prefix EmbeddingGemma 2 was trained with, a
/// question as a search query, a passage as a document without a title.
pub(super) fn prompt(role: EmbedRole, text: &str) -> String {
    let text = clean(text);
    match role {
        EmbedRole::Query => format!("task: search result | query: {text}"),
        EmbedRole::Passage => format!("title: none | text: {text}"),
    }
}

/// The text as it can be read from the file: line breaks as `\n`, the separator and control
/// characters (but line breaks and tabs) each made a space. The program reads the file as text,
/// which on Windows ends it at a Ctrl-Z; the rest (quotes, backslashes, any script, emoji)
/// passes as it is, since the program is told not to read escapes (`--no-escape`).
fn clean(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace(SEPARATOR, " ")
        .chars()
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\t' {
                ' '
            } else {
                c
            }
        })
        .collect()
}

/// The file of texts: the prompts one after another, the separator between them.
pub(super) fn texts_file(prompts: &[String]) -> String {
    prompts.join(SEPARATOR)
}

/// The program's arguments: the model, the texts from a file, the vectors printed as one JSON
/// array, each of length 1, the model's own mean pooling, and a context that holds the longest
/// text the limits let through (the batch is the context, as the program wants, so each text is
/// read in one pass); on a Mac every layer on the graphics processor with Metal, which the
/// processor build on Windows ignores.
pub(super) fn arguments(model: &str, texts: &str, threads: i32) -> Vec<String> {
    let context = CONTEXT.to_string();
    [
        "-m",
        model,
        "-f",
        texts,
        "--embd-separator",
        SEPARATOR,
        "--embd-output-format",
        "array",
        "--embd-normalize",
        "2",
        "--pooling",
        "mean",
        "--no-escape",
        "-c",
        &context,
        "-b",
        &context,
        "-ub",
        &context,
        "-t",
        &threads.to_string(),
        "-ngl",
        "99",
        "--no-warmup",
    ]
    .iter()
    .map(|a| (*a).to_owned())
    .collect()
}

/// The vectors the program printed (`[[0.0123,…],[…]]`), whatever it wrote around them.
pub(super) fn vectors(printed: &str) -> Result<Vec<Vec<f32>>, ProviderError> {
    let unreadable = |why: String| {
        ProviderError::Other(format!(
            "The offline embedding model's answer could not be read: {why}"
        ))
    };
    let (Some(start), Some(end)) = (printed.find('['), printed.rfind(']')) else {
        return Err(unreadable("it printed no vectors".into()));
    };
    if end < start {
        return Err(unreadable("it printed no vectors".into()));
    }
    serde_json::from_str(&printed[start..=end]).map_err(|e| unreadable(e.to_string()))
}

/// The vectors checked against what was asked, cut to `dimensions` and each made of length 1:
/// one for each of `count` texts, each of the model's own length, every number finite.
/// EmbeddingGemma 2 was trained so that its first dimensions carry the most (Matryoshka); a cut
/// vector is no longer of length 1, so it is normalised after the cut.
pub(super) fn finish(
    vectors: Vec<Vec<f32>>,
    count: usize,
    native: usize,
    dimensions: usize,
) -> Result<Vec<Vec<f32>>, ProviderError> {
    if vectors.len() != count {
        return Err(ProviderError::Other(format!(
            "The offline embedding model gave {} vectors for {count} texts",
            vectors.len()
        )));
    }
    vectors
        .into_iter()
        .map(|mut vector| {
            if vector.len() != native {
                return Err(ProviderError::Other(format!(
                    "The offline embedding model gave a vector of {} numbers, not {native}",
                    vector.len()
                )));
            }
            vector.truncate(dimensions);
            let norm = vector
                .iter()
                .map(|x| f64::from(*x) * f64::from(*x))
                .sum::<f64>()
                .sqrt();
            if !norm.is_finite() || norm == 0.0 {
                return Err(ProviderError::Other(
                    "The offline embedding model gave a vector that cannot be used".into(),
                ));
            }
            for x in &mut vector {
                // Back to single precision after dividing in double.
                #[allow(clippy::cast_possible_truncation)]
                let unit = (f64::from(*x) / norm) as f32;
                *x = unit;
            }
            Ok(vector)
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn each_role_has_its_prefix() {
        assert_eq!(
            prompt(EmbedRole::Query, "le chemin de fer Dakar-Niger"),
            "task: search result | query: le chemin de fer Dakar-Niger"
        );
        assert_eq!(
            prompt(EmbedRole::Passage, "Les cheminots sont en grève."),
            "title: none | text: Les cheminots sont en grève."
        );
    }

    #[test]
    fn a_text_keeps_its_lines_quotes_scripts_and_emoji() {
        let text = "« Il a dit : \"non\" »\nسلام عليكم\t😀 C:\\dossier\\n";
        assert_eq!(clean(text), text);
        // Windows line ends and the separator do not survive; nor a Ctrl-Z.
        assert_eq!(clean("a\r\nb\rc<#kalamu-text#>d\u{1a}e"), "a\nb\nc d e");
        let file = texts_file(&["un".into(), "deux\ntrois".into()]);
        assert_eq!(
            file.split(SEPARATOR).collect::<Vec<_>>(),
            ["un", "deux\ntrois"]
        );
    }

    #[test]
    fn the_arguments_read_a_file_and_print_unit_vectors_as_json() {
        let args = arguments("m.gguf", "texts.txt", 6);
        let pair = |flag: &str| {
            let i = args.iter().position(|a| a == flag).unwrap();
            args[i + 1].clone()
        };
        assert_eq!(pair("-m"), "m.gguf");
        assert_eq!(pair("-f"), "texts.txt");
        assert_eq!(pair("--embd-separator"), SEPARATOR);
        assert_eq!(pair("--embd-output-format"), "array");
        assert_eq!(pair("--embd-normalize"), "2");
        assert_eq!(pair("--pooling"), "mean");
        assert_eq!(pair("-c"), CONTEXT.to_string());
        assert_eq!(pair("-b"), CONTEXT.to_string());
        assert_eq!(pair("-ub"), CONTEXT.to_string());
        assert_eq!(pair("-t"), "6");
        assert!(args.iter().any(|a| a == "--no-escape"));
    }

    #[test]
    fn the_vectors_are_read_from_what_the_program_printed() {
        assert_eq!(
            vectors("\n[[0.6000000,0.8000000],[1.0000000,0.0000000]]\n\n").unwrap(),
            [vec![0.6, 0.8], vec![1.0, 0.0]]
        );
        assert!(vectors("").is_err());
        assert!(vectors("] nothing [").is_err());
        // What llama.cpp prints for a number that is not one.
        assert!(vectors("[[nan,0.1]]").is_err());
    }

    #[test]
    fn vectors_are_counted_measured_cut_and_normalised() {
        let unit = finish(vec![vec![3.0, 4.0, 0.0, 0.0]], 1, 4, 4).unwrap();
        assert_eq!(unit, [vec![0.6, 0.8, 0.0, 0.0]]);
        // Cut to two numbers, then of length 1 again.
        let cut = finish(vec![vec![0.0, 2.0, 5.0, 5.0]], 1, 4, 2).unwrap();
        assert_eq!(cut, [vec![0.0, 1.0]]);
        let norm: f32 = finish(vec![vec![0.1, 0.2, 0.3, 0.4]], 1, 4, 4).unwrap()[0]
            .iter()
            .map(|x| x * x)
            .sum();
        assert!((norm - 1.0).abs() < 1e-6, "{norm}");
        assert!(matches!(
            finish(vec![vec![1.0; 4]], 2, 4, 4),
            Err(ProviderError::Other(m)) if m.contains("1 vectors for 2 texts")
        ));
        assert!(matches!(
            finish(vec![vec![1.0; 3]], 1, 4, 4),
            Err(ProviderError::Other(m)) if m.contains("3 numbers, not 4")
        ));
        assert!(finish(vec![vec![0.0; 4]], 1, 4, 4).is_err());
        assert!(finish(vec![vec![f32::INFINITY, 0.0, 0.0, 0.0]], 1, 4, 4).is_err());
    }
}
