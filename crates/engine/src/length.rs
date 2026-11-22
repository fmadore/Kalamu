//! A text an analysis's model surely cannot read, refused when the job is made, before anything
//! is sent or recorded, and named by the estimate before Start: a cited analysis's text one
//! passage of which is too long even read in parts ([`Engine::too_long_at`]), and any other
//! analysis's text longer than its model reads at once, since that text is sent whole. Measured
//! as the catalogue measures it ([`kalamu_catalog::ModelInfo::too_long`]), never by a provider's
//! or a model's name; a model the catalogue gives no limit, and a text that cannot be read now,
//! are left to the provider and the run.

use kalamu_catalog::Catalog;
use kalamu_core::{InputKind, SetupRequest, SourceFile, Task};

use crate::setup::NewJob;
use crate::{Engine, EngineError, TextTooLong};

impl Engine {
    /// Refuses a new analysis holding a text one of its set-ups' models cannot read
    /// ([`Engine::unreadable_texts`]), with the first such text and the model's name.
    pub(crate) fn check_length(&self, catalog: &Catalog, new: &NewJob) -> Result<(), EngineError> {
        if new.task != Task::Analysis {
            return Ok(());
        }
        for request in &new.setups {
            let cites = self.request_cites(new.task, request)?;
            if let Some((_, long)) = self
                .unreadable_texts(catalog, new.task, request, cites, &new.sources)
                .into_iter()
                .next()
            {
                let model = catalog
                    .model(&request.provider, &request.model)
                    .map_or_else(|| request.model.clone(), |m| m.name.clone());
                return Err(EngineError::TextTooLong {
                    file: long.file,
                    model,
                    words: long.words,
                    most: long.most_words,
                });
            }
        }
        Ok(())
    }

    /// The texts an analysis's set-up cannot read, each with its place among the sources, in the
    /// order given: for a set-up that cites the text (`cites`), those it cannot read even in
    /// parts; for any other, those longer than its model reads at once. None for another task.
    pub(crate) fn unreadable_texts(
        &self,
        catalog: &Catalog,
        task: Task,
        request: &SetupRequest,
        cites: bool,
        sources: &[SourceFile],
    ) -> Vec<(usize, TextTooLong)> {
        match (task, cites) {
            (Task::Analysis, true) => self.too_long_at(catalog, request, sources),
            (Task::Analysis, false) => too_long_whole(catalog, request, sources),
            _ => Vec::new(),
        }
    }
}

/// The text sources an analysis that cites nothing cannot send, each read whole in one request
/// and longer than its model reads at once.
fn too_long_whole(
    catalog: &Catalog,
    request: &SetupRequest,
    sources: &[SourceFile],
) -> Vec<(usize, TextTooLong)> {
    let Some(model) = catalog.model(&request.provider, &request.model) else {
        return Vec::new();
    };
    let Some(most) = model.most_chars() else {
        return Vec::new();
    };
    sources
        .iter()
        .enumerate()
        .filter(|(_, s)| s.kind == InputKind::Text)
        // A file of no more bytes than the model reads characters surely fits, unread.
        .filter(|(_, s)| s.size_bytes == 0 || s.size_bytes > most)
        .filter_map(|(at, source)| {
            let text = kalamu_media::TextSource::open(std::path::Path::new(&source.path))
                .and_then(|t| t.text())
                .ok()?;
            let long = model.too_long_chars(text.chars().count() as u64)?;
            Some((
                at,
                TextTooLong {
                    source: source.id,
                    file: source.file_name.clone(),
                    words: long.words,
                    most_words: long.most_words,
                },
            ))
        })
        .collect()
}
