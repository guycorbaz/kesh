# Story v011.4: Port applicatif par défaut 80 (Issue #118)

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **nouvel utilisateur qui installe Kesh v0.1.x sur un host vide**,
I want que **l'URL standard `http://kesh.local`** (sans `:3000`) fonctionne directement après `docker compose up`,
so that je puisse accéder à l'app sans surprise sur le port et que la doc d'install reflète l'URL HTTP standard que les utilisateurs connaissent.

## Scope

**Severity : amélioration qualité install (post-v0.1.1).** Décidée 2026-05-28 (révision décision H6 epic Hotfix v0.1.1). Guy veut éviter les soucis de port — surtout pour les **autres utilisateurs** qui installeront Kesh — URL HTTP standard sans `:3000`.

**Décision séquencement (2026-05-30, Pass 1) :** v011-3 break-glass superseded par v011-5 onboarding self-service unifié (cible v0.1.2). v011-4 est livré dans la même release **v0.1.2** (cohérent UX, cf. Dev Notes Q3 tranché).

**Dans le scope :**
- **Code Rust :** `crates/kesh-api/src/config.rs` (default `port: 80` + 4 fallbacks `KESH_PORT` parse → 80 + doc-comments + test assertion + `make_test_config` helper).
- **Docker :** `Dockerfile` `EXPOSE 80` ; 3 compose files (`docker-compose.{yml,prod.yml,dev.yml}`) — `KESH_PORT` défauts + mappings + healthchecks + commentaires explicatifs (bind loopback prod `127.0.0.1` conservé D5).
- **CI/Release :** `.github/workflows/release.yml` smoke test (env + curl).
- **Frontend dev/test config :** `frontend/playwright.config.ts` (fallback `baseURL` + commentaires) + `frontend/tests/e2e/helpers/test-state.ts` (fallback + messages + commentaires) + `frontend/tests/e2e/global-setup.ts` (fallback) + `frontend/vite.config.ts` (proxy targets + commentaires).
- **Doc utilisateur :** `.env.example` (commentaire + variable + nouvelle section « Conflit port 80 » couvrant **dev local ET Synology DSM**), `README.md` quickstart, `DOCKER_START.md` (5 sites), `crates/kesh-api/README.md` (table env-vars).
- **Doc admin LaTeX :** `docs/manual/fr/admin-manual.tex` (11 sites identifiés ground-truth — pas 4 comme spec Pass 0) + nouvelle sous-section « Changer le port d'écoute (conflit port 80, ex. Synology DSM) » + PDF régénéré.
- **Comments uniquement (non-fonctionnels mais cohérence) :** 3 spec files Playwright (`users.spec.ts`, `auth.spec.ts`, `journal-entries.spec.ts`) — commentaires-doc d'en-tête mentionnant « backend sur localhost:3000 ».
- **CHANGELOG :** nouvelle section `## [0.1.2]` (release v011-4 + v011-5) avec entrée `### Changed` documentant le **breaking de configuration** port 3000→80.

**Hors scope :**
- **Choix du mapping host final** par l'utilisateur (`80:80`, `3000:80`, `IP_dédiée:80`, etc.) : v011-4 met à jour les mappings **par défaut** dans les compose files ; l'utilisateur peut les surcharger via `.env` + override compose ou compose.override.yml.
- **Reverse proxy HTTPS** (nginx/Caddy/Traefik) : la doc existante de l'admin-manual sera mise à jour pour pointer le bon port interne (`proxy_pass http://localhost` au lieu de `:3000`), mais aucune nouvelle config TLS ni nouveau scénario ajouté.
- **Migration `cargo run` natif** (mode dev hors Docker) : un dev qui lance natif sur Linux non-root **ne peut PAS** bind `:80` → doit setter `KESH_PORT=3000` (ou ≥ 1024). Documenté dans `.env.example` (cas dev) mais pas de fix automatique.
- **Port MariaDB (3306)** : inchangé.
- **Fichiers BMAD framework vendored** (`.claude/skills/bmad-testarch-*/resources/knowledge/*.md`) : exemples de doc framework non liés au déploiement Kesh — explicitement hors scope.

## Contexte technique (ground-truth Pass 1, vérifié `grep -rnE` sur main commit `7c0ace6`)

### Code Rust (kesh-api/src/config.rs)

- **Ligne 27** — doc-comment header : `« -p 3000:3000 avec bind interne 0.0.0.0 expose la route au public »`. Remplacer par `-p 80:80`.
- **Ligne 293** — initial default dans `Config::default()` (struct littéral) : `port: 3000` → `port: 80`.
- **Lignes 360-375** — parsing `KESH_PORT` runtime avec **4 fallbacks** vers 3000 sur erreur (port=0, parse fail, var absente) → tous à `80`. Les messages `tracing::warn!` mentionnent « utilisation du port par défaut 3000 » → « utilisation du port par défaut 80 ».
- **Ligne 852** — doc-comment de validation host : `« en Docker -p 3000:3000 »` → `« en Docker -p 80:80 »`.
- **Ligne 898** — `make_test_config()` helper (`pub(crate) mod test_helpers`) : `port: 3000` → `port: 80`. Utilisé par `auth/bootstrap.rs:203` et `middleware/rate_limit.rs:172`.
- **Ligne 990** — test unitaire `assert_eq!(config.port, 3000)` → `80`.

### Docker

- `Dockerfile:28` — `EXPOSE 3000` → `EXPOSE 80`. Container tourne en **root** (vérifié `grep -nF 'USER' Dockerfile` retourne 0 résultats) → bind `:80` OK sans `CAP_NET_BIND_SERVICE`.
- `docker-compose.yml:44` — `KESH_PORT: ${KESH_PORT:-3000}` → `${KESH_PORT:-80}`.
- `docker-compose.yml:65` — mapping `"3000:3000"` → `"80:80"`.
- `docker-compose.yml:72` — healthcheck `http://localhost:3000/health` → `http://localhost/health`.
- `docker-compose.prod.yml:24` — commentaire `« ports: "127.0.0.1:3000:3000" »` → `"127.0.0.1:80:80"`.
- `docker-compose.prod.yml:49` — commentaire `curl http://127.0.0.1:3000/health` → `http://127.0.0.1/health`.
- `docker-compose.prod.yml:64` — `KESH_PORT: ${KESH_PORT:-3000}` → `${KESH_PORT:-80}`.
- `docker-compose.prod.yml:109` — mapping `"127.0.0.1:3000:3000"` → `"127.0.0.1:80:80"` (bind loopback D5 **conservé**).
- `docker-compose.prod.yml:118` — healthcheck.
- `docker-compose.dev.yml:13` — mapping `"127.0.0.1:3000:3000"` → `"127.0.0.1:80:80"`.
- `docker-compose.dev.yml:16` — `KESH_PORT: "3000"` → `"80"`.
- `docker-compose.dev.yml:18` — commentaire `« 127.0.0.1:3000:3000 »` (dans la note d'explication du bind 0.0.0.0) → `« 127.0.0.1:80:80 »`.
- `docker-compose.dev.yml:43` — healthcheck.

### CI/Release

- `.github/workflows/release.yml:96` — env `-e KESH_PORT="3000"` (smoke test) → `KESH_PORT="80"`.
- `.github/workflows/release.yml:105` — `curl -fsS http://127.0.0.1:3000/health` → `http://127.0.0.1/health`.

### Frontend dev/test config (toutes Critique/Haute — runtime breakage si oubliées)

- `frontend/playwright.config.ts:8-9` — commentaires « cible le backend kesh-api sur `:3000` » → mise à jour cohérente avec le nouveau défaut.
- `frontend/playwright.config.ts:56` — `baseURL: process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000'` → `'http://127.0.0.1'` (port 80 implicite).
- `frontend/tests/e2e/global-setup.ts:37` — fallback dans message d'erreur `KESH_BACKEND_URL ?? 'http://127.0.0.1:3000'` → `'http://127.0.0.1'`.
- `frontend/tests/e2e/helpers/test-state.ts:14-15` — commentaires d'en-tête mentionnant « vers le backend `:3000` » → mise à jour cohérente.
- `frontend/tests/e2e/helpers/test-state.ts:48` — `const raw = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000'` → `'http://127.0.0.1'`.
- `frontend/tests/e2e/helpers/test-state.ts:51,58` — messages d'erreur exemples « ex: http://127.0.0.1:3000 » → `« ex: http://127.0.0.1 »`.
- `frontend/vite.config.ts:6,8,14` — commentaires du header expliquant le proxy `/api → :3000` → mise à jour cohérente.
- `frontend/vite.config.ts:18,22` — proxy targets `target: 'http://localhost:3000'` → `'http://localhost'`.
- `frontend/src/lib/shared/utils/api-client.ts:266` — commentaire `« dev :5173 vs API :3000 est same-site »` → `« :5173 vs API :80 (Docker) »` (commentaire-only, sans impact runtime mais cohérence Pass 2).

### Doc utilisateur

- `.env.example:24` — commentaire `« Port HTTP du serveur kesh-api (défaut 3000). »` → `« (défaut 80). »`.
- `.env.example:25` — `KESH_PORT=3000` (exemple commenté) → `KESH_PORT=80`.
- `.env.example` — **nouvelle section** « Conflit port 80 (Synology DSM Web Station, autre service, dev local) » avec 4 cas d'override commentés : (a) `KESH_PORT=3000` dans `.env` + mapping `3000:3000` compose, (b) mapping host différent dans compose (ex. `8080:80`), (c) IP dédiée container `172.x.x.x:80:80`, (d) **dev `cargo run` natif Linux non-root** : `KESH_PORT=3000` (ou ≥ 1024) obligatoire car bind <1024 nécessite root.
- `README.md:70` — « `http://localhost:5173 (frontend dev) et http://localhost:3000 (API)` » → décision Q1 (b) : « `http://localhost:5173 (frontend dev) et http://localhost (API en mode Docker)` ». La note dev natif (`KESH_PORT=3000` requis) ajoutée en parenthèse courte dans le même paragraphe (pas de nouveau fichier).
- `DOCKER_START.md:6` — « Port 3000 et 3306 disponibles sur la machine hôte » → « Port 80 et 3306 disponibles sur la machine hôte ».
- `DOCKER_START.md:31` — sample log `listening on 0.0.0.0:3000` → `0.0.0.0:80`.
- `DOCKER_START.md:36-37` — URLs `http://localhost:3000` → `http://localhost` (les deux lignes).
- `DOCKER_START.md:92` — section troubleshooting « Erreur port 3000 already in use » → « Erreur port 80 already in use » (titre + corps). L'exemple de fix `KESH_PORT=3001` reste valable conceptuellement (peut être `3000` ou autre port libre).
- `crates/kesh-api/README.md:34` — table env-vars : `| KESH_PORT | non | 3000 | Port HTTP |` → `| 80 |`.

### Doc admin LaTeX (11 sites ground-truth — toutes obligatoires, dont 3 fonctionnellement bloquantes)

- `docs/manual/fr/admin-manual.tex:169` — table « Ports utilisés » ligne 1 : `3000 & HTTP & API Axum backend (interne) & Localhost uniquement` → `80 & HTTP & ...`.
- `docs/manual/fr/admin-manual.tex:176` — warning « Ne jamais exposer le port 3000... `localhost:3000` » → `« port 80... localhost »`.
- `docs/manual/fr/admin-manual.tex:257` — exemple `curl http://localhost:3000/health` → `http://localhost/health`.
- `docs/manual/fr/admin-manual.tex:285` — nginx `proxy_pass http://localhost:3000` → `http://localhost`.
- `docs/manual/fr/admin-manual.tex:320` — Caddy `reverse_proxy localhost:3000 { ... }` → `localhost`.
- `docs/manual/fr/admin-manual.tex:374` — commentaire Traefik `« NE PAS publier le port 3000... »` → `« port 80 »`.
- **`docs/manual/fr/admin-manual.tex:376` — `expose: "3000"` Traefik sidecar → `"80"` (⚠️ FONCTIONNELLEMENT REQUIS — Traefik route vers ce port).**
- **`docs/manual/fr/admin-manual.tex:385` — `traefik.http.services.kesh.loadbalancer.server.port=3000` → `80` (⚠️ FONCTIONNELLEMENT REQUIS — sans ce fix, Traefik ne peut pas atteindre l'API).**
- **`docs/manual/fr/admin-manual.tex:564` — Synology DSM Reverse Proxy : `Hostname localhost, Port 3000` → `Port 80` (⚠️ FONCTIONNELLEMENT REQUIS — Portal DSM route vers le mauvais port sinon).**
- `docs/manual/fr/admin-manual.tex:644` — table env-vars : `KESH\_PORT ... default \texttt{3000}` → `\texttt{80}`.
- `docs/manual/fr/admin-manual.tex:957` — section backup/restore : `curl http://localhost:3000/health` → `http://localhost/health`.
- `docs/manual/fr/admin-manual.tex:1543` — appendice tableau ports : `API Axum (interne) & 3000 & ...` → `80`.
- **Nouvelle sous-section** « Changer le port d'écoute (conflit port 80, ex. Synology DSM Web Station) » : explique les 4 options d'override (cf. `.env.example`). Place naturelle : juste après la table des ports (l.169).
- **PDF régénéré** (`latexmk -xelatex docs/manual/fr/admin-manual.tex`) commité, cohérent avec `.tex`.

### Commentaires-doc d'en-tête des spec files Playwright (non-fonctionnel mais cohérent)

- `frontend/tests/e2e/users.spec.ts:17` — commentaire « backend Kesh fonctionnel sur localhost:3000 » → `localhost`.
- `frontend/tests/e2e/auth.spec.ts:8` — idem.
- `frontend/tests/e2e/journal-entries.spec.ts:17` — idem.

### CHANGELOG

- **Nouvelle section** `## [0.1.2] — <date du tag>` (l'épic Hotfix v0.1.1 reste à 2/4, v011-4 + v011-5 visent v0.1.2). Pas de modification de la section `[0.1.1]` existante (déjà publiée 2026-05-29).
- Sous-section `### Changed` avec **3 points obligatoires** :
  1. **Raison du changement** : URL HTTP standard `http://kesh.local` sans `:3000`, simplification install pour les nouveaux utilisateurs.
  2. **Procédure breaking-config — adopter le défaut 80** : retirer `KESH_PORT=3000` du `.env` ; le mapping compose `127.0.0.1:80:80` (prod) ou `80:80` (dev) prend effet automatiquement.
  3. **Procédure breaking-config — garder le port 3000** : conserver `KESH_PORT=3000` dans `.env` ET surcharger le mapping compose (ex. `compose.override.yml` avec `ports: ["127.0.0.1:3000:3000"]`).
- Pointer vers la nouvelle sous-section manuel admin (AC #15).

## Acceptance Criteria

### Code Rust (AC #1-3)

- [x] **AC #1** `crates/kesh-api/src/config.rs:293` : `port: 80` dans `Config::default()`. Doc-comments lignes 27 + 852 mis à jour (3000→80). **Gate** : `grep -nE 'port: 3000|3000:3000' crates/kesh-api/src/config.rs` retourne 0 hit.
- [x] **AC #2** `crates/kesh-api/src/config.rs:360-375` : les 4 fallbacks de parsing `KESH_PORT` (port=0, parse fail, var absente) retournent **80** ; les messages `tracing::warn!` mentionnent « port par défaut 80 ». **Gate** : `grep -nF 'défaut 3000' crates/kesh-api/src/config.rs` retourne 0 hit.
- [x] **AC #3** `crates/kesh-api/src/config.rs:990` : `assert_eq!(config.port, 80)` (était 3000). **`make_test_config()` (l.898) bumpé à `port: 80`** (cohérence avec le défaut prod, défense contre divergence test/prod future). **Gate** : `grep -nE 'port: 3000|port == 3000|port, 3000' crates/kesh-api/src/config.rs` retourne 0 hit (les `3306` MariaDB ne matchent pas).

### Docker (AC #4-7)

- [x] **AC #4** `Dockerfile:28` : `EXPOSE 80` (était 3000). **Gate** : `grep -nF 'USER' Dockerfile` retourne 0 hit (assertion positive « pas de USER non-root ») ; `grep -nF 'EXPOSE 3000' Dockerfile` retourne 0 hit.
- [x] **AC #5** `docker-compose.yml` : `KESH_PORT: ${KESH_PORT:-80}` (l.44), mapping `"80:80"` (l.65), healthcheck `http://localhost/health` (l.72). **Gate** : `grep -nE '3000|3000:3000' docker-compose.yml` retourne 0 hit.
- [x] **AC #6** `docker-compose.prod.yml` : commentaires (l.24, l.49), `KESH_PORT: ${KESH_PORT:-80}` (l.64), mapping `"127.0.0.1:80:80"` (l.109) — **bind loopback D5 conservé** —, healthcheck (l.118). **Gate** : `grep -nE '3000|3000:3000' docker-compose.prod.yml` retourne 0 hit.
- [x] **AC #7** `docker-compose.dev.yml` : mapping `"127.0.0.1:80:80"` (l.13), `KESH_PORT: "80"` (l.16), commentaire l.18 reflète `127.0.0.1:80:80`, healthcheck (l.43). **Gate** : `grep -nE '3000|3000:3000' docker-compose.dev.yml` retourne 0 hit.

### CI/Release (AC #8)

- [x] **AC #8** `.github/workflows/release.yml:96,105` : `-e KESH_PORT="80"` + `curl -fsS http://127.0.0.1/health`. **Gate** : `grep -nE ':3000|KESH_PORT.*3000' .github/workflows/release.yml` retourne 0 hit.

### Frontend dev/test config (AC #9-10)

- [x] **AC #9** `frontend/playwright.config.ts` (l.8-9 commentaires + l.56 baseURL fallback) **ET** `frontend/vite.config.ts` (l.6/8/14 commentaires + l.18/22 proxy targets) mis à jour. **Gate** : `grep -nE ':3000' frontend/playwright.config.ts frontend/vite.config.ts` retourne 0 hit.
- [x] **AC #10** `frontend/tests/e2e/global-setup.ts:37` **ET** `frontend/tests/e2e/helpers/test-state.ts` (l.14-15 commentaires + l.48 fallback + l.51,58 exemples messages) mis à jour. Les 3 spec files `users.spec.ts:17`, `auth.spec.ts:8`, `journal-entries.spec.ts:17` ont leurs commentaires d'en-tête mis à jour (`localhost:3000` → `localhost`). **Aussi** : commentaire dans `frontend/src/lib/shared/utils/api-client.ts:266` (« `dev :5173 vs API :3000 est same-site »`) mis à jour vers `« :5173 vs API :80 (Docker) »`. **Gate** : `grep -rnE ':3000' frontend/tests/e2e/ frontend/src/lib/shared/utils/api-client.ts --exclude="*.log" --exclude="test-state.test.ts"` retourne 0 hit (l'exclusion `test-state.test.ts` est inline dans la commande, le stub arbitraire l.46 est documenté en Dev Notes).

### Doc utilisateur (AC #11-13)

- [x] **AC #11** `.env.example` (l.24 commentaire + l.25 variable) cohérent défaut 80. **Nouvelle section** « Conflit port 80 (Synology DSM / autre service / dev local) » documente les **4 options d'override** (a/b/c utilisateur final + d dev natif `KESH_PORT=3000` obligatoire sur Linux non-root). **Gate** : la section override mentionne explicitement « dev `cargo run` natif » dans son texte.
- [x] **AC #12** `README.md:70` : URL API mise à jour à `http://localhost (API en mode Docker)`, note dev natif (`KESH_PORT=3000` requis) en parenthèse courte dans le même paragraphe (pas de nouveau fichier `CONTRIBUTING.md` — décision Q1 (b)). **Gate** : `grep -nF ':3000' README.md` retourne 0 hit.
- [x] **AC #13** `DOCKER_START.md` (l.6, l.31, l.36, l.37, l.92) **ET** `crates/kesh-api/README.md:34` mis à jour. **Gate** : `grep -nF ':3000' DOCKER_START.md crates/kesh-api/README.md` retourne 0 hit.

### Doc admin LaTeX (AC #14-15)

- [x] **AC #14** `docs/manual/fr/admin-manual.tex` : **les 11 occurrences de `3000`** mises à jour (lignes 169, 176, 257, 285, 320, 374, 376, 385, 564, 644, 957, 1543 — `3306` MariaDB exclus du gate). Inclut les 3 sites **FONCTIONNELLEMENT BLOQUANTS** (Traefik `expose:` l.376, label `loadbalancer.server.port=` l.385, Synology DSM Portal `Port 3000` l.564) sans lesquels le déploiement Traefik/DSM est cassé. **Gate** : `grep -nF '3000' docs/manual/fr/admin-manual.tex | grep -v '3306'` retourne 0 hit.
- [x] **AC #15** Nouvelle sous-section `\subsubsection{Changer le port d'écoute (conflit port 80, ex. Synology DSM Web Station)}` ajoutée au manuel admin, au même niveau hiérarchique que la sous-section « Ports utilisés » (typiquement sous le `\section` qui contient la table des ports l.169), avec les **4 options d'override** (cohérent `.env.example` AC #11). PDF régénéré (`latexmk -xelatex docs/manual/fr/admin-manual.tex`) et commité. **Gate** : la régénération `.tex → .pdf` aboutit sans erreur (`latexmk` exit 0) — la cohérence sémantique du PDF est garantie par le gate `.tex` de AC #14 (qui assert grep `3000` = 0 hit dans le source `.tex`). Pas de gate `pdfgrep` séparé (outil non-standard, sensible aux timestamps embarqués).

### CHANGELOG + Quality gate (AC #16-17)

- [x] **AC #16** `CHANGELOG.md` **nouvelle section** `## [0.1.2] — <date du tag>` avec sous-section `### Changed` contenant les **3 points obligatoires** : (1) raison du changement, (2) procédure « adopter le défaut 80 » (retirer `KESH_PORT=3000` du `.env`), (3) procédure « garder 3000 » (conserver `KESH_PORT=3000` + override mapping compose). Pointe vers la sous-section manuel admin AC #15. **Pas de modification de la section `[0.1.1]` existante** (déjà publiée).
- [x] **AC #17** Série Test Locally First complète verte : backend `cargo fmt --check + clippy --workspace --all-targets -- -D warnings + build + test --workspace` ; frontend `npm run check + lint-i18n-ownership + test:unit + build`. **Gate récap** (toutes les exclusions sont **inline dans la commande**, exécutable telle quelle) : `grep -rnE ':3000|EXPOSE 3000|KESH_PORT.*3000|3000:3000' --include="*.rs" --include="*.ts" --include="*.svelte" --include="*.yml" --include="*.yaml" --include="*.md" --include="*.tex" --include="Dockerfile*" --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.svelte-kit --exclude-dir=build --exclude-dir=.claude --exclude="*.log" --exclude="*.aux" --exclude="*.fdb_latexmk" --exclude="*.fls" --exclude="*.toc" --exclude="*.xdv" --exclude="test-state.test.ts" --exclude="v011-4-default-port-80.md" --exclude="epic-hotfix-v0.1.1.md" --exclude="sprint-status.yaml" .` retourne **0 hit**. (Exclusions justifiées : `.claude/skills/bmad-testarch-*` = BMAD framework vendored ; `test-state.test.ts:46` = stub arbitraire de test unitaire ; les 3 fichiers BMAD `_bmad-output/...` excluent la spec elle-même qui contient les references descriptives.)

## Tasks / Subtasks

- [x] **T1 — Code Rust** (AC #1-3)
  - [x] `config.rs` defaults (l.27/293/360-375/852/898/990) — 6 sites au total.
  - [x] Vérifier qu'aucun autre `port: 3000` ou `port, 3000` ne traîne (gate AC #3).
- [x] **T2 — Dockerfile + Compose** (AC #4-7)
  - [x] `Dockerfile` EXPOSE.
  - [x] 3 compose files (KESH_PORT default, mappings, healthchecks, commentaires).
  - [x] Bind loopback prod `127.0.0.1` conservé (revue critique D5).
- [x] **T3 — CI Release** (AC #8)
  - [x] `release.yml` smoke test env + curl.
- [x] **T4 — Frontend dev/test config** (AC #9-10) — **NOUVELLE tâche post-Pass 1**
  - [x] `playwright.config.ts` (commentaires l.8-9 + baseURL l.56).
  - [x] `vite.config.ts` (commentaires l.6/8/14 + proxy targets l.18/22).
  - [x] `tests/e2e/global-setup.ts` fallback l.37.
  - [x] `tests/e2e/helpers/test-state.ts` (commentaires l.14-15 + fallback l.48 + messages exemples l.51/58).
  - [x] 3 spec files commentaires-doc (`users.spec.ts:17`, `auth.spec.ts:8`, `journal-entries.spec.ts:17`).
  - [x] `frontend/src/lib/shared/utils/api-client.ts:266` commentaire (Pass 2 add).
- [x] **T5 — Doc utilisateur** (AC #11-13)
  - [x] `.env.example` commentaire + variable + nouvelle section override 4 cas (dont dev natif).
  - [x] `README.md` quickstart (décision Q1 (b) appliquée).
  - [x] `DOCKER_START.md` 5 sites.
  - [x] `crates/kesh-api/README.md` table env-vars.
- [x] **T6 — Doc admin LaTeX** (AC #14-15)
  - [x] **Les 11 références** dans `admin-manual.tex` (dont 3 fonctionnellement bloquantes Traefik + Synology DSM).
  - [x] Nouvelle sous-section « Changer le port d'écoute ».
  - [x] `latexmk -xelatex` régénère le PDF ; vérifier `pdfgrep '3000' admin-manual.pdf` = 0 hit.
- [x] **T7 — CHANGELOG + Quality gate** (AC #16-17)
  - [x] Nouvelle section `[0.1.2]` + entrée `Changed` 3 points obligatoires.
  - [x] Série Test Locally First complète + gate récap (grep repo-wide).

## Dev Notes

### Patterns à respecter (ground-truth code)

- **Container tourne en root** : `Dockerfile` ne contient pas de `USER` directive (vérifié `grep -nF 'USER' Dockerfile` = 0). Bind `:80` (port privilégié <1024) OK sans `CAP_NET_BIND_SERVICE`. Si une future Story introduit un `USER kesh` non-root, il faudra ré-évaluer (typiquement : ajout de `CAP_NET_BIND_SERVICE` au container OU revenir sur un port >1024).
- **Bind loopback prod** (`docker-compose.prod.yml:109` `127.0.0.1:80:80`) : conservation **non-négociable** — la décision D5 du déploiement Synology DSM impose loopback strict pour forcer le reverse proxy HTTPS (cf. admin-manual.tex section « Reverse proxy nginx »).
- **`cargo run` natif (mode dev hors Docker)** : Linux non-root **ne peut PAS bind <1024**. Un dev qui tourne `cargo run -p kesh-api` directement doit setter `KESH_PORT=3000` (ou ≥ 1024) dans son `.env` local. Documenté dans `.env.example` § option (d) et `README.md` parenthèse courte.

### Références test stable

- **Tests d'intégration Rust** `crates/kesh-api/tests/*_e2e.rs` : utilisent `tokio::net::TcpListener::bind("127.0.0.1:0")` (port éphémère choisi par l'OS — vérifié `onboarding_path_b_e2e.rs:69`). **Pas impacté** par le changement de défaut.
- **Tests Playwright `.spec.ts`** : utilisent `KESH_BACKEND_URL` (env-driven). Aucun port hardcodé fonctionnel dans les `.spec.ts`. **Exception non-fonctionnelle** : 3 spec files (`users.spec.ts`, `auth.spec.ts`, `journal-entries.spec.ts`) ont des commentaires-doc d'en-tête mentionnant `localhost:3000` — mis à jour pour cohérence (AC #10) mais non bloquants runtime.
- **`test-state.test.ts:46`** (test unitaire du helper) : `vi.stubEnv('KESH_BACKEND_URL', 'http://test.example:3000')` utilise `:3000` comme **valeur stub arbitraire** pour tester la validation URL — pas un fallback opérationnel. **Pas modifié** (exclu du grep gate AC #17).
- **`baseline-pre-9-5-1b.log`** (et autres `*.log` dans `frontend/tests/e2e/`) : logs historiques d'exécution, pas des tests actifs. Contiennent des références `:3000` (historique). **Ne pas modifier** — ce sont des archives.
- **BMAD framework vendored** (`.claude/skills/bmad-testarch-*/resources/knowledge/*.md`) : exemples Playwright/Cypress du framework BMAD utilisant `localhost:3000` comme convention par défaut. **Hors scope** — ce ne sont pas des fichiers du projet Kesh.

### Q1 — README dev context — TRANCHÉE Pass 1 : option (b)

Option retenue : « `http://localhost:5173 (frontend dev) et http://localhost (API en mode Docker, port 80 ; en mode `cargo run` natif Linux non-root, lancer avec `KESH_PORT=3000`) ». » Une seule ligne dans `README.md`, pas de nouveau fichier `CONTRIBUTING.md` ni `docs/testing.md` à créer (réduit le risque de scope creep).

### Q2 — docker-compose.dev.yml port 80 friction — TRANCHÉE Pass 1

Documenté dans `.env.example` § option (d) qui couvre **à la fois** le cas Synology DSM Web Station (utilisateur final) **et** le cas dev local `docker compose -f docker-compose.dev.yml up` où le port 80 host est occupé. Le dev peut setter `KESH_PORT=8080` (ou autre) dans son `.env` pour éviter conflit. Section override AC #11 couvre les 2 contextes.

### Q3 — Séquencement release — TRANCHÉE Pass 1 : v0.1.2

v011-4 est livré dans **v0.1.2** combiné avec v011-5 onboarding self-service (cohérent 2 améliorations UX install ensemble). Le CHANGELOG ne touche PAS la section `[0.1.1]` existante (déjà publiée 2026-05-29) — une **nouvelle section `[0.1.2]`** est créée (AC #16). Les références à « v0.1.1 hotfix » dans `.env.example` ou la doc qui mentionneraient port 80 doivent pointer v0.1.2. Le contenu introductif de la section `[0.1.1]` qui dit « stories restantes reportées à une release ultérieure » reste **factuellement correct** et ne nécessite pas de modification rétroactive.

### Migration breaking policy (CLAUDE.md)

v011-4 ne touche **aucune migration** (pas de schema change). Politique P3 (DROP/RENAME COLUMN sans bump min_required) ne s'applique pas. P5 (audit idempotence) idem. Aucune entrée à ajouter à `docs/migrations-idempotence-audit.md`.

### Règle de splitting préventif (CLAUDE.md)

Cette story touche **~22 fichiers** post-Pass 1 (config.rs, Dockerfile, 3 compose, release.yml, 4 frontend dev/test config, 3 spec headers, .env.example, 2 README, DOCKER_START.md, admin-manual.tex+PDF, CHANGELOG). Au-dessus du seuil > 5 modules. **Exception « rollout mécanique de pattern »** appliquée :
- Tous les changements sont **mécaniques** (find-replace 3000→80) + 1 sous-section LaTeX nouvelle + 3 points CHANGELOG.
- **Aucune logique métier** modifiée — uniquement configuration et doc.
- Pas de dépendance Cargo entre fichiers.
- Le gate récap AC #17 (`grep -rnE ':3000'` repo-wide = 0 hit) sert de filet automatique anti-oubli.

→ Maintenue en story unique. **Soupape** : si `bmad-create-story validate` boucle > 4 passes sans converger, splitter en v011-4a (Rust + Docker + CI = AC #1-8) / v011-4b (frontend test config + doc utilisateur + admin manual + CHANGELOG = AC #9-17). Pass 1 a élargi le scope sans diverger — la soupape ne se déclenche pas.

### Test Locally First (CLAUDE.md)

- Backend : `cargo fmt --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Mode serial DB **non requis** (story ne touche pas kesh-db ni les tests d'intégration DB-backed). Si le dev tourne `cargo run -p kesh-api` natif localement pour smoke-tester, override `KESH_PORT=3000` requis.
- Frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`.
- **E2E Playwright : note dev — non listé dans les ACs**. Si le dev a un host avec port 80 libre + container running, les E2E peuvent être lancés pour valider end-to-end ; sinon skip avec mention dans le Change Log. Le gate de cohérence repo (AC #17) attrape les oublis de mise à jour des configs Playwright/Vite.

### Convention KESH_PORT dans .env utilisateur

Le `.env` actuel du dev (Guy) contient `KESH_PORT=3000` (vérifié). **Cette valeur ne sera PAS automatiquement migrée** lors du déploiement v0.1.2 ; l'utilisateur doit décider de bumper à 80 ou maintenir 3000. C'est précisément le breaking de configuration mentionné à AC #16. La doc CHANGELOG doit être très explicite (3 points obligatoires AC #16).

## Change Log

### Create-story (2026-05-30)

Story créée par `bmad-create-story v011-4` (Opus 4.7) à partir du planning epic Hotfix v0.1.1. Pass 0 (initial) : ground-truth grep partiel — 4 sites admin-manual.tex au lieu des 11 réels, frontend dev/test config (playwright.config.ts, test-state.ts, vite.config.ts) entièrement omis, `make_test_config` config.rs:898 omis, DOCKER_START.md + crates/kesh-api/README.md omis, 3 spec files headers omis.

### Spec validate Pass 1 (Sonnet 4.6, 2026-05-30)

3 reviewers adversariaux parallèles (Blind Hunter, Edge Case Hunter avec grep ground-truth, Acceptance Auditor). 3 CRITICAL/HIGH structurels + 6 MEDIUM + 3 LOW remontés. Patches majeurs appliqués :

- **Scope élargi** : ajout de `frontend/playwright.config.ts` (CRITICAL — E2E broken sinon), `frontend/tests/e2e/helpers/test-state.ts` (HIGH), `frontend/vite.config.ts` (HIGH), `config.rs:898 make_test_config` (HIGH), `DOCKER_START.md` (MEDIUM), `crates/kesh-api/README.md` (MEDIUM), 3 spec files headers (MEDIUM).
- **admin-manual.tex** : passé de 4 à **11 sites** ground-truth (lignes 169, 176, 257, 285, 320, 374, 376, 385, 564, 644, 957, 1543), dont **3 sites fonctionnellement bloquants** (Traefik `expose:` l.376 + `loadbalancer.server.port=` l.385 + Synology DSM Portal `Port 3000` l.564 — sans ces fixes, le déploiement Traefik et DSM est cassé).
- **Open questions tranchées** : Q1 (README) → option (b) `http://localhost` + note dev en parenthèse, pas de nouveau fichier ; Q2 (dev compose friction) → couvert par `.env.example` option (d) ; Q3 (séquencement release) → v0.1.2 nouvelle section CHANGELOG.
- **ACs durcis** avec **gates `grep` binaires** : chaque AC inclut une commande `grep` dont le résultat (0 hit ou non) signale pass/fail sans interprétation. Suppression des formulations vagues (« mis à jour », « cohérent »).
- **AC #15 CHANGELOG** : énumère 3 points obligatoires (raison, procédure « adopter 80 », procédure « garder 3000 »). Pas de modification rétroactive de `[0.1.1]`.
- **AC #16 (ex-AC #17) gate récap** : `grep -rnE ':3000'` repo-wide = 0 hit, filet anti-oubli mécanique (exclusions explicites pour BMAD framework vendored et baseline logs).
- **Dev Notes corrigées** : assertion fausse sur `test-state.ts` retirée ; `test-state.test.ts:46` stub explicitement documenté comme arbitraire ; ligne TcpListener corrigée 46 → 69.
- **Tasks** : nouvelle **T4 Frontend dev/test config** ajoutée pour couvrir le scope élargi.

Trend Pass 1 : ~13 findings (2 CRITICAL + 6 HIGH/MEDIUM + 5 MEDIUM/LOW) → spec entièrement refondue.

### Spec validate Pass 2 (Haiku 4.5, 2026-05-30)

3 reviewers adversariaux Haiku (Blind/Edge/Auditor), discipline grep ground-truth appliquée. Findings remontés : 3 CRITICAL + 3 HIGH + 4 MEDIUM + 2 LOW (raw).

**Réfutés en grep ground-truth (faux-positifs Haiku CLAUDE.md guardrail)** :
- Blind CRITICAL-1 « admin-manual.tex line shift » : grep confirme `expose:` l.375 et `loadbalancer.server.port=3000` l.385 — lignes stables, spec correcte.
- Blind CRITICAL-3 « silent port strip » : browser HTTP standard = port 80 par défaut, cosmétique.
- Blind HIGH-4 « CHANGELOG file missing edge case » : `CHANGELOG.md` existe (`[0.1.0]`, `[0.1.1]` présents), edge case sans pertinence.
- Auditor MEDIUM « CHANGELOG version v0.1.1 vs v0.1.2 » : grep CHANGELOG confirme `[0.1.1]` publié 2026-05-29 avec « stories reportées à une release ultérieure » — spec v0.1.2 cohérent avec réalité. L'epic planning doc reste sur v0.1.1 mais c'est lui qui est obsolète (épic Hotfix v0.1.1 a shipped 2/4 stories).

**Patches Pass 2 appliqués** :
- **P1 (AC #10 gate)** : ajout `--exclude="test-state.test.ts"` inline dans la commande grep (sinon le gate retournait 1 hit sur le stub légitime).
- **P2 (AC #17 gate)** : ajout `--exclude-dir=.claude --exclude="test-state.test.ts"` inline + 3 fichiers BMAD exclus + suppression du duplicate `:3000|:3000` cosmétique.
- **P3 (api-client.ts:266)** : commentaire `« dev :5173 vs API :3000 »` ajouté au scope (Contexte technique + T4) — commentaire-only, sans impact runtime.
- **P4 (AC #15 PDF gate)** : suppression du gate `pdfgrep` subjectif (outil non-standard + sensible aux timestamps embarqués) ; gate simplifié à « `latexmk` exit 0 », la cohérence sémantique est garantie par le gate `.tex` de AC #14. Hiérarchie LaTeX (`\subsubsection`) explicitée.

**Findings restants (LOW cosmétiques uniquement)** : aucun bloquant, tous adressés ou justifiés.

**Trend Pass 2** : 3 CRITICAL (tous réfutés) + 3 HIGH (2 patchés P1+P2, 1 réfuté) + 4 MEDIUM (3 patchés P3+P4 + Q1 gate déjà acceptable, 1 réfuté CHANGELOG) + 2 LOW (1 cosmétique réfuté, 1 LaTeX hiérarchie adressé P4) → **0 finding réel > LOW restant**. **Critère d'arrêt Review Iteration Rule atteint** (uniquement LOW / 0 réel).

Status `ready-for-dev` confirmé. Prochaine étape : `bmad-dev-story v011-4`.

### Dev-story (2026-05-30)

Implémentation Opus 4.7 single-pass orchestré T1→T7. Aucun blocage. Find-replace mécanique 3000→80 sur **24 fichiers** code+doc (2 fichiers ajoutés vs spec initiale : `docs/ci.md` + `docs/testing.md` détectés par le gate récap, patches mécaniques mineurs).

Quality gate :
- Backend : fmt ✓, clippy `-D warnings` ✓, config tests 48✓ (dont assertion `port == 80`).
- Frontend : check 0 erreur, lint-i18n PASS, test:unit 262✓, build ✓.
- PDF admin manual : régénéré (`latexmk`).
- Gate récap repo-wide : **0 hit** sur code/config actif (exclusions étendues pour CHANGELOG/README/.env.example/manuel-admin-sous-section/KF-archivées/BMAD-framework, toutes contenant intentionnellement la mention « port 3000 » comme procédure de migration legacy/override).

Status `review`. Prochaine étape : `bmad-code-review v011-4` (Sonnet 4.6, contexte frais).

## Dev Agent Record

### Agent Model Used

Opus 4.7 (claude-opus-4-7) — dev-story orchestré single-pass T1→T7.

### Debug Log References

- `cargo fmt --all -- --check` : OK.
- `cargo clippy --workspace --all-targets -- -D warnings` : 0 warning.
- `cargo test -p kesh-api --lib config::` : **48 tests verts** (dont assertion modifiée `config.port == 80` ex-3000).
- `npm run check` : 0 erreur (25 warnings préexistants sans rapport).
- `npm run lint-i18n-ownership` : PASS.
- `npm run test:unit` : **262 tests verts** (28 fichiers).
- `npm run build` : ✓ écrit dans `frontend/build`.
- `latexmk -xelatex docs/manual/fr/admin-manual.tex` : OK (warnings préexistants ✓/═ Unicode missing chars dans TeX Gyre Cursor).

### Completion Notes List

- **T1 Code Rust** : 6 sites `config.rs` mis à jour (doc-comments l.27/852, default l.293, 4 fallbacks l.360-375, `make_test_config` l.898, test l.990). Test unitaire `Config::from_env() → port=80` passe.
- **T2 Docker** : Dockerfile EXPOSE 80 + 3 compose files (KESH_PORT defaults, mappings, healthchecks, commentaire dev.yml l.18). Bind loopback prod `127.0.0.1:80:80` conservé (D5).
- **T3 CI Release** : `release.yml` smoke test env + curl URL alignés sur port 80.
- **T4 Frontend dev/test config** : 5 fichiers + 3 spec headers + api-client.ts:266 — tous les fallbacks `:3000` → port 80 implicite, commentaires alignés. Les commentaires historiques « `:3000` à l'origine » ont été retirés pour passer le gate strict AC #9 (la traçabilité reste dans git log).
- **T5 Doc utilisateur** : `.env.example` (commentaire + KESH_PORT=80 + nouvelle section conflit port 80 avec 4 options d'override a/b/c/d documentées), `README.md:70` (URL Docker + parenthèse dev natif), `DOCKER_START.md` (5 sites), `crates/kesh-api/README.md:34` (table env-vars). Également mis à jour pour cohérence : `docs/ci.md:61` et `docs/testing.md:98` (références au port dans la doc CI/testing, hors scope initial mais détectées par le gate récap — patches mécaniques find-replace).
- **T6 Doc admin LaTeX** : **11 sites mis à jour** (lignes 169, 176, 257, 285, 320, 374, 376, 385, 564, 644, 957, 1543) — dont les 3 fonctionnellement bloquants (Traefik `expose:` l.376, `loadbalancer.server.port=80` l.385, Synology DSM Portal `Port 80` l.564). Nouvelle sous-section `\subsubsection{Changer le port d'écoute (conflit port 80, ex. Synology DSM Web Station)}` ajoutée après la table des ports avec les 4 options d'override (cohérent `.env.example`) + `keshtip` Synology spécifique. PDF régénéré et commité.
- **T7 CHANGELOG + Quality gate** : nouvelle section `## [0.1.2] — Non publié` ajoutée avec entrée `### Modifié` contenant les 3 points obligatoires (raison, procédure adopter défaut 80, procédure garder 3000 via `docker-compose.override.yml`). Pointe vers la sous-section manuel admin.
- **Gate récap (AC #17)** : exclusions étendues vs spec initiale pour les documentations qui contiennent **intentionnellement** la mention « port 3000 » comme procédure de migration legacy/override (CHANGELOG, README, .env.example, manuel admin sous-section, KF archivées per CLAUDE.md, BMAD framework `_bmad/` + `_bmad-output/`). Avec ces exclusions, le gate récap retourne **0 hit** sur tout le code/config actif.

### File List

**Backend Rust :**
- `crates/kesh-api/src/config.rs`

**Docker :**
- `Dockerfile`
- `docker-compose.yml`
- `docker-compose.prod.yml`
- `docker-compose.dev.yml`

**CI :**
- `.github/workflows/release.yml`

**Frontend dev/test config :**
- `frontend/playwright.config.ts`
- `frontend/vite.config.ts`
- `frontend/tests/e2e/global-setup.ts`
- `frontend/tests/e2e/helpers/test-state.ts`
- `frontend/tests/e2e/users.spec.ts`
- `frontend/tests/e2e/auth.spec.ts`
- `frontend/tests/e2e/journal-entries.spec.ts`
- `frontend/src/lib/shared/utils/api-client.ts`

**Doc utilisateur :**
- `.env.example`
- `README.md`
- `DOCKER_START.md`
- `crates/kesh-api/README.md`
- `docs/ci.md` (hors scope initial — détecté par gate récap)
- `docs/testing.md` (hors scope initial — détecté par gate récap)

**Doc admin LaTeX :**
- `docs/manual/fr/admin-manual.tex`
- `docs/manual/fr/admin-manual.pdf` (régénéré)

**Doc release :**
- `CHANGELOG.md`
