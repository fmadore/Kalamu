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

/// `n` pages of one document, read at the suggested reasoning level.
fn pages(n: u32) -> Workload<'static> {
    Workload {
        pages: n,
        documents: 1,
        ..Workload::default()
    }
}

#[test]
fn estimates_scale_with_pages_and_minutes() {
    let catalog = Catalog::bundled();
    let flash = catalog.model(&ProviderId::new("gemini"), "gemini-3.8-flash");
    let flash = flash.cloned().unwrap_or_else(|| catalog.models[0].clone());
    let ten = catalog
        .estimate(&flash, Task::TextExtraction, &pages(10))
        .usd;
    let twenty = catalog
        .estimate(&flash, Task::TextExtraction, &pages(20))
        .usd;
    assert!(matches!((ten, twenty), (Some(a), Some(b)) if (b - 2.0 * a).abs() < 1e-9));

    let ocr = catalog
        .model(&ProviderId::new("mistral"), "mistral-ocr-4-1")
        .cloned();
    if let Some(ocr) = ocr {
        let usd = catalog.estimate(&ocr, Task::TextExtraction, &pages(5)).usd;
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

    let estimate = catalog.estimate(&model, Task::Analysis, &pages(1));
    let (Some(usd), Some(economy)) = (estimate.usd, estimate.economy_usd) else {
        panic!("{estimate:?}");
    };
    assert!((economy - usd / 2.0).abs() < 1e-12);
    model.pricing.batch_discount = None;
    assert_eq!(
        catalog
            .estimate(&model, Task::Analysis, &pages(1))
            .economy_usd,
        None
    );
}

/// What jobs were billed in the 1.10.0 test pass, which the estimates fell far short of on
/// newspaper pages (about $0.35 for $1.21) and on screening (about $0.018 for $0.20): each is
/// now met, a little above, with the figures of the job as New job sends them.
#[test]
fn estimates_meet_what_the_1_10_test_pass_was_billed() {
    let catalog = Catalog::bundled();
    let model = |provider: &str, id: &str| {
        catalog
            .model(&ProviderId::new(provider), id)
            .unwrap_or_else(|| panic!("{id}"))
    };
    let (hydrogen, sol) = (
        model("gemini", "gemini-4-hydrogen"),
        model("openai", "gpt-6.2-sol"),
    );
    let sonnet = model("anthropic", "claude-sonnet-5-5");
    let usd = |model: &ModelInfo, task: Task, work: Workload<'_>| {
        catalog
            .estimate(model, task, &work)
            .usd
            .unwrap_or_else(|| panic!("{} has a price", model.id))
    };
    let newspaper = |n: u32| Workload {
        pages: n,
        documents: 1,
        prompt_chars: 4_000,
        dense: true,
        ..Workload::default()
    };
    let within = |estimate: f64, billed: f64, what: &str| {
        assert!(
            estimate >= 0.85 * billed && estimate <= 1.3 * billed,
            "{what}: estimated ${estimate:.3}, billed ${billed:.3}"
        );
    };

    // The workflow's first step: 36 newspaper pages read by Gemini 4 Hydrogen.
    let hydrogen_36 = usd(hydrogen, Task::TextExtraction, newspaper(36));
    within(hydrogen_36, 1.21, "36 newspaper pages");
    // The 5 April issue, its 4 pages read by Hydrogen and by GPT-6.2 Sol.
    let both = usd(hydrogen, Task::TextExtraction, newspaper(4))
        + usd(sol, Task::TextExtraction, newspaper(4));
    within(both, 0.31, "4 pages by two models");
    // A table of contents of a 4-page issue: its pages, and the step that combines them.
    let contents = usd(
        hydrogen,
        Task::Contents,
        Workload {
            prompt_chars: 2_400,
            ..pages(4)
        },
    ) + catalog
        .estimate_merge(hydrogen, 4, 2_200)
        .usd
        .unwrap_or_default();
    within(contents, 0.058, "a table of contents");
    // A handwritten page of 1755 by Claude Sonnet 5.5.
    let handwritten = usd(
        sonnet,
        Task::TextExtraction,
        Workload {
            prompt_chars: 5_000,
            ..pages(1)
        },
    );
    within(handwritten, 0.021, "a handwritten page");
    // Screening ten corrected texts with Hydrogen, nine of about 4,800 words (28,800
    // characters): each document answered in a call of its own, its thinking billed.
    let screening = usd(
        hydrogen,
        Task::Analysis,
        Workload {
            text_chars: 9 * 28_800 + 1_500,
            documents: 10,
            prompt_chars: 3_000,
            ..Workload::default()
        },
    );
    within(screening, 0.20, "screening ten documents");
    // An ordinary page is not priced as a newspaper's.
    let ordinary = usd(
        hydrogen,
        Task::TextExtraction,
        Workload {
            dense: false,
            ..newspaper(36)
        },
    );
    assert!(
        ordinary < hydrogen_36 / 1.5,
        "{ordinary} against {hydrogen_36}"
    );
}

/// What the test pass of 1.10.0-rc2 was billed, from the provenance files' usage, where the
/// estimates of rc1 still fell short: Gemini's thinking on a page, a newspaper page read with
/// the printed-document prompt, the interview analysis, and a strip of a page read again. Each
/// is now estimated within 0.8 to 1.3 times what it cost, and the cases that were near stay so.
#[test]
fn estimates_meet_what_the_rc2_test_pass_was_billed() {
    let catalog = Catalog::bundled();
    let model = |provider: &str, id: &str| {
        catalog
            .model(&ProviderId::new(provider), id)
            .unwrap_or_else(|| panic!("{id}"))
    };
    let (hydrogen, neon, sol) = (
        model("gemini", "gemini-4-hydrogen"),
        model("gemini", "gemini-4-neon"),
        model("openai", "gpt-6.2-sol"),
    );
    let (sonnet, mistral) = (
        model("anthropic", "claude-sonnet-5-5"),
        model("mistral", "mistral-ocr-4-1"),
    );
    let usd = |model: &ModelInfo, task: Task, work: &Workload<'_>| {
        catalog
            .estimate(model, task, work)
            .usd
            .unwrap_or_else(|| panic!("{} has a price", model.id))
    };
    // What a call was billed, from the input and output tokens the provider reported.
    let billed = |model: &ModelInfo, input: u64, output: u64| {
        catalog
            .actual_cost(
                model,
                &kalamu_core::Usage {
                    input_tokens: input,
                    output_tokens: output,
                    ..kalamu_core::Usage::default()
                },
            )
            .unwrap_or_default()
    };
    let within = |estimate: f64, billed: f64, what: &str| {
        let ratio = estimate / billed;
        assert!(
            (0.8..=1.3).contains(&ratio),
            "{what}: estimated ${estimate:.4}, billed ${billed:.4} ({ratio:.2} times)"
        );
    };

    // One job of three set-ups, low thinking, the printed-document prompt (4,196 characters):
    // the 1-page `daily-nebraskan-1911.pdf`, whose text layer holds about 1,940 words, and the
    // handwritten page of 1755.
    let printed = 4_196;
    let newspaper = Workload {
        dense_pages: 1,
        prompt_chars: printed,
        ..pages(1)
    };
    let handwritten = Workload {
        prompt_chars: printed,
        ..pages(1)
    };
    let mut total = (0.0, 0.0);
    for (model, page, letter) in [
        (sol, (12_497, 2_495), (4_033, 404)),
        (hydrogen, (5_374, 5_127), (2_185, 1_783)),
        (neon, (5_374, 5_486), (2_185, 2_327)),
    ] {
        let estimate = usd(model, Task::TextExtraction, &newspaper)
            + usd(model, Task::TextExtraction, &handwritten);
        let cost = billed(model, page.0, page.1) + billed(model, letter.0, letter.1);
        within(estimate, cost, &model.id);
        within(
            usd(model, Task::TextExtraction, &handwritten),
            billed(model, letter.0, letter.1),
            &format!("{}, the handwritten page", model.id),
        );
        total = (total.0 + estimate, total.1 + cost);
    }
    within(total.0, total.1, "three set-ups");
    // The page counted as dense from its text layer: read as an ordinary page, it would be
    // priced at about half.
    let ordinary = usd(hydrogen, Task::TextExtraction, &handwritten);
    assert!(ordinary < usd(hydrogen, Task::TextExtraction, &newspaper) / 1.5);

    // The interview analysis: a transcript of 1,568 bytes, the interview prompt, its themes and
    // quotations two fields of quoted entries.
    let interview = Workload {
        text_chars: 1_568,
        documents: 1,
        prompt_chars: 5_615,
        cited_entries: 2,
        ..Workload::default()
    };
    within(
        usd(hydrogen, Task::Analysis, &interview),
        billed(hydrogen, 3_082, 2_521),
        "the interview analysis",
    );

    // A strip of two lines of the 1755 page read again with Claude Sonnet 5.5 and the
    // handwriting prompt (5,950 characters): priced as the picture sent, far below the whole
    // page ($0.021); Mistral OCR bills a page whatever its size.
    let strip = |model: &ModelInfo| {
        usd(
            model,
            Task::TextExtraction,
            &Workload {
                prompt_chars: 5_950,
                region: Some(0.8 * 0.08),
                ..pages(1)
            },
        )
    };
    let page = usd(
        sonnet,
        Task::TextExtraction,
        &Workload {
            prompt_chars: 5_950,
            ..pages(1)
        },
    );
    within(page, 0.021, "the whole handwritten page");
    assert!(
        (0.004..0.008).contains(&strip(sonnet)),
        "a strip: ${:.4}",
        strip(sonnet)
    );
    assert!((strip(mistral) - 0.004).abs() < 1e-12);

    // Near or under before, and still: a records analysis of a 4-page issue, a translation of
    // a 207-word text, a page of Mistral OCR.
    let records = catalog
        .estimate_records(hydrogen, 4, 2_829, None)
        .usd
        .unwrap_or_default();
    within(records, billed(hydrogen, 9_088, 3_210), "records");
    let translation = usd(
        hydrogen,
        Task::Translation,
        &Workload {
            prompt_chars: 3_457,
            ..pages(1)
        },
    );
    within(translation, billed(hydrogen, 1_416, 1_509), "a translation");
    within(
        usd(mistral, Task::TextExtraction, &pages(1)),
        0.004,
        "Mistral OCR",
    );
    // GPT-6.2 Sol's People, places and organisations on an issue of about 5,000 words
    // ($0.019 to $0.022 billed) was estimated a little high, and is not raised.
    let entities = usd(
        sol,
        Task::Analysis,
        &Workload {
            text_chars: 29_000,
            documents: 1,
            prompt_chars: 2_255,
            ..Workload::default()
        },
    );
    assert!(
        (0.019..0.031).contains(&entities),
        "entities: ${entities:.4}"
    );
}

/// A model's own measure of what it thinks on a page replaces the allowance on an ordinary page,
/// scaled by the reasoning level as the allowance is; a dense page keeps the larger of the two.
#[test]
fn a_models_own_page_thinking_replaces_the_allowance() {
    let catalog = Catalog::bundled();
    let mut model = catalog
        .model(&ProviderId::new("gemini"), "gemini-4-hydrogen")
        .cloned()
        .unwrap_or_else(|| panic!("no Hydrogen"));
    model.pricing = Pricing {
        input_per_mtok: Some(1.0),
        output_per_mtok: Some(1.0),
        ..Pricing::default()
    };
    let at = |model: &ModelInfo, work: Workload<'_>| {
        catalog
            .estimate(model, Task::TextExtraction, &work)
            .usd
            .unwrap_or_default()
            * 1_000_000.0
    };
    // 1,100 tokens in, 900 of text and 1,400 of thinking out.
    model.page_thinking = Some(1_400);
    assert!((at(&model, pages(1)) - 3_400.0).abs() < 1e-6);
    // Twice at `medium`.
    let medium = at(
        &model,
        Workload {
            reasoning: Some("medium"),
            ..pages(1)
        },
    );
    assert!((medium - 4_800.0).abs() < 1e-6, "{medium}");
    // Without a measure, the allowance of 600.
    model.page_thinking = None;
    assert!((at(&model, pages(1)) - 2_600.0).abs() < 1e-6);
    // A dense page: 3,000 in, 2,000 of text and 2,500 of thinking, a smaller measure ignored.
    model.page_thinking = Some(100);
    let dense = Workload {
        dense_pages: 1,
        ..pages(1)
    };
    assert!((at(&model, dense.clone()) - 7_500.0).abs() < 1e-6);
    // A region a tenth of the page is priced as a fifth of one: 220 in, 200 out.
    let region = at(
        &model,
        Workload {
            region: Some(0.1),
            ..pages(1)
        },
    );
    assert!((region - 420.0).abs() < 1e-6, "{region}");
}

/// Thinking is billed as output: a higher reasoning level costs more, a model told not to
/// reason or unable to costs only its answer.
#[test]
fn a_higher_reasoning_level_is_priced_higher() {
    let catalog = Catalog::bundled();
    let mut model = catalog
        .model(&ProviderId::new("openai"), "gpt-6.2-sol")
        .cloned()
        .unwrap_or_else(|| panic!("no Sol"));
    let at = |model: &ModelInfo, level: Option<&str>| {
        catalog
            .estimate(
                model,
                Task::Analysis,
                &Workload {
                    reasoning: level,
                    ..pages(1)
                },
            )
            .usd
            .unwrap_or_default()
    };
    // The suggestion for an analysis is `low`.
    assert_eq!(at(&model, None), at(&model, Some("low")));
    assert!(at(&model, Some("high")) > at(&model, Some("medium")));
    assert!(at(&model, Some("medium")) > at(&model, Some("low")));
    // 1,100 tokens in at $2 and 700 out at $10, without thinking.
    model.reasoning = None;
    let none = at(&model, Some("high"));
    assert!((none - 0.0092).abs() < 1e-12, "{none}");
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
    let four = catalog.estimate_records(&model, 4, 400, None);
    let (Some(usd), Some(economy)) = (four.usd, four.economy_usd) else {
        panic!("{four:?}");
    };
    assert!((usd - 4.0 * 0.0124).abs() < 1e-12, "{usd}");
    assert!((economy - usd / 2.0).abs() < 1e-12);
    let eight = catalog.estimate_records(&model, 8, 400, None).usd;
    assert!(eight.is_some_and(|e| (e - 2.0 * usd).abs() < 1e-12));
    // Once per document: 4,400 + 100 tokens in, 1,500 out: $0.024, under half.
    let once = catalog
        .estimate(
            &model,
            Task::Analysis,
            &Workload {
                prompt_chars: 400,
                ..pages(4)
            },
        )
        .usd;
    assert!(once.is_some_and(|o| (o - 0.024).abs() < 1e-12), "{once:?}");
    // A model priced by the page alone has no token price for rows.
    model.pricing = Pricing {
        per_page: Some(0.001),
        ..Pricing::default()
    };
    assert_eq!(catalog.estimate_records(&model, 4, 400, None).usd, None);
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
    let estimate = catalog.estimate(
        &model,
        Task::AudioTranscription,
        &Workload {
            audio_minutes: 2.0,
            ..Workload::default()
        },
    );
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
    // More thinking: the next level the model has, never past the bound.
    let sol = model("gpt-6.2-sol")
        .reasoning
        .as_ref()
        .unwrap_or_else(|| panic!("GPT-6.2 Sol reasons"));
    assert_eq!(sol.above("low", "high"), Some("medium"));
    assert_eq!(sol.above("medium", "high"), Some("high"));
    assert_eq!(sol.above("high", "high"), None);
    assert_eq!(sol.above("low", "low"), None);
    let medium_levels = medium
        .reasoning
        .as_ref()
        .unwrap_or_else(|| panic!("reasons"));
    assert_eq!(
        medium_levels.above("none", "high"),
        medium_levels.levels.get(1).map(String::as_str)
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
