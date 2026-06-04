---
epic: 17
title: "Infra & Souveraineté"
version: v0.2
status: planning
sourceArtifact: Session scoping v0.2 2026-06-04 (post-release v0.1.8) — décision Guy via AskUserQuestion (E17 infra en tête d'exécution + E16 facturation dédié)
relatedIssues:
  - "#133 [v0.2] TOCTOU race on POST /setup/admin allows 2 concurrent admins (v0.2-milestone, sécurité)"
  - "#100 [CR] API externes avec clé PAT (lecture / lecture-écriture) pour intégrations IA & tiers (v0.2-milestone)"
  - "#112 [feature] Export/import complet d'une installation Kesh via interface admin (v0.2-milestone)"
  - "#122 [feature] Recovery mot de passe production-grade (forgot-password email + alternatives) (v0.2-milestone)"
relatedDecisions:
  - "v0.2 démarre par l'infra (E17) avant les features métier comptables (E11 TVA, etc.) — solidifier souveraineté + sécurité avant de bâtir dessus"
  - "Numérotation Epic = identifiant de domaine ; ordre d'exécution indépendant. E17 numéroté après E15 mais exécuté en premier de v0.2 (pas de renumérotation TVA/Avoirs/... dans le PRD)"
  - "Cluster facturation (#144/#151/#152) sorti dans un Epic dédié E16 — PAS dans E17 (domaine métier facture distinct de l'infra)"
  - "#100/#112/#122 sont chacune quasi de taille Epic — splitting préventif quasi-certain au bmad-create-story (cf. CLAUDE.md §Règle de splitting préventif)"
crates:
  - kesh-api (middleware auth PAT + table api_keys ; endpoints admin full-export/full-import ; endpoints forgot-password + reset ; fix TOCTOU setup/admin via lock ; audit-trail actor_type='api_key')
  - kesh-db (migrations api_keys, password_reset_tokens, users.email ; row sentinelle / advisory lock pour sérialiser setup)
  - frontend (page /settings/api-keys ; page /admin/backup + /admin/restore ; page publique /reset-password + champ email onboarding/users)
  - kesh-i18n (templates email FR/DE/IT/EN forgot-password ; i18n pages API keys / backup / restore)
  - infra (env vars SMTP KESH_SMTP_* + KESH_FEATURE_FORGOT_PASSWORD ; doc .env.example)
  - docs (docs/api-external.md + manuel admin §migration/restauration + §recovery mot de passe + CHANGELOG)
stories:
  - 17-1-fix-toctou-setup-admin
  - 17-2-api-pat-integrations (split quasi-certain)
  - 17-3-export-import-installation (split quasi-certain)
  - 17-4-recovery-mot-de-passe (split quasi-certain)
---

# Epic 17 — Infra & Souveraineté

## Vue d'ensemble

**Objectif :** Solidifier les fondations d'infrastructure, de sécurité et de souveraineté des données de Kesh avant d'attaquer les features métier comptables de v0.2 (TVA, avoirs, budgets, clôture). Quatre axes : (1) fermer une race condition de sécurité connue, (2) ouvrir Kesh aux intégrations externes via API à clé, (3) garantir la portabilité/sauvegarde complète d'une installation, (4) offrir un recovery de mot de passe production-grade.

**Pourquoi en premier dans v0.2 :** décision de scoping 2026-06-04 — bâtir les features métier sur une base où la sécurité (#133), l'extensibilité (#100) et la continuité des données (#112/#122) sont solides évite d'avoir à re-traverser tout le code métier ultérieurement. C'est l'inverse du « feature first, infra later » qui accumule la dette.

**Périmètre :** 4 stories (dont 3 quasi-certainement splittées en sous-stories au `bmad-create-story`). Aucune feature comptable nouvelle.

**Hors scope E17 (reporté v0.2+ / v0.3) :**
- **Fine-grained permissions PAT** par ressource (`invoices:read`, …) — scope binaire global read / read-write seulement (#100 §2).
- **OAuth / SSO** (Google, Authentik, Keycloak) — story séparée éventuelle (#122 Alt 1).
- **2FA TOTP** (#122 §5) — souhaitable mais optionnel, candidat sous-story à arbitrer ou report.
- **SMS / security questions recovery** — explicitement écartés (#122 Alt 3/4).
- **Webhooks / MCP server Kesh-natif** — Epic « Intégrations & Écosystème » futur éventuel (#100 alternative).
- **Chiffrement du fichier d'export** par défaut — responsabilité utilisateur, doc GPG/age (#112 §Sécurité).
- **Rate-limiting per-clé PAT** — pas en MVP, à ajouter si abus observés (#100 §7).

**Provenance :**
- Session scoping v0.2 2026-06-04 (post-release v0.1.8) — Guy acte E17 infra en tête + E16 facturation dédié via AskUserQuestion.
- Issues backlog GitHub déjà labellisées `v0.2-milestone` : #133, #100, #112, #122.

**Dépendances amont :**
- v0.1.8 publiée (Docker Hub `gcorbaz/kesh:0.1.8`, 2026-06-04) — base prod stable.
- Dette catégorie A levée (#140 fiscal_year, v0.1.7) — zero carry-forward respecté, v0.2 débloqué.
- Story 10-2 (`_kesh_version` + downgrade protection) — réutilisée par #112 (version compat check à l'import) et #133 (row sentinelle candidate pour le lock).
- Story 10-5 (tokens httpOnly + refresh tokens) — base auth réutilisée par #122 (revoke refresh tokens au reset).
- Story v011-5 (onboarding self-service `POST /setup/admin`) — #133 corrige sa dette L1 ; #121 break-glass `KESH_ADMIN_RESET` = fallback offline de #122.

**Dépendances aval (E11 TVA & suivants) :** E11 ne démarre pas tant qu'E17 n'est pas mergé sur `main`. Le fix TOCTOU (#133) et la base API/recovery durcissent la surface avant l'arrivée du code métier TVA.

---

## Décisions clés posées au scoping 2026-06-04

| # | Décision | Implication |
|---|---|---|
| D1 | E17 (infra) exécuté **en premier** de v0.2, avant E11 TVA | Roadmap README mise à jour : ordre d'exécution explicite. Numéro 17 = identifiant domaine, pas ordre |
| D2 | Cluster facturation (#144/#151/#152) → **E16 dédié**, hors E17 | E17 reste strictement infra/sécurité/souveraineté |
| D3 | #100/#112/#122 quasi-Epic chacune → **splitting préventif** attendu au `create-story` | 17-2/17-3/17-4 produiront probablement des sous-stories (17-3a backend export, 17-3b UI export, 17-3c backend import, …). Décision finale au validate |
| D4 | Auth PAT = **Option A** (réutiliser routes `/api/v1/*`, middleware accepte cookie session OU PAT Bearer) | Pas de duplication de routes `/external/*`. Middleware factorisé (#100 §5 reco) |
| D5 | PAT scope = **binaire global** read / read-write, **1 company_id par clé** | Pas de fine-grained v0.2. Cohérent multi-tenant scoping Epic 7-1 KF-002 (#100 §2/§3) |
| D6 | Export/import = format unifié auto-portant (#112 Alt 2 écartée), **pas** de chevauchement avec export per-company Story 9-2b | 9-2b reste pour users non-Admin par-company ; #112 = Admin installation complète |
| D7 | Import = **backup automatique pré-import** (rollback safety net) + validation version compat (réutilise `_kesh_version` Story 10-2) | Refus si downgrade impossible. Modal confirmation forte UI |
| D8 | Recovery = **forgot-password email** (primary) + **break-glass `.env` #121** conservé en fallback offline | Feature désactivable `KESH_FEATURE_FORGOT_PASSWORD=false` pour déploiements sans SMTP |
| D9 | Champ **`email` sur users** (migration nullable backward-compatible) | Onboarding admin + page `/users` éditent l'email. Forgot-password matche `email OR username` |
| D10 | Fix TOCTOU #133 = **story-zéro** (petit, sécurise la base) en premier de l'Epic | Lock applicatif (`SELECT … FOR UPDATE` sur row sentinelle, ou `GET_LOCK` MySQL) autour du check+INSERT |
| D11 | 2FA TOTP (#122 §5) = **arbitrage à la spec 17-4** — inclus en sous-story ou reporté v0.3 | Ne pas bloquer le forgot-password core sur le 2FA |

---

## Stories

### Story 17-1 : Fix sécurité TOCTOU `POST /setup/admin` (#133)

**As a** opérateur d'une instance Kesh exposée
**I want** que la création du 1er admin soit atomique (impossible de créer 2 admins concurrents)
**So that** une race condition ne puisse pas compromettre le bootstrap de sécurité de l'installation

**Périmètre :**
- Sérialiser le check `user_count == 0` + `INSERT INTO users` dans une transaction avec lock : `SELECT … FOR UPDATE` sur une row sentinelle (candidate : `_kesh_version` Story 10-2) **ou** advisory lock MySQL `GET_LOCK('setup_admin', 5)`.
- Conserver le gate auto-disable `user_count > 0 → 410 Gone` et le rate-limit IP existants (défense en profondeur).
- Test : 2 requêtes concurrentes usernames distincts → exactement 1 admin créé, l'autre `410`/conflit. Convertir le commentaire du test E2E `setup_admin_e2e.rs` (AC #22) en assertion réelle.

**Notes :** story-zéro, petite (< 1 module métier), sécurise la base avant les grosses stories. Pas de splitting attendu.

---

### Story 17-2 : API externe à clé PAT pour intégrations (#100) — *split quasi-certain*

**As a** PME utilisant Kesh
**I want** générer des clés d'accès API (lecture ou lecture-écriture) liées à une company
**So that** une IA externe ou un logiciel tiers puisse consommer mes données comptables sans partager mes identifiants utilisateur

**Périmètre (D4/D5) :**
- Table DB `api_keys` (hash Argon2id de la clé, scope `read`/`read-write`, `company_id`, `last_used_at`, expiration optionnelle, FK user créateur).
- Format clé `kesh_pat_<base62>` affichée **une seule fois** à la création (seul le hash stocké).
- Middleware auth factorisé : accepte cookie session JWT (UI) **OU** `Authorization: Bearer kesh_pat_…` (API), route selon header présent — réutilise les routes `/api/v1/*` existantes (Option A).
- RBAC scope : `read` → GET seulement ; `read-write` → GET+POST+PUT+PATCH+DELETE.
- Page `/settings/api-keys` (per-company) : liste, création (modal nom+scope+expiration), révocation.
- Audit-trail : `actor_type='api_key'`, `actor_api_key_id`, `actor_user_id` conservé (OLICo Art. 9).
- Doc `docs/api-external.md` (curl/Python/JS/MCP) + OpenAPI (`utoipa`) si faisable.

**Split pressenti :** 17-2a backend (table + middleware + CRUD + audit), 17-2b frontend (page settings), 17-2c doc/OpenAPI. Décision au `create-story validate`.

---

### Story 17-3 : Export/import complet d'installation (#112) — *split quasi-certain*

**As a** administrateur Kesh
**I want** exporter toute l'installation dans un fichier unique et le réimporter sur une autre instance via l'UI admin
**So that** je puisse migrer ou sauvegarder mes données sans accès SSH/Docker

**Périmètre (D6/D7) :**
- `POST /api/v1/admin/full-export` (RBAC Admin) → ZIP/`.keshbackup` : dump SQL portable (tables Kesh uniquement) + binaires uploads + métadonnées (version, date, SHA-256, `_kesh_version`) + audit_log préservé. **Streaming** progressif.
- `POST /api/v1/admin/full-import` (RBAC Admin) : upload multipart, validation version compat (réutilise downgrade-protection Story 10-2) + SHA-256, **backup auto pré-import** (rollback), truncate+restore tables Kesh + binaires, re-run migrations idempotentes.
- UI `/admin/backup` (bouton export + progress + download) et `/admin/restore` (upload + modal confirmation forte + progress).
- Test E2E double instance Docker Compose : export A → import B → équivalence fonctionnelle.

**Split pressenti (cf. #112 sous-stories A–F) :** 17-3a backend export, 17-3b UI export, 17-3c backend import, 17-3d UI import, 17-3e E2E double-instance, 17-3f doc. Décision au validate.

---

### Story 17-4 : Recovery mot de passe production-grade (#122) — *split quasi-certain*

**As a** utilisateur Kesh ayant oublié son mot de passe
**I want** un flux self-service de réinitialisation par email (magic link)
**So that** je récupère l'accès sans intervention SSH/admin système

**Périmètre (D8/D9/D11) :**
- `POST /api/v1/auth/forgot-password` ({username|email}) → token magic-link (UUID v4, single-use, TTL 30 min, table `password_reset_tokens`) → email lien.
- Page publique `/reset-password?token=…` : valide token + nouveau mdp + revoke refresh tokens + audit log.
- Config SMTP (`KESH_SMTP_*`) + fail-fast si feature activée mais SMTP incomplet ; `KESH_FEATURE_FORGOT_PASSWORD=false` → fallback break-glass #121.
- Migration `users.email` nullable + champ onboarding admin + édition page `/users`.
- Templates email i18n FR/DE/IT/EN. Rate-limit anti-énumération + throttling brute-force tokens.
- **2FA TOTP (D11)** : arbitrage à la spec — sous-story dédiée ou report v0.3.

**Split pressenti :** 17-4a backend forgot/reset + SMTP + migration email, 17-4b frontend reset + champ email, 17-4c templates i18n + doc, (17-4d 2FA TOTP optionnel). Décision au validate.

---

## Risques & points d'attention

- **Taille E17** : 3 stories quasi-Epic. Surveiller la règle de splitting préventif (> 5 modules OU > 4 passes validate → split). Le découpage en sous-stories story-zéro (pattern) + rollout est privilégié.
- **#112 import = opération destructrice** (truncate + restore). Le backup auto pré-import (D7) est non-négociable. Le smoke-test E2E double-instance est le garde-fou de régression.
- **Migration breaking ?** `users.email ADD COLUMN nullable` = non-breaking (cf. CLAUDE.md §Migration breaking policy P1). `password_reset_tokens` / `api_keys` = `CREATE TABLE` non-breaking. Aucun bump `kesh_version_min_required` attendu — à confirmer story par story. Chaque nouveau `.sql` → ligne `docs/migrations-idempotence-audit.md` (P5).
- **Dépendance SMTP côté utilisateur** (#122) : la feature doit dégrader proprement (désactivable) pour les déploiements offline NAS.
- **Audit-trail OLICo** : PAT (#100) et export/import (#112) doivent loguer dans `audit_log` (conformité Art. 9).
