//! Export profiles saved for reuse, one JSON file each like pipelines, and the file that shares
//! one as a template. A template never carries a project's values: only its columns, and the
//! fixed texts when the researcher asks for them.

use std::path::Path;

use kalamu_core::provenance::GeneratorInfo;
use kalamu_core::{ExportProfile, Timestamp};
use serde::{Deserialize, Serialize};

use crate::PromptError;
use crate::folder::{Folder, Saved};

const FILE_FORMAT: &str = "kalamu-export-profile";

impl Saved for ExportProfile {
    const PREFIX: &'static str = "profile-";
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// A profile as a file to share.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileFile {
    pub format: String,
    pub version: u32,
    pub generator: GeneratorInfo,
    pub exported_at: Timestamp,
    pub profile: ExportProfile,
}

pub struct ProfileLibrary(Folder<ExportProfile>);

impl ProfileLibrary {
    /// Loads saved profiles from `dir`, creating it if needed; an unreadable file is skipped.
    pub fn open(dir: &Path) -> Result<Self, PromptError> {
        Folder::open(dir).map(Self)
    }

    /// Nothing saved; for tests.
    pub fn in_memory() -> Self {
        Self(Folder::in_memory())
    }

    /// By name, ignoring case.
    pub fn list(&self) -> Vec<ExportProfile> {
        self.0.list()
    }

    pub fn get(&self, id: &str) -> Result<ExportProfile, PromptError> {
        self.0.get(id)
    }

    /// Saves a profile under its id, or a new one, once checked. Saving under a name already
    /// used replaces that profile, as a pipeline's name does.
    pub fn save(&mut self, profile: ExportProfile) -> Result<ExportProfile, PromptError> {
        let mut profile = profile.tidied();
        profile.check()?;
        profile.id = self.0.id_for(&profile)?;
        profile.updated_at = Timestamp::now();
        self.0.put(profile)
    }

    /// A copy under a name not yet taken.
    pub fn duplicate(&mut self, id: &str, name: &str) -> Result<ExportProfile, PromptError> {
        let mut copy = self.get(id)?;
        copy.id = String::new();
        copy.name = self.0.free_name(name);
        self.save(copy)
    }

    pub fn delete(&mut self, id: &str) -> Result<(), PromptError> {
        self.0.delete(id)
    }

    /// The profile as a file to share: its fixed texts left empty unless `fixed_texts`.
    pub fn file(
        &self,
        id: &str,
        fixed_texts: bool,
        app_version: &str,
    ) -> Result<String, PromptError> {
        let mut profile = self.get(id)?;
        if !fixed_texts {
            profile = profile.without_fixed_texts();
        }
        profile.id = String::new();
        let file = ProfileFile {
            format: FILE_FORMAT.into(),
            version: 1,
            generator: GeneratorInfo {
                name: "Kalamu".into(),
                version: app_version.into(),
            },
            exported_at: Timestamp::now(),
            profile,
        };
        serde_json::to_string_pretty(&file).map_err(crate::io)
    }

    /// Adds the profile in a shared file under a name not yet taken, once checked.
    pub fn import(&mut self, json: &str) -> Result<ExportProfile, PromptError> {
        let file: ProfileFile =
            serde_json::from_str(json).map_err(|e| PromptError::InvalidImport(e.to_string()))?;
        if file.format != FILE_FORMAT || file.version > 1 {
            return Err(PromptError::InvalidImport(file.format));
        }
        let mut profile = file.profile.tidied();
        profile.check()?;
        profile.id = String::new();
        profile.name = self.0.free_name(&profile.name);
        self.save(profile)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use kalamu_core::{
        DescriptionField, ProfileColumn, ProfileError, ProfileStarter, ProfileValue,
    };

    use super::*;

    fn profile(name: &str) -> ExportProfile {
        let mut columns = ProfileStarter::DublinCore.columns();
        columns.push(ProfileColumn {
            heading: "o:item_set".into(),
            value: ProfileValue::Fixed {
                text: "Fonds Mamadou Diallo".into(),
            },
        });
        ExportProfile {
            id: String::new(),
            name: name.into(),
            columns,
            list_separator: "|".into(),
            missing: "n.d.".into(),
            join_lines: true,
            updated_at: Timestamp(0),
        }
    }

    #[test]
    fn profiles_are_saved_duplicated_and_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let mut library = ProfileLibrary::open(dir.path()).unwrap();
        let saved = library.save(profile(" Omeka S ")).unwrap();
        assert_eq!(saved.name, "Omeka S");
        assert!(saved.id.starts_with("profile-"));
        let copy = library.duplicate(&saved.id, "Omeka S").unwrap();
        assert_eq!(copy.name, "Omeka S (2)");
        assert_ne!(copy.id, saved.id);
        assert_eq!(copy.columns, saved.columns);
        let reopened = ProfileLibrary::open(dir.path()).unwrap();
        assert_eq!(reopened.list().len(), 2);
        library.delete(&copy.id).unwrap();
        assert_eq!(ProfileLibrary::open(dir.path()).unwrap().list().len(), 1);
    }

    #[test]
    fn a_profile_that_fails_its_check_is_not_saved() {
        let mut library = ProfileLibrary::in_memory();
        let mut twice = profile("Twice");
        twice.columns[1].heading = " DCTERMS:identifier".into();
        assert!(matches!(
            library.save(twice),
            Err(PromptError::Profile(ProfileError::Duplicate {
                column: 2,
                first: 1
            }))
        ));
        let mut forged = profile("Forged");
        forged.id = "../settings".into();
        assert!(library.save(forged).is_err());
        assert!(library.list().is_empty());
    }

    #[test]
    fn a_shared_profile_leaves_its_fixed_texts_out_unless_asked() {
        let mut library = ProfileLibrary::in_memory();
        let saved = library.save(profile("Omeka S")).unwrap();
        let template = library.file(&saved.id, false, "1.3.0").unwrap();
        assert!(!template.contains("Mamadou"));
        assert!(
            !template.contains(&saved.id),
            "the id stays on this computer"
        );
        let imported = library.import(&template).unwrap();
        assert_eq!(imported.name, "Omeka S (2)");
        assert_eq!(imported.columns.len(), saved.columns.len());
        assert_eq!(imported.missing, "n.d.");
        assert_eq!(
            imported.columns[0].value,
            ProfileValue::Part {
                field: DescriptionField::Shelfmark
            }
        );
        let whole = library.file(&saved.id, true, "1.3.0").unwrap();
        assert!(whole.contains("Fonds Mamadou Diallo"));
        assert!(matches!(
            library.import("{\"format\":\"kalamu-workflow\"}"),
            Err(PromptError::InvalidImport(_))
        ));
    }
}
