# Story 38-1 : coffre des secrets et connexions de prestataire

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`, numéros de ligne relevés au commit `8b3d548d`. Epic 38
(`_bmad-output/planning-artifacts/epic-38-paypal.md`) : décisions E1, E2, E3, E4, E18, E24 ; contrats **C-38-1**,
**C-38-2**, C-38-11, C-38-12, C-38-13 ; décisions de Beat B11, B14, B22, B23 (`epics-38-40-cadrage.md`), qui priment.
**Dépend de** : l'Epic 37 livré (liste close des absences admises de 37-1 AC-6, R1 ; numéros de migration postérieurs à
`20261114000003`). **Suivie par** : 38-2 (comptes prestataires, première table enfant d'une connexion), 38-3
(passages, colonnes `last_run_*`), 38-7 (« Tester la connexion » réel, `accountEmail`, `payerId`), 38-9a (écrans).
**Migration non breaking** (DDL seul, deux tables neuves). Aucun écran : l'interface est en 38-9a.*

## Story

En tant qu'**Administrateur** d'une société qui encaisse par PayPal, je veux **créer une connexion de prestataire**
(prestataire, libellé, date « télécharger depuis », téléchargement quotidien ou à la demande, réglages propres au
prestataire) et **y déposer ses secrets**, chiffrés en base par une clé que seul le serveur connaît et jamais
réaffichés, afin que Kesh puisse plus tard télécharger seul les mouvements de ce compte sans qu'une sauvegarde, un
export, un journal ou une réponse d'API n'emporte jamais l'accès au compte de paiement.

## Exigences

Reprises de la fiche d'epic (§ *38-1*), complétées là où un AC en ajoute (marquées *ajout*).

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-1-1` | Un Administrateur crée, lit, modifie et supprime une connexion : prestataire, libellé, date « télécharger depuis », réglage `daily`\|`manual`, réglages du prestataire (`settings`). | AC-3, AC-4, AC-5 |
| `FR-38-1-2` | Il dépose ou remplace les secrets de la connexion ; la réponse dit seulement `secretSet` et `secretUpdatedAt`. | AC-6 |
| `FR-38-1-3` | Les réglages sont validés par la struct du prestataire avant écriture (`PROVIDER_SETTINGS_INVALID`) ; un prestataire inconnu se liste sans être utilisable (`PROVIDER_UNKNOWN`). | AC-2, AC-4 |
| `FR-38-1-4` | La suppression d'une connexion est refusée tant qu'un compte prestataire y est rattaché (`PROVIDER_CONNECTION_IN_USE`). | AC-5 |
| `FR-38-1-5` | Une restauration complète efface les secrets ; la connexion passe « secret à saisir ». | AC-8 |
| `FR-38-1-6` *(ajout)* | La remise à zéro de l'installation vide les connexions et efface leurs secrets. | AC-8 |
| `FR-38-1-7` *(ajout)* | « Tester la connexion » (`POST …/test`) rend l'état local de la connexion (réglages lisibles, clé présente, secret déchiffrable) ; l'appel au prestataire s'y ajoute en 38-3/38-7. | AC-7 |
| `NFR-38-1-1` | Secrets chiffrés AES-256-GCM (nonce aléatoire, AAD, version de clé) par `KESH_SECRETS_KEY` ; variable absente : Kesh démarre, connexions inutilisables (`PROVIDER_SECRETS_KEY_MISSING`, 409) ; gabarit refusé. | AC-1, AC-6 |
| `NFR-38-1-2` | Aucun secret dans les journaux, erreurs, réponses, audit, exports ni sauvegarde (test qui cherche la valeur partout). | AC-9 |
| `NFR-38-1-3` | Routes des connexions et des secrets réservées à une session d'Administrateur (`require_not_pat`). | AC-3 |
| `NFR-38-1-4` | Migration non breaking ; `settings` sous `CHECK (JSON_VALID … JSON_TYPE = 'OBJECT')` ; tables à `backup.rs` et à la liste close R1. | AC-2, AC-8 |
| `NFR-38-1-5` *(ajout)* | Une clé changée ou un chiffré altéré se signale (`PROVIDER_SECRET_UNREADABLE`), jamais par une panique ni un secret vide. | AC-1, AC-7 |
| `NFR-38-1-6` *(ajout)* | La variable est transmise par les deux compose, décrite au `.env.example` et au manuel d'administration (garde `configuration_transmise`). | AC-10 |

## Critères d'acceptation

1. **AC-1 — Clé `KESH_SECRETS_KEY` et module `kesh_api::secrets` (C-38-1).**
   - **Lecture** dans `Config::from_env` (`crates/kesh-api/src/config.rs:590`), par `env_nonempty` comme les autres
     secrets (`:670`). Forme : **32 octets codés en base64 standard** (avec ou sans `=` final, espaces de bord
     retirés par `env_nonempty`). Trois issues, testées :
     - **absente ou vide** → `Config.secrets_key = None`, démarrage normal, **un** `tracing::warn!` sans valeur
       (« KESH_SECRETS_KEY absente : les connexions de prestataire sont inutilisables ») ;
     - **gabarit** (`is_template_placeholder`, `config.rs:1538`, appelé comme pour `KESH_JWT_SECRET`, `:683`) →
       `ConfigError::InsecureSecretsKey`, refus de démarrer, `Display` sans la valeur (patron
       `InsecureJwtSecret`, `config.rs:41`) ;
     - **illisible** (base64 invalide, ou longueur décodée ≠ 32) → `ConfigError::InvalidSecretsKey { decoded_len:
       Option<usize> }`, refus de démarrer, sans la valeur.
     Refuser plutôt que d'ignorer une clé mal formée : une clé tronquée chiffrerait sans erreur puis ne déchiffrerait
     plus rien après correction.
   - **Type** `SecretsKey` (dans `kesh_api::secrets`) : enveloppe des 32 octets, `Debug` qui n'affiche que
     `SecretsKey(<masquée>, empreinte=…)`, effacement à la libération (`zeroize`), jamais `Serialize`. L'**empreinte**
     de la clé = 16 premiers caractères hexadécimaux de SHA-256(clé) (`sha2`, déjà dépendance, `Cargo.toml:55`) : elle
     ne révèle rien d'utile et permet de dire « la clé a changé » plutôt que « secret corrompu ». `Config` porte
     `secrets_key: Option<Arc<SecretsKey>>` ; son `Debug` reste masqué (patron du module, `config.rs:1-5`), test.
   - **`seal(key, aad, plaintext) -> Sealed { nonce: [u8; 12], ciphertext: Vec<u8> }`** et **`open(key, aad, &Sealed)
     -> Result<Zeroizing<String>, SecretError>`** : AES-256-GCM (crate `aes-gcm` 0.10, RustCrypto, licence
     Apache-2.0 OR MIT, compatible EUPL-1.2 — **dépendance neuve**, déclarée au `Cargo.toml` de `kesh-api` avec un
     commentaire de story comme les autres, et `base64` 0.22, déjà au `Cargo.lock:301-302` en transitive) ; nonce de
     96 bits tiré de `OsRng` **à chaque `seal`** (jamais réutilisé, jamais dérivé) ; tag de 128 bits concaténé au
     chiffré (forme de `aes-gcm`). **AAD** canonique, une seule fonction `aad(company_id, connection_id, name,
     key_version)` → `b"kesh-provider-secret|v1|company=<id>|connection=<id>|name=<nom>|key=<version>"` : un chiffré
     déplacé vers une autre société, une autre connexion ou un autre nom de secret ne s'ouvre pas (test pour chacun
     des trois). **`KEY_VERSION: u8 = 1`** constante (rotation hors périmètre, C-38-13).
   - `SecretError` : `KeyMissing`, `KeyChanged { stored_fingerprint }` (empreinte stockée ≠ empreinte courante),
     `Unreadable` (échec d'authentification GCM, forme invalide) — **aucune variante ne porte de clair**, `Display`
     sans données. Tests unitaires : aller-retour ; deux `seal` du même clair → nonces et chiffrés différents ;
     chiffré altéré d'un octet → `Unreadable` ; mauvaise clé → `KeyChanged` quand l'empreinte diffère ; AAD
     modifiée → `Unreadable`.
   - **Seule voie de lecture d'un clair** : une fonction `pub(crate)` de ce module, appelée par `POST …/test`
     (AC-7, pour vérifier le déchiffrement, le clair jeté aussitôt) et, à partir de 38-3, par le transport d'un
     passage. Une garde de source (test qui lit `crates/kesh-api/src`) liste les appelants admis de `open` ; un appel
     neuf la fait rougir (même famille que `lancement_par_main_seulement`, `auto_backup.rs`).

2. **AC-2 — Migration `2026111X000001_provider_connections.sql`** (non breaking, DDL seul ; numéro de date au
   développement, **postérieur à toutes les migrations de l'Epic 37** — `20261114000003` au plus tard d'après
   `37-3a`) :
   - **`payment_provider_connections`** (table **applicative**, C-38-2) : `id BIGINT AUTO_INCREMENT PRIMARY KEY`,
     `company_id BIGINT NOT NULL` (FK `companies`, `RESTRICT`), `provider VARCHAR(32) CHARACTER SET ascii COLLATE
     ascii_bin NOT NULL` (`chk_ppc_provider` : `BINARY provider REGEXP '^[a-z][a-z0-9_]{1,31}$'` — **format seul**,
     aucune énumération, E4 : Wise et Stripe s'ajoutent sans DDL), `label VARCHAR(100) NOT NULL`
     (`chk_ppc_label_len` : `CHAR_LENGTH(TRIM(label)) BETWEEN 1 AND 100`), `fetch_from DATE NOT NULL`, `schedule
     VARCHAR(10) CHARACTER SET ascii COLLATE ascii_bin NOT NULL DEFAULT 'manual'` (`chk_ppc_schedule` : `BINARY
     schedule IN ('daily','manual')`, domaine fermé : un réglage de planification inconnu ne doit jamais se lire en
     « quotidien »), `last_run_at DATETIME(3) NULL`, `last_run_status VARCHAR(20) CHARACTER SET ascii COLLATE ascii_bin
     NULL` (format seul, `REGEXP '^[a-z_]{1,20}$'`, lu avec `Other(String)`, E4 ; **écrit par 38-3**), `last_error
     VARCHAR(500) NULL` (écrit par 38-3, jamais un secret), `settings JSON NOT NULL DEFAULT '{}'`
     (`chk_ppc_settings` : `JSON_VALID(settings) AND JSON_TYPE(settings) = 'OBJECT'`, patron
     `20261025000001_mcp_keys_and_server_settings.sql:81-83`, E24), `created_by_user_id BIGINT NOT NULL` (FK `users`,
     `RESTRICT` — auteur du passage quotidien en 38-3, E5), `version INT NOT NULL DEFAULT 1`, `created_at`,
     `updated_at` `DATETIME(3) NOT NULL` ; `UNIQUE uq_ppc_company_label (company_id, label)` (deux connexions
     « PayPal Lightning » seraient indiscernables à l'écran et au passage).
     ⚠️ `JSON` est un `LONGTEXT` validé sous MariaDB (E24) : la colonne se lit en `String` (`settings_json`), jamais
     filtrée en SQL.
   - **`_kesh_provider_secrets`** (table **système**, E2, préfixe `_` obligatoire,
     `convention_du_prefixe_des_tables_systeme`, `backup.rs:966`) : `connection_id BIGINT NOT NULL`, `name
     VARCHAR(40) CHARACTER SET ascii COLLATE ascii_bin NOT NULL` (`REGEXP '^[a-z][a-z0-9_]{1,39}$'`), `company_id
     BIGINT NOT NULL` (redondant avec la connexion, porté pour l'AAD et le contrôle de portée), `nonce BINARY(12) NOT
     NULL`, `ciphertext VARBINARY(16384) NOT NULL` (une clé privée PEM de Wise, 39, y tient), `key_version TINYINT
     UNSIGNED NOT NULL`, `key_fingerprint CHAR(16) CHARACTER SET ascii NOT NULL`, `updated_by_user_id BIGINT NOT NULL`
     (sans FK : table système, comme `audit_log.user_id` depuis 25-1a), `updated_at DATETIME(3) NOT NULL` ; `PRIMARY
     KEY (connection_id, name)` ; FK `connection_id → payment_provider_connections(id) ON DELETE CASCADE`.
     ⛔ **La cascade ne suffit pas** : la restauration et la remise à zéro tournent en `FOREIGN_KEY_CHECKS = 0`
     (`restore_body`, `backup.rs:753` ; `reset_body`, `crates/kesh-seed/src/lib.rs:559-565`), où InnoDB n'exécute
     aucune action en cascade — et une connexion restaurée reprend son `id`. D'où l'effacement explicite de l'AC-8.
   - **P5** : une ligne à `docs/migrations-idempotence-audit.md` (`tracked-by-sqlx`, deux `CREATE TABLE` sans `IF NOT
     EXISTS`) ; total recompté à la source aux **deux** sites (en-tête de section et ligne `Total`), partition
     recomptée (CLAUDE.md, P5). **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` exécuté, chaque
     site inspecté et cité au Change Log. **P7** : aucune écriture de données, rien à trier
     (`every_data_backfill_migration_is_triaged` reste vert sans entrée). **P8** : sans objet (fichier neuf).
     **Squash de test** `crates/kesh-db/test-schema/` mis à jour (garde `test_schema_guard.rs`).
   - **Liste close des absences admises** (37-1 AC-6, R1, E18) : `payment_provider_connections` y entre en
     `Absence::Table` (nom de la constante fixé au développement de 37-1) ; une sauvegarde antérieure à l'Epic 38
     s'importe sans connexion. Le test de 37-1 qui vérifie chaque entrée (table dans `TABLES_TO_TRUNCATE`, ajoutée par
     une migration de l'epic) est **étendu** aux migrations de l'Epic 38 (borne basse inchangée, version de la
     migration lue au `MIGRATOR`) ; l'import d'une sauvegarde « d'avant l'Epic 38 » (export courant retouché, patron
     des sauvegardes 0.17.4/0.17.5 d'`admin_full_import_e2e.rs`) rend 200, sans connexion.
   - **Inscriptions** : `payment_provider_connections` à `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs:46`),
     **avant `companies` et `users`** et après toute future table enfant (38-2 placera `payment_provider_accounts`
     **avant** elle) — place proposée : juste **après `bank_imports`** (:101) et avant `bank_profiles` (:102), pour que les
     tables de 38-2 et 38-3 (enfants de la connexion, et `provider_fetch_runs`, que `bank_imports` référencera) s'y
     intercalent dans l'ordre enfants → parents ;
     absente de `RESET_PRESERVED_TABLES` (:137) : une connexion est une donnée de la société, vidée par la remise à zéro
     (`reset_partition_is_guarded_by_the_schema`, :1106, reste vert : la table rejoint `companies` directement).
     `_kesh_provider_secrets` à la constante de test `TABLES_SYSTEME` (`backup.rs:949`) — **jamais** à
     `TABLES_TO_TRUNCATE` : elle n'est ni exportée ni restaurée (E2). Export CSV utilisateur :
     `payment_provider_connections` à **`TABLES_HORS_EXPORT`** (`crates/kesh-api/src/exports/global.rs:170`) avec le
     motif « Réglage d'une connexion de prestataire de paiement (libellé, planification, identifiant d'application),
     sans valeur comptable ; la sauvegarde la porte, sans ses secrets », patron `mcp_server_settings` (`:240`) ; la
     garde `export_couvre_toutes_les_tables` reste verte.

3. **AC-3 — Routes et droits (C-38-2, NFR-38-1-3).** Toutes dans **`admin_routes`** (`crates/kesh-api/src/lib.rs:258`,
   bloc `KESH-ADMIN-ROUTES`), **avant** ses deux `route_layer` (`:513-520` : `require_admin_role` puis
   `require_not_pat`, dont l'ordre rend `403 API_KEY_ADMIN_FORBIDDEN` à toute clé d'API, même d'Administrateur) — la
   garde de source d'`admin_pat_denied_e2e` le contrôle :
   - `GET /api/v1/provider-connections` (liste de la société), `POST /api/v1/provider-connections` ;
   - `GET|PATCH|DELETE /api/v1/provider-connections/{id}` ;
   - `PUT /api/v1/provider-connections/{id}/secrets` ;
   - `POST /api/v1/provider-connections/{id}/test`.
   Comptable et Consultation → `403` du RBAC ; clé d'API → `403 API_KEY_ADMIN_FORBIDDEN` (tests sur les sept routes,
   un Comptable et une clé d'API d'Administrateur). Une connexion d'une autre société → `404` (portée `company_id`,
   jamais d'existence révélée). ⚠️ **Point de contact 38-3** : `POST …/fetch` est ouvert au Comptable (E10) ; 38-3
   décide si le Comptable lit aussi l'état des connexions (lecture restreinte, sans `settings`) — 38-1 n'ouvre rien
   hors de l'Administrateur.
   **Forme de réponse** d'une connexion (`ProviderConnectionResponse`, camelCase) : `id`, `provider`, `label`,
   `fetchFrom` (`AAAA-MM-JJ`), `schedule`, `settings` (objet resérialisé depuis la struct, AC-4 ; `null` si le
   prestataire est inconnu ou les réglages illisibles), `usable` (booléen), `unusableReason` (`null` |
   `provider_unknown` | `settings_invalid` | `secrets_key_missing` | `secret_missing` | `secret_unreadable`), `secretSet`
   (booléen : toutes les entrées de secret exigées par le prestataire existent), `secretUpdatedAt` (le plus récent,
   `null` sinon), `lastRunAt`, `lastRunStatus`, `lastError` (`null` en 38-1), `createdAt`, `updatedAt`, `version`.
   **Jamais** de nonce, de chiffré, d'empreinte ni de nom de colonne du coffre. `usable` et `unusableReason` se
   calculent dans cet ordre de priorité : prestataire inconnu → réglages illisibles → clé absente → secret manquant →
   secret illisible (empreinte de clé différente : sans déchiffrer ; chiffré altéré : vu seulement par `…/test`).
   **Registre des routes (C-38-12)** : `LIB_ROUTES` (`crates/kesh-api/tests/audit_route_registry.rs:382`) **+5**
   routes mutantes — `("post", "provider_connections::create_connection", Traced, SansEcritureAuJournal)`,
   `("patch", "…::update_connection", Traced, …)`, `("delete", "…::delete_connection", Traced, …)`, `("put",
   "…::put_secrets", Traced, …)`, `("post", "…::test_connection", NoMatter("n'écrit rien en 38-1 : contrôle local ;
   38-7 y écrit accountEmail et payerId et la passe à Traced"), SansEcritureAuJournal)` — ; le total de
   `the_registry_partition_is_what_the_story_declares` (`:1009`) et son message suivent **en delta** (+5 routes, +4
   tracées, +1 sans mutation), **cumulés** avec ceux de l'Epic 37 tels qu'ils seront à la fusion (146 au commit de
   relevé, avant l'Epic 37). Le partitionnement de rejeu suit (`SansEcritureAuJournal` ×5 : aucune n'écrit au journal
   comptable).

4. **AC-4 — Créer et modifier ; réglages typés (FR-38-1-1, FR-38-1-3, E24).**
   - **Structs** dans `kesh-core::providers::settings` (module neuf, serde seul, sans I/O — lu par `kesh-api` et, dès
     38-3, par `kesh-providers`) : `enum Provider { Paypal, Other(String) }` (`as_str`, `FromStr` **tolérant** : jamais
     d'erreur, E4) ; `enum ProviderSettings { Paypal(PaypalSettings), Other }` ; `PaypalSettings`
     `#[serde(deny_unknown_fields, rename_all = "camelCase")]` : `client_id: Option<String>` (identifiant de
     l'application REST, non secret chez PayPal), `account_email: Option<String>` et `payer_id: Option<String>`
     (**posés par le serveur** en 38-7, lus en 38-1). Tout champ ajouté plus tard est `Option` + `#[serde(default)]`
     (`eventKindOverrides` : 38-8). ⚠️ `deny_unknown_fields` est **voulu** (contrat de `ColumnMapping`,
     `crates/kesh-import/src/csv/profile.rs:36-48`) : un binaire antérieur refuse bruyamment des réglages écrits par
     un binaire ultérieur, jamais en silence. `parse_settings(provider, json) -> Result<ProviderSettings,
     SettingsError>` : prestataire `Other` → `ProviderSettings::Other` sans lire le JSON ; JSON illisible → erreur.
     **Aucun champ secret** dans aucune struct : un `clientSecret` glissé dans `settings` est une clé inconnue → refus
     (test nommé).
   - **Entité et dépôt** `kesh-db` : `entities::payment_provider_connection` (`settings_json: String`, `provider:
     String`, patron `BankProfile::parse_column_mapping`, `crates/kesh-db/src/entities/bank_profile.rs:50-57`, sans
     dépendance de `kesh-db` vers `kesh-core::providers` : le décodage typé se fait dans `kesh-api`) ;
     `repositories::payment_provider_connections` : `list_by_company`, `find_by_id_for_company`, `create_in_tx`,
     `update_in_tx` (version contrôlée, `OptimisticLockConflict`), `delete_in_tx` — chaque écriture **avec son audit
     dans la même transaction**.
   - **`POST /api/v1/provider-connections`**, corps `{ provider, label, fetchFrom, schedule?, settings? }` :
     `provider` ∉ prestataires connus de ce binaire (`paypal` seul en 38-1) → `400 PROVIDER_UNKNOWN`,
     `details.provider` ; `label` vide, trop long (> 100 caractères Unicode après `trim`) ou portant un caractère de
     contrôle (`kesh_core::text::is_hidden_in_line`, comme la description d'un compte bancaire,
     `routes/bank_accounts.rs:480-482`) → `400 VALIDATION_ERROR` ; `fetchFrom` postérieure au **lendemain** de la date
     du jour UTC → `400 VALIDATION_ERROR` (le serveur n'a pas de fuseau : il lit `Utc::now().date_naive()`,
     `crates/kesh-api/src/services/rapports.rs:862-870` ; la tolérance d'un jour admet le jour de Zurich entre minuit et
     2 h) (la borne des trois
     ans de PayPal est en 38-7) ; `schedule` absent = `manual` ; `settings` absent = `{}` ; `settings` désérialisé
     dans la struct du prestataire (refus → `400 PROVIDER_SETTINGS_INVALID`, `details.reason` = message serde **sans
     valeur reçue** — le nom de la clé fautive seulement) ; un champ posé par le serveur (`accountEmail`, `payerId`)
     fourni par le client → même refus, `details.field`. `clientId` : 1 à 128 caractères `[A-Za-z0-9_-]`, sinon même
     refus. Libellé déjà pris dans la société → `409 PROVIDER_CONNECTION_LABEL_DUPLICATE` (contrainte
     `uq_ppc_company_label`, mappage de l'erreur 1062 sur ce nom d'index). `created_by_user_id` = l'Administrateur.
     Réponse `201`, forme de l'AC-3. La struct est **resérialisée** avant l'écriture : aucun JSON reçu n'est stocké tel
     quel (E24).
   - **`PATCH /api/v1/provider-connections/{id}`**, corps `{ version, label?, fetchFrom?, schedule?, settings? }` —
     champ absent = inchangé ; `settings` présent **remplace** les champs modifiables du prestataire (`clientId`) et
     **conserve** ceux que le serveur pose (`accountEmail`, `payerId`) ; `provider` **immuable** (présent → `400
     VALIDATION_ERROR`, `details.field = "provider"`). Mêmes refus qu'au `POST` ; connexion d'un prestataire inconnu →
     `409 PROVIDER_UNKNOWN` (on ne réécrit pas des réglages qu'on ne sait pas lire) ; réglages stockés illisibles et
     `settings` absent du corps → l'écriture des autres champs passe (le libellé se corrige), `settings` présent →
     remplacé après validation. Version périmée → `409 OPTIMISTIC_LOCK_CONFLICT`. Requête sans changement → `200`
     sans audit ni incrément (court-circuit no-op, patron KF-004).
   - **Audit** (`crates/kesh-api/src/audit_labels.rs`) : type d'entité `payment_provider_connection` (inséré dans
     `ENTITY_TYPES` à son rang alphabétique) ; actions `payment_provider_connection.created`, `.updated`, `.deleted`,
     `.secrets_updated` (dans `ACTIONS`, triées) ; libellés aux **quatre** locales
     (`crates/kesh-i18n/locales/{fr,de,it,en}/messages.ftl`, clés dérivées par le module) ; détail : instantané
     `{ provider, label, fetchFrom, schedule, settings }` avant/après — `settings` ne contient aucun secret par
     construction (AC-4, struct) ; `secrets_updated` porte seulement `{ names: ["client_secret"] }`. La garde
     `tests/audit_label_registry.rs` reste verte.

5. **AC-5 — Supprimer (FR-38-1-1, FR-38-1-4).** `DELETE /api/v1/provider-connections/{id}` avec corps `{ version }`
   (patron `ArchiveBankAccountBody`, `routes/bank_accounts.rs:205`) : suppression **physique** de la ligne et de ses
   secrets (cascade, et `DELETE` explicite du coffre dans la même transaction, pour ne pas dépendre de l'état de
   `FOREIGN_KEY_CHECKS` de la session) ; audit `payment_provider_connection.deleted` (instantané). Refus `409
   PROVIDER_CONNECTION_IN_USE`, `details.bankAccountIds` (triés), tant qu'une ligne enfant la désigne. **En 38-1, aucune
   table enfant n'existe encore** : le code et son message (quatre locales) se posent ici, avec le mappage de l'erreur
   MariaDB **1451** (clé étrangère `RESTRICT`) **sur le nom de la contrainte** (patron du mappage des FK du dépôt) ; le
   **test du refus vit en 38-2**, qui crée `payment_provider_accounts` (FK `RESTRICT`) et y ajoute la lecture des
   `bankAccountIds` sous verrou ; 38-3 y ajoute les passages (`provider_fetch_runs`, à trancher en 38-3 : `RESTRICT`
   ou suppression des passages). *Écart à l'epic, de forme seulement* : FR-38-1-4 est portée par 38-1 mais ne se teste
   qu'avec la table de 38-2.

6. **AC-6 — Déposer les secrets (FR-38-1-2, NFR-38-1-1, E3).** `PUT /api/v1/provider-connections/{id}/secrets`,
   corps propre au prestataire, `deny_unknown_fields` — PayPal : `{ "clientSecret": "<chaîne>" }` (1 à 512 caractères
   ASCII imprimables, sans espace de bord ; gabarit `<…>` ou `changeme` refusé par la même fonction que la
   configuration). Clé absente → `409 PROVIDER_SECRETS_KEY_MISSING` (rien n'est écrit) ; prestataire inconnu → `409
   PROVIDER_UNKNOWN` ; corps refusé → `400 PROVIDER_SECRETS_INVALID`, `details.field` seulement, **jamais la valeur**
   (le corps refusé n'est ni journalisé ni renvoyé). Écriture : `seal` (AC-1) avec l'AAD de la ligne, `INSERT … ON
   DUPLICATE KEY UPDATE` de chaque nom (`client_secret`), `key_version`, `key_fingerprint`, `updated_by_user_id`,
   `updated_at` ; la connexion prend `version + 1` et `updated_at`, sous `SELECT … FOR UPDATE` de sa ligne (pas de
   `version` dans le corps : remplacer un secret n'entre pas en conflit avec une modification de libellé ; le
   verrou de ligne sérialise deux dépôts concurrents) ; audit `payment_provider_connection.secrets_updated`, détail
   `{ names }`. Réponse `200` : `{ secretSet: true, secretUpdatedAt, version }` — rien d'autre. Le clair est tenu dans
   un `Zeroizing<String>` du corps à `seal`, et le type du corps n'implémente ni `Debug` dérivé ni `Serialize`
   (`Debug` manuel masqué) : un `tracing::debug!(?body)` futur n'emporterait rien.

7. **AC-7 — Tester la connexion, contrôle local (FR-38-1-7).** `POST /api/v1/provider-connections/{id}/test` rend
   `200 { usable, unusableReason, checks: [{ check, ok }] }` où `check` ∈ `provider_known`, `settings_readable`,
   `client_id_present` (PayPal), `secrets_key_present`, `secret_present`, `secret_decryptable` — ce dernier **ouvre**
   le secret par la voie unique de l'AC-1 et jette le clair aussitôt (aucune comparaison, aucune trace). Rien n'est
   écrit. *Point de contact* : 38-3 ajoute le contrôle `provider_reachable` par `Connector::balances`, et 38-7 pose
   `accountEmail`/`payerId` aux réglages (la route passe alors `Traced`). Tests : clé absente, secret absent, clé
   changée (`secret_unreadable` par l'empreinte), chiffré altéré en base (`secret_unreadable` par GCM), tout vert.

8. **AC-8 — Restauration, remise à zéro, sauvegarde (FR-38-1-5, FR-38-1-6, E2).**
   - **Restauration complète** (`run_backup_and_restore`, `crates/kesh-api/src/routes/admin.rs:361`) : dans **sa**
     transaction, après `restore_tables_in_tx` (`:441`) et à côté de la révocation des sessions (`:512-519`, même
     esprit : un accès ne survit pas à une restauration), `DELETE FROM _kesh_provider_secrets` **en entier** ; le
     nombre effacé entre au détail de l'audit `admin.full_import` (`providerSecretsErased`). Un refus ultérieur de la
     restauration annule aussi l'effacement (même transaction). Test : une connexion `id = N` avec secret, export,
     suppression de la connexion, nouvelle connexion recréée puis **restauration de l'export** (la connexion revient
     avec `id = N`) → `secretSet: false`, `unusableReason = "secret_missing"` ; et un second test où l'archive vient
     d'une **autre** installation dont une connexion porte le même `id` qu'une connexion locale : aucun secret local
     ne s'y rattache. **Décidé (E2, ex-Q4)** : les secrets se ressaisissent après toute restauration, même avec la
     même clé.
   - **Sauvegarde avant mise à jour** (`pre_upgrade_backup`) : n'exporte que les tables sans préfixe `_` — le coffre
     n'y entre pas (test d'inventaire existant, rien à ajouter hormis `TABLES_SYSTEME`).
   - **Remise à zéro** (`reset_body`, `crates/kesh-seed/src/lib.rs:559`) : la connexion est vidée par
     `reset_cleared_tables()` (dérivée, `backup.rs:156`) ; **ajouter** `DELETE FROM _kesh_provider_secrets` dans la
     même transaction (la table système n'est pas dans la liste, et la cascade ne joue pas en `FOREIGN_KEY_CHECKS =
     0`). Test au patron de `reset_restores_the_stub_columns_in_place` (`onboarding_audit_e2e.rs`).
   - **`.keshbackup`** : `payment_provider_connections` y passe par l'export générique (`export_table`,
     `backup.rs:214`, colonnes lues à `information_schema` ; la colonne `JSON` y voyage comme
     `bank_profiles.column_mapping`) ; aller-retour testé (réglages identiques, `secretSet: false` après).

9. **AC-9 — Aucun secret nulle part (NFR-38-1-2).** Un test d'intégration dépose un secret marqueur unique
   (`kesh-test-secret-<uuid>`) puis exerce : les sept routes (corps de **toutes** les réponses, erreurs comprises —
   `PUT …/secrets` refusé pour un champ en trop, `PATCH` avec `settings.clientSecret`) ; `audit_log` (toutes les
   colonnes texte de toutes les lignes) ; le `.keshbackup` complet (octets de l'archive décompressée) ; l'export CSV
   global (octets du ZIP décompressé) ; les journaux `tracing` du processus de test (abonné de test qui capture la
   sortie, niveau `TRACE`) ; et `_kesh_provider_secrets.ciphertext` (le marqueur n'y figure pas en clair). Le
   marqueur n'apparaît **nulle part**. Variante : la même recherche sur le **nom** `clientSecret` dans les réponses
   (seul `secretSet` en parle).

10. **AC-10 — Configuration distribuée et manuel (NFR-38-1-6).**
    - `KESH_SECRETS_KEY` transmise par `docker-compose.yml` et `docker-compose.prod.yml` sous `environment:` (forme
      `${KESH_SECRETS_KEY:-}` : vide signifiant « absente », liste `VIDE_SIGNIFIANT` de
      `crates/kesh-api/tests/configuration_transmise.rs`) ; ligne commentée au `.env.example` avec la commande de
      génération (`openssl rand -base64 32`) ; emplacement de lecture inscrit à `EMPLACEMENTS_AUTORISES` ; la garde
      `configuration_transmise` (familles L, T, V, E, F) reste verte. `docker-compose.dev.yml` : clé de développement
      factice **non gabarit** (comme les autres secrets de dev), si la garde G17 l'exige.
    - **Manuel d'administration** (`docs/manual/fr/admin-manual.tex`, section des variables d'environnement, près de
      `KESH_JWT_SECRET`, `:253`, `:577-579`) : rôle de la clé ; génération ; **perte ou changement de la clé = secrets à
      ressaisir** (rien d'autre n'est perdu) ; les secrets ne voyagent **ni** dans la sauvegarde **ni** dans la
      restauration, qui les efface : à ressaisir après toute restauration ; la clé ne se met **jamais** dans la
      sauvegarde ni à côté d'elle. PDF régénéré (`make fr` dans `docs/manual/`) et contrôlé aplati (`pdftotext … | tr
      '\n' ' '`). Les écrans (38-9a) complètent la section. **Recette E2E** : `docs/testing.md` § *Prérequis
      Playwright local* et le bloc du `CLAUDE.md` gagnent `KESH_SECRETS_KEY` (valeur factice de 32 octets base64,
      non gabarit) — sans elle, les E2E de 38-9 ne peuvent pas déposer de secret.
    - **MCP** : aucun outil neuf (E23, décidé) ; les connexions et le coffre ne sont exposés par aucun outil — rien à
      ajouter au registre des outils.

## Tâches / sous-tâches

- [ ] **T1 — Dépendances et clé** (AC-1) : `aes-gcm` 0.10 (features `aes`, `alloc`, `getrandom`, `zeroize`), `base64`
      0.22, `zeroize` au `Cargo.toml` de `kesh-api`, commentaire de story ; `cargo deny`/licences si l'outillage les
      contrôle. `Config` : champ, lecture, deux variantes de `ConfigError`, `Display` sans valeur, tests (absente,
      gabarit, base64 invalide, 31 et 33 octets, valide). `AppState` porte la clé (par `config`).
  - [ ] T1.1 module `kesh_api::secrets` (`SecretsKey`, `aad`, `seal`, `open`, `SecretError`, `KEY_VERSION`) et ses
        tests unitaires ; garde de source des appelants d'`open`.
- [ ] **T2 — Migration et schéma** (AC-2) : fichier SQL, squash `test-schema`, ligne d'audit d'idempotence et
      compteurs recomptés, P6 exécuté et cité, `TABLES_TO_TRUNCATE`, `TABLES_SYSTEME` (test), `TABLES_HORS_EXPORT`,
      liste close R1 et extension de son test, test d'import d'une sauvegarde d'avant l'Epic 38.
- [ ] **T3 — Réglages typés** (AC-4) : `kesh-core::providers::{mod, settings}`, `Provider`, `ProviderSettings`,
      `PaypalSettings`, `parse_settings`, tests (clé inconnue refusée, `clientSecret` refusé, prestataire inconnu,
      JSON non objet, champ serveur conservé au `PATCH`).
- [ ] **T4 — Entité, dépôt, routes** (AC-3 à AC-7) : `kesh-db` (entité, dépôt, mappage 1062/1451 par nom de
      contrainte), `kesh-api::routes::provider_connections`, variantes `AppError` (`PROVIDER_UNKNOWN`,
      `PROVIDER_SETTINGS_INVALID`, `PROVIDER_SECRETS_INVALID`, `PROVIDER_SECRETS_KEY_MISSING`,
      `PROVIDER_SECRET_UNREADABLE`, `PROVIDER_CONNECTION_IN_USE`, `PROVIDER_CONNECTION_LABEL_DUPLICATE`) et leurs
      messages aux quatre locales, montage dans `admin_routes`, registre des routes (+5) et décomptes.
- [ ] **T5 — Audit** (AC-4) : `ENTITY_TYPES`, `ACTIONS`, libellés quatre locales, détails sans secret.
- [ ] **T6 — Restauration et remise à zéro** (AC-8) : effacement du coffre dans `run_backup_and_restore` et
      `reset_body`, détail d'audit, tests (identifiants réutilisés, autre installation, refus ultérieur qui annule).
- [ ] **T7 — Recherche du secret partout** (AC-9).
- [ ] **T8 — Configuration distribuée, manuel, recette E2E** (AC-10) : compose ×2 (+ dev si requis), `.env.example`,
      garde `configuration_transmise`, manuel d'administration et PDF, `docs/testing.md`, bloc E2E du `CLAUDE.md`.
- [ ] **T9 — Gates** : gate complet (story qui touche `kesh-db/migrations` : ciblage interdit, CLAUDE.md), base de
      gate remise à zéro avant ; `cargo deny` si présent ; aucun E2E neuf (aucun écran), E2E complet au dernier
      commit de code (règle D7) pour vérifier que la variable neuve ne casse pas le montage.

## Notes de développement

- **Patron du chiffrement** : aucun chiffrement symétrique n'existe au dépôt (relevé de l'epic, inventaire *Secrets*) ;
  les clés d'API et MCP sont **hachées** (`20260605000001_api_keys.sql:23`), ce qui ne convient pas ici : le secret
  doit être relu pour appeler PayPal. Ne pas réinventer : `aes-gcm` avec `OsRng`, nonce aléatoire de 96 bits —
  au volume d'un coffre (quelques secrets par connexion, réécrits rarement), le risque de collision de nonce est
  négligeable ; ne **jamais** utiliser de compteur ni de nonce dérivé de l'identifiant.
- **AAD et restauration** : l'AAD lie le chiffré à `(société, connexion, nom, version de clé)`. Elle **ne suffit
  pas** contre la réutilisation d'identifiants après restauration (l'AAD d'une connexion restaurée `N` est la même
  que celle de l'ancienne `N`) : seul l'effacement de l'AC-8 la ferme. Ne pas retirer l'effacement en croyant l'AAD
  suffisante.
- **`FOREIGN_KEY_CHECKS = 0`** : restauration (`restore_body`) et remise à zéro (`reset_body`) ; aucune cascade n'y
  joue. Toute table enfant d'une connexion ajoutée plus tard (38-2, 38-3) est **applicative** et passe par
  `TABLES_TO_TRUNCATE` ; seul le coffre, système, demande un effacement explicite.
- **Ordre de `TABLES_TO_TRUNCATE`** : enfants → parents ; `payment_provider_connections` est enfant de `companies` et
  `users` ; 38-2 et 38-3 inséreront leurs tables **avant** elle.
- **Variable d'environnement** : toute lecture passe par `env_nonempty` (garde (L) de `configuration_transmise`).
- **Pas de `Copy`** sur `Provider` (porte un `String`) : comparer par `as_str()`.
- **Réponses** : `settings` rendu est la struct resérialisée (champs `null` omis ou non : à fixer une fois, test).
- **38-9a** affichera `unusableReason` ; ses textes vivent là, pas ici (seuls les messages d'erreur d'API sont posés
  par 38-1).
- **Références** : epic E1, E2, E3, E24, C-38-1, C-38-2 ; `config.rs:590, 670-683, 1538` ; `backup.rs:46, 137, 156,
  214, 753, 949, 966, 1106` ; `admin.rs:361, 441, 512-519` ; `kesh-seed/src/lib.rs:559-610` ; `lib.rs:258, 513-520` ;
  `rbac.rs:58` ; `exports/global.rs:116, 170, 240` ; `audit_labels.rs:24` ; `audit_route_registry.rs:382, 1009` ;
  `bank_profile.rs:50-57` ; `csv/profile.rs:36-48` ; `20261025000001_mcp_keys_and_server_settings.sql:71-83` ;
  37-1 AC-6 (liste close).

## Questions ouvertes

- **Q-38-1-a** (38-3, 38-9a) : le Comptable, qui peut lancer un passage (`POST …/fetch`, E10), doit-il **lire**
  l'état des connexions (lecture restreinte sans `settings`) ? 38-1 réserve tout à l'Administrateur, conformément à
  NFR-38-1-3 ; à trancher en 38-3.
- **Q-38-1-b** (Beat) : l'unicité du libellé par société (`PROVIDER_CONNECTION_LABEL_DUPLICATE`) est un ajout de cette
  fiche ; la retirer si deux connexions homonymes ont un usage.

## Change Log

- 2026-11-15 — rédaction (Opus).
