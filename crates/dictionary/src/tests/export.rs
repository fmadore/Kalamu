//! What leaves Kalamu: the exports, with ids, original forms and review status; and the
//! folding that only suggests look-alikes.

use super::*;

#[test]
fn exports_keep_ids_original_forms_and_review_status() {
    let f = fixture();
    let letter = f.document("letter");
    f.analysis(
        std::slice::from_ref(&letter),
        &[&[(&["Moussa Traoré", "Traore Moussa", "Imam Diallo"], &[])]],
    );
    let entry = f
        .dictionary
        .merge(
            f.project,
            "people",
            &[form("Moussa Traoré"), form("Traore Moussa")],
            "Moussa Traoré",
        )
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_str(&f.dictionary.export_json(f.project).unwrap()).unwrap();
    assert_eq!(json["format"], "kalamu-dictionary");
    assert_eq!(json["entries"][0]["id"], entry.id.to_string());
    assert_eq!(json["entries"][0]["forms"][1], "Traore Moussa");
    assert_eq!(json["entries"][0]["reviewed"], true);

    let csv = f.dictionary.export_csv(f.project).unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(
        lines[0],
        "document_id,document,job,setup,field,original,entry_id,label,reviewed"
    );
    assert!(lines.iter().any(|l| l.contains(&format!(
        "people,Traore Moussa,{},Moussa Traoré,yes",
        entry.id
    ))));
    assert!(lines.iter().any(|l| l.ends_with("people,Imam Diallo,,,no")));
}

#[test]
fn folding_sets_aside_capitals_accents_punctuation_and_word_order() {
    assert_eq!(folded("Côte d’Ivoire"), "cote d ivoire");
    assert_eq!(folded("CÔTE-D'IVOIRE"), "cote d ivoire");
    assert_eq!(
        word_order_key("Traoré Moussa"),
        word_order_key("moussa traore")
    );
    assert_eq!(word_order_key("Bamako"), None);
}
