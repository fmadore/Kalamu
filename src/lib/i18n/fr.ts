// The French interface, typed `Messages` so that a key missing or added here fails the type
// check. Written as a French-speaking researcher reads it: the terms of archival practice and
// the digital humanities (pièce, vérité terrain, couche texte), not calques of the English.
// Product, provider and model names stay as they are. French typography: « » with no-break
// spaces, and a no-break space before : ; ? and !.

import type { Messages } from './en.js';

export const fr: Messages = {
	locale: { name: 'Français', tag: 'fr' },
	/** Languages a model lists that the system has no name for. */
	languageNames: { kea: 'capverdien', rup: 'aroumain' } as Record<string, string>,
	/** The default prompts, by id: the core writes them in English. */
	presets: {
		'text_extraction.printed': {
			name: 'Document imprimé ou dactylographié',
			description:
				'Livres, rapports, lettres dactylographiées et formulaires, en toute langue, de gauche à droite ou de droite à gauche ; orthographe conservée telle qu’imprimée.'
		},
		'text_extraction.newspaper': {
			name: 'Journal ou magazine',
			description:
				'Article par article à travers les colonnes, les mots coupés par les colonnes recomposés.'
		},
		'text_extraction.handwritten': {
			name: 'Document manuscrit',
			description:
				'Lettres, registres et manuscrits en toute écriture, latine, arabe et ajami comprises ; orthographe, signes diacritiques et sens de lecture tels qu’écrits.'
		},
		'audio_transcription.interview': {
			name: 'Entretien ou enregistrement, transcrit',
			description:
				'Mot à mot dans les langues parlées, avec l’identification des locuteurs et un horodatage à chaque prise de parole.'
		},
		'audio_transcription.translate_english': {
			name: 'Entretien ou enregistrement, traduit en anglais',
			description:
				'Tout ce qui est dit, en anglais, avec locuteurs et horodatage. Pour une autre langue, dupliquez-le et changez la langue cible.'
		},
		'analysis.summary_research': {
			name: 'Résumé et mots-clés',
			description:
				'Ce que dit chaque document en particulier, avec les noms, chiffres et dates, aussi long que le document l’exige, et des mots-clés pour la recherche.'
		},
		'analysis.summary_publication': {
			name: 'Publication scientifique, sciences sociales',
			description:
				'Question, argument, cadre théorique, méthodes, données, terrain et résultats d’un article, d’un chapitre ou d’un livre, champ par champ, pour un état de l’art.'
		},
		'analysis.entities': {
			name: 'Personnes, lieux et organisations',
			description:
				'Les personnes, organisations, lieux et sujets que nomme chaque document, orthographiés comme dans le texte.'
		},
		'analysis.tone': {
			name: 'Tonalité du document',
			description:
				'Si chaque document présente son sujet de façon positive ou négative, et à quel point il exprime des opinions, avec une justification pour chaque réponse.'
		},
		'contents.periodical': {
			name: 'Numéro de journal ou de magazine',
			description: 'Une entrée par article, avec sa rubrique, ses auteurs, ses pages et un résumé.'
		},
		'contents.archival_file': {
			name: 'Dossier d’archives, pièce à pièce',
			description:
				'L’inventaire d’un dossier numérisé d’un seul tenant : chaque lettre, rapport ou coupure de presse avec sa date, son expéditeur, son destinataire et un résumé.'
		},
		'contents.book': {
			name: 'Livre, rapport ou thèse',
			description: 'Parties, chapitres et sections avec leurs pages et un résumé de chacun.'
		},
		'text_correction.general': {
			name: 'Document imprimé ou journal',
			description:
				'Corrige seulement les erreurs de lecture automatique et remet les colonnes dans l’ordre ; l’orthographe, la formulation et le style de l’époque sont conservés.'
		},
		'dictionary.merges': {
			name: 'Dictionnaire : regroupements proposés',
			description:
				'Envoyé quand le dictionnaire d’un projet demande à un modèle de proposer des regroupements. Les champs de la réponse sont fixes : le dictionnaire les lit.'
		}
	} as Record<string, { name: string; description: string }>,
	app: {
		name: 'Kalamu'
	},
	shell: {
		jobs: 'Traitements',
		running: 'En cours',
		unfinished: 'Inachevés',
		toCheck: 'À vérifier',
		finished: 'Terminés',
		noJobs: 'Les traitements que vous lancez apparaissent ici.',
		newJobShortcut: 'N',
		offlineReady: 'Whisper prêt sur cet ordinateur',
		collapse: 'Masquer la liste des traitements',
		expand: 'Afficher la liste des traitements',
		sidebarShortcut: 'B',
		jobMenu: (name: string) => `Actions pour « ${name} »`,
		addToProject: 'Ajouter à un projet',
		noProjects: 'Aucun projet pour l’instant. Créez-en un dans Projets.',
		addedTo: (project: string) => `Ajouté au projet « ${project} ».`,
		pauseFirst: 'Mettez le traitement en pause ou annulez-le avant de le supprimer.',
		showProjects: 'Afficher les projets',
		hideProjects: 'Masquer les projets',
		dropOnProject: 'Déposez-le sur un projet pour l’y ajouter',
		dragHint: 'Faites-le glisser sur un projet de la liste pour l’y ajouter'
	},
	nav: {
		newJob: 'Nouveau traitement',
		projects: 'Projets',
		workflows: 'Chaînes de traitement',
		jobs: 'Traitements',
		prompts: 'Prompts',
		settings: 'Paramètres',
		glossary: 'Glossaire',
		/** Back to the screen left, in a masthead. */
		back: 'Retour'
	},
	task: {
		text_extraction: 'Extraire le texte',
		text_extractionHint: 'Documents imprimés et manuscrits : PDF, JPEG, PNG, TIFF',
		audio_transcription: 'Transcrire un enregistrement',
		audio_transcriptionHint:
			'Enregistrements et fichiers vidéo sur cet ordinateur : MP3, M4A, WAV, MP4',
		youtube: 'Transcrire une vidéo YouTube',
		youtubeHint: 'Vidéos YouTube publiques, à partir de leur lien. Rien n’est téléchargé',
		analysis: 'Analyser des textes',
		analysisHint:
			'Résumés, personnes et lieux, tonalité : vous choisissez ce qu’il faut relever. Textes, fichiers Word, PDF et résultats antérieurs',
		contents: 'Table des matières détaillée',
		contentsHint:
			'Numéros de journaux et de magazines, dossiers d’archives, livres : une entrée par article ou par pièce, avec ses pages',
		text_correction: 'Corriger un texte existant',
		text_correctionHint:
			'Texte issu d’une lecture automatique, d’une bibliothèque numérique ou d’un PDF, corrigé page par page : TXT, Word, PDF'
	},
	useText: {
		button: 'Utiliser ce texte dans un nouveau traitement',
		short: 'Utiliser ce texte',
		hint: 'Transmet le texte, tel que vous l’avez corrigé, à un nouveau traitement, sans relire les fichiers. Les corrections faites ensuite ne parviennent pas au nouveau traitement.',
		analyse: 'L’analyser',
		analyseHint: 'Personnes et lieux, un résumé, sa tonalité, ou vos propres champs.',
		correct: 'Le corriger',
		correctHint:
			'Un modèle corrige les erreurs de lecture en conservant l’orthographe et la formulation ; chaque modification est signalée.',
		workflow: 'Lui appliquer une chaîne de traitement',
		workflowHint: 'Plusieurs étapes successives, parmi vos chaînes ou à partir d’un exemple.'
	},
	units: {
		tab: 'Pièces',
		lede: 'Chaque article, lettre ou pièce, sous la forme de pages de ce document, à lire, analyser ou exporter séparément. La table des matières reste telle quelle.',
		make: 'Créer les pièces à partir des entrées',
		makeHint:
			'Une pièce par entrée, avec son titre, ses pages et ses champs. Les pièces sont ensuite à vous de corriger : corriger les entrées par la suite ne les modifie pas.',
		made: (n: number) => (n === 1 ? '1 pièce créée.' : `${n} pièces créées.`),
		unknownPages: (entry: number, pages: string) =>
			`L’entrée ${entry} indique ${pages}, que ce document ne comporte pas.`,
		unreadPages: (entry: number, pages: string) =>
			`Les pages de l’entrée ${entry} (« ${pages} ») n’ont pas pu être lues : indiquez les pages de la pièce.`,
		count: (n: number) => (n === 1 ? '1 pièce' : `${n} pièces`),
		selected: (n: number, total: number) =>
			n <= 1 ? `${n} sur ${total} sélectionnée` : `${n} sur ${total} sélectionnées`,
		select: (title: string) => `Sélectionner « ${title} »`,
		selectAll: 'Sélectionner toutes les pièces',
		untitled: 'Sans titre',
		noPages: 'Pas encore de pages',
		shares: (page: string, titles: string) => `${page} contient aussi ${titles}.`,
		precision:
			'Une pièce se compose de pages entières : quand deux pièces partagent une page, chacune la prend en entier, et un modèle qui lit l’une voit aussi l’autre.',
		title: 'Titre',
		pages: 'Pages',
		pagesHint:
			'Dans l’ordre de lecture, selon la numérotation de ce document : 1, 3-5, 8 (gauche).',
		halves: { left: 'gauche', right: 'droite' },
		edit: 'Corriger',
		editItem: (title: string) => `Corriger « ${title} »`,
		save: 'Enregistrer',
		cancel: 'Annuler',
		add: 'Ajouter une pièce',
		remove: 'Retirer',
		removeItem: (title: string) => `Retirer « ${title} »`,
		joinNext: 'Joindre à la suivante',
		joinHint:
			'Les pages de la pièce suivante sont ajoutées après celles-ci : un article et sa suite.',
		split: 'Scinder',
		splitHint:
			'Deux pièces qu’une entrée a prises pour une seule : la seconde commence à la page choisie.',
		splitAt: (page: string) => `${page} et suivantes`,
		extract: 'Extraire le texte',
		extractHint:
			'Un nouveau traitement lit seulement les pages des pièces sélectionnées, chaque pièce comme un fichier distinct.',
		analyse: 'Analyser',
		analyseHint:
			'Une nouvelle analyse lit les pages de chaque pièce sélectionnée comme un seul document.',
		export: 'Exporter',
		exportHint:
			'Les pages de chaque pièce sélectionnée en PDF, dans un dossier de votre choix, avec la liste des pièces en CSV et en JSON.',
		exported: (n: number) => (n === 1 ? '1 fichier écrit.' : `${n} fichiers écrits.`),
		pdfOnly:
			'Pour l’instant, seules les pièces d’un PDF peuvent être lues ou exportées séparément.',
		notCombined:
			'Les pièces sont créées à partir des entrées regroupées : menez d’abord le traitement à son terme.'
	},
	workflows: {
		title: 'Chaînes de traitement',
		lede: 'Des configurations exécutées l’une après l’autre sur les mêmes fichiers : extraire le texte, le vérifier, puis l’analyser.',
		newWorkflow: 'Nouvelle chaîne',
		import: 'Importer une chaîne',
		imported: (name: string) => `« ${name} » importée avec ses prompts.`,
		saved: 'Chaînes enregistrées',
		empty:
			'Aucune chaîne enregistrée pour l’instant. Partez d’un exemple ci-dessous, ou composez-en une à partir de vos configurations.',
		steps: (n: number) => (n === 1 ? '1 étape' : `${n} étapes`),
		version: (n: number) => `version ${n}`,
		runs: 'Exécutions',
		noRuns: 'Les chaînes que vous exécutez apparaissent ici, avec le traitement de chaque étape.',
		started: (when: string) => `Lancée le ${when}`,
		back: 'Toutes les chaînes',
		name: 'Nom de la chaîne',
		stepsTitle: 'Étapes',
		noSteps:
			'Ajoutez une première étape : un prompt, tel que Kalamu le fournit ou tel que vous l’avez écrit, ou l’une de vos configurations enregistrées.',
		addStep: 'Ajouter une étape',
		addStepHint: 'Choisir un prompt ou une configuration',
		fromPipelines: 'Vos configurations',
		fromPrompts: (task: string) => `Prompts · ${task}`,
		prompt: 'Prompt',
		promptMissing: 'Un prompt qui n’est plus proposé',
		saveNeedsStep: 'Ajoutez une étape pour enregistrer la chaîne',
		stepNumber: (n: number) => `Étape ${n}`,
		reads: 'Porte sur',
		readsFiles: 'Les fichiers ou le texte sur lesquels vous l’exécutez',
		readsStep: (n: number, name: string) =>
			`Le texte de l’étape ${n} (${name}), tel que vous l’avez corrigé`,
		review: 'Me laisser vérifier les résultats avant de continuer',
		reviewHint:
			'Kalamu attend que vous les ayez vérifiés et corrigés en relecture, et reprend quand vous cliquez sur Continuer.',
		moveUp: (n: number) => `Monter l’étape ${n}`,
		moveDown: (n: number) => `Descendre l’étape ${n}`,
		remove: (n: number) => `Retirer l’étape ${n}`,
		save: 'Enregistrer',
		savedAs: (name: string, version: number) => `« ${name} » enregistrée, version ${version}.`,
		savedNote:
			'Enregistrer crée une nouvelle version. Les exécutions déjà lancées gardent les étapes et les prompts de leur départ.',
		duplicate: 'Dupliquer',
		copyName: (name: string) => `${name} (copie)`,
		export: 'Exporter',
		exported: (file: string) =>
			`Écrit dans ${file}, avec une notice Markdown à côté. Aucune clé, aucun fichier ni dossier n’y figure.`,
		delete: 'Supprimer',
		deleteConfirm: (name: string) =>
			`Supprimer la chaîne « ${name} » ? Ses exécutions et leurs traitements sont conservés.`,
		run: 'Exécuter sur des fichiers',
		runTitle: 'Exécuter cette chaîne',
		chooseFiles: 'Choisir des fichiers',
		files: (n: number) => (n === 1 ? '1 fichier' : `${n} fichiers`),
		runName: 'Nom de l’exécution',
		unsaved: 'Enregistrez la chaîne avant de l’exécuter.',
		sends: 'Ce que chaque étape envoie',
		sendsFiles: (provider: string) => `les fichiers, à ${provider}`,
		sendsText: (n: number, provider: string) => `le texte de l’étape ${n}, à ${provider}`,
		offline: 'reste sur cet ordinateur',
		costKnown: (usd: string) => `environ ${usd}`,
		costLater: (n: number) => `connu une fois les fichiers lus par l’étape ${n}`,
		costUnknown: 'coût inconnu',
		cloudAfterOffline:
			'Une étape lit sur cet ordinateur, et une étape suivante envoie son texte à un fournisseur en ligne. Retirez cette dernière si les documents doivent rester sur cet ordinateur.',
		start: 'Lancer',
		fromProject: (name: string) => `Ses traitements rejoignent le projet « ${name} ».`,
		onText: (files: string) =>
			`S’exécute sur le texte déjà lu, tel que vous l’avez corrigé : ${files}. Choisissez une chaîne qui commence par une correction ou une analyse.`,
		notOnText: 'part de numérisations ou d’enregistrements',
		runOf: (name: string, version: number) => (version > 0 ? `${name}, version ${version}` : name),
		diagram: {
			title: 'Comment les étapes s’enchaînent',
			replay: 'Rejouer',
			files: 'Vos fichiers',
			check: 'Vous vérifiez',
			checkNote: 'puis cliquez sur Continuer',
			written: 'Résultats enregistrés'
		},
		examplesTitle: 'Partir d’un exemple',
		useExample: 'Utiliser cet exemple',
		examplesHint:
			'Chaque exemple utilise les modèles pour lesquels vous avez une clé. Ouvrez-en un, ajustez-le si vous le souhaitez, enregistrez-le, puis exécutez-le sur vos fichiers.',
		examples: {
			letters: {
				name: 'Lettres : lire, vérifier, puis personnes et lieux',
				about: 'Lettres ou rapports numérisés, dont le texte est vérifié avant le relevé des noms.',
				steps: ['Lire le texte', 'Personnes, lieux et organisations']
			},
			interviews: {
				name: 'Entretiens : transcrire, vérifier, puis résumer',
				about:
					'Enregistrements transcrits, transcription corrigée, puis résumée pour la recherche.',
				steps: ['Transcrire l’entretien', 'Résumé et mots-clés']
			},
			library: {
				name: 'Texte de bibliothèque numérique : corriger, puis résumer',
				about:
					'Texte issu de la lecture automatique d’une bibliothèque numérique, corrigé, puis résumé.',
				steps: ['Corriger le texte', 'Résumé et mots-clés']
			}
		},
		stepState: {
			waiting: 'En attente',
			running: 'En cours',
			paused: 'Non lancée',
			review: 'Prête à vérifier',
			attention: 'Pages non lues',
			done: 'Terminée',
			done_with_errors: 'Terminée avec des erreurs',
			cancelled: 'Annulée',
			blocked: 'Ne peut pas démarrer',
			failed: 'N’a pas pu démarrer'
		},
		runStatus: {
			running: 'En cours',
			needs_you: 'Vous attend',
			finished: 'Terminée',
			stopped: 'Arrêtée'
		},
		waitingFor: (n: number) => `Démarre quand l’étape ${n} est terminée.`,
		reviewNotice:
			'Relisez et corrigez ses résultats, puis continuez : les étapes suivantes lisent le texte tel que vous le laissez.',
		attentionNotice:
			'Certains segments n’ont pas pu être lus. Relancez-les depuis la page du traitement, ou continuez avec ce qui a été lu : les segments laissés de côté sont indiqués dans la provenance.',
		pausedNotice: 'Son traitement n’est pas en cours. Ouvrez-le pour le lancer ou le reprendre.',
		blockedNotice: 'L’étape qu’elle lit a été annulée, supprimée ou n’a pas pu démarrer.',
		failedNotice: (reason: string) => `Elle n’a pas pu démarrer : ${reason}`,
		continue: 'Continuer',
		continueAnyway: 'Continuer avec ce qui a été lu',
		openJob: 'Ouvrir le traitement',
		openReview: 'Relire',
		approved: (when: string) => `Poursuivie le ${when}`,
		approvedPartial: (when: string) => `Poursuivie le ${when} avec les segments lus`,
		cancel: 'Arrêter l’exécution',
		cancelConfirm:
			'Arrêter cette exécution ? Les étapes en cours sont annulées et aucune autre ne démarre. Ce qui a été lu est conservé.',
		forget: 'Retirer de cette liste',
		forgetConfirm:
			'Retirer cette exécution de la liste ? Ses traitements et leurs résultats sont conservés.',
		banner: {
			step: (n: number, of: number, run: string) => `Étape ${n} sur ${of} de « ${run} ».`,
			review: (next: string) =>
				`Quand vous aurez vérifié et corrigé le texte, cliquez sur Continuer : ${next} le lira alors tel que vous l’aurez laissé.`,
			attention: (next: string) =>
				`Certaines pages n’ont pas pu être lues. Relancez-les, ou continuez : ${next} lira les pages qui l’ont été.`,
			next: (next: string) => `Ensuite : ${next}, une fois ce traitement terminé.`,
			nextAfterCheck: (next: string) => `Ensuite : ${next}, une fois le texte vérifié.`,
			see: 'Voir la chaîne'
		},
		notify: {
			review: 'prêt à vérifier, puis cliquez sur Continuer',
			attention: 'certaines pages n’ont pas été lues ; relancez-les ou continuez'
		}
	},
	dictionary: {
		title: 'Dictionnaire',
		lede: 'Les noms et les termes relevés par vos analyses, champ par champ. Rassemblez les graphies d’une même personne, d’un même lieu ou d’un même terme en une entrée, sous le libellé que vous retenez. Les réponses des analyses restent telles que les modèles les ont données.',
		coverage: (read: number, total: number) =>
			`Relevés dans les analyses de ${read} ${read <= 1 ? 'document' : 'documents'} sur ${total}.`,
		empty:
			'Aucune analyse de ces documents ne relève encore de noms. Lancez sur les documents du projet une analyse comportant un champ de type liste, comme « Personnes, lieux et organisations ».',
		field: 'Champ',
		fieldCount: (label: string, rows: number, toReview: number) =>
			`${label} (${rows}${toReview > 0 ? `, ${toReview} à vérifier` : ''})`,
		scope: 'Documents',
		wholeProject: 'Tout le projet',
		search: 'Chercher un nom',
		show: 'Afficher',
		filter: {
			all: 'Tout',
			to_review: 'À vérifier',
			similar: 'Variantes possibles',
			reviewed: 'Vérifiés',
			unseen: 'Plus relevés'
		},
		noMatch: 'Aucun résultat.',
		columnLabel: 'Entrée',
		columnDocuments: 'Documents',
		columnAgreement: 'Relevé par',
		columnStatus: 'Statut',
		agreement: (listed: number, read: number) =>
			`${listed} ${listed <= 1 ? 'résultat' : 'résultats'} sur ${read}`,
		agreementHint:
			'Combien des résultats qui ont lu ces documents l’ont relevé. Deux modèles qui relèvent un nom, c’est un accord, pas une seconde mention.',
		reviewed: 'Vérifiée',
		toReview: 'À vérifier',
		unseen: 'Plus relevée',
		unseenHint:
			'Aucune de ses formes n’est plus relevée par les analyses : un traitement a été supprimé, ou ses réponses corrigées. L’entrée est conservée.',
		similar: 'Peut-être identique à',
		similarHint:
			'Graphies semblables une fois écartés les majuscules, les accents, la ponctuation et l’ordre des mots. Vous seul décidez s’il s’agit d’une même entrée.',
		selectSimilar: 'Les sélectionner ensemble',
		apart: 'Distincte de',
		withdraw: (label: string) => `Ne plus garder distincte de ${label}`,
		selected: (n: number) => (n <= 1 ? `${n} sélectionnée` : `${n} sélectionnées`),
		clearSelection: 'Désélectionner',
		selectAll: 'Sélectionner tout ce qui est affiché',
		select: (label: string) => `Sélectionner ${label}`,
		merge: 'Fusionner en une entrée',
		mergeLabel: 'Libellé de l’entrée',
		mergeConfirm: 'Fusionner',
		markReviewed: 'Marquer comme vérifiée',
		markToReview: 'Marquer à vérifier',
		keepApart: 'Garder distinctes',
		keepApartHint: 'Deux personnes, lieux ou termes différents, si semblables qu’ils paraissent.',
		details: (label: string) => `Détails de ${label}`,
		forms: 'Formes relevées',
		formDocuments: (n: number) => (n === 1 ? '1 document' : `${n} documents`),
		takeOut: (form: string) => `Retirer « ${form} » de cette entrée`,
		rename: 'Renommer',
		label: 'Libellé',
		save: 'Enregistrer',
		cancel: 'Annuler',
		evidence: 'Où elle a été relevée',
		noEvidence: 'Aucun résultat ne la relève plus.',
		openReview: 'Ouvrir la relecture',
		export: 'Exporter',
		exported:
			'Le dictionnaire a été enregistré en JSON, avec à côté un CSV de toutes les formes relevées.',
		fileName: (project: string) => `Dictionnaire ${project}`,
		proposals: {
			title: 'Regroupements proposés',
			ask: 'Demander à un modèle de proposer des regroupements',
			plan: (field: string, rows: number, groups: number) =>
				groups === 1
					? `Les ${rows} lignes de ${field} sont envoyées en une seule requête, avec leurs graphies, les titres de trois documents au plus qui nomment chacune, et les paires que vous avez gardées distinctes.`
					: `Les ${rows} lignes de ${field} sont trop nombreuses pour une seule requête : elles sont envoyées en ${groups} requêtes d’environ 100 000 tokens chacune, par ordre alphabétique sans tenir compte des accents, chacune partageant un dixième de ses lignes avec la suivante. Les noms éloignés dans l’alphabet ne sont pas comparés. Chaque ligne part avec ses graphies, les titres de trois documents au plus qui la nomment, et les paires que vous avez gardées distinctes.`,
			prompt: 'Voir ou modifier le prompt',
			sends: (provider: string) =>
				`Envoyé à ${provider} avec votre clé, sous la forme d’un traitement d’analyse à part : son coût et ses réponses figurent dans Traitements. Rien n’est fusionné avant que vous l’acceptiez.`,
			noKey: (provider: string) => `Ajoutez d’abord une clé pour ${provider} dans Paramètres.`,
			economy: 'Mode économique : moitié prix, réponse dans la journée',
			cost: (usd: string) => `Environ ${usd}.`,
			costUnknown: 'Le coût n’a pas pu être estimé.',
			start: 'Envoyer',
			jobName: (field: string) => `Dictionnaire : regroupements dans ${field}`,
			request: (status: string, answered: number, groups: number, open: number, date: string) =>
				`Demandé le ${date} : ${status.toLowerCase()}, ${answered} ${answered <= 1 ? 'groupe traité' : 'groupes traités'} sur ${groups}, ${open <= 1 ? `${open} proposition en attente` : `${open} propositions en attente`}.`,
			unknown: (n: number) =>
				n === 1
					? '1 renvoi à une ligne inconnue, écarté'
					: `${n} renvois à des lignes inconnues, écartés`,
			openJob: 'Ouvrir le traitement',
			forget: 'Oublier',
			forgetHint:
				'Oublier cette demande et ses propositions. Le traitement reste dans Traitements.',
			none: 'Plus aucune proposition : chacune a été fusionnée, écartée ou gardée distincte, ou le modèle n’en a trouvé aucune.',
			certainty: {
				certain: 'Certain',
				likely: 'Probable',
				possible: 'Possible'
			},
			certaintyHint:
				'Le degré de certitude que le modèle déclare, avec ses propres mots. Ce n’est pas une probabilité.',
			dismiss: 'Écarter',
			dismissedCount: (n: number) => (n <= 1 ? `${n} écartée` : `${n} écartées`),
			takeBack: 'Rétablir'
		}
	},
	search: {
		title: 'Recherche',
		lede: 'Chaque passage des documents du projet qui contient vos mots, dans le texte tel que vous l’avez corrigé, chacun s’ouvrant là où il se trouve. Les majuscules, les accents et les signes vocaliques de l’arabe sont ignorés. Rien n’est envoyé : la recherche se fait sur cet ordinateur.',
		query: 'Mots à chercher',
		placeholder: 'Un nom, une organisation, une expression…',
		go: 'Chercher',
		tooShort: 'Tapez au moins trois lettres.',
		mode: 'Trouver',
		modes: {
			phrase: 'L’expression exacte',
			all_words: 'Tous ces mots',
			near: 'Ces mots proches les uns des autres'
		},
		near: 'Au plus',
		nearWords: 'mots d’écart',
		quotesHint:
			'Chaque mot est cherché dans la même page ou la même réponse. Mettez entre guillemets les mots qui vont ensemble : « mission catholique » Koupéla.',
		ignored: (words: string) => `Laissés de côté, trop courts pour être cherchés : ${words}.`,
		preparing: (done: number, total: number) =>
			`Préparation du texte pour la recherche : ${done} sur ${total <= 1 ? `${total} page ou partie` : `${total} pages ou parties`}…`,
		scope: 'Documents',
		wholeProject: 'Tout le projet',
		tag: 'Étiquette',
		anyTag: 'Toute étiquette',
		task: 'Résultats de',
		anyTask: 'Tous les traitements',
		review: 'Pages',
		reviewFilter: { any: 'Toutes', reviewed: 'Relues', not_reviewed: 'Pas encore relues' },
		results: 'Résultats',
		resultChoice: { preferred: 'Le résultat retenu, s’il y en a un', all: 'Tous les résultats' },
		generated: 'Chercher aussi ce que les modèles en ont écrit : analyses et tables des matières',
		searching:
			'Recherche en cours… La première recherche d’un projet prépare son texte, ce qui peut prendre un moment.',
		matches: (matches: number, documents: number) =>
			`${matches <= 1 ? `${matches} occurrence` : `${matches} occurrences`} dans ${documents <= 1 ? `${documents} document` : `${documents} documents`}`,
		listedOnly: (listed: number, matches: number) =>
			`Seules les ${listed} premières occurrences sur ${matches} sont listées. Restreignez la recherche pour voir les autres.`,
		none: 'Introuvable dans le texte cherché.',
		coverage: (searched: number, documents: number) =>
			`Texte cherché dans ${searched} ${searched <= 1 ? 'document' : 'documents'} sur ${documents}.`,
		unread: (n: number) =>
			n === 1
				? 'Rien n’est encore lu dans 1 document pour les types cherchés : son contenu n’est pas cherché.'
				: `Rien n’est encore lu dans ${n} documents pour les types cherchés : leur contenu n’est pas cherché.`,
		piecesUnread: (n: number) =>
			n === 1
				? '1 page ou partie des résultats cherchés reste à lire, ou n’a pas pu être lue.'
				: `${n} pages ou parties des résultats cherchés restent à lire, ou n’ont pas pu être lues.`,
		withoutPreference: (n: number) =>
			n === 1
				? '1 document a plusieurs résultats sans résultat retenu : tous ont été cherchés.'
				: `${n} documents ont plusieurs résultats sans résultat retenu : tous ont été cherchés.`,
		absence:
			'Ne rien trouver ne signifie pas que les documents n’en parlent jamais : seul le texte lu et conservé est cherché, et un modèle peut avoir mal lu un nom.',
		preferred: 'Retenu',
		generatedResult: 'Écrit par un modèle',
		stale: 'Périmée',
		staleHint: 'Le texte lu par cette analyse a été corrigé depuis.',
		reason: 'justification',
		open: (place: string) => `Ouvrir ${place} en relecture`,
		rebuild: 'Préparer à nouveau le texte',
		rebuildHint:
			'La recherche conserve une copie du texte de vos résultats, préparée pour la recherche, dans le dossier de données de Kalamu. La préparer à nouveau repart de vos résultats et de vos corrections ; rien n’est perdu.',
		rebuilt: 'Le texte sera préparé à nouveau à la prochaine recherche.',
		moved:
			'Les mots ne sont plus là où la recherche les a trouvés : le texte a été corrigé depuis. La recherche de la relecture montre où ils se trouvent maintenant.'
	},
	queue: {
		title: 'À vérifier',
		lede: 'Les pages et les réponses de ces documents qui demandent votre attention, chacune avec son motif. La liste est tirée de ce que Kalamu conserve déjà : rien n’est envoyé à un modèle. Vous seul marquez une page comme relue ; l’accord des modèles ne vaut pas relecture.',
		scope: 'Documents',
		wholeProject: 'Tout le projet',
		selection: (n: number) =>
			n === 1 ? 'Le document sélectionné' : `Les ${n} documents sélectionnés`,
		checkSelection: 'Ce qu’il faut vérifier',
		loading: 'Recherche de ce qui est à vérifier…',
		coverage: {
			documents: (read: number, total: number) =>
				`${read} ${read <= 1 ? 'document lu' : 'documents lus'} sur ${total}`,
			pieces: (processed: number, pieces: number) =>
				`${processed} ${processed <= 1 ? 'page ou partie lue' : 'pages ou parties lues'} sur ${pieces}`,
			unprocessed: (n: number) =>
				n <= 1 ? `${n} en échec ou pas encore lue` : `${n} en échec ou pas encore lues`,
			reviewed: (reviewed: number, processed: number) =>
				`${reviewed} ${reviewed <= 1 ? 'relue' : 'relues'} sur ${processed}`,
			flagged: (flagged: number, unresolved: number) =>
				`${flagged} ${flagged <= 1 ? 'signalée' : 'signalées'}, ${unresolved} encore en suspens`
		},
		coverageHint:
			'Compté sur tous les résultats lus ci-dessous, quel que soit l’affichage. Seules les pages relues sont vérifiées ; les pages signalées ne sont pas un échantillon de l’ensemble.',
		results: 'Résultats lus',
		resultsHint:
			'Là où vous avez retenu un résultat pour un document, seul celui-ci est lu. Ailleurs, chaque résultat est lu et listé ici : écartez ceux que vous ne voulez pas vérifier. Un document lu par plusieurs modèles reste un seul document.',
		preferred: (n: number) =>
			n === 1
				? '1 document lu à travers son résultat retenu'
				: `${n} documents lus à travers leur résultat retenu`,
		resultChoice: (label: string, job: string, documents: number) =>
			`${label}, ${job} (${documents === 1 ? '1 document' : `${documents} documents`})`,
		show: 'Afficher',
		reason: {
			failed: 'En échec',
			not_read: 'Non lu',
			truncated: 'Tronqué',
			disagrees: 'Les modèles divergent',
			stale: 'Source modifiée',
			not_reviewed: 'Non relu'
		},
		showResolved: 'Inclure les pages relues',
		processing: 'En échec ou non lu',
		processingHint:
			'Rien n’a été lu ici, il n’y a donc encore rien à vérifier : relancez les pages en échec sur la page du traitement, ou lancez le traitement.',
		answers: 'Réponses à vérifier',
		none: 'Rien ne demande votre attention pour les motifs affichés.',
		empty:
			'Aucun document ici n’a encore été lu. Lisez-en avec un nouveau traitement, puis revenez.',
		detail: {
			failed: (error: string | null) => (error ? `En échec : ${error}` : 'En échec'),
			notRead: 'Pas encore lu',
			neverStarted: 'Pas encore lancé',
			truncated: 'Arrêté à la longueur maximale du modèle : la fin manque peut-être',
			disagreesWords: (labels: string, words: number) =>
				`Lit autrement que ${labels} : ${words <= 1 ? `${words} mot` : `${words} mots`}`,
			disagreesFields: (labels: string, fields: string) =>
				`Répond autrement que ${labels} : ${fields}`,
			stale: 'A lu un texte corrigé depuis',
			gone: 'A lu un texte dont le traitement a été supprimé depuis',
			unitChanged: 'A lu une pièce dont les pages ont changé depuis',
			unitGone: 'A lu une pièce supprimée depuis',
			notReviewed: 'Pas encore relu'
		},
		wholeResult: 'Le résultat entier',
		preferredBadge: 'Retenu',
		reviewedBadge: 'Relu',
		answered: (when: string) => `Réponse du ${when}`,
		answeredHint:
			'La date d’enregistrement de la réponse. Une nouvelle réponse est à relire de nouveau.',
		open: (document: string, place: string) => `Ouvrir ${document}, ${place}, en relecture`,
		more: (n: number) => `Afficher ${n} de plus`,
		banner: {
			position: (at: number, total: number) => `${at} sur ${total} à vérifier`,
			previous: 'Précédent à vérifier',
			next: 'Suivant à vérifier',
			back: 'Retour à la liste'
		}
	},
	projects: {
		title: 'Projets',
		lede: 'Vos documents, regroupés comme vous l’entendez, avec tout ce qui en a été lu.',
		newProject: 'Nouveau projet',
		name: 'Nom',
		description: 'Description',
		descriptionHint: 'L’objet du projet, pour vous et pour vos collègues.',
		create: 'Créer',
		cancel: 'Annuler',
		save: 'Enregistrer',
		empty:
			'Aucun projet pour l’instant. Un projet rassemble vos documents, en collections, avec chaque transcription, extraction et analyse qui en a été faite.',
		showArchived: 'Afficher les projets archivés',
		archived: 'Archivé',
		counts: (documents: number, collections: number, jobs: number) =>
			`${documents === 1 ? '1 document' : `${documents} documents`} · ${
				collections === 1 ? '1 collection' : `${collections} collections`
			} · ${jobs === 1 ? '1 traitement' : `${jobs} traitements`}`,
		updated: (date: string) => `Modifié le ${date}`,
		back: 'Projets',
		edit: 'Modifier',
		archive: 'Archiver',
		restore: 'Restaurer',
		archivedNotice:
			'Ce projet est archivé. Rien n’y a changé ; restaurez-le pour qu’il figure de nouveau dans la liste.',
		delete: 'Supprimer le projet',
		deleteConfirm: (name: string) =>
			`Supprimer le projet « ${name} » ? Ses collections et ses documents sont retirés de Kalamu. Les fichiers sur votre ordinateur sont conservés, et les traitements et leurs résultats restent dans Traitements.`,
		allDocuments: 'Tous les documents',
		collections: 'Collections',
		newCollection: 'Nouvelle collection',
		collectionName: 'Nom de la collection',
		renameCollection: 'Renommer',
		deleteCollection: 'Supprimer la collection',
		deleteCollectionConfirm: (name: string) =>
			`Supprimer la collection « ${name} » ? Ses documents restent dans le projet.`,
		activity: 'Activité',
		activityLede:
			'Les traitements qui ont lu les documents de ce projet, du plus récent au plus ancien.',
		noActivity:
			'Aucun traitement n’a encore lu ces documents. Sélectionnez des documents et choisissez « Nouveau traitement avec ceux-ci ».',
		activityFilters: 'Filtrer les traitements',
		filterBy: {
			task: 'Tâche',
			status: 'État',
			model: 'Modèle',
			workflow: 'Chaîne',
			period: 'Lancé'
		},
		anyTask: 'Toute tâche',
		anyStatus: 'Tout état',
		anyModel: 'Tout modèle',
		anyWorkflow: 'Dans une chaîne ou non',
		period: {
			any: 'Toute période',
			week: '7 derniers jours',
			month: '30 derniers jours',
			year: 'Dernière année'
		},
		clearFilters: 'Tout afficher',
		noActivityMatch: 'Aucun traitement ne correspond à ces filtres.',
		activityShown: (shown: number, total: number) =>
			shown === total
				? total === 1
					? '1 traitement'
					: `${total} traitements`
				: `${shown} ${shown <= 1 ? 'traitement' : 'traitements'} sur ${total}`,
		activityCost: (cost: string) => `${cost} dépensés`,
		activityUnpriced: (n: number) => `${n} sans prix (hors ligne ou pas encore de réponse)`,
		addFiles: 'Ajouter des fichiers',
		addFolder: 'Ajouter un dossier',
		addedFiles: (n: number) => (n === 1 ? '1 document ajouté' : `${n} documents ajoutés`),
		dropHere: 'Déposez-les ici pour les ajouter au projet',
		removedFromCollection: (n: number, collection: string) =>
			`${n === 1 ? '1 document retiré' : `${n} documents retirés`} de « ${collection} »`,
		notReadable: (n: number) =>
			n === 1
				? '1 fichier n’a pas pu être lu et a été laissé de côté.'
				: `${n} fichiers n’ont pas pu être lus et ont été laissés de côté.`,
		attachJob: 'Ajouter un traitement existant',
		noDocuments:
			'Pas encore de documents ici. Ajoutez des fichiers, ou un traitement lancé auparavant : ses résultats l’accompagnent, sans être payés une seconde fois.',
		search: 'Chercher un document',
		noMatch: 'Aucun document ne correspond.',
		sortBy: 'Trier par',
		sortTitle: 'Titre',
		sortAdded: 'Date d’ajout',
		sortRun: 'Dernière lecture',
		columnTitle: 'Document',
		columnKind: 'Type',
		columnCollections: 'Collections et étiquettes',
		anyTag: 'Toute étiquette',
		tagFilter: 'Étiquette',
		tagCount: (tag: string, n: number) => `${tag} (${n})`,
		addTag: 'Ajouter une étiquette',
		addTagHint: 'Une nouvelle étiquette, ou une déjà utilisée',
		tagButton: 'Étiqueter',
		removeTag: 'Retirer une étiquette…',
		removeTagFrom: (tag: string) => `Retirer l’étiquette ${tag}`,
		renameTag: 'Renommer l’étiquette',
		renameTagTo: (tag: string) => `Nouveau nom pour « ${tag} »`,
		tagRenamed: (n: number) =>
			n <= 1 ? `Renommée sur ${n} document.` : `Renommée sur ${n} documents.`,
		describe: {
			title: 'Description de la source',
			lede: 'L’original tel qu’un fonds d’archives ou une bibliothèque le décrit, comme vous le citeriez. C’est vous qui la rédigez, jamais un modèle, et aucun résultat ne la modifie. Un champ laissé vide reprend celui du projet.',
			projectTitle: 'Commune à ses documents',
			projectLede:
				'Les éléments de description que partagent tous les documents, comme le dépôt ou le fonds. La valeur propre à un document les remplace.',
			fields: {
				repository: 'Dépôt',
				fonds: 'Fonds ou collection',
				series: 'Série',
				file: 'Carton ou dossier',
				shelfmark: 'Cote',
				creator: 'Auteur ou producteur',
				date: 'Date',
				url: 'Adresse web',
				citation: 'Référence à citer',
				rights: 'Droits'
			},
			fromProject: 'Repris du projet',
			save: 'Enregistrer la description',
			saved: 'Description enregistrée.',
			export: 'Exporter les descriptions',
			exported: 'Les descriptions ont été enregistrées en JSON, avec un CSV à côté.'
		},
		profiles: {
			title: 'Exporter les descriptions',
			fileName: (project: string) => `Descriptions ${project}`,
			lede: 'L’export propre à Kalamu donne chaque élément de chaque description en indiquant d’où il vient. Un profil écrit les colonnes que vous choisissez, dans votre ordre et sous vos intitulés, pour un tableur ou l’import d’un autre outil. Il sert à tous les projets et ne garde aucune valeur de celui-ci.',
			columns: 'Colonnes',
			kalamu: 'Tous les éléments, en indiquant d’où ils viennent',
			newProfile: 'Nouveau profil',
			starters: {
				every: 'À partir de toutes les colonnes',
				dublin_core: 'À partir des termes Dublin Core'
			},
			import: 'Importer un profil…',
			imported: (name: string) => `Profil « ${name} » importé.`,
			edit: 'Modifier',
			duplicate: 'Dupliquer',
			copyName: (name: string) => `${name} (copie)`,
			share: 'Partager…',
			shareFixed:
				'Inclure les textes fixes ? Ils peuvent être propres à ce projet. Sans eux, les colonnes restent et leurs textes sont laissés vides.',
			shared: 'Le profil a été enregistré dans un fichier qu’un autre Kalamu peut importer.',
			delete: 'Supprimer',
			deleteConfirm: (name: string) =>
				`Supprimer le profil « ${name} » ? Les fichiers déjà exportés restent tels quels.`,
			name: 'Nom',
			heading: 'Intitulé',
			value: 'Valeur',
			fixedText: 'Texte de chaque ligne',
			groups: {
				document: 'Le document',
				parts: 'Sa description',
				from: 'D’où vient chaque élément',
				result: 'Son résultat pour une tâche',
				other: 'Autre'
			},
			tasks: {
				text_extraction: 'Extraction du texte',
				text_correction: 'Correction du texte',
				audio_transcription: 'Transcription',
				analysis: 'Analyse',
				contents: 'Table des matières'
			},
			resultValues: {
				status: (task: string) => `${task} : où en est la relecture`,
				model: (task: string) => `${task} : modèle`,
				text: (task: string) => `${task} : texte, avec les corrections`
			},
			answer: 'Analyse : une réponse',
			labelled: 'Analyse : une réponse, avec les libellés du dictionnaire',
			answerKey: 'Champ',
			resultHint:
				'Une colonne de résultat lit le résultat préféré du document pour la tâche, ou le seul qu’il a. S’il en a plusieurs et qu’aucun n’est préféré, la cellule reste vide : choisissez-en un sur la page du document. Avec les libellés du dictionnaire, chaque nom rassemblé dans une entrée est écrit sous son libellé, et un nom qu’aucune entrée ne rassemble tel que le modèle l’a donné.',
			values: {
				document_id: 'Identifiant du document',
				title: 'Titre',
				file_name: 'Nom du fichier',
				path: 'Chemin',
				project: 'Nom du projet',
				collections: 'Collections',
				tags: 'Étiquettes',
				fixed: 'Un texte fixe'
			},
			fromPart: (part: string) => `${part} : d’où il vient`,
			moveUp: (n: number) => `Monter la colonne ${n}`,
			moveDown: (n: number) => `Descendre la colonne ${n}`,
			remove: (n: number) => `Retirer la colonne ${n}`,
			addColumn: 'Ajouter une colonne',
			separator: 'Entre les éléments d’une liste',
			separatorHint: 'Dans une cellule CSV ; le JSON garde les listes telles quelles.',
			missing: 'Texte d’une cellule vide',
			missingHint: 'Rien si le champ reste vide ; le JSON écrit null.',
			joinLines: 'Remplacer les retours à la ligne d’une valeur par une espace',
			problems: {
				no_name: 'Donnez un nom au profil.',
				no_columns: 'Ajoutez au moins une colonne.',
				unnamed: (column: number) => `La colonne ${column} a besoin d’un intitulé.`,
				bad_heading: (column: number) =>
					`L’intitulé de la colonne ${column} doit tenir sur une ligne.`,
				duplicate: (column: number, first: number) =>
					`La colonne ${column} a le même intitulé que la colonne ${first} ; les majuscules ne les distinguent pas.`,
				bad_separator: 'Le séparateur de liste ne peut être vide ni contenir de retour à la ligne.',
				bad_value: (column: number) =>
					`La colonne ${column} demande ce que Kalamu ne peut pas donner : le texte d’une tâche qui en écrit un, ou une réponse désignée par son champ.`
			},
			save: 'Enregistrer le profil',
			saved: 'Profil enregistré.',
			cancel: 'Annuler',
			export: 'Exporter…',
			close: 'Fermer'
		},
		columnResults: 'Résultats',
		columnDate: 'Ajouté',
		selectAll: 'Sélectionner tous les documents affichés',
		select: (title: string) => `Sélectionner ${title}`,
		selected: (n: number) => (n <= 1 ? `${n} sélectionné` : `${n} sélectionnés`),
		clearSelection: 'Désélectionner',
		addTo: 'Ajouter à une collection…',
		removeFromCollection: 'Retirer de cette collection',
		newJobWith: 'Nouveau traitement avec ceux-ci',
		workflowWith: 'Exécuter une chaîne',
		removeDocuments: 'Retirer du projet',
		removeDocumentsConfirm: (n: number) =>
			`Retirer ${n === 1 ? 'ce document' : `ces ${n} documents`} du projet ? Les fichiers, les traitements qui les ont lus et leurs résultats sont conservés.`,
		missing: 'Fichier introuvable',
		missingHint:
			'Le fichier n’est plus là où il a été ajouté. Ses résultats sont conservés ; remettez-le en place pour le relire.',
		sameContent: (n: number) =>
			n === 1 ? 'Même contenu qu’un autre' : `Même contenu que ${n} autres`,
		noResults: 'Pas encore lu',
		kind: {
			pdf: 'PDF',
			image: 'Image',
			audio: 'Enregistrement',
			video: 'Vidéo',
			text: 'Texte',
			youtube: 'YouTube'
		},
		taskResults: (task: string, n: number, reviewed: number, preferred: boolean) =>
			`${task} : ${n === 1 ? '1 résultat' : `${n} résultats`}${reviewed > 0 ? `, ${reviewed} relu${reviewed > 1 ? 's' : ''}` : ''}${preferred ? (n === 1 ? ', retenu' : ', dont un retenu') : ''}`,
		resultCount: (n: number) => (n === 1 ? '1 résultat' : `${n} résultats`),
		reviewedMark: (n: number) => (n === 1 ? '1 relu' : `${n} relus`),
		preferredMark: 'Retenu : le résultat sur lequel s’appuyer pour ce document',
		fromProject: (name: string) =>
			`Lancé depuis le projet « ${name} » : les fichiers du traitement deviennent ses documents, et ses résultats les leurs.`,
		linkFailed: (name: string) =>
			`Le traitement a démarré, mais n’a pas pu être ajouté au projet « ${name} ». Ajoutez-le depuis le projet avec « Ajouter un traitement existant ».`,
		attach: {
			title: 'Ajouter un traitement existant',
			lede: 'Ses résultats rejoignent les documents dont ils ont été tirés. Rien n’est envoyé de nouveau.',
			job: 'Traitement',
			chooseJob: 'Choisir un traitement',
			file: 'Fichier',
			document: 'Document',
			newDocument: 'Un nouveau document',
			skip: 'Laisser de côté',
			proposed: (title: string) => `${title} (le même fichier)`,
			sameContent: (title: string) => `${title} (même contenu, autre fichier)`,
			already: (title: string) => `${title} (déjà ajouté)`,
			sameContentHint:
				'Des fichiers au contenu identique peuvent être des copies d’une même pièce, ou deux pièces qui se trouvent identiques. Choisissez pour chacun.',
			confirm: 'Ajouter',
			attached: (n: number) =>
				n === 1 ? '1 fichier ajouté au projet' : `${n} fichiers ajoutés au projet`
		},
		document: {
			back: (project: string) => project,
			source: 'Original',
			reveal: 'Afficher dans le dossier',
			collections: 'Collections',
			notInCollection: 'Dans aucune collection.',
			tags: 'Étiquettes',
			noTags:
				'Pas encore d’étiquette. Les étiquettes sont vos propres mots-clés, pour retrouver des documents d’une collection à l’autre.',
			rename: 'Renommer',
			title: 'Titre',
			results: 'Résultats',
			noResults: 'Rien n’a encore été lu dans ce document.',
			job: 'Traitement',
			setup: 'Modèle',
			progress: 'Lu',
			reviewed: 'Relu',
			cost: 'Coût',
			preferred: 'Retenu',
			prefer: 'Retenir',
			preferHint:
				'Le résultat sur lequel vous vous appuyez pour cette tâche. Un traitement plus récent ne le remplace jamais.',
			open: 'Relire',
			openJob: 'Page du traitement',
			detach: 'Détacher',
			detachConfirm:
				'Détacher ce résultat du document ? Le traitement et ses résultats sont conservés.',
			pieces: (finished: number, total: number, failed: number) =>
				`${finished} sur ${total}${failed > 0 ? `, ${failed} en échec` : ''}`,
			reviewedCount: (reviewed: number, total: number, corrected: number) =>
				`${reviewed} sur ${total}${corrected > 0 ? `, ${corrected} corrigé${corrected > 1 ? 's' : ''}` : ''}`,
			sameContentTitle: 'Même contenu',
			sameContentLede:
				'Ces documents ont exactement le même contenu. S’il s’agit d’une même pièce, fusionnez-les : leurs résultats et leurs collections sont réunis, et rien n’est relu.',
			merge: 'Fusionner dans celui-ci',
			mergeConfirm: (other: string) =>
				`Fusionner « ${other} » dans ce document ? Ses résultats et ses collections sont transférés ici, et il est retiré.`
		}
	},
	newJob: {
		startOver: 'Recommencer',
		startOverHint: 'Vider le formulaire : fichiers, modèles et réglages',
		then: {
			title: 'Ensuite (facultatif)',
			hint: 'Une fois le texte lu, Kalamu peut l’analyser pour vous : les personnes et les lieux qu’il nomme, un résumé, sa tonalité.',
			analyse: 'Analyser le texte',
			nothing: 'Rien de plus',
			check: 'Me laisser d’abord vérifier le texte',
			checkHint:
				'Recommandé. Corrigez ce que le modèle a mal lu ; l’analyse lit ensuite votre texte corrigé.',
			whatHappens: (analysis: string, model: string) =>
				`Une fois le texte lu, ${model} l’analyse (${analysis}) aussitôt. Son coût dépend du texte : il n’est connu qu’une fois le texte lu.`,
			whatHappensChecked: (analysis: string, model: string) =>
				`Une fois le texte lu, vérifiez-le en relecture, puis cliquez sur Continuer : ${model} l’analyse (${analysis}). Rien n’est envoyé pour l’analyse avant que vous le fassiez. Son coût dépend du texte.`,
			workflowName: (task: string, analysis: string) => `${task}, puis ${analysis}`
		},
		title: 'Nouveau traitement',
		lede: 'Choisissez les fichiers, un modèle et un prompt. Le coût est estimé avant tout envoi.',
		taskLabel: 'Que voulez-vous faire ?',
		filesTitle: 'Fichiers',
		addFiles: 'Ajouter des fichiers',
		addFolder: 'Ajouter un dossier',
		dropHint: 'Ou déposez des fichiers et des dossiers n’importe où dans cette fenêtre.',
		dropHere: 'Déposez-les ici pour les ajouter au traitement',
		setAside: {
			files: (names: string, task: string, n: number) =>
				n === 1
					? `Non ajouté : ${names}. ${task} ne lit pas ce type de fichier.`
					: `Non ajoutés : ${names}. ${task} ne lit pas ces types de fichiers.`,
			folders: (names: string, task: string) =>
				`Rien n’a été ajouté depuis ${names} : aucun fichier que ${task} sait lire.`,
			kinds:
				'Kalamu lit les PDF, les images (JPEG, PNG, TIFF, WebP, HEIC), les enregistrements, les fichiers vidéo et les textes (TXT, Markdown, Word, Excel, CSV).',
			youtube:
				'Ici, les vidéos s’ajoutent par leur lien YouTube. Les enregistrements et fichiers vidéo de cet ordinateur se lisent avec « Transcrire un enregistrement ».',
			instead: (task: string) => `Plutôt : ${task}`,
			insteadHint: 'Change de tâche et y ajoute ces fichiers',
			dismiss: 'Fermer ce message'
		},
		noFiles: 'Pas encore de fichiers.',
		remove: 'Retirer',
		pages: (n: number) => (n === 1 ? '1 page' : `${n} pages`),
		hoursMinutes: (h: number, min: string) => `${h} h ${min} min`,
		minutes: (min: number) => `${min} min`,
		seconds: (s: number) => `${s} s`,
		evenOut: 'Égaliser le volume',
		frozenText: 'texte corrigé, dans son état actuel',
		itemPages: 'les pages d’une pièce, dans leur état actuel',
		splitSpreads: 'Scinder les doubles pages',
		languageTitle: 'Langue',
		languageAuto: 'Détecter automatiquement',
		languagePick: 'Choisir les langues',
		analysisLanguageTitle: 'Langue des réponses',
		analysisLanguageAuto: 'Celle du document',
		analysisLanguageHint:
			'Choisissez une langue pour que les résumés, les sujets et les justifications y soient rédigés, quelle que soit la langue du document. Les noms restent écrits comme dans le document.',
		contentsLanguageHint:
			'Choisissez une langue pour que le résumé de chaque entrée y soit rédigé, quelle que soit la langue des pages. Les titres et les noms restent tels qu’imprimés.',
		languageHint:
			'Préciser la langue aide pour les documents courts ou bruités. Choisissez-en plusieurs pour les documents ou enregistrements qui mêlent les langues.',
		addLanguage: 'Ajouter une langue…',
		findLanguage: 'Tapez pour chercher une langue',
		favouriteLanguages: 'Favorites',
		allLanguages: 'Toutes les langues',
		noLanguageFound: 'Aucune langue de ce nom.',
		addFavourite: (language: string) => `Ajouter « ${language} » aux favorites`,
		removeFavourite: (language: string) => `Retirer « ${language} » des favorites`,
		favouritesHint:
			'Marquez d’une étoile les langues que vous utilisez le plus pour les trouver en premier.',
		languagesNarrowed: (models: string) =>
			`Seules les langues que ${models} prend en charge sont proposées.`,
		languageUnsupported: (language: string, models: string) =>
			`${models} ne prend pas en charge la langue « ${language} » : retirez-la ou choisissez un autre modèle.`,
		languageOneOnly: (models: string) =>
			`Une seule langue est transmise à ${models} : si vous en choisissez plusieurs, la langue est détectée.`,
		languageNotWithTimes: (models: string) =>
			`La langue n’est transmise à ${models} que s’il n’identifie pas les locuteurs et n’horodate pas les lignes ; dans les deux cas, la langue est détectée.`,
		languageNeedsTwoLetters: (language: string, models: string) =>
			`On ne peut indiquer à ${models} que des langues ayant un code à deux lettres : « ${language} » ne lui est donc pas transmise, et le modèle la détecte.`,
		languageNotDetected: (models: string) =>
			`${models} ne détecte pas la langue : choisissez les langues des documents.`,
		languageDataMissing: (languages: string) =>
			`Données de langue pas encore sur cet ordinateur : ${languages}. Téléchargez-les une fois dans Paramètres, puis lancez le traitement.`,
		languageIgnoredAll: (models: string) =>
			`${models} reconnaît la langue de lui-même : il n’y a rien à choisir.`,
		languageIgnoredSome: (models: string) =>
			`${models} reconnaît la langue de lui-même ; votre choix s’applique aux autres modèles.`,
		setupsTitle: 'Modèle et prompt',
		setupsHint: 'Ajoutez un second modèle pour comparer leurs résultats côte à côte.',
		provider: 'Fournisseur',
		model: 'Modèle',
		prompt: 'Prompt',
		noPrompt: 'Ce modèle ne prend pas de prompt',
		modelLanguages: (n: number) => `Les ${n} langues qu’il prend en charge`,
		reasoning: {
			title: 'Réflexion',
			suggested: (level: string) => `${level} (suggéré)`,
			level: (level: string) =>
				({
					none: 'Aucune',
					minimal: 'Minimale',
					low: 'Faible',
					medium: 'Moyenne',
					high: 'Élevée',
					xhigh: 'Très élevée',
					max: 'Maximale'
				})[level] ?? level,
			hint: 'Plus de réflexion coûte plus cher et prend plus de temps.'
		},
		label: 'Libellé',
		compareAdd: 'Comparer avec un autre modèle',
		analysis: 'Analyse',
		kindOfDocument: 'Type de document',
		finds: 'Remplit',
		entryHas: 'Chaque entrée comporte',
		mergeTitle: 'Regrouper les pages avec',
		mergeHint:
			'Lire chaque page est le plus difficile. Regrouper les pages en entrées est plus simple : un modèle plus petit et moins cher, de n’importe quel fournisseur, s’en acquitte généralement bien.',
		mergeRow: (model: string) => `${model}, pour regrouper les pages`,
		editFields: 'Modifier les champs dans Prompts',
		pipelinesTitle: 'Partir d’une configuration',
		pipelineSave: 'Enregistrer comme configuration',
		pipelineName: 'Nom de la configuration',
		pipelineSaved: (name: string) =>
			`Enregistrée sous « ${name} ». Elle apparaît en haut de Nouveau traitement.`,
		pipelineApplied: (name: string) => `Réglages de « ${name} » appliqués. Ajoutez les fichiers.`,
		pipelineDelete: (name: string) => `Supprimer la configuration « ${name} »`,
		pipelineDeleteConfirm: (name: string) =>
			`Supprimer la configuration « ${name} » ? Les traitements déjà effectués avec elle sont conservés.`,
		pipelineExport: (name: string) =>
			`Exporter « ${name} » : modèles, réglages et prompts complets, pour les citer ou les partager`,
		pipelineExported: (file: string) =>
			`${file}.json enregistré, à importer dans Kalamu, et ${file}.md, pour une section méthodologique.`,
		pipelineImport: 'Importer une configuration',
		pipelineImported: (name: string) => `« ${name} » importée avec ses prompts.`,
		linesTitle: 'Retours à la ligne',
		together: {
			title: 'Pages par appel',
			one: 'Une page à la fois',
			one_hint: 'Recommandé. Chaque page est lue séparément.',
			several: 'Plusieurs pages ensemble',
			several_hint:
				'Le modèle voit un article en entier quand il se poursuit sur la page suivante. Moins d’appels.',
			whole: 'Le document entier',
			whole_hint: 'Autant de pages que le modèle peut en écrire en une réponse.',
			pages: 'Pages par appel',
			safety:
				'Kalamu n’envoie jamais ensemble plus de pages que le modèle ne peut en écrire en une réponse, et redécoupe la réponse en pages pour la relecture. Une page que le modèle omet, ou que sa réponse tronque, est relue seule.',
			economy: 'En mode économique, les pages sont envoyées une à une.',
			alone:
				'Les images, et les PDF dont les doubles pages sont scindées, sont lus une page à la fois.',
			noPrompt: (models: string) => `${models} lit les pages une à une.`
		},
		lines: {
			running_text: 'Texte continu',
			running_text_hint:
				'Lignes réunies en paragraphes et mots coupés en fin de ligne recomposés. Idéal pour lire et pour chercher dans le texte.',
			as_on_page: 'Comme sur la page',
			as_on_page_hint:
				'Chaque ligne conservée là où elle finit sur la page, traits d’union de fin de ligne compris. Plus facile à confronter à la numérisation, mais un mot coupé sur deux lignes se trouve moins bien par la recherche.'
		},
		linesNoPromptAll: (models: string) =>
			`${models} ne prend pas d’instructions et dispose les lignes à sa façon : il n’y a rien à choisir.`,
		linesNoPromptSome: (models: string) =>
			`${models} ne prend pas d’instructions et dispose les lignes à sa façon ; ce choix s’applique aux autres modèles.`,
		removeSetup: 'Retirer',
		noKey: (provider: string) => `Ajoutez une clé ${provider} dans Paramètres pour l’utiliser.`,
		noProvider: 'Ajoutez une clé d’API dans Paramètres pour choisir un modèle.',
		diarize: 'Identifier les locuteurs',
		noSpeakers: 'Ce modèle ne sait pas identifier les locuteurs',
		costly:
			'Très coûteux, et très probablement plus que ce que demande cette tâche : les modèles recommandés lisent l’imprimé, l’écriture manuscrite et la parole presque aussi bien pour une fraction du prix. Faites d’abord un essai sur un échantillon.',
		notDownloaded: 'Ce modèle hors ligne n’a pas encore été téléchargé.',
		openSettings: 'Le télécharger dans Paramètres',
		timestamps: 'Horodatage',
		translate: 'L’écrire en anglais',
		translateHint:
			'Whisper écrit en anglais ce qui est dit, quelle que soit la langue parlée. La transcription est alors une traduction, ce que la provenance consigne.',
		vocabulary: {
			title: 'Noms et mots à orthographier',
			placeholder: 'Samory Touré\nBobo-Dioulasso\nmadrasa',
			hint: 'Personnes, lieux et termes tels que vous les écrivez, un par ligne. Whisper les lit avant chaque partie de l’enregistrement et les écrit ainsi quand il les entend. Les soixante premiers environ sont pris en compte.',
			fromProject: 'Ajouter les noms du dictionnaire de ce projet',
			noProjectNames:
				'Le dictionnaire de ce projet n’a pas encore de noms : ils viennent de ses analyses.'
		},
		sampleTitle: 'Faire d’abord un essai',
		sample: (n: number) => `Seulement les ${n} premières pages de chaque document`,
		outputTitle: 'Résultats',
		formats: 'Formats',
		besideSources: 'À côté des fichiers originaux',
		inFolder: 'Dans un dossier',
		chooseFolder: 'Choisir…',
		estimate: 'Coût estimé',
		estimateUnknown: 'pas de prix publié',
		estimateNote:
			'Une estimation fondée sur le nombre de pages et les prix publiés ; le coût réel peut varier.',
		paceTitle: 'Rapidité et prix',
		pace: {
			fast: 'Plus rapide',
			fast_hint:
				'Chaque page est envoyée séparément, et les résultats arrivent au fil de la lecture.',
			economy: 'Moins cher',
			economy_hint:
				'Toutes les pages sont envoyées ensemble au service de traitement par lots du fournisseur, pour environ la moitié du prix. Les résultats peuvent prendre jusqu’à une journée ; Kalamu peut être fermé entre-temps et les récupère à sa prochaine ouverture.'
		},
		paceNotBatched: (names: string) =>
			`${names} n’a pas de service de traitement par lots dans Kalamu et s’exécute toujours page par page, à son prix habituel.`,
		start: 'Lancer',
		startShortcut: '↵',
		starting: 'Lancement…',
		name: 'Nom du traitement',
		inspecting: 'Lecture des fichiers…',
		linksTitle: 'Vidéos',
		linkField: 'Liens YouTube',
		linkPlaceholder: 'https://www.youtube.com/watch?v=…',
		addLinks: 'Ajouter',
		linksHint:
			'Collez un ou plusieurs liens, un par ligne. Les liens watch, youtu.be, Shorts et en direct fonctionnent ; pas les playlists ni les chaînes.',
		openVideo: 'Ouvrir sur YouTube',
		openVideoNamed: (title: string) => `Ouvrir ${title} sur YouTube`,
		linksAdded: (added: number, refused: number) =>
			[
				added <= 1 ? `${added} vidéo ajoutée` : `${added} vidéos ajoutées`,
				refused > 0 ? (refused === 1 ? '1 lien non ajouté' : `${refused} liens non ajoutés`) : ''
			]
				.filter(Boolean)
				.join(', '),
		saveIn: 'Enregistrer les résultats dans',
		linkFolder:
			'Une vidéo n’a pas de dossier à elle : ses transcriptions et ses fichiers de provenance en demandent un.',
		needFolder: 'Choisissez un dossier pour les résultats avant de lancer le traitement.',
		estimateVideo:
			'La durée d’une vidéo n’est connue qu’une fois que Gemini l’a lue : le coût s’affiche donc dans Traitements en cours d’exécution.'
	},
	warning: {
		double_page_spread: (pages: string) =>
			`Deux pages côte à côte (page ${pages}) : les modèles risquent de lire d’une page à l’autre. Scindez-les avant de lancer le traitement.`,
		low_resolution: (dpi: number, pages: string) =>
			`Résolution faible, environ ${dpi} ppp (page ${pages}) : les petits caractères et l’écriture manuscrite risquent d’être mal lus. Numérisez de nouveau à 300 ppp ou plus si possible.`,
		has_text_layer: (n: number) =>
			`Ce PDF contient déjà du texte sur ${n} des premières pages. L’extraction n’est peut-être pas nécessaire, ou le texte existant peut servir de point de comparaison.`,
		unsupported_image: (detail: string) => detail,
		large_file: 'Ce fichier est volumineux ; il sera découpé avant l’envoi.',
		tool_missing:
			'La lecture de ce fichier nécessite FFmpeg, introuvable à côté de l’application. Réinstallez Kalamu, ou convertissez le fichier en MP3.',
		no_sound_track: 'Ce fichier n’a pas de piste audio : il n’y a rien à transcrire.',
		long_recording: (length: string, pieces: number) =>
			`Enregistrement long (${length}) : il sera envoyé en ${pieces} segments, coupés aux pauses. L’identification des locuteurs peut varier d’un segment à l’autre.`,
		quiet_recording: (db: number) =>
			`Enregistrement très faible (environ ${db} dB) : les paroles prononcées à voix basse risquent d’être manquées. Égaliser le volume aide généralement.`
	},
	tips: {
		title: 'Conseils pour de meilleurs résultats',
		scanning:
			'Une page par image ou par page de PDF, pas une double page ouverte ; à plat et entièrement dans le cadre ; un éclairage uniforme, sans ombre ni reflet de flash ; 300 à 400 ppp ; couleur ou niveaux de gris plutôt que noir et blanc pour les documents pâlis ou manuscrits.',
		recording:
			'Le micro près des locuteurs, une pièce calme, un enregistrement par entretien, et une brève présentation orale avec les noms et la date.',
		youtubePublic:
			'Seules les vidéos publiques peuvent être lues : les vidéos privées et non répertoriées sont refusées. Kalamu demande à YouTube le titre de chaque vidéo quand vous l’ajoutez ; Google lit ensuite la vidéo elle-même, et rien n’est téléchargé sur cet ordinateur.',
		youtubePreview:
			'La lecture de liens YouTube est une fonction en préversion de Gemini, et Google peut en modifier les limites. Une clé gratuite permet de lire jusqu’à 8 heures de vidéo par jour. Chaque vidéo est transcrite d’un seul tenant, ce qui convient aux vidéos de deux heures environ au plus.',
		results:
			'Faites un essai sur un échantillon avant tout un dossier, comparez deux modèles sur les documents difficiles, vérifiez noms, chiffres et dates sur la source, et conservez les originaux.',
		analysisUnit:
			'Un fichier par document : un PDF de plusieurs pages est lu en entier et reçoit un seul jeu de réponses. Placez chaque article, lettre ou rapport dans un fichier distinct. Plusieurs documents dans un même fichier reçoivent des réponses mêlées, et un document réparti sur plusieurs fichiers en reçoit plusieurs, partielles.',
		analysisInput:
			'Les numérisations comme le texte conviennent : le modèle lit lui-même les pages d’un PDF. Pour un imprimé pâli ou une écriture manuscrite, extraire et corriger d’abord le texte garantit que chaque réponse repose sur une lecture que vous avez vérifiée.',
		analysisFields:
			'Décrivez chaque champ comme vous le feriez pour un assistant de recherche, proposez des choix qui ne se recoupent pas, et demandez une justification. Faites d’abord un essai sur quelques documents, et comparez deux modèles lorsque la réponse relève du jugement, comme la tonalité.',
		contentsUnit:
			'Un fichier par numéro, dossier d’archives ou volume : il est lu page par page, puis les pages sont regroupées. Réunissez d’abord en un seul PDF les photographies d’un même dossier d’archives.',
		contentsCheck:
			'Faites un essai sur les premières pages, puis vérifiez les étendues de pages et les articles qui courent sur plusieurs pages : la relecture montre chaque entrée à côté de sa première page.',
		correctionInput:
			'Kalamu corrige le texte déjà présent dans le fichier : la couche texte d’un PDF, ou un fichier texte provenant d’une bibliothèque numérique (Gallica, Europeana, Internet Archive…). Une numérisation sans couche texte relève plutôt d’Extraire le texte.',
		correctionCheck:
			'La relecture met en évidence chaque modification par rapport au texte d’origine. Vérifiez les noms, les chiffres et les dates, que la lecture automatique se trompe le plus souvent à lire.'
	},
	pipeline: {
		title: 'Comment ça marche',
		replay: 'Rejouer',
		moreFiles: (n: number) => (n === 1 ? 'et 1 autre fichier' : `et ${n} autres fichiers`),
		theModel: 'Le modèle',
		extractSource: 'Un PDF ou des images',
		extractSplit: 'Découpé en pages : un segment par page',
		extractSpreads: 'Les doubles pages sont d’abord scindées en deux',
		extractRead: (model: string) => `${model} lit chaque page séparément`,
		extractJoin: 'Réuni en un texte par fichier, numéros de page conservés',
		extractPlace: 'Et note l’emplacement de son texte sur la page',
		extractPlaces: (models: string) => `${models} note aussi l’emplacement de son texte`,
		extractTextOnly:
			'Le texte seul, sans son emplacement sur la page : ni PDF interrogeable ni ALTO',
		extractSearchable: 'Aussi un fichier ALTO XML et un PDF interrogeable, chaque mot à sa place',
		extractPdfOnly: 'Le PDF interrogeable pour les fichiers PDF seulement',
		analysisWhole: 'Lu en entier, d’un seul tenant, quel que soit le nombre de pages',
		analysisFill: (model: string) => `${model} remplit vos champs`,
		analysisTable: 'Une ligne de réponses par document, prête pour un tableur',
		exampleFields: ['Résumé', 'Personnes', 'Lieux', 'Tonalité'],
		document: (n: number) => `Document ${n}`,
		audioSource: 'Un enregistrement',
		audioCut: (pieces: number) =>
			pieces === 1
				? 'Assez court pour être envoyé en entier : un segment'
				: `Coupé aux pauses en ${pieces} segments de 20 min au plus`,
		audioCutExample: 'Coupé aux pauses en segments de 20 min au plus',
		audioRead: (model: string) => `${model} transcrit chaque segment`,
		audioJoin: 'Réuni en une transcription horodatée',
		audioSpeakers: 'L’identification des locuteurs peut varier d’un segment à l’autre',
		youtubeSource: 'Un lien YouTube public',
		youtubeSend: 'Seul le lien quitte cet ordinateur : rien n’est téléchargé',
		youtubeRead: (model: string) => `${model} regarde toute la vidéo d’un seul tenant`,
		youtubeJoin: 'Une transcription horodatée',
		thisPc: 'Cet ordinateur',
		google: 'Google',
		contentsSource: 'Un long document : un numéro, un dossier d’archives, un volume',
		contentsRead: (model: string) => `${model} relève les articles ou les pièces de chaque page`,
		contentsContinues: 'Les mentions telles que « suite page 5 » sont conservées',
		contentsMerge: (model: string) =>
			`${model} regroupe les pages : une entrée par article ou par pièce`,
		contentsTable: 'Une table des matières détaillée : chaque entrée avec ses pages et vos champs',
		exampleEntries: ['Éditorial', 'Entretien', 'Courrier des lecteurs'],
		correctionSource: 'Un texte qui existe déjà : la couche texte d’un PDF ou un fichier texte',
		correctionSplit: 'Un segment par page, ou par passage d’un texte long',
		correctionRead: (model: string) => `${model} corrige les erreurs de lecture automatique`,
		correctionCompare:
			'Chaque modification signalée par rapport au texte d’origine, à garder ou à annuler'
	},
	scores: {
		title: 'Erreurs relevées en relecture',
		rates: (characters: string, words: string) =>
			`${characters} des caractères et ${words} des mots erronés`,
		summary: (rates: string, plain: string, on: string) => `${rates} (${plain}), ${on}`,
		noErrors: (on: string) => `Aucune erreur, ${on}`,
		oneIn: (n: string) => `environ 1 caractère sur ${n}`,
		many: 'la moitié des caractères ou plus',
		pages: (n: number) => (n === 1 ? 'sur 1 page relue' : `sur ${n} pages relues`),
		parts: (n: number) => (n === 1 ? 'sur 1 partie relue' : `sur ${n} parties relues`),
		note: 'Calculé sur les pages que vous avez corrigées ou marquées comme relues : les erreurs qui vous ont échappé ne sont pas comptées.'
	},
	jobs: {
		title: 'Traitements',
		empty: 'Pas encore de traitement. Lancez-en un depuis Nouveau traitement.',
		resume: 'Reprendre',
		pause: 'Mettre en pause',
		cancel: 'Annuler',
		save: 'Enregistrer',
		retry: 'Relancer les pages en échec',
		delete: 'Supprimer',
		deleteConfirm: 'Supprimer ce traitement ? Les fichiers déjà écrits restent sur le disque.',
		review: 'Relire',
		summarise: 'Analyser les résultats',
		analyseSetup: 'Texte de',
		derivedChanged:
			'Le texte lu par cette analyse a été corrigé depuis. Ses réponses restent telles quelles ; analysez de nouveau le texte pour tenir compte des corrections.',
		derivedAgain: 'Analyser le texte corrigé',
		derivedGone:
			'Le traitement dont provient ce texte a été supprimé. Le texte lu est conservé, et ses réponses restent telles quelles.',
		noTextOutputs:
			'Ce traitement n’a écrit aucun fichier texte à analyser. Ajoutez Texte (.txt) à ses formats et exportez de nouveau.',
		openFolder: 'Afficher dans le dossier',
		openFile: 'Ouvrir dans son application',
		rename: 'Renommer',
		nameLabel: 'Nom du traitement',
		progress: (done: number, total: number) => `${done} segments sur ${total}`,
		failed: (n: number) => `${n} en échec`,
		truncated: (n: number) => (n <= 1 ? `${n} tronqué` : `${n} tronqués`),
		truncatedNote:
			'La réponse du modèle s’est arrêtée à sa longueur maximale : la fin de ces segments manque peut-être. La relecture les signale ; vérifiez-les sur la source.',
		files: 'Fichiers',
		livePreview: 'Dernier résultat',
		outputs: 'Écrits',
		created: 'Créé',
		task: 'Tâche',
		models: 'Modèles',
		pieces: 'Segments',
		cost: 'Dépensé',
		interruptedNotice:
			'Ce traitement s’est arrêté avant la fin, à la fermeture de l’application ou parce qu’une clé a cessé de fonctionner. Les segments terminés sont enregistrés : la reprise continue là où il s’était arrêté, et rien n’est envoyé ni payé deux fois.',
		pausedNotice: 'En pause. Reprenez pour continuer là où il s’était arrêté.',
		waitingNotice: (n: number, due: string) =>
			`${n === 1 ? '1 segment est' : `${n} segments sont`} au service de traitement par lots du fournisseur, avec une réponse au plus tard le ${due}. Kalamu peut être fermé entre-temps : il récupère les réponses à sa prochaine ouverture.`,
		heldNotice: (n: number) =>
			`${n === 1 ? '1 segment est' : `${n} segments sont`} encore au service de traitement par lots du fournisseur. Reprenez pour récupérer les réponses ; rien n’est envoyé ni payé deux fois.`,
		pace: 'Rapidité',
		paceName: { fast: 'Plus rapide', economy: 'Moins cher (par lots)' },
		empty_new: 'Lancer un traitement'
	},
	allJobs: {
		title: 'Tous les traitements',
		count: (n: number) => (n <= 1 ? `${n} traitement` : `${n} traitements`),
		link: (n: number) => `Tous les traitements (${n})`,
		search: 'Chercher dans les noms et les fichiers',
		anyTask: 'Toutes les tâches',
		anyStatus: 'Tous les états',
		sortBy: 'Trier par',
		sortNewest: 'Les plus récents',
		sortOldest: 'Les plus anciens',
		sortName: 'Nom',
		columnName: 'Nom',
		columnTask: 'Tâche',
		columnStatus: 'État',
		columnPieces: 'Segments',
		columnCost: 'Dépensé',
		columnDate: 'Créé le',
		files: (n: number) => (n <= 1 ? `${n} fichier` : `${n} fichiers`),
		noMatch: 'Aucun traitement ne correspond à ces filtres.',
		selectAll: 'Sélectionner tous les traitements affichés',
		select: (name: string) => `Sélectionner « ${name} »`,
		selected: (n: number) => (n <= 1 ? `${n} sélectionné` : `${n} sélectionnés`),
		clearSelection: 'Désélectionner',
		actions: 'Actions',
		addTo: 'Ajouter à un projet…',
		addedTo: (n: number, project: string) =>
			`${n <= 1 ? `${n} traitement ajouté` : `${n} traitements ajoutés`} au projet « ${project} ».`,
		delete: 'Supprimer',
		deleteConfirm: (n: number) =>
			`Supprimer ${n <= 1 ? 'ce traitement' : `ces ${n} traitements`} ? Les fichiers déjà écrits restent sur le disque.`,
		runningKept:
			'Un traitement en cours ne peut pas être supprimé : mettez-le en pause ou annulez-le d’abord.'
	},
	status: {
		pending: 'En attente',
		running: 'En cours',
		paused: 'En pause',
		interrupted: 'Interrompu',
		completed: 'Terminé',
		completed_with_errors: 'Terminé avec des erreurs',
		cancelled: 'Annulé',
		done: 'Terminé',
		cached: 'En cache',
		submitted: 'Chez le fournisseur',
		failed: 'En échec'
	},
	review: {
		title: 'Relecture',
		back: 'Retour au traitement',
		page: (label: string) => `Page ${label}`,
		pageHalf: (n: number, half: 'left' | 'right') =>
			`Page ${n}, moitié ${half === 'left' ? 'gauche' : 'droite'}`,
		pageHalfShort: (n: number, half: 'left' | 'right') => `${n}${half === 'left' ? 'g' : 'd'}`,
		passage: (n: number) => `Passage ${n}`,
		segment: (label: string) => `À partir de ${label}`,
		document: 'Document entier',
		video: 'Vidéo entière',
		openVideo: 'Ouvrir sur YouTube',
		videoPlayer: 'La vidéo, dans le lecteur de YouTube',
		videoHint: 'Cliquez sur une ligne pour lire la vidéo à partir de là.',
		videoHintOpen: 'Cliquez sur une ligne pour ouvrir la vidéo à ce moment sur YouTube.',
		playVideo: 'Lire la vidéo',
		playVideoNote: 'Lue depuis YouTube, dans son lecteur à confidentialité renforcée.',
		videoBlocked:
			'Cette vidéo ne peut pas être lue ici : son propriétaire ne l’autorise peut-être pas, ou cet ordinateur est hors ligne.',
		previous: 'Précédent',
		next: 'Suivant',
		source: 'Source',
		itemMenu: (name: string) => `Que faire de « ${name} » ?`,
		renameItem: 'Renommer',
		removeItem: 'Retirer des réponses',
		itemLocates: 'Marqué dans la source tant qu’il a le focus.',
		itemKeys:
			'Marqué dans la source tant qu’il a le focus. Entrée, ou la touche de menu, le renomme ou le retire.',
		pageImage: 'Image de la page : les flèches la déplacent, avec Maj par pas plus longs',
		itemRemoved: (name: string) => `« ${name} » retiré des réponses`,
		itemEverywhere:
			'Modifié dans les réponses corrigées de ce document, pour chaque modèle qui l’a relevé. Les réponses du modèle sont conservées.',
		edit: 'Corriger',
		save: 'Enregistrer la correction',
		discard: 'Abandonner les modifications',
		revert: 'Revenir au texte du modèle',
		reviewed: 'Relu',
		markReviewed: 'Marquer comme relu',
		corrected: 'Corrigé',
		showDiff: 'Mettre en évidence les différences',
		scores: 'Scores',
		cer: 'Taux d’erreur par caractère',
		wer: 'Taux d’erreur par mot',
		scoredPages: (n: number) => (n === 1 ? 'sur 1 page relue' : `sur ${n} pages relues`),
		noReference:
			'Corrigez une page, ou marquez-la comme relue, pour évaluer les modèles par rapport à elle. En attendant, la vue montre où ils divergent.',
		agreement: (pct: string) => `${pct} de différence`,
		truncated:
			'La réponse du modèle s’est arrêtée ici à sa longueur maximale : la fin manque peut-être. Vérifiez-la sur la source, ou corrigez-la.',
		noPreview: 'Pas d’aperçu pour ce type de fichier.',
		notDone: 'Pas encore lu.',
		saved: 'Enregistré ; les fichiers de résultats ont été mis à jour.',
		pieceReviewed: (label: string) => `${label}, relu`,
		zoomOutShortcut: '−',
		zoomInShortcut: '+',
		zoomFitShortcut: '0',
		zoomIn: 'Agrandir',
		zoomOut: 'Réduire',
		zoomHint: (key: string) =>
			`${key} + molette : zoomer · glisser : déplacer · double-clic : agrandir`,
		turnLeft: 'Faire pivoter la page vers la gauche',
		turnRight: 'Faire pivoter la page vers la droite',
		play: 'Lire',
		pausePlayback: 'Pause',
		position: 'Position dans ce segment',
		noAudio: 'Ce segment n’a pas pu être lu.',
		lines: 'Lignes de la transcription',
		noTimings:
			'Ce modèle n’a pas donné de minutage : les lignes ne peuvent pas suivre l’enregistrement.',
		reviewedOf: (n: number, total: number) => `${n} relu${n > 1 ? 's' : ''} sur ${total}`,
		readingText: 'Texte lu',
		correcting: 'Correction',
		omitted: 'om.',
		speakers: 'Locuteurs',
		speakersHint:
			'Donnez de vrais noms aux étiquettes du modèle. Ils les remplacent ici et dans chaque fichier ; le texte du modèle lui-même est conservé.',
		speakerPlaceholder: 'Nom',
		speakerName: (label: string) => `Nom pour ${label}`,
		saveNames: 'Enregistrer les noms',
		namesSaved: 'Noms enregistrés ; les fichiers de résultats ont été mis à jour.',
		speakersPieces:
			'Cet enregistrement a été envoyé en plusieurs segments : une même étiquette peut désigner une personne différente dans chacun. Les noms s’appliquent à tout l’enregistrement.',
		yourCorrections: 'Vos corrections',
		answers: 'Réponses',
		noneFound: 'Rien de relevé',
		onePerLine: 'Un par ligne.',
		reviewedShortcut: '↵',
		markNotReviewed: 'Marquer comme non relu',
		pages: 'Pages',
		differences: (n: number) => (n <= 1 ? `${n} différence` : `${n} différences`),
		noDifferences: 'Les modèles concordent',
		previousDifference: 'Différence précédente',
		nextDifference: 'Différence suivante',
		differenceOf: (n: number, total: number) => `${n} sur ${total}`,
		disagree: 'Les modèles lisent ce passage différemment',
		corrected_here: 'Vous avez corrigé ce passage',
		useReading: 'Mettre dans le texte',
		inUse: 'Dans le texte',
		ownReading: 'Votre propre lecture',
		ownReadingPlaceholder: 'Tapez ce que porte la source',
		apply: 'Appliquer',
		readingKept: 'Le texte du modèle est conservé ; votre choix est enregistré comme correction.',
		modelText: (label: string) => `${label}, texte du modèle`,
		models: 'Modèles',
		legendDiffer: 'Les modèles divergent',
		legendCorrected: 'Votre correction',
		fitWidth: 'Ajuster à la largeur',
		compare: 'Comparer',
		compareWith: 'Texte comparé',
		closeCompare: 'Fermer la comparaison',
		resizeSource: 'Redimensionner la source et le texte',
		resizeCompare: 'Redimensionner le texte et le texte comparé',
		addedHere: 'Un autre modèle ajoute des mots ici',
		leftOutHere: 'Ce modèle omet des mots ici',
		pickReading: 'Choisir la bonne version',
		moveReadings: 'Déplacer cette boîte (faites-la glisser ou utilisez les flèches)',
		original: 'Texte d’origine',
		changedHere: 'La correction a modifié ce passage',
		legendChanged: 'Modifié par rapport à l’original',
		originalSiglum: 'O',
		contentsTitle: 'Table des matières',
		entries: (n: number) => (n <= 1 ? `${n} entrée` : `${n} entrées`),
		entryPages: 'Pages',
		entryTitle: 'Titre',
		addEntry: 'Ajouter une entrée',
		removeEntry: 'Retirer cette entrée',
		goToPage: (page: string) => `Afficher la page ${page}`,
		pageReading: 'Lu sur cette page',
		pageNote: 'Aussi sur cette page',
		notCombined: 'Les pages n’ont pas encore été regroupées.',
		editEntries: 'Corriger les entrées',
		search: 'Rechercher',
		searchShortcut: 'F',
		searchText: 'Rechercher dans ce document',
		searchEntries: 'Rechercher dans les entrées',
		found: (n: number) => (n <= 1 ? `${n} trouvé` : `${n} trouvés`),
		foundAt: (at: number, n: number) => `${at} sur ${n}`,
		notFound: 'Introuvable',
		previousMatch: 'Occurrence précédente',
		nextMatch: 'Occurrence suivante',
		listMatches: 'Liste',
		listMatchesHint: 'Lister les occurrences par page',
		matches: 'Occurrences',
		moreMatches: (shown: number, n: number) => `Les ${shown} premières sur ${n} sont listées.`,
		closeSearch: 'Fermer la recherche',
		entriesFound: (n: number, total: number) =>
			`${n} ${n <= 1 ? 'entrée' : 'entrées'} sur ${total}`,
		noEntryFound: 'Aucune entrée ne contient ces mots.',
		searchAnswers: 'Rechercher dans les réponses',
		searchAnswersAndText: 'Rechercher dans les réponses et le document',
		answersFound: (n: number, total: number) =>
			`${n} ${n <= 1 ? 'réponse' : 'réponses'} sur ${total}`,
		inDocument: (n: number) => `${n} dans le document`,
		noAnswerFound: 'Aucune réponse ne contient ces mots.'
	},
	prompts: {
		title: 'Prompts',
		hint: 'Chaque tâche est livrée avec des prompts éprouvés. Modifiez-en un pour l’adapter ; votre version le remplace jusqu’à ce que vous le réinitialisiez. Variables : {languages}, {speakers}.',
		builtin: 'Par défaut',
		modified: 'Modifié',
		own: 'Le vôtre',
		name: 'Nom',
		description: 'Description',
		instructions: 'Instructions (prompt système)',
		userPrompt: 'Requête envoyée avec chaque page',
		userPromptDocument: 'Requête envoyée avec chaque document',
		mergeInstructions: 'Instructions pour regrouper les pages',
		save: 'Enregistrer',
		duplicate: 'Dupliquer',
		copyName: (name: string) => `${name} (copie)`,
		newPrompt: 'Nouveau prompt',
		newName: 'Mon prompt',
		task: 'Utilisé pour',
		taskHint:
			'Un prompt appartient à une tâche : Nouveau traitement le propose quand cette tâche est choisie.',
		newHint:
			'Écrivez vos instructions. La requête et, pour une analyse ou une table des matières, ses champs partent du prompt par défaut de cette tâche ; modifiez-les au besoin.',
		instructionsPlaceholder:
			'Par exemple : Vous lisez des pages de journaux imprimés d’Afrique occidentale française, de 1940 à 1960. Transcrivez le texte tel qu’il est imprimé, colonne par colonne, en gardant l’orthographe d’origine.',
		discard: 'Abandonner',
		reset: 'Rétablir la version par défaut',
		delete: 'Supprimer',
		export: 'Exporter…',
		import: 'Importer…',
		imported: (n: number) => (n <= 1 ? `${n} importé` : `${n} importés`),
		saved: 'Enregistré',
		unsaved: 'Modifications non enregistrées',
		analysisHint:
			'Une analyse est un ensemble de champs à remplir pour chaque document. Créez un nouveau prompt, ou dupliquez l’une de celles-ci pour partir d’elle.',
		contentsHint:
			'Une table des matières se fait en deux étapes : des instructions pour lire chaque page, et des instructions pour regrouper les pages. Chaque entrée a un titre et des pages ; les champs ci-dessous ajoutent ce que vous voulez pour chacune.',
		dictionary: 'Dictionnaire',
		dictionaryHint:
			'Envoyé quand vous demandez à un modèle de proposer des regroupements dans le dictionnaire d’un projet. Modifiez les instructions pour changer ce qu’il regroupe ; la réinitialisation rétablit la version par défaut.',
		dictionaryFields:
			'Les champs de la réponse sont fixes (les lignes de chaque groupe, un libellé, une justification et un degré de certitude), car le dictionnaire les lit. Les instructions disent quoi regrouper.'
	},
	fields: {
		title: 'Champs',
		hint: 'Ce qu’il faut relever dans chaque document. Chaque champ devient une colonne dans l’export en tableur.',
		name: 'Nom',
		kind: 'Réponse',
		kinds: {
			text: 'Texte',
			list: 'Liste de noms ou de termes',
			choice: 'Un choix',
			choices: 'Plusieurs choix'
		},
		max: 'Au plus',
		values: 'Choix, séparés par des virgules',
		valuesPlaceholder: 'Positive, Neutre, Négative',
		instructions: 'Ce qui relève de ce champ',
		instructionsPlaceholder:
			'Par exemple : villes, villages et régions ; les mosquées comptent comme des lieux.',
		reason: 'Demander une justification',
		remove: (label: string) => `Retirer ${label || 'ce champ'}`,
		up: 'Monter',
		down: 'Descendre',
		add: 'Ajouter un champ',
		templates: {
			summary: {
				label: 'Résumé',
				instructions: 'Ce que dit ce document en particulier : qui, quoi, où, quand.'
			},
			people: { label: 'Personnes', instructions: 'Les personnes nommées dans le document.' },
			places: { label: 'Lieux', instructions: 'Les lieux nommés dans le document.' },
			organisations: {
				label: 'Organisations',
				instructions:
					'Associations, partis, administrations, entreprises, institutions religieuses.'
			},
			keywords: {
				label: 'Mots-clés',
				instructions: 'Les termes que l’on chercherait, précis plutôt que généraux.'
			},
			tone: {
				label: 'Tonalité',
				instructions: 'La manière dont le document présente son sujet principal.'
			},
			custom: { label: 'Autre', instructions: '' }
		},
		toneValues: ['Positive', 'Neutre', 'Négative', 'Mitigée']
	},
	settings: {
		title: 'Paramètres',
		lede: 'Clés, modèles hors ligne, traitement, apparence et langue.',
		tabsLabel: 'Sections des paramètres',
		tabs: {
			keys: 'Clés d’API',
			offline: 'Modèles hors ligne',
			general: 'Général',
			about: 'À propos'
		},
		keysTitle: 'Clés d’API',
		keysHint: (keychain: string) =>
			`Les clés sont conservées dans ${keychain} et envoyées seulement à leur fournisseur. Les fichiers vont directement de cet ordinateur au fournisseur que vous choisissez.`,
		keySaved: 'Enregistrée',
		keyMissing: 'Aucune clé',
		keyPlaceholder: 'Collez la clé',
		save: 'Enregistrer',
		test: 'Tester',
		remove: 'Retirer',
		getKey: 'Obtenir une clé',
		testOk: (n: number) => `La clé fonctionne : ${n} modèles disponibles`,
		offlineTitle: 'Travailler hors ligne',
		whisperTitle: 'Transcription avec Whisper',
		tesseractTitle: 'Extraction de texte avec Tesseract',
		tesseractHint: (version: string) =>
			`Tesseract ${version} lit les pages imprimées sur cet ordinateur : rien n’est envoyé, aucune clé ni aucun compte n’est nécessaire, et il fonctionne sans connexion internet une fois une langue téléchargée. Il est fait pour l’imprimé net ; l’écriture manuscrite, les pages pâlies ou abîmées et les mises en page complexes se lisent mieux avec un modèle en ligne. Sa vitesse dépend du processeur de cet ordinateur : d’environ une seconde à plusieurs secondes par page. Il ne détecte pas la langue : téléchargez chaque langue que vous lisez, puis choisissez-la dans Nouveau traitement.`,
		latinScript: (languages: string) =>
			`Langues écrites en lettres latines sans modèle propre : ${languages}`,
		noLanguageData: 'Aucune langue téléchargée pour l’instant.',
		ocrTitle: 'Extraction de texte avec des modèles OCR',
		ocrHint:
			'De petits modèles de vision entraînés à lire des pages, qui s’exécutent sur cet ordinateur : rien n’est envoyé, aucune clé ni aucun compte n’est nécessaire, et ils fonctionnent sans connexion internet une fois téléchargés. Ils lisent bien mieux que Tesseract l’imprimé abîmé, les vieux journaux en colonnes, les tableaux et les mises en page mêlées, mais ils sont bien plus lents : de quelques dizaines de secondes à plus d’une minute par page, selon le processeur de cet ordinateur, sa mémoire et la quantité de texte sur la page. Sur un Mac, et sous Windows quand Kalamu trouve un processeur graphique qu’il peut utiliser, ils s’exécutent sur le processeur graphique et vont plusieurs fois plus vite. Comme tout modèle de langue, ils peuvent écrire du texte qui n’est pas sur la page : comparez les pages douteuses avec Tesseract ou un modèle en ligne. Pour l’imprimé net, Tesseract est plus rapide ; pour de gros volumes, un modèle en ligne.',
		ocrSpeed: (seconds: number, graphics: boolean) =>
			`Sur cet ordinateur${graphics ? ', avec son processeur graphique' : ''} : environ ${seconds} s par page, d’après les dernières pages lues.`,
		ocrGraphics: (name: string) =>
			`Faire tourner Whisper, les modèles OCR et les modèles de langue sur le processeur graphique (${name})`,
		ocrGraphicsHint:
			'En général plus rapide que le processeur, souvent plusieurs fois. Seuls les modèles dont on sait qu’ils lisent correctement sur ce processeur graphique l’utilisent ; les autres lisent avec le processeur. Si une page y échoue, Kalamu la relit avec le processeur et s’en tient au processeur jusqu’à son prochain démarrage. Les modèles de langue hors ligne ci-dessous suivent aussi ce choix, en y plaçant autant du modèle que sa mémoire en contient, de même que Whisper sous Windows : un enregistrement qui y échoue est retranscrit avec le processeur. Désactivez cette option pour laisser le processeur graphique libre pour d’autres tâches.',
		ocrNotTimed: 'Pas encore mesuré sur cet ordinateur : les premières pages lues le diront.',
		ocrNoEngine:
			'Cette copie de Kalamu a été compilée sans le moteur qui fait tourner les modèles OCR : les installateurs et la version du Store le contiennent.',
		ocrCpu: (missing: string) =>
			`Il manque à ce processeur ${missing}, dont les modèles OCR hors ligne ont besoin. Utilisez plutôt Tesseract ou un modèle en ligne.`,
		llmTitle: 'Analyses avec des modèles de langue',
		llmHint:
			'De petits modèles de langue de Google, qui s’exécutent sur cet ordinateur, pour les analyses, les tables des matières et les corrections : rien n’est envoyé, aucune clé ni aucun compte n’est nécessaire, et ils fonctionnent sans connexion internet une fois téléchargés. Ils sont bien moins bons que les modèles en ligne : leurs réponses sont plus pauvres, ils peuvent oublier ou inventer des noms et des faits, et ils suivent moins bien les longues consignes, alors vérifiez ce qu’ils écrivent. Ils lisent des textes d’environ 11 000 mots au plus, et des pages photographiées ; pour analyser un PDF, extrayez d’abord son texte. Ils sont lents sur le processeur, jusqu’à plusieurs minutes pour un long texte, et plusieurs fois plus rapides sur un processeur graphique : celui d’un Mac, ou sous Windows un processeur graphique que Kalamu peut utiliser ; s’il a peu de mémoire propre, il prend une partie du modèle. Chacun demande plusieurs gigaoctets de mémoire libre pendant qu’il travaille.',
		llmNoEngine:
			'Cette copie de Kalamu a été compilée sans le moteur qui fait tourner les modèles de langue hors ligne : les installateurs et la version du Store le contiennent.',
		llmCpu: (missing: string) =>
			`Il manque à ce processeur ${missing}, dont les modèles de langue hors ligne ont besoin. Utilisez plutôt un modèle en ligne.`,
		addLanguageData: 'Télécharger une langue pour Tesseract',
		findLanguageData: 'Chercher une langue par son nom',
		oneDownload:
			'Un téléchargement à la fois : le suivant pourra commencer une fois celui-ci terminé.',
		offlineHint:
			'Whisper s’exécute sur cet ordinateur : les enregistrements ne le quittent jamais, aucune clé ni aucun compte n’est nécessaire, et il fonctionne sans connexion internet une fois un modèle téléchargé. Il est plus lent que les modèles en ligne et ne sait pas identifier les locuteurs. Large v3 Turbo convient à la plupart des ordinateurs.',
		offlineCpu: (missing: string) =>
			`Il manque à ce processeur ${missing}, dont la transcription hors ligne a besoin. Utilisez plutôt un modèle en ligne.`,
		modelInstalled: 'Téléchargé',
		modelMissing: 'Non téléchargé',
		modelDownloading: (pct: number) => `Téléchargement… ${pct} %`,
		modelInUse: 'Utilisé par un traitement',
		download: 'Télécharger',
		cancelDownload: 'Annuler',
		removeModel: 'Supprimer',
		size: (bytes: number) =>
			bytes >= 1e9
				? `${new Intl.NumberFormat('fr', { maximumFractionDigits: 1, minimumFractionDigits: 1 }).format(bytes / 1e9)} Go`
				: `${Math.round(bytes / 1e6)} Mo`,
		jobsTitle: 'Traitement',
		concurrency: 'Pages envoyées simultanément',
		concurrencyHint:
			'Prend effet à la prochaine ouverture de Kalamu. Réduisez-le si un fournisseur demande à Kalamu de ralentir.',
		catalogTitle: 'Modèles',
		catalogUpdated: (date: string) => `Liste des modèles du ${date}`,
		catalogRefresh: 'Chercher une liste de modèles à jour',
		catalogAuto: 'Vérifier au démarrage',
		catalogNote:
			'Les prix sont en dollars américains, tirés de la liste que publie chaque fournisseur à la date ci-dessus. Ils changent : Kalamu n’affiche donc que des estimations. Un modèle marqué « préversion » peut changer ou être retiré.',
		appearanceTitle: 'Apparence et langue',
		theme: 'Thème',
		themeSystem: 'Celui du système',
		themeLight: 'Clair',
		themeDark: 'Sombre',
		language: 'Langue de l’interface',
		languageSystem: 'Celle du système',
		languageHint:
			'Les prompts, les réponses des modèles et les fichiers écrits par Kalamu gardent leur propre langue : choisissez la langue des réponses dans Nouveau traitement.',
		aboutTitle: 'À propos',
		version: (v: string) => `Version ${v}`,
		dataFolder: 'Dossier des données',
		credits:
			'Kalamu utilise {ffmpeg} sous licence LGPL v2.1 (son {source} est publié avec chaque version), {opus} (BSD), {whisper} (MIT), les {models} (MIT), {tesseract} et ses données de langue (Apache 2.0), {leptonica} (BSD), {hayro} (MIT), {llama} (MIT), {lighton} (Apache 2.0), {glm} (MIT) et {gemma} (Apache 2.0). Les licences complètes se trouvent dans le dossier des licences installé avec l’application.',
		creditNames: {
			ffmpeg: 'FFmpeg',
			source: 'code source',
			opus: 'Opus',
			whisper: 'whisper.cpp',
			models: 'modèles Whisper',
			tesseract: 'Tesseract',
			leptonica: 'Leptonica',
			hayro: 'hayro',
			llama: 'llama.cpp',
			lighton: 'LightOnOCR-3',
			glm: 'GLM-OCR',
			gemma: 'Gemma 4'
		},
		privacy:
			'Rien n’est envoyé au développeur : ni fichiers, ni clés, ni résultats, ni données d’utilisation. Un texte produit par l’IA peut être fluide mais faux ; vérifiez un échantillon sur la source.'
	},
	report: {
		link: 'Signaler un problème',
		review: 'Signaler un problème',
		note: 'Un bogue, une idée, ou un texte choquant écrit par un modèle : dites-le-nous sur GitHub. La page s’ouvre dans votre navigateur ; Kalamu n’envoie rien, ne partagez donc que ce qui peut être rendu public.'
	},
	glossary: {
		title: 'Glossaire',
		lede: 'Les mots qu’emploie Kalamu, pour les collègues qui découvrent ces outils, et l’usage de chaque format d’export.',
		wordsTitle: 'Mots',
		formatsTitle: 'Formats d’export',
		keysTitle: 'Clavier',
		keysLede:
			'Les raccourcis, selon l’endroit où ils agissent. Ceux de l’écran derrière attendent tant qu’une boîte de dialogue est ouverte.',
		keysOr: 'ou',
		keyPlaces: {
			everywhere: 'Partout',
			newJob: 'Nouveau traitement',
			review: 'En relecture',
			search: 'Dans la recherche d’une relecture',
			scan: 'Sur un scan'
		},
		keys: {
			newJob: 'Ouvrir Nouveau traitement',
			sidebar: 'Replier ou déplier la barre latérale',
			find: 'Aller au champ de filtre ou de recherche : dans Traitements, les documents d’un projet, sa recherche et son dictionnaire, et ici',
			back: (pageBack: string) =>
				`Revenir à l’écran d’où vous venez, comme le bouton Précédent de la souris. En relecture, ${pageBack} tourne plutôt à la page précédente.`,
			escape: 'Fermer un menu ou une boîte de dialogue',
			start: 'Lancer le traitement',
			addLinks: 'Ajouter les liens YouTube collés',
			newLine: 'Commencer une autre ligne de liens',
			reviewed: 'Marquer la page comme relue, ou non relue ; pendant une correction, l’enregistrer',
			nextPage: 'Page ou partie suivante',
			previousPage: 'Page ou partie précédente',
			nextDifference: 'Passage suivant où les modèles divergent, ses versions ouvertes',
			previousDifference: 'Passage précédent où les modèles divergent',
			closeReadings: 'Fermer les versions',
			moveReadings: 'Écarter les versions, depuis la poignée à leur gauche ; avec Maj, plus loin',
			splitter:
				'Déplacer la limite entre deux volets, une fois qu’elle a le focus ; avec Maj, plus loin',
			search: 'Chercher dans le texte, ou revenir au champ de recherche',
			nextMatch: 'Occurrence suivante',
			previousMatch: 'Occurrence précédente',
			closeSearch: 'Fermer la liste des occurrences, puis la recherche',
			zoomIn: 'Agrandir',
			zoomOut: 'Réduire',
			fit: 'Ajuster la page à la largeur du volet'
		},
		filter: 'Chercher un mot',
		noMatch: 'Aucun mot de ce nom.',
		seeAlso: 'Voir aussi',
		whatIs: (word: string) => `Ce que l’on entend par « ${word} »`,
		openGlossary: 'Ouvrir le glossaire',
		openPrompts: 'Voir et modifier les prompts',
		terms: {
			ocr: {
				word: 'OCR et HTR',
				text: 'La reconnaissance optique de caractères (OCR) transforme le texte imprimé d’une image en un texte que l’on peut chercher et modifier ; la reconnaissance d’écriture manuscrite (HTR) en fait autant pour l’écriture à la main. Kalamu fait l’une et l’autre avec des modèles d’IA qui lisent la page entière d’un coup, plutôt que lettre par lettre comme les anciens moteurs d’OCR. Ils s’en sortent mieux avec les colonnes, l’impression pâlie et les écritures difficiles, mais peuvent aussi se tromper de façons nouvelles (voir Hallucination). Pour l’imprimé net, Kalamu propose aussi Tesseract, un moteur d’OCR de l’ancienne sorte, sur votre propre ordinateur.'
			},
			llm: {
				word: 'LLM (grand modèle de langage)',
				text: 'Un modèle entraîné sur de très grandes quantités de texte pour écrire la suite la plus probable de ce qu’on lui donne. Les modèles Gemini, GPT, Claude et ceux de Mistral sont des LLM qui lisent aussi les images et le son. Ce qu’ils écrivent est fluide et souvent juste, mais jamais garanti : c’est un texte probable, pas une copie.'
			},
			whisper: {
				word: 'Transcription hors ligne (Whisper)',
				text: 'Whisper est un modèle ouvert de reconnaissance de la parole que Kalamu exécute sur votre propre ordinateur, après un téléchargement unique dans Paramètres. Rien n’est envoyé nulle part et rien n’est facturé. Il est plus lent que les modèles en ligne et ne sait pas distinguer les locuteurs.'
			},
			tesseract: {
				word: 'Extraction de texte hors ligne (Tesseract)',
				text: 'Tesseract est un moteur d’OCR ouvert que Kalamu exécute sur votre propre ordinateur. Téléchargez une fois dans Paramètres chaque langue que vous lisez ; ensuite, rien n’est envoyé nulle part et rien n’est facturé. Il lit bien l’imprimé net et note l’emplacement de chaque mot sur la page, ce qui donne des fichiers ALTO XML et des PDF interrogeables. L’écriture manuscrite, les pages pâlies ou abîmées et les mises en page complexes se lisent mieux avec un modèle d’IA. Il ne détecte pas la langue, vous la choisissez donc, et sa vitesse dépend du processeur de votre ordinateur.'
			},
			apiKey: {
				word: 'Clé d’API',
				text: 'Un code délivré par un fournisseur d’IA (Google, OpenAI, Anthropic, Mistral) qui permet à Kalamu d’envoyer du travail sous votre compte ; le fournisseur facture ce qui est utilisé. Kalamu conserve les clés dans le trousseau du système (le Gestionnaire d’identification sous Windows, le trousseau sur Mac) et envoie chacune seulement au fournisseur qui l’a délivrée.'
			},
			prompt: {
				word: 'Prompt',
				text: 'Les instructions envoyées au modèle avec chaque page, enregistrement ou texte : quoi transcrire, comment le mettre en forme, dans quelle langue, quoi laisser de côté. Kalamu est livré avec des prompts rédigés pour les types de documents courants. Dans Prompts (menu de gauche), ils sont listés par tâche : lisez ce qui est envoyé, adaptez-les à vos documents, ou écrivez le vôtre avec Nouveau prompt. Nouveau traitement propose ceux de la tâche choisie. Les modèles spécialisés d’OCR et de reconnaissance de la parole, comme Mistral OCR, Voxtral et Whisper, ne prennent pas de prompt.'
			},
			setup: {
				word: 'Combinaison et comparaison',
				text: 'Une combinaison, c’est un fournisseur, un modèle et un prompt appliqués aux fichiers d’un traitement. Un traitement qui en compte deux ou plus les compare : en relecture, chacune est désignée par une lettre (son sigle), et les passages où elles lisent différemment sont mis en évidence, comme une édition critique note les variantes de ses témoins.'
			},
			pipeline: {
				word: 'Configuration',
				text: 'Les réglages d’un traitement enregistrés sous un nom : tâche, modèles, prompts, langues, formats et destination, pour que le traitement suivant démarre en un clic. Exportée, une configuration emporte ses prompts complets et une notice Markdown à citer dans une section méthodologique ou à partager avec des collègues. Plusieurs configurations exécutées l’une après l’autre forment une chaîne de traitement.'
			},
			workflow: {
				word: 'Chaîne de traitement',
				text: 'Plusieurs traitements exécutés l’un après l’autre sur les mêmes fichiers, chacun appelé étape : lire le texte des numérisations, le vérifier, puis y relever les personnes et les lieux. Une étape porte sur les fichiers ou sur le texte produit par une étape précédente, tel que vous l’avez corrigé. Là où vous avez demandé à vérifier, Kalamu attend que vous cliquiez sur Continuer, sur la page du traitement ou dans sa relecture, pour que l’étape suivante lise le texte que vous avez validé. Choisissez « Ensuite » en bas de Nouveau traitement pour la plus simple, ou composez-en de plus longues dans Chaînes de traitement.'
			},
			tokens: {
				word: 'Tokens',
				text: 'Les unités dans lesquelles les modèles comptent le texte et les fournisseurs le facturent. Un token est un fragment de mot, environ quatre caractères d’anglais en moyenne ; les autres langues et écritures en demandent souvent davantage. Les images et le son se comptent aussi en tokens. Le coût estimé avant un traitement, comme le coût constaté après, découle du prix par million de tokens lus et écrits du fournisseur.'
			},
			reasoning: {
				word: 'Réflexion (effort de raisonnement)',
				text: 'Le travail que certains modèles consacrent à une question avant d’y répondre. Les étapes intermédiaires sont facturées comme des tokens écrits, mais ne sont pas montrées. Plus d’effort peut aider pour les jugements difficiles, comme une analyse ou une écriture ardue, mais coûte plus cher et prend plus de temps. Chaque modèle propose ses propres niveaux ; Nouveau traitement suggère le plus bas pour la lecture des pages et des enregistrements, et vous pouvez en choisir un autre pour chaque modèle sous Réflexion. Le niveau utilisé est consigné dans la provenance.'
			},
			economy: {
				word: 'Mode économique (par lots)',
				text: 'Envoie un traitement au service de traitement par lots du fournisseur, qui répond dans la journée pour environ la moitié du prix. Kalamu récupère les réponses à sa prochaine ouverture ; l’application n’a pas besoin de rester ouverte entre-temps.'
			},
			groundTruth: {
				word: 'Vérité terrain',
				text: 'Un texte tenu pour exact, par rapport auquel on mesure le texte produit par un modèle. Dans Kalamu, chaque page que vous corrigez, ou que vous marquez comme relue sans la modifier, devient la vérité terrain de cette page.'
			},
			errorRates: {
				word: 'Taux d’erreur par caractère et par mot (CER, WER)',
				text: 'La part des caractères, ou des mots, qu’un modèle a mal lus par rapport à la vérité terrain : ajoutés, omis ou remplacés, rapportés à la longueur du texte exact. Un taux d’erreur par caractère de 2 % correspond à environ un caractère erroné sur cinquante. Kalamu ne les calcule que sur les pages que vous avez relues : ils montrent les erreurs que vous avez repérées, pas celles qui vous ont échappé.'
			},
			hallucination: {
				word: 'Hallucination',
				text: 'Un texte qu’écrit un modèle et qui ne figure pas dans la source : un mot plausible là où la page est illisible, une phrase que personne n’a prononcée, un nom qu’une analyse puise ailleurs. La fluidité d’un texte ne prouve pas son exactitude. La vérification d’un échantillon sur la source, que permet la relecture, en est la garantie.'
			},
			provenance: {
				word: 'Provenance',
				text: 'La trace de la façon dont un résultat a été produit. À côté de chaque résultat, Kalamu écrit un fichier provenance.json avec le modèle et sa version exacte, le prompt, les réglages de chaque requête, les dates, les coûts, les pages corrigées ou relues et, pour un document d’un projet, son titre et la description de sa source, pour que la méthode puisse être décrite et reproduite.'
			},
			machineReadable: {
				word: 'Lisible par machine',
				text: 'Sous une forme qu’un logiciel peut traiter sans qu’une personne la lise. Le JSON et le CSV sont lisibles par machine ; un document Word beaucoup moins, et une numérisation pas du tout. Choisissez-les quand les résultats seront comptés, interrogés ou importés dans une base de données.'
			}
		},
		formats: {
			txt: 'Texte brut, lisible partout, le plus simple à interroger ou à reprendre dans d’autres outils. Un repère indique le début de chaque page.',
			markdown:
				'Du texte avec un balisage léger pour les titres et les listes, lisible tel quel et bien adapté aux outils de prise de notes comme Obsidian.',
			docx: 'Un document Word avec un titre pour chaque page, à lire, annoter ou partager avec des collègues qui travaillent sous Word.',
			json: 'Chaque page, segment, réponse ou entrée avec ses détails, lisible par machine, pour les scripts et les bases de données.',
			csv: 'Un tableau avec une ligne par page, segment, document ou entrée (une analyse a une colonne par champ), pour Excel, LibreOffice ou un logiciel de statistiques.',
			srt: 'Des sous-titres avec leur minutage, pour les lecteurs vidéo et les logiciels de montage.',
			vtt: 'Des sous-titres pour le web (WebVTT), tels que les utilisent les lecteurs vidéo en ligne.',
			searchable_pdf:
				'Le PDF d’origine avec une couche texte invisible, pour pouvoir y chercher et en copier le texte. Seulement avec les modèles qui indiquent l’emplacement du texte sur la page : pour l’instant, Tesseract et Mistral OCR.',
			alto: 'Le texte de chaque page avec l’emplacement de chaque bloc, ligne et mot, au format ALTO que lisent les bibliothèques numériques, eScriptorium et Transkribus. Seulement avec les modèles qui indiquent l’emplacement du texte sur la page : Tesseract place chaque mot ; Mistral OCR place les paragraphes, dont les lignes et les mots sont alors estimés. Les corrections gardent l’emplacement de leurs mots.',
			provenance:
				'Toujours écrit à côté des résultats : comment ils ont été produits (voir Provenance).'
		}
	},
	support: {
		title: 'Soutenir le développement',
		note: 'Kalamu est un projet de {author} (Université de Bayreuth). S’il vous est utile, vous pouvez contribuer à financer son développement. Ce soutien est entièrement facultatif et ne débloque aucune fonction supplémentaire.',
		link: 'Offrez-moi un café'
	},
	consent: {
		title: 'Envoyer vos fichiers à un fournisseur d’IA ?',
		ask: (providers: string, policy: string) =>
			`Kalamu va envoyer le contenu de vos fichiers à ${providers} : les pages, les enregistrements ou le texte, avec les noms et toute autre donnée personnelle qu’ils contiennent. Chaque fournisseur les lit avec ses modèles d’IA, sous votre propre compte et votre clé d’API, et les conserve selon ses propres conditions. Le développeur de Kalamu ne reçoit rien.\n\nCette question n’est posée qu’une fois par fournisseur. Politique de confidentialité de Kalamu : ${policy}`,
		send: 'Envoyer'
	},
	access: {
		ask: (folder: string) =>
			`Pour enregistrer les résultats à côté de vos fichiers, Kalamu a besoin de votre autorisation pour le dossier « ${folder} ». Choisissez ce dossier dans la fenêtre suivante.`,
		choose: 'Choisir le dossier…',
		missing: (folder: string) =>
			`Kalamu ne peut pas enregistrer dans « ${folder} ». Choisissez ce dossier quand il vous est demandé, ou choisissez un seul dossier pour tous les résultats sous Résultats.`
	},
	format: {
		txt: 'Texte (.txt)',
		markdown: 'Markdown (.md)',
		docx: 'Word (.docx)',
		json: 'JSON',
		srt: 'Sous-titres (.srt)',
		vtt: 'Sous-titres (.vtt)',
		csv: 'Tableur (.csv)',
		searchable_pdf: 'PDF interrogeable',
		alto: 'ALTO XML (position des mots)'
	},
	close: {
		title: 'Des traitements sont en cours',
		body: (tray: string) =>
			`Ils peuvent continuer en arrière-plan, avec Kalamu dans ${tray}. Ou quittez maintenant : les traitements sont mis en pause, les pages terminées sont conservées, et vous les reprendrez la prochaine fois depuis Traitements.`,
		background: 'Continuer en arrière-plan',
		quit: 'Mettre en pause et quitter',
		stay: 'Annuler',
		waiting: (n: number) =>
			n === 1
				? 'En attente d’1 segment déjà envoyé. Il est payé : Kalamu l’enregistre avant de se fermer, ce qui prend quelques minutes au plus.'
				: `En attente de ${n} segments déjà envoyés. Ils sont payés : Kalamu les enregistre avant de se fermer, ce qui prend quelques minutes au plus.`,
		quitNow: 'Quitter maintenant',
		hidden: (reopen: string) =>
			`Kalamu travaille toujours. Cliquez sur son icône dans ${reopen} pour le rouvrir.`
	},
	os: {
		windows: {
			keychain: 'le Gestionnaire d’identification Windows',
			tray: 'la zone de notification, à côté de l’horloge',
			reopen: 'la zone de notification'
		},
		macos: {
			keychain: 'le trousseau de macOS',
			tray: 'la barre des menus',
			reopen: 'la barre des menus ou le Dock'
		}
	},
	keys: {
		ctrl: 'Ctrl',
		shift: 'Maj',
		alt: 'Alt',
		escape: 'Échap',
		pageUp: 'Pg préc',
		pageDown: 'Pg suiv'
	},
	tray: {
		open: 'Ouvrir Kalamu',
		pause: 'Mettre en pause tous les traitements',
		quit: 'Quitter',
		quitPausing: 'Mettre en pause et quitter',
		idle: 'Aucun traitement en cours',
		running: (jobs: number, percent: number) =>
			`${jobs === 1 ? '1 traitement en cours' : `${jobs} traitements en cours`} · ${percent} %`
	},
	common: {
		cancel: 'Annuler',
		undo: 'Annuler',
		save: 'Enregistrer',
		close: 'Fermer',
		dismissMessage: 'Fermer ce message',
		loading: 'Chargement…',
		colon: ' :',
		semicolon: ' ; ',
		quoted: (text: string) => `« ${text} »`,
		sources: 'Sources',
		unknown: 'Inconnu',
		preview: 'préversion',
		retiring: 'bientôt retiré',
		opensInBrowser: ' (s’ouvre dans votre navigateur)',
		linkFailed: (url: string) =>
			`Cette page n’a pas pu être ouverte dans votre navigateur : ${url}`,
		unsaved: 'Vos modifications ne sont pas enregistrées. Quitter et les abandonner ?',
		discardChanges: 'Abandonner les modifications',
		keepEditing: 'Continuer à modifier'
	},
	error: {
		'search.too_short': 'Tapez un mot d’au moins trois lettres.',
		'search.stopped': 'La recherche a été remplacée par une plus récente.',
		'search.index':
			'La recherche n’a pas pu lire sa copie du texte de vos résultats. Préparez à nouveau le texte depuis la recherche ; vos résultats ne sont pas touchés.',
		'project.not_found': 'Ce projet, ou ce à quoi il renvoie, n’existe plus.',
		'project.elsewhere': 'Cet élément appartient à un autre projet, ou n’existe plus.',
		'project.no_name': 'Donnez-lui un nom.',
		'project.no_tag': 'Tapez l’étiquette à ajouter.',
		'project.export': 'L’export n’a pas pu être écrit.',
		'dictionary.no_entry':
			'Cette entrée ne figure plus dans le dictionnaire. Elle a peut-être été fusionnée ou scindée.',
		'dictionary.no_label': 'Donnez un libellé à l’entrée.',
		'dictionary.namespaces':
			'Ces lignes proviennent de champs différents, des personnes et des lieux par exemple, et restent distinctes.',
		'dictionary.kept_apart':
			'Vous avez décidé de garder certaines de ces lignes distinctes. Revenez d’abord sur cette décision pour les fusionner.',
		'dictionary.too_few': 'Choisissez au moins deux lignes.',
		'dictionary.same_entry': 'Elles forment déjà une seule entrée. Retirez-en d’abord une.',
		'dictionary.nothing_to_compare': 'Ce champ compte moins de deux lignes à comparer.',
		'unit.pages_empty': 'Indiquez au moins une page.',
		'unit.pages_unreadable':
			'Utilisez des numéros et des étendues de pages, comme 1, 3-5, 8 (gauche).',
		'unit.no_such_page': 'Ce document n’a pas de telle page.',
		'unit.not_contents': 'Seule une table des matières comporte des pièces.',
		'unit.already_made': 'Les pièces ont déjà été créées à partir de ces entrées.',
		'unit.no_pages': 'Indiquez d’abord les pages de cette pièce.',
		'unit.not_pdf': 'Pour l’instant, seules les pièces d’un PDF peuvent être lues séparément.',
		'unit.not_joinable':
			'Seules deux pièces différentes de la même table des matières peuvent être réunies.',
		'unit.other_result':
			'Cette pièce appartient à une autre table des matières. Ouvrez-la depuis celle-ci.',
		'unit.bad_split': 'Coupez une pièce entre deux de ses pages.',
		'unit.none_chosen': 'Choisissez au moins une pièce.',
		'job.not_found': 'Ce traitement n’existe plus.',
		'job.not_text':
			'Seuls le texte extrait, les transcriptions et le texte corrigé peuvent être analysés de cette façon.',
		'job.nothing_read':
			'Rien n’a encore été lu : il n’y a donc rien à analyser. Lancez le traitement, ou attendez ses premières pages.',
		'job.database': 'La base de données des traitements n’a pas pu être lue ou écrite.',
		'job.running': 'Ce traitement est déjà en cours.',
		'job.empty': 'Ajoutez au moins un fichier et choisissez un modèle.',
		'job.link_needs_folder':
			'Choisissez un dossier pour les résultats : une vidéo YouTube n’a pas de dossier à elle à côté duquel les écrire.',
		'source.private':
			'Cette vidéo est privée : elle ne peut pas être lue. Seules les vidéos publiques peuvent être transcrites.',
		'source.not_found': 'YouTube n’a pas de vidéo à ce lien : elle a peut-être été supprimée.',
		'source.not_youtube':
			'Ce n’est pas le lien d’une vidéo YouTube. Utilisez un lien watch, youtu.be, Shorts ou en direct ; les playlists et les chaînes ne peuvent pas être lues.',
		'media.encrypted':
			'Ce PDF est protégé par un mot de passe. Retirez le mot de passe, puis ajoutez de nouveau le fichier.',
		'media.unsupported': 'Ce type de fichier n’est pas encore pris en charge.',
		'media.read': 'Le fichier n’a pas pu être lu.',
		'prompt.invalid':
			'Le prompt n’a pas pu être enregistré : il lui faut un nom et des instructions.',
		'prompt.no_fields': 'Une analyse demande au moins un champ.',
		'prompt.unnamed_field': 'Chaque champ doit avoir un nom.',
		'prompt.duplicate_field': 'Deux champs portent le même nom.',
		'prompt.few_values': 'Un choix demande au moins deux valeurs entre lesquelles choisir.',
		'prompt.unnamed_pipeline': 'Une configuration demande un nom et au moins un modèle.',
		'workflow.incomplete':
			'Une chaîne de traitement demande un nom et au moins une étape, chacune avec un modèle.',
		'workflow.no_steps': 'Une chaîne de traitement demande au moins une étape.',
		'workflow.not_earlier': 'Une étape ne peut lire qu’une étape qui la précède.',
		'workflow.not_text':
			'Une étape ne peut lire que le texte d’une extraction, d’une transcription ou d’une correction.',
		'workflow.cannot_read_text':
			'La tâche de cette étape ne lit pas de texte : faites-lui plutôt lire les fichiers.',
		'workflow.not_finished': 'Cette étape n’est pas encore terminée.',
		'profile.no_name': 'Un profil d’export a besoin d’un nom.',
		'profile.no_columns': 'Un profil d’export a besoin d’au moins une colonne.',
		'profile.unnamed_column': 'Chaque colonne d’un profil d’export a besoin d’un intitulé.',
		'profile.bad_heading': 'L’intitulé d’une colonne doit tenir sur une ligne.',
		'profile.duplicate_heading': 'Deux colonnes ont le même intitulé.',
		'profile.bad_separator':
			'Le séparateur de liste ne peut être vide ni contenir de retour à la ligne.',
		'profile.bad_value': 'Une colonne demande une valeur que Kalamu ne peut pas donner.',
		'prompt.invalid_file':
			'Ce n’est pas un fichier de prompts, de configuration ou de chaîne de traitement Kalamu, ou il provient d’une version plus récente de Kalamu.',
		'provider.unknown': 'Fournisseur inconnu.',
		'provider.missing_key':
			'Aucune clé d’API n’est enregistrée pour ce fournisseur. Ajoutez-en une dans Paramètres.',
		'provider.consent':
			'Kalamu n’a pas l’autorisation d’envoyer vos fichiers à ce fournisseur. Recommencez et choisissez Envoyer quand la question est posée.',
		'provider.unsupported': 'Ce modèle ne sait pas effectuer cette tâche.',
		'source.unsupported':
			'L’un des fichiers ne peut pas être lu pour cette tâche. Retirez-le, ou choisissez une autre tâche.',
		'provider.auth': 'Le fournisseur a refusé la clé d’API. Vérifiez-la dans Paramètres.',
		'provider.quota': 'Votre compte chez ce fournisseur n’a plus de crédit ni de quota.',
		'provider.rate_limited':
			'Le fournisseur a demandé à Kalamu de ralentir ; la page sera relancée.',
		'provider.transient': 'Le fournisseur n’a pas pu être joint.',
		'provider.blocked': 'Le fournisseur a refusé de renvoyer le texte de cette page.',
		'provider.invalid_request': 'Le fournisseur a rejeté la requête.',
		'provider.other': 'Le fournisseur a envoyé une réponse inattendue.',
		'provider.invalid_answer':
			'La réponse du modèle ne correspondait pas aux champs de l’analyse ; Kalamu lui repose la question.',
		'provider.cut_off':
			'La réponse s’est arrêtée à la limite de sortie du modèle avant d’être complète. Envoyez moins de pages par appel, demandez des champs moins nombreux ou plus courts, ou choisissez un modèle capable d’écrire une réponse plus longue.',
		'provider.not_installed':
			'Ce modèle hors ligne ou ses données de langue n’ont pas encore été téléchargés. Téléchargez-les dans Paramètres.',
		'provider.interrupted': 'Arrêté avant la fin ; il reprendra quand le traitement reprendra.',
		'offline.unknown_model': 'Ce modèle hors ligne est inconnu de cette version de Kalamu.',
		'offline.busy': 'Un autre modèle est en cours de téléchargement ; attendez qu’il se termine.',
		'offline.in_use': 'Un traitement en cours utilise ce modèle ; mettez-le d’abord en pause.',
		'offline.cancelled': 'Téléchargement annulé.',
		'offline.download':
			'Le modèle n’a pas pu être téléchargé. Vérifiez votre connexion internet et réessayez.',
		'offline.integrity': 'Le modèle téléchargé était endommagé et a été supprimé. Réessayez.',
		'offline.io': 'Le fichier du modèle n’a pas pu être lu ou écrit.',
		'output.write': 'Les fichiers de résultats n’ont pas pu être écrits.',
		'keys.keychain': 'Le trousseau du système a refusé la demande.',
		'keys.format':
			'Cette clé contient un caractère qu’aucune clé d’API ne comporte : elle n’a pas été enregistrée.',
		'settings.write': 'Les paramètres n’ont pas pu être enregistrés.',
		'catalog.download': 'La liste des modèles n’a pas pu être téléchargée.',
		'catalog.invalid': 'La liste des modèles téléchargée n’a pas pu être lue.',
		'file.read': 'Le fichier n’a pas pu être lu ou écrit.',
		'file.open': 'Ce fichier n’a pas pu être ouvert. Il a peut-être été déplacé ou supprimé.',
		'file.missing':
			'Le fichier n’est plus là où il a été ajouté : ses pages ne peuvent pas être affichées. Ses résultats sont conservés ; remettez-le à sa place pour le voir ici.',
		'app.internal': 'Une erreur s’est produite dans l’application.',
		'app.notDesktop': 'Cette fonction n’est disponible que dans l’application de bureau.'
	}
};
