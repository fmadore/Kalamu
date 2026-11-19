//! The researcher's question kept with an analysis prompt of their own: saved trimmed, kept
//! through a restart, a duplicate, an export and an import; dropped when empty or for another
//! task; and absent from files written before it existed, which read unchanged.

use super::*;

const PORT: &str = "Le travail des femmes au port";

#[test]
fn a_question_is_kept_with_an_analysis_prompt_through_a_restart_and_a_copy()
-> Result<(), PromptError> {
    let dir = tempfile::tempdir().map_err(io)?;
    let mut library = PromptLibrary::open(dir.path())?;
    let mut own = library.get("analysis.interview")?;
    own.id = String::new();
    own.name = "Port".into();
    own.question = Some(format!("  {PORT}\n"));
    let own = library.save(own)?;
    assert_eq!(own.question.as_deref(), Some(PORT), "saved trimmed");
    // Same fields and instructions as the ready-made prompt it was copied from.
    let shipped = library.get("analysis.interview")?;
    assert_eq!(own.schema, shipped.schema);
    assert_eq!(own.instructions, shipped.instructions);
    assert_eq!(shipped.question, None);

    let reopened = PromptLibrary::open(dir.path())?;
    assert_eq!(reopened.get(&own.id)?.question.as_deref(), Some(PORT));
    let copy = library.duplicate(&own.id, "Port 2")?;
    assert_eq!(copy.question.as_deref(), Some(PORT));

    // Emptied, it is none; it is listed among the analysis prompts like any other.
    let mut emptied = copy;
    emptied.question = Some("   ".into());
    assert_eq!(library.save(emptied)?.question, None);
    assert!(
        library
            .list(Some(Task::Analysis))
            .iter()
            .any(|p| p.id == own.id)
    );
    Ok(())
}

#[test]
fn only_an_analysis_keeps_a_question() -> Result<(), PromptError> {
    let mut library = PromptLibrary::in_memory();
    let mut own = library.get("text_extraction.printed")?;
    own.id = String::new();
    own.question = Some(PORT.into());
    assert_eq!(library.save(own)?.question, None);
    Ok(())
}

#[test]
fn a_question_travels_in_a_shared_file_and_older_files_read_unchanged() -> Result<(), PromptError> {
    let mut library = PromptLibrary::in_memory();
    let mut own = library.get("analysis.interview")?;
    own.id = String::new();
    own.question = Some(PORT.into());
    let own = library.save(own)?;
    let shared = library.export(std::slice::from_ref(&own.id))?;
    assert!(shared.contains(PORT), "{shared}");
    let imported = library.import(&shared)?;
    assert_eq!(imported[0].question.as_deref(), Some(PORT));
    assert_ne!(imported[0].id, own.id);

    // A prompt without one writes no such key, so files stay as they were written before.
    let plain = library.export(&["analysis.interview".to_string()])?;
    assert!(!plain.contains("\"question\""), "{plain}");
    let older = r#"{"format":"kalamu-prompts","version":1,"presets":[{"id":"user-x",
        "task":"analysis","name":"Old","instructions":"Read it.","user_prompt":"Fill it.",
        "schema":{"fields":[{"key":"","label":"Summary","instructions":"","kind":{"type":"text"},
        "reason":false}]}}]}"#;
    assert_eq!(library.import(older)?[0].question, None);
    Ok(())
}
