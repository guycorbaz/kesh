# Story 17.2c: API externe à clé PAT — Documentation (`docs/api-external.md` + synchro docs visibles)

Status: review

<!-- Issue de la scission de la story 17-2 (spec convergée 5 passes validate, 2026-06-04). 17-2c = Partie C documentation. Dépend de 17-2a (backend, comportement auth final figé, MERGÉE PR #168) et 17-2b (frontend, page /settings/api-keys, MERGÉE PR #169). Voir 17-2-api-pat-integrations.md pour le contexte parent complet. -->

## Story

As a **développeur/intégrateur tiers (ou IA externe) consommant l'API de Kesh**,
I want **une documentation d'usage claire de l'API externe à clé PAT — authentification Bearer, scopes, scoping company, gestion des clés, exemples curl/Python/JS/MCP**,
so that **je puisse brancher un script, un agent IA ou un logiciel tiers sur mes données comptables sans deviner le contrat HTTP ni lire le code source Rust**.

## Contexte & provenance

- **Issue GitHub** : [#100](https://github.com/guycorbaz/kesh/issues/100) `[CR] API externes avec clé PAT (lecture / lecture-écriture) pour intégrations IA & logiciels tiers` — `enhancement` + `v0.2-milestone`. **17-2c est la dernière sous-story qui ferme #100.**
- **Epic** : 17 « Infra & Souveraineté » — Story 17-2 (cf. `_bmad-output/planning-artifacts/epic-17.md` décisions D4/D5).
- **Scission** : cette story **17-2c** est la **Partie C documentation** issue du split de **17-2** (spec parente `17-2-api-pat-integrations.md`, convergée 5 passes validate). Elle vient **après** :
  - **17-2a backend** (MERGÉE PR #168, squash sur `main`) — fige le comportement auth final (Bearer PAT, scopes, codes 403, routes CRUD).
  - **17-2b frontend** (MERGÉE PR #169, squash sur `main`) — page `/settings/api-keys` (création/affichage one-time/révocation).
- **Nature de la story** : **doc-only**. Aucun code Rust/Svelte exécutable n'est touché. Concrètement : (1) **création** de `docs/api-external.md`, (2) **synchro** des supports doc visibles (CHANGELOG, README, manuel admin LaTeX FR).

> ⚠️ **Story doc-only — implications process** :
> - **Pas de quality gate Rust/Svelte** (cf. CLAUDE.md §« Quand sauter » la règle Test Locally First : les commits doc-only ne touchent pas de code exécutable). **Exception** : si T-C2 décide d'annoter des routes avec `utoipa` (déconseillé, cf. AC2), la quality gate backend redevient obligatoire.
> - **Régénération du PDF du manuel admin obligatoire** si le `.tex` est modifié (convention projet : les PDF sont versionnés, cf. PR #102). `latexmk -xelatex` est disponible localement (`/usr/bin/latexmk`, `/usr/bin/xelatex`).
> - **Tout dans la même PR** (CLAUDE.md §« Règle d'inclusion ») : `docs/api-external.md` + synchros CHANGELOG/README/manuel dans un seul commit/PR cohérent.

## Décisions de conception (héritées, déjà tranchées — ne PAS rouvrir)

Reprises de la spec parente convergée + des décisions code-review 17-2a/17-2b. La doc doit refléter **exactement** ces comportements (ground-truth code, pas le wording de la spec) :

- **Auth = Option A** : pas de routes `/external/*`. Les routes `/api/v1/*` existantes acceptent SOIT le cookie session JWT (UI web) SOIT `Authorization: Bearer kesh_pat_…` (API externe).
- **Scope binaire global** : `read` (GET/HEAD/OPTIONS uniquement) | `read-write` (toutes méthodes, **sous réserve du RBAC rôle du créateur**). Pas de fine-grained per-ressource en v0.2.
- **1 `company_id` par clé** : toute requête PAT est scopée à la company de la clé. Jamais d'accès cross-company.
- **DC6 — gestion des clés interdite via PAT** : les routes `/api/v1/settings/api-keys` ne sont accessibles **qu'en session JWT cookie (UI web)**. Un PAT (même `read-write`) → `403 API_KEY_MANAGEMENT_FORBIDDEN`. **Documenter explicitement** : on crée/révoque les clés dans l'UI web, pas via l'API.
- **Limitation L3 / KF-036 ([#167](https://github.com/guycorbaz/kesh/issues/167), v0.3)** : un PAT `read-write` créé par un **Admin** peut atteindre les routes `require_admin_role` (`/api/v1/users` CRUD/reset-password, `/api/v1/company/invoice-settings`) — auto-propagation partielle contournant l'esprit de DC6. **À documenter comme avertissement de sécurité** : « créez des clés avec le rôle minimal nécessaire ; un PAT créé par un Admin hérite des pouvoirs Admin ».

## Acceptance Criteria

### AC1 — `docs/api-external.md` (T-C1) — le livrable central

Créer `docs/api-external.md` (français, registre développeur/intégrateur, cohérent avec les autres docs `docs/*.md`). Le document **DOIT** couvrir, avec le contrat **ground-truth** ci-dessous (cf. §Dev Notes pour les valeurs exactes vérifiées dans le code) :

1. **Vue d'ensemble & cas d'usage** : à quoi sert l'API PAT (IA externe, scripts, ETL, BI, ERP), et le modèle « une clé = une company, au nom de son créateur ».
2. **Authentification** : header `Authorization: Bearer kesh_pat_<…>`. Format du token : préfixe `kesh_pat_` + 27 caractères base62 (`0-9A-Za-z`), longueur totale **36 caractères**. Mentionner que le secret est **affiché une seule fois** à la création (jamais re-récupérable) et que seul un `SHA-256` est stocké côté serveur.
3. **Création & gestion des clés** : se fait **dans l'UI web** (`/settings/api-keys`, accessible depuis `/settings`) — **pas** via l'API (DC6). Décrire le flux UI : créer (nom + scope + expiration optionnelle) → copier la clé une fois → révoquer.
4. **Scopes** : tableau `read` vs `read-write` avec les méthodes HTTP autorisées. Préciser que la permission effective = **intersection(rôle du créateur, scope de la clé)** : un PAT `read-write` dont le créateur est `Comptable` ne peut pas faire ce qu'un Admin ferait.
5. **Scoping company** : la clé ne voit que les données de sa company. Pas de paramètre company à passer — il est dérivé de la clé.
6. **URL de base** : `http(s)://<hôte>:<port>` + préfixe `/api/v1`. Expliquer que l'hôte/port dépend du déploiement (cf. manuel admin §ports). Exemple générique `https://kesh.example.ch/api/v1`.
7. **Exemples fonctionnels** sur des endpoints réels existants (cf. liste ground-truth Dev Notes) — un GET (lecture) et un POST (écriture, scope `read-write`) :
   - **`curl`** (lecture + écriture).
   - **Python** (`requests`).
   - **JavaScript** (`fetch`).
   - **Config MCP / agent IA** : comment câbler l'en-tête `Authorization: Bearer kesh_pat_…` dans un client MCP HTTP générique ou un agent (l'API REST est consommable par tout client HTTP ; il n'existe **pas** de serveur MCP Kesh-natif en v0.2 — le documenter comme tel, cf. epic-17 hors-scope « Webhooks / MCP server Kesh-natif »).
8. **Gestion des erreurs** : format JSON `{ "error": { "code": "…", "message": "…" } }` + tableau des codes pertinents pour un consommateur API (cf. ground-truth Dev Notes) : `401 UNAUTHENTICATED`, `403 API_KEY_READ_ONLY`, `403 API_KEY_MANAGEMENT_FORBIDDEN`, `400 VALIDATION_ERROR`, `404`.
9. **Sécurité & bonnes pratiques** : ne jamais committer une clé ; préférer une **expiration** ; révoquer en cas de fuite (effet immédiat) ; **principe du moindre privilège** (rôle du créateur minimal + scope `read` si lecture seule suffit) ; avertissement **L3/KF-036** (un PAT créé par un Admin hérite des pouvoirs Admin).
10. **Limitations v0.2 (section dédiée, tracée)** : scope binaire global (pas de fine-grained per-ressource, #100 §2) ; **pas de rate-limiting per-clé** (#100 §7, risque clé permanente) ; gestion des clés réservée à l'UI (DC6) ; **pas de spec OpenAPI** (cf. AC2, v0.3) ; auto-propagation Admin (L3/KF-036). Chaque limitation pointe vers son issue/numéro de suivi.

### AC2 — OpenAPI `utoipa` : évaluation et décision (T-C2)

`utoipa` est **absent du workspace** (vérifié ground-truth : aucune occurrence dans `Cargo.toml` ni `crates/*/Cargo.toml`). Annoter rétroactivement les ~60 routes `/api/v1/*` avec `utoipa` est un refactor lourd hors-scope d'une story doc. **Décision attendue (alignée sur AC12 parent « OpenAPI si faisable sans refactor lourd — sinon limitation v0.3 »)** : **documenter l'absence d'OpenAPI comme limitation v0.3** dans `docs/api-external.md` (§Limitations) **plutôt que** forcer l'annotation. Si le dev-story juge malgré tout un sous-ensemble faisable (ex. documenter manuellement un mini-schéma OpenAPI YAML statique des seules routes externes courantes, sans dépendance `utoipa`), c'est acceptable **mais** ne doit pas réintroduire de code Rust ni casser la quality gate. **Par défaut : pas d'OpenAPI, limitation documentée.**

### AC3 — Synchro des docs visibles (T-C3, CLAUDE.md §Synchroniser TOUTES les docs)

1. **CHANGELOG.md** — sous `## [Non publié]`, ajouter une entrée **`### Added`** « API externe à clé PAT (#100) » décrivant la feature côté utilisateur (clés API read/read-write par company, page `/settings/api-keys`, auth Bearer, doc `docs/api-external.md`). Registre : fiduciaires/PME (pas de jargon Rust). La section `### Sécurité` existante (#133) reste ; ajouter `### Added` au-dessus ou en cohérence avec l'ordre Keep a Changelog (`Added` avant `Security`).
2. **README.md** :
   - **§Fonctionnalités** : ajouter une puce « **API externe à clé PAT** — clés d'accès read/read-write par company pour intégrations IA & logiciels tiers (auth Bearer) ✓ ». **Ne PAS** mettre `(à venir)` (la feature est livrée).
   - **§Feuille de route** : la ligne v0.2 mentionne déjà « E17 Infra & Souveraineté (API PAT, …) ». Vérifier qu'elle reste cohérente — pas de changement de statut requis (E17 toujours en cours, 📋/🚧). Ne PAS marquer E17 done (les autres stories 17-3/17-4 restent).
3. **Manuel admin LaTeX FR** (`docs/manual/fr/admin-manual.tex`) — ajouter une **`\subsection{Clés API (PAT) — accès programmatique}`** dans la **`\section{Sécurité}`** (insérer logiquement après `\subsection{Authentification JWT + refresh tokens}` ligne ~1297, ou après RBAC ~1308). Contenu admin : que sont les PAT, où les gérer (`/settings/api-keys`), scopes, scoping company, avertissement de sécurité L3 (PAT Admin = pouvoirs Admin), renvoi vers `docs/api-external.md` pour le détail intégrateur. **Régénérer le PDF** (`cd docs/manual/fr && latexmk -xelatex admin-manual.tex`) et **committer le `.pdf`** (convention projet PR #102).
   - *(Le manuel **user** `user-manual.tex` est destiné aux fiduciaires/comptables ; l'API PAT est une feature technique/admin → couverte par le manuel admin, pas le user. Ne pas toucher le user-manual sauf si une section « intégrations » existante l'appelle.)*

### AC4 — Cohérence ground-truth (anti-régression doc)

Toutes les valeurs techniques de `docs/api-external.md` (routes, méthodes, codes d'erreur, format token, shapes JSON) **DOIVENT** correspondre au code réellement mergé (17-2a/17-2b sur `main`), **pas** au wording de la spec parente. Le §Dev Notes ci-dessous fournit le contrat vérifié par `grep`/`Read` sur le code source — l'utiliser comme source de vérité. Toute divergence constatée entre la spec et le code → le **code** prime.

## Tasks / Subtasks

- [x] **T-C1 — `docs/api-external.md`** (AC: #1, #4)
  - [x] Créer `docs/api-external.md` avec les 10 sections d'AC1. Utiliser le contrat ground-truth du §Dev Notes (token `kesh_pat_`+27 base62 = 36 chars ; routes ; codes erreur ; shapes JSON).
  - [x] Exemples curl/Python/JS sur des endpoints réels (cf. liste Dev Notes) : un GET lecture (`GET /api/v1/contacts`) et un POST écriture read-write (`POST /api/v1/contacts`). Payloads vérifiés contre `routes/contacts.rs` (`CreateContactRequest` camelCase + `ContactType` PascalCase `Entreprise`).
  - [x] Section MCP/agent IA : câblage générique de l'en-tête Bearer dans un client HTTP MCP ; absence de serveur MCP Kesh-natif en v0.2 précisée.
  - [x] Section Limitations v0.2 (scope global, pas de rate-limit, gestion UI-only DC6, pas d'OpenAPI, L3/KF-036) avec liens issues.
- [x] **T-C2 — OpenAPI `utoipa` (évaluation)** (AC: #2)
  - [x] `utoipa` confirmé absent (`grep` workspace). Absence d'OpenAPI documentée en limitation v0.3 dans `docs/api-external.md` §9. Aucune crate ajoutée, aucune route annotée (décision par défaut).
- [x] **T-C3 — Synchro docs visibles** (AC: #3)
  - [x] CHANGELOG.md : entrée `### Added` API PAT sous `## [Non publié]` (ordre Keep a Changelog : `Added` avant `Sécurité`).
  - [x] README.md : puce §Fonctionnalités (sans `(à venir)`) ; §Feuille de route v0.2 passée `📋 Backlog → 🚧 En cours` (E17 a 3 stories mergées, anti-dérive planning ; E17 PAS marqué done car 17-3/17-4 restent).
  - [x] Manuel admin `.tex` : `\subsection{Clés API (PAT)}` dans §Sécurité (après JWT) + `\begin{keshwarning}` (env correct du shared style) + **PDF régénéré** (`latexmk -xelatex`, exit 0, 52 p.) + `.pdf`/aux versionnés (convention repo PR #102).
- [x] **T-C4 — Fermeture #100** (AC: #1)
  - [x] Le commit/PR finale mentionne `closes #100` (17-2c = dernière sous-story de 17-2). Reliquats #100 couverts : auth ✓ 17-2a (#168), UI ✓ 17-2b (#169), doc ✓ 17-2c.

## Dev Notes

> **Source de vérité = le code mergé** (17-2a PR #168 + 17-2b PR #169 sur `main`). Les valeurs ci-dessous ont été vérifiées par `Read`/`grep` lors de la création de cette story. Le dev-story DOIT les réutiliser telles quelles dans `docs/api-external.md`.

### Contrat HTTP ground-truth (vérifié sur le code)

**Format du token** (`crates/kesh-api/src/auth/api_key.rs`) :
- `PAT_PREFIX = "kesh_pat_"` (case-sensitive exact).
- Corps = base62 (`0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz`) à **largeur fixe 27** (`PAT_BASE62_LEN = 27`), encodant 160 bits (`PAT_ENTROPY_BYTES = 20`).
- **Token complet = 36 caractères** (`kesh_pat_` = 9 + 27). Stockage serveur = `SHA-256(token)` hex 64 chars (jamais le clair).

**Routes de gestion** (`crates/kesh-api/src/routes/api_keys.rs`, montées dans `lib.rs` sous `comptable_routes` → guard `require_comptable_role` ; **DC6 : JWT cookie only**) :
- `GET /api/v1/settings/api-keys` → `200` `[ApiKeyResponse]` (toutes les clés, actives + révoquées, `created_at DESC`).
- `POST /api/v1/settings/api-keys` → `201` `CreateApiKeyResponse` (secret en clair **une fois**).
- `DELETE /api/v1/settings/api-keys/{id}` (body `{ "version": <i32> }`) → `204 No Content`.
- Un PAT sur l'une de ces routes → `403 API_KEY_MANAGEMENT_FORBIDDEN` (`ensure_not_pat`).

**Shapes JSON (camelCase, `#[serde(rename_all = "camelCase")]`)** :
- `ApiKeyResponse` : `{ id, name, scope, createdAt, lastUsedAt, revokedAt, expiresAt, version }` — **jamais** `keyHash` ni secret.
- `CreateApiKeyRequest` : `{ name, scope, expiresAt? }` — `expiresAt` RFC 3339 (ex. `"2027-01-01T00:00:00Z"`), optionnel.
- `CreateApiKeyResponse` : `{ id, name, scope, createdAt, key }` où `key = "kesh_pat_…"` (une seule fois).
- `RevokeApiKeyRequest` : `{ version }`.

**Format d'erreur** (`crates/kesh-api/src/errors.rs` `build_response`) :
- JSON : `{ "error": { "code": "<CODE>", "message": "<message localisé locale serveur>" } }`.
- Codes pertinents pour un consommateur API :
  | HTTP | code | Cause |
  |------|------|-------|
  | 401 | `UNAUTHENTICATED` | token absent / invalide / révoqué / expiré / créateur désactivé |
  | 403 | `API_KEY_READ_ONLY` | clé `read` + méthode mutante (POST/PUT/PATCH/DELETE) |
  | 403 | `API_KEY_MANAGEMENT_FORBIDDEN` | PAT tentant de gérer des clés (`/settings/api-keys`) |
  | 400 | `VALIDATION_ERROR` | nom vide / > 255 chars / scope invalide / `expiresAt` dans le passé |
  | 404 | (NotFound) | clé absente ou d'une autre company (anti-énumération) |
- i18n : le message est résolu sur la **locale serveur globale** (`init_error_i18n`), PAS sur l'`Accept-Language` du client API. Les 4 clés Fluent (`error-api-key-read-only`, `error-api-key-management-forbidden`) existent en FR/DE/IT/EN (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`).

**Endpoints `/api/v1/*` consommables (exemples réels, vérifiés `lib.rs`)** :
- Lecture (scope `read` suffit, tout rôle authentifié) : `GET /api/v1/accounts`, `/api/v1/contacts`, `/api/v1/contacts/{id}`, `/api/v1/products`, `/api/v1/products/{id}`, `/api/v1/invoices`, `/api/v1/invoices/{id}`, `/api/v1/journal-entries`, `/api/v1/journal-entries/{id}`, `/api/v1/vat-rates`, `/api/v1/auth/me`.
- Écriture (scope `read-write` requis + RBAC rôle) : `POST /api/v1/contacts`, `POST /api/v1/products`, `POST /api/v1/invoices`, `POST /api/v1/accounts`, etc. — **choisir un exemple simple et vérifier son payload** dans le handler correspondant avant de l'écrire dans la doc.
- ⚠️ Routes `require_admin_role` (`/api/v1/users`, `/api/v1/company/invoice-settings`) : atteignables par un PAT **read-write créé par un Admin** (L3/KF-036) — à mentionner comme avertissement, pas comme exemple recommandé.

### Comportement auth (rappel pour la doc)

- Le middleware `require_auth` route vers le chemin PAT **uniquement si** `token.starts_with("kesh_pat_")` (case-sensitive). Sinon → décodage JWT (cookie ou Bearer JWT). Un PAT valide produit le même `CurrentUser` (avec `api_key_id = Some(id)`) → aucune route ne change.
- Le PAT relit l'**état courant** du créateur en DB (rôle, actif) à chaque requête → une désactivation ou un changement de rôle du créateur prend effet **immédiatement** (≠ JWT qui fait confiance aux claims). À mentionner : « révoquer/désactiver le créateur invalide ses clés sur-le-champ ».
- Révocation d'une clé = effet **immédiat** (la requête suivante → 401).

### Audit-trail (pour la section sécurité/conformité, optionnel mais utile)

- Les mutations via PAT sont auditées `actor_type='api_key'` + `actor_api_key_id` sur les call-sites disposant du contexte d'acteur (handlers + helpers threadés, catégories i/ii de DC5). Certains chemins de lecture (`reports`) et les audits internes kesh-db (catégorie iii) restent `actor_type='user'` (limitations **L1/L2** v0.3). Imputabilité toujours préservée (`user_id` = créateur de la clé). À mentionner brièvement dans la doc conformité OLICo si pertinent — ne pas sur-détailler.

### Project Structure Notes

- `docs/api-external.md` = nouveau fichier, à la racine de `docs/` (cf. autres docs `docs/testing.md`, `docs/ci.md`, `docs/optimistic-locking-patterns.md`). Pas de sous-dossier.
- Pas de variance de structure. Story purement documentaire — aucun module de code touché (sauf décision T-C2 contraire, déconseillée).
- Le manuel admin a une `\section{Sécurité}` (ligne ~1272) avec déjà `\subsection{Authentification JWT + refresh tokens}` (~1297) — point d'insertion naturel de la nouvelle subsection PAT.

### Invariants / pièges à éviter

- **Ne PAS documenter de routes `/external/*`** : elles n'existent pas (Option A — routes `/api/v1/*` partagées).
- **Ne PAS prétendre** qu'on crée/révoque des clés via l'API : DC6 l'interdit (UI web only).
- **Ne PAS inventer** de pagination, de filtres, de champs de réponse non présents dans le code. Vérifier chaque exemple contre le handler réel.
- **Ne PAS marquer E17 done** dans le README (17-3/17-4 restent).
- **Régénérer le PDF** du manuel si le `.tex` change (sinon PDF versionné incohérent).
- **doc-only** : pas de quality gate Rust/Svelte attendue (sauf si T-C2 réintroduit du code, à éviter).

### References

- [Source: _bmad-output/implementation-artifacts/17-2-api-pat-integrations.md#AC12 + T-C1..T-C3] — périmètre doc de la spec parente convergée.
- [Source: _bmad-output/implementation-artifacts/17-2a-api-pat-backend.md] — contrat backend final (DC1-DC6, codes erreur, audit, L1/L2/L3).
- [Source: crates/kesh-api/src/auth/api_key.rs] — format token `kesh_pat_`+27 base62, SHA-256, `validate_pat`.
- [Source: crates/kesh-api/src/routes/api_keys.rs] — routes CRUD, shapes JSON, `ensure_not_pat` (DC6).
- [Source: crates/kesh-api/src/errors.rs:629-705] — `build_response` (shape `{error:{code,message}}`), codes `API_KEY_READ_ONLY`/`API_KEY_MANAGEMENT_FORBIDDEN`/`UNAUTHENTICATED`/`VALIDATION_ERROR`.
- [Source: crates/kesh-api/src/lib.rs:131-488] — liste des routes `/api/v1/*` (exemples GET/POST).
- [Source: crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl:11-12] — i18n des 2 codes 403 (4 langues).
- [Source: docs/manual/fr/admin-manual.tex §Sécurité ~1272-1336] — point d'insertion subsection PAT.
- [Source: CHANGELOG.md §[Non publié] + README.md §Fonctionnalités/§Feuille de route] — supports à synchroniser.
- [Source: GitHub #100] — issue d'origine à fermer ; KF-036 #167 (L3 v0.3).
- [Source: CLAUDE.md §Synchroniser TOUTES les docs, §« Quand sauter » Test Locally First, §Issue Tracking Rule] — contraintes process.

## Change Log — split

**Créée 2026-06-07 par scission de la story 17-2** (`bmad-create-story 17-2c`). 17-2c = **Partie C documentation** (T-C1..T-C3 + fermeture #100), extraite de la spec parente `17-2-api-pat-integrations.md` (convergée 5 passes validate). Le contenu doc dépend du comportement backend final figé par 17-2a (MERGÉE #168) et de l'UI livrée par 17-2b (MERGÉE #169) — le contrat ground-truth du §Dev Notes a été re-vérifié sur le code mergé sur `main` (`34f830d`), pas sur le wording de spec.

- **Scope conservé (17-2c)** : AC12 parent (`docs/api-external.md`, OpenAPI éval), T-C1/T-C2/T-C3, synchro CHANGELOG/README/manuel admin, fermeture #100.
- **Re-validate** : optionnel. Story doc-only à faible risque adversarial ; le contrat technique est ground-truthé sur le code mergé. Un `bmad-create-story validate 17-2c` reste possible si l'on veut challenger l'exhaustivité des sections doc.

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 (1M context) — `bmad-dev-story 17-2c`, single-pass T-C1→T-C4 (2026-06-07). Story doc-only.

### Debug Log References

- `latexmk -xelatex admin-manual.tex` : **exit 0**, PDF 52 pages (234 474 octets). Warnings « Missing character » (✓ U+2713, ═ U+2550) **préexistants** (utilisés ailleurs dans le doc, hors de l'ajout PAT) — non bloquants.
- Ground-truth vérifié avant rédaction (pas d'hallucination) :
  - token `kesh_pat_`+27 base62 = 36 chars (`auth/api_key.rs` `PAT_PREFIX`/`PAT_BASE62_LEN`).
  - routes/shapes (`routes/api_keys.rs`), codes erreur (`errors.rs` : `UNAUTHENTICATED`/`API_KEY_READ_ONLY`/`API_KEY_MANAGEMENT_FORBIDDEN`/`VALIDATION_ERROR`).
  - exemple contacts : `CreateContactRequest` camelCase + `ContactType` PascalCase `Entreprise` (`routes/contacts.rs` + `entities/contact.rs`).
  - `utoipa` absent du workspace (`grep` Cargo.toml).
  - env LaTeX `keshwarning` (pas `warning`) confirmé dans `docs/manual/shared/kesh-style.sty:289`.

### Completion Notes List

**Story doc-only — aucun code Rust/Svelte exécutable touché → pas de quality gate backend/frontend (CLAUDE.md §« Quand sauter »).** T-C2 n'a PAS réintroduit de code (OpenAPI documenté comme limitation v0.3).

- **T-C1** — `docs/api-external.md` créé (10 sections AC1) : vue d'ensemble, auth Bearer + format token, gestion UI-only (DC6), portées, scoping company, URL de base, exemples curl/Python/JS/MCP, ressources disponibles, sécurité/moindre privilège, limitations v0.2, table des codes d'erreur. Toutes les valeurs techniques ground-truthées sur le code mergé (17-2a #168 + 17-2b #169, main `34f830d`).
- **T-C2** — OpenAPI : `utoipa` absent confirmé → documenté en limitation v0.3 (§9 du guide). Aucune dépendance/annotation ajoutée.
- **T-C3** — synchro :
  - `CHANGELOG.md` : `### Added` « API externe à clé PAT (#100) » sous `## [Non publié]`.
  - `README.md` : puce §Fonctionnalités (livré, sans `(à venir)`) + lien `docs/api-external.md` ; §Feuille de route ligne v0.2 `📋 Backlog → 🚧 En cours` (E17 a 3 stories mergées — correction de dérive ; E17 non marqué `done`, 17-3/17-4 restent).
  - `docs/manual/fr/admin-manual.tex` : nouvelle `\subsection{Clés API (PAT)}` dans §Sécurité (après JWT) + encadré `keshwarning` moindre privilège/L3. PDF régénéré et versionné (+ fichiers aux suivis par git, convention PR #102).
- **T-C4** — `closes #100` dans le commit final (dernière sous-story de 17-2 ; auth+UI+doc complets).
- **Décision documentée** : statut roadmap v0.2 corrigé en `En cours` (au-delà du « pas de changement requis » de la spec) car la spec interdisait seulement de marquer E17 *done* — laisser `Backlog` aurait menti sur l'état réel (règle anti-dérive README CLAUDE.md).

### File List

**Nouveaux fichiers :**
- `docs/api-external.md`

**Fichiers modifiés :**
- `CHANGELOG.md` (entrée `### Added` API PAT)
- `README.md` (puce §Fonctionnalités + statut roadmap v0.2)
- `docs/manual/fr/admin-manual.tex` (subsection Clés API dans §Sécurité)
- `docs/manual/fr/admin-manual.pdf` (régénéré) + fichiers auxiliaires latexmk suivis (`.aux`, `.out`, `.toc`, `.xdv`, `.fdb_latexmk`)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (statut 17-2c)
- `_bmad-output/implementation-artifacts/17-2c-api-pat-doc.md` (cette story : tasks, Dev Agent Record, statut)
