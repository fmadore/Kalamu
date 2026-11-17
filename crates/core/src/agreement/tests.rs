use super::*;
use crate::{AnalysisSchema, Field, PromptSnapshot, ProviderId, SetupOptions};

fn close(actual: Option<f64>, expected: f64) {
    match actual {
        Some(a) => assert!((a - expected).abs() < 1e-9, "{a} is not {expected}"),
        None => panic!("no value, expected {expected}"),
    }
}

/// `count` pieces answered `a` by the first set-up and `b` by the second.
fn repeat(pairs: &[(usize, &'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    pairs
        .iter()
        .flat_map(|&(count, a, b)| std::iter::repeat_n((a, b), count))
        .collect()
}

#[test]
fn kappa_matches_the_published_example() {
    // Wikipedia's "Cohen's kappa", first worked example: 50 grant proposals, two readers
    // saying yes or no. Both yes 20, A yes and B no 5, A no and B yes 10, both no 15:
    // pₒ = 0.7, pₑ = 0.5 · 0.6 + 0.5 · 0.4 = 0.5, κ = 0.4.
    let choices = repeat(&[
        (20, "Yes", "Yes"),
        (5, "Yes", "No"),
        (10, "No", "Yes"),
        (15, "No", "No"),
    ]);
    let (agreement, kappa) = choice_agreement(choices);
    close(agreement, 0.7);
    close(kappa, 0.4);
}

#[test]
fn kappa_is_low_when_agreement_is_mostly_chance() {
    // Wikipedia's second example: the same 60% agreement as chance nearly reaches it.
    // pₑ = 0.6 · 0.7 + 0.4 · 0.3 = 0.54, κ = 0.06 / 0.46.
    let choices = repeat(&[
        (45, "Yes", "Yes"),
        (15, "Yes", "No"),
        (25, "No", "Yes"),
        (15, "No", "No"),
    ]);
    let (agreement, kappa) = choice_agreement(choices);
    close(agreement, 0.6);
    close(kappa, 0.06 / 0.46);
}

#[test]
fn kappa_over_several_values_counted_by_hand() {
    // Five pieces, three agreeing. The first gave Positive 3, Neutral 1, Mixed 1; the second
    // Positive 2, Neutral 3. Chance: (3 · 2 + 1 · 3 + 1 · 0) / 25 = 9/25, so
    // κ = (3/5 − 9/25) / (1 − 9/25) = 6/16.
    let choices = [
        ("Positive", "Positive"),
        ("Positive", "Neutral"),
        ("Neutral", "Neutral"),
        ("Mixed", "Neutral"),
        ("Positive", "Positive"),
    ];
    let (agreement, kappa) = choice_agreement(choices);
    close(agreement, 0.6);
    close(kappa, 0.375);
}

#[test]
fn choices_agree_whatever_their_capitals_and_accents() {
    let (agreement, kappa) = choice_agreement([("Négatif", "negatif"), ("Positif", " POSITIF ")]);
    close(agreement, 1.0);
    close(kappa, 1.0);
}

#[test]
fn kappa_cannot_be_computed_when_every_answer_is_one_value() {
    // Chance agreement is 1: kappa would divide by zero. Agreement still shows.
    let (agreement, kappa) = choice_agreement(repeat(&[(12, "Neutral", "Neutral")]));
    close(agreement, 1.0);
    assert_eq!(kappa, None);
}

#[test]
fn kappa_is_zero_or_below_when_the_two_never_agree() {
    // Each set-up always gives its own value: chance agreement 0, observed 0.
    let (agreement, kappa) = choice_agreement(repeat(&[(4, "Positive", "Negative")]));
    close(agreement, 0.0);
    close(kappa, 0.0);
    // Each gives both values, never together: worse than chance.
    let (agreement, kappa) = choice_agreement(repeat(&[
        (2, "Positive", "Negative"),
        (2, "Negative", "Positive"),
    ]));
    close(agreement, 0.0);
    close(kappa, -1.0);
}

#[test]
fn nothing_counted_gives_no_value() {
    assert_eq!(choice_agreement([]), (None, None));
}

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn overlap_is_the_items_both_listed_over_those_either_listed() {
    let a = names(&["Bamako", "Dakar"]);
    let b = names(&["bamako", "Ségou"]);
    assert!((overlap(&a, &b) - 1.0 / 3.0).abs() < 1e-9);
    // Capitals, accents and punctuation aside, an item counted once.
    let a = names(&["Ségou", "Traoré, Moussa", "Segou"]);
    let b = names(&["SEGOU", "Traore Moussa"]);
    assert!((overlap(&a, &b) - 1.0).abs() < 1e-9);
}

#[test]
fn two_empty_lists_agree_and_one_empty_list_does_not() {
    assert!((overlap(&[], &[]) - 1.0).abs() < 1e-9);
    assert!(
        (overlap(&names(&["  ", "—"]), &[]) - 1.0).abs() < 1e-9,
        "blank items are nothing"
    );
    assert!(overlap(&names(&["Dakar"]), &[]).abs() < 1e-9);
}

fn field(key: &str, kind: FieldKind) -> Field {
    Field {
        key: key.into(),
        label: key.to_uppercase(),
        instructions: String::new(),
        kind,
        reason: false,
    }
}

fn setup(fields: Vec<Field>) -> Setup {
    Setup {
        id: SetupId::new(),
        label: "a".into(),
        provider: ProviderId::new("gemini"),
        model: "gemini-flash".into(),
        prompt: PromptSnapshot {
            preset_id: None,
            preset_name: "Tone".into(),
            template: String::new(),
            text: String::new(),
            user_prompt: String::new(),
            schema: Some(AnalysisSchema { fields }),
        },
        options: SetupOptions::default(),
        merge: None,
        pages_cap: None,
    }
}

fn tone() -> FieldKind {
    FieldKind::Choice {
        values: names(&["Positive", "Neutral", "Negative"]),
    }
}

fn answers(tone: &str, places: &[&str], summary: &str) -> Vec<FieldAnswer> {
    vec![
        FieldAnswer {
            key: "summary".into(),
            value: FieldValue::Text(summary.into()),
            reason: None,
        },
        FieldAnswer {
            key: "tone".into(),
            value: FieldValue::Text(tone.into()),
            reason: Some("Because.".into()),
        },
        FieldAnswer {
            key: "places".into(),
            value: FieldValue::List(names(places)),
            reason: None,
        },
    ]
}

#[test]
fn a_job_is_counted_field_by_field_and_pair_by_pair() {
    let all = || {
        vec![
            field("summary", FieldKind::Text),
            field("tone", tone()),
            field("places", FieldKind::List { max: None }),
        ]
    };
    // The third set-up does not ask for places.
    let setups = vec![
        setup(all()),
        setup(all()),
        setup(vec![
            field("summary", FieldKind::Text),
            field("tone", tone()),
        ]),
    ];
    let mut tally = AgreementTally::new(&setups);
    assert!(!tally.is_empty());

    let one = [
        answers("Neutral", &["Dakar", "Bamako"], "A meeting."),
        answers("Neutral", &["dakar"], "A meeting in Dakar."),
        answers("Positive", &[], "Cordial."),
    ];
    let two = [
        answers("Negative", &[], "x"),
        answers("Positive", &[], "y"),
        answers("Negative", &[], "z"),
    ];
    // The second set-up failed this piece; the third has not reached it.
    let three = [answers("Positive", &["Ségou"], "x")];
    tally.add(&one.iter().map(|a| Some(a.as_slice())).collect::<Vec<_>>());
    tally.add(&two.iter().map(|a| Some(a.as_slice())).collect::<Vec<_>>());
    tally.add(&[Some(three[0].as_slice()), None, None]);
    let job = tally.finish();

    // Free text is not compared.
    let keys: Vec<&str> = job.fields.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, ["tone", "places"]);

    let tone = &job.fields[0];
    assert_eq!(tone.label, "TONE");
    assert_eq!(tone.measure, AgreementMeasure::Choice);
    let pairs: Vec<(SetupId, SetupId)> = tone.pairs.iter().map(|p| (p.first, p.second)).collect();
    assert_eq!(
        pairs,
        [
            (setups[0].id, setups[1].id),
            (setups[0].id, setups[2].id),
            (setups[1].id, setups[2].id)
        ]
    );
    // a and b: Neutral/Neutral, Negative/Positive. Agreement 1/2; chance (1 · 1 + 1 · 0 +
    // 0 · 1) / 4 = 1/4, so κ = (1/2 − 1/4) / (3/4) = 1/3.
    assert_eq!(tone.pairs[0].pieces, 2);
    close(tone.pairs[0].agreement, 0.5);
    close(tone.pairs[0].kappa, 1.0 / 3.0);
    // a and c: Neutral/Positive, Negative/Negative.
    assert_eq!(tone.pairs[1].pieces, 2);
    close(tone.pairs[1].agreement, 0.5);

    // Places: only the first two ask for them. Piece one: {dakar, bamako} and {dakar}, 1/2;
    // piece two: both empty, 1. Piece three is missing from the second.
    let places = &job.fields[1];
    assert_eq!(places.measure, AgreementMeasure::Overlap);
    assert_eq!(places.pairs.len(), 1);
    assert_eq!(places.pairs[0].pieces, 2);
    close(places.pairs[0].agreement, 0.75);
    assert_eq!(places.pairs[0].kappa, None);
}

#[test]
fn a_field_no_piece_answered_by_both_shows_no_value() {
    let setups = vec![
        setup(vec![field("tone", tone())]),
        setup(vec![field("tone", tone())]),
    ];
    let mut tally = AgreementTally::new(&setups);
    let a = answers("Neutral", &[], "");
    tally.add(&[Some(a.as_slice()), None]);
    let job = tally.finish();
    let pair = &job.fields[0].pairs[0];
    assert_eq!((pair.pieces, pair.agreement, pair.kappa), (0, None, None));
}

#[test]
fn nothing_is_compared_without_two_set_ups_asking_a_choice_or_a_list() {
    let text = || vec![field("summary", FieldKind::Text)];
    assert!(AgreementTally::new(&[setup(text()), setup(text())]).is_empty());
    assert!(AgreementTally::new(&[setup(vec![field("tone", tone())])]).is_empty());
    // The same key asked as a choice by one and a list by the other is not compared.
    let list = setup(vec![field("tone", FieldKind::List { max: None })]);
    assert!(AgreementTally::new(&[setup(vec![field("tone", tone())]), list]).is_empty());
    assert_eq!(AgreementTally::new(&[]).finish(), JobAgreement::default());
}
