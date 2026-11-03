// Project search for the browser preview (`?mock&view=project`, then Search): the sample
// bulletin's two readings and the sample interview, searched as the search crate searches them,
// simplified (no index, no analyses). Development builds only.

import type { DocumentHits } from '../bindings/DocumentHits.js';
import type { Hit } from '../bindings/Hit.js';
import type { PieceView } from '../bindings/PieceView.js';
import type { ResultHits } from '../bindings/ResultHits.js';
import type { SearchRequest } from '../bindings/SearchRequest.js';
import type { SearchView } from '../bindings/SearchView.js';
import { context, find, folded } from '../search.js';
import { audioJob, audioPieces, job, pieces } from './mockJobs.js';
import { DOCUMENT_ID } from './mockProjects.js';

const INTERVIEW_ID = '00000000-0000-0000-0000-0000000000d2';

function hits(views: PieceView[], query: string): Hit[] {
	return views.flatMap((v) => {
		const text = v.corrected_text ?? v.ai_text ?? '';
		const found = find(text, query);
		const lines = v.segments.flatMap((s) => find(s.text, query).map(() => Number(s.start_ms)));
		const start = v.piece.kind === 'segment' ? Number(v.piece.start_ms) : 0;
		return found.map((m, nth) => ({
			index: v.index,
			piece: v.piece,
			nth,
			field: null,
			entry: null,
			reason: false,
			...context(text, m),
			at_ms:
				v.piece.kind === 'segment' && lines.length === found.length ? start + lines[nth] : null,
			reviewed: v.reviewed,
			corrected: v.corrected_text !== null
		}));
	});
}

function result(
	of: typeof job,
	setupIndex: number,
	views: PieceView[],
	query: string,
	preferred: boolean
): ResultHits {
	const setup = of.setups[setupIndex];
	return {
		job_id: of.id,
		job_name: of.name,
		setup_id: setup.id,
		setup_label: setup.label,
		source_id: of.sources[0].id,
		task: of.task,
		preferred,
		generated: false,
		stale: false,
		hits: hits(
			views.filter((v) => v.setup_id === setup.id),
			query
		)
	};
}

function search(request: SearchRequest): SearchView {
	if ([...folded(request.query)].length < 3) throw { id: 'search.too_short' };
	const all = request.results === 'all';
	const documents: DocumentHits[] = [
		{
			document_id: DOCUMENT_ID,
			title: 'Bulletin, May 1957',
			// The second reading is the preferred one in the sample project.
			results: [
				...(all ? [result(job, 0, pieces, request.query, false)] : []),
				result(job, 1, pieces, request.query, true)
			]
		},
		{
			document_id: INTERVIEW_ID,
			title: 'Interview with the imam',
			results: [result(audioJob, 0, audioPieces(), request.query, false)]
		}
	]
		.map((d) => ({ ...d, results: d.results.filter((r) => r.hits.length > 0) }))
		.filter((d) => d.results.length > 0);
	const matches = documents.flatMap((d) => d.results).reduce((n, r) => n + r.hits.length, 0);
	return {
		documents,
		matches,
		listed: matches,
		coverage: {
			documents: 6,
			searched: 2,
			unread: 4,
			pieces_unread: 0,
			without_preference: 0,
			pieces: 5
		}
	};
}

export function searchCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'project_search':
			return search(a.request as SearchRequest);
		case 'rebuild_search_index':
			return null;
	}
	return undefined;
}
