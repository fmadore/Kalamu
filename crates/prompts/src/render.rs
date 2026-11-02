//! Filling a preset's variables and describing an analysis's fields for the model: what each
//! request's instructions say, whatever preset they come from.

use kalamu_core::{AnalysisSchema, FieldKind, LanguageChoice, LineBreaks};

/// Values for a preset's variables.
#[derive(Debug, Clone, Default)]
pub struct RenderContext {
    pub languages: LanguageChoice,
    /// English names for the chosen languages, in order (`French`, `Mooré`).
    pub language_names: Vec<String>,
    pub speakers: Option<u32>,
    /// The researcher's choice for a text extraction's lines; it overrides the preset's own.
    pub line_breaks: Option<LineBreaks>,
    /// The languages are the ones the model writes its answers in (an analysis, a table of
    /// contents), not the source's.
    pub answers: bool,
}

/// Appended when the researcher chose how lines are kept. Stated as overriding the preset, whose
/// own instructions may say the opposite (the defaults write running text unless told
/// otherwise), so one choice works with every preset, the user's own included.
const AS_ON_PAGE: &str = "\n\n## Line breaks\n\nKeep the lines of the source: start a new line \
wherever a line ends on the page, keep end-of-line hyphens exactly as they are, and do not join \
lines or rejoin words split across lines. Separate paragraphs with a blank line. This overrides \
any instruction above about joining lines or removing hyphens.";

const RUNNING_TEXT: &str = "\n\n## Line breaks\n\nWrite running text: join the lines of each \
paragraph, rejoin a word split across two lines and drop its line-break hyphen, but keep real \
compound hyphens (as in arc-en-ciel or peut-être). Separate paragraphs with a blank line. This \
overrides any instruction above about keeping line breaks or end-of-line hyphens.";

/// `{languages}` for answers left in the source's language. Said outright, because a model
/// reading English instructions drifts into English on some pages and not on others.
const ANSWER_LANGUAGE_AUTO: &str = "the language the source is written in (the one most of it \
is in, when it mixes several), even though these instructions are in English";

/// Replaces `{languages}` and `{speakers}`. When the user chose languages but the template does
/// not mention them, a short note is appended so the choice still reaches the model; so is the
/// choice of line breaks.
pub fn render(template: &str, ctx: &RenderContext) -> String {
    let languages = match (ctx.language_names.is_empty(), ctx.answers) {
        (true, true) => ANSWER_LANGUAGE_AUTO.to_string(),
        (true, false) => "the language(s) of the source, detected automatically".to_string(),
        (false, _) => ctx.language_names.join(", "),
    };
    let speakers = ctx
        .speakers
        .map(|n| n.to_string())
        .unwrap_or_else(|| "an unknown number of".into());
    let mut out = template
        .replace("{languages}", &languages)
        .replace("{speakers}", &speakers);
    let explicit = matches!(ctx.languages, LanguageChoice::Explicit(_));
    if explicit && !ctx.language_names.is_empty() && !template.contains("{languages}") {
        out.push_str(&if ctx.answers {
            format!(
                "\n\n## Language\n\nWrite your answers in {languages}, whatever the language of \
                 the source; keep names as the source spells them."
            )
        } else {
            format!("\n\n## Language\n\nThe source is in {languages}.")
        });
    }
    match ctx.line_breaks {
        Some(LineBreaks::AsOnPage) => out.push_str(AS_ON_PAGE),
        Some(LineBreaks::RunningText) => out.push_str(RUNNING_TEXT),
        None => {}
    }
    out
}

/// An analysis's fields in words, appended to its instructions: each field's name in the answer,
/// what kind of answer it takes, and what belongs in it.
pub fn describe(schema: &AnalysisSchema) -> String {
    let mut out = String::from(
        "\n\n## What to find\n\nAnswer with exactly these fields, filled from the document:\n",
    );
    describe_fields(&schema.fields, "", &mut out);
    out.push_str(
        "\n\nWrite each name or term in a list as the document spells it, and leave a list \
         empty when the document has nothing for it. Choose only from the values given, spelled \
         exactly as listed.",
    );
    out
}

/// One numbered line per field; an entries field's own fields indented under it.
fn describe_fields(fields: &[kalamu_core::Field], indent: &str, out: &mut String) {
    for (i, field) in fields.iter().enumerate() {
        let kind = match &field.kind {
            FieldKind::Entries { .. } => {
                "a list of entries, each with the fields below".to_string()
            }
            FieldKind::Text => "text".to_string(),
            FieldKind::List { max: Some(max) } => {
                format!("a list of names or terms, at most {max}, most important first")
            }
            FieldKind::List { max: None } => {
                "a list of names or terms, most important first".to_string()
            }
            FieldKind::Choice { values } => format!("exactly one of: {}", values.join(", ")),
            FieldKind::Choices { values } => {
                format!("any of: {}; none if none applies", values.join(", "))
            }
        };
        out.push_str(&format!(
            "\n{indent}{}. {} (`{}`, {kind})",
            i + 1,
            field.label.trim(),
            field.key
        ));
        if !field.instructions.trim().is_empty() {
            out.push_str(&format!(": {}", field.instructions.trim()));
        }
        if field.reason {
            out.push_str(&format!(
                " Give the reason for your answer in one sentence in `{}`.",
                kalamu_core::analysis::reason_key(&field.key)
            ));
        }
        if let FieldKind::Entries { fields } = &field.kind {
            describe_fields(fields, &format!("{indent}   "), out);
        }
    }
}
