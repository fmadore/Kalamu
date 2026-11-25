# 2026-11-25 · The model list checked before it is trusted, and read by the versions installed

## Context

Every installed copy of Kalamu downloads the model list (`crates/catalog/catalog.json`) from
`main` at launch and when Settings asks, so each merge reaches them all at once
([#279](https://github.com/fmadore/Kalamu/issues/279), from the
[review of 25 November](2026-11-25-review-safety-nets.md)). Until now a copy kept whatever
parsed: no limit on the download's size; no check of sense, so a list with no model, a negative
price or a successor naming nothing would be used; a bad list, once cached, won at every launch
until one with a later date came, even with refreshing turned off; lists were compared by the
text of their date, so a correction made the same day never arrived; and the link to a
provider's key page, opened in the browser, was held to the hosts the bundled list names only by
a test of the bundled list, a downloaded one needing no more than `https://`. The providers'
addresses are constants in the code, so a list cannot send requests elsewhere, but it can change
prices, estimates and what New job offers without anyone seeing.

The other direction was untested: `Task`, `ModelStatus`, `Tier` and `LanguageHints` are read
strictly, so one value a build does not know makes it refuse the whole list, said nowhere but
in the logs at the info level. The tests read an old list with today's build, never today's list
with the builds people have.

## Decision

- **Every list is checked after it is read, the bundled one and a downloaded one alike**
  (`Catalog::from_json`, `crates/catalog/src/checks.rs`). Refused: a list larger than 1 MB
  (`MAX_BYTES`; the list is about 65 KB), and the download stopped there; a `schema_version`
  higher than the build knows, before anything else is read; no provider, no model or no
  language; `updated` not written `YYYY-MM-DD`; a provider or model listed twice; a model of an
  unknown provider, without a task, a type of file (`type/subtype`) or a language, or with
  `"languages"` a word other than `any`; a price negative or not a number, a batch discount
  outside 0 to 1, a limit of 0, reasoning without levels; a model the default for a task it
  cannot do, or the default while replaced or retired; a successor that is not another model of
  the same provider, or a line of successors that loops; an alias naming no model or taking a
  model's own id; an embedding model of an unknown provider, said to run on this computer when
  its provider does not (or the reverse), without a length of vector, or with a negative price;
  no model offered for one of the three tasks providers know (reading pages, transcribing,
  analysing), so New job always has a default. Each refusal has its test.
- **Links held to the hosts the bundled list names.** A provider's `key_url` is empty for a
  provider on this computer, and otherwise a plain `https://` address (no user, port, space or
  backslash) on one of the hosts of the bundled list's key pages. It is the only address a list
  carries; the link to a release is made from a version number checked to be digits
  ([decision](2026-11-10-shell.md)). A new provider's host arrives with the release that adds
  its module, which is needed anyway.
- **A revision number, compared instead of the date.** `revision` counts every change to the
  file, raised by one each time and never reset; of two lists the higher wins, so a correction
  the same day arrives. `updated` stays, the day prices were checked, shown in Settings. A list
  without the number (one cached by an older build) reads as revision 0, below the bundled list,
  which starts at 1. A daily revision (`updated` then `revision`) was weighed: it keeps comparing
  the date's text, which a mistyped day would break.
- **A list that fails is dropped, and said once in Settings.** A download that fails its checks
  is not kept; the list in use stays (the bundled one, or a list downloaded before that passed);
  the failure is logged as a warning, and Settings shows under *Models*, below the list's date,
  that the latest list failed Kalamu's checks and is not used, until a list passes. Pressed by
  hand, *Check for an updated model list* also says so in its message. A cached list that fails
  (one kept by an older build) is deleted at launch and the bundled one used, so a bad list no
  longer wins at every launch. A list for a newer schema is not called bad: it needs a newer
  Kalamu, and is set aside without a notice.
- **Unknown values read leniently where that is safe.** Only names are read so, never values of
  the wrong kind, which still refuse the list:
  - a `status` (`ModelStatus`) or a `tier` (`Tier`) the build does not know hides that model: a
    status may mean the model must not be sent, and a tier places it in New job;
  - a `language_hints` (`LanguageHints`) it does not know hides the model: read as another, it
    would send languages in a way the model's API refuses;
  - a task (`Task`) it does not know is left out of `tasks`, `recommended_for` and
    `provider_default_for`: the model still does the tasks the build knows. A model left with
    none is hidden.
  What pointed at a hidden model is forgotten: a successor the build hides leaves the older
  model offered, as it was before the successor existed, and an alias for one is dropped. The
  checks then run on what is left, so hiding can never leave a task without a model unnoticed.
- **CI reads today's list with every release in people's hands**
  (`.github/workflows/catalog.yml`, `scripts/catalog-compat.sh`), on a pull request or a push
  to `main` that changes `catalog.json`, and by hand. For each tag `vX.Y.Z` from 1.0.0 on, found
  in the repository so a new release joins on the day it is tagged, it takes that tag's
  `kalamu-core` and `kalamu-catalog` and their lock, builds them alone with today's toolchain
  (no app, native library or whisper environment), and reads the list with the tag's own
  `Catalog::from_json`. 1.6.0, withdrawn from the Mac App Store, is still on the releases page,
  so it is included; the Store copies (1.10.0 and later) update themselves but are covered by
  the same tags. The 0.x test pre-releases are not. A pull request that changes the list must
  also raise its revision, checked against the base branch.

## Consequences

- Locally, on 25 November 2026, the list with `revision` added was read by every release from
  1.0.0 to 1.10.1, all of its 39 models: older builds ignore the new field, as they ignore every
  field they do not know. Given the list with one model's tier set to an unknown word, 1.10.1's
  reader refused it whole, which is what the job exists to catch (only 1.10.1 was tried). The fifteen builds took 39 minutes on the
  maintainer's ARM64 laptop under other load; the workflow runs only when the list, the script
  or the workflow changes, with an hour's limit, and its first runs will say what it costs on
  GitHub's runners.
- Builds from before this change still compare dates: a same-day correction reaches them only
  with the next day's list, and a list that fails the new checks is still used by them. The
  cross-version job holds that each of them at least reads every new list.
- A rule added to the checks binds every list from then on, read by every build that has the
  rule. The cross-version job builds each tag's own checks with its reader, so a list a released
  build would refuse fails before merging, not after.
- The notice in Settings is one line in one place, not a message that interrupts, and is held
  in memory only (a `catalog_refused` command reads it): a list that passes clears it, and a bad
  list on `main` is said again at each launch that downloads it, until it is corrected.
- `docs/development.md` says how to change the list: raise `revision` by one in every pull
  request that changes the file, and let the cross-version job read it.
