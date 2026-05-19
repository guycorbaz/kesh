# Story 9.5-1c: Fix a11y violations — KF #91 layout DropdownMenu + KF #55 axe-core 6 pages

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a mainteneur projet Kesh,
I want fixer la violation `nested-interactive` wcag2a 4.1.2 dans le composant app-shell `+layout.svelte` (KF #91), mesurer empiriquement la cascade de réduction de violations qu'elle apporte sur les **6 tests axe-core** distribués sur **5 spec files** (`auth.spec.ts` qui contient 2 tests `:90` login + `:98` layout principal + `contacts.spec.ts:150` + `homepage-settings.spec.ts:61` + `invoices.spec.ts:87` + `products.spec.ts:78`) couvrant les routes KF #55 (login + layout principal + contacts + homepage + invoices + products), catégoriser les violations résiduelles par root cause (color-contrast / alt-text / ARIA / landmarks / heading-order / focus-management / nested-interactive autres), puis appliquer la **règle R2 du parent epic-9-5.md** : si cumul violations résiduelles > 100 → splitter en sous-stories 9-5-1c-quick (fixes mécaniques) + 9-5-1c-structural (refactor architecture a11y) ; sinon → fixer en-story,
so that les 6 pages testées en axe-core ne présentent plus les violations KF #55 + KF #91 : la violation `nested-interactive` du DropdownMenu profile (1 occurrence × 6 pages `(app)/*` cascadées, **page login `/login` non affectée car elle utilise `routes/login/+page.svelte` sans le layout app-shell**) est éliminée par fix Phase A, la conformité WCAG 2 niveau A est restaurée sur les flux critiques v0.1, le pattern bits-ui `child` snippet (ou classes Tailwind directes sur `<DropdownMenu.Trigger>`) est documenté pour les futurs wrappers DropdownMenu/Sheet/Dialog, et les KFs #55 + #91 sont fermées (ou transférées vers des sous-stories spécifiques 9-5-1c-quick + 9-5-1c-structural avec scope précis si R2 déclenché Phase C).

## Scope

Story d'implémentation **a11y scopée app-shell + audit empirique**. Périmètre précis :

- **Composant app-shell** : `frontend/src/routes/(app)/+layout.svelte` (1 fichier — lignes 136-143 actuelles : `<DropdownMenu.Trigger><Button variant="ghost">...</Button></DropdownMenu.Trigger>` ; le `<DropdownMenu.Root>` ligne 135 + `</DropdownMenu.Trigger>` ligne 144 ne sont pas touchés). **Note ground-truth** : `bits-ui 2.16.5` confirmé installé (`package.json`), `child` snippet API disponible (`MenuTriggerPropsWithoutHTML = WithChild<{...}>` dans `node_modules/bits-ui/dist/bits/menu/types.d.ts`), et `DropdownMenu.Trigger` accepte `class` prop directement (variant A fonctionnel — `...restProps` forwarded). **Préférence projet** = variante B snippet `child` (cohérent `design-system/+page.svelte:151` — voir Dev Notes §"Pattern bits-ui"). Une seconde occurrence `DropdownMenu.Trigger` dans `frontend/src/routes/design-system/+page.svelte:151` utilise **déjà** le pattern `{#snippet child({ props })}` correct — ne pas la modifier.
- **Tests axe-core à mesurer** (6 KF #55 + 2 KF #91) :
  - `frontend/tests/e2e/auth.spec.ts:90` (page login — 109 violations baseline 2026-04-30)
  - `frontend/tests/e2e/auth.spec.ts:98` (layout principal — 82 violations baseline)
  - `frontend/tests/e2e/contacts.spec.ts:150` (liste contacts — 90 violations empirique 2026-05-19)
  - `frontend/tests/e2e/homepage-settings.spec.ts:61` (page accueil — 82 violations empirique 2026-05-19)
  - `frontend/tests/e2e/invoices.spec.ts:87` (liste factures empty — 49 violations empirique 2026-05-19)
  - `frontend/tests/e2e/products.spec.ts:78` (liste produits — 90 violations empirique 2026-05-19)
  - `frontend/tests/e2e/reports.spec.ts:85` (reports empty — 49 violations empirique 2026-05-19, KF #91 canonique)
  - `frontend/tests/e2e/reports.spec.ts:96` (reports populated — 49 violations empirique 2026-05-19)
- **0 fichier de production applicatif** modifié hors `+layout.svelte` en Phase A. Phase C (implémentation post-R2) peut toucher des composants partagés (Sidebar / Header / Toast / Modal / ContactPicker / etc.) selon catégorisation.

**Cumul violations empirique 2026-05-19** : ~600 violations cumulées sur les 8 tests (109 + 82 + 90 + 82 + 49 + 90 + 49 + 49). **R2 split très probable** dès la phase de mesure.

**Hors scope 9-5-1c (par construction post-9-5-1a triage)** :

- KF #54 (cascade 401) — ✅ fermée 9-5-1b.
- KF #57 (state/timing) — ✅ fermée 9-5-1b, split en KF-028 + KF-029 nouvelles.
- KF #47 (Story 3-7 AC #22 fallback toast) — déférée 9-5-1d.
- KF #50 (AC #29 race REPEATABLE READ) — déférée 9-5-1d.
- Audit a11y exhaustif **autres pages** (journal-entries, mode-expert, onboarding, users, fiscal-years, etc.) — hors scope (les tests axe-core existants couvrent uniquement les 5+1 pages listées).
- Audit a11y avec **outils tiers** (Lighthouse, WAVE) — hors scope (le projet utilise axe-core via Playwright comme source de vérité).
- Migration `bits-ui` vers une autre lib UI — hors scope (le fix KF #91 utilise un pattern bits-ui supporté).

## Acceptance Criteria

### Pré-flight environnement et baseline

1. **Given** un workspace Kesh à jour avec `main` `35344c9` + branche `chore/epic-9-5-planning` checkée, **When** la story démarre, **Then** prérequis confirmés : `cargo build --workspace` clean, `cd frontend && npm install && npm run build` clean, MariaDB démarré + migrations appliquées, Playwright Chromium installé via `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npx playwright install chromium` (memory `reference_playwright_ubuntu26`), backend kesh-api démarré KESH_TEST_MODE=true + KESH_HOST=127.0.0.1 (override .env qui a KESH_HOST=0.0.0.0). Si backend déjà running depuis 9-5-1b → réutiliser. **Note contexte (pas un prerequis actif)** : commit BMAD upgrade `c6f9444` sur branche `chore/bmad-upgrade-6.6.0` en attente de merge via PR #95 — aucune action requise pour 9-5-1c, ce commit ne touche pas le code frontend/Rust applicatif.

2. **Given** la baseline a11y pré-fix, **When** `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 KESH_BACKEND_URL=http://127.0.0.1:3000 npx playwright test auth.spec.ts:90 auth.spec.ts:98 contacts.spec.ts:150 homepage-settings.spec.ts:61 invoices.spec.ts:87 products.spec.ts:78 reports.spec.ts:85 reports.spec.ts:96 --reporter=list 2>&1 | tee tests/e2e/baseline-pre-9-5-1c-a11y.log` est exécuté depuis `frontend/`, **Then** confirmer **8/8 tests fail** avec violations axe-core (≥ 600 violations cumul). Si l'une des conditions n'est PAS satisfaite (e.g. cascade-clear post-9-5-1b a déjà fixé certaines), réviser le scope avant de continuer.

### Phase A — Fix KF #91 DropdownMenu nested-interactive

3. **Given** la cause racine documentée KF #91 (`<DropdownMenu.Trigger><Button>...</Button></DropdownMenu.Trigger>` produit 2 `<button>` imbriqués → `nested-interactive` wcag2a 4.1.2 serious), **When** le fix est appliqué à `frontend/src/routes/(app)/+layout.svelte:136-143` (Trigger ouvrante + bloc Button + fermeture Button — la ligne `</DropdownMenu.Trigger>:144` peut être préservée selon la variante choisie), **Then** il satisfait :
   - **Pattern bits-ui recommandé** : utiliser le snippet `child` (ou équivalent `asChild`) de `DropdownMenu.Trigger` pour forwarder les props/ref au `<Button>` intérieur **OU** simplement utiliser le styling Tailwind directement sur le `<DropdownMenu.Trigger>` natif sans wrapper `<Button>` (préférer cette option si elle préserve le look — un seul `<button>` rendu, classes appliquées directement sur lui).
   - **Préservation visuelle** : le look-and-feel du bouton profil (icône User + texte rôle + chevron) reste identique. Aucune régression visuelle perceptible utilisateur (manual check via headless browser ou Playwright screenshot avant/après si dispo).
   - **Préservation fonctionnelle** : click ouvre le menu, keyboard navigation (Tab, Enter, Escape) inchangée, focus visible préservé.
   - **Aucun composant `bits-ui` upstream patché** : la solution utilise l'API existante de bits-ui (pas de fork lib, pas de monkey-patching).

4. **Given** le patch AC #3 appliqué, **When** `npx playwright test reports.spec.ts:85 reports.spec.ts:96 --reporter=list` est exécuté, **Then** :
   - **0 occurrence** de la règle `nested-interactive` (id `nested-interactive`, target `#bits-c1` ou variante) dans les violations.
   - Si les 2 tests `reports.spec.ts` passent maintenant (0 violations résiduelles) → KF #91 résolue à 100% par cette fix unique. Sinon → KF #91 partiellement résolue (la règle `nested-interactive` du DropdownMenu est partie, mais d'autres violations restent — les catégoriser en Phase B).

### Phase B — Mesure cascade + catégorisation R2

5. **Given** le patch AC #3 (Phase A) appliqué, **When** `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 KESH_BACKEND_URL=http://127.0.0.1:3000 npx playwright test auth.spec.ts:90 auth.spec.ts:98 contacts.spec.ts:150 homepage-settings.spec.ts:61 invoices.spec.ts:87 products.spec.ts:78 reports.spec.ts:85 reports.spec.ts:96 --reporter=list 2>&1 | tee tests/e2e/post-kf91-9-5-1c.log` est ré-exécuté, **Then** mesurer le **delta cascade** : (a) compter le nombre de violations par test post-fix vs baseline AC #2, (b) calculer la **réduction cumulée** : `sum(baseline_violations) - sum(post_kf91_violations)`, (c) confirmer la prédiction empirique : le fix KF #91 devrait retirer **~1 violation `nested-interactive` par page `(app)/*`** (1 seul noeud `#bits-c1` par page dans le DOM bits-ui — confirmé par `grep -c "nested-interactive" baseline-post-9-5-1b.log` → 1 violation par page, pas un cluster). Délivrable cascade attendue : **-7 à -8 violations cumul** (6 routes `(app)/*` distinctes × 1 violation chacune, dont `/` testée 2× par `auth.spec.ts:98` + `homepage-settings.spec.ts:61` qui ciblent le même DOM — cf. Dev Notes §"Cascade KF #91"). Marginal vs total ≥ 600. La page `/login` (auth.spec.ts:90) n'utilise pas le layout app-shell donc non-cascade.

6. **Given** les violations résiduelles post-fix-KF #91, **When** la catégorisation est effectuée, **Then** un **tableau Markdown** par catégorie axe-core est produit dans le Change Log :
   - `color-contrast` (4.1.3 wcag2aa, ratio insuffisant) — fix mécanique : ajuster les couleurs CSS du design system.
   - `alt-text` / `image-alt` (1.1.1 wcag2a, images sans alt) — fix mécanique : ajouter `alt=""` ou `alt="description"` selon décoratif vs informatif.
   - `aria-*` (multiple wcag2a, label/role/state ARIA manquants ou invalides) — fix mécanique simple (`aria-label="..."`) à moyen (refactor ARIA pattern).
   - `landmark-*` (1.3.1 wcag2a, régions de page sans `<main>` / `<nav>` / etc.) — fix architectural : restructurer le HTML sémantique du layout.
   - `heading-order` (1.3.1 wcag2a, sauts h1 → h3 sans h2) — fix architectural : reprise hiérarchie titres par page.
   - `region` (best-practice, contenu hors landmark) — fix architectural lié à `landmark-*`.
   - `nested-interactive` autres (4.1.2 wcag2a, autres boutons imbriqués hors DropdownMenu — e.g. dans tableaux, formulaires) — fix mécanique à architectural selon contexte.
   - `focus-management` (multiple, ordre tab/focus visible incorrect) — fix architectural.
   - Autres règles découvertes empiriquement.

7. **And** pour chaque catégorie, indiquer (a) le nombre total de violations dans la catégorie cumul 8 tests, (b) le nombre de pages affectées, (c) un échantillon de l'extrait HTML coupable (1-2 lignes max), (d) la classification **mécanique** vs **architectural** vs **mixte**.

### Phase C — Décision R2 (gate)

8. **Given** le tableau de catégorisation Phase B (AC #6 + #7), **When** la règle R2 du parent epic-9-5.md est évaluée (description inline §"Story 9.5-1c" ligne 87 : « Split possible 9-5-1c-quick + 9-5-1c-structural si > 100 violations résiduelles » ; aucune section §"Risque R2" séparée — R2 est référencé par anticipation dans cette spec story), **Then** :
   - **Si cumul violations résiduelles post-KF #91 ≤ 100** : passer directement à Phase D (fix in-story) — AC #9-#12 applicables.
   - **Si cumul violations résiduelles post-KF #91 > 100** : **R2 déclenché** (seuil strict `> 100` cohérent parent epic-9-5.md ligne 87) → splitter en sous-stories AC #13-#14 applicables (Phase D skip).
   - **Garde-fou de classification** : si parmi les violations résiduelles, **plus de 80% relèvent de catégories architecturales** (landmark-* / heading-order / region / focus-management) — calculé sur le **nombre total de violations** (formule explicite : `sum(violations dans catégories architecturales) / sum(toutes violations résiduelles) > 0.80`, PAS un comptage par catégorie qui pourrait donner un ratio différent), même avec un total < 100 violations, considérer R2 déclenché (le scope architectural justifie le split même à faible volume).

### Phase D — Implémentation in-story (si R2 NON déclenché AC #8)

9. **Given** R2 NON déclenché, **When** les violations résiduelles sont fixées par catégorie, **Then** chaque catégorie est patchée dans son propre commit séparé : `fix(a11y/<category>): <short description> (refs #55)`. Exemple : `fix(a11y/color-contrast): adjust button text contrast on light theme (refs #55)`.

10. **And** pour les fix touchant des composants partagés (e.g. `Button.svelte`, `Input.svelte` du design system), valider visuellement qu'aucune régression UI n'est introduite (manual check 2-3 pages représentatives + Playwright re-run de la suite E2E entière hors scope a11y).

11. **And** pour les fix touchant les pages directement (e.g. `routes/(app)/contacts/+page.svelte`), valider que le test E2E `axe-core` correspondant passe maintenant (0 violations).

12. **Given** AC #9-#11 satisfaits, **When** la suite axe-core est re-runnée, **Then** **8/8 tests pass** (0 violations cumulées). Si 1 ou plusieurs tests échouent encore avec des violations résiduelles, escalader : (a) si <10 violations résiduelles totales → documenter comme « known accepted v0.1 » avec justification dans Dev Notes ; (b) si ≥10 résiduelles → R2 déclenché tardivement, basculer sur AC #13-#14.

### Phase D-bis — Décision split (si R2 déclenché AC #8 OU AC #12-b)

13. **Given** R2 déclenché, **When** la décision de split est rendue, **Then** **2 sous-stories sont créées** dans `_bmad-output/implementation-artifacts/sprint-status.yaml` :
    - **9-5-1c-quick** — Fix mécanique a11y : color-contrast + alt-text + ARIA simple. Scope précis : (a) liste pages affectées par catégorie, (b) liste fichiers à patcher (composants partagés Button/Input/Toast + pages spécifiques selon catégorisation), (c) effort estimé (rapide — patches CSS + attributs HTML).
    - **9-5-1c-structural** — Fix architectural a11y : landmark-* + heading-order + region + focus-management + nested-interactive non-DropdownMenu. Scope précis : (a) restructuration sémantique HTML par page, (b) refactor hiérarchie titres, (c) audit focus management cross-pages.
    - **Note importante** : les 2 sous-stories ne peuvent pas être implémentées en parallèle si elles touchent le même composant partagé (e.g. un fix `color-contrast` sur `Button.svelte` quick et un fix `focus-management` sur le même `Button.svelte` structural — conflit Git). Séquencer 1c-quick puis 1c-structural pour éviter merge conflicts.

14. **And** l'entrée `9-5-1c-kf-fix-a11y` sprint-status passe en status `split` (analogue 9-5-1 historique post-9-5-1a), ne sera pas implémentée directement.

### Closure GitHub Issues

15. **Given** Phase D complétée (R2 non-déclenché chemin) **OU** Phase D-bis complétée (split chemin), **When** la story est marquée done, **Then** :
    - **KF #91** : fermée systématiquement par commit dédié `fix(a11y): close KF #91 KF-027 DropdownMenu.Trigger nested-interactive (closes #91)`. La fix Phase A est toujours appliquée, indépendamment de R2.
    - **KF #55** : fermée **uniquement si chemin Phase D** (toutes violations cumul fixées). Si chemin Phase D-bis (split), **KF #55 reste ouverte** — sera fermée par 9-5-1c-quick + 9-5-1c-structural cumulés (la dernière des 2 à se merger ferme #55).

### Test Locally First + non-régression

16. **Given** la story 9-5-1c touche un fichier `.svelte` de production (`+layout.svelte`) et potentiellement d'autres composants en Phase D, **When** un commit est créé, **Then** la règle CLAUDE.md `Test Locally First` s'applique **intégralement** :
    - Backend Rust : `cargo fmt --all -- --check` + `cargo build --workspace --all-targets` + `cargo clippy --workspace --all-targets -- -D warnings` (skip `cargo test --workspace` si 0 modif Rust).
    - Frontend Svelte : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` — les 4 checks obligatoires.
    - E2E : la mesure axe-core de Phases B et D EST le travail de la story, sert d'évidence Test Locally First sur le scope a11y.

17. **And** aucune régression sur les autres tests E2E hors scope a11y : run rapide `npx playwright test auth.spec.ts contacts.spec.ts homepage-settings.spec.ts invoices.spec.ts products.spec.ts reports.spec.ts users.spec.ts --grep-invert "axe a11y|axe-core"` (flag `--grep-invert`, **PAS** `--grep -v` qui matche literalement le test name `-v` et fait tourner aucun test — bug de syntaxe Playwright CLI 1.59.1 ; pattern alternation regex standard `|`, pas l'échappement sed `\|`) → exclut les tests axe-core eux-mêmes pour ne mesurer que les régressions UI/comportement. Comparer aux baselines pré-9-5-1c. **Note** : `+layout.svelte` est rendu sur toutes les pages `(app)/*`, donc une régression visuelle/comportementale sur le DropdownMenu impacterait l'ensemble de la suite E2E — vigilance requise.

18. **And** `npm run test:unit` reste à 253/253 pass post-changes (sauf si AC #10 nécessite ajout de tests Vitest pour helpers a11y partagés, auquel cas le compte peut augmenter).

## Tasks / Subtasks

- [ ] **T1** Pré-flight environnement (AC: #1)
  - [ ] T1.1 Vérifier branche `chore/epic-9-5-planning` checkée + working tree propre.
  - [ ] T1.2 Backend kesh-api running KESH_TEST_MODE=true + KESH_HOST=127.0.0.1 (réutiliser si déjà up depuis 9-5-1b, sinon redémarrer). Sanity check `curl -fsS http://127.0.0.1:3000/api/v1/_test/seed -X POST -H 'Content-Type: application/json' -d '{"preset":"with-company"}'` → `{"preset":"with-company","ok":true}`. **Important** : ce curl truncate + re-seed la DB — c'est volontaire et nécessaire pour repartir d'un état déterministe pour AC #2 (sinon comparaison violations baseline vs post-fix invalidée par état stale 9-5-1b). Si le backend a été tué depuis 9-5-1b, le redémarrage seul appliquera les migrations sans seed — le seed `with-company` reste nécessaire avant T2.1.
  - [ ] T1.3 `cargo build --workspace` propre. `cd frontend && npm run build` propre.
  - [ ] T1.4 `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` exporté en session (ou inline par commande).

- [ ] **T2** Phase A — Baseline pré-fix 8 tests axe-core (AC: #2)
  - [ ] T2.1 Depuis `frontend/`, exécuter `npx playwright test auth.spec.ts:90 auth.spec.ts:98 contacts.spec.ts:150 homepage-settings.spec.ts:61 invoices.spec.ts:87 products.spec.ts:78 reports.spec.ts:85 reports.spec.ts:96 --reporter=list 2>&1 | tee tests/e2e/baseline-pre-9-5-1c-a11y.log`.
  - [ ] T2.2 Confirmer 8/8 fail. Compter violations par test via `grep -E "Received  \+ [0-9]+" tests/e2e/baseline-pre-9-5-1c-a11y.log`. Total attendu ≥ 600. Si certains tests passent (cascade-clear post-9-5-1b ou autre), retirer du scope avec note dans Change Log et confirmer auprès de Guy avant de continuer.
  - [ ] T2.3 `git add -f tests/e2e/baseline-pre-9-5-1c-a11y.log` (cohérent précédent `baseline-pre-9-5-1b.log` 9-5-1b force-add). Commit `chore(9-5-1c): baseline pre-fix a11y — 8 tests axe-core failing`.

- [ ] **T3** Phase A — Fix KF #91 DropdownMenu pattern (AC: #3)
  - [ ] T3.1 Lire `frontend/src/routes/(app)/+layout.svelte:136-143` + grep tout autre usage `DropdownMenu.Trigger` dans le repo : `grep -rn "DropdownMenu.Trigger" frontend/src/`. **Attendu : 2 occurrences** validées ground-truth Pass 1 validate : (a) `+layout.svelte:136` — wrapper `<Button>` à l'intérieur → **problématique** (violation WCAG 2.1 4.1.2 `nested-interactive`) → **à patcher** ; (b) `design-system/+page.svelte:151` — utilise déjà `{#snippet child({ props })}` (pattern bits-ui 2.x correct, non-violating) → **NE PAS modifier**. Si une 3ᵉ occurrence émerge (régression future ou ajout post-spec), évaluer individuellement avant patch.
  - [ ] T3.2 Consulter https://bits-ui.com/docs/components/dropdown-menu pour le pattern `child` snippet ou `asChild` (à adapter selon version `bits-ui` du projet — voir `frontend/package.json` pour la version installée).
  - [ ] T3.3 Appliquer le fix selon **une des 2 variantes Dev Notes §"Pattern bits-ui DropdownMenu.Trigger"** (post-Pass 3 MEDIUM-01 correctif) — **NE PAS copier les classes Tailwind manuellement** (risque régression a11y focus-visible) : (a) **variante A DRY** — `import { buttonVariants } from '$lib/components/ui/button/button.svelte'` + `<DropdownMenu.Trigger class={buttonVariants({ variant: 'ghost' })}>` ; (b) **variante B (préférence projet)** — `<DropdownMenu.Trigger>{#snippet child({ props })}<Button variant="ghost" {...props}>...</Button>{/snippet}</DropdownMenu.Trigger>` (cohérent `design-system/+page.svelte:151`). Préserver les attributs ARIA + le `aria-hidden` sur les icônes User/ChevronDown.
  - [ ] T3.4 `npm run check` : 0 erreurs Svelte (les warnings pré-existants 25 restent acceptables). Pas de nouveau warning sur le composant patché.
  - [ ] T3.5 `npm run build` : build OK (adapter-static génère `frontend/build/`).
  - [ ] T3.6 Commit `fix(a11y): close KF #91 KF-027 DropdownMenu.Trigger nested-interactive (closes #91)` — body cite le pattern bits-ui utilisé + diff `+layout.svelte`.

- [ ] **T4** Phase A — Verif KF #91 (AC: #4)
  - [ ] T4.1 `npx playwright test reports.spec.ts:85 reports.spec.ts:96 --reporter=list 2>&1 | tee tests/e2e/post-kf91-reports.log`.
  - [ ] T4.2 `grep -c "nested-interactive" tests/e2e/post-kf91-reports.log` : 0 attendu (la règle `nested-interactive` doit avoir disparu sur le DropdownMenu profile).
  - [ ] T4.3 Documenter dans Change Log : statut 2 tests reports (pass/fail), nombre de violations restantes par test (si fail = violations restantes hors `nested-interactive` doivent être catégorisées Phase B), confirmation que KF #91 spécifique est partie.

- [ ] **T5** Phase B — Mesure cascade + catégorisation (AC: #5, #6, #7)
  - [ ] T5.1 Re-run 8 tests : `npx playwright test auth.spec.ts:90 auth.spec.ts:98 contacts.spec.ts:150 homepage-settings.spec.ts:61 invoices.spec.ts:87 products.spec.ts:78 reports.spec.ts:85 reports.spec.ts:96 --reporter=list 2>&1 | tee tests/e2e/post-kf91-all-9-5-1c.log`.
  - [ ] T5.2 **Étape prep per-test** : compter violations résiduelles par test : `grep -E "Received  \+ [0-9]+" tests/e2e/post-kf91-all-9-5-1c.log` → produire un tableau Markdown **5 colonnes** (intermédiaire pour traçabilité, **distinct** de T5.5 qui aggrège par catégorie cumul) : `Test path:line` | `Baseline violations (T2.2)` | `Post-KF #91 violations` | `Delta` | `Top 3 categories observées (id axe-core, e.g. heading-order,color-contrast,landmark-one-main)`. La dernière colonne est une string CSV courte — pas 3 colonnes séparées. Ce tableau alimente T5.3-T5.4 qui font l'aggregation per-catégorie pour T5.5.
  - [ ] T5.3 Catégoriser **chaque type de violation** par règle axe-core (id `color-contrast`, `image-alt`, `landmark-one-main`, `heading-order`, `region`, `nested-interactive`, `aria-*`, etc.) — extraire via `grep -A 5 "\"id\":" tests/e2e/post-kf91-all-9-5-1c.log` ou Playwright HTML reporter.
  - [ ] T5.4 Classifier **chaque catégorie** : (a) **mécanique** (color-contrast, image-alt, aria-label simple, attribute fixes) — fix < 5 LoC par occurrence ; (b) **architectural** (landmark-one-main, heading-order, region, focus-management, refactor HTML sémantique) — fix nécessite restructure 10+ LoC par occurrence ; (c) **mixte** (e.g. aria-* qui peut être simple ou refactor selon contexte).
  - [ ] T5.5 Produire tableau récapitulatif dans Change Log :
    ```
    | Catégorie axe-core | Total violations | Pages affectées | Classification | Échantillon HTML |
    |---|---|---|---|---|
    | color-contrast | X | 5/8 | mécanique | <button class="text-gray-400">Cancel</button> (ratio 2.1:1) |
    | heading-order | Y | 6/8 | architectural | h1 → h3 sans h2 ligne 27 +page.svelte |
    | ... | ... | ... | ... | ... |
    ```

- [ ] **T6** Phase C — Décision R2 (gate) (AC: #8)
  - [ ] T6.1 Évaluer la règle R2 : cumul violations résiduelles **> 100** ? (seuil strict cohérent AC #8 + parent epic-9-5.md ligne 87)
  - [ ] T6.2 Si cumul < 100 ET ≤ 80% architectural → **R2 NON déclenché** → continuer vers T7 (Phase D in-story).
  - [ ] T6.3 Si cumul **> 100** OU > 80% architectural → **R2 déclenché** → skip T7, aller à T8 (Phase D-bis split).
  - [ ] T6.4 Documenter la décision R2 dans Change Log avec justification chiffrée (cumul violations + ratio mécanique/architectural).

- [ ] **T7** Phase D — Fix in-story (si R2 NON, AC: #9, #10, #11, #12)
  - [ ] T7.1 Pour chaque catégorie du tableau T5.5, appliquer fix par patch séparé. Commit pattern : `fix(a11y/<category>): <short description> (refs #55)`.
  - [ ] T7.2 Pour composants partagés modifiés, valider 2-3 pages représentatives manuellement (screenshot ou DevTools) + run E2E hors-a11y de la suite pour détecter régressions UI.
  - [ ] T7.3 Pour pages directement modifiées, valider que le test axe-core correspondant pass.
  - [ ] T7.4 Re-run final 8 tests : `npx playwright test auth.spec.ts:90 auth.spec.ts:98 contacts.spec.ts:150 homepage-settings.spec.ts:61 invoices.spec.ts:87 products.spec.ts:78 reports.spec.ts:85 reports.spec.ts:96 --reporter=list 2>&1 | tee tests/e2e/baseline-post-9-5-1c-a11y.log`. **8/8 pass** attendu.
  - [ ] T7.5 Si <10 violations résiduelles totales malgré T7.1-T7.3 → documenter « known accepted v0.1 » avec justification dans Dev Notes + `closes #55` quand-même. Si ≥10 résiduelles → R2 tardif déclenché, basculer T8.
  - [ ] T7.6 `git add -f tests/e2e/baseline-post-9-5-1c-a11y.log` + commit dédié `fix(a11y): close KF #55 KF-023 axe-core 6 pages — Z violations cleared (closes #55)`.

- [ ] **T8** Phase D-bis — Split sub-stories (si R2 OUI, AC: #13, #14)
  - [ ] T8.1 Créer entrée `9-5-1c-quick-a11y-mechanical` dans `_bmad-output/implementation-artifacts/sprint-status.yaml` avec scope précis (catégories mécaniques du T5.5 + fichiers à patcher).
  - [ ] T8.2 Créer entrée `9-5-1c-structural-a11y-architectural` dans sprint-status avec scope précis (catégories architecturales du T5.5).
  - [ ] T8.3 Mettre à jour `9-5-1c-kf-fix-a11y` → status `split` + commentaire « R2 déclenché : Z violations cumul, X% architectural. Décomposé en 9-5-1c-quick + 9-5-1c-structural ».
  - [ ] T8.4 Mettre à jour `_bmad-output/planning-artifacts/epic-9-5.md` section « Décision split préventif » avec note « 9.5-1c R2 déclenché empiriquement <date> — split en 9-5-1c-quick + 9-5-1c-structural. KF #55 reste ouverte jusqu'à closure 9-5-1c-structural (la dernière des 2 sous-stories à merger). KF #91 fermée par commit T3.6 ».
  - [ ] T8.5 **KF #55 reste OUVERTE** (closure différée vers 9-5-1c-quick + 9-5-1c-structural). Pas de commit `closes #55` dans 9-5-1c.

- [ ] **T9** Documentation finale + sprint-status (AC: #16, #18)
  - [ ] T9.1 Mise à jour `_bmad-output/implementation-artifacts/sprint-status.yaml` : entrée `9-5-1c-kf-fix-a11y` → `in-progress` (start T2) → `review` (après T7 ou T8) → `done` (après code-review converge). Si R2 split (T8) → status `split` au lieu de `done`.
  - [ ] T9.2 Mise à jour `_bmad-output/planning-artifacts/epic-9-5.md` § split décision avec résultat empirique 9-5-1c.
  - [ ] T9.3 Build doc Change Log avec : (a) baseline T2 + post-KF #91 T5 + tableau catégorisation T5.5, (b) décision R2 T6 + justification chiffrée, (c) chemin Phase D ou D-bis pris, (d) commits closure.

- [ ] **T10** Test Locally First — checks CI complets (AC: #16, #17)
  - [ ] T10.1 Backend Rust : `cargo fmt --all -- --check` + `cargo build --workspace --all-targets` + `cargo clippy --workspace --all-targets -- -D warnings`. Skip `cargo test --workspace` si 0 modif Rust.
  - [ ] T10.2 Frontend Svelte : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` (4 checks doivent tous passer).
  - [ ] T10.3 AC #17 non-régression : `npx playwright test auth.spec.ts contacts.spec.ts homepage-settings.spec.ts invoices.spec.ts products.spec.ts reports.spec.ts users.spec.ts --grep-invert "axe a11y|axe-core"` (flag `--grep-invert` correct CLI Playwright 1.59.1 vs `--grep -v` invalide qui matche literalement `-v` ; pattern alternation `|` standard regex). Comparer aux baselines pré-9-5-1c.
  - [ ] T10.4 Push branche `chore/epic-9-5-planning` reporté à fin Epic 9.5 (pattern « avoid parallel PRs »).

## Dev Notes

### Cadrage scope minimal — pattern Story 9-5-1a/2/3/1b

Cette story 9-5-1c suit la même discipline que les sous-stories Epic 9.5 précédentes : **scope minimaliste, file-list explicite, anti-pattern Story 7-1 historique** (4 passes spec validate sur scope > 5 modules). Ici le scope core est limité à `+layout.svelte` (Phase A) + mesure empirique + décision gate R2. Si R2 ne se déclenche pas (improbable vu les chiffres baseline empiriques), Phase D est in-story mais reste scoped à des fichiers identifiés en T5.

### Empirique pré-spec — données 2026-05-19 baseline post-9-5-1b

Les chiffres de violations sont mesurés empiriquement le 2026-05-19 (run combiné AC #17 non-régression 9-5-1b + run 9-spec post-T4-T6 9-5-1b). Ils peuvent différer du baseline 2026-04-30 (KF #55 original) si l'app a évolué entre temps. La baseline AC #2 est la source de vérité pour cette story.

**Violations cumulées 2026-05-19 (somme indicative ≥ 600)** :
- auth.spec.ts:90 (login) — 109 (baseline 2026-04-30 conforme)
- auth.spec.ts:98 (layout principal) — 82 (baseline 2026-04-30 conforme)
- contacts.spec.ts:150 — 90 (nouveau measurement)
- homepage-settings.spec.ts:61 — 82 (nouveau measurement)
- invoices.spec.ts:87 — 49 (nouveau measurement, inclut nested-interactive KF #91 confirmé)
- products.spec.ts:78 — 90 (nouveau measurement)
- reports.spec.ts:85 (empty) — 49 (nouveau measurement, KF #91 canonique)
- reports.spec.ts:96 (populated) — 49 (idem)

### Pattern bits-ui `DropdownMenu.Trigger` — KF #91 fix probable

La lib `bits-ui` (version `^2.16.5` confirmée ground-truth Pass 1 validate) fournit des composants headless. Le pattern actuel `+layout.svelte:136-143` (Trigger ouvrante l.136 → Button l.137-143) :

```svelte
<DropdownMenu.Trigger>
    <Button variant="ghost" class="flex items-center gap-2">
        <User class="h-4 w-4" aria-hidden="true" />
        <span class="text-sm">{authState.currentUser?.role ?? 'Utilisateur'}</span>
        <ChevronDown class="h-3 w-3" aria-hidden="true" />
    </Button>
</DropdownMenu.Trigger>
```

produit 2 `<button>` imbriqués : un par `<DropdownMenu.Trigger>` (rendered par bits-ui) + un par `<Button>` (composant projet). Violation `nested-interactive` wcag2a 4.1.2 « serious ».

**⚠️ Attention focus-visible a11y** : `<Button variant="ghost">` applique ~19 classes Tailwind via `buttonVariants` (helper `tailwind-variants`) — `base` (10 classes inclut `focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3` **critique a11y** + `aria-invalid:*` + `transition-all` + sizing icons) + `variant.ghost` (`hover:bg-muted hover:text-foreground aria-expanded:bg-muted aria-expanded:text-foreground`) + `size.default` (`h-8 gap-1.5 px-2.5`). **Une story a11y ne doit PAS introduire une régression `focus-visible`** — copier les classes à la main est risqué et non-DRY.

**Fix variante A préférée — import `buttonVariants` helper** : utiliser le helper exporté pour générer la string de classes complète, identique au `<Button>` d'origine :

```svelte
<script>
    import { buttonVariants } from '$lib/components/ui/button/button.svelte';
    // ... autres imports
</script>

<DropdownMenu.Trigger class={buttonVariants({ variant: 'ghost' })}>
    <User class="h-4 w-4" aria-hidden="true" />
    <span class="text-sm">{authState.currentUser?.role ?? 'Utilisateur'}</span>
    <ChevronDown class="h-3 w-3" aria-hidden="true" />
</DropdownMenu.Trigger>
```

**Avantage** : focus-visible + sizing + transitions préservés à 100% identiques à `<Button variant="ghost">`. Pas de duplication classes. Future-proof si `buttonVariants` évolue. Cohérent design-system.

**Fix variante B fallback — snippet `child` (pattern bits-ui 2.x utilisé déjà dans `design-system/+page.svelte:151`)** : préserver le composant `<Button>` en passant ses props via le snippet :

```svelte
<DropdownMenu.Trigger>
    {#snippet child({ props })}
        <Button variant="ghost" class="flex items-center gap-2" {...props}>
            <User class="h-4 w-4" aria-hidden="true" />
            <span class="text-sm">{authState.currentUser?.role ?? 'Utilisateur'}</span>
            <ChevronDown class="h-3 w-3" aria-hidden="true" />
        </Button>
    {/snippet}
</DropdownMenu.Trigger>
```

**Avantage** : le composant `<Button>` reste utilisé tel quel, le snippet `child` indique à bits-ui de **remplacer son trigger natif** par le `<Button>` du child (un seul `<button>` rendu, `nested-interactive` résolu). Pattern identique à `design-system/+page.svelte:151` — cohérence projet.

**À déterminer T3.1-T3.3** selon préférence (les 2 variantes sont fonctionnellement équivalentes a11y-wise) :
- **A** plus DRY (le helper `buttonVariants` est la source de vérité unique), 1 LoC import + class={...}.
- **B** plus minimal au niveau diff (snippet wrap autour du Button existant), 2 LoC supplémentaires (snippet + close).

Préférence projet : **variante B** car le pattern est déjà utilisé dans `design-system/+page.svelte:151` (cohérence intra-codebase). Choix final documenté T3.6 commit body.

**Path correct de `Button.svelte`** : `frontend/src/lib/components/ui/button/button.svelte` (et NON `frontend/src/lib/shared/ui/Button.svelte` qui n'existe pas — vérifié ground-truth Pass 3 Opus).

### Cascade KF #91 sur les 7 autres tests KF #55

Hypothèse : le DropdownMenu profile est rendu via `+layout.svelte` sur **toutes** les routes `(app)/*`. Donc :

- `contacts.spec.ts:150` charge `/contacts` → DOM contient le DropdownMenu trigger → axe-core voit la violation `nested-interactive`.
- `homepage-settings.spec.ts:61` charge `/` → idem.
- `invoices.spec.ts:87` charge `/invoices` → idem.
- `products.spec.ts:78` charge `/products` → idem.
- `reports.spec.ts:85/96` charge `/reports` → idem (KF #91 canonique).

**Pages non-cascade KF #91** : `auth.spec.ts:90` charge `/login` qui utilise un layout `(auth)` séparé sans DropdownMenu profile. `auth.spec.ts:98` charge le layout principal `/` après login — donc cascade KF #91 applicable.

**Important — route `/` testée deux fois** : `auth.spec.ts:98` et `homepage-settings.spec.ts:61` ciblent **tous deux la route `/`** (vérifié ground-truth Pass 3 Opus : `await expect(page).toHaveURL('/')` dans les 2 tests, **82 violations identiques** dans la baseline empirique 2026-05-19). Un fix layout `(app)` clear ces 2 tests simultanément avec un seul changement.

**Prédiction T5 corrigée** : fix KF #91 retire ~1 violation `nested-interactive` × 8 occurrences (6 routes `(app)/*` distinctes, dont `/` testé 2× → +1 occurrence supplémentaire dans le cumul) = **-7 à -8 violations cumul** (au lieu du -6 initialement annoncé qui ne comptait pas la duplication test `/`). Marginale vu le total ≥ 600 — la majorité des violations sont structurelles (heading-order, landmarks, color-contrast) et nécessitent fix per-page.

### Risque R2 — probabilité quasi-certaine vu les chiffres baseline

Le cumul baseline ≥ 600 violations est très supérieur au seuil R2 (100). Même si fix KF #91 retire 100 violations cumul (très optimiste), il restera ≥ 500. **R2 sera déclenché en T6.** L'équipe doit anticiper que 9-5-1c se conclura par un split en T8, créant 9-5-1c-quick + 9-5-1c-structural pour Epic 9.5.

### Risque R3 — `+layout.svelte` modif peut casser tests E2E hors a11y

Le DropdownMenu profile est utilisé pour le toggle mode (mode-expert.spec.ts:26 — déjà KF-029 #97), pour le logout, et pour la sélection langue (non fonctionnel v0.1 mais visible). Une modif sur `+layout.svelte:136-143` peut casser le toggle mode-expert (le sélecteur `button:has-text("Mode")` cible probablement le bouton Mode dans le menu).

**Mitigation T10.3** : non-régression run E2E avec `--grep-invert "axe a11y|axe-core"` (flag correct Playwright CLI 1.59.1 — `--grep -v` est invalide, matcherait literalement le nom de test `-v`) exclut les tests axe-core eux-mêmes (qu'on patche par construction) et valide que le reste de la suite reste verte. Si mode-expert ou un autre test régresse, le fix `+layout.svelte` doit préserver le DOM structure du DropdownMenu (id, data-testid, aria-attrs) — pas seulement supprimer le `<Button>` interne. Vérifier que `button:has-text("Mode")` (KF-029 #97 cas) reste cible-able post-fix.

### Coordination avec 9-5-1d (KF #47 + #50)

9-5-1d touche `fiscal-years.spec.ts` (KF #47 AC #22 fallback toast) et `kf004_no_op_e2e.rs` (KF #50 race REPEATABLE READ). Aucun chevauchement de fichiers avec 9-5-1c. Peut être implémentée en parallèle de 9-5-1c (mêmes branche ou branche séparée — à arbitrer avec Guy selon préférence pattern « avoid parallel PRs »).

### Memory carries

- `reference_playwright_ubuntu26` : obligatoire `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` (déjà appliqué T1.4).
- `feedback_haiku_review_diff_combined` : discipline grep ground-truth obligatoire pour les passes Haiku review. Variant méta-spec vs code source documenté (cf. 9-5-1b Pass 2 Haiku).
- `feedback_avoid_parallel_prs` : pas de PR séparée — branche `chore/epic-9-5-planning` cumul Epic 9.5.

### Project Structure Notes

- **Fichiers édités par 9-5-1c (Phase A — toujours)** :
  - `frontend/src/routes/(app)/+layout.svelte` (lignes 136-143 — fix DropdownMenu pattern, ~5-10 LoC modifiées selon variante A ou B).
  - `_bmad-output/implementation-artifacts/9-5-1c-kf-fix-a11y.md` (cette spec, Change Log final).
  - `_bmad-output/implementation-artifacts/sprint-status.yaml` (statut entry).
  - `_bmad-output/planning-artifacts/epic-9-5.md` (section split mise à jour).
  - `frontend/tests/e2e/baseline-pre-9-5-1c-a11y.log` + `baseline-post-9-5-1c-a11y.log` (force-added, cohérent 9-5-1b).

- **Fichiers édités par 9-5-1c (Phase D in-story si R2 NON)** :
  - Selon catégorisation T5.5. Probablement composants partagés `frontend/src/lib/shared/ui/` + pages spécifiques `frontend/src/routes/(app)/*/+page.svelte`.

- **Fichiers NON touchés** :
  - **Aucun** fichier `.rs` modifié.
  - **Aucun** test E2E `.spec.ts` modifié (les tests axe-core eux-mêmes restent inchangés — c'est leur DOM cible qui change).
  - **Aucun** test Vitest modifié sauf si Phase D ajoute des helpers a11y partagés.

- **GitHub Issues** :
  - **#91 fermée** systématiquement par T3.6 (`closes #91`), indépendamment de R2.
  - **#55 fermée** uniquement si R2 NON et T7.4 8/8 pass (`closes #55`). Sinon transfère vers 9-5-1c-quick + 9-5-1c-structural.

### Testing standards summary

- **Pattern axe-core E2E** : tests existants utilisent `AxeBuilder({ page }).analyze()` puis `expect(results.violations).toEqual([])`. La pattern reste inchangée par 9-5-1c — seul le DOM cible est modifié.
- **Pattern bits-ui forwarding** : à appliquer uniformément si plusieurs `DropdownMenu.Trigger>Button` ou `Sheet.Trigger>Button` sont identifiés en T3.1. Cohérence sur l'app shell.
- **Visual regression** : pas de test automatisé visual regression dans Kesh v0.1. Vérification manuelle ou via Playwright screenshot ad-hoc dans T7.2 si Phase D.

### Estimation effort

- **T1 (pré-flight)** : 10 min (backend déjà running depuis 9-5-1b si session continue).
- **T2 (baseline)** : 5-10 min.
- **T3 (fix KF #91)** : 30-60 min (recherche bits-ui pattern + code + npm check/build).
- **T4 (verif KF #91)** : 5 min.
- **T5 (mesure cascade + catégorisation)** : 45-60 min (extraction et tableau).
- **T6 (R2 gate)** : 5 min.
- **T7 (Phase D in-story)** : **N/A si R2 split** — sinon 2-5h selon volume.
- **T8 (split sub-stories)** : 30-45 min (création entries sprint-status + epic-9-5.md update).
- **T9-T10 (doc + Test Locally First)** : 30 min.
- **Total chemin R2 split (probable)** : ~2-3h.
- **Total chemin Phase D in-story (improbable)** : ~5-8h.

### References

- [Source: _bmad-output/planning-artifacts/epic-9-5.md#Story-9.5-1] — spec parent + R2 split rule.
- [Source: _bmad-output/implementation-artifacts/9-5-1a-kf-triage.md] — triage amont (mapping KF #55 + #91 → 9-5-1c a11y).
- [Source: _bmad-output/implementation-artifacts/9-5-1b-kf-fix-infra.md] — pattern dev-story mode orchestré complet réutilisable + baselines force-add precedent.
- [Source: frontend/src/routes/(app)/+layout.svelte:136-143] — code cible KF #91 fix (Trigger ouvrante + Button + Button fermante).
- [Source: frontend/src/routes/design-system/+page.svelte:151] — référence pattern correct bits-ui 2.x `{#snippet child({ props })}`, ne pas modifier.
- [Source: frontend/tests/e2e/baseline-post-9-5-1b.log] — baseline empirique 2026-05-19 (homepage + invoices a11y violations).
- [GitHub Issue #55 KF-023] — axe-core 6 pages a11y violations.
- [GitHub Issue #91 KF-027] — DropdownMenu.Trigger nested-interactive wcag2a 4.1.2.
- [bits-ui docs](https://bits-ui.com/docs/components/dropdown-menu) — pattern asChild / child snippet (version exacte selon `frontend/package.json`).
- [axe-core rules](https://dequeuniversity.com/rules/axe/4.11/) — référence catégories violations.
- [Source: CLAUDE.md§Test Locally First] — checks CI obligatoires.
- [Source: CLAUDE.md§Review Iteration Rule] — cycle review 2-3 passes attendues, LLM différent par passe.
- [Source: CLAUDE.md§Règle de splitting préventif] — discipline file-list explicite + R2 anticipé.

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

### Pass 1 spec validate — 2026-05-19, Sonnet 4.6 (subagent contexte frais)

**Verdict trend** : 0 CRITICAL + 2 HIGH + 4 MEDIUM + 3 LOW = 9 findings (Convergence : NON).

**Discipline grep ground-truth Sonnet** appliquée — 9/9 ground-truth verifications positives (chaque CRITICAL/HIGH vérifié par lecture directe ou commande CLI avant émission).

**Patches appliqués (9/9 — tous patchables sans defer)** :

1. **HIGH-01 — Playwright `--grep -v` syntaxe invalide (3 occurrences AC #17 + T10.3 + Dev Notes R3)** : la CLI Playwright 1.59.1 n'accepte que `--grep-invert`, pas `--grep -v`. Vérifié `npx playwright test --help` → `-g, --grep <grep> ... --grep-invert <grep>` uniquement. **Patch** : `--grep -v "axe-core"` → `--grep-invert "axe a11y|axe-core"` (alternation regex standard `|` sans backslash `\|` qui est syntaxe sed inadéquate). 3 occurrences corrigées + commentaire de garde-fou explicite « PAS `--grep -v` qui matche literalement le test name `-v` ».

2. **HIGH-02 — AC #5 prédiction cascade incohérente avec Dev Notes** : AC #5 disait « ~5-10 violations `nested-interactive` par page », Dev Notes disaient « ~1 violation × 6 pages = -6 cumul ». Empirique baseline-post-9-5-1b.log confirme 1 noeud `#bits-c1` par page (pas un cluster). **Patch** : AC #5 réécrit pour aligner : « ~1 violation `nested-interactive` par page `(app)/*` (1 seul noeud `#bits-c1` par page dans le DOM bits-ui) ; cascade attendue -6 violations cumul (marginal vs total ≥ 600) ».

3. **MEDIUM-01 — Story "so that" factuellement faux sur login** : disait « page login ne présente plus 109 violations dont ~50 cascading nested-interactive depuis le shell », mais `/login` utilise `routes/login/+page.svelte` sans le layout app-shell (vérifié ground-truth — aucun layout `(auth)/+layout.svelte` n'existe, login est sous le layout racine sans `DropdownMenu`). **Patch** : "so that" réécrit pour distinguer login (109 violations d'autres catégories, non-cascade) et les 6 pages `(app)/*` (cascade KF #91 applicable).

4. **MEDIUM-02 — Référence inexistante §"Risque R2 — KF #55 si > 100 violations"** : `grep -nF "Risque R2" epic-9-5.md` → aucun résultat. R2 est mentionné inline ligne 87 description Story 9.5-1c « (R2 ci-dessous) » qui promet une section non-écrite. **Patch** : AC #8 réécrit pour citer la source réelle (« description inline §"Story 9.5-1c" ligne 87 ; aucune section §"Risque R2" séparée — R2 est référencé par anticipation dans cette spec story »).

5. **MEDIUM-03 — T3.1 ambiguïté sur 2 occurrences DropdownMenu.Trigger** : `grep -rn "DropdownMenu.Trigger" frontend/src/` retourne 2 hits : (a) `+layout.svelte:136` problématique → patcher, (b) `design-system/+page.svelte:151` utilise déjà `{#snippet child({ props })}` correct → NE PAS modifier. Sans précision, le dev pourrait « fixer » le pattern déjà correct. **Patch** : T3.1 réécrit pour énumérer explicitement les 2 occurrences ground-truth avec verdict per-fichier.

6. **MEDIUM-04 — Garde-fou 80% architectural calcul non spécifié** : « plus de 80% relèvent de catégories architecturales » ambigu entre (a) 80% du nombre de violations OU (b) 80% du nombre de catégories. Les deux donnent des résultats opposés (e.g. 1 cat architectural × 200 violations + 4 cat mécaniques × 10 = 71% par count vs 20% par catégorie). **Patch** : AC #8 garde-fou précise « calculé sur le nombre total de violations : `sum(violations dans catégories architecturales) / sum(toutes violations résiduelles) > 0.80`, PAS un comptage par catégorie ».

7. **LOW-01 — Incohérence `:135-144` vs `:136-144` vs `:136-143`** : 4+ occurrences avec numéros divergents. Ligne réelle à modifier = 136-143 (Trigger ouvrante + Button + ferme Button), 144 = `</DropdownMenu.Trigger>` non-modifiée. **Patch** : alignement uniforme sur `:136-143` (Scope + AC #3 + T3.1 + Dev Notes + Project Structure + References).

8. **LOW-02 — AC #17 pattern manque `axe a11y`** : disait `--grep -v "axe-core"`, mais les tests `reports.spec.ts:85/96` utilisent la description `reports page has zero axe a11y violations` (sans le terme `axe-core`). Sans pattern complet ils auraient été inclus en non-régression. **Patch** : AC #17 aligné sur T10.3 (`--grep-invert "axe a11y|axe-core"`).

9. **LOW-03 — AC #1 `c6f9444` BMAD upgrade ambigu** : disait « commit BMAD upgrade `c6f9444` localement (PR #95 toujours en attente merge à ce stade) » présenté comme un prérequis. Le commit est sur branche `chore/bmad-upgrade-6.6.0`, pas sur `chore/epic-9-5-planning` — un dev pourrait penser devoir cherry-pick/merger. C'est juste du contexte. **Patch** : AC #1 reformulé en « **Note contexte (pas un prerequis actif)** : commit BMAD upgrade `c6f9444` sur branche `chore/bmad-upgrade-6.6.0` en attente de merge via PR #95 — aucune action requise pour 9-5-1c ».

**Bonus ground-truth findings ajoutés au Scope** (utiles pour le dev) :
- `bits-ui 2.16.5` installé confirmé (`MenuTriggerPropsWithoutHTML = WithChild<{...}>` dans `types.d.ts`).
- `DropdownMenu.Trigger` accepte `class` prop directement (variant A préféré — `...restProps` forwarded).
- Référence cross-spec ajoutée vers `design-system/+page.svelte:151` comme pattern correct.

**Recommandation Sonnet** : Pass 2 Haiku 4.5 avec discipline grep ground-truth obligatoire (cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`).

**Modèle Pass 1** : Sonnet 4.6 (subagent isolé, contexte frais — spec créée par Opus 4.7, règle CLAUDE.md `LLM différent passe précédente` respectée).

### Pass 2 spec validate — 2026-05-19, Haiku 4.5 (subagent contexte frais)

**Verdict trend brut** : 0 CRITICAL + 0 HIGH + 1 MEDIUM + 1 LOW = 2 findings (Convergence : NON — 1 MEDIUM > LOW déclenche Pass 3).

**Discipline grep ground-truth Haiku** : appliquée — toutes les vérifications Pass 1 patches (`--grep-invert`, `~1 violation`, `description inline`, `2 occurrences`, `sum(violations`, `:136-143`, `Note contexte`) confirmées intégrées correctement par Haiku via Read direct. **0 régression** détectée par Pass 1 patches. **0 faux-positif Haiku** observé (cycle court 2-passes pour `9-5-1b` avait dismissed 1C faux-positif méta-spec vs code source — pas reproductible ici car les 2 findings Haiku Pass 2 sont des observations légitimes de la spec elle-même, pas d'allégations de patches non appliqués).

**Patches appliqués (2/2)** :

1. **MEDIUM-01 — T5.2 « 9 colonnes » incohérent avec énumération 5 items** : T5.2 disait « tableau Markdown 9 colonnes » suivi de 5 colonnes listées (`Test path:line`, `Baseline`, `Post-KF #91`, `Delta`, `Categories breakdown (top 3)`). Ambiguïté : (a) 5 colonnes + last comme CSV string OU (b) 5 + 3 sub-colonnes catégories = 8 (toujours ≠ 9). Confusion supplémentaire avec T5.5 qui aggrège per-catégorie cumul (AC #7) — structure totalement différente. **Patch** : T5.2 réécrit explicitement « **5 colonnes** » + clarification « last colonne CSV string courte — pas 3 colonnes séparées » + précision que T5.2 est étape prep per-test, distincte de T5.5 aggregation per-catégorie cumul. Lien T5.3-T5.4 mentionné comme passerelle.

2. **LOW-01 — AC #1 backend session reuse stale seed risk** : « Si backend déjà running depuis 9-5-1b → réutiliser » sans mention de re-seed. Risque que T2.1 baseline pre-fix tourne sur état stale 9-5-1b → comparaison violations baseline vs post-fix invalidée. **Patch** : T1.2 sanity check curl `/api/v1/_test/seed` annoté « truncate + re-seed la DB — c'est volontaire et nécessaire pour repartir d'un état déterministe ». Garde-fou explicite « si le backend a été tué depuis 9-5-1b, le redémarrage seul applique les migrations sans seed — le seed reste nécessaire avant T2.1 ».

**Recommandation Haiku** : Pass 3 Opus 4.7 (cycle CLAUDE.md `Sonnet → Haiku → Opus` validé empiriquement Epic 9 retrospective Insight I1). Vérifier les 2 patches MEDIUM-01 + LOW-01 + une dernière passe d'audit holistique de la spec post-Pass 1 + Pass 2 patches.

**Modèle Pass 2** : Haiku 4.5 (subagent isolé, contexte frais — règle CLAUDE.md `LLM différent passe précédente` respectée Sonnet → Haiku).

### Pass 3 spec validate — 2026-05-19, Opus 4.7 (subagent contexte frais)

**Verdict trend brut** : 0 CRITICAL + 0 HIGH + 1 MEDIUM + 3 LOW = 4 findings (Convergence : NON — 1 MEDIUM > LOW déclenche Pass 4).

**Discipline grep ground-truth Opus** : 11/11 positive — toutes les 11 patches prior (Pass 1 + Pass 2) confirmés intégrés par grep -nF + Read direct. Toutes les claims externes (Button.svelte, bits-ui types.d.ts, menu-trigger.svelte, layout.svelte, auth.spec.ts:98 + homepage-settings.spec.ts:61 cibles `/`, baseline-post-9-5-1b.log violations) vérifiées via Read.

**Patches appliqués (4/4 — tous patchables sans defer)** :

1. **MEDIUM-01 — Dev Notes `variante préférée` ghost classes ≠ réel + path Button.svelte faux** : l'exemple Dev Notes proposait `class="inline-flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium hover:bg-accent"` (~7 classes simplifiées) alors que `<Button variant="ghost">` applique ~19 classes via `buttonVariants` (`base` ligne 7 inclut `focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3` **critique a11y** + `aria-invalid:*` + `transition-all` + sizing icons ; `variant.ghost` ligne 13 ; `size.default` ligne 18). Un dev appliquant l'exemple littéralement introduirait une **régression a11y `focus-visible`** sur une story qui devrait l'améliorer (ironie). Path `frontend/src/lib/shared/ui/Button.svelte` cité dans Dev Notes n'existe pas — vrai path = `frontend/src/lib/components/ui/button/button.svelte`. **Patch** : section Dev Notes « Pattern bits-ui DropdownMenu.Trigger » réécrite avec :
   - **Avertissement focus-visible** explicite en tête (« une story a11y ne doit PAS introduire une régression `focus-visible` — copier les classes à la main est risqué et non-DRY »).
   - **Variante A préférée DRY** : `import { buttonVariants } from '$lib/components/ui/button/button.svelte'` + `<DropdownMenu.Trigger class={buttonVariants({ variant: 'ghost' })}>` — helper `tailwind-variants` génère la string complète identique à `<Button>`.
   - **Variante B fallback minimal-diff** : snippet `child` cohérent `design-system/+page.svelte:151` (pattern projet déjà utilisé) — préserve `<Button variant="ghost">` tel quel, bits-ui remplace son trigger natif par le child Button.
   - **Préférence projet documentée** : variante B (cohérence intra-codebase avec design-system).
   - **Path corrigé** : `frontend/src/lib/components/ui/button/button.svelte` (vérifié ground-truth Pass 3 Opus).

2. **LOW-01 — Seuil R2 `≥ 100` vs parent epic `> 100` strict** : AC #8 disait `< 100 → Phase D, ≥ 100 → split`, mais parent epic-9-5.md ligne 87 dit `> 100 strict`. Boundary case `cumul == 100` : spec disait split, parent disait non-split. Inconsistance cross-document. **Patch** : AC #8 aligné sur `> 100` strict cohérent parent : `≤ 100 → Phase D, > 100 → split` (+ référence explicite « seuil strict `> 100` cohérent parent epic-9-5.md ligne 87 »).

3. **LOW-02 — « 5 pages KF #55 » avec 6 éléments énumérés** : Story line 10 disait « 6 tests axe-core des 5 pages KF #55 (login + layout principal + contacts + homepage + invoices + products) » — 6 items énumérés mais compte annoncé 5. Source : parent epic-9-5.md dit « 5 pages » référant aux **5 spec files** (`auth.spec.ts` contient 2 tests). **Patch** : reformulé « 6 tests axe-core distribués sur 5 spec files (`auth.spec.ts` qui contient 2 tests `:90` login + `:98` layout principal + ...) couvrant les routes KF #55 ».

4. **LOW-03 — Cascade prédiction -6 sous-estimait duplication `/`** : `auth.spec.ts:98` et `homepage-settings.spec.ts:61` ciblent **tous deux** la route `/` (vérifié ground-truth Opus : `await expect(page).toHaveURL('/')` + 82 violations identiques empirique). Le fix layout `(app)` clear ces 2 tests simultanément. Prédiction Dev Notes « -6 cumul (6 pages × 1 violation) » sous-comptait — c'est plutôt -7 à -8 (6 routes app distinctes, dont `/` testée 2× → +1 dans le cumul tests). **Patch** : Dev Notes section « Cascade KF #91 » ajoute note « auth.spec.ts:98 + homepage-settings.spec.ts:61 ciblent tous deux `/` (DOM identique, 82 violations identiques baseline). Prédiction T5 corrigée : -7 à -8 violations cumul ».

**Note Opus** : aucun finding CRITICAL / HIGH détecté après 11 patches prior — la spec convergeait déjà sur le fond. Les 4 findings Pass 3 Opus sont des polish/precision (1 sérieux : focus-visible a11y risk) que les passes Sonnet+Haiku n'avaient pas creusé. Pattern cohérent retro Epic 9 Insight I1 : « Opus catches subtle UX-for-dev-agent issues that Sonnet+Haiku miss ».

**Recommandation Opus** : Pass 4 Sonnet 4.6 (cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`) pour valider convergence post-Pass 3 patches. Cycle 4-passes attendu cohérent Story 8-5b (5 passes) + Story 9-2b (4 passes).

**Modèle Pass 3** : Claude Opus 4.7 (subagent isolé, contexte frais — règle CLAUDE.md `LLM différent passe précédente` respectée Haiku → Opus).

### Pass 4 spec validate — 2026-05-19, Sonnet 4.6 (subagent contexte frais, cycle clos)

**Verdict trend brut** : 0 CRITICAL + 0 HIGH + 0 MEDIUM + 3 LOW = 3 findings (Convergence : **OUI** — critère CLAUDE.md « Uniquement findings LOW » atteint).

**Discipline grep ground-truth Sonnet** : 10/10 positive — toutes les 15 patches prior (9 Pass 1 + 2 Pass 2 + 4 Pass 3) confirmées intégrées + cross-verification des inconsistances résiduelles par grep -nF.

**3 LOW findings identifiées** (inconsistances Pass 3 patches non-propagées dans tasks/AC adjacents — toutes patchables triviales) :

1. **LOW-01 — T6.1/T6.3 seuil `≥ 100` vs AC #8 `> 100`** : Pass 3 LOW-01 a patché AC #8 ligne 82 à `> 100` strict mais T6.1 + T6.3 lignes 162/164 conservaient `≥ 100`. Inconsistance task vs AC. **Patch** : T6.1 + T6.3 alignés à `> 100` strict avec note explicite « cohérent AC #8 + parent epic-9-5.md ligne 87 ».

2. **LOW-02 — AC #5 cascade `-6` vs Dev Notes `-7 à -8` corrigé Pass 3** : Pass 3 LOW-03 a corrigé Dev Notes ligne 286 mais AC #5 ligne 63 conservait `-6 violations cumul`. **Patch** : AC #5 aligné à `-7 à -8 violations cumul` avec référence Dev Notes §"Cascade KF #91" + explicite route `/` testée 2× par auth.spec.ts:98 + homepage-settings.spec.ts:61.

3. **LOW-03 — T3.3 + Scope ligne 17 décrivent l'ancienne approche pré-Pass 3 MEDIUM-01** : T3.3 disait `(a) variante préférée — appliquer ses classes Tailwind directement sur <DropdownMenu.Trigger>` (l'approche que Pass 3 MEDIUM-01 a identifiée comme risquée focus-visible). Scope ligne 17 disait `variant A préféré`. Dev Notes Pass 3 disait préférence variante B. **Patch** : T3.3 réécrite avec garde-fou explicite « **NE PAS copier les classes Tailwind manuellement** (risque régression a11y focus-visible) » + énumère les 2 variantes A/B Dev Notes-compatibles + précise préférence projet B ; Scope ligne 17 soft « variant A fonctionnel » + ajoute « **Préférence projet** = variante B snippet `child` (cohérent design-system/+page.svelte:151) ».

**Verdict effectif Pass 4 final** (après 3 micro-patches LOW résiduels) : **0 CRITICAL + 0 HIGH + 0 MEDIUM + 0 LOW = CONVERGENCE COMPLÈTE** atteinte. Critère arrêt cycle CLAUDE.md largement satisfait.

**Trend cumul cycle 4-passes** :
- Pass 1 Sonnet 4.6 : 0C+2H+4M+3L = 9 → 9 patches.
- Pass 2 Haiku 4.5 : 0C+0H+1M+1L = 2 → 2 patches.
- Pass 3 Opus 4.7 : 0C+0H+1M+3L = 4 → 4 patches.
- Pass 4 Sonnet 4.6 : 0C+0H+0M+3L = 3 → 3 patches → **0 résiduel**.
- **Total : 18 patches sur 4 passes. Cycle complet `Sonnet → Haiku → Opus → Sonnet` respecté.**

**Pattern « Pass 3 Opus catches the subtle stuff »** confirmé empiriquement (cf. retrospective Epic 9 Insight I1) : Pass 3 Opus a identifié le risque focus-visible a11y critique que Pass 1 Sonnet + Pass 2 Haiku avaient missé. Pass 4 Sonnet ferme le cycle en validant la propagation des patches Pass 3.

**Modèle Pass 4** : Sonnet 4.6 (subagent isolé, contexte frais — règle CLAUDE.md `LLM différent passe précédente` respectée Opus → Sonnet, ferme le cycle).

**Statut final spec** : `ready-for-dev` confirmé. Prête pour `bmad-dev-story 9-5-1c` (mode orchestré complet attendu vu nature E2E + a11y mesure empirique). LLM dev ≠ Pass 4 → recommandé Opus 4.7 (cohérent dev-story 9-5-1b).
