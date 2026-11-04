# Prompt — validation P5 de la fiche v0172-session (release 0.17.3 : #876, la session ; CR #891), lentille complète, Sonnet

Tu valides la **fiche** de la Story v0172-session de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), visée pour la
release **0.17.3**, **sans migration**. La fiche prescrit : l'inactivité du jeton de renouvellement à **60 minutes par défaut** (CR #891,
FR13) et **au moins le double** du jeton d'accès (fonction unique, appelée par `from_env` seul ; une valeur plus courte est portée au
double, avec un avertissement) ; une **grâce de rotation** de 30 s mesurée à l'horloge de la base ; des **lectures de fond** qui ne
renouvellent pas ; la **révocation de tous les jetons de renouvellement** à la restauration complète, dans sa transaction ; la mise à jour
de FR13 dans le PRD et les epics. Le chemin existant (401, renouvellement, rejeu) fait le reste. La validation P4 (Opus, 0/0/7/13) a été
remédiée, et deux mécanismes ont été **retirés** par souci de simplicité (le repli `GET /auth/me`, le renouvellement silencieux
d'`hydrate()`) ; la course entre deux onglets est déclarée **résiduelle**.

Worktree **`/home/gcorbaz/devel/kesh-0172s`**, branche `story/v0172-session`, partie de `1be0f701` (le code applicatif est celui de
`d7ec2ea2`). **Ne lis rien sous `/home/gcorbaz/devel/kesh-0172b`** (un autre agent y développe). Langue du rapport : **français**.
Modèle : **Sonnet**. **Passe complète**, une seule lentille, R (la règle et sa preuve) et F (les textes et ce qu'ils affirment).
**Contexte frais** : refais les relevés ; ne lis le rapport P4 (`/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-p4.md`) qu'**après**
les tiens.

## À lire (chemins sous `/home/gcorbaz/devel/kesh-0172s`)

1. La fiche `_bmad-output/implementation-artifacts/v0172-session.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v0172-choix-autonomes.md`, section `v0172s` (en vigueur : C-v0172s-14, 15, 16, 17,
   20, 21, 22 à 25 ; C-18 et 19 retirés ; C-1 à 13 écartés).
3. Les issues, en lecture seule : `gh api repos/guycorbaz/kesh/issues/876`, `…/891`, et leurs `/comments`.
4. Le code : `crates/kesh-api/src/config.rs` (lecture des durées, `from_fields_for_test`, `make_test_config`, tests `:2680-2730`),
   `routes/auth.rs` (en entier), `routes/admin.rs` (`full_import`, `run_backup_and_restore`, `:150-480`), `routes/setup.rs`,
   `middleware/auth.rs`, `main.rs`, `crates/kesh-db/src/repositories/refresh_tokens.rs`, `crates/kesh-db/src/backup.rs`
   (`restore_tables_in_tx`), `crates/kesh-db/src/post_restore.rs`, `crates/kesh-api/tests/{auth_e2e,admin_full_import_e2e,admin_backup_e2e}.rs` ;
   `frontend/src/lib/shared/utils/api-client.ts` et son test, `frontend/src/lib/app/stores/auth.svelte.ts`,
   `frontend/src/lib/features/admin-backup/` (`BackupScheduleBanner.svelte` et son test, `backup-schedule.svelte.ts`,
   `backup-schedule.api.ts` et son test, `AutoBackupSection.svelte`) ; `.env.example`, `docker-compose.yml`, `docker-compose.prod.yml` ;
   `_bmad-output/planning-artifacts/prd.md` (`:270-290`, `:385-395`), `epics.md` (`:55-70`, `:605-625`).
5. Le **manuel** : `docs/manual/fr/user-manual.tex` et `docs/manual/fr/admin-manual.tex` — l'inventaire du § *Manuel* (82 et 16 lignes,
   relevés insensibles à la casse). **Le PDF aussi**, lu aplati vers la sortie standard seulement :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`).
6. Pour contexte : `CLAUDE.md` du worktree ; `docs/agents-orchestration.md` ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## Axes à exercer

**La remédiation de la P4 — axe obligatoire, à traiter en premier**

- Pour chacun des 20 findings de la P4, le traitement écrit au Change Log (« Validation P4 (Opus) ») est-il **réellement** dans la fiche et
  le registre, et **juste** ? Les findings déclarés « sans objet » le sont-ils (P13 à P15, liés à `hydrate()`) ? Celui renvoyé à
  l'orchestrateur (P19, fiche mère) l'est-il à bon droit ?
- **Les retraits** : sans le repli `/me` et sans le renouvellement d'`hydrate()`, le défaut de #876 est-il toujours corrigé pour un
  utilisateur qui travaille (saisie longue, consultation de documents, enregistrement) ? Les conséquences déclarées (course résiduelle
  entre onglets ; nouvel onglet ou rechargement après l'échéance du jeton d'accès ⇒ reconnexion) sont-elles exactes, complètes, et
  écrites au manuel ? La fiche garde-t-elle, hors § 8 et Change Log, une trace des mécanismes retirés (AC, mutation, tâche, manuel) ?
- **La révocation à la restauration complète** (§ 9, C-23, AC-4, M5, M5 bis) : le point d'insertion dans la transaction est-il juste
  (après la garde de comptage, après le rejeu des backfills, avant l'audit et le commit) ? Touche-t-elle la restauration depuis l'assistant
  (`/onboarding/restore`) — est-ce le même chemin ? La garde de comptage, l'audit, le rejeu `post_restore` et les tests existants
  (`sessionInvalidated`, `admin_full_import_e2e.rs:326`, `admin_backup_e2e.rs:814`) restent-ils justes ? La sauvegarde de sécurité prise
  avant l'import est-elle affectée ?
- **La règle du double** dans `from_env` seul : les tests existants nommés à AC-1 sont-ils tous ceux qui changent ? AC-2 déclaré garde
  (vert avant) : un test rouge avant correctif établit-il bien le défaut (AC-1) ?
- **La grâce à l'horloge de la base** : la requête prescrite (`revoked_at > NOW(3) - INTERVAL ? SECOND`) est-elle juste avec la session
  `time_zone = '+00:00'` ? Les tests AC-3 (a), (b), (c) — et la modification de `refresh_replay_after_rotation_revokes_all` — sont-ils
  décidables ?
- **Les lectures de fond** : le chemin d'appel prescrit (`BackupScheduleBanner.svelte:34` → `refreshBackupScheduleState(call)` →
  `fetchBackupSchedule(call)` → `apiClient.get(url, call)`) est-il complet ; `background` est-il conservé au rejeu ; un autre appelant
  (`AutoBackupSection`) change-t-il ?
- **Le PRD** (§ *PRD*) : les lignes citées disent-elles ce que la fiche leur fait dire ; une autre mention de la durée de session manque-t-elle
  (cherche `15 min`, `inactivit`, `FR13` dans `_bmad-output/planning-artifacts/`) ?

**La règle et sa preuve**

- Chaque `fichier:ligne` de la fiche dit-il ce qu'elle lui fait dire ?
- AC-1 à AC-7 décidables ; mutations M1 à M6 bis tuées par le test qu'elles nomment ; une régression probable sans mutation ?
- Le décompte des modules (cinq) et la liste des fichiers : un fichier que les AC ou les Tâches font toucher manque-t-il ?
- Règles du dépôt : aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 88) ; aucun mot-clé de fermeture prescrit ; gate complet
  en cours de boucle (repository touché) et E2E complet au dernier commit de code ; aucune donnée réelle ; aucune mention restante d'une
  release 0.17.2 hors Change Log.

**Les textes et ce qu'ils affirment**

- **Le manuel** — axe obligatoire : chaque phrase prescrite est-elle vraie au code après correctif ? Rejoue les deux relevés insensibles à
  la casse et les décomptes (82 = 17 au tableau + 65 hors classe ; 16 = 3 + 13) ; chaque classement tient-il ? Le PDF dit-il la même chose
  que le `.tex` ?
- **La configuration livrée** : les commentaires prescrits de `.env.example` sont-ils vrais ?
- **CHANGELOG** (section 0.17.3) : les deux puces sont-elles vraies ?

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-p5.md`, **écrit par l'outil d'écriture de fichier (`Write`),
une seule fois**. Il contient :

1. **Les findings**, `P1`, `P2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), origine (**remédiation P4** ou **défaut
   d'origine**), lieu (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède. Une affirmation de
   présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne
   compte pas.
3. Un décompte final par sévérité et par origine, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture hors du rapport**, et le rapport par l'outil `Write` seulement : **aucune redirection** (`>`, `>>`, `2>`, `&>`,
  `| tee`), **ni vers un fichier, ni vers `/dev/null`** ; **aucune option qui écrit un fichier** (`-o`, `--output`, `pdftotext` vers un
  fichier) ; **aucun fichier temporaire** — ni dans `/tmp`, ni dans le scratchpad, ni ailleurs. `pdftotext` s'emploie avec `-` (sortie
  standard) seulement. Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni `node`, ni
  `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, ni le NAS, ni `/home/gcorbaz/devel/kesh-0172b` ; aucune donnée
  réelle dans le rapport.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture, sans
  redirection), `git log`, `git show`, `git diff` (en lecture), `pdftotext … -`, `gh api` en `GET`.
