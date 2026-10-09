# Prompt — validation P3 de la spec, Story 15-1a2-i

*Versionné le 2026-10-09. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0` = origin/main,
qui contient le socle du lettrage : 15-1a-i, PR #587, et 15-1a-ii, PR #593). **Fiche** :
`_bmad-output/implementation-artifacts/15-1a2-i-lettrage-des-pieces-clients.md`. **Troisième passe** ; P1 (Sonnet ×2) et P2 (Opus ×2) remédiées, la P2 par `7b8a3e6d` (un seul remédiateur pour les
trois fiches du lettrage, décision de REFUS au délettrage — C-15-1a2-2 révisée deux fois, C-15-1a2-10 à 18, C-15-1b-9 à
12 —, extraction de la 15-1b-0). Rapports : `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-i-validate-p{1,2}-{R,F}.md`.
**La sévérité se déplace vers la dernière remédiation : relis d'abord ce que `7b8a3e6d` a écrit dans CETTE fiche**,
contre le code de `056997b0`. En priorité : le rang 2 bis (`document_group_frozen_by_periods`, lu sur l'écriture examinée) couvre-t-il le groupe que la dissolution trouvera ensuite ? `reconciliation_cancel::cancel_in_tx` étape (4) refuse-t-il AVANT de défaire le lien ? l'affirmation « l'étape 5 est inatteignable, rien ne dissout un groupe hors période sauf la tolérance `lock_books` » est-elle vraie au code ? le réemploi du code `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (C-15-1a2-11, confirmé par l'orchestrateur) est-il rendu sans ambiguïté à l'écran et dans `api-external.md` ? la dérogation à 6 modules (C-15-1a2-13) tient-elle ?
Toute fonction, tout nom, code d'erreur ou colonne cités doivent exister (`grep -nF`) ou être dits créés.
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
elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-i-validate-p3-<R|F>.md` ; dernier message : le
chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`,
`sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
