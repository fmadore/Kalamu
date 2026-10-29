# 2026-10-29 · One frame for the three reviews

## Context

The [checkpoint of the same day](2026-10-29-checkpoint.md) found the review's frame copied in
`ReviewView` (1,508 lines), `AnalysisReview` (778) and `ContentsReview` (682), each with 150 to
200 lines of the same styles, and drifting apart: only the text review had Ctrl Enter, the table
of contents stacked its panes at 900 pixels instead of 1000 and had no handle between them, and
the analysis review kept its split under a key of its own. The text review showed a text file's
correction as a scan whose preview never came, and named page halves by the engine's raw `left`
and `right`. Search in the review, the next feature, would have had to be built three times.

## Decisions

- **`ReviewFrame`** holds what the reviews share, taking each review's content as snippets: the
  masthead (back to the job, the file, the menu of pages, the actions and the reviewed button
  with its shortcut), the source, the text or answers with their bar, a line or a panel under
  them, a third pane while a compared text is open, the handles on the boundaries, the strip
  of pages, and the keys. Ctrl Enter marks the piece reviewed and Page Up, Page Down, or Alt and
  the arrows turn the pages, in all three reviews; the table of contents' plain arrows give way
  to these, which do not fight the text fields. All three stack at 1000 pixels.
- **`SourcePane`** shows the source the same way everywhere: a scan on its lightbox with zoom
  and, where no strip runs below, buttons to turn its pages; a text file set as text, with the
  words pointed at found in it; or a player. A text file's correction now shows the passage
  being corrected.
- **Shared pieces**: `SiglaKey` (the set-ups with their letters, under a comparison),
  `CorrectionEditor` (whose text is being corrected, and saving, discarding or going back to
  the model's output; Ctrl Enter saves, which marks the piece reviewed as the button does) and
  `FieldInput` (one answer in the control its kind needs, for the analysis and the table of
  contents alike, so an entry's field with several choices gets boxes to tick rather than a
  text box). `MarkedText` (the reading text and the compared text, with their marks) and
  `ReadingsPopover` (which places itself under or over its words) leave `ReviewView`.
- **Logic out of the components**, tested: `review.svelte.ts` loads the job and the source's
  pieces, saves a correction and reads them again, shows a scan's page (only the last asked
  for), and keeps the panes' boundaries, the same for the three reviews, under the text
  review's key; `pieces.ts` names the pieces, page halves through the translation file ("Page
  3, left half", "3l"); `fields.ts` turns answers into an editor's draft and back, keeping the
  model's reasons (once copied in the analysis review and in `contents.ts`); `stored.ts` keeps
  a convenience on this computer, the sidebar's fold included.
- The pane and its bar are global classes (`.pane`, `.pane-bar`), like the masthead, so the
  frame and the source pane draw them alike.

## Consequences

- `ReviewView` is 690 lines, `AnalysisReview` 267 and `ContentsReview` 367; the frame is 334
  and its source pane 215. Search in the review is built once, in the frame.
- The analysis review gains Ctrl Enter, "Fit width" and the turning buttons' page count; the
  table of contents gains the handle between its panes. A split made in the analysis review
  before is forgotten once, as it now shares the text review's.
- Checked in the browser preview with the sample data: the six reviews (comparison with its
  compared text and readings open, correction, recording, YouTube video, analysis, table of
  contents), their editors, Ctrl Enter saving from an editor, the page keys and F, wide,
  stacked and narrow, light and dark. In a window both narrow and short, with the compared text
  open, the stacked panes leave the texts only their bars; this was so before and is left for
  the layout work on narrow windows.
- Still to do from the checkpoint's refactor: New job, the job page and the sidebar in smaller
  components; `run.rs` and `batch.rs` split; the rules on which files and models suit a task
  given to the front end by the engine.
