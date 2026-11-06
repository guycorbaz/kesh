# Revue de code P1 — Story 33-3b (les graphiques de l'accueil), trois lentilles B, E, A

Contexte frais. Modèle : Sonnet ou Opus, jamais Haiku pour une première passe. Une lentille par agent.

- **Worktree** : `/home/gcorbaz/devel/kesh-e333`, branche `story/33-3-accueil`. **Base du diff** : `97162a47` (la 33-3a, relue à part). `git diff 97162a47 HEAD`.
- **Fiche** : `_bmad-output/implementation-artifacts/33-3b-accueil-graphiques.md`. **Choix** : `e33-choix-autonomes-e332.md` (C-e332-16 à 26, sauf 21, 22).
- **Contrat** : `33-3a-accueil-agregats.md`, *Contrat*.
- **Captures, à regarder** : `/home/gcorbaz/devel/kesh-gate-logs/e333b/captures/` : `peuple-12-1280`, `peuple-12-360`, `exercice-36-1280`, `exercice-36-360`, `exercice-132-1280`, `mois-negatif-1280`, `part-100-note-1280`, `vide-1280`, `aucun-exercice-1280`, `chargement-1280`, `echec-1280`, `tableau-ouvert-1280`, `focus-clavier-1280`, `de-CH-1280` (`.png`).
- **Journaux** : `kesh-gate-logs/e333/rouges.log`, `kesh-gate-logs/e333b/mut/`, le squelette de T0 (`kesh-gate-logs/e333b/t0.log`, `squelette.spec.ts.joue`), l'E2E (`kesh-gate-logs/e333b/e2e-complet*.log`).
- **Rapports** (un par lentille, seul fichier écrit) : `/home/gcorbaz/devel/kesh-gate-logs/e333b-review-p1-B.md`, `…-E.md`, `…-A.md`.

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni `python3`, ni `scripts/prepare-release.sh`, ni aucun script du dépôt ; ni `git checkout`, `reset`, `stash`, `push`, `tag` ; ni `gh` qui écrit ; ni `rm`. **Aucune redirection, aucun fichier temporaire** hors du rapport. `grep` est une fonction vers `ugrep` : `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence porte une sortie `grep -nF` copiée. Aucune donnée réelle.

## Les lentilles

- **B (Blind Hunter)** : le diff seul. Géométrie (arcs, grand arc, cumul, échelle), `Big` et `Number`, réactivité de Svelte (`$derived`, constantes figées à l'initialisation), identifiants de `<pattern>` et de `<title>` uniques, type union du résumé.
- **E (Edge Case Hunter)** : le diff et le code autour. Un montant énorme, un montant à quatre décimales sous le centime sur l'axe, un mois tout négatif, une part de 0,0001, cinq parts plus « Autres » à la frontière de la palette, un nom de compte très long ou porteur de balises (échappement), un exercice d'un mois, de 13, de 36, de 37, de 132 mois, la graduation `0`, deux `<svg>` de légende à motif `url(#…)` hors du SVG qui les définit (la carte de camembert sans dessin en a-t-elle ?), un clic modifié (`Ctrl`) sur une marque, un `goto` sur un exercice clos entre-temps.
- **A (Acceptance Auditor)** : la fiche contre le code et les tests : AC1 à AC22, AC2-bis, T0 à T6, mutations M1 à M20 (`M13`, `M17` jouées en deux variantes : 22 lignes au journal). Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Axes à exercer par toutes

1. **Accessibilité** : aucun descendant focalisable dans un `<svg role="img">` ; tableaux, `caption`, `th scope` ; la couleur jamais seule ; le parcours au clavier d'AC13 ; le squelette et sa sonde de T0.
2. **Le manuel, PDF compris** : `docs/manual/fr/user-manual.tex`, § *Tableau de bord* et *Revenus, dépenses et principaux postes*, et `docs/manual/fr/user-manual.pdf`, aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), contre le code et les captures. Toute phrase, y compris les deux limites écrites (C-e332-9, C-e332-14). `README.md` (retrait de #165 et #166), `CHANGELOG.md`.
3. **Les quatre locales** : `homepage-charts-*` dans `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`, sens et variables `{ $… }` identiques à l'appel, fallback français égal au `.ftl`.
4. **C-e332-26** : à 360 px, l'application reste à 1280 px (`min-w-[1280px]`). Le choix de ne rien adapter tient-il ? La fiche demandait aussi une lecture à 360 px : ce qu'on y voit est-il dit vrai ?
5. **`formatSwissAmount`** (`journal-entries/balance.ts`) : fichier partagé, à deux décimales rien ne bouge-t-il ?

## Le rapport

Décompte CRITICAL / HIGH / MEDIUM / LOW, recompté. Chaque constat : fichier:ligne, sortie citée, conséquence, remédiation proposée. **La liste des axes exercés et de ceux qui ne l'ont pas été** : une passe qui ne la déclare pas ne compte pas.
