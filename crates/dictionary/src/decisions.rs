//! The researcher's decisions: merging forms and entries into one, taking forms out again,
//! naming an entry, marking it reviewed, and keeping two apart. Each is kept, survives a
//! restart, and can be undone by its opposite; none touches a result.

use kalamu_core::{
    DictionaryError, Entry, EntryId, KeptApart, ProjectId, Timestamp, dictionary::form_of,
};

use crate::inventory::holder;
use crate::{Dictionary, Item};

/// A label with its spaces tidied, refused when nothing is left.
fn labelled(label: &str) -> Result<String, DictionaryError> {
    form_of(label).ok_or(DictionaryError::NoLabel)
}

impl Dictionary {
    /// One of the project's entries.
    fn entry(&self, project: ProjectId, id: EntryId) -> Result<Entry, DictionaryError> {
        self.check_project(project)?;
        self.store
            .entries(project)?
            .into_iter()
            .find(|e| e.id == id)
            .ok_or(DictionaryError::NoEntry)
    }

    /// Takes a form out of whichever entry of the namespace holds it, removing the entry when
    /// it was its last.
    fn release(
        &self,
        project: ProjectId,
        namespace: &str,
        form: &str,
    ) -> Result<(), DictionaryError> {
        let entries = self.store.entries(project)?;
        if let Some(entry) = holder(&entries, namespace, form) {
            let mut entry = entry.clone();
            entry.forms.retain(|f| f != form);
            if entry.forms.is_empty() {
                self.store.delete_entry(entry.id)?;
            } else {
                entry.updated_at = Timestamp::now();
                self.store.put_entry(&entry)?;
            }
        }
        Ok(())
    }

    /// The entry an item stands for, made from the form alone (unreviewed) when no entry holds
    /// it, so a decision about a form has an id to keep.
    fn materialise(
        &self,
        project: ProjectId,
        namespace: &str,
        item: &Item,
    ) -> Result<Entry, DictionaryError> {
        match item {
            Item::Entry { id } => {
                let entry = self.entry(project, *id)?;
                if entry.namespace != namespace {
                    return Err(DictionaryError::Namespaces);
                }
                Ok(entry)
            }
            Item::Form { form } => {
                let entries = self.store.entries(project)?;
                if let Some(entry) = holder(&entries, namespace, form) {
                    return Ok(entry.clone());
                }
                let now = Timestamp::now();
                let entry = Entry {
                    id: EntryId::new(),
                    project_id: project,
                    namespace: namespace.to_string(),
                    label: form.clone(),
                    forms: vec![form.clone()],
                    reviewed: false,
                    created_at: now,
                    updated_at: now,
                };
                self.store.put_entry(&entry)?;
                Ok(entry)
            }
        }
    }

    /// Makes entries and forms of one field one entry under `label`, marked reviewed. The
    /// oldest entry keeps its id (and what it was kept apart from); forms held by entries not
    /// chosen leave them. Refused when two of them were kept apart.
    pub fn merge(
        &self,
        project: ProjectId,
        namespace: &str,
        items: &[Item],
        label: &str,
    ) -> Result<Entry, DictionaryError> {
        self.check_project(project)?;
        let label = labelled(label)?;
        if items.len() < 2 {
            return Err(DictionaryError::TooFew);
        }
        let mut chosen: Vec<Entry> = Vec::new();
        let mut loose: Vec<String> = Vec::new();
        for item in items {
            match item {
                Item::Entry { id } => {
                    let entry = self.entry(project, *id)?;
                    if entry.namespace != namespace {
                        return Err(DictionaryError::Namespaces);
                    }
                    if !chosen.iter().any(|e| e.id == entry.id) {
                        chosen.push(entry);
                    }
                }
                Item::Form { form } => {
                    let form = form_of(form).ok_or(DictionaryError::NoLabel)?;
                    if !loose.contains(&form) {
                        loose.push(form);
                    }
                }
            }
        }
        let apart = self.store.apart(project)?;
        for (i, a) in chosen.iter().enumerate() {
            for b in &chosen[i + 1..] {
                if apart.contains(&KeptApart::new(a.id, b.id)) {
                    return Err(DictionaryError::KeptApart);
                }
            }
        }
        // Loose forms another entry holds leave it, unless that entry is merged whole.
        for form in &loose {
            let entries = self.store.entries(project)?;
            if holder(&entries, namespace, form)
                .is_some_and(|e| !chosen.iter().any(|c| c.id == e.id))
            {
                self.release(project, namespace, form)?;
            }
        }
        // The store lists entries in the order they were made, which a timestamp cannot tell
        // apart within a millisecond.
        let made: Vec<EntryId> = self.store.entries(project)?.iter().map(|e| e.id).collect();
        chosen.sort_by_key(|e| made.iter().position(|id| *id == e.id));
        let now = Timestamp::now();
        let mut kept = match chosen.first() {
            Some(first) => first.clone(),
            None => Entry {
                id: EntryId::new(),
                project_id: project,
                namespace: namespace.to_string(),
                label: label.clone(),
                forms: Vec::new(),
                reviewed: true,
                created_at: now,
                updated_at: now,
            },
        };
        for form in chosen.iter().flat_map(|e| &e.forms).chain(&loose) {
            if !kept.forms.contains(form) {
                kept.forms.push(form.clone());
            }
        }
        kept.label = label;
        kept.reviewed = true;
        kept.updated_at = now;
        self.store.put_entry(&kept)?;
        for gone in chosen.iter().skip(1) {
            for pair in apart.iter().filter(|p| p.holds(gone.id)) {
                let other = if pair.a == gone.id { pair.b } else { pair.a };
                self.store
                    .put_apart(project, &KeptApart::new(kept.id, other))?;
            }
            self.store.delete_entry(gone.id)?;
        }
        Ok(kept)
    }

    /// Takes forms out of an entry: each is its own again, not yet reviewed. An entry left with
    /// no form is removed, which undoes a merge.
    pub fn split(
        &self,
        project: ProjectId,
        id: EntryId,
        forms: &[String],
    ) -> Result<(), DictionaryError> {
        let mut entry = self.entry(project, id)?;
        entry.forms.retain(|f| !forms.contains(f));
        if entry.forms.is_empty() {
            self.store.delete_entry(id)?;
        } else {
            entry.updated_at = Timestamp::now();
            self.store.put_entry(&entry)?;
        }
        Ok(())
    }

    /// Gives an entry the label the researcher prefers, which need not be one of its forms.
    pub fn relabel(
        &self,
        project: ProjectId,
        id: EntryId,
        label: &str,
    ) -> Result<Entry, DictionaryError> {
        let mut entry = self.entry(project, id)?;
        entry.label = labelled(label)?;
        entry.updated_at = Timestamp::now();
        self.store.put_entry(&entry)?;
        Ok(entry)
    }

    /// Marks rows reviewed, or not; a form no entry holds becomes an entry of its own.
    pub fn set_reviewed(
        &self,
        project: ProjectId,
        namespace: &str,
        items: &[Item],
        reviewed: bool,
    ) -> Result<(), DictionaryError> {
        self.check_project(project)?;
        for item in items {
            if !reviewed && matches!(item, Item::Form { .. }) {
                continue;
            }
            let mut entry = self.materialise(project, namespace, item)?;
            entry.reviewed = reviewed;
            entry.updated_at = Timestamp::now();
            self.store.put_entry(&entry)?;
        }
        Ok(())
    }

    /// Records that two rows are not the same, however alike they look; a form becomes an
    /// entry of its own to carry the decision.
    pub fn keep_apart(
        &self,
        project: ProjectId,
        namespace: &str,
        a: &Item,
        b: &Item,
    ) -> Result<(), DictionaryError> {
        self.check_project(project)?;
        let a = self.materialise(project, namespace, a)?;
        let b = self.materialise(project, namespace, b)?;
        if a.id == b.id {
            return Err(DictionaryError::SameEntry);
        }
        self.store.put_apart(project, &KeptApart::new(a.id, b.id))?;
        Ok(())
    }

    /// Withdraws a decision to keep two entries apart.
    pub fn undo_apart(
        &self,
        project: ProjectId,
        a: EntryId,
        b: EntryId,
    ) -> Result<(), DictionaryError> {
        self.check_project(project)?;
        self.store.remove_apart(&KeptApart::new(a, b))?;
        Ok(())
    }
}
