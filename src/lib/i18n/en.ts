// The English interface. Every visible string lives here so French and German can be added as
// `fr.ts` and `de.ts` typed `: Messages` without touching components, as in Live Translation.

export const en = {
	locale: { name: 'English', tag: 'en' },
	app: {
		name: 'Scriptorium',
		tagline: 'Transcription and text extraction'
	},
	shell: {
		collapse: 'Fold the menu',
		expand: 'Open the menu',
		running: (name: string) => `Running: ${name}`,
		unfinished: (n: number) => (n === 1 ? '1 unfinished job' : `${n} unfinished jobs`)
	},
	nav: {
		newJob: 'New job',
		jobs: 'Jobs',
		prompts: 'Prompts',
		settings: 'Settings'
	},
	task: {
		text_extraction: 'Extract text',
		text_extractionHint: 'Printed and handwritten documents: PDF, JPEG, PNG, TIFF',
		audio_transcription: 'Transcribe audio',
		audio_transcriptionHint: 'Interviews, recordings and videos: MP3, M4A, WAV, MP4',
		analysis: 'Analyse texts',
		analysisHint:
			'Summaries, people and places, tone: you choose what to find. Texts, Word files, PDFs and earlier results'
	},
	newJob: {
		title: 'New job',
		lede: 'Choose the files, a model and a prompt. The cost is estimated before anything is sent.',
		taskLabel: 'What do you want to do?',
		filesTitle: 'Files',
		addFiles: 'Add files',
		addFolder: 'Add a folder',
		dropHint: 'Or drop files and folders anywhere on this window.',
		noFiles: 'No files yet.',
		remove: 'Remove',
		pages: (n: number) => (n === 1 ? '1 page' : `${n} pages`),
		hoursMinutes: (h: number, min: string) => `${h} h ${min} min`,
		minutes: (min: number) => `${min} min`,
		seconds: (s: number) => `${s} s`,
		evenOut: 'Even out the volume',
		splitSpreads: 'Split double pages',
		languageTitle: 'Language',
		languageAuto: 'Detect automatically',
		languagePick: 'Choose languages',
		analysisLanguageTitle: 'Language of the answers',
		analysisLanguageAuto: 'Same as the document',
		analysisLanguageHint:
			'Choose a language to have summaries, subjects and reasons written in it, whatever the language of the document. Names stay as the document spells them.',
		languageHint:
			'Choosing helps with short or noisy material. Pick several for documents or recordings that mix languages.',
		addLanguage: 'Add a language…',
		setupsTitle: 'Model and prompt',
		setupsHint: 'Add a second model to compare their results side by side.',
		provider: 'Provider',
		model: 'Model',
		prompt: 'Prompt',
		noPrompt: 'This model takes no prompt',
		label: 'Label',
		compareAdd: 'Compare with another model',
		analysis: 'Analysis',
		finds: 'Fills in',
		editFields: 'Change the fields in Prompts',
		pipelinesTitle: 'Start from a pipeline',
		pipelineSave: 'Save as a pipeline',
		pipelineName: 'Name of the pipeline',
		pipelineSaved: (name: string) => `Saved as “${name}”. It appears at the top of New job.`,
		pipelineApplied: (name: string) => `Settings of “${name}” applied. Add the files.`,
		pipelineDelete: (name: string) => `Delete the pipeline “${name}”`,
		pipelineDeleteConfirm: (name: string) =>
			`Delete the pipeline “${name}”? Jobs already run with it are kept.`,
		linesTitle: 'Line breaks',
		lines: {
			running_text: 'Running text',
			running_text_hint:
				'Lines joined into paragraphs and words split across lines rejoined. Best for reading and for searching the text.',
			as_on_page: 'As on the page',
			as_on_page_hint:
				'Every line kept where it ends on the page, end-of-line hyphens included. Easiest to check against the scan, but a word split across two lines is harder to find by search.'
		},
		linesNoPrompt: 'Models that take no prompt, such as Mistral OCR, lay out lines their own way.',
		removeSetup: 'Remove',
		noKey: (provider: string) => `Add a ${provider} key in Settings to use it.`,
		noProvider: 'Add an API key in Settings to choose a model.',
		diarize: 'Label speakers',
		noSpeakers: 'This model cannot label speakers',
		costly:
			'Very expensive, and most likely more than this task needs: the recommended models read print, handwriting and speech almost as well for a fraction of the cost. Try a sample first.',
		notDownloaded: 'This offline model has not been downloaded yet.',
		openSettings: 'Download in Settings',
		timestamps: 'Timestamps',
		sampleTitle: 'Try a sample first',
		sample: (n: number) => `Only the first ${n} pages of each document`,
		outputTitle: 'Results',
		formats: 'Formats',
		besideSources: 'Next to the original files',
		inFolder: 'In a folder',
		chooseFolder: 'Choose…',
		estimate: 'Estimated cost',
		estimateUnknown: 'no published price',
		estimateNote: 'An estimate from page counts and published prices; actual costs vary.',
		start: 'Start',
		starting: 'Starting…',
		name: 'Job name',
		inspecting: 'Reading files…'
	},
	warning: {
		double_page_spread: (pages: string) =>
			`Two pages side by side (page ${pages}): models may read across both. Split them before running.`,
		low_resolution: (dpi: number, pages: string) =>
			`Low resolution, about ${dpi} dpi (page ${pages}): small print and handwriting may be misread. Rescan at 300 dpi or more if you can.`,
		has_text_layer: (n: number) =>
			`This PDF already contains text on ${n} of the first pages. Extraction may not be needed, or the existing text can serve to compare.`,
		unsupported_image: (detail: string) => detail,
		large_file: 'This file is large; it will be split before sending.',
		tool_missing:
			'Reading this file needs FFmpeg, which was not found next to the app. Reinstall Scriptorium, or convert the file to MP3.',
		no_sound_track: 'This file has no sound track, so there is nothing to transcribe.',
		long_recording: (length: string, pieces: number) =>
			`Long recording (${length}): it will be sent in ${pieces} pieces, cut at pauses. Speaker labels may differ from one piece to the next.`,
		quiet_recording: (db: number) =>
			`Very quiet recording (about ${db} dB): quiet speech may be missed. Evening out the volume usually helps.`
	},
	tips: {
		title: 'Tips for better results',
		scanning:
			'One page per image or PDF page, not an open double-page spread; flat and fully in frame; even light without shadows or flash glare; 300–400 dpi; colour or greyscale rather than black and white for faded or handwritten material.',
		recording:
			'Microphone close to the speakers, a quiet room, one recording per interview, and a short spoken introduction with names and date.',
		results:
			'Try a sample before a whole folder, compare two models on difficult material, check names, numbers and dates against the source, and keep the originals.'
	},
	jobs: {
		title: 'Jobs',
		lede: (n: number) =>
			n === 0 ? 'Nothing yet.' : n === 1 ? 'One job, newest first.' : `${n} jobs, newest first.`,
		empty: 'No jobs yet. Start one from New job.',
		resumeTitle: 'Unfinished jobs',
		resumeHint:
			'These jobs stopped before they finished, when the app closed or a key stopped working. Resume them to continue where they left off; finished pages are not sent again.',
		resume: 'Resume',
		pause: 'Pause',
		cancel: 'Cancel',
		save: 'Save',
		retry: 'Retry failed pages',
		delete: 'Delete',
		deleteConfirm: 'Delete this job? Files already written stay on disk.',
		review: 'Review',
		summarise: 'Analyse the results',
		noTextOutputs:
			'This job wrote no text file to analyse. Add Text (.txt) to its formats and export again.',
		openFolder: 'Show files',
		progress: (done: number, total: number) => `${done} of ${total} pieces`,
		failed: (n: number) => (n === 1 ? '1 failed' : `${n} failed`),
		spent: (usd: string) => `${usd} spent this run`,
		files: 'Files',
		comparison: 'Comparison',
		livePreview: 'Latest result',
		outputs: 'Written',
		created: 'Created'
	},
	status: {
		pending: 'Waiting',
		running: 'Running',
		paused: 'Paused',
		interrupted: 'Interrupted',
		completed: 'Done',
		completed_with_errors: 'Done with errors',
		cancelled: 'Cancelled',
		done: 'Done',
		cached: 'From cache',
		failed: 'Failed'
	},
	review: {
		title: 'Review',
		back: 'Back to job',
		page: (label: string) => `Page ${label}`,
		segment: (label: string) => `From ${label}`,
		document: 'Whole document',
		keywords: 'Keywords',
		previous: 'Previous',
		next: 'Next',
		source: 'Source',
		aiText: 'Model output',
		edit: 'Correct',
		save: 'Save correction',
		discard: 'Discard changes',
		revert: 'Back to model output',
		reviewed: 'Reviewed',
		markReviewed: 'Mark as reviewed',
		corrected: 'Corrected',
		showDiff: 'Highlight differences',
		scores: 'Scores',
		cer: 'Character error rate',
		wer: 'Word error rate',
		scoredPages: (n: number) => (n === 1 ? 'on 1 corrected page' : `on ${n} corrected pages`),
		noReference:
			'Correct a page to score the models against it. Until then, the view shows where they disagree.',
		agreement: (pct: string) => `${pct} different`,
		noPreview: 'No preview for this file type.',
		notDone: 'Not processed yet.',
		saved: 'Saved; the result files were updated.',
		pieceReviewed: (label: string) => `${label}, reviewed`,
		zoomIn: 'Zoom in',
		zoomOut: 'Zoom out',
		showInText: 'Show in the text',
		play: 'Play',
		pausePlayback: 'Pause',
		position: 'Position in this piece',
		noAudio: 'This piece could not be played.',
		lines: 'Transcript lines',
		noTimings: 'This model gave no timings, so lines cannot follow the recording.',
		reviewedOf: (n: number, total: number) => `Reviewed ${n} of ${total}`,
		readingText: 'Reading text',
		readingTextOf: 'reading text',
		correcting: 'Correcting',
		apparatus: 'Where the witnesses differ',
		sigla: 'Sigla',
		corrSiglum: 'corr.',
		added: 'add.',
		omitted: 'om.',
		speakers: 'Speakers',
		speakersHint:
			"Give the model's labels real names. They replace the labels here and in every file; the model's own output is kept.",
		speakerPlaceholder: 'Name',
		speakerName: (label: string) => `Name for ${label}`,
		saveNames: 'Save names',
		namesSaved: 'Names saved; the result files were updated.',
		speakersPieces:
			'This recording was sent in several pieces: the same label may be a different person in each piece. Names apply to the whole recording.',
		yourCorrections: 'Your corrections',
		correctionsKept: "Kept apart: the model's text is never overwritten",
		notReviewed: 'Not reviewed yet',
		answers: 'Answers',
		answersOf: 'answers of',
		noneFound: 'None found',
		onePerLine: 'One per line.'
	},
	prompts: {
		title: 'Prompts',
		hint: 'Each task ships with tested prompts. Edit one to adapt it; your version replaces it until you reset it. Variables: {languages}, {speakers}.',
		builtin: 'Default',
		modified: 'Edited',
		own: 'Yours',
		name: 'Name',
		description: 'Description',
		instructions: 'Instructions (system prompt)',
		userPrompt: 'Request sent with each page',
		userPromptDocument: 'Request sent with each document',
		save: 'Save',
		duplicate: 'Duplicate',
		copyName: (name: string) => `${name} (copy)`,
		reset: 'Reset to default',
		delete: 'Delete',
		export: 'Export…',
		import: 'Import…',
		imported: (n: number) => `${n} imported`,
		saved: 'Saved',
		unsaved: 'Unsaved changes',
		analysisHint:
			'An analysis is a set of fields to fill in for each document. Start from one of these, then duplicate it to make your own.'
	},
	fields: {
		title: 'Fields',
		hint: 'What to find in each document. Each field becomes a column in the spreadsheet export.',
		name: 'Name',
		kind: 'Answer',
		kinds: {
			text: 'Text',
			list: 'List of names or terms',
			choice: 'One choice',
			choices: 'Several choices'
		},
		max: 'At most',
		values: 'Choices, separated by commas',
		valuesPlaceholder: 'Positive, Neutral, Negative',
		instructions: 'What belongs here',
		instructionsPlaceholder: 'For example: towns, villages and regions; mosques count as places.',
		reason: 'Ask for a reason',
		remove: (label: string) => `Remove ${label || 'this field'}`,
		up: 'Move up',
		down: 'Move down',
		add: 'Add a field',
		templates: {
			summary: {
				label: 'Summary',
				instructions: 'What this document in particular says: who, what, where, when.'
			},
			people: { label: 'People', instructions: 'People named in the document.' },
			places: { label: 'Places', instructions: 'Places named in the document.' },
			organisations: {
				label: 'Organisations',
				instructions: 'Associations, parties, administrations, companies, religious bodies.'
			},
			keywords: {
				label: 'Keywords',
				instructions: 'Terms someone would search for, specific rather than general.'
			},
			tone: { label: 'Tone', instructions: 'How the document presents its main subject.' },
			custom: { label: 'Other', instructions: '' }
		},
		toneValues: ['Positive', 'Neutral', 'Negative', 'Mixed']
	},
	settings: {
		title: 'Settings',
		lede: 'Keys, offline models, processing and appearance.',
		keysTitle: 'API keys',
		keysHint:
			'Keys are stored in Windows Credential Manager and sent only to their provider. Files go directly from this PC to the provider you choose.',
		keySaved: 'Saved',
		keyMissing: 'No key',
		keyPlaceholder: 'Paste the key',
		save: 'Save',
		test: 'Test',
		remove: 'Remove',
		getKey: 'Get a key',
		testOk: (n: number) => `Key works: ${n} models available`,
		offlineTitle: 'Offline transcription',
		offlineHint:
			'Whisper runs on this PC: recordings never leave it, no key or account is needed, and it works without internet once a model is downloaded. It is slower than the cloud models and cannot label speakers. Large v3 Turbo suits most PCs.',
		offlineCpu: (missing: string) =>
			`This processor lacks ${missing}, which offline transcription needs. Use a cloud model instead.`,
		modelInstalled: 'Downloaded',
		modelMissing: 'Not downloaded',
		modelDownloading: (pct: number) => `Downloading… ${pct}%`,
		modelInUse: 'In use by a job',
		download: 'Download',
		cancelDownload: 'Cancel',
		removeModel: 'Remove',
		size: (bytes: number) =>
			bytes >= 1e9 ? `${(bytes / 1e9).toFixed(1)} GB` : `${Math.round(bytes / 1e6)} MB`,
		jobsTitle: 'Processing',
		concurrency: 'Pages sent at the same time',
		concurrencyHint: 'Applies from the next launch. Lower it if a provider reports rate limits.',
		catalogTitle: 'Models',
		catalogUpdated: (date: string) => `Model list from ${date}`,
		catalogRefresh: 'Check for an updated model list',
		catalogAuto: 'Check at startup',
		appearanceTitle: 'Appearance',
		theme: 'Theme',
		themeSystem: 'Same as Windows',
		themeLight: 'Light',
		themeDark: 'Dark',
		aboutTitle: 'About',
		version: (v: string) => `Version ${v}`,
		dataFolder: 'Data folder',
		credits:
			'Scriptorium uses FFmpeg under the LGPL v2.1 (its source is published with each release), Opus (BSD), whisper.cpp (MIT) and the Whisper models (MIT). The full licences are in the licences folder installed with the app.',
		privacy:
			'Nothing is sent to the developer: no files, keys, results or telemetry. AI output can contain fluent but wrong text; check a sample against the source.'
	},
	format: {
		txt: 'Text (.txt)',
		markdown: 'Markdown (.md)',
		docx: 'Word (.docx)',
		json: 'JSON',
		srt: 'Subtitles (.srt)',
		vtt: 'Subtitles (.vtt)',
		csv: 'Spreadsheet (.csv)',
		searchable_pdf: 'Searchable PDF (Mistral OCR)'
	},
	close: {
		title: 'Jobs are still running',
		body: 'They can keep running in the background, with Scriptorium in the notification area next to the clock. Or quit now: the jobs pause, finished pages are kept, and you resume them next time from Jobs.',
		background: 'Keep running in the background',
		quit: 'Pause and quit',
		stay: 'Cancel',
		hidden:
			'Scriptorium is still working. Click its icon in the notification area to open it again.'
	},
	tray: {
		open: 'Open Scriptorium',
		pause: 'Pause all jobs',
		quit: 'Quit',
		quitPausing: 'Pause jobs and quit',
		idle: 'No jobs running',
		running: (jobs: number, percent: number) =>
			`${jobs === 1 ? '1 job' : `${jobs} jobs`} running · ${percent}%`
	},
	common: {
		cancel: 'Cancel',
		save: 'Save',
		close: 'Close',
		loading: 'Loading…',
		preview: 'preview',
		retiring: 'retiring'
	},
	error: {
		'job.not_found': 'This job no longer exists.',
		'job.database': 'The job database could not be read or written.',
		'job.running': 'This job is already running.',
		'job.empty': 'Add at least one file and choose a model.',
		'media.encrypted': 'This PDF is password-protected. Remove the password and add it again.',
		'media.unsupported': 'This file type is not supported yet.',
		'media.read': 'The file could not be read.',
		'prompt.invalid': 'The prompt could not be saved: it needs a name and instructions.',
		'prompt.no_fields': 'An analysis needs at least one field.',
		'prompt.unnamed_field': 'Every field needs a name.',
		'prompt.duplicate_field': 'Two fields have the same name.',
		'prompt.few_values': 'A choice needs at least two values to choose from.',
		'prompt.unnamed_pipeline': 'A pipeline needs a name and at least one model.',
		'provider.unknown': 'Unknown provider.',
		'provider.missing_key': 'No API key is saved for this provider. Add one in Settings.',
		'provider.unsupported': 'This model cannot do this task.',
		'provider.auth': 'The provider rejected the API key. Check it in Settings.',
		'provider.quota': 'The provider account has no credit or quota left.',
		'provider.rate_limited': 'The provider asked to slow down; the page will be retried.',
		'provider.transient': 'The provider could not be reached.',
		'provider.blocked': 'The provider refused to return text for this page.',
		'provider.invalid_request': 'The provider rejected the request.',
		'provider.other': 'The provider sent an unexpected response.',
		'provider.invalid_answer':
			"The model's answer did not fit the analysis's fields; it is asked again.",
		'provider.not_installed':
			'This offline model has not been downloaded. Download it in Settings.',
		'provider.interrupted': 'Stopped before the end; it will continue when the job resumes.',
		'offline.unknown_model': 'This offline model is not known to this version of Scriptorium.',
		'offline.busy': 'Another model is downloading; wait for it to finish.',
		'offline.in_use': 'A running job uses this model; pause it first.',
		'offline.cancelled': 'Download cancelled.',
		'offline.download':
			'The model could not be downloaded. Check the internet connection and try again.',
		'offline.integrity': 'The downloaded model was damaged and has been discarded. Try again.',
		'offline.io': 'The model file could not be read or written.',
		'output.write': 'The result files could not be written.',
		'keys.keychain': 'Windows Credential Manager refused the request.',
		'keys.format': 'This key contains a character no API key has, so it was not saved.',
		'settings.write': 'The settings could not be saved.',
		'catalog.download': 'The model list could not be downloaded.',
		'catalog.invalid': 'The downloaded model list could not be read.',
		'file.read': 'The file could not be read or written.',
		'app.internal': 'Something went wrong inside the app.',
		'app.notDesktop': 'This works only in the desktop app.'
	}
};

export type Messages = typeof en;
