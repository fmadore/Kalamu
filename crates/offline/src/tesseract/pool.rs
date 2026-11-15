//! Tesseract's idle engines, kept for the next page of a job in the same languages.
//!
//! An engine is keyed by the languages it loaded (`fra+eng`), so two jobs in different
//! languages each find theirs instead of rebuilding one on every page. The pool holds at most
//! as many engines as pages are read at once ([`super::max_engines`]): that many always lets
//! every reading slot find its engine, whatever mix of languages the running jobs need, and
//! never keeps idle more than the jobs could use together. Past that, the engine unused the
//! longest goes. An idle engine holds no lease on its language data, so Settings may delete
//! it; the deletion then releases the engines that loaded it ([`Pool::release`]).

use std::time::{Duration, Instant};

/// Idle items, each under its key, with the files it loaded and when it was last used.
pub(super) struct Pool<T> {
    bound: usize,
    idle: Vec<Idle<T>>,
}

struct Idle<T> {
    key: String,
    /// The ids of the files the item loaded.
    files: Vec<&'static str>,
    item: T,
    last_used: Instant,
}

impl<T> Pool<T> {
    pub(super) fn new(bound: usize) -> Self {
        Self {
            bound: bound.max(1),
            idle: Vec::new(),
        }
    }

    /// The item under `key` used most recently, taken out of the pool.
    pub(super) fn take(&mut self, key: &str) -> Option<T> {
        let at = self
            .idle
            .iter()
            .enumerate()
            .filter(|(_, idle)| idle.key == key)
            .max_by_key(|(_, idle)| idle.last_used)
            .map(|(at, _)| at)?;
        Some(self.idle.swap_remove(at).item)
    }

    /// Keeps `item` under `key`, used now. Returns the items it pushed out, the least recently
    /// used first, for the caller to drop outside its lock.
    pub(super) fn put(&mut self, key: String, files: Vec<&'static str>, item: T) -> Vec<T> {
        self.put_at(key, files, item, Instant::now())
    }

    fn put_at(
        &mut self,
        key: String,
        files: Vec<&'static str>,
        item: T,
        last_used: Instant,
    ) -> Vec<T> {
        self.idle.push(Idle {
            key,
            files,
            item,
            last_used,
        });
        let mut out = Vec::new();
        while self.idle.len() > self.bound {
            let Some(oldest) = self
                .idle
                .iter()
                .enumerate()
                .min_by_key(|(_, idle)| idle.last_used)
                .map(|(at, _)| at)
            else {
                break;
            };
            out.push(self.idle.swap_remove(oldest).item);
        }
        out
    }

    /// Takes out every item that loaded the file `id`.
    pub(super) fn release(&mut self, id: &str) -> Vec<T> {
        self.drain(|idle| idle.files.contains(&id))
    }

    /// Takes out every item unused for longer than `age`.
    pub(super) fn sweep(&mut self, age: Duration) -> Vec<T> {
        self.sweep_at(age, Instant::now())
    }

    fn sweep_at(&mut self, age: Duration, now: Instant) -> Vec<T> {
        self.drain(|idle| now.saturating_duration_since(idle.last_used) > age)
    }

    fn drain(&mut self, out: impl Fn(&Idle<T>) -> bool) -> Vec<T> {
        let (gone, kept) = std::mem::take(&mut self.idle).into_iter().partition(out);
        self.idle = kept;
        gone.into_iter().map(|idle: Idle<T>| idle.item).collect()
    }

    #[cfg(test)]
    fn keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.idle.iter().map(|idle| idle.key.as_str()).collect();
        keys.sort_unstable();
        keys
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn two_jobs_in_different_languages_each_find_their_engines() {
        let mut pool = Pool::new(4);
        assert!(pool.put("fra".into(), vec!["tesseract-fra"], 1).is_empty());
        assert!(pool.put("eng".into(), vec!["tesseract-eng"], 2).is_empty());
        // Alternating pages: each takes its own back and returns it.
        for _ in 0..3 {
            assert_eq!(pool.take("fra"), Some(1));
            assert!(pool.put("fra".into(), vec!["tesseract-fra"], 1).is_empty());
            assert_eq!(pool.take("eng"), Some(2));
            assert!(pool.put("eng".into(), vec!["tesseract-eng"], 2).is_empty());
        }
        assert_eq!(
            pool.take("fra+eng"),
            None,
            "another set of languages is another engine"
        );
        assert_eq!(pool.keys(), ["eng", "fra"]);
    }

    #[test]
    fn past_its_bound_the_least_recently_used_goes() {
        let start = Instant::now();
        let at = |s: u64| start + Duration::from_secs(s);
        let mut pool = Pool::new(2);
        assert!(pool.put_at("fra".into(), vec![], "fra", at(1)).is_empty());
        assert!(pool.put_at("eng".into(), vec![], "eng", at(2)).is_empty());
        // French is used again: English is now the oldest.
        assert_eq!(pool.take("fra"), Some("fra"));
        assert!(pool.put_at("fra".into(), vec![], "fra", at(3)).is_empty());
        assert_eq!(pool.put_at("deu".into(), vec![], "deu", at(4)), ["eng"]);
        assert_eq!(pool.keys(), ["deu", "fra"]);
    }

    #[test]
    fn the_most_recently_used_of_one_key_is_taken() {
        let start = Instant::now();
        let mut pool = Pool::new(4);
        pool.put_at("fra".into(), vec![], 1, start);
        pool.put_at("fra".into(), vec![], 2, start + Duration::from_secs(1));
        assert_eq!(pool.take("fra"), Some(2));
        assert_eq!(pool.take("fra"), Some(1));
        assert_eq!(pool.take("fra"), None);
    }

    #[test]
    fn deleting_language_data_releases_every_engine_that_loaded_it() {
        let mut pool = Pool::new(4);
        pool.put("fra".into(), vec!["tesseract-fra"], 1);
        pool.put("fra+eng".into(), vec!["tesseract-fra", "tesseract-eng"], 2);
        pool.put("eng".into(), vec!["tesseract-eng"], 3);
        let mut released = pool.release("tesseract-fra");
        released.sort_unstable();
        assert_eq!(released, [1, 2]);
        assert_eq!(pool.keys(), ["eng"]);
        assert!(pool.release("tesseract-fra").is_empty());
    }

    #[test]
    fn the_idle_sweep_releases_what_sat_unused() {
        let start = Instant::now();
        let mut pool = Pool::new(4);
        pool.put_at("fra".into(), vec![], 1, start);
        pool.put_at("eng".into(), vec![], 2, start + Duration::from_secs(500));
        let later = start + Duration::from_secs(600);
        assert_eq!(pool.sweep_at(Duration::from_secs(180), later), [1]);
        assert_eq!(pool.keys(), ["eng"]);
    }
}
