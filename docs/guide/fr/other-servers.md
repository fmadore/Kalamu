---
layout: guide
lang: fr
title: Autres serveurs
order: 7
lede: "Votre université fait peut-être tourner des modèles d’IA pour ses membres, ou un ordinateur plus puissant de votre bureau fait tourner Ollama. Kalamu peut se servir de leurs modèles, après avoir essayé chacun."
description: "Ajouter Ollama ou le serveur de type OpenAI d’une université à Kalamu, ce pour quoi ses modèles sont proposés, et où vont vos fichiers."
---

[English](../other-servers.md)

## Ajouter Ollama sur cet ordinateur

Si [Ollama](https://ollama.com) est installé et dispose d’un modèle (voir
[Avant de commencer](#avant-de-commencer)) :

1. Ouvrez **Paramètres**, puis **Autres serveurs**, et choisissez **Ajouter un serveur**.
   Donnez-lui un **Nom** de votre choix (il s’affiche partout où figure le nom d’un
   fournisseur) et l’**Adresse** `http://localhost:11434`. Laissez **Ce serveur demande une
   clé** décoché. Cliquez sur **Se connecter**.
2. Kalamu indique ce qu’il a trouvé, par exemple *Ollama 0.40.18, sur cet ordinateur.*, et
   liste les **Modèles de ce serveur**. Cochez ceux que vous voulez utiliser.
3. Cliquez sur **Essayer et ajouter**. Kalamu essaie chaque modèle, montre ce qu’il sait faire
   et enregistre le serveur. Ses modèles sont désormais dans **Nouveau traitement**.

La suite de ce chapitre détaille chaque étape, ce que décide l’essai, et que faire quand
quelque chose ne marche pas.

## Quand cela vaut la peine

- **Votre établissement propose un service** qui lui est propre, en général à une adresse « de
  type OpenAI », et vous avez le droit, ou la consigne, d’y envoyer vos documents plutôt qu’à
  une entreprise.
- **Un autre ordinateur de votre bureau fait tourner Ollama**, avec plus de mémoire ou une
  meilleure carte graphique que le vôtre.
- **Vous voulez un modèle plus grand que ceux que Kalamu fait tourner lui-même** dans
  [Modèles hors ligne](offline.md), et votre ordinateur a la mémoire nécessaire.

Inutile de vous en occuper sinon : les [modèles hors ligne](offline.md) de Kalamu ne demandent
aucun réglage, et les quatre fournisseurs en ligne lisent les scans difficiles et l’écriture
manuscrite mieux que la plupart des modèles qu’une seule machine peut faire tourner. Essayez
quelques-unes de vos pages avant d’y faire passer toute une collection.

Rien de ces autres serveurs n’apparaît dans Nouveau traitement tant que vous n’en avez pas
ajouté un.

## Avant de commencer

**Ollama** est gratuit et fonctionne sous Windows comme sur macOS.

1. Téléchargez-le sur [ollama.com](https://ollama.com) et installez-le. Il tourne ensuite en
   arrière-plan, avec son icône dans la zone de notification sous Windows et dans la barre des
   menus sur un Mac.
2. Ouvrez un terminal (**Terminal** sous Windows, depuis le menu Démarrer ; **Terminal** sur un
   Mac, dans Applications › Utilitaires) et téléchargez un modèle qui lit les images :

   ```
   ollama pull gemma3:4b
   ```

   `ollama pull` télécharge une fois pour toutes le modèle nommé après lui. Un modèle plus
   grand, comme `gemma3:12b`, lit mieux et demande plus de mémoire. Les noms changent :
   cherchez les modèles marqués *vision* dans la
   [bibliothèque de modèles d’Ollama](https://ollama.com/search). Un modèle qui ne lit que du
   texte peut quand même faire des analyses, des corrections et des traductions, mais pas lire
   des pages.
3. `ollama list` affiche les modèles que vous avez téléchargés.

**Ollama sur un autre ordinateur de votre réseau** ne répond qu’à son propre ordinateur tant
que la personne à qui il appartient ne lui dit pas le contraire. Sur l’ordinateur où tourne
Ollama, réglez `OLLAMA_HOST` sur `0.0.0.0:11434`, comme
[la documentation d’Ollama](https://docs.ollama.com/faq#how-can-i-expose-ollama-on-my-network)
(en anglais) l’explique pour chaque système, puis redémarrez Ollama. Dans Kalamu, l’adresse
est alors `http://<nom ou adresse de cet ordinateur>:11434`.

**Le service d’une université ou d’un centre de calcul** vous donne son adresse, et le plus
souvent une clé. Demandez à la personne qui le gère quels modèles lisent les images.

## Ajouter un serveur, étape par étape

### 1. Le nom et l’adresse

Dans **Paramètres › Autres serveurs**, choisissez **Ajouter un serveur**. Le **Nom** est à
votre choix. L’**Adresse** est celle où le serveur répond : Ollama sur cet ordinateur répond à
`http://localhost:11434`. Ne cochez **Ce serveur demande une clé** que s’il en demande une, et
collez la **Clé**. Cliquez sur **Se connecter**.

### 2. Ce que Kalamu a trouvé

Kalamu demande au serveur ce qu’il est et quels modèles il a ; vous n’avez pas à choisir de
type de serveur. Il le dit en une ligne, par exemple *Ollama 0.40.18, sur le réseau, à
l’adresse 192.168.1.20.* ou *Un serveur de type OpenAI, sur cet ordinateur.* Pour une adresse
du réseau qui commence par `http://`, il prévient que vos fichiers circulent sans chiffrement
(voir [Ce qui est envoyé, et où](#ce-qui-est-envoye-et-ou)).

Sous **Modèles de ce serveur**, chaque modèle indique sa taille et ce que le serveur en dit
(*lit les images, selon le serveur*, ou *ne lit pas les images, selon le serveur*). Cochez les
modèles à utiliser. Un modèle qui *S’exécute chez ollama.com, pas sur ce serveur : il ne peut
donc pas être ajouté.* ne peut pas être coché : Ollama transmettrait vos pages aux serveurs
d’une entreprise, sans que rien ne le montre derrière une adresse de votre propre ordinateur.

### 3. Plus de choix, si besoin

Vous n’avez rien à toucher ici pour commencer : Kalamu remplit chaque valeur. Chaque modèle a
ses propres **Plus de choix** :

- **Lit jusqu’à** : combien de jetons (des morceaux de mots) le modèle lit à la fois, réponse
  comprise. Kalamu part de ce pour quoi le modèle a été entraîné, jusqu’à 32 768. Davantage lui
  permet de lire des pages et des textes plus longs, mais prend plus de mémoire sur le serveur.
- **Réponse la plus longue** : le nombre maximal de jetons qu’il peut écrire en une réponse.
- **Prix par million de jetons lus** et **Prix par million de jetons écrits** : en dollars
  américains, si votre établissement facture. Laissés vides, le coût d’un traitement se lit
  « sans prix ».

Sous les modèles, les **Plus de choix** du serveur lui-même contiennent **Requêtes
simultanées**, de 1 à 8, une par défaut. Davantage n’accélère que si le serveur répond à
plusieurs à la fois.

Cliquez sur **Essayer et ajouter**.

## L’essai, et ce qu’il décide

Avant qu’un modèle soit proposé où que ce soit, Kalamu l’essaie avec trois petites requêtes à
lui : une question d’une ligne ; une petite image, fournie avec Kalamu, qui contient un nombre ;
et un court formulaire de deux champs à remplir à propos d’une phrase. **Aucun de vos fichiers
n’est envoyé.**

Chaque modèle affiche ensuite deux lignes, chacune **Oui** ou **Non** :

| Ligne | Oui veut dire |
| --- | --- |
| **Lit les pages** | il a lu le nombre de l’image : proposé pour l’extraction de texte. |
| **Remplit le formulaire d’une analyse** | il a rempli le formulaire correctement : proposé pour les analyses, les corrections et les traductions. |

Avec les deux, il est aussi proposé pour les tables des matières. Un modèle qui n’a fait ni
l’un ni l’autre n’est pas ajouté.

Vous ne pouvez pas passer outre l’essai, et c’est ce qui vous protège : un modèle qui ne voit
pas les images répondrait sinon à chaque page d’un scan de quarante pages par une lecture
inventée ou vide, avec la même assurance.

- Le premier essai peut prendre **quelques minutes**, le temps que le serveur charge le
  modèle ; **Arrêter** l’interrompt, et le modèle n’est pas ajouté.
- Un modèle a été mis à jour sur le serveur ? **Essayer de nouveau**, à côté de lui dans
  **Vos serveurs**, refait son essai.
- **« Non » pour la lecture des pages** signifie le plus souvent un modèle qui ne lit que du
  texte : choisissez-en un qui lit les images. **« Non » pour le formulaire** signifie le plus
  souvent un modèle trop petit pour s’en tenir à un formulaire : choisissez-en un plus grand.

## S’en servir

Dans **Nouveau traitement**, les modèles du serveur sont réunis sous leur propre titre,
**Autres serveurs**, chacun accompagné du nom que vous avez donné au serveur, et seulement pour
les tâches que son essai a montrées. Ils lisent une page à la fois. Le mode moins cher ne
s’applique pas à eux. Ils ne sont proposés ni pour transcrire des enregistrements (Whisper,
dans [Modèles hors ligne](offline.md), le fait sur votre ordinateur) ni pour la
[recherche par le sens](projects.md#la-recherche-par-le-sens-et-la-carte).

## Ce qui est envoyé, et où {#ce-qui-est-envoye-et-ou}

- **Sur cet ordinateur** (`localhost`), rien n’en sort, et Kalamu ne demande rien.
- **Sur le réseau**, vos fichiers vont à une machine que quelqu’un fait tourner, selon les
  règles de cette personne ou de cet établissement, et non à une entreprise sous votre compte.
  Avant le premier traitement, Kalamu pose la question une fois, en nommant le serveur et son
  adresse (*Envoyer vos fichiers à un autre serveur ?*), et la pose de nouveau si l’adresse
  change, avant qu’un traitement créé plus tôt ne reprenne là-bas.
- **Une adresse qui commence par `http://`** envoie vos fichiers, et la clé s’il y en a une,
  sans chiffrement sur le réseau. C’est courant à l’intérieur du réseau d’un établissement, et
  c’est une raison de ne pas utiliser une telle adresse à travers Internet : demandez une
  adresse en `https://` à la personne qui gère le serveur.
- **Kalamu ne connaît que l’adresse.** Ce que le programme qui s’y trouve garde, et qui peut le
  lire, dépend de la personne qui le fait tourner : demandez-le-lui.
- **La clé** est gardée dans le Gestionnaire d’identification de Windows ou dans le Trousseau
  de macOS, et n’est envoyée qu’à ce serveur.

Le fichier `provenance.json` d’un export nomme le serveur et son adresse. La
[politique de confidentialité](../../privacy.md) (en anglais) dit la même chose en détail.

## Ce que cela coûte

Rien, sauf si votre établissement facture. Sans prix dans **Plus de choix**, le coût d’un
traitement se lit « sans prix ».

## Jamais une lecture partielle

Une page ou un texte plus long que ce que le modèle lit sur ce serveur est refusé, jamais
coupé en silence. La page échoue avec un message, en anglais pour l’instant, qui commence par
*The page or text is longer than* (« la page ou le texte est plus long que ce que » le modèle
lit sur le serveur). Deux remèdes : augmenter **Lit jusqu’à** pour ce modèle, si la machine a
la mémoire nécessaire ; ou choisir un modèle qui lit davantage.

Avec Ollama, Kalamu demande au serveur de refuser plutôt que de couper. On ne peut pas le
demander à un serveur de type OpenAI : Kalamu compare le nombre de jetons que le serveur dit
avoir lus avec le texte envoyé, et refuse une réponse qui n’en a lu qu’une partie. Un serveur
qui ne dit pas combien il a lu est cru sur parole.

## Modifier ou retirer un serveur

Dans **Vos serveurs**, **Modifier** rouvre les trois mêmes étapes ; un modèle déjà essayé à la
même adresse garde son essai. **Retirer** demande d’abord, et dit ce qui part avec lui : ses
traitements et leurs résultats restent, mais un traitement en pause ou interrompu sur ce serveur
ne pourra pas être terminé, et les configurations et chaînes de traitement enregistrées qui
l’utilisent échoueront. Un serveur ajouté de nouveau est un nouveau serveur, dont on choisit de
nouveau les modèles. L’adresse d’un serveur ne peut pas être changée, ni le serveur retiré,
pendant qu’un traitement tourne dessus : mettez le traitement en pause, ou laissez-le finir.
Après un changement d’adresse, **Reprendre** sur un traitement créé avant redemande si vos
fichiers peuvent y être envoyés.

## Quand cela ne marche pas

| Kalamu dit | Ce qu’il faut vérifier |
| --- | --- |
| *Rien ne répond à l’adresse* … | L’adresse, et qu’Ollama (ou le programme du serveur) tourne sur cet ordinateur. Pour un autre ordinateur, que la personne à qui il appartient a réglé `OLLAMA_HOST` (voir [Avant de commencer](#avant-de-commencer)). |
| *Quelque chose répond à l’adresse* …*, mais ce n’est pas un serveur que Kalamu connaît* | L’adresse, auprès de la personne qui gère le serveur. |
| *Le serveur à l’adresse* … *demande une clé.* | Cochez **Ce serveur demande une clé** et collez-la. |
| *Le serveur à l’adresse* … *a refusé la clé.* | La clé, auprès de la personne qui gère le serveur. |
| *The page or text is longer than* … (en anglais) | Augmentez **Lit jusqu’à**, ou choisissez un modèle qui lit davantage (ci-dessus). |
| … *is no longer on* … (en anglais : le modèle n’est plus sur le serveur) | Le modèle a été retiré ou renommé sur le serveur : reconnectez-vous avec **Modifier** et cochez son nouveau nom. |

Pour tout le reste : [En cas de problème](help.md).

## Encore nouveau

Les autres serveurs sont nouveaux dans Kalamu. Ils ont été essayés avec des réponses
enregistrées d’Ollama, pas encore avec toutes les sortes de serveurs. Si quelque chose ne
marche pas comme ce chapitre le dit, [signalez-le](help.md#questions).
