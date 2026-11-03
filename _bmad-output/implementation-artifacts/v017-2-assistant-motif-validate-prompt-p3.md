# Prompt — validation P3 de la fiche v017-2 (release 0.17.0 : #809), passe ciblée, une lentille, Sonnet

Tu valides la **remédiation de la validation P2** de la fiche de la Story v017-2 de Kesh (comptabilité suisse,
Rust/Axum, SvelteKit, MariaDB 10.11) : l'assistant d'installation affiche le motif d'un refus du serveur (#809, P3).
La P2 (Opus, deux lentilles) a rendu 0 CRITICAL, 0 HIGH, 2 MEDIUM, 13 LOW, tous remédiés. Elle a aussi relevé que la
remédiation P1 avait oublié deux propagations (une ligne de mutation, une annotation du registre) : **c'est le défaut
que tu chasses**.

Worktree `/home/gcorbaz/devel/kesh-0172`, branche `story/v017-2-assistant-motif`. Langue du rapport : **français**.
Modèle : **Sonnet**. **Passe ciblée** : une seule lentille, en **contexte frais** — ne t'appuie ni sur ce que la fiche
dit d'elle-même, ni sur les passes antérieures au-delà de ce qui est demandé ci-dessous.

## Ton périmètre

1. Le diff de la remédiation P2 : `git diff 8c7e3287..HEAD -- _bmad-output/implementation-artifacts/` (fiche
   `v017-2-assistant-motif.md`, registre `v017-2-assistant-motif-choix-autonomes.md`, `sprint-status.yaml`).
2. Les deux rapports de la P2, pour savoir ce que chaque correction devait faire :
   `/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-validate-p2-R.md` (R1 à R7) et
   `/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-validate-p2-F.md` (F1 à F8).
3. Le code que les corrections citent, pour vérifier chaque citation neuve.

## Ce que tu vérifies

### A. Chaque correction P2, une par une (R1-R7, F1-F8)

- La correction traite-t-elle le finding, **et seulement lui**, sans contredire une autre section de la fiche ?
- Chaque `fichier:ligne` **neuf** du diff dit-il ce que la fiche lui fait dire ? En particulier :
  `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts:180` et `:262` ; `frontend/src/lib/shared/i18n-keys.test.ts:767`
  et `:700-707` ; `crates/kesh-api/src/config.rs:848` ; `crates/kesh-api/src/routes/i18n.rs:20-22` ;
  `frontend/src/lib/shared/utils/i18n.svelte.ts:41-45` ; `crates/kesh-api/src/routes/onboarding.rs:874-875` et
  `:1238-1298` ; `frontend/tests/e2e/helpers/test-state.ts:92, 125` ; `frontend/src/lib/shared/utils/error-message.ts:20-22` ;
  `docs/manual/fr/user-manual.tex:144-158` ; `docs/manual/fr/admin-manual.tex:757` ; `frontend/src/routes/onboarding/+page.svelte:141, 159, 339, 356`.
- **AC-4, forme fixée** : le bloc de code prescrit compile-t-il dans `+page.svelte` tel qu'il est (imports présents :
  `isApiError`, `toast`, `msg` ; `errorMessageOf` à importer) ? Garde-t-il les replis actuels au caractère près
  (`+page.svelte:141-159`) ? Le delta annoncé de `sitesTotal` (+1 pour `+page.svelte` : +1 S1, +1 indication, −1
  finalisation) est-il juste, compte tenu de **toutes** les écritures `msg(`/`i18nMsg(` que la story ajoute ou retire
  (AC-2, AC-6, AC-4) ? `CLES_RELEVEES` 344 → 346 : les deux clés neuves entrent-elles bien dans le domaine relevé
  (`PREFIXES`, `i18n-un-repli-par-cle.test.ts:65`), et aucune autre clé neuve n'y entre-t-elle ?
- **La capture de-CH sous `KESH_LANG=de`** : la prescription est-elle jouable avec la recette E2E du dépôt
  (`CLAUDE.md` du worktree, § *E2E (Playwright)*) sans toucher la base du NAS ? `KESH_LANG=de` est-elle la valeur que
  `config.rs` accepte (et non `de-CH`) ?
- **La capture de la finalisation par `page.route`** : la forme du corps simulé est-elle celle que lit
  `parseErrorResponse` (`api-client.ts:238-270`) ?
- **L'issue demandée (F2)** : le constat est-il exact au code et au manuel ?
- **L'incise au manuel (F6)** et la phrase « langue de l'interface » (F2) sont-elles vraies au code ?

### B. La propagation — axe obligatoire

Pour **chaque** correction P2 : a-t-elle été propagée partout où la même chose est écrite ? Cherche-la, en grepant la
**valeur** (un nombre, un nom de fichier, une clé, une ligne), non la formulation :

- dans la fiche : AC, *Tâches*, *Tests et mutations prévus*, **tableau des mutations** (M1 à M12 : chaque test neuf ou
  modifié a-t-il sa mutation, chaque mutation dit-elle le bon test ?), *E2E*, *Captures*, *Ce que les gardes ne voient
  pas*, *Fichiers touchés*, *Issues à ouvrir*, *Change Log* (l'entrée « Validation P2 » dit-elle vrai, et ses décomptes —
  2 MEDIUM, 13 LOW, la liste des LOW — se recomptent-ils depuis les deux rapports ?) ;
- dans le **registre** : les choix touchés (C-v017-2-1, -2, -3, -4, et tout autre que la correction concerne) portent-ils
  une annotation datée, **sans réécriture** du texte d'origine ?
- dans `sprint-status.yaml` : le commentaire est-il exact ?
- les entrées passées du Change Log (création, Validation P1) sont-elles **intactes** (`git diff 8c7e3287..HEAD` ne doit
  y montrer aucune ligne retirée) ?

Exemples de commandes : `grep -nF "CLES_RELEVEES" …`, `grep -nF "sitesTotal" …`, `grep -nF "KESH_LANG" …`,
`grep -nF "MOTIF-DEJA" …`, `grep -nF "M12" …`, `grep -nF "disposeContextSafe" …`, `grep -nF "874" …`.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-validate-p3.md`.
Il contient :

1. **Les findings**, numérotés `P3-1`, `P3-2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) qui
   fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF` copiée ne
   compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un — chaque correction P2 (R1-R7, F1-F8) sous l'axe A, chaque
   lieu de propagation sous l'axe B — avec, pour chaque axe exercé, ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni
  `latexmk`, ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
