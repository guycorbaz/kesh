# Story v011.4: Port applicatif par défaut 80 (Issue #118)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **nouvel utilisateur qui installe Kesh v0.1.x sur un host vide**,
I want que **l'URL standard `http://kesh.local`** (sans `:3000`) fonctionne directement après `docker compose up`,
so that je puisse accéder à l'app sans surprise sur le port et que la doc d'install reflète l'URL HTTP standard que les utilisateurs connaissent.

## Scope

**Severity : amélioration qualité install (post-v0.1.1).** Décidée 2026-05-28 (révision décision H6 epic Hotfix v0.1.1). Guy veut éviter les soucis de port — surtout pour les **autres utilisateurs** qui installeront Kesh — URL HTTP standard sans `:3000`.

**Décision séquencement (2026-05-30) :** v011-3 break-glass a été superseded par v011-5 onboarding self-service unifié (cible v0.1.2). v011-4 reste seule story éligible pour shipping v0.1.x rapide.

**Dans le scope :**
- `crates/kesh-api/src/config.rs` : défaut `port: u16 = 80` (était 3000) + 4 fallbacks d'erreur + doc-comments + test assertion.
- `Dockerfile` : `EXPOSE 80` (était 3000).
- `docker-compose.{yml,prod.yml,dev.yml}` : `KESH_PORT` + mappings + healthchecks alignés sur 80 (bind loopback prod `127.0.0.1` conservé D5).
- `.github/workflows/release.yml` : smoke test `docker run` + `curl` sur port 80 (était 3000).
- `.env.example` : commentaire `KESH_PORT` + nouvelle procédure d'override en cas de conflit port 80 (Synology DSM Web Station, etc.) avec exemples (`KESH_PORT=3000`, `3000:80` mapping host, IP dédiée).
- `README.md` quickstart : URL `http://localhost/...` (sans `:3000`) pour le mode Docker.
- `frontend/tests/e2e/global-setup.ts` : fallback `KESH_BACKEND_URL` aligné sur 80 (était 3000).
- `docs/manual/fr/admin-manual.tex` : table des ports + warnings + exemples `curl` + nginx `proxy_pass` mis à jour ; nouvelle sous-section « Changer le port d'écoute (conflit port 80, ex. Synology DSM) ». PDF régénéré.
- `CHANGELOG.md` `[0.1.1]` (ou nouvelle section v0.1.x si tag séparé) : entrée `Changed` documente le **breaking de configuration** (utilisateurs v0.1.0/v0.1.1 doivent accepter le défaut 80 OU setter `KESH_PORT=3000` dans leur `.env`/mapping).

**Hors scope :**
- Le **mapping host** reste au choix de l'utilisateur (`80:80`, `3000:80`, `IP_dédiée:80`, etc.). v011-4 ne touche pas à comment l'utilisateur expose le container côté host.
- Reverse proxy HTTPS (nginx/Caddy/Traefik) — la doc existante de l'admin-manual sera mise à jour pour pointer `proxy_pass http://localhost` au lieu de `:3000`, mais aucune nouvelle config TLS ajoutée.
- Migration `cargo run` natif (mode dev hors Docker) : un dev qui lance natif sur Linux non-root ne peut PAS bind `:80` → doit setter `KESH_PORT=3000` (ou ≥ 1024) dans son env. Documenté dans `.env.example` mais pas de fix automatique.
- Le port MariaDB (3306) reste inchangé.

## Contexte technique (ground-truth 2026-05-30)

Toutes les références à port 3000 dans le code et la doc à corriger. Vérifié `grep -nE ":3000|3000/|KESH_PORT|EXPOSE 3000" ...` sur la branche `main` au commit `7c0ace6`.

### Code Rust

- `crates/kesh-api/src/config.rs:27` — doc-comment de header : « `-p 3000:3000` avec bind interne `0.0.0.0` expose la route au public ». Remplacer par `80:80` (cohérent avec la nouvelle default).
- `crates/kesh-api/src/config.rs:293` — initial default dans `Config::default()` (struct littéral) : `port: 3000` → `port: 80`.
- `crates/kesh-api/src/config.rs:360-375` — parsing `KESH_PORT` runtime avec **4 fallbacks** vers 3000 sur erreur (port=0, parse fail, var absente) → tous à `80`. Les messages `tracing::warn!` mentionnent « utilisation du port par défaut 3000 » → « utilisation du port par défaut 80 ».
- `crates/kesh-api/src/config.rs:852` — doc-comment de validation host : `« en Docker -p 3000:3000 »` → `« en Docker -p 80:80 »`.
- `crates/kesh-api/src/config.rs:990` — test unitaire `assert_eq!(config.port, 3000)` → `80`.

### Docker

- `Dockerfile:28` — `EXPOSE 3000` → `EXPOSE 80`. Le container tourne en **root** (pas de `USER` dans Dockerfile, vérifié) → bind `:80` OK sans `CAP_NET_BIND_SERVICE`.
- `docker-compose.yml:44,65,72` — `KESH_PORT: ${KESH_PORT:-3000}` → `:-80`, mapping `"3000:3000"` → `"80:80"`, healthcheck `http://localhost:3000/health` → `http://localhost/health`.
- `docker-compose.prod.yml:24,49,64,109,118` — commentaires d'exposition (lignes 24, 49), `KESH_PORT: ${KESH_PORT:-3000}` → `:-80`, mapping `"127.0.0.1:3000:3000"` → `"127.0.0.1:80:80"` (bind loopback **conservé** D5), healthcheck `http://localhost:3000/health` → `http://localhost/health`.
- `docker-compose.dev.yml:13,16,18-19,43` — mapping `"127.0.0.1:3000:3000"` → `"127.0.0.1:80:80"`, `KESH_PORT: "3000"` → `"80"`, commentaires explicatifs lignes 18-19, healthcheck `http://localhost:3000/health` → `http://localhost/health`. ⚠️ Cohérent avec dev container-based ; ne pas confondre avec `cargo run` natif (cf. Dev Notes).

### CI/Release

- `.github/workflows/release.yml:96` — env `-e KESH_PORT="3000"` (dans `docker run` du smoke test) → `KESH_PORT="80"` + mapping `--network=host` ne change pas mais le `curl` cible change.
- `.github/workflows/release.yml:105` — `curl -fsS http://127.0.0.1:3000/health` → `http://127.0.0.1/health` (port 80 implicite).

### Doc utilisateur

- `.env.example:24-25` — commentaire `# Port HTTP du serveur kesh-api (défaut 3000).` → `# Port HTTP du serveur kesh-api (défaut 80).` + `KESH_PORT=3000` → `KESH_PORT=80` (commenté en exemple). Ajouter section « Conflit port 80 (Synology DSM Web Station / autre service) » avec 3 options d'override commentées : (a) `KESH_PORT=3000` dans `.env` + mapping `3000:3000` compose, (b) mapping host différent dans compose (ex. `8080:80`), (c) IP dédiée container `172.x.x.x:80:80`.
- `README.md:70` — « `http://localhost:5173 (frontend dev) et http://localhost:3000 (API)` » → décider Q1 ci-dessous.
- `frontend/tests/e2e/global-setup.ts:37` — fallback `KESH_BACKEND_URL ?? 'http://127.0.0.1:3000'` → `'http://127.0.0.1'` (port 80 implicite). Les tests E2E déjà existants sont env-driven (`KESH_BACKEND_URL`) donc l'override CI/local fonctionne.

### Doc admin (manuel LaTeX FR)

- `docs/manual/fr/admin-manual.tex:169` — table « Ports utilisés » : ligne `3000 & HTTP & API Axum backend (interne) & Localhost uniquement` → `80 & HTTP & API Axum backend (interne) & Localhost uniquement`.
- `docs/manual/fr/admin-manual.tex:176` — warning : « Ne jamais exposer le port 3000 (API Axum)... reverse proxy HTTPS qui termine TLS et fait suivre vers `localhost:3000` » → mise à jour de tous les `3000` → `80`.
- `docs/manual/fr/admin-manual.tex:257` — exemple `curl http://localhost:3000/health` → `http://localhost/health`.
- `docs/manual/fr/admin-manual.tex:285` — nginx `proxy_pass http://localhost:3000` → `http://localhost`.
- **Nouvelle sous-section** « Changer le port d'écoute (conflit port 80, ex. Synology DSM) » : explique les 3 options d'override (`.env`, mapping compose, IP dédiée). Place naturelle : juste après la table des ports (l.169) ou dans la section déploiement Synology DSM.
- **PDF régénéré** (`latexmk -xelatex`) cohérent avec `.tex` (artefacts `.aux/.toc/...` non commités).

### CHANGELOG

- `CHANGELOG.md` `[0.1.1]` (ou nouveau bloc v0.1.x selon décision séquencement release) : entrée **`### Changed`** intitulée « Port d'écoute par défaut : 3000 → 80 ». Explique pourquoi (URL HTTP standard sans `:3000`, simplifie install). Note explicitement le **breaking de configuration** : utilisateurs existants doivent (a) accepter le défaut 80 (mettre à jour `.env`/mapping si custom), OU (b) garder 3000 via `KESH_PORT=3000` + mapping. Pointer vers la nouvelle sous-section du manuel admin pour la procédure d'override.

## Acceptance Criteria

### Code Rust (AC #1-3)

- [ ] **AC #1** `crates/kesh-api/src/config.rs:293` : default `port: 80` dans `Config::default()`. Doc-comments lignes 27 + 852 mis à jour (3000→80).
- [ ] **AC #2** `crates/kesh-api/src/config.rs:360-375` : les 4 fallbacks de parsing `KESH_PORT` (port=0, parse fail, var absente) retournent **80** ; les messages `tracing::warn!` mentionnent « port par défaut 80 ».
- [ ] **AC #3** `crates/kesh-api/src/config.rs:990` : `assert_eq!(config.port, 80)` (était 3000) + tout autre test qui assert le default port mis à jour.

### Docker (AC #4-7)

- [ ] **AC #4** `Dockerfile:28` : `EXPOSE 80` (était 3000). Vérifier qu'aucun `USER` non-root n'a été ajouté entre-temps (sinon ajouter `CAP_NET_BIND_SERVICE` ou revenir sur la décision).
- [ ] **AC #5** `docker-compose.yml` : `KESH_PORT: ${KESH_PORT:-80}`, mapping `"80:80"`, healthcheck `http://localhost/health`.
- [ ] **AC #6** `docker-compose.prod.yml` : `KESH_PORT: ${KESH_PORT:-80}`, mapping `"127.0.0.1:80:80"` (bind loopback D5 conservé), healthcheck `http://localhost/health`. Commentaires d'exposition lignes 24 + 49 mis à jour.
- [ ] **AC #7** `docker-compose.dev.yml` : `KESH_PORT: "80"`, mapping `"127.0.0.1:80:80"`, healthcheck `http://localhost/health`. Commentaires explicatifs (l.18-19) cohérents avec la nouvelle valeur.

### CI/Release (AC #8-9)

- [ ] **AC #8** `.github/workflows/release.yml:96` : `-e KESH_PORT="80"` (smoke test).
- [ ] **AC #9** `.github/workflows/release.yml:105` : `curl -fsS http://127.0.0.1/health` (sans `:3000`).

### Doc utilisateur (AC #10-13)

- [ ] **AC #10** `.env.example:24-25` : commentaire + variable cohérents avec défaut 80. **Nouvelle section** « Conflit port 80 (Synology DSM / autre service) » documente les 3 options d'override avec exemples concrets.
- [ ] **AC #11** `README.md:70` : URL API mise à jour selon décision Q1 (proposition : `http://localhost` pour mode Docker, mention native dev séparée). Aucune autre URL `:3000` dans le README.
- [ ] **AC #12** `frontend/tests/e2e/global-setup.ts:37` : fallback `KESH_BACKEND_URL ?? 'http://127.0.0.1'` (port 80 implicite). Les specs E2E continuent de respecter `KESH_BACKEND_URL` (env-driven, pas de port hardcodé dans les `.spec.ts`).

### Doc admin LaTeX (AC #13-14)

- [ ] **AC #13** `docs/manual/fr/admin-manual.tex` : 4 références `3000` mises à jour (table ports l.169, warning l.176, exemple curl l.257, nginx proxy_pass l.285). PDF régénéré (`latexmk -xelatex`).
- [ ] **AC #14** Nouvelle sous-section « Changer le port d'écoute (conflit port 80, ex. Synology DSM) » ajoutée au manuel admin avec les 3 options d'override + exemples concrets. PDF régénéré.

### Quality gate (AC #15-16)

- [ ] **AC #15** `CHANGELOG.md` entrée `### Changed` documente le breaking de configuration (port 3000 → 80) avec procédure de maintien du port 3000 pour les utilisateurs existants. Pointer vers la sous-section manuel admin (AC #14).
- [ ] **AC #16** Série Test Locally First complète verte (cargo fmt + clippy `-D warnings` + build + workspace tests + npm check/lint-i18n/test:unit + build). E2E Playwright non-bloquant mais à vérifier vert si Backend tournant sur port 80 (peut nécessiter `sudo` ou `KESH_PORT=3000` selon machine du dev).

## Tasks / Subtasks

- [ ] **T1 — Code Rust** (AC #1-3)
  - [ ] `config.rs` defaults (l.293, fallbacks l.360-375, doc-comments l.27/852).
  - [ ] `config.rs` test assertion (l.990) — vérifier qu'aucun autre test n'assert le port défaut.
- [ ] **T2 — Dockerfile + Compose** (AC #4-7)
  - [ ] `Dockerfile` EXPOSE.
  - [ ] 3 compose files (KESH_PORT default, mappings, healthchecks).
  - [ ] Bind loopback prod `127.0.0.1` conservé (revue critique).
- [ ] **T3 — CI Release** (AC #8-9)
  - [ ] `release.yml` smoke test env + curl.
- [ ] **T4 — Doc utilisateur** (AC #10-12)
  - [ ] `.env.example` (commentaire + variable + section override).
  - [ ] `README.md` quickstart (selon décision Q1).
  - [ ] `frontend/tests/e2e/global-setup.ts` fallback.
- [ ] **T5 — Doc admin LaTeX** (AC #13-14)
  - [ ] 4 références dans `admin-manual.tex`.
  - [ ] Nouvelle sous-section « Changer le port d'écoute ».
  - [ ] `latexmk -xelatex` régénère le PDF.
- [ ] **T6 — CHANGELOG + Quality gate** (AC #15-16)
  - [ ] Entrée `Changed` avec procédure d'override.
  - [ ] Test Locally First complète.

## Dev Notes

### Patterns à respecter (ground-truth code)

- **Container tourne en root** : `Dockerfile` ne contient pas de `USER` directive (vérifié l.1-29). Bind `:80` (port privilégié <1024) OK sans `CAP_NET_BIND_SERVICE` ni `setcap`. Si une future Story 10-x introduit un `USER kesh` non-root, il faudra ré-évaluer (typiquement : ajout de `CAP_NET_BIND_SERVICE` au container OU revenir sur un port >1024).
- **Bind loopback prod** (`docker-compose.prod.yml:109` `127.0.0.1:80:80`) : conservation **non-négociable** — la décision D5 du déploiement Synology DSM impose loopback strict pour forcer le reverse proxy HTTPS (cf. admin-manual.tex section « Reverse proxy nginx »).
- **`cargo run` natif (mode dev hors Docker)** : Linux non-root **ne peut PAS bind <1024**. Si un dev tourne `cargo run -p kesh-api` directement (workflow Test Locally First backend), il doit setter `KESH_PORT=3000` (ou ≥ 1024) dans son `.env` local. À documenter dans `.env.example` (« si vous lancez kesh-api en natif sur Linux non-root, override `KESH_PORT=3000` »).

### Références test stable

- **Tests d'intégration** `crates/kesh-api/tests/*_e2e.rs` : utilisent `tokio::net::TcpListener::bind("127.0.0.1:0")` (port éphémère choisi par l'OS — vérifié `onboarding_path_b_e2e.rs:46`). **Pas impacté** par le changement de defaut.
- **Tests Playwright** `frontend/tests/e2e/*.spec.ts` : utilisent `KESH_BACKEND_URL` (env-driven, fallback dans global-setup.ts). Pas de port hardcodé dans les specs. Le fallback à mettre à jour (AC #12) couvre le cas où la var n'est pas settée.
- **`baseline-pre-9-5-1b.log`** (et autres `*.log` dans `frontend/tests/e2e/`) : logs historiques, pas des tests actifs. Contiennent des références `:3000` (historique). **Ne pas modifier** — ce sont des archives.

### Q1 — README dev context (à trancher en spec validate)

Ligne 70 actuelle : « L'application est accessible sur http://localhost:5173 (frontend dev) et http://localhost:3000 (API). »

Options :
- **(a)** Conserver tel quel (le dev mode décrit est `cargo run` natif où KESH_PORT=3000 reste pratique). Simple, mais le défaut config.rs (80) diverge du README.
- **(b)** Mettre à jour à « http://localhost (API en mode Docker) ; en mode dev natif `cargo run`, lancer avec `KESH_PORT=3000` sur Linux non-root puis `http://localhost:3000` ». Précis mais verbeux.
- **(c)** Mettre à jour à « http://localhost » et déplacer la note dev mode dans `CONTRIBUTING.md` ou `docs/testing.md`. Plus propre.

Proposition : **(c)**.

### Q2 — Friction `docker-compose.dev.yml` port 80 (cf. epic Q7)

Si le dev utilise `docker compose -f docker-compose.dev.yml up`, le container bind `:80` et map `127.0.0.1:80:80`. **Sur le host**, port 80 doit être libre — souvent occupé par un service local (nginx perso, etc.). Le dev devra setter `KESH_PORT=8080` (ou autre) dans son `.env` pour éviter conflit.

Documenter clairement dans `.env.example` qu'il s'agit du **port côté container**, l'utilisateur peut remapper côté host via le `ports:` du compose (`HOST_PORT:80`).

### Q3 — Séquencement release

v011-4 peut être livré dans :
- **(a) v0.1.2 hotfix UX** combiné avec v011-5 onboarding self-service (cible probable). Cohérent : 2 améliorations UX install ensemble.
- **(b) v0.1.1.1 patch micro** isolé (tag séparé) pour pousser le port 80 rapidement. Sur-ingénierie probable pour 0.5j de doc.
- **(c) v0.2 release majeure** — port 80 considéré comme un changement de paramètre d'install. Délai trop long.

Proposition : **(a)** v0.1.2 avec v011-5.

### Migration breaking policy (CLAUDE.md)

v011-4 ne touche **aucune migration** (pas de schema change). La politique P3 (DROP/RENAME COLUMN sans bump min_required) ne s'applique pas. Aucun audit `docs/migrations-idempotence-audit.md` à ajouter.

### Règle de splitting préventif (CLAUDE.md)

Cette story touche **~10 fichiers** (config.rs ×1, Dockerfile ×1, 3 compose ×3, release.yml ×1, .env.example ×1, README ×1, global-setup.ts ×1, admin-manual.tex+PDF ×1, CHANGELOG ×1). Seuil > 5 modules dépassé (CLAUDE.md règle de splitting), mais :
- **Tous les changements sont mécaniques** (find-replace 3000→80) + 1 sous-section LaTeX nouvelle.
- **Aucune logique métier** modifiée — uniquement de la configuration et de la doc.
- **Pas de dépendance Cargo** entre fichiers.

→ Maintenue en story unique conforme à l'exception « rollout mécanique de pattern » de la règle de splitting. Si `bmad-create-story validate` boucle > 4 passes, splitter en v011-4a (config Rust + Docker + CI) / v011-4b (doc admin + .env + README + CHANGELOG).

### Test Locally First (CLAUDE.md)

- Backend : `cargo fmt --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Pas besoin du mode serial DB (story ne touche pas kesh-db). Si dev tourne `cargo run -p kesh-api` natif pour smoke-tester localement, override `KESH_PORT=3000` requis.
- Frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`.
- E2E : conditionnel — si le dev a un host avec port 80 libre + container running, sinon skip avec mention dans le Change Log.

### Convention KESH_PORT dans .env utilisateur

Le `.env` actuel du dev (Guy) contient `KESH_PORT=3000` (vérifié). **Cette valeur ne sera PAS automatiquement migrée** lors du déploiement v0.1.2 ; l'utilisateur doit décider de bumper à 80 ou maintenir 3000. C'est précisément le breaking de configuration mentionné à AC #15. La doc CHANGELOG doit être très explicite : « Si vous gardez votre `.env` existant avec `KESH_PORT=3000`, vous devez aussi conserver le mapping `3000:3000` (compose) ou utiliser un mapping explicite `HOST_PORT:3000`. Sinon, retirez `KESH_PORT=3000` du `.env` pour adopter le défaut 80. »

## Change Log

### Create-story (2026-05-30)

Story créée par `bmad-create-story v011-4` (Opus 4.7) à partir du planning epic Hotfix v0.1.1. Analyse ground-truth exhaustive : `grep -nE ":3000|3000/|KESH_PORT|EXPOSE 3000"` sur la branche `main` au commit `7c0ace6` (post-merge PR #131 v011-5 backlog). 15-18 sites identifiés avec numéros de lignes exactes. v011-3 break-glass superseded par v011-5 → v011-4 reste la story éligible pour shipping v0.1.x rapide.

3 questions ouvertes (Q1 README dev context, Q2 friction dev compose port 80, Q3 séquencement release) à trancher en spec validate.

Status `ready-for-dev`. Prochaine étape : `bmad-create-story validate v011-4` (boucle Sonnet → Haiku → ... jusqu'à 0 > LOW) puis `bmad-dev-story v011-4`.

## Dev Agent Record

### Agent Model Used

_(à remplir au dev-story)_

### Debug Log References

_(à remplir au dev-story)_

### Completion Notes List

_(à remplir au dev-story)_

### File List

_(à remplir au dev-story)_
