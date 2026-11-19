// New job's Start: once the researcher agrees to what each cloud provider is sent, the job is
// created and started, or, with an analysis to follow, the two-step run that reads the text and
// analyses it; a job started from a project joins it, and the job's page opens.

import type { Pace } from './bindings/Pace.js';
import type { Preset } from './bindings/Preset.js';
import { mayStart } from './dialogs.js';
import { messages } from './i18n/index.js';
import { offeredFormats } from './newJob.js';
import type { JobChoices } from './newJobChoices.svelte.js';
import type { JobFiles } from './newJobFiles.svelte.js';
import { toRequest } from './newJobModels.js';
import type { FollowingAnalysis } from './newJobThen.svelte.js';
import { app, go, liveJob, refreshJobs, toast, type FromProject } from './state.svelte.js';
import { api } from './tauri.js';

/** The form as Start reads it: its choices, files and the analysis after, the pace chosen, the
 * project it was opened from, and what to do once the job has started. */
export interface Starting {
	choices: JobChoices;
	files: JobFiles;
	following: FollowingAnalysis;
	pace: () => Pace;
	project: FromProject | undefined;
	started: () => void;
}

/** Starts the job, or the run with its analysis after; nothing when the researcher declines
 * what would be sent. Throws what the engine refuses. */
export async function startJob(s: Starting): Promise<void> {
	const { choices, files, following } = s;
	const catalog = choices.catalog;
	const preset = following.chosen();
	// Asked once for each cloud provider the job, or its analysis after, sends files to.
	const setups = [
		...choices.drafts.map((d) => toRequest(catalog, choices.task, d)),
		...(preset
			? following.pipeline(preset, s.pace(), $state.snapshot(choices.destination)).setups
			: [])
	];
	const beside = choices.destination.kind === 'beside_sources';
	if (!(await mayStart(setups, catalog, $state.snapshot(files.sources), beside))) return;
	if (preset) {
		await startWithAnalysis(s, preset);
		return;
	}
	const m = messages();
	const task = choices.task;
	const sources = files.sources;
	const job = await api.createJob({
		name: files.name.trim() || sources[0]?.file_name || m.newJob.title,
		task,
		sources: $state.snapshot(sources),
		setups: choices.drafts.map((d) => toRequest(choices.catalog, task, d)),
		settings: choices.settings(s.pace()),
		output: {
			formats: choices.formats.filter((f) =>
				offeredFormats(choices.catalog, choices.rule, choices.drafts, sources).includes(f)
			),
			destination: $state.snapshot(choices.destination),
			options: $state.snapshot(app.settings!.export_options)
		}
	});
	liveJob(job.id);
	// Started from a project: its files are the project's documents. The job runs even if this
	// fails, and can be added from the project later.
	const project = s.project;
	if (project) {
		await api
			.linkJob(project.id, job.id, project.collectionId ?? null)
			.catch(() => toast(m.projects.linkFailed(project.name), { kind: 'error' }));
	}
	await api.startJob(job.id);
	await refreshJobs();
	s.started();
	go({ name: 'jobs', jobId: job.id });
}

/** Reads the text, then analyses it: a two-step workflow, unsaved, run on these files. */
async function startWithAnalysis(s: Starting, preset: Preset) {
	const { choices, files, following } = s;
	const m = messages();
	const task = choices.task;
	const sources = files.sources;
	const jobName = files.name.trim() || sources[0]?.file_name || m.newJob.title;
	const view = await api.startRun({
		workflow: following.workflow(
			choices.pipeline(m.task[task], s.pace(), true),
			preset,
			s.pace(),
			$state.snapshot(choices.destination)
		),
		name: jobName,
		sources: $state.snapshot(sources),
		project: s.project?.id ?? null,
		collection: s.project?.collectionId ?? null
	});
	const first = view.steps[0]?.job?.id;
	if (first) liveJob(first);
	await refreshJobs();
	s.started();
	go(first ? { name: 'jobs', jobId: first } : { name: 'workflows', runId: view.run.id });
}
