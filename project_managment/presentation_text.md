# Objectifs globaux du projet (GUIGZ)

Ce projet fait suite à la demande d'un étudiant, Quentin COURDEROT en troisième année en spécialité informatique.
Celui-ci a demandé à son enseignant Jérôme Bastien de l'aider à écrire un algorithme pour déterminer la trajectoire d’un rayon lumineux lorsque celui-ci vient frapper un miroir plan fini.

Le but de notre projet est donc, tout d'abord, comme il l'a demandé à son enseignant, de simuler des rayons lumineux avec un miroir. On définira plus tard quels miroirs nous choisirons.
On utilisera ces simulations pour étudier le comportement des rayons lumineux dans un grand ensemble de miroirs.

# Les grandes Parties (EYMERIC)

Les grandes parties ou les milestone que nous devrons atteindre se découpent en trois principales versions du simulateur.
On commencera par développer une version 1 en 2 dimensions avec des miroirs plan fini comme la demandée l'élève originellement.
Ensuite une fois que cette version fonctionnera, sera optimisé et testé dans un maximum de cas, on passera à la version 2 avec des miroirs plus complexe.
On commencera par des miroirs circulaires et on envisagera des miroirs plus complexes en courbe de Bézier, par exemple, selon la difficulté d'implémentation.
On pourra ensuite, dans une troisième tant, développer le simulateur en 3D ou même en nd si cela n'est pas trop complexe.
Enfin, s'il nous reste du temps, on intégrera des outils automatisés d'analyse automatique de trajectoire tel qu'un détecteur de boucle par exemple.


# Moyens, verrous (MOMO TU VENDS RUST)

Pour développer ces différentes versions du simulateur, nous utiliserons le langage de programmation Rust.
Ce dernier est un
Langage de choix pour créer des programmes très rapides, notamment grâce à sa nature de langage compilé.
Les programmes ecrits en Rust, sont depourvus de failles de securite de memoire, erreurs d'acces, ou tout type de comportement non-defini, en conjonction avec son systeme de gestion d'erreur, et ses fonctionalites de tests automatises, ils sont surs, robustes et simples a debugger. 

Cependant, tous ces avantages viennent avec une difficulté de programmation supérieure ont un langage plus haut niveau et moins rapide
On pourra rencontrer des difficultés, par exemple, sur l'affichage 3D avec wgpu.
Cela risque un potentiel retard. C'est pourquoi nous rajouterons impérativement une fonction d'export des résultats dans
un fichier JSON afin de pouvoir récupérer les données de la simulation sans passer par la visualisation interne au
programme.
Cette fonctionnalité permettra en outre d'exporter ces données dans un autre logiciel pour poursuivre l'étude au-delà.
Dans le cadre de ce projet.

Les autres verrous intrinsèques au projet sont le calcul efficace de l'intersection entre le rayon lumineux et les miroirs.

La généralisation en 3D risque également de ne pas être simple et demandera probablement beaucoup de modifications du code.

# Exigences à atteindre (EYMERIC)

Le simulateur devra supporter deux principales exigences : 
Tout d'abord, les simulations devront évidemment être physiquement justes.
C'est-à-dire que si on reproduit le système de miroir dans la vraie vie, le rayon doit exactement suivre le même chemin que dans la simulation.

Le simulateur devra également pour calculer les 100 réflexions dans un ensemble de plus de 100 miroirs en maximum une seconde afin de pouvoir faire des simulations très chargées.

# Calendrier prévisionnel (GUIGZ)

demerden sie sich