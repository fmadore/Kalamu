---
layout: guide
lang: fr
title: Exporter et citer
order: 8
lede: "Emportez vos résultats vérifiés là où vous en avez besoin, avec une trace de leur fabrication que vous pouvez citer."
description: "Les formats d’export et la provenance dans Kalamu."
---

[English](../export.md)

## Formats

Depuis la relecture d’un traitement, l’export propose les formats qui conviennent à son type de
résultat.

| Format | Utile pour |
| --- | --- |
| Texte brut, Markdown | Lire, citer, passer dans n’importe quel autre outil. |
| Word | Partager avec des collègues, retravailler le texte. |
| JSON, CSV | Analyser dans un tableur, avec R ou Python ; les analyses s’exportent avec une ligne par document, ou une ligne par entrée pour un registre. |
| Sous-titres | Les transcriptions avec leur horodatage, à superposer à une vidéo. |
| PDF interrogeable | Le scan d’origine avec le texte derrière, pour pouvoir y chercher et en copier des passages. |
| ALTO XML, PAGE XML | Chaque mot avec sa place sur la page, pour les bibliothèques numériques et les éditions. Vos corrections sont incluses. |

Les exports utilisent toujours le texte **tel que vous l’avez corrigé**. La page sur les
[formats de sortie](../../formats.md) (en anglais) décrit chaque fichier, champ par champ.

## Provenance

Chaque export s’accompagne d’un fichier `provenance.json`. Il indique, pour chaque résultat :

- le fournisseur et le modèle exact ;
- le prompt complet et les réglages ;
- la date du traitement ;
- si le texte a été corrigé et relu par une personne ;
- la façon dont la source est décrite dans son projet (lieu de conservation, cote, référence).

Gardez-le avec les fichiers exportés. C’est lui qui vous permet, à vous ou à un relecteur, de
dire exactement comment un texte a été produit, et qui rend un résultat reproductible, autant
que ces modèles le permettent.

## Citer votre méthode

Pour la partie méthodologique d’un article, une **configuration** peut être exportée avec son
modèle, ses réglages et ses prompts complets. Une phrase de ce genre suffit alors, avec les
fichiers en annexe ou dans un dépôt :

> Les pages ont été transcrites avec *modèle* (*fournisseur*, *mois et année*) dans Kalamu
> *version*, avec le prompt donné en annexe. *N* pages sur *M* ont été vérifiées face aux scans
> et corrigées par l’auteur ; le texte non corrigé et le texte corrigé sont tous deux conservés.

Pour une grosse collection relue par échantillon, exportez aussi la liste de relecture du
projet : elle indique quelles pages ont été tirées et combien ont été vérifiées. Voir
[Ce qu’il faut vérifier](projects.md#ce-quil-faut-verifier).

## Citer Kalamu

Kalamu est un logiciel libre, sous licence MIT. Citez la version que vous avez utilisée et le
dépôt : [github.com/fmadore/Kalamu](https://github.com/fmadore/Kalamu).
