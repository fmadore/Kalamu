import { describe, expect, it } from 'vitest';
import { embedUrl, playerCommand, playerNews, splitLinks, videoAt } from './youtube.js';

describe('YouTube links', () => {
	it('splits pasted text into links, once each', () => {
		expect(
			splitLinks(
				' https://youtu.be/a\nhttps://youtu.be/b, https://youtu.be/a\t https://youtu.be/c '
			)
		).toEqual(['https://youtu.be/a', 'https://youtu.be/b', 'https://youtu.be/c']);
		expect(splitLinks('  ')).toEqual([]);
	});

	it('opens a video at the start of a line', () => {
		const url = 'https://www.youtube.com/watch?v=dQw4w9WgXcQ';
		expect(videoAt(url, 70_900)).toBe(`${url}&t=70s`);
		expect(videoAt('https://youtu.be/dQw4w9WgXcQ', 0)).toBe('https://youtu.be/dQw4w9WgXcQ?t=0s');
	});

	it('embeds a video from the privacy-enhanced domain and reads the player', () => {
		const url = embedUrl('https://www.youtube.com/watch?v=dQw4w9WgXcQ', 'http://tauri.localhost');
		expect(url).toBe(
			'https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ?enablejsapi=1&rel=0&playsinline=1&origin=http%3A%2F%2Ftauri.localhost'
		);
		expect(
			embedUrl('https://www.youtube.com/watch?v=dQw4w9WgXcQ', 'x', {
				language: 'en',
				startMs: 99_500,
				autoplay: true
			})
		).toContain('&hl=en&start=99&autoplay=1');
		expect(embedUrl('https://www.youtube.com/', 'x')).toBeNull();
		expect(JSON.parse(playerCommand('seekTo', [70, true]))).toMatchObject({
			event: 'command',
			func: 'seekTo',
			args: [70, true]
		});
		expect(
			playerNews('{"event":"infoDelivery","info":{"currentTime":12.5,"playerState":1}}')
		).toEqual({ time: 12.5, playing: true });
		expect(playerNews('{"event":"onError","info":150}')).toEqual({ error: 150 });
		expect(playerNews('{"event":"onReady"}')).toEqual({});
		expect(playerNews('not json')).toBeNull();
	});
});
