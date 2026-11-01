# 2026-11-01 · Workflows, first increment: extraction → analysis → export

## Context

[#49](https://github.com/fmadore/Kalamu/issues/49) asks for workflows the researcher builds and
saves: extract text, optionally stop to review it, analyse the text as corrected, export, each
step with its own model and prompt. Its first increment is extraction → analysis → export on
the foundations of #47 (snapshots) and #57 (projects); review stages for the dictionary (#48)
come later. iwac-ai-pipelines chains its steps through files on disk, read by the next script;
zmo-ai-pipelines runs one notebook per task; Live Translation has no chaining. Nothing freezes
a step's input or waits for a review, so the design is Kalamu's own; what is ported is the
order of the steps and the choice of model and prompt per step.

A change across `core`, `store`, `prompts`, `engine` and `app` is an architecture checkpoint
(AGENTS.md). Its findings are at the end.

## Decisions

- **A step is a pipeline.** A workflow is an ordered list of saved pipelines, each with what
  it reads (the files, or an earlier step's text) and whether to stop for a review after it.
  Existing pipelines become steps as they are, a pipeline on its own is a one-step workflow
  (`Workflow::from_pipeline`), and the export reuses each pipeline's record, prompts in full.
  An ordered list, not a graph: a step reads one input, from a step before it, so no cycle can
  form; `check_steps` in `core` refuses a step reading itself or a later one, the answers of an
  analysis or a table of contents, or text with a task that reads scans. Several inputs (the
  text plus an accepted dictionary) wait for #48.
- **Definitions are versioned; runs are frozen.** Saving a workflow raises its version. A run
  keeps a copy of every step with its prompts rendered and frozen when it starts (`RunStep`,
  holding `Setup`s as a job does), its sources, and the version, so saving the workflow again
  or editing a preset never changes a run already started. Run definitions live in the job
  database (`runs`, `run_steps`), saved definitions as JSON files beside pipelines.
- **Each step is an ordinary job.** The engine creates a step's job when its input is ready,
  and a step that reads text reads a snapshot of the upstream step's first set-up, taken at
  that moment (#47). Caching, resuming, batches, retries, progress, files, the review and
  provenance are those of any job; nothing bypasses the engine. Running a workflow again reuses
  every unchanged answer from the cache, so only the steps whose model, prompt or text changed
  are paid for: "invalidate only dependent work" falls out of the cache keys rather than a
  dependency graph.
- **The engine advances runs itself.** When a job's work ends, the engine looks up the run it
  belongs to and starts the steps whose input is now ready, one advance at a time (a lock keeps
  two steps finishing together from starting a step twice). The interface never decides that a
  step may start.
- **Going on is the researcher's decision.** A step waits when the step it reads asked for a
  review, or finished with pieces not read; only *Continue* lets it go on, recorded with the
  time and whether some pieces were missing (`Approval`), and written to provenance. A pending
  decision, a paused job or a failed piece is never taken as one. Continuing despite missing
  pieces is explicit and traceable: the snapshot lists them, as it does for *Analyse this text*.
- **Nothing is sent at launch without the researcher.** Work the researcher started (a job run,
  a step continued) starts the next steps; batches collected at launch, and steps whose input
  became ready as the app closed, only make the next step's job, which waits like an
  interrupted job (`resume_runs`). This keeps the launch rule of the
  [29 October checkpoint](2026-10-29-cost-and-quitting.md).
- **A step's job keeps pointing at its job.** `run_steps` holds the job's id without a foreign
  key, as snapshots do: a job deleted leaves the step marked "cannot start" downstream rather
  than starting it again.
- **States are read, not stored.** A step's state (waiting, running, ready for review, some
  pages not read, done, blocked…) and the run's (running, waiting for you, finished, stopped)
  are computed from the jobs' statuses each time (`step_state`, `run_status` in `core`), so
  they cannot drift from the jobs. A job the engine has started counts as running before its
  loop records so.
- **Costs before starting** are those New job gives, for each step that reads the files; a step
  that reads text costs what that text holds, said to be known once the step before has read
  it. Each step says what it sends and to which provider, and a step on this PC followed by one
  that sends its text to the cloud is pointed out, so offline work never silently reaches a
  cloud provider.
- **Projects.** A run started from a project's selection carries the project; the app ties
  each step's job to its documents when the engine reports it started (`StepStarted`), the
  text steps through the snapshot's lineage as *Analyse this text* does. The engine knows
  only the ids.
- **Interface.** Workflows sits under Projects. A workflow is built from saved pipelines, with
  steps moved and removed and what each reads kept pointing at the same step; the run page
  shows each step with its job and asks to continue where a step waits.

## Checkpoint

- **Dependencies still point inwards.** `core` holds the run types, the input check and the
  state rules; `store` persists runs behind `JobStore`; `prompts` holds definitions and their
  file; `engine` orchestrates; `app` wires commands and ties steps to projects. No new
  dependency between internal crates.
- **One job each.** The engine's `create_job` was split into checking, freezing set-ups and
  inserting, so a run freezes its steps at start and inserts each job later with the same
  code; no second way to make a job.
- **Resuming, caching, progress, provenance** cover workflows because each step is a job.
  Provenance gains `workflow`; events gain `StepStarted` and `RunChanged`.
- **Size.** `engine/src/workflow.rs` is about 450 lines and `core/src/workflow.rs` about 420
  with its tests; `store/src/lib.rs` reaches about 750, as accepted on 29 October, its run code
  in `runs.rs`.
- **To watch**: `JobStore` now carries jobs, snapshots and runs. If dictionary decisions (#48)
  also land in the engine's store, split a `RunStore` trait off it. `estimate_cost` lives in
  the app command, so Workflows asks it once per step; moving the estimate into the engine
  would let a run's cost be computed in one place.

## For colleagues new to it (same day)

Reviewed against the researchers the app is for, the first screens asked for too much: two
concepts (pipeline, then workflow) and two screens before anything ran, a *Continue* found only
on the run's own page, and words like "reads the text of step 1". Changed:

- **Then, in New job.** An analysis chosen at the end of New job, by default after a check,
  starts an unsaved two-step workflow: the same engine path, nothing new to learn. The analysis
  runs on the reading provider's model when it has one and a key, in the researcher's usual
  formats; its cost is said to depend on the text.
- **Continue where they are.** The engine answers which run and step a job is (`job_step`); the
  job's page and its review show a banner with *Continue*; the sidebar lists waiting jobs
  first; the notification says what is waited for. The run's page stays for the whole picture.
- **Examples** built with New job's own choice of model, so each uses a key the researcher
  has; **an animated drawing** of the steps, as New job draws a task; **a glossary entry** with
  help marks; **plainer words**, and an input shown as text when there is nothing to choose.

## Consequences

- Left in #49: several inputs per step (with #48's dictionary), project-level steps, an
  export step choosing results across steps, a run's whole estimate in the engine, and
  restarting a run from a step with a changed definition.
- The privacy policy lists what runs and saved workflows keep, with a new effective date; the
  app sends nothing new.
