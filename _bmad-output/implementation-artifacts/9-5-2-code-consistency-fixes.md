# Story 9.5-2: Code consistency fixes (config tests env + audit JSON keys snake_case)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a mainteneur projet Kesh,
I want fixer deux incohérences code Epic 9.5 catégorie A : (1) régression locale `kesh-api::config::tests::*` (N tests fail à diagnostiquer sur 34 totaux — comptage exact à confirmer par dev en T2, l'epic-9-5.md mentionnait « 20/24 » mais le nombre réel de tests est 34 post-Stories ultérieures) liée à la collision avec `.env` projet via `dotenvy::dotenv()` rechargement inconditionnel dans `from_env()`, (2) `audit_log.details_json` JSON keys incohérence entre 3 fonctions `emit_*_audit` (2 en camelCase Story 9-1/9-2a + 1 en snake_case Story 9-2b),
so that `cargo test -p kesh-api --lib config::tests` passe **34/34** localement même avec `.env` projet présent, et que les JSON keys `audit_log.details_json` soient uniformément **snake_case** dans les 3 fonctions audit (cohérent SQL JSON path `details_json->>'$.field_name'` future-proof, AC #23 spec 9-2b explicite + retro Epic 9 challenge C3).

## Scope

Story **code-only backend Rust** stricte. Aucun fichier frontend (`.ts` / `.svelte`) touché. Aucune migration DB. Aucun changement API HTTP visible côté client (les query params URL `?fiscalYearId=` restent camelCase — c'est l'API HTTP convention REST, pas les keys JSON internes).

**Fichiers production touchés** :

- `crates/kesh-api/src/config.rs` — module `#[cfg(test)] mod tests` lignes 687-1278 + éventuellement `Cargo.toml` si nouvelle dev-dependency ajoutée (`serial_test` ou `temp-env`).
- `crates/kesh-api/src/routes/reports.rs` — 2 fonctions à migrer : `emit_report_audit` (Story 9-1, ligne 678) + `emit_report_export_audit` (Story 9-2a, ligne 728).

**Fichiers tests touchés** :

- `crates/kesh-api/src/config.rs` — module tests `#[cfg(test)] mod tests` (si refactor isolation env).
- `crates/kesh-api/tests/reports_e2e.rs` — assertions audit_log lignes 978-991 (5 keys camelCase à migrer en snake_case).

**Hors scope** :

- **Query params HTTP URL** (`?fiscalYearId=...&periodStart=...&periodEnd=...`) — restent camelCase (convention REST + breaking change client). Le scope est uniquement les **clés JSON dans `audit_log.details_json`** (côté serveur, accédé par SQL JSON path).
- **`emit_global_export_audit`** (Story 9-2b, `crates/kesh-api/src/routes/exports.rs:140`) — **déjà conforme snake_case** (cf. commentaire ligne 162 « clés snake_case pour permettre les SQL JSON paths »). Aucune action.
- **Test E2E `report.exported` audit_log** — limite de couverture pré-existante : `grep "report.exported" crates/kesh-api/tests/` → 0 occurrence. La fonction `emit_report_export_audit` n'a aucun test E2E assertant ses keys (contrairement à `emit_report_audit` testé `reports_e2e.rs:956`). Cette absence de test est documentée comme L1 (limite v0.2) — pas une régression introduite par cette story.
- **Migration test couverture `report.exported`** — hors scope. Si l'ajout est facile (~10 lignes pattern identique `reports_e2e.rs:956`), peut être inclus mais reclassé en bonus, pas un AC obligatoire.
- **DB migration** : `audit_log.details_json` est de type `JSON` (MariaDB blob binaire) — la migration camelCase → snake_case **ne modifie aucune ligne historique** (les lignes audit_log déjà insérées avec keys camelCase restent en DB, mais les nouvelles insertions seront en snake_case). Acceptable v0.1 (pas de prod, cf. memory `project_prod_deployment_gating`).

## Acceptance Criteria

### Item 1 — Fix `config::tests` env isolation

1. **Given** un workspace Kesh avec `.env` projet contenant `KESH_HOST=0.0.0.0` + `KESH_TEST_MODE=true` (état réel observé en dev), **When** `cargo test -p kesh-api --lib config::tests` est exécuté (depuis racine repo, parallel ou serial), **Then** **34/34 tests passent** (zéro fail dû à pollution `.env`). Solution = isolation env explicite côté tests, pas dépendance ordre d'exécution.

2. **Given** la solution choisie pour AC #1 (recommandée Pass 1 spec validate = option (d) `#[cfg(not(test))]` autour de `dotenvy::dotenv()` dans `from_env()` ; alternative option (d-bis) wrapper `from_env_for_test()` si préférence pattern explicite), **When** documentée dans Dev Agent Record, **Then** justification écrite incluant : (a) cause root **réelle** confirmée par diagnostic T2 (dotenvy reload inconditionnel ligne 309, pas `KESH_TEST_MODE` manquant dans `reset_env()` — Pass 1 validate a corrigé cette hypothèse), (b) approche choisie + raison, (c) trade-offs vs alternatives (a)/(b)/(c) rejetées.

3. **And** la solution **ne casse aucun test existant** : vérifier par grep que les 34 tests `config::tests` set leurs vars explicitement post-`reset_env()` (aucun ne valide le comportement « lit `.env` correctement »). Confirmation Pass 1 spec validate : aucun test trouvé qui dépend de `dotenvy::dotenv()` en mode test.

4. **And** **0 régression** sur `cargo test -p kesh-api --lib` global (la modif ne casse pas d'autres tests unit du crate).

### Item 2 — JSON keys snake_case standardisation

5. **Given** `emit_report_audit` (`crates/kesh-api/src/routes/reports.rs:678`), **When** une ligne `audit_log` est insérée avec `action = 'report.generated'`, **Then** `details_json` contient les clés **snake_case** : `report_type`, `fiscal_year_id`, `period_start`, `period_end`, `journal_filter` (vs camelCase actuel `reportType, fiscalYearId, periodStart, periodEnd, journalFilter`).

6. **Given** `emit_report_export_audit` (`crates/kesh-api/src/routes/reports.rs:728`), **When** une ligne `audit_log` est insérée avec `action = 'report.exported'`, **Then** `details_json` contient les clés **snake_case** : `report_type`, `format`, `fiscal_year_id`, `period_start`, `period_end`, `journal_filter` (`format` est déjà mot simple unchanged).

7. **Given** le test E2E `crates/kesh-api/tests/reports_e2e.rs` ligne ~978-991 qui assertit les keys camelCase de `report.generated`, **When** la migration snake_case est appliquée, **Then** les 5 assertions sont mises à jour :
   - `assert_eq!(details["reportType"], ...)` → `assert_eq!(details["report_type"], ...)`
   - `assert_eq!(details["fiscalYearId"], ...)` → `assert_eq!(details["fiscal_year_id"], ...)`
   - `details.get("periodStart").is_some()` → `details.get("period_start").is_some()`
   - `details.get("periodEnd").is_some()` → `details.get("period_end").is_some()`
   - `details.get("journalFilter").is_some()` → `details.get("journal_filter").is_some()`
   - Commentaire ligne 956-958 (« AC #25 exige `details_json = { reportType, ... }` ») mis à jour pour refléter snake_case.

8. **Given** la migration, **When** `cargo test -p kesh-api --test reports_e2e` est exécuté, **Then** **28/28 tests passent** (baseline pré-story préservée + assertions migrées).

9. **And** **aucune autre assertion camelCase audit_log** ne traîne dans `tests/` : `grep -rn "details\[\"reportType\"\]\|details\[\"fiscalYearId\"\]\|details\[\"periodStart\"\]\|details\[\"periodEnd\"\]\|details\[\"journalFilter\"\]" crates/kesh-api/tests/` retourne 0 match post-migration.

### Cohérence globale + non-régression

10. **Given** la story implémentée, **When** `cargo test --workspace -- --test-threads=1` est exécuté (pattern CI), **Then** **toutes** les baselines préservées : 28/28 reports_e2e + 20/20 reports_export_e2e + 21/21 exports_global_e2e + autres tests workspace. **0 régression**.

11. **And** `cargo fmt --all -- --check` clean + `cargo clippy --workspace --all-targets -- -D warnings` clean (cohérent `Test Locally First` CLAUDE.md §Backend Rust).

12. **And** **standardisation documentée** : un commentaire de référence est ajouté soit dans `crates/kesh-api/src/routes/reports.rs` (en tête de `emit_report_audit`) soit dans `architecture.md` (section audit) qui formalise la convention : « `audit_log.details_json` JSON keys = **snake_case** pour cohérence SQL JSON path future-proof. Les autres surfaces API (HTTP query params, frontend metadata.json) **restent** camelCase per convention REST. ». Référence cross-projet pour les futures fonctions `emit_*_audit` (Epic 10 TVA `vat.calculated`, Epic 11 pain.001 `payment.created`, etc.).

13. **And** `emit_global_export_audit` (`crates/kesh-api/src/routes/exports.rs:140`) reste **inchangé** (déjà snake_case conforme Story 9-2b Pass 1 H2). Confirmer par `git diff --stat HEAD` post-implémentation.

14. **And** **0 changement breaking API HTTP** : les query params URL `?fiscalYearId=`, `?periodStart=`, `?periodEnd=`, `?journalFilter=`, `?reportType=` restent camelCase inchangés (cf. `crates/kesh-api/src/routes/reports.rs` deserialize `#[serde(rename_all = "camelCase")]` sur `ExportQuery` ou équivalent). Vérifier par grep que le pattern `#[serde(rename_all = "camelCase")]` sur les structs query params est conservé.

15. **Given** les commits, **When** lus, **Then** chaque commit a un scope clair (e.g. `fix(kesh-api): config::tests env isolation (34/34 local)` puis `refactor(kesh-api): audit_log JSON keys snake_case (emit_report_*_audit)`). Pas de commit mélangé Item 1 + Item 2 (les 2 items sont indépendants — facilite revert).

16. **Given** la story complétée, **When** `git diff --stat main` est lu, **Then** scope confirmé : **2-4 fichiers modifiés uniquement** dans `crates/kesh-api/` + éventuellement `Cargo.toml` (si nouvelle dev-dependency). Aucun fichier `_bmad-output/` hors story file lui-même. Aucun fichier `frontend/`.

## Tasks / Subtasks

- [ ] **T1** Pre-flight (AC: #11)
  - [ ] T1.1 Vérifier branche `chore/epic-9-5-planning` à jour : `git status` clean modulo les `.claude/skills/` delta préexistant orthogonal.
  - [ ] T1.2 `cargo build --workspace` clean depuis racine.
  - [ ] T1.3 Lire `crates/kesh-api/src/config.rs` lignes 687-1278 (module `#[cfg(test)] mod tests` complet) pour comprendre `env_lock()` + `reset_env()` existants.
  - [ ] T1.4 Lire `crates/kesh-api/src/routes/reports.rs` lignes 670-770 (`emit_report_audit` + `emit_report_export_audit` bodies).
  - [ ] T1.5 Lire `crates/kesh-api/src/routes/exports.rs:140-200` (`emit_global_export_audit` body) pour confirmer pattern snake_case canonique.
  - [ ] T1.6 Lire `crates/kesh-api/tests/reports_e2e.rs` lignes 950-995 (assertions audit_log camelCase actuelles).

- [ ] **T2** Diagnostic Item 1 — cause root `config::tests` fail (AC: #1, #2)
  - [ ] T2.1 Lancer `cargo test -p kesh-api --lib config::tests -- --test-threads=1 --nocapture 2>&1 | tee /tmp/config-tests-pre.log` (avec `.env` projet présent). Compter nb fail + identifier les tests qui échouent.
  - [ ] T2.2 Pour 2-3 tests échouants représentatifs, analyser cause root : (a) quelle env var manque dans `reset_env()` ? (b) ordre setup tests ? (c) `dotenvy::dotenv()` reloaded inopinément ?
  - [ ] T2.3 Documenter cause root précise dans Dev Agent Record (Debug Log References).

- [ ] **T3** Implémenter fix Item 1 (AC: #1, #2, #3, #4)
  - [ ] T3.1 Appliquer **option (d) recommandée par Pass 1 spec validate** : annoter `dotenvy::dotenv().ok()` ligne 309 de `crates/kesh-api/src/config.rs` avec `#[cfg(not(test))]` pour skipper le chargement `.env` en mode test (cause root réelle = dotenvy rechargement après `reset_env()`, cf. Dev Notes §"Cause root Item 1"). Alternative si Guy préfère pattern explicite : option (d-bis) wrapper `from_env_for_test()` — à arbitrer en dev si la version cfg-attribute pose problème. Options (a)/(b)/(c) **rejetées** comme insuffisantes seules.
  - [ ] T3.2 Pas de nouvelle dev-dependency requise pour option (d). Si option (d-bis) wrapper choisi, modifier ~34 call-sites tests pour appeler `from_env_for_test()` à la place de `from_env()`.
  - [ ] T3.3 Appliquer l'approche choisie. `cargo build --workspace -p kesh-api --tests` clean.
  - [ ] T3.4 Re-lancer `cargo test -p kesh-api --lib config::tests -- --test-threads=1` → vérifier **34/34** pass.
  - [ ] T3.5 Re-lancer avec parallel (`cargo test -p kesh-api --lib config::tests`) → vérifier **34/34** pass aussi (idéal mais pas obligatoire si `--test-threads=1` est imposé par convention CI).
  - [ ] T3.6 Documenter dans Dev Agent Record : approche choisie + justification trade-offs.

- [ ] **T4** Implémenter Item 2 — migrer `emit_report_audit` snake_case (AC: #5)
  - [ ] T4.1 Dans `crates/kesh-api/src/routes/reports.rs:678-720` (`emit_report_audit`), modifier le bloc `serde_json::json!({...})` : renommer 5 keys camelCase → snake_case (`reportType → report_type`, `fiscalYearId → fiscal_year_id`, `periodStart → period_start`, `periodEnd → period_end`, `journalFilter → journal_filter`).
  - [ ] T4.2 Vérifier que les usages de ces clés en lecture (e.g. SQL JSON paths dans tests ou requêtes) sont compatibles ou migrés. `grep -rn "details_json.*reportType\|details_json.*->'\\$.reportType'" crates/` pour identifier.
  - [ ] T4.3 `cargo build --workspace -p kesh-api` clean.

- [ ] **T5** Implémenter Item 2 — migrer `emit_report_export_audit` snake_case (AC: #6)
  - [ ] T5.1 Dans `crates/kesh-api/src/routes/reports.rs:728-770` (`emit_report_export_audit`), modifier le bloc `serde_json::json!({...})` : renommer 5 keys (idem T4.1, `format` reste `format` mot simple inchangé).
  - [ ] T5.2 `cargo build --workspace -p kesh-api` clean.

- [ ] **T6** Migrer assertions test E2E `reports_e2e.rs` (AC: #7)
  - [ ] T6.1 Dans `crates/kesh-api/tests/reports_e2e.rs:978-991`, remplacer les 5 assertions camelCase par snake_case :
    - `details["reportType"]` → `details["report_type"]`
    - `details["fiscalYearId"]` → `details["fiscal_year_id"]`
    - `details.get("periodStart")` → `details.get("period_start")`
    - `details.get("periodEnd")` → `details.get("period_end")`
    - `details.get("journalFilter")` → `details.get("journal_filter")`
  - [ ] T6.2 Mettre à jour le commentaire ligne 956-958 (« AC #25 exige `details_json = { reportType, ... }` ») pour refléter snake_case avec mention « cohérent §audit_log keys convention §12 ».
  - [ ] T6.3 `cargo build --workspace -p kesh-api --tests` clean.

- [ ] **T7** Documenter convention `audit_log.details_json` snake_case (AC: #12)
  - [ ] T7.1 Choix de localisation documentation : (a) commentaire docstring au-dessus de `emit_report_audit` dans `reports.rs` (proche du code, discoverable), OU (b) section nouvelle dans `_bmad-output/planning-artifacts/architecture.md` §"Audit log conventions", OU (c) ligne dans CLAUDE.md §"Code Quality Rules". **Recommandation** : (a) — proximité du code + automatique discoverability par grep `emit_*_audit`.
  - [ ] T7.2 Rédiger le commentaire : « **Convention projet** : `audit_log.details_json` JSON keys = **snake_case** (cohérent SQL JSON path `details_json->>'$.field_name'` future-proof, AC #23 Story 9-2b explicite). Les query params HTTP URL + frontend metadata.json restent **camelCase** per convention REST/JS. Référence : `emit_global_export_audit` (Story 9-2b) = canonique snake_case. Migration camelCase → snake_case 2026-05-18 Story 9-5-2 pour `emit_report_audit` + `emit_report_export_audit`. »
  - [ ] T7.3 Ajouter le commentaire au-dessus de la fonction `emit_report_audit` (avant la docstring `/// Audit log...`).

- [ ] **T8** Tests + validation finale (AC: #8, #9, #10, #11, #13, #14)
  - [ ] T8.1 `cargo fmt --all -- --check` clean.
  - [ ] T8.2 `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - [ ] T8.3 `cargo build --workspace --all-targets` clean.
  - [ ] T8.4 `cargo test -p kesh-api --lib config::tests` → 34/34 pass (re-vérif T3.4).
  - [ ] T8.5 `cargo test -p kesh-api --test reports_e2e -- --test-threads=1` → 28/28 pass (baseline préservée + assertions migrées).
  - [ ] T8.6 `cargo test -p kesh-api --test reports_export_e2e -- --test-threads=1` → 20/20 pass (baseline préservée, aucune modif test).
  - [ ] T8.7 `cargo test -p kesh-api --test exports_global_e2e -- --test-threads=1` → 21/21 pass (baseline préservée, `emit_global_export_audit` inchangé).
  - [ ] T8.8 `cargo test --workspace -- --test-threads=1` → tout pass (regression suite complète CI-style).
  - [ ] T8.9 Vérifier `git diff --stat exports.rs` → **0 modification** (sanity check AC #13).
  - [ ] T8.10 `grep -rn "details\[\"reportType\"\]\|details\[\"fiscalYearId\"\]\|details\[\"periodStart\"\]\|details\[\"periodEnd\"\]\|details\[\"journalFilter\"\]" crates/kesh-api/tests/` → 0 match (AC #9).
  - [ ] T8.11 Vérifier query params API HTTP inchangés : `grep -n "rename_all.*camelCase\|fiscalYearId\|periodStart" crates/kesh-api/src/routes/reports.rs` → patterns `#[serde(rename_all = "camelCase")]` sur structs query params préservés (AC #14).

- [ ] **T9** Commits séparés (AC: #15)
  - [ ] T9.1 Commit 1 : `fix(kesh-api): config::tests env isolation — 34/34 pass local avec .env présent` (T2-T3, message body avec cause root + approche choisie).
  - [ ] T9.2 Commit 2 : `refactor(kesh-api): audit_log JSON keys snake_case (emit_report_audit + emit_report_export_audit) — Epic 9.5 Story 9-5-2 item 2` (T4-T7, message body avec scope keys migrées + assertions test mises à jour).
  - [ ] T9.3 **NE PAS** mélanger Item 1 et Item 2 dans un seul commit (revert sélectif facilité).

## Dev Notes

### Cause root Item 1 — diagnostic confirmé par grep ground-truth Pass 1 spec validate

Le fichier `config.rs:687-1278` a déjà :
- Un `env_lock()` `Mutex` global pour sérialiser les tests qui touchent env vars (ligne 700).
- Une fonction `reset_env()` (lignes 711-726) qui `env::remove_var` pour **toutes** les vars `KESH_*` connues, **incluant `KESH_TEST_MODE` ligne 725** (vérifié ground-truth Pass 1).

**Cause root réelle** : `dotenvy::dotenv().ok()` est appelé **inconditionnellement** ligne 309 au début de `Config::from_env()`. Après `reset_env()`, les vars sont bien purgées du process, **mais** le prochain appel à `from_env()` recharge `.env` projet (contenant `KESH_HOST=0.0.0.0` + `KESH_TEST_MODE=true`) avant de lire `env::var(...)`. Les tests qui n'override pas `KESH_HOST` explicitement obtiennent donc la combinaison `KESH_HOST=0.0.0.0` + `KESH_TEST_MODE=true` → `ConfigError::TestModeWithPublicBind` (erreur définie ligne 61-63).

**Confirmation in-code** : le test `config_from_env_with_database_url` ligne 742-745 contient déjà ce commentaire explicite : « Set explicite pour neutraliser un éventuel `.env` local qui porterait `KESH_HOST=0.0.0.0` (dotenvy charge `.env` avant de lire les vars). ». Les tests qui pré-datent ce commentaire ne font pas ce set explicite et sont les candidats au fail.

**Solution recommandée (T3.1 option d — corrigée Pass 1)** : annoter `dotenvy::dotenv().ok()` ligne 309 avec `#[cfg(not(test))]` (ou wrapper conditionnel équivalent) pour skipper le chargement `.env` en mode test. Diff = 1-2 lignes. Aucune nouvelle dependency.

**Justification** : aucun des 34 tests existants ne valide le comportement « lit `.env` correctement » — ils tous set leurs vars explicitement via `set_var` post-`reset_env`. Le risque théorique de l'option (d) est donc nul empiriquement.

**Alternatives (insuffisantes seules ou plus invasives)** :

- **Option (a) — étendre `reset_env()`** : INSUFFISANTE seule. Même si on ajoute `KESH_BACKEND_URL` ou `KESH_LANG` au reset, `dotenvy::dotenv()` rechargera `.env` au prochain `from_env()`, annulant la purge. **NE PAS choisir cette option seule.**
- **Option (b) — `serial_test = "3"` + `#[serial(env)]`** : ne résout pas la cause root (dotenvy reload). Utile pour serialisation parallèle mais le `Mutex` `env_lock()` existant gère déjà ça. Trade-off : ~30 annotations à ajouter sans bénéfice nouveau. **Non recommandé.**
- **Option (c) — `temp-env` crate** : `temp_env::with_var("KESH_HOST", "127.0.0.1", || { Config::from_env() })` workaround par test individuel. Scope par test, propre, mais ne fix pas la cause root globale. Trade-off : ~10-15 tests à refactor (verbeux). **Acceptable comme défense en profondeur après option (d).**
- **Option (d-bis) — wrapper `from_env_for_test()`** : créer une fn `pub(crate) fn from_env_for_test() -> Result<Self, ConfigError>` qui skippe `dotenvy::dotenv()` et que les tests appellent à la place. Plus explicite que `#[cfg(not(test))]` mais demande ~34 call-sites à migrer. **Trade-off acceptable si Guy préfère explicite vs cfg-magique.**

**Décision spec validate Pass 1 — Recommandation principale** : option (d) `#[cfg(not(test))]`. Le dev peut choisir option (d-bis) si préférence pour pattern explicite vs cfg-attribute. Options (a)/(b)/(c) **rejetées** comme insuffisantes seules.

### Migration JSON keys — pattern référence Story 9-2b

`emit_global_export_audit` (Story 9-2b Pass 1 H2) est la référence canonique snake_case. Code à imiter (`crates/kesh-api/src/routes/exports.rs:140-180`) :

```rust
async fn emit_global_export_audit(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    byte_size: usize,
    csv_count: usize,
    fiscal_year_scope: &str,
    duration_ms: u64,
) {
    // ...
    details_json: Some(serde_json::json!({
        "company_id": company_id,
        "byte_size": byte_size,
        "csv_count": csv_count,
        "fiscal_year_scope": fiscal_year_scope,
        "duration_ms": duration_ms,
    })),
    // ...
}
```

Le commentaire ligne 162 « Pass 1 code-review H2 (C1 AA-MEDIUM-03) — clés snake_case pour permettre les SQL JSON paths `details_json->>'$.company_id'` (cohérent AC #23 + AC #29(m) ground-truth spec). » est la justification canonique. T7.2 doit s'en inspirer.

### Pourquoi snake_case côté audit_log et camelCase côté API HTTP

| Surface | Convention | Justification |
|---|---|---|
| Query params URL HTTP (`?fiscalYearId=...`) | camelCase | Convention REST + JS frontend. Breaking change client si modifié. |
| Request/Response body HTTP (`{"reportType": "..."}`) | camelCase via `#[serde(rename_all = "camelCase")]` sur structs | Idem |
| `metadata.json` frontend (`{"keshVersion", "exportDate", ...}`) | camelCase | Lu par `package.json`-style tooling JS |
| **`audit_log.details_json` côté serveur** | **snake_case** | **SQL JSON path `details_json->>'$.field_name'` future-proof. Pas exposé côté client. Convention SQL/Rust snake_case naturelle.** |

Le mix est intentionnel — chaque surface respecte sa convention native. La story 9-5-2 corrige une déviation : `emit_report_audit` + `emit_report_export_audit` étaient incohérents avec cette convention.

### Risque R1 — assertions test E2E à mettre à jour

Les 5 assertions camelCase de `reports_e2e.rs:978-991` (test `report_audit_emitted_on_balance_sheet_200` ou similaire) **doivent** être migrées simultanément, sinon le test casse au build. T6 couvre ça mais le dev agent doit vérifier qu'il n'y a pas d'autres assertions camelCase pour audit_log dans d'autres tests (AC #9 grep checkpoint).

### Risque R2 — `dotenvy::dotenv()` reload

Si l'approche T3.1 option (d) est choisie (skip dotenv en test), vérifier que les tests qui chargent intentionnellement `.env` ne sont pas dans `config::tests`. Si oui → autre approche.

### Memory carries

- `feedback_zero_tech_debt_carryforward` : cette story résout 2 dettes catégorie A héritées Epic 9 retro (C2 + C3). Cohérent politique projet.
- `feedback_haiku_review_diff_combined` : la discipline grep ground-truth s'appliquera lors du code-review — chaque assertion sur lignes précises doit être vérifiable.

### Project Structure Notes

- **Fichiers modifiés (production)** : `crates/kesh-api/src/config.rs` (test module) + `crates/kesh-api/src/routes/reports.rs` (2 fns audit).
- **Fichiers modifiés (tests)** : `crates/kesh-api/tests/reports_e2e.rs` (assertions migrées) + potentiellement `crates/kesh-api/src/config.rs` (test module) selon approche T3.
- **Fichier Cargo.toml** : modifié uniquement si nouvelle dev-dependency ajoutée (T3.1 option b/c).
- **Aucun fichier frontend** touché.
- **Aucun fichier `_bmad-output/`** touché (hors story file lui-même + sprint-status).

### Testing standards summary

- **Tests existants à valider** :
  - `cargo test -p kesh-api --lib config::tests` — cible 34/34 pass.
  - `cargo test -p kesh-api --test reports_e2e` — cible 28/28 pass.
  - `cargo test -p kesh-api --test reports_export_e2e` — cible 20/20 pass (aucune modif test).
  - `cargo test -p kesh-api --test exports_global_e2e` — cible 21/21 pass (aucune modif).
  - `cargo test --workspace` — tout pass.
- **Pas de nouveau test ajouté** (story de fix + standardisation, pas de nouvelle feature).
- **Test Locally First** : applicable (modifs `.rs` réelles). Exécuter `cargo fmt --check` + `cargo clippy --workspace -- -D warnings` + `cargo test --workspace -- --test-threads=1` localement avant push.

### References

- [Source: _bmad-output/planning-artifacts/epic-9-5.md#Story-9.5-2] — spec parent epic
- [Source: _bmad-output/implementation-artifacts/epic-9-retro-2026-05-17.md#C2-C3] — challenges héritage Epic 9
- [Source: crates/kesh-api/src/config.rs] — module config + tests à fixer
- [Source: crates/kesh-api/src/routes/reports.rs:678] — emit_report_audit camelCase à migrer
- [Source: crates/kesh-api/src/routes/reports.rs:728] — emit_report_export_audit camelCase à migrer
- [Source: crates/kesh-api/src/routes/exports.rs:140] — emit_global_export_audit snake_case canonique (référence)
- [Source: crates/kesh-api/tests/reports_e2e.rs:956-991] — assertions audit_log camelCase à migrer
- [Source: CLAUDE.md§Test Locally First] — checks pré-push obligatoires
- [Source: CLAUDE.md§Tech debt management] — politique zero carry-forward (cette story est catégorie A héritage)

## Dev Agent Record

### Agent Model Used

(À renseigner par le dev — typiquement Claude Opus 4.7 ou Sonnet 4.6.)

### Debug Log References

(Vide à la création — sera renseigné post-T2 diagnostic avec cause root précise `config::tests` + post-T6 avec extraits assertions migrées.)

### Completion Notes List

(Vide à la création — sera renseigné post-dev avec : approche choisie Item 1, baselines préservées, justification trade-offs.)

### File List

- `crates/kesh-api/src/config.rs` — modifié (module test isolation env étendue OU annotations `#[serial(env)]` selon approche T3).
- `crates/kesh-api/src/routes/reports.rs` — modifié (2 fonctions `emit_*_audit` keys snake_case + commentaire convention §audit_log).
- `crates/kesh-api/tests/reports_e2e.rs` — modifié (5 assertions audit_log snake_case + commentaire mis à jour).
- `crates/kesh-api/Cargo.toml` — modifié uniquement si nouvelle dev-dependency ajoutée T3.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — modifié (status `9-5-2` cycle ready-for-dev → in-progress → review → done).
- `_bmad-output/implementation-artifacts/9-5-2-code-consistency-fixes.md` — cette spec, Change Log + Dev Agent Record + status review.

## Change Log

### Pass 1 spec validate — 2026-05-18, Sonnet 4.6 (subagent contexte frais)

**Verdict trend** : 0 CRITICAL + 2 HIGH + 1 MEDIUM + 2 LOW = 5 findings (Convergence : NON).

**Discipline grep ground-truth Sonnet** appliquée — 11+ vérifications confirmées par grep/Read direct sur les fichiers source. 2 hallucinations spec attrapées (la spec Opus 4.7 affirmait des faits FAUX).

**Patches appliqués (3/5 — F-04 et F-05 LOW non bloquants)** :

1. **F-01 HIGH** — Hypothèse cause root incorrecte. Opus 4.7 avait affirmé « `reset_env()` ne purge pas `KESH_TEST_MODE` ». **Ground-truth Sonnet** : `grep -n "KESH_TEST_MODE.*remove_var" config.rs` → ligne **725** purge bien `KESH_TEST_MODE` (en plus de 14 autres vars). **Vraie cause root** : `dotenvy::dotenv().ok()` ligne 309 inconditionnel dans `from_env()` recharge `.env` projet (KESH_HOST=0.0.0.0 + KESH_TEST_MODE=true) après chaque `reset_env()`. Confirmation in-code : le test `config_from_env_with_database_url` ligne 742 documente déjà ce comportement. **Patch** : Dev Notes §Cause root entièrement réécrite, option (d) `#[cfg(not(test))]` autour `dotenvy::dotenv()` recommandée comme solution principale, options (a)/(b)/(c) reclassées « insuffisantes seules » avec justification.

2. **F-02 HIGH** — Comptage tests incorrect. La spec disait « 24/24 tests » mais `grep -c "^    #\[test\]$" config.rs` = **34 tests**. L'epic-9-5.md avait « 20/24 » (ancien chiffre obsolète). **Patch** : `24/24` → `34/34` partout (6 occurrences §Story/AC #1/T3.4/T3.5/T8.4/Testing standards). §Story reformulée pour « N tests fail à diagnostiquer sur 34 totaux ».

3. **F-03 MEDIUM** (couvert par P1) — Option (a) recommandée n'adresse pas la cause root réelle. Option (d) substituée comme primaire. AC #2 reformulé pour mentionner option (d) recommandée + option (d-bis) wrapper alternative explicite. AC #3 confirmé par grep ground-truth aucun test ne dépend de `.env` chargement.

**Findings non-patchés (2 LOW)** :

- **F-04 LOW** — Confirmation positive : `emit_global_export_audit` ligne 140 exports.rs confirmé snake_case canonique. Aucune action requise.
- **F-05 LOW** — Nit T6.2 commentaire AC #25 reference. Optionnel — préciser story source post-migration dans le commentaire test mis à jour.

**Checks ground-truth additionnels confirmés (10+) sans finding** : `emit_report_audit` ligne 678 ✓, `emit_report_export_audit` ligne 728 ✓, 5 keys camelCase emit_report_audit (697-701) ✓, 6 keys camelCase emit_report_export_audit (748-754) ✓, 5 assertions test E2E (978-991) ✓, 0 test pour `report.exported` ✓, `serial_test`/`temp-env` absents Cargo.toml ✓, `dotenvy::dotenv().ok()` ligne 309 ✓, structs query params `#[serde(rename_all = "camelCase")]` confirmé `ReportQuery`/`JournalReportQuery`/`ExportQuery`, aucune SQL JSON path camelCase en prod ✓, aucune autre assertion camelCase audit_log dans tests/ ✓.

**Leçon** : même en clamant « discipline grep ground-truth », Opus 4.7 en main loop a sauté la vérification sur l'hypothèse cause root. Sonnet 4.6 en subagent isolé a attrapé la faute. Pattern valide : **les subagents isolés (contexte frais) sont plus rigoureux que la main loop**.

**Trend** : Pass 1 (Sonnet) 0C+2H+1M+2L → 3 patches → Pass 2 Haiku 4.5 attendue (cycle CLAUDE.md).

**Modèle Pass 1** : Sonnet 4.6 (subagent isolé, contexte frais — spec créée par Opus 4.7).
