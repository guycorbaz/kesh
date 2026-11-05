# Prompt — validation P6 de la fiche v0172-session (release 0.17.3 : #876, la session ; CR #891), lentille complète, Opus

Tu valides la **fiche** de la Story v0172-session de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), visée pour la
release **0.17.3**, **sans migration**. La fiche prescrit : l'inactivité du jeton de renouvellement à **60 minutes par défaut** (CR #891,
FR13) et **au moins le double** du jeton d'accès (fonction unique, appelée par `from_env` seul ; une valeur plus courte est portée au
double, avec un avertissement) ; une **grâce de rotation** de 30 s mesurée à l'horloge de la base ; des **lectures de fond** qui ne
renouvellent pas ; la **révocation de tous les jetons de renouvellement** à la restauration complète, dans sa transaction ; la mise à jour
de FR13 dans le PRD et les epics ; un **renouvellement au démarrage** sous forme minimale (`tryRefresh()` extrait de `doRefresh`, sous le
mutex d'onglet, sans diffusion ni redirection, appelé une fois par `hydrate()`). Le chemin existant (401, renouvellement, rejeu) fait le
reste ; la course entre deux onglets est déclarée **résiduelle**. La validation P5 (Sonnet, 0/1/4/9) a été remédiée : motif de révocation
`"password_change"` (contrainte CHECK, sans migration), échec après révocation testé par une archive sans Admin, renouvellement au
démarrage rétabli, dérogation de splitting (six modules).

⚠️ **Sixième passe** : la boucle approche du plafond de huit (`CLAUDE.md`, *Review Iteration Rule*). Dès cette passe, l'orchestrateur ne
remédie plus lui-même ; une remédiation écrite après la dernière passe permise n'est relue par personne. Signale donc avec soin ce qui
serait mieux **retiré** que corrigé.

Worktree **`/home/gcorbaz/devel/kesh-0172s`**, branche `story/v0172-session`, partie de `1be0f701` (le code applicatif est celui de
`d7ec2ea2`). **Ne lis rien sous `/home/gcorbaz/devel/kesh-0172b`** (un autre agent y développe). Langue du rapport : **français**.
Modèle : **Opus**. **Passe complète**, une seule lentille, R (la règle et sa preuve) et F (les textes et ce qu'ils affirment).
**Contexte frais** : refais les relevés ; ne lis le rapport P5 (`/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-p5.md`) qu'**après**
les tiens.

## À lire (chemins sous `/home/gcorbaz/devel/kesh-0172s`)

1. La fiche `_bmad-output/implementation-artifacts/v0172-session.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v0172-choix-autonomes.md`, section `v0172s` (en vigueur : C-v0172s-14 à 17, 20 à
   24, 26, 27 ; C-18 retiré ; C-19 et 25 remplacés ; C-1 à 13 écartés).
3. Les issues, en lecture seule : `gh api repos/guycorbaz/kesh/issues/876`, `…/891`, et leurs `/comments`.
4. Le code : `crates/kesh-api/src/config.rs` (lecture des durées, `from_fields_for_test`, `make_test_config`, tests `:2680-2730`),
   `routes/auth.rs` (en entier), `routes/admin.rs` (`full_import`, `run_backup_and_restore`, `:150-480`), `routes/setup.rs`,
   `middleware/auth.rs`, `main.rs`, `auth/bootstrap.rs` (`:340-370`), `routes/users.rs` (`:360-500`),
   `crates/kesh-db/src/repositories/refresh_tokens.rs`, `crates/kesh-db/migrations/20260406000001_refresh_tokens_revoked_reason.sql`,
   `crates/kesh-db/test-schema/0001_schema_squash.sql` (`:1170-1190`), `crates/kesh-db/src/backfill.rs`, `crates/kesh-api/README.md`, `crates/kesh-db/src/backup.rs`
   (`restore_tables_in_tx`), `crates/kesh-db/src/post_restore.rs`, `crates/kesh-api/tests/{auth_e2e,admin_full_import_e2e,admin_backup_e2e}.rs` ;
   `frontend/src/lib/shared/utils/api-client.ts` et son test, `frontend/src/lib/app/stores/auth.svelte.ts` et son test,
   `frontend/src/hooks.client.ts`, `frontend/src/lib/features/vat-rates/` (imports du store),
   `frontend/src/lib/features/admin-backup/` (`BackupScheduleBanner.svelte` et son test, `backup-schedule.svelte.ts`,
   `backup-schedule.api.ts` et son test, `AutoBackupSection.svelte`) ; `.env.example`, `docker-compose.yml`, `docker-compose.prod.yml` ;
   `_bmad-output/planning-artifacts/prd.md` (`:270-290`, `:385-395`), `epics.md` (`:55-70`, `:605-625`), `docs/kesh-specifications.txt`.
5. Le **manuel** : `docs/manual/fr/user-manual.tex` et `docs/manual/fr/admin-manual.tex` — l'inventaire du § *Manuel* (82 et 16 lignes,
   relevés insensibles à la casse). **Le PDF aussi**, lu aplati vers la sortie standard seulement :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`).
6. Pour contexte : `CLAUDE.md` du worktree ; `docs/agents-orchestration.md` ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## Axes à exercer

**La remédiation de la P5 — axe obligatoire, à traiter en premier**

- Pour chacun des 14 findings de la P5, le traitement écrit au Change Log (« Validation P5 (Sonnet) ») est-il **réellement** dans la fiche
  et le registre, et **juste** ? Un remède a-t-il introduit un défaut neuf ?
- **Le motif `"password_change"`** : passe-t-il la contrainte sur **toutes** les bases (migration et squash de test) ? Un lecteur du motif
  (code, requêtes, écrans, exports, audit) est-il égaré par des lignes révoquées « pour changement de mot de passe » après un import ?
- **Le point d'insertion de la révocation** (juste avant `SELECT MIN(id)`, `routes/admin.rs:488-492`) : relis `admin.rs:400-560` ; est-ce
  bien après 5-bis, 5-ter et la garde, avant le refus « aucun compte Admin », la réparation 5-quater, l'audit et le commit ? L'archive
  sans Admin d'AC-4 (b) est-elle constructible avec les helpers de `admin_full_import_e2e.rs`, et franchit-elle bien les étapes
  précédentes (parse, compat, sauvegarde de sécurité, garde de comptage) pour échouer **après** la révocation ? M5 bis (révocation sur le
  pool avant la transaction) s'exécute-t-elle sans attente de verrou, et rougit-elle bien T4 (a) et (b) ?
- **Le renouvellement au démarrage** (§ 8 bis, C-26, AC-5 bis, M7 à M7 quater) : l'extraction de `tryRefresh()` hors de `doRefresh`
  garde-t-elle **exactement** le comportement des six tests du chemin 401 (`api-client.test.ts:205, :246, :270, :327, :488, :567`) — effacement
  et redirection au même moment, dans les mêmes branches (réseau, réponse illisible, `expiresIn` absent, statut non OK) ? L'import dynamique
  depuis `auth.svelte.ts` est-il nécessaire et suffisant (cycle d'import réel ?) ; le mutex est-il vraiment partagé avec le chemin 401 ?
  La justification « P13 à P15 ne reviennent pas » est-elle vraie, y compris pour le canal `auth-change` reçu par un autre onglet après
  un `login`/`logout` ordinaire, et pour `/login`, `/setup`, `/onboarding` ? Les tests existants d'`auth.svelte.test.ts` qui changent
  sont-ils trouvés (cherche-les) ?
- **La dérogation de splitting** (C-27) : est-elle conforme à la règle (justification et risque accepté écrits) ?
- **Les textes ajoutés** : `crates/kesh-api/README.md` (`:41`, `:161`, `:179`), `docs/kesh-specifications.txt` (sites et tri),
  `admin-manual.tex:2292/:2322` (cohérence avec `:2360`), la course au manuel (`:95`, `:2997`), la puce *Modifié* du CHANGELOG.

**La règle et sa preuve**

- Chaque `fichier:ligne` de la fiche dit-il ce qu'elle lui fait dire ?
- AC-1 à AC-7 (et AC-5 bis) décidables ; mutations M1 à M7 quater tuées par le test qu'elles nomment ; une régression probable sans mutation ?
- Le décompte des modules (six, sous dérogation) et la liste des fichiers : un fichier que les AC ou les Tâches font toucher manque-t-il ?
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

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-p6.md`, **écrit par l'outil d'écriture de fichier (`Write`),
une seule fois**. Il contient :

1. **Les findings**, `P1`, `P2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), origine (**remédiation P5** ou **défaut
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
