# Story 7.4 : KF-005 — Index FULLTEXT pour la recherche

Status: ready-for-dev

<!-- Note: Validation est optionnelle. Lancer `bmad-create-story validate` pour une revue qualité multi-passes avant `dev-story`. -->

## Story

As a **mainteneur de Kesh**,
I want **remplacer le pattern `LIKE '%query%'` (full table scan) par un index FULLTEXT MariaDB + `MATCH () AGAINST () IN BOOLEAN MODE` sur les colonnes texte longues recherchées dans `contacts`, `products`, `journal_entries` et `invoices` (via JOIN sur `contacts.name`)**,
so that **la performance des requêtes de recherche reste acceptable au-delà de ~50k lignes par table, que la dette KF-005 (issue [#5](https://github.com/guycorbaz/kesh/issues/5)) soit fermée avant l'Epic 8 (Import bancaire) qui injectera mécaniquement de gros volumes (10×+ lignes de factures, écritures journalières), et que les utilisateurs avancés Kesh ne soient pas confrontés à des recherches de plusieurs secondes au quotidien**.

### Contexte

**Story 7-4 = closure de KF-005 (issue [#5](https://github.com/guycorbaz/kesh/issues/5))** dans l'Epic 7 (Tech Debt Closure, inséré 2026-04-20 par décision rétro Epic 6).

**Aujourd'hui, la recherche full-text dans 4 repositories (`contacts.rs`, `products.rs`, `journal_entries.rs`, `invoices.rs`) utilise `LIKE '%query%'` avec `ESCAPE '\\'`** (cf. `crates/kesh-db/src/repositories/contacts.rs:167-177`, `products.rs:125-135`, `journal_entries.rs:374-379`, `invoices.rs:252-264`). Reproduction MariaDB :

```sql
EXPLAIN SELECT * FROM contacts WHERE name LIKE '%foo%';
-- type: ALL  (full table scan, pas d'index utilisé)
```

**Performance** :
- Acceptable jusqu'à ~10k lignes (sub-100ms)
- Dégradation linéaire au-delà (O(n) full scan)
- **Critique à partir de ~50k contacts/produits** ou pour `journal_entries` qui croît mécaniquement (1 entrée par opération comptable, ~10×/jour PME ⇒ 30k entrées/an)
- Issue KF-005 cite explicitement `invoice_lines.description` (potentiellement 10× plus de lignes que de factures) — **mais cette colonne n'est PAS recherchée v0.1** (cf. §scope hors story)

**Pourquoi maintenant et pas v0.2** ? **Epic 8 (Import bancaire) injectera massivement** des transactions bancaires → écritures comptables → augmentation rapide du volume `journal_entries`. Si KF-005 reste ouvert, le bug se manifestera dès les premiers imports CAMT.053 réels (typiquement 100-500 transactions/mois × 12 mois × 5 clients beta = 6k-30k entrées en quelques semaines de prod beta). Décision Guy 2026-04-20 (rétro Epic 6) : fermer KF-005 maintenant pour stabiliser le pattern de recherche avant Epic 8.

**Status sprint** : `epic-7: in-progress` (déjà), `7-4-kf-005-fulltext-search-index: backlog → ready-for-dev` à la fin de cette spec.

### Scope verrouillé — colonnes à indexer FULLTEXT

KF-005 issue #5 cite 5 colonnes : `contacts.name, contacts.address, products.name, products.description, invoice_lines.description`. La cartographie du code (cf. §références) montre une réalité différente :

| Repository | Colonnes actuellement recherchées (LIKE) | Décision Story 7-4 |
|---|---|---|
| `contacts.rs:171-175` | `name`, `email` | FULLTEXT sur `name` ; `email` reste LIKE |
| `products.rs:129-133` | `name`, `description` | FULLTEXT sur `name`, `description` |
| `journal_entries.rs:374` | `description` | FULLTEXT sur `description` |
| `invoices.rs:256-261` | `i.invoice_number`, `i.payment_terms`, `c.name` (JOIN contacts) | `c.name` bénéficie de l'index `contacts.name` ; `invoice_number` et `payment_terms` restent LIKE |

**Justifications de la liste finale (4 colonnes FULLTEXT)** :

1. **`contacts.name`** ✅ — long texte (VARCHAR(255)), recherche par nom métier ; bénéficie aussi à la recherche `invoices` (via JOIN).
2. **`products.name`** ✅ — long texte (VARCHAR(255)), volume potentiel élevé en prod (catalogue articles).
3. **`products.description`** ✅ — long texte (VARCHAR(1000)), tokenizable.
4. **`journal_entries.description`** ✅ — long texte (VARCHAR(500)), volume mécaniquement croissant (FR69 du PRD : « rechercher des écritures par libellé »).

**Colonnes exclues v0.1 (justification ligne par ligne)** :

- **`contacts.email`** — format structuré (`user@domain.tld`), substring `@gmail` typique. FULLTEXT tokenize sur `@` et `.` → `@gmail` ne match pas. **Garder LIKE**.
- **`contacts.address`** (KF-005 cite) — pas dans le code search v0.1 (cf. `contacts.rs:171-175` ne teste que `name OR email`). Hors scope tant que pas recherché.
- **`invoices.invoice_number`** — format structuré court (ex. `INV-2026-00042`, 13-20 chars), volume modéré (~10k/an PME), recherche typique par préfixe ou exact match. **LIKE acceptable v0.1**.
- **`invoices.payment_terms`** — VARCHAR(255) optionnel, faible cardinalité (souvent répété : « 30 jours net »), **LIKE acceptable**.
- **`invoice_lines.description`** (KF-005 cite) — actuellement **pas recherché** dans le code (`invoices.rs` search ne joint pas `invoice_lines`). Étendre la search à `invoice_lines` serait une feature, pas une perf fix. Hors scope KF-005 v0.1.
- **`contacts.ide_number`**, **`bank_accounts.iban/qr_iban`**, **`users.username`** — exact-match colonnes (UNIQUE constraint), pas de LIKE actuel. Pas concernés.

**Total** : **4 index FULLTEXT** à créer en migration unique.

### Scope volontairement HORS story — décisions tranchées

**Hors scope avec justification** (rejeter explicitement les sirènes) :

1. **Trait/abstraction `SearchableRepository<T>`** — YAGNI v0.1. 4 callsites avec syntaxe SQL différente (JOINs invoices, simple WHERE contacts/products/journal_entries). Coût d'abstraction > coût de duplication.
2. **Migration online avec `pt-online-schema-change`** — pas pertinent v0.1 : Kesh est mono-tenant local par déploiement, fenêtre de maintenance acceptée. Algorithme `INPLACE` MariaDB 10.11 suffit pour les volumes v0.1 (cf. §migration online).
3. **Search ranking par pertinence** (BM25/TF-IDF) — `BOOLEAN MODE` ne ranke pas. UX inchangée v0.1 (tri par `name ASC` ou `entry_date DESC` préservé). `NATURAL LANGUAGE MODE` (avec ranking) est rejeté car il perd la sémantique prefix-search (cf. §mode FULLTEXT).
4. **Recherche multi-langue avec stemming** (lemmatisation FR/DE/IT) — non supporté nativement par MariaDB FULLTEXT. Hors scope v0.1, peut-être Sphinx/Manticore en v0.3+ si besoin user émerge.
5. **Search dans `invoice_lines.description`** — cf. ci-dessus, c'est une feature, pas un perf fix. Story dédiée v0.2 si demande utilisateur émerge (ex. « retrouver toutes les factures où j'ai facturé `consultation` »).
6. **Migration ALTER TABLE des colonnes en TEXT** — les colonnes actuelles sont VARCHAR (255-1000). FULLTEXT supporte VARCHAR. Pas besoin de passer en TEXT.
7. **Configuration `innodb_ft_min_token_size`** — défaut 3 caractères acceptable. Documenter la limitation (cf. §UX impact tokens courts) plutôt que modifier le param système (impact global, non-trivial à déployer).

### §mode FULLTEXT — choix BOOLEAN MODE avec prefix wildcard auto-append

**Comparaison NATURAL vs BOOLEAN** :

| Aspect | NATURAL LANGUAGE | BOOLEAN | Décision |
|---|---|---|---|
| Ranking par pertinence | ✅ (BM25-like) | ❌ | Indifférent v0.1 (tri custom préservé) |
| Substring match (`Mar` → `Marie`) | ❌ word match exact | ✅ avec wildcard `Mar*` | **BOOLEAN gagne** |
| Stop-words filtrés | ✅ (« le », « de » exclus) | ❌ tous mots indexés | **BOOLEAN gagne** (pas de surprise UX) |
| Opérateurs avancés | ❌ | ✅ (`+`, `-`, `*`, `"..."`) | **BOOLEAN gagne** |
| Min word length | 3 chars (`ft_min_word_len`) | 3 chars (`innodb_ft_min_token_size`) | Égalité |

**Décision** : **BOOLEAN MODE** + auto-append du wildcard `*` côté repository pour préserver l'UX prefix-search actuelle.

**Pattern Rust** (à appliquer dans les 4 repositories) :

```rust
// AVANT
let pattern = format!("%{}%", escape_like(trimmed));
qb.push(" AND (name LIKE ").push_bind(pattern.clone()).push(" ESCAPE '\\\\' ");

// APRÈS
let term = escape_boolean_ft(trimmed); // échappe +, -, ", *, etc.
let bool_query = format!("{}*", term); // append wildcard pour préserver prefix UX
qb.push(" AND MATCH(name) AGAINST(").push_bind(bool_query).push(" IN BOOLEAN MODE)");
```

**Helper `escape_boolean_ft(input: &str) -> String`** (à créer, co-localisé ou dans `kesh-db/src/util.rs`) :

- Liste des caractères à échapper en BOOLEAN MODE : `+ - > < ( ) ~ * " @ \`
- Stratégie : préfixer chaque caractère spécial par `\` ou supprimer (selon ce qui fait sens UX). Recommandation : supprimer `*` et `"` du payload utilisateur (les opérateurs sont contrôlés côté repo) ; échapper le reste.
- Edge cases : payload vide après échappement → bypass FULLTEXT (la query devient inopérante avec un terme vide). Le repo doit faire `if term.is_empty() { skip search clause }`.

### §UX impact — tokens courts, stop-words, breaking changes

**Limites BOOLEAN MODE à documenter dans le doc-comment du helper** :

1. **Min token size = 3 caractères** (`innodb_ft_min_token_size` défaut)
   - Recherche `« CH »`, `« le »`, `« de »` → **0 match** (tokens trop courts ignorés à l'indexation)
   - Mitigation v0.1 : Documenter dans l'aide UI / placeholder input (« Recherchez par mot d'au moins 3 lettres »).
   - Mitigation v0.2+ : Configurer `innodb_ft_min_token_size=1` (impact perf global), ou tomber back sur LIKE quand `term.len() < 3` (hybride fallback).
   - **Décision v0.1** : limitation acceptée, pas de fallback automatique. Documenter.

2. **Tokenization sur whitespace + ponctuation**
   - `"INV-2026-00042"` → tokens `« INV »`, `« 2026 »`, `« 00042 »` (3 tokens distincts)
   - Recherche `« INV-2026 »` → **0 match** (tiret = séparateur de tokens, recherche `« INV-2026 »` cherche les 2 tokens `INV` ET `2026` ensemble en BOOLEAN AND, mais avec `*` wildcard le comportement diffère)
   - **Mitigation** : `invoice_number` reste en LIKE (cf. §scope) — n'utilise pas FULLTEXT.

3. **Substring `info` → `infox`** (ex. utilisateur tape « info » pour trouver « infomercial »)
   - LIKE `'%info%'` matche les 2.
   - BOOLEAN `'info*'` matche `infox`, `infomercial`, `infos` (tout ce qui commence par `info`).
   - BOOLEAN `'*info'` ❌ pas supporté MariaDB (suffix wildcard non disponible).
   - **Conséquence** : recherche par milieu de mot perdue (ex. user cherche `« mar »` pour trouver `« Camargo »`). Acceptable v0.1 (rare en contexte comptable suisse, prefix search dominant).

4. **Préservation des tests existants**
   - 3 tests search existent : `contacts.rs:1017-1101` (×2), `products.rs:802-838`. Tous utilisent prefix de mots (`Beta` matchant `TestContact Beta`, `Alpha` matchant `TestProduct Alpha`).
   - Avec BOOLEAN + wildcard auto-append : `Beta*` matche `Beta` et `BetaX` ; le test passe.
   - Test `test_filter_escape_like_wildcard` (`contacts.rs:1064-1101`) cherche `« 100% »` — en BOOLEAN MODE, `%` est tokenizé comme séparateur. Le test devra être adapté ou supprimé (cf. T9).

### §migration online — algorithme INPLACE MariaDB 10.11

**Risque** : `CREATE FULLTEXT INDEX` sur InnoDB peut locker la table en écriture pendant la création.

**Mitigation MariaDB 10.11** : utiliser `ALGORITHM=INPLACE, LOCK=NONE` quand possible :

```sql
ALTER TABLE contacts ADD FULLTEXT INDEX ft_contacts_name (name)
ALGORITHM=INPLACE, LOCK=NONE;
```

**Limites documentées** :
- `LOCK=NONE` peut être refusé par MariaDB selon la version exacte / contention. Si refus → `LOCK=SHARED` (lit OK, écriture bloquée).
- Sur volumes v0.1 (< 50k lignes par table), même `LOCK=DEFAULT` se complète en quelques secondes — fenêtre de maintenance acceptable pour un déploiement self-hosted PME.

**Recommandation pour la migration SQL** :

```sql
-- Ajout des index FULLTEXT KF-005
SET innodb_lock_wait_timeout = 120; -- 2 min, sécurité

ALTER TABLE contacts
ADD FULLTEXT INDEX ft_contacts_name (name),
ALGORITHM=INPLACE, LOCK=NONE;

ALTER TABLE products
ADD FULLTEXT INDEX ft_products_name (name),
ADD FULLTEXT INDEX ft_products_description (description),
ALGORITHM=INPLACE, LOCK=NONE;

ALTER TABLE journal_entries
ADD FULLTEXT INDEX ft_journal_entries_description (description),
ALGORITHM=INPLACE, LOCK=NONE;
```

**Note SQLx migrations** : SQLx applique les migrations en transaction par défaut sur MySQL/MariaDB. Mais `ALTER TABLE ... ADD FULLTEXT` ne peut pas être dans une transaction (DDL implicite commit). Vérifier le comportement SQLx — si problème, splitter la migration en 4 fichiers séparés ou utiliser `-- migrate:next` directives.

### §multi-tenant scoping — préservation `WHERE company_id = ?`

**Vérification critique** (Story 7-1 KF-002 closure 2026-04-29 a hardener le multi-tenant scoping codebase-wide) :

Toutes les queries search incluent déjà `WHERE company_id = ?` AVANT le filtre LIKE. La migration vers MATCH AGAINST DOIT préserver cette composition AND :

```sql
-- AVANT
WHERE company_id = ? AND (name LIKE ? OR email LIKE ?)

-- APRÈS
WHERE company_id = ? AND (MATCH(name) AGAINST(? IN BOOLEAN MODE) OR email LIKE ?)
```

**Conséquence performance** : MariaDB choisit l'index `idx_contacts_company_active (company_id, active)` (BTREE) ou `ft_contacts_name` (FULLTEXT) selon le selectivity. Le **plan optimal** dépend du volume :
- Si la company a 100 contacts sur 50k DB-wide → BTREE (`company_id`) puis filter MATCH en mémoire : OK.
- Si la company a 10k contacts sur 50k → FULLTEXT puis filter `company_id` : OK.

**Test à ajouter** : `EXPLAIN SELECT ...` dans un test sqlx pour vérifier que l'un des deux index est utilisé (cf. T7).

### §audit log et autres patterns existants

**Audit log** : la recherche est READ-only, pas d'audit. Inchangé.

**Verrouillage optimiste** (Story 1-8, étendu Story 7-3 KF-004) : non concerné (search ne mute pas).

**Pagination + tri whitelist** (`*SortBy::as_sql_column()`) : préservé tel quel. Le `MATCH AGAINST` remplace uniquement le filtre WHERE, pas l'ORDER BY.

## Acceptance Criteria

1. **Migration SQL applicable** — Given une DB MariaDB 10.11 fraîchement migrée jusqu'à `20260429*` (avant Story 7-4), When la nouvelle migration `2026MMDD000001_kf005_fulltext_indexes.sql` est appliquée, Then 4 index FULLTEXT sont créés (`ft_contacts_name`, `ft_products_name`, `ft_products_description`, `ft_journal_entries_description`) sans erreur ; idempotent en re-run du test (sqlx::test recreate DB).

2. **Helper `escape_boolean_ft` créé et testé** — Given une string utilisateur arbitraire (incluant `+`, `-`, `*`, `"`, `(`, `)`, `~`, `<`, `>`, `@`, `\`), When `escape_boolean_ft(input)` est appelé, Then la string retournée est safe pour insertion dans `MATCH AGAINST (... IN BOOLEAN MODE)` (caractères spéciaux contrôlés). Tests unitaires couvrent : caractères spéciaux × 11, payload vide, payload entier accent (`é, è, ê`), payload tokens courts (`< 3 chars`).

3. **`contacts::list_by_company_paginated` utilise FULLTEXT pour `name`** — Given un company_id et `search: Some("Mar")`, When la query est exécutée, Then la clause SQL contient `MATCH(name) AGAINST('Mar*' IN BOOLEAN MODE)` (ou équivalent escaped) ET `email LIKE '%Mar%' ESCAPE '\\\\'` (la branche email reste en LIKE), composées en OR. Le test `test_filter_by_search_name` existant doit toujours passer.

4. **`products::list_by_company_paginated` utilise FULLTEXT pour `name` et `description`** — Idem AC #3 mais sur 2 colonnes FULLTEXT. Test `test_filter_by_search` existant doit passer.

5. **`journal_entries::list_by_company_paginated` utilise FULLTEXT pour `description`** — Idem. Aucun test search préexistant ; un nouveau test doit être ajouté (cf. T7.3).

6. **`invoices::list_by_company_paginated` bénéficie indirectement de `ft_contacts_name`** — Given le pattern actuel `c.name LIKE ? ESCAPE`, When la query est ré-implémentée, Then la clause `c.name` utilise `MATCH(c.name) AGAINST(? IN BOOLEAN MODE)` (l'index FULLTEXT sur `contacts.name` est partagé entre les requêtes contacts directes et invoices via JOIN). Les clauses `invoice_number` et `payment_terms` restent en LIKE.

7. **Multi-tenant scoping préservé** — Pour chaque repo modifié, le test `find/list` avec un terme de recherche qui matcherait dans une AUTRE company doit retourner 0 résultats (la clause `WHERE company_id = ?` filtre avant ou avec le MATCH). Ajouter un test `test_search_does_not_leak_cross_company` par repo (4 tests).

8. **EXPLAIN confirme l'utilisation de l'index** — Given une recherche `MATCH AGAINST` sur une table peuplée avec ≥ 10 lignes (suffisant pour que MariaDB ne choisisse pas full-scan), When `EXPLAIN SELECT ... MATCH AGAINST ...` est exécuté, Then l'output contient `type: fulltext` ou `key: ft_<table>_<col>`. Test sqlx ajouté par table (4 tests EXPLAIN).

9. **Tests existants `test_filter_by_search_*` passent sans modification** ou avec adaptation documentée — Given les tests `contacts.rs:1017-1101`, `products.rs:802-838`, When le code repository est ré-implémenté, Then les tests passent (le pattern de matching `Beta` → `TestContact Beta` reste valide en BOOLEAN+wildcard). Le test `test_filter_escape_like_wildcard` (cherche `« 100% »`) doit être ADAPTÉ (en BOOLEAN MODE, `%` est tokenizé comme séparateur — soit le test cherche `« 100 »` directement, soit il vérifie que `%` est correctement échappé/strippé par `escape_boolean_ft`).

10. **Documentation pattern mise à jour** — `docs/optimistic-locking-patterns.md` n'est PAS la bonne place ; créer `docs/search-patterns.md` avec : (i) liste des 4 index FULLTEXT créés et leurs colonnes, (ii) quand utiliser FULLTEXT vs LIKE (règle : VARCHAR(255+) longs textes user-generated → FULLTEXT ; structured short → LIKE), (iii) limitations BOOLEAN MODE (tokens ≥ 3 chars, prefix wildcard auto-append, pas de suffix wildcard), (iv) exemple d'utilisation du helper `escape_boolean_ft`.

11. **`docs/known-failures.md` archive mise à jour** — Status KF-005 `open` → `closed (Story 7-4, YYYY-MM-DD)`. Issue GitHub #5 fermée par commit final via `closes #5`.

12. **Régression suite verte** — `cargo test -p kesh-db --tests -- --test-threads=1` ✅ green (218+ tests, +2-4 nouveaux tests EXPLAIN + cross-company search isolation). `cargo test -p kesh-api` ✅ green (194 E2E tests). `cargo clippy --all-targets -- -D warnings` ✅ green. `cargo fmt --all --check` ✅ clean.

13. **Performance vérifiée** (acceptance critère informatif, pas test automatisé v0.1) — Sur une table contacts seedée à 50k lignes (script de seed-perf séparé OU stress test manuel), la recherche `LIKE '%Mar%'` en baseline mesure ~500ms+ ; après KF-005, `MATCH AGAINST 'Mar*'` doit être < 50ms (10× speedup attendu sur cette taille). **Pas de test auto** — exécution manuelle documentée dans le Change Log story par le dev.

14. **README — Feuille de route et section Fonctionnalités inchangées** — Given la story est de la dette technique pure (pas de feature user-visible nouvelle, pas de release), When une vérification du README post-merge, Then aucune entrée à modifier dans la « Feuille de route » ni dans « Fonctionnalités » (cf. CLAUDE.md règle Sync README — la story n'introduit ni epic done ni feature livrée listée).

## Tasks / Subtasks

### T1 — Helper `escape_boolean_ft` (AC: #2)

- [ ] T1.1 Créer le fichier `crates/kesh-db/src/util.rs` (s'il n'existe pas) ou ajouter à un module existant util/search.
- [ ] T1.2 Implémenter `pub fn escape_boolean_ft(input: &str) -> String` :
  - Strip ou escape les caractères BOOLEAN MODE spéciaux : `+ - > < ( ) ~ * " @ \`
  - Stratégie recommandée : **strip** `* "` (les opérateurs FULLTEXT contrôlés par le repo, pas par user) ; **escape** les autres avec `\` (préserver la string-as-is sans break).
  - Trim whitespace en entrée.
  - Si après stripping, la string est vide → retourner `""` (le caller doit alors skipper le filtre MATCH).
- [ ] T1.3 Tests unitaires (in-module `#[cfg(test)] mod tests`) :
  - `test_escape_strip_wildcard_and_quote()` : `"foo*bar"` → `"foobar"` ; `"un \"deux\""` → `"un deux"`.
  - `test_escape_special_chars()` : `"foo+bar"` → `"foo\\+bar"` ; idem pour `-`, `(`, `)`, `~`, `<`, `>`, `@`.
  - `test_escape_accents_preserved()` : `"Crémant"` → `"Crémant"` (accents UTF-8 préservés).
  - `test_escape_empty_after_strip()` : `"***"` → `""`.
  - `test_escape_short_token()` : `"de"` → `"de"` (le helper ne juge pas la longueur, c'est au caller).
- [ ] T1.4 Documenter le helper avec doc-comment `///` listant les caractères traités + exemple d'utilisation.

### T2 — Migration SQL `kf005_fulltext_indexes` (AC: #1, #11)

- [ ] T2.1 Créer le fichier `crates/kesh-db/migrations/2026MMDD000001_kf005_fulltext_indexes.sql` (substituer MMDD par la date du jour de l'implémentation).
- [ ] T2.2 Contenu :
  ```sql
  -- Migration 7-4 / KF-005 : Index FULLTEXT pour recherche performante sur colonnes texte longues.
  -- Remplace LIKE '%query%' (full table scan) par MATCH AGAINST IN BOOLEAN MODE.
  -- Cible 4 colonnes : contacts.name, products.name, products.description, journal_entries.description.

  ALTER TABLE contacts
      ADD FULLTEXT INDEX ft_contacts_name (name),
      ALGORITHM=INPLACE, LOCK=NONE;

  ALTER TABLE products
      ADD FULLTEXT INDEX ft_products_name (name),
      ADD FULLTEXT INDEX ft_products_description (description),
      ALGORITHM=INPLACE, LOCK=NONE;

  ALTER TABLE journal_entries
      ADD FULLTEXT INDEX ft_journal_entries_description (description),
      ALGORITHM=INPLACE, LOCK=NONE;
  ```
- [ ] T2.3 **Tester en local** : `cargo sqlx migrate run` ou run d'un test sqlx (`cargo test -p kesh-db test_create_contact -- --exact` qui force MIGRATOR à run). Vérifier `SHOW INDEX FROM contacts;` dans MariaDB CLI : `Index_type: FULLTEXT` listé.
- [ ] T2.4 **Vérifier comportement transactionnel SQLx** : si la migration échoue avec « DDL cannot be in transaction », splitter en 4 fichiers (1 par ALTER TABLE) ou utiliser une syntaxe SQLx-compatible (`-- noqa: TX`).

### T3 — Refactor `contacts::list_by_company_paginated` (AC: #3, #7)

- [ ] T3.1 Au-dessus de `contacts.rs:167-177`, importer le helper : `use crate::util::search::escape_boolean_ft;` (path à adapter selon T1.1).
- [ ] T3.2 Remplacer le bloc LIKE par :
  ```rust
  if let Some(raw) = query.search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
      let escaped = escape_boolean_ft(raw);
      if !escaped.is_empty() {
          let bool_query = format!("{}*", escaped);
          let email_pattern = format!("%{}%", escape_like(raw));
          qb.push(" AND (MATCH(name) AGAINST(")
              .push_bind(bool_query)
              .push(" IN BOOLEAN MODE) OR email LIKE ")
              .push_bind(email_pattern)
              .push(" ESCAPE '\\\\')");
      }
  }
  ```
- [ ] T3.3 Vérifier que la query COUNT (`contacts.rs:296-336`) est cohérente — si elle utilise aussi le filtre search, appliquer le même refactor.
- [ ] T3.4 Lancer `cargo test -p kesh-db --test contacts_repository -- --test-threads=1` et le test inline `test_filter_by_search_name` (`contacts.rs:1017-1061`). Doit passer.

### T4 — Refactor `products::list_by_company_paginated` (AC: #4)

- [ ] T4.1 Idem T3 mais pour `products.rs:125-135`. Les 2 colonnes FULLTEXT (`name`, `description`) sont composées en OR :
  ```rust
  qb.push(" AND (MATCH(name) AGAINST(")
      .push_bind(bool_query.clone())
      .push(" IN BOOLEAN MODE) OR MATCH(description) AGAINST(")
      .push_bind(bool_query)
      .push(" IN BOOLEAN MODE))");
  ```
  - **Note** : MariaDB FULLTEXT permet aussi un index combiné `FULLTEXT (name, description)` mais NON utilisé ici car on veut compter `name` et `description` séparément (pour ranking futur si on passe en NATURAL).
- [ ] T4.2 Test `test_filter_by_search` (`products.rs:802-838`) doit passer.

### T5 — Refactor `journal_entries::list_by_company_paginated` (AC: #5)

- [ ] T5.1 Idem T3 pour `journal_entries.rs:374-379` (1 colonne FULLTEXT `description`).
- [ ] T5.2 Pas de test search préexistant — créer `test_filter_by_description_search` dans `journal_entries.rs` mod tests : seed 3 entrées avec descriptions distinctes, search `"<premier_mot>"`, vérifier que seul le résultat attendu est retourné.

### T6 — Refactor `invoices::list_by_company_paginated` (AC: #6)

- [ ] T6.1 `invoices.rs:252-264` : la search joint `c.name` (contacts) + `i.invoice_number` + `i.payment_terms`. Refactor : `c.name` passe en MATCH AGAINST ; `i.invoice_number` et `i.payment_terms` restent en LIKE.
  ```rust
  qb.push(" AND (MATCH(c.name) AGAINST(")
      .push_bind(bool_query)
      .push(" IN BOOLEAN MODE) OR COALESCE(i.invoice_number, '') LIKE ")
      .push_bind(like_pattern.clone())
      .push(" ESCAPE '\\\\' OR COALESCE(i.payment_terms, '') LIKE ")
      .push_bind(like_pattern)
      .push(" ESCAPE '\\\\')");
  ```
- [ ] T6.2 Vérifier que la query COUNT correspondante (probablement dans le même fichier autour de `invoices.rs:610-670`) est aussi mise à jour.
- [ ] T6.3 Si pas de test search inline existant, en créer un dans `invoices.rs` mod tests : seed 2 contacts + 2 factures, search par contact name, vérifier filtrage correct.

### T7 — Tests EXPLAIN (AC: #8)

- [ ] T7.1 Créer un test integration `crates/kesh-db/tests/kf005_fulltext_index_e2e.rs` ou ajouter aux test files existants par repo.
- [ ] T7.2 Pour chaque table avec FULLTEXT (`contacts`, `products`, `journal_entries`) :
  - Seed ≥ 10 lignes avec contenu varié.
  - Run `EXPLAIN FORMAT=JSON SELECT ... WHERE company_id = ? AND MATCH(<col>) AGAINST(? IN BOOLEAN MODE)`.
  - Parser le JSON, vérifier que `key` ou `index` contient `ft_<table>_<col>` OU que `access_type: fulltext`.
  - **Note MariaDB** : sur petites tables, l'optimizer peut choisir le FULL SCAN même avec un index FULLTEXT (cost-based). Si le test flake, augmenter le seed à 100+ lignes ou commenter avec `#[ignore]` + doc-link sur l'optimizer behavior.
- [ ] T7.3 Test isolation cross-company (AC #7) : créer 2 companies, seed 1 entrée par company avec le même mot recherché, vérifier que la query scopée à company A ne retourne que les résultats de A.

### T8 — Documentation (AC: #10)

- [ ] T8.1 Créer `docs/search-patterns.md` :
  - Section 1 : « Quand utiliser FULLTEXT vs LIKE » (règle pragmatique : long-text user-generated VARCHAR(255+) → FULLTEXT ; structured/short → LIKE).
  - Section 2 : « Liste des 4 index FULLTEXT créés Story 7-4 » (table avec colonne, index name, repository qui l'utilise).
  - Section 3 : « Limitations BOOLEAN MODE » (min token ≥ 3 chars, prefix wildcard auto-append, pas de suffix wildcard, tokenization sur whitespace + ponctuation, conséquence sur recherches IDE/IBAN/numéros).
  - Section 4 : « Helper `escape_boolean_ft` » (signature, exemple d'utilisation, list des caractères traités).
  - Section 5 : « Pourquoi BOOLEAN et pas NATURAL LANGUAGE » (1 paragraphe : préservation prefix UX, pas de stop-words magiques, pas de besoin ranking v0.1).
- [ ] T8.2 Mettre à jour `docs/known-failures.md` ou son archive (CLAUDE.md indique que `docs/known-failures.md` est archivé depuis 2026-04-18) : ajouter une note de closure si l'archive le permet, sinon documenter la fermeture uniquement via le commit final + l'issue GitHub #5.

### T9 — Adaptation tests existants (AC: #9)

- [ ] T9.1 `contacts.rs::test_filter_by_search_name` (l. 1017) — doit passer sans modif (cherche `"Beta"` matchant `"TestContact Beta"` ; en BOOLEAN+wildcard `Beta*` matche `Beta` exact, OK).
- [ ] T9.2 `contacts.rs::test_filter_escape_like_wildcard` (l. 1064) — cherche `"100%"`. En BOOLEAN MODE, `%` n'est pas un opérateur LIKE — c'est un caractère ordinaire (mais tokenisé comme séparateur si présent en milieu de mot). Adapter le test :
  - Soit : changer le terme cherché en `"100"` et vérifier que le résultat correspond.
  - Soit : vérifier que `escape_boolean_ft("100%")` strip ou échappe correctement le `%`.
  - **Décision** : adapter le test pour vérifier le comportement post-escape via le helper. Renommer en `test_search_handles_special_chars`.
- [ ] T9.3 `products.rs::test_filter_by_search` (l. 802) — doit passer sans modif (`"Alpha"` matchant `"TestProduct Alpha"`).

### T10 — Verification + commit final (AC: #11, #12, #13)

- [ ] T10.1 Lancer `cargo test -p kesh-db --tests -- --test-threads=1` ✅ green.
- [ ] T10.2 Lancer `cargo test -p kesh-api` ✅ green.
- [ ] T10.3 `cargo clippy --all-targets -- -D warnings` ✅ green.
- [ ] T10.4 `cargo fmt --all --check` ✅ clean.
- [ ] T10.5 (Optionnel — informatif) Run perf manuel : seed 10k contacts via script ad-hoc, mesurer search baseline LIKE puis post-FULLTEXT. Documenter le delta dans le Change Log.
- [ ] T10.6 Mettre à jour status story `review` (à la fin de l'impl, avant code-review) et sprint-status.yaml `review`.
- [ ] T10.7 Commit final : `Story 7-4 : KF-005 — FULLTEXT search indexes (closes #5)` ; vérifier le tag `closes #5` pour fermeture auto de l'issue GitHub.

## Dev Notes

### Patterns architecturaux applicables

- **DRY (Don't Repeat Yourself)** — le helper `escape_boolean_ft` est l'occasion de regrouper la logique d'échappement search dans `crates/kesh-db/src/util.rs` (ou équivalent). Si le module n'existe pas, créer la sous-arborescence proprement (`util/mod.rs` + `util/search.rs`). Voir aussi le helper `escape_like` dupliqué 4× dans contacts/products/journal_entries/invoices (`contacts.rs:33-42`, etc.) — **bonus de refactoring** : extraire `escape_like` dans le même module pour fermer la dette de duplication 4× notée dans les commentaires (`invoices.rs:40-41` : « 4e duplication — dette critique notée »). À évaluer avec Guy si on inclut le bonus dans cette story ou si on le sort en story dédiée.

  **Décision (à valider en review)** : embarquer le bonus `escape_like` dans cette story (T1 étendu) car le module `util/search.rs` est créé pour `escape_boolean_ft` de toutes façons. Coût marginal négligeable (4 import statements + suppression de 4 fonctions privées dupliquées). Bénéfice : ferme une dette tech notée dans les commentaires du code.

- **Multi-tenant scoping** (Story 7-1 KF-002, fermée 2026-04-29) — TOUTES les queries search incluent déjà `WHERE company_id = ?`. La migration FULLTEXT préserve cette invariante. T7.3 valide explicitement l'isolation cross-company.

- **Verrouillage optimiste** (Story 7-3 KF-004) — non concerné (search read-only).

- **Audit log** — non concerné (search read-only).

- **Pagination + tri whitelist** (`*SortBy::as_sql_column()`) — préservé tel quel ; le MATCH AGAINST remplace uniquement le filtre WHERE.

### Composants source à toucher

- `crates/kesh-db/src/util.rs` ou `crates/kesh-db/src/util/search.rs` (créer ou étendre) — helpers `escape_boolean_ft` + bonus `escape_like` mutualisé.
- `crates/kesh-db/src/repositories/contacts.rs` (T3) — refactor search clause + import helper.
- `crates/kesh-db/src/repositories/products.rs` (T4) — idem.
- `crates/kesh-db/src/repositories/journal_entries.rs` (T5) — idem.
- `crates/kesh-db/src/repositories/invoices.rs` (T6) — idem (search joint plus complexe).
- `crates/kesh-db/migrations/2026MMDD000001_kf005_fulltext_indexes.sql` (T2) — nouvelle migration.
- `crates/kesh-db/tests/kf005_fulltext_index_e2e.rs` (T7) — tests EXPLAIN + cross-company.
- `docs/search-patterns.md` (T8) — nouvelle doc pattern.
- `docs/known-failures.md` (T8) — archive update KF-005 closed.

### Standards de testing

- Tests sqlx pour core business logic (helper, queries refactorées).
- Tests EXPLAIN pour validation index utilisé.
- Tests cross-company pour validation multi-tenant.
- Pas de test E2E HTTP requis (KF-005 est pur perf, pas de comportement HTTP-observable changeant — les routes `/contacts`, `/products`, etc. retournent toujours 200 + même body shape).
- Pas de test Playwright requis.

### Project Structure Notes

- Conformité avec `crates/kesh-db/` structure existante (repositories par entité, migrations timestampées).
- Variance : création de `util/` ou `util/search.rs` — module nouveau dans `kesh-db`. Vérifier qu'aucun module `util` n'existe déjà sous un autre nom (ex. `helpers`, `common`). À ce jour : pas de module utility centralisé dans `kesh-db/src/`. **Nouvelle structure justifiée**.

### References

- [Source: GitHub issue #5 (KF-005)](https://github.com/guycorbaz/kesh/issues/5) — root cause, scope, reproduction
- [Source: `_bmad-output/planning-artifacts/prd.md:478` (FR69)] — recherche écritures par libellé
- [Source: `crates/kesh-db/src/repositories/contacts.rs:33-42`] — helper `escape_like` à mutualiser
- [Source: `crates/kesh-db/src/repositories/contacts.rs:167-177`] — pattern LIKE actuel à remplacer
- [Source: `crates/kesh-db/src/repositories/products.rs:125-135`] — idem
- [Source: `crates/kesh-db/src/repositories/journal_entries.rs:374-379`] — idem (1 colonne)
- [Source: `crates/kesh-db/src/repositories/invoices.rs:252-264`] — idem (search joint, 3 colonnes)
- [Source: `crates/kesh-db/migrations/20260414000001_contacts.sql`] — DDL contacts (engine InnoDB, charset utf8mb4_unicode_ci, FULLTEXT compatible)
- [Source: `crates/kesh-db/migrations/20260415000001_products.sql`] — DDL products
- [Source: `crates/kesh-db/migrations/20260412000001_journal_entries.sql`] — DDL journal_entries
- [Source: `crates/kesh-db/migrations/20260416000001_invoices.sql`] — DDL invoices (pas de FULLTEXT prévu sur cette table directement — bénéficie via JOIN sur contacts)
- [Source: `crates/kesh-db/src/lib.rs:19`] — `MIGRATOR` macro
- [Source: `CLAUDE.md` Code Quality Rules] — DRY, Documentation, Testing, E2E Playwright
- [Source: `CLAUDE.md` Review Iteration Rule] — multi-pass code review obligatoire si finding > LOW
- [Source: `CLAUDE.md` Issue Tracking Rule] — KF-005 fermeture via issue GitHub #5 + commit `closes #5`
- [Source: PR #51 closure 2026-04-29] — Story 7-3 KF-004 closed, pattern multi-pass review (Sonnet → Haiku) validé

### Prior story intelligence (Story 7-3 / PR #51 mergée 2026-04-29)

- **Pattern multi-pass review confirmé** : Sonnet ×3 (P1) → Haiku ×3 (P2). Critère d'arrêt CLAUDE.md atteint avec convergence orthogonale (Acceptance Auditor 0 findings P2).
- **Patterns réutilisables** :
  - Decision sourcing dans des sections §X (ex. §race-condition Story 7-3 a permis tracer la décision « race acceptée v0.1 »). À reproduire ici avec §mode FULLTEXT, §UX impact, §migration online.
  - Faux positifs « scale-sensitive » sur `Decimal::eq` rejetés en P2 par vérification empirique. Pour 7-4, la même vérification empirique peut être nécessaire sur le comportement EXPLAIN MariaDB.
- **Issues follow-up déjà tracées** :
  - [#49 KF-020](https://github.com/guycorbaz/kesh/issues/49) — `invoices::update` SELECT FOR UPDATE (Epic 8 prerequisite).
  - [#50 KF-021](https://github.com/guycorbaz/kesh/issues/50) — Test E2E déterministe race REPEATABLE READ.

### Git intelligence (5 derniers commits)

```
7529204 Story 7-3: KF-004 — update() no-op short-circuit (closes #4) (#51)
64dad5e Stories 3-7 + 7-2 closure — Gestion exercices comptables + KF-003 TVA DB-driven (#48)
d092224 Story 3-7: Gestion des exercices comptables (spec) (#46)
b63dc4e Story 7-1: KF-002 Multi-Tenant Audit + Code Review Pass 4 Remediation (#42)
7c8822d Story 6-2: Multi-tenant scoping refactor — Pass 3 remediation complete (#29)
```

**Patterns émergents** :
- Toutes les stories Epic 7 (tech debt) suivent le pattern : impl T1-Tn → review Pass 1 → review Pass 2 → closure commit avec `closes #N`.
- 100% des stories récentes passent par PR (branche protégée `main`).
- Les commits de PR sont en squash-merge (`(#NN)` suffixe).
- Documentation systématique dans `docs/<topic>-patterns.md` quand un pattern réutilisable est introduit (cf. `docs/optimistic-locking-patterns.md` Story 7-3).

### Latest tech information (MariaDB 10.11 FULLTEXT, vérifications préalables)

- MariaDB 10.11 supporte InnoDB FULLTEXT depuis 10.0.5 (largement disponible).
- `ALGORITHM=INPLACE, LOCK=NONE` supporté pour `ADD FULLTEXT` depuis MariaDB 10.2 (✓ v0.1 cible).
- `innodb_ft_min_token_size` défaut = 3 caractères (modifiable mais nécessite redémarrage + REBUILD INDEX).
- `BOOLEAN MODE` opérateurs : `+ -` (must/must-not), `> <` (relevance modifier), `( )` (grouping), `~` (penalty), `*` (prefix wildcard, suffix non-supporté), `"..."` (phrase).
- `NATURAL LANGUAGE MODE` : ranking BM25-like, stop-words filtrés selon `default_collation` (utf8mb4_unicode_ci stop-words = liste anglaise par défaut, NON francophone — autre raison de préférer BOOLEAN qui n'utilise pas de stop-words).
- Limite `MATCH ... AGAINST` : 1024 caractères max par défaut, largement suffisant pour Kesh.

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

(à remplir par le dev)

### Completion Notes List

(à remplir par le dev)

### File List

(à remplir par le dev)
