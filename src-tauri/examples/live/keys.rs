//! Where the keys come from, and what keeps them out of everything the tool writes.
//!
//! A provider's key is read from `KALAMU_<PROVIDER>_API_KEY` (`KALAMU_GEMINI_API_KEY`,
//! `KALAMU_OPENAI_API_KEY`, `KALAMU_ANTHROPIC_API_KEY`, `KALAMU_MISTRAL_API_KEY`) when that
//! variable is set and not blank, and otherwise from the system keychain, under the entries the
//! app writes, unless `LIVE_KEYCHAIN=off`. Every key is read once, at the start; none is ever
//! printed, and [`Redactor`] takes each one, and anything shaped like a key or an identifier, out
//! of what the tool prints and writes (the console, the summary, the fixtures).

use std::collections::BTreeMap;
use std::sync::Arc;

use kalamu_core::{ApiKey, ProviderId};
use kalamu_engine::KeySource;

/// The keychain's service, as the app writes it (`src/secrets.rs`).
pub const KEYCHAIN_SERVICE: &str = "io.github.fmadore.kalamu";

/// The environment variable that holds a provider's key: `KALAMU_GEMINI_API_KEY`.
pub fn env_name(provider: &ProviderId) -> String {
    let id: String = provider
        .as_str()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    format!("KALAMU_{id}_API_KEY")
}

/// How the system keychain is read for a provider; tests pass their own, or none.
pub type ReadKeychain<'a> = &'a dyn Fn(&ProviderId) -> Option<String>;

/// Where a key was found, which the tool may say; never the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Environment,
    Keychain,
}

impl Origin {
    pub fn words(self) -> &'static str {
        match self {
            Self::Environment => "environment",
            Self::Keychain => "keychain",
        }
    }
}

/// The keys found for the providers asked about, read once.
#[derive(Default)]
pub struct Keys {
    found: BTreeMap<String, (ApiKey, Origin)>,
}

impl Keys {
    /// Each provider's key: the environment's when set and not blank, else the keychain's
    /// when one is given. `env` and `keychain` are passed in so that tests read neither.
    pub fn gather(
        providers: &[ProviderId],
        env: impl Fn(&str) -> Option<String>,
        keychain: Option<ReadKeychain<'_>>,
    ) -> Self {
        let mut found = BTreeMap::new();
        for provider in providers {
            let from_env = env(&env_name(provider))
                .map(|k| k.trim().to_string())
                .filter(|k| !k.is_empty())
                .map(|k| (k, Origin::Environment));
            let from_keychain = || {
                keychain
                    .and_then(|read| read(provider))
                    .map(|k| k.trim().to_string())
                    .filter(|k| !k.is_empty())
                    .map(|k| (k, Origin::Keychain))
            };
            if let Some((key, origin)) = from_env.or_else(from_keychain) {
                found.insert(provider.as_str().to_string(), (ApiKey::new(key), origin));
            }
        }
        Self { found }
    }

    /// The system keychain's entry for a provider, as the app saved it.
    pub fn system_keychain(provider: &ProviderId) -> Option<String> {
        keyring::Entry::new(KEYCHAIN_SERVICE, &format!("{}-api-key", provider.as_str()))
            .and_then(|e| e.get_password())
            .ok()
    }

    pub fn origin(&self, provider: &ProviderId) -> Option<Origin> {
        self.found.get(provider.as_str()).map(|(_, o)| *o)
    }

    /// What [`Redactor`] must take out of every text.
    pub fn redactor(&self) -> Redactor {
        Redactor::new(
            self.found
                .values()
                .map(|(k, _)| k.expose().to_string())
                .collect(),
        )
    }
}

impl KeySource for Keys {
    fn key(&self, provider: &ProviderId) -> Option<ApiKey> {
        self.found.get(provider.as_str()).map(|(k, _)| k.clone())
    }
}

/// Takes the keys, and what looks like a key or an identifier, out of a text before it is
/// printed or written.
#[derive(Clone, Default)]
pub struct Redactor {
    secrets: Arc<Vec<String>>,
}

/// Prefixes of the keys the providers issue.
const KEY_PREFIXES: [&str; 4] = ["sk-", "sk_", "AIza", "key-"];
/// Prefixes of the identifiers they give requests, batches, files and accounts.
const ID_PREFIXES: [&str; 14] = [
    "req_",
    "msgbatch_",
    "batch_",
    "batch-",
    "file-",
    "file_",
    "org-",
    "proj_",
    "msg_",
    "resp_",
    "chatcmpl-",
    "toolu_",
    "user-",
    "job-",
];

impl Redactor {
    pub fn new(secrets: Vec<String>) -> Self {
        let mut secrets: Vec<String> = secrets.into_iter().filter(|s| s.len() >= 4).collect();
        // The longest first, so that a key holding another is taken whole.
        secrets.sort_by_key(|s| std::cmp::Reverse(s.len()));
        Self {
            secrets: Arc::new(secrets),
        }
    }

    /// `text` with every key the tool holds replaced by `[key]`, and any word shaped like a key
    /// or an identifier by `[key]` or `[id]`.
    pub fn text(&self, text: &str) -> String {
        let mut out = text.to_string();
        for secret in self.secrets.iter() {
            out = out.replace(secret.as_str(), "[key]");
        }
        scrub_words(&out)
    }

    /// [`Self::text`], cut to `max` characters, on one line: for a summary's cell.
    pub fn line(&self, text: &str, max: usize) -> String {
        let flat: String = self
            .text(text)
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect();
        let flat = flat.split_whitespace().collect::<Vec<_>>().join(" ");
        if flat.chars().count() <= max {
            return flat;
        }
        let cut: String = flat.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", cut.trim_end())
    }
}

/// Each run of letters, digits, `-` and `_` judged on its own.
fn scrub_words(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        if !word.is_empty() {
            out.push_str(&judge(word));
            word.clear();
        }
    };
    for c in text.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// What a word becomes: `[key]` for one shaped like a provider's key, `[id]` for an identifier
/// (a known prefix, a UUID, a long number such as a project's, a long run mixing letters and
/// digits), else itself. A model's name ("voxtral-mini-transcribe-3-0") is short in each part
/// and stays.
fn judge(word: &str) -> String {
    if word.len() >= 16 && KEY_PREFIXES.iter().any(|p| word.starts_with(p)) {
        return "[key]".into();
    }
    if word.len() >= 12 && ID_PREFIXES.iter().any(|p| word.starts_with(p)) {
        return "[id]".into();
    }
    if is_uuid(word) || (word.len() >= 10 && word.chars().all(|c| c.is_ascii_digit())) {
        return "[id]".into();
    }
    let mixed = |part: &str| {
        part.len() >= 16
            && part.chars().any(|c| c.is_ascii_digit())
            && part.chars().any(|c| c.is_ascii_alphabetic())
    };
    if word.split(['-', '_']).any(mixed) {
        return "[id]".into();
    }
    word.to_string()
}

fn is_uuid(word: &str) -> bool {
    let parts: Vec<&str> = word.split('-').collect();
    parts.len() == 5
        && parts
            .iter()
            .zip([8, 4, 4, 4, 12])
            .all(|(p, n)| p.len() == n && p.chars().all(|c| c.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> ProviderId {
        ProviderId::new(s)
    }

    #[test]
    fn variable_names_follow_the_provider_id() {
        assert_eq!(env_name(&id("gemini")), "KALAMU_GEMINI_API_KEY");
        assert_eq!(env_name(&id("openai")), "KALAMU_OPENAI_API_KEY");
        assert_eq!(env_name(&id("local-llm")), "KALAMU_LOCAL_LLM_API_KEY");
    }

    #[test]
    fn the_environment_comes_first_and_a_blank_variable_counts_as_unset() {
        let env = |name: &str| match name {
            "KALAMU_GEMINI_API_KEY" => Some("  env-gemini-key ".into()),
            "KALAMU_OPENAI_API_KEY" => Some("   ".into()),
            _ => None,
        };
        let keychain = |p: &ProviderId| Some(format!("keychain-{}-key", p.as_str()));
        let providers = [id("gemini"), id("openai"), id("anthropic")];
        let keys = Keys::gather(&providers, env, Some(&keychain));
        assert_eq!(keys.origin(&id("gemini")), Some(Origin::Environment));
        assert_eq!(
            keys.key(&id("gemini")).map(|k| k.expose().to_string()),
            Some("env-gemini-key".into())
        );
        assert_eq!(keys.origin(&id("openai")), Some(Origin::Keychain));
        assert_eq!(keys.origin(&id("anthropic")), Some(Origin::Keychain));
    }

    #[test]
    fn with_the_keychain_off_only_the_environment_counts() {
        let env = |name: &str| (name == "KALAMU_MISTRAL_API_KEY").then(|| "m-key-123".into());
        let keys = Keys::gather(&[id("mistral"), id("gemini")], env, None);
        assert!(keys.key(&id("mistral")).is_some());
        assert!(keys.key(&id("gemini")).is_none());
        assert_eq!(keys.origin(&id("gemini")), None);
    }

    #[test]
    fn a_key_never_shows_in_debug_output_or_redacted_text() {
        let env = |_: &str| Some("plain-secret-value".to_string());
        let keys = Keys::gather(&[id("gemini")], env, None);
        let key = keys.key(&id("gemini"));
        assert!(!format!("{key:?}").contains("plain-secret-value"));
        let redactor = keys.redactor();
        let message = "Invalid key plain-secret-value for this project";
        assert_eq!(redactor.text(message), "Invalid key [key] for this project");
        assert!(!redactor.line(message, 20).contains("plain"));
    }

    #[test]
    fn identifiers_and_key_shapes_are_taken_out_but_model_names_stay() {
        let r = Redactor::default();
        let text = r#"{"message":"Incorrect API key provided: sk-proj-abcdefgh12345678 on req_0123456789abcdef for project 123456789012 (batch 3f2a9c1e-1b2c-4d5e-8f90-a1b2c3d4e5f6), file-Abc123Def456Ghi789. Model voxtral-mini-transcribe-3-0 at gemini-embedding-2: retry in 33s. Limit GenerateRequestsPerDayPerProjectPerModel 1048576"}"#;
        let out = r.text(text);
        assert!(!out.contains("sk-proj"), "{out}");
        assert!(!out.contains("req_0123"), "{out}");
        assert!(!out.contains("123456789012"), "{out}");
        assert!(!out.contains("3f2a9c1e"), "{out}");
        assert!(!out.contains("Abc123"), "{out}");
        for kept in [
            "voxtral-mini-transcribe-3-0",
            "gemini-embedding-2",
            "retry in 33s",
            "GenerateRequestsPerDayPerProjectPerModel",
            "1048576",
        ] {
            assert!(out.contains(kept), "{kept} lost: {out}");
        }
        assert!(out.contains("AIza") || !text.contains("AIza"));
        assert_eq!(r.text("AIzaSyA1234567890abcdefGHIJ"), "[key]");
    }

    #[test]
    fn a_line_is_flat_and_cut() {
        let r = Redactor::default();
        assert_eq!(r.line("one\ntwo\tthree", 50), "one two three");
        assert_eq!(r.line("abcdefghij", 5), "abcd…");
    }
}
