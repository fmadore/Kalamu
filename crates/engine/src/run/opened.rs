//! A source of a running job: opened and planned ([`Engine::prepare`]), its units sent from
//! the job's one pool of calls ([`Engine::send_unit`]), and finished once none of them is left
//! running ([`Engine::finish_source`]): pages combined, parts brought together, files written.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use kalamu_core::{Job, ProgressEvent, Setup, SetupId, SourceFile, SourceId, StopSignal, Task};
use kalamu_media::OpenSource;

use super::{stop_for, stop_requested};
use crate::aligned::Lines;
use crate::batch::{BatchOutcome, Gathered};
use crate::cited::Quotable;
use crate::group::Unit;
use crate::output::OutputNames;
use crate::piece::{Input, Outcome};
use crate::plan::Existing;
use crate::{Control, Engine, EngineError};

/// What every source of a run reads.
pub(super) struct Shared<'j> {
    pub(super) job: &'j Job,
    pub(super) control: &'j Mutex<Control>,
    pub(super) stop: &'j StopSignal,
    pub(super) existing: &'j Existing,
    /// Set-ups sent as batches.
    pub(super) batched: &'j HashSet<SetupId>,
    pub(super) spent: &'j Arc<Mutex<f64>>,
    pub(super) names: &'j OutputNames,
}

/// What the sources of a run add up to.
#[derive(Default)]
pub(super) struct Tally {
    pub(super) failed: bool,
    /// A piece stopped part-way is pending again; the job must not read as finished.
    pub(super) interrupted: bool,
    /// Sources that could not be opened or planned in this run: nothing of theirs to combine.
    pub(super) unread: HashSet<SourceId>,
    pub(super) gathered: HashMap<SetupId, Gathered>,
    pub(super) batch: BatchOutcome,
}

/// A source opened and planned, whose units are being sent.
pub(super) struct Opened<'j> {
    pub(super) source: &'j SourceFile,
    /// Its place in the job's sources, which tells the answers of two sources apart.
    pub(super) index: usize,
    opened: Arc<OpenSource>,
    lines: Option<Arc<Lines>>,
    quotable: Option<Arc<Quotable>>,
    queue: std::vec::IntoIter<Unit>,
    /// Its units sent and not yet back.
    pub(super) running: usize,
}

impl Opened<'_> {
    pub(super) fn has_next(&self) -> bool {
        !self.queue.as_slice().is_empty()
    }

    /// Nothing of it is running, and nothing is left to send or nothing more will be.
    pub(super) fn settled(&self, stopped: bool) -> bool {
        self.running == 0 && (stopped || !self.has_next())
    }
}

/// What preparing a source came to.
pub(super) enum Prepared<'j> {
    Ready(Opened<'j>),
    /// It could not be opened or planned: reported, and the job goes on without it.
    Unread,
    /// Paused or cancelled while it was planned: the job ends as asked.
    Stopped,
}

impl Engine {
    /// Opens and plans a source, records its pieces, and gathers those sent as batches.
    pub(super) async fn prepare<'j>(
        &self,
        run: &Shared<'_>,
        tally: &mut Tally,
        index: usize,
        source: &'j SourceFile,
    ) -> Result<Prepared<'j>, EngineError> {
        let (job, existing) = (run.job, run.existing);
        // A file changed since it was added is refused: its pieces would be cached under the
        // fingerprint of what it held then.
        // Opened with the job's stop signal, so a pause ends FFmpeg's work on a recording.
        let open = || {
            let (source, stop) = (source.clone(), run.stop.clone());
            let ffmpeg = self.config.ffmpeg.clone();
            tokio::task::spawn_blocking(move || {
                kalamu_media::check_unchanged(&source)?;
                OpenSource::open_with_stop(&source, ffmpeg.as_ref(), &stop)
            })
        };
        let mut opened = open()
            .await
            .map_err(|e| EngineError::Output(e.to_string()))?;
        // A frozen text whose file was lost or cut short is written again from the store when
        // the result it was frozen from still gives the same text, then opened again.
        // A text that cannot be written again is reported as unread, as before.
        if opened.is_err() && matches!(self.rewrite_frozen(source), Ok(true)) {
            opened = open()
                .await
                .map_err(|e| EngineError::Output(e.to_string()))?;
        }
        let unread = |tally: &mut Tally, error: &str| {
            tally.failed = true;
            tally.unread.insert(source.id);
            self.source_failed(job, source, existing, error)
                .map(|()| Prepared::Unread)
        };
        let opened = match opened {
            Ok(o) => Arc::new(o),
            Err(e) => return unread(tally, &e.to_string()),
        };
        // A translation of a transcript that kept its times reads its lines.
        let lines = match self.timed_lines(job, source) {
            Ok(lines) => lines,
            Err(error) => return unread(tally, &error),
        };
        // An analysis whose fields cite the text reads its turns.
        let quotable = match self.quotable(job, source) {
            Ok(quotable) => quotable,
            Err(error) => return unread(tally, &error),
        };
        let pieces = match self
            .plan(job, source, &opened, existing, lines.as_deref())
            .await?
        {
            Ok(pieces) => pieces,
            // Paused or cancelled while a recording's pauses were found: nothing is
            // planned, and the job ends as asked.
            Err(_) if stop_requested(run.control).is_some() => return Ok(Prepared::Stopped),
            Err(error) => return unread(tally, &error),
        };
        self.emit(ProgressEvent::SourcePrepared {
            job_id: job.id,
            source_id: source.id,
            pieces: pieces.len() as u32,
        });

        let work = self.record_pieces_quoting(
            job,
            source,
            &pieces,
            existing,
            lines.as_deref(),
            quotable.as_deref(),
        )?;

        // Timed lines are read back against the lines each passage sent, and quotations
        // looked for in the text, which a batch's collection does not do: they are sent at
        // once, in economy mode too: the rule the estimate prices by ([`crate::batched`]).
        let (to_batch, work): (Vec<_>, Vec<_>) = work.into_iter().partition(|(setup, _, _)| {
            crate::batched::in_batch(
                run.batched.contains(&setup.id),
                job.task,
                source,
                crate::cited::cites(setup),
            )
        });
        for (setup, state, key) in to_batch {
            if stop_requested(run.control).is_some() {
                break;
            }
            let outcome = self
                .gather(
                    job,
                    &source.file_name,
                    &setup,
                    state,
                    &key,
                    &opened,
                    &mut tally.gathered,
                    run.stop,
                )
                .await?;
            if let Some(message) = &outcome.fatal {
                stop_for(run.control, job.id, message);
            }
            tally.batch.merge(outcome);
        }

        let queue = self.units(job, source, work).into_iter();
        Ok(Prepared::Ready(Opened {
            source,
            index,
            opened,
            lines,
            quotable,
            queue,
            running: 0,
        }))
    }

    /// Takes the next unit of `from` and returns its call, to run on its own task.
    pub(super) fn send_unit(
        self: &Arc<Self>,
        run: &Shared<'_>,
        from: &mut Opened<'_>,
    ) -> Option<impl Future<Output = Result<Outcome, EngineError>> + Send + 'static> {
        let unit = from.queue.next()?;
        from.running += 1;
        let engine = self.clone();
        let (job, source, opened, spent, stop, lines, quotable) = (
            run.job.clone(),
            from.source.clone(),
            from.opened.clone(),
            run.spent.clone(),
            run.stop.clone(),
            from.lines.clone(),
            from.quotable.clone(),
        );
        Some(async move {
            match unit {
                Unit::One((setup, state, key)) => {
                    let stem = crate::source_stem(&source.file_name, "page");
                    let passage = lines.as_ref().and_then(|l| l.passage(&state.piece, &stem));
                    let cite = quotable
                        .as_ref()
                        .and_then(|q| q.call(&setup, &state.piece, &stem));
                    let input = match (passage, cite) {
                        (Some(passage), _) => Input::Lines(passage),
                        (None, Some(cite)) => Input::Cited(cite),
                        (None, None) => Input::Open(opened),
                    };
                    engine
                        .process_piece(&job, &source, setup.step(), state, key, input, spent, &stop)
                        .await
                }
                Unit::Group(setup, items) => {
                    engine
                        .process_group(&job, &source, &setup, items, opened, spent, &stop)
                        .await
                }
            }
        })
    }

    /// Finishes a source none of whose units is running: combines its pages and brings its
    /// parts together unless the job `stopped`, then writes the outputs of every set-up whose
    /// pieces of it are all done.
    pub(super) async fn finish_source(
        &self,
        run: &Shared<'_>,
        tally: &mut Tally,
        done: &Opened<'_>,
        stopped: bool,
    ) -> Result<(), EngineError> {
        let (job, source) = (run.job, done.source);
        let settle = |outcome: Outcome, tally: &mut Tally| match outcome {
            Outcome::Finished => {}
            Outcome::Interrupted => tally.interrupted = true,
            Outcome::Failed => tally.failed = true,
            Outcome::Fatal(message) => {
                stop_for(run.control, job.id, &message);
                tally.failed = true;
            }
        };
        // A table of contents: combine the pages of each set-up that read them all.
        if job.task == Task::Contents && !stopped {
            for setup in job.setups.iter().filter(|s| !run.batched.contains(&s.id)) {
                let outcome = self
                    .merge_pages(job, source, setup, run.spent, run.stop)
                    .await?;
                settle(outcome, tally);
            }
        }

        // A long text read in parts: bring the parts of each set-up together.
        if let Some(text) = done.quotable.as_ref().filter(|_| !stopped) {
            for setup in job.setups.iter().filter(|s| text.parts(s).is_some()) {
                let how = crate::windows::BringTogether { source, setup };
                let outcome = self
                    .second_step(job, source, setup, run.spent, run.stop, &how)
                    .await?;
                settle(outcome, tally);
            }
        }

        // Write outputs for every set-up whose pieces of this source are all done.
        let setups: Vec<&Setup> = job.setups.iter().collect();
        tally.failed |= self.export_finished(job, run.names, source.id, &setups)?;
        Ok(())
    }
}
