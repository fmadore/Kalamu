//! Records analyses in the library: the two ready-made ones, and the researcher's own saved
//! with their columns keyed and checked.

use kalamu_core::{Field, FieldKind, records};

use super::*;

fn text_field(label: &str) -> Field {
    Field {
        key: String::new(),
        label: label.into(),
        instructions: String::new(),
        kind: FieldKind::Text,
        reason: false,
    }
}

#[test]
fn the_ready_made_records_read_every_entry_as_written() -> Result<(), PromptError> {
    let library = PromptLibrary::in_memory();
    for id in ["analysis.directory", "analysis.register"] {
        let preset = library.get(id)?;
        let schema = preset.schema.clone().unwrap_or_default();
        assert!(records::is_records(&schema), "{id}");
        let text = preset.snapshot(&RenderContext::default()).text;
        // The columns are described under their group, each by its key.
        for column in records::columns(&schema) {
            assert!(text.contains(&format!("(`{}`", column.key)), "{id}");
        }
        // Literal readings: repetition marks kept, nothing carried down.
        assert!(text.contains("ditto mark"), "{id}");
        assert!(text.contains("Do not carry a value down"), "{id}");
        assert!(preset.user_prompt.contains("this page"), "{id}");
    }
    Ok(())
}

#[test]
fn a_records_analysis_is_saved_with_its_columns_keyed_and_checked() -> Result<(), PromptError> {
    let mut library = PromptLibrary::in_memory();
    let mut own = library.get("analysis.directory")?;
    own.id = String::new();
    own.name = "Annuaire".into();
    if let Some(FieldKind::Entries { fields }) = own.schema.as_mut().map(|s| &mut s.fields[0].kind)
    {
        fields.push(text_field("Street"));
    }
    let saved = library.save(own.clone())?;
    let keys: Vec<String> = records::columns(saved.schema.as_ref().unwrap_or(&Default::default()))
        .iter()
        .map(|c| c.key.clone())
        .collect();
    assert_eq!(keys.last().map(String::as_str), Some("street"));

    // A field beside the group, and a group with no column, are refused with their own ids.
    let mut beside = own.clone();
    if let Some(schema) = beside.schema.as_mut() {
        schema.fields.push(text_field("Year"));
    }
    let refused = library.save(beside).err();
    assert_eq!(refused.map(|e| e.code()), Some("prompt.group_alone"));
    let mut empty = own;
    if let Some(FieldKind::Entries { fields }) =
        empty.schema.as_mut().map(|s| &mut s.fields[0].kind)
    {
        fields.clear();
    }
    let refused = library.save(empty).err();
    assert_eq!(refused.map(|e| e.code()), Some("prompt.empty_group"));
    Ok(())
}
