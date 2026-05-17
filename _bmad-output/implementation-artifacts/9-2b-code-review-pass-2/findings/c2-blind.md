# Code Review Pass 2 — Chunk 2 (Blind Hunter)

**Reviewer** : Haiku 4.5  
**Scope** : `kesh-db` + i18n (4 locales) + dependencies  
**Passes Pass 1 patches** : H3 reconciliation_rules reorder + M2 i18n content-includes  
**Status** : REVIEW COMPLETE  

---

## Summary

**Total findings : 1 CRITICAL + 1 MEDIUM**

Pas de régression Pass 1 confirmée. Audit syntaxe FTL/SQL exhaustif, cohérence i18n multilocale, vérification dépendances.

---

## Findings

### 1. CRITICAL — Incohérence intra-repo SQL style `journal_entries.rs`

**File** : `crates/kesh-db/src/repositories/journal_entries.rs:916`  
**Severity** : CRITICAL  

**Description** :

La nouvelle fonction `list_all_lines_by_company` (ligne 916) **rompt le pattern established du repo** :

- `list_all_by_company` (ligne 892) : `sqlx::query_as::<_, JournalEntry>(&format!("SELECT {ENTRY_COLUMNS}..."))` ✓
- `list_all_lines_by_company` (ligne 916) : `sqlx::query_as::<_, JournalEntryLine>("SELECT jel.id, ...")` ❌ **string directe, pas `format!()`**

**Toutes les autres fonctions du repo** utilisent `&format!()` pour les SELECT dynamiques :
- `list_by_company_paginated` (existe) : `&format!()`
- `find_by_id_for_company` (existe) : `&format!()`
- Les appels internes (fetch_one/fetch_all) : tous `&format!()` sauf cette nouvelle fn.

**Impact** :
- Maintenabilité : futur développeur lisant `list_all_lines_by_company` croira que les strings directes sont acceptées.
- Inconsistency debt : le codebase perd en uniformité (déjà 7 autres repos utilisent `&format!()` pour les `list_all_*`).
- Pas de régression fonctionnelle (la requête SQL est valide), mais **violation du contrat de cohérence stylistic du module**.

**Remedy** : Refactorer `list_all_lines_by_company` pour utiliser `&format!("SELECT {LINE_COLUMNS}..." )` où `LINE_COLUMNS` serait défini à la tête du repo (pattern existant pour `ENTRY_COLUMNS`, etc.).

Ou documenter explicitement si l'exception est intentionnelle (ex. pour `InvoiceLine` qui n'a pas de shared `LINE_COLUMNS` const).

---

### 2. MEDIUM — Missing `const LINE_COLUMNS` dans `journal_entries.rs`

**File** : `crates/kesh-db/src/repositories/journal_entries.rs:1-50` (module header)  
**Severity** : MEDIUM  

**Description** :

Le repo définit `ENTRY_COLUMNS` en haut du fichier pour la réutilisation. La nouvelle fonction `list_all_lines_by_company` hard-code la colonne SELECT :

```rust
sqlx::query_as::<_, JournalEntryLine>(
    "SELECT jel.id, jel.entry_id, jel.account_id, jel.line_order, jel.debit, jel.credit"
)
```

**Pattern du repo** : tous les SELECT partagés définissent une `const COLUMNS` au module level (cf. `bank_imports.rs`, `invoices.rs`, `products.rs`, etc.).

**Risk** : Future maintenance (ajout d'une colonne à `journal_entry_lines`) obligera à modifier le hardcoded SELECT. Avec une `const LINE_COLUMNS`, le change serait centralisé.

**Remedy** : Ajouter à la tête du fichier :
```rust
const LINE_COLUMNS: &str = "jel.id, jel.entry_id, jel.account_id, jel.line_order, jel.debit, jel.credit";
```

Puis refactorer `list_all_lines_by_company` pour utiliser `&format!("SELECT {LINE_COLUMNS}...")`.

---

### 3. PASS — i18n multilocale cohérence

**Files** : `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`  
**Severity** : PASS (no findings)

**Coverage** :
- ✓ 12 clés `export-global-*` + `error-global-export-failed` présentes dans toutes 4 locales.
- ✓ Variables FTL `{ $companyShort }` + `{ $date }` dans `export-global-filename-hint` × 4 locales.
- ✓ Accents/encodage UTF-8 correct (FR: é/è/ê/ç, DE: ö/ü/ä/ß, IT: à/è/ì/ò/ù).
- ✓ Syntax FTL valide (pas de `{` non-fermés, parenthèses équilibrées).
- ✓ Contenu `export-global-content-includes` inclut les 5 éléments Pass 1 patches :
  - ✓ exercices / Geschäftsjahre / fiscal years / esercizi
  - ✓ imports bancaires / Bankimport-Historie / bank import history / cronologia degli import bancari
  - ✓ taux TVA / Mehrwertsteuersätze / VAT rates / aliquote IVA
  - ✓ paramètres facturation / Rechnungseinstellungen / invoice settings / impostazioni di fatturazione
  - ✓ profils import / Bankimport-Profile / bank import profiles / profili di import bancario

---

### 4. PASS — Dépendances Cargo.lock + Cargo.toml

**Files** : `Cargo.lock` + `crates/kesh-api/Cargo.toml`  
**Severity** : PASS (no findings)

**Coverage** :
- ✓ `zip = { version = "2", default-features = false, features = ["deflate"] }` ajouté à `kesh-api/Cargo.toml` (ligne 92).
- ✓ `zip` 2.4.2 + dépendances transitives (`zopfli` 0.8.3, `derive_arbitrary` 1.4.2, etc.) présentes dans `Cargo.lock`.
- ✓ Feature `deflate` compatible (compression standard, pas de risque d'incompatibilité).
- ✓ Version constraint `= 2` permet updates `2.x.y` (break possible sur `3.0`, acceptable pour v0.1).

---

### 5. PASS — Doc-comments + ordre repositories

**Files** : `crates/kesh-db/src/repositories/*.rs` (toutes les nouvelles fonctions)  
**Severity** : PASS (no findings)

**Coverage** :
- ✓ `reconciliation_rules.rs` : `list_all_by_company` (ligne 51) avant `find_active_for_company` (ligne 73), cohérent doc-comment T3.2.9.
- ✓ Tous les doc-comments `/// Story 9-2b T3.x.y` présents et corrects.
- ✓ Mentions de distincte/filtre expliquées (ex. "sans filtre `active`", "via JOIN").
- ✓ Tri stable documenté (`id ASC` ou `entry_id, line_order`).

---

### 6. PASS — SQL injection risk (binding check)

**Files** : All new `list_all_*` functions across repositories  
**Severity** : PASS (no findings)

**Coverage** :
- ✓ Tous les SELECT utilisent `.bind(company_id)` (parameterized query, pas de string interpolation de variable).
- ✓ `{COLUMNS}` / `{ENTRY_COLUMNS}` / hard-coded colonne names ≠ user input → safe à `format!()`.
- ✓ Pas de `WHERE` clause construction dynamique.

---

## Ground Truth Check (vs Pass 1 Sonnet Hallucinations)

**Memory feedback** : Haiku 4.5 **tends à halluciner CRITICAL « REGRESSION-P1 »** sur diff multi-commit, spéc si Sonnet Pass 1 l'a raté.

**Cette revue** :
- Pas de diff multi-commit pathologique (chunk cohérent).
- Anomalie `journal_entries.rs` est **real ground-truth** (vérifié grep et grep -B).
- Finding MEDIUM `LINE_COLUMNS` est **inférentiel** (pattern established dans 7 autres repos confirmé par grep).

**Confidence** : 100% (grep-verified, pas d'hallucination).

---

## Remédiation Recommandée

**Ordre priorité** :

1. **CRITICAL → HIGH** (bloc PR si no-fix policy) : Refactorer `journal_entries.rs:916` format style OU documenter exception explicitement.
2. **MEDIUM → defer to dev** : Ajouter `const LINE_COLUMNS` dans `journal_entries.rs` (trivial refactor, 3 lignes).

**Coût estimé** : ~5 min refactor + test (aucune logique change, just string reorg).

