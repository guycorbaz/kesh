# Prompt — validation P1 de la spec, Story 15-1c

*Versionné le 2026-10-09. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0` = origin/main,
qui contient le socle du lettrage : 15-1a-i, PR #587, et 15-1a-ii, PR #593). **Fiche** :
`_bmad-output/implementation-artifacts/15-1c-proposition-ecran.md`. Première passe de CETTE fiche dans sa forme actuelle : l'écran « Postes ouverts » — consulter, lettrer, délettrer
(closes #518). Elle a été réécrite le 2026-10-08, AVANT la validation des fiches dont elle consomme les contrats ; ces
fiches sont désormais validées et ont écrit à son intention une section « Pour la 15-1c » (dans la 15-1b, points 1 à
11) : contrat JSON de la vue et des propositions, `reason` / `documentState` / `amountDue`, `manuallyLetterable`,
`inOpenPeriod`, `letteringOrigin`, `letterable` sur les comptes, refus 409 sur un compte devenu non lettrable, « au X »
non stable, la mise en garde de l'AC8 sur la Balance RÉFUTÉE (F-1 de la 15-1b P1). **Premier axe : relire la 15-1c
contre ces contrats** (fiches 15-1b et 15-1b-0, 15-1a-i pour `POST`/`DELETE /letterings`, 15-1a2-0 pour le refus) —
tout champ, code, route ou texte cité doit exister dans le code de `e892dcfa` ou dans une fiche validée qui le crée.
Puis : le frontend existant (`frontend/src`, routes, composants de comptes et de Grand livre), l'i18n ×4, le manuel
(`.tex` + PDF aplati), les E2E (Playwright) prescrits — un parcours de bout en bout lettrer → délettrer —, le message
`ENTRY_LETTERED` (C124 : l'écran de délettrage qu'il prescrit, c'est cette story), la règle de découpage.
Fiches voisines (contexte, ne pas valider) : `15-1a-i-marque-du-lettrage.md`, `15-1a-ii-gardes-du-lettrage.md`,
`15-1a2-lettrage-des-pieces.md`, `15-1b-vue-lignes-ouvertes.md`, `15-1c-proposition-ecran.md`. Issue (par
`gh api repos/guycorbaz/kesh/issues/518`) : #518. Registre : `epic-15-choix-autonomes.md` (C90–C131, C-15-1a-i-*,
C-15-1a-ii-*). Règles : `CLAUDE.md` (dont § « Un appariement automatique propose, il ne crée jamais » et § Migration
breaking policy).

## Lentilles

- **R — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md`. Chaque phrase de #518 qui
  revient à cette story couverte par un AC testable, chaque AC par une tâche, chaque tâche par un test ; implémentable
  sans deviner (fonctions, fichiers, codes d'erreur, clés i18n 4 locales, replis Svelte, migration éventuelle et son
  triage P5/P6/P7) ; faits cités revérifiés au code ; recompte des AC et des tâches ; frontière avec les fiches
  voisines (rien en double, rien entre deux chaises).
- **F — Full-scope adversary** : la conception elle-même, contre le socle livré. Les chemins qui écrivent ou lisent ce
  que la story change (inventorier les sites NON résolus) ; transactions, verrous, ordre des contrôles, cycles avec
  la création/dissolution de groupe, la contre-passation qui lettre, la clôture, la modification et la suppression
  d'écriture, les règlements et les avoirs ; tests existants qui changeront de sens ; frontend ; i18n ; le **manuel**
  (`docs/manual/fr/*.tex` et le PDF aplati `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`),
  `docs/api-external.md`, CHANGELOG ; la règle de découpage (§ « Règle de splitting préventif », amendement D5).

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (fiche et code), **preuve** (commande et
sortie, ou code cité relu), correction proposée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans
elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p1-<R|F>.md` ; dernier message : le
chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`,
`sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
