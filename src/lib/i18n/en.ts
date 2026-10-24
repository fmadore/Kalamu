// The English interface. Every visible string lives here so French and German can be added as
// `fr.ts` and `de.ts` typed `: Messages` without touching components, as in Live Translation.

export const en = {
	locale: { name: 'English', tag: 'en' },
	app: {
		name: 'Scriptorium',
		tagline: 'Transcription and text extraction'
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
		audio_transcriptionHint: 'Interviews and recordings: MP3, M4A, WAV, FLAC'
	},
	newJob: {
		title: 'New job',
		taskLabel: 'What do you want to do?',
		filesTitle: 'Files',
		addFiles: 'Add files',
		addFolder: 'Add a folder',
		dropHint: 'Or drop files and folders anywhere on this window.',
		noFiles: 'No files yet.',
		remove: 'Remove',
		pages: (n: number) => (n === 1 ? '1 page' : `${n} pages`),
		splitSpreads: 'Split double pages',
		languageTitle: 'Language',
		languageAuto: 'Detect automatically',
		languagePick: 'Choose languages',
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
		removeSetup: 'Remove',
		noKey: (provider: string) => `Add a ${provider} key in Settings to use it.`,
		noProvider: 'Add an API key in Settings to choose a model.',
		diarize: 'Label speakers',
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
		large_file: 'This file is large; it will be split before sending.'
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
		empty: 'No jobs yet. Start one from New job.',
		resumeTitle: 'Unfinished jobs',
		resumeHint:
			'These jobs stopped before they finished, when the app closed or a key stopped working. Resume them to continue where they left off; finished pages are not sent again.',
		resume: 'Resume',
		pause: 'Pause',
		cancel: 'Cancel',
		retry: 'Retry failed pages',
		delete: 'Delete',
		deleteConfirm: 'Delete this job? Files already written stay on disk.',
		review: 'Review',
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
		zoomIn: 'Zoom in',
		zoomOut: 'Zoom out'
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
		save: 'Save',
		duplicate: 'Duplicate',
		copyName: (name: string) => `${name} (copy)`,
		reset: 'Reset to default',
		delete: 'Delete',
		export: 'Export…',
		import: 'Import…',
		imported: (n: number) => `${n} imported`,
		saved: 'Saved',
		unsaved: 'Unsaved changes'
	},
	settings: {
		title: 'Settings',
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
		csv: 'Spreadsheet (.csv)'
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
		close: 'Close',
		loading: 'Loading…',
		preview: 'preview'
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
