// The English interface. Every visible string lives here so French and German can be added as
// `fr.ts` and `de.ts` typed `: Messages` without touching components, as in Live Translation.

export const en = {
	locale: { name: 'English', tag: 'en' },
	/** Languages a model lists that the system has no name for. */
	languageNames: { kea: 'Kabuverdianu', rup: 'Aromanian' } as Record<string, string>,
	app: {
		name: 'Kalamu',
		tagline: 'Transcription and text extraction'
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
		offlineReady: 'Whisper ready on this computer',
		collapse: 'Hide the job list',
		expand: 'Show the job list',
		sidebarShortcut: 'B',
		/** The menu a right click (Control-click on a Mac) opens on a job in the list. */
		jobMenu: (name: string) => `Actions for “${name}”`,
		addToProject: 'Add to a project',
		noProjects: 'No project yet. Create one under Projects.',
		addedTo: (project: string) => `Added to the project “${project}”.`,
		pauseFirst: 'Pause or cancel the job before deleting it.'
	},
	nav: {
		newJob: 'New job',
		projects: 'Projects',
		workflows: 'Workflows',
		jobs: 'Jobs',
		prompts: 'Prompts',
		settings: 'Settings',
		glossary: 'Glossary'
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
			'Machine-read text from a digital library or a PDF, corrected page by page: TXT, Word, PDF'
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
		workflow: 'Run a workflow on it',
		workflowHint: 'Several steps one after another, from your workflows or an example.'
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
			'Each selected item’s pages as a PDF in a folder you choose, with a list of the items as CSV and JSON.',
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
		noSteps: 'Add a first step from one of your pipelines.',
		noPipelines:
			'Steps are made from pipelines. Set up a job in New job and save it as a pipeline first.',
		addStep: 'Add a step',
		addStepHint: 'From a pipeline',
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
		noEvidence: 'No result lists it any longer.',
		openReview: 'Open the review',
		export: 'Export',
		exported: 'The dictionary was saved as JSON, with a CSV of every form found beside it.',
		proposals: {
			title: 'Proposed merges',
			ask: 'Ask a model to propose merges',
			plan: (field: string, rows: number, groups: number) =>
				groups === 1
					? `All ${rows} rows of ${field} are sent in one request, with their spellings, the titles of up to three documents naming each, and the pairs you kept apart.`
					: `The ${rows} rows of ${field} are too many for one request: they are sent in ${groups} requests of about 100,000 tokens each, in alphabetical order with accents set aside, each sharing a tenth of its rows with the next. Names far apart in the alphabet are not compared. Each row goes with its spellings, the titles of up to three documents naming it, and the pairs you kept apart.`,
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
		tooShort: 'Type three letters at least.',
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
		deleteConfirm: (name: string) =>
			`Delete the project “${name}”? Its collections and documents are removed from Kalamu. The files on your computer are kept, and the jobs and their results stay under Jobs.`,
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
		notReadable: (n: number) =>
			n === 1
				? '1 file could not be read and was left out.'
				: `${n} files could not be read and were left out.`,
		attachJob: 'Add an existing job',
		noDocuments:
			'No documents here yet. Add files, or add a job you ran before: its results come with it, without being paid for again.',
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
		tagged: (tag: string, n: number) =>
			`${n === 1 ? '1 document' : `${n} documents`} tagged “${tag}”`,
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
			'The file is no longer where it was added. Its results are kept; put it back to read it again.',
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
		taskResults: (task: string, n: number, reviewed: number) =>
			`${task}: ${n === 1 ? '1 result' : `${n} results`}${reviewed > 0 ? `, ${reviewed} reviewed` : ''}`,
		preferredMark: 'preferred',
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
			unprefer: 'No longer preferred',
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
		pipelineExported: (file: string) =>
			`Saved ${file}.json, to import into Kalamu, and ${file}.md, for a methods section.`,
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
				'All pages are sent together to the provider’s batch service, at about half the price. Results can take up to a day; Kalamu can be closed in the meantime and collects them when it opens again.'
		},
		paceNotBatched: (names: string) =>
			`${names} has no batch service in Kalamu and still runs page by page, at its usual price.`,
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
			`Very quiet recording (about ${db} dB): quiet speech may be missed. Evening out the volume usually helps.`
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
		/** Under the reading step, for a model that reports where text sits (Mistral OCR); the
		 * second names it beside models that do not. */
		extractPlace: 'And notes where each block of text sits on the page',
		extractPlaces: (models: string) => `${models} also notes where each block of text sits`,
		/** Under the reading step, for models that do not (language models). */
		extractTextOnly: 'Text only, not where it sits on the page: no searchable PDF',
		extractSearchable: 'Also a searchable PDF: the original, with an invisible text layer',
		extractPdfOnly: 'For PDF files only, not images',
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
			'The model’s answer stopped at its length limit, so the end of these pieces may be missing. The review marks them; check them against the source.',
		files: 'Files',
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
		pageHalf: (n: number, half: 'left' | 'right') =>
			`Page ${n}, ${half === 'left' ? 'left' : 'right'} half`,
		pageHalfShort: (n: number, half: 'left' | 'right') => `${n}${half === 'left' ? 'l' : 'r'}`,
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
			'This video cannot be played here: its owner may not allow it, or this computer is offline.',
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
		truncated:
			'The model’s answer stopped at its length limit here, so the end may be missing. Check it against the source, or correct it.',
		noPreview: 'No preview for this file type.',
		notDone: 'Not read yet.',
		saved: 'Saved; the result files were updated.',
		pieceReviewed: (label: string) => `${label}, reviewed`,
		zoomOutShortcut: '−',
		zoomInShortcut: '+',
		zoomFitShortcut: '0',
		zoomIn: 'Zoom in',
		zoomOut: 'Zoom out',
		turnLeft: 'Turn the page left',
		turnRight: 'Turn the page right',
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
			'Give the model’s labels real names. They replace the labels here and in every file; the model’s own output is kept.',
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
		noAnswerFound: 'No answer holds these words.'
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
			'An analysis is a set of fields to fill in for each document. Duplicate one of these to make your own.',
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
		offlineTitle: 'Offline transcription',
		offlineHint:
			'Whisper runs on this computer: recordings never leave it, no key or account is needed, and it works without an internet connection once a model is downloaded. It is slower than the cloud models and cannot label speakers. Large v3 Turbo suits most computers.',
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
		concurrencyHint:
			'Takes effect the next time Kalamu opens. Lower it if a provider asks Kalamu to slow down.',
		catalogTitle: 'Models',
		catalogUpdated: (date: string) => `Model list from ${date}`,
		catalogRefresh: 'Check for an updated model list',
		catalogAuto: 'Check at startup',
		appearanceTitle: 'Appearance',
		theme: 'Theme',
		themeSystem: 'Same as the system',
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
			'Nothing is sent to the developer: no files, keys, results or usage data. AI output can contain fluent but wrong text; check a sample against the source.'
	},
	report: {
		/** Store policy 11.16: a way to report what the models write. */
		link: 'Report a problem',
		review: 'Report a problem with this output',
		note: 'Output that is inappropriate, harmful or invented, or a bug: report it on GitHub. The form opens in your browser; Kalamu sends nothing, so share only what is safe to make public.'
	},
	glossary: {
		title: 'Glossary',
		lede: 'The words Kalamu uses, for colleagues new to these tools, and what each export format suits.',
		wordsTitle: 'Words',
		formatsTitle: 'Export formats',
		filter: 'Find a word',
		noMatch: 'No word by that name.',
		seeAlso: 'See also',
		/** The help link beside a word in the app. */
		whatIs: (word: string) => `What is meant by “${word}”`,
		/** At the foot of a word's definition, opened from its help link. */
		openGlossary: 'Open the glossary',
		/** By theme: what a model is, then how it is used, then how its results are checked and
		 * recorded. The page lists them alphabetically by word. */
		terms: {
			ocr: {
				word: 'OCR and HTR',
				text: 'Optical character recognition (OCR) turns printed text in an image into text you can search and edit; handwritten text recognition (HTR) does the same for handwriting. Kalamu does both with AI models that read the whole page at once, rather than letter by letter as older OCR engines did. They cope better with columns, faded print and difficult hands, but can also go wrong in new ways (see hallucination).'
			},
			llm: {
				word: 'LLM (large language model)',
				text: 'A model trained on very large amounts of text to write the most likely continuation of what it is given. Gemini, GPT, Claude and Mistral’s models are LLMs that also read images and sound. What they write is fluent and often right, but never guaranteed: it is a likely text, not a copy.'
			},
			whisper: {
				word: 'Offline transcription (Whisper)',
				text: 'Whisper is an open speech-recognition model that Kalamu runs on your own computer, after a one-time download in Settings. Nothing is sent anywhere and nothing is billed. It is slower than the cloud models and cannot tell speakers apart.'
			},
			apiKey: {
				word: 'API key',
				text: 'A code from an AI provider (Google, OpenAI, Anthropic, Mistral) that lets Kalamu send work under your account; the provider bills what is used. Kalamu keeps keys in the system’s keychain (Credential Manager on Windows, the Keychain on a Mac) and sends each only to the provider that issued it.'
			},
			prompt: {
				word: 'Prompt',
				text: 'The instructions sent to the model with each page, recording or text: what to transcribe, how to lay it out, which language, what to leave out. Kalamu comes with prompts written for common kinds of material, which you can adapt in Prompts. Dedicated OCR and speech models, such as Mistral OCR, Voxtral and Whisper, take no prompt.'
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
			economy: {
				word: 'Cheaper mode (batches)',
				text: 'Sends a job to the provider’s batch service, which answers within a day at about half the price. Kalamu collects the answers when it next opens; the app does not need to stay open meanwhile.'
			},
			groundTruth: {
				word: 'Ground truth',
				text: 'A text known to be correct, against which a model’s output is measured. In Kalamu, every page you correct, or mark as reviewed without changing it, becomes the ground truth for that page.'
			},
			errorRates: {
				word: 'Character and word error rates (CER, WER)',
				text: 'The share of characters, or of words, that a model got wrong compared with the ground truth: those added, left out or replaced, divided by the length of the correct text. A character error rate of 2% is about one character wrong in fifty. Kalamu counts them only on the pages you reviewed, so they show the errors you caught, not the ones you missed.'
			},
			hallucination: {
				word: 'Hallucination',
				text: 'Text a model writes that is not in the source: a plausible word where the page is illegible, a sentence nobody spoke, a name an analysis supplies from elsewhere. Fluent output is no evidence of accuracy. Checking a sample against the source, as the review allows, is the safeguard.'
			},
			provenance: {
				word: 'Provenance',
				text: 'The record of how a result was made. Beside every output Kalamu writes a provenance.json file with the model and its exact version, the prompt, the settings each request carried, dates, costs, and which pages were corrected or reviewed, so the method can be described and repeated.'
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
			csv: 'A spreadsheet with one row per page, segment, document or entry (an analysis has a column per field), for Excel, LibreOffice or statistics software.',
			srt: 'Subtitles with their times, for video players and editing software.',
			vtt: 'Subtitles for the web (WebVTT), as web video players use them.',
			searchable_pdf:
				'The original PDF with an invisible text layer, so it can be searched and its text copied. Only from models that report where text sits on the page: currently Mistral OCR.',
			provenance: 'Always written beside the results: how they were made (see Provenance).'
		}
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
		ctrl: 'Ctrl'
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
		unknown: 'Not known',
		preview: 'preview',
		retiring: 'retiring',
		opensInBrowser: ' (opens in your browser)',
		linkFailed: (url: string) => `This page could not be opened in your browser: ${url}`
	},
	error: {
		'search.too_short': 'Type three letters at least.',
		'search.stopped': 'The search was replaced by a newer one.',
		'search.index':
			'The search could not read its copy of your results’ text. Prepare the text again from the search; your results are not affected.',
		'project.not_found': 'This project, or what it refers to, no longer exists.',
		'project.elsewhere': 'This belongs to another project, or no longer exists.',
		'project.no_name': 'Give it a name.',
		'project.no_tag': 'Type the tag to add.',
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
		'job.not_found': 'This job no longer exists.',
		'job.not_text': 'Only extracted text, transcripts and corrected text can be analysed this way.',
		'job.nothing_read':
			'Nothing has been read yet, so there is nothing to analyse. Run the job, or wait for its first pages.',
		'job.database': 'The job database could not be read or written.',
		'job.running': 'This job is already running.',
		'job.empty': 'Add at least one file and choose a model.',
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
		'prompt.few_values': 'A choice needs at least two values to choose from.',
		'prompt.unnamed_pipeline': 'A pipeline needs a name and at least one model.',
		'workflow.incomplete': 'A workflow needs a name and at least one step, each with a model.',
		'workflow.no_steps': 'A workflow needs at least one step.',
		'workflow.not_earlier': 'A step can only read a step that comes before it.',
		'workflow.not_text':
			'A step can only read the text of an extraction, a transcription or a correction.',
		'workflow.cannot_read_text':
			'This step’s task does not read text: let it read the files instead.',
		'workflow.not_finished': 'This step has not finished yet.',
		'prompt.invalid_file':
			'This is not a file of Kalamu prompts, a pipeline or a workflow, or it comes from a newer version of Kalamu.',
		'provider.unknown': 'Unknown provider.',
		'provider.missing_key': 'No API key is saved for this provider. Add one in Settings.',
		'provider.unsupported': 'This model cannot do this task.',
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
			'The model’s answer did not match the fields of the analysis; Kalamu asks it again.',
		'provider.not_installed':
			'This offline model has not been downloaded. Download it in Settings.',
		'provider.interrupted': 'Stopped before the end; it will continue when the job resumes.',
		'offline.unknown_model': 'This offline model is not known to this version of Kalamu.',
		'offline.busy': 'Another model is downloading; wait for it to finish.',
		'offline.in_use': 'A running job uses this model; pause it first.',
		'offline.cancelled': 'Download cancelled.',
		'offline.download':
			'The model could not be downloaded. Check your internet connection and try again.',
		'offline.integrity': 'The downloaded model was damaged and has been discarded. Try again.',
		'offline.io': 'The model file could not be read or written.',
		'output.write': 'The result files could not be written.',
		'keys.keychain': 'The system’s keychain refused the request.',
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
