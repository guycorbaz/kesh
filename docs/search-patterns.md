# Patterns de recherche Kesh — FULLTEXT vs LIKE

**Story 7-4 / KF-005** (closed 2026-04-30) — Migration de `LIKE '%query%'`
(full table scan O(n)) vers `MATCH AGAINST IN BOOLEAN MODE` (FULLTEXT
InnoDB) pour les colonnes texte longues. Document de référence pour les
contributions futures touchant à la recherche.

## 1. Quand utiliser FULLTEXT vs LIKE

Règle pragmatique :

| Type de colonne | Stratégie | Justification |
|---|---|---|
| Texte long user-generated `VARCHAR(255+)` (noms métier, descriptions, libellés d'écriture, etc.) | **FULLTEXT** `MATCH AGAINST IN BOOLEAN MODE` | Index inversé, sub-100ms même sur > 50k lignes. |
| Texte structuré court (numéro de facture `INV-YYYY-NNNNN`, IBAN, IDE, username, etc.) | **LIKE** `... ESCAPE '\\'` | FULLTEXT tokenize sur ponctuation → casse les fragments structurés. Volume modéré → full scan acceptable v0.1. |
| Email (format `user@domain.tld`) | **LIKE** | Le `@` et le `.` sont des séparateurs FULLTEXT — un fragment `@gmail` ne matcherait jamais. |

## 2. Liste des 4 index FULLTEXT créés Story 7-4

| Table | Colonne | Index name | Repository qui l'utilise |
|---|---|---|---|
| `contacts` | `name` | `ft_contacts_name` | `contacts::list_by_company_paginated` (et indirectement `invoices::list_by_company_paginated` + `invoices::due_dates_summary` via JOIN sur `c.name`). |
| `products` | `name` | `ft_products_name` | `products::list_by_company_paginated`. |
| `products` | `description` | `ft_products_description` | `products::list_by_company_paginated` (composé en OR avec `name`). |
| `journal_entries` | `description` | `ft_journal_entries_description` | `journal_entries::list_by_company_paginated`. |

Migration SQL : `crates/kesh-db/migrations/20260430000001_kf005_fulltext_indexes.sql`.

## 3. Limitations BOOLEAN MODE à connaître

1. **Min token size = 3 caractères** (`innodb_ft_min_token_size` défaut).
   Recherche `« CH »`, `« le »`, `« de »` → **0 match** (tokens trop courts
   ignorés à l'indexation). Documenter dans les placeholders UI quand
   pertinent.
2. **Tokenization sur whitespace + ponctuation**. `« INV-2026-00042 »`
   tokenize en `INV`, `2026`, `00042` → recherche `« INV-2026 »` ne donne
   pas le résultat attendu. **C'est pourquoi `invoice_number` reste en
   LIKE**.
3. **Pas de suffix wildcard ni de mid-word match**. `MATCH AGAINST 'mar*'`
   matche les mots COMMENÇANT par `mar` ; `*mar` ou `*mar*` ne sont PAS
   supportés en MariaDB InnoDB FULLTEXT. **Régression UX v0.1 acceptée** :
   `« argo »` ne trouve plus `« Camargo »` ; documentée dans 3 tests
   régression `test_search_no_longer_matches_mid_word` (contacts,
   products, journal_entries).
4. **Sémantique multi-mots** : `"foo bar*"` est interprété comme « les
   mots sans préfixe `+`/`-` sont **optionnels avec ranking de
   pertinence** » (cf. MySQL docs § 14.9.2). Fonctionnellement = OR
   inclusif. Si un AND strict devient nécessaire en v0.2+, splitter par
   whitespace et appender `+ ... +` à chaque token.

## 4. Helper `escape_boolean_ft`

Module : `crates/kesh-db/src/util/search.rs`.

Signature :

```rust
pub fn escape_boolean_ft(input: &str) -> String;
```

**Stratégie : strip TOTAL** (pas escape). Le backslash-escaping
(`\+`, `\-`, etc.) en BOOLEAN MODE n'est pas garanti déterministe selon
la version MariaDB exacte. Le strip total donne un comportement
prévisible sur toutes versions 11.x.

**10 caractères opérateurs supprimés** : `+ - > < ( ) ~ * " \`

**Caractères PRÉSERVÉS** :

- `@` (non-opérateur BOOLEAN MODE — utile pour fragments d'email).
- `%` `_` (silencieusement ignorés par le tokenizer InnoDB FULLTEXT).
- `$ ^ [ ] | .` (caractères regex — non-opérateurs FULLTEXT).
- Accents UTF-8 (`utf8mb4_unicode_ci` tokenize correctement).

**Pattern d'usage côté repository** :

```rust
use crate::util::search::escape_boolean_ft;

if let Some(raw) = query.search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
    let escaped = escape_boolean_ft(raw);
    if !escaped.is_empty() {
        let bool_query = format!("{escaped}*"); // prefix wildcard auto-append
        qb.push(" AND MATCH(name) AGAINST(")
            .push_bind(bool_query)
            .push(" IN BOOLEAN MODE)");
    }
}
```

Le helper `escape_like` (échappement `LIKE`) vit dans le même module et
est utilisé pour les colonnes restées en LIKE (`email`, `invoice_number`,
`payment_terms`).

## 5. Pourquoi `BOOLEAN MODE` et pas `NATURAL LANGUAGE MODE`

| Aspect | NATURAL LANGUAGE | BOOLEAN |
|---|---|---|
| Ranking par pertinence (BM25-like) | ✅ | ❌ |
| Substring/prefix match | ❌ word match exact | ✅ avec wildcard `Mar*` |
| Stop-words filtrés | ✅ (souvent surprise UX FR) | ❌ tous mots indexés |
| Min word length | 3 chars | 3 chars |

**Décision Story 7-4** : `BOOLEAN MODE` + auto-append `*` côté repository
préserve la sémantique prefix-search pré-existante (UX inchangée pour le
95% des cas usage), évite les surprises stop-words (la liste par défaut
de `utf8mb4_unicode_ci` est anglaise), et conserve le tri custom
(`name ASC`, `entry_date DESC`) intact puisqu'on ne dépend pas du
ranking.

## 6. Pattern hybride MATCH OR LIKE — décision v0.1

Trois callsites combinent `MATCH AGAINST` (sur la colonne FULLTEXT) avec
`LIKE` (sur des colonnes structurées préservées) en composition `OR` :

- `contacts::list_by_company_paginated` : `MATCH(name) OR email LIKE OR client_number LIKE` *(Story 16-3b, #151 — `client_number` rejoint `email` en `LIKE` et non l'index FULLTEXT : ses séparateurs, `CLI-2026-00042`, cassent les tokens exactement comme le `@` d'un email. ⚠️ La disjonction est présente dans les **deux** branches du callsite — celle du terme échappé vide et celle du cas courant ; n'en traiter qu'une fait cesser silencieusement la recherche par numéro quand le terme n'est fait que d'opérateurs FULLTEXT.)*
- `invoices::list_by_company_paginated` : `MATCH(c.name) OR invoice_number LIKE OR payment_terms LIKE`
- `invoices::due_dates_summary` : idem invoices ci-dessus

**Observation EXPLAIN sur dataset 100 lignes** (cf. test `t7_4a_explain_hybrid_match_or_like_contacts`) :
l'optimizer MariaDB ne place PAS l'index FULLTEXT dans `possible_keys`
de la query hybride et fait un table scan systématique. C'est une
limitation connue (pas d'`index_merge` MariaDB pour FULLTEXT + BTREE
sur disjonction).

**Décision v0.1 (Story 7-4)** : accepter le full scan sur les callsites
hybrides — volumes prod < 10k contacts/company restent sub-secondaires.
Les callsites mono-colonne (`products`, `journal_entries`,
`contacts.name` seul si l'utilisateur ne tape rien dans email) bénéficient
pleinement de l'index FULLTEXT.

**v0.2 (à évaluer)** : si la dette devient observable (latence sur
dataset prod), refactor vers une `UNION` de 2-3 SELECT distincts (1
MATCH, 1-2 LIKE). Pas pré-emptif — wait for evidence.

## 7. Procédure de récupération échec migration

`SQLx` n'enveloppe PAS les migrations MySQL/MariaDB dans une transaction
(DDL = auto-commit côté serveur). Si la migration `20260430000001_kf005_*`
échoue mid-parcours (ex. disk full sur la 3e `ALTER`), les statements
précédents restent persistés.

Procédure de récupération opérateur :

```sql
-- 1. Diagnostiquer : lister les index FULLTEXT déjà créés
SHOW INDEX FROM contacts WHERE Index_type = 'FULLTEXT';
SHOW INDEX FROM products WHERE Index_type = 'FULLTEXT';
SHOW INDEX FROM journal_entries WHERE Index_type = 'FULLTEXT';

-- 2. Drop ceux qui existent déjà (selon le résultat de l'étape 1).
-- Exemple si seuls ft_contacts_name et ft_products_name existent :
ALTER TABLE contacts DROP INDEX ft_contacts_name;
ALTER TABLE products DROP INDEX ft_products_name;
-- (un DROP qui échoue avec erreur 1091 « can't DROP, check that key
-- exists » est OK — la migration n'avait simplement pas atteint cet
-- index avant l'échec)

-- 3. Vérifier que la table _sqlx_migrations ne contient PAS la
-- migration partielle (sinon SQLx croira qu'elle est déjà appliquée
-- et la skippera silencieusement, laissant la DB inconsistante)
DELETE FROM _sqlx_migrations WHERE version = 20260430000001;

-- 4. Relancer la migration via cargo sqlx migrate run (ou MIGRATOR au
-- boot app)
```

**Important** : ne JAMAIS re-run le fichier de migration directement
(via `mariadb < migration.sql`) sans nettoyer `_sqlx_migrations` d'abord.

## 8. Limitations futures à évaluer (hors scope v0.1)

- **Recherche multi-langue avec stemming** (lemmatisation FR/DE/IT) :
  non supporté nativement par MariaDB FULLTEXT. À évaluer Sphinx/Manticore
  en v0.3+ si besoin émerge.
- **Search dans `invoice_lines.description`** : feature, pas perf fix.
  Story dédiée v0.2 si demande utilisateur (« retrouver toutes les
  factures où j'ai facturé `consultation` »).
- **Mid-word search restauré** : nécessite `innodb_ft_min_token_size=1`
  (impact perf global) ou bascule Sphinx/Manticore. Tests régression
  `test_search_no_longer_matches_mid_word` détecteront automatiquement
  un changement de comportement.
- **Search ranking BM25 / TF-IDF** : passage en `NATURAL LANGUAGE MODE`,
  abandon du tri custom. À ne faire que si UX justifie.
