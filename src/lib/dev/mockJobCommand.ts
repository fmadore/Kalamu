// The job and review commands for the browser preview: the sample jobs listed, opened, renamed
// and deleted, their pieces, recordings and scans, error rates and word diffs, and the
// corrections kept on the pieces as the store keeps them. Development builds only.

import type { Correction } from '../bindings/Correction.js';
import SAMPLE_SCAN from './sample-scan.jpg?inline';
import { progressOf } from './mockLarge.js';
import { mockRegions } from './mockRegions.js';
import {
	analysisAgreement,
	analysisJob,
	analysisPieces,
	archiveJobs,
	article,
	audioJob,
	audioPieces,
	audioSetup,
	job,
	nameSpeakers,
	pausedJob,
	pieces,
	recording,
	renameListed,
	addListed,
	setupA,
	setupB,
	source,
	summaries,
	texts,
	video,
	videoJob,
	videoPieces
} from './mockJobs.js';
import { contentsJob, contentsPieces, correctionJob, correctionPieces } from './mockContents.js';
import { editRows, recordsJob, recordsPieces } from './mockRecords.js';

/** Jobs deleted while looking at the mock, left out of the list from then on. */
const deletedJobs = new Set<string>();

/** The recording's address, made once the review first asks for it. */
let toneUrl: string | undefined;

/** Nine seconds of a quiet tone as a WAV file, standing in for a recording. */
function toneWav(): ArrayBuffer {
	const rate = 8_000;
	const samples = rate * 9;
	const buffer = new ArrayBuffer(44 + samples * 2);
	const view = new DataView(buffer);
	const text = (at: number, value: string) =>
		[...value].forEach((c, i) => view.setUint8(at + i, c.charCodeAt(0)));
	text(0, 'RIFF');
	view.setUint32(4, 36 + samples * 2, true);
	text(8, 'WAVEfmt ');
	view.setUint32(16, 16, true);
	view.setUint16(20, 1, true);
	view.setUint16(22, 1, true);
	view.setUint32(24, rate, true);
	view.setUint32(28, rate * 2, true);
	view.setUint16(32, 2, true);
	view.setUint16(34, 16, true);
	text(36, 'data');
	view.setUint32(40, samples * 2, true);
	for (let i = 0; i < samples; i++) {
		view.setInt16(44 + i * 2, Math.sin((2 * Math.PI * 220 * i) / rate) * 2_000, true);
	}
	return buffer;
}

/** A word diff like the engine's, so the review's apparatus shows real variants. */
function wordDiff(left: string, right: string): { kind: string; text: string }[] {
	const a = left.split(/(\s+)/).filter(Boolean);
	const b = right.split(/(\s+)/).filter(Boolean);
	const lcs = Array.from({ length: a.length + 1 }, () => new Array<number>(b.length + 1).fill(0));
	for (let i = a.length - 1; i >= 0; i--)
		for (let j = b.length - 1; j >= 0; j--)
			lcs[i][j] = a[i] === b[j] ? lcs[i + 1][j + 1] + 1 : Math.max(lcs[i + 1][j], lcs[i][j + 1]);
	const out: { kind: string; text: string }[] = [];
	const push = (kind: string, text: string) => {
		const last = out[out.length - 1];
		if (last?.kind === kind) last.text += text;
		else out.push({ kind, text });
	};
	let i = 0;
	let j = 0;
	while (i < a.length || j < b.length) {
		if (i < a.length && j < b.length && a[i] === b[j]) push('same', (i++, j++, a[i - 1]));
		else if (j < b.length && (i >= a.length || lcs[i][j + 1] >= lcs[i + 1][j]))
			push('added', b[j++]);
		else push('removed', a[i++]);
	}
	return out;
}

/** Keeps a correction on its sample piece, as the store keeps it, so the review shows it again
 * when it reads the pieces anew. */
function keepCorrection(c: Correction): void {
	const piece = [
		...pieces,
		...analysisPieces,
		...contentsPieces,
		...correctionPieces,
		...recordsPieces
	].find((p) => p.setup_id === c.setup_id && p.source_id === c.source_id && p.index === c.index);
	if (!piece) return;
	piece.corrected_text = c.text;
	piece.corrected_fields = c.fields ?? null;
	piece.reviewed = c.reviewed;
}

/** Error rates against reviewed pages: the comparison and the interviews have some. */
function mockScores(jobId: string) {
	if (jobId === job.id) {
		return [
			{
				setup_id: setupA.id,
				label: setupA.label,
				cer: 0.012,
				wer: 0.04,
				scored_pages: 1,
				cost_usd: 0.0036
			},
			{
				setup_id: setupB.id,
				label: setupB.label,
				cer: 0.031,
				wer: 0.09,
				scored_pages: 1,
				cost_usd: 0.012
			}
		];
	}
	if (jobId === audioJob.id) {
		return [
			{
				setup_id: audioSetup.id,
				label: audioSetup.label,
				cer: 0.023,
				wer: 0.081,
				scored_pages: 2,
				cost_usd: 0.02
			}
		];
	}
	return [];
}

export function jobCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'list_jobs':
			return [
				...summaries,
				{
					...summaries[0],
					id: analysisJob.id,
					name: analysisJob.name,
					task: 'analysis',
					pieces_total: 2,
					pieces_finished: 2
				},
				...[contentsJob, correctionJob].map((j, i) => ({
					...summaries[0],
					id: j.id,
					name: j.name,
					task: j.task,
					pieces_total: i === 0 ? 4 : 2,
					pieces_finished: i === 0 ? 4 : 2,
					setup_count: 1
				})),
				{
					...summaries[0],
					id: recordsJob.id,
					name: recordsJob.name,
					task: recordsJob.task,
					status: 'completed_with_errors',
					pieces_total: 3,
					pieces_finished: 2,
					setup_count: 1
				},
				...archiveJobs.map(({ files: _, ...summary }) => summary)
			].filter((j) => !deletedJobs.has(j.id));
		case 'job_sources':
			return [
				{ job_id: job.id, names: [source.file_name] },
				{ job_id: audioJob.id, names: [recording.file_name] },
				{ job_id: videoJob.id, names: [video.file_name, video.title ?? ''] },
				...archiveJobs.map((j) => ({ job_id: j.id, names: j.files }))
			];
		case 'delete_job':
			deletedJobs.add(String(a.id));
			return null;
		case 'delete_jobs':
			for (const id of a.ids as string[]) deletedJobs.add(id);
			return null;
		case 'get_job':
			if (a.id === pausedJob.id) return pausedJob;
			if (a.id === videoJob.id) return videoJob;
			if (a.id === contentsJob.id) return contentsJob;
			if (a.id === correctionJob.id) return correctionJob;
			if (a.id === recordsJob.id) return recordsJob;
			return a.id === audioJob.id ? audioJob : a.id === analysisJob.id ? analysisJob : job;
		// A paused or interrupted job runs again, as the engine has it, until the next reading of
		// the list; `?resume-fails` refuses, as the engine does for a provider without its key.
		case 'start_job': {
			if (new URLSearchParams(location.search).has('resume-fails')) {
				throw { id: 'provider.missing_key', detail: 'mistral' };
			}
			for (const s of summaries) if (s.id === a.id) s.status = 'running';
			return null;
		}
		case 'rename_job': {
			const name = String(a.name).trim();
			if (!name) return null;
			for (const j of [job, audioJob, videoJob, analysisJob]) if (j.id === a.id) j.name = name;
			for (const s of [...summaries, ...archiveJobs]) if (s.id === a.id) s.name = name;
			return null;
		}
		case 'job_outputs': {
			const stem = source.path.replace(/\.[^.\\]+$/, '');
			return [`${stem}.txt`, `${stem}.docx`, `${stem}.provenance.json`];
		}
		case 'open_output':
			return null;
		// The corrected text of the sample extraction, frozen for an analysis.
		case 'snapshot_job':
			return [
				{
					...article,
					id: '00000000-0000-0000-0000-0000000000f9',
					path: 'C:\\Users\\you\\AppData\\Local\\Kalamu Data\\snapshots\\ab.txt',
					file_name: 'bulletin-1957-05.txt',
					derived_from: {
						snapshot_id: 'snapshot-1',
						job_id: a.id,
						setup_id: a.setup,
						source_id: source.id,
						original_path: source.path
					}
				}
			];
		case 'derived_state':
			return [];
		case 'job_progress':
			return progressOf(pieces, a.sources as string[]);
		case 'source_pieces':
			if (a.jobId === analysisJob.id) return analysisPieces;
			if (a.jobId === contentsJob.id) return contentsPieces;
			if (a.jobId === correctionJob.id) return correctionPieces;
			if (a.jobId === recordsJob.id) return recordsPieces;
			if (a.jobId === videoJob.id) return videoPieces();
			return a.jobId === audioJob.id ? audioPieces() : pieces;
		case 'source_text':
			return texts[0];
		case 'save_speaker_names':
			nameSpeakers((a.names as { names: Record<string, string> }).names);
			return null;
		case 'piece_audio':
			// The tone stands in for a whole recording, the piece for a span of it.
			toneUrl ??= URL.createObjectURL(new Blob([toneWav()], { type: 'audio/wav' }));
			return { path: toneUrl, start_ms: 2_000, end_ms: 8_000, original: true };
		case 'page_regions':
			return mockRegions(String(a.jobId), String(a.setupId), Number(a.index));
		case 'piece_preview':
		case 'piece_thumbnail':
			// A synthetic scan as an image: shown everywhere, headless browsers included.
			return { mime: 'image/jpeg', data: SAMPLE_SCAN.split(',')[1] ?? '' };
		case 'compare_source':
			return { scores: mockScores(String(a.jobId)), pages: [] };
		case 'job_scores': {
			const scores = mockScores(String(a.jobId));
			const scored = a.jobId === job.id ? source : a.jobId === audioJob.id ? recording : null;
			return {
				total: scores,
				sources: scored ? [{ source_id: scored.id, scores }] : []
			};
		}
		case 'job_agreement':
			return a.jobId === analysisJob.id ? analysisAgreement() : { fields: [] };
		case 'diff_texts':
			return wordDiff(String(a.left ?? ''), String(a.right ?? ''));
		case 'save_correction':
			keepCorrection(a.correction as Correction);
			return null;
		case 'save_corrections':
			// One transaction in the store; here, each kept in turn.
			for (const c of a.corrections as Correction[]) keepCorrection(c);
			return null;
		case 'rename_listed':
			return renameListed(a);
		case 'add_listed':
			return addListed(a);
		case 'edit_rows':
			return editRows(a);
		case 'job_text_outputs':
		case 'export_source':
			return [];
		default:
			return undefined;
	}
}
