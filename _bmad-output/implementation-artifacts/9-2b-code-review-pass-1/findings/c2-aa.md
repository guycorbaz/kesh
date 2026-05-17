# Acceptance Auditor — Chunk 2 : repos + i18n + deps

**Story** : 9-2b Export global ZIP  
**Scope chunk** : `crates/kesh-db/src/repositories/*.rs` (10 nouvelles fns) + `crates/kesh-i18n/locales/*` (12 clés × 4 locales) + `crates/kesh-api/Cargo.toml` + `Cargo.lock`  
**Date** : 2026-05-17

---

## Résumé exécutif

Chunk 2 conforme sur l'essentiel : 10 nouvelles fns `list_all_by_company`/`list_all_lines_by_company` avec signatures correctes (`pool, company_id -> Result<Vec<Entity>, DbError>`), absence de filtres `active`/`status` dans les clauses `WHERE` (souveraineté respectée), JOIN single-query anti-N+1 pour `journal_entry_lines` et `invoice_lines`, 12 clés i18n identiques dans les 4 locales, naming convention `export-global-*` / `nav-export-global` / `error-global-export-failed` conforme, et `zip = "2"` seul ajouté (sha2 + csv déjà présents). Un seul finding de sévérité LOW (bug de documentation Rust).

**Verdict global : PASS — 0 finding > LOW**

---

## Findings

### LOW — Doc comment orphelin sur `reconciliation_rules::list_all_by_company`

**AC / Décision violée** : T3.2.9 (qualité doc Rust, pas d'AC fonctionnel)  
**Fichier:ligne** : `crates/kesh-db/src/repositories/reconciliation_rules.rs:45-57`

La nouvelle fn `list_all_by_company` a été insérée directement après les dernières lignes du bloc `///` de `find_active_for_company` (contexte diff : `/// flow GET /api/v1/reconciliation/proposals.`). En Rust, un bloc `///` continu est attaché au premier item qui suit — `list_all_by_company` hérite donc du doc de `find_active_for_company` (« Liste toutes les rules actives…  Utilisé par kesh_reconciliation::rules::first_matching_rule »), et `find_active_for_company` se retrouve sans doc_comment du tout. Pas de bug fonctionnel, mais `cargo doc` génère une documentation trompeuse pour les deux fonctions.

**Correction suggérée** : ajouter une ligne vide entre `/// flow GET /api/v1/reconciliation/proposals.` et le début du bloc doc de `list_all_by_company`, ou insérer la nouvelle fn APRÈS `find_active_for_company` avec son propre bloc doc propre.

---

## Vérifications réussies (conformité AC / Décisions)

| Vérification | Résultat |
|---|---|
| T3.2.1–T3.2.10 : 10 fns créées (8 `list_all_by_company` + 2 `list_all_lines_by_company`) | ✓ 10 fns présentes dans le diff |
| Signatures exactes `pool: &MySqlPool, company_id: i64 -> Result<Vec<Entity>, DbError>` | ✓ Toutes conformes |
| Absence filtre `active`/`status` dans WHERE (T3.2.3/T3.2.4/T3.2.8/T3.2.9 souveraineté) | ✓ Colonnes `status`/`active` apparaissent en SELECT list seulement |
| T3.2.2/T3.2.5 — JOIN single-query anti-N+1 pour `journal_entry_lines` et `invoice_lines` | ✓ `JOIN journal_entries je ON jel.entry_id = je.id WHERE je.company_id = ?` et `JOIN invoices i ON il.invoice_id = i.id WHERE i.company_id = ?` |
| T3.2.2 — Tri `ORDER BY jel.entry_id, jel.line_order` (spec) | ✓ Conforme |
| T3.2.5 — Tri `ORDER BY il.invoice_id, il.position` (spec) | ✓ Conforme |
| T1.1 — `zip = { version = "2", default-features = false, features = ["deflate"] }` | ✓ Conforme (spec file notes: « zip = "2" (default-features=false + deflate) ») |
| T1.2 — `sha2` NON re-ajouté (déjà présent depuis Story 8-1b) | ✓ Ligne de contexte uniquement |
| T1.1 note — `csv` NON re-ajouté (déjà présent depuis Story 9-2a) | ✓ Ligne de contexte uniquement |
| T1.3 — `hex` NON ajouté (Decision §hex-encoding) | ✓ Absent du diff |
| Cargo.lock — `zip 2.4.2` résolu | ✓ Présent |
| T11 — 12 clés × 4 locales (fr-CH / de-CH / it-CH / en-CH) | ✓ 12/12 dans chaque locale |
| T11 — Naming `export-global-*` (10 clés) + `nav-export-global` + `error-global-export-failed` | ✓ Conforme spec T11.1 |
| T11 — Pas d'accent dans les noms de clés (`souverainete` sans accent) | ✓ Conforme |
| bank_profiles.rs — pattern `format!("SELECT {} FROM ...", COLUMNS)` cohérent avec le reste du fichier | ✓ Cohérent style existant |
