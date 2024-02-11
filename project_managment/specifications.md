# Mirror verse

## cahier des charges

### objectif

L'objectif de ce projet est d'étudier le comportement d'un rayon lumineux à la rencontre de miroirs.

On étudiera, par exemple, si, en envoyant le rayon lumineux sur un grand nombre de miroirs, la trajectoire est chaotique ou si le rayon finit par sortir de l'ensemble des miroirs.

On considérera qu'une trajectoire est chaotique si, après n réflexion, on ne constate aucune récurrence.

### réponse technique

Pour ce faire, on écrira un outil de simulation des rayons lumineux avec des miroirs.

La simulation devra physiquement être juste. (coller au maximum à la réalité dans tous les cas.)

La simulation s'appuiera sur la seconde loi de Snell-Descarte sur la réflexion.

Le simulateur sera développé avec le langage Rust afin d'avoir un maximum d'optimisation et de s'assurer d'un minimum de bugs imprévus.

La simulation disposera d'un outil de visualisation permettant de se déplacer dans le monde virtuel pour constater simplement le résultat de la simulation.

#### fonctionnalité v1

- On devra pouvoir éditer facilement l'ensemble des miroirs pour la simulation. Probablement via une simple description en JSON.
- On devra également pouvoir choisir la direction et le point de départ du rayon.
- On devra pouvoir visualiser aisément le trajet du rayon lumineux.
- On devra supporter les miroirs plans.
- La simulation devra au moins fonctionner en 2D.
 La V1 utilisera des bases locales et des symétries plutôt que des angles afin d'anticiper la généralisation en 3D.


#### fonctionnalité v2

- On devra supporter les types de miroirs :
    + plan
    + circulaire
    + en courbe de Bézier

#### fonctionnalité v3

- La simulation devra au moins fonctionner en 3D (ou ND).

#### Fonctionnalités v4

- on devra ajouter, selon les besoins, des fonctionnalités d'analyse de la trajectoire du rayon :
    + détection automatique de la sortie de l'ensemble.
    + détection automatique d'une boucle (le rayon passe 2 fois au même endroit)

## organisation temporelle

```mermaid
gantt
    title le diagramme de Gantt
    dateFormat  DD/MM/YYYY
    tickInterval 7day

    section Version 1
    Réflexion, création du cahier des charges    :done, a1, 01/02/2024, 7d
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


