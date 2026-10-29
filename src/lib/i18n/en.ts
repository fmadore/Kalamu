// The English interface. Every visible string lives here so French and German can be added as
// `fr.ts` and `de.ts` typed `: Messages` without touching components, as in Live Translation.

export const en = {
	locale: { name: 'English', tag: 'en' },
	app: {
		name: 'Kalamu',
		tagline: 'Transcription and text extraction'
	},
	shell: {
		jobs: 'Jobs',
		running: 'Running',
		unfinished: 'Unfinished',
		finished: 'Finished',
		noJobs: 'Jobs you start appear here.',
		newJobShortcut: 'Ctrl N',
		offlineReady: 'Whisper ready on this PC',
		collapse: 'Hide the job list',
		expand: 'Show the job list',
		sidebarShortcut: 'Ctrl B'
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
		audio_transcriptionHint: 'Recordings and video files on this PC: MP3, M4A, WAV, MP4',
		youtube: 'Transcribe YouTube',
		youtubeHint: 'Public YouTube videos, from their link. Nothing is downloaded',
		analysis: 'Analyse texts',
		analysisHint:
			'Summaries, people and places, tone: you choose what to find. Texts, Word files, PDFs and earlier results',
		contents: 'Detailed table of contents',
		contentsHint:
			'Newspaper and magazine issues, archival files, books: one entry per article or item, with its pages',
		text_correction: 'Correct existing text',
		text_correctionHint:
			'Machine-read text from a digital library or a PDF, corrected page by page: TXT, Word, PDF'
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
		contentsLanguageHint:
			'Choose a language to have every entry’s summary written in it, whatever the language of the pages. Titles and names stay as printed.',
		languageHint:
			'Choosing helps with short or noisy material. Pick several for documents or recordings that mix languages.',
		addLanguage: 'Add a language…',
		findLanguage: 'Type to find a language',
		favouriteLanguages: 'Favourites',
		allLanguages: 'All languages',
		noLanguageFound: 'No language by that name.',
		addFavourite: (language: string) => `Add ${language} to favourites`,
		removeFavourite: (language: string) => `Remove ${language} from favourites`,
		favouritesHint: 'Star the languages you use most to find them first.',
		languagesNarrowed: (models: string) =>
			`Only the languages ${models} can transcribe are listed.`,
		languageUnsupported: (language: string, models: string) =>
			`${models} cannot use ${language}: remove it or choose another model.`,
		languageIgnoredAll: (models: string) =>
			`${models} recognises the language on its own, so there is nothing to choose.`,
		languageIgnoredSome: (models: string) =>
			`${models} recognises the language on its own; your choice applies to the other models.`,
		setupsTitle: 'Model and prompt',
		setupsHint: 'Add a second model to compare their results side by side.',
		provider: 'Provider',
		model: 'Model',
		prompt: 'Prompt',
		noPrompt: 'This model takes no prompt',
		label: 'Label',
		compareAdd: 'Compare with another model',
		analysis: 'Analysis',
		kindOfDocument: 'Kind of document',
		finds: 'Fills in',
		entryHas: 'Each entry has',
		mergeTitle: 'Combine the pages with',
		mergeHint:
			'Reading each page is the hard part. Combining the pages into entries is easier, so a smaller, cheaper model, from any provider, usually does it well.',
		mergeRow: (model: string) => `${model}, combining the pages`,
		editFields: 'Change the fields in Prompts',
		pipelinesTitle: 'Start from a pipeline',
		pipelineSave: 'Save as a pipeline',
		pipelineName: 'Name of the pipeline',
		pipelineSaved: (name: string) => `Saved as “${name}”. It appears at the top of New job.`,
		pipelineApplied: (name: string) => `Settings of “${name}” applied. Add the files.`,
		pipelineDelete: (name: string) => `Delete the pipeline “${name}”`,
		pipelineDeleteConfirm: (name: string) =>
			`Delete the pipeline “${name}”? Jobs already run with it are kept.`,
		pipelineExport: (name: string) =>
			`Export “${name}”: models, settings and full prompts, to cite or share`,
		pipelineExported: (file: string) =>
			`Saved ${file}.json, to import into Kalamu, and ${file}.md, for a methods section.`,
		pipelineImport: 'Import a pipeline',
		pipelineImported: (name: string) => `Imported “${name}” with its prompts.`,
		linesTitle: 'Line breaks',
		lines: {
			running_text: 'Running text',
			running_text_hint:
				'Lines joined into paragraphs and words split across lines rejoined. Best for reading and for searching the text.',
			as_on_page: 'As on the page',
			as_on_page_hint:
				'Every line kept where it ends on the page, end-of-line hyphens included. Easiest to check against the scan, but a word split across two lines is harder to find by search.'
		},
		linesNoPromptAll: (models: string) =>
			`${models} takes no instructions and lays out lines its own way, so there is nothing to choose.`,
		linesNoPromptSome: (models: string) =>
			`${models} takes no instructions and lays out lines its own way; this choice applies to the other models.`,
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
		paceTitle: 'Speed and price',
		pace: {
			fast: 'Faster',
			fast_hint: 'Each page is sent on its own, and results arrive as they are read.',
			economy: 'Cheaper',
			economy_hint:
				'Every page is sent together to the provider’s batch service, at about half the price. Results can take up to a day; Kalamu can be closed in the meantime and collects them when it opens again.'
		},
		paceNotBatched: (names: string) =>
			`${names} has no batch service in Kalamu and still runs page by page, at its usual price.`,
		start: 'Start',
		startShortcut: 'Ctrl ↵',
		starting: 'Starting…',
		name: 'Job name',
		inspecting: 'Reading files…',
		linksTitle: 'Videos',
		linkField: 'YouTube links',
		linkPlaceholder: 'https://www.youtube.com/watch?v=…',
		addLinks: 'Add',
		linksHint:
			'Paste one link or several, one per line. Watch, youtu.be, Shorts and live links work; playlists and channels do not.',
		openVideo: 'Open on YouTube',
		openVideoNamed: (title: string) => `Open ${title} on YouTube`,
		linksAdded: (added: number, refused: number) =>
			[
				added === 1 ? '1 video added' : `${added} videos added`,
				refused > 0 ? (refused === 1 ? '1 link not added' : `${refused} links not added`) : ''
			]
				.filter(Boolean)
				.join(', '),
		saveIn: 'Save results in',
		linkFolder:
			'A video has no folder of its own, so its transcripts and provenance files need one.',
		needFolder: 'Choose a folder for the results to start.',
		estimateVideo:
			'A video’s length is known only once Gemini has read it, so the cost shows in Jobs as it runs.'
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
			'Reading this file needs FFmpeg, which was not found next to the app. Reinstall Kalamu, or convert the file to MP3.',
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
		youtubePublic:
			'Only public videos can be read: private and unlisted ones are refused. Kalamu asks YouTube for each video’s title when you add it; Google then reads the video itself, and nothing is downloaded to this PC.',
		youtubePreview:
			'Reading YouTube links is a preview feature of Gemini, and Google may change its limits. A free key reads up to 8 hours of video a day. Each video is transcribed in one go, which suits videos up to about two hours.',
		results:
			'Try a sample before a whole folder, compare two models on difficult material, check names, numbers and dates against the source, and keep the originals.',
		analysisUnit:
			'One file per document: a PDF of several pages is read as a whole and gets one set of answers. Put each article, letter or report in its own file. Several documents in one file get mixed answers, and one document spread over several files gets several partial ones.',
		analysisInput:
			"Scans and text both work: the model reads a PDF's pages itself. For faded print or handwriting, extracting and correcting the text first means every answer rests on a reading you have checked.",
		analysisFields:
			'Describe each field as you would to a research assistant, give choices that do not overlap, and ask for a reason. Try it on a few documents first, and compare two models where the answer is a judgement, such as tone.',
		contentsUnit:
			'One file per issue, archival file or volume: it is read page by page, then the pages are combined. Photographs of one file should first be put together into a single PDF.',
		contentsCheck:
			'Try the first pages as a sample, then check the page ranges and the articles that run over several pages: the review shows each entry beside its first page.',
		correctionInput:
			'Kalamu corrects the text already in the file: a PDF’s own text layer, or a text file from a digital library (Gallica, Europeana, Internet Archive…). A scan without a text layer needs Extract text instead.',
		correctionCheck:
			'The review highlights every change against the original text. Check names, numbers and dates, which machine reading gets wrong most often.'
	},
	// The diagram under the tasks in New job: how the work is split and put back together, with
	// the user's own files once they are added.
	pipeline: {
		title: 'How it works',
		replay: 'Play again',
		moreFiles: (n: number) => (n === 1 ? 'and 1 more file' : `and ${n} more files`),
		theModel: 'The model',
		extractSource: 'A PDF or images',
		extractSplit: 'Split into pages: one piece per page',
		extractSpreads: 'Double pages are split in two first',
		extractRead: (model: string) => `${model} reads each page on its own`,
		extractJoin: 'Joined into one text per file, page numbers kept',
		analysisSource: 'A text, Word file or PDF',
		analysisWhole: 'Read whole, as one piece, however many pages',
		analysisFill: (model: string) => `${model} fills in your fields`,
		analysisTable: 'One row of answers per document, ready for a spreadsheet',
		/** Stand-ins for a prompt's fields, when it has none to show. */
		exampleFields: ['Summary', 'People', 'Places', 'Tone'],
		document: (n: number) => `Document ${n}`,
		audioSource: 'A recording',
		// The length of a piece is DEFAULT_PIECE_MS in crates/media/src/audio.rs.
		audioCut: (pieces: number) =>
			pieces === 1
				? 'Short enough to send whole: one piece'
				: `Cut at pauses into ${pieces} pieces of at most 20 min`,
		audioCutExample: 'Cut at pauses into pieces of at most 20 min',
		audioRead: (model: string) => `${model} transcribes each piece`,
		audioJoin: 'Joined into one transcript, with timestamps',
		audioSpeakers: 'Speaker labels may differ from one piece to the next',
		youtubeSource: 'A public YouTube link',
		youtubeSend: 'Only the link leaves this PC: nothing is downloaded',
		youtubeRead: (model: string) => `${model} watches the whole video in one go`,
		youtubeJoin: 'A transcript with timestamps',
		thisPc: 'This PC',
		google: 'Google',
		contentsSource: 'A long document: an issue, an archival file, a volume',
		contentsRead: (model: string) => `${model} lists the articles or items on each page`,
		contentsContinues: 'Notes such as “continued on page 5” are kept',
		contentsMerge: (model: string) => `${model} combines the pages: one entry per article or item`,
		contentsTable: 'A detailed table of contents: each entry with its pages and your fields',
		/** Stand-ins for a table of contents' entries, before there are any. */
		exampleEntries: ['Editorial', 'Interview', 'Letters to the editor'],
		correctionSource: 'Text that already exists: a PDF’s text layer or a text file',
		correctionSplit: 'One piece per page, or per passage of a long text',
		correctionRead: (model: string) => `${model} corrects the machine-reading errors`,
		correctionCompare: 'Every change shown against the original text, to keep or undo'
	},
	scores: {
		title: 'Errors found in review',
		rates: (characters: string, words: string) =>
			`${characters} of characters and ${words} of words wrong`,
		summary: (rates: string, plain: string, on: string) => `${rates} (${plain}), ${on}`,
		noErrors: (on: string) => `No errors, ${on}`,
		oneIn: (n: string) => `about 1 character in ${n}`,
		many: 'half the characters or more',
		pages: (n: number) => (n === 1 ? 'on 1 reviewed page' : `on ${n} reviewed pages`),
		parts: (n: number) => (n === 1 ? 'on 1 reviewed part' : `on ${n} reviewed parts`),
		note: 'Counted against the pages you corrected or marked as reviewed, so errors you did not catch are not counted.'
	},
	jobs: {
		title: 'Jobs',
		empty: 'No jobs yet. Start one from New job.',
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
		openFolder: 'Show in folder',
		openFile: 'Open in its app',
		rename: 'Rename',
		nameLabel: 'Job name',
		progress: (done: number, total: number) => `${done} of ${total} pieces`,
		failed: (n: number) => (n === 1 ? '1 failed' : `${n} failed`),
		files: 'Files',
		livePreview: 'Latest result',
		outputs: 'Written',
		created: 'Created',
		task: 'Task',
		models: 'Models',
		pieces: 'Pieces',
		cost: 'Spent this run',
		interruptedNotice:
			'This job stopped before it finished, when the app closed or a key stopped working. Finished pieces are saved: resuming continues where it left off, and nothing is sent or paid twice.',
		pausedNotice: 'Paused. Resume to continue where it left off.',
		waitingNotice: (n: number, due: string) =>
			`${n === 1 ? '1 piece is' : `${n} pieces are`} with the provider’s batch service, answered by ${due} at the latest. Kalamu can be closed in the meantime: it collects the answers when it opens again.`,
		heldNotice: (n: number) =>
			`${n === 1 ? '1 piece is' : `${n} pieces are`} still with the provider’s batch service. Resume to collect the answers; nothing is sent or paid twice.`,
		pace: 'Speed',
		paceName: { fast: 'Faster', economy: 'Cheaper (batch)' },
		empty_new: 'Start a job'
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
		submitted: 'With the provider',
		failed: 'Failed'
	},
	review: {
		title: 'Review',
		back: 'Back to job',
		page: (label: string) => `Page ${label}`,
		passage: (n: number) => `Passage ${n}`,
		segment: (label: string) => `From ${label}`,
		document: 'Whole document',
		video: 'Whole video',
		openVideo: 'Open on YouTube',
		videoPlayer: 'The video, in YouTube’s player',
		videoHint: 'Click a line to play the video from there.',
		videoHintOpen: 'Click a line to open the video there on YouTube.',
		playVideo: 'Play the video',
		playVideoNote: 'Plays from YouTube, in its privacy-enhanced player.',
		videoBlocked:
			'This video cannot be played here: its owner may not allow it, or this PC is offline.',
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
		scoredPages: (n: number) => (n === 1 ? 'on 1 reviewed page' : `on ${n} reviewed pages`),
		noReference:
			'Correct a page, or mark it as reviewed, to score the models against it. Until then, the view shows where they disagree.',
		agreement: (pct: string) => `${pct} different`,
		noPreview: 'No preview for this file type.',
		notDone: 'Not processed yet.',
		saved: 'Saved; the result files were updated.',
		pieceReviewed: (label: string) => `${label}, reviewed`,
		zoomIn: 'Zoom in',
		zoomOut: 'Zoom out',
		play: 'Play',
		pausePlayback: 'Pause',
		position: 'Position in this piece',
		noAudio: 'This piece could not be played.',
		lines: 'Transcript lines',
		noTimings: 'This model gave no timings, so lines cannot follow the recording.',
		reviewedOf: (n: number, total: number) => `Reviewed ${n} of ${total}`,
		readingText: 'Reading text',
		correcting: 'Correcting',
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
		answers: 'Answers',
		noneFound: 'None found',
		onePerLine: 'One per line.',
		reviewedShortcut: 'Ctrl ↵',
		markNotReviewed: 'Mark as not reviewed',
		pages: 'Pages',
		differences: (n: number) => (n === 1 ? '1 difference' : `${n} differences`),
		noDifferences: 'The models agree',
		previousDifference: 'Previous difference',
		nextDifference: 'Next difference',
		differenceOf: (n: number, total: number) => `${n} of ${total}`,
		disagree: 'The models read this differently',
		corrected_here: 'You corrected this',
		useReading: 'Use this reading',
		inUse: 'In the text',
		ownReading: 'Your own reading',
		ownReadingPlaceholder: 'Type what the source says',
		apply: 'Apply',
		readingKept: "The model's text is kept; your choice is stored as a correction.",
		modelText: (label: string) => `${label}, model output`,
		models: 'Models',
		legendDiffer: 'Models disagree',
		legendCorrected: 'Your correction',
		fitWidth: 'Fit width',
		compare: 'Compare',
		compareWith: 'Text compared',
		closeCompare: 'Close the comparison',
		resizeSource: 'Resize the source and the text',
		resizeCompare: 'Resize the text and the compared text',
		addedHere: 'Another model adds words here',
		leftOutHere: 'This model leaves out words here',
		pickReading: 'Choose the correct reading',
		original: 'Original text',
		changedHere: 'The correction changed this',
		legendChanged: 'Changed from the original',
		originalSiglum: 'O',
		contentsTitle: 'Table of contents',
		entries: (n: number) => (n === 1 ? '1 entry' : `${n} entries`),
		entryPages: 'Pages',
		entryTitle: 'Title',
		addEntry: 'Add an entry',
		removeEntry: 'Remove this entry',
		goToPage: (page: string) => `Show page ${page}`,
		pageReading: 'Read on this page',
		pageNote: 'Also on this page',
		notCombined: 'The pages have not been combined yet.',
		editEntries: 'Correct the entries'
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
		mergeInstructions: 'Instructions for combining the pages',
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
			'An analysis is a set of fields to fill in for each document. Start from one of these, then duplicate it to make your own.',
		contentsHint:
			'A table of contents has two steps: instructions for reading each page, and instructions for combining the pages. Every entry has a title and pages; the fields below add what you want for each.'
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
		/** `{name}` marks a link; the names it shows are in `creditNames`. */
		credits:
			'Kalamu uses {ffmpeg} under the LGPL v2.1 (its {source} is published with each release), {opus} (BSD), {whisper} (MIT) and the {models} (MIT). The full licences are in the licences folder installed with the app.',
		creditNames: {
			ffmpeg: 'FFmpeg',
			source: 'source',
			opus: 'Opus',
			whisper: 'whisper.cpp',
			models: 'Whisper models'
		},
		privacy:
			'Nothing is sent to the developer: no files, keys, results or telemetry. AI output can contain fluent but wrong text; check a sample against the source.'
	},
	support: {
		title: 'Support development',
		/** Says plainly that paying changes nothing in the app. */
		note: 'Kalamu is an independent project by Frédérick Madore (University of Bayreuth). If it is useful to you, you can help fund its development. Supporting is entirely optional and does not unlock additional features.',
		link: 'Buy me a coffee'
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
		body: 'They can keep running in the background, with Kalamu in the notification area next to the clock. Or quit now: the jobs pause, finished pages are kept, and you resume them next time from Jobs.',
		background: 'Keep running in the background',
		quit: 'Pause and quit',
		stay: 'Cancel',
		hidden: 'Kalamu is still working. Click its icon in the notification area to open it again.'
	},
	tray: {
		open: 'Open Kalamu',
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
		retiring: 'retiring',
		opensInBrowser: ' (opens in your browser)',
		linkFailed: (url: string) => `This page could not be opened in your browser: ${url}`
	},
	error: {
		'job.not_found': 'This job no longer exists.',
		'job.database': 'The job database could not be read or written.',
		'job.running': 'This job is already running.',
		'job.empty': 'Add at least one file and choose a model.',
		'job.link_needs_folder':
			'Choose a folder for the results: a YouTube video has no folder of its own to write them next to.',
		'source.private':
			'This video is private, so it cannot be read. Only public videos can be transcribed.',
		'source.not_found': 'YouTube has no video at this link: it may have been removed.',
		'source.not_youtube':
			'This is not a link to a YouTube video. Use a watch, youtu.be or Shorts link; playlists and channels cannot be read.',
		'media.encrypted': 'This PDF is password-protected. Remove the password and add it again.',
		'media.unsupported': 'This file type is not supported yet.',
		'media.read': 'The file could not be read.',
		'prompt.invalid': 'The prompt could not be saved: it needs a name and instructions.',
		'prompt.no_fields': 'An analysis needs at least one field.',
		'prompt.unnamed_field': 'Every field needs a name.',
		'prompt.duplicate_field': 'Two fields have the same name.',
		'prompt.few_values': 'A choice needs at least two values to choose from.',
		'prompt.unnamed_pipeline': 'A pipeline needs a name and at least one model.',
		'prompt.invalid_file':
			'This is not a Kalamu presets or pipeline file, or it comes from a newer version of Kalamu.',
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
		'offline.unknown_model': 'This offline model is not known to this version of Kalamu.',
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
		'file.open': 'This file could not be opened. It may have been moved or deleted.',
		'app.internal': 'Something went wrong inside the app.',
		'app.notDesktop': 'This works only in the desktop app.'
	}
};

export type Messages = typeof en;
