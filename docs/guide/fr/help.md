---
layout: guide
lang: fr
title: En cas de problème
nav: Dépannage
order: 9
lede: "Les problèmes les plus courants, et que faire pour chacun."
description: "Résoudre les problèmes dans Kalamu, avec les réponses aux questions fréquentes."
---

[English](../help.md)

## Une clé ne fonctionne pas

Dans **Paramètres › Clés d’API**, cliquez sur **Tester**. Si le fournisseur refuse la clé,
vérifiez que vous l’avez copiée en entier, qu’elle n’a pas été supprimée sur le site du
fournisseur, et que la facturation est bien configurée sur le compte (la plupart des
fournisseurs refusent les appels à l’API sans elle, même pour les modèles bon marché).

## Un traitement s’est arrêté, ou des pages ont échoué

Ouvrez le traitement : chaque page en échec en donne la raison, en toutes lettres. Les causes
habituelles sont une connexion perdue, un fournisseur surchargé (Kalamu attend et réessaie tout
seul), ou une page que le fournisseur refuse. Relancez le traitement : seul ce qui a échoué est
renvoyé.

Un traitement mis en pause, ou interrompu parce que l’application a été fermée, se trouve en bas
de la barre latérale, sous *À reprendre*.

## Le texte est coupé, ou des lignes manquent

- Une page signalée comme tronquée a atteint la longueur maximale que le modèle peut écrire :
  envoyez moins de pages à la fois.
- Des lignes qui manquent au milieu d’une page sont une erreur du modèle. Faites lire la page
  par un second modèle et [comparez](review.md#comparer-des-modeles), ou prenez un modèle plus
  puissant pour ces pages.
- Pour une double page photographiée en une seule image, acceptez la correction que propose
  Nouveau traitement, afin que chaque page soit lue séparément.

## Un modèle hors ligne est très lent

C’est normal sur un ordinateur sans processeur graphique utilisable, surtout pour les modèles de
vision et de langue. Prenez une taille de Whisper plus petite, Tesseract pour l’imprimé net, ou
un modèle en ligne. Voir [Travailler hors ligne](offline.md#a-quelle-vitesse).

## Le coût est plus élevé que prévu

Regardez ce que le traitement a réellement consommé, et le prix du modèle dans la liste.
L’estimation compte ce que le modèle écrit et la réflexion qui lui est facturée avant d’écrire,
au niveau choisi sous *Réflexion*, et compte les pages lues avec la consigne *Journal ou magazine*
comme des pages pleines de petits caractères. Les longues pages manuscrites, les pages
particulièrement denses et les analyses aux longues réponses peuvent encore coûter plus que les
moyennes de l’estimation. Pour le lot suivant : un modèle plus petit, ou le
[mode moins cher](costs.md#trois-facons-de-payer-moins).

## Où sont mes résultats ?

Sur votre ordinateur, dans le dossier de données de Kalamu, jusqu’à ce que vous supprimiez le
traitement. Supprimer un traitement supprime ses résultats ; vos fichiers d’origine ne sont
jamais touchés. Pour garder ou partager des résultats, exportez-les.

## Questions

**Kalamu est-il vraiment gratuit ?** Oui : gratuit et libre, avec toutes ses fonctions. Vous
ne payez que le fournisseur d’IA que vous choisissez, si vous en choisissez un.

**Est-ce que Kalamu existe en français ?** L’interface est en anglais et en français. Les
documents peuvent être dans n’importe quelle langue que le modèle sait lire.

**Puis-je me fier au résultat ?** Autant que vous l’avez vérifié. Kalamu rend la vérification
rapide et en garde la trace ; il ne peut pas rendre un modèle fiable.

**Autre chose ?** Ouvrez un [ticket](https://github.com/fmadore/Kalamu/issues) sur GitHub, ou
cherchez un mot dans le **Glossaire** de l’application.
