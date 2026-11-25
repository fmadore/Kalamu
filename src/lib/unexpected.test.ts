import { describe, expect, it } from 'vitest';
import { BURST_MS, beginsBurst, faultReport } from './unexpected.js';

describe('a failure nobody expected', () => {
	it('sends an error’s name, message and stack', () => {
		const error = new TypeError('x is undefined');
		const report = faultReport('error', error, 'review');
		expect(report).toMatchObject({
			kind: 'error',
			screen: 'review',
			name: 'TypeError',
			message: 'x is undefined'
		});
		expect(report.stack).toContain('unexpected.test');
	});

	it('sends only the id of the core’s error, and only the type of anything else thrown', () => {
		expect(
			faultReport('rejection', { id: 'job.not_found', detail: 'C:\\Archives\\a.pdf' }, 'jobs')
		).toEqual({
			kind: 'rejection',
			screen: 'jobs',
			name: 'AppError',
			message: 'job.not_found',
			stack: ''
		});
		expect(faultReport('rejection', 'Le Gouverneur général', 'review')).toMatchObject({
			name: 'string',
			message: ''
		});
		expect(faultReport('error', null, 'review').name).toBe('null');
	});

	it('cuts a long message before sending it', () => {
		expect(faultReport('error', new Error('a'.repeat(5000)), 'review').message).toHaveLength(1000);
	});

	it('is said once for a burst, again once the burst is over', () => {
		const start = 1_000_000;
		expect(beginsBurst(start)).toBe(true);
		expect(beginsBurst(start + 1000)).toBe(false);
		// Each failure keeps the burst going.
		expect(beginsBurst(start + 1000 + BURST_MS - 1)).toBe(false);
		expect(beginsBurst(start + 1000 + 2 * BURST_MS)).toBe(true);
	});
});
