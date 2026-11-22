//! Reading queries and placing their terms: modes, alternatives, words of two letters, and
//! the rules written for the interface.

use kalamu_core::{JobId, PassageKey, PassagePart, SetupId, SourceId};

use super::*;
use crate::fold::{MIN_QUERY, UNSPACED};

/// What a query looks for as each mode reads it, group by group, and what it leaves out.
fn groups(query: &str, mode: MatchMode) -> (Vec<Vec<String>>, Vec<String>) {
    let q = parse(query, mode);
    let mut groups = vec![Vec::new(); q.groups];
    for t in q.terms {
        groups[t.group].push(t.typed);
    }
    (groups, q.ignored)
}

/// The shortest word searched for, the operators and the scripts written without spaces,
/// with queries and what each looks for, written for the interface, which reads queries
/// alike and waits for a word long enough before searching: `cargo test` keeps the file
/// current, as it does `taskRules.json`.
#[test]
fn export_query_rules_for_the_interface() -> Result<(), Box<dyn std::error::Error>> {
    let queries = [
        "pè",
        "Père",
        "  de  ",
        "de la",
        "le pape",
        "\"de la\"",
        "« un »",
        "ab, cd.",
        "Ḥajj",
        "à",
        "l'",
        "北京",
        "北京市",
        "imam OR marabout Koupéla",
        "« mission catholique » OU « mission protestante »",
        "Mali OR",
        "OR Mali",
        "x OR y",
        "Mali or Ghana",
        "a OR OR b",
        "\"OR\" Mali",
        "(Bâ OR Sy)",
    ];
    let modes = [MatchMode::Phrase, MatchMode::AllWords, MatchMode::Near];
    let cases: Vec<serde_json::Value> = queries
        .iter()
        .flat_map(|q| {
            modes.iter().map(move |m| {
                let (groups, ignored) = groups(q, *m);
                serde_json::json!({
                    "query": q,
                    "mode": m,
                    "searchable": !groups.is_empty(),
                    "groups": groups,
                    "ignored": ignored,
                })
            })
        })
        .collect();
    let rules = serde_json::json!({
        "min_query": MIN_QUERY,
        "operators": OPERATORS,
        "unspaced": UNSPACED.iter().map(|(a, b)| [a, b]).collect::<Vec<_>>(),
        "cases": cases,
    });
    kalamu_core::test_support::write_interface_json("src/lib/searchRules.json", &rules)
}

fn passage(texts: &[&str]) -> Passage {
    Passage {
        key: PassageKey {
            job_id: JobId::new(),
            setup_id: SetupId::new(),
            source_id: SourceId::new(),
            index: 0,
        },
        stamp: String::new(),
        piece: PieceRef::page(1),
        reviewed: false,
        corrected: false,
        parts: texts
            .iter()
            .map(|t| PassagePart {
                field: None,
                entry: None,
                entry_number: None,
                reason: false,
                text: (*t).into(),
                lines: Vec::new(),
                by_hand: Vec::new(),
            })
            .collect(),
    }
}

/// Each match listed as `term#nth: words [marked]`.
fn shown(texts: &[&str], query: &str, mode: MatchMode, near: u32) -> Vec<String> {
    let q = parse(query, mode);
    let mut found = hits(&passage(texts), &q, mode, near, &HashMap::new());
    found.sort_by_key(|f| f.order);
    found
        .into_iter()
        .map(Found::listed)
        .map(|h| {
            let chars: Vec<char> = h.words.chars().collect();
            let marked: Vec<String> = h
                .marks
                .iter()
                .map(|m| chars[m.start as usize..m.end as usize].iter().collect())
                .collect();
            format!("{}#{}: {} [{}]", h.term, h.nth, h.words, marked.join("|"))
        })
        .collect()
}

#[test]
fn words_are_read_as_the_mode_asks() {
    let q = parse("  la mission catholique ", MatchMode::Phrase);
    assert_eq!(q.terms.len(), 1);
    assert_eq!(q.terms[0].typed, "la mission catholique");
    assert!(q.ignored.is_empty());

    let q = parse(
        "« mission catholique » Koupéla, la à UCM ucm",
        MatchMode::AllWords,
    );
    let typed: Vec<&str> = q.terms.iter().map(|t| t.typed.as_str()).collect();
    // Two letters are a word; one is too short.
    assert_eq!(typed, ["mission catholique", "Koupéla", "la", "UCM"]);
    assert_eq!(q.ignored, ["à"]);
    assert_eq!(
        q.index_terms(),
        [
            vec!["mission catholique"],
            vec!["koupela"],
            vec![" la "],
            vec!["ucm"]
        ]
    );
    // Quotes left open keep the rest together.
    let q = parse("\"union culturelle musulmane", MatchMode::Near);
    assert_eq!(q.terms[0].typed, "union culturelle musulmane");
}

#[test]
fn or_and_ou_in_capitals_give_alternatives() {
    let s = |v: &[&[&str]]| -> Vec<Vec<String>> {
        v.iter()
            .map(|g| g.iter().map(|t| t.to_string()).collect())
            .collect()
    };
    // Between two words, the second joins the first; the others stay required.
    assert_eq!(
        groups("imam OR marabout Koupéla", MatchMode::AllWords).0,
        s(&[&["imam", "marabout"], &["Koupéla"]])
    );
    assert_eq!(
        groups("Koupéla imam OU marabout OR cheikh", MatchMode::Near).0,
        s(&[&["Koupéla"], &["imam", "marabout", "cheikh"]])
    );
    // Quoted phrases are alternatives too, and an operator between quotes is a word.
    assert_eq!(
        groups(
            "« mission catholique » OR \"mission protestante\" Koupéla",
            MatchMode::AllWords
        )
        .0,
        s(&[&["mission catholique", "mission protestante"], &["Koupéla"]])
    );
    assert_eq!(
        groups("\"Mali OR Ghana\"", MatchMode::AllWords).0,
        s(&[&["Mali OR Ghana"]])
    );
    // In a phrase, each run of words between operators is one phrase.
    assert_eq!(
        groups(
            "  union culturelle   OR ligue islamique OU « jeunesse musulmane » ",
            MatchMode::Phrase
        )
        .0,
        s(&[&["union culturelle", "ligue islamique", "jeunesse musulmane"]])
    );
    // In small letters, a word like any other; with nothing on one side, left out.
    assert_eq!(
        groups("Mali or Ghana", MatchMode::AllWords).0,
        s(&[&["Mali"], &["or"], &["Ghana"]])
    );
    assert_eq!(groups("Mali OR", MatchMode::Phrase).0, s(&[&["Mali"]]));
    assert_eq!(groups("OR Mali OR", MatchMode::AllWords).0, s(&[&["Mali"]]));
    // Punctuation around an operator set aside, as around a word.
    assert_eq!(
        groups("(Bâ OR Sy)", MatchMode::AllWords).0,
        s(&[&["Bâ", "Sy"]])
    );
    // A short alternative is left out and named, its group kept by the others.
    assert_eq!(
        groups("x OR Mali", MatchMode::AllWords),
        (s(&[&["Mali"]]), vec!["x".to_string()])
    );
    assert!(groups("x OR y", MatchMode::Near).0.is_empty());
    // The index is asked for a term of each group.
    assert_eq!(
        parse("Bâ OR Sy Dakar", MatchMode::AllWords).index_terms(),
        [vec![" ba ", " sy "], vec!["dakar"]]
    );
}

#[test]
fn each_alternative_found_is_marked_and_opened_on() {
    let texts = [
        "Le marabout de Koupéla reçoit, bien plus tard dans la journée, l'imam.",
        "L'imam seul.",
        "Le cheikh de Koupéla.",
    ];
    // Any alternative with the other group, each listed under its own words.
    assert_eq!(
        shown(&texts, "imam OR marabout koupela", MatchMode::AllWords, 0),
        [
            "marabout#0: marabout [marabout]",
            "koupela#0: Koupéla [Koupéla]",
            "imam#0: imam [imam]",
        ]
    );
    // Near: the alternative that stands close is the one marked and opened on.
    assert_eq!(
        shown(&texts, "imam OR cheikh koupela", MatchMode::Near, 2),
        ["cheikh#0: cheikh de Koupéla [cheikh|Koupéla]"]
    );
    // A phrase or another.
    assert_eq!(
        shown(
            &["La mission catholique et la Mission protestante"],
            "mission catholique OR mission protestante",
            MatchMode::Phrase,
            0
        ),
        [
            "mission catholique#0: mission catholique [mission catholique]",
            "mission protestante#0: Mission protestante [Mission protestante]",
        ]
    );
}

#[test]
fn a_word_of_two_letters_is_found_on_its_own_and_counted_as_the_review_counts() {
    // `nth` counts Bamako's "ba" as the review's search does, so it opens on the right one.
    assert_eq!(
        shown(
            &["Bamako : Amadou Bâ et (BA), Kaba"],
            "ba",
            MatchMode::Phrase,
            0
        ),
        ["ba#1: Bâ [Bâ]", "ba#2: BA [BA]"]
    );
    // A part with the letters only inside words holds no match.
    assert!(shown(&["Bamako, Kaba"], "Bâ", MatchMode::Phrase, 0).is_empty());
    assert_eq!(
        shown(
            &["Amadou Bâ et Mamadou Sy"],
            "Bâ OR Sy",
            MatchMode::Phrase,
            0
        ),
        ["Bâ#0: Bâ [Bâ]", "Sy#0: Sy [Sy]"]
    );
    // Near each other, with another word.
    assert_eq!(
        shown(&["Le cheikh Ba, à Dakar"], "ba dakar", MatchMode::Near, 2),
        ["ba#0: Ba, à Dakar [Ba|Dakar]"]
    );
    // Arabic: a word on its own, not the letters joined to the next.
    assert_eq!(
        shown(&["فيه كتاب في المسجد"], "في", MatchMode::Phrase, 0),
        ["في#1: في [في]"]
    );
}

#[test]
fn every_word_anywhere_in_a_part_each_place_its_own_match() {
    let texts = [
        "La mission de Koupéla. Plus loin, la mission encore.",
        "Koupéla seule",
        "Mission et KOUPELA",
    ];
    assert_eq!(
        shown(&texts, "koupela mission", MatchMode::AllWords, 0),
        [
            "mission#0: mission [mission]",
            "koupela#0: Koupéla [Koupéla]",
            "mission#1: mission [mission]",
            // The second part holds one word only, but its match is counted.
            "mission#2: Mission [Mission]",
            "koupela#2: KOUPELA [KOUPELA]",
        ]
    );
}

#[test]
fn words_near_each_other_within_the_distance_asked() {
    let text = "L'imam Ouédraogo a parlé. Bien plus tard, dans un autre quartier de la \
                ville, le grand imam est venu, et Ouédraogo aussi.";
    // Two words apart at most: the first imam only.
    assert_eq!(
        shown(&[text], "imam ouedraogo", MatchMode::Near, 2),
        ["imam#0: imam Ouédraogo [imam|Ouédraogo]"]
    );
    // Further apart, the second pair too, each word marked and the words between not.
    assert_eq!(
        shown(&[text], "imam ouedraogo", MatchMode::Near, 4),
        [
            "imam#0: imam Ouédraogo [imam|Ouédraogo]",
            "imam#1: imam est venu, et Ouédraogo [imam|Ouédraogo]",
        ]
    );
    // In either order.
    assert_eq!(
        shown(
            &["Ouédraogo, dit l'imam"],
            "imam ouedraogo",
            MatchMode::Near,
            2
        ),
        ["ouedraogo#0: Ouédraogo, dit l'imam [Ouédraogo|imam]"]
    );
}

#[test]
fn near_marks_follow_the_words_shown_across_line_breaks() {
    assert_eq!(
        shown(
            &["union\n\n  des   musulmans"],
            "union musulmans",
            MatchMode::Near,
            1
        ),
        ["union#0: union des musulmans [union|musulmans]"]
    );
}

#[test]
fn a_phrase_is_one_term_marked_whole() {
    assert_eq!(
        shown(&["UCM, ucm et Ucm"], "ucm", MatchMode::Phrase, 0),
        ["ucm#0: UCM [UCM]", "ucm#1: ucm [ucm]", "ucm#2: Ucm [Ucm]"]
    );
}
