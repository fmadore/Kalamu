//! An embedder that calls nothing: each text's vector is its bag of words, hashed into the
//! vector's places, so that texts sharing words lie close and the same text always gives the
//! same vector. For tests of the search by meaning, the map and the engine's preparation runs.

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
    ApiKey, EmbedLimits, EmbedRequest, EmbedRole, Embedded, Embedder, ProviderError, ProviderId,
};

/// A call made, as the fake saw it.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub role: EmbedRole,
    pub texts: Vec<String>,
    pub keyed: bool,
}

type Hook = Box<dyn Fn(usize) + Send + Sync>;

pub struct FakeEmbedder {
    id: ProviderId,
    local: bool,
    limits: EmbedLimits,
    /// Answered, one per call, before any vector: a rate limit, a refused key.
    failures: Mutex<VecDeque<ProviderError>>,
    calls: Mutex<Vec<Call>>,
    /// Told the number of each successful call, from 1, after it answers.
    after_call: Mutex<Option<Hook>>,
    /// What `check_ready` answers for a model not yet downloaded.
    missing: Mutex<bool>,
}

impl FakeEmbedder {
    /// A cloud embedder, which needs a key and bills a token for each word.
    pub fn cloud(id: &str, limits: EmbedLimits) -> Self {
        Self::new(id, false, limits)
    }

    /// An embedder run on this computer: no key, nothing billed.
    pub fn local(id: &str, limits: EmbedLimits) -> Self {
        Self::new(id, true, limits)
    }

    fn new(id: &str, local: bool, limits: EmbedLimits) -> Self {
        Self {
            id: ProviderId::new(id),
            local,
            limits,
            failures: Mutex::default(),
            calls: Mutex::default(),
            after_call: Mutex::new(None),
            missing: Mutex::new(false),
        }
    }

    /// Answers the next calls with these errors, one each, before answering again.
    pub fn fail_with(&self, errors: impl IntoIterator<Item = ProviderError>) {
        lock(&self.failures).extend(errors);
    }

    /// Runs `hook` after each successful call, with its number from 1.
    pub fn after_call(&self, hook: impl Fn(usize) + Send + Sync + 'static) {
        *lock(&self.after_call) = Some(Box::new(hook));
    }

    /// Says its model is not downloaded.
    pub fn set_missing(&self, missing: bool) {
        *lock(&self.missing) = missing;
    }

    /// Every call answered with vectors, in order.
    pub fn calls(&self) -> Vec<Call> {
        lock(&self.calls).clone()
    }

    /// Every text embedded as a passage, in order.
    pub fn passages(&self) -> Vec<String> {
        self.calls()
            .into_iter()
            .filter(|c| c.role == EmbedRole::Passage)
            .flat_map(|c| c.texts)
            .collect()
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

/// The words of a text, small letters, each character of a script written without spaces (Chinese,
/// Japanese) a word of its own.
fn words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    for c in text.chars().flat_map(char::to_lowercase) {
        let unspaced = matches!(c as u32, 0x3040..=0x30FF | 0x3400..=0x9FFF | 0xAC00..=0xD7AF);
        if c.is_alphanumeric() && !unspaced {
            word.push(c);
            continue;
        }
        if !word.is_empty() {
            out.push(std::mem::take(&mut word));
        }
        if unspaced {
            out.push(c.to_string());
        }
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

/// The text's bag of words in `dimensions` places, of length 1: the same text always gives the
/// same vector, and texts sharing words lie close.
pub fn fake_vector(text: &str, dimensions: usize) -> Vec<f32> {
    let mut v = vec![0f32; dimensions.max(1)];
    for w in words(text) {
        // FNV-1a.
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in w.bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        let place = (h % v.len() as u64) as usize;
        v[place] += 1.0;
    }
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        v[0] = 1.0;
    } else {
        v.iter_mut().for_each(|x| *x /= norm);
    }
    v
}

#[async_trait]
impl Embedder for FakeEmbedder {
    fn id(&self) -> ProviderId {
        self.id.clone()
    }

    fn needs_key(&self) -> bool {
        !self.local
    }

    fn check_ready(&self, model: &str) -> Result<(), ProviderError> {
        if *lock(&self.missing) {
            return Err(ProviderError::NotInstalled(model.to_string()));
        }
        Ok(())
    }

    fn limits(&self, _model: &str) -> EmbedLimits {
        self.limits
    }

    async fn embed(
        &self,
        key: Option<&ApiKey>,
        request: EmbedRequest<'_>,
    ) -> Result<Embedded, ProviderError> {
        if request.stop.is_stopped() {
            return Err(ProviderError::Interrupted);
        }
        if let Some(err) = lock(&self.failures).pop_front() {
            return Err(err);
        }
        if request.texts.len() > self.limits.texts
            || request
                .texts
                .iter()
                .any(|t| t.chars().count() > self.limits.chars)
        {
            return Err(ProviderError::InvalidRequest("over the limits".into()));
        }
        let dimensions = request.dimensions.unwrap_or(64) as usize;
        let vectors = request
            .texts
            .iter()
            .map(|t| fake_vector(t, dimensions))
            .collect();
        let tokens = if self.local {
            0
        } else {
            request.texts.iter().map(|t| words(t).len() as u64).sum()
        };
        let n = {
            let mut calls = lock(&self.calls);
            calls.push(Call {
                role: request.role,
                texts: request.texts.to_vec(),
                keyed: key.is_some(),
            });
            calls.len()
        };
        if let Some(hook) = lock(&self.after_call).as_ref() {
            hook(n);
        }
        Ok(Embedded {
            vectors,
            input_tokens: tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn dot(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    #[test]
    fn texts_sharing_words_lie_closer_than_texts_that_do_not() {
        let a = fake_vector("the mosque of Bobo-Dioulasso", 256);
        let b = fake_vector("a new mosque in Bobo-Dioulasso", 256);
        let c = fake_vector("tax receipts of the colony", 256);
        assert!(dot(&a, &b) > dot(&a, &c));
        assert!((dot(&a, &a) - 1.0).abs() < 1e-5);
        assert_eq!(fake_vector("", 8)[0], 1.0);
        assert_eq!(words("北京大学 abc"), ["北", "京", "大", "学", "abc"]);
    }
}
