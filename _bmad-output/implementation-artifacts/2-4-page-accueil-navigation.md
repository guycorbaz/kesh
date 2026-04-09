# Story 2.4: Page d'accueil & navigation

Status: review

## Story

As a **utilisateur**,
I want **accéder rapidement aux fonctions principales depuis la page d'accueil**,
so that **je puisse travailler efficacement**.

### Contexte

Quatrième story de l'Epic 2. La sidebar, le header et le footer existent déjà (Story 1.10). Cette story enrichit la **page d'accueil** avec des widgets de résumé (états vides pour le MVP, les données viendront des Epics 3-6), crée une **page Paramètres centralisée** avec les sous-sections correspondant à l'onboarding, et active la **recherche globale** (placeholder fonctionnel). C'est une story frontend-only — pas de nouveaux endpoints backend.

### Décisions de conception

- **Homepage widgets vides** : les écritures, factures et soldes bancaires n'existent pas encore (Epics 3, 5, 6). Les widgets affichent des états vides mode-aware (Guidé: explication + suggestion + bouton, Expert: bouton seul). Les données réelles seront branchées par les Epics correspondants.
- **Page Paramètres** : sous-sections Organisation, Comptabilité, Comptes bancaires, Utilisateurs. Pour le MVP, les 3 premières affichent les données de l'onboarding en lecture seule (sans édition — l'édition viendra dans des stories futures). Utilisateurs redirige vers `/users`.
- **Recherche globale** : le champ de recherche dans le header passe de `disabled` à actif avec un message "Recherche bientôt disponible" (toast info au focus). L'implémentation réelle de la recherche est une story future.
- **Vocabulaire et labels** : déjà en place dans la sidebar (Story 1.10). Cette story vérifie la conformité et ajuste si nécessaire.
- **Disclaimer FR7** : déjà dans le footer du layout (Story 1.10). Vérifié conforme.

## Acceptance Criteria (AC)

1. **Page d'accueil** — Given utilisateur connecté, When affichage page d'accueil, Then 3 widgets : "Dernières écritures" (état vide), "Factures ouvertes" (état vide), "Soldes comptes bancaires" (état vide ou données si bank_account configuré). Chaque widget en mode Guidé affiche explication + suggestion + bouton. En mode Expert : bouton seul.
2. **Navigation sidebar** — Given sidebar, When affichage, Then navigation par activité organisée par fréquence — Quotidien: Accueil, Facturer, Payer, Import; Mensuel: Écritures, Réconciliation, Rapports; Séparé: Paramètres. Labels en vocabulaire utilisateur. (Déjà implémenté Story 1.10 — vérification de conformité.)
3. **Page Paramètres** — Given menu Paramètres, When affichage, Then configuration centralisée avec sous-sections : Organisation (nom, adresse, IDE, type), Comptabilité (langue comptable), Comptes bancaires (IBAN, banque), Utilisateurs (lien vers /users). Données en lecture seule pour le MVP.
4. **Recherche globale** — Given recherche globale dans le header, When focus ou saisie, Then toast "Recherche bientôt disponible — Epic futur". Le champ n'est plus `disabled`.
5. **Disclaimer FR7** — And disclaimer légal "ne remplace pas un fiduciaire" visible. (Déjà implémenté — vérification.)
6. **Pas de cul-de-sac** — And aucun cul-de-sac : toujours un chemin de retour, bouton navigateur fonctionnel.
7. **Tests** — And tests vitest pour les widgets homepage (rendu états vides), test Playwright page d'accueil + page Paramètres.

## Tasks / Subtasks

### T1 — Page d'accueil : widgets de résumé (AC: #1)
- [x] T1.1 Remplacer le contenu placeholder de `frontend/src/routes/(app)/+page.svelte` par 3 widgets (cards) : "Dernières écritures", "Factures ouvertes", "Soldes comptes bancaires".
- [x] T1.2 Chaque widget affiche un état vide mode-aware :
  - Mode Guidé (`modeState.value === 'guided'`) : titre + explication + suggestion + bouton d'action (ex: "Commencez par saisir votre première écriture")
  - Mode Expert : titre + bouton d'action seul
- [x] T1.3 Widget "Soldes comptes bancaires" : fetch `GET /api/v1/companies/current` (T2.6 — **implémenter T2.6 en premier**). Si `bankAccounts` non-vide, afficher le nom de la banque et "Aucune transaction importée". Sinon, état vide "Configurez un compte bancaire". Note : ne PAS utiliser `stepCompleted >= 7` car skip-bank atteint step 7 sans créer de bank_account.
- [x] T1.4 Layout responsive : grille 3 colonnes sur écran large, 1 colonne sur écran étroit (min-w-[1280px] garanti).

### T2 — Endpoint backend + Page Paramètres centralisée (AC: #3)
- [x] T2.1 **Backend d'abord** — Créer `crates/kesh-api/src/routes/companies.rs` avec `GET /api/v1/companies/current` : retourne `{ company: Company, bankAccounts: BankAccount[] }` en camelCase. Authentifié (tout rôle). Utilise `companies::list(pool, 1, 0)` + `bank_accounts::list_by_company()`. Si aucune company → 404. Ajouter `pub mod companies;` dans `routes/mod.rs`, enregistrer dans `authenticated_routes` de `build_router()`.
- [x] T2.2 Frontend API + types : `CompanyCurrentResponse` dans `frontend/src/lib/features/settings/settings.types.ts`, fonction `fetchCompanyCurrent()` dans `settings.api.ts`.
- [x] T2.3 Créer `frontend/src/routes/(app)/settings/+page.svelte` — remplacer le placeholder. Page avec 4 sous-sections (cards) : Organisation, Comptabilité, Comptes bancaires, Utilisateurs.
- [x] T2.4 Section "Organisation" : afficher name, address, IDE, org_type, instance_language depuis `GET /api/v1/companies/current`.
- [x] T2.5 Section "Comptabilité" : afficher accounting_language. Note : `instance_language` va dans la section Organisation (pas Comptabilité — c'est la langue d'interface, pas comptable).
- [x] T2.6 Section "Comptes bancaires" : afficher bank_name, IBAN depuis bankAccounts. Bouton "Modifier" → toast "Édition bientôt disponible".
- [x] T2.7 Section "Utilisateurs" : lien vers `/users` (page existante Story 1.12).

### T3 — Recherche globale (AC: #4)
- [x] T3.1 Modifier `(app)/+layout.svelte` : retirer `disabled` du champ de recherche.
- [x] T3.2 Au focus ou à la saisie, afficher un toast info "Recherche bientôt disponible". Debounce pour éviter le spam de toasts.

### T4 — Clés i18n (AC: #1, #3, #4)
- [x] T4.1 Ajouter les clés dans les 4 fichiers `.ftl` :
  - `homepage-entries-title` / `homepage-entries-empty` / `homepage-entries-empty-guided` / `homepage-entries-action`
  - `homepage-invoices-title` / `homepage-invoices-empty` / `homepage-invoices-empty-guided` / `homepage-invoices-action`
  - `homepage-bank-title` / `homepage-bank-empty` / `homepage-bank-empty-guided` / `homepage-bank-configured` / `homepage-bank-no-transactions`
  - `settings-title` / `settings-org-title` / `settings-accounting-title` / `settings-bank-title` / `settings-users-title`
  - `settings-field-name` / `settings-field-address` / `settings-field-ide` / `settings-field-org-type` / `settings-field-instance-language` / `settings-field-accounting-language`
  - `search-coming-soon`

### T5 — Tests (AC: #7)
- [x] T5.1 Tests vitest : widgets homepage (rendu état vide guidé vs expert).
- [x] T5.2 Test Playwright : page d'accueil affiche 3 widgets, page Paramètres affiche 4 sections.
- [x] T5.3 Tests E2E API : `GET /api/v1/companies/current` retourne la company.

## Dev Notes

### État existant de la sidebar (Story 1.10)

La sidebar est déjà implémentée dans `(app)/+layout.svelte` avec exactement la structure demandée par l'AC #2 :
```
Quotidien: Accueil, Facturer, Payer, Importer
Mensuel: Écritures, Réconciliation, Rapports
Séparé: Paramètres
Admin: Utilisateurs (conditionnel)
```
Les labels sont déjà en vocabulaire utilisateur. **Pas de modification nécessaire** sauf vérification.

Note : l'épic dit "Paiements" — le codebase utilise "Payer" (plus actif, vocabulaire utilisateur). L'épic ne mentionne pas "Accueil" dans le groupe Quotidien — ajouté car c'est le point d'entrée principal. Ces choix de vocabulaire sont intentionnels (UX spec "vocabulaire utilisateur, pas technique").

### Widgets homepage — pattern

```svelte
<div class="grid grid-cols-3 gap-6">
  <div class="rounded-lg border p-6">
    <h3 class="text-lg font-semibold">{title}</h3>
    {#if modeState.value === 'guided'}
      <p class="mt-2 text-text-muted">{explanation}</p>
      <p class="mt-1 text-sm">{suggestion}</p>
    {/if}
    <Button class="mt-4">{actionLabel}</Button>
  </div>
</div>
```

### Page Paramètres — données

Pour afficher les données company et bank_accounts, on a besoin d'un nouvel endpoint API `GET /api/v1/companies/current`. Pattern simple :
- Authentifié (tout rôle)
- Retourne `{ company: Company, bankAccounts: BankAccount[] }`
- En camelCase

Cet endpoint utilise les repositories existants `companies::list(pool, 1, 0)` et `bank_accounts::list_by_company(pool, company_id)`. Route à enregistrer dans le bloc `authenticated_routes` de `build_router()` (tout rôle, authentifié).

### Recherche globale — toast debounce

Le champ de recherche est déjà dans le header avec `disabled`. Retirer `disabled`, ajouter un handler `onfocus` ou `oninput` avec un toast une seule fois (flag `searchToastShown`).

### Endpoint backend léger

C'est la seule pièce backend de cette story. Créer `routes/companies.rs` avec un handler `GET /api/v1/companies/current`. Pas de CRUD complet — juste un GET read-only.

### Project Structure Notes

- **Modification** : `frontend/src/routes/(app)/+page.svelte` (widgets homepage)
- **Modification** : `frontend/src/routes/(app)/settings/+page.svelte` (page paramètres)
- **Modification** : `frontend/src/routes/(app)/+layout.svelte` (search field enabled)
- **Nouveau** : `crates/kesh-api/src/routes/companies.rs` (GET /api/v1/companies/current)
- **Modifications** : `routes/mod.rs`, `lib.rs` (enregistrer route)
- **Modifications i18n** : `crates/kesh-i18n/locales/fr-CH/messages.ftl`, `de-CH/messages.ftl`, `it-CH/messages.ftl`, `en-CH/messages.ftl`

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-2-Story-2.4] — AC BDD
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Navigation] — Sidebar, vocabulaire, anti-pattern Bexio
- [Source: _bmad-output/planning-artifacts/prd.md#FR82] — Page d'accueil
- [Source: _bmad-output/planning-artifacts/prd.md#FR7] — Disclaimer légal
- [Source: _bmad-output/implementation-artifacts/2-2-flux-onboarding-chemin-a-exploration.md] — Layout existant
- [Source: _bmad-output/implementation-artifacts/2-3-flux-onboarding-chemin-b-production.md] — Données company/bank

## Dev Agent Record

### Agent Model Used

Opus 4.6

### Debug Log References

### Completion Notes List

- T2.1: Backend endpoint `GET /api/v1/companies/current` — retourne company + bankAccounts en camelCase. 404 si aucune company. 3 tests E2E (200, 404, 401).
- T1: Homepage 3 widgets mode-aware (écritures, factures, banque). Fetch companies/current pour widget banque. Grid 3 cols.
- T2.3-T2.7: Page Paramètres avec 4 sections (Organisation, Comptabilité, Comptes bancaires, Utilisateurs). Données read-only.
- T3: Recherche globale activée (disabled → onfocus toast, flag once-per-session).
- T4: 28 clés i18n dans 4 locales (homepage + settings + search).
- Aucune régression : onboarding E2E 9/9, Path B E2E 6/6, vitest 50/50.

### File List

#### New Files
- `crates/kesh-api/src/routes/companies.rs` — GET /api/v1/companies/current
- `crates/kesh-api/tests/companies_e2e.rs` — 3 tests E2E
- `frontend/src/lib/features/settings/settings.types.ts` — CompanyCurrentResponse types
- `frontend/src/lib/features/settings/settings.api.ts` — fetchCompanyCurrent()

#### Modified Files
- `crates/kesh-api/src/routes/mod.rs` — pub mod companies
- `crates/kesh-api/src/lib.rs` — route /api/v1/companies/current dans authenticated_routes
- `frontend/src/routes/(app)/+page.svelte` — 3 widgets homepage mode-aware
- `frontend/src/routes/(app)/settings/+page.svelte` — page Paramètres 4 sections
- `frontend/src/routes/(app)/+layout.svelte` — search enabled + toast + IncompleteBanner import
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` — 28 clés homepage/settings
- `crates/kesh-i18n/locales/de-CH/messages.ftl` — 28 clés
- `crates/kesh-i18n/locales/it-CH/messages.ftl` — 28 clés
- `crates/kesh-i18n/locales/en-CH/messages.ftl` — 28 clés

## Change Log

| Date | Passe | Modèle | Findings | Patches |
|------|-------|--------|----------|---------|
| 2026-04-09 | Implémentation | Opus 4.6 | — | T1-T5 complètes, E2E 3/3 + vitest 50/50, aucune régression |
