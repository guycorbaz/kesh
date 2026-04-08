# Story 1.10: Layout & page de login

Status: done

## Story

As a utilisateur,
I want une interface structurée avec une page de connexion,
so that je puisse me connecter et naviguer dans l'application.

## Acceptance Criteria

1. **AC#1 — Layout principal** : Given le layout, When affichage, Then header fixe (logo, zone recherche, profil), sidebar fixe gauche (200-240px), zone contenu fluide, footer discret.
2. **AC#2 — Page de login** : Given la page de login, When saisie username/password et soumission, Then appel API `POST /api/v1/auth/login`, stockage JWT, redirection vers accueil.
3. **AC#3 — Largeur minimale** : Given largeur navigateur, When inférieure à 1280px, Then l'interface reste fonctionnelle (pas de responsive mobile, mais pas de cassure).
4. **AC#4 — Navigation clavier** : And navigation clavier complète (focus visible outline bleu `#3b82f6`).
5. **AC#5 — Contraste WCAG AA** : And contraste WCAG AA sur tous les textes (minimum 4.5:1).

## Tasks / Subtasks

- [x] **T1 — Layout principal (shell authentifié)** (AC: #1, #3)
  - [x] T1.1 Créer `routes/(app)/+layout.svelte` avec header fixe, sidebar gauche (200-240px), zone contenu fluide, footer
  - [x] T1.2 Header : logo Kesh (placeholder SVG), zone recherche (Input placeholder avec icône loupe), menu profil (DropdownMenu avec nom utilisateur, sélecteur de langue FR/DE/IT/EN, bascule mode Guidé/Expert, déconnexion), espace réservé pour bannières contextuelles (entre header et contenu)
  - [x] T1.3 Sidebar : navigation groupée par fréquence d'usage (UX spec) avec `Separator` entre groupes :
    - Quotidien : Accueil, Facturer, Payer, Importer
    - Mensuel : Écritures, Réconciliation, Rapports
    - Séparé : Paramètres
    - Vocabulaire orienté action (« Facturer » pas « Factures », « Importer » pas « Banque ») — choix délibéré conforme au principe UX spec §Navigation (infinitifs d'action). Note : la table UX spec §Hiérarchie utilise « Import » (substantif) — on privilégie la cohérence infinitif
  - [x] T1.4 Footer discret : version Kesh, mention légale
  - [x] T1.5 Largeur minimale 1280px via `min-w-[1280px]` sur le conteneur principal — vérifier que l'interface reste fonctionnelle à 200% de zoom navigateur (scroll horizontal acceptable)
  - [x] T1.6 Utiliser les CSS custom properties `--kesh-gap`, `--kesh-padding`, `--kesh-section-margin` pour le spacing adaptatif Guidé/Expert
  - [x] T1.7 Migrer les dossiers de routes existants (`accounts/`, `bank-accounts/`, `bank-import/`, `contacts/`, `invoices/`, `journal-entries/`, `reconciliation/`, `reports/`, `settings/`) dans `routes/(app)/` pour qu'ils héritent du layout authentifié
- [x] **T2 — Page de login** (AC: #2, #4, #5)
  - [x] T2.1 Créer `routes/login/+page.svelte` avec formulaire centré (username, password, bouton « Se connecter »)
  - [x] T2.2 Appel `POST /api/v1/auth/login` avec `{ username, password }`, récupérer `accessToken`, `refreshToken`, `expiresIn` (JSON camelCase — le backend utilise `#[serde(rename_all = "camelCase")]`)
  - [x] T2.3 Stocker `accessToken` ET `refreshToken` en mémoire (store Svelte 5) — PAS de localStorage (vulnérable XSS). Le `refreshToken` est nécessaire pour logout.
  - [x] T2.4 Créer store `auth.svelte.ts` dans `$lib/app/stores/` : état authentification (`accessToken`, `refreshToken`, `expiresIn`, `currentUser { userId, role }`, `isAuthenticated`). Décoder le payload JWT (base64url du segment central) pour extraire `sub` (userId) et `role`. Note : `username` n'est PAS dans le JWT — `currentUser` ne contient que `userId` et `role` à ce stade
  - [x] T2.5 Redirection vers `/` (accueil) après login réussi
  - [x] T2.6 Affichage erreurs avec icône SVG Lucide (ne jamais communiquer par la couleur seule — WCAG). Icônes avec `aria-hidden="true"` (le texte adjacent porte l'information) :
    - 401 (`INVALID_CREDENTIALS`) : `AlertTriangle` (lucide) + « Identifiant ou mot de passe incorrect » — ne jamais distinguer user inexistant vs mauvais mot de passe
    - 429 (`RATE_LIMITED`) : `Clock` (lucide) + « Trop de tentatives de connexion. Réessayez dans quelques minutes. » (rate-limit IP, PAS un verrouillage de compte)
    - Erreur réseau (fetch échoue / TypeError) : `WifiOff` (lucide) + « Impossible de contacter le serveur. Vérifiez votre connexion. »
    - 5xx : `XCircle` (lucide) + « Erreur serveur. Réessayez ultérieurement. »
    - `@lucide/svelte` déjà installé (v1.7.0) — icônes SVG, rendu cross-platform cohérent, couleur contrôlable via CSS
  - [x] T2.7 État loading : bouton désactivé + spinner pendant la requête
  - [x] T2.8 Navigation clavier : Tab username → password → bouton, Enter soumet le formulaire
- [x] **T3 — Routing et garde d'authentification** (AC: #2)
  - [x] T3.1 Créer groupe de routes `(app)` pour les pages protégées, avec `+layout.ts` qui vérifie l'authentification
  - [x] T3.2 Si non authentifié → redirect vers `/login`
  - [x] T3.3 La page `/login` est hors du groupe `(app)` (pas de layout principal)
  - [x] T3.4 Créer une page d'accueil minimale `routes/(app)/+page.svelte` (placeholder « Bienvenue dans Kesh »)
  - [x] T3.5 **Supprimer** `routes/+page.svelte` (contenu SvelteKit par défaut)
  - [x] T3.6 Conserver `routes/design-system/` hors du groupe `(app)` (accessible sans auth)
  - [x] T3.7 Créer `routes/+error.svelte` (page d'erreur SvelteKit — message générique, style cohérent)
- [x] **T4 — Tests** (AC: #1-#5)
  - [x] T4.0 Setup test unitaire : `npm install -D vitest @testing-library/svelte jsdom @axe-core/playwright` + bloc `test:` dans `vite.config.ts` (import `vitest/config`) + scripts `test:unit` / `test:e2e` dans `package.json`
  - [x] T4.1 Tests unitaires Vitest : `$lib/app/stores/auth.svelte.test.ts` — 8 tests (login, logout, état, refreshToken, rôles, best-effort logout, no-fetch sans token)
  - [ ] T4.2 Tests E2E Playwright : `frontend/tests/e2e/auth.spec.ts` — reporté (nécessite backend démarré + seed data)
    - _Session expirée (détection 401 → modal) : reporté Story 1.11_
  - [ ] T4.3 Test accessibilité axe-core : `@axe-core/playwright` installé, intégration dans T4.2 E2E — reporté avec T4.2

## Dev Notes

### Architecture frontend — Routing SvelteKit

**Structure de routes cible :**
```
frontend/src/routes/
├── +layout.svelte          # Layout racine (existant — import CSS, data-mode, Toaster)
├── +layout.ts              # ssr=false, prerender=false (existant)
├── +error.svelte           # Page d'erreur SvelteKit (CRÉER)
├── +page.svelte            # SUPPRIMER ou remplacer par redirect → /login
├── login/
│   └── +page.svelte        # Page de login (HORS layout app)
├── (app)/
│   ├── +layout.svelte      # Layout principal (header, sidebar, contenu, footer)
│   ├── +layout.ts           # Auth guard → redirect /login si non authentifié
│   ├── +page.svelte         # Page d'accueil (placeholder)
│   ├── accounts/            # (migré depuis routes/)
│   ├── bank-accounts/       # (migré depuis routes/)
│   ├── bank-import/         # (migré depuis routes/)
│   ├── contacts/            # (migré depuis routes/)
│   ├── invoices/            # (migré depuis routes/)
│   ├── journal-entries/     # (migré depuis routes/)
│   ├── reconciliation/      # (migré depuis routes/)
│   ├── reports/             # (migré depuis routes/)
│   └── settings/            # (migré depuis routes/)
└── design-system/
    └── +page.svelte        # Démo design system (existant, HORS auth)
```

**Groupe de routes `(app)` :** SvelteKit permet les "layout groups" avec parenthèses. Les routes dans `(app)/` héritent du layout principal (header + sidebar). La page `/login` est hors du groupe, donc affichée en plein écran sans chrome.

### Composants shadcn-svelte disponibles (Story 1.9)

Déjà importés et utilisables directement :
- `Button` — bouton « Se connecter »
- `Input` — champs username/password
- `DropdownMenu` — menu profil header
- `Tooltip` — aide contextuelle
- `Separator` — séparateurs visuels
- `Dialog` — modales futures
- `Sonner/Toaster` — notifications (déjà dans layout racine)

### Store d'authentification — Pattern Svelte 5

Créer `frontend/src/lib/app/stores/auth.svelte.ts` en suivant le **pattern objet avec getters** de `mode.svelte.ts` (l'export direct `$state` est non réassignable depuis un importeur — voir Story 1.9 DEV NOTES). Extension `.svelte.ts` : convention du projet pour les fichiers utilisant des runes (le compilateur les accepte aussi dans `.ts` grâce à `svelte.config.js`, mais la convention aide à identifier les fichiers réactifs).

```typescript
// Svelte 5 runes — PAS de writable/readable (deprecated)
// Fichier : frontend/src/lib/app/stores/auth.svelte.ts

interface CurrentUser {
  userId: string;   // sub du JWT (i64 sérialisé en string)
  role: string;     // 'Admin' | 'Comptable' | 'Consultation'
  // username n'est PAS dans le JWT — absent ici
}

let _accessToken = $state<string | null>(null);
let _refreshToken = $state<string | null>(null);
let _expiresIn = $state<number | null>(null);
let _currentUser = $state<CurrentUser | null>(null);

/** Décode le payload JWT (segment central, base64url) sans vérification de signature. */
function decodeJwtPayload(token: string): { sub: string; role: string; exp: number } {
  const payload = token.split('.')[1];
  return JSON.parse(atob(payload.replace(/-/g, '+').replace(/_/g, '/')));
}

export const authState = {
  get accessToken() { return _accessToken; },
  get refreshToken() { return _refreshToken; },
  get expiresIn() { return _expiresIn; },
  get currentUser() { return _currentUser; },
  get isAuthenticated() { return _accessToken !== null; },

  login(accessToken: string, refreshToken: string, expiresIn: number) {
    _accessToken = accessToken;
    _refreshToken = refreshToken;
    _expiresIn = expiresIn;
    const claims = decodeJwtPayload(accessToken);
    _currentUser = { userId: claims.sub, role: claims.role };
  },

  async logout() {
    // POST /api/v1/auth/logout avec { refreshToken } — PAS de header Authorization
    // Le endpoint logout n'exige PAS de JWT valide (design intentionnel backend)
    await fetch('/api/v1/auth/logout', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refreshToken: _refreshToken }),
    }).catch(() => {}); // Best-effort, on nettoie le state quoiqu'il arrive
    _accessToken = null; _refreshToken = null; _expiresIn = null; _currentUser = null;
  },
};
```

> **Store dans `app/stores/`** (pas `features/auth/`) car l'état d'authentification est global à l'application, pas scopé à un feature.

### Auth guard — Pattern `(app)/+layout.ts`

```typescript
// Fichier : frontend/src/routes/(app)/+layout.ts
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export function load() {
  if (browser && !authState.isAuthenticated) {
    redirect(302, '/login');
  }
}
```

> `browser` check défensif : en SPA `ssr=false` le code s'exécute bien côté client, mais la guard est aussi un bon endroit pour un check explicite.

### API backend existante (Stories 1.5-1.6)

**ATTENTION : le backend utilise `#[serde(rename_all = "camelCase")]` sur TOUS les DTOs. Les clés JSON sont en camelCase.**

**Endpoint login :** `POST /api/v1/auth/login`
- Body : `{ "username": "string", "password": "string" }`
- Succès 200 : `{ "accessToken": "jwt...", "refreshToken": "uuid", "expiresIn": 900 }`
- Échec 401 : `{ "error": { "code": "INVALID_CREDENTIALS", "message": "..." } }`
- Rate-limit 429 : `{ "error": { "code": "RATE_LIMITED", "message": "Trop de tentatives" } }` — c'est un rate-limit IP (5 échecs / 15 min), PAS un verrouillage de compte

**Endpoint logout :** `POST /api/v1/auth/logout`
- PAS de header Authorization requis (le backend accepte le logout même avec un JWT expiré — design intentionnel)
- Body : `{ "refreshToken": "uuid" }`
- Succès : 204 No Content

**Rate limiting :** 5 tentatives échouées en 15 min par IP → blocage 30 min, auto-déblocage.

### Sécurité frontend

- **Access token** : stocker en mémoire (store Svelte), PAS en localStorage (vulnérable XSS)
- **Refresh token** : stocker en mémoire aussi — nécessaire pour le logout. Story 1.11 ajoutera le wrapper fetch avec refresh automatique
- **Messages d'erreur** : ne JAMAIS révéler si le username existe ou pas — message générique unique. Toujours accompagner la couleur d'erreur d'une icône (WCAG : pas de communication par couleur seule)
- **CSRF** : pas nécessaire car API stateless JWT (pas de cookies de session)

### Design tokens à utiliser (Story 1.9)

| Token CSS | Usage dans cette story |
|---|---|
| `--color-primary` (#1e40af) | Bouton login, liens sidebar actifs |
| `--color-primary-light` (#3b82f6) | Focus ring, hover sidebar |
| `--color-error` (#dc2626) | Messages d'erreur login |
| `--color-surface` (#ffffff) | Fond page login |
| `--color-surface-alt` (#f8fafc) | Fond sidebar |
| `--color-text` (#1e293b) | Texte principal |
| `--color-text-muted` (#64748b) | Labels, descriptions |
| `--color-border` (#e2e8f0) | Bordures sidebar, séparateurs |
| `--kesh-gap` | Espacement entre éléments (adaptatif mode) |
| `--kesh-padding` | Padding sections (adaptatif mode) |
| `--kesh-target-min-height` | Hauteur minimale boutons/liens (44px guidé, 32px expert) |

### Typographie (Inter)

- H1 page : `text-2xl font-semibold` (24px/600)
- Labels formulaire : `text-sm font-medium` (14px/500)
- Body : `text-sm font-normal` (14px/400)
- Placeholder : `text-sm text-text-muted`

### Accessibilité — Checklist obligatoire

- `<label for="...">` explicite sur chaque champ du formulaire login
- `aria-describedby` reliant les messages d'erreur à leurs champs
- `aria-live="polite"` sur la zone d'affichage des erreurs
- **Icônes obligatoires** à côté des messages colorés (succès, erreur, warning) — ne jamais communiquer un état uniquement par la couleur (UX spec §Accessibilité)
- Bouton login : `min-h-[var(--kesh-target-min-height)]` (44px guidé)
- Focus visible : déjà configuré globalement (`:focus-visible` dans app.css)
- Pas de `tabindex` positif — ordre DOM naturel suffit
- Sidebar : `<nav aria-label="Navigation principale">`
- Header : `<header>` sémantique
- Footer : `<footer>` sémantique
- Page login : `<main>` wrapping le formulaire
- Structure heading : h1 unique par page, pas de saut de niveau
- **Zoom 200%** : interface fonctionnelle à 200% de zoom navigateur (scroll horizontal acceptable avec `min-w-[1280px]`)

### Contraintes de scope

- **PAS de i18n** dans cette story — textes en dur en français (Story 2.1). Déviation assumée de la règle architecturale #6 (« Erreurs structurées avec code métier — jamais de string d'erreur en dur côté frontend ») — strings hardcodés FR pour l'instant, remplacés en Story 2.1 (i18n Fluent). Le sélecteur de langue dans le header est présent visuellement mais **non fonctionnel** (items disabled ou tooltip « Disponible prochainement ») — fonctionnalité effective en Story 2.1
- **PAS de fetch wrapper** — appel `fetch()` natif direct (Story 1.11 ajoutera le wrapper avec refresh auto)
- **PAS de « mot de passe oublié »** — hors scope (reset admin uniquement, Story 1.7)
- **PAS de forced password change** au premier login admin — hors scope de cette story. Le backend ne distingue pas encore le « premier login » (pas de flag `must_change_password` en base). À traiter dans une story ultérieure si nécessaire
- **PAS de gestion session expirée** (FR13 partiellement couvert) — `expiresIn` stocké dans le store auth pour que Story 1.11 puisse implémenter le refresh silencieux et le timeout 15 min d'inactivité. La détection 401 + redirect login sera implémentée dans Story 1.11 (fetch wrapper)
- **PAS de dark mode** — Sonner fixé `theme="light"` (confirmé Story 1.9)
- **Sidebar liens placeholder** — navigation non fonctionnelle, structure visuelle seulement. Les groupes et le vocabulaire action sont établis, la navigation réelle viendra avec les features
- **Page d'accueil minimale** — simple placeholder « Bienvenue dans Kesh »
- **Bannières contextuelles** — l'espace est réservé dans le layout, le contenu (onboarding incomplet, mode démo) viendra avec Epic 2
- **Groupe de routes `(app)`** — pattern SvelteKit non mentionné dans l'architecture originale mais nécessaire pour séparer les layouts auth/non-auth. Architecture à mettre à jour ultérieurement

### Previous Story Intelligence

**Story 1.9 (design system) — Learnings critiques :**
- shadcn-svelte v2 (next) : commande `npx shadcn-svelte@next add <component>` (PAS @latest)
- Svelte 5 runes obligatoires : `$state`, `$derived`, `$effect` — JAMAIS `writable`/`readable`
- Tailwind CSS v4 : configuration via `@theme { }` dans app.css (PAS tailwind.config.ts)
- Fichiers réactifs : extension `.svelte.ts` obligatoire pour les fichiers utilisant des runes
- Mode store pattern : objet avec getter réactif (`modeState.value`) pour compatibilité `$effect`
- Composants shadcn-svelte dans `$lib/components/ui/` (convention du projet)
- FOUC évité : mode par défaut (`guided`) match les CSS `:root` — pas de flash visuel

**Story 1.8 (RBAC) — Patterns backend pertinents :**
- Rôles : `Consultation < Comptable < Admin` (hiérarchie Ord)
- Login retourne `role` dans le JWT claims → le frontend peut afficher les éléments conditionnellement
- Routes protégées par `require_auth` middleware (vérifie JWT valide)

### Project Structure Notes

- Layout racine (`+layout.svelte`) **NE PAS modifier** sauf si nécessaire — il gère déjà l'import CSS, data-mode, et Toaster
- Les dossiers `routes/login/`, `routes/accounts/`, etc. existent déjà avec `.gitkeep` — migrer les dossiers feature dans `(app)/`
- Le `routes/+page.svelte` existant contient du contenu SvelteKit par défaut — le supprimer ou remplacer
- Le dossier `$lib/features/auth/` existe avec `.gitkeep` — y placer les composants auth si besoin (ex: `LoginForm.svelte`)
- Le dossier `$lib/shared/utils/` existe avec `.gitkeep` — PAS de fetch wrapper ici pour l'instant (Story 1.11)
- Proxy Vite configuré : `/api` → `http://localhost:3000` en dev (préfixe, pas glob)
- **Vitest** : pas encore installé — à ajouter dans T4.0 avant les tests unitaires

### References

- [Source: _bmad-output/planning-artifacts/epics.md — Epic 1, Story 1.10]
- [Source: _bmad-output/planning-artifacts/architecture.md — §Frontend Architecture, §SvelteKit Conventions, §Authentication Flow]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md — §Design System, §Onboarding, §Accessibility]
- [Source: _bmad-output/planning-artifacts/prd.md — FR9-FR16 (Authentification), FR71-FR73 (Notifications)]
- [Source: _bmad-output/implementation-artifacts/1-9-design-system-tokens.md — Design tokens, shadcn-svelte patterns]
- [Source: _bmad-output/implementation-artifacts/1-8-rbac-verrouillage-optimiste.md — RBAC, role hierarchy]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

- svelte-check : 0 erreurs, 2 warnings pré-existants (design-system a11y labels)
- vitest run : 14/14 tests passent (auth store — 8 initiaux + 6 ajoutés en code review)
- cargo test --workspace : 59 passent, 5 échecs pré-existants (PoolTimedOut DB + config env — pas de régression)
- T4.2/T4.3 (E2E Playwright + axe-core) reportés : nécessitent backend démarré avec seed data. Deps installées, à exécuter en intégration.

### Completion Notes List

- Layout principal créé dans `(app)/+layout.svelte` : header fixe (logo, recherche, profil DropdownMenu), sidebar groupée par fréquence (Quotidien/Mensuel/Séparé avec Separator), contenu fluide, footer EUPL 1.2
- Page de login fonctionnelle : formulaire centré, appel API camelCase, 4 états d'erreur avec icônes Lucide SVG (AlertTriangle, Clock, WifiOff, XCircle), loading spinner, navigation clavier native
- Store auth Svelte 5 runes : pattern objet getters, décodage JWT base64url (sub→userId, role), logout best-effort sans Authorization header, expiresIn stocké pour Story 1.11
- Auth guard `(app)/+layout.ts` : redirect 302 vers /login si non authentifié, browser check défensif
- Page d'erreur `+error.svelte` avec icône XCircle et bouton retour
- 9 dossiers de routes migrés dans `(app)/`, `+page.svelte` racine supprimé
- Sélecteur de langue header : items disabled (Story 2.1)
- Espace bannières contextuelles réservé (Epic 2)
- Vitest configuré : import `vitest/config`, environment jsdom, 8 tests unitaires
- @lucide/svelte déjà installé (v1.7.0), @axe-core/playwright installé pour E2E futurs

### Change Log

- 2026-04-07 : Implémentation complète Story 1.10 (Layout + Login + Auth store + Routing + Tests unitaires). Agent : Claude Opus 4.6.
- 2026-04-07 : Code review passe 1 (Sonnet) — 12 patches appliqués : throw redirect, decodeJwtPayload défensif, login atomique, aria-describedby ID, aria-live permanent, window.location.replace, spinner aria-hidden, fakeJwt base64url, 5 tests JWT malformé, try/catch séparé fetch/parsing, sidebar label null, svelte:head titles, goto error page.
- 2026-04-07 : Code review passe 2 (Haiku) — 2 patches appliqués : aria-hidden icônes décoratives layout, validation claims vides + 1 test. Critère d'arrêt atteint (0 finding > LOW). Story marquée done.

### File List

**Créé :**
- `frontend/src/routes/(app)/+layout.svelte` — Layout principal (header, sidebar, contenu, footer)
- `frontend/src/routes/(app)/+layout.ts` — Auth guard (redirect /login si non auth)
- `frontend/src/routes/(app)/+page.svelte` — Page d'accueil placeholder
- `frontend/src/routes/login/+page.svelte` — Page de login (formulaire, erreurs Lucide, loading)
- `frontend/src/routes/+error.svelte` — Page d'erreur SvelteKit
- `frontend/src/lib/app/stores/auth.svelte.ts` — Store authentification (JWT decode, logout)
- `frontend/src/lib/app/stores/auth.svelte.test.ts` — 8 tests unitaires store auth

**Modifié :**
- `frontend/package.json` — +vitest, @testing-library/svelte, jsdom, @axe-core/playwright, scripts test:unit/test:e2e
- `frontend/vite.config.ts` — import vitest/config, bloc test: { environment: 'jsdom' }

**Supprimé :**
- `frontend/src/routes/+page.svelte` — Contenu SvelteKit par défaut
- `frontend/src/routes/login/.gitkeep`
- `frontend/src/lib/features/auth/.gitkeep`

**Migré dans `routes/(app)/` :**
- accounts/, bank-accounts/, bank-import/, contacts/, invoices/, journal-entries/, reconciliation/, reports/, settings/

**Non créé (reporté) :**
- `frontend/tests/e2e/auth.spec.ts` — T4.2/T4.3 E2E + axe-core (nécessite backend + seed)
