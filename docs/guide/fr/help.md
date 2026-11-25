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

Ouvrez le traitement : chaque page en échec explique la raison en langage clair. Les causes
habituelles sont une connexion perdue, un fournisseur surchargé (Kalamu attend et réessaie tout
seul), ou une page que le fournisseur refuse. Relancez le traitement : seul ce qui a échoué est
renvoyé.

Une analyse qui renvoie une réponse vide pour un long texte n’est pas conservée : Kalamu
réessaie avec davantage de réflexion à chaque fois quand le modèle le permet (*Moyenne*, puis
*Élevée*), et garde la première réponse complète en indiquant le niveau utilisé. Si toutes les
tentatives restent vides, la page liste les niveaux essayés : choisissez un autre modèle plutôt
que de relancer.

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
au niveau choisi sous *Réflexion* (certains modèles, comme Gemini 4, réfléchissent bien plus
que d’autres), et compte comme des pages pleines de petits caractères celles lues avec la
consigne *Journal ou magazine* et celles d’un PDF dont la couche de texte est en petits
caractères. Une zone relue est comptée pour la partie de la page envoyée. Les longues pages
manuscrites, un journal numérisé en images et lu avec une autre consigne, et les analyses aux
longues réponses peuvent encore coûter plus que les moyennes de l’estimation. Pour le lot suivant : un modèle plus petit, ou le
[mode moins cher](costs.md#trois-facons-de-payer-moins).

## Où sont mes résultats ?

Sur votre ordinateur, dans le dossier de données de Kalamu, jusqu’à ce que vous supprimiez le
traitement. Supprimer un traitement supprime ses résultats ; vos fichiers d’origine ne sont
jamais touchés. Pour garder ou partager des résultats, exportez-les.

## Les copies de sécurité de votre travail

Kalamu garde tout (vos traitements, vos corrections, vos projets et les réponses que vous avez
payées) dans un seul fichier, sa base de données. Il copie ce fichier de lui-même : une fois
chaque jour où vous ouvrez Kalamu, les trois dernières copies étant gardées, et une fois avant
qu’une mise à jour ne le modifie. Les copies restent sur votre ordinateur. **Paramètres ›
Général › Copies de sécurité** les liste avec leur date et leur taille, montre leur dossier
avec **Afficher dans l’Explorateur de fichiers** (**Afficher dans le Finder** sur un Mac), et
**Ouvrir cette copie…** en ouvre une à la place de la base, après vous l’avoir demandé : Kalamu
redémarre, et la base telle qu’elle était est gardée dans le même dossier, *Mise de côté* ;
rien n’est donc effacé. Le travail fait après la date de la copie n’apparaît pas. Ouvrir une
copie attend qu’aucun traitement ne soit en cours.

Une copie n’est pas faite s’il devait rester moins de 1 Go libre sur le disque ; les Paramètres
le disent alors. Libérez de la place, et la copie du lendemain sera faite.

## Kalamu dit que sa base de données ne peut pas être lue

Si le fichier a été abîmé (une coupure de courant, un disque qui lâche), Kalamu le dit au
démarrage, avant d’ouvrir sa fenêtre, et propose **Ouvrir la copie**, avec la date de la copie,
ou, s’il n’y a pas encore de copie, **Démarrer avec une liste vide**. Dans les deux cas, le
fichier endommagé est gardé, jamais effacé, dans le dossier des copies que le message indique ;
**Quitter** ne change rien. Le message indique aussi le dossier des journaux de Kalamu, à
joindre à un [signalement](https://github.com/fmadore/Kalamu/issues).

Certains dégâts n’apparaissent que lorsque Kalamu vérifie tout le fichier, peu après son
ouverture : il vous demande alors s’il faut ouvrir à sa place la copie la plus récente, et
Paramètres › Général › Copies de sécurité le rappelle tant que ce n’est pas fait.

Si un traitement créé par une version plus récente de Kalamu ne peut pas être lu, il n’apparaît
pas dans la liste et il est signalé une fois au démarrage ; il est gardé tel quel.

## Un écran ne fonctionne plus

Kalamu affiche alors *Cet écran ne fonctionne plus* à sa place. Tout ce que vous avez
enregistré est conservé, et les traitements en cours continuent. Cliquez sur **Revenir en
arrière**, ou choisissez un autre écran dans la barre latérale ; si un écran ne marche
toujours pas, cliquez sur **Recharger la fenêtre**. Kalamu note ce qui s’est passé dans son
journal, sans rien du texte de vos documents : le dossier `logs` du dossier de données
qu’indiquent les Paramètres sous Windows, et `~/Library/Logs/Kalamu` sur un Mac. Joignez le
fichier de ce jour-là quand vous signalez le problème.

## Questions

**Kalamu est-il vraiment gratuit ?** Oui : gratuit et libre, avec toutes ses fonctions. Vous
ne payez que le fournisseur d’IA que vous choisissez, si vous en choisissez un.

**Est-ce que Kalamu existe en français ?** L’interface est en anglais et en français. Les
documents peuvent être dans n’importe quelle langue que le modèle sait lire.

**Puis-je me fier au résultat ?** Autant que vous l’avez vérifié. Kalamu rend la vérification
rapide et en garde la trace ; il ne peut pas rendre un modèle fiable.

**Autre chose ?** Ouvrez un [ticket](https://github.com/fmadore/Kalamu/issues) sur GitHub, ou
cherchez un mot dans le **Glossaire** de l’application.
