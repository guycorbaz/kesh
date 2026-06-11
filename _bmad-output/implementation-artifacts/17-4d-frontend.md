# Story 17.4d: Frontend recovery (pages publiques forgot/reset-password + email UI)

Status: review

<!-- Extraite de la spec parente UMBRELLA 17-4 (`17-4-recovery-mot-de-passe.md`), validate CONVERGÉ 6 passes. Contenu déjà adversarialement revu (Partie D : AC17-22 + transverses AC30/31, T-D1..T-D6). Re-validate optionnel. -->
<!-- DÉPEND de 17-4c (DONE : contrat API forgot/reset-password + forgotPasswordEnabled dans /health) et de 17-4a (DONE : plumbing email backend setup/users). BLOQUE 17-4e (E2E Playwright des pages). -->

## Story

As a **utilisateur de Kesh ayant oublié son mot de passe (ou un admin posant l'email de recovery des comptes)**,
I want **une page publique `/forgot-password` (saisie identifiant → message générique), une page publique `/reset-password?token=` (nouveau mot de passe → redirection login), un lien « Mot de passe oublié ? » sur le login affiché seulement si le feature est activé, et le champ email exposé dans le wizard setup et les dialogues users**,
so that **le flux recovery self-service livré par 17-4c soit utilisable de bout en bout dans le navigateur, y compris sur un déploiement HTTP LAN (NAS), sans casser l'email des comptes lors d'une édition (dette ECH2-2)**.

## Contexte & cadrage

**Issue source :** [#122](https://github.com/guycorbaz/kesh/issues/122). Épopée 17-4, Partie D. Position dans le split A–F (SÉRIE) : 17-4a ✅ → 17-4b ✅ → 17-4c ✅ → **17-4d (ici)** → 17-4e (tests) → 17-4f (doc). **BLOQUE 17-4e** (les E2E Playwright ciblent ces pages).

**Contrat API consommé (posé par 17-4c, vérifié ground-truth 2026-06-11) :**
- `POST /api/v1/auth/forgot-password` corps `{ "identifier": string }` → **toujours `200` avec corps VIDE** (le handler retourne `StatusCode::OK` sans JSON — anti-énum DC4), ou `429 RATE_LIMITED` (5 req / 15 min / IP, blocage 30 min, partagé avec reset). ⚠️ Routes montées **seulement si** `KESH_FEATURE_FORGOT_PASSWORD=true` — sinon `404`.
- `POST /api/v1/auth/reset-password` corps `{ "token": string, "newPassword": string }` (serde camelCase) → `200 { "status": "ok" }` | `400 INVALID_OR_EXPIRED_TOKEN` (générique : inconnu/expiré/utilisé/compte désactivé) | `400 VALIDATION_ERROR` (politique mdp, `password_min_length` défaut 12) | `429`.
- `GET /health` → `{ status, db, version, forgotPasswordEnabled }` — **les deux branches** 200/503 portent le flag (DC9).

**Ce qui est DÉJÀ posé côté backend (ne PAS retoucher) :**
- 17-4a : `SetupAdminRequest.email: Option<String>` (`routes/setup.rs`), `UpdateUserRequest.email` + `CreateUserRequest.email`, `UserResponse.email: Option<String>` (`routes/users.rs`), validation format serveur.
- 17-4c : garde `username.contains('@')` → `400` à la création (users + setup) — un username avec `@` est réservé au routage email du recovery.

**Scope 17-4d :** T-D1 module feature `auth-recovery` (api+types+tests vitest) ; T-D2 page `/forgot-password` ; T-D3 page `/reset-password` ; T-D4 lien conditionnel login + store flag ; T-D5 champ email SetupForm + dialogues users CRUD (**ferme la dette ECH2-2**) ; T-D6 i18n ×4 + `lint-i18n-ownership` ; T-D7 quality gate frontend.

**Hors scope :** E2E Playwright des parcours (17-4e) ; tests d'intégration Rust (17-4e) ; doc manuels/.env.example/CHANGELOG (17-4f) ; toute modification backend (le contrat 17-4c est figé) ; colonne email dans le tableau users (lecture via dialogue edit suffit v0.2, cf. décision DD-5).

## Décisions de conception applicables

- **DC4 (héritée)** — `/forgot-password` affiche **toujours** le même message générique de succès (« Si un compte correspond, un email a été envoyé »), que l'appel matche ou non. Seuls `429` (rate-limit) et erreur réseau affichent autre chose. **Jamais** de distinction existant/inexistant côté UI.
- **DC9 (héritée)** — le lien « Mot de passe oublié ? » du login est conditionné sur `forgotPasswordEnabled` lu via `/health`. Feature off → lien absent (et les pages, si visitées directement, reçoivent `404` de l'API → message générique d'indisponibilité).
- **DD-1 — store flag dédié** : créer `frontend/src/lib/shared/utils/feature-flags.svelte.ts` (pattern exact `app-version.svelte.ts:17-31`) avec `forgotPasswordEnabled` (défaut `false` = lien masqué tant que `/health` n'a pas répondu). Peuplé aux **deux** points qui parsent déjà `/health` : boot `+layout.svelte:54-66` et `pollHealth` `api-health.svelte.ts:55-73` (étendre le type local `{ db?, version?, forgotPasswordEnabled? }`).
- **DD-2 — pages publiques hors `(app)`** : `routes/forgot-password/+page.svelte` + `routes/reset-password/+page.svelte` à la racine de `routes/` (pattern `/login`, AUCUN guard — pas le pattern `/setup` qui redirige les authentifiés : un user authentifié qui suit un lien email reset valide doit pouvoir l'utiliser).
- **DD-3 — corps vide du 200 forgot-password** : vérifier au dev comment `apiClient.post` traite un `200` sans corps JSON (risque `res.json()` qui throw). Si le client suppose du JSON, utiliser le traitement adapté dans `auth-recovery.api.ts` (ex. variante qui ne parse pas le corps) — **ne pas modifier le backend** (contrat figé).
- **DD-4 — sémantique PUT users (ECH2-2)** : le dialogue edit envoie **TOUJOURS** `email` dans le corps du PUT (la valeur éditée, ou la valeur courante inchangée, ou `null` si l'admin vide le champ délibérément). Ajouter `email` au type frontend `UserResponse` (`lib/shared/types/user.ts:5-13`, actuellement absent).
- **DD-5 — email optionnel partout côté UI** : SetupForm et dialogues users acceptent un email vide (compte non-recouvrable par self-service → break-glass #121, c'est un choix utilisateur). Validation UI minimale « contient `@` » si non-vide (AC21) ; le serveur valide le format complet. Hint i18n recommandant de le renseigner (recovery).
- **AC31 (héritée, HTTP-LAN safe)** — `$props.id()` pour tous les IDs DOM générés (pattern `ContactPicker.svelte:19-27`) ; AUCUNE API secure-context-only (`crypto.randomUUID`/`subtle`/`clipboard`) — cf. `feedback_no_secure_context_apis_http_lan`, bugs #143/#145. Les pages recovery sont précisément celles servies en HTTP LAN.

## Acceptance Criteria

> Numérotation continue de l'umbrella (AC17-22, Partie D + transverses 30/31).

17. **Page publique `/forgot-password`** (`frontend/src/routes/forgot-password/+page.svelte`, racine routes, pas de guard — DD-2) : un champ « nom d'utilisateur ou email » + bouton « Envoyer le lien de réinitialisation » → `requestPasswordReset(identifier)`. Après soumission : **toujours** le message générique de succès (DC4), y compris pour un identifiant inconnu. `429` → message rate-limit (pattern login `+page.svelte:58-60`). Erreur réseau/`404` (feature off) → message générique d'indisponibilité. Bouton désactivé pendant la requête + champ requis non-vide. Lien retour vers `/login`. `data-testid` sur champ/bouton/messages (pour 17-4e).
18. **Page publique `/reset-password`** (`frontend/src/routes/reset-password/+page.svelte`) : lit `token` du query param (`$page.url.searchParams`) ; champs « nouveau mot de passe » + « confirmer » (validation locale : longueur ≥ 12 calque `SetupForm.svelte:44`, correspondance) ; `resetPassword(token, newPassword)`. Succès → message + lien/redirection `/login`. `400 INVALID_OR_EXPIRED_TOKEN` → « lien invalide ou expiré » + bouton « Refaire une demande » vers `/forgot-password`. `400 VALIDATION_ERROR` → message politique mdp. `429` → message rate-limit. Token absent du query param → état « lien invalide » direct (pas d'appel API). **HTTP-LAN safe** (AC31/DD-6). `data-testid` (17-4e).
19. **Module feature** `frontend/src/lib/features/auth-recovery/` : `auth-recovery.api.ts` (`requestPasswordReset(identifier: string): Promise<void>` — tolère le corps vide DD-3 ; `resetPassword(token: string, newPassword: string): Promise<void>`) + `auth-recovery.types.ts` + tests unitaires vitest (pattern `bank-import.api.test.ts:1-133` : mock fetch, assert URL/corps camelCase/erreurs mappées). Erreurs typées via `isApiError` (codes `INVALID_OR_EXPIRED_TOKEN`, `VALIDATION_ERROR`, `RATE_LIMITED`).
20. **Lien « Mot de passe oublié ? »** sur `/login` (`routes/login/+page.svelte`, sous le champ password) **conditionné** sur le store `forgotPasswordEnabled` (DD-1). Feature off ou `/health` pas encore répondu → lien absent du DOM.
21. **Champ `email` dans SetupForm** (`lib/features/setup/SetupForm.svelte` après les champs password ; `setupAdmin` étendu `email?: string` → corps `{ username, password, email? }`) **et** dans les dialogues users (`routes/(app)/users/+page.svelte`) : create (`createEmail`, POST étendu) et edit (`editEmail` initialisé depuis `editUser.email`, **PUT envoie toujours `email`** — DD-4, ferme ECH2-2). Type frontend `UserResponse.email: string | null` ajouté. Validation UI « contient `@` » si non-vide (DD-5) ; email vide envoyé comme absent/`null` (cohérent backend `Option<String>`).
22. **i18n ×4 locales** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`) : bloc feature-scoped `auth-recovery-*` (titres, labels, message générique succès, lien invalide/expiré, rate-limit, indisponible) + clés setup/users pour le champ email (`setup-email-*` dans le bloc setup, `users-*` ou clé globale `user-field-email` selon l'ownership du linter). `npm run lint-i18n-ownership` PASS (AC30) — les pages publiques utilisent `i18nMsg(key, fallback)` avec fallback FR (même mécanique que `/login` et `/setup` pré-auth, cf. note dev i18n).

### Transverses

- **Sécurité** : le token n'apparaît jamais dans un log frontend ni dans un toast ; pas d'écho de l'identifiant dans le message de succès (anti-énum) ; pas de nouvelle surface CSRF (endpoints publics sans cookie).
- **Aucune modification backend.** Build/tests verts standalone (T-D7).

## Tasks / Subtasks

- [x] **T-D1** Module `lib/features/auth-recovery/` : `auth-recovery.api.ts` + `.types.ts` + `auth-recovery.api.test.ts` (vitest, mock fetch — vérifier corps camelCase `newPassword`, tolérance corps vide du 200 forgot DD-3, mapping erreurs). (AC: 19)
- [x] **T-D2** Page `/forgot-password` (runes Svelte 5, composants `Input`/`Button` ui, zone erreur `role="alert" aria-live="polite"` calque login `+page.svelte:95-115`, `data-testid`). (AC: 17)
- [x] **T-D3** Page `/reset-password` (token query param, 2 champs password + validation locale, états succès/erreurs typées, `$props.id()` pour IDs DOM, `data-testid`). (AC: 18, 31)
- [x] **T-D4** Store `feature-flags.svelte.ts` (DD-1) peuplé au boot `+layout.svelte:54-66` + `pollHealth` `api-health.svelte.ts:55-73` ; lien conditionnel sur `/login`. (AC: 20)
- [x] **T-D5** Email UI : `UserResponse.email` type frontend + `SetupForm` (state + validation + `setupAdmin` signature) + dialogue create users (`createEmail`) + dialogue edit users (`editEmail` pré-rempli, PUT envoie toujours `email` — ferme ECH2-2). (AC: 21)
- [x] **T-D6** Clés FTL ×4 locales (`auth-recovery-*` + email setup/users) + `npm run lint-i18n-ownership` PASS. (AC: 22, 30)
- [x] **T-D7** Quality gate Test Locally First **frontend** : `npm run check` + `npm run lint-i18n-ownership` + `npm run test:unit` + `npm run build` (depuis `frontend/`). Backend non touché → `cargo` non requis sauf si les `.ftl` changent (alors `cargo test -p kesh-i18n` rapide). (AC: transverse)

## Dev Notes

### Ground-truth frontend (exploration 2026-06-11, 83 lectures)

**Page login (`frontend/src/routes/login/+page.svelte:1-169`) :**
- Champs username/password lignes 122-145, `apiClient.post('/api/v1/auth/login', …)` lignes 35-42, gestion erreurs : 401 → message credentials (55-57), **429 → « Trop de tentatives »** (58-60), NETWORK_ERROR (52-54), zone `role="alert" aria-live="polite"` (95-115). Icônes Lucide (`AlertTriangle`, `Clock`, `WifiOff`, `XCircle`). Insérer le lien recovery sous le champ password (~ligne 145), conditionné DD-1.

**Guards & routes publiques :**
- `(app)/+layout.ts:9-46` redirige les non-authentifiés vers `/login` — les nouvelles pages NE vont PAS sous `(app)`.
- `/login` : aucun guard (un authentifié peut la visiter). `/setup/+layout.ts:21-25` redirige les authentifiés vers `/` — **ne PAS reprendre ce pattern** pour les pages recovery (DD-2 : un user authentifié suivant un lien reset valide doit pouvoir reset).
- Root `+layout.ts:15-19` : `ssr = false`, CSR only.

**Interceptor 401 (`api-client.ts:400-418`) :** sur 401 hors `AUTH_EXCLUDED_URLS`, tente un refresh puis `window.location.replace('/login?reason=session_expired')`. Les 2 endpoints recovery sont publics (jamais de 401) → pas de problème. Ne PAS appeler d'API auth-required depuis ces pages.

**`/health` :** consommé à 2 endroits — boot `+layout.svelte:54-66` (`fetch('/health', { signal: AbortSignal.timeout(2000) })`, parse `{ db?, version? }`, `appVersion.set`) et `pollHealth` `api-health.svelte.ts:55-73` (même shape). **Étendre les 2** avec `forgotPasswordEnabled?` → store DD-1. Pattern store : `app-version.svelte.ts:17-31` (`$state` + getter + `set()`).

**apiClient (`lib/shared/utils/api-client.ts`) :** `apiClient.post<T>(url, body)`, credentials `include` (268), erreurs parsées `parseErrorResponse()` → `{ code, message, details?, status }` (208-240), type guard `isApiError(err)`. **Retry-After du 429 non exploité par le client** — afficher un message générique rate-limit comme le login. ⚠️ **DD-3 : vérifier le comportement sur `200` corps vide** (le forgot-password ne renvoie pas de JSON) avant d'écrire `requestPasswordReset`.

**Pattern feature module :** `lib/features/setup/` (`setup.api.ts:32-57` — `setupAdmin(username, password)` à étendre `email?`) et `lib/features/contacts/` (`contacts.api.ts:1-59`). Tests : `bank-import.api.test.ts:1-133` (vi.stubGlobal fetch, beforeEach authState — pour auth-recovery PAS besoin d'authState, endpoints publics).

**SetupForm (`SetupForm.svelte:1-209`) :** states `username/password/passwordConfirm` (15-17), validations `$derived` (43-46, longueur Unicode-safe `[...password].length >= MIN_PASSWORD`), test file `SetupForm.test.ts:1-102`. Backend `SetupAdminRequest.email` **déjà présent** (17-4a). Ajouter `email` state + `$derived` « vide OU contient @ » + Input `type="email"` + hint.

**Users CRUD (`routes/(app)/users/+page.svelte:1-485`) :** states create (23-31 : `createUsername/Password/Confirm/Role`), edit (`editUser/editRole/editActive`, PAS de `editError`), POST 111-115 (sans email), **PUT 146-149 SANS `email`** → c'est la dette ECH2-2 : backend `UPDATE users SET role = ?, active = ?, email = ?, version = version + 1 …` (sémantique remplacement, email absent = `NULL`). Dialogues : create 351-396, edit 399-433. `UserResponse` frontend (`lib/shared/types/user.ts:5-13`) **sans `email`** — à ajouter (`email: string | null`). Toasts via `svelte-sonner`.

**i18n :** `i18nMsg(key, fallback, args?)` (`i18n.svelte.ts:1-31`), messages chargés via `GET /api/v1/i18n/messages` (`loadI18nMessages`). Sur les pages pré-auth (`/login`, `/setup`), c'est le **fallback** inline qui s'affiche si les messages ne sont pas chargés — même mécanique pour les pages recovery (fallback FR dans le code, clés FTL ×4 pour le rendu post-load). Linter : `frontend/scripts/lint-i18n-ownership.js` — clés feature-scoped utilisables uniquement dans leur dossier feature ; globals `error-*`/`common-*`/`tooltip-*`. Bloc `auth-recovery-*` → utilisable dans `lib/features/auth-recovery/` **et vérifier** si les pages `routes/forgot-password|reset-password` comptent comme la feature (sinon : soit composants de page dans la feature ré-exportés, soit entrée ownership — calquer ce que fait `/setup` avec `setup-*` et `SetupForm` importé par `routes/setup/+page.svelte`).
- ⚠️ Clés FTL = fichiers `crates/kesh-i18n/locales/*-CH/messages.ftl` (crate Rust) → si modifiés, lancer aussi `cargo test -p kesh-i18n` (tests de parité de clés entre locales, si présents) — vérifier au dev.

**HTTP-LAN safe :** `$props.id()` pattern `ContactPicker.svelte:19-27`. Composants : `lib/components/ui/input/input.svelte` (types text/password/email, `aria-invalid`), `button/button.svelte`, `dialog/`, toasts `svelte-sonner`.

**E2E testids (préparer 17-4e) :** pattern setup `setup-username`/`setup-password`/`setup-submit` (`tests/e2e/setup.spec.ts:1-108`) → `forgot-identifier`/`forgot-submit`/`forgot-success`, `reset-password`/`reset-password-confirm`/`reset-submit`/`reset-error`…

### Pièges connus à NE PAS reproduire

- `crypto.randomUUID()`/`navigator.clipboard` sur pages HTTP LAN → page blanche (#145/#143). `$props.id()` uniquement.
- Oublier d'envoyer `email` au PUT users → effacement silencieux (ECH2-2, sémantique remplacement).
- Echo de l'identifiant saisi dans le message de succès forgot → fuite anti-énum (DC4).
- Hardcoder le flag à `true` par défaut dans le store → lien visible avant la réponse `/health` sur une install feature-off.

### References

- [Source: umbrella `_bmad-output/implementation-artifacts/17-4-recovery-mot-de-passe.md` — Partie D AC17-22, T-D1..T-D6, transverses AC30/31]
- [Source: `_bmad-output/implementation-artifacts/17-4c-backend-endpoints.md` — contrat API figé (corps, codes erreurs, gating 404) + garde `@` usernames]
- [Source: `_bmad-output/implementation-artifacts/17-4a-db-foundation.md:142` — dette ECH2-2 (PUT remplacement, owner 17-4d UI)]
- [Source: frontend/src/routes/login/+page.svelte:35-60,95-145 — appel API, erreurs, zone alert, point d'insertion lien]
- [Source: frontend/src/routes/(app)/+layout.ts:9-46 ; routes/setup/+layout.ts:21-25 ; routes/+layout.ts:15-19 — guards]
- [Source: frontend/src/routes/+layout.svelte:54-66 + lib/shared/utils/api-health.svelte.ts:55-73 — 2 parseurs /health à étendre]
- [Source: frontend/src/lib/shared/utils/app-version.svelte.ts:17-31 — pattern store à calquer (DD-1)]
- [Source: frontend/src/lib/shared/utils/api-client.ts:208-240,268,400-418 — parseur erreurs, credentials, interceptor 401]
- [Source: frontend/src/lib/features/setup/{SetupForm.svelte:15-46,setup.api.ts:32-57,SetupForm.test.ts} — pattern form/api/test]
- [Source: frontend/src/routes/(app)/users/+page.svelte:23-50,111-115,146-149,351-433 — states + POST/PUT + dialogues]
- [Source: frontend/src/lib/shared/types/user.ts:5-13 — UserResponse sans email (à étendre)]
- [Source: frontend/src/lib/features/bank-import/bank-import.api.test.ts:1-133 — pattern vitest mock fetch]
- [Source: frontend/scripts/lint-i18n-ownership.js — règles ownership clés]
- [Source: frontend/src/lib/components/invoices/ContactPicker.svelte:19-27 — $props.id() HTTP-LAN safe]
- [Source: crates/kesh-api/src/routes/health.rs:24-53 — shape /health avec forgotPasswordEnabled]
- [Source: CLAUDE.md §Test Locally First (frontend), §Review Iteration Rule ; memory `feedback_no_secure_context_apis_http_lan`]

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (dev-story single-pass, 2026-06-11).

### Debug Log References

### Completion Notes List

- **DD-3 résolu sans contournement** : `request<T>` (`api-client.ts:429-433`) tolère nativement un `200` sans corps JSON (`res.json()` échoue → `undefined`) — `apiClient.post<void>` suffit pour le forgot-password, testé (`auth-recovery.api.test.ts` cas « corps vide »).
- **Pattern SetupForm suivi** : formulaires dans `lib/features/auth-recovery/` (ownership i18n `auth-recovery-*` scanné par le linter), wrappers minces dans `routes/`. Les warnings svelte-check `state_referenced_locally` sur l'init de `invalidLink` depuis la prop `token` ont été levés via `$derived(missing || rejectedToken)`.
- **Idiome par fichier respecté** : pages recovery + SetupForm = `i18nMsg(clé, fallback FR)` ; page login et dialogues users = français hardcodé (ces fichiers n'utilisent pas i18nMsg — pré-existant, hors-scope d'harmoniser). Clés FTL ×4 ajoutées : bloc `auth-recovery-*` (21 clés) + `setup-email-*` (3 clés).
- **ECH2-2 fermée** : `UserResponse.email` typé frontend, `openEdit` pré-remplit `editEmail`, le PUT envoie **toujours** `email` (valeur éditée / inchangée / `null` si vidé délibérément — hint « Vider le champ supprime l'email » dans le dialogue).
- **MIN_PASSWORD 12 hardcodé** dans ResetPasswordForm = même limitation documentée que SetupForm (ECH1-4 v011-5, deferred-work) ; le backend re-valide (400 VALIDATION_ERROR affiché).
- Quality gate frontend (T-D7) : `npm run check` 0 erreur ; `lint-i18n-ownership` PASS ; vitest 36 suites / 304 tests verts (dont 6 nouveaux auth-recovery.api + 2 nouveaux SetupForm email + 1 assertion mise à jour signature 3-args) ; `npm run build` OK ; `cargo test -p kesh-i18n --all-targets` 21/21 (parité clés ×4 locales).

### File List

**Nouveaux fichiers :**
- frontend/src/lib/features/auth-recovery/auth-recovery.types.ts — types contrat 17-4c
- frontend/src/lib/features/auth-recovery/auth-recovery.api.ts — requestPasswordReset + resetPassword
- frontend/src/lib/features/auth-recovery/auth-recovery.api.test.ts — 6 tests vitest (URL, camelCase, corps vide, erreurs typées)
- frontend/src/lib/features/auth-recovery/ForgotPasswordForm.svelte — formulaire AC17 (anti-énum, $props.id())
- frontend/src/lib/features/auth-recovery/ResetPasswordForm.svelte — formulaire AC18 (états succès/lien-invalide/erreurs)
- frontend/src/lib/shared/utils/feature-flags.svelte.ts — store DD-1 (défaut false)
- frontend/src/routes/forgot-password/+page.svelte — wrapper public DD-2
- frontend/src/routes/reset-password/+page.svelte — wrapper public, lit ?token=

**Modifiés :**
- frontend/src/routes/+layout.svelte — parse forgotPasswordEnabled au boot /health
- frontend/src/lib/shared/utils/api-health.svelte.ts — idem dans pollHealth
- frontend/src/routes/login/+page.svelte — lien conditionnel « Mot de passe oublié ? » (AC20)
- frontend/src/lib/shared/types/user.ts — UserResponse.email: string | null (doc ECH2-2)
- frontend/src/lib/features/setup/setup.api.ts — setupAdmin(username, password, email?)
- frontend/src/lib/features/setup/SetupForm.svelte — champ email + validation DD-5 (AC21)
- frontend/src/lib/features/setup/SetupForm.test.ts — +2 tests email, assertion 3-args
- frontend/src/routes/(app)/users/+page.svelte — createEmail + editEmail (PUT envoie toujours email, ECH2-2)
- crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl — +24 clés ×4 (auth-recovery-* + setup-email-*)

## Change Log

### Dev-story (Fable 5, 2026-06-11)

- T-D1..T-D7 implémentés en single-pass, AC17-22 + AC30/31 satisfaits. 8 nouveaux fichiers, 9 modifiés.
- Vérification ground-truth en ouverture : DD-3 (corps vide) résolu par lecture d'api-client (pas de code de contournement nécessaire) ; `LinkIcon` (alias lucide suffixé) et `Button href` validés avant usage.
- Aucun changement backend (contrat 17-4c intact). `ssr=false` global → pages publiques CSR pures, pas de guard ajouté (DD-2 documenté dans les wrappers).
- Quality gate complet vert (cf. Completion Notes). Statut `in-progress → review`. Prochaine étape : `bmad-code-review 17-4d` (LLM différent du dev — dev = Fable → Pass 1 Sonnet ou Opus).
