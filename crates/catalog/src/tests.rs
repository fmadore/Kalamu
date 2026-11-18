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

/// A records analysis reads each page with the instructions and writes a page of rows for it,
/// where an analysis answered once per document reads the pages once and writes one answer.
#[test]
fn a_records_analysis_is_priced_page_by_page() {
    let catalog = Catalog::bundled();
    let mut model = catalog.models[0].clone();
    model.pricing = Pricing {
        input_per_mtok: Some(2.0),
        output_per_mtok: Some(10.0),
        batch_discount: Some(0.5),
        ..Pricing::default()
    };
    // 400 characters of instructions are 100 tokens. Each page: 1,100 + 100 tokens in at $2,
    // 1,000 out at $10: $0.0124.
    let four = catalog.estimate_records(&model, 4, 400);
    let (Some(usd), Some(economy)) = (four.usd, four.economy_usd) else {
        panic!("{four:?}");
    };
    assert!((usd - 4.0 * 0.0124).abs() < 1e-12, "{usd}");
    assert!((economy - usd / 2.0).abs() < 1e-12);
    let eight = catalog.estimate_records(&model, 8, 400).usd;
    assert!(eight.is_some_and(|e| (e - 2.0 * usd).abs() < 1e-12));
    // Once per document: 4,400 + 100 tokens in, 1,500 out: $0.024, under half.
    let once = catalog.estimate(&model, Task::Analysis, 4, 0.0, 400).usd;
    assert!(once.is_some_and(|o| (o - 0.024).abs() < 1e-12), "{once:?}");
    // A model priced by the page alone has no token price for rows.
    model.pricing = Pricing {
        per_page: Some(0.001),
        ..Pricing::default()
    };
    assert_eq!(catalog.estimate_records(&model, 4, 400).usd, None);
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
    // A translation takes the same models as a correction: analysis models that read text.
    assert_eq!(ids(Task::Translation), corrections);
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

/// Every entry says where it stands among its provider's models: the default an entry without
/// one is read as is for a catalogue written by hand elsewhere, not for the bundled one.
#[test]
fn every_model_names_its_tier() -> Result<(), CatalogError> {
    let json: serde_json::Value =
        serde_json::from_str(BUNDLED).map_err(|e| CatalogError::Invalid(e.to_string()))?;
    for model in json["models"].as_array().into_iter().flatten() {
        assert!(
            matches!(model["tier"].as_str(), Some("top" | "balanced" | "light")),
            "{}",
            model["id"]
        );
    }
    Ok(())
}

/// The embedding models are a list of their own: each names a provider the catalogue knows,
/// runs on this computer exactly when its provider does and then has no price, offers at least
/// one length of vector, and never appears among the job models New job offers.
#[test]
fn embedding_models_are_listed_apart_from_job_models() -> Result<(), CatalogError> {
    let catalog = Catalog::from_json(BUNDLED)?;
    let models = catalog.embedding_models();
    assert!(models.iter().any(|m| m.local));
    assert!(models.iter().any(|m| !m.local));
    for model in models {
        let provider = catalog.provider(&model.provider);
        assert!(provider.is_some(), "{} names an unknown provider", model.id);
        assert_eq!(provider.map(|p| p.local), Some(model.local), "{}", model.id);
        assert_eq!(model.input_price.is_none(), model.local, "{}", model.id);
        assert!(!model.dimensions.is_empty(), "{}", model.id);
        assert!(model.dimensions.iter().all(|d| *d > 0), "{}", model.id);
        assert!(model.max_chars >= 1_000, "{}", model.id);
        assert!(!model.name.trim().is_empty(), "{}", model.id);
        assert_eq!(
            models
                .iter()
                .filter(|m| m.provider == model.provider && m.id == model.id)
                .count(),
            1,
            "{} is listed twice",
            model.id
        );
        assert_eq!(
            catalog.embedding_model(&model.provider, &model.id),
            Some(model)
        );
        // Not a job model: no task lists it, and no lookup of a job model finds it.
        assert!(
            catalog.model(&model.provider, &model.id).is_none(),
            "{}",
            model.id
        );
        for task in [
            Task::TextExtraction,
            Task::AudioTranscription,
            Task::Analysis,
            Task::Contents,
            Task::TextCorrection,
            Task::Translation,
        ] {
            assert!(
                catalog.models_for(task).iter().all(|m| m.id != model.id),
                "{} {task:?}",
                model.id
            );
        }
        assert!(catalog.merge_models().iter().all(|m| m.id != model.id));
    }
    // The model run on this computer that the offline crate embeds with.
    let local = catalog.embedding_model(&ProviderId::new("local-llm"), "embeddinggemma-2");
    assert_eq!(
        local.map(|m| (
            m.name.as_str(),
            m.dimensions.as_slice(),
            m.max_chars,
            m.multilingual
        )),
        Some(("EmbeddingGemma 2", &[768, 512, 256, 128][..], 2_000, true))
    );
    Ok(())
}

/// An older build reads today's file, the list it does not know ignored, and this build reads a
/// file written before the list, as listing none: it never replaces the bundled one, which is
/// newer, so the embedding models are not lost.
#[test]
fn a_catalogue_without_embedding_models_still_reads() -> Result<(), CatalogError> {
    let parse = |json: &str| -> Result<serde_json::Value, CatalogError> {
        serde_json::from_str(json).map_err(|e| CatalogError::Invalid(e.to_string()))
    };
    let mut old = parse(BUNDLED)?;
    if let Some(fields) = old.as_object_mut() {
        fields.remove("embedding_models");
    }
    old["updated"] = "2026-11-01".into();
    let read = Catalog::from_json(&old.to_string())?;
    assert!(read.embedding_models().is_empty());
    assert_eq!(read.models, Catalog::bundled().models);
    let newest = Catalog::newest(Some(&old.to_string()));
    assert!(!newest.embedding_models().is_empty());

    let mut newer = parse(BUNDLED)?;
    for model in newer["embedding_models"]
        .as_array_mut()
        .into_iter()
        .flatten()
    {
        model["a_field_not_yet_known"] = true.into();
    }
    assert_eq!(Catalog::from_json(&newer.to_string())?, Catalog::bundled());
    // Written back as read, the list included.
    let again = serde_json::to_string(&Catalog::bundled())
        .map_err(|e| CatalogError::Invalid(e.to_string()))?;
    assert_eq!(Catalog::from_json(&again)?, Catalog::bundled());
    Ok(())
}

#[test]
fn embedding_costs_scale_with_tokens_and_nothing_is_paid_on_this_computer() {
    let catalog = Catalog::bundled();
    let gemini = catalog.embedding_model(&ProviderId::new("gemini"), "gemini-embedding-2");
    let cost = gemini
        .and_then(|m| embedding_cost(m, 2_000_000))
        .unwrap_or(-1.0);
    assert!((cost - 0.4).abs() < 1e-12, "{cost}");
    assert_eq!(gemini.and_then(|m| embedding_cost(m, 0)), Some(0.0));
    let small = catalog.embedding_model(&ProviderId::new("openai"), "text-embedding-3-small");
    let cost = small
        .and_then(|m| embedding_cost(m, 500_000))
        .unwrap_or(-1.0);
    assert!((cost - 0.01).abs() < 1e-12, "{cost}");
    let local = catalog.embedding_model(&ProviderId::new("local-llm"), "embeddinggemma-2");
    assert_eq!(local.and_then(|m| embedding_cost(m, 9_999_999)), Some(0.0));
    let Some(gemini) = gemini else {
        panic!("gemini-embedding-2 is in the catalogue");
    };
    let unpriced = EmbeddingModel {
        input_price: None,
        ..gemini.clone()
    };
    assert_eq!(embedding_cost(&unpriced, 1_000), None);
}
