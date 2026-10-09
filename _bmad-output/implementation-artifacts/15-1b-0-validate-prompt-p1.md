# Prompt — validation P1 de la spec, Story 15-1b-0

*Versionné le 2026-10-09. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0` = origin/main,
qui contient le socle du lettrage : 15-1a-i, PR #587, et 15-1a-ii, PR #593). **Fiche** :
`_bmad-output/implementation-artifacts/15-1b-0-propriete-des-lignes-par-lot.md`. Première passe de CETTE fiche, créée par `7b8a3e6d` (extraction de la T2 de la 15-1b, signal D5 de sa validation P2) :
une **refonte du socle** — `document_owners` par lot qui devient la seule source de `reversal_blockers` et
`first_document_owner`, signature `&mut MySqlConnection`, inventaire fermé des appelants (6 sites de production, 6
appels de test), route de contre-passation en transaction de lecture, oracle INDÉPENDANT (requête gelée de `056997b0`
dans un module de test + valeurs écrites à la main), trois mutations. Axes prioritaires : l'inventaire des appelants
est-il fermé (`grep -rn` de chaque fonction sur `crates/`) ? l'oracle est-il vraiment indépendant (pas vert par
construction) ? AC1 « 3 requêtes pour 1 200 écritures » : comment compte-t-on les requêtes, est-ce testable ? la
refonte change-t-elle un ordre de verrous, un résultat de `reversal_blockers` pour un cas existant (tests existants
qui changeraient de sens) ? exception kesh-db (gate complet) écrite ?
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
elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1b-0-validate-p1-<R|F>.md` ; dernier message : le
chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`,
`sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
