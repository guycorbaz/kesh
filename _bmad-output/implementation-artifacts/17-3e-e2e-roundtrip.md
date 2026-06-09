# Story 17.3e: Test d'intégration Rust round-trip export↔import (`admin_backup_e2e.rs`)

Status: ready-for-dev

<!-- Sous-story de l'épopée 17-3 (export/import installation, #112). Extraite de la spec umbrella (Partie E, AC22 / T-E1). Dépend de 17-3a (export) + 17-3c (import), tous DONE. -->
<!-- Test-only : aucune logique applicative, verrouille la correction fonctionnelle de bout en bout. -->

## Story

As a **mainteneur de Kesh**,
I want **un test d'intégration Rust qui exécute le cycle complet export → suppression → import sur un jeu de données riche et vérifie l'équivalence exacte**,
so that **toute régression future du moteur `.keshbackup` (fidélité de sérialisation, ordre FK, complétude du restore) soit détectée automatiquement en CI**.

## Contexte & cadrage

**Épopée 17-3 (#112) :** 17-3a (export **DONE**) → 17-3b (UI export **DONE**) → 17-3c (import **DONE**) → 17-3d (UI import **DONE**) → **17-3e (cette story, E2E intégration)** → 17-3f doc.

**⚠️ Périmètre = DELTA, pas duplication.** Deux fichiers de tests E2E existent déjà :
- `admin_full_export_e2e.rs` (17-3a, 7 tests) : structure ZIP, manifest, SHA, RBAC, anti-PAT, streaming.
- `admin_full_import_e2e.rs` (17-3c, 10 tests) : round-trip **minimal** (companies + 1 admin), O-1 audit, RBAC, anti-PAT, **refus 409 version / 400 SHA tamper / 400 format / 400 schema-mismatch**, **rollback transactionnel**, DC11 onboarding.

17-3e ajoute le **seul manque** : un **round-trip exhaustif sur données riches multi-tables** avec **équivalence per-table** + **login admin source** + **intégrité FK** + **audit préservé**. Les cas de **refus** (version/tamper/format) et de **rollback** sont **déjà couverts par 17-3c** (`admin_full_import_e2e.rs`) → **cross-référencés, NON dupliqués** (AC22 satisfait à l'échelle de la suite de tests).

**Playwright double-instance Docker** (use-case migration cross-instance fidèle) = **dette v0.3 documentée** (trop lourd MVP ; le test d'intégration Rust couvre l'équivalence fonctionnelle). Tracer en issue `v0.2-milestone`/`v0.3` à la livraison de l'épopée.

**Pas de changement applicatif, pas de migration.** Story test-only.

## Acceptance Criteria

1. **(AC22 — round-trip riche)** Nouveau fichier `crates/kesh-api/tests/admin_backup_e2e.rs` avec un test `#[sqlx::test]` qui :
   - **seed un jeu de données riche** via `kesh_db::test_fixtures::seed_accounting_company` (companies, users, fiscal_years, accounts, company_invoice_settings, vat_rates — ≥ 6 tables peuplées) ;
   - **capture une baseline** : `COUNT(*)` par table pour les 22 tables (`kesh_db::backup::TABLES_TO_TRUNCATE`) ;
   - **exporte** via `GET /api/v1/admin/full-export` (JWT admin seedé) → bytes `.keshbackup` ;
   - **mute l'état** entre export et import (ex. crée une company « Ghost ») pour prouver le **remplacement** ;
   - **importe** via `POST /api/v1/admin/full-import` (multipart, JWT admin) → `200` ;
   - **assert équivalence per-table** : `COUNT(*)` de chaque table == baseline (Ghost supprimée, tout restauré exactement).

2. **(AC22 — login admin source)** Après l'import, un `POST /api/v1/auth/login` avec les identifiants de l'admin **source** (`username="admin"`, `password="admin123"` — cf. `test_fixtures::ADMIN_PASSWORD_HASH`) retourne `200` → prouve la fidélité de `users.password_hash` + la restauration des users + l'intégrité de la chaîne FK `users→companies`.

3. **(AC22 — intégrité FK + audit)** Post-import :
   - **FK** : une requête joignant des tables restaurées réussit et renvoie les données attendues (ex. `company_invoice_settings.default_receivable_account_id` pointe sur un `accounts.id` réel ; le `fiscal_years` appartient à la company) ;
   - **audit** : l'`audit_log` contient une entrée `admin.full_import` (`entity_type='installation'`) post-restore, et l'historique source est préservé (les rows `audit_log` exportées sont présentes).

4. **(Cross-référence, pas duplication)** Le fichier documente (commentaire d'en-tête) que les cas **refus version `409` / SHA tamper `400` / format `400` / schema-mismatch `400`** et **rollback transactionnel** sont couverts par `admin_full_import_e2e.rs` (17-3c). 17-3e ne les ré-implémente PAS.

5. **(Qualité)** `cargo test --workspace -j1 -- --test-threads=1` vert (le nouveau test inclus), 0 régression. Pas de `unwrap()` masquant un échec silencieux (assertions explicites avec messages).

## Tasks / Subtasks

- [ ] **T-E1** Créer `crates/kesh-api/tests/admin_backup_e2e.rs` : harness HTTP (réutiliser le pattern `spawn_app` + `forge_jwt` de `admin_full_import_e2e.rs` — `TestApp`, `test_config`, `spawn_app`, `forge_jwt`). (AC: 1)
- [ ] **T-E2** Test `full_roundtrip_rich_dataset_preserves_all_tables` : seed riche (`seed_accounting_company`, éventuellement + `seed_contact_and_product` pour contacts/products) → baseline counts (22 tables) → GET export (JWT admin) → créer company « Ghost » → POST import → assert équivalence per-table + Ghost absente. (AC: 1)
- [ ] **T-E3** Assertions complémentaires dans le même test (ou tests dédiés) : login admin source `200` (AC2) ; FK join (`company_invoice_settings`→`accounts`, `fiscal_years`→`companies`) (AC3) ; audit `admin.full_import` présent + rows audit source préservées (AC3). (AC: 2, 3)
- [ ] **T-E4** Commentaire d'en-tête documentant le périmètre (delta vs 17-3a/17-3c) + cross-référence refus/rollback + dette Playwright double-instance v0.3. (AC: 4)

## Dev Notes

### Réutilisation (ground-truth)

| Brique | Chemin | Usage |
|---|---|---|
| Seed riche | `kesh_db::test_fixtures::seed_accounting_company(&pool) -> SeededCompany` (`crates/kesh-db/src/test_fixtures.rs:80`, **`pub mod` compilé en permanence**, déjà utilisé par `exports_global_e2e.rs` etc.) | Peuple companies/users/fiscal_years/accounts/company_invoice_settings/vat_rates. `SeededCompany { company_id, fiscal_year_id, admin_user_id, changeme_user_id, accounts: HashMap }` |
| Admin seedé | `username="admin"`, password `"admin123"` (`ADMIN_PASSWORD_HASH`, `test_fixtures.rs:31-33`) | Login source (AC2) + forge JWT (role `Admin`) |
| Contacts/products | `kesh_db::test_fixtures::seed_contact_and_product(&pool, ...)` (`:441`, **après** seed_accounting_company) | Enrichir le dataset (optionnel, AC1) |
| Inventaire tables | `kesh_db::backup::TABLES_TO_TRUNCATE` (`pub`, 22 entrées) | Itérer les `COUNT(*)` baseline + équivalence |
| Cœur export | `kesh_api::admin_backup::export::build_keshbackup(&pool)` (alternative à GET HTTP si besoin) | — (préférer le GET HTTP end-to-end) |
| Harness HTTP | `crates/kesh-api/tests/admin_full_import_e2e.rs` (`spawn_app`, `forge_jwt`, `backup_form`, `post_import`, `test_config`, `TEST_JWT_SECRET`) | **Copier/adapter** le harness (les helpers ne sont pas partagés entre fichiers de tests — duplication assumée, pattern projet) |
| Endpoints | `GET /api/v1/admin/full-export` (17-3a) + `POST /api/v1/admin/full-import` (17-3c) + `POST /api/v1/auth/login` | Round-trip + login |
| Multipart upload | `reqwest::multipart::Form` champ `file` (cf. `admin_full_import_e2e.rs::backup_form`) | POST import |

### Détails

- **JWT admin** : `forge_jwt(admin_user_id, "Admin", company_id)` (le seed retourne `admin_user_id` + `company_id`). Le rôle DOIT être `Admin` (RBAC + l'export/import l'exigent).
- **Équivalence per-table** : capturer `BTreeMap<&str, i64>` des counts avant export ; après import, recomparer. `onboarding_state` : le seed n'en crée pas (count 0) ; le restore l'exclut (DC11) → reste 0 → équivalence OK. `_kesh_version`/`_sqlx_migrations` : hors `TABLES_TO_TRUNCATE`, non comptées.
- **Ghost** : `companies::create` (ou INSERT) d'une company supplémentaire APRÈS l'export ; l'import doit la supprimer (la baseline companies count ne l'inclut pas).
- **Login** : `POST /api/v1/auth/login` `{username:"admin", password:"admin123"}` → `200`. Le rate-limiter du `test_config` (100 tentatives) tolère.
- **Audit** : `seed_accounting_company` ne crée pas forcément de rows `audit_log` ; l'assertion « source préservée » vérifie que le COUNT audit_log post-import == baseline + 1 (l'entrée `admin.full_import`). Si baseline audit = 0, assert `>= 1` et l'entrée `admin.full_import` présente.
- ⚠️ Le harness HTTP des tests ne partage pas ses helpers entre fichiers (`crates/kesh-api/tests/*.rs` sont des crates de test séparés) → **copier** `spawn_app`/`forge_jwt`/etc. depuis `admin_full_import_e2e.rs` (duplication assumée, conforme au pattern projet — cf. les multiples harnesses dupliqués entre fichiers e2e).

### Standards projet (CLAUDE.md)

- **Test Locally First** : modif touche les tests d'intégration DB → `cargo test --workspace -j1 -- --test-threads=1` (mode serial, MariaDB requis). `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -D warnings`.
- **Migration breaking policy** / **Pattern batch** : N/A (test-only).
- **Commit par étape BMAD**, pas de push auto. Branche active : `story/17-3-export-import-installation`.

### Project Structure Notes

- **Nouveau** : `crates/kesh-api/tests/admin_backup_e2e.rs` (test d'intégration). Aucun fichier applicatif touché.
- Nom de fichier conforme à l'umbrella (AC22 mentionne `admin_backup_e2e.rs`).

### References

- [Source: _bmad-output/implementation-artifacts/17-3-export-import-installation.md] — Partie E AC22, §Test E2E double-instance (dette v0.3)
- [Source: _bmad-output/implementation-artifacts/17-3c-backend-import.md] — refus/rollback déjà couverts (admin_full_import_e2e.rs)
- [Source: crates/kesh-db/src/test_fixtures.rs:80,441] — seed_accounting_company, seed_contact_and_product
- [Source: crates/kesh-api/tests/admin_full_import_e2e.rs] — harness HTTP à copier
- [Source: crates/kesh-db/src/backup.rs] — TABLES_TO_TRUNCATE
- [Source: CLAUDE.md] — Test Locally First (serial), commit/branch

## Dev Agent Record

### Agent Model Used

_(à compléter par dev-story)_

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Étape | Modèle | Résumé |
|------|-------|--------|--------|
| 2026-06-09 | create-story (sous-story) | Opus 4.8 | Story 17-3e (E2E intégration round-trip) extraite umbrella Partie E (AC22). **Delta** : round-trip riche multi-tables (seed_accounting_company) + équivalence per-table + login admin source (admin/admin123) + FK + audit ; refus/rollback déjà couverts 17-3c (cross-réf, pas de duplication). Playwright double-instance = dette v0.3. Test-only, aucun fichier applicatif. T-E1..T-E4. Prochaine : `bmad-dev-story 17-3e`. |
