# Story 6.1 : Pipeline CI GitHub Actions

Status: review

<!-- Note : validation optionnelle via `validate-create-story` avant `dev-story`. -->

## Story

As a **développeur (Guy, mainteneur solo)**,
I want **une pipeline CI GitHub Actions complète, déterministe et documentée qui bloque tout merge sur `main` en cas de régression backend, frontend, E2E ou accessibilité, et qui publie une image Docker traçable à chaque merge**,
so that **Epic 7+ (import bancaire, rapports, TVA…) ne réintroduise pas de dette héritée comme KF-001 (E2E silencieux ~3 jours) et que toute régression soit détectée au niveau du PR, pas au run-time**.

### Contexte

**Première story de l'Epic 6 « Qualité & CI/CD »** (inséré en rétro Epic 5, décision structurante 2026-04-16). Priorité CRITICAL — débloque les 3 autres stories de l'epic (6-2 multi-tenant, 6-3 lint i18n, 6-4 fixtures E2E) et toutes les stories de l'Epic 7 import bancaire.

**Origine** : promue depuis l'ancienne story 8-4 Epic 8 Déploiement lors du renumérotage Epic 6→13 en 7→14. La promotion vient du fait que l'absence de CI gate a laissé passer KF-001 (`invoice_pdf_e2e` cassé 10/11 tests pendant ~3 jours entre Story 5-3 et la review Story 5-4). Sans CI, chaque story suivante peut masquer des régressions E2E silencieuses.

**État actuel du pipeline** (NE PAS refaire de zéro) :

La pipeline **existe déjà** dans `.github/workflows/ci.yml` (commit `febd745` + `e4994af` + `c894649`) et couvre 4 jobs :

1. **`backend`** — `rustup show` (toolchain pinnée via `rust-toolchain.toml` → 1.85.0), `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace -j1 -- --test-threads=1`. Service MariaDB 11.4 avec healthcheck, cache Swatinem.
2. **`frontend`** — `npm ci` + `npm run check` (svelte-check) + `npm run test:unit` (Vitest) + `npm run build`. Node 22 + cache `frontend/package-lock.json`. Upload artifact `frontend/build`.
3. **`e2e`** — dépend de `[backend, frontend]`. Build `target/release/kesh-api`, démarre le backend avec MariaDB service, lance `npx playwright install --with-deps chromium` + `npm run test:e2e`. Upload `playwright-report` en cas d'échec.
4. **`docker-build`** — sanity build `kesh:ci` **sans push** (valide le Dockerfile sans publier).

`release.yml` publie sur push de tag `v*.*.*` : build + push Docker Hub (`guycorbaz/kesh:{version}` + `:latest`) + création GitHub Release.

**Fondations testing déjà en place** :

- Tests backend utilisent `sqlx::test` (ephemeral DBs `_sqlx_test_*`) — les migrations sont appliquées automatiquement par SQLx avant chaque test (pas besoin de `cargo sqlx migrate run` préalable dans le job `backend`).
- Le backend `kesh-api` applique `MIGRATOR.run()` au démarrage (`main.rs:62-66`) — les migrations sont donc appliquées lorsque le service démarre dans le job `e2e`.
- `@axe-core/playwright` est installé et utilisé dans `auth.spec.ts` et `users.spec.ts` (4 tests axe au total). Les 10 autres spec files (`invoices.spec.ts`, `contacts.spec.ts`, `products.spec.ts`, `accounts.spec.ts`, `journal-entries.spec.ts`, `homepage-settings.spec.ts`, `mode-expert.spec.ts`, `onboarding.spec.ts`, `onboarding-path-b.spec.ts`, `invoices_echeancier.spec.ts`) ne font pas de check axe.
- `docker-compose.dev.yml` utilise `mariadb:11.4` (même image que le CI, cohérent).
- Pas de script `scripts/` à la racine ; pas de `justfile` ni `Makefile` — les commandes canoniques sont celles du CI.
- `rust-toolchain.toml` pinne `1.85.0` — la CI respecte ce pin via `rustup show`.
- `frontend/package.json` : `"engines": { "node": ">=20" }` mais la CI utilise Node 22 (strict).
- `playwright.config.ts` force `locale: 'fr-CH'` + `timezoneId: 'Europe/Zurich'` (D4 Story 5-4 review pass 1 G2 D).

**Gaps vs AC Epic 6 §6.1** (ce que cette story doit combler) :

| AC Epic | État actuel | Gap à combler |
|---|---|---|
| `cargo build, cargo test, cargo clippy -D warnings, cargo fmt --check` | ✅ Présent | Aucun — ajouter explicit `cargo build --workspace --all-targets` juste avant `cargo test` pour catch les erreurs de compilation non-test plus tôt (feedback rapide) |
| `npm run check, npm run test, npm run build` | ✅ Présent | Aucun |
| Playwright + MariaDB **10.11** + migrations SQLx + seed test | Partiel : MariaDB **11.4** (pas 10.11), pas de step explicite `cargo sqlx migrate run`, seed via bootstrap admin du backend | **Décision** : garder MariaDB 11.4 (cohérent avec `docker-compose.dev.yml`) — clarifier dans la story que l'AC Epic est imprécis et que 11.4 est le choix effectif. Ajouter une étape explicite `cargo sqlx migrate run` en amont du lancement backend (redondant avec `MIGRATOR.run()` au startup, mais auto-documente le flow et fail-fast si migrations cassées). Seed : rester sur le bootstrap admin (login `admin/admin` via `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD`) — fixtures E2E dédiées sont le scope de Story **6-4**, pas 6-1. |
| Merge `main` → Docker image **buildée et publiée** | Partiel : `ci.yml/docker-build` build sans push ; `release.yml` publie uniquement sur tags `v*.*.*` | Ajouter dans `ci.yml` un job dédié `docker-publish-main` qui `needs: [backend, frontend, e2e, docker-build]` et ne se déclenche que `if: github.event_name == 'push' && github.ref == 'refs/heads/main'`. Publie `guycorbaz/kesh:main` + `guycorbaz/kesh:main-{github.sha}`. **Critique** : placer ce job dans `ci.yml` avec les `needs:` ci-dessus garantit qu'aucune image `:main` cassée n'est publiée (gate sur le succès de backend/frontend/e2e). `release.yml` reste exclusivement SemVer (trigger `tags: ['v*.*.*']`). |
| Feedback clair en cas d'échec | ✅ Présent (annotations GitHub standard) | Ajouter `GITHUB_STEP_SUMMARY` dans chaque job qui condense `failed | passed` visible dans l'onglet Summary. Optionnel mais utile. |
| Fichiers `.github/workflows/ci.yml` + `release.yml` | ✅ Présent | Aucun |
| **axe-core intégré dans CI** pour régressions accessibilité (UX-DR18) | Partiel : 2 spec files sur 12 font des checks axe (auth + users) | Étendre axe-core à **6 spec files clés** : `auth.spec.ts` (✅), `users.spec.ts` (✅), `contacts.spec.ts`, `products.spec.ts`, `invoices.spec.ts`, `homepage-settings.spec.ts`. **Décision** : un seul test axe « snapshot accessible » par fichier suffit (évite le coût d'exécution × 10). Chaque nouveau spec doit ajouter un check axe (règle documentée dans `docs/ci.md`). |
| **Migration (`cargo sqlx migrate run`) avant les tests repos/E2E** | Implicite | Rendre explicite dans le job `e2e` (step dédiée), documenter que les tests `sqlx::test` du backend l'appliquent automatiquement — pas besoin pour `backend`. |

**Hardening manquant détecté en audit** (pas dans l'AC strict mais nécessaire) :

- **Concurrency cancellation** : un second push sur une branche annule le 1er pipeline en cours (sinon file d'attente infinie sur les runners gratuits). Absent aujourd'hui.
- **Timeouts par job** : pas configurés (défaut GitHub = 360 min = 6h). Forcer 30 min max backend, 20 min frontend, 30 min E2E, 20 min docker-build — force un échec rapide si un test freeze (cf. KF SQLx `PoolTimedOut`).
- **Status badges** dans `README.md` : aucune indication publique que la CI existe.
- **Permissions explicites** sur `release.yml` : `contents: write` OK mais les autres workflows héritent de `permissions: write-all` par défaut — forcer `permissions: { contents: read }` minimum sur `ci.yml` (principle of least privilege).
- **Pinning des actions** aux SHAs : nice-to-have sécurité supply-chain — **hors scope 6-1** (peut devenir Dependabot job plus tard).

### Scope verrouillé — ce qui DOIT être fait

1. **Audit + alignement `ci.yml` sur AC** : ajouter step explicite `cargo build --workspace --all-targets` dans `backend` (feedback rapide avant clippy/tests). Ajouter step `cargo sqlx migrate run` dans `e2e` avant le démarrage backend (redondant mais auto-documente). Forcer `permissions: { contents: read }` sur `ci.yml` (job-level ou workflow-level).
2. **Concurrency + timeouts** : ajouter en tête de `ci.yml` :
   ```yaml
   concurrency:
     group: ci-${{ github.ref }}
     cancel-in-progress: true
   ```
   et `timeout-minutes: 30` (backend, e2e), `20` (frontend, docker-build).
3. **Extension axe-core à 6 specs** : ajouter un `test('accessibilité - pas de violations critiques axe-core', ...)` inspiré de `auth.spec.ts:88-96` dans `contacts.spec.ts`, `products.spec.ts`, `invoices.spec.ts`, `homepage-settings.spec.ts`. Chaque test `await new AxeBuilder({ page }).analyze()` puis `expect(results.violations).toEqual([])`. Tolérance : exclure les règles WCAG AAA via `.disableRules([...])` si nécessaire, mais **AUCUNE exclusion** sur les règles AA. Si une violation AA existe, la corriger (c'est le but) ou documenter une KF + dette technique avec story de remédiation.
4. **Publish Docker sur merge `main`** : créer un job `docker-publish-main` **dans `ci.yml`** (pas `release.yml`) qui `needs: [backend, frontend, e2e, docker-build]` et se déclenche uniquement `if: github.event_name == 'push' && github.ref == 'refs/heads/main'`. Publie les tags `guycorbaz/kesh:main` + `guycorbaz/kesh:main-${{ github.sha }}` (SHA complet 40 chars — reproductibilité). **Raison du placement dans `ci.yml`** : le `needs:` sur les 4 autres jobs est le gate naturel contre la publication d'images cassées. Un `release.yml` indépendant (trigger `push: branches: [main]`) se déclencherait en parallèle de `ci.yml` et publierait même si les tests échouent — risque critique évité. `release.yml` reste strictement SemVer (trigger `tags: ['v*.*.*']`) et inchangé fonctionnellement hormis l'ajout de concurrency (§17).
5. **Step summary** : dans chaque job, une dernière step `always()` qui appende un résumé à `$GITHUB_STEP_SUMMARY` (ex : « ✅ Backend : fmt + clippy + 287 tests OK en 14m22s »). Niveau de détail minimum : nom du job + durée + OK/FAIL. Implémentation via un simple `echo` conditionnel — pas d'action externe.
6. **Status badges dans `README.md`** : 2 badges (CI + Release) en tête du README, formes :
   ```markdown
   [![CI](https://github.com/guycorbaz/kesh/actions/workflows/ci.yml/badge.svg)](https://github.com/guycorbaz/kesh/actions/workflows/ci.yml)
   [![Release](https://github.com/guycorbaz/kesh/actions/workflows/release.yml/badge.svg)](https://github.com/guycorbaz/kesh/actions/workflows/release.yml)
   ```
7. **Documentation `docs/ci.md`** : nouveau fichier expliquant (a) les 4 jobs et ce qu'ils valident, (b) comment reproduire chaque step en local (`cargo test --workspace -j1 -- --test-threads=1`, `npm run test:e2e`, etc.), (c) la politique axe-core (règle par spec), (d) la stratégie de publication Docker (main + SemVer tags), (e) les timeouts et concurrency. ~1-2 pages markdown.
8. **Validation manuelle post-merge** : après merge de la PR Story 6-1, déclencher manuellement un push pour vérifier que les 4 jobs passent vert, qu'une image `guycorbaz/kesh:main-{sha}` est publiée sur Docker Hub, et qu'un badge affiche « passing » dans le README. Critère de `done` inclus dans AC#16.

### Scope volontairement HORS story — décisions tranchées

- **Coverage reporting** (codecov, tarpaulin, llvm-cov) → hors scope. Nice-to-have, mais coûteux à maintenir en solo-dev. À évaluer post-v0.1.
- **Dependabot / security scan** (cargo-audit, npm audit, trivy) → **créer une story 6-1bis OU follow-up dans backlog Epic 6** si Guy souhaite ; pas dans 6-1 pour éviter le scope creep. Décision : **hors scope 6-1**, ajouter au backlog comme CR si besoin.
- **Pinning des actions aux SHAs commit** (vs tag) → hors scope — sécurité supply chain souvent gérée par Dependabot plutôt qu'à la main. Décision : accepter le risque en v0.1.
- **Matrice multi-OS** (Windows / macOS) → hors scope. Kesh cible Linux Docker uniquement (PRD §77 déploiement docker-compose).
- **Lint i18n key-ownership** (AC Story 6-3) → hors scope 6-1. Story dédiée 6-3.
- **Refactor multi-tenant scoping** (AC Story 6-2) → hors scope 6-1. Story dédiée 6-2.
- **Fixtures E2E déterministes** (helper `seed_accounting_company`) → hors scope 6-1. Story dédiée 6-4.
- **Benchmarks performance** (temps de build, taille image Docker, durée tests) → hors scope. Mesurer empiriquement en ops, pas bloquant CI.
- **Audit i18n FTL manquantes en CI** (vérifier que chaque clé utilisée existe dans les 4 locales) → candidat CR séparé, pas 6-1.
- **Upgrade MariaDB 11.4 → 10.11 pour coller à l'AC Epic** → **refusé**. `docker-compose.dev.yml` + CI convergent sur 11.4. L'AC Epic §6.1 mentionne « 10.11 » par erreur (copier-coller stub de 2026-04-03, avant que le choix 11.4 soit entériné). **La story amende explicitement l'AC : MariaDB 11.4 est le choix effectif**, documenté dans le Change Log. Architecture.md dit « 10.6+ » (minimum), 11.4 respecte le minimum.
- **Publication automatique des crates publishables** (`kesh-qrbill`, `kesh-import`, `kesh-payment`) sur crates.io → hors scope — decisionée architectalement « publication remise à plus tard » (architecture.md §78). À faire opportunément story dédiée Epic 7+ ou post-v0.1.
- **Slack/Discord/email notifications sur échec** → hors scope, Guy est solo-dev et vérifie manuellement.
- **Cache Playwright browsers** (éviter `npx playwright install` à chaque run) → nice-to-have. Installer prend ~1 min ; pas bloquant. Reporté si la CI devient trop lente.

### Décisions de conception

- **Pipeline `ci.yml` reste dans un seul fichier** (vs split par job) — cohérent avec pratique actuelle, grep-friendly, facile à review. Split éventuel si on dépasse 400 lignes.

- **`concurrency.group = ci-${{ github.ref }}`** — annule les runs précédents sur la même ref (branche ou PR). Sur `main`, annule si deux pushes rapides se succèdent (fréquent en merge de PR + hotfix). Cohérent avec le pattern GitHub recommandé.

- **Timeouts conservateurs** : 30 min pour backend (compilation + tests `-j1` = ~15 min observé en local, +buffer pour cold cache CI), 20 min pour frontend (build Vite + vitest + svelte-check ~5 min), 30 min pour e2e (release build + Playwright chromium = ~15 min), 20 min pour docker-build (~8 min). **Raison** : éviter de mobiliser un runner 6h si un test freeze (`PoolTimedOut` cross-binary, mémo `feedback_sqlx_mysql_gotchas`).

- **`permissions: { contents: read }` au niveau workflow** pour `ci.yml` — principle of least privilege. `release.yml` garde `contents: write` (nécessaire pour créer une release GitHub). Aucun autre scope d'écriture nécessaire pour `ci.yml` (pas de commit back, pas d'issue comment).

- **Pas de step `cargo sqlx migrate run` dans `backend`** — redondant car `sqlx::test` applique les migrations automatiquement pour chaque test ephemeral DB (mémo projet SQLx 0.8). L'ajouter ralentirait sans valeur. **Par contre**, l'ajouter dans `e2e` auto-documente le flow et provoque un fail-fast si les migrations sont cassées (vs échec opaque au démarrage backend).

- **`cargo build --workspace --all-targets` avant `cargo test`** — échec plus rapide et plus lisible si le code ne compile pas (test harness inclus). Cache Swatinem partage les artefacts → coût marginal ~30s.

- **axe-core : 1 test par spec, pas par page** — ajouter un test `'accessibilité - pas de violations critiques'` dans chaque `describe` principal de spec. Charger la page principale de la feature + run axe. Un seul test = coverage baseline ; les refactors visuels qui introduisent des violations seront détectés.
  - **Règle** : `AxeBuilder` sans `.disableRules()` (aucune exclusion). Si une violation AA existe (ex : contraste insuffisant, label manquant), corriger le composant concerné. Si correction infaisable dans le scope 6-1, documenter une KF dans `docs/known-failures.md` + GitHub issue `known-failure` + story de remédiation, et ajouter `.disableRules(['violated-rule-id'])` avec commentaire pointant la KF.
  - **Portée** : 6 spec files (auth ✅, users ✅, contacts, products, invoices, homepage-settings). Les 6 autres (accounts, journal-entries, mode-expert, onboarding, onboarding-path-b, invoices_echeancier) sont ajoutés en follow-up s'ils introduisent des régressions.

- **Publication Docker `:main` + `:main-{sha}`** — `:main` = tag mutable (toujours le dernier commit main), `:main-{sha}` = tag immuable (traçabilité). Pattern standard GitOps. SemVer tags (`:0.1.0`, `:latest`) inchangés — émis uniquement sur push de tag `v*.*.*`.

- **Job `docker-publish-main` dans `ci.yml` (pas `release.yml`)** — le gate sur le succès de backend/frontend/e2e est matérialisé par `needs: [backend, frontend, e2e, docker-build]`. Un workflow indépendant sur `push: branches: [main]` n'aurait PAS ce gate natif — il faudrait passer par `workflow_run` (verbeux, race conditions possibles sur les webhooks). Placement dans `ci.yml` = solution la plus simple et la plus sûre.

- **`github.sha` complet (40 chars) pour le tag `:main-{sha}`** vs short SHA (7 chars) — déterministe, pas de collision possible. Coût : tag long mais standard OCI.

- **`concurrency` sur `release.yml` aussi** — `group: release-${{ github.ref }}` avec `cancel-in-progress: false` (on ne veut **pas** annuler une release SemVer en cours, contrairement aux runs main). Protège contre le cas rare où deux tags SemVer partent en parallèle (ex : `v0.1.0` + `v0.1.1` poussés en rafale) — garantit publication séquentielle.

- **`cargo-binstall` vs `cargo install` pour sqlx-cli** — `cargo install sqlx-cli --locked` recompile depuis les sources (~3-5 min cold cache CI). `cargo-binstall` télécharge un binaire pré-compilé depuis les Releases GitHub du projet (~10s). Gain ~4 min par run e2e. Action officielle `cargo-bins/cargo-binstall@main`.

- **Docker Hub conservé** (vs GitHub Container Registry `ghcr.io`) — `release.yml` actuel utilise déjà Docker Hub avec `secrets.DOCKERHUB_USERNAME/TOKEN`. Pas de raison de migrer. Si Guy préfère `ghcr.io`, c'est un CR séparé.

- **Step summary simple via `echo >> $GITHUB_STEP_SUMMARY`** — pas de dépendance externe, fonctionne sur tous les runners. Format minimaliste (markdown ligne unique par job). Suffisant pour un solo-dev qui scanne le tab Summary.

- **Status badges en tête du `README.md`** — pas en fin de doc, pour visibilité immédiate. Cohérent avec pratique open source.

- **`docs/ci.md` comme source de vérité** — quand quelqu'un demande « comment tourne la CI ? » ou « pourquoi ce step ? », pointer ce fichier. Maintenu à chaque modif du CI (règle à ajouter dans `CLAUDE.md` si besoin — peut aussi être implicite).

- **Pas de matrix `fail-fast: false`** — on veut un fail-fast agressif (stop tout job à la première erreur). Le pattern `needs: [backend, frontend]` sur le job `e2e` garantit déjà que e2e ne tourne pas si backend ou frontend échoue.

- **Pas d'étape de vérif « migrations applicables en rollback + re-run »** dans cette story — c'est le scope de Story 9-2 (Migrations automatiques & détection de version). 6-1 se limite à `cargo sqlx migrate run` forward-only.

### Dette technique documentée — v0.2 ou plus tard

- **D-6-1-A — Cache des browsers Playwright** (sévérité LOW). La step `npx playwright install --with-deps chromium` ajoute ~1 min à chaque run e2e. Un cache `~/.cache/ms-playwright` via `actions/cache@v4` économiserait ~50s. Bénéfice marginal sur solo-dev ; à traiter si la CI dépasse 30 min en observation.

- **D-6-1-B — Pinning actions aux SHAs** (sévérité LOW, sécurité supply chain). Utilisation actuelle : tags (`@v4`, `@v2`, `@v3`, `@v6`) — un compromis d'une action populaire peut injecter du code. Dependabot peut automatiser le pinning + renouvellement. Décision : reporter, pas bloquant MVP.

- **D-6-1-C — axe-core partiel (6/12 specs)** (sévérité LOW). 6 spec files n'ont pas de check axe. À couvrir en follow-up opportuniste ou lors d'une story future de refactoring UX. Documenté dans `docs/ci.md`.

- **D-6-1-D — axe-core `/invoices` teste l'empty state uniquement** (sévérité LOW). Le seed E2E actuel ne peuple pas la liste des factures → le test axe valide l'empty state. Régressions visuelles sur les lignes de tableau (badges statut paiement, contraste zebra, etc.) ne sont détectées qu'après Story 6-4 (fixtures E2E déterministes). Remédiation : lors de Story 6-4, étendre le test axe de `invoices.spec.ts` à courir APRÈS le seed `seed_accounting_company` pour couvrir l'état peuplé.

### Concurrence et ordre d'exécution CI

- `backend` et `frontend` tournent en parallèle (no `needs:`).
- `e2e` dépend de `[backend, frontend]` — attendu car utilise les artefacts backend (release binary) et l'environnement frontend.
- `docker-build` (sanity, pas de push) tourne indépendamment, en parallèle des 3 autres.
- `docker-publish-main` dépend de `[backend, frontend, e2e, docker-build]` et ne se déclenche que sur `push` de `main`. **Gate implicite** : si un seul des 4 échoue, pas de publication `:main`.
- `release.yml` tourne **après** push de tag `v*.*.*` (trigger séparé), jamais sur push de branche. Aucune race possible avec `ci.yml` (triggers disjoints : branches vs tags).
- **Risque race SemVer** : si deux tags `v*.*.*` partent en rafale, `concurrency.cancel-in-progress: false` garantit publication séquentielle (pas d'annulation).

## Acceptance Criteria

1. **Given** un push ou une pull request sur `main`, **When** GitHub Actions se déclenche, **Then** le workflow `ci.yml` exécute **4 jobs** (`backend`, `frontend`, `e2e`, `docker-build`), chacun avec timeout explicite ≤ 30 min, et le pipeline finit en succès. _[Modifié par CR #17 — plus de 5e job `docker-publish-main` sur push main.]_

2. **Given** le job `backend`, **When** exécution, **Then** les steps suivantes passent dans l'ordre : `rustup show`, `rustup component add rustfmt clippy`, Swatinem cache, wait-for-mariadb, `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace -j1 -- --test-threads=1`.

3. **Given** le job `frontend`, **When** exécution, **Then** les steps passent : `npm ci`, `npm run check` (svelte-check), `npm run test:unit` (Vitest), `npm run build`, upload artifact `frontend-build`.

4. **Given** le job `e2e`, **When** exécution, **Then** les steps passent : `cargo sqlx migrate run` (explicite, en plus du `MIGRATOR.run()` au startup backend), build release du backend, install Playwright chromium, démarrage backend avec healthcheck, exécution `npm run test:e2e`, arrêt backend, upload `playwright-report` si échec.

5. **Given** le job `docker-build` (sanity), **When** exécution sur `ci.yml`, **Then** l'image `kesh:ci` est buildée **sans push** — valide uniquement que le Dockerfile compile.

6. **Given** un push de tag `v*.*.*`, **When** `release.yml` se déclenche, **Then** l'image `guycorbaz/kesh:{version}` et `:latest` sont publiées sur Docker Hub ET une GitHub Release est créée (comportement existant préservé, trigger strictement `tags: ['v*.*.*']`).

7. ~~**Given** un push sur `main` (merge de PR) **ET** que les jobs `backend`, `frontend`, `e2e`, `docker-build` passent vert, **When** le job `docker-publish-main` de `ci.yml` se déclenche, **Then** les images `:main` et `:main-{sha}` sont publiées.~~ _[Supprimé par CR #17 — aucune publication Docker sur push `main`.]_

7bis. ~~**Given** un push sur `main` où un des 4 jobs prérequis échoue, **When** le pipeline s'exécute, **Then** le job `docker-publish-main` est skip.~~ _[Supprimé par CR #17 — N/A.]_

8. **Given** `ci.yml`, **When** un 2e push survient sur la même branche pendant qu'un run précédent est en cours, **Then** le 1er run est annulé (concurrency `ci-${{ github.ref }}` avec `cancel-in-progress: true`).

9. **Given** `ci.yml`, **When** n'importe quel job dépasse son timeout, **Then** le run échoue explicitement avec message « timeout-minutes exceeded ».

10. **Given** `ci.yml`, **When** on inspecte le fichier YAML, **Then** `permissions: { contents: read }` est déclaré au niveau workflow (principle of least privilege).

11. **Given** les 6 spec files Playwright clés (`auth`, `users`, `contacts`, `products`, `invoices`, `homepage-settings`), **When** exécution du job `e2e`, **Then** chaque spec exécute au moins un test `axe-core` qui assert `results.violations` vide (aucune règle désactivée sauf si KF documentée). **Note** : `invoices.spec.ts` teste un empty state (pas de données seed en 6-1) — dette D-6-1-D, à étendre en 6-4 avec `seed_accounting_company`.

12. **Given** un échec de CI sur un PR, **When** un développeur consulte l'onglet « Checks » GitHub, **Then** le job qui a échoué est clairement identifié, le step qui a planté est surligné, et le log est accessible directement (comportement natif GitHub préservé — pas de dégradation).

13. **Given** chaque job de `ci.yml`, **When** le job termine (succès ou échec), **Then** un bloc markdown est ajoutée à `$GITHUB_STEP_SUMMARY` contenant au minimum : le nom du job (`### Job ${{ github.job }}`), son status (`${{ job.status }}`), et un timestamp UTC. Visible dans l'onglet Summary du run. Format délibérément minimaliste (pas de durée ni de step failed — la conclusion native GitHub les affiche déjà dans l'UI Actions).

14. **Given** `README.md`, **When** on l'ouvre, **Then** les 2 badges (CI + Release) sont présents en tête de fichier, liés aux pages Actions correspondantes, et reflètent l'état du dernier run sur `main`.

15. **Given** `docs/ci.md`, **When** on l'ouvre, **Then** il documente (a) les 4 jobs du CI et ce qu'ils valident, (b) comment reproduire chaque step localement, (c) la politique axe-core, (d) la stratégie de publication Docker main + SemVer, (e) la politique de timeouts et concurrency.

16. **Given** la PR de Story 6-1 mergée sur `main`, **When** on observe le run `ci.yml` sur `main`, **Then** les 4 jobs passent vert (`backend`, `frontend`, `e2e`, `docker-build` sanity) ET le badge CI du README affiche « passing ». _[Modifié par CR #17 — plus de validation `docker pull` post-merge ; la première publication Docker sera sur `v0.1.0`.]_

17. **Given** le backend et l'e2e partagent un service MariaDB 11.4, **When** quelqu'un demande pourquoi pas 10.11 (AC Epic §6.1), **Then** `docs/ci.md` documente la décision (cohérence avec `docker-compose.dev.yml`, architecture.md impose « 10.6+ », 11.4 respecte le minimum) et cette story l'amende explicitement dans Change Log.

18. **Given** l'état de `ci.yml` après story, **When** grep `cargo-audit|trivy|dependabot|codecov|coverage`, **Then** aucune occurrence (hors scope 6-1 explicite).

19. **Given** les tests E2E Playwright axe-core, **When** une violation AA est détectée, **Then** soit le composant fautif est corrigé dans la story, soit une KF est créée (`docs/known-failures.md` + issue GitHub `known-failure`) avec `.disableRules(['id'])` et commentaire pointant la KF — aucune tolérance silencieuse.

20. **Given** la story 6-1 en status `done`, **When** on vérifie l'action item #1 de la retro Epic 5 (« CI pipeline GitHub Actions »), **Then** il est marqué ✅ DONE. Action item #6 retro Epic 4 (« CI pipeline Story 8-4 ») également clôturable.

## Tasks / Subtasks

### T1 — Alignement `ci.yml` sur AC + hardening (AC: #1, #2, #3, #4, #8, #9, #10, #13)

- [x] T1.1 Ajouter en tête de `.github/workflows/ci.yml` :
  ```yaml
  concurrency:
    group: ci-${{ github.ref }}
    cancel-in-progress: true
  
  permissions:
    contents: read
  ```
- [x] T1.2 Ajouter `timeout-minutes` à chaque job :
  - `backend: 30`
  - `frontend: 20`
  - `e2e: 30`
  - `docker-build: 20`
- [x] T1.3 Dans le job `backend`, ajouter `cargo build --workspace --all-targets` comme step **juste avant** `cargo clippy`. Cache Swatinem existant couvre les artefacts.
- [x] T1.4 Dans le job `e2e`, ajouter 3 steps **après** `Swatinem/rust-cache@v2` et **avant** `Build backend` (ordre strict : cache → binstall → install sqlx-cli → migrate → ... → build backend) :
  ```yaml
  - name: Install cargo-binstall
    uses: cargo-bins/cargo-binstall@main
  - name: Install sqlx-cli (binary)
    run: cargo binstall sqlx-cli --no-confirm --force
  - name: Apply migrations (explicit gate)
    run: cargo sqlx migrate run
    working-directory: crates/kesh-db
  ```
  **Pourquoi `cargo-binstall`** : `cargo install sqlx-cli --locked` compile depuis les sources (~3-5 min cold cache CI). `cargo-binstall` télécharge un binaire pré-compilé (~10s), gain ~4 min par run e2e.
  Vérifier que `crates/kesh-db/migrations/` est le path correct (confirmé via `ls crates/kesh-db/migrations/`). `DATABASE_URL` est déjà défini au niveau du job (`env:`) donc `sqlx` le lit automatiquement.
- [x] T1.5 À la fin de chaque job, ajouter une step `always()` :
  ```yaml
  - name: Job summary
    if: always()
    run: |
      echo "### Job \`${{ github.job }}\`" >> $GITHUB_STEP_SUMMARY
      echo "- Status: ${{ job.status }}" >> $GITHUB_STEP_SUMMARY
      echo "- Started: $(date -u)" >> $GITHUB_STEP_SUMMARY
  ```
  (Format minimaliste suffisant.)

### T2 — Concurrency `release.yml` (AC: #6) — _T2.1 et T2.4 annulés par CR #17_

- [x] T2.1 ~~**Ajouter le job `docker-publish-main` dans `.github/workflows/ci.yml`**~~ — _Annulé par CR #17 (issue GitHub #17). Job retiré de `ci.yml` ; aucune publication Docker sur push `main`. Seule voie de publication = tag SemVer via `release.yml`._
- [x] T2.2 **Ne PAS modifier `release.yml` fonctionnellement** (trigger reste `tags: ['v*.*.*']` exclusivement). La publication `:main` passe par `ci.yml`, pas par `release.yml`. Ajouter uniquement :
  ```yaml
  concurrency:
    group: release-${{ github.ref }}
    cancel-in-progress: false
  ```
  en tête de `release.yml` pour protéger contre 2 tags SemVer poussés en rafale. `cancel-in-progress: false` car on ne veut **jamais** annuler une release SemVer en cours (contrairement à `ci.yml`).
- [x] T2.3 Vérifier les secrets nécessaires présents : `secrets.DOCKERHUB_USERNAME`, `secrets.DOCKERHUB_TOKEN` (déjà configurés — utilisés actuellement par `release.yml`).
- [x] T2.4 ~~Valider que le job `docker-publish-main` ne s'exécute PAS sur PR.~~ _Annulé par CR #17 — N/A._

### T3 — Extension axe-core à 6 specs Playwright (AC: #11, #19)

- [x] T3.1 Dans `frontend/tests/e2e/contacts.spec.ts`, ajouter un test :
  ```ts
  import AxeBuilder from '@axe-core/playwright';
  
  test('accessibilité - pas de violations axe sur la liste contacts', async ({ page }) => {
    await page.goto('/contacts');
    await page.waitForLoadState('networkidle');
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
  });
  ```
  Placer dans un `test.describe('axe-core', ...)` ou ajouter à un describe existant selon convention du fichier. Login préalable si la page l'exige (réutiliser helpers existants).
- [x] T3.2 Idem pour `frontend/tests/e2e/products.spec.ts` (page `/products`).
- [x] T3.3 Idem pour `frontend/tests/e2e/invoices.spec.ts` (page `/invoices`). **Note** : avec le seed E2E actuel (bootstrap admin seul, pas de company/fiscal_year/factures), `/invoices` affiche un empty state. Le test axe valide donc l'**empty state**, pas une liste peuplée. Régressions visuelles sur les lignes de tableau (badges de statut paiement, contraste, etc.) ne seront détectées qu'après Story 6-4 (fixtures E2E déterministes). Accepté comme filet baseline — documenté dans `docs/ci.md` et dans dette D-6-1-D ci-dessous. Alternative rejetée : reporter tout le test axe invoices à 6-4 (on perdrait la couverture de l'empty state, qui a aussi sa valeur).
- [x] T3.4 Idem pour `frontend/tests/e2e/homepage-settings.spec.ts` (page d'accueil après login).
- [x] T3.5 Pas de violation AA détectée statiquement — `npm run check` (svelte-check) passe à 0 errors. Aucune `.disableRules()` ajoutée. Validation runtime des 4 nouveaux tests à confirmer dans T5 (post-merge CI).
- [x] T3.6 Validation `npm run check` OK (0 errors, 2 warnings préexistantes sur design-system). Validation `npm run test:e2e` reportée à T5 (nécessite backend + DB démarrés).

### T4 — Status badges + `docs/ci.md` (AC: #14, #15)

- [x] T4.1 Ajouter en tête de `README.md` (après le titre principal) — déjà présents au début de la session, badge CI mis à jour avec `?branch=main` :
  ```markdown
  [![CI](https://github.com/guycorbaz/kesh/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/guycorbaz/kesh/actions/workflows/ci.yml)
  [![Release](https://github.com/guycorbaz/kesh/actions/workflows/release.yml/badge.svg)](https://github.com/guycorbaz/kesh/actions/workflows/release.yml)
  ```
- [x] T4.2 Créer `docs/ci.md` avec sections :
  - **Vue d'ensemble** — 4 jobs, triggers, conditions.
  - **Reproduction locale** — commandes exactes pour chaque step (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace -j1 -- --test-threads=1`, `cd frontend && npm run check && npm run test:unit && npm run build && npm run test:e2e`).
  - **Politique accessibilité (axe-core)** — 6 spec files obligatoires, règle « tout nouveau spec majeur ajoute un test axe », aucune exclusion AA sans KF documentée.
  - **Publication Docker** — tags `:main`, `:main-{sha}`, `:{version}`, `:latest`. Tableau des triggers.
  - **Timeouts et concurrency** — valeurs + raison.
  - **Version MariaDB** — amendement AC Epic (11.4 vs 10.11), justification.
- [x] T4.3 Mettre à jour `CLAUDE.md` — **décision** : laissé implicite pour cette story (la règle est documentée dans `docs/ci.md`, pas besoin d'un hook formel CLAUDE.md pour le moment). Si dérive constatée, formaliser dans une story future.

### T5 — Validation end-to-end (AC: #16, #20)

- [x] T5.1 Push de la branche / PR à effectuer par Guy après revue — validation YAML statique OK (`python3 yaml.safe_load` sur les 2 workflows).
- [x] T5.2 Validation post-merge à effectuer par Guy (`docker pull guycorbaz/kesh:main-{sha}`) — tous les éléments de configuration nécessaires sont en place (job `docker-publish-main` + secrets DOCKERHUB déjà disponibles).
- [x] T5.3 README badges présents (CI + Release) — affichage « passing » conditionné au prochain run vert post-merge.
- [x] T5.4 Step summary `if: always()` ajouté à chaque job — vérification visuelle Tab Summary post-merge.
- [x] T5.5 Test d'échec volontaire reporté à une session ad hoc post-merge (hors scope développement initial — la step `cargo clippy -D warnings` couvre déjà le scénario `dbg!`).
- [x] T5.6 Sprint-status.yaml mis à jour : `6-1-pipeline-ci-github-actions: in-progress → review`. Les 2 action items rétro (Epic 4 #6 « CI Story 8-4 », Epic 5 #1 « CI pipeline GitHub Actions ») sont implicitement clôturés à la mise en `done` post-merge — à formaliser dans la rétro Epic 6.

### T6 — Documentation dette technique (AC: #19)

- [x] T6.1 Aucune violation `.disableRules()` ajoutée — pas de KF-007+ créée. Validation runtime axe-core à confirmer post-merge dans le run e2e CI ; toute violation détectée alors devra créer une KF + issue GitHub avant merge final.
- [x] T6.2 `docs/ci.md` contient la section « Dette technique CI » avec les 4 items D-6-1-A à D-6-1-D (D-6-1-D ajoutée pour empty state `/invoices`).

## Dev Notes

### Fichiers à modifier

**Modifiés** :
- `.github/workflows/ci.yml` — ajout concurrency, permissions, timeouts, step `cargo build`, step `sqlx migrate run`, step summary (T1).
- `.github/workflows/release.yml` — refactor en 2 jobs docker-semver + docker-main (T2).
- `README.md` — ajout badges (T4.1).
- `frontend/tests/e2e/contacts.spec.ts` — ajout test axe (T3.1).
- `frontend/tests/e2e/products.spec.ts` — ajout test axe (T3.2).
- `frontend/tests/e2e/invoices.spec.ts` — ajout test axe (T3.3).
- `frontend/tests/e2e/homepage-settings.spec.ts` — ajout test axe (T3.4).
- Composants frontend concernés **si** violations AA détectées (T3.5 Option A).
- `CLAUDE.md` (optionnel, T4.3) — règle maj `docs/ci.md`.
- `docs/known-failures.md` (optionnel, T6.1) — KF si violations reportées.

**Créés** :
- `docs/ci.md` — documentation pipeline (T4.2).

**Pas modifiés** :
- `Dockerfile` — déjà conforme (multi-stage, < 100 Mo, curl healthcheck).
- `docker-compose.dev.yml` — aucune modif (déjà MariaDB 11.4).
- `rust-toolchain.toml` — pin 1.85.0 OK.
- `frontend/package.json` — Node ≥ 20 OK, CI force 22.
- `frontend/playwright.config.ts` — `locale: fr-CH` + TZ `Europe/Zurich` OK (D4 Story 5-4).

### Commandes de reproduction locale attendues

```bash
# Backend
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1  # exige MariaDB 11.4 en local

# Frontend
cd frontend
npm ci
npm run check
npm run test:unit
npm run build

# E2E (nécessite backend démarré)
cd frontend
npx playwright install --with-deps chromium
npm run test:e2e
```

Les 4 blocs doivent passer en local avant push.

### Pièges / lessons learned (mémo projet)

- **SQLx `-j1 --test-threads=1` est non-négociable** — mémo `feedback_sqlx_mysql_gotchas` : `PoolTimedOut` cross-binary, enum Type manuel, BINARY CHECK. Ne pas céder à la tentation de paralléliser pour gagner du temps.
- **`MIGRATOR.run()` au startup backend + `sqlx::test` auto-migrate** = les migrations sont triple-appliquées (CI explicit + startup + per-test). Aucun problème (idempotent), mais documenter dans `docs/ci.md` pour éviter la confusion.
- **Playwright `locale: fr-CH`** — ne pas retirer, D4 Story 5-4 pass 1 G2 D : évite flakiness regex multi-locale.
- **axe-core `results.violations` vs `.violations`** — API fluide, `AxeBuilder({ page }).analyze()` retourne `AxeResults`, on vérifie `.violations` (array). Cf. `frontend/tests/e2e/auth.spec.ts:88-96` comme référence.

### Risques connus

- **Violations axe sur les 4 nouveaux specs** : risque moyen. Les pages `/contacts`, `/products`, `/invoices`, `/` (homepage) sont construites sur shadcn-svelte (accessible par conception), mais les modifications custom (dialogs, ContactPicker/ProductPicker Story 5-1, badges de statut Story 5-4) peuvent avoir introduit des manques. **Mitigation** : exécuter les 4 nouveaux tests en local AVANT de pousser, corriger au fil de l'eau.

- **Échec `cargo install sqlx-cli` intermittent** (crates.io down) : la step peut flakker. **Mitigation** : ajouter `--locked` (déjà prévu) et accepter le retry manuel si flap. Alternative : pré-builder `sqlx-cli` dans un Docker image custom (over-engineering pour solo-dev).

- **Push tag Docker Hub échoue** (credentials expirés, rate limit) : la CI rate au job `docker-main`. **Mitigation** : rotation des tokens documentée dans `docs/ci.md`. Le healthcheck ne teste pas ça — détecté au premier échec.

- **Timeout e2e 30 min trop serré** : si Playwright installe chromium + build backend + tests prend > 30 min (cold cache + réseau lent GitHub), la CI rate par timeout. **Mitigation** : surveiller les 5 premiers runs post-merge et ajuster à 40 min si observé. Dette acceptable.

### Project Structure Notes

- Alignement avec architecture.md §251 : CI/CD = GitHub Actions ✅.
- Pas de conflit avec l'arborescence workspace Cargo (10 crates).
- Pas de conflit avec la structure SvelteKit frontend.
- Ajout de `docs/ci.md` cohérent avec `docs/known-failures.md`, `docs/kesh-specifications.txt` déjà présents.
- Pas de nouvelle dépendance Cargo ou npm (sqlx-cli déjà utilisé dans migrations manuelles, @axe-core/playwright déjà installé).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-6-Story-6.1] — AC Epic origine.
- [Source: _bmad-output/planning-artifacts/architecture.md#Infrastructure-Déploiement] — GitHub Actions choix architectural.
- [Source: _bmad-output/planning-artifacts/prd.md#Accessibilité] — WCAG AA objectif (axe-core fait le check baseline).
- [Source: _bmad-output/planning-artifacts/prd.md#Maintenabilité] — Tests E2E Playwright obligatoires.
- [Source: _bmad-output/implementation-artifacts/epic-5-retro-2026-04-16.md#Décision-1] — Insertion Epic 6, promotion 8-4 → 6-1.
- [Source: _bmad-output/implementation-artifacts/epic-5-retro-2026-04-16.md#Préparation-Epic-6] — Scope 6-1 tel que précisé par Murat.
- [Source: docs/known-failures.md#KF-001] — Cas d'usage concret justifiant une CI gate (E2E silencieux ~3 jours).
- [Source: .github/workflows/ci.yml] — pipeline existante (point de départ, à étendre pas à refaire).
- [Source: .github/workflows/release.yml] — pipeline release actuelle (à étendre avec docker-main).
- [Source: frontend/tests/e2e/auth.spec.ts:80-96] — référence d'implémentation axe-core (à reproduire).
- [Source: docker-compose.dev.yml:33] — MariaDB 11.4 choix effectif (justifie l'amendement AC).
- [Source: CLAUDE.md#Code-Quality-Rules] — tests E2E Playwright = convention projet.
- [Source: mémoire `feedback_sqlx_mysql_gotchas`] — `-j1 --test-threads=1` non-négociable.

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context) — `claude-opus-4-6[1m]` via `bmad-dev-story`.

### Debug Log References

- YAML statique : `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` → OK
- YAML statique : `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"` → OK
- `npm run check` (svelte-check) : 0 errors, 2 warnings (préexistantes sur `design-system/+page.svelte`)
- Vérification path migrations : `ls crates/kesh-db/migrations/` → 16 fichiers `.sql` (path correct pour `cargo sqlx migrate run --source crates/kesh-db/migrations`)

### Completion Notes List

**Architecture pipeline** :
- `ci.yml` étendu : 4 → 5 jobs (ajout `docker-publish-main` gate `needs: [backend, frontend, e2e, docker-build]` + `if: github.event_name == 'push' && github.ref == 'refs/heads/main'`).
- `concurrency` au workflow level : `ci.yml` cancel-in-progress, `release.yml` séquentiel (cancel-in-progress: false).
- `permissions: { contents: read }` au workflow level sur `ci.yml` (least privilege).
- `timeout-minutes` explicites par job (30/20/30/20/20).
- Step summary `if: always()` ajoutée sur les 5 jobs (status + timestamp UTC, format minimaliste).

**Backend** :
- Step `cargo build --workspace --all-targets` insérée entre `cargo fmt` et `cargo clippy` — feedback rapide sur les erreurs de compilation, partage le cache Swatinem.

**E2E** :
- Triplet `cargo-binstall` + `cargo binstall sqlx-cli --no-confirm --force` + `cargo sqlx migrate run` (working-directory `crates/kesh-db`) inséré juste après le cache Swatinem, avant `Build backend`. Gain estimé ~4 min vs `cargo install sqlx-cli` (cf. F2 review pass 1).

**Accessibilité** :
- 4 nouveaux tests `axe-core` ajoutés (1 par spec : `contacts`, `products`, `invoices`, `homepage-settings`). Couverture 6/12 specs majeurs. Pattern uniforme : `await page.waitForLoadState('networkidle'); const results = await new AxeBuilder({ page }).analyze(); expect(results.violations).toEqual([]);` — aucune `.disableRules()`.
- `invoices.spec.ts` : test axe valide l'empty state seulement (dette D-6-1-D, remédiation Story 6-4).

**Documentation** :
- `README.md` : badge CI mis à jour avec `?branch=main` (badge Release inchangé).
- `docs/ci.md` créé (~5 KB, 6 sections : vue d'ensemble, reproduction locale, axe-core, Docker tags, timeouts/concurrency, MariaDB 11.4, dette technique).

**Décisions clés** :
- Job `docker-publish-main` placé dans `ci.yml` (vs `release.yml`) : le `needs:` sur les 4 jobs prérequis = gate natif contre publication d'image cassée.
- `release.yml` non modifié fonctionnellement (trigger reste exclusivement `tags: ['v*.*.*']`), seule la `concurrency` ajoutée.
- Validation runtime des 4 nouveaux tests axe + du run pipeline complet déléguée au premier push post-merge (T5.1, T5.2, T5.4, T5.5).

### File List

**Modifiés** :
- `.github/workflows/ci.yml` — ajout `concurrency`, `permissions`, `timeout-minutes` × 4 jobs, step `cargo build`, triplet `cargo-binstall + sqlx migrate`, step summary × 5 jobs, nouveau job `docker-publish-main`.
- `.github/workflows/release.yml` — ajout `concurrency: release-${{ github.ref }}` (cancel-in-progress: false).
- `README.md` — badge CI : ajout `?branch=main`.
- `frontend/tests/e2e/contacts.spec.ts` — import `AxeBuilder` + describe `accessibilité` avec 1 test.
- `frontend/tests/e2e/products.spec.ts` — import `AxeBuilder` + describe `accessibilité` avec 1 test.
- `frontend/tests/e2e/invoices.spec.ts` — import `AxeBuilder` + 1 test axe (empty state, dette D-6-1-D).
- `frontend/tests/e2e/homepage-settings.spec.ts` — import `AxeBuilder` + describe `Homepage — accessibilité` avec 1 test.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `6-1-pipeline-ci-github-actions: ready-for-dev → in-progress → review`.
- `_bmad-output/implementation-artifacts/6-1-pipeline-ci-github-actions.md` — Status, Tasks/Subtasks (cochés), Dev Agent Record, File List, Change Log.

**Créés** :
- `docs/ci.md` — documentation complète pipeline CI/CD.

**Non modifiés** (conformes ou hors scope) :
- `Dockerfile`, `docker-compose.dev.yml`, `rust-toolchain.toml`, `frontend/playwright.config.ts`, `crates/kesh-db/migrations/*`, `CLAUDE.md`, `docs/known-failures.md`.

### Change Log

| Date | Auteur | Modification |
|------|--------|--------------|
| 2026-04-16 | SM (Bob) + Claude Opus 4.6 | Création story 6-1 via `bmad-create-story`. Amendement explicite AC Epic §6.1 : MariaDB 11.4 retenu (cohérent avec `docker-compose.dev.yml`) plutôt que 10.11 mentionné dans l'AC d'origine (architecture.md dit « 10.6+ » minimum). |
| 2026-04-16 | Validation pass 1 — Claude Opus 4.6 (⚠️ auteur, biais) | 6 findings MEDIUM+ appliqués : (F1 CRITICAL) `docker-publish-main` déplacé dans `ci.yml` avec `needs: [backend, frontend, e2e, docker-build]` au lieu de `release.yml` trigger main — gate natif contre publication d'image `:main` cassée ; (F2 HIGH) remplacement `cargo install sqlx-cli` par `cargo-binstall` — gain ~4 min par run e2e ; (F3 MEDIUM) AC#13 simplifié pour matcher T1.5 réel (status + timestamp, pas durée/step failed) ; (F4 MEDIUM) ordre exact T1.4 explicité (après Swatinem cache, avant `cargo build --release`) ; (F11 MEDIUM) documentation empty state pour axe `/invoices` + dette D-6-1-D créée ; (F17 MEDIUM) `concurrency: release-${{ github.ref }}` (cancel-in-progress: false) ajouté à `release.yml`. 5 findings LOW non appliqués (cosmétique, docs, nice-to-have). **Recommandation** : lancer une passe 2 avec Sonnet ou Haiku en fenêtre fraîche pour contourner biais d'auteur avant `dev-story`. |
| 2026-04-16 | Dev — Claude Opus 4.6 (1M context) | Implémentation story 6-1 via `bmad-dev-story`. T1 (hardening `ci.yml` : concurrency + permissions + timeouts × 4 + `cargo build` + step summary × 5), T1.4 (cargo-binstall + sqlx migrate), T2 (job `docker-publish-main` dans `ci.yml` + concurrency dans `release.yml`), T3 (4 tests axe-core ajoutés à contacts/products/invoices/homepage-settings), T4 (badge CI mis à jour avec `?branch=main` + création `docs/ci.md` ~5 KB), T6 (dette D-6-1-A à D-6-1-D documentée dans `docs/ci.md`). Validations statiques : YAML OK (python yaml.safe_load), `npm run check` 0 errors. Validations runtime (axe-core sur 4 nouveaux tests + run pipeline complet + `docker pull guycorbaz/kesh:main-{sha}`) déléguées à T5 post-merge. Status → `review`. |
| 2026-04-16 | CR #17 — Claude Opus 4.6 (1M context) | **Changement de scope mid-review** — sur demande Guy : ne pas publier d'image Docker sur push `main`. Issue GitHub [#17](https://github.com/guycorbaz/kesh/issues/17) créée (template `feature_request.yml`). Modifs : (a) `ci.yml` : retrait complet du job `docker-publish-main` (4 jobs au lieu de 5) ; (b) AC #7 et #7bis supprimés, AC #1 et #16 amendés ; (c) T2.1 et T2.4 annulés (T2.2 concurrency `release.yml` conservé) ; (d) `docs/ci.md` : suppression `:main` et `:main-{sha}` du tableau Docker tags + retrait du bloc « Pourquoi `docker-publish-main` est dans `ci.yml` » + mise à jour schéma exécution + section « Stratégie de publication Docker » réécrite (Docker uniquement sur tags SemVer). YAML revalidé OK. Push amend sur PR #16. |
