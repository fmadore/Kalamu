// Draws the one page of a page preview (a single-page PDF from `piece_preview`) with pdf.js,
// for the source pane and for the thumbnails in the review's strip of pages.

function bytes(b64: string): Uint8Array {
	const raw = atob(b64);
	const out = new Uint8Array(raw.length);
	for (let i = 0; i < raw.length; i++) out[i] = raw.charCodeAt(i);
	return out;
}

/**
 * Renders the page onto `target`, `width` CSS pixels wide. Sharp on high-density screens: drawn
 * at the device pixel ratio, displayed at CSS size.
 */
export async function renderPdfPage(
	data: string,
	target: HTMLCanvasElement,
	width: number
): Promise<void> {
	const pdfjs = await import('pdfjs-dist');
	const worker = await import('pdfjs-dist/build/pdf.worker.min.mjs?url');
	pdfjs.GlobalWorkerOptions.workerSrc = worker.default;
	// Decoders, colour profiles, character maps and fonts, served by vite.config.js.
	const at = (folder: string) => new URL(`/pdfjs/${folder}/`, window.location.href).href;
	const task = pdfjs.getDocument({
		data: bytes(data),
		wasmUrl: at('wasm'),
		cMapUrl: at('cmaps'),
		standardFontDataUrl: at('standard_fonts'),
		iccUrl: at('iccs')
	});
	try {
		const page = await (await task.promise).getPage(1);
		const ratio = window.devicePixelRatio || 1;
		const natural = page.getViewport({ scale: 1 });
		const viewport = page.getViewport({ scale: (width / natural.width) * ratio });
		target.width = viewport.width;
		target.height = viewport.height;
		target.style.width = `${viewport.width / ratio}px`;
		await page.render({ canvas: target, viewport }).promise;
	} finally {
		await task.destroy();
	}
}
