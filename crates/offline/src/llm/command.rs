//! What the two programs are told, and the answer read from what they print back.

use super::{BATCH, CONTEXT, IMAGE_TOKENS};

/// The arguments both programs share, but for the output limit (`-n`, added by `process`): the
/// model, greedy decoding, the answer's schema, the context and the threads; and on a graphics processor (Metal, or the Vulkan build's device)
/// as many layers as its memory holds beside the fixed context, the rest on the processor
/// (`-ngl auto` with `--fit on`), so a card with little memory of its own still helps rather
/// than failing or being passed over. The processor-only build ignores both.
fn common_arguments(model: &str, schema: &str, threads: i32) -> Vec<String> {
    [
        "-m",
        model,
        "-jf",
        schema,
        "--jinja",
        // Greedy decoding: the same request answers the same each time, as the cache assumes.
        "--temp",
        "0",
        "-c",
        &CONTEXT.to_string(),
        "-b",
        &BATCH.to_string(),
        "-ub",
        &BATCH.to_string(),
        "-t",
        &threads.to_string(),
        "-ngl",
        "auto",
        "--fit",
        "on",
        "--no-warmup",
    ]
    .iter()
    .map(|a| (*a).to_owned())
    .collect()
}

/// The text program's arguments: the instructions, the text and the request from files, as one
/// turn of a conversation in the model's own chat template, without thinking (Gemma 4 E4B and
/// E2B write no thoughts when it is off, so the schema holds from the first token), and without
/// printing the request back.
pub(super) fn text_arguments(
    model: &str,
    schema: &str,
    system: &str,
    user: &str,
    threads: i32,
) -> Vec<String> {
    let mut args = common_arguments(model, schema, threads);
    args.extend(
        [
            "-sysf",
            system,
            "-f",
            user,
            "-cnv",
            "-st",
            "--reasoning",
            "off",
            "--no-display-prompt",
        ]
        .iter()
        .map(|a| (*a).to_owned()),
    );
    args
}

/// The multimodal program's arguments: the page image first, then the request.
pub(super) fn page_arguments(
    model: &str,
    encoder: &str,
    schema: &str,
    page: &str,
    system: &str,
    user: &str,
    threads: i32,
) -> Vec<String> {
    let mut args = common_arguments(model, schema, threads);
    args.extend([
        "--mmproj".to_owned(),
        encoder.to_owned(),
        "--image".to_owned(),
        page.to_owned(),
        "--image-max-tokens".to_owned(),
        IMAGE_TOKENS.to_string(),
        "-sys".to_owned(),
        system.to_owned(),
        "-p".to_owned(),
        format!("<__media__>{user}"),
    ]);
    args
}

/// The answer the program printed: the text program ends it with ` [end of text]`, and anything
/// around the JSON object is dropped.
pub(super) fn answer(printed: &str) -> &str {
    let printed = printed.trim();
    let printed = printed.strip_suffix("[end of text]").unwrap_or(printed);
    match (printed.find('{'), printed.rfind('}')) {
        (Some(start), Some(end)) if start < end => &printed[start..=end],
        _ => printed.trim(),
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::llm::tests::summary;
    use kalamu_core::{FieldValue, ProcessOutput, structured};

    #[test]
    fn the_arguments_read_files_hold_the_schema_and_turn_thinking_off() {
        let args = text_arguments("m.gguf", "s.json", "i.txt", "r.txt", 6);
        let pair = |flag: &str| {
            let i = args.iter().position(|a| a == flag).unwrap();
            args[i + 1].clone()
        };
        assert_eq!(pair("-m"), "m.gguf");
        assert_eq!(pair("-jf"), "s.json");
        assert_eq!(pair("-sysf"), "i.txt");
        assert_eq!(pair("-f"), "r.txt");
        assert_eq!(pair("--reasoning"), "off");
        assert_eq!(pair("--temp"), "0");
        assert_eq!(pair("-c"), CONTEXT.to_string());
        assert_eq!(pair("-t"), "6");
        assert_eq!(pair("-ngl"), "auto");
        assert_eq!(pair("--fit"), "on");
        assert!(args.iter().any(|a| a == "-st") && args.iter().any(|a| a == "--no-display-prompt"));

        let args = page_arguments("m.gguf", "e.gguf", "s.json", "p.png", "Sys", "Fill.", 6);
        let pair = |flag: &str| {
            let i = args.iter().position(|a| a == flag).unwrap();
            args[i + 1].clone()
        };
        assert_eq!(pair("--mmproj"), "e.gguf");
        assert_eq!(pair("--image"), "p.png");
        assert_eq!(pair("-p"), "<__media__>Fill.");
        assert_eq!(pair("-sys"), "Sys");
        assert_eq!(pair("--image-max-tokens"), IMAGE_TOKENS.to_string());
    }

    #[test]
    fn the_answer_is_the_json_object_the_program_printed() {
        assert_eq!(
            answer("{\"summary\":\"Une réunion.\"} [end of text]\n"),
            "{\"summary\":\"Une réunion.\"}"
        );
        assert_eq!(answer("\n{\"a\":1}\n"), "{\"a\":1}");
        let output = structured::finish(
            ProcessOutput {
                text: answer("{\"summary\":\"Une réunion.\"} [end of text]").to_owned(),
                ..ProcessOutput::default()
            },
            &summary(),
        )
        .unwrap();
        assert_eq!(
            output.fields[0].value,
            FieldValue::Text("Une réunion.".into())
        );
    }
}
