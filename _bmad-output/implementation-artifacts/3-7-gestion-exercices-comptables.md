# Story 3.7: Gestion des exercices comptables

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **utilisateur (Admin ou Comptable)**,
I want **créer, lister, renommer et clôturer mes exercices comptables depuis l'interface**,
so that **je puisse valider mes factures et saisir des écritures sans passer par SQL direct, et préparer la clôture annuelle (FR60) en conformité avec le CO art. 957-964**.

### Contexte

**Story bloquante post-Epic-6** ré-ouvrant l'Epic 3. Trois fonctions critiques exigent un `fiscal_year` Open :

- `journal_entries::create` (Story 3.2) — refuse une écriture si la date n'est pas dans un exercice ouvert.
- `validate_invoice` (Story 5.2) — `find_open_covering_date()` lock + check dans la transaction de validation.
- `mark_as_paid` (Story 5.4) — pareil pour la date de paiement.

**Aujourd'hui** : aucune UI ni API ne permet à l'utilisateur de créer un exercice. Path A (démo) est sauvé par `kesh_seed::seed_demo` (lignes 145-159) qui crée transparente­ment un exercice pour l'année calendaire courante. **Path B (production) reste bloqué** — un utilisateur qui finalise l'onboarding production ne peut **pas** valider de facture sans un `INSERT INTO fiscal_years` SQL direct. Cette story débloque Path B et prépare Epic 8 (Import bancaire, qui réconcilie avec des factures validées).

**Pourquoi Epic 3 est ré-ouverte** : à la rétro Epic 3 cette story a été reportée pour shipper plus vite ; sa nécessité est devenue évidente quand Epic 5 a livré la facturation. Sprint-status passe `epic-3: done → in-progress` à la création de cette story.

### Scope verrouillé — ce qui RESTE à faire

1. **Couche API** — 5 nouveaux endpoints dans un nouveau module `crates/kesh-api/src/routes/fiscal_years.rs` :
   - `GET /api/v1/fiscal-years` (authenticated, liste scopée company)
   - `GET /api/v1/fiscal-years/{id}` (authenticated, lecture détaillée)
   - `POST /api/v1/fiscal-years` (comptable+, création)
   - `PUT /api/v1/fiscal-years/{id}` (comptable+, rename only)
   - `POST /api/v1/fiscal-years/{id}/close` (comptable+, transition Open→Closed)
2. **Audit log** — ajouter `audit_log::insert_in_tx` dans `fiscal_years::create`, `update_name`, `close` (pattern story 3.5). Le repo actuel ne le fait pas encore (cf. ligne *"Audit log integration: NOT YET implemented"* du recherche). Refactor des fn pour accepter `user_id`.
3. **Repo extension** — nouvelles fonctions :
   - `fiscal_years::update_name(pool: &MySqlPool, user_id: i64, id: i64, new_name: String) -> Result<FiscalYear, DbError>` — pas d'optimistic version (cf. Décisions). Validation : `new_name.trim() != ""` retourne `DbError::Invariant`. Audit log via wrapper `{before, after}`. Lock `SELECT ... FOR UPDATE` sur la row avant l'UPDATE pour figer le before-snapshot (Pattern 5 — cf. Décisions et T1.7).
   - `fiscal_years::find_by_id_in_company(pool, company_id, id) -> Result<Option<FiscalYear>>` — query `WHERE company_id = ? AND id = ?` directement (Pass 1 H-3 fix : remplace le pattern fetch-then-check qui est Anti-Pattern 4 du doc multi-tenant).
   - `fiscal_years::create_if_absent_in_tx(tx, user_id, new) -> Result<Option<FiscalYear>>` — insert atomique avec `WHERE NOT EXISTS` (Pass 1 H-4 fix). Retourne `Some(fy)` si créé, `None` si déjà existant. Inclut audit_log si créé.
   - Refactor `fiscal_years::create(pool, user_id, new)` — accepte user_id, audit log via snapshot direct. Avant l'INSERT : pré-check overlap via `find_overlapping(tx, company_id, start, end)` ET pré-check name unicité via `find_by_name(tx, company_id, name)` pour distinguer les deux UNIQUE constraints existantes (Pass 1 H-5 + H-6).
   - `fiscal_years::find_overlapping(tx, company_id, start_date, end_date) -> Result<Option<FiscalYear>>` — `SELECT ... WHERE company_id = ? AND start_date <= ? AND end_date >= ? FOR UPDATE LIMIT 1` (overlap d'intervalles fermés, Pass 1 H-6).
   - `fiscal_years::find_by_name(tx, company_id, name) -> Result<Option<FiscalYear>>` — pour distinguer le cas "nom dupliqué" du cas "overlap" avant l'INSERT.
   - `fiscal_years::create_for_seed(pool, new) -> Result<FiscalYear>` — variante non-auditée pour `kesh_seed::seed_demo` (contexte système, pas utilisateur).
4. **Frontend feature lib** — `frontend/src/lib/features/fiscal-years/` avec `fiscal-years.api.ts`, `fiscal-years.types.ts`, `fiscal-year-helpers.ts` (validation date order côté client, helpers de format).
5. **UI page Admin/Comptable** — `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` :
   - Liste triée `start_date DESC` avec colonnes name, start_date, end_date, status, actions (rename, close).
   - Bouton « Nouvel exercice » → modale formulaire (nom, dates).
   - Bouton « Clôturer » avec confirmation.
   - Lien dans la home settings (`+page.svelte` racine de settings).
6. **i18n** — ~17 clés × 4 locales (FR/DE/IT/EN) sous le préfixe `fiscal-year-*` et `error-fiscal-year-*`.
7. **Onboarding Path B — auto-création par défaut + opt-out** — étendre `routes/onboarding.rs::finalize` pour créer un fiscal_year après le `insert_with_defaults_in_tx` avec un nom = année courante. Une `LegacyImportRequest` (futur, non en scope ici) pourrait skip — pour l'instant **toujours créer** si aucun exercice n'existe pour la company. Path A inchangé (déjà couvert par seed_demo).
8. **Tests E2E** — `crates/kesh-api/tests/fiscal_years_e2e.rs` : create / list / close / update + tests d'intégration avec `validate_invoice` et avec l'onboarding finalize.
9. **Tests Playwright** — scenario `fiscal-years.spec.ts` : créer un exercice, valider une facture, vérifier que ça marche (réutilise la story 5-2 setup).

### Scope volontairement HORS story — décisions tranchées

- **FR61 — auto-report des soldes** : reporté Story 14-1 (Clôture & report). Cette story livre le bouton Close (transition d'état) mais **pas** l'écriture des A-nouveaux. Le bouton Close marque `status='Closed'` et c'est tout. Story 14-1 (epic 14) ajoutera le report effectif.
- **Couverture multi-exercice non-chronologique** : v0.1 mono-exercice ouvert visible à l'utilisateur, mais le schéma permet d'avoir plusieurs exercices (le contraint UNIQUE est `(company_id, start_date)`, pas `status`). On n'empêche pas l'utilisateur d'avoir plusieurs Open simultanés — le contrôle `find_open_covering_date` les distingue par date. Pas de validation supplémentaire applicative.
- **Page UI consultation audit log fiscal_year** : reportée post-MVP (cohérent avec la décision story 3.5 de ne pas livrer une page `/audit`).
- **Édition des dates** : interdite après création (seul `name` mutable). AC explicite. Si l'utilisateur veut changer les dates, il doit recréer (mais l'exercice initial n'est pas supprimable — mitigé par UNIQUE constraint sur start_date qui force un nom différent ou un déplacement de date qui violerait la constraint).
- **Renommage post-clôture** (Pass 1 M-3) : **autorisé**. Justification — le CO art. 957-964 protège l'intégrité des montants et des dates ; corriger un libellé descriptif (« Exercice 2027 » → « Exercice 2027 (clôturé) ») n'altère aucune donnée comptable. L'audit log trace la modification avec wrapper before/after. Test E2E AC #6 valide ce comportement.
- **Suppression** : aucune fonction `delete` au niveau DB ni API. Conforme CO art. 957-964 (10 ans de conservation). Si l'utilisateur crée un exercice par erreur, il devra contacter le support (post-v0.1 : feature spéciale ou correction directe DB par admin système).
- **Décision UX onboarding Path B** (Pass 1 L-3 — clôt le point laissé ouvert dans epics.md:859) : auto-création **transparente par défaut**, **pas opt-in**. Justification — l'utilisateur Path B finalise pour pouvoir saisir des écritures et valider des factures ; bloquer la finalize ou exiger une case à cocher supplémentaire pour un exercice qu'il devra de toute façon créer ajoute du friction sans bénéfice. L'auto-création d'« Exercice {YYYY courante} » est un choix sûr — il peut renommer ou clôturer ensuite.
- **Optimistic locking sur fiscal_years.update_name** : **non requis** v0.1. La table n'a pas de colonne `version` (à ajouter dans une migration future si besoin). Pour l'update_name la fenêtre de race est minuscule (deux admins qui renomment le même exercice en même temps — hautement improbable, et la dernière écriture gagne, sans corruption de données comptables). À ré-évaluer en v0.2 si retour utilisateur.
- **Notification email à la clôture** : hors scope v0.1 (pas d'infra mail). Toast + audit log suffisent.

### Décisions de conception

- **Routes structure** : nouveau fichier `crates/kesh-api/src/routes/fiscal_years.rs` (module séparé, mod register dans `routes/mod.rs`). Pattern identique à `accounts.rs` / `companies.rs`.
- **Mounting** : 2 endpoints en `authenticated_routes` (GET list + GET id), 3 en `comptable_routes` (POST create, PUT update, POST close). Cohérent avec AC : « Admin + Comptable ».
- **Réponse JSON** : structure `FiscalYearResponse { id, companyId, name, startDate, endDate, status, createdAt, updatedAt }` en camelCase via `#[serde(rename_all = "camelCase")]`. Status sérialisé `"Open" | "Closed"` (PascalCase, cohérent avec `FiscalYearStatus` enum existant).
- **Validation overlap** (Pass 1 H-5 + H-6) : pré-check applicatif **dans la même tx FOR UPDATE** au lieu de se fier uniquement aux contraintes DB. Deux raisons :
  - Le schéma a 2 UNIQUE constraints (`uq_fiscal_years_company_name` ET `uq_fiscal_years_company_start_date`) ; sans pré-check, le mapping `UniqueConstraintViolation` est ambigu.
  - epics.md:854 exige « non-chevauchement avec exercices existants » qui couvre le cas Jan-Dec 2027 + Jul 2027-Jun 2028 (start_date différents, intervalles chevauchants). Aucune contrainte DB ne couvre ce cas.
  - **Algorithme `create()`** :
    1. `tx.begin()`.
    2. `find_overlapping(tx, company_id, start, end) FOR UPDATE` → si Some, retourner `DbError::Invariant("overlap")` mappé → `AppError::Validation("error-fiscal-year-overlap")`.
    3. `find_by_name(tx, company_id, name) FOR UPDATE` → si Some, retourner `DbError::Invariant("name-duplicate")` mappé → `AppError::Validation("error-fiscal-year-name-duplicate")`.
    4. INSERT.
    5. Audit log.
    6. Commit.
  - Les contraintes DB restent un filet de sécurité — si une race extrême passe le pré-check (impossible sous FOR UPDATE), la DB rejette avec `UniqueConstraintViolation` mappé sur message générique `error-fiscal-year-conflict`.
- **Validation date order** : côté frontend ET côté DB (CHECK constraint `chk_fiscal_years_dates`). Le backend ne re-valide pas applicativement — il laisse le DB constraint parler via mapping `DbError::CheckConstraintViolation`.
- **Audit log entries** :
  - `fiscal_year.created` — `details_json` = snapshot complet de la row insérée (id, name, dates, status, company_id).
  - `fiscal_year.updated` — wrapper `{ "before": {...}, "after": {...} }` (cohérent avec story 3.5 « wrapper uniquement pour transitions à 2 états »).
  - `fiscal_year.closed` — snapshot direct de la row après transition (id, status="Closed", `updated_at` reflète automatiquement le moment de la clôture via `ON UPDATE CURRENT_TIMESTAMP(3)`). Pass 1 M-2 fix : ne pas référencer `closed_at` qui n'existe pas dans le schéma — `updated_at` suffit.
- **Onboarding finalize auto-create fiscal_year** (Pass 1 H-4 fix) : utilise `fiscal_years::create_if_absent_in_tx(&mut tx, current_user.user_id, new_fy)` — insert atomique avec sous-requête `WHERE NOT EXISTS` qui ferme le TOCTOU window. Pas de check-then-insert. Audit log inclus si rows_affected == 1. Cohérent avec la rigueur Pattern 5 établie en Story 7-1 P3-P6 (lock-or-atomic, jamais de check-puis-acte hors lock).
- **Lock ordering** (Pass 1 M-9 fix) : cette story **introduit DEUX nouveaux lock sites** dans `fiscal_years` :
  - `fiscal_years::update_name` — `SELECT FOR UPDATE` sur la row avant l'UPDATE (nécessaire pour figer le before-snapshot d'audit log).
  - `fiscal_years::create` — `SELECT FOR UPDATE` (overlap + name pré-checks dans la même tx).
  - `fiscal_years::create_if_absent_in_tx` — pas de FOR UPDATE applicatif, mais l'INSERT … WHERE NOT EXISTS gère l'atomicité.
  - **Lock order** : `fiscal_years` n'est jamais combiné avec d'autres tables dans une même tx (toutes les fns ouvrent leur propre tx interne). Pas de risk de cross-table deadlock. Le `find_open_covering_date` existant (Story 5.2) acquiert FOR UPDATE sur fiscal_years APRÈS son lock sur invoices — toujours cohérent (fiscal_years après invoices dans cette tx-là). **T1.7 (nouvelle tâche)** : ajouter une entrée pour fiscal_years dans Pattern 5 du doc `docs/MULTI-TENANT-SCOPING-PATTERNS.md`.
- **i18n keys naming** : préfixe `fiscal-year-*` pour les libellés UI, `error-fiscal-year-*` pour les codes d'erreur. Cohérent avec la convention existante (`account-*`, `contact-*`, etc.).
- **Notification toasts** : utilise les helpers `notifySuccess` / `notifyError` (story 3.5). Pas de `toast.*` direct.

## Acceptance Criteria (AC)

1. **Page liste** — Given un user Admin ou Comptable, When il accède à `/settings/fiscal-years`, Then il voit un tableau triée `start_date DESC` avec colonnes `name`, `start_date` (format ISO local FR/DE/IT/EN), `end_date`, `status` (badge Open vert / Closed gris), et 2 actions par ligne (`Renommer`, `Clôturer` — ce dernier n'apparaît que si `status=Open`).

2. **Création — formulaire** — Given le bouton « Nouvel exercice » sur la page liste, When clic, Then une modale s'ouvre avec champs : `name` (texte libre, requis, non vide après trim), `start_date` (datepicker, requis), `end_date` (datepicker, requis). Validation côté frontend : `end_date > start_date`, `name.trim() != ""`. Le formulaire pré-remplit `name = "Exercice {YYYY}"` et `start_date = 1er janvier de l'année courante` et `end_date = 31 décembre de l'année courante` (suggestion modifiable).

3. **Création — appel API + retour** — Given le formulaire valide soumis, When `POST /api/v1/fiscal-years` retourne `201 Created` + body `FiscalYearResponse`, Then la modale ferme, le tableau se rafraîchit (la nouvelle ligne apparaît en tête car `start_date DESC`), et un toast vert `notifySuccess('fiscal-year-created')` s'affiche.

4. **Création — erreur overlap** — Given un exercice existe déjà avec la même `start_date` pour cette company, When création tentée, Then `POST /api/v1/fiscal-years` retourne `400 Validation` avec `code='VALIDATION_ERROR'` et `message=t('error-fiscal-year-overlap')`. Le toast rouge s'affiche, la modale reste ouverte avec les valeurs saisies pour correction.

5. **Création — erreur dates invalides** — Given `start_date >= end_date`, When création, Then frontend bloque avant POST (validation client). En backup : si bypass (curl direct), le DB CHECK constraint déclenche `DbError::CheckConstraintViolation` → `AppError::Validation('error-fiscal-year-dates-invalid')` → 400.

6. **Renommage — formulaire** — Given une ligne avec status `Open` (renommage permis aussi sur Closed — `name` reste mutable), When clic « Renommer », Then modale d'édition avec uniquement le champ `name`. Les dates sont affichées en lecture seule grisée (visibles mais non éditables).

7. **Renommage — appel API** — Given le nouveau nom, When `PUT /api/v1/fiscal-years/{id}` body `{ name }` retourne `200`, Then la ligne se met à jour, toast vert `notifySuccess('fiscal-year-renamed')`.

8. **Renommage — erreur nom déjà utilisé** — Given un autre exercice de la même company porte déjà ce nom, When PUT, Then `400 Validation` `code='VALIDATION_ERROR'` `message=t('error-fiscal-year-name-duplicate')`.

9. **Clôture — bouton + confirmation** — Given une ligne avec status `Open`, When clic « Clôturer », Then modale de confirmation : « Vous êtes sur le point de clôturer l'exercice {name}. Cette action est **irréversible** : aucune écriture, facture ou paiement ne pourra plus être enregistré sur cette période. Confirmer ? » avec bouton rouge « Clôturer définitivement » et « Annuler ».

10. **Clôture — appel API** — Given confirmation, When `POST /api/v1/fiscal-years/{id}/close` retourne `200` + body `FiscalYearResponse` avec `status='Closed'`, Then la ligne se met à jour (badge passe au gris « Closed »), le bouton « Clôturer » disparaît, toast vert `notifySuccess('fiscal-year-closed')`.

11. **Clôture — déjà clos** — Given un exercice déjà `Closed` (race entre 2 onglets ou refresh manqué), When POST close, Then `409 Conflict` `code='ILLEGAL_STATE_TRANSITION'` (mappé par le variant existant `AppError::Database(DbError::IllegalStateTransition)`, voir `errors.rs:440-447` — pas de nouveau variant à créer) `message=t('error-fiscal-year-already-closed')`. Le frontend affiche un toast rouge et rafraîchit la liste pour resynchroniser.

12. **RBAC** — Given un user Lecteur (rôle non-Admin non-Comptable), When il accède à `/settings/fiscal-years`, Then il voit la liste en lecture seule (pas de bouton « Nouvel exercice », « Renommer », « Clôturer »). Les routes POST / PUT / POST close retournent `403 Forbidden`. (V0.1 RBAC : seuls Admin et Comptable peuvent muter.)

13. **Onboarding Path B — auto-création** — Given un user finalise l'onboarding production (Path B, `is_demo=false`, step 7→8), When `POST /api/v1/onboarding/finalize` réussit, Then un fiscal_year est automatiquement créé pour l'année calendaire courante (`name="Exercice {YYYY}"`, `start_date={YYYY}-01-01`, `end_date={YYYY}-12-31`, `status=Open`) **uniquement si** `fiscal_years::list_by_company(company_id).is_empty()`. Si déjà un exercice existe (re-finalize ou autre), skip silencieusement.

14. **Onboarding Path A — auto-création (déjà existant)** — Le `kesh_seed::seed_demo` crée déjà un fiscal_year (lignes 145-159 actuelles). **Cette story ne modifie pas seed_demo.** Vérification : un test E2E `fiscal_years_e2e::demo_path_creates_fiscal_year` valide la non-régression.

15. **Audit log — création** — Given `POST /api/v1/fiscal-years` réussit, When la transaction commit, Then une entrée `audit_log` est insérée avec `action='fiscal_year.created'`, `user_id={current_user}`, `entity_type='fiscal_year'`, `entity_id={new_fy.id}`, `details_json={snapshot complet}`. Test : vérifier l'entrée via `audit_log::find_by_entity('fiscal_year', new_fy.id)`.

16. **Audit log — rename** — Given `PUT /api/v1/fiscal-years/{id}` réussit, When commit, Then `audit_log` entry avec `action='fiscal_year.updated'`, `details_json={ "before": snapshot, "after": snapshot }`. Wrapper `before/after` car transition à 2 états (cohérent décision story 3.5).

17. **Audit log — close** — Given `POST /api/v1/fiscal-years/{id}/close` réussit, When commit, Then `audit_log` entry avec `action='fiscal_year.closed'`, `details_json={snapshot post-close}` (status='Closed', `updated_at` reflète automatiquement le moment de clôture via le `ON UPDATE CURRENT_TIMESTAMP(3)` du schéma). **Pas de colonne `closed_at` à référencer** (Pass 1 M-2 — la colonne n'existe pas, `updated_at` suffit). Snapshot direct (pas de wrapper — c'est une transition à 1 résultat).

18. **Audit log — onboarding auto-create** — Given finalize Path B crée un fiscal_year automatiquement (AC #13), When commit, Then audit_log entry avec `action='fiscal_year.created'` et `user_id={admin du tenant}` (= `current_user.user_id` du handler finalize — le user qui finalise). Cohérent — pas de seed system bypass car finalize est une action utilisateur.

19. **i18n complet** — Toutes les clés UI fiscal-year-* + error-fiscal-year-* présentes dans les 4 locales (`fr-CH`, `de-CH`, `it-CH`, `en-CH`). Aucun hardcode FR dans le code Svelte. Liste des clés (~17) dans Tasks T6.

20. **Tests E2E backend** — `crates/kesh-api/tests/fiscal_years_e2e.rs` couvre les cas suivants. Setup : utiliser `spawn_app` (pattern `crates/kesh-api/tests/onboarding_e2e.rs`). Pour le test Lecteur, créer manuellement un user avec `INSERT INTO users (..., role) VALUES (..., 'Lecteur')` après spawn_app. Tous les payloads en JSON camelCase :
   - **create_happy_path** : POST `{name: "Exercice 2027", startDate: "2027-01-01", endDate: "2027-12-31"}` → 201 + body avec id, status='Open'.
   - **create_overlap** (Pass 1 H-6) : pré-insérer Exercice 2027 (Jan-Dec) puis POST `{name: "Mid 2027", startDate: "2027-07-01", endDate: "2028-06-30"}` → 400 / `VALIDATION_ERROR` / message contient « overlap ».
   - **create_duplicate_name** (Pass 1 H-5) : pré-insérer Exercice 2027 puis POST `{name: "Exercice 2027", startDate: "2028-01-01", endDate: "2028-12-31"}` → 400 / `VALIDATION_ERROR` / message contient « name-duplicate » ou « déjà utilisé ».
   - **create_dates_invalid** : POST `{name: "X", startDate: "2027-12-31", endDate: "2027-01-01"}` → 400 (CHECK constraint).
   - **list_empty** : GET sur DB vierge → `200` + `[]`.
   - **list_populated_desc_order** : insérer 2025/2026/2027, GET → array dans l'ordre 2027, 2026, 2025.
   - **get_by_id_happy** : POST + récupérer id → GET `/{id}` → 200.
   - **get_by_id_missing** : GET `/9999` → 404.
   - **get_by_id_other_company_returns_404** (Pass 1 H-8 IDOR) : pré-insérer un fiscal_year pour `company_id=2` (ajouter une 2e company de test), GET avec un user de `company_id=1` → 404 (anti-énumération, pas 403).
   - **update_name_happy** : PUT `/{id}` `{name: "FY 2027"}` → 200 + body avec nouveau name.
   - **update_name_duplicate** : pré-insérer 2027 et 2028, PUT 2028 `{name: "Exercice 2027"}` → 400 / `name-duplicate`.
   - **update_name_other_company_returns_404** (Pass 1 H-8) : PUT sur fiscal_year d'une autre company → 404.
   - **update_name_empty** : PUT `{name: "   "}` → 400 / `name-empty`.
   - **close_happy** : POST `/{id}/close` sur Open → 200 + body avec status='Closed'.
   - **close_already_closed** : POST close sur déjà Closed → 409 / `ILLEGAL_STATE_TRANSITION`.
   - **close_other_company_returns_404** (Pass 1 H-8) : POST close sur fiscal_year d'une autre company → 404.
   - **rbac_post_create_lecteur_returns_403** : POST `/api/v1/fiscal-years` avec user role=Lecteur → 403.
   - **rbac_put_update_lecteur_returns_403** : PUT avec Lecteur → 403.
   - **rbac_post_close_lecteur_returns_403** : POST close avec Lecteur → 403.
   - **rbac_get_list_no_auth_returns_401** : GET sans token → 401.
   - **path_b_finalize_creates_fiscal_year** : flow finalize Path B → vérifier `list_by_company().len() == 1` + name = `Exercice {current_year}`.
   - **path_b_finalize_idempotent_with_existing_fiscal_year** : pré-insérer un fiscal_year, finalize Path B → vérifier `list_by_company().len() == 1` (pas de doublon) + audit_log NE contient PAS d'entrée fiscal_year.created supplémentaire.

21. **Tests Playwright** — `frontend/tests/e2e/fiscal-years.spec.ts` : un user Comptable se connecte, navigue vers `/settings/fiscal-years`, crée un exercice 2027, valide une facture datée 2027-06-15 (réutilise infra story 5-2), confirme que l'écriture comptable est créée (la facture passe en `Validated`), revient sur `/settings/fiscal-years`, clôture l'exercice, retente une validation de facture 2027 → erreur "exercice clôturé" (FR24).

22. **Fallback toast actionnable validate_invoice sans fiscal_year** (Pass 1 H-9 — epics.md:858) — Given un user qui a (manuellement ou par cas exceptionnel) une instance sans fiscal_year ouvert, When il tente `POST /invoices/{id}/validate` et reçoit `400` avec `code='NO_FISCAL_YEAR'` ou message backend mentionnant l'absence d'exercice, Then le frontend affiche un toast actionnable « Créez d'abord un exercice comptable dans Paramètres → Exercices » avec un bouton/lien qui navigue vers `/settings/fiscal-years`. Les clés i18n : `error-fiscal-year-missing` + `go-to-settings`. Implémentation T5.9.

23. **DELETE non supporté** (Pass 1 M-11 — CO art. 957-964) — Given une tentative `DELETE /api/v1/fiscal-years/{id}`, When la requête arrive, Then la réponse est `405 Method Not Allowed` (axum renvoie automatiquement 405 si la route n'est pas enregistrée pour cette méthode). Aucun handler `delete_fiscal_year` n'est créé. Aucun bouton suppression dans l'UI. Test E2E vérifie le 405.

## Tasks / Subtasks

### T1 — Repository : audit log + update_name + create variants + lock-aware lookups (AC: #2-#4, #6-#11, #15-#18, #22)

- [ ] T1.1 Refactor `fiscal_years::create` :
  - Signature finale : `pub async fn create(pool: &MySqlPool, user_id: i64, new: NewFiscalYear) -> Result<FiscalYear, DbError>`.
  - Algorithme (Pass 1 H-5 + H-6) : `tx = pool.begin()` → `find_overlapping(tx, ...) FOR UPDATE` → si Some, rollback + `Err(DbError::Invariant("overlap"))` → `find_by_name(tx, ...) FOR UPDATE` → si Some, rollback + `Err(DbError::Invariant("name-duplicate"))` → INSERT → audit_log snapshot direct → commit.
  - Note : utiliser `Invariant("overlap")` et `Invariant("name-duplicate")` comme codes ; le handler T2.4 distingue via le contenu du `String` interne (pattern simple) ou via deux variants enum dédiés (préférer le simple pour v0.1, ajouter un commentaire si évolution future).
- [ ] T1.2 Ajouter `fiscal_years::create_if_absent_in_tx(tx: &mut sqlx::Transaction<'_, sqlx::MySql>, user_id: i64, new: NewFiscalYear) -> Result<Option<FiscalYear>, DbError>` — Pass 1 H-4 (insert atomique pour onboarding finalize).
  - SQL : `INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) SELECT ?, ?, ?, ?, 'Open' WHERE NOT EXISTS (SELECT 1 FROM fiscal_years WHERE company_id = ?)`.
  - Si `rows_affected == 1` → `SELECT id FROM fiscal_years WHERE company_id = ? AND name = ?` → audit_log → return `Some(fy)`.
  - Si `rows_affected == 0` → return `Ok(None)` (idempotent, pas d'audit).
  - Pas de FOR UPDATE applicatif — l'atomicité est garantie par la sous-requête NOT EXISTS dans la même tx.
- [ ] T1.3 Ajouter `fiscal_years::update_name(pool: &MySqlPool, user_id: i64, id: i64, new_name: String) -> Result<FiscalYear, DbError>` — Pass 1 H-7 (signature harmonisée, **pas de `expected_version`**).
  - Validation : `new_name.trim() != ""` sinon `DbError::Invariant("name-empty")`.
  - Algorithme : `tx = begin()` → `SELECT * FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE` (utiliser `find_by_id_in_company_locked`, voir T1.5) → si None, rollback + `DbError::NotFound` → check `find_by_name(tx, company_id, new_name)` ≠ None pour distinguer le doublon → UPDATE SET name = ?, updated_at = NOW() WHERE id = ? → re-SELECT after → audit_log wrapper `{before, after}` → commit.
  - Le caller (T2.5) doit déjà avoir vérifié `company_id` via `find_by_id_in_company` ; l'update_name re-vérifie pour défense en profondeur.
- [ ] T1.4 Refactor `fiscal_years::close` :
  - Signature finale : `pub async fn close(pool: &MySqlPool, user_id: i64, id: i64) -> Result<FiscalYear, DbError>`.
  - Logique inchangée (transition Open → Closed avec guard `WHERE status='Open'` qui retourne `DbError::IllegalStateTransition` si déjà Closed).
  - Ajouter audit_log snapshot direct avec `action='fiscal_year.closed'` et `details_json = snapshot post-close`.
- [ ] T1.5 Ajouter `fiscal_years::find_by_id_in_company(pool: &MySqlPool, company_id: i64, id: i64) -> Result<Option<FiscalYear>, DbError>` — Pass 1 H-3 (Anti-Pattern 4 fix).
  - SQL : `SELECT ... FROM fiscal_years WHERE id = ? AND company_id = ?`. Pas de fetch-then-check côté handler.
  - Variante locked : `find_by_id_in_company_locked(tx, company_id, id)` avec `FOR UPDATE` pour les paths qui suivent par un UPDATE.
- [ ] T1.6 Ajouter `fiscal_years::find_overlapping(tx: &mut Transaction, company_id: i64, start_date: NaiveDate, end_date: NaiveDate) -> Result<Option<FiscalYear>, DbError>` — Pass 1 H-6.
  - SQL : `SELECT ... FROM fiscal_years WHERE company_id = ? AND start_date <= ? AND end_date >= ? FOR UPDATE LIMIT 1` (chevauchement d'intervalles fermés).
- [ ] T1.7 Ajouter `fiscal_years::find_by_name(tx: &mut Transaction, company_id: i64, name: &str) -> Result<Option<FiscalYear>, DbError>` — Pass 1 H-5 (distinguer le cas duplicate name avant l'INSERT).
  - SQL : `SELECT ... FROM fiscal_years WHERE company_id = ? AND name = ? FOR UPDATE LIMIT 1`.
- [ ] T1.8 Ajouter `fiscal_years::create_for_seed(pool: &MySqlPool, new: NewFiscalYear) -> Result<FiscalYear, DbError>` — variante non-auditée pour `kesh_seed::seed_demo`. Cohérent avec décision story 3.5 sur `bulk_create_from_chart` (contexte système, pas utilisateur).
- [ ] T1.9 Mettre à jour TOUS les callsites de `fiscal_years::create` (Pass 1 H-1) :
  - `crates/kesh-seed/src/lib.rs:~150` — remplacer par `create_for_seed(pool, new)`.
  - Tous les tests dans `crates/kesh-db/tests/fiscal_years_repository.rs` (~9 callsites — passer `user_id=1` admin de test).
  - Vérifier via `grep -rn "fiscal_years::create" crates/` qu'aucun autre callsite n'est manqué.
  - Si `journal_entries.rs:1096` ou autre route appelle `fiscal_years::create` directement (ne devrait pas mais à vérifier), passer `current_user.user_id`.
- [ ] T1.10 Modifier `fiscal_years::list_by_company` (Pass 1 M-1) — **garder ASC en repo** (pas de breaking change pour les callers existants comme story 5.2). Le tri DESC pour l'UI sera fait côté handler T2.2 via `result.reverse()` ou un sort_by.
- [ ] T1.11 Compléter les tests `crates/kesh-db/tests/fiscal_years_repository.rs` :
  - `test_create_writes_audit_log`
  - `test_create_rejects_overlap_with_existing` (Pass 1 H-6 — exercice 2027 + tentative Jul 2027-Jun 2028 → Invariant("overlap"))
  - `test_create_rejects_duplicate_name` (Pass 1 H-5 — distinct du test overlap)
  - `test_update_name_writes_audit_log_with_before_after`
  - `test_update_name_rejects_empty`
  - `test_update_name_rejects_duplicate_name`
  - `test_update_name_not_found`
  - `test_close_writes_audit_log`
  - `test_create_for_seed_does_not_audit`
  - `test_create_if_absent_in_tx_creates_when_empty`
  - `test_create_if_absent_in_tx_skips_when_exists`
  - `test_find_by_id_in_company_returns_none_for_other_company` (Pass 1 H-8 multi-tenant scoping)
- [ ] T1.12 Documenter les nouveaux lock sites dans `docs/MULTI-TENANT-SCOPING-PATTERNS.md` Pattern 5 — Pass 1 M-9. Ajouter une entrée `fiscal_years::create / update_name / close / find_*_locked` à la table avec description « FOR UPDATE locks pour pré-check unicité/overlap et figer le before-snapshot d'audit log ».

### T2 — API routes : `crates/kesh-api/src/routes/fiscal_years.rs` (AC: #2-#4, #6-#11, #15-#17, #20)

- [ ] T2.1 Créer le module `fiscal_years.rs`. Structures DTO :
  - `FiscalYearResponse { id, company_id (i64 → camelCase companyId), name, start_date, end_date, status (string Open/Closed), created_at, updated_at }` avec `#[serde(rename_all = "camelCase")]` (cohérent avec contacts.rs, accounts.rs).
  - `CreateFiscalYearRequest { name: String, startDate: NaiveDate, endDate: NaiveDate }`.
  - `UpdateFiscalYearRequest { name: String }`.
- [ ] T2.2 Handler `list_fiscal_years(State, Extension<CurrentUser>) -> Vec<FiscalYearResponse>`. Appelle `fiscal_years::list_by_company(pool, current_user.company_id)` (renvoie ASC). Renverse côté handler avant retour : `result.reverse()` ou `result.sort_by(|a, b| b.start_date.cmp(&a.start_date))`. Renvoie `200`. Pass 1 M-1 fix : on ne touche pas le repo (qui reste ASC pour stories 5.2 / 3.2 callers).
- [ ] T2.3 Handler `get_fiscal_year(State, Extension<CurrentUser>, Path(id)) -> FiscalYearResponse`. Appelle `fiscal_years::find_by_id_in_company(pool, current_user.company_id, id)` (Pass 1 H-3 — query scopée directement, pas de fetch-then-check). Si `None`, retourner `AppError::Database(DbError::NotFound)` → 404 (anti-énumération — pattern story 6-2 multi-tenant audit). Pas de 403.
- [ ] T2.4 Handler `create_fiscal_year(State, Extension<CurrentUser>, Json<CreateFiscalYearRequest>) -> 201 + FiscalYearResponse`. Construit `NewFiscalYear { company_id: current_user.company_id, name, start_date, end_date }`. Appelle `fiscal_years::create(pool, current_user.user_id, new)`. Map les erreurs DB (Pass 1 H-5) :
  - `DbError::Invariant("overlap")` → `AppError::Validation(t("error-fiscal-year-overlap"))`.
  - `DbError::Invariant("name-duplicate")` → `AppError::Validation(t("error-fiscal-year-name-duplicate"))`.
  - `DbError::Invariant("name-empty")` → `AppError::Validation(t("error-fiscal-year-name-empty"))` (cas extrême, devrait être validé côté frontend).
  - `DbError::CheckConstraintViolation` → `AppError::Validation(t("error-fiscal-year-dates-invalid"))` (filet de sécurité si bypass frontend).
  - `DbError::UniqueConstraintViolation` → `AppError::Validation(t("error-fiscal-year-conflict"))` (filet de sécurité — ne devrait jamais arriver après les pré-checks T1.6/T1.7).
  - `DbError::ForeignKeyViolation` → `AppError::Internal` (ne devrait jamais arriver — JWT garantit company_id valide).
- [ ] T2.5 Handler `update_fiscal_year(State, Extension<CurrentUser>, Path(id), Json<UpdateFiscalYearRequest>) -> 200 + FiscalYearResponse`. Vérification multi-tenant via `fiscal_years::find_by_id_in_company` (Pass 1 H-3) — si None, 404. Si Some, appelle `update_name(pool, current_user.user_id, id, new_name)`. Map :
  - `DbError::NotFound` → 404 (race entre find et update — improbable, mais propagation propre).
  - `DbError::Invariant("name-empty")` → `AppError::Validation(t("error-fiscal-year-name-empty"))`.
  - `DbError::Invariant("name-duplicate")` → `AppError::Validation(t("error-fiscal-year-name-duplicate"))`.
- [ ] T2.6 Handler `close_fiscal_year(State, Extension<CurrentUser>, Path(id)) -> 200 + FiscalYearResponse`. Vérification multi-tenant via `fiscal_years::find_by_id_in_company`. Appelle `close(pool, current_user.user_id, id)`. Pas de mapping spécial à ajouter — `DbError::IllegalStateTransition` est déjà mappé via `AppError::Database(DbError::IllegalStateTransition)` → HTTP 409 / `ILLEGAL_STATE_TRANSITION` (cf. errors.rs:440-447, Pass 1 C-1).
- [ ] T2.7 Mounting des routes dans `crates/kesh-api/src/lib.rs::build_router` :
  - 2 endpoints en `authenticated_routes` : `GET /api/v1/fiscal-years`, `GET /api/v1/fiscal-years/{id}`.
  - 3 endpoints en `comptable_routes` : `POST /api/v1/fiscal-years`, `PUT /api/v1/fiscal-years/{id}`, `POST /api/v1/fiscal-years/{id}/close`.
- [ ] T2.8 Tests E2E `crates/kesh-api/tests/fiscal_years_e2e.rs` couvrant tous les ACs #2-#4, #6-#12, #15-#17. Pattern `spawn_app` (cohérent avec les autres `*_e2e.rs`).

### T3 — Onboarding finalize Path B auto-create (AC: #13, #14, #18, #20)

- [ ] **T3.0 (PRÉREQUIS — Pass 1 C-2)** — Ajouter `Extension<CurrentUser>` à la signature du handler `finalize`. Aujourd'hui `pub async fn finalize(State(state): State<AppState>) -> ...`. Doit devenir `pub async fn finalize(State(state): State<AppState>, Extension(current_user): Extension<CurrentUser>) -> ...`. Cohérent avec les handlers post-Story 3.5 (create_journal_entry, etc.). Aucun caller direct (axum injecte automatiquement via le router). Tests E2E continueront de fonctionner car l'auth middleware injecte `CurrentUser` en amont.
- [ ] T3.1 Modifier `crates/kesh-api/src/routes/onboarding.rs::finalize`. Après `insert_with_defaults_in_tx` et avant le UPDATE step_completed=8, ajouter un bloc avec **insert atomique anti-TOCTOU (Pass 1 H-4)** :
  ```rust
  // AC #13: auto-create fiscal_year for current calendar year if none exists.
  // Pass 1 H-4 fix: utiliser insert atomique au lieu de check-then-insert pour
  // éviter une race contre seed_demo concurrent. La sous-requête NOT EXISTS
  // s'exécute avec le même snapshot que l'INSERT dans la même tx.
  let year = chrono::Utc::now().naive_utc().date().year();
  let fy_name = format!("Exercice {year}");
  let fy_start = chrono::NaiveDate::from_ymd_opt(year, 1, 1).expect("valid");
  let fy_end = chrono::NaiveDate::from_ymd_opt(year, 12, 31).expect("valid");
  let inserted_rows = sqlx::query(
      "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
       SELECT ?, ?, ?, ?, 'Open' \
       WHERE NOT EXISTS (SELECT 1 FROM fiscal_years WHERE company_id = ?)"
  )
  .bind(company.id).bind(&fy_name).bind(fy_start).bind(fy_end).bind(company.id)
  .execute(&mut *tx).await.map_err(map_db_error)?
  .rows_affected();
  // Si rows == 1, on a créé un nouveau fiscal_year ; on doit l'auditer.
  // Si rows == 0, un exercice existait déjà (idempotent — skip silencieusement).
  if inserted_rows == 1 {
      // Récupérer l'id pour audit_log puis auditer
      let new_fy_id = sqlx::query_scalar::<_, i64>(
          "SELECT id FROM fiscal_years WHERE company_id = ? AND name = ?"
      )
      .bind(company.id).bind(&fy_name)
      .fetch_one(&mut *tx).await.map_err(map_db_error)?;
      // Snapshot pour audit (cf. T1.2 create_in_tx pattern)
      // ... appel audit_log::insert_in_tx
  }
  ```
  Note : alternative plus propre = appeler une nouvelle `fiscal_years::create_if_absent_in_tx(tx, user_id, new) -> Result<Option<FiscalYear>>` qui encapsule cette logique. Préférer cette approche pour réutilisabilité.
- [ ] T3.2 Tests : étendre `onboarding_e2e.rs` ou `onboarding_path_b_e2e.rs` :
  - `path_b_finalize_creates_fiscal_year` — finalize Path B → vérifier `fiscal_years::list_by_company().len() == 1` + `name == "Exercice {YYYY}"`.
  - `path_b_finalize_idempotent_with_existing_fiscal_year` — pré-insérer un fiscal_year, finalize, vérifier `list_by_company().len() == 1` (pas dupliqué) + audit_log n'a PAS d'entrée fiscal_year.created supplémentaire.
  - `path_b_finalize_concurrent_creates_only_one` — 2 finalize concurrents (semi-difficile à tester proprement, peut être skip si trop complexe).

### T4 — Frontend feature lib `frontend/src/lib/features/fiscal-years/` (AC: #1-#11)

- [ ] T4.1 Créer `fiscal-years.types.ts` :
  ```ts
  export interface FiscalYearResponse {
      id: number;
      companyId: number;
      name: string;
      startDate: string; // YYYY-MM-DD
      endDate: string;
      status: 'Open' | 'Closed';
      createdAt: string;
      updatedAt: string;
  }
  export interface CreateFiscalYearRequest { name: string; startDate: string; endDate: string; }
  export interface UpdateFiscalYearRequest { name: string; }
  ```
- [ ] T4.2 Créer `fiscal-years.api.ts` avec 5 fonctions typées : `listFiscalYears()`, `getFiscalYear(id)`, `createFiscalYear(req)`, `updateFiscalYear(id, req)`, `closeFiscalYear(id)`. Utiliser `apiClient` (pattern accounts/contacts).
- [ ] T4.3 Créer `fiscal-years.helpers.ts` (Pass 1 M-6 — pluriel cohérent avec api.ts/types.ts) :
  - `validateFiscalYearForm({ name, startDate, endDate }): string | null` — retourne null si OK ou clé i18n d'erreur (`'error-fiscal-year-name-empty'` | `'error-fiscal-year-dates-invalid'`).
  - `formatFiscalYearLabel(fy: FiscalYearResponse): string` — format affichage (`"Exercice 2027 (Open)"`).
  - `currentYearDefaults(): { name: string; startDate: string; endDate: string }` — retourne `{ name: 'Exercice {YYYY}', startDate: '{YYYY}-01-01', endDate: '{YYYY}-12-31' }` calculé via `new Date().getFullYear()` (pas de hardcode 2027).

### T5 — Frontend UI page (AC: #1-#11)

- [ ] T5.1 Créer `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`. Server load via `onMount` : appelle `listFiscalYears()`. State Svelte 5 (`$state`).
- [ ] T5.2 Liste tableau avec composants `Table.*` shadcn-svelte (cohérent avec `users/+page.svelte`). Colonnes : nom, date début, date fin, statut (Badge component avec couleur), actions.
- [ ] T5.3 Bouton « Nouvel exercice » → modale `Dialog.*` + formulaire `FiscalYearForm.svelte` (sous-composant). Pré-remplir avec `currentYearDefaults()`. Validation client via `validateFiscalYearForm`.
- [ ] T5.4 Bouton « Renommer » par ligne → même modale en mode édition (pre-fill avec la row, désactive les datepickers). Soumet via `updateFiscalYear`.
- [ ] T5.5 Bouton « Clôturer » par ligne (visible uniquement si `status === 'Open'`) → modale de confirmation `Dialog.*` avec texte AC #9. Soumet via `closeFiscalYear`.
- [ ] T5.6 Gestion des erreurs : `try/catch` + `notifyError(t(err.code))`. Pattern story 3.5.
- [ ] T5.7 Lien dans `frontend/src/routes/(app)/settings/+page.svelte` vers `/settings/fiscal-years` (nouvelle ligne dans la home settings).
- [ ] T5.8 RBAC frontend : depuis `currentUser.role` (déjà disponible via store auth), masquer les boutons mutateurs si rôle != Admin && != Comptable. Le backend reste la source de vérité (403 si bypass).
- [ ] T5.9 Fallback toast actionnable (Pass 1 H-9 — AC #22) — dans le handler d'erreur de `validate_invoice` côté frontend (`frontend/src/lib/features/invoices/`), détecter le cas où l'erreur backend est `code='NO_FISCAL_YEAR'` ou message contient « exercice » et afficher un toast avec un lien cliquable :
  ```svelte
  toast.error(
    msg('error-fiscal-year-missing', "Créez d'abord un exercice comptable"),
    { action: { label: msg('go-to-settings', 'Ouvrir Paramètres'), onClick: () => goto('/settings/fiscal-years') } }
  );
  ```
  Vérifier que `svelte-sonner` supporte les actions cliquables dans les toasts (sinon fallback : toast simple + bouton dédié dans la page facture).

### T6 — i18n (AC: #19)

- [ ] T6.1 Ajouter dans les 4 locales `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` :
  - `fiscal-year-title` (« Exercices comptables » / « Geschäftsjahre » / « Esercizi contabili » / « Fiscal Years »)
  - `fiscal-year-list-empty`
  - `fiscal-year-create-button` (« Nouvel exercice »)
  - `fiscal-year-name-label`, `fiscal-year-start-date-label`, `fiscal-year-end-date-label`, `fiscal-year-status-label`
  - `fiscal-year-status-open`, `fiscal-year-status-closed`
  - `fiscal-year-rename-button`, `fiscal-year-close-button`
  - `fiscal-year-close-confirmation-title`, `fiscal-year-close-confirmation-body`, `fiscal-year-close-confirmation-action`
  - `fiscal-year-created`, `fiscal-year-renamed`, `fiscal-year-closed` (notifications success)
  - `error-fiscal-year-overlap`, `error-fiscal-year-name-duplicate`, `error-fiscal-year-name-empty`, `error-fiscal-year-dates-invalid`, `error-fiscal-year-already-closed`, `error-fiscal-year-conflict` (filet sécurité — UNIQUE constraint inattendue après pré-checks)
  - `error-fiscal-year-missing` (Pass 1 H-9 / AC #22 — toast actionnable validate_invoice sans fiscal_year)
  - `go-to-settings` (Pass 1 H-9 / AC #22 — label du bouton dans le toast)
  - `settings-fiscal-years-link` (texte du lien depuis la page settings home)
  - **Total : ~21 clés × 4 locales = ~84 entrées.**
- [ ] T6.2 Lancer le lint i18n (`node frontend/scripts/lint-i18n-ownership.js`) après modification : doit rester PASS (pas de cross-feature key sharing).

### T7 — (supprimée Pass 1 C-1)

`AppError::IllegalStateTransition` existe déjà via `AppError::Database(DbError::IllegalStateTransition)` mappé dans `errors.rs:440-447` vers HTTP 409 + code `ILLEGAL_STATE_TRANSITION`. Aucun travail nécessaire — le variant est déjà là, le handler renvoie l'erreur DB qui est mappée automatiquement par `From<DbError> for AppError`. Référence : commit b63dc4e (Story 7-1 merge) introduit ce mapping.

### T8 — Tests Playwright E2E (AC: #21)

- [ ] T8.1 Créer `frontend/tests/e2e/fiscal-years.spec.ts`. Setup : seed_demo (réutilise fixtures story 6-4).
- [ ] T8.2 Scenario 1 « create + list » : naviguer vers `/settings/fiscal-years`, vérifier la liste contient l'exercice de seed, créer un nouvel exercice 2027, vérifier qu'il apparaît en tête (DESC).
- [ ] T8.3 Scenario 2 « validate invoice with fiscal_year » : créer un exercice 2027, créer une facture datée 2027-06-15, valider, vérifier que l'écriture est créée (réutilise routes `validate_invoice` story 5-2).
- [ ] T8.4 Scenario 3 « close + reject post-close » : clôturer l'exercice 2027, retenter la validation d'une nouvelle facture 2027 → 400 avec message « exercice clôturé » (FR24).
- [ ] T8.5 Scenario 4 « rename » : renommer l'exercice « Exercice 2027 » en « FY 2027 », vérifier que la liste reflète. Vérifier ensuite qu'on peut aussi renommer un exercice **Closed** (Pass 1 L-2) — clôturer FY 2027, renommer en « FY 2027 (clôturé) », vérifier succès.
- [ ] T8.6 Scenario 5 « Path A demo regression » (Pass 1 M-10 — couvre AC #14) : seed_demo via flow d'onboarding démo standard, naviguer vers `/settings/fiscal-years`, vérifier qu'un exercice avec `name='Exercice {YYYY}'`, `status='Open'`, dates 1er janvier-31 décembre est bien affiché. Test la non-régression de seed_demo après refactor T1.9.

### T9 — Sprint-status sync (méta)

- [ ] T9.1 À la création de cette story : passer `epic-3: done → in-progress` dans sprint-status.yaml. La fermeture de l'epic (rétro Epic 3 désormais `done` au lieu de `optional` ?) à décider à la fin (peut rester `optional` selon préférence Guy).
- [ ] T9.2 À la complétion de la story : passer `3-7-gestion-exercices-comptables: review` (puis `done` après code review).
- [ ] T9.3 Re-vérifier `3-6-journaux-personnalisables` qui reste `backlog`. Décision **hors scope 3.7** — cette story ne ferme pas Epic 3 (3-6 reste à faire ou à reclasser).

## Dev Notes

### Architecture & Patterns à respecter

- **Multi-tenant scoping** (KF-002, Pattern 5 dans `docs/MULTI-TENANT-SCOPING-PATTERNS.md`) :
  - **Tous les handlers GET/PUT/POST close** doivent vérifier que `current_user.company_id == fiscal_year.company_id` avant de répondre. Sinon 404 (pas 403, anti-énumération).
  - Le handler **list** filtre via `list_by_company(pool, current_user.company_id)` — déjà scopé.
  - Le handler **create** force `company_id = current_user.company_id` côté backend (jamais accepté du payload client).
- **Lock ordering** : cette story n'introduit pas de lock chains. Les FOR UPDATE existants (`find_open_covering_date`) restent inchangés. Pas de modification à Pattern 5 nécessaire.
- **Audit log** : pattern story 3.5 :
  - Wrapper `{before, after}` pour update (2 états).
  - Snapshot direct pour create / close (1 état).
  - Refactor des fns repo pour accepter `user_id`. Tx interne ouverte par la fn.
- **Error mapping** : pattern stories 3.2, 3.3, 5.2 :
  - DB `UniqueConstraintViolation` → `Validation('code i18n')`.
  - DB `CheckConstraintViolation` → `Validation('code i18n')`.
  - DB `IllegalStateTransition` → nouveau variant `AppError::IllegalStateTransition` (409).
- **Transactions** (Pass 1 M-7) : les handlers fiscal_years n'ouvrent **pas** de tx au niveau handler — la tx est interne aux fns repo (`create`, `update_name`, `close`, `create_if_absent_in_tx`). Aucun `best_effort_rollback` à câbler côté handler. Le seul cas tx-aware côté handler est `onboarding::finalize` (T3.0) qui partage déjà sa tx avec `insert_with_defaults_in_tx` et utilise les helpers existants.

### Source tree à toucher

```
crates/kesh-api/src/
├── lib.rs                             # T2.7 — mount routes
└── routes/
    ├── mod.rs                         # Pass 1 M-4 — pub mod fiscal_years; (mod register)
    ├── fiscal_years.rs                # NEW — T2.1-T2.7
    └── onboarding.rs                  # T3.0/T3.1 — Extension<CurrentUser> + auto-create

crates/kesh-api/tests/
└── fiscal_years_e2e.rs                # NEW — T2.8

crates/kesh-db/src/
└── repositories/
    └── fiscal_years.rs                # T1.1-T1.5 — refactor with user_id + audit + create_in_tx

crates/kesh-db/tests/
└── fiscal_years_repository.rs         # T1.6 — extend with audit log tests

crates/kesh-i18n/locales/
├── fr-CH/messages.ftl                 # T6.1 — ~17 keys
├── de-CH/messages.ftl                 # T6.1
├── it-CH/messages.ftl                 # T6.1
└── en-CH/messages.ftl                 # T6.1

crates/kesh-seed/src/
└── lib.rs                             # T1.9 — switch seed_demo to fiscal_years::create_for_seed

docs/
└── MULTI-TENANT-SCOPING-PATTERNS.md   # T1.12 — add fiscal_years entry to Pattern 5

frontend/src/
├── lib/features/fiscal-years/         # NEW
│   ├── fiscal-years.api.ts            # T4.2
│   ├── fiscal-years.types.ts          # T4.1
│   └── fiscal-year-helpers.ts         # T4.3
└── routes/(app)/settings/
    ├── +page.svelte                   # T5.7 — add link
    └── fiscal-years/
        └── +page.svelte               # NEW — T5.1-T5.6

frontend/tests/e2e/
└── fiscal-years.spec.ts               # NEW — T8.1-T8.5
```

### Project Structure Notes

- Le module `routes/fiscal_years.rs` est un **nouveau module** côté API. À ajouter dans `routes/mod.rs` (`pub mod fiscal_years;`).
- Côté frontend, `fiscal-years` est une nouvelle feature. Le segment URL utilise `/settings/fiscal-years` (avec tiret) — cohérent avec le standard sluggify.
- Aucune nouvelle migration DB requise : le schéma `fiscal_years` est déjà en place depuis `20260404000001_initial_schema.sql`. Si on souhaite ajouter optimistic locking en v0.2, ce sera une migration séparée (column `version INT NOT NULL DEFAULT 0`).

### Testing standards

- **Backend Rust** : `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` pour les tests d'intégration. Chaque test = DB éphémère propre. Pattern `crates/kesh-api/tests/spawn_app.rs` pour les E2E HTTP.
- **Frontend Vitest** : `vitest run` pour les helpers. Mocks `apiClient` via `vi.mock`.
- **Playwright** : `npm run test:e2e` (réutilise les fixtures déterministes story 6-4).
- **CI gate** : tous les tests doivent passer + cargo fmt + cargo clippy + svelte-check + ESLint + lint-i18n-ownership.

### Previous story intelligence (Stories 3-1 à 3-5, 5-2, 6-2, 7-1)

- **Story 3.5** (Notifications + audit) : a établi le pattern `audit_log::insert_in_tx` + `notify*.ts` helpers + tooltip pattern. **Réutiliser sans réinventer.**
- **Story 5.2** (Validate invoice) : utilise `find_open_covering_date(tx, company_id, date)` avec FOR UPDATE. Cette story 3.7 ne touche pas à cette fn — elle ajoute juste les exercices que `find_open_covering_date` peut découvrir.
- **Story 6-2** (Multi-tenant scoping) : a établi le pattern multi-tenant via `current_user.company_id`. Tous les handlers fiscal_years doivent suivre. Issue #40 (KF-002 reset gating) **fermée** — ne pas réintroduire de pattern divergent.
- **Story 7-1** (KF-002 audit + Pass 4-7 review cycle) : a renforcé les patterns Pattern 5 (lock ordering), best_effort_rollback, env_flag_enabled (KESH_PRODUCTION_RESET), `OnboardingResetForbidden` distinct error variant. **Lire `docs/MULTI-TENANT-SCOPING-PATTERNS.md`** avant de coder cette story.
- **Issue #43** (KF-002-H-002 deadlock middleware v0.2) : non bloquante pour 3.7 (pas de nouvelle lock chain).

### Git intelligence

- Recent commits (post Pass 7 merge) :
  - `b63dc4e` Story 7-1 squash merge (multi-tenant + reset gating)
  - `7c8822d` Story 6-2 + 6-3 + 7-6 squash
- **Conventions de message** : `fix(story-X-Y): description`, `feat(story-X-Y): description`, `docs(story-X-Y): description`. Une story par branche en général. Squash merge sur main via PR.
- **Branche cible** : `story/3-7-gestion-exercices-comptables` (créée par dev agent au moment de l'implémentation).

### Latest tech information

- **sqlx 0.8** : `Transaction<'_, MySql>` API stable. `sqlx::test` migrator pattern stable.
- **Axum** : extractor `Extension<CurrentUser>` éprouvé.
- **Svelte 5** runes (`$state`, `$derived`, `$effect`) : pattern largement utilisé dans le code existant. **Ne pas mélanger avec les stores Svelte 4.**
- **Tailwind 4** : utiliser les classes Tailwind directement, pas de CSS-in-JS.
- **shadcn-svelte** : composants Dialog, Table, Button, Input, Badge déjà installés.
- **Date pickers** : utiliser `<input type="date">` natif (pattern story 5-1 InvoiceForm). Pas de lib externe.

### Project context reference

- PRD : `_bmad-output/planning-artifacts/prd.md` — FR23, FR24, FR60, FR88.
- Architecture : `_bmad-output/planning-artifacts/architecture.md` — Section RBAC, Section Audit Log.
- Epics : `_bmad-output/planning-artifacts/epics.md` — Epic 3, Story 3.7 (lignes 843-866) **mais cette story file est la source canonique** (les ACs ici sont étendus vs. epics.md).

### References

- [Source: `crates/kesh-db/src/repositories/fiscal_years.rs` — fns existantes : create, find_by_id, find_covering_date, find_open_covering_date, list_by_company, close]
- [Source: `crates/kesh-db/src/entities/fiscal_year.rs` — FiscalYear, FiscalYearStatus, NewFiscalYear]
- [Source: `crates/kesh-db/migrations/20260404000001_initial_schema.sql` — table fiscal_years, contraintes UNIQUE/CHECK/FK]
- [Source: `crates/kesh-api/src/lib.rs` — build_router, RBAC tiers admin_routes/comptable_routes/authenticated_routes]
- [Source: `crates/kesh-api/src/routes/onboarding.rs` — handler finalize, pattern best_effort_rollback]
- [Source: `crates/kesh-api/src/routes/contacts.rs` — pattern handler CRUD avec multi-tenant scoping]
- [Source: `crates/kesh-seed/src/lib.rs:145-159` — auto-create fiscal_year en seed_demo (existant, ne pas régresser)]
- [Source: `_bmad-output/implementation-artifacts/3-5-notifications-aide-contextuelle-audit.md` — pattern audit log + helpers notify]
- [Source: `_bmad-output/implementation-artifacts/5-2-validation-numerotation-factures.md` — utilisation de find_open_covering_date]
- [Source: `_bmad-output/implementation-artifacts/7-1-audit-complete-kf-002-multi-tenant.md` — patterns multi-tenant scoping consolidés]
- [Source: `docs/MULTI-TENANT-SCOPING-PATTERNS.md` — Pattern 5 lock ordering]
- [Source: `_bmad-output/planning-artifacts/epics.md:843-866` — Story 3.7 spec originale]
- [Source: `_bmad-output/planning-artifacts/prd.md` — FR60 (close), FR24 (immutability post-close), FR88 (audit)]

## Dev Agent Record

### Agent Model Used

(à remplir par le dev agent au moment du dev-story)

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date       | Version | Description                                                                                                                                                                                                                                                                          | Auteur                              |
| ---------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------|
| 2026-04-27 | 0.1     | Story créée via bmad-create-story après merge Story 7-1. Re-ouvre Epic 3 (was done).                                                                                                                                                                                                 | Claude Opus 4.7 (1M context)        |
| 2026-04-27 | 0.2     | Pass 1 spec validate (3 reviewers Sonnet) → 25 patches : C-1 T7 supprimé (variant existe déjà), C-2 finalize Extension<CurrentUser>, H-1 callsites list, H-2 PATCH→PUT, H-3 find_by_id_in_company, H-4 atomic insert anti-TOCTOU, H-5 UNIQUE differentiation, H-6 overlap detection, H-7 update_name signature, H-8 IDOR tests, H-9 fallback toast AC #22, MEDIUM cleanup (closed_at→updated_at, file naming, best_effort_rollback, ASC vs DESC, Pattern 5 doc, +AC #23 DELETE 405). Trend Pass 1: 22 findings >LOW. | Claude Opus 4.7 (1M context)        |
