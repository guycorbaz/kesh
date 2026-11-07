# Revue de code P2 ciblée — Stories 33-3a et 33-3b, une lentille sur la remédiation de la P1

Contexte frais, une seule lentille. Modèle : Sonnet ou Opus (Haiku n'est permis qu'à une passe ciblée : celle-ci l'est, mais la remédiation touche deux couches et une règle métier, préférer Sonnet ou Opus).

**Pourquoi une passe ciblée** : la sévérité se déplace vers ce qu'on vient d'écrire (`CLAUDE.md`, *La passe ciblée*). Ne relis pas les deux stories : relis **le diff de la remédiation**, et ce qu'il casse.

- **Worktree** : `/home/gcorbaz/devel/kesh-e333`, branche `story/33-3-accueil`.
- **Diff à relire** : `git diff 1e76cda7 HEAD` (`1e76cda7` est le dernier commit relu en P1).
- **Rapports de la P1**, à lire pour contrôler la remédiation : `/home/gcorbaz/devel/kesh-gate-logs/e333a-review-p1-{B,E,A}.md`, `e333b-review-p1-{B,E,A}.md`. **Comptes rendus** : Change Log et Dev Agent Record de `33-3a-accueil-agregats.md` et `33-3b-accueil-graphiques.md`, registre `e33-choix-autonomes-e332.md` (C-e332-9, C-e332-26).
- **Journaux** : `kesh-gate-logs/e333a/mut/`, `e333b/mut/`, `e333/gate4.out`, `e333b/e2e-complet-r6.log`, le rejeu de M7 `e333b/m7-e2e.log`. **Captures** : `kesh-gate-logs/e333b/captures/`, à regarder, `reel-six-parts-1280.png` et `pres-de-100-1280.png` d'abord, puis les `*-fenetre.png`.
- **Rapport, seul fichier écrit** : `/home/gcorbaz/devel/kesh-gate-logs/e333-review-p2.md`.

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni `python3`, ni `scripts/prepare-release.sh`, ni aucun script du dépôt ; ni `git checkout`, `reset`, `stash`, `push`, `tag` ; ni `gh` qui écrit ; ni `rm`. **Aucune redirection, aucun fichier temporaire** hors du rapport. `grep` est une fonction vers `ugrep` : `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence porte une sortie `grep -nF` copiée. Aucune donnée réelle.

## Les axes

1. **La paire à date anormale** (`REVERSAL_PAIR_EXCLUSION`) : ordre des liaisons `?` dans les deux requêtes ; le test neuf et M19a, M19b. L'invariant tient-il maintenant sans réserve, aussi pour une paire dont **les deux** côtés sont hors bornes, et pour une paire à cheval dont un côté est hors bornes ? Lis `income_statement.rs` contre la requête.
2. **`full` par extrémités confondues** (`pieSlices`) : une part à 100 % exacte y entre-t-elle toujours (`Math.sin(2π)`, arrondi à trois décimales) ? Une part de 50 %, une part minuscule y entrent-elles à tort ? Plusieurs parts, la grande n'étant pas la première ? Le `<circle>` et la minuscule dessinée après : que voit-on ? Le trait coloré (E-5) ne cache-t-il pas une petite part ?
3. **Les trames** (`patternTransform`), `chartAmount` (quatre décimales : tableaux, titres, description), `niceStep` (2,5 écarté seulement sous 0,1), `max-w-*` du dessin, `[overflow-wrap:anywhere]`, les jetons en `var(--color-…)` dans des attributs de présentation.
4. **Les deux clés au singulier et le compte des comptes** (B-5) : les quatre locales, variables égales à l'appel, fallback égal au `.ftl` français ; un compte unique avec `otherExpenses` est-il impossible ? Le changement de `postes` en `comptes` se relit-il juste aux quatre langues ?
5. **Les E2E** : `homepage-charts.spec.ts`, scénarios 2, 3, 6, 6b : prouvent-ils ce que dit leur titre ? Le point cliqué au scénario 3 est-il dans la part ? Les aides d'`api-fixtures.ts` : `createExpenseEntryViaApi` et un compte existant d'un autre type.
6. **Le manuel, PDF compris** : `docs/manual/fr/user-manual.tex` et `.pdf` aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) : « trois dixièmes au plus », et le test qui en atteint la borne ; `docs/api-external.md`.
7. **C-e332-26** et le commentaire de `MonthlyBarsCard.svelte` : les mesures citées égalent-elles les captures et le journal `e333b/caps5.log` ? Les captures simulées se disent-elles telles, partout ?
8. **Les comptes rendus** : les décomptes (tests, mutations, clés) se recomptent ; la colonne « Tuée par » sépare-t-elle l'observé et le déduit ? Un compte rendu de passe ancien a-t-il été réécrit (interdit) ?
9. **Propagation** : `fiscal_year_id = je.fiscal_year_id` et « d'un dixième », `postes de charges` dans `docs/`, `crates/kesh-i18n/`, `frontend/src/`, le registre.

## Le rapport

Décompte CRITICAL / HIGH / MEDIUM / LOW, recompté. Chaque constat : fichier:ligne, sortie citée, conséquence, remédiation proposée, et **s'il naît de la remédiation ou lui préexiste**. **La liste des axes exercés et de ceux qui ne l'ont pas été** : une passe qui ne la déclare pas ne compte pas. Une boucle ne se clôt que sur une passe sans constat au-dessus de LOW.
