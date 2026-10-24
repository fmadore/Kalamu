use scriptorium_core::{ExportDocument, ExportError, ExportFormat, Exporter, Task};

/// Plain text, with `--- Page N ---` lines between pages as in zmo-ai-pipelines.
pub struct TextExporter;

impl Exporter for TextExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Txt
    }

    fn supports(&self, _task: Task) -> bool {
        true
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let mut out = String::new();
        for i in 0..doc.parts.len() {
            if let Some(marker) = crate::marker(doc, i) {
                if !out.is_empty() {
                    out.push_str("\n\n");
                }
                out.push_str(&format!("--- {marker} ---\n\n"));
            } else if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(&crate::part_text(doc, i));
        }
        out.push('\n');
        Ok(out.into_bytes())
    }
}
