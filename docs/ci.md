# Pipeline CI/CD Kesh

Documentation de la pipeline GitHub Actions définie dans `.github/workflows/ci.yml` et `.github/workflows/release.yml`.

## Vue d'ensemble

La pipeline `ci.yml` se déclenche sur :

- **Pull request** vers `main` → le job `changes`, puis 4 jobs (`backend`, `frontend`, `docker-build`, `e2e-smoke`), qui doivent être sautés si la PR ne porte que `_bmad-output/` (cas nominal encore à relever ; l'échec, l'annulation et le délai dépassé du filtre l'ont été, voir ci-dessous)
- **Push** sur `main` (= merge de PR) → `changes`, puis les mêmes 4 jobs, toujours
- **Pull request** vers une branche `epic/**` (la branche d'un epic, depuis le 2026-11-15) → la même chose que vers `main`. Les branches d'epic ne sont pas protégées : la CI y informe, elle ne bloque pas la fusion d'une story dans sa tranche.

**Le cache Rust** (`Swatinem/rust-cache`, job `backend`) porte la clé partagée `kesh-backend` et **n'est écrit que par les exécutions sur `main`** (`save-if`) : une PR lit le cache de sa base, puis celui de `main`, sans en écrire. Motif : GitHub plafonne le cache d'un dépôt à 10 Go (8,2 Go occupés le 2026-11-15, dont la moitié en caches de PR) et évince les plus anciens ; des branches d'epic écrivant chacune le leur feraient tout recompiler. Une PR qui change `Cargo.lock` ne trouve donc pas de cache à sa mesure et compile davantage. Le **cache Docker** (`type=gha`) n'est pas restreint : `e2e-smoke` reconstruit l'image de la PR depuis ce que `docker-build` vient d'y écrire (`CLAUDE.md` § *Travailler à plusieurs développeurs*).

**Le filtre, et son périmètre étroit** *(Story 33-11, #928)* : le job `changes` lance `scripts/ci-seuls-artefacts.sh`, qui rend `code=false` **seulement** sur une `pull_request` dont le diff, non vide, ne nomme que des fichiers sous `_bmad-output/`. Les quatre jobs **doivent** alors être sautés, et un job sauté **doit** compter pour réussi aux checks requis — ce que la documentation de GitHub affirme et que les relevés de l'AC-6 de la story n'ont pas encore confirmé (`_bmad-output/implementation-artifacts/33-11-filtre-ci-preuves-ac6.md`) ; c'est pourquoi le filtre n'est pas un `paths-ignore` au niveau du workflow, qui les laisserait en attente. Tout le reste échoue vers la CI complète : `push`, diff vide, SHA inconnu, `git` en échec, renommage depuis ou vers un autre dossier, et `changes` lui-même en échec : les conditions excluent le seul cas `changes` réussi et sortie `false`. **Relevé le 2026-11-07 sur la PR #965** (commits temporaires défaits par `git revert` avant la fusion) : `changes` en **échec** (run 42402136024) — les quatre jobs tournent jusqu'au vert, aucun sauté ; `changes` **annulé** (run 42406580735) — les quatre `cancelled`, aucun sauté, PR `BLOCKED` ; `changes` au **délai dépassé** (run 42406684036) — `changes` `cancelled`, les quatre tournent jusqu'au vert, aucun sauté. Le filtre échoue donc fermé dans les trois cas. **Manuels, `docs/`, `README.md`, `CHANGELOG.md`, `CLAUDE.md` et `website/` restent hors du filtre**, parce que des tests les lisent (`textes_coherents.rs`, `mcp_matrice_manuel.rs`, `configuration_transmise.rs`). `changes` n'est pas un check requis. ⚠️ **Élargir le périmètre demande de vérifier d'abord que rien ne lit les fichiers visés** : l'erreur de ce filtre est muette, elle saute des tests à tort. **Reste à relever** : le cas nominal — une PR qui ne porte que `_bmad-output/` saute les quatre jobs **et** reste fusionnable (`CLEAN`, quatre contextes `SKIPPED`) ; il ne peut l'être qu'une fois `ci.yml` hors du diff, sur la PR de la rétrospective de l'Epic 33 (relevé (c) de l'AC-6), et ce paragraphe se rétracte s'il le contredit.

*(Jusqu'à la Story 33-11, 2026-11-07, ce document disait : « Aucun filtre `paths:` : les quatre jobs tournent à chaque push, même sur un commit de documentation seule. » Il n'y a toujours aucun filtre `paths:`.)*

*(Jusqu'à la Story 31-2, ce document décrivait un job `e2e` complet qui n'avait jamais existé dans `ci.yml` — issue #577. Il décrit désormais le dépôt ; les décisions D-6-1-* relatives à ce job sont annotées « non retenu » en fin de document.)*

Aucune image Docker n'est publiée sur push `main` — la publication n'a lieu **que** sur push de tag `v*.*.*` (via `release.yml`). Le job `docker-build` du `ci.yml` valide les deux compose distribués (`docker compose config -q` sur `docker-compose.yml` et `docker-compose.prod.yml` — clé inconnue ou en double, interpolation mal formée) — `docker-compose.yml` dans les **deux sens** : il doit se lire avec des mots de passe MariaDB factices, et **refuser** sans eux en nommant `MARIADB_ROOT_PASSWORD` ou `MARIADB_PASSWORD` (Story 15-13a, #551) — puis build l'image en mode sanity (sans push) pour valider que le `Dockerfile` compile.

La pipeline `release.yml` se déclenche **uniquement** sur push de tag `v*.*.*` et publie une image Docker Hub SemVer + une GitHub Release.

### Schéma d'exécution

```
                    [changes]                        ← filtre des artefacts, non requis
                   /    |    \
[backend]         [frontend]         [docker-build]   ← sanity, no push
                                           |
                                      [e2e-smoke]     ← image de la PR, une spec Playwright
```

| Job (nom affiché) | Trigger | Timeout | Dépendances | Vérification requise de `main` |
|---|---|---|---|---|
| `changes` (Filtre des artefacts) | push, PR | 10 min | — | non |
| `backend` (Backend (Rust)) | push, PR | 120 min | `changes` | oui |
| `frontend` (Frontend (Svelte)) | push, PR | 20 min | `changes` | oui |
| `docker-build` (Docker build (sanity)) | push, PR | 20 min | `changes` | oui |
| `e2e-smoke` (E2E smoke (Playwright)) | push, PR | 25 min | `changes`, `docker-build` | oui — depuis le 2026-10-24 (décision de Guy) |
| `docker` (release) | push tag `v*.*.*` | (défaut) | — | — |

### Le smoke E2E — job `e2e-smoke` (Story 31-2, #577)

Il vérifie que **le harnais Playwright est vivant**, pas les fonctionnalités : une seule spec,
`frontend/tests/e2e/smoke.spec.ts`, deux tests — connexion par l'interface puis `GET /api/v1/auth/me` par
`authedApiContext` (200, nom `admin`), et la même route **sans** cookie (401 ; sans cette sonde négative, le 200
pourrait venir d'une route ouverte). C'est le mode d'échec de l'issue #285 : un contexte API sans cookie rendait 401
partout, et rien ne signalait une suite qui cessait de tourner. La **suite complète** reste locale, avant tout push
(`CLAUDE.md` § E2E).

Montage : sur une PR de fork, MariaDB par `docker run` depuis le premier miroir qui sert l'image (ECR, `mirror.gcr.io`, Docker Hub ; le service est désactivé) ; connexion Docker
Hub (sautée sans secret, PR de fork : images de base par le miroir ECR, `moby/buildkit` par `mirror.gcr.io`), `setup-buildx`, reconstruction de l'image de la PR depuis le cache `gha` laissé par
`docker-build` (`load: true`), puis `docker run --network=host` avec `KESH_TEST_MODE=true`, `KESH_HOST=127.0.0.1` (le
mode test refuse un bind non-loopback), `KESH_COOKIE_SECURE=false` (HTTP), `KESH_PORT=3000` et
`DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh` — le service MariaDB du job, utilisateur `kesh` aux droits de
la production. Les migrations s'appliquent au démarrage du backend ; le seed est celui du backend
(`POST /api/v1/_test/seed`, appelé par `seedTestState`). Attente de `/health` en boucle (journal du conteneur affiché
si elle échoue), puis `npm ci`, `npx playwright install --with-deps chromium` et
`npx playwright test tests/e2e/smoke.spec.ts`. En cas d'échec : journal du backend, et `frontend/test-results/`
(traces) téléversé, rétention 3 jours.

Il exerce aussi, **sur chaque PR que le filtre ne saute pas**, le chemin « image + utilisateur `kesh` + migrations au démarrage » que reprend le
smoke `/health` de `release.yml`. **Durée mesurée au premier run** (PR #702, 2026-10-23) : **16 min 29 s** — ≈ 10 min pour
`npx playwright install --with-deps chromium`, ≈ 5 min de construction de l'image depuis le cache `gha`, ≈ 4 s pour la
spec. Au-delà des 10 min visées : le cache des navigateurs (D-6-1-A) est le levier, à mettre en place.

**Repli** si le cache `gha` ne tient pas (reconstruction complète de l'image à chaque run) : un job qui compile
`kesh-api` et télécharge l'artefact `frontend-build` — écarté d'abord (C-31-2-2) parce qu'il ajoute un second
`cargo build` pour tester un binaire de débogage, qui n'est pas ce qu'on publie.

### L'utilisateur MariaDB de la CI : `kesh`, pas `root` (Story 31-2)

Le job `backend` applique `scripts/mariadb-init/01-dev-grants.sql` en `root` (étape « Apply dev grants », après
« Install mariadb-client », `-h 127.0.0.1`), puis emploie `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh` :
`kesh` a exactement les droits du conteneur de dev — `ALL` sur `kesh`.*, `\_sqlx\_test%`.* et `kesh%`.*, aucun droit
global. Migrations et seed restent en `root` : c'est de l'outillage, pas le chemin de l'application. Les jobs
`e2e-smoke` et `release.yml` emploient le `kesh` que crée le service (`ALL` sur `kesh`.*, les droits de la production).

Pourquoi : une CI en `root` fait ce que faisait un conteneur de dev trop large — elle laisse passer un test qui dépend
d'un privilège que la production n'a pas (Stories 29-4 et 29-8, `INNODB_TRX`). Table des droits par environnement :
`docs/testing.md` § « Base de dev jetable ».

⚠️ **Le smoke de `release.yml` ne se vérifie qu'à la publication** : un échec y bloque la release avant le push de
l'image. Le risque est borné par `e2e-smoke`, qui exerce sur chaque PR non filtrée la même image avec le même utilisateur. Premier
smoke de publication en utilisateur `kesh` (#698) : tag `v0.15.0`, 2026-10-24, **vert** (run 40080415340).

## Reproduction locale

Toutes les commandes du CI peuvent être reproduites en local. Pré-requis : MariaDB 10.11 démarré et accessible via `DATABASE_URL` (alignement Story 10-1 D3 — parité prod NAS Synology Package Center DSM).

### Backend (Rust)

```bash
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1
```

**`scripts/gate-ci.sh kesh_ci_<suffixe> [journal]`** rejoue la dernière ligne telle quelle, comme la CI : base recréée sur le MariaDB de dev, migrations appliquées une à une par `mariadb … < fichier`, seed, puis `cargo test` dans un environnement fermé, sans `DATABASE_URL` du shell ni variable de l'appelant, et refus si un `.env` est atteignable, à la racine, dans un parent ou dans un crate. Dernière ligne de sa sortie standard : `gate-ci : N passés, M échoués, K ignorés`, recomptée depuis `test result:` (le journal et le code suivent, sur la sortie d'erreur). Le journal donné doit être un fichier ordinaire (sinon refus, code 2 : un périphérique comme `/dev/full` se relirait sans fin) ; s'il s'écrit mal en cours de route, aucun décompte n'est rendu (code 5). ⚠️ `cargo test` s'arrête au premier binaire en échec, comme en CI : le décompte d'un rouge ne couvre que les binaires déjà passés. Écarts déclarés : le nom de la base, le MariaDB de dev (datadir en tmpfs, durabilité réduite, d'autres bases à côté), ses droits déjà posés, `HOME` réel (la configuration cargo de la station est lue), les variables du runner. **Les tests des deux scripts de la Story 33-10 ajoutent une à deux minutes à la suite** (`recette-montee.test.sh` : 99 à 104 s sous nextest à la station, deux tirages, dont, par construction et non mesurés, 22 à 24 s pour le seul cas du client `docker` sourd ; `gate-ci.test.sh` < 1 s), et demandent `python3`, `ss`, `jq`, `realpath`, `md5sum`, `timeout`, présents sur `ubuntu-latest`. Leurs enveloppes Rust les bornent à 4 Go de mémoire virtuelle et 600 s. **Mesuré le 2026-11-07, Story 33-10, à la station, cible déjà compilée : 18 min 37 s, 206 binaires, 4823 passés, 0 échoué, 16 ignorés** ; sans cible chaude, davantage, et la CI n'a pas de tmpfs. `scripts/test-fast.sh`, lui, lance nextest, un processus par test : avec le correctif `env_lock` de la 0.17.4 défait, il reste vert (4815 passés) quand `gate-ci.sh` rend les **17 échecs** que la CI avait rendus.

**Pourquoi `-j1 --test-threads=1`** : les tests partagent un état global (i18n, locale d'erreurs) et l'isolation cross-binary SQLx exige une exécution séquentielle pour éviter `PoolTimedOut`. Cf. mémo projet `feedback_sqlx_mysql_gotchas`.

### Frontend (Svelte)

```bash
cd frontend
npm ci
npm run check       # svelte-check
npm run test:unit   # Vitest
npm run build       # Vite build
```

### E2E (Playwright)

La CI ne joue que le smoke (`e2e-smoke`) ; la **suite complète** se joue en local avant tout push. Le montage —
backend `cargo run -p kesh-api` qui sert le frontend buildé, `KESH_TEST_MODE=true`, `KESH_COOKIE_SECURE=false`,
`KESH_BACKEND_URL` — est la recette de `docs/testing.md` § « Prérequis Playwright local », qui fait foi.

```bash
cd frontend
npx playwright install --with-deps chromium
npm run test:e2e                                   # suite complète
npx playwright test tests/e2e/smoke.spec.ts        # le seul smoke, comme en CI
```

Locale Playwright forcée à `fr-CH` + TZ `Europe/Zurich` dans `playwright.config.ts` — ne pas modifier (évite la flakiness multi-locale, cf. D4 review pass 1 G2 D Story 5-4).

## Politique accessibilité (axe-core)

Chaque spec Playwright **majeur** doit inclure au moins un test `axe-core` qui assert `results.violations` est vide.

### Specs avec couverture axe (6/12)

| Spec | Page testée | Story d'origine |
|---|---|---|
| `auth.spec.ts` | `/login`, layout principal | 1.11 |
| `users.spec.ts` | `/users` (liste, formulaire) | 1.12 |
| `contacts.spec.ts` | `/contacts` | 6.1 |
| `products.spec.ts` | `/products` | 6.1 |
| `invoices.spec.ts` | `/invoices` (empty state — voir D-6-1-D) | 6.1 |
| `homepage-settings.spec.ts` | `/` (homepage) | 6.1 |

### Règle pour tout nouveau spec majeur

```ts
import AxeBuilder from '@axe-core/playwright';

test('axe-core sans violations sur <page>', async ({ page }) => {
    await page.goto('/<page>');
    await page.waitForLoadState('networkidle');
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
});
```

### Aucune exclusion AA sans KF documentée

`AxeBuilder` est appelé **sans** `.disableRules()`. Si une violation WCAG AA est détectée :

1. **Préféré** : corriger le composant fautif dans la story courante.
2. **Sinon** : créer une KF dans `docs/known-failures.md` + GitHub issue `known-failure` + ajouter `.disableRules(['violated-id'])` avec un commentaire pointant la KF.

Aucune tolérance silencieuse.

## Stratégie de publication Docker

La publication d'images Docker se fait **uniquement** sur push de tag SemVer `v*.*.*` (via `release.yml`). Aucune image n'est publiée sur push `main` — décision tracée dans le CR #17.

| Tag | Trigger | Workflow | Mutabilité |
|---|---|---|---|
| `:{version}` (ex `:0.1.0`, `:0.16.1-rc1`) | push tag `v*.*.*` | `release.yml/docker` | Immuable — release SemVer ou pré-version |
| `:latest` | push tag `v*.*.*` **sans suffixe** | `release.yml/docker` | Mutable — toujours la dernière release SemVer |

Un tag de **pré-version** — suffixe après un tiret, `vX.Y.Z-rcN` — ne pousse que `:X.Y.Z-rcN` : `:latest` ne bouge pas,
et sa GitHub Release est marquée pré-version. Cargo reste à `X.Y.Z` ; le smoke compare la base du tag à `/health`. Les
candidates servent à jouer une recette sur l'image publiée avant de figer les textes du tag final. *(Depuis la
préparation de la 0.16.1, 2026-10-29 : jusque-là, tout tag déplaçait `:latest` et créait une release ordinaire.)*

Le job `docker-build` du `ci.yml` valide d'abord les deux compose distribués (`docker compose config -q` ; pour `docker-compose.yml`, avec des mots de passe MariaDB factices, puis l'échec **exigé** sans eux), puis build l'image en mode sanity (sans push) sur chaque PR/push afin de détecter au plus tôt toute régression du `Dockerfile`. L'image est jetée à la fin du runner. La **liste** des variables transmises par les compose est gardée par le test Rust `configuration_transmise` (job `backend`, Story 15-11a).

### Docker Hub : tirages authentifiés, secrets requis

Depuis la Story 31-2 (action A2 de la rétrospective de l'Epic 15), **tout tirage depuis Docker Hub est authentifié** par
les secrets `DOCKERHUB_USERNAME` et `DOCKERHUB_TOKEN`, configurés au niveau du dépôt GitHub (Settings → Secrets and
variables → Actions ; à renouveler chaque année ou en cas d'exposition). Auparavant, le service `mariadb:10.11` était
tiré **anonymement**, et un `429` de Docker Hub faisait échouer le job `backend` sans rapport avec le diff.

| Où | Ce qui est tiré | Comment c'est authentifié | PR de fork (sans secret) |
|---|---|---|---|
| `ci.yml`, job `backend` | service `mariadb:10.11` | `services.mariadb.credentials` | service désactivé ; `mariadb:10.11` par `docker run`, anonyme, du premier miroir qui la sert (ECR, `mirror.gcr.io`, Docker Hub) |
| `ci.yml`, job `e2e-smoke` | service `mariadb:10.11` ; images de base du `Dockerfile` ; `moby/buildkit` | `credentials` + `docker/login-action` | service désactivé, MariaDB par `docker run` (même chaîne de miroirs) ; images de base par ECR, `moby/buildkit` par `mirror.gcr.io`, anonymes, sans reprise |
| `ci.yml`, job `docker-build` | images de base du `Dockerfile` (`rust`, `node`, `debian`) ; `moby/buildkit` | `docker/login-action` | images de base par ECR, `moby/buildkit` par `mirror.gcr.io`, anonymes, sans reprise |
| `release.yml`, job `docker` | service `mariadb:10.11` ; images de base ; `moby/buildkit` ; puis le push | `credentials` + `docker/login-action` | sans objet : déclenché par un tag du dépôt |

- **La connexion précède `docker/setup-buildx-action`** dans les trois jobs qui construisent : le builder
  `docker-container` tire `moby/buildkit` depuis Docker Hub à cette étape (avec le secret ; sans lui, de
  `mirror.gcr.io`, ci-dessous). ⚠️ Que ce tirage profite de la connexion est
  une **hypothèse** sur le comportement de l'action, à constater au journal du premier run (tirage authentifié ou non) —
  non vérifiée localement.
- **Les PR issues de forks tournent sans secret, par des registres publics anonymes** (miroir ECR, `mirror.gcr.io`, Docker Hub ; C-31-2-3, révisé le 2026-11-24 sur décision
  du mainteneur : le dépôt reçoit désormais des PR de contributeurs, depuis leur fork). Ni `pull_request_target`, ni
  secret exposé : la CI d'un fork reste un `pull_request` ordinaire, en lecture seule. Les règles :
  - **Le service MariaDB** (jobs `backend` et `e2e-smoke`) reste un `services:` tiré de Docker Hub avec ses
    `credentials` sur `main` et les branches du dépôt. Sur une PR de fork, son image vaut `''` : un service d'image vide
    ne démarre pas, et MariaDB démarre à la place par l'étape « Start MariaDB (PR de fork) », qui lance
    `scripts/ci-mariadb-fork.sh` : `docker run` de `mariadb:10.11`, sans authentification, tirée d'abord par une
    **chaîne de miroirs avec reprise** (ci-dessous), avec les mêmes variables, le même port (3306 publié sur l'hôte) et le même contrôle de santé, attendu « healthy »
    avant la suite. Le script est testé sur toute PR par `scripts/tests/ci-mariadb-fork.test.sh` (étape « Tests du
    démarrage MariaDB de fork » du job `backend`), qui compare aussi ses valeurs à celles des deux `services:`. La
    décision se lit dans le contexte `github`
    (`github.event_name == 'pull_request' && github.event.pull_request.head.repo.full_name != github.repository`) :
    `jobs.<id>.services` n'a pas accès à `secrets`. La comparaison des noms couvre aussi un fork supprimé
    (`head.repo` nul), que `head.repo.fork` laisserait passer pour une branche du dépôt ; la garde sur l'événement
    empêche un `push` (sans `pull_request`) d'être pris pour un fork.
    **Pourquoi ne pas garder le service sur un fork** : ses `credentials` y vaudraient `''`, et le modèle du job, évalué
    avant toute étape, peut refuser des chaînes vides pour `username` et `password` (`Unexpected value ''`) — ce que le
    code du runner qui ignore des identifiants vides (`ContainerRegistryLogin`, `actions/runner`) ne rattrape pas, puisqu'il
    s'exécute après. Sur un fork, les `credentials` reçoivent donc une valeur de remplissage (`sans-objet`), jamais
    employée puisque le service ne démarre pas ; sur une PR du dépôt sans secret (Dependabot), ils restent vides, comme
    avant. Aucun job n'est dédoublé ni sauté : les noms des quatre checks requis ne changent pas.
  - **La chaîne de miroirs de MariaDB** (`scripts/ci-mariadb-fork.sh`, depuis le run 45525662619 de la PR #1560, la
    première PR de fork après la #1521) : à ce run, le smoke E2E a échoué à « Start MariaDB (PR de fork) » sur un refus
    d'ECR, `toomanyrequests: Data limit exceeded` (exit 125), alors que le job `backend` avait obtenu l'image quelques
    minutes plus tôt — un quota anonyme, intermittent (vraisemblablement compté par adresse, que les runners partagent ;
    non établi par le run). Le script tire
    donc l'image avant `docker run`, dans cet ordre : `public.ecr.aws/docker/library/mariadb:10.11`,
    `mirror.gcr.io/library/mariadb:10.11`, puis `docker.io/library/mariadb:10.11` en anonyme. Chaque miroir est essayé
    trois fois, à 5 puis 10 secondes d'intervalle (le délai double, borné à 20 s) ; le journal nomme chaque refus et le
    miroir qui a servi ; `docker run --pull=never` démarre l'image obtenue. Si aucun ne la sert, l'étape échoue avec
    « aucun miroir n'a servi l'image ». Au pire, la chaîne ajoute 45 secondes d'attente. Relevé le 2026-11-25, tiré
    sans identifiants : les trois miroirs servaient `mariadb:10.11` sous la **même empreinte d'index**
    (`sha256:e5fe75b2…`) — un relevé daté, non une garantie : l'étiquette est mouvante (comme celle du service), et
    `mirror.gcr.io`, un cache, peut servir un temps un correctif plus ancien que Docker Hub. Le test (`scripts/tests/ci-mariadb-fork.test.sh`) éprouve la reprise sur le même miroir, le
    passage au suivant, l'ordre des trois, le délai borné et l'échec de tous ; une sonde de mutation qui retire la
    reprise le fait rougir.
  - ⚠️ **Limite : les images de base du `Dockerfile` et `moby/buildkit` n'ont pas de reprise.** Sans secret, elles sont
    tirées par BuildKit (`build-push-action`) et par `setup-buildx-action` depuis ECR et `mirror.gcr.io`, au même
    quota anonyme pour ECR : les images de base (`rust` surtout) pèsent davantage que MariaDB, et un refus ferait
    échouer `docker-build` ou `e2e-smoke` de la même façon. Une reprise n'y est ni simple ni sûre : le builder
    `docker-container` a son propre magasin d'images, si bien qu'un `docker pull` préalable ne lui sert à rien ;
    reprendre le build sur un autre `REGISTRY` doublerait la durée d'un build qui échoue pour une autre raison (le job
    `docker-build` a 20 minutes) et changerait la clé du cache `gha` qu'`e2e-smoke` relit ; les miroirs de registre de
    `buildkitd` ne se règlent que pour `docker.io`, et leur repli sur un refus n'est pas éprouvé ici. Sur un tel refus,
    le remède est de relancer le job.
  - **Les connexions `docker/login-action`** (jobs `docker-build` et `e2e-smoke`) ne tournent que si le secret est
    présent : le job pose `HAS_DH: ${{ secrets.DOCKERHUB_TOKEN != '' }}` dans son `env`, et l'étape teste
    `env.HAS_DH == 'true'` (`secrets` n'est pas lisible dans un `if:` d'étape ; le booléen ne livre pas la valeur du
    secret). Sans secret, le build passe `REGISTRY=public.ecr.aws/docker/library` au `Dockerfile`, dont les trois `FROM`
    lisent `${REGISTRY}` (défaut `docker.io/library`, la même référence que `rust:…` seul, gardé par G21 dans
    `textes_coherents.rs`), et `setup-buildx-action` reçoit `driver-opts: image=mirror.gcr.io/moby/buildkit:buildx-stable-1` ;
    avec le secret, `build-args` et `driver-opts` sont vides et le build de `main` et des branches du dépôt reste celui
    d'avant. Aucun push dans `ci.yml`.
  - **Deux critères, gardés tels quels** : le service MariaDB suit le critère « fork » (contexte `github`, seul lisible
    dans `services`), la connexion et le build suivent le critère « secret » (`HAS_DH`). Ils coïncident sur une PR de
    fork et sur une PR ou un `push` du dépôt quand le secret existe. Ils divergent dans deux cas, écrits ici : une PR
    **Dependabot** (du dépôt, sans les secrets d'Actions), et un `push` sur `main` si le secret `DOCKERHUB_TOKEN` a été
    **supprimé** (non seulement invalidé). Dans les deux, la connexion est sautée **sans signal**, le build passe par
    ECR et `mirror.gcr.io`, et le service est tiré de Docker Hub (sans authentification si les `credentials` vides
    sont acceptés) ; rien de faux n'est publié
    (`release.yml` n'est pas touché), mais la provenance du build de contrôle change. Un secret **invalide**, lui,
    échoue toujours bruyamment à la connexion.
  - **`moby/buildkit`** : `public.ecr.aws/docker/library` ne sert que les images officielles, et `moby/buildkit` n'en est
    pas une. Il est tiré de `mirror.gcr.io/moby/buildkit:buildx-stable-1`, le miroir de Docker Hub tenu par Google, sans
    authentification. Relevé le 2026-11-24 : tiré sans identifiants, même empreinte d'index
    (`sha256:baf6138e…`) que `docker.io/moby/buildkit:buildx-stable-1` lue sur Docker Hub (et que
    `public.ecr.aws/vend/moby/buildkit`, miroir tiers, écarté). `mirror.gcr.io` ne garde que les images demandées
    souvent : si l'étiquette en sortait, le builder échouerait à démarrer, bruyamment.
  - **Le cache `type=gha`** d'une PR (fork ou non) lit celui de sa base et n'écrit que dans la portée de la PR
    (`refs/pull/N/merge`), invisible des autres ; GitHub ne dit pas si une PR de fork peut y écrire : `cache-to` porte
    donc `ignore-error=true` sans secret, pour qu'une écriture refusée ne fasse pas échouer le build. Si elle est
    refusée, `e2e-smoke` ne trouve pas les couches de `docker-build` et reconstruit l'image entière, au risque de son
    délai de 25 min. Le cache Rust (`Swatinem/rust-cache`) n'est de toute façon écrit que depuis `main`.
  - **Provenance** : `public.ecr.aws/docker/library` est le miroir des images officielles Docker que tient AWS, et
    `mirror.gcr.io` celui de Docker Hub que tient Google ; ils servent les mêmes images que Docker Hub, mais ce n'est pas
    Docker Hub. Ils ne servent que les builds sans secret, jamais la release. *(Le repli noté jusque-là pour le service,
    `mirror.gcr.io`, n'a pas été retenu pour les images officielles : il ne sert ici que `moby/buildkit`.)*
  - ⚠️ **Non éprouvé sur GitHub à la rédaction** : contrôlés en local, la syntaxe (`actionlint` 1.7.7), le build du
    `Dockerfile` avec `REGISTRY=public.ecr.aws/docker/library` et le tirage anonyme des images des deux miroirs ; le
    premier run d'une PR de fork constatera le reste — service d'image vide non démarré et `credentials` de
    remplissage acceptés, MariaDB par `docker run` (essayé en local : « healthy » en une dizaine de secondes, connexion
    en `kesh`), tirages anonymes au journal, connexions sautées, builder sur `mirror.gcr.io`, cache `gha`.
    *(Annoté le 2026-11-25 : le premier run d'une PR de fork, 45525662619 sur la #1560, a échoué au tirage anonyme
    d'ECR — d'où la chaîne de miroirs ci-dessus.)*

  *(Jusqu'au 2026-11-24, ce paragraphe disait : « Les PR issues de forks et de Dependabot n'ont pas accès aux secrets :
  elles échouent au démarrage du service ou à l'étape de connexion — échec bruyant, jamais un faux vert. Aucun
  contournement conditionnel n'est écrit (C-31-2-3 : dépôt tenu par un seul développeur, sans fork actif ni
  `dependabot.yml`). » Sur la PR #1467, venue d'un fork, `Backend (Rust)` a bien été rouge ; la cause exacte — modèle
  refusé sur des `credentials` vides, ou tirage anonyme refusé — n'est pas relevée ici, faute du journal. Le démarrage
  par `docker run` sur un fork vaut dans les deux cas.)*
- **Un jeton invalide bloque les fusions.** La branche protégée `main` exige **quatre** vérifications
  (`gh api repos/guycorbaz/kesh/branches/main/protection/required_status_checks` : `Backend (Rust)`, `Docker build
  (sanity)`, `Frontend (Svelte)`, et `E2E smoke (Playwright)` depuis le 2026-10-24) ; les deux premières dépendent du jeton (service `credentials`, connexion). Un jeton
  expiré se voit donc à la première PR qui suit, par un rouge de démarrage — renouveler le secret, ne pas relancer.
  ⚠️ Un secret **supprimé** ne bloque plus rien : la connexion est sautée et le build passe par les miroirs (ci-dessus,
  « Deux critères »).

## Timeouts et concurrency

### Timeouts par job

| Job | Timeout | Motivation |
|---|---|---|
| `changes` | 10 min | `checkout` en `fetch-depth: 0`, test du script, puis le script ; quelques secondes attendues, non mesurées. Son dépassement fait annuler le job, cas à relever à l'AC-6 (e) de la Story 33-11 |
| `backend` | 120 min | Compilation + `cargo test -j1 --test-threads=1` ; 19 min 14 s mesurées sur runner le 2026-08-16 (PR #311). La marge est délibérée : un timeout se dimensionne sur le pire cas (cf. commentaire de `ci.yml`) |
| `frontend` | 20 min | Build Vite + Vitest + svelte-check ≈ 5 min |
| `docker-build` | 20 min | Build multi-stage ≈ 8 min |
| `e2e-smoke` | 25 min | Image depuis le cache `gha` + installation de chromium + une spec ; 16 min 29 s au premier run (PR #702) |

Les timeouts évitent qu'un test gelé (typiquement `PoolTimedOut` SQLx cross-binary) mobilise un runner pendant 6 h (défaut GitHub Actions).

### Concurrency

```yaml
# ci.yml
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

# release.yml
concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false
```

- `ci.yml` annule le run précédent si un nouveau push survient sur la même ref (branche ou PR) — évite la file d'attente sur runners gratuits.
- `release.yml` garde `cancel-in-progress: false` — on ne veut **jamais** annuler une release SemVer en cours, même si deux tags partent en rafale (ex `v0.1.0` puis `v0.1.1`). Garantit publication séquentielle.

## Permissions

`ci.yml` déclare `permissions: { contents: read }` au niveau workflow (principle of least privilege). Aucun scope d'écriture nécessaire (pas de commit back, pas d'issue comment).

`release.yml` garde `contents: write` (nécessaire pour créer une GitHub Release via `softprops/action-gh-release`).

## Migrations SQLx

Les migrations sont appliquées à **trois** endroits :

1. **CI explicite** dans le job `backend` (étape « Apply migrations to kesh DB ») : une boucle `mariadb <` en `root` sur `crates/kesh-db/migrations/*.sql`. **Requis** par les tests `kesh-db` qui utilisent `test_pool()` — connexion directe à la DB `kesh` (pas `sqlx::test` éphémère). ⚠️ Cette boucle ne remplit pas `_sqlx_migrations` : la base `kesh` du job ne sert qu'à ces tests, aucun backend n'y démarre.
2. **Démarrage du backend** : `MIGRATOR.run()` au démarrage de `kesh-api` (`crates/kesh-api/src/main.rs`) — c'est ainsi que le job `e2e-smoke` et le smoke de `release.yml` migrent leur base, comme en production.
3. **Tests `sqlx::test`** : SQLx applique les migrations (ou le squash `test-schema/`) sur chaque base éphémère `_sqlx_test_*`.

## Seed CI (job `backend`)

Le job `backend` injecte plusieurs rows dans la DB `kesh` après les migrations. Contenu aligné avec le helper `crates/kesh-db/src/test_fixtures.rs::seed_accounting_company` (Story 6-4) :

| Table | Row | Raison |
|---|---|---|
| `companies` | `'CI Seed Company'` (org_type `Independant`, langues `FR`/`FR`) | `kesh-db::repositories::*::tests::get_company_id()` exige ≥ 1 company |
| `users` | `'admin'` (role `Admin`, hash Argon2id de `admin123`) | `get_admin_user_id()` exige ≥ 1 user `Admin` |
| `accounts` × 5 | `1000/1100` (Asset), `2000` (Liability), `3000` (Revenue), `4000` (Expense) | `two_accounts()` exige ≥ 2 comptes actifs ; couverture des 4 types pour tests journal_entries |
| `fiscal_years` | `'Exercice CI 2020-2030'` (Open) | `journal_entries::tests::setup()` exige un exercice ouvert couvrant `today` |

**Notes** :
- Le hash Argon2id est pré-calculé (cf. `ADMIN_PASSWORD_HASH` dans `test_fixtures.rs`) pour que les tests E2E qui loguent `admin/admin123` trouvent le user.
- Le fiscal_year couvre 2020-2030 délibérément large pour tolérer la dérive d'horloge entre runs CI.
- Le bootstrap admin de `kesh-api` ne peut pas remplir ce rôle car il crée un user mais **pas** de company (la company passe par le flow onboarding, hors scope d'un seed CI).

## Fixtures E2E déterministes (Story 6-4)

Le job `e2e-smoke` (comme le montage local) active `KESH_TEST_MODE=true` pour exposer `POST /api/v1/_test/seed` et ses presets (`fresh`, `with-company`/`post-onboarding`, `with-data`, …). Le helper Playwright `frontend/tests/e2e/helpers/test-state.ts` appelle cet endpoint avant chaque scénario pour garantir un état de DB reproductible.

Garde-fou sécurité : le backend refuse de démarrer si `KESH_TEST_MODE=true` **et** `KESH_HOST != 127.0.0.1` (cf. `ConfigError::TestModeWithPublicBind`). Loopback strict requis — `0.0.0.0` rejeté.

Le fail-fast avant les specs est le `globalSetup` de Playwright (`frontend/tests/e2e/global-setup.ts`), qui appelle `seedTestState('with-company')` une fois et liste les prérequis si l'endpoint ne répond pas ; aucune étape `curl` ne le précède dans le job.

## Décision MariaDB 10.11 (Story 10-1 D3 — parité prod NAS)

**Mise à jour 2026-05-21** (Story 10-1 D3) : alignement à `mariadb:10.11` partout (dev + CI + prod) pour parité avec la version Package Center Synology DSM 7.x du NAS de Guy (production v0.1.0). Compat MariaDB ≥ 10.6 confirmée par sanity check pre-flight (26 migrations passent sur 10.11 vierge, dont la VIRTUAL+UNIQUE column de `reconciliation_rules` Story 8-5b). *(Depuis la Story 33-1b, le plancher est 10.7 : `NATURAL_SORT_KEY`.)*

**Historique** : la pipeline CI utilisait antérieurement MariaDB 11.4 (Story 6-1 décision 2026-04-15). Le passage à 10.11 dans Story 10-1 vise à éviter tout drift entre la version testée en CI et celle utilisée en prod sur le NAS. `architecture.md` impose « MariaDB 10.7+ » comme minimum depuis la Story 33-1b (« 10.6+ » auparavant) — 10.11 respecte le minimum avec marge.

### Justification mono-version 10.11 (Story 10-2 AC #20)

La pipeline CI utilise **un seul service MariaDB** en version 10.11 — aucune matrice `strategy.matrix.mariadb-version` n'est introduite. Trois raisons :

- **Cible prod unique** : le NAS Synology Package Center DSM ≥ 7.2 ne propose que MariaDB 10.x stable (10.11 spécifiquement). Une matrice qui inclurait 11.x ferait passer des tests sur un moteur que personne ne tournera en prod, sans gain de couverture.
- **Bugs 10.11-specific seraient masqués par 11** : si une feature dépend d'un comportement spécifique à MariaDB 11, elle passerait sur la branche matrice 11 mais casserait silencieusement sur 10.11 — sans matrice, le test 10.11 attrape immédiatement.
- **Compat upstream MariaDB ≥ 10.7 documentée** : le tri des numéros internes emploie `NATURAL_SORT_KEY`, apparue en 10.7 (Story 33-1b, `C-e331-19`) ; la migration `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql:27-28`, qui ne se touche pas, parle encore de 10.6+ pour sa propre colonne. Pour les opérateurs hors NAS Synology qui voudraient tourner sur 10.7/10.8/10.9/10.10, cette compat **n'est pas testée par la CI projet** — c'est de la documentation, pas une garantie.

## Dette technique CI

| ID | Description | Sévérité | Statut |
|---|---|---|---|
| **D-6-1-A** | Cache des browsers Playwright (`~/.cache/ms-playwright` via `actions/cache@v4`) — économie ~50s par run e2e | LOW | Reporté — bénéfice marginal solo-dev ; à reconsidérer si `e2e-smoke` dépasse 10 min (Story 31-2) |
| **D-6-1-B** | Pinning des actions GitHub aux SHA commit (vs `@v4` etc.) — sécurité supply chain | LOW | Reporté — Dependabot peut automatiser |
| **D-6-1-C** | Couverture axe-core partielle (6/12 specs) — manquent `accounts`, `journal-entries`, `mode-expert`, `onboarding`, `onboarding-path-b`, `invoices_echeancier` | LOW | Follow-up opportuniste |
| **D-6-1-D** | Test axe `/invoices` valide l'empty state uniquement — l'état peuplé sera couvert par Story 6-4 (`seed_accounting_company`) | LOW | Partiellement résolu — Story 6-4 mergée, adoption dans le spec `invoices` à faire en follow-up |
| **D-6-1-E** | Tests Playwright bloqués post-seed (`continue-on-error: true` sur le step `Run Playwright`) | HAUTE | KF-007 / issue #19 — ⛔ blocant release v0.1, tracé pour Story 6-5 |

*(Annotation du 2026-10-23, Story 31-2 : D-6-1-A et D-6-1-E décrivent un job `e2e` complet en CI — **non retenu** : ce job n'a jamais figuré dans `ci.yml` (#577). La CI joue le seul smoke `e2e-smoke` ; D-6-1-A s'y applique, D-6-1-E est sans objet.)*
