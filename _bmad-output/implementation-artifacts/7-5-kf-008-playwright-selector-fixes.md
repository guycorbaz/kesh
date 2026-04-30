---
spec: "7-5-kf-008-playwright-selector-fixes"
story_id: 7.5
epic: 7
story_num: 5
title: "KF-008 — Stabilisation des sélecteurs Playwright (strict mode)"
status: "ready-for-dev"
related_kf: "KF-008"
related_issue: 27
created: 2026-04-30
last_updated: 2026-04-30
stepsCompleted:
  - spec-created
  - spec-validated
---

# Story 7.5 : KF-008 — Stabilisation des sélecteurs Playwright (strict mode)

**Status:** ready-for-dev
**Epic:** 7 (Technical Debt Closure)
**Related KF:** KF-008
**Related GitHub Issue:** [#27](https://github.com/guycorbaz/kesh/issues/27)

---

## Vue d'ensemble

**Objectif :** clore KF-008 en stabilisant les ~36 tests Playwright actuellement en échec à cause de violations du *strict mode* (`getByText()` / `getByRole()` ambigus matchant plusieurs éléments) et amener la suite E2E locale à **100 % de tests verts**, sans `.first()` ni `.nth()` de contournement.

**Valeur :** la suite E2E redevient un filet de sécurité fiable. La règle « pas de selector flaky » se généralise et bloque les régressions UI avant qu'elles n'atteignent `main`. Pré-requis pour réintroduire un job E2E bloquant en CI (hors scope, future story).

**Priorité :** MEDIUM — non bloquant fonctionnellement (les flux marchent en prod), mais bloquant pour la confiance CI v0.1 et pour la fermeture des dettes Epic 7.

---

## Story

**As a** mainteneur solo (Guy)
**I want** que tous les tests Playwright passent localement sans violation strict-mode et utilisent des sélecteurs stables (`data-testid` ou rôles spécifiques)
**So that** je peux refactorer la copie UI sans casser les E2E, détecter les régressions UI avant prod, et fermer définitivement KF-008 / issue #27.

---

## Contexte

### KF-008 — État actuel

- **Issue GitHub :** [#27](https://github.com/guycorbaz/kesh/issues/27) — *open*, labels `known-failure` + `technical-debt`.
- **Symptôme :** 36 des 76 tests Playwright échouent en *strict mode* (Playwright `>= 1.30`).
- **Exemple type :** `getByText('admin')` résout 5 éléments → `Error: locator.<action>: Error: strict mode violation`.
- **Diagnostiqué :** Story 6-4 (fixtures E2E déterministes), documenté dans `frontend/DEBUGGING-KF007.md`.
- **Hors scope KF-007 :** Story 6-5 a corrigé le bug de persistance JWT/localStorage (auth flow OK, 40/76 verts) mais n'a pas touché aux sélecteurs.

### Story 7-6 — Fondations posées

Story 7-6 (statut `review`, fichier `_bmad-output/implementation-artifacts/7-6-e2e-selector-refactoring.md` — pas de PR distincte ; les artefacts ont été poussés dans la PR #29 / commit `7c8822d` qui adresse principalement Story 6-2 multi-tenant, le 2026-04-24) a livré :

- **Pattern `data-testid`** documenté dans `frontend/docs/E2E_TESTING_BEST_PRACTICES.md` (kebab-case sémantique).
- **Audit script** `frontend/scripts/audit-e2e-selectors.js` — recense les sélecteurs fragiles (192 trouvés au dernier run : 41 `getByText` HIGH + 151 `getByRole` MEDIUM).
- **2 specs refactorées** : `onboarding.spec.ts` (AC 5/6) et `users.spec.ts` (liste utilisateurs admin).
- **Composants instrumentés** : `users/+page.svelte` (3 testids), `InvoiceForm.svelte` (3 testids), `fiscal-years/+page.svelte` (3 testids — déjà ajoutés par Story 3-7).

**Total testid existants :** ~9 répartis sur 3 fichiers Svelte. Tout le reste du codebase est encore vierge.

### Ce qu'il reste à faire (scope Story 7-5)

11 specs sont touchées par cette story (T3–T13), dont `auth.spec.ts` listé pour information (0 failures attendues). Volume HIGH/MEDIUM par sortie audit :

| Spec | `getByText` HIGH | `getByRole` MEDIUM | Tests existants | Tests en échec (estim.) |
|---|---|---|---|---|
| `accounts.spec.ts` | 10 | 2 | 8 | ~6 |
| `auth.spec.ts` | 1 | 0 | 4 | 0 (passe déjà) |
| `contacts.spec.ts` | 0 | 16 | 8 | ~4 |
| `fiscal-years.spec.ts` | 0 | 7 | ~6 | ~3 (Pass 1 reclassé KF-019) |
| `homepage-settings.spec.ts` | 7 | 0 | 4 | ~3 |
| `invoices_echeancier.spec.ts` | 0 | ~10 | 4 | ~2 |
| `invoices.spec.ts` | 1 | ~25 | 8 | ~5 |
| `journal-entries.spec.ts` | 0 | ~45 | 8 | ~6 |
| `mode-expert.spec.ts` | 0 | 0* | 2 | ~1 |
| `onboarding-path-b.spec.ts` | 6 | ~5 | ~5 | ~3 |
| `products.spec.ts` | 0 | ~31 | 8 | ~5 |
| `vat-rates.spec.ts` | 0 | 3 | ~5 | ~2 |
| **Total** | **~25** | **~155** | **~76** | **~36** |

> *Les chiffres « tests en échec » sont indicatifs ; le baseline réel doit être capturé en T1 avant tout refactor.*
>
> \* `mode-expert.spec.ts` utilise `button:has-text("Mode")` (sélecteur CSS, non détecté par `audit-e2e-selectors.js` qui ne scanne que `getByText`/`getByLabel`/`getByRole`) — voir T10 pour le fix.

Remarque : tous les `getByText` ne causent pas de strict-mode violation. Beaucoup sont seulement *brittle*. La priorité immédiate est l'élimination des **violations strict-mode** ; la généralisation `data-testid` sur les `getByText` restants est une amélioration de robustesse de second ordre, à garder dans le même refactor pour éviter une seconde passe.

### Hors scope

- **Réintroduire le job E2E en CI** — actuellement absent de `.github/workflows/ci.yml` (3 jobs : backend / frontend / docker-build). À traiter dans une story dédiée Epic 7 ou Epic 10 (Déploiement) une fois la suite locale verte.
- **Multi-worker parallel E2E** (`workers: 1` figé en `playwright.config.ts`) — gardé jusqu'à ce qu'on ait DB-par-worker.
- **Refactorer les 151 `getByRole`** qui ne provoquent **pas** de violation strict-mode et restent stables — à laisser tels quels sauf si ambiguïté avérée.
- **Tests AC #22 fiscal-years (KF-019, issue #47)** — gap dette technique distincte, ne pas y toucher dans cette story.

---

## Acceptance Criteria

### AC 1 — Baseline mesuré

**Étant donné** un backend local démarré (`KESH_TEST_MODE=true KESH_HOST=127.0.0.1` + DB seedée),
**Quand** je lance `cd frontend && npm run test:e2e -- --reporter=list 2>&1 | tee tests/e2e/baseline-pre-7-5.log`,
**Alors** le log capture le nombre exact de tests `passed` / `failed` / `skipped` et la liste précise des failures avec leur message strict-mode (ou autre).

### AC 2 — Plan de refactor priorisé

**Étant donné** le baseline AC #1,
**Quand** j'analyse chaque échec,
**Alors** je classe chaque test en échec en deux catégories documentées dans la section *Dev Agent Record* :
- **Strict-mode violation** → fix par `data-testid` ou rôle spécifique (PRIORITAIRE).
- **Autre cause** (timing, fixture, flux) → reclassé dans une nouvelle KF/issue GitHub si non lié au scope sélecteurs.

### AC 3 — `data-testid` instrumentés sur les composants ciblés

**Étant donné** la liste des composants accédés par les tests en échec strict-mode,
**Quand** j'inspecte chaque composant Svelte concerné (`frontend/src/routes/(app)/**/*.svelte` et `frontend/src/lib/components/**/*.svelte`),
**Alors** chaque composant cible expose un attribut `data-testid="<kebab-case>"` stable :
- Conteneurs liste : `data-testid="<entity>-table"` (ex. `account-table`, `contact-table`, `product-table`, `journal-entry-table`, `invoice-table`).
- Lignes de tableau : `data-testid="<entity>-row-{<key>}"` (key = id, slug ou number — préférer un identifiant qui existe dans la fixture).
- Boutons d'action principale : `data-testid="<entity>-create-button"`, `<entity>-archive-button`, etc.
- Champs critiques : `data-testid="<entity>-<field>-input"` quand l'`id` HTML existant ne suffit pas.
- Toasts / bannières : `data-testid="<entity>-<state>-banner"`.

### AC 4 — Suite Playwright 100 % verte localement

**Étant donné** tous les composants AC #3 instrumentés et tous les specs refactorés,
**Quand** je lance `cd frontend && npm run test:e2e -- --reporter=list`,
**Alors** la sortie ne contient **aucune ligne `failed`** et tous les tests **non-`test.skip`** passent. Les `test.skip` honnêtes existants (KF-019 / future-work) — **exactement 9 au baseline** (fiscal-years.spec.ts × 1 KF-019, contacts.spec.ts × 2 filtres combinés / pagination, journal-entries.spec.ts × 6 future-work) — restent skipped et sont listés explicitement dans le rapport final. **Aucun nouveau `test.skip` ne doit être ajouté** dans le cadre de cette story : tout test qu'il n'est pas possible de rendre vert doit être documenté dans une nouvelle issue GitHub (cf. règle CLAUDE.md issue tracking) avant d'être skipé, et la décision de skip doit être justifiée dans le Change Log.

### AC 5 — Zéro violation strict-mode

**Étant donné** une suite verte (AC #4),
**Quand** je grep le log de sortie pour `strict mode violation`,
**Alors** zéro occurrence. **Aucun** `.first()` / `.nth(N)` n'a été ajouté en T2-T13 *uniquement pour contourner* une ambiguïté strict-mode (les usages légitimes — récupérer la première ligne de tableau pour un test, etc. — restent autorisés et doivent être commentés `// strict-mode safe: there are intentionally many rows`).

### AC 6 — Audit script vert

**Étant donné** les patches appliqués,
**Quand** je lance `node frontend/scripts/audit-e2e-selectors.js`,
**Alors** le compteur `getByText` HIGH descend à **≤ 9** (acceptable : `auth.spec.ts:34` `Kesh` titre stable + 8 occurrences `onboarding.spec.ts` explicitement hors scope, refactor déjà fait Story 7-6 sur AC 5/6 uniquement, reste de la spec sera traité en story dette technique séparée). Le compteur global de findings est documenté dans le Change Log avant/après. **Détail attendu post-refactor :** 41 HIGH baseline − 24 fixés via T3–T13 (10 accounts + 7 homepage-settings + 1 invoices + 6 onboarding-path-b ; auth.spec.ts:34 `Kesh` reste tel quel) − 8 fixés via T3bis (users.spec.ts) = **9 résiduels** = 8 onboarding.spec.ts (refactor partiel Story 7-6 sur AC 5/6 uniquement, dette technique séparée) + 1 auth.spec.ts:34 (titre `Kesh`, stable et unique).

> **Note exit code :** `audit-e2e-selectors.js` sort en `exit(1)` dès qu'il y a `findings.length > 0`. Comportement attendu post-refactor : exit code 1 avec 9 résiduels acceptés. **Ne pas traiter ce exit code 1 comme un échec** ; valider uniquement le compteur HIGH imprimé (`≤ 9`).

### AC 7 — Helper / convention `data-testid` factorisée

**Étant donné** les nombreuses interactions répétées (clic ligne, ouverture dialog, soumission formulaire),
**Quand** j'écris ou refactore un test,
**Alors** je peux soit utiliser directement `page.locator('[data-testid="..."]')`, soit utiliser un helper documenté (ex. `byTestId(page, 'user-table')` dans `frontend/tests/e2e/helpers/`) — l'un ou l'autre, pas les deux. Le choix est justifié dans `E2E_TESTING_BEST_PRACTICES.md` mis à jour si un helper est introduit.

### AC 8 — Documentation mise à jour

**Étant donné** le pattern généralisé,
**Quand** j'inspecte `frontend/docs/E2E_TESTING_BEST_PRACTICES.md`,
**Alors** la section *Examples by Feature* couvre au moins : accounts, contacts, products, invoices, journal-entries, fiscal-years, homepage-settings, vat-rates (8 features). Le compteur baseline / final de l'audit est documenté.

### AC 9 — Régression auth / onboarding inchangées

**Étant donné** la suite refactorée,
**Quand** je relance `npm run test:e2e -- auth.spec.ts onboarding.spec.ts`,
**Alors** ces 2 specs (déjà refactorées en 6-5 / 7-6 ou stables d'origine) restent **100 % vertes** sans modification fonctionnelle ni changement de testid existant.

> **Note scope :** `users.spec.ts` est désormais inclus dans T3bis (refactor partiel à compléter — 8 HIGH lignes 41/61/62/80/88/94/101/110) et n'est donc plus un test de régression "intouché" ; `onboarding-path-b.spec.ts` est refactoré en T11. Les 3 testids existants posés Story 7-6 sur `users/+page.svelte` (`user-table` l.284, `user-row-{user.username}` l.296, `current-user-badge` l.300) doivent rester verts pendant T3bis.

### AC 10 — Issue #27 fermée par commit

**Étant donné** AC #4 et AC #5 satisfaits,
**Quand** la PR de cette story est mergée sur `main`,
**Alors** le message du commit / PR contient `closes #27` (ou `fixes #27`) et l'issue GitHub bascule en `closed` automatiquement. **Aucune** édition de `docs/known-failures.md` (archivé depuis 2026-04-18).

### AC 11 — Code quality

**Étant donné** les modifications,
**Quand** je lance la batterie pré-PR,
**Alors** :
- `cd frontend && npm run check` ✅ (svelte-check 0 erreur)
- `cd frontend && npm run build` ✅
- `cd frontend && npm run test:unit` ✅ (régressions vitest)
- `cargo fmt --all -- --check` ✅, `cargo clippy --all-targets --all-features -- -D warnings` ✅ (si touché côté Rust — improbable mais à vérifier).

---

## Tasks / Subtasks

- [ ] **T1 — Baseline & inventaire** (AC #1, AC #2)
  - [ ] T1.1 Démarrer backend local : `KESH_TEST_MODE=true KESH_HOST=127.0.0.1 KESH_STATIC_DIR=../frontend/build cargo run -p kesh-api`
  - [ ] T1.2 Build frontend statique : `cd frontend && npm run build`
  - [ ] T1.3 Lancer `npm run test:e2e -- --reporter=list 2>&1 | tee tests/e2e/baseline-pre-7-5.log`
  - [ ] T1.4 Lancer `node scripts/audit-e2e-selectors.js > tests/e2e/audit-pre-7-5.txt`
  - [ ] T1.5 Classer chaque échec : *strict-mode violation* (in scope) vs *autre* (créer issue GitHub si nécessaire).
  - [ ] T1.5.bis Vérifier explicitement `auth.spec.ts:34 getByText('Kesh')` : exécuter `npm run test:e2e -- auth.spec.ts --reporter=list` et confirmer **0** violation strict-mode sur la page `/login`. Si violation détectée → ajouter T13bis dans la story (remplacer par `getByRole('heading', { name: 'Kesh', level: 1 })` ou `data-testid="login-logo"` sur le logo de la page de login). Si aucune violation → ce HIGH reste accepté tel quel pour AC #6 ≤ 9.
  - [ ] T1.6 Documenter le baseline dans la section *Dev Agent Record / Baseline*.

- [ ] **T2 — Composants Svelte : ajouter `data-testid`** (AC #3)
  - [ ] T2.1 `accounts/+page.svelte` (ou équivalent) : `account-table`, `account-row-{number}`, `account-create-button`, `account-edit-button`, `account-archive-toggle`, `account-create-banner`.
  - [ ] T2.2 `contacts/+page.svelte` + composants liés : `contact-table`, `contact-row-{id}`, `contact-create-button`, `contact-archive-button`, `contact-archive-dialog`, `contact-type-filter`.
  - [ ] T2.3 `products/+page.svelte` + composants liés : `product-table`, `product-row-{slug-or-id}`, `product-create-button`, `product-edit-button`, `product-archive-button`.
  - [ ] T2.4 `invoices/+page.svelte` + `InvoiceForm.svelte` (compléments) : `invoice-table`, `invoice-row-{number}`, `invoice-create-button`, `invoice-validate-button`, `invoice-pdf-link`. *(InvoiceForm a déjà `invoice-config-warning`, `invoice-line-vat-rate`, `create-invoice-button`.)*
  - [ ] T2.5 `journal-entries/+page.svelte` + `JournalEntryForm.svelte` : `journal-entry-table`, `journal-entry-row-{id}`, `journal-entry-create-button`, `journal-entry-line-debit-input`, `journal-entry-line-credit-input`, `journal-entry-validate-button`.
  - [ ] T2.6 `homepage-settings/+page.svelte` (ou équivalent layout/sidebar) : `homepage-card-recent-entries`, `homepage-card-open-invoices`, `homepage-card-bank-accounts`, `sidebar-link-organization`, `sidebar-link-accounting`, etc. — vérifier le layout réel avant de nommer.
  - [ ] T2.7 `vat-rates.spec.ts` n'a **pas** de page dédiée `/settings/vat-rates` — les selects de taux TVA sont dans `frontend/src/routes/(app)/products/+page.svelte` (select TVA produit) et dans `frontend/src/lib/components/invoices/InvoiceForm.svelte` (testid `invoice-line-vat-rate` déjà présent). Inventorier les 3 `getByRole` MEDIUM des lignes 34/70/80 du spec et instrumenter les sélecteurs concernés (préférer un id HTML existant `#form-vat-rate` si déjà présent ou ajouter `data-testid="product-vat-rate-select"`).
  - [ ] T2.8 `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` (route `/invoices/due-dates`, *pas* `echeancier`) : `echeancier-table`, `echeancier-row-{invoice-number}`, `mark-paid-button`, `mark-paid-dialog`, `mark-paid-confirm`.
  - [ ] T2.9 Mode-expert : la bascule est un `DropdownMenu.Item` inline dans `frontend/src/routes/(app)/+layout.svelte` (l. 134), *pas* un composant séparé. Le store est `frontend/src/lib/app/stores/mode.svelte.ts`. Ajouter `data-testid="mode-toggle-button"` sur le `<Button>` qui ouvre le dropdown et `data-testid="mode-toggle-{guided|expert}"` sur les `DropdownMenu.Item`. Le spec `mode-expert.spec.ts` cherche `button:has-text("Mode")` — le testid sur le bouton parent suffira.
  - [ ] T2.10 `onboarding-path-b` : composants spécifiques au flux Path B (langue comptable, coordonnées, compte bancaire). **Bannière "Configuration incomplète" ≠ `invoice-config-warning`** — c'est `frontend/src/lib/shared/components/IncompleteBanner.svelte`, voir T11.2.
  - [ ] T2.11 Vérifier qu'aucun `data-testid` existant (Story 7-6 / 3-7) n'est cassé ou renommé.

- [ ] **T3 — Refactor `accounts.spec.ts`** (AC #4, AC #5)
  - [ ] T3.1 Remplacer `getByText('Plan comptable')` (l. 35) → `getByRole('heading', { name: 'Plan comptable', level: 1 })` (un seul `h1` par page, sémantique a11y-friendly, pas besoin de `data-testid` sur les headings).
  - [ ] T3.2 Remplacer `getByText('1000')`, `getByText('2000')` (l. 42-43) → `[data-testid="account-row-1000"]`.
  - [ ] T3.3 Remplacer `getByText('Actif').first()`, `Passif`.first() (l. 50-51) → utiliser le badge avec testid (`[data-testid="account-row-1000"] [data-testid="account-type-badge"]`).
  - [ ] T3.4 Remplacer `getByText('Nouveau compte')` → `[data-testid="account-create-button"]`.
  - [ ] T3.5 Remplacer `getByText('Compte ${testNumber} créé')` → toast assertion via `[data-testid="toast-success"]` ou regex sur `getByRole('alert')`.
  - [ ] T3.6 Remplacer `getByText('Afficher les archivés')` (l. 106) → `[data-testid="show-archived-toggle"]`.
  - [ ] T3.7 Lancer `npm run test:e2e -- accounts.spec.ts` → 8/8 verts.

- [ ] **T3bis — Compléter refactor `users.spec.ts`** (AC #4, AC #5, AC #6) — *complète Story 7-6 (3 testids existants à NE PAS casser : `user-table`, `user-row-{username}`, `current-user-badge`)*
  - [ ] T3bis.1 Remplacer `sidebar.getByText('Utilisateurs')` (l.41) → `[data-testid="nav-link-users"]`. **Dépendance d'ordre :** T9.2 (instrumentation des `<a>` de la sidebar inline dans `(app)/+layout.svelte`) doit être terminée AVANT T3bis.1, sinon le testid n'existe pas et le test échoue. Si T3bis est exécuté avant T9, faire T9.2 d'abord (≈ 5 lignes Svelte) puis revenir à T3bis.1.
  - [ ] T3bis.2 Remplacer `getByText('Nouvel utilisateur')` (l.61, l.80, l.94) → `[data-testid="user-create-button"]` — ajouter le testid sur le bouton dans `frontend/src/routes/(app)/users/+page.svelte`.
  - [ ] T3bis.3 Remplacer `getByText('Créez un nouveau compte')` / autres titres dialog (l.62) → `[data-testid="user-create-dialog"]` parent + scoping interne via `getByRole('button', ...)` non ambigu.
  - [ ] T3bis.4 Remplacer les `getByText` restants (l.88, l.101, l.110) selon leur rôle (toast, label de table, bouton secondaire) — préférer testid si récurrent, sinon `getByRole` spécifique.
  - [ ] T3bis.5 Préserver intacts les 3 testids posés en Story 7-6 sur `users/+page.svelte` (vérifier exhaustivement par `grep -n "data-testid" frontend/src/routes/(app)/users/+page.svelte` avant édition) : `user-table`, `user-row-{user.username}`, `current-user-badge`. Si un autre testid `user-*` est introduit en T3bis, l'ajouter à la liste mais sans casser ces 3 existants.
  - [ ] T3bis.6 Lancer `npm run test:e2e -- users.spec.ts` → 100% non-skip verts.

- [ ] **T4 — Refactor `contacts.spec.ts`** *(important : seuls les `getByRole` réellement ambigus doivent être remplacés ; les `getByRole` déjà scoped via `row.getByRole(...)` ou `dialog.getByRole(...)` ne sont pas en violation strict-mode et peuvent rester si leur intention sémantique est claire)*
  - [ ] T4.1 Remplacer chaque `getByRole('button', { name: /Nouveau contact/ })` → `[data-testid="contact-create-button"]`.
  - [ ] T4.2 Remplacer `getByRole('heading', { name: /Carnet d'adresses/ })` → `getByRole('heading', { name: 'Carnet d\'adresses', level: 1 })` (un seul h1, pas besoin de testid).
  - [ ] T4.3 Pattern dialog d'archivage : passer de `page.locator('tr', { hasText: uniqueName })` + `getByRole('button', { name: /Archiver/ })` → `[data-testid="contact-row-{id}"] [data-testid="contact-archive-button"]` + `[data-testid="contact-archive-dialog"] [data-testid="contact-archive-confirm"]`. *(Garder `row.getByRole(...)` si la `row` est déjà scopée et l'action stable.)*
  - [ ] T4.4 Vérifier les 2 `test.skip` existants restent skipped (filtres combinés, pagination — Story 4.2 / post-MVP).
  - [ ] T4.5 Lancer `npm run test:e2e -- contacts.spec.ts` → 5+ verts (selon nb tests actifs hors skip).

- [ ] **T5 — Refactor `products.spec.ts`**
  - [ ] T5.1 Inventaire des `getByRole` ambigus (~31 occurrences) — repérer ceux qui matchent plusieurs éléments.
  - [ ] T5.2 Appliquer pattern table + row + bouton-d'action (cf. T3 / T4).
  - [ ] T5.3 Lancer `npm run test:e2e -- products.spec.ts` → 8/8 verts.

- [ ] **T6 — Refactor `invoices.spec.ts`**
  - [ ] T6.1 Inventaire ~25 `getByRole` + 1 `getByText('Prestation libre')`.
  - [ ] T6.2 Appliquer pattern table + ligne + form. Attention aux flux multi-étapes (création → validation → paiement).
  - [ ] T6.3 Lancer `npm run test:e2e -- invoices.spec.ts` → 8/8 verts.

- [ ] **T7 — Refactor `invoices_echeancier.spec.ts`**
  - [ ] T7.1 Replacer `getByRole('heading', { name: /Échéancier/i })` → testid sur le `h1` ou le conteneur.
  - [ ] T7.2 Stabiliser la séquence `Marquer payée` → dialog → `Confirmer` (regex multi-locale `/Marquer payée|Mark as paid|Segna.../`) — préférer `[data-testid="mark-paid-button"]` + `[data-testid="mark-paid-confirm"]` qui sont locale-agnostic.
  - [ ] T7.3 Lancer `npm run test:e2e -- invoices_echeancier.spec.ts` → 4/4 verts.

- [ ] **T8 — Refactor `journal-entries.spec.ts`** (le plus volumineux : ~45 `getByRole`)
  - [ ] T8.1 Décomposer par bloc `test.describe` et identifier les groupements de selectors répétés (lignes de débit/crédit, bouton ajouter ligne, bouton enregistrer).
  - [ ] T8.2 Pattern : `[data-testid="journal-entry-line-{n}-debit"]` / `-credit` / `-account` pour chaque ligne. Compteur `n` 0-indexé, généré par la boucle Svelte.
  - [ ] T8.3 Bouton submit : `[data-testid="journal-entry-submit"]`.
  - [ ] T8.4 Lancer `npm run test:e2e -- journal-entries.spec.ts` → 8/8 verts.

- [ ] **T9 — Refactor `homepage-settings.spec.ts`**
  - [ ] T9.1 Cards d'accueil : `[data-testid="homepage-card-recent-entries"]`, etc.
  - [ ] T9.2 Sidebar : la nav latérale est inline dans `frontend/src/routes/(app)/+layout.svelte` (l. 164), *pas* un composant `Sidebar.svelte` séparé. Les `<a href="/...">` sont générés depuis le tableau `navGroups` inline. Ajouter `data-testid="nav-link-{slug}"` (slug = derniè partie du href : `organization`, `accounting`, `bank-accounts`, `users`, etc.) directement sur les `<a>`.
  - [ ] T9.3 Lancer `npm run test:e2e -- homepage-settings.spec.ts` → 4/4 verts.

- [ ] **T10 — Refactor `mode-expert.spec.ts`**
  - [ ] T10.1 Identifier la bascule mode guidé/expert et lui donner un testid stable.
  - [ ] T10.2 Lancer `npm run test:e2e -- mode-expert.spec.ts` → 2/2 verts.

- [ ] **T11 — Refactor `onboarding-path-b.spec.ts`**
  - [ ] T11.1 Étapes du flux : `[data-testid="onboarding-step-{slug}"]` (independant, langue-comptable, coordonnees, compte-bancaire).
  - [ ] T11.2 Bannière "Configuration incomplète" rendue dans `(app)/+layout.svelte` (l.158) via `frontend/src/lib/shared/components/IncompleteBanner.svelte` : ajouter `data-testid="incomplete-config-banner"` sur le `<div role="status">` (l. 9–11). Remplacer `getByText('Configuration incomplète')` par `[data-testid="incomplete-config-banner"]` dans `onboarding-path-b.spec.ts` (l. 57, l. 78). **Ne PAS confondre avec `invoice-config-warning`** (testid distinct dans `InvoiceForm.svelte`, contexte création de facture, jamais rendu sur `/`).
  - [ ] T11.3 Lancer `npm run test:e2e -- onboarding-path-b.spec.ts` → tests verts.

- [ ] **T12 — Refactor `fiscal-years.spec.ts`**
  - [ ] T12.1 Vérifier que les 3 testids ajoutés par Story 3-7 (`fiscal-year-create-button`, `fiscal-year-table`, `fiscal-year-row-{id}`) sont effectivement utilisés. Compléter avec `fiscal-year-rename-button`, `fiscal-year-close-button`, `fiscal-year-confirm-close-button`.
  - [ ] T12.2 Garder le `test.skip` honnête de KF-019 (issue #47) inchangé.
  - [ ] T12.3 Lancer `npm run test:e2e -- fiscal-years.spec.ts` → tests non-skip verts.

- [ ] **T13 — Refactor `vat-rates.spec.ts`** *(refactor conditionnel — n'agir que sur les sélecteurs effectivement en violation strict-mode au baseline T1)*
  - [ ] T13.1 Inventorier les ~6 `getByRole` MEDIUM (l. 34/70/80) du spec et **vérifier dans le baseline T1** si l'un d'entre eux cause une violation strict-mode. Le spec utilise déjà `getByTestId('invoice-line-vat-rate')` (l. 52, posé Story 7-6).
  - [ ] T13.2 Si zéro violation → ne rien refactorer (les `getByRole` non-ambigus restent tels quels, cf. story intro). Si violations → instrumenter les selects TVA dans `frontend/src/routes/(app)/products/+page.svelte` (`#form-vat-rate` ou ajouter `data-testid="product-vat-rate-select"`) et mettre à jour le spec.
  - [ ] T13.3 Lancer `npm run test:e2e -- vat-rates.spec.ts` → tests verts (skips honnêtes admis).

- [ ] **T14 — Validation globale & audit final** (AC #4, AC #5, AC #6)
  - [ ] T14.1 Suite complète : `npm run test:e2e -- --reporter=list 2>&1 | tee tests/e2e/baseline-post-7-5.log`. **0 failed**, X passed, Y skipped (Y = nb test.skip honnêtes pré-existants).
  - [ ] T14.2 Vérifier `grep -c "strict mode violation" tests/e2e/baseline-post-7-5.log` → **0**.
  - [ ] T14.3 Re-lancer audit : `node scripts/audit-e2e-selectors.js > tests/e2e/audit-post-7-5.txt`. Comparer compteurs HIGH/MEDIUM avant/après.
  - [ ] T14.4 Documenter trend dans Change Log : `192 brittle (41H+151M) → N (xH+yM)`.
  - [ ] T14.5 **Détection `.first()` / `.nth(N)` introduits sans justification** (vérification AC #5) : `git diff origin/main -- "frontend/tests/e2e/**/*.spec.ts" | grep -E "^\+.*\.(first|nth)\("` → toute occurrence ajoutée doit être suivie d'un commentaire `// strict-mode safe: ...` justifiant la sémantique « première ligne quelconque ». Lister les exceptions dans le Change Log.

- [ ] **T15 — Documentation `E2E_TESTING_BEST_PRACTICES.md`** (AC #8)
  - [ ] T15.1 Étendre la section *Examples by Feature* avec accounts, contacts, products, invoices, journal-entries, fiscal-years, vat-rates, homepage-settings.
  - [ ] T15.2 Ajouter une section *Naming reference* listant la convention par entité (table / row / button / input / banner / dialog).
  - [ ] T15.3 Documenter le compteur baseline → final.
  - [ ] T15.4 Si helper `byTestId` introduit (T16), documenter son usage et le comparer au pattern direct.
  - [ ] T15.5 Corriger les passages où `E2E_TESTING_BEST_PRACTICES.md` qualifie `.first()` d'« also acceptable » (≈ l. 63 et 79) : aligner sur AC #5 — `.first()` n'est pas un contournement acceptable du strict mode, seulement une sémantique légitime (« première ligne quelconque »), à commenter `// strict-mode safe: ...`.

- [ ] **T16 — Helper `byTestId` (optionnel)** (AC #7)
  - [ ] T16.1 Si un helper réduit clairement la duplication (≥ 20 sites d'appel), le créer dans `frontend/tests/e2e/helpers/selectors.ts` :
    ```ts
    export const byTestId = (page: Page, id: string) => page.locator(`[data-testid="${id}"]`);
    ```
  - [ ] T16.2 Sinon, conserver le pattern direct `page.locator('[data-testid="..."]')` et documenter ce choix dans T15.

- [ ] **T17 — Code quality & PR** (AC #10, AC #11)
  - [ ] T17.1 `cd frontend && npm run check && npm run build && npm run test:unit` ✅
  - [ ] T17.2 `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings` ✅ (si Rust touché — improbable).
  - [ ] T17.3 Commit local sur branche `story/7-5-kf-008-playwright-selector-fixes` avec message terminé par `closes #27`.
  - [ ] T17.4 Push à la demande de Guy (cf. règle commit/push CLAUDE.md — pas d'auto-push).
  - [ ] T17.5 Mettre à jour `_bmad-output/implementation-artifacts/sprint-status.yaml` : `7-5-kf-008-playwright-selector-fixes: ready-for-dev` → `review` après implémentation.

---

## Dev Notes

### Architecture E2E (rappel)

- **Stack tests :** Playwright `^1.59.1` + `@axe-core/playwright`.
- **Config :** `frontend/playwright.config.ts` — `workers: 1`, `baseURL: http://127.0.0.1:3000` (le backend Rust sert aussi la SPA via `KESH_STATIC_DIR`), locale `fr-CH`, TZ `Europe/Zurich`.
- **Helpers :** `frontend/tests/e2e/helpers/test-state.ts` exporte `seedTestState(preset)` (`fresh` | `post-onboarding` | `with-company` | `with-data`) et `clearAuthStorage(page)`.
- **Pattern login :** `page.goto('/login')` → `page.fill('#username', 'admin')` + `'#password', 'admin123'` → `page.click('button[type="submit"]')` → `expect(page).toHaveURL('/')`.
- **Isolation :** chaque spec appelle `seedTestState(...)` en `beforeAll`, et `clearAuthStorage(page)` en `afterEach` (depuis Story 6-5 qui a closed KF-007).

### Convention de nommage `data-testid`

Le standard imposé par Story 7-6 (cf. `frontend/docs/E2E_TESTING_BEST_PRACTICES.md`) :

- **Format :** `kebab-case`.
- **Pattern entité :** `<entity>-<element>[-<key>]`.
  - Conteneur liste : `<entity>-table` (singulier).
  - Ligne : `<entity>-row-{<key>}` où `{key}` est un identifiant naturel (number, slug, id court).
  - Bouton primaire : `<entity>-<action>-button` (`create`, `edit`, `archive`, `delete`, `validate`).
  - Champ : `<entity>-<field>-input` (préférer l'`id` HTML existant si disponible — pas de double).
  - Dialog : `<entity>-<action>-dialog` + `<entity>-<action>-confirm`.
  - Bannière / toast : `<entity>-<state>-banner` (warning, success, error).
- **Stabilité :** un testid ne doit JAMAIS contenir de texte traduit, de date ni d'identifiant aléatoire (ok pour la `key` Svelte de la ligne mais pas dans le nom du testid lui-même).
- **Choix de la `key` par entité :**
  - `accounts` : numéro de compte (ex. `account-row-1000`) — clé naturelle comptable, stable, lisible.
  - `contacts`, `products`, `journal-entries`, `invoices`, `users` : `id` DB ou `slug` selon ce qui existe dans la fixture (`with-company` / `with-data`).
  - `vat-rates` : `code` (ex. `vat-rate-row-tva-8-1`) si présent, sinon `id`.
  - Single-tenant garanti par Story 7-1 (KF-002) — pas de risque de collision cross-company sur ces keys.
- **Placement :** sur l'élément interactif (button, input, td, tr) ou le conteneur principal (table, dialog).

### Stratégie de remédiation strict-mode

Trois patterns hiérarchisés :

1. **PRÉFÉRÉ — `data-testid` explicite :**
   ```ts
   await expect(page.locator('[data-testid="contact-row-42"]')).toBeVisible();
   ```
2. **ACCEPTABLE — rôle + `data-testid` parent :**
   ```ts
   await page
     .locator('[data-testid="contact-row-42"]')
     .getByRole('button', { name: /Archiver/ })
     .click();
   ```
3. **DÉCONSEILLÉ — `.first()` / `.nth(N)` :** uniquement si le test exerce volontairement la sémantique « première ligne quelconque » (et alors un commentaire `// strict-mode safe: ...` est obligatoire).

**Ce qu'il NE faut PAS faire :**

- Ajouter `strict: false` à la config Playwright → masque les vraies ambiguïtés.
- Concaténer plusieurs `getByText()` en chaîne pour disambiguer (lecture difficile, brittle).
- Renommer un testid existant (Story 7-6 / 3-7) — risque de casser les specs déjà refactorés.

### Mapping spec → composants Svelte (pour T2)

| Spec | Routes / composants principaux à instrumenter |
|---|---|
| `accounts.spec.ts` | `frontend/src/routes/(app)/accounts/+page.svelte` (formulaire et tableau **inline**, pas de composant `AccountForm.svelte` séparé) |
| `contacts.spec.ts` | `frontend/src/routes/(app)/contacts/+page.svelte` (formulaire et dialog d'archivage **inline**, pas de composant séparé) |
| `products.spec.ts` | `frontend/src/routes/(app)/products/+page.svelte` (formulaire **inline**) |
| `invoices.spec.ts` | `frontend/src/routes/(app)/invoices/**` + `InvoiceForm.svelte` (déjà partiellement instrumenté Story 7-6) |
| `invoices_echeancier.spec.ts` | `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` + dialog paiement |
| `journal-entries.spec.ts` | `frontend/src/routes/(app)/journal-entries/+page.svelte` + `JournalEntryForm.svelte` (à confirmer dans `src/lib/features/journal-entries/`) |
| `homepage-settings.spec.ts` | `frontend/src/routes/(app)/+page.svelte` (homepage cards) + nav latérale **inline** dans `frontend/src/routes/(app)/+layout.svelte` (pas de `Sidebar.svelte` séparé) |
| `vat-rates.spec.ts` | Pas de page dédiée — interactions sur `frontend/src/routes/(app)/products/+page.svelte` (select TVA produit) + `frontend/src/lib/components/invoices/InvoiceForm.svelte` (testid `invoice-line-vat-rate` posé Story 7-6) |
| `fiscal-years.spec.ts` | `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` (Story 3-7 — partiellement instrumenté) |
| `mode-expert.spec.ts` | Bascule **inline** dans `frontend/src/routes/(app)/+layout.svelte` (l. 134, `DropdownMenu.Item`) + store `frontend/src/lib/app/stores/mode.svelte.ts` |
| `onboarding-path-b.spec.ts` | `frontend/src/routes/onboarding/**` + bannière `frontend/src/lib/shared/components/IncompleteBanner.svelte` rendue dans `(app)/+layout.svelte` |

> Le dev doit vérifier les chemins réels avant T2 : la structure `frontend/src/lib/components/` et `frontend/src/lib/features/` peut différer entité par entité.

### Quelques pièges connus

- **i18n :** plusieurs tests utilisent regex multi-locale (`/Marquer payée|Mark as paid|Segna.../`). Avec `data-testid`, plus besoin — c'est l'un des bénéfices indirects.
- **Toast `svelte-sonner` :** typiquement `getByRole('alert')` — vérifier que le composant toast a bien un rôle ARIA `alert` ou ajouter `data-testid="toast-{type}"` sur l'élément racine.
- **Dialogs `bits-ui` :** ils exposent un `role="dialog"` natif. Filtrer par testid sur l'enfant (`[data-testid="contact-archive-dialog"] [data-testid="contact-archive-confirm"]`) est plus robuste que `page.getByRole('dialog').getByRole('button', { name: 'Archiver' })`.
- **Lignes de tableau dynamiques :** la `key` du `data-testid` doit venir d'un identifiant qui existe dans la fixture (`with-company` / `with-data`). Utiliser un compteur d'index (`row-{i}`) est OK si l'ordre est garanti par le seed.
- **Story 6-2 multi-tenant :** les fixtures `with-company` créent une seule company, donc pas de risque de collision testid cross-tenant — mais penser à scoper les keys si jamais une fixture multi-company est introduite (Story 6-2 / KF-002 audit).
- **Vat-rates DB-driven (Story 7-2) :** les taux TVA viennent maintenant de la DB. La fixture `seed_accounting_company` seed 4 taux Suisse (8.10 / 3.80 / 2.60 / 0.00 — exemption au taux 0 %). `vat-rates.spec.ts:38` attend `toHaveCount(4)`. Vérifier le seed avant de blâmer le sélecteur.

### Fichiers à NE PAS toucher

- `frontend/src/lib/auth.ts` (sauf si un bug auth ressort — improbable, KF-007 closed).
- `frontend/playwright.config.ts` (sauf nécessité absolue — `workers: 1` est intentionnel).
- `.github/workflows/ci.yml` (réintroduction E2E hors scope, future story).
- `docs/known-failures.md` (archivé 2026-04-18).
- `docs/change_request.md` (archivé 2026-04-16).

---

## Previous Story Intelligence

### Story 7-6 (E2E Selector Refactoring) — leçons clés

- **Pattern `data-testid` validé** : kebab-case, sémantique, sur conteneur ou interactif.
- **Audit script utile** : permet de mesurer le trend objectivement. À relancer en T1 et T14.
- **Best practices doc déjà en place** : ne pas réinventer, étendre seulement.
- **Refactor partiel** : seules `onboarding.spec.ts` (AC 5/6) et `users.spec.ts` ont été touchées. Les 9 testids existants (3 fichiers) ne couvrent qu'une fraction du besoin.
- **`getByRole` peut rester** quand non ambigu et accessibility-friendly. Seuls les `getByText` brittle et les `getByRole` qui matchent plusieurs éléments doivent partir.

### Story 6-5 (Fix Playwright E2E auth) — leçons clés

- **`clearAuthStorage(page)` en `afterEach` est obligatoire** dans toutes les specs : sans lui, les tokens fuient entre tests. Pattern déjà appliqué partout en `tests/e2e/helpers/test-state.ts`. Ne pas casser.
- **`localStorage` (clé `kesh:auth:accessToken` / `kesh:auth:refreshToken`)** est la source de vérité auth, pas les cookies.
- **Local-first debugging** : Playwright `--debug` + DevTools → souvent plus rapide que d'itérer en CI. Cette story se fait 100 % en local.
- **Pre-existing CI failure** : avant que KF-007 soit fix, tous les tests échouaient avec `401`. Aujourd'hui, avec auth qui marche, les seuls échecs restants sont strict-mode + edge cases.

### Story 6-4 (Fixtures E2E déterministes) — leçons clés

- **`seedTestState(preset)` + endpoint `/api/v1/_test/seed`** > seed inline SQL. Pattern à conserver sans changement.
- **`workers: 1`** est nécessaire car DB partagée. Ne pas toucher.
- **`KESH_TEST_MODE=true`** gate les endpoints `_test/*` et refuse les binds non-loopback. Pré-requis pour tout dev local.

### Story 7-2 (KF-003 TVA DB-driven)

- Les taux TVA sont en DB (table `vat_rates`). Si `vat-rates.spec.ts` a 5 tests dont ~2 en échec, vérifier que la fixture les seed correctement avant de blâmer le selector.

### Story 7-1 (KF-002 Multi-Tenant Audit)

- Aucune incidence directe, mais le scoping multi-tenant garantit que les fixtures `with-company` n'ont qu'une company. Les testids `<entity>-row-{id}` sont sûrs sans préfixage tenant.

---

## Testing Strategy

### Avant patch (T1)

```bash
# Terminal 1 — backend
export KESH_TEST_MODE=true
export KESH_HOST=127.0.0.1
export KESH_STATIC_DIR=../frontend/build
export DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh
cd crates/kesh-api && cargo run

# Terminal 2 — frontend build + tests
cd frontend
npm run build
npm run test:e2e -- --reporter=list 2>&1 | tee tests/e2e/baseline-pre-7-5.log
node scripts/audit-e2e-selectors.js > tests/e2e/audit-pre-7-5.txt
```

### Pendant patch (T3-T13)

- Spec par spec : `npm run test:e2e -- <spec>.spec.ts --reporter=list` après chaque refactor.
- `npm run check` après chaque ajout de `data-testid` côté Svelte (Svelte parse les attrs HTML mais svelte-check valide la syntaxe).
- Commit BMAD par étape (cf. règle CLAUDE.md « Commit systématique »).

### Après patch (T14)

```bash
npm run test:e2e -- --reporter=list 2>&1 | tee tests/e2e/baseline-post-7-5.log
grep -c "strict mode violation" tests/e2e/baseline-post-7-5.log  # → 0
node scripts/audit-e2e-selectors.js > tests/e2e/audit-post-7-5.txt
diff tests/e2e/audit-pre-7-5.txt tests/e2e/audit-post-7-5.txt
```

### Régression

- `auth.spec.ts` 4/4 verts (intouchés).
- `onboarding.spec.ts` 3+/3+ verts (refactorés en 7-6, intouchés ici).
- `users.spec.ts` partiellement refactoré en 7-6 (3 testids existants `user-table` / `user-row-{username}` / `current-user-badge`) — refactor actif via T3bis (cf. AC #9 note de scope), pas une simple vérification de régression.
- Tests unitaires : `npm run test:unit` 0 régression.
- Build : `npm run build` 0 erreur.

### Critères « done » avant code review

- AC #1 à #11 tous ✅.
- Sprint-status mis à jour : `7-5-kf-008-playwright-selector-fixes: review`.
- Branch poussée à la demande de Guy.

---

## Références

### GitHub
- Issue [#27 — KF-008](https://github.com/guycorbaz/kesh/issues/27)
- Story 7-6 (sans PR séparée) : artefacts livrés dans la PR Story 6-2 [#29](https://github.com/guycorbaz/kesh/pull/29) (commit `7c8822d`)
- PR Story 6-5 : [#28](https://github.com/guycorbaz/kesh/pull/28) (commit `765af6d`)
- PR Story 6-4 : [#18](https://github.com/guycorbaz/kesh/pull/18) (commit `520e8df`)

### Fichiers clés
- `frontend/playwright.config.ts`
- `frontend/tests/e2e/helpers/test-state.ts`
- `frontend/scripts/audit-e2e-selectors.js`
- `frontend/docs/E2E_TESTING_BEST_PRACTICES.md`
- `frontend/DEBUGGING-KF007.md`
- `_bmad-output/implementation-artifacts/7-6-e2e-selector-refactoring.md`
- `_bmad-output/implementation-artifacts/6-5-fix-playwright-e2e-auth-flow.md`
- `_bmad-output/implementation-artifacts/6-4-fixtures-e2e-deterministes.md`

### Specs Playwright concernées
- `frontend/tests/e2e/accounts.spec.ts`
- `frontend/tests/e2e/contacts.spec.ts`
- `frontend/tests/e2e/products.spec.ts`
- `frontend/tests/e2e/invoices.spec.ts`
- `frontend/tests/e2e/invoices_echeancier.spec.ts`
- `frontend/tests/e2e/journal-entries.spec.ts`
- `frontend/tests/e2e/homepage-settings.spec.ts`
- `frontend/tests/e2e/mode-expert.spec.ts`
- `frontend/tests/e2e/onboarding-path-b.spec.ts`
- `frontend/tests/e2e/fiscal-years.spec.ts`
- `frontend/tests/e2e/vat-rates.spec.ts`

### Doc upstream
- Playwright locators best practices : https://playwright.dev/docs/locators
- Playwright strict mode : https://playwright.dev/docs/api/class-locator#locator-error-strict-mode-violation

---

## Dev Agent Record

### Agent Model Used

_(à remplir par bmad-dev-story)_

### Baseline pré-implémentation (T1)

_(à remplir par le dev — tableau récapitulatif des passed / failed / skipped et liste des tests en échec catégorisés strict-mode vs autre)_

### Plan de refactor (T2)

_(à remplir : composants Svelte instrumentés et testids ajoutés)_

### Debug Log References

_(à remplir)_

### Completion Notes List

_(à remplir)_

### File List

_(à remplir : liste exhaustive des fichiers Svelte instrumentés + specs refactorés + docs mis à jour)_

---

## Change Log

### 2026-04-30 — Story Creation
- **Status :** ready-for-dev
- **Auteur :** bmad-create-story (`/bmad-create-story 7-5`)
- **Sources analysées :**
  - GitHub issue #27 (KF-008 body)
  - `_bmad-output/implementation-artifacts/7-6-e2e-selector-refactoring.md` (pattern `data-testid` + audit script)
  - `_bmad-output/implementation-artifacts/6-5-fix-playwright-e2e-auth-flow.md` (auth flow + helpers)
  - `_bmad-output/implementation-artifacts/6-4-fixtures-e2e-deterministes.md` (seedTestState + workers=1)
  - `frontend/docs/E2E_TESTING_BEST_PRACTICES.md`
  - `frontend/scripts/audit-e2e-selectors.js` + run live (192 brittle selectors : 41 HIGH + 151 MEDIUM)
  - Inventaire des 14 specs E2E existantes
  - `frontend/playwright.config.ts`
- **Décisions de scope :**
  - INCLUS : refactor des 11 specs en échec strict-mode (T3–T13 + T3bis users.spec.ts) + instrumentation `data-testid` des composants ciblés + extension doc + audit final.
  - EXCLU : réintroduction job E2E en CI, multi-worker, refactor des 151 `getByRole` non-ambigus, fix KF-019 (#47 — gap couverture AC #22 fiscal-years), refactor partiel restant `onboarding.spec.ts` (8 HIGH hors scope, à traiter en story dette technique séparée).
- **Cible numérique :** 0 strict-mode violation, ≥ 36 tests recouverts, audit `getByText` HIGH ≤ 9 (cf. AC #6 — 8 résiduels onboarding.spec.ts hors scope + 1 auth.spec.ts:34 `Kesh` accepté si non-violant).
- **Next :** `/bmad-dev-story 7-5` pour implémentation.

### 2026-04-30 — Spec Validate Loop (5 passes)
- **Workflow :** `/bmad-create-story validate` — boucle multi-passes adversariale (CLAUDE.md « Règle de remédiation des revues »).
- **Trend numérique :**
  - **Pass 1 (Sonnet)** — 14 findings : 2 CRITICAL + 4 HIGH + 5 MEDIUM + 3 LOW.
  - **Pass 2 (Haiku)** — 7 findings : 0 CRITICAL + 2 HIGH + 3 MEDIUM + 2 LOW.
  - **Pass 3 (Opus)** — 9 findings : 0 CRITICAL + 2 HIGH + 3 MEDIUM + 4 LOW.
  - **Pass 4 (Sonnet)** — 4 findings : 0 CRITICAL + 0 HIGH + 1 MEDIUM + 3 LOW.
  - **Pass 5 (Haiku — closure check)** — 0 findings. **Critère d'arrêt CLAUDE.md atteint** (0 finding > LOW).
- **Cycle LLM utilisé :** Opus (création) → Sonnet (P1) → Haiku (P2) → Opus (P3) → Sonnet (P4) → Haiku (P5). Chaque passe en contexte frais, patches appliqués avant la suivante.
- **Patches appliqués (résumé global) :**
  - **Scope élargi (Option A choisie par Guy)** : ajout T3bis pour refactor partiel `users.spec.ts` (8 HIGH lignes 41/61/62/80/88/94/101/110), ajustement AC #6 threshold à `≤ 9` (vs `≤ 5` initial), narrow AC #9 pour retirer `users.spec.ts` du périmètre « régression intouchée ».
  - **Chemins composants corrigés** : `invoices/echeancier/` → `invoices/due-dates/` (T2.8) ; `/settings/vat-rates/` n'existe pas → selects TVA inline dans `/products` + `InvoiceForm` (T2.7) ; mode-expert toggle inline dans `(app)/+layout.svelte` l. 134, pas un composant séparé (T2.9) ; sidebar inline dans `(app)/+layout.svelte` l. 164, pas de `Sidebar.svelte` (T9.2) ; bannière « Configuration incomplète » = `IncompleteBanner.svelte` (`lib/shared/components/`), **pas** `invoice-config-warning` (T11.2) ; mapping table nettoyée des `AccountForm`/`ContactForm`/`ProductForm`/`ContactArchiveDialog.svelte` inexistants — formulaires inline dans `+page.svelte`.
  - **Compteurs scope table corrigés** : homepage-settings 7 HIGH | 0 MEDIUM (vs 6 | ~5) ; contacts 0 | 16 (vs 0 | 12) ; mode-expert 0 | 0\* avec note CSS selector ; vat-rates 0 | 3 (vs 0 | ~6) ; total tests existants ~76 (vs ~70). « 10 specs » → « 11 specs ».
  - **Provenance Story 7-6 corrigée** : pas de PR distincte ; artefacts livrés via PR #29 / commit `7c8822d` qui adresse principalement Story 6-2 multi-tenant (Pass 3 Opus a vérifié via `gh pr view 29`).
  - **Compteur testids users.spec.ts corrigé** : 5 → 3 testids (3 endroits) avec liste explicite (`user-table` l. 284, `user-row-{user.username}` l. 296, `current-user-badge` l. 300).
  - **AC #6 math clarifiée** : 41 HIGH baseline − 24 fixés (T3-T13 hors auth) − 8 fixés (T3bis users) = 9 résiduels = 8 onboarding + 1 auth. Note exit code `audit-e2e-selectors.js`.
  - **AC #4 hardening anti-loophole** : interdit explicitement l'ajout de nouveaux `test.skip`, baseline cité (9 honnêtes : fiscal-years × 1 KF-019, contacts × 2, journal-entries × 6).
  - **AC #5 validation** : T14.5 ajoutée — `git diff` détecte les `.first()`/`.nth()` introduits sans commentaire `// strict-mode safe`. T15.5 corrige la contradiction dans `E2E_TESTING_BEST_PRACTICES.md` (≈ l. 63 et 79).
  - **T1.5.bis ajouté** : vérification explicite que `auth.spec.ts:34 getByText('Kesh')` n'introduit pas de violation strict-mode latente sur `/login`.
  - **T13 vat-rates conditionnel** : refactor uniquement si baseline T1 montre violation strict-mode ; cohérent avec la règle « ne pas refactorer les `getByRole` non-ambigus ».
  - **Convention naming par entité** : ajout d'une section *Choix de la `key` par entité* (numéro pour comptes, id/slug pour autres) en Dev Notes.
  - **Numéros de ligne consolidés** : T13.1 `vat-rates.spec.ts` l. 34/70/80 (Pass 3 a corrigé l. 81 → 80).
  - **Fixture vat_rates** : 4 taux (8.10/3.80/2.60/0.00 — exemption 0%), pas 3.
- **Reclassements / dette technique :** aucun finding > LOW reclassé en dette persistante. Les 8 HIGH résiduels d'`onboarding.spec.ts` étaient déjà documentés hors scope dès la création (Story 7-6 n'a refactoré que AC 5/6) ; ils restent à traiter en story dette technique future.
- **Verdict final :** spec convergente, prête pour `bmad-dev-story 7-5`. Cohérence end-to-end vérifiée par Pass 5 (Haiku) — coherent ACs, sequenced tasks with explicit dependencies (T9.2 → T3bis.1), prior-story intelligence intégrée correctement.

