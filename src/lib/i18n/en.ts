// The English interface. Every visible string lives here; other languages are `fr.ts` and the
// like, typed `: Messages` so a missing or extra key fails the type check, as in Live
// Translation.

export const en = {
	locale: { name: 'English', tag: 'en' },
	/** Languages a model lists that the system has no name for. */
	languageNames: { kea: 'Kabuverdianu', rup: 'Aromanian' } as Record<string, string>,
	/**
	 * The default prompts' names and descriptions, by id, where the interface language differs
	 * from the core's English; empty here, since the core's text is English.
	 */
	presets: {} as Record<string, { name: string; description: string }>,
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
		offlineReady: 'Whisper ready on this computer',
		collapse: 'Hide the job list',
		expand: 'Show the job list',
		sidebarShortcut: 'B',
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
		dragHint: 'Drag it onto a project in the list to add it there'
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
		noSteps:
			'Add a first step: a prompt, as Kalamu ships it or as you wrote it, or one of your saved pipelines.',
		addStep: 'Add a step',
		addStepHint: 'Choose a prompt or a pipeline',
		fromPipelines: 'Your pipelines',
		fromPrompts: (task: string) => `Prompts · ${task}`,
		prompt: 'Prompt',
		promptMissing: 'A prompt no longer listed',
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
		fileName: (project: string) => `${project} dictionary`,
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
		mode: 'Find',
		modes: {
			phrase: 'The exact phrase',
			all_words: 'All these words',
			near: 'These words near each other'
		},
		near: 'At most',
		nearWords: 'words apart',
		quotesHint:
			'Each word is looked for in the same page or answer. Put words that belong together between quotes: “mission catholique” Koupéla.',
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
			'Where you chose a preferred result for a document, only it is read. Elsewhere every result is read and listed here: set aside those you do not want to check. A document read by several models is still one document.',
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
		empty: 'No document here has been read yet. Read some with a new job, then come back.',
		detail: {
			failed: (error: string | null) => (error ? `Failed: ${error}` : 'Failed'),
			notRead: 'Not read yet',
			neverStarted: 'Not started yet',
			truncated: 'Stopped at the model’s length limit: the end may be missing',
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
			exported: 'The descriptions were saved as JSON, with a CSV beside it.'
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
				'Kalamu reads PDFs, images (JPEG, PNG, TIFF, WebP, HEIC), recordings, video files and texts (TXT, Markdown, Word, Excel, CSV).',
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
		/** Under the reading step, for a model that reports where text sits (Tesseract, Mistral
		 * OCR); the second names it beside models that do not. */
		extractPlace: 'And notes where its text sits on the page',
		extractPlaces: (models: string) => `${models} also notes where its text sits`,
		/** Under the reading step, for models that do not (language models). */
		extractTextOnly: 'Text only, not where it sits on the page: no searchable PDF or ALTO',
		extractSearchable: 'Also ALTO XML and a searchable PDF, each word in its place',
		extractPdfOnly: 'The searchable PDF for PDF files only',
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
		previous: 'Previous',
		next: 'Next',
		source: 'Source',
		itemMenu: (name: string) => `What to do with “${name}”`,
		renameItem: 'Rename',
		removeItem: 'Remove from the answers',
		itemLocates: 'Marked in the source while it has the focus.',
		itemKeys:
			'Marked in the source while it has the focus. Enter, or the menu key, renames or removes it.',
		pageImage: 'Page image: the arrow keys move it, with Shift by longer steps',
		itemRemoved: (name: string) => `“${name}” removed from the answers`,
		itemEverywhere:
			'Changed in this document’s corrected answers, for every model that found it. The model’s own answers are kept.',
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
		lede: 'Keys, offline models, processing, appearance and language.',
		tabsLabel: 'Settings sections',
		tabs: {
			keys: 'API keys',
			offline: 'Offline models',
			general: 'General',
			about: 'About'
		},
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
		addLanguageData: 'Download a language for Tesseract',
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
		clearUnusedConfirm:
			'Remove the answers kept from deleted jobs? Asking for the same pages again would be paid for again.',
		clearUnusedOk: 'Clear',
		clearUnusedDone: (count: number, answers: string, size: string) =>
			count === 0
				? 'No unused answers to remove.'
				: `${answers} ${count === 1 ? 'answer' : 'answers'} removed, ${size} freed.`,
		/** At start-up, when the settings file could not be read and was moved aside. */
		setAside: (path: string) =>
			`Your settings could not be read, so Kalamu started with its defaults. The old file is kept as ${path}.`
	},
	report: {
		/** Bugs and feedback; also the way Store policy 11.16 asks for to flag harmful output,
		 * which the note names so a reviewer finds it. */
		link: 'Report a problem',
		review: 'Report a problem',
		note: 'A bug, an idea, or offensive output from a model: tell us on GitHub. It opens in your browser; Kalamu sends nothing, so share only what is safe to make public.'
	},
	glossary: {
		title: 'Glossary',
		lede: 'The words Kalamu uses, for colleagues new to these tools, and what each export format suits.',
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
			start: 'Start the job',
			addLinks: 'Add the YouTube links pasted',
			newLine: 'Start another line among the links',
			reviewed: 'Mark the page reviewed, or not reviewed; while correcting, save the correction',
			nextPage: 'Next page or part; Page Down scrolls a pane instead when it has the focus',
			previousPage: 'Previous page or part; Page Up scrolls a pane instead when it has the focus',
			differenceSteps: 'On a difference in the text: the previous or next one',
			differenceEnds: 'On a difference in the text: the first or last one',
			openReadings: 'On a difference in the text: open its readings, the focus in them',
			nameSteps:
				'On a name in an analysis: the previous or next one (Home and End: the first or last)',
			nameMenu: 'On a name in an analysis: the menu to rename or remove it',
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
				text: 'Tesseract is an open OCR engine that Kalamu runs on your own computer. Download each language you read once in Settings; after that nothing is sent anywhere and nothing is billed. It reads clean print well and notes where every word sits on the page, which gives ALTO XML and searchable PDFs. Handwriting, faded or damaged pages and complex layouts read better with an AI model. It does not detect the language, so you choose it, and its speed depends on your computer’s processor.'
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
			csv: 'A spreadsheet with one row per page, segment, document or entry (an analysis has a column per field), for Excel, LibreOffice or statistics software.',
			srt: 'Subtitles with their times, for video players and editing software.',
			vtt: 'Subtitles for the web (WebVTT), as web video players use them.',
			searchable_pdf:
				'The original PDF with an invisible text layer, so it can be searched and its text copied. Only from models that report where text sits on the page: currently Tesseract and Mistral OCR.',
			alto: 'Each page’s text with where every block, line and word sits, in the ALTO standard that digital libraries, eScriptorium and Transkribus read. Only from models that report where text sits on the page: Tesseract places each word; Mistral OCR places paragraphs, whose lines and words are then estimated. Corrections keep their words’ places.',
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
		csv: 'Spreadsheet (.csv)',
		searchable_pdf: 'Searchable PDF',
		alto: 'ALTO XML (word positions)'
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
		preview: 'preview',
		retiring: 'retiring',
		opensInBrowser: ' (opens in your browser)',
		/** An exported record's Markdown, written under another name than the JSON's. */
		markdownBeside: (name: string) =>
			`The Markdown was saved as “${name}”, since a file of yours already had its name.`,
		linkFailed: (url: string) => `This page could not be opened in your browser: ${url}`,
		/** Asked before leaving a correction, a prompt or a workflow with changes not saved. */
		unsaved: 'Your changes are not saved. Leave and discard them?',
		discardChanges: 'Discard changes',
		keepEditing: 'Keep editing'
	},
	error: {
		'search.too_short': 'Type a word of three letters at least.',
		'search.stopped': 'The search was replaced by a newer one.',
		'search.index':
			'The search could not read its copy of your results’ text. Prepare the text again from the search; your results are not affected.',
		'project.not_found': 'This project, or what it refers to, no longer exists.',
		'project.elsewhere': 'This belongs to another project, or no longer exists.',
		'project.no_name': 'Give it a name.',
		'project.no_tag': 'Type the tag to add.',
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
		'provider.cut_off':
			'The answer stopped at the model’s output limit before it was complete. Send fewer pages in each call, ask for fewer or shorter fields, or choose a model that can write a longer answer.',
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
		'file.read': 'The file could not be read or written.',
		'file.open': 'This file could not be opened. It may have been moved or deleted.',
		'file.missing':
			'The file is no longer where it was added, so its pages cannot be shown. Its results are kept; put it back to see it here.',
		'app.internal': 'Something went wrong inside the app.',
		'app.jobs_running': 'Not while a job is running: pause it or wait for it to finish.',
		'app.notDesktop': 'This works only in the desktop app.'
	}
};

export type Messages = typeof en;
