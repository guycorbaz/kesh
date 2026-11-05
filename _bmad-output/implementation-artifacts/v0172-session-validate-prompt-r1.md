# Prompt — validation R1 de la fiche v0172-session reprise (release 0.17.3 : #876, CR #891), lentille complète, Sonnet

Tu valides la **fiche reprise** de la Story v0172-session de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), visée
pour la release **0.17.3**. Le Project Lead a décidé le 2026-11-04 (commentaire de #891) que la durée d'inactivité de la session se règle
**dans l'interface d'administration**, s'applique **exactement** (aucun multiple de la durée du jeton d'accès) et se **remet à zéro à
chaque activité** de l'utilisateur (souris, clavier…). La fiche prescrit : un réglage dans une table **système**
`_kesh_installation_settings` (une migration, DDL seul, défaut 60, bornes 1 à 1440), un écran (Administration → Paramètres, section
Session, Administrateur seul) et sa route auditée ; `refresh_tokens.expires_at` comme échéance d'inactivité, copiée dans une revendication
`idle_exp` du jeton d'accès contrôlée **sans tolérance** par le middleware ; un renouvellement qui **recopie** l'échéance ; un **battement**
`POST /api/v1/auth/activity { idleMs }` (au plus toutes les 15 s, âge exact de la dernière activité, aucune rotation, `exp` inchangé) ;
un module navigateur (`session-idle.svelte.ts`) partagé entre onglets par `localStorage`, une minuterie d'une seconde, **aucun verrou** ;
`KESH_REFRESH_INACTIVITY_MINUTES` ignorée avec avertissement. Elle **garde** de la conception précédente (validée de P3 à P6) : la
révocation de tous les jetons à la restauration complète (motif `password_change`), la grâce de rotation de 30 s, la course résiduelle
entre onglets, le renouvellement au démarrage minimal (`tryRefresh()`), les inventaires des manuels, la mise à jour de FR13.

C'est la **première passe** d'une boucle neuve : la conception a changé, les passes P3 à P6 portaient sur une autre. Worktree
**`/home/gcorbaz/devel/kesh-0172s`**, branche `story/v0172-session` ; numéros de ligne de `c547dc41` (code applicatif de `d7ec2ea2`).
**Ne lis rien sous `/home/gcorbaz/devel/kesh-0172a` ni `/home/gcorbaz/devel/kesh-0172b`** (des agents y développent). Langue du rapport :
**français**. Modèle : **Sonnet**. **Passe complète**, une seule lentille, R (la règle et sa preuve) et F (les textes et ce qu'ils
affirment). **Contexte frais** : fais tes propres relevés ; les rapports des passes précédentes (`/home/gcorbaz/devel/kesh-gate-logs/
v0172s-validate-p*.md`) ne se lisent, si tu le veux, qu'**après** les tiens.

## À lire (chemins sous `/home/gcorbaz/devel/kesh-0172s`)

1. La fiche `_bmad-output/implementation-artifacts/v0172-session.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v0172-choix-autonomes.md`, section `v0172s` (en vigueur : C-v0172s-14, 17, 23, 24,
   26 et 28 à 42 ; les autres sont remplacés, retirés ou écartés).
3. Les issues, en lecture seule : `gh api repos/guycorbaz/kesh/issues/876`, `…/891`, et leurs `/comments`.
4. Le code serveur : `crates/kesh-api/src/config.rs` (`:200-215`, `:440-515`, `:730-760`, `:1180-1190`, `:1610-1640`, `:2680-2730`),
   `main.rs` (`:225-255`), `auth/jwt.rs`, `middleware/auth.rs`, `routes/auth.rs` (en entier), `routes/setup.rs`, `routes/admin.rs`
   (`:150-560`), `routes/backup_schedule.rs`, `lib.rs` (`:250-360`, `:480-510`, `:1100-1130`, `:1315-1330`), `audit_labels.rs`,
   `auth/api_key.rs`, `helpers.rs` ; `crates/kesh-db/src/repositories/{refresh_tokens,backup_schedule,mcp_server_settings}.rs`,
   `crates/kesh-db/src/backup.rs` (`:25-60`, `:840-870`), `crates/kesh-db/src/post_restore.rs` (registres et exemptions),
   `crates/kesh-db/migrations/{20261024000001_backup_schedule,20261025000001_mcp_keys_and_server_settings,20260406000001_refresh_tokens_revoked_reason}.sql`,
   `crates/kesh-db/test-schema/0001_schema_squash.sql` (`:1-60`, `:1170-1190`), `crates/kesh-db/tests/test_schema_guard.rs`,
   `crates/kesh-db/migrations.sha384`, `docs/migrations-idempotence-audit.md` ; les tests `crates/kesh-api/tests/{auth_e2e,
   admin_full_import_e2e,admin_backup_e2e,audit_route_registry,admin_pat_denied_e2e}.rs`.
5. Le code navigateur : `frontend/src/lib/shared/utils/api-client.ts` et son test, `frontend/src/lib/app/stores/auth.svelte.ts` et son
   test, `frontend/src/hooks.client.ts`, `frontend/src/routes/+layout.svelte`, `frontend/src/routes/(app)/+layout.svelte`,
   `frontend/src/routes/(app)/settings/+page.svelte` et son test, `frontend/src/lib/features/settings/`, `frontend/src/routes/login/+page.svelte`,
   `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte`, `frontend/src/lib/features/admin-backup/BackupScheduleBanner.svelte`.
6. La configuration livrée : `.env.example`, `docker-compose.yml`, `docker-compose.prod.yml`, `crates/kesh-api/README.md` ; le PRD :
   `_bmad-output/planning-artifacts/prd.md` (`:270-290`, `:385-395`), `epics.md` (`:55-70`, `:605-625`), `docs/kesh-specifications.txt`.
7. Le **manuel** : `docs/manual/fr/user-manual.tex` et `docs/manual/fr/admin-manual.tex` — rejoue les relevés du § *Manuel* de la fiche
   (82, 16, et le relevé `inactivit`). **Le PDF aussi**, lu aplati vers la sortie standard seulement :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`).
8. Pour contexte : `CLAUDE.md` du worktree (*Migration breaking policy* P1–P8, *Règle de splitting préventif*, *Review Iteration Rule*) ;
   `docs/agents-orchestration.md` ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## Axes à exercer

**Les quatre exigences du Project Lead — axe obligatoire, à traiter en premier**

- **Exactitude** : la fiche prétend que le serveur refuse **toute** requête à partir de `dernière activité + N`, à la seconde près, et
  que le navigateur ferme au même instant. Cherche un chemin qui accepte une requête au-delà : un jeton sans `idle_exp` (lesquels en
  émettent encore après correctif ?), la tolérance de 60 s de `decode`, une route protégée qui ne passe pas par le middleware, le
  renouvellement, le battement, `/me`, l'assistant, le changement de mot de passe, la réinitialisation par courriel, le recours d'urgence
  (`auth/bootstrap.rs`), une clé d'API ou le serveur MCP. Cherche aussi un chemin qui **raccourcit** l'échéance sans que la fiche le dise
  (battement perdu, horloges, arrondis, course entre battement et renouvellement). Les deux écarts déclarés sont-ils les seuls ?
- **Remise à zéro** : la liste d'événements couvre-t-elle « clavier, souris (mouvement, clic, molette), toucher, défilement, retour sur
  l'onglet » ? Le filtre des mouvements fictifs (`screenX`/`screenY`) est-il juste, et ne rate-t-il pas un vrai mouvement ? Le contrôle de
  l'échéance avant l'enregistrement est-il prescrit partout (événement, retour sur l'onglet, démarrage) ? Le partage entre onglets par
  `localStorage` tient-il sans verrou (deux onglets, un onglet qui s'ouvre, un onglet qui se ferme dans les 15 s, un onglet d'un autre
  utilisateur après déconnexion dans le même navigateur) ? Le constat sur l'aperçu PDF est-il exact (visionneuse d'un `<iframe>` `blob:`,
  `blur` de `window`) ?
- **Le réglage** : table système, sans ligne initiale, défaut 60, bornes, audit dans la transaction, verrou optimiste (version 0 ⇒
  `INSERT`) ; une restauration complète le garde-t-elle vraiment (inventaires, `TABLES_TO_TRUNCATE`, réinitialisation de démonstration) ?
  Le sort de la variable (ignorée, avertissement, pas d'amorçage, paramètre de test gardé et ignoré) est-il cohérent, et le passage d'une
  installation qui la pose est-il écrit ?
- **La fin de session** : le message, ce qui est perdu, l'onglet en arrière-plan, la veille ; `hydrate()` après l'échéance (sans message)
  — est-ce bien ce que le code ferait ?

**La migration**

- P1 à P8 du `CLAUDE.md` appliqués à la migration prescrite : non breaking sans bump est-il juste ? P5 (ligne et compteurs), P6 (sites
  positionnels — cherche-les toi-même : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`), P7 (le détecteur la classera-t-il
  sans écriture ?), P8 (registre des checksums), le squash de test et sa garde, l'exclusion de `backup.rs` et son test. Le retour à la
  0.17.1 est-il écrit partout où il doit l'être (CHANGELOG, manuel, note de mise à jour) ?

**La simplicité et le découpage**

- La conception est-elle la plus simple qui tienne les quatre exigences ? Un mécanisme prescrit est-il superflu ; un mécanisme écarté
  (lecture en base à chaque requête, `BroadcastChannel`, `sendBeacon`, alerte préalable) était-il nécessaire ?
- Le décompte des modules (huit) et la dérogation C-v0172s-41 sont-ils conformes à la *Règle de splitting préventif* ; l'argument « le
  serveur seul fermerait la session malgré l'activité » est-il vrai ? Un fichier que les AC ou les Tâches font toucher manque-t-il au
  § *Modules* ?

**La règle et sa preuve**

- Chaque `fichier:ligne` de la fiche dit-il ce qu'elle lui fait dire ?
- AC-1 à AC-11 décidables ; tests rouges avant correctif déclarés justes ; mutations M1 à M14 tuées par le test qu'elles nomment ; une
  régression probable sans mutation ? Les tests existants qui changent sont-ils tous trouvés (cherche-les : `expires_at`, `minutes(15)`,
  `refresh_inactivity`, `CurrentUser {`, `LoginResponse`, `MeResponse`, compte d'appels de `fetch` dans `auth.svelte.test.ts`) ?
- Règles du dépôt : aucun mot-clé de fermeture prescrit ; gate complet en cours de boucle (repository et migration) ; E2E complet au
  dernier commit de code ; aucune donnée réelle.

**Ce qui est gardé**

- La révocation à l'import (point d'insertion, motif, annulation sur échec), la grâce, la course, `tryRefresh()` : encore justes dans la
  conception neuve ? Le renouvellement qui **recopie** l'échéance change-t-il quelque chose à la grâce ou à la détection de vol ?

**Les textes et ce qu'ils affirment**

- **Le manuel** — axe obligatoire : chaque phrase prescrite est-elle vraie au code après correctif ? Rejoue les relevés et les
  décomptes (82 = 17 + 65 ; 16 = 3 + 13 ; `inactivit` : trois lignes) ; chaque classement tient-il ? Le PDF dit-il la même chose que le
  `.tex` ? La sous-section neuve dit-elle la précision réelle ?
- **La configuration livrée**, **le CHANGELOG** (section 0.17.3 : quatre puces et l'avertissement de migration), **FR13** et ses copies :
  vrais, et conformes au commentaire de #891 ?

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172s-validate-r1.md`, **écrit par l'outil d'écriture de fichier
(`Write`), une seule fois**. Il contient :

1. **Les findings**, `R1-1`, `R1-2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), lieu (`fichier:ligne`), **la ligne lue
   citée** (sortie `grep -nF` ou extrait copié) et le remède. Une affirmation de présence ou d'absence sans sortie `grep -nF` copiée ne
   compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne
   compte pas.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture hors du rapport**, et le rapport par l'outil `Write` seulement : **aucune redirection** (`>`, `>>`, `2>`, `&>`,
  `| tee`), **ni vers un fichier, ni vers `/dev/null`** ; **aucune option qui écrit un fichier** (`-o`, `--output`, `pdftotext` vers un
  fichier) ; **aucun fichier temporaire** — ni dans `/tmp`, ni dans le scratchpad, ni ailleurs. `pdftotext` s'emploie avec `-` (sortie
  standard) seulement. Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `node`, ni `make`, ni `latexmk`, ni `python3`, ni
  `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, ni le NAS, ni `/home/gcorbaz/devel/kesh-0172a`,
  `/home/gcorbaz/devel/kesh-0172b` ; aucune donnée réelle dans le rapport.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture, sans
  redirection), `git log`, `git show`, `git diff` (en lecture), `pdftotext … -`, `gh api` en `GET`.
