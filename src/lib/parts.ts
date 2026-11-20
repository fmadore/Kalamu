// A long interview read in parts, as its review shows it: the answers of the step that brought
// the parts together stand for the whole interview, and a part that could not be read is said
// plainly, with the minutes it covers when the transcript has times. The parts themselves are
// the engine's (`PieceRef::Turns`); their answers are never shown.

import type { PieceView } from './bindings/PieceView.js';
import type { Turn } from './bindings/Turn.js';
import { clock } from './pieces.js';

/** A part that could not be read: its number, its turns, its times when known, and why. */
export interface UnreadPart {
	number: number;
	first: number;
	last: number;
	from: string | null;
	to: string | null;
	error: string | null;
}

/** How a set-up read the interview: in how many parts, and those not read; none when it read it
 * whole. */
export interface PartsRead {
	count: number;
	unread: UnreadPart[];
}

/** The piece whose answers stand for the source under a set-up: what brought a long text's
 * parts together, else its first piece. */
export function wholeView(views: PieceView[], setupId: string | undefined): PieceView | undefined {
	const mine = views.filter((v) => v.setup_id === setupId);
	return mine.some((v) => v.piece.kind === 'turns')
		? mine.find((v) => v.piece.kind === 'whole')
		: mine[0];
}

/** The parts a set-up read the source in, with the turns' times to name those not read. */
export function partsOf(
	views: PieceView[],
	setupId: string | undefined,
	turns: Turn[] | undefined
): PartsRead | null {
	const parts = views.filter((v) => v.setup_id === setupId && v.piece.kind === 'turns');
	if (parts.length === 0) return null;
	const at = (turn: number) => {
		const ms = turns?.[turn - 1]?.start_ms;
		return ms === null || ms === undefined ? null : clock(ms);
	};
	const unread = parts.flatMap((v) => {
		if (v.status !== 'failed' || v.piece.kind !== 'turns') return [];
		const { number, first, last } = v.piece;
		const end = turns?.[last - 1]?.end_ms;
		return [
			{
				number,
				first,
				last,
				from: at(first),
				to: at(last + 1) ?? (end === null || end === undefined ? null : clock(end)),
				error: v.error
			}
		];
	});
	return { count: parts.length, unread };
}
