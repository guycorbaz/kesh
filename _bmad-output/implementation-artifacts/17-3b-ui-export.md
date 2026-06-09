# Story 17.3b: UI admin export complet d'installation (`/admin/backup`)

Status: review

<!-- Sous-story de l'épopée 17-3 (export/import installation, #112). Extraite de la spec umbrella `17-3-export-import-installation.md` (Partie B), convergée au validate en 5 passes. Contenu déjà adversarialement revu. Re-validate optionnel. -->
<!-- CONSOMME `GET /api/v1/admin/full-export` posé par 17-3a (DONE, `fb43669`). Ne dépend QUE de 17-3a (pas de 17-3c). Parallélisable. Première route `(app)/admin/` côté front. -->

## Story

As a **administrateur d'une installation Kesh**,
I want **une page d'administration avec un bouton « Exporter toute l'installation » qui télécharge le fichier `.keshbackup`**,
so that **je puisse déclencher une sauvegarde complète depuis l'interface web, sans accès SSH/Docker ni outil en ligne de commande**.

## Contexte & cadrage

**Épopée 17-3 (#112) — split A–F :** 17-3a (backend export, **DONE** `fb43669`) → **17-3b (cette story, UI export)** → 17-3c (backend import, **DONE** `d9e3080`) → 17-3d UI import → 17-3e E2E → 17-3f doc. La spec **umbrella** `17-3-export-import-installation.md` reste la source de contexte complète.

**Rôle :** câbler l'UI sur l'endpoint `GET /api/v1/admin/full-export` (livré par 17-3a) — bouton de déclenchement + download du blob + lien sidebar Admin-only. Story **frontend pure** (Svelte 5 runes), aucun changement backend.

**⚠️ Distinction critique avec l'export 9-2b existant :** la sidebar contient **déjà** un lien « Export global » → `/export` (Story 9-2b : `GET /api/v1/exports/global.zip`, **per-company, CSV, 16 tables**, accessible aux non-Admin). Le nouveau lien 17-3b est **distinct** : **sauvegarde de l'installation complète** (toutes companies + users + données système, `.keshbackup`, **Admin strict**). Les libellés UI doivent lever toute ambiguïté (ex. « Sauvegarde complète (installation) » vs « Export global (société) ») — ne PAS fusionner ni remplacer le lien `/export`.

**Pas de changement backend, pas de migration.** Story frontend uniquement.

## Acceptance Criteria

> Numérotation continue avec l'umbrella (Partie B = AC8–10, + transverses applicables).

1. **(AC8)** Page `/admin/backup` (route `src/routes/(app)/admin/backup/+page.svelte`), **visible uniquement pour le rôle Admin**, avec un bouton « Exporter toute l'installation » qui déclenche `GET /api/v1/admin/full-export` via `apiClient.getBlob` et **télécharge** le fichier (pattern `triggerDownload` de `exports.api.ts`). Le nom de fichier provient de l'en-tête `Content-Disposition` ; fallback `kesh-installation-{YYYY-MM-DD}.keshbackup`.

2. **(AC9)** Pendant l'export : **indicateur de chargement** (bouton désactivé + libellé « Export en cours… ») et **guard de ré-entrance** (un seul export à la fois — premier `if` de la fonction). **Gestion d'erreur** : si `getBlob` jette une `ApiError` (ex. `403` non-Admin/PAT, `500`), afficher un message d'erreur lisible (toast `svelte-sonner` `toast.error` OU encart inline — cohérent avec le pattern de la page de référence `settings/api-keys`). **Succès** → message de succès (toast `toast.success` ou encart). *(Le `403` ne devrait pas survenir via l'UI car la page est Admin-gated, mais l'erreur doit rester gérée défensivement.)*

3. **(AC10)** Lien **« Sauvegarde complète (installation) »** ajouté au groupe `administration` de la sidebar (`src/routes/(app)/+layout.svelte`), **`adminOnly`** (visible si `isAdmin`), pointant vers `/admin/backup`, **i18n FR/DE/IT/EN** (clé `nav-*`, voir §i18n & sidebar). Ne remplace pas le lien `/export` per-company existant.

### Transverses applicables

4. **(AC27 — i18n ownership)** Les clés frontend respectent `lint-i18n-ownership`. Clés feature-scoped `backup-*` pour la page (la feature `lib/features/admin-backup/` les possède) ; clé sidebar en namespace `nav-*` (utilisé dans `+layout.svelte`, cohérent avec les autres items nav). **`npm run lint-i18n-ownership` PASS.** Importer `i18nMsg` depuis **`$lib/shared/utils/i18n.svelte`** directement (PAS depuis une feature — anti-pattern relevé sur `settings/+page.svelte`).

5. **(AC28 — HTTP-LAN safe)** Aucune API secure-context-only en runtime. `URL.createObjectURL` (download) est **sûr en HTTP** (cf. `feedback_no_secure_context_apis_http_lan`). Pas de `crypto.randomUUID`/`navigator.clipboard`/`crypto.subtle` non-gardé. Utiliser `$props.id()` si un id DOM est nécessaire.

## Tasks / Subtasks

- [x] **T-B1** Feature front `src/lib/features/admin-backup/admin-backup.api.ts` : fonction `downloadFullExport(): Promise<void>` via `apiClient.getBlob('/api/v1/admin/full-export')` → `response.blob()` → filename depuis `Content-Disposition` → `triggerDownload`. **DC-B1 (réutilisation download)** : voir Dev Notes — dupliquer localement `parseContentDispositionFilename` + `triggerDownload` (~30 lignes, scope-safety, zéro régression sur 9-2b), OU réutiliser via extraction shared-util si le dev préfère (tracé comme cleanup v0.2). (AC: 8)
- [x] **T-B2** Page `src/routes/(app)/admin/backup/+page.svelte` (runes Svelte 5 : `$state` pour `exporting`/`errorMsg`, `$derived` au besoin) : bouton « Exporter toute l'installation », état chargement (désactivé + « Export en cours… »), **guard ré-entrance** (`if (exporting) return;` first-line), gestion d'erreur typée (message lisible depuis `ApiError`), succès. **Header de page** expliquant brièvement la différence avec l'export per-company. Import `i18nMsg` depuis `$lib/shared/utils/i18n.svelte`. (AC: 8, 9, 28)
- [x] **T-B3** Sidebar : ajouter l'item à `navGroups` groupe `administration` → `adminOnly` (`src/routes/(app)/+layout.svelte`), forme `{ i18nKey: 'nav-admin-backup', fallback: 'Sauvegarde complète', href: '/admin/backup' }`. **Vérifier que le rendu de la boucle `adminOnly` (≈ ligne 304-305) gère la forme `i18nKey`** (les `items` principaux la gèrent ; les `adminOnly` actuels sont en `label:` hardcodé FR — FINDING-7) ; sinon, adapter le rendu pour supporter `i18nKey` (utiliser le même helper que les items principaux). Clé `nav-admin-backup` + i18n **FR/DE/IT/EN** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`). (AC: 10, 27)
- [x] **T-B4** Tests : **unit composant** (vitest) — bouton rend, clic appelle `downloadFullExport` (mocké), état chargement désactive le bouton + guard ré-entrance (2e clic ne relance pas), erreur affichée. Test unit `admin-backup.api.ts` (`parseContentDispositionFilename` si dupliqué localement : RFC 5987 + 6266 + fallback). **`npm run lint-i18n-ownership` PASS.** *(E2E Playwright optionnel — non bloquant ; un test « bouton visible Admin / déclenche download » peut être ajouté en 17-3e ou ici si trivial.)* (AC: 27)

## Dev Notes

### Décisions de conception

| # | Décision (périmètre 17-3b) |
|---|---|
| **DC-B1** | **Réutilisation download** : dupliquer localement `parseContentDispositionFilename` + `triggerDownload` dans `admin-backup.api.ts` (~30 lignes). Rationale : éviter (a) un **import cross-feature** depuis `lib/features/export/` (anti-pattern projet) et (b) de toucher le chemin de download 9-2b qui fonctionne (risque de régression pour zéro bénéfice utilisateur). L'extraction vers `lib/shared/utils/download.ts` (3ᵉ feature à dupliquer = seuil franchi) est **tracée comme cleanup v0.2 / Epic 15** (cohérent note umbrella 9-2b §triggerDownload-reuse). Le dev PEUT faire l'extraction s'il le juge net (tests frontend complets disponibles), mais ce n'est pas exigé. |
| **DC-B2** | **Pas de fusion** avec le lien `/export` per-company existant (9-2b) — deux fonctions distinctes (installation Admin vs société). Libellés UI désambiguïsés. |
| **DC-B3** | Filename depuis `Content-Disposition` (le backend 17-3a le fournit, `kesh-installation-{date}.keshbackup`) ; fallback côté front. Le front ne calcule pas le nom. |

### Réutilisation — patterns frontend (ground-truth)

| Brique | Chemin:ligne | Usage 17-3b |
|---|---|---|
| `apiClient.getBlob` | `src/lib/shared/utils/api-client.ts:527` (`getBlob(url) -> Promise<Response>`, jette `ApiError` sur non-2xx via `parseErrorResponse`) | **Directe** — `downloadFullExport` |
| Pattern download complet | `src/lib/features/export/exports.api.ts` (`downloadGlobalExport` + `parseContentDispositionFilename` [**exporté**] + `triggerDownload` [privé, cleanup `finally`]) | **Modèle** à dupliquer (DC-B1) — adapter l'URL `/api/v1/admin/full-export` + fallback `kesh-installation-{YYYY-MM-DD}.keshbackup` |
| Page de référence (état/erreur/toast) | `src/routes/(app)/settings/api-keys/+page.svelte` (runes `$state`, `toast` `svelte-sonner` `:15`, `i18nMsg` depuis `$lib/shared/utils/i18n.svelte` `:10`) | Modèle page (état `exporting`, toasts succès/erreur) |
| Sidebar navGroups | `src/routes/(app)/+layout.svelte:52` (`navGroups`), `:79` groupe `administration`, `:93` `adminOnly`, `:43` `isAdmin`, `:304` rendu `adminOnly` | Ajouter l'item `adminOnly` (T-B3) ; **vérifier rendu i18nKey** |
| Type `NavItem` | `+layout.svelte:18-19` (`{ i18nKey, fallback, href }` \| `{ label, href }`) | Utiliser la forme `i18nKey` pour le nouvel item |
| i18n | `src/lib/shared/utils/i18n.svelte.ts` (`i18nMsg(key, fallback, args?)`) ; `.ftl` `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` | Clés `backup-*` (page) + `nav-admin-backup` (sidebar) ×4 locales |
| Lint ownership | `frontend/scripts/lint-i18n-ownership.js` (`GLOBAL_NAMESPACES = ['error','tooltip','common','mode','shortcut','demo']` `:16` ; vérifie l'ownership des clés feature dans `lib/features/*`) | `backup-*` possédées par `lib/features/admin-backup/` ; `nav-*` utilisé dans `+layout.svelte` (route, hors check feature) |
| HTTP-LAN safe | `feedback_no_secure_context_apis_http_lan` ; `URL.createObjectURL` OK en HTTP ; `$props.id()` (ex. `ContactPicker.svelte`) | AC28 |

### i18n & sidebar — détails

- **Clés page** (`backup-*`, namespace feature `admin-backup`) — exemples : `backup-page-title` (« Sauvegarde complète de l'installation »), `backup-page-description` (distinction vs export société), `backup-action-export` (« Exporter toute l'installation »), `backup-action-exporting` (« Export en cours… »), `backup-toast-success`, `backup-error-generic`. **FR/DE/IT/EN** dans les 4 `.ftl`.
- **Clé sidebar** : `nav-admin-backup` (fallback `Sauvegarde complète`), **FR/DE/IT/EN**. Placée près des autres `nav-*` dans les `.ftl`.
- ⚠️ **Rendu `adminOnly`** : la boucle `{#each group.adminOnly as item}` (≈ `:305`) doit gérer la forme `{ i18nKey, fallback, href }` (et pas seulement `{ label, href }`). Si elle ne gère que `label`, factoriser le rendu d'un `NavItem` (le bloc des `items` principaux gère déjà `i18nKey`) pour éviter de réintroduire du FR hardcodé contre AC10. Vérifier `data-testid` (`navTestid(href)` `:188`) cohérent pour `/admin/backup`.

### Réutilisation — backend déjà livré (17-3a)

- `GET /api/v1/admin/full-export` : **Admin strict + anti-PAT**, `Content-Type: application/octet-stream`, `Content-Disposition: attachment; filename="kesh-installation-{YYYY-MM-DD}.keshbackup"`. Réponse = conteneur ZIP `.keshbackup`. Aucun corps de requête (GET). Cf. `crates/kesh-api/src/routes/admin.rs::full_export`.
- Le gating RBAC est **doublé** : sidebar `isAdmin` (front, UX) + `require_admin_role` (back, sécurité). Un non-Admin qui forcerait l'URL `/admin/backup` verrait la page mais l'appel API retournerait `403` (géré AC9).

### Standards projet (CLAUDE.md)

- **Test Locally First** (frontend) avant push : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build`. **E2E** si routes/pages touchées : `npm run test:e2e` (browsers installés ; `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` requis sur Ubuntu 26.04+, cf. `reference_playwright_ubuntu26`). Cette story ajoute une route → E2E pertinent (au moins un smoke test, ou délégué 17-3e).
- **Migration breaking policy** : N/A (frontend). **Pattern batch** : N/A.
- **Commit par étape BMAD**, pas de push auto. Branche active : `story/17-3-export-import-installation` (stack des sous-stories 17-3).
- **HTTP-LAN safe** : impératif (déploiement NAS HTTP), cf. AC28.

### Project Structure Notes

- **Nouveau** : `src/lib/features/admin-backup/admin-backup.api.ts` + `src/routes/(app)/admin/backup/+page.svelte` (+ test(s)). **Premier usage du préfixe `/admin/` côté front** — cohérent avec le gating `isAdmin`. 17-3d ajoutera `/admin/restore` (même dossier `/admin/`).
- **Modifié** : `src/routes/(app)/+layout.svelte` (item sidebar) + 4 `.ftl` (clés). 
- Aucun conflit avec 9-2b (chemins/scope distincts).

### References

- [Source: _bmad-output/implementation-artifacts/17-3-export-import-installation.md] — spec umbrella (Partie B AC8-10, §Réutilisation Frontend, risques)
- [Source: _bmad-output/implementation-artifacts/17-3a-backend-export.md] — endpoint `GET /admin/full-export` consommé
- [Source: src/lib/features/export/exports.api.ts] — pattern download (triggerDownload, parseContentDispositionFilename)
- [Source: src/routes/(app)/settings/api-keys/+page.svelte] — modèle page (runes, toasts, i18nMsg)
- [Source: src/routes/(app)/+layout.svelte:52-95,304] — sidebar navGroups + adminOnly
- [Source: frontend/scripts/lint-i18n-ownership.js] — règles ownership i18n
- [Source: CLAUDE.md] — Test Locally First (frontend + E2E), HTTP-LAN safe, commit/branch

## Dev Agent Record

### Agent Model Used

Opus 4.8 (claude-opus-4-8[1m]) — single-pass orchestré T-B1→T-B4.

### Debug Log References

Quality gate frontend (Test Locally First) : `npm run check` **0 erreurs** (25 warnings pré-existants, aucun dans admin-backup), `npm run lint-i18n-ownership` **PASS**, `npm run test:unit` **33 fichiers / 287 tests** (dont 11 nouveaux admin-backup), `npm run build` ✓. E2E `sidebar-navigation.spec.ts` vérifié non-impacté (navigue 5 items non-adminOnly, pas de compte total).

### Completion Notes List

- **Architecture** : logique + UI extraites dans le composant feature `lib/features/admin-backup/AdminBackupPanel.svelte` (testable + ownership i18n) ; la route `(app)/admin/backup/+page.svelte` est un thin wrapper qui le rend. Première route sous `(app)/admin/` côté front.
- **`admin-backup.api.ts`** : `downloadFullExport()` via `apiClient.getBlob('/api/v1/admin/full-export')` + parse `Content-Disposition` + `triggerDownload`. DC-B1 : `parseContentDispositionFilename` + `triggerDownload` **dupliqués localement** (pas d'import cross-feature, zéro régression 9-2b ; extraction shared-util tracée v0.2/Epic 15).
- **`AdminBackupPanel.svelte`** : `$state` `exporting`/`errorMsg`, **guard ré-entrance** (`if (exporting) return`), bouton désactivé + libellé « Export en cours… » pendant l'export, succès → `toast.success`, erreur → encart `role="alert"` + `toast.error` (message `ApiError` ou fallback). `i18nMsg` importé de `$lib/shared/utils/i18n.svelte` (pas cross-feature). HTTP-LAN safe (`URL.createObjectURL`, aucune API secure-context).
- **Sidebar** : item `adminOnly` `{ i18nKey: 'nav-admin-backup', ... href: '/admin/backup' }` — rendu via `getItemLabel` existant (gère déjà `i18nKey`, aucune modif de rendu nécessaire). Distinct du lien `/export` per-company (9-2b).
- **i18n** : clés renommées `backup-*` → **`admin-backup-*`** (le lint exige préfixe = nom du dossier feature `admin-backup`) + `nav-admin-backup`, **FR/DE/IT/EN** (8 clés ×4 locales).
- **Tests** : `admin-backup.api.test.ts` (parseContentDispositionFilename RFC 5987/6266/fallback + downloadFullExport happy/403) + `AdminBackupPanel.test.ts` (rend, clic→downloadFullExport, état désactivé + guard ré-entrance, erreur→encart+toast). 11 tests verts.
- **Backend** : aucun changement (consomme l'endpoint 17-3a). Aucune migration.

### File List

**Nouveaux fichiers :**
- `frontend/src/lib/features/admin-backup/admin-backup.api.ts`
- `frontend/src/lib/features/admin-backup/AdminBackupPanel.svelte`
- `frontend/src/lib/features/admin-backup/admin-backup.api.test.ts`
- `frontend/src/lib/features/admin-backup/AdminBackupPanel.test.ts`
- `frontend/src/routes/(app)/admin/backup/+page.svelte`

**Fichiers modifiés :**
- `frontend/src/routes/(app)/+layout.svelte` — item sidebar `administration.adminOnly`.
- `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` — 8 clés (`nav-admin-backup` + `admin-backup-*`).

### Change Log

| Date | Étape | Modèle | Résumé |
|------|-------|--------|--------|
| 2026-06-09 | dev-story | Opus 4.8 | Implémentation single-pass T-B1→T-B4. Feature `lib/features/admin-backup/` (`admin-backup.api.ts` downloadFullExport + `AdminBackupPanel.svelte` testable) + route thin `(app)/admin/backup/+page.svelte` + item sidebar `adminOnly` (i18nKey, rendu via getItemLabel existant) + 8 clés i18n ×4 locales (renommées `admin-backup-*` pour satisfaire le lint = préfixe dossier feature). DC-B1 download dupliqué localement (pas d'import cross-feature). 5 nouveaux + 5 modifiés fichiers. **Quality gate** : check 0 erreurs, lint-i18n-ownership PASS, test:unit 33/33 fichiers 287 tests (11 nouveaux), build ✓. E2E sidebar non-impacté. Backend inchangé. Status review. Prochaine : `bmad-code-review 17-3b` (Sonnet 4.6, LLM différent). |
| 2026-06-09 | create-story (sous-story) | Opus 4.8 | Story 17-3b (UI export) extraite de l'umbrella Partie B (AC8-10). Ancrée sur les patterns frontend réels : `apiClient.getBlob:527`, pattern download `exports.api.ts` (triggerDownload + parseContentDispositionFilename), sidebar navGroups `administration.adminOnly`, page de réf `settings/api-keys`. Décisions : DC-B1 dupliquer download localement (scope-safety, extraction shared-util tracée v0.2) ; DC-B2 distinction stricte vs export per-company 9-2b ; clé sidebar `nav-admin-backup` i18nKey (vérifier rendu adminOnly). T-B1..T-B4. Frontend pur, aucun backend. Re-validate optionnel. Prochaine : `bmad-dev-story 17-3b`. |
