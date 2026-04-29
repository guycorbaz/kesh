# Code Review — Story 3.7 « Gestion des exercices comptables » — Pass 1

**Date** : 2026-04-28
**Reviewers** : Blind Hunter (Sonnet), Edge Case Hunter (Sonnet), Acceptance Auditor (Sonnet) — fenêtres fraîches, parallèles
**Source du diff** : working tree (changements non committés, 23 fichiers, ~3988 lignes)
**Mode** : `full` (avec spec)
**Spec** : `_bmad-output/implementation-artifacts/3-7-gestion-exercices-comptables.md` (23 ACs)

## Synthèse

| Sévérité | Brut total | Après dédup/reject |
|----------|-----------:|-------------------:|
| CRITICAL | 1          | 1                  |
| HIGH     | 7          | 3                  |
| MEDIUM   | 17         | 8                  |
| LOW      | 9          | 8                  |
| **Total > LOW** | **25** | **12**         |

**Findings rejetés** (false positives ou bruits) : 9. Détail en fin de document.

Per CLAUDE.md « Règle de remédiation des revues » : 12 findings > LOW → **une nouvelle passe est requise** après application des patches (LLM différent, fenêtre fraîche, max 8 passes).

---

## CRITICAL

### F1 — `create_if_absent_in_tx` : pas de catch sur `UniqueConstraintViolation` sous finalize concurrent

- **Sources** : Edge Case Hunter C-1
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:195-237` (fonction `create_if_absent_in_tx`)
- **Catégorie** : patch
- **Détail** : Sous MariaDB REPEATABLE READ, deux `POST /onboarding/finalize` concurrents pour la même company peuvent tous les deux passer le `WHERE NOT EXISTS` (chaque tx voit le snapshot pré-INSERT). Le premier commit, le second frappe `uq_fiscal_years_company_name` ou `uq_fiscal_years_company_start_date` → `DbError::UniqueConstraintViolation` qui remonte en HTTP 500 au lieu d'être idempotent (`Ok(None)` attendu par contrat de la fonction).
- **Impact** : Deuxième client de finalize concurrent reçoit 500 au lieu d'un succès silencieux. Brise l'invariant « idempotent » documenté dans le code (`if rows_affected == 0 → Ok(None)`).
- **Patch** : Catcher `Err(DbError::UniqueConstraintViolation(_)) => Ok(None)` après l'INSERT ou faire un `SELECT FOR UPDATE` préalable sur la company row pour sérialiser les finalize concurrents.

---

## HIGH

### F2 — `update_name` et `close` repo : pas de scoping `company_id` dans le SQL

- **Sources** : Blind Hunter H-1+H-2 / Edge Case Hunter H-1+H-2 / Acceptance Auditor « Décision conception »
- **Location** :
  - `crates/kesh-db/src/repositories/fiscal_years.rs:261-265` (update_name SELECT FOR UPDATE)
  - `crates/kesh-db/src/repositories/fiscal_years.rs:495-504` (close UPDATE)
- **Catégorie** : patch
- **Détail** : Les deux fonctions mutatrices repo prennent uniquement `id` (pas `company_id`) et leurs requêtes SQL ne filtrent pas sur `company_id`. La défense multi-tenant est mono-couche (uniquement le handler via `find_by_id_in_company`). Si une de ces fonctions est appelée depuis un autre code path (test interne, futur caller, refactor qui retire le pre-check du handler), elle muterait silencieusement une row cross-tenant.
  - `update_name` : le pre-check duplicate (l. 281-291) utilise `before.company_id` lu *depuis la row trouvée*, donc la row source détermine elle-même le scope — un id cross-tenant serait pleinement renommé.
  - `close` : `WHERE id = ? AND status = 'Open'` accepte n'importe quel id, indépendamment de `company_id`.
- **Impact** : Régression défense en profondeur par rapport au pattern Story 6-2 / Story 7-1. La spec T1.5 mentionne explicitement une variante `find_by_id_in_company_locked` qui n'est pas implémentée.
- **Patch** : Ajouter `company_id` dans la signature des deux fonctions et dans toutes leurs requêtes SQL (`AND company_id = ?` partout). Mettre à jour les call sites (handlers + tests).

### F3 — Nom > 50 caractères → erreur 500 non mappée (côté backend ET frontend)

- **Sources** : Edge Case Hunter H-3, Edge L-2
- **Location** :
  - DB : `fiscal_years.name VARCHAR(50)` (migration `20260404000001_initial_schema.sql:42`)
  - Backend : `crates/kesh-api/src/routes/fiscal_years.rs:171-196` (create_fiscal_year handler) + `update_fiscal_year`
  - Frontend : `frontend/src/lib/features/fiscal-years/fiscal-years.helpers.ts:13-21` (`validateFiscalYearForm`)
- **Catégorie** : patch
- **Détail** : Aucune validation de longueur côté handler ni côté `validateFiscalYearForm`. MariaDB rejette avec « Data too long for column 'name' » → `DbError::Sqlx(DatabaseError(...))` non spécialement mappé → HTTP 500 au lieu de 400 / `VALIDATION_ERROR`.
- **Impact** : UX dégradée (500 générique). Pas de fuite d'information mais comportement non conforme aux ACs #2/#5 qui demandent une validation client + backend cohérente.
- **Patch** :
  1. Ajouter `if input.name.length > 50 return 'error-fiscal-year-name-too-long'` dans `validateFiscalYearForm`.
  2. Ajouter une validation handler-side (ou clé Invariant `FY_NAME_TOO_LONG_KEY`) avec mapping vers `AppError::Validation(t("error-fiscal-year-name-too-long"))`.
  3. Ajouter clé i18n `error-fiscal-year-name-too-long` × 4 locales.

### F4 — `find_overlapping` : ordre des binds inversé par rapport aux paramètres de la fn

- **Sources** : Blind Hunter H-4, Acceptance Auditor (Décision conception, info)
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:425-443`
- **Catégorie** : patch (cosmétique mais sur invariant métier critique)
- **Détail** : SQL `WHERE start_date <= ? AND end_date >= ?` avec `.bind(end_date).bind(start_date)`. Le résultat est mathématiquement correct (chevauchement d'intervalles fermés `[s, e]` vs `[s', e']` → `s <= e' AND e >= s'`), mais l'ordre des `.bind()` est contre-intuitif vs l'ordre des paramètres de la fonction (`start_date, end_date`). Un futur refactor qui réordonnerait les binds pour « cohérence » introduirait silencieusement un bug : `s <= s' AND e >= e'` ne détecte plus l'overlap.
- **Impact** : Piège de maintenance future sur un invariant métier critique (les exercices ne doivent pas chevaucher).
- **Patch** : Renommer les placeholders dans la requête pour clarifier l'intent — par exemple utiliser des variables nommées via `format!` ou ajouter un commentaire explicite au-dessus du `.bind()` :
  ```rust
  // Algèbre overlap : E.start <= N.end AND E.end >= N.start
  .bind(end_date)   // → start_date <= end_date_param
  .bind(start_date) // → end_date >= start_date_param
  ```
  Alternative : aligner l'ordre `.bind()` sur les paramètres en réécrivant le SQL : `WHERE end_date >= ? AND start_date <= ?` avec `.bind(start_date).bind(end_date)`.

---

## MEDIUM

### F5 — `notifyMissingFiscalYearOrFallback` affiche un message faux pour `FISCAL_YEAR_CLOSED`

- **Sources** : Blind Hunter M-2
- **Location** : `frontend/src/lib/shared/utils/notify.ts:67-102`
- **Catégorie** : patch
- **Détail** : Le helper intercepte `FISCAL_YEAR_INVALID`, `NO_FISCAL_YEAR` ET `FISCAL_YEAR_CLOSED` avec le même message « Créez d'abord un exercice comptable ». Or `FISCAL_YEAR_CLOSED` signifie qu'un exercice **existe** mais est clôturé — l'utilisateur ne doit PAS en créer un nouveau, il doit changer la date de sa saisie. Le message actuel peut induire à créer un doublon d'exercice.
- **Impact** : UX trompeur. AC #22 demande un toast actionnable correct, le message « Créez » est sémantiquement incorrect pour le cas Closed.
- **Patch** : Différencier le message :
  - `FISCAL_YEAR_INVALID` / `NO_FISCAL_YEAR` → « Créez d'abord un exercice comptable » + bouton Paramètres.
  - `FISCAL_YEAR_CLOSED` → « L'exercice qui couvre cette date est clôturé. Vérifiez la date ou consultez vos exercices. » + bouton Paramètres (lecture).
  - Ajouter clé i18n `error-fiscal-year-closed-action` × 4 locales.

### F6 — Clé i18n `fiscal-year-close-confirmation-body` : pas d'interpolation `{name}`

- **Sources** : Acceptance Auditor (AC #9 + AC #19)
- **Location** : `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` + `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`
- **Catégorie** : patch
- **Détail** : AC #9 spécifie « Vous êtes sur le point de clôturer l'exercice {name}. Cette action est irréversible : … ». La clé `.ftl` actuelle dit seulement « Cette action est irréversible : … » sans phrase d'ouverture ni interpolation `{name}`. Le fallback hardcodé Svelte inclut le nom, mais en production avec i18n actif, l'utilisateur voit la version sans nom.
- **Impact** : L'utilisateur ne sait pas quel exercice il clôture (action irréversible). Risque d'erreur opérationnelle sur un projet comptable.
- **Patch** :
  1. Ajouter variable Fluent `{ $name }` dans la clé `.ftl` × 4 locales : `fiscal-year-close-confirmation-body = Vous êtes sur le point de clôturer l'exercice { $name }. Cette action est irréversible : aucune écriture, facture ou paiement ne pourra plus être enregistré sur cette période. Confirmer ?`
  2. Adapter l'appel Svelte pour passer `{ name: closeTarget.name }` à `i18nMsg`.

### F7 — Tests Playwright AC #22 (T8.7/T8.8) sont des annotations vides

- **Sources** : Acceptance Auditor (AC #21), Blind Hunter L-4
- **Location** : `frontend/tests/e2e/fiscal-years.spec.ts:89-142`
- **Catégorie** : patch
- **Détail** : Les deux tests `'création de journal_entry hors fiscal_year ouvre le toast actionnable'` et `'validation facture hors fiscal_year ouvre le toast actionnable'` n'ont aucun `expect()`. Ils se contentent de `test.info().annotations.push({...})` qui est un commentaire déguisé en test. Ils passent toujours, ne vérifient rien.
- **Impact** : AC #22 est un AC fonctionnel critique (fallback UX pour utilisateurs sans exercice). Aucune protection régression Playwright.
- **Patch** : Implémenter les vraies assertions :
  - Modifier les fixtures pour préparer un état sans fiscal_year couvrant la date.
  - Soumettre l'action.
  - `await expect(page.getByRole('alert')).toContainText(/Créez d'abord/);`
  - `await page.getByRole('button', { name: /Paramètres/ }).click();`
  - `await expect(page).toHaveURL(/\/settings\/fiscal-years/);`

### F8 — `update_fiscal_year` handler ne trim pas `req.name`

- **Sources** : Edge Case Hunter M-1
- **Location** : `crates/kesh-api/src/routes/fiscal_years.rs:215` (et `update_name` repo l. 287)
- **Catégorie** : patch
- **Détail** : `create_fiscal_year` fait `req.name.trim().to_string()`. `update_fiscal_year` passe `req.name` brut à `update_name`. Le check `if new_name.trim().is_empty()` rejette les chaînes 100 % whitespace, mais `"  FY 2027  "` passe et est stocké tel quel. La UNIQUE constraint sous `utf8mb4_unicode_ci` traite `" FY 2027 "` comme distinct de `"FY 2027"` → quasi-doublons possibles.
- **Impact** : Incohérence create vs update. Quasi-doublons possibles.
- **Patch** : Trimer dans `update_fiscal_year` handler avant l'appel : `fiscal_years::update_name(&state.pool, current_user.user_id, id, req.name.trim().to_string()).await?` OU centraliser le trim dans `update_name` repo.

### F9 — AC #14 : test E2E `demo_path_creates_fiscal_year` manquant

- **Sources** : Acceptance Auditor (AC #14)
- **Location** : `crates/kesh-api/tests/fiscal_years_e2e.rs`
- **Catégorie** : patch
- **Détail** : AC #14 exige un test E2E `fiscal_years_e2e::demo_path_creates_fiscal_year` qui valide que `seed_demo` crée bien un fiscal_year `Open` avec dates 1er jan-31 déc + nom `Exercice {YYYY}` + **pas d'audit log** (puisqu'on a migré vers `create_for_seed`). Ce test n'existe pas dans le fichier. Le test repo `test_create_for_seed_does_not_audit` couvre l'absence d'audit côté repo, mais pas la régression E2E du flux démo.
- **Impact** : Une régression future sur `seed_demo` (ex. migration accidentelle vers `create()` qui auditerait) ne serait pas détectée.
- **Patch** : Ajouter un test E2E qui appelle l'endpoint démo (ou directement `seed_demo`) et vérifie via `list_by_company` + `audit_log::find_by_entity` les invariants AC #14.

### F10 — Deadlock possible entre deux `create()` concurrents (locks `find_overlapping` + `find_by_name`)

- **Sources** : Edge Case Hunter M-2
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:107-138`
- **Catégorie** : patch
- **Détail** : Tx T1 acquiert FOR UPDATE sur range `[Jan, Dec]` via `find_overlapping`, puis FOR UPDATE sur l'index name. Tx T2 fait l'inverse sur des dates/noms qui se croisent → cycle de locks possible selon les pages d'index InnoDB. MariaDB tue une des transactions (error 1213) → mappé `DbError::Sqlx(...)` → HTTP 500.
- **Impact** : Faible (création concurrente d'exercices est rare et user-driven), mais surface d'erreur 500 dégradée pour un cas pourtant bien défini.
- **Patch** : Soit catcher l'error 1213 dans `map_db_error` et retourner `DbError::Conflict` mappé en 409, soit ajouter un retry middleware (déjà tracé issue #43 pour v0.2). Pour Pass 1 : documenter explicitement le risque dans Pattern 5 et noter que le retry middleware d'issue #43 couvrira ce cas, OU défer en LOW.

### F11 — `closeTarget` non réinitialisé si `loadFiscalYears()` échoue après `ILLEGAL_STATE_TRANSITION`

- **Sources** : Edge Case Hunter M-4
- **Location** : `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` (handler `submitClose`)
- **Catégorie** : patch
- **Détail** : Dans la branche `ILLEGAL_STATE_TRANSITION`, `closeError = msg(...)` puis `closeOpen = false` puis `await loadFiscalYears()`. Si `loadFiscalYears()` lance une erreur (réseau), elle propage hors du `try/catch` de `submitClose`, laissant `closeSubmitting = true` (le `finally` ne tourne pas). Le bouton « Clôturer définitivement » reste bloqué.
- **Impact** : UX dégradé pour un cas réel (perte de connexion + race close).
- **Patch** : Wrap `loadFiscalYears()` dans son propre try/catch avec un toast d'erreur séparé, ou déplacer le reset `closeSubmitting = false` dans un `finally` qui englobe tous les await.

### F12 — Double affichage d'erreur dans submitCreate / submitRename (formulaire + toast)

- **Sources** : Blind Hunter M-6
- **Location** : `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` (lignes ~3521-3527 et ~3559-3565 du diff)
- **Catégorie** : patch
- **Détail** : Sur erreur de validation backend, le code fait à la fois `createError = err.message` (affiché inline via `<p role="alert">`) ET `notifyError(err.message)` (toast). L'utilisateur voit deux fois la même erreur simultanément. Anti-pattern UX vs convention Story 3.5.
- **Impact** : UX dégradé, bruit visuel.
- **Patch** : Choisir une seule surface — soit inline (préférable pour erreurs de validation contextuelles), soit toast (préférable pour erreurs réseau / inattendues). Conserver la branche fallback toast pour les erreurs hors validation.

---

## LOW

### F13 — AC #20 test `create_with_injected_company_id_ignored` : pas de vérification DB directe

- **Sources** : Acceptance Auditor (AC #20)
- **Location** : `crates/kesh-api/tests/fiscal_years_e2e.rs` (test homonyme)
- **Catégorie** : patch
- **Détail** : Le test vérifie `body["companyId"] == admin_company_id` dans la réponse HTTP, pas en DB. Spec demande `find_by_id(new_fy.id).company_id == 1` pour valider que le storage est bien isolé du payload.
- **Patch** : Ajouter un appel `find_by_id` post-201 et vérifier `company_id`.

### F14 — `find_covering_date` sans `ORDER BY` : non-déterministe si overlap historique

- **Sources** : Edge Case Hunter M-3 (downgrade LOW, contraintes UNIQUE protègent en pratique)
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:364-381`
- **Catégorie** : patch
- **Détail** : Si deux exercices couvrent la même date (injection DB directe ou bug historique), `LIMIT 1` sans `ORDER BY` retourne arbitrairement. Les UNIQUE constraints DB empêchent ce cas en pratique, mais ajouter `ORDER BY status DESC, start_date DESC` (priorise Open puis le plus récent) est une défense en profondeur cheap.
- **Patch** : Ajouter `ORDER BY status DESC, start_date DESC` dans la requête.

### F15 — `Utc::now()` dans `finalize` : décalage timezone Europe/Zurich (1h/an)

- **Sources** : Blind Hunter M-4 + Edge Case Hunter L-1
- **Location** : `crates/kesh-api/src/routes/onboarding.rs:638` (et `frontend/src/lib/features/fiscal-years/fiscal-years.helpers.ts` `currentYearDefaults`)
- **Catégorie** : defer
- **Détail** : Un finalize entre 00h-01h heure suisse le 1er janvier (UTC = 31 déc YYYY-1) crée « Exercice YYYY-1 » au lieu de « Exercice YYYY ». Cosmétique (renommable) et fenêtre 1h/an. Mais c'est tout de même surprenant pour un logiciel CH.
- **Recommandation** : v0.2, bascule sur la timezone de la company (entité `companies` n'a pas de colonne tz aujourd'hui — décision séparée).

### F16 — E2E test isolation : pas de teardown DB après tests qui créent des fiscal_years

- **Sources** : Edge Case Hunter M-5
- **Location** : `frontend/tests/e2e/fiscal-years.spec.ts:20-22, 53-86`
- **Catégorie** : patch
- **Détail** : `afterEach` clear seulement le storage auth, pas la DB. Le test « crée Exercice 2031 E2E » fait `seedTestState('with-company')` au début pour reset, ce qui mitige. Mais en CI parallèle ou si le seed est partagé, les tests pourraient interférer.
- **Patch** : Ajouter `seedTestState('with-company')` en `beforeEach` (pas seulement `beforeAll`) ou créer un fixture isolé.

### F17 — `$app/navigation` import dans `shared/notify.ts` : couplage SvelteKit dans un module utility

- **Sources** : Blind Hunter L-5
- **Location** : `frontend/src/lib/shared/utils/notify.ts:15`
- **Catégorie** : patch
- **Détail** : `$app/navigation` est SvelteKit-runtime-only. Son import dans un module `shared` rend le module non-testable en Vitest pur sans setup SvelteKit, et peut causer des erreurs SSR si le module est importé côté serveur.
- **Patch** : Injecter `goto` en paramètre du helper (`notifyMissingFiscalYearOrFallback(err, navigate?)`) OU déplacer le helper hors de `shared/` (vers `lib/features/fiscal-years/` ou un module dédié à la navigation-aware notification).

### F18 — Test `path_b_finalize_idempotent_with_existing_fiscal_year` : assertion vacuus sur audit log

- **Sources** : Blind Hunter L-3
- **Location** : `crates/kesh-api/tests/fiscal_years_e2e.rs` (test homonyme)
- **Catégorie** : patch
- **Détail** : Le test pré-insère via `create_for_seed` (qui n'audit pas) puis vérifie `!entries.iter().any(|e| e.action == "fiscal_year.created")`. Comme `create_for_seed` n'écrit pas d'audit, l'assertion est triviale même si `finalize` créait un audit en doublon. Pour vraiment tester l'idempotence audit, il faut pré-insérer via `create()` (qui audit), puis vérifier qu'il n'y a qu'**une** entrée audit, pas deux.
- **Patch** : Soit pré-insérer via `create(pool, user_id, ...)` et asserter `entries.len() == 1`, soit passer par finalize une première fois puis tester l'idempotence du second.

### F19 — `LIMIT 1 FOR UPDATE` vs `FOR UPDATE LIMIT 1` : ordre incohérent entre fonctions

- **Sources** : Blind Hunter L-1
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:431-435` (find_overlapping) vs `:454-458` (find_by_name)
- **Catégorie** : patch
- **Détail** : `find_overlapping` utilise `LIMIT 1 FOR UPDATE`, `find_by_name` utilise `FOR UPDATE LIMIT 1` (vérifier — en réalité `find_by_name` utilise `LIMIT 1 FOR UPDATE` aussi selon la lecture du code). MariaDB accepte les deux formes. Cohérence stylistique à harmoniser.
- **Patch** : Choisir une forme et l'appliquer partout. La forme canonique recommandée par MariaDB est `... FOR UPDATE` en fin (LIMIT placé avant).

### F20 — `find_overlapping` `LIMIT 1` : suffisant mais pas blocking pour `find_open_covering_date`

- **Sources** : Auditor (Décision conception, info)
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:425`
- **Catégorie** : reject (covered par F4 + F19)
- **Détail** : Doublon de F4 (ordre des binds) + F19 (ordre LIMIT/FOR UPDATE). Dropper.

---

## Findings rejetés (false positives ou bruit)

| # | Source | Raison du reject |
|---|--------|------------------|
| R1 | Blind M-1 (`update_name` UPDATE missing `updated_at = NOW()`) | **False positive**. La colonne `fiscal_years.updated_at` a `ON UPDATE CURRENT_TIMESTAMP(3)` (migration `20260404000001` ligne 47) → MariaDB met à jour automatiquement sur tout UPDATE qui change une autre colonne. |
| R2 | Blind M-3 (status='Open' hardcodé incohérent avec DEFAULT) | Le DEFAULT DB est aussi `'Open'`, pas de divergence aujourd'hui. Si DEFAULT change un jour, c'est une décision schema-wide. Nit. |
| R3 | Blind M-5 (validateFiscalYearForm Date timezone) | **False positive**. `new Date('YYYY-MM-DD')` est toujours parsé comme UTC midnight en JS. La comparaison `end <= start` est sûre. |
| R4 | Blind M-7 (TOCTOU close handler) | Doublon de F2 (cluster on company_id missing in repo close). |
| R5 | Blind H-3 (create_if_absent_in_tx fragile post-INSERT lookup) | Paranoïa théorique sur `last_insert_id` non-utilisé. Le re-lookup via `(company_id, name)` avec UNIQUE constraint est robuste. Pas de bug. |
| R6 | Blind L-2 (FiscalYearStatus serialization not pinned by test) | E2E tests asserting `body["status"] == "Open"` / `"Closed"` pinnent indirectement le format. |
| R7 | Auditor AC #6 (dd vs disabled inputs cosmétique) | Spec ambiguë (« visibles mais non éditables » est satisfait par texte statique). Functionnellement OK. |
| R8 | Edge L-3 (go-to-settings i18n key non-namespaced) | Lint i18n-ownership PASS aujourd'hui. À surveiller v0.2 mais pas un finding actionable maintenant. |
| R9 | F20 (LIMIT 1 dans find_overlapping) | Doublon de F4 + F19. |

---

## Recommandation suite

Per CLAUDE.md « Règle de remédiation » :
1. **Appliquer les patches** sur F1-F19 (1 CRITICAL, 3 HIGH, 8 MEDIUM, 7 LOW patchables).
2. **F15 → defer** (timezone v0.2).
3. **Lancer Pass 2** avec un LLM différent (cycle Opus → Sonnet → Haiku → Opus : Pass 1 = Sonnet → Pass 2 = Haiku ou Opus).
4. **Fenêtre fraîche** pour Pass 2.
5. **Boucle jusqu'à 0 finding > LOW OU 8 passes**.

## Reviewers

| Layer | Modèle | Findings bruts | Findings retenus après dédup |
|-------|--------|----------------|------------------------------|
| Blind Hunter | Sonnet | 16 | 11 (5 reject) |
| Edge Case Hunter | Sonnet | 12 | 11 (1 reject doublon) |
| Acceptance Auditor | Sonnet | 6 | 5 (1 reject cosmétique) |

Total brut : 34. Total après triage : 20 (12 > LOW). Reject : 9 + 5 doublons fusionnés = 14.

---

**Pass 1 status** : 12 findings > LOW → boucle requise par CLAUDE.md.
