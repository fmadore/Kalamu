// The one bridge to the Rust side, as in Live Translation: typed commands and events, and a
// graceful failure outside the desktop app (a plain `npm run dev` in a browser). Every type
// comes from `src/lib/bindings`, generated from the Rust definitions by `cargo test`.

import type { AnswerField } from './bindings/AnswerField.js';
import type { AppError } from './bindings/AppError.js';
import type { AppInfo } from './bindings/AppInfo.js';
import type { Catalog } from './bindings/Catalog.js';
import type { ComparisonReport } from './bindings/ComparisonReport.js';
import type { AttachChoice } from './bindings/AttachChoice.js';
import type { Collection } from './bindings/Collection.js';
import type { Correction } from './bindings/Correction.js';
import type { DictionaryView } from './bindings/DictionaryView.js';
import type { Pace } from './bindings/Pace.js';
import type { PassageAsk } from './bindings/PassageAsk.js';
import type { PassageEstimate } from './bindings/PassageEstimate.js';
import type { PassageProposal } from './bindings/PassageProposal.js';
import type { ReadingEffect } from './bindings/ReadingEffect.js';
import type { RegionAsk } from './bindings/RegionAsk.js';
import type { RegionReading } from './bindings/RegionReading.js';
import type { ProposalPlan } from './bindings/ProposalPlan.js';
import type { ProposalsView } from './bindings/ProposalsView.js';
import type { QueueItemKey } from './bindings/QueueItemKey.js';
import type { QueueSelection } from './bindings/QueueSelection.js';
import type { QueueView } from './bindings/QueueView.js';
import type { ResultKey } from './bindings/ResultKey.js';
import type { SampleSpec } from './bindings/SampleSpec.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { DocumentDetail } from './bindings/DocumentDetail.js';
import type { DocumentSources } from './bindings/DocumentSources.js';
import type { Entry } from './bindings/Entry.js';
import type { Evidence } from './bindings/Evidence.js';
import type { Item } from './bindings/Item.js';
import type { SpeakerNames } from './bindings/SpeakerNames.js';
import type { TaskRules } from './bindings/TaskRules.js';
import type { Pipeline } from './bindings/Pipeline.js';
import type { ExportProfile } from './bindings/ExportProfile.js';
import type { ProfileColumn } from './bindings/ProfileColumn.js';
import type { ProfileError } from './bindings/ProfileError.js';
import type { ProfileStarter } from './bindings/ProfileStarter.js';
import type { JobStep } from './bindings/JobStep.js';
import type { RunView } from './bindings/RunView.js';
import type { StartRun } from './bindings/StartRun.js';
import type { Workflow } from './bindings/Workflow.js';
import type { DerivedState } from './bindings/DerivedState.js';
import type { DocumentUnit } from './bindings/DocumentUnit.js';
import type { SeedReport } from './bindings/SeedReport.js';
import type { UnitEdit } from './bindings/UnitEdit.js';
import type { UnitsView } from './bindings/UnitsView.js';
import type { DiffSpan } from './bindings/DiffSpan.js';
import type { EstimateRequest } from './bindings/EstimateRequest.js';
import type { Estimate } from './bindings/Estimate.js';
import type { RestEstimate } from './bindings/RestEstimate.js';
import type { Expanded } from './bindings/Expanded.js';
import type { RowEdit } from './bindings/RowEdit.js';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { ImageDocument } from './bindings/ImageDocument.js';
import type { Job } from './bindings/Job.js';
import type { JobScores } from './bindings/JobScores.js';
import type { JobAgreement } from './bindings/JobAgreement.js';
import type { JobProgress } from './bindings/JobProgress.js';
import type { JobSummary } from './bindings/JobSummary.js';
import type { JobSources } from './bindings/JobSources.js';
import type { KeyStatus } from './bindings/KeyStatus.js';
import type { ListedAdded } from './bindings/ListedAdded.js';
import type { NewJob } from './bindings/NewJob.js';
import type { OfflineStatus } from './bindings/OfflineStatus.js';
import type { PagePreview } from './bindings/PagePreview.js';
import type { PhotoChecks } from './bindings/PhotoChecks.js';
import type { PageRegions } from './bindings/PageRegions.js';
import type { PageHalf } from './bindings/PageHalf.js';
import type { DrawnRegion } from './bindings/DrawnRegion.js';
import type { RegionDraft } from './bindings/RegionDraft.js';
import type { PieceAudio } from './bindings/PieceAudio.js';
import type { PieceView } from './bindings/PieceView.js';
import type { Preset } from './bindings/Preset.js';
import type { Project } from './bindings/Project.js';
import type { SearchRequest } from './bindings/SearchRequest.js';
import type { SearchProgress } from './bindings/SearchProgress.js';
import type { Draft } from './described.js';
import type { SearchView } from './bindings/SearchView.js';
import type { EmbeddingSpace } from './bindings/EmbeddingSpace.js';
import type { MapView } from './bindings/MapView.js';
import type { MeaningProgress } from './bindings/MeaningProgress.js';
import type { MeaningRequest } from './bindings/MeaningRequest.js';
import type { MeaningStatus } from './bindings/MeaningStatus.js';
import type { MeaningView } from './bindings/MeaningView.js';
import type { Neighbour } from './bindings/Neighbour.js';
import type { SpaceChoice } from './bindings/SpaceChoice.js';
import type { ProjectSummary } from './bindings/ProjectSummary.js';
import type { ProjectFileCheck } from './bindings/ProjectFileCheck.js';
import type { ProjectFileOpened } from './bindings/ProjectFileOpened.js';
import type { ProjectFilePlan } from './bindings/ProjectFilePlan.js';
import type { ProjectFileProgress } from './bindings/ProjectFileProgress.js';
import type { ProjectFileSaved } from './bindings/ProjectFileSaved.js';
import type { ProjectView } from './bindings/ProjectView.js';
import type { SourceFile } from './bindings/SourceFile.js';
import type { SourceMatch } from './bindings/SourceMatch.js';
import type { ProgressEvent } from './bindings/ProgressEvent.js';
import type { PrunedResults } from './bindings/PrunedResults.js';
import type { UnusedResults } from './bindings/UnusedResults.js';
import type { Release } from './bindings/Release.js';
import type { Settings } from './bindings/Settings.js';
import type { TagRenamed } from './bindings/TagRenamed.js';
import type { Task } from './bindings/Task.js';
import type { ShellLabels } from './bindings/ShellLabels.js';
import { localisePreset } from './i18n/index.js';

/** The published privacy policy, named wherever the app asks before sending files. */
export const PRIVACY_URL = 'https://fmadore.github.io/Kalamu/privacy';

export const isTauri = (): boolean =>
	typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export type Unlisten = () => void;

/** Listens while a component is on screen. Returns the cleanup for `onMount` or `$effect`, which
 *  also removes a listener registered only after the component has gone. */
export function listenWhileMounted(subscribe: () => Promise<Unlisten>): Unlisten {
	let unlisten: Unlisten | undefined;
	let gone = false;
	void subscribe().then((u) => {
		if (gone) u();
		else unlisten = u;
	});
	return () => {
		gone = true;
		unlisten?.();
	};
}

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauri()) {
		throw { id: 'app.notDesktop', detail: cmd } satisfies AppError;
	}
	const { invoke } = await import('@tauri-apps/api/core');
	return invoke<T>(cmd, args);
}

async function listen<T>(event: string, handler: (payload: T) => void): Promise<Unlisten> {
	if (!isTauri()) return () => {};
	const { listen } = await import('@tauri-apps/api/event');
	return listen<T>(event, (e) => handler(e.payload));
}

export const api = {
	appInfo: () => invoke<AppInfo>('app_info'),
	getSettings: () => invoke<Settings>('get_settings'),
	saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),
	getCatalog: () => invoke<Catalog>('get_catalog'),
	refreshCatalog: () => invoke<Catalog>('refresh_catalog'),
	taskRules: () => invoke<TaskRules[]>('task_rules'),
	/** A release newer than this copy, named by the model list; never in a store's copy. */
	newerRelease: () => invoke<Release | null>('newer_release'),
	/** Deletes the answers no job refers to any more; refused while a job runs. */
	clearUnusedResults: () => invoke<PrunedResults>('clear_unused_results'),
	unusedResults: () => invoke<UnusedResults>('unused_results'),
	keyStatus: () => invoke<KeyStatus[]>('key_status'),
	setApiKey: (provider: string, key: string) => invoke<void>('set_api_key', { provider, key }),
	clearApiKey: (provider: string) => invoke<void>('clear_api_key', { provider }),
	testApiKey: (provider: string) => invoke<string[]>('test_api_key', { provider }),
	offlineStatus: () => invoke<OfflineStatus>('offline_status'),
	downloadModel: (id: string) => invoke<void>('download_model', { id }),
	cancelModelDownload: () => invoke<void>('cancel_model_download'),
	removeModel: (id: string) => invoke<void>('remove_model', { id }),
	// The defaults' names and descriptions in the interface language.
	listPresets: (task?: Task) =>
		invoke<Preset[]>('list_presets', { task: task ?? null }).then((p) => p.map(localisePreset)),
	listPipelines: () => invoke<Pipeline[]>('list_pipelines'),
	savePipeline: (pipeline: Pipeline) => invoke<Pipeline>('save_pipeline', { pipeline }),
	deletePipeline: (id: string) => invoke<void>('delete_pipeline', { id }),
	/** Writes the JSON and its Markdown, and gives where the Markdown went. */
	exportPipeline: (id: string, path: string) => invoke<string>('export_pipeline', { id, path }),
	importPipeline: (path: string) => invoke<Pipeline>('import_pipeline', { path }),
	listWorkflows: () => invoke<Workflow[]>('list_workflows'),
	saveWorkflow: (workflow: Workflow) => invoke<Workflow>('save_workflow', { workflow }),
	duplicateWorkflow: (id: string, name: string) =>
		invoke<Workflow>('duplicate_workflow', { id, name }),
	deleteWorkflow: (id: string) => invoke<void>('delete_workflow', { id }),
	exportWorkflow: (id: string, path: string) => invoke<string>('export_workflow', { id, path }),
	importWorkflow: (path: string) => invoke<Workflow>('import_workflow', { path }),
	startRun: (start: StartRun) => invoke<RunView>('start_run', { start }),
	listRuns: () => invoke<RunView[]>('list_runs'),
	getRun: (id: string) => invoke<RunView>('get_run', { id }),
	jobStep: (id: string) => invoke<JobStep | null>('job_step', { id }),
	approveStep: (id: string, step: number) => invoke<void>('approve_step', { id, step }),
	cancelRun: (id: string) => invoke<void>('cancel_run', { id }),
	deleteRun: (id: string) => invoke<void>('delete_run', { id }),
	savePreset: (preset: Preset) => invoke<Preset>('save_preset', { preset }),
	deletePreset: (id: string) => invoke<void>('delete_preset', { id }),
	duplicatePreset: (id: string, name: string) => invoke<Preset>('duplicate_preset', { id, name }),
	exportPresets: (ids: string[], path: string) => invoke<void>('export_presets', { ids, path }),
	importPresets: (path: string) => invoke<Preset[]>('import_presets', { path }),
	expandPaths: (paths: string[], task: Task) => invoke<Expanded>('expand_paths', { paths, task }),
	providersToConsent: (providers: string[]) =>
		invoke<string[]>('providers_to_consent', { providers }),
	consentProviders: (providers: string[]) => invoke<void>('consent_providers', { providers }),
	foldersToAsk: (paths: string[]) => invoke<string[]>('folders_to_ask', { paths }),
	keepAccess: (paths: string[]) => invoke<void>('keep_access', { paths }),
	inspectFiles: (paths: string[]) => invoke<InspectedFile[]>('inspect_files', { paths }),
	/** Images already inspected, made the pages of one document, in the natural order of their
	 * names; an empty name gives the document its folder's. */
	imageDocument: (name: string, sources: SourceFile[]) =>
		invoke<ImageDocument>('image_document', { name, sources }),
	/** The photographs among `sources`, in this order, checked against each other: the same
	 * page photographed twice, a blurred photograph. `null` when a newer check or
	 * `stopPhotoChecks` stopped it. */
	checkPhotos: (sources: SourceFile[]) => invoke<PhotoChecks | null>('check_photos', { sources }),
	stopPhotoChecks: () => invoke<void>('stop_photo_checks'),
	inspectLinks: (links: string[]) => invoke<InspectedFile[]>('inspect_links', { links }),
	estimateCost: (request: EstimateRequest) => invoke<Estimate>('estimate_cost', { request }),
	createJob: (job: NewJob) => invoke<Job>('create_job', { job }),
	/** The fingerprint of the prompt each set-up would freeze, nothing created. */
	promptFingerprints: (job: NewJob) => invoke<string[]>('prompt_fingerprints', { job }),
	startJob: (id: string) => invoke<void>('start_job', { id }),
	/** After a sample (#215): every page with one of its set-ups, the pages it read from the
	 * cache; and another model on its pages. Each job is made, not started. */
	restEstimate: (job: string, setup: string) =>
		invoke<RestEstimate>('rest_estimate', { job, setup }),
	restOfSample: (job: string, setup: string, name: string) =>
		invoke<Job>('rest_of_sample', { job, setup, name }),
	compareSample: (job: string, provider: string, model: string, name: string) =>
		invoke<Job>('compare_sample', { job, provider, model, name }),
	pauseJob: (id: string) => invoke<void>('pause_job', { id }),
	cancelJob: (id: string) => invoke<void>('cancel_job', { id }),
	retryFailed: (id: string) => invoke<void>('retry_failed', { id }),
	deleteJob: (id: string) => invoke<void>('delete_job', { id }),
	/** Deletes several jobs, all or none. */
	deleteJobs: (ids: string[]) => invoke<void>('delete_jobs', { ids }),
	renameJob: (id: string, name: string) => invoke<void>('rename_job', { id, name }),
	listJobs: () => invoke<JobSummary[]>('list_jobs'),
	/** The names of every job's sources, for finding a job by a file it read. */
	jobSources: () => invoke<JobSources[]>('job_sources'),
	getJob: (id: string) => invoke<Job>('get_job', { id }),
	/** The job's pieces held in batches, and each of `sources` counted under every set-up. */
	jobProgress: (id: string, sources: string[]) =>
		invoke<JobProgress>('job_progress', { id, sources }),
	jobTextOutputs: (id: string) => invoke<string[]>('job_text_outputs', { id }),
	/** Freezes a job's corrected text under a set-up: every source's, or one source's. */
	snapshotJob: (id: string, setup: string, source: string | null = null) =>
		invoke<SourceFile[]>('snapshot_job', { id, setup, source }),
	derivedState: (id: string) => invoke<DerivedState[]>('derived_state', { id }),
	/** A table of contents' document units for one source under one set-up (#50). */
	unitsView: (jobId: string, setupId: string, sourceId: string) =>
		invoke<UnitsView>('units_view', { jobId, setupId, sourceId }),
	seedUnits: (jobId: string, setupId: string, sourceId: string) =>
		invoke<SeedReport>('seed_units', { jobId, setupId, sourceId }),
	saveUnit: (edit: UnitEdit) => invoke<DocumentUnit>('save_unit', { edit }),
	deleteUnit: (id: string) => invoke<void>('delete_unit', { id }),
	joinUnits: (first: string, second: string) =>
		invoke<DocumentUnit>('join_units', { first, second }),
	splitUnit: (id: string, at: number) => invoke<DocumentUnit[]>('split_unit', { id, at }),
	/** The units' pages frozen as PDFs, as sources for a new job. */
	unitSources: (ids: string[]) => invoke<SourceFile[]>('unit_sources', { ids }),
	exportUnits: (ids: string[], folder: string) => invoke<string[]>('export_units', { ids, folder }),
	jobOutputs: (id: string) => invoke<string[]>('job_outputs', { id }),
	openOutput: (id: string, path: string) => invoke<void>('open_output', { id, path }),
	/** Opens the original file of a page the job read. */
	openOriginal: (id: string, path: string) => invoke<void>('open_original', { id, path }),
	sourcePieces: (jobId: string, sourceId: string) =>
		invoke<PieceView[]>('source_pieces', { jobId, sourceId }),
	piecePreview: (jobId: string, sourceId: string, index: number) =>
		invoke<PagePreview>('piece_preview', { jobId, sourceId, index }),
	/** Where the text of a piece sits on it, under one set-up, placed in the text shown. */
	pageRegions: (jobId: string, sourceId: string, setupId: string, index: number) =>
		invoke<PageRegions>('page_regions', { jobId, sourceId, setupId, index }),
	/** The researcher's regions on a page with no text piece of its own (an analysis's or a
	 * table of contents' review), by the page's number and half. */
	drawnRegions: (jobId: string, sourceId: string, page: number, half: PageHalf | null) =>
		invoke<PageRegions>('drawn_regions', { jobId, sourceId, page, half }),
	/** Saves a region drawn over the scan, or the box given an engine's region, or a change
	 * to either; the region as kept. */
	saveRegion: (draft: RegionDraft) => invoke<DrawnRegion>('save_region', { draft }),
	/** Deletes a region drawn or adjusted; the region as it was, for Undo. */
	removeRegion: (id: string) => invoke<DrawnRegion>('remove_region', { id }),
	restoreRegion: (region: DrawnRegion) => invoke<DrawnRegion>('restore_region', { region }),
	/** A page drawn `width` pixels wide, as a small image, for the strip of thumbnails. */
	pieceThumbnail: (jobId: string, sourceId: string, index: number, width: number) =>
		invoke<PagePreview>('piece_thumbnail', { jobId, sourceId, index, width }),
	pieceAudio: (jobId: string, sourceId: string, index: number, convert: boolean) =>
		invoke<PieceAudio>('piece_audio', { jobId, sourceId, index, convert }),
	/** A file `pieceAudio` named, as an address the player streams (Tauri's asset protocol). */
	fileUrl: async (path: string): Promise<string> => {
		const { convertFileSrc } = await import('@tauri-apps/api/core');
		return convertFileSrc(path);
	},
	sourceText: (jobId: string, sourceId: string) =>
		invoke<string>('source_text', { jobId, sourceId }),
	saveCorrection: (correction: Correction) => invoke<void>('save_correction', { correction }),
	/** Makes one change to a page's rows in a records analysis, in its corrected answers. */
	editRows: (jobId: string, setupId: string, sourceId: string, index: number, change: RowEdit) =>
		invoke<void>('edit_rows', { jobId, setupId, sourceId, index, change }),
	/** Stores several corrections, all or none. */
	saveCorrections: (corrections: Correction[]) => invoke<void>('save_corrections', { corrections }),
	/** Renames a listed name in every set-up that found it, or takes it out when `to` is null,
	 * all or none; gives back those set-ups' corrections as they stood, to put back. */
	renameListed: (
		jobId: string,
		sourceId: string,
		index: number,
		key: string,
		from: string,
		to: string | null
	) => invoke<Correction[]>('rename_listed', { jobId, sourceId, index, key, from, to }),
	/** Adds names or choices a model missed to a list answer (one per line or between
	 * semicolons) in every set-up that read the piece, all or none; gives back those set-ups'
	 * corrections as they stood, to put back, and what became of each item. */
	addListed: (jobId: string, sourceId: string, index: number, key: string, items: string[]) =>
		invoke<ListedAdded>('add_listed', { jobId, sourceId, index, key, items }),
	/** What asking a model to correct a passage of the review would cost; refused as asking. */
	passageEstimate: (ask: PassageAsk) => invoke<PassageEstimate>('passage_estimate', { ask }),
	/** Asks a model to correct a passage, through the engine: its proposal, applied by nothing. */
	askPassage: (ask: PassageAsk) => invoke<PassageProposal>('ask_passage', { ask }),
	cancelPassage: (id: string) => invoke<void>('cancel_passage', { id }),
	/** Writes a proposal, or its edit, into the piece's corrections; the correction as it stood. */
	acceptPassage: (id: string, text: string) => invoke<Correction>('accept_passage', { id, text }),
	discardPassage: (id: string) => invoke<void>('discard_passage', { id }),
	undoPassage: (id: string, before: Correction) => invoke<void>('undo_passage', { id, before }),
	/** What reading a region of a page again would cost; refused as asking would be. The engine
	 * cuts the region from the original to know whether it was read before. */
	regionReadingEstimate: (ask: RegionAsk) =>
		invoke<PassageEstimate>('region_reading_estimate', { ask }),
	/** Asks a model to read a region again, through the engine: its reading, applied by nothing. */
	askRegionReading: (ask: RegionAsk) => invoke<RegionReading>('ask_region_reading', { ask }),
	cancelRegionReading: (id: string) => invoke<void>('cancel_region_reading', { id }),
	/** Writes a reading, or its edit, where it goes (the region's text, or the page's text in place
	 * of the region's words); what it replaced, for Undo. */
	acceptRegionReading: (id: string, text: string) =>
		invoke<ReadingEffect>('accept_region_reading', { id, text }),
	discardRegionReading: (id: string) => invoke<void>('discard_region_reading', { id }),
	undoRegionReading: (id: string, before: ReadingEffect) =>
		invoke<void>('undo_region_reading', { id, before }),
	saveSpeakerNames: (jobId: string, names: SpeakerNames) =>
		invoke<void>('save_speaker_names', { jobId, names }),
	exportSource: (jobId: string, setupId: string, sourceId: string) =>
		invoke<string[]>('export_source', { jobId, setupId, sourceId }),
	compareSource: (jobId: string, sourceId: string) =>
		invoke<ComparisonReport>('compare_source', { jobId, sourceId }),
	jobScores: (jobId: string) => invoke<JobScores>('job_scores', { jobId }),
	jobAgreement: (jobId: string) => invoke<JobAgreement>('job_agreement', { jobId }),
	diffTexts: (left: string, right: string) => invoke<DiffSpan[]>('diff_texts', { left, right }),
	listProjects: () => invoke<ProjectSummary[]>('list_projects'),
	createProject: (name: string, description: string) =>
		invoke<Project>('create_project', { name, description }),
	updateProject: (id: string, name: string, description: string) =>
		invoke<Project>('update_project', { id, name, description }),
	archiveProject: (id: string, archived: boolean) =>
		invoke<Project>('archive_project', { id, archived }),
	deleteProject: (id: string) => invoke<void>('delete_project', { id }),
	projectView: (id: string) => invoke<ProjectView>('project_view', { id }),
	createCollection: (project: string, name: string) =>
		invoke<Collection>('create_collection', { project, name }),
	renameCollection: (project: string, id: string, name: string) =>
		invoke<void>('rename_collection', { project, id, name }),
	deleteCollection: (project: string, id: string) =>
		invoke<void>('delete_collection', { project, id }),
	addDocuments: (project: string, sources: SourceFile[], collection: string | null) =>
		invoke<string[]>('add_documents', { project, sources, collection }),
	renameDocument: (project: string, id: string, title: string) =>
		invoke<void>('rename_document', { project, id, title }),
	deleteDocuments: (project: string, ids: string[]) =>
		invoke<void>('delete_documents', { project, ids }),
	addToCollection: (project: string, collection: string, ids: string[]) =>
		invoke<void>('add_to_collection', { project, collection, ids }),
	removeFromCollection: (project: string, collection: string, ids: string[]) =>
		invoke<void>('remove_from_collection', { project, collection, ids }),
	tagDocuments: (project: string, ids: string[], tag: string) =>
		invoke<void>('tag_documents', { project, ids, tag }),
	untagDocuments: (project: string, ids: string[], tag: string) =>
		invoke<void>('untag_documents', { project, ids, tag }),
	renameTag: (project: string, from: string, to: string) =>
		invoke<TagRenamed>('rename_tag', { project, from, to }),
	describeDocument: (project: string, id: string, description: Draft) =>
		invoke<void>('describe_document', { project, id, description }),
	setSourceDefaults: (id: string, defaults: Draft) =>
		invoke<Project>('set_source_defaults', { id, defaults }),
	/** Every part with where it came from, or with `profile` the columns of a saved profile.
	 *  This and the other table exports return the files beside the chosen one written under
	 *  another name, a file of the researcher's having theirs (`tableBeside`). */
	exportDescriptions: (
		project: string,
		collection: string | null,
		profile: string | null,
		path: string
	) => invoke<string[]>('export_descriptions', { project, collection, profile, path }),
	/** One analysis's answers over the project's documents, with their tags, as a workbook at
	 *  `path` and as CSV beside it. */
	exportAnswers: (project: string, job: string, setup: string, path: string) =>
		invoke<string[]>('export_answers', { project, job, setup, path }),
	/** The text of each chosen document, frozen as sources an analysis reads, and those left
	 *  out. */
	documentSources: (project: string, documents: string[]) =>
		invoke<DocumentSources>('document_sources', { project, documents }),
	listExportProfiles: () => invoke<ExportProfile[]>('list_export_profiles'),
	saveExportProfile: (profile: ExportProfile) =>
		invoke<ExportProfile>('save_export_profile', { profile }),
	duplicateExportProfile: (id: string, name: string) =>
		invoke<ExportProfile>('duplicate_export_profile', { id, name }),
	deleteExportProfile: (id: string) => invoke<void>('delete_export_profile', { id }),
	checkExportProfile: (profile: ExportProfile) =>
		invoke<ProfileError | null>('check_export_profile', { profile }),
	profileStarter: (starter: ProfileStarter) =>
		invoke<ProfileColumn[]>('profile_starter', { starter }),
	projectAnswerFields: (project: string) =>
		invoke<AnswerField[]>('project_answer_fields', { project }),
	shareExportProfile: (id: string, fixedTexts: boolean, path: string) =>
		invoke<void>('share_export_profile', { id, fixedTexts, path }),
	importExportProfile: (path: string) => invoke<ExportProfile>('import_export_profile', { path }),
	projectSearch: (project: string, request: SearchRequest) =>
		invoke<SearchView>('project_search', { project, request }),
	rebuildSearchIndex: () => invoke<void>('rebuild_search_index'),
	prepareProjectSearch: (project: string) => invoke<void>('prepare_project_search', { project }),
	/** The search by meaning's set-up: the model, the estimate (nothing sent), runs and cost. */
	meaningStatus: (project: string) => invoke<MeaningStatus>('meaning_status', { project }),
	chooseMeaningModel: (project: string, provider: string, model: string) =>
		invoke<SpaceChoice>('choose_meaning_model', { project, provider, model }),
	consentMeaning: (project: string) => invoke<void>('consent_meaning', { project }),
	/** Starts preparing the project's text: the one call that sends it to a cloud model. */
	prepareMeaning: (project: string) => invoke<void>('prepare_meaning', { project }),
	pauseMeaning: (project: string) => invoke<void>('pause_meaning', { project }),
	cancelMeaning: (project: string) => invoke<void>('cancel_meaning', { project }),
	removeMeaningSpace: (project: string, space: EmbeddingSpace) =>
		invoke<void>('remove_meaning_space', { project, space }),
	meaningSearch: (project: string, request: MeaningRequest) =>
		invoke<MeaningView>('meaning_search', { project, request }),
	meaningMap: (project: string) => invoke<MapView>('meaning_map', { project }),
	meaningNeighbours: (project: string, document: string) =>
		invoke<Neighbour[]>('meaning_neighbours', { project, document }),
	dictionaryView: (project: string, collection: string | null) =>
		invoke<DictionaryView>('dictionary_view', { project, collection }),
	dictionaryEvidence: (project: string, namespace: string, forms: string[]) =>
		invoke<Evidence[]>('dictionary_evidence', { project, namespace, forms }),
	dictionaryMerge: (project: string, namespace: string, items: Item[], label: string) =>
		invoke<Entry>('dictionary_merge', { project, namespace, items, label }),
	dictionarySplit: (project: string, entry: string, forms: string[]) =>
		invoke<void>('dictionary_split', { project, entry, forms }),
	dictionaryRelabel: (project: string, entry: string, label: string) =>
		invoke<Entry>('dictionary_relabel', { project, entry, label }),
	dictionarySetReviewed: (project: string, namespace: string, items: Item[], reviewed: boolean) =>
		invoke<void>('dictionary_set_reviewed', { project, namespace, items, reviewed }),
	dictionaryKeepApart: (project: string, namespace: string, a: Item, b: Item) =>
		invoke<void>('dictionary_keep_apart', { project, namespace, a, b }),
	dictionaryUndoApart: (project: string, a: string, b: string) =>
		invoke<void>('dictionary_undo_apart', { project, a, b }),
	exportDictionary: (project: string, path: string) =>
		invoke<string[]>('export_dictionary', { project, path }),
	dictionaryPlan: (project: string, collection: string | null, namespace: string) =>
		invoke<ProposalPlan>('dictionary_plan', { project, collection, namespace }),
	dictionaryPropose: (
		project: string,
		collection: string | null,
		namespace: string,
		name: string,
		setup: SetupRequest,
		pace: Pace
	) => invoke<Job>('dictionary_propose', { project, collection, namespace, name, setup, pace }),
	dictionaryProposals: (project: string) =>
		invoke<ProposalsView>('dictionary_proposals', { project }),
	dictionaryDismiss: (project: string, namespace: string, key: string, dismissed: boolean) =>
		invoke<void>('dictionary_dismiss', { project, namespace, key, dismissed }),
	dictionaryForgetRequest: (project: string, request: string) =>
		invoke<void>('dictionary_forget_request', { project, request }),
	mergeDocuments: (project: string, from: string, into: string) =>
		invoke<void>('merge_documents', { project, from, into }),
	documentDetail: (project: string, id: string) =>
		invoke<DocumentDetail>('document_detail', { project, id }),
	setPreferred: (
		project: string,
		document: string,
		task: Task,
		job: string | null,
		setup: string | null
	) => invoke<void>('set_preferred', { project, document, task, job, setup }),
	attachPlan: (project: string, job: string) =>
		invoke<SourceMatch[]>('attach_plan', { project, job }),
	attachJob: (project: string, job: string, choices: AttachChoice[], collection: string | null) =>
		invoke<string[]>('attach_job', { project, job, choices, collection }),
	linkJob: (project: string, job: string, collection: string | null) =>
		invoke<string[]>('link_job', { project, job, collection }),
	/** Links several jobs to their documents, all or none. */
	linkJobs: (project: string, jobs: string[], collection: string | null) =>
		invoke<string[]>('link_jobs', { project, jobs, collection }),
	detachJob: (project: string, document: string, job: string, source: string) =>
		invoke<void>('detach_job', { project, document, job, source }),
	reviewQueue: (project: string, selection: QueueSelection) =>
		invoke<QueueView>('review_queue', { project, selection }),
	/** Sets a result aside from the list over `selection`, kept with the project; with `aside`
	 * false, brings it back. */
	setResultAside: (project: string, selection: QueueSelection, result: ResultKey, aside: boolean) =>
		invoke<void>('set_result_aside', { project, selection, result, aside }),
	/** Draws a new random sample of `size` pages nothing flags (with `seed`, or a new one);
	 * without a size, removes it. */
	setReviewSample: (
		project: string,
		selection: QueueSelection,
		size: number | null,
		seed: number | null = null
	) => invoke<SampleSpec | null>('set_review_sample', { project, selection, size, seed }),
	/** Keeps an item aside with a note, for the answer it holds; without a note, puts it back. */
	keepAside: (project: string, item: QueueItemKey, revision: string | null, note: string | null) =>
		invoke<void>('keep_aside', { project, item, revision, note }),
	exportReviewManifest: (project: string, selection: QueueSelection, path: string) =>
		invoke<string[]>('export_review_manifest', { project, selection, path }),
	/** What saving a project to one file would write, with and without its original files. */
	planProjectFile: (project: string) => invoke<ProjectFilePlan>('plan_project_file', { project }),
	/** Saves a project to one file at `path`; its progress comes as `project-file-progress`. */
	saveProjectFile: (project: string, path: string, withOriginals: boolean) =>
		invoke<ProjectFileSaved>('save_project_file', { project, path, withOriginals }),
	/** Opens a project file as a new project beside the others. */
	openProjectFile: (path: string) => invoke<ProjectFileOpened>('open_project_file', { path }),
	/** Checks a project file without opening it; its progress comes as `project-file-progress`. */
	checkProjectFile: (path: string) => invoke<ProjectFileCheck>('check_project_file', { path }),
	setShellLabels: (labels: ShellLabels) => invoke<void>('set_shell_labels', { labels }),
	hideWindow: () => invoke<void>('hide_window'),
	/** Whether changes are not saved, so closing the window or quitting asks first. */
	setDraftOpen: (open: boolean) => invoke<void>('set_draft_open', { open }),
	quitApp: () => invoke<void>('quit_app')
};

export const on = {
	progress: (handler: (event: ProgressEvent) => void) => listen('progress', handler),
	/** The window closed or the app quit while jobs run or changes are not saved: whether jobs
	 * run, to ask about them once the changes are settled. */
	closeRequested: (handler: (jobs: boolean) => void) => listen<boolean>('close-requested', handler),
	/** Settings… in the application menu (macOS). */
	openSettings: (handler: () => void) => listen<null>('open-settings', handler),
	searchProgress: (handler: (event: SearchProgress) => void) => listen('search-progress', handler),
	/** How far a project's text is prepared for the search by meaning, after each batch. */
	meaningProgress: (handler: (event: MeaningProgress) => void) =>
		listen('meaning-progress', handler),
	/** The offline status changed outside a command: read it again. */
	offlineChanged: (handler: () => void) => listen<null>('offline-changed', handler),
	/** The model list was refreshed at start-up: read it again, with the release it names. */
	catalogChanged: (handler: () => void) => listen<null>('catalog-changed', handler),
	/** How far a project file is written or opened, in bytes of the files it carries. */
	projectFileProgress: (handler: (event: ProjectFileProgress) => void) =>
		listen('project-file-progress', handler)
};

export async function revealInFolder(path: string): Promise<void> {
	if (!isTauri()) return;
	const { revealItemInDir } = await import('@tauri-apps/plugin-opener');
	await revealItemInDir(path);
}

export async function openUrl(url: string): Promise<void> {
	if (!isTauri()) {
		window.open(url, '_blank', 'noopener');
		return;
	}
	const { openUrl } = await import('@tauri-apps/plugin-opener');
	await openUrl(url);
}

/** Paths dropped onto the window. */
export async function onDrop(handler: (paths: string[]) => void): Promise<Unlisten> {
	if (!isTauri()) return () => {};
	const { getCurrentWebview } = await import('@tauri-apps/api/webview');
	return getCurrentWebview().onDragDropEvent((event) => {
		if (event.payload.type === 'drop') handler(event.payload.paths);
	});
}

/** Whether files are being dragged over the window: from when they enter it until they leave,
 *  are dropped, or the drag is given up. */
export async function onDragging(handler: (over: boolean) => void): Promise<Unlisten> {
	if (!isTauri()) return () => {};
	const { getCurrentWebview } = await import('@tauri-apps/api/webview');
	return getCurrentWebview().onDragDropEvent((event) => {
		handler(event.payload.type === 'enter' || event.payload.type === 'over');
	});
}

/** A system notification, when the window is not in front (or `always`). */
export async function notify(title: string, body: string, always = false): Promise<void> {
	if (!isTauri() || (!always && document.hasFocus())) return;
	const { isPermissionGranted, requestPermission, sendNotification } =
		await import('@tauri-apps/plugin-notification');
	let granted = await isPermissionGranted();
	if (!granted) granted = (await requestPermission()) === 'granted';
	if (granted) sendNotification({ title, body });
}
