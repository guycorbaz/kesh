# Story 17.2b: API externe à clé PAT — Frontend (page de gestion `/settings/api-keys`)

Status: review

<!-- Issue de la scission de la story 17-2 (spec convergée 5 passes validate, 2026-06-04). 17-2b = Partie B frontend (rollout après la story-foundation 17-2a backend, mergée PR #168). Voir 17-2-api-pat-integrations.md pour le contexte parent complet et 17-2a-api-pat-backend.md pour le contrat backend réel consommé ici. -->

## Story

As a **PME utilisant Kesh (rôle Comptable ou Admin)**,
I want **une page `/settings/api-keys` pour créer, lister et révoquer mes clés d'accès API, avec affichage one-time du secret et bouton copier**,
so that **je puisse gérer mes intégrations IA / logiciels tiers depuis l'UI web sans toucher à la base de données ni à des appels API manuels**.

## Contexte & provenance

- **Issue GitHub** : [#100](https://github.com/guycorbaz/kesh/issues/100) `[CR] API externes avec clé PAT (lecture / lecture-écriture)` — `enhancement` + `v0.2-milestone`.
- **Epic** : 17 « Infra & Souveraineté » — Story 17-2 (split 17-2a/b/c).
- **Scission** : cette story **17-2b** est la **Partie B frontend**, en **rollout** après la story-foundation **17-2a** (backend, mergée **PR #168**, commits `b9e9c8c`/`ad33430`/`d7ac81e`). 17-2b consomme les routes CRUD `/api/v1/settings/api-keys` livrées par 17-2a. La doc externe (OpenAPI, `docs/api-external.md`) reste pour **17-2c**.
- **Dépendance dure** : 17-2b ne peut être livrée qu'**après** 17-2a (routes + codes d'erreur). 17-2a est mergée → 17-2b débloquée. La branche `story/17-2b-api-pat-frontend` est **stackée** sur `story/17-2-api-pat-integrations` (HEAD `d7ac81e`) pour disposer du code backend.

> ⚠️ Cette story est **frontend uniquement** (page Svelte + i18n `api-keys-*` + E2E Playwright). Aucun changement backend Rust attendu (le contrat est figé par 17-2a). **Exception** : les clés i18n `api-keys-*` de la page se déclarent dans les 4 fichiers `crates/kesh-i18n/locales/*/messages.ftl` (le frontend ne possède pas de fichiers de locales séparés — il hydrate via `GET /api/v1/i18n/messages`).

## Contrat backend consommé (figé par 17-2a — ground-truth `routes/api_keys.rs`)

⚠️ **Route réelle : `/api/v1/settings/api-keys`** (préfixe `settings/`, vérifié `lib.rs:278-283`), **PAS** `/api/v1/api-keys`. Guard `require_comptable_role` (DC4 : Comptable + Admin).

| Méthode | Route | Body | Réponse (serde camelCase) |
|---------|-------|------|----------------------------|
| `GET` | `/api/v1/settings/api-keys` | — | `ApiKeyResponse[]` = `{ id, name, scope, createdAt, lastUsedAt, revokedAt, expiresAt, version }` (toutes les clés, actives + révoquées, triées `createdAt DESC` ; **jamais** le hash ni le secret) |
| `POST` | `/api/v1/settings/api-keys` | `{ name, scope, expiresAt? }` | `CreateApiKeyResponse` = `{ id, name, scope, createdAt, key }` — **`key` = `kesh_pat_…` en clair, retourné UNE SEULE FOIS** |
| `DELETE` | `/api/v1/settings/api-keys/{id}` | `{ version }` (optimistic lock) | `204 No Content` |

- `scope` ∈ `{ "read", "read-write" }` (chaînes exactes).
- `expiresAt` : RFC 3339 (`DateTime<Utc>`, ex. `2027-01-01T00:00:00Z`), optionnel. **Validé futur-only côté backend** (Pass 1 17-2a) → `400 Validation` si passé.
- `name` : non-vide après trim, **≤ 255 caractères** (Pass 1 17-2a) → `400 Validation` sinon.
- Dates (`createdAt`/`lastUsedAt`/`revokedAt`/`expiresAt`) : sérialisées `NaiveDateTime` (sans timezone) — à parser/formatter comme les dates `bank-accounts`.
- Codes d'erreur backend déjà i18n'd (17-2a) : `API_KEY_READ_ONLY` (403), `API_KEY_MANAGEMENT_FORBIDDEN` (403). Le second protège la route si un PAT tentait de gérer des clés — **non atteignable depuis cette page** (UI = session JWT cookie), mais reste un filet de sécurité backend.

## Décisions de conception (héritées du validate parent + ground-truth frontend)

- **DC-B1 — Page sous `/settings/api-keys`, lien depuis `/settings`.** Route SvelteKit `frontend/src/routes/(app)/settings/api-keys/+page.svelte`. Lien « Clés API » ajouté à `routes/(app)/settings/+page.svelte` (calque section `Comptes bancaires`).
- **DC-B2 — Feature module calqué sur `bank-accounts`.** `frontend/src/lib/features/api-keys/` : `api-keys.api.ts` (`listApiKeys`/`createApiKey`/`revokeApiKey` via `apiClient`), `api-keys.types.ts` (interfaces camelCase). Le `revoke` envoie `version` **dans le body** du DELETE (convention projet `bank-accounts::archive`, confirmée au code-review 17-2a).
- **DC-B3 — Secret one-time + copie HTTP-LAN-safe (AC10, MÉMOIRE CRITIQUE).** Le secret `kesh_pat_…` n'est retourné qu'à la création → l'afficher dans un encart persistant jusqu'à fermeture explicite, avec bouton « Copier ». **`navigator.clipboard` est `undefined` hors secure-context (HTTP LAN NAS)** → fallback `document.execCommand('copy')` obligatoire (cf. [[feedback_no_secure_context_apis_http_lan]], bug #145). Idem : **ne JAMAIS utiliser `crypto.randomUUID`** pour des IDs DOM → `$props.id()` (Svelte 5).
- **DC-B4 — i18n préfixe exclusif `api-keys-`.** Toutes les chaînes de la page utilisent `i18nMsg('api-keys-…', 'fallback FR')`. Le préfixe `api-keys-` (avec le « s », dérivé du dossier) est **exclusif** au sens de `lint-i18n-ownership.js`. Les clés se déclarent dans les **4** `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`.
- **DC-B5 — Svelte 5 runes strict.** `$state`/`$derived`/`$props` partout (pas de `let` réactif legacy, pas de `on:`). Chargement client via `onMount` (pas de `+page.ts` SSR), calque `bank-accounts/+page.svelte`.
- **DC-B6 — Confirmation de révocation forte.** Encart de confirmation inline (calque `bank-accounts` archive-confirm), pas de simple `confirm()`. Sur `OPTIMISTIC_LOCK_CONFLICT` (409) → recharger la liste + message.

## Acceptance Criteria

### Frontend — feature module + page

1. **AC1 (parent AC9) — Feature module `api-keys`** : `frontend/src/lib/features/api-keys/api-keys.api.ts` expose `listApiKeys()`, `createApiKey(payload)`, `revokeApiKey(id, version)` via `apiClient` (chemin `/api/v1/settings/api-keys`). `api-keys.types.ts` définit `ApiKey` (liste), `NewApiKey` (création), `CreatedApiKey` (réponse avec `key`) en camelCase. Erreurs surfacées via `isApiError` (code + message).

2. **AC2 (parent AC9) — Page `/settings/api-keys`** (`routes/(app)/settings/api-keys/+page.svelte`) : tableau listant **toutes** les clés (nom, scope, créée le, dernière utilisation, statut — `Active` / `Révoquée le …` / `Expire le …`), triées `createdAt DESC`. État vide explicite. Chargement via `onMount` + état `loading`/`loadError`. Garde de rôle : route sous `(app)` (authentifiée) ; le backend `require_comptable_role` rejette un Viewer (surfacer le 403).

3. **AC3 (parent AC9) — Création via modal/form** : bouton « Nouvelle clé » → formulaire (champ `nom` requis `maxlength=255`, `select` scope `read`/`read-write`, date d'expiration optionnelle — picker qui empêche une date passée). Validation client (nom non-vide ≤ 255, expiration future) **et** surface des `400` backend (`name` trop long, expiration passée).

4. **AC4 (parent AC10) — Affichage one-time du secret + copie** : après création, afficher l'encart secret `kesh_pat_…` (mono, persistant jusqu'à fermeture explicite avec avertissement « copiez-la maintenant, elle ne sera plus affichée »). Bouton « Copier » **HTTP-LAN-safe** (`navigator.clipboard` si dispo, sinon fallback `document.execCommand('copy')`) + toast succès/échec. Le secret n'est jamais re-affiché ni re-fetchable (le `GET` ne le renvoie pas).

5. **AC5 (parent AC9) — Révocation avec confirmation forte** : bouton « Révoquer » par ligne (clé active uniquement) → encart de confirmation inline → `revokeApiKey(id, version)` → recharger. Les clés révoquées restent affichées (historique, `revokedAt`). Conflit optimistic lock (409) → recharger + message.

6. **AC6 (parent AC9) — Lien depuis `/settings`** : section/lien « Clés API » ajouté à `routes/(app)/settings/+page.svelte` (calque section `Comptes bancaires`), `data-testid` stable.

### Frontend — i18n

7. **AC7 (parent AC11) — i18n 4 langues, préfixe `api-keys-`** : toutes les chaînes de la page + feature internationalisées FR/DE/IT/EN, préfixe `api-keys-` exclusif (catégories `api-keys-labels-*` / `api-keys-actions-*` / `api-keys-errors-*` / `api-keys-confirm-*` / `api-keys-toast-*`, calque `bank-accounts-*`). Clés déclarées dans les 4 `crates/kesh-i18n/locales/*/messages.ftl`. `npm run lint-i18n-ownership` vert. La section dans `/settings/+page.svelte` utilise des clés `settings-api-keys-*` (namespace `settings`, hors feature folder).

### Quality gate

8. **AC8 (parent AC13) — Quality gate frontend vert** : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` (depuis `frontend/`). Backend inchangé → checks Rust no-op (mais relancer `cargo build`/`test` si un `.ftl` modifié casse un test de complétude i18n).

9. **AC9 (parent AC13) — E2E Playwright** : `frontend/tests/e2e/api-keys.spec.ts`. Scénario : login (`admin`/`admin123`, `seedTestState('with-company')`) → `/settings/api-keys` → créer une clé `read-write` → capturer le secret one-time → l'utiliser comme `Authorization: Bearer kesh_pat_…` (contexte Playwright dédié `extraHTTPHeaders`) sur un `GET /api/v1/...` (**200**) et un `POST` autorisé (**200/201**, scope read-write) → révoquer via l'UI → réutiliser le token → **401**. + cas clé `read` : `POST` → **403 API_KEY_READ_ONLY**. `data-testid` préfixe `api-keys-*`.

## Tasks / Subtasks

- [x] **T-B1 — Feature module `api-keys` (api + types)** (AC: #1)
  - [x] `api-keys.types.ts` : `ApiKey`, `NewApiKeyPayload`, `CreatedApiKey`, `ApiKeyScope = 'read' | 'read-write'`. Dates = `string` (NaiveDateTime sans TZ).
  - [x] `api-keys.api.ts` : `listApiKeys()`/`createApiKey()`/`revokeApiKey(id, version)` via `apiClient` sur `/api/v1/settings/api-keys` ; `delete(url, { version })` (204). Types ré-exportés depuis le module api. + `api-keys.api.test.ts` (4 tests Vitest).

- [x] **T-B2 — Util clipboard HTTP-LAN-safe** (AC: #4)
  - [x] `frontend/src/lib/shared/utils/clipboard.ts` : `copyToClipboard()` — `navigator.clipboard?.writeText` puis fallback `document.execCommand('copy')`. + `clipboard.test.ts` (4 tests : moderne, fallback HTTP, writeText rejette→fallback, échec→false).

- [x] **T-B3 — Page `/settings/api-keys` + création + secret one-time + révocation** (AC: #2, #3, #4, #5)
  - [x] `routes/(app)/settings/api-keys/+page.svelte` : runes Svelte 5, `onMount`, state-machine `mode`, form inline (nom `maxlength=255` validé, `<select>` natif scope [évite bug #143 Select sur HTTP LAN], date `min=demain`), encart secret one-time (copier + toast + avertissement + fermer), révocation inline-confirm + 409. IDs DOM statiques (page singleton, pas de `crypto.randomUUID`).
  - [x] `data-testid` stables (page-title/create-button/name-input/scope-select/expires-input/submit/secret(-value/-copy/-close)/api-key-row-<id>/revoke-<id>/revoke-confirm(-button)/empty/list).

- [x] **T-B4 — Lien depuis `/settings`** (AC: #6)
  - [x] Section « Clés API » dans `routes/(app)/settings/+page.svelte` + `data-testid="settings-api-keys-manage-link"`, clés `settings-api-keys-{title,manage,hint}`.

- [x] **T-B5 — i18n FR/DE/IT/EN** (AC: #7)
  - [x] 36 clés (`api-keys-*` ×33 + `settings-api-keys-*` ×3) dans les 4 `.ftl`, même ordre. Placeholder `{ $date }` (Fluent, interpolé frontend). `lint-i18n-ownership` vert.

- [x] **T-B6 — E2E Playwright** (AC: #9)
  - [x] `frontend/tests/e2e/api-keys.spec.ts` (2 tests : read-write GET 200/POST 201/révoqué→401 ; read POST→403 `API_KEY_READ_ONLY`). Token PAT réel via `playwrightRequest.newContext({ extraHTTPHeaders: { Authorization: Bearer } })`. Parse validé (`--list`). **Run complet = gate séparé** (stack live + browsers, hors CI principale).

- [x] **T-B7 — Quality gate** (AC: #8, #9)
  - [x] `npm run check` ✅ (0 erreur), `lint-i18n-ownership` ✅, `npm run test:unit` ✅ **275/275** (8 nouveaux), `npm run build` ✅, `cargo test -p kesh-i18n` ✅ **21/21**. E2E full-run avant PR.

## Dev Notes

> Frontend uniquement. Le backend (routes, codes d'erreur, audit) est figé par 17-2a (PR #168). Ne pas modifier le Rust sauf ajout de clés `.ftl`.

### Cartographie de réutilisation (analyse exhaustive — 3 agents Explore)

**Feature module / API client** :
- Calque : `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` + `.types.ts`.
- `apiClient` : `$lib/shared/utils/api-client` — `.get<T>()`/`.post<T>()`/`.delete<T>(path, body)`. Chemins relatifs. Erreurs → `ApiError` (`$lib/shared/types/api` : `{ code, message, details?, status }`), guard `isApiError(err)`.
- **`delete` envoie le body** (`{ version }`) — pattern confirmé `archiveBankAccount(id, version)`.

**Page CRUD** :
- Calque : `frontend/src/routes/(app)/bank-accounts/+page.svelte`. Svelte **5.54** runes (`$state`/`$derived`/`$derived.by`), `onMount`, state-machine `mode` (`none`/`create`/`revoke-confirm`), formulaire inline (pas Dialog), encart confirmation destructive inline (`archive-confirm` calque).
- Composants : `Button` (`$lib/components/ui/button` ; variants `default`/`outline`/`ghost`/`destructive`), `Input` (`$lib/components/ui/input`), `Select` (`$lib/components/ui/select`), `Table` (`$lib/components/ui/table`). `toast` (`svelte-sonner`). Icônes `@lucide/svelte` (`Copy`, `Eye`, `Trash2`, …).

**Settings** :
- `frontend/src/routes/(app)/settings/+page.svelte` : sections `<section class="rounded-lg border …">` avec `<h2>` + `<Button href=…>`. Ajouter section « Clés API » après « Comptes bancaires ».

**i18n** :
- Mécanisme frontend : `i18nMsg(key, fallback, args?)` depuis `$lib/shared/utils/i18n.svelte`. Hydraté au layout depuis `GET /api/v1/i18n/messages` (servi par `crates/kesh-api/src/routes/i18n.rs` depuis les `.ftl`).
- **Les fichiers de locales sont backend** : `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (format Fluent). Pas de fichier de locale séparé côté frontend.
- `lint-i18n-ownership.js` : feature multi-segment (`api-keys`) → match préfixe `api-keys-`. Namespaces globaux autorisés : `error`/`tooltip`/`common`/`mode`/`shortcut`/`demo`. Les 2 codes `error-api-key-*` (backend 17-2a) sont namespace `error` (global) → utilisables partout.
- Template de nommage `bank-accounts-*` : `…-labels-page-title`, `…-actions-create`, `…-confirm-archive`, `…-toast-create-success`, `…-errors-*`.

**E2E Playwright** :
- `frontend/tests/e2e/*.spec.ts` (`testMatch` strict `.spec.ts`), `workers: 1`, `locale fr-CH`, `baseURL http://127.0.0.1`. `testDir tests/e2e`.
- Helpers : `./helpers/test-state` (`seedTestState('with-company')` = admin/admin123 + company + fiscal_year + 5 comptes ; `authedApiContext(page)` clone le `storageState` cookies httpOnly → `APIRequestContext` ; `disposeContextSafe`). Login form : `#username`/`#password`/`button[type=submit]`.
- **Token PAT** : créer un `request.newContext({ baseURL, extraHTTPHeaders: { Authorization: 'Bearer ' + secret } })` séparé (le PAT n'est pas dans les cookies). `disposeContextSafe` en `finally`.
- Robustesse : `data-testid` + `getByRole`, `toBeVisible({ timeout })`, pas de `waitForTimeout`, suffixes uniques. Éviter de dépendre des flux fragiles (KF-029/30/31/32 onboarding/bank).

### Invariants à NE PAS casser (régressions)

- Ne pas modifier le contrat backend (`routes/api_keys.rs`, `lib.rs`). La route est `/api/v1/settings/api-keys`.
- `lint-i18n-ownership` : aucune clé non-`api-keys-`/non-globale dans `features/api-keys/` ni dans la page (hors `settings-*` pour la section settings).
- Pas d'API secure-context-only en runtime (`navigator.clipboard`, `crypto.randomUUID`, `crypto.subtle`) sans fallback → page blanche sur HTTP LAN ([[feedback_no_secure_context_apis_http_lan]]).
- Le secret `key` n'apparaît **que** dans la réponse de création ; ne jamais le logger, le persister en store, ni l'envoyer ailleurs.

### Sécurité — points d'attention

- Le secret one-time ne doit pas survivre à un reload (state éphémère, pas de `localStorage`).
- Avertissement clair « copiez maintenant, non re-affichable ».
- La page est session-JWT-cookie uniquement ; un PAT ne peut pas l'atteindre (DC6 backend = filet). Pas de logique frontend spécifique requise.

### Project Structure Notes

Nouveaux fichiers : `frontend/src/lib/features/api-keys/{api-keys.api.ts,api-keys.types.ts}`, `frontend/src/routes/(app)/settings/api-keys/+page.svelte`, `frontend/src/lib/shared/utils/clipboard.ts` (si absent), `frontend/tests/e2e/api-keys.spec.ts`. Modifiés : `frontend/src/routes/(app)/settings/+page.svelte`, les 4 `crates/kesh-i18n/locales/*/messages.ftl`.

### Dépendances inter-stories

- **Dépend de 17-2a** (mergée PR #168) : routes + codes 403. ✅ débloquée.
- **Bloque 17-2c** (doc/OpenAPI) : la doc référencera la page livrée ici.

### References

- [Source: _bmad-output/implementation-artifacts/17-2-api-pat-integrations.md §Partie B + AC9-11] — scope frontend parent.
- [Source: _bmad-output/implementation-artifacts/17-2a-api-pat-backend.md + crates/kesh-api/src/routes/api_keys.rs] — contrat backend réel.
- [Source: frontend/src/lib/features/bank-accounts/ + routes/(app)/bank-accounts/+page.svelte + routes/(app)/settings/+page.svelte] — patterns calque.
- [Source: frontend/scripts/lint-i18n-ownership.js] — règle de propriété i18n.
- [Source: frontend/tests/e2e/helpers/test-state.ts + accounts.spec.ts + fiscal-years.spec.ts] — patterns E2E + token bearer.
- Mémoires : [[feedback_no_secure_context_apis_http_lan]] (clipboard/crypto HTTP LAN), [[reference_playwright_ubuntu26]] (E2E Ubuntu).

## Change Log — split

- 2026-06-06 `bmad-create-story 17-2b` — Partie B frontend extraite de la spec parente 17-2 (convergée 5 passes). Contrat backend figé par 17-2a (PR #168 mergée + CI verte). Analyse exhaustive via 3 agents Explore (feature/page patterns, i18n + lint-ownership, E2E Playwright + token bearer). Ground-truth corrigé : route = `/api/v1/settings/api-keys`. Décisions DC-B1..B6. AC1-9, tasks T-B1..T-B7. Recommandation modèle dev-story : Opus ou Sonnet (rollout frontend mécanique mais avec gotchas HTTP-LAN clipboard + i18n ownership). Re-validate optionnel (contenu déjà adversarialement revu en parent). Prochaine : `bmad-create-story validate 17-2b` (optionnel) OU `bmad-dev-story 17-2b`.

## Dev Agent Record

### Agent Model Used

Opus 4.8 (1M context) — `bmad-dev-story 17-2b`, single-pass T-B1→T-B7.

### Debug Log References

- `npm run check` : 0 erreur (25 warnings `state_referenced_locally`/a11y pré-existants dans d'autres fichiers, hors-scope).
- `npm run lint-i18n-ownership` : PASS (préfixe `api-keys-` ; la page vit sous `routes/` donc hors scope du linter qui ne scanne que `src/lib/features/`).
- `npm run test:unit` : 31 fichiers / **275 tests** verts (dont 8 nouveaux : 4 `api-keys.api` + 4 `clipboard`).
- `npm run build` : OK (adapter-static).
- `cargo test -p kesh-i18n` : **21 tests** verts (`load_all_locales` + parse des 4 `.ftl` avec les 36 nouvelles clés).
- E2E `playwright test api-keys --list` : 2 tests parsés OK. Run complet non exécuté (nécessite backend `KESH_TEST_MODE` + preview + browsers — gate séparé hors CI principale).

### Completion Notes List

- **Route ground-truth** : `/api/v1/settings/api-keys` (préfixe `settings/`). Le `revokeApiKey` envoie `version` en **body** DELETE (convention projet `bank-accounts::archive`, confirmée code-review 17-2a) ; backend `204`.
- **Clipboard HTTP-LAN-safe** (#145) : `copyToClipboard` tente `navigator.clipboard` (secure-context) puis fallback `execCommand('copy')` — fonctionne sur déploiement HTTP NAS. Pas de `crypto.randomUUID` (IDs DOM statiques, page singleton).
- **`<select>` natif** pour le scope (pas le composant `Select` de la lib) — cohérent `bank-accounts` + évite le bug #143 (scroll Select sur HTTP LAN).
- **Secret one-time** : encart `createdSecret` éphémère (state, jamais `localStorage`), persistant jusqu'à fermeture explicite + avertissement « copiez maintenant ». Le `GET` ne renvoie jamais le secret.
- **i18n** : 36 clés × 4 locales. Le préfixe `api-keys-` est exclusif au sens lint, mais la page étant sous `routes/` n'est pas scannée — discipline préfixe respectée par hygiène/futur-proofing. Placeholder `{ $date }` rendu littéral par `all_messages` (None args) → interpolé côté frontend par `i18nMsg`.
- **Aucun changement backend Rust** hormis l'ajout de clés `.ftl` (contrat 17-2a figé). **0 nouvelle dépendance npm**.
- **Validation client + surface des 400 backend** : `name` non-vide ≤ 255 + `expiresAt` futur (`min=demain` sur le date-picker) ; les erreurs backend (`400`, `409 OPTIMISTIC_LOCK_CONFLICT`) sont surfacées (form-error / toast + reload).
- **DC6** : la page est session-JWT-cookie uniquement ; aucune logique frontend spécifique (le `403 API_KEY_MANAGEMENT_FORBIDDEN` backend reste un filet).

### File List

**Nouveaux fichiers :**
- `frontend/src/lib/features/api-keys/api-keys.types.ts`
- `frontend/src/lib/features/api-keys/api-keys.api.ts`
- `frontend/src/lib/features/api-keys/api-keys.api.test.ts`
- `frontend/src/lib/shared/utils/clipboard.ts`
- `frontend/src/lib/shared/utils/clipboard.test.ts`
- `frontend/src/routes/(app)/settings/api-keys/+page.svelte`
- `frontend/tests/e2e/api-keys.spec.ts`

**Fichiers modifiés :**
- `frontend/src/routes/(app)/settings/+page.svelte` (section « Clés API » + lien)
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` (36 clés)
- `crates/kesh-i18n/locales/de-CH/messages.ftl` (36 clés)
- `crates/kesh-i18n/locales/it-CH/messages.ftl` (36 clés)
- `crates/kesh-i18n/locales/en-CH/messages.ftl` (36 clés)

### Change Log

- 2026-06-06 `bmad-dev-story 17-2b` (Opus 4.8) — implémentation single-pass T-B1→T-B7. Feature module `api-keys` (api/types + test), util `clipboard` HTTP-LAN-safe (+ test), page `/settings/api-keys` (liste + création secret one-time + révocation inline-confirm), lien depuis `/settings`, 36 clés i18n × 4 locales, E2E `api-keys.spec.ts` (2 tests). Quality gate vert : check 0-err, lint-i18n PASS, unit 275/275, build OK, kesh-i18n 21/21. Status `in-progress → review`. Prochaine : `bmad-code-review 17-2b` (LLM différent — Sonnet/Haiku). E2E full-run avant PR.
