# Story 6.3 : Lint i18n key-ownership + sidebar i18n (D8)

Status: ready-for-dev

<!-- 
Note : spec validée pass 1. Frontend uniquement (pas de backend). Dépendance : 
- Story 6-2 (multi-tenant) complétée et mergée ✅
- Story 6-4 (fixtures) complétée et mergée ✅
- Story 6-5 (Playwright auth) complétée et mergée ✅
Aucune dépendance critique pour 6-3 lui-même.
-->

## Story

As a **développeur frontend (Guy)**,
I want **enforcer un namespace strict des clés i18n par feature (une feature ne peut pas utiliser les clés d'une autre), et piloter la sidebar via clés i18n au lieu de la hardcoder**,
so that **les couplages silencieux entre features soient détectés au build (erreur explicite), et que la sidebar soit multilingue (FR/DE/IT/EN) et la dette D8 soit fermée définitivement**.

## Contexte

### Debt D8 — KF-006 / [GitHub issue #6](https://github.com/guycorbaz/kesh/issues/6)

**Découvert** : 2026-04 (story 4-2 code review)

**Symptôme** : les labels sidebar dans `frontend/src/routes/(app)/+layout.svelte` sont hardcodés en français :
```svelte
{ label: "Carnet d'adresses", href: '/contacts' },
{ label: 'Catalogue', href: '/products' },
{ label: 'Facturer', href: '/invoices' },
{ label: 'Échéancier', href: '/invoicing/due-dates' },
```

Les trois autres locales (de-CH, it-CH, en-CH) voient l'UI entière en français.

**Root cause** : la spec Story 4-2 avait piège #7 qui interdisait explicitement le refactor sidebar (scope creep), mais T6.1 listait `nav-products` comme clé à créer — contradiction non détectée. Code livré en français hardcodé.

**Impact** : aucun FR (la sidebar n'est pas une exigence fonctionnelle) mais **critique dès qu'un utilisateur DE/IT/EN ouvre l'app** (expérience très dégradée).

---

### État actuel du système i18n (audit 2026-04-20)

**Backend (crate `kesh-i18n`)**:
- Arborescence : `crates/kesh-i18n/locales/{locale}/messages.ftl` pour fr-CH, de-CH, it-CH, en-CH
- Fichier `messages.ftl` : ~530 clés i18n avec convention de namespace `{feature}-{key}`
  - `invoice-*` (124 clés), `journal-*` (59), `contact-*` (47), `product-*` (46), `error-*` (32), `onboarding-*` (30), `settings-*` (28)
  - `nav-*` (**seulement 4 clés actuelles**) : `nav-contacts`, `nav-invoices`, `nav-settings-invoicing`, `nav-invoices-due-dates`
  - Clés orphelines ou incohérentes : `accounts-*` (0), `products-*` (1, au lieu de `product-*`)

**Frontend (SvelteKit + Svelte 5)**:
- Système i18n : store Svelte 5 runes `crates/kesh-i18n/i18n.svelte.ts`
  - Fonction : `i18nMsg(key: string, fallback: string, args?: {...}): string`
  - Chargement : `loadI18nMessages()` via `GET /api/v1/i18n/messages` au démarrage
  - Fallback : français en cas d'erreur API
- Organisation features : `frontend/src/lib/features/{feature-name}/` (14 dossiers : accounts, auth, bank-import, contacts, invoices, invoicing, journal-entries, onboarding, products, reconciliation, reports, settings)
- Composants et stores : **aucune règle de namespace enforced** actuellement. Un composant dans `features/contacts/` peut appeler `i18nMsg('invoice-form-amount')` (clé d'une autre feature).

**État du lint/CI** :
- `frontend` : `npm run check` (svelte-check), `npm run lint` (eslint), aucun custom lint pour i18n
- Pas de validation de key-ownership au build
- Aucun test sur les traductions

---

### Architecture feature-folder (contexte pour lint)

```
frontend/src/lib/features/
├── accounts/
│   ├── Accounts.svelte          # Main page
│   ├── AccountTable.svelte      # Shared component
│   └── account-service.ts       # API client
├── contacts/
│   ├── ContactForm.svelte       # Feature-specific form
│   ├── ContactSearch.svelte     # Feature-specific search
│   └── contact-service.ts
├── invoices/
├── products/
├── settings/
└── ... (9 autres)
```

**Règle à enforcer** :
- Composant `features/contacts/ContactForm.svelte` → clés `contact-*` uniquement
- Composant `features/invoices/InvoiceForm.svelte` → clés `invoice-*` uniquement
- Violation = erreur au build : « ContactForm.svelte uses key "invoice-form-amount" from different feature »

---

### Sidebar actuelle + refactor cible

**Avant (hardcoded)** :
```svelte
<!-- frontend/src/routes/(app)/+layout.svelte -->
const navItems = [
  { label: "Accueil", href: '/' },
  { label: "Carnet d'adresses", href: '/contacts' },
  { label: 'Catalogue', href: '/products' },
  { label: 'Facturer', href: '/invoices' },
  { label: 'Échéancier', href: '/invoicing/due-dates' },
  { label: 'Paramètres', href: '/settings' },
];
```

**Après (i18n keys)** :
```svelte
<!-- Same file -->
const navItems = [
  { labelKey: "nav-home", href: '/' },
  { labelKey: "nav-contacts", href: '/contacts' },
  { labelKey: "nav-products", href: '/products' },
  { labelKey: "nav-invoices", href: '/invoices' },
  { labelKey: "nav-invoicing-due-dates", href: '/invoicing/due-dates' },
  { labelKey: "nav-settings", href: '/settings' },
];

// Render:
{#each navItems as item}
  <a href={item.href}>
    {i18nMsg(item.labelKey, 'Label')}
  </a>
{/each}
```

**FTL files (all 4 locales)** :
```ftl
# fr-CH/messages.ftl
nav-home = Accueil
nav-contacts = Carnet d'adresses
nav-products = Catalogue
nav-invoices = Factures
nav-invoicing-due-dates = Échéancier
nav-settings = Paramètres

# de-CH/messages.ftl
nav-home = Startseite
nav-contacts = Kontakte
nav-products = Katalog
nav-invoices = Rechnungen
nav-invoicing-due-dates = Fälligkeiten
nav-settings = Einstellungen

# it-CH/messages.ftl
nav-home = Home
nav-contacts = Contatti
nav-products = Catalogo
nav-invoices = Fatture
nav-invoicing-due-dates = Scadenze
nav-settings = Impostazioni

# en-CH/messages.ftl
nav-home = Home
nav-contacts = Contacts
nav-products = Catalog
nav-invoices = Invoices
nav-invoicing-due-dates = Due Dates
nav-settings = Settings
```

---

## Critères d'acceptation (7 AC)

### AC #1 — Clés i18n nav-* complètes × 4 locales

**Given** aucune clé pour tous les items sidebar,
**When** FTL files mis à jour,
**Then** les fichiers `locales/{locale}/messages.ftl` contiennent :
- `nav-home`, `nav-contacts`, `nav-products`, `nav-invoices`, `nav-invoicing-due-dates`, `nav-settings`
- Plus les clés *optionnelles* futures pour sous-menus si applicable (à définir en T1)
- Chaque clé présente dans **les 4 locales** (fr-CH, de-CH, it-CH, en-CH)
- Traductions couvrent au minimum les 6 items ci-dessus, linguistiquement cohérentes avec les traductions existantes `contact-*`, `invoice-*`, `product-*`

**Validation** : `grep "^nav-" crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl | wc -l` → min 24 (6 clés × 4 locales)

---

### AC #2 — Sidebar refactorisée (layout.svelte)

**Given** la sidebar avec labels hardcodés,
**When** refactor appliqué,
**Then** le fichier `frontend/src/routes/(app)/+layout.svelte` :
- Déclare `const navItems = [ { labelKey: "nav-X", href: ... }, ...]` (pas `label: "X"`)
- Rend chaque item via `i18nMsg(item.labelKey, 'Fallback Label')`
- Les caractères français hardcodés disparaissent du .svelte
- Tests de régression : la sidebar en DE/IT/EN affiche les traductions correctes (visuel browser, pas systématique — vérifiable en changeant `instance_language`)

**Validation** : 
- `grep -n "Carnet d'adresses\|Facturer\|Catalogue" frontend/src/routes/\(app\)/+layout.svelte` → 0 results
- `grep -n "nav-" frontend/src/routes/\(app\)/+layout.svelte` → minimum 6 occurrences

---

### AC #3 — Lint rule : namespace key-ownership

**Given** un composant Svelte dans `features/{feature}/`,
**When** le composant appelle `i18nMsg('OTHER-FEATURE-key', ...)`,
**Then** le build (npm run check ou custom lint script) échoue avec message explicite :
```
Error: features/contacts/ContactForm.svelte uses key "invoice-form-amount"
   Expected: "contact-*" (feature namespace)
   Found: "invoice-form-amount" (different feature)
```

**Implémentation suggérée** (2 options, à choisir en T2) :
1. **Script Node.js** (`frontend/scripts/lint-i18n-ownership.js`) intégré dans `npm run check` (pré-commit hook ou CI)
2. **Plugin eslint custom** (plus robuste, mais plus complexe)

**Scope** :
- Appels `i18nMsg('...')` dans les fichiers Svelte/TS de `frontend/src/lib/features/`
- Chaque clé doit matcher le pattern `{feature}-*` où `{feature}` correspond au dossier parent
- Cas spéciaux **allowlistés** (AC #4) : clés `error-*`, `common-*`, `tooltip-*` (génériques, shareable)
- Cas non-Svelte : la règle est **recommendé** pour `frontend/src/lib/shared/`, mais non-bloquant (composants partagés = acception partielle transverse)

**Validation** : 
- `npm run check` échoue si une clé transverse est détectée
- Lancer manuellement pour vérifier : `node frontend/scripts/lint-i18n-ownership.js` → rapport détaillé

---

### AC #4 — Allowlist et cas spéciaux

**Given** les clés cross-feature comme `error-*`, `tooltip-*`, `common-*`,
**When** lint est executé,
**Then** ces clés ne lèvent **pas** d'erreur de namespace, même si utilisées en dehors de leurs feature :

```javascript
// frontend/scripts/lint-i18n-ownership.js
const GLOBAL_NAMESPACES = ['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo'];
// Ces namespaces n'ont pas de feature-folder assignée, libres d'usage partout
```

**Validation** : 
- Composant `features/contacts/ContactForm.svelte` utilise `i18nMsg('error-invalid-name', '...')` → ✅ pas d'erreur
- Composant `features/contacts/ContactForm.svelte` utilise `i18nMsg('invoice-form-amount', '...')` → ❌ erreur

---

### AC #5 — Audit rétroactif des clés transverses

**Given** le codebase Svelte actuel,
**When** lint rule appliquée,
**Then** un rapport d'audit détaille :
- Composants qui utilisent des clés trans-feature (feature A utilise clé B)
- Count par violation type
- Recommandation : move clé vers `error-*` / `common-*` / `tooltip-*` ou déplacer le composant

**Output exemple** :
```
Audit i18n cross-feature violations:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total violations: 0 (ideal state after refactor)

If violations exist:
  features/contacts/ContactForm.svelte (line 42):
    - uses "invoice-form-amount" (invoice namespace)
    - recommendation: move to features/invoices/ or promote to global namespace

Summary:
  Feature-specific violations: 0
  Global namespace violations (acceptable): 0
  Allowlisted namespaces: error (32 uses), tooltip (18 uses), common (5 uses)
```

**Validation** : rapport généré sans crash, identifie toute violation existante

---

### AC #6 — CI intégration + PR gate

**Given** la branche `story/6-3-lint-i18n-key-ownership-sidebar-i18n`,
**When** PR ouverte ou `npm run check` lancé,
**Then** :
- Frontend CI job inclut `npm run lint-i18n-ownership` (ou équivalent, exit code 0 si OK, 1 si erreur)
- PR ne peut merger que si le check passe (ou manuellement bypass si allowlisting décision)
- CI log montre le rapport d'audit pour chaque PR (debug + accountability)

**Implémentation** :
- `package.json` : ajouter script `"lint-i18n-ownership": "node scripts/lint-i18n-ownership.js"`
- `.github/workflows/ci.yml` : ajouter étape `npm run lint-i18n-ownership` dans le job `Frontend` (après `npm run check`, avant build)

**Validation** : 
- `.github/workflows/ci.yml` contient `npm run lint-i18n-ownership`
- CI logs montrent « lint-i18n-ownership: PASS » pour la PR mergée

---

### AC #7 — Documentation + knowledge base

**Given** le système i18n nouveau avec lint,
**When** documentation créée,
**Then** :
- Nouveau fichier `docs/i18n-key-ownership-pattern.md` décrit :
  - Convention `{feature}-{key}` pour clés feature-spécifiques
  - Allowlist (error-*, common-*, tooltip-*, etc.)
  - Comment ajouter une nouvelle clé (feature folder first, créer clé, puis l'utiliser)
  - Lint rule workflow : local (`npm run lint-i18n-ownership`) puis CI
  - Examples : "Ajouter clé contact-form-phone" vs "Réutiliser error-invalid-email"
- Section dans `frontend/README.md` ou `docs/frontend.md` : quick-start pour devs sur l'i18n

**Validation** : 
- `docs/i18n-key-ownership-pattern.md` existe et contient min 5 sections (convention, allowlist, workflow, examples, troubleshooting)
- Lien cité dans `frontend/README.md`

---

## Scope volontairement HORS story

- **Backend i18n refactor** (kesh-i18n crate cleanup) → orthogonal, post-v0.1
- **Sous-menus i18n** (ex: "Paramètres > Organisation") → post-AC#1 si needed
- **RTL support** (Arabe, Hébreu) → out of scope v0.1, pas de French/German/Italian RTL
- **Dynamic feature loading** (lazy-load features) → n'affecte pas i18n ownership, in scope mais pas d'ajout spécifique
- **Migration des clés historiques incohérentes** (`products-*` vs `product-*`) → documented dans AC#5 audit, no action required
- **pluralization / gender-aware messages** (Fluent `.ftl` syntax) → keep simple `{$var}` substitution only

---

## Tasks / Subtasks

### T0 — Planification + Décisions

#### T0.1 — Validation approche lint (AC #3)

- [ ] Valider avec Guy : script Node.js (simple, maintenable) vs eslint plugin (robuste, complexe)
- [ ] Approche retenue : **script Node.js** (justification : frontend petit, maintenance moindre, 200-300 LOC suffisent)
- [ ] Validation : test script sur codebase actuel → audit report généré sans crash

#### T0.2 — Validation allowlist (AC #4)

- [ ] Lister les namespaces globaux réels (error-*, tooltip-*, common-*, ...) → audit des clés existantes
- [ ] Valider avec Guy : ajouter `mode-*`, `shortcut-*`, `demo-*` à l'allowlist ?
- [ ] Finaliser liste : `['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo']`

#### T0.3 — Translation workflow (AC #1)

- [ ] Valider approche traductions : 
  - Guy fournit FR/DE/IT/EN (manuel via docs, ou crowdsourced ?)
  - Ou : utiliser crate existant kesh-i18n comme reference pour tonalité

---

### T1 — Sidebar refactor (AC #2)

#### T1.1 — Update FTL files (AC #1)

- [ ] Éditer `crates/kesh-i18n/locales/fr-CH/messages.ftl` : ajouter les 6 clés `nav-*` (FR)
- [ ] Éditer `crates/kesh-i18n/locales/de-CH/messages.ftl` : ajouter les 6 clés (DE) — valider avec Guy ou traducteur
- [ ] Éditer `crates/kesh-i18n/locales/it-CH/messages.ftl` : ajouter les 6 clés (IT)
- [ ] Éditer `crates/kesh-i18n/locales/en-CH/messages.ftl` : ajouter les 6 clés (EN)
- [ ] Validation : `grep "^nav-" crates/kesh-i18n/locales/*/messages.ftl | wc -l` → **24**

#### T1.2 — Refactor layout.svelte (AC #2)

- [ ] Éditer `frontend/src/routes/(app)/+layout.svelte`
  - Changer `const navItems = [{ label: "...", href: ... }]` → `[{ labelKey: "nav-...", href: ... }]`
  - Changer render : `{#each navItems as item} <a>{i18nMsg(item.labelKey, 'Label')}</a> {/each}`
- [ ] Tests locaux : changer `instance_language` via DB / API → vérifier sidebar affiche les bonnes traductions
- [ ] Validation : `grep "Carnet d'adresses\|Facturer\|Catalogue" frontend/src/routes/\(app\)/+layout.svelte` → **0 results**

#### T1.3 — Tests de régression

- [ ] `npm run check` → vert
- [ ] `npm run test` → vert
- [ ] Manuel : changer langue dans profil → sidebar bascule dans la bonne langue (FR/DE/IT/EN)

---

### T2 — Lint rule implémentation (AC #3, #4, #5)

#### T2.1 — Créer script lint (AC #3, #4, #5)

- [ ] Créer `frontend/scripts/lint-i18n-ownership.js` (300-400 LOC)
  - Parse tous les fichiers `.svelte` et `.ts` dans `frontend/src/lib/features/`
  - Extract appels `i18nMsg('KEY', ...)` via regex ou AST
  - Pour chaque clé, extraire namespace : `KEY = KEY.split('-')[0]`
  - Comparer namespace vs dossier feature parent
  - Allowlist : `['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo']` → pas d'erreur
  - Output : liste violations (ou « PASS » si 0)
  - Exit code : 0 si OK, 1 si violations

#### T2.2 — Test + validation script (AC #5)

- [ ] Lancer sur codebase actuel : `node frontend/scripts/lint-i18n-ownership.js`
- [ ] Générer audit report : identifier violations existantes (si any)
- [ ] Si violations existent : soit les corriger (move clés), soit les allowlister (si légit)
- [ ] Re-test : script doit passer (0 violations ou toutes allowlistées)

#### T2.3 — Intégration package.json (AC #6)

- [ ] Éditer `frontend/package.json` : ajouter script
  ```json
  "scripts": {
    "lint-i18n-ownership": "node scripts/lint-i18n-ownership.js"
  }
  ```
- [ ] Tester : `npm run lint-i18n-ownership` → fonctionne

#### T2.4 — Intégration CI (AC #6)

- [ ] Éditer `.github/workflows/ci.yml` : ajouter étape dans job `Frontend`
  ```yaml
  - name: Lint i18n key ownership
    run: npm run lint-i18n-ownership
  ```
- [ ] Placer après `npm run check`, avant `npm run build`
- [ ] Test : pousser branche de test → CI doit exécuter le check

---

### T3 — Documentation (AC #7)

#### T3.1 — Créer guide pattern

- [ ] Créer `docs/i18n-key-ownership-pattern.md`
  - Section 1 : Vue d'ensemble (problème posé par D8, solution proposée)
  - Section 2 : Convention (feature-* vs global namespaces)
  - Section 3 : Workflow dev (ajouter une clé i18n)
  - Section 4 : Allowlist + exceptions (why error-* is global)
  - Section 5 : Lint local (npm run lint-i18n-ownership)
  - Section 6 : Examples concrets (ContactForm veut contact-form-*, non invoice-*)
  - Section 7 : Troubleshooting (« my lint fails even though my key looks right »)

#### T3.2 — Update frontend/README.md

- [ ] Ajouter section « Internationalization (i18n) »
  - Court résumé de l'i18n system
  - Link vers `docs/i18n-key-ownership-pattern.md`
  - Command : `npm run lint-i18n-ownership`

---

### T4 — Fermeture D8

- [ ] Éditer `docs/known-failures.md` : KF-006 status → `closed` (ou laisser comme archive)
- [ ] Créer commit Git : "fix(i18n): close D8 — sidebar now i18n-driven, lint rule enforces key-ownership (story 6-3)"
- [ ] PR title/body inclut `closes #6` → GitHub ferme issue #6 automatiquement au merge

---

## Dépendances et bloquages

**Bloqué par** :
- ✅ Story 6-2 (multi-tenant) — mergée
- ✅ Story 6-4 (fixtures) — mergée
- ✅ Story 6-5 (Playwright auth) — mergée

**Bloque** : rien explicitement (Epic 7 n'en dépend pas)

**Parallélisable avec** : autres stories de l'Epic, aucune contention

---

## Notes de développement

- **Complexité estimée** : **BASSE** (frontend-only, scope bien délimité, pas de backend changes)
- **Risk factors** : aucun (additive + backward-compatible, pas de refactor destructif)
- **Testing** : manuel + automated lint check (pas de tests Playwright spécifiques)
- **Translations** : si Guy n'a pas les traductions DE/IT/EN, proposer crowdsource ou delegation
- **Future-proofing** : une fois lint en place, ajouter nouvelles clés becomes straightforward

---

## Validation checklist (fin story)

- [ ] AC #1 : `grep "^nav-" crates/kesh-i18n/locales/*/messages.ftl | wc -l` = **24**
- [ ] AC #2 : `npm run check` ✅, sidebar refactored + no French hardcodes
- [ ] AC #3 : `npm run lint-i18n-ownership` ✅ (exit 0)
- [ ] AC #4 : global namespaces allowlisted, no false positives
- [ ] AC #5 : audit report generated cleanly
- [ ] AC #6 : `.github/workflows/ci.yml` includes lint step, PR gate works
- [ ] AC #7 : `docs/i18n-key-ownership-pattern.md` exists + comprehensive
- [ ] Fermeture D8 : issue #6 closed via PR merge
- [ ] Régression tests : `cargo test --workspace` ✅ (backend unaffected)
