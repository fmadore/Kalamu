//! The photograph checks on photographs made from the two sample pages (`samples/`): a
//! handwritten report of 1755 and a printed bulletin. No real retakes are committed, so each
//! case is made here, at the size the checks work at: the same page photographed again (turned
//! a degree or two, nearer, shifted, in other light, saved again as JPEG), the same page
//! blurred (out of focus, or moved), and pages of one register or one newspaper that share
//! their ruling or layout but not what is written on them. The thresholds were set on these;
//! real reading-room photographs are still to be tried (see the decision of 20 November 2026).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::OnceLock;

use image::{GrayImage, ImageFormat};

use super::compare::{Verdict, compare, tiny_likeness};
use super::grey::{Grey, fit};
use super::measure::{Measured, WORK_SIDE, measure};
use super::*;

fn sample(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("samples")
        .join(name);
    std::fs::read(path).unwrap()
}

fn to_grey(img: &GrayImage) -> Grey {
    Grey::from_u8(img.width() as usize, img.height() as usize, img.as_raw())
}

fn to_image(g: &Grey) -> GrayImage {
    GrayImage::from_raw(g.w as u32, g.h as u32, g.to_u8()).unwrap()
}

/// A sample page decoded and reduced to the working size, once for every test.
fn page(name: &'static str) -> &'static Grey {
    static HAND: OnceLock<Grey> = OnceLock::new();
    static PRINT: OnceLock<Grey> = OnceLock::new();
    let cell = if name == "dodington-1755.jpg" {
        &HAND
    } else {
        &PRINT
    };
    cell.get_or_init(|| {
        let img = image::load_from_memory(&sample(name)).unwrap().into_luma8();
        let g = to_grey(&img);
        let (w, h) = fit(g.w, g.h, WORK_SIDE);
        g.reduce(w, h)
    })
}

fn handwritten() -> &'static Grey {
    page("dodington-1755.jpg")
}

fn printed() -> &'static Grey {
    page("bulletin-page.jpg")
}

/// Saved again as a JPEG of `quality`, and read back.
fn recompressed(g: &Grey, quality: u8) -> Grey {
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality)
        .encode_image(&to_image(g))
        .unwrap();
    to_grey(
        &image::load_from_memory_with_format(&out, ImageFormat::Jpeg)
            .unwrap()
            .into_luma8(),
    )
}

/// The page photographed again: turned by `angle` degrees, `scale` nearer, moved by `shift`
/// pixels, in light `gain` times as bright plus `offset`, saved as JPEG. What falls outside the
/// first photograph is the table under the page, a dark grey.
fn retake(g: &Grey, angle: f32, scale: f32, shift: (f32, f32), gain: f32, offset: f32) -> Grey {
    let moved = g.warp(angle, scale, shift);
    let px = moved
        .px
        .iter()
        .map(|&v| {
            let v = if v.is_nan() { 60.0 } else { v };
            (v * gain + offset).clamp(0.0, 255.0)
        })
        .collect();
    recompressed(&Grey::new(g.w, g.h, px), 82)
}

/// The page out of focus: a Gaussian blur of `sigma` pixels at the working size.
fn out_of_focus(g: &Grey, sigma: f32) -> Grey {
    recompressed(&to_grey(&image::imageops::blur(&to_image(g), sigma)), 85)
}

/// The page moved while taken: each pixel the average of `length` pixels along a line.
fn moved(g: &Grey, length: usize) -> Grey {
    let mut px = vec![0.0f32; g.w * g.h];
    for y in 0..g.h {
        for x in 0..g.w {
            let (x0, x1) = (x.saturating_sub(length / 2), (x + length / 2 + 1).min(g.w));
            let sum: f32 = (x0..x1).map(|i| g.at(i, y)).sum();
            px[y * g.w + x] = sum / (x1 - x0) as f32;
        }
    }
    recompressed(&Grey::new(g.w, g.h, px), 85)
}

/// A page of a register: printed ruling (a heading band, rows, four columns) on slightly
/// shaded paper, its rows filled in by hand with lines cut from the handwritten sample, chosen
/// by `seed`. Two seeds give two pages of the same register: the same ruling and heading,
/// different entries. Rows past `filled` are left blank.
fn register_page(seed: u64, filled: usize) -> Grey {
    let (w, h) = (1152usize, 1536usize);
    let hand = handwritten();
    let heading = printed();
    let mut px = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            px[y * w + x] = 228.0 - 18.0 * (x as f32 / w as f32) - 10.0 * (y as f32 / h as f32);
        }
    }
    let mut page = Grey::new(w, h, px);
    // The printed heading: the bulletin's masthead, the same on every page.
    for y in 0..150 {
        for x in 60..w - 60 {
            let v = heading.at(x * heading.w / w, 40 + y);
            let p = &mut page.px[(y + 30) * w + x];
            *p = p.min(v);
        }
    }
    let rows: Vec<usize> = (0..26).map(|r| 230 + r * 48).collect();
    let cols = [80usize, 200, 640, 900, w - 80];
    let ink = |page: &mut Grey, x: usize, y: usize, v: f32| {
        let p = &mut page.px[y * w + x];
        *p = p.min(v);
    };
    for &y in &rows {
        for x in cols[0]..=cols[4] {
            ink(&mut page, x, y, 90.0);
            ink(&mut page, x, y + 1, 110.0);
        }
    }
    for &x in &cols {
        for y in rows[0]..=rows[rows.len() - 1] {
            ink(&mut page, x, y, 90.0);
            ink(&mut page, x + 1, y, 110.0);
        }
    }
    // Entries: in each filled row, each column after the first takes a line of handwriting
    // from a place in the sample that the seed picks.
    let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let mut next = |below: usize| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((state >> 33) as usize) % below
    };
    for (r, &y) in rows.iter().enumerate().take(filled.min(rows.len() - 1)) {
        for c in 0..cols.len() - 1 {
            let (x0, x1) = (cols[c] + 6, cols[c + 1] - 6);
            let len = if c == 0 {
                x1 - x0
            } else {
                (x1 - x0) * (5 + next(5)) / 10
            };
            let (sx, sy) = (60 + next(hand.w - len - 120), 200 + next(hand.h - 400));
            let _ = r;
            for dy in 0..40 {
                for dx in 0..len {
                    let v = hand.at(sx + dx, sy + dy);
                    // Only the ink, not the old paper.
                    if v < 150.0 {
                        ink(&mut page, x0 + dx, y + 4 + dy, v);
                    }
                }
            }
        }
    }
    recompressed(&page, 88)
}

/// Another issue of the printed bulletin: the same masthead and columns, each column's text
/// moved down by a different number of lines, so different words stand at every place.
fn other_issue(g: &Grey) -> Grey {
    let mut out = g.clone();
    let top = g.h / 6;
    let columns = 3;
    for c in 0..columns {
        let (x0, x1) = (c * g.w / columns, (c + 1) * g.w / columns);
        let move_by = (g.h - top) / 5 + c * 37;
        for y in top..g.h {
            let from = top + (y - top + move_by) % (g.h - top);
            for x in x0..x1 {
                out.px[y * g.w + x] = g.at(x, from);
            }
        }
    }
    recompressed(&out, 85)
}

fn verdict(a: &Grey, b: &Grey) -> Verdict {
    compare(&measure(a), &measure(b))
}

fn sharpness_of(g: &Grey) -> f32 {
    measure(g).sharpness.unwrap()
}

#[test]
fn a_page_photographed_again_is_the_same_page() {
    for (name, page) in [("handwritten", handwritten()), ("printed", printed())] {
        for (angle, scale, shift, gain, offset) in [
            (0.0, 1.0, (0.0, 0.0), 1.0, 0.0),
            (1.5, 1.02, (18.0, -12.0), 0.85, 10.0),
            (-2.0, 0.97, (-25.0, 20.0), 1.15, -15.0),
            (0.7, 1.04, (40.0, 30.0), 0.7, 30.0),
        ] {
            let again = retake(page, angle, scale, shift, gain, offset);
            let v = verdict(page, &again);
            assert!(
                v.same,
                "{name} turned {angle}°, scale {scale}, {shift:?}: {v:?}"
            );
        }
    }
}

#[test]
fn a_blurred_retake_is_still_the_same_page() {
    let page = handwritten();
    let again = out_of_focus(&retake(page, 1.0, 1.01, (10.0, 5.0), 0.9, 5.0), 1.5);
    let v = verdict(page, &again);
    assert!(v.same, "{v:?}");
}

#[test]
fn a_register_page_photographed_again_is_the_same_page() {
    let page = register_page(1, 20);
    let again = retake(&page, -1.2, 1.02, (-15.0, 22.0), 0.9, 8.0);
    let v = verdict(&page, &again);
    assert!(v.same, "{v:?}");
}

#[test]
fn two_pages_of_one_register_are_not_the_same_page() {
    for (a, b, filled_b) in [(1, 2, 20), (3, 4, 25), (5, 6, 8), (7, 8, 3)] {
        let first = register_page(a, 20);
        let second = register_page(b, filled_b);
        let v = verdict(&first, &second);
        assert!(!v.same, "seeds {a} and {b}, {filled_b} rows: {v:?}");
        // Even photographed a little differently.
        let v = verdict(&first, &retake(&second, 1.0, 1.01, (8.0, -6.0), 0.95, 0.0));
        assert!(!v.same, "seeds {a} and {b} retaken: {v:?}");
    }
}

#[test]
fn two_issues_of_one_bulletin_are_not_the_same_page() {
    let page = printed();
    let v = verdict(page, &other_issue(page));
    assert!(!v.same, "{v:?}");
}

#[test]
fn different_documents_are_not_even_candidates() {
    let hand = measure(handwritten());
    let print = measure(printed());
    let register = measure(&register_page(1, 20));
    assert!(tiny_likeness(&hand, &print) < FAR_LIKENESS);
    assert!(!compare(&hand, &print).same);
    assert!(!compare(&hand, &register).same);
}

#[test]
fn blur_lowers_sharpness_whatever_its_kind() {
    for page in [handwritten(), printed()] {
        let sharp = sharpness_of(page);
        let slightly = sharpness_of(&out_of_focus(page, 1.0));
        let blurred = sharpness_of(&out_of_focus(page, 2.0));
        let shaken = sharpness_of(&moved(page, 9));
        assert!(
            sharp > slightly && slightly > blurred,
            "{sharp} {slightly} {blurred}"
        );
        assert!(sharp >= BLUR_CEILING, "a sharp page: {sharp}");
        assert!(blurred < BLUR_FLOOR, "out of focus: {blurred}");
        assert!(shaken < BLUR_FLOOR, "moved: {shaken}");
    }
}

#[test]
fn sharpness_does_not_change_with_light_or_jpeg() {
    let page = handwritten();
    let sharp = sharpness_of(page);
    let dim = sharpness_of(&retake(page, 0.0, 1.0, (0.0, 0.0), 0.6, 40.0));
    assert!((sharp - dim).abs() < 0.15 * sharp, "{sharp} {dim}");
}

/// A page with a single short line written on it, the rest blank paper.
fn almost_empty() -> Grey {
    let hand = handwritten();
    let (w, h) = (hand.w, hand.h);
    let mut px = vec![215.0f32; w * h];
    for y in 0..40 {
        for x in 0..w / 3 {
            px[(h / 3 + y) * w + w / 4 + x] = hand.at(w / 4 + x, 400 + y).max(120.0);
        }
    }
    recompressed(&Grey::new(w, h, px), 85)
}

#[test]
fn a_blank_or_almost_empty_page_is_never_blurred() {
    let blank = recompressed(&Grey::new(800, 1100, vec![210.0; 800 * 1100]), 85);
    assert_eq!(measure(&blank).sharpness, None);
    let nearly = almost_empty();
    let (pairs, blurred) = judge(
        &batch(&[&nearly, handwritten(), printed()]),
        2,
        &StopSignal::default(),
    )
    .unwrap();
    assert!(pairs.is_empty());
    assert!(!blurred.contains(&0), "{:?}", measure(&nearly).sharpness);
}

fn batch(pages: &[&Grey]) -> Vec<Option<Measured>> {
    pages.iter().map(|g| Some(measure(g))).collect()
}

#[test]
fn a_batch_finds_the_retakes_and_suggests_the_sharper() {
    let hand = handwritten();
    let print = printed();
    let register = register_page(11, 22);
    let register_next = register_page(12, 22);
    let retaken_blurred = out_of_focus(&retake(hand, 1.2, 1.02, (12.0, -8.0), 0.9, 6.0), 2.0);
    let shaken = moved(print, 9);
    let pages = [
        &register,
        hand,
        &retaken_blurred,
        &register_next,
        print,
        &shaken,
    ];
    let (pairs, blurred) = judge(&batch(&pages), 2, &StopSignal::default()).unwrap();
    // The two pages of the register are not taken for one; the handwritten page and its
    // blurred retake are, and so are the bulletin and its shaken retake, the sharp one kept.
    assert_eq!(
        pairs,
        vec![
            SamePair {
                first: 1,
                second: 2,
                keep: 1,
                clearly_sharper: true,
            },
            SamePair {
                first: 4,
                second: 5,
                keep: 4,
                clearly_sharper: true,
            }
        ]
    );
    // The blurred retakes are warned about in their pairs, not again on their own.
    assert!(blurred.is_empty(), "{blurred:?}");
}

#[test]
fn a_page_blurred_beside_sharp_ones_is_flagged_by_the_batch() {
    let pages: Vec<Grey> = (0..5).map(|s| register_page(20 + s, 22)).collect();
    let soft = out_of_focus(&register_page(30, 22), 1.2);
    let mut all: Vec<&Grey> = pages.iter().collect();
    all.push(&soft);
    let measured = batch(&all);
    let (_, blurred) = judge(&measured, 2, &StopSignal::default()).unwrap();
    assert_eq!(
        blurred,
        vec![5],
        "{:?}",
        measured
            .iter()
            .map(|m| m.as_ref().and_then(|m| m.sharpness))
            .collect::<Vec<_>>()
    );
}

fn write_jpeg(dir: &Path, name: &str, g: &Grey) -> SourceFile {
    let path = dir.join(name);
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90)
        .encode_image(&to_image(g))
        .unwrap();
    std::fs::write(&path, out).unwrap();
    crate::inspect(&path, None).unwrap()
}

#[test]
fn photographs_on_disk_are_checked_in_their_order_and_shown() {
    let dir = tempfile::tempdir().unwrap();
    let print = printed();
    let sources = vec![
        write_jpeg(dir.path(), "IMG_0142.jpg", print),
        write_jpeg(
            dir.path(),
            "IMG_0143.jpg",
            &out_of_focus(&retake(print, -1.0, 1.02, (10.0, 6.0), 0.9, 8.0), 2.0),
        ),
        write_jpeg(dir.path(), "IMG_0144.jpg", handwritten()),
    ];
    let originals: Vec<Vec<u8>> = sources
        .iter()
        .map(|s| std::fs::read(&s.path).unwrap())
        .collect();
    let report = check_photos(&sources, &StopSignal::default()).unwrap();
    assert_eq!(
        report.warnings,
        vec![InputWarning {
            source_id: sources[1].id,
            kind: WarningKind::SamePage {
                other: sources[0].id,
                keep: sources[0].id,
                clearly_sharper: true,
            },
        }]
    );
    let shown: Vec<SourceId> = report.views.iter().map(|v| v.source_id).collect();
    assert_eq!(shown, vec![sources[0].id, sources[1].id]);
    for view in &report.views {
        let page = image::load_from_memory(&view.page).unwrap();
        assert!(page.width() <= VIEW_WIDTH);
        let detail = image::load_from_memory(&view.detail).unwrap();
        assert!(detail.width() <= VIEW_WIDTH && detail.height() < detail.width());
    }
    // Originals are only read.
    for (source, bytes) in sources.iter().zip(originals) {
        assert_eq!(std::fs::read(&source.path).unwrap(), bytes);
    }
}

#[test]
fn a_raised_stop_ends_the_check() {
    let stop = StopSignal::default();
    stop.stop();
    let pages = batch(&[handwritten(), printed()]);
    assert!(matches!(judge(&pages, 2, &stop), Err(MediaError::Stopped)));
}

/// Prints what the checks measure on every case, to set the thresholds:
/// `cargo test -p kalamu-media --release photos::tests::metrics -- --ignored --nocapture`.
#[test]
#[ignore = "prints measurements; run by hand"]
fn metrics() {
    let hand = handwritten();
    let print = printed();
    let show = |label: &str, a: &Grey, b: &Grey| {
        let (ma, mb) = (measure(a), measure(b));
        println!(
            "{label:40} tiny {:.2} hash {:2} {:?}",
            tiny_likeness(&ma, &mb),
            (ma.hash ^ mb.hash).count_ones(),
            compare(&ma, &mb)
        );
    };
    for (name, page) in [("hand", hand), ("print", print)] {
        show(&format!("{name} itself"), page, page);
        show(
            &format!("{name} retake 1.5"),
            page,
            &retake(page, 1.5, 1.02, (18.0, -12.0), 0.85, 10.0),
        );
        show(
            &format!("{name} retake -2"),
            page,
            &retake(page, -2.0, 0.97, (-25.0, 20.0), 1.15, -15.0),
        );
        show(
            &format!("{name} retake 3 / 1.05"),
            page,
            &retake(page, 3.0, 1.05, (40.0, 30.0), 0.7, 30.0),
        );
        show(
            &format!("{name} retake blurred"),
            page,
            &out_of_focus(&retake(page, 1.0, 1.01, (10.0, 5.0), 0.9, 5.0), 2.0),
        );
    }
    show(
        "print retake -1 blurred 2",
        print,
        &out_of_focus(&retake(print, -1.0, 1.02, (10.0, 6.0), 0.9, 8.0), 2.0),
    );
    show(
        "hand retake blurred 3",
        hand,
        &out_of_focus(&retake(hand, -1.0, 1.02, (10.0, 6.0), 0.9, 8.0), 3.0),
    );
    show("bulletin other issue", print, &other_issue(print));
    for (a, b, f) in [(1, 2, 20), (3, 4, 25), (5, 6, 8), (7, 8, 3), (9, 10, 22)] {
        show(
            &format!("register {a}/{b} ({f} rows)"),
            &register_page(a, 20),
            &register_page(b, f),
        );
    }
    let reg = register_page(1, 20);
    show(
        "register retake",
        &reg,
        &retake(&reg, -1.2, 1.02, (-15.0, 22.0), 0.9, 8.0),
    );
    show("hand / print", hand, print);
    show("hand / register", hand, &reg);
    for (name, page) in [("hand", hand), ("print", print), ("register", &reg)] {
        println!(
            "{name:10} sharp {:?} s1.0 {:?} s1.5 {:?} s2 {:?} s3 {:?} move5 {:?} move9 {:?} dim {:?}",
            measure(page).sharpness,
            measure(&out_of_focus(page, 1.0)).sharpness,
            measure(&out_of_focus(page, 1.5)).sharpness,
            measure(&out_of_focus(page, 2.0)).sharpness,
            measure(&out_of_focus(page, 3.0)).sharpness,
            measure(&moved(page, 5)).sharpness,
            measure(&moved(page, 9)).sharpness,
            measure(&retake(page, 0.0, 1.0, (0.0, 0.0), 0.6, 40.0)).sharpness,
        );
    }
    for (name, page) in [("hand", hand), ("print", print)] {
        for sigma in [0.5, 0.7, 1.0, 3.0, 5.0] {
            let b = out_of_focus(page, sigma);
            println!(
                "{name} sigma {sigma}: sharp {:?} content {:.1}",
                measure(&b).sharpness,
                super::measure::content(&b)
            );
        }
        println!("{name} content {:.1}", super::measure::content(page));
    }
    let nearly = almost_empty();
    println!(
        "almost empty {:?} content {:.1}",
        measure(&nearly).sharpness,
        super::measure::content(&nearly)
    );
    let blank = recompressed(&Grey::new(800, 1100, vec![210.0; 800 * 1100]), 85);
    println!("blank content {:.1}", super::measure::content(&blank));
}

/// How long the check takes on 300 photographs of 12 megapixels, written as JPEGs to a
/// temporary folder (about 500 MB, removed after):
/// `cargo test -p kalamu-media --release photos::tests::three_hundred -- --ignored --nocapture`.
#[test]
#[ignore = "writes 300 large photographs; run by hand"]
fn three_hundred_photographs() {
    let dir = tempfile::tempdir().unwrap();
    let base = image::load_from_memory(&sample("dodington-1755.jpg"))
        .unwrap()
        .into_rgb8();
    let (w, h) = base.dimensions();
    let mut sources = Vec::new();
    for n in 0..300u32 {
        // Each photograph a different part of the page, so few look alike; every tenth is
        // taken twice.
        let k = if n % 10 == 9 { n - 1 } else { n };
        let x = (k * 397) % (w / 2);
        let y = (k * 613) % (h / 2);
        let part = image::imageops::crop_imm(&base, x, y, w / 2, h / 2).to_image();
        let photo =
            image::imageops::resize(&part, 3000, 4000, image::imageops::FilterType::Triangle);
        let path = dir.path().join(format!("IMG_{n:04}.jpg"));
        let mut out = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 85)
            .encode_image(&photo)
            .unwrap();
        std::fs::write(&path, out).unwrap();
        let mut source = crate::inspect(&path, None).unwrap();
        source.page_count = Some(1);
        sources.push(source);
    }
    let stop = StopSignal::default();
    let start = std::time::Instant::now();
    let report = check_photos(&sources, &stop).unwrap();
    println!(
        "300 photographs of 12 MP: {:.1} s on {} threads, {} warnings, {} views",
        start.elapsed().as_secs_f32(),
        workers(),
        report.warnings.len(),
        report.views.len()
    );
    let index = |id: SourceId| sources.iter().position(|s| s.id == id).unwrap();
    for w in &report.warnings {
        match w.kind {
            WarningKind::SamePage { other, .. } => {
                println!("same page: {} and {}", index(other), index(w.source_id));
            }
            _ => println!("{:?}: {}", w.kind, index(w.source_id)),
        }
    }
    // The steps apart: reading every photograph, then laying the neighbours over each other.
    let start = std::time::Instant::now();
    let measured = parallel(&sources, workers(), &stop, |s| {
        measure::measure_file(Path::new(&s.path)).ok()
    })
    .unwrap();
    let read = start.elapsed();
    let start = std::time::Instant::now();
    let _ = judge(&measured, workers(), &stop).unwrap();
    println!(
        "reading {:.1} s, comparing {:.1} s",
        read.as_secs_f32(),
        start.elapsed().as_secs_f32()
    );
}
