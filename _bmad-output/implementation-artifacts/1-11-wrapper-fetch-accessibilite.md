# Story 1.11: Wrapper fetch & accessibilité

Status: done

## Story

As a développeur frontend,
I want un client API robuste et une base d'accessibilité,
so that toutes les communications avec l'API soient fiables et l'interface accessible.

## Acceptance Criteria

1. **AC#1 — Refresh automatique** : Given un access_token expiré, When le wrapper fetch détecte 401, Then refresh automatique du token via `POST /api/v1/auth/refresh`, si échec → redirection login.
2. **AC#2 — Erreurs structurées** : Given une erreur API, When réponse 4xx/5xx, Then le wrapper parse l'erreur structurée `{ error: { code, message, details? } }` et la rend disponible au composant appelant.
3. **AC#3 — État loading** : Given une requête en cours, When loading, Then variable loading booléenne disponible pour afficher spinner/skeleton.
4. **AC#4 — Zoom 200%** : And interface fonctionnelle à 200% de zoom navigateur.
5. **AC#5 — Raccourcis clavier** : And raccourcis clavier : Ctrl+S sauvegarder, Tab/Shift+Tab navigation formulaires.
6. **AC#6 — axe-core** : And axe-core configuré pour les tests d'accessibilité.

## Tasks / Subtasks

- [x] **T1 — Types API partagés** (AC: #2)
  - [x] T1.1 Créer `$lib/shared/types/api.ts` avec les types `ApiError` et `PaginatedResponse<T>` conformes à l'architecture
  - [x] T1.2 `ApiError` : `{ code: string; message: string; details?: Record<string, unknown>; status: number }`
  - [x] T1.3 `PaginatedResponse<T>` : `{ items: T[]; total: number; offset: number; limit: number }`
- [x] **T2 — Fetch wrapper (`api-client.ts`)** (AC: #1, #2, #3)
  - [x] T2.1 Créer `$lib/shared/utils/api-client.ts` — wrapper autour de `fetch()` natif
  - [x] T2.2 Ajouter automatiquement `Authorization: Bearer {accessToken}` sur chaque requête (sauf `/auth/login`, `/auth/logout`, et `/auth/refresh`)
  - [x] T2.3 Intercepter les réponses 401 : tenter un refresh via `POST /api/v1/auth/refresh` avec `{ refreshToken }`, puis retry la requête originale avec le nouveau token
  - [x] T2.4 Si le refresh échoue (401 `INVALID_REFRESH_TOKEN` ou erreur réseau) : appeler `authState.clearSession()` (nouvelle méthode, cleanup state SANS appel API — le token est déjà invalide côté serveur) + `window.location.replace('/login?reason=session_expired')`
  - [x] T2.5 Token rotation : après un refresh réussi, mettre à jour `authState` avec les nouveaux `accessToken`, `refreshToken`, `expiresIn`
  - [x] T2.6 Mutex de refresh : si plusieurs requêtes reçoivent 401 simultanément, une seule doit faire le refresh — les autres attendent le résultat. Si le retry après refresh échoue à nouveau en 401 → ne PAS retenter le refresh (guard anti-boucle infinie), déclencher `clearSession()` + redirect login
  - [x] T2.7 Parser les erreurs structurées (`{ error: { code, message, details? } }`) et retourner un `ApiError` typé
  - [x] T2.8 Exposer une interface simple : `apiClient.get<T>(url)`, `apiClient.post<T>(url, body)`, `apiClient.put<T>(url, body)`, `apiClient.delete(url)` — toutes retournent `Promise<T>` ou throw `ApiError`
  - [x] T2.9 Gérer les cas limites : réponse non-JSON (HTML erreur proxy), réseau injoignable, timeout
- [x] **T3 — Refactoring page de login + message session expirée** (AC: #1, #2)
  - [x] T3.1 Refactorer `routes/login/+page.svelte` pour utiliser `apiClient` au lieu de `fetch()` direct — login utilise l'exclusion par URL (T2.2 : `/auth/login` sans Authorization header ni refresh)
  - [x] T3.2 Simplifier le error handling du login grâce à `ApiError` typé
  - [x] T3.3 Détecter le paramètre URL `?reason=session_expired` et afficher un message contextuel « Votre session a expiré. Veuillez vous reconnecter. » avec icône `Clock` (Lucide) — satisfait FR72 (modal session expirée) sans implémentation de modal complète
- [x] **T4 — Accessibilité fondamentale** (AC: #4, #5)
  - [x] T4.1 Vérifier que l'interface est fonctionnelle à 200% de zoom navigateur (layout, login, sidebar) — scroll horizontal acceptable avec `min-w-[1280px]`
  - [x] T4.2 Implémenter le raccourci `Ctrl+S` global (sauvegarde — émet un événement custom `kesh:save` sur `window`, les composants écoutent)
  - [x] T4.3 Vérifier que Tab/Shift+Tab navigue correctement dans tous les formulaires existants (login)
  - [x] T4.4 Vérifier focus trap dans les modales/dropdowns (shadcn-svelte/Bits UI le gère nativement — confirmer que Échap ferme les modales/dropdowns, flèches naviguent dans les menus)
- [x] **T5 — Tests** (AC: #1-#6)
  - [x] T5.1 Tests unitaires Vitest : `$lib/shared/utils/api-client.test.ts`
    - GET/POST/PUT/DELETE avec Authorization header
    - Refresh automatique sur 401 + retry
    - Refresh échoué → logout + redirect
    - Mutex de refresh (2 requêtes 401 simultanées)
    - Parsing erreurs structurées → ApiError
    - Réseau injoignable → erreur typée
    - 503 SERVICE_UNAVAILABLE → ApiError avec message DB (FR89)
    - Guard anti-boucle : retry après refresh retourne 401 → clearSession (pas de 2e refresh)
  - [x] T5.2 ~~Tests unitaires types~~ — Supprimé : `api.ts` ne contient que des interfaces TypeScript (pas de runtime guards). Pas de fichier de test nécessaire
  - [x] T5.3 Tests E2E Playwright : `frontend/tests/e2e/auth.spec.ts`
    - Login réussi → redirection accueil, affichage header/sidebar
    - Login échoué → message d'erreur affiché
    - Accès page protégée sans auth → redirect login
    - Raccourci Ctrl+S déclenche l'événement kesh:save
  - [x] T5.4 Tests accessibilité axe-core : intégrer `@axe-core/playwright` dans les tests E2E — `checkA11y()` sur page login et layout principal

## Dev Notes

### API Client — Architecture

**Fichier cible :** `frontend/src/lib/shared/utils/api-client.ts`

Le wrapper encapsule `fetch()` natif (zéro dépendance) et fournit :
1. Injection automatique du header `Authorization: Bearer {accessToken}`
2. Interception 401 → refresh token → retry transparent
3. Parsing erreurs structurées → `ApiError`
4. Interface typée GET/POST/PUT/DELETE

```typescript
// Pattern d'utilisation par les composants
import { apiClient } from '$lib/shared/utils/api-client';

// GET typé
const users = await apiClient.get<User[]>('/api/v1/users');

// POST typé
const newUser = await apiClient.post<User>('/api/v1/users', { username, password, role });

// PUT avec version (optimistic locking)
const updated = await apiClient.put<User>(`/api/v1/users/${id}`, { ...data, version });

// DELETE
await apiClient.delete(`/api/v1/users/${id}`);
```

### Token Refresh — Flux détaillé

```
Requête API → 401 reçu
  → refreshLock acquis ?
    → OUI : attendre le résultat du refresh en cours
    → NON : acquérir le lock
      → POST /api/v1/auth/refresh { refreshToken: authState.refreshToken }
        → 200 : { accessToken, refreshToken, expiresIn } (camelCase)
          → authState.login(newAccessToken, newRefreshToken, newExpiresIn)
          → retry requête originale avec nouveau token
          → libérer le lock
        → 401 (INVALID_REFRESH_TOKEN) ou erreur réseau :
          → authState.clearSession() (sans fetch logout — le token est déjà invalide)
          → window.location.replace('/login?reason=session_expired')
          → libérer le lock
  → Retry de la requête originale avec nouveau token
    → Si retry retourne encore 401 : NE PAS retenter le refresh (guard anti-boucle)
      → clearSession() + redirect login
```

**Mutex de refresh :** Utiliser un `Promise` partagé. Quand la première requête 401 démarre le refresh, les suivantes reçoivent la même Promise et attendent son résultat.

```typescript
let refreshPromise: Promise<boolean> | null = null;

async function refreshTokens(): Promise<boolean> {
  if (refreshPromise) return refreshPromise;
  refreshPromise = doRefresh();
  try { return await refreshPromise; }
  finally { refreshPromise = null; }
}
```

### Backend API — Contrats vérifiés

**Tous les DTOs utilisent `#[serde(rename_all = "camelCase")]` — les clés JSON sont en camelCase.**

**Endpoint refresh :** `POST /api/v1/auth/refresh`
- Body : `{ "refreshToken": "uuid" }`
- Succès 200 : `{ "accessToken": "jwt...", "refreshToken": "new-uuid", "expiresIn": 900 }`
- Échec 401 : `{ "error": { "code": "INVALID_REFRESH_TOKEN", "message": "Session expirée" } }`
- **Token rotation** : chaque refresh invalide l'ancien token et en émet un nouveau
- **Détection de vol** : si un token déjà rotaté est réutilisé → mass revoke de tous les tokens de l'utilisateur

**Format d'erreur structuré (toutes routes) :**
```json
{ "error": { "code": "ERROR_CODE", "message": "Message lisible" } }
```
Codes connus : `INVALID_CREDENTIALS`, `UNAUTHENTICATED`, `RATE_LIMITED`, `INVALID_REFRESH_TOKEN`, `OPTIMISTIC_LOCK_CONFLICT`, `RESOURCE_CONFLICT`, `FORBIDDEN`, `NOT_FOUND`, `VALIDATION_ERROR`, `CANNOT_DISABLE_SELF`, `CANNOT_DISABLE_LAST_ADMIN`, `SERVICE_UNAVAILABLE`, `INTERNAL_ERROR`

**Statuts HTTP et traitement frontend :**
| Status | Code erreur | Action frontend |
|--------|-------------|-----------------|
| 400 | `VALIDATION_ERROR` | Détails sous les champs (futur) |
| 400 | `CANNOT_DISABLE_SELF`, `CANNOT_DISABLE_LAST_ADMIN` | Message d'erreur métier dans le composant |
| 401 | `UNAUTHENTICATED` | JWT expiré/absent → déclenche refresh automatique (T2.3) |
| 401 | `INVALID_CREDENTIALS` | Message erreur login (pas de refresh) |
| 401 | `INVALID_REFRESH_TOKEN` | Redirect `/login?reason=session_expired` (session expirée) |
| 403 | `FORBIDDEN` | Message « Accès refusé » |
| 409 | `OPTIMISTIC_LOCK_CONFLICT` | Modal conflit de version (futur — erreur retournée au composant) |
| 409 | `RESOURCE_CONFLICT` | Message doublon/conflit (distinct du lock optimiste) |
| 429 | `RATE_LIMITED` | Message « Trop de tentatives » — lire le header `Retry-After` si présent |
| 500 | `INTERNAL_ERROR` | Banner « Erreur serveur » |
| 503 | `SERVICE_UNAVAILABLE` | Banner « Serveur indisponible — vérifiez que la base de données est accessible » (FR89) |

### Auth Store — État actuel (Story 1.10)

Le store `authState` expose déjà :
- `accessToken`, `refreshToken`, `expiresIn` (getters)
- `currentUser: { userId, role }` (décodé du JWT)
- `isAuthenticated` (basé sur `accessToken !== null`)
- `login(accessToken, refreshToken, expiresIn)` — valide le JWT puis affecte
- `logout()` — POST /api/v1/auth/logout (best-effort) + cleanup state

**Modification nécessaire pour T2.4 :** Ajouter une méthode `clearSession()` qui nettoie le state SANS appeler l'API logout (pour le cas où le refresh échoue — le token est déjà invalide côté serveur). `clearSession()` fait la même chose que `logout()` sauf le `fetch()` vers `/api/v1/auth/logout`.

### Raccourcis clavier — Pattern

**Ctrl+S :** Événement custom `kesh:save` sur `window`. Les composants de saisie écoutent cet événement pour déclencher leur action de sauvegarde. Dans cette story, le mécanisme est mis en place — les composants qui l'utilisent viendront dans les stories futures (écritures, factures...).

```typescript
// Dans un composant layout ou un module global
function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault();
    window.dispatchEvent(new CustomEvent('kesh:save'));
  }
}
```

**Tab/Shift+Tab :** Déjà natif dans le navigateur. Vérifier que l'ordre DOM est logique et que les éléments `disabled` ne capturent pas le focus indûment.

### Contraintes de scope

- **PAS de modal conflit de version** — la gestion 409 dans `apiClient` retourne l'erreur au composant, mais la modal UI sera créée dans les stories applicatives (Epic 3+)
- **PAS de validation inline sous les champs** — la gestion 400 parse les `details` mais l'affichage par champ sera dans les stories formulaires
- **PAS de i18n** — strings hardcodés FR (Story 2.1). Déviation assumée de l'architecture règle #6
- **PAS de timer d'inactivité proactif** — le refresh se fait à la détection du 401 (lazy refresh). **Dette sécurité documentée** : FR13 exige la terminaison de session après 15 min d'inactivité. Le lazy refresh seul prolonge la session indéfiniment tant que le refresh token est valide. Un timer d'inactivité (mousemove/keydown → reset, setTimeout → logout) devra être ajouté dans une story de remédiation (Epic 2 ou dette technique post-Epic 1). Impact : un poste non verrouillé reste accessible au-delà de 15 min. Propriétaire : à définir en rétrospective Epic 1
- **PAS de Ctrl+N** — sera implémenté avec le formulaire d'écriture (Epic 3, Story 3.2)
- **PAS de refactoring des futures routes** — seule la page de login est refactorée pour utiliser `apiClient`. Les routes applicatives utiliseront `apiClient` quand elles seront implémentées

### Previous Story Intelligence

**Story 1.10 (layout & login) — Learnings critiques :**
- `auth.svelte.ts` : pattern objet avec getters Svelte 5, JWT decode défensif (validation segments, claims non vides, atomicité)
- Login page : 4 états d'erreur avec icônes Lucide SVG, `aria-live="polite"` permanent dans le DOM, `aria-describedby` conditionnel
- Auth guard `(app)/+layout.ts` : `throw redirect(302, '/login')` avec `export const ssr = false`
- Code review patches : `window.location.replace` (pas href), spinner `aria-hidden`, icônes décoratives `aria-hidden`
- `@axe-core/playwright` déjà installé, intégration E2E reportée à cette story
- `@lucide/svelte` v1.7.0 déjà installé
- Vitest configuré : `vitest/config` import, environment jsdom

**Story 1.6 (refresh token) — Patterns backend :**
- Refresh endpoint : `POST /api/v1/auth/refresh` avec token rotation + détection de vol
- Token rotation : chaque refresh invalide l'ancien (revoked_reason = "rotation")
- Si token déjà rotaté réutilisé → mass revoke (theft_detected)
- Rate limiting uniquement sur `/auth/login` (pas sur refresh)

### Project Structure Notes

- `$lib/shared/utils/` existe avec `.gitkeep` — y créer `api-client.ts`
- `$lib/shared/types/` existe avec `.gitkeep` — y créer `api.ts`
- Proxy Vite : `/api` → `http://localhost:3000` en dev (préfixe)
- Vitest déjà configuré : `vite.config.ts` avec `test: { environment: 'jsdom' }`
- 14 tests unitaires existants dans `auth.svelte.test.ts` — ne pas casser

### References

- [Source: _bmad-output/planning-artifacts/epics.md — Epic 1, Story 1.11]
- [Source: _bmad-output/planning-artifacts/architecture.md — §API Client Pattern, §Error Handling, §Accessibility]
- [Source: _bmad-output/planning-artifacts/prd.md — FR12-FR16, FR71-FR73, FR89]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md — §Accessibilité, §Raccourcis clavier]
- [Source: _bmad-output/implementation-artifacts/1-10-layout-page-de-login.md — Auth store, login patterns, deferred items]
- [Source: crates/kesh-api/src/routes/auth.rs — RefreshRequest/RefreshResponse, token rotation]
- [Source: crates/kesh-api/src/errors.rs — AppError, INVALID_REFRESH_TOKEN]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

Aucun problème majeur rencontré. Une erreur de typage corrigée dans les tests (cast `unknown[]` → `string`).

### Completion Notes List

- ✅ T1 : Interfaces `ApiError` et `PaginatedResponse<T>` créées dans `$lib/shared/types/api.ts`
- ✅ T2 : `apiClient` wrapper fetch complet avec : injection Authorization auto (exclusion auth URLs), refresh 401 avec mutex (Promise partagée), guard anti-boucle infinie, parsing erreurs structurées, gestion réseau/non-JSON, interface GET/POST/PUT/DELETE typée
- ✅ T2 (auth store) : méthode `clearSession()` ajoutée à `authState` — cleanup state sans appel API logout
- ✅ T3 : Page login refactorée pour utiliser `apiClient`, détection `?reason=session_expired` avec message contextuel et icône Clock
- ✅ T4 : Handler Ctrl+S global ajouté dans `(app)/+layout.svelte` via `<svelte:window onkeydown>`, émet événement custom `kesh:save`. Zoom 200% couvert par `min-w-[1280px]` existant. Tab/Shift+Tab natif OK. Focus trap shadcn/Bits UI natif
- ✅ T5 : 16 tests unitaires Vitest (api-client) + 6 tests E2E Playwright (auth + axe-core accessibilité). 30 tests totaux passent (0 régression)

### Change Log

- 2026-04-08 : Implémentation complète Story 1.11 — api-client, refactoring login, accessibilité, tests (Opus 4.6)
- 2026-04-08 : Code review passe 1 (Haiku 4.5 × 3 couches) — 4 patches appliqués : P1 type guard isApiError, P2 tests INVALID_CREDENTIALS/RATE_LIMITED/isApiError, P3 fix race E2E Ctrl+S, P4 validation réponse refresh. 35 tests passent
- 2026-04-08 : Code review passe 2 (Haiku 4.5 × 3 couches, contexte frais) — 2 patches appliqués : P5 timeout AbortController 30s, P6 test refresh JSON malformé + test timeout. Assertions adaptées pour signal AbortSignal. 37 tests passent
- 2026-04-08 : Code review passe 3 (Haiku 4.5 × 3 couches, contexte frais) — 1 patch appliqué : P7 try/catch res.json() dans doRefresh (évite SyntaxError non rattrapé). Acceptance Auditor : PASS, 0 finding. 37 tests passent
- 2026-04-08 : Code review passe 4 (Haiku 4.5 × 3 couches, contexte frais) — 0 patch, 10 findings rejetés. Critère d'arrêt atteint : zéro finding > LOW. Story marquée done

### File List

**Créés :**
- `frontend/src/lib/shared/types/api.ts` — Types ApiError, PaginatedResponse
- `frontend/src/lib/shared/utils/api-client.ts` — Fetch wrapper (JWT, refresh, erreurs)
- `frontend/src/lib/shared/utils/api-client.test.ts` — 16 tests unitaires fetch wrapper
- `frontend/tests/e2e/auth.spec.ts` — 6 tests E2E Playwright (auth + accessibilité axe-core)

**Modifiés :**
- `frontend/src/lib/app/stores/auth.svelte.ts` — Ajout méthode `clearSession()`
- `frontend/src/routes/login/+page.svelte` — Refactoring apiClient + message session expirée
- `frontend/src/routes/(app)/+layout.svelte` — Handler Ctrl+S global
