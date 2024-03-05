# MirrorVerse

Élèves : Guillaume Calderon, Mohammed Ali, Eymeric Déchelette

Enseignant : Jérôme Bastien

## Cahier des charges

### Contexte

Ce projet fait suite à une demande d'élève, Quentin COURDERO, en troisième année en informatique à Polytech. Il a demandé à son enseignant Jérôme Bastien de l'aider à écrire un algorithme pour déterminer la trajectoire d’un rayon lumineux lorsque celui-ci vient frapper un miroir plan fini. 

### Objectif

L'objectif de ce projet est donc d'étudier le comportement d'un rayon lumineux lorsqu'il rencontre des miroirs. Ce dernier a alors deux comportements possibles : il peut être piégé dans le nid de mirroirs et se réfléchir à l'infini, ou il peut parvenir à sortir du nid de mirroirs. Sa trajectoire, quant à elle, peut suivre un motif ou être chaotique. On considérera qu'une trajectoire est chaotique si, après n réflexions (n dépendant du cas étudié), on ne constate aucune répétition.

Dans le cadre de ce projet, nous coderons un outil simulant le comportement de rayons lumineux lorsqu'ils rencontrent des miroirs. La simulation devra être juste physiquement, c'est à dire qu'elle devra coller au maximum à la réalité. Elle s'appuiera sur la seconde loi de Snell-Descartes (réflexion) et devra obligatoirement fonctionner en 2 dimensions avec des miroirs plans. 

La simulation pourra par la suite être enrichie, en prenant en compte par exemple plus de dimensions ou en intégrant un plus grande variété de miroirs.

### Réponse Technique

Pour répondre au mieux aux exigences de ce projet, le simulateur sera développé avec le langage Rust. En effet, ce langage permet d'avoir un maximum d'optimisation et de s'assurer d'un minimum de bugs imprévus. 
<span style="color:red">
--> ça ça veut rien dire. Optimization de quoi ? De l'algo ? Du temps pour tourner ? Il faut préciser. Aussi, pourquoi ce ce langage permet d'optimiser plus qu'un autre ? Minimum de bugs imprévus ? Pourquoi ? Pourquoi ce langage permet ça et pas les autres ? Qu'est-ce qu'un bug imprévu ? </span>

On utilisera la bibliothèque nAlgebra qui nous permettra de manipuler aisément différentes notions mathématiques telles que les vecteurs, les points, etc.

La simulation intégrera de plus un outil de visualisation permettant de se déplacer dans le monde virtuel comprenant les miroirs et rayons simulés. Cela permettra de constater simplement et rapidement le résultat de la simulation. Cet outil de visualisation sera développé à l'aide de la bibliothèque wgpu.

### Difficultés Attendues
<span style="color:red"> (si le titre n'est pas imposé) </span>

Pour la réalisation de ce projet, nous avons identifié deux difficultés majeures. La première concerne la détection de l'intersection entre les rayons et les miroirs. Celle-ci se doit d'être exacte, car toute imprécision, même minime, se traduira par de gros écarts entre la similation et la réalité après un grand nombre de réflexions. 
<span style="color:red"> T'avais mis efficace mais ça veut dire quoi efficace dans ce contexte ? faite rapidement ? être optimisée ? il faut expliquer. par ex: "L'algorithme calculant cette detection doit de plus être optimisé pour tourner le plus rapidemment possible, car nous allons être amenés à la faire tourner très souvent (ordre de grandeur) </span>

La deuxième difficulté concerne la technologie d'affichage. Elle demandera beaucoup de recherches documentaires car les réalisateurs du projet auront besoin de se former sur cette technologie relativement nouvelle pour eux. <span style="color:red"> du coup risque de retard ? Si oui: Ces recherches documentaires ont été intégrées dans l'organisation temporelle du projet mais présentent un risque de retard. </span>


### Milestones

Pour ce projet, nous prevoyons 4 milestones, qui seront différentes versions de l'algorithme. Chaque version apportera des fonctionnalités supplémentaires. 


#### Fonctionnalités v1
Pour cette première version, on devra pouvoir:
- Editer facilement l'ensemble <span style="color:red"> (?? la configuration plutôt non?) </span> des miroirs pour la simulation. Ceci se fera probablement via une description en JSON.
- Choisir la direction et le point de départ du rayon.
- Visualiser aisément le trajet du rayon lumineux.

Cette simulation devra de plus supporter les miroirs plan et fonctionner en 2D.
Cette première version utilisera cependant déjà des bases locales et des symétries plutôt que des angles afin d'anticiper la généralisation en 3D.


#### Fonctionnalités v2
Cette deuxième version devra supporter les types de miroirs suivants :
+ plan,
+ circulaire,
+ en courbe de Bézier.

#### Fonctionnalités v3
L'objectif minimal de la troisième version est d'obtenir une simulation fonctionnant en 3D. Son fonctionnement en nD serait un plus. 

#### Fonctionnalités v4
Enfin, la quatrième version devra intégrer, selon les besoins, des fonctionnalités d'analyse de la trajectoire du rayon. On pense notamment à la détection automatique de la sortie du rayon de l'ensemble de miroirs, ou à la détection automatique d'une boucle (le rayon passe 2 fois au même endroit).

## Organisation temporelle

<span style="color:red"> j'ai pas l'extension pour lire facilement le gantt je te laisse modifier si nécessaire en fonction de ce que j'ai modifié + haut </span>

```mermaid
gantt
    title le diagramme de Gantt
    dateFormat  DD/MM/YYYY
    tickInterval 14day

    section Version 1
    Réflexion, création du cahier des charges    :done, a1, 01/02/2024, 14d
    Début de la programmation   :milestone, done, m1, after a1,
    Création affichage  :active, after a1, 7d
    Création architecture miroir   :active, after a1, 7d
    Création réflexion basique rayon   :active, a2, after a1, 7d
    Liaison des différentes parties  :a3, after a2, 7d
    Tests d'intégration :a4, after a3, 4d
    fin de la version 1 :milestone, m2, after a4,

    section Version 2
    Amélioration affichage  :after m2, 7d
    Création de nouveau miroir   :after m2, 7d
    Nouvelles réflexions    :a5, after m2, 7d
    Liaison des différentes parties  :a6, after a5, 7d
    Tests d'intégration :a7, after a6, 4d
    fin de la version 2 :milestone, m3, after a7,

    section Version 3
    Généralisation affichage  :after m3, 7d
    Généralisation des miroirs   :after m3, 7d
    Généralisation des réflexions    :a8, after m3, 7d
    Liaison des différentes parties  :a9, after a8, 7d
    Tests d'intégration :a10, after a9, 4d
    fin de la version 3 :milestone, m4, after a10,

    section Version 4
    Adaptation de la structure du programme  :a11, after m4, 7d
    Développement détecteur de récurrence   :after a11, 7d
    Développement du détecteur de sorties    :a12, after a11, 7d
    Liaison des différentes parties  :a13, after a12, 7d
    Tests d'intégration :a14, after a13, 4d
    fin de la version 4 :milestone, m5, after a14,

    section Analyse et rapport
    Essaie de simulation    :a15, after m5, 3d
    Analyse des résultats   :a16, after a15, 4d
    Écriture d'un rapport d'analyse :a17, after a16, 4d
    fin de projet   :milestone, m6, after a17,
```


