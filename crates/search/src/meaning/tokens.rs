//! The tokens a text would be billed, estimated from its characters before anything is sent:
//! about four characters a token in a Latin script, two in other alphabets (Arabic, Cyrillic,
//! Greek), one in scripts written without spaces (Chinese, Japanese, Thai). Providers count with
//! their own tokenizers, so the interface calls this an estimate; the run records what they
//! billed.

use crate::fold::UNSPACED;

pub fn estimated_tokens(text: &str) -> u64 {
    let mut quarters: u64 = 0;
    for c in text.chars() {
        let code = c as u32;
        quarters += if c.is_ascii() {
            1
        } else if UNSPACED.iter().any(|&(a, b)| (a..=b).contains(&code))
            || (0xAC00..=0xD7AF).contains(&code)
        {
            4
        } else {
            2
        };
    }
    quarters.div_ceil(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_text_is_about_four_characters_a_token_and_chinese_one() {
        assert_eq!(estimated_tokens("abcdefgh"), 2);
        assert_eq!(estimated_tokens("北京大学"), 4);
        assert_eq!(estimated_tokens("مسجد"), 2);
        assert_eq!(estimated_tokens(""), 0);
    }
}
