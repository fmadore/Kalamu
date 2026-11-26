//! One test for each refusal, and for each value read leniently.

use serde_json::{Value, json};

use crate::{BUNDLED, Catalog, CatalogError, MAX_BYTES};

fn bundled() -> Value {
    serde_json::from_str(BUNDLED).unwrap_or(Value::Null)
}

/// The bundled list with one change.
fn changed(change: impl FnOnce(&mut Value)) -> String {
    let mut list = bundled();
    change(&mut list);
    list.to_string()
}

/// The bundled list's model `id`.
fn model<'a>(list: &'a mut Value, id: &str) -> &'a mut Value {
    let models = list["models"].as_array_mut();
    match models.and_then(|m| m.iter_mut().find(|m| m["id"] == id)) {
        Some(model) => model,
        None => panic!("no model {id}"),
    }
}

/// The list with one change is refused, for a reason that says `why`.
fn refused(why: &str, change: impl FnOnce(&mut Value)) {
    match Catalog::from_json(&changed(change)) {
        Err(CatalogError::Refused(problems)) => {
            assert!(problems.contains(why), "{problems:?} does not say {why:?}");
        }
        other => panic!("not refused for {why:?}: {:?}", other.map(|c| c.revision)),
    }
}

#[test]
fn the_bundled_list_passes_every_check() {
    let catalog = Catalog::read(BUNDLED);
    let catalog = catalog.unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(catalog.problems(&super::hosts(&catalog)), [] as [String; 0]);
    assert!(catalog.revision > 0, "the bundled list carries a revision");
}

#[test]
fn a_list_too_large_is_refused_before_it_is_read() {
    let mut list = bundled();
    list["notes"] = "x".repeat(MAX_BYTES).into();
    match Catalog::from_json(&list.to_string()) {
        Err(CatalogError::Refused(why)) => assert!(why.contains("larger than")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_newer_schema_is_too_new() {
    let list = changed(|l| l["schema_version"] = 2.into());
    assert!(matches!(
        Catalog::from_json(&list),
        Err(CatalogError::TooNew)
    ));
    // Even written in a way this build cannot read at all.
    let list = changed(|l| {
        l["schema_version"] = 2.into();
        l["models"] = "elsewhere".into();
    });
    assert!(matches!(
        Catalog::from_json(&list),
        Err(CatalogError::TooNew)
    ));
}

#[test]
fn a_list_that_is_not_json_is_unreadable() {
    assert!(matches!(
        Catalog::from_json("<html>"),
        Err(CatalogError::Invalid(_))
    ));
}

#[test]
fn an_empty_list_is_refused() {
    refused("no model", |l| l["models"] = json!([]));
    refused("no provider", |l| l["providers"] = json!([]));
    refused("no language", |l| l["languages"] = json!([]));
}

#[test]
fn a_day_written_otherwise_is_refused() {
    refused("updated is not a day", |l| {
        l["updated"] = "2026-11-5".into()
    });
    refused("updated is not a day", |l| {
        l["updated"] = "2026-13-01".into()
    });
    refused("updated is not a day", |l| l["updated"] = "tomorrow".into());
}

#[test]
fn a_negative_price_is_refused() {
    refused("input_per_mtok is negative", |l| {
        model(l, "gemini-4-hydrogen")["pricing"]["input_per_mtok"] = (-1.0).into();
    });
    refused("per_minute is negative", |l| {
        model(l, "whisper-base")["pricing"]["per_minute"] = (-0.01).into();
    });
    refused("batch_discount is not between 0 and 1", |l| {
        model(l, "gemini-4-hydrogen")["pricing"]["batch_discount"] = 1.5.into();
    });
    refused("input_price is negative", |l| {
        l["embedding_models"][0]["input_price"] = (-0.1).into();
    });
}

#[test]
fn a_limit_of_nothing_is_refused() {
    refused("max_input_tokens is 0", |l| {
        model(l, "gemini-4-hydrogen")["limits"]["max_input_tokens"] = 0.into();
    });
}

#[test]
fn a_model_of_an_unknown_provider_or_listed_twice_is_refused() {
    refused("names an unknown provider", |l| {
        model(l, "gemini-4-hydrogen")["provider"] = "nobody".into();
    });
    refused("is listed twice", |l| {
        let copy = model(l, "whisper-base").clone();
        if let Some(models) = l["models"].as_array_mut() {
            models.push(copy);
        }
    });
    refused("provider gemini is listed twice", |l| {
        let copy = l["providers"][0].clone();
        if let Some(providers) = l["providers"].as_array_mut() {
            providers.push(copy);
        }
    });
}

#[test]
fn a_model_without_tasks_inputs_or_languages_is_refused() {
    refused("has no task", |l| {
        model(l, "whisper-base")["tasks"] = json!([])
    });
    refused("reads no type of file", |l| {
        model(l, "whisper-base")["inputs"] = json!([]);
    });
    refused("reads no type of file", |l| {
        model(l, "whisper-base")["inputs"] = json!(["audio"]);
    });
    refused("offers no language", |l| {
        model(l, "whisper-base")["languages"] = json!([]);
    });
    refused("neither \"any\" nor a list", |l| {
        model(l, "gemini-4-hydrogen")["languages"] = "all".into();
    });
    refused("reasons at no level", |l| {
        model(l, "gemini-4-hydrogen")["reasoning"]["levels"] = json!([]);
    });
}

#[test]
fn a_default_for_a_task_it_cannot_do_or_no_longer_offered_is_refused() {
    refused("recommended_for names a task it cannot do", |l| {
        model(l, "gemini-3.5-transcribe")["recommended_for"] = json!(["analysis"]);
    });
    refused(
        "provider_default_for names a model no longer offered",
        |l| {
            model(l, "gemini-4-hydrogen")["retired"] = true.into();
        },
    );
}

#[test]
fn a_task_without_a_model_is_refused() {
    refused("no model offered for audio_transcription", |l| {
        for m in l["models"].as_array_mut().into_iter().flatten() {
            if m["tasks"]
                .as_array()
                .is_some_and(|t| t.contains(&"audio_transcription".into()))
            {
                m["retired"] = true.into();
            }
        }
    });
}

#[test]
fn a_successor_that_names_no_model_or_loops_is_refused() {
    refused("successor gemini-9 is not another model", |l| {
        model(l, "gemini-3.8-flash")["successor"] = "gemini-9".into();
    });
    refused("successor gpt-6.2-sol is not another model", |l| {
        model(l, "gemini-3.8-flash")["successor"] = "gpt-6.2-sol".into();
    });
    refused("is not another model", |l| {
        model(l, "gemini-3.8-flash")["successor"] = "gemini-3.8-flash".into();
    });
    refused("its line of successors loops", |l| {
        model(l, "gemini-4-neon")["successor"] = "gemini-3.1-pro-preview".into();
    });
}

#[test]
fn an_alias_that_names_no_model_or_is_a_model_is_refused() {
    refused("names no model", |l| {
        l["aliases"][0]["model"] = "nothing".into()
    });
    refused("is a model's id", |l| {
        l["aliases"][0]["id"] = "zai-glm-5-3".into()
    });
}

#[test]
fn an_embedding_model_said_to_run_here_for_a_cloud_provider_is_refused() {
    refused("runs elsewhere than its provider", |l| {
        l["embedding_models"][0]["local"] = true.into();
    });
    refused("gives no length of vector", |l| {
        l["embedding_models"][0]["dimensions"] = json!([]);
    });
    refused("max_chars is 0", |l| {
        l["embedding_models"][0]["max_chars"] = 0.into()
    });
}

/// A provider's key page opens in the browser: only on a host the bundled list names, written
/// as a plain `https://` address.
#[test]
fn a_key_page_elsewhere_is_refused() {
    for url in [
        "https://example.com/keys",
        "http://aistudio.google.com/apikey",
        "https://aistudio.google.com.example.com/apikey",
        "https://user@aistudio.google.com/apikey",
        "https://aistudio.google.com:8443/apikey",
        "https://aistudio.google.com/api key",
        "javascript:alert(1)",
        "file:///C:/Windows",
    ] {
        refused("provider gemini:", |l| {
            l["providers"][0]["key_url"] = url.into()
        });
    }
    refused("names no page for its keys", |l| {
        l["providers"][0]["key_url"] = "".into();
    });
    // The same hosts with another page, or written in capitals, are allowed.
    let list = changed(|l| {
        l["providers"][0]["key_url"] = "https://AIStudio.google.com/other?x=1".into();
    });
    assert!(Catalog::from_json(&list).is_ok());
}

/// A model whose status, tier or way of taking languages this build does not know is hidden,
/// and the list still reads: a newer build's list never locks this one out.
#[test]
fn a_model_with_an_unknown_status_tier_or_language_hints_is_hidden() {
    for (field, value) in [
        ("status", "sunset"),
        ("tier", "ultra"),
        ("language_hints", "none"),
    ] {
        let list = changed(|l| model(l, "gpt-6-luna")[field] = value.into());
        let catalog = Catalog::from_json(&list).unwrap_or_else(|e| panic!("{field}: {e}"));
        assert_eq!(catalog.models.len(), Catalog::bundled().models.len() - 1);
        assert!(
            catalog.models.iter().all(|m| m.id != "gpt-6-luna"),
            "{field}"
        );
    }
}

/// A value of the wrong kind is not an unknown name: it refuses the list.
#[test]
fn a_value_of_the_wrong_kind_is_unreadable() {
    let list = changed(|l| model(l, "gpt-6-luna")["tier"] = 3.into());
    assert!(matches!(
        Catalog::from_json(&list),
        Err(CatalogError::Invalid(_))
    ));
}

/// A task this build does not know is left out; a model left with no task it knows is hidden.
#[test]
fn an_unknown_task_is_left_out_and_a_model_with_none_known_is_hidden() {
    let list = changed(|l| {
        let m = model(l, "gemini-4-hydrogen");
        for list in ["tasks", "recommended_for", "provider_default_for"] {
            if let Some(tasks) = m[list].as_array_mut() {
                tasks.push("illustration".into());
            }
        }
        model(l, "gpt-6-luna")["tasks"] = json!(["illustration"]);
    });
    let catalog = Catalog::from_json(&list).unwrap_or_else(|e| panic!("{e}"));
    let bundled = Catalog::bundled();
    let hydrogen = |c: &Catalog| {
        c.models
            .iter()
            .find(|m| m.id == "gemini-4-hydrogen")
            .map(|m| (m.tasks.clone(), m.recommended_for.clone()))
    };
    assert_eq!(hydrogen(&catalog), hydrogen(&bundled));
    assert!(catalog.models.iter().all(|m| m.id != "gpt-6-luna"));
}

/// The successor of a model may be one this build hides: the older model stays offered, as it
/// was before its successor existed, and an alias for a hidden model is forgotten.
#[test]
fn a_hidden_successor_leaves_the_older_model_offered() {
    let list = changed(|l| {
        model(l, "gpt-6.2-sol")["tier"] = "ultra".into();
        model(l, "deepseek-v4-1-flash-1017")["status"] = "sunset".into();
    });
    let catalog = Catalog::from_json(&list).unwrap_or_else(|e| panic!("{e}"));
    let older = catalog.models.iter().find(|m| m.id == "gpt-6.1-sol");
    assert_eq!(
        older.map(|m| (m.successor.is_none(), m.is_offered())),
        Some((true, true))
    );
    assert!(catalog.aliases.is_empty());
}

/// Of two lists, the higher revision wins: a correction made the same day arrives, and a list
/// written before the number never replaces the bundled one.
#[test]
fn the_higher_revision_wins_whatever_the_day() {
    let bundled = Catalog::bundled();
    let correction = changed(|l| {
        l["revision"] = (bundled.revision + 1).into();
        model(l, "gpt-6-luna")["name"] = "GPT-6 Luna, corrected".into();
    });
    let newest = Catalog::newest(Some(&correction));
    assert_eq!(newest.updated, bundled.updated);
    assert_eq!(newest.revision, bundled.revision + 1);
    assert!(
        newest
            .models
            .iter()
            .any(|m| m.name == "GPT-6 Luna, corrected")
    );

    // A later day is not enough.
    let later = changed(|l| {
        l["updated"] = "2999-01-01".into();
        model(l, "gpt-6-luna")["name"] = "Later".into();
    });
    assert_eq!(Catalog::newest(Some(&later)), bundled);
    let unnumbered = changed(|l| {
        if let Some(fields) = l.as_object_mut() {
            fields.remove("revision");
        }
        l["updated"] = "2999-01-01".into();
    });
    assert_eq!(
        Catalog::from_json(&unnumbered).map(|c| c.revision).ok(),
        Some(0)
    );
    assert_eq!(Catalog::newest(Some(&unnumbered)), bundled);
}

/// A list that fails its checks never replaces the bundled one, however high its revision: a
/// bad list kept from before stops winning at the next launch.
#[test]
fn a_refused_list_never_replaces_the_bundled_one() {
    let bad = changed(|l| {
        l["revision"] = 9_999.into();
        model(l, "gemini-4-hydrogen")["pricing"]["input_per_mtok"] = (-1.0).into();
    });
    assert!(Catalog::from_json(&bad).is_err());
    assert_eq!(Catalog::newest(Some(&bad)), Catalog::bundled());
}

/// A server of the researcher's own is theirs to add: a list that names one, by its `own`
/// field or by an id of the form servers are given, is refused whole.
#[test]
fn a_list_naming_a_server_of_ones_own_is_refused() {
    let why = "a server of the researcher's own";
    refused(why, |l| {
        l["providers"][0]["own"] = json!({
            "address": "http://192.168.1.20:11434",
            "host": "192.168.1.20",
            "this_computer": false,
            "key": false,
            "dialect": "ollama",
        });
    });
    refused(why, |l| {
        if let Some(providers) = l["providers"].as_array_mut() {
            providers.push(json!({
                "id": "own-1a2b3c4d",
                "name": "Office",
                "key_url": "",
                "key_prefix": null,
                "local": true,
            }));
        }
    });
}
