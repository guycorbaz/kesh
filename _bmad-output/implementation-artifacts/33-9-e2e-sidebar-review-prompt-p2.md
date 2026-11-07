# Revue P2 ciblée — 33-9, la remédiation de la P1 (#424), une lentille (lecture seule)

Passe ciblée, contexte frais, français soigné. La remédiation ne porte que des textes : aucun code ni test n'a bougé depuis `d96707e2`.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 diff 1620b2de <tête> -- CHANGELOG.md docs CLAUDE.md _bmad-output/implementation-artifacts/33-9-e2e-sidebar.md
git -C /home/gcorbaz/devel/kesh-e337 log --oneline -3 story/33-9-e2e-sidebar
```

`<tête>` : le premier SHA du `log` ci-dessus. Le worktree porte une autre branche : lis par `git show story/33-9-e2e-sidebar:<fichier>`. Findings de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e339-review-p1-{B,E,A}.md`.

## Axes obligatoires

1. **Le CHANGELOG contre le code** (`frontend/src/routes/(app)/+layout.svelte`, `handleToggle`, `expandedState`, l'`$effect`, à la référence), phrase par phrase : navigation interne, prochain chargement, état déjà enregistré, clic qui le corrige. Le chemin d'un `'true'` parasite sur *Administration*, puis un clic.
2. **`docs/testing.md`, la note de la KF-052** : le numéro `:90`, « l'un des deux tirages », la réserve, contre les journaux `/home/gcorbaz/devel/kesh-gate-logs/e339/` (`e2e-rsonde2/`, `e2e-rvingt/`, `e2e-rsuite1/`, `e2e-rsuite2/`). Le calcul « 7 à 9 » et les relevés datés intacts.
3. **La fiche** : AC-3 précisé, Change Log : ce qu'il dit traité, écrit, ou renvoyé en issue ; un finding de la P1 oublié ?
4. **Le manuel** — `docs/manual/fr/user-manual.tex` et son PDF (`git show story/33-9-e2e-sidebar:docs/manual/fr/user-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`) : toujours rien sur la mémorisation des groupes ?

## Contraintes

⛔ **La lentille écrit son rapport à ce chemin, et à lui seul : `/home/gcorbaz/devel/kesh-gate-logs/e339-review-p2.md`.** Un rapport rendu par message ne compte pas. Lecture seule hors de ce fichier : ni redirection ni fichier temporaire (les tubes sont permis), aucune compilation ni exécution ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport porte : **les axes réellement exercés et ceux qui ne l'ont pas été** ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
