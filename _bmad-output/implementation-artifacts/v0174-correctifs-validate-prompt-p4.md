# Prompt — validation P4 des fiches v0174-correctifs (release 0.17.4), Haiku, passe ciblée sur la remédiation de la P3

Tu valides, en **passe ciblée** (`CLAUDE.md`, § *La passe ciblée*), la remédiation de la validation P3 des fiches de la story
`v0174-correctifs` de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), release corrective **0.17.4**, sans
migration. Worktree `/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs` ; le code est celui de `7bd4d5c0`
(seuls des fichiers `_bmad-output` ont changé depuis). Langue du rapport : **français**. Modèle : **Haiku**. **Une seule
lentille**, **contexte frais** : ne te fie ni aux fiches ni aux rapports antérieurs ; relis au code.

**Ton périmètre est le seul commit de la remédiation**, `git show 54949350` : ce qu'il ajoute, retire ou récrit dans
`_bmad-output/implementation-artifacts/v0174-correctifs.md` (index), `v0174-correctifs-1.md`, `-2.md`, `-3.md`,
`v0174-choix-autonomes-v0174c.md` et `sprint-status.yaml`. Le reste des fiches n'est relu que pour juger une ligne de ce commit
(contradiction, propagation manquante). Pour savoir ce qui était demandé, lis `/home/gcorbaz/devel/kesh-gate-logs/v0174c-validate-p3.md`
— pour le comprendre, non pour t'y fier.

## ⛔ Vérité de terrain obligatoire

Toute affirmation de **présence ou d'absence** (un texte qui serait ou ne serait pas dans une fiche, une ligne de code qui
porterait ou non un motif) s'appuie sur une sortie **`grep -nF "<chaîne exacte>" <fichier>`** copiée dans le rapport — `-F`
obligatoire (les métacaractères de Rust et de TypeScript fausseraient une regex). Lis le **fichier final**, non les numéros de
ligne d'un hunk de `git show` : un finding fondé sur l'index d'un diff est un faux positif connu de ce modèle.

## Axes

1. **Chaque ligne du diff est-elle vraie au code ?**
   - fiche 3, tâche 3 : `CLES_RELEVEES` 399 → 401 — relis `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts:51-72`
     (`PREFIXES`) et `:232` ; des trois clés neuves, lesquelles portent un préfixe relevé ? La commande de recompte de
     `sitesTotal` (`grep -oE '(i18nMsg|[^a-zA-Z]msg)\('`) est-elle celle des en-têtes `i18n-keys.test.ts:701-799`, et
     `i18n-literal-reader.js:480` (`noms`) la justifie-t-il ?
   - fiche 3, tâche 4 (doc de `local-date.ts`) et § 3.5 (inventaire `ContactResponse` « à trier ») : cohérents avec la
     conception de la garde et avec `git grep -n 'ContactResponse' -- frontend/src` ?
   - fiche 2 : `routes/bank_imports.rs:1472` (`drop(lock_tx)`) et `:1479-1481` ; `crates/kesh-api/src/inbox_import.rs:177-186` ;
     le second grep de contrôle rend-il bien ces sites et `backup.rs:619` ? La liste de `docs/api-external.md:493` après retrait de
     `PERIOD_LOCKED` compte-t-elle quatre codes, cohérents avec « 7 avec et 4 sans » ?
   - index : la puce #817 du CHANGELOG — `POST /api/v1/onboarding/coordinates` existe-t-il (`crates/kesh-api/src/lib.rs:1168`) ?
     L'annotation datée de la P1 est-elle juste, et l'entrée de la P1 laissée telle quelle ?
2. **Propagation** : l'ancienne valeur (`402`, `:1478`, `seul le **début**`, `grep -o 'i18nMsg('`, `'): ContactResponse'`) ne
   subsiste-t-elle que dans des entrées datées des Change Logs ? `sprint-status.yaml` (`:1` et la clé `v0174-correctifs`)
   suit-il ?
3. **Le commit introduit-il un défaut neuf** (contradiction entre fiches, AC indécidable, prémisse fausse) ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois** (outil Write), à `/home/gcorbaz/devel/kesh-gate-logs/v0174c-validate-p4.md`.
Il contient :

1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), la fiche, le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède. Une affirmation de présence
   ou d'absence sans sortie copiée ne compte pas.
2. **Les axes exercés** et **non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne
   compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture** hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`, `tee`),
  pas même dans le scratchpad. Un fichier créé par erreur se **déclare** ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`.
- Aucune commande `gh` qui écrit ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `mariadb`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt
  (les lire avec `cat` est permis).
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/Projets actuels/kesh/`, ni le NAS ;
  aucune donnée réelle dans le rapport.
- `grep` est une fonction vers `ugrep` dans ce shell : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture),
  `git log`, `git show`, `git diff`, `pdftotext … -` vers la sortie standard, `gh api` en `GET`.
