# Release 0.20.0 — choix autonomes de la Story 49-4a (clé `e494a`)

Registre de la reprise du développement de `49-4a-capacite-pdf-avant-validation` (Opus 5.5, `bmad-dev-story`, 2026-11-24).
Chaque choix : contexte, option retenue, autres options, réversibilité.

## C-e494a-1 — Mutations par l'outil Edit, binaires copiés pour l'E2E

- **Contexte** : la consigne impose l'outil Edit pour le code ; chaque mutation demande une reconstruction, et la voie base
  est partagée avec trois autres agents (file d'attente de 20 à 40 minutes).
- **Retenu** : un script en lecture seule du dépôt (`kesh-gate-logs/e494/dev/holder.sh`) tient le verrou de la voie base et
  exécute à la demande le filtre déposé ; les mutations restent des `Edit` suivis de `git checkout -- <fichier>` et `touch`.
  Pour T6, les binaires sain et muté sont copiés une fois (`kesh-api-sain`, `kesh-api-mute`) et le Playwright se joue hors verrou
  de base, sous `.gate-front.lock`, sur un port propre (3494).
- **Écartées** : une mutation par file d'attente (une heure chacune) ; un `sed` sur le code (interdit).
- **Réversibilité** : sans objet (outillage hors dépôt).

## C-e494a-2 — SPA servie depuis le build de l'épique

- **Contexte** : le worktree n'a ni `node_modules` ni `frontend/build`, et la story ne change aucun fichier de `frontend/src`.
- **Retenu** : `KESH_STATIC_DIR` pointe sur le build de `/home/gcorbaz/devel/kesh-e49/frontend/build` (état `87e00ba05`, `frontend/src`
  identique) ; `node_modules` de ce même worktree lié par un lien symbolique le temps du Playwright, retiré ensuite (`git status` vide).
- **Écartées** : `npm ci` (réseau, minutes, sans apport) ; un build neuf.
- **Réversibilité** : totale.

## C-e494a-3 — Le rouge par mutation, non avant le code

- **Contexte** : le code et les tests avaient été écrits avant la pause, sans compilation.
- **Retenu** : le premier vert du gate ciblé est journalisé tel quel (`cible-1.log`) ; chaque test est ensuite rendu rouge par une
  mutation qui le vise (onze, au Dev Agent Record). Écart déclaré à « test rouge avant code ».
- **Écartées** : retirer le code pour constater un rouge d'avant (équivalent à T1-1 et plus coûteux).
- **Réversibilité** : sans objet.

## C-e494a-4 — Manuels : phrase de la 49-4a substituée à celle de la 49-1c sur la capacité

- **Contexte** : à la fusion, `user-manual.tex` portait deux phrases sur « huit lignes au plus » (49-1c, texte seul ; 49-4a,
  arrondi, refus à la validation, limite du rappel).
- **Retenu** : celle de la 49-4a, plus complète, après « Elle s'imprime sur toute la largeur du PDF… » (la 49-1c y disait « chaque ligne
  imprimée prend la place d'une ligne de facture, … huit au plus, moins avec le récapitulatif de TVA » : redit une fois, plus juste).
- **Écartées** : garder les deux (redite).
- **Réversibilité** : totale (texte).
