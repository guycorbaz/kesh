---
epic: 10
title: "Déploiement & Opérations"
version: v0.1
status: planning
sourceArtifact: _bmad-output/planning-artifacts/epics.md §"Epic 9 Déploiement & Opérations" (archive — mapping ancien → courant ligne 25) + retro Epic 9.5 action items pré-Epic 10 + session pre-flight 2026-05-21
relatedFRs:
  - FR1 (install < 15 min via docker-compose)
  - FR2 (config via env vars)
  - FR3 (admin initial via env)
  - FR78 (backup recommandé avant new version)
  - FR79 (sqlx migrate run auto au boot)
  - FR89 (frontend SPA accessible même si DB down)
  - UX-DR43 (page d'attente élégante)
  - UX-DR44 (guide install livré avec compose + .env.example)
relatedDecisions:
  - "v0.1.0 = artefact distribuable Docker Hub + install NAS Synology Guy (mise en prod immédiate sauf blocker)"
  - "Continuité données garantie à partir de v0.1.0 — toutes versions ultérieures migrent sans intervention utilisateur"
  - "MariaDB externe NAS Synology (Package Center 10.11.11) — pas de MariaDB bundled en prod"
  - "Backup natif Synology Package Center MariaDB (planifié DSM) + DSM Hyper Backup sur volumes"
  - "HTTP only LAN privé pour v0.1.0 — reverse proxy = dette catégorie B v0.2+"
  - "amd64-only — workflow `release.yml` actuel OK (NAS Synology x86_64 récent)"
  - "Tokens httpOnly + Secure cookies — Option A sécurité (refonte storage backend + frontend)"
  - "CI tests MariaDB < 11 uniquement (pin 10.11 = parity avec NAS prod)"
crates:
  - kesh-api (boot fail-fast secrets/admin, downgrade protection migrations, `_kesh_version` table, healthcheck DB-aware 503)
  - frontend (résilience DB down + retrait localStorage tokens + endpoint `/me` consumer)
  - kesh-db (audit idempotence 26 migrations + table `_kesh_version`)
  - infra (docker-compose.prod.yml séparé, .env.example sans defaults insecure, alignment MariaDB 10.11 partout)
  - .github/workflows (CI matrix MariaDB 10.11, release.yml smoke test post-build)
  - docs (manuel admin install NAS Synology + procédure backup DSM + CHANGELOG v0.1.0)
stories:
  - 10-1-hardening-docker-production
  - 10-2-migrations-idempotence-downgrade-protection
  - 10-3-resilience-frontend-db-inaccessible
  - 10-4-manuel-install-synology-backup-dsm-changelog
  - 10-5-httponly-tokens-security
---

# Epic 10 — Déploiement & Opérations

## Vue d'ensemble

**Objectif :** Livrer une image Docker `gcorbaz/kesh:v0.1.0` publiée sur Docker Hub, installable sur un NAS Synology x86_64 récent avec MariaDB 10.11 (Package Center DSM), prête pour usage production réel avec données comptables. La clôture d'Epic 10 = release v0.1.0 = passage en prod (sauf blocker découvert).

**Périmètre :** 5 stories de hardening + livrable manuel install + amélioration pipeline release.yml. Aucune feature comptable nouvelle. Toutes les fonctions métier de v0.1 (Epics 1-9.5) sont déjà livrées et stables.

**Hors scope v0.1.0 :**
- HTTPS / reverse proxy (LAN privé en v0.1.0, dette catégorie B v0.2+)
- Backup automatisé par script intégré image (backup = DSM Package Center MariaDB tools + Hyper Backup, doc manuelle en 10-4)
- Auto-update strategy (pull manuel pour v0.1.0, Watchtower éventuellement v0.2+)
- Build multi-arch arm64 (amd64-only confirmé, NAS Guy = x86_64 récent)
- TVA Suisse (= Epic 11, première feature v0.2)

**Provenance :**
- Scope PRD originel : `epics.md` §"Epic 9 Déploiement & Opérations" (archive historique, mapping ancien → courant ligne 25 du fichier)
- Retro Epic 9.5 (2026-05-20) action items pré-Epic 10 : audit cohérence planning (PR #101 ✅), doc utilisateur PME (manuels LaTeX PR #102 ✅), Milestone v0.2 (différée post-Epic 10 — décision Guy 2026-05-21)
- Session pre-flight 2026-05-21 : audit critique du repo (Dockerfile + docker-compose + release.yml + tokens localStorage + migrations MariaDB 10.6+) → 5 stories + 1 amélioration pipeline identifiées

**Dépendances amont :**
- Epic 9.5 done (PR #99 mergée `b6c4987`) — zero tech debt carry-forward catégorie A respecté
- PR #101 mergée `d12cbe2` — drift numérotation epics fixé
- PR #102 mergée `47cb02f` — manuels LaTeX FR (admin/user/marketing) base éditoriale pour 10-4

**Dépendances aval (Epic 11 TVA Suisse) :** Epic 11 ne démarre **pas** tant qu'Epic 10 n'est pas mergé sur `main` ET que la prod NAS de Guy ne tourne pas avec v0.1.0. Le but est de découvrir en prod réelle ce qui manque avant de basculer en feature work v0.2.

---

## Décisions clés posées en pre-flight 2026-05-21

| # | Décision | Implication |
|---|---|---|
| D1 | v0.1.0 = Docker Hub `gcorbaz/kesh` + install NAS Guy en prod réelle | Pas de release intermédiaire v0.1.x — la 1ère release publiée est la 1ère en prod |
| D2 | MariaDB externe NAS Synology (10.11.11 Package Center DSM) | `docker-compose.prod.yml` sans service `mariadb`. Connexion via `DATABASE_URL` à l'hôte. Backup = outils DSM natifs |
| D3 | Compat MariaDB 10.6+ confirmée (audit code) — pas de feature 11-only utilisée | CI test matrix bloqué à MariaDB 10.11 (parity NAS Synology). Alignement composes + commentaires migrations |
| D4 | Tokens auth en cookies httpOnly + Secure + SameSite=Strict (pas localStorage) | Refonte storage Story 10-5 : backend `Set-Cookie` + frontend retire localStorage + endpoint `/me` consumer. Issue #41 cat A closure |
| D5 | HTTP only LAN privé v0.1.0 | Reverse proxy = dette catégorie B v0.2+. Manuel admin avertit explicitement « LAN privé uniquement » |
| D6 | Backup = DSM Package Center MariaDB scheduled + Hyper Backup volumes | Pas de script `kesh-backup` intégré image. Doc procédure en 10-4 |
| D7 | Continuité données garantie dès v0.1.0 | Story 10-2 : audit idempotence 26 migrations + table `_kesh_version` + downgrade protection au boot |
| D8 | amd64-only | `platforms: linux/amd64` dans release.yml inchangé |
| D9 | Réseau Docker **externe** géré par l'utilisateur (Guy l'a déjà créé sur son NAS) | `docker-compose.prod.yml` déclare `networks: { kesh-net: { external: true } }` — manuel admin 10-4 documente uniquement « créer le réseau Docker `kesh-net` avant `docker compose up` », pas d'option networking à choisir |
| D10 | Bootstrap admin **idempotent** (vérifié `crates/kesh-api/src/auth/bootstrap.rs:39-47` + 4 tests) — pas de reseed si users existent | Aucune action requise. Manuel 10-4 peut documenter rappel : `KESH_ADMIN_PASSWORD` n'est utilisé qu'au tout premier boot avec users table vide |
| D11 | Pas d'autre installation Kesh existante (Guy = single user pre-prod) — breaking change cookies httpOnly Story 10-5 confirmé acceptable | Story 10-5 peut casser le flow auth localStorage → cookies sans préoccupation rétro-compat |

---

## Stories

### Story 10-1 : Hardening Docker production

**As a** administrateur Kesh
**I want** un livrable Docker production durci (sans defaults insecure, logs rotatés, ressources bornées)
**So that** je puisse déployer Kesh sur mon NAS sans configuration sécurité supplémentaire et sans risque de saturation disque/mémoire

**Périmètre :**

1. **`docker-compose.prod.yml` séparé** (nouveau fichier) :
   - Service unique `kesh-api` (pas de `mariadb` bundled — décision D2)
   - `DATABASE_URL` pointant vers MariaDB hôte (valeur exacte fournie par l'utilisateur via `.env` — voir D9 networking)
   - Réseau Docker **externe** déclaré dans le compose (`networks: { kesh-net: { external: true } }`) — l'utilisateur le crée sur son NAS avant le `docker compose up`
   - Pas de bind-mount `./crates:ro` (dev-only — anti-pattern prod)
   - Pas d'exposition `3306:3306` (DB externe gérée DSM)
   - `KESH_HOST=127.0.0.1` (binding loopback puisque pas de reverse proxy v0.1.0)
   - `restart: unless-stopped`
2. **Fail-fast boot sur secrets default ou faibles** dans `kesh-api/src/config.rs` (ou `main.rs`) :
   - `KESH_JWT_SECRET` : refuser si vide, contient `change-me`, ou longueur < 32 caractères. Log erreur explicite + exit non-zero.
   - `KESH_ADMIN_PASSWORD` : refuser si vide, contient `changeme`, ou longueur < 12 caractères. Log erreur explicite + exit non-zero.
   - Test unitaire chaque cas (3 scénarios par variable).
3. **Log rotation Docker** dans `docker-compose.prod.yml` :
   ```yaml
   logging:
     driver: json-file
     options:
       max-size: 10m
       max-file: 5
   ```
4. **Resource limits** dans `docker-compose.prod.yml` :
   - `mem_limit: 1g` sur kesh-api (protection runaway, OK pour usage solo NAS)
   - `cpus: 2.0` (optionnel — à benchmark)
5. **Alignment MariaDB 10.11 partout** (décision D3) :
   - `docker-compose.yml:4` `mariadb:11-jammy` → `mariadb:10.11`
   - `docker-compose.dev.yml:43` `mariadb:11.4` → `mariadb:10.11`
   - `.github/workflows/ci.yml:27` `mariadb:11.4` → `mariadb:10.11`
   - Mise à jour commentaire migration `20260513000001_reconciliation_rules.sql:28` (« Docker Compose pin `mariadb:11-jammy`, OK » → « pin `mariadb:10.11`, requis MariaDB ≥ 10.6 »)
6. **`.env.example`** (nouveau fichier) :
   - Toutes les variables d'env documentées avec commentaires + exemples + générateur recommandé (e.g. `KESH_JWT_SECRET=$(openssl rand -base64 32)`)
   - Aucun default insecure (placeholders explicites `<GENERATE_ME>`)

**Critères d'acceptation :**

- **Given** `docker-compose.prod.yml`, **When** `docker compose -f docker-compose.prod.yml config` exécuté, **Then** validation YAML OK et aucune référence à un service `mariadb` interne.
- **Given** `kesh-api` démarré avec `KESH_JWT_SECRET=change-me-32-bytes-minimum-secret-generate-with-openssl-rand-hex-32` (default actuel docker-compose.yml), **When** boot, **Then** exit non-zero + log `FATAL: KESH_JWT_SECRET contains default placeholder, refuse to boot`. Idem 3 scénarios (vide / contient `change-me` / longueur < 32).
- **Given** `kesh-api` démarré avec `KESH_ADMIN_PASSWORD=changeme`, **When** boot, **Then** exit non-zero + log équivalent. Idem 3 scénarios.
- **Given** `docker-compose.prod.yml` running 24h+ avec logs verbose, **When** `du -sh /var/lib/docker/containers/...`, **Then** taille logs ≤ 50 MB (5 fichiers × 10 MB max — décision #3).
- **Given** workflow CI relancé après modif `ci.yml:27`, **When** services job, **Then** `mariadb:10.11` démarre + tous les tests Rust passent (verify compat MariaDB 10.11 préservée).
- **Given** `.env.example`, **When** lecture, **Then** documenté `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -base64 32>` et `KESH_ADMIN_PASSWORD=<GENERATE_ME: openssl rand -base64 24>` (placeholders explicites).
- **And** `Test Locally First` (CLAUDE.md) appliqué avant push.

**Effort estimé :** 1-2 jours.

**Path-dependency :** indépendant. Peut démarrer en premier.

---

### Story 10-2 : Migrations idempotence + downgrade protection + CI MariaDB 10.11

**As a** administrateur Kesh
**I want** que toute mise à jour Kesh applique automatiquement les migrations DB sans risque de corruption, et qu'une image plus ancienne refuse de démarrer sur une DB déjà migrée plus récemment
**So that** je n'aie aucune crainte d'updater (rollback impossible silencieusement = données préservées)

**Périmètre :**

1. **Audit idempotence des 26 migrations existantes** (`crates/kesh-db/migrations/*.sql`) :
   - Vérification chaque migration : si déjà appliquée, ré-exécution = no-op (pas de DROP TABLE IF EXISTS qui supprimerait données).
   - sqlx tracking via table `_sqlx_migrations` est déjà géré nativement — vérifier qu'aucune migration ne contredit ce mécanisme par un side-effect destructif.
   - Documentation : ajout d'un commentaire `-- idempotent: yes/no + détail` en tête de chaque migration.
2. **Table `_kesh_version` + downgrade protection** :
   - Nouvelle migration `2026MMDD_kesh_version.sql` qui crée `_kesh_version (id, kesh_version_min_required, applied_at)`.
   - Au boot de `kesh-api`, après `sqlx migrate run`, écrire la version actuelle binaire (e.g. `v0.1.0`) dans la table.
   - **Downgrade protection** : avant `sqlx migrate run`, lire la version min-required existante. Si `version_du_binaire < version_min_required`, refuser le boot avec log explicite « Database was migrated by a newer Kesh version (X), refuse to downgrade to Y ».
3. **Test fresh install** (intégration) :
   - Test E2E ou Playwright qui : (a) wipe la DB, (b) démarre kesh-api, (c) vérifie que toutes les migrations s'appliquent, (d) vérifie qu'un seed minimal fonctionne (création company + 1 invoice + 1 journal_entry).
4. **Test upgrade path** (intégration) :
   - Test qui : (a) restore une DB en version `migration N-3` (3 migrations en retard), (b) démarre kesh-api binaire actuel, (c) vérifie que les 3 migrations restantes s'appliquent sans perte de données existantes (compter rows pre/post + checksums sur quelques tables).
5. **CI matrix MariaDB 10.11 only** (décision D3) :
   - Pas de matrice 10.11 + 11 — la matrice 11 n'aurait aucune valeur (notre cible prod est 10.11, des features 11 passant sur 11 mais failing sur 10.11 ne seraient pas détectées par 11).
   - `ci.yml` reste avec un seul service `mariadb:10.11`.

**Critères d'acceptation :**

- **Given** chaque fichier `crates/kesh-db/migrations/*.sql` (26 fichiers), **When** review, **Then** chacun porte un commentaire d'idempotence explicite (« idempotent: yes » + détail) et aucune migration ne contient `DROP TABLE` non-conditionnel sur tables avec données.
- **Given** une DB MariaDB 10.11 vierge, **When** `cargo run -p kesh-api`, **Then** les 26+1 migrations s'appliquent + une row écrite dans `_kesh_version` avec `kesh_version_min_required = v0.1.0` (ou plus précis).
- **Given** une DB MariaDB 10.11 déjà migrée par un binaire `v0.2.0` (= `_kesh_version.kesh_version_min_required = v0.2.0`), **When** démarrage d'un binaire `v0.1.0`, **Then** exit non-zero + log `FATAL: Database migrated by Kesh v0.2.0, current binary v0.1.0 cannot downgrade safely. Restore backup or upgrade binary.`.
- **Given** test intégration `tests/migrations_fresh_install.rs`, **When** exécuté en CI, **Then** PASS sur DB MariaDB 10.11 vierge + seed minimal OK.
- **Given** test intégration `tests/migrations_upgrade_path.rs`, **When** exécuté en CI avec DB en version N-3, **Then** migrations appliquées + rows pre-existing préservées (assertion COUNT(*) + checksums).
- **Given** `ci.yml`, **When** review, **Then** un seul service `mariadb:10.11` (pas de matrice 11+10.11).
- **And** **0 régression** sur les 250+ tests Vitest + cargo test workspace.

**Effort estimé :** 2-3 jours.

**Path-dependency :** indépendant. Peut démarrer en parallèle de 10-1.

---

### Story 10-3 : Résilience frontend si DB inaccessible

**As a** utilisateur Kesh
**I want** voir une page d'erreur claire et professionnelle si la base de données est temporairement indisponible
**So that** je comprenne le problème sans paniquer et que je puisse continuer à naviguer dans les pages déjà chargées en mode dégradé

**Périmètre :** (scope PRD `epics.md` Story 9.3 — FR89 + UX-DR43)

1. **Healthcheck `/health` DB-aware** : retourne `200 OK` si DB joignable, `503 Service Unavailable` si DB down. Body JSON `{ status: "ok"|"degraded", db: true|false, version: "v0.1.0" }`.
2. **Frontend SPA accessible même si DB down** :
   - SvelteKit servi via tower-http::ServeDir (statique) reste fonctionnel.
   - Page de login + pages déjà cachées en client (e.g. dashboard avec données mises en cache localStorage navigation) accessibles.
3. **Banner i18n « DB temporairement indisponible »** quand un API call échoue avec timeout ou 503 :
   - Composant `<DegradedBanner />` Svelte affiché en haut de l'app.
   - Traductions FR/DE/IT/EN (4 locales projet).
   - Retry exponentiel sur les requêtes API (300ms, 1s, 3s, 10s, give up).
4. **Logo + version Kesh** visible sur la page de login même si DB down (preuve que le frontend est servi correctement).

**Critères d'acceptation :**

- **Given** `kesh-api` démarré et MariaDB stoppée, **When** GET `/health`, **Then** réponse 503 + body `{ status: "degraded", db: false, version: "v0.1.0" }`.
- **Given** MariaDB joignable, **When** GET `/health`, **Then** 200 + body `{ status: "ok", db: true, version: "v0.1.0" }`.
- **Given** MariaDB stoppée pendant utilisation utilisateur, **When** appel API depuis page chargée, **Then** banner i18n s'affiche « Base de données temporairement indisponible — réessai automatique en cours » (FR) + équivalents DE/IT/EN.
- **Given** banner affiché, **When** MariaDB redémarre, **Then** banner disparaît automatiquement au prochain `/health` 200.
- **Given** `docker compose stop mariadb` (sur dev compose), **When** ouverture `http://localhost:3000` dans browser, **Then** SPA chargée + page de login visible + tentative login retourne erreur explicite (pas erreur technique brute).
- **And** page d'attente UX-DR43 conforme (pas d'erreur technique brute style « 500 Internal Server Error » au visage de l'utilisateur).
- **And** tests E2E Playwright : 3 scénarios (DB down at load / DB down mid-navigation / DB recovery).
- **And** **0 régression** sur baselines E2E existantes (~76 tests Playwright projet).

**Effort estimé :** 1-2 jours.

**Path-dependency :** indépendant. Peut démarrer en parallèle.

---

### Story 10-4 : Manuel install Synology + backup procédure DSM + CHANGELOG v0.1.0

**As a** administrateur Kesh non-expert Docker
**I want** un manuel d'installation pas-à-pas pour mon NAS Synology, avec procédure backup native DSM, et un CHANGELOG humain pour v0.1.0
**So that** je puisse installer + backuper Kesh sans assistance externe et comprendre les changements à chaque update

**Périmètre :** (scope PRD `epics.md` Story 9.4 — UX-DR44 + FR1)

1. **Manuel `install-synology.pdf`** (LaTeX, basé sur structure manuel admin PR #102) :
   - Section "Prérequis" : NAS Synology x86_64 + DSM 7.2+ + Container Manager installé + MariaDB 10.x Package Center installé + **réseau Docker externe créé** (e.g. `kesh-net`) auquel le service MariaDB et `kesh-api` seront connectés
   - Section "Installation MariaDB" : étape-par-étape pour créer DB `kesh` + user `kesh` + privilèges via phpMyAdmin DSM (screenshots ou texte précis)
   - Section "Réseau Docker externe" : commande `docker network create kesh-net` (ou nom au choix) si pas déjà fait + vérification connexion (DB visible depuis le réseau)
   - Section "Configuration" : `.env` à créer depuis `.env.example` avec génération secrets (`openssl rand -base64 32`) + `DATABASE_URL` pointant vers le hostname MariaDB sur `kesh-net`
   - Section "Premier login" : checklist post-install (`/health` OK, login admin, créer première company, vérifier audit_log)
   - Section "Backup" : configuration DSM Package Center MariaDB scheduled backup + DSM Hyper Backup sur le volume `/volume1/docker/kesh/` (config files + .env)
   - Section "Update" : pull nouvelle image Docker Hub + restart container + vérification migrations auto + vérification `_kesh_version`
   - Section "Troubleshooting" : port occupé, container ne démarre pas (logs Container Manager), DB unreachable
   - Avertissement explicite encadré rouge : **« v0.1.0 : HTTP only, LAN privé uniquement. Pas d'exposition internet sans reverse proxy TLS — feature v0.2+ »**
2. **`CHANGELOG.md`** (nouveau fichier racine) :
   - Entrée v0.1.0 humanisée (pas juste auto-générée par release.yml) : récap fonctionnel des Epics 1-10 livrés
   - Format Keep a Changelog (Added / Changed / Fixed / Security / Removed)
   - Bilingual : FR primaire + EN section pour future communauté
3. **Section "Backup" du manuel** :
   - Procédure backup `mariadb-dump` (CLI ou DSM Package Center)
   - Procédure restore : `docker compose down` + `mariadb < backup.sql` + `docker compose up -d`
   - Recommandation : backup hebdo automatique DSM + backup manuel avant chaque update
   - Note explicite : pas de backup automatique intégré image v0.1.0, c'est volontaire — DSM est la source unique de backup
4. **Livrable bundle release** :
   - `docker-compose.prod.yml`, `.env.example`, `install-synology.pdf`, `CHANGELOG.md` joints à la GitHub Release v0.1.0 (via release.yml `softprops/action-gh-release` artifacts)

**Critères d'acceptation :**

- **Given** `install-synology.pdf` finalisé (LaTeX build OK), **When** lecture par un admin Synology familier avec Container Manager mais pas avec Kesh, **Then** install complète possible en < 30 min sans assistance externe (test idéalement Guy lui-même sur son NAS).
- **Given** la section "Backup", **When** suivie, **Then** un backup MariaDB Kesh est planifié dans DSM Package Center + un Hyper Backup configuré sur le volume Docker.
- **Given** la section "Update", **When** suivie sur une install v0.1.0 → v0.1.1 simulée, **Then** migrations auto + `_kesh_version` updated + aucune perte de donnée.
- **Given** `CHANGELOG.md`, **When** lecture, **Then** v0.1.0 documenté avec récap fonctionnel humain (pas juste liste de PR titles).
- **Given** GitHub Release v0.1.0, **When** ouverte, **Then** `docker-compose.prod.yml`, `.env.example`, `install-synology.pdf`, `CHANGELOG.md` joints en assets.
- **And** manuel disponible en FR primaire. DE/IT/EN possibles v0.2+ (stubs LaTeX à créer si scope permet).

**Effort estimé :** 1 jour (s'appuie largement sur structure LaTeX existante manuel admin PR #102).

**Path-dependency :** **bloquant** par 10-1 (`.env.example` à finaliser) + 10-2 (procédure update à valider) + 10-5 (procédure premier login à ajuster si cookies httpOnly). Section "Réseau Docker externe" indépendante (décision D9, périmètre user-managed).

---

### Story 10-5 : httpOnly tokens (sécurité — Option A)

**As a** utilisateur Kesh
**I want** que mes tokens d'authentification soient inaccessibles au JavaScript (cookies httpOnly + Secure + SameSite=Strict)
**So that** une faille XSS éventuelle ne permette pas le vol immédiat de mon access_token / refresh_token

**Provenance :** GitHub Issue #41 [KF-002] (catégorie A confirmée pre-flight Epic 10) — checklist détaillée existante dans le body.

**Constat actuel** :
- `frontend/src/lib/app/stores/auth.svelte.ts:81-103` persiste access_token + refresh_token dans `window.localStorage`
- Aucune occurrence `Set-Cookie HttpOnly` ou `set_cookie` dans `crates/kesh-api/src/routes/auth.rs`
- Risque XSS = vol immédiat tokens depuis n'importe quel onglet malveillant

**Périmètre :**

1. **Backend `kesh-api/src/routes/auth.rs`** :
   - POST `/api/v1/auth/login` : au succès, émettre 2 cookies via `Set-Cookie` :
     - `kesh_access_token=<jwt>; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=900` (15 min)
     - `kesh_refresh_token=<token_uuid>; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=2592000` (30 jours)
   - POST `/api/v1/auth/refresh` : lire `kesh_refresh_token` cookie, re-émettre `kesh_access_token`
   - POST `/api/v1/auth/logout` : émettre cookies expirés (`Max-Age=0`) pour invalidation immédiate
   - GET `/api/v1/auth/me` (nouveau endpoint) : middleware lit `kesh_access_token` cookie, retourne `{ user_id, username, role, expires_in }` pour permettre au frontend de récupérer l'identité utilisateur sans lire les tokens
   - Middleware auth : préférer lecture cookie, fallback Authorization header (transition douce + tests existants intacts)
2. **Frontend `auth.svelte.ts`** :
   - Retirer tous les `localStorage.setItem(STORAGE_KEY_ACCESS_TOKEN, ...)` + `setItem(STORAGE_KEY_REFRESH_TOKEN, ...)`
   - Conserver `expiresIn` éventuellement en localStorage (pas sensible, juste pour UX refresh proactif) OU mieux : récupérer via `/api/v1/auth/me`
   - Retrait du payload `Authorization: Bearer <token>` côté fetch (browser envoie cookie automatiquement avec `credentials: 'include'`)
   - Au démarrage de l'app, fetch `/api/v1/auth/me` pour restaurer l'état auth (vs lecture localStorage actuelle ligne 137)
3. **CSP headers défensifs** (defense-in-depth même si XSS bloqué par httpOnly) :
   - Backend ajoute `Content-Security-Policy: default-src 'self'; script-src 'self'; ...` sur les réponses HTML
   - À finaliser selon ce que tolère SvelteKit (inline styles éventuels)
4. **Tests XSS** :
   - Test E2E Playwright : injecte un script malveillant simulé, tente `document.cookie` lecture, vérifie que les tokens httpOnly ne sont pas accessibles (test `expect(...).toBeUndefined()`).
5. **Migration douce ou breaking ?**
   - Breaking accepté : Guy = single user single NAS, aucun client en prod actuellement. Pas de session existante à préserver.
   - À documenter dans CHANGELOG.md (Security section) + manuel admin (section "Premier login" peut nécessiter ajustement si UI/UX login change).

**Critères d'acceptation :**

- **Given** POST `/api/v1/auth/login` avec credentials valides, **When** response, **Then** 2 headers `Set-Cookie` avec `HttpOnly; Secure; SameSite=Strict` sur `kesh_access_token` + `kesh_refresh_token`.
- **Given** `Set-Cookie kesh_access_token`, **When** inspection DevTools, **Then** flag HttpOnly présent et `document.cookie` JavaScript retourne chaîne vide ou ne contient pas le token.
- **Given** session frontend après login, **When** GET `/api/v1/auth/me`, **Then** retourne `{ user_id, username, role, expires_in }` sans avoir besoin d'`Authorization: Bearer` (cookie envoyé automatiquement).
- **Given** localStorage navigateur après login, **When** inspection `localStorage.getItem(STORAGE_KEY_ACCESS_TOKEN)`, **Then** retourne `null` ou `undefined` (plus de stockage).
- **Given** logout, **When** POST `/api/v1/auth/logout`, **Then** 2 `Set-Cookie ...; Max-Age=0` qui invalident les cookies + refresh_token révoqué en DB (mécanisme Story 1-6).
- **Given** test E2E Playwright `tests/e2e/security/xss_token_protection.spec.ts`, **When** exécuté, **Then** payload XSS simulé ne peut accéder aux tokens via `document.cookie` ou `localStorage`.
- **Given** CSP header présent, **When** inspection DevTools `network`, **Then** header `Content-Security-Policy: default-src 'self'; ...` sur réponses HTML.
- **And** **issue #41 fermée** avec `closes #41` dans commit final story.
- **And** **0 régression** sur tests auth existants (rate-limit, refresh, rotation, etc. — Story 1-5, 1-6).
- **And** CHANGELOG.md (Story 10-4) section Security documente le breaking change.

**Effort estimé :** 1-2 jours.

**Path-dependency :** indépendant des autres stories Epic 10 (mais 10-4 doit synchroniser sa section "Premier login" si l'UI/UX login change). Conseillé de faire 10-5 en parallèle de 10-1/10-2/10-3.

---

## Amélioration parallèle (hors story dédiée)

### `release.yml++` — Smoke test image post-build

**Owner :** Guy ou orchestrateur Claude.
**Justification :** actuellement `release.yml` build + push Docker Hub sans aucun test runtime sur l'image construite. Une image cassée (e.g. binaire qui crash au boot) peut être publiée + pull par Guy sur NAS = downtime non-détecté en CI.

**Périmètre :**

1. Ajouter un job `smoke-test` dans `release.yml` après le build, avant le push :
   - `docker compose -f docker-compose.smoke.yml up -d` (compose ad-hoc avec MariaDB 10.11 + image build artifact)
   - `wait-on http://localhost:3000/health` (timeout 60s)
   - `curl -f http://localhost:3000/health` → assertion 200 + body `{ status: "ok", db: true }`
   - POST `/api/v1/auth/login` avec admin/password seed → assertion 200 + cookies présents (test 10-5 cookie cooking)
   - `docker compose down`
2. Le push Docker Hub n'a lieu **que si** smoke-test PASS.
3. Si smoke-test FAIL, release.yml fail avec log explicite.

**Critères de succès** :
- `release.yml` exécuté sur tag git `v0.1.0-rc1` (test) : smoke-test PASS + image pushée Docker Hub `:v0.1.0-rc1`.
- `release.yml` exécuté avec une image volontairement cassée (test négatif) : smoke-test FAIL + image **non** pushée.

**Effort estimé :** 0.5 jour. Peut être ajouté en fin d'Epic 10 (avant le 1er tag git v0.1.0 réel) — pas critique de le faire en début.

---

## Critères d'arrêt Epic 10 (= release v0.1.0 prête)

Epic considéré « done » quand **toutes** les conditions ci-dessous sont satisfaites :

- [ ] 5/5 stories avec status `done` dans `sprint-status.yaml` (10-1, 10-2, 10-3, 10-4, 10-5)
- [ ] Issue #41 [KF-002] httpOnly token storage **fermée** sur GitHub (via Story 10-5)
- [ ] `release.yml++` smoke test implémenté et testé (au moins un tag de test `v0.1.0-rc1` PASS)
- [ ] Image Docker Hub `gcorbaz/kesh:v0.1.0` + `:latest` publiée (via tag git `v0.1.0`)
- [ ] Manuel `install-synology.pdf` finalisé + joint à la GitHub Release v0.1.0
- [ ] `CHANGELOG.md` v0.1.0 humanisé + joint à la GitHub Release v0.1.0
- [ ] **Install effective sur NAS Synology Guy** + login + création company + saisie d'1 écriture comptable + 1 facture → tout fonctionne (validation prod réelle)
- [ ] Backup natif DSM Package Center MariaDB testé sur le NAS de Guy (planifié + 1 backup manuel exécuté avec succès + restore validé sur DB de test)
- [x] Migrations idempotence + downgrade protection validés (test fresh install + test upgrade path PASS en CI) — Story 10-2 PR #106 verte 2026-05-22
- [x] CI matrice MariaDB 10.11 verte sur tous les tests Rust workspace — Story 10-2 PR #106 verte 2026-05-22 (mono-version 10.11, no matrice, cf. docs/ci.md §"Justification mono-version 10.11")
- [ ] Rétrospective Epic 10 produite (status `done` dans `sprint-status.yaml`)
- [ ] PR Epic 10 mergée sur `main` (pattern « avoid parallel PRs » memory `feedback_avoid_parallel_prs` — rétro incluse dans PR de la dernière story)
- [ ] **0 régression** sur baselines existantes : 250+ Vitest + cargo test workspace + 76 Playwright E2E

---

## Risques & questions ouvertes

| # | Risque / question | À traiter dans |
|---|---|---|
| Q1 | ~~Networking container~~ — **RÉSOLU** par décision D9 : Guy a déjà créé un réseau Docker externe sur son NAS. Le compose le déclare comme `external: true`, manuel 10-4 ne documente que la création du réseau côté NAS. | — (résolu pré-Epic 10) |
| Q2 | Build multi-arch arm64 — hors scope v0.1.0 par décision D8 (Synology x86_64 récent). Si Guy change de NAS un jour, ajouter `linux/arm64` à `platforms:` release.yml + test QEMU. À noter en dette catégorie B v0.2 si décision change. | Hors Epic 10 (dette v0.2+) |
| Q3 | Reverse proxy + HTTPS — dette catégorie B v0.2+ (décision D5). Story dédiée à créer au kickoff Epic v0.2 qui ouvre l'exposition WAN du NAS. | Hors Epic 10 (dette v0.2+) |
| Q4 | Audit log UI consultable — Story 3-5 a livré la persistence backend audit_log mais il n'y a pas de UI pour le consulter en v0.1. Manuel admin pourrait documenter requête SQL directe ou section "Consulter audit log via phpMyAdmin DSM" en attendant Epic v0.2 UI dédiée. | Story 10-4 (section éventuelle) |
| Q5 | ~~Reseed admin~~ — **RÉSOLU** par vérification code (décision D10) : `bootstrap.rs:39-47` est idempotent, `SELECT COUNT(*) FROM users` return early si users existent. 4 tests prouvent l'idempotence. Manuel 10-4 documente le comportement (env utilisé uniquement au 1er boot avec table vide). | — (résolu pré-Epic 10) |
| Q6 | ~~Breaking change cookies httpOnly~~ — **RÉSOLU** par décision D11 : pas d'autre install Kesh existante (Guy = single user pre-prod). Story 10-5 peut casser le flow auth localStorage → cookies sans préoccupation rétro-compat. | — (résolu pré-Epic 10) |
| Q7 | `_kesh_version.kesh_version_min_required` — comment lire la version du binaire au runtime ? `env!("CARGO_PKG_VERSION")` (standard Rust) à confirmer Story 10-2 spec validate. Backup : variable env `KESH_VERSION` injectée par le Dockerfile au build. | Story 10-2 spec validate |

---

## Références

- `_bmad-output/planning-artifacts/epics.md` lignes 1154-1215 — scope PRD originel "Epic 9 Déploiement & Opérations" (mapping ancien → courant ligne 25)
- `_bmad-output/planning-artifacts/prd.md` — FRs 1, 2, 3, 78, 79, 89 + UX-DR43, 44
- `_bmad-output/implementation-artifacts/epic-9-5-retro-2026-05-20.md` — action items pré-Epic 10 + retro Epic 9.5
- Memory `project_session_state_2026_05_20_end` — état pré-Epic 10 + 4 action items
- Memory `project_prod_deployment_gating` — révisée pre-flight 2026-05-21 (v0.1.0 = prod NAS Guy)
- Memory `feedback_avoid_parallel_prs` — PR retro groupée dans dernière story
- Memory `feedback_zero_tech_debt_carryforward` — politique projet
- `Dockerfile` — multi-stage Rust + Svelte build → debian:slim (à conserver)
- `docker-compose.yml` — dev compose, à aligner MariaDB 10.11 + dépublier de prod (création `docker-compose.prod.yml` séparé)
- `.github/workflows/release.yml` — pipeline Docker Hub `gcorbaz/kesh` sur tag `v*.*.*` (à enrichir smoke test)
- `.github/workflows/ci.yml` — CI MariaDB 10.11 to align (décision D3)
- `crates/kesh-db/migrations/*.sql` — 26 migrations à auditer pour idempotence (Story 10-2)
- `crates/kesh-api/src/routes/auth.rs` — refonte cookies httpOnly (Story 10-5)
- `frontend/src/lib/app/stores/auth.svelte.ts` — retrait localStorage tokens (Story 10-5)
- GitHub Issue #41 [KF-002] — httpOnly token storage cat A à fermer Story 10-5
- PR #102 mergée `47cb02f` — manuels LaTeX FR (admin/user/marketing), base éditoriale pour `install-synology.pdf` Story 10-4
