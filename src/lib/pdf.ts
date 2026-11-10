// Draws a page of a source: the one page of a page preview (a single-page PDF from
// `piece_preview`) with pdf.js, or an image, for the source pane. The strip's thumbnails are
// drawn small by the app (`piece_thumbnail`) and need none of this.

/** The one pdf.js worker every page opened shares, started with the first: each page opened
 * otherwise starts a worker of its own, a thread and its memory, while the previous one is
 * still being let go. */
type Opened = { pdfjs: typeof import('pdfjs-dist'); worker: import('pdfjs-dist').PDFWorker };
let shared: Promise<Opened> | null = null;

function pdfWorker(): Promise<Opened> {
	shared ??= (async () => {
		const pdfjs = await import('pdfjs-dist');
		const url = await import('pdfjs-dist/build/pdf.worker.min.mjs?url');
		pdfjs.GlobalWorkerOptions.workerSrc = url.default;
		return { pdfjs, worker: new pdfjs.PDFWorker({ name: null }) };
	})().catch((e: unknown) => {
		// A worker that failed to start is tried again with the next page.
		shared = null;
		throw e;
	});
	return shared.then((opened) => {
		if (!opened.worker.destroyed) return opened;
		shared = null;
		return pdfWorker();
	});
}

function bytes(b64: string): Uint8Array {
	const raw = atob(b64);
	const out = new Uint8Array(raw.length);
	for (let i = 0; i < raw.length; i++) out[i] = raw.charCodeAt(i);
	return out;
}

/** A quarter turn clockwise, as the viewer counts them: 0, 90, 180 or 270 degrees. */
export type Rotation = 0 | 90 | 180 | 270;

/** The next rotation a quarter turn clockwise (`1`) or anticlockwise (`-1`). */
export function turn(rotation: Rotation, by: 1 | -1): Rotation {
	return ((((rotation + by * 90) % 360) + 360) % 360) as Rotation;
}

/** The most pixels a drawing may hold, well inside what any webview's canvas allows: zoomed
 * further, the page is drawn at this size and enlarged, a little softer but never blank. */
const MAX_PIXELS = 48_000_000;
const MAX_SIDE = 16_000;

/** The pixel ratio a page `width` CSS pixels wide and `aspect` times as tall is drawn at:
 * the screen's, unless that would pass the limits above. */
function drawRatio(width: number, aspect: number): number {
	const screen = window.devicePixelRatio || 1;
	const height = width * aspect;
	return Math.min(
		screen,
		Math.sqrt(MAX_PIXELS / Math.max(width * height, 1)),
		MAX_SIDE / Math.max(width, height, 1)
	);
}

/** Copies a finished drawing onto the visible canvas, shown at CSS size. */
function show(drawing: HTMLCanvasElement, target: HTMLCanvasElement, ratio: number) {
	target.width = drawing.width;
	target.height = drawing.height;
	target.style.width = `${drawing.width / ratio}px`;
	target.getContext('2d')?.drawImage(drawing, 0, 0);
}

/**
 * A page opened once and drawn at any width and rotation. Each drawing is made on a canvas of
 * its own and copied to the visible one only if no later drawing has started meanwhile: a pane
 * being resized starts several in quick succession, and pdf.js must never draw twice on one
 * canvas at the same time (the second drawing fails, and the first can come out upside down).
 */
export class SourcePage {
	#latest = 0;

	private constructor(
		private readonly page: import('pdfjs-dist').PDFPageProxy | null,
		private readonly image: HTMLImageElement | null,
		private readonly close: () => Promise<void>
	) {}

	/** Opens a preview: a single-page PDF, or an image in a format the browser draws. */
	static async open(mime: string, data: string): Promise<SourcePage> {
		if (mime !== 'application/pdf') {
			const image = new Image();
			image.src = `data:${mime};base64,${data}`;
			await image.decode();
			return new SourcePage(null, image, async () => {});
		}
		const { pdfjs, worker } = await pdfWorker();
		// Decoders, colour profiles, character maps and fonts, served by vite.config.js.
		const at = (folder: string) => new URL(`/pdfjs/${folder}/`, window.location.href).href;
		// Destroying the document leaves the shared worker running for the next page.
		const task = pdfjs.getDocument({
			worker,
			data: bytes(data),
			wasmUrl: at('wasm'),
			cMapUrl: at('cmaps'),
			standardFontDataUrl: at('standard_fonts'),
			iccUrl: at('iccs')
		});
		try {
			const page = await (await task.promise).getPage(1);
			return new SourcePage(page, null, () => task.destroy());
		} catch (e) {
			await task.destroy();
			throw e;
		}
	}

	/**
	 * Draws the page onto `target`, `width` CSS pixels wide once turned by `rotation`. Sharp on
	 * high-density screens: drawn at the device pixel ratio, shown at CSS size; zoomed very far,
	 * at fewer pixels than that (`drawRatio`). Resolves false when a later drawing took its place.
	 */
	async draw(target: HTMLCanvasElement, width: number, rotation: Rotation = 0): Promise<boolean> {
		const ticket = ++this.#latest;
		let ratio = window.devicePixelRatio || 1;
		const drawing = document.createElement('canvas');
		if (this.page) {
			// The page's own rotation, from the PDF, plus the reader's.
			const turned = (this.page.rotate + rotation) % 360;
			const natural = this.page.getViewport({ scale: 1, rotation: turned });
			ratio = drawRatio(width, natural.height / natural.width);
			const viewport = this.page.getViewport({
				scale: (width / natural.width) * ratio,
				rotation: turned
			});
			drawing.width = Math.round(viewport.width);
			drawing.height = Math.round(viewport.height);
			await this.page.render({ canvas: drawing, viewport }).promise;
		} else if (this.image) {
			const sideways = rotation === 90 || rotation === 270;
			const [w, h] = [this.image.naturalWidth, this.image.naturalHeight];
			ratio = drawRatio(width, sideways ? w / h : h / w);
			const scale = (width / (sideways ? h : w)) * ratio;
			drawing.width = Math.round((sideways ? h : w) * scale);
			drawing.height = Math.round((sideways ? w : h) * scale);
			const context = drawing.getContext('2d');
			if (!context) return false;
			context.translate(drawing.width / 2, drawing.height / 2);
			context.rotate((rotation * Math.PI) / 180);
			context.drawImage(this.image, (-w * scale) / 2, (-h * scale) / 2, w * scale, h * scale);
		}
		if (ticket !== this.#latest) return false;
		show(drawing, target, ratio);
		return true;
	}

	destroy(): Promise<void> {
		return this.close();
	}
}
