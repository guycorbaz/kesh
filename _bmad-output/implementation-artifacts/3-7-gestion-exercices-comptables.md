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
   - `PATCH /api/v1/fiscal-years/{id}` (comptable+, rename only)
   - `POST /api/v1/fiscal-years/{id}/close` (comptable+, transition Open→Closed)
2. **Audit log** — ajouter `audit_log::insert_in_tx` dans `fiscal_years::create`, `update_name`, `close` (pattern story 3.5). Le repo actuel ne le fait pas encore (cf. ligne *"Audit log integration: NOT YET implemented"* du recherche). Refactor des fn pour accepter `user_id`.
3. **Repo extension** — nouvelle fn `fiscal_years::update_name(pool, id, expected_version, user_id, new_name) -> Result<FiscalYear>`. **Pas de version optimiste pour l'instant** (le schéma actuel n'a pas de colonne `version` sur fiscal_years — voir Décisions). Validation : `new_name.trim() != ""`.
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
- **Suppression** : aucune fonction `delete` au niveau DB ni API. Conforme CO art. 957-964 (10 ans de conservation). Si l'utilisateur crée un exercice par erreur, il devra contacter le support (post-v0.1 : feature spéciale ou correction directe DB par admin système).
- **Optimistic locking sur fiscal_years.update_name** : **non requis** v0.1. La table n'a pas de colonne `version` (à ajouter dans une migration future si besoin). Pour l'update_name la fenêtre de race est minuscule (deux admins qui renomment le même exercice en même temps — hautement improbable, et la dernière écriture gagne, sans corruption de données comptables). À ré-évaluer en v0.2 si retour utilisateur.
- **Notification email à la clôture** : hors scope v0.1 (pas d'infra mail). Toast + audit log suffisent.

### Décisions de conception

- **Routes structure** : nouveau fichier `crates/kesh-api/src/routes/fiscal_years.rs` (module séparé, mod register dans `routes/mod.rs`). Pattern identique à `accounts.rs` / `companies.rs`.
- **Mounting** : 2 endpoints en `authenticated_routes` (GET list + GET id), 3 en `comptable_routes` (POST create, PATCH update, POST close). Cohérent avec AC : « Admin + Comptable ».
- **Réponse JSON** : structure `FiscalYearResponse { id, companyId, name, startDate, endDate, status, createdAt, updatedAt }` en camelCase via `#[serde(rename_all = "camelCase")]`. Status sérialisé `"Open" | "Closed"` (PascalCase, cohérent avec `FiscalYearStatus` enum existant).
- **Validation overlap** : **côté DB uniquement** via UNIQUE constraint `uq_fiscal_years_company_start_date`. La route mappe `DbError::UniqueConstraintViolation` → `AppError::Validation("Un exercice existe déjà avec cette date de début")`. Pas de pré-check (race window inutile).
- **Validation date order** : côté frontend ET côté DB (CHECK constraint `chk_fiscal_years_dates`). Le backend ne re-valide pas applicativement — il laisse le DB constraint parler via mapping `DbError::CheckConstraintViolation`.
- **Audit log entries** :
  - `fiscal_year.created` — `details_json` = snapshot complet de la row insérée (id, name, dates, status, company_id).
  - `fiscal_year.updated` — wrapper `{ "before": {...}, "after": {...} }` (cohérent avec story 3.5 « wrapper uniquement pour transitions à 2 états »).
  - `fiscal_year.closed` — snapshot direct de la row après transition (id, status="Closed", closed_at = updated_at).
- **Onboarding finalize auto-create fiscal_year** : nouveau bloc dans `finalize` après `insert_with_defaults_in_tx`. Vérifie `fiscal_years::list_by_company(...).is_empty()` avant de créer (idempotent face à une re-finalize). Crée pour l'année calendaire courante (1er janvier → 31 décembre, name = `"Exercice {YYYY}"`). Si un exercice existe déjà (cas Path A demo via seed_demo, ou re-finalize), **skip silencieusement**. **Cette logique réutilise `fiscal_years::create_in_tx`** (NOUVELLE fn à ajouter en plus de la pool variant — pattern miroir des `insert_with_defaults` / `insert_with_defaults_in_tx` story 5.2).
- **Lock ordering** : la nouvelle `fiscal_years::create_in_tx` n'acquiert pas de FOR UPDATE (création — pas de row à locker). Le `find_open_covering_date` existant utilise déjà FOR UPDATE et est documenté Pattern 5 (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`). Cette story n'introduit pas de nouveau lock site, donc rien à ajouter à Pattern 5.
- **i18n keys naming** : préfixe `fiscal-year-*` pour les libellés UI, `error-fiscal-year-*` pour les codes d'erreur. Cohérent avec la convention existante (`account-*`, `contact-*`, etc.).
- **Notification toasts** : utilise les helpers `notifySuccess` / `notifyError` (story 3.5). Pas de `toast.*` direct.

## Acceptance Criteria (AC)

1. **Page liste** — Given un user Admin ou Comptable, When il accède à `/settings/fiscal-years`, Then il voit un tableau triée `start_date DESC` avec colonnes `name`, `start_date` (format ISO local FR/DE/IT/EN), `end_date`, `status` (badge Open vert / Closed gris), et 2 actions par ligne (`Renommer`, `Clôturer` — ce dernier n'apparaît que si `status=Open`).

2. **Création — formulaire** — Given le bouton « Nouvel exercice » sur la page liste, When clic, Then une modale s'ouvre avec champs : `name` (texte libre, requis, non vide après trim), `start_date` (datepicker, requis), `end_date` (datepicker, requis). Validation côté frontend : `end_date > start_date`, `name.trim() != ""`. Le formulaire pré-remplit `name = "Exercice {YYYY}"` et `start_date = 1er janvier de l'année courante` et `end_date = 31 décembre de l'année courante` (suggestion modifiable).

3. **Création — appel API + retour** — Given le formulaire valide soumis, When `POST /api/v1/fiscal-years` retourne `201 Created` + body `FiscalYearResponse`, Then la modale ferme, le tableau se rafraîchit (la nouvelle ligne apparaît en tête car `start_date DESC`), et un toast vert `notifySuccess('fiscal-year-created')` s'affiche.

4. **Création — erreur overlap** — Given un exercice existe déjà avec la même `start_date` pour cette company, When création tentée, Then `POST /api/v1/fiscal-years` retourne `400 Validation` avec `code='VALIDATION_ERROR'` et `message=t('error-fiscal-year-overlap')`. Le toast rouge s'affiche, la modale reste ouverte avec les valeurs saisies pour correction.

5. **Création — erreur dates invalides** — Given `start_date >= end_date`, When création, Then frontend bloque avant POST (validation client). En backup : si bypass (curl direct), le DB CHECK constraint déclenche `DbError::CheckConstraintViolation` → `AppError::Validation('error-fiscal-year-dates-invalid')` → 400.

6. **Renommage — formulaire** — Given une ligne avec status `Open` (renommage permis aussi sur Closed — `name` reste mutable), When clic « Renommer », Then modale d'édition avec uniquement le champ `name`. Les dates sont affichées en lecture seule grisée (visibles mais non éditables).

7. **Renommage — appel API** — Given le nouveau nom, When `PATCH /api/v1/fiscal-years/{id}` body `{ name }` retourne `200`, Then la ligne se met à jour, toast vert `notifySuccess('fiscal-year-renamed')`.

8. **Renommage — erreur nom déjà utilisé** — Given un autre exercice de la même company porte déjà ce nom, When PATCH, Then `400 Validation` `code='VALIDATION_ERROR'` `message=t('error-fiscal-year-name-duplicate')`.

9. **Clôture — bouton + confirmation** — Given une ligne avec status `Open`, When clic « Clôturer », Then modale de confirmation : « Vous êtes sur le point de clôturer l'exercice {name}. Cette action est **irréversible** : aucune écriture, facture ou paiement ne pourra plus être enregistré sur cette période. Confirmer ? » avec bouton rouge « Clôturer définitivement » et « Annuler ».

10. **Clôture — appel API** — Given confirmation, When `POST /api/v1/fiscal-years/{id}/close` retourne `200` + body `FiscalYearResponse` avec `status='Closed'`, Then la ligne se met à jour (badge passe au gris « Closed »), le bouton « Clôturer » disparaît, toast vert `notifySuccess('fiscal-year-closed')`.

11. **Clôture — déjà clos** — Given un exercice déjà `Closed` (race entre 2 onglets ou refresh manqué), When POST close, Then `409 Conflict` `code='ILLEGAL_STATE_TRANSITION'` `message=t('error-fiscal-year-already-closed')`. Le frontend affiche un toast rouge et rafraîchit la liste pour resynchroniser.

12. **RBAC** — Given un user Lecteur (rôle non-Admin non-Comptable), When il accède à `/settings/fiscal-years`, Then il voit la liste en lecture seule (pas de bouton « Nouvel exercice », « Renommer », « Clôturer »). Les routes POST / PATCH / POST close retournent `403 Forbidden`. (V0.1 RBAC : seuls Admin et Comptable peuvent muter.)

13. **Onboarding Path B — auto-création** — Given un user finalise l'onboarding production (Path B, `is_demo=false`, step 7→8), When `POST /api/v1/onboarding/finalize` réussit, Then un fiscal_year est automatiquement créé pour l'année calendaire courante (`name="Exercice {YYYY}"`, `start_date={YYYY}-01-01`, `end_date={YYYY}-12-31`, `status=Open`) **uniquement si** `fiscal_years::list_by_company(company_id).is_empty()`. Si déjà un exercice existe (re-finalize ou autre), skip silencieusement.

14. **Onboarding Path A — auto-création (déjà existant)** — Le `kesh_seed::seed_demo` crée déjà un fiscal_year (lignes 145-159 actuelles). **Cette story ne modifie pas seed_demo.** Vérification : un test E2E `fiscal_years_e2e::demo_path_creates_fiscal_year` valide la non-régression.

15. **Audit log — création** — Given `POST /api/v1/fiscal-years` réussit, When la transaction commit, Then une entrée `audit_log` est insérée avec `action='fiscal_year.created'`, `user_id={current_user}`, `entity_type='fiscal_year'`, `entity_id={new_fy.id}`, `details_json={snapshot complet}`. Test : vérifier l'entrée via `audit_log::find_by_entity('fiscal_year', new_fy.id)`.

16. **Audit log — rename** — Given `PATCH /api/v1/fiscal-years/{id}` réussit, When commit, Then `audit_log` entry avec `action='fiscal_year.updated'`, `details_json={ "before": snapshot, "after": snapshot }`. Wrapper `before/after` car transition à 2 états (cohérent décision story 3.5).

17. **Audit log — close** — Given `POST /api/v1/fiscal-years/{id}/close` réussit, When commit, Then `audit_log` entry avec `action='fiscal_year.closed'`, `details_json={snapshot post-close}` (status='Closed', updated_at, etc.). Snapshot direct (pas de wrapper — c'est une transition à 1 résultat).

18. **Audit log — onboarding auto-create** — Given finalize Path B crée un fiscal_year automatiquement (AC #13), When commit, Then audit_log entry avec `action='fiscal_year.created'` et `user_id={admin du tenant}` (= `current_user.user_id` du handler finalize — le user qui finalise). Cohérent — pas de seed system bypass car finalize est une action utilisateur.

19. **i18n complet** — Toutes les clés UI fiscal-year-* + error-fiscal-year-* présentes dans les 4 locales (`fr-CH`, `de-CH`, `it-CH`, `en-CH`). Aucun hardcode FR dans le code Svelte. Liste des clés (~17) dans Tasks T6.

20. **Tests E2E backend** — `crates/kesh-api/tests/fiscal_years_e2e.rs` couvre : create happy path, create overlap (UNIQUE), create dates invalid (CHECK), list empty + populated, get_by_id, get_by_id missing → 404, update_name, update_name duplicate, close happy path, close already_closed → 409, RBAC : POST sans Comptable → 403, RBAC : GET sans auth → 401, finalize Path B auto-creates fiscal_year, finalize Path B idempotent (already exists → skip).

21. **Tests Playwright** — `frontend/tests/e2e/fiscal-years.spec.ts` : un user Comptable se connecte, navigue vers `/settings/fiscal-years`, crée un exercice 2027, valide une facture datée 2027-06-15 (réutilise infra story 5-2), confirme que l'écriture comptable est créée (la facture passe en `Validated`), revient sur `/settings/fiscal-years`, clôture l'exercice, retente une validation de facture 2027 → erreur "exercice clôturé" (FR24).

## Tasks / Subtasks

### T1 — Repository : audit log + update_name + create_in_tx (AC: #2-#4, #6-#7, #15-#18)

- [ ] T1.1 Refactor `fiscal_years::create(pool, new)` → `fiscal_years::create(pool, user_id, new)`. Ouvrir tx interne (pattern story 3.5 accounts), insérer fiscal_year, insérer audit_log avec `action='fiscal_year.created'` et snapshot, commit. Tous les call sites doivent passer `user_id`.
- [ ] T1.2 Ajouter `fiscal_years::create_in_tx(tx, user_id, new)` — variante tx-aware pour réutilisation depuis `onboarding::finalize`. Pattern miroir `insert_with_defaults` / `insert_with_defaults_in_tx` story 5.2. Commentaire `// MIRROR: keep synchronized with create()`.
- [ ] T1.3 Ajouter `fiscal_years::update_name(pool, user_id, id, new_name) -> Result<FiscalYear>`. Validation : `new_name.trim() != ""` (sinon `DbError::Invariant`). UPDATE SQL `SET name = ?, updated_at = NOW() WHERE id = ?`. Récupérer la row before via SELECT FOR UPDATE, faire l'update, récupérer la row after, écrire audit_log avec wrapper `{before, after}`, commit. Si `id` introuvable → `DbError::NotFound`.
- [ ] T1.4 Refactor `fiscal_years::close(pool, id)` → `fiscal_years::close(pool, user_id, id)`. Insérer audit_log avec snapshot post-close (status='Closed', updated_at). La logique métier `Open → Closed` reste inchangée.
- [ ] T1.5 Mettre à jour les call sites existants de `create` (notamment `kesh_seed::seed_demo` ligne ~150). **Décision** : `seed_demo` doit-il auditer ? Cohérent avec story 3.5 décision sur `bulk_create_from_chart` : **non**, le contexte seed est système, pas utilisateur. Ajouter une fonction non-auditée `create_for_seed(pool, new)` ou passer `user_id=1` (admin bootstrap). **Préférer `create_for_seed`** pour clarté (pas de fausse traçabilité).
- [ ] T1.6 Compléter les tests `crates/kesh-db/tests/fiscal_years_repository.rs` : `test_create_writes_audit_log`, `test_update_name_writes_audit_log_with_before_after`, `test_close_writes_audit_log`, `test_update_name_rejects_empty`, `test_update_name_not_found`, `test_create_for_seed_does_not_audit`.

### T2 — API routes : `crates/kesh-api/src/routes/fiscal_years.rs` (AC: #2-#4, #6-#11, #15-#17, #20)

- [ ] T2.1 Créer le module `fiscal_years.rs`. Structures DTO :
  - `FiscalYearResponse { id, company_id (i64 → camelCase companyId), name, start_date, end_date, status (string Open/Closed), created_at, updated_at }` avec `#[serde(rename_all = "camelCase")]` (cohérent avec contacts.rs, accounts.rs).
  - `CreateFiscalYearRequest { name: String, startDate: NaiveDate, endDate: NaiveDate }`.
  - `UpdateFiscalYearRequest { name: String }`.
- [ ] T2.2 Handler `list_fiscal_years(State, Extension<CurrentUser>) -> Vec<FiscalYearResponse>`. Appelle `fiscal_years::list_by_company(pool, current_user.company_id)`. Renvoie `200`. Tri `start_date DESC` (modifier le repo `list_by_company` ou faire le sort côté handler — **modifier le repo** car c'est plus naturel d'avoir l'ordre canonique en DB).
  - **Note** : la fn actuelle ordonne `start_date ASC` (cf. recherche). Story 3.7 modifie l'ORDER BY. Vérifier qu'aucun call site existant ne dépend de l'ordre ASC (probablement aucun — c'est une nouvelle UI). Si un dep existe, ajouter un param `order: Order` ou accepter un breaking change documenté.
- [ ] T2.3 Handler `get_fiscal_year(State, Extension<CurrentUser>, Path(id)) -> FiscalYearResponse`. Appelle `find_by_id` puis vérifie `result.company_id == current_user.company_id` (multi-tenant scoping — KF-002 pattern, sinon 403 ou 404 indistinguable).
  - **Décision** : retourner `404 NotFound` si `company_id` ne match pas (anti-énumération — pattern story 6-2 multi-tenant audit). Pas de 403.
- [ ] T2.4 Handler `create_fiscal_year(State, Extension<CurrentUser>, Json<CreateFiscalYearRequest>) -> 201 + FiscalYearResponse`. Construit `NewFiscalYear { company_id: current_user.company_id, name, start_date, end_date }`. Appelle `fiscal_years::create(pool, current_user.user_id, new)`. Map les erreurs DB :
  - `DbError::UniqueConstraintViolation` → `AppError::Validation('error-fiscal-year-overlap')` (le frontend reçoit le code générique `VALIDATION_ERROR`, le message i18n est résolu).
  - `DbError::CheckConstraintViolation` → `AppError::Validation('error-fiscal-year-dates-invalid')`.
  - `DbError::ForeignKeyViolation` → `AppError::Internal` (ne devrait jamais arriver — JWT garantit company_id valide).
- [ ] T2.5 Handler `update_fiscal_year(State, Extension<CurrentUser>, Path(id), Json<UpdateFiscalYearRequest>) -> 200 + FiscalYearResponse`. Vérification multi-tenant via `find_by_id` (404 si pas notre company). Appelle `update_name`. Map `DbError::UniqueConstraintViolation` → `error-fiscal-year-name-duplicate`.
- [ ] T2.6 Handler `close_fiscal_year(State, Extension<CurrentUser>, Path(id)) -> 200 + FiscalYearResponse`. Vérification multi-tenant. Appelle `close`. Map `DbError::IllegalStateTransition` → `AppError::IllegalStateTransition` (créer le variant si pas existant — actuellement `OptimisticLockConflict` est ce qui s'en rapproche le plus mais pas sémantiquement correct).
  - **Décision** : créer un nouveau variant `AppError::IllegalStateTransition` qui mappe sur HTTP 409 + code `ILLEGAL_STATE_TRANSITION`. Cohérent avec le wording AC #11.
- [ ] T2.7 Mounting des routes dans `crates/kesh-api/src/lib.rs::build_router` :
  - 2 endpoints en `authenticated_routes` : `GET /api/v1/fiscal-years`, `GET /api/v1/fiscal-years/{id}`.
  - 3 endpoints en `comptable_routes` : `POST /api/v1/fiscal-years`, `PATCH /api/v1/fiscal-years/{id}`, `POST /api/v1/fiscal-years/{id}/close`.
- [ ] T2.8 Tests E2E `crates/kesh-api/tests/fiscal_years_e2e.rs` couvrant tous les ACs #2-#4, #6-#12, #15-#17. Pattern `spawn_app` (cohérent avec les autres `*_e2e.rs`).

### T3 — Onboarding finalize Path B auto-create (AC: #13, #14, #18, #20)

- [ ] T3.1 Modifier `crates/kesh-api/src/routes/onboarding.rs::finalize`. Après `insert_with_defaults_in_tx` et avant le UPDATE step_completed=8, ajouter un bloc :
  ```rust
  // AC #13: auto-create fiscal_year for current calendar year if none exists
  let existing = kesh_db::repositories::fiscal_years::list_by_company(&state.pool, company.id).await?;
  if existing.is_empty() {
      let year = chrono::Utc::now().naive_utc().date().year();
      let new_fy = NewFiscalYear {
          company_id: company.id,
          name: format!("Exercice {year}"),
          start_date: chrono::NaiveDate::from_ymd_opt(year, 1, 1).expect("valid"),
          end_date: chrono::NaiveDate::from_ymd_opt(year, 12, 31).expect("valid"),
      };
      kesh_db::repositories::fiscal_years::create_in_tx(&mut tx, current_user.user_id, new_fy).await?;
  }
  ```
- [ ] T3.2 Tests : étendre `onboarding_e2e.rs` ou `onboarding_path_b_e2e.rs` :
  - `path_b_finalize_creates_fiscal_year` — finalize Path B → vérifier `fiscal_years::list_by_company().len() == 1` + `name == "Exercice {YYYY}"`.
  - `path_b_finalize_idempotent_with_existing_fiscal_year` — pré-insérer un fiscal_year, finalize, vérifier `list_by_company().len() == 1` (pas dupliqué).
- [ ] T3.3 Vérifier que le `current_user` est disponible dans le handler `finalize`. Si pas le cas (le handler actuel n'extrait pas `Extension<CurrentUser>`), le rajouter — cohérent avec story 3.5 qui l'a fait pour create_journal_entry.

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
- [ ] T4.3 Créer `fiscal-year-helpers.ts` :
  - `validateFiscalYearForm({ name, startDate, endDate }): string | null` — retourne null si OK ou message d'erreur i18n.
  - `formatFiscalYearLabel(fy)` — format affichage.
  - `currentYearDefaults()` — retourne `{ name: 'Exercice 2027', startDate: '2027-01-01', endDate: '2027-12-31' }` pour pré-remplir le formulaire.

### T5 — Frontend UI page (AC: #1-#11)

- [ ] T5.1 Créer `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`. Server load via `onMount` : appelle `listFiscalYears()`. State Svelte 5 (`$state`).
- [ ] T5.2 Liste tableau avec composants `Table.*` shadcn-svelte (cohérent avec `users/+page.svelte`). Colonnes : nom, date début, date fin, statut (Badge component avec couleur), actions.
- [ ] T5.3 Bouton « Nouvel exercice » → modale `Dialog.*` + formulaire `FiscalYearForm.svelte` (sous-composant). Pré-remplir avec `currentYearDefaults()`. Validation client via `validateFiscalYearForm`.
- [ ] T5.4 Bouton « Renommer » par ligne → même modale en mode édition (pre-fill avec la row, désactive les datepickers). Soumet via `updateFiscalYear`.
- [ ] T5.5 Bouton « Clôturer » par ligne (visible uniquement si `status === 'Open'`) → modale de confirmation `Dialog.*` avec texte AC #9. Soumet via `closeFiscalYear`.
- [ ] T5.6 Gestion des erreurs : `try/catch` + `notifyError(t(err.code))`. Pattern story 3.5.
- [ ] T5.7 Lien dans `frontend/src/routes/(app)/settings/+page.svelte` vers `/settings/fiscal-years` (nouvelle ligne dans la home settings).
- [ ] T5.8 RBAC frontend : depuis `currentUser.role` (déjà disponible via store auth), masquer les boutons mutateurs si rôle != Admin && != Comptable. Le backend reste la source de vérité (403 si bypass).

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
  - `error-fiscal-year-overlap`, `error-fiscal-year-name-duplicate`, `error-fiscal-year-dates-invalid`, `error-fiscal-year-already-closed`
  - `settings-fiscal-years-link` (texte du lien depuis la page settings home)
- [ ] T6.2 Lancer le lint i18n (`node frontend/scripts/lint-i18n-ownership.js`) après modification : doit rester PASS (pas de cross-feature key sharing).

### T7 — AppError extensions (AC: #11)

- [ ] T7.1 Ajouter variant `AppError::IllegalStateTransition` dans `crates/kesh-api/src/errors.rs` :
  ```rust
  /// Tentative de transition d'état invalide (ex: clôturer un exercice déjà clos).
  /// HTTP 409 Conflict — code unique `ILLEGAL_STATE_TRANSITION`.
  #[error("Transition d'état invalide")]
  IllegalStateTransition,
  ```
- [ ] T7.2 Ajouter le mapping dans `IntoResponse` :
  ```rust
  AppError::IllegalStateTransition => build_response(
      StatusCode::CONFLICT,
      "ILLEGAL_STATE_TRANSITION",
      &t("error-illegal-state-transition", "Transition d'état invalide"),
  ),
  ```
- [ ] T7.3 Ajouter mapping `From<DbError::IllegalStateTransition>` automatique (extend the `Database` variant si pas déjà couvert).

### T8 — Tests Playwright E2E (AC: #21)

- [ ] T8.1 Créer `frontend/tests/e2e/fiscal-years.spec.ts`. Setup : seed_demo (réutilise fixtures story 6-4).
- [ ] T8.2 Scenario 1 « create + list » : naviguer vers `/settings/fiscal-years`, vérifier la liste contient l'exercice de seed, créer un nouvel exercice 2027, vérifier qu'il apparaît en tête (DESC).
- [ ] T8.3 Scenario 2 « validate invoice with fiscal_year » : créer un exercice 2027, créer une facture datée 2027-06-15, valider, vérifier que l'écriture est créée (réutilise routes `validate_invoice` story 5-2).
- [ ] T8.4 Scenario 3 « close + reject post-close » : clôturer l'exercice 2027, retenter la validation d'une nouvelle facture 2027 → 400 avec message « exercice clôturé » (FR24).
- [ ] T8.5 Scenario 4 « rename » : renommer l'exercice « Exercice 2027 » en « FY 2027 », vérifier que la liste reflète.

### T9 — Sprint-status sync (méta)

- [ ] T9.1 À la création de cette story : passer `epic-3: done → in-progress` dans sprint-status.yaml. La fermeture de l'epic (rétro Epic 3 désormais `done` au lieu de `optional` ?) à décider à la fin (peut rester `optional` selon préférence Guy).
- [ ] T9.2 À la complétion de la story : passer `3-7-gestion-exercices-comptables: review` (puis `done` après code review).
- [ ] T9.3 Re-vérifier `3-6-journaux-personnalisables` qui reste `backlog`. Décision **hors scope 3.7** — cette story ne ferme pas Epic 3 (3-6 reste à faire ou à reclasser).

## Dev Notes

### Architecture & Patterns à respecter

- **Multi-tenant scoping** (KF-002, Pattern 5 dans `docs/MULTI-TENANT-SCOPING-PATTERNS.md`) :
  - **Tous les handlers GET/PATCH/POST close** doivent vérifier que `current_user.company_id == fiscal_year.company_id` avant de répondre. Sinon 404 (pas 403, anti-énumération).
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
- **Best-effort rollback** (pattern P12 du Pass 5) : pour les early returns dans les handlers, utiliser `best_effort_rollback(tx).await` (helper dans `routes/onboarding.rs:28-33`). Nouvelle helper à factoriser dans un module commun ? **Décision** : pour cette story, l'inline `let _ = tx.rollback().await;` suffit — refactor à un module helper si pattern se généralise.

### Source tree à toucher

```
crates/kesh-api/src/
├── errors.rs                          # AC #11 — new AppError::IllegalStateTransition
├── lib.rs                             # T2.7 — mount routes
└── routes/
    ├── fiscal_years.rs                # NEW — T2.1-T2.7
    └── onboarding.rs                  # T3.1 — auto-create fiscal_year in finalize

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
└── lib.rs                             # T1.5 — switch seed_demo to fiscal_years::create_for_seed

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

| Date       | Version | Description                                                                                  | Auteur                                  |
| ---------- | ------- | -------------------------------------------------------------------------------------------- | --------------------------------------- |
| 2026-04-27 | 0.1     | Story créée via bmad-create-story après merge Story 7-1. Re-ouvre Epic 3 (was done).         | Claude Opus 4.7 (1M context)            |
