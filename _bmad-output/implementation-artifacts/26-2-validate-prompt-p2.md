# Prompt — validation P2 de la spec, Story 26-2 (description d'un compte bancaire)

*Versionné le 2026-10-10. Deux lentilles (**Opus**), contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
P1 Sonnet → **P2 Opus** (passe complète) ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-26-2`, branche `story/26-2`. Fiche :
`_bmad-output/implementation-artifacts/26-2-description-compte-bancaire.md` — **une fiche, deux stories** :
26-2a socle (AC1-8, PR `refs #622`) puis 26-2b écrans (AC9-12, PR `closes #622`), cf. C-26-2-15.
**Commit de la remédiation P1 à relire : `f7f3e212`** (`git show f7f3e212`) ; base de la fiche `1c85e499`, découpage
`278119fe`. Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/26-2-validate-p1-R.md` et `…-p1-F.md` (8 MEDIUM, 15 LOW,
tous déclarés traités — bilan au Change Log de la fiche). Registre : `epic-26-choix-autonomes.md`, C-26-2-1 à 25
(neufs : 16 à 25). Epic : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md`. Issue #622
(`gh api repos/guycorbaz/kesh/issues/622` et `…/622/comments`).
Règles : `CLAUDE.md` (§ Review Iteration Rule et sa passe ciblée, § Le prompt d'une passe doit NOMMER le manuel,
§ Inventorier les sites NON RÉSOLUS, § Propagation post-patch, § Recompter ses propres comptes rendus, § Migration
breaking policy P1-P8). Voisines : la **26-1** (profil CSV, migration `20261011000001`, fusionne la première, bumpe
`kesh_version_min_required` à 0.13.1) et la **26-3** (manuels, garde de `kesh-restore.sh`).

**Le motif à chasser** : *la sévérité se déplace vers ce qu'on vient d'écrire.* La P1 n'a trouvé aucun HIGH ; la
remédiation a réécrit la garde AC11, AC1 (migration après la 26-1), AC2, AC3, AC4, AC9, AC12, AC13, AC14. C'est elle
qu'il faut relire d'abord.

## Axes prioritaires

1. **La remédiation `f7f3e212` elle-même** — chaque ligne du bilan P1 (Change Log de la fiche) : le constat est-il
   réellement fermé dans les AC et les tâches, pas seulement dans le bilan ? La correction a-t-elle introduit une
   contradiction ailleurs dans la fiche (Status, tableau de découpage, AC, tâches, Dev Notes, registre) ? Greper les
   **valeurs** : `20261011000001`/`20261011000002`, `\b(78|79|80)\b`, `\b(44|45|46)\b`, `\b60\b`, `AFTER`,
   `C-26-2-14`, `bank_label`, `BankAccountOptionSource`.
2. **La garde AC11 réécrite** (C-26-2-18) : décompte du jeton `bankName` par fichier. Est-elle implémentable telle
   qu'écrite (collecte Vitest de fichiers `.svelte`/`.ts`, exclusion des tests) ? La table attendue est-elle
   **dérivable** de l'état final (refais le décompte actuel : `grep -row bankName frontend/src --include=*.svelte
   --include=*.ts | grep -v "\.test\.ts:\|\.spec\.ts:" | cut -d: -f1 | sort | uniq -c`, et projette l'état après
   AC7/AC9) ? Que laisse-t-elle passer encore ? Interaction avec la 26-1, qui retouche les profils d'import.
3. **Migration après la 26-1** (C-26-2-25) : la procédure de rebase et de recompte (P5 cinq compteurs, P6
   `migrations_upgrade_path.rs` total/soustracteur/généalogie, squash, **`crates/kesh-db/migrations.sha384`** et le
   test `published_migrations_keep_their_checksums` de `crates/kesh-db/tests/test_schema_guard.rs`) est-elle
   complète et exécutable ? Que se passe-t-il si la 26-2a doit commencer avant la fusion de la 26-1 ?
4. **Ce que la P1 n'a pas exercé** (ses propres déclarations) : pertinence et faisabilité des mutations **M1-M16 +
   M8-bis** (chacune rougit-elle le test nommé ? M13 en particulier, M15 « contre le squash ») ; textes i18n des
   quatre locales (AC8) et recompte de `frontend/src/lib/shared/i18n-keys.test.ts` (`ATTENDU.sitesTotal` — les deux
   clés neuves apparaissent dans **quatre** sites `i18nMsg`, création et édition) ; `lint-i18n-ownership` ;
   comportement de `ALGORITHM=INSTANT` sans `AFTER` (précédent `20260531000001`) ; `post_restore.rs` /
   `every_data_backfill_migration_is_triaged` (la migration n'écrit rien : vérifier que le détecteur ne la prend pas
   pour un backfill) ; l'E2E (isolement par archivage, comptes non principaux — compatible avec
   `bank-accounts-crud.spec.ts` et ses tests d'archivage du principal ?).
5. **Le manuel** (`docs/manual/fr/user-manual.tex`, section neuve prescrite par AC12) : ce que la fiche fait
   affirmer au manuel est-il vrai **du code tel qu'il sera** (listes et sélecteurs nommés, « ni sur la QR-facture
   ni dans le fichier de paiement », survie à l'onboarding, effacement par `PUT`) ? Le PDF actuel ne contient pas
   la section : contrôle le `.tex` et ce que la fiche prescrit ; PDF aplati (`pdftotext … - | tr '\n' ' ' | tr -s ' '`)
   pour vérifier qu'aucun passage **existant** ne contredit la description (onboarding l. 202, tableau de bord).
6. **Frontière 26-2a / 26-2b** : chaque partie livrable et testable seule ; M12-M15 en a, M16 et M8-bis en b ;
   `bank_account_display_name` sans appelant en a ; aucun test de b qui supposerait un fichier de a non encore écrit,
   ni l'inverse.

## Lentilles

- **R — Auditeur d'acceptation** (Opus) : `.claude/skills/bmad-create-story/checklist.md` ; chaque phrase de l'issue
  et de son commentaire couverte par un AC testable, chaque AC par une tâche, chaque tâche par un test ; **chaque
  ligne du bilan P1 confrontée au texte des AC** ; recomptes de la fiche (14 AC, 17 mutations, 10 sites, 19 constats
  distincts du bilan) refaits depuis la source.
- **F — Full-scope adversary** (Opus) : la conception après remédiation — garde AC11, macro de colonnes et
  `find_by_id_in_tx`, ensemble Unicode refusé, contrat du `PUT`, procédure de rebase sur la 26-1 ; ce que le manuel
  et le CHANGELOG affirmeraient de faux ; tests existants qui changeront de sens.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie
copiée), correction proposée ; pour chaque finding, dis s'il **vient de la remédiation `f7f3e212`** ou de la fiche
d'origine. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée (attention aux motifs
mono-ligne sur du SQL multi-ligne : la P1 a conclu à tort à l'absence d'un précédent `AFTER … INSTANT`). ⛔ **La liste
des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-2-validate-p2-<R|F>.md` ; dernier message : le chemin, le bilan, une ligne
par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh` et les scripts Synology), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`,
`bash` sur un script du dépôt, `sha384sum` vers un fichier, `gh` en écriture, SQL. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
