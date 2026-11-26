//! What a model list must hold before it is used, the bundled one and a downloaded one alike
//! ([decision](../../../docs/decisions/2026-11-25-model-list-checked.md)).
//!
//! Reading is lenient where that is safe: a model whose status, tier or way of taking languages
//! this build does not know is hidden, and a task it does not know is left out of a model's
//! tasks, so that a list written for a newer build still reads here. Everything else that is
//! wrong refuses the whole list, and the list in use stays.

use std::collections::BTreeSet;

use kalamu_core::Task;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{Catalog, LanguageHints, LanguageSupport, ModelStatus, Tier};

/// The largest model list read, in bytes: a download that grows past it is stopped. The list
/// held about 65 KB when the limit was set; sixteen times that leaves room for years of models.
pub const MAX_BYTES: usize = 1024 * 1024;

/// The tasks every list must offer a model for: the three kinds of call providers know. The
/// other tasks are made of these.
const BASE_TASKS: [Task; 3] = [
    Task::TextExtraction,
    Task::AudioTranscription,
    Task::Analysis,
];

/// A model a list names but this build hides, by provider and id.
pub(crate) type Hidden = BTreeSet<(String, String)>;

/// Hides the models whose status, tier or way of taking languages this build does not know,
/// and leaves out of each model's task lists the tasks it does not know; a model left with no
/// task is hidden. Only names (strings) are read leniently: a value of the wrong kind is left
/// for the typed reading to refuse.
pub(crate) fn hide_unknown(list: &mut Value) -> Hidden {
    let mut hidden = Hidden::new();
    let Some(models) = list.get_mut("models").and_then(Value::as_array_mut) else {
        return hidden;
    };
    models.retain_mut(|model| {
        let known = known_name::<ModelStatus>(model, "status")
            && known_name::<Tier>(model, "tier")
            && known_name::<LanguageHints>(model, "language_hints")
            && keep_known_tasks(model);
        if !known {
            let name = |field: &str| model[field].as_str().unwrap_or_default().to_string();
            hidden.insert((name("provider"), name("id")));
        }
        known
    });
    hidden
}

/// Whether the field is absent, not a name, or a name this build knows.
fn known_name<T: DeserializeOwned>(model: &Value, field: &str) -> bool {
    match model.get(field) {
        Some(Value::String(name)) => {
            serde_json::from_value::<T>(Value::String(name.clone())).is_ok()
        }
        _ => true,
    }
}

/// Leaves out the tasks this build does not know; false when a model that listed tasks is left
/// with none it can do.
fn keep_known_tasks(model: &mut Value) -> bool {
    let known = |task: &Value| match task {
        Value::String(name) => serde_json::from_value::<Task>(Value::String(name.clone())).is_ok(),
        _ => true,
    };
    let mut left_some = true;
    for field in ["tasks", "recommended_for", "provider_default_for"] {
        if let Some(tasks) = model.get_mut(field).and_then(Value::as_array_mut) {
            let listed = !tasks.is_empty();
            tasks.retain(known);
            if field == "tasks" && listed && tasks.is_empty() {
                left_some = false;
            }
        }
    }
    left_some
}

impl Catalog {
    /// Forgets what pointed at a hidden model: a successor this build cannot offer leaves the
    /// older model offered, as it was before its successor existed, and an alias for a hidden
    /// model is dropped.
    pub(crate) fn forget_hidden(&mut self, hidden: &Hidden) {
        let is_hidden =
            |provider: &str, id: &str| hidden.contains(&(provider.to_string(), id.to_string()));
        for model in &mut self.models {
            if model
                .successor
                .as_deref()
                .is_some_and(|s| is_hidden(model.provider.as_str(), s))
            {
                model.successor = None;
            }
        }
        self.aliases
            .retain(|a| !is_hidden(a.provider.as_str(), &a.model));
    }

    /// Every problem of sense found in the list, empty when there is none. `hosts` are the
    /// only hosts a provider's key page may be on: those the bundled list names.
    pub(crate) fn problems(&self, hosts: &BTreeSet<String>) -> Vec<String> {
        let mut problems = Vec::new();
        let mut problem = |p: String| problems.push(p);

        if self.schema_version == 0 {
            problem("schema_version is 0".into());
        }
        if !is_day(&self.updated) {
            problem(format!(
                "updated is not a day written YYYY-MM-DD: {:?}",
                self.updated
            ));
        }
        if self.providers.is_empty() {
            problem("no provider".into());
        }
        if self.models.is_empty() {
            problem("no model".into());
        }
        if self.languages.is_empty() {
            problem("no language".into());
        }

        let mut providers = BTreeSet::new();
        for provider in &self.providers {
            let id = provider.id.as_str();
            if id.trim().is_empty() || provider.name.trim().is_empty() {
                problem(format!("a provider without an id or a name: {id:?}"));
            }
            if !providers.insert(id) {
                problem(format!("provider {id} is listed twice"));
            }
            // A server of the researcher's own is theirs to add, never a list's: a list naming
            // one could pass off an address as a provider the researcher chose.
            if provider.own.is_some() || id.starts_with(crate::ID_PREFIX) {
                problem(format!(
                    "provider {id} is a server of the researcher's own, which a list never names"
                ));
            }
            match (provider.key_url.is_empty(), provider.local) {
                (true, true) => {}
                (true, false) => problem(format!("provider {id} names no page for its keys")),
                (false, _) => {
                    if let Err(e) = allowed_link(&provider.key_url, hosts) {
                        problem(format!("provider {id}: {e}"));
                    }
                }
            }
        }

        let mut models = BTreeSet::new();
        for model in &self.models {
            let at = format!("model {}/{}", model.provider, model.id);
            if model.id.trim().is_empty() || model.name.trim().is_empty() {
                problem(format!("{at} has no id or no name"));
            }
            if !providers.contains(model.provider.as_str()) {
                problem(format!("{at} names an unknown provider"));
            }
            if !models.insert((model.provider.as_str(), model.id.as_str())) {
                problem(format!("{at} is listed twice"));
            }
            if model.tasks.is_empty() {
                problem(format!("{at} has no task"));
            }
            if model.inputs.is_empty() || model.inputs.iter().any(|i| !i.contains('/')) {
                problem(format!(
                    "{at} reads no type of file, or one written without a '/'"
                ));
            }
            for (list, tasks) in [
                ("recommended_for", &model.recommended_for),
                ("provider_default_for", &model.provider_default_for),
            ] {
                if tasks.iter().any(|t| !model.tasks.contains(t)) {
                    problem(format!("{at}: {list} names a task it cannot do"));
                }
                if !tasks.is_empty() && !model.is_offered() {
                    problem(format!("{at}: {list} names a model no longer offered"));
                }
            }
            match &model.languages {
                LanguageSupport::Any(word) if word != "any" => {
                    problem(format!("{at}: languages is neither \"any\" nor a list"));
                }
                LanguageSupport::List(codes) if codes.is_empty() => {
                    problem(format!("{at} offers no language"));
                }
                _ => {}
            }
            let p = &model.pricing;
            for (name, price) in [
                ("input_per_mtok", p.input_per_mtok),
                ("output_per_mtok", p.output_per_mtok),
                ("per_minute", p.per_minute),
                ("per_page", p.per_page),
                ("cached_input_per_mtok", p.cached_input_per_mtok),
                ("cache_write_per_mtok", p.cache_write_per_mtok),
            ] {
                if price.is_some_and(|v| !v.is_finite() || v < 0.0) {
                    problem(format!("{at}: {name} is negative or not a number"));
                }
            }
            if p.batch_discount.is_some_and(|d| !(0.0..=1.0).contains(&d)) {
                problem(format!("{at}: batch_discount is not between 0 and 1"));
            }
            let l = &model.limits;
            for (name, limit) in [
                ("max_request_mb", l.max_request_mb),
                ("max_pages", l.max_pages),
                ("max_audio_minutes", l.max_audio_minutes),
                ("max_output_tokens", l.max_output_tokens),
                ("max_input_tokens", l.max_input_tokens),
            ] {
                if limit == Some(0) {
                    problem(format!("{at}: {name} is 0"));
                }
            }
            if let Some(reasoning) = &model.reasoning
                && (reasoning.levels.is_empty()
                    || reasoning.levels.iter().any(|l| l.trim().is_empty()))
            {
                problem(format!("{at} reasons at no level, or at an empty one"));
            }
        }

        // A successor is another model of the same provider, and a line of successors ends.
        for model in &self.models {
            let Some(successor) = &model.successor else {
                continue;
            };
            let at = format!("model {}/{}", model.provider, model.id);
            if successor == &model.id
                || !models.contains(&(model.provider.as_str(), successor.as_str()))
            {
                problem(format!(
                    "{at}: successor {successor} is not another model it lists"
                ));
            } else if !self.line_ends(model) {
                problem(format!("{at}: its line of successors loops"));
            }
        }
        let mut aliases = BTreeSet::new();
        for alias in &self.aliases {
            let (provider, id) = (alias.provider.as_str(), alias.id.as_str());
            if !models.contains(&(provider, alias.model.as_str())) {
                problem(format!("alias {provider}/{id} names no model"));
            }
            if models.contains(&(provider, id)) || !aliases.insert((provider, id)) {
                problem(format!(
                    "alias {provider}/{id} is a model's id, or is listed twice"
                ));
            }
        }

        let mut embedding = BTreeSet::new();
        for model in &self.embedding_models {
            let at = format!("embedding model {}/{}", model.provider, model.id);
            match self.provider(&model.provider) {
                None => problem(format!("{at} names an unknown provider")),
                // Said "nothing is sent" of a model whose provider is reached over the network.
                Some(p) if p.local != model.local => {
                    problem(format!("{at} runs elsewhere than its provider"));
                }
                Some(_) => {}
            }
            if !embedding.insert((model.provider.as_str(), model.id.as_str())) {
                problem(format!("{at} is listed twice"));
            }
            if model.id.trim().is_empty() || model.name.trim().is_empty() {
                problem(format!("{at} has no id or no name"));
            }
            if model.dimensions.is_empty() || model.dimensions.contains(&0) {
                problem(format!("{at} gives no length of vector, or one of 0"));
            }
            if model.max_chars == 0 {
                problem(format!("{at}: max_chars is 0"));
            }
            if model.input_price.is_some_and(|v| !v.is_finite() || v < 0.0) {
                problem(format!("{at}: input_price is negative or not a number"));
            }
        }

        let mut languages = BTreeSet::new();
        for language in &self.languages {
            let code = language.code.as_str();
            if code.trim().is_empty() || language.name.trim().is_empty() {
                problem(format!("a language without a code or a name: {code:?}"));
            }
            if !languages.insert(code) {
                problem(format!("language {code} is listed twice"));
            }
        }

        // Every task has a model offered for it, so New job always has a default.
        for task in BASE_TASKS {
            if self.recommended(task, &[]).is_none() {
                problem(format!("no model offered for {}", task.as_str()));
            }
        }
        problems
    }

    /// Whether following a model's successors reaches a model without one.
    fn line_ends(&self, model: &crate::ModelInfo) -> bool {
        let mut current = model;
        for _ in 0..=self.models.len() {
            let Some(next) = current.successor.as_deref() else {
                return true;
            };
            match self
                .models
                .iter()
                .find(|m| m.provider == current.provider && m.id == next)
            {
                Some(m) => current = m,
                None => return true,
            }
        }
        false
    }
}

/// The hosts of the providers' key pages a list names: for the bundled list, the only hosts a
/// downloaded list's links may be on.
pub(crate) fn hosts(catalog: &Catalog) -> BTreeSet<String> {
    catalog
        .providers
        .iter()
        .filter_map(|p| https_host(&p.key_url))
        .map(str::to_ascii_lowercase)
        .collect()
}

/// A link a list may carry: an `https://` address on one of `hosts`, written plainly.
fn allowed_link(url: &str, hosts: &BTreeSet<String>) -> Result<(), String> {
    let host = https_host(url).ok_or_else(|| format!("{url:?} is not a plain https:// address"))?;
    if hosts.contains(&host.to_ascii_lowercase()) {
        Ok(())
    } else {
        Err(format!(
            "{url:?} is on {host}, which the bundled list does not name"
        ))
    }
}

/// The host of an `https://` address with no user, port, space or control character in it.
fn https_host(url: &str) -> Option<&str> {
    if url
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return None;
    }
    let rest = url.strip_prefix("https://")?;
    let host = rest.split(['/', '?', '#']).next()?;
    let plain = !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    plain.then_some(host)
}

/// `YYYY-MM-DD`, with a month and a day that can be.
pub(crate) fn is_day(text: &str) -> bool {
    let b = text.as_bytes();
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    if !(digits(0..4) && digits(5..7) && digits(8..10)) {
        return false;
    }
    let number = |r: std::ops::Range<usize>| text[r].parse::<u32>().unwrap_or(0);
    (1..=12).contains(&number(5..7)) && (1..=31).contains(&number(8..10))
}

#[cfg(test)]
mod tests;
