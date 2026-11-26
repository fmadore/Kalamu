use super::*;
use crate::own::is_own_id;
use kalamu_core::address::AddressError;

const DAY: &str = "2026-11-26";

fn trial(reads_pages: bool, fills_forms: bool) -> OwnTrial {
    OwnTrial {
        reads_pages,
        fills_forms,
        digest: Some("sha256:abc".into()),
        tried_at: DAY.into(),
    }
}

fn model(id: &str) -> ModelDraft {
    ModelDraft {
        id: id.into(),
        context: 32_768,
        max_output: 8_192,
        input_per_mtok: None,
        output_per_mtok: None,
    }
}

fn draft(name: &str, address: &str, models: Vec<ModelDraft>) -> ServerDraft {
    ServerDraft {
        id: None,
        name: name.into(),
        address: address.into(),
        dialect: Dialect::Ollama,
        asks_key: false,
        at_once: 1,
        models,
    }
}

/// A list with the office Mac saved, its two models tried.
fn saved() -> (OwnServers, OwnServer) {
    let mut trials = Trials::default();
    trials.record("http://192.168.1.20:11434", "gemma3:12b", trial(true, true));
    trials.record("http://192.168.1.20:11434", "llama3", trial(false, true));
    let mut list = OwnServers::default();
    let saved = list
        .apply(
            draft(
                " Office Mac ",
                "HTTP://192.168.1.20:11434/",
                vec![model("gemma3:12b"), model("llama3")],
            ),
            &trials,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    (list, saved.server)
}

#[test]
fn a_new_server_is_saved_under_a_new_id_each_model_doing_what_its_trial_showed() {
    let (list, server) = saved();
    assert!(is_own_id(server.id.as_str()));
    assert_eq!(server.name, "Office Mac");
    assert_eq!(server.address, "http://192.168.1.20:11434");
    assert_eq!(list.servers, std::slice::from_ref(&server));
    let gemma = &server.models[0];
    assert!(gemma.reads_pages && gemma.fills_forms);
    assert_eq!(gemma.tried_at.as_deref(), Some(DAY));
    assert_eq!(gemma.digest.as_deref(), Some("sha256:abc"));
    assert!(!server.models[1].reads_pages && server.models[1].fills_forms);
    assert!(list.problems().is_empty());
}

#[test]
fn a_model_without_a_trial_there_or_that_did_nothing_is_not_saved() {
    let mut list = OwnServers::default();
    let mut trials = Trials::default();
    trials.record("http://localhost:11434", "blind", trial(false, false));
    trials.record("http://elsewhere:11434", "gemma3:12b", trial(true, true));
    let refused = list.apply(
        draft("Mac", "http://localhost:11434", vec![model("gemma3:12b")]),
        &trials,
    );
    assert_eq!(
        refused,
        Err(DraftError::NotTried {
            model: "gemma3:12b".into()
        })
    );
    let refused = list.apply(
        draft("Mac", "http://localhost:11434", vec![model("blind")]),
        &trials,
    );
    let refused = refused.err().unwrap_or_else(|| panic!("saved"));
    assert_eq!(refused.code(), "server.did_nothing");
    assert_eq!(refused.params(), [("model", "blind".to_string())]);
    assert!(list.servers.is_empty(), "nothing saved");
}

#[test]
fn a_server_changed_keeps_its_id_and_its_trials_while_its_address_stays() {
    let (mut list, server) = saved();
    let mut cheaper = model("gemma3:12b");
    cheaper.context = 16_384;
    cheaper.input_per_mtok = Some(0.25);
    let changed = ServerDraft {
        id: Some(server.id.clone()),
        asks_key: true,
        at_once: 2,
        ..draft("Office", "http://192.168.1.20:11434", vec![cheaper])
    };
    let saved = list
        .apply(changed, &Trials::default())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(saved.server.id, server.id);
    assert!(!saved.address_changed());
    assert_eq!(saved.previous.as_ref(), Some(&server));
    assert!(saved.server.key);
    assert_eq!(saved.server.models.len(), 1, "llama3 was unticked");
    let gemma = &saved.server.models[0];
    assert!(gemma.reads_pages && gemma.fills_forms);
    assert_eq!((gemma.context, gemma.input_per_mtok), (16_384, Some(0.25)));
    assert_eq!(list.servers, std::slice::from_ref(&saved.server));

    // At another address, the models must be tried again there; once they are, consent is
    // asked again.
    let moved = ServerDraft {
        id: Some(server.id.clone()),
        ..draft("Office", "http://10.0.0.5:11434", vec![model("gemma3:12b")])
    };
    assert_eq!(
        list.apply(moved.clone(), &Trials::default()),
        Err(DraftError::NotTried {
            model: "gemma3:12b".into()
        })
    );
    let mut trials = Trials::default();
    trials.record("http://10.0.0.5:11434/", "gemma3:12b", trial(false, true));
    let saved = list.apply(moved, &trials).unwrap_or_else(|e| panic!("{e}"));
    assert!(saved.address_changed());
    assert!(!saved.server.models[0].reads_pages, "the new trial's word");
}

#[test]
fn a_draft_refused_leaves_the_list_as_it_was_and_says_why_with_its_values() {
    let (mut list, server) = saved();
    let before = list.clone();
    let mut trials = Trials::default();
    trials.record("http://localhost:11434", "gemma3:12b", trial(true, true));
    let same_name = list.apply(
        draft(
            "office mac",
            "http://localhost:11434",
            vec![model("gemma3:12b")],
        ),
        &trials,
    );
    let same_name = same_name.err().unwrap_or_else(|| panic!("saved"));
    assert_eq!(same_name.code(), "server.same_name");
    assert_eq!(same_name.params(), [("name", "office mac".to_string())]);
    assert_eq!(list, before);

    let bad_address = list.apply(draft("Uni", "ftp://uni", vec![]), &trials);
    let bad_address = bad_address.err().unwrap_or_else(|| panic!("saved"));
    assert_eq!(
        bad_address,
        DraftError::Problem(ServerProblem::Address {
            server: "Uni".into(),
            reason: AddressError::Scheme
        })
    );
    assert_eq!(bad_address.code(), "server.address_scheme");

    let too_many = ServerDraft {
        at_once: 9,
        ..draft("Uni", "https://llm.uni.example/v1", vec![])
    };
    let too_many = list.apply(too_many, &trials).err();
    let too_many = too_many.unwrap_or_else(|| panic!("saved"));
    assert_eq!(too_many.code(), "server.at_once");
    assert_eq!(
        too_many.params(),
        [("server", "Uni".to_string()), ("most", "8".to_string())]
    );

    let gone = ServerDraft {
        id: Some(ProviderId::new("own-ffffffff")),
        ..draft("Gone", "http://localhost:11434", vec![])
    };
    assert_eq!(
        list.apply(gone, &trials).map_err(|e| e.code()),
        Err("server.not_found")
    );
    assert_eq!(list, before);

    assert_eq!(list.remove(&server.id), Some(server.clone()));
    assert!(list.servers.is_empty());
    assert_eq!(list.remove(&server.id), None);
}

#[test]
fn a_server_found_shows_the_models_already_added_with_their_limits() {
    let (_, server) = saved();
    let found_model = |id: &str| FoundServerModel {
        id: id.into(),
        vision: Some(true),
        audio: None,
        trained_context: Some(131_072),
        elsewhere: None,
        size_bytes: Some(8_000_000_000),
        digest: None,
        context: 32_768,
        max_output: 8_192,
        added: false,
    };
    let mut mac = server.clone();
    mac.models[0].context = 12_000;
    let list = OwnServers {
        servers: vec![mac.clone()],
    };
    let found = FoundServer::new(
        Dialect::Ollama,
        "http://192.168.1.20:11434".into(),
        Some("0.40.18".into()),
        vec![found_model("gemma3:12b"), found_model("qwen3:8b")],
    );
    assert_eq!(found.host, "192.168.1.20");
    assert!(!found.this_computer);
    for shown in [
        list.found(None, found.clone()),
        list.found(Some(&mac.id), found.clone()),
    ] {
        assert!(shown.models[0].added);
        assert_eq!(shown.models[0].context, 12_000);
        assert!(!shown.models[1].added);
        assert_eq!(shown.models[1].context, 32_768);
    }
    // A new server at another address has nothing added.
    let elsewhere = FoundServer {
        address: "http://localhost:11434".into(),
        ..found
    };
    assert!(list.found(None, elsewhere).models.iter().all(|m| !m.added));
}

#[test]
fn a_model_tried_has_its_trial_s_day_and_no_price() {
    let tried = OwnModel::tried(" gemma3:12b ", true, false, 32_768, 8_192, None, DAY);
    assert_eq!(tried.id, "gemma3:12b");
    assert!(tried.reads_pages && !tried.fills_forms);
    assert_eq!(tried.tried_at.as_deref(), Some(DAY));
    assert_eq!((tried.input_per_mtok, tried.output_per_mtok), (None, None));
    assert!(tried.model_info(&ProviderId::new("own-0000abcd")).is_some());
}

/// The address Settings keeps and the address `servers.json` may hold are one rule: what
/// `checked_address` keeps is never refused by the list's checks, and an address it refuses
/// never passes them.
#[test]
fn the_list_takes_exactly_the_addresses_kalamu_keeps() {
    let with = |address: &str| OwnServers {
        servers: vec![OwnServer {
            id: ProviderId::new("own-0000abcd"),
            name: "Mac".into(),
            address: address.into(),
            dialect: Dialect::Ollama,
            key: false,
            at_once: 1,
            models: Vec::new(),
        }],
    };
    for typed in [
        "http://localhost:11434",
        "HTTP://LocalHost:11434/",
        "https://llm.uni.example/v1",
        "http://127.1",
        "http://[::1]:11434",
        "http://192.168.1.20:11434/api/",
        "",
        "localhost:11434",
        "ftp://host",
        "http://",
        "http://:11434",
        "http://me@host",
        "http://host/?x=1",
        "http://host/#x",
        "http://host lan",
        "http://host\\x",
        "http://host:99999",
        "not an address",
    ] {
        let kept = checked_address(typed);
        assert_eq!(
            kept.is_ok(),
            with(typed).problems().is_empty(),
            "{typed:?}: {:?}",
            with(typed).problems()
        );
        if let Ok(kept) = kept {
            assert!(with(&kept).problems().is_empty(), "{kept}");
        }
    }
}
