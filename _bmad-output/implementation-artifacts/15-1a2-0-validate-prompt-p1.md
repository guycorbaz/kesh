# Prompt — validation P1 de la spec, Story 15-1a2-0

*Versionné le 2026-10-09. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0` = origin/main,
qui contient le socle du lettrage : 15-1a-i, PR #587, et 15-1a-ii, PR #593). **Fiche** :
`_bmad-output/implementation-artifacts/15-1a2-0-lettrage-fige-avec-la-periode.md`. Première passe de CETTE fiche, créée par `76e7893a` (extraite de la 15-1a2-i — F-1 de sa validation P3) : le
**refus** des annulations qui dissoudraient un groupe de lettrage `document` dont aucune ligne n'est en période ouverte
(rang 2 bis de `SettlementCancelBlocker`, `document_group_frozen_by_periods`, règle `open_period_rule`), dans QUATRE
gestes (règlement et solde clients, dé-rapprochement, `cancel_settlement_in_tx` et `cancel_in_tx` fournisseurs), code
réemployé `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (C-15-1a2-11), textes ×4, écran, manuel (deux listes exhaustives de
motifs), `api-external.md` (§ 10 `:566`, listes `:426`/`:434` — l'écart préexistant est l'issue #595), remède précis
(C-15-1a2-20). **Dormante** : aucune production ne pose encore l'origine `Document` ; elle s'éprouve en SQL brut.
Historique de la décision : C-15-1a2-2 révisée deux fois, C-15-1a2-10. Dérogation de découpage : C-15-1a2-23. Axes
prioritaires : l'ordre des refus dans chaque file (le rang 2 bis avant ce qu'il doit précéder, après ce qu'il doit
suivre) et les prédicteurs (un prédicteur ne doit jamais annoncer un motif que le clic ne refuse pas, ni l'inverse) ;
le prédicat lu sur l'écriture examinée couvre-t-il le groupe que la 15-1a2-i dissoudra ? le test en SQL brut prouve-t-il
le refus (groupe posé à la main) et ne passe-t-il pas à vide ? la propagation des textes (dérogation : axe à part entière).
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
elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-0-validate-p1-<R|F>.md` ; dernier message : le
chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`,
`sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
