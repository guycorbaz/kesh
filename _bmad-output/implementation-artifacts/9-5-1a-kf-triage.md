# Story 9.5-1a: KF triage rapide — re-test + closures résolues

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a mainteneur projet Kesh,
I want exécuter individuellement chaque KF Epic 7/9 ouverte (#47, #50, #54, #55, #57, #91) avec son test E2E ou unit correspondant, documenter l'état réel observé (encore reproductible / résolue par effet de bord / partiellement résolue), fermer celles qui passent maintenant via `closes #N` dans le commit message, puis arbitrer le scope précis des sous-stories suivantes 9-5-1b (E2E infra), 9-5-1c (a11y), 9-5-1d (KFs spécifiques),
so that le périmètre d'implémentation des fixes restants est cadré, le backlog GitHub Issues reflète la réalité du code post-Epic 9, et le cycle d'Epic 9.5 progresse sur des sous-stories scopées au strict nécessaire (anti pattern Story 7-1 historique 4 passes validate sur scope cross-cutting).

## Scope

Story **focused sur le triage** (re-exécution de tests existants + tracking issues), pas sur l'implémentation de fixes complexes. **Story low-code par construction** — touche au plus :

- **Code production** : 0 fichier modifié (cette story ne fixe rien — le fix éventuel se fait en 9-5-1b/c/d).
- **Tests** : 0 nouveau test ajouté. La story consiste à **exécuter** les tests E2E + cargo tests existants pour chaque KF.
- **GitHub Issues** : 0 à 6 issues fermées via `closes #N` dans le commit final (selon nb de KFs résolues par effet de bord).
- **Documents projet** :
  - `_bmad-output/implementation-artifacts/sprint-status.yaml` — mise à jour scope précis des placeholders 9-5-1b/c/d après triage.
  - `_bmad-output/planning-artifacts/epic-9-5.md` — mise à jour scope définitif 9-5-1b/c/d basé sur les KFs résiduelles.
  - `_bmad-output/implementation-artifacts/9-5-1a-kf-triage.md` (cette spec) — Change Log final avec résultats triage par KF.

**Périmètre tests à ré-exécuter** :

| KF GitHub | Test E2E ou unit à lancer | Statut hypothèse |
|---|---|---|
| #47 KF-019 (Story 3-7 AC#22 fallback toast) | `npm run test:e2e -- fiscal-years.spec.ts` (cherche `test.skip` AC #22) | Encore active (skip statique) |
| #50 KF-021 (AC#29 race REPEATABLE READ no-op) | `cargo test --workspace -p kesh-api --test kf004_no_op_e2e -- --test-threads=1` | Encore active (smoke OK mais ne détecte pas race) |
| #54 KF-022 (cascade 401 helpers) | `npm run test:e2e -- invoices.spec.ts invoices_echeancier.spec.ts journal-entries.spec.ts` | Probablement encore active |
| #55 KF-023 (axe-core 6 pages a11y) | `npm run test:e2e -- auth.spec.ts contacts.spec.ts homepage-settings.spec.ts invoices.spec.ts products.spec.ts` (filtrer tests axe-core) | Probablement encore active |
| #57 KF-025 (state/timing/redirect dispersés) | `npm run test:e2e -- fiscal-years.spec.ts mode-expert.spec.ts onboarding.spec.ts onboarding-path-b.spec.ts homepage-settings.spec.ts users.spec.ts journal-entries.spec.ts` | Probablement encore active |
| #91 KF-027 (DropdownMenu reports a11y) | `npm run test:e2e -- reports.spec.ts` (filter `a11y violations`) | Encore active (reproduit sur main `6495731`) |

**Hors scope 9-5-1a** :
- **Aucun fix** appliqué dans cette story. Tout fix non-trivial est explicitement re-classé dans 9-5-1b/c/d après triage.
- **Cas limite** : si une KF passe et le « fix » est en réalité un changement de commentaire ou de skip annotation (~1-2 lignes triviales), la fermer dans 9-5-1a uniquement si le diff total reste < 10 lignes. Au-delà → 9-5-1b/c/d selon nature.
- Pas de migration de tests (`test.skip` → `test.only` ou inverse) — délégué aux sous-stories.
- Pas de refactor de helpers (`test-state.ts`) — c'est explicitement le scope de 9-5-1b.
- Pas de patch de composants Svelte (DropdownMenu wrapper) — scope 9-5-1c.

## Acceptance Criteria

### Pré-flight environnement

1. **Given** un workspace Kesh à jour avec `main` `35344c9` + branche `chore/epic-9-5-planning` checkée, **When** le triage démarre, **Then** prérequis CI confirmés : `cargo build --workspace` clean, `cd frontend && npm install && npm run build` clean, MariaDB démarré + seed CI appliqué, Playwright browsers installés via `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npx playwright install` (cf. memory `reference_playwright_ubuntu26`).

### Re-test par KF (6 ACs individuels)

2. **Given** KF #47 (Story 3-7 AC#22 fallback toast), **When** `npm run test:e2e -- fiscal-years.spec.ts` est exécuté avec capture du reporter Playwright HTML, **Then** documenter dans Change Log :
   - Statut `test.skip` dans `frontend/tests/e2e/fiscal-years.spec.ts` (block `describe('AC #22 — fallback toast actionnable')`) : présent / supprimé / modifié.
   - Tests réellement exécutés vs skippés.
   - **Décision** : si `test.skip` toujours présent ET aucun test AC #22 réel n'existe → KF encore active → route vers 9-5-1d.

3. **Given** KF #50 (Story 7-3 AC#29 race REPEATABLE READ), **When** `cargo test --workspace -p kesh-api --test kf004_no_op_e2e -- --test-threads=1` est exécuté, **Then** documenter :
   - Test `no_op_with_parallel_mutation_returns_409_when_sequential` actuel : passe / échoue / absent.
   - Si test existe en mode smoke séquentiel et passe : KF encore active (le smoke ne détecte pas la race, c'est le sujet de la KF) → route vers 9-5-1d.
   - Si un test déterministe race a été ajouté entre-temps : vérifier qu'il échoue sur `200 stale` (comportement v0.1 attendu) — si OK → fermer.

4. **Given** KF #54 (cascade 401 helpers E2E), **When** `npm run test:e2e -- invoices.spec.ts invoices_echeancier.spec.ts journal-entries.spec.ts` est exécuté, **Then** documenter le compte de failures avec message `401 Unauthorized` ou `expect(resp.ok()).toBeTruthy()` à `tests/e2e/helpers/test-state.ts:46`. Si zéro failure de ce type → KF résolue par effet de bord (probablement Story 7-x ou 8-x fix middleware) → fermer. Sinon → route vers 9-5-1b (avec liste précise des tests touchés).

5. **Given** KF #55 (axe-core 6 pages a11y), **When** `npm run test:e2e -- auth.spec.ts contacts.spec.ts homepage-settings.spec.ts invoices.spec.ts products.spec.ts --reporter=html` est exécuté, **Then** documenter :
   - Nombre de violations actuelles par page (vs baseline 2026-04-30 : 109 login + 82 layout principal + ~6 autres pages).
   - Top 3 catégories de violations (color-contrast / region / landmark-one-main / heading-order / etc.).
   - Si total violations < 10 → KF probablement résolue → fermer. Si ≥ 10 → route vers 9-5-1c.

6. **Given** KF #57 (state/timing/redirect dispersés), **When** `npm run test:e2e -- fiscal-years.spec.ts mode-expert.spec.ts onboarding.spec.ts onboarding-path-b.spec.ts homepage-settings.spec.ts users.spec.ts journal-entries.spec.ts` est exécuté, **Then** documenter :
   - Liste précise des tests qui échouent encore (vs baseline 2026-04-30 ~13 failures).
   - Catégorisation par root cause (timeout / `toBeVisible` / `toHaveURL` / seedTestState).
   - Si zéro failure de ce type → KF résolue → fermer. Sinon → route vers 9-5-1b avec liste précise.

7. **Given** KF #91 (DropdownMenu nested-button reports a11y), **When** `npm run test:e2e -- reports.spec.ts` est exécuté avec filtrage `a11y violations` (lignes 85 et 96 du fichier), **Then** documenter :
   - Test `reports page has zero axe a11y violations (empty state)` : pass / fail.
   - Test `reports page has zero axe a11y violations (populated state)` : pass / fail.
   - Violation `nested-interactive` sur `#bits-c1` (DropdownMenu.Trigger wrap Button dans `+layout.svelte:132-141`) : présente / absente.
   - Si 2/2 pass → KF résolue → fermer. Sinon → route vers 9-5-1c.

### Décision orchestration sous-stories

8. **Given** les 6 KFs re-testées et documentées, **When** la décision d'orchestration est rendue, **Then** chaque sous-story placeholder 9-5-1b/c/d est :
   - **Soit gardée en backlog avec scope précis défini** (liste exacte des KFs résiduelles + fichiers touchés + tests à fixer) — entrée sprint-status mise à jour.
   - **Soit annulée** (status `deleted` ou `merged-into`) si aucune KF ne tombe dans sa catégorie après triage. Documenter dans sprint-status le motif d'annulation.

9. **Given** un map résiduel KF → sous-story finalisé, **When** la story 9-5-1a est marquée done, **Then** le fichier `epic-9-5.md` est mis à jour avec :
   - Section « Décision split préventif » mise à jour (les sous-stories définitives + scope précis).
   - Liste des KFs fermées avec lien GitHub (`closes #N`).
   - Liste des KFs déférées en sous-story avec route précise.

### Commits + traçabilité GitHub

10. **Given** une KF résolue par effet de bord (test passe maintenant), **When** la story est commitée, **Then** un commit dédié `chore(9-5-1a): close KF #N (résolu par effet de bord — <description>)` ferme l'issue GitHub via `closes #N` (1 commit par KF résolue, **pas** un commit géant qui ferme toutes les issues). Justification : traçabilité Git claire + ferme issues une par une.

11. **And** chaque commit fermant une KF inclut dans son body :
    - Test E2E ou unit exécuté + commande exacte (e.g. `npm run test:e2e -- fiscal-years.spec.ts`).
    - Résultat (pass count / fail count).
    - Story du projet probablement responsable de la résolution par effet de bord (e.g. « probablement résolu par Story 8-1b middleware auth refactor »).
    - Référence au commit auteur de la résolution si identifiable via `git log --oneline -- <test path>`.

### Test Locally First

12. **Given** la story 9-5-1a est documentation/triage-only (aucun fichier source `.rs` / `.ts` / `.svelte` modifié hors `_bmad-output/`), **When** un commit est créé, **Then** l'exemption CLAUDE.md §"Quand sauter" s'applique : pas de `cargo test --workspace` ni `npm run check` requis avant push (le test-running EST le travail de la story, déjà exécuté pour les 6 KFs).

### Critères de complétion 9-5-1a

13. **Given** la story 9-5-1a est marquée done, **When** son Change Log est lu, **Then** il contient un **tableau récapitulatif** :
    ```
    | KF GitHub | Statut post-triage | Sous-story de fix | Commit closure (si applicable) |
    |---|---|---|---|
    | #47 KF-019 | active / résolue | 9-5-1d / closed | <sha> |
    | #50 KF-021 | ... | ... | ... |
    | ... | ... | ... | ... |
    ```

14. **And** le sprint-status reflète l'état finalisé : `9-5-1a-kf-triage: done` + les placeholders 9-5-1b/c/d ont chacun un scope précis OU sont annulés.

15. **And** l'epic 9-5.md décrit les sous-stories définitives 9-5-1b/c/d avec leur scope précis (basé sur le triage), prêtes pour `bmad-create-story` séquentiel.

16. **And** aucune régression : `cargo build --workspace` + `npm run build` doivent toujours passer (la story ne touche aucun fichier source).

## Tasks / Subtasks

- [ ] **T1** Pré-flight environnement (AC: #1)
  - [ ] T1.1 Vérifier branche `chore/epic-9-5-planning` checkée + à jour avec `git status`.
  - [ ] T1.2 `cargo build --workspace` propre (aucune erreur de compilation).
  - [ ] T1.3 `cd frontend && npm install` (si modifs `package.json` depuis dernière fois) + `npm run build` propre.
  - [ ] T1.4 Démarrer MariaDB local + appliquer seed CI (`crates/kesh-db/scripts/seed-ci.sh` ou équivalent — vérifier README.md du projet pour la commande exacte).
  - [ ] T1.5 Installer Playwright browsers : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npx playwright install` (memory `reference_playwright_ubuntu26`).
  - [ ] T1.6 Démarrer le backend en mode test : `KESH_TEST_MODE=true KESH_HOST=127.0.0.1 KESH_STATIC_DIR=frontend/build cargo run -p kesh-api &` (background).

- [ ] **T2** Re-test KF #47 KF-019 fiscal-years AC#22 fallback toast (AC: #2)
  - [ ] T2.1 `cd frontend && npx playwright test fiscal-years.spec.ts --reporter=list` (depuis frontend/).
  - [ ] T2.2 Grep le fichier pour annotations skip : `grep -n "test.skip" tests/e2e/fiscal-years.spec.ts`.
  - [ ] T2.3 Documenter dans Change Log : nb tests pass/fail/skip + état AC #22 block + décision (closed vs route to 9-5-1d).
  - [ ] T2.4 Si résolue : commit `chore(9-5-1a): close KF #47 KF-019 (...)` avec `closes #47`. Sinon : ajouter à liste 9-5-1d résiduel.

- [ ] **T3** Re-test KF #50 KF-021 AC#29 race REPEATABLE READ (AC: #3)
  - [ ] T3.1 `cargo test --workspace -p kesh-api --test kf004_no_op_e2e -- --test-threads=1 --nocapture` (depuis racine repo).
  - [ ] T3.2 Vérifier si test déterministe race existe : `grep -nE "race|REPEATABLE|FOR UPDATE" crates/kesh-api/tests/kf004_no_op_e2e.rs`.
  - [ ] T3.3 Documenter dans Change Log : pass count + présence test déterministe + statut KF + décision.
  - [ ] T3.4 Si résolue : commit avec `closes #50`. Sinon : ajouter à 9-5-1d résiduel.

- [ ] **T4** Re-test KF #54 KF-022 cascade 401 helpers E2E (AC: #4)
  - [ ] T4.1 `cd frontend && npx playwright test invoices.spec.ts invoices_echeancier.spec.ts journal-entries.spec.ts --reporter=html` (HTML reporter pour faciliter diagnostic 401).
  - [ ] T4.2 Compter occurrences `401 Unauthorized` ou `createContact failed: 401` ou `tests/e2e/helpers/test-state.ts:46` dans le rapport.
  - [ ] T4.3 Si zéro failure 401 : `git log -- frontend/tests/e2e/helpers/test-state.ts` pour identifier le commit auteur du fix (typiquement Story 6-x ou 8-x).
  - [ ] T4.4 Documenter dans Change Log : nb tests 401 fail/pass + commit auteur résolution si identifiable + décision.
  - [ ] T4.5 Si résolue : commit avec `closes #54`. Sinon : ajouter scope précis à 9-5-1b (liste exacte tests + helpers à fixer).

- [ ] **T5** Re-test KF #55 KF-023 axe-core a11y 6 pages (AC: #5)
  - [ ] T5.1 `cd frontend && npx playwright test auth.spec.ts contacts.spec.ts homepage-settings.spec.ts invoices.spec.ts products.spec.ts --reporter=html` + ouvrir `playwright-report/index.html`.
  - [ ] T5.2 Compter violations axe-core par page : login (baseline 109) / layout (baseline 82) / contacts / homepage / invoices empty / products.
  - [ ] T5.3 Top 3 catégories de violations (`color-contrast` / `region` / `landmark-one-main` / `heading-order` / `nested-interactive` / etc.) à documenter.
  - [ ] T5.4 Documenter dans Change Log : tableau violations par page + top 3 catégories + décision.
  - [ ] T5.5 Si toutes < 10 ET total < 30 violations résiduelles : commit avec `closes #55`. Sinon : ajouter scope précis à 9-5-1c (pages prioritaires + types violations).

- [ ] **T6** Re-test KF #57 KF-025 state/timing/redirect dispersés (AC: #6)
  - [ ] T6.1 `cd frontend && npx playwright test fiscal-years.spec.ts mode-expert.spec.ts onboarding.spec.ts onboarding-path-b.spec.ts homepage-settings.spec.ts users.spec.ts journal-entries.spec.ts --reporter=list 2>&1 | tee /tmp/kf57-output.log`.
  - [ ] T6.2 Compter failures (vs baseline ~13) en filtrant les motifs : `toBeVisible` / `toBeEnabled` / `toHaveURL` / timeout 30s.
  - [ ] T6.3 Catégoriser par root cause probable (seedTestState manquant fiscal_year / auth state shift / brittle selectors `getByText`).
  - [ ] T6.4 Documenter dans Change Log : liste tests échouant + root cause par catégorie + décision.
  - [ ] T6.5 Si zéro failure : commit `closes #57`. Sinon : ajouter scope précis à 9-5-1b ou 9-5-1d selon root cause (timing → 9-5-1b ; specifics → 9-5-1d).

- [ ] **T7** Re-test KF #91 KF-027 DropdownMenu reports a11y (AC: #7)
  - [ ] T7.1 `cd frontend && npx playwright test reports.spec.ts --reporter=list` + capturer les 5 tests (3 pass attendus tabs/balance-sheet/T12.4 + 2 fail attendus a11y).
  - [ ] T7.2 Vérifier `frontend/src/routes/(app)/+layout.svelte:132-141` : `<DropdownMenu.Trigger><Button>...` toujours présent ?
  - [ ] T7.3 Documenter dans Change Log : statut 5 tests + état composant +layout.svelte + décision.
  - [ ] T7.4 Si 2 fail a11y résolus : commit `closes #91`. Sinon : ajouter scope précis à 9-5-1c (composant bits-ui wrap à patcher).

- [ ] **T8** Décision orchestration sous-stories 9-5-1b/c/d (AC: #8, #9, #13, #14, #15)
  - [ ] T8.1 Lister les KFs résiduelles (non fermées en T2-T7) par catégorie : (a) infra E2E (#54, #57), (b) a11y (#55, #91), (c) specifics (#47, #50).
  - [ ] T8.2 Pour chaque sous-story 9-5-1b/c/d, écrire son scope final :
    - Si la catégorie a 0 KF résiduelle → annuler la sous-story (status `deleted` ou note `merged-into` dans sprint-status).
    - Si la catégorie a ≥ 1 KF résiduelle → écrire scope précis : KFs concernées + fichiers touchés (liste exacte de `*.spec.ts` / helpers / composants).
  - [ ] T8.3 Mettre à jour `_bmad-output/implementation-artifacts/sprint-status.yaml` : entrées 9-5-1b/c/d avec scope finalisé OU status annulé.
  - [ ] T8.4 Mettre à jour `_bmad-output/planning-artifacts/epic-9-5.md` : compléter la section « Décision split préventif appliquée 2026-05-18 » avec les scopes finalisés post-triage.
  - [ ] T8.5 Construire le tableau récapitulatif AC #13 dans le Change Log de cette story.

- [ ] **T9** Vérification finale Test Locally First exemption (AC: #12, #16)
  - [ ] T9.1 `git diff --stat HEAD` : confirmer que seuls les fichiers `_bmad-output/` + `sprint-status.yaml` + spec file sont modifiés. **Aucun** fichier `.rs` / `.ts` / `.svelte` modifié hors commits closure GitHub (qui sont sur les tests, pas le code prod).
  - [ ] T9.2 Sanity check : `cargo build --workspace 2>&1 | tail -3` clean + `cd frontend && npm run build 2>&1 | tail -5` clean (devraient être identiques à avant la story par construction — story ne touche pas le code).
  - [ ] T9.3 Pas de `cargo test --workspace` requis (exemption doc-only — l'execution des tests EST le travail de la story déjà fait T2-T7).

## Dev Notes

### Cadrage scope minimal — anti-pattern Story 7-1

Cette story 9-5-1a est volontairement **scope-minimaliste** pour éviter de répéter Story 7-1 historique (4 passes spec validate, 7+ modules touchés simultanément). Le triage rapide produit un map précis des KFs résiduelles, qui peuvent ensuite être adressées par sous-stories spécialisées (9-5-1b infra / 9-5-1c a11y / 9-5-1d specifics). Chaque sous-story aura un cycle review attendu de ≤ 2 passes vs 4+ attendues si fait en une seule.

### Approche triage = re-exécution + observation, pas implémentation

Les 6 KFs ont été ouvertes entre Story 3-7 et Story 9-1. Depuis :
- **Stories 7-x** ont refactor multi-tenant scoping + KF-002 audit
- **Stories 8-x** ont refactor middleware auth, batch endpoints, manual reconciliation
- **Story 9-x** ont ajouté `kesh-report` crate + exports PDF/CSV/ZIP

Hypothèse : certaines KFs ont pu être résolues **par effet de bord** d'un refactor latéral (e.g. Story 6-5 storage shift KF-007 closure aurait pu fixer la cascade 401 du KF #54). Le triage 9-5-1a confirme empiriquement quelles KFs sont encore reproductibles.

**Estimation heuristique pré-triage** (à confirmer par l'exécution réelle) :
- Hypothèse haute : 2/6 KFs résolues par effet de bord → 4 sous-stories nécessaires
- Hypothèse moyenne : 1/6 résolue → 4 sous-stories (mais une avec scope réduit)
- Hypothèse basse : 0/6 résolue → 3 sous-stories complètes (b/c/d toutes nécessaires)

### Memory carries

- `reference_playwright_ubuntu26` : sur Ubuntu 26.04 obligatoire `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` pour install + run E2E.
- `feedback_haiku_review_diff_combined` : la discipline grep ground-truth s'applique aussi aux reviews de cette story (mais surface review = principalement Markdown du Change Log).

### Coordination avec sous-stories suivantes

Une fois 9-5-1a done :
- **9-5-1b** (infra E2E) prend les KFs résiduelles de catégorie « E2E test infrastructure » — typiquement #54 (helpers cascade 401) + #57 partiel (timing/state).
- **9-5-1c** (a11y) prend les KFs résiduelles de catégorie « a11y violations » — typiquement #55 (axe-core 6 pages) + #91 (DropdownMenu wcag).
- **9-5-1d** (specifics) prend les KFs résiduelles « cas spécifiques » — typiquement #47 (AC#22 fallback toast tests) + #50 (AC#29 race REPEATABLE READ test déterministe) + résiduels #57 selon catégorisation.

Si une sous-story se retrouve avec 0 KF résiduelle (e.g. toutes les KFs a11y sont fermées en triage), elle est **annulée** (pas implémentée) — la sprint-status entry est marquée `deleted` avec motif documenté.

### Risque R1 — KF #91 composant bits-ui

Le fix de KF #91 (DropdownMenu nested-button wcag) peut nécessiter :
- Soit un patch du composant `bits-ui` (upstream lib, hors scope projet) — pas viable v0.1.
- Soit un wrapper custom Svelte qui retire le `<Button>` interne et utilise un simple `<div role="button">` (cf. bits-ui issue tracker ou docs).
- Soit acceptation v0.2 et labellage `v0.2-milestone` (avec gate v0.1 → catégorie A per règle CLAUDE.md §Tech debt management).

**Décision triage** : 9-5-1a ne tranche pas — la décision (fix vs reporter v0.2) revient à 9-5-1c. 9-5-1a documente uniquement si la KF est encore reproductible.

### Risque R2 — KF #55 si > 100 violations

Si le triage de KF #55 révèle > 100 violations a11y (cohérent baseline 2026-04-30 : 109 sur login), 9-5-1c sera trop large pour une story unique. Re-split nécessaire :
- 9-5-1c-quick-fixes : color-contrast + alt-text + ARIA labels (mécanique)
- 9-5-1c-structural : landmarks + heading-order + focus-management (architectural)

Cette décision est documentée dans 9-5-1a si nécessaire, et bloque le passage 9-5-1c → ready-for-dev.

### Project Structure Notes

- **Fichiers édités par 9-5-1a** :
  - `_bmad-output/implementation-artifacts/9-5-1a-kf-triage.md` (cette spec, Change Log + Dev Agent Record)
  - `_bmad-output/implementation-artifacts/sprint-status.yaml` (entrées 9-5-1a + 9-5-1b/c/d scopes finalisés)
  - `_bmad-output/planning-artifacts/epic-9-5.md` (section Split décision finalisée)
- **Fichiers NON touchés** : aucun fichier source `.rs` / `.ts` / `.svelte` modifié par cette story.
- **GitHub Issues** : 0 à 6 fermées via `closes #N` dans messages de commits dédiés.

### Testing standards summary

- **Tests à exécuter** : 6 batteries de tests existants (5 E2E Playwright + 1 cargo). Pas de nouveau test ajouté.
- **Baselines** : `cargo test --workspace` + `npm run test:unit` + `npm run test:e2e` baselines vertes pré-story restent identiques post-story (story ne modifie pas de code).
- **Test Locally First exemption** : doc/triage-only — pas de check CI requis avant push (l'execution des tests EST le travail de la story).

### References

- [Source: _bmad-output/planning-artifacts/epic-9-5.md#Story-9.5-1] — spec parent epic + Q3 split anticipé
- [Source: _bmad-output/implementation-artifacts/epic-9-retro-2026-05-17.md] — challenge C-cleanup KFs Epic 7 dormantes
- [GitHub Issue #47 KF-019] — Story 3-7 AC#22 fallback toast
- [GitHub Issue #50 KF-021] — AC#29 race REPEATABLE READ
- [GitHub Issue #54 KF-022] — cascade 401 helpers E2E
- [GitHub Issue #55 KF-023] — axe-core 6 pages a11y
- [GitHub Issue #57 KF-025] — state/timing/redirect dispersés
- [GitHub Issue #91 KF-027] — DropdownMenu reports a11y
- [Source: CLAUDE.md§Test Locally First] — exemption commits doc-only
- [Source: CLAUDE.md§Règle de splitting préventif] — règle déclenchée pour 9-5-1 → split → 9-5-1a triage

## Dev Agent Record

### Agent Model Used

(À renseigner — typiquement Claude Opus 4.7 ou Sonnet 4.6 pour dev-story.)

### Debug Log References

(Vide à la création — sera renseigné post-dev avec extracts des tests qui montrent l'état de chaque KF.)

### Completion Notes List

(Vide à la création — sera renseigné post-triage avec résumé : KFs fermées vs résiduelles + scopes finalisés 9-5-1b/c/d.)

### File List

- `_bmad-output/implementation-artifacts/9-5-1a-kf-triage.md` — cette spec, Change Log + Dev Agent Record peuplés post-dev.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — entrées 9-5-1a + 9-5-1b/c/d scopes finalisés.
- `_bmad-output/planning-artifacts/epic-9-5.md` — section Split décision complétée.
- Commits closure GitHub si KFs résolues : 0-6 commits `chore(9-5-1a): close KF #N (...)` avec body justifié.

## Change Log

(Vide à la création — sera renseigné post-dev avec tableau récapitulatif des 6 KFs + statut + sous-story affectée + commit closure.)
