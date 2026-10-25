# Story 32.5 : les clés MCP et les réglages du serveur, en base 🗄️

Status: ready-for-dev

*Fiche rédigée le 2026-10-25, en autonomie (Opus 5.5), worktree `kesh-32-5`, branche `story/32-5` partie de `main`
`e59d63e8` ; remédiée après les validations P1 à P7 le même jour. Registre : `epic-32-choix-autonomes-325.md` (C-32-5-1 à
C-32-5-28). Clé courte : `325`. Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh ») ; issue voisine : #722
(« la sauvegarde emporte tout ce qu'il faut pour redémarrer, hors `.env` ») — toutes deux en `refs` seulement. Les
numéros de ligne cités sont ceux de `e59d63e8`.*

⚠️ **Une fiche, quatre parties** (C-32-5-1, révisé en validation P1) : **32-5a** les tables et la sauvegarde, **32-5b**
l'authentification par clé MCP, **32-5c** les routes de gestion, **32-5d** la clé MCP à la remise à zéro et à la
réparation. Chaque partie a ses AC, ses tâches, ses mutations, ses gates et sa propre boucle de revue. Ordre
**imposé** (validation P2, R10 = F9 ; C-32-5-27) : **a → b → d → c**. Rien dans le code n'oblige d avant c, mais si c
passait d'abord, une clé créée par ses routes serait effacée par la remise à zéro jusqu'à la 32-5d ; d avant c tient
aussi la série des stories `kesh-db` (C-32-0-12). Clés au registre de sprint (la clé
`32-5-cles-et-reglages-mcp` est `split`, `0b0835a1`) : `32-5a-tables-et-sauvegarde-mcp`,
`32-5b-authentification-cle-mcp` (renommée à la validation P1 : elle ne porte plus que l'authentification),
`32-5c-routes-gestion-cles-mcp`, `32-5d-cle-mcp-principal` (née du redécoupage).

## Story

En tant que Guy, qui va brancher un agent sur ses livres réels par le serveur MCP de la 0.16.0, je veux un **second
type de clé**, propre au MCP, à **portées fines** et à **politique de validation par clé**, créé par l'administrateur
seul, et des **réglages du serveur** (désactivé par défaut, filtre d'adresse) **en base et dans la sauvegarde**, afin
qu'aucune clé d'API n'ouvre le MCP, qu'aucune clé MCP n'ouvre l'API, et qu'une restauration rende un serveur qui
fonctionne — clés comprises.

## Origine

- **Kickoff** (`planning-artifacts/epic-32-serveur-mcp.md` § *32-5*) : table propre (C-32-0-6), préfixe distinct,
  condensat SHA-256, affichage unique, révocation, expiration, dernière utilisation, liste de portées ; génération et
  condensat **réutilisés** d'`auth/api_key.rs` ; portées `books:read`, `invoices:read`, `contacts:read`,
  `documents:read`, `prompts:write` (C-32-0-7), **aucune** portée d'administration ni d'écriture des livres ;
  réglages dans une table **de société, au `.keshbackup`** (C-32-0-8) ; routes de gestion réservées à
  l'Administrateur (C-32-0-9), refusées à toute requête authentifiée par clé ; **une clé restaurée authentifie** ; une
  clé MCP présentée à `/api/v1` rend 401, un PAT présenté aux routes MCP aussi.
- **Réponses de Guy du 2026-10-24 au soir** (fichier d'epic § *Réponses* ; spike § 7 *Réponses*) :
  - **Q12** — validation avant envoi **par clé** : `requise` par défaut, `dispensée` pour une clé que l'administrateur
    désigne, chaque envoi journalisé. **La 32-5 porte ce réglage en base** ; son **application** relève des stories
    suivantes : le répartiteur de la **32-6** (qui refuse une clé `requise` tant que la 32-8 n'est pas livrée — spike
    § 6.2, point 6 — et trace chaque appel), la **poignée** de la Q10 (c) et sa table des envois en **32-8**.
  - **Q11** — lecture seule **applicative**, plus le mode de session `READ ONLY` ; ni utilisateur MariaDB de plus, ni
    variable `.env` de plus. **La 32-5 ne suppose rien d'autre** : elle lit par la `Lecture`, écrit la dernière
    utilisation par la `Trace` (32-3), et fonctionne que la `Lecture` soit en lecture seule ou non.
  - **Q4** — non : Kesh n'est pas en service, la 32-12 est retirée ; **la limite s'écrit** au manuel d'administration
    et au CHANGELOG par la story qui change l'inventaire, **celle-ci** (32-5a).
- **Hypothèses de travail** (questions sans réponse, recommandations du kickoff, écrites comme telles) :
  - **Q3** — la **création** d'une clé MCP est **réservée à l'Administrateur** (C-32-0-9, C-32-5-7) ;
  - **Q7** — les réglages MCP sont **restaurés tels quels** (C-32-0-8, C-32-5-11) ;
  - **Q5** — les cinq portées ci-dessus (C-32-0-7).
- **#722** — le CR demande l'**inventaire clos** de ce qui vit hors du `.keshbackup`, réglages de l'Epic 32 compris
  (prompts, activation du serveur, clés MCP), et, pour chaque élément, de l'emporter ou de dire pourquoi il reste à
  part, au manuel § restauration. **La 32-5 est cohérente avec lui et n'y ajoute rien hors sauvegarde** : les clés MCP
  et les réglages du serveur (activation comprise) **entrent** au `.keshbackup`, et le manuel le dit (AC-a13 (4),
  C-32-5-25). Elle ne ferme pas #722 : `_kesh_backup_schedule` reste à part, et les prompts sont la 32-10a.
- **Spike** (`planning-artifacts/epic-32-spike-mcp.md`) § 4.4 : la validation de la clé MCP, distincte de
  `validate_pat`, écrit sa dernière utilisation sur `Trace` ; § 6 (32-5) : la clé se donne au client en portée
  `local`/`user`, jamais en `.mcp.json` versionné sauf `${VARIABLE}` — **texte du manuel, porté par la 32-7** (aucun
  écran ni section MCP du manuel ici, C-32-5-16).

## Ce que le code dit (relevé au code le 2026-10-25, `e59d63e8`)

Chaque fait a été lu ; la commande ou la ligne est citée.

1. **Les PAT.** `PAT_PREFIX = "kesh_pat_"` (`auth/api_key.rs:30`), 160 bits (`:33`), base62 à largeur fixe 27
   (`:38`, `:79-94`), `sha256_hex` (`:97-107`), `generate_pat` (`:113-119`) ; `generate_reset_token` réutilise déjà
   l'entropie et l'encodage (`:134-140`). `validate_pat` (`:153-190`) : recherche de la clé **active** jointe au
   créateur (`api_keys::find_active_auth_by_key_hash`, `repositories/api_keys.rs:123-141` : non révoquée, non
   expirée, `JOIN users`), créateur inactif → 401 (`:163-167`), `touch_last_used` *best-effort* sur le même pool
   (`:185-187`).
2. **Le middleware de l'API** ne connaît que deux formes de jeton : préfixe `kesh_pat_` exact → `validate_pat`
   (`middleware/auth.rs:134`) ; tout le reste → `jwt::decode` (`:146`), qui échoue en 401 (`jwt.rs:96`). Le jeton lu au
   cookie `kesh_access_token` suit le même chemin (`auth.rs:83`, puis `:134`). **Un jeton `kesh_mcp_…` y échoue donc
   déjà en 401** — aucun test ne le fige (`grep -rln "kesh_mcp_" crates frontend/src docs` → rien).
3. **La table `api_keys`** (`migrations/20260605000001_api_keys.sql:18-38`) : `key_hash CHAR(64)` unique, `scope`
   contrôlé par `CHECK`, `expires_at`/`last_used_at`/`revoked_at`, `version` (verrou optimiste de la révocation), FK
   `companies` et `users` en `RESTRICT`. L'entité ne dérive ni `Serialize` ni `Deserialize` et masque le condensat dans
   `Debug` (`entities/api_key.rs:1-12`, `:94-111`).
4. **Les routes des PAT** (`routes/api_keys.rs`) : liste (`:104-112`), création (`:116-200` : nom borné à 255,
   expiration future, secret rendu **une fois**, audit `api_key.created` dans la transaction), révocation (`:205-250` :
   pré-contrôle 404 par société, verrou optimiste, audit `api_key.revoked`) ; `ensure_not_pat` (`:95-100`). Montées
   Comptable+ (`lib.rs:803-813`).
5. **`admin_routes`** (`lib.rs:221-402`, entre les marqueurs `KESH-ADMIN-ROUTES-BEGIN`/`END`) : `require_admin_role`
   puis `require_not_pat` (`lib.rs:393-400`) ; un PAT y reçoit `403 API_KEY_ADMIN_FORBIDDEN` quel que soit son
   créateur (`middleware/rbac.rs:42-66`, fonction l. 58). Le test `admin_pat_denied_e2e` compte **33** constructeurs de
   méthode entre les marqueurs (`lib.rs:222-224`, `tests/admin_pat_denied_e2e.rs:43-46`, et les « 33 » de `:323`,
   `:382`, `:409`, `:769`). `audit_route_registry.rs` fige l'inventaire des routes mutantes à **124** (`:768`) et le
   registre à **127** (`:795`, doc `:27-32`).
6. **La couche de services** (32-3) : `Portee { BooksRead, InvoicesRead }` (`services/acces.rs:20-27`) ;
   `ContexteAcces::mcp(company_id, user_id, portees)` (`:78-84`) ; `Lecture` et `Trace`, pool visible de la seule
   couche de services (`services/poignees.rs:30`, `:49`) ; le doc-comment de `Trace::pool` la destine « aux seules
   fonctions d'audit de `services/rapports.rs` » (`:45-48`) ; l'inventaire du module annonce que « la clé MCP de la
   32-5 écrira sa dernière utilisation sur la `Trace` » (`services/mod.rs:80-83`). Emplois de `Portee::` : **quatorze
   occurrences sur onze lignes** (`grep -rno "Portee::" crates --include=*.rs | wc -l` → 14 ; `grep -rn … | wc -l` →
   11). ⚠️ `tests/services_motif.rs` les **énumère** : `const TOUTES: [Portee; 2]` (`:869`) et `autre_portee`, un
   `match` **exhaustif** sur les deux variantes (`:894-898`) — il ne compile plus à cinq variantes. Gardes lexicales du
   même fichier : aucun `.pool(` hors de `services/` (`:2103-2114`, motif `:1909`) ; `Lecture::new`/`Trace::new` aux
   deux seules lignes de `lib.rs` (`:1885-1894`, `:2120-2127`) ; dans `services/`, **exactement cinq fichiers**
   (`ac10_vi`, `:2137-2141`) et deux lignes seulement qui nomment `axum`, `AppState` ou `CurrentUser` (`:1897-1906`).
   Une `Lecture` de test en `SET SESSION TRANSACTION READ ONLY` existe (`pool_lecture_seule`, `:833-855`).
7. **L'inventaire de la sauvegarde** : `TABLES_TO_TRUNCATE` (`backup.rs:44-96`) compte **39** tables
   (`awk '/^pub const TABLES_TO_TRUNCATE/,/^\];/' crates/kesh-db/src/backup.rs | grep -cE '^\s+"[a-z_]+",'` → 39) ;
   `api_keys` y est l. 75, enfant de `companies` et `users`. `parse_and_verify` exige l'égalité des inventaires dans les
   deux sens (`admin_backup/import.rs:234-250`). `backup_inventory_matches_schema` compare la liste au schéma **du
   squash**, hors `_sqlx_migrations`, `_kesh_version`, `_kesh_backup_schedule` (`backup.rs:821-848`). L'export
   d'installation compte **39** fichiers de données (`tests/admin_full_export_e2e.rs:282`). `restore_body`
   (`backup.rs:627-730`) supprime puis réinsère toute table de l'inventaire, hors `onboarding_state` et `audit_log`.
8. **La remise à zéro** conserve `RESET_PRESERVED_TABLES` = `api_keys`, `companies`, `onboarding_state`,
   `password_reset_tokens`, `refresh_tokens`, `users` (`backup.rs:111-118`, doc `:99-110`) et vide le reste
   (`reset_cleared_tables`, `:129-135`). Le test `reset_partition_is_guarded_by_the_schema` (`:918-1037`) : règle 1
   (une table vidée rejoint `companies`), règle 3 (aucune table conservée ne référence une table vidée), **règle 4**
   (les tables conservées portant une clé vers `companies` sont **exactement** `{api_keys, users}`, `:1025-1036`, doc
   `:910-914`). Les tests de `reset_demo` vivent dans `kesh-api/tests/onboarding_audit_e2e.rs` ; **`kesh-seed` n'a
   aucun test de base** (deux tests purs, `kesh-seed/src/lib.rs:665-677`) et ne dépend pas de `kesh-api`.
9. **La règle des principaux orphelins** (`repositories/companies.rs:278-385`) : verrouille les utilisateurs
   orphelins, **révoque** les clés d'API actives orphelines quelle que soit la cible, puis, avec une cible, rattache
   les utilisateurs et **repointe** toutes les clés orphelines ; `OrphanPrincipals` (`:255-271`, `is_empty` `:267-271`)
   porte `api_keys_revoked` et `api_keys_repointed`, que **quatre** consommateurs écrivent au détail d'une entrée
   d'audit ou au journal du serveur : `repair_installation_in_tx` lui-même (`companies.rs:680-683`,
   `installation.repaired`), `kesh-seed/src/lib.rs:646-647` (`installation.reset`), `routes/onboarding.rs:1245-1246`,
   `auth/bootstrap.rs:432-433`. Des tests les comparent **à égalité JSON stricte** :
   `onboarding_audit_e2e.rs:2055-2070` et `:2460-2472` (lectures de champ `:2362-2365`, `:2407`), `bootstrap.rs:1237-1238`,
   `:1307`, `:1336`. Le montage d'orphelins des tests de `kesh-db` (`tests/support/installations_atteintes.rs:38-41`)
   orpheline `users` et `api_keys` seulement. L'ordre des verrous `users → api_keys` est écrit à
   `companies.rs:560-566`, `kesh-seed/src/lib.rs:489-493`, `auth/bootstrap.rs:1-12`, `routes/admin.rs:510-511` et
   `docs/MULTI-TENANT-SCOPING-PATTERNS.md:327-328`.
10. **La fenêtre d'importabilité (P7).** `POST_RESTORE_BACKFILLS` (`post_restore.rs:210-255`) porte **une** entrée,
    `20261010000001` (classe A, `:225-254`), dont le commentaire dit : « Hors fenêtre ? Non : aucune table applicative
    n'est créée depuis `20260917000001` » (`:250-251`). `registry_entries_are_within_import_window` (`:929-940`)
    recalcule la fenêtre depuis le `MIGRATOR` : **toute migration qui crée une table de `TABLES_TO_TRUNCATE` la
    referme** (`last_table_creating_migration`, `:958-970` ; `creates_application_table`, `:978-1000`). Compteurs codés
    en dur : 8 exemptions « Hors fenêtre » (`:1128`), 20 exemptions (`:1228`). `RETIRED_BACKFILLS` (`:294-…`) garde les
    entrées sorties, que `post_restore_class_a.rs:83-89` éprouve encore. Quatre tests d'import lisent l'entrée au
    **rapport de production** (`lettering_entry`, `admin_full_import_e2e.rs:1657-1663`) :
    `full_import_replays_document_lettering` (`:1860`), `full_import_keeps_a_dissolved_free_reversal_pair` (`:1899`),
    `full_import_of_an_up_to_date_base_is_a_noop` (`:1949`), `full_import_letters_a_reopened_historical_piece`
    (`:1978`) ; et `full_import_report_mirrors_the_production_registry` (`:1554`) exige un registre **non vide**
    (`assert!(!report.is_empty(), …)`, `:1586`). Le précédent de la 24-2 — registre refermé, cas passés à
    `replay_retired` — est écrit `:943-969`. **Textes que la fermeture rend faux** : les justifications d'exemption de
    `20261010000002` (`:559-568` : « une sauvegarde d'avant la 15-1a2-ii importée laisse ces paires ouvertes » ; « portées
    par 20261010000001, au registre »), de `20261009000001` (`:575`), `20261011000001` (`:582`) et `20261015000001`
    (`:591`), qui raisonnent sur l'import d'une « sauvegarde antérieure » désormais refusée, et de `20260917000001`
    (`:436-445`, « la dernière créatrice de table applicative ») ; les commentaires `post_restore.rs:211-224` et
    `:250-251`, `repositories/letterings.rs:22-25` et `tests/post_restore_class_a.rs:43-46` (« entrée de classe A de
    `POST_RESTORE_BACKFILLS` »), `admin_full_import_e2e.rs` — **sept** occurrences de « n'est PAS vide » : le
    doc-comment de `replay_retired` (`:953`) et six commentaires de corps (`:1093`, `:1176`, `:1223`, `:1316`, `:1388`,
    `:1502`) et `:1614` (« Version de M1, au registre (classe A) »), `tests/lettering_documents_backfill.rs:2` et `:46`
    (« registre de rejeu »), `tests/letterings_lexical.rs:19` (« rejouée à l'import »). L'en-tête de la migration
    `20261010000002_lettering_reversal_pairs_backfill.sql:3` (« 20261010000001, au registre de rejeu ») devient faux
    aussi, mais **ne se corrige pas** (P8, migration publiée) : la rectification va dans sa justification d'exemption.
    ⚠️ **`replay_retired` passe le registre retiré ENTIER** (`admin_full_import_e2e.rs:990-995`) : dès que
    `20261010000001` y entre, ses neuf appelants existants (l. 1094, 1177, 1224, 1317, 1389, 1503, 2395, 2512, 2589)
    la rejouent aussi (classe A, gardée `lettering_key IS NULL`, sans effet attendu sur leurs assertions, filtrées par
    entrée ou par action — à constater au gate), et `full_import_replays_backfills_in_increasing_version_order`
    (`:1450`), qui asserte la liste **exacte** des versions rendues (`:1528-1535`, quatre entrées depuis la 25-2-c,
    commentaire `:1522-1527`), rougit.
11. **L'export global** : toute table de l'inventaire est soit exportée, soit à `TABLES_HORS_EXPORT` avec un motif
    (`exports/global.rs:168-…`, garde `:637-690`) ; `api_keys` y est, motif « SECRET » (`:183-188`) ; le commentaire
    `:166-167` dit « Les **dix** ci-dessous sont dans la sauvegarde, qui porte les 39 tables » — la liste en compte
    **neuf** aujourd'hui. Le manuel utilisateur dit ce que l'export ne contient pas, « clés d'API et jetons de
    session » (`user-manual.tex:3027`).
12. **Le journal d'audit** : toute action et tout type d'entité écrits par la production ont leur libellé
    (`audit_labels.rs:24-54` types, `:66-187` actions, dont `api_key.created`/`revoked` l. 79-80), sous la garde
    `tests/audit_label_registry.rs` ; libellés dans les quatre locales (`kesh-i18n/locales/*/messages.ftl`, motif
    `audit-log-action-…`, `api_key.*` à `fr-CH/messages.ftl:2731-2732`). Le manuel d'administration compte les routes
    mutantes tracées, « 108 des 114 routes à verbe mutant au moment de la rédaction » (`admin-manual.tex:2929`).
13. **Un précédent de réglage d'administration à verrou optimiste** : `PUT /api/v1/admin/backup-schedule`
    (`routes/backup_schedule.rs:124-149`) — 400 traduit hors domaine (clé `admin-backup-schedule-error-out-of-domain`,
    repli égal à la valeur fr-CH, garde G9 de `textes_coherents.rs`), 409 sur version périmée, audit écrit par le dépôt.
14. **Colonnes JSON** : précédent `bank_profiles.column_mapping JSON NOT NULL … CHECK (JSON_VALID(column_mapping))`
    (`20260505000001_bank_profiles.sql:27`, `:49`), **lu par `CAST(column_mapping AS CHAR)`**
    (`repositories/bank_profiles.rs:22`) — MariaDB rend une colonne `JSON` en `BLOB` ; `decode_cell` sérialise un
    JSON (rapporté `longtext`) en chaîne (`backup.rs:307-316`), rebindée telle quelle à l'import. Aucun `CHECK` du dépôt
    n'emploie `JSON_TYPE` ni `JSON_LENGTH` (`grep -rln "JSON_TYPE\|JSON_LENGTH" crates/kesh-db/migrations` → rien).
15. **Le chemin positionnel des migrations (P6)** : `migrations_upgrade_path.rs:119-123` (`assert_eq!(total, 83)`) et
    `:208` (`total - 49`, frontière 34) ; les autres sites (`grep -rn "migrations.len()\|apply_migrations_up_to"
    crates/`) résolvent par version (`common::migrations_before`).
16. **Le squash et le registre des checksums.** Le squash (`crates/kesh-db/test-schema/0001_schema_squash.sql`) se
    **régénère** par `scripts/regen-test-schema.sh` (MariaDB démarré) ; `test_schema_guard.rs` rougit s'il dérive.
    `published_migrations_keep_their_checksums` (`test_schema_guard.rs:602-…`) exige **une ligne de
    `migrations.sha384` par migration** du `MIGRATOR` ; l'en-tête du registre : « Une ligne s'AJOUTE quand une
    migration est créée ; elle ne se MODIFIE jamais » (l. 13-14) ; la 31-1c l'a ajoutée dans la PR de sa migration
    (`git show 95ac2735 --stat`).
17. **Le manuel d'administration** : l'avertissement « secret » du `.keshbackup` nomme « les condensés des clés API
    (PAT) » (`admin-manual.tex:2230`) ; *Ce qui reste hors du fichier* (`:2237-2243`) ; le paragraphe **Compatibilité**
    du format 2 (`:2246`) ; le § *Reprises de données rejouées à l'import* (`:2311-2330`) affirme qu'« une sauvegarde
    antérieure retrouve, à l'import, le lettrage de ses factures soldées » (`:2321`) ; la remise à zéro « conserve les
    utilisateurs et les clés d'API » (`:1414`), la réparation #528 « révoque » les clés d'API orphelines (`:1416`,
    `:2461`) ; la requête de contrôle InnoDB attend « au moins 40 » tables (`:2205`) — vrai avant comme après. Le § *Reprises*
    dit « Kesh **rejoue donc ces reprises à la fin de chaque import** » (`:2317`), garde une puce #528 toujours vraie
    (la réparation tourne à chaque import, `routes/admin.rs:512-519`) et finit par un `keshwarning` (`:2326-2330`,
    « réimporter cette même sauvegarde appliquera cette fois la reprise ») ; l'import recalcule aussi, hors registre, la
    canonique des numéros de client à chaque fois (`routes/admin.rs:456-464`, `kesh_db::backfill`), ce que le manuel
    ne décrit pas. Le § *Restaurer sur une nouvelle machine* (`:2371-…`) dit qu'une sauvegarde de format 1 « restaure
    la comptabilité » (`:2373`) et fait installer une version « **égale ou plus récente** » que celle qui a produit la
    sauvegarde (`:2376`) ; `:1505` présente le `.keshbackup` de format 1 comme restaurable. *Ce qui reste hors du
    fichier* (`:2237-2243`) ne nomme pas le réglage de la sauvegarde automatique (`_kesh_backup_schedule`, exclu de
    l'export, `backup.rs:825-826`). Le **manuel utilisateur** répète ce que la 32-5d change : la remise à zéro
    « conserve les utilisateurs et les clés d'API » (`user-manual.tex:190`), le détail d'`installation.reset` nomme
    « les utilisateurs ou clés d'API rattachés » (`:3202`), celui de la réparation « les clés d'API révoquées »
    (`:3209-3212`). Le
    CHANGELOG a sa section `## [0.16.0] — Non publié` (`CHANGELOG.md:11`), qui ne porte qu'un `### Corrigé` (l. 13).
18. **`ipnet 2.12.0`** est dans `Cargo.lock` (l. 1872-1873), sans dépendance directe d'aucun crate de Kesh
    (`grep -n 'ipnet' crates/*/Cargo.toml` → rien). `IpNet::from_str` exige un `/préfixe` et conserve les bits d'hôte
    (`ipnet-2.12.0/src/parser.rs:263-293`).
19. **Deux listes fermées que la 32-5a fait rougir.** (a) La fixture **figée** de la vraie 0.14.0
    (`tests/fixtures/keshbackup-0.14.0-format1.keshbackup`, « Jamais régénérée », `admin_full_import_e2e.rs:3605-3608`)
    porte l'inventaire de la 0.14.0 ; `ac10_la_sauvegarde_de_la_0_14_0_s_importe` (`:3878-3900`, AC10 de la 31-1a)
    attend **200** — après la 32-5a, `parse_and_verify` la refuse. C'est le seul test qui éprouvera la limite de Q4
    contre une sauvegarde réellement produite par une version antérieure. (b) `EMPTY_BEFORE_RESET`
    (`onboarding_audit_e2e.rs:1965-1983`, comparée à `:2013-2019`) est la liste **fermée** des tables vides avant une
    remise à zéro ; les tables conservées se comparent par instantané (`:2024-2028`, `:2079-2084` : `users`,
    `api_keys`, `refresh_tokens`, `password_reset_tokens`).

## Le découpage (C-32-5-1, recompté en validation P1)

Convention de compte (celle du kickoff) : un crate pour `kesh-db`, `kesh-seed`, `kesh-i18n` et la documentation ; un
module de second niveau pour `kesh-api` (`auth`, `services`, `routes`, `exports`, `audit_labels`, `admin_backup` par
ses tests) ; un fichier de test compte avec le module qu'il éprouve ; `tests/common/` (support partagé) compte avec le
module dont il sert les tests.

| Partie | Objet | Modules | 🗄️ |
|---|---|---|---|
| **32-5a** | les deux tables, leurs entités et dépôts, l'inventaire de la sauvegarde, la fenêtre de P7, l'export global, la limite de Q4 au manuel et au CHANGELOG, le refus de la sauvegarde 0.14.0 | `kesh-db` ; `kesh-api/exports` ; `kesh-api/admin_backup` (ses tests) ; `kesh-api/routes` (`tests/onboarding_audit_e2e.rs`, liste `EMPTY_BEFORE_RESET`) ; docs (audit d'idempotence, manuels, CHANGELOG) — **5** | oui |
| **32-5b** | l'authentification par clé MCP : génération DRY, `Portee` unique, `services::cles_mcp::authentifier`, refus croisés, clé restaurée qui authentifie | `kesh-api/auth` (`api_key.rs`) ; `kesh-api/services` (et `tests/services_motif.rs`, `tests/common/`) ; `kesh-api/admin_backup` (le test de restauration) ; `kesh-api/middleware` (par `cles_mcp_e2e.rs`, qui éprouve `require_auth`, F9) — **4** | non (aucun fichier de `kesh-db`) |
| **32-5c** | les routes de gestion (clés, réglages) et leurs gardes, le journal d'audit, les messages traduits | `kesh-api/routes` (+ `lib.rs`, `Cargo.toml`) ; `kesh-api/audit_labels` ; `kesh-i18n` ; docs (`api-external.md`, manuel `:2929`) — **4** | non |
| **32-5d** | la clé MCP à la remise à zéro (conservée) et à la réparation (règle des principaux orphelins), et les textes qui le disent | `kesh-db` (`backup.rs`, `companies.rs`, leurs tests) ; `kesh-seed` ; `kesh-api/auth` (`bootstrap.rs`) ; `kesh-api/routes` (`onboarding.rs`, commentaire d'`admin.rs`, `tests/onboarding_audit_e2e.rs`) ; docs (manuel d'administration `:1414`, `:1416`, `:2461`, manuel utilisateur `:190`, `:3202`, `:3209-3212`, `MULTI-TENANT-SCOPING-PATTERNS.md`) — **5** | oui |

La première rédaction comptait trois parties (4 / 5 / 4) ; la validation P1 a ajouté à l'ancienne partie b un
quatrième consommateur, un fichier de tests et deux passages du manuel (F3, F4, R4, R10), ce qui la portait à **six**
modules : elle se scinde en **b** (authentification) et **d** (remise à zéro et réparation), chacune sous le plafond.
La validation P2 (F3) ajoute à la 32-5a `onboarding_audit_e2e.rs` (module `routes`) : **5**. Recompte (P2, F8) :
**17 modules en somme** des quatre parties (5 + 3 + 4 + 5), **10 distincts** — *recompté en P3 (F9) : la 32-5b
compte aussi `middleware` par ses tests, d'où* **18 en somme** (5 + 4 + 4 + 5), **11 distincts** (`kesh-db`,
`kesh-seed`, `kesh-i18n`, docs, `kesh-api/auth`, `services`, `routes`, `exports`, `audit_labels`, `admin_backup`,
`middleware`).

**Aucun état intermédiaire n'ouvre d'accès ni ne fait mentir un texte** : après 32-5a, les tables existent et sont
sauvegardées, mais aucune clé ne se crée ni ne s'authentifie ; après 32-5b, une clé s'authentifie au service, mais
aucune route ne la crée et aucun endpoint ne la consomme (`/mcp` est la 32-6) ; jusqu'à 32-5d, la remise à zéro **vide**
`mcp_keys` — sans effet, puisque d précède c et qu'aucune clé n'existe donc hors des tests, et l'authentification joint `companies`, si bien qu'une clé orpheline
n'authentifie jamais ; 32-5d change le comportement de la remise à zéro et de la réparation **et**, dans la même PR, les
passages du manuel qui les décrivent (C-32-5-22).

## Critères d'acceptation

### Partie A — 32-5a : les tables et la sauvegarde

1. **AC-a1 — la migration.** Un seul fichier `crates/kesh-db/migrations/20261025000001_mcp_keys_and_server_settings.sql`
   (version vérifiée libre : `ls crates/kesh-db/migrations | grep 20261025` → rien au départ), deux `CREATE TABLE`
   (schéma au § *Dev Notes*), **aucune** instruction qui écrit des données — le détecteur de P7 (`writes_data`, premier
   mot-clé de chaque statement, `every_data_backfill_migration_is_triaged`) fait foi ; `grep -nE
   "^\s*(INSERT|UPDATE|DELETE|REPLACE)\b"` sur le fichier → rien n'en est qu'un complément. En-tête : « NON BREAKING
   (`CREATE TABLE` de deux entités neuves) — pas de bump de `kesh_version_min_required` ; comme toute migration, une
   0.15.0 refuse de démarrer sur une base qui la porte, le retour passe par la sauvegarde d'avant la mise à jour » (P1,
   P2) ; il dit aussi que les deux tables **entrent** à `TABLES_TO_TRUNCATE`, donc que **les sauvegardes antérieures ne
   s'importent plus** (Q4) et que la fenêtre de P7 se referme.
2. **AC-a2 — contraintes de `mcp_keys`.** Chaque contrainte est éprouvée par un test de dépôt qui attend le refus de
   MariaDB : nom vide ou blanc ; `key_hash` en double ; `scopes` non JSON, JSON non tableau, tableau vide ;
   `send_validation` hors `('required', 'exempt')` ; FK `company_id` et `created_by_user_id` (`RESTRICT`). Défaut de
   `send_validation` : `required` (test qui insère sans la colonne). Si `JSON_TYPE`/`JSON_LENGTH` ne sont pas admis dans
   un `CHECK` (fait 14), le repli des Dev Notes s'applique et les tests « non tableau » et « vide » portent sur le
   refus **par Rust**, écrit au Change Log.
3. **AC-a3 — contraintes de `mcp_server_settings`.** Clé primaire `company_id` (une ligne par société au plus), FK
   `companies` `RESTRICT` ; `enabled` défaut **faux** ; les trois listes JSON valides et tableaux. Testé comme AC-a2.
4. **AC-a4 — entités.** `kesh_db::entities::mcp_key` : `McpScope` (cinq variantes `BooksRead`, `InvoicesRead`,
   `ContactsRead`, `DocumentsRead`, `PromptsWrite` ; `as_str` → `books:read`, `invoices:read`, `contacts:read`,
   `documents:read`, `prompts:write` ; `FromStr` strict ; dérive `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
   Hash` ; `McpScope::ALL: [McpScope; 5]`), `McpScopes` (ensemble non vide, trié, sans doublon ; encodage/décodage JSON
   strict : une chaîne inconnue est une **erreur**, jamais ignorée), `SendValidation { Required, Exempt }`, `McpKey`,
   `NewMcpKey` — **ni `Serialize` ni `Deserialize`**, `Debug` manuel masquant `key_hash` (motif
   `entities/api_key.rs:94-111`). `kesh_db::entities::mcp_server_settings` : `McpServerSettings` et
   `McpServerSettings::defaults(company_id)` (C-32-5-8 à 10 ; constantes nommées). Tests unitaires : aller-retour de
   chaque portée, chaîne inconnue refusée, ensemble vide refusé, doublon normalisé, ordre canonique, `Debug` sans
   condensat.
5. **AC-a5 — dépôts.** `repositories::mcp_keys` : `create_in_tx`, `list_by_company(include_revoked)`,
   `find_by_id_for_company` (`None` pour une autre société), `find_active_auth_by_key_hash` (clé non révoquée, non
   expirée, **jointe à `users` et à `companies`** — une clé orpheline n'est jamais trouvée, C-32-5-12), `revoke_in_tx`
   (verrou optimiste, motif `api_keys.rs:149-172`), `set_send_validation_in_tx` (verrou optimiste), `touch_last_used`
   (`revoked_at IS NULL`, motif `:177-189`). `repositories::mcp_server_settings` : `get(company_id) -> Option<…>` (aucune
   écriture) et `save_in_tx(company_id, version, settings)` : `version = 0` → `INSERT` (409 si une ligne existe),
   `version ≥ 1` → `UPDATE … WHERE version = ?` (409 sinon). Les colonnes `JSON` se lisent par `CAST(… AS CHAR)`
   (fait 14). Chaque fonction a son test, branches de refus comprises ; tout est scopé par société (KF-002).
6. **AC-a6 — l'inventaire.** `mcp_keys` à `TABLES_TO_TRUNCATE` **à côté d'`api_keys`** (enfant de `companies` et
   `users`), `mcp_server_settings` **à côté de `company_invoice_settings`** (enfant de `companies`), chacune avec son
   commentaire de place ; **ni l'une ni l'autre** à `RESET_PRESERVED_TABLES` dans cette partie (la remise à zéro les
   vide : règles 1 et 3 vertes ; `mcp_keys` y passe en 32-5d). `backup_inventory_matches_schema` vert, et **rouge** si
   l'une manque (mutation Ma1). Le compte de `admin_full_export_e2e.rs:282` passe de 39 à **41** en nommant les deux
   tables dans son commentaire (« il se met à jour en NOMMANT ce qui entre ») ; `exports/global.rs:166-167` dit
   « Les **onze** ci-dessous … les **41** tables » (recompté sur la liste). **`EMPTY_BEFORE_RESET`** (fait 19 (b),
   C-32-5-26) : `populate` (`onboarding_audit_e2e.rs:1906`) **peuple** une ligne de chaque table neuve, si bien que
   `reset_clears_every_company_table_and_traces_itself` prouve que la remise à zéro les vide (et non qu'elles
   l'étaient déjà) ; la liste fermée reste inchangée.
7. **AC-a7 — l'aller-retour.** `full_roundtrip_rich_dataset_preserves_all_tables` (`admin_backup_e2e.rs`) gagne une
   ligne `mcp_keys` et une ligne `mcp_server_settings` (dont des tableaux JSON non vides) avant l'export : le compte par
   table reste égal après l'import (le test ne passe pas à vide sur les deux tables neuves). Un test dédié compare,
   colonne par colonne, les deux lignes avant et après un export puis un import par HTTP — portées, politique,
   `last_used_at` et `revoked_at` à la milliseconde, listes JSON à l'identique — et vérifie qu'un **serveur activé** dans
   la sauvegarde l'est après l'import (**hypothèse Q7**, C-32-5-11).
8. **AC-a8 — l'export global.** `mcp_keys` et `mcp_server_settings` à `TABLES_HORS_EXPORT` avec leur motif (C-32-5-20) ;
   la garde de `exports/global.rs:637-690` est verte, et rouge si l'une des deux n'y est pas (mutation Ma2) ;
   `exports_global_e2e.rs` vert. Le manuel utilisateur (`user-manual.tex:3027`) nomme aussi, parmi ce que l'export ne
   contient pas, les clés et les réglages du serveur MCP.
9. **AC-a9 — la fenêtre de P7 (C-32-5-13).** `20261010000001` passe de `POST_RESTORE_BACKFILLS` à `RETIRED_BACKFILLS`
   (même contenu) ; son exemption entre à `EXEMPT_MIGRATIONS`, fondement **`ExemptionBasis::Durable`** (comme les huit autres « Hors
   fenêtre » ; `PerishableSince` est exclusif du marqueur), justification **commençant par** « Hors fenêtre depuis
   20261025000001 (mcp_keys, Story 32-5) : » ; compteurs 8 → **9** et 20 → **21**, chacun avec sa ligne de
   commentaire (motif `:1106-1124`, `:1205-1226`). **Textes réécrits** (fait 10), sans changer leur fondement
   `Durable` et **sans la chaîne « aucune version publiée »**, qu'une exemption `Durable` ne peut porter (garde
   `every_exemption_declares_a_coherent_basis`, `post_restore.rs:1178`) :
   - `20261010000002` : plus « au registre » ; « Coût assumé » devient « sans objet depuis 20261025000001 : une
     sauvegarde qui ne porte pas `mcp_keys` est refusée au contrôle de couverture » ; une phrase y dit que l'en-tête de
     la migration (« au registre de rejeu », `…_lettering_reversal_pairs_backfill.sql:3`) est périmé et ne se corrige
     pas (P8) ;
   - `20261009000001`, `20261011000001`, `20261015000001` : la phrase sur « une sauvegarde antérieure » devient « sans
     objet depuis 20261025000001 : une sauvegarde antérieure à cette migration ne porte pas `mcp_keys` et est refusée
     au contrôle de couverture » (P2, F11 : borner à « 0.16.0 et suivantes » ne désignerait rien) ;
   - `20260917000001` : elle n'est plus la dernière créatrice de table ; son fondement reste la **couverture** ;
   - commentaires : `post_restore.rs:211-224` (« M1 ci-dessous », « TROIS reprennent matière ») et `:250-251`, `letterings.rs:22-25`, `post_restore_class_a.rs:43-46`,
     `lettering_documents_backfill.rs:2` et `:46`, `letterings_lexical.rs:19`.
   Les deux justifications qui se disent « dans la fenêtre » alors qu'elles en sont sorties depuis la 25-2-c
   (`20260909000001`, `20260915000001`) sont **hors périmètre**, signalées au Change Log.
   `registry_entries_are_within_import_window`, `every_data_backfill_migration_is_triaged`,
   `exemptions_claiming_out_of_window_really_are_out_of_window`, `every_exemption_declares_a_coherent_basis` et le
   décompte des fondements sont verts.
10. **AC-a10 — les tests d'import du lettrage.** Les quatre tests de `admin_full_import_e2e.rs:1860-2028` lisent l'entrée
    `20261010000001` par `replay_retired` (le registre retiré injecté, motif `:943-969` ; le manifeste est en main par
    `unzip` ; `retired_entry` rend l'entrée typée, `.outcome.code()` et `.rows_affected` remplacent `m1["outcome"]` et
    `m1["rows_affected"]`) au lieu du rapport de production, et gardent leurs assertions sur les marques ;
    `full_import_report_mirrors_the_production_registry` revient à son état de la 25-2-c : rapport **vide**, égal au
    registre vide, avec un commentaire qui le dit (et sans l'assertion de montage de `:1586`) ;
    `full_import_replays_backfills_in_increasing_version_order` (`:1450`) attend **cinq** entrées, `LETTERING_BACKFILL`
    en dernier, et son commentaire `:1522-1527` passe de quatre à cinq ; `LETTERING_BACKFILL` (son commentaire `:1614`)
    et `lettering_entry` sont réécrits ou retirés ; le doc-comment de `replay_retired` (`:953`) et les six commentaires de corps qui répètent « n'est PAS vide »
    (fait 10) disent le nouvel état.
    Les neuf autres appelants de `replay_retired` (fait 10) rejouent désormais `20261010000001` : le gate le constate,
    le Change Log le dit. Aucun des tests convertis ne passe à vide : chacun garde au moins une assertion sur une
    marque posée par le rejeu.
11. **AC-a11 — P5 et P6.** Une ligne au tableau de `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx` :
    `CREATE TABLE` sans `IF NOT EXISTS` ; non breaking ; aucune donnée écrite → ni registre ni exemption ; les deux
    tables entrent à l'inventaire et referment la fenêtre ; **`20261010000001` passe au registre retiré et à
    `EXEMPT_MIGRATIONS`**, ce que la ligne dit — patron de la ligne de la 25-2-c, `:91`) ; la ligne de `20261010000001`
    (`:99`, « inscrite au registre … en CLASSE A …, rejouée à chaque import ») gagne la mention « retirée du registre
    par la 32-5a, exemptée Hors fenêtre depuis 20261025000001 », et celle de `20261010000002` (`:100`), qui n'a jamais
    été au registre, dit seulement que son coût assumé ne se présente plus : une sauvegarde d'avant la 15-1a2-ii est
    refusée au contrôle de couverture depuis 20261025000001 ; la ligne `:89` (`20260910000001`), restée au présent depuis la 25-2-c, est signalée au Change Log ; en-tête `## Table d'audit (84 migrations)`, `Total` 84
    (« + 1 Story 32-5 »), `tracked-by-sqlx` 74, `yes` 10, `no` 0 — **recomptés** :
    `ls crates/kesh-db/migrations/*.sql | wc -l` et `grep -c '^| \`20' docs/migrations-idempotence-audit.md` → 84 et
    84, et somme des trois partitions = 84. P6 : `migrations_upgrade_path.rs` passe à `total == 84` et `total - 50`
    (frontière **34** inchangée), avec la ligne « Story 32-5 : 49 → 50, frontière inchangée (84 - 50 = 34) » et la ligne
    « = 84 » au cumul (`:118`) ; **chaque occurrence** de `\b49\b` et de `\b83\b` du fichier
    (`grep -nE "\b(83|49)\b" crates/kesh-db/tests/migrations_upgrade_path.rs`, quinze lignes au départ), triée à la
    main, suit — on grepe la valeur, non une liste de lignes ; les autres sites du `grep` de P6 sont
    relus et résolvent par version.
12. **AC-a12 — le squash et le registre des checksums.** Le squash est régénéré (`scripts/regen-test-schema.sh`) ; la
    ligne `20261025000001 <sha384 du fichier final>` est **ajoutée** à `crates/kesh-db/migrations.sha384` dans la même
    PR (fait 16, motif de la 31-1c), **en dernier**, une fois les `CHECK` éprouvés (AC-a2) : à partir de là, le fichier
    de migration ne se modifie plus (P8). `test_schema_guard.rs` vert
    (`cargo nextest run -E 'binary(test_schema_guard)'`), dont `published_migrations_keep_their_checksums`.
13. **AC-a13 — la limite de Q4, au manuel et au CHANGELOG (C-32-5-21, révisé en P2).** Dans
    `docs/manual/fr/admin-manual.tex` — **chaque phrase vraie dès la fusion de la 32-5a** :
    (1) paragraphe **Compatibilité** (`:2246`) : la règle **sans date** (F12 — elle vaut depuis la v0.12.0 au moins,
    qui a ajouté `invoice_settlements` et `journal_entry_number_sequences`) : une sauvegarde ne se réimporte que dans une
    version de Kesh **qui porte les mêmes tables** ; la 0.16.0 en ajoute deux, si bien qu'elle **refuse** les
    sauvegardes de la 0.15.0 et d'avant (message *« Fichier de sauvegarde invalide ou corrompu »*, motif au journal
    du serveur « table … absente du manifeste ») ; la phrase « La 0.15.0 importe les sauvegardes des deux formats »
    se borne à la 0.15.0. Sur la restauration d'une sauvegarde plus ancienne, le manuel dit **seulement**, outre la
    règle ci-dessus (C-32-5-28, réduit en validation P5) : une sauvegarde plus ancienne se restaure **avec la version
    qui l'a produite**, dont le manuel se trouve **dans le dépôt, au tag de cette version** ; **ensuite, on met à
    jour**. Aucune procédure n'est prescrite, aucune section d'un manuel ancien n'est citée, aucun fichier joint à une
    release n'est promis (`release.yml:161-165` n'en attache aucun). Le lieu se dit **en mots** : un gabarit
    `raw.githubusercontent.com/…/v<version>/` ferait rougir la garde G19 (`textes_coherents.rs:1250-1285`).
    **Phrase distincte**, hors de ce qui précède (P6-2) : après la mise à jour vers la 0.16.0, **prendre une sauvegarde
    complète juste après la mise à jour** — les sauvegardes précédentes, automatiques comprises, ne s'y importent plus ;
    (2) § *Reprises de données rejouées à l'import* (F4 = R4) : **aucune reprise du registre** n'est plus rejouée depuis
    la 0.16.0 — une sauvegarde assez ancienne pour en avoir besoin n'est plus importable ; la phrase de tête (`:2317`)
    et l'alinéa du lettrage (`:2321`) le disent ; la puce #528 **reste** (la réparation tourne à chaque import, sans
    effet sur une installation saine), précisée : une sauvegarde touchée par #528 date d'une 0.12.x et n'est plus
    importable ; le recalcul des numéros de client, qui tourne aussi à chaque import, est nommé ; le `keshwarning`
    (`:2326-2330`) est **réécrit** : réimporter une sauvegarde d'avant le rejeu n'est plus possible, seule la
    correction à la main reste (rôles, compte de produit) ;
    (3) l'avertissement « secret » (`:2230`) nomme aussi, **depuis la 0.16.0**, les condensés des clés du serveur MCP
    (vrai dès cette partie : la table est sauvegardée, même vide) ;
    (4) *Ce que la sauvegarde emporte* et *Ce qui reste hors du fichier* (`:2237-2243`, #722) : les clés et les
    réglages du serveur MCP — **activation comprise** — **sont dans** le fichier ; la liste de ce qui reste hors du
    fichier gagne **le réglage et l'état de la sauvegarde automatique**, propres à l'installation (R9 ; défaut
    antérieur, suivi par #722) ;
    (5) **La classe, non la ligne** (P7-1) : le développement relit **en entier** le § *Restaurer sur une nouvelle
    machine* (`:2371-…`) et le paragraphe *Compatibilité* (`:2246`) ; **toute** mention de format (« format 1 »,
    « format 2 ») ou de version (« 0.15.0 », « ou plus récente », « égale ») y est réécrite pour rester vraie après la
    32-5a. Premier site connu, **sans que la liste soit close** : `:2373`, « La sauvegarde doit être de format 2 (0.15.0
    ou plus récente) pour emporter les pièces » — la 0.16.0 refuse une sauvegarde de la 0.15.0, la clause se réduit au
    fait que le format 2 emporte les pièces, la version renvoyant à *Compatibilité*. **C'est le grep qui ferme la
    liste** : `grep -nE "format [12]|0\.15\.0|ou plus récente|égale" docs/manual/fr/admin-manual.tex` restreint aux
    lignes de ces deux §, et le même motif sur le PDF aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' '
    | tr -s ' ' | grep -oiE ".{60}(format [12]|0\.15\.0|ou plus récente|égale).{80}"`), **sorties citées au Dev Agent
    Record**, chaque occurrence restante justifiée. En particulier, `:2373` (« une sauvegarde de format 1 restaure la
    comptabilité ») et `:2376` (« dans une version **égale ou plus récente** ») deviennent : l'étape 1 **ne nomme plus aucune
    version à installer** ; elle se réduit à la règle — la version installée doit porter les mêmes tables que la
    sauvegarde — et renvoie au paragraphe *Compatibilité* que prescrit (1), seul lieu de la phrase sur une sauvegarde
    plus ancienne (P6-1) ; une sauvegarde de format 1 (0.14.x et avant) ne s'importe plus dans la 0.16.0 ; `:1505` dit de même du
    `.keshbackup` de format 1, et *Les pièces à l'import* (`:2279`, « Une sauvegarde de format 1 … restaurée sur la
    même installation, elle retrouve les pièces ») se borne aux versions qui l'importent encore (0.15.x). Les autres
    occurrences de « format 1 » restent vraies et ne changent pas : `:1501` (le format 1 n'emporte pas le PDF), `:2266`
    (refus d'un manifeste de format 1 qui déclare des pièces), `:1629` (la répétition de clôture exporte et réimporte
    dans une même version).
    `CHANGELOG.md`, section `[0.16.0]` : **créer** `### Modifié`, une entrée ⚠️ de même substance : la règle des mêmes tables, la phrase de (1) sur une sauvegarde plus ancienne, et le
    rappel de prendre une sauvegarde complète juste après la mise à jour. PDF régénéré (`make fr` dans `docs/manual/`) et **contrôlé aplati**, apostrophes typographiques
    comprises (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}mêmes
    tables.{80}"` rend la phrase ; idem « au tag de cette version », « juste après la mise à jour », « serveur MCP », « sauvegarde automatique » dans la liste, et
    plus aucune occurrence de « égale ou plus récente », « 0.15.0 ou plus récente » ni de « appliquera cette fois »).
14. **AC-a14 — propagation.** Rendent zéro site périmé (les mentions historiques datées restent, marquées comme
    telles) : `grep -rnE "\b39\b" crates docs --include=*.rs --include=*.tex --include=*.md | grep -i table` ;
    `grep -rn "dix ci-dessous" crates` ; `grep -rn "aucune table applicative n'est créée depuis" crates` ;
    `grep -rn "classe A de \`POST_RESTORE_BACKFILLS\`\|au registre (Story 15-1a2-ii\|registre de rejeu\|au registre (classe A)\|rejouée à l'import" crates --include=*.rs` ;
    `grep -rn "n'est PAS vide" crates/kesh-api/tests/admin_full_import_e2e.rs` ;
    `grep -rnE "égale ou plus récente|0\.15\.0 ou plus récente|format 1 restaure|appliquera cette fois|rejoue donc ces reprises" docs/manual` ;
    et le grep de classe d'AC-a13 (5) (`format [12]|0\.15\.0|ou plus récente|égale`) sur les deux § qu'il vise.
    ⚠️ Lignes attendues, exemptées : l'en-tête de `20261010000002_lettering_reversal_pairs_backfill.sql:3` et celui de
    `20260909000001_closing_accounts_not_postable.sql:52` (migrations, P8), les historiques datés de
    `post_restore.rs:421` et `:497` (« Figurait au registre de rejeu jusque-là ») ; chaque occurrence de « format 1 »
    au manuel est relue contre AC-a13 (5).
15. **AC-a15 — la sauvegarde de la 0.14.0 est refusée (F2, C-32-5-26).** `ac10_la_sauvegarde_de_la_0_14_0_s_importe`
    (`admin_full_import_e2e.rs:3878-3900`) devient `ac10_la_sauvegarde_de_la_0_14_0_est_refusee` : la fixture figée
    rend **400** `INVALID_BACKUP_STRUCTURE`, **aucune** entrée `admin.full_import`, et aucune table changée (compte par
    table avant et après) ; son doc-comment dit que l'AC10 de la 31-1a est dépassé par Q4 et pourquoi. C'est le seul
    test de la limite de Q4 contre une sauvegarde réellement produite par une version antérieure ; le format 1
    reste éprouvé par `format_1_restauree_sur_la_meme_installation` (`:3859`, par le helper `en_format_1`, `:3692`, qui part d'un export courant). La fixture et son générateur
    (`admin_full_export_e2e.rs:509-534`, `#[ignore]`) ne changent pas.

### Partie B — 32-5b : l'authentification par clé MCP

16. **AC-b1 — génération (DRY, C-32-5-15).** `auth/api_key.rs` : une fonction privée commune (entropie, base62 à
    largeur fixe, condensat) appelée par `generate_pat` et par la nouvelle `generate_mcp_key` ; `MCP_PREFIX =
    "kesh_mcp_"`. Tests : format `kesh_mcp_` + 27 base62 (36 caractères), condensat = `sha256_hex(jeton)`, unicité,
    **aucun des deux préfixes n'est préfixe de l'autre**, et les tests existants de `generate_pat` inchangés et verts.
    `grep -c "OsRng.fill_bytes" crates/kesh-api/src/auth/api_key.rs` rend 2 avant (l. 115, 136) et ne croît pas.
17. **AC-b2 — `Portee` unique (C-32-5-4, révisé).** `services::acces::Portee` devient `pub use
    kesh_db::entities::McpScope as Portee;` ; les variantes et dérivations de la 32-3 sont conservées ; les **trois** occurrences de
    `src/` (`rapports.rs:134`, `:172`, `plan_comptable.rs:26`) compilent inchangées, les **onze** de `services_motif.rs`
    sont revues. **`tests/services_motif.rs` change** (fait 6) :
    `TOUTES` devient `McpScope::ALL` (cinq) ; `autre_portee` ne se limite plus à l'autre des deux pilotes — chaque test
    de refus de portée (R7 de la 32-3) éprouve la clé qui porte **les quatre autres portées** (`ALL` privé de la portée
    exigée) et attend le refus — dont `:1201` (`autre_portee`) et le second refus de `plan_comptable::lister`, portée en dur
    (`:1220`, `for portees in [vec![], vec![Portee::InvoicesRead]]`) ; le positif reste la portée exigée seule. `ac10_vi` compte **un fichier de plus**
    que sur `main` au départ de la 32-5b (`services/cles_mcp.rs` ; cinq à six au relevé de `e59d63e8`, à recompter à
    T-b0 : la 32-4 ajoute des services), `RESIDUS_VI` inchangé. Les comptes de ce paragraphe (trois occurrences de
    `src/`, onze de `services_motif.rs`, `:1201`, `:1220`) sont ceux de `e59d63e8` : recomptés à T-b0. `services_motif.rs`
    gagne `mod common;` pour appeler `pool_lecture_seule` (il n'en a pas aujourd'hui ; les sous-modules de `common/` y
    compilent alors, sans effet — F9).
18. **AC-b3 — l'authentification (C-32-5-14).** `services::cles_mcp::authentifier(&Lecture, &Trace, jeton)` rend
    `CleMcpAuthentifiee { cle_id, acces: ContexteAcces /* canal Mcp, portées de la clé */, validation: SendValidation }`
    ou `AppError::Unauthenticated` (401) — **même** erreur, au texte près côté client, pour : préfixe absent ou autre
    (dont un PAT **valide et actif**), clé inconnue, révoquée, expirée, créateur inactif, créateur qui n'est plus
    Administrateur (**hypothèse Q3**, constante `ROLE_CLE_MCP`, C-32-5-7), société absente. **Une erreur de la base n'est jamais
    un 401** : elle se propage (`AppError::Database`, par `?`, motif `validate_pat`, `auth/api_key.rs:159-161`). Ordre :
    préfixe exact (octets) **avant tout accès à la base** — prouvé par deux cas, poignées sur un pool `connect_lazy` vers
    un **port fermé** (`127.0.0.1:1`) avec un `acquire_timeout` court (le `stub_pool` de `middleware/auth.rs:231-235`
    vise `127.0.0.1:3306`, joignable sur la station : il ne convient pas) : (α) jeton **sans** le préfixe → **401** ;
    (β) témoin, jeton **avec** le préfixe → erreur de base, **pas** 401 — sans (β), une implémentation qui
    uniformiserait tout refus en 401 laisserait passer Mb6 ; recherche sur la **`Lecture`** ; dernière utilisation *best-effort* sur la **`Trace`**
    (un échec est journalisé, n'échoue pas). Le module ne nomme ni `axum`, ni `AppState`, ni `CurrentUser` dans ses
    lignes de code (garde (vi), `RESIDUS_VI` inchangé) ; `poignees.rs:45-48` et `services/mod.rs:80-83` disent le
    nouvel usage de la `Trace`, et les sites antérieurs à Q11 (« utilisateur MariaDB limité au `SELECT`, ou les
    deux ») disent la décision — lecture seule applicative, mode de session `READ ONLY`, à la 32-6 : `poignees.rs:9-12`
    et `services/mod.rs:93-95` (F13, F2 = R5 ; les deux sites de `lib.rs:147-148` et `:158-159` vont à la 32-5c, qui
    touche `lib.rs`) ; le doc-comment de `ContexteAcces` (`acces.rs:57-58`, « les tests et la 32-6 ») nomme aussi
    `cles_mcp::authentifier`, son constructeur de production (R6).
19. **AC-b4 — refus croisés.** (a) Une clé MCP **active**, portée `books:read`, présentée à `GET /api/v1/accounts` en
    `Authorization: Bearer` **et** en cookie `kesh_access_token` — **deux requêtes distinctes**, le cookie primant sur
    l'en-tête (`middleware/auth.rs:83-87`) — → **401** les deux fois, et sa `last_used_at` reste
    `NULL` ; (b) la même clé présentée à une route d'`admin_routes` → 401 ; (c) un **PAT actif** `read-write` passé à
    `authentifier` → 401, et sa `last_used_at` (table `api_keys`) reste inchangée ; (d) le jeton d'un PAT recopié
    derrière le préfixe `kesh_mcp_` → 401 (condensat absent de `mcp_keys`). **Le refus d'un PAT par l'endpoint `/mcp`
    lui-même est un AC de la 32-6** (aucun endpoint ici) ; la fiche 32-6 le reprend tel quel. Tests dans un fichier neuf
    `crates/kesh-api/tests/cles_mcp_e2e.rs` (montage de `tests/common/`).
20. **AC-b5 — la séparation lecture / trace, à l'exécution.** Un test appelle `authentifier` avec une `Lecture` en
    session `READ ONLY` (helper `pool_lecture_seule` de `services_motif.rs:833-855`, **déplacé** dans `tests/common/` et appelé par les deux
    fichiers — DRY, F15) et une `Trace`
    ordinaire : succès, et `last_used_at` posée. Témoin : sur cette `Lecture`, l'`UPDATE mcp_keys SET last_used_at = …`
    **lui-même** échoue en 1792 (sans quoi un changement de la requête de `touch_last_used` ne serait pas couvert).
    Mutation Mb3 (la dernière utilisation écrite sur la `Lecture`) → le test rougit.
21. **AC-b6 — une clé restaurée authentifie (C-32-5-23).** Test d'intégration HTTP **dans `admin_full_import_e2e.rs`**,
    dont il réutilise le montage privé (`spawn_app`, `seed_business`, `export_backup`, `unzip`, `import_ok`,
    l. 75-800) : créer la clé K1 (dépôt + `generate_mcp_key`), exporter, **supprimer K1** (`DELETE FROM mcp_keys` direct,
    le dépôt n'offrant pas de suppression) et créer K2, importer le fichier : K1 **authentifie** avec ses portées et sa
    politique d'origine, K2 **n'authentifie plus** (absente de la sauvegarde) ; une clé révoquée avant l'export reste
    refusée après l'import. Aucun privilège posé à la main.

### Partie C — 32-5c : les routes de gestion

22. **AC-c1 — les routes (C-32-5-17)**, dans le bloc `admin_routes` : `GET /api/v1/admin/mcp/keys` (toutes les clés de
    la société, révoquées comprises, jamais de condensat ni de secret) ; `POST /api/v1/admin/mcp/keys` (`name`,
    `scopes` non vide, `expiresAt` facultatif et futur, `sendValidation` facultatif, défaut `required`) → **201** avec
    le secret **une seule fois** ; `DELETE /api/v1/admin/mcp/keys/{id}` (verrou optimiste, 404 pour une autre société)
    → 204 ; `PUT /api/v1/admin/mcp/keys/{id}/send-validation` (`sendValidation`, `version`) → 200 ;
    `GET /api/v1/admin/mcp/settings` (défauts et `version: 0` sans ligne) ; `PUT /api/v1/admin/mcp/settings` (tous les
    champs, `version`) → 200.
23. **AC-c2 — gardes.** Pour **chaque** route : Comptable → 403 `FORBIDDEN` ; PAT, même créé par un Administrateur →
    403 `API_KEY_ADMIN_FORBIDDEN` (jambe `read-write`, `admin_pat_denied_e2e.rs:769`) et, pour un PAT `read` sur une
    méthode mutante, `API_KEY_READ_ONLY` (jambe `read-only`, `:792-815`) ; clé MCP → 401 ; sans authentification → 401.
    Les deux handlers `GET` **ne portent pas** `ensure_not_pat` (la couche suffit) ; la répartition `(8, 25)` → **`(10, 29)`**
    (`:830-836`) et `discriminating` 6 → **8** (`:881-888`) suivent — à **chaque** site de ces valeurs : le doc-comment
    `:795-797` (« Les 8 `get` … les 25 méthodes mutantes … ces vingt-cinq-là ») et la généalogie `:831-833`, qui gagne
    la ligne « (8, 25) → (10, 29) : Story 32-5c ». Les deux commentaires antérieurs à Q11 de
    `lib.rs:147-148` et `:158-159` (F2 = R5) se corrigent ici. Les six couples entrent à
    `admin_pat_denied_e2e` (33 → **39**, à chaque site du nombre : `lib.rs:224`, `admin_pat_denied_e2e.rs:46`, `:323`,
    `:382`, `:409`, `:769`) ; les quatre routes mutantes à `audit_route_registry.rs` (`Traced`, et la classe de rejeu
    que l'inventaire établit), inventaires 124 → **128** et 127 → **131** (`:768`, `:795`, doc `:27-32`), `traced` 118 → **122** (`:770-771`, avec
    sa ligne de généalogie au message), la partition de rejeu (`:1096-1129` : `sans_ecriture` 99 ou `rejouees`, selon la
    classe établie, +4 ; `exemptees` 3 inchangé ; `tout.len()` 127 → **131**, `:1129`) et leurs messages.
24. **AC-c3 — validation des clés.** Nom vide ou > 255 caractères, portée inconnue (`books:write`, `admin`, `read`),
    liste vide, expiration passée, politique inconnue → 400, **rien d'écrit, aucun audit** ; portées en double →
    normalisées. Les messages passent par `t(…)` avec repli égal à la valeur fr-CH (garde G9), clés dans les **quatre**
    locales.
25. **AC-c4 — validation des réglages (C-32-5-19, révisé).** Chaque entrée de `allowedNetworks` et `trustedProxies` est
    lue d'abord comme **adresse** (`IpAddr`, stockée `/32` ou `/128`), sinon comme **réseau** (`IpNet`) ; un réseau qui
    porte des **bits d'hôte** (`10.1.2.3/8`) est **refusé** (400, message qui donne la forme attendue, ici
    `10.0.0.0/8`) ; la forme stockée est celle d'`IpNet::to_string` (canonique) ; chaque origine est
    `http(s)://hôte[:port]` sans chemin, stockée en minuscules ; listes bornées (64 entrées, 255 caractères l'une) ;
    doublons retirés ; entrée invalide → 400 traduit, rien d'écrit. 409 sur version périmée ; `version: 0` alors qu'une
    ligne existe → 409. `Cargo.lock` gagne `ipnet` dans les dépendances de `kesh-api` (même version 2.12.0).
26. **AC-c5 — audit (C-32-5-18).** `mcp_key.created` (nom, portées, politique), `mcp_key.revoked`,
    `mcp_key.send_validation_changed` (avant, après), `mcp_server_settings.updated` (avant, après) — dans la transaction
    de l'écriture ; **ni secret ni condensat** dans aucun détail (test qui cherche le jeton et son condensat dans
    `audit_log.details_json` → rien). Types `mcp_key`, `mcp_server_settings` et les quatre actions à `audit_labels.rs`,
    libellés dans les quatre locales ; `audit_label_registry.rs` vert.
27. **AC-c6 — une clé créée par la route s'authentifie** par `services::cles_mcp::authentifier` avec exactement les
    portées et la politique demandées ; après `DELETE`, elle ne s'authentifie plus ; après le `PUT` de politique, la
    nouvelle politique est rendue.
28. **AC-c7 — documentation.** `docs/api-external.md` § 2 *Format de la clé* : une clé d'API a la forme `kesh_pat_…` ;
    une clé `kesh_mcp_…` (serveur MCP, 0.16.0) **n'est pas** une clé d'API et `/api/v1` la refuse en 401 ; rien
    d'autre (les § 6.4 et § 9 changent avec la 32-11, prémisse 10 du kickoff). Manuel d'administration `:2929` : le
    compte « N des M routes à verbe mutant » **recompté** à la date de la 32-5c (commande écrite au Change Log). PDF
    régénéré et contrôlé aplati.

### Partie D — 32-5d : la clé MCP à la remise à zéro et à la réparation

29. **AC-d1 — la remise à zéro conserve les clés (C-32-5-11).** `mcp_keys` entre à `RESET_PRESERVED_TABLES` (doc
    `backup.rs:99-110` complétée) ; règle 4 du test de partition : `{api_keys, mcp_keys, users}` (doc `:910-914`,
    message `:1033-1036`) ; « les six tables conservées » (`:848-851`) → sept. Test **dans
    `kesh-api/tests/onboarding_audit_e2e.rs`** (C-32-5-23 ; `kesh-seed` n'a ni test de base ni accès à
    `authentifier`, fait 8) : après `reset_demo`, une clé MCP active créée avant **authentifie encore**, et
    `mcp_server_settings` est **vide** (serveur désactivé). Dans le test de la remise à zéro (fait 19 (b)), `mcp_keys`
    rejoint les deux listes d'instantanés des tables conservées (`onboarding_audit_e2e.rs:2024-2028`, `:2079-2084`) —
    la ligne que `populate` y a posée en 32-5a survit, identique.
30. **AC-d2 — principaux orphelins (C-32-5-12, révisé).** `reattach_orphan_principals_in_tx` verrouille les clés MCP
    **après** les clés d'API, révoque les actives orphelines **quelle que soit la cible**, puis, avec une cible, les
    repointe toutes ; `OrphanPrincipals` gagne `mcp_keys_revoked: Vec<RevokedMcpKey>` (aucun condensat) et
    `mcp_keys_repointed`, pris en compte par `is_empty`. **Les quatre consommateurs** (fait 9) écrivent **toujours** les
    deux champs (`[]` et `0` quand il n'y a rien : JSON d'audit à forme stable), et leurs tests à égalité stricte sont
    complétés (`onboarding_audit_e2e.rs:2055-2070`, `:2460-2472`, `bootstrap.rs:1237-1238` ; lectures de champ `onboarding_audit_e2e.rs:2362-2407`, `bootstrap.rs:1307`, `:1336`, `companies_repository.rs:619-862`). Le montage
    d'orphelins (`installations_atteintes.rs:38-41`) orpheline aussi `mcp_keys`. Tests : (i) une clé MCP active
    orpheline à côté d'utilisateurs et de clés d'API orphelins est révoquée **et** repointée (cible) ou révoquée seule
    (sans cible), n'authentifie dans aucun cas, et paraît au détail de l'entrée ; (ii) **la clé MCP est le seul
    orphelin** — aucun utilisateur ni clé d'API orphelin (montage sous `FOREIGN_KEY_CHECKS = 0`, comme
    `mount_orphans`) : `is_empty()` est faux, `repair_installation_in_tx` écrit son entrée `installation.repaired`, la
    clé est à son détail (cas qui tue Md3, F5). Doc-comments et texte de l'ordre des verrous
    `users → api_keys → mcp_keys` à chaque site du fait 9 (`companies.rs:560-566`, `kesh-seed/src/lib.rs:489-493`,
    `bootstrap.rs:1-12`, `routes/admin.rs:510-511`, `MULTI-TENANT-SCOPING-PATTERNS.md:327-328`).
31. **AC-d3 — le manuel dit le nouveau comportement, dans la même PR (C-32-5-22).** `admin-manual.tex:1414` : la remise
    à zéro conserve aussi les clés du serveur MCP — « revoyez-les » s'y applique (une clé créée pendant une
    démonstration garde l'accès en lecture aux livres de production) — et remet le serveur MCP à « désactivé » ;
    `:1416` et `:2461` : la réparation révoque aussi les clés MCP orphelines et les rattache, révoquées. **Manuel
    utilisateur** (R3) : `user-manual.tex:190` (la remise à zéro conserve aussi les clés du serveur MCP), `:3202`
    (le détail d'`installation.reset` nomme aussi les clés MCP rattachées), `:3209-3212` (celui de la réparation, les
    clés MCP révoquées). Les deux PDF régénérés et contrôlés aplatis. La section *Le serveur MCP* (création, portées, écran) reste à la 32-7, qui **reprend** ces trois
    passages et les trois du manuel utilisateur pour y renvoyer (report inscrit à la section 32-7 du fichier d'epic).

## Tâches

### 32-5a
- [ ] **T-a0** — recontrôle au code des faits 7, 10, 11, 14, 15, 16, 17 (ligne lue citée au Change Log) ; `git log -1`
  = partie de `main` à jour ; version de migration libre.
- [ ] **T-a1** — migration (AC-a1), `CHECK` éprouvés **à la main** sur `kesh_325` (le fichier peut encore changer).
- [ ] **T-a2** — entités et dépôts, leurs tests (AC-a2 à AC-a5) ; **puis**, les `CHECK` éprouvés par les tests
  d'AC-a2, squash régénéré et ligne de `migrations.sha384` **en dernier** (AC-a12) — à partir de là, P8.
- [ ] **T-a3** — inventaire, export global, aller-retour (AC-a6 à AC-a8).
- [ ] **T-a4** — fenêtre de P7, justifications, tests d'import du lettrage, refus de la 0.14.0 (AC-a9, AC-a10,
  AC-a15).
- [ ] **T-a5** — audit d'idempotence, P6 (AC-a11).
- [ ] **T-a6** — manuels, PDF, CHANGELOG, propagation (AC-a13, AC-a14) — et § *À recontrôler à l'étape 0 du développement* (P8-1 à P8-4, `user-manual.tex:1914`).
- [ ] **T-a7** — mutations Ma1 à Ma7, gates (§ *Gates*).

### 32-5b
- [ ] **T-b0** — recontrôle des faits 1, 2, 6 ; 32-5a fusionnée ; **recompte**, sur `main` au départ de la 32-5b, des
  fichiers de `services/`, des emplois de `Portee::` et des refus de portée de `services_motif.rs` (la 32-4a, PR #745,
  et sans doute la 32-4b, seront fusionnées avant : leurs comptes changent) — écrits au Dev Agent Record.
- [ ] **T-b1** — génération DRY, `Portee`, `services_motif.rs` (AC-b1, AC-b2).
- [ ] **T-b2** — `services::cles_mcp::authentifier` et ses tests (AC-b3, AC-b5).
- [ ] **T-b3** — refus croisés et clé restaurée (AC-b4, AC-b6).
- [ ] **T-b4** — mutations Mb1 à Mb6, gates.

### 32-5c
- [ ] **T-c0** — recontrôle des faits 4, 5, 12, 13, 18 ; 32-5d fusionnée (ordre imposé, C-32-5-27).
- [ ] **T-c1** — routes, gardes, validation (AC-c1 à AC-c4) ; `ipnet` en dépendance directe (même version).
- [ ] **T-c2** — audit, libellés, quatre locales (AC-c5) ; AC-c6.
- [ ] **T-c3** — documentation (AC-c7).
- [ ] **T-c4** — mutations Mc1 à Mc6, gates.

### 32-5d
- [ ] **T-d0** — recontrôle des faits 8, 9, 17 ; 32-5b fusionnée.
- [ ] **T-d1** — remise à zéro (AC-d1).
- [ ] **T-d2** — principaux orphelins, quatre consommateurs, montage, textes de l'ordre des verrous (AC-d2).
- [ ] **T-d3** — manuel, PDF (AC-d3).
- [ ] **T-d4** — mutations Md1 à Md4, gates.

## Tests et mutations prévus

Une mutation à la fois, restaurée par `git checkout -- <f> && touch <f>` (`memory/mutation-restauree-binaire-perime`),
diff résiduel nul vérifié ; chacune nomme le test qui doit rougir. Une mutation qui survit se déclare.

| # | Mutation | Doit rougir |
|---|---|---|
| Ma1 | retirer `mcp_keys` (puis `mcp_server_settings`) de `TABLES_TO_TRUNCATE` | `backup_inventory_matches_schema` |
| Ma2 | retirer `mcp_keys` de `TABLES_HORS_EXPORT` | garde de `exports/global.rs:637-690` |
| Ma3 | décodage de `McpScopes` qui **ignore** une portée inconnue | test unitaire « chaîne inconnue refusée » |
| Ma4 | `find_active_auth_by_key_hash` sans `revoked_at IS NULL` (puis sans le filtre d'expiration, puis sans `JOIN companies`) | tests de dépôt révoquée / expirée / orpheline |
| Ma5 | remettre `20261010000001` au registre de production | `registry_entries_are_within_import_window` |
| Ma6 | une colonne JSON de `mcp_server_settings` rebindée mal à l'import (p. ex. la liste vidée) | AC-a7 (comparaison colonne par colonne) — et **non** le saut de la table entière à l'export, qui rougit déjà par le refus d'import (`import.rs:236`) et le compte de 41 |
| Ma7 | `parse_and_verify` qui tolère une table absente du manifeste | AC-a15 (la fixture 0.14.0 s'importerait) |
| Mb1 | `authentifier` qui admet le préfixe `kesh_pat_` **et** cherche dans `api_keys` (une recherche de plus, seule, est équivalente — F6) | AC-b4 (c) |
| Mb2 | `require_auth` qui route `kesh_mcp_` vers l'authentification MCP | AC-b4 (a) |
| Mb3 | dernière utilisation écrite par la `Lecture` | AC-b5 |
| Mb4 | créateur rétrogradé accepté (contrôle de rôle retiré) | test « créateur non Administrateur » |
| Mb5 | créateur inactif accepté | test « créateur inactif » |
| Mb6 | contrôle du préfixe déplacé après la recherche (ou retiré) | AC-b3 (α), adossé au témoin (β) |
| Mc1 | route de création hors du bloc `admin_routes` (montée Comptable) | AC-c2 |
| Mc2 | secret présent dans la réponse de liste | test de liste |
| Mc3 | détail d'audit portant le condensat | AC-c5 |
| Mc4 | `version = 0` qui écrase une ligne existante | AC-c4 |
| Mc5 | réseau à bits d'hôte accepté (ou stocké tel quel) | AC-c4 |
| Mc6 | portée `books:write` acceptée | AC-c3 |
| Md1 | `mcp_keys` retirée de `RESET_PRESERVED_TABLES` | AC-d1 (test d'`onboarding_audit_e2e.rs`) |
| Md2 | la règle des orphelins ne révoque pas les clés MCP | AC-d2 (i) |
| Md3 | `is_empty` ignore les clés MCP | AC-d2 (ii) — l'orphelin MCP seul |
| Md4 | `repair_installation_in_tx` n'écrit pas `mcp_keys_revoked` au détail (`companies.rs:680-683`) | AC-d2 (ii) |

## Gates

- **32-5a et 32-5d : 🗄️ — gate complet même en cours de boucle** (`CLAUDE.md`, exception `kesh-db`) : base remise à
  zéro avant chaque gate complet (bases `kesh_325`/`kesh_e2e_325` propres au worktree, **sans** redémarrer le
  conteneur, `docs/agents-orchestration.md` § 2 et § 4), sous le verrou `flock …/.gate.lock`, par
  `scripts/test-fast.sh --ci` ; fmt, clippy `-D warnings`, build `--all-targets`.
- **32-5b et 32-5c** : gate ciblé entre les passes (`binary(…)` des tests touchés, fmt et clippy sur le workspace),
  complet au push.
- **E2E complet au dernier commit de code de chaque partie** (la migration démarre sur `kesh_e2e_325`, base
  persistante : c'est le seul contrôle de P8 en local) ; aucun écran ne change, les échecs attendus se jugent fichier
  par fichier contre `docs/testing.md` § *Les échecs attendus*.
- Frontend : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` en 32-5c (les quatre locales sont lues par le
  frontend).
- **Recette visuelle : sans objet** — aucun contrôle visible ne change (C-32-5-16) ; l'écran est la 32-7.
- Le Dev Agent Record **n'affirme que ce qui a tourné**, avec son périmètre (`CLAUDE.md` § *Recompter*).

## Ce que les gardes ne voient pas

- **La séparation `Lecture`/`Trace` en production.** En 32-5, les deux poignées enveloppent le même pool
  (`lib.rs:152-163`) : une dernière utilisation écrite par la `Lecture` réussirait en production. Seul le test d'AC-b5
  (une `Lecture` de test en `READ ONLY`) le voit, jusqu'à la `Lecture` de production de la 32-6.
- **Le refus d'un PAT par `/mcp`** n'est prouvé qu'au service (AC-b4 c) ; à l'endpoint, c'est la 32-6.
- **L'application de la politique de validation** : aucune garde de la 32-5 ne vérifie que la 32-6 lit
  `CleMcpAuthentifiee::validation` — c'est un AC de la 32-6 (répartiteur) et de la 32-8 (poignée).
- **Le sens des réglages** : la 32-5 valide la forme des plages, mandataires et origines ; qu'un filtre activé refuse
  réellement un pair public, que `X-Forwarded-For` ne soit lu que d'un mandataire déclaré, que `enabled = false` rende
  404, rien ici ne le prouve (32-6).
- **Les tests unitaires de `post_restore.rs` qui ne parcourent que le registre** (`:1272`, `:1308`, `:1365`, `:1417`,
  `:1500`) **retournent à vide** quand le registre se vide (état connu de la 25-2-c, commentaire `:216-224`) ; la
  machinerie reste éprouvée par `post_restore_class_a.rs` et par `replay_retired`.
- **L'ordre des verrous** `users → api_keys → mcp_keys` est raisonné, non testé (comme celui de la 15-7b3,
  `companies.rs:560-566`).
- **Les justifications d'exemption** ne sont contrôlées par le test que sur leur marqueur « Hors fenêtre » ; leur
  **texte** ne l'est que par la revue — d'où l'énumération d'AC-a9.
- **`backup_inventory_matches_schema`** prouve que la liste égale le schéma, pas qu'une colonne JSON fait l'aller-retour
  à l'identique : c'est AC-a7.
- **Le manuel** : aucun gate ne le lit ; les prompts de revue nomment l'axe et exigent le PDF aplati.
- **La limite de Q4** n'est éprouvée que contre **une** version antérieure (la fixture 0.14.0, AC-a15) ; le refus d'une
  sauvegarde 0.15.0 se déduit du même contrôle de couverture, sans fixture propre.
- **Les neuf appelants existants de `replay_retired`** rejoueront aussi le lettrage : leur innocuité est raisonnée sur
  leurs assertions, non prouvée avant le gate (AC-a10).

## Dev Notes

### Schéma (indicatif dans la forme, exigé dans le fond)

```sql
CREATE TABLE mcp_keys (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    created_by_user_id BIGINT NOT NULL COMMENT 'Créateur — la clé lit en son nom, tant qu''il est actif et Administrateur',
    name VARCHAR(255) NOT NULL,
    key_hash CHAR(64) NOT NULL COMMENT 'SHA-256(jeton) hex — jamais le secret',
    scopes JSON NOT NULL COMMENT 'Tableau trié, sans doublon : books:read, invoices:read, contacts:read, documents:read, prompts:write',
    send_validation VARCHAR(16) NOT NULL DEFAULT 'required' COMMENT 'required | exempt (Q12)',
    expires_at DATETIME(3) NULL,
    last_used_at DATETIME(3) NULL,
    revoked_at DATETIME(3) NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_mcp_keys_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_mcp_keys_created_by FOREIGN KEY (created_by_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT chk_mcp_keys_name_nonempty CHECK (CHAR_LENGTH(TRIM(name)) > 0),
    CONSTRAINT chk_mcp_keys_scopes CHECK (JSON_VALID(scopes) AND JSON_TYPE(scopes) = 'ARRAY' AND JSON_LENGTH(scopes) >= 1),
    CONSTRAINT chk_mcp_keys_send_validation CHECK (send_validation IN ('required', 'exempt')),
    CONSTRAINT uq_mcp_keys_key_hash UNIQUE (key_hash),
    INDEX idx_mcp_keys_company (company_id),
    INDEX idx_mcp_keys_created (created_at DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE mcp_server_settings (
    company_id BIGINT NOT NULL PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    address_filter_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    allowed_networks JSON NOT NULL,
    trusted_proxies JSON NOT NULL,
    allowed_origins JSON NOT NULL,
    version INT NOT NULL DEFAULT 1,
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_mcp_server_settings_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT chk_mcp_settings_networks CHECK (JSON_VALID(allowed_networks) AND JSON_TYPE(allowed_networks) = 'ARRAY'),
    CONSTRAINT chk_mcp_settings_proxies CHECK (JSON_VALID(trusted_proxies) AND JSON_TYPE(trusted_proxies) = 'ARRAY'),
    CONSTRAINT chk_mcp_settings_origins CHECK (JSON_VALID(allowed_origins) AND JSON_TYPE(allowed_origins) = 'ARRAY')
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

⚠️ `ENGINE=InnoDB` **explicite** sur les deux tables (le manuel relève deux tables sans moteur explicite,
`admin-manual.tex:2195`). ⚠️ `JSON_TYPE`/`JSON_LENGTH` dans un `CHECK` n'ont **aucun précédent** au dépôt (fait 14) :
à éprouver à T-a1 sur `kesh_325` ; à défaut, `JSON_VALID` seul et la règle tenue par Rust, écrit au Change Log.
⚠️ Lecture des colonnes `JSON` : `CAST(… AS CHAR)` (fait 14). ⚠️ Dès que la ligne de `migrations.sha384` est inscrite
(AC-a12), le fichier de migration **ne se modifie plus** (P8) ; une correction ultérieure va dans une migration
suivante.

### Esquisse des types (indicative)

```rust
// kesh-db/src/entities/mcp_key.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum McpScope { BooksRead, InvoicesRead, ContactsRead, DocumentsRead, PromptsWrite }
impl McpScope { pub const ALL: [McpScope; 5] = [/* … */]; }
pub struct McpScopes(BTreeSet<McpScope>);            // non vide ; encode/décode JSON strict
pub enum SendValidation { Required, Exempt }          // "required" | "exempt"

// kesh-api/src/services/cles_mcp.rs
pub const ROLE_CLE_MCP: Role = Role::Admin;           // hypothèse Q3 — AUTHENTIFICATION seulement ; la création
                                                      // est gardée par require_admin_role (rbac.rs:31-33, en dur)
pub struct CleMcpAuthentifiee { pub cle_id: i64, pub acces: ContexteAcces, pub validation: SendValidation }
pub async fn authentifier(lecture: &Lecture, trace: &Trace, jeton: &str) -> Result<CleMcpAuthentifiee, AppError>;
```

### Ce que reçoivent les stories suivantes

- **32-6** (**inscrit à sa section du fichier d'epic le 2026-10-25**, pour le refus d'un PAT et la règle du 404) :
  `services::cles_mcp::authentifier` à appeler depuis sa garde (ni PAT ni session) ; le refus d'un PAT à
  l'endpoint (AC à reprendre de l'AC-b4 c) ; `CleMcpAuthentifiee::validation` à appliquer au répartiteur (refus d'une
  clé `required` tant que la 32-8 n'est pas livrée) ; les réglages par `mcp_server_settings::get` + `defaults` (aucune
  écriture), **lus sur la `Lecture`**. ⚠️ **Point de conception laissé à la 32-6** : les réglages sont **par société**,
  alors que « désactivé → 404, indiscernable d'une route absente » se décide **avant** l'authentification — la fiche
  32-6 fixe la règle (par exemple : servi seulement si une société a `enabled = TRUE`, puis 404 identique si la société
  de la clé ne l'a pas). Le journal de chaque appel, l'acteur de la trace (`actor_type`, cf. #431, et la question
  laissée par la 32-3 sur l'acteur d'une clé MCP) sont à la 32-6.
- **32-7** — **inscrit à sa section du fichier d'epic le 2026-10-25** : l'écran sur les
  routes de la 32-5c ; au manuel, la section *Le serveur MCP* — création d'une clé, affichage unique, portées,
  politique de validation, ce que le filtre ne garantit pas, la clé donnée au client en portée `local`/`user` (spike
  § 6), la plage `100.64.0.0/10` d'un VPN à ajouter soi-même (C-32-5-10), le comportement d'une restauration
  « serveur activé » (Q7) — et les renvois depuis les trois passages que la 32-5d aura amendés (`admin-manual.tex:1414`,
  `:1416`, `:2461`).
- **32-8** : la poignée de la Q10 (c), sa table des envois (🗄️, nouvelle fermeture de la fenêtre de P7), la politique
  `exempt` (seule voie hors validation, chaque envoi journalisé).
- **32-10a** : `prompts:write` existe ; aucune table de prompts ici ; ses tables entrent au `.keshbackup` (#722).

### Fichiers touchés (prévus)

- **32-5a** : `crates/kesh-db/migrations/20261025000001_mcp_keys_and_server_settings.sql` (neuf) ;
  `crates/kesh-db/migrations.sha384` ; `crates/kesh-db/src/entities/{mcp_key.rs, mcp_server_settings.rs, mod.rs}` ;
  `crates/kesh-db/src/repositories/{mcp_keys.rs, mcp_server_settings.rs, mod.rs, letterings.rs (commentaire)}` ;
  `crates/kesh-db/src/backup.rs` ; `crates/kesh-db/src/post_restore.rs` ;
  `crates/kesh-db/test-schema/0001_schema_squash.sql` (régénéré) ;
  `crates/kesh-db/tests/{migrations_upgrade_path.rs, post_restore_class_a.rs (commentaire), mcp_keys_repository.rs
  (neuf)}` ; `crates/kesh-db/tests/{lettering_documents_backfill.rs, letterings_lexical.rs}` (commentaires) ;
  `crates/kesh-api/src/exports/global.rs` ; `crates/kesh-api/tests/{admin_full_export_e2e.rs,
  admin_backup_e2e.rs, admin_full_import_e2e.rs, onboarding_audit_e2e.rs}` ; `docs/migrations-idempotence-audit.md` ;
  `docs/manual/fr/{admin-manual,user-manual}.{tex,pdf}` ; `CHANGELOG.md`.
- **32-5b** : `crates/kesh-api/src/auth/api_key.rs` ; `crates/kesh-api/src/services/{acces.rs, cles_mcp.rs (neuf),
  mod.rs, poignees.rs}` ; `crates/kesh-api/tests/{services_motif.rs, cles_mcp_e2e.rs (neuf),
  admin_full_import_e2e.rs, common/mod.rs (pool_lecture_seule)}`.
- **32-5c** : `crates/kesh-api/src/routes/{mcp_keys.rs, mcp_server_settings.rs (neufs), mod.rs}` ;
  `crates/kesh-api/src/lib.rs` ; `crates/kesh-api/Cargo.toml`, `Cargo.lock` (`ipnet`) ;
  `crates/kesh-api/src/audit_labels.rs` ; `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ;
  `crates/kesh-api/tests/{admin_pat_denied_e2e.rs, audit_route_registry.rs, mcp_admin_routes_e2e.rs (neuf)}` ;
  `docs/api-external.md` ; `docs/manual/fr/admin-manual.{tex,pdf}`.
- **32-5d** : `crates/kesh-db/src/{backup.rs, repositories/companies.rs}` ;
  `crates/kesh-db/tests/{companies_repository.rs, support/installations_atteintes.rs}` ; `crates/kesh-seed/src/lib.rs` ;
  `crates/kesh-api/src/auth/bootstrap.rs` ; `crates/kesh-api/src/routes/{onboarding.rs, admin.rs (commentaire)}` ;
  `crates/kesh-api/tests/{onboarding_audit_e2e.rs, admin_full_import_e2e.rs (si une attente d'`installation.repaired`
  y lit les champs)}` ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md` ; `docs/manual/fr/{admin-manual,user-manual}.{tex,pdf}`.

### Risques

- **Élevé — surface de sécurité neuve** : un secret qui fuit (liste, journal, `Debug`), une clé qui ouvre l'API, un PAT
  qui ouvre le MCP. D'où le type sans `Serialize`, les refus croisés testés dans les deux sens, les mutations Mb1, Mb2,
  Mb6, Mc2, Mc3.
- **Moyen — la fenêtre de P7** : des tests et des textes qui dépendent d'un registre non vide (fait 10, fait 17) ; le
  motif de la 24-2 le règle mécaniquement, mais un test converti peut **passer à vide** — d'où l'exigence d'AC-a10.
- **Moyen — la règle des orphelins** : code de réparation partagé par quatre appelants ; un champ oublié au détail
  d'audit est un défaut **muet** (Md3, Md4).
- **Faible — le squash et le registre des checksums** : régénéré par script, ligne inscrite en dernier ; la garde les
  compare au vrai schéma.

### Références

`planning-artifacts/epic-32-serveur-mcp.md` (§ *32-5*, *Ce que le code dit déjà* 1 à 4, *Points d'attention*,
*Questions* et *Réponses*) ; `planning-artifacts/epic-32-spike-mcp.md` (§ 4.3, § 4.4, § 5.3, § 6, § 6.1, § 7) ;
`epic-32-choix-autonomes.md` (C-32-0-6 à 9, 12, 13) ; `epic-32-choix-autonomes-323.md` (C-32-3-4) ; fiche
`32-3-services-motif.md` (le motif) ; #722 ; `CLAUDE.md` § *Migration breaking policy* (P1 à P8), *Review Iteration
Rule*, *Règle de splitting préventif*, *Recette visuelle*, *Test Locally First* ; `docs/agents-orchestration.md`.

## Points à soumettre à Guy

1. **Q3 — qui crée une clé MCP ?** Hypothèse de travail : l'Administrateur seul, et une clé cesse d'authentifier dès
   que son créateur n'est plus Administrateur ou n'est plus actif (C-32-5-7). **Le vrai coût d'une ouverture au
   Comptable** : la constante `ROLE_CLE_MCP` ne règle que l'authentification ; la création est gardée par la couche
   d'`admin_routes` (`require_admin_role`, `Role::Admin` en dur, `rbac.rs:31-33`). Il faudrait donc sortir les **quatre
   routes des clés** du bloc (les deux routes des réglages restent à l'Administrateur), les remonter sous
   `require_comptable_role` avec `ensure_not_pat` (motif des clés d'API), changer la constante, et refaire les gardes et
   leurs compteurs (`admin_pat_denied_e2e` : 39 → 35, `(10, 29)` → `(9, 26)`, `discriminating` 8 → 7) — une modification de routage,
   non d'une ligne. Rien n'est branché d'avance tant que tu n'as pas tranché.
2. **Q7 — les réglages restaurés tels quels ?** Hypothèse : oui, un serveur activé dans la sauvegarde l'est après
   l'import (C-32-5-11). L'autre voie : désactivé à toute restauration, réactivé à la main.
3. **Q5 — les cinq portées** `books:read`, `invoices:read`, `contacts:read`, `documents:read`, `prompts:write` : toujours
   l'hypothèse du kickoff.
4. **Le découpage en quatre parties** (32-5a à 32-5d), une fiche et quatre revues, dans l'ordre a → b → d → c : le
   relevé au code et les validations ont trouvé **onze modules distincts** (dix-huit en somme des quatre parties)
   là où le kickoff en comptait trois (C-32-5-1).
5. **La remise à zéro de démonstration** conserve les clés MCP (comme les clés d'API) et remet le serveur à
   « désactivé » (C-32-5-11). Te convient-il ?
6. **La plage d'un VPN** (`100.64.0.0/10`, utilisée par Tailscale entre autres) n'est **pas** admise par défaut par le
   filtre d'adresse : quel VPN utilises-tu pour joindre le NAS ? S'il adresse ses clients dans cette plage, il faudra
   l'ajouter à l'écran (C-32-5-10).
7. **Pour information — une conséquence de Q4** : la 0.16.0 refuse les sauvegardes de la 0.15.0, et, du même coup, le
   rattrapage du lettrage rejoué à l'import (0.13.0) n'a plus d'objet et se retire (C-32-5-13). Rien n'est perdu pour
   une installation neuve ; le manuel dit la règle, la phrase sur une sauvegarde plus ancienne et le rappel de la sauvegarde
   complète (AC-a13 (1), C-32-5-28) ; il ne décrit aucune procédure détaillée de restauration *(« aucune marche » jusqu'à l'étape 0 de la 32-5a, P8-4)* : la 0.14.0 n'a pas de restauration sur machine neuve, et la 32-5 ne la pallie pas,
   puisque tu installeras à neuf. ⚠️ Si une installation existante (celle du NAS) était
   mise à jour en 0.16.0 au lieu d'une installation neuve, **toutes** ses sauvegardes antérieures, automatiques
   comprises, cesseraient de se restaurer sur la 0.16.0 : d'où la sauvegarde complète juste après la mise à jour, que le manuel et le CHANGELOG prescrivent (AC-a13 (1)).
   *(La prémisse « Kesh n'est pas en service » est la tienne ; son écart avec `CLAUDE.md`, qui dit les livres réels
   tenus sur le NAS, t'est déjà posé par l'orchestrateur — la fiche ne le tranche pas, F14.)*

## À recontrôler à l'étape 0 du développement

Constats de la validation P7 laissés sans remédiation, parce qu'ils demandaient plus qu'une annotation ou un mot :

- **P7-6 (LOW)** — dans la phrase prescrite de l'AC-a13 (1), « après la mise à jour vers la 0.16.0, prendre une
  sauvegarde complète juste après la mise à jour » répète « la mise à jour », et le « seulement » de la phrase sur une
  sauvegarde plus ancienne précède une liste que suit, dix lignes plus bas, un énoncé de plus. Au développement : écrire
  au manuel une phrase sans redite (p. ex. « après la mise à jour vers la 0.16.0, prendre sans attendre une sauvegarde
  complète »), et ajuster en conséquence le motif du contrôle aplati (« juste après la mise à jour »).

Constats de la validation P8 (dernière passe permise), **non remédiés à la fiche** — rien n'y serait relu. L'agent de
développement fait chaque correction **à l'étape 0**, et la déclare au Change Log, **ligne lue citée** :

- **P8-1 (MEDIUM)** — le grep de classe d'AC-a13 (5) : sa moitié **PDF** n'est pas restreinte aux deux § (185
  occurrences aujourd'hui), et le motif `égale` capte « légale », « légalement », « légales », « également ». Correction
  attendue : **retirer la moitié PDF** du grep de classe (le PDF reste contrôlé par la liste d'absence de l'AC-a13 :
  « égale ou plus récente », « 0.15.0 ou plus récente », « appliquera cette fois ») et écrire **`égale ou`** au lieu
  de `égale` aux trois sites du motif (AC-a13 (5) deux fois, AC-a14). Le grep ne porte alors que sur le `.tex`
  restreint aux deux §. *(L'AC-a13 (5) n'est pas modifié à la fiche : c'est ce point qui fait foi au développement.)*
- **P8-2 (LOW)** — « texte d'origine au commit `481116dd` » est faux : le texte d'origine de la précision P4 se lit à
  **`00475d19`**. Correction : remplacer `481116dd` par `00475d19` dans l'annotation P7 du fichier d'epic (puce Q4) ;
  dans l'entrée *Validation P7* de ce Change Log, une annotation datée, sans réécriture.
- **P8-3 (LOW)** — cette section n'était nommée par aucune tâche : renvoi ajouté à T-a6 (la seule écriture faite à la
  fiche après la P8, d'une ligne). À l'étape 0, constater que le renvoi suffit.
- **P8-4 (LOW)** — « cette marche est remplacée » (C-32-5-21) et « aucune marche de restauration » (point 7) décrivent
  plus absente qu'elle ne l'est la suite gardée par l'AC-a13 (1). Correction : au registre, « cette marche est
  **réduite** par C-32-5-28 à la phrase de l'AC-a13 (1) » ; au point 7, « il ne décrit aucune procédure détaillée de
  restauration ». Peut aussi rester tel quel (constat sans effet sur le code).
- **Hors périmètre, signalé par la P8** — `docs/manual/fr/user-manual.tex:1914` (« Une sauvegarde faite **avant** la
  0.15.0 n'en contient que la trace […] : restaurée sur une autre machine… ») : même famille que la classe d'AC-a13 (5),
  hors des deux § que vise le grep. À relire à l'étape 0 : la phrase doit rester vraie après la 32-5a (une version qui
  importe encore cette sauvegarde) ; sinon, la borner comme les mentions du format 1 du manuel d'administration.

## Dev Agent Record

### Agent Model Used

*(à remplir)*

### Étape 0 — recontrôle (ligne lue citée)

*(à remplir, par partie)*

### Completion Notes List

*(à remplir)*

### File List

*(à remplir)*

## Change Log

- **2026-10-25 — fiche rédigée** (Opus 5.5, en autonomie). Trois parties (C-32-5-1), 29 AC (A : 14, B : 8, C : 7),
  19 tâches, 20 mutations ; registre C-32-5-1 à C-32-5-21 ; prompt de validation P1 : `325-validate-prompt-p1.md`.
- **2026-10-25 — Validation P1** (passe complète, deux lentilles en contexte frais, **Sonnet** ×2 ; rapports
  `kesh-gate-logs/325-validate-p1-R.md` et `325-validate-p1-F.md`). **Trend : R 0 CRITICAL / 2 HIGH / 3 MEDIUM /
  7 LOW ; F 0 / 0 / 5 / 4.** Deux findings convergents (R2 = F1, R4 = F3), R8 = F8 ; prémisses de R1 et R2 revérifiées
  par l'orchestrateur. Remédiation (Opus 5.5) :
  - **R1 (HIGH)** — `published_migrations_keep_their_checksums` exige une ligne de `migrations.sha384` par migration :
    fait 16 et AC-a12 réécrits (ligne ajoutée dans la même PR, en dernier ; P8 dès lors) ; C-32-5-24.
  - **R2 = F1 (HIGH)** — `autre_portee` est un `match` exhaustif et `TOUTES` vaut `[Portee; 2]` : AC-b2 réécrit
    (`McpScope::ALL`, refus éprouvé avec les quatre autres portées), `services_motif.rs` aux fichiers de 32-5b ;
    C-32-5-4 révisé (quatorze occurrences sur onze lignes, F7).
  - **R3 (MEDIUM)** — test de la remise à zéro déplacé dans `onboarding_audit_e2e.rs` (AC-d1) ; C-32-5-23.
  - **R4 = F3 (MEDIUM)** — quatre consommateurs (`companies.rs:680-683` compris), tests à égalité JSON stricte
    (`onboarding_audit_e2e.rs`, `bootstrap.rs`) et montage `installations_atteintes.rs:38-41` nommés ; champs toujours
    présents (AC-d2) ; C-32-5-12 révisé.
  - **R5 (MEDIUM)** — les justifications de `20261010000002`, `20261009000001`, `20261011000001`, `20261015000001`,
    `20260917000001` et quatre commentaires énumérés à réécrire (AC-a9, fait 10).
  - **F2 (MEDIUM)** — `ac10_vi` 5 → 6 (AC-b2, AC-b3).
  - **F4 (MEDIUM)** — choix : **la partie qui change le comportement corrige le manuel** (AC-d3, `:1414`, `:1416`,
    `:2461`) ; la 32-7 y renvoie ; C-32-5-22.
  - **F5 (MEDIUM)** — cas « la clé MCP est le seul orphelin » (AC-d2 (ii)), qui tue Md3 et Md4.
  - **Recompte des modules** : l'ancienne partie b passait à six (F3, F4, R10) → scindée en **32-5b** (3) et **32-5d**
    (5) ; a 4, c 4 ; C-32-5-1 révisé.
  - **LOW traités** : R6 (`CAST(… AS CHAR)`), R7 = C-32-5-19 révisé (adresse d'abord, bits d'hôte refusés,
    `Cargo.lock`), R8 = F8 (test de restauration dans `admin_full_import_e2e.rs`, `DELETE` direct), R9 = F6 (« dix » →
    onze, 33 → 39, 124/127 → 128/131, doc de `backup.rs`, commentaires du registre, AC-a14 étendu), R10 (`MULTI-
    TENANT-SCOPING-PATTERNS.md` et `admin.rs` en 32-5d, `user-manual.tex:3027` en 32-5a, `### Modifié` à créer, points
    de suite de la 32-7), R11 (test au pool injoignable, Mb6), R12 (citations), F6, F7, F9 (Ma6 redéfinie, témoin 1792
    sur l'`UPDATE` de `mcp_keys`, avertissement « secret » daté, complément du `grep` d'AC-a1).
  - **#722** (lu par l'orchestrateur) : cohérence vérifiée et écrite (§ *Origine*, AC-a13 (4), C-32-5-25).
  - **Symptômes grepés** sur le dépôt après remédiation : `39` (tables), `dix ci-dessous`, `\b33\b` (bloc d'admin),
    `124`/`127`, `api_keys_revoked`, `users → api_keys`, `POST_RESTORE_BACKFILLS`, `20261010000001`, `kesh_pat_`
    (« toujours »), `migrations.sha384` — chaque site atteint est nommé à un AC ou à *Fichiers touchés*.
  - Nouveaux totaux : **30 AC** (A 14, B 6, C 7, D 3), **23 tâches** (8 / 5 / 5 / 5), **22 mutations** (Ma 6, Mb 6,
    Mc 6, Md 4) ; registre C-32-5-1 à C-32-5-25. Prochaine passe : P2 complète, Opus, R et F
    (`325-validate-prompt-p2.md`).
- **2026-10-25 — Validation P2** (passe complète, deux lentilles en contexte frais, **Opus** ×2 ; rapports
  `kesh-gate-logs/325-validate-p2-R.md` et `325-validate-p2-F.md`). **Trend : P1 R 0/2/3/7, F 0/0/5/4 (2 HIGH,
  8 MEDIUM) → P2 R 0/0/3/7, F 0/1/4/10 (1 HIGH, 7 MEDIUM).** Convergents : R4 = F4, R7 = F7, R10 = F9, R5 = F10.
  Prémisses de F1 et F2 revérifiées par l'orchestrateur. Remédiation (Opus 5.5) :
  - **F1 (HIGH)** — la procédure de restauration (`admin-manual.tex:2373`, `:2376`) et `:1505` deviennent fausses dès la
    32-5a : AC-a13 (5), AC-a14 étendu.
  - **F2** — `ac10_la_sauvegarde_de_la_0_14_0_s_importe` devient le test du refus (AC-a15 neuf, mutation Ma7) ;
    C-32-5-26.
  - **F3** — `EMPTY_BEFORE_RESET` : `populate` peuple les deux tables (AC-a6), `onboarding_audit_e2e.rs` entre en 32-5a,
    qui passe à **5** modules ; en 32-5d, `mcp_keys` rejoint les instantanés des tables conservées (AC-d1).
  - **F4 = R4** — § *Reprises* : « aucune reprise **du registre** », puce #528 gardée et précisée, recalcul des
    numéros de client nommé, `keshwarning` `:2326-2330` réécrit (AC-a13 (2)).
  - **F5** — une erreur de base se propage, jamais 401 ; deux cas au port fermé `127.0.0.1:1` (α, β) ; `stub_pool`
    écarté, il est joignable (AC-b3, Mb6).
  - **R1** — sixième test (`full_import_replays_backfills_in_increasing_version_order`), neuf appelants de
    `replay_retired`, quatre commentaires de plus ; l'en-tête de `20261010000002` laissé intact (P8), dit dans sa
    justification (fait 10, AC-a9, AC-a10, AC-a14).
  - **R2** — *(annoté en P6 : forme remplacée par C-32-5-28)* la marche à suivre part d'une base vide (AC-a13 (1), CHANGELOG) ; la même phrase du fichier d'epic (§
    *Réponses de Guy*, Q4) **annotée** au 2026-10-25, la décision inchangée.
  - **R3** — manuel utilisateur `:190`, `:3202`, `:3209-3212` en 32-5d (AC-d3).
  - **LOW** : R5 = F10 (compteurs `:770-771`, `:1096-1129`), R6 (garde `:1178`, formulation proposée), R7 = F7 (trois
    occurrences hors de `services_motif.rs`, `:1201`, `:1220`), R8 (cookie `:83-87`, deux requêtes ; messages de P6),
    R9 (`_kesh_backup_schedule` ajouté à « reste hors du fichier »), R10 = F9 (**ordre a → b → d → c imposé**,
    C-32-5-27), F6 (Mb1 redéfinie), F8 (dix modules distincts, dix-sept en somme), F11, F12 (règle sans date, vraie
    depuis la v0.12.0 : `git tag --contains` des commits qui créent `invoice_settlements` et
    `journal_entry_number_sequences` → `v0.12.0`), F13 (`poignees.rs:9-12` en 32-5b), F15 (`pool_lecture_seule` dans
    `tests/common/`), F14 (point 7 à Guy complété, la contradiction avec `CLAUDE.md` non tranchée). Non traités : R4
    point « réparation #528 rendue sans objet » au-delà de la précision écrite ; aucun autre.
  - **Reports inscrits au fichier d'epic** : section 32-6 (refus d'un PAT à `/mcp`, règle du 404 face aux réglages par
    société) et section 32-7 (section *Le serveur MCP* du manuel, plage `100.64.0.0/10`, restauration d'un serveur
    activé, renvois aux passages corrigés en 32-5d).
  - ⚠️ **Recyclage déclaré** (`CLAUDE.md`, *Règle de splitting préventif*, amendement D5). Cinq findings naissent de la
    remédiation P1 ou reprennent un de ses défauts sous une autre forme : **R3** (F4 de la P1 sous une autre forme — le
    même oubli, dans le manuel utilisateur), **F5** (le test au pool injoignable, écrit pour R11, pouvait passer à
    vide), **F7 = R7** (la correction du décompte F7 l'a mal reprise dans l'AC-b2), **F8** (« douze modules »,
    écrit par la remédiation), **R10 = F9** (la phrase d'ordre, née du redécoupage). Les autres — F1, F2, F3, F4, R1,
    R2 — sont des défauts **d'origine**, distincts de ceux de la P1. La sévérité baisse (2 HIGH / 8 MEDIUM → 1 / 7)
    sans que la boucle tourne en rond ; le signal est déclaré au Project Lead, le découpage n'est pas relancé (aucune
    partie ne dépasse cinq modules après recompte).
  - **Symptômes grepés par la valeur** sur le dépôt après remédiation : `égale ou plus récente`, `format 1`,
    `appliquera cette fois`, `rejoue donc ces reprises`, `registre de rejeu`, `au registre (classe A)`, `rejouée à
    l'import`, `EMPTY_BEFORE_RESET`, `\b118\b`/`\b127\b`/`\b124\b` (`audit_route_registry.rs`), `stub_pool`,
    `quatorze`, `douze`, `installer l'image de la version` — chaque site atteint est nommé à un AC, au fait 10 ou au
    fait 17, ou déclaré exempté (en-tête P8).
  - Nouveaux totaux : **31 AC** (A 15, B 6, C 7, D 3), **23 tâches**, **23 mutations** (Ma 7, Mb 6, Mc 6, Md 4) ;
    registre C-32-5-1 à C-32-5-27. Prochaine passe : P3 complète, Sonnet, R et F (`325-validate-prompt-p3.md`).
- **2026-10-25 — Validation P3** (passe complète, deux lentilles en contexte frais, **Sonnet** ×2 ; rapports
  `kesh-gate-logs/325-validate-p3-R.md` et `325-validate-p3-F.md`). **Trend : P1 2 HIGH / 8 MEDIUM → P2 1 / 7 → P3
  R 0/0/0/7, F 0/0/1/9 (0 HIGH, 1 MEDIUM).** Convergents : F2 = R5, F5 = R2, F8 = R4, F9 (i) = R7. Prémisse de F1
  revérifiée par l'orchestrateur. Remédiation (Opus 5.5), **sur la fiche seule** (aucun code) :
  - **F1 (MEDIUM)** — `ROLE_CLE_MCP` ne pilote que l'authentification ; la création est gardée par `require_admin_role`
    (`rbac.rs:31-33`, `Role::Admin` en dur). C-32-5-7, l'esquisse des Dev Notes et le point 1 à Guy disent le vrai coût
    d'une ouverture au Comptable (une modification de routage) ; la constante n'est **pas** branchée sur le handler.
  - **LOW** : R1 (jambes `read-write`/`read-only`, `(8, 25)` → `(10, 29)`, `discriminating` 6 → 8, les `GET` sans
    `ensure_not_pat`) ; R2 = F5 (P6 : chaque occurrence de `\b49\b`/`\b83\b`, `:208`) ; R3 (sept « n'est PAS vide ») ;
    R4 = F8 (lignes `:99`/`:100` de l'audit d'idempotence, ligne neuve qui dit le déplacement, `:89` signalée) ; F2 = R5
    (quatre sites antérieurs à Q11 : `poignees.rs`, `services/mod.rs` en 32-5b, `lib.rs` ×2 en 32-5c) ; R6
    (`ContexteAcces`) ; R7 = F9 (i) (citations `:83`, `:208`, `format_1_restauree_sur_la_meme_installation`, `:2013`,
    `:2024`) ; F3 (squash et `migrations.sha384` en fin de T-a2, après les tests des `CHECK`) ; F4 (`:211-224`,
    `ExemptionBasis::Durable`) ; F6 (versions sans l'assistant) ; F7 (comptes de la 32-5b relatifs à `main` au départ,
    recomptés à T-b0 — la 32-4a, PR #745, la précédera) ; F9 (ii) (`mod common;` à `services_motif.rs`), (iii)
    (`middleware` compté : 32-5b à **4** modules, **11 distincts, 18 en somme**) ; F10 (annotations du registre de
    sprint rafraîchies).
  - **Recyclage** : F2 et F5 sont des propagations incomplètes de correctifs P2 (F13 et R8 de la P2) ; F1 est un défaut
    **d'origine** (C-32-5-7, première rédaction). Sévérité en baisse continue.
  - **Symptômes grepés par la valeur** sur la fiche, le registre, le fichier d'epic et le registre de sprint :
    `création ET`, `une constante change`, `seule source de deux`, `auth.rs:77-79`, `:209`, `en_format_1`, `:2012`,
    `:2023`, `:211-216`, `:950-956`, `dix modules distincts`/`dix-sept` — restent seulement les mentions datées des
    entrées P1 et P2 de ce Change Log. Au dépôt : `utilisateur .?SELECT` (quatre sites, tous nommés), `n'est PAS vide`
    (sept, nommés), `\b(83|49)\b` dans `migrations_upgrade_path.rs` (règle de la valeur).
  - Totaux inchangés : 31 AC, 23 tâches, 23 mutations ; registre C-32-5-1 à 27. La remédiation ne touchant aucune ligne
    de code, la passe suivante est **ciblée** (P4, une lentille regression hunter, Opus, sur le seul commit de cette
    remédiation, `325-validate-prompt-p4.md`).
- **2026-10-25 — Validation P4** (passe **ciblée**, une lentille regression hunter, contexte frais, **Opus**, sur le
  seul commit `bafe2c9b` ; rapport `kesh-gate-logs/325-validate-p4.md`). **Trend : P1 2 HIGH / 8 MEDIUM → P2 1 / 7 →
  P3 0 / 1 → P4 0 / 0 / 1 MEDIUM / 6 LOW.** Remédiation (Opus 5.5), **sur la fiche, le registre et le fichier d'epic
  seuls** :
  - **H1 (MEDIUM) — retrait** (décision de l'orchestrateur, C-32-5-28 ; *annoté en P6 : la forme décrite ici a été réduite en P5 puis en P6*). ⚠️ **Recyclage déclaré** : c'est la
    **troisième** forme du même défaut, chacune née du correctif précédent — R2 en P2, F6 en P3, H1 en P4 — sur la
    procédure de restauration d'une sauvegarde ancienne. AC-a13 (1) ne prescrit plus de chemin pour une version sans
    l'assistant ; le manuel dit trois choses : mêmes tables ; l'image de la version qui a produit la sauvegarde, sur une
    base vide, et la procédure du **manuel de cette version** publié à son tag ; puis la mise à jour. Le lieu se dit en
    mots : un gabarit `raw.githubusercontent.com/guycorbaz/kesh/v<version>/…` ferait rougir G19
    (`textes_coherents.rs:1250-1285`). Point 7 à Guy et annotation du fichier d'epic (puce Q4, précision datée)
    alignés.
  - **LOW** : H2 (onze modules distincts, `middleware` dans la liste, italique refermée) ; H3 (annotation de
    `32-5b` au registre de sprint : 4 modules) ; H4 (doc-comment `:795-797` et généalogie `:831-833`
    d'`admin_pat_denied_e2e.rs`) ; H5 (`acces.rs:57-58`) ; H6 (la ligne `:100` de l'audit d'idempotence ne dit pas
    « retirée du registre » d'une migration qui n'y fut jamais ; « 32-5a ») ; H7 (quatre routes des clés, les
    réglages restant à l'Administrateur : 39 → 35, `(10, 29)` → `(9, 26)`, `discriminating` 8 → 7 — point 1 à Guy et
    C-32-5-7).
  - **Valeurs grepées** sur la fiche, le registre, le fichier d'epic et le registre de sprint :
    `administrateur d'amorçage`, `n'a pas l'assistant`, `sortir les six`, `39 → 33`, `acces.rs:~72`, `la même pour
    son`, `3 modules, sans`, `11 distincts***` → plus aucune occurrence hors des entrées datées du Change Log ;
    `raw.githubusercontent.com/guycorbaz/kesh/v<` → une seule, au registre (C-32-5-28, l'exemple écarté ; hors des
    fichiers que G19 lit).
    *(Annoté en validation P5, P5-3 : faux à sa date — trois occurrences, une au registre et deux dans ce Change Log,
    l'exemple écarté et la liste du grep elle-même ; toutes hors des fichiers que G19 lit.)*
  - Totaux inchangés : 31 AC, 23 tâches, 23 mutations ; registre C-32-5-1 à C-32-5-28. Remédiation sans code :
    passe suivante **ciblée** (P5, regression hunter, Sonnet, `325-validate-prompt-p5.md`).
- **2026-10-25 — Validation P5** (passe **ciblée**, regression hunter, contexte frais, **Sonnet**, sur `00475d19` ;
  rapport `kesh-gate-logs/325-validate-p5.md`). **Trend : P1 2 HIGH / 8 MEDIUM → P2 1 / 7 → P3 0 / 1 → P4 0 / 1 → P5
  0 / 0 / 1 MEDIUM / 4 LOW.** Remédiation (Opus 5.5), sur la fiche, le registre, le fichier d'epic et le registre de
  sprint seuls :
  - **P5-1 (MEDIUM)** et **P5-2** — *(annoté en P6 : la phrase décrite ici ne vit plus qu'à l'AC-a13 (1))* la phrase promettait « le PDF joint à la release GitHub » : `release.yml:161-165`
    n'attache aucun fichier (l'erreur venait de la consigne de la P4, vérifié par l'orchestrateur). Réduction à ce qui
    est vérifiable, sans procédure : mêmes tables ; une sauvegarde plus ancienne se restaure avec la version qui l'a
    produite, dont le manuel est dans le dépôt au tag de cette version ; ensuite, on met à jour. Plus de « base vide »
    ni de section d'un manuel ancien (la 0.14.0 n'a pas de restauration sur machine neuve ; Kesh n'est pas en service,
    Guy installera à neuf). AC-a13 (1), C-32-5-28, précision du fichier d'epic et point 7 à Guy alignés ; la phrase
    fausse `admin-manual.tex:2430` est tracée en **#746**, non corrigée ici. ⚠️ **Recyclage** : quatrième passe de
    suite sur la même phrase (R2, F6, H1, P5-1) ; le retrait la réduit désormais à trois énoncés vérifiables.
  - **LOW** : P5-3 (le décompte du grep déclaré à l'entrée P4 était faux : annoté à sa place, daté) ; P5-4 (ligne de la
    clé `32-5` du registre de sprint : état après P5) ; P5-5 (« trois choses, et rien de plus » comptait une redite et un
    quatrième énoncé : la règle n'est plus répétée, le rappel de sauvegarde n'est plus prescrit à cet endroit). *(Annoté en P6, P6-2 :
    faux — le rappel n'était prescrit nulle part ailleurs ; il est rétabli en P6 comme phrase distincte.)*
  - ⚠️ **Le prompt de la P5 désignait le commit par `git log --grep "validation P4"`** : sur une branche qui contient
    l'historique de `main`, ce motif peut retrouver un commit de `main` plutôt que celui de la remédiation. Le prompt de
    la P6 désigne le commit **par son SHA**, en dur.
  - **Valeurs grepées** sur la fiche, le registre, le fichier d'epic et le registre de sprint : `joint à la release`,
    `PDF de la release`, `base vide` (AC-a13 et point 7), `rien de plus`, `manuel de cette version` → plus aucune
    prescription vivante ; restent les entrées datées du Change Log (P2 à P4) et l'historique du choix C-32-5-21 et du
    contexte de C-32-5-28.
  - Totaux inchangés : 31 AC, 23 tâches, 23 mutations ; registre C-32-5-1 à C-32-5-28. Passes : **5 sur 8**. Suivante :
    P6 ciblée, regression hunter, Opus (`325-validate-prompt-p6.md`).
- **2026-10-25 — Validation P6** (passe **ciblée**, regression hunter, contexte frais, **Opus**, sur `b7505390` ; rapport
  `kesh-gate-logs/325-validate-p6.md`). **Trend : … → P4 0 / 1 → P5 0 / 1 → P6 0 / 0 / 2 MEDIUM / 1 LOW.** Passes :
  **6 sur 8**. Remédiation (Opus 5.5), sur la fiche, le registre, le fichier d'epic et le registre de sprint seuls :
  - ⚠️ **Recyclage déclaré — cinquième forme** du défaut de la restauration d'une sauvegarde ancienne (R2, F6, H1, P5-1,
    P6-1) : l'AC-a13 (5) prescrivait encore, dans la procédure de l'assistant, « en pratique la version qui l'a
    produite, puis la mise à jour ».
  - **P6-1 (MEDIUM), par retrait** : l'étape 1 (`admin-manual.tex:2376`) ne nomme plus aucune version ; elle se réduit
    à la règle des mêmes tables et renvoie à *Compatibilité*. **Seule l'AC-a13 (1) porte désormais la phrase** sur une
    sauvegarde plus ancienne : le point 7, C-32-5-28, C-32-5-21 et la précision du fichier d'epic y renvoient au lieu de
    la répéter. *(Annoté en P7, P7-3 : inexact pour C-32-5-21, **annoté** et non réduit — son ancienne marche reste en
    clair, sous l'annotation déplacée après elle ; et la précision du fichier d'epic a été réécrite au lieu d'être
    annotée, P7-4.)*
  - **P6-2 (MEDIUM), par rétablissement** : « prendre une sauvegarde complète juste après la mise à jour » redevient une
    phrase distincte de *Compatibilité* et du CHANGELOG, contrôlée au PDF aplati ; le point 7 y renvoie ; l'entrée P5 qui
    le disait « plus prescrit à cet endroit » est annotée.
  - **P6-3 (LOW)** : les deux occurrences datées de « base vide » hors de la fiche (puce Q4 du fichier d'epic,
    annotation P2 ; annotation P2 de `32-5a` au registre de sprint) sont annotées « forme remplacée par C-32-5-28 ».
  - **Valeurs grepées** sur la fiche, le registre, le fichier d'epic et le registre de sprint : `version qui l'a
    produite`, `puis la mise à jour`, `installer l'image`, `en pratique la version`, `base vide` — la seule prescription
    vivante est l'AC-a13 (1) ; les autres occurrences sont des entrées datées, annotées, ou la décision d'origine de Guy
    (puce Q4, suivie de ses précisions datées).
  - Totaux inchangés : 31 AC, 23 tâches, 23 mutations ; registre C-32-5-1 à C-32-5-28. Suivante : P7 ciblée, Sonnet
    (`325-validate-prompt-p7.md`), qui relit aussi **toute la fiche** à la recherche d'une prescription restante.
- **2026-10-25 — Validation P7** (passe **ciblée**, regression hunter, contexte frais, **Sonnet**, sur `2e0ea04c`, avec un
  axe sur toute la fiche ; rapport `kesh-gate-logs/325-validate-p7.md`). **Trend : P1 2 HIGH / 8 MEDIUM → P2 1 / 7 →
  P3 0 / 1 → P4 0 / 1 → P5 0 / 1 → P6 0 / 2 → P7 0 / 0 / 1 MEDIUM / 5 LOW.** Passes : **7 sur 8** ; la P8 est la
  dernière permise. Remédiation (Opus 5.5), sur la fiche, le registre, le fichier d'epic et le registre de sprint seuls :
  - **P7-1 (MEDIUM), traité par classe** : la sixième forme du même défaut (« format 2 (0.15.0 ou plus récente) », à
    `admin-manual.tex:2373`, que la 0.16.0 rend faux) n'est pas corrigée ligne à ligne. L'AC-a13 (5) prescrit de relire
    en entier *Restaurer sur une nouvelle machine* et *Compatibilité*, d'y réécrire toute mention de format ou de
    version pour qu'elle reste vraie après la 32-5a, et de fermer la liste **par un grep** (`format [12]|0\.15\.0|ou plus
    récente|égale`, sur le `.tex` et sur le PDF aplati, sorties citées au Dev Agent Record). `:2373` est nommée comme
    premier site connu, sans prétendre la liste close. L'AC-a14 et le contrôle du PDF reçoivent ces motifs.
  - **LOW traités par un mot ou une annotation** : P7-2 (point 7 à Guy : la règle, la phrase et le rappel ; aucune marche
    de restauration) ; P7-3 (annotation de C-32-5-21 déplacée après la parenthèse qu'elle annote ; l'entrée P6 annotée :
    C-32-5-21 est annoté, non réduit) ; P7-4 (la précision datée P4 du fichier d'epic, réécrite en P5 et P6, **annotée**
    sans être réécrite de nouveau ; texte d'origine au commit `481116dd`) ; P7-5 (en-tête : « P1 à P7 »).
  - **Reporté** : P7-6 (redite dans la phrase prescrite) → section *À recontrôler à l'étape 0 du développement*.
  - **Valeurs grepées** sur la fiche, le registre, le fichier d'epic et le registre de sprint : `0.15.0 ou plus récente`
    (désormais au seul titre de motif à greper : AC-a13 (5), contrôle du PDF de l'AC-a13, AC-a14), `en une seule phrase`, `P1 à P4`, `ne prescrit aucune
    procédure` (point 7) → plus aucune occurrence vivante.
  - Totaux inchangés : 31 AC, 23 tâches, 23 mutations ; registre C-32-5-1 à C-32-5-28. Suivante : **P8, dernière passe
    permise**, ciblée, Opus (`325-validate-prompt-p8.md`).
- **2026-10-25 — Validation P8 — VALIDATION TERMINÉE AU PLAFOND** (passe **ciblée**, regression hunter, contexte frais,
  **Opus**, sur `3a57bbce` ; rapport `kesh-gate-logs/325-validate-p8.md` : **0 / 0 / 1 MEDIUM / 3 LOW**).
  - **Trend complet** : P1 complète (Sonnet ×2) R 0/2/3/7, F 0/0/5/4 → P2 complète (Opus ×2) R 0/0/3/7, F 0/1/4/10 → P3
    complète (Sonnet ×2) R 0/0/0/7, F 0/0/1/9 → P4 ciblée (Opus) 0/0/1/6 → P5 ciblée (Sonnet) 0/0/1/4 → P6 ciblée
    (Opus) 0/0/2/1 → P7 ciblée (Sonnet) 0/0/1/5 → P8 ciblée (Opus) 0/0/1/3. Soit 2 HIGH / 8 MEDIUM en P1, plus aucun
    HIGH depuis la P3.
  - ⚠️ **Recyclage déclaré** : la phrase sur la restauration d'une sauvegarde ancienne a pris **six formes** (R2 en P2,
    F6 en P3, H1 en P4, P5-1 en P5, P6-1 en P6, P7-1 en P7), chacune née du correctif précédent ; elle a été traitée par
    **retraits** successifs (C-32-5-28 : plus de chemin pour une version sans assistant, plus de fichier de release, plus
    de version nommée à l'étape 1, puis traitement par classe) ; le MEDIUM de la P8 (P8-1) porte sur le contrôle de cette
    même classe.
  - **Clôture au plafond**, selon la règle de Guy du 2026-10-24 (`CLAUDE.md`, *Review Iteration Rule*) : la P8 est la
    huitième et dernière passe permise ; **aucune remédiation de fond** n'est écrite après elle — la seule écriture à la
    fiche est le renvoi d'une ligne de T-a6 (P8-3) ; P8-1 à P8-4 et `user-manual.tex:1914` sont **renvoyés à l'étape 0
    du développement** (§ *À recontrôler à l'étape 0 du développement*), où l'agent les corrige et les déclare au
    Change Log, ligne lue citée. Depuis la sixième passe, les remédiations ont été des retraits ou des réductions
    décidés par l'orchestrateur.
  - Totaux : 31 AC, 23 tâches, 23 mutations ; registre C-32-5-1 à C-32-5-28. La fiche est prête pour le développement
    des quatre parties, dans l'ordre a → b → d → c.

