# Prompt — validation P2 de la spec, Story 15-1a2-ii

*Versionné le 2026-10-09. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0` = origin/main,
qui contient le socle du lettrage : 15-1a-i, PR #587, et 15-1a-ii, PR #593). **Fiche** :
`_bmad-output/implementation-artifacts/15-1a2-ii-fournisseurs-et-rattrapage.md`. **Deuxième passe** ; la P1 (Sonnet ×2) a été remédiée par `7fca24f9` (15-1a2, découpée en 15-1a2-i et 15-1a2-ii ;
index `15-1a2-lettrage-des-pieces.md`, choix C-15-1a2-1 à 9) et `e1fa4b6c` (15-1b, choix C-15-1b-1 à 8). Rapports P1 :
`/home/gcorbaz/devel/kesh-gate-logs/15-1a2-validate-p1-{R,F}.md`, `15-1b-validate-p1-{R,F}.md`. **La sévérité se
déplace vers la dernière remédiation : relis d'abord ce qu'elle a écrit**, contre le code de `056997b0` (socle livré :
`letterings.rs`, `journal_entries.rs` gel `ENTRY_LETTERED` et R6, migration `20261009000001`, `post_restore.rs`).
Toute fonction, tout nom, code d'erreur ou colonne cités doivent exister (`grep -nF`) ou être dits créés.
Axes propres : les DEUX migrations de rattrapage (M1 au registre de rejeu en classe A, M2 exemptée `Durable`,
justification sans « Hors fenêtre ») — l'extrait porte-t-il toutes les écritures de sa migration
(`extract_carries_every_write_statement_of_its_source_migration`) ? la classe A est-elle légitime (P7 : chaque statement
gardé contre l'écrasement d'un choix de l'utilisateur) ? les paires `reversal` d'achats de factures fournisseurs
annulées placées en M1 (C106) ; compteurs P5 76 → 78, `EXEMPT_MIGRATIONS` 16 → 17 ; P6 ; bump ou non de
`min_required` (P1–P2-bis : une migration de DONNÉES n'est pas breaking, vérifie) ; `confirm_batch` → `pay_in_tx`.
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
elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-ii-validate-p2-<R|F>.md` ; dernier message : le
chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`,
`sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
