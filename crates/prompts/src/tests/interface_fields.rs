//! The default prompts' fields as shipped, written for the interface as ts-rs writes the types:
//! `cargo test` keeps the file current and CI checks it is committed. The interface shows a
//! default field's label, help and choices in its own language only where they are, word for
//! word, the text in this file, and its tests check that each has a translation.

use std::collections::BTreeMap;

use kalamu_core::Field;

use super::*;

#[test]
fn export_default_fields_for_the_interface() -> Result<(), Box<dyn std::error::Error>> {
    let fields: BTreeMap<String, Vec<Field>> = PromptLibrary::in_memory()
        .list(None)
        .into_iter()
        // The dictionary's prompt is read and edited in Prompts only: its answer is never shown
        // as fields.
        .filter(|p| p.builtin && p.id != DICTIONARY_MERGES)
        .filter_map(|p| p.schema.map(|s| (p.id, s.fields)))
        .collect();
    assert!(fields.contains_key("analysis.entities"));
    kalamu_core::test_support::write_interface_json("src/lib/i18n/defaultFields.json", &fields)
}
