# 2026-11-20 · Review from the seat of a researcher new to computing: the order of what comes next

## Context

Between 1 and 19 November the code moved faster than two things around it: the issues, most
of which still read as they did in October while their first increments had shipped, and the
trying by hand, since nearly all of what 1.9.0 adds has run against recorded answers only and
nothing written since 1.6.0 has run on a Mac. A review of the repository, the roadmap and the
issues was made on 20 November from the seat of who Kalamu is for: a historian, sociologist
or anthropologist who is not at ease with computers.

It found the app true to its rules (the model's text never overwritten, no quotation that is
not in the transcript, no merge made by itself, no chat) and found five places where such a
researcher stops, each checked in the code:

1. The first launch opens *Settings › API keys* with no welcome, and a researcher who works
   only offline is sent there at every launch (`+page.svelte` tests saved keys alone). The
   sidebar's line looks at Whisper's models only and names Whisper whatever is installed.
2. A `.csv` opened by double-click in Excel set to French or German lands in one column.
3. The default prompts' names are translated; their fields and choices are not.
4. Nothing in the app opens the guide, which exists in English only.
5. An empty project shows six sections at once.

## Decision

- **Kalamu is not short of features; it is ahead of what has been tried.** No new feature
  goes into 1.9.0. Its test pass on Windows and its day on a Mac come first.
- **Then a plain-first round**, for 1.9.1 or 1.10.0:
  [#208](https://github.com/fmadore/Kalamu/issues/208) (first launch, a start without a key),
  [#209](https://github.com/fmadore/Kalamu/issues/209) (tables as `.xlsx`, `corrected` per
  row in records), [#210](https://github.com/fmadore/Kalamu/issues/210) (default fields in
  the interface language), [#211](https://github.com/fmadore/Kalamu/issues/211) (the guide
  from the app, and in French), [#212](https://github.com/fmadore/Kalamu/issues/212) (an
  empty project's orientation).
- **The two questions left "to decide" on 17 November are decided.** Tables get an Excel
  workbook beside the CSV, which stays as it is for other tools, rather than a CSV whose
  separator follows the system: a workbook opens the same in every language of Excel and
  carries no formula reading. `corrected` in a records export is per row; `reviewed` stays per
  page.
- **Among the research workflows, four come first**: the second slice of interview analysis
  (#38), research-question screening (#51, on the citing field #38 built), photographs from
  the archive (#53, #52, [#216](https://github.com/fmadore/Kalamu/issues/216)), and entries
  across pages with their neighbours' context (#171, #54).
- **New directions, each designed in a decision before any code**: a project file
  ([#213](https://github.com/fmadore/Kalamu/issues/213)), assisted pseudonymisation
  ([#214](https://github.com/fmadore/Kalamu/issues/214)), a sample first
  ([#215](https://github.com/fmadore/Kalamu/issues/215)), references from Zotero
  ([#217](https://github.com/fmadore/Kalamu/issues/217)).
- **Last in line**: assisted TEI encoding (#37) and Linux packages (#87), which serve few of
  the researchers Kalamu is for and cost much to keep.

## Consequences

- The open issues with a shipped first increment (#38, #47, #48, #49, #50, #52, #84, #85,
  #167) each carry a comment of 20 November saying what is done and what is left, so that an
  open issue reads as work still to do. #148 and #144 stay open until the test pass.
- The roadmap's *Next* opens with the test pass, then the plain-first round, and *What is
  left of the research workflows* opens with the order above.
- The plain-first round changes no stored data and no request to a provider, so the privacy
  policy is unchanged by it. The project file and pseudonymisation will change it.
