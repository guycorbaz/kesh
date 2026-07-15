# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Kesh** is a Swiss personal and small business accounting software. It is currently in the design/planning phase, developed using the BMAD (Breakthrough Method of Agile AI-driven Development) framework. No application code has been written yet — the repository contains BMAD framework assets and design artifacts.

**Target stack**: Rust backend (Axum), Svelte frontend, MariaDB, web app only (no Tauri). Configuration via environment variables. Deployment via docker-compose.

**PRD**: `_bmad-output/planning-artifacts/prd.md` — Swiss accounting focus: QR Bill 2.2, pain.001.001.03, CAMT.053.001.04, multilingual (FR/DE/IT/EN).

## Communication

The user (Guy) works in **French**. All conversation and document output should be in French unless otherwise specified.

## Repository Structure

```
_bmad/                  # BMAD framework (agents, workflows, skills, config)
  bmm/                  # Build Method Methodology — discovery → implementation phases
  wds/                  # Workflow Design System — UX and design workflows
  cis/                  # Creative Innovation Strategy — strategic methodologies
  tea/                  # Test Architecture — QA and testing workflows
  bmb/                  # Builder — meta-skills for creating agents/workflows
  core/                 # Universal skills (brainstorming, reviews, editing)
  _config/              # CSV manifests (agents, skills, workflows, files)
  _memory/              # Persistent memory for sidecar agents
_bmad-output/           # Generated artifacts (planning, implementation, test)
design-artifacts/       # Project deliverables by phase (A through G)
  A-Product-Brief/      # Product positioning
  B-Trigger-Map/        # Business goals → user psychology
  C-UX-Scenarios/       # User interaction scenarios
  D-Design-System/      # UI components and tokens
  E-PRD/                # Requirements + design deliveries
  F-Testing/            # Test plans
  G-Product-Development/ # Implementation artifacts
docs/                   # Project documentation
.claude/skills/         # 118 installed BMAD skills for Claude Code
```

## BMAD Architecture

**Agents** are named personas (PM, Developer, Architect, QA, etc.) defined in `.md` files with menus that invoke skills or workflows. **Skills** are self-contained capabilities (52 total). **Workflows** are multi-step stateful processes (51 total) using step-file architecture — each step in a separate file, loaded just-in-time. Progress is tracked in document frontmatter.

Key manifests in `_bmad/_config/`: `agent-manifest.csv`, `skill-manifest.csv`, `workflow-manifest.csv`.

BMAD module config: `_bmad/bmm/config.yaml` — defines project name, user name, language preferences, and output paths.

## Key Patterns

- Workflows execute steps sequentially — never skip steps
- State is stored in document frontmatter (`stepsCompleted` array)
- Design artifacts follow the A→G phase progression
- Skills are invoked via `/skill-name` slash commands in Claude Code
- All generated output goes to `_bmad-output/` (not mixed with framework files)

## Code Quality Rules

- **DRY (Don't Repeat Yourself)** — No duplicated code. Extract shared logic into reusable functions/modules.
- **Documentation** — Source code must be documented following best practices: public APIs, complex logic, module-level docs. Rust: use `///` doc comments. Svelte: use JSDoc where appropriate.
- **Testing** — Test everything that can be tested. Unit tests for all business logic (especially the accounting engine, VAT calculations, and financial computations). Integration tests for parsers (CAMT.053, QR Bill, pain.001).
- **E2E Testing** — Use Playwright for all end-to-end tests. Each user journey from the PRD maps to a Playwright test scenario.
- **Batch API conventions** — Pour les endpoints batch (style `accept_batch` qui retournent `{ accepted, failed }`), cf. §"Pattern batch — FailedProposal per-proposal" sous §"Review Iteration Rule".

## Test Locally First

**Règle « Test Locally First »** : avant chaque `git push` qui ouvre ou met à jour une PR, exécuter localement la même série de checks que la CI. La CI rouge sur un détail rattrapable localement (ex. `cargo fmt --check`) coûte un cycle de revue + re-run + cache invalidé. Le coût local est de l'ordre de la minute si le workspace est chaud.

**Carry-forward Epic 6 retro** (2026-04-20) — codifié 2026-05-03 dans le prep sprint Epic 8, après la PR #66 retombée rouge sur `cargo fmt --check` (long-line dans `bank_imports.rs`, fix trivial mais coûte un cycle CI complet).

### Backend (Rust)

À lancer depuis la racine du repo. Ces 4 checks sont **exactement** ceux de `Backend (Rust)` dans `.github/workflows/ci.yml`.

```sh
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Note : la CI utilise `cargo test --workspace -j1 -- --test-threads=1` pour serializer les tests d'intégration DB. En local sans MariaDB démarré, `cargo test --workspace` (parallèle) suffit pour couvrir les tests unitaires sans I/O ; lancer le mode serial uniquement si la modif touche `kesh-db` ou les tests d'intégration.

#### Gate rapide — `cargo-nextest` (recommandé si MariaDB démarré)

Alternative **1,40× plus rapide** au `cargo test -j1 --test-threads=1` (mesuré 2026-07-13 : **54 min → 38 min** sur la suite complète, 1802 tests, 0 flake). Un script enveloppe fmt + clippy + nextest :

```sh
scripts/test-fast.sh            # fmt + clippy + nextest (défaut)
scripts/test-fast.sh --no-lint  # nextest seul (itération rapide)
scripts/test-fast.sh --ci       # profil ci (retries=1, fail-fast off)
```

Pré-requis : `cargo install cargo-nextest` (ou binaire prébuilt `https://get.nexte.st`) + MariaDB dev démarré. Config dans `.config/nextest.toml`.

**Pourquoi seulement 1,40× et pas 6× ?** Le goulot n'est pas le CPU mais MariaDB : chaque test `#[sqlx::test]` (~894) crée une base éphémère et y **rejoue les 51 migrations** (DDL sérialisé par les metadata-locks). Au-delà de **6 threads la contention devient contre-productive** (32 threads → 3 flakes `reconciliation_*_e2e` KF-038 #228 + tests « slow », run cassé). Le plafond est donc figé à 6 dans la config. Le vrai levier (squash du schéma de test + durabilité MariaDB relâchée sur la DB jetable) est suivi dans l'**issue #251** — tant qu'il n'est pas livré, nextest = gain modeste + stabilité, pas une révolution.

### Frontend (Svelte)

À lancer depuis `frontend/`. Mêmes étapes que `Frontend (Svelte)` dans la CI.

```sh
cd frontend
npm run check
npm run lint-i18n-ownership
npm run test:unit
npm run build
```

### E2E (Playwright)

**Pas exécuté par la CI principale** (cf. `ci.yml`) — donc d'autant plus critique en local si la modif touche le frontend ou les routes API consommées par les pages.

```sh
cd frontend
npm run test:e2e
```

Pré-requis : MariaDB démarré + seed CI appliqué + Playwright browsers installés (cf. `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` sur Ubuntu 26.04+ — limitation upstream Playwright ≤ 1.49).

### Quand sauter

Cette règle ne s'applique pas aux **commits doc-only** (markdown, yaml de planning, README, CLAUDE.md lui-même) qui ne touchent pas de code exécutable. Pour ces commits, la CI elle-même est généralement no-op (pas de Rust ni de TypeScript modifié → cache hit instantané).

## Review Iteration Rule

**Règle de remédiation des revues (code review et spec validate)** :

Tant qu'une passe de revue remonte **au moins un finding de sévérité supérieure à LOW** (c'est-à-dire `CRITICAL`, `HIGH`, ou `MEDIUM`), on **relance une nouvelle passe de revue** après application des patches. Le critère d'arrêt est :

- **Uniquement des findings de sévérité `LOW`** (nits cosmétiques, améliorations de lisibilité, documentation mineure), OU
- **Maximum 8 passes atteint** (limite de budget LLM)

Pour chaque nouvelle passe de revue sur la même story :
- **Utiliser un LLM différent** de la passe précédente si possible (cycle Sonnet → Haiku → Opus → Sonnet, validé empiriquement Epic 9 retrospective Insight I1 sur 3 cycles complets), afin de contourner le biais d'auteur sur les patches qu'on vient d'appliquer. Les régressions introduites par la remédiation ne sont souvent détectables que par un modèle orthogonal.
- **Fenêtre de contexte fraîche** — ne pas réutiliser le contexte de la passe précédente.
- **Patches appliqués avant passe N+1** : chaque finding trouvé en passe N est remédié avant relancer la passe N+1.
- **Documenter dans le Change Log final** (pas une entrée par passe) : résumé du trend numérique (passe 1: X findings → passe 2: Y → ... → passe N: 0 > LOW), modèles LLM utilisés, décisions de reclassement.

**Boucle automatique** :
- `bmad-create-story validate` : relancer automatiquement en boucle après chaque passe (LLM différent, patches appliqués, contexte frais) jusqu'à atteindre 0 CRITICAL/HIGH/MEDIUM OU 8 passes.
- `bmad-code-review` : appliqué après implémentation (`dev-story` complétée), même boucle.

Cette règle s'applique à :
- `bmad-create-story validate` (revue de spec multi-passes)
- `bmad-code-review` (revue de code adversariale)
- Toute revue adversariale similaire où le budget LLM le permet

**Exception** : si un finding `MEDIUM+` est explicitement reclassé en **dette technique documentée** (dans une section `Security debt` / `Performance debt` / équivalente du story file ou des Dev Notes) avec un propriétaire et une story de remédiation planifiée, il compte comme « résolu » pour cette itération.

### Haiku-specific guardrails — grep ground-truth obligatoire

**Symptôme observé** : les reviewers Haiku 4.5 (`BlindHunter` / `EdgeCaseHunter` typiquement) peuvent affirmer **CRITICAL** ou **HIGH** « REGRESSION-P1 — patch X n'a pas été appliqué » sur un diff combiné multi-commit, alors que le patch **est** présent dans le fichier.

**Cause racine** : Haiku traite mal l'indexation des line numbers d'un diff `git show A B` quand le 2e commit `B` re-touche des hunks du 1er commit `A`. Les line numbers du 2e hunk correspondent au file post-A (pas au file final post-B), et Haiku peut chercher la ligne X et y voir le contenu de A, ratant le patch de B.

**Règle d'application** — pour tout finding `CRITICAL` ou `HIGH` affirmant **soit l'absence d'un code attendu** (« patch X n'a pas été appliqué »), **soit la présence d'un anti-pattern non-corrigé** (« ligne Y contient encore un `unwrap()` non-sécurisé »), l'orchestrateur **DOIT** vérifier ground-truth avant de traiter le finding comme réel :

- **Pattern textuel grepable sur une ligne précise** : exécuter `grep -nF "<chaîne issue du patch>" <file>` — le flag `-F` (fixed-string) est **obligatoire** pour éviter les faux-positifs sur les métacaractères regex (`.`, `*`, `[`, `(`, `\`, etc.) qui apparaissent fréquemment dans du code Rust/TS (e.g. `Vec<i64>`, `amount * rate`, `unwrap_or(0.0)`).
- **Pattern multi-ligne** (struct sur 4 lignes, bloc `if let` indenté, chaîne de méthodes) : `grep -n` opère ligne par ligne. Choisir une **ligne unique représentative** discriminante (ouverture de la struct + 1 mot unique), ou utiliser `grep -nFA <N>` pour récupérer les N lignes suivantes et vérifier le bloc complet (N typiquement 3-5 ; ajuster selon la longueur estimée du bloc patché).
- **Finding architectural ou comportement runtime** (e.g. « la fonction ne retourne pas d'erreur en cas d'overflow », « le flux d'erreur descend silencieusement à travers 3 niveaux d'`?` ») où aucun pattern textuel précis ne suffit : **vérification manuelle par `Read` direct** du fichier concerné + inspection ciblée du flux. Documenter la vérification dans le Change Log de la story (section `### Pass N review`) avec extrait du code lu. **Note** : un ordre de `app.use(...)` middleware ou la séquence d'appels dans une fonction reste textuel grepable (`grep -nF "app.use"` retourne l'ordre exact) — réserver « architectural » aux flux cross-fonction ou cross-fichier sans pattern unique discriminant.
- **Résultat** :
  - Vérification confirme l'observation (pattern absent OU anti-pattern présent OU comportement architectural confirmé) → finding réel, appliquer le patch.
  - Vérification réfute l'observation → **dismiss** comme faux-positif Haiku. Documenter dans le Change Log (e.g. « BH2-1 CRITICAL réfuté par `grep -nF` ligne X — faux-positif Haiku indexing diff multi-commit »).

**Mitigation préférée** : à partir de la Pass 2 d'un cycle review, donner à Haiku un **diff unique** (le commit final `HEAD vs main` aplati) plutôt que la séquence de commits intermédiaires. Évite la confusion d'indexation à la source.

**Scope du bug** : spécifique Haiku 4.5. Sonnet 4.6 et Opus 4.7 ne reproduisent pas l'erreur d'indexation diff multi-commit (validé empiriquement Epic 9 et antérieur). Pour autant, la discipline grep ground-truth s'applique à **tous** les modèles par hygiène — Haiku reste le cas pathologique connu, les autres modèles l'appliquent par défense en profondeur.

*(cf. memory `feedback_haiku_review_diff_combined`, validé empiriquement Stories 8-5a-bis Pass 2 Haiku 2026-05-12 [BH2-1 missing scale() validation ligne 2120 + BH2-2 missing != bank_ledger guard ligne 2172] et 9-2b Pass 2 Haiku 2026-05-15 [route.continue() sans await ligne 66 + resolveDownload! sans validation ligne 201] — 4 hallucinations CRITICAL/HIGH réfutées par grep ground-truth)*

### Pattern batch — FailedProposal per-proposal

**Règle** : pour tout endpoint type `accept_batch` (qui traite N proposals/operations en une seule requête HTTP et retourne un body `{ accepted: [...], failed: [...] }`), **aucune erreur per-proposal ne doit escalader en `AppError` global** retournée comme HTTP error code (4xx/5xx). Chaque erreur d'une proposal individuelle est encapsulée en `FailedProposal` dans le `failed[]` du response body, avec **HTTP 200 OK** au niveau de la requête (un succès partiel reste un succès HTTP).

**Exceptions explicites** — les `AppError` global restent autorisées **uniquement** pour les erreurs qui invalident la requête entière en amont du traitement per-proposal :

- `401 Unauthorized` — auth middleware (token absent / expiré)
- `403 Forbidden` — RBAC global (rôle insuffisant pour l'endpoint)
- `400 Bad Request` — body parse fail / schéma JSON invalide
- `500 Internal Server Error` — DB pool fermé, panic, IO catastrophique

Toute erreur **qui dépend de la proposal individuelle** (validation `amount > 0`, FK manquant, race condition optimistic lock, currency mismatch, business rule violation) → `FailedProposal` per-proposal.

**Champs obligatoires de `FailedProposal`** (signature canonique Epic 8) :

- **identifiant business de la proposition** (e.g. `bank_transaction_id: i64` pour la réconciliation Epic 8 ; pour pain.001 paiements batch Epic 11 ce sera probablement `payment_id` ou équivalent, à adapter selon le type de proposition). **Anti-pattern** : NE PAS utiliser un index positionnel `proposal_index: usize` — fragile à toute réorganisation du batch par le client.
- `error_code: String` — constante canonique recommandée (e.g. `"BANK_ACCOUNT_NOT_CONFIGURED"`, `"RECONCILIATION_RULE_NO_LONGER_MATCHES"`). **JAMAIS** interpolation `format!("error: {}", e)` ; pour contexte dynamique utiliser le champ `details`.
- `details: Option<serde_json::Value>` — JSON object additionnel pour contexte spécifique au code (e.g. `{ "bankAccountId": 17 }`).

**Garde-fou défensif** — dans un `match` exhaustif sur les variants d'un type sum (`Rule`, `ProposalType`, etc.), **NE PAS** utiliser `unreachable!()` aux sites variants (un `unreachable!()` crashe la Tokio task sans log si un futur refactor introduit un variant manquant). Préférer `tracing::error!(...) + retour d'AppError::Internal(...)`. **Important** : cette `AppError::Internal` tombe sous l'exception globale ligne « 500 Internal Server Error » ci-dessus — un variant manquant est un **bug structurel détecté à l'exécution** (refactor incomplet : variant `Rule::NewType` ajouté au code mais oublié dans le match correspondant ; signature de fonction modifiée mais call site non-mis-à-jour ; ajout d'un type intentionnellement non-Send qui passe Send par erreur). À l'inverse, une « validation métier non-anticipée » du type *« amount peut être négatif dans certaines factures »* ou *« currency manquante sur une transaction legacy »* reste une erreur business per-proposal → `FailedProposal`. Le critère décidable : **est-ce que le code compile** ? Si oui mais le `match` est incomplet à cause d'un refactor → bug structurel → `AppError::Internal`. Si la donnée d'entrée est inattendue mais le code compile et le `match` est exhaustif → erreur métier → `FailedProposal`.

**Référence canonique** : `accept_one_invoice` (Story 8-4), `accept_one_split` (Story 8-5a-bis), `accept_one_rule` (Story 8-5b). Le pattern est inviolable sur ces 3 implémentations Epic 8.

**Réutilisation prévue** : Epic 11 (pain.001 paiements batch — un fichier XML contient N transactions, chaque transaction peut échouer indépendamment) + tout endpoint futur retournant `{ accepted, failed }`. **Note** : CAMT.053 (Epic 12) est de l'**import** raw (parser → INSERT bank_transactions sans décision utilisateur per-transaction), donc ne suit PAS ce pattern — sa réconciliation post-import utilise déjà l'API Epic 8 `accept_batch` qui implémente ce pattern.

*(pattern hérité Epic 8 — cf. rétrospective Epic 8 Insight I2 + Story 8-5b Pass 4 ECH4-1 correction `BANK_ACCOUNT_NOT_CONFIGURED` `200 + failed[]` au lieu de `412` AppError global)*

### Règle de splitting préventif

**Si une story qui n'est pas encore en spec validate satisfait l'un de ces deux critères, la splitter en sous-stories avant de lancer `bmad-create-story`** :

- **Scope cross-cutting** : la story touche **plus de 5 modules** distincts (crates Rust, packages npm, ou modules métier de premier niveau type `kesh-core/accounting`, `kesh-api/routes/invoices`, `frontend/src/features/invoices`).
- **Profondeur d'incertitude** : `bmad-create-story validate` boucle au-delà de **4 passes adversariales** sans converger sur 0 finding > LOW.

Pourquoi : dans les deux cas, la story est trop large pour être tenue dans un seul mental-model adversarial fiable. Les régressions introduites par les patches d'une passe N deviennent invisibles aux passes N+1 (saturation contextuelle) et finissent par être détectées seulement post-merge en code review ou pire, en prod.

**Précédent : Story 7-1** (KF-002 audit + multi-tenant scoping refactor) — 4 passes spec validate Opus/Sonnet/Haiku/Opus avant convergence, scope étalé sur 7+ modules (`kesh-core`, `kesh-db`, `kesh-api/routes/{invoices,journal_entries,companies,...}`). Lesson rétro Epic 7 : avec un split en 7-1a (audit/baseline) + 7-1b (scoping pattern) + 7-1c (rollout par module), chaque sous-story aurait pu converger en ≤ 2 passes.

**Comment splitter** : dégager d'abord un story-zero qui pose le **pattern** (helper, type, helper test) sur 1-2 modules pilotes, puis enchaîner des sous-stories de **rollout** strictement mécaniques (apply pattern aux N-2 modules restants). La sous-story rollout est revue au file-by-file plutôt qu'en passes adversariales globales.

**Exception** : si un *split forcé* introduit des cycles de dépendance Cargo ou des merges intermédiaires impossibles à tester en isolation, garder la story unique et documenter explicitement la dérogation dans le story file (section `Dérogation règle de splitting` avec justification + accepted risk).

## Tech debt management — zero carry-forward policy

**Règle projet** : pas de cumul de dette technique inter-epic. À chaque rétrospective d'epic, **toutes les vraies dettes (catégorie A ci-dessous) doivent être adressées (fix appliqué OU explicitement reclassées en catégorie B avec justification + story de remédiation planifiée)** avant le kickoff de l'Epic N+1.

### Triage obligatoire — 3 catégories à chaque rétrospective

- **Catégorie A — vraie dette** : bug latent, incohérence non-documentée, action retrospective non-complétée d'un Epic antérieur, KF dormante GitHub ouverte (sans label `v0.2-milestone`). **DOIT être fixée** avant kickoff Epic suivant.
- **Catégorie B — limitation v0.2 légitime** : feature ou limitation documentée avec scope explicite (style `L1` / `L2` / ... dans story file ou Dev Notes) **et** mécanisme de planification de la remédiation (au choix : story dédiée créée pour un Epic futur, OU label `v0.2-milestone` sur l'issue GitHub qui tient lieu de planification implicite — la story de remédiation sera créée au plus tard au kickoff de l'Epic qui consommera le backlog v0.2). Acceptable indéfiniment tant que tracée. **Exception au reclassement automatique** : si une KF labellée `v0.2-milestone` est **également** marquée gate bloquant v0.1 (e.g. dans le PRD, une story spec, ou une décision de release explicite), le gate v0.1 prime → la KF reste catégorie **A** jusqu'à fix ou levée explicite du gate.
- **Catégorie C — décision design intentionnelle** : pattern volontaire (e.g. tables exclues d'un export pour raison de sécurité, INNER JOIN avec FK garante, audit-trail-only acceptée v0.1). **Pas une dette.**

### Critical path

Les items catégorie **A** passent du « cleanup parallèle optionnel » au **bloquant kickoff Epic suivant** dans la section Action Items de chaque rétrospective. Cette transition est non-négociable : si un item A traîne au moment du kickoff, soit on le fixe immédiatement, soit on le reclasse formellement en B (avec justification écrite + story remédiation planifiée).

**Triage hors fenêtre rétrospective** — si une dette catégorie A est découverte **en cours d'Epic N+1** (e.g. semaine 2 d'un Epic feature, pas pendant une rétrospective), triage immédiat selon sévérité :

- **Critique pour l'Epic en cours** (bloque une story active OU introduit une régression imminente sur des baselines vertes) → créer une story de fix **dans l'Epic N+1 en cours**, traiter immédiatement.
- **Non-critique pour l'Epic en cours** → ajouter à la liste des items A bloquant le **kickoff de l'Epic N+2** (équivalent du carry-forward d'un Epic à l'autre, c'est l'exception au zero carry-forward pour les découvertes hors-rétrospective). Documenter dans la rétrospective courante quand elle viendra.

L'arbitrage de sévérité est fait par le Project Lead au moment de la découverte, pas par l'orchestrateur LLM.

**Soupape — item A résistant** : si un item A résiste à **3+ tentatives de fix successives** (1 tentative = 1 cycle complet `bmad-dev-story` → `bmad-code-review` où le fix échoue à résoudre l'item OU introduit une régression sur d'autres baselines), il peut être **exceptionnellement** reclassé en catégorie B avec :

- Justification écrite « résistance constatée après N tentatives » dans la rétrospective, avec liste des cycles et raisons d'échec.
- Story de remédiation planifiée **Epic N+2** (pas Epic N+1 — laisser le temps d'investigation hors charge feature).
- Issue GitHub labellée `technical-debt` + `v0.2-milestone` + commentaire détaillé des tentatives de fix et de leurs échecs.
- Suivi spécifique (revue à chaque rétrospective Epic N+1, Epic N+2) pour éviter que la résistance ne devienne un `wontfix` de facto.

Cette soupape est l'**exception explicitement codifiée** à la règle zero carry-forward du paragraphe initial — pas une contradiction. L'item reclassé en B reste tracé (story de remédiation Epic N+2 + label GitHub + suivi rétro), ce qui le distingue d'un report silencieux. Si l'Epic N+1 est lui-même un Epic « Technical Debt Closure » dédié, la soupape Epic N+2 reste applicable (la résistance documentée justifie de laisser le fix mûrir un cycle de plus).

### Pattern Epic dédié cleanup

Si le volume d'items catégorie A est élevé (seuil indicatif : **> 8 items** OU couvre **> 2 axes distincts** — e.g. KFs + code consistency + process codification), créer un **Epic dédié de type « Technical Debt Closure »** plutôt qu'un méga-Epic suivant mélangeant feature + dette. Précédents projet :

- **Epic 7 historique « Technical Debt Closure »** — KF-001..007 fermées pré-Epic 8. Pattern de référence.
- **Epic 9.5 « Technical Debt Closure »** — ~13 items A post-Epic 9, 4 stories. Applique cette politique de manière systématique.

### Distinction au triage

Les limitations documentées (style `L1-L18` dans story files) qualifient en catégorie **B** si **et seulement si** scope explicite **+** story de remédiation planifiée. Sinon → catégorie **A** (dette implicite). KFs ouvertes GitHub Issues = candidates catégorie A par défaut sauf labelling explicite `v0.2-milestone` (cohérent §"Issue Tracking Rule").

*(politique formalisée 2026-05-17 rétrospective Epic 9 — cf. memory `feedback_zero_tech_debt_carryforward` + pattern Epic 7 historique « Technical Debt Closure »)*

## Issue Tracking Rule

**Règle de traçage des CR, KF et bug reports** :

**GitHub Issues est l'unique source de vérité.** Toute nouvelle découverte d'un **CR (Change Request)**, d'une **KF (Known Failure)** ou d'un **bug report** DOIT être créée comme GitHub Issue sur [guycorbaz/kesh/issues](https://github.com/guycorbaz/kesh/issues) en utilisant le template approprié dans `.github/ISSUE_TEMPLATE/`.

**Pas de tracking local en parallèle** — aucun fichier dans le repo (Markdown, YAML, tableau de story) ne doit maintenir sa propre liste de KF/CR/bugs. Pas de double-tracking, pas de sync bidirectionnelle, pas de dérive de source de vérité.

| Type | Template | Labels appliqués par le template |
|------|----------|----------------------------------|
| Bug report | `bug_report.yml` | `bug`, `triage` |
| KF | `known_failure.yml` | `known-failure`, `triage` (+ `technical-debt` à ajouter manuellement si dette persistante) |
| CR / feature request | `feature_request.yml` | `enhancement`, `triage` |

Titre homogène pour les KF : `[KF-NNN] description` — facilite la recherche visuelle dans la liste d'issues.

### Quand créer une issue

- **Bug report** : dès qu'un comportement incorrect est reproduit, **hors du flux normal de dev d'une story en cours**. Si le bug est découvert pendant l'implémentation d'une story liée, le corriger directement dans la story et le documenter dans le Change Log de la story.
- **KF** : dès qu'un test cassé ou un comportement défaillant est détecté **hors scope du travail courant**.
- **CR** : **avant** tout changement de scope qui modifie le PRD ou les AC d'une story déjà validée (`done` ou en `review`). Ne pas faire de modification silencieuse du scope.

### Commits qui adressent une issue

Chaque commit qui adresse partiellement ou totalement une issue doit mentionner son numéro :
- **Fermer l'issue** : `fix(api): close IDOR on contacts (#2)` ou `... (closes #2)` ou `... (fixes #2)` — GitHub ferme automatiquement l'issue au merge sur `main`.
- **Référencer sans fermer** : `fix: round invoice totals (refs #42)` — lie le commit à l'issue sans la fermer.

### Legacy

Deux fichiers dans `docs/` sont **archivés et ne doivent plus être mis à jour** — ils ne servent que de trace historique :

- `docs/change_request.md` — archivé depuis 2026-04-16, 8 CR migrés sur GitHub.
- `docs/known-failures.md` — archivé depuis 2026-04-18, 7 KF migrées sur GitHub (KF-001 à KF-007).

Toute nouvelle KF/CR/bug → GitHub uniquement. Ne **pas** rouvrir ces fichiers pour y ajouter des entrées.

## Migration breaking policy

**Politique introduite Story 10-2** — protège contre les downgrades silencieux corrupteurs.

**(P1) Définition** : Une migration est **breaking** si elle introduit un état du schéma qu'un binaire Kesh antérieur ne peut **plus** consommer correctement (ex. `DROP COLUMN` d'une colonne lue par un SELECT du binaire antérieur, `RENAME TABLE`, `MODIFY COLUMN` ou `CHANGE COLUMN` introduisant un type incompatible ex. DECIMAL → VARCHAR). La majorité des migrations (`ADD COLUMN` nullable, `ADD INDEX`, `CREATE TABLE` de nouvelle entité) sont **non-breaking** car les anciens binaires les ignorent.

**(P2) Procédure de bump** : Quand une migration breaking est introduite, la migration elle-même DOIT contenir, **en dernière instruction**, un `UPDATE _kesh_version SET kesh_version_min_required = '<version-de-la-PR-qui-introduit-la-migration>' WHERE id = 1;`. La version est figée dans le SQL (pas via paramètre runtime), comme la version d'origine `'0.1.0'` figée dans `crates/kesh-db/migrations/20260522000001_kesh_version.sql`.

**(P2-bis) Le bump `min_required` va TOUJOURS de pair avec le bump de version Cargo du workspace** — codifié 2026-07-14 après un incident réel (Story 21-3, 1er bump `min_required` du repo). Si `min_required` passe à `X.Y.Z`, **tous les crates du workspace** (`crates/*/Cargo.toml`, `version = "…"`) DOIVENT être ≥ `X.Y.Z` **dans le même commit/PR**. Sinon le binaire courant (dont `env!("CARGO_PKG_VERSION")` est encore l'ancienne version) devient **plus ancien que sa propre DB** → `check_downgrade_protection` (`main.rs`) **refuse le boot** ET l'import backup (`admin_backup`, le manifeste porte `min_required`). **Piège de détection** : ce défaut est **invisible en `bmad-create-story validate` (statique)** — il ne se manifeste qu'au **runtime** (boot / import), donc au **gate workspace complet** (les suites `admin_backup_e2e` / `admin_full_import_e2e` / `migrations_fresh_install` l'attrapent). Ne JAMAIS marquer une story avec bump `min_required` « done » sans avoir passé le gate runtime complet. Le bump `min_required` et le bump Cargo sont **les deux moitiés de la même action de version**.

**(P3) Garde-fou code review** : Si une PR introduit une migration `DROP TABLE`, `DROP COLUMN`, `RENAME TABLE`, `RENAME COLUMN`, `MODIFY COLUMN <type>`, ou `CHANGE COLUMN <name> <type>` **sans** UPDATE de `kesh_version_min_required`, c'est un finding **CRITICAL** à remonter en passe `bmad-code-review`. Note dialecte : MariaDB utilise `MODIFY COLUMN <type>` ou `CHANGE COLUMN <old> <new> <type>` pour les changements de type (la syntaxe PostgreSQL `ALTER COLUMN <name> TYPE <type>` n'est **pas** supportée en MariaDB — référence locale `crates/kesh-db/migrations/20260419000002_users_company_id.sql:23` utilise bien `MODIFY COLUMN`). Le rationale : ces opérations sont celles dont l'omission du bump min_required exposerait silencieusement les utilisateurs à un downgrade corrupteur. Inversement, `ADD COLUMN nullable` / `ADD INDEX` / `CREATE TABLE` n'imposent pas de bump.

**(P4) Exception documentée** : Si une migration utilise une de ces opérations mais reste **techniquement compatible** avec un binaire antérieur (rare — typiquement `DROP` d'une colonne jamais lue), l'auteur de la PR doit ajouter un commentaire SQL `-- breaking-skip-bump: <justification>` dans la migration, et un Pass code-review devra confirmer la justification. Sinon par défaut → bump obligatoire.

**(P5) Garde-fou audit idempotence** : Toute PR introduisant un nouveau fichier `crates/kesh-db/migrations/*.sql` DOIT ajouter une ligne correspondante au tableau `docs/migrations-idempotence-audit.md` avec verdict (`yes` / `no` / `tracked-by-sqlx`) + justification. Si une PR ajoute un `.sql` migration sans modifier `docs/migrations-idempotence-audit.md`, c'est un finding **MEDIUM** à remonter en passe `bmad-code-review`. Rationale : éviter que l'audit doc dérive silencieusement au fil des Epics suivants — symétrique de la discipline P3.

## Règle de commit et push

**Règle de branchement avant commit** :

Avant le **premier** `Edit`/`Write` d'une nouvelle story, feature, fix ou tâche de maintenance — **toujours brancher d'abord, commiter ensuite** :

```sh
git checkout main && git pull --ff-only
git checkout -b story/X-Y-slug    # ou chore/..., fix/..., docs/...
# puis Edit/Write/commit
```

Pourquoi cette règle existe : sans elle, les premiers commits d'une story atterrissent sur `main` local, puis on `git checkout -b` après coup ce qui emporte les commits sur la nouvelle branche, mais laisse `main` local décalé. Après le squash-merge de la PR, `main` local pointe encore sur l'ancien commit et diverge de `origin/main`. Symptôme : `git pull` qui refuse de fast-forward, état incohérent, risque de `git reset --hard` mal ciblé.

**Garde-fou outillé** : un hook `pre-commit` versionné dans `scripts/hooks/pre-commit` refuse les commits directs sur `main`/`master`. Installation (à exécuter une fois par clone du repo, et après tout update du hook) :

```sh
bash scripts/install-hooks.sh
```

Le script copie les hooks dans `.git/hooks/` (chemin par défaut, indépendant de la branche checkout — un hook activé via `core.hooksPath scripts/hooks` ne protégerait pas les branches qui ne contiennent pas encore le fichier hook, ex. `main` pré-merge).

Bypass exceptionnel : `git commit --no-verify`, à justifier dans le message de commit.

Branches typiques :
- `story/X-Y-slug` — implémentation d'une story BMAD (ex. `story/7-5-kf-008-playwright-selector-fixes`).
- `chore/...` — maintenance, sprint-status, doc post-merge, infra.
- `fix/...` — bugfix hors story (rare — préférer story dédiée si scope > trivial).
- `docs/...` — mise à jour pure de documentation hors flux story.

**Commit systématique après chaque étape BMAD** :

On commit localement après chaque étape structurante du workflow BMAD, sans attendre :

- **Après `bmad-create-story`** (ou `bmad-quick-spec`) — la spec est un artefact versionné, pas un brouillon.
- **Après chaque passe de `bmad-create-story validate`** — chaque passe produit un Change Log entry + patches éventuels, qu'il faut tracer séparément.
- **Après `bmad-dev-story`** (ou `bmad-quick-dev`) — l'implémentation est prête à être revue.
- **Après chaque passe de `bmad-code-review`** — idem validate, chaque passe a ses findings + patches.

Un commit par étape, pas un commit géant en fin de story. Ça permet de revenir en arrière proprement et de voir le fil du processus dans `git log`.

**Push à la demande ou en fin d'epic** :

On **ne push pas automatiquement** après chaque commit. Deux déclencheurs :

1. **Sur demande explicite** de Guy (ex. « pousse », « fais une PR », « ouvre la PR »).
2. **À la fin d'un epic**, après la rétrospective (`bmad-retrospective`). Le push de fin d'epic est le moment où l'on regroupe plusieurs stories dans un PR (ou plusieurs PRs) et où l'on matérialise la clôture de l'epic.

**Exception** : si une règle de workflow BMAD spécifique impose un push (ex. CI check obligatoire avant passe suivante), on push. À justifier dans le message de commit ou dans la conversation.

**Synchroniser le planning du README à chaque commit** :

Avant de créer un commit, vérifier que la **section « Feuille de route »** de `README.md` reflète encore l'état du projet. Le but est d'éviter qu'elle dérive au fil des epics et donne une fausse image (epic marqué "En cours" alors que toutes ses stories sont done, feature listée *(à venir)* alors qu'elle est livrée, etc.).

Vérifier en particulier après :

- **Clôture d'un epic** (rétro done) → mettre à jour le statut dans le tableau des versions (✅ Done / 🚧 En cours / 📋 Backlog).
- **Story qui livre une feature listée dans la section « Fonctionnalités »** → retirer le marqueur *(à venir)*.
- **Renumérotation d'epics** (cf. décision rétro Epic 5 qui a renuméroté 6→13 en 7→14) → propager le nouveau découpage.
- **Changement de scope d'une version** (feature déplacée v0.1 → v0.2 ou inverse, validé via CR) → refléter dans le tableau.

Si une mise à jour est nécessaire, l'inclure **dans le même commit** que le changement qui l'a déclenchée (typiquement le merge de la dernière story de l'epic, ou la rétro). Pas de commit séparé « sync README » a posteriori — sinon le `git log` ne raconte plus l'histoire.

Si le commit ne change rien à la planification (refactor interne, fix de bug, code review patches) le README reste tel quel.

**Synchroniser TOUTES les docs avant tout push / création de release** :

Au-delà du planning README à chaque commit (ci-dessus), élargir la vérification à **tous les supports de documentation** au moment d'**actions visibles à l'extérieur du repo** :

- **À tout `git push` qui ouvre/met à jour une PR** → vérifier que les docs touchées par la branche sont cohérentes.
- **À toute création de release** (tag annoté `vX.Y.Z` + push tag déclenchant `.github/workflows/release.yml`) → audit complet de toutes les docs.
- **À toute clôture d'epic** (post-`bmad-retrospective`) → idem audit complet.

**Supports de doc à auditer** :

| Support | Localisation | Trigger CI/CD | Quand vérifier |
|---------|--------------|---------------|----------------|
| `README.md` (racine) | `/README.md` | rendu auto par GitHub sur la page repo | À chaque commit (cf. paragraphe précédent) + push/release |
| Site GitHub Pages | `website/` (HTML statique) | `.github/workflows/deploy-pages.yml` déclenché par push sur `main` touchant `website/**` | Push/release (rebuild automatique) |
| Manuel admin LaTeX FR | `docs/manual/fr/admin-manual.tex` (+ `.pdf` régénéré) | release (PDF attaché ?) | Release + clôture d'epic majeure (changement install/config/sécurité) |
| Manuel user LaTeX FR | `docs/manual/fr/user-manual.tex` (+ `.pdf`) | idem | Release + stories qui ajoutent/modifient des features visibles utilisateur |
| Brochure marketing LaTeX FR | `docs/manual/fr/marketing-brochure.tex` (+ `.pdf`) | idem | Release (changements de positionnement / pricing) |
| Manuels DE/IT/EN | `docs/manual/{de,en,it}/` (vide v0.1, traductions v0.2+) | idem | Release majeure ; v0.1 = noter "à traduire" si gap |
| CHANGELOG | `CHANGELOG.md` racine (à créer Story 10-4) | release (lecture par tooling release) | Release **obligatoire**, push de fin d'epic recommandé |

**Liste de contrôle pré-push / pré-release** :

1. **README.md « Feuille de route »** : tableau des versions à jour (✅/🚧/📋 par epic), section « Fonctionnalités » sans `(à venir)` pour ce qui est livré.
2. **`website/index.html` + `website/roadmap.html`** : claims marketing alignés avec ce qui est *réellement* dans `main` à la release. Pas d'over-promise sur des features non-mergées. Pas d'under-claim sur des features livrées (visibilité Google Search Console).
3. **`website/issues.html`** : si la page liste les KFs ou roadmap connue, synchroniser avec l'état actuel des labels GitHub Issues (`known-failure`, `enhancement`, `v0.2-milestone`).
4. **Manuels LaTeX FR** : si la story touche install/config/sécurité/usage utilisateur, mettre à jour la section correspondante (admin-manual pour DevOps, user-manual pour fiduciaires/comptables). Régénérer le PDF (`latexmk -xelatex` dans `docs/manual/fr/`) et le commiter (la convention projet est de versionner les PDFs cf. PR #102).
4-bis. **Macro version des manuels — gate release OBLIGATOIRE et inconditionnel** : à **toute création de release** (pas seulement si la story touche un manuel), bumper les trois macros de `docs/manual/shared/kesh-style.sty` pour qu'elles correspondent EXACTEMENT à la version publiée : `\keshVersion{X.Y.Z}` (= version Cargo / champ `version` de `/health`), `\keshReleaseDate{AAAA-MM-JJ}` (date de publication Docker Hub) et `\keshTargetRelease{vX.Y}`. **Puis régénérer les 3 PDF** (`make fr` dans `docs/manual/`) et les commiter — la page de titre et l'en-tête de chaque manuel affichent `\keshVersion`, donc un macro périmé ment sur toutes les pages. *Précédent (2026-06-27, doc-sync v0.3) : la macro était restée bloquée à `0.1.0-dev`/`v0.1` à travers v0.2 ET v0.3 — l'étape « manuels » 4 était conditionnée « si la story touche un manuel », ce qui laissait passer les releases purement code. D'où ce gate inconditionnel.* **Garde-fou** : si un tag `vX.Y.Z` est créé alors que `\keshVersion` ≠ `X.Y.Z`, c'est un défaut de release à corriger avant le push du tag.
5. **CHANGELOG** (une fois créé Story 10-4) : entrée pour la release courante avec sections `Added` / `Changed` / `Fixed` / `Security` / `Removed` (style [Keep a Changelog](https://keepachangelog.com/)).
6. **`docs/known-failures.md` + `docs/change_request.md`** : déjà archivés depuis 2026-04-16/18 — **pas** d'update ici, juste vérifier qu'aucune nouvelle entrée n'a été ajoutée par erreur (les KF/CR vont sur GitHub Issues désormais, cf. §"Issue Tracking Rule").

**Règle d'inclusion** : tout update de doc déclenché par la story DOIT être **dans le même commit** ou la même PR que le code qui le motive (pas de PR doc séparée a posteriori, sauf cas où plusieurs stories y contribuent et que la doc serait rejouée — alors batched dans la PR de la dernière story de l'epic, cohérent `feedback_avoid_parallel_prs`).

**Si la doc n'est pas touchée** (refactor interne, fix de bug sans changement comportemental visible, code review patches cosmétiques) : tous les supports restent tels quels — pas de "doc-only commit" cosmétique pour la forme.

**Coût d'oubli** : un manuel admin obsolète après une story qui change le format `.env` ou les ports réseau induit du support utilisateur cassé. Une feuille de route README qui ment fait perdre la confiance d'un éval/contributeur. Un CHANGELOG manquant à la release rend impossible le diff "qu'est-ce qui change pour mes utilisateurs". Les 3 minutes de vérification valent les heures de support en aval.
