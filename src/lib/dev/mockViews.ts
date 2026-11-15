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
import { RUN_ID, WORKFLOW_ID } from './mockWorkflows.js';

/** Whether the window shows files being dragged over it (`?dragging`), which only the desktop
 * app can tell. */
export function mockDragging(): boolean {
	return new URLSearchParams(location.search).has('dragging');
}

/** The screen to open on, from `?mock&view=…`, for screenshots of each screen. */
export function mockView(): View | null {
	switch (new URLSearchParams(location.search).get('view') ?? '') {
		case 'all-jobs':
			return { name: 'allJobs' };
		case 'jobs':
			return { name: 'jobs', jobId: job.id };
		case 'review':
			return { name: 'review', jobId: job.id, sourceId: source.id };
		case 'review-audio':
			return { name: 'review', jobId: audioJob.id, sourceId: recording.id };
		case 'review-analysis':
			return { name: 'review', jobId: analysisJob.id, sourceId: article.id };
		case 'review-youtube':
			return { name: 'review', jobId: videoJob.id, sourceId: video.id };
		case 'review-contents':
			return { name: 'review', jobId: contentsJob.id, sourceId: contentsSource.id };
		case 'review-correction':
			return { name: 'review', jobId: correctionJob.id, sourceId: gallica.id };
		case 'prompts':
		case 'prompts-analysis':
			return { name: 'prompts' };
		case 'settings':
			return { name: 'settings' };
		case 'settings-general':
			return { name: 'settings', tab: 'general' };
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
