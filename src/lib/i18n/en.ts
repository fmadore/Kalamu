// The English interface. Every visible string lives here; other languages are `fr.ts` and the
// like, typed `: Messages` so a missing or extra key fails the type check, as in Live
// Translation.

/** A default field's words in another language: shown, never stored or sent to a model. */
export interface FieldText {
	label: string;
	help: string;
	/** A choice field's choices, by the English value the model answers with and Kalamu stores. */
	values?: Record<string, string>;
	/** The fields of each entry of a repeating group or of entries that quote the text, by key. */
	fields?: Record<string, FieldText>;
}

/** A default prompt's name and description, and its fields by key, in another language. */
export interface PresetText {
	name: string;
	description: string;
	fields?: Record<string, FieldText>;
}

export const en = {
	locale: { name: 'English', tag: 'en' },
	/** Languages a model lists that the system has no name for. */
	languageNames: { kea: 'Kabuverdianu', rup: 'Aromanian' } as Record<string, string>,
	/**
	 * The default prompts' names and descriptions, by id, where the interface language differs
	 * from the core's English, and their fields' labels, help and choices by field key (shown by
	 * `localiseFields`); empty here, since the core's text is English.
	 */
	presets: {} as Record<string, PresetText>,
	app: {
		name: 'Kalamu'
	},
	shell: {
		jobs: 'Jobs',
		running: 'Running',
		unfinished: 'Unfinished',
		toCheck: 'For you to check',
		finished: 'Finished',
		noJobs: 'Jobs you start appear here.',
		/** The key after the shortcut modifier (Ctrl, or ⌘ on a Mac: `platform.ts`). */
		newJobShortcut: 'N',
		/** At the sidebar's foot, once anything that runs a job offline is installed. */
		offlineReady: 'Models ready on this computer, no internet needed',
		/** The guide's link, in the first-launch welcome, the Glossary and Settings › About. */
		guideLink: 'Read the guide',
		/** This language's guide, relative to the English one (`./`): French is `fr/`. A language
		 * the guide is not written in keeps `./`. */
		guidePage: './',
		collapse: 'Hide the job list',
		expand: 'Show the job list',
		sidebarShortcut: 'B',
		settingsShortcut: ',',
		/** The menu a right click (Control-click on a Mac) opens on a job in the list. */
		jobMenu: (name: string) => `Actions for “${name}”`,
		addToProject: 'Add to a project',
		noProjects: 'No project yet. Create one under Projects.',
		addedTo: (project: string) => `Added to the project “${project}”.`,
		pauseFirst: 'Pause or cancel the job before deleting it.',
		showProjects: 'Show the projects',
		hideProjects: 'Hide the projects',
		/** Said while a finished job is dragged over the list of projects. */
		dropOnProject: 'Drop on a project to add the job to it',
		dragHint: 'Drag it onto a project in the list to add it there',
		/** The sidebar's foot: jobs paused, or cut short by a quit or a crash, to go on with. */
		toResume: 'To resume',
		resumeJob: (name: string) => `Resume “${name}”`,
		/** Under the job stopped last, the others stopped too, shown on asking. */
		resumeOthers: (n: number) => (n === 1 ? '1 more' : `${n} more`),
		/** Said politely once a job resumes from the sidebar's foot. */
		resumed: (name: string) => `“${name}” resumed.`
	},
	nav: {
		newJob: 'New job',
		projects: 'Projects',
		workflows: 'Workflows',
		jobs: 'Jobs',
		prompts: 'Prompts',
		settings: 'Settings',
		glossary: 'Glossary',
		/** Back to the screen left, in a masthead. */
		back: 'Back'
	},
	task: {
		text_extraction: 'Extract text',
		text_extractionHint: 'Printed and handwritten documents: PDF, JPEG, PNG, TIFF',
		audio_transcription: 'Transcribe audio',
		audio_transcriptionHint: 'Recordings and video files on this computer: MP3, M4A, WAV, MP4',
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
			'Machine-read text from a digital library or a PDF, corrected page by page: TXT, Word, PDF',
		translation: 'Translate a text',
		translationHint:
			'Text you already have, from a file, a PDF or an earlier result, put into another language page by page: TXT, Word, PDF'
	},
	useText: {
		button: 'Use this text in a new job',
		short: 'Use this text',
		hint: 'Sends the text, as you corrected it, to a new job without reading the files again. Corrections you make afterwards do not reach the new job.',
		analyse: 'Analyse it',
		analyseHint: 'People and places, a summary, its tone, or fields of your own.',
		correct: 'Correct it',
		correctHint:
			'A model fixes reading errors, keeping the spelling and wording; every change is shown.',
		translate: 'Translate it',
		translateHint:
			'Into a language you choose, everything kept in order; the review shows it beside the text.',
		workflow: 'Run a workflow on it',
		workflowHint: 'Several steps one after another, from your workflows or an example.',
		interview: 'Analyse this interview',
		interviewHint:
			'Its themes, keywords and the passages worth quoting, each with who speaks and when.'
	},
	/** "Analyse this interview": one question, the model said in one line, and one button. */
	interview: {
		title: 'Analyse this interview',
		lede: 'Kalamu sends the transcript, as you corrected it, to a model, which finds its themes, keywords and the passages worth quoting. A passage is kept only if its words are found in the transcript.',
		question: 'What is your research about?',
		questionHint:
			'A sentence or two helps the model pick out what matters to your work. You can leave it empty.',
		questionPlaceholder:
			'For example: how young traders find their first customers, and the help their families give them.',
		model: (name: string, provider: string) => `Model: ${name} (${provider})`,
		cost: (usd: string) => `Estimated cost: ${usd}`,
		/** Said in place of the cost when the transcript is too long for the model chosen. */
		tooLong: (model: string, words: string, most: string) =>
			`This transcript is too long for ${model}: about ${words} words, where it reads about ${most} at once. Choose another model under More choices.`,
		more: 'More choices',
		start: 'Analyse',
		preparing: 'Getting the transcript ready…',
		jobName: (file: string) => `Interview analysis: ${file}`,
		research: 'Your research',
		researchNew: 'A new description',
		keep: 'Keep this for my next interviews',
		keptName: 'Name',
		keptHint: 'You can change or delete it in Prompts.',
		openPrompts: 'Open Prompts',
		/** Said beside the cost when the transcript is too long to be read at once. */
		inParts: (n: number) =>
			`This interview is long: it will be read in ${n} parts, then brought together.`,
		/** Under More choices: the researcher's own themes or interview guide. */
		themes: 'Your own themes or interview guide',
		themesHint:
			'Optional. One theme per line. Each theme found says whether it comes from your list or from the interview, and a theme the interview does not take up is said to be missing.',
		themesPlaceholder: 'For example:\nFirst customers\nHelp from the family\nSavings groups'
	},
	/** "Which of these speak of my question?": a project's documents read against one question,
	 *  then filtered by the answer and kept or set aside by the researcher. */
	screening: {
		action: 'Which of these speak of my question?',
		title: 'Which of these documents speak of my question?',
		lede: 'Kalamu sends the text of each document, as you corrected it, to a model, which says whether it speaks of your question, why, and with which passages. A passage is kept only if its words are found in the document. “Unsure” is an honest answer the model may give.',
		question: 'Your research question',
		questionHint: 'In a sentence or two, as you would ask a colleague.',
		questionPlaceholder:
			'For example: what the market women asked of the town council, and how it answered, 1950 to 1960.',
		documents: (n: number) =>
			n === 1 ? '1 document will be read.' : `${n} documents will be read.`,
		noText: (titles: string, n: number) =>
			n === 1
				? `Left out, not read into text yet: ${titles}. Read it first with New job.`
				: `Left out, not read into text yet: ${titles}. Read them first with New job.`,
		undecided: (titles: string) =>
			`Left out, read more than once with no reading preferred: ${titles}. Choose one in the document first.`,
		nothing:
			'None of these documents has been read into text yet. Read them first with New job, then come back.',
		model: (name: string, provider: string) => `Model: ${name} (${provider})`,
		cost: (usd: string) => `Estimated cost: ${usd}`,
		/** Documents the model chosen cannot read even in parts: left out, the others read. */
		tooLong: (files: string, model: string, n: number) =>
			n === 1
				? `Left out, too long for ${model} even read in parts: ${files}. Another model, under More choices, may read it.`
				: `Left out, too long for ${model} even read in parts: ${files}. Another model, under More choices, may read them.`,
		more: 'More choices',
		start: 'Start',
		preparing: 'Getting the texts ready…',
		jobName: (question: string) => `My question: ${question}`,
		/** The filter of a project's documents by what an analysis answered. */
		filter: 'Filter by an answer',
		anyAnswer: 'Any answer',
		answerCount: (answer: string, n: number) => `${answer} (${n})`,
		group: (field: string, job: string) => `${field} · ${job}`,
		exportTable: 'Export the whole table',
		/** What the export holds, whatever answer the table is filtered by. */
		exportTableHint:
			'Every document this question was asked of, whatever its answer, as a spreadsheet and as CSV.',
		exported: 'Table exported, as a spreadsheet and as CSV.',
		/** The researcher's decision, kept as a tag of the documents. */
		keep: 'Keep',
		setAside: 'Set aside',
		keptTag: 'Kept',
		setAsideTag: 'Set aside',
		decided: (tag: string, n: number) =>
			n === 1 ? `1 document tagged “${tag}”` : `${n} documents tagged “${tag}”`,
		/** Above the answers of a long document read in parts, and a part that could not be read. */
		docReadInParts: (n: number) =>
			`This document was long: it was read in ${n} parts, then brought together.`,
		docPagesNotRead: (first: number, last: number) =>
			first === last
				? `Page ${first} could not be read: the answer leaves it out.`
				: `Pages ${first} to ${last} could not be read: the answer leaves them out.`,
		docPassagesNotRead: (first: number, last: number) =>
			`Passages ${first} to ${last} could not be read: the answer leaves them out.`,
		/** Under a document's title while the table is filtered by an answer. */
		passagesOf: (title: string) => `Passages quoted from “${title}”`,
		keepThis: (title: string) => `Keep “${title}”`,
		setAsideThis: (title: string) => `Set aside “${title}”`,
		decisionTakenOff: (tag: string) => `Tag “${tag}” taken off`,
		/** A document tagged both kept and set aside, perhaps in two interface languages. */
		decisionConflict: 'Kept and set aside: choose one',
		/** A question kept with the project, asked again of the documents added since. */
		notReadAgainst: (n: number, question: string) =>
			n === 1
				? `1 document has not been read against “${question}” yet.`
				: `${n} documents have not been read against “${question}” yet.`,
		askAgain: 'Read the new documents against this question',
		/** Beside a question asked under two versions of its prompt, in the filter. */
		askedSince: (when: string) => `first asked ${when}`,
		/** Asked again after its prompt was edited: the answers form a group of their own. */
		promptChanged:
			'The prompt has changed since this question was asked. The new answers will be kept apart from the earlier ones, in the filter and in the exported table.'
	},
	/** The cards of an answer that quotes the text: a theme, or a passage worth quoting. */
	cited: {
		listen: 'Listen',
		listenTo: (said: string) => `Listen: ${said}`,
		showInText: 'Show in the text',
		showInTextOf: (said: string) => `Show in the text: ${said}`,
		quotes: 'Quotations',
		noQuote: 'No quotation left.',
		edit: 'Edit',
		editEntry: (title: string) => `Edit “${title}”`,
		editThis: 'Edit this entry',
		remove: 'Remove',
		removeEntry: (title: string) => `Remove “${title}”`,
		removeThis: 'Remove this entry',
		removeQuote: (start: string) => `Remove the quotation “${start}”`,
		quoteRemoved: 'Quotation removed from the answers',
		entryRemoved: (title: string) => `“${title}” removed from the answers`,
		thisRemoved: 'Entry removed from the answers',
		leftOut: (n: number) =>
			n === 1
				? '1 quotation could not be found in the text and was left out.'
				: `${n} quotations could not be found in the text and were left out.`,
		dropped: (n: number) =>
			n === 1
				? '1 entry was left out: none of its quotations was found in the text.'
				: `${n} entries were left out: none of their quotations was found in the text.`,
		/** In the editor of the other answers, where quotations are not changed. */
		inCards: 'Themes and quotations are changed in their cards, outside this editor.',
		/** Above the answers of a long interview read in parts. */
		readInParts: (n: number) =>
			`This interview was long: it was read in ${n} parts, then brought together.`,
		/** A part that could not be read, by its minutes when the transcript has times. */
		partNotRead: (from: string | null, to: string | null, first: number, last: number) =>
			from && to
				? `The part from ${from} to ${to} could not be read: its themes and quotations are missing.`
				: from
					? `The part from ${from} to the end could not be read: its themes and quotations are missing.`
					: `Turns ${first} to ${last} could not be read: their themes and quotations are missing.`,
		readAgain: 'Read it again',
		readingAgain: 'The part is being read again.',
		/** The menu of a passage selected in the transcript, and what it does. */
		addQuotation: 'Add as a quotation',
		quoteAdded: (title: string) => `Quotation added to “${title}”`,
		addedByYou: 'Added by you'
	},
	units: {
		tab: 'Items',
		lede: 'Each article, letter or item as pages of this document, to read, analyse or export on its own. The table of contents stays as it is.',
		make: 'Make items from the entries',
		makeHint:
			'One item per entry, with its title, pages and fields. The items are then yours to correct: correcting the entries afterwards does not change them.',
		made: (n: number) => (n === 1 ? '1 item made.' : `${n} items made.`),
		unknownPages: (entry: number, pages: string) =>
			`Entry ${entry} names ${pages}, which this document does not have.`,
		unreadPages: (entry: number, pages: string) =>
			`The pages of entry ${entry} (“${pages}”) could not be read: give the item its pages.`,
		count: (n: number) => (n === 1 ? '1 item' : `${n} items`),
		selected: (n: number, total: number) => `${n} of ${total} selected`,
		select: (title: string) => `Select “${title}”`,
		selectAll: 'Select all the items',
		untitled: 'Untitled',
		noPages: 'No pages yet',
		shares: (page: string, titles: string) => `${page} also holds ${titles}.`,
		precision:
			'An item is made of whole pages: where two share a page, each takes all of it, and a model reading one also sees the other.',
		title: 'Title',
		pages: 'Pages',
		pagesHint: 'In reading order, as this document numbers them: 1, 3-5, 8 (left).',
		/** How a list of pages names a page's halves: "8 (left)". */
		halves: { left: 'left', right: 'right' },
		edit: 'Correct',
		editItem: (title: string) => `Correct “${title}”`,
		save: 'Save',
		cancel: 'Cancel',
		add: 'Add an item',
		remove: 'Remove',
		removeItem: (title: string) => `Remove “${title}”`,
		joinNext: 'Join with the next',
		joinHint: 'The next item’s pages are added after these: an article and its continuation.',
		split: 'Split',
		splitHint: 'Two items an entry took for one: the second starts at the page chosen.',
		splitAt: (page: string) => `${page} onwards`,
		extract: 'Extract the text',
		extractHint: 'A new job reads only the selected items’ pages, each item as a file of its own.',
		analyse: 'Analyse',
		analyseHint: 'A new analysis reads each selected item’s pages as one document.',
		export: 'Export',
		exportHint:
			'Each selected item’s pages as a PDF in a folder you choose, with a list of the items as an Excel spreadsheet, a CSV and JSON.',
		exported: (n: number) => (n === 1 ? '1 file written.' : `${n} files written.`),
		pdfOnly: 'Only a PDF’s items can be read or exported on their own for now.',
		notCombined: 'Items are made from the combined entries: run the job to the end first.'
	},
	workflows: {
		title: 'Workflows',
		lede: 'Pipelines run one after another on the same files: extract the text, check it, then analyse it.',
		newWorkflow: 'New workflow',
		import: 'Import a workflow',
		imported: (name: string) => `Imported “${name}” with its prompts.`,
		saved: 'Saved workflows',
		empty: 'No workflow saved yet. Start from an example below, or build one from your pipelines.',
		steps: (n: number) => (n === 1 ? '1 step' : `${n} steps`),
		version: (n: number) => `version ${n}`,
		runs: 'Runs',
		noRuns: 'Workflows you run appear here, with each step’s job.',
		started: (when: string) => `Started ${when}`,
		back: 'All workflows',
		name: 'Name of the workflow',
		stepsTitle: 'Steps',
		noSteps:
			'Add a first step: a prompt, as Kalamu ships it or as you wrote it, or one of your saved pipelines.',
		addStep: 'Add a step',
		addStepHint: 'Choose a prompt or a pipeline',
		fromPipelines: 'Your pipelines',
		fromPrompts: (task: string) => `Prompts · ${task}`,
		prompt: 'Prompt',
		promptMissing: 'A prompt no longer listed',
		language: 'Translate into',
		languageNone: 'Choose a language',
		saveNeedsStep: 'Add a step to save the workflow',
		stepNumber: (n: number) => `Step ${n}`,
		reads: 'Works on',
		readsFiles: 'The files or text you run it on',
		readsStep: (n: number, name: string) =>
			`The text from step ${n} (${name}), as you corrected it`,
		review: 'Let me check the results before going on',
		reviewHint:
			'Kalamu waits while you check and correct them in the review, and goes on when you press Continue.',
		moveUp: (n: number) => `Move step ${n} up`,
		moveDown: (n: number) => `Move step ${n} down`,
		remove: (n: number) => `Remove step ${n}`,
		save: 'Save',
		savedAs: (name: string, version: number) => `Saved “${name}”, version ${version}.`,
		savedNote:
			'Saving makes a new version. Runs already started keep the steps and prompts they started with.',
		duplicate: 'Duplicate',
		copyName: (name: string) => `${name} (copy)`,
		export: 'Export',
		exported: (file: string) =>
			`Written to ${file}, with a Markdown record beside it. No keys, files or folders are included.`,
		delete: 'Delete',
		deleteConfirm: (name: string) =>
			`Delete the workflow “${name}”? Its runs and their jobs are kept.`,
		run: 'Run on files',
		runTitle: 'Run this workflow',
		chooseFiles: 'Choose files',
		files: (n: number) => (n === 1 ? '1 file' : `${n} files`),
		runName: 'Name of the run',
		unsaved: 'Save the workflow before running it.',
		sends: 'What each step sends',
		sendsFiles: (provider: string) => `the files, to ${provider}`,
		sendsText: (n: number, provider: string) => `the text of step ${n}, to ${provider}`,
		offline: 'stays on this computer',
		costKnown: (usd: string) => `about ${usd}`,
		costLater: (n: number) => `known once step ${n} has read the files`,
		costUnknown: 'cost unknown',
		cloudAfterOffline:
			'A step reads on this computer, and a later one sends its text to a cloud provider. Remove the later step if the material must stay on this computer.',
		start: 'Start',
		fromProject: (name: string) => `Its jobs join the project “${name}”.`,
		onText: (files: string) =>
			`Runs on the text already read, as you corrected it: ${files}. Choose a workflow that starts by correcting or analysing.`,
		notOnText: 'starts from scans or recordings',
		runOf: (name: string, version: number) => (version > 0 ? `${name}, version ${version}` : name),
		diagram: {
			title: 'How the steps follow each other',
			replay: 'Play again',
			files: 'Your files',
			check: 'You check',
			checkNote: 'then press Continue',
			written: 'Results saved'
		},
		examplesTitle: 'Start from an example',
		useExample: 'Use this example',
		examplesHint:
			'Each example uses the models you have keys for. Open one, adjust it if you like, save it, then run it on your files.',
		examples: {
			letters: {
				name: 'Letters: read, check, then people and places',
				about: 'Scanned letters or reports, their text checked before names are listed.',
				steps: ['Read the text', 'People, places and organisations']
			},
			interviews: {
				name: 'Interviews: transcribe, check, then summarise',
				about: 'Recordings transcribed, the transcript corrected, then summarised for research.',
				steps: ['Transcribe the interview', 'Summary and keywords']
			},
			library: {
				name: 'Digital library text: correct, then summarise',
				about: 'Machine-read text from a digital library, corrected, then summarised.',
				steps: ['Correct the text', 'Summary and keywords']
			},
			translated: {
				name: 'Documents in another language: read, check, translate, then summarise',
				about:
					'Scanned documents read and checked, translated into the language you choose, then summarised from the translation.',
				steps: ['Read the text', 'Translate the text', 'Summary and keywords']
			}
		},
		stepState: {
			waiting: 'Waiting',
			running: 'Running',
			paused: 'Not started',
			review: 'Ready for you to check',
			attention: 'Some pages not read',
			done: 'Done',
			done_with_errors: 'Done with errors',
			cancelled: 'Cancelled',
			blocked: 'Cannot start',
			failed: 'Could not start'
		},
		runStatus: {
			running: 'Running',
			needs_you: 'Waiting for you',
			finished: 'Finished',
			stopped: 'Stopped'
		},
		waitingFor: (n: number) => `Starts when step ${n} is done.`,
		reviewNotice:
			'Review and correct its results, then continue: the next steps read the text as you leave it.',
		attentionNotice:
			'Some pieces could not be read. Retry them on the job’s page, or continue with what was read: the pieces left out are named in the provenance.',
		pausedNotice: 'Its job is not running. Open it to start or resume it.',
		blockedNotice: 'The step it reads was cancelled, deleted or could not start.',
		failedNotice: (reason: string) => `It could not start: ${reason}`,
		continue: 'Continue',
		continueAnyway: 'Continue with what was read',
		openJob: 'Open the job',
		openReview: 'Review',
		approved: (when: string) => `Continued ${when}`,
		approvedPartial: (when: string) => `Continued ${when} with the pieces read`,
		cancel: 'Stop the run',
		cancelConfirm:
			'Stop this run? Running steps are cancelled and no further step starts. What was read is kept.',
		forget: 'Remove from this list',
		forgetConfirm: 'Remove this run from the list? Its jobs and their results are kept.',
		banner: {
			step: (n: number, of: number, run: string) => `Step ${n} of ${of} in “${run}”.`,
			review: (next: string) =>
				`When you have checked and corrected the text, press Continue: ${next} will then read it as you left it.`,
			attention: (next: string) =>
				`Some pages could not be read. Retry them, or continue: ${next} will read the pages that were.`,
			next: (next: string) => `Then: ${next}, once this job is done.`,
			nextAfterCheck: (next: string) => `Then: ${next}, once you have checked the text.`,
			see: 'See the workflow'
		},
		notify: {
			review: 'ready for you to check, then press Continue',
			attention: 'some pages were not read; retry them or continue'
		}
	},
	dictionary: {
		title: 'Dictionary',
		lede: 'The names and terms your analyses listed, field by field. Gather the spellings of one person, place or term into an entry under the label you prefer. The analyses’ answers stay as the models gave them.',
		coverage: (read: number, total: number) =>
			`Gathered from the analyses of ${read} of ${total === 1 ? '1 document' : `${total} documents`}.`,
		empty:
			'No analysis of these documents lists names yet. Run an analysis with a list field, such as “People, places and organisations”, on the project’s documents.',
		field: 'Field',
		fieldCount: (label: string, rows: number, toReview: number) =>
			`${label} (${rows}${toReview > 0 ? `, ${toReview} to review` : ''})`,
		scope: 'Documents',
		wholeProject: 'The whole project',
		search: 'Find a name',
		show: 'Show',
		filter: {
			all: 'All',
			to_review: 'To review',
			similar: 'Possible variants',
			reviewed: 'Reviewed',
			unseen: 'No longer found'
		},
		noMatch: 'Nothing matches.',
		columnLabel: 'Entry',
		columnDocuments: 'Documents',
		columnAgreement: 'Listed by',
		columnStatus: 'Status',
		agreement: (listed: number, read: number) =>
			`${listed} of ${read === 1 ? '1 result' : `${read} results`}`,
		agreementHint:
			'How many of the results that read these documents listed it. Two models listing a name is agreement, not a second mention.',
		reviewed: 'Reviewed',
		toReview: 'To review',
		unseen: 'No longer found',
		unseenHint:
			'None of its forms is listed by the analyses any longer: a job was deleted, or its answers corrected. The entry is kept.',
		similar: 'Possibly the same as',
		similarHint:
			'Spelled alike once capitals, accents, punctuation and word order are set aside. Only you decide whether they are one.',
		selectSimilar: 'Select them together',
		apart: 'Kept apart from',
		withdraw: (label: string) => `No longer keep apart from ${label}`,
		selected: (n: number) => (n === 1 ? '1 selected' : `${n} selected`),
		clearSelection: 'Clear',
		selectAll: 'Select all shown',
		select: (label: string) => `Select ${label}`,
		merge: 'Merge into one entry',
		mergeLabel: 'Label of the entry',
		mergeConfirm: 'Merge',
		markReviewed: 'Mark reviewed',
		markToReview: 'Mark to review',
		keepApart: 'Keep apart',
		keepApartHint: 'Two different people, places or terms, however alike they look.',
		details: (label: string) => `Details of ${label}`,
		forms: 'Forms as written',
		formDocuments: (n: number) => (n === 1 ? '1 document' : `${n} documents`),
		takeOut: (form: string) => `Take “${form}” out of this entry`,
		rename: 'Rename',
		label: 'Label',
		save: 'Save',
		cancel: 'Cancel',
		evidence: 'Where it was found',
		/** A spelling the researcher wrote in a corrected answer, not the model. */
		byHand: 'added or renamed by you',
		noEvidence: 'No result lists it any longer.',
		openReview: 'Open the review',
		export: 'Export',
		exported:
			'The dictionary was saved as JSON, with every form found beside it as an Excel spreadsheet and a CSV.',
		fileName: (project: string) => `${project} dictionary`,
		proposals: {
			title: 'Proposed merges',
			ask: 'Ask a model to propose merges',
			plan: (field: string, rows: number, groups: number) =>
				groups === 1
					? `All ${rows} rows of ${field} are sent in one request, with their spellings, the titles of up to three documents naming each, and the pairs you kept apart.`
					: `The ${rows} rows of ${field} are too many for one request: they are sent in ${groups} requests. First in alphabetical order with accents set aside, about 100,000 tokens at a time, each request sharing a tenth of its rows with the next. Then, so that names far apart in the alphabet are compared too, the rows that share a word or initials or sound alike, such as “Ouédraogo, Moussa” and “Moussa Ouédraogo”. Each row goes with its spellings, the titles of up to three documents naming it, and the pairs you kept apart.`,
			prompt: 'See or edit the prompt',
			sends: (provider: string) =>
				`Sent to ${provider} with your key, as an analysis job of its own: its cost and answers are in Jobs. Nothing is merged until you accept it.`,
			noKey: (provider: string) => `Add a key for ${provider} in Settings first.`,
			economy: 'Cheaper mode: half price, answered within a day',
			cost: (usd: string) => `About ${usd}.`,
			costUnknown: 'The cost could not be estimated.',
			start: 'Send',
			jobName: (field: string) => `Dictionary: merges in ${field}`,
			request: (status: string, answered: number, groups: number, open: number, date: string) =>
				`Asked on ${date}: ${status.toLowerCase()}, ${answered} of ${groups === 1 ? '1 group' : `${groups} groups`} answered, ${open === 1 ? '1 proposal' : `${open} proposals`} open.`,
			unknown: (n: number) =>
				n === 1
					? '1 reference to an unknown row, left out'
					: `${n} references to unknown rows, left out`,
			openJob: 'Open the job',
			forget: 'Forget',
			forgetHint: 'Forget this request and its proposals. The job stays in Jobs.',
			none: 'No proposal left: every one was merged, turned down or kept apart, or the model found none.',
			certainty: {
				certain: 'Certain',
				likely: 'Likely',
				possible: 'Possible'
			},
			certaintyHint: 'How sure the model says it is, in its own words. It is not a probability.',
			dismiss: 'Turn down',
			dismissedCount: (n: number) => (n === 1 ? '1 turned down' : `${n} turned down`),
			takeBack: 'Take back'
		}
	},
	search: {
		title: 'Search',
		lede: 'Every passage of the project’s documents that holds your words, in the text as you corrected it, each opening where it stands. Capitals, accents and Arabic vowel marks are ignored. Nothing is sent: the search runs on this computer.',
		query: 'Words to find',
		placeholder: 'A name, an organisation, a phrase…',
		go: 'Search',
		tooShort: 'Type two letters at least.',
		mode: 'Find',
		modes: {
			phrase: 'The exact phrase',
			all_words: 'All these words',
			near: 'These words near each other'
		},
		near: 'At most',
		nearWords: 'words apart',
		quotesHint:
			'Each word is looked for in the same page or answer. Put words that belong together between quotes, and OR in capitals between two words when either will do: “mission catholique” Koupéla OR Ouagadougou.',
		orHint:
			'Write OR in capitals between two phrases to find either: mission catholique OR mission protestante.',
		ignored: (words: string) => `Left out, too short to look for: ${words}.`,
		preparing: (done: number, total: number) =>
			`Preparing the text for searching: ${done} of ${total === 1 ? '1 page or part' : `${total} pages or parts`}…`,
		scope: 'Documents',
		wholeProject: 'The whole project',
		tag: 'Tag',
		anyTag: 'Any tag',
		task: 'Results of',
		anyTask: 'Every task',
		review: 'Pages',
		reviewFilter: { any: 'All', reviewed: 'Reviewed', not_reviewed: 'Not yet reviewed' },
		results: 'Readings',
		resultChoice: { preferred: 'The preferred one, where chosen', all: 'Every reading' },
		generated: 'Also search what models wrote about them: analyses and tables of contents',
		searching:
			'Searching… The first search of a project prepares its text, which can take a moment.',
		matches: (matches: number, documents: number) =>
			`${matches === 1 ? '1 match' : `${matches} matches`} in ${documents === 1 ? '1 document' : `${documents} documents`}`,
		listedOnly: (listed: number, matches: number) =>
			`The first ${listed} of ${matches} matches are listed. Narrow the search to see the rest.`,
		none: 'Not found in the text searched.',
		coverage: (searched: number, documents: number) =>
			`Searched the text of ${searched} of ${documents === 1 ? '1 document' : `${documents} documents`}.`,
		unread: (n: number) =>
			n === 1
				? '1 document has nothing read yet of the kinds searched: its content is not searched.'
				: `${n} documents have nothing read yet of the kinds searched: their content is not searched.`,
		piecesUnread: (n: number) =>
			n === 1
				? '1 page or part of the results searched is still to read, or could not be read.'
				: `${n} pages or parts of the results searched are still to read, or could not be read.`,
		withoutPreference: (n: number) =>
			n === 1
				? '1 document has several readings and none preferred: every reading was searched.'
				: `${n} documents have several readings and none preferred: every reading was searched.`,
		absence:
			'Finding nothing does not mean the documents never mention it: only the text read and kept is searched, and a model may have misread a name.',
		preferred: 'Preferred',
		generatedResult: 'Written by a model',
		writtenByYou: 'Written by you',
		writtenByYouHint: 'You added this name to the answers, or wrote it in your own form.',
		stale: 'Out of date',
		staleHint: 'The text this analysis read has been corrected since.',
		reason: 'reason',
		open: (place: string) => `Open ${place} in the review`,
		rebuild: 'Prepare the text again',
		rebuildHint:
			'The search keeps a copy of your results’ text, prepared for searching, in Kalamu’s data folder. Preparing it again starts from your results and corrections; nothing is lost.',
		rebuilt: 'The text will be prepared again at the next search.',
		moved:
			'The words are no longer where the search found them: the text was corrected since. The search in the review shows where they are now.'
	},
	meaning: {
		/** The choice above the project's search between its two kinds. */
		kind: 'Search by',
		kinds: { words: 'Words', meaning: 'Meaning' },
		lede: 'Passages close in meaning to a question or a description, even when they use other words or another language, in the text read from the documents as you corrected it. A model turns each passage into numbers once, when you prepare the project, and each question you type with it. You get passages to read: no answer, no summary.',
		query: 'Question or description',
		placeholder: 'Quarrels over who leads the Friday prayer…',
		go: 'Search',
		searching: 'Searching…',
		similarity: 'Similarity',
		similarityHint:
			'How close a passage is to your question as this model sees it, from −1 to 1. It is not a probability nor a mark of relevance: compare passages with each other, not with a threshold.',
		found: (listed: number, compared: number) =>
			`The ${listed === 1 ? 'closest document' : `${listed} closest documents`} of ${compared === 1 ? '1 compared' : `${compared} compared`}, closest first`,
		none: 'No passage was compared: prepare the project’s text first.',
		open: (place: string) => `Open ${place} in the review`,
		likeThis: 'Documents like this one',
		coverage: {
			compared: (compared: number, documents: number) =>
				`Compared the text of ${compared} of ${documents === 1 ? '1 document' : `${documents} documents`}.`,
			notPrepared: (n: number) =>
				n === 1
					? '1 document has text not yet prepared for this model: it was not compared.'
					: `${n} documents have text not yet prepared for this model: they were not compared.`,
			missing: (n: number) =>
				n === 1
					? '1 page or part is not prepared for this model yet.'
					: `${n} pages or parts are not prepared for this model yet.`,
			changed: (n: number) =>
				n === 1
					? '1 page or part was corrected since it was prepared: it is left out until you update.'
					: `${n} pages or parts were corrected since they were prepared: they are left out until you update.`,
			absence:
				'A passage not listed does not mean the documents never speak of it: only the text prepared is compared, and the model can miss what a word search finds.'
		},
		setup: {
			title: 'Set-up',
			show: 'Show the set-up',
			hide: 'Hide the set-up',
			noModels:
				'No model for the search by meaning is available in this copy of Kalamu yet. The search by words works as before.',
			model: 'Model',
			chooseModel: 'Choose a model…',
			local: 'On this computer: nothing is sent',
			cloud: (provider: string) => `Sent to ${provider}`,
			price: (usd: string) => `${usd} per million tokens`,
			languages: { many: 'many languages', english: 'mainly English' },
			gone: 'The model chosen for this project is no longer offered. Choose another: what the old one prepared is kept until you remove it below.',
			notInstalled:
				'This model is not downloaded yet. Download it in Settings › Offline models, then come back.',
			missingKey: 'No API key is saved for this provider. Save one in Settings › API keys.',
			openOffline: 'Open Offline models',
			openKeys: 'Open API keys',
			consentTitle: (provider: string) => `Send this project’s text to ${provider}?`,
			consentBody: (provider: string) =>
				`To prepare the search by meaning, Kalamu sends the corrected text of this project’s documents to ${provider}, which turns it into numbers. Afterwards, each question you type in the search by meaning is sent to ${provider} too. What ${provider} keeps is set by its own terms. Nothing is sent until you start preparing.`,
			consent: 'Agree to send it',
			consented: (provider: string, when: string) =>
				`On ${when} you agreed to send this project’s text and questions to ${provider}.`,
			estimate: 'What preparing will do',
			pieces: (n: number) => (n === 1 ? '1 page or part read' : `${n} pages or parts read`),
			prepared: (n: number) => `${n} already prepared`,
			missing: (n: number) => `${n} not prepared yet`,
			changed: (n: number) => `${n} corrected since they were prepared`,
			chunks: (n: number) => (n === 1 ? '1 passage to send' : `${n} passages to send`),
			chunksLocal: (n: number) => (n === 1 ? '1 passage to prepare' : `${n} passages to prepare`),
			chars: (n: string) => `${n} characters`,
			tokens: (n: string) => `about ${n} tokens`,
			cost: (usd: string) => `about ${usd}`,
			under: (usd: string) => `less than ${usd}`,
			noPrice: 'The price of this model is not known.',
			estimateNote:
				'An estimate: the provider counts tokens its own way. What it bills is recorded as preparing goes.',
			localNote: 'On this computer: nothing is sent and nothing is billed. It can take a while.',
			skipped: (n: number) =>
				n === 1
					? '1 passage with almost no letters (a page number, a stamp) is left out.'
					: `${n} passages with almost no letters (a page number, a stamp) are left out.`,
			nothingRead: 'Nothing has been read yet in this project to prepare.',
			prepare: 'Prepare the text',
			update: 'Update',
			resume: 'Continue',
			ready: (when: string) => `Everything read is prepared. Last finished on ${when}.`,
			readyNever: 'Everything read is prepared.',
			running: (done: number, total: number) => `Preparing: ${done} of ${total} passages…`,
			pause: 'Pause',
			cancel: 'Cancel',
			runs: 'Preparations',
			state: {
				running: 'Running',
				paused: 'Paused',
				cancelled: 'Cancelled',
				interrupted: 'Interrupted',
				failed: 'Failed',
				done: 'Finished'
			},
			run: (chunks: number, tokens: string) =>
				`${chunks === 1 ? '1 passage' : `${chunks} passages`}, ${tokens} tokens billed`,
			spent: (usd: string) => `Recorded for this project so far, questions included: ${usd}.`,
			queries: (n: number) => (n === 1 ? '1 question asked' : `${n} questions asked`),
			kept: 'Kept on this computer',
			keptLine: (chunks: number, size: string) =>
				`${chunks === 1 ? '1 passage' : `${chunks} passages`}, ${size}`,
			inUse: 'In use',
			remove: 'Remove',
			removed: 'Removed. Preparing the text again with this model would send it again.',
			lostNote:
				'These numbers are kept in Kalamu’s data folder. Removed or lost, they cost only what preparing the text again costs: your results and corrections are not touched.',
			size: (bytes: number) =>
				bytes >= 1e9
					? `${(bytes / 1e9).toFixed(1)} GB`
					: bytes >= 1e6
						? `${(bytes / 1e6).toFixed(1)} MB`
						: `${Math.max(1, Math.round(bytes / 1e3))} kB`
		},
		map: {
			title: 'Map',
			lede: 'Each document as one point, placed by its text as the model sees it: documents close together have similar wording and subject. Positions are approximate and the axes mean nothing. A long document’s point is the average of everything in it.',
			made: (model: string, when: string) =>
				`Made with ${model}, from the text prepared on ${when}.`,
			madeNever: (model: string) => `Made with ${model}.`,
			shown: (share: string) =>
				`Its two directions show about ${share} of the differences between the documents; the list of nearest documents beside it uses all of them.`,
			placed: (placed: number, documents: number) =>
				`${placed} of ${documents === 1 ? '1 document' : `${documents} documents`} placed.`,
			unread: (n: number) => `${n} with nothing read yet.`,
			notPrepared: (n: number) => `${n} not prepared for this model yet.`,
			changed: (n: number) => `${n} corrected since they were prepared.`,
			withoutPreference: (n: number) =>
				`${n} with several readings and none preferred: the first reading is placed.`,
			tooFew: (placed: number) =>
				`A map needs at least five documents placed, and ${placed === 0 ? 'none is' : placed === 1 ? 'only 1 is' : `only ${placed} are`}. Prepare more of the project’s text, or add documents.`,
			noModel:
				'The map is drawn from the text prepared for the search by meaning. Choose a model and prepare the text in Search, under Meaning.',
			openSetup: 'Open the search by meaning',
			groupBy: 'Group by',
			groups: { collection: 'Collection', tag: 'Tag' },
			inNone: 'In none',
			others: 'Others',
			legend: 'Legend',
			zoomIn: 'Zoom in',
			zoomOut: 'Zoom out',
			whole: 'Show the whole map',
			canvas:
				'Map of the documents. Arrow keys move from point to point, Enter opens the document, plus and minus zoom.',
			point: (title: string, group: string) => (group ? `${title}, ${group}` : title),
			documents: 'Documents placed',
			selected: 'Selected',
			nearest: 'Nearest documents',
			noneSelected:
				'Choose a point, or a document in the list, to see the documents nearest to it.',
			noNeighbours: 'No other document is placed.',
			openDocument: 'Open the document',
			chunks: (n: number) => (n === 1 ? '1 passage averaged' : `${n} passages averaged`)
		}
	},
	queue: {
		title: 'To check',
		lede: 'The pages and answers across these documents that need you, each with its reason. It is read from what Kalamu already keeps: nothing is sent to a model. Only you mark a page as reviewed; models agreeing is not a review.',
		scope: 'Documents',
		wholeProject: 'The whole project',
		selection: (n: number) => (n === 1 ? 'The selected document' : `The ${n} selected documents`),
		checkSelection: 'What to check',
		loading: 'Gathering what needs checking…',
		coverage: {
			documents: (read: number, total: number) =>
				`${read} of ${total === 1 ? '1 document' : `${total} documents`} read`,
			pieces: (processed: number, pieces: number) =>
				`${processed} of ${pieces === 1 ? '1 page or part' : `${pieces} pages or parts`} read`,
			unprocessed: (n: number) => `${n} failed or not read yet`,
			reviewed: (reviewed: number, processed: number) => `${reviewed} of ${processed} reviewed`,
			flagged: (flagged: number, unresolved: number) =>
				`${flagged} flagged, ${unresolved} still open`
		},
		coverageHint:
			'Counted over every result read below, whatever is shown. The reviewed pages are the only ones checked; flagged pages are not a sample of the whole.',
		results: 'Results read',
		resultsHint:
			'Where you chose a preferred result for a document, only it is read. Elsewhere every result is read and listed here: set aside those you do not want to check. The project keeps what you set aside for this list, and pages are drawn at random without them. A document read by several models is still one document.',
		preferred: (n: number) =>
			n === 1
				? '1 document read through its preferred result'
				: `${n} documents read through their preferred result`,
		resultChoice: (label: string, job: string, documents: number) =>
			`${label}, ${job} (${documents === 1 ? '1 document' : `${documents} documents`})`,
		show: 'Show',
		reason: {
			failed: 'Failed',
			not_read: 'Not read',
			truncated: 'Cut off',
			disagrees: 'Models disagree',
			stale: 'Input changed',
			not_reviewed: 'Not reviewed'
		},
		showResolved: 'Include reviewed',
		processing: 'Failed or not read',
		processingHint:
			'Nothing was read here, so there is nothing to check yet: retry the failed pages on the job’s page, or start the job.',
		answers: 'Answers to check',
		none: 'Nothing needs you for the reasons shown.',
		empty:
			'Nothing has been read here yet. Once a job has read these documents, this list shows the pages that failed or were cut off, that two models read differently, or that are not yet reviewed.',
		detail: {
			failed: (error: string | null) => (error ? `Failed: ${error}` : 'Failed'),
			notRead: 'Not read yet',
			neverStarted: 'Not started yet',
			truncated: 'Stopped at the model’s length limit: the end may be missing',
			truncatedFilter: 'Stopped by the provider’s filter: the end may be missing',
			truncatedOther: 'Stopped early by the provider: the end may be missing',
			disagreesWords: (labels: string, words: number) =>
				`Reads differently from ${labels}: ${words === 1 ? '1 word' : `${words} words`}`,
			disagreesFields: (labels: string, fields: string) =>
				`Answers differently from ${labels}: ${fields}`,
			stale: 'Read a text that has been corrected since',
			gone: 'Read a text whose job has since been deleted',
			unitChanged: 'Read an item whose pages have changed since',
			unitGone: 'Read an item that has since been deleted',
			notReviewed: 'Not reviewed yet'
		},
		wholeResult: 'The whole result',
		preferredBadge: 'Preferred',
		reviewedBadge: 'Reviewed',
		answered: (when: string) => `Answered ${when}`,
		answeredHint: 'When the answer was stored. A new answer is reviewed afresh.',
		open: (document: string, place: string) => `Open ${document}, ${place}, in the review`,
		more: (n: number) => `Show ${n} more`,
		export: 'Export the list',
		exported: 'The list was saved as JSON, with an Excel spreadsheet and a CSV beside it.',
		fileName: (project: string) => `${project} review list`,
		sample: {
			title: 'Drawn at random',
			hint: 'Pages drawn at random from those nothing flags, to see how the rest of these documents reads. Flagged pages were picked for a problem: what you find among them says nothing of the whole.',
			size: 'Pages',
			draw: 'Draw at random',
			drawAgain: 'Draw again',
			drawAgainHint:
				'A new seed, and so other pages. The pages drawn now will no longer be marked.',
			remove: 'Remove the sample',
			summary: (drawn: number, candidates: number) =>
				`${drawn} of the ${candidates === 1 ? '1 page' : `${candidates} pages`} nothing flags`,
			seed: (seed: number) => `Seed ${seed}`,
			seedHint:
				'The same seed draws the same pages from the same documents: note it to cite this sample.',
			drawnOn: (when: string) => `drawn ${when}`,
			reviewed: (reviewed: number, drawn: number) => `${reviewed} of ${drawn} reviewed`,
			none: 'No page here is read and free of flags, so there is nothing to draw.',
			badge: 'Drawn at random',
			seedField: 'Seed',
			seedFieldHint:
				'Leave it empty for a new draw, or type the seed of a sample, read in a manifest or an article, to draw the same pages again.',
			seedInvalid: 'A seed is a number of six digits at most.'
		},
		keep: {
			action: 'Keep aside',
			label: (document: string, place: string) => `Keep ${document}, ${place}, aside`,
			note: 'Note',
			notePlaceholder: 'Why, and what to do later',
			save: 'Keep aside',
			cancel: 'Cancel',
			putBack: 'Put back',
			badge: 'Kept aside',
			earlier: 'Kept aside with an earlier answer',
			earlierHint: (note: string) =>
				`The answer has changed since you kept it aside${note ? ` (“${note}”)` : ''}: this one is listed with the others.`,
			section: 'Kept aside',
			sectionHint:
				'Items you set apart for later, with your note. They stay listed and still need you: keeping an item aside resolves nothing.'
		},
		banner: {
			position: (at: number, total: number) => `${at} of ${total} to check`,
			previous: 'Previous to check',
			next: 'Next to check',
			back: 'Back to the list'
		}
	},
	projects: {
		title: 'Projects',
		lede: 'Your documents, grouped as you like, with everything read from each.',
		newProject: 'New project',
		name: 'Name',
		description: 'Description',
		descriptionHint: 'What the project is about, for you and for colleagues.',
		create: 'Create',
		cancel: 'Cancel',
		save: 'Save',
		empty:
			'No project yet. A project keeps your documents together, in collections, with every transcription, extraction and analysis made of them.',
		showArchived: 'Show archived projects',
		archived: 'Archived',
		counts: (documents: number, collections: number, jobs: number) =>
			`${documents === 1 ? '1 document' : `${documents} documents`} · ${
				collections === 1 ? '1 collection' : `${collections} collections`
			} · ${jobs === 1 ? '1 job' : `${jobs} jobs`}`,
		updated: (date: string) => `Changed ${date}`,
		back: 'Projects',
		edit: 'Edit',
		archive: 'Archive',
		restore: 'Restore',
		archivedNotice:
			'This project is archived. Nothing in it has changed; restore it to list it again.',
		delete: 'Delete project',
		deleteConfirm: (name: string, copied: number) =>
			copied === 0
				? `Delete the project “${name}”? Its collections and documents are removed from Kalamu. The files on your computer are kept, and the jobs and their results stay under Jobs.`
				: `Delete the project “${name}”? Its collections and documents are removed from Kalamu, with ${copied === 1 ? 'the original file' : `the ${copied} original files`} Kalamu copied into its data folder when it opened the project file. Your own files elsewhere are kept, and the jobs and their results stay under Jobs, showing ${copied === 1 ? 'that file' : 'those files'} as missing.`,
		allDocuments: 'All documents',
		collections: 'Collections',
		newCollection: 'New collection',
		collectionName: 'Name of the collection',
		renameCollection: 'Rename',
		deleteCollection: 'Delete collection',
		deleteCollectionConfirm: (name: string) =>
			`Delete the collection “${name}”? Its documents stay in the project.`,
		activity: 'Activity',
		activityLede: 'The jobs that read this project’s documents, newest first.',
		noActivity:
			'No job has read these documents yet. Select documents and choose “New job with these”.',
		activityFilters: 'Filter the jobs',
		filterBy: {
			task: 'Task',
			status: 'State',
			model: 'Model',
			workflow: 'Workflow',
			period: 'Started'
		},
		anyTask: 'Any task',
		anyStatus: 'Any state',
		anyModel: 'Any model',
		anyWorkflow: 'In a workflow or not',
		period: {
			any: 'Any time',
			week: 'Past 7 days',
			month: 'Past 30 days',
			year: 'Past year'
		},
		clearFilters: 'Show all',
		noActivityMatch: 'No job matches these filters.',
		activityShown: (shown: number, total: number) =>
			shown === total ? (total === 1 ? '1 job' : `${total} jobs`) : `${shown} of ${total} jobs`,
		activityCost: (cost: string) => `${cost} spent`,
		activityUnpriced: (n: number) =>
			`${n === 1 ? '1' : n} without a price (offline or not yet answered)`,
		addFiles: 'Add files',
		addFolder: 'Add a folder',
		addedFiles: (n: number) => (n === 1 ? '1 document added' : `${n} documents added`),
		/** Over the project while files are dragged over the window. */
		dropHere: 'Drop here to add them to the project',
		removedFromCollection: (n: number, collection: string) =>
			`${n === 1 ? '1 document' : `${n} documents`} removed from “${collection}”`,
		notReadable: (n: number) =>
			n === 1
				? '1 file could not be read and was left out.'
				: `${n} files could not be read and were left out.`,
		attachJob: 'Add an existing job',
		noDocuments:
			'No documents here yet. Add files, or add a job you ran before: its results come with it, without being paid for again.',
		/** In a project with no documents yet, in place of the table: what to do first. */
		start: {
			title: 'Start by adding your documents',
			first: (files: string, folder: string, job: string) =>
				`Choose “${files}” or “${folder}” for the sources you want to read. Already ran a job on them? “${job}” brings in its results, without paying again.`,
			rest: 'Nothing else has to be filled in. The other sections fill themselves as jobs read your documents: the search, the map, what to check, the dictionary and the activity.',
			/** Above a project's search while nothing in it has been read. */
			nothingToSearch:
				'Nothing has been read in this project yet. Once a job has read its documents, their text can be searched here, by words or by meaning.'
		},
		search: 'Find a document',
		noMatch: 'No document matches.',
		sortBy: 'Sort by',
		sortTitle: 'Title',
		sortAdded: 'Date added',
		sortRun: 'Last read',
		columnTitle: 'Document',
		columnKind: 'Kind',
		columnCollections: 'Collections and tags',
		anyTag: 'Any tag',
		tagFilter: 'Tag',
		tagCount: (tag: string, n: number) => `${tag} (${n})`,
		addTag: 'Add a tag',
		addTagHint: 'A new tag, or one already used',
		tagButton: 'Tag',
		removeTag: 'Remove a tag…',
		removeTagFrom: (tag: string) => `Remove the tag ${tag}`,
		renameTag: 'Rename tag',
		renameTagTo: (tag: string) => `New name for “${tag}”`,
		tagRenamed: (n: number) => (n === 1 ? 'Renamed on 1 document.' : `Renamed on ${n} documents.`),
		describe: {
			title: 'Source description',
			lede: 'The original as an archive or library describes it, as you would cite it. You write it, never a model, and no result changes it. A part left empty takes the project’s.',
			projectTitle: 'Shared by its documents',
			projectLede:
				'Parts of the description every document shares, such as the repository or the fonds. A document’s own value takes their place.',
			fields: {
				repository: 'Repository',
				fonds: 'Fonds or collection',
				series: 'Series',
				file: 'Box or folder',
				shelfmark: 'Shelfmark or call number',
				creator: 'Creator',
				date: 'Date',
				url: 'Web address',
				citation: 'Citation',
				rights: 'Rights'
			},
			fromProject: 'From the project',
			save: 'Save description',
			saved: 'Description saved.',
			export: 'Export descriptions',
			exported:
				'The descriptions were saved as JSON, with an Excel spreadsheet and a CSV beside it.'
		},
		profiles: {
			title: 'Export descriptions',
			/** The file's name the save dialog suggests. */
			fileName: (project: string) => `${project} descriptions`,
			lede: 'Kalamu’s own export gives every part of each description with where it came from. A profile writes the columns you choose, in your order and under your headings, for a spreadsheet or another tool’s import. It is kept for every project and holds none of this one’s values.',
			columns: 'Columns',
			kalamu: 'Every part, with where it came from',
			newProfile: 'New profile',
			starters: {
				every: 'From every column',
				dublin_core: 'From Dublin Core terms'
			},
			import: 'Import a profile…',
			imported: (name: string) => `Profile “${name}” imported.`,
			edit: 'Edit',
			duplicate: 'Duplicate',
			copyName: (name: string) => `${name} (copy)`,
			share: 'Share…',
			shareFixed:
				'Include the fixed texts? They may belong to this project. Without them the columns stay, their texts left empty.',
			shared: 'The profile was saved as a file another Kalamu can import.',
			delete: 'Delete',
			deleteConfirm: (name: string) =>
				`Delete the profile “${name}”? Files already exported stay as they are.`,
			name: 'Name',
			heading: 'Heading',
			value: 'Value',
			fixedText: 'Text in every row',
			groups: {
				document: 'The document',
				parts: 'Its description',
				from: 'Where each part came from',
				result: 'Its result for a task',
				other: 'Other'
			},
			tasks: {
				text_extraction: 'Text extraction',
				text_correction: 'Text correction',
				translation: 'Translation',
				audio_transcription: 'Transcription',
				analysis: 'Analysis',
				contents: 'Table of contents'
			},
			resultValues: {
				status: (task: string) => `${task}: how far reviewed`,
				model: (task: string) => `${task}: model`,
				text: (task: string) => `${task}: text, as corrected`
			},
			answer: 'Analysis: an answer',
			labelled: 'Analysis: an answer, under the dictionary’s labels',
			answerKey: 'Field',
			resultHint:
				'A result’s column reads the document’s preferred result for the task, or its only one. When it has several and none is preferred, the cell is left empty: choose one on the document’s page. Under the dictionary’s labels, each name an entry gathers is written under its label, and a name no entry holds as the model gave it.',
			values: {
				document_id: 'Document id',
				title: 'Title',
				file_name: 'File name',
				path: 'Path',
				project: 'Project name',
				collections: 'Collections',
				tags: 'Tags',
				fixed: 'A fixed text'
			},
			fromPart: (part: string) => `${part}: where it came from`,
			moveUp: (n: number) => `Move column ${n} up`,
			moveDown: (n: number) => `Move column ${n} down`,
			remove: (n: number) => `Remove column ${n}`,
			addColumn: 'Add a column',
			separator: 'Between the items of a list',
			separatorHint: 'In a CSV cell; JSON keeps lists as lists.',
			missing: 'What an empty cell holds',
			missingHint: 'Nothing if left blank; JSON writes null.',
			joinLines: 'Line breaks inside a value made a space',
			problems: {
				no_name: 'Give the profile a name.',
				no_columns: 'Add at least one column.',
				unnamed: (column: number) => `Column ${column} needs a heading.`,
				bad_heading: (column: number) => `The heading of column ${column} must fit on one line.`,
				duplicate: (column: number, first: number) =>
					`Column ${column} has the same heading as column ${first}; capitals do not tell them apart.`,
				bad_separator: 'The list separator cannot be empty or hold a line break.',
				bad_value: (column: number) =>
					`Column ${column} asks for what Kalamu cannot give: a text from a task that writes one, or an answer named by its field.`
			},
			save: 'Save profile',
			saved: 'Profile saved.',
			cancel: 'Cancel',
			export: 'Export…',
			close: 'Close'
		},
		columnResults: 'Results',
		columnDate: 'Added',
		selectAll: 'Select all documents shown',
		select: (title: string) => `Select ${title}`,
		selected: (n: number) => (n === 1 ? '1 selected' : `${n} selected`),
		clearSelection: 'Clear',
		addTo: 'Add to a collection…',
		removeFromCollection: 'Remove from this collection',
		newJobWith: 'New job with these',
		workflowWith: 'Run a workflow',
		removeDocuments: 'Remove from project',
		removeDocumentsConfirm: (n: number) =>
			`Remove ${n === 1 ? 'this document' : `these ${n} documents`} from the project? The files, the jobs that read them and their results stay.`,
		missing: 'File missing',
		missingHint:
			'The file is not where it was added, or this project came from another computer without its files. Its results are kept; put the file back to read it again.',
		sameContent: (n: number) =>
			n === 1 ? 'Same content as 1 other' : `Same content as ${n} others`,
		noResults: 'Not read yet',
		kind: {
			pdf: 'PDF',
			image: 'Image',
			audio: 'Recording',
			video: 'Video',
			text: 'Text',
			youtube: 'YouTube'
		},
		/** A task's marks in the table's Results column, in words. */
		taskResults: (task: string, n: number, reviewed: number, preferred: boolean) =>
			`${task}: ${n === 1 ? '1 result' : `${n} results`}${reviewed > 0 ? `, ${reviewed} reviewed` : ''}${preferred ? (n === 1 ? ', preferred' : ', one of them preferred') : ''}`,
		resultCount: (n: number) => (n === 1 ? '1 result' : `${n} results`),
		reviewedMark: (n: number) => (n === 1 ? '1 reviewed' : `${n} reviewed`),
		preferredMark: 'Preferred: the result relied on for this document',
		fromProject: (name: string) =>
			`Started from the project “${name}”: the job’s files become its documents, and its results theirs.`,
		linkFailed: (name: string) =>
			`The job started, but could not be added to the project “${name}”. Add it from the project with “Add an existing job”.`,
		attach: {
			title: 'Add an existing job',
			lede: 'Its results join the documents they were read from. Nothing is sent again.',
			job: 'Job',
			chooseJob: 'Choose a job',
			file: 'File',
			document: 'Document',
			newDocument: 'A new document',
			skip: 'Leave out',
			proposed: (title: string) => `${title} (the same file)`,
			sameContent: (title: string) => `${title} (same content, another file)`,
			already: (title: string) => `${title} (already added)`,
			sameContentHint:
				'Files with the same content may be copies of one item, or two items that happen to match. Choose for each.',
			confirm: 'Add',
			attached: (n: number) =>
				n === 1 ? '1 file added to the project' : `${n} files added to the project`
		},
		document: {
			back: (project: string) => project,
			source: 'Original',
			reveal: 'Show in folder',
			collections: 'Collections',
			notInCollection: 'In no collection.',
			tags: 'Tags',
			noTags: 'No tag yet. Tags are labels of your own, to find documents across collections.',
			rename: 'Rename',
			title: 'Title',
			results: 'Results',
			noResults: 'Nothing has been read from this document yet.',
			job: 'Job',
			setup: 'Model',
			progress: 'Read',
			reviewed: 'Reviewed',
			cost: 'Cost',
			preferred: 'Preferred',
			prefer: 'Make preferred',
			preferHint: 'The result you rely on for this task. A newer run never replaces it.',
			open: 'Review',
			openJob: 'Job page',
			detach: 'Untie',
			detachConfirm: 'Untie this result from the document? The job and its results stay.',
			pieces: (finished: number, total: number, failed: number) =>
				`${finished} of ${total}${failed > 0 ? `, ${failed} failed` : ''}`,
			reviewedCount: (reviewed: number, total: number, corrected: number) =>
				`${reviewed} of ${total}${corrected > 0 ? `, ${corrected} corrected` : ''}`,
			sameContentTitle: 'Same content',
			sameContentLede:
				'These documents have exactly the same content. If they are one item, merge them: their results and collections come together, and nothing is read again.',
			merge: 'Merge into this one',
			mergeConfirm: (other: string) =>
				`Merge “${other}” into this document? Its results and collections move here and it is removed.`
		}
	},
	newJob: {
		/** Empties the form kept from an earlier visit. */
		startOver: 'Start over',
		startOverHint: 'Empty the form: files, models and settings',
		then: {
			title: 'Then (optional)',
			hint: 'Once the text is read, Kalamu can analyse it for you: the people and places it names, a summary, its tone.',
			analyse: 'Analyse the text',
			nothing: 'Nothing more',
			check: 'Let me check the text first',
			checkHint:
				'Recommended. Correct what the model misread; the analysis then reads your corrected text.',
			whatHappens: (analysis: string, model: string) =>
				`When the text is read, ${model} analyses it (${analysis}) straight away. Its cost depends on the text, so it is known once the text is read.`,
			whatHappensChecked: (analysis: string, model: string) =>
				`When the text is read, check it in the review, then press Continue: ${model} analyses it (${analysis}). Nothing is sent for the analysis before you do. Its cost depends on the text.`,
			workflowName: (task: string, analysis: string) => `${task}, then ${analysis}`
		},
		title: 'New job',
		lede: 'Choose the files, a model and a prompt. The cost is estimated before anything is sent.',
		taskLabel: 'What do you want to do?',
		filesTitle: 'Files',
		addFiles: 'Add files',
		addFolder: 'Add a folder',
		dropHint: 'Or drop files and folders anywhere on this window.',
		/** Over New job while files are dragged over the window. */
		dropHere: 'Drop here to add them to the job',
		/** Files and folders added that the task chosen does not read. */
		setAside: {
			files: (names: string, task: string, n: number) =>
				n === 1
					? `Not added: ${names}. ${task} does not read this kind of file.`
					: `Not added: ${names}. ${task} does not read these kinds of file.`,
			folders: (names: string, task: string) =>
				`Nothing added from ${names}: no file in it that ${task} reads.`,
			kinds:
				'Kalamu reads PDFs, images (JPEG, PNG, TIFF, WebP, HEIC), recordings, video files and texts (TXT, Markdown, Word, CSV).',
			youtube:
				'Here videos are added by their YouTube link. Recordings and video files on this computer are read by “Transcribe audio”.',
			instead: (task: string) => `${task} instead`,
			insteadHint: 'Switches the task and adds these files to it',
			dismiss: 'Dismiss this message'
		},
		noFiles: 'No files yet.',
		remove: 'Remove',
		pages: (n: number) => (n === 1 ? '1 page' : `${n} pages`),
		hoursMinutes: (h: number, min: string) => `${h} h ${min} min`,
		minutes: (min: number) => `${min} min`,
		seconds: (s: number) => `${s} s`,
		evenOut: 'Even out the volume',
		frozenText: 'corrected text, as it is now',
		itemPages: 'an item’s pages, as they are now',
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
		translationLanguageTitle: 'Translate into',
		translationLanguageHint:
			'The text is translated into this language, whatever language or languages it is in. Names of people, places and organisations stay as the text spells them.',
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
		languagesNarrowed: (models: string) => `Only the languages ${models} can read are listed.`,
		languageUnsupported: (language: string, models: string) =>
			`${models} cannot use ${language}: remove it or choose another model.`,
		languageOneOnly: (models: string) =>
			`Only one language is passed to ${models}: with several chosen, the language is detected instead.`,
		languageNotWithTimes: (models: string) =>
			`The language is passed to ${models} only when it neither labels speakers nor times the lines; with either, the language is detected.`,
		languageNeedsTwoLetters: (language: string, models: string) =>
			`${models} can be told only languages with a two-letter code, so ${language} is not passed on: the model detects it.`,
		languageNotDetected: (models: string) =>
			`${models} does not detect the language: choose the languages of the documents.`,
		languageDataMissing: (languages: string) =>
			`Language data not yet on this computer: ${languages}. Download it once in Settings, then start.`,
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
		/** How the model fields list models, and the tiers they are grouped under. */
		tiers: {
			listBy: 'List models by',
			by: { provider: 'Provider', tier: 'Tier' },
			/** Headings in a model list. */
			group: { top: 'Most capable', balanced: 'Balanced', light: 'Lightweight' },
			/** Beside the chosen model. */
			one: { top: 'Most capable', balanced: 'Balanced', light: 'Lightweight' },
			local: 'On this computer',
			/** Under a model whose provider has no key yet. */
			noKey: (provider: string) => `No ${provider} key yet: add one in Settings.`
		},
		/** Under a model that documents its languages: opens the list. */
		modelLanguages: (n: number) => `The ${n} languages it supports`,
		reasoning: {
			title: 'Thinking',
			suggested: (level: string) => `${level} (suggested)`,
			level: (level: string) =>
				({
					none: 'None',
					minimal: 'Minimal',
					low: 'Low',
					medium: 'Medium',
					high: 'High',
					xhigh: 'Extra high',
					max: 'Maximum'
				})[level] ?? level,
			hint: 'More thinking costs more and takes longer.'
		},
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
		/** `markdown` is the Markdown file's name, `file.md` unless a file of the researcher's had
		 * that name. */
		pipelineExported: (file: string, markdown: string) =>
			`Saved ${file}.json, to import into Kalamu, and ${markdown}, for a methods section.`,
		pipelineImport: 'Import a pipeline',
		pipelineImported: (name: string) => `Imported “${name}” with its prompts.`,
		linesTitle: 'Line breaks',
		together: {
			title: 'Pages per call',
			one: 'One page at a time',
			one_hint: 'Recommended. Each page is read on its own.',
			several: 'Several pages together',
			several_hint:
				'The model sees an article whole when it continues on the next page. Fewer calls.',
			whole: 'The whole document',
			whole_hint: 'As many pages as the model can write in one answer.',
			pages: 'Pages per call',
			safety:
				'Kalamu never sends more pages together than the model can write in one answer, and splits the answer back into pages for the review. A page the model leaves out, or that its answer cuts short, is read again on its own.',
			economy: 'In the cheaper batch mode, pages are sent one at a time.',
			alone: 'Images, and PDFs whose double pages are split, are read one page at a time.',
			noPrompt: (models: string) => `${models} reads pages one at a time.`
		},
		grouping: {
			title: 'How the images are read',
			separate: 'Separate documents',
			separate_hint: 'Each image is a document of its own, with its own result files.',
			one: 'Pages of one document',
			one_hint:
				'The images are the pages of one document: read in the order below and written as one result.',
			name: 'Name of the document',
			nameHint: 'The result files are named after it, in the folder of the first page.',
			order: 'Order of the pages',
			orderHint:
				'Sorted by the numbers in the file names, page 2 before page 10. Move a page with its buttons, or drag it.',
			/** In place of `orderHint` once a page was moved by hand. */
			orderHintMoved: 'In the order you chose. Move a page with its buttons, or drag it.',
			sorting: 'Putting the pages in order…',
			position: (n: number) => `Page ${n}`,
			moveUp: (name: string) => `Move ${name} up`,
			moveDown: (name: string) => `Move ${name} down`,
			missing: (from: number, to: number) =>
				from === to
					? `No file numbered ${from} before this one`
					: `No files numbered ${from} to ${to} before this one`,
			duplicate: (n: number) => `Another file is also numbered ${n}`,
			naturalOrder: 'Back to the order of the names',
			formats:
				'A searchable PDF is written only from a PDF. ALTO and PAGE XML, when the model gives positions, place each page in its own image.',
			refused: 'An analysis reads each image as a document of its own.'
		},
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
		modelReplaced: (model: string, next: string) =>
			`${next} replaces ${model}. This set-up keeps ${model} while its provider serves it, so its results stay comparable with earlier ones.`,
		modelRetired: (model: string) =>
			`Its provider no longer serves ${model}: choose another model to start the job.`,
		useModel: (model: string) => `Use ${model}`,
		notDownloaded: 'This offline model has not been downloaded yet.',
		openSettings: 'Download in Settings',
		timestamps: 'Timestamps',
		translate: 'Write it in English',
		translateHint:
			'Whisper writes what is said in English, whatever language is spoken. The transcript is then a translation, which provenance records.',
		vocabulary: {
			title: 'Names and words to spell',
			placeholder: 'Samory Touré\nBobo-Dioulasso\nmadrasa',
			hint: 'People, places and terms as you write them, one a line. Whisper reads them before each part of the recording and spells them so when it hears them. The first sixty or so are used.',
			fromProject: 'Add the names in this project’s dictionary',
			noProjectNames: 'This project’s dictionary has no names yet: they come from its analyses.'
		},
		sampleTitle: 'Try a sample first',
		sample: (n: number) => `Only the first ${n} pages of each document`,
		/** New job's suggestion for a long document: a few pages first, then the rest (#215). */
		trySample: {
			title: 'Try on a few pages first',
			offer: (pages: number, n: number) =>
				`${pages} pages is a lot to read at once. Try the model on ${n} pages first: you see how well it reads them and what it costs, then run the rest from the job’s page without paying for these pages again.`,
			one: (n: number) => `Try on ${n} pages`,
			two: (n: number) => `Try on ${n} pages with two models`,
			chosen: (n: number) =>
				`Kalamu will read ${n} pages spread through each document, not the first ones, which are often a cover or a title page. Once you have looked at them, run the rest from the job’s page.`,
			chosenTwo: (n: number) =>
				`Kalamu will read ${n} pages spread through each document with both models, not the first ones, which are often a cover or a title page. Compare them in the review, then run the rest with the one you prefer from the job’s page.`,
			every: 'Read every page',
			/** The sample in the results step, when its pages are spread through each document. */
			spread: (n: number) => `${n} pages spread through each document`
		},
		outputTitle: 'Results',
		formats: 'Formats',
		subtitlesNeedTimes:
			'Subtitles (SRT, WebVTT) are offered for the translation of a transcript with times, sent here from its job with Use this text in a new job.',
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
				'All pages are sent together to the provider’s batch service, at about half the price. Results can take up to a day; Kalamu can be closed in the meantime and collects them when it opens again.'
		},
		paceNotBatched: (names: string) =>
			`${names} has no batch service in Kalamu and still runs page by page, at its usual price.`,
		paceQuotes: (names: string) =>
			`With ${names}, an analysis that quotes the text is sent at once, at the usual price, so that Kalamu can check each quotation against the text.`,
		paceTimedLines: (names: string) =>
			`With ${names}, a transcript translated line by line is sent at once, at the usual price, so that Kalamu can check that no line is missing.`,
		start: 'Start',
		startShortcut: '↵',
		starting: 'Starting…',
		name: 'Name of the job',
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
		needFolder: 'Choose a folder for the results before starting.',
		/** Said where Start is, which is off, when a text is too long for a model chosen. */
		tooLong: (file: string, model: string, words: string, most: string, others: number) =>
			`${file} is too long for ${model}: about ${words} words, where it reads about ${most} at once${others === 0 ? '' : others === 1 ? ', and so is 1 other text' : `, and so are ${others} other texts`}. Choose another model, or split the text into shorter documents.`,
		estimateVideo:
			'A video’s length is known only once Gemini has read it, so the cost shows in Jobs as it runs.'
	},
	warning: {
		double_page_spread: (pages: string) =>
			`Two pages side by side (page ${pages}): models may read across both. Split them before running.`,
		low_resolution: (dpi: number, pages: string) =>
			`Low resolution, about ${dpi} dpi (page ${pages}): small print and handwriting may be misread. Rescan at 300 dpi or more if you can.`,
		has_text_layer: (n: number) =>
			`This PDF already contains text on ${n} of the first pages. Extraction may not be needed, or the existing text can serve for comparison.`,
		unsupported_image: (detail: string) => detail,
		large_file: 'This file is large; it will be split before sending.',
		tool_missing:
			'Reading this file needs FFmpeg, which was not found next to the app. Reinstall Kalamu, or convert the file to MP3.',
		no_sound_track: 'This file has no audio track, so there is nothing to transcribe.',
		long_recording: (length: string, pieces: number) =>
			`Long recording (${length}): it will be sent in ${pieces} pieces, cut at pauses. Speaker labels may differ from one piece to the next.`,
		quiet_recording: (db: number) =>
			`Very quiet recording (about ${db} dB): quiet speech may be missed. Evening out the volume usually helps.`,
		same_page: (first: string, second: string) => `${first} and ${second} look like the same page`,
		blurred: (name: string) => `${name} looks blurred`,
		/** The check of photographs against each other: retakes of one page, blurred photos. */
		photos: {
			checking: 'Looking for photos taken twice and blurred photos…',
			sharper: (name: string) => `${name} is sharper. Sending both would pay for the page twice.`,
			asSharp: 'Both look about as sharp. Sending both would pay for the page twice.',
			blurredWhy: 'Blurred letters are often misread. Retake it if you can.',
			stays: 'Leaving a photo out changes only this job’s list: the file stays on your computer.',
			keepSharper: 'Keep the sharper one',
			keepOnly: (name: string) => `Keep only ${name}`,
			keepBoth: 'Keep both',
			keepIt: 'Keep it',
			leaveOut: 'Leave it out',
			sharperMark: 'Sharper',
			closeUp: (name: string) => `${name}, close-up`,
			whole: (name: string) => `${name}, whole photo`,
			leftOut: (name: string) => `${name} left out of this job`
		}
	},
	tips: {
		title: 'Tips for better results',
		scanning:
			'One page per image or PDF page, not an open double-page spread; flat and fully in frame; even light without shadows or flash glare; 300–400 dpi; colour or greyscale rather than black and white for faded or handwritten material.',
		recording:
			'Microphone close to the speakers, a quiet room, one recording per interview, and a short spoken introduction with names and date.',
		youtubePublic:
			'Only public videos can be read: private and unlisted ones are refused. Kalamu asks YouTube for each video’s title when you add it; Google then reads the video itself, and nothing is downloaded to this computer.',
		youtubePreview:
			'Reading YouTube links is a preview feature of Gemini, and Google may change its limits. A free key reads up to 8 hours of video a day. Each video is transcribed in one go, which suits videos up to about two hours.',
		results:
			'Try a sample before a whole folder, compare two models on difficult material, check names, numbers and dates against the source, and keep the originals.',
		analysisUnit:
			'One file per document: a PDF of several pages is read as a whole and gets one set of answers. Put each article, letter or report in its own file. Several documents in one file get mixed answers, and one document spread over several files gets several partial ones.',
		analysisInput:
			'Scans and text both work: the model reads a PDF’s pages itself. For faded print or handwriting, extracting and correcting the text first means every answer rests on a reading you have checked.',
		analysisFields:
			'Describe each field as you would to a research assistant, give choices that do not overlap, and ask for a reason. Try it on a few documents first, and compare two models where the answer is a judgement, such as tone.',
		contentsUnit:
			'One file per issue, archival file or volume: it is read page by page, then the pages are combined. Put photographs of one archival file together into a single PDF first.',
		contentsCheck:
			'Try the first pages as a sample, then check the page ranges and the articles that run over several pages: the review shows each entry beside its first page.',
		correctionInput:
			'Kalamu corrects the text already in the file: a PDF’s own text layer, or a text file from a digital library (Gallica, Europeana, Internet Archive…). A scan without a text layer needs Extract text instead.',
		correctionCheck:
			'The review highlights every change against the original text. Check names, numbers and dates, which machine reading gets wrong most often.',
		translationInput:
			'Kalamu translates text that already exists: a text or Word file, a PDF’s own text layer, or the text of an earlier extraction, transcription or correction through Use this text in a new job. Correct a machine-read text first: the translation is only as good as the text it reads.',
		translationCheck:
			'The review shows each page of the text beside its translation. Check names, figures and dates, and the words the model marked as unsure, such as [word?].'
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
		/** In place of `extractJoin` when images are read as the pages of one document. */
		extractJoinDocument: 'Joined into one text for the document, page numbers kept',
		/** Under the reading step, for a model that reports where text sits (Tesseract, Mistral
		 * OCR); the second names it beside models that do not. */
		extractPlace: 'And notes where its text sits on the page',
		extractPlaces: (models: string) => `${models} also notes where its text sits`,
		/** Under the reading step, for models that do not (language models). */
		extractTextOnly:
			'Text only, not where it sits on the page: no searchable PDF, ALTO or PAGE XML',
		extractSearchable: 'Also ALTO XML, PAGE XML and a searchable PDF, each word in its place',
		extractPdfOnly: 'The searchable PDF for PDF files only',
		analysisWhole: 'Read whole, as one piece, however many pages',
		analysisFill: (model: string) => `${model} fills in your fields`,
		analysisTable: 'One row of answers per document, ready for a spreadsheet',
		/** An analysis whose fields are one repeating group (One row per entry). */
		recordsSource: 'A register, a directory or a list: a PDF or images',
		recordsRead: (model: string) =>
			`${model} lists each page’s entries in reading order, in your columns`,
		recordsTable:
			'One row per entry, with its file, page and place on the page, ready for a spreadsheet',
		/** The heading of the records table's column of page numbers. */
		recordsPage: 'Page',
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
		youtubeSend: 'Only the link leaves this computer: nothing is downloaded',
		youtubeRead: (model: string) => `${model} watches the whole video in one go`,
		youtubeJoin: 'A transcript with timestamps',
		thisPc: 'This computer',
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
		correctionCompare: 'Every change shown against the original text, to keep or undo',
		translationSource:
			'A text that already exists: a file, a PDF’s text layer or an earlier result',
		translationRead: (model: string) => `${model} translates each page, leaving nothing out`,
		translationBeside: 'Each page of the text beside its translation, to check and correct'
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
	agreement: {
		title: 'Agreement between models',
		intro:
			'How often the models gave the same answers, field by field, over every document both answered. Their own answers are compared, before your corrections. Free-text fields are not counted.',
		field: 'Field',
		pair: 'Models',
		measure: 'Measured by',
		measures: { choice: 'Same choice', overlap: 'Items in common' },
		agreement: 'Agreement',
		kappa: 'Kappa',
		compared: 'Answers compared',
		pairOf: (a: string, b: string) => `${a} and ${b}`,
		none: 'None yet',
		notApplicable: 'Does not apply',
		noKappa: 'Cannot be computed',
		kappaNote:
			'Kappa sets aside the agreement two models would reach by chance, given how often each gives each answer: 1 means they always agree, 0 no more often than chance, below 0 less often. When both gave every document the same answer, chance alone explains their agreement and kappa cannot be computed.',
		overlapNote:
			'For lists, the items both gave out of all the items either gave, ignoring capitals and accents, averaged over the documents; two empty lists count as agreeing.'
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
		analyseSetup: 'Text from',
		derivedChanged:
			'The text this analysis read has been corrected since. Its answers stay as they were; analyse the text again to include the corrections.',
		derivedAgain: 'Analyse the corrected text',
		derivedGone:
			'The job this text came from was deleted. The text it read is kept, and its answers stay as they are.',
		noTextOutputs:
			'This job wrote no text file to analyse. Add Text (.txt) to its formats and export again.',
		openFolder: 'Show in folder',
		openFile: 'Open in its app',
		rename: 'Rename',
		nameLabel: 'Name of the job',
		progress: (done: number, total: number) => `${done} of ${total} pieces`,
		failed: (n: number) => (n === 1 ? '1 failed' : `${n} failed`),
		truncated: (n: number) => `${n} cut off`,
		truncatedNote:
			'The model’s answer stopped before its end, at its length limit or by the provider’s filter, so the end of these pieces may be missing. The review marks them and says why; check them against the source.',
		files: 'Files',
		/** A job's files: every one, or only those with a failed piece, however far down. */
		filesShown: 'Files shown',
		allFiles: (n: number) => `All files (${n})`,
		failedFiles: (n: number) => `Files with failures (${n})`,
		/** Read out when only the files with failures are listed. */
		failedListed: (n: number, total: number) =>
			`${n === 1 ? '1 file' : `${n} files`} with failures listed, of ${total}`,
		livePreview: 'Latest result',
		outputs: 'Written',
		created: 'Created',
		task: 'Task',
		models: 'Models',
		pieces: 'Pieces',
		cost: 'Spent',
		interruptedNotice:
			'This job stopped before it finished, when the app closed or a key stopped working. Finished pieces are saved: resuming continues where it left off, and nothing is sent or paid for twice.',
		pausedNotice: 'Paused. Resume to continue where it left off.',
		waitingNotice: (n: number, due: string) =>
			`${n === 1 ? '1 piece is' : `${n} pieces are`} with the provider’s batch service, answered by ${due} at the latest. Kalamu can be closed in the meantime: it collects the answers when it opens again.`,
		heldNotice: (n: number) =>
			`${n === 1 ? '1 piece is' : `${n} pieces are`} still with the provider’s batch service. Resume to collect the answers; nothing is sent or paid for twice.`,
		pace: 'Speed',
		paceName: { fast: 'Faster', economy: 'Cheaper (batch)' },
		/** A sample's page: the rest of its documents, or another model on its pages (#215). */
		sample: {
			first: (n: number) => `A sample: the first ${n} pages of each document.`,
			spread: (n: number) => `A sample: ${n} pages spread through each document.`,
			next: 'Once you have looked at them, run the rest, or try another model on the same pages.',
			runRest: (model: string) => `Run the rest with ${model}`,
			left: (n: number) => (n === 1 ? '1 page left' : `${n} pages left`),
			about: (cost: string) => `about ${cost}`,
			free: (n: number) =>
				n === 1
					? 'The page already read costs nothing.'
					: `The ${n} pages already read cost nothing.`,
			/** The name of the job that reads every page, after the sample's. */
			restName: (name: string) => `${name} (every page)`,
			compareLede: (model: string) =>
				`Another model reads the same pages beside ${model}. Only the new model is paid for: the pages already read are not sent again.`,
			compareStart: 'Start',
			compareCost: (cost: string) => `About ${cost} for the new model.`,
			/** The name of the comparison, after the sample's. */
			compareName: (name: string) => `${name} (compared)`
		},
		empty_new: 'Start a job'
	},
	/** The Jobs page: every job in a table, found, filtered and sorted. */
	allJobs: {
		title: 'All jobs',
		count: (n: number) => (n === 1 ? '1 job' : `${n} jobs`),
		/** The sidebar's link under its most recent finished jobs. */
		link: (n: number) => `All jobs (${n})`,
		search: 'Search names and files',
		anyTask: 'All tasks',
		anyStatus: 'All statuses',
		sortBy: 'Sort by',
		sortNewest: 'Newest first',
		sortOldest: 'Oldest first',
		sortName: 'Name',
		columnName: 'Name',
		columnTask: 'Task',
		columnStatus: 'Status',
		columnPieces: 'Pieces',
		columnCost: 'Spent',
		columnDate: 'Created',
		files: (n: number) => (n === 1 ? '1 file' : `${n} files`),
		noMatch: 'No job matches these filters.',
		selectAll: 'Select all jobs shown',
		select: (name: string) => `Select “${name}”`,
		selected: (n: number) => (n === 1 ? '1 selected' : `${n} selected`),
		clearSelection: 'Clear',
		actions: 'Actions',
		addTo: 'Add to a project…',
		addedTo: (n: number, project: string) =>
			`${n === 1 ? '1 job' : `${n} jobs`} added to the project “${project}”.`,
		delete: 'Delete',
		deleteConfirm: (n: number) =>
			`Delete ${n === 1 ? 'this job' : `these ${n} jobs`}? Files already written stay on disk.`,
		runningKept: 'Running jobs cannot be deleted: pause or cancel them first.'
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
		pageHalf: (n: number, half: 'left' | 'right') =>
			`Page ${n}, ${half === 'left' ? 'left' : 'right'} half`,
		pageHalfShort: (n: number, half: 'left' | 'right') => `${n}${half === 'left' ? 'l' : 'r'}`,
		passage: (n: number) => `Passage ${n}`,
		turns: (n: number, first: number, last: number) => `Part ${n} (turns ${first}–${last})`,
		segment: (label: string) => `From ${label}`,
		document: 'Whole document',
		video: 'Whole video',
		openVideo: 'Open on YouTube',
		originalFile: (name: string) => `Original: ${name}`,
		originalHint: 'Open the original image of this page',
		videoPlayer: 'The video, in YouTube’s player',
		videoHint: 'Click a line to play the video from there.',
		videoHintOpen: 'Click a line to open the video there on YouTube.',
		playVideo: 'Play the video',
		playVideoNote: 'Plays from YouTube, in its privacy-enhanced player.',
		videoBlocked:
			'This video cannot be played here: its owner may not allow it, or this computer is offline.',
		previous: 'Previous',
		next: 'Next',
		source: 'Source',
		showOriginal: 'Show the text read',
		showOriginalHint: 'The page’s text as the model was given it, in place of the page image',
		showPage: 'Show the page',
		showPageHint: 'The page image, in place of the text the model was given',
		itemMenu: (name: string) => `What to do with “${name}”`,
		renameItem: 'Rename',
		removeItem: 'Remove from the answers',
		itemKeys:
			'Marked in the source while it has the focus. Enter, or the menu key, renames or removes it.',
		pageImage: 'Page image: the arrow keys move it, with Shift by longer steps',
		itemRemoved: (name: string) => `“${name}” removed from the answers`,
		itemEverywhere:
			'Changed in this document’s corrected answers, for every model that found it. The model’s own answers are kept.',
		/** The button at the end of a list answer that opens a field to add to it. */
		addItem: 'Add',
		addTo: (field: string) => `Add to “${field}”`,
		addName: (field: string) => `Add to “${field}”: one name, or several separated by semicolons`,
		addChoice: (field: string) => `Choice to add to “${field}”`,
		addPlaceholder: 'A name; another name',
		addHint:
			'Enter adds and keeps the field open; Escape closes it. Added for every model that read the document; the models’ own answers are kept.',
		addEmpty: 'Type a name to add.',
		allChosen: 'Every choice is already listed.',
		itemsAdded: (names: string[]) =>
			`${names.map((n) => `“${n}”`).join(', ')} added to the answers`,
		alreadyListed: (names: string[]) =>
			`Already in the list: ${names.map((n) => `“${n}”`).join(', ')}`,
		notAChoice: (names: string[]) =>
			`Not one of this field’s choices: ${names.map((n) => `“${n}”`).join(', ')}`,
		/** Read after a name the researcher added or renamed, which no model listed. */
		yours: (name: string) => `${name}, your correction`,
		yoursHint: 'Added or renamed by you; no model listed it',
		choiceKeys: 'Marked in the source while it has the focus. Enter, or the menu key, removes it.',
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
		truncated:
			'The model’s answer stopped at its length limit here, so the end may be missing. Check it against the source, or correct it.',
		truncatedFilter:
			'The provider’s filter stopped the model’s answer here, so the end may be missing. Check it against the source, or correct it; another model may read it whole.',
		truncatedOther:
			'The provider stopped the model’s answer here before its end, so the end may be missing. Check it against the source, or correct it.',
		noPreview: 'No preview for this file type.',
		notDone: 'Not read yet.',
		saved: 'Saved; the result files follow in a few seconds.',
		pieceReviewed: (label: string) => `${label}, reviewed`,
		zoomOutShortcut: '−',
		zoomInShortcut: '+',
		zoomFitShortcut: '0',
		zoomIn: 'Zoom in',
		zoomOut: 'Zoom out',
		zoomHint: (key: string) => `${key} + wheel: zoom · drag: move · double-click: zoom in`,
		turnLeft: 'Turn the page left',
		turnRight: 'Turn the page right',
		regionsShow: 'Show text regions',
		regionsHint:
			'Outline the blocks, lines and words the model placed on the page; select one to find its text',
		regionsFallback:
			'This model did not place the text on this page: check the text against the whole page.',
		regionsLabel: 'Text regions on the page: the arrow keys go from one to the next',
		regionMarked: 'Marked in the text.',
		regionNotPlaced:
			'Not marked in the text: these words stand there more than once, or not exactly as read.',
		regionNotPlacedCorrected:
			'Not marked in the text: the page was corrected, and these words no longer stand there once, exactly as read.',
		regionEstimated:
			'Approximate: the model placed only the whole block, so its lines and words are estimated.',
		regionClose: 'Close',
		/** The researcher's own regions over the scan: drawn, moved, resized, removed. */
		regionEdit: {
			ownHint: 'Show the regions you drew on the page, and draw new ones',
			/** The switch where a page has only the researcher's own regions. */
			showOwn: 'Show regions',
			draw: 'Draw a region',
			drawHint:
				'Drag over the page to draw a box around a part of it. Press again to place a box in the middle of the page and move it with the arrow keys.',
			drawing:
				'Drag over the page to draw the box, or press “Draw a region” again to place one in the middle. Escape cancels.',
			adjusting: (shift: string) =>
				`Drag the box to move it, or its corners and sides to resize it. With the keyboard: the arrow keys move it, ${shift} and the arrow keys resize it, Enter saves, Escape cancels.`,
			box: 'The region’s box',
			moveResize: 'Move or resize',
			done: 'Done',
			cancel: 'Cancel',
			remove: 'Remove',
			putBack: 'Put back the model’s box',
			adjusted: 'You moved or resized this box. The model’s own box is kept and can be put back.',
			yours: 'Your region',
			ownName: (text: string) => (text ? `Your region: ${text}` : 'Your region'),
			adjustedName: (text: string) => `${text} (your box)`,
			text: 'What is written here (optional)',
			textHint: (keys: string) => `Saved when you leave the field, or with ${keys}.`,
			moveCard: 'Move this card',
			moveCardHint:
				'Drag to move this card out of the way, or use the up and down arrow keys. A double click puts it back.',
			menu: 'What to do with this region',
			copyText: 'Copy its text',
			read: 'Read this region again…',
			saved: 'Region saved.',
			removed: 'Region removed.',
			putBackDone: 'The model’s box is back.',
			noPositions:
				'This model did not place the text on this page: check the text against the whole page, or draw a region yourself.',
			noneYet: 'No region on this page yet. Draw one around a part you want to note.',
			said: {
				right: 'Moved right',
				left: 'Moved left',
				up: 'Moved up',
				down: 'Moved down',
				wider: 'Wider',
				narrower: 'Narrower',
				taller: 'Taller',
				shorter: 'Shorter'
			}
		},
		play: 'Play',
		pausePlayback: 'Pause',
		position: 'Position in this piece',
		noAudio: 'This piece could not be played.',
		lines: 'Transcript lines',
		noTimings: 'This model gave no timings, so lines cannot follow the recording.',
		reviewedOf: (n: number, total: number) => `Reviewed ${n} of ${total}`,
		readingText: 'Reading text',
		correcting: 'Correcting',
		/** In a place's readings, where a text has nothing: no words were read there. */
		noText: '(no text)',
		speakers: 'Speakers',
		speakersHint:
			'Give the model’s labels real names. They replace the labels here and in every file; the model’s own output is kept.',
		speakerPlaceholder: 'Name',
		speakerName: (label: string) => `Name for ${label}`,
		saveNames: 'Save names',
		namesSaved: 'Names saved; the result files follow in a few seconds.',
		speakersPieces:
			'This recording was sent in several pieces: the same label may be a different person in each piece. Names apply to the whole recording.',
		yourCorrections: 'Your corrections',
		answers: 'Answers',
		noneFound: 'None found',
		onePerLine: 'One per line.',
		reviewedShortcut: '↵',
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
		showReadings: 'Show the readings here',
		copyCompared: 'Copy the whole text compared',
		compareMenu: 'What to do in the text compared',
		inUse: 'In the text',
		ownReading: 'Your own reading',
		ownReadingPlaceholder: 'Type what the source says',
		apply: 'Apply',
		readingKept: 'The model’s text is kept; your choice is stored as a correction.',
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
		moveReadings: 'Move this box (drag it or use the arrow keys)',
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
		editEntries: 'Correct the entries',
		search: 'Search',
		searchShortcut: 'F',
		searchText: 'Search this document',
		searchEntries: 'Search the entries',
		found: (n: number) => (n === 1 ? '1 found' : `${n} found`),
		foundAt: (at: number, n: number) => `${at} of ${n}`,
		notFound: 'Not found',
		previousMatch: 'Previous match',
		nextMatch: 'Next match',
		listMatches: 'List',
		listMatchesHint: 'List the matches by page',
		matches: 'Matches',
		moreMatches: (shown: number, n: number) => `The first ${shown} of ${n} are listed.`,
		closeSearch: 'Close the search',
		entriesFound: (n: number, total: number) =>
			`${n} of ${total} ${total === 1 ? 'entry' : 'entries'}`,
		noEntryFound: 'No entry holds these words.',
		searchAnswers: 'Search the answers',
		searchAnswersAndText: 'Search the answers and the document',
		answersFound: (n: number, total: number) =>
			`${n} of ${total} ${total === 1 ? 'answer' : 'answers'}`,
		inDocument: (n: number) => `${n} in the document`,
		noAnswerFound: 'No answer holds these words.',
		rowsTitle: 'Rows on this page',
		rows: (n: number) => (n === 1 ? '1 row' : `${n} rows`),
		rowNumber: 'No.',
		rowActions: 'Row',
		noRows: 'No entry was read on this page.',
		rowsNotRead:
			'This page could not be read, so its entries are missing. Retry it from the job’s page.',
		addRow: 'Add a row',
		insertRow: (n: number) => `Add a row after row ${n}`,
		removeRow: (n: number) => `Remove row ${n}`,
		cell: (column: string, n: number) => `${column}, row ${n}`,
		noChoice: 'None',
		rowsBy: 'Rows read by',
		rowsKept:
			'Each change is saved at once, apart from the model’s rows, which stay as they were. Separate a list’s names with semicolons.'
	},
	/** Correcting a passage selected in the text review with a model. */
	passage: {
		menu: 'What to do with the selected text',
		copy: 'Copy',
		correct: 'Correct this passage with a model…',
		button: 'Correct with a model…',
		buttonHint:
			'Ask a model to correct the selected text. Nothing changes until you accept its proposal.',
		title: 'Correct a passage with a model',
		passage: 'Selected passage',
		context: 'Context sent with it, not to be changed',
		chars: (n: number) => (n === 1 ? '1 character' : `${n} characters`),
		sentCloud: (provider: string, context: number) =>
			`Sent to ${provider}: the selected passage, and up to ${context} characters before and after it from the same page or part, marked as context the model must not change. Nothing else from the document is sent.`,
		sentOffline: (model: string) => `Read on this computer by ${model}; nothing is sent.`,
		cut: 'The selection went beyond the text of this page or part: only that text is taken.',
		tooShort: (n: number) => `Select at least ${n} letters or figures.`,
		tooLong: (n: number, max: number) =>
			`The selection is ${n} characters long. At most ${max} are sent at once, so that the corrected passage comes back whole in one answer; correct a longer stretch with a correction job.`,
		costAbout: (usd: string) => `About ${usd}`,
		costStored: 'Asked before: the proposal kept is shown again, at no cost.',
		costOffline: 'On this computer, at no cost.',
		costUnknown: 'This model has no published price.',
		noKey: (provider: string) => `Add a key for ${provider} in Settings first.`,
		notInstalled: (model: string) => `Download ${model} in Settings first.`,
		ask: 'Ask for a proposal',
		waiting: (model: string) => `Waiting for ${model}…`,
		stop: 'Stop waiting',
		stopped:
			'Stopped waiting. If the answer still comes, it is kept, and asking again costs nothing.',
		ready: 'Proposal ready.',
		proposal: 'Proposed correction, with the changes marked',
		removed: 'removed',
		added: 'added',
		notes: {
			context_removed: 'The model gave back some of the context; it was taken out.',
			commentary_removed:
				'The model added words of its own around the passage; they were taken out.',
			length_changed: 'The proposal is much longer or shorter than the passage: check it closely.',
			unchanged: 'The model found nothing to correct.'
		},
		accept: 'Accept',
		edit: 'Edit before accepting',
		editLabel: 'Proposed correction, to edit',
		discard: 'Discard',
		failed: (reason: string) =>
			`No proposal (${reason}). An answer that is not a correction of the passage is not offered: try again, or another model or prompt.`,
		changed:
			'The text under the selection changed while the model was answering, so the proposal was not applied.',
		lookAgain: 'Look again',
		accepted: 'Passage corrected; the result files follow in a few seconds.',
		undone: 'Correction of the passage undone.'
	},
	/** Reading one region of a page again with a model, from the review. */
	regionReading: {
		title: 'Read this region again',
		picture: 'The part of the page that will be read',
		noPicture: 'The part of the page inside the box will be read.',
		readBy: (model: string) => `Read by ${model}`,
		change: 'Choose another model',
		defaultPrompt: 'Default',
		sentCloud: (provider: string) =>
			`Only this part of the page is sent to ${provider}, as a picture. Nothing else from the document is sent.`,
		sentOffline: 'Read on this computer; nothing is sent.',
		costAbout: (usd: string) => `About ${usd}.`,
		costOffline: 'Free, on this computer.',
		costStored: 'Already read: no cost.',
		costUnknown: 'This model has no published price.',
		noKey: (provider: string) => `Add a key for ${provider} in Settings first.`,
		notInstalled: (model: string) => `Download ${model} in Settings first.`,
		noModel:
			'None of your models reads pictures of pages. Add a key in Settings, or download a model that runs on this computer.',
		read: 'Read',
		waiting: (model: string) => `${model} is reading this part of the page…`,
		stop: 'Stop',
		stopped: 'Stopped. If the answer still comes, it is kept, and reading again costs nothing.',
		ready: 'Reading ready.',
		answer: 'What the model read',
		replaced: 'The words now in the page’s text, and the new ones',
		removed: 'removed',
		added: 'added',
		useRegion: 'Use as this region’s text',
		useText: 'Replace these words in the page’s text',
		edit: 'Edit first',
		editLabel: 'What the model read, to edit',
		copy: 'Copy',
		copied: 'Copied.',
		discard: 'Discard',
		close: 'Close',
		nowhere:
			'Kalamu cannot tell exactly where this region’s words are in the page’s text, so this reading can be copied but not put into the text.',
		failed: (reason: string) => `Nothing to offer (${reason}). Try again, or choose another model.`,
		changed:
			'The words in the page’s text changed while the model was reading, so nothing was replaced.',
		lookAgain: 'Look again',
		acceptedRegion: 'The region’s text is saved.',
		acceptedText: 'Words replaced; the result files follow in a few seconds.',
		undone: 'Reading of the region undone.'
	},
	prompts: {
		title: 'Prompts',
		hint: 'Each task comes with tested prompts. Edit one to adapt it; your version replaces it until you reset it. Variables: {languages}, {speakers}.',
		builtin: 'Default',
		modified: 'Edited',
		own: 'Yours',
		name: 'Name',
		description: 'Description',
		instructions: 'Instructions (system prompt)',
		userPrompt: 'Request sent with each page',
		userPromptDocument: 'Request sent with each document',
		question: 'What is your research about?',
		questionHint:
			'Optional. Sent with each document, so you write it once for all your interviews. A description typed in Analyse this interview replaces it.',
		themes: 'Your own themes or interview guide',
		themesHint:
			'Optional. One theme per line, sent with each document. Themes typed in Analyse this interview replace them.',
		mergeInstructions: 'Instructions for combining the pages',
		dense: 'Pages full of text',
		denseHint:
			'For newspapers and other pages of close columns. When a job reads several pages in one call, fewer go together, so the answer has room for all their text.',
		save: 'Save',
		duplicate: 'Duplicate',
		copyName: (name: string) => `${name} (copy)`,
		newPrompt: 'New prompt',
		newName: 'My prompt',
		task: 'Used for',
		taskHint: 'A prompt belongs to one task: New job offers it when that task is chosen.',
		newHint:
			'Write your instructions. The request, and for an analysis or a table of contents its fields, start from this task’s default prompt; change them as you need.',
		instructionsPlaceholder:
			'For example: You read printed newspaper pages from French West Africa, 1940 to 1960. Transcribe the text exactly as printed, column by column, keeping the original spelling.',
		discard: 'Discard',
		reset: 'Reset to default',
		delete: 'Delete',
		export: 'Export…',
		import: 'Import…',
		imported: (n: number) => `${n} imported`,
		saved: 'Saved',
		unsaved: 'Unsaved changes',
		analysisHint:
			'An analysis is a set of fields to fill in for each document. Create a new prompt, or duplicate one of these to start from it.',
		contentsHint:
			'A table of contents has two steps: instructions for reading each page, and instructions for combining the pages. Every entry has a title and pages; the fields below add what you want for each.',
		dictionary: 'Dictionary',
		dictionaryHint:
			'Sent when you ask a model to propose merges in a project’s dictionary. Edit the instructions to change what it groups; reset restores the default.',
		dictionaryFields:
			'The answer’s fields are fixed (the rows of each group, a label, a reason and a certainty), because the dictionary reads them. The instructions say what to group.'
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
			choices: 'Several choices',
			cited: 'Passages quoted from the text'
		},
		/** Under a field of quoted passages, which comes with a ready-made prompt. */
		citedNote:
			'Kalamu checks each quotation word for word against the text. What each entry holds comes with the ready-made prompt and cannot be changed here.',
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
		rowsToggle: 'One row per entry',
		rowsHint:
			'For a register, a directory or a list of names: each page is read on its own, and every entry on it becomes a row whose columns are the fields below. Turned off, the analysis answers once for each document.',
		rowsLabel: 'Entries',
		rowsInstructions: 'Every entry on this page, one row each, in reading order.',
		rowsWhich: 'Which entries to read',
		columns: 'Columns',
		columnsHint:
			'What to read in each entry. Each column becomes a column in the spreadsheet export, with the page and the entry’s place on it.',
		firstColumn: 'Name',
		addColumn: 'Add a column',
		removeColumn: (label: string) => `Remove ${label || 'this column'}`,
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
		lede: 'Keys, offline models, processing, appearance and language.',
		tabsLabel: 'Settings sections',
		tabs: {
			keys: 'API keys',
			offline: 'Offline models',
			general: 'General',
			about: 'About'
		},
		/** The welcome at the top of Settings, only while no key is saved and nothing is
		 * installed offline: two ways to start, each opening its tab. */
		welcomeTitle: 'Welcome to Kalamu',
		welcomeLede:
			'To read your documents and recordings, Kalamu needs a model. Choose one of two ways to start; you can add the other later.',
		welcomeOffline: 'Use a model on this computer',
		welcomeOfflineHint:
			'Free, with no account. Nothing leaves your computer. The model is downloaded once; it is slower than an online one.',
		welcomeKey: 'Add a key from an online provider',
		welcomeKeyHint:
			'Faster and often more accurate. You open an account with the provider, which charges for what you use, and your files are sent to it.',
		welcomeGuide: 'New to these tools? The guide explains each step.',
		/** Settings › About, before the guide's link. */
		aboutGuide: 'Step by step, from installing Kalamu to exporting checked results.',
		keysTitle: 'API keys',
		keysHint: (keychain: string) =>
			`Keys are stored in ${keychain} and sent only to their provider. Files go directly from this computer to the provider you choose.`,
		keySaved: 'Saved',
		keyMissing: 'No key',
		keyPlaceholder: 'Paste the key',
		save: 'Save',
		test: 'Test',
		remove: 'Remove',
		getKey: 'Get a key',
		testOk: (n: number) => `Key works: ${n} models available`,
		offlineTitle: 'Working offline',
		whisperTitle: 'Transcription with Whisper',
		tesseractTitle: 'Text extraction with Tesseract',
		tesseractHint: (version: string) =>
			`Tesseract ${version} reads printed pages on this computer: nothing is sent, no key or account is needed, and it works without an internet connection once a language is downloaded. It is meant for clean print; handwriting, faded or damaged pages and complex layouts read better with a cloud model. Its speed depends on this computer’s processor: from about a second to several seconds a page. It does not detect the language: download each language you read, then choose it in New job.`,
		latinScript: (languages: string) =>
			`Languages written in Latin letters without a model of their own: ${languages}`,
		noLanguageData: 'No language downloaded yet.',
		ocrTitle: 'Text extraction with OCR models',
		ocrHint:
			'Small vision models trained to read pages, run on this computer: nothing is sent, no key or account is needed, and they work without an internet connection once downloaded. They read degraded print, old newspapers in columns, tables and mixed layouts far better than Tesseract, but are much slower: from tens of seconds to over a minute a page, depending on this computer’s processor, its memory and how much text the page holds. On a Mac, and on Windows when Kalamu finds a graphics processor it can use, they run on the graphics processor and are several times faster. Like any language model they can write text that is not on the page, so compare doubtful pages with Tesseract or a cloud model. For clean print Tesseract is quicker; for large volumes a cloud model is.',
		ocrSpeed: (seconds: number, graphics: boolean) =>
			`On this computer${graphics ? ', with its graphics processor' : ''}: about ${seconds} s a page, from the pages it read last.`,
		ocrGraphics: (name: string) =>
			`Run Whisper, the OCR and the language models on the graphics processor (${name})`,
		ocrGraphicsHint:
			'Usually faster than the processor, often several times. Only the models known to read correctly on this graphics processor use it; the others read on the processor. If a page fails there, Kalamu reads it again on the processor and keeps to the processor until it is restarted. The offline language models below follow this choice too, putting as much of the model there as its memory holds, and so does Whisper on Windows: a recording that fails there is transcribed again on the processor. Turn this off to leave the graphics processor free for other work.',
		ocrNotTimed: 'Not yet timed on this computer: the first pages read will tell.',
		ocrNoEngine:
			'This copy of Kalamu was built without the engine that runs OCR models: the installers and the Store’s version carry it.',
		ocrCpu: (missing: string) =>
			`This processor lacks ${missing}, which offline OCR models need. Use Tesseract or a cloud model instead.`,
		llmTitle: 'Analyses with language models',
		llmHint:
			'Small language models from Google, run on this computer, for analyses, tables of contents and corrections: nothing is sent, no key or account is needed, and they work without an internet connection once downloaded. They are far weaker than the cloud models: their answers are plainer, they can miss or invent names and facts, and they follow long instructions less well, so check what they write. They read texts of up to about 11,000 words, and photographed pages; to analyse a PDF, extract its text first. They are slow on a processor, up to several minutes for a long text, and several times faster on a graphics processor: a Mac’s, or on Windows one Kalamu can use; one with little memory of its own takes part of the model. Each needs several gigabytes of free memory while it runs.',
		llmNoEngine:
			'This copy of Kalamu was built without the engine that runs offline language models: the installers and the Store’s version carry it.',
		llmCpu: (missing: string) =>
			`This processor lacks ${missing}, which offline language models need. Use a cloud model instead.`,
		embeddingTitle: 'Search by meaning',
		embeddingHint:
			'A small model from Google, run on this computer, that lets a project’s search find passages by what they mean, not only by their words, and draws the map of its documents: nothing is sent, no key or account is needed, and it works without an internet connection once downloaded. It reads more than a hundred languages. It is quick, even on a processor, and needs less than a gigabyte of free memory while it runs.',
		embeddingNoEngine:
			'This copy of Kalamu was built without the engine that runs the offline search by meaning: the installers and the Store’s version carry it.',
		embeddingCpu: (missing: string) =>
			`This processor lacks ${missing}, which the offline search by meaning needs. Choose a cloud model for it instead.`,
		embeddingName: 'EmbeddingGemma 2',
		embeddingDescription:
			'Google’s model that places texts of like meaning close together, for texts only, in more than a hundred languages.',
		addLanguageData: 'Download a language for Tesseract',
		/** Beside a language New job asked for whose data is not downloaded. */
		askedByNewJob: 'New job needs this language. Download it here.',
		findLanguageData: 'Find a language by its name',
		oneDownload: 'One download at a time: the next can start once this one is done.',
		offlineHint:
			'Whisper runs on this computer: recordings never leave it, no key or account is needed, and it works without an internet connection once a model is downloaded. It is slower than the cloud models and cannot label speakers. Large v3 Turbo suits most computers.',
		offlineCpu: (missing: string) =>
			`This processor lacks ${missing}, which offline transcription needs. Use a cloud model instead.`,
		modelInstalled: 'Downloaded',
		modelMissing: 'Not downloaded',
		modelDownloading: (pct: number) => `Downloading… ${pct}%`,
		modelStopped: (pct: number) => `Stopped at ${pct}%`,
		modelKept: (size: string) =>
			`The ${size} already downloaded are kept: the download will continue from there.`,
		continueDownload: 'Continue download',
		discardDownload: 'Discard',
		modelInUse: 'In use by a job',
		download: 'Download',
		cancelDownload: 'Cancel',
		removeModel: 'Remove',
		size: (bytes: number) =>
			bytes >= 1e9 ? `${(bytes / 1e9).toFixed(1)} GB` : `${Math.round(bytes / 1e6)} MB`,
		jobsTitle: 'Processing',
		concurrency: 'Pages sent at the same time',
		concurrencyHint:
			'Takes effect the next time Kalamu opens. Lower it if a provider asks Kalamu to slow down.',
		catalogTitle: 'Models',
		catalogUpdated: (date: string) => `Model list from ${date}`,
		catalogRefresh: 'Check for an updated model list',
		catalogAuto: 'Check at startup',
		catalogRefused:
			'The latest model list from Kalamu’s repository failed Kalamu’s checks, so it is not used. Kalamu keeps the list above.',
		catalogNote:
			'Prices are in US dollars, from each provider’s published list on the date above. They change, so Kalamu shows estimates only. A model marked “preview” may change or be withdrawn.',
		appearanceTitle: 'Appearance and language',
		theme: 'Theme',
		themeSystem: 'Same as the system',
		themeLight: 'Light',
		themeDark: 'Dark',
		language: 'Language of the interface',
		languageSystem: 'Same as the system',
		languageHint:
			'Prompts, the models’ answers and the files Kalamu writes keep their own language: choose the language of the answers in New job.',
		aboutTitle: 'About',
		version: (v: string) => `Version ${v}`,
		dataFolder: 'Data folder',
		/** `{name}` marks a link; the names it shows are in `creditNames`. */
		credits:
			'Kalamu uses {ffmpeg} under the LGPL v2.1 (its {source} is published with each release), {opus} (BSD), {whisper} (MIT), the {models} (MIT), {tesseract} and its language data (Apache 2.0), {leptonica} (BSD), {hayro} (MIT), {llama} (MIT), {lighton} (Apache 2.0), {glm} (MIT) and {gemma} (Apache 2.0). The full licences are in the licences folder installed with the app.',
		creditNames: {
			ffmpeg: 'FFmpeg',
			source: 'source',
			opus: 'Opus',
			whisper: 'whisper.cpp',
			models: 'Whisper models',
			tesseract: 'Tesseract',
			leptonica: 'Leptonica',
			hayro: 'hayro',
			llama: 'llama.cpp',
			lighton: 'LightOnOCR-3',
			glm: 'GLM-OCR',
			gemma: 'Gemma 4'
		},
		privacy:
			'Nothing is sent to the developer: no files, keys, results or usage data. AI output can contain fluent but wrong text; check a sample against the source.',
		/** A release newer than this copy, named by the model list (never in a store's copy). */
		newerRelease: (version: string) => `Kalamu ${version} is available.`,
		releasePage: 'What’s new, and the download',
		hideRelease: 'Hide until the next version',
		storageTitle: 'Stored answers',
		storageHint:
			'Kalamu keeps every answer a model gave, so a page asked for again with the same settings costs nothing. Clearing removes the answers kept from jobs you deleted, to free space on the disk; asking for one of those pages again would be paid for again.',
		clearUnused: 'Clear unused results',
		clearUnusedBusy: 'Available once no job is running.',
		clearUnusedOk: 'Clear',
		clearUnusedDone: (count: number, answers: string, size: string) =>
			count === 0
				? 'No unused answers to remove.'
				: `${answers} ${count === 1 ? 'answer' : 'answers'} removed, ${size} freed.`,
		/** At start-up, when the settings file could not be read and was moved aside. */
		setAside: (path: string) =>
			`Your settings could not be read, so Kalamu started with its defaults. The old file is kept as ${path}.`,
		/** Clear unused results, when counting found none: said without asking. */
		clearUnusedNothing: 'Nothing to remove: every stored answer belongs to a job you kept.',
		/** Clear unused results, asked once counted. */
		clearUnusedAsk: (count: number, answers: string, bytes: number) =>
			`Remove ${answers} ${count === 1 ? 'answer' : 'answers'} kept from deleted jobs (${
				bytes < 1e6
					? 'less than 1 MB'
					: `about ${bytes >= 1e9 ? `${(bytes / 1e9).toFixed(1)} GB` : `${Math.round(bytes / 1e6)} MB`}`
			})? Asking for the same pages again would be paid for again.`
	},
	/** The copies of the job database (Settings, General) and what a launch did to it. */
	database: {
		title: 'Safety copies',
		hint: 'Kalamu keeps copies of its job database, which holds your jobs, your corrections, your projects and the answers you paid for: one each day you open Kalamu, the last three kept, and one before an update changes it. They stay on this computer, in this folder:',
		show: { windows: 'Show in File Explorer', macos: 'Show in Finder' },
		none: 'No copy yet: the first is made a moment after Kalamu opens.',
		listLabel: 'Copies kept',
		kinds: {
			daily: 'Daily copy',
			before_update: 'Before an update',
			set_aside: 'Set aside'
		},
		/** A copy's line: its date, what it is, its size. */
		line: (date: string, kind: string, size: string) => `${date} · ${kind} · ${size}`,
		setAsideHint:
			'A file set aside is your job database as it was when a copy took its place, or when it could not be read. Kalamu never deletes it: you can delete it yourself once you no longer need it.',
		open: 'Open this copy…',
		openBusy: 'Available once no job is running.',
		openAsk: (date: string) =>
			`Open the copy of ${date} in place of your job database? Kalamu will restart. Your database as it is now is kept in the copies folder, so nothing is deleted, but what you did after ${date} will not show.`,
		openOk: 'Open and restart',
		skipped: (free: string) =>
			`No copy was made this time: the disk would have been left with too little free space (${free} free). Free some space so that Kalamu can make copies again.`,
		damaged:
			'Kalamu found damage in its job database a moment after it opened. Open the newest copy below to go back to it.',
		damagedAsk: (date: string) =>
			`Kalamu found damage in its job database. Open the copy of ${date} in its place? Kalamu will restart. The damaged file is kept in the copies folder, so nothing is deleted, but what you did after ${date} will not show.`,
		damagedNoCopy:
			'Kalamu found damage in its job database, and there is no copy of it yet. Export what matters to you now; Kalamu’s logs say more.',
		later: 'Not now',
		restored: (date: string) =>
			`The copy of ${date} is open in place of your job database. The database it replaced is kept in the copies folder.`,
		restoreFailed:
			'The copy you chose could not be opened, so Kalamu opened your job database as it was. Kalamu’s logs say why.',
		startedEmpty:
			'Your job database could not be read, so Kalamu started with an empty list. The damaged file is kept in the copies folder.',
		/** At start-up: jobs whose record this version cannot read, left out of the list. */
		unreadable: (names: string, count: number) =>
			count === 1
				? `One job could not be read by this version of Kalamu, so it is not listed: ${names}. Its record is kept as it is; a newer version of Kalamu may read it.`
				: `${count} jobs could not be read by this version of Kalamu, so they are not listed: ${names}. Their records are kept as they are; a newer version of Kalamu may read them.`
	},
	report: {
		/** Bugs and feedback; also the way Store policy 11.16 asks for to flag harmful output,
		 * which the note names so a reviewer finds it. */
		link: 'Report a problem',
		review: 'Report a problem',
		note: 'A bug, an idea, or offensive output from a model: tell us on GitHub. It opens in your browser; Kalamu sends nothing, so share only what is safe to make public.'
	},
	/** A failure nobody expected: a screen that broke as it was drawn, or an error no code
	 * caught. Both are written to the log. */
	unexpected: {
		screenTitle: 'This screen stopped working',
		screenBody:
			'Something went wrong in Kalamu while showing it. Everything you saved is kept, and running jobs carry on. Go back, or choose another screen in the sidebar.',
		back: 'Go back',
		reload: 'Reload the window',
		/** In the failure toast, once for a burst of them. */
		message:
			'Something went wrong in Kalamu. Everything you saved is kept. If the screen looks wrong, open it again from the sidebar.',
		/** The start-up failure, tried again. */
		retry: 'Try again'
	},
	glossary: {
		title: 'Glossary',
		lede: 'The words Kalamu uses, for colleagues new to these tools, and what each export format suits.',
		/** Above the filter, before the guide's link. */
		guideLead: 'To learn Kalamu step by step, from the first job to the export:',
		wordsTitle: 'Words',
		formatsTitle: 'Export formats',
		/** The shortcuts, by where they work, written for the system in use. */
		keysTitle: 'Keyboard',
		keysLede:
			'The shortcuts, by where they work. A shortcut of the screen behind waits while a dialog is open.',
		/** Between two keys that do the same. */
		keysOr: 'or',
		keyPlaces: {
			everywhere: 'Everywhere',
			newJob: 'New job',
			review: 'In a review',
			search: 'In a review’s search',
			scan: 'On a scan'
		},
		keys: {
			newJob: 'Open New job',
			sidebar: 'Fold or unfold the sidebar',
			find: 'Go to the filter or search field: on Jobs, a project’s documents, its search and dictionary, and here',
			back: (pageBack: string) =>
				`Go back to the screen you came from, as the mouse’s back button does. In a review ${pageBack} turns to the previous page instead.`,
			escape: 'Close a menu or a dialog, or put away a failure message that has the focus',
			undo: 'Undo what the message at the bottom of the window reports, while it offers Undo (outside a text box)',
			start: 'Start the job',
			addLinks: 'Add the YouTube links pasted',
			newLine: 'Start another line among the links',
			reviewed:
				'Mark the page reviewed, or not reviewed; while correcting, save the correction, which marks the page reviewed, as Save correction does',
			nextPage: 'Next page or part; Page Down scrolls a pane instead when it has the focus',
			previousPage: 'Previous page or part; Page Up scrolls a pane instead when it has the focus',
			differenceSteps: 'On a difference in the text: the previous or next one',
			differenceEnds: 'On a difference in the text: the first or last one',
			openReadings: 'On a difference in the text: open its readings, the focus in them',
			nameSteps:
				'On a name in an analysis: the previous or next one (Home and End: the first or last)',
			nameMenu: 'On a name in an analysis: the menu to rename or remove it',
			passageMenu: 'On text selected in the text: the menu to copy it or have a model correct it',
			nextDifference: 'Next place where the models differ, its readings open',
			previousDifference: 'Previous place where the models differ',
			closeReadings: 'Close the readings, from their own field too',
			moveReadings: 'Move the readings aside, from the grip at their left; with Shift, further',
			splitter: 'Move the boundary between two panes, once it has the focus; with Shift, further',
			search: 'Search the text, or go back to the search field',
			nextMatch: 'Next match',
			previousMatch: 'Previous match',
			closeSearch: 'Close the list of matches, then the search',
			zoomIn: 'Zoom in',
			zoomOut: 'Zoom out',
			fit: 'Fit the page to the pane’s width',
			pan: 'On the page image: move the page; with Shift, by most of the pane'
		},
		filter: 'Find a word',
		noMatch: 'No word by that name.',
		seeAlso: 'See also',
		/** The help link beside a word in the app. */
		whatIs: (word: string) => `What is meant by “${word}”`,
		/** At the foot of a word's definition, opened from its help link. */
		openGlossary: 'Open the glossary',
		openPrompts: 'See and edit the prompts',
		/** By theme: what a model is, then how it is used, then how its results are checked and
		 * recorded. The page lists them alphabetically by word. */
		terms: {
			ocr: {
				word: 'OCR and HTR',
				text: 'Optical character recognition (OCR) turns printed text in an image into text you can search and edit; handwritten text recognition (HTR) does the same for handwriting. Kalamu does both with AI models that read the whole page at once, rather than letter by letter as older OCR engines did. They cope better with columns, faded print and difficult hands, but can also go wrong in new ways (see hallucination). For clean print it also offers Tesseract, an OCR engine of the older kind, on your own computer.'
			},
			llm: {
				word: 'LLM (large language model)',
				text: 'A model trained on very large amounts of text to write the most likely continuation of what it is given. Gemini, GPT, Claude and Mistral’s models are LLMs that also read images and sound. What they write is fluent and often right, but never guaranteed: it is a likely text, not a copy.'
			},
			whisper: {
				word: 'Offline transcription (Whisper)',
				text: 'Whisper is an open speech-recognition model that Kalamu runs on your own computer, after a one-time download in Settings. Nothing is sent anywhere and nothing is billed. It is slower than the cloud models and cannot tell speakers apart.'
			},
			tesseract: {
				word: 'Offline text extraction (Tesseract)',
				text: 'Tesseract is an open OCR engine that Kalamu runs on your own computer. Download each language you read once in Settings; after that nothing is sent anywhere and nothing is billed. It reads clean print well and notes where every word sits on the page, which gives ALTO and PAGE XML files and searchable PDFs. Handwriting, faded or damaged pages and complex layouts read better with an AI model. It does not detect the language, so you choose it, and its speed depends on your computer’s processor.'
			},
			apiKey: {
				word: 'API key',
				text: 'A code from an AI provider (Google, OpenAI, Anthropic, Mistral) that lets Kalamu send work under your account; the provider bills what is used. Kalamu keeps keys in the system’s keychain (Credential Manager on Windows, the Keychain on a Mac) and sends each only to the provider that issued it.'
			},
			prompt: {
				word: 'Prompt',
				text: 'The instructions sent to the model with each page, recording or text: what to transcribe, how to lay it out, which language, what to leave out. Kalamu comes with prompts written for common kinds of material. Prompts (in the menu on the left) lists them by task: read what is sent, adapt them to your documents, or write your own with New prompt. New job offers those of the task chosen. Dedicated OCR and speech models, such as Mistral OCR, Voxtral and Whisper, take no prompt.'
			},
			setup: {
				word: 'Set-up and comparison',
				text: 'A set-up is one provider, model and prompt run over a job’s files. A job with two or more set-ups compares them: in the review each is named by a letter (its siglum), and the places where they read differently are highlighted, as an edition notes the readings of its witnesses.'
			},
			pipeline: {
				word: 'Pipeline',
				text: 'A job’s settings saved under a name: task, models, prompts, languages, formats and destination, so the next job starts in one click. Exported, a pipeline carries its full prompts and a Markdown record to cite in a methods section or share with colleagues. Several pipelines run one after another make a workflow.'
			},
			workflow: {
				word: 'Workflow',
				text: 'Several jobs run one after another on the same files, each called a step: read the text of the scans, check it, then find the people and places in it. A step works on the files or on the text a step before it produced, as you corrected it. Where you asked to check, Kalamu waits until you press Continue, on the job’s page or in its review, so the next step reads the text you approved. Choose “Then” at the bottom of New job for the simplest one, or build longer ones in Workflows.'
			},
			tokens: {
				word: 'Tokens',
				text: 'The units in which models count text and providers bill it. A token is a piece of a word, about four characters of English on average; other languages and scripts often take more. Images and sound are counted in tokens too. The estimated cost before a job, and the cost after it, come from the provider’s price per million tokens read and written.'
			},
			reasoning: {
				word: 'Thinking (reasoning effort)',
				text: 'How much some models work through a question before answering. The intermediate steps are billed as tokens written but not shown. More effort can help with hard judgements, such as an analysis or a difficult hand, but costs more and takes longer. Each model offers its own levels; New job suggests the lowest for reading pages and recordings, and you can choose another for each model under Thinking. The level used is recorded in the provenance.'
			},
			tier: {
				word: 'Tier',
				text: 'Where a model stands among its provider’s own: its most capable (slower and dearer, for the hardest hands and documents), the middle of its range, or its lightweight one (fast and cheap, for clean print). A tier compares a provider’s models with each other, not with another provider’s: a balanced model of one may read better than the most capable of another. Models on this computer are classed among their own family. New job lists models under their tiers, by provider or all together; the price and a trial on a few pages tell the rest.'
			},
			economy: {
				word: 'Cheaper mode (batches)',
				text: 'Sends a job to the provider’s batch service, which answers within a day at about half the price. Kalamu collects the answers when it next opens; the app does not need to stay open meanwhile. An analysis that quotes the text, and a transcript translated line by line, are still sent at once, at the usual price, so that Kalamu can check each answer against the text.'
			},
			groundTruth: {
				word: 'Ground truth',
				text: 'A text known to be correct, against which a model’s output is measured. In Kalamu, every page you correct, or mark as reviewed without changing it, becomes the ground truth for that page.'
			},
			errorRates: {
				word: 'Character and word error rates (CER, WER)',
				text: 'The share of characters, or of words, that a model got wrong compared with the ground truth: those added, left out or replaced, divided by the length of the correct text. A character error rate of 2% is about one character wrong in fifty. Kalamu counts them only on the pages you reviewed, so they show the errors you caught, not the ones you missed.'
			},
			agreement: {
				word: 'Agreement between models (kappa)',
				text: 'How often two models given the same documents gave the same answers to an analysis. For a choice from a list, Kalamu shows the share of documents where both chose the same value, and Cohen’s kappa, which sets aside the agreement expected by chance: two models that nearly always answer “Neutral” agree often without that saying much. A kappa of 1 is full agreement, 0 no better than chance. For a list of names or themes, it shows how many of the names either found both found. A job’s page shows it when the job compares models on an analysis; the answers compared are the models’ own, before your corrections.'
			},
			hallucination: {
				word: 'Hallucination',
				text: 'Text a model writes that is not in the source: a plausible word where the page is illegible, a sentence nobody spoke, a name an analysis supplies from elsewhere. Fluent output is no evidence of accuracy. Checking a sample against the source, as the review allows, is the safeguard.'
			},
			provenance: {
				word: 'Provenance',
				text: 'The record of how a result was made. Beside every output Kalamu writes a provenance.json file with the model and its exact version, the prompt, the settings each request carried, dates, costs, which pages were corrected or reviewed and, for a project’s document, its title and the description of its source, so the method can be described and repeated.'
			},
			machineReadable: {
				word: 'Machine-readable',
				text: 'In a form other software can process without a person reading it. JSON and CSV are machine-readable; a Word document much less so, and a scan not at all. Choose them when results will be counted, searched or imported into a database.'
			}
		},
		formats: {
			txt: 'Plain text, readable anywhere and the simplest to search or put into other tools. A marker shows where each page starts.',
			markdown:
				'Text with light marks for headings and lists, readable as it is and well suited to note-taking tools such as Obsidian.',
			docx: 'A Word document with a heading for each page, to read, annotate or share with colleagues who work in Word.',
			json: 'Every page, segment, answer or entry with its details, machine-readable, for scripts and databases.',
			xlsx: 'A spreadsheet with one row per page, segment, document or entry (an analysis has a column per field) that opens with a double click in Excel, LibreOffice or Numbers, whatever their language. Every cell is text, exactly as written. Choose it to read, sort or filter results in a spreadsheet.',
			csv: 'The same table as plain text with commas, for R, Python, statistics software or a database import. Excel set to French or German may show it all in one column: open the .xlsx instead.',
			srt: 'Subtitles with their times, for video players and editing software.',
			vtt: 'Subtitles for the web (WebVTT), as web video players use them.',
			searchable_pdf:
				'The original PDF with an invisible text layer, so it can be searched and its text copied. Only from models that report where text sits on the page: currently Tesseract and Mistral OCR.',
			alto: 'Each page’s text with where every block, line and word sits, in the ALTO standard that digital libraries, eScriptorium and Transkribus read. Only from models that report where text sits on the page: Tesseract places each word; Mistral OCR places paragraphs, whose lines and words are then estimated. Corrections keep their words’ places.',
			page_xml:
				'Each page’s text in a file of its own (name.p0001.page.xml), with where every region, line and word sits, in the PAGE format that Transkribus, eScriptorium and Aletheia read. Only from models that report where text sits on the page: Tesseract places each word; Mistral OCR places blocks, whose lines are then estimated and marked as such. Corrections keep their words’ places.',
			provenance: 'Always written beside the results: how they were made (see Provenance).'
		}
	},
	support: {
		title: 'Support development',
		/** Says plainly that paying changes nothing in the app. */
		/** `{author}` is a link to the author's page. */
		note: 'Kalamu is a project by {author} (University of Bayreuth). If it is useful to you, you can help fund its development. Supporting is entirely optional and does not unlock additional features.',
		link: 'Buy me a coffee'
	},
	/** Asked once for each cloud provider, before the first job, workflow run or dictionary
	 *  proposal sends the researcher's files to it. */
	consent: {
		title: 'Send your files to an AI provider?',
		/** `providers` is a list of provider names; `policy` the privacy policy's address. */
		ask: (providers: string, policy: string) =>
			`Kalamu is about to send the content of your files to ${providers}: the pages, recordings or text, with the names and any other personal information they hold. Each provider reads them with its AI models under your own account and API key, and keeps them as its own terms say. Kalamu’s developer receives nothing.\n\nYou are asked once for each provider. Kalamu’s privacy policy: ${policy}`,
		send: 'Send'
	},
	/** The Mac App Store's build, where macOS lets Kalamu write only in folders the researcher
	 *  has chosen. */
	access: {
		/** Before the folder picker opens. `folder` is the folder's name. */
		ask: (folder: string) =>
			`To save results next to your files, Kalamu needs your permission for the folder “${folder}”. Choose that folder in the next window.`,
		choose: 'Choose the folder…',
		/** Another folder was chosen than the one asked for. */
		missing: (folder: string) =>
			`Kalamu may not save in “${folder}”. Choose that folder when asked, or choose one folder for all results under Results.`
	},
	format: {
		txt: 'Text (.txt)',
		markdown: 'Markdown (.md)',
		docx: 'Word (.docx)',
		json: 'JSON',
		srt: 'Subtitles (.srt)',
		vtt: 'Subtitles (.vtt)',
		xlsx: 'Spreadsheet for Excel (.xlsx)',
		csv: 'Table for other tools: R, Python… (.csv)',
		searchable_pdf: 'Searchable PDF',
		alto: 'ALTO XML (word positions)',
		page_xml: 'PAGE XML (one file per page)'
	},
	close: {
		title: 'Jobs are still running',
		body: (tray: string) =>
			`They can keep running in the background, with Kalamu in ${tray}. Or quit now: the jobs pause, finished pages are kept, and you resume them next time from Jobs.`,
		background: 'Keep running in the background',
		quit: 'Pause and quit',
		stay: 'Cancel',
		waiting: (n: number) =>
			n === 1
				? 'Waiting for 1 piece already sent. It is paid for, so Kalamu stores it before closing; this takes a few minutes at most.'
				: `Waiting for ${n} pieces already sent. They are paid for, so Kalamu stores them before closing; this takes a few minutes at most.`,
		quitNow: 'Quit now',
		hidden: (reopen: string) =>
			`Kalamu is still working. Click its icon in ${reopen} to open it again.`
	},
	/** What each operating system calls the places Kalamu uses; `platform.ts` picks one. */
	/** A project saved as one file, to open on another computer or give to a colleague. */
	projectFile: {
		save: 'Save to a file…',
		title: 'Save this project to a file',
		lede: 'One file with this project, to open it on another computer, give it to a colleague, or keep it somewhere safe.',
		reading: 'Looking at what the file will hold…',
		inFile: 'In the file',
		documents: (documents: number, jobs: number) =>
			`${documents === 1 ? '1 document' : `${documents} documents`} and what ${
				jobs === 1 ? '1 job' : `${jobs} jobs`
			} read from them`,
		yourWork:
			'Your corrections and what you marked as checked, the collections, tags and descriptions, and the dictionary',
		never: 'Never in the file: your API keys, your settings, or anything from your other projects.',
		originals: 'Include the original files (scans, recordings)',
		originalsHint: (files: number, size: string) =>
			`${files === 1 ? '1 file' : `${files} files`}, about ${size}. Without them, the file holds the text and your work on it, and the pages and recordings stay on this computer.`,
		noOriginals: 'This project has no original files on this computer.',
		originalsMissing: (n: number) =>
			n === 1
				? '1 original file can no longer be found on this computer and will be left out.'
				: `${n} original files can no longer be found on this computer and will be left out.`,
		size: (size: string) => `The file will take about ${size}.`,
		bytes: (bytes: number) =>
			bytes < 1e6
				? 'less than 1 MB'
				: bytes < 1e9
					? `${Math.round(bytes / 1e6)} MB`
					: `${(bytes / 1e9).toFixed(1)} GB`,
		saveButton: 'Save…',
		cancel: 'Cancel',
		close: 'Close',
		saving: 'Saving the project…',
		leftOut: (n: number) =>
			n === 1
				? '1 original file was left out: it could not be found, or it changed since it was read.'
				: `${n} original files were left out: they could not be found, or they changed since they were read.`,
		show: { windows: 'Show in File Explorer', macos: 'Show in Finder' },
		fileType: 'Kalamu project',
		open: 'Open a project file…',
		opening: 'Opening the project file…',
		opened: (name: string, documents: number, jobs: number) =>
			`“${name}” is open, with ${documents === 1 ? '1 document' : `${documents} documents`} and ${
				jobs === 1 ? '1 job' : `${jobs} jobs`
			}. It is a new project beside your others: nothing of theirs was changed.`,
		openedWithout:
			'The original files were not saved in it: the text, your corrections and the exports are all here, and the pages and recordings will show once the project is saved again with its original files.',
		missingTexts: (n: number) =>
			n === 1
				? 'One corrected text that a job read was not in the file: the job and its results are here, without the text it read.'
				: `${n} corrected texts that jobs read were not in the file: the jobs and their results are here, without the texts they read.`,
		missingText: (what: string, jobs: string) => `${what}, read by ${jobs}`,
		check: 'Check a project file…',
		checkTitle: 'Check a project file',
		checkLede:
			'Kalamu reads the file and checks that everything in it is intact. It does not open it as a project and changes nothing.',
		checking: 'Checking the file…',
		verdict: {
			whole: 'This file is whole: everything in it can be opened.',
			incomplete: 'This file opens, but some texts are missing from it.',
			damaged:
				'This file is damaged: parts of it no longer match what was saved, so it cannot be opened. Ask for it again, or save it again.'
		},
		checkedFile: (name: string, date: string, version: string) =>
			`“${name}”, saved on ${date} by Kalamu ${version}.`,
		textsWhole: (n: number) =>
			n === 1
				? 'The corrected text the jobs read is in it, intact.'
				: `The ${n} corrected texts the jobs read are in it, intact.`,
		checkMissingTexts: (n: number) =>
			n === 1
				? 'One corrected text that a job read is not in the file: opening it brings back the job and its results, without the text it read.'
				: `${n} corrected texts that jobs read are not in the file: opening it brings back the jobs and their results, without the texts they read.`,
		damagedParts: (n: number): string =>
			n === 1
				? 'This part no longer matches what was saved:'
				: 'These parts no longer match what was saved:',
		originalsIn: (n: number) =>
			n === 1 ? '1 original file is in it, intact.' : `${n} original files are in it, intact.`,
		originalsNotChosen:
			'It was saved without the original files (scans, recordings): the text and your work on it are all in it.',
		originalsNotCarried: (n: number) =>
			n === 1
				? '1 original file could not be put in it: it could not be found, or it changed since it was read.'
				: `${n} original files could not be put in it: they could not be found, or they changed since they were read.`,
		toFind: (n: number, missing: string) =>
			n === 1
				? `On another computer, this document will show as “${missing}” until its file is found again:`
				: `On another computer, these ${n} documents will show as “${missing}” until their files are found again:`,
		renamed: (n: number) =>
			n === 1
				? 'Opened on this computer, 1 original file would take another name, because this system does not accept its name as it is, or another file already has it:'
				: `Opened on this computer, ${n} original files would take another name, because this system does not accept their names as they are, or another file already has them:`,
		renamedFile: (documents: string, from: string, to: string) =>
			`${documents ? `${documents}: ` : ''}${from} becomes ${to}`,
		savedChecked: (path: string) =>
			`The project is saved in ${path} and checked: everything in it can be opened.`
	},
	os: {
		windows: {
			keychain: 'Windows Credential Manager',
			tray: 'the notification area next to the clock',
			reopen: 'the notification area'
		},
		macos: {
			keychain: 'the macOS Keychain',
			tray: 'the menu bar',
			reopen: 'the menu bar or the Dock'
		}
	},
	/** Key names in shortcuts; on a Mac the modifier is written ⌘. */
	keys: {
		ctrl: 'Ctrl',
		shift: 'Shift',
		alt: 'Alt',
		escape: 'Esc',
		pageUp: 'Page Up',
		pageDown: 'Page Down',
		home: 'Home',
		end: 'End',
		space: 'Space',
		menu: 'Menu key'
	},
	tray: {
		open: 'Open Kalamu',
		pause: 'Pause all jobs',
		quit: 'Quit',
		quitPausing: 'Pause jobs and quit',
		idle: 'No jobs running',
		/** The status line, its numbers filled in by the app as jobs run: `percent` and `jobs`
		 * are placeholders, to be kept as they are. */
		runningOne: (percent: string) => `1 job running · ${percent}%`,
		runningMany: (jobs: string, percent: string) => `${jobs} jobs running · ${percent}%`
	},
	common: {
		cancel: 'Cancel',
		/** In the toast, after a removal it can take back. */
		undo: 'Undo',
		save: 'Save',
		close: 'Close',
		dismissMessage: 'Dismiss this message',
		loading: 'Loading…',
		/** After a label, before what it labels: French sets a space before it. */
		colon: ':',
		/** Between items of a list that hold commas of their own. */
		semicolon: '; ',
		/** A title quoted in running text. */
		quoted: (text: string) => `“${text}”`,
		/** What the file picker calls the files Kalamu reads. */
		sources: 'Sources',
		unknown: 'Not known',
		/** Said for a value left empty, such as the cost of a job nothing was paid for. */
		none: 'None',
		preview: 'preview',
		retiring: 'retiring',
		/** A model a newer one replaces, kept for set-ups that name it. */
		replaced: 'replaced',
		/** A model its provider no longer serves. */
		retired: 'no longer served',
		opensInBrowser: ' (opens in your browser)',
		/** An exported record's Markdown, written under another name than the JSON's. */
		markdownBeside: (name: string) =>
			`The Markdown was saved as “${name}”, since a file of yours already had its name.`,
		/** An exported table's workbook or CSV, written under another name than the file chosen,
		 *  `names` already listed and quoted. */
		tableBeside: (names: string, count: number) =>
			count === 1
				? `The table was saved as ${names}, since a file of yours already had that name.`
				: `The table was saved as ${names}, since files of yours already had those names.`,
		linkFailed: (url: string) => `This page could not be opened in your browser: ${url}`,
		/** Asked before leaving a correction, a prompt or a workflow with changes not saved. */
		unsaved: 'Your changes are not saved. Leave and discard them?',
		discardChanges: 'Discard changes',
		keepEditing: 'Keep editing',
		/** Under a long table drawn a page at a time. */
		shownRows: (shown: number, total: number) => `${shown} of ${total} shown`,
		showMore: (n: number) => `Show ${n} more`,
		/** An error that names the step of a workflow at fault. */
		atStep: (n: number, sentence: string) => `Step ${n}: ${sentence}`
	},
	error: {
		'meaning.unavailable': 'The search by meaning is not available in this copy of Kalamu.',
		'meaning.no_model': 'Choose a model for the search by meaning first.',
		'meaning.unknown_model': 'This model is not offered any more. Choose another.',
		'meaning.no_consent':
			'Agree first to send this project’s text to the model’s provider: nothing has been sent.',
		'meaning.running': 'The project’s text is already being prepared.',
		'meaning.empty_query': 'Type a question or a description.',
		'search.too_short': 'Type a word of two letters at least.',
		'search.stopped': 'The search was replaced by a newer one.',
		'search.index':
			'The search could not read its copy of your results’ text. Prepare the text again from the search; your results are not affected.',
		'project.not_found': 'This project, or what it refers to, no longer exists.',
		'project.elsewhere': 'This belongs to another project, or no longer exists.',
		'project.no_name': 'Give it a name.',
		'project.no_tag': 'Type the tag to add.',
		'project.seed': 'A seed is a number of six digits at most.',
		'project.export': 'The export could not be written.',
		'dictionary.no_entry':
			'This entry is no longer in the dictionary. It may have been merged or split.',
		'dictionary.no_label': 'Give the entry a label.',
		'dictionary.namespaces':
			'These come from different fields, people and places for example, and stay apart.',
		'dictionary.kept_apart':
			'You decided to keep some of these apart. Withdraw that decision first to merge them.',
		'dictionary.too_few': 'Choose at least two rows.',
		'dictionary.same_entry': 'These are already in one entry. Take one out of it first.',
		'dictionary.nothing_to_compare': 'This field has fewer than two rows to compare.',
		'unit.pages_empty': 'Name at least one page.',
		'unit.pages_unreadable': 'Use page numbers and ranges, such as 1, 3-5, 8 (left).',
		'unit.no_such_page': 'This document has no such page.',
		'unit.not_contents': 'Only a table of contents has items.',
		'unit.already_made': 'The items were already made from these entries.',
		'unit.no_pages': 'Give this item its pages first.',
		'unit.not_pdf': 'Only a PDF’s items can be read on their own for now.',
		'unit.not_joinable': 'Only two different items of the same table of contents can be joined.',
		'unit.other_result': 'This item belongs to another table of contents. Open it from there.',
		'unit.bad_split': 'Split an item between two of its pages.',
		'unit.none_chosen': 'Choose at least one item.',
		'job.not_found': 'This job no longer exists.',
		'job.not_text':
			'Only extracted text, transcripts, corrected text and translations can be analysed this way.',
		'job.nothing_read':
			'Nothing has been read yet, so there is nothing to analyse. Run the job, or wait for its first pages.',
		'passage.too_short': 'Select at least 3 letters or figures to correct.',
		'passage.too_long':
			'The passage is too long to correct at once; correct a longer stretch with a correction job.',
		'passage.context': 'The context around the passage is too long.',
		'passage.changed':
			'The text under the selection changed, so nothing was sent or applied. Select the passage again.',
		'passage.empty': 'The model’s answer was empty.',
		'passage.too_different':
			'The model’s answer is too different from the passage to be a correction of it.',
		'passage.not_waiting': 'This proposal is no longer waiting for a decision.',
		'region.not_finite': 'The region could not be measured. Draw it again.',
		'region.outside': 'The region reaches outside the page. Draw it within the page.',
		'region.too_small': 'The region is too small. Draw a larger box.',
		'region.no_page': 'This page is not in the file, so no region can be drawn on it.',
		'region.adjusts':
			'The region you changed is no longer on this page as it was read. Draw the box again.',
		'region.not_found': 'This region no longer exists.',
		'region_reading.changed':
			'The words where this reading would go have changed, so nothing was sent or replaced. Look at the page again.',
		'region_reading.nowhere':
			'This reading has no certain place in the page’s text. Copy it instead.',
		'region_reading.empty': 'The model found no text in this region.',
		'region_reading.cannot_read': 'This model cannot read the text of a page. Choose another.',
		'region_reading.not_waiting': 'This reading is no longer waiting for a decision.',
		'media.region_not_finite': 'The region could not be measured. Draw it again.',
		'media.region_outside_page':
			'The region lies outside the page. Draw it again over the part to read.',
		'media.region_too_small': 'The region is too small to read. Draw a larger box.',
		'media.region_no_page': 'Only a page of a PDF or an image can be read again by region.',
		'media.region_no_images':
			'This model does not read pictures, and a region is sent as one. Choose a model that reads pages.',
		'job.database': 'The job database could not be read or written.',
		'job.running': 'This job is already running.',
		'job.empty': 'Add at least one file and choose a model.',
		'job.no_language': 'Choose the language to translate into.',
		'job.mixed_reading':
			'An analysis giving one row per entry reads page by page, so it cannot be compared with one answering once per document. Choose the same kind of analysis for every model.',
		'job.link_needs_folder':
			'Choose a folder for the results: a YouTube video has no folder of its own to write them next to.',
		'source.private':
			'This video is private, so it cannot be read. Only public videos can be transcribed.',
		'source.not_found': 'YouTube has no video at this link: it may have been removed.',
		'source.not_youtube':
			'This is not a link to a YouTube video. Use a watch, youtu.be, Shorts or live link; playlists and channels cannot be read.',
		'media.encrypted':
			'This PDF is password-protected. Remove the password, then add the file again.',
		'media.unsupported': 'This file type is not supported yet.',
		'media.read': 'The file could not be read.',
		'prompt.invalid': 'The prompt could not be saved: it needs a name and instructions.',
		'prompt.no_fields': 'An analysis needs at least one field.',
		'prompt.unnamed_field': 'Every field needs a name.',
		'prompt.duplicate_field': 'Two fields have the same name.',
		'prompt.reserved_field':
			'This is the name Kalamu gives the quotations: give the field another name.',
		'job.cited_needs_text':
			'Quotations can only be found in a transcript or a text. Transcribe or read this file first, then choose Use this text.',
		'job.text_too_long':
			'This text is too long for the model chosen to read at once. Choose a model that reads more.',
		'cited.not_cited': 'This answer holds no quotations to change.',
		'cited.no_entry': 'This entry is no longer in the answers. Look at them again.',
		'cited.no_quotation': 'This quotation is no longer in the answers. Look at them again.',
		'cited.no_field': 'This field is not one of the entry’s.',
		'cited.not_a_choice': 'Choose one of the field’s values, or leave it empty.',
		'cited.not_in_turn': 'Select a passage within one turn of the transcript.',
		'cited.invented':
			'Only quotations Kalamu found word for word in the text can be kept: this one is not among them.',
		'prompt.few_values': 'A choice needs at least two values to choose from.',
		'prompt.empty_group': 'One row per entry needs at least one column.',
		'prompt.nested_group': 'A column cannot itself hold rows.',
		'prompt.two_groups': 'An analysis gives one row per entry once, not twice.',
		'prompt.group_alone':
			'With one row per entry, every field is a column of the row: move the other fields into it.',
		'records.not_records': 'This analysis has no rows to correct.',
		'records.no_row': 'This row is no longer on the page. The rows were read again.',
		'records.no_column': 'This column is not in the analysis.',
		'records.not_a_choice': 'Choose one of the column’s values, or leave the cell empty.',
		'prompt.unnamed_pipeline': 'A pipeline needs a name and at least one model.',
		'workflow.incomplete': 'A workflow needs a name and at least one step, each with a model.',
		'workflow.no_steps': 'A workflow needs at least one step.',
		'workflow.not_earlier': 'A step can only read a step that comes before it.',
		'workflow.not_text':
			'A step can only read the text of an extraction, a transcription, a correction or a translation.',
		'workflow.cannot_read_text':
			'This step’s task does not read text: let it read the files instead.',
		'workflow.not_finished': 'This step has not finished yet.',
		'profile.no_name': 'An export profile needs a name.',
		'profile.no_columns': 'An export profile needs at least one column.',
		'profile.unnamed_column': 'Every column of an export profile needs a heading.',
		'profile.bad_heading': 'A column’s heading must fit on one line.',
		'profile.duplicate_heading': 'Two columns have the same heading.',
		'profile.bad_separator': 'The list separator cannot be empty or hold a line break.',
		'profile.bad_value': 'A column asks for a value Kalamu cannot give.',
		'prompt.invalid_file':
			'This is not a file of Kalamu prompts, a pipeline or a workflow, or it comes from a newer version of Kalamu.',
		'provider.unknown': 'Unknown provider.',
		'provider.missing_key': 'No API key is saved for this provider. Add one in Settings.',
		'provider.consent':
			'Kalamu has not been allowed to send your files to this provider. Start again and choose Send when asked.',
		'provider.unsupported': 'This model cannot do this task.',
		'provider.retired':
			'Its provider no longer serves this model. Choose the model that replaces it in New job.',
		'provider.cannot_read':
			'This model cannot read some of the files: images need a model that reads images, and an analysis of a PDF a model that reads PDFs. Choose another model, or extract the text first and analyse the result.',
		'source.unsupported':
			'One of the files cannot be read for this task. Remove it, or choose another task.',
		'provider.auth': 'The provider rejected the API key. Check it in Settings.',
		'provider.quota': 'Your account with this provider has no credit or quota left.',
		'provider.rate_limited': 'The provider asked Kalamu to slow down; the page will be retried.',
		'provider.transient': 'The provider could not be reached.',
		'provider.blocked': 'The provider refused to return text for this page.',
		'provider.invalid_request': 'The provider rejected the request.',
		'provider.other': 'The provider sent an unexpected response.',
		'provider.invalid_answer':
			'The model’s answer did not give the fields of the analysis, even when asked again. Choose a higher level under Thinking, or another model.',
		'provider.cut_off':
			'The answer stopped at the model’s output limit before it was complete. Send fewer pages in each call, ask for fewer or shorter fields, or choose a model that can write a longer answer.',
		'provider.cut_off_filter':
			'The provider’s filter stopped the answer before it was complete, so it cannot be read: the provider will not write about something in this text. Try another model or provider.',
		'provider.cut_off_other':
			'The provider stopped the answer before it was complete, for a reason other than the output limit, so it cannot be read. Try again later, or choose another model.',
		'provider.not_installed':
			'This offline model, or the language data it needs, has not been downloaded. Download it in Settings.',
		'provider.interrupted': 'Stopped before the end; it will continue when the job resumes.',
		'offline.unknown_model': 'This offline model is not known to this version of Kalamu.',
		'offline.busy': 'Another model is downloading; wait for it to finish.',
		'offline.in_use': 'A running job uses this model; pause it first.',
		'offline.cancelled': 'Download cancelled.',
		'offline.download':
			'The model could not be downloaded. Check your internet connection and try again: the download continues where it stopped.',
		'offline.integrity': 'The downloaded model was damaged and has been discarded. Try again.',
		'offline.io': 'The model file could not be read or written.',
		'offline.no_space':
			'There is not enough free space on the drive that holds the models. Free some space and try again.',
		'output.write': 'The result files could not be written.',
		'keys.keychain': 'The system’s keychain refused the request.',
		'keys.format': 'This key contains a character no API key has, so it was not saved.',
		'settings.write': 'The settings could not be saved.',
		'catalog.download': 'The model list could not be downloaded.',
		'catalog.invalid': 'The downloaded model list could not be read.',
		'catalog.refused':
			'The downloaded model list failed Kalamu’s checks, so it is not used. Kalamu keeps the list it has.',
		'file.read': 'The file could not be read or written.',
		'file.open': 'This file could not be opened. It may have been moved or deleted.',
		'file.missing':
			'The file is no longer where it was added, so its pages cannot be shown. Its results are kept; put it back to see it here.',
		'app.internal': 'Something went wrong inside the app.',
		'app.jobs_running': 'Not while a job is running: pause it or wait for it to finish.',
		'app.notDesktop': 'This works only in the desktop app.',
		'project.file_not_kalamu': 'This file is not a Kalamu project file.',
		'project.file_damaged':
			'This project file is damaged, perhaps cut short while it was copied. Ask for it again, or copy it again.',
		'project.file_newer':
			'This project file was saved by a newer version of Kalamu. Update Kalamu, then open it again.',
		'project.file_write':
			'The project file could not be written. Check that there is enough free space, or choose another place.',
		'project.file_read': 'The project file could not be read.',
		'project.frozen_text_missing':
			'The project was not saved: the corrected text a job read, which Kalamu kept as it was then, is no longer in Kalamu’s data folder. A file saved earlier under this name is unchanged.',
		'project.frozen_text_changed':
			'The project was not saved: the corrected text a job read, which Kalamu kept as it was then, has changed since. A file saved earlier under this name is unchanged.'
	}
};

export type Messages = typeof en;
