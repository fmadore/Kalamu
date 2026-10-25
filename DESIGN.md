---
name: Scriptorium
description: AI transcription and text extraction for researchers, checked against the source like a critical edition.
colors:
  page-white: "#ffffff"
  pane-grey: "#f5f6f8"
  desk-grey: "#ececec"
  hairline: "#e2e3e7"
  hairline-strong: "#c6c9d1"
  hairline-hover: "#9aa0ad"
  section-rule: "#111111"
  bookcloth: "#1d2b4f"
  bookcloth-light: "#2a3a63"
  bookcloth-line: "#34446d"
  on-bookcloth: "#ffffff"
  rubric: "#b3261e"
  lemma-tint: "#dce3f2"
  ink-bright: "#0f1115"
  ink-body: "#1b1e24"
  ink-secondary: "#464b55"
  ink-muted: "#5f636d"
  ok-green: "#2e6b47"
  warn-umber: "#8a5300"
  danger-red: "#a8201a"
  dark-page: "#11151f"
  dark-paper: "#161b27"
  dark-desk: "#0c0f16"
  dark-section-rule: "#c9cfdc"
  dark-bookcloth: "#18264a"
  dark-accent: "#afc3ec"
  dark-rubric: "#ec8478"
  dark-lemma-tint: "#2a3963"
  dark-ink-body: "#dde1e8"
typography:
  wordmark:
    fontFamily: "'EB Garamond Variable', Garamond, 'Times New Roman', serif"
    fontSize: "1.875rem"
    fontWeight: 500
    lineHeight: 1
    letterSpacing: "0.01em"
  headline:
    fontFamily: "'EB Garamond Variable', Garamond, 'Times New Roman', serif"
    fontSize: "2rem"
    fontWeight: 500
    lineHeight: 1.2
    letterSpacing: "0.005em"
  title:
    fontFamily: "'EB Garamond Variable', Garamond, 'Times New Roman', serif"
    fontSize: "1.375rem"
    fontWeight: 500
    lineHeight: 1.2
  subtitle:
    fontFamily: "'EB Garamond Variable', Garamond, 'Times New Roman', serif"
    fontSize: "1.125rem"
    fontWeight: 500
    lineHeight: 1.2
  reading:
    fontFamily: "'EB Garamond Variable', Garamond, 'Times New Roman', serif"
    fontSize: "1.3125rem"
    fontWeight: 400
    lineHeight: 1.65
    fontFeature: "onum"
  apparatus:
    fontFamily: "'EB Garamond Variable', Garamond, 'Times New Roman', serif"
    fontSize: "1.0625rem"
    fontWeight: 400
    lineHeight: 1.6
    fontFeature: "onum"
  running-head:
    fontFamily: "'EB Garamond Variable', Garamond, 'Times New Roman', serif"
    fontSize: "1.125rem"
    fontWeight: 400
    letterSpacing: "0.08em"
    fontFeature: "all-small-caps"
  siglum:
    fontFamily: "'EB Garamond Variable', Garamond, 'Times New Roman', serif"
    fontSize: "1.2em"
    fontWeight: 600
    lineHeight: 1
    letterSpacing: "0.04em"
    fontFeature: "all-small-caps"
  body:
    fontFamily: "'Source Sans 3 Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "0.9375rem"
    fontWeight: 400
    lineHeight: 1.45
  label:
    fontFamily: "'Source Sans 3 Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "0.8125rem"
    fontWeight: 600
    lineHeight: 1.45
  caption:
    fontFamily: "'Source Sans 3 Variable', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
    fontSize: "0.75rem"
    fontWeight: 600
    lineHeight: 1.5
  prompt-mono:
    fontFamily: "ui-monospace, 'Cascadia Mono', Consolas, monospace"
    fontSize: "0.92em"
    fontWeight: 400
    lineHeight: 1.55
rounded:
  hairline: "2px"
  tag: "4px"
  control: "6px"
  overlay: "8px"
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
    backgroundColor: "{colors.page-white}"
    textColor: "{colors.ink-body}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "0 1rem"
    height: "2.25rem"
  button-primary:
    backgroundColor: "{colors.bookcloth}"
    textColor: "{colors.on-bookcloth}"
    rounded: "{rounded.control}"
    padding: "0 1rem"
    height: "2.25rem"
  button-primary-hover:
    backgroundColor: "{colors.bookcloth-light}"
    textColor: "{colors.on-bookcloth}"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.ink-body}"
    rounded: "{rounded.control}"
    padding: "0 1rem"
    height: "2.25rem"
  button-danger:
    backgroundColor: "{colors.page-white}"
    textColor: "{colors.danger-red}"
    rounded: "{rounded.control}"
    padding: "0 1rem"
    height: "2.25rem"
  button-sm:
    padding: "0 0.75rem"
    height: "2rem"
  button-masthead-primary:
    backgroundColor: "{colors.on-bookcloth}"
    textColor: "{colors.bookcloth}"
    rounded: "{rounded.control}"
    padding: "0 1rem"
    height: "2.25rem"
  input:
    backgroundColor: "{colors.page-white}"
    textColor: "{colors.ink-body}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "0.25rem 0.75rem"
    height: "2.25rem"
  chip:
    backgroundColor: "transparent"
    textColor: "{colors.bookcloth}"
    typography: "{typography.caption}"
    rounded: "{rounded.tag}"
    padding: "0 0.5rem"
  chip-corrected:
    backgroundColor: "transparent"
    textColor: "{colors.rubric}"
    rounded: "{rounded.tag}"
    padding: "0 0.5rem"
  masthead:
    backgroundColor: "{colors.bookcloth}"
    textColor: "{colors.on-bookcloth}"
    padding: "1.5rem 2rem 1rem"
    height: "6.5rem"
  rail:
    backgroundColor: "{colors.bookcloth}"
    textColor: "{colors.on-bookcloth}"
    width: "14.5rem"
  nav-item:
    backgroundColor: "transparent"
    textColor: "{colors.on-bookcloth}"
    rounded: "{rounded.control}"
    padding: "0 0.75rem"
    height: "2.5rem"
  nav-item-active:
    backgroundColor: "{colors.bookcloth-light}"
    textColor: "{colors.on-bookcloth}"
  siglum:
    textColor: "{colors.rubric}"
    typography: "{typography.siglum}"
  edition-page:
    backgroundColor: "{colors.page-white}"
    textColor: "{colors.ink-bright}"
    typography: "{typography.reading}"
    padding: "1.5rem 2rem 3rem 5rem"
    width: "44rem"
  source-desk:
    backgroundColor: "{colors.desk-grey}"
    padding: "1.5rem"
---

# Design System: Scriptorium

## Overview

**Creative North Star: "The Critical Edition"**

The AI's text is one witness. Every screen is set like a scholarly edition: a white page carrying the reading text in Garamond, the other models' readings and the researcher's corrections set beneath it as an apparatus criticus, each witness named by a red siglum. The shell around the page is bookcloth: a deep blue rail and masthead, the binding the edition sits in. Controls are quiet, in a humanist sans, so the text keeps the authority. The source (a scan or a recording) rests on a grey desk beside the page, an object on a table rather than a panel in an app.

Density is that of a working edition, not a dashboard: generous measure for reading text, compact controls, hairline rules for structure. Sections open with a black rule, the way an edition opens its apparatus, and are never boxed. The world refuses two things by name: the AI-product arrangement of glowing cards and chat bubbles, and the parchment costume the name "Scriptorium" invites. There is no paper texture, no aged tint, no faux manuscript ornament.

The same world holds from a 640px window snapped beside another to a 2560px monitor, and in light and dark themes that follow Windows by default.

**Key Characteristics:**
- White page ground, bookcloth blue (#1d2b4f) shell, black text.
- Rubric red reserved for sigla, marginal line numbers and the researcher's corrections.
- Pale blue tint for selection and highlighted lemmata.
- EB Garamond for everything read; Source Sans 3 for everything operated.
- Sections divided by rules, never cards; shadows only on objects lifted off the desk and on overlays.
- Lucide line icons at a 1.5px stroke, always after or beside words.

## Colors

A restrained scholar's palette: white page, black ink, one bookcloth blue, and one rubric red whose rarity carries meaning.

### Primary
- **Bookcloth Blue** (#1d2b4f): the rail, every masthead, primary buttons, the play button, chips' text and outline. Owns the shell; on the page it appears only as action. Hover lifts to **Bookcloth Light** (#2a3a63), also the active nav item's ground; **Bookcloth Line** (#34446d) draws the rules inside the shell. In dark mode the shell deepens (dark-bookcloth) and actions on the page turn to a pale periwinkle (dark-accent) so they stay legible.

### Secondary
- **Rubric** (#b3261e; dark-rubric #ec8478 in dark): sigla, marginal line numbers, the corrected-chip and the researcher's corrected lemmata and readings. Nothing else on the page is red in this hue.

### Tertiary
- **Lemma Tint** (#dce3f2; dark-lemma-tint in dark): text selection, lemmata marked as having a variant, a lit line number, the "added" side of a diff, and the audio segment now playing.

### Neutral
- **Page White** (#ffffff): the page, the edition paper and button/input grounds. Dark: dark-page and dark-paper.
- **Pane Grey** (#f5f6f8): the quiet secondary surface.
- **Desk Grey** (#ececec; dark-desk in dark): only behind a source (scan or player), so it reads as an object on a table.
- **Section Rule** (#111111; dark-section-rule in dark): the 1px rule that opens a section and the apparatus.
- **Hairline / Strong / Hover** (#e2e3e7, #c6c9d1, #9aa0ad): pane dividers, control borders, and the border a control takes on hover.
- **Ink** (bright #0f1115 for headings and reading text, body #1b1e24, secondary #464b55, muted #5f636d).
- **Status**: ok-green (#2e6b47), warn-umber (#8a5300), danger-red (#a8201a), each used as text plus a translucent wash or border, never as a filled block.

### Named Rules
**The Rubric Rule.** Rubric red marks only sigla, marginal line numbers and the researcher's corrections. If something red is neither a witness's name, a line reference nor a correction, it is wrong.

**The Bookcloth Rule.** Blue owns the binding (rail, mastheads) and the actions. It is not a decorative fill for sections on the page.

**The Desk Rule.** Desk grey sits only behind a source. The edition page and every form sit on white.

## Typography

**Text Font:** EB Garamond Variable (with Garamond, Times New Roman for Arabic script, serif)
**Interface Font:** Source Sans 3 Variable (with Segoe UI Variable, Segoe UI, system-ui)
**Mono Font:** ui-monospace, Cascadia Mono, Consolas (prompt text only)

**Character:** an old-style book face for reading, titling and the edition's apparatus, paired with a humanist sans that stays out of its way for every control. The pairing says "edition first, tool second".

### Hierarchy
- **Wordmark** (Garamond 500, 1.875rem, line-height 1; 1.5rem at ≤820px): the product name in the rail, bottom-aligned on the masthead line.
- **Headline** (Garamond 500, 2rem, 1.2; 1.625rem at ≤820px): every masthead title, including the document title in review.
- **Title** (Garamond 500, 1.375rem): section headings under their rule. **Subtitle** (Garamond 500, 1.125rem) below that; task names in New job are set at 1.25rem in Garamond.
- **Reading** (Garamond 400, 1.3125rem, 1.65, oldstyle figures, max 44rem): the edition's text and the correction editor. Rises to 1.5rem in windows ≥1680px.
- **Apparatus** (Garamond 400, 1.0625rem, 1.6, oldstyle figures): apparatus entries, set as number, lemma ] reading, siglum.
- **Running head** (Garamond all-small-caps, 1.125rem, letter-spacing 0.08em): the edition page's running head (file · page, reading text of A) and the sigla key's heading.
- **Siglum** (Garamond 600 all-small-caps, 1.2em of its context, letter-spacing 0.04em, rubric): a witness letter, or "corr." for the researcher. Larger (1.6rem) in a pane's bar.
- **Body** (Source Sans 3 400, 0.9375rem, 1.45): all interface text, buttons (600), navigation (1rem in the rail).
- **Label** (Source Sans 3 600, 0.8125rem): field labels, small buttons, metadata. **Caption** (600, 0.75rem): chips and counts.
- Marginal and apparatus line numbers use lining tabular figures at 0.8em, weight 600, so they align in their column.

### Named Rules
**The Two Faces Rule.** Garamond carries reading text, titles, the wordmark, small-caps running heads and sigla. Source Sans 3 carries every control. A button, field, tab or label in Garamond is a defect; a document text in the sans is a defect.

**The Mono Quarantine Rule.** Monospace appears only in the prompt editor, because prompt text is written like code. Nowhere else.

## Layout

The app is a fixed shell: a 14.5rem bookcloth rail at left and a scrolling main area. The rail's wordmark block is exactly the masthead height (6.5rem), so the rail's bottom rule and every masthead's bottom rule run as one line across the window. The rail's foot holds the running background job with a thin progress bar.

Each screen opens with a masthead (bookcloth band, bottom-aligned title and actions, padding 1.5rem 2rem 1rem) followed by a **sheet**: a single column, max 64rem, padding 1.5rem 2rem 3rem, sections stacked with a 2rem gap. Sections open with their rule and 1rem of space above their content.

Review is the exception: below the masthead, a scores strip, then two panes split 0.9fr (source on the desk) to 1.1fr (the edition page). The edition page is indented 5rem on the left so line numbers and sigla sit in a true margin, with the text grid pulling each line back 3.5rem to place its number on the first baseline.

New job ends in a sticky, window-wide footer carrying the estimate and Start; its inner content aligns with the sheet's 64rem measure.

Spacing follows a 4px-based scale (0.25, 0.5, 0.75, 1, 1.5, 2, 3rem). Controls are 2rem (small), 2.25rem (default) and 2.5rem (large) tall.

**Responsive range:** 640px (the minimum window) to 2560px.
- **≤820px**: the rail becomes a top bar (wordmark plus wrapped nav, running job hidden), the masthead drops its fixed height, headline shrinks to 1.625rem, sheet and pane padding tighten to 1rem.
- **≤960px**: review stacks the source above the edition (source capped at 40vh, min 16rem).
- **≥1680px**: reading text rises from 1.3125rem to 1.5rem.
- Component-local breakpoints refine single grids (Jobs at 1100px, Prompts at 1000px, New job's task tiles at 760px, Settings at 700px).

## Elevation & Depth

The page is flat. Structure comes from rules and grounds, not from lifting surfaces. Shadows exist only for things that are physically "on top": a source resting on the desk, an overlay, and the sticky footer that content passes beneath.

### Shadow Vocabulary
- **Flyout** (`box-shadow: 0 18px 48px -16px rgba(15, 20, 35, 0.35)`; dark `0 24px 64px -16px rgba(0, 0, 0, 0.7)`): the scan page and the audio player strip on the desk, the close dialog, the toast.
- **Lifted edge** (`box-shadow: 0 -10px 24px -18px rgba(15, 20, 35, 0.45)`): the top of New job's sticky footer, with a strong hairline, so content reads as passing under it rather than cut.

### Named Rules
**The No Shadowed Cards Rule.** Sections never get a shadow, a fill or a box. If a section needs separating, it gets the section rule.

## Shapes

Corners are gently rounded on controls (6px for buttons, inputs, nav items, task tiles and audio lines), slightly more on overlays (8px), near-square on chips (4px) and on inline highlights (2px on lemmata and line numbers, 3px on an apparatus entry's wash). Fully round only where the form is round: the play button, the scrubber thumb, the rail's count badge.

Lines carry the structure: 1px everywhere, black for the section and apparatus rules, hairline grey for pane dividers and control borders, bookcloth-line inside the shell.

Icons come from Lucide (`@lucide/svelte`) through `Icon.svelte`, at an absolute 1.5px stroke in the current colour, play and pause filled: back, chevron, close, play, pause, plus, zoom in and out. No hand-drawn icons. They are decorative; the control carries the accessible name.

## Components

### Buttons
Plain and firm: a sans label in weight 600, never an icon alone except where the meaning is universal (zoom, close, play).
- **Shape:** gently rounded (6px), 2.25rem tall, 0 1rem padding; small is 2rem tall with 0.75rem padding.
- **Default:** white ground, strong hairline border, body ink. Hover darkens the border to the hover hairline.
- **Primary:** bookcloth ground and border, white text; hover lifts to bookcloth light.
- **Ghost:** transparent; hover takes the bookcloth wash (6% blue).
- **Danger:** danger-red text with a danger border.
- **In a masthead:** non-primary buttons turn transparent with a 42% white border; primary inverts to white with bookcloth text.
- **Disabled:** 50% opacity. **Focus:** 2px bookcloth-light outline offset 2px (white in the rail).
- Transitions: background and border colour, 120ms ease-out; removed under reduced motion.

### Chips
- **Style:** transparent ground, 1px outline in 35% bookcloth, bookcloth text, caption type, 4px corners.
- **Variants:** warn (umber), ok (green, outlined in its own colour), danger (red), corrected (rubric). Chips never fill.

### Sections
- **Corner Style:** none.
- **Background:** transparent.
- **Shadow Strategy:** none (see Elevation).
- **Border:** a single 1px section rule on top.
- **Internal Padding:** 1rem above content, 0.75rem between rows.

### Inputs / Fields
- **Style:** white ground, strong hairline border, 6px corners, 2.25rem tall, padding 0.25rem 0.75rem (textareas 0.75rem, line-height 1.55). Label above in label type, secondary ink.
- **Hover:** border to the hover hairline. **Focus:** the global 2px outline.
- In a masthead, a select keeps its white ground with a white border.

### Navigation
- **Rail:** bookcloth, 14.5rem; Garamond wordmark in a block as tall as the masthead; nav items in the sans at 1rem, 2.5rem tall, 78% white at rest, full white on hover with bookcloth-light ground; the active item is bold on bookcloth light. A white pill counts jobs.
- **Narrow (≤820px):** the rail becomes a top bar; nav items wrap in a row at 2.25rem.

### Masthead
The bookcloth band at the top of every screen: back link (muted white, underline on hover) above a Garamond headline, actions at right, all aligned to the bottom edge; 6.5rem tall so its rule meets the rail's.

### Edition Page (signature)
The source sits on the desk beside a white page. The page carries:
- a **running head** in small caps (file · page at left, "reading text" with its siglum at right);
- the **reading text** in Garamond with **marginal line numbers in rubric**, printed every fifth line and on every line with a variant, on the line's first baseline;
- lemmata with a variant marked in lemma tint; a corrected lemma set in rubric;
- the **apparatus criticus** below a black rule: line number in the margin, lemma, a bracket, then each reading followed by its siglum, readings separated by colons; corrections carry the "corr." siglum;
- the **sigla key** below a hairline, each witness's siglum hung in the margin beside its name, metadata and its controls.

**Signature interaction:** hovering or focusing an apparatus entry lights its lemma (bookcloth chip ground with a thin outline) and its line number (tint), and hovering the lemma lights the entry (bookcloth wash); focusing an entry scrolls the lemma into view.

### Audio Player
On the desk, the player is an authored white strip with the flyout shadow: a round bookcloth play button, tabular time, a 4px scrubber track filled in bookcloth with a white-ringed thumb. Below, transcript lines as rows with a muted timestamp; hover takes the wash, the segment now playing takes the lemma tint.

### Task Tiles
New job's three task choices are selectable tiles: strong hairline border, 6px corners, a Garamond task name; selected takes the bookcloth border, an inset 1px bookcloth ring and the wash. They are controls, not sections.

## Do's and Don'ts

### Do:
- **Do** open every section with the 1px section rule and nothing else around it.
- **Do** keep rubric red (#b3261e / #ec8478) for sigla, marginal line numbers and corrections only.
- **Do** use lemma tint (#dce3f2) for selection, marked lemmata and the playing segment.
- **Do** set anything read (documents, titles, running heads, sigla) in EB Garamond and every control in Source Sans 3.
- **Do** put sources (scans, players) on the desk grey, with the flyout shadow, and keep the edition on white.
- **Do** keep the masthead and the rail's wordmark block at the same 6.5rem so their rules meet.
- **Do** put words before icons; use Lucide icons through `Icon.svelte`.
- **Do** design for 640px to 2560px, with the shell collapsing at 820px, review stacking at 960px and reading text rising at 1680px.

### Don't:
- **Don't** box sections in cards, fill them, or give them shadows.
- **Don't** add parchment, paper texture, aged tints or manuscript ornament.
- **Don't** use glowing cards, gradients or chat-bubble styling.
- **Don't** use red for decoration, emphasis or status; errors use danger red as text, not rubric.
- **Don't** use monospace outside the prompt editor.
- **Don't** use glyph characters (arrows, crosses, triangles) as icons.
- **Don't** hard-code visible text; every string comes from the translation files.

## Open Questions

Left by the finish review for the user, not shipped:
- A collation table in Jobs, comparing set-ups across pages.
- An interrupted job with Resume in the rail's foot.
- A capture of a correction set in rubric with the "corr." sigla row, to confirm that state visually.
