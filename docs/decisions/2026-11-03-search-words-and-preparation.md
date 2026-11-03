# 2026-11-03 · Project search: several words, the text prepared ahead, matches opened in place

## Context

The [first delivery](2026-11-03-project-search.md) of [#86](https://github.com/fmadore/Kalamu/issues/86)
found one literal phrase, prepared a project's text during its first search, and opened an
analysis or a table of contents with only its own search filled in. Its *Left for later* named
the next steps: several words anywhere in a passage, proximity, preparing the text in the
background with its progress shown, and opening a match at its entry or field.

None of the reference projects searches across results (see the first decision), so the design
is again Kalamu's own.

## Decisions

- **Three ways to read the words, chosen, not typed.** *The exact phrase* stays the default;
  *All these words* and *These words near each other* sit beside it in a menu, with the
  distance (10 words unless changed, 0 to 100) shown only for the last. No operators to learn:
  the one piece of syntax is quotes, straight, curly or French, which keep words together as one
  term in the last two modes. Alternatives (one word or another) are left for later.
- **A term is three letters at least.** The index matches three characters at a time, so a
  shorter word cannot be looked for; rather than refuse the query, such words (`la`, `de`) are
  left out and named under the count. A query left with nothing to look for is refused as
  before.
- **Within one part of a passage.** *All these words* asks for every term in the same page,
  recording piece or answer, the parts the index already keeps apart; a table of contents' entry
  field is its own part. The index finds the passages holding every term (FTS5 `AND` of
  phrases); the words are then placed in each part in Rust, as before.
- **What a match is.** In *All these words*, each place of each term, listed in reading order.
  In *Near*, each stretch where every term stands with at most the distance's words between the
  first and the last, the earliest such stretch to end and then the next after it, none
  overlapping; the words between are shown, only the terms are marked (`Hit.marks`, offsets in
  the words shown, where a run of white space is one space).
- **Opening a match needs no change in the review.** Each match carries a term (`Hit.term`) and
  which of that term's matches it is in its piece (`nth`), counted through the whole piece as the
  review's own search counts them, parts that hold not every term included. The review opens
  with that term in its search and goes to that match. For words near each other, the first of
  them is the one opened on.
- **The text prepared when Search opens.** `Search::prepare` brings the index up to date with
  every result tied to the project, read or generated, so the first search finds it ready.
  Bringing the index up to date is now planned for every job before any passage is made, so the
  total is known and progress (`search-progress`, pieces done of those that needed it) is sent
  after each batch, by the preparation and by a search alike. A search begun meanwhile stops the
  preparation between batches and carries on from where it got to; a newer preparation stops an
  older one. A stopped preparation is not an error.
- **An analysis opens at its field, a table of contents at its entry.** Passages now number a
  table of contents' entries (`entry_number`, from 1, as the review numbers them), which raised
  the passages' version, so every passage is made again at the next search. The review scrolls
  to the field or entry, marks it with a rule, and shows the entry's first page.

## Consequences

- The search's request gains `mode` and `near`, its view `terms` and `ignored`, each match
  `term`, `marks` and `entry_number`; the index's `find` takes several terms.
- The app gains a command, `prepare_project_search`, and an event, `search-progress`.
- The app sends nothing new and keeps nothing new; the privacy policy is unchanged.
- `search/src/terms.rs` holds reading a query and placing its terms; `query.rs` chooses the
  results and lists the matches. Both are under 500 lines.
