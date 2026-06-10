# Story 17.3d: UI admin import complet d'installation (`/admin/restore`)

Status: done

<!-- Sous-story de l'épopée 17-3 (export/import installation, #112). Extraite de la spec umbrella `17-3-export-import-installation.md` (Partie D), convergée au validate en 5 passes. Contenu déjà adversarialement revu. Re-validate optionnel. -->
<!-- CONSOMME `POST /api/v1/admin/full-import` posé par 17-3c (DONE, `d9e3080`). Dépend de 17-3c. Dernière brique UI de l'épopée (ferme la boucle export↔import). -->

## Story

As a **administrateur d'une installation Kesh**,
I want **une page d'administration où je téléverse un fichier `.keshbackup` et confirme explicitement le remplacement total des données, après quoi je suis déconnecté**,
so that **je puisse restaurer/migrer une installation complète depuis l'interface web, en pleine conscience du caractère destructeur de l'opération**.

## Contexte & cadrage

**Épopée 17-3 (#112) — split A–F :** 17-3a (backend export **DONE**) → 17-3b (UI export **DONE**) → 17-3c (backend import **DONE** `d9e3080`) → **17-3d (cette story, UI import)** → 17-3e E2E → 17-3f doc. La spec **umbrella** reste la source de contexte complète.

**Rôle :** câbler l'UI sur l'endpoint `POST /api/v1/admin/full-import` (livré par 17-3c) — sélecteur de fichier + **modal de confirmation forte** (opération destructrice : remplace TOUTES les données + déconnexion) + gestion d'erreurs typées + **redirection `/login` au succès** (la réponse `sessionInvalidated:true` signale que les `refresh_tokens` destination ont été remplacés). Story **frontend pure** (Svelte 5 runes), aucun changement backend.

**⚠️ Opération destructrice irréversible côté UX :** contrairement à l'export (idempotent, sans effet de bord), l'import **remplace l'intégralité de l'installation**. La confirmation forte est **non-négociable** (AC19) : pas d'envoi sans double action explicite. Le backend prend déjà un backup pré-import (filet de sécurité serveur, 17-3c) mais l'utilisateur n'y a pas accès sans SSH (ne PAS exposer le chemin).

**Pas de changement backend, pas de migration.** Story frontend uniquement.

## Acceptance Criteria

> Numérotation continue avec l'umbrella (Partie D = AC18–21, + transverses AC27/28).

1. **(AC18)** Page `/admin/restore` (route `src/routes/(app)/admin/restore/+page.svelte`), **Admin only** — gating **doublé** : (a) **guard `+page.ts`** (redirect `/` si `role !== 'Admin'`, pattern `users/+page.ts` — leçon 17-3b : la visibilité sidebar seule ne suffit pas), (b) sidebar `isAdmin`. Sélecteur de fichier `.keshbackup` (input `type=file`, `accept=".keshbackup"`), upload via `apiClient.postFormData` (champ `file`, pattern `bank-import`).

2. **(AC19)** **Confirmation forte (non-négociable)** : avant tout envoi, un **modal `Dialog`** (`lib/components/ui/dialog/`) avertit explicitement : « Cette action va **remplacer TOUTES les données** de l'installation actuelle. Une sauvegarde de l'état actuel sera créée côté serveur avant l'import. **Vous serez déconnecté** et devrez vous reconnecter avec les identifiants de l'instance importée. » + **double action** (Confirmer / Annuler). **Aucun envoi sans confirmation explicite** (le clic sur « Importer » ouvre le modal ; seul le bouton « Confirmer » du modal déclenche l'upload). Pas d'envoi non plus sans fichier sélectionné.

3. **(AC20)** Pendant l'import : **indicateur de progression** (bouton désactivé + libellé « Import en cours… » + `aria-busy`) et **guard de ré-entrance**. **Gestion d'erreurs typées** (depuis `ApiError.error.code` / status) :
   - `409 IMPORT_VERSION_INCOMPATIBLE` → message version (source `details.sourceMinRequired` vs `details.binaryVersion`) ;
   - `400 INVALID_BACKUP_STRUCTURE` → message « fichier invalide/corrompu » ;
   - `400 IMPORT_SCHEMA_MISMATCH` → message « schéma incompatible » (table `details.table`) ;
   - `413` (Payload Too Large) → message limite de taille ;
   - `500 ADMIN_FULL_IMPORT_FAILED` → message « échec, état précédent préservé ».
   **Ne PAS afficher de chemin de backup interne** (non accessible à l'utilisateur, AC sécurité umbrella).

4. **(AC20-succès)** **Succès** (`200`, corps `{ backupCreated, tablesRestored, rowsRestored, sourceVersion, sessionInvalidated:true }`) → message de succès bref puis **déconnexion propre + redirection `/login`** : `await authState.logout()` (best-effort — le cookie est déjà invalidé côté serveur, les `refresh_tokens` ayant été remplacés) puis `window.location.replace('/login')` (pattern bouton déconnexion existant). La session courante est invalide ⇒ l'utilisateur se reconnecte avec les identifiants **de l'instance importée**.

5. **(AC21)** Lien **« Restaurer / Importer installation »** ajouté au groupe `administration` de la sidebar (`adminOnly`), pointant vers `/admin/restore`, **i18n FR/DE/IT/EN** (clé `nav-admin-restore`). Distinct du lien `/admin/backup` (export) ajouté en 17-3b.

### Transverses applicables

6. **(AC27 — i18n ownership)** Clés feature **`admin-restore-*`** (préfixe = nom du dossier feature `lib/features/admin-restore/`, **exigé par `lint-i18n-ownership`** — leçon 17-3b : un préfixe `restore-*` serait rejeté car ≠ dossier). Clé sidebar `nav-admin-restore`. **`npm run lint-i18n-ownership` PASS.** Importer `i18nMsg` depuis `$lib/shared/utils/i18n.svelte`.

7. **(AC28 — HTTP-LAN safe)** Aucune API secure-context-only. Pas de `crypto.randomUUID`/`navigator.clipboard`/`crypto.subtle` non-gardé ; `$props.id()` pour les ids DOM si nécessaire. `window.location.replace` est sûr en HTTP.

## Tasks / Subtasks

- [x] **T-D1** Feature front `src/lib/features/admin-restore/admin-restore.api.ts` : `uploadFullImport(file: File): Promise<FullImportResponse>` via `apiClient.postFormData('/api/v1/admin/full-import', form)` (form avec champ `file`, pattern `bank-import.api.ts`). Type `FullImportResponse` = `{ backupCreated: boolean; tablesRestored: number; rowsRestored: number; sourceVersion: string; sessionInvalidated: boolean }` (camelCase, miroir de la réponse 17-3c). (AC: 18)
- [x] **T-D2** Composant `src/lib/features/admin-restore/AdminRestorePanel.svelte` (runes Svelte 5, testable, possède les clés `admin-restore-*`) : input file (`accept=".keshbackup"`, `$state` du fichier sélectionné), bouton « Importer » désactivé tant qu'aucun fichier ; clic → ouvre le **modal `Dialog` de confirmation forte** (AC19) ; « Confirmer » → upload (guard ré-entrance `if (importing) return`), bouton désactivé + `aria-busy` + « Import en cours… » ; **erreurs typées** (AC20, via `isApiError` + `err.error.code`/status) ; **succès** → message + `await authState.logout()` + `window.location.replace('/login')` (AC20-succès). `<svelte:head><title>`. Import `i18nMsg` depuis `$lib/shared/utils/i18n.svelte`. (AC: 18, 19, 20, 27, 28)
- [x] **T-D3** Route `src/routes/(app)/admin/restore/+page.svelte` (thin wrapper rendant `<AdminRestorePanel />`) + **`src/routes/(app)/admin/restore/+page.ts`** (guard Admin, copie de `admin/backup/+page.ts`). Item sidebar `administration.adminOnly` `{ i18nKey: 'nav-admin-restore', fallback: 'Restaurer / Importer', href: '/admin/restore' }` (`+layout.svelte`, rendu via `getItemLabel` existant). Clés i18n `nav-admin-restore` + `admin-restore-*` **FR/DE/IT/EN** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`). (AC: 18, 21, 27)
- [x] **T-D4** Tests : **unit composant** (vitest, `@testing-library/svelte`, mock `admin-restore.api` + `i18nMsg` + `svelte-sonner` + `authState`) — (a) bouton « Importer » désactivé sans fichier ; (b) **confirmation bloque l'envoi** : clic « Importer » n'appelle PAS `uploadFullImport` (ouvre le modal) ; seul « Confirmer » l'appelle ; (c) état chargement + guard ré-entrance ; (d) erreur typée affichée ; (e) succès → `authState.logout` appelé (mocké). Test unit `admin-restore.api.ts` (form contient le champ `file`). **`npm run lint-i18n-ownership` PASS.** *(E2E Playwright déféré 17-3e.)* (AC: 19, 27)

## Dev Notes

### Décisions de conception (leçons 17-3b appliquées)

| # | Décision (périmètre 17-3d) |
|---|---|
| **DC-D1** | **Guard `+page.ts` Admin obligatoire** (pas seulement sidebar `isAdmin`) — copie de `admin/backup/+page.ts` (lui-même copie de `users/+page.ts`). Sans lui, accès direct par URL afficherait l'UI destructrice. Défense en profondeur ; le RBAC backend `require_admin_role` reste l'autorité de sécurité. |
| **DC-D2** | **Clés i18n `admin-restore-*`** (préfixe = nom dossier feature, exigé par le lint — un `restore-*` serait rejeté). |
| **DC-D3** | **Confirmation forte via `Dialog` modal** — l'upload n'est JAMAIS déclenché par le bouton « Importer » directement, seulement par « Confirmer » dans le modal. Pas de fichier → bouton désactivé. |
| **DC-D4** | **Succès → `authState.logout()` (best-effort) + `window.location.replace('/login')`** (pattern bouton déconnexion `+layout.svelte:259`). `window.location.replace` (pas `goto`) force un reload complet → re-hydrate auth → reste sur /login car session invalide. |
| **DC-D5** | **Ne pas exposer le chemin du backup pré-import** dans l'UI (loggé serveur uniquement, 17-3c AC13). |

### Réutilisation — patterns frontend (ground-truth)

| Brique | Chemin:ligne | Usage 17-3d |
|---|---|---|
| Upload multipart | `src/lib/features/bank-import/bank-import.api.ts` (`postFormData` + `FormData` champ `file`) | **Modèle** `uploadFullImport` |
| `apiClient.postFormData` | `src/lib/shared/utils/api-client.ts:543` (`postFormData<T>(url, form)`, jette `ApiError` sur non-2xx) | Upload import |
| `isApiError` + `ApiError` | `src/lib/shared/utils/api-client.ts` (`isApiError(err)`, `err.error.code`, `err.message`) | Erreurs typées (AC20) |
| Modal `Dialog` (bits-ui) | `src/lib/components/ui/dialog/` (`Dialog.Root/Content/Header/Title/Description/Footer`, `index.ts`) | Confirmation forte (AC19) |
| Déconnexion + redirect | `src/routes/(app)/+layout.svelte:258-261` (`await authState.logout(); window.location.replace('/login')`) | Succès (AC20) |
| Auth store | `src/lib/app/stores/auth.svelte.ts` (`logout()` POST /auth/logout best-effort `.catch` + clear local) | DC-D4 |
| Guard route Admin | `src/routes/(app)/admin/backup/+page.ts` (créé 17-3b) / `users/+page.ts` | Copier pour `/admin/restore` (DC-D1) |
| Page/composant de réf | `src/lib/features/admin-backup/AdminBackupPanel.svelte` (créé 17-3b — runes, état, toast, svelte:head, aria-busy) | Modèle composant symétrique |
| Sidebar | `src/routes/(app)/+layout.svelte` (`administration.adminOnly`, `getItemLabel` gère `i18nKey`) | Item `nav-admin-restore` |
| Lint ownership | `frontend/scripts/lint-i18n-ownership.js` (préfixe clé = nom dossier feature multi-segment) | `admin-restore-*` (AC27) |
| i18n | `src/lib/shared/utils/i18n.svelte.ts` (`i18nMsg(key, fallback, args?)`) ; `.ftl` 4 locales | Clés `admin-restore-*` + `nav-admin-restore` |

### Réutilisation — backend déjà livré (17-3c)

`POST /api/v1/admin/full-import` (`crates/kesh-api/src/routes/admin.rs::full_import`) : **Admin strict + anti-PAT**, multipart champ `file`, `DefaultBodyLimit` (`KESH_ADMIN_IMPORT_MAX_MB` défaut 512). Réponses :
- `200` + `{ backupCreated, tablesRestored, rowsRestored, sourceVersion, sessionInvalidated:true }` (camelCase) ;
- `400 INVALID_BACKUP_STRUCTURE` (structure ZIP / SHA tamper / formatVersion) ;
- `400 IMPORT_SCHEMA_MISMATCH` (`details: { table, unknownColumns, missingRequiredColumns }`) ;
- `409 IMPORT_VERSION_INCOMPATIBLE` (`details: { sourceMinRequired, binaryVersion }`) ;
- `413` (taille) ; `500 ADMIN_FULL_IMPORT_FAILED` (échec restore, état préservé) ; `403` (non-Admin / PAT).

### Standards projet (CLAUDE.md)

- **Test Locally First** (frontend) : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build`. **E2E** : route ajoutée → un smoke test pertinent (ou délégué 17-3e double-instance). `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` sur Ubuntu 26.04+.
- **Migration breaking policy** / **Pattern batch** : N/A (frontend).
- **Commit par étape BMAD**, pas de push auto. Branche active : `story/17-3-export-import-installation`.
- **HTTP-LAN safe** : impératif (AC28).

### Project Structure Notes

- **Nouveau** : `src/lib/features/admin-restore/` (`admin-restore.api.ts` + `AdminRestorePanel.svelte` + tests) + `src/routes/(app)/admin/restore/{+page.svelte,+page.ts}`. 2ᵉ route sous `(app)/admin/` (après `/admin/backup`).
- **Modifié** : `src/routes/(app)/+layout.svelte` (item sidebar) + 4 `.ftl` (clés).
- Aucun conflit avec 17-3b (`/admin/backup`) ni 9-2b (`/export`).

### References

- [Source: _bmad-output/implementation-artifacts/17-3-export-import-installation.md] — spec umbrella (Partie D AC18-21, §Réutilisation Frontend)
- [Source: _bmad-output/implementation-artifacts/17-3c-backend-import.md] — endpoint `POST /admin/full-import` consommé (réponse + codes d'erreur)
- [Source: _bmad-output/implementation-artifacts/17-3b-ui-export.md] — patterns établis (guard +page.ts, AdminBackupPanel, i18n admin-* , svelte:head, aria-busy)
- [Source: src/lib/features/bank-import/bank-import.api.ts] — pattern upload postFormData
- [Source: src/lib/components/ui/dialog/] — modal de confirmation
- [Source: src/routes/(app)/+layout.svelte:258-261] — déconnexion + redirect /login
- [Source: CLAUDE.md] — Test Locally First, HTTP-LAN safe, commit/branch

## Dev Agent Record

### Agent Model Used

Opus 4.8 (claude-opus-4-8[1m]) — single-pass orchestré T-D1→T-D4.

### Debug Log References

Quality gate frontend (Test Locally First) : `npm run check` **0 erreurs** (25 warnings pré-existants), `npm run lint-i18n-ownership` **PASS**, `npm run test:unit` **35 fichiers / 295 tests** (6 nouveaux admin-restore), `npm run build` ✓. E2E sidebar non-impacté (navigue 5 items non-adminOnly). E2E dédié déféré 17-3e.

### Completion Notes List

- **`admin-restore.api.ts`** : `uploadFullImport(file)` via `apiClient.postFormData('/api/v1/admin/full-import', form)` (champ `file`) + type `FullImportResponse` (miroir camelCase 17-3c).
- **`AdminRestorePanel.svelte`** : input file (`accept=".keshbackup"`, `$state` selectedFile), bouton « Importer » désactivé sans fichier → **ouvre le modal `Dialog`** (DC-D3, n'envoie rien) ; **« Confirmer »** seul déclenche l'upload (guard ré-entrance, `aria-busy`, « Import en cours… »). **Erreurs typées** (`isApiError` + `err.code` : `IMPORT_VERSION_INCOMPATIBLE`/`IMPORT_SCHEMA_MISMATCH`/`INVALID_BACKUP_STRUCTURE` → messages dédiés avec `details`, 413/500 → `err.message`). **Succès** → toast + `authState.logout()` + `window.location.replace('/login')` (DC-D4). `<svelte:head><title>`. `i18nMsg` depuis `$lib/shared/utils/i18n.svelte`. *(Note : `ApiError` expose `err.code`/`err.details` au top-level — pas `err.error.code` ; corrigé vs la formulation de l'AC20.)*
- **Route** : `(app)/admin/restore/+page.svelte` thin + **`+page.ts` guard Admin** (DC-D1, copie de `admin/backup/+page.ts`).
- **Sidebar** : item `adminOnly` `nav-admin-restore` (rendu via `getItemLabel`), distinct de `nav-admin-backup` (17-3b).
- **i18n** : 15 clés `admin-restore-*` + `nav-admin-restore`, **FR/DE/IT/EN** (placeholders `{ $src }`/`{ $bin }`/`{ $table }` pour version/schéma).
- **Tests** : `AdminRestorePanel.test.ts` (bouton désactivé sans fichier ; **« Importer » n'appelle PAS upload, ouvre le modal** ; « Confirmer » → upload + logout + redirect ; erreur → encart, pas de redirect) + `admin-restore.api.test.ts` (FormData champ `file` + 409). 6 tests verts (modal bits-ui testé via `findByTestId`).
- **Backend** : aucun changement. Aucune migration.

### File List

**Nouveaux fichiers :**
- `frontend/src/lib/features/admin-restore/admin-restore.api.ts`
- `frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte`
- `frontend/src/lib/features/admin-restore/admin-restore.api.test.ts`
- `frontend/src/lib/features/admin-restore/AdminRestorePanel.test.ts`
- `frontend/src/routes/(app)/admin/restore/+page.svelte`
- `frontend/src/routes/(app)/admin/restore/+page.ts`

**Fichiers modifiés :**
- `frontend/src/routes/(app)/+layout.svelte` — item sidebar `nav-admin-restore`.
- `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` — 16 clés (`nav-admin-restore` + 15 `admin-restore-*`).

### Change Log

| Date | Étape | Modèle | Résumé |
|------|-------|--------|--------|
| 2026-06-09 | code-review (cycle) | Sonnet→Haiku→Opus | **CYCLE CONVERGÉ en 3 passes**, trend > LOW **~2HIGH+MEDIUMs → réfutés+1 fix → 0**. **P1 Sonnet** : Acceptance+EdgeCase 0>LOW ; Blind 2 HIGH (double-submit + logout-throw) **réfutés par EdgeCase** mais durcis défensivement vu l'op destructrice — restructure `confirmImport` (`importing=true` en 1er + side-effects succès hors try, logout best-effort + redirect garanti) + extension early-reject + showCloseButton=false + aria-describedby + tests vi.waitFor. **P2 Haiku** : Blind HIGH « input non vidé » réfuté (selectedFile=source de vérité) → fix trivial `input.value=''` ; EdgeCase HIGH « Escape ferme dialog » **dismissed** (reviewer conclut lui-même safe, AC19 respecté) ; MEDIUM aria-describedby-timing réfuté (Svelte flush atomique). **P3 Opus** (catch-architectural) : **0 > LOW** sur les 3 reviewers — 7 axes vérifiés (contrat confirm-only-upload, async ordering, state machine, succès→logout→redirect, mapping codes erreur exact vs errors.rs, i18n placeholders, a11y) ; 1 LOW noté (401-sur-POST retry api-client, mitigé verrou backend + restore idempotent, hors-scope). Quality gate final : check 0 erreurs, lint-i18n-ownership PASS, test:unit 35/35 296 tests, build ✓. Status → done. |
| 2026-06-09 | dev-story | Opus 4.8 | Implémentation single-pass T-D1→T-D4. Feature `lib/features/admin-restore/` (`admin-restore.api.ts uploadFullImport` + `AdminRestorePanel.svelte`) : input file + **modal Dialog confirmation forte** (upload uniquement via « Confirmer », DC-D3) + erreurs typées (409/400×2/+message) + succès→`authState.logout()`+`window.location.replace('/login')` (DC-D4) + route thin + **`+page.ts` guard Admin** (DC-D1) + item sidebar `nav-admin-restore` + 16 clés i18n×4. 6 nouveaux + 5 modifiés fichiers. **Quality gate** : check 0 erreurs, lint-i18n-ownership PASS, test:unit 35/35 fichiers 295 tests (6 nouveaux), build ✓. Backend inchangé. Status review. Prochaine : `bmad-code-review 17-3d` (Sonnet 4.6, LLM différent). |
| 2026-06-09 | create-story (sous-story) | Opus 4.8 | Story 17-3d (UI import) extraite de l'umbrella Partie D (AC18-21). Ancrée sur les patterns réels : `postFormData` (bank-import), Dialog (bits-ui), déconnexion `authState.logout()`+`window.location.replace('/login')` (+layout.svelte:259), guard `+page.ts` (admin/backup créé 17-3b), composant de réf `AdminBackupPanel`. **Leçons 17-3b intégrées** : DC-D1 guard +page.ts obligatoire, DC-D2 clés `admin-restore-*` (préfixe=dossier), svelte:head, aria-busy. DC-D3 confirmation forte (upload uniquement via « Confirmer » du modal). DC-D4 succès→logout+redirect /login (sessionInvalidated). Frontend pur, consomme 17-3c. T-D1..T-D4. Prochaine : `bmad-dev-story 17-3d`. |
