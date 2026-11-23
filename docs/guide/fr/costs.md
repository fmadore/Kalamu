---
layout: guide
lang: fr
title: Coûts et confidentialité
order: 7
lede: "Ce que coûte un traitement, comment payer moins, et où vont vos fichiers, exactement."
description: "Coûts estimés, mode moins cher par lots, résultats réutilisés et confidentialité dans Kalamu."
---

[English](../costs.md)

## Ce que coûte un traitement

Kalamu est gratuit. Avec un modèle en ligne, vous payez le fournisseur, sur votre propre compte,
pour ce que vous envoyez et ce que le modèle écrit. Les prix varient beaucoup d’un modèle à
l’autre : le plus petit modèle d’un fournisseur peut coûter dix fois moins que son plus grand,
voire moins encore.

Avant le lancement d’un traitement, le **Coût estimé** indique ce qu’il devrait coûter, à partir
du nombre de pages et des prix publiés par le fournisseur. Une fois le traitement fini, vous
voyez ce qui a été réellement consommé. La facture du fournisseur fait foi : les prix changent,
et la liste de prix de Kalamu porte sa date.

Un conseil simple : essayez d’abord le plus petit modèle sur quelques pages représentatives.
Pour beaucoup de collections d’imprimés nets, il n’en faut pas plus.

## Trois façons de payer moins {#trois-facons-de-payer-moins}

- **Ne jamais payer deux fois.** Le résultat de chaque page est enregistré dès qu’il arrive. Un
  traitement qui échoue en route, ou que vous arrêtez, reprend sans renvoyer ce qui est déjà
  fait. Relancer un traitement avec les mêmes fichiers, le même modèle, le même prompt et les
  mêmes réglages ne coûte rien : Kalamu reconnaît le travail et le réutilise.
- **Le mode moins cher.** Pour une grosse collection qui n’est pas urgente, choisissez
  **Moins cher** dans Nouveau traitement : le traitement part au service de traitement par lots
  du fournisseur, à moitié prix, et la réponse arrive dans la journée, souvent plus tôt. Vous
  pouvez fermer l’application entre-temps : les résultats sont récupérés à sa prochaine
  ouverture. Deux sortes de travail partent quand même tout de suite, au prix habituel, parce
  que Kalamu vérifie chaque réponse dans le texte dès qu’elle arrive : une analyse qui cite le
  texte (comme le tri selon une question de recherche, ou l’analyse d’un entretien) et une
  transcription traduite ligne par ligne avec ses horodatages. L’estimation les compte au prix
  habituel, et sous **Rapidité et prix** Nouveau traitement dit pour quels modèles c’est le
  cas ; quand rien dans le traitement ne peut partir par lots, **Moins cher** n’est pas
  proposé.
- **Les modèles hors ligne** ne coûtent rien du tout. Voir [Travailler hors ligne](offline.md).

## Où vont vos fichiers

- **Chez le fournisseur que vous avez choisi, et nulle part ailleurs.** Les fichiers vont
  directement de votre ordinateur à Google, OpenAI, Anthropic ou Mistral, avec votre propre clé
  d’API. Kalamu n’a pas de serveur : son auteur ne voit jamais vos fichiers, vos clés ni vos
  résultats.
- **Vos clés** sont gardées dans le Gestionnaire d’identification de Windows ou dans le
  Trousseau de macOS, et ne sont envoyées qu’à leur fournisseur.
- **Vos résultats** sont gardés sur votre ordinateur, dans le dossier de données de Kalamu,
  jusqu’à ce que vous les supprimiez.
- **Hors ligne**, rien ne quitte l’ordinateur.

Ce que chaque fournisseur fait de ce qu’il reçoit (combien de temps il le garde, s’il peut s’en
servir pour améliorer ses modèles) dépend de ses conditions d’utilisation de l’API, en général
plus strictes que celles de son application de discussion grand public, et de l’accord
éventuel passé avec votre établissement. Si vos sources sont sensibles (entretiens avec des
personnes vivantes, archives à accès restreint), lisez ces conditions, renseignez-vous auprès de
votre établissement, et pensez à travailler hors ligne.

La [politique de confidentialité](../../privacy.md) (en anglais) dit en détail ce que Kalamu
envoie, à qui, et ce qu’il garde.

## Avant d’envoyer des sources qui ne vous appartiennent pas

Vérifiez que vous en avez le droit : le règlement de la salle de lecture d’un service
d’archives, un formulaire de consentement, la licence d’un éditeur. Kalamu ne peut pas le
savoir : il envoie ce que vous lui donnez au fournisseur que vous choisissez.
