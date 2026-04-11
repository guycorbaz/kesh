# Story 3.4: Recherche, pagination & tri des écritures

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **utilisateur (comptable ou indépendant)**,
I want **retrouver rapidement mes écritures par libellé, montant, date, journal, avec tri et pagination**,
so that **je puisse naviguer efficacement dans ma comptabilité sans scroll infini ni Ctrl+F**.

### Contexte

Quatrième story de l'Epic 3. S'appuie sur **3.2** (création d'écritures, repository `list_recent_by_company`) et **3.3** (modification/suppression). Cette story :

1. **Étend le repository** `journal_entries` avec une fonction `list_by_company_paginated` qui accepte un struct `JournalEntryListQuery` (filtres + tri + pagination) et retourne `{ items, total, offset, limit }`.
2. **Étend la route** `GET /api/v1/journal-entries` avec des query params (`description`, `amountMin`, `amountMax`, `dateFrom`, `dateTo`, `journal`, `sortBy`, `sortDir`, `offset`, `limit`) et remplace la réponse flat par une envelope `{ items, total, offset, limit }` (pattern `ListResponse<T>`).
3. **Remplace la liste frontend** dans `+page.svelte` par un tableau avec barre de filtres (inputs controlés + debounced fetch), headers cliquables pour le tri, et contrôles de pagination en pied de tableau.
4. **Établit un pattern réutilisable** — la struct de query, l'envelope de réponse, et le composant TypeScript `usePagination` (hook interne à la feature) serviront de modèle pour les stories ultérieures (contacts 4.1, factures 5.x, imports 6.x).

**Scope strictement limité** : cette story touche UNIQUEMENT la liste des écritures de `/journal-entries`. La refactorisation générique en module partagé est **différée** : si le pattern se répète en story 4.1 (contacts), on extraira à ce moment. YAGNI.

### Scope verrouillé — ce qui sort de la story

- **Recherche par numéro de facture** (AC source epic 3 ligne 813) : **reportée en story 5.x (facturation)**. Justification : le champ `invoice_number` n'existe pas dans `journal_entries` — il sera introduit par l'epic 5 lorsque les factures génèreront automatiquement leurs écritures comptables. Ajout prématuré = scope creep. Test `test.skip` avec note explicite.
- **Recherche full-text avancée** (FULLTEXT index MariaDB, stemming, tolérance fautes de frappe) : v0.1 se contente d'un `LIKE '%...%'` sur la description (suffisant pour 2-5 users et < 10k écritures/exercice).
- **Sauvegarde de filtres prédéfinis** (ex: « mes écritures de banque de ce mois ») : post-MVP.
- **Export du résultat filtré** (CSV/PDF) : story 7.x rapports & exports.
- **Tri multi-colonnes** : v0.1 = tri sur **une seule colonne**. Shift+clic pour multi-tri reporté post-MVP.

### Décisions de conception

- **Struct `JournalEntryListQuery`** dans `kesh-db/src/repositories/journal_entries.rs` (module-local, pas publique au-delà). Tous les champs optionnels (`Option<T>`). Défauts : `limit = 50`, `offset = 0`, `sort_by = SortBy::EntryDate`, `sort_dir = SortDirection::Desc`.
- **Enums `SortBy` + `SortDirection`** dans `kesh-core` (logique pure, sérialisation serde ; pas de trait SQLx). Variants `SortBy` : `EntryDate | EntryNumber | Journal | Description`. Pas de tri par « total » : ce serait un SUM par ligne, trop coûteux pour v0.1. Variants `SortDirection` : `Asc | Desc`.
  - **Raison `kesh-core`** : ces enums sont sérialisés dans les query params HTTP et potentiellement réutilisables par d'autres listes (contacts, factures). Pas de dépendance DB → vivent dans la logique métier pure.
- **Mapping `SortBy` → colonne SQL** via une fonction locale `sort_column(sort_by: SortBy) -> &'static str` qui retourne la colonne exacte (`"entry_date"`, `"entry_number"`, `"journal"`, `"description"`). **Whitelist stricte** — jamais de concat de string user-input dans le SQL (anti-injection). Le tri `ORDER BY {col} {dir}` utilise `{col}` depuis la whitelist et `{dir}` = `ASC` / `DESC` littéral (pas de bind param — SQLx ne supporte pas les paramètres pour `ORDER BY`).
- **Filtres WHERE dynamiques** : construction SQL avec conditions conditionnelles basées sur `Option::is_some()`. Exemple :
  ```rust
  let mut conditions: Vec<&str> = vec!["company_id = ?"];
  let mut bindings: Vec<Binding> = vec![Binding::I64(company_id)];
  if query.description.is_some() {
      conditions.push("description LIKE ?");
      bindings.push(Binding::String(format!("%{}%", query.description.unwrap())));
  }
  // ... etc
  let sql = format!("SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE {} ORDER BY {} {} LIMIT ? OFFSET ?", conditions.join(" AND "), sort_col, sort_dir);
  ```
  - **Attention** : SQLx 0.8 n'a pas de query builder natif simple. Le pattern `QueryBuilder` existe mais est overkill ici. **Décision** : construire la query en deux temps : (a) une fonction `build_where_clauses(&query) -> (String, Vec<DynBinding>)` retourne les conditions + bindings, (b) le call site formatte le SQL final et bind les paramètres dans l'ordre. Helper générique pour les bindings : utiliser l'enum local `DynBinding` avec variants `I64`, `String`, `Decimal`, `Date`, ou plus simple — utiliser `sqlx::QueryBuilder::<MySql>` qui supporte `push_bind` pour composer dynamiquement.
  - **Décision finale tranchée** : utiliser `sqlx::QueryBuilder::<MySql>::new("SELECT ... FROM journal_entries")` puis `.push(" WHERE company_id = ")` + `.push_bind(company_id)` + conditionnellement ajouter les filtres. Pour `ORDER BY` (non-paramétrable) utiliser `.push(format!(" ORDER BY {sort_col} {sort_dir}"))`. Pour `LIMIT/OFFSET` utiliser `.push_bind(limit).push_bind(offset)`. Ce pattern est celui recommandé par la doc SQLx 0.8.
- **Count total séparé** : deux queries consécutives dans la même méthode — un `SELECT COUNT(*)` avec les mêmes `WHERE` (pas de `ORDER BY`/`LIMIT`) pour `total`, puis un `SELECT ... ORDER BY ... LIMIT OFFSET` pour `items`. Pas de `SQL_CALC_FOUND_ROWS` (déprécié MariaDB 10.6+). Acceptable pour v0.1 : 2 queries parallèles via `tokio::try_join!` si nécessaire, sinon séquentielles.
- **Décision simplicité** : séquentiel (2 queries enchaînées). `try_join!` introduirait une complexité supplémentaire (gestion de 2 pools ou 2 tx, pattern plus lourd) sans bénéfice pour 50-500 rows. Si le benchmark post-MVP montre un goulot, on refactorera.
- **Limites hard** : `limit` max 500 (borné dans la route handler), `offset` max `i64::MAX` (acceptable — MariaDB plafonne de toute façon). Défaut `limit = 50`.
- **Envelope réponse** `ListResponse<T> { items: Vec<T>, total: i64, offset: i64, limit: i64 }` dans `kesh-api/src/routes/mod.rs` (module partagé pour réutilisation future). Sérialisation camelCase.
- **Debouncing frontend** : 300 ms sur les inputs text (`description`, `amount_min`, `amount_max`). Dates et journal → refetch immédiat au changement. Implémentation via un helper `debounce<T>(fn, delay): fn` local à la feature (pas de dépendance externe).
- **UX tri** : clic sur un header → toggle direction si même colonne, sinon change colonne et reset direction à `Desc` (convention comptable — plus récent en haut). Indicateur visuel : flèche ↓ ou ↑ à droite du label.
- **UX pagination** : pied de tableau avec « X-Y sur N », boutons Précédent/Suivant, sélecteur de `pageSize` (25/50/100). Pas de saut direct à une page (pas de numérotation cliquable) — suffit pour v0.1. **Pas de lazy loading / infinite scroll** (UX comptable favorise pagination explicite).
- **État dans l'URL query string** (optionnel mais recommandé) : les filtres/sort/offset sont reflétés dans `?description=...&sortBy=...` via `$page.url.searchParams`. Permet le partage de lien, le rafraîchissement, et le retour arrière. **Décision** : **oui**, patrimoine facile avec SvelteKit et pattern standard. Bind via `goto(url, { replaceState: true, noScroll: true })` sans reload.
- **Réutilisation du dialog de suppression et du bouton ✎/✕ story 3.3** : inchangés, continuent de fonctionner dans le tableau paginé. La `loadAll()` de la story 3.3 devient `loadFiltered()` (avec les query params courants).
- **Pas de cache client** : chaque changement de filtre/page → nouveau fetch. Acceptable à 50ms+ pour 2-5 users.

## Acceptance Criteria (AC)

1. **Liste avec pagination par défaut** — Given la page `/journal-entries`, When affichage initial, Then le backend retourne les 50 dernières écritures triées par `{sort_by} {sort_dir}, entry_number DESC` (secondary sort **toujours** sur `entry_number DESC` pour stabilité — même si l'utilisateur trie par `Journal`, les écritures du même journal restent ordonnées par numéro décroissant). Défaut : `sort_by = EntryDate`, `sort_dir = Desc`. Envelope `{ items, total, offset: 0, limit: 50 }`. Le pied de tableau affiche « 1-50 sur X » (ou « Aucune écriture » si vide).
2. **Recherche par libellé (LIKE)** — Given un input « description » dans la barre de filtres, When l'utilisateur tape « facture » (après debounce 300 ms), Then `GET /api/v1/journal-entries?description=facture&offset=0` → le backend retourne les écritures dont `description LIKE '%facture%'` (case-insensitive via la **collation de table `utf8mb4_unicode_ci`** héritée depuis la migration story 3.2 — pas de clause `COLLATE` explicite dans la requête puisque c'est déjà le default de la table), paginées.
3. **Filtre par plage de montants** — Given 2 inputs « montant min » et « montant max » (décimaux, vides = pas de filtre), When l'utilisateur saisit `amountMin=100&amountMax=500`, Then le backend filtre les écritures dont le **total** (= `SUM(debit)` par entry) est dans `[100, 500]`. **Implémentation** : via une sous-requête `WHERE id IN (SELECT entry_id FROM journal_entry_lines GROUP BY entry_id HAVING SUM(debit) BETWEEN ? AND ?)` — acceptable pour v0.1 avec < 10k écritures.
4. **Filtre par plage de dates** — Given 2 inputs date `dateFrom` et `dateTo`, When saisie, Then le backend filtre sur `entry_date BETWEEN ? AND ?` (bornes incluses).
5. **Filtre par journal** — Given un dropdown « journal » (5 valeurs + « Tous »), When sélection, Then le backend filtre sur `journal = ?`.
6. **Tri par colonne** — Given un header cliquable (Date / N° / Journal / Libellé), When clic, Then :
   - Si la colonne est déjà triée → toggle direction (asc ↔ desc).
   - Sinon → change de colonne + direction = desc (convention comptable).
   - Une flèche ↓ ou ↑ apparaît à droite du header actif.
7. **Pagination — Précédent/Suivant** — Given une liste de plus de 50 écritures, When clic sur « Suivant », Then `offset += limit`, fetch, et le pied affiche la nouvelle fenêtre. « Précédent » désactivé si `offset === 0`, « Suivant » désactivé si `offset + limit >= total`.
8. **Sélecteur de taille de page** — Given le sélecteur « 25 / 50 / 100 », When changement, Then `limit` mis à jour, `offset` remis à 0, refetch.
9. **Envelope API standard** — Given `GET /api/v1/journal-entries?...`, When réponse, Then format JSON strict : `{ "items": [...], "total": 123, "offset": 50, "limit": 50 }`. Les 4 champs sont obligatoires. `total` est le nombre total d'écritures correspondant aux filtres (sans pagination).
10. **Anti-injection SQL sur le tri** — Given un attaquant qui envoie `?sortBy=entry_date;DROP TABLE journal_entries--`, When le backend traite la requête, Then la désérialisation serde de `Query<JournalEntryListQueryRequest>` échoue car la valeur n'est pas dans l'enum `SortBy` → Axum retourne `400 Bad Request` via `QueryRejection` (comportement par défaut d'Axum — voir T3.2bis pour le mapping explicite en `AppError`). Aucun SQL n'est exécuté. Le test vérifie le code HTTP 400 et l'absence de row `DROP TABLE` dans les logs.
11. **Limite hard sur `limit`** — Given `?limit=10000`, When requête, Then le backend clamp silencieusement à 500 (ou retourne `400` si on préfère être strict — **décision** : clamp silencieux, pour être lenient avec les clients).
12. **État URL synchronisé** — Given un filtre actif, When l'utilisateur rafraîchit la page ou partage l'URL, Then les filtres et la pagination sont restaurés depuis `$page.url.searchParams`.
13. **Debouncing inputs** — Given l'utilisateur tape dans l'input description, When chaque keystroke, Then un seul fetch déclenché 300 ms après le dernier keystroke (pas de burst). Test Vitest sur le helper `debounce`.
14. **Édition/suppression survivent au filtre** — Given un tableau filtré, When l'utilisateur clique ✎ ou ✕ sur une ligne, Then le formulaire d'édition ou le dialog de suppression s'ouvre normalement (patterns story 3.3 inchangés). Après action, `loadFiltered()` rafraîchit la liste en préservant les filtres courants.
15. **Tests** :
    - Tests d'intégration DB `journal_entries::list_by_company_paginated` : filtre par description, filtre par plage montant, filtre par dates, filtre par journal, tri asc/desc sur chaque colonne, pagination, count total séparé.
    - Tests unitaires `kesh-core` : sérialisation/désérialisation `SortBy` et `SortDirection`, enum exhaustif (coverage).
    - Tests Vitest frontend : helper `debounce` (fake timers), helper de construction des query params depuis l'état, helper d'extraction des filtres depuis `URLSearchParams`.
    - Tests Playwright : filtre par libellé avec debounce, filtre par plage montant, filtre par date, tri ascendant/descendant sur une colonne, pagination Précédent/Suivant, changement de `pageSize`, état URL préservé après rafraîchissement. Scénario **reporté** : recherche par numéro de facture (nécessite le champ `invoice_number` — story 5.x).
16. **i18n** — Libellés des filtres, placeholder, boutons pagination, libellé du sélecteur de taille dans les 4 langues. Aucun hardcode (règle A3).

## Tasks / Subtasks

### T1 — Enums `SortBy` + `SortDirection` dans kesh-core (AC: #6, #10, #15)

- [x] T1.1 Créer `crates/kesh-core/src/listing/mod.rs` (nouveau module) avec :
  - Enum `SortDirection { Asc, Desc }` avec `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]` + méthode **`as_sql_keyword() -> &'static str`** retournant `"ASC"` ou `"DESC"` (littéral validé, jamais exposé comme bind — nom explicite pour éviter toute confusion avec une « représentation SQL d'un statement »).
  - Enum `SortBy` — variants pour les listes d'écritures pour v0.1 : `EntryDate | EntryNumber | Journal | Description`. Mêmes dérives. Méthode **`as_sql_column() -> &'static str`** retournant la colonne SQL littérale (whitelist anti-injection). Les deux méthodes suivent la même convention de nommage `as_sql_*`.
  - **Sérialisation serde — casse des variants** : les variants PascalCase (`"EntryDate"`, `"Desc"`) sont conservés par défaut. **Cohérence avec le pattern du projet** : l'enum `Journal` (story 3.2) utilise déjà la même convention `"Achats"`, `"Ventes"`, etc. Le DTO route `JournalEntryListQueryRequest` annote ses **champs** en `#[serde(rename_all = "camelCase")]` (donc `amountMin`, `sortBy`, `dateFrom`), mais les **valeurs** des enums imbriqués restent PascalCase par design Rust/serde standard (`rename_all` ne propage pas aux variants). **Exemple** : requête `GET /api/v1/journal-entries?amountMin=100&sortBy=EntryDate&sortDir=Desc` — les NOMS sont camelCase, les VALEURS sont PascalCase. C'est le comportement Rust idiomatique et cohérent avec le reste de l'API Kesh. **Ne PAS** tenter d'uniformiser en mettant `#[serde(rename_all = "camelCase")]` sur les enums — cela créerait une divergence avec `Journal` déjà en place.
  - **Attention serde** : les noms sérialisés sont en PascalCase par défaut (`"EntryDate"`, `"Desc"`). Le frontend envoie exactement ces valeurs. Si le client envoie `"entry_date"` (snake_case), serde rejette — bien.
  - Tests unitaires : roundtrip serde, exhaustivité des variants, `as_sql_column()` retourne chaque colonne attendue, `as_sql()` retourne `"ASC"`/`"DESC"`.
- [x] T1.2 Ajouter `pub mod listing;` dans `crates/kesh-core/src/lib.rs`.

### T2 — Repository `journal_entries::list_by_company_paginated` (AC: #1-#9, #11, #15)

- [x] T2.1 Dans `crates/kesh-db/src/repositories/journal_entries.rs`, ajouter :
  ```rust
  pub struct JournalEntryListQuery {
      pub description: Option<String>,
      pub amount_min: Option<Decimal>,
      pub amount_max: Option<Decimal>,
      pub date_from: Option<NaiveDate>,
      pub date_to: Option<NaiveDate>,
      pub journal: Option<Journal>,
      pub sort_by: SortBy,            // défaut EntryDate
      pub sort_dir: SortDirection,    // défaut Desc
      pub limit: i64,                 // défaut 50, clamp max 500
      pub offset: i64,                // défaut 0
  }

  pub struct JournalEntryListResult {
      pub items: Vec<JournalEntryWithLines>,
      pub total: i64,
      pub offset: i64,
      pub limit: i64,
  }

  pub async fn list_by_company_paginated(
      pool: &MySqlPool,
      company_id: i64,
      query: JournalEntryListQuery,
  ) -> Result<JournalEntryListResult, DbError>
  ```
- [x] T2.2 Implémentation :
  1. **Clamp** `limit` à `[1, 500]`, `offset` à `>= 0` — **garde-fou défensif uniquement**. La source de vérité du clamp est le **route handler** (T3.2), qui applique le clamp canonique AVANT d'appeler le repository. Ce clamp repository est là au cas où un call site futur contournerait le handler. **Ne pas remonter d'erreur** ici — silencieux.
  2. **Construire la clause WHERE** via `sqlx::QueryBuilder::<MySql>::new(...)`. Conditions dynamiques :
     - Toujours : `WHERE company_id = ?`
     - Si `description.is_some()` : `AND description LIKE ?` avec `format!("%{}%", desc)` (attention : les `%` dans la saisie user doivent être échappés — sinon `%50%` devient `%%50%%` = match tout avec 50. **Décision** : échapper `%` et `_` dans la saisie via `.replace('%', "\\%").replace('_', "\\_")` avant concat + ajouter `ESCAPE '\\'` à la clause LIKE).
     - Si `date_from.is_some()` : `AND entry_date >= ?`
     - Si `date_to.is_some()` : `AND entry_date <= ?`
     - Si `journal.is_some()` : `AND journal = ?`
     - Si `amount_min.is_some() || amount_max.is_some()` : `AND id IN (SELECT entry_id FROM journal_entry_lines GROUP BY entry_id HAVING SUM(debit) BETWEEN ? AND ?)`.
       - **Schéma vérifié (migration story 3.2)** : `journal_entry_lines.debit DECIMAL(19,4)` → plage max `999'999'999'999'999.9999` (15 chiffres entiers + 4 décimales).
       - **Bornes par défaut** : `min = Decimal::ZERO` si absent, `max = Decimal::from_str("999999999999999.9999").unwrap()` si absent (valeur safe, largement sous la limite du type, pas de risque d'overflow).
       - **Ne PAS utiliser `Decimal::MAX`** côté Rust — le type `rust_decimal::Decimal` supporte jusqu'à ~10²⁸ mais MariaDB `DECIMAL(19,4)` clamperait / tronquerait → erreur SQL. Utiliser la constante DB-safe documentée ci-dessus.
       - Performance acceptable v0.1 (< 10k entries) — l'index `idx_jel_entry` sur `journal_entry_lines(entry_id)` est suffisant pour le GROUP BY.
  3. **Query 1 — count total** : `SELECT COUNT(*) FROM journal_entries WHERE {conditions}` (pas de `ORDER BY`/`LIMIT`).
  4. **Query 2 — items** : `SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE {conditions} ORDER BY {sort_col} {sort_dir} LIMIT ? OFFSET ?`. **`sort_col` et `sort_dir` sont littéraux** depuis les enums validés (anti-injection).
  5. **Charger les lignes** pour chaque entry retournée (N+1 acceptable pour `limit <= 100`).
  6. Retourner `JournalEntryListResult { items, total, offset, limit }`.
- [x] T2.3 **Helper interne `push_where_clauses`** : extraire la logique conditionnelle dans une fonction privée. Signature : `fn push_where_clauses<'a>(qb: &mut QueryBuilder<'a, MySql>, company_id: i64, query: &'a JournalEntryListQuery)`. **Attention — appel sur deux `QueryBuilder` DISTINCTS** :
  ```rust
  // Instance 1 — count (aucune clause ORDER BY / LIMIT)
  let mut count_qb: QueryBuilder<MySql> =
      QueryBuilder::new("SELECT COUNT(*) FROM journal_entries");
  push_where_clauses(&mut count_qb, company_id, &query);
  let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await.map_err(map_db_error)?;

  // Instance 2 — items (SELECT + ORDER BY + LIMIT OFFSET)
  let mut items_qb: QueryBuilder<MySql> =
      QueryBuilder::new(format!("SELECT {ENTRY_COLUMNS} FROM journal_entries"));
  push_where_clauses(&mut items_qb, company_id, &query);
  items_qb.push(format!(" ORDER BY {} {}, entry_number DESC LIMIT ", sort_col, sort_dir_sql));
  items_qb.push_bind(clamped_limit);
  items_qb.push(" OFFSET ");
  items_qb.push_bind(clamped_offset);
  let entries: Vec<JournalEntry> = items_qb.build_query_as::<JournalEntry>().fetch_all(pool).await.map_err(map_db_error)?;
  ```
  **CRITIQUE** : ne JAMAIS réutiliser le même `QueryBuilder` pour le count et les items (un `QueryBuilder` encode un état mutable et ne peut être réutilisé après `build_*`). Ce pattern à deux instances est le seul correct.
- [x] T2.4 Tests d'intégration DB : `test_list_paginated_default`, `test_list_filter_description`, `test_list_filter_description_escapes_percent`, `test_list_filter_amount_range`, `test_list_filter_date_range`, `test_list_filter_journal`, `test_list_sort_each_column_asc_desc`, `test_list_pagination_offset_limit`, `test_list_count_accurate_after_filter`. Nettoyage via `delete_all_by_company`.

### T3 — Envelope `ListResponse<T>` + extension route GET (AC: #9, #10, #11)

- [x] T3.1 Créer `crates/kesh-api/src/routes/mod.rs` (ou nouveau fichier `types.rs` exporté) :
  ```rust
  #[derive(Debug, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct ListResponse<T: Serialize> {
      pub items: Vec<T>,
      pub total: i64,
      pub offset: i64,
      pub limit: i64,
  }
  ```
  - Réutilisable par les listes futures (contacts, factures, imports). Pour v0.1 on le met dans `routes/mod.rs` — extraction post-MVP si besoin.
- [x] T3.2 Étendre `routes/journal_entries.rs` :
  - Nouveau DTO `JournalEntryListQueryRequest` (serde deserialize depuis query params) avec tous les champs optionnels. Dates en `String` parsées via `NaiveDate::from_str` côté handler pour pouvoir rejeter avec `AppError::Validation` si format invalide.
  - Défaut `limit = 50` via `#[serde(default = "default_limit")]`, `offset = 0`, `sort_by = SortBy::EntryDate`, `sort_dir = SortDirection::Desc`.
  - Clamp `limit` à `[1, 500]` dans le handler (lenient — pas de 400, on ajuste silencieusement).
  - Construire `JournalEntryListQuery` et appeler `journal_entries::list_by_company_paginated`.
  - Retourner `Json(ListResponse { items, total, offset, limit })`.
  - Parse des dates : utiliser `NaiveDate::from_str(...)` — si erreur, `AppError::Validation("dateFrom invalide : {date}")`.
  - Parse du `amount_min`/`amount_max` : `Decimal::from_str(...)` — si erreur, `AppError::Validation`.
- [x] T3.3 **Remplacer** l'ancien handler `list_journal_entries` (qui utilisait `list_recent_by_company`) par le nouveau. **Supprimer** `list_recent_by_company` (vérifié 2026-04-10 via `grep -rn list_recent_by_company` : appelée uniquement dans `routes/journal_entries.rs:190` + 1 test `repositories/journal_entries.rs:955`. **Pas d'appel depuis le dashboard homepage story 2.4 ni ailleurs.**). Supprimer aussi le test d'intégration DB orphelin `test_list_recent_sorted_desc` — il sera remplacé par les nouveaux tests `test_list_paginated_*` de T2.4.
- [x] T3.3bis **Mapping `QueryRejection` → `AppError`** : par défaut, Axum retourne un `400 Bad Request` avec un corps texte (non-JSON) lorsqu'une désérialisation `Query<T>` échoue. Pour conserver le format de réponse structuré `{ "error": { "code": "VALIDATION_ERROR", "message": ... } }` cohérent avec les autres routes, utiliser le pattern Axum standard :
  - **Option A (recommandée, minimaliste)** : accepter le comportement par défaut Axum. Le code HTTP est bien `400`, le corps est texte. Le frontend accepte les 400 comme "validation error" peu importe le corps. Test : vérifier le code HTTP 400.
  - **Option B (plus propre)** : remplacer `Query<JournalEntryListQueryRequest>` par un extractor custom qui wrappe `serde_urlencoded::from_str` et retourne `AppError::Validation(msg)` en cas d'erreur. Plus de code, mais cohérent avec le format d'erreur du reste de l'API.
  - **Décision pour v0.1** : **Option A** (comportement Axum par défaut). Simplicité prioritaire. Si le frontend montre une UX cassée à cause du format texte, refactor en Option B post-MVP.
  - **Documenter dans les commentaires du handler** que le comportement par défaut Axum est intentionnel, pour éviter qu'un dev futur implémente la Option B sans raison.
- [x] T3.4 Tests unitaires de mapping HTTP : `list_returns_envelope`, `list_clamps_limit_above_500`, `list_rejects_invalid_date_format`, `list_rejects_invalid_sortby_enum` (test de désérialisation serde avec valeur hors enum → `400` code HTTP, pas de vérif du format du corps puisque Option A retient le comportement par défaut Axum).

### T4 — Frontend : types + helper query params (AC: #2-#9, #12, #13, #16)

- [x] T4.1 Étendre `frontend/src/lib/features/journal-entries/journal-entries.types.ts` :
  ```ts
  export type SortBy = 'EntryDate' | 'EntryNumber' | 'Journal' | 'Description';
  export type SortDirection = 'Asc' | 'Desc';

  export interface JournalEntryListQuery {
      description?: string;
      amountMin?: string;
      amountMax?: string;
      dateFrom?: string;
      dateTo?: string;
      journal?: Journal;
      sortBy?: SortBy;
      sortDir?: SortDirection;
      offset?: number;
      limit?: number;
  }

  export interface ListResponse<T> {
      items: T[];
      total: number;
      offset: number;
      limit: number;
  }
  ```
- [x] T4.2 Mettre à jour `journal-entries.api.ts::fetchJournalEntries(query?: JournalEntryListQuery)` — construire les query params via `URLSearchParams`, omettre les champs vides/undefined. Changer le type de retour de `Promise<JournalEntryResponse[]>` vers `Promise<ListResponse<JournalEntryResponse>>`.
- [x] T4.3 Créer un helper `frontend/src/lib/features/journal-entries/query-helpers.ts` :
  - `serializeQuery(query: JournalEntryListQuery): URLSearchParams` — construit les params en omettant les champs vides.
  - `parseQueryFromUrl(searchParams: URLSearchParams): JournalEntryListQuery` — reconstitue l'état depuis l'URL.
  - Tests Vitest dans `query-helpers.test.ts` : roundtrip, omission des champs vides, valeurs par défaut ignorées.
- [x] T4.4 Créer un helper `debounce.ts` (local à la feature, pas de dépendance externe) :
  ```ts
  export interface DebouncedFn<Args extends unknown[]> {
      (...args: Args): void;
      /** Annule le timeout en cours — à appeler au démontage du composant. */
      cancel(): void;
  }

  export function debounce<Args extends unknown[]>(
      fn: (...args: Args) => void,
      delay: number
  ): DebouncedFn<Args> {
      let timeoutId: ReturnType<typeof setTimeout> | null = null;

      const debounced = ((...args: Args) => {
          if (timeoutId) clearTimeout(timeoutId);
          timeoutId = setTimeout(() => fn(...args), delay);
      }) as DebouncedFn<Args>;

      debounced.cancel = () => {
          if (timeoutId) {
              clearTimeout(timeoutId);
              timeoutId = null;
          }
      };

      return debounced;
  }
  ```
  - **Cleanup obligatoire** dans T5.1 : le composant `+page.svelte` doit appeler `.cancel()` au démontage pour éviter un `loadFiltered()` sur un composant démonté. Pattern Svelte 5 :
    ```ts
    const debouncedLoad = debounce(loadFiltered, 300);
    $effect(() => {
        // ... logique qui appelle debouncedLoad(query) ...
        return () => debouncedLoad.cancel();  // cleanup au démontage
    });
    ```
  - Tests Vitest avec `vi.useFakeTimers()` : un seul appel après burst, reset du timer, vérification que `.cancel()` empêche l'appel même après `advanceTimersByTime(delay + 1)`.

### T5 — Frontend : page `+page.svelte` étendue avec filtres, tri, pagination (AC: #1-#14)

- [x] T5.1 Étendre `frontend/src/routes/(app)/journal-entries/+page.svelte` :
  - **Transition BREAKING (ligne concrète)** : le state actuel est `let entries = $state<JournalEntryResponse[]>([]);`. Après refactor, `loadAll()` appelait `fetchJournalEntries()` et faisait `entries = entriesResult.value` (tableau direct). Après story 3.4, `fetchJournalEntries(query)` retourne `Promise<ListResponse<JournalEntryResponse>>`, donc :
    ```ts
    // AVANT (story 3.3)
    if (entriesResult.status === 'fulfilled') entries = entriesResult.value;

    // APRÈS (story 3.4)
    if (entriesResult.status === 'fulfilled') {
        entries = entriesResult.value.items;  // ← extraction depuis l'envelope
        total = entriesResult.value.total;
        offset = entriesResult.value.offset;
        limit = entriesResult.value.limit;
    }
    ```
    TypeScript strict détectera l'incompatibilité à la compilation — pas de risque silencieux.
  - État : `query: JournalEntryListQuery` (initialisé depuis `$page.url.searchParams` via `parseQueryFromUrl`), `entries: JournalEntryResponse[]` (inchangé), `total: number = 0`, `offset: number = 0`, `limit: number = 50`.
  - Fonction `loadFiltered()` qui appelle `fetchJournalEntries(query)` et met à jour `entries`, `total`, `offset`, `limit`. Appelée au mount, sur chaque changement de filtre (via debounce pour les inputs text, immédiat pour dates/journal/sort/pagination), et après `onSuccess` de création/édition/suppression (remplace `loadAll()` story 3.3).
  - **Barre de filtres** au-dessus du tableau : 6 inputs (description, amountMin, amountMax, dateFrom, dateTo, journal dropdown avec option « Tous »). Inputs text utilisent `debounce` 300 ms ; dates et journal sont immédiats. Bouton « Réinitialiser » qui reset tous les filtres et `offset=0`.
  - **Headers cliquables** pour le tri : Date, N°, Journal, Libellé. Clic → toggle direction si même colonne, sinon change colonne + direction=Desc. Indicateur ↓/↑ à droite du label. Libellé et Total **non triables** côté backend → pas d'indicateur (le header « Total » n'a pas d'événement `onclick`).
  - **Pied de tableau** : `« X-Y sur N » + [Précédent] [Suivant] + [25/50/100]`. Précédent désactivé si `offset === 0`, Suivant si `offset + limit >= total`.
  - **Sync URL** : à chaque changement de `query`, appeler `goto(url, { replaceState: true, noScroll: true, keepFocus: true })` avec les query params à jour. Utiliser `$effect` pour réagir aux changements de `query`.
  - Préserver les boutons ✎/✕ et le dialog de suppression (story 3.3 inchangés). `loadFiltered()` remplace `loadAll()` dans les callbacks `onSuccess`.
- [x] T5.2 **Gotcha Svelte 5 + SvelteKit `goto` — décision tranchée** : `goto` avec `replaceState: true` déclenche une mise à jour de `$page.url.searchParams`. Si un `$effect` lit ce searchParams pour restaurer l'état, on obtient une boucle `state → url → state`. **Solution retenue** : wrapper l'appel à `goto` dans `untrack(() => goto(url, { replaceState: true, noScroll: true, keepFocus: true }))` (import `untrack` depuis `svelte`). `untrack` est l'idiome officiel Svelte 5 pour sortir de la réactivité à l'intérieur d'un `$effect`. Cela évite que les dépendances réactives à l'intérieur de `goto` (notamment la lecture de `$page.url`) ne re-déclenchent le même `$effect`. Pattern :
  ```ts
  import { untrack } from 'svelte';
  import { goto } from '$app/navigation';

  $effect(() => {
      const url = new URL($page.url);
      const params = serializeQuery(query);
      url.search = params.toString();
      untrack(() => goto(url, { replaceState: true, noScroll: true, keepFocus: true }));
  });
  ```
  **Pas** d'approche `isUpdatingUrl` flag — trop fragile. **Pas** de `$effect.root` — utile pour du nettoyage manuel mais surdimensionné ici.

### T6 — Clés i18n + tests Playwright (AC: #15, #16)

- [x] T6.1 Ajouter dans les 4 fichiers `.ftl` (FR/DE/IT/EN) :
  - `journal-entries-filter-description` (placeholder)
  - `journal-entries-filter-amount-min`, `journal-entries-filter-amount-max`
  - `journal-entries-filter-date-from`, `journal-entries-filter-date-to`
  - `journal-entries-filter-journal`, `journal-entries-filter-journal-all`
  - `journal-entries-filter-reset` (bouton)
  - `journal-entries-pagination-range` (`{ $from }-{ $to } sur { $total }`)
  - `journal-entries-pagination-prev`, `journal-entries-pagination-next`
  - `journal-entries-pagination-page-size`
  - `journal-entries-sort-asc-indicator`, `journal-entries-sort-desc-indicator` (aria-label pour les flèches de tri)
  - **Clés column labels déjà présentes depuis story 3.2** (vérifié 2026-04-10 via grep) : `journal-entries-col-number`, `journal-entries-col-date`, `journal-entries-col-journal`, `journal-entries-col-description`, `journal-entries-col-total`. **Ne PAS les recréer** — les réutiliser telles quelles comme labels des headers cliquables.
- [x] T6.2 Étendre `frontend/tests/e2e/journal-entries.spec.ts` avec un `describe('Story 3.4 — recherche & pagination')` :
  - `filter-description` : créer 2 écritures avec des libellés distincts, filtrer par un mot, vérifier que seule l'écriture correspondante apparaît.
  - `filter-amount-range` : créer des écritures à 100, 500, 1000, filtrer `amountMin=200&amountMax=800`, vérifier que seule celle à 500 apparaît.
  - `filter-date-range` : créer 3 écritures à des dates différentes, filtrer, vérifier.
  - `filter-journal` : filtrer par « Banque », vérifier.
  - `sort-entry-date` : clic sur header Date, vérifier l'ordre asc/desc.
  - `pagination-next-prev` : créer > 50 écritures (scénario lourd — OU mocker la liste à 150 entries via seed étendu), vérifier Précédent/Suivant.
  - `page-size-change` : changer `pageSize` de 50 à 25, vérifier le nombre de rows.
  - `url-state-preserved` : appliquer des filtres, rafraîchir la page, vérifier que les filtres sont restaurés.
  - `debounce-description` : mesurer que le fetch n'est déclenché qu'une fois après un burst de keystrokes (utiliser `page.route` avec counter).
  - **`test.skip`** : `filter-invoice-number` — reporté à la story 5.x (champ `invoice_number` inexistant en v0.1).
  - **`test.skip`** : `filter-closed-fiscal-year` — pas directement lié mais hors scope 3.4.

## Dev Notes

### Architecture — où va quoi

```
kesh-core/src/listing/mod.rs               # nouveau — SortBy, SortDirection
kesh-db/src/repositories/journal_entries.rs # extension — JournalEntryListQuery, list_by_company_paginated, tests
kesh-api/src/routes/mod.rs                  # extension — ListResponse<T> générique
kesh-api/src/routes/journal_entries.rs      # extension — DTO query request, refactor handler list
frontend/src/lib/features/journal-entries/
├── journal-entries.types.ts                # extension — SortBy, SortDirection, ListQuery, ListResponse
├── journal-entries.api.ts                  # extension — fetchJournalEntries(query)
├── query-helpers.ts                        # nouveau — serializeQuery, parseQueryFromUrl
├── query-helpers.test.ts                   # nouveau
├── debounce.ts                              # nouveau
└── debounce.test.ts                         # nouveau
frontend/src/routes/(app)/journal-entries/+page.svelte  # extension — filtres, tri, pagination, sync URL
kesh-i18n/locales/*/messages.ftl            # extension — 11 clés × 4 langues
frontend/tests/e2e/journal-entries.spec.ts  # extension — describe Story 3.4
```

### Patterns existants à réutiliser

- **Pattern transactionnel repository** (story 3.2/3.3) : `list_by_company_paginated` N'A PAS besoin de transaction (lecture seule) — `&MySqlPool` suffit.
- **`sqlx::QueryBuilder`** : disponible dans SQLx 0.8 (feature `macros` active), pattern recommandé pour les WHERE dynamiques.
- **Helper `get_company`** (v0.1 mono-company) : réutiliser dans le handler `list_journal_entries` (déjà en place).
- **`i18nMsg` frontend** : import depuis `$lib/features/onboarding/onboarding.svelte` (attention : **sans `.ts`**, c'est bien un fichier `.svelte.ts` dont le chemin import s'écrit sans extension). Pattern confirmé dans `+page.svelte` story 3.3 ligne 5.
- **`Intl.NumberFormat('de-CH')`** : pour le formatage suisse des totaux dans le pied de tableau.
- **shadcn-svelte** : composants `Input`, `Select`, `Button` — aucun nouveau composant externe.

### Pièges identifiés

1. **SQL injection sur `ORDER BY`** : SQLx ne supporte PAS les bind params pour la clause `ORDER BY`. La seule protection est la **whitelist stricte** via les enums `SortBy::as_sql_column()` qui retournent des littéraux `&'static str`. Toute mention de `query.sort_by` dans le SQL DOIT passer par cette fonction, jamais par formatage direct d'une string utilisateur.
2. **Échappement `%` et `_` dans LIKE** : si un utilisateur cherche « 50% » (pourcentage), le `%` doit être échappé. Côté Rust (littéral source), écrire `" ESCAPE '\\\\'"` (4 backslashes en source → 2 backslashes dans la string runtime → 1 backslash dans le SQL qui est la syntaxe d'`ESCAPE`). Côté escape du user input : `input.replace('%', "\\\\%").replace('_', "\\\\_")` (même logique — 4 backslashes en source Rust pour produire `\%` littéral dans le SQL). Sinon `LIKE '%50%%'` matche toute string contenant « 50 » suivi de n'importe quoi. Idem pour `_`.
3. **`QueryBuilder::push_bind` vs types multiples** : le `QueryBuilder` retourne `&mut Self` mais chaque `.push_bind(val)` exige que `val` implémente `Encode<MySql>`. Les types variables (String, i64, Decimal, NaiveDate) doivent être encodés via des branches conditionnelles. **Attention** : ne pas passer un `Option<T>` — tester `is_some()` avant de push.
4. **`sqlx::QueryBuilder::build_query_as::<JournalEntry>()`** : la query finale est compilée en un objet `Query<'_, MySql, MySqlArguments>`. Pour récupérer les rows typées, utiliser `.build_query_as::<JournalEntry>()` puis `.fetch_all(pool)`. Vérifier la syntaxe exacte SQLx 0.8.
5. **Count + items dans la même transaction ?** : pas nécessaire pour v0.1 — acceptable que `total` soit légèrement obsolète si une écriture est créée entre les 2 queries (race très courte, PME 2-5 users). Post-MVP, si le besoin se fait sentir, utiliser une tx READ UNCOMMITTED ou snapshot.
6. **Date parsing côté backend** : `NaiveDate::from_str("2026-04-10")` fonctionne pour ISO. Mais si le frontend envoie `"10/04/2026"` (format suisse), ça échoue. **Décision** : le frontend envoie TOUJOURS en ISO (format HTML `<input type="date">` natif). Documenter dans le DTO.
7. **`limit = 0`** : interdit (retournerait 0 items mais peut être un bug client). Clamper à `max(1, limit.min(500))`.
8. **Svelte 5 `$effect` + `goto`** : le pattern de sync URL peut créer une boucle de réactivité. Utiliser un `let isInternal = $state(false)` pour flagger les updates propres au store, OU utiliser `untrack(() => goto(...))` de `svelte`. Tester manuellement pendant T5.
9. **`debounce` + cleanup** : quand le composant se démonte, le `setTimeout` en cours doit être annulé pour éviter un `loadFiltered()` sur un composant démonté. Implémenter un cleanup dans `$effect(() => { return () => { if (timeoutId) clearTimeout(timeoutId); } })` ou utiliser `onDestroy`.
10. **Sous-requête `amount_min/max` performance** : `id IN (SELECT entry_id FROM journal_entry_lines GROUP BY entry_id HAVING SUM(debit) BETWEEN ? AND ?)` — MariaDB optimise ça raisonnablement avec l'index `idx_jel_entry`. Pour un très grand volume post-MVP, on pourra matérialiser un champ `total_debit` sur `journal_entries` avec un trigger. YAGNI v0.1.

### Previous Story Intelligence (3.1, 3.2, 3.3)

- **Enum serde PascalCase par défaut** (3.2 F1) : les variants `SortBy::EntryDate` sérialisent en `"EntryDate"`. Le frontend envoie exactement cette casse. Tout autre format = 400. Pas besoin d'annotation `#[serde(rename_all)]`.
- **Helper `get_company` dupliqué** (3.2 patch, 3.3 même pattern) : dans le handler `list_journal_entries`, utiliser la fonction déjà présente en haut de `routes/journal_entries.rs`. Pas d'import croisé.
- **P10 story 3.2** : filtre `company_id` systématique dans toutes les requêtes — défense en profondeur.
- **Pattern de pagination standard** : l'envelope `{ items, total, offset, limit }` est déjà mentionnée dans l'architecture (section Format Patterns). Première implémentation concrète dans cette story — servira de modèle.
- **Dette T9.3 héritée** : pas de framework TestClient HTTP. Couverture via tests DB intégration + Vitest unit + Playwright.
- **`big.js` frontend** (story 3.2 P9) : déjà installé. Pas besoin d'ajouter `decimal.js` — le formatage des totaux continue d'utiliser `formatSwissAmount`.
- **Ordre d'écriture par défaut** (story 3.2 `list_recent_by_company`) : `entry_date DESC, entry_number DESC`. Identique au défaut de cette story.

### Git Intelligence (5 derniers commits + uncommitted)

```
<uncommitted> Story 3.2 + 3.3 (toutes les 2 en review/done, non encore mergées)
b096a22 feat: chart of accounts (Story 3.1)
07f0563 feat: mode Guided/Expert (Story 2.5)
84673de feat: homepage dashboard (Story 2.4)
58c3ad2 feat: onboarding Path B (Story 2.3)
```

- **`crates/kesh-db/src/repositories/journal_entries.rs`** : ~730 lignes actuellement (story 3.2 + 3.3). Story 3.4 ajoute ~150-200 lignes. Pas de refactor nécessaire — garder la structure existante.
- **`routes/journal_entries.rs`** : ~600 lignes actuellement. L'extension 3.4 remplace le handler `list_journal_entries` et ajoute le DTO query. Pas de nouveau fichier.
- **`+page.svelte`** : ~260 lignes actuellement. L'extension 3.4 ajoute la barre de filtres + pagination + sync URL → probablement 150 lignes de plus. Pas besoin de refactor en sous-composants pour v0.1.

### Latest Tech Information

- **SQLx 0.8 `QueryBuilder`** : https://docs.rs/sqlx/0.8/sqlx/struct.QueryBuilder.html — méthode clé `push_bind(value)` pour bind, `push(raw_sql)` pour SQL littéral, `build_query_as::<T>()` pour compiler en query typée, `build()` pour query non-typée.
- **MariaDB `LIKE ... ESCAPE`** : supporté nativement. Syntaxe : `description LIKE ? ESCAPE '\\'`. Attention au double échappement Rust (`"\\\\"` pour un backslash littéral).
- **SvelteKit `goto` + query params** : https://kit.svelte.dev/docs/modules#$app-navigation-goto — `goto(url, { replaceState, noScroll, keepFocus })`. Compatible Svelte 5.
- **Vitest fake timers** : `vi.useFakeTimers()` + `vi.advanceTimersByTime(ms)` + `vi.useRealTimers()` — idéal pour tester `debounce`.

### Security debt (dettes connues acceptées)

- **T9.3** héritée de 3.1/3.2/3.3 : pas de framework TestClient HTTP. Couverture via unit + DB integration + Playwright.
- **Multi-tenant `get_company`** : pattern v0.1 mono-company (dette D1 code review 3.2). Inchangé par cette story.
- **A11y focus trap** (patch P3 story 3.3) : dette reportée à une story A11y transverse post-MVP. Les dialogs de 3.3 réutilisés dans 3.4 héritent de la même limitation.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-3-Story-3.4] — AC BDD lignes 803-818
- [Source: _bmad-output/planning-artifacts/prd.md#FR69-FR70] — Recherche par montant/libellé/facture/date, pagination/tri
- [Source: _bmad-output/planning-artifacts/architecture.md#Format-Patterns] — Envelope `{ items, total, offset, limit }` standardisée
- [Source: _bmad-output/implementation-artifacts/3-2-saisie-ecritures-en-partie-double.md] — Pattern `list_recent_by_company`, helper `get_company`, envelope vs tableau direct
- [Source: _bmad-output/implementation-artifacts/3-3-modification-suppression-ecritures.md] — Pattern frontend table + dialog confirmation, intégration dans `+page.svelte`
- [Source: crates/kesh-db/src/repositories/journal_entries.rs::list_recent_by_company] — Fonction à étendre/remplacer
- [Source: crates/kesh-api/src/routes/journal_entries.rs::list_journal_entries] — Handler à refactorer
- [Source: crates/kesh-core/src/types/money.rs] — `Money(Decimal)` pour le filtre montant
- [Source: CLAUDE.md#Review-Iteration-Rule] — 2-3 passes adversariales prévues

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-04-10: Création de la story 3.4 (Claude Opus 4.6, 1M context) — scope verrouillé sur recherche/tri/pagination de la liste des écritures. Décisions clés :
  - Recherche par numéro de facture **reportée en story 5.x** (le champ `invoice_number` n'existe pas dans `journal_entries` — introduction prématurée = scope creep).
  - Enums `SortBy` + `SortDirection` dans `kesh-core/listing/` (réutilisables par futures listes contacts/factures).
  - Pattern `sqlx::QueryBuilder::<MySql>` retenu pour les WHERE dynamiques (vs helper maison).
  - Envelope `ListResponse<T>` dans `kesh-api/routes/mod.rs` — réutilisable, pas de module partagé dédié (YAGNI, extraction post-MVP).
  - Anti-SQL injection sur `ORDER BY` via whitelist stricte d'enum + littéraux `&'static str` (SQLx ne supporte pas les bind params pour `ORDER BY`).
  - Échappement `%` et `_` dans LIKE avec `ESCAPE '\\'`.
  - Filtre `amount_min/max` via sous-requête `HAVING SUM(debit) BETWEEN` — performance acceptable v0.1 (< 10k entries), matérialisation post-MVP si besoin.
  - Sync URL via SvelteKit `goto(url, { replaceState: true })` — partage/rafraîchissement/retour arrière.
  - Debounce 300 ms sur inputs text frontend (helper maison, pas de dépendance externe).
  - Aucune nouvelle migration DB, pas de nouveau audit, pas de transactions complexes — story significativement plus simple que 3.2/3.3.
  - Dette T9.3 (framework TestClient HTTP) et A11y (focus trap) héritées.
- 2026-04-10: Revue adversariale passe 1 (Explore subagent, Sonnet 4.6, contexte vierge — LLM orthogonal à Opus auteur) — 2 CRITICAL, 3 HIGH, 4 MEDIUM, 3 LOW. Les 9 findings > LOW tous patchés :
  - **C1 (CRITICAL)** : T3.3 « ATTENTION dashboard homepage » créait un faux risque → vérifié empiriquement (`grep -rn list_recent_by_company`), pas d'appel hors handler + 1 test. Instruction ferme de suppression + suppression du test orphelin.
  - **C2 (CRITICAL)** : AC#10 affirmait « 400 VALIDATION_ERROR » sans documenter le mapping `QueryRejection → AppError`. Ajout d'une sous-tâche T3.3bis qui tranche explicitement : **Option A** (comportement par défaut Axum, corps texte, HTTP 400 OK, simplicité v0.1) retenue. Option B (extractor custom) documentée comme refactor post-MVP si besoin.
  - **H1 (HIGH)** : T2.3 `push_where_clauses` signature ambiguë sur un ou deux `QueryBuilder`. Réécrit avec exemple de code complet montrant **deux instances distinctes** (count + items) et le warning CRITIQUE que `QueryBuilder` ne peut pas être réutilisé.
  - **H2 (HIGH)** : T2.2 ne documentait pas le type exact de `journal_entry_lines.debit`. Schéma confirmé `DECIMAL(19,4)` (migration story 3.2). Valeur safe `Decimal::from_str("999999999999999.9999")` documentée pour `amount_max` absent, avec warning anti-`Decimal::MAX`.
  - **H3 (HIGH)** : Contradiction « clamp lenient » (AC#11) vs « clamp défensif repo » (T2.2). Tranché : **source de vérité = route handler**, le clamp repo est un garde-fou défensif silencieux (pas d'erreur remontée).
  - **M1 (MEDIUM)** : Gotcha Svelte 5 `$effect` + `goto` — 3 solutions suggérées sans trancher. Tranché sur `untrack(() => goto(...))` (idiome officiel Svelte 5), avec exemple de code complet. Rejet explicite des alternatives `$effect.root` et flag `isUpdatingUrl`.
  - **M2 (MEDIUM)** : Transition breaking `fetchJournalEntries` tableau → envelope sous-documentée. T5.1 réécrit avec snippet AVANT/APRÈS concret et liste explicite des nouveaux state vars (`total`, `offset`, `limit`).
  - **M3 (MEDIUM)** : Clés i18n des column labels potentiellement manquantes. Vérifié empiriquement (`grep journal-entries-col`) : **toutes déjà présentes depuis story 3.2**. T6.1 mis à jour avec note explicite « Ne PAS recréer ».
  - **M4 (MEDIUM)** : Import `i18nMsg` path ambigu. Clarifié : chemin `$lib/features/onboarding/onboarding.svelte` **sans `.ts`** (c'est un fichier `.svelte.ts` mais l'import SvelteKit n'inclut pas l'extension).
  - **LOW (3)** : (a) Méthode `as_sql()` renommée `as_sql_keyword()` pour cohérence avec `as_sql_column()`. (b) Pattern Rust literal exact `"\\\\"` pour `ESCAPE` documenté. (c) AC#1 ajouté secondary sort `entry_number DESC` explicite pour stabilité de tri.
- 2026-04-10: Revue adversariale passe 2 (Explore subagent, Haiku 4.5, contexte vierge — LLM orthogonal à Sonnet passe 1). Vérification : **les 9 patches passe 1 sont tous présents et cohérents** dans le markdown.
  - 1 finding MEDIUM (camelCase/PascalCase mix dans le DTO query) + 2 LOW (cleanup debounce, collation LIKE).
  - **Reclassement MEDIUM → LOW après analyse** : le mix camelCase (noms de champs) / PascalCase (valeurs d'enum imbriqués) est le **comportement Rust/serde standard** — `rename_all` ne propage pas aux variants d'enum. **Cohérent avec le pattern du projet** : l'enum `Journal` story 3.2 utilise déjà des variants PascalCase (`"Achats"`, `"Ventes"`). Uniformiser en mettant `rename_all = "camelCase"` sur les enums créerait une divergence avec le pattern existant. **Décision intentionnelle documentée** dans T1.1 avec exemple de requête HTTP concrète et note « Ne PAS tenter d'uniformiser ».
  - **2 LOW patchés par rigueur** : (a) T4.4 `debounce` étendu avec interface `DebouncedFn<Args>` exposant `.cancel()` + exemple concret de cleanup `$effect` dans T5.1 ; (b) AC#2 clarifié : collation `utf8mb4_unicode_ci` héritée de la table (pas de clause `COLLATE` explicite dans la requête).
- 2026-04-10: **Critère d'arrêt CLAUDE.md formellement ATTEINT** après 2 passes orthogonales (Sonnet → Haiku). 12 patches au total (9 passe 1 + 3 passe 2 dont 1 reclassé). 0 finding > LOW résiduel (le finding MEDIUM passe 2 reclassé en LOW après analyse empirique du pattern Rust/serde standard déjà utilisé dans `Journal` story 3.2). Story 3.4 **prête pour `dev-story`**.
- 2026-04-10: **Implémentation complète (dev-story, Claude Opus 4.6, 1M context)**. Toutes les tâches T1-T6 exécutées. **81/81 tests** (28 backend + 53 Vitest), 0 régression.
- 2026-04-10: **Code review adversarial — Passe 1** (Sonnet+Sonnet+Haiku, contexte vierge). Verdict : **BLOCK** — 9 findings appliqués + 6 faux positifs Haiku rejetés (Haiku Auditor avait cru le repository/handler/types absents — confusion par le diff combiné 3.2+3.3+3.4).
  - **P1 HIGH** : `$effect` sync URL — `new URL(page.url)` lu hors `untrack` → boucle réactive. Fix : enveloppement complet dans `untrack(...)`.
  - **P2 HIGH** : Cross-validation `amount_min > amount_max` et `date_from > date_to` manquante → 0 résultats silencieux. Fix : rejet `400 VALIDATION_ERROR`.
  - **P3 HIGH** : Pagination sans guard `loading` → race fetch concurrent. Fix : `if (loading) return;` sur `onPrevPage`/`onNextPage`/`onPageSizeChange`.
  - **P4 MEDIUM** : `serializeQuery` n'omettait pas `sortBy=EntryDate`/`sortDir=Desc` (défauts). Fix : conditions `!== 'EntryDate'`/`!== 'Desc'` + 3 nouveaux tests Vitest.
  - **P5 MEDIUM** : Écrasement `offset`/`limit` côté client depuis la réponse serveur. Fix : ne plus écraser, client = source de vérité. **Trade-off accepté** : si serveur clamp silencieusement, l'UI affichera une pagination potentiellement trompeuse — amélioration UX `wasClipped` post-MVP.
  - **P6 MEDIUM** : `amount_min/max` négatifs acceptés silencieusement. Fix : validation `>= 0` dans le handler.
  - **P7 LOW** : Imports `ArrowDown`/`ArrowUp` inutilisés. Fix : retrait.
  - **P8 LOW** : "Chargement…" hardcodé. Fix : nouvelle clé i18n `journal-entries-loading` × 4 langues.
  - **P9 LOW** : Commentaire `decimal_max_safe` trompeur (calcul ~1e20 hors capacité DECIMAL(19,4)). Fix : `Decimal::from_str("999999999999999.9999")` aligné exactement avec DECIMAL(19,4) + commentaire correct.
  - **Régression test corrigée** : test `serializeQuery > sérialise les champs non vides` mis à jour pour utiliser `sortBy: 'EntryNumber'`/`sortDir: 'Asc'` (les défauts sont désormais omis post-P4).
  - Compilation clean post-patches : `cargo check --workspace` OK, **56/56 tests Vitest** (3 nouveaux P4), 28/28 tests backend, svelte-check 0 erreur.
- 2026-04-10: **Code review adversarial — Passe 2** (Haiku 4.5, LLM orthogonal à Sonnet passe 1) sur le diff des patches uniquement (1002 lignes — pas de mélange story 3.2/3.3 cette fois, briefing explicite ajouté pour éviter les faux positifs). Verdict : **APPROVE clean**. Vérification des 9 patches : tous OK, aucune régression. 1 MEDIUM + 2 LOW résiduels reclassés en LOW (trade-offs intentionnels documentés).
- 2026-04-10: **Critère d'arrêt CLAUDE.md formellement ATTEINT** après 2 passes orthogonales post-dev (Sonnet+Sonnet+Haiku → Haiku). Story 3.4 **marquée `done`**. Bilan final :
  - **84 tests** passent (28 backend + 56 Vitest), 0 régression
  - **9 patches code review post-dev** + **12 patches validation pré-dev** = **21 patches** au total
  - **2 passes pré-dev** + **2 passes post-dev** = **4 passes adversariales orthogonales**
  - Dette technique mineure : (a) UI buttons pagination non visuellement désactivés pendant `loading` (cosmétique, le guard JS suffit pour la safety) ; (b) flag `wasClipped` dans la réponse serveur si clamp limit (post-MVP) ; (c) renommage cosmétique du test `serializeQuery > sérialise les champs non vides`.
