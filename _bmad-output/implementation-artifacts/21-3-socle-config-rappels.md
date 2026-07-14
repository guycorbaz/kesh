# Story 21.3: Socle de configuration des rappels + templates par niveau (backend)

Status: ready-for-dev

<!-- Créée 2026-07-13 par bmad-create-story. Cartographie ground-truth par 4 agents Explore parallèles (patrons vat_rates / company_invoice_settings / email_templates / conventions migration-export-backup). Story backend « socle » de l'Epic 21 : elle POSE les tables et le sous-système de config des rappels, SANS aucune évaluation d'éligibilité ni envoi (21-5a/21-5b) ni frontend (21-4). Décisions figées par le plan d'epic (D5, D6, D7, D14, section D) + critique adversariale 3 agents. -->

## Story

En tant que **PME suisse assujettie à la TVA**,
je veux **configurer mes niveaux de rappel (délais + frais) et disposer de textes d'e-mail de rappel par niveau, avec un jeu de défauts sensé prêt à l'emploi**,
afin de **pouvoir ensuite relancer mes débiteurs de façon graduée (Epic 21) sans repartir d'une page blanche**.

## Contexte

Cette story est le **socle backend** du système de rappels (dunning). Elle ne fait qu'établir la configuration et les défauts ; l'éligibilité, la liste à rappeler et l'envoi arrivent en 21-5a/21-5b, le frontend en 21-4.

Trois briques :
1. **`dunning_levels`** — collection company-scoped des niveaux de rappel (délai depuis l'étape précédente + frais), calquée sur `vat_rates` (sentinel lock, verrou optimiste, audit) MAIS avec des niveaux **numérotés contigus** (renumérotation à la suppression — cf. D5).
2. **`company_dunning_settings`** — singleton de config (période de grâce + discriminant `seeded_at`), calqué sur `company_invoice_settings` (get-or-create, verrou optimiste).
3. **Type `invoice_reminder` + templates par niveau** — extension du sous-système `email_templates` d'Epic 20 : nouveau type d'enum, colonne `level_number`, **résolution en cascade**, défauts Rust par niveau 1-3 (option **A+** du plan, D-section).

Le tout **zéro-config** : un **seed idempotent sous sentinel lock** pose 3 niveaux par défaut au premier accès, avec sémantique explicite « table vidée volontairement = dunning désactivé, pas de résurrection » (discriminant `seeded_at`).

## Acceptance Criteria

### Base de données & migrations

1. **Migration A — tables dunning (non-breaking, 2 `CREATE TABLE`)**. Nouveau fichier `crates/kesh-db/migrations/20260713000001_dunning_config.sql` (ajuster le timestamp au jour du dev) :
   - **`dunning_levels`** (calque `vat_rates` DDL, `20260428000001_vat_rates.sql:13-31`) : `id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY`, `company_id BIGINT NOT NULL`, `level_number SMALLINT NOT NULL` (1-based, contigu), `delay_days INT NOT NULL`, `fee_amount DECIMAL(7,2) NOT NULL` (max 99999.99, borné par CHECK), `version INT NOT NULL DEFAULT 0`, `created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3)`, `updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)`. Contraintes : FK `fk_dunning_levels_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT` ; `UNIQUE uq_dunning_levels_company_level (company_id, level_number)` ; `CHECK chk_dunning_levels_fee_range (fee_amount >= 0 AND fee_amount <= 10000)` (D5 : bornes 0..10'000, scale 2) ; `CHECK chk_dunning_levels_delay_nonneg (delay_days >= 0)` ; `CHECK chk_dunning_levels_level_positive (level_number >= 1)` ; `INDEX idx_dunning_levels_company (company_id)`. `ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci`.
   - **`company_dunning_settings`** (calque `company_invoice_settings` DDL, `20260417000001_invoice_validation.sql:35-51`) : `company_id BIGINT NOT NULL PRIMARY KEY` (unicité par company = PK, rend `INSERT IGNORE` idempotent), `grace_period_days INT NOT NULL DEFAULT 5`, `seeded_at DATETIME(3) NULL` (**discriminant** : NULL = jamais seedé, NON-NULL = seedé une fois — pas de résurrection), `version INT NOT NULL DEFAULT 1`, `created_at`/`updated_at DATETIME(3)` idem. FK `fk_cds_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT` ; `CHECK chk_cds_grace_nonneg (grace_period_days >= 0)`.
   - **PAS de backfill INSERT** ici (le seed est lazy sous sentinel lock — AC 7, contrairement à `vat_rates` qui backfille en migration). Rationale : le seed doit poser `seeded_at` et être idempotent/annulable, ce qu'un backfill de migration ne permet pas proprement.
2. **Migration B — extension `email_templates` (BREAKING sur les données)**. Nouveau fichier `crates/kesh-db/migrations/20260713000002_email_templates_reminder.sql` :
   - `ALTER TABLE email_templates ADD COLUMN level_number SMALLINT NOT NULL DEFAULT 0;` (0 = générique — **jamais de NULL dans l'UNIQUE**, leçon Epic 20 MariaDB ; les lignes existantes deviennent niveau 0).
   - Remplacer l'UNIQUE (`20260708000001_email_templates.sql:48-49`) : `ALTER TABLE email_templates DROP INDEX uq_email_templates_company_type_language;` puis `ADD CONSTRAINT uq_email_templates_company_type_language_level UNIQUE (company_id, template_type, language, level_number);` (`company_id` reste colonne de tête → le leftmost-prefix documenté ligne 20-23 tient).
   - Étendre le CHECK `template_type` (`:42-43`) — **MariaDB ne modifie pas un CHECK en place** : `ALTER TABLE email_templates DROP CONSTRAINT chk_email_templates_template_type;` puis `ADD CONSTRAINT chk_email_templates_template_type CHECK (template_type IN ('invoice_send','invoice_reminder'));`.
   - **Bump breaking (P2, D14)** — **dernière instruction du fichier** : `UPDATE _kesh_version SET kesh_version_min_required = '0.7.0' WHERE id = 1;`. Rationale (à écrire en commentaire SQL) : un binaire v0.6 downgradé fait un **500** sur `list_effective_for_company` dès qu'une ligne `template_type='invoice_reminder'` existe (son `FromStr` strict rejette la valeur, `Decode` échoue). C'est la **donnée**, pas le DDL, qui est breaking → bump honnête, coût nul (v0.7.0 = release cible epic). Version figée dans le SQL (comme `'0.1.0'` d'origine).
3. **Compteurs & audits transverses** (famille « nouvelle table/migration → compteur figé » — 3 régressions de ce type à l'Epic 20) :
   - `crates/kesh-db/tests/migrations_upgrade_path.rs` : incrémenter le compteur de migrations attendues (`grep -nF 'MIGRATOR.migrations.len' + assert_eq!` — **vérifier la valeur exacte** : 51 constaté par `ls migrations/*.sql | wc -l`, +2 → **53** ; le message d'assertion mentionne la story courante).
   - `docs/migrations-idempotence-audit.md` : ajouter **2 lignes** (une par fichier) + mettre à jour les **Statistiques** (`Total : 51 → 53`, `tracked-by-sqlx` +2). Verdict `tracked-by-sqlx` pour les 2 (re-exécution hors sqlx = erreur 1050/1060 ; migration A non-breaking, migration B breaking-bump documenté). **P5 obligatoire** (sinon finding MEDIUM code-review).
   - `crates/kesh-db/tests/migrations_fresh_install.rs` : ajouter `dunning_levels` et `company_dunning_settings` à la liste des tables attendues (`:40` zone).
   - **⚠️ Balayage des tests `_kesh_version` cassés par le 1er bump `min_required` (HIGH-1 validate P3 — raté par Pass 1 ET Pass 2)** : le bump `'0.1.0' → '0.7.0'` change la baseline que **plusieurs tests hard-codent**. Les tests `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` appliquent le migrateur complet → `min_required` vaut `'0.7.0'` après migration B. À corriger (faire `grep -rnF '"0.1.0"' crates/kesh-db/tests/migrations_*.rs` + `grep -nF 'check_downgrade_protection' …` et traiter CHAQUE occurrence sémantiquement) :
     - `migrations_fresh_install.rs:238` `assert_eq!(min_required, "0.1.0")` → `"0.7.0"` (+ commentaire doc `:225-227` qui cite `('0.1.0','0.1.0')`). Vérifier si `:239 last_applied` change (last_applied est posé au boot, pas par la migration — probablement reste `'0.1.0'` en test pur migrateur ; **confirmer par exécution**).
     - `migrations_upgrade_path.rs:398` `downgrade_protection_aligned_when_binary_equals_min` : appelle `check_downgrade_protection(&pool, "0.1.0")` (`:400`) en attendant `Aligned` → après bump, `0.1.0 < 0.7.0` → `DowngradeRefused` → `unwrap()` panique. **Réécrire** : appeler avec `"0.7.0"` (binary == min).
     - `migrations_upgrade_path.rs:411` `downgrade_protection_binary_ahead_when_binary_greater` : `check_downgrade_protection(&pool, "0.2.0")` (`:413`) attend `BinaryAhead{db_min:"0.1.0"}` avec `assert_eq!(db_min, "0.1.0")` (`:416`) → après bump, `0.2.0 < 0.7.0` → `DowngradeRefused` → panique. **Réécrire** : binary `> 0.7.0` (ex. `"0.8.0"`), attendu `db_min "0.7.0"`.
     - Autres call-sites `check_downgrade_protection(&pool, "0.1.0")` (`:298, :321, :347, :381`) : **auditer chacun** — ceux qui testent le REJET (`downgrade_protection_rejects_old_binary:373`) restent verts (rejet toujours attendu) ; ceux qui attendent Aligned/BinaryAhead cassent. Ne pas se fier à cette liste : re-grep à l'implémentation (les numéros de ligne bougeront avec les patches).

### Entités

4. **`crates/kesh-db/src/entities/dunning_level.rs`** (calque `vat_rate.rs`) :
   - `struct DunningLevel` — `#[derive(Debug, Clone, sqlx::FromRow)]`, **PAS de `Serialize`** (anti-fuite `company_id`, comme `VatRate` ; exposition REST via `DunningLevelResponse`). Champs : `id: i64`, `company_id: i64`, `level_number: i16`, `delay_days: i32`, `fee_amount: Decimal` (`rust_decimal::Decimal`), `version: i32`, `created_at`/`updated_at: NaiveDateTime`.
   - `struct NewDunningLevel { company_id, delay_days, fee_amount }` (pas de `level_number` — posé par le repo = MAX+1 ; pas de `version` — posé à 0).
   - `struct UpdateDunningLevel { delay_days: i32, fee_amount: Decimal }` (`level_number` **immutable** — le réordonnancement n'est pas au périmètre v1 ; note explicite dans le fichier).
   - Ré-export dans `entities/mod.rs`.
5. **`crates/kesh-db/src/entities/company_dunning_settings.rs`** (calque `company_invoice_settings.rs`) :
   - `struct CompanyDunningSettings` — `#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]` + `#[serde(rename_all = "camelCase")]`. Champs : `company_id: i64`, `grace_period_days: i32`, `seeded_at: Option<NaiveDateTime>`, `version: i32`, `created_at`/`updated_at: NaiveDateTime`.
   - `struct CompanyDunningSettingsUpdate { grace_period_days: i32 }` (SANS `version` — géré par le repo ; SANS `seeded_at` — jamais piloté par le client).
   - Ré-export dans `entities/mod.rs`.
6. **`EmailTemplateType::InvoiceReminder`** — les **6 sites Rust** exacts (`entities/email_template.rs` sauf le dernier) :
   - **Site 1** définition variant `:25-28` : ajouter `InvoiceReminder`.
   - **Site 2** `const ALL` `:31` : **taille `[…; 1]` → `[…; 2]`** + `EmailTemplateType::InvoiceReminder`.
   - **Site 3** `as_str()` `:34-36` : `Self::InvoiceReminder => "invoice_reminder"`.
   - **Site 4** `allowed_variables()` `:42-53` : pour `InvoiceReminder`, déclarer `&["salutation","contactName","invoiceNumber","amount","dueDate","companyName","reminderLevel","reminderFee","totalDue","daysOverdue"]` (les 6 de base + les 4 nouvelles de D15 ; **déclarées ici même si `build_reminder_vars` qui les alimentera arrive en 21-5b** — sinon `validate_tokens` refuserait les défauts de rappel). Note : `reminderLevel` est déclarée pour personnalisation/usage futur ; les corps par défaut v1 (Dev Notes) ne l'utilisent pas — c'est intentionnel (`all_defaults_only_use_allowed_variables` vérifie utilisés⊆autorisés, pas l'inverse, donc pas de test rouge).
   - **Site 5** `FromStr` `:66-74` : `"invoice_reminder" => Ok(Self::InvoiceReminder)`.
   - **Site 6** `default_template()` `entities/email_template_defaults.rs:21-54` (AC 8).
   - Les impls `Type<MySql>`/`Encode`/`Decode` (`:76-99`) délèguent à `as_str()`/`parse()` → **rien à modifier** (le compilateur signalera tout `match` exhaustif oublié).
   - Ajouter `level_number: i16` au `struct EmailTemplate` (`email_template.rs:103-113`) + à la const `COLUMNS` du repo (`repositories/email_templates.rs:42-43`).
   - **Ajouter aussi `pub level_number: i16` au `struct EffectiveEmailTemplate` (`email_template.rs:118-127`)** (H1 validate P2 — grep confirmé : la struct n'a que `template_type/language/subject/body/version/is_default/allowed_variables`) : `list_effective_for_company` doit porter le niveau de chaque template pour que 21-4 puisse lister/éditer par niveau. **Sémantique du `level_number` retourné = le SLOT demandé** (le niveau pour lequel on résout), PAS le niveau source de la cascade : `get_effective(reminder, FR, 2)` renvoie toujours `level_number = 2`, que le texte vienne de l'override niveau 2, de l'override niveau 0, du défaut Rust niveau 2 ou du générique (`is_default`/`version` reflètent la SOURCE ; `level_number` reflète le SLOT). Répercuter sur les helpers `to_effective_override`/`to_effective_default` (`email_templates.rs:59-85` — ils reçoivent le `level_number` demandé) et sur le DTO `EmailTemplateResponse` (`routes/email_templates.rs:31-42`, champ `level_number`/`levelNumber`).

### Repositories

7. **`crates/kesh-db/src/repositories/dunning_levels.rs`** (calque `vat_rates.rs`) :
   - `list_all_by_company(pool, company_id) -> Vec<DunningLevel>` `ORDER BY level_number`.
   - `find_by_id_for_company(tx, company_id, id) -> Option<DunningLevel>` (helper interne, prend une tx).
   - `create_for_company(tx, company_id, new: &NewDunningLevel) -> DunningLevel` : `level_number = COALESCE(MAX(level_number),0)+1` de la company (calculé sous sentinel lock par le handler — **AC 12**), INSERT, relit via `find_by_id_for_company` sinon `DbError::Invariant`.
   - `update_for_company(tx, company_id, id, fields: &UpdateDunningLevel, expected_version) -> DunningLevel` : verrou optimiste **double garde** (pré-check `version != expected` → `OptimisticLockConflict` + `UPDATE … SET delay_days=?, fee_amount=?, version=version+1 WHERE id=? AND company_id=? AND version=?` + garde `rows_affected()==0`). Calque exact `vat_rates::update_for_company:199-237`.
   - `delete_and_renumber(tx, company_id, id, expected_version) -> ()` : **hard delete + renumérotation** (D5). Séquence sous sentinel lock : pré-check version → `DELETE FROM dunning_levels WHERE id=? AND company_id=? AND version=?` (garde `rows==0` → conflict/not-found) → `UPDATE dunning_levels SET level_number = level_number - 1, version = version + 1 WHERE company_id=? AND level_number > ?` (compacte la suite). ⚠️ **La renumérotation DOIT bumper `version`** (`version = version + 1`) sur les lignes déplacées — sinon le verrou optimiste est contourné : un client détenant l'ancien `version` d'un niveau renuméroté éditerait silencieusement une ligne qui a changé d'identité fonctionnelle (H1 validate P1). **Rationale du hard-delete** (vs soft-delete `vat_rates`) : les rappels déjà envoyés snapshotent `level_number` + `fee_amount` dans `invoice_reminders` (21-5a) → l'historique est protégé, la renumérotation ne le corrompt pas. Documenter en tête de fonction.
   - `count_for_company(tx, company_id) -> i64` (utile seed + éligibilité 21-5a).
   - Réutiliser `bank_accounts::acquire_company_sentinel_lock` (voir Dev Notes) — **jamais** un nouveau sentinel.
8. **`crates/kesh-db/src/repositories/company_dunning_settings.rs`** (calque `company_invoice_settings.rs`) :
   - `get_or_create_default(pool, company_id) -> CompanyDunningSettings` + `get_or_create_default_in_tx(tx, company_id)` **en miroir** (marqueur `MIRROR`, la variante tx finit par `… WHERE company_id = ? FOR UPDATE`), via `INSERT IGNORE INTO company_dunning_settings (company_id) VALUES (?)` + `SELECT`. La course concurrente est absorbée par le PK.
   - `update(pool, company_id, expected_version, user_id, changes: CompanyDunningSettingsUpdate) -> CompanyDunningSettings` : calque exact `company_invoice_settings::update:131-233` — `INSERT IGNORE` lazy, SELECT before, check version → `OptimisticLockConflict`, court-circuit **no-op** (helper `is_no_op_change` comparant `grace_period_days`), `UPDATE … version=version+1 WHERE version=?`, garde `rows==0`, SELECT after, **audit `company_dunning_settings.updated`** (`NewAuditLogEntry::user`, before/after JSON) dans la tx, commit. **`seeded_at` n'est JAMAIS modifié par `update`** (piloté seulement par le seed AC 9).
9. **Seed idempotent sous sentinel lock** (D7) — fonction (repo `dunning_levels` ou module dédié `dunning_seed`) `ensure_seeded_in_tx(tx, company_id) -> ()` :
   - Prendre `acquire_company_sentinel_lock(tx, company_id)`.
   - `get_or_create_default_in_tx` sur `company_dunning_settings` (crée la row lazy).
   - **Si `settings.seeded_at IS NULL`** : INSERT des **3 niveaux par défaut** — (level 1, delay 10, fee 0.00), (level 2, delay 10, fee 20.00), (level 3, delay 10, fee 40.00) — puis `UPDATE company_dunning_settings SET grace_period_days = 5, seeded_at = NOW(3), version = version + 1 WHERE company_id = ?`.
   - **Si `seeded_at IS NOT NULL`** : **no-op** (déjà seedé au moins une fois — une table `dunning_levels` vide alors = **désactivation volontaire**, PAS de résurrection). Sémantique D7 « vide = désactivé ».
   - Appelée **au premier accès config (GET settings dunning) ET à la première évaluation d'éligibilité** (le hook éligibilité est en 21-5a — cette story fournit juste la fonction + l'appel depuis le GET settings). Idempotente (le sentinel lock sérialise ; deux appels concurrents → un seede, l'autre voit `seeded_at` posé).
   - **Seed LAZY uniquement** (L2 validate P1) : NE PAS ajouter d'appel dans `kesh-seed::seed_demo` — cohérent avec `company_invoice_settings` (lazy, pas de backfill migration). La 1re visite de la page Réglages > Rappels (ou la 1re éligibilité en 21-5a) déclenche le seed. `kesh-seed` reste inchangé par cette story.
10. **Cascade `email_templates`** (`repositories/email_templates.rs`) — étendre la résolution override→défaut d'un seul palier à **quatre** :
    - `get_effective(pool, company_id, template_type, language, level_number)` : tenter override DB `(company, type, langue, N)` → sinon override DB `(company, type, langue, 0)` → sinon `to_effective_default(type, langue, N)` (défaut Rust niveau N ∈ 1..3) → sinon défaut Rust générique. Le champ `level_number` renvoyé reflète la résolution effective.
    - `list_effective_for_company` : la double boucle `ALL × LANGUAGES` (`:129-139`) intègre la dimension niveau **uniquement pour `invoice_reminder`** (`invoice_send` reste au seul niveau 0). ⚠️ **Borne dynamique, PAS statique 0..3** (H4 validate P1) : `dunning_levels` autorise un nombre illimité de niveaux (`create` = MAX+1, sans plafond) et un override `email_templates` peut exister pour un niveau ≥ 4 (route `upsert_override` générique) — un `0..3` codé en dur le rendrait invisible/ingérable côté 21-4. Énumérer `0..=max(niveau_max_configuré, 3)` pour `invoice_reminder`. **Décision figée (H2 validate P2, contre le couplage)** : `list_effective_for_company` **reçoit la borne en paramètre** — signature `list_effective_for_company(pool, company_id, max_reminder_level: i16)` — plutôt que d'interroger `dunning_levels` depuis le repo `email_templates` (garde les deux sous-systèmes découplés ; le repo email_templates ne SELECT pas dunning_levels). L'**appelant** calcule `max_reminder_level` via `dunning_levels::count_for_company`/`MAX(level_number)` puis le passe. ⚠️ **Le SEUL call-site prod actuel est `list_email_templates` (`routes/email_templates.rs:105`, existant Epic 20) — PAS 21-4** (MEDIUM-1 validate P3) : ajouter `max_reminder_level: i16` à la signature **casse la compilation de 21-3** tant que ce handler ne passe pas l'argument. **21-3 DOIT donc modifier `list_email_templates`** pour câbler `dunning_levels::count_for_company`/`MAX(level_number)` → `max(MAX, 3)` et le passer (sinon la page settings Epic 20 perdrait les niveaux configurés dès cette story). `grep -rn list_effective_for_company crates/` pour confirmer l'unicité du call-site prod. Le **0** = générique, `1..=N` = les niveaux configurés (+ défauts Rust 1..3 si pas d'override).
    - `upsert_override`/`restore_default` : threader `level_number` dans les WHERE (`:268` upsert, `:341` restore ; le `:97` est celui de `get_effective` traité au bullet précédent) + INSERT (`:219-221`) + `template_snapshot_json` (`:47-57`).
    - **Rétro-compat** : tous les call-sites existants de `get_effective(InvoiceSend, …)` passent `level_number = 0` (invoice_send est toujours niveau 0). Balayer `grep -rn get_effective` (notamment `invoice_email.rs:216-222`). Le handler poly-type `get_email_template` (`routes/email_templates.rs:121`, path `/{template_type}/{language}` sans segment niveau) passe `level_number = 0` → renvoie le générique/niveau 0 (LOW-3 P3 : acceptable en 21-3, le segment niveau arrivera avec l'UI 21-4).

### Défauts Rust par niveau

11. **`entities/email_template_defaults.rs`** — étendre `default_template` pour porter le niveau. Choix de signature à figer à l'implémentation (recommandé : `default_template(template_type, language, level_number) -> (&'static str, &'static str)` — threader `level_number=0` aux call-sites `invoice_send` existants ; `grep -rn 'default_template(' crates/` pour les recenser — au moins `to_effective_default` `email_templates.rs:71-85` + le test `all_defaults_only_use_allowed_variables`). Contenu :
    - **`invoice_reminder` × niveaux 1-3 × 4 langues** = 12 bras + un **générique** (niveau ≥ 4 ou fallback). **Sujets figés** (D-section) : niveau 1 FR « Rappel de paiement » / DE « Zahlungserinnerung » / IT « Sollecito di pagamento » / EN « Payment reminder » ; niveau 2 FR « 2e rappel » / DE « 2. Mahnung » / IT « 2° sollecito » / EN « Second reminder » ; niveau 3 FR « Dernier rappel avant poursuite » / DE « Letzte Mahnung vor Betreibung » / IT « Ultimo sollecito prima dell'esecuzione » / EN « Final reminder before debt collection » ; générique FR « Rappel de paiement » / DE « Mahnung » / IT « Sollecito » / EN « Payment reminder ».
    - **Corps** : calquer le ton et la structure des défauts `invoice_send` existants (`:22-53`), en escalade de fermeté par niveau, utilisant **exclusivement** les variables déclarées en AC 6 (`{salutation}`, `{contactName}`, `{invoiceNumber}`, `{amount}` [TTC], `{dueDate}`, `{daysOverdue}`, `{totalDue}`, `{reminderFee}`, `{companyName}`). Le **corps FR canonique** de chaque niveau est fixé dans les Dev Notes ci-dessous ; DE/IT/EN reflètent la même structure/variables au standard de traduction des défauts `invoice_send` (fiduciaire suisse). Le test `all_defaults_only_use_allowed_variables` (`email_templates_repository` zone `:63-79`) couvrira automatiquement la cohérence tokens dès que `InvoiceReminder ∈ ALL`.

### Routes & RBAC

12. **`crates/kesh-api/src/routes/dunning_levels.rs`** (calque `routes/vat.rs`) : DTOs camelCase `DunningLevelResponse` (**sans `company_id`**), `CreateDunningLevelBody { delay_days, fee_amount }`, `UpdateDunningLevelBody { delay_days, fee_amount, version }`, `DeleteDunningLevelBody { version }`. Handlers : `list` (tout rôle auth) ; `create`/`update`/`delete` (Admin) ouvrant `pool.begin()` → **sentinel lock EN PREMIER** → validation forme (fee 0..10000 + `scale_within(fee,2)`, delay ≥ 0) → mutation repo → `audit_log::insert_in_tx` (`dunning_level.created`/`.updated`/`.deleted`, `NewAuditLogEntry::for_actor` avec `api_key_id`) → `tx.commit()`.
13. **`crates/kesh-api/src/routes/company_dunning_settings.rs`** (calque `routes/company_invoice_settings.rs`) : `GET` (tout rôle auth) — **séquence tx explicite** (M1 validate P2) : `let mut tx = pool.begin()` → `ensure_seeded_in_tx(&mut tx, company.id)` (AC 9 — prend le sentinel lock, crée la row settings via `get_or_create_default_in_tx`, seede les 3 niveaux si `seeded_at IS NULL`) → `let settings = company_dunning_settings::get_or_create_default_in_tx(&mut tx, company.id)` (relit l'état post-seed) → `tx.commit()` → réponse. (Alternative : `ensure_seeded_in_tx` retourne directement les settings — au choix du dev, mais UNE seule tx.) `PUT` (Admin) → validation `grace_period_days ≥ 0` → `update(pool, company.id, req.version, user_id, changes)`. Réponse expose `version` + `seededAt`.
14. **Montage routeur + RBAC** (`lib.rs`) — anti-footgun (route montée après le `;` d'un `.route_layer` = bypass auth silencieux) :
    - **Mutations** `dunning_levels` (POST/PUT/DELETE) + **PUT** `company_dunning_settings` → sous-routeur `admin_routes` (protégé `require_admin_role`, `lib.rs:250-252` zone).
    - **GET** `dunning_levels` + **GET** `company_dunning_settings` → sous-routeur authentifié (tous rôles, `lib.rs:537`/`:590` zone).
    - Paths proposés : `/api/v1/dunning-levels` (+ `/{id}`), `/api/v1/company/dunning-settings`.

### Export souveraineté & backup

15. **Backup admin (auto-synchronisé au schéma — sinon test rouge immédiat)** : ajouter `dunning_levels` et `company_dunning_settings` à `crates/kesh-db/src/backup.rs` `TABLES_TO_TRUNCATE` (ordre FK enfants→parents : `dunning_levels` avant `companies` ; `company_dunning_settings` avant `companies`). Le test `backup_inventory_matches_schema` (`backup.rs:579-602`) échoue tant que ce n'est pas fait. Incrémenter le compteur e2e `crates/kesh-api/tests/admin_full_export_e2e.rs:274` (`34 → 36`).
16. **Export souveraineté `global.zip`** (D26 — les nouvelles tables entrent dans l'export ; `invoice_reminders` viendra en 21-5a) : ajouter `dunning_levels` + `company_dunning_settings` à la macro `push_csv!` (`crates/kesh-api/src/exports/global.rs:204-249`, insérer après `company_invoice_settings` pour un ordre logique — L3 validate P2), les serializers (`exports/csv_tables.rs`) + queries repo. **Contrairement au backup, l'export global n'est PAS auto-vérifié** — chaque compteur/liste est à mettre à jour à la main (H2/H3 validate P1) :
    - `global.rs:258-259` : **les deux** `debug_assert_eq!(files.len(), 16 …)` **et** `debug_assert_eq!(tables_meta.len(), 16 …)` → `18` (⚠️ pas `:255-256`).
    - `global.rs:183` : `Vec::with_capacity(17)` → `19`.
    - `exports_global_e2e.rs:619` : `assert_eq!(entries.len(), 17, …)` → `19` (⚠️ `:607` est la déclaration `fn`, l'assertion est en `:619`).
    - `exports_global_e2e.rs:622` : le `HashSet expected` littéral des noms de fichiers → ajouter `"dunning_levels.csv"` + `"company_dunning_settings.csv"` (sinon `assert_eq!(names, expected)` échoue même le compte corrigé).
    - `exports_global_e2e.rs:693` : `assert_eq!(tables.len(), 16, "expected 16 tables in metadata")` → `18` (**3e compteur, omis de la 1re rédaction**).
    - `exports_global_e2e.rs:775` : le `BTreeMap<&str, u64> expected` des rowCounts → ajouter les 2 nouvelles tables avec `0` (couverture, sinon test vert mais non couvrant).
    - **`global.rs:274` `csv_count: 16` → `18`** (littéral en dur exposé dans `metadata.json` de l'export — pièce juridique D26) + **`exports_global_e2e.rs:1055` `assert_eq!(details["csv_count"]…, 16)` → `18`** (HIGH-2 validate P3 — même classe de compteur oublié que H2/H3 : sans ce patch le `metadata.json` mentirait 16 vs 18, ou le test passe rouge).
    - **Doc-drift** (LOW-2) : mettre à jour les commentaires « 16 tables/SELECTs/CSV » de `global.rs` (`:15, :4/6/13, :122, :180-181`) → 18 (pas de test rouge mais la discipline projet réprouve la dérive doc).

### Tests

17. **Repo `crates/kesh-db/tests/dunning_levels_repository.rs`** (`#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, calque `vat_rates_repository.rs`) : create append (level=MAX+1), update optimistic-conflict (version stale), **delete + renumérotation** (supprimer niveau 2 sur 3 → l'ex-niveau 3 devient 2, contiguïté préservée), **`version` de la ligne renumérotée bumpée** (H1 : lire l'ex-niveau 3 avant suppression du niveau 2, supprimer, relire → `version` a augmenté ET un `update_for_company` avec l'ancien `expected_version` est rejeté `OptimisticLockConflict`), borne `fee_amount` (CHECK 0..10000 rejette 10000.01 et négatif), isolation cross-tenant.
18. **Repo `crates/kesh-db/tests/company_dunning_settings_repository.rs`** : get-or-create implicite, idempotence, no-op (grace inchangé → version stable + 0 audit), version-bump + 1 audit, version-mismatch → conflict, **`seeded_at` NULL vs non-NULL** (le seed pose `seeded_at` ; un 2e appel ne re-seede pas ; vider `dunning_levels` puis re-appeler seed ⇒ **reste vide**, pas de résurrection).
19. **Seed** : test `ensure_seeded_creates_three_levels_and_stamps_seeded_at` + `ensure_seeded_is_idempotent` (2 appels → toujours 3 niveaux, `seeded_at` inchangé) + `ensure_seeded_does_not_resurrect_after_manual_empty`.
20. **Cascade email_templates** (étendre `email_templates_repository.rs` + `email_templates_e2e.rs`) : `get_effective(invoice_reminder, FR, level=2)` retombe sur défaut Rust niveau 2 quand aucun override ; override DB niveau 2 gagne sur override niveau 0 ; override niveau 0 gagne sur défaut Rust ; **UNIQUE `(company, type, langue, level)`** rejette un doublon exact mais accepte le même `(type, langue)` à des niveaux différents (adapter `unique_constraint_rejects_duplicate_row:397`). Ajuster les assertions « quatre défauts » (`assert_eq!(list.len(), 4)` en `email_templates_repository.rs:94`, fn `list_effective_returns_four_defaults_for_fresh_company:87` ; e2e `assert_eq!(…, 4)` en `email_templates_e2e.rs:293`, fn `:278`) au nouveau produit cartésien : **cas zéro-config = 20 templates effectifs** = 4 langues × (1 `invoice_send` niveau 0 + 4 `invoice_reminder` niveaux 0-3). Ajouter un test `list_effective_exposes_configured_level_above_3` (créer un niveau 4 + son override → il apparaît dans la liste, preuve de la borne dynamique H4). `all_defaults_only_use_allowed_variables` doit passer avec les 12+ nouveaux défauts.
21. **E2E `crates/kesh-api/tests/dunning_levels_e2e.rs`** + `company_dunning_settings_e2e.rs` (calque `vat_rates_e2e.rs`) : happy Admin, **non-Admin → 403** (Comptable), **cross-tenant → 404** (jamais 403 — anti-énumération KF-002), verrou optimiste → 409, GET tous rôles (Consultation 200). GET settings sur company fraîche → 3 niveaux seedés visibles.

### Doc & gate

22. **`CHANGELOG.md`** section `[Non publié]` : entrée `Added` (socle rappels : niveaux configurables + grâce + type e-mail `invoice_reminder` par niveau). Pas de manuel utilisateur ici (les réglages visibles arrivent en 21-4 ; la doc manuel/CGV est en 21-8). **Aucun frontend** (21-4).
23. **Quality gate Test Locally First backend complet** vert (fmt/clippy/build + `cargo test` ; ou `scripts/test-fast.sh`). Attention KF-038 (#228) flake pré-existant réconciliation sous contention — ne pas confondre avec une régression.

## Tasks / Subtasks

- [x] **T1 — Migrations** (AC 1-3) : `20260714000001_dunning_config.sql` (2 tables) + `20260714000002_email_templates_reminder.sql` (ALTER + bump `min_required='0.7.0'`) + compteur `migrations_upgrade_path` 51→53 + idempotence audit (2 lignes + stats 51→53, tracked-by-sqlx 40→42) + `migrations_fresh_install` (2 tables + sweep min_required). **Piège résolu** : erreur 1553 (FK a besoin de l'index UNIQUE `company_id`) → créer le nouvel UNIQUE AVANT de dropper l'ancien. Gate : `migrations_fresh_install` 3/3 + `migrations_upgrade_path` 8/8.
- [ ] **T2 — Entités** (AC 4-6) : `dunning_level.rs`, `company_dunning_settings.rs`, `EmailTemplateType::InvoiceReminder` (6 sites) + `level_number` sur `EmailTemplate` + mod.rs ré-exports.
- [ ] **T3 — Repos dunning** (AC 7-8) : `dunning_levels.rs` (list/create-append/update/delete-renumber/count, sentinel lock) + `company_dunning_settings.rs` (get-or-create miroir + update no-op/optimistic/audit).
- [ ] **T4 — Seed idempotent** (AC 9) : `ensure_seeded_in_tx` sous sentinel lock, 3 niveaux + grâce 5 + `seeded_at`, sémantique « vide = désactivé ».
- [ ] **T5 — Cascade + défauts email** (AC 10-11) : cascade `get_effective`/`list_effective`/`upsert`/`restore` + `default_template` 12 bras + générique + rétro-compat `level_number=0` sur invoice_send.
- [ ] **T6 — Routes + RBAC** (AC 12-14) : `dunning_levels.rs` + `company_dunning_settings.rs` + montage `admin_routes`/authentifié + audits.
- [ ] **T7 — Export/backup** (AC 15-16) : `TABLES_TO_TRUNCATE` +2 + compteur backup e2e 34→36 + `push_csv!` +2 + serializers/queries + debug_assert 16→18 + exports_global_e2e 17→19.
- [ ] **T8 — Tests** (AC 17-21) : repos + seed + cascade + e2e RBAC/IDOR/verrou.
- [ ] **T9 — Doc + gate** (AC 22-23) : CHANGELOG + Test Locally First.

## Dev Notes

### Pièges identifiés (ground-truth 2026-07-13, 4 agents Explore)

- **Sentinel lock = `bank_accounts::acquire_company_sentinel_lock(tx, company_id)`** (`repositories/bank_accounts.rs:588`) — `SELECT id FROM companies WHERE id = ? FOR UPDATE`. La sentinelle est la row `companies` du tenant (pas la table `dunning_levels`). À prendre **en tout début de handler mutant, avant tout autre SELECT/mutation**. **NE PAS** créer un nouveau helper — réutiliser (comme `vat_rates`, `projects`, `users`).
- **`dunning_levels` ≠ `vat_rates` sur la suppression** : `vat_rates` fait du **soft-delete** (`active=FALSE`, jamais de hard-delete, car un taux a servi des factures historiques). `dunning_levels` fait du **hard-delete + renumérotation** (D5) car l'historique est protégé par les snapshots `invoice_reminders` (21-5a). Ne pas copier le soft-delete de vat_rates.
- **`seeded_at` est une nouveauté SANS précédent** dans le codebase (les discriminants de bootstrap existants sont des booléens : `companies.is_stub`, `onboarding_state.is_demo` — pas des timestamps). L'`INSERT IGNORE` seul **ne distingue pas** « row jamais créée » de « row vidée volontairement » → c'est `seeded_at` qui porte cette sémantique, **entièrement dans la logique du repo** : posé `NOW(3)` uniquement dans le chemin seed, laissé `NULL` par le `INSERT IGNORE` lazy de `get_or_create_default`.
- **Migration breaking = les DONNÉES, pas le DDL** : `ADD COLUMN level_number DEFAULT 0` et l'ALTER UNIQUE sont compatibles avec un binaire v0.6 (il ne SELECT/INSERT pas `level_number`). C'est l'apparition d'une valeur `template_type='invoice_reminder'` qui casse le `FromStr` strict du binaire downgradé → 500 sur `list_effective_for_company`. D'où le bump `kesh_version_min_required='0.7.0'` (P2/D14). **Aucun bump n'existe encore dans le repo** (`min_required` est resté `'0.1.0'`) — ce sera le 1er ; le garde-fou P3 code-review le validera.
- **Compteur migrations divergent entre agents** (47 vs 51) : source de vérité = `ls crates/kesh-db/migrations/*.sql | wc -l` = **51** + `grep -nF 'assert_eq!' crates/kesh-db/tests/migrations_upgrade_path.rs`. **Vérifier la valeur exacte au dev** avant de l'incrémenter à 53.
- **`allowed_variables()` déclare les variables de rappel MAINTENANT** (`reminderLevel`, `reminderFee`, `totalDue`, `daysOverdue`) même si leur builder `build_reminder_vars` arrive en **21-5b** — sinon `validate_tokens` refuserait les corps par défaut qui les utilisent.
- **Rétro-compat cascade** : tous les `get_effective(InvoiceSend, …)` existants doivent passer `level_number = 0`. `grep -rn get_effective crates/` — notamment `invoice_email.rs:216-222` (Epic 20). Ne pas casser l'envoi de factures.
- **Backup auto-vérifié** : `backup_inventory_matches_schema` (`backup.rs:579`) compare `TABLES_TO_TRUNCATE` à `information_schema` → **toute nouvelle table oubliée = test rouge immédiat**. C'est le filet (vs export global, non auto-vérifié, à ne pas oublier manuellement).

### Patterns à réutiliser (ne PAS réinventer)

- **Collection + sentinel lock** : `repositories/vat_rates.rs` (create/update/deactivate prenant `&mut Transaction`, double garde version, audit `insert_in_tx` dans la tx). Routes : `routes/vat.rs` (DTO Response sans `company_id`, validation forme pure `validate_rate`/`scale_within`, sentinel lock en tête de handler).
- **Singleton get-or-create** : `repositories/company_invoice_settings.rs` (INSERT IGNORE + SELECT, variantes pool/tx en miroir avec marqueur `MIRROR`, no-op KF-004 `is_no_op_change`, verrou optimiste, audit). Routes : `routes/company_invoice_settings.rs` (GET auth + PUT admin séparés).
- **Sous-système templates** : `entities/email_template.rs` (enum + `ALL`/`as_str`/`allowed_variables`/`FromStr`) + `entities/email_template_defaults.rs` (`default_template` `match → (&'static str, &'static str)`) + `repositories/email_templates.rs` (`get_effective`/`list_effective_for_company`/`upsert_override`/`restore_default`) + `routes/email_templates.rs`. Moteur `kesh-core/src/email_template_engine.rs` **agnostique au niveau — ne rien y toucher**.
- **Seed sous lock** : patron `kesh-seed/src/lib.rs:70-160` (`seed_demo`, lock-and-validate `SELECT … FOR UPDATE` en tx courte). Ici adapter à la clé company + au discriminant `seeded_at`.
- **RBAC** : `middleware::rbac::require_admin_role`/`require_comptable_role` (`middleware/rbac.rs:31-40`), enum `Role` (`Consultation<Comptable<Admin`, `entities/user.rs:20-24`). Montage : mutations dans `admin_routes` (`lib.rs:250-252`), lectures dans le routeur authentifié.
- **Audit** : `NewAuditLogEntry::for_actor(user_id, api_key_id, action, entity_type, entity_id, Some(details_json))` (routes) ou `::user(...)` (repos type company_invoice_settings) via `audit_log::insert_in_tx` **dans la même tx**.
- **Erreurs → HTTP** : `DbError::NotFound → 404`, `DbError::OptimisticLockConflict → 409` (`errors.rs:1862-1875`), `AppError::Validation → 4xx`, `403` via middleware. Cross-tenant → **404** (anti-énumération KF-002), jamais 403.

### Corps FR canoniques des défauts de rappel (à traduire DE/IT/EN au même standard)

> Structure calquée sur les défauts `invoice_send` (`email_template_defaults.rs:22-53`). Variables autorisées uniquement (AC 6). Ton en escalade. À figer/affiner en validate ; DE/IT/EN suivent la même trame.

- **Niveau 1** (sujet « Rappel de paiement ») : salutation, rappel courtois que la facture `{invoiceNumber}` de `{amount}` échue le `{dueDate}` (depuis `{daysOverdue}` jours) reste ouverte, invitation à régler le montant dû `{totalDue}`, formule de politesse, `{companyName}`.
- **Niveau 2** (sujet « 2e rappel ») : ton plus ferme, mention des frais de rappel `{reminderFee}` inclus dans `{totalDue}`, délai bref attendu.
- **Niveau 3** (sujet « Dernier rappel avant poursuite ») : mise en demeure, mention qu'à défaut de paiement une procédure de recouvrement sera engagée, `{totalDue}` exigible, coordonnées `{companyName}`.
- **Générique** (niveaux ≥ 4) : neutre-ferme, rappel de la facture ouverte et du montant dû.

### Environnement de test (rappels projet)

- `kesh-db` en série pour les tests d'intégration DB (`cargo test -p kesh-db -- --test-threads=1`), ou gate rapide `scripts/test-fast.sh` (nextest, lib kesh-db sérialisé).
- `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` = DB éphémère isolée par test — parallélisable.

### Hors scope (garde-fous anti-creep)

- **AUCUNE évaluation d'éligibilité, liste à rappeler, `invoice_reminders`, `dunning_paused_at`** → 21-5a.
- **AUCUN envoi, `build_reminder_vars`, preview, lot** → 21-5b (cette story ne fait que *déclarer* les variables de rappel dans `allowed_variables`).
- **AUCUN frontend** (page settings/dunning, refactor email-templates multi-type/multi-niveau) → 21-4.
- **AUCUN réordonnancement de niveaux** (drag-drop) — `level_number` immutable en update ; create append / delete renumber seulement.
- **Balance âgée** → 21-7.

### Dérogation / watch-point règle de splitting (CLAUDE.md)

Cette story touche ~6 modules (`kesh-db/{entities,repositories,migrations,backup}`, `kesh-api/{routes,exports,lib}`), à la limite haute du critère « > 5 modules ». Le plan d'epic (critique 3 agents) l'a néanmoins scopée comme story unique, contrairement à 21-5a/21-5b pré-splittées. **Décision** : suivre le plan (les deux briques — tables dunning et extension email_templates — sont thématiquement « le socle config backend »), MAIS **si `bmad-create-story validate` dépasse 4 passes sans converger (critère 2), splitter en 21-3a (tables dunning + settings + seed + export/backup) / 21-3b (email_templates : type `invoice_reminder` + `level_number` + cascade + défauts)** — les deux sont indépendantes (aucun cycle Cargo, chacune testable en isolation) et toutes deux prérequises de 21-4/21-5b. Précédent : 21-2 splittée après échec de convergence en 4 passes.

### Project Structure Notes

- Nouveaux fichiers : `migrations/20260713000001_dunning_config.sql`, `migrations/20260713000002_email_templates_reminder.sql`, `entities/dunning_level.rs`, `entities/company_dunning_settings.rs`, `repositories/dunning_levels.rs`, `repositories/company_dunning_settings.rs`, `routes/dunning_levels.rs`, `routes/company_dunning_settings.rs`, tests repo+e2e associés.
- Fichiers modifiés : `entities/{email_template.rs, email_template_defaults.rs, mod.rs}`, `repositories/email_templates.rs`, **`routes/email_templates.rs`** (DTO `EmailTemplateResponse` + ses 2 `impl From` `:44`/`:58` avec `level_number` ; call-site `list_email_templates:105` passe `max_reminder_level` — LOW-1/MEDIUM-1 P3), `backup.rs`, `exports/{global.rs, csv_tables.rs}`, `lib.rs`, `docs/migrations-idempotence-audit.md`, `tests/{migrations_upgrade_path.rs, migrations_fresh_install.rs}` (assertions `min_required` post-bump — HIGH-1), `admin_full_export_e2e.rs`, `exports_global_e2e.rs` (compteurs `:619/:693/:775/:1055`), `email_templates_repository.rs`, `email_templates_e2e.rs`, `CHANGELOG.md`.

### References

- [Source: _bmad-output/planning-artifacts/epic-21-echeances-relances.md#Décisions figées — D5, D6, D7, D14, section D (option A+), D22, D26]
- [Source: crates/kesh-db/migrations/20260428000001_vat_rates.sql + 20260613000001_vat_rates_crud.sql — DDL collection]
- [Source: crates/kesh-db/migrations/20260417000001_invoice_validation.sql:35-51 — DDL singleton]
- [Source: crates/kesh-db/migrations/20260708000001_email_templates.sql:29-50 — DDL email_templates + CHECK:42-43 + UNIQUE:48-49]
- [Source: crates/kesh-db/src/repositories/{vat_rates.rs, company_invoice_settings.rs, email_templates.rs}]
- [Source: crates/kesh-db/src/entities/{vat_rate.rs, company_invoice_settings.rs, email_template.rs, email_template_defaults.rs}]
- [Source: crates/kesh-db/src/repositories/bank_accounts.rs:588 — acquire_company_sentinel_lock]
- [Source: crates/kesh-api/src/routes/{vat.rs, company_invoice_settings.rs, email_templates.rs} + lib.rs (montage RBAC:250-252,537,590)]
- [Source: crates/kesh-db/src/backup.rs:34-69,579-602 — TABLES_TO_TRUNCATE + inventory test]
- [Source: crates/kesh-api/src/exports/global.rs:183,204-249,255-256 — push_csv + compteurs]
- [Source: CLAUDE.md — Migration breaking policy P1-P5, Règle de splitting préventif]

## Dev Agent Record

### Agent Model Used

Opus 4.8 (1M context) — run 2026-07-14.

### Debug Log References

- **T1** : `sqlx::migrate!()` n'a pas détecté les nouveaux `.sql` au 1er run (macro embarquée à la compilation) → `touch crates/kesh-db/src/lib.rs` force le rebuild du MIGRATOR. Puis erreur MariaDB 1553 « Cannot drop index uq_email_templates_company_type_language: needed in a foreign key constraint » → réordonné migration B pour créer le nouvel UNIQUE (préfixe `company_id`) AVANT de dropper l'ancien.

### Completion Notes List

### File List

## Change Log

### Validate Pass 1 (2026-07-13, Sonnet 4.6) — 4 HIGH + 2 MEDIUM + 2 LOW, patchés

Passe adversariale avec grep ground-truth (~35 refs `fichier:ligne` vérifiées, ~30/33 exactes). Split 21-3a/21-3b **non** recommandé (angles morts localisés, pas un problème de taille). Findings remédiés :
- **H1** (défaut de conception) — la renumérotation `delete_and_renumber` ne bumpait pas `version` → verrou optimiste contourné pour les lignes déplacées. **Patch** : `UPDATE … level_number = level_number - 1, version = version + 1 …` (AC 7) + test dédié (AC 17).
- **H2** — compteurs `exports_global_e2e.rs` incomplets : l'assertion est en `:619` (pas `:607` = décl. fn), + le `HashSet expected` (`:622`), + un **3e compteur oublié** `assert_eq!(tables.len(), 16)` (`:693`), + le `BTreeMap` rowCounts (`:775`). **Patch** : AC 16 liste les 4 sites (grep-confirmés).
- **H3** — `global.rs` `debug_assert_eq!` en `:258-259` (pas `:255-256`), les **deux** (`files.len()` + `tables_meta.len()`). **Patch** : réf corrigée (grep-confirmé).
- **H4** — produit cartésien `list_effective_for_company` sous-spécifié : un `0..3` statique masquerait un override de niveau ≥ 4 (niveaux illimités). **Patch** : borne dynamique `0..=max(niveau_max_configuré, 3)` (AC 10) + compte zéro-config figé = 20 templates + test niveau > 3 (AC 20).
- **M1** — renvoi croisé « AC 9 » (seed) au lieu de « AC 12 » (handler create) dans AC 7. **Patch** : corrigé.
- **M2** — subsumé par H2 (`:607` = décl. fn). 
- **L1** — `reminderLevel` déclarée non utilisée par les défauts v1. **Patch** : note intentionnelle (AC 6).
- **L2** — seed lazy vs `kesh-seed` démo non tranché. **Patch** : Dev Note « seed LAZY uniquement, kesh-seed inchangé » (AC 9).

**Trend** : Pass 1 → 4 HIGH / 2 MEDIUM (> LOW) → patchés. Relance Pass 2 (LLM différent, contexte frais).

### Validate Pass 2 (2026-07-14, Haiku 4.5, contexte frais) — 2 HIGH + 2 MEDIUM + 3 LOW, patchés

Passe adversariale (régressions Pass 1 + angles morts). Les 2 HIGH Haiku **vérifiés par grep ground-truth** (discipline anti-faux-positif Haiku) avant remédiation — les deux **réels** :
- **H1** (grep confirmé : `EffectiveEmailTemplate:118-127` n'a pas `level_number`) — le Pass 1 avait raté que la struct de retour de `get_effective` doit porter le niveau résolu (AC 10 l'exige). **Patch** : `level_number` ajouté à `EffectiveEmailTemplate` + helpers `to_effective_*` + DTO `EmailTemplateResponse` (AC 6).
- **H2** — méthode de la borne dynamique `list_effective_for_company` laissée ambiguë (2 options). **Patch** : tranché — l'**appelant** passe `max_reminder_level: i16` (découple le repo email_templates de dunning_levels), pas de SELECT cross-table dans le repo (AC 10).
- **M1** — ordre des appels du handler GET seed non explicite. **Patch** : séquence tx figée `begin → ensure_seeded_in_tx → get_or_create_default_in_tx → commit` (AC 13).
- **M2** — signature `default_template` : call-sites à recenser. **Patch** : hint `grep -rn 'default_template('` (AC 11).
- **L1** (compteur migrations 53 confirmé — non-finding), **L2** (texte commentaire breaking — déjà demandé AC 2, no-op), **L3** (ordre insertion global.rs → après `company_invoice_settings`, AC 16).

**Trend** : Pass 1 (4 HIGH/2 MED) → Pass 2 (2 HIGH/2 MED, dont H1 raté par Sonnet) → patchés. Relance Pass 3 (Opus, contexte frais). Split toujours non recommandé (2 passes, angles morts localisés).

### Validate Pass 3 (2026-07-14, Opus 4.8, contexte frais) — 2 HIGH + 1 MEDIUM + 3 LOW, patchés

Passe de scellage, ~25 vérifs grep ground-truth (toutes les refs Pass 1/2 re-confirmées exactes ; bump 0.7.0 protège au boot via `version.rs:230`, cohérent). Findings > LOW remédiés — **les 2 HIGH ratés par Pass 1 ET Pass 2**, tous deux gates rouges garantis :
- **HIGH-1** (grep confirmé) — le **1er bump `min_required='0.7.0'` du repo** casse des tests qui hard-codent `'0.1.0'` : `migrations_fresh_install.rs:238`, `downgrade_protection_aligned_when_binary_equals_min:398`, `downgrade_protection_binary_ahead:411` (assert `db_min=="0.1.0":416`). **Patch** : AC 3 impose le balayage complet (`grep '"0.1.0"'` + `check_downgrade_protection`, traiter chaque occurrence sémantiquement).
- **HIGH-2** (grep confirmé) — AC 16 omettait `global.rs:274 csv_count:16` (littéral exposé dans `metadata.json`, pièce juridique) + `exports_global_e2e.rs:1055`. **Patch** : ajoutés à AC 16 + doc-drift.
- **MEDIUM-1** — le seul call-site prod de `list_effective_for_company` est `list_email_templates:105` (existant Epic 20), PAS 21-4 → le changement de signature H2-P2 casse la compile de 21-3 s'il n'est pas câblé ici. **Patch** : AC 10 impose de modifier `list_email_templates` (câbler `dunning_levels::count` → max(MAX,3)) + File List.
- **LOW-1** File List += `routes/email_templates.rs` ; **LOW-2** doc-drift global.rs ; **LOW-3** `get_email_template` poly-type `level=0`. + clarification sémantique `level_number` retourné = SLOT demandé (pas source cascade).

**Trend** : P1 (4H/2M) → P2 (2H/2M) → P3 (2H/1M) → patchés. Relance Pass 4 (retour Sonnet, cycle). ⚠️ Pass 4 = seuil règle de splitting (critère 2 : > 4 passes) : si Pass 4 ne converge pas → split 21-3a/21-3b (frontière naturelle : HIGH-1/MED-1 côté email_templates, HIGH-2 côté export). Opus : findings mécaniques/localisés, split non requis à ce stade.

### Validate Pass 4 (2026-07-14, Sonnet 4.6, contexte frais) — CONVERGÉ (0 > LOW)

Passe de convergence. Vérifications grep ground-truth ciblées sur les patches P3 (les moins re-vérifiés) :
- **HIGH-1** (balayage `min_required`) : liste AC 3 **exactement complète** — les 6 call-sites `check_downgrade_protection` audités ; `:298/:321/:347` indépendants de la version, `:381` écrase `min_required` en dur (reste vert), seuls `:400 aligned` et `:413 binary_ahead` cassent (couverts). `:281 last_applied` non affecté (colonne distincte). ✅
- **HIGH-2** (`csv_count`) : tous les sites `16/17` liés aux tables exportées couverts (`global.rs:183/258/259/274` + `exports_global_e2e.rs:619/622/693/775/1055`), aucun oublié. ✅
- **MEDIUM-1** : `list_effective_for_company` a bien un **unique** call-site prod (`:105`), l'autre est un test. Patch cohérent. ✅
- Compteurs mutuellement cohérents (migrations 53, backup 36, export 18/19, csv_count 18), 6 sites enum + WHERE/INSERT email_templates tous grep-confirmés exacts, décisions epic D5/D6/D7/D14/section D toutes couvertes.
- **2 LOW** (imprécisions de pointeur de ligne AC 10 `:97` attribué à get_effective, AC 20 assertions `:94`/`:293` vs décl. fn) → **corrigés** (nits, aucun risque fonctionnel).

**Trend final** : P1 (4H/2M) → P2 (2H/2M) → P3 (2H/1M) → **P4 (0 > LOW) CONVERGÉ**. LLM : Sonnet→Haiku→Opus→Sonnet (cycle complet). Split **non déclenché** (convergence à Pass 4, critère 2 « > 4 passes » non atteint ; findings toujours mécaniques/localisés). **Spec prête pour `bmad-dev-story`.**
