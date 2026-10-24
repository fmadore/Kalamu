use scriptorium_core::{ExportDocument, ExportError, ExportFormat, Exporter, Task};

/// Markdown with a short header naming the source and model, and a heading per page.
pub struct MarkdownExporter;

impl Exporter for MarkdownExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Markdown
    }

    fn supports(&self, _task: Task) -> bool {
        true
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let mut out = format!(
            "# {}\n\n*{} · {} · prompt: {}*\n",
            doc.source.file_name, doc.setup.provider, doc.setup.model, doc.setup.prompt.preset_name
        );
        for i in 0..doc.parts.len() {
            out.push('\n');
            if let Some(marker) = crate::marker(doc, i) {
                out.push_str(&format!("## {marker}\n\n"));
            }
            out.push_str(&crate::part_text(doc, i));
            out.push('\n');
        }
        Ok(out.into_bytes())
    }
}
