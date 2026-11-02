//! A folder of saved definitions, one JSON file each under its id: what pipelines and workflows
//! share. Saving under a name already used replaces that definition, as saving a file under its
//! name would; an id that is not one this folder gave is refused, so no path can be built from
//! one.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{PromptError, io, write_atomic};

/// A definition kept in a [`Folder`].
pub(crate) trait Saved: Clone + Serialize + DeserializeOwned {
    /// Begins every id this kind is given (`pipeline-`).
    const PREFIX: &'static str;
    fn id(&self) -> &str;
    fn name(&self) -> &str;
}

pub(crate) struct Folder<T> {
    dir: Option<PathBuf>,
    saved: BTreeMap<String, T>,
}

impl<T: Saved> Folder<T> {
    /// Loads what is saved in `dir`, creating it if needed; an unreadable file is skipped.
    pub(crate) fn open(dir: &Path) -> Result<Self, PromptError> {
        fs::create_dir_all(dir).map_err(io)?;
        let mut saved = BTreeMap::new();
        for entry in fs::read_dir(dir).map_err(io)? {
            let path = entry.map_err(io)?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(item) = fs::read_to_string(&path)
                .map_err(io)
                .and_then(|s| serde_json::from_str::<T>(&s).map_err(io))
            {
                saved.insert(item.id().to_string(), item);
            }
        }
        Ok(Self {
            dir: Some(dir.to_path_buf()),
            saved,
        })
    }

    /// Nothing saved; for tests.
    pub(crate) fn in_memory() -> Self {
        Self {
            dir: None,
            saved: BTreeMap::new(),
        }
    }

    /// By name, ignoring case.
    pub(crate) fn list(&self) -> Vec<T> {
        let mut out: Vec<T> = self.saved.values().cloned().collect();
        out.sort_by_key(|item| item.name().to_lowercase());
        out
    }

    pub(crate) fn get(&self, id: &str) -> Result<T, PromptError> {
        self.saved
            .get(id)
            .cloned()
            .ok_or_else(|| PromptError::NotFound(id.to_string()))
    }

    /// The id `item` is saved under: its own, else that of a definition of the same name, else
    /// a new one.
    pub(crate) fn id_for(&self, item: &T) -> Result<String, PromptError> {
        let id = match item.id() {
            "" => self
                .saved
                .values()
                .find(|s| s.name().to_lowercase() == item.name().to_lowercase())
                .map(|s| s.id().to_string())
                .unwrap_or_else(|| format!("{}{}", T::PREFIX, uuid::Uuid::new_v4())),
            id => id.to_string(),
        };
        if uuid::Uuid::parse_str(id.trim_start_matches(T::PREFIX)).is_err() {
            return Err(PromptError::NotFound(id));
        }
        Ok(id)
    }

    /// Writes `item`, whose id came from [`Folder::id_for`].
    pub(crate) fn put(&mut self, item: T) -> Result<T, PromptError> {
        if let Some(dir) = &self.dir {
            write_atomic(&dir.join(format!("{}.json", item.id())), &item)?;
        }
        self.saved.insert(item.id().to_string(), item.clone());
        Ok(item)
    }

    pub(crate) fn delete(&mut self, id: &str) -> Result<(), PromptError> {
        if self.saved.remove(id).is_none() {
            return Err(PromptError::NotFound(id.to_string()));
        }
        if let Some(dir) = &self.dir {
            let path = dir.join(format!("{id}.json"));
            if path.exists() {
                fs::remove_file(path).map_err(io)?;
            }
        }
        Ok(())
    }

    /// `name`, or `name (2)`, `name (3)`… when it is taken: an import or a copy never replaces
    /// a definition of the same name.
    pub(crate) fn free_name(&self, name: &str) -> String {
        let taken = |n: &str| {
            self.saved
                .values()
                .any(|s| s.name().to_lowercase() == n.to_lowercase())
        };
        let base = name.trim().to_string();
        let mut out = base.clone();
        let mut n = 2;
        while taken(&out) {
            out = format!("{base} ({n})");
            n += 1;
        }
        out
    }

    /// What is saved under `id`, if anything.
    pub(crate) fn saved(&self, id: &str) -> Option<&T> {
        self.saved.get(id)
    }
}
