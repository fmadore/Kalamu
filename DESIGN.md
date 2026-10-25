---
name: Scriptorium
description: AI transcription, text extraction and analysis for researchers, checked against the source.
colors:
  chrome-grey: "#f2f3f5"
  surface: "#ffffff"
  surface-quiet: "#fafafb"
  stage: "#1e1f22"
  stage-raised: "#2a2b2f"
  stage-line: "#34353a"
  on-stage: "#e4e5e9"
  on-stage-muted: "#a9acb4"
  line: "#e3e4e8"
  line-soft: "#eeeff2"
  line-strong: "#d5d7dc"
  line-hover: "#a4a8b0"
  graphite-ink: "#111214"
  graphite-ink-hover: "#2b2d32"
  on-ink: "#ffffff"
  on-ink-muted: "#b9bcc3"
  wash: "rgba(17, 18, 20, 0.05)"
  highlighter: "#ffe066"
  on-highlighter: "#111214"
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
  text-bright: "#111214"
  text-body: "#1b1c1f"
  text-secondary: "#4a4e57"
  text-muted: "#5e626b"
  text-faint: "#8f939b"
  dark-chrome: "#16171a"
  dark-surface: "#1c1d20"
  dark-surface-raised: "#24252a"
  dark-stage: "#0f1012"
  dark-line: "#2c2d31"
  dark-ink: "#ecedef"
  dark-highlighter: "#e8c547"
  dark-hand-blue: "#7fa2ff"
  dark-text-body: "#dfe1e5"
  dark-text-muted: "#9a9ea6"
typography:
  title:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "1rem"
    fontWeight: 600
    lineHeight: 1.25
    letterSpacing: "-0.01em"
  heading:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "0.9375rem"
    fontWeight: 600
    lineHeight: 1.25
    letterSpacing: "-0.01em"
  body:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "0.9375rem"
    fontWeight: 400
    lineHeight: 1.45
  label:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 500
    lineHeight: 1.45
  caption:
    fontFamily: "'Instrument Sans Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "0.78125rem"
    fontWeight: 500
    lineHeight: 1.45
  reading:
    fontFamily: "'Source Serif 4 Variable', 'Times New Roman', serif"
    fontSize: "1.125rem"
    fontWeight: 400
    lineHeight: 1.7
  reading-small:
    fontFamily: "'Source Serif 4 Variable', 'Times New Roman', serif"
    fontSize: "1.0625rem"
    fontWeight: 400
    lineHeight: 1.6
  machine:
    fontFamily: "'IBM Plex Mono', 'Cascadia Mono', Consolas, monospace"
    fontSize: "0.92em"
    fontWeight: 400
  machine-mark:
    fontFamily: "'IBM Plex Mono', 'Cascadia Mono', Consolas, monospace"
    fontSize: "0.6875rem"
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
    width: "16rem"
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

# Design System: Scriptorium

## Overview

**Creative North Star: "Graphite & highlighter"**

A researcher checking a transcript works the way they would with a printout: a grey pencil for structure, a yellow highlighter for what needs another look, a blue pen for their own corrections. The interface is that desk kit and nothing more. Grey surfaces and hairlines do the structural work, black is kept for the one action that moves the work forward, yellow says "look here" and blue says "you changed this". Everything else stays quiet so those three marks can be read at a glance across a page of text.

The source is the authority, so it gets the best seat: a scan lies on a dark lightbox in both themes, the brightest thing on screen, with its controls floating on the dark in a small pill. The text being checked sits beside it on white, in a reading serif. The interface around both is a plain sans; anything a machine produced (file names, model names, times, costs, counts, shortcuts) is set in a monospace, so the researcher can tell their material, the tool and the machine apart by the face alone. Density is that of a working tool used for hours on a laptop: compact controls, generous reading measure, no decoration.

The same world holds from a window snapped beside another to a large monitor, and in light and dark themes that follow Windows by default. In dark mode the ink flips to near white, so the primary action stays the highest-contrast object on screen.

**Key Characteristics:**
- Graphite neutrals: a grey chrome sidebar (#f2f3f5), white work surface, hairline borders.
- Black (graphite ink) is the only button fill, used once per work area and paired with its shortcut.
- Highlighter yellow marks only where models disagree or where a name is found in the document.
- Hand blue marks only the researcher's own corrections, never AI output.
- Scans sit on a dark lightbox (#1e1f22 light, #0f1012 dark) in both themes.
- Three faces with three jobs: Instrument Sans, Source Serif 4, IBM Plex Mono.
- Lucide line icons at an absolute 1.5px stroke, beside words.

## Colors

A graphite palette of greys and one black, with two marker colours whose rarity carries the meaning and a small set of status hues used as text on a wash.

### Primary
- **Graphite Ink** (#111214; dark-ink #ecedef in dark): primary buttons, the sidebar's New job button, the toast, progress fills, the play button and scrubber, the selected task's inset ring, focus outlines and the checked state of native controls. Hover lifts to **Graphite Ink Hover** (#2b2d32). Text on it is **On Ink** (#ffffff), with **On Ink Muted** (#b9bcc3) for the shortcut inside a primary button.

### Secondary
- **Highlighter** (#ffe066; dark-highlighter #e8c547): words where the set-ups read differently, as a filled mark with a 2px bleed; a name pointed at in an analysed document; the swatch in the difference count and the legend; the marker stroke in the app's mark. Text on it is always **On Highlighter** (#111214), in both themes.

### Tertiary
- **Hand Blue** (#2456d6; dark-hand-blue #7fa2ff): the researcher's corrections, as a 2px underline under corrected words, the "Corrected" chip and the legend key. **Hand Wash** (10% blue) is the hover ground of a corrected word and the text selection colour.

### Neutral
- **Chrome Grey** (#f2f3f5; dark-chrome #16171a): the sidebar, plain chips. The frame around the work.
- **Surface** (#ffffff; dark-surface #1c1d20): the main work surface, mastheads, buttons, inputs, the reading text. **Surface Quiet** (#fafafb): side panels (New job's summary), the review page strip, the audio source pane, hover on choice tiles.
- **Stage** (#1e1f22; dark-stage #0f1012): the lightbox under a scan. **Stage Raised** (#2a2b2f) for the zoom pill, **Stage Line** (#34353a) for its rules, **On Stage** (#e4e5e9) and **On Stage Muted** (#a9acb4) for text on it.
- **Lines**: line (#e3e4e8) for borders and dividers, line-soft (#eeeff2) between rows of one table, line-strong (#d5d7dc) for input borders, dashed slots and sigla, line-hover (#a4a8b0) for a border under the pointer.
- **Wash** (5% ink): hover ground for ghost buttons, list rows and transcript lines.
- **Text**: bright (#111214) for headings and values, body (#1b1c1f), secondary (#4a4e57), muted (#5e626b) for hints and metadata, faint (#8f939b) only for decoration that repeats a value given elsewhere (line numbers, the slash in a breadcrumb).
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
- **Title** (Instrument Sans 600, 1rem, 1.25, -0.01em): masthead titles, the app name, step headings in New job.
- **Heading** (Instrument Sans 600, 0.9375rem, 1.25): section headings (h2); h3 at 0.875rem.
- **Body** (Instrument Sans 400, 0.9375rem, 1.45): all interface text. Paragraphs in a sheet keep a 75ch measure.
- **Label** (Instrument Sans 500, 0.875rem): buttons, field labels, list items, inputs.
- **Caption** (Instrument Sans 500, 0.78125rem): chips, metadata, hints, sidebar group heads, the page strip.
- **Reading** (Source Serif 4 400, 1.125rem, 1.7, max 42rem): the text being checked and the correction editor. Rises to 1.25rem in windows 1680px and wider.
- **Reading small** (Source Serif 4 400, 1.0625rem, 1.55 to 1.65): readings in the readings card, transcript lines, analysis answers, text previews in a job.
- **Machine** (IBM Plex Mono 400, 0.92em of its context): file names, model and set-up labels, times, costs, counts, page numbers, output paths, the prompt editor.
- **Machine mark** (IBM Plex Mono, 0.6875rem): keyboard shortcuts, step numbers (01, 02), marginal line numbers, sigla, sidebar counts. The smallest size in the system.
- **Figure** (IBM Plex Mono 500, 1.875rem, -0.02em): the cost estimate in New job's summary; 1.375rem when the summary becomes a footer.
- Weights stay at 400, 500 and 600. Tabular figures where numbers line up.

### Named Rules
**The Three Voices Rule.** Instrument Sans for everything operated, Source Serif 4 only for the text being checked, IBM Plex Mono for anything a machine produced. A control in the serif, or a document in the sans, is a defect.

## Layout

The app is a fixed shell: a 16rem chrome-grey sidebar at left and a scrolling white main area. The sidebar holds the mark and name, the black New job button with Ctrl N, the jobs list grouped Running, Unfinished and Finished (each group head a caption with a mono count), and at its foot the tools (Prompts, Settings) above a hairline, with an offline-ready line and green dot. A job row shows its name with a task icon, a caption line of status, and a 3px ink progress bar while running. The current row or tool is lifted onto white with the raised shadow. The sidebar folds (its button beside the name, or Ctrl B) to a 3.25rem **rail** of the same controls as icons, named by their tooltips: the job list leaves, New job and the tools stay; the choice is remembered on the computer.

The **mark** is a graphite tile with 5-unit corners on a 22-unit grid, two white text lines and, between them, a wider highlighter bar: a page with one line marked. The sidebar draws it at 22px and the app icon (taskbar, Start, Store, installer) is the same shapes, from `src-tauri/icons/source.svg`; its lines are 2 units thick so they hold at 16px.

Each screen opens with a **masthead**: at least 3.5rem tall, white, a hairline below, the title (and a back link or breadcrumb) at left and actions at right. Below it a **sheet**: one column up to 64rem, padding 1.5rem 2rem 3rem, sections 2rem apart.

**New job** is a form beside a summary: the form (max 52rem) runs numbered steps, each with a mono step number (01, 02…) hung in a 2.75rem margin and counted by CSS so a step a task does not need leaves no gap; a 21rem sticky summary aside on surface-quiet carries the estimate as a large mono figure, the rows behind it, and the large Start button with its shortcut. Choices such as the three tasks are one joined control: tiles in a single bordered strip, the selected tile lifted to white with a 1.5px inset ink ring and a filled radio.

**Review** splits the work area 0.95fr (source) to 1.05fr (text); **Compare** in the masthead adds a third pane with another set-up's whole text (a third each). The boundaries between panes are **splitters**: a 9px invisible grip on the hairline that shows a 3px line on hover (line-hover) and while dragged or focused (bright text); arrow keys move it 2% (Shift 10%), a double click puts it back, and no pane gets narrower than about 16rem. Widths and the open compare pane are remembered on the computer; the analysis review has the same splitter between document and answers. Each pane has a 2.75rem bar on top. The text pane sets each line in a grid with a 2.25rem number column, numbers printed every fifth line and on every line that differs. A page strip runs along the bottom on surface-quiet: one small tile per page with its state, and a tally with a green progress bar.

**Job detail** sets the job's facts as one bordered strip of cells divided by hairlines, then the files as a bordered table with per-set-up progress bars.

Spacing follows a 4px-based scale (0.25, 0.5, 0.75, 1, 1.5, 2, 3rem). Controls are 1.875rem (small), 2.25rem (default) and 2.75rem (large) tall.

**Responsive range:**
- **1100px and narrower**: New job's summary leaves the side and becomes a sticky footer with the figure and Start, lifted by a soft upward shadow.
- **1000px and narrower**: Review stacks the source above the text (source 16rem to 42vh), and the compared text below both; the splitters hide. Prompts stacks its list above the editor.
- **820px and narrower**: masthead and sheet padding tighten to 1rem; the review tally hides.
- **760px and narrower**: the sidebar sits above the work (at most 38vh), folded to one row; joined choice strips become one column.
- **1680px and wider**: reading text rises to 1.25rem.

## Elevation & Depth

Flat by default, with tonal layering doing most of the work: chrome grey behind, white for the work, a quiet grey for side panels. Shadows appear on three kinds of thing: a selected item lifted onto white, an overlay that floats over content, and a scan lying on the lightbox.

### Shadow Vocabulary
- **Raised** (`box-shadow: 0 0 0 1px var(--line), 0 1px 2px rgba(17, 18, 20, 0.06)`; dark: the ring alone): the current sidebar job or tool, the active prompt, the audio player, the transcript line now playing.
- **Pop** (`box-shadow: 0 0 0 1px var(--line), 0 12px 32px -8px rgba(17, 18, 20, 0.18)`): the language list in New job.
- **Flyout** (`box-shadow: 0 18px 48px -16px rgba(17, 18, 20, 0.35)`): the readings over the text, the close dialog and the toast.
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
- **Default:** white ground, line border, bright text; hover darkens the border to line-hover.
- **Primary:** graphite ink ground and border, white text, a muted mono shortcut inside; hover lifts to ink hover.
- **Ghost:** transparent; hover takes the wash.
- **Danger:** danger-red text with a pale red border, never a red fill.
- **Disabled:** 45% opacity. **Focus:** a 2px ink outline offset 2px (on-stage colour over the lightbox).
- Transitions: background and border colour, 120ms ease-out; removed under reduced motion.

### Chips
- **Style:** caption type on a tinted ground, 5px corners, no border.
- **Variants:** plain (chrome grey), ok, warn, danger and hand, each its text tone on its wash with a 6px dot of the full colour before the label.

### Siglum
A set-up's letter wherever set-ups are compared: a 1.25rem square of mono uppercase in secondary text, line-strong border, 4px corners, white ground.

### Inputs / Fields
- **Style:** white ground, line-strong border, 7px corners, 2.25rem tall, padding 0.25rem 0.75rem; textareas 0.75rem padding and 1.55 line height. Label above in secondary text, weight 500.
- **Hover:** border to line-hover. **Focus:** border to bright text with a 3px wash ring, no outline.

### Navigation
- **Sidebar:** chrome grey, 16rem (3.25rem folded), hairline right edge. Rows are 7px-cornered, transparent at rest, wash on hover, lifted onto white when current. The New job button is a full-width ink button; when New job is open it takes a 1.5px ink ring offset by the chrome.

### Masthead
White, at least 3.5rem, hairline below; a title in title type, an optional muted lede or a back link, actions at right. In Review the title is a breadcrumb: the job name, a faint slash, then the file name in mono, with a pager (previous, a mono select, next) and the actions.

### Review: the lightbox and the marked text (signature)
- **Source pane:** a scan on the stage, padded 1.5rem with room below for a centred **zoom pill** (stage raised, 9px corners, ghost controls in on-stage colours, a mono percentage). A recording instead sits on surface-quiet with the player on top.
- **Text pane:** a bar with the set-up's label in mono, a count of differences beside a yellow swatch, and a switch to show them; the reading text below; a foot with mono metadata and a legend (yellow block for "models differ", blue line for "your correction").
- **Disagreements:** each differing word is a button set in the highlighter with a 2px bleed. Pointing at it or opening it adds a 1.5px bright ring outside the yellow. Words another model adds, where the text has none, are a 0.3em highlighter bar between the words.
- **Readings:** pointing at a disagreement for 300ms shows its readings over the text, under the words (over them when the pane has no room below), max 25rem, white with the flyout shadow and 10px corners, rising 3px into place over 120ms. They stay while the pointer is on the words or on them; a click, F, or focusing inside pins them until Escape, the close button or a click elsewhere. The one in the text comes first, in bold, with an "In the text" chip; each other reading is a whole row to click, its words in the reading serif over who reads them (mono), "Use this reading" at right; a field takes your own.
- **Compared text:** another set-up's own words on paper, in the reading serif without line numbers, its differences in the same highlighter (a gap mark where it leaves words out). Pointing at a difference in either text rings it in both; clicking one in the compared text pins its readings in the text. A select picks the set-up when there are several.
- **Corrections:** corrected words lose the yellow and take a 2px hand-blue underline offset 0.25em; hover takes the hand wash.
- **Page strip:** 2rem tiles with a mono page number; reviewed pages in ok-green with a check; the current page ringed in bright text. A tally beside it with a 4px green bar.

### Audio Player
A white strip with 10px corners and the raised shadow: a round 2.5rem ink play button, mono times, a 4px track filled in ink with an ink thumb ringed in white. Below, transcript lines: a mono timestamp, a speaker tag, and what was said in the reading serif. Hover takes the wash; the line now playing is lifted onto white.

### New job steps and summary
Numbered steps with mono numbers in the margin; joined choice strips for task and output; a model set-up as a white bordered card, side-by-side columns (min 17rem) when comparing, with a dashed column that adds another model. The summary aside ends in the large Start button with Ctrl Enter.

### Notices
A full-width banner or inline notice in warn-text on the warn wash (7px corners inline), stating what happened and what to do. File problems hang under their file in the same treatment, danger-tinted when a file cannot be used.

## Do's and Don'ts

### Do:
- **Do** use graphite ink (#111214, #ecedef in dark) as the only button fill, once per work area, with its shortcut beside the label.
- **Do** keep highlighter yellow (#ffe066 / #e8c547) for disagreements and found terms, always with ink text on it.
- **Do** mark the researcher's corrections in hand blue (#2456d6 / #7fa2ff) as an underline on top of the text.
- **Do** put scans on the stage (#1e1f22 / #0f1012) with the page shadow, and the controls over it in on-stage colours.
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
