# Prompt — revue de code P1 de la Story 31-1c (la sauvegarde automatique faite par Kesh), trois lentilles, Sonnet

Tu relis le développement de la **Story 31-1c** de Kesh (comptabilité suisse) : une table système
`_kesh_backup_schedule` (migration **non breaking** `20261024000001_backup_schedule.sql`, réinjectée au squash de test) ;
une tâche de fond lancée par `main.rs` qui, à intervalle réglable, écrit une sauvegarde `.keshbackup` complète dans
`KESH_ADMIN_BACKUP_DIR/auto/`, la relit, applique une rétention qui protège la dernière sauvegarde complète, refuse
d'écrire si l'espace manque et classe `disk_full` ; un verrou en mémoire (`AppState::backup_lock`) partagé avec
l'export et l'import ; cinq routes d'administration (réglage, état, « Sauvegarder maintenant », liste et téléchargement
des sauvegardes du serveur) ; la section de l'écran, le bandeau de l'application, i18n ×4, E2E, manuel d'administration.
Issue #503 (la PR de la 31-1c la ferme). Worktree `/home/gcorbaz/devel/kesh-31-1c`, branche `story/31-1c`, partie de
`origin/main` = `e71cd351` (31-1a et 31-1b mergées). Le diff à relire, **aplati** (un seul diff, pas la suite des
commits) : `git -C /home/gcorbaz/devel/kesh-31-1c diff e71cd351...HEAD`. Langue du rapport : français. Modèle :
**Sonnet**.

Lis d'abord : la fiche `_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md` — **partie c seulement**
(AC23 à AC40 ; tâches T-c1 à T-c7 ; mutations M-c1 à M-c24 ; Dev Agent Record de la 31-1c ; dans le Change Log, les
entrées des validations P5 à P8 et l'étape 0 du développement de la 31-1a, qui listent les LOW confiés à la 31-1c :
R5, R6, R8, R9, F2, F4, L1 à L4) — et le registre `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md`
(C-31-1-81 et suivants pour le développement ; C-31-1-15 à 20, 27, 30, 32, 34, 40, 42, 44, 45, 50 à 52, 54 à 58 pour la
conception). Le Dev Agent Record déclare des gates, des chiffres et des mutations : **ne les prends pas pour argent
comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/311c-*.log` (lecture seule). Les captures
de l'AC40 sont dans `/home/gcorbaz/devel/kesh-gate-logs/311c-captures/` (lis-les comme images). Les parties a et b de la
fiche sont **hors périmètre** : un défaut qui ne vit qu'en elles n'est pas un finding.

Rapports, un par lentille : `/home/gcorbaz/devel/kesh-gate-logs/311c-review-p1-B.md`, `…-E.md`, `…-A.md`. Chaque
rapport porte : un bilan CRITICAL / HIGH / MEDIUM / LOW ; chaque finding avec sa preuve (sortie `grep -nF` ou `sed -n`
copiée) ; la liste des **axes exercés** et celle des **axes non exercés** — un « 0 finding » sans cette liste ne compte
pas comme passe.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. Aucun fichier temporaire : compare
  par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni `scripts/regen-test-schema.sh`, ni `make`, ni `latexmk`, ni `cargo` (aucune
  sous-commande), ni `npm`, ni `npx`, ni `vitest`, ni `playwright`, ni `node`, ni `docker`, ni `mariadb`, ni `sqlx`, ni
  `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ; `gh issue view 503` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `sha384sum`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-api/src/auto_backup.rs` : `is_due` / `next_due_at` / `is_overdue` (bornes, horloge en arrière,
   intervalle 6 h contre délai de reprise d'une heure), `classify_write_error`, `fichiers_reguliers` (liens, fichiers
   disparus, dossier absent), `lister_les_sauvegardes`, `supprimer_les_partiels` (âge de 10 minutes, date illisible),
   `taille_des_documents`, `seuil_d_espace` (débordements), `appliquer_la_retention` (tri par nom, protégés, surplus),
   `relire`, `ecrire`, `run_once` (ordre : verrou, réglage, tentative, écriture, statut, rétention ; ce qui est en `Err`
   et ce qui est un statut ; le garde tenu jusqu'à la fin), `spawn` et `un_tour`. Une erreur peut-elle laisser un
   `.partial`, supprimer une sauvegarde saine, ou écrire un statut faux ?
2. `crates/kesh-api/src/routes/backup_schedule.rs` : validation du `PUT`, 409 de version, `run_backup_now` (tâche
   attendue, `Skipped` → 409, audit), `list_backups`, `download_backup` (traversée, lien symbolique, nom hors forme,
   type inconnu ; le fichier ouvert est-il bien celui listé ? fenêtre entre la liste et l'ouverture), `emit_audit`.
3. `crates/kesh-api/src/routes/admin.rs` : le garde de `full_export` (déplacé dans `remplir`, rendu avant la diffusion ?
   que se passe-t-il si `export_installation` échoue ?), celui de `full_import` (pris après la vérification, avant le
   `FOR UPDATE`), le nettoyage des `.partial` pré-import ; `write_backup_file` devenue `pub(crate)`.
4. `crates/kesh-api/src/{lib.rs,main.rs,errors.rs}` et les huit littéraux `AppState { … }` ; `Cargo.toml` (`rustix`).
5. `crates/kesh-db` : la migration (contraintes, `CHECK` sur un `NULL`, défauts), `migrations.sha384`, le squash et
   `scripts/regen-test-schema.sh` (§ 7-bis : relevé, contrôle de forme, réinjection), `repositories/backup_schedule.rs`
   (`update_settings` : no-op, conflit, audit dans la transaction ; `record_*` : jamais `version` ; `record_success` :
   `IF(?, ?, last_complete_file)`), `post_restore.rs` (exemption, 19 → 20), `backup.rs`, `test_fixtures.rs`,
   `tests/test_schema_guard.rs`, `tests/migrations_upgrade_path.rs` (83/49, frontière 34).
6. Frontend : `api-client.ts` (`post` gagne `call`), `features/admin-backup/` (`backup-schedule.api.ts`,
   `backup-schedule.svelte.ts`, `backup-format.ts`, `AutoBackupSection.svelte`, `BackupScheduleBanner.svelte`,
   `AdminBackupPanel.svelte`), `routes/(app)/+layout.svelte`. Un non-Admin peut-il déclencher une lecture 403 en boucle ?
   Le bandeau peut-il casser le layout ? Les dates et le retard sont-ils bien ceux du serveur ?
7. i18n : les quatre `messages.ftl` — 43 clés d'écran, `error-backup-in-progress`, trois libellés d'audit ; replis
   Svelte **identiques** au fr-CH (au caractère près) ; variables `{ $count }`, `{ $date }`, `{ $mib }` présentes dans
   chaque traduction ; `i18n-keys.test.ts` (2289 → 2332) et `i18n-libelle-en-dur.test.ts` (68 → 70, 54 → 56) : les
   ventilations écrites sont-elles justes (recompte aux deux bornes) ?
8. Documentation : `docs/manual/fr/admin-manual.tex` **et son PDF** (`docs/manual/fr/admin-manual.pdf`, aplati) — la
   sous-section *La sauvegarde automatique* (`sec:sauvegarde-automatique`), *La sauvegarde de sécurité pré-import*,
   *Stratégie recommandée*, le tableau *Quelle méthode choisir ?*, *Passer à la 0.15.0* ; `CHANGELOG.md`
   (`[0.15.0] — Non publié`) ; `README.md` ; `.env.example` ; les deux `docker-compose*.yml` ;
   `docs/migrations-idempotence-audit.md`. **Le manuel promet-il ce que le code ne fait pas** (seuil d'espace, reprise
   après une heure, rétention, `.partial` de dix minutes, bandeau, texte cité de l'écran — chaque citation entre
   guillemets se retrouve-t-elle au catalogue fr-CH ?) ?

## Lentille E — Edge cases (chaque branche)

1. `is_due` / `next_due_at` / `is_overdue` : intervalle 6, 12, 24, 168 ; réussite et tentative nulles, égales,
   futures, une milliseconde de part et d'autre des bornes ; désactivée. Le bandeau `overdue` peut-il s'allumer à tort
   juste après une mise à jour, ou rester éteint sur une tâche morte ?
2. `run_once` : verrou pris ; réglage illisible ; `record_attempt` en échec ; dossier `auto` absent, fichier, lien,
   non inscriptible ; espace illisible ; espace juste au seuil ; écriture qui échoue à l'ouverture du `.partial`, pendant
   les tables, pendant les pièces, au `sync_all`, au renommage (nom déjà présent) ; relecture qui échoue et suppression
   qui échoue ; `record_success` en échec **après** l'écriture (fichier écrit, état non enregistré : la rétention
   tourne-t-elle ? la sauvegarde est-elle perdue pour la liste ?) ; rétention avec `retention_count` 1 et 60,
   `last_complete_file` absent du disque, nom d'une sauvegarde datée du futur.
3. Concurrence : export manuel, import, sauvegarde automatique et « Sauvegarder maintenant » croisés deux à deux ; un
   client parti pendant l'export ; l'ordre `backup_lock` puis `FOR UPDATE` partout ; deux processus (angle mort écrit).
4. Routes : `PUT` avec champs manquants, types faux, version négative, réglage inchangé (version inchangée ?) ;
   téléchargement d'un fichier supprimé entre la liste et l'ouverture ; nom à la casse différente ; `kind`
   `pre-import` avec un nom `auto`.
5. Frontend : réponses partielles ou `null` ; `retentionCount` vide ou hors bornes dans le champ ; double clic
   « Enregistrer » / « Sauvegarder maintenant » ; 409 suivi de « Recharger » ; échec de la liste seule ; `importMaxMib`
   0 ; bandeau pour un utilisateur qui change de rôle sans recharger ; minuterie de 30 minutes nettoyée au démontage.
6. Tests qui pourraient passer **à vide** : `admin_auto_backup_e2e.rs` (montage, assertions de montage, `TestEnv`
   qui tronque ou sature — l'erreur arrive-t-elle bien par le chemin prévu ?), les gardes lexicales de
   `auto_backup::tests` (motifs, troncature des commentaires), `squash_seeds_the_backup_schedule_row`, la spec E2E
   `backup-schedule.spec.ts` (états posés par interception : que prouvent-ils et que ne prouvent-ils pas ? le
   manifeste lu par l'analyseur maison ; le réglage remis aux défauts en fin de spec).
7. Mutations M-c1 à M-c25 : pour chacune, le diff du journal `311c-mut-<M>.log` est-il la mutation décrite, le test qui
   la tue est-il celui que le Dev Agent Record nomme, et l'échec porte-t-il sur l'assertion qui compte (et non sur une
   compilation) ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC23 à AC40, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC23** : colonnes, contraintes, `ENGINE`, charset ; P5 (ligne d'audit et trois compteurs **recomptés** :
  `ls crates/kesh-db/migrations/*.sql | wc -l`, `grep -c '^| \`20' docs/migrations-idempotence-audit.md`), P6
  (résultat du `grep` écrit au Dev Agent Record), P7 (exemption `Durable`), P8 (empreinte inscrite : recalcule-la par
  `sha384sum`) ; squash, garde, `backup_inventory_matches_schema`.
- **AC24** à **AC30** : chaque règle de la fiche retrouvée dans le code, et son test.
- **AC26** : la garde lexicale du lancement écarte-t-elle bien les lignes `//` ? `main.rs` l'appelle-t-il après la
  construction de l'état et avant `axum::serve` ?
- **AC27** : les neuf sites, les gardes possédés, la garde lexicale (motifs de la fiche et formes ajoutées).
- **AC28** : écart C-31-1-82 (seuls les `.partial` abandonnés depuis dix minutes) — justifié, tenu au manuel ?
- **AC31** à **AC33** : rôles `alert` / `status`, bandeau Admin et activée, `partial` sans bandeau, `importMaxMib`,
  mention « trop grande » (stricte), rappel permanent, textes ×4.
- **AC32** : `admin_pat_denied_e2e.rs` — 33 couples, `(8, 25)`, `GUARDED_BY_HANDLER`, `assert_eq!(discriminating, 6)` ;
  registres d'audit (124 / 127 routes, 118 tracées, 99 sans écriture au journal) ; `docs/api-external.md` vérifié
  sans ajout (écrit au Dev Agent Record).
- **AC34** : le manuel donne le téléchargement par l'écran comme voie normale, SSH en secours.
- **AC35**, **AC36** : chaque cas de la liste de l'AC36 retrouvé dans `admin_auto_backup_e2e.rs` (ou ailleurs, nommé).
- **AC37** : chaque cas Vitest de la fiche.
- **AC38** : la spec (réglage gardé, « Sauvegarder maintenant », ligne `auto`, téléchargement et `formatVersion: 2`,
  défauts remis, Comptable sans menu ni page). Écart C-31-1-83 (états par interception) : acceptable ?
- **AC39** : chaque point de la liste de l'AC39 au manuel **et au PDF** ; CHANGELOG (migration, retour à la 0.14.0 par
  la sauvegarde d'avant) ; README.
- **AC40** : les captures (a) à (r) (sans (o)) présentes, conformes à leur état, voie d'obtention dite au Dev Agent
  Record ; (q) à 390 px.
- **LOW confiés à la 31-1c** (R5, R6, R8, R9, F2, F4 de la P5 ; L1 à L4 de la P8) : traités ou consignés ?
- **Gates** : les chiffres du Dev Agent Record (nextest, Vitest, `check`, `lint-i18n-ownership`, `build`, E2E et liste
  des échecs jugés contre `docs/testing.md` § *Les échecs attendus*) correspondent-ils **aux journaux** ? Le gate
  complet et l'E2E ont-ils tourné **au dernier commit de code** (compare au `git log --oneline e71cd351..HEAD`) ?
- **Écarts déclarés** au Dev Agent Record : sont-ils exacts et complets ?
