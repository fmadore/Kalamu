// Sample projects for the browser preview (`?mock&view=projects`, `view=project`,
// `view=document`): one project of mission archives whose documents were read by the sample
// jobs. Development builds only.

import type { Collection } from '../bindings/Collection.js';
import type { Described } from '../bindings/Described.js';
import type { Document } from '../bindings/Document.js';
import type { DocumentDetail } from '../bindings/DocumentDetail.js';
import type { DocumentResult } from '../bindings/DocumentResult.js';
import type { Job } from '../bindings/Job.js';
import type { Project } from '../bindings/Project.js';
import type { ProjectSummary } from '../bindings/ProjectSummary.js';
import type { ProjectView } from '../bindings/ProjectView.js';
import type { SourceFile } from '../bindings/SourceFile.js';
import { FIELDS, kept, type Draft } from '../described.js';
import { dictionaryCommand } from './mockDictionary.js';

export const PROJECT_ID = '00000000-0000-0000-0000-0000000000p1';
export const DOCUMENT_ID = '00000000-0000-0000-0000-0000000000d1';
const LETTERS = '00000000-0000-0000-0000-0000000000c1';
const BULLETINS = '00000000-0000-0000-0000-0000000000c2';

const day = 86_400_000;
const now = Date.now();

let projects: Project[] = [
	{
		id: PROJECT_ID,
		name: 'Missions in French West Africa',
		description:
			'Bulletins, letters and interviews on Catholic and Muslim associations, 1945–1960.',
		archived: false,
		source_defaults: { repository: 'Archives nationales du Sénégal', fonds: 'Fonds AOF' },
		created_at: now - 30 * day,
		updated_at: now - day
	},
	{
		id: '00000000-0000-0000-0000-0000000000p2',
		name: 'Ouagadougou interviews 2024',
		description: '',
		archived: true,
		source_defaults: {},
		created_at: now - 90 * day,
		updated_at: now - 60 * day
	}
];
let collections: Collection[] = [
	{ id: LETTERS, project_id: PROJECT_ID, name: 'Letters', created_at: now - 20 * day },
	{ id: BULLETINS, project_id: PROJECT_ID, name: 'Bulletins', created_at: now - 20 * day }
];
let documents: Document[] = [];
let members: [string, string][] = [];
/** Document id → the jobs and sources that read it. */
let links: [string, string, string][] = [];
const preferred = new Set<string>();

export interface MockJobs {
	job: Job;
	audioJob: Job;
	analysisJob: Job;
	extras: SourceFile[];
}

let jobs: Job[] = [];

/** A document's description as it stands, as `kalamu-projects` gives it. */
function described(document: Document): Described[] {
	const defaults = projects.find((p) => p.id === document.project_id)?.source_defaults ?? {};
	return FIELDS.flatMap((field): Described[] => {
		const own = document.source_description[field];
		const shared = defaults[field];
		if (own) return [{ field, value: own, by: 'document' }];
		return shared ? [{ field, value: shared, by: 'project' }] : [];
	});
}

/** Fills the sample project from the sample jobs, once. */
export function seedProjects(sample: MockJobs): void {
	if (documents.length > 0) return;
	jobs = [sample.job, sample.audioJob, sample.analysisJob];
	const doc = (id: string, title: string, source: SourceFile, age: number): Document => ({
		id,
		project_id: PROJECT_ID,
		title,
		source,
		created_at: now - age * day,
		tags: [],
		source_description: {}
	});
	documents = [
		doc(DOCUMENT_ID, 'Bulletin, May 1957', sample.job.sources[0], 12),
		doc(
			'00000000-0000-0000-0000-0000000000d2',
			'Interview with the imam',
			sample.audioJob.sources[0],
			9
		),
		...sample.extras.map((s, i) =>
			doc(`00000000-0000-0000-0000-0000000000e${i}`, s.file_name.replace(/\.[^.]+$/, ''), s, 5 - i)
		)
	];
	members = [
		[BULLETINS, DOCUMENT_ID],
		[LETTERS, documents[2].id],
		[LETTERS, documents[3].id],
		[BULLETINS, documents[3].id]
	];
	links = [
		[DOCUMENT_ID, sample.job.id, sample.job.sources[0].id],
		[DOCUMENT_ID, sample.analysisJob.id, sample.analysisJob.sources[0].id],
		[documents[1].id, sample.audioJob.id, sample.audioJob.sources[0].id]
	];
	documents[0].tags = ['Dakar', 'Pilgrimage'];
	documents[1].tags = ['Interviews'];
	documents[2].tags = ['Dakar'];
	preferred.add(`${DOCUMENT_ID}/${sample.job.setups[1].id}`);
}

/** The tags of some documents, counted, as the store counts them. */
function tagsOf(of: Document[]): { name: string; documents: number }[] {
	const counts = new Map<string, number>();
	for (const tag of of.flatMap((d) => d.tags)) counts.set(tag, (counts.get(tag) ?? 0) + 1);
	return [...counts]
		.map(([name, n]) => ({ name, documents: n }))
		.sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }));
}

function results(documentId: string): DocumentResult[] {
	return links
		.filter(([d]) => d === documentId)
		.flatMap(([, jobId, sourceId]) => {
			const job = jobs.find((j) => j.id === jobId);
			if (!job) return [];
			return job.setups.map((setup, i) => ({
				job_id: job.id,
				job_name: job.name,
				job_status: job.status,
				task: job.task,
				created_at: job.created_at,
				source_id: sourceId,
				setup_id: setup.id,
				setup_label: setup.label,
				provider: setup.provider,
				model: setup.model,
				pieces: 3,
				finished: 3,
				failed: 0,
				reviewed: i === 1 ? 3 : 1,
				corrected: i === 1 ? 1 : 0,
				cost_usd: 0.004 + i * 0.03,
				preferred: preferred.has(`${documentId}/${setup.id}`)
			}));
		});
}

function view(id: string): ProjectView {
	const project = projects.find((p) => p.id === id) ?? projects[0];
	const mine = documents.filter((d) => d.project_id === project.id);
	return {
		project,
		collections: collections
			.filter((c) => c.project_id === project.id)
			.sort((a, b) => a.name.localeCompare(b.name))
			.map((collection) => ({
				collection,
				documents: members.filter(([c]) => c === collection.id).length
			})),
		documents: mine.map((document) => {
			const rs = results(document.id);
			const tasks = [...new Set(rs.map((r) => r.task))].map((task) => {
				const of = rs.filter((r) => r.task === task);
				return {
					task,
					results: of.length,
					finished: of.filter((r) => r.finished === r.pieces).length,
					reviewed: of.filter((r) => r.reviewed === r.pieces).length,
					preferred: of.some((r) => r.preferred)
				};
			});
			return {
				document,
				collections: members.filter(([, d]) => d === document.id).map(([c]) => c),
				tasks,
				missing: document.source.file_name.startsWith('lettre-1912'),
				same_content: 0,
				last_run: rs.length > 0 ? Math.max(...rs.map((r) => r.created_at)) : null
			};
		}),
		tags: tagsOf(mine),
		jobs: [...new Set(links.map(([, j]) => j))]
	};
}

/** Answers a project command, or `undefined` for any other. */
export function projectCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'list_projects':
			return projects.map((project): ProjectSummary => ({
				project,
				documents: documents.filter((d) => d.project_id === project.id).length,
				collections: collections.filter((c) => c.project_id === project.id).length,
				jobs: project.id === PROJECT_ID ? 3 : 0
			}));
		case 'create_project': {
			const project: Project = {
				id: `00000000-0000-0000-0000-0000000000p${projects.length + 1}`,
				name: String(a.name).trim(),
				description: String(a.description ?? '').trim(),
				archived: false,
				source_defaults: {},
				created_at: Date.now(),
				updated_at: Date.now()
			};
			projects = [project, ...projects];
			return project;
		}
		case 'update_project':
		case 'archive_project': {
			projects = projects.map((p) =>
				p.id !== a.id
					? p
					: cmd === 'archive_project'
						? { ...p, archived: Boolean(a.archived) }
						: { ...p, name: String(a.name), description: String(a.description) }
			);
			return projects.find((p) => p.id === a.id);
		}
		case 'delete_project':
			projects = projects.filter((p) => p.id !== a.id);
			return null;
		case 'project_view':
			return view(String(a.id));
		case 'create_collection': {
			const c: Collection = {
				id: `00000000-0000-0000-0000-0000000000c${collections.length + 1}`,
				project_id: String(a.project),
				name: String(a.name),
				created_at: Date.now()
			};
			collections = [...collections, c];
			return c;
		}
		case 'rename_collection':
			collections = collections.map((c) => (c.id === a.id ? { ...c, name: String(a.name) } : c));
			return null;
		case 'delete_collection':
			collections = collections.filter((c) => c.id !== a.id);
			members = members.filter(([c]) => c !== a.id);
			return null;
		case 'add_documents': {
			const added = (a.sources as SourceFile[]).map((source, i) => {
				const document: Document = {
					id: `00000000-0000-0000-0000-00000000${String(documents.length + i).padStart(4, '0')}`,
					project_id: String(a.project),
					title: source.file_name.replace(/\.[^.]+$/, ''),
					source,
					created_at: Date.now(),
					tags: [],
					source_description: {}
				};
				return document;
			});
			documents = [...documents, ...added];
			if (a.collection)
				members.push(...added.map((d) => [String(a.collection), d.id] as [string, string]));
			return added.map((d) => d.id);
		}
		case 'add_to_collection':
			// As the store: a document is in a collection once.
			for (const id of a.ids as string[])
				if (!members.some(([c, d]) => c === a.collection && d === id))
					members.push([String(a.collection), id]);
			return null;
		case 'remove_from_collection':
			members = members.filter(
				([c, d]) => !(c === a.collection && (a.ids as string[]).includes(d))
			);
			return null;
		case 'tag_documents': {
			const typed = String(a.tag).trim().replace(/\s+/g, ' ');
			if (!typed) throw { id: 'project.no_tag', detail: '' };
			const spelled =
				documents.flatMap((d) => d.tags).find((t) => t.toLowerCase() === typed.toLowerCase()) ??
				typed;
			documents = documents.map((d) =>
				(a.ids as string[]).includes(d.id) && !d.tags.includes(spelled)
					? { ...d, tags: [...d.tags, spelled].sort((x, y) => x.localeCompare(y)) }
					: d
			);
			return null;
		}
		case 'untag_documents':
			documents = documents.map((d) =>
				(a.ids as string[]).includes(d.id)
					? { ...d, tags: d.tags.filter((t) => t.toLowerCase() !== String(a.tag).toLowerCase()) }
					: d
			);
			return null;
		case 'delete_documents':
			documents = documents.filter((d) => !(a.ids as string[]).includes(d.id));
			return null;
		case 'rename_document':
			documents = documents.map((d) => (d.id === a.id ? { ...d, title: String(a.title) } : d));
			return null;
		case 'document_detail': {
			const document = documents.find((d) => d.id === a.id) ?? documents[0];
			const detail: DocumentDetail = {
				document,
				collections: members.filter(([, d]) => d === document.id).map(([c]) => c),
				missing: false,
				same_content: [],
				results: results(document.id),
				described: described(document)
			};
			return detail;
		}
		case 'describe_document':
			documents = documents.map((d) =>
				d.id === a.id ? { ...d, source_description: kept(a.description as Draft) } : d
			);
			return null;
		case 'set_source_defaults': {
			projects = projects.map((p) =>
				p.id === a.id ? { ...p, source_defaults: kept(a.defaults as Draft) } : p
			);
			return projects.find((p) => p.id === a.id);
		}
		case 'rename_tag': {
			// As `Projects::rename_tag`: spaces tidied, a tag in use keeping its spelling.
			const from = String(a.from).toLowerCase();
			const typed = String(a.to).split(/\s+/).filter(Boolean).join(' ');
			if (!typed) throw { id: 'project.no_tag' };
			const inUse = documents
				.flatMap((d) => d.tags)
				.find((t) => t.toLowerCase() === typed.toLowerCase());
			const to = typed.toLowerCase() === from ? typed : (inUse ?? typed);
			let changed = 0;
			documents = documents.map((d) => {
				if (!d.tags.some((t) => t.toLowerCase() === from)) return d;
				changed += 1;
				const rest = d.tags.filter((t) => ![from, to.toLowerCase()].includes(t.toLowerCase()));
				return { ...d, tags: [...rest, to].sort((x, y) => x.localeCompare(y)) };
			});
			return { changed, name: to };
		}
		case 'export_descriptions':
			return null;
		case 'set_preferred':
			for (const key of [...preferred]) if (key.startsWith(`${a.document}/`)) preferred.delete(key);
			if (a.setup) preferred.add(`${a.document}/${a.setup}`);
			return null;
		case 'attach_plan': {
			const job = jobs.find((j) => j.id === a.job);
			return (job?.sources ?? []).map((s) => {
				const linked = links.find(([, j, src]) => j === a.job && src === s.id)?.[0] ?? null;
				const proposed = documents.find((d) => d.source.path === s.path)?.id ?? null;
				return {
					source_id: s.id,
					file_name: s.file_name,
					path: s.path,
					linked,
					proposed,
					same_content: []
				};
			});
		}
		case 'attach_job':
		case 'link_job':
		case 'detach_job':
		case 'merge_documents':
			return [];
		default:
			return dictionaryCommand(cmd, a);
	}
}
