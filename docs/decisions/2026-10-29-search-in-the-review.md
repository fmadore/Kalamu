# 2026-10-29 · Search in the review

## Context

A researcher checking a long issue or a two-hour interview needs to find a name or a phrase:
every place a person is mentioned, the page where a word was misread. The review showed one page
at a time and had no way to look through the rest. The reviews' [shared
frame](2026-10-29-review-frame.md) made it possible to build the search once.

## Decisions

- **The text searched is the text as the review shows it**: for each piece, the leading
  set-up's text with the researcher's corrections. The other set-ups' readings are not
  searched; they are one click away in the compared text. A table of contents searches its
  entries, every field, and keeps those that hold the words.
- **Matching forgives what a reader would**: capitals, accents and Arabic vowel marks are
  ignored, and any run of spaces or line breaks matches one space, so "mission catholique"
  finds the words across a line break. Two letters at least. Matches are found in the text's own
  offsets (`search.ts`), so they are marked exactly where they stand.
- **Everything runs in the window**, on the pieces the review has already loaded: nothing is
  sent and nothing is kept, so the privacy policy does not change.
- **Matches are marked with the browser's highlights** (`::highlight(found)`, `found.ts`), not
  by splitting the text into more elements: the reading text's marks for differences and
  corrections stay as they are, and the two can overlap. WebView2 supports them; elsewhere the
  search still counts, lists and goes to matches, without the marks.
- **Found words are grey, the match gone to is ink.** Yellow already marks where the models
  disagree and blue is the researcher's hand; a third meaning for either would blur both.
- **A recording's matches carry the time of their line** when the timed lines hold as many
  matches as the text: then the n-th match is in the line holding the n-th match. Where a
  correction changed the words, the time is left out and the match is listed under its piece.
  Going to a match plays the recording or the YouTube video from its line.

## Consequences

- Ctrl F opens the search in the text review and the table of contents' review; the analysis
  review, one document beside its answers, has none yet.
- The list shows the first 300 matches, with a note when there are more; all are counted and
  stepped through.
- Tested: matching (capitals, accents, Arabic vowel marks, spaces and line breaks, separate
  accents, overlaps, the two-letter minimum), the words around a match, numbering the matches
  of each piece, tying a recording's matches to their lines or leaving the time out, and which
  entries of a table of contents hold the words. Checked in the browser preview: matches across
  pages and pieces, going to one on another page, a recording playing from its line, the
  entries filtered, Escape, light and dark, wide and narrow.
