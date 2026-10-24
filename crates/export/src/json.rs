use scriptorium_core::{ExportDocument, ExportError, ExportFormat, Exporter, Task};
use serde_json::json;

/// Everything, machine-readable: source, set-up, each part with its corrections and segments,
/// and the provenance record. The schema is documented in `docs/formats.md`.
pub struct JsonExporter;

pub const SCHEMA: &str = "scriptorium-result";
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
                // Only summaries have keywords; other documents keep their version 1 shape.
                if !p.keywords.is_empty() {
                    part["keywords"] = json!(p.keywords);
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
