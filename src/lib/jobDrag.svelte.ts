// Dragging a finished job from the sidebar onto a project, which adds it to that project as the
// job's menu does. Done with the pointer rather than the browser's own drag and drop, which the
// window gives over to files dropped from File Explorer or Finder.

import type { JobSummary } from './bindings/JobSummary.js';
import { describeError } from './errors.js';
import { messages } from './i18n/index.js';
import { app, refreshProjects, toast } from './state.svelte.js';
import { api } from './tauri.js';

/** A job pressed, held, or carried, and where. */
type Held = { job: JobSummary; x: number; y: number };

export class JobDrag {
	/** The job carried and where the pointer is, once it moved far enough to be a drag. */
	dragging = $state<Held | null>(null);
	/** The project under the pointer, by its id. */
	dropOn = $state<string | null>(null);
	/** Set when a drag ends, so the click that follows it does not open the job. */
	dragged = false;
	#press: Held | null = null;
	readonly #onstart: () => void;

	/** `onstart` runs as a drag begins: the projects are unfolded to drop on. */
	constructor(onstart: () => void) {
		this.#onstart = onstart;
	}

	press = (e: PointerEvent, job: JobSummary): void => {
		if (e.button !== 0 || e.pointerType === 'touch') return;
		this.#press = { job, x: e.clientX, y: e.clientY };
	};

	move = (e: PointerEvent): void => {
		const press = this.#press;
		if (!press) return;
		if (!this.dragging) {
			if (Math.hypot(e.clientX - press.x, e.clientY - press.y) < 6) return;
			this.#onstart();
		}
		this.dragging = { job: press.job, x: e.clientX, y: e.clientY };
		const under = document.elementFromPoint(e.clientX, e.clientY);
		this.dropOn = under?.closest<HTMLElement>('[data-project]')?.dataset.project ?? null;
	};

	/** Ends a drag; the click the browser sends after it is swallowed. */
	end = (): void => {
		const was = this.dragging !== null;
		this.#press = null;
		this.dragging = null;
		this.dropOn = null;
		if (!was) return;
		this.dragged = true;
		setTimeout(() => (this.dragged = false));
	};

	/** Adds the job carried to the project it was let go on, if any. */
	drop = async (): Promise<void> => {
		const drag = this.dragging;
		const target = app.projects.find((p) => p.project.id === this.dropOn);
		this.end();
		if (!drag || !target) return;
		const m = messages();
		try {
			await api.linkJob(target.project.id, drag.job.id, null);
			toast(m.shell.addedTo(target.project.name));
			await refreshProjects();
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		}
	};
}
