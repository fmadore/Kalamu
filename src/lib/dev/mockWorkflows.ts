// Sample workflows for the browser preview (`?mock&view=workflows`, `view=workflow`,
// `view=run`): extraction then analysis of the sample bulletin, its run waiting for the
// extraction's review. Development builds only.

import type { JobSummary } from '../bindings/JobSummary.js';
import type { Pipeline } from '../bindings/Pipeline.js';
import type { RunStep } from '../bindings/RunStep.js';
import type { RunView } from '../bindings/RunView.js';
import type { SourceFile } from '../bindings/SourceFile.js';
import type { StartRun } from '../bindings/StartRun.js';
import type { Workflow } from '../bindings/Workflow.js';

export const WORKFLOW_ID = 'workflow-00000000-0000-0000-0000-0000000000w1';
export const RUN_ID = '00000000-0000-0000-0000-0000000000r1';

let workflows: Workflow[] = [];
let runs: RunView[] = [];

function frozen(pipeline: Pipeline, input: RunStep['input'], review: boolean): RunStep {
	return {
		name: pipeline.name,
		task: pipeline.task,
		input,
		review,
		setups: [],
		settings: pipeline.settings,
		output: {
			formats: pipeline.formats,
			destination: pipeline.destination,
			options: { piece_markers: true, speaker_labels: true, timestamps: true }
		},
		progress: { job_id: null, approval: null, error: null }
	};
}

/** One workflow from the sample pipelines, and a run of it on the sample bulletin. */
export function seedWorkflows(
	extract: Pipeline,
	analyse: Pipeline,
	extraction: JobSummary,
	source: SourceFile
): void {
	workflows = [
		{
			id: WORKFLOW_ID,
			name: 'Bulletins: text, then people and places',
			version: 3,
			steps: [
				{ pipeline: extract, input: { kind: 'sources' }, review: true },
				{ pipeline: analyse, input: { kind: 'step', step: 0 }, review: false }
			],
			updated_at: Date.now() - 86_400_000
		}
	];
	const first = frozen(extract, { kind: 'sources' }, true);
	first.progress.job_id = extraction.id;
	runs = [
		{
			run: {
				id: RUN_ID,
				name: 'Bulletin, June 1957',
				workflow_id: WORKFLOW_ID,
				workflow_name: workflows[0].name,
				version: 3,
				created_at: Date.now() - 3_600_000,
				sources: [source],
				steps: [first, frozen(analyse, { kind: 'step', step: 0 }, false)],
				project: null,
				collection: null,
				cancelled: false
			},
			status: 'needs_you',
			steps: [
				{ state: 'review', job: extraction },
				{ state: 'waiting', job: null }
			]
		}
	];
}

/** Answers a workflow command, or `undefined` for any other. */
export function workflowCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'list_workflows':
			return workflows;
		case 'save_workflow': {
			const w = a.workflow as Workflow;
			const id = w.id || `workflow-${workflows.length + 1}`;
			const saved = { ...w, id, version: w.version + 1, updated_at: Date.now() };
			workflows = [...workflows.filter((x) => x.id !== id), saved];
			return saved;
		}
		case 'duplicate_workflow': {
			const original = workflows.find((x) => x.id === a.id);
			if (!original) return null;
			const copy = {
				...original,
				id: `workflow-${workflows.length + 1}`,
				name: a.name as string,
				version: 1
			};
			workflows = [...workflows, copy];
			return copy;
		}
		case 'delete_workflow':
			workflows = workflows.filter((x) => x.id !== a.id);
			return null;
		case 'export_workflow':
			return null;
		case 'list_runs':
			return runs;
		case 'get_run':
			return runs.find((r) => r.run.id === a.id) ?? null;
		case 'job_step': {
			for (const view of runs) {
				const step = view.steps.findIndex((s) => s.job?.id === a.id);
				if (step >= 0) return { view, step };
			}
			return null;
		}
		case 'approve_step': {
			const view = runs.find((r) => r.run.id === a.id);
			if (view) {
				const step = a.step as number;
				view.run.steps[step].progress.approval = { at: Date.now(), partial: false };
				view.steps[step].state = 'done';
				view.steps[step + 1] = { state: 'running', job: null };
				view.status = 'running';
			}
			return null;
		}
		case 'cancel_run': {
			const view = runs.find((r) => r.run.id === a.id);
			if (view) {
				view.run.cancelled = true;
				view.status = 'stopped';
				view.steps = view.steps.map((s) =>
					s.state === 'waiting' ? { ...s, state: 'cancelled' } : s
				);
			}
			return null;
		}
		case 'delete_run':
			runs = runs.filter((r) => r.run.id !== a.id);
			return null;
		case 'start_run': {
			const start = a.start as StartRun;
			const view: RunView = {
				run: {
					id: `00000000-0000-0000-0000-0000000000r${runs.length + 2}`,
					name: start.name,
					workflow_id: start.workflow.id,
					workflow_name: start.workflow.name,
					version: start.workflow.version,
					created_at: Date.now(),
					sources: start.sources,
					steps: start.workflow.steps.map((s) => frozen(s.pipeline, s.input, s.review)),
					project: start.project,
					collection: start.collection,
					cancelled: false
				},
				status: 'running',
				steps: start.workflow.steps.map((s, i) => ({
					state: i === 0 || s.input.kind === 'sources' ? 'running' : 'waiting',
					job: null
				}))
			};
			runs = [view, ...runs];
			return view;
		}
		default:
			return undefined;
	}
}
