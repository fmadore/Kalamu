// Draws a page of a source: the one page of a page preview (a single-page PDF from
// `piece_preview`) with pdf.js, or an image, for the source pane and the thumbnails in the
// review's strip of pages.

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
			return new SourcePage(page, null, () => task.destroy());
		} catch (e) {
			await task.destroy();
			throw e;
		}
	}

	/**
	 * Draws the page onto `target`, `width` CSS pixels wide once turned by `rotation`. Sharp on
	 * high-density screens: drawn at the device pixel ratio, shown at CSS size. Resolves false
	 * when a later drawing took its place.
	 */
	async draw(target: HTMLCanvasElement, width: number, rotation: Rotation = 0): Promise<boolean> {
		const ticket = ++this.#latest;
		const ratio = window.devicePixelRatio || 1;
		const drawing = document.createElement('canvas');
		if (this.page) {
			// The page's own rotation, from the PDF, plus the reader's.
			const turned = (this.page.rotate + rotation) % 360;
			const natural = this.page.getViewport({ scale: 1, rotation: turned });
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

/** Draws a page once, `width` CSS pixels wide: for the strip's thumbnails. */
export async function renderPdfPage(
	data: string,
	target: HTMLCanvasElement,
	width: number
): Promise<void> {
	const page = await SourcePage.open('application/pdf', data);
	try {
		await page.draw(target, width);
	} finally {
		await page.destroy();
	}
}
