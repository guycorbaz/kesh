# Story 14.2 : Réouverture d'un exercice clôturé (verrou réversible tracé)

## Status

ready-for-dev

## Story

**As a** administrateur de Kesh (fiduciaire, PME, indépendant),
**I want** pouvoir **rouvrir un exercice comptable clôturé** — avec un **motif obligatoire** et une **trace d'audit** — pour corriger une erreur découverte après clôture,
**so that** je ne sois pas bloqué par une clôture prématurée, tout en gardant une **piste d'audit** de qui a rouvert quoi, quand et pourquoi (CO art. 957-964), et sans jamais casser la cohérence des exercices postérieurs.

## Contexte

### Ce qui existe (14-1 et antérieur)

La clôture d'un exercice (`fiscal_years` `Open → Closed`) est aujourd'hui un **verrou irréversible** :

- `fiscal_years::close` (`fiscal_years.rs:568-625`) fait un `UPDATE ... SET status='Closed' WHERE ... status='Open'`, écrit un audit `"fiscal_year.closed"` (`:617-621`), le tout dans une transaction.
- Toute tentative de re-clôture d'un exercice déjà clos retourne **`DbError::IllegalStateTransition`** avec le message *« réouverture interdite (CO art. 957-964) »* (`fiscal_years.rs:599-601`). **C'est cette décision que 14-2 change.**
- L'**immutabilité** des écritures d'un exercice clos est appliquée par **trois gardes** dans `journal_entries.rs` (create `:189-201`, update `:707-735`, delete `:970-991`) qui lisent le **statut vivant** de `fiscal_years` sous verrou `FOR UPDATE` → un compte rendu `Closed` bloque l'édition (`DbError::FiscalYearClosed`).
- **14-1** (modèle temps réel virtuel) : les soldes sont **calculés en direct**, **aucune table snapshot** n'existe (confirmé : `grep snapshot|closing_balance migrations/` = 0). La note 14-1 anticipe explicitement 14-2 : *« un snapshot d'exercice rouvert devra être invalidé/recalculé (déclenché par `fiscal_year.reopened`). En v1 virtuel pur, rien à invalider. »*

### La décision que 14-2 inverse

> *« un exercice clôturé ne peut pas être ré-ouvert »* (ancienne AC 13.1 / doc module `fiscal_years.rs:3-16`).

**Nouvelle décision (Guy, 2026-07-24) — modèle Odoo « verrou réversible tracé »** : la clôture reste un verrou, mais un **administrateur** peut le **lever** avec un **motif obligatoire** et un **audit**. Un exercice rouvert redevient éditable ; il pourra être **re-clôturé** (via le `close` existant). Techniquement la DB n'a jamais interdit `Closed → Open` (le `CHECK` autorise, 0 trigger) — l'interdiction est **purement applicative** dans `close()`. La lever est un changement additif, sans migration.

### Ce qui n'est PAS dans 14-2

- ❌ **Motif sur la clôture** : `close` reste sans motif (décision D5). 14-2 = réouverture seulement.
- ❌ **Nouvelle colonne / migration** : le motif vit dans l'audit `details_json` (décision D1). Pas de `reopened_at/by/reason` sur `fiscal_years`.
- ❌ **Snapshot de soldes de clôture** : n'existe pas (14-1 virtuel). La réouverture n'invalide rien. La couture snapshot future (issue liée 14-1) devra brancher l'invalidation sur `fiscal_year.reopened` — **documenté**, non implémenté ici.
- ❌ **Affectation du résultat / écritures de clôture** : le modèle virtuel n'en génère pas (14-1).
- ❌ **Changement des gardes d'immutabilité `journal_entries.rs`** : elles lisent le statut vivant → un flip `Closed → Open` ré-active l'édition **automatiquement**, sans y toucher.

## Décisions de conception (tranchées par Guy 2026-07-24, avant dev)

### D1 — Stockage du motif : audit `details_json` uniquement (PAS de migration)

Le motif obligatoire est écrit dans le **journal d'audit** (`audit_log.details_json`), aux côtés du snapshot before/after, **sans aucune colonne dédiée** ni migration :

```json
// audit_log : action="fiscal_year.reopened", entity_type="fiscal_year", entity_id=<fy.id>
{ "before": { "status": "Closed", ... }, "after": { "status": "Open", ... }, "motif": "Correction TVA Q3 oubliée" }
```

Le journal d'audit (rétention 10 ans, CO) est la **source de vérité** du qui/quand/pourquoi. `action VARCHAR(64)` accueille `"fiscal_year.reopened"` (20 car.) ; `details_json JSON NULL` accueille le motif. **Aucune migration** → P3/P5 (Migration breaking policy) **sans objet**.

### D2 — Réouverture = flip guardé `Closed → Open` + audit before/after, sur le modèle de `update_name`

Nouvelle fn repo `fiscal_years::reopen(pool, user_id, company_id, id, motif) -> Result<FiscalYear, DbError>` calquée sur **`update_name` (`fiscal_years.rs:290-371`)**, PAS sur `close` (qui n'a qu'un snapshot direct) :

- `pool.begin()` → `SELECT ... WHERE id=? AND company_id=? FOR UPDATE` capture le `before` (`None → NotFound`).
- **Garde LIFO** (D3) exécutée dans la même transaction, sous verrou.
- `UPDATE fiscal_years SET status='Open' WHERE id=? AND company_id=? AND status='Closed'`.
- `rows_affected == 0` → re-SELECT du statut pour désambiguïser : si `Open` déjà → **`DbError::IllegalStateTransition`** (409, « exercice déjà ouvert ») ; si `None` → `NotFound` ; sinon `Invariant` (défensif).
- fetch `after` → audit `insert_in_tx(build_audit_entry(user_id, "fiscal_year.reopened", id, json!({"before":.., "after":.., "motif":..})))` → `commit`.
- **Verrou `FOR UPDATE`** obligatoire (comme `close` et `journal_entries`) pour ne pas courir avec une clôture/édition en vol. Ordre de verrou respecté : `companies → projects → fiscal_years` (Pattern 5, `journal_entries.rs:176-179`).

### D3 — Garde-fou d'ordre : LIFO strict (tout exercice postérieur clos bloque)

La réouverture est **refusée** s'il existe **au moins un exercice de `start_date` strictement supérieure ET de statut `Closed`** dans la même société. Conséquence : on rouvre **du plus récent clos vers l'ancien** (il faut rouvrir FY2025 avant FY2024 si les deux sont clos). Préserve la chaîne d'immutabilité — un exercice clos « scelle » les soldes que les exercices antérieurs alimentent (report à-nouveau cumulatif 14-1).

- Nouvelle query repo `find_later_closed_in_tx(&mut tx, company_id, start_date) -> Option<FiscalYear>` : `WHERE company_id=? AND start_date > ? AND status='Closed' ORDER BY start_date ASC LIMIT 1 FOR UPDATE` (miroir du pattern `FOR UPDATE` de `find_overlapping` `:491-511`). Aucune query « strictement après » n'existe aujourd'hui.
- Si un tel exercice existe → **`DbError::IllegalStateTransition`** (409) avec message FR-CH générique (« Réouverture impossible : un exercice postérieur est clos ; rouvrez-le d'abord. » via `t()`). Le **frontend désactive** le bouton (tooltip nommant l'exercice bloquant) en calculant côté client depuis la liste des exercices — le 409 serveur est le **filet** (défense en profondeur), rarement atteint depuis l'UI.

### D4 — Admin uniquement (plus strict que la clôture) + motif obligatoire non vide

- La clôture est **Comptable+** (`comptable_routes`). La réouverction est **Admin uniquement** : route enregistrée dans le sous-routeur `admin_routes` scellé par `require_admin_role` (`lib.rs:182-271`). **Aucun** check de rôle dans le handler (fait au router-layer). `AppError::Forbidden` → 403 pour Comptable/Consultation.
- **Motif obligatoire** : validation `let motif = req.motif.trim().to_string(); if motif.is_empty() { return Err(AppError::Validation(t("error-fiscal-year-reopen-motif-empty", ...))); }` → **400 `VALIDATION_ERROR`** (miroir exact de `fiscal_years.rs:186-193`). Le frontend pré-valide (bouton de confirmation désactivé tant que le motif est vide).

### D5 — `ensure_not_pat` : réouverture interdite via clé API (PAT)

La réouverture d'un exercice clos est une opération **privilégiée et sensible** (réglementaire CO). À l'instar de `admin/full-export` (`routes/admin.rs:37`), le handler appelle **`ensure_not_pat(&current_user)?`** (`api_keys.rs:95-100`) → `AppError::ApiKeyManagementForbidden` si l'appel vient d'une clé API. Corollaire : l'acteur d'audit est **toujours un utilisateur humain** → `build_audit_entry` (constructeur `::user`) reste correct, pas besoin du chemin `for_actor`.

### D6 — `close` inchangé ; message d'« interdiction » reformulé

- **`close` n'est PAS modifié** (reste le verrou Comptable+ sans motif).
- La branche `fiscal_years.rs:599-601` (aujourd'hui *« réouverture interdite »* quand `close` retrouve un exercice déjà clos) est **conservée** — elle sert d'idempotence de `close` (re-clore un exercice clos = no-op erreur). Son **message est reformulé** pour ne plus prétendre que la réouverture est impossible (elle l'est via `reopen`), p.ex. *« exercice {id} déjà clos »*. La **doc module `fiscal_years.rs:3-16`** et le doc `errors.rs:37` (qui donnent « réouverture pas autorisée » en exemple) sont **mis à jour**.

## Acceptance Criteria

### A. Backend — fn repo `reopen` (flip guardé + audit motif)

- **Given** un exercice `Closed`, **When** `fiscal_years::reopen(pool, user_id, company_id, id, motif)` est appelé, **Then** le statut passe à `Open`, une entrée d'audit **`fiscal_year.reopened`** est écrite dans la **même transaction** avec `details_json = { before, after, motif }`, et l'exercice mis à jour est retourné.
- **And** l'appel prend un verrou `FOR UPDATE` sur la ligne `fiscal_years` (capture `before`), cohérent avec `close`/`journal_entries` (anti-course), ordre de verrou `companies → projects → fiscal_years` respecté.
- **And** réouvrir un exercice **déjà `Open`** → `DbError::IllegalStateTransition` (aucune écriture d'audit ; transaction annulée).
- **And** exercice inexistant / autre société → `DbError::NotFound`.

### B. Backend — garde LIFO (D3)

- **Given** un exercice `Closed` **et** un exercice de `start_date` strictement postérieure **également `Closed`** (même société), **When** on tente de rouvrir le premier, **Then** `DbError::IllegalStateTransition` (message FR-CH générique) ; **aucun** flip, **aucun** audit.
- **And** si l'exercice postérieur est `Open` (ou n'existe pas), la réouverture **réussit**.
- **And** la garde s'exécute **dans la transaction** sous `FOR UPDATE` (pas de fenêtre de course avec une clôture concurrente).

### C. Backend — immutabilité ré-activée par le flip (sans toucher `journal_entries`)

- **Given** un exercice rouvert (`Open`), **When** on crée / modifie / supprime une écriture datée dans cet exercice, **Then** l'opération **réussit** (les gardes `journal_entries.rs` lisent le statut vivant `Open`) — **aucune** modification de `journal_entries.rs` n'est requise.
- **And** l'exercice rouvert peut être **re-clôturé** via `close` existant (re-verrou + audit `fiscal_year.closed`).
- **Régression** : un exercice resté `Closed` bloque toujours l'édition (`DbError::FiscalYearClosed`) — inchangé.

### D. API — endpoint Admin-only `POST /api/v1/fiscal-years/{id}/reopen`

- **Given** un utilisateur **Admin** authentifié (pas via PAT), **When** `POST /api/v1/fiscal-years/{id}/reopen` avec body `{ "motif": "…" }` non vide, **Then** **200** + `FiscalYearResponse` (statut `Open`).
- **And** route enregistrée dans `admin_routes` (`lib.rs:182-268`, scellé `require_admin_role`) — **Comptable/Consultation → 403 `FORBIDDEN`**, non-authentifié → **401**.
- **And** appel via **clé API (PAT)** → **403** (`ensure_not_pat`, D5).
- **And** motif absent / vide (après `trim`) → **400 `VALIDATION_ERROR`**.
- **And** exercice d'une autre société / inexistant → **404** (pré-check `find_by_id_in_company`, anti-énumération, miroir `close_fiscal_year:242-259`).
- **And** exercice déjà `Open` → **409 `ILLEGAL_STATE_TRANSITION`**.
- **And** garde LIFO violée (exercice postérieur clos) → **409 `ILLEGAL_STATE_TRANSITION`** (message générique localisé serveur).
- **And** nouveau DTO `ReopenFiscalYearRequest { motif: String }` (camelCase `motif`), `#[serde]`.

### E. Frontend — bouton Réouvrir (Admin-only) + modal motif obligatoire

- **Given** la page `settings/fiscal-years`, **When** un exercice est `Closed` **et** l'utilisateur est **Admin**, **Then** un bouton **Réouvrir** (icône `LockOpen`/`Unlock`) apparaît dans la cellule d'actions (branche `{#if fy.status === 'Closed'}` + `{#if isAdmin}`), **distinct** du gating `canMutate` (qui inclut Comptable).
- **And** cliquer ouvre un **modal** exigeant un **motif** : textarea requise, bouton de confirmation **désactivé tant que le motif est vide** (`clientError` dérivé + `disabled={submitting || !!clientError}`), non-fermable pendant l'envoi, erreur inline — modelé sur `DunningPauseDialog.svelte` (présentation, parent-owns-API) + mécanique champ-requis de `ManualReminderDialog.svelte`.
- **And** la confirmation appelle `reopenFiscalYear(id, { motif })` ; succès → toast `fiscal-year-reopened` + rechargement de la liste (hors try/catch, comme `submitClose:166-201`) ; échec serveur → message serveur affiché.
- **And garde LIFO côté client** : si un exercice de `startDate` postérieure est `Closed`, le bouton Réouvrir est **désactivé** avec un tooltip nommant l'exercice bloquant (`i18nMsg` avec variable `{ $name }`) — calculé depuis la liste déjà chargée.
- **And** un utilisateur **Comptable/Consultation** ne voit **pas** le bouton Réouvrir.

### F. Frontend — types & API client

- `fiscal-years.types.ts` : `ReopenFiscalYearRequest { motif: string }`.
- `fiscal-years.api.ts` : `reopenFiscalYear(id, req)` → `apiClient.post(`/api/v1/fiscal-years/${id}/reopen`, req)`.
- `isAdmin = $derived(authState.currentUser?.role === 'Admin')` ajouté à la page (miroir `settings/invoicing/+page.svelte:65`).

### G. i18n (4 locales) — hardcode interdit

- Nouvelles clés dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, convention `fiscal-year-*` (bloc `:657-686`) : `fiscal-year-reopen-button`, `fiscal-year-reopen-confirmation-title`, `fiscal-year-reopen-confirmation-body` (variable `{ $name }`), `fiscal-year-reopen-motif-label`, `fiscal-year-reopen-confirmation-action`, `fiscal-year-reopened` (toast), `fiscal-year-reopen-blocked-later-closed` (tooltip client, variable `{ $name }`), `error-fiscal-year-reopen-motif-empty`, `error-fiscal-year-already-open`, `error-fiscal-year-reopen-blocked` (message serveur générique LIFO). **4 locales synchronisées.**

### H. Tests & gate

- **Repo** (`crates/kesh-db/tests/fiscal_years_repository.rs`) :
  - reopen d'un `Closed` → `Open` + audit `fiscal_year.reopened` présent avec `details_json.motif` == motif fourni et `before/after` corrects (interroger `audit_log`).
  - reopen d'un `Open` → `IllegalStateTransition`, **aucune** ligne d'audit ajoutée.
  - reopen d'un id d'une autre société → `NotFound`.
  - **LIFO** : FY_N clos + FY_{N+1} clos → reopen FY_N **bloqué** ; reopen FY_{N+1} d'abord → OK, puis FY_N → OK.
  - **LIFO permissif** : FY_N clos + FY_{N+1} **ouvert** → reopen FY_N **OK**.
  - **Immutabilité ré-activée** : après reopen, `journal_entries::create/update/delete` sur une écriture de l'exercice **réussit** ; puis re-`close` → re-bloqué (`FiscalYearClosed`). *(Fix structurel : prouve que le statut vivant pilote l'immutabilité, pas de flag dupliqué.)*
- **E2E** (`crates/kesh-api/tests/fiscal_years_e2e.rs`) : Admin+motif → 200 ; Comptable → 403 ; Consultation → 403 ; sans auth → 401 ; PAT → 403 ; motif vide → 400 ; cross-tenant → 404 ; déjà ouvert → 409 ; LIFO → 409 ; réponse `FiscalYearResponse` statut `Open`.
- **Frontend Vitest** : bouton Réouvrir visible **seulement** si `Closed` **et** Admin ; désactivé (tooltip) si exercice postérieur clos ; modal confirmation désactivé tant que motif vide ; submit appelle `reopenFiscalYear`.
- **Gate Test Locally First** complet vert (backend fmt/build/clippy/test + frontend check/lint-i18n/test/build). **Pas de migration.**

### I. Doc-sync

- **CHANGELOG** `[Non publié]` : la réouverture d'un exercice clôturé est désormais possible (Admin, motif, audit, garde LIFO) — **inverse** l'ancienne règle « un exercice clôturé ne peut pas être ré-ouvert ».
- **Manuel admin** (`docs/manual/fr/admin-manual.tex`, §« Configuration des exercices comptables » `:1324` + tableau des rôles `:1336`) : documenter la réouverture Admin-only + motif + garde LIFO + trace d'audit.
- **Doc module** `fiscal_years.rs:3-16` + doc `errors.rs:37` mis à jour (ne plus affirmer « réouverture interdite »).

## Tasks / Subtasks

- [ ] **T1 — Repo `fiscal_years::reopen`** (`fiscal_years.rs`, nouvelle fn calquée sur `update_name:290-371`) : tx + `FOR UPDATE` before, garde LIFO (T2), `UPDATE ... status='Open' WHERE status='Closed'`, désambiguïsation `rows_affected==0` (déjà Open → `IllegalStateTransition` ; None → `NotFound`), audit `fiscal_year.reopened` `json!({before, after, motif})` via `build_audit_entry`/`insert_in_tx`, commit. — AC-A.
- [ ] **T2 — Query LIFO** `find_later_closed_in_tx(&mut tx, company_id, start_date)` (`WHERE start_date > ? AND status='Closed' ... LIMIT 1 FOR UPDATE`, miroir `find_overlapping`) + branchement dans `reopen` → `IllegalStateTransition` si présent. — AC-B.
- [ ] **T3 — Reformuler `close`/docs** : message `fiscal_years.rs:599-601` (« déjà clos », sans « réouverture interdite ») ; doc module `:3-16` ; doc `errors.rs:37`. **`close` non modifié fonctionnellement.** — AC/D6.
- [ ] **T4 — API handler `reopen_fiscal_year`** (`routes/fiscal_years.rs`, miroir `close_fiscal_year:242-259`) : `ensure_not_pat` (D5), pré-check `find_by_id_in_company` (404), validation motif non vide → `AppError::Validation` (400, miroir `:186-193`), appel `reopen`, `Json<FiscalYearResponse>`. Nouveau DTO `ReopenFiscalYearRequest { motif }`. — AC-D.
- [ ] **T5 — Route Admin-only** : enregistrer `POST /api/v1/fiscal-years/{id}/reopen` dans le bloc **`admin_routes` (`lib.rs:182-268`)**, scellé par `require_admin_role` (`:269-271`). **PAS** dans `comptable_routes`. — AC-D.
- [ ] **T6 — Frontend** : `ReopenFiscalYearRequest` (types) + `reopenFiscalYear` (api) ; page `settings/fiscal-years/+page.svelte` — `isAdmin` derived, bouton Réouvrir (`{#if fy.status==='Closed'}` + `{#if isAdmin}`, désactivé + tooltip si LIFO client), état + `submitReopen()` (clone `submitClose:166-201`), modal motif (mirror `DunningPauseDialog` + champ requis `ManualReminderDialog`). — AC-E/F.
- [ ] **T7 — i18n** : 10 clés `fiscal-year-reopen-*` / `error-fiscal-year-*` × **4 locales** (AC-G).
- [ ] **T8 — Tests** : repo (`fiscal_years_repository.rs`) + e2e (`fiscal_years_e2e.rs`) + Vitest frontend (cf. AC-H). Chaque patch avec son test (`feedback_review_patch_needs_test`).
- [ ] **T9 — Doc-sync** : CHANGELOG + admin-manual.tex + docs modules (AC-I). Créer une **issue snapshot-invalidation** (ou compléter l'issue snapshot existante de 14-1) notant que `fiscal_year.reopened` devra invalider un futur snapshot ; rien à coder en v1 virtuel. **Pas de migration.**

## Dev Notes

### Ancres ground-truth (vérifiées 2026-07-24 par 3 agents de cartographie)

**Backend — repo & entités**
- `crates/kesh-db/src/entities/fiscal_year.rs` : `FiscalYearStatus` (`:17-24`, `Open`/`Closed`, serde PascalCase, `as_str:26-33`, `FromStr:35-44`) ; struct `FiscalYear` (`:72-83`).
- `crates/kesh-db/src/repositories/fiscal_years.rs` : `close` (`:568-625` ; guard SQL `:576-585` ; audit `:617-621` ; branche « déjà clos » `:599-601`) ; **`update_name` = miroir** (`:290-371`, autorisé sur Closed, before/after audit `:359-367`) ; `snapshot_json` (`:64-75`) ; `build_audit_entry(user_id, action, entity_id, details)` (`:79-86`, entity_type hardcodé `"fiscal_year"`) ; `find_overlapping` (`:491-511`, pattern `FOR UPDATE`) ; `list_by_company` (`:541-551`) ; module doc (`:3-16`, à MAJ). Actions existantes : `fiscal_year.created/updated/closed`. **Aucun** `reopened` (grep 0).
- **Immutabilité (NE PAS toucher)** `crates/kesh-db/src/repositories/journal_entries.rs` : gardes statut vivant `FOR UPDATE` — create `:189-201`, update `:707-735`, delete `:970-991` → `DbError::FiscalYearClosed`. Ordre de verrou `companies → projects → fiscal_years` (`:176-179`).
- **Audit** `crates/kesh-db/src/repositories/audit_log.rs` : `insert_in_tx(tx, NewAuditLogEntry)` (`:29-75`, prend une tx, ne commit jamais). `crates/kesh-db/src/entities/audit_log.rs` : `NewAuditLogEntry` (`:119-130` : `user_id, action, entity_type, entity_id, details_json: Option<Value>, actor_type, actor_api_key_id`), `::user` (`:136-152`), `ActorType` (`:25-30`). DDL `migrations/20260413000001_audit_log.sql:13-26` (`action VARCHAR(64)`, `details_json JSON NULL`, rétention, pas de delete).
- **Erreurs** `crates/kesh-db/src/errors.rs` : `IllegalStateTransition(String)` (`:39` → 409 `ILLEGAL_STATE_TRANSITION` ; doc `:37` à nuancer) ; `NotFound` (`:18` → 404) ; `Invariant(String)` (`:136`, défensif) ; `FiscalYearClosed` (`:46`, variant immutabilité).

**API**
- `crates/kesh-api/src/routes/fiscal_years.rs` : `close_fiscal_year` (`:242-259`, miroir handler) ; DTOs (`:39-80`) ; **validation non-vide** (`:186-193`, `AppError::Validation → 400`). `use crate::errors::{AppError, t}` (`:32`).
- `crates/kesh-api/src/lib.rs` : **`admin_routes`** (`:182-268`) scellé `require_admin_role` (`:269-271`) — y ajouter la route reopen ; `close` en `comptable_routes` (`:475-478`) sous `require_comptable_role` (`:571-572`).
- `crates/kesh-api/src/middleware/rbac.rs` : `check_role` (`:19`), `require_admin_role` (`:31`). `Role` enum `crates/kesh-db/src/entities/user.rs:20-27` (Admin 2 > Comptable 1 > Consultation 0).
- `crates/kesh-api/src/errors.rs` : `AppError::Forbidden` (`:83` / `:889-893` → 403 `FORBIDDEN`) ; `AppError::Validation` (`:66` / `:866-868` → 400 `VALIDATION_ERROR`).
- **PAT** `crates/kesh-api/src/routes/api_keys.rs:95-100` `ensure_not_pat` ; `crates/kesh-api/src/middleware/auth.rs:44` `CurrentUser.api_key_id` ; précédent `routes/admin.rs:37`.

**Frontend**
- Page **`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`** : `canMutate` (`:56-58`, Admin **ou** Comptable) ; bouton close (`:263-272`, `{#if fy.status==='Open'}`) ; dialog close (`:394-432`) ; état (`:50-53`) ; `submitClose` (`:166-201`, gère `ILLEGAL_STATE_TRANSITION`, reload hors try/catch). **Pas de `+page.ts` guard** (page reste Comptable-accessible → gating in-component).
- `frontend/src/lib/features/fiscal-years/fiscal-years.api.ts:33` (`closeFiscalYear`, `apiClient.post`) ; `…/fiscal-years.types.ts` (`FiscalYearStatus:8`, `FiscalYearResponse:10-21`, requests `:23-33`) ; `…/fiscal-years.helpers.ts:22` (validation → clé i18n).
- Rôle courant : `frontend/src/lib/app/stores/auth.svelte.ts:83/44` (`authState.currentUser?.role`) ; `isAdmin` pattern `frontend/src/routes/(app)/settings/invoicing/+page.svelte:65`.
- Modals à mirror : `frontend/src/lib/features/reminders/DunningPauseDialog.svelte` (présentation, parent-owns-API, reset-on-open, erreur inline) + `frontend/src/lib/features/reminders/ManualReminderDialog.svelte` (`clientError` dérivé + `disabled={submitting || !!clientError}`).
- i18n `crates/kesh-i18n/locales/fr-CH/messages.ftl:657-686` (bloc `fiscal-year-*`) ; helper `i18nMsg(key, fallback, vars?)`.

**Tests & docs**
- `crates/kesh-db/tests/fiscal_years_repository.rs` ; `crates/kesh-api/tests/fiscal_years_e2e.rs`. Pas de test frontend fiscal-years existant (à créer).
- `docs/manual/fr/admin-manual.tex:1324` (§exercices), `:1336` (rôles) ; `CHANGELOG.md`.

### Pièges, par ordre de coût

1. **Ne PAS toucher `journal_entries.rs`** : les 3 gardes lisent le statut vivant → le flip ré-active l'édition « gratuitement ». Le prouver par test (AC-C), pas par modification.
2. **Verrou `FOR UPDATE`** dans `reopen` **et** la query LIFO : sinon course avec une clôture/édition concurrente (l'équation/immutabilité repose sur la sérialisation). Ordre `companies → projects → fiscal_years`.
3. **Motif dans l'audit uniquement** (D1) : `json!({"before":.., "after":.., "motif":..})`. Ne PAS ajouter de colonne. Le test lit `audit_log.details_json`.
4. **Admin-only au router-layer** (D4) : route dans `admin_routes`, **jamais** de check de rôle dans le handler. Ne pas la mettre par erreur dans `comptable_routes` (où vit `close`).
5. **`ensure_not_pat`** (D5) avant tout : la réouverction n'est pas une opération d'automatisation.
6. **Garde LIFO = `start_date >` strict** (D3) : borne stricte, statut `Closed`. Frontend désactive proactivement ; serveur = filet 409.
7. **`close` idempotence** : la branche `:599-601` reste (re-clore un exercice clos = erreur), seul son **message** change (D6). Ne pas la supprimer.
8. **`already-open` vs `LIFO`** partagent `IllegalStateTransition` (409) : les distinguer par **message serveur** ; le frontend affiche `err.message` sur échec du modal (les deux messages sont localisés serveur via `t()`).

### Limitations documentées (catégorie B)

- **L1 — motif non requêtable en 1er plan** : le motif vit dans `audit_log.details_json` (D1, décision Guy). Le retrouver = requête sur l'audit (source de vérité 10 ans). Une future colonne dédiée = migration hors scope si un besoin de reporting émerge.
- **L2 — messages serveur `already-open` / `LIFO` non code-distincts** : les deux sont `IllegalStateTransition` (409) ; la distinction est **textuelle** (message serveur localisé). Suffisant v0.1 (le frontend désactive proactivement le cas LIFO). Un code d'erreur dédié = amélioration future si un client API a besoin de brancher dessus.
- **L3 — invalidation snapshot différée** : aucun snapshot n'existe (14-1 virtuel), donc rien à invalider. Quand la couture snapshot de clôture sera livrée, elle DEVRA écouter `fiscal_year.reopened` — tracé dans l'issue snapshot, hors scope 14-2.

### References

- Conception : note léguée **14-1:43/52/156** (interaction réouverture ↔ snapshot, ancre `fiscal_years.rs:600`) ; sprint-status (scope Admin+motif+audit+garde d'ordre) ; décisions Guy 2026-07-24 (D1 audit-only, D3 LIFO strict).
- Norme : CO art. 957-964 (immutabilité + rétention 10 ans ; la réouverction tracée reste conforme — la piste d'audit prime).
- Conventions : CLAUDE.md § Test Locally First, § Review Iteration Rule, § Règle de commit, § Issue Tracking Rule.

## Change Log — create-story

Spec créée 2026-07-24 par cartographie ground-truth parallèle (3 agents Explore : backend clôture/audit/immutabilité, API+RBAC, frontend/modal). L'Epic 14 de `epics.md` (« Justificatifs, Lettrage ») est **périmé** — le scope 14-2 vient de la note sprint-status + 14-1 + décisions Guy. 2 forks tranchés par Guy avant rédaction : **D1** motif → audit `details_json` seul (pas de migration) ; **D3** garde d'ordre **LIFO strict** (tout exercice postérieur clos bloque). Décisions dérivées : D2 (fn `reopen` calquée `update_name`, flip guardé + audit before/after), D4 (Admin-only router-layer + motif obligatoire), D5 (`ensure_not_pat`), D6 (`close` inchangé, message reformulé). **Pas de migration** (P3/P5 sans objet). Prochaine étape : `bmad-create-story validate`.
