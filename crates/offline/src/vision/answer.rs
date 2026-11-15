//! Reading the OCR models' answers: LightOnOCR's grounded blocks placed on the page, and the
//! line an answer cut short wrote over and over kept once.

use kalamu_core::{TextRegion, plain_words};

/// Block kinds that hold no text from the page: LightOnOCR describes pictures and turns charts
/// into tables of their values, which are its reading of the page, not words on it.
const NOT_TEXT: [&str; 4] = ["image", "header_image", "footer_image", "chart"];

/// A placeholder at the start of a line, `![kind](x0,y0,x1,y1)`, as its kind, its box in
/// thousandths of the page, and what follows it on the same line: through llama.cpp the block's
/// first line of text comes right after it (`![title](110,51,880,133) # BULLETIN…`), where the
/// model's own examples break the line.
fn placeholder(line: &str) -> Option<(&str, [f32; 4], &str)> {
    let rest = line.trim_start().strip_prefix("![")?;
    let (kind, rest) = rest.split_once("](")?;
    let (numbers, after) = rest.split_once(')')?;
    if kind.is_empty()
        || !kind
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '_' || c == '+')
    {
        return None;
    }
    let mut values = numbers.split(',').map(|n| n.trim().parse::<f32>().ok());
    let mut corners = [0.0; 4];
    for corner in &mut corners {
        *corner = values.next()??;
    }
    values
        .next()
        .is_none()
        .then_some((kind, corners, after.trim()))
}

/// LightOnOCR's grounded answer as the page's Markdown and its blocks of text on the page.
/// Without a single placeholder the whole answer is the page's text, placed nowhere.
pub(super) fn grounded(answer: &str) -> (String, Vec<TextRegion>) {
    let mut blocks: Vec<(String, [f32; 4], Vec<&str>)> = Vec::new();
    let mut before: Vec<&str> = Vec::new();
    for line in answer.lines() {
        if let Some((kind, corners, first)) = placeholder(line) {
            let lines = if first.is_empty() {
                Vec::new()
            } else {
                vec![first]
            };
            blocks.push((kind.trim_end_matches('+').to_owned(), corners, lines));
        } else if let Some(block) = blocks.last_mut() {
            block.2.push(line);
        } else {
            before.push(line);
        }
    }
    if blocks.is_empty() {
        return (answer.trim().to_owned(), Vec::new());
    }
    let mut texts: Vec<String> = Vec::new();
    let lead = before.join("\n").trim().to_owned();
    if !lead.is_empty() {
        texts.push(lead);
    }
    let mut regions = Vec::new();
    for (kind, [x0, y0, x1, y1], lines) in blocks {
        if NOT_TEXT.contains(&kind.as_str()) {
            continue;
        }
        let text = lines.join("\n").trim().to_owned();
        if text.is_empty() {
            continue;
        }
        let words = plain_words(&text);
        let (x0, y0) = ((x0 / 1000.0).clamp(0.0, 1.0), (y0 / 1000.0).clamp(0.0, 1.0));
        let (x1, y1) = ((x1 / 1000.0).clamp(0.0, 1.0), (y1 / 1000.0).clamp(0.0, 1.0));
        if !words.is_empty() && x1 > x0 && y1 > y0 {
            regions.push(TextRegion::block(x0, y0, x1 - x0, y1 - y0, words));
        }
        texts.push(text);
    }
    (texts.join("\n\n"), regions)
}

/// An answer that ran to the limit often ends in a line written over and over: kept once.
pub(super) fn without_runaway(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let Some(last) = lines.iter().rposition(|l| !l.trim().is_empty()) else {
        return text.to_owned();
    };
    let repeated = lines[last].trim();
    let mut first = last;
    let mut times = 1;
    for (i, line) in lines[..last].iter().enumerate().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line != repeated {
            break;
        }
        first = i;
        times += 1;
    }
    if times < 4 {
        return text.to_owned();
    }
    lines[..=first].join("\n")
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounded_blocks_are_placed_and_pictures_left_out() {
        let answer = "![title](80,70,920,140)\n# Le Réveil du Dahomey\n\n\
![text](80,175,920,225)\nLa **réunion** s'est tenue à Porto-Novo.\n\n\
![image](80,285,460,545)\nUn homme devant une mosquée.\n\n\
![text+](80,560,920,600)\nElle reprendra demain.\n";
        let (text, regions) = grounded(answer);
        assert_eq!(
            text,
            "# Le Réveil du Dahomey\n\nLa **réunion** s'est tenue à Porto-Novo.\n\nElle reprendra demain."
        );
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].text, "Le Réveil du Dahomey");
        assert_eq!(regions[1].text, "La réunion s'est tenue à Porto-Novo.");
        let r = &regions[1];
        assert!((r.x - 0.08).abs() < 1e-6 && (r.y - 0.175).abs() < 1e-6);
        assert!((r.width - 0.84).abs() < 1e-6 && (r.height - 0.05).abs() < 1e-6);
        assert!(!text.contains("mosquée"));
    }

    #[test]
    fn an_answer_without_placeholders_is_text_placed_nowhere() {
        let (text, regions) = grounded("  Une page sans blocs.\n");
        assert_eq!(text, "Une page sans blocs.");
        assert!(regions.is_empty());
        // Square brackets in the text are not placeholders.
        assert_eq!(placeholder("![voir](page 3)"), None);
        assert_eq!(placeholder("![text](1,2,3)"), None);
        assert_eq!(
            placeholder("![aside_text](10, 20, 30, 40)"),
            Some(("aside_text", [10.0, 20.0, 30.0, 40.0], ""))
        );
    }

    /// LightOnOCR's answer through llama.cpp, from the workflow's run on the sample page: each
    /// block's text starts on its placeholder's line.
    #[test]
    fn a_block_whose_text_starts_on_its_placeholder_line_is_placed() {
        let answer = "\n![title](110,51,880,133) # BULLETIN DE L'AFRIQUE OCCIDENTALE FRANÇAISE\n\n\
![text](841,155,922,171) DAKAR 3 MAI 1957 PAGE 1\n\n\
![text](82,268,499,392) Le Gouverneur général a reçu ce matin les délégués.\n\
Ils ont remis un mémoire.\n";
        let (text, regions) = grounded(answer);
        assert_eq!(
            text,
            "# BULLETIN DE L'AFRIQUE OCCIDENTALE FRANÇAISE\n\nDAKAR 3 MAI 1957 PAGE 1\n\n\
Le Gouverneur général a reçu ce matin les délégués.\nIls ont remis un mémoire."
        );
        assert_eq!(regions.len(), 3);
        assert_eq!(
            regions[0].text,
            "BULLETIN DE L'AFRIQUE OCCIDENTALE FRANÇAISE"
        );
        assert!((regions[1].x - 0.841).abs() < 1e-6 && (regions[1].y - 0.155).abs() < 1e-6);
        assert!(!text.contains("]("));
    }

    #[test]
    fn a_line_written_over_and_over_at_the_limit_is_kept_once() {
        let looped = "Titre\nLigne\nLigne\nLigne\nLigne\nLigne";
        assert_eq!(without_runaway(looped), "Titre\nLigne");
        let fine = "Titre\nLigne\nLigne";
        assert_eq!(without_runaway(fine), fine);
    }
}
