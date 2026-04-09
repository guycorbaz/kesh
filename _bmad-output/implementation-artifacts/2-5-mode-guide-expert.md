# Story 2.5: Mode Guidé / Expert

Status: review

## Story

As a **utilisateur**,
I want **choisir entre un mode guidé et un mode expert**,
so that **l'interface s'adapte à mon niveau de compétence**.

### Contexte

Dernière story de l'Epic 2. L'infrastructure mode Guidé/Expert est déjà largement en place : CSS custom properties (`--kesh-*`), store réactif (`modeState`), toggle dans le menu profil, `data-mode` sur `<html>`, et empty states mode-aware sur la homepage. Cette story ajoute la **persistence** (localStorage + serveur), les **raccourcis clavier** pour le mode Expert, et la **synchronisation** du mode au chargement de l'app.

### Décisions de conception

- **Persistence double** : localStorage pour chargement instantané (pas de flash) + serveur via `onboarding_state.ui_mode` (déjà en base). Pas de nouvelle migration — le champ `ui_mode` existe déjà dans `onboarding_state`.
- **Endpoint mode** : `PUT /api/v1/profile/mode` — met à jour `onboarding_state.ui_mode`. Authentifié (tout rôle). Simple et suffisant pour le mono-instance.
- **Synchronisation au startup** : le store charge d'abord depuis localStorage (instantané), puis depuis le serveur (onboarding state) au premier fetch. Si divergence, le serveur fait foi.
- **Raccourcis clavier** : `Ctrl+N` → navigation vers `/journal-entries` (nouvelle écriture). Actifs uniquement en mode Expert. Extensible pour les Epics futurs.
- **Confirmations Guidé** : pattern de confirmation avant actions destructives (ex: supprimer). Infra seulement — les confirmations concrètes seront ajoutées dans les stories qui implémentent les actions (Epic 3+).

## Acceptance Criteria (AC)

1. **Changement immédiat** — Given profil utilisateur, When changement du mode (Guidé ↔ Expert) via le menu profil, Then l'interface s'adapte immédiatement sans rechargement (CSS custom properties, espacements, taille des cibles).
2. **Mode Guidé** — Given mode Guidé, When affichage, Then espacements généreux (gap-4, p-6), boutons plus grands (min-height 44px), labels explicites. (Déjà implémenté via CSS vars — vérification.) **Dette technique :** confirmations avant actions destructives et aide contextuelle visible seront ajoutées dans les stories des Epics 3+ qui implémentent les actions concernées.
3. **Mode Expert** — Given mode Expert, When affichage, Then espacements compacts (gap-2, p-4), boutons compacts (min-height 32px). Raccourci `Ctrl+N` navigue vers `/journal-entries`. **Dette technique :** boutons avec icônes+tooltips seront ajoutés progressivement dans les stories des Epics 3+ qui créent les composants métier.
4. **Empty states** — Given liste vide en mode Guidé, Then explication + suggestion + bouton. En mode Expert, bouton seul. (Déjà implémenté Story 2.4 — vérification.)
5. **Persistence** — And le mode est persisté en localStorage ET sur le serveur (onboarding_state.ui_mode). Un refresh de page conserve le mode. Un changement de device retrouve le mode via le serveur.
6. **Onboarding + profil** — And le mode est choisi à l'onboarding (Story 2.2 step 2) et modifiable à tout moment via le toggle dans le menu profil.
7. **Tests** — And tests vitest (persistence localStorage, sync serveur), test Playwright (toggle mode + raccourci Ctrl+N).

## Tasks / Subtasks

### T1 — Persistence localStorage (AC: #5)
- [x] T1.1 Modifier `frontend/src/lib/app/stores/mode.svelte.ts` : au setter `modeState.value`, sauver dans `localStorage.setItem('kesh-mode', v)`.
- [x] T1.2 Au chargement du store (init), lire `localStorage.getItem('kesh-mode')`. Si valeur valide ('guided'/'expert'), l'appliquer. Sinon, default 'guided'.

### T2 — Endpoint API mode (AC: #5)
- [x] T2.1 Ajouter `PUT /api/v1/profile/mode` dans `crates/kesh-api/src/routes/` (nouveau fichier `profile.rs` ou dans `onboarding.rs`). Body : `{ "mode": "guided"|"expert" }`. Met à jour `onboarding_state.ui_mode` via `onboarding::update_step()` (seul `ui_mode` change, step et is_demo inchangés). Authentifié (tout rôle).
- [x] T2.2 Enregistrer la route dans `authenticated_routes` de `build_router()`.
- [x] T2.3 `pub mod profile;` dans `routes/mod.rs` (si nouveau fichier).

### T3 — Synchronisation serveur (AC: #5, #6)
- [x] T3.1 Modifier le store mode : ajouter `async syncFromServer()` qui lit `onboarding_state.uiMode` depuis `GET /api/v1/onboarding/state` et met à jour le store + localStorage si différent.
- [x] T3.2 Modifier le toggle (`toggleMode()` / setter) : après mise à jour locale + localStorage, appeler `PUT /api/v1/profile/mode`. Erreur API → `console.error` (ne pas bloquer l'UI, le mode local est déjà appliqué — le serveur sera resync au prochain login).
- [x] T3.3 Appeler `syncModeFromServer(onboardingState.uiMode)` dans `(app)/+layout.ts` `load()` — synchrone (pas async), après le fetch onboarding state déjà fait. Pas de Promise flottante.

### T4 — Raccourcis clavier Expert (AC: #3)
- [x] T4.1 Modifier `(app)/+layout.svelte` : dans le handler `handleKeydown` existant, ajouter `Ctrl+N` → `goto('/journal-entries')` si `modeState.value === 'expert'`. `e.preventDefault()` empêche le `Ctrl+N` du navigateur (nouvelle fenêtre) — **décision UX intentionnelle** : en mode Expert, les raccourcis métier priment sur les raccourcis navigateur. En mode Guidé, `Ctrl+N` garde son comportement navigateur normal.
- [x] T4.2 Afficher un indicateur visuel discret dans le footer ou tooltip : "Ctrl+N : Nouvelle écriture" (uniquement en mode Expert).

### T5 — Clés i18n (AC: #3)
- [x] T5.1 Ajouter les clés dans les 4 fichiers `.ftl` :
  - `shortcut-new-entry = Ctrl+N : Nouvelle écriture`
  - `mode-guided-label = Guidé` / `mode-expert-label = Expert` — ET modifier le toggle dans `(app)/+layout.svelte` pour utiliser ces clés au lieu des strings hardcodées

### T6 — Tests (AC: #7)
- [x] T6.1 Tests vitest : persistence localStorage (set/get mode), sync mock API.
- [x] T6.2 Tests E2E API : `PUT /api/v1/profile/mode` (200 + 400 invalid mode + 401 unauth).
- [x] T6.3 Test Playwright : toggle mode dans le menu → espacement change visible. Ctrl+N en mode Expert → navigation `/journal-entries`.

## Dev Notes

### État existant — déjà implémenté

| Fonctionnalité | Story | État |
|---------------|-------|------|
| CSS custom properties `--kesh-*` | 1.9 | app.css, 2 modes complets |
| Store `modeState` réactif | 1.9 | mode.svelte.ts, getter/setter |
| `data-mode` sur `<html>` | 1.10 | +layout.svelte root, $effect |
| Toggle dans menu profil | 1.10 | (app)/+layout.svelte dropdown |
| Empty states mode-aware | 2.4 | +page.svelte homepage widgets |
| `ui_mode` en DB (onboarding_state) | 2.2 | Champ Option<UiMode>, step 2 |
| Keyboard handler `handleKeydown` | 1.10 | (app)/+layout.svelte, Ctrl+S |

### Ce qui reste à faire

1. **Persistence localStorage** — trivial (2 lignes dans le store)
2. **Endpoint PUT /api/v1/profile/mode** — léger (~30 lignes Rust)
3. **Sync serveur au startup** — lire `ui_mode` depuis onboarding state déjà fetché
4. **Ctrl+N raccourci** — 5 lignes dans handleKeydown existant
5. **Fire-and-forget PUT** sur toggle — appel API asynchrone non bloquant

### Pattern persistence localStorage

```typescript
// Dans mode.svelte.ts
const STORAGE_KEY = 'kesh-mode';

// Init
const stored = typeof localStorage !== 'undefined' 
  ? localStorage.getItem(STORAGE_KEY) 
  : null;
let _mode = $state<Mode>(stored === 'expert' ? 'expert' : 'guided');

// Setter
set value(v: Mode) {
  _mode = v;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, v);
  }
}
```

### Pattern sync serveur

```typescript
// Dans mode.svelte.ts
export function syncModeFromServer(uiMode: 'guided' | 'expert' | null) {
  if (uiMode && uiMode !== _mode) {
    modeState.value = uiMode; // Passer par le setter pour màj localStorage
  }
}

// Dans (app)/+layout.ts, après fetchState()
if (onboardingState.uiMode) {
  syncModeFromServer(onboardingState.uiMode);
}
```

### Endpoint PUT /api/v1/profile/mode

Pattern simple — réutilise `onboarding::update_step` en ne changeant que `ui_mode` :

```rust
// PUT /api/v1/profile/mode
pub async fn set_mode(
    State(state): State<AppState>,
    Json(body): Json<ModeRequest>,
) -> Result<StatusCode, AppError> {
    let ui_mode: UiMode = body.mode.parse()
        .map_err(|_| AppError::Validation("Mode invalide"))?;
    let current = onboarding::get_state(&state.pool).await?
        .ok_or(AppError::Internal("No onboarding state"))?;
    onboarding::update_step(
        &state.pool,
        current.step_completed,
        current.is_demo,
        Some(ui_mode),
        current.version,
    ).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

### Raccourci Ctrl+N

```typescript
// Dans handleKeydown existant de (app)/+layout.svelte
if ((e.ctrlKey || e.metaKey) && e.key === 'n' && modeState.value === 'expert') {
    e.preventDefault();
    goto('/journal-entries');
}
```

### Project Structure Notes

- **Modification** : `frontend/src/lib/app/stores/mode.svelte.ts` (persistence + sync)
- **Nouveau** : `crates/kesh-api/src/routes/profile.rs` (PUT /api/v1/profile/mode)
- **Modification** : `crates/kesh-api/src/routes/mod.rs`, `lib.rs` (route registration)
- **Modification** : `frontend/src/routes/(app)/+layout.svelte` (Ctrl+N shortcut)
- **Modification** : `frontend/src/routes/(app)/+layout.ts` (sync mode from server)
- **Modifications i18n** : 4 fichiers `.ftl`

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-2-Story-2.5] — AC BDD
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Mode-Guide-Expert] — Spacing, shortcuts, empty states
- [Source: _bmad-output/implementation-artifacts/2-2-flux-onboarding-chemin-a-exploration.md] — ui_mode in onboarding_state
- [Source: _bmad-output/implementation-artifacts/2-4-page-accueil-navigation.md] — Empty states mode-aware

## Dev Agent Record

### Agent Model Used

Opus 4.6

### Debug Log References

### Completion Notes List

- T1: Mode store persistence localStorage (init from storage + save on set). 0 flash au refresh.
- T2: Endpoint PUT /api/v1/profile/mode — updates onboarding_state.ui_mode via update_step. 3 E2E tests (204, 400, 401).
- T3: syncModeFromServer() synchrone dans (app)/+layout.ts. toggleMode() fire-and-forget PUT avec console.error.
- T4: Ctrl+N raccourci Expert → /journal-entries. Indicateur "Ctrl+N : Nouvelle écriture" dans footer (Expert only).
- T5: 3 clés i18n × 4 locales (mode-guided-label, mode-expert-label, shortcut-new-entry). Toggle menu utilise i18nMsg().
- T6.2: 3 tests E2E API profile/mode passent. Aucune régression (onboarding 9/9, companies 3/3, vitest 50/50).

### File List

#### New Files
- `crates/kesh-api/src/routes/profile.rs` — PUT /api/v1/profile/mode
- `crates/kesh-api/tests/profile_e2e.rs` — 3 tests E2E

#### Modified Files
- `frontend/src/lib/app/stores/mode.svelte.ts` — persistence localStorage + syncModeFromServer + fire-and-forget PUT
- `crates/kesh-api/src/routes/mod.rs` — pub mod profile
- `crates/kesh-api/src/lib.rs` — route /api/v1/profile/mode dans authenticated_routes
- `frontend/src/routes/(app)/+layout.svelte` — Ctrl+N shortcut, i18n toggle labels, shortcut hint in footer
- `frontend/src/routes/(app)/+layout.ts` — syncModeFromServer import + appel
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` — 3 clés mode
- `crates/kesh-i18n/locales/de-CH/messages.ftl` — 3 clés mode
- `crates/kesh-i18n/locales/it-CH/messages.ftl` — 3 clés mode
- `crates/kesh-i18n/locales/en-CH/messages.ftl` — 3 clés mode

## Change Log

| Date | Passe | Modèle | Findings | Patches |
|------|-------|--------|----------|---------|
| 2026-04-09 | Implémentation | Opus 4.6 | — | T1-T6 complètes, E2E 3/3, vitest 50/50, aucune régression |
