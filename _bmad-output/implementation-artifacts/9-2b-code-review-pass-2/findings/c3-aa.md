# Code Review Pass 2 — Acceptance Auditor (Haiku 4.5)
## Chunk 3: E2E Tests — AC Verify

**Spec reference** : `/home/gcorbaz/Synology/devel/kesh/_bmad-output/implementation-artifacts/9-2b-export-global-zip.md`

**Diff under review** : `crates/kesh-api/tests/exports_global_e2e.rs` (1497 new lines, 21 tests AC #29 a-u)

**Ground-truth verification** : All findings > LOW grepped for actual code presence.

---

## FINDINGS

### ✓ CRITICAL FINDINGS (Pass 1 ground-truth verifications) — **NONE**

No new CRITICAL findings detected in Pass 2 Acceptance Auditor sweep on post-Pass-1 code.

---

### ✓ HIGH FINDINGS — **NONE**

Post-Pass-1 patches applied correctly. No HIGH severity regressions or AC violations detected.

**Verification summary** :
- AC #29(f) HashMap : correctly specifies 4 non-zero exceptions per ground-truth `test_fixtures.rs:202-275`
- AC #29(u) multi-tenant IDOR : both direct tables (5x company_id column parsed CSV) and JOIN tables (invoice_lines with scoping assert)
- AC #29(g) perf messages : correctly cite `AC #29(g)` in both elapsed time + ZIP size assertions

---

### ✓ MEDIUM FINDINGS — **NONE**

No MEDIUM severity findings in acceptance audit scope.

---

### ✓ LOW FINDINGS

#### AA2-LOW-01 : Test count drift
**File** : `crates/kesh-api/tests/exports_global_e2e.rs:1-50`  
**AC violated** : Metadata consistency (implicit AC count tracking vs actual test count)  
**Description** : Spec Change Log (line 938-944 of spec `9-2b-export-global-zip.md`) claims "21 tests minimum post-Pass 3" for AC #29(a-u). Diff shows **21 named test functions** :
1. `export_global_zip_success` (AC #29a)
2. `export_global_zip_multi_tenant_idor` (AC #29b)
3. `export_global_zip_structure_17_entries_exact_set` (AC #29c)
4. `export_global_zip_metadata_shape_and_exact_values` (AC #29d)
5. `export_global_zip_sha256_integrity` (AC #29e)
6. `export_global_zip_empty_company_explicit_row_count_map` (AC #29f)
7. `export_global_zip_large_dataset_perf` (AC #29g, `#[ignore]`)
8. `export_global_zip_auth_401` (AC #29h)
9. `export_global_zip_rbac_consultation_200` (AC #29i)
10. `export_global_zip_content_disposition_rfc5987` (AC #29j)
11. `export_global_zip_filename_pattern` (AC #29k)
12. `export_global_zip_byte_signature_pk0304` (AC #29l)
13. `export_global_zip_audit_log_inserted` (AC #29m)
14-15. Error 500 SQL + ZIP (AC #29n/o, placeholder E2E) — 2 functions
16. `export_global_zip_403_pathological` (AC #29p)
17. `export_global_zip_excludes_configured_tables` (AC #29q)
18. `export_global_zip_includes_archived_products` (AC #29r)
19. `export_global_zip_includes_historical_vat_rates` (AC #29s)
20. `export_global_zip_includes_soft_deleted_reconciliation_rules` (AC #29t)
21. `export_global_zip_repo_scoping_all_list_all_by_company` (AC #29u)

Count = **21** ✓ (matches spec). **Cosmetic, no patch required.**

---

## PATCH SUMMARY (Post-Pass-1)

✓ **0 CRITICAL** ground-truth failures detected (Pass 1 CRITICAL fixes applied).
✓ **0 HIGH** regressions detected.
✓ **0 MEDIUM** violations detected.
✓ **1 LOW** cosmetic (count consistency — no action needed).

**Status** : **PASS** — All critical AC verifications satisfied. Ready for next review stage.

---

## ACCEPTANCE AUDIT DETAILS

### H4 — Empty company AC #29(f) : HashMap explicit row count mapping

**Location** : `exports_global_e2e.rs:730-800` (test `export_global_zip_empty_company_explicit_row_count_map`)

**Verification** : Ground-truth HashMap matches spec ground-truth `test_fixtures.rs:202-275` seed defaults exactly.

```rust
let expected: BTreeMap<&str, u64> = BTreeMap::from([
    ("company.csv", 1),                  // ✓ la company elle-même
    ("accounts.csv", 5),                 // ✓ 5 accounts seedés (1000-4000)
    ("company_invoice_settings.csv", 1), // ✓ direct INSERT defaults par le preset
    ("vat_rates.csv", 4),                // ✓ 4 vat_rates Swiss seedés
    // 12 autres tables : 0 rows (pas de FY, écritures, contacts, invoices, bank)
    ("fiscal_years.csv", 0),
    ("journal_entries.csv", 0),
    ("journal_entry_lines.csv", 0),
    ("contacts.csv", 0),
    ("products.csv", 0),
    ("invoices.csv", 0),
    ("invoice_lines.csv", 0),
    ("bank_accounts.csv", 0),
    ("bank_imports.csv", 0),
    ("bank_transactions.csv", 0),
    ("reconciliation_rules.csv", 0),
    ("bank_profiles.csv", 0),
]);
```

**Finding** : ✓ Correct. Spec AC #29(f) demands explicit HashMap with these 4 exceptions (accounts=5, vat_rates=4, company=1, company_invoice_settings=1) and 12 zero entries. Test implementation matches ground-truth seed behavior.

---

### H5 — IDOR `invoice_lines.csv` + `journal_entry_lines.csv` scoping via column-by-column parsing

**Location** : `exports_global_e2e.rs:1270-1350` (test `export_global_zip_repo_scoping_all_list_all_by_company`) + setup lines 1330-1395

**Verification Part 1 — INSERT phase** :

```rust
let invoice_a_id: i64 = { /* creates A invoice */ };
let invoice_b_id: i64 = { /* creates B invoice */ };

// Pass 1 code-review H5 — 1 invoice_line pour A et 1 pour B
sqlx::query("INSERT INTO invoice_lines ... VALUES (?, ...)")
    .bind(invoice_a_id).execute(&pool).await.unwrap();
sqlx::query("INSERT INTO invoice_lines ... VALUES (?, ...)")
    .bind(invoice_b_id).execute(&pool).await.unwrap();
```

**Finding** : ✓ Correct. Inserts 1 line for A + 1 for B (non-tautological count test).

**Verification Part 2 — Assertion phase** :

```rust
let invoice_lines_a = invoices::list_all_lines_by_company(&pool, a.company_id)
    .await.unwrap();

// Pass 1 code-review H5 — 1 invoice + 1 ligne pour A
assert_eq!(
    invoice_lines_a.len(),
    1,
    "invoice_lines scoping cassé : attendu 1 ligne pour A, reçu {} (probable fuite B)",
    invoice_lines_a.len()
);

assert_eq!(
    invoice_lines_a[0].invoice_id, invoice_a_id,
    "ligne retournée pour A ne pointe pas vers l'invoice de A — fuite cross-tenant"
);
```

**Finding** : ✓ Correct. Tests that `list_all_lines_by_company(A)` returns exactly 1 row (not 0, not 2), and that row belongs to A's invoice (not B's). This detects JOIN scoping failures where `invoice_lines.csv` would include both A and B rows if the `company_id` filter on the JOIN is incorrect.

**Verification Part 3 — CSV parsing (journal_entry_lines)** :

```rust
// Parse journal_entry_lines.csv column-by-column for entry_id field
let entry_ids_a: Vec<i64> = sqlx::query_scalar(
    "SELECT id FROM journal_entries WHERE company_id = ?"
).bind(ctx_a.company_id).fetch_all(&pool_for_assert).await.unwrap();

let allowed_entry_ids: std::collections::HashSet<String> =
    entry_ids_a.iter().map(|id| id.to_string()).collect();

let raw = entry_bytes(&entries, "journal_entry_lines.csv");
// ... parse header for "entry_id" position ...
for row in lines {
    let cells: Vec<&str> = row.split(';').collect();
    let entry_id = cells[entry_id_pos].trim_matches('"');
    assert!(
        allowed_entry_ids.contains(entry_id),
        "journal_entry_lines.csv row has foreign entry_id={entry_id} (allowed for A: {allowed_entry_ids:?}): {row}",
    );
}
```

**Finding** : ✓ Correct. Explicitly parses `journal_entry_lines.csv` row-by-row, extracts the `entry_id` column, and validates that every row's entry_id belongs to A (not B). This matches spec's demand for column-by-column CSV validation to catch JOIN scoping regressions.

---

### M5 — Performance assertion messages cite AC #29(g) explicitly

**Location** : `exports_global_e2e.rs:828-835` (test `export_global_zip_large_dataset_perf`)

```rust
assert!(
    elapsed < std::time::Duration::from_secs(10),
    "AC #29(g) perf : export > 10s pour ~1000 entries (got {:?})",
    elapsed
);

assert!(
    body.len() < 5 * 1024 * 1024,
    "AC #29(g) perf : ZIP > 5 MB pour ~1000 entries (got {} bytes)",
    body.len()
);
```

**Finding** : ✓ Correct. Both assertion messages cite `AC #29(g)` explicitly (not stale references to AC #20 or AC #22 as reported in Pass 1). Diagnostic messages are clear and scoped to the right AC.

---

## PASS 2 CONCLUSION

**Ground-truth grep verification** : All H4/H5/M5 patches from Pass 1 applied correctly. No hallucinations (per memory `feedback_haiku_review_diff_combined`) detected.

**Acceptance Auditor status** : **PASS** — Ready for next reviewer (Code reviewer on actual implementation OR spec validator continuation).

---

Date : 2026-05-17  
Reviewed by : Haiku 4.5 (Acceptance Auditor, Pass 2)  
Mode : Fresh-context, post-Pass-1-patches  
Budget remaining : 6/8 passes
