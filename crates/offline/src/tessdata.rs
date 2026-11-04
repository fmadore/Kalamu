//! Tesseract's language data: one file per language, from `tesseract-ocr/tessdata_best` at a
//! pinned commit, checked against its SHA-256 like Whisper's models.
//!
//! `tessdata_best` rather than `tessdata_fast`: the most accurate models Tesseract publishes, at
//! two to fifteen megabytes a language, and slower to read with; accuracy matters more than
//! speed for archives. Languages Kalamu lists that Tesseract has no model for, the Latin-script
//! languages of West and Central Africa among them, read with the Latin script model, which
//! covers every language written in Latin letters.

use crate::models::{ModelFile, ModelSet};

/// `tesseract-ocr/tessdata_best` at its latest commit (9 March 2024).
const BEST: &str = "https://raw.githubusercontent.com/tesseract-ocr/tessdata_best/e12c65a915945e4c28e237a9b52bc4a8f39a0cec";
const BEST_SCRIPT: &str = "https://raw.githubusercontent.com/tesseract-ocr/tessdata_best/e12c65a915945e4c28e237a9b52bc4a8f39a0cec/script";

/// The identity of the language data, recorded with each result.
pub const DATA_REVISION: &str = "tessdata_best@e12c65a";

macro_rules! data {
    ($id:literal, $file:literal, $bytes:literal, $sha:literal, [$($lang:literal),*]) => {
        ModelFile {
            id: concat!("tesseract-", $id),
            file: concat!($file, ".traineddata"),
            bytes: $bytes,
            sha256: $sha,
            source: BEST,
            languages: &[$($lang),*],
        }
    };
}

pub static FILES: [ModelFile; 27] = [
    data!(
        "eng",
        "eng",
        15_400_601,
        "8280aed0782fe27257a68ea10fe7ef324ca0f8d85bd2fd145d1c2b560bcb66ba",
        ["en"]
    ),
    data!(
        "fra",
        "fra",
        3_972_885,
        "907743d98915c91a3906dfbf6e48b97598346698fe53aaa797e1a064ffcac913",
        ["fr"]
    ),
    data!(
        "ara",
        "ara",
        12_603_724,
        "ab9d157d8e38ca00e7e39c7d5363a5239e053f5b0dbdb3167dde9d8124335896",
        ["ar"]
    ),
    data!(
        "deu",
        "deu",
        8_628_461,
        "8407331d6aa0229dc927685c01a7938fc5a641d1a9524f74838cdac599f0d06e",
        ["de"]
    ),
    data!(
        "por",
        "por",
        8_159_939,
        "711de9dbb8052067bd42f16b9119967f30bada80d57e2ef24f65d09f531adb04",
        ["pt"]
    ),
    data!(
        "spa",
        "spa",
        13_570_187,
        "e2c1ffdad8b30f26c45d4017a9183d3a7f9aa69e59918be4f88b126fac99ab2c",
        ["es"]
    ),
    data!(
        "ita",
        "ita",
        8_863_635,
        "8df9c89176fb93f56bf4b2d4ede04c01c1f31d4b7697fbd76cc336df700f3f38",
        ["it"]
    ),
    data!(
        "nld",
        "nld",
        8_903_736,
        "92e7a1ad4bf8082e268de57c7823316ec024935702c6ed2a1e473b3a071aa733",
        ["nl"]
    ),
    data!(
        "yor",
        "yor",
        3_736_121,
        "3380579148d53533f0ad740b825248959067f83c3b1903c49d11652054339295",
        ["yo"]
    ),
    data!(
        "swa",
        "swa",
        4_914_855,
        "102ebfc90724b4aebfab5bad0447ab124b7e2e82877b24895b31b0009e6fd0e0",
        ["sw"]
    ),
    data!(
        "amh",
        "amh",
        8_389_639,
        "e29d8041115f924c8fb0ad11c42034fa0f5d76cd974903b56816e06e73767974",
        ["am"]
    ),
    data!(
        "afr",
        "afr",
        12_800_552,
        "6f80b369205a51298a2113c662585d52dfc81bb9331c21845538cec959ebe09a",
        ["af"]
    ),
    data!(
        "fas",
        "fas",
        3_325_955,
        "99e420969b5ddd2cb135b416316a7ed417c59c4faf9e0d28941348f6448114df",
        ["fa"]
    ),
    data!(
        "tur",
        "tur",
        7_456_265,
        "e0c3338dc17503dc7d335a507c9ae01b2b46cfd07561171e1e1ac55d85e8e438",
        ["tr"]
    ),
    data!(
        "urd",
        "urd",
        7_994_323,
        "65af3f6e50a3c3df15d7b405aab5d467e53433207745c8f5be5b8ff1a58a67c8",
        ["ur"]
    ),
    data!(
        "hin",
        "hin",
        11_895_564,
        "bd2e65a2184af08a167b0be2439e91fa5edbc4394399ca2f692b843ae26e78d6",
        ["hi"]
    ),
    data!(
        "rus",
        "rus",
        15_301_764,
        "b617eb6830ffabaaa795dd87ea7fd251adfe9cf0efe05eb9a2e8128b7728d6b6",
        ["ru"]
    ),
    data!(
        "pol",
        "pol",
        11_978_867,
        "e80cc4cefbdface06e9223f43f089556b9dcf104020fbc0a200f6863c57d4405",
        ["pl"]
    ),
    data!(
        "chi_sim",
        "chi_sim",
        13_077_423,
        "4fef2d1306c8e87616d4d3e4c6c67faf5d44be3342290cf8f2f0f6e3aa7e735b",
        ["zh"]
    ),
    data!(
        "jpn",
        "jpn",
        14_330_109,
        "36bdf9ac823f5911e624c30d0553e890b8abc7c31a65b3ef14da943658c40b79",
        ["ja"]
    ),
    data!(
        "kor",
        "kor",
        12_528_128,
        "f888d4038348a0c3d25151e7f452bda0d74ca275b18cab146798bcbb94084fff",
        ["ko"]
    ),
    data!(
        "ind",
        "ind",
        8_253_606,
        "1f6596041ffb4cd5094e5f98764db43cfde04edb8f02b988f90ebc1353ac73b8",
        ["id"]
    ),
    data!(
        "msa",
        "msa",
        8_230_552,
        "2fa693701d2fb3685cf3e0a64bc1bfb97d66a50f93c07b2ebc0ff360ea62621f",
        ["ms"]
    ),
    data!(
        "heb",
        "heb",
        3_704_077,
        "dbaa827aea6bc21215638447f17783a1004987c2d0bf5573d111fee397abdae5",
        ["he"]
    ),
    data!(
        "ell",
        "ell",
        8_945_021,
        "288b4ea00bab450cf39893d22f04a2835a8469b673b5b20dcb39b159d1bbc9b8",
        ["el"]
    ),
    data!(
        "lat",
        "lat",
        9_705_145,
        "60054dd32ac03ebd9b4f87d0665bd491a7ffe0d3ee99ceb91aefaf348b65e94f",
        ["la"]
    ),
    // Every language written in Latin letters, for those Tesseract has no model of its own for.
    ModelFile {
        id: "tesseract-latin",
        file: "Latin.traineddata",
        bytes: 101_402_885,
        sha256: "cf7216998bf7f363f18e3def04cc6b28ae954d2a5a3bdad09f743c9d3538ef35",
        source: BEST_SCRIPT,
        languages: &[
            "ha", "ig", "ff", "wo", "bm", "dyu", "mos", "ee", "fon", "tw", "so", "ln", "zu", "xh",
            "mg",
        ],
    },
];

pub static TESSDATA: ModelSet = ModelSet {
    files: &FILES,
    companion: None,
};

/// The language data that reads a catalogue language.
pub fn for_language(code: &str) -> Option<&'static ModelFile> {
    FILES.iter().find(|f| f.languages.contains(&code))
}

/// Every catalogue language Tesseract can read, in the order of the files.
#[cfg(test)]
pub fn languages() -> impl Iterator<Item = &'static str> {
    FILES.iter().flat_map(|f| f.languages.iter().copied())
}

/// The name Tesseract loads a file under: its file name without `.traineddata`.
pub(crate) fn tesseract_name(file: &ModelFile) -> &'static str {
    file.file.trim_end_matches(".traineddata")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_language_reads_with_one_file() {
        let all: Vec<&str> = languages().collect();
        let mut unique = all.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(all.len(), unique.len());
        assert_eq!(
            for_language("mos").map(|f| f.id),
            Some("tesseract-latin"),
            "Mooré reads with the Latin script model"
        );
        assert_eq!(for_language("fr").map(tesseract_name), Some("fra"));
        for file in &FILES {
            assert_eq!(file.sha256.len(), 64, "{}", file.id);
            assert!(!file.languages.is_empty(), "{}", file.id);
        }
    }
}
