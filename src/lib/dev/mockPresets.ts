// Settings, prompts and saved pipelines for the browser preview (`?mock&view=prompts`, New job):
// the settings a fresh install would hold, the default prompts Prompts lists, and two saved
// pipelines, kept in memory. Development builds only.

import type { Pipeline } from '../bindings/Pipeline.js';
import type { Preset } from '../bindings/Preset.js';
import type { Settings } from '../bindings/Settings.js';
import { contentsPresets } from './mockContents.js';
import { EXPORT_OPTIONS } from './mockJobs.js';

const settings: Settings = {
	concurrency: 2,
	text_formats: ['txt', 'docx', 'json'],
	audio_formats: ['txt', 'docx', 'srt', 'json'],
	destination: { kind: 'beside_sources' },
	export_options: EXPORT_OPTIONS,
	theme: new URLSearchParams(location.search).get('theme') ?? 'system',
	language: new URLSearchParams(location.search).get('lang') ?? 'system',
	refresh_catalog: false,
	favourite_languages: ['fr', 'ar'],
	last_folder: null,
	ai_consent: [],
	ocr_graphics_processor: true
};

/** The prompt a dictionary sends for proposed merges, as Prompts lists it. */
const dictionaryPreset: Preset = {
	id: 'dictionary.merges',
	task: 'analysis',
	name: 'Dictionary: proposed merges',
	description:
		"Sent when a project's dictionary asks a model to propose merges. Its answer's fields are fixed: the dictionary reads them.",
	instructions:
		'You help a researcher clean the dictionary of a research project… Never group rows the researcher kept apart.',
	user_prompt: 'Propose groups among the rows above, following the instructions given.',
	builtin: true,
	modified: false
};

const presets: Preset[] = [
	{
		id: 'text_extraction.printed',
		task: 'text_extraction',
		name: 'Printed or typed document',
		description:
			'Books, reports, typed letters and forms in any language, left to right or right to left; spelling kept as printed.',
		instructions:
			'# Printed or typed document\n\nYou transcribe printed and typewritten documents…',
		user_prompt: 'Transcribe this page completely, following the instructions given.',
		builtin: true,
		modified: false
	},
	{
		id: 'text_extraction.newspaper',
		task: 'text_extraction',
		name: 'Newspaper or magazine',
		description:
			'Article by article across the columns, with the words the columns split rejoined.',
		instructions: '# Newspaper or magazine page…',
		user_prompt: 'Transcribe this page completely, following the instructions given.',
		builtin: true,
		modified: true,
		dense: true
	},
	{
		id: 'audio_transcription.interview',
		task: 'audio_transcription',
		name: 'Interview or recording, transcribed',
		description:
			'Word for word in the languages spoken, with speaker labels and a timestamp at each turn.',
		instructions: '# Interview or recording, transcribed…',
		user_prompt: 'Transcribe this recording.',
		builtin: true,
		modified: false
	},
	{
		id: 'analysis.entities',
		task: 'analysis',
		name: 'People, places and organisations',
		description: 'The people, organisations, places and subjects each document names.',
		instructions: '# People, places and organisations\n\nYou index documents for a researcher…',
		user_prompt: 'Read the document above and fill in each field.',
		builtin: true,
		modified: false,
		schema: {
			fields: [
				{
					key: 'people',
					label: 'People',
					instructions: 'People named in the document.',
					kind: { type: 'list', max: 30 },
					reason: false
				},
				{
					key: 'places',
					label: 'Places',
					instructions: 'Places named in the document.',
					kind: { type: 'list', max: 20 },
					reason: false
				},
				{
					key: 'subjects',
					label: 'Subjects',
					instructions: "The document's specific subjects.",
					kind: { type: 'list', max: 6 },
					reason: false
				}
			]
		}
	},
	{
		id: 'analysis.tone',
		task: 'analysis',
		name: 'Tone of the document',
		description: 'Whether each document presents its subject positively or negatively.',
		instructions: '# Tone of the document\n\nYou read documents for a researcher…',
		user_prompt: 'Read the document above and fill in each field.',
		builtin: true,
		modified: false,
		schema: {
			fields: [
				{
					key: 'main_subject',
					label: 'Main subject',
					instructions: 'What the document is mainly about, in a few words.',
					kind: { type: 'text' },
					reason: false
				},
				{
					key: 'tone',
					label: 'Tone',
					instructions: 'How the document presents its main subject.',
					kind: { type: 'choice', values: ['Positive', 'Neutral', 'Negative', 'Mixed'] },
					reason: true
				}
			]
		}
	},
	{
		id: 'analysis.image_description',
		task: 'analysis',
		name: 'Image description and keywords',
		description:
			'What each photograph, poster, map or other image shows, as a catalogue would describe it, with keywords, the text written on it and the names that text gives.',
		instructions: '# Image description and keywords\n\nYou describe images for a researcher…',
		user_prompt: 'Look at the image above and fill in each field.',
		builtin: true,
		modified: false,
		made_for: 'image',
		schema: {
			fields: [
				{
					key: 'description',
					label: 'Description',
					instructions: 'What is visibly there; the first sentence a summary on its own.',
					kind: { type: 'text' },
					reason: false
				},
				{
					key: 'kind_of_image',
					label: 'Kind of image',
					instructions: 'What the image is as an object, not what it shows.',
					kind: {
						type: 'choice',
						values: [
							'Photograph',
							'Poster or advertisement',
							'Map or plan',
							'Drawing or painting',
							'Printed page',
							'Manuscript page',
							'Object',
							'Other'
						]
					},
					reason: false
				},
				{
					key: 'keywords',
					label: 'Keywords',
					instructions: 'Subjects visibly present, specific before general.',
					kind: { type: 'list', max: 12 },
					reason: false
				},
				{
					key: 'text_in_the_image',
					label: 'Text in the image',
					instructions: 'Captions, signs, inscriptions and stamps, as written.',
					kind: { type: 'text' },
					reason: false
				},
				{
					key: 'people',
					label: 'People',
					instructions: "People named in the image's own text.",
					kind: { type: 'list', max: 20 },
					reason: false
				},
				{
					key: 'places',
					label: 'Places',
					instructions: "Places named in the image's own text.",
					kind: { type: 'list', max: 20 },
					reason: false
				},
				{
					key: 'organisations',
					label: 'Organisations',
					instructions: "Organisations named in the image's own text.",
					kind: { type: 'list', max: 20 },
					reason: false
				}
			]
		}
	}
];

/** Saved pipelines, as New job lists them. */
let pipelines: Pipeline[] = [
	{
		id: 'pipeline-0',
		name: 'Printed bulletins',
		task: 'text_extraction',
		setups: [
			{
				provider: 'gemini',
				model: 'gemini-3.8-flash',
				preset_id: 'text_extraction.printed',
				label: null,
				options: { temperature: null, diarize: false, timestamps: false, speakers: null }
			}
		],
		settings: {
			languages: { mode: 'explicit', languages: ['fr'] },
			sample_pages: null,
			line_breaks: 'running_text',
			pace: 'fast'
		},
		formats: ['txt', 'docx'],
		destination: { kind: 'beside_sources' },
		updated_at: 0
	},
	{
		id: 'pipeline-1',
		name: 'Press, people and places',
		task: 'analysis',
		setups: [
			{
				provider: 'gemini',
				model: 'gemini-3.8-flash',
				preset_id: 'analysis.entities',
				label: null,
				options: { temperature: null, diarize: false, timestamps: false, speakers: null }
			}
		],
		settings: {
			languages: { mode: 'explicit', languages: ['fr'] },
			sample_pages: null,
			line_breaks: null,
			pace: 'fast'
		},
		formats: ['csv', 'json'],
		destination: { kind: 'beside_sources' },
		updated_at: 0
	}
];

/** The two saved pipelines the sample workflows are made from. */
export function samplePipelines(): [Pipeline, Pipeline] {
	return [pipelines[0], pipelines[1]];
}

export function presetCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'get_settings':
			return settings;
		case 'export_pipeline':
			return String(a.path).replace(/\.json$/i, '.md');
		case 'list_presets': {
			// `view=prompts-analysis` opens Prompts on an analysis, for screenshots.
			const first = location.search.includes('prompts-analysis') ? 'analysis' : '';
			const ordered = [...presets, ...contentsPresets, dictionaryPreset].sort(
				(x, y) => Number(y.task === first) - Number(x.task === first)
			);
			// The dictionary's prompt is listed in Prompts, never offered for documents.
			return a.task ? ordered.filter((p) => p.task === a.task && p !== dictionaryPreset) : ordered;
		}
		case 'list_pipelines':
			return pipelines;
		case 'save_pipeline': {
			const saved = { ...(a.pipeline as Pipeline), id: `pipeline-${pipelines.length + 1}` };
			pipelines = [...pipelines.filter((p) => p.name !== saved.name), saved];
			return saved;
		}
		case 'delete_pipeline':
			pipelines = pipelines.filter((p) => p.id !== a.id);
			return null;
		default:
			return undefined;
	}
}
