use scriptorium_core::LanguageCode;

use super::*;

#[test]
fn every_task_has_defaults_and_ids_are_unique() {
    let library = PromptLibrary::in_memory();
    assert!(!library.list(Some(Task::TextExtraction)).is_empty());
    assert!(!library.list(Some(Task::AudioTranscription)).is_empty());
    let mut ids: Vec<_> = defaults::DEFAULTS.iter().map(|d| d.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), defaults::DEFAULTS.len());
    assert!(
        defaults::DEFAULTS
            .iter()
            .all(|d| !d.instructions.trim().is_empty())
    );
}

#[test]
fn editing_a_default_overrides_it_and_reset_restores_it() -> Result<(), PromptError> {
    let dir = tempfile::tempdir().map_err(io)?;
    let mut library = PromptLibrary::open(dir.path())?;
    let mut preset = library.get("text_extraction.newspaper")?;
    preset.instructions = "Shorter.".into();
    library.save(preset)?;

    let reopened = PromptLibrary::open(dir.path())?;
    let edited = reopened.get("text_extraction.newspaper")?;
    assert_eq!(edited.instructions, "Shorter.");
    assert!(edited.modified);

    let mut library = reopened;
    library.delete("text_extraction.newspaper")?;
    let restored = library.get("text_extraction.newspaper")?;
    assert!(!restored.modified);
    assert!(restored.instructions.contains("Paragraph Consolidation"));
    Ok(())
}

#[test]
fn user_presets_are_saved_duplicated_and_deleted() -> Result<(), PromptError> {
    let dir = tempfile::tempdir().map_err(io)?;
    let mut library = PromptLibrary::open(dir.path())?;
    let copy = library.duplicate("text_extraction.printed_diplomatic", "Colonial reports")?;
    assert!(copy.id.starts_with("user-"));
    assert!(!copy.builtin);
    assert_eq!(
        PromptLibrary::open(dir.path())?.get(&copy.id)?.name,
        "Colonial reports"
    );
    library.delete(&copy.id)?;
    assert!(matches!(
        library.get(&copy.id),
        Err(PromptError::NotFound(_))
    ));
    Ok(())
}

#[test]
fn import_never_overwrites_existing_presets() -> Result<(), PromptError> {
    let mut library = PromptLibrary::in_memory();
    let shared = library.export(&["text_extraction.newspaper".to_string()])?;
    let imported = library.import(&shared)?;
    assert_eq!(imported.len(), 1);
    assert!(imported[0].id.starts_with("user-"));
    assert!(!library.get("text_extraction.newspaper")?.modified);
    assert!(matches!(
        library.import(r#"{"format":"other","version":1,"presets":[]}"#),
        Err(PromptError::InvalidImport(_))
    ));
    Ok(())
}

#[test]
fn chosen_languages_reach_the_prompt() {
    let ctx = RenderContext {
        languages: LanguageChoice::Explicit(vec![
            LanguageCode::new("fr"),
            LanguageCode::new("mos"),
        ]),
        language_names: vec!["French".into(), "Mooré".into()],
        speakers: Some(2),
    };
    assert_eq!(
        render("Speakers: {speakers}. Languages: {languages}.", &ctx),
        "Speakers: 2. Languages: French, Mooré."
    );
    assert!(render("Transcribe.", &ctx).ends_with("The source is in French, Mooré."));
    assert_eq!(
        render("Transcribe.", &RenderContext::default()),
        "Transcribe."
    );
}
