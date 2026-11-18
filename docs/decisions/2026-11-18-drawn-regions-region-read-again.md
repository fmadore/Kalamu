# 2026-11-18 · The researcher's own regions over a scan, and a region read again

## Context

The first step of source-linked review ([#84](https://github.com/fmadore/Kalamu/issues/84),
[15 November](2026-11-15-records-regions-batch-files.md)) drew over the scan what the engines
had stored. The roadmap's next item was its second step: region identities, regions the
researcher draws or adjusts, a region read again by a model, and regions in the other reviews.

The maintainer set the rule for the round: new features for complex work are welcome, but the
app must stay friendly to people with little practice of computers. The specification already
says every screen stays plain, the advanced choices folded away; here it was a condition of
each brief and of the review. Four agents wrote the pieces, each in a worktree: region
identities and the researcher's layer, the cut of a region in `media`, the reading through the
engine with its dialog, and the drawing interface. They were reviewed and gathered on one
branch, the dialog connected to the region's card there, and every check run again.

## Decision

### Plain first

- **Nothing new shows until the regions are shown.** A test opens the four reviews and checks
  it. The analysis, table-of-contents and records reviews gain one switch on a scan, *Show
  regions*; a text file or a recording gains nothing.
- **One named button for each thing**, in words: *Draw a region*, *Move or resize*, *Remove*,
  *Put back the model's box*, *Read this region again…*. No visible string says coordinates,
  crop, layer, origin or granularity; French says *zone* and *cadre*.
- **A mode ends by itself** after one box, and Escape always leaves it. A slip of the hand (a
  click, a box too small to mean anything) keeps nothing and raises no error.
- **Everything is taken back**: a removal and a reading used, from the message that follows;
  a move, with Escape.
- **The keyboard draws too.** *Draw a region* pressed a second time places a box in the
  middle of the view, since a screen reader in browse mode sends a click, not a key.
- **The reading's dialog has one path**: the part that will be read, the model, what is sent,
  what it costs, *Read*. Another model is behind one button.

### Identity, granularity and origin

- **A model's region is named within its result**: `r3`, `r3_l2`, `r3_l2_w5`, from 1, by one
  function in `core` that PAGE XML calls too. `PageRegions.result` names the stored result
  (its cache key) those ids belong to.
- **They differ from PAGE XML's on a split spread**, where the review shows a half alone and
  numbers it alone, and PAGE XML numbers the whole page. The export was left as it is and the
  difference written down.
- **Origin** is the model's, estimated (Kalamu's lines and words within a block placed whole),
  or the researcher's; the list is open, for imported and aligned positions in the third step.

### The researcher's layer

- **A table of its own** (`drawn_regions`), never the stored result. A region is kept by its
  page's number and half, not by a piece, so the analysis and contents reviews, which have no
  piece for a page, show the same regions; with the source file's fingerprint as its version.
- **A box given to a model's region** names the result and the region it adjusts. The region
  stays in its place in the list, with its text and its place in the text, under the
  researcher's box. One box per region: saving again updates it.
- **When the page is read again**, the adjusted region no longer exists: the box is then
  shown as one the researcher drew. While another set-up still shows that result (comparison),
  the box is that set-up's alone and is not laid over the other's regions.
- **Kept with the job**, deleted with it. A region drawn in an extraction's job is not shown
  in an analysis's job of the same file: left for later, with documents.

### A region read again

- **Cut by `media`** from the original: a PDF page's boxes narrowed to the region, as a
  spread's half is, and drawn at 300 dpi, so a small box is as sharp as the page allows
  without drawing the whole page; an image cut at its own pixels after its EXIF turn. Tested
  at each rotation and with a CropBox smaller than the MediaBox.
- **Through the engine**, as a passage corrected by a model is: recorded before and after,
  stoppable, cached under a key over the picture's fingerprint and the set-up, costed, its
  cost in the job's. A text extraction of a picture, by any model that reads pictures, a
  local one included.
- **Where a reading goes is decided before it is asked**: the text of a region the researcher
  drew; the region's words in the page's text, as a correction, where their place is certain
  and still holds; nowhere, copied only, where it is not. The page's result never changes.
- **The estimate is one page's**, which errs high for a small region: no finer guess was
  invented.
- **An empty answer is not offered**, though kept, since it was paid for.

### Found on the way

- The page's box in the review shared a class name with a global style and took its padding,
  so every region was drawn against a box wider than the page. Fixed; the first step, shipped
  in 1.8.0, drew its regions slightly off.
- An image source asked for a page it does not have now refuses, where it returned page 1.

## Consequences

- `core` gains `regions::drawn` and `region_reading`; `store` two tables; `engine`
  `regions/drawn.rs` and `region_reading`; `media` `region.rs`; ten commands. No new
  dependency, no new crate, and dependencies still point inwards.
- `provenance.json` lists the researcher's regions and the readings used, without their text
  ([formats](../formats.md)). ALTO, PAGE XML and the searchable PDF keep the model's own
  positions: the researcher's boxes there are left to do.
- The privacy policy says what a reading sends: the part of the page, as a picture.
- Left of #84: the exports above; an analysis's answer, a contents entry or a records row tied
  to the regions it was read from; regions kept with a document across jobs; an estimate by
  the region's size; the third step.
- Not tried: a touch screen, Narrator and VoiceOver, forced colours, a real Mac, a live
  reading with any provider. A window of 640 by 560 leaves little of the scan free while a
  box is drawn with the pointer; the keyboard's way works there.
