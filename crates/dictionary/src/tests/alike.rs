//! Rows likely the same though far apart in the alphabet: the keys that bring them together,
//! and the extra groups they make, bounded, deterministic, never with rows kept apart. All
//! names are invented.

use std::collections::BTreeSet;
use std::ops::Range;

use kalamu_core::dictionary::proposal;

use crate::alike::{CLUSTER_ROWS, GROUP_ROWS, groups, keys, sound, words};
use crate::requests::windows;

use super::*;

/// Whether two forms share a key, a full name's initials meeting only a form written with
/// them.
fn meet(a: &str, b: &str) -> bool {
    let (ka, kb) = (keys(a), keys(b));
    let initials = |x: &BTreeSet<String>, y: &BTreeSet<String>| {
        x.iter()
            .filter_map(|k| k.strip_prefix("i:"))
            .any(|k| y.contains(&format!("f:{k}")))
    };
    ka.intersection(&kb).any(|k| !k.starts_with("f:")) || initials(&ka, &kb) || initials(&kb, &ka)
}

#[test]
fn words_fold_accents_punctuation_and_apostrophes() {
    assert_eq!(words("Ouédraogo, Moussa"), ["ouedraogo", "moussa"]);
    assert_eq!(words("N'Diaye"), ["ndiaye"]);
    assert_eq!(words("Saʿid Kaboré"), ["said", "kabore"]);
    assert_eq!(words("Jeanne d’Almeida"), ["jeanne", "d", "almeida"]);
    assert_eq!(words("Abd-el-Kader"), ["abd", "el", "kader"]);
}

#[test]
fn the_sound_key_is_tolerant_of_transliteration() {
    let alike = [
        // ou, u, w and o
        ("Moussa", "Musa"),
        ("Ouagadougou", "Wagadugu"),
        ("Ouédraogo", "Wedraogo"),
        ("Oumarou", "Umaru"),
        ("Ousmane", "Usman"),
        // kh and h, h left out, final e
        ("Khadidja", "Hadija"),
        ("Hassane", "Assan"),
        ("Abdoulaye", "Abdullahi"),
        // dj and j, doubled letters
        ("Tidjani", "Tijani"),
        ("Alassane", "Alasane"),
        ("Sall", "Sal"),
        // ai, ay, ei, ey, y and i
        ("Seydou", "Saïdou"),
        ("Seydou", "Seidu"),
        ("Yahya", "Yaya"),
        ("Gueye", "Gaye"),
        ("Meyer", "Maier"),
        // sh, ch, sch, tch
        ("Chaibou", "Shaibu"),
        ("Schirmer", "Shirmer"),
        ("Tchakoura", "Chakoura"),
        // German spellings and final d
        ("Müller", "Mueller"),
        ("Goethe", "Göthe"),
        ("Schmidt", "Schmitt"),
        ("Mohamed", "Mohammet"),
        // c before e and i
        ("Cissé", "Sissé"),
    ];
    for (a, b) in alike {
        let (a_, b_) = (sound(&words(a)[0]), sound(&words(b)[0]));
        assert_eq!(a_, b_, "{a} and {b}");
    }
    let apart = [
        ("Traoré", "Touré"),
        ("Diallo", "Diarra"),
        ("Konaté", "Koné"),
        ("Sow", "Sy"),
        ("Keita", "Kane"),
        ("Meyer", "Müller"),
        ("Diop", "Diouf"),
        ("Cissé", "Sissoko"),
        ("Zongo", "Songo"),
        ("Bamba", "Banda"),
        ("Mamadou", "Mamadi"),
    ];
    for (a, b) in apart {
        let (a_, b_) = (sound(&words(a)[0]), sound(&words(b)[0]));
        assert_ne!(a_, b_, "{a} and {b}");
    }
}

#[test]
fn names_meet_whatever_their_word_order_titles_initials_and_spelling() {
    let alike = [
        // Word order, accents, punctuation.
        ("Ouédraogo, Moussa", "Moussa Ouedraogo"),
        ("KABORÉ Aminata", "Aminata Kaboré"),
        // Titles and particles left out.
        ("Cheikh Ibrahima Sanou", "Shaykh Ibrahima Sanou"),
        ("El Hadj Oumarou Kanazoé", "Oumarou Kanazoe"),
        ("Dr Friedrich von Albersdorf", "Friedrich Albersdorf"),
        ("Mgr Joseph Ouattara", "Joseph Watara"),
        // Initials against full forenames.
        ("El Hadj Oumarou Kanazoé", "Kanazoé, O."),
        ("A. B. Sawadogo", "Boukary Adama Sawadogo"),
        // One word or several, kh written k.
        ("Abdoul Aziz Zerbo", "Abdoulaziz Zerbo"),
        ("Abd-el-Kader Barry", "Abdelkader Barry"),
        ("Mokhtar Dicko", "Moctar Dicko"),
        // A rare word alone, and transliterated places.
        ("Kanazoé", "Oumarou Kanazoé"),
        ("Ouagadougou", "Wagadugu"),
        ("N'Diaye Fatou", "Fatou Ndiaye"),
    ];
    for (a, b) in alike {
        assert!(
            meet(a, b),
            "{a} and {b} should meet: {:?} {:?}",
            keys(a),
            keys(b)
        );
    }
    let apart = [
        ("Moussa Traoré", "Moussa Touré"),
        ("Ibrahim Diallo", "Ibrahim Diarra"),
        ("Kanazoé, O.", "Kanazoé, A."),
        ("Saint-Louis", "Saint-Denis"),
        ("El Hadj", "Cheikh"),
        ("Imam Konaté", "Imam Koné"),
        ("Hans Meyer", "Hans Müller"),
    ];
    for (a, b) in apart {
        // They share at most a word, which a common forename (`Moussa`) makes worthless.
        let shared: Vec<String> = keys(a).intersection(&keys(b)).cloned().collect();
        assert!(
            shared
                .iter()
                .all(|k| k.starts_with("w:") || k.starts_with("f:")),
            "{a} and {b} share {shared:?}"
        );
        if shared.is_empty() {
            assert!(!meet(a, b), "{a} and {b}");
        }
    }
    // Neither particles nor titles are words of a name: alone they have no key.
    assert!(keys("El Hadj").is_empty());
    assert!(keys("Saint-Louis").contains("w:LUIS"));
    assert!(!keys("Saint-Louis").contains("w:SAINT"));
    assert!(!meet("Saint-Louis", "Saint-Denis"));
}

/// A field of a few hundred invented names: forenames and surnames combined, with some written
/// surname first, with initials, a title or another spelling, which the alphabet sets apart.
fn field() -> Vec<String> {
    names()
        .into_iter()
        .flat_map(|(a, b)| [Some(a), b])
        .flatten()
        .collect()
}

/// The names that have a variant, beside it.
fn variants() -> Vec<(String, String)> {
    names()
        .into_iter()
        .filter_map(|(a, b)| Some((a, b?)))
        .collect()
}

/// Each name, and the variant written beside it when there is one.
fn names() -> Vec<(String, Option<String>)> {
    let forenames = [
        "Moussa",
        "Aminata",
        "Boukary",
        "Salif",
        "Mariam",
        "Issouf",
        "Rasmané",
        "Awa",
        "Lassina",
        "Hamidou",
        "Pascal",
        "Clémence",
        "Friedrich",
        "Yacouba",
        "Seydou",
        "Ousmane",
        "Fatimata",
        "Karim",
        "Adama",
        "Bintou",
    ];
    let surnames = [
        "Ouédraogo",
        "Kaboré",
        "Zerbo",
        "Sanou",
        "Kanazoé",
        "Tiendrébéogo",
        "Barry",
        "Dicko",
        "Albersdorf",
        "Nikiéma",
        "Compaoré",
        "Yaméogo",
        "Bationo",
        "Somé",
    ];
    let mut out = Vec::new();
    for (i, f) in forenames.iter().enumerate() {
        for (j, s) in surnames.iter().enumerate() {
            let name = format!("{f} {s}");
            let variant = match (i + j) % 7 {
                0 => Some(format!("{s}, {f}")),
                1 => Some(format!("{s}, {}.", &f[..1])),
                2 => Some(format!("Cheikh {f} {s}")),
                3 if f.contains("ou") => Some(format!("{} {s}", f.replace("ou", "u"))),
                _ => None,
            };
            out.push((name, variant));
        }
    }
    out
}

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort_by_cached_key(|n| (folded(n), n.clone()));
    names
}

/// The extra groups of `names` in requests of `budget`, as the names they hold.
fn extra(
    names: &[String],
    apart: &BTreeSet<(usize, usize)>,
    budget: usize,
) -> (Vec<Range<usize>>, Vec<Vec<usize>>) {
    let forms: Vec<Vec<&str>> = names.iter().map(|n| vec![n.as_str()]).collect();
    let costs: Vec<usize> = names.iter().map(|n| n.chars().count() / 3 + 4).collect();
    let ranges = windows(&costs, budget);
    let groups = groups(&forms, apart, &ranges, &costs, budget);
    (ranges, groups)
}

#[test]
fn far_apart_rows_go_in_bounded_groups_the_same_whatever_their_order() {
    let names = sorted(field());
    let (ranges, groups_) = extra(&names, &BTreeSet::new(), 800);
    let rows = names.len();
    assert!((300..500).contains(&rows), "{rows} rows");
    assert!(ranges.len() > 4, "{} alphabetical groups", ranges.len());
    assert!(!groups_.is_empty());

    // Bounded: no group beyond its size, the extra rows no more than the field's own.
    let sent: usize = groups_.iter().map(Vec::len).sum();
    assert!(sent <= rows, "{sent} extra rows for {rows}");
    assert!(groups_.iter().all(|g| g.len() <= GROUP_ROWS));
    for group in &groups_ {
        let unique: BTreeSet<_> = group.iter().collect();
        assert_eq!(unique.len(), group.len(), "a row once per group");
    }
    println!(
        "{rows} rows, {} alphabetical groups, {} extra groups of {sent} rows",
        ranges.len(),
        groups_.len()
    );

    // Every name meets its variant: in an alphabetical group, or else in an extra one.
    let at = |n: &str| names.iter().position(|x| x == n).unwrap();
    let mut far = 0;
    for (name, variant) in variants() {
        let (a, b) = (at(&name), at(&variant));
        if ranges.iter().any(|r| r.contains(&a) && r.contains(&b)) {
            continue;
        }
        far += 1;
        assert!(
            groups_.iter().any(|g| g.contains(&a) && g.contains(&b)),
            "{name} and {variant}"
        );
    }
    println!("{far} variants far apart in the alphabet");
    assert!(far > 30);

    // Not one cluster lies inside an alphabetical group: those are already compared.
    for group in &groups_ {
        assert!(
            !ranges
                .iter()
                .any(|r| group.iter().all(|i| r.contains(i)) && group.len() <= CLUSTER_ROWS)
        );
    }

    // The same groups, by name, whatever order the rows came in.
    let mut shuffled = field();
    shuffled.reverse();
    shuffled.rotate_left(97);
    let again = sorted(shuffled);
    let (_, groups_again) = extra(&again, &BTreeSet::new(), 800);
    let by_name = |names: &[String], groups: &[Vec<usize>]| -> Vec<Vec<String>> {
        groups
            .iter()
            .map(|g| g.iter().map(|&i| names[i].clone()).collect())
            .collect()
    };
    assert_eq!(by_name(&names, &groups_), by_name(&again, &groups_again));
    // And nothing extra when the field fits in one request.
    assert!(extra(&names, &BTreeSet::new(), INPUT_TOKENS).1.is_empty());
}

#[test]
fn rows_kept_apart_are_never_in_one_extra_group() {
    let names = sorted(field());
    let at = |n: &str| names.iter().position(|x| x == n).unwrap();
    let (a, b) = (at("Moussa Ouédraogo"), at("Ouédraogo, Moussa"));
    let (c, d) = (at("Cheikh Moussa Zerbo"), at("Moussa Zerbo"));
    let apart: BTreeSet<(usize, usize)> = [(a.min(b), a.max(b)), (c.min(d), c.max(d))].into();
    let (_, groups_) = extra(&names, &apart, 800);
    assert!(!groups_.is_empty());
    for group in &groups_ {
        for &(x, y) in &apart {
            assert!(!(group.contains(&x) && group.contains(&y)), "{group:?}");
        }
    }
}

#[test]
fn a_field_too_long_for_one_request_also_sends_its_rows_alike() {
    let f = fixture();
    let letter = f.document("letter");
    let names = field();
    let people: Vec<&str> = names.iter().map(String::as_str).collect();
    f.analysis(std::slice::from_ref(&letter), &[&[(&people, &[])]]);
    let ouedraogo = entry_of_form(&f, "Moussa Ouédraogo");
    let reversed = entry_of_form(&f, "Ouédraogo, Moussa");
    f.dictionary
        .keep_apart(
            f.project,
            "people",
            &Item::Entry { id: ouedraogo },
            &Item::Entry { id: reversed },
        )
        .unwrap();

    let prepared = f
        .dictionary
        .prepare_within(f.project, None, "people", 800)
        .unwrap();
    let alphabetical: Vec<&PreparedGroup> = prepared
        .groups
        .iter()
        .filter(|g| !g.file_name.contains("-alike-"))
        .collect();
    let alike: Vec<&PreparedGroup> = prepared
        .groups
        .iter()
        .filter(|g| g.file_name.contains("-alike-"))
        .collect();
    let n = alphabetical.len();
    assert!(n > 1);
    assert_eq!(alphabetical[0].file_name, format!("people-1-of-{n}.txt"));
    assert!(!alike.is_empty());
    assert_eq!(
        alike[0].file_name,
        format!("people-alike-1-of-{}.txt", alike.len())
    );
    // Written like the others, every row numbered with its line, and read back by its forms.
    for group in &alike {
        assert!(group.text.starts_with("Field: People\n\nRows:\nR1. "));
        let numbered = group
            .text
            .lines()
            .filter(|l| l.starts_with('R') && l[1..].starts_with(|c: char| c.is_ascii_digit()))
            .count();
        assert_eq!(numbered, group.rows.len());
        // The pair kept apart is never sent together, so never said either.
        assert!(!group.text.contains("Kept apart"));
    }
    let with = |form: &str| {
        alike
            .iter()
            .filter(|g| g.rows.iter().any(|r| r.iter().any(|x| x == form)))
            .count()
    };
    assert!(with("Cheikh Moussa Zerbo") > 0);
    // Every group's text is its own, so none is one source twice.
    let texts: BTreeSet<&str> = prepared.groups.iter().map(|g| g.text.as_str()).collect();
    assert_eq!(texts.len(), prepared.groups.len());
}

/// The entry a form becomes when reviewed alone.
fn entry_of_form(f: &Fixture, form_: &str) -> kalamu_core::EntryId {
    f.dictionary
        .set_reviewed(f.project, "people", &[form(form_)], true)
        .unwrap();
    f.store
        .entries(f.project)
        .unwrap()
        .into_iter()
        .find(|e| e.forms.iter().any(|x| x == form_))
        .unwrap()
        .id
}

#[test]
fn a_field_that_fits_one_request_is_sent_as_before_byte_for_byte() {
    let f = fixture();
    let letter = f.document("letter");
    f.analysis(
        std::slice::from_ref(&letter),
        &[&[(
            &["Moussa Traoré", "Ouédraogo, Moussa", "Moussa Ouédraogo"],
            &[],
        )]],
    );
    let prepared = f
        .dictionary
        .prepare_proposals(f.project, None, "people")
        .unwrap();
    assert_eq!(prepared.groups.len(), 1);
    assert_eq!(prepared.groups[0].file_name, "people.txt");
    // The text as Kalamu wrote it before rows alike were sent, so cached answers still hit.
    assert_eq!(
        prepared.groups[0].text,
        "Field: People\n\nRows:\n\
         R1. Moussa Ouédraogo | named in 1 document: \"letter\"\n\
         R2. Moussa Traoré | named in 1 document: \"letter\"\n\
         R3. Ouédraogo, Moussa | named in 1 document: \"letter\"\n"
    );
}

#[test]
fn a_proposal_two_groups_reach_is_listed_once() {
    let f = fixture();
    let letter = f.document("letter");
    f.analysis(
        std::slice::from_ref(&letter),
        &[&[(
            &["Moussa Ouédraogo", "Nikiéma Salif", "Ouédraogo, Moussa"],
            &[],
        )]],
    );
    let rows = |forms: &[&str]| -> Vec<Vec<String>> {
        forms.iter().map(|f| vec![f.to_string()]).collect()
    };
    // An alphabetical group and a group of rows alike, both holding the pair.
    let prepared = Prepared {
        namespace: "people".into(),
        label: "People".into(),
        rows: 3,
        groups: vec![
            PreparedGroup {
                file_name: "people-1-of-1.txt".into(),
                text: "alphabetical".into(),
                rows: rows(&["Moussa Ouédraogo", "Nikiéma Salif", "Ouédraogo, Moussa"]),
            },
            PreparedGroup {
                file_name: "people-alike-1-of-1.txt".into(),
                text: "alike".into(),
                rows: rows(&["Ouédraogo, Moussa", "Moussa Ouédraogo"]),
            },
        ],
    };
    let sources: Vec<SourceFile> = ["alphabetical", "alike"]
        .iter()
        .map(|name| SourceFile {
            kind: InputKind::Text,
            ..source(name)
        })
        .collect();
    let setup = Setup {
        id: SetupId::new(),
        label: "model".into(),
        provider: ProviderId::new("gemini"),
        model: "model".into(),
        prompt: PromptSnapshot {
            preset_id: Some("dictionary.merges".into()),
            preset_name: "Dictionary: proposed merges".into(),
            template: String::new(),
            text: String::new(),
            user_prompt: String::new(),
            schema: Some(proposal::schema()),
        },
        options: SetupOptions::default(),
        merge: None,
        pages_cap: None,
        windows: None,
    };
    let job = Job {
        id: JobId::new(),
        name: "Dictionary: merges in People".into(),
        task: Task::Analysis,
        created_at: Timestamp::now(),
        sources: sources.clone(),
        setups: vec![setup.clone()],
        settings: JobSettings::default(),
        output: OutputSettings::default(),
        status: JobStatus::Completed,
    };
    f.store.create_job(&job).unwrap();
    // Each group names the pair by its own references.
    for (index, (source, refs)) in sources.iter().zip([["R1", "R3"], ["R2", "R1"]]).enumerate() {
        let key = format!("{}-{index}", job.id);
        let group = vec![
            answer(
                proposal::ROWS,
                FieldValue::List(refs.iter().map(|r| r.to_string()).collect()),
            ),
            answer(proposal::LABEL, FieldValue::Text("Moussa Ouédraogo".into())),
            answer(proposal::REASON, FieldValue::Text("Word order.".into())),
            answer(proposal::CERTAINTY, FieldValue::Text("likely".into())),
        ];
        f.store
            .put_result(&StoredResult {
                cache_key: key.clone(),
                provider: ProviderId::new("gemini"),
                model: "model".into(),
                output: ProcessOutput {
                    fields: vec![answer(proposal::GROUPS, FieldValue::Entries(vec![group]))],
                    ..ProcessOutput::default()
                },
                request: Vec::new(),
                cost_usd: Some(0.01),
                duration_ms: 1,
                created_at: Timestamp::now(),
            })
            .unwrap();
        f.store
            .upsert_piece(&PieceState {
                job_id: job.id,
                setup_id: setup.id,
                source_id: source.id,
                index: index as u32,
                piece: PieceRef::Whole,
                status: PieceStatus::Done,
                cache_key: Some(key),
                error: None,
                attempts: 1,
                updated_at: Timestamp::now(),
            })
            .unwrap();
    }
    let sha: Vec<String> = sources.iter().map(|s| s.sha256.clone()).collect();
    f.dictionary
        .record_request(f.project, None, &prepared, job.id, &sha)
        .unwrap();

    let view = f.dictionary.proposals(f.project).unwrap();
    assert_eq!(view.requests[0].answered, 2);
    assert_eq!(view.requests[0].proposals, 1);
    assert_eq!(view.proposals.len(), 1);
    let members: Vec<&str> = view.proposals[0]
        .members
        .iter()
        .map(|m| m.label.as_str())
        .collect();
    assert_eq!(members, ["Moussa Ouédraogo", "Ouédraogo, Moussa"]);
}
