# Story 1.9 : Design system & tokens

Status: done

## Story

As a **développeur frontend**,
I want **un design system Kesh configuré avec les tokens visuels**,
so that **tous les composants partagent une identité visuelle cohérente**.

### Décisions de conception

- **Tailwind CSS v4 — configuration CSS** : Tailwind v4 utilise `@theme` dans le CSS (pas tailwind.config.ts). Les tokens sont définis dans `app.css` via `@theme { }`. Les variables suivent les namespaces Tailwind v4 : `--color-*` pour couleurs, `--font-*` pour polices, `--font-size-*` pour tailles.
- **Mode Guidé / Expert via CSS custom properties** : un attribut `data-mode="guided"` ou `data-mode="expert"` sur `<html>` contrôle les espacements via CSS custom properties. Un état réactif Svelte 5 (`$state` dans un fichier `.svelte.ts`) gère la valeur. Le mode par défaut est "guided" (pas de flash — c'est le défaut CSS).
- **shadcn-svelte v2 (next)** : le `components.json` existant pointe vers `next.shadcn-svelte.com`. La commande CLI correcte est `npx shadcn-svelte@next add <component>` (PAS `@latest` qui pointe vers la v1 stable incompatible). Les composants sont copiés dans `$lib/components/ui/`.
- **Inter via @fontsource/inter** : la UX spec exige "police système, pas de chargement externe". `@fontsource/inter` est un package npm qui bundle la police Inter en self-hosted (pas de requête Google Fonts). Conforme à la spec UX et adapté au contexte comptable (offline-ready).
- **Tests unitaires de composants** : vérifier que les tokens sont appliqués et que le mode Guidé/Expert fonctionne. Tests basiques avec Vitest pour valider la configuration (pas de tests visuels exhaustifs — ceux-ci viendront avec Story 1.10).
- **Svelte 5 runes obligatoire** : `svelte.config.js` active `runes: true` globalement. Tout code réactif doit utiliser `$state`, `$derived`, `$effect` — PAS `writable`/`readable` de `svelte/store` (Svelte 4 API dépréciée en mode runes).
- **Couleurs étendues depuis la UX spec** : l'AC#1 inclut des tokens au-delà de ceux listés dans l'epic (info, fond, texte, bordures). Ces tokens proviennent de la UX spec §Système de Couleurs et sont nécessaires pour un design system fonctionnel.

## Acceptance Criteria (AC)

1. **Palette de couleurs** — Given Tailwind CSS + shadcn-svelte configurés, When inspection des composants, Then les design tokens Kesh sont appliqués : bleu ardoise `#1e40af` (primaire), bleu clair `#3b82f6` (primaire clair/focus), vert `#16a34a` (succès), rouge `#dc2626` (erreur), ambre `#d97706` (attention), info `#0ea5e9`, fond `#ffffff`/`#f8fafc`, texte `#1e293b`/`#64748b`, bordures `#e2e8f0`. [Source: UX spec §Système de Couleurs]
2. **Typographie** — Given typographie configurée, When inspection, Then Inter (self-hosted via @fontsource) est la police unique avec `font-variant-numeric: tabular-nums` sur les montants (utiliser la classe Tailwind `tabular-nums`).
3. **Espacements mode Guidé** — Given mode Guidé actif (défaut), When inspection des espacements, Then `gap-4` (16px), `p-6` (24px), `my-8` (32px), hauteur ligne tableau 48px.
4. **Espacements mode Expert** — Given mode Expert actif (`data-mode="expert"` sur `<html>`), When inspection des espacements, Then `gap-2` (8px), `p-4` (16px), `my-4` (16px), hauteur ligne tableau 36px.
5. **Composants de base** — Given shadcn-svelte v2, When vérification, Then les composants Button, Input, Select, Table, Dialog, Toast, Tooltip, DropdownMenu sont importés et fonctionnels avec le thème Kesh.

## Tasks / Subtasks

### T1 — Tokens de couleur et typographie dans Tailwind CSS v4 (AC: #1, #2)
- [x] T1.1 Installé `@fontsource/inter` (v5.2.8)
- [x] T1.2 Configurer `frontend/src/app.css` — ajouter **après** l'import Tailwind :
  ```css
  @import '@fontsource/inter/400.css';
  @import '@fontsource/inter/500.css';
  @import '@fontsource/inter/600.css';

  @theme {
    --color-primary: #1e40af;
    --color-primary-light: #3b82f6;
    --color-success: #16a34a;
    --color-error: #dc2626;
    --color-warning: #d97706;
    --color-info: #0ea5e9;
    --color-surface: #ffffff;
    --color-surface-alt: #f8fafc;
    --color-text: #1e293b;
    --color-text-muted: #64748b;
    --color-border: #e2e8f0;
    --font-sans: 'Inter', system-ui, sans-serif;
  }
  ```
- [x] T1.3 Ajouter les styles globaux dans `app.css` (après @theme) :
  ```css
  body {
    font-family: var(--font-sans);
    color: var(--color-text);
    background-color: var(--color-surface);
  }
  ```
- [x] T1.4 Vérifier que les classes Tailwind générées fonctionnent : `text-primary`, `bg-success`, `border-border`, `font-sans`, `tabular-nums`
- [x] T1.5 Configurer les tailles de texte dans @theme si nécessaire (sinon utiliser les classes Tailwind par défaut : `text-sm` 14px, `text-base` 16px, `text-lg` 20px, `text-xl` 24px)

### T2 — Système d'espacement Guidé / Expert (AC: #3, #4)
- [x] T2.1 Définir les CSS custom properties de mode dans `app.css` :
  ```css
  :root, [data-mode="guided"] {
    --kesh-gap: 1rem;
    --kesh-padding: 1.5rem;
    --kesh-section-margin: 2rem;
    --kesh-table-row-height: 48px;
    --kesh-target-min-height: 44px;
  }
  [data-mode="expert"] {
    --kesh-gap: 0.5rem;
    --kesh-padding: 1rem;
    --kesh-section-margin: 1rem;
    --kesh-table-row-height: 36px;
    --kesh-target-min-height: 32px;
  }
  ```
- [x] T2.2 Créer le state réactif Svelte 5 `frontend/src/lib/app/stores/mode.svelte.ts` :
  ```ts
  export type Mode = 'guided' | 'expert';
  export let mode = $state<Mode>('guided');
  ```
  Note : fichier `.svelte.ts` (pas `.ts`) pour que les runes `$state` fonctionnent.
- [x] T2.3 Dans `+layout.svelte`, appliquer `data-mode` sur `<html>` via `$effect` :
  ```svelte
  <script>
    import { mode } from '$lib/app/stores/mode.svelte';
    $effect(() => {
      document.documentElement.setAttribute('data-mode', mode);
    });
  </script>
  ```
  Note : `document.documentElement` est safe car `ssr = false` dans `+layout.ts`. Pas de flash (FOUC) car le mode par défaut ("guided") correspond au CSS `:root` — le `$effect` ne change rien au premier rendu.

### T3 — Import des composants shadcn-svelte v2 (AC: #5)
- [x] T3.1 Importer les composants via le CLI shadcn-svelte **v2 (next)** :
  ```bash
  cd frontend
  npx shadcn-svelte@next add button input select table dialog
  npx shadcn-svelte@next add toast tooltip dropdown-menu
  ```
  Vérifier la commande exacte dans la doc `next.shadcn-svelte.com` si `@next` ne fonctionne pas — le CLI peut aussi être `npx shadcn@latest add` (CLI unifié).
- [x] T3.2 Vérifier que les composants sont dans `frontend/src/lib/components/ui/`
- [x] T3.3 Personnaliser le thème shadcn pour utiliser les tokens Kesh (adapter les CSS variables dans les composants si nécessaire)
- [x] T3.4 Créer une page de démonstration `frontend/src/routes/design-system/+page.svelte` montrant :
  - Tous les boutons (primary, secondary, destructive, outline, ghost)
  - Input + Select + labels
  - Table avec données de test (montants avec `tabular-nums`)
  - Dialog + Toast + Tooltip + DropdownMenu
  - Toggle Guidé/Expert montrant le changement d'espacement

### T4 — Accessibilité de base (AC: #1, #2)
- [x] T4.1 Configurer le focus ring accessible dans `app.css` :
  ```css
  :focus-visible {
    outline: 2px solid var(--color-primary-light);
    outline-offset: 2px;
  }
  ```
- [x] T4.2 Vérifier les contrastes WCAG AA : texte principal sur fond blanc ≥ 4.5:1, boutons primaires ≥ 4.5:1

### T5 — Vérification et non-régression (AC: #1-#5)
- [x] T5.1 Exécuter `npm run build` pour vérifier que le frontend compile
- [x] T5.2 Exécuter `npm run check` (svelte-check) pour vérifier le typage
- [x] T5.3 Vérifier visuellement la page design-system (T3.4) : couleurs, typographie, espacements guidé/expert, composants
- [x] T5.4 Vérifier que le `cn.ts` existant (`$lib/utils/cn.ts`) fonctionne avec les nouvelles classes

## Dev Notes

### Tailwind CSS v4 — namespaces de variables

Les variables `@theme` en Tailwind v4 suivent des namespaces stricts :
- **Couleurs** : `--color-*` → génère `text-primary`, `bg-success`, `border-border`, etc.
- **Polices** : `--font-*` → génère `font-sans`, `font-mono`, etc. (ex: `--font-sans` → classe `font-sans`)
- **Tailles de texte** : `--font-size-*` → génère `text-sm`, `text-base`, etc.

La classe Tailwind `tabular-nums` existe déjà nativement (génère `font-variant-numeric: tabular-nums`). Pas besoin de créer une classe CSS manuelle.

### Svelte 5 runes — PAS de writable/readable

Le projet a `runes: true` dans `svelte.config.js`. Utiliser :
- `$state()` au lieu de `writable()`
- `$derived()` au lieu de `derived()`
- `$effect()` au lieu de `onMount` + souscription

Les fichiers contenant des runes doivent avoir l'extension `.svelte.ts` (pas `.ts`).

### shadcn-svelte v2 (next) — commande CLI

Le `components.json` existant utilise le registre `next.shadcn-svelte.com` (v2). La commande correcte est :
```bash
npx shadcn-svelte@next add <component>
```
Si `@next` ne résout pas, essayer :
```bash
npx shadcn@latest add <component>
```
Le CLI unifié `shadcn` détecte automatiquement le framework (Svelte) depuis `components.json`.

### Inter self-hosted via @fontsource

`@fontsource/inter` est un package npm qui contient les fichiers de police Inter. L'import dans `app.css` :
```css
@import '@fontsource/inter/400.css';
@import '@fontsource/inter/500.css';
@import '@fontsource/inter/600.css';
```

Pas de requête réseau vers Google Fonts. Conforme à la UX spec ("police système, pas de chargement externe") et adapté au contexte comptable (fonctionnement offline possible).

### Mode par défaut et FOUC

Le mode par défaut est "guided" — les CSS custom properties sont définies sur `:root` (qui est `<html>`). Le `$effect` dans `+layout.svelte` ne s'exécute qu'après le montage, mais comme le mode par défaut est "guided" et que les valeurs CSS `:root` correspondent, il n'y a **pas de flash** (FOUC).

Si un utilisateur a sauvegardé "expert" (futur — Story 2.5 ajoutera la persistance localStorage), un flash sera visible au premier rendu. Ce sera adressé en Story 2.5 avec un `<script>` bloquant dans `app.html`.

### Fichiers existants à connaître

- `frontend/src/app.css` — actuellement `@import 'tailwindcss';` seulement → T1 enrichit ce fichier
- `frontend/src/routes/+layout.svelte` — importe `app.css` → T2.3 ajoute le binding data-mode
- `frontend/src/routes/+layout.ts` — exporte `ssr = false`, `prerender = false` → pas de modification
- `frontend/src/lib/utils/cn.ts` — utilitaire `cn()` (clsx + twMerge) déjà en place
- `frontend/components.json` — config shadcn-svelte v2 (next) déjà en place → pas de modification
- `frontend/svelte.config.js` — `runes: true` activé → impose `$state`/`$effect`
- `frontend/vite.config.ts` — plugin `@tailwindcss/vite` configuré → pas de modification

### Fichiers à créer / modifier

**Nouveaux fichiers :**
- `frontend/src/lib/app/stores/mode.svelte.ts` — state réactif mode Guidé/Expert
- `frontend/src/routes/design-system/+page.svelte` — page démo design system
- `frontend/src/lib/components/ui/*` — composants shadcn-svelte (générés par CLI)

**Fichiers à modifier :**
- `frontend/src/app.css` — ajouter @fontsource imports, @theme tokens, custom properties mode, focus ring, body styles
- `frontend/src/routes/+layout.svelte` — ajouter $effect pour data-mode sur `<html>`
- `frontend/package.json` — ajout dépendance `@fontsource/inter` (via npm install)

### Dépendances

**Nouvelle dépendance npm :**
- `@fontsource/inter` — police Inter self-hosted

**Dépendances existantes (Story 1.1) :**
- `tailwindcss@4.2.2`, `@tailwindcss/vite`, `bits-ui@2.16.5`, `clsx`, `tailwind-merge`, `tailwind-variants`

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-1, Story 1.9] — AC et user story
- [Source: _bmad-output/planning-artifacts/architecture.md#Frontend] — SvelteKit, Tailwind, shadcn-svelte stack
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Couleurs,Typographie,Espacements] — Palette couleurs, typographie, espacements, accessibilité
- [Source: frontend/package.json] — Dépendances installées (Tailwind 4.2.2, bits-ui 2.16.5, Svelte 5.54)
- [Source: frontend/components.json] — Config shadcn-svelte v2 (registre next.shadcn-svelte.com)
- [Source: frontend/svelte.config.js] — runes: true (Svelte 5 obligatoire)
- [Source: frontend/src/app.css] — Actuellement `@import 'tailwindcss'` seulement
- [Source: frontend/src/lib/utils/cn.ts] — Utilitaire cn() existant (clsx + twMerge)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

- shadcn-svelte `@next` CLI non fonctionnel (registre renvoie HTML) → fallback sur `@latest` (v1.2.7) qui fonctionne
- `toast` n'existe pas dans le registre nova → utiliser `sonner` (le composant toast shadcn-svelte)
- `$state` exporté ne peut pas être réassigné depuis un importeur → pattern getter/setter/toggle
- Composants shadcn importent `$lib/utils.js` → créé barrel `src/lib/utils.ts` avec types manquants

### Completion Notes List

- T1 : @fontsource/inter installé, @theme configuré avec 11 couleurs + Inter font-sans
- T2 : Mode Guidé/Expert via CSS custom properties + store Svelte 5 (getter/setter pattern)
- T3 : 9 composants shadcn-svelte importés (button, input, select, table, dialog, sonner, tooltip, dropdown-menu, separator) + page démo /design-system
- T4 : Focus ring :focus-visible configuré avec --color-primary-light
- T5 : Build OK, svelte-check 0 erreurs, backend 69 tests passent

### Change Log

- 2026-04-06 : Story 1.9 implémentée — design tokens Kesh, typographie Inter, mode Guidé/Expert, 9 composants shadcn-svelte, page démo
- 2026-04-06 : Code review passe #1 (Sonnet) — 6 patches :
  - CRITICAL-01 : Sonner mode-watcher supprimé → theme="light" hardcodé (pas de dark mode)
  - HIGH-01 : mode store réactivité corrigée → modeState objet avec getter réactif pour $effect
  - MEDIUM-01 : --color-popover + --color-popover-foreground ajoutés pour Sonner
  - MEDIUM-02 : WithoutChildren ajouté proactivement dans utils.ts
  - LOW-01 : Sonner/Toast ajouté à la page démo avec 3 boutons
  - LOW-02 : tabular-nums retiré des Table.Head (headers texte, pas montants)

### File List

**Nouveaux fichiers :**
- frontend/src/lib/app/stores/mode.svelte.ts
- frontend/src/lib/utils.ts (barrel pour shadcn-svelte)
- frontend/src/routes/design-system/+page.svelte
- frontend/src/lib/components/ui/* (9 composants shadcn-svelte)

**Fichiers modifiés :**
- frontend/src/app.css (tokens, typographie, espacements, focus ring)
- frontend/src/routes/+layout.svelte (data-mode binding)
- frontend/package.json (@fontsource/inter + dépendances shadcn)
- frontend/package-lock.json
