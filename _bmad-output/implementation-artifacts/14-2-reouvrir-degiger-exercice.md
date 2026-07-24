# Story 14.2 : Réouverture d'un exercice clôturé (verrou réversible tracé)

## Status

review

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
- **Désambiguïsation « déjà ouvert » AVANT la garde LIFO** (finding P1-M4) : `before.status == Open` → retour anticipé `DbError::Invariant(FY_REOPEN_ALREADY_OPEN_KEY)` (évite d'attribuer à tort un cas « déjà ouvert » au message LIFO **et** évite une query LIFO superflue).
- **Garde LIFO** (D3) exécutée ensuite, dans la même transaction, sous verrou.
- `UPDATE fiscal_years SET status='Open' WHERE id=? AND company_id=? AND status='Closed'`.
- `rows_affected == 0` (statut changé après le `before` = course) → re-SELECT : `Open` → `Invariant(FY_REOPEN_ALREADY_OPEN_KEY)` ; `None` → `NotFound` ; sinon `Invariant(FY_REOPEN_UNEXPECTED_KEY)` (défensif).
- fetch `after` → audit `insert_in_tx(build_audit_entry(user_id, "fiscal_year.reopened", id, json!({"before":.., "after":.., "motif":..})))` → `commit`.
- **Mapping des deux 409 distincts** : voir **D7**. Les deux causes de conflit (`already-open`, `LIFO`) sont émises comme `DbError::Invariant(KEY)` namespacés puis re-mappées en 409 à **message distinct** — le générique `DbError::IllegalStateTransition` est **log-only** (`kesh-api/errors.rs:2124`, jamais renvoyé au client) et n'est donc **pas** utilisé par `reopen`.
- **Verrou `FOR UPDATE`** obligatoire (comme `close`/`journal_entries`) pour ne pas courir avec une clôture/édition en vol. `reopen` ne verrouille **que** `fiscal_years` (+ insert `audit_log`) — table **isolée**, aucune chaîne cross-table (doc module `fiscal_years.rs:9-16`) ; la mention Pattern 5 « `companies → projects → fiscal_years` » (`journal_entries.rs:176-179`) est **sans objet ici** (aucune de ces tables n'est touchée) et retirée pour ne pas induire en erreur (finding P1-LOW).

### D3 — Garde-fou d'ordre : LIFO strict (tout exercice postérieur clos bloque)

La réouverture est **refusée** s'il existe **au moins un exercice de `start_date` strictement supérieure ET de statut `Closed`** dans la même société. Conséquence : on rouvre **du plus récent clos vers l'ancien** (il faut rouvrir FY2025 avant FY2024 si les deux sont clos). Objectif : imposer un **ordre de réouverture** cohérent (on ne « dé-scelle » pas un ancien exercice en laissant un plus récent clos au-dessus).

> **Portée exacte de la garantie (finding P1-H2, décision Gui 2026-07-24 = « documenter la limitation ») :** cette garde discipline **seulement la réouverture**. Elle n'ajoute **aucune** garde symétrique sur `close` (D6 : `close` non modifié). Un `close` peut donc toujours clore un exercice alors qu'un exercice **antérieur** est `Open` (comportement **pré-existant**, pas introduit par 14-2). En modèle virtuel 14-1 (report à-nouveau recalculé en direct, **aucun snapshot**), cette asymétrie **ne corrompt rien** — cf. limitation **L4**. Une garde FIFO symétrique sur `close` deviendra pertinente **si/quand** la couture snapshot de clôture est livrée.

- Nouvelle query repo `find_later_closed_in_tx(&mut tx, company_id, start_date) -> Option<FiscalYear>` : `WHERE company_id=? AND start_date > ? AND status='Closed' ORDER BY start_date ASC LIMIT 1 FOR UPDATE` (miroir du pattern `FOR UPDATE` de `find_overlapping` `:491-511`). Aucune query « strictement après » n'existe aujourd'hui. **Note verrouillage** : le filtre `status='Closed'` n'est pas indexé (seul `uq_fiscal_years_company_start_date (company_id, start_date)` l'est) → le `FOR UPDATE` s'appuie sur le next-key locking InnoDB du range scan (hypothèse à documenter + tester, cf. AC-H course concurrente / finding P1-M6).
- Si un tel exercice existe → `DbError::Invariant(FY_REOPEN_LIFO_BLOCKED_KEY)` → re-mappé en **409** (`ILLEGAL_STATE_TRANSITION`) avec message FR-CH **distinct** (« Réouverture impossible : un exercice postérieur est clos ; rouvrez-le d'abord. » via `t("error-fiscal-year-reopen-blocked")`, cf. D7). Le **frontend désactive** le bouton (tooltip nommant l'exercice bloquant) en calculant côté client depuis la liste des exercices — le 409 serveur est le **filet** (défense en profondeur), rarement atteint depuis l'UI.

### D4 — Admin uniquement (plus strict que la clôture) + motif obligatoire non vide

- La clôture est **Comptable+** (`comptable_routes`). La réouverture est **Admin uniquement** : route enregistrée dans le sous-routeur `admin_routes` (bloc `.route(...)` `lib.rs:182-268`) scellé par `require_admin_role` (`.route_layer(...)` `lib.rs:269-271`). **Aucun** check de rôle dans le handler (fait au router-layer). `AppError::Forbidden` → 403 pour Comptable/Consultation.
- **Motif obligatoire ET borné** : `let motif = req.motif.trim().to_string();`
  - `if motif.is_empty() { return Err(AppError::Validation(t("error-fiscal-year-reopen-motif-empty", ...))); }` → **400 `VALIDATION_ERROR`** (miroir exact de `fiscal_years.rs:186-193`).
  - `if motif.chars().count() > REOPEN_MOTIF_MAX { return Err(AppError::Validation(t("error-fiscal-year-reopen-motif-too-long", ...))); }` → **400** (finding P1-M5). `const REOPEN_MOTIF_MAX: usize = 500;` — **borne miroir client+serveur** sur le modèle exact de `PAUSE_NOTE_MAX = 500` (`dunning_reminders.rs:133`/`201` + `DunningPauseDialog.svelte:20`), pour ne pas laisser un motif de plusieurs Mo grossir `audit_log` (rétention 10 ans, sans purge).
  - Le frontend pré-valide (bouton de confirmation désactivé si motif vide **ou** > `REOPEN_MOTIF_MAX` ; `maxlength` sur la textarea).
  - **Note asymétrie (P2-M1)** : `chars().count()` (Rust, graphèmes ~ scalar values) et le `maxlength` HTML (UTF-16 code units) peuvent diverger sur emoji/multi-octets. Asymétrie mineure **acceptée** (identique au précédent `PAUSE_NOTE_MAX`, `crates/kesh-api/src/routes/dunning_reminders.rs:133`, sans incident) ; le serveur reste l'autorité (400 si dépassement) — le `maxlength` client n'est qu'un garde-fou UX.
  - **Validation au handler assumée (finding P3-Opus2)** : contrairement à `update_name` qui valide **au repo** (`fiscal_years.rs:298-306`), 14-2 valide le motif **au handler** (miroir de `close_fiscal_year`, `:186-193`). Justifié : caller de production **unique** = la route Admin ; le handler garde vide+longueur en amont ; le repo `reopen` écrit `motif` tel quel dans `details_json`. **Écart au miroir `update_name` documenté** pour que le dev ne « corrige » pas par erreur vers une garde repo. ⚠️ Si un jour un caller **non-route** apparaît (batch applicatif, seed métier), déplacer la garde au repo (émettre `FY_REOPEN_MOTIF_EMPTY_KEY`/`FY_REOPEN_MOTIF_TOO_LONG_KEY` en `Invariant`, aligné `update_name`) — `details_json JSON` n'a **aucun** filet de longueur DB.

### D5 — `ensure_not_pat` : réouverture interdite via clé API (PAT)

La réouverture d'un exercice clos est une opération **privilégiée et sensible** (réglementaire CO). À l'instar de `admin/full-export` (`routes/admin.rs:37`), le handler appelle **`ensure_not_pat(&current_user)?`** (`api_keys.rs:95-100`) → `AppError::ApiKeyManagementForbidden` si l'appel vient d'une clé API. Corollaire : l'acteur d'audit est **toujours un utilisateur humain** → `build_audit_entry` (constructeur `::user`) reste correct, pas besoin du chemin `for_actor`.

### D6 — `close` inchangé ; message d'« interdiction » reformulé

- **`close` n'est PAS modifié** (reste le verrou Comptable+ sans motif).
- La branche `fiscal_years.rs:599-601` (aujourd'hui *« réouverture interdite »* quand `close` retrouve un exercice déjà clos) est **conservée** — elle sert d'idempotence de `close` (re-clore un exercice clos = no-op erreur). Son **message est reformulé** pour ne plus prétendre que la réouverture est impossible (elle l'est via `reopen`), p.ex. *« exercice {id} déjà clos »*. La **doc module `fiscal_years.rs:3-16`** et le doc `errors.rs:37` (qui donnent « réouverture pas autorisée » en exemple) sont **mis à jour**.

### D7 — Deux 409 distincts : `Invariant(KEY)` namespacé + `map_reopen_error` (finding P1-C1)

**Problème** : `DbError::IllegalStateTransition(String)` a son `Display` **log-only** (`kesh-api/errors.rs:2124` : `tracing::warn!` + message figé `t("error-illegal-state", "Transition d'état interdite")`). Le texte porté par la variante **n'atteint jamais le client** (contrat `kesh-db/errors.rs:8-9` : « Ne jamais exposer le `Display` au frontend »). Émettre `IllegalStateTransition("déjà ouvert")` vs `IllegalStateTransition("postérieur clos")` produirait donc **le même** message générique → les clés i18n dédiées seraient **mortes**.

**Décision Gui 2026-07-24 = « messages distincts, code partagé »** — reproduire le pattern **déjà en place** dans ce module (`FY_NAME_DUPLICATE_KEY` + `map_create_error`/`map_update_error`, `fiscal_years.rs:87-142`) :

1. **Constantes clés namespacées** (repo `fiscal_years.rs`, à côté de `FY_NAME_DUPLICATE_KEY`) : `FY_REOPEN_ALREADY_OPEN_KEY`, `FY_REOPEN_LIFO_BLOCKED_KEY` (+ `FY_REOPEN_UNEXPECTED_KEY` défensif). `reopen` émet `DbError::Invariant(<KEY>)`.
2. **Nouveau variant `AppError::IllegalState(String)`** (`kesh-api/errors.rs`) → mappé **409** code `ILLEGAL_STATE_TRANSITION` mais avec le **message passé** (déjà localisé par `t()` au niveau du mapper). Porter un message **déjà localisé** dans le `String` d'un variant `AppError` est **cohérent avec le précédent `AppError::Validation`** (que `map_create_error` construit déjà via `AppError::Validation(t("error-...", ...))`, `fiscal_years.rs:89-105`) — ce n'est pas une rupture de l'invariant « String = détail brut » (finding P2-M2). Réutilisable, distinct du générique `DbError::IllegalStateTransition` (inchangé, conservé pour `close`).
3. **`map_reopen_error`** (route `routes/fiscal_years.rs`, miroir `map_create_error`) : `Invariant(k) if k == FY_REOPEN_ALREADY_OPEN_KEY → AppError::IllegalState(t("error-fiscal-year-already-open", ...))` ; `… == FY_REOPEN_LIFO_BLOCKED_KEY → AppError::IllegalState(t("error-fiscal-year-reopen-blocked", ...))` ; tout autre `Invariant` **retombe** vers le mapping global (→ 500 `INTERNAL_ERROR`, défensif, cohérent `FY_REOPEN_UNEXPECTED_KEY`).

**Conséquence** : les deux cas restent **409 `ILLEGAL_STATE_TRANSITION`** (code machine **partagé** — un code d'erreur *distinct* reste future work, cf. L2) mais avec des **messages utilisateur distincts** et localisés. Le code partagé + message distinct est exactement ce que veulent D3/§G sans sur-ingénierie. **Les tests e2e assertent le *contenu* du message** (pas seulement le code HTTP), sinon le défaut C1 repasserait silencieusement la gate (finding P1-C1).

## Acceptance Criteria

### A. Backend — fn repo `reopen` (flip guardé + audit motif)

- **Given** un exercice `Closed`, **When** `fiscal_years::reopen(pool, user_id, company_id, id, motif)` est appelé, **Then** le statut passe à `Open`, une entrée d'audit **`fiscal_year.reopened`** est écrite dans la **même transaction** avec `details_json = { before, after, motif }`, et l'exercice mis à jour est retourné.
- **And** l'appel prend un verrou `FOR UPDATE` sur la ligne `fiscal_years` (capture `before`), cohérent avec `close`/`journal_entries` (anti-course). `reopen` ne verrouille que `fiscal_years` (table isolée, cf. D2) — pas de chaîne cross-table (correction contradiction interne P3-F1).
- **And** réouvrir un exercice **déjà `Open`** → `DbError::Invariant(FY_REOPEN_ALREADY_OPEN_KEY)` (re-mappé 409, cf. D7) ; **aucune** écriture d'audit ; transaction annulée. La désambiguïsation « déjà ouvert » est faite **avant** la garde LIFO (P1-M4).
- **And** exercice inexistant / autre société → `DbError::NotFound`.

### B. Backend — garde LIFO (D3)

- **Given** un exercice `Closed` **et** un exercice de `start_date` strictement postérieure **également `Closed`** (même société), **When** on tente de rouvrir le premier, **Then** `DbError::Invariant(FY_REOPEN_LIFO_BLOCKED_KEY)` (re-mappé 409 avec message FR-CH **distinct**, cf. D7) ; **aucun** flip, **aucun** audit.
- **And** si l'exercice postérieur est `Open` (ou n'existe pas), la réouverture **réussit**.
- **And** la garde s'exécute **dans la transaction** sous `FOR UPDATE` (pas de fenêtre de course avec une clôture concurrente).

### C. Backend — immutabilité ré-activée par le flip (sans toucher `journal_entries`)

- **Given** un exercice rouvert (`Open`), **When** on crée / modifie / supprime une écriture datée dans cet exercice, **Then** l'opération **réussit** (les gardes `journal_entries.rs` lisent le statut vivant `Open`) — **aucune** modification de `journal_entries.rs` n'est requise.
- **And** l'exercice rouvert peut être **re-clôturé** via `close` existant (re-verrou + audit `fiscal_year.closed`).
- **And (comportement intentionnel du modèle virtuel 14-1, finding P2-M4)** : éditer une écriture d'un exercice rouvert dont un exercice **postérieur** est `Open` recalcule **en direct** le report à-nouveau de cet exercice postérieur (report CALCULÉ, aucun snapshot — `balance_sheet.rs`). C'est **voulu** (cœur du modèle temps réel virtuel), **pas** une corruption ; cohérent avec L4. Aucune invalidation à déclencher tant que la couture snapshot (L3) n'existe pas.
- **Régression** : un exercice resté `Closed` bloque toujours l'édition (`DbError::FiscalYearClosed`) — inchangé.

### D. API — endpoint Admin-only `POST /api/v1/fiscal-years/{id}/reopen`

- **Given** un utilisateur **Admin** authentifié (pas via PAT), **When** `POST /api/v1/fiscal-years/{id}/reopen` avec body `{ "motif": "…" }` non vide, **Then** **200** + `FiscalYearResponse` (statut `Open`).
- **And** route enregistrée dans `admin_routes` (`lib.rs:182-268`, scellé `require_admin_role`) — **Comptable/Consultation → 403 `FORBIDDEN`**, non-authentifié → **401**.
- **And** appel via **clé API (PAT)** → **403** (`ensure_not_pat`, D5).
- **And** motif absent / vide (après `trim`) → **400 `VALIDATION_ERROR`** (`error-fiscal-year-reopen-motif-empty`).
- **And** motif > `REOPEN_MOTIF_MAX` (500) caractères → **400 `VALIDATION_ERROR`** (`error-fiscal-year-reopen-motif-too-long`, P1-M5).
- **And** exercice d'une autre société / inexistant → **404** (pré-check `find_by_id_in_company`, anti-énumération, miroir `close_fiscal_year:242-259`).
- **And** exercice déjà `Open` → **409 `ILLEGAL_STATE_TRANSITION`** avec message **`error-fiscal-year-already-open`** (contenu distinct, pas le générique — D7).
- **And** garde LIFO violée (exercice postérieur clos) → **409 `ILLEGAL_STATE_TRANSITION`** avec message **`error-fiscal-year-reopen-blocked`** (contenu distinct localisé serveur — D7).
- **And** nouveau DTO `ReopenFiscalYearRequest { motif: String }` (camelCase `motif`), `#[serde]`.

### E. Frontend — bouton Réouvrir (Admin-only) + modal motif obligatoire

- **Given** la page `settings/fiscal-years`, **When** un exercice est `Closed` **et** l'utilisateur est **Admin**, **Then** un bouton **Réouvrir** (icône `LockOpen`/`Unlock`) apparaît dans la cellule d'actions (branche `{#if fy.status === 'Closed'}` + `{#if isAdmin}`), **distinct** du gating `canMutate` (qui inclut Comptable).
- **And** cliquer ouvre un **modal** exigeant un **motif** : textarea requise avec `maxlength={REOPEN_MOTIF_MAX}` (500), bouton de confirmation **désactivé tant que le motif est vide OU > `REOPEN_MOTIF_MAX`** (`clientError` dérivé + `disabled={submitting || !!clientError}`), non-fermable pendant l'envoi, erreur inline — modelé sur `DunningPauseDialog.svelte` (présentation, parent-owns-API, borne `PAUSE_NOTE_MAX`) + mécanique champ-requis de `ManualReminderDialog.svelte`.
- **And** la confirmation appelle `reopenFiscalYear(id, { motif })` ; succès → toast `fiscal-year-reopened` + rechargement de la liste (hors try/catch, comme `submitClose:166-201`) ; échec serveur → message serveur affiché.
- **And garde LIFO côté client** : si un exercice de `startDate` postérieure est `Closed`, le bouton Réouvrir est **désactivé** avec un tooltip nommant l'exercice bloquant — le **plus proche postérieur clos** (`min(startDate)` parmi les postérieurs clos), pour s'aligner sur le serveur `find_later_closed_in_tx ORDER BY start_date ASC LIMIT 1` (finding P3-F4) — via `i18nMsg` avec variable `{ $name }`, calculé depuis la liste déjà chargée.
- **And** un utilisateur **Comptable/Consultation** ne voit **pas** le bouton Réouvrir.

### F. Frontend — types & API client

- `fiscal-years.types.ts` : `ReopenFiscalYearRequest { motif: string }`.
- `fiscal-years.api.ts` : `reopenFiscalYear(id, req)` → `apiClient.post(`/api/v1/fiscal-years/${id}/reopen`, req)`.
- `isAdmin = $derived(authState.currentUser?.role === 'Admin')` ajouté à la page (miroir `settings/invoicing/+page.svelte:65`).

### G. i18n (4 locales) — hardcode interdit

- **11 clés** (P1-M5 ajoute `-motif-too-long`) dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, convention `fiscal-year-*` (bloc `:657-686`) : `fiscal-year-reopen-button`, `fiscal-year-reopen-confirmation-title`, `fiscal-year-reopen-confirmation-body` (variable `{ $name }`), `fiscal-year-reopen-motif-label`, `fiscal-year-reopen-confirmation-action`, `fiscal-year-reopened` (toast), `fiscal-year-reopen-blocked-later-closed` (tooltip client, variable `{ $name }`), `error-fiscal-year-reopen-motif-empty`, `error-fiscal-year-reopen-motif-too-long`, `error-fiscal-year-already-open` (message serveur, **atteint via D7**), `error-fiscal-year-reopen-blocked` (message serveur LIFO, **atteint via D7**). **4 locales synchronisées.** ⚠️ Les deux dernières ne sont vivantes **que** grâce au `map_reopen_error` de D7 — sans lui elles seraient mortes (finding P1-C1).

### H. Tests & gate

- **Repo** (`crates/kesh-db/tests/fiscal_years_repository.rs`) :
  - reopen d'un `Closed` → `Open` + audit `fiscal_year.reopened` présent avec `details_json.motif` == motif fourni et `before/after` corrects (interroger `audit_log`).
  - reopen d'un `Open` → `Invariant(FY_REOPEN_ALREADY_OPEN_KEY)`, **aucune** ligne d'audit ajoutée.
  - reopen d'un id d'une autre société → `NotFound`.
  - **LIFO** : FY_N clos + FY_{N+1} clos → reopen FY_N **bloqué** (`Invariant(FY_REOPEN_LIFO_BLOCKED_KEY)`) ; reopen FY_{N+1} d'abord → OK, puis FY_N → OK.
  - **LIFO permissif** : FY_N clos + FY_{N+1} **ouvert** → reopen FY_N **OK**.
  - **LIFO 3 exercices intercalés (P4-LOW)** : FY1 clos, FY2 **ouvert**, FY3 clos → reopen FY1 **bloqué** en citant FY3 (le plus proche postérieur clos via `ORDER BY start_date ASC LIMIT 1`) ; prouve que la query n'a pas besoin de notion d'adjacence.
  - **Immutabilité ré-activée** : après reopen, `journal_entries::create/update/delete` sur une écriture de l'exercice **réussit** ; puis re-`close` → re-bloqué (`FiscalYearClosed`). *(Fix structurel : prouve que le statut vivant pilote l'immutabilité, pas de flag dupliqué.)*
  - **Course concurrente (P1-M6)** : deux connexions concurrentes (`tokio::join!`) — `reopen(FY_N)` vs `close(FY_{N+1})` — l'issue est déterministe (sérialisée par le `FOR UPDATE`) : soit reopen échoue LIFO, soit close attend puis l'un des deux voit l'état cohérent. Le test **prouve l'absence de la fenêtre** décrite en D3 (note verrouillage) et documente l'hypothèse next-key locking.
- **E2E** (`crates/kesh-api/tests/fiscal_years_e2e.rs`) : Admin+motif → 200 ; Comptable → 403 ; Consultation → 403 ; sans auth → 401 ; PAT → 403 ; motif vide → 400 ; **motif > 500 → 400** (P1-M5) ; cross-tenant → 404 ; **déjà ouvert → 409 ET `error.message` == `error-fiscal-year-already-open` localisé** (pas le générique) ; **LIFO → 409 ET `error.message` == `error-fiscal-year-reopen-blocked`** (P1-C1 : asserter le **contenu**, pas seulement le code) ; réponse `FiscalYearResponse` statut `Open`.
- **Frontend Vitest** : bouton Réouvrir visible **seulement** si `Closed` **et** Admin ; désactivé (tooltip) si exercice postérieur clos ; modal confirmation désactivé tant que motif vide **ou > 500** ; submit appelle `reopenFiscalYear`. **+ (P4-F2)** le dialogue de **clôture** ne rend plus « définitivement » ni « ne pourra plus être enregistré » (assertion texte, 1 locale, couvre clé + fallback).
- **Gate Test Locally First** complet vert (backend fmt/build/clippy/test + frontend check/lint-i18n/test/build). **Pas de migration.**

### I. Doc-sync

- **CHANGELOG** `[Non publié]` : la réouverture d'un exercice clôturé est désormais possible (Admin, motif, audit, garde LIFO) — **inverse** l'ancienne règle « un exercice clôturé ne peut pas être ré-ouvert ».
- **Manuel admin** (`docs/manual/fr/admin-manual.tex`, §« Configuration des exercices comptables » `:1324` + tableau des rôles `:1336`) : documenter la réouverture Admin-only + motif + garde LIFO + trace d'audit.
- **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) — **finding P1-H1** : la section « Réouverture d'un exercice clôturé » (`:464-466`) affirme aujourd'hui à tort qu'un admin rouvre *« via une procédure CLI »* (fausse — aucune CLI n'existe, `admin-manual.tex:2098-2100` le confirme). **La corriger** pour décrire le vrai mécanisme (bouton UI Admin + motif obligatoire + trace d'audit + garde LIFO). Corriger aussi, tant qu'on y est, `:445-457` qui décrit encore des « écritures de clôture » générées (périmé depuis 14-1 virtuel).
- **README** (`README.md:208`, ligne v0.8.0) — **finding P1-M1** : retirer/reformuler la mention *« réouverture d'exercice ... à suivre »* (feature désormais livrée) — **dans le même commit** (règle CLAUDE.md « Synchroniser le planning du README »).
- **Doc module** `fiscal_years.rs:3-16` + doc `errors.rs:37` mis à jour (ne plus affirmer « réouverture interdite »).

### J. Hygiène `close` / doc-message (finding P1-M3, couvre D6)

- **Given** un exercice **déjà `Closed`**, **When** `close` est ré-appelé (idempotence), **Then** l'erreur reste `DbError::IllegalStateTransition` (variant **inchangé**), mais son **message** (log-only) ne prétend plus « réouverture interdite » (reformulé « exercice déjà clos »).
- **And (user-facing, finding P3-F1 MEDIUM)** : le **dialogue de confirmation de clôture** ne doit plus mentir sur l'irréversibilité **absolue**. Mettre à jour, pour **`-body` ET `-action`**, **les 4 locales ET les 2 fallbacks svelte hardcodés** (finding P4-F1) :
  - `fiscal-year-close-confirmation-body` : 4 locales (`fr-CH:671`, `de-CH:640`, `it-CH:640`, `en-CH:640`) **+ fallback `+page.svelte:405`** — retirer *« ne pourra plus être enregistré »* absolu → formulation qui **préserve le garde-fou sans mentir**, p.ex. FR *« Cette action clôture l'exercice : aucune écriture ne pourra plus y être saisie **tant qu'il reste clôturé** ; seul un administrateur peut le rouvrir (motif tracé). Confirmer ? »*.
  - `fiscal-year-close-confirmation-action` : 4 locales (`fr-CH:672` *« Clôturer définitivement »* etc.) **+ fallback `+page.svelte:428`** (finding P4-F1 — oublié au patch P3-F1) → *« Clôturer »*.
  - ⚠️ Les 2 fallbacks svelte sont **atteignables** (i18n chargée en fire-and-forget non bloquant `+layout.svelte:52` → le fallback s'affiche pendant la fenêtre de chargement et si l'API i18n échoue) — les corriger n'est PAS cosmétique.
  - C'est **exactement** la même hygiène anti-mensonge que P1-H1 (manuel) et D6 (message log-only), appliquée à l'assertion **la plus visible** (montrée à chaque clôture).
- **And** la doc module `fiscal_years.rs:3-16` (ajouter `reopen` à la liste des fns mutatrices, nit P3-N1) et le doc `errors.rs:37` n'affirment plus que la réouverture est impossible.
- **Test (a) — backend `close()` reformulé (doc/log-only)** : le test existant (`fiscal_years_repository.rs:288`) n'assert que le **variant** (pas la chaîne) → le changement de message ne casse rien ; ajouter une assertion `grep`/string-check légère confirmant que le message/la doc ne contient plus « réouverture interdite ». Le `Display` n'étant jamais exposé au client (`kesh-db/errors.rs:8-9`), **aucun test comportemental client requis pour (a)**.
- **Test (b) — dialogue de clôture user-facing P3-F1 (finding P4-F2)** : ce patch **EST** rendu à l'utilisateur → il vient **avec son test** (`feedback_review_patch_needs_test`). **Vitest** : assertion sur le texte rendu du dialogue de clôture — il ne contient plus « définitivement » ni « ne pourra plus être enregistré » (1 locale suffit ; couvre clé + fallback). Cette assertion est distincte du disclaimer « log-only » de (a), qui ne vaut **que** pour le message backend.

## Tasks / Subtasks

- [x] **T1 — Repo `fiscal_years::reopen`** (`fiscal_years.rs`, nouvelle fn calquée sur `update_name:290-371`) : constantes clés `FY_REOPEN_ALREADY_OPEN_KEY` / `FY_REOPEN_LIFO_BLOCKED_KEY` / `FY_REOPEN_UNEXPECTED_KEY` (à côté de `FY_NAME_DUPLICATE_KEY`, D7) ; tx + `FOR UPDATE` before ; **désambiguïsation « déjà ouvert » via `before.status == Open` AVANT la garde LIFO** (P1-M4) → `Invariant(FY_REOPEN_ALREADY_OPEN_KEY)` ; garde LIFO (T2) ; `UPDATE ... status='Open' WHERE status='Closed'` ; `rows_affected==0` → re-SELECT (`Open`→`Invariant(ALREADY_OPEN)` ; `None`→`NotFound` ; sinon `Invariant(UNEXPECTED)`) ; audit `fiscal_year.reopened` `json!({before, after, motif})` via `build_audit_entry`/`insert_in_tx` ; commit. — AC-A.
- [x] **T2 — Query LIFO** `find_later_closed_in_tx(&mut tx, company_id, start_date)` (`WHERE start_date > ? AND status='Closed' ... LIMIT 1 FOR UPDATE`, miroir `find_overlapping`) + branchement dans `reopen` → `Invariant(FY_REOPEN_LIFO_BLOCKED_KEY)` si présent. Documenter l'hypothèse next-key locking dans le doc-comment (comme `find_overlapping`). — AC-B.
- [x] **T3 — Reformuler `close`/docs** : message `fiscal_years.rs:599-601` (« déjà clos », sans « réouverture interdite ») ; doc module `:3-16` (+ **ajouter `reopen`** à la liste des fns, P3-N1) ; doc `errors.rs:37`. **`close` non modifié fonctionnellement.** — AC-J/D6.
- [x] **T4 — API handler `reopen_fiscal_year` + mapping D7** (`routes/fiscal_years.rs`, miroir `close_fiscal_year:242-259`) : `ensure_not_pat` (D5) ; pré-check `find_by_id_in_company` (404) ; validation motif **non vide ET ≤ `REOPEN_MOTIF_MAX=500`** → `AppError::Validation` (400, miroir `:186-193`, P1-M5) ; appel `reopen` **via `.map_err(map_reopen_error)`** ; `Json<FiscalYearResponse>`. **Nouveau `map_reopen_error`** (miroir `map_create_error:87`) + **nouveau variant `AppError::IllegalState(String)`** (`kesh-api/errors.rs` → 409 `ILLEGAL_STATE_TRANSITION` + message passé, D7). Nouveau DTO `ReopenFiscalYearRequest { motif }`. — AC-D/D7.
- [x] **T5 — Route Admin-only** : enregistrer `POST /api/v1/fiscal-years/{id}/reopen` dans le bloc **`admin_routes` (`lib.rs:182-268`)**, scellé par `require_admin_role` (`:269-271`). **PAS** dans `comptable_routes`. — AC-D.
- [x] **T6 — Frontend** : `ReopenFiscalYearRequest` (types) + `reopenFiscalYear` (api) ; page `settings/fiscal-years/+page.svelte` — `isAdmin` derived, bouton Réouvrir (`{#if fy.status==='Closed'}` + `{#if isAdmin}`, désactivé + tooltip **nommant le plus proche postérieur clos** si LIFO client, P3-F4), état + `submitReopen()` (clone `submitClose:166-201`), modal motif (mirror `DunningPauseDialog` + champ requis `ManualReminderDialog`) avec `maxlength=REOPEN_MOTIF_MAX` + `clientError` sur vide/trop long (P1-M5). **+ Corriger les DEUX fallbacks svelte hardcodés du dialogue de clôture : `:405` (body) ET `:428` (action « Clôturer définitivement » → « Clôturer »)** (irréversibilité absolue → « tant qu'il reste clôturé », P3-F1 + P4-F1). — AC-E/F/J.
- [x] **T7 — i18n** : **11 nouvelles clés** `fiscal-year-reopen-*` / `error-fiscal-year-*` (dont `-motif-too-long`) × **4 locales** (AC-G) **+ ÉDITER 2 clés existantes** `fiscal-year-close-confirmation-body` et `fiscal-year-close-confirmation-action` × 4 locales (retirer l'irréversibilité absolue / « définitivement », P3-F1/AC-J).
- [x] **T8 — Tests** : repo (`fiscal_years_repository.rs`, dont **course concurrente** P1-M6) + e2e (`fiscal_years_e2e.rs`, dont **assertion du contenu de message** already-open/LIFO P1-C1 + **motif > 500** P1-M5) + Vitest frontend (cf. AC-H). Chaque patch avec son test (`feedback_review_patch_needs_test`).
- [x] **T9 — Doc-sync** : CHANGELOG + admin-manual.tex + **`user-manual.tex:464-466`+`:445-457`** (P1-H1, corriger la fausse « procédure CLI ») + **`README.md:208`** (P1-M1, retirer « à suivre ») + docs modules (AC-I). Créer une **issue GitHub snapshot-invalidation** (ou compléter l'issue snapshot existante de 14-1) — labellée **`v0.2-milestone`** (P1-M2, formalise L3 catégorie B) — notant que `fiscal_year.reopened` devra invalider un futur snapshot ; rien à coder en v1 virtuel. **Pas de migration.**

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
2. **Verrou `FOR UPDATE`** dans `reopen` **et** la query LIFO : sinon course avec une clôture/édition concurrente (l'équation/immutabilité repose sur la sérialisation). `reopen` ne touche **que** `fiscal_years` (table isolée, doc module `:9-16`) — **pas** de chaîne cross-table `companies → projects → fiscal_years` (mention retirée, P1-LOW). La query LIFO s'appuie sur le next-key locking InnoDB (filtre `status` non indexé) — hypothèse **testée** (course concurrente, AC-H / P1-M6).
3. **Motif dans l'audit uniquement** (D1) : `json!({"before":.., "after":.., "motif":..})`. Ne PAS ajouter de colonne. Le test lit `audit_log.details_json`.
4. **Admin-only au router-layer** (D4) : route dans `admin_routes`, **jamais** de check de rôle dans le handler. Ne pas la mettre par erreur dans `comptable_routes` (où vit `close`).
5. **`ensure_not_pat`** (D5) avant tout : la réouverture n'est pas une opération d'automatisation.
6. **Garde LIFO = `start_date >` strict** (D3) : borne stricte, statut `Closed`. Frontend désactive proactivement ; serveur = filet 409.
7. **`close` idempotence** : la branche `:599-601` reste (re-clore un exercice clos = erreur), seul son **message** change (D6). Ne pas la supprimer.
8. **`already-open` vs `LIFO`** (D7, P1-C1) : NE PAS émettre `DbError::IllegalStateTransition` (son `Display` est **log-only**, `kesh-api/errors.rs:2124` → message générique unique → clés i18n mortes). Émettre `Invariant(KEY)` namespacés + `map_reopen_error` → `AppError::IllegalState(t(...))` (409, code partagé, **messages distincts**). Le frontend affiche `err.message` sur échec du modal (les deux messages sont localisés **au mapper** via `t()`). Les tests e2e assertent le **contenu** du message.

### Limitations documentées (triage catégories, finding P1-M2)

- **L1 — motif non requêtable en 1er plan (catégorie C — décision design intentionnelle)** : le motif vit dans `audit_log.details_json` (D1, décision Gui explicite ; l'audit rétention 10 ans EST la source de vérité). Ce n'est **pas** une dette (pas un raccourci sous contrainte) — pas de suivi requis. Une future colonne dédiée (migration) ne serait justifiée que si un besoin de reporting 1er-plan émerge.
- **L2 — code d'erreur 409 partagé entre `already-open` / `LIFO` (catégorie C — décision design intentionnelle)** : depuis D7, les deux cas ont des **messages distincts** localisés ; seul le **code machine** `ILLEGAL_STATE_TRANSITION` est partagé. Un code d'erreur *distinct* (pour qu'un client API branche dessus programmatiquement) est un **choix de simplicité assumé** (le frontend désactive proactivement le cas LIFO ; le message distinct suffit à l'UX). Amélioration future si un consommateur API le requiert — pas une dette.
- **L3 — invalidation snapshot différée (catégorie B — dette tracée)** : aucun snapshot n'existe (14-1 virtuel), donc rien à invalider. Quand la couture snapshot de clôture sera livrée, elle DEVRA écouter `fiscal_year.reopened`. **Suivi** : issue GitHub snapshot labellée `v0.2-milestone` (T9) — planification implicite conforme politique zero-carry-forward.
- **L4 — `close` n'impose pas de clôturer les exercices antérieurs d'abord (catégorie C — comportement pré-existant / décision Gui)** : finding P1-H2. La garde LIFO (D3) discipline la **réouverture** mais pas la **clôture** — un `close` peut clore un exercice avec un antérieur `Open` (vrai **avant** 14-2, `close` inchangé D6). En modèle virtuel 14-1 (report à-nouveau recalculé en direct, aucun snapshot), cette asymétrie **ne corrompt aucun solde**. Une garde FIFO symétrique sur `close` deviendra pertinente **si/quand** la couture snapshot de clôture est livrée (à évaluer avec L3).

### References

- Conception : note léguée **14-1:43/52/156** (interaction réouverture ↔ snapshot, ancre `fiscal_years.rs:600`) ; sprint-status (scope Admin+motif+audit+garde d'ordre) ; décisions Guy 2026-07-24 (D1 audit-only, D3 LIFO strict).
- Norme : CO art. 957-964 (immutabilité + rétention 10 ans ; la réouverture tracée reste conforme — la piste d'audit prime).
- Conventions : CLAUDE.md § Test Locally First, § Review Iteration Rule, § Règle de commit, § Issue Tracking Rule.

## Change Log — create-story

Spec créée 2026-07-24 par cartographie ground-truth parallèle (3 agents Explore : backend clôture/audit/immutabilité, API+RBAC, frontend/modal). L'Epic 14 de `epics.md` (« Justificatifs, Lettrage ») est **périmé** — le scope 14-2 vient de la note sprint-status + 14-1 + décisions Guy. 2 forks tranchés par Guy avant rédaction : **D1** motif → audit `details_json` seul (pas de migration) ; **D3** garde d'ordre **LIFO strict** (tout exercice postérieur clos bloque). Décisions dérivées : D2 (fn `reopen` calquée `update_name`, flip guardé + audit before/after), D4 (Admin-only router-layer + motif obligatoire), D5 (`ensure_not_pat`), D6 (`close` inchangé, message reformulé). **Pas de migration** (P3/P5 sans objet). Prochaine étape : `bmad-create-story validate`.

## Change Log — validate

**Passe 1 (Sonnet ×3 — BlindHunter / EdgeCaseHunter / AcceptanceAuditor, contexte frais, ground-truth grep obligatoire)** : **1 CRITICAL, 2 HIGH, 5 MEDIUM, ~2 LOW** (findings dédupliqués). BlindHunter : 0 > LOW (toutes les ancres `fichier:ligne` vérifiées exactes). Le CRITICAL a **convergé indépendamment** sur AcceptanceAuditor + EdgeCaseHunter → signal fort.

Findings & remédiation :
- **P1-C1 (CRITICAL)** — `DbError::IllegalStateTransition` est **log-only** (`kesh-api/errors.rs:2124`) → les 2 messages 409 distincts promis (already-open / LIFO) seraient identiques et les clés i18n mortes. **Décision Gui = « messages distincts, code partagé »** → nouvelle **D7** : `Invariant(KEY)` namespacés + `map_reopen_error` + variant `AppError::IllegalState(String)` (409 code partagé, message distinct). Tests e2e assertent le **contenu** du message. Patché D2/D3/D7/AC-A/B/D/G/H, T1/T2/T4/T8, Piège 8, L2.
- **P1-H1 (HIGH)** — `user-manual.tex:464-466` annonce une fausse « procédure CLI » de réouverture. Ajouté à AC-I/T9.
- **P1-H2 (HIGH)** — `close` ne garde pas l'ordre côté clôture (asymétrie vs D3). **Décision Gui = « documenter la limitation »** → D3 assoupli + **L4** (catégorie C, pré-existant, sans corruption en modèle virtuel).
- **P1-M1** — `README.md:208` (« à suivre ») ajouté au doc-sync T9.
- **P1-M2** — L1/L2 reclassées **catégorie C** (décisions design Gui), L3 formalisée **catégorie B** (issue `v0.2-milestone`, T9).
- **P1-M3** — D6/T3 sans AC/test → nouvelle **AC-J** (hygiène message/doc `close`, doc/log-only).
- **P1-M4** — ordre `reopen` : désambiguïsation « déjà ouvert » **avant** garde LIFO (D2/T1).
- **P1-M5** — borne `REOPEN_MOTIF_MAX=500` client+serveur (modèle `PAUSE_NOTE_MAX`) + clé i18n `-motif-too-long` (D4/AC-D/G/H, T4/T6/T7/T8).
- **P1-M6** — course concurrente `reopen`/`close` : test d'intégration + hypothèse next-key locking documentée (D3/AC-H/T2/T8).
- **P1-LOW** — ordre de verrou `companies→projects→fiscal_years` retiré de D2 (reopen isole `fiscal_years`) ; span `lib.rs` harmonisé D4.

**Passe 2 (Haiku ×3 — ancres+régressions / edge+logique / complétude+conventions, contexte frais, spec patchée, garde-fou grep ground-truth actif)** : **0 CRITICAL, 0 HIGH net-nouveau, 0 MEDIUM net-nouveau** après triage. Aucune hallucination Haiku détectée (les CRITICAL/HIGH ont été grep-vérifiés).

Triage des findings Haiku :
- **H1 (HIGH déclaré)** — message `close` « réouverture interdite » (`fiscal_years.rs:600`, grep-confirmé) : **déjà couvert** par D6/AC-J/T3 (la remédiation est planifiée, pas un trou de spec). Non net-nouveau.
- **P2-M2 (MED)** — `AppError::IllegalState(String)` porte un message pré-localisé : **non-issue**, cohérent avec le précédent `AppError::Validation` (`map_create_error` fait déjà `Validation(t(...))`). → note ajoutée D7.
- **P2-M3 (MED)** — hypothèse next-key locking : **déjà couvert** (note D3 + test course AC-H/P1-M6).
- **P2-M4 (MED, auto-reclassé LOW)** — édition post-reopen recalcule le report des exercices postérieurs ouverts : conforme modèle virtuel → **note explicite ajoutée à AC-C**.
- **P2-M1 (MED→LOW)** — asymétrie `chars().count()` / `maxlength` UTF-16 : note défensive ajoutée D4 (idem `PAUSE_NOTE_MAX`).
- **LOW** — typo « réouverction » ×2 corrigée ; L1/L2 (doc module, routing) déjà couverts T3/T5.

Trend : **Passe 1 (Sonnet ×3) : 1 CRIT / 2 HIGH / 5 MED > LOW → Passe 2 (Haiku ×3) : 0 > LOW net-nouveau.** Critère d'arrêt atteint au sens strict. Passe 3 (Opus ×3, modèle orthogonal) lancée en **confirmation** (le pattern projet 3-passes + Haiku = modèle le plus faible justifient une passe de contrôle orthogonale avant de sceller).

**Passe 3 (Opus ×3 — deep correctness / edge+invariants / complétude+final, contexte frais, confirmation)** : **0 CRITICAL, 0 HIGH, 1 MEDIUM (P3-F1), ~5 LOW.** Deux des trois Opus votent « prête pour dev » ; le troisième trouve **1 MEDIUM net-nouveau** manqué par les 4 reviewers précédents.

- **P3-F1 (MEDIUM)** — le **dialogue de confirmation de clôture** (`fiscal-year-close-confirmation-body` × 4 locales + fallback svelte `:405` + label `-action` « Clôturer définitivement ») affirme encore l'**irréversibilité absolue** (« ne pourra plus être enregistré ») — user-facing, sur la même page que le futur bouton Réouvrir, et **faux** dès 14-2 livrée. Même classe de mensonge que P1-H1 (manuel) / D6 (message log-only), mais l'assertion la **plus visible**. Grep-vérifié (4 locales + `:405`). **Patché** → AC-J + T6/T7 (édition des 2 clés existantes × 4 locales + fallback, formulation préservant le garde-fou).
- **P3-Opus2 (LOW)** — validation motif au handler seulement (≠ miroir `update_name` repo-side) : **choix assumé documenté** dans D4 (caller unique = route Admin ; forward-note si caller non-route futur).
- **P3-F1-Opus3 (LOW)** — contradiction résiduelle : AC-A conservait encore la mention d'ordre de verrou cross-table retirée ailleurs → **retirée** de AC-A.
- **P3-F4 (LOW)** — tooltip client LIFO : nommer **le plus proche postérieur clos** (aligné serveur `ORDER BY start_date ASC LIMIT 1`) → AC-E/T6.
- **P3-N1/N2 (nits)** — ajouter `reopen` à la liste des fns du module doc (T3) ; chemin crate `dunning_reminders.rs` (D4). Appliqués.
- **P3-F2/F3 (LOW, sans action)** — ordre 404-avant-400 (acceptable, route Admin) ; branche `rows_affected==0` défensive/inatteignable (belt-and-suspenders assumé). Confirmés sains, aucune modif.

Confirmations Opus (creusées en profondeur, toutes vertes) : D7 bout-en-bout implémentable + non sur-ingénieré ; transaction/audit atomiques ; garde LIFO correcte sur 3+ exercices ; **multi-tenant sans IDOR** ; courses reopen×reopen / reopen×close sérialisées par `FOR UPDATE` ; invariant comptable (report recalculé pour TOUS les postérieurs, balance équilibrée) ; piste d'audit CO 957-964 complète (acteur+horodatage+motif) ; frontend ancré patterns réels.

Trend : **P1 (Sonnet×3) 1 CRIT/2 HIGH/5 MED → P2 (Haiku×3) 0>LOW net → P3 (Opus×3) 1 MED (P3-F1) + LOW.** P3-F1 (MEDIUM) patché → une **passe 4 de confirmation (Sonnet, modèle orthogonal à Opus)** requise par la Review Iteration Rule.

**Passe 4 (Sonnet ×3 — confirmation P3-F1 / edge résiduel / complétude, contexte frais)** : **0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW.** Les **2 MEDIUM portent tous deux sur le patch P3-F1 lui-même** (motif `feedback_review_patch_needs_test` : la remédiation devient la source du finding suivant) — ni l'un ni l'autre n'est une nouvelle classe de défaut, ce sont des **complétions** du même fix :
- **P4-F1 (MED)** — P3-F1 a corrigé le fallback svelte du **corps** (`+page.svelte:405`) mais oublié celui du **label** `-action` (`:428`, grep-vérifié « Clôturer définitivement »). Fallback **atteignable** (i18n fire-and-forget non bloquant). **Patché** → AC-J + T6 couvrent désormais les **2** fallbacks.
- **P4-F2 (MED)** — le bullet « Test » d'AC-J appliquait le disclaimer « doc/log-only, pas de test client » au **mauvais** patch : il ne vaut que pour le message backend `close()`, pas pour le changement de **copie user-facing** P3-F1. **Patché** → AC-J scindé en Test (a) backend log-only / Test (b) frontend Vitest ; assertion copie ajoutée à AC-H.
- **P4-LOW** — test LIFO 3-exercices intercalés (`Closed-Open-Closed`) : ajouté à AC-H (renforce, non requis par la logique SQL).

Confirmations Sonnet (toutes vertes) : interactions cross-modules (factures/fournisseurs/avoirs/réconciliation via statut vivant ; `balance_sheet.rs` ignore le statut) ; états limites `fiscal_years` ; audit before/after cohérent même après `update_name` sur Closed ; reload frontend sûr (`loadFiscalYears` a son propre try/catch) ; 100% des ancres re-grep-vérifiées exactes ; sprint-status déjà `ready-for-dev`.

Trend : **P1 (Sonnet×3) 1 CRIT/2 HIGH/5 MED → P2 (Haiku×3) 0>LOW → P3 (Opus×3) 1 MED (P3-F1) → P4 (Sonnet×3) 2 MED (complétions de P3-F1).** Tendance strictement décroissante en **gravité et périmètre** (architecture → une seule copie de dialogue + son test). Ce n'est PAS le cas « sur-étalement » de la règle de splitting (le design a convergé dès P3, tous les Opus jugeant le cœur sain) — c'est la **queue de convergence** d'un unique fix P3-F1. Les 2 MED patchés (fix structurel : les 2 fallbacks + le test qui garde-fou la copie) → **Passe 5 de confirmation (Haiku, prochain du cycle)** requise.

**Passe 5 (Haiku ×3 — confirmation P4-F1/F2 / balayage global ancres+logique / conventions+doc-sync, contexte frais, garde-fou grep actif)** : **0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW** sur les 3 lentilles. Les 2 patches P4 grep-vérifiés **complètement appliqués** (les 2 fallbacks `:405`+`:428` ; Test scindé a/b + assertion Vitest). Aucune régression, aucune contradiction résiduelle, toutes les ancres re-vérifiées exactes, sprint-status déjà `ready-for-dev` (ligne confirmée). Aucune hallucination Haiku.

---

### ✅ CONVERGENCE — spec scellée `ready-for-dev` (2026-07-24)

**Trend numérique (Review Iteration Rule)** :

| Passe | Modèles (×3, contexte frais) | Findings > LOW | Détail |
|-------|------------------------------|----------------|--------|
| 1 | **Sonnet** ×3 | **8** (1 CRIT / 2 HIGH / 5 MED) | C1 mapping erreur, H1 user-manual CLI, H2 asymétrie close, M1 README, M2 triage L, M3 AC-J, M4 ordre, M5 motif borné, M6 course |
| 2 | **Haiku** ×3 | **0** net-nouveau | Tous les MED = doc-tasks déjà planifiées (T3/T9) ; 0 hallucination |
| 3 | **Opus** ×3 | **1** (1 MED) | P3-F1 : dialogue de clôture ment sur l'irréversibilité (manqué par P1-2) |
| 4 | **Sonnet** ×3 | **2** (2 MED) | P4-F1/F2 : complétions de P3-F1 (fallback `:428` oublié + test copie) |
| 5 | **Haiku** ×3 | **0** | Convergé sur les 3 lentilles ✓ |

**Cycle LLM** : Sonnet → Haiku → Opus → Sonnet → Haiku (5 passes, ×3 agents = 15 revues adversariales, chacune avec grep ground-truth). Critère d'arrêt atteint : 0 finding > LOW en passe 5 (< 8 passes max).

**Décisions Gui tranchées en cours de boucle** : (C1) « messages 409 distincts, code partagé » → D7 ; (H2) « documenter l'asymétrie close » → L4.

**Reclassements** : L1 (motif non-requêtable) et L2 (code 409 partagé) → **catégorie C** (décisions design intentionnelles, pas dette) ; L3 (snapshot) → **catégorie B** tracée (issue `v0.2-milestone`, T9) ; L4 (asymétrie close) → **catégorie C** (pré-existant, sans corruption en modèle virtuel).

**Note sur la longueur de boucle (règle de splitting préventif)** : 5 passes > le seuil indicatif de 4, MAIS ce n'est **pas** le cas « sur-étalement / profondeur d'incertitude » que la règle cible — le **design a convergé dès P3** (tous les Opus jugeant le cœur backend sain et implémentable). Les passes 4-5 n'ont traité que la **queue de convergence d'un unique fix** (P3-F1, dialogue de clôture) et sa complétion (2 fallbacks + son test). La gravité et le périmètre ont strictement décru (architecture → une copie de dialogue). Pas de split requis.

**Cœur confirmé sain (creusé par les 15 revues)** : D7 mapping erreur bout-en-bout ; transaction/audit atomiques ; garde LIFO correcte 2 & 3+ exercices ; **multi-tenant sans IDOR** ; courses (reopen×reopen, reopen×close, LIFO concurrent) sérialisées par `FOR UPDATE` ; invariant comptable (report recalculé pour TOUS les postérieurs, balance équilibrée) ; piste d'audit CO 957-964 complète ; interactions cross-modules débloquées par le statut vivant ; frontend ancré patterns réels ; **pas de migration**.

**Prochaine étape** : `bmad-dev-story` sur la branche `story/14-2-reouverture-exercice`.

## Dev Agent Record

### Change Log — dev-story (2026-07-24)

Implémentation complète T1→T9 en un seul passage (spec scellée 5 passes, aucune ambiguïté). **Pas de migration** (D1 : motif dans `audit_log.details_json`). Aucune déviation du design validé.

- **T1/T2 (repo)** : `fiscal_years::reopen` calquée sur `update_name` (flip guardé `Closed→Open` + audit `fiscal_year.reopened` `{before, after, motif}`, `FOR UPDATE`, désambiguïsation « déjà ouvert » avant LIFO, re-SELECT défensif) + `find_later_closed_in_tx` (garde LIFO, `start_date > ?` strict, `ORDER BY start_date ASC LIMIT 1 FOR UPDATE`, hypothèse next-key locking documentée). Constantes `FY_REOPEN_ALREADY_OPEN_KEY` / `FY_REOPEN_LIFO_BLOCKED_KEY` / `FY_REOPEN_UNEXPECTED_KEY` + `REOPEN_MOTIF_MAX=500`.
- **T3 (D6)** : message de re-clôture reformulé (« déjà clos (re-clôture no-op) », ne prétend plus « réouverture interdite ») ; doc module + `close` doc-comment + doc `kesh-db/errors.rs` `IllegalStateTransition` mis à jour. `close` non modifié fonctionnellement.
- **T4/T5 (API, D7)** : handler `reopen_fiscal_year` (`ensure_not_pat` → pré-check 404 → validation motif vide/≤500 → `reopen` via `map_reopen_error`) ; nouveau variant `AppError::IllegalState(String)` → 409 `ILLEGAL_STATE_TRANSITION` (message déjà localisé) ; `map_reopen_error` (2 clés → 2 messages distincts) ; DTO `ReopenFiscalYearRequest`. Route montée dans `admin_routes` (scellé `require_admin_role`).
- **T6/T7 (frontend + i18n)** : `reopenFiscalYear` (api) + `ReopenFiscalYearRequest` (types) ; page `+page.svelte` — `isAdmin` derived, bouton Réouvrir (`Closed && isAdmin`, désactivé + tooltip `nearestLaterClosed` pour la garde LIFO client), `submitReopen` (clone `submitClose`, reload hors try/catch), dialogue motif (textarea `maxlength=500` + `reopenClientError` vide/trop-long → `disabled`). **Les 2 fallbacks svelte du dialogue de clôture corrigés** (`:405` body + action « Clôturer »). 11 nouvelles clés i18n × 4 locales + **2 clés close éditées** (`-body` / `-action`, retrait de l'irréversibilité absolue / « définitivement ») × 4 locales.
- **T8 (tests)** : 9 tests repo (`fiscal_years_repository.rs` : audit+motif, déjà-ouvert sans audit, NotFound cross-tenant, LIFO bloqué+ordonné, LIFO permissif, LIFO 3-exercices intercalés, immutabilité ré-activée par le flip, message close reformulé, **course concurrente `reopen`/`close` sérialisée**) + 10 tests e2e (`fiscal_years_e2e.rs` : Admin 200+audit, Comptable 403, Consultation 403, sans-auth 401, PAT 403, motif vide 400, motif >500 400, cross-tenant 404, **déjà-ouvert 409 message distinct**, **LIFO 409 message distinct**) + 5 tests Vitest (`fiscal-years-page.test.ts` : visibilité Admin/Comptable, désactivation+tooltip LIFO, modal désactivé/activé + submit appelle `reopenFiscalYear`, **dialogue clôture sans « définitivement »/« ne pourra plus être enregistré »**).
- **T9 (doc-sync)** : CHANGELOG `[Non publié]` (réouverture + reformulation clôture) ; `README.md` v0.8.0 (« à suivre » → livrée) ; `admin-manual.tex` §exercices (paragraphe réouverture Admin-only + motif + LIFO + audit) ; `user-manual.tex` (clôture = verrou virtuel, plus d'« écritures de clôture générées » ; réouverture réécrite : UI Admin + motif, plus de fausse « procédure CLI ») ; docs modules. Issue GitHub **#270** (snapshot) complétée : label `v0.2-milestone` + commentaire L3 (`fiscal_year.reopened` devra invalider le futur snapshot).

### Completion Notes

- **Gate** : `cargo fmt --all --check` ✅, `cargo clippy --workspace --all-targets -D warnings` ✅, `cargo build` ✅ ; tests reopen ciblés verts (repo 9/9, e2e 10/10) ; frontend `npm run check` (0 err), `lint-i18n-ownership` PASS, `test:unit` 436/436, `build` ✅. Gate complet serial `cargo test --workspace` lancé sur une DB de gate fraîchement seedée `kesh_gate` (la DB dev `kesh` de dogfooding n'est pas dans l'état seed_demo attendu par les tests `test_pool()`).
- **Pas de migration** (P3/P5 sans objet).
- **Aucune modification de `journal_entries.rs`** : l'immutabilité ré-activée par le flip est prouvée par test (AC-C), pas par modification — le statut vivant pilote la garde.

### File List

**Backend (kesh-db)**
- `crates/kesh-db/src/repositories/fiscal_years.rs` (modifié — `reopen`, `find_later_closed_in_tx`, constantes, message close reformulé, doc module)
- `crates/kesh-db/src/errors.rs` (modifié — doc `IllegalStateTransition`)
- `crates/kesh-db/tests/fiscal_years_repository.rs` (modifié — 9 tests reopen)

**Backend (kesh-api)**
- `crates/kesh-api/src/errors.rs` (modifié — variant `AppError::IllegalState` + mapping 409)
- `crates/kesh-api/src/routes/fiscal_years.rs` (modifié — DTO, `map_reopen_error`, handler `reopen_fiscal_year`, doc)
- `crates/kesh-api/src/lib.rs` (modifié — route `/reopen` dans `admin_routes`)
- `crates/kesh-api/tests/fiscal_years_e2e.rs` (modifié — 10 tests reopen + helpers Comptable/create-close)

**i18n**
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (modifiés — 11 clés reopen + 2 clés close éditées)

**Frontend**
- `frontend/src/lib/features/fiscal-years/fiscal-years.types.ts` (modifié — `ReopenFiscalYearRequest`)
- `frontend/src/lib/features/fiscal-years/fiscal-years.api.ts` (modifié — `reopenFiscalYear`)
- `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` (modifié — bouton + modal + fallbacks close)
- `frontend/src/routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts` (créé — 5 tests Vitest)

**Docs**
- `CHANGELOG.md`, `README.md`, `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/user-manual.tex` (modifiés)
- GitHub issue #270 (label `v0.2-milestone` + commentaire L3)
