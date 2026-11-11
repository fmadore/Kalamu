# 2026-11-11 · The review's fifth round: what was left of #146, #148 and #149

## Context

The [review of 9 November](2026-11-09-review.md) left the keyboard and accessibility (#146),
the shell (#148) and the interface's navigation and smaller screens (#149) in three issues.
Four rounds took most of them up: the [second round](2026-11-10-second-round.md), the
[shell](2026-11-10-shell.md), the [fourth round](2026-11-10-fourth-round.md) and the
[offline engines](2026-11-10-offline-robustness.md). The issues' boxes were never ticked, so
this round first read each bullet and acceptance box against the code as it is on `main`
today, then did what was left that could be done and proved here (Windows ARM64, the interface
in Chromium and WebKit on its sample data). No Rust changed.

## The audit

*Done* means the code does it now, with where; *partly* says what is missing. The decision
entries were read as claims and each was checked in the code.

### #146 Keyboard and accessibility

| Item | State | Evidence |
| --- | --- | --- |
| Readings far from their word in the tab order; focus into them from the keyboard, back on close; roving tabindex over the marks | Done | `MarkedText.svelte:80-94` (one tab stop, ← →), `ReviewView.svelte:262-300` (`choose` with `keyboard`, `focusReadings`, `closeReadings`), the readings inside the text pane after their words |
| An analysis's names located by hover, renamed or removed by right click only | Done | `AnalysisReview.svelte:334-356` (one tab stop, ← →, focus marks it in the source, Enter or the menu key opens `ItemMenu`), focus kept after a change `:173-195` |
| A zoomed scan pans by drag only; panes with no `tabindex`, unscrollable in WebKit | Done | `SourcePane.svelte:299-308` (the page image a tab stop, arrows move it), `keys.ts:96-122` `scrollableByKeys`, used by `ReviewFrame.svelte:250,266` |
| The sidebar's job menu needs a right click or the menu key | Done | `SidebarJobs.svelte:140-144`, a ⋯ button reached by Tab |
| Page Up and Page Down turn pages: decide | Done | They turn pages except in a field or on a pane that scrolls and has the focus: `ReviewFrame.svelte:141-142`, `keys.ts:136` `pageKeysStay` |
| Control boundaries under 3:1 | Done | `--control-line` `app.css:49-50,163-164`, used at `:306,420`; DESIGN.md's *Control line* |
| `--text-faint` carrying real text | Done | Left only on decoration: `AnalysisReview.svelte:474` (the `; ` between names), `MarkedText.svelte:149` (line numbers, `aria-hidden`), `:203`, `ReviewFrame.svelte:306` (the crumb's slash, `aria-hidden`); disabled menu items `JobMenu.svelte:198` muted |
| The toast's live region inserted with its text | Done | `+page.svelte`: `role="alert"` and `role="status"` always mounted, empty |
| Errors shown as a four-second toast | Done | A failure stays until dismissed (close button or Escape), in the `alert` region; this round gives the focus back once it is dismissed |
| The result of testing a key has no live role | Done | `SettingsView.svelte:206` |
| Acceptance: a text page reviewed, corrected and marked reviewed without a pointer, on Windows and in WebKit | Done in both engines | `e2e/keyboard.spec.ts`, first test, in Chromium and WebKit; a real Mac still to try (*To check*) |
| Acceptance: a name located, renamed and removed from the keyboard | Done | `e2e/keyboard.spec.ts`, second test, *Undo* included |
| Acceptance: input and button boundaries at 3:1 in both themes, DESIGN.md updated | Done | As above; by eye on a real screen still to do |
| Acceptance: an error stays readable until dismissed and is announced | Done here | `e2e/keyboard.spec.ts`, third test (in the `alert` region after five seconds); what VoiceOver and Narrator say is to check |

### #148 The app's shell

| Item | State | Evidence |
| --- | --- | --- |
| A start-up failure leaves no window and no message | Done for what setup opens | `setup.rs:35,172,188` sort the cause, `:78-90` `fail`; `start_failure.rs` (English and French, `rfd`); `StoreError::Newer` for a newer database. A failure before setup (Tauri's own `build`, a webview that cannot be made) is still only logged: `lib.rs:226-228` |
| An unreadable `settings.json` silently becomes the defaults | Done | `settings.rs:69-106` moves it to `settings.json.bad` (or `.2.bad`…); `+page.svelte` says it once as a failure |
| No update notice for the installer and the disk image | Done | `catalog.json:4` `latest_version`, `crates/catalog/src/release.rs:102`, `commands/settings.rs:39-40` (nothing for a Store copy: `platform::updated_by_store`, `windows.rs:106`, `macos.rs:240`), the line in `Sidebar.svelte` and Settings › About; privacy policy updated |
| A signed Tauri updater weighed in a decision entry | Done in this entry | See *Decisions* |
| `key_status` on the window's thread; a failed read cached as "no key" | Done | `commands/keys.rs:13` async; `secrets.rs:127-149` keeps only a key or `NoEntry` |
| Exporting a pipeline or workflow overwrites `name.md` | Done | `commands/mod.rs:197-205` through `kalamu_prompts::markdown_beside` |
| `KALAMU_FFMPEG_DIR` and `KALAMU_LLAMA_DIR` in release builds | Done | `crates/media/src/ffmpeg.rs:59-61`, `crates/offline/src/llama.rs:195-197`, both behind `cfg!(debug_assertions)` |
| Tray, taskbar and notification computed in the webview | Done | `activity.rs` (subscribed in `setup.rs:43-44`); the interface sends only the words (`set_shell_labels`) |
| Source-level failures not kept | Done | `crates/engine/src/plan.rs:143`, tested in `crates/engine/src/tests/sources.rs:54` |
| `strip = true` leaves `panic.log` without symbols on macOS | Decided, not changed | `Cargo.toml:73`; [shell](2026-11-10-shell.md) keeps it until the macOS build's size is measured |
| Acceptance: an unreadable database shows a message on both systems and exits | Built, not tried | Needs the app on a PC and a Mac (*To check*) |
| Acceptance: an older build shows the notice with a link; Store builds never | Done in the interface | `e2e/keyboard.spec.ts` (the line, its link, hidden for that version across a reload); the Store side is Rust, to try on the packages |
| Acceptance: tray and notification right with the window hidden for the whole job | Built, not tried | Needs the app (*To check*) |
| Acceptance: privacy policy updated if what is fetched changes | Done | `docs/privacy.md:204`: the same catalogue file, nothing new fetched |

### #149 Interface

| Item | State | Evidence |
| --- | --- | --- |
| No history; `QueueTrail`, `lastSearch` and `DocumentView`'s `from` patch around it | Done | `history.ts`, `state.svelte.ts:245-279` (`go`, `back`, `returnTo`); Alt ←, ⌘ [ and the mouse's back button in `+page.svelte`; the review's Back button `ReviewFrame.svelte:161-169`. `DocumentView`'s `from` is gone; `QueueTrail` and `lastSearch` stay as the state of the list and the search that Back comes back to (the queue's banner also steps through it) |
| Ctrl F on Jobs, Documents, Dictionary and Glossary | Done | `+page.svelte:97-104` (`data-find`), the fields in `AllJobs`, `DocumentTable`, `DictionaryView`, `GlossaryView`, `ProjectSearch` |
| Remembered: Jobs' filters and sort, the document table's sort and tag, a project's pane, each source's rotation | Done | `AllJobs.svelte:31-38`, `DocumentTable.svelte:53-67`, `ProjectWorkspace.svelte:63`, `SourcePane.svelte:63-73` |
| A shortcut overview in the Glossary | Done | `glossary.ts:94` `keyboard(m, mac)`, `GlossaryView.svelte:19`; tested for both systems in `glossary.test.ts:52-62`; French through `fr.ts` |
| A drop target that shows | Done | `state.svelte.ts:235` `takeFiles`, used by `NewJob` and `ProjectWorkspace`; the dashed frame in `+page.svelte`. A real drag still to try |
| Undo for removing a name and a document from a collection | Done | `AnalysisReview.svelte:166-169`, `DocumentTable.svelte:137` |
| Renaming, linking and deleting several in one command each | Done | `commands/review.rs:189` `rename_listed`, `commands/projects.rs:372` `link_jobs`, `commands/jobs.rs:389` `delete_jobs` |
| The job page's facts strip | Done | `JobFacts.svelte:60-73`, the error rates in a row of their own with the sentence once; per file the sentence is a title (`ScoreFoot.svelte:53`) |
| A recording's transcript shown twice | Done | [fourth round](2026-11-10-fourth-round.md): the turns under the player, the passage heard marked in the text (`heard.ts`) |
| A project's Results column and *To check*'s fingerprint | Done | `DocumentTable.svelte:347-373` (a button named by the sentence); the fingerprint gone |
| At 760 px and narrower, check down to 640 × 560 | Done | `e2e/keyboard.spec.ts`: Jobs, a review, a project, New job and Settings at 640 × 560, the list one row above the work and nothing scrolling sideways |
| Dark tokens written twice | Done | `app.css:140-149`, one block on `[data-theme='dark']` |
| Four translation keys nothing reads | Done | Three are gone; `task.youtubeHint` is read as `m.task[`${t}Hint`]` (`TaskPicker.svelte:29`) |
| Acceptance: Back from a review returns to the search, the queue, the document or the job, with its state | Done | `e2e/keyboard.spec.ts`: the search with its words and matches, the list with *Include reviewed* still ticked, the document; the job through the crumb (`returnTo`) |
| Acceptance: each remembered choice survives a restart | Done here | `e2e/keyboard.spec.ts` (Jobs' filter and sort, a project's pane, after a reload); rotation and the document table's sort by reading only |
| Acceptance: the shortcut overview for Windows and a Mac, English and French | Done | `glossary.test.ts`; `webkit.spec.ts` opens every screen in French |
| Acceptance: DESIGN.md records any change to a layout rule | Done | No rule changed; the narrowest size checked is now written under *760px and narrower* |

## Decisions

- **The keyboard paths are proved with the keyboard.** `e2e/keyboard.spec.ts` runs, in
  Chromium (WebView2's engine) and WebKit (macOS's): a text page reviewed through its readings
  (Tab to the differences, ← → between them, Enter into the readings, Escape back to the words,
  F to the next, a reading used, Ctrl or ⌘ Enter); an analysis's name reached by Tab and marked
  in the source, renamed and removed from its menu, then put back with *Undo*; a failure that
  stays; Back from a review to the search, the list of what to check and the document; Ctrl F
  on Jobs, a project's dictionary and the Glossary; the remembered filters, sort and pane after
  a reload; a window of 640 × 560; the release line. Each was found already working, except
  the two below.
- **A failure dismissed gives the focus back.** Dismissing the failure message from the
  keyboard left the focus on nothing, since its button goes with it. The focus now goes back to
  what had it before it went to the message, when that is still there (`+page.svelte`,
  `dismissFailure`). A confirmation's *Undo* is left as it is: the action places the focus
  itself (the name put back).
- **Hiding the release line gives the focus to Glossary**, the last of the sidebar's tools
  above it, for the same reason.
- **The sample data keeps a renamed name.** The preview's `rename_listed` gave back the
  corrections without changing anything, and `save_corrections` did nothing, so the rename and
  *Undo* could not be seen or tested. Both now change the sample answers as the store would
  (`mockJobs.ts`, `renameListed`; `mock.ts`, `keepCorrection`). Development builds only.
- **No updater.** The notice asks nothing new of any server: it reads `latest_version` from the
  catalogue the app already fetches, and Store copies never show it. Tauri's signed updater was
  weighed and left: it would fetch a manifest from GitHub at each launch (a new request, and a
  line in the privacy policy), needs a signing key pair and its secret in the release workflow,
  replaces the installed files itself (which the installer and the disk image do well enough by
  hand), and has to be kept out of the Microsoft Store and Mac App Store builds, which must not
  update themselves. To revisit if copies installed from GitHub are seen to stay on old versions.
- **A failure before setup is not given a message yet.** Tauri makes the window and its webview
  inside its event loop, so a missing or broken WebView2 panics there (`panic.log`) rather than
  coming back as `build`'s error, and `build`'s own failures (the event loop) are rare. A
  message there would need the start-up words and the logs folder before the log exists, and
  could only be tried on a PC without WebView2. Left (see *Left*).

## What was run

On Windows ARM64, in this worktree:

- `npm run lint`, `npm run check` (845 files, no error or warning), `npm test` (41 files, 222
  tests), `npx prettier --check .`: passed.
- `KALAMU_E2E_PORT=4431 npx playwright test`: 97 passed, 4 skipped (the Mac-only ones), 3 failed
  in WebKit, all waiting five seconds for a page while every worker started at once and other
  checkouts were building: `interactions.spec.ts`'s shortcuts and large-job tests (unchanged by
  this round) and `keyboard.spec.ts`'s Ctrl F. Run again alone, all three passed; the Ctrl F
  test now gives its field fifteen seconds. Every test of `keyboard.spec.ts` passed in both
  engines when run on its own.
- No Rust changed, so no Cargo command was run.

## Left

- **#148**: a message for a launch that fails before setup (above); `strip = "debuginfo"`,
  waiting for the macOS build's size.
- **Tab on a Mac.** WebKit, like Safari, moves Tab between fields only unless the system's
  keyboard navigation is on; buttons (the differences, an analysis's names, the review's Back)
  are then reached with Option Tab. F and Shift F reach the differences either way. The
  Glossary's Keyboard section could say so; not done, as it is to be seen on a real Mac first.
- The document table's sort and a source's rotation are remembered (read in the code) but not
  checked by a test.

## To check

- **On a real Mac**, in the app: the text page's path and an analysis's names with Tab, with
  and without *Keyboard navigation* in System Settings; ⌘ [ in a review; the release line's
  absence in the Mac App Store build.
- **With VoiceOver and Narrator**: a failure read out at once and a confirmation politely; the
  readings and the name menu as dialogs and menus; the focus given back after dismissing.
- **In the app on Windows and a Mac**: what the [shell](2026-11-10-shell.md) and the
  [fourth round](2026-11-10-fourth-round.md) list (the start-up message, the tray with the
  window hidden for a whole job, the release line in the Store packages), still not tried.
- A real drag of files over New job and a project.

## Consequences

- AGENTS.md, the specification and the privacy policy are unchanged: nothing new is sent or
  kept, and no behaviour the specification describes changed.
- [DESIGN.md](../../DESIGN.md) says where the focus goes after a failure is dismissed and after
  the release line is hidden, and the narrowest window checked.
- [development.md](../development.md) lists what `e2e/keyboard.spec.ts` covers.
