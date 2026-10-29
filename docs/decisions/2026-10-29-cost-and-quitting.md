# 2026-10-29 · Work neither lost nor paid twice

## Context

The [checkpoint of the same day](2026-10-29-checkpoint.md) found five ways work already paid
for could be lost and paid again, or a job could do what the researcher had not asked:

- Quitting waited five seconds, then exited under cloud calls still running: a long recording
  piece, a YouTube video or a whole document read by an analysis was dropped after the provider
  had done (and billed) the work, and sent again on the next launch.
- A retry's wait (up to the provider's own `retry-after`) ignored pause and cancel, so a paused
  job sent one more paid attempt, and kept running past the quit's five seconds.
- An engine error during a run returned at once, leaving the calls already started running
  unwatched; the job read as stopped while they went on, and a resume could send them again.
- At launch a job with a batch at the provider was started in full: besides collecting it sent
  every page not yet sent, overriding a pause the researcher had made.
- The cost shown was the current run's, from zero, kept only in memory: after a resume or a
  restart, what the job had cost before disappeared.

## Decisions

- **Quitting waits for the pieces in flight.** Pausing already lets them finish and be stored;
  quitting now waits for the running jobs to stop, up to eleven minutes (the HTTP client gives up
  on a call after ten, so past that something is stuck). The close dialog says how many pieces
  are still coming back, from the progress the window already has, and offers "Quit now";
  quitting a second time, from the dialog or the notification area, exits at once.
- **A pause or cancel ends a retry's wait**, checked every tenth of a second; the piece goes back
  to the queue unpaid, to be read on resuming.
- **An engine error drains the run first**: no new piece starts, the job's stop signal is set so
  no piece retries, and the calls already started finish and are stored before the error is
  reported.
- **At launch, batches are only collected** (`Engine::collect_waiting`): a batch recorded but
  never confirmed goes back to the queue, the waiting ones are collected and the documents they
  complete written, and nothing is sent; a table of contents is not combined. The job then keeps
  the status it had (paused, or interrupted by the close) unless the answers finished it.
- **The cost shown is the job's.** `JobStore::spent` sums the results the job's own pieces
  stored (`done`, not `cached`, which another job paid for); a run starts from it and says so at
  once, and the job list carries it (`JobSummary::cost_usd`), so the job page shows what the job
  has cost over all its runs, running or not.

## Consequences

- The jobs interrupted by the last close are listed at launch even when they have batches to
  collect, since they no longer resume by themselves.
- Tested in the engine (a pause during a thirty-second retry wait ends the job at once with no
  second call; at relaunch a batch job collects its waiting batch, sends nothing and stays
  interrupted, then sends the rest when resumed) and in the store (a job's spending counts its
  own pieces, not those it reused). The quit wait is in the app crate and was not tested
  automatically; the close dialog's waiting state was type-checked but not seen, as the browser
  preview cannot send a window-close event.
- Still to do from the checkpoint: answers billed but failing their fields are not counted in
  the cost, as a failed call carries no usage.
