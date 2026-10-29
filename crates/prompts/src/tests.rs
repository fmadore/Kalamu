use kalamu_core::LanguageCode;

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
        line_breaks: None,
        answers: false,
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

#[test]
fn an_answer_language_is_the_answers_not_the_sources() {
    let auto = RenderContext {
        answers: true,
        ..RenderContext::default()
    };
    let text = render("Write the fields in {languages}.", &auto);
    assert!(
        text.contains("the language the source is written in"),
        "{text}"
    );
    assert!(text.contains("even though these instructions are in English"));

    let english = RenderContext {
        languages: LanguageChoice::Explicit(vec![LanguageCode::new("en")]),
        language_names: vec!["English".into()],
        answers: true,
        ..RenderContext::default()
    };
    let note = render("Summarise.", &english);
    assert!(note.ends_with("Write your answers in English, whatever the language of the source; keep names as the source spells them."), "{note}");
    assert!(!note.contains("The source is in"));

    // A table of contents names the answer language in both steps, and never as the source's.
    let library = PromptLibrary::in_memory();
    for preset in library.list(Some(Task::Contents)) {
        let page = preset.snapshot(&english).text;
        let merge = preset
            .merge_snapshot(&english)
            .unwrap_or_else(|| panic!("no merge"))
            .text;
        assert!(
            page.contains("Write the fields in English"),
            "{}",
            preset.id
        );
        assert!(!page.contains("is in English"), "{}", preset.id);
        assert!(
            merge.contains("in that one language for every entry"),
            "{}",
            preset.id
        );
    }
}

#[test]
fn the_line_choice_overrides_the_preset() {
    let kept = RenderContext {
        line_breaks: Some(LineBreaks::AsOnPage),
        ..RenderContext::default()
    };
    let joined = RenderContext {
        line_breaks: Some(LineBreaks::RunningText),
        ..RenderContext::default()
    };
    let text = render("Join lines into paragraphs.", &kept);
    assert!(text.starts_with("Join lines into paragraphs.\n\n## Line breaks"));
    assert!(text.contains("keep end-of-line hyphens exactly"));
    assert!(text.ends_with("joining lines or removing hyphens."));
    assert!(render("Keep line breaks.", &joined).contains("rejoin a word split across two lines"));
}

#[test]
fn every_analysis_default_has_valid_fields_described_in_its_prompt() {
    let library = PromptLibrary::in_memory();
    let analyses = library.list(Some(Task::Analysis));
    assert!(analyses.len() >= 4);
    for preset in analyses {
        let schema = preset.schema.clone().unwrap_or_default();
        assert_eq!(schema.validate(), Ok(()), "{}", preset.id);
        let snapshot = preset.snapshot(&RenderContext::default());
        assert_eq!(snapshot.schema.as_ref(), Some(&schema));
        assert!(snapshot.text.contains("## What to find"), "{}", preset.id);
        for field in &schema.fields {
            assert!(snapshot.text.contains(&format!("(`{}`", field.key)));
        }
        assert!(!snapshot.text.contains("{languages}"));
    }
    let tone = library
        .get("analysis.tone")
        .map(|p| p.snapshot(&RenderContext::default()));
    let text = tone.map(|s| s.text).unwrap_or_default();
    assert!(text.contains("(`tone`, exactly one of: Positive, Neutral, Negative, Mixed)"));
    assert!(text.contains("in `tone_reason`"));
}

#[test]
fn an_analysis_cannot_be_saved_without_fields_and_others_lose_theirs() {
    let mut library = PromptLibrary::in_memory();
    let mut analysis = library
        .get("analysis.entities")
        .unwrap_or_else(|e| panic!("{e}"));
    analysis.id = String::new();
    analysis.schema = None;
    assert!(matches!(
        library.save(analysis.clone()),
        Err(PromptError::Schema(SchemaError::Empty))
    ));
    let mut extraction = library
        .get("text_extraction.newspaper")
        .unwrap_or_else(|e| panic!("{e}"));
    extraction.id = String::new();
    extraction.schema = library.get("analysis.tone").ok().and_then(|p| p.schema);
    let saved = library.save(extraction).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(saved.schema, None);
}

#[test]
fn a_table_of_contents_freezes_both_steps_with_the_entry_fields() {
    let library = PromptLibrary::in_memory();
    for preset in library.list(Some(Task::Contents)) {
        assert!(preset.merge_instructions.is_some(), "{}", preset.id);
        let ctx = RenderContext {
            languages: LanguageChoice::Explicit(vec![LanguageCode::new("fr")]),
            language_names: vec!["French".into()],
            ..RenderContext::default()
        };
        let page = preset.snapshot(&ctx);
        let merge = preset
            .merge_snapshot(&ctx)
            .unwrap_or_else(|| panic!("no merge"));
        let page_schema = page.schema.unwrap_or_else(|| panic!("no page schema"));
        assert_eq!(page_schema.fields[0].key, kalamu_core::contents::ENTRIES);
        assert!(page.text.contains("`title`") && page.text.contains("`continues`"));
        assert!(merge.text.contains("`pages`"));
        assert!(merge.text.contains("French") && !merge.text.contains("{languages}"));
        assert_eq!(merge.user_prompt, MERGE_REQUEST);
        assert_eq!(
            kalamu_core::contents::entry_fields(&page_schema),
            preset.schema.clone().map(|s| s.fields).unwrap_or_default()
        );
    }
    assert!(
        library
            .get("text_extraction.newspaper")
            .unwrap_or_else(|e| panic!("{e}"))
            .merge_snapshot(&RenderContext::default())
            .is_none()
    );
}

#[test]
fn a_contents_preset_needs_its_fields_and_its_second_step() {
    let mut library = PromptLibrary::in_memory();
    let mut own = library
        .get("contents.archival_file")
        .unwrap_or_else(|e| panic!("{e}"));
    own.id = String::new();
    own.merge_instructions = Some(" ".into());
    assert!(matches!(
        library.save(own.clone()),
        Err(PromptError::Incomplete)
    ));
    own.merge_instructions = Some("Combine.".into());
    if let Some(schema) = own.schema.as_mut() {
        schema.fields[0].key = "pages".into();
    }
    assert!(matches!(
        library.save(own.clone()),
        Err(PromptError::Schema(_))
    ));
    own.schema = library
        .get("contents.archival_file")
        .ok()
        .and_then(|p| p.schema);
    assert!(library.save(own).is_ok());
}

#[test]
fn a_correction_answers_with_its_text_in_one_field() {
    let library = PromptLibrary::in_memory();
    let preset = library.default_for(Task::TextCorrection);
    assert_eq!(preset.id, "text_correction.general");
    let snapshot = preset.snapshot(&RenderContext {
        line_breaks: Some(LineBreaks::AsOnPage),
        ..RenderContext::default()
    });
    assert_eq!(snapshot.schema, Some(kalamu_core::correction::schema()));
    assert!(snapshot.text.contains("in the field `text`"));
    assert!(snapshot.text.contains("## Line breaks"));
    assert!(!snapshot.text.contains("## What to find"));
}
