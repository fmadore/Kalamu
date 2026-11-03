# 2026-11-03 · The French interface

## Context

Every visible string went through `src/lib/i18n/en.ts` from the first screen, so that French
and German could be added without touching components. Much of Kalamu's intended audience
works in French: archivists and researchers on francophone West Africa among the first.

## Decisions

- **`fr.ts` beside `en.ts`, typed `Messages`.** A key missing from or added to the French
  catalog fails `npm run check`; a test compares the two catalogs' keys, and which of them are
  text, lists or functions of how many arguments, so a message that changed shape in English
  is noticed in French too.
- **Chosen in Settings, under Appearance and language**: *Same as the system*, English or
  Français, each language named in its own words. *Same as the system* takes the first of the
  system's preferred languages that Kalamu has, else English, so a German speaker who also
  reads French gets French until German exists. The system's languages come from Rust
  (`sys-locale`) rather than the webview, because WebKit on macOS gives an app only the
  languages its bundle declares. The choice is saved with the other settings, in Rust.
- **A change of language draws the screens again.** Components take their text once, when
  drawn; the shell is keyed on the language, so each screen is drawn anew in it. The language
  changes only in Settings, and what the screens share (the view, the jobs, their progress)
  lives outside them, so nothing is lost. The tray menu is pushed again with it.
- **Words of the trade, not calques.** A job is a *traitement*, a workflow a *chaîne de
  traitement*, a pipeline (a job's saved settings) a *configuration*, a set-up a *combinaison*,
  a piece of work (a page or a stretch of a recording) a *segment*, an item of a table of
  contents a *pièce*, as in an archival file. The review is a *relecture*; a model's reading is
  a *leçon*, its letter a *sigle*, as an edition names the readings of its witnesses; ground
  truth is the *vérité terrain* of the French OCR and HTR literature. *Prompt* and *token* stay
  as French researchers use them. Product, provider and model names stay as they are.
- **French typography**: guillemets with no-break spaces, a no-break space before a colon and a
  narrow one before `;`, `?` and `!`, and `45 %`.
- **Dates, numbers and costs** were already written through `Intl` in the interface language
  (`formatDateTime`, `formatUsd`, `formatPercent`); the two places that were not, a dictionary
  request's date and the review's zoom, now are. Costs stay in US dollars, as providers bill
  them: `0,12 $US`.
- **The default prompts' names and descriptions are translated; the prompts are not.** The core
  writes the defaults in English, which the models follow best, and the answers' language is
  chosen in New job. The French names are applied where the interface lists the prompts, to
  defaults left as shipped; the user's own prompts, and defaults they edited, stay as written.
- **Halves of a page in a list of pages**: the engine reads and writes `8 (left)`; the
  interface shows and accepts `8 (gauche)`, translating at the edge so the core keeps one
  notation.

## Not translated yet

- The macOS application menu, which is Tauri's default (Edit, Window…), and its Quit item.
- The models' descriptions and the catalogue's notes, which come from the model catalogue, a
  data file refreshed from the repository; it would need a field per language.
- The files Kalamu writes: headings in Word exports, the Markdown record of a pipeline or a
  workflow, `provenance.json`. They are research records shared with others, kept in English.
- Technical detail after an error's sentence (a provider's own message, a path), and the
  error the engine stores with a page that failed, shown in a job's files, the review and the
  queue of what to check: it is kept as English text, not as an id the interface could word.
- The tray menu's labels in the moment between the app starting and the interface loading.

## Consequences

Adding German is `de.ts`, one line in `LOCALES` and `CATALOGS`, and the default prompts'
names in it. The Store listing can now have a French version, with French screenshots.
