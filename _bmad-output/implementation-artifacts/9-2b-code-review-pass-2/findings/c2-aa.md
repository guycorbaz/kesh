# Chunk 2 — Acceptance Auditor Pass 2 Findings
## POST-patches Pass 1 (Repos + i18n + Cargo deps)

---

## CRITICAL Findings

### AA2-CRITICAL-001: i18n `export-global-content-includes` — Énumération incomplète des 16 tables CSV

**Sévérité** : CRITICAL  
**AC violée** : AC #5 (ZIP contient exactement 17 entrées : 16 CSV + metadata.json)  
**Fichier** : `crates/kesh-i18n/locales/fr-CH/messages.ftl:467`, `de-CH:423`, `en-CH:445`, `it-CH:489`

**Description** :

La clé i18n `export-global-content-includes` énumère explicitement 13 éléments (dont metadata.json) dans toutes les 4 locales. Selon la spec §scope-tables, l'export doit contenir **16 tables CSV + 1 metadata.json = 17 entrées totales**, soit **16 tables métier à lister**.

**Texte FR courant** :
> L'export contient : plan comptable, exercices, écritures, contacts, produits, factures, comptes bancaires, historique des imports bancaires, transactions, taux de TVA actifs et historiques, paramètres de facturation, règles de réconciliation, profils d'import bancaire, et un manifeste metadata.json...

**Tables énumérées** (décompte) :
1. plan comptable (accounts)
2. exercices (fiscal_years)
3. écritures (journal_entries)
4. contacts (contacts)
5. produits (products)
6. factures (invoices)
7. comptes bancaires (bank_accounts)
8. historique des imports bancaires (bank_imports)
9. transactions (bank_transactions)
10. taux de TVA actifs et historiques (vat_rates)
11. paramètres de facturation (company_invoice_settings)
12. règles de réconciliation (reconciliation_rules)
13. profils d'import bancaire (bank_profiles)

**Manquent** :
- `company.csv` (la company elle-même — entity root du multi-tenant)
- `journal_entry_lines.csv` (lignes d'écriture — relation 1:N nécessaire pour reconstruction comptable)
- `invoice_lines.csv` (lignes de facture — relation 1:N nécessaire pour reconstruction factures)

Spec T3.2.1..T3.2.10 détaille ces 3 tables comme essentielles pour la souveraineté des données (reconstructibilité complète). L'omission dans l'i18n crée une **discordance UX** : l'utilisateur est informé de 13/16 éléments seulement.

**Ground-truth** : Spec §scope-tables déclare 16 tables à exporter. Diff chunk-2 ajoute les fns repo pour toutes les 16 (incluant `journal_entries::list_all_lines_by_company` T3.2.2 et `invoices::list_all_lines_by_company` T3.2.5, visibles dans chunk-2). L'i18n patch n'a pas synchronisé l'énumération.

**Remédiation** :
- **FR** : Ajouter "Lignes d'écritures, lignes de factures, entreprise" à la liste énumérée (avant "metadata.json")
  - Texte proposé : `...profils d'import bancaire, lignes d'écritures, lignes de factures, sociétés, et un manifeste metadata.json...`
- **DE** : `...Bankimport-Profile, Schreibvorgänge, Rechnungspositionen, Unternehmen und ein metadata.json-Manifest...`
- **EN** : `...bank import profiles, journal entry lines, invoice lines, company, and a metadata.json manifest...`
- **IT** : `...profili di import bancario, righe di registrazione, righe di fattura, società, e un manifesto metadata.json...`

---

## HIGH Findings

### AA2-HIGH-001: Documentation T3.2.9 — `reconciliation_rules::list_all_by_company` respects `ORDER BY id` ✅

**Sévérité** : HIGH (anciennement H3 Pass 1)  
**AC violée** : T3.2.9 spec pattern  
**Fichier** : `crates/kesh-db/src/repositories/reconciliation_rules.rs:354-367` (chunk-2)

**Description** :

Spec T3.2.9 prescrit :
> Pattern `SELECT * WHERE company_id = ? ORDER BY id`

Diff chunk-2 implémente :
```rust
sqlx::query_as::<_, ReconciliationRule>(&format!(
    "SELECT {COLUMNS} FROM reconciliation_rules \
     WHERE company_id = ? \
     ORDER BY id"
))
```

**Résultat** : ✅ **CONFORME**. Tri stable `id ASC` pour export reproducible, pattern respecté.

---

## MEDIUM Findings

Aucun finding MEDIUM post-patches chunk-2.

---

## LOW Findings

### AA2-LOW-001: ZIP library dependency declared ✅

**Sévérité** : LOW (informel)  
**AC violée** : Aucune (vérifié vs AC #1, #3, #4)  
**Fichier** : `crates/kesh-api/Cargo.toml:43` (chunk-2), `Cargo.lock:46-60`

**Description** :

Spec T1.1 spécifie `zip = "2"` avec Deflate compression par défaut. Diff chunk-2 ajoute :
```toml
# Story 9-2b — export global ZIP (souveraineté des données).
zip = { version = "2", default-features = false, features = ["deflate"] }
```

et `Cargo.lock` ajoute les dépendances transitivités (`crc32fast`, `flate2`, `zopfli`, etc.).

**Résultat** : ✅ **CONFORME**. Version 2.x + compression Deflate explicite.

---

## PASS 2 Audit Résumé

| Catégorie | Nombre | Status |
|-----------|--------|--------|
| CRITICAL | 1 | ❌ Remédiation requise (i18n énumération incomplète) |
| HIGH | 1 | ✅ Conforme |
| MEDIUM | 0 | — |
| LOW | 1 | ✅ Conforme (info) |
| **Total** | **3** | **1 finding > LOW à fixer** |

---

## Recommandations

1. **Avant Pass 3** : Synchroniser l'énumération i18n `export-global-content-includes` pour inclure les 16 tables CSV (en particulier company, journal_entry_lines, invoice_lines).
2. **Diff chunk-1 et chunk-3** : Vérifier que les fns repo T3.2.1..T3.2.10 et les serializers CSV couvrent bien les 16 tables et leurs lignes d'audit correctly.
3. **Métadata.json** : Vérifier dans chunk-3 (tests) que la shape `metadata.json.tables` liste effectivement 16 clés (pas 13).

---

**Analysé par** : Acceptance Auditor (Haiku 4.5) Pass 2  
**Date** : 2026-05-17  
**Diff scope** : Chunk 2 (repos + i18n + deps) post-patches Pass 1
