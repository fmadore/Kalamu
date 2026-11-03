// The sample table of contents' items (#50) for the browser preview, kept in memory: made from
// its three entries, corrected, joined, split and sent on as the desktop app's engine would.
// The page lists are read simply (numbers and ranges); the engine's reading is the real one.

import type { DocumentUnit } from '../bindings/DocumentUnit.js';
import type { FieldAnswer } from '../bindings/FieldAnswer.js';
import type { PieceRef } from '../bindings/PieceRef.js';
import type { SourceFile } from '../bindings/SourceFile.js';
import type { UnitEdit } from '../bindings/UnitEdit.js';
import type { UnitsView } from '../bindings/UnitsView.js';
import { contentsJob, contentsPieces, contentsSource } from './mockContents.js';

let units: DocumentUnit[] = [];
let next = 1;

const PAGES = 3;

function parse(text: string): PieceRef[] {
	const pages: PieceRef[] = [];
	for (const item of text.split(/[,;]/)) {
		const [from, to = from] = item.split('-').map((n) => Number(n.trim()));
		if (!Number.isInteger(from) || !Number.isInteger(to) || from < 1 || to < from) {
			throw { id: 'unit.pages_unreadable', detail: `“${item.trim()}”` };
		}
		for (let n = from; n <= to; n++) {
			if (n > PAGES) throw { id: 'unit.no_such_page', detail: `page ${n}` };
			if (!pages.some((p) => p.kind === 'page' && p.number === n))
				pages.push({ kind: 'page', number: n });
		}
	}
	if (pages.length === 0) throw { id: 'unit.pages_empty', detail: 'no pages' };
	return pages;
}

const typed = (pages: PieceRef[]) =>
	pages.map((p) => (p.kind === 'page' ? String(p.number) : '')).join(', ');

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
			pages: typed(u.pages),
			shared: u.pages.flatMap((page) => {
				const others = sorted.filter(
					(o) =>
						o.id !== u.id &&
						o.pages.some(
							(p) => p.kind === 'page' && page.kind === 'page' && p.number === page.number
						)
				);
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
				...unit(e.title, parse(e.pages), i, e.fields),
				entry: i + 1
			}));
			return { made: units.length, problems: [] };
		}
		case 'save_unit': {
			const edit = a.edit as UnitEdit;
			const pages = parse(edit.pages);
			const old = units.find((u) => u.id === edit.id);
			if (old) {
				Object.assign(old, { title: edit.title.trim(), pages, revision: old.revision + 1 });
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
			first.pages = [
				...first.pages,
				...second.pages.filter(
					(p) => !first.pages.some((q) => JSON.stringify(q) === JSON.stringify(p))
				)
			];
			units = units.filter((u) => u.id !== second.id);
			return first;
		}
		case 'split_unit': {
			const first = units.find((u) => u.id === a.id);
			const at = Number(a.at);
			if (!first || at < 1 || at >= first.pages.length) throw { id: 'unit.pages_empty' };
			for (const u of units) if (u.position > first.position) u.position += 1;
			const second = unit(first.title, first.pages.slice(at), first.position + 1, first.fields);
			first.pages = first.pages.slice(0, at);
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
