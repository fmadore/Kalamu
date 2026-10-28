//! YouTube videos, transcribed from their address: the provider fetches the video, and
//! Kalamu never downloads or stores it. Only Gemini reads YouTube links today; a model says
//! so by listing [`YOUTUBE_LINK`] among its catalogue inputs.

use crate::fingerprint::sha256_hex;
use crate::{InputKind, SourceFile, SourceId};

/// The form a YouTube video is sent in: not a real MIME type, but the address of the video,
/// carried as the prepared input's bytes. A provider that lists it in its accepted inputs sends
/// the link instead of a file.
pub const YOUTUBE_LINK: &str = "video/x-youtube";

/// The video id in a YouTube watch, `youtu.be`, Shorts, live or embed link, with or without
/// `https://`. Playlists, channels and other sites give `None`.
pub fn video_id(link: &str) -> Option<String> {
    let link = link.trim();
    let rest = link
        .strip_prefix("https://")
        .or_else(|| link.strip_prefix("http://"))
        .unwrap_or(link);
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    let host = host.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    let host = host.strip_prefix("m.").unwrap_or(host);
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let path = path.split('#').next().unwrap_or_default();
    let id = match host {
        "youtu.be" => path.split('/').next(),
        "youtube.com" | "music.youtube.com" | "youtube-nocookie.com" => {
            let mut parts = path.split('/');
            match parts.next() {
                Some("watch") => query
                    .split('#')
                    .next()
                    .unwrap_or_default()
                    .split('&')
                    .find_map(|pair| pair.strip_prefix("v=")),
                Some("shorts" | "live" | "embed") => parts.next(),
                _ => None,
            }
        }
        _ => None,
    }?;
    is_video_id(id).then(|| id.to_string())
}

/// Eleven characters from YouTube's URL-safe alphabet.
fn is_video_id(id: &str) -> bool {
    id.len() == 11
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// The canonical address of a video, as sent to the provider and recorded in provenance.
pub fn watch_url(id: &str) -> String {
    format!("https://www.youtube.com/watch?v={id}")
}

impl SourceFile {
    /// A video added by its link. Its fingerprint is the video id, so the same video pasted as
    /// a Shorts or `youtu.be` link finds the same cached results.
    pub fn youtube(link: &str) -> Option<Self> {
        let id = video_id(link)?;
        Some(Self {
            id: SourceId::new(),
            path: watch_url(&id),
            file_name: format!("youtube-{id}"),
            sha256: sha256_hex(format!("youtube:{id}").as_bytes()),
            size_bytes: 0,
            kind: InputKind::YouTube,
            page_count: None,
            duration_ms: None,
            split_spreads: false,
            even_out_volume: false,
            title: None,
        })
    }

    /// Names a video after its title, so researchers find it by what it is: in New job, the
    /// review and its result files. The id stays in the file name, so two videos with the same
    /// title never share files.
    pub fn with_title(mut self, title: &str) -> Self {
        let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
        if title.is_empty() {
            return self;
        }
        if let Some(id) = video_id(&self.path) {
            let safe = file_safe(&title);
            if !safe.is_empty() {
                self.file_name = format!("{safe} [{id}]");
            }
        }
        self.title = Some(title);
        self
    }
}

/// A title made safe as a Windows file name: no reserved characters, no trailing dots or
/// spaces, and short enough to leave room for the folder and extension.
fn file_safe(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .take(100)
        .collect();
    cleaned.trim_end_matches(['.', ' ']).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_form_of_link_gives_the_video_id() {
        for link in [
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtube.com/watch?feature=share&v=dQw4w9WgXcQ&t=42s",
            "youtube.com/watch?v=dQw4w9WgXcQ",
            "http://m.youtube.com/watch?v=dQw4w9WgXcQ#comments",
            "https://youtu.be/dQw4w9WgXcQ?si=abc",
            "https://www.youtube.com/shorts/dQw4w9WgXcQ",
            "https://www.youtube.com/live/dQw4w9WgXcQ?feature=shared",
            "https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ",
            "https://music.youtube.com/watch?v=dQw4w9WgXcQ",
            "  https://WWW.YouTube.com/watch?v=dQw4w9WgXcQ  ",
        ] {
            assert_eq!(video_id(link).as_deref(), Some("dQw4w9WgXcQ"), "{link}");
        }
    }

    #[test]
    fn other_links_are_refused() {
        for link in [
            "",
            "dQw4w9WgXcQ",
            "https://www.youtube.com/playlist?list=PL123",
            "https://www.youtube.com/@channel",
            "https://www.youtube.com/watch?v=short",
            "https://vimeo.com/123456789",
            "https://youtube.com.example.org/watch?v=dQw4w9WgXcQ",
            "https://www.youtube.com/watch?v=dQw4w9WgXc!",
        ] {
            assert_eq!(video_id(link), None, "{link}");
        }
    }

    #[test]
    fn a_video_is_one_source_whatever_its_link() {
        let a = SourceFile::youtube("https://youtu.be/dQw4w9WgXcQ");
        let b = SourceFile::youtube("https://www.youtube.com/shorts/dQw4w9WgXcQ");
        let (Some(a), Some(b)) = (a, b) else {
            panic!("both links are YouTube videos");
        };
        assert_eq!(a.sha256, b.sha256);
        assert_eq!(a.path, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert_eq!(a.kind, InputKind::YouTube);
    }

    #[test]
    fn a_title_names_the_video_and_its_files_safely() {
        let Some(video) = SourceFile::youtube("https://youtu.be/dQw4w9WgXcQ") else {
            panic!("a YouTube link");
        };
        let titled = video
            .clone()
            .with_title("  Prêche du vendredi : « Le jeûne »?  ...");
        assert_eq!(
            titled.title.as_deref(),
            Some("Prêche du vendredi : « Le jeûne »? ...")
        );
        assert_eq!(
            titled.file_name,
            "Prêche du vendredi - « Le jeûne »- [dQw4w9WgXcQ]"
        );
        assert_eq!(
            titled.sha256, video.sha256,
            "the title does not change the cache key"
        );
        assert_eq!(video.clone().with_title("   ").file_name, video.file_name);
    }
}
