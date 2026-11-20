# 2026-11-20 · Duplicate and blurred photographs flagged before a job is sent

## Context

In a reading room a researcher often photographs the same page two or three times: the hand
moved, the focus hesitated. A folder of 300 phone photographs dropped into New job was then
paid for and reviewed with its retakes, and sometimes the blurred take was the one read
([#216](https://github.com/fmadore/Kalamu/issues/216)). The input checks in `media` (spreads,
resolution, a text layer, a recording's level) look at one file at a time; this one compares the
photographs of a batch with each other. None of the reference projects compares inputs, so there
was nothing to port.

The hard case is not the retake but the page that looks like it: two pages of one register, of
one printed form or of one newspaper share their ruling and layout, and differ only in what is
written on them. A 64-bit average or difference hash calls them the same page (on the synthetic
register below, the difference hashes of two different pages differ in 8 to 17 bits of 64, those
of a page and its retake in 13 to 25), so a hash can only choose what to look at more closely.

## Decisions

- **One pass, in `media`, beside the other checks** (`crates/media/src/photos.rs`,
  `photos/{measure,compare,grey}.rs`), behind `Engine::check_photos` and two commands,
  `check_photos` and `stop_photo_checks`. Two kinds join `WarningKind` in `core`:
  `SamePage { other, keep, clearly_sharper }`, carried by the later photograph of a pair, and
  `Blurred`. Each warned photograph comes back drawn small and close up (`PhotoView`, two JPEGs
  at most 400 pixels wide), so the researcher judges by eye. No new crate: the image decoding
  `media` already has, and some 900 lines (comments included) of grey-image arithmetic (area reduction, a warp by a
  similarity, box filters, correlations).
- **Each photograph decoded once, reduced, and let go** (`measure`). Decoded upright (its EXIF
  orientation applied), turned grey, reduced by area averaging to 1,536 pixels on its long side,
  about what a model is given of a page (Claude scales larger images to 1,568), so blur too fine
  to survive that reduction, which a model would not see either, is not reported. From that copy
  are kept: a 384 × 288 grey copy (110 KB) for the comparisons, a 32 × 24 copy and a difference
  hash for the far pairs, its sharpness and where its writing is densest. 300 photographs hold
  about 33 MB; no decoded photograph outlives its measurement.
- **Neighbours first.** Every photograph is compared with the two that follow it in the order
  read (the files', or the document's pages' when the images are one document), which is where
  retakes sit; a pair farther apart only when their difference hashes differ in at most 8 bits
  and their 32 × 24 copies correlate at 0.75 or more. A retake turned 3 degrees can correlate as
  little as 0.29 with its page at 32 × 24, so neighbours are always compared in full.
- **Same page: the second laid over the first, then compared tile by tile** (`compare`).
  - *Laying over*: on a pyramid of the 384 × 288 copy, turns of −3 to 3 degrees by 1 and scales
    of 0.95, 1 and 1.05 with shifts of up to a tenth of the page are tried at an eighth of the
    size; the three best are narrowed by half a degree and 0.025 of scale at a quarter, then by a
    quarter of a degree and 0.01 at a half, each with its best shift (correlation of the grey
    copies brought to a mean of 0 and a spread of 1).
  - *Tiles*: on both copies, the detail of the writing is kept (each pixel minus the average of
    the 5 × 5 square around it, which takes away shading and exposure), then every ruled line is
    taken away (each pixel minus the average of the 25 pixels along its row, then along its
    column: a line, the same along its length, goes; letters stay). Without that step, two
    different pages of the register below correlated at a median of 0.75 to 0.92 and were taken
    for one; with it, 0.09 to 0.18. The page is cut into 8 × 6 tiles. A tile holds writing when
    its detail is at least 0.6 of the page's; it disagrees when one photograph has writing there
    and the other under 0.35 of it (except at the edge, where a retake may simply show less of
    the page), or when the two correlate under 0.5 at their best shift of up to 2 pixels.
  - *Verdict*: the same page when at least 6 tiles hold writing, at most 25% of them disagree,
    and their median correlation is 0.45 or more. A blank page is never judged against another.
  - *A blurred retake*: when one of the two is soft (sharpness under 0.75) and the other sharper
    by 0.1 or more, both copies are softened by a 3 × 3 average before their detail is taken, so
    their writing compares rather than their sharpness.
- **Sharpness from two scales** (`measure::sharpness`). A sharp stroke steps from ink to paper
  within a pixel at the working size and at half of it alike; a blurred stroke spreads its step
  over several pixels, which halving the size halves, so its steps are twice as steep at half
  the size. Sharpness is the ratio of the steepest steps (the strongest 0.5% of pixels) at the
  working size to those at half of it: towards 1 sharp, towards 0.5 blurred, whatever is written
  and in whatever light. Steps across and steps down are read apart and the lower ratio kept,
  since a hand that moved blurs only the edges across its movement (read together, a page shaken
  sideways measured 0.85 to 1.36, as sharp as a still one or sharper). A page whose strongest edges at an eighth of the size are
  fainter than 10 grey levels holds too little to judge, and is never called blurred.
- **Blurred: against the batch and against a floor.** A photograph is blurred when its
  sharpness is under 0.6 whatever the batch, or, in a batch of at least four that can be judged,
  under 0.78 of the batch's median while also under 0.85 (so a sharp batch does not make a sharp
  page look blurred, and a soft phone does not make every page blurred). Of two photographs of
  one page, the sharper is the one suggested, said to be clearly sharper when it is higher by 6%
  or more; the other is not warned about again as blurred.
- **Fast enough and stoppable.** The work runs on half the processors, at most six, never on the
  interface's thread. Each thread stops before its next photograph or pair once the check's
  `StopSignal` is raised: by a newer check, or by leaving New job (`stop_photo_checks`), and the
  stopped check gives `None`. New job checks again only when photographs are added; removing one,
  moving a page or answering a warning needs no new check.
- **Measured on this machine** (Windows on ARM, 12 logical processors, release build, other
  builds of two other checkouts running beside it): 300 JPEGs of 12 megapixels (3,000 × 4,000)
  took 38 s in one run and 52 s in another on six threads, about half reading the photographs
  and the rest comparing 600 pairs of neighbours and drawing the 72 photographs warned about
  (`photos::tests::three_hundred_photographs`, ignored by default). Unloaded, one photograph
  takes about 300 ms to read (220 ms of it decoding the JPEG), and one pair about 40 ms to
  compare. The check runs in the background while the form is filled in, says so in a line under
  the files, and never blocks Start; a minute for 300 photographs is slow, and keeping
  measurements between calls (below) and a reduced decoding are the ways to make it faster.
  In that run, the 30 retakes planted among the 300 (every tenth photograph taken twice) were
  all found, and the 6 other pairs found were crops of the page shifted by 1 to 3% of its size,
  which are the same page; nothing else was flagged. Decoding straight to grey with `zune-jpeg` (the decoder `image` itself uses) was
  tried and saved nothing measurable, so no crate was added. The decoder offers no reduced
  decoding (`image` 0.25 dropped JPEG's scaled DCT), which would be the next saving.
- **In New job** (`PhotoNotice`, `newJobPhotos.svelte.ts`): under the later photograph of a pair,
  in the notice treatment, "IMG_0142 and IMG_0143 look like the same page", the two side by side
  on the stage, whole and close up at the place where the writing is densest, the sharper one
  marked, and *Keep the sharper one* (or *Keep only IMG_0142* when they are about as sharp) and
  *Keep both*; a blurred photograph alone, *Keep it* and *Leave it out*. Leaving a photograph out
  takes it from this job's list only, never from the disk, and from the order of the pages when
  the researcher set it by hand (otherwise the engine orders what is left); the toast's Undo puts
  it back where it was. Nothing is left out without the researcher. A check that fails says
  nothing, and the photographs are sent as they are.

## What the thresholds were set on

**Synthetic photographs only.** No real retakes are in the repository, so every case is made at
test time (`photos/tests.rs`) from the two sample pages (`samples/dodington-1755.jpg`,
handwritten, 2,922 × 4,298; `samples/bulletin-page.jpg`, printed, 992 × 1,403), nothing large
committed:

- *Retakes*: the page turned by −2 to 3 degrees, 0.97 to 1.05 times nearer, shifted by up to
  40 pixels at the working size, 0.7 to 1.15 times as bright, the table showing at the edges,
  saved again as JPEG at quality 82.
- *Blur*: Gaussian blur of 0.5 to 5 pixels at the working size, and a hand moved over 5 and 9
  pixels.
- *Different pages, same form*: a register drawn at test time (a printed heading taken from the
  bulletin, ruled rows and four columns on shaded paper), its rows filled with lines of
  handwriting cut from the 1755 page at places a seed picks; two seeds are two pages of one
  register, 3 to 25 rows filled. And another issue of the bulletin: the same masthead and
  columns, each column's text moved down by a different number of lines.

What they measured:

| Case | Tiles disagreeing | Median correlation | Verdict |
| --- | --- | --- | --- |
| Retakes of the handwritten page (4 kinds, one blurred) | 0 | 0.90 to 0.93 | same |
| Retakes of the printed page (4 kinds, two blurred) | 0 to 3 of 21 to 29 (≤ 14%) | 0.60 to 0.88 | same |
| Retake of a register page | 0 of 23 | 0.80 | same |
| Two pages of one register (5 pairs) | 19 to 33 of 25 to 39 (≥ 76%) | 0.09 to 0.18 | different |
| Two issues of the bulletin | 25 of 31 | 0.08 | different |
| Handwritten page beside printed page or register | all | 0.04 to 0.05 | different |

| Page | Sharp | Blur 0.5 | 0.7 | 1 | 2 | 3 | Moved 5 | Moved 9 | Dim, JPEG |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Handwritten | 0.94 | 0.77 | 0.67 | 0.60 | 0.53 | 0.51 | 0.60 | 0.58 | 0.89 |
| Printed | 0.91 | 0.84 | 0.74 | 0.62 | 0.52 | 0.51 | 0.55 | 0.56 | 0.92 |
| Register | 0.98 | | | 0.60 | 0.51 | 0.52 | 0.57 | 0.53 | 0.97 |

## False positives and negatives seen

- **Seen while tuning, now fixed**: two pages of one register taken for one page (the ruling
  carried the correlation) until ruled lines were averaged away; a blurred retake of the printed
  page missed (11 to 12 tiles disagreeing) until both photographs were softened before comparing
  and the scale was narrowed at the finest step; a page shaken sideways measured as sharp until
  steps across and down were read apart.
- **Accepted**: a printed form whose pages differ only in a name and a date in one corner may
  be taken for one page (most tiles are the same print); *Keep both* answers it. A page with a
  single short line on it is not judged for blur (too little to judge), so a blurred one is
  missed. Retakes more than two places apart in the order are found only when their layouts
  already agree closely (difference hashes within 8 bits): a retake taken at the end of a session
  may be missed.
- **Not yet known**: how these thresholds hold on real reading-room photographs: perspective
  (a phone not square to the page), a curved binding, a hand or a weight on the page, a page
  photographed once landscape and once portrait (never compared), flash glare, very small or
  faint handwriting, and different phones in one batch. They need trying on real batches before
  they are trusted; the measurements above are synthetic.

## Consequences

- No warning is stored with the job: the check runs in New job, and what the researcher chose
  is the list of files sent. Nothing changes in the store, the cache or `provenance.json`; no
  migration.
- Adding photographs to a large batch checks the whole batch again (each photograph decoded
  again). Keeping each photograph's measurement in the engine between calls would make adding a
  few to 300 cheap; left until it is wanted.
- **Left for [#53](https://github.com/fmadore/Kalamu/issues/53)** (persistent image
  preparation): skewed or rotated pages, crop, dark or low-contrast photographs, and a
  perspective correction that would let the comparison tolerate a phone held at an angle. The
  sharpness measure and the laying over of two photographs here are the pieces such a check
  would start from.
