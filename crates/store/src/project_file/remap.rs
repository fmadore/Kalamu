//! What changes in a row as it comes in from a project file: every identifier the file holds
//! given a new one wherever it appears (a column, a JSON document, a scope's name), and every
//! path of a file the project file carried pointed at where it was written on this computer.
//! The models' answers are never passed through it.

use std::collections::HashMap;

/// Old identifiers to new ones, old paths to new ones.
#[derive(Debug, Default)]
pub(super) struct Remap {
    pub ids: HashMap<String, String>,
    pub paths: HashMap<String, String>,
}

impl Remap {
    /// The text with its paths and identifiers replaced, or `None` when nothing changed. Paths
    /// first, as a path may hold an identifier (a folder Kalamu named after a project): the new
    /// path holds only new ones, which no file can have named.
    pub fn text(&self, text: &str) -> Option<String> {
        let pathed = self.json_paths(text);
        let current = pathed.as_deref().unwrap_or(text);
        let replaced = self.ids(current);
        replaced.or(pathed)
    }

    /// A text that is wholly an identifier the file holds, whatever its form, or any other text
    /// through [`Self::text`].
    pub fn cell(&self, text: &str) -> Option<String> {
        self.ids.get(text).cloned().or_else(|| self.text(text))
    }

    /// Every identifier written as a UUID (`8-4-4-4-12` hexadecimal digits) that the file holds.
    fn ids(&self, text: &str) -> Option<String> {
        let bytes = text.as_bytes();
        let mut out = String::new();
        let mut copied = 0;
        let mut i = 0;
        while i + 36 <= bytes.len() {
            if is_uuid(&bytes[i..i + 36])
                && (i == 0 || !is_word(bytes[i - 1]))
                && bytes.get(i + 36).is_none_or(|b| !is_word(*b))
                && let Some(new) = self.ids.get(&text[i..i + 36])
            {
                out.push_str(&text[copied..i]);
                out.push_str(new);
                i += 36;
                copied = i;
            } else {
                i += 1;
            }
        }
        if copied == 0 {
            return None;
        }
        out.push_str(&text[copied..]);
        Some(out)
    }

    /// The paths in a JSON document: every string that is wholly a path the file carried.
    fn json_paths(&self, text: &str) -> Option<String> {
        if self.paths.is_empty() || !(text.starts_with('{') || text.starts_with('[')) {
            return None;
        }
        let mut value: serde_json::Value = serde_json::from_str(text).ok()?;
        if !self.walk(&mut value) {
            return None;
        }
        serde_json::to_string(&value).ok()
    }

    /// Replaces every path in `value`, walking all of it; whether any was.
    fn walk(&self, value: &mut serde_json::Value) -> bool {
        let children: Vec<&mut serde_json::Value> = match value {
            serde_json::Value::String(s) => {
                let Some(new) = self.paths.get(s.as_str()) else {
                    return false;
                };
                *s = new.clone();
                return true;
            }
            serde_json::Value::Array(items) => items.iter_mut().collect(),
            serde_json::Value::Object(map) => map.values_mut().collect(),
            _ => return false,
        };
        let mut changed = false;
        for child in children {
            changed |= self.walk(child);
        }
        changed
    }
}

fn is_uuid(window: &[u8]) -> bool {
    window.iter().enumerate().all(|(i, b)| match i {
        8 | 13 | 18 | 23 => *b == b'-',
        _ => b.is_ascii_hexdigit(),
    })
}

/// A byte that would make a UUID part of a longer word.
fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD: &str = "6f1c2b9e-0a1b-4c2d-8e3f-123456789abc";
    const NEW: &str = "11111111-2222-4333-8444-555555555555";

    fn remap() -> Remap {
        let mut r = Remap::default();
        r.ids.insert(OLD.into(), NEW.into());
        r.ids.insert("p1".into(), "p2".into());
        r.paths.insert(
            r"C:\Archives\a.pdf".into(),
            "/data/originals/x/a.pdf".into(),
        );
        r
    }

    #[test]
    fn identifiers_are_replaced_wherever_they_stand() {
        let r = remap();
        assert_eq!(r.text(OLD).as_deref(), Some(NEW));
        assert_eq!(
            r.text(&format!("collection:{OLD}")).as_deref(),
            Some(format!("collection:{NEW}").as_str())
        );
        assert_eq!(
            r.text(&format!(r#"{{"job_id":"{OLD}","n":1}}"#)).as_deref(),
            Some(format!(r#"{{"job_id":"{NEW}","n":1}}"#).as_str())
        );
        // Another identifier, and one inside a longer word, are left.
        assert_eq!(r.text("00000000-0000-4000-8000-000000000000"), None);
        assert_eq!(r.text(&format!("x{OLD}")), None);
        // A whole cell is replaced whatever its form.
        assert_eq!(r.cell("p1").as_deref(), Some("p2"));
        assert_eq!(r.text("p1"), None);
    }

    #[test]
    fn paths_are_replaced_only_where_a_string_is_wholly_one() {
        let r = remap();
        let json = r#"{"path":"C:\\Archives\\a.pdf","note":"C:\\Archives\\a.pdf and more"}"#;
        let out = r.text(json).unwrap_or_default();
        let value: serde_json::Value = serde_json::from_str(&out).unwrap_or_default();
        assert_eq!(value["path"], "/data/originals/x/a.pdf");
        assert_eq!(value["note"], r"C:\Archives\a.pdf and more");
        // Plain text is not read as JSON.
        assert_eq!(r.text(r"C:\Archives\a.pdf"), None);
    }
}
