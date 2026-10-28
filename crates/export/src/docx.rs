use std::io::Cursor;

use docx_rs::{BreakType, Docx, Paragraph, Run};
use kalamu_core::{ExportDocument, ExportError, ExportFormat, Exporter, Task};

/// A Word document: title, a line naming the model, then each page under a small heading.
/// Line breaks inside paragraphs are kept, since diplomatic transcriptions rely on them.
pub struct DocxExporter;

impl Exporter for DocxExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Docx
    }

    fn supports(&self, _task: Task) -> bool {
        true
    }

    fn export(&self, doc: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let mut docx = Docx::new()
            .add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&doc.source.file_name).bold().size(32)),
            )
            .add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text(format!(
                            "{} · {} · prompt: {}",
                            doc.setup.provider, doc.setup.model, doc.setup.prompt.preset_name
                        ))
                        .italic()
                        .size(18)
                        .color("666666"),
                ),
            );
        for i in 0..doc.parts.len() {
            if let Some(marker) = crate::marker(doc, i) {
                docx = docx.add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(marker).bold().size(24)),
                );
            }
            for block in crate::part_text(doc, i).split("\n\n") {
                let mut run = Run::new();
                for (n, line) in block.lines().enumerate() {
                    if n > 0 {
                        run = run.add_break(BreakType::TextWrapping);
                    }
                    run = run.add_text(line);
                }
                docx = docx.add_paragraph(Paragraph::new().add_run(run));
            }
        }
        let mut out = Cursor::new(Vec::new());
        docx.build()
            .pack(&mut out)
            .map_err(|e| ExportError::Write {
                format: "Word",
                message: e.to_string(),
            })?;
        Ok(out.into_inner())
    }
}
