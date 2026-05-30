---
epic: hotfix-v0.1.1
title: "Hotfix v0.1.1 — Onboarding fresh-install + admin recovery + logs fichier"
version: v0.1.1
status: planning
sourceArtifact: Découvertes prod 2026-05-27 lors du 1er déploiement v0.1.0 sur NAS Synology Guy
relatedFRs:
  - FR1 (install < 15 min via docker-compose) — bug bloquant catch-22 viole cette exigence en pratique
  - FR3 (admin initial via env) — recovery break-glass complète le cycle de vie de l'admin
  - FR78 (backup recommandé) — logs fichier alignent les logs au backup Hyper Backup
relatedDecisions:
  - "Pas de workaround SQL en prod, attendre v0.1.1 propre (Guy 2026-05-27)"
  - "Onboarding fix Option A : bootstrap stub company + admin du .env (Guy 2026-05-27)"
  - "Break-glass via KESH_ADMIN_RESET=true (quick fix recovery offline, complète .env-driven bootstrap)"
  - "Recovery production-grade (forgot-password SMTP, 2FA, lockout) reportée v0.2+ via Issue #122"
  - "Logs fichier en ./log/ relatif au cwd docker-compose (cohérent Hyper Backup scope unique)"
  - "Workflow full BMAD pour les 4 stories (qualité Epic 10 maintenue)"
  - "Port défaut 80 (Issue #118) AJOUTÉ à v0.1.1 (Guy 2026-05-28) — qualité install pour les autres utilisateurs ; le mapping host reste au choix de l'utilisateur (ex. 3000:80, IP dédiée container)"
crates:
  - kesh-api (bootstrap stub company + admin, break-glass reset, tracing-appender layer fichier, port défaut 80 config.rs)
  - kesh-db (audit_log event break_glass_reset)
  - infra (docker-compose.{prod,dev,base}.yml port 80 + mount ./log/, Dockerfile EXPOSE 80, .env.example sections nouvelles)
  - docs (manuel admin sections « J'ai oublié mon mot de passe » + « Configuration logs fichier » + « Changer le port d'écoute » + PDF régénéré)
  - .github/workflows (release.yml smoke test sur port 80, ci.yml inchangé)
stories:
  - v011-1-file-logs-rotation
  - v011-2-fix-catch22-onboarding-fresh-install
  - v011-3-break-glass-admin-reset
  - v011-4-default-port-80
---

# Epic Hotfix v0.1.1 — Onboarding fresh-install + admin recovery + logs fichier + port 80

## Vue d'ensemble

**Contexte déclencheur :** Le **premier déploiement prod v0.1.0** sur NAS Synology de Guy (2026-05-27) a découvert un bug bloquant (#120) qui empêche TOUT nouvel utilisateur de compléter l'install. Bug catégorie A « hors fenêtre rétrospective » → traitement immédiat via une release hotfix v0.1.1 (cf. CLAUDE.md règle « tech debt management — découverte hors rétrospective »).

**Objectif :** Livrer une image Docker `gcorbaz/kesh:v0.1.1` qui corrige le bug d'install + ajoute deux features de qualité de vie ops (recovery admin break-glass + logs fichier accessibles hors `docker logs`) découvertes en même temps que le bug bloquant.

**Périmètre :** 4 stories. Aucune feature comptable nouvelle (pure ops). Toute la base v0.1.0 reste utilisable telle quelle après upgrade — pas de migration utilisateur DB breaking (le changement de port par défaut est un breaking *de configuration*, documenté CHANGELOG, pas un breaking DB → pas de bump `kesh_version_min_required`, cf. H8).

**Hors scope v0.1.1 :**
- Recovery password production-grade (forgot-password email, 2FA, lockout policies) → reporté v0.2+ via Issue #122 (large scope, demande PRD dédiée + infra SMTP).
- TVA Suisse (= Epic 11, première vraie feature v0.2) → kickoff Epic 11 conditionné à v0.1.1 mergée + Guy capable d'utiliser sa prod NAS sans bug bloquant.

**Provenance :**
- Issue GitHub #120 (catch-22 onboarding, cat A bloquant) — découverte 2026-05-27 lors de l'install Guy
- Issue GitHub #121 (break-glass via .env) — gap recovery identifié pendant le debug #120
- Issue GitHub #119 (logs fichier rotation) — demande Guy pour debugging facilité

**Dépendances amont :**
- v0.1.0 publié sur Docker Hub (`gcorbaz/kesh:0.1.0`) + tagué GitHub Release — done 2026-05-27 commit `adae305`
- Epic 10 mergé sur `main` — done
- Aucune autre dépendance feature

**Dépendances aval (Epic 11 TVA Suisse) :** Epic 11 ne démarre pas tant que v0.1.1 n'est pas mergée sur `main` ET que Guy a validé l'install fresh + un cycle de recovery break-glass sur sa prod NAS. Le but est de **vérifier en prod réelle que l'install marche** avant de passer en feature work v0.2.

---

## Décisions clés posées 2026-05-27

| # | Décision | Implication |
|---|---|---|
| H1 | **Pas de workaround SQL** ce soir, on attend v0.1.1 propre | Prod Guy gelée temporairement, pas de pollution DB par INSERT manuel |
| H2 | **Option A** pour fix onboarding (bootstrap stub company + admin .env) | Conserve le flow auth existant. Wizard détecte mode « setup en cours » et propose renommage/config. Minimum de change. Le `.env` reste source de vérité du 1er admin (cohérent avec break-glass) |
| H3 | **Break-glass `KESH_ADMIN_RESET=true`** comme recovery v0.1.1 | Quick fix offline, ~30 lignes Rust, pas de schema change, pas de SMTP. Trade-off : exige accès SSH au NAS. Acceptable v0.1.x ; recovery production-grade reportée Issue #122 v0.2+ |
| H4 | **Logs fichier en `./log/`** relatif au cwd docker-compose (pas `/var/log/kesh` host) | Backup Hyper Backup scope unique sur `/volume1/docker/kesh/` (env + compose + log + DB sont co-localisés). Opt-out via `KESH_LOG_FILE_PATH=""` pour Docker users qui préfèrent stdout pur |
| H5 | **Workflow full BMAD uniforme** pour les 3 stories | Cycle complet `create-story + spec validate 4 passes + dev-story + code-review 4 passes` par story. Qualité Epic 10 maintenue. ~1.5-2 jours total |
| H6 | **Port défaut 80 (Issue #118) AJOUTÉ à v0.1.1** (révisé 2026-05-28) | Qualité install pour les autres utilisateurs : URL HTTP standard sans `:3000`. Défaut applicatif `KESH_PORT=80` (`config.rs`) + `EXPOSE 80` (`Dockerfile`) + les 3 compose pointent le container sur `:80`. Le **mapping host reste au choix de l'utilisateur** (ex. `3000:80`, IP dédiée container macvlan) — doc explique comment remapper si conflit port 80 (notamment Synology DSM / Web Station). **N'inverse PAS D5** : le bind loopback `127.0.0.1` du compose prod est conservé (reverse proxy reste le modèle d'exposition LAN recommandé). Décision Guy 2026-05-28 |
| H7 | **Pas de PRs parallèles** — 3 stories en commits stackés sur `epic-hotfix-v0.1.1` ou PRs séquentielles | Cohérent `feedback_avoid_parallel_prs`. À décider au moment de la 1ère PR selon scope effectif |
| H8 | **0 breaking change** depuis v0.1.0 — pas de bump `kesh_version_min_required` | Migration P3 (CLAUDE.md migration breaking policy) : aucune migration de cette release ne tombe dans les opérations breaking (`DROP COLUMN`, `RENAME`, `MODIFY COLUMN`). Si une story introduit `ADD COLUMN nullable`, c'est non-breaking → pas de bump |
| H9 | **Logs fichier en premier** (Story v011-1) — pas en dernier | Sans logs accessibles, le debugging des Stories v011-2 (catch-22) et v011-3 (break-glass) en dev/test serait pénible. Livrer cette story-zéro d'abord donne aux 2 autres une infrastructure d'observabilité. Décision Guy 2026-05-27 |

---

## Stories

### Story v011-1 : Logs fichier avec rotation (Issue #119)

**Severity : story-zéro infrastructure.** Livré en premier pour donner les 2 stories suivantes (v011-2 catch-22 + v011-3 break-glass) une infrastructure de logs accessible hors `docker logs` — facilite le debugging local + en prod NAS (décision H9).

**Scope :** Ajouter une seconde sortie de logs **fichier avec rotation native** via `tracing-appender`, configurable via env vars, mount par défaut `./log/:/var/log/kesh` dans `docker-compose.prod.yml`.

**Acceptance criteria (high-level) :**
- [ ] `tracing-appender` ajouté à `crates/kesh-api/Cargo.toml`.
- [ ] `KESH_LOG_FILE_PATH` + `KESH_LOG_FILE_ROTATION` (`daily`/`hourly`/`never`) + `KESH_LOG_FILE_MAX_FILES` + `KESH_LOG_FILE_FORMAT` (`pretty`/`json`) lus dans `config.rs` avec validations + defaults.
- [ ] Subscriber tracing avec layer stdout (existant) + layer fichier conditionnel.
- [ ] Tests unitaires `config.rs` couvrent defaults + validations.
- [ ] Test intégration : container avec `KESH_LOG_FILE_PATH=/tmp/kesh-test.log` → après `curl /health`, fichier contient des entrées.
- [ ] `docker-compose.prod.yml` : mount `./log:/var/log/kesh` **activé par défaut** + `KESH_LOG_FILE_PATH: /var/log/kesh/kesh.log` dans env.
- [ ] `.gitignore` ajoute `log/` (logs ne doivent jamais être commités).
- [ ] `.env.example` section nouvelle « Logs fichier » avec exemples + warning perms volume host.
- [ ] Manuel admin sous-section « Configuration des logs fichier » dans § Configuration.
- [ ] CHANGELOG v0.1.1 entrée `Added`.
- [ ] CI verte.

**Effort estimé :** ~0.5 jour (scope contenu — pas de schema change, ajout dépendance Rust, doc, mount compose).

### Story v011-2 : Fix catch-22 onboarding fresh-install (Issue #120)

**Severity : catégorie A bloquante.** Sans ce fix, aucun utilisateur ne peut compléter l'install de Kesh v0.1.0 sans connaissance d'un workaround SQL.

**Scope :** Modifier `crates/kesh-api/src/auth/bootstrap.rs` pour **créer simultanément une company stub ET l'admin depuis `.env`** quand DB vide (`company_count == 0 AND user_count == 0`).

**Modèle pseudo-Rust :**
```rust
if company_count == 0 && user_count == 0 {
    // Créer une company stub minimaliste — le wizard la complétera
    let stub_id = sqlx::query("INSERT INTO companies (name, language, is_stub, created_at, updated_at) VALUES (?, ?, TRUE, NOW(), NOW())")
        .bind("Setup en cours")
        .bind(&config.lang)
        .execute(pool).await?
        .last_insert_id();

    // Créer l'admin attaché à cette company stub
    let hash = hash_password_async(config.admin_password.clone()).await?;
    users::create(pool, NewUser {
        username: config.admin_username.clone(),
        password_hash: hash,
        role: Role::Admin,
        active: true,
        company_id: stub_id as i64,
    }).await?;

    tracing::info!(
        "✅ bootstrap: company stub (id={}) + admin '{}' créés depuis .env. Compléter l'onboarding via UI pour renommer/configurer.",
        stub_id, config.admin_username
    );
}
```

**Composante DB :** Migration `ADD COLUMN is_stub BOOLEAN NOT NULL DEFAULT FALSE` sur `companies` (non-breaking, valeur défaut existante pour rows existantes). Permet au frontend de détecter qu'une company est en mode « setup pending » et d'afficher un état UX adapté (banner ou wizard partiel pour finir la config).

**Composante frontend (mineure) :**
- `onboardingState.fetchState()` reçoit le flag `isStub` de la company courante.
- Si `isStub == true` au login, l'utilisateur est dirigé vers le wizard de complétion (renommer company, choisir plan comptable, exercice initial) au lieu du flow normal.
- Si l'utilisateur clôt le wizard (ou complète sans renommer), warning visible « Votre company a un nom de placeholder ».

**Acceptance criteria (high-level — à détailler en spec validate) :**
- [ ] Migration `add_companies_is_stub.sql` créée et ajoutée à `docs/migrations-idempotence-audit.md`.
- [ ] `bootstrap.rs` crée stub company + admin si `company_count == 0 AND user_count == 0`.
- [ ] Tests unitaires bootstrap couvrent : fresh install (création stub+admin), reset (skip — users existe), partial state (company existe sans user, edge case).
- [ ] Frontend détecte `isStub` et propose flow complétion wizard.
- [ ] Test E2E Playwright : fresh install → wizard complet → login OK → renommage company → isStub passe à false.
- [ ] CHANGELOG v0.1.1 entrée `Fixed` détaillée.
- [ ] CI verte, 0 régression sur baseline existante.

**Effort estimé :** ~1 jour (story complexe — backend + DB migration + frontend + tests E2E).

### Story v011-3 : Break-glass admin reset via KESH_ADMIN_RESET (Issue #121)

**Severity : amélioration ops (recovery offline).** Sans ce fix, un admin qui oublie son mdp est lock-out sans recovery propre.

**Scope :** Ajouter une variable `KESH_ADMIN_RESET=true` qui déclenche au boot un reset du hash password de l'admin matching `KESH_ADMIN_USERNAME` vers `KESH_ADMIN_PASSWORD`.

**Acceptance criteria (high-level) :**
- [ ] `KESH_ADMIN_RESET` lu dans `config.rs` (bool, default `false`, validation strict des valeurs `"true"`/`"1"`/`"yes"` case-insensitive).
- [ ] Si `KESH_ADMIN_RESET=true` au boot : trouve user `KESH_ADMIN_USERNAME`, vérifie role Admin, hash + UPDATE password_hash, revoke refresh_tokens, audit_log event `admin_break_glass_reset`.
- [ ] Fail-fast si user introuvable OU pas Admin.
- [ ] Log ERROR explicite avec rappel de retirer la var.
- [ ] Tests unitaires : reset OK, user not found, user not admin, KESH_ADMIN_RESET=false skip.
- [ ] Test intégration : container restart avec KESH_ADMIN_RESET=true + nouveau mdp → login OK.
- [ ] `.env.example` section nouvelle « Recovery break-glass » avec exemple commenté + warning de retirer la var post-reset.
- [ ] Manuel admin (`docs/manual/fr/admin-manual.tex` + PDF) : sous-section « J'ai oublié mon mot de passe administrateur » avec procédure step-by-step.
- [ ] CHANGELOG v0.1.1 entrée `Added` : « Recovery break-glass admin password via `KESH_ADMIN_RESET` env var ».
- [ ] CI verte.

**Effort estimé :** ~0.5 jour (scope contenu — pas de schema change, code Rust limité, doc).

### Story v011-4 : Port par défaut 80 (Issue #118)

**Severity : amélioration qualité install.** Décidée 2026-05-28 (révision décision H6). Guy veut éviter les soucis de port en prod, **surtout pour les autres utilisateurs** qui installeront Kesh — URL HTTP standard `http://kesh.local` sans `:3000`.

**Scope :** Changer le port applicatif par défaut de `3000` à `80` (port interne du container) et aligner les artefacts Docker + doc. Le **mapping host reste au choix de l'utilisateur** : si le port 80 est occupé (Synology DSM, Web Station), l'utilisateur remappe lui-même (`3000:80`, IP dédiée container, etc.) sans toucher au défaut applicatif. Le bind loopback `127.0.0.1` du compose prod (D5) est conservé.

**Surface (ground-truth 2026-05-28) :**
- `crates/kesh-api/src/config.rs` — défaut `port: 3000` (ligne ~293) + 4 fallbacks `3000` (lignes ~363/364, ~369/372, ~375) → `80` ; doc comment (ligne ~27).
- `Dockerfile` — `EXPOSE 3000` (ligne 28) → `EXPOSE 80`. Container tourne en root (pas de `USER`) → bind 80 OK sans `CAP_NET_BIND_SERVICE`.
- `docker-compose.prod.yml` — `KESH_PORT: ${KESH_PORT:-80}`, mapping `127.0.0.1:80:80` (loopback D5 conservé), healthcheck `http://localhost/health`, commentaires d'exposition mis à jour.
- `docker-compose.yml` — `KESH_PORT: ${KESH_PORT:-80}`, mapping `80:80`, healthcheck.
- `docker-compose.dev.yml` — `KESH_PORT: "80"`, mapping `127.0.0.1:80:80`, healthcheck. ⚠️ cf. Q7 (friction `cargo run` natif).
- `crates/kesh-api/src/config.rs` tests — assertions sur le port défaut `3000` → `80`.
- `.github/workflows/release.yml` — smoke test `docker run` + `curl` sur port 80.
- `.env.example` — commentaire `KESH_PORT` + note conflit port 80 / procédure d'override.
- `README.md` quickstart — `curl http://localhost/health` (sans `:3000`).
- `docs/manual/fr/admin-manual.tex` (+ PDF régénéré) — sous-section « Changer le port d'écoute (conflit port 80, ex. Synology DSM) ».
- `CHANGELOG.md` v0.1.1 — entrée `Changed`.

**Acceptance criteria (high-level — à détailler en spec validate) :**
- [ ] `config.rs` défaut `KESH_PORT=80` + tests assertions mis à jour.
- [ ] `Dockerfile` `EXPOSE 80`.
- [ ] 3 compose pointent le container sur `:80` (healthchecks alignés), bind loopback prod conservé.
- [ ] `.env.example` documente le défaut 80 + procédure override en cas de conflit (Synology DSM / Web Station) avec exemples (`3000:80`, IP dédiée container).
- [ ] Manuel admin FR : sous-section « Changer le port d'écoute » (+ PDF régénéré).
- [ ] README quickstart : `curl http://localhost/health` (sans `:3000`).
- [ ] Release smoke test (`release.yml`) curl sur port 80 vert.
- [ ] CHANGELOG v0.1.1 entrée `Changed` documente le breaking *de configuration* (utilisateurs v0.1.0 doivent accepter le défaut 80 OU setter `KESH_PORT=3000` dans leur `.env`/mapping).
- [ ] CI verte, 0 régression (vérifier que les E2E Playwright + smoke test référencent le port via env/config et non en dur).

**Effort estimé :** ~0.5 jour (Small — pas de schema change ; config + Docker + doc ; vigilance sur les références de port en dur dans tests/smoke).

**Séquencement :** Livré **en dernier** (v011-4), après le fix bloquant catch-22 (v011-2). Si l'E2E fresh-install de v011-2 ou le smoke test release référencent le port en dur, les re-valider contre 80 dans cette story.

---

### Story v011-5 : Onboarding self-service (admin créé via UI au 1er boot)

**Severity : amélioration UX install (post-v0.1.1, ajoutée 2026-05-30).** Le mécanisme `.env`-bootstrap livré par v011-2 fonctionne mais exige que l'utilisateur édite `.env` avant le 1er `docker compose up`, connaisse le mot de passe choisi, et le change après login. Pattern non-standard pour les apps self-hosted (Jellyfin, Bitwarden, Sonarr, Vaultwarden, etc. demandent l'admin via un formulaire web au 1er lancement). C'est très exactement ce que décrivait la section « fictive » du manuel admin (réécrite v011-2 commit `d36953a`) — concept connu, jamais implémenté.

**Scope :** Remplacer le mécanisme `.env`-bootstrap par un flow web self-service. Au 1er boot sur DB vide :
- Le bootstrap crée **uniquement** la company stub (`is_stub=TRUE`, pas d'admin) si `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` ne sont pas renseignés.
- Toutes les routes API protégées renvoient `423 Locked` tant que `users` est vide ; le frontend détecte et redirige vers `/setup`.
- Route ouverte unique `POST /api/v1/setup/admin` qui accepte `{ username, password }`, refuse si un user existe déjà (auto-disable au 1er succès), hash le password, crée l'admin attaché à la company stub, renvoie les cookies de session HttpOnly (réutilise Story 10-5).
- Frontend : écran « Bienvenue dans Kesh » avec formulaire username/password (≥12 chars) + redirection automatique vers le wizard onboarding existant après création.
- `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` deviennent **optionnels** : si présents au bootstrap, comportement v011-2 inchangé (legacy/CI/Test/déploiements v0.1.1 existants) ; sinon mode setup-UI.

**Interaction avec v011-3 (break-glass) :** si l'admin est créé via UI (pas de `KESH_ADMIN_USERNAME` en `.env`), `KESH_ADMIN_RESET` n'a pas de cible naturelle. v011-3 devra soit (a) accepter un override `KESH_ADMIN_RESET_USERNAME` au boot, soit (b) cibler par défaut le seul user Admin si unique. À trancher au spec validate.

**Acceptance criteria (high-level — à détailler en spec validate) :**
- [ ] Bootstrap : si `users` vide et `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` non renseignés → ne crée que la company stub (pas d'admin).
- [ ] Bootstrap : si `KESH_ADMIN_*` présents → comportement v011-2 strictement inchangé (legacy/CI/Test).
- [ ] Route `POST /api/v1/setup/admin` (open, sans auth) : crée l'admin si `user_count == 0`, renvoie 410 Gone sinon (auto-disable).
- [ ] Validation password ≥ 12 chars (politique `KESH_PASSWORD_MIN_LENGTH` existante).
- [ ] Login direct post-création (cookies HttpOnly Story 10-5 réutilisés, pas de re-login manuel).
- [ ] Routes API protégées renvoient 423 Locked (ou code équivalent) tant que `users` est vide ; permet au frontend de distinguer « pas authentifié » vs « pas encore setup ».
- [ ] Frontend : route `/setup` (publique) avec formulaire + détection 423 + redirection ; redirection vers `/onboarding` post-création.
- [ ] Rate-limit IP-based sur `/setup/admin` (l'endpoint s'auto-disable au 1er succès, mais protège contre brute-force username avant succès).
- [ ] Manuel admin FR : section « Premier démarrage » à nouveau réécrite (création admin via UI) + warning « bloquer l'accès réseau public avant 1er boot — qui touche `/setup/admin` en premier devient admin ».
- [ ] CHANGELOG : entrée `Changed` (mécanisme onboarding) + `Deprecated` (`.env` admin si décision deprecation tranchée v0.2).
- [ ] PDF manuel admin régénéré + KF-035 (#127) partiellement adressée (la section onboarding du manuel devient ground-truth).
- [ ] Tests : unit (bootstrap skip admin si env non set), intégration (POST /setup/admin success + 410 si user existe + 423 sur routes protégées), E2E Playwright (fresh-install setup form → wizard → app).

**Questions ouvertes (à trancher en spec validate) :**
- **Cible release** : v0.1.2 (hotfix UX) ou v0.2 (refactor onboarding plus large) ?
- **Deprecation `.env` admin** : couper net en v0.2 ou maintenir indéfiniment comme fallback CI/Test ?
- **Interaction v011-3 break-glass** : `KESH_ADMIN_RESET_USERNAME` override ou « single-admin auto-target » ?
- **Bind loopback obligatoire avant setup** : forcer 127.0.0.1 tant que `users` vide (sécurité), ou s'appuyer uniquement sur le warning manuel ?

**Effort estimé :** ~2 jours (Medium — schema change léger, route + middleware backend, écran frontend, security review obligatoire sur le gate de setup).

**Séquencement :** À planifier APRÈS v011-3 (le break-glass reste utile pour les déploiements v0.1.1 existants qui ont déjà un admin `.env`). Probablement cible v0.2 (refactor pas hotfix), mais peut être v0.1.2 si décision rapide. Indépendant de v011-4 port 80.

---

## Critères d'arrêt Epic Hotfix v0.1.1 (= release v0.1.1 prête)

- [ ] 4/4 stories avec status `done` dans `sprint-status.yaml` (`v011-1`, `v011-2`, `v011-3`, `v011-4`).
- [ ] Issue #119 [logs fichier rotation] **fermée** sur GitHub (via Story v011-1).
- [ ] Issue #120 [catch-22 onboarding] **fermée** sur GitHub (via Story v011-2).
- [ ] Issue #121 [break-glass admin reset] **fermée** sur GitHub (via Story v011-3).
- [ ] Issue #118 [port défaut 80] **fermée** sur GitHub (via Story v011-4).
- [ ] Image Docker Hub `gcorbaz/kesh:v0.1.1` + `:latest` (re-pointé) publiée (via tag git `v0.1.1`).
- [ ] CHANGELOG.md v0.1.1 humanisé + section `[0.1.1] — YYYY-MM-DD` complète.
- [ ] **Install fresh effective sur NAS Synology Guy** (re-deploy de zéro) : `docker compose up -d` → wizard accessible → company créée → admin loggé → 0 friction.
- [ ] **Cycle break-glass testé sur prod NAS** : `KESH_ADMIN_RESET=true` + nouveau mdp → restart → login OK avec nouveau mdp → audit_log entry présent.
- [ ] **Logs fichier visibles** dans `./log/` côté NAS, rotation fonctionnelle (testée en simulant `docker compose restart` répétés).
- [ ] **Port 80 effectif** : install fresh joignable sur le port standard (`http://<nas>` ou via mapping/proxy de l'utilisateur), procédure d'override en cas de conflit documentée et vérifiée.
- [ ] Rétrospective Hotfix v0.1.1 produite (court — 3 stories, ~2 jours, peu de findings attendus).
- [ ] PR(s) Epic Hotfix v0.1.1 mergée(s) sur `main`.
- [ ] **0 régression** sur baselines existantes : 250+ Vitest + cargo test workspace + 76 Playwright E2E + smoke test release.yml.
- [ ] **Pre-flight docs sync avant tag** (règle CLAUDE.md) : README sans claim sur v0.1.0 obsolète, website/index.html mention v0.1.1 disponible, manuels FR à jour.

---

## Risques & questions ouvertes

| # | Question | À résoudre |
|---|---|---|
| Q1 | **Migration `is_stub` rétro-compat** — les déploiements v0.1.0 existants (théoriquement aucun à part Guy qui sera reset) auront `is_stub = FALSE` par défaut. Aucun impact runtime. À confirmer en spec validate Story v011-2 | Story v011-2 spec validate Pass 1 |
| Q2 | **Le wizard frontend doit-il bloquer l'accès au reste de l'app tant que la company est `is_stub == true` ?** Strict (oui) ou laxe (warning + accès partiel) ? Affecte UX et complexité | Story v011-2 spec validate (peut être tranché par Guy en Pass 1) |
| Q3 | **Break-glass impacte audit log compliance OLICo Art. 9** ? Le reset par env var hors UI peut sembler bypasser l'audit-trail. Mitigation : audit_log event distinct `admin_break_glass_reset` + warning ERROR + doc explicite « usage exceptionnel à tracer hors-app » | Story v011-3 spec validate (consulter règle « zero carry-forward ») |
| Q4 | **Logs fichier perms volume Docker** — si `kesh-api` tourne avec UID 0 (root, cas actuel) et écrit dans `./log/`, le owner host-side sera `root`. Synology DSM peut-il backuper proprement ? Tester avant clôture v0.1.1 | Story v011-1 dev (test en prod réelle Guy) |
| Q5 | **Bundle des 3 stories en 1 PR ou 3 PRs séquentielles ?** Cohérent `feedback_avoid_parallel_prs` mais 1 grosse PR peut être lourde. À trancher après spec validate Story v011-2 (selon taille des patches estimée) | Pré-PR Story v011-2 |
| Q6 | **CHANGELOG entry v0.1.1 = 1 section ou 1 section par story ?** Cohérent Keep a Changelog : sections par type (Added/Fixed/Changed/Security) regroupant tous les changements de la release | Doc avant release tag |
| Q7 | **Friction `cargo run` natif (Story v011-4)** — défaut `KESH_PORT=80` impose à un dev qui lance `cargo run` sans env de binder le port 80 (privilégié < 1024 sur Linux/macOS) : le bind échoue sans `CAP_NET_BIND_SERVICE`/root. (Le mapping host Docker, lui, ne demande pas root car c'est le daemon qui binde.) Mitigation : doc dev + `.env` local `KESH_PORT=3000`, ou défaut dev distinct. À trancher | Story v011-4 spec validate Pass 1 |

---

## Références

- Issue #120 : Catch-22 fresh install onboarding (cat A bloquant)
- Issue #121 : Break-glass admin reset via `.env`
- Issue #119 : Logs fichier avec rotation
- Issue #122 : Recovery production-grade SMTP+2FA (v0.2+, NOT in v0.1.1)
- Issue #118 : Port défaut 80 (AJOUTÉ v0.1.1 via Story v011-4)
- `crates/kesh-api/src/config.rs` — port défaut 3000→80 Story v011-4 (+ `KESH_LOG_FILE_*` v011-1, `KESH_ADMIN_RESET` v011-3)
- `Dockerfile:28` (`EXPOSE 3000`→80) + `docker-compose.{prod,dev,yml}` mappings + `.github/workflows/release.yml` smoke test — Story v011-4
- `crates/kesh-api/src/auth/bootstrap.rs` — code à modifier Story v011-2 + v011-3
- `crates/kesh-api/src/routes/users.rs:264` — `PUT /reset-password` existant (admin RBAC, ne couvre pas le break-glass solo)
- `crates/kesh-api/src/config.rs` — nouvelles env vars à ajouter (`KESH_LOG_FILE_*` Story v011-1, `KESH_ADMIN_RESET` Story v011-3)
- `frontend/src/routes/onboarding/+layout.ts` — auth gate qui cause le catch-22 (à comprendre + adapter pour gérer `isStub`)
- `docker-compose.prod.yml` — mount `./log/` à ajouter Story v011-1
- `.env.example` — 2 sections nouvelles (`KESH_ADMIN_RESET` + `KESH_LOG_FILE_*`)
- `docs/manual/fr/admin-manual.tex` — 2 nouvelles sous-sections à ajouter + PDF régénéré
- `CLAUDE.md` règles applicables : « zero carry-forward » (cat A bloquant), « migration breaking policy » (aucune migration breaking ici), « avoid parallel PRs », « synchroniser TOUTES les docs avant release »
