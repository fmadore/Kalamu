---
name: Kalamu
description: AI transcription, text extraction and analysis for researchers, checked against the source.
colors:
  chrome-grey: "#efece6"
  surface: "#fdfcfa"
  surface-quiet: "#f8f6f2"
  stage: "#1f1e1c"
  stage-raised: "#2b2a27"
  stage-line: "#383632"
  on-stage: "#e8e6e1"
  on-stage-muted: "#aeaaa2"
  line: "#e5e1d9"
  line-soft: "#eeebe5"
  line-strong: "#d6d1c7"
  line-hover: "#a8a297"
  control-line: "#8a8478"
  control-line-hover: "#5d5950"
  graphite-ink: "#171612"
  graphite-ink-hover: "#2e2c27"
  on-ink: "#ffffff"
  on-ink-muted: "#bdb9b0"
  on-ink-danger: "#ff8a7d"
  wash: "rgba(23, 22, 18, 0.05)"
  highlighter: "#ffd84d"
  on-highlighter: "#171612"
  hand-blue: "#2456d6"
  hand-wash: "rgba(36, 86, 214, 0.1)"
  ok-green: "#1f7a4d"
  ok-text: "#155c39"
  ok-wash: "#e3f2ea"
  warn-amber: "#d97f00"
  warn-text: "#7a4300"
  warn-wash: "#fff6e5"
  danger-red: "#b42318"
  danger-text: "#9a1c13"
  danger-wash: "#fdecea"
  danger-border: "#f2c9c4"
  speaker-1: "#0f8a7e"
  speaker-1-wash: "#dcf3ef"
  speaker-1-text: "#0b5c54"
  speaker-2: "#d97f00"
  speaker-2-wash: "#ffebcc"
  speaker-2-text: "#7a4300"
  text-bright: "#171612"
  text-body: "#1e1d19"
  text-secondary: "#4d4a42"
  text-muted: "#615d54"
  text-faint: "#938e84"
  dark-chrome: "#171614"
  dark-surface: "#1d1c1a"
  dark-surface-raised: "#272522"
  dark-stage: "#100f0e"
  dark-line: "#2f2d29"
  dark-control-line: "#7a756c"
  dark-control-line-hover: "#a09a8f"
  dark-on-ink-danger: "#b42318"
  dark-ink: "#eeece6"
  dark-highlighter: "#e8c547"
  dark-hand-blue: "#7fa2ff"
  dark-text-body: "#e4e0d8"
  dark-text-muted: "#a09a8f"
typography:
  title:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 600
    lineHeight: 1.25
    letterSpacing: "-0.01em"
  heading:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "1.125rem"
    fontWeight: 600
    lineHeight: 1.25
    letterSpacing: "-0.01em"
  body:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.45
  label:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "0.9375rem"
    fontWeight: 500
    lineHeight: 1.45
  caption:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 500
    lineHeight: 1.45
  reading:
    fontFamily: "'Source Serif 4 Variable', 'Times New Roman', serif"
    fontSize: "1.1875rem"
    fontWeight: 400
    lineHeight: 1.7
  reading-small:
    fontFamily: "'Source Serif 4 Variable', 'Times New Roman', serif"
    fontSize: "1.125rem"
    fontWeight: 400
    lineHeight: 1.6
  machine:
    fontFamily: "'IBM Plex Mono', 'Cascadia Mono', Consolas, monospace"
    fontSize: "0.92em"
    fontWeight: 400
  machine-mark:
    fontFamily: "'IBM Plex Mono', 'Cascadia Mono', Consolas, monospace"
    fontSize: "0.75rem"
    fontWeight: 400
    lineHeight: 1
  figure:
    fontFamily: "'IBM Plex Mono', 'Cascadia Mono', Consolas, monospace"
    fontSize: "1.875rem"
    fontWeight: 500
    letterSpacing: "-0.02em"
rounded:
  highlight: "2px"
  tag: "5px"
  control: "7px"
  card: "10px"
  overlay: "10px"
  pill: "999px"
spacing:
  "1": "0.25rem"
  "2": "0.5rem"
  "3": "0.75rem"
  "4": "1rem"
  "5": "1.5rem"
  "6": "2rem"
  "7": "3rem"
components:
  button:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-bright}"
    borderColor: "{colors.control-line}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "0 0.75rem"
    height: "2.25rem"
  button-primary:
    backgroundColor: "{colors.graphite-ink}"
    textColor: "{colors.on-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "0 0.75rem"
    height: "2.25rem"
  button-primary-hover:
    backgroundColor: "{colors.graphite-ink-hover}"
    textColor: "{colors.on-ink}"
  button-primary-lg:
    backgroundColor: "{colors.graphite-ink}"
    textColor: "{colors.on-ink}"
    rounded: "{rounded.control}"
    padding: "0 1.5rem"
    height: "2.75rem"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.text-bright}"
    rounded: "{rounded.control}"
    padding: "0 0.75rem"
    height: "2.25rem"
  button-ghost-hover:
    backgroundColor: "{colors.wash}"
  button-danger:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.danger-red}"
    rounded: "{rounded.control}"
    padding: "0 0.75rem"
    height: "2.25rem"
  button-sm:
    padding: "0 0.625rem"
    height: "1.875rem"
  input:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-bright}"
    borderColor: "{colors.control-line}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "0.25rem 0.75rem"
    height: "2.25rem"
  chip:
    backgroundColor: "{colors.chrome-grey}"
    textColor: "{colors.text-secondary}"
    typography: "{typography.caption}"
    rounded: "{rounded.tag}"
    padding: "0.125rem 0.5rem"
  chip-ok:
    backgroundColor: "{colors.ok-wash}"
    textColor: "{colors.ok-text}"
    rounded: "{rounded.tag}"
  chip-warn:
    backgroundColor: "{colors.warn-wash}"
    textColor: "{colors.warn-text}"
    rounded: "{rounded.tag}"
  chip-danger:
    backgroundColor: "{colors.danger-wash}"
    textColor: "{colors.danger-text}"
    rounded: "{rounded.tag}"
  chip-hand:
    backgroundColor: "{colors.hand-wash}"
    textColor: "{colors.hand-blue}"
    rounded: "{rounded.tag}"
  siglum:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-secondary}"
    typography: "{typography.machine-mark}"
    size: "1.25rem"
  sidebar:
    backgroundColor: "{colors.chrome-grey}"
    textColor: "{colors.text-body}"
    width: "18rem"
  sidebar-item-current:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-bright}"
    rounded: "{rounded.control}"
    padding: "0.5rem 0.625rem"
  masthead:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-bright}"
    typography: "{typography.title}"
    padding: "0.5rem 1.5rem"
    height: "3.5rem"
  lightbox:
    backgroundColor: "{colors.stage}"
    textColor: "{colors.on-stage}"
    padding: "1.5rem 1.5rem 4.5rem"
  zoom-pill:
    backgroundColor: "{colors.stage-raised}"
    textColor: "{colors.on-stage}"
    padding: "0.25rem"
  disagreement:
    backgroundColor: "{colors.highlighter}"
    textColor: "{colors.on-highlighter}"
    typography: "{typography.reading}"
    rounded: "{rounded.highlight}"
  readings-card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-bright}"
    typography: "{typography.label}"
    rounded: "{rounded.card}"
    padding: "0.75rem"
    width: "25rem"
  reading-text:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-body}"
    typography: "{typography.reading}"
    padding: "1.5rem 2rem 3rem 1rem"
    width: "42rem"
  page-strip-piece:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-secondary}"
    typography: "{typography.caption}"
    height: "2rem"
  player:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-bright}"
    rounded: "{rounded.card}"
    padding: "0.5rem 1rem 0.5rem 0.5rem"
  summary-aside:
    backgroundColor: "{colors.surface-quiet}"
    textColor: "{colors.text-body}"
    padding: "1.5rem"
    width: "21rem"
  toast:
    backgroundColor: "{colors.graphite-ink}"
    textColor: "{colors.on-ink}"
    rounded: "{rounded.control}"
    padding: "0.75rem 1rem"
---

# Design System: Kalamu

## Overview

**Creative North Star: "Warm graphite & highlighter"**

A researcher checking a transcript works the way they would with a printout: a grey pencil for structure, a yellow highlighter for what needs another look, a blue pen for their own corrections. The interface is that desk kit and nothing more. Paper-warm greys and hairlines do the structural work, black is kept for the one action that moves the work forward, yellow says "look here" and blue says "you changed this". Everything else stays quiet so those three marks can be read at a glance across a page of text. The greys lean warm, toward paper rather than a lab bench, and the highlighter is also the colour of the app's mark, so the one brand colour is the one that means "look here".

The source is the authority, so it gets the best seat: a scan lies on a dark lightbox in both themes, the brightest thing on screen, with its controls floating on the dark in a small pill. The text being checked sits beside it on white, in a reading serif. The interface around both is a plain sans; anything a machine produced (file names, model names, times, costs, counts, shortcuts) is set in a monospace, so the researcher can tell their material, the tool and the machine apart by the face alone. Density is that of a working tool used for hours on a laptop: compact controls, generous reading measure, no decoration.

The same world holds from a window snapped beside another to a large monitor, and in light and dark themes that follow Windows by default. In dark mode the ink flips to near white, so the primary action stays the highest-contrast object on screen.

**Key Characteristics:**
- Warm graphite neutrals: a paper-grey chrome sidebar (#efece6), a warm white work surface (#fdfcfa), hairline borders.
- Black (graphite ink) is the only button fill, used once per work area and paired with its shortcut.
- Highlighter yellow marks only where models disagree or where a name is found in the document.
- Hand blue marks only the researcher's own corrections, never AI output.
- Scans sit on a dark lightbox (#1f1e1c light, #100f0e dark) in both themes.
- Three faces with three jobs: Instrument Sans, Source Serif 4, IBM Plex Mono.
- Lucide line icons at an absolute 1.5px stroke, beside words.

## Colors

A warm graphite palette of paper greys and one ink, with two marker colours whose rarity carries the meaning and a small set of status hues used as text on a wash.

### Primary
- **Graphite Ink** (#171612; dark-ink #eeece6 in dark): primary buttons, the sidebar's New job button, the toast, progress fills, the play button and scrubber, the selected task's inset ring, focus outlines and the checked state of native controls. Hover lifts to **Graphite Ink Hover** (#2e2c27). Text on it is **On Ink** (#ffffff), with **On Ink Muted** (#bdb9b0) for the shortcut inside a primary button and the toast's close button, and **On Ink Danger** (#ff8a7d; #b42318 on the near-white ink of dark mode, 7.9:1 and 5.6:1) for the dot that marks a failure in the toast.

### Secondary
- **Highlighter** (#ffd84d; dark-highlighter #e8c547): words where the set-ups read differently, as a filled mark with a 2px bleed; a name pointed at in an analysed document; the swatch in the difference count and the legend; the marker stroke in the app's mark. Text on it is always **On Highlighter** (#171612), in both themes.

### Tertiary
- **Hand Blue** (#2456d6; dark-hand-blue #7fa2ff): the researcher's corrections, as a 2px underline under corrected words, the "Corrected" chip and the legend key. **Hand Wash** (10% blue) is the hover ground of a corrected word and the text selection colour.

### Neutral
- **Chrome Grey** (#efece6; dark-chrome #171614): the sidebar, plain chips. The frame around the work.
- **Surface** (#fdfcfa; dark-surface #1d1c1a), with **Surface Raised** (#ffffff; #272522) for what is lifted onto it: the main work surface, mastheads, buttons, inputs, the reading text. **Surface Quiet** (#f8f6f2): side panels (New job's summary), the review page strip, the audio source pane, hover on choice tiles.
- **Stage** (#1f1e1c; dark-stage #100f0e): the lightbox under a scan. **Stage Raised** (#2b2a27) for the zoom pill, **Stage Line** (#383632) for its rules, **On Stage** (#e8e6e1) and **On Stage Muted** (#aeaaa2) for text on it.
- **Lines**: line (#e5e1d9) for borders and dividers, line-soft (#eeebe5) between rows of one table, line-strong (#d6d1c7) for dashed slots, sigla and drawings, line-hover (#a8a297) for a border under the pointer (splitters, cards, page tiles).
- **Control line** (#8a8478; dark-control-line #7a756c): the edge of what is operated, where the edge is what tells it from the page: fields, selects, text areas, the language pickers' boxes, the default button, the radio drawn in the task strip. At least 3:1 against every surface a control sits on (light: 3.7 on white, 3.6 on surface, 3.4 on surface quiet, 3.2 on chrome; dark: 3.3 on surface raised, 3.7 on surface, 3.6 on surface quiet, 3.9 on chrome), as WCAG asks of a control's boundary, and still a pencil line, lighter than any text. **Control line hover** (#5d5950; #a09a8f in dark) under the pointer and on a view that is on (pressed); focus takes bright text. Dividers, tables and hairlines keep the quiet lines.
- **Wash** (5% ink): hover ground for ghost buttons, list rows and transcript lines.
- **Found** (16% ink; 20% white in dark): words a search found, as a wash over the text; the match gone to is filled in graphite ink with on-ink text, the way a focus is drawn. Neither yellow, which marks the models' disagreements, nor blue, which is the researcher's hand.
- **Text**: bright (#171612) for headings and values, body (#1e1d19), secondary (#4d4a42), muted (#615d54) for hints and metadata (a field's kind in an analysis, an item a menu cannot do now), faint (#938e84, 3.2:1) only for decoration that repeats a value given elsewhere (line numbers, the slash in a breadcrumb), never for words to read.
- **Status**: ok-green (#1f7a4d), warn-amber (#d97f00), danger-red (#b42318), each with a darker text tone and a pale wash. Used as coloured text on a wash, a 6px dot, or a thin bar; never as a filled button.
- **Speakers**: speaker 1 teal (#0f8a7e; #5cc8bc in dark) and speaker 2 amber (#d97f00), each as a tag of dark text on a pale wash, told apart by lightness as well as hue. Neither is blue, which belongs to the researcher's hand.

### Named Rules
**The Black Button Rule.** Graphite ink is the only button fill. Each work area has at most one primary button, and it shows its keyboard shortcut (Ctrl N for New job, Ctrl Enter for Start and Mark reviewed). The sidebar's New job button is the shell's standing action and does not count against a screen.

**The Highlighter Rule.** Yellow means "look here": where the models disagree, or where something you pointed at is found. It is never a warning, a brand fill or decoration. Warnings are amber text on a pale amber wash.

**The Hand Rule.** Blue is the researcher's own hand. Corrections are underlined in blue on top of the text; the AI's output underneath is never overwritten or restyled as if it were the researcher's.

**The Lightbox Rule.** A scan always sits on the stage, dark in both themes, so the page is the brightest thing on screen. Controls over it use the on-stage colours, not the interface's.

## Typography

**Interface Font:** Instrument Sans Variable (with Segoe UI Variable, Segoe UI, system-ui)
**Text Font:** Source Serif 4 Variable (with Times New Roman, which also covers Arabic script)
**Machine Font:** IBM Plex Mono (with Cascadia Mono, Consolas)

**Character:** a crisp contemporary grotesque that stays out of the way, a sturdy reading serif with optical sizes for the documents themselves, and a plain mono that labels whatever a machine produced. Three faces, three jobs, never swapped.

### Hierarchy
- **Title** (Instrument Sans 600, 1.25rem, 1.25, -0.01em): masthead titles, the app name, step headings in New job.
- **Heading** (Instrument Sans 600, 1.125rem, 1.25): section headings (h2); h3 at 1rem.
- **Body** (Instrument Sans 400, 1rem, 1.45): all interface text. Paragraphs in a sheet keep a 75ch measure.
- **Label** (Instrument Sans 500, 0.9375rem): buttons, field labels, list items, inputs.
- **Caption** (Instrument Sans 500, 0.875rem): chips, metadata, hints, sidebar group heads, the page strip.
- **Reading** (Source Serif 4 400, 1.1875rem, 1.7): the text being checked and the correction editor, as wide as their pane. Rises to 1.3125rem in windows 1680px and wider.
- **Reading small** (Source Serif 4 400, 1.125rem, 1.55 to 1.65): readings in the readings card, transcript lines, analysis answers, text previews in a job.
- **Machine** (IBM Plex Mono 400, 0.92em of its context): file names, model and set-up labels, times, costs, counts, page numbers, output paths, the prompt editor.
- **Machine mark** (IBM Plex Mono, 0.75rem): keyboard shortcuts, step numbers (01, 02), marginal line numbers, sigla, sidebar counts. The smallest size in the system.
- **Figure** (IBM Plex Mono 500, 1.875rem, -0.02em): the cost estimate in New job’s summary; 1.5rem when the summary becomes a footer.
- Weights stay at 400, 500 and 600. Tabular figures where numbers line up.

### Named Rules
**The Three Voices Rule.** Instrument Sans for everything operated, Source Serif 4 only for the text being checked, IBM Plex Mono for anything a machine produced. A control in the serif, or a document in the sans, is a defect. One exception, which is the signature rather than a control: the app's name beside the mark in the sidebar is set in Source Serif 4 600 at 1.1875rem (optical size 48).

## Layout

The app is a fixed shell: an 18rem chrome-grey sidebar at left and a scrolling white main area. The sidebar holds the mark and name, the black New job button with Ctrl N, the jobs list grouped For you to check, Running, Unfinished and Finished (each group head a caption with a mono count; Finished shows the eight most recent, plus the job open in the main area if it is older, and ends with an *All jobs (N)* link to the Jobs page), and at its foot the tools (Prompts, Settings, Glossary) above a hairline, with an offline-ready line and green dot. A job row shows its name with a task icon, a caption line of status, and a 3px ink progress bar while running. The current row or tool is lifted onto white with the raised shadow. Under Projects, the projects are listed, indented, each with a mono count of its documents, folded by a chevron (remembered on the computer). A finished job dragged from the list (with the pointer: the window keeps the browser’s own drag and drop for files) shows as a small ink chip under the pointer; every project takes a hairline ring and the one under the pointer an ink ring, and dropping adds the job to it. The sidebar folds (its button beside the name, or Ctrl B) to a 3.25rem **rail** of the same controls as icons, named by their tooltips: the job list leaves, New job, the Jobs page and the tools stay; the choice is remembered on the computer.

The **mark** is "two witnesses": on a warm graphite tile (#1e1d1b), a cream handwritten sheet with brown strokes lies behind a white transcribed sheet whose middle line is under the highlighter. It keeps its own colours in both themes, with a faint rim so the tile holds on a dark taskbar. The app icon from 64px up (Start, Store, installer) is a rendered master, `src-tauri/icons/source.png`, generated with OpenAI GPT Image (prompt in the `.json` beside it); at 16 to 48px (taskbar, tray, title bar) and in the sidebar at 22px it is a simplified drawing on a 22-unit grid, `src-tauri/icons/small.svg`, whose strokes hold at 16px. `npx tauri icon src-tauri/icons/source.png` writes the large sizes; the small ones and the ICO's 16 to 48px frames are rendered from `small.svg`.

Each screen opens with a **masthead**: at least 3.5rem tall, white, a hairline below, the title (and a back link or breadcrumb) at left and actions at right. Below it a **sheet**: one column as wide as the window, padding 1.5rem 2rem 3rem, sections 2rem apart.

**Back** returns to the screen the researcher came from, as it was left: a project's search with its results, the list of what to check at its item, a document, a job. It is Alt ← (and ⌘[ on a Mac) and the mouse's back button anywhere outside a field; in a review Alt and the arrows turn pages, so there a Mac's ⌘[ is the only key. The last 30 screens are kept, in memory only, and a change inside a screen (a project's pane, a collection, a review's page, stepping through what to check) replaces its place rather than adding one. A back link that names where it goes (*Projects*, the project's name, *All workflows*, the job in a review's breadcrumb) goes back when that is where the screen was opened from, and otherwise opens it; a review opened from elsewhere adds an icon-only Back button before its breadcrumb. Every way out still asks first about a changed draft.

**Remembered on the computer**, each read back only if still valid (a filter or tag no longer offered stops filtering): the Jobs page's task, status and sort (not the words typed), each project's pane and its table's sort and tag, and each scan's quarter turns (for the 200 sources turned last). **Ctrl F** (⌘F) goes to the screen's filter or search field (Jobs, a project's documents, search and dictionary, the Glossary), marked `data-find`; a review keeps its own search.

While files are dragged over the window, the screen that takes them (New job, a project) is covered by a **drop target**: a 2px dashed line-strong ring inset 0.75rem from the work area, on the wash, with a raised label saying what dropping does; it goes when the files leave, are dropped or the drag is given up.

**New job** is a form beside a summary (a ghost **Start over** in the masthead empties it once it holds files or a name; the form is otherwise kept when the researcher leaves for Settings and comes back): the form (as wide as the window leaves it) runs numbered steps, each with a mono step number (01, 02…) hung in a 2.75rem margin and counted by CSS so a step a task does not need leaves no gap; a 21rem sticky summary aside on surface-quiet carries the estimate as a large mono figure, the rows behind it, and the large Start button with its shortcut. Choices such as the three tasks are one joined control: tiles in a single bordered strip, the selected tile lifted to white with a 1.5px inset ink ring and a filled radio.

**Review** splits the work area 0.95fr (source) to 1.05fr (text); **Compare** in the masthead adds a third pane with another set-up's whole text (a third each). The boundaries between panes are **splitters**: a 9px invisible grip on the hairline that shows a 3px line on hover (line-hover) and while dragged or focused (bright text); arrow keys move it 2% (Shift 10%), a double click puts it back, and no pane gets narrower than about 16rem. Widths and the open compare pane are remembered on the computer, the same for the three reviews (text, analysis, table of contents), which share one frame. Each pane has a 2.75rem bar on top. The text pane sets each line in a grid with a 2.25rem number column, numbers printed every fifth line and on every line that differs. **Search** (Ctrl F, or the magnifier in the masthead) opens a row under the masthead: the field, the count ("2 of 11"), previous and next, a **List** of the matches in a flyout grouped by page under sticky mono headings, each with the words around it and, in a recording, its time; Escape closes the list, then the search. A page strip runs along the bottom on surface-quiet: for a scan, each page as a small thumbnail with its number under it; for a recording, a numbered tile per piece; and a tally with a green progress bar.

**Jobs** is one bordered table of every job (`.data-table`), with a filter row above it (`.table-filters`) and, when rows are chosen, a selection bar (`.selection-bar`). Its columns have fixed widths, so a long name wraps rather than widening the page, and drop one by one through container queries as the page itself narrows, whether the sidebar is open or folded.

**Long tables are drawn a page at a time**: the Jobs table, a project's documents, its dictionary and a job's files draw their first 200 rows, with "200 of 5000 shown" and a ghost **Show 200 more** under them (`MoreRows`), as the list of what to check does; the focus goes to the first row added. Finding, filtering, sorting, ticking all and the selection bar work on every row, not only those drawn, and a filter or sort draws from the first page again. A screen reader hears the whole count (`aria-rowcount`, each row's `aria-rowindex`).

In a project's documents, each task's marks in the **Results** column (its icon, the count, a check for reviewed, a star for preferred) are one quiet button that opens the document: each mark names itself on pointing, and the keyboard reaching the button shows the whole in words under it, in a small raised label (the flyout shadow, 5px corners), which is also its accessible name.

The **Glossary** ends with **Keyboard**: every shortcut, grouped by where it works (everywhere, New job, a review, a review's search, a scan), the keys in mono as the system in use writes them, alternatives joined by a muted "or".

**Settings** opens on a tab list (API keys, Offline, General, About), each tab a sheet of its own; a link into Settings opens the tab it is about.

**Job detail** sets the job's facts as one bordered strip of cells divided by hairlines, the short facts in one row and the errors found in review in a row of their own under them, as wide as the strip so each set-up's sentence stays on one line; then the files as a bordered table with per-set-up progress bars, counted by the engine for the files drawn (`job_progress`) rather than from every piece.

Spacing follows a 4px-based scale (0.25, 0.5, 0.75, 1, 1.5, 2, 3rem). Controls are 1.875rem (small), 2.25rem (default) and 2.75rem (large) tall.

**Responsive range:**
- **1100px and narrower**: New job's summary leaves the side and becomes a sticky footer with the figure and Start, lifted by a soft upward shadow.
- **1000px and narrower**: every review stacks the source above the text (source 16rem to 42vh), and the compared text below both; the splitters hide. In a short window the source gives way first and the text keeps at least 16rem; when even that does not fit, the work area scrolls above the page strip. Prompts stacks its list above the editor.
- **820px and narrower**: masthead and sheet padding tighten to 1rem; the review tally hides.
- **760px and narrower**: the sidebar sits above the work, folded to one row whatever was chosen in a wide window (its button or Ctrl B unfolds it to a scrolling column of at most 38vh); joined choice strips become one column.
- **1680px and wider**: reading text rises to 1.3125rem.

At no width does a view scroll sideways: grids, tables, rows and diagrams fit the width there is.

## Elevation & Depth

Flat by default, with tonal layering doing most of the work: chrome grey behind, white for the work, a quiet grey for side panels. Shadows appear on three kinds of thing: a selected item lifted onto white, an overlay that floats over content, and a scan lying on the lightbox.

### Shadow Vocabulary
- **Raised** (`box-shadow: 0 0 0 1px var(--line), 0 1px 2px rgba(23, 22, 18, 0.06)`; dark: the ring alone): the current sidebar job or tool, the active prompt, the audio player, the transcript line now playing.
- **Pop** (`box-shadow: 0 0 0 1px var(--line), 0 12px 32px -8px rgba(23, 22, 18, 0.18)`): the language list in New job.
- **Flyout** (`box-shadow: 0 18px 48px -16px rgba(23, 22, 18, 0.35)`): the readings over the text, the close dialog and the toast.
- **Page** (`box-shadow: 0 1px 2px rgba(0, 0, 0, 0.45), 0 16px 40px rgba(0, 0, 0, 0.4)`): a scan on the lightbox, with a 2px corner.
- **Zoom pill** (`box-shadow: 0 0 0 1px var(--stage-line), 0 8px 24px rgba(0, 0, 0, 0.35)`): the controls floating over the stage.

### Named Rules
**The Lift Means Selected Rule.** In lists, being lifted onto white with the raised shadow means "this is the one you are on". Hover only takes the wash.

## Shapes

Corners are soft and small: 7px on buttons, inputs and list rows, 10px on cards, tables, joined choice strips, the player and overlays, 5px on chips and speaker tags, 2px on highlights and the scan's edge. Fully round only where the form is round: the play button, the scrubber thumb, radios and status dots (6px).

Lines carry the structure: 1px hairlines everywhere; a dashed strong line for an empty slot that adds something (a drop zone, "add a model" column). Sections in a sheet are not boxed: a hairline above, a heading, no fill.

Icons come from Lucide through one icon component, at an absolute 1.5px stroke in the current colour, play and pause filled. They are decorative; the control carries the accessible name.

## Components

### Buttons
Quiet and firm: a sans label at weight 500, an icon only where its meaning is universal (zoom, previous, next, close).
- **Shape:** 7px corners, 2.25rem tall, 0 0.75rem padding; small 1.875rem with 0.625rem padding and caption type; large 2.75rem with 1.5rem padding.
- **Default:** white ground, control-line border, bright text; hover darkens the border to control-line-hover, as does a view that is on (pressed in, on the wash).
- **Primary:** graphite ink ground and border, white text, a muted mono shortcut inside; hover lifts to ink hover.
- **Ghost:** transparent; hover takes the wash.
- **Danger:** danger-red text with a pale red border, never a red fill.
- **Disabled:** 45% opacity. In a menu or list of choices, an item that cannot be used now is muted text with no hover, legible beside the bright items. **Focus:** a 2px ink outline offset 2px (on-stage colour over the lightbox).
- Transitions: background and border colour, 120ms ease-out; removed under reduced motion.

### Chips
- **Style:** caption type on a tinted ground, 5px corners, no border.
- **Variants:** plain (chrome grey), ok, warn, danger and hand, each its text tone on its wash with a 6px dot of the full colour before the label.

### Siglum
A set-up's letter wherever set-ups are compared: a 1.25rem square of mono uppercase in secondary text, line-strong border, 4px corners, white ground.

### Inputs / Fields
- **Style:** white ground, control-line border, 7px corners, 2.25rem tall, padding 0.25rem 0.75rem; textareas 0.75rem padding and 1.55 line height. Label above in secondary text, weight 500.
- **Hover:** border to control-line-hover. **Focus:** border to bright text with a 3px wash ring, no outline.
- **Checkboxes and radios** are the system's own, checked in ink (`accent-color`), with the system's edge. Joined choice strips keep a quiet line round them: their radios, not their edge, say they are choices.

### Navigation
- **Sidebar:** chrome grey, 18rem (3.25rem folded), hairline right edge. Rows are 7px-cornered, transparent at rest, wash on hover, lifted onto white when current. A job row's menu is a small ghost ⋯ button at its end, shown only under the pointer, with the focus in the row or while the menu is open (the name makes room for it then); Tab always reaches it after its job, and the menu opens under it, the focus going back to it when the menu closes. The New job button is a full-width ink button; when New job is open it takes a 1.5px ink ring offset by the chrome.

### Masthead
White, at least 3.5rem, hairline below; a title in title type, an optional muted lede or a back link, actions at right. In Review the title is a breadcrumb: the job name, a faint slash, then the file name in mono, with a pager (previous, a mono select, next) and the actions.

### Review: the lightbox and the marked text (signature)
- **Source pane:** a scan on the stage, padded 1.5rem with room below for a centred **zoom pill** (stage raised, 9px corners, ghost controls in on-stage colours, a mono percentage). The quarter turns of a page scanned sideways are remembered for the source. Zooming, from 25% to 600%, follows Ctrl and the wheel or a pinch frame by frame (the drawing is stretched at once and drawn sharp again once it rests) and keeps the point under the pointer in place; the buttons, Ctrl + and Ctrl − glide there in 180ms around the middle of the pane. A page larger than its pane is moved by dragging it (grab cursor), gliding on a little when let go, or from the keyboard: the page is a tab stop ("Page image"), its focus a 2px on-stage outline inside the pane's edge, and the arrow keys move it 48px, with Shift most of the pane; a double click zooms to 250% on the place, or back to the pane's width. Until the reader has zoomed once, a pill above the zoom pill says how, while the pointer is over the page. A recording instead sits on surface-quiet with the player on top.
- **Text pane:** a bar with the set-up's label in mono, a count of differences beside a yellow swatch, and a switch to show them; the reading text below; a foot with mono metadata and a legend (yellow block for "models differ", blue line for "your correction").
- **Disagreements:** each differing word is a button set in the highlighter with a 2px bleed. Pointing at it or opening it adds a 1.5px bright ring outside the yellow. A text's differences are one tab stop however many there are, held by the one open or last reached (the first otherwise): ← and → go from one to the next, Home and End to the first and last, without opening them; ↑ and ↓ still scroll the pane. Words another model adds, where the text has none, are a 0.3em highlighter bar between the words.
- **Readings:** pointing at a disagreement for 300ms shows its readings over the text, under the words (over them when the pane has no room below), max 25rem, white with the flyout shadow and 10px corners, rising 3px into place over 120ms. They stay while the pointer is on the words or on them; a click, F, or focusing inside pins them until Escape, the close button or a click elsewhere. Opened from the keyboard (Enter or Space on the words, F or Shift F), they take the focus, on their first reading to use; Tab runs through them from the words, which they follow in the tab order, and Escape (in their own field too), the close button or a reading used gives the focus back to the words, or after a correction to that place or the next difference once the differences are worked out again. Readings shown by pointing never take the focus. Their head is a handle: dragged (or moved with the arrow keys from the grip at its left, Shift for longer steps) they move aside to show the sentence under them, and stay where they were put. Choosing a difference in either text brings it level in the other. The one in the text comes first, in bold, with an "In the text" chip; each other reading is a whole row to click, its words in the reading serif over who reads them (mono), "Use this reading" at right; a field takes your own.
- **Compared text:** another set-up's own words on paper, in the reading serif without line numbers, its differences in the same highlighter (a gap mark where it leaves words out). Pointing at a difference in either text rings it in both; clicking one in the compared text pins its readings in the text. A select picks the set-up when there are several.
- **Corrections:** corrected words lose the yellow and take a 2px hand-blue underline offset 0.25em; hover takes the hand wash.
- **Turning pages:** Page Up and Page Down, or Alt ← and Alt →, turn to the previous or next piece, except in a field or editor, and except that Page Up and Page Down scroll a pane that has the focus to be scrolled (the page image, a text file, a text pane holding nothing else to focus), as they do everywhere. Every pane scrolls from the keyboard in both webviews: one holding a button or a field is scrolled from there; one holding nothing focusable becomes a tab stop, named for what it holds, once its text is taller than it.
- **Analysis answers:** the names of an answer are one tab stop, ← and → going from one to the next; a name pointed at or focused takes the highlighter and is marked where it stands in the document. A click, Enter, a right click or the menu key (Shift F10) opens its menu, rename or remove, under the name; closing it gives the focus back to the name, or after a change to the name renamed or the next one.
- **Page strip:** for a scan, each page as a 2.5rem-wide thumbnail (3:4, top of the page, 2px corners, a hairline ring) over its mono number; thumbnails are drawn by the app at their own size (`piece_thumbnail`, a few kilobytes) as they scroll into view, two at a time, and kept for the session; the page in the source pane is drawn with pdf.js, one worker shared by every page opened. For a recording, 2rem tiles with a mono number. Reviewed pages show a check and ok-green number; the current page is ringed in bright text and lifted onto the raised surface. A tally beside it with a 4px green bar.

### Audio Player
A white strip with 10px corners and the raised shadow: a round 2.5rem ink play button, mono times, a 4px track filled in ink with an ink thumb ringed in white. Below, transcript lines: a mono timestamp, a speaker tag, and what was said in the reading serif. Hover takes the wash; the line now playing is lifted onto white.

### New job steps and summary
Numbered steps with mono numbers in the margin; joined choice strips for task and output; a model set-up as a white bordered card, side-by-side columns (min 17rem) when comparing, with a dashed column that adds another model. The summary aside ends in the large Start button with Ctrl Enter.

Under the task strip, **How it works** sits on surface-quiet with 10px corners: the task's three or four steps in a row, each a small drawing over a caption, joined by Lucide chevrons (two a row, without chevrons, in a narrow form). The drawings use the interface's own marks: white sheets with line-strong hairlines, line-hover strokes for text, the highlighter on names an analysis finds (with ink on it), the speaker colours in a transcript, ink for a progress sweep; file names, page numbers and lengths in mono. It plays once, about 2.5 s, when a task is picked or Play again is pressed; its resting styles are the last frame, so reduced motion shows it whole.

### Toast
A message about what was just done, on ink with the flyout shadow, bottom right. A confirmation stays four seconds and is read out politely. A failure is read out at once and stays until it is put away with its close button or Escape: the same ink, marked by a 6px on-ink-danger dot before the words, never a red fill. Both speak through live regions that are always there, empty, so the words put in are read in every webview. A confirmation of a removal that can be taken back (a name from an analysis's answers, documents from a collection) ends in a small ghost button, *Undo*, on-ink with a 14% on-ink hover and an on-ink focus ring, reachable by Tab; it then stays eight seconds, and the focus comes back to the name put back.

### Notices
A full-width banner or inline notice in warn-text on the warn wash (7px corners inline), stating what happened and what to do. File problems hang under their file in the same treatment, danger-tinted when a file cannot be used.

A newer release is not a warning: it is a quiet line in caption type and muted text at the sidebar's foot, under the offline line, with its link and a small ghost close button that hides it until the next version; Settings › About says it too, without the button. Store copies never show it.

## Do's and Don'ts

### Do:
- **Do** use graphite ink (#171612, #eeece6 in dark) as the only button fill, once per work area, with its shortcut beside the label.
- **Do** keep highlighter yellow (#ffd84d / #e8c547) for disagreements and found terms, always with ink text on it.
- **Do** mark the researcher's corrections in hand blue (#2456d6 / #7fa2ff) as an underline on top of the text.
- **Do** put scans on the stage (#1f1e1c / #100f0e) with the page shadow, and the controls over it in on-stage colours.
- **Do** set file names, models, times, costs, counts and shortcuts in IBM Plex Mono, and only the text being checked in Source Serif 4.
- **Do** separate with greys and hairlines first; reach for a shadow only for the selected item, an overlay or the scan.
- **Do** show status as coloured text on a wash, with a 6px dot.
- **Do** keep icons to Lucide at the absolute 1.5px stroke, beside a word.

### Don't:
- **Don't** fill a button in any colour but ink: no blue, green or red primary buttons.
- **Don't** use yellow for warnings, emphasis or branding; warnings are amber on a pale amber wash.
- **Don't** use hand blue for links, selected states or AI output; it belongs to the researcher's hand (and the text selection it makes).
- **Don't** put a scan on white or on the interface grey.
- **Don't** set controls in the serif or documents in the sans.
- **Don't** box sections in shadowed cards; a section is a hairline and a heading.
- **Don't** use glyph characters (arrows, crosses, triangles) as icons.
- **Don't** add uppercase eyebrows or kickers above titles.

## Open Questions

Left for the user, not shipped as rules:
- The direction brief says yellow also marks "still to review"; the build does not do this (the page strip shows reviewed pages in green and leaves the rest plain). Decide whether unreviewed pages should take the highlighter.
- Recordings sit on surface-quiet, not on the lightbox. Decide whether the Lightbox Rule covers audio.
- A few radii sit off the scale (zoom pill 9px, zoom buttons and page tiles 6px, siglum 4px); fold them into the scale or record them.
