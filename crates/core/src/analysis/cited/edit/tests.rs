//! A quotation the researcher adds from a passage selected in the text: found in the text, its
//! words, turn, speaker and times the text's, marked as theirs, kept in their corrections only
//! while the text holds it there, and taken out as any.

use super::super::tests::{cited_answers, doc, transcript};
use super::*;

/// Where `words` start and end in the text, in characters, `pad` characters wider on each side.
fn span(text: &str, words: &str, pad: u32) -> (u32, u32) {
    let at = text
        .find(words)
        .unwrap_or_else(|| panic!("{words} not in the text"));
    let start = text[..at].chars().count() as u32;
    (start - pad, start + words.chars().count() as u32 + pad)
}

#[test]
fn a_passage_selected_is_added_as_the_researchers_quotation_with_the_turns_speaker_and_time() {
    let built = transcript();
    let (schema, model) = cited_answers();
    // The selection takes the space before the sentence: it is left out.
    let (start, end) = span(&built.text, "Au début, on ne savait pas.", 1);
    let add = CitedEdit::AddQuotation {
        entry: 1,
        start,
        end,
    };
    let mut added = edit_in(&schema.fields, &model, "themes", &add, Some(doc(&built)))
        .unwrap_or_else(|e| panic!("{e}"));
    let entry = &added[1]
        .value
        .cited()
        .unwrap_or_else(|| panic!("cited"))
        .entries[1];
    let mine = entry
        .evidence
        .last()
        .unwrap_or_else(|| panic!("no quotation"));
    assert_eq!(mine.quote, "Au début, on ne savait pas.");
    assert_eq!(mine.anchor.start, start + 1);
    assert_eq!((mine.turn, mine.speaker.as_deref()), (2, Some("Awa")));
    assert_eq!(mine.start_ms, Some(9_000));
    assert!(mine.added);
    assert_eq!(entry.evidence.len(), 2);
    // Added twice, it is there once.
    let again = edit_in(&schema.fields, &added, "themes", &add, Some(doc(&built)))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(again, added);

    // Kept while the text holds it there; never without the text, nor changed.
    assert_eq!(
        keeps_evidence_in(
            &schema.fields,
            &model,
            &mut added.clone(),
            Some(doc(&built))
        ),
        Ok(())
    );
    assert_eq!(
        keeps_evidence(&schema.fields, &model, &mut added.clone()),
        Err(CitedError::Invented("Themes".into()))
    );
    if let FieldValue::Cited(value) = &mut added[1].value
        && let Some(e) = value.entries[1].evidence.last_mut()
    {
        e.quote = "Au début, nous savions tout.".into();
    }
    assert_eq!(
        keeps_evidence_in(
            &schema.fields,
            &model,
            &mut added.clone(),
            Some(doc(&built))
        ),
        Err(CitedError::Invented("Themes".into()))
    );

    // Taken out as any quotation.
    let removed = edit_in(
        &schema.fields,
        &again,
        "themes",
        &CitedEdit::RemoveQuotation {
            entry: 1,
            quotation: 1,
        },
        Some(doc(&built)),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(removed, model);
}

#[test]
fn a_passage_across_two_turns_or_without_the_text_is_not_added() {
    let built = transcript();
    let (schema, model) = cited_answers();
    let (start, _) = span(&built.text, "on ne savait pas.", 0);
    let (_, end) = span(&built.text, "Et ensuite", 0);
    let across = CitedEdit::AddQuotation {
        entry: 0,
        start,
        end,
    };
    for doc in [Some(doc(&built)), None] {
        assert_eq!(
            edit_in(&schema.fields, &model, "themes", &across, doc),
            Err(CitedError::NotInTurn)
        );
    }
    // Spaces alone are no passage.
    let (blank, _) = span(&built.text, "Et ensuite", 1);
    let spaces = CitedEdit::AddQuotation {
        entry: 0,
        start: blank,
        end: blank + 1,
    };
    assert_eq!(
        edit_in(&schema.fields, &model, "themes", &spaces, Some(doc(&built))).map(|_| ()),
        Err(CitedError::NotInTurn)
    );
}
