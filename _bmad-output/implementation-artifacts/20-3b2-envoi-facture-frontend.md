# Story 20.3b2: Envoi de facture par e-mail — frontend

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

En tant que comptable d'une PME,
je veux un bouton « Envoyer par e-mail » sur la fiche facture qui ouvre une modale pré-remplie (objet/corps dans la langue du client, destinataire verrouillé),
afin d'envoyer la QR-facture PDF à mon client en deux clics, sans quitter Kesh.

**Périmètre de cette sous-story : frontend uniquement** (+ clés FTL dans `crates/kesh-i18n` — seul contact avec le repo Rust, aucun code backend). Le backend est livré par la **20-3b1** (done, commits `e308e7e1`/`711396de`). Les manuels + E2E Playwright du flux complet sont en **20-4**.

### Note de découpage

Split 20-3b → 20-3b1 (backend) + 20-3b2 (frontend) acté à la spec 20-3b1 (règle de splitting préventif CLAUDE.md, >5 modules). Contrat d'API consommé ici = exactement ce que la 20-3b1 a livré, y compris les 2 erreurs ajoutées par sa code-review (409 `EMAIL_SENT_INVOICE_GONE`, 400 `CONTACT_ARCHIVED`).

## Acceptance Criteria

**Feature flag `smtpConfigured` (décision #3 epic)**

1. `feature-flags.svelte.ts` expose `smtpConfigured` — pattern **exact** `forgotPasswordEnabled` (feature-flags.svelte.ts:26-40) : `_smtpConfigured = $state<boolean>(false)` (défaut `false` = anti-faux-affordance), getter + `setSmtpConfigured(value: unknown)` no-op si non-booléen.
2. Les **2 call-sites `/health`** peuplent le flag : `+layout.svelte` (onMount, :35-75 — ajouter `smtpConfigured?: unknown` au type inline :60-64 + appel setter à côté de :68) et `api-health.svelte.ts` (`pollHealth`, :56-83 — idem type inline + setter :72). `/health` reste appelé en `fetch` natif (jamais `apiClient` — raison documentée api-health.svelte.ts:38-44, à préserver).

**Types & wrappers API**

3. `invoices.types.ts` : `InvoiceResponse` + `emailedAt: string | null` + `emailedTo: string | null` (:26-46, camelCase — absents aujourd'hui, vérifié). Nouveaux types `EmailPreviewResponse { to: string | null; language: EmailLanguage; subject: string; body: string }` et `SendInvoiceEmailRequest { subject: string; body: string }` avec `type EmailLanguage = 'FR' | 'DE' | 'IT' | 'EN'`.
4. `invoices.api.ts` : `getInvoiceEmailPreview(id: number): Promise<EmailPreviewResponse>` → `apiClient.get('/api/v1/invoices/${id}/email-preview')` ; `sendInvoiceEmail(id: number, req: SendInvoiceEmailRequest): Promise<InvoiceResponse>` → `apiClient.post('.../send-email', req)`. Named exports directs (pas de barrel `index.ts` dans `invoices/` — convention du dossier).
5. `contacts.types.ts` : `ContactResponse` + `language: EmailLanguage | null` + `salutation: Salutation` (`type Salutation = 'Monsieur' | 'Madame' | 'Neutre'`) ; `CreateContactRequest`/`UpdateContactRequest` + `language?: EmailLanguage | null` + `salutation?: Salutation` (absents aujourd'hui, vérifié :27-79 ; miroir serde backend contacts.rs).
6. `settings.types.ts` : `CompanyJson` + `email: string | null` + `version: number` (:1-9 — absents aujourd'hui ; le backend les expose depuis 20-3b1). `settings.api.ts` : `updateCompanyEmail(req: { email: string | null; version: number }): Promise<CompanyJson>` → `apiClient.put('/api/v1/companies/current/email', req)`.

**Fiche facture — bouton + modale (décisions #13/#16 epic)**

7. Bouton « Envoyer par e-mail » (ou « Renvoyer par e-mail » si `invoice.emailedAt` non nul) dans le **bloc `validated`** de la barre d'actions (`invoices/[id]/+page.svelte` :341-384, à côté du bouton PDF), visible si `canManage`. Si `!featureFlags.smtpConfigured` → bouton **grisé** (`disabled`) + **tooltip** expliquant « L'envoi d'e-mails n'est pas configuré (variables KESH_SMTP_*) — voir le manuel administrateur ». ⚠️ Pièges : (a) un `<button disabled>` ne déclenche pas les événements hover — envelopper le trigger dans un `<span>` ; (b) construire directement sur les **primitives** bits-ui `$lib/components/ui/tooltip` (`Tooltip.Provider/Root/Trigger/Content`) — `AccountingTooltip.svelte` n'est PAS réutilisable ici (pattern glossaire à 2 clés `tooltip-{term}-natural/-technical`, incompatible avec un message libre à 1 clé ; il sert seulement d'exemple d'usage des primitives). Aucun combo « grisé + tooltip » n'existe encore dans l'app — celui-ci fait référence.
8. Clic → le parent appelle `getInvoiceEmailPreview(id)` (spinner/disabled pendant le fetch) puis ouvre `SendEmailDialog` pré-remplie. Erreurs preview : 400 `CONTACT_ARCHIVED` → toast (`notifyError`) sans ouvrir la modale ; autres `ApiError` → toast générique. Un `to: null` dans la preview n'est PAS une erreur (voir AC#10).
9. Nouveau composant `src/lib/features/invoices/SendEmailDialog.svelte` — modèle **`MarkPaidDialog.svelte`** (106 l.) : même convention de responsabilité (le composant **n'appelle jamais l'API** — il émet `onConfirm(subject, body)`, le parent gère l'appel et les erreurs), mêmes props (type `Props` :14-22 + destructure :24-31 ; ici `open`, `onOpenChange`, `submitting`, `errorMsg`, + `preview: EmailPreviewResponse`), même pattern Dialog partagé (`Dialog.Root/Content/Header/Title/Footer`, boutons du Footer :97-104), même bloc erreur (`clientError`/`errorMsg`, classes destructive), mêmes gardes double-submit (Annuler `disabled={submitting}`, Confirmer `disabled={submitting || !!clientError}`). Reset des champs via `$effect` quand `open` passe à true (:40-42 chez le modèle ; ici re-hydrater depuis `preview`). IDs DOM via `$props.id()` (piège #145 — PAS de `crypto.randomUUID` ; **nota** : MarkPaidDialog utilise un id en dur `mark-paid-date` :78-82, le précédent `$props.id()` à suivre est `settings/email-templates/+page.svelte:26` `const uid = $props.id();`).
10. Contenu de la modale : **destinataire READ-ONLY** (texte affiché, jamais un input — décision #13 : verrouillé `contacts.email`) ; si `preview.to === null` → message « Le contact n'a pas d'adresse e-mail — renseignez-la sur la fiche contact » + bouton Envoyer `disabled` ; **objet** (Input) et **corps** (`<textarea>`, ~10 lignes, mêmes classes que les inputs) éditables, pré-remplis depuis `preview.subject`/`preview.body` ; `clientError` si objet ou corps vide après trim.
11. Gestion des erreurs du **send** (dans le parent, pattern `err.code` via `isApiError` — jamais `err.status` seul) :
    - `INVOICE_EMAIL_EMPTY_CONTENT` (422), `SMTP_SEND_FAILED` (500) → erreur **inline** dans la modale (elle reste ouverte, message backend ; pour 500 le message précise déjà l'échec — la facture n'est PAS marquée, on peut réessayer).
    - `CONTACT_EMAIL_MISSING` / `CONTACT_ARCHIVED` (400) → toast + fermer la modale (l'état contact a changé sous nos pieds).
    - `SMTP_NOT_CONFIGURED` (412 — flag périmé côté client) → toast + fermer + `featureFlags.setSmtpConfigured(false)` (le bouton se grise).
    - `RATE_LIMITED` (429) → toast avec le message backend (`err.message` — **nota** : le message FTL `error-rate-limited` = « Trop de tentatives », SANS le délai ; le header `Retry-After` n'est pas exposé par `ApiError`/`parseErrorResponse` et cette story est frontend-only → ne PAS promettre le délai à l'utilisateur ; même comportement que les 429 existants de SetupForm/ForgotPasswordForm).
    - `EMAIL_SENT_INVOICE_GONE` (409) → toast **warning** avec le message backend (« l'e-mail a bien été envoyé… ne renvoyez pas ») + fermer + retour à la liste des factures (`goto('/invoices')` — la facture n'existe plus).
    - Erreurs PDF héritées (`INVOICE_NOT_VALIDATED`, etc.) → inline générique (message backend).
    - Succès → `invoice = <réponse>` (contient `emailedAt`/`emailedTo` à jour), toast succès, fermer.
12. Métadonnées fiche facture : bloc « Envoyée le » dans la grille (:410-437), pattern **exact** « Payée le » (:431-436) : `{#if invoice.emailedAt}` → label `text-text-muted` + `{invoice.emailedAt.slice(0, 10)}` + destinataire `{invoice.emailedTo}` sur la 2e ligne (« Envoyée le 11.07.2026 à pia@example.ch » — garder le format date ISO tronqué comme paidAt).

**Fiche contact — langue & civilité (décisions #11/#12 epic)**

13. Formulaire contact (`contacts/+page.svelte`, Dialog :580-722) : 2 nouveaux `<select>` natifs (pattern exact du select type :599-612) —
    - **« Langue de correspondance »** : options « Héritée (langue de l'instance) » (valeur `''` → envoyée `null`) + FR/DE/IT/EN. Visible pour les deux types de contact.
    - **« Civilité »** : options Neutre/Monsieur/Madame. Visible **uniquement si `formContactType === 'Personne'`** (bloc :615-631 — une Entreprise reçoit toujours la formule neutre côté backend, ne pas afficher un champ sans effet). À la bascule Personne→Entreprise, la valeur repart à `Neutre` au submit.
    - États `$state` (:57-76), reset dans `openCreate()` (:206-225 — `''`/`Neutre`), hydratation dans `openEdit(c)` (:227-246 — `c.language ?? ''` / `c.salutation`), payload dans `submitForm()` (:271-327 — `language: formLanguage === '' ? null : formLanguage`, `salutation: formContactType === 'Personne' ? formSalutation : 'Neutre'`).
    - IDs en dur `form-language`/`form-salutation` (cohérent avec les ids existants du formulaire `form-name`/`form-type` — ce formulaire n'utilise pas `$props.id()`, ne pas mélanger les deux styles dans un même formulaire).

**Réglages — e-mail de la société (Reply-To, décision #2 epic)**

14. Section « Organisation » de `settings/+page.svelte` (:41-67, `<dl>` read-only) : ligne « E-mail (adresse de réponse) » affichant `company.email ?? '—'`, avec **édition inline Admin-only** (pattern `isAdmin = $derived(authState.currentUser?.role === 'Admin')`, modèle email-templates/+page.svelte:28) : bouton « Modifier » → input + Enregistrer/Annuler. Aide sous le champ : « Adresse de réponse (Reply-To) des factures envoyées par e-mail. Vide = pas d'adresse de réponse. »
15. Validation client `isPlausibleEmail` (`src/lib/shared/utils/email.ts`) si non vide ; champ vidé → `email: null` (effacement). Erreurs : 400 `VALIDATION_ERROR` → inline ; 409 `OPTIMISTIC_LOCK_CONFLICT` → recharger `fetchCompanyCurrent()` + message « Conflit de version — données rechargées » (modèle email-templates :130-143). La `version` envoyée vient de `data.company.version`.

**i18n (×4 locales) & lint**

16. Clés FTL ajoutées aux **4 locales** `crates/kesh-i18n/locales/*/messages.ftl` (servies au front par `/api/v1/i18n/messages` ; fallbacks FR inline dans chaque `i18nMsg`) : bouton/tooltip envoi (`invoice-send-email-button`, `invoice-resend-email-button`, `invoice-send-email-smtp-tooltip`), modale (`invoice-send-email-title`, `invoice-send-email-to-label`, `invoice-send-email-to-missing`, `invoice-send-email-subject-label`, `invoice-send-email-body-label`, `invoice-send-email-confirm`, `invoice-send-email-success`, `invoice-send-email-error-empty`), métadonnée (`invoice-detail-emailed-at-label`), fiche contact (`contact-form-language`, `contact-form-language-inherited`, `contact-form-salutation`, `contact-salutation-neutre/-monsieur/-madame`), réglages (`settings-field-company-email`, `settings-company-email-help`, `settings-company-email-invalid`). Les libellés FR/DE/IT/EN suivent le ton des clés existantes.
17. `lint-i18n-ownership` PASS : `SendEmailDialog.svelte` (sous `features/invoices/`) consomme des clés `invoice-*` → **piège #30** (dossier pluriel vs préfixe singulier) : ajouter ses entrées à `KNOWN_VIOLATIONS` (`frontend/scripts/lint-i18n-ownership.js` :22-105, à côté des 6 entrées `MarkPaidDialog.svelte`). Les clés utilisées dans `src/routes/` ne sont pas scannées (FEATURES_PATH :17) — aucun ajout pour la fiche facture/contact/réglages.

**Tests & non-régression**

18. Tests unitaires vitest : nouveau `invoices.api.test.ts` (2 wrappers — pattern `vi.mock('$lib/shared/utils/api-client')`, modèle `reconciliation/rules/rules.api.test.ts` :1-60) ; `settings.api.test.ts` étendu ou créé (`updateCompanyEmail` : PUT bon path + payload) ; test du setter `setSmtpConfigured` (no-op non-booléen) si un fichier de test feature-flags existe, sinon couvert via le test api-health existant étendu. Pas de test unitaire de composant Svelte (convention : couverture UI via E2E — ici différée 20-4).
19. Gate frontend **Test Locally First** : `npm run check` (0 erreur), `npm run lint-i18n-ownership` (PASS), `npm run test:unit` (0 échec), `npm run build` (OK) + **E2E Playwright de non-régression ciblés** : `contacts.spec.ts` (le formulaire gagne 2 champs — les sélecteurs `#form-*` existants ne doivent pas casser), `invoices.spec.ts`, `email-templates.spec.ts`, contre backend sur DB `kesh_e2e` (recette : binaire kesh-api, `DATABASE_URL=…/kesh_e2e`, migre forward au boot ; `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`). **Aucun nouveau spec Playwright** (E2E du flux d'envoi = 20-4).

## Tasks / Subtasks

- [x] **T1 — Feature flag `smtpConfigured`** (AC: #1, #2)
  - [x] T1.1 Store `feature-flags.svelte.ts` (getter/setter, défaut false)
  - [x] T1.2 2 call-sites `/health` (`+layout.svelte` + `api-health.svelte.ts` pollHealth)
- [x] **T2 — Types & wrappers API** (AC: #3-#6)
  - [x] T2.1 `invoices.types.ts` (`emailedAt`/`emailedTo`, `EmailPreviewResponse`, `SendInvoiceEmailRequest`) + `invoices.api.ts` (preview + send)
  - [x] T2.2 `contacts.types.ts` (`language`/`salutation` sur Response + requests)
  - [x] T2.3 `settings.types.ts` (`CompanyJson.email`/`.version`) + `settings.api.ts` (`updateCompanyEmail`)
- [x] **T3 — Fiche facture** (AC: #7-#12)
  - [x] T3.1 Bouton Envoyer/Renvoyer + gate flag + tooltip (wrapper span, piège hover-disabled)
  - [x] T3.2 `SendEmailDialog.svelte` (modèle MarkPaidDialog, to read-only, `$props.id()`)
  - [x] T3.3 Parent : fetch preview + matrice d'erreurs send (AC#11) + toast/goto 409
  - [x] T3.4 Bloc « Envoyée le … à … » (grille métadonnées)
- [x] **T4 — Fiche contact : langue + civilité** (AC: #13)
- [x] **T5 — Réglages : e-mail société inline Admin** (AC: #14, #15)
- [x] **T6 — i18n ×4 + lint** (AC: #16, #17)
- [x] **T7 — Tests & gate** (AC: #18, #19)
  - [x] T7.1 Tests unitaires api/wrappers
  - [x] T7.2 `check` + `lint-i18n-ownership` + `test:unit` + `build` + E2E non-régression (contacts/invoices/email-templates, DB `kesh_e2e`)
  - [x] T7.3 Commit sur `story/20-1-envoi-factures-email`

## Dev Notes

### Contrat backend consommé (20-3b1, ground-truth — le code est dans le même repo)

- `GET /api/v1/invoices/{id}/email-preview` (Comptable+) → 200 `{ to: string|null, language: "FR"|"DE"|"IT"|"EN", subject, body }`. Erreurs : 404 (facture/tenant), **400 `CONTACT_ARCHIVED`**, 403 (Consultation), 401.
- `POST /api/v1/invoices/{id}/send-email` (Comptable+) body `{ subject, body }` (camelCase, pas de `to`) → 200 `InvoiceResponse` (avec `emailedAt`/`emailedTo`). Gardes ordonnées : 429 `RATE_LIMITED` (header Retry-After) → 412 `SMTP_NOT_CONFIGURED` → 404 → 400 `CONTACT_EMAIL_MISSING`/`CONTACT_ARCHIVED` → 422 `INVOICE_EMAIL_EMPTY_CONTENT` → erreurs PDF héritées (400 `INVOICE_NOT_VALIDATED`…) → 500 `SMTP_SEND_FAILED` (facture NON marquée) → **409 `EMAIL_SENT_INVOICE_GONE`** (e-mail PARTI, facture supprimée pendant l'envoi — message backend explicite anti-renvoi).
- `GET /health` → `smtpConfigured: bool` dans les 2 branches 200/503 (= config SMTP complète ET mailer construit).
- `PUT /api/v1/companies/current/email` (**Admin-only**) body `{ email: string|null, version: number }` → 200 `CompanyJson` (avec `email`, `version` incrémenté). Erreurs : 400 `VALIDATION_ERROR` (e-mail invalide), 409 `OPTIMISTIC_LOCK_CONFLICT`, 403 (Comptable). `GET /companies/current` renvoie `{ company: CompanyJson, bankAccounts: [...] }` — `company.version` disponible.
- Contacts : `language`/`salutation` acceptés en create/update (`#[serde(default)]` — absents = inchangé/défauts), renvoyés dans `ContactResponse` (`salutation` non-Option, défaut `Neutre`).

### Ground-truth frontend (cartographie 3 agents Explore, 2026-07-11)

**Fiche facture** : `src/routes/(app)/invoices/[id]/+page.svelte` (640 l.) — `canManage` (dérivé rôle Admin/Comptable, début de fichier), bloc `validated` de la barre d'actions **:341-384** (boutons conditionnels `canManage`/`isAdmin` :373-384), bloc `cancelled` :385+, pattern handlers try/catch + flag submitting, erreurs testées par `err.code` (`isApiError`), **double affichage** erreur = inline modale + `notifyError` toast, verrou 409 → reload + fermer. Grille métadonnées **:410-437**, « Payée le » **:431-436**. Helper spécial `notifyMissingFiscalYearOrFallback`.

**MarkPaidDialog** (`features/invoices/MarkPaidDialog.svelte`, 106 l.) : type `Props` :14-22 + destructure :24-31, convention « le composant n'appelle jamais l'API » (en-tête :1-7), reset `$effect` sur `open` :40-42, `clientError` `$derived.by`, boutons Footer :97-104, **id en dur** `mark-paid-date` :78-82 (ne PAS y chercher `$props.id()`). Dialog partagé `$lib/components/ui/dialog` (bits-ui).

**⚠️ La feature s'appelle `invoices`** (`src/lib/features/invoices/` — `invoicing/` existe mais est VIDE). Pas de barrel `index.ts` : imports directs par fichier. Wrappers existants : `getInvoice` :42-44, `markInvoicePaid` :99-101, etc. `InvoiceResponse` :26-47 (montants en **string**, ne jamais convertir en number). PDF téléchargé inline via `apiClient.getBlob`.

**Feature flags** : `src/lib/shared/utils/feature-flags.svelte.ts` — `$state` module-level (OK car app CSR pure, `ssr=false`), pattern complet :26-40. Peuplé par `+layout.svelte:35-75` (fetch natif `/health`, timeout 2 s, type inline :60-64, setter :68) et `api-health.svelte.ts:56-83` (poll 5 s en état dégradé, setter :72). Consommateur existant : login :150-158 (`{#if featureFlags.forgotPasswordEnabled}`).

**Réglages** : hub `settings/+page.svelte` (180 l.), section Organisation :41-67 = `<dl>` read-only (name/orgType/address/ideNumber/instanceLanguage), chargée par `fetchCompanyCurrent()` (:16-24). `settings.api.ts` n'a QUE cette fonction ; `settings.types.ts` `CompanyJson` :1-9 SANS `email` ni `version`. Admin-gating : `isAdmin` dérivé + `{#if !isAdmin}` message (email-templates :28, :197-199).

**Tooltip** : composants bits-ui `$lib/components/ui/tooltip` (Root/Trigger/Content/Provider/Portal) + wrapper métier `AccountingTooltip.svelte` (usage JournalEntryForm :266+). **Aucun combo « bouton disabled + tooltip » n'existe** — à construire ; un `<button disabled>` ne fire pas les events hover → trigger = `<span>` englobant.

**Fiche contact** : PAS de route dédiée — tout dans `contacts/+page.svelte` (765 l.), Dialog create/edit partagé :580-722. États `$state` :57-76, `openCreate` :206-225, `openEdit` :227-246, `submitForm` :271-327 (payload :277-295, verrou `version` :298, catch `OPTIMISTIC_LOCK_CONFLICT` :310 → `conflictOpen`). Select natif type :599-612 (à copier). Bloc conditionnel Personne :615-631. **IDs en dur** (`form-name`, `form-type`…) — ce formulaire n'utilise PAS `$props.id()` ; rester cohérent localement. `contacts.spec.ts` E2E utilise ces ids (`#form-name` :55) — ne pas les renommer.

**API client** : `apiClient.{get,post,put,patch,delete,getBlob}` (`shared/utils/api-client.ts` :521-571), `isApiError` :19-28, `ApiError { code, message, details?, status }`, refresh 401 transparent, `parseErrorResponse` :208-240. Toasts `notify.ts` (`notifySuccess/notifyError/notifyWarning`, svelte-sonner). `isPlausibleEmail` dans `shared/utils/email.ts`.

**i18n** : clés dans `crates/kesh-i18n/locales/*/messages.ftl` (servies par `GET /api/v1/i18n/messages`), `i18nMsg(key, fallbackFR, args?)` (`i18n.svelte.ts` :13-18). Lint `frontend/scripts/lint-i18n-ownership.js` : ne scanne QUE `src/lib/features` (:17), namespaces globaux :16 (`error`, `tooltip`, `common`…), `KNOWN_VIOLATIONS` :22-105 (piège #30 : dossiers pluriels `invoices/`/`contacts/` vs préfixes singuliers `invoice-*`/`contact-*` — 6 entrées MarkPaidDialog :23-28 = précédent exact pour SendEmailDialog). Aucune clé `salutation` existante.

**Tests** : vitest ; pattern mock api = `vi.mock('$lib/shared/utils/api-client')` (modèle `rules.api.test.ts` :1-60) ; il n'existe AUCUN test pour `invoices.api.ts` aujourd'hui (seulement helpers). E2E : `frontend/tests/e2e/` (PAS `frontend/e2e/`, qui est vide), helpers `test-state.ts` (`seedTestState('with-company')`), login redéfini par spec, `email-templates.spec.ts` = modèle récent (20-2).

### Décisions de conception (refinements assumés)

- **E-mail société : édition inline dans la section Organisation** (pas de sous-page `settings/company/`) — un seul champ éditable ne justifie pas une sous-page ; le hub garde son rôle, le gate `isAdmin` s'applique au bouton Modifier (les non-admins voient la valeur en lecture seule comme le reste de la section).
- **Civilité masquée pour les Entreprises** : le backend force la formule neutre pour `Entreprise` quelle que soit la civilité stockée — afficher le select serait un champ sans effet (UX mensongère). `salutation: 'Neutre'` envoyé au submit pour une Entreprise.
- **Preview fetchée par le parent avant l'ouverture** de la modale (pas de fetch dans le Dialog) — cohérent avec la convention MarkPaidDialog « le composant n'appelle jamais l'API », et permet de traiter `CONTACT_ARCHIVED` par un simple toast sans modale fantôme.
- **409 `EMAIL_SENT_INVOICE_GONE` → toast warning + `goto('/invoices')`** : la facture n'existe plus, rester sur sa fiche déclencherait des 404 en cascade ; le message backend (anti-renvoi) est affiché tel quel.
- **412 → `setSmtpConfigured(false)`** : le flag client se resynchronise sans attendre le prochain poll `/health`.

### Frontières de scope

- **AUCUN fichier Rust** hors `crates/kesh-i18n/locales/*/messages.ftl` (clés UI ×4). Aucun manuel (20-4), **aucun nouveau spec Playwright** (20-4 : round-trip MockMailer, gate rôle, destinataire verrouillé, fallback zéro-config).
- Les specs E2E existants doivent rester verts (non-régression AC#19) ; `contacts.spec.ts` est le plus exposé (formulaire modifié).
- Ne pas toucher `MarkPaidDialog`, ni le flux recovery/login, ni la page email-templates (20-2).

### Testing standards summary

- Gate frontend : `npm run check` / `npm run lint-i18n-ownership` / `npm run test:unit` / `npm run build` (CLAUDE.md §Test Locally First).
- E2E non-régression : backend contre DB `kesh_e2e` (migre forward au boot — la DB dev `kesh` est réparée mais `kesh_e2e` reste la cible E2E), `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, `KESH_BACKEND_URL=http://127.0.0.1:8181`.

### References

- [Source: `_bmad-output/planning-artifacts/epic-20-envoi-factures-email.md`] — décisions #2/#3 (Reply-To, gate SMTP + dégradation gracieuse), #11-#13 (langue/civilité/destinataire verrouillé), #16 (renvoi), L20-3 (« envoyée » = remise SMTP).
- [Source: `_bmad-output/implementation-artifacts/20-3b1-envoi-facture-backend.md`] — contrat d'API complet + Senior Developer Review (erreurs 409/400 ajoutées en Pass 1).
- [Source: cartographie 3 agents Explore frontend 2026-07-11 — fiche facture/invoices, feature-flags/health/settings, contacts/i18n] (références précises dans les Dev Notes).
- [Source: `_bmad-output/implementation-artifacts/20-2-admin-templates-ui.md`] — patterns frontend 20-2 (verrou optimiste 409, `$props.id()`, lint-i18n).
- [Source: CLAUDE.md §Test Locally First, §Review Iteration Rule]

## Change Log

- 2026-07-11 — `bmad-dev-story` COMPLETED run unique (Fable 5) : T1-T7, 0 déviation spec. Gate frontend vert (check 0 err, lint-i18n PASS, unit 377/377, build OK, E2E non-régression 15/15+2 skip pré-existants contre `kesh_e2e`). Story → review.
- 2026-07-11 — `validate-create-story` **CONVERGÉ 2 passes** (Sonnet 5 → Haiku 4.5, contextes frais). Pass 2 Haiku : 7 findings bruts → **0 réel**. 6 CRITICAL/HIGH/MEDIUM dismissés en bloc = **erreur de catégorie** (le validateur a traité la spec comme une implémentation à auditer : « les types/le flag/les wrappers n'existent pas » — c'est le travail à faire décrit par la spec, qui annonce elle-même ces absences « vérifié ») ; 1 LOW réfuté ground-truth (« ContactPersonsManager.svelte n'existe pas » — `ls` : le fichier existe, 4 258 octets, 16 références dans lint-i18n-ownership.js — hallucination Haiku, discipline grep CLAUDE.md appliquée) ; 1 « finding » était un constat de non-contradiction. Trend : 8 (2 HIGH) → 0. Statut ready-for-dev confirmé.
- 2026-07-11 — `validate-create-story` Pass 1 (Sonnet 5, contexte frais) : 8 findings (2 HIGH + 3 MEDIUM + 3 LOW), tous patchés. HIGH = références de lignes fausses héritées de la cartographie Explore (`invoices/[id]/+page.svelte` fait 640 l. — bloc validated :341-384, Payée le :431-436 ; MarkPaidDialog fait 106 l. — Footer :97-104), vérifiées ground-truth avant patch. MEDIUM = attribution `$props.id()` corrigée (MarkPaidDialog utilise un id en dur ; précédent réel = email-templates/+page.svelte:26), `AccountingTooltip` requalifié (pattern glossaire 2 clés, inutilisable pour un message libre → primitives bits-ui directement), promesse « le message 429 contient le délai » retirée (FTL `error-rate-limited` sans délai, header Retry-After non exposé par ApiError — story frontend-only). Le contrat backend et toutes les autres références (contacts, settings, feature-flags, lint i18n, E2E) vérifiés exacts par le validateur. Pass 2 (Haiku) à suivre.

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (claude-fable-5) — run unique 2026-07-11.

### Debug Log References

- Import types invoices.api.ts : virgule dupliquée lors de l'extension du bloc `import type` (erreur `Identifier expected` au check) — corrigé + tri alphabétique.
- Tooltip : premier jet avec `Tooltip.Provider` explicite → retiré (le wrapper shadcn `Tooltip.Root` embarque déjà son Provider, vérifié `tooltip.svelte`). Trigger via `{#snippet child({ props })}` sur un `<span>` (anti button-in-button + hover sur bouton disabled).

### Completion Notes List

- **T1-T7 conformes à la spec, 0 déviation.** Gate : `check` 0 erreur (26 warnings pré-existants), `lint-i18n-ownership` PASS (7 entrées SendEmailDialog ajoutées à KNOWN_VIOLATIONS, précédent MarkPaidDialog), `test:unit` **377/377** (+6 nouveaux : 2 wrappers invoices, 2 settings, 2 feature-flags), `build` OK, **E2E non-régression 15 passed / 2 skipped (pré-existants) / 0 failed** (contacts + invoices + email-templates, backend `kesh_e2e` migré forward au boot — `/health` y expose `smtpConfigured:false`, prouvant le contrat 20-3b1 en conditions réelles).
- Matrice d'erreurs du send implémentée par `switch (err.code)` : inline (422/500), toast+fermer (400 contact), toast+`setSmtpConfigured(false)` (412), toast (429), toast warning+`goto('/invoices')` (409 EMAIL_SENT_INVOICE_GONE).
- Civilité affichée uniquement pour `Personne` (payload force `Neutre` pour Entreprise) ; langue `''` → `null` (héritée). IDs en dur `form-language`/`form-salutation` (cohérence formulaire contacts) ; `$props.id()` dans SendEmailDialog (#145).
- E-mail société : édition inline Admin-only section Organisation, `isPlausibleEmail` client, 409 → reload + message, effacement → `null`.
- Périmètre : 0 fichier Rust hors FTL ×4 ; aucun nouveau spec Playwright (20-4) ; recovery/MarkPaidDialog/email-templates intouchés.

### File List

**Nouveaux**

- `frontend/src/lib/features/invoices/SendEmailDialog.svelte`
- `frontend/src/lib/features/invoices/invoices.api.test.ts`
- `frontend/src/lib/features/settings/settings.api.test.ts`
- `frontend/src/lib/shared/utils/feature-flags.svelte.test.ts`

**Modifiés — frontend**

- `frontend/src/lib/shared/utils/feature-flags.svelte.ts` (flag `smtpConfigured`)
- `frontend/src/routes/+layout.svelte` + `frontend/src/lib/shared/utils/api-health.svelte.ts` (2 call-sites /health)
- `frontend/src/lib/features/invoices/invoices.types.ts` (`emailedAt`/`emailedTo`, `EmailLanguage`, `EmailPreviewResponse`, `SendInvoiceEmailRequest`)
- `frontend/src/lib/features/invoices/invoices.api.ts` (`getInvoiceEmailPreview`, `sendInvoiceEmail`)
- `frontend/src/lib/features/contacts/contacts.types.ts` (`ContactLanguage`, `Salutation`, champs Response/requests)
- `frontend/src/lib/features/settings/settings.types.ts` (`CompanyJson.email`/`.version`, `UpdateCompanyEmailRequest`) + `settings.api.ts` (`updateCompanyEmail`)
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (bouton + tooltip disabled, handlers preview/send, « Envoyée le », montage SendEmailDialog)
- `frontend/src/routes/(app)/contacts/+page.svelte` (selects langue + civilité, states/reset/hydrate/payload)
- `frontend/src/routes/(app)/settings/+page.svelte` (e-mail société inline Admin)
- `frontend/scripts/lint-i18n-ownership.js` (KNOWN_VIOLATIONS ×7)

**Modifiés — i18n**

- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (27 clés ×4 — section Story 20-3b2)
