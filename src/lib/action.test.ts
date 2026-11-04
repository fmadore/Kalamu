import { describe, expect, it } from 'vitest';
import { Action } from './action.svelte.js';
import { en } from './i18n/en.js';

describe('an action on a screen', () => {
	it('runs the work, then reads again, busy throughout', async () => {
		const seen: string[] = [];
		const action: Action = new Action({
			after: () => seen.push(`after, busy ${action.busy}`)
		});
		const ok = await action.run(async () => {
			seen.push(`work, busy ${action.busy}`);
		});
		expect(ok).toBe(true);
		expect(seen).toEqual(['work, busy true', 'after, busy true']);
		expect(action.busy).toBe(false);
	});

	it('says what failed in the interface language, and does not read again', async () => {
		let after = 0;
		const action = new Action({ after: () => after++ });
		const ok = await action.run(() => Promise.reject({ id: 'job.not_found' }));
		expect(ok).toBe(false);
		expect(after).toBe(0);
		expect(action.error).toBe(en.error['job.not_found']);
		expect(action.busy).toBe(false);
	});

	it('clears the last error as the next work starts', async () => {
		const action = new Action();
		await action.run(() => Promise.reject(new Error('no disk')));
		expect(action.error).toBe('no disk');
		await action.run(async () => {
			expect(action.error).toBeNull();
		});
		expect(action.error).toBeNull();
	});

	it('passes the error up instead, when the screen shows it', async () => {
		const reported: string[] = [];
		const action = new Action({ report: (message) => reported.push(message) });
		await action.run(() => Promise.reject(new Error('locked')));
		expect(reported).toEqual(['locked']);
		expect(action.error).toBeNull();
	});

	it('follows a work with what it is given, the work’s result included, or with nothing', async () => {
		const seen: unknown[] = [];
		const action = new Action({ after: () => seen.push('default') });
		await action.run(
			async () => 'kept',
			(result) => seen.push(result)
		);
		await action.run(async () => 1, null);
		expect(seen).toEqual(['kept']);
	});

	it('counts a failure while reading again as the work’s', async () => {
		const action = new Action({ after: () => Promise.reject(new Error('gone')) });
		expect(await action.run(async () => undefined)).toBe(false);
		expect(action.error).toBe('gone');
	});
});
