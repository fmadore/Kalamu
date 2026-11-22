//! A region of a page read again by a model on request (#84): the box cut from the original at
//! full resolution by `media`, sent as a text extraction of that picture (the preset chosen,
//! frozen as a job's would be, or the default one; the job's languages and line breaks; the
//! provider's retries), its answer cached under a key of its own and offered as a proposal,
//! through the steps every request from the review goes through ([`crate::asked`]); the
//! picture is cut before it is recorded, so a stop meanwhile sends nothing. Only the
//! researcher's acceptance writes it: as the text of a region they drew, or in place of the
//! words an engine's region holds in the page's text, in the same transaction as the record
//! that says a model read it. The page's own reading is never changed.
//!
//! No job is made: a reading belongs to the source reviewed, and its cost to that source's job,
//! whatever the job's task, so an analysis's or a table of contents' review can read a region
//! of its pages too. A refusal billed by the provider is charged to the page's piece, as the
//! passages' are.

use kalamu_core::fingerprint::sha256_hex;
use kalamu_core::region_reading as reading;
use kalamu_core::{
    ApiKey, CacheKey, Job, JobSettings, JobStore, PieceRef, PieceState, PieceStatus, PreparedInput,
    ProposalStatus, ReadingEffect, ReadingTarget, RegionAsk, RegionError, RegionReading,
    RegionReadingError, Setup, SetupRequest, SourceFile, StopSignal, StoreError, Task, Timestamp,
};
use kalamu_media::{Crop, MediaError, OpenSource, RegionRefusal};

use crate::asked::{Asked, Call, PassageEstimate, Waiting};
use crate::control::lock;
use crate::{Engine, EngineError};

mod decide;

impl Asked for RegionReading {
    type Effect = ReadingEffect;

    fn load(store: &dyn JobStore, id: &str) -> Result<Self, StoreError> {
        store.region_reading(id)
    }

    fn record(
        &self,
        store: &dyn JobStore,
        effect: Option<&ReadingEffect>,
    ) -> Result<(), StoreError> {
        store.put_region_reading(self, effect)
    }

    /// None: whatever the model read is stored, as a text extraction's is.
    fn refusal(&self, _answer: &str) -> Option<String> {
        None
    }

    /// Offered when the model found text, failed when not.
    fn answered(&mut self, text: &str) {
        self.answered_at = Some(Timestamp::now());
        if text.trim().is_empty() {
            self.status = ProposalStatus::Failed;
            self.error = Some(RegionReadingError::Empty.to_string());
        } else {
            self.status = ProposalStatus::Answered;
            self.answer = Some(text.to_string());
        }
    }
}

/// A region is read as a page's text is.
const TASK: Task = Task::TextExtraction;

/// A reading's request, before its picture is cut.
struct Planned {
    job: Job,
    source: SourceFile,
    setup: Setup,
    settings: JobSettings,
    /// The picture formats the model reads.
    accepted: Vec<&'static str>,
    /// The piece a refusal the provider billed is charged to.
    charged: PieceState,
}

/// A store's "not found" for a region, as the region's own error.
fn missing(e: StoreError) -> EngineError {
    match e {
        StoreError::NotFound => EngineError::Region(RegionError::NotFound),
        e => e.into(),
    }
}

fn refused(e: RegionReadingError) -> EngineError {
    EngineError::RegionReading(e)
}

impl Engine {
    /// Checks a request against the store as it is now, and freezes the text extraction preset
    /// for the model chosen, as a job would: a box on a page the source has, a target still
    /// there (the region drawn, or the words of the text where they were), a model that reads a
    /// page's text from a picture.
    fn plan_reading(&self, ask: &RegionAsk) -> Result<Planned, EngineError> {
        kalamu_core::regions::check(&ask.as_draft()).map_err(EngineError::Region)?;
        let (job, source) = self.source_in(ask.job_id, ask.source_id)?;
        crate::regions::drawn::on_page(&source, ask.page, ask.half)?;
        let charged = self.charged_piece(&job, ask)?;
        let catalog = self.current_catalog();
        let model = catalog.model(&ask.provider, &ask.model);
        if let Some(model) = model {
            if model.retired {
                return Err(EngineError::RetiredModel(model.name.clone()));
            }
            if !model.does(TASK) {
                return Err(refused(RegionReadingError::CannotRead(model.name.clone())));
            }
        }
        let provider = self.provider(&ask.provider)?;
        let accepted = crate::piece::accepted_inputs(provider.as_ref(), model, TASK);
        if accepted.is_empty() {
            return Err(refused(RegionReadingError::CannotRead(ask.model.clone())));
        }
        if !accepted
            .iter()
            .any(|m| matches!(*m, "image/png" | "image/jpeg"))
        {
            return Err(MediaError::Region(RegionRefusal::NoImages).into());
        }
        let settings = JobSettings {
            languages: job.settings.languages.clone(),
            line_breaks: job.settings.line_breaks,
            ..JobSettings::default()
        };
        let request = SetupRequest {
            provider: ask.provider.clone(),
            model: ask.model.clone(),
            preset_id: ask.preset_id.clone(),
            label: None,
            options: Default::default(),
            merge: None,
        };
        let setup = self
            .freeze_setups(&catalog, TASK, &settings, std::slice::from_ref(&request))?
            .into_iter()
            .next()
            .ok_or(EngineError::EmptyJob)?;
        Ok(Planned {
            job,
            source,
            setup,
            settings,
            accepted,
            charged,
        })
    }

    /// Whether the target is still there, and the piece a billed refusal is charged to: the
    /// page's piece whose text the reading would change, else a piece of the page under any
    /// set-up, else the page itself under the first set-up (an analysis read the whole
    /// document).
    fn charged_piece(&self, job: &Job, ask: &RegionAsk) -> Result<PieceState, EngineError> {
        let page = PieceRef::Page {
            number: ask.page,
            half: ask.half,
        };
        match &ask.target {
            ReadingTarget::Region => {
                let region = self.store.region(&ask.region_id).map_err(missing)?;
                let same = region.job_id == ask.job_id
                    && region.source_id == ask.source_id
                    && region.on(ask.page, ask.half);
                if !same {
                    return Err(EngineError::Region(RegionError::NotFound));
                }
            }
            ReadingTarget::Text {
                setup_id,
                index,
                start,
                end,
                was,
            } => {
                let state = self.finished_piece(ask.job_id, *setup_id, ask.source_id, *index)?;
                let shown = self.shown_text(&state)?;
                if state.piece != page || reading::locate(&shown, *start, *end, was).is_none() {
                    return Err(refused(RegionReadingError::Changed));
                }
                return Ok(state);
            }
            ReadingTarget::Nowhere => {}
        }
        for setup in &job.setups {
            let pieces = self.store.source_pieces(job.id, setup.id, ask.source_id)?;
            if let Some(state) = pieces.into_iter().find(|s| s.piece == page) {
                return Ok(state);
            }
        }
        let setup_id = job
            .setups
            .first()
            .map(|s| s.id)
            .ok_or(EngineError::EmptyJob)?;
        Ok(PieceState {
            job_id: job.id,
            setup_id,
            source_id: ask.source_id,
            index: ask.page.saturating_sub(1),
            piece: page,
            status: PieceStatus::Pending,
            cache_key: None,
            error: None,
            attempts: 0,
            updated_at: Timestamp::now(),
        })
    }

    /// The region cut from the original, off the async thread, its file checked as a job checks
    /// it: changed or gone, it is refused as a job's piece would be. The last picture cut is
    /// kept, so the request after an estimate sends it without cutting it again.
    async fn cut_region(
        &self,
        planned: &Planned,
        ask: &RegionAsk,
        stop: &StopSignal,
    ) -> Result<PreparedInput, EngineError> {
        let source = &planned.source;
        let corners = [ask.x, ask.y, ask.width, ask.height].map(f32::to_bits);
        let label = format!(
            "{}|{}|{}|{:?}|{corners:?}|{}",
            source.id,
            source.sha256,
            ask.page,
            ask.half,
            planned.accepted.join(",")
        );
        if let Some((cut, input)) = lock(&self.region_cut).as_ref()
            && *cut == label
        {
            return Ok(input.clone());
        }
        let crop = Crop {
            x: ask.x,
            y: ask.y,
            width: ask.width,
            height: ask.height,
        };
        let piece = PieceRef::Page {
            number: ask.page,
            half: ask.half,
        };
        let stem = crate::source_stem(&source.file_name, "page");
        let (source, accepted, stop) = (source.clone(), planned.accepted.clone(), stop.clone());
        let ffmpeg = self.config.ffmpeg.clone();
        let input = tokio::task::spawn_blocking(move || {
            kalamu_media::check_unchanged(&source)?;
            let opened = OpenSource::open_with_stop(&source, ffmpeg.as_ref(), &stop)?;
            opened.prepare_region(&piece, crop, &accepted, &stem)
        })
        .await
        .map_err(|e| EngineError::Output(e.to_string()))??;
        *lock(&self.region_cut) = Some((label, input.clone()));
        Ok(input)
    }

    /// What reading the region would cost: the picture cut, priced as the share of a page of a
    /// text extraction it covers (its picture, text and thinking; the instructions whole), as a
    /// job's estimate prices a page, and as a whole page for a model priced by the page; nothing
    /// when the answer is stored already, nothing to pay on this computer, none for a model
    /// without a price.
    pub async fn region_reading_estimate(
        &self,
        ask: &RegionAsk,
    ) -> Result<PassageEstimate, EngineError> {
        let planned = self.plan_reading(ask)?;
        let input = self
            .cut_region(&planned, ask, &StopSignal::default())
            .await?;
        let key = CacheKey::region(
            &planned.setup,
            &planned.settings.languages,
            &sha256_hex(&input.bytes),
        );
        let prompt = &planned.setup.prompt;
        self.estimated(&ask.provider, &key, |catalog| {
            let m = catalog.model(&ask.provider, &ask.model)?;
            let chars = prompt.text.len() + prompt.user_prompt.len();
            let work = kalamu_catalog::Workload {
                pages: 1,
                documents: 1,
                prompt_chars: chars,
                region: Some(f64::from(ask.width * ask.height)),
                reasoning: planned.setup.options.reasoning.as_deref(),
                ..Default::default()
            };
            catalog.estimate(m, TASK, &work).usd
        })
    }

    /// Asks the chosen model to read a region again, and gives back the reading: recorded
    /// before the call and after it; taken from the cache when the same picture was read the
    /// same way before; failed when the model found no text. Ends early with the reading
    /// cancelled when [`Engine::cancel_region_reading`] is called meanwhile; an answer that
    /// arrives after is still stored, so asking again costs nothing.
    pub async fn ask_region_reading(&self, ask: RegionAsk) -> Result<RegionReading, EngineError> {
        let planned = self.plan_reading(&ask)?;
        let api_key = self.ready_key(
            &ask.provider,
            &planned.setup.model,
            &planned.settings.languages,
        )?;
        let waiting = Waiting::new(&self.region_readings, &ask.id);
        self.read_region(ask, planned, api_key, &waiting.stop).await
    }

    async fn read_region(
        &self,
        ask: RegionAsk,
        planned: Planned,
        api_key: ApiKey,
        stop: &StopSignal,
    ) -> Result<RegionReading, EngineError> {
        let mut proposal = RegionReading {
            ask: ask.clone(),
            status: ProposalStatus::Asked,
            prompt: planned.setup.prompt.clone(),
            request: Vec::new(),
            cache_key: String::new(),
            answer: None,
            cost_usd: None,
            cached: false,
            error: None,
            accepted: None,
            asked_at: Timestamp::now(),
            answered_at: None,
            decided_at: None,
        };
        let input = match self.cut_region(&planned, &ask, stop).await {
            Ok(input) if !stop.is_stopped() => input,
            // Stopped while the page was opened or cut: nothing was sent.
            Ok(_) | Err(EngineError::Media(MediaError::Stopped)) => {
                proposal.status = ProposalStatus::Cancelled;
                proposal.answered_at = Some(Timestamp::now());
                self.store.put_region_reading(&proposal, None)?;
                return Ok(proposal);
            }
            Err(e) => return Err(e),
        };
        let key = CacheKey::region(
            &planned.setup,
            &planned.settings.languages,
            &sha256_hex(&input.bytes),
        );
        proposal.cache_key = key.0.clone();
        if self.asked_or_cached(&mut proposal, &key)? {
            return Ok(proposal);
        }
        let Planned {
            mut job,
            setup,
            settings,
            charged,
            ..
        } = planned;
        job.task = TASK;
        job.settings = settings;
        let call = Call {
            job,
            setup,
            charged,
            input,
            key,
            api_key,
        };
        self.call_raced(proposal, call, stop).await
    }

    /// Stops waiting for a region's reading; nothing when it is not waiting.
    pub fn cancel_region_reading(&self, id: &str) {
        Waiting::cancel(&self.region_readings, id);
    }
}
