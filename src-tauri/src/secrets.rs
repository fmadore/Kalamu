//! API keys in Windows Credential Manager, one entry per provider, adapted from Live
//! Translation's `secrets.rs`. Keys are read on the Rust side only, when a request needs them;
//! they never reach the interface, the job database, logs or provenance.
//!
//! In debug builds a `GEMINI_API_KEY`-style environment variable is accepted as a fallback for
//! development.

use std::collections::HashMap;
use std::sync::Mutex;

use kalamu_core::{ApiKey, ProviderId};
use kalamu_engine::KeySource;

use crate::errors::{AppError, id};

/// The bundle identifier.
const SERVICE: &str = "io.github.fmadore.kalamu";
/// The identifier before the app was renamed from Scriptorium (0.1 and 0.2). A key still
/// stored under it is moved to `SERVICE` the first time it is read.
const LEGACY_SERVICE: &str = "io.github.fmadore.scriptorium";

fn account(provider: &ProviderId) -> String {
    format!("{}-api-key", provider.as_str())
}

fn env_var(provider: &ProviderId) -> String {
    match provider.as_str() {
        "anthropic" => "ANTHROPIC_API_KEY".into(),
        other => format!("{}_API_KEY", other.to_ascii_uppercase()),
    }
}

/// The first character no provider issues in a key, with its 1-based position. The message
/// names the character, never the key, since error lines get screenshotted.
fn check_format(key: &str) -> Result<(), String> {
    match key.chars().zip(1..).find(|(c, _)| !matches!(c, '!'..='~')) {
        Some((c, pos)) => Err(format!(
            "character U+{:04X} at position {pos}",
            u32::from(c)
        )),
        None => Ok(()),
    }
}

/// Keys from Credential Manager, cached in memory so a job does not hit the keychain on every
/// page.
#[derive(Default)]
pub struct CredentialKeys {
    cache: Mutex<HashMap<ProviderId, Option<String>>>,
}

impl CredentialKeys {
    fn entry(provider: &ProviderId) -> Result<keyring::Entry, AppError> {
        Self::entry_in(SERVICE, provider)
    }

    fn entry_in(service: &str, provider: &ProviderId) -> Result<keyring::Entry, AppError> {
        keyring::Entry::new(service, &account(provider)).map_err(|e| AppError::new(id::KEYCHAIN, e))
    }

    fn stored(service: &str, provider: &ProviderId) -> Option<String> {
        Self::entry_in(service, provider)
            .ok()?
            .get_password()
            .ok()
            .filter(|key| !key.trim().is_empty())
    }

    fn read(provider: &ProviderId) -> Option<String> {
        if let Some(key) = Self::stored(SERVICE, provider) {
            return Some(key);
        }
        if let Some(key) = Self::stored(LEGACY_SERVICE, provider) {
            // Moved, not copied, so the key lives in one place; left where it was if the
            // new entry cannot be written.
            if Self::entry(provider).is_ok_and(|e| e.set_password(&key).is_ok())
                && let Ok(old) = Self::entry_in(LEGACY_SERVICE, provider)
            {
                let _ = old.delete_credential();
            }
            return Some(key);
        }
        if cfg!(debug_assertions) {
            return std::env::var(env_var(provider))
                .ok()
                .filter(|k| !k.trim().is_empty());
        }
        None
    }

    pub fn set(&self, provider: &ProviderId, key: &str) -> Result<(), AppError> {
        let key = key.trim();
        if key.is_empty() {
            return Err(AppError::new(id::KEY_FORMAT, "the key is empty"));
        }
        check_format(key).map_err(|detail| AppError::new(id::KEY_FORMAT, detail))?;
        Self::entry(provider)?
            .set_password(key)
            .map_err(|e| AppError::new(id::KEYCHAIN, e))?;
        self.lock().insert(provider.clone(), Some(key.to_string()));
        Ok(())
    }

    pub fn clear(&self, provider: &ProviderId) -> Result<(), AppError> {
        // The legacy entry too, so a key not yet moved does not come back.
        for service in [SERVICE, LEGACY_SERVICE] {
            match Self::entry_in(service, provider)?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(e) => return Err(AppError::new(id::KEYCHAIN, e)),
            }
        }
        self.lock().insert(provider.clone(), None);
        Ok(())
    }

    pub fn has(&self, provider: &ProviderId) -> bool {
        self.key(provider).is_some()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<ProviderId, Option<String>>> {
        self.cache.lock().unwrap_or_else(|p| p.into_inner())
    }
}

impl KeySource for CredentialKeys {
    fn key(&self, provider: &ProviderId) -> Option<ApiKey> {
        let mut cache = self.lock();
        let entry = cache
            .entry(provider.clone())
            .or_insert_with(|| Self::read(provider));
        entry.clone().map(ApiKey::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_as_providers_issue_them_pass() {
        for key in [
            // Joined at compile time so secret scanners do not flag a made-up key.
            concat!("AIza", "SyA-1b2C3d4E5f6G7h8I9j0K_lMnOpQrStU"),
            "sk-proj-Ab3dEf_gH1jK-lMn0pQ",
            concat!("sk-ant-", "api03-abc_DEF-123"),
        ] {
            assert_eq!(check_format(key), Ok(()), "{key}");
        }
    }

    #[test]
    fn a_pasted_space_or_invisible_character_is_named_without_the_key() {
        let err = check_format("sk-secret\u{200B}value")
            .err()
            .unwrap_or_default();
        assert_eq!(err, "character U+200B at position 10");
        assert!(!err.contains("secret"));
        assert!(check_format("sk-abc def").is_err());
    }

    #[test]
    fn accounts_and_variables_follow_the_provider_id() {
        assert_eq!(account(&ProviderId::new("gemini")), "gemini-api-key");
        assert_eq!(env_var(&ProviderId::new("anthropic")), "ANTHROPIC_API_KEY");
        assert_eq!(env_var(&ProviderId::new("openai")), "OPENAI_API_KEY");
    }
}
