# Chunk 2 — Blind Hunter Findings
**Revieweur** : Blind Hunter (Pass 1)
**Scope** : kesh-db repos × 10 fns + i18n × 4 locales + Cargo deps
**Diff** : `chunk-2-repos-i18n-deps.diff` (~492 lignes)
**Date** : 2026-05-17

---

## F-C2-01 — HIGH — Doc comment de `find_active_for_company` fusionné avec celui de `list_all_by_company`
**Fichier** : `crates/kesh-db/src/repositories/reconciliation_rules.rs:45-71`

Le bloc `///` aux lignes 45-57 contient la documentation **de deux fonctions distinctes** : les 5 premières lignes décrivent `find_active_for_company` (trie par `priority ASC, id ASC`, utilisé par le flow proposals), les 5 suivantes décrivent `list_all_by_company`. Or ce bloc entier est attaché à `list_all_by_company` (ligne 58). En conséquence :

- `list_all_by_company` a un doc comment qui mentionne un comportement qui n'est pas le sien (tri priority, lien vers `first_matching_rule`).
- `find_active_for_company` (ligne 73) n'a **plus aucun** doc comment — elle est tombée dans l'oubli lors de l'insertion de la nouvelle fn.

`rustdoc` et les IDEs afficheront la mauvaise documentation. Toute équipe cherchant la fonction "utilisée par proposals" lira un doc trompeur.

**Fix** : scinder le bloc en deux `///` séparés, chacun attaché à sa fn.

---

## F-C2-02 — HIGH — `ORDER BY entry_date ASC` contre un index `(company_id, entry_date DESC)` → filesort systématique sur export
**Fichier** : `crates/kesh-db/src/repositories/journal_entries.rs:897-899`

```sql
WHERE company_id = ? ORDER BY entry_date, id
```

L'index existant est `idx_journal_entries_company_date (company_id, entry_date DESC)`. MariaDB/InnoDB ne peut pas inverser la direction de lecture d'un index B-Tree pour honorer `ORDER BY entry_date ASC` — il effectuera un **filesort** après le scan partiel par `company_id`. Pour une company avec 5 000+ écritures, c'est une sort sur un buffer en mémoire ou disque, hors pagination.

Pour les autres tables, `ORDER BY id` se résout via la PK (clustered index) sans filesort, ce qui est acceptable. Le problème est **spécifique** à `journal_entries` dont le tri métier est `entry_date, id` et non `id` seul.

**Fix** : soit passer à `ORDER BY id` (conforme aux autres fns), soit ajouter un index `(company_id, entry_date ASC, id ASC)` dédié export.

---

## F-C2-03 — MEDIUM — Dépendance `zopfli` (compresseur C-wrapper) tirée par `zip = "deflate"` malgré `default-features = false`
**Fichier** : `crates/kesh-api/Cargo.toml:44`, `Cargo.lock:5071-5112`

La déclaration `zip = { version = "2", default-features = false, features = ["deflate"] }` vise à minimiser les dépendances. Pourtant, le Cargo.lock résolu inclut `zopfli 0.8.3` (+ ses deps `bumpalo`, `simd-adler32`) : ces paquets n'étaient **pas** présents dans le lock avant ce commit (confirmé par `git show b891bff:Cargo.lock | grep zopfli` → vide). La feature `"deflate"` dans zip 2.4.x active `deflate-zopfli` en plus de `deflate-miniz` — comportement non documenté dans le changelog zip 2.x.

Impact : surface de code binaire augmentée, zopfli ajoute ~120 Ko en release, et sa licence (Apache-2.0 avec clause de restriction sur benchmarks) mérite validation.

**Fix** : auditer si `features = ["deflate-miniz"]` (ou équivalent non-zopfli dans zip 2.4.x) élimine cette dépendance. Si non, documenter l'acceptation explicite.

---

## F-C2-04 — MEDIUM — Isolation multi-tenant `invoice_lines` non prouvée par le test scoping
**Fichier** : `chunk-3-e2e-tests.diff:1422-1426` (hors scope direct chunk 2, mais découle de la fn `list_all_lines_by_company` dans ce chunk)

Le test `export_global_zip_repo_scoping_all_list_all_by_company` vérifie l'isolation de 8 tables `list_all_by_company`. Pour `invoices::list_all_lines_by_company`, il insère **0 `invoice_lines`** et assert `len == 0`. Ce test ne prouve **pas** que la requête JOIN isole correctement les lignes d'une company A des lignes de company B — si la condition `WHERE i.company_id = ?` était absente, le test passerait quand même (zéro ligne des deux côtés). La requête SQL elle-même est correcte, mais l'absence de preuve par données concrètes laisse un angle mort dans la suite de régression.

**Fix** : insérer au moins 1 `invoice_line` pour company A **et** 1 pour company B, puis assert que `list_all_lines_by_company(A)` retourne exactement 1 et `[0].invoice_id` appartient à A.

---

## F-C2-05 — MEDIUM — `content-includes` i18n omet 3 tables réellement exportées (`vat_rates`, `bank_imports`, `bank_profiles`)
**Fichiers** : tous les `messages.ftl:export-global-content-includes`

La clé `export-global-content-includes` liste dans les 4 locales : *plan comptable, écritures, contacts, produits, factures, comptes bancaires, transactions, règles de réconciliation*. Or le handler ZIP exporte **également** :
- `vat_rates.csv` (taux TVA historiques)
- `bank_imports.csv` (historique des imports)
- `bank_profiles.csv` (profils d'import bancaire)

Ces 3 fichiers sont présents dans le ZIP mais absents de la description affichée à l'utilisateur. L'utilisateur ne peut donc pas vérifier exhaustivement l'intégrité de l'export contre ce qui est annoncé, ce qui contredit la promesse de souveraineté des données.

**Fix** : ajouter ces 3 éléments dans la clé `content-includes` × 4 locales. Recalibrer la description courte (`export-global-description`) en conséquence si nécessaire.

---

## F-C2-06 — MEDIUM — Incohérence de style SQL : colonnes hard-codées dans `list_all_by_company` / `list_all_lines_by_company` d'`invoices.rs` alors qu'une constante `LINE_COLUMNS` existe
**Fichier** : `crates/kesh-db/src/repositories/invoices.rs:203-214` et `:226-241`

`invoices.rs` possède `const LINE_COLUMNS` (ligne 37-38) mais `list_all_by_company` hard-code les 14 colonnes de `Invoice` et `list_all_lines_by_company` hard-code les 9 colonnes de `InvoiceLine`. Toutes les autres nouvelles fns (`bank_imports`, `bank_transactions`, `products`, etc.) utilisent `{COLUMNS}` via `format!`. Cette incohérence crée un risque de dérive silencieuse : si une colonne est ajoutée à `Invoice` ou `InvoiceLine`, les fns `list_all_*` seront les seules à ne pas bénéficier de la mise à jour de la constante.

Note : les colonnes hard-codées sont aujourd'hui exactes (vérifiées contre les structs). C'est un risque de maintenance, pas un bug actuel.

**Fix** : pour `list_all_by_company`, créer `const INVOICE_COLUMNS` ou réutiliser `FIND_INVOICE_SCOPED_SQL` (adapter). Pour `list_all_lines_by_company`, utiliser `LINE_COLUMNS` avec alias de table (ex. `LINE_COLUMNS.split(", ").map(|c| format!("il.{c}")).join(", ")`), ou ajouter une constante `LINE_COLUMNS_ALIASED`.

---

## F-C2-07 — MEDIUM — `&format!(...)` inutile dans les fns qui n'interpolent qu'une constante compile-time (`bank_imports`, `bank_transactions`, `products`, `reconciliation_rules`, `bank_profiles`, `journal_entries`)
**Fichiers** : tous les repos concernés

Exemple dans `bank_imports.rs:204-207` :
```rust
sqlx::query_as::<_, BankImport>(&format!(
    "SELECT {COLUMNS} FROM bank_imports WHERE company_id = ? ORDER BY id"
))
```

`COLUMNS` est une `const &str` — l'interpolation `{COLUMNS}` dans `format!` est résolue à la compilation, mais la chaîne résultante est **allouée sur le heap** à chaque appel (allocation `String` → `.as_str()` → drop). L'équivalent `sqlx::query_as::<_, BankImport>(concat!("SELECT ", COLUMNS, " FROM ..."))` ou simplement un `const SQL` pré-calculé éviterait cette allocation. `journal_entries.rs` et `bank_transactions.rs` font de même. Ce pattern est répété **6 fois** dans les nouvelles fns.

**Fix** : remplacer `&format!("SELECT {COLUMNS} ...")` par `const SQL: &str = concat!("SELECT ", COLUMNS, " FROM ...");` (si COLUMNS est `&str` statique — ce qui est le cas ici).

---

## F-C2-08 — LOW — Clé i18n `export-global-souverainete-note` contient un mot français dans son identifiant
**Fichiers** : tous les `messages.ftl`

La convention des clés i18n dans ce projet est exclusivement anglaise (`export-global-title`, `export-global-description`, etc.). La clé `export-global-souverainete-note` utilise le mot français **souverainete** (sans accent, non-ASCII-safe mais acceptable en TOML/FTL). Une convention cohérente serait `export-global-sovereignty-note` ou `export-global-data-ownership-note`.

Impact faible — les runtimes FTL ne se soucient pas du sens de la clé — mais cela compliquera les recherches dans les outils de traduction et introduira une exception dans les conventions.

---

## F-C2-09 — LOW — `export-global-description` ne mentionne pas les "produits" dans son résumé court alors que `content-includes` les liste
**Fichiers** : tous les `messages.ftl:export-global-description`

`export-global-description` liste : `(comptes, écritures, contacts, factures, transactions bancaires)`. Ce résumé bref est affiché dans l'UI avant que l'utilisateur ne lise `content-includes`. Les produits sont exportés mais absents du résumé. Incohérence cosmétique entre le résumé et la liste détaillée.

---

## F-C2-10 — LOW — Aucune `LIMIT` de sécurité sur les fns `list_all_by_company` : OOM non gouverné
**Fichiers** : tous les nouveaux repos

Les 10 fns `list_all_by_company` / `list_all_lines_by_company` récupèrent **toutes les lignes** en mémoire via `fetch_all`. C'est intentionnel (export souveraineté). Cependant, aucun commentaire ne documente la borne supérieure attendue ni la décision de ne pas paginer. Sur une instance avec 500K transactions bancaires (usage intensif sur 10 ans), l'allocation mémoire au niveau du handler sera non bornée.

Ce finding est classé LOW car la story documente explicitement l'approche "full export" et la contrainte OR 10 ans. C'est une dette de performance documentée acceptable en v0.1. Mais l'absence de commentaire `// PERF: unbounded — acceptable jusqu'à X rows (§borne-perf-spec)` laisse les futurs développeurs sans signal.

---

## Récapitulatif

| # | Sévérité | Titre résumé |
|---|----------|--------------|
| F-C2-01 | HIGH | Doc comment `find_active_for_company` fusionné avec `list_all_by_company` |
| F-C2-02 | HIGH | `ORDER BY entry_date ASC` vs index `DESC` → filesort sur export journal |
| F-C2-03 | MEDIUM | `zopfli` tiré par `zip/deflate` malgré `default-features = false` |
| F-C2-04 | MEDIUM | Test scoping `invoice_lines` ne prouve pas l'isolation (0 lignes insérées) |
| F-C2-05 | MEDIUM | `content-includes` i18n omet `vat_rates`, `bank_imports`, `bank_profiles` |
| F-C2-06 | MEDIUM | Colonnes hard-codées dans `invoices.rs` au lieu de réutiliser `LINE_COLUMNS` |
| F-C2-07 | MEDIUM | `&format!("{COLUMNS}")` alloue un `String` inutile à chaque appel (×6) |
| F-C2-08 | LOW | Clé i18n `souverainete-note` non conforme à la convention d'identifiants anglais |
| F-C2-09 | LOW | `export-global-description` omet les produits (mentionnés dans `content-includes`) |
| F-C2-10 | LOW | Aucune `LIMIT` ni commentaire borne sur les fns `list_all_by_company` |

**Total : 2 HIGH, 5 MEDIUM, 3 LOW. Pas de CRITICAL.**
