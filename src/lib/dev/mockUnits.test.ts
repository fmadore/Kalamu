import { describe, expect, it } from 'vitest';
import type { DocumentUnit } from '../bindings/DocumentUnit.js';
import type { PieceRef } from '../bindings/PieceRef.js';
import { contentsJob, contentsSource } from './mockContents.js';
import { formatPages, parsePages, unitCommand } from './mockUnits.js';

const page = (number: number, half?: 'left' | 'right'): PieceRef =>
	half ? { kind: 'page', number, half } : { kind: 'page', number };
const spread = [page(7), page(8, 'left'), page(8, 'right'), page(9)];

describe('the items mock reads pages as the engine does', () => {
	it('takes a half page, or the page when it is not split', () => {
		expect(parsePages('8 (left)', spread)).toEqual([page(8, 'left')]);
		expect(parsePages('8', spread)).toEqual([page(8, 'left'), page(8, 'right')]);
		expect(parsePages('9 (right)', spread)).toEqual([page(9)]);
		expect(parsePages('2 (left), 1')).toEqual([page(2), page(1)]);
	});

	it('reads ranges, page marks and dashes, each page once', () => {
		expect(parsePages('pp. 1–2; 2, 3')).toEqual([page(1), page(2), page(3)]);
	});

	it('names what it cannot read', () => {
		expect(() => parsePages('4')).toThrow(expect.objectContaining({ id: 'unit.no_such_page' }));
		expect(() => parsePages('8 (left)')).toThrow(
			expect.objectContaining({ id: 'unit.no_such_page', detail: '8 (left)' })
		);
		expect(() => parsePages('3-1')).toThrow(
			expect.objectContaining({ id: 'unit.pages_unreadable' })
		);
		expect(() => parsePages(' , ')).toThrow(expect.objectContaining({ id: 'unit.pages_empty' }));
	});

	it('writes pages back as they are typed', () => {
		expect(formatPages([page(8, 'left'), page(8, 'right')])).toBe('8');
		expect(formatPages([page(1), page(2), page(3), page(5, 'left')])).toBe('1-3, 5 (left)');
	});
});

describe('the items mock counts revisions as the engine does', () => {
	const save = (title: string, pages: string) =>
		unitCommand('save_unit', {
			edit: {
				job_id: contentsJob.id,
				setup_id: contentsJob.setups[0].id,
				source_id: contentsSource.id,
				title,
				pages
			}
		}) as DocumentUnit;

	it('gives the item kept a new revision when joining and splitting', () => {
		const first = save('Article', '1');
		const second = save('Continued', '2 (left)');
		expect(first.revision).toBe(1);
		const joined = unitCommand('join_units', { first: first.id, second: second.id });
		expect(joined).toMatchObject({ id: first.id, revision: 2, pages: [page(1), page(2)] });
		const [kept, cut] = unitCommand('split_unit', { id: first.id, at: 1 }) as DocumentUnit[];
		expect(kept).toMatchObject({ revision: 3, pages: [page(1)] });
		expect(cut).toMatchObject({ revision: 1, pages: [page(2)] });
	});

	it('refuses a join with itself and a split outside the pages, as the engine does', () => {
		const one = save('Alone', '3');
		expect(() => unitCommand('join_units', { first: one.id, second: one.id })).toThrow(
			expect.objectContaining({ id: 'unit.not_joinable' })
		);
		expect(() => unitCommand('split_unit', { id: one.id, at: 1 })).toThrow(
			expect.objectContaining({ id: 'unit.bad_split' })
		);
	});
});
