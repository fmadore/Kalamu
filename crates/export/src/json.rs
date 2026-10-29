use kalamu_core::{ExportDocument, ExportError, ExportFormat, Exporter, Task};
use serde_json::json;

/// Everything, machine-readable: source, set-up, each part with its corrections and segments,
/// and the provenance record. The schema is documented in `docs/formats.md`.
pub struct JsonExporter;

pub const SCHEMA: &str = "kalamu-result";
pub const SCHEMA_VERSION: u32 = 1;

impl Exporter for JsonExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Json
    }

    fn supports(&self, _task: Task) -> bool {
        true
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let parts: Vec<_> = doc
            .parts
            .iter()
            .map(|p| {
                let mut part = json!({
                    "piece": p.piece,
                    "label": p.piece.label(),
                    "text": p.text,
                    "segments": p.segments,
                    "corrected": p.corrected,
                    "reviewed": p.reviewed,
                });
                // A piece that could not be read: its place kept, with the reason.
                if let Some(reason) = &p.failed {
                    part["failed"] = json!(reason);
                }
                // Only analyses have fields: their answers by key, and the reasons given. Other
                // documents keep their version 1 shape.
                // A table of contents' entries, each an object of its fields' values.
                if doc.task == Task::Contents {
                    let entries: Vec<serde_json::Value> = kalamu_core::contents::entries(&p.fields)
                        .iter()
                        .map(|entry| {
                            json!(
                                entry
                                    .iter()
                                    .map(|a| (a.key.clone(), json!(a.value)))
                                    .collect::<serde_json::Map<_, _>>()
                            )
                        })
                        .collect();
                    part["entries"] = json!(entries);
                } else if !p.fields.is_empty() {
                    let fields: serde_json::Map<_, _> = p
                        .fields
                        .iter()
                        .map(|a| (a.key.clone(), json!(a.value)))
                        .collect();
                    part["fields"] = json!(fields);
                    let reasons: serde_json::Map<_, _> = p
                        .fields
                        .iter()
                        .filter_map(|a| Some((a.key.clone(), json!(a.reason.as_ref()?))))
                        .collect();
                    if !reasons.is_empty() {
                        part["reasons"] = json!(reasons);
                    }
                }
                part
            })
            .collect();
        let value = json!({
            "schema": SCHEMA,
            "schema_version": SCHEMA_VERSION,
            "task": doc.task,
            "source": {
                "file_name": doc.source.file_name,
                "sha256": doc.source.sha256,
                "size_bytes": doc.source.size_bytes,
            },
            "setup": {
                "label": doc.setup.label,
                "provider": doc.setup.provider,
                "model": doc.setup.model,
                "prompt": doc.setup.prompt.preset_name,
            },
            "parts": parts,
            "provenance": doc.provenance,
        });
        serde_json::to_vec_pretty(&value).map_err(|e| ExportError::Write {
            format: "JSON",
            message: e.to_string(),
        })
    }
}
