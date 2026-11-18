//! Rows likely the same though far apart in the alphabet (#48). When a field is too long for
//! one request its rows go in alphabetical groups, so `Moussa Ouédraogo` never meets
//! `Ouédraogo, Moussa`, nor `Cheikh Ibrahima Sanou` `Shaykh Ibrahima Sanou`. Here each row gets
//! cheap keys from its forms, and the rows sharing a key that few rows hold are sent again
//! together, in extra groups the model reads like the others. A key is only a reason to look:
//! nothing here says two rows are one.
//!
//! The keys: the name's words in any order ([`sound`] keys, so transliterations meet); the
//! name written as one word or several (`Abdoulaziz`, `Abdoul Aziz`); each word with the
//! initials of the others (`Kanazoé, O.` and `Oumarou Kanazoé`); and each word alone, which a
//! key held by many rows makes worthless, so such a key is dropped. Particles and titles are
//! not words of the name.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::folded;

/// A key more rows than this hold is no evidence (`Moussa`, `Traoré` in a large field): it
/// makes no cluster.
pub(crate) const CLUSTER_ROWS: usize = 10;
/// Rows in one extra group at most, so the model compares clusters it can hold in view.
pub(crate) const GROUP_ROWS: usize = 400;

/// Words that say little about who someone is, so no key of their own: the articles and
/// particles of the languages whose names a field holds in Latin script (Arabic, French,
/// German, Dutch, Portuguese, Italian) and patronymic links (`ibn`, `ould`, `ag`); with
/// [`TITLES`], titles of religion, rank and address in the spellings they are usually found
/// in. Single letters are initials, in neither list. Both are kept short: a word left out
/// only makes a key less precise, a word put in makes every name lose that word.
const PARTICLES: &[&str] = &[
    // Arabic article, also assimilated, and patronymic links.
    "al", "el", "ul", "ad", "ar", "as", "at", "az", "an", "ash", "ibn", "bin", "ben", "ould", "ag",
    // European particles.
    "de", "des", "du", "la", "le", "les", "di", "da", "do", "dos", "das", "van", "von", "vom",
    "der", "den", "zu", "zum",
];

/// Titles: left out of every key, even the name written as one word.
#[rustfmt::skip]
const TITLES: &[&str] = &[
    // Religious titles, in the spellings found in French, English and German.
    "hadj", "hadji", "hajj", "haji", "alhaji", "alhadji", "elhadj", "elhadji",
    "cheikh", "cheick", "cheik", "sheikh", "sheik", "shaykh", "shaikh",
    "imam", "mallam", "malam", "oustaz", "ustaz", "ustadh", "serigne",
    "pere", "abbe", "rev", "mgr", "monseigneur", "soeur", "frere",
    "saint", "sainte", "st", "ste",
    // Address and rank.
    "dr", "pr", "prof", "mr", "mrs", "ms", "mme", "mlle", "me", "sir", "herr", "frau",
];

/// Apostrophes and the marks transliterations use for ʿayn and hamza.
const APOSTROPHES: &[char] = &['\'', '’', 'ʼ', 'ʻ', '`', '´', 'ʿ', 'ʾ'];

/// A form's folded words. An apostrophe after a lone `d` or `l` is a French elision and
/// separates two words (`d'Almeida`); any other joins them (`N'Diaye` is `Ndiaye`).
pub(crate) fn words(form: &str) -> Vec<String> {
    let chars: Vec<char> = form.chars().collect();
    let mut plain = String::with_capacity(form.len());
    for (i, c) in chars.iter().enumerate() {
        if !APOSTROPHES.contains(c) {
            plain.push(*c);
            continue;
        }
        let elision = i >= 1
            && matches!(chars[i - 1], 'd' | 'D' | 'l' | 'L')
            && (i < 2 || !chars[i - 2].is_alphanumeric());
        if elision {
            plain.push(' ');
        }
    }
    folded(&plain)
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// A folded word as it sounds, tolerant of transliteration: `ou`, `u`, `w` and `o` alike;
/// `e`, `i`, `y` and `ai`, `ei` alike; `sh`, `ch`, `sch` and `tch` alike; `dj` and `j`;
/// `kh` and `h`, the `h` itself left out; doubled letters as one; a final `e` silent and a final
/// `d` as `t`. Letters of other scripts are kept as they are.
pub(crate) fn sound(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() > 2 && chars.last() == Some(&'e') {
        chars.pop();
    }
    // An `h` on its own is not heard (`Ahmed`, `Abdullahi`); in `sh`, `kh`, `th` it is part
    // of the letter.
    let chars: Vec<char> = chars
        .iter()
        .enumerate()
        .filter(|&(i, &c)| {
            c != 'h' || (i > 0 && matches!(chars[i - 1], 's' | 'c' | 'k' | 'g' | 'd' | 't' | 'p'))
        })
        .map(|(_, &c)| c)
        .collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let (symbol, used): (&str, usize) = match &chars[i..] {
            ['t' | 's', 'c', 'h', ..] => ("S", 3),
            ['e', 'a', 'u', ..] => ("U", 3),
            ['s' | 'c', 'h', ..] => ("S", 2),
            ['k', 'h', ..] => ("", 2),
            ['g', 'h', ..] => ("G", 2),
            ['d', 'j', ..] => ("J", 2),
            ['d', 'h', ..] => ("D", 2),
            ['t', 'h', ..] | ['d', 't', ..] => ("T", 2),
            ['p', 'h', ..] => ("F", 2),
            ['c', 'k', ..] | ['q', 'u', ..] => ("K", 2),
            ['g', 'u', 'e' | 'i' | 'y', ..] => ("G", 2),
            ['a' | 'o', 'u', ..] | ['o', 'o' | 'e', ..] | ['u', 'e', ..] => ("U", 2),
            ['a' | 'e', 'i' | 'y', ..] => ("I", 2),
            ['a', 'e', ..] => ("A", 2),
            ['c', 'e' | 'i' | 'y', ..] => ("S", 1),
            [c, ..] => {
                let symbol = match c {
                    'a' => "A",
                    'e' | 'i' | 'y' => "I",
                    'o' | 'u' | 'w' => "U",
                    'c' | 'k' | 'q' => "K",
                    'x' => "KS",
                    _ => {
                        out.extend(c.to_uppercase());
                        i += 1;
                        continue;
                    }
                };
                (symbol, 1)
            }
            [] => break,
        };
        out.push_str(symbol);
        i += used;
    }
    let mut key = String::with_capacity(out.len());
    for c in out.chars() {
        if !key.ends_with(c) {
            key.push(c);
        }
    }
    if key.ends_with('D') {
        key.pop();
        key.push('T');
    }
    key
}

/// The keys of one form: none when it holds no word of a name.
pub(crate) fn keys(form: &str) -> BTreeSet<String> {
    let words = words(form);
    let mut out = BTreeSet::new();
    push_keys(&words, &mut out);
    // `kh` is written `k` as often as `h` (`Mokhtar`, `Moctar`): both are tried.
    if words.iter().any(|w| w.contains("kh")) {
        let k: Vec<String> = words.iter().map(|w| w.replace("kh", "k")).collect();
        push_keys(&k, &mut out);
    }
    out
}

fn push_keys(words: &[String], out: &mut BTreeSet<String>) {
    let mut full: Vec<(char, String)> = Vec::new();
    let mut initials: Vec<char> = Vec::new();
    let mut spoken: Vec<&str> = Vec::new();
    for word in words {
        let mut letters = word.chars();
        let (Some(first), rest) = (letters.next(), letters.next()) else {
            continue;
        };
        if rest.is_none() {
            if first.is_alphabetic() {
                initials.push(first);
            }
        } else if !TITLES.contains(&word.as_str()) {
            spoken.push(word);
            if !PARTICLES.contains(&word.as_str()) {
                full.push((first, sound(word)));
            }
        }
    }
    if full.is_empty() {
        return;
    }
    initials.sort_unstable();
    // The whole name in any order, initials included.
    let mut bag: Vec<String> = full.iter().map(|(_, s)| s.clone()).collect();
    bag.extend(initials.iter().map(char::to_string));
    bag.sort_unstable();
    bag.dedup();
    out.insert(format!("b:{}", bag.join(" ")));
    // Written as one word or several, particles included (`Abd el Kader`, `Abdelkader`).
    if spoken.len() > 1 && initials.is_empty() {
        out.insert(format!("b:{}", sound(&spoken.concat())));
    }
    for (i, (_, word)) in full.iter().enumerate() {
        let mut others: Vec<char> = full
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .map(|(_, (first, _))| *first)
            .collect();
        // A word and the initials of the other words: `f:` from a name written in full, `i:`
        // from one written with initials, which is what meets it.
        let kind = if initials.is_empty() { "f" } else { "i" };
        others.extend(&initials);
        if !others.is_empty() {
            others.sort_unstable();
            let others: String = others.into_iter().collect();
            out.insert(format!("{kind}:{word}|{others}"));
        }
        if !word.chars().all(|c| c.is_ascii_digit()) {
            out.insert(format!("w:{word}"));
        }
    }
}

/// The extra groups of a field too long for one request, as indices into its rows (sorted as
/// sent): clusters of rows sharing a key no more than [`CLUSTER_ROWS`] rows hold, leaving out
/// any one alphabetical group (`windows`) already holds whole, never with two rows the
/// researcher kept apart (`apart`, pairs of indices, lower first). The clusters of the
/// strongest keys go first (the whole name, then initials, then one word), the smallest first
/// among them, until the extra rows would outnumber the field's own, so a request costs at most
/// twice the alphabetical groups; they are packed into groups of at most [`GROUP_ROWS`] rows and
/// `budget` tokens. Nothing when the field fits in one request. The same for the same rows,
/// whatever order they came in.
pub(crate) fn groups(
    forms: &[Vec<&str>],
    apart: &BTreeSet<(usize, usize)>,
    windows: &[Range<usize>],
    costs: &[usize],
    budget: usize,
) -> Vec<Vec<usize>> {
    if windows.len() < 2 {
        return Vec::new();
    }
    let mut holders: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    for (row, forms) in forms.iter().enumerate() {
        for key in forms.iter().flat_map(|f| keys(f)) {
            holders.entry(key).or_default().insert(row);
        }
    }
    let kept_apart = |a: usize, b: usize| apart.contains(&(a.min(b), a.max(b)));
    let covered = |rows: &[usize]| {
        let (first, last) = (rows[0], rows[rows.len() - 1]);
        windows.iter().any(|w| w.start <= first && last < w.end)
    };

    // Each cluster with the rank of its strongest key: the whole name, then initials, then a
    // word alone.
    let mut clusters: BTreeMap<Vec<usize>, u8> = BTreeMap::new();
    for (key, rows) in &holders {
        let rank = match key.as_bytes()[0] {
            b'b' => 0,
            b'i' => 1,
            _ => 2,
        };
        // Full names alike only by their initials are no reason to look; a full name and one
        // written with those initials are.
        let rows: BTreeSet<usize> = match key.strip_prefix("i:") {
            _ if key.starts_with("f:") => continue,
            Some(rest) => rows
                .iter()
                .chain(holders.get(&format!("f:{rest}")).into_iter().flatten())
                .copied()
                .collect(),
            None => rows.clone(),
        };
        if rows.len() < 2 || rows.len() > CLUSTER_ROWS {
            continue;
        }
        // Rows kept apart go into separate clusters, each row into the first it may join.
        let mut parts: Vec<Vec<usize>> = Vec::new();
        for row in rows {
            match parts
                .iter_mut()
                .find(|p| p.iter().all(|&other| !kept_apart(row, other)))
            {
                Some(part) => part.push(row),
                None => parts.push(vec![row]),
            }
        }
        for part in parts {
            if part.len() > 1 && !covered(&part) {
                let best = clusters.entry(part).or_insert(rank);
                *best = (*best).min(rank);
            }
        }
    }
    // A cluster another holds whole, by a key as strong, adds nothing.
    let all: Vec<(Vec<usize>, u8)> = clusters.into_iter().collect();
    let mut holding: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, (cluster, _)) in all.iter().enumerate() {
        for &row in cluster {
            holding.entry(row).or_default().push(i);
        }
    }
    let mut kept: Vec<&(Vec<usize>, u8)> = all
        .iter()
        .filter(|(c, rank)| {
            !holding[&c[0]].iter().any(|&o| {
                let (other, strength) = &all[o];
                strength <= rank
                    && other.len() > c.len()
                    && c.iter().all(|r| other.binary_search(r).is_ok())
            })
        })
        .collect();
    kept.sort_by(|(a, x), (b, y)| {
        x.cmp(y)
            .then_with(|| a.len().cmp(&b.len()))
            .then_with(|| a.cmp(b))
    });

    let limit = forms.len();
    let mut packs: Vec<(Vec<usize>, usize)> = Vec::new();
    let mut sent = 0;
    for (cluster, _) in kept {
        let fits = |(rows, tokens): &(Vec<usize>, usize)| {
            let new: Vec<usize> = cluster
                .iter()
                .copied()
                .filter(|r| !rows.contains(r))
                .collect();
            let cost: usize = new.iter().map(|&r| costs[r]).sum();
            rows.len() + new.len() <= GROUP_ROWS
                && tokens + cost <= budget
                && new
                    .iter()
                    .all(|&r| rows.iter().all(|&other| !kept_apart(r, other)))
        };
        let index = match packs.iter().position(fits) {
            Some(index) => index,
            None => {
                packs.push((Vec::new(), 0));
                packs.len() - 1
            }
        };
        let (rows, tokens) = &mut packs[index];
        let new: Vec<usize> = cluster
            .iter()
            .copied()
            .filter(|r| !rows.contains(r))
            .collect();
        if sent + new.len() > limit {
            if rows.is_empty() {
                packs.pop();
            }
            continue;
        }
        sent += new.len();
        *tokens += new.iter().map(|&r| costs[r]).sum::<usize>();
        rows.extend(new);
    }
    packs.into_iter().map(|(rows, _)| rows).collect()
}
