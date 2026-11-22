//! The shipped defaults' ids, in the order New job lists them: splitting or adding to their
//! files must leave this list as it is.

use super::*;

#[test]
fn the_defaults_keep_their_ids_in_their_order() {
    let ids: Vec<&str> = defaults::DEFAULTS.iter().map(|d| d.id).collect();
    assert_eq!(
        ids,
        [
            "text_extraction.printed",
            "text_extraction.newspaper",
            "text_extraction.handwritten",
            "audio_transcription.interview",
            "audio_transcription.translate_english",
            "analysis.summary_research",
            "analysis.summary_publication",
            "analysis.entities",
            "analysis.tone",
            "analysis.image_description",
            "analysis.directory",
            "analysis.register",
            "analysis.interview",
            "analysis.screening",
            "contents.periodical",
            "contents.archival_file",
            "contents.book",
            "text_correction.general",
            "text_correction.handwritten",
            "translation.research",
        ]
    );
}
