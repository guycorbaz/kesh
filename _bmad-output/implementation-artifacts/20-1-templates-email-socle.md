# Story 20.1: Socle templates d'e-mail (backend) — table, moteur `{var}`, CRUD Admin

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

En tant qu'administrateur d'une PME utilisant Kesh,
je veux que le backend expose un sous-système de templates d'e-mail **company-scoped**, multilingue (FR/DE/IT/EN), pour le type `invoice_send` — avec moteur de substitution `{var}` sûr, validation au save, verrou optimiste, audit trail et CRUD Admin —,
afin que je puisse (Story 20-2, UI Admin) personnaliser le contenu des e-mails envoyés à mes clients sans dépendre d'un déploiement de code, avec un fallback zéro-config qui fonctionne dès le premier jour.

Cette story est le **story-zéro** de l'Epic 20 (#224). Elle ne touche NI le mailer (`crates/kesh-api/src/mail/`, réservé à 20-3b) NI le frontend (réservé à 20-2) NI `contacts` (`language`/`salutation`, réservé à 20-3b) NI l'envoi de facture. Elle livre uniquement le socle backend : table, moteur, validation, defaults, CRUD Admin.

## Acceptance Criteria

**Table & schéma**

1. Migration `crates/kesh-db/migrations/20260708000001_email_templates.sql` crée la table `email_templates` : `id BIGINT AUTO_INCREMENT PK`, `company_id BIGINT NOT NULL` + `FOREIGN KEY ... REFERENCES companies(id) ON DELETE CASCADE`, `template_type VARCHAR(50) NOT NULL` (**pas** `type` — convention projet `_type` suffixe, cf. `contact_type`/`org_type`/`account_type`), `language CHAR(2) NOT NULL`, `subject TEXT NOT NULL`, `body TEXT NOT NULL`, `version INT NOT NULL DEFAULT 1`, `created_at`/`updated_at DATETIME(6)`. `CHECK` sur `template_type IN ('invoice_send')` (élargissable par migration future — non-breaking). `CHECK (BINARY language IN (BINARY 'FR', BINARY 'DE', BINARY 'IT', BINARY 'EN'))` (calqué `chk_companies_instance_language`). `UNIQUE (company_id, template_type, language)`. Index `company_id`. Migration **non-breaking** (ADD TABLE) → pas de bump `kesh_version_min_required` (P3).
2. `docs/migrations-idempotence-audit.md` reçoit une ligne pour cette migration (verdict `tracked-by-sqlx`, justification : nouvelle table, non-breaking) — garde-fou P5.

**Types & moteur de substitution**

3. `EmailTemplateType` (enum, aujourd'hui : `InvoiceSend` seul) est défini dans `kesh-db/src/entities/email_template.rs`, avec `#[derive(Serialize, Deserialize)]` `#[serde(rename_all = "snake_case")]` (pour que `templateType` sérialise/désérialise en `"invoice_send"` dans les DTOs JSON, symétrique de `Language`'s `#[serde(rename_all = "UPPERCASE")]`) + `as_str()`/`FromStr` (calqués `Language`/`OrgType` de `company.rs`) + implémentation `sqlx::Type<MySql>`/`Encode`/`Decode` (même pattern exact que `Language`/`OrgType`, lignes 45-68 et 110-133 de `company.rs`). **Ne PAS** créer un nouvel enum `Language` — réutiliser `kesh_db::entities::Language` (FR/DE/IT/EN) tel quel pour la colonne `language`.
4. `EmailTemplateType::allowed_variables(&self) -> &'static [&'static str]` retourne, pour `InvoiceSend` : `["salutation", "contactName", "invoiceNumber", "amount", "dueDate", "companyName"]`.
5. Le moteur de substitution `{var}` vit dans `kesh-core/src/email_template_engine.rs` (crate **pure**, aucune dépendance DB — `kesh-core` ne dépend pas de `kesh-db`, cf. Dev Notes) :
   - `extract_tokens(text: &str) -> Vec<String>` — scan single-pass des séquences `{nom}`, retourne les noms dédupliqués dans l'ordre d'apparition.
   - `validate_tokens(subject: &str, body: &str, allowed: &[&str]) -> Result<(), Vec<String>>` — `Err(unknown)` listant les tokens de `subject`+`body` qui ne sont PAS dans `allowed` (dédupliqués).
   - `render(template: &str, vars: &HashMap<String, String>) -> String` — substitution single-pass : un token connu (`vars.contains_key`) est remplacé par sa valeur ; un token inconnu est laissé **littéral** (jamais d'erreur — rendu infaillible). **Invariant critique anti-injection** : la fonction ne réanalyse JAMAIS le texte déjà produit — si une valeur substituée contient elle-même une séquence `{token}`, cette séquence n'est PAS interprétée (le scan avance strictement sur le template source, jamais sur le résultat).
6. Tests unitaires `kesh-core` couvrant : substitution simple, token inconnu laissé littéral, dédup de `extract_tokens`, non-double-substitution (valeur contenant `{token}` non ré-interprétée), tokens sensibles à la casse (`{Amount}` ≠ `{amount}`).

**Defaults**

7. Textes par défaut FR/DE/IT/EN pour `invoice_send` en **constantes Rust** (pas de FTL — Fluent ignore silencieusement les variables inconnues, incompatible avec la validation stricte, cf. `loader.rs:14-19`). Module `kesh-db/src/entities/email_template_defaults.rs` (ou co-localisé dans `email_template.rs`) : `fn default_template(template_type: EmailTemplateType, language: Language) -> (&'static str, &'static str)` (subject, body). Chaque texte par défaut utilise exclusivement des tokens ∈ `allowed_variables()` de son type (auto-cohérence testée, cf. AC #8).
8. Test unitaire garantissant qu'AUCUN texte par défaut ne contient de token hors de `allowed_variables()` (évite qu'un futur défaut cassé ne soit jamais détecté avant le rendu réel en 20-3b).

**Repository (résolution override→défaut, verrou optimiste, audit, no-op)**

9. `kesh-db/src/repositories/email_templates.rs` expose :
   - `get_effective(pool, company_id, template_type, language) -> Result<EffectiveEmailTemplate, DbError>` — retourne la ligne override si présente, sinon synthétise depuis les defaults. `EffectiveEmailTemplate { template_type, language, subject, body, version: Option<i32>, is_default: bool, allowed_variables: Vec<String> }` (`version = None` quand `is_default = true`).
   - `list_effective_for_company(pool, company_id) -> Result<Vec<EffectiveEmailTemplate>, DbError>` — un élément par combinaison `EmailTemplateType::ALL × Language::ALL` (4 en v1 : 1 type × 4 langues).
   - `upsert_override(pool, company_id, template_type, language, expected_version: Option<i32>, user_id, subject, body) -> Result<EmailTemplate, DbError>` — `expected_version = None` signifie "je crois qu'aucun override n'existe" → `INSERT` ; si une ligne existe déjà (race) → `DbError::OptimisticLockConflict`. `expected_version = Some(v)` signifie "je modifie l'override existant à la version v" → `UPDATE ... WHERE version = ?` ; `0` ligne affectée (version stale OU ligne supprimée entre-temps par un `restore_default` concurrent) → `DbError::OptimisticLockConflict`.
   - `restore_default(pool, company_id, template_type, language, user_id) -> Result<(), DbError>` — `DELETE` de la ligne override si présente. **Idempotent** : si aucune ligne n'existe déjà, no-op silencieux (pas d'erreur, pas d'audit).
10. **No-op (KF-004)** : dans `upsert_override`, si une ligne override existe déjà et que `subject`+`body` soumis sont **identiques** à la ligne persistée → court-circuit AVANT toute mutation (pas de bump `version`, pas d'écriture audit), retour de la ligne inchangée. Suivre exactement le pattern `is_no_op_change` documenté dans `docs/optimistic-locking-patterns.md` (comparaison champ-à-champ, helper co-localisé au-dessus de la fonction).
11. **Audit** : `upsert_override` (création ET modification) écrit `email_template.updated` dans `audit_log` (via `audit_log::insert_in_tx`, `NewAuditLogEntry::for_actor(user_id, current_user.api_key_id, ...)`) avec `details_json = { before, after }` en camelCase (convention `company_invoice_settings.rs`, la plus proche structurellement — pas la convention snake_case de `reports.rs` qui est une exception documentée à ce module). `entity_type = "email_template"`, `entity_id = ` id de la ligne. `restore_default` (quand une ligne existait effectivement) écrit `email_template.restored_default` avec `details_json = { before }`. Aucune écriture audit sur no-op (ni `upsert_override` no-op, ni `restore_default` idempotent sans ligne).
12. `docs/optimistic-locking-patterns.md` — ajouter une ligne au tableau « Repositories couverts » : `email_templates.rs` / `upsert_override` / `subject, body`.

**Validation au save**

13. Au `PUT` (voir endpoint ci-dessous), avant toute persistance : `subject` et `body` non vides (trim) → sinon `AppError::Validation` (400). Puis `kesh_core::email_template_engine::validate_tokens(&subject, &body, template_type.allowed_variables())` → si tokens inconnus, `AppError::EmailTemplateUnknownVariables { unknown_vars: Vec<String> }` — nouveau variant dans `crates/kesh-api/src/errors.rs`, mappé en **422** `EMAIL_TEMPLATE_UNKNOWN_VARIABLES` avec `details: { unknownVariables: [...] }` (pattern `BankImportUnsupportedCurrency`/`BankImportNoMatchingStatement`, `errors.rs:1191-1217`).

**Endpoints CRUD Admin-only**

14. Routes **Admin-only** (`admin_routes`, `route_layer(require_admin_role)`, `lib.rs`) :
    - `GET /api/v1/admin/email-templates` → `200` `[EmailTemplateResponse]` — les 4 combinaisons type×langue résolues (override ou défaut).
    - `GET /api/v1/admin/email-templates/{template_type}/{language}` → `200` `EmailTemplateResponse` unique. `{template_type}`/`{language}` invalides (hors enum) → `400 VALIDATION_ERROR` (pattern `parse_journal`, `company_invoice_settings.rs:89-92`).
    - `PUT /api/v1/admin/email-templates/{template_type}/{language}` body `{ subject, body, expectedVersion: Option<i32> }` → `200` `EmailTemplateResponse` mis à jour. Conflit verrou optimiste → `409 OPTIMISTIC_LOCK_CONFLICT` (réutilise `DbError::OptimisticLockConflict`, **aucun nouveau variant requis**).
    - `DELETE /api/v1/admin/email-templates/{template_type}/{language}` → `204 No Content` (restaure le défaut, idempotent).
15. `EmailTemplateResponse` (DTO `Serialize`, `camelCase`) : `templateType`, `language`, `subject`, `body`, `version: Option<i32>`, `isDefault: bool`, `allowedVariables: Vec<String>`.
16. **Invariant zéro-config testé** : `GET /api/v1/admin/email-templates` sur une company neuve (aucune ligne `email_templates`) renvoie 4 entrées toutes `isDefault: true` avec les textes par défaut — jamais de 404, jamais de tableau vide.
17. RBAC : `GET`/`PUT`/`DELETE` sur `/api/v1/admin/email-templates*` retournent `403` pour `Comptable` et `Consultation` (uniquement `Admin` autorisé — cohérent décision epic-20 §10 "Admin-only", pas de lecture Comptable+ en v1 puisqu'aucun autre consommateur API n'existe avant 20-3b, qui rendra côté serveur sans passer par cet endpoint REST).

## Tasks / Subtasks

- [ ] **T1 — Migration + schéma** (AC: #1, #2)
  - [ ] T1.1 Créer `crates/kesh-db/migrations/20260708000001_email_templates.sql` (schéma complet AC #1)
  - [ ] T1.2 Ajouter la ligne correspondante dans `docs/migrations-idempotence-audit.md`
  - [ ] T1.3 Vérifier `cargo sqlx migrate run` / build applique la migration proprement (offline mode si `SQLX_OFFLINE=true` — régénérer `.sqlx/` si nécessaire, cf. Dev Notes)

- [ ] **T2 — Entités & enum** (AC: #3, #4)
  - [ ] T2.1 `EmailTemplateType` dans `kesh-db/src/entities/email_template.rs` (`as_str`/`FromStr`/`Type<MySql>`/`Encode`/`Decode`, calqué `Language`)
  - [ ] T2.2 `EmailTemplateType::ALL` (const, mirroring `Locale::ALL`) + `allowed_variables()`
  - [ ] T2.3 Struct `EmailTemplate` (`sqlx::FromRow`) : `id, company_id, template_type, language, subject, body, version, created_at, updated_at`
  - [ ] T2.4 Struct `EffectiveEmailTemplate` (résultat résolu override|défaut)

- [ ] **T3 — Moteur `{var}` (kesh-core)** (AC: #5, #6)
  - [ ] T3.1 `crates/kesh-core/src/email_template_engine.rs` : `extract_tokens`, `validate_tokens`, `render`
  - [ ] T3.2 Tests unitaires (substitution simple, token inconnu littéral, dédup, non-double-substitution, casse)
  - [ ] T3.3 Exporter le module depuis `kesh-core/src/lib.rs`

- [ ] **T4 — Defaults** (AC: #7, #8)
  - [ ] T4.1 Textes par défaut FR/DE/IT/EN pour `invoice_send` (`email_template_defaults.rs`), tokens ∈ `{salutation, contactName, invoiceNumber, amount, dueDate, companyName}`
  - [ ] T4.2 Test : chaque défaut ne contient que des tokens déclarés (boucle sur `EmailTemplateType::ALL × Language::ALL`, appel `validate_tokens`)

- [ ] **T5 — Repository** (AC: #9, #10, #11, #12)
  - [ ] T5.1 `get_effective` + `list_effective_for_company`
  - [ ] T5.2 `upsert_override` (INSERT si `expected_version = None`, UPDATE sinon, `is_no_op_change` co-localisé, audit `email_template.updated`)
  - [ ] T5.3 `restore_default` (DELETE idempotent, audit `email_template.restored_default` seulement si une ligne existait)
  - [ ] T5.4 Mettre à jour `docs/optimistic-locking-patterns.md` (tableau « Repositories couverts »)
  - [ ] T5.5 Tests d'intégration `crates/kesh-db/tests/email_templates_repository.rs` (`#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`) : défaut sans ligne, création (version=1), update (version+1), conflit version stale (409-equivalent `DbError::OptimisticLockConflict`), no-op (version inchangée + pas d'audit — vérifier `audit_log` count), restore puis re-GET retombe sur défaut, `UNIQUE(company_id, template_type, language)` respectée, cross-tenant (2 companies, lignes indépendantes)

- [ ] **T6 — Validation & AppError** (AC: #13)
  - [ ] T6.1 Nouveau variant `AppError::EmailTemplateUnknownVariables { unknown_vars: Vec<String> }` (`errors.rs`) + mapping 422 `EMAIL_TEMPLATE_UNKNOWN_VARIABLES` avec `details.unknownVariables`
  - [ ] T6.2 Validation subject/body non vides + appel `validate_tokens` dans le handler `PUT`

- [ ] **T7 — Endpoints Admin** (AC: #14, #15, #16, #17)
  - [ ] T7.1 `crates/kesh-api/src/routes/email_templates.rs` : `list_email_templates` (GET collection), `get_email_template` (GET unique), `update_email_template` (PUT), `restore_email_template_default` (DELETE)
  - [ ] T7.2 DTOs `EmailTemplateResponse` + `UpdateEmailTemplateRequest { subject, body, expected_version: Option<i32> }` (camelCase)
  - [ ] T7.3 Enregistrement des 4 routes dans `admin_routes` (`lib.rs`, `route_layer(require_admin_role)` déjà présent sur ce sous-routeur)
  - [ ] T7.4 Tests e2e `crates/kesh-api/tests/email_templates_e2e.rs` : RBAC (Admin 200 / Comptable 403 / Consultation 403), round-trip CRUD complet, zéro-config (AC #16), validation 422 tokens inconnus, conflit 409 version stale, restore 204 puis re-GET défaut

- [ ] **T8 — Test Locally First & commit**
  - [ ] T8.1 `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (backend uniquement, story 100% Rust)
  - [ ] T8.2 Commit(s) sur `story/20-1-envoi-factures-email` (branche déjà créée, cf. Dev Notes)

## Dev Notes

### Frontières strictes de scope (ne pas dépasser)

- **NE PAS** toucher `crates/kesh-api/src/mail/` (trait `Mailer`, `SmtpMailer`) — extension pièce jointe/display-name/Reply-To = Story 20-3b.
- **NE PAS** toucher `contacts` (`language`, `salutation`) — Story 20-3b.
- **NE PAS** créer d'endpoint d'envoi de facture (`POST /invoices/{id}/send-email`) — Story 20-3b.
- **NE PAS** toucher le frontend (`frontend/`) — Story 20-2 consomme cette API.
- **NE PAS** toucher le mail de récupération de mot de passe (FTL/Fluent, instance-level) — hors scope de tout l'Epic 20 (décision Guy + critique 3 agents, cf. `epic-20-envoi-factures-email.md` §Objectif).

### Architecture crates — où vit quoi (important, prévient une erreur de placement)

`kesh-core` ne dépend PAS de `kesh-db` (sens inverse : `kesh-db` dépend de `kesh-core`). `kesh-core` n'a pas de dépendance `sqlx`. Conséquence directe sur le placement du code :

- Le moteur `{var}` (`extract_tokens`/`validate_tokens`/`render`) est **pur** (aucune connaissance de `EmailTemplateType`/`Language`, opère sur `&str`/`&[&str]`/`&HashMap<String,String>`) → **`kesh-core`**, exactement comme `kesh_core::invoice_format`.
- `EmailTemplateType`/`Language` (colonnes DB, `sqlx::Type`/`Encode`/`Decode`) → **`kesh-db/src/entities/`**, co-localisé avec `Company`/`OrgType`/`Language` existants dans `company.rs` (ces impls sqlx ne peuvent PAS vivre dans `kesh-core` sans y ajouter une dépendance `sqlx` — écart avec la convention actuelle du crate).
- Le glue code (`template_type.allowed_variables()` → `kesh_core::email_template_engine::validate_tokens(...)`) vit dans le handler `kesh-api` (qui dépend des deux crates), pas dans `kesh-db` ni `kesh-core`.

### Patterns à réutiliser tels quels (ne pas réinventer)

- **Table company-scoped avec FK CASCADE** : `crates/kesh-db/migrations/20260505000001_bank_profiles.sql` (référence citée dans le planning epic-20). Convention nommage : `fk_email_templates_company`, `uq_email_templates_company_type_language`, `idx_email_templates_company`, `ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci`.
- **Verrou optimiste + no-op (KF-004)** : `docs/optimistic-locking-patterns.md` intégralement + implémentation `crates/kesh-db/src/repositories/company_invoice_settings.rs:109-233` (`is_no_op_change` co-localisé, court-circuit **après** version-check, **avant** mutation, `tx.rollback()` + retour `Ok(before)`).
- **CHECK constraint langue** : `chk_companies_instance_language` dans `20260404000001_initial_schema.sql:18` — `CHECK (BINARY instance_language IN (BINARY 'FR', BINARY 'DE', BINARY 'IT', BINARY 'EN'))`. Le `BINARY` force la comparaison case-sensitive (collation par défaut MariaDB est case-insensitive) — **obligatoire**, pas cosmétique (cf. test `parse_journal_unknown_rejected` qui vérifie ce point pour un autre enum).
- **Audit trail transactionnel** : `audit_log::insert_in_tx(tx, NewAuditLogEntry::for_actor(...))` (`crates/kesh-db/src/repositories/audit_log.rs:29-44`, `crates/kesh-db/src/entities/audit_log.rs:132-189`). Écrit **dans la même transaction** que la mutation (jamais best-effort hors-tx pour une mutation d'entité métier — le pattern best-effort de `reports.rs` est réservé aux actions read-only de type "génération de rapport").
- **CRUD Admin-only "settings" company-scoped** : `crates/kesh-api/src/routes/company_invoice_settings.rs` (structure DTO/handler la plus proche du besoin) + enregistrement dans `admin_routes`, `crates/kesh-api/src/lib.rs:140-192` (RBAC appliqué au niveau du sous-routeur via `route_layer`, pas de check inline dans le handler).
- **Erreur 422 structurée avec `details`** : `crates/kesh-api/src/errors.rs:1191-1217` (`BankImportUnsupportedCurrency`, `BankImportNoMatchingStatement`) — modèle exact pour `EmailTemplateUnknownVariables`.
- **Formatage suisse** (pour référence future 20-3b, PAS utilisé dans cette story — aucune donnée Invoice résolue ici) : `kesh_i18n::formatting::format_money`/`format_date` (apostrophe U+2019, `dd.mm.yyyy`).

### Pourquoi PAS Fluent/FTL pour le moteur ni pour les defaults

`kesh-i18n`'s `loader.rs:14-19` : Fluent **ignore silencieusement** une variable `{ $var }` non fournie (`is_missing_variable_error` non traité comme erreur bloquante). C'est incompatible avec l'exigence produit "validation stricte au save + liste des tokens inconnus" (AC #13) et avec l'exigence sécurité "rendu infaillible mais jamais silencieusement incorrect". D'où : moteur maison `{var}` (syntaxe volontairement différente de `{ $var }` Fluent pour éviter toute confusion visuelle) + defaults en constantes Rust plutôt qu'en fichiers `.ftl`.

### Sémantique `expectedVersion` (upsert override) — point de conception à ne pas rater

Contrairement aux repositories `update()` existants (`company_invoice_settings`, etc.) où la ligne existe **toujours** (lazy-created au premier GET), `email_templates` n'a de ligne que si un override a été créé. `expectedVersion` porte donc une sémantique double :

- `None`/absent (le champ `EmailTemplateResponse.version` était `null` car `isDefault: true`) → le client "croit" qu'il n'y a pas encore d'override → le repo doit `INSERT`. Si une ligne existe déjà (race avec un autre onglet Admin), traiter comme conflit optimiste (`DbError::OptimisticLockConflict`, pas un crash SQL sur violation `UNIQUE` — attraper l'erreur SQL 1062 ou vérifier l'existence avant insert dans la même transaction).
- `Some(v)` → override existant à la version `v` → `UPDATE ... WHERE version = ?`. `0` ligne affectée (version stale OU un `DELETE` concurrent via `restore_default` a fait disparaître la ligne) → `DbError::OptimisticLockConflict` dans les deux cas (le client refera un GET pour voir l'état réel).

### RBAC — pourquoi tout Admin-only (pas de lecture Comptable+)

Contrairement à `company_invoice_settings` (`GET` ouvert à tout rôle authentifié), **aucun** endpoint `email-templates` n'a de consommateur non-Admin en v1 : le seul lecteur est la page Admin `settings/email-templates` (Story 20-2, elle-même Admin-only). Le rendu réel à l'envoi (Story 20-3b) se fera **côté serveur** (le handler `send-email` appelle directement le repository, pas cette route REST) — donc pas besoin d'élargir l'accès. Si un besoin de lecture Comptable+ émerge plus tard (ex. aperçu avant envoi), ce sera un CR explicite (cf. Issue Tracking Rule), pas une extrapolation de cette story.

### Extraction des path params `{template_type}`/`{language}` — pas d'enum Axum `Path`

Aucun endpoint existant du projet n'extrait un enum directement via `Path<(EnumA, EnumB)>` (serde-based). Pour rester cohérent avec le seul précédent d'enum "parsé depuis une valeur utilisateur" du codebase (`parse_journal`, `company_invoice_settings.rs:89-92`), extraire les deux segments en `Path<(String, String)>` puis `.parse::<EmailTemplateType>()`/`.parse::<Language>()` manuellement dans le handler, chaque erreur de parse mappée en `AppError::Validation(...)`. Ne PAS tenter l'extraction Axum `Path<(EmailTemplateType, Language)>` (chemin non testé dans ce codebase, comportement de désérialisation serde d'énums unitaires via `Path` non vérifié empiriquement ici — le contrôle explicite via `FromStr` est le choix sûr et déjà éprouvé).

**Casse** : `Language::from_str` est strictement case-sensitive (`"FR"`, pas `"fr"` — cohérent avec le `CHECK (BINARY language IN ...)` de la table). Le futur frontend (Story 20-2) devra construire les URLs avec le code langue en majuscules.

### Naming — écart volontaire vs le libellé informel du planning epic

Le document `epic-20-envoi-factures-email.md` (§ Décisions figées #4) parle de colonne `type`. **Ne pas suivre littéralement** : la convention SQL du projet est de toujours suffixer `_type` (`contact_type`, `org_type`, `account_type`, `creditor_address_type`, `reference_type`, `settlement_type` — zéro exception trouvée dans `crates/kesh-db/migrations/*.sql`). Colonne = `template_type`.

### Project Structure Notes

- Fichiers **nouveaux** : `crates/kesh-db/migrations/20260708000001_email_templates.sql`, `crates/kesh-db/src/entities/email_template.rs` (+ `email_template_defaults.rs` optionnellement séparé), `crates/kesh-db/src/repositories/email_templates.rs`, `crates/kesh-db/tests/email_templates_repository.rs`, `crates/kesh-core/src/email_template_engine.rs`, `crates/kesh-api/src/routes/email_templates.rs`, `crates/kesh-api/tests/email_templates_e2e.rs`.
- Fichiers **modifiés** : `crates/kesh-db/src/entities/mod.rs` (export), `crates/kesh-db/src/repositories/mod.rs` (export), `crates/kesh-core/src/lib.rs` (export du module `email_template_engine`), `crates/kesh-api/src/errors.rs` (nouveau variant), `crates/kesh-api/src/routes/mod.rs` (export), `crates/kesh-api/src/lib.rs` (4 routes dans `admin_routes`), `docs/migrations-idempotence-audit.md`, `docs/optimistic-locking-patterns.md`.
- Aucun conflit détecté avec la structure unifiée du projet (`kesh-core` = business logic pure, `kesh-db` = entities+repositories, `kesh-api` = routes/handlers — respecté intégralement).

### Testing standards summary

- **Unit (`kesh-core`)** : moteur `{var}` — substitution, token inconnu, dédup, anti-double-substitution, casse (§T3.2).
- **Unit (`kesh-db` entities)** : auto-cohérence des defaults vs `allowed_variables()` (§T4.2).
- **Intégration (`kesh-db/tests`)** : `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, cf. `crates/kesh-db/tests/company_invoice_settings_repository.rs` comme modèle de setup (`companies::create(...)` pour obtenir un `company_id` de test). Couverture §T5.5.
- **E2E (`kesh-api/tests`)** : RBAC 3 rôles, CRUD round-trip, zéro-config, 422, 409. Modèle de structure : tests e2e existants du crate `kesh-api` (`tests/*.rs`, setup serveur test + `reqwest`/client interne — suivre le pattern déjà en place dans le crate, pas de nouveau framework).
- **Test Locally First (CLAUDE.md)** : story 100% Rust → les 4 checks `Backend (Rust)` uniquement (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`). Pas de check frontend (aucun fichier Svelte touché). Pas de Playwright (aucune route consommée par une page avant 20-2/20-3b).

### Git intelligence

Branche `story/20-1-envoi-factures-email` déjà créée et poussée (planning seul, aucun code). Commits précédents sur cette branche = 100% doc-only (kickoff epic-20 + recadrage post-critique 3 agents). Le premier commit de code de cette story sera donc le premier commit non-doc sur la branche — pas de pattern de code antérieur à apprendre sur CETTE branche ; les patterns cités ci-dessus viennent de `main` (stories antérieures : 5.2, 7-3, 8-2).

### Migration — offline `.sqlx` cache

Si le workspace utilise `SQLX_OFFLINE=true` en CI/local (vérifier `.sqlx/` à la racine ou `crates/kesh-db/.sqlx/`), régénérer le cache après la migration + toute nouvelle requête `sqlx::query!`/`query_as!` macro-typée. **Note** : les repositories du projet utilisent majoritairement `sqlx::query`/`query_as::<_, T>` (non macro, cf. `company_invoice_settings.rs`) plutôt que les macros `query!`/`query_as!` — si ce style est suivi (recommandé, cohérent avec le reste du crate), **pas besoin** de régénérer `.sqlx/` (pas de vérification compile-time contre le schéma réel).

### References

- [Source: `_bmad-output/planning-artifacts/epic-20-envoi-factures-email.md`] — décisions figées #1-17, découpage stories, limitations L20-1/L20-2/L20-3.
- [Source: `crates/kesh-db/migrations/20260505000001_bank_profiles.sql`] — modèle table company-scoped FK CASCADE.
- [Source: `crates/kesh-db/migrations/20260404000001_initial_schema.sql#L11-18`] — modèle CHECK constraint langue.
- [Source: `docs/optimistic-locking-patterns.md`] — pattern `is_no_op_change` intégral.
- [Source: `crates/kesh-db/src/repositories/company_invoice_settings.rs`] — modèle repository verrou optimiste + audit + no-op.
- [Source: `crates/kesh-api/src/routes/company_invoice_settings.rs`] — modèle handler CRUD Admin-only.
- [Source: `crates/kesh-api/src/errors.rs#L1191-1217`] — modèle erreur 422 structurée avec `details`.
- [Source: `crates/kesh-api/src/lib.rs#L140-192`] — enregistrement `admin_routes` + `require_admin_role`.
- [Source: `crates/kesh-db/src/entities/company.rs#L70-133`] — `Language`/`OrgType` (enum DB réutilisable + pattern `sqlx::Type`/`Encode`/`Decode`).
- [Source: `crates/kesh-i18n/src/loader.rs#L14-19`] — Fluent ignore silencieusement les variables inconnues (raison du rejet FTL pour le moteur/defaults).
- [Source: `docs/migrations-idempotence-audit.md`] — format table à respecter pour la nouvelle entrée.
- [Source: CLAUDE.md §Migration breaking policy] — P1 (non-breaking = ADD TABLE), P5 (garde-fou audit idempotence).
- [Source: CLAUDE.md §Test Locally First] — 4 checks backend.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
