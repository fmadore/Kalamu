//! A region's reading accepted: as the text of a region the researcher drew, or in place of an
//! engine's region's words in the page's text, undone as it was, refused when those words
//! changed or the reading has no place to go; listed in provenance only once accepted.

use kalamu_core::provenance::ReadingPlace;
use kalamu_core::{ProposalStatus, ReadingEffect, ReadingTarget, RegionAsk, RegionReadingError};

use super::*;

#[tokio::test]
async fn a_reading_accepted_into_a_drawn_region_is_its_text_until_undone() {
    let s = setting().await;
    s.engine
        .ask_region_reading(s.ask("g1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    let before = s
        .engine
        .accept_region_reading("g1", "Note en marge : vu le 3 mai 1956".into())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.drawn_text(), "Note en marge : vu le 3 mai 1956");
    // The page's text is not touched, and the region's former state is given back.
    assert_eq!(s.shown().as_deref(), Some(TEXT));
    assert!(matches!(&before, ReadingEffect::Region(r) if r.text.is_empty()));
    // Accepted once only.
    assert!(matches!(
        s.engine.accept_region_reading("g1", "x".into()),
        Err(EngineError::RegionReading(RegionReadingError::NotWaiting))
    ));
    // Provenance names the model, the region and where the reading went, not the text.
    let doc = s
        .engine
        .export_document(&s.job, s.job.setups[0].id, s.job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let readings = &doc.provenance.region_readings;
    assert_eq!(readings.len(), 1);
    assert_eq!(readings[0].region, s.drawn);
    assert_eq!(readings[0].model, "reader-model");
    assert_eq!(readings[0].went_to, ReadingPlace::Region);
    assert!(!readings[0].edited);
    let json = serde_json::to_string(&doc.provenance).unwrap_or_default();
    assert!(!json.contains("vu le 3 mai"));
    // Undone: the region as it was, and no longer in provenance.
    s.engine
        .undo_region_reading("g1", before)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(s.drawn_text().is_empty());
    assert_eq!(
        s.store.region_reading("g1").map(|r| r.status).ok(),
        Some(ProposalStatus::Undone)
    );
    let doc = s
        .engine
        .export_document(&s.job, s.job.setups[0].id, s.job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(doc.provenance.region_readings.is_empty());
    let json = serde_json::to_value(&doc.provenance).unwrap_or_default();
    assert!(json.get("region_readings").is_none());
}

#[tokio::test]
async fn a_reading_accepted_into_the_page_s_text_replaces_the_region_s_words() {
    let s = setting().await;
    let model_text = {
        let piece = s.store.pieces(s.job.id).unwrap_or_default().remove(0);
        s.store
            .finished_result(&piece)
            .ok()
            .flatten()
            .map(|r| r.output.text)
    };
    *s.reader.answer.lock().unwrap_or_else(|p| p.into_inner()) = "deux cents fidèles".into();
    s.engine
        .ask_region_reading(s.ask_text("g1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    let before = s
        .engine
        .accept_region_reading("g1", "deux cents fidèles musulmans".into())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        s.shown().as_deref(),
        Some("Le Gouverneur général a reçu ce matin deux cents fidèles musulmans de la ville.")
    );
    // Left reviewed as it was; the model's own text untouched; the region drawn untouched.
    assert!(s.store.corrections(s.job.id).unwrap_or_default()[0].reviewed);
    let piece = s.store.pieces(s.job.id).unwrap_or_default().remove(0);
    assert_eq!(
        s.store
            .finished_result(&piece)
            .ok()
            .flatten()
            .map(|r| r.output.text),
        model_text
    );
    assert!(s.drawn_text().is_empty());
    let doc = s
        .engine
        .export_document(&s.job, s.job.setups[0].id, s.job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        doc.provenance.region_readings[0].went_to,
        ReadingPlace::Text
    );
    assert!(doc.provenance.region_readings[0].edited);
    // Not counted with the pieces' own results.
    assert_eq!(doc.provenance.cost_usd, {
        let mut plain = doc.provenance.clone();
        plain.region_readings.clear();
        plain.cost_usd
    });
    assert!(matches!(&before, ReadingEffect::Correction(c) if c.text.as_deref() == Some(TEXT)));
    s.engine
        .undo_region_reading("g1", before)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(s.shown().as_deref(), Some(TEXT));
}

#[tokio::test]
async fn a_reading_whose_words_changed_meanwhile_is_not_applied() {
    let s = setting().await;
    s.engine
        .ask_region_reading(s.ask_text("g1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    // The researcher corrected the same words by hand while the model answered.
    let typed = TEXT.replace("deux cents fideles", "200 fidèles");
    s.engine
        .save_correction(Correction {
            job_id: s.job.id,
            setup_id: s.job.setups[0].id,
            source_id: s.job.sources[0].id,
            index: 0,
            text: Some(typed.clone()),
            fields: None,
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(
        s.engine
            .accept_region_reading("g1", "deux cents fidèles".into()),
        Err(EngineError::RegionReading(RegionReadingError::Changed))
    ));
    assert_eq!(s.shown(), Some(typed));
    // Asking about it again is refused before anything is cut or sent.
    let err = s.engine.ask_region_reading(s.ask_text("g2")).await.err();
    assert_eq!(err.map(|e| e.code()), Some("region_reading.changed"));
    assert_eq!(s.calls(), 1);
}

#[tokio::test]
async fn a_reading_with_no_place_in_the_text_can_be_read_not_written() {
    let s = setting().await;
    let ask = RegionAsk {
        region_id: "r2".into(),
        target: ReadingTarget::Nowhere,
        ..s.ask("g1")
    };
    let reading = s
        .engine
        .ask_region_reading(ask)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(reading.status, ProposalStatus::Answered);
    let err = s.engine.accept_region_reading("g1", "x".into()).err();
    assert_eq!(err.map(|e| e.code()), Some("region_reading.nowhere"));
    assert_eq!(s.shown().as_deref(), Some(TEXT));
    s.engine
        .discard_region_reading("g1")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        s.store.region_reading("g1").map(|r| r.status).ok(),
        Some(ProposalStatus::Discarded)
    );
    let doc = s
        .engine
        .export_document(&s.job, s.job.setups[0].id, s.job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(doc.provenance.region_readings.is_empty());
}

#[tokio::test]
async fn an_undo_that_does_not_fit_the_reading_is_refused() {
    let s = setting().await;
    s.engine
        .ask_region_reading(s.ask("g1"))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    let before = s
        .engine
        .accept_region_reading("g1", "Note".into())
        .unwrap_or_else(|e| panic!("{e}"));
    // A correction offered to undo a region's reading: refused, nothing changed.
    let wrong = ReadingEffect::Correction(Correction {
        job_id: s.job.id,
        setup_id: s.job.setups[0].id,
        source_id: s.job.sources[0].id,
        index: 0,
        text: None,
        fields: None,
        reviewed: false,
        updated_at: Timestamp::now(),
    });
    assert!(s.engine.undo_region_reading("g1", wrong).is_err());
    assert_eq!(s.drawn_text(), "Note");
    assert_eq!(s.shown().as_deref(), Some(TEXT));
    s.engine
        .undo_region_reading("g1", before.clone())
        .unwrap_or_else(|e| panic!("{e}"));
    // Undone once only.
    assert!(s.engine.undo_region_reading("g1", before).is_err());
}
