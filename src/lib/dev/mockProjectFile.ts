// The pretend back end's project files (#213): what saving the sample project would write, the
// save itself, and a file opened as a new project. The native dialogs answer with a path when
// they ask for a Kalamu project file. `?damaged-file`: the file opened is refused as damaged.
// `?missing-text`: the file opened lacks a corrected text a job read, as 1.10.0 could save it.
// The file opened brings a job with it, listed with the others from then on.

import type { JobSummary } from '../bindings/JobSummary.js';
import type { Project } from '../bindings/Project.js';
import type { ProjectFileOpened } from '../bindings/ProjectFileOpened.js';
import type { ProjectFilePlan } from '../bindings/ProjectFilePlan.js';
import type { ProjectFileSaved } from '../bindings/ProjectFileSaved.js';
import { projectCommand } from './mockProjects.js';

declare global {
	interface Window {
		__kalamuSavedProject?: { project: string; path: string; withOriginals: boolean };
	}
}

const SAVED_AT = 'C:\\Users\\me\\Documents\\Letters from Kayes.kalamu';
/** The jobs the project files opened brought, copied into the job database. */
export const openedJobs: JobSummary[] = [];
const OPENED_FROM = 'C:\\Users\\me\\Downloads\\Mission letters.kalamu';

/** Answers a project file command, or `undefined` for any other. */
export function projectFileCommand(cmd: string, a: Record<string, unknown>): unknown {
	const asks = () => JSON.stringify(a).includes('"kalamu"');
	switch (cmd) {
		case 'plan_project_file':
			return {
				name: 'Letters from Kayes',
				documents: 4,
				jobs: 3,
				results_bytes: 12_400_000,
				originals: 4,
				originals_bytes: 3_400_000_000,
				originals_missing: 1,
				file_name: 'Letters from Kayes.kalamu'
			} satisfies ProjectFilePlan;
		case 'save_project_file':
			window.__kalamuSavedProject = a as unknown as Window['__kalamuSavedProject'];
			return {
				path: String(a.path),
				bytes: a.withOriginals ? 3_412_400_000 : 12_400_000,
				originals: a.withOriginals ? 3 : 0,
				originals_left: a.withOriginals ? 1 : 0
			} satisfies ProjectFileSaved;
		case 'open_project_file': {
			if (new URLSearchParams(location.search).has('damaged-file'))
				throw { id: 'project.file_damaged', detail: 'database disk image is malformed' };
			const project = projectCommand('create_project', {
				name: 'Mission letters',
				description: 'Letters of the mission, 1912–1960'
			}) as Project;
			const now = Date.now();
			openedJobs.push({
				id: `00000000-0000-0000-0000-00000000f${String(openedJobs.length).padStart(3, '0')}`,
				name: 'Mission letters, text',
				task: 'text_extraction',
				status: 'completed',
				created_at: now - 86_400_000,
				updated_at: now,
				source_count: 1,
				setup_count: 1,
				pieces_total: 2,
				pieces_finished: 2,
				pieces_failed: 0,
				pieces_truncated: 0,
				cost_usd: 0.004,
				models: ['gemini-3.8-flash'],
				workflow: null
			});
			return {
				project_id: project.id,
				name: project.name,
				documents: 0,
				jobs: 1,
				originals: 0,
				with_originals: false,
				saved_at: Date.now(),
				missing_texts: new URLSearchParams(location.search).has('missing-text')
					? [
							{
								documents: ['Letter of 1915'],
								jobs: ['Mission letters, summary'],
								file_name: 'lettre 1.txt'
							}
						]
					: []
			} satisfies ProjectFileOpened;
		}
		case 'plugin:dialog|save':
			return asks() ? SAVED_AT : undefined;
		case 'plugin:dialog|open':
			return asks() ? OPENED_FROM : undefined;
		default:
			return undefined;
	}
}
