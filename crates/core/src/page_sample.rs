//! Which pages a sample reads (#215): every page, the first few, or a few spread through the
//! document, so a researcher sees how a model reads typical pages before paying for the rest.
//!
//! Spread pages leave out the first page, most often a cover or a title page that reads unlike
//! the others, and take the middle page of each equal stretch of the rest: three pages of a
//! forty-page document are pages 8, 21 and 34. The same document always gives the same pages,
//! so a sample resumed reads what it planned, and the job that reads the rest finds them in the
//! cache.

use crate::JobSettings;

/// The pages of each document a job reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageSample {
    /// Every page.
    #[default]
    All,
    /// The first pages only.
    First(u32),
    /// So many pages spread through the document, the first page left out.
    Spread(u32),
}

impl PageSample {
    /// The page numbers (from 1, in order) read of a document of `count` pages; every page of a
    /// document no longer than the sample.
    pub fn pages(self, count: u32) -> Vec<u32> {
        match self {
            Self::All => (1..=count).collect(),
            Self::First(n) => (1..=count.min(n)).collect(),
            Self::Spread(n) if n >= count => (1..=count).collect(),
            Self::Spread(0) => Vec::new(),
            Self::Spread(n) => {
                // Pages 2 to `count`, at least `n` of them, cut into `n` equal stretches: the
                // middle of each, so no two are the same page.
                let span = u64::from(count - 1);
                let n64 = u64::from(n);
                (0..n64)
                    .map(|i| 2 + ((2 * i + 1) * span / (2 * n64)) as u32)
                    .collect()
            }
        }
    }

    /// Whether `count` pages are more than the sample reads, so that it reads only some.
    pub fn cuts(self, count: u32) -> bool {
        match self {
            Self::All => false,
            Self::First(n) | Self::Spread(n) => count > n,
        }
    }
}

impl JobSettings {
    /// The pages of each document this job reads.
    pub fn page_sample(&self) -> PageSample {
        match self.sample_pages {
            None => PageSample::All,
            Some(n) if self.sample_spread => PageSample::Spread(n),
            Some(n) => PageSample::First(n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spread_pages_leave_the_cover_out_and_cover_the_document() {
        assert_eq!(PageSample::Spread(3).pages(40), [8, 21, 34]);
        assert_eq!(PageSample::Spread(3).pages(4), [2, 3, 4]);
        assert_eq!(PageSample::Spread(1).pages(10), [6]);
        assert_eq!(PageSample::Spread(4).pages(200), [26, 76, 126, 176]);
        // No longer than the sample: every page, the first one too.
        assert_eq!(PageSample::Spread(3).pages(3), [1, 2, 3]);
        assert_eq!(PageSample::Spread(5).pages(2), [1, 2]);
        for count in 1..120 {
            for n in 1..12 {
                let pages = PageSample::Spread(n).pages(count);
                assert_eq!(pages.len() as u32, n.min(count));
                assert!(pages.windows(2).all(|w| w[0] < w[1]), "{n} of {count}");
                assert!(pages.iter().all(|&p| (1..=count).contains(&p)));
            }
        }
    }

    #[test]
    fn first_pages_and_every_page() {
        assert_eq!(PageSample::First(3).pages(40), [1, 2, 3]);
        assert_eq!(PageSample::First(3).pages(2), [1, 2]);
        assert_eq!(PageSample::All.pages(3), [1, 2, 3]);
        assert!(PageSample::First(3).cuts(4));
        assert!(!PageSample::Spread(3).cuts(3));
        assert!(!PageSample::All.cuts(1_000));
    }

    #[test]
    fn a_job_spreads_its_sample_only_when_asked() {
        let mut settings = JobSettings::default();
        assert_eq!(settings.page_sample(), PageSample::All);
        settings.sample_pages = Some(3);
        assert_eq!(settings.page_sample(), PageSample::First(3));
        settings.sample_spread = true;
        assert_eq!(settings.page_sample(), PageSample::Spread(3));
        // Settings stored before spread samples existed read as they did.
        let old: JobSettings = serde_json::from_str(
            r#"{"languages":{"mode":"auto"},"sample_pages":3,"line_breaks":null,"pace":"fast"}"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(old.page_sample(), PageSample::First(3));
        let written = serde_json::to_string(&old).unwrap_or_else(|e| panic!("{e}"));
        assert!(!written.contains("sample_spread"), "{written}");
    }
}
