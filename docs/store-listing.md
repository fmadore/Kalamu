# Store listing text

Paste-ready text for Kalamu's two store listings, kept for the **next submission**, version
1.5.0: the Microsoft Store (Partner Center) first, whose `Kalamu_1.5.0.msixbundle` the release
workflow builds (the first, 1.0.0, was certified on 2 November 2026), in
[English](#english-united-states) and [French](#french-france), then the
[Mac App Store](#mac-app-store-app-store-connect) (App Store Connect), whose first submission is
the version current when a Mac is at hand (1.4.1 was prepared but has not been sent). The submission and update steps are in [microsoft-store.md](microsoft-store.md) and
[mac-app-store.md](mac-app-store.md). Check every block against the app as packaged before
pasting: a listing that promises what a reviewer cannot find fails certification.

Every paragraph inside a block is one unwrapped line, because Partner Center keeps line breaks;
blank lines separate paragraphs. Keep it that way when editing, and do not let a formatter
re-wrap the blocks.

Microsoft's [listing field documentation](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)
allows 10,000 characters for Description, 1,500 for What's new, and 20 features of 200
characters each. Short description accepts 1,000 characters, but only the first 270 show in
some views; the one below fits. Keywords allow up to 7 entries of at most 40 characters, with
no more than 21 words across all of them.

**Languages.** English (United States) and, from 1.5.0, French (France), with the French
screenshots in [store/screenshots/windows/fr/](store/screenshots/windows/fr/). In Partner Center,
add French (France) under **Store listings** → *Manage additional languages*, then fill it from
[French (France)](#french-france) below. German follows once the interface is translated.

**Before pasting**, replace the provider model names if the catalogue has changed, and check
that the features listed still match the app.

## English (United States)

### Description — full text

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with the AI provider of your choice, then check every result against its source. Kalamu works with Google Gemini, OpenAI, Anthropic Claude and Mistral under your own account and API key, so their usage charges apply and are billed by them; the app shows an estimated cost before anything is sent. Recordings can also be transcribed offline with Whisper, and printed pages read offline with Tesseract or, for difficult print, a small OCR model, free, after a one-time download.

Extract text from PDFs and images, printed or handwritten: newspapers, reports, letters, registers and manuscripts. Keep the lines as on the page, to check against the scan, or join them into running text for reading and searching. Before a job starts, Kalamu checks the files and suggests fixes: double pages to split, scans whose resolution is too low, recordings that are too quiet.

Transcribe recordings and videos with speakers and timestamps, and give the speakers their real names. Long recordings are cut at pauses and sent piece by piece. Offline, Whisper transcribes on your PC with no account, key or internet connection, and nothing leaves the computer; give it the names and words to spell, or have it write the speech in English. Speaker labels need a cloud model. With Gemini, public YouTube videos are transcribed from their link, without downloading them.

Make a detailed table of contents of a long document: a newspaper or magazine issue, an archival file scanned in one piece, a book or report. One model reads each page, another, cheaper one combines the pages into one entry per article or item, with its pages and the fields you choose. Correct the noisy text that digital libraries supply, page by page, with every change shown against the original.

Analyse texts with fields you define, as you would brief a research assistant: a summary, the people, places and organisations a document names, its tone with a reason, themes from your own list. Ready-made analyses cover common needs, including the question, argument, methods and findings of a scholarly publication for a literature review. Answers come back in the shape you asked for and can be exported as a spreadsheet with a column per field.

Check every result beside its source: the page next to its text, or the recording next to its transcript, line by line. Corrections are kept apart from the AI output, which is never overwritten, and you mark what you have reviewed. Search a document or a recording across all its pages, corrections included, and go to each match. A glossary explains the words these tools use, from OCR and tokens to error rates and hallucination, for colleagues new to them. Compare two or more models on the same documents: their differences are highlighted in the text, with error rates against the pages you have corrected.

Large jobs are safe and affordable. Work is saved piece by piece: a job interrupted by closing the app, a crash or a lost connection resumes where it stopped, and nothing is sent or paid for twice. Long jobs keep running from the notification area while the window is closed. For large collections, choose the cheaper mode: the provider's batch service answers within a day at about half the price, and Kalamu collects the results when it next opens.

Export to text, Markdown, Word, JSON, CSV, subtitles (SRT and WebVTT), and searchable PDF and ALTO XML, with Tesseract, Mistral OCR or LightOnOCR. Each output comes with a provenance.json file recording the model, prompt and settings, so your methods can be described and repeated. Save a job's settings as a pipeline to start the next batch in one click, and export it with its full prompts to cite in a methods section or share with colleagues. Chain pipelines into a workflow, extraction then analysis for instance, that stops for your review before the next step reads the text.

Keep a project's material together: documents in collections and tags, each with every result read from it whatever job made it, and the one you prefer marked. Search the text of all its documents at once, corrections included, and open each passage where it stands. A project dictionary gathers the people, places and terms your analyses listed; merge the spellings that are one person under your own label, keep two apart, and let a model propose merges that you accept or turn down.

Before the first job sent to each provider, Kalamu says what will be sent and asks you. Your files go straight from your PC to the provider you choose; offline, nothing leaves the PC. API keys are kept in Windows Credential Manager. The developer receives no files, results, keys or telemetry, and the app sells no subscription, credits or API access. Every result is text generated by the AI model you choose: it can be fluent and wrong, so check a sample against the source before relying on it, and use Report a problem in the review or in Settings to tell the developer about inappropriate output. An optional Buy Me a Coffee link supports development and unlocks nothing.

Requires Windows 11 with the Microsoft Edge WebView2 Runtime, with native x64 and ARM64 packages. The interface is in English and French; documents and recordings can be in any language the chosen model reads. Free and open source under the MIT licence.

Google Gemini and YouTube are trademarks of Google LLC; OpenAI is a trademark of OpenAI OpCo, LLC; Claude is a trademark of Anthropic, PBC; Mistral AI is a trademark of Mistral AI; Microsoft Word is a trademark of the Microsoft group of companies. Kalamu is an independent project, not affiliated with or endorsed by any of them.
```

### What's new in this version — 1.5.0

```text
Read difficult print offline: two small OCR models, LightOnOCR-3 and GLM-OCR, read old newspapers in columns, faded type and tables on your PC, free and with nothing sent, after a one-time download. LightOnOCR places each block on the page, for ALTO and a searchable PDF. Much slower than Tesseract: Settings and New job show how long a page takes on your PC.

Whisper learns the names and words you give it, can write a recording in English, and follows a recording that changes language, with subtitle lines cut where the words fall.

Also: a workflow step from any prompt; rename a name across an analysis's answers; New job names the files it cannot read and keeps a job's name in step with its files; model descriptions in French.
```

The 1.4.1 text, for the record:

```text
Before the first job sent to each AI provider, Kalamu now says what will be sent to it and asks you, once for each provider. Whisper and Tesseract, which send nothing, are never asked about.

A page typed by hand reaches ALTO and the searchable PDF; word positions are no longer guessed and shown as measured; photos taken upside down are turned upright before they are read.

The window no longer freezes while a correction or an export is saved during a running job. Dialogs keep the keyboard inside, and a file dropped with its folder is listed once.
```

The 1.4.0 text, for the record:

```text
Extract text offline with Tesseract, for clean print: free, nothing sent, each word with its place on the page. Download a language once in Settings.

Export ALTO XML with each block, line and word in its place, corrections included, from Tesseract or Mistral OCR.

All jobs: one page to search, filter and sort every job, and to rename, add to a project or delete several at once.

The whole window for every view, larger type, and a smoother review: zoom the scan about the pointer, both texts move together, and the readings box can be moved aside.

Also: write a new prompt from nothing; projects in the sidebar, with jobs dragged onto them; Settings in tabs.
```

The 1.3.0 text, for the record:

```text
In French: choose the interface's language in Settings, or let it follow the system's.

Search a whole project: every passage that holds your words, in the text as you corrected it, each opening its review at that place. The search runs on your PC; nothing is sent.

What to check: one list of the pages and answers that still need you, with the reason for each, from a project, a collection or a selection.

Describe each source as an archive does, and export the descriptions in your own columns (Dublin Core terms included), beside each document's corrected text, review status or analysis answers.

Also: Gemini 3.5 Transcribe names its speakers and shows its cost; OpenAI's prompt caching costs less; CSV files open in Excel with their accents; rename a tag across a project; Report a problem leads with bugs and ideas.
```

The 1.2.0 text, for the record:

```text
Items of a table of contents: turn the entries of a newspaper issue or an archival file into items, correct, join or split them, then extract, analyse or export one article or letter on its own.

Simpler prompts: one default per kind of material, for printed, newspaper and handwritten pages in any language and reading direction, Arabic and Ajami included. Your saved pipelines and workflows keep working.

Your jobs and settings now live in a folder named Kalamu Data, moved there once on first start; Settings shows where.

Also: New job shows which models also give a searchable PDF; Mistral Large 4 at its current, lower price; clearer wording across the interface.
```

The 1.1.2 text, for the record:

```text
Projects: gather documents in collections and tags, each with every result read from it, whatever job made it, and the one you prefer.

A project dictionary of the people, places and terms your analyses listed: merge spellings under your own label, keep two apart, export as JSON and CSV. A model can propose merges; nothing is merged until you accept.

Workflows: run pipelines one after another on the same files, stopping for your review before the next step reads the text. Then, at the end of New job, analyses the text once you have checked it.

Also: send any finished text into a new analysis or correction without reading the originals again; several pages per call for text extraction; a thinking level for models that reason; Gemini 3.5 Transcribe, Google's speech-to-text model, for recordings; a menu on each job in the list to rename it, add it to a project or delete it; definitions in a bubble beside each word; corrections opened where you are reading; and fixes to cost counting, workflow steps and zoomed pages.
```

The 1.0.0 text, for the record: first release in the Microsoft Store; search in the review
(Ctrl F); the glossary; Report a problem; one layout for every review; cut-off answers marked;
changed files refused; files written despite a failing page; the third-party licences.

### Features — one feature per line

```text
Extract text from scanned and handwritten PDFs and images with Gemini, OpenAI, Claude or Mistral
Transcribe recordings and videos with timestamps, in the cloud with speakers or offline
Offline, with no account, key or cost: transcription with Whisper, and text extraction with Tesseract or small OCR models
Transcribe public YouTube videos from their link with Gemini, without downloading them
Analyse texts with your own fields or ready-made ones: people, places, tone, a publication's argument
Make detailed tables of contents of newspaper issues, archival files and books, then read each article on its own
Correct existing OCR text page by page, every change shown against the original
Check every result beside its source, page by page or line by line; corrections never overwrite the AI's text
Search a document, a recording or a whole project at once, corrections included, and open each passage in its source
Projects: collections and tags of documents, each with every result read from it
A project dictionary of the names your analyses found, with merges a model proposes and you decide
Compare models on the same documents, with differences highlighted and error rates
Estimated cost shown before anything is sent, from page counts and published prices
Cheaper mode: send a large job to the provider's batch service at about half the price
Interrupted jobs resume where they stopped, nothing paid twice; long jobs run with the window closed
Save a job's settings as a pipeline, or chain pipelines into a workflow with review stops
Export to text, Markdown, Word, JSON, CSV, SRT, WebVTT, searchable PDF and ALTO XML, each with a provenance file
File checks for double pages, low resolution and quiet recordings, with suggested fixes
API keys kept in Windows Credential Manager; files go straight to the provider you choose
Light and dark themes; native Windows x64 and ARM64; free and open source
```

### Short description

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with Gemini, OpenAI, Claude or Mistral under your own key, or offline with Whisper, Tesseract and OCR models. Check every result beside its source; corrections never overwrite the AI's text.
```

### Keywords — up to 7

```text
handwritten text recognition
OCR
interview transcription
speech to text
qualitative research
digital humanities
historical documents
```

## Additional system requirements

Enter each line as a separate requirement (under 200 characters):

```text
Windows 11 with the Microsoft Edge WebView2 Runtime; native x64 and ARM64 packages
Cloud features: internet and your own Gemini, OpenAI, Anthropic or Mistral account and API key; the provider bills usage
Offline transcription: internet once to download a Whisper model (about 60 MB to 1.1 GB), then runs on the CPU
Offline text extraction: internet once to download each language for Tesseract (about 3 to 15 MB; 100 MB for languages in Latin letters without their own model), then runs on the CPU
Offline OCR models: internet once to download a model (about 1.1 or 1.4 GB), then runs on the CPU, from tens of seconds to over a minute a page
```

Recommended memory: 8 GB; Whisper's larger models and the OCR models need more, and their speed
depends on the processor.

## French (France)

The same listing in French, for Partner Center's French (France) entry. Names in the text are
those of the French interface (*Nouveau traitement*, *Chaînes de traitement*, *Signaler un
problème*), so the listing and the app use the same words. The limits are those of the English
fields; the text here fits them.

### Description — texte complet

```text
Lisez des documents numérisés et manuscrits, transcrivez des entretiens et analysez des textes avec le fournisseur d’IA de votre choix, puis vérifiez chaque résultat sur sa source. Kalamu fonctionne avec Google Gemini, OpenAI, Anthropic Claude et Mistral, avec votre propre compte et votre propre clé d’API : leurs frais d’utilisation s’appliquent et vous sont facturés par eux, et l’application affiche une estimation du coût avant tout envoi. Les enregistrements peuvent aussi être transcrits hors ligne avec Whisper, et les pages imprimées lues hors ligne avec Tesseract ou, pour l’imprimé difficile, un petit modèle OCR, gratuitement, après un téléchargement unique.

Extrayez le texte de PDF et d’images, imprimés ou manuscrits : journaux, rapports, lettres, registres et manuscrits. Gardez les lignes comme sur la page, pour les comparer au scan, ou réunissez-les en texte suivi pour la lecture et la recherche. Avant de lancer un traitement, Kalamu vérifie les fichiers et propose des corrections : doubles pages à séparer, scans de trop faible résolution, enregistrements trop faibles.

Transcrivez des enregistrements et des vidéos avec les locuteurs et l’horodatage, et donnez aux locuteurs leur vrai nom. Les longs enregistrements sont coupés aux pauses et envoyés morceau par morceau. Hors ligne, Whisper transcrit sur votre PC sans compte, sans clé et sans connexion internet, et rien ne quitte l’ordinateur ; donnez-lui les noms et les mots à orthographier, ou faites-lui écrire les propos en anglais. Distinguer les locuteurs demande un modèle en ligne. Avec Gemini, les vidéos YouTube publiques sont transcrites à partir de leur lien, sans être téléchargées.

Dressez la table des matières détaillée d’un long document : un numéro de journal ou de revue, un dossier d’archives numérisé d’un seul tenant, un livre ou un rapport. Un modèle lit chaque page, un autre, moins cher, réunit les pages en une entrée par article ou par pièce, avec ses pages et les champs que vous choisissez. Corrigez page par page le texte bruité que fournissent les bibliothèques numériques, chaque modification montrée face à l’original.

Analysez des textes avec les champs que vous définissez, comme vous le feriez pour un assistant de recherche : un résumé, les personnes, lieux et organisations qu’un document nomme, sa tonalité avec une justification, des thèmes tirés de votre propre liste. Des analyses toutes prêtes couvrent les besoins courants, dont la question, l’argument, les méthodes et les résultats d’une publication scientifique pour une revue de littérature. Les réponses reviennent dans la forme demandée et s’exportent en tableur, une colonne par champ.

Vérifiez chaque résultat à côté de sa source : la page à côté de son texte, ou l’enregistrement à côté de sa transcription, ligne par ligne. Les corrections sont conservées à part du texte produit par l’IA, qui n’est jamais écrasé, et vous marquez ce que vous avez relu. Cherchez dans toutes les pages d’un document ou d’un enregistrement, corrections comprises, et allez à chaque occurrence. Un glossaire explique les mots de ces outils, de l’OCR et des tokens aux taux d’erreur et aux hallucinations, pour les collègues qui les découvrent. Comparez deux modèles ou plus sur les mêmes documents : leurs différences sont surlignées dans le texte, avec les taux d’erreur par rapport aux pages que vous avez corrigées.

Les gros traitements sont sûrs et abordables. Le travail est enregistré morceau par morceau : un traitement interrompu par la fermeture de l’application, un plantage ou une coupure de connexion reprend là où il s’est arrêté, et rien n’est envoyé ni payé deux fois. Les longs traitements continuent depuis la zone de notification, fenêtre fermée. Pour les grandes collections, choisissez le mode économique : le service par lots du fournisseur répond dans la journée pour environ la moitié du prix, et Kalamu récupère les résultats à sa prochaine ouverture.

Exportez en texte, Markdown, Word, JSON, CSV, sous-titres (SRT et WebVTT), PDF interrogeable et ALTO XML, avec Tesseract, Mistral OCR ou LightOnOCR. Chaque export est accompagné d’un fichier provenance.json qui consigne le modèle, le prompt et les réglages, pour décrire et reproduire votre méthode. Enregistrez les réglages d’un traitement comme configuration pour lancer le lot suivant en un clic, et exportez-la avec ses prompts complets pour la citer dans une section de méthodologie ou la partager avec vos collègues. Enchaînez des configurations dans une chaîne de traitement, par exemple l’extraction puis l’analyse, qui s’arrête pour votre relecture avant que l’étape suivante ne lise le texte.

Gardez ensemble le matériau d’un projet : des documents rangés en collections et en étiquettes, chacun avec tous les résultats tirés de lui, quel que soit le traitement qui les a produits, et celui que vous préférez marqué. Cherchez dans le texte de tous ses documents à la fois, corrections comprises, et ouvrez chaque passage là où il se trouve. Un dictionnaire de projet rassemble les personnes, lieux et termes relevés par vos analyses ; regroupez sous votre propre libellé les graphies qui désignent une même personne, gardez-en deux séparées, et laissez un modèle proposer des regroupements que vous acceptez ou refusez.

Avant le premier traitement envoyé à chaque fournisseur, Kalamu indique ce qui sera envoyé et vous le demande. Vos fichiers vont directement de votre PC au fournisseur que vous choisissez ; hors ligne, rien ne quitte le PC. Les clés d’API sont conservées dans le Gestionnaire d’identification de Windows. Le développeur ne reçoit ni fichiers, ni résultats, ni clés, ni télémétrie, et l’application ne vend ni abonnement, ni crédits, ni accès à une API. Chaque résultat est un texte produit par le modèle d’IA que vous choisissez : il peut être fluide et faux, alors vérifiez un échantillon sur la source avant de vous y fier, et utilisez Signaler un problème, dans la relecture ou dans les Paramètres, pour signaler au développeur un résultat inapproprié. Un lien facultatif, Offrez-moi un café, soutient le développement et ne débloque rien.

Nécessite Windows 11 avec Microsoft Edge WebView2 Runtime, en versions natives x64 et ARM64. L’interface est en français et en anglais ; les documents et les enregistrements peuvent être dans toute langue que lit le modèle choisi. Gratuit et open source sous licence MIT.

Google Gemini et YouTube sont des marques de Google LLC ; OpenAI est une marque d’OpenAI OpCo, LLC ; Claude est une marque d’Anthropic, PBC ; Mistral AI est une marque de Mistral AI ; Microsoft Word est une marque du groupe Microsoft. Kalamu est un projet indépendant, ni affilié à aucune d’elles ni approuvé par elles.
```

### Nouveautés de cette version — 1.5.0

```text
Lisez l’imprimé difficile hors ligne : deux petits modèles OCR, LightOnOCR-3 et GLM-OCR, lisent sur votre PC les vieux journaux en colonnes, les caractères pâlis et les tableaux, gratuitement et sans rien envoyer, après un téléchargement unique. LightOnOCR place chaque bloc sur la page, pour l’ALTO et le PDF interrogeable. Bien plus lents que Tesseract : les Paramètres et Nouveau traitement indiquent le temps que prend une page sur votre PC.

Whisper apprend les noms et les mots que vous lui donnez, peut écrire un enregistrement en anglais et suit un enregistrement qui change de langue, avec des lignes de sous-titres coupées là où tombent les mots.

Aussi : une étape de chaîne à partir de n’importe quel prompt ; renommer un nom dans toutes les réponses d’une analyse ; Nouveau traitement nomme les fichiers qu’il ne sait pas lire et garde le nom d’un traitement accordé à ses fichiers ; les descriptions des modèles en français.
```

### Fonctionnalités — une par ligne

```text
Extraire le texte de PDF et d’images numérisés ou manuscrits avec Gemini, OpenAI, Claude ou Mistral
Transcrire des enregistrements et des vidéos avec l’horodatage, en ligne avec les locuteurs ou hors ligne
Hors ligne, sans compte, sans clé ni frais : transcription avec Whisper, extraction de texte avec Tesseract ou de petits modèles OCR
Transcrire des vidéos YouTube publiques à partir de leur lien avec Gemini, sans les télécharger
Analyser des textes avec vos propres champs ou des champs tout prêts : personnes, lieux, tonalité, argument d’une publication
Dresser la table des matières détaillée de numéros de journaux, de dossiers d’archives et de livres, puis lire chaque article à part
Corriger page par page un texte OCR existant, chaque modification montrée face à l’original
Vérifier chaque résultat à côté de sa source, page par page ou ligne par ligne ; les corrections n’écrasent jamais le texte de l’IA
Chercher dans un document, un enregistrement ou tout un projet, corrections comprises, et ouvrir chaque passage dans sa source
Projets : des collections et des étiquettes de documents, chacun avec tous les résultats tirés de lui
Un dictionnaire de projet des noms trouvés par vos analyses, avec des regroupements proposés par un modèle et décidés par vous
Comparer des modèles sur les mêmes documents, avec les différences surlignées et les taux d’erreur
Coût estimé affiché avant tout envoi, d’après le nombre de pages et les prix publiés
Mode économique : envoyer un gros traitement au service par lots du fournisseur pour environ la moitié du prix
Les traitements interrompus reprennent là où ils se sont arrêtés, rien n’est payé deux fois ; les longs traitements continuent fenêtre fermée
Enregistrer les réglages d’un traitement comme configuration, ou enchaîner des configurations avec des arrêts pour relecture
Exporter en texte, Markdown, Word, JSON, CSV, SRT, WebVTT, PDF interrogeable et ALTO XML, chacun avec un fichier de provenance
Vérification des fichiers : doubles pages, faible résolution et enregistrements faibles, avec corrections proposées
Clés d’API dans le Gestionnaire d’identification de Windows ; les fichiers vont directement au fournisseur choisi
Thèmes clair et sombre ; versions natives Windows x64 et ARM64 ; gratuit et open source
```

### Description courte

```text
Lisez des documents numérisés et manuscrits, transcrivez des entretiens et analysez des textes avec Gemini, OpenAI, Claude ou Mistral et votre propre clé, ou hors ligne. Vérifiez chaque résultat à côté de sa source ; les corrections n’écrasent jamais le texte de l’IA.
```

### Mots-clés — 7 au plus

```text
reconnaissance d'écriture manuscrite
OCR
transcription d'entretiens
reconnaissance vocale
recherche qualitative
humanités numériques
documents historiques
```

### Configuration requise supplémentaire

```text
Windows 11 avec Microsoft Edge WebView2 Runtime ; versions natives x64 et ARM64
Fonctions en ligne : internet et votre propre compte et clé d’API Gemini, OpenAI, Anthropic ou Mistral ; le fournisseur facture l’utilisation
Transcription hors ligne : internet une fois pour télécharger un modèle Whisper (de 60 Mo à 1,1 Go environ), puis fonctionne sur le processeur
Extraction de texte hors ligne : internet une fois par langue de Tesseract (3 à 15 Mo ; 100 Mo pour les langues en lettres latines sans modèle propre), puis sur le processeur
Modèles OCR hors ligne : internet une fois pour télécharger un modèle (1,1 ou 1,4 Go), puis sur le processeur, de quelques dizaines de secondes à plus d’une minute par page
```

### Légendes des captures d’écran

The French set is in [store/screenshots/windows/fr/](store/screenshots/windows/fr/), under the
same names as the English one; each caption goes in the screenshot's Description field (under
200 characters).

| # | Légende |
| --- | --- |
| 1 | Vérifiez chaque page à côté de son scan. Là où deux modèles lisent différemment, les mots sont surlignés et vous choisissez la bonne version. |
| 2 | Gardez ensemble les documents d’un projet, en collections et en étiquettes, chacun avec tous les résultats tirés de lui, quel que soit le traitement. |
| 3 | Lisez lettres et registres manuscrits ligne par ligne, gardées comme sur la page pour comparer facilement chaque ligne à l’original. |
| 4 | Enchaînez des étapes : lire les pages, les vérifier, puis analyser le texte corrigé. Rien ne continue tant que vous ne l’avez pas décidé. |
| 5 | Rassemblez les personnes et les lieux relevés dans tout un projet et regroupez leurs graphies sous votre libellé, ou laissez un modèle vous le proposer. |
| 6 | Transcrivez des entretiens avec les locuteurs et l’horodatage, donnez aux locuteurs leur nom et cliquez sur une ligne pour l’écouter. |
| 7 | Posez vos propres questions à chaque texte : résumés, personnes, lieux et tonalité, chaque réponse à côté du document dont elle vient. |
| 8 | Choisissez vos fichiers, une tâche et un modèle, et voyez le coût estimé avant que quoi que ce soit ne soit envoyé au fournisseur. |
| 9 | Faites d’un numéro de journal ou d’un dossier d’archives une table des matières détaillée : chaque article avec ses pages, ses auteurs et son résumé. |
| 10 | Transcrivez avec Whisper et lisez les pages imprimées avec Tesseract, hors ligne : gratuit, sans compte ni clé, et rien ne quitte votre PC. |

## Notes for certification

Fill in the bracketed parts at submission. These notes are seen only by the certification
team; a test key may be given here, never in the listing.

```text
Product: Kalamu. Version: 1.5.0.0, native x64 and ARM64. Windows 11 with Edge WebView2 Runtime. Kalamu extracts text from documents, transcribes recordings and analyses texts with an AI provider the user configures with their own API key (Google Gemini, OpenAI, Anthropic Claude or Mistral), or works offline with Whisper (transcription), and Tesseract and two small OCR models, LightOnOCR-3 and GLM-OCR (text extraction). The app supplies no cloud account or credits; provider usage is billed to the user's own account, as the listing states.

Sample files for testing: https://github.com/fmadore/Kalamu/releases/download/v1.5.0/kalamu-samples.zip, with a README.txt: a 20-second spoken recording (interview.wav, made with Windows speech synthesis), a handwritten French report of 1755 (dodington-1755.jpg, Bibliothèque nationale de France, public domain), a 1911 university newspaper page with its machine-read text layer (daily-nebraskan-1911.pdf, Library of Congress, public domain), a whole four-page issue of the same paper, scans only, for a table of contents (daily-nebraskan-1911-issue.pdf), and an invented newspaper page with its text (bulletin-page.jpg, bulletin.txt).

Offline route, with no account or key:
1. Open Settings, choose the Offline tab, and under Transcription with Whisper download Whisper Base (about 60 MB; internet needed once). The download can be cancelled.
2. Choose New job, then Transcribe audio. Add interview.wav from the sample files. Under Provider, choose Whisper (on this computer), and under Model, Whisper Base. Choose Start.
3. The job runs on the CPU and shows its progress on the job's page. When it is done, choose Review: the transcript appears beside a player; click a line to hear it. Edit a word: the correction is kept apart from the original output. Press Ctrl F and type "Mopti" to find the word in the transcript.
4. Disconnect the network and repeat with another recording to confirm it runs offline. Results are written next to the sample file, as .txt and .docx with a provenance.json.
5. Offline text extraction: in Settings > Offline, under Text extraction with Tesseract, download French (4 MB, once). Choose New job, then Extract text, add bulletin-page.jpg, choose Tesseract (on this computer) and French, and Start. The review shows the page beside its text; the formats include ALTO XML and a searchable PDF.
6. Optional, slower: under Text extraction with OCR models, download LightOnOCR-3 1B (1.1 GB, once) and read bulletin-page.jpg with it the same way (Provider: OCR models (on this computer)). On the processor a page takes from tens of seconds to a few minutes; Settings then shows the time it took.

Cloud features: text extraction (dodington-1755.jpg), correction of existing text (daily-nebraskan-1911.pdf), analyses (bulletin.txt) and cloud transcription need the user's own key, entered in Settings and kept in Windows Credential Manager. [Either: A test Gemini API key with a small spending cap is provided for certification: <key>. Paste it in Settings under Google Gemini, then run New job > Extract text on bulletin-page.jpg (Gemini refuses dodington-1755.jpg without a reason; Claude and Mistral read it), New job > Correct existing text on daily-nebraskan-1911.pdf, and New job > Analyse texts on bulletin.txt with People, places and organisations. Or: No test key is provided; these features need a key from the provider, as the listing says.] Before the first job sent to a provider, a dialog names it, says what is sent and links the privacy policy; Send records consent for that provider. An estimated cost is shown before each job starts. The cheaper mode sends a job to the provider's batch service, which may take up to a day to answer; it is optional and off by default.

YouTube: in Transcribe YouTube, the user pastes a link to a public video; Google's Gemini API reads the video from YouTube itself. Kalamu does not download, store or convert YouTube videos. The app asks YouTube's public oEmbed endpoint for the video title, and the review plays the video in YouTube's privacy-enhanced embedded player, loaded only when the user plays it.

Generative AI (Store Policy 11.16): every result is text generated by the AI model the user chooses. The listing says so, results are always shown beside their source for checking, and Report a problem (the flag in the header of every review, and the button in Settings > About) opens GitHub’s issue forms in the default browser, among them one to report inappropriate output to the developer. The app itself sends nothing.

Voluntary support link (Store Policy 10.8.2): Buy me a coffee, at the foot of the sidebar and in Settings › About, under Support development, opens https://buymeacoffee.com/fmadore in the default browser. Buy Me a Coffee, a third-party service, takes any payment on its own site. The app has no in-app purchase, collects no payment information, and a donation unlocks, removes or changes nothing. The Product declaration "This product allows users to make purchases, but does not use the Microsoft Store commerce system" is ticked for this link.

The full-trust desktop process reads the files the user selects, converts audio with bundled FFmpeg executables (LGPL; source published with each release, licence terms in the listing), runs Whisper and Tesseract on the CPU, runs the OCR models in a bundled llama.cpp executable (MIT) started once per page, or in its second build that uses the graphics processor through Vulkan when the PC has one the app can use, stores API keys in Windows Credential Manager and writes results where the user chooses. It connects only to the provider the user selects, to Hugging Face for the Whisper and OCR model downloads the user requests, to GitHub for the public model catalogue and Tesseract's language data the user requests and to YouTube as described above. There is no developer server, telemetry, driver, service or auto-start task.
```

## Hero image

[store/hero-1920x1080.png](store/hero-1920x1080.png), 1920 × 1080, for the listing's 16:9
hero art: newspapers, a letter and a cassette recorder whose lines settle into a clean page of
text. It is an illustration generated with OpenAI GPT Image (`gpt-image-2.5-flare`); its prompt
and settings are in the `.json` beside it. It shows no real document, script or person, and
carries no text, so the Store's title and buttons can sit over its right side.

## Screenshots

One set of ten scenes for both stores, taken again for 1.4.1: 1.4.0's were refused (below),
and those of 1.2.0 before them do not show the wider views, larger type, projects in the
sidebar, Jobs link or Settings tabs. Take them from the installed Store package, in one theme
throughout (the light one, as the 1.0.0 captures); the Mac App Store's on a Mac, from the
TestFlight build ([below](#screenshots-1)). Store rules: `.png`, at least 1366 × 768,
under 50 MB; ten at most, five to eight recommended. The first one or two are what most people
see, in search results and at the top of the listing, so they carry what sets Kalamu apart:
checking AI text against its source. Keep what matters in the top two-thirds, where the Store
draws no overlay. No logo or marketing line in the image: each caption goes in the screenshot's
Description field (under 200 characters), which is also what a screen reader announces.

**How to take them.** Capture the window alone (Win+Shift+S, window mode) at 1920 × 1080 with
Windows display scaling at 100 %, or at 2560 × 1440 at 125 %, and keep one size for the whole
set. Wait for toasts to go. Name the jobs as a researcher would ("Le Réveil du Dahomey, 1937",
"Interviews, Ouagadougou 2024"), and keep the sidebar open on the same short list of jobs
throughout, so the set reads as one person's work. Rename the folders first: no `C:\Users\<name>`
path, no real key, no personal data in a file name.

**Every legible word is listing content.** The Store reads the text in a screenshot as the
listing's own words: 1.4.0 was refused under policy 11.5 (offensive content, "metadata") for two
screenshots of *The Monitor* (Omaha, 1915), whose masthead names the readers it served in the
wording of its time. Historical sources carry such wording, so before a document goes into a
screenshot, read what it shows (masthead, headlines, the text pane, analysis answers, names in
the dictionary) and leave out anything on race, religion, violence or politics that a reviewer
could take out of context, as well as caricatures in its advertisements. Prefer everyday
material: a student newspaper, local news, a scientific report. Before uploading, open each
image at full size and read it once more, word by word.

**The material.** All of it was screened this way, and comes in two zips:
`kalamu-samples.zip` (the release's sample files) and `kalamu-screenshot-material.zip` (nine
whole issues of *The Daily Nebraskan* of spring 1911 and `interview-two-speakers.wav`; made on
the PC in `screenshot-material/`, which git ignores, and described in its README). Each scene
uses:

| # | Files |
| --- | --- |
| 1 | `The Daily Nebraskan 1911-04-05.pdf`, the whole issue of the page in `daily-nebraskan-1911.pdf`, so the strip of pages has pages to mark reviewed |
| 2 | The demo project below |
| 3 | `dodington-1755.jpg`, with Claude or Mistral |
| 4, 5 | The *Newspapers* workflow run on the nine issues |
| 6 | `interview-two-speakers.wav`, with a cloud model that labels speakers, the two named in the review |
| 7 | An article of an issue, analysed once its text is corrected: the agricultural students (5 April) or a convocation lecture |
| 8 | The folder of the nine issues |
| 9 | `daily-nebraskan-1911-issue.pdf` (24 May 1911, four pages) |
| 10 | Settings › Offline models, or `bulletin-page.jpg` with Tesseract |

*The Monitor* and the *Victoria Daily Times* of 1915, used before, are out of the samples and
the screenshots: the *Victoria* for its war news, and no pre-war year either, since every issue
of 1912 screened as a replacement carried the period's words on Asian immigrants and Black
Americans.

**Other material whose rights allow it**, for scenes added later. Everything in `samples/`: the recording and the invented
page (made for Kalamu, CC0), the 1755 manuscript and the 1911 newspaper (public domain, sources
in its README); more public-domain newspapers from the Library of Congress's
[Chronicling America](https://chroniclingamerica.loc.gov/) (no known restrictions), or older
issues whose rights holder agrees; a handwritten letter in the public domain on Wikimedia
Commons; your own recording, or one made with the speaker's consent for this use. Gallica and
most archives restrict commercial reuse: a Store listing is safest without them.

**A demo project** for the shots that need one (2, 4 and 5), from public-domain material only:
a project *The Daily Nebraskan, Lincoln 1911* with collections *February–March*, *April–May*,
*Correspondence* and *Interviews*; the nine issues of `kalamu-screenshot-material.zip` (spring
1911, four pages each, from Chronicling America), `dodington-1755.jpg` and
`interview-two-speakers.wav`; tags
such as *agriculture*, *athletics* and *theatre* on a few documents. Save the example *Letters: read, check, then people
and places* as a workflow *Newspapers*, its first step on the newspaper preset, and run it on the
issues from the project's selection, so the dictionary
has the same people under several forms of their names (*William Jennings Bryan*, *W. J.
Bryan*, *Mr. Bryan*, who spoke at convocation on 28 March; *Prof. E. H. Barbour*, *Professor
Barbour*) for a model to propose merging, and a pair to keep apart (*Grace Bryan*, his daughter,
whose wedding the 24 May issue announces). *Chancellor Avery* and *Dean Burnett* recur across
issues as well.

**The runs to make**, in this order, since the project, the dictionary and the analysis build on
the workflow. Six runs; scenes 2, 8 and 10 need none. Name each job as a researcher would
(*Daily Nebraskan, spring 1911*, *Dodington report, 1755*, *Interview, Ouagadougou*), and check
each estimate before starting: only the first run is large (36 pages read, then analysed).

1. **Workflow *Newspapers*** (scenes 4 and 5). In Workflows, start from the example *Letters:
   read, check, then people and places*, name it *Newspapers*, and set its first step's preset
   to *Newspaper or magazine*; the second stays *People, places and organisations*. Run it on
   the nine issues from the project's selection. Take scene 4 when it stops after the first
   step for review, *Continue* in view; correct a few words, then continue. Once the second step
   is done, open the project's dictionary, ask a model to propose merges, and take scene 5 with
   a proposal open.
2. **Extract text, two set-ups** (scene 1): `The Daily Nebraskan 1911-04-05.pdf`, for instance
   Gemini and Claude, on *Newspaper or magazine*. Mark a page or two reviewed, then open the
   column on the agricultural students or the dramatic club with one set of readings open.
3. **Extract text, handwriting** (scene 3): `dodington-1755.jpg` with Claude or Mistral (Gemini
   refuses it), lines kept as on the page; correct one word.
4. **Transcribe audio** (scene 6): `interview-two-speakers.wav` in French with Gemini, which
   labels speakers; give both speakers names in the review.
5. **Analyse texts, two set-ups** (scene 7): from the corrected result of the 5 April issue,
   send its text into a new analysis whose prompt asks for a summary, people, places and a tone
   with its reason (in Prompts, a copy of *People, places and organisations* with a summary and
   a tone field added), on two models; point at a name the set-ups disagree on.
6. **Detailed table of contents** (scene 9): `daily-nebraskan-1911-issue.pdf` on *Newspaper or
   magazine issue*.

Without a run: the project (scene 2), made by hand as above; New job (scene 8), Extract text on
the folder of the nine issues with Gemini and Mistral OCR, captured before *Start* and left
unstarted; Settings › Offline models (scene 10), with Whisper Base and French for Tesseract downloaded,
or Tesseract on `bulletin-page.jpg` for the $0.00 estimate.

| # | What to show | Caption |
| --- | --- | --- |
| 1 | **Comparison in the review.** A newspaper column scanned on the left; on the right its text from set-up A, with two or three words highlighted where B reads differently, and one set of readings open over the text. The strip of pages below, a few marked reviewed. | Check every page beside its scan. Where two models read differently, the words are highlighted and you choose the right reading. |
| 2 | **A project.** *The Daily Nebraskan* project's documents: collections in the side list, tags on a few rows, the results column showing what each was read with and how much is reviewed, one row selected with the selection bar's actions. | Keep a project's documents together in collections and tags, each with every result read from it, whatever job made it. |
| 3 | **Handwriting.** `dodington-1755.jpg`, read with Claude or Mistral (Gemini refuses it), beside its transcription, lines kept as on the page, one word corrected (blue). | Read handwritten letters and registers line by line, kept as on the page so each line is easy to check against the original. |
| 4 | **A workflow run.** The *Newspapers* run with its first step done and stopped for review, the next step waiting, and *Continue* in view; or the workflow's drawing of its steps in Workflows. | Chain steps into a workflow: read the pages, check them, then analyse the corrected text. Nothing goes on until you say so. |
| 5 | **The dictionary.** The People field: rows with their spellings and the documents naming them, look-alikes pointed out, and a model's proposal open (rows, label, reason, certainty) with Accept and Turn down. | Gather the people and places your analyses found across a project, and merge their spellings under your own label, or let a model propose merges you decide on. |
| 6 | **Transcript.** The player above the transcript, speakers given real names, timestamps, the line being spoken highlighted. | Transcribe interviews with speakers and timestamps, give the speakers their names, and click any line to hear it. |
| 7 | **Analysis.** An article beside its answers: summary, people, places and tone with its reason, two set-ups' sigla on the names they disagree on, a name pointed at and found in the text. | Ask your own questions of each text: summaries, people, places and tone, every answer shown beside the document it comes from. |
| 8 | **New job, ready to start.** Extract text chosen, a folder of issues added, two set-ups (Gemini and Mistral OCR), the diagram of the task, and the estimate on the right with a cost per model. | Choose your files, a task and a model, and see the estimated cost before anything is sent to the provider. |
| 9 | **Table of contents.** A *Daily Nebraskan* issue's page on the left, the entries on the right with titles, pages, authors and summaries, the entries on the page shown marked; the strip of pages below. | Turn a whole newspaper issue or archival file into a detailed table of contents: each article with its pages, authors and summary. |
| 10 | **Offline.** Settings › Offline models with a Whisper model and French for Tesseract downloaded, or New job with Tesseract (on this computer) chosen and the estimate at $0.00. | Transcribe recordings with Whisper and read printed pages with Tesseract, offline: free, no account or key, and nothing leaves your PC. |

Search, the glossary, Prompts, the job's page and the Jobs page are left out of the ten: the
description and features name them. Name the captures after the rows above
(`01-comparison.png` … `10-offline.png`).

The Microsoft Store set for 1.4.1 is in
[store/screenshots/windows/](store/screenshots/windows/), `en/` for the English listing and
`fr/` for the French one; the Mac App Store's go beside it in `store/screenshots/macos/`. The
Windows set: 1920 × 1080 window captures of version 1.4.1 on
Windows, light theme. Scene 4 shows the workflow's drawing in Workflows, and scene 10 New job
with Tesseract on `bulletin-page.jpg`. In scene 1, pages 3 and 4 are marked reviewed without
corrections, so its footer's error rates score set-up A at 0 %: correct those pages, or retake
the image, before relying on the figures it shows. Read each image again at full size before
uploading it.

Leave out an image rather than show a feature that does not run as shown: certification checks
that the screenshots match the app.

# Mac App Store (App Store Connect)

Paste-ready text for the Mac App Store listing, written for version 1.5.0, the first to be
submitted if it is ready before a Mac is (1.4.1 was prepared but has not been sent; for it, leave
out the OCR models below). The steps,
the age rating and the App Privacy answers are in [mac-app-store.md](mac-app-store.md).
App Store Connect's limits, checked when this text was written: name and subtitle 30
characters, promotional text 170, description 4,000, keywords 100 (comma-separated, without
spaces after the commas, and no other company's trademark, Guideline 2.3.7), review notes
4,000. The text here fits: name 6, subtitle 25, promotional
text 167, description 3509, keywords 100,
review notes 3502.

Differences from the Microsoft Store text: macOS words (the Keychain, "your Mac"); no Buy Me a
Coffee line, since the store build hides that link (Guideline 3.1.1); the description in short
headed sections, since the App Store shows only its first lines before *more*; and the same
facts otherwise. **Languages**: English (U.S.) first; French can be added as a localisation
with its own screenshots.

## English (U.S.)

### Name

```text
Kalamu
```

If the name is taken on the store, `Kalamu for Researchers` (22 characters).

### Subtitle

```text
Read, transcribe, analyse
```

### Promotional text

Can be changed at any time without a new version.

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with the AI provider you choose, or offline, then check every result beside its source.
```

### Description

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with the AI provider of your choice, then check every result against its source. Kalamu works with Google Gemini, OpenAI, Anthropic Claude and Mistral under your own account and API key: their usage charges apply and are billed by them, and the app shows an estimated cost before anything is sent. Recordings can also be transcribed offline with Whisper, and printed pages read offline with Tesseract or, for difficult print, a small OCR model, free, after a one-time download.

EXTRACT AND CORRECT TEXT
Read PDFs and images, printed or handwritten: newspapers, reports, letters, registers, manuscripts. Keep the lines as on the page, or join them into running text. Kalamu checks the files first and suggests fixes for double pages, low resolution and quiet recordings. Correct the noisy text digital libraries supply, with every change shown against the original.

TRANSCRIBE
Recordings and videos with speakers and timestamps; give the speakers their real names. Offline, Whisper transcribes on your Mac with no account or key, and nothing leaves the computer; give it the names and words to spell. With Gemini, public YouTube videos are transcribed from their link.

TABLES OF CONTENTS AND ANALYSES
Turn a newspaper issue, an archival file or a book into a detailed table of contents, one entry per article with its pages, then read each article on its own. Analyse texts with fields you define, as you would brief a research assistant: a summary, the people, places and organisations named, the tone with a reason, themes from your own list.

CHECK EVERY RESULT
The page beside its text, the recording beside its transcript, line by line. Corrections are kept apart from the AI output, which is never overwritten, and you mark what you have reviewed. Compare two models on the same pages, their differences highlighted, with error rates against your corrections. Search a document, or a whole project, corrections included.

PROJECTS AND WORKFLOWS
Keep documents in collections and tags, each with every result read from it. A project dictionary gathers the people, places and terms your analyses found; merge spellings under your own label. Save a job's settings as a pipeline, or chain steps into a workflow that stops for your review.

SAFE FOR LARGE JOBS
Work is saved piece by piece: an interrupted job resumes where it stopped, and nothing is paid twice. The cheaper mode sends large jobs to the provider's batch service at about half the price.

OPEN FORMATS
Text, Markdown, Word, JSON, CSV, subtitles, searchable PDF and ALTO XML, each with a provenance file recording the model, prompt and settings, for your methods section.

YOUR DATA
Before the first job sent to each provider, Kalamu says what will be sent and asks you. Your files go straight from your Mac to the provider you choose; offline, nothing leaves it. API keys are kept in the macOS Keychain. The developer receives no files, results, keys or telemetry, and the app sells no subscription or credits. Results are generated by AI: check a sample against the source before relying on them.

For macOS 13 or later on Apple Silicon. Interface in English and French; documents in any language the chosen model reads. Free and open source (MIT).

Google Gemini and YouTube are trademarks of Google LLC; OpenAI of OpenAI OpCo, LLC; Claude of Anthropic, PBC; Mistral AI of Mistral AI. Kalamu is independent and not affiliated with or endorsed by any of them.
```

### Keywords

```text
OCR,HTR,handwriting,transcription,speech to text,interview,archive,historian,digital humanities,ALTO
```

### What's New

Not asked for the first version on the store. From the next one, adapt the Microsoft Store's
*What's new* above.

### URLs and copyright

| Field | Value |
| --- | --- |
| Support URL | `https://github.com/fmadore/Kalamu/issues` |
| Marketing URL | `https://github.com/fmadore/Kalamu` |
| Privacy Policy URL | `https://fmadore.github.io/Kalamu/privacy` |
| Copyright | `2026 Frédérick Madore` |
| Primary category | Productivity |
| Secondary category | Education |

## App Review Information

**Sign-in required**: no. **Contact**: the maintainer's name, telephone and email (App Store
Connect asks; they are not public). Fill in the bracketed part of the notes; a test key may be
given here, never in the listing. Notes:

```text
Kalamu extracts text from documents, transcribes recordings and analyses texts with an AI provider the user sets up with their own API key (Google Gemini, OpenAI, Anthropic Claude or Mistral), or offline with Whisper (transcription), and Tesseract and two small OCR models, LightOnOCR-3 and GLM-OCR (printed text). No sign-in: the app has no account of its own and sells nothing; provider usage is billed to the user's own account.

Sample files: https://github.com/fmadore/Kalamu/releases/download/v1.5.0/kalamu-samples.zip (interview.wav, a recording; bulletin-page.jpg, an invented newspaper page; dodington-1755.jpg, a 1755 manuscript; daily-nebraskan-1911.pdf, a 1911 newspaper page; bulletin.txt).

Offline route, no key needed:
1. Settings > Offline > Transcription with Whisper: download Whisper Base (about 60 MB, once).
2. New job > Transcribe audio, add interview.wav, Provider: Whisper (on this Mac), Model: Whisper Base. Kalamu asks for permission to save results in the file's folder: choose that folder in the panel. Start.
3. When done, Review: the transcript beside a player; click a line to hear it, edit a word (the correction is kept apart), press Cmd F and type "Mopti".
4. Settings > Offline > Text extraction with Tesseract: download French (4 MB). New job > Extract text, bulletin-page.jpg, Tesseract, French, Start. The formats include ALTO XML and a searchable PDF.
5. Optional: Settings > Offline > Text extraction with OCR models: download LightOnOCR-3 1B (1.1 GB). Read bulletin-page.jpg with it the same way (Provider: OCR models (on this computer)); it runs on the Mac's graphics processor.

Cloud features need the user's own key, entered in Settings and kept in the Keychain. [Either: A test Gemini key with a spending cap: <key>. Paste it in Settings under Google Gemini, then New job > Extract text on bulletin-page.jpg, or Analyse texts on bulletin.txt. Or: No test key is provided.] Before the first job sent to a provider, a dialog names the provider, says what is sent (the pages, recordings or text and any personal information in them), that it goes under the user's account and the provider's terms, and links the privacy policy; Send records consent for that provider, Cancel sends nothing (Guideline 5.1.2(i)).

Downloads (2.4.5(iv), 2.5.2): Whisper models and the OCR models (from Hugging Face) and Tesseract language data (from GitHub) are model weights and data files, not code; the features using them are in this build and are only offered for download in Settings, each verified by SHA-256. The model catalogue refreshed at start is a JSON list of models and prices from the app's GitHub repository, not code. There is no self-updater.

Sandbox: the app reads only files and folders the user picks or drops, and keeps security-scoped bookmarks so jobs resume after a restart. FFmpeg and ffprobe inside the bundle convert audio, and llama.cpp's llama-mtmd-cli, also inside the bundle, runs the OCR models once per page; all three are signed to inherit the sandbox and open no connection. Outgoing connections only: the chosen provider, Hugging Face and GitHub for downloads the user asks for, YouTube's oEmbed and privacy-enhanced player for a public video the user pastes.

Every result is text generated by the model the user chooses; it is always shown beside its source to be checked. Report a problem (the flag in each review and Settings > About) opens GitHub's issue forms in the browser. The developer receives no data, files, keys or telemetry.
```

## Screenshots

At least one, up to ten, 16:10, without transparency, `.png` or `.jpeg`, all one size: **2880 ×
1800** (a Retina Mac at its default scaling), or 2560 × 1600, 1440 × 900 or 1280 × 800. Take them
on a Mac, of the Mac App Store build, so the window's title bar, traffic lights and menu bar are
macOS's: screenshots of the Windows app would not show the app as it runs (Guideline 2.3.3).
Use the same ten scenes, captions and material as the Microsoft Store above, with "your Mac"
for "your PC" in the tenth, and screen them the same way: App Review reads screenshots as
metadata too, and asks that metadata suit all audiences (Guideline 2.3.8), Kalamu being rated
4+. The step-by-step for the day on a Mac is Part C of [mac-test-day.md](mac-test-day.md). Capture the window alone with ⌘⇧4 then Space, which adds a shadow with
transparency: hold ⌥ while clicking to leave it out, or export the image without alpha. Keep
the set in [store/screenshots/macos/](store/screenshots/), `en/` and `fr/`, beside the Windows one.
