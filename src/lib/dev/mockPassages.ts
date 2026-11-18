// Passages corrected by a pretend model in the browser preview, as the engine does it
// (`engine::passage`): checked against the text as the review shows it, answered after a
// moment unless stopped, kept so the same request again costs nothing, and written into the
// piece's corrections only when accepted. The "model" puts straight apostrophes right and
// mends a few misreadings. Development builds only.

import type { Correction } from '../bindings/Correction.js';
import type { PassageAsk } from '../bindings/PassageAsk.js';
import type { PassageProposal } from '../bindings/PassageProposal.js';
import type { PieceView } from '../bindings/PieceView.js';
import { passageProblem } from '../passage.js';

/** A piece whose corrections the preview keeps, by set-up, source and index. */
export type FindPiece = (setupId: string, sourceId: string, index: number) => PieceView | undefined;

const proposals = new Map<string, PassageProposal>();
/** Answers already given, by what was asked. */
const answered = new Map<string, string>();
/** Requests waiting, by id, to stop waiting for. */
const waiting = new Map<string, () => void>();

const keyOf = (a: PassageAsk) =>
	JSON.stringify([a.provider, a.model, a.preset_id, a.before, a.passage, a.after]);

/** Where the passage starts in `text`, as `core::passage_correction::locate` finds it. */
function locate(text: string, a: PassageAsk): number | null {
	const whole = a.before + a.passage + a.after;
	const starts: number[] = [];
	for (let i = text.indexOf(whole); i >= 0; i = text.indexOf(whole, i + 1)) {
		starts.push(i + a.before.length);
	}
	return starts.find((s) => s === a.at) ?? (starts.length === 1 ? starts[0] : null);
}

function correct(passage: string): string {
	return passage
		.replace(/(\p{L})'(\p{L})/gu, '$1’$2')
		.replace(/fideles/g, 'fidèles')
		.replace(/Gouvemeur/g, 'Gouverneur');
}

const fail = (id: string): never => {
	throw { id };
};

function check(a: PassageAsk, find: FindPiece): PieceView {
	const problem = passageProblem(a.passage);
	if (problem) fail(problem === 'short' ? 'passage.too_short' : 'passage.too_long');
	const piece = find(a.setup_id, a.source_id, a.index) ?? fail('job.nothing_read');
	if (locate(piece.corrected_text ?? piece.ai_text ?? '', a) === null) fail('passage.changed');
	return piece;
}

function proposal(a: PassageAsk): PassageProposal {
	return {
		ask: a,
		status: 'asked',
		prompt: {
			preset_id: a.preset_id,
			preset_name: 'Printed document or newspaper',
			template: '',
			text: '',
			user_prompt: ''
		},
		request: [],
		cache_key: keyOf(a),
		answer: null,
		notes: [],
		cost_usd: null,
		cached: false,
		error: null,
		accepted: null,
		asked_at: Date.now(),
		answered_at: null,
		decided_at: null
	};
}

export function passageCommand(
	cmd: string,
	a: Record<string, unknown>,
	find: FindPiece,
	local: (provider: string) => boolean
): unknown {
	switch (cmd) {
		case 'passage_estimate': {
			const ask = a.ask as PassageAsk;
			check(ask, find);
			const offline = local(ask.provider);
			const cached = answered.has(keyOf(ask));
			return { usd: offline || cached ? null : 0.0004, offline, cached };
		}
		case 'ask_passage': {
			const ask = a.ask as PassageAsk;
			try {
				check(ask, find);
			} catch (e) {
				return Promise.reject(e);
			}
			const p = proposal(ask);
			proposals.set(ask.id, p);
			const stored = answered.get(keyOf(ask));
			// `?passage-slow`: the model takes long enough to be stopped.
			const slow = new URLSearchParams(location.search).has('passage-slow');
			return new Promise<PassageProposal>((resolve) => {
				const done = (status: 'answered' | 'cancelled') => {
					waiting.delete(ask.id);
					clearTimeout(timer);
					const answer = stored ?? correct(ask.passage);
					if (status === 'answered') answered.set(keyOf(ask), answer);
					Object.assign(p, {
						status,
						answer: status === 'answered' ? answer : null,
						notes: status === 'answered' && answer === ask.passage ? ['unchanged'] : [],
						cost_usd: status === 'answered' && !stored ? 0.0004 : null,
						cached: stored !== undefined,
						answered_at: Date.now()
					});
					resolve({ ...p });
				};
				const timer = setTimeout(
					() => done('answered'),
					stored !== undefined ? 0 : slow ? 60_000 : 400
				);
				waiting.set(ask.id, () => done('cancelled'));
			});
		}
		case 'cancel_passage':
			waiting.get(String(a.id))?.();
			return null;
		case 'accept_passage': {
			const p = proposals.get(String(a.id));
			if (!p || p.status !== 'answered') return fail('passage.not_waiting');
			const ask = p.ask;
			const piece = find(ask.setup_id, ask.source_id, ask.index) ?? fail('job.nothing_read');
			const shown = piece.corrected_text ?? piece.ai_text ?? '';
			const at = locate(shown, ask);
			if (at === null) return fail('passage.changed');
			const text = String(a.text);
			const before: Correction = {
				job_id: ask.job_id,
				setup_id: ask.setup_id,
				source_id: ask.source_id,
				index: ask.index,
				text: piece.corrected_text,
				reviewed: piece.reviewed,
				updated_at: 0
			};
			piece.corrected_text = shown.slice(0, at) + text + shown.slice(at + ask.passage.length);
			Object.assign(p, { status: 'accepted', accepted: text, decided_at: Date.now() });
			return before;
		}
		case 'discard_passage': {
			const p = proposals.get(String(a.id));
			if (p?.status === 'answered') p.status = 'discarded';
			return null;
		}
		case 'undo_passage': {
			const p = proposals.get(String(a.id));
			const before = a.before as Correction;
			if (!p || p.status !== 'accepted') return fail('passage.not_waiting');
			const piece = find(before.setup_id, before.source_id, before.index);
			if (piece) {
				piece.corrected_text = before.text;
				piece.reviewed = before.reviewed;
			}
			p.status = 'undone';
			return null;
		}
		default:
			return undefined;
	}
}
