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

**La suite complète tourne EN LOCAL, avant tout `git push`** — au même titre que les gates backend et frontend. Elle n'est pas exécutée pendant l'itération : la doctrine reste le gate ciblé entre les passes, le gate complet au push.

```sh
cd frontend
npm run test:e2e
```

Pré-requis : MariaDB démarré + seed CI appliqué + Playwright browsers installés (cf. `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` sur Ubuntu 26.04+ — limitation upstream Playwright ≤ 1.49).

#### ⚠️ `KESH_COOKIE_SECURE=false` est OBLIGATOIRE en local HTTP — sans lui, TOUTE la suite échoue en 401

**Le montage qui marche**, vérifié le 2026-08-04 :

```sh
# terminal 1 — backend, servant aussi le frontend buildé
KESH_TEST_MODE=true KESH_HOST=127.0.0.1 KESH_COOKIE_SECURE=false \
  DATABASE_URL='mysql://kesh:kesh_dev@127.0.0.1:3306/kesh_e2e' \
  KESH_JWT_SECRET='<32+ octets>' KESH_ADMIN_PASSWORD='<12+ caractères>' \
  KESH_PORT=3000 KESH_STATIC_DIR=frontend/build \
  cargo run -p kesh-api

# terminal 2
cd frontend
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 \
  KESH_BACKEND_URL=http://127.0.0.1:3000 npm run test:e2e
```

**Pourquoi**, à la ligne près — pour que personne ne re-diagnostique ce symptôme :

```js
// node_modules/playwright-core/lib/server/cookieStore.js:36
matches(url) {
  if (this._raw.secure && (url.protocol !== "https:" && !isLocalHostname(url.hostname)))
    return false;                       // ← le cookie n'est PAS joint
// node_modules/playwright-core/lib/server/network.js:62
function isLocalHostname(hostname) {
  return hostname === "localhost" || hostname.endsWith(".localhost");
}
```

**`127.0.0.1` n'est PAS un « local hostname » pour Playwright.** Un cookie `Secure` sur `http://127.0.0.1` est donc rejeté par le `APIRequestContext` que crée `authedApiContext()` — alors que Chromium, lui, le tolère sur loopback. Résultat : le navigateur est authentifié, le contexte API ne l'est jamais, et **toute la suite** échoue en `401` avec `unauth: missing or malformed authorization` côté backend.

⚠️ **`SameSite` n'y est pour rien** — `matches()` ne consulte que `secure`, `domain` et `path`. L'attribut `sameSite` est parsé mais jamais relu au filtrage. *(Diagnostic erroné produit puis réfuté le 2026-08-04, cf. plus bas.)*

**La règle de méthode qui a manqué, et qui coûte cher :**

1. **Chercher la doc du dépôt AVANT de diagnostiquer.** `docs/testing.md` § « Prérequis Playwright local » documentait ce piège **depuis juillet**, et `playwright.config.ts:20` y renvoie. Un montage a été reconstitué par tâtonnement à côté d'une recette existante et correcte.
2. **Ne JAMAIS déclarer une hypothèse « écartée » sans l'avoir exécutée.** L'hypothèse « cookie `Secure` sur du HTTP » a été notée écartée alors que la suite n'avait **jamais** été relancée avec `KESH_COOKIE_SECURE=false` — seule la forme de l'en-tête avait été vérifiée au `curl`. Une issue, une story et un amendement de ce fichier ont été construits sur cette conclusion fausse ; c'est une revue adversariale, lisant le code source de Playwright, qui l'a réfutée.
3. C'est la même faute que « ne déclarer que ce qui a tourné », appliquée au **diagnostic** au lieu du gate : *une hypothèse éliminée par raisonnement n'est pas une hypothèse testée.*

**La CI n'exécute PAS la suite complète** (coût récurrent : MariaDB, seed, navigateurs, durée) — **mais elle exécute un SMOKE** : une seule spec, login puis un appel API authentifié, 2-3 minutes.

⚠️ **Le smoke ne teste aucune fonctionnalité, et c'est délibéré : il vérifie que le HARNAIS EST VIVANT.** Les fonctionnalités restent couvertes par le gate local avant push.

**Pourquoi les deux niveaux, et pas seulement le local.** Une règle qui repose sur la seule discipline de qui pousse a la même faiblesse que celle qui impose de « ne déclarer que ce qui a tourné » : on peut l'affirmer sans l'avoir fait. Et une story qui ne touche pas le frontend fait **légitimement** sauter les E2E — si le harnais casse à ce moment-là, personne ne le saura.

**Précédent qui a motivé la règle (issue #285, 2026-08-04)** : `authedApiContext()` rendait un contexte **non authentifié** — le login pose des cookies `SameSite=Strict` qu'un contexte API, sans site initiateur, ne joint pas. **Toute la suite** était inopérante, et la panne n'a été découverte qu'à l'occasion d'une story sans rapport, parce que trois circonstances se cumulaient : la CI ne lançait pas les E2E, les tests manuels de Guy passent par un navigateur, et **rien ne signale une suite qui cesse de tourner**. Un test jamais lancé ne rougit pas — **il se tait**. C'est le mode d'échec du test muet, déjà payé sur `backfill_skips_archived_accounts` (16-1a) et reproduit sur les mutations de 16-2a, où quatre tests supprimés par mégarde ont rendu un impeccable « 16 passed, 0 failed ».

⚠️ **Un E2E n'est pas un test comme un autre.** C'est le **seul** qui vérifie qu'une valeur traverse réellement la frontière HTTP : Vitest teste la construction du payload, les tests Rust la validation, et **ni l'un ni l'autre ne voit une clé qui disparaît entre les deux**. Le sauter n'a pas le même coût que sauter un test unitaire.

### Pendant une boucle de revue — gate ciblé, gate complet au push

**Le déclencheur du gate complet est le `push`, pas le commit.** Une boucle `bmad-code-review` produit 2 à 4 commits de patches, chacun local et invisible de l'extérieur ; y attacher un gate complet coûte **~1 h par passe** (mesuré : 59 min, 2102 tests) pour des patches qui touchent souvent un seul fichier. Entre les passes, lancer un **gate ciblé** :

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run -E 'binary(<le binaire de test touché>)'   # le rayon d'impact
```

`fmt` et `clippy` restent sur le **workspace entier** — ils coûtent ~1 min et c'est précisément ce qui, sur la Story 16-1d, a fait échouer un gate complet en `exit 101` (`clippy::cmp_owned`) **au bout des 59 minutes**. Les passer en pré-vol est le geste le moins cher du dépôt.

Le gate **complet** reste obligatoire : avant tout `git push`, avant de déclarer une story `done`, et au dernier commit d'une boucle de revue.

**Deux réserves, sans lesquelles la règle se retourne contre nous** :

- **Le story file ne doit affirmer que ce qui a tourné.** Pas de « gate vert, N/N » dans un Dev Agent Record ou un Change Log si seul le binaire ciblé a été exécuté — écrire alors « gate ciblé `binary(x)` vert, gate complet au push ». C'est la contrepartie non négociable : les passes de revue suivantes **lisent ce record et le prennent pour argent comptant**, et une passe adversariale menée sur une base faussement déclarée verte ne mesure plus rien.
- **Exception `kesh-db` : gate complet même en cours de boucle.** Dès qu'un patch touche `crates/kesh-db/migrations/`, `post_restore.rs` ou un repository, le ciblage est interdit. C'est exactement ce que codifient les garde-fous **P6** et **P7** de la § « Migration breaking policy » : ces modes d'échec ne naissent ni du code écrit ni de la spec, mais de l'**interaction** avec des tests que la PR ne touche pas, et **seul le gate réellement exécuté les révèle**. Précédent Story 16-1a : `backfill_skips_archived_accounts` s'est mis à **passer à vide** — un test muet ne signale rien, et aucun gate ciblé ne l'aurait vu.

*(codifié 2026-08-03, arbitrage de Guy pendant la boucle de revue de la Story 16-1d.)*

### Quand sauter

Cette règle ne s'applique pas aux **commits doc-only** (markdown, yaml de planning, README, CLAUDE.md lui-même) qui ne touchent pas de code exécutable. Pour ces commits, la CI elle-même est généralement no-op (pas de Rust ni de TypeScript modifié → cache hit instantané).

## Plafonds mémoire — pourquoi une session de travail meurt en plein gate

**Règle** : sur la station de dev, lancer tout travail lourd (gate backend, build workspace, suite vitest, E2E) via `scripts/mem-guard.sh`, qui l'exécute dans un cgroup à mémoire bornée. `scripts/test-fast.sh` le fait déjà tout seul.

```sh
scripts/mem-guard.sh --protect-shell        # une fois par nouvelle fenêtre/onglet
scripts/mem-guard.sh cargo build --workspace --all-targets
scripts/mem-guard.sh npm --prefix frontend run test:unit
scripts/test-fast.sh                        # déjà sous plafond, rien à faire
scripts/mem-guard.sh --status               # état, politique oomd, derniers OOM
```

### Le mode d'échec, qui n'est pas celui qu'on croit

Ce n'est pas « le build consomme trop ». C'est **qui meurt**.

`systemd-oomd` surveille la tranche `user@1000.service` et, dès que sa *pression mémoire* dépasse 50 % pendant 20 s, il tue **le cgroup descendant qui recycle le plus de pages**. Or un onglet de terminal est **un seul cgroup**, qui contient l'agent de travail **et** tous ses enfants : `cargo`, `rustc`, `mold`, `node`, `chromium`. Le gate et la session qui l'a lancé sont donc dans la même boîte : quand oomd frappe, il emporte les deux, et tout le contexte de travail avec.

Trois conséquences qui se déduisent mal :

- **La victime n'est pas forcément la fautive.** Plusieurs fenêtres travaillant en parallèle sur des projets différents additionnent leur pression sur la même tranche utilisateur. Le 2026-08-11, deux OOM en six minutes (12:57, scope à 18,3 Go ; 13:03, scope à 9 Go) : la cause réelle était un `lualatex` emballé — 7 Go puis 16,7 Go en cent secondes — lancé depuis une **autre** fenêtre, sur un **autre** projet. Avant d'incriminer le gate en cours, lire `scripts/mem-guard.sh --status` et `ps -eo pid,rss,args --sort=-rss | head`.
- **Baisser `jobs` ne borne rien.** C'est une réduction de probabilité, pas une garantie : 4 `rustc` peuvent tenir 12 Go à eux seuls, et un seul processus emballé suffit. Seul un plafond de cgroup est une borne.
- **Un plafond ne sert pas à faire échouer le build, mais à choisir le mort.** Sous `mem-guard`, le pic tue le gate (code 137) et laisse vivre le terminal. Un gate qu'on relance coûte des minutes ; une session perdue coûte tout son contexte.

### `MemoryHigh` est un piège — mesuré, pas supposé

L'étranglement doux (`MemoryHigh`) semble le réglage souhaitable : recycler dans le scope plutôt que tuer. **Il est nuisible ici.** Mesures du 2026-08-11, allocation de 4 Go sous plafond de 512 Mo :

| Réglage | Résultat |
|---|---|
| `MemoryHigh=384M` `MemoryMax=512M` `swap=0` | aucune progression en 120 s — étranglé, jamais tué |
| `MemoryHigh=448M` `MemoryMax=512M` `swap=256M` | idem (timeout) |
| `MemoryHigh=infinity` `MemoryMax=512M` `swap=0` | **137 en 2 s**, net |
| `MemoryHigh=infinity` `MemoryMax=512M` `swap=256M` | **137 à ~768 Mo**, net |

Le noyau pénalise chaque allocation d'un sommeil proportionnel au retard du recyclage : un build qui franchit le seuil ne tombe pas, il **rampe**, en gardant sa mémoire et en continuant d'alimenter la pression qu'oomd mesure. On obtient un gate qui paraît figé **et** une station toujours sous tension. `mem-guard.sh` laisse donc `MemoryHigh` à `infinity` par défaut ; ne pas le réactiver sans refaire la mesure.

### Réglages de parallélisme, et le piège de précédence

| Site | Valeur | Ce qu'elle borne |
|---|---|---|
| `.cargo/config.toml` → `[build] jobs` | 4 | processus `rustc` simultanés (~2 Go pièce sur les gros crates) |
| `.cargo/config.toml` → `--thread-count=4` (mold) | 4 | threads internes de l'éditeur de liens |
| `Cargo.toml` → `[profile.dev] debug` | `line-tables-only` | volume de DWARF construit puis recopié dans chaque binaire de test |
| `frontend/vite.config.ts` → `maxWorkers` | 4 | processus Node+jsdom de vitest (défaut : **31** ici) |
| `.config/nextest.toml` → `test-threads` | 6 | binaires de test simultanés — plafond fixé par la contention MariaDB, pas par la RAM ; ne pas le changer pour des raisons de mémoire |

⚠️ **Le `.cargo/config.toml` du projet ÉCRASE celui de la station** (`~/.cargo/config.toml`). Cargo ne fusionne pas les valeurs scalaires : le fichier le plus proche du projet gagne. Le `jobs = 4` global posé sur la station après le diagnostic du 2026-07-23 était donc **sans effet dans ce dépôt** — seul de tous les projets Rust de la machine, il compilait à 8. C'est un mode d'échec silencieux : le réglage existe, il est correct, et il ne s'applique pas. Toute modification du `jobs` de la station doit être répercutée ici.

⚠️ **`[profile.dev] debug = "line-tables-only"`** conserve fichier et ligne dans les backtraces de panic et d'échec de test — l'usage réel. Ce qu'on perd, c'est l'inspection des variables sous gdb/lldb. Pour une session de débogage pas-à-pas, surcharger ponctuellement sans toucher au fichier : `CARGO_PROFILE_DEV_DEBUG=full cargo test -p kesh-core <test>`.

### Ce qui a été mesuré, et ce qui ne l'a pas été

Sur la station (32 cœurs, 30 Go), après application des réglages ci-dessus :

| Gate | Résultat |
|---|---|
| `cargo build --workspace --all-targets` **à froid** sous plafond 8 Go | vert — 414 crates, 129 binaires de test, 0 erreur |
| `cargo fmt --check` + `cargo clippy --workspace --all-targets -D warnings` | vert, 0 warning — **pic de mémoire anonyme : 1,9 Go** |
| `npm run test:unit` sous plafond 4 Go | vert — 63 fichiers, 512 tests, 23 s |

⚠️ **Ne pas lire `memory.peak` comme une consommation.** Le relevé brut du cgroup de build affichait 8 Go, soit exactement le plafond — mais `memory.peak` compte **le cache de pages**, et ce build écrit 12 Go dans `target/`. Le cache est recyclable : le noyau l'a simplement rendu, sans jamais tuer. La grandeur qui décrit le besoin réel est `memory.stat/anon`, d'où les 1,9 Go ci-dessus. Confondre les deux fait conclure à une saturation là où il n'y en a pas.

**Non mesuré** : `cargo nextest run` complet et la suite Playwright n'ont pas été rejoués sous plafond (ils exigent MariaDB et le seed). Les plafonds les couvrent par construction, mais aucun chiffre n'est déclaré pour eux.

### En CI, rien de tout ceci ne s'applique

`mem-guard.sh` détecte l'absence de systemd utilisateur et exécute la commande telle quelle, avec un avertissement — il n'échoue jamais pour la seule raison qu'il ne peut pas poser de plafond. Les plafonds de parallélisme (`jobs = 4`, `maxWorkers: 4`), eux, sont versionnés et donc actifs en CI, où ils sont de toute façon adaptés aux runners `ubuntu-latest` à 4 cœurs.

*(codifié 2026-08-11, après deux OOM ayant emporté des sessions Claude Code en cours de story 16-3b.)*

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

### Propagation post-patch — grep du symptôme avant la passe suivante

**Règle** : après avoir appliqué un patch de remédiation (revue de spec ou de code), et **avant** de relancer la passe suivante, `grep` le **symptôme corrigé** — pas seulement le site corrigé — sur **tout le dépôt** : code, spec/story file, doc-comments, tests, i18n (les 4 locales), fallbacks Svelte, manuels LaTeX. Lister les sites atteints et les traiter dans le **même** patch.

Concrètement, la question n'est pas *« ai-je corrigé la ligne signalée ? »* mais *« où ailleurs cette même formulation / ce même calcul / cette même priorité est-il écrit ? »*.

Pourquoi : c'est le mode d'échec le plus récurrent du processus. Il est signalé à chaque rétrospective depuis l'Epic 21 et **la codification « un patch vient AVEC son test » n'a pas suffi** — un test couvre le site corrigé, pas les copies du symptôme ailleurs. Récidives mesurées sur l'Epic 14 (5 findings de passe `N+1` qui sont tous des résidus d'un patch de passe `N`) :

| Occurrence | Résidu laissé par le patch |
|---|---|
| 14-2 validate P3→P4 | le patch corrige le fallback svelte `+page.svelte:405`, oublie `:428` → 2 MED en P4 |
| 14-4 validate P3→P4 | le fix `count_by_company` laisse 3 reformulations contradictoires dans la spec → 1 MED en P4 |
| 14-4 review P3→P4 | l'amendement de priorité n'est propagé qu'au `GET /status`, ni au `POST` ni au repo → 1 MED (convergé BlindHunter + EdgeCaseHunter) en P4 |

Les 5 occurrences auraient été attrapées par ce seul geste. Un patch dont le symptôme n'a **pas** été grepé sur le dépôt n'est pas terminé.

*(codifié 2026-07-26, rétrospective Epic 14 action A8 — complète, ne remplace pas, `feedback_review_patch_needs_test`)*

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
- **Non-convergence réelle** : une passe `N+1` de `bmad-create-story validate` remonte une sévérité **égale ou supérieure** à la passe `N` (e.g. `MED → MED`, ou pire `MED → HIGH`). C'est le signal que la remédiation n'entame pas le problème.

Pourquoi : dans les deux cas, la story est trop large pour être tenue dans un seul mental-model adversarial fiable. Les régressions introduites par les patches d'une passe N deviennent invisibles aux passes N+1 (saturation contextuelle) et finissent par être détectées seulement post-merge en code review ou pire, en prod.

**Amendement 2026-07-26 (rétro Epic 14, décision D-C)** — le second critère était auparavant un **compteur de passes** (« validate boucle au-delà de 4 passes »). Il a été remplacé par un critère de **sévérité** parce qu'il produisait des faux positifs : les stories **14-2** (5 passes) et **14-4** (5 passes) ont franchi le seuil de 4 sans être splittées, et les deux ont **convergé proprement** (14-2 : `1 CRIT/2 HIGH/5 MED → 0 → 1 MED → 2 MED → 0` ; 14-4 : `1 CRIT/2 HIGH/6 MED → 0 → 3 MED → 1 MED → 0`). Une convergence **lente mais monotone** (`CRIT → MED → MED → 0`) est le signe d'une revue qui travaille, pas d'une story trop large — les passes tardives y trouvent des défauts *réels et décroissants* (c'est Opus en P3 de 14-4 qui a attrapé le doublon d'écriture d'ouverture). Ce qui doit déclencher le split, c'est la **stagnation ou la régression de sévérité**, pas la durée. Le plafond de 8 passes de la §"Review Iteration Rule" reste le garde-fou de budget.

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

## Clôture d'epic — la revue de projet suit la rétrospective

**Règle** : à la fin de chaque `bmad-retrospective`, enchaîner sur la **revue de projet** dans `/home/gcorbaz/travail/Projets actuels/kesh`. La rétrospective ne clôt pas l'epic à elle seule.

Les deux exercices ne regardent pas la même chose, et c'est pourquoi l'un ne remplace pas l'autre :

- La **rétrospective** regarde *vers l'intérieur du dépôt* — ce que l'epic a produit, ce que les passes de revue ont appris, quelle dette il laisse. Son horizon est le code et le processus.
- La **revue de projet** regarde *le projet comme dossier* — échéances, dépendances avec d'autres dossiers, ressources, engagements pris ailleurs que dans le dépôt. Rien de tout cela n'est visible depuis `git log`, et rien ne le rappelle si on ne va pas le chercher.

Un epic peut donc se clore proprement côté code tout en laissant dériver le dossier de projet : c'est exactement le trou que cette règle ferme.

**Ordre imposé** : `bmad-retrospective` → revue de projet. La rétrospective d'abord, parce que ses conclusions (dette reportée, story de remédiation planifiée, changement de scope) sont des **entrées** de la revue de projet — l'inverse ferait travailler la revue sur un état périmé.

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

**Vérifier la ligne ET les compteurs, en recomptant la source.** Le **total** de migrations apparaît à **deux** endroits du fichier — l'**en-tête de section** `## Table d'audit (N migrations)` et la ligne `Total` des « Statistiques ». S'y ajoutent **trois compteurs de partition** (`yes`, `tracked-by-sqlx`, `no`) dont la **somme** doit égaler ce total. Tous se **recomptent depuis le tableau**, jamais ne s'incrémentent de confiance. Recompte de contrôle :

```sh
ls crates/kesh-db/migrations/*.sql | wc -l                 # doit égaler les DEUX sites du total…
grep -c '^| `20' docs/migrations-idempotence-audit.md      # …et le nombre de lignes du tableau
```

⚠️ Les compteurs de partition ne valent **pas** le total — les aligner dessus casserait l'invariant qu'ils servent à tenir. *(Cette précision a dû être écrite deux fois : la passe 1 de revue de la 16-1a-bis, en corrigeant un compteur faux, avait énoncé une règle fausse — « les trois sites doivent donner le même nombre » — que la passe 2 a rattrapée. Corriger un compteur et écrire la règle de son contrôle sont deux gestes distincts.)* Précédent : au moment de la Story 16-1a, le compteur `tracked-by-sqlx` annonçait 45 pour **52** réelles — dérive de 7 accumulée sur les Epics 20/21, parce que 11 lignes de migration avaient été ajoutées **sous** la section « Maintenance future » au lieu du tableau, et que les statistiques ne comptaient que le premier bloc. La passe 7 de `validate` de la 16-1a déclare pourtant avoir « re-vérifié les compteurs » : **sept passes adversariales ont confirmé un nombre faux**, parce que relire une valeur n'est pas recompter sa source.

**(P6) Garde-fou couplage positionnel des migrations** : Toute PR introduisant un nouveau fichier `crates/kesh-db/migrations/*.sql` DOIT exécuter `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et **inspecter chaque site**. Un test qui indexe les migrations **par position** (`total - N`, `&all[..n]`) change silencieusement de sens à chaque migration ajoutée : la fenêtre qu'il croit appliquer se décale d'un cran.

Chaque site doit satisfaire l'un des deux critères :

- **résolution par version** — l'index est obtenu par `.position(|m| m.version == <version>)`, ce qui rend le montage insensible aux ajouts futurs (**à préférer**) ; OU
- **couplage positionnel assumé** — il matérialise une frontière historique voulue, et porte alors **obligatoirement** un garde-fou fail-loud : `assert_eq!(total, <N>)` codé en dur, ou une assertion de montage vérifiant l'état attendu du schéma avant la migration cible.

Un site positionnel **sans** garde-fou est un finding **MEDIUM** en `bmad-code-review`.

Rationale — ce mode d'échec est **invisible en revue de diff** : il ne naît ni du code écrit ni de la spec, mais de l'**interaction** entre la migration ajoutée et des tests que la PR ne touche pas. Seul le gate réellement exécuté le révèle. Précédent Story 16-1a : 3 tests touchés, dont `upgrade_path_preserves_data` (échec net grâce à son `assert_eq!(total, 55)` volontaire), `backfill_matches_seed_for_every_chart` (échec grâce à son assertion de montage) et surtout **`backfill_skips_archived_accounts`, qui s'est mis à passer À VIDE** — ses rôles ressortaient `NULL` non pas parce que le backfill écartait correctement un compte archivé, mais parce qu'il ne tournait plus du tout sur ces lignes. Un test muet ne détecte plus aucune régression, et rien ne le signale. C'est la raison d'être du critère « assertion de montage obligatoire ».

*(codifié 2026-07-28, passe 1 de `bmad-code-review` de la Story 16-1a)*

**(P7) Garde-fou triage des backfills de données** : Toute PR introduisant une migration qui **écrit des données** DOIT la trier — soit au registre `POST_RESTORE_BACKFILLS`, soit à la liste `EXEMPT_MIGRATIONS` **avec une justification écrite** (`crates/kesh-db/src/post_restore.rs`). Le test `every_data_backfill_migration_is_triaged` l'impose et échoue en nommant le fichier ; un manquement laissé passer est un finding **MEDIUM** en `bmad-code-review`.

Rationale : `_sqlx_migrations` n'est pas restaurée par l'import d'installation. Sans rejeu, restaurer un `.keshbackup` antérieur à une migration de backfill **rouvre définitivement** le bug qu'elle fermait — la migration reste marquée appliquée et ne repassera jamais. Et le contrôle de schéma ne voit rien : `is_required()` est faux dès qu'une colonne porte un `DEFAULT`, pas seulement quand elle est nullable, si bien qu'une colonne `NOT NULL DEFAULT …` est silencieusement **réinitialisée à son défaut**.

**Détecter, c'est chercher LARGE.** Le détecteur couvre `UPDATE`, `INSERT` **toutes formes** (`INTO … SELECT`, `INTO … VALUES`, `IGNORE`, `LOW_PRIORITY`, `HIGH_PRIORITY`, `ON DUPLICATE KEY UPDATE`), `REPLACE` et `DELETE`, sur du SQL **décommenté** et découpé en statements **multi-lignes**. Il classe sur le **premier mot-clé** du statement, et c'est ce qui lui donne cette largeur sans énumération à maintenir. Le coût d'une forme en trop est une exemption d'une ligne ; le coût d'une forme manquante est un garde-fou **muet**. C'est arrivé **deux fois** pendant la seule spécification de la Story 16-1c : d'abord un `grep "INSERT INTO.*SELECT"` **mono-ligne** (le `SELECT` était à la ligne suivante), puis un motif `INSERT\s+INTO` qui ratait les quatre `INSERT **IGNORE** INTO` d'une autre migration. Attention aussi à `ON UPDATE CURRENT_TIMESTAMP`, présent dans une vingtaine de migrations : c'est du **DDL**.

**Trier, ce n'est pas forcément inscrire au registre.** Un backup n'est importable que si son inventaire de tables est **identique** au nôtre (`parse_and_verify` compare dans les deux sens), et les tables ne font que s'ajouter : une migration **antérieure à la dernière création de table applicative** est donc hors d'atteinte, son cas de déclenchement étant refusé en 400 bien avant le rejeu. L'y inscrire produirait du code mort **qui paraît fonctionner** — et, s'il est en classe A, exécuté à chaque import. Le test `registry_entries_are_within_import_window` recalcule cette fenêtre depuis le `MIGRATOR` et échoue si une entrée en sort.

**L'exemption est l'issue la moins coûteuse, donc celle qu'il faut contrôler.** Le garde-fou de triage propose lui-même « soit l'ajouter à `EXEMPT_MIGRATIONS` avec une justification écrite » : rédiger une justification est plus rapide qu'inscrire une entrée au registre, et une justification fausse désactive le rejeu **définitivement et en silence**. D'où le symétrique `exemptions_claiming_out_of_window_really_are_out_of_window`, qui recalcule la fenêtre pour toute justification invoquant l'argument. Il la reconnaît à un **marqueur textuel** : une justification « hors fenêtre » DOIT commencer par la chaîne `Hors fenêtre`, faute de quoi elle échappe au contrôle et redevient une affirmation crue sur parole.

**Classer, ça se déclare, ça ne se devine pas.** Classe **A** (rejeu inconditionnel) uniquement si **tous** les statements sont gardés contre l'écrasement d'une valeur posée par l'utilisateur ; classe **B** (rejeu conditionné à l'absence d'une colonne sentinelle) sinon — et la classe B n'est valide que si le **DDL et le backfill sont dans le même fichier**. Ne **jamais** classer par détection textuelle de `IS NULL` / `NOT EXISTS` : un `NOT EXISTS` peut être un prédicat **structurel** de ciblage et non une garde d'idempotence. Et ne pas réutiliser le critère « un `NULL` n'est l'expression d'aucun choix » : il est **faux** dès qu'une route écrit le champ en *full-replace*.

*(codifié 2026-08-01, Story 16-1c / issue #281)*

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
