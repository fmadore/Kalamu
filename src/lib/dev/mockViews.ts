// Where the browser preview opens (`?mock&view=…`), for screenshots of each screen, and what
// only the desktop app can tell about the window. Development builds only.

import type { View } from '../state.svelte.js';
import { contentsJob, contentsSource, correctionJob, gallica } from './mockContents.js';
import {
	analysisJob,
	article,
	audioJob,
	job,
	recording,
	source,
	video,
	videoJob
} from './mockJobs.js';
import { DOCUMENT_ID, PROJECT_ID } from './mockProjects.js';
import { scanAnalysisJob } from './mockDrawn.js';
import { recordsJob, recordsSource } from './mockRecords.js';
import { INTERVIEW_JOB, interviewJob } from './mockInterview.js';
import { RUN_ID, WORKFLOW_ID } from './mockWorkflows.js';

/** Whether the window shows files being dragged over it (`?dragging`), which only the desktop
 * app can tell. */
export function mockDragging(): boolean {
	return new URLSearchParams(location.search).has('dragging');
}

/** The screen made to fail as it is drawn (`?broken=prompts`, a view's name), for the test of
 * what the interface does with a failure nobody expected (`e2e/unexpected.spec.ts`). */
export function mockBroken(): string | null {
	return new URLSearchParams(location.search).get('broken');
}

/** The screen to open on, from `?mock&view=…`, for screenshots of each screen. */
export function mockView(): View | null {
	switch (new URLSearchParams(location.search).get('view') ?? '') {
		case 'all-jobs':
			return { name: 'allJobs' };
		case 'jobs':
			return { name: 'jobs', jobId: job.id };
		case 'jobs-analysis':
			return { name: 'jobs', jobId: analysisJob.id };
		case 'review':
			return { name: 'review', jobId: job.id, sourceId: source.id };
		case 'review-audio':
			return { name: 'review', jobId: audioJob.id, sourceId: recording.id };
		case 'review-analysis':
			return { name: 'review', jobId: analysisJob.id, sourceId: article.id };
		case 'review-analysis-scan':
			return { name: 'review', jobId: scanAnalysisJob.id, sourceId: source.id };
		case 'review-youtube':
			return { name: 'review', jobId: videoJob.id, sourceId: video.id };
		case 'review-contents':
			return { name: 'review', jobId: contentsJob.id, sourceId: contentsSource.id };
		case 'review-interview':
			return { name: 'review', jobId: INTERVIEW_JOB, sourceId: interviewJob.sources[0].id };
		case 'jobs-audio':
			return { name: 'jobs', jobId: audioJob.id };
		case 'review-records':
			return { name: 'review', jobId: recordsJob.id, sourceId: recordsSource.id };
		case 'review-correction':
			return { name: 'review', jobId: correctionJob.id, sourceId: gallica.id };
		case 'prompts':
		case 'prompts-analysis':
			return { name: 'prompts' };
		case 'settings':
			return { name: 'settings' };
		case 'settings-general':
			return { name: 'settings', tab: 'general' };
		case 'settings-servers':
			return { name: 'settings', tab: 'servers' };
		case 'settings-about':
			return { name: 'settings', tab: 'about' };
		case 'glossary':
			return { name: 'glossary' };
		case 'new':
			return { name: 'new' };
		case 'projects':
			return { name: 'projects' };
		case 'project':
			return { name: 'projects', projectId: PROJECT_ID };
		case 'document':
			return { name: 'projects', projectId: PROJECT_ID, documentId: DOCUMENT_ID };
		case 'queue':
			return { name: 'projects', projectId: PROJECT_ID, pane: 'queue' };
		case 'search':
			return { name: 'projects', projectId: PROJECT_ID, pane: 'search' };
		case 'map':
			return { name: 'projects', projectId: PROJECT_ID, pane: 'map' };
		case 'workflows':
			return { name: 'workflows' };
		case 'workflow':
			return { name: 'workflows', workflowId: WORKFLOW_ID };
		case 'run':
			return { name: 'workflows', runId: RUN_ID };
		default:
			return null;
	}
}
