use super::*;

/// The flags New job reads must match what the provider modules send: OpenAI's
/// transcriptions take two-letter codes, and Whisper, Voxtral and the diarizing model one
/// language.
#[test]
fn transcription_language_hints_match_the_providers() -> Result<(), CatalogError> {
    let catalog = Catalog::from_json(BUNDLED)?;
    for model in catalog
        .models
        .iter()
        .filter(|m| m.tasks.contains(&Task::AudioTranscription))
    {
        let expected = match (model.provider.as_str(), model.id.as_str()) {
            ("whisper", _) | ("openai", "gpt-4o-transcribe-diarize") => LanguageHints::One,
            ("mistral", _) => LanguageHints::OneWithoutTimestamps,
            _ => LanguageHints::Several,
        };
        assert_eq!(model.language_hints, expected, "{}", model.id);
        assert_eq!(
            model.two_letter_codes,
            model.provider.as_str() == "openai",
            "{}",
            model.id
        );
    }
    Ok(())
}

/// Translations are for the interface's other languages, and say something.
#[test]
fn translations_are_for_interface_languages() -> Result<(), CatalogError> {
    let catalog = Catalog::from_json(BUNDLED)?;
    let all = catalog
        .providers
        .iter()
        .flat_map(|p| &p.translations)
        .chain(catalog.models.iter().flat_map(|m| &m.translations));
    for (language, t) in all {
        assert!(["fr", "de"].contains(&language.as_str()), "{language}");
        assert!(t.name.is_some() || t.description.is_some());
        assert!(
            t.name
                .iter()
                .chain(&t.description)
                .all(|s| !s.trim().is_empty())
        );
    }
    Ok(())
}

#[test]
fn bundled_catalogue_is_valid_and_consistent() -> Result<(), CatalogError> {
    let catalog = Catalog::from_json(BUNDLED)?;
    for model in &catalog.models {
        assert!(
            catalog.provider(&model.provider).is_some(),
            "{} names an unknown provider",
            model.id
        );
        assert!(!model.tasks.is_empty(), "{} has no task", model.id);
        assert!(!model.inputs.is_empty(), "{} reads nothing", model.id);
        assert!(
            model
                .recommended_for
                .iter()
                .all(|t| model.tasks.contains(t)),
            "{} is recommended for a task it cannot do",
            model.id
        );
        assert!(
            model
                .provider_default_for
                .iter()
                .all(|t| model.tasks.contains(t)),
            "{} is a default for a task it cannot do",
            model.id
        );
        if let LanguageSupport::Any(word) = &model.languages {
            assert_eq!(word, "any");
        }
    }
    // Each provider has exactly one default for each task it offers.
    for provider in &catalog.providers {
        for task in [
            Task::TextExtraction,
            Task::AudioTranscription,
            Task::Analysis,
        ] {
            let models = catalog
                .models
                .iter()
                .filter(|m| m.provider == provider.id && m.tasks.contains(&task));
            let defaults = models
                .clone()
                .filter(|m| m.provider_default_for.contains(&task))
                .count();
            let expected = usize::from(models.count() > 0);
            assert_eq!(defaults, expected, "{} {task:?}", provider.id);
        }
    }
    for task in [
        Task::TextExtraction,
        Task::AudioTranscription,
        Task::Analysis,
    ] {
        assert!(catalog.recommended(task, &[]).is_some());
    }
    Ok(())
}

#[test]
fn a_newer_download_replaces_the_bundled_catalogue() {
    let mut newer = Catalog::bundled();
    newer.updated = "2999-01-01".into();
    newer.models.truncate(1);
    let json = serde_json::to_string(&newer).unwrap_or_default();
    assert_eq!(Catalog::newest(Some(&json)).models.len(), 1);

    let mut older = Catalog::bundled();
    older.updated = "2000-01-01".into();
    let json = serde_json::to_string(&older).unwrap_or_default();
    assert_eq!(Catalog::newest(Some(&json)), Catalog::bundled());
    assert_eq!(Catalog::newest(Some("not json")), Catalog::bundled());
}

#[test]
fn recommendation_prefers_providers_with_a_key() {
    let catalog = Catalog::bundled();
    let only_claude = [ProviderId::new("anthropic")];
    let model = catalog.recommended(Task::TextExtraction, &only_claude);
    assert_eq!(model.map(|m| m.provider.as_str()), Some("anthropic"));
}

#[test]
fn estimates_scale_with_pages_and_minutes() {
    let catalog = Catalog::bundled();
    let flash = catalog.model(&ProviderId::new("gemini"), "gemini-3.8-flash");
    let flash = flash.cloned().unwrap_or_else(|| catalog.models[0].clone());
    let ten = catalog
        .estimate(&flash, Task::TextExtraction, 10, 0.0, 0)
        .usd;
    let twenty = catalog
        .estimate(&flash, Task::TextExtraction, 20, 0.0, 0)
        .usd;
    assert!(matches!((ten, twenty), (Some(a), Some(b)) if (b - 2.0 * a).abs() < 1e-9));

    let ocr = catalog
        .model(&ProviderId::new("mistral"), "mistral-ocr-4-1")
        .cloned();
    if let Some(ocr) = ocr {
        let usd = catalog.estimate(&ocr, Task::TextExtraction, 5, 0.0, 0).usd;
        assert!(usd.is_some_and(|u| (u - 0.02).abs() < 1e-9), "{usd:?}");
    }
}

#[test]
fn cached_input_and_batches_are_billed_at_their_own_prices() {
    let catalog = Catalog::bundled();
    let mut model = catalog.models[0].clone();
    model.pricing = Pricing {
        input_per_mtok: Some(2.0),
        output_per_mtok: Some(10.0),
        cached_input_per_mtok: Some(0.2),
        cache_write_per_mtok: Some(2.5),
        batch_discount: Some(0.5),
        ..Pricing::default()
    };
    let usage = |batch| kalamu_core::Usage {
        input_tokens: 1_000_000,
        cached_input_tokens: 600_000,
        cache_write_tokens: 100_000,
        output_tokens: 100_000,
        batch,
        ..kalamu_core::Usage::default()
    };
    // 0.3 M plain at $2, 0.6 M cached at $0.2, 0.1 M written at $2.5, 0.1 M out at $10.
    let fast = catalog.actual_cost(&model, &usage(false));
    assert!(fast.is_some_and(|c| (c - 1.97).abs() < 1e-9), "{fast:?}");
    let economy = catalog.actual_cost(&model, &usage(true));
    assert!(
        economy.is_some_and(|c| (c - 0.985).abs() < 1e-9),
        "{economy:?}"
    );

    let estimate = catalog.estimate(&model, Task::Analysis, 1, 0.0, 0);
    let (Some(usd), Some(economy)) = (estimate.usd, estimate.economy_usd) else {
        panic!("{estimate:?}");
    };
    assert!((economy - usd / 2.0).abs() < 1e-12);
    model.pricing.batch_discount = None;
    assert_eq!(
        catalog
            .estimate(&model, Task::Analysis, 1, 0.0, 0)
            .economy_usd,
        None
    );
}

/// Gemini 3.5 Transcribe is estimated by the minute, as Google estimates it, but its answer
/// reports tokens, not seconds: the cost comes from them, as the live run of 3 November 2026
/// was billed (1,308 tokens read and 177 written for 52 seconds).
#[test]
fn a_transcriber_billed_by_tokens_is_estimated_by_the_minute() {
    let catalog = Catalog::bundled();
    let model = catalog
        .model(&ProviderId::new("gemini"), "gemini-3.5-transcribe")
        .cloned()
        .unwrap_or_else(|| panic!("no transcribe"));
    let estimate = catalog.estimate(&model, Task::AudioTranscription, 0, 2.0, 0);
    assert_eq!(estimate.usd, Some(0.01));
    let usage = kalamu_core::Usage {
        input_tokens: 1_308,
        output_tokens: 177,
        ..kalamu_core::Usage::default()
    };
    let cost = catalog.actual_cost(&model, &usage);
    assert!(cost.is_some_and(|c| (c - 0.00474).abs() < 1e-6), "{cost:?}");
}

#[test]
fn contents_and_corrections_take_the_analysis_models_that_read_their_input() {
    let catalog = Catalog::bundled();
    let ids = |task| -> Vec<String> {
        catalog
            .models_for(task)
            .iter()
            .map(|m| m.id.clone())
            .collect()
    };
    let contents = ids(Task::Contents);
    assert!(contents.contains(&"gemini-3.8-flash".to_string()));
    assert!(
        !contents.iter().any(|m| m.starts_with("deepseek")),
        "reads text only"
    );
    assert!(
        !contents.iter().any(|m| m.starts_with("mistral-ocr")),
        "no analysis"
    );
    let corrections = ids(Task::TextCorrection);
    assert!(corrections.iter().any(|m| m.starts_with("deepseek")));
    assert!(!corrections.iter().any(|m| m.starts_with("voxtral")));
    assert!(
        catalog
            .merge_models()
            .iter()
            .any(|m| m.id.starts_with("deepseek"))
    );
    assert_eq!(
        catalog
            .recommended(Task::Contents, &[])
            .map(|m| m.id.as_str()),
        catalog
            .recommended(Task::Analysis, &[])
            .map(|m| m.id.as_str())
    );
    let flash = catalog
        .model(&ProviderId::new("gemini"), "gemini-3.8-flash")
        .cloned()
        .unwrap_or_else(|| panic!("no flash"));
    let merge = catalog.estimate_merge(&flash, 40, 0);
    assert!(merge.usd.is_some() && merge.economy_usd.is_none());
}

#[test]
fn reasoning_suggests_the_lowest_for_reading_and_keeps_to_the_levels_a_model_has() {
    let catalog = Catalog::bundled();
    let model = |id: &str| {
        catalog
            .models
            .iter()
            .find(|m| m.id == id)
            .unwrap_or_else(|| panic!("{id}"))
    };
    for m in &catalog.models {
        if let Some(r) = &m.reasoning {
            assert!(!r.levels.is_empty(), "{}", m.id);
            if let Some(d) = &r.default {
                assert!(r.levels.contains(d), "{} default", m.id);
            }
            let mut ranks = r.levels.iter().map(|l| rank(l));
            assert!(
                ranks
                    .clone()
                    .zip(ranks.by_ref().skip(1))
                    .all(|(a, b)| a < b),
                "{} levels out of order",
                m.id
            );
        }
    }
    let opus = model("claude-opus-5-5");
    assert_eq!(opus.limits.max_output_tokens, Some(128_000));
    let level = |m: &ModelInfo, task, chosen| m.tuning(task, chosen).reasoning;
    // Reading a page: the lowest; an analysis: `low`; a choice the model has: kept.
    assert_eq!(
        level(opus, Task::TextExtraction, None).as_deref(),
        Some("low")
    );
    let terra = model("gpt-6.1-terra");
    assert_eq!(
        level(terra, Task::TextExtraction, None).as_deref(),
        Some("none")
    );
    assert_eq!(level(terra, Task::Analysis, None).as_deref(), Some("low"));
    assert_eq!(
        level(opus, Task::Analysis, Some("xhigh")).as_deref(),
        Some("xhigh")
    );
    // A level the model lacks: the nearest it has, the lower of two equally near.
    let medium = model("mistral-medium-3-5");
    assert_eq!(level(medium, Task::Analysis, None).as_deref(), Some("none"));
    assert_eq!(
        level(medium, Task::Analysis, Some("xhigh")).as_deref(),
        Some("high")
    );
    let large = model("mistral-large-4-0");
    assert_eq!(
        level(large, Task::Analysis, Some("none")).as_deref(),
        Some("low")
    );
    assert_eq!(
        level(large, Task::Analysis, Some("max")).as_deref(),
        Some("high")
    );
    // OCR and speech-to-text models do not reason.
    assert_eq!(
        level(model("mistral-ocr-4-1"), Task::TextExtraction, Some("high")),
        None
    );
    // Where Mistral documents no output limit, none is set.
    assert_eq!(medium.limits.max_output_tokens, None);
}

/// The abilities providers read from the catalogue (#147) are, for today's models, exactly what
/// they decided by name before: Claude's fallback for Opus and Sonnet 5.5, OpenAI's speakers
/// for the diarizing model, Gemini's speech request for its transcribe model, and the lowest
/// thinking level Mistral's hosted GLM and DeepSeek accept.
#[test]
fn abilities_are_what_providers_once_read_from_names() -> Result<(), CatalogError> {
    let catalog = Catalog::from_json(BUNDLED)?;
    for model in &catalog.models {
        let id = model.id.as_str();
        let tuning = model.tuning(Task::AudioTranscription, None);
        match model.provider.as_str() {
            "anthropic" => assert_eq!(
                tuning.refusal_fallback,
                ["claude-opus-5-5", "claude-sonnet-5-5"].contains(&id),
                "{id}"
            ),
            "openai" if model.does(Task::AudioTranscription) => {
                assert_eq!(tuning.speakers, id.contains("diarize"), "{id}");
            }
            "gemini" => assert_eq!(tuning.no_prompt, id.contains("transcribe"), "{id}"),
            // The guess applied only to a model the catalogue lacks: every chat model listed
            // has its levels, GLM's and DeepSeek's starting at `low`.
            "mistral" if model.tasks == [Task::Analysis] => {
                let first = model.reasoning.as_ref().and_then(|r| r.levels.first());
                assert!(first.is_some(), "{id}");
                if id.starts_with("zai-glm") || id.starts_with("deepseek") {
                    assert_eq!(first.map(String::as_str), Some("low"), "{id}");
                }
            }
            _ => {}
        }
        assert!(!tuning.refusal_fallback || model.provider.as_str() == "anthropic");
    }
    Ok(())
}

/// A catalogue written before the abilities were (a cache from an older release, its fields
/// absent) loads, reads as lacking them, and is never preferred to the bundled one, which has
/// them: this build never reads a catalogue older than its own. A newer catalogue's fields
/// this build does not know are ignored, so older releases read today's file.
#[test]
fn a_catalogue_without_the_abilities_never_replaces_the_bundled_one() -> Result<(), CatalogError> {
    let parse = |json: &str| -> Result<serde_json::Value, CatalogError> {
        serde_json::from_str(json).map_err(|e| CatalogError::Invalid(e.to_string()))
    };
    let mut old = parse(BUNDLED)?;
    for model in old["models"].as_array_mut().into_iter().flatten() {
        if let Some(fields) = model.as_object_mut() {
            fields.remove("refusal_fallback");
        }
    }
    let same_day = old.to_string();
    let read = Catalog::from_json(&same_day)?;
    assert!(read.models.iter().all(|m| !m.refusal_fallback));
    let opus = |c: &Catalog| {
        c.model(&ProviderId::new("anthropic"), "claude-opus-5-5")
            .map(|m| m.tuning(Task::TextExtraction, None).refusal_fallback)
    };
    assert_eq!(opus(&Catalog::newest(Some(&same_day))), Some(true));
    old["updated"] = "2026-10-01".into();
    assert_eq!(opus(&Catalog::newest(Some(&old.to_string()))), Some(true));

    let mut newer = parse(BUNDLED)?;
    newer["a_field_not_yet_known"] = "9.9.9".into();
    for model in newer["models"].as_array_mut().into_iter().flatten() {
        model["an_ability_not_yet_known"] = true.into();
    }
    assert_eq!(Catalog::from_json(&newer.to_string())?, Catalog::bundled());
    Ok(())
}
