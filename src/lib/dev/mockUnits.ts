// The sample table of contents' items (#50) for the browser preview, kept in memory: made from
// its three entries, corrected, joined, split and sent on as the desktop app's engine would.
// Page lists are read as `kalamu_core::units::parse_pages` reads them: numbers, ranges and a
// half page such as `8 (left)`, among the source's pages; joining and splitting count a new
// revision of the item kept, as `Engine::join_units` and `split_unit` do.

import type { DocumentUnit } from '../bindings/DocumentUnit.js';
import type { FieldAnswer } from '../bindings/FieldAnswer.js';
import type { PageHalf } from '../bindings/PageHalf.js';
import type { PieceRef } from '../bindings/PieceRef.js';
import type { SourceFile } from '../bindings/SourceFile.js';
import type { UnitEdit } from '../bindings/UnitEdit.js';
import type { UnitsView } from '../bindings/UnitsView.js';
import { contentsJob, contentsPieces, contentsSource } from './mockContents.js';

let units: DocumentUnit[] = [];
let next = 1;

/** The source's pages, as its pieces name them. */
const PIECES: PieceRef[] = contentsPieces.map((v) => v.piece).filter((p) => p.kind === 'page');

const same = (a: PieceRef, b: PieceRef) => JSON.stringify(a) === JSON.stringify(b);

/** A page as the engine writes it: `8`, `8 (left)`. */
const label = (page: PieceRef) =>
	page.kind !== 'page' ? '' : page.half ? `${page.number} (${page.half})` : String(page.number);

/** `"8 (left)"` → `["8", "left"]`; `"3-5"` → `["3-5", undefined]`. */
function splitHalf(item: string): [string, PageHalf | undefined] {
	for (const half of ['left', 'right'] as const) {
		const at = item.toLowerCase().indexOf(half);
		if (at >= 0) return [item.slice(0, at).trimEnd().replace(/\(+$/, '').trimEnd(), half];
	}
	return [item, undefined];
}

/** `"3-5"`, `"3–5"`, `"p. 3"`, `"pp. 3-5"` → `[3, 5]`; a reversed range is unreadable. */
function readRange(range: string): [number, number] | null {
	const ends = range
		.trim()
		.replace(/^[\p{L}.\s]+/u, '')
		.split(/[-–—]/)
		.map((n) => n.trim());
	if (ends.length > 2 || !ends.every((n) => /^\d+$/.test(n))) return null;
	const first = Number(ends[0]);
	const last = Number(ends[1] ?? ends[0]);
	return first >= 1 && first <= last ? [first, last] : null;
}

/** The pages a list names among the source's, in the order written and each once. A page split
 * into halves is both halves unless one is named; a page not split is the page, whichever half
 * is named. */
export function parsePages(text: string, pieces: PieceRef[] = PIECES): PieceRef[] {
	const pages: PieceRef[] = [];
	for (const raw of text.split(/[,;]/)) {
		const item = raw.trim();
		if (!item) continue;
		const [range, half] = splitHalf(item);
		const read = readRange(range);
		if (!read) throw { id: 'unit.pages_unreadable', detail: `“${item}”` };
		for (let number = read[0]; number <= read[1]; number++) {
			const found = pieces.filter(
				(p) => p.kind === 'page' && p.number === number && (!half || !p.half || p.half === half)
			);
			if (found.length === 0) {
				throw { id: 'unit.no_such_page', detail: label({ kind: 'page', number, half }) };
			}
			for (const piece of found) if (!pages.some((p) => same(p, piece))) pages.push(piece);
		}
	}
	if (pages.length === 0) throw { id: 'unit.pages_empty' };
	return pages;
}

/** The pages as the researcher reads and types them: `1, 3-5, 8 (left)`; both halves of a page
 * in order read as the page. */
export function formatPages(pages: PieceRef[]): string {
	const items: [number, PageHalf | undefined][] = [];
	for (const page of pages) {
		if (page.kind !== 'page') continue;
		const last = items.at(-1);
		if (page.half === 'right' && last?.[0] === page.number && last[1] === 'left') {
			items[items.length - 1] = [page.number, undefined];
		} else items.push([page.number, page.half]);
	}
	const out: string[] = [];
	for (let i = 0; i < items.length; i++) {
		const [first, half] = items[i];
		if (half) {
			out.push(label({ kind: 'page', number: first, half }));
			continue;
		}
		let last = first;
		while (i + 1 < items.length && items[i + 1][0] === last + 1 && !items[i + 1][1]) {
			last++;
			i++;
		}
		out.push(last > first ? `${first}-${last}` : String(first));
	}
	return out.join(', ');
}

/** Whether `pages` cover any of `page`: a whole page and either of its halves overlap. */
function overlaps(pages: PieceRef[], page: PieceRef): boolean {
	if (page.kind !== 'page') return false;
	return pages.some(
		(p) =>
			p.kind === 'page' &&
			p.number === page.number &&
			(!p.half || !page.half || p.half === page.half)
	);
}

function unit(title: string, pages: PieceRef[], position: number, fields: FieldAnswer[] = []) {
	const id = `00000000-0000-0000-0000-00000000u${String(next++).padStart(3, '0')}`;
	return {
		id,
		job_id: contentsJob.id,
		setup_id: contentsJob.setups[0].id,
		source_id: contentsSource.id,
		position,
		title,
		fields,
		pages,
		entry: null,
		revision: 1,
		created_at: Date.now(),
		updated_at: Date.now()
	} satisfies DocumentUnit;
}

function view(): UnitsView {
	const sorted = [...units].sort((a, b) => a.position - b.position);
	return {
		units: sorted.map((u) => ({
			unit: u,
			pages: formatPages(u.pages),
			shared: u.pages.flatMap((page) => {
				const others = sorted.filter((o) => o.id !== u.id && overlaps(o.pages, page));
				return others.length > 0 ? [{ page, with: others.map((o) => o.id) }] : [];
			})
		})),
		entries: 3,
		readable: true
	};
}

/** The combined entries' titles, pages and other fields. */
function entries(): { title: string; pages: string; fields: FieldAnswer[] }[] {
	const whole = contentsPieces.find((p) => p.piece.kind === 'whole');
	const list = (whole?.ai_fields[0]?.value ?? []) as FieldAnswer[][];
	return list.map((answers) => ({
		title: String(answers.find((a) => a.key === 'title')?.value ?? ''),
		pages: String(answers.find((a) => a.key === 'pages')?.value ?? ''),
		fields: answers.filter((a) => !['title', 'pages', 'continues'].includes(a.key))
	}));
}

function frozen(u: DocumentUnit): SourceFile {
	const n = String(u.position + 1).padStart(2, '0');
	return {
		...contentsSource,
		id: `${u.id}-pdf`,
		path: `C:\\Users\\you\\AppData\\Local\\Kalamu Data\\snapshots\\${u.id}.pdf`,
		file_name: `an-nasr-12 ${n} ${u.title.replace(/[[\]:?"]/g, '').trim()}.pdf`,
		page_count: u.pages.length,
		derived_from: {
			snapshot_id: `snapshot-${u.id}`,
			job_id: u.job_id,
			setup_id: u.setup_id,
			source_id: u.source_id,
			original_path: contentsSource.path,
			unit: { id: u.id, title: u.title, revision: u.revision, pages: u.pages }
		}
	};
}

/** Answers the item commands; `undefined` for any other. */
export function unitCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'units_view':
			return view();
		case 'seed_units': {
			if (units.length > 0) throw { id: 'unit.already_made' };
			units = entries().map((e, i) => ({
				...unit(e.title, parsePages(e.pages), i, e.fields),
				entry: i + 1
			}));
			return { made: units.length, problems: [] };
		}
		case 'save_unit': {
			const edit = a.edit as UnitEdit;
			const pages = parsePages(edit.pages);
			const old = units.find((u) => u.id === edit.id);
			if (old) {
				Object.assign(old, {
					title: edit.title.trim(),
					pages,
					revision: old.revision + 1,
					updated_at: Date.now()
				});
				return old;
			}
			const made = unit(
				edit.title.trim(),
				pages,
				Math.max(-1, ...units.map((u) => u.position)) + 1
			);
			units.push(made);
			return made;
		}
		case 'delete_unit':
			units = units.filter((u) => u.id !== a.id);
			return null;
		case 'join_units': {
			const first = units.find((u) => u.id === a.first);
			const second = units.find((u) => u.id === a.second);
			if (!first || !second) throw { id: 'job.not_found' };
			if (first.id === second.id) throw { id: 'unit.not_joinable' };
			first.pages = [
				...first.pages,
				...second.pages.filter((p) => !first.pages.some((q) => same(q, p)))
			];
			first.revision += 1;
			first.updated_at = Date.now();
			units = units.filter((u) => u.id !== second.id);
			return first;
		}
		case 'split_unit': {
			const first = units.find((u) => u.id === a.id);
			const at = Number(a.at);
			if (!first) throw { id: 'job.not_found' };
			if (at < 1 || at >= first.pages.length) throw { id: 'unit.bad_split' };
			for (const u of units) if (u.position > first.position) u.position += 1;
			const second = unit(first.title, first.pages.slice(at), first.position + 1, first.fields);
			first.pages = first.pages.slice(0, at);
			first.revision += 1;
			first.updated_at = Date.now();
			units.push(second);
			return [first, second];
		}
		case 'unit_sources':
			return (a.ids as string[])
				.map((id) => units.find((u) => u.id === id))
				.filter((u) => u !== undefined)
				.sort((x, y) => x.position - y.position)
				.map(frozen);
		case 'export_units':
			return [
				...(a.ids as string[]).map((id) => `${a.folder}\\${id}.pdf`),
				'units.csv',
				'units.json'
			];
		default:
			return undefined;
	}
}
