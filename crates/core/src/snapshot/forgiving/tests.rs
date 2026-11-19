use super::*;
use crate::PieceRef;
use crate::snapshot::{PartText, build, slice};

/// The text at each place found.
fn found(text: &str, quote: &str) -> Vec<String> {
    find(text, quote)
        .into_iter()
        .map(|(s, e)| slice(text, s, e).unwrap_or_default().to_string())
        .collect()
}

#[test]
fn an_exact_copy_is_found_where_it_stands() {
    let text = "Awa: Nous partions tôt au marché.";
    assert_eq!(found(text, "Nous partions tôt"), ["Nous partions tôt"]);
    let (s, e) = find(text, "Nous partions tôt")[0];
    assert_eq!((s, e), (5, 22));
    assert!(find(text, "absent").is_empty());
}

#[test]
fn white_space_of_any_kind_and_length_is_forgiven() {
    let text = "Nous\u{a0}partions\n  tôt,\u{202f}le\tmatin";
    assert_eq!(
        found(text, "Nous partions tôt, le matin"),
        ["Nous\u{a0}partions\n  tôt,\u{202f}le\tmatin"]
    );
    assert_eq!(found("un deux", "un   deux"), ["un deux"]);
    // But a space is never invented where the text has none, nor taken away.
    assert!(find("undeux", "un deux").is_empty());
    assert!(find("un deux", "undeux").is_empty());
}

#[test]
fn french_guillemets_with_their_inner_spaces_match_straight_or_curly_quotes() {
    let text = "Il a dit\u{a0}: «\u{a0}On ne savait pas\u{a0}» et il est parti.";
    for quote in [
        "dit\u{a0}: \"On ne savait pas\" et il",
        "dit : “On ne savait pas” et il",
        "dit : « On ne savait pas » et il",
    ] {
        assert_eq!(
            found(text, quote),
            ["dit\u{a0}: «\u{a0}On ne savait pas\u{a0}» et il"],
            "{quote}"
        );
    }
    // Apostrophes of any kind.
    let text = "C’était l’année de l’indépendance.";
    assert_eq!(
        found(text, "C'était l'année"),
        ["C’était l’année"],
        "a straight apostrophe matches a curly one"
    );
    assert_eq!(found("l'école", "l’école"), ["l'école"]);
}

#[test]
fn an_ellipsis_character_matches_three_dots_either_way() {
    assert_eq!(
        found("On attendait… puis rien.", "attendait... puis"),
        ["attendait… puis"]
    );
    assert_eq!(
        found("On attendait... puis rien.", "attendait… puis"),
        ["attendait... puis"]
    );
}

#[test]
fn what_a_model_wraps_a_quotation_in_is_forgiven_and_left_out() {
    let text = "Le chef a dit que la route serait finie avant la saison des pluies.";
    for quote in [
        "\"la route serait finie\"",
        "« la route serait finie »",
        "…la route serait finie…",
        "...la route serait finie...",
        "[…] la route serait finie […]",
        "‘la route serait finie’",
    ] {
        assert_eq!(found(text, quote), ["la route serait finie"], "{quote}");
    }
}

#[test]
fn a_final_punctuation_mark_is_forgiven_and_the_texts_own_kept() {
    let text = "Il pleuvait. Nous sommes restés.";
    // Added by the model where the text has none.
    assert_eq!(found(text, "Il pleuvait, nous"), Vec::<String>::new());
    assert_eq!(
        found("Il pleuvait beaucoup", "Il pleuvait."),
        ["Il pleuvait"]
    );
    // The text's own kept when the model gave it.
    assert_eq!(found(text, "Il pleuvait."), ["Il pleuvait."]);
    assert_eq!(found(text, "Nous sommes restés"), ["Nous sommes restés"]);
    // After the no-break space French puts before it.
    assert_eq!(
        found("Vraiment\u{202f}?", "Vraiment ?"),
        ["Vraiment\u{202f}?"]
    );
}

#[test]
fn nothing_else_is_forgiven() {
    let text = "Nous partions tôt au marché.";
    // Case, accents, a word changed, missing or added, an elision in the middle.
    for quote in [
        "nous partions tôt",
        "Nous partions tot",
        "Nous partions tard",
        "Nous tôt au marché",
        "Nous partions très tôt",
        "Nous partions […] marché",
        "Nous partions tôt au marché ce jour-là",
    ] {
        assert!(find(text, quote).is_empty(), "{quote} must not be found");
    }
    // Nothing quoted, or only punctuation and marks.
    for quote in ["", "   ", "...", "« »", "."] {
        assert!(find(text, quote).is_empty(), "{quote:?} quotes nothing");
    }
}

#[test]
fn arabic_is_found_in_characters() {
    let text = "قال الإمام: السلام عليكم ورحمة الله.";
    let at = find(text, "«السلام عليكم»");
    assert_eq!(at.len(), 1);
    assert_eq!(slice(text, at[0].0, at[0].1), Some("السلام عليكم"));
    // Its own final mark, kept when given.
    assert_eq!(found("هل أنت هنا؟ نعم", "هل أنت هنا؟"), ["هل أنت هنا؟"]);
}

#[test]
fn a_repeated_phrase_is_found_each_time_and_each_anchor_in_its_piece() {
    let pages = [PieceRef::page(1), PieceRef::page(2)];
    let (text, parts) = build(&[
        PartText {
            piece: &pages[0],
            text: "On ne savait pas. Puis on a su.",
            reviewed: false,
            corrected: false,
            failed: false,
        },
        PartText {
            piece: &pages[1],
            text: "Au début, «\u{a0}on ne savait pas\u{a0}».",
            reviewed: false,
            corrected: false,
            failed: false,
        },
    ]);
    let anchors = locate(&text, &parts, "on ne savait pas");
    assert_eq!(anchors.len(), 1, "case is not forgiven");
    assert_eq!(anchors[0].part, 1);
    let anchors = locate(&text, &parts, "ne savait pas");
    assert_eq!(anchors.len(), 2);
    assert_eq!((anchors[0].part, anchors[1].part), (0, 1));
    for anchor in &anchors {
        let own = slice(&text, anchor.start, anchor.end).unwrap_or_default();
        assert_eq!(crate::snapshot::check(&text, &parts, anchor, own), Ok(()));
    }
    // A quotation from the end of one page into the next is in neither.
    assert!(locate(&text, &parts, "on a su. Au début").is_empty());
}

#[test]
fn a_quotation_opening_with_the_texts_own_mark_keeps_it() {
    let text = "Ensuite, «\u{a0}on ne savait pas\u{a0}» est devenu notre devise.";
    assert_eq!(
        found(text, "« on ne savait pas » est devenu notre devise"),
        ["«\u{a0}on ne savait pas\u{a0}» est devenu notre devise"]
    );
    assert_eq!(
        found(text, "\"on ne savait pas\" est devenu"),
        ["«\u{a0}on ne savait pas\u{a0}» est devenu"]
    );
    // Marks a model wraps around words the text has without them are still forgiven.
    assert_eq!(
        found(text, "« est devenu notre devise »"),
        ["est devenu notre devise"]
    );
}
