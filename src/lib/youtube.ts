// YouTube videos transcribed from their link. Checking a link is the engine's job
// (`inspect_links`); this only splits what was pasted and points at a moment in a video.

/** How the catalogue marks a model that reads YouTube links (`YOUTUBE_LINK` in the core). */
export const YOUTUBE_LINK = 'video/x-youtube';

/** The links in pasted text: one per line, or separated by spaces or commas. */
export function splitLinks(text: string): string[] {
	return [...new Set(text.split(/[\s,]+/).filter(Boolean))];
}

/** A video's address opened at a moment, as YouTube's own "copy link at time" writes it. */
export function videoAt(url: string, ms: number): string {
	const seconds = Math.max(0, Math.floor(ms / 1000));
	return `${url}${url.includes('?') ? '&' : '?'}t=${seconds}s`;
}

/** Where the embedded player lives: YouTube's privacy-enhanced domain, the one the app allows. */
export const EMBED_ORIGIN = 'https://www.youtube-nocookie.com';

/**
 * The embedded player for a video's canonical address (`watch?v=<id>`), driven by messages:
 * `enablejsapi` lets the review seek it and follow the time, `origin` names the app to it, and
 * `hl` speaks the interface's language. `start` and `autoplay` open it playing at a moment.
 */
export function embedUrl(
	watchUrl: string,
	origin: string,
	options: { language?: string; startMs?: number; autoplay?: boolean } = {}
): string | null {
	const id = new URL(watchUrl).searchParams.get('v');
	if (!id) return null;
	const query = new URLSearchParams({ enablejsapi: '1', rel: '0', playsinline: '1', origin });
	if (options.language) query.set('hl', options.language);
	if (options.startMs) query.set('start', String(Math.floor(options.startMs / 1000)));
	if (options.autoplay) query.set('autoplay', '1');
	return `${EMBED_ORIGIN}/embed/${id}?${query}`;
}

/** A command for the embedded player, in the form its message API reads. */
export function playerCommand(func: string, args: unknown[] = []): string {
	return JSON.stringify({ event: 'command', func, args, id: 1, channel: 'widget' });
}

/** What a message from the embedded player says: its time, whether it plays, or an error. */
export interface PlayerNews {
	/** Seconds into the video. */
	time?: number;
	playing?: boolean;
	/** YouTube's error code: 101 and 150 mean the owner does not allow embedding. */
	error?: number;
}

export function playerNews(data: unknown): PlayerNews | null {
	try {
		const message = typeof data === 'string' ? JSON.parse(data) : data;
		if (typeof message !== 'object' || message === null) return null;
		if (message.event === 'onError') return { error: Number(message.info) };
		const info = message.info ?? {};
		const news: PlayerNews = {};
		if (typeof info.currentTime === 'number') news.time = info.currentTime;
		// 1 is playing, 3 buffering; anything else is stopped.
		if (typeof info.playerState === 'number') news.playing = [1, 3].includes(info.playerState);
		return news;
	} catch {
		return null;
	}
}
