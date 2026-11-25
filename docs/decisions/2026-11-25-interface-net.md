# 2026-11-25 · The interface's net for unexpected failures, and its commands held to the shell's

## Context

The review of 25 November (#280, [safety nets](2026-11-25-review-safety-nets.md)) found that
expected failures are said well (`Action`, `describeError`, messages that stay) but unexpected
ones not at all: no boundary, no `error` or `unhandledrejection` listener, no way for the
interface to write to the log. A screen that threw as it was drawn stayed blank, with nothing
said and nothing logged, so a bug report had nothing to quote. A failure at start-up showed a
chip with no way to try again. And the bridge's 195 command names and their arguments matched
the shell's by care alone: Rust's tests tie the handler and the capability to `COMMANDS`, but
nothing tied `src/lib/tauri.ts` to them.

## Decision

- **One boundary around the screens** (`<svelte:boundary>` in `+page.svelte`, Svelte 5.58),
  inside `main`, so the sidebar beside it keeps working. Its failed state (`ScreenFailed`)
  says that the screen stopped working, that everything saved is kept and running jobs carry
  on (true: corrections are stored as they are saved, and jobs run in the shell), and offers
  *Go back* when there is a screen to go back to and *Reload the window*. Opening any other
  screen, from the sidebar or a shortcut, resets the boundary, so the next screen is drawn
  afresh. One boundary rather than one per view: every view sits in the same place, and a
  view's own parts that fail take the whole view, which is the unit a researcher can leave.
- **Global listeners** (`unexpected.ts`, installed first in `onMount`) send what no code caught
  to one new command, `log_interface_error`, and show one failure toast for a burst: a burst
  ends once ten seconds pass without a failure. The boundary's failures are logged the same way,
  without the toast, the screen saying it in its place. A core error (`AppError`) that nobody
  caught is said in its own words, as an `Action` would have said it. SvelteKit runs a
  boundary's error through its `handleError` and hands the boundary only `{ status: 500,
  message: 'Internal Error' }`, so a screen's failure is sent from `src/hooks.client.ts`, where
  the error is still whole, not from the boundary's `onerror`. Chromium's
  `ResizeObserver loop` notice is not an error and is ignored.
- **What is sent, and what is written.** The interface sends the error's name, its message cut
  to 1,000 characters, its stack cut to 4,000, the kind (`error`, `rejection`, `screen`) and the
  view's name. For a core error it sends only its id, never its detail (a path, a provider's
  words); for anything thrown that is not an error, only its type, since it may be text. The
  shell then cuts it down, whatever it received, and writes one line at the `error` level:
  - the name and the view keep only the characters a name in code holds (64 and 32 at most);
  - the message loses everything between quotation marks (`"…"`, `'…'`, `` `…` ``, `“…”`,
    `‘…’`, `«…»`), which is where engines put values and fragments of what they read
    (`Unexpected token 'L', "Le Gouverneur…" is not valid JSON`); it is put on one line and cut
    to 300 characters. Property names quoted by the engine go too: the stack says where;
  - the stack keeps only lines that are frames, each as its function's name and the file's
    path on the server with line and column (`load _app/immutable/chunks/x.js:12:345`), the
    address it is served from and anything after `?` dropped, twelve frames at most.
  So the log's rule holds: no document text, no keys, nothing typed or answered.
- **Bounded twice.** The interface sends the same failure once a burst and at most twenty
  different ones; the shell writes at most ten a minute and, past them, one line saying how many
  it left out once the minute is over. The limiter is a static in the command's module rather
  than a field of `AppState`: it is the command's own and needs nothing else.
- **Background readings fail into the expected path.** The `void` calls whose callee can reject
  and that ran in the background, without a screen waiting on them, now end in `sayFailure`
  (the failure toast, in the interface language, not repeated while the same one shows):
  `refreshJobs` when progress events stream in, when a job finishes and when a project file is
  opened; `refreshOffline` when New job opens, when the shell says the status changed, and in
  every place of Settings' offline models (its poll included); `refreshKeys` after a key is
  saved, tested or removed. The workflow list's reading of runs fails into its own `Action`.
  The other `void` calls were read and left: their callees catch their own failures.
- **Try again at start-up.** The start-up reading is one function; its failure chip has a *Try
  again* button that runs it again, the listeners it registers registered once.
- **The commands checked** (`src/lib/commands.test.ts`): it reads `COMMANDS` from
  `command_names.rs` and fails when the bridge (`tauri.ts`, `tauriCited.ts`) calls a name the
  shell lacks or leaves one uncalled; it reads each `#[tauri::command]` function's parameters
  (Tauri's own left out, the others camelCased as Tauri names them) and fails when a call
  leaves one out (an `Option` may be) or passes one the command does not take; and it fails
  when the development mock answers a name the shell lacks. Generated wrappers (from `ts-rs`
  or `tauri-specta`) would make the same checks by construction, but would rewrite the bridge
  and its 195 hand-written doc comments and add a dependency; the test settles both for now.
- **Dead code removed**: `keepUnder` (`zoom.ts`), `withPassage` (`passage.ts`) and the
  translation key `settings.clearUnusedConfirm` (replaced by `clearUnusedAsk`). Kept:
  `task.youtubeHint` and `useText.correctHint`, `workflowHint`, `interviewHint`, read by name
  (`m.task[`${t}Hint`]`, `u[`${choice}Hint`]`); and `glossary.keys.passageMenu`, which
  describes a shortcut that exists (the menu key or Shift+F10 on text selected) but is missing
  from the Glossary's list: a row to add, not a key to remove.

## Consequences

- A failure while a screen is drawn leaves the app usable and leaves a line in the log that a
  bug report can quote; the guide's Help page says what to do and where the log is.
- An event handler's error is not caught by the boundary (Svelte catches rendering and effects
  only): it goes to the global listener, which logs it and shows the toast.
- The development mock answers `log_interface_error` by keeping each report on
  `window.__kalamuLoggedErrors`; `?mock&broken=<view>` makes that screen fail as it is drawn
  (development builds only), for `e2e/unexpected.spec.ts`.
- `docs/privacy.md` says the logs record the interface's failures, cut down as above.
- A new command is now held in four places by tests: the handler, `COMMANDS`, the capability and
  the bridge.
