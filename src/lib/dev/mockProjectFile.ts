// The pretend back end's project files (#213): what saving the sample project would write, the
// save itself, and a file opened as a new project. The native dialogs answer with a path when
// they ask for a Kalamu project file. `?damaged-file`: the file opened is refused as damaged.
// `?missing-text`: the file opened lacks a corrected text a job read, as 1.10.0 could save it.
// Checking a file (#246) finds it whole; `?altered-file`: a part no longer matches what was
// saved; `?newer-file`: refused as newer; `?renamed`: an original would take another name.
// The file opened brings a job with it, listed with the others from then on.
// `?with-originals`: the file opened carries its originals, copied into the data folder, which
// deleting the project then says it removes (#274).

import type { JobSummary } from '../bindings/JobSummary.js';
import type { Project } from '../bindings/Project.js';
import type { ProjectFileCheck } from '../bindings/ProjectFileCheck.js';
import type { ProjectFileOpened } from '../bindings/ProjectFileOpened.js';
import type { ProjectFilePlan } from '../bindings/ProjectFilePlan.js';
import type { ProjectFileSaved } from '../bindings/ProjectFileSaved.js';
import { copiedOriginals, projectCommand } from './mockProjects.js';

declare global {
	interface Window {
		__kalamuSavedProject?: { project: string; path: string; withOriginals: boolean };
	}
}

const SAVED_AT = 'C:\\Users\\me\\Documents\\Letters from Kayes.kalamu';
/** The jobs the project files opened brought, copied into the job database. */
export const openedJobs: JobSummary[] = [];
const OPENED_FROM = 'C:\\Users\\me\\Downloads\\Mission letters.kalamu';

function query(): URLSearchParams {
	return new URLSearchParams(location.search);
}

/** What checking the sample project's file finds: whole unless the page asks otherwise. */
function checked(withOriginals: boolean): ProjectFileCheck {
	const altered = query().has('altered-file');
	const missing = query().has('missing-text');
	return {
		verdict: altered ? 'damaged' : missing ? 'incomplete' : 'whole',
		name: 'Mission letters',
		saved_at: Date.UTC(2026, 10, 21, 9, 30),
		app_version: '1.10.0',
		documents: 4,
		jobs: 3,
		texts: missing ? 1 : 2,
		missing_texts: missing
			? [
					{
						documents: ['Letter of 1915'],
						jobs: ['Mission letters, summary'],
						file_name: 'lettre 1.txt'
					}
				]
			: [],
		damaged: altered ? ['Letter of 1915'] : [],
		with_originals: withOriginals,
		originals: withOriginals ? 3 : 0,
		originals_not_chosen: withOriginals ? 0 : 4,
		originals_left: withOriginals ? 1 : 0,
		to_find: withOriginals
			? ['Letter of 1916']
			: ['Letter of 1915', 'Letter of 1916', 'Draft of 1917', 'Map of Kayes'],
		renamed: query().has('renamed')
			? [{ documents: ['Draft of 1917'], from: 'draft?.pdf', to: 'draft_.pdf' }]
			: []
	};
}

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
				originals_left: a.withOriginals ? 1 : 0,
				// Only a file that checks is kept: a save that succeeds is whole.
				check: {
					...checked(Boolean(a.withOriginals)),
					verdict: 'whole',
					damaged: [],
					missing_texts: []
				}
			} satisfies ProjectFileSaved;
		case 'check_project_file':
			if (query().has('newer-file'))
				throw { id: 'project.file_newer', detail: 'file format 2; this version reads 1' };
			return checked(false);
		case 'open_project_file': {
			if (new URLSearchParams(location.search).has('damaged-file'))
				throw { id: 'project.file_damaged', detail: 'database disk image is malformed' };
			const project = projectCommand('create_project', {
				name: 'Mission letters',
				description: 'Letters of the mission, 1912–1960'
			}) as Project;
			const withOriginals = query().has('with-originals');
			if (withOriginals) copiedOriginals.set(project.id, 3);
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
				originals: withOriginals ? 3 : 0,
				with_originals: withOriginals,
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
