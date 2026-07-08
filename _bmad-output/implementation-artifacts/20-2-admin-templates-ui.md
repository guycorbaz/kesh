# Story 20.2: Section Admin « Modèles d'e-mail » (frontend)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

En tant qu'administrateur d'une PME utilisant Kesh,
je veux une page `Paramètres → Modèles d'e-mail` (Admin-only) où je peux, par type d'e-mail et par langue (FR/DE/IT/EN), éditer l'objet et le corps du message, voir les variables autorisées, et restaurer le défaut d'un clic,
afin de personnaliser le contenu des e-mails envoyés à mes clients sans dépendre d'un déploiement de code — tout en gardant un fallback zéro-config qui fonctionne si je n'ouvre jamais cette page.

Cette story consomme l'API livrée par la **Story 20-1** (socle backend `email_templates`, statut `done`). Elle ne touche **aucun** code backend : uniquement le frontend SvelteKit (nouvelle feature `email-templates`, nouvelle route settings, entrées de navigation, tests). Le seul type v1 est `invoice_send`.

## Acceptance Criteria

**Module feature & API**

1. Nouveau dossier feature `frontend/src/lib/features/email-templates/` avec :
   - `email-templates.types.ts` : `EmailTemplateResponse` = `{ templateType: string; language: 'FR'|'DE'|'IT'|'EN'; subject: string; body: string; version: number | null; isDefault: boolean; allowedVariables: string[] }` + `UpdateEmailTemplatePayload = { subject: string; body: string; expectedVersion?: number | null }`.
   - `email-templates.api.ts` (calqué `vat-rates.api.ts`) : `listEmailTemplates()` → `GET /api/v1/admin/email-templates`, `getEmailTemplate(type, lang)` → `GET .../{type}/{LANG}`, `updateEmailTemplate(type, lang, payload)` → `PUT .../{type}/{LANG}`, `restoreEmailTemplateDefault(type, lang)` → `DELETE .../{type}/{LANG}` (retour `void`, 204).
   - `index.ts` ré-exportant l'API publique (pattern `vat-rates/index.ts`).
2. **Langue en MAJUSCULES dans l'URL** : toute construction d'URL passe `language.toUpperCase()` (le backend `Language::from_str` est case-sensitive, `FR` pas `fr` — sinon 400). Les codes langue manipulés en frontend sont déjà en majuscules (`'FR'|'DE'|'IT'|'EN'`).

**Route & page**

3. Nouvelle route `frontend/src/routes/(app)/settings/email-templates/+page.svelte`, Admin-only (guard client `isAdmin = $derived(authState.currentUser?.role === 'Admin')` + rendu conditionnel « Accès réservé aux administrateurs » ; le backend reste la source de vérité via 403). Pattern de page calqué `settings/invoicing/+page.svelte` (chargement `onMount`, runes `$state`, pas de `+page.ts`).
4. Chargement : `onMount` appelle `listEmailTemplates()`, stocke les 4 (v1 : 1 type × 4 langues) entrées. États : `loading`, `loadError`, `submitting`, plus les données. En cas d'échec de chargement → message d'erreur inline (pas de crash).

**Éditeur multilingue**

5. Pour le type `invoice_send` (seul en v1), un éditeur avec **onglets de langue FR/DE/IT/EN** (pattern `role="tablist"`/`role="tab"` fait main, cf. `invoices/due-dates/+page.svelte` — **pas** de composant Tabs, aucun n'existe dans le projet). Un `$state` `activeLang` pilote l'onglet actif. Chaque onglet a un `data-testid` (`email-template-lang-tab-FR`…) pour les tests E2E.
6. Sous l'onglet actif : champ **objet** (`<input>`) + champ **corps** (`<textarea>`), pré-remplis avec `subject`/`body` de l'entrée effective de cette langue. Couples `label`/`for` avec IDs stables via **`$props.id()`** (rune Svelte — **jamais** `crypto.randomUUID`, indisponible en HTTP LAN, cf. Dev Notes). `data-testid` sur les deux champs (`email-template-subject`, `email-template-body`).
7. **Panneau d'aide des variables** : liste des `allowedVariables` du type (rendues comme `{salutation}`, `{contactName}`, etc.), affichée à côté/sous l'éditeur, pour que l'utilisateur sache quels tokens il peut insérer. Statique (lecture seule) — pas besoin de bouton copier en v1 (si ajouté, utiliser `copyToClipboard` de `$lib/shared/utils/clipboard.ts`, jamais `navigator.clipboard` direct).
8. **Indicateur défaut vs personnalisé** : afficher visuellement si l'entrée de la langue courante est `isDefault` (ex. badge « Défaut » / « Personnalisé »).

**Enregistrement (verrou optimiste + validation)**

9. Bouton **« Enregistrer »** (par langue) appelle `updateEmailTemplate(type, lang, { subject, body, expectedVersion })` où `expectedVersion` = la `version` de l'entrée courante (`null` si `isDefault` → création). Sur succès : `notifySuccess`, ré-hydrater l'entrée locale avec la réponse (nouvelle `version`, `isDefault=false`).
10. **Conflit 409** (`err.status === 409`) : `notifyError` + recharger l'entrée de cette langue via `getEmailTemplate(type, lang)` et ré-hydrater les champs (pattern `invoicing` `OPTIMISTIC_LOCK_CONFLICT`). L'utilisateur voit l'état frais et peut ré-appliquer.
11. **Validation 422** (`err.status === 422 && err.code === 'EMAIL_TEMPLATE_UNKNOWN_VARIABLES'`) : lire `err.details?.unknownVariables` (array de strings) et afficher un message d'erreur listant les variables inconnues (idéalement en les surlignant / les nommant), sans quitter l'éditeur. Ne PAS traiter ce cas comme une erreur générique.
12. **Erreur subject/body vide** (`err.status === 400`) : afficher le message d'erreur retourné (`err.message`). Optionnellement, désactiver le bouton Enregistrer si subject ou body est vide côté client (garde-fou UX, la validation backend reste autoritaire).

**Restaurer le défaut**

13. Bouton **« Restaurer le défaut »** (par langue, actif seulement si l'entrée courante n'est PAS déjà `isDefault`) ouvre une **modale de confirmation** (primitives `$lib/components/ui/dialog`, gabarit `settings/fiscal-years/+page.svelte` — action destructive/irréversible). Confirmation → `restoreEmailTemplateDefault(type, lang)` → sur 204 : `notifySuccess`, recharger l'entrée (redevient `isDefault=true`, champs repeuplés avec le texte par défaut). `data-testid` sur le bouton déclencheur + le bouton de confirmation.

**Navigation**

14. Entrée de menu dans la sidebar `frontend/src/routes/(app)/+layout.svelte`, groupe `administration.adminOnly` (visible Admin uniquement) : lien vers `/settings/email-templates` (le `data-testid` `nav-link-settings-email-templates` est auto-généré par `navTestid(href)`).
15. Carte « Modèles d'e-mail » sur la page index `frontend/src/routes/(app)/settings/+page.svelte` (pattern `<section>` calqué sur la carte « Taux de TVA »), avec bouton « Gérer » → `/settings/email-templates` + `data-testid`.

**i18n d'interface**

16. Tous les libellés d'UI de la page (titres, labels, boutons, messages) passent par `i18nMsg('email-templates-<clé>', '<fallback FR>')` (module `$lib/shared/utils/i18n.svelte`). Namespace de clé propre à la feature (`email-templates-*`) pour respecter le lint d'ownership (`npm run lint-i18n-ownership`) — les clés `email-templates-*` doivent rester utilisées dans la feature `email-templates` (ou la route), pas ailleurs. **Ne pas confondre** ces clés d'UI (fallback FR immédiat) avec le contenu métier multilingue des templates (subject/body, données de l'API 20-1).

**Invariant zéro-config**

17. Test E2E vérifiant qu'un Admin ouvrant la page sur une company neuve voit les 4 langues toutes marquées « Défaut » avec le texte par défaut non vide — jamais de page vide ni d'erreur (miroir frontend de l'AC #16 de 20-1).

## Tasks / Subtasks

- [ ] **T1 — Module feature `email-templates`** (AC: #1, #2)
  - [ ] T1.1 `frontend/src/lib/features/email-templates/email-templates.types.ts`
  - [ ] T1.2 `email-templates.api.ts` (4 fonctions, URL langue `.toUpperCase()`)
  - [ ] T1.3 `index.ts` (ré-exports)
  - [ ] T1.4 Test unitaire `email-templates.api.test.ts` (mock `apiClient`, vérifie URLs majuscules + méthodes ; gabarit `rules.api.test.ts`/`admin-restore.api.test.ts`)

- [ ] **T2 — Page + éditeur multilingue** (AC: #3, #4, #5, #6, #7, #8)
  - [ ] T2.1 Route `settings/email-templates/+page.svelte` (guard Admin, `onMount` load, runes `$state`)
  - [ ] T2.2 Onglets langue FR/DE/IT/EN (`role="tablist"`, `data-testid`)
  - [ ] T2.3 Champs objet/corps (`$props.id()` pour label/for, `data-testid`)
  - [ ] T2.4 Panneau variables autorisées + badge Défaut/Personnalisé

- [ ] **T3 — Enregistrement (verrou + validation)** (AC: #9, #10, #11, #12)
  - [ ] T3.1 Bouton Enregistrer → `updateEmailTemplate` avec `expectedVersion`
  - [ ] T3.2 Gestion 409 (recharge + ré-hydrate), 422 (liste `unknownVariables`), 400 (message)

- [ ] **T4 — Restaurer le défaut** (AC: #13)
  - [ ] T4.1 Bouton + modale confirmation (Dialog, gabarit fiscal-years)
  - [ ] T4.2 `restoreEmailTemplateDefault` → recharge l'entrée

- [ ] **T5 — Navigation + i18n** (AC: #14, #15, #16)
  - [ ] T5.1 Entrée sidebar `+layout.svelte` (adminOnly)
  - [ ] T5.2 Carte `settings/+page.svelte`
  - [ ] T5.3 Toutes les clés `email-templates-*` avec fallback FR ; `npm run lint-i18n-ownership` vert

- [ ] **T6 — Tests E2E** (AC: #17, + parcours principaux)
  - [ ] T6.1 `frontend/tests/e2e/email-templates.spec.ts` (gabarit `fiscal-years.spec.ts`/`vat-rates.spec.ts`) : login admin → nav → page charge 4 langues défaut (AC #17), éditer+enregistrer une langue (persistance), conflit/validation si faisable, restaurer défaut via modale
  - [ ] T6.2 Vérifier la non-régression : rôle non-Admin ne voit pas le lien / page affiche « accès réservé »

- [ ] **T7 — Test Locally First & commit**
  - [ ] T7.1 Depuis `frontend/` : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, `npm run test:e2e` (story frontend → checks Frontend + E2E, cf. CLAUDE.md)
  - [ ] T7.2 Commit(s) sur `story/20-1-envoi-factures-email` (branche epic-20 en cours)

## Dev Notes

### Frontières strictes de scope

- **Frontend uniquement** — NE PAS toucher au backend (`crates/`). L'API 20-1 est figée et testée.
- **Type `invoice_send` seul** (v1). L'UI doit être structurée pour accueillir d'autres types plus tard (rappel, récurrent), mais n'en affiche qu'un. Si un seul type, un sélecteur de type n'est pas obligatoire — mais garder le code générique (itérer sur les types retournés par `listEmailTemplates`, ne pas hardcoder `invoice_send` dans le rendu au point de bloquer l'ajout futur).
- **Pas de config SMTP ici** — le transport (env vars) est hors scope (manuel admin, Story 20-4). Cette page ne configure QUE le contenu des templates.
- **Pas d'envoi de facture** — le bouton « Envoyer par e-mail » sur la fiche facture est Story 20-3b.

### Contrainte critique — HTTP LAN / secure-context (bug #145)

Kesh se déploie en **HTTP LAN** (NAS Synology, pas de HTTPS). Les API navigateur secure-context-only (`crypto.randomUUID`, `crypto.subtle`, `navigator.clipboard`) sont `undefined` en prod et **crashent la page** (page blanche) — invisible en dev `localhost`. **Interdits en runtime `frontend/src/`.**

- **IDs de formulaire** (label/for de l'éditeur) → `const uid = $props.id();` puis `id="{uid}-subject"` / `for="{uid}-subject"`. Preuve du pattern exact : `settings/invoicing/+page.svelte` (commentaire + usage). **Jamais** `crypto.randomUUID`.
- **Clipboard** (si bouton copier variable ajouté) → `copyToClipboard()` de `$lib/shared/utils/clipboard.ts` (fallback `execCommand`), jamais `navigator.clipboard` direct.

### Patterns à réutiliser (ne pas réinventer)

- **Page settings objet-unique + verrou optimiste** : `frontend/src/routes/(app)/settings/invoicing/+page.svelte` — modèle direct (onMount, `$state`, guard `isAdmin`, gestion `OPTIMISTIC_LOCK_CONFLICT` avec recharge). Adapter : ici l'entité est par (type, langue), le conflit est un `409` (tester `err.status === 409`), la validation un `422` code `EMAIL_TEMPLATE_UNKNOWN_VARIABLES`.
- **CRUD settings + i18n + toast** : `frontend/src/routes/(app)/settings/vat-rates/+page.svelte`.
- **Module feature API** : `frontend/src/lib/features/vat-rates/{vat-rates.api.ts,vat-rates.types.ts,index.ts}` — structure exacte à cloner.
- **Wrapper HTTP** : `apiClient` de `$lib/shared/utils/api-client` (`get`/`put`/`delete<void>`). Erreurs = **exceptions** `ApiError { code, message, details?, status }` ; `isApiError(err)` type-guard ; `err.status` = HTTP, `err.code` = code métier, `err.details` = objet libre (ici `unknownVariables`). `DELETE` 204 → `apiClient.delete<void>(url)` retourne `undefined`.
- **Onglets** (aucun composant Tabs) : pattern fait main `role="tablist"`/`role="tab"` de `invoices/due-dates/+page.svelte` (~L322).
- **Modale confirmation destructive** (aucun `ConfirmDialog` générique) : primitives `$lib/components/ui/dialog` (`Dialog.Root/Content/Header/Title/Description/Footer/Close`), gabarit `settings/fiscal-years/+page.svelte` (~L394), état `$state(false)` + `bind:open`.
- **Notifications** : `notifySuccess`/`notifyError` de `$lib/shared/utils/notify`.
- **i18n UI** : `i18nMsg('key', 'fallback FR', args?)` de `$lib/shared/utils/i18n.svelte` (traductions DE/IT/EN de l'UI viennent du backend `/api/v1/i18n/messages` — pas de fichier de messages frontend à éditer ; le fallback FR suffit pour livrer, les autres langues d'UI sont un ajout backend séparé si souhaité).

### Sémantique verrou optimiste (rappel 20-1)

`version` d'une entrée est `null` quand `isDefault: true` (aucune ligne en base). Au premier enregistrement d'une langue en défaut, envoyer `expectedVersion: null` (ou champ absent) → le backend fait un `INSERT` (version 1). Ensuite `version` est un entier, envoyé à chaque `PUT`. Un `409` signifie qu'un autre onglet/admin a modifié entre-temps → recharger.

### i18n ownership linter

`npm run lint-i18n-ownership` impose que les clés spécifiques à une feature ne soient utilisées que dans le dossier de cette feature. Utiliser le namespace `email-templates-*` pour toutes les nouvelles clés d'UI, et les garder dans la route `settings/email-templates` / la feature `email-templates`. Les namespaces globaux autorisés partout : `error-*`, `tooltip-*`, `common-*`, `mode-*`, `shortcut-*`, `demo-*` (cf. `frontend/scripts/lint-i18n-ownership.js`). Ne pas ajouter aux `KNOWN_VIOLATIONS`.

### Project Structure Notes

- **Nouveaux fichiers** : `frontend/src/lib/features/email-templates/{email-templates.types.ts,email-templates.api.ts,index.ts,email-templates.api.test.ts}`, `frontend/src/routes/(app)/settings/email-templates/+page.svelte`, `frontend/tests/e2e/email-templates.spec.ts`.
- **Fichiers modifiés** : `frontend/src/routes/(app)/+layout.svelte` (entrée sidebar adminOnly), `frontend/src/routes/(app)/settings/+page.svelte` (carte).
- Aucun conflit avec la structure unifiée (features co-localisées `src/lib/features/<feature>/`, routes SvelteKit `routes/(app)/settings/<page>/`).

### Testing standards summary

- **Unit (Vitest)** : `email-templates.api.test.ts` co-localisé, mock `apiClient`, vérifie que `getEmailTemplate('invoice_send', 'FR')` construit bien `/api/v1/admin/email-templates/invoice_send/FR` (majuscules) et que `delete` cible la bonne URL. Gabarits : `reconciliation.api.test.ts`, `rules.api.test.ts`, `admin-restore.api.test.ts`.
- **E2E (Playwright)** : `email-templates.spec.ts`, gabarit `fiscal-years.spec.ts` + `vat-rates.spec.ts`. Seed `seedTestState('with-company')` en `beforeEach`, `clearAuthStorage(page)` en `afterEach`, login `admin`/`admin123`, sélection par `getByTestId(...)`. Couvre AC #17 (4 langues défaut à l'ouverture), édition+enregistrement, restauration via modale. Pré-requis E2E : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` (Ubuntu 26.04+, cf. reference mémoire), MariaDB + seed CI.
- **Test Locally First (CLAUDE.md)** : story frontend → checks `Frontend (Svelte)` (`npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`) **+** E2E (`npm run test:e2e`, critique car nouvelle route + interactions). Pas de check backend (aucun `.rs` touché).

### References

- [Source: `_bmad-output/planning-artifacts/epic-20-envoi-factures-email.md`] — décision #10 (section Admin), découpage 20-2, invariant zéro-config.
- [Source: `_bmad-output/implementation-artifacts/20-1-templates-email-socle.md`] — contrat API consommé (4 endpoints, DTO `EmailTemplateResponse`, codes 409/422, langue MAJUSCULES, `expectedVersion` Option).
- [Source: `frontend/src/routes/(app)/settings/invoicing/+page.svelte`] — modèle page objet-unique + verrou optimiste + `$props.id()`.
- [Source: `frontend/src/routes/(app)/settings/vat-rates/+page.svelte`] — modèle CRUD settings Admin + i18n + toast + note secure-context.
- [Source: `frontend/src/lib/features/vat-rates/`] — structure module feature (api/types/index).
- [Source: `frontend/src/lib/shared/utils/api-client.ts`] — `apiClient`, `isApiError`, `ApiError{code,message,details,status}`.
- [Source: `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`] — modèle modale confirmation destructive (Dialog).
- [Source: `frontend/src/routes/(app)/invoices/due-dates/+page.svelte`] — pattern onglets fait main.
- [Source: `frontend/src/routes/(app)/+layout.svelte`] — sidebar `navGroups`/`adminOnly`.
- [Source: `frontend/tests/e2e/fiscal-years.spec.ts`] — gabarit E2E page settings admin.
- [Source: `frontend/scripts/lint-i18n-ownership.js`] — règle d'ownership des clés i18n.
- [Source: mémoire `feedback_no_secure_context_apis_http_lan`] — interdiction `crypto.randomUUID` etc., `$props.id()`.
- [Source: CLAUDE.md §Test Locally First] — checks Frontend + E2E.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
