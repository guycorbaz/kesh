# Story 21.6c: Intégrations & découvrabilité des rappels (frontend)

Status: ready-for-dev

<!-- Créée 2026-07-17 par bmad-create-story. Cartographie ground-truth par 3 agents Explore (fiche facture / dashboard / backend+tests). Dernière sous-story du split 21-6 (2026-07-16). CONSOMME 21-6a (dunningPausedAt exposé sur InvoiceResponse) ET 21-6b (page /invoices/reminders + feature reminders/). FRONTEND PUR : endpoints history/pause/resume livrés par 21-5a, aucun code Rust, aucune migration. Décisions Guy 2026-07-17 : pause via modale (note) + reprise directe ; compteur dans le widget « Factures ouvertes » existant ; liens croisés bidirectionnels échéancier ↔ Rappels. -->

## Story

En tant que **comptable d'une PME suisse**,
je veux **voir l'historique des rappels sur la fiche d'une facture, y suspendre/reprendre ses relances, repérer d'un coup d'œil depuis le tableau de bord combien de factures sont à rappeler, et naviguer entre l'échéancier et la page Rappels**,
afin de **piloter le suivi des débiteurs sans chercher — chaque information est là où je la regarde déjà**.

## Contexte

**Tout le backend est livré** (21-5a). Cette story câble quatre intégrations frontend qui rendent les rappels *découvrables* là où l'utilisateur travaille déjà :

1. **Historique des rappels sur la fiche facture** — une section listant les rappels envoyés/manuels d'une facture (`GET /api/v1/invoices/{id}/reminders`).
2. **Toggle de suspension sur la fiche facture** — suspendre/reprendre les rappels d'une facture (`PUT .../dunning-pause` / `/dunning-resume`), avec le badge « Suspendu » et la note.
3. **Compteur dashboard** — « N factures à rappeler » sur le widget « Factures ouvertes » (dérivé client de `GET /api/v1/dunning/reminders`, D25).
4. **Liens croisés** échéancier ↔ page Rappels.

**21-6c est FRONTEND PUR** : aucun endpoint nouveau, aucun code Rust, aucune migration. Les endpoints existent depuis 21-5a ; il manque seulement les wrappers API frontend, l'UI, et les clés i18n.

### Décisions figées (Guy, 2026-07-17)

- **D-c1 — Pause via modale (note), reprise directe.** « Suspendre » ouvre une modale (patron `ManualReminderDialog`) avec un champ **note optionnel** ; « Reprendre » agit directement sans modale (le backend nulle la note à la reprise). La note a une valeur métier (motif du litige, traçabilité).
- **D-c2 — Compteur dans le widget « Factures ouvertes » existant.** Le widget aujourd'hui 100% statique gagne le **premier fetch** du dashboard (patron widget bancaire : `onMount` + `catch` **vide**, dégradation par masquage) et affiche « N à rappeler » + lien vers `/invoices/reminders`. Pas de 4e carte.
- **D-c3 — Liens croisés bidirectionnels.** Un lien dans l'en-tête de l'échéancier (`/invoices/due-dates`) vers Rappels, et un lien retour dans l'en-tête de la page Rappels vers l'échéancier.

### ⚠️ Piège n°1 — verrou optimiste après pause/resume (confirmé ground-truth)

`pause_dunning`/`resume_dunning` renvoient un **`DunningPauseResponse`** (`{ invoiceId, dunningPausedAt, dunningPausedNote, version }`), **PAS** un `InvoiceResponse` complet — contrairement à `mark_as_paid` qui renvoie la facture entière (`invoices.rs:1002`) et permet `invoice = await markInvoicePaid(...)`. Or **la pause incrémente `version`** (`version = version + 1`, repo `invoices.rs:1665/1679`). Si l'UI ne ré-applique pas la nouvelle `version` à son état local, la **prochaine** action de la fiche (mark-paid `:215`, unmark `:243`, validate, delete — toutes envoient `version: invoice.version`) prendra un **409 OPTIMISTIC_LOCK_CONFLICT**. **Obligatoire** : après pause/resume, faire `invoice = { ...invoice, version, dunningPausedAt, dunningPausedNote }` (les 3 champs de la réponse). Le patron de récupération 409 existant (`+page.svelte:222-229` : toast + `invoice = await getInvoice(id)`) reste le filet.

### Hors scope (garde-fous)

- **Lien vers la balance âgée** → **21-7** (`/reports` n'a aucune synchro URL, l'onglet n'est pas adressable — reporté par le plan).
- **Deep-link vers la fiche contact** : IMPOSSIBLE (`/contacts/[id]` n'existe pas, édition en modale). Ne pas tenter.
- **Cumuls par contact** (TTC/frais/jours de retard) : `ReminderCandidateResponse` ne les porte pas (L21-8). Le compteur dashboard est un simple **nombre de factures**, pas un montant.
- **Câblage i18n de la page liste `/invoices`** → issue #255. **Boutons d'action sans nom accessible** → issue #256. **Filtre statut sans reset offset** → issue #257. Ne pas corriger ici.
- **Aucun backend** : ne PAS recréer d'endpoint (history/pause/resume livrés 21-5a).

## Acceptance Criteria

### A. Feature `reminders/` — wrappers & types manquants

1. **`frontend/src/lib/features/reminders/reminders.types.ts`** gagne (les autres existent déjà) :
   - `DunningPauseResponse { invoiceId: number; dunningPausedAt: string | null; dunningPausedNote: string | null; version: number }`.
   - `PauseDunningRequest { version: number; note: string | null }` ; `ResumeDunningRequest { version: number }`.
   - **`ReminderResponse` existe déjà** (`:62-77`, conforme au DTO backend) → **le réutiliser tel quel** pour l'historique (`Vec<ReminderResponse>` → `ReminderResponse[]`). Ne PAS le redéfinir.

2. **`frontend/src/lib/features/reminders/reminders.api.ts`** gagne 3 wrappers (via `apiClient`, calque des existants) :
   - `listReminderHistory(invoiceId): Promise<ReminderResponse[]>` → `get('/api/v1/invoices/${invoiceId}/reminders')`.
   - `pauseDunning(invoiceId, payload: PauseDunningRequest): Promise<DunningPauseResponse>` → `put('/api/v1/invoices/${invoiceId}/dunning-pause', payload)`.
   - `resumeDunning(invoiceId, payload: ResumeDunningRequest): Promise<DunningPauseResponse>` → `put('/api/v1/invoices/${invoiceId}/dunning-resume', payload)`.
   - Re-exports **explicites** (pas de wildcard) à ajouter dans `index.ts` **et** dans les blocs `export type/…` de `reminders.api.ts`.

3. **Tests vitest** `reminders.api.test.ts` (étendre) : chaque nouveau wrapper appelle le bon chemin/méthode/body ; `pauseDunning(42, { version: 3, note: 'litige' })` → `put('/api/v1/invoices/42/dunning-pause', { version: 3, note: 'litige' })` ; `listReminderHistory(42)` → `get('/api/v1/invoices/42/reminders')`.

### B. Fiche facture — historique des rappels

4. **Section « Historique des rappels »** insérée dans `frontend/src/routes/(app)/invoices/[id]/+page.svelte` **entre la fin de la table des lignes (`:621`) et la fermeture du `space-y-6` (`:622`)**, dans le bloc `{:else if invoice}`. Chargée au `onMount` (après `getInvoice`) via `listReminderHistory(id)`, échec toléré (`catch` → liste vide, pas de toast bloquant — l'historique est secondaire). **RBAC** : l'endpoint est tous-rôles-auth, donc pas de garde de rôle nécessaire pour l'historique (Consultation peut le lire).

5. **Composant `ReminderHistory.svelte`** (sous `features/reminders/`, namespace `reminders-*`) : reçoit `reminders: ReminderResponse[]`. Affiche un tableau/liste **triée telle que reçue** (backend `ORDER BY sent_at DESC` — plus récent d'abord, ne pas re-trier). Par ligne : date (`sentAt.slice(0,10)`), niveau (`reminders-level-name` avec `{ $level }`), canal (`channel` : « E-mail » / « Manuel » via clés i18n), frais (`feeAmount` formaté `formatInvoiceTotal`), destinataire (`sentTo ?? '—'`). **Un rappel annulé** (`cancelledAt !== null`) est visuellement distingué (texte barré / mention « annulé le … ») — pas de champ booléen, tester la non-nullité de `cancelledAt`. Liste vide → message « Aucun rappel envoyé ».

### C. Fiche facture — toggle de suspension

6. **État de suspension exposé** : la fiche lit désormais `invoice.dunningPausedAt` (déjà désérialisé, jamais rendu aujourd'hui). Si non-null → afficher le **badge « Suspendu »** (réutiliser `DunningPausedBadge` de 21-6a, `features/invoices/`, avec la note en infobulle) près du titre/statut, et la **note** (`dunningPausedNote`) dans la section historique ou près du badge.

7. **Boutons dans la barre d'actions `validated`** (`+page.svelte:444-526`, **RBAC `canManage`** = Admin|Comptable déjà dérivé `:46-48`) :
   - Si `dunningPausedAt === null` → bouton **« Suspendre les rappels »** → ouvre `DunningPauseDialog` (modale note).
   - Si `dunningPausedAt !== null` → bouton **« Reprendre les rappels »** → appelle directement `resumeDunning` (pas de modale, D-c1).

8. **`DunningPauseDialog.svelte`** (présentationnel, patron `ManualReminderDialog` — n'appelle JAMAIS l'API, émet `onConfirm(note)`) : champ **note optionnel** (`<textarea>`, borne 500 = `PAUSE_NOTE_MAX`), `$props.id()` pour les IDs DOM, reset à l'ouverture, `submitting` en prop. **Anti-double-submit** : flag parent `pauseSubmitting`, garde de ré-entrance, boutons `disabled`, `onOpenChange` non-fermable en vol (patron `SendEmailDialog`, cohérent 21-6b).

9. **Handlers `confirmPause(note)` / `confirmResume()`** dans `+page.svelte` :
   - `pauseDunning(id, { version: invoice.version, note })` puis **`invoice = { ...invoice, version: res.version, dunningPausedAt: res.dunningPausedAt, dunningPausedNote: res.dunningPausedNote }`** (piège n°1 — ré-appliquer la version).
   - `resumeDunning(id, { version: invoice.version })` idem (la reprise nulle `dunningPausedAt` **et** `dunningPausedNote` → `res` les porte à null).
   - Recharger l'historique après un toggle (une reprise/suspension ne crée pas de ligne d'historique, mais garder l'UI cohérente).
   - **Codes d'erreur** (toast `notifyError` traduit, patron des autres handlers de la page) : `OPTIMISTIC_LOCK_CONFLICT` (409) → toast + `invoice = await getInvoice(id)` (refetch, patron `:222-229`) ; `INVOICE_NOT_PAUSED` (422, reprise d'une facture non suspendue — race UI) → toast + refetch ; `VALIDATION_ERROR` (400) → toast ; `NOT_FOUND` (404) → toast. Anti-double-submit couvre le double-clic.

### D. Dashboard — compteur « à rappeler »

10. **Widget « Factures ouvertes »** (`frontend/src/routes/(app)/+page.svelte:76-93`, aujourd'hui statique) gagne le **premier fetch du dashboard**, patron widget bancaire (`:13-26`) :
    - `authState` importé, `canManage = $derived(role === 'Admin' || role === 'Comptable')`.
    - `onMount` : **si `!canManage`, ne PAS fetcher** (un Consultation prendrait un 403 sur `/dunning/reminders` Comptable+). Sinon `try { const res = await listReminders(); reminderCount = res.groups.reduce((n, g) => n + g.invoices.length, 0); } catch { /* silencieux, pas de toast — un widget qui échoue ne pollue pas l'accueil */ } finally { reminderLoaded = true; }`.
    - Affichage : si `canManage && reminderLoaded && reminderCount > 0` → « **N** facture(s) à rappeler » (clé `homepage-reminders-count` avec `{ $n }`) + lien `href="/invoices/reminders"`. Sinon le widget garde son état actuel (empty-state + lien « Créer une facture »).
    - `data-testid="homepage-reminders-count"` sur la valeur.

11. **Le compteur est un NOMBRE DE FACTURES** (somme des `invoices.length`), pas un montant (L21-8 — aucun cumul). Ne pas afficher de total CHF.

### E. Liens croisés échéancier ↔ Rappels

12. **En-tête échéancier** (`frontend/src/routes/(app)/invoices/due-dates/+page.svelte:299-301`) : ajouter dans le `<div class="mb-6">` un lien/bouton vers `/invoices/reminders` (`<Button href=…>` déjà importé, ou `goto`). Libellé `due-dates-link-reminders` (« Voir les rappels »).

13. **En-tête page Rappels** (`frontend/src/routes/(app)/invoices/reminders/+page.svelte:257-258`) : ajouter un lien/bouton retour vers `/invoices/due-dates`. Libellé `reminders-link-due-dates` (« Voir l'échéancier »). **Attention namespace** : ce fichier est une **route** (hors lint `features/`) — la clé peut être `reminders-*` sans souci (déjà le namespace de la page).

### F. i18n

14. **Nouvelles clés dans les 4 FTL** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), traductions réelles FR/DE/IT/EN :
    - Historique (`reminders-history-*`) : titre de section, en-têtes de colonnes, canal e-mail/manuel, « annulé le », « aucun rappel ».
    - Suspension (`reminders-pause-*` / `reminders-resume-*`) : boutons « Suspendre/Reprendre les rappels », titre + corps + note du dialog, confirmation.
    - Erreurs toggle : `INVOICE_NOT_PAUSED` (« Cette facture n'est plus suspendue »), le reste retombe sur `err.message`.
    - Dashboard : `homepage-reminders-count` (`{ $n }`).
    - Liens : `due-dates-link-reminders`, `reminders-link-due-dates`.
    **Placement lint** : les composants sous `features/reminders/` (`ReminderHistory`, `DunningPauseDialog`) ne peuvent utiliser que `reminders-*` (piège #30) — respecté par construction. Les clés `homepage-*` (route dashboard) et `due-dates-*` (route échéancier) sont hors périmètre du lint.

### G. Tests

15. **E2E** — étendre `frontend/tests/e2e/invoices.spec.ts` (il contient déjà le describe « suspension des rappels 21-6a » et le helper `pauseInvoiceViaApi:110-127`). Scénarios :
    - **Toggle suspension bout-en-bout** : créer facture validée → `goto('/invoices/{id}')` → cliquer « Suspendre les rappels » → note dans la modale → confirmer → badge « Suspendu » visible → cliquer « Reprendre » → badge disparaît. **Vérifie l'absence de 409 sur une action suivante** : après pause, tenter « Marquer payée » (ou re-suspendre) → doit réussir (prouve que la version a été ré-appliquée — piège n°1).
    - **Historique** : pré-peupler via `recordManualReminder` (l'API — le manuel n'exige PAS une facture échue, seulement validée+non-payée) → `goto('/invoices/{id}')` → la section historique affiche la/les ligne(s), triées, canal « Manuel ».
    - **`data-testid`** nouveaux : `dunning-pause-button`, `dunning-resume-button`, `dunning-pause-confirm`, `reminder-history`, `reminder-history-row`.
16. **E2E dashboard** — un test dans son propre spec `frontend/tests/e2e/homepage-reminders.spec.ts` (le compteur est sur le dashboard, pas la fiche facture — ne pas l'entasser dans `invoices.spec.ts`). Seed une facture échue+éligible → `goto('/')` → `getByTestId('homepage-reminders-count')` affiche ≥ 1. **Rôle Consultation** : le compteur n'est pas fetché (pas de 403 en console) — asserter que le widget garde son empty-state.
    ⚠️ **Piège helper `overdueDate` (M1 validate)** : `overdueDate(days)` (`today − days`, seuil éligibilité niveau 1 = `today − 15j`) est aujourd'hui une fonction **locale non exportée** de `reminders.spec.ts:44`. Un nouveau spec ne peut pas l'importer. **Avant** ce test : **promouvoir `overdueDate` dans `frontend/tests/e2e/helpers/api-fixtures.ts`** (l'ajouter avec `export`, puis remplacer la déclaration locale de `reminders.spec.ts` par un import — DRY, patron `createAndValidateInvoiceViaApi`). Ne PAS dupliquer la fonction.
17. **vitest** : les 3 nouveaux wrappers (AC 3).
18. **axe** (patron scopé) : sur la fiche facture avec section historique + badge, `AxeBuilder().include(...).disableRules(['color-contrast','button-name']).analyze()` → 0 violation dans le sous-arbre de la story (dettes #253/#256 pré-existantes neutralisées). Le badge et les nouveaux boutons **de cette story** doivent être conformes AA (badge en `--color-text`, boutons avec libellé texte — pas icône seule).

### H. Gate & documentation

19. **Gate local complet** (Test Locally First) :
    ```sh
    cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build
    cd frontend && npm run test:e2e     # PAS dans la CI → critique
    # backend : aucun code Rust touché (FTL seulement) → fmt/clippy triviaux + kesh-i18n
    cargo fmt --all -- --check && cargo build --workspace --all-targets && cargo clippy --workspace --all-targets -- -D warnings && cargo test -p kesh-i18n
    ```
    ⚠️ Rappel : backend E2E contre `kesh_e2e` migré + SMTP factice ; `npm run build` avant chaque run E2E ; `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` ; ne jamais piper le runner. `cd frontend` explicite (le cwd peut errer après une tâche de fond — leçon 21-6b).
20. **CHANGELOG** `[Non publié]` → `Ajouté` : historique des rappels + suspension sur la fiche facture, compteur « à rappeler » au tableau de bord, navigation échéancier ↔ rappels. **Manuels → 21-8.** **README** : Epic 21 déjà 🚧, aucun changement de statut d'epic.

## Tasks / Subtasks

- [ ] **T1 — Wrappers & types `reminders/`** (AC: 1, 2, 3, 17) — `DunningPauseResponse`+requests, `listReminderHistory`/`pauseDunning`/`resumeDunning`, re-exports explicites, vitest.
- [ ] **T2 — Historique sur la fiche facture** (AC: 4, 5) — `ReminderHistory.svelte`, insertion `:621-622`, chargement toléré, `cancelledAt` distingué.
- [ ] **T3 — Toggle suspension** (AC: 6, 7, 8, 9) — badge `dunningPausedAt`, boutons Suspendre/Reprendre, `DunningPauseDialog` (note, anti-double-submit), handlers avec **ré-application de version (piège n°1)** + codes d'erreur.
- [ ] **T4 — Compteur dashboard** (AC: 10, 11) — widget « Factures ouvertes » + 1er fetch (patron bancaire, catch silencieux), garde `canManage` (pas de 403 Consultation), compteur = somme `invoices.length`.
- [ ] **T5 — Liens croisés** (AC: 12, 13) — échéancier → Rappels et retour.
- [ ] **T6 — i18n 4 FTL** (AC: 14).
- [ ] **T7 — E2E + axe** (AC: 15, 16, 18) — toggle bout-en-bout (dont **anti-régression 409**), historique, compteur dashboard (+ rôle Consultation), axe scopé. **Prérequis : promouvoir `overdueDate` dans `api-fixtures.ts` (M1).**
- [ ] **T8 — Gate complet + CHANGELOG** (AC: 19, 20).

## Dev Notes

### Pièges, par ordre de coût

1. **Verrou optimiste après pause/resume (piège n°1).** `DunningPauseResponse` ≠ `InvoiceResponse` : ré-appliquer `{ version, dunningPausedAt, dunningPausedNote }` à l'état `invoice`, sinon 409 sur la prochaine action. Test anti-régression obligatoire (AC 15). C'est LE piège de cette story.
2. **RBAC du compteur dashboard.** `/dunning/reminders` est Comptable+. Le widget doit **ne pas fetcher** pour Consultation (garde `canManage` avant l'appel), sinon 403 en console. Le `catch` reste vide (widget qui échoue ≠ toast).
3. **Piège #30 i18n.** `ReminderHistory`/`DunningPauseDialog` sous `features/reminders/` → clés `reminders-*` uniquement. Les routes (dashboard, échéancier) sont hors lint.
4. **Contraste du badge.** Réutiliser `DunningPausedBadge` (21-6a, corrigé `--color-text`), pas le gabarit `PaymentStatusBadge` sous-AA.

### Leçon de review héritée (à appliquer dès le dev)

**Sur un bug d'état/course, chercher le patron structurel** (21-6b : 4 passes de review parce que la remédiation créait le défaut suivant ; convergence via un fix qui rend le défaut impossible par construction). Ici : la ré-application de version est le patron sûr — la faire **immédiatement** après le retour de pause/resume, dans le même handler, pas ailleurs. **Un patch de review vient avec son test.** **Disclosure non sélective** : documenter toutes les déviations.

### Contrats backend (ground-truth, à ne pas re-deviner)

| Action | Méthode + chemin | Requête | Réponse | RBAC |
|---|---|---|---|---|
| Historique | `GET /api/v1/invoices/{id}/reminders` | — | 200 `ReminderResponse[]` (tri `sentAt DESC`) | tous rôles auth |
| Suspendre | `PUT /api/v1/invoices/{id}/dunning-pause` | `{ version, note? }` | 200 `DunningPauseResponse` | Comptable+ |
| Reprendre | `PUT /api/v1/invoices/{id}/dunning-resume` | `{ version }` | 200 `DunningPauseResponse` | Comptable+ |
| Compteur (dérivé) | `GET /api/v1/dunning/reminders` | — | `{ groups }` → Σ `invoices.length` | Comptable+ |
| Peupler historique (test) | `POST /api/v1/invoices/{id}/reminders/manual` | `{ levelNumber, sentAt, note? }` | 201 `ReminderResponse` | Comptable+ |

Codes d'erreur toggle : 409 `OPTIMISTIC_LOCK_CONFLICT`, 422 `INVOICE_NOT_PAUSED` (reprise d'une non-suspendue), 400 `VALIDATION_ERROR` (version<0 / note>500), 404 `NOT_FOUND`. `sentAt` du manuel = `NaiveDateTime` `T12:00:00` (bug #249).

### Gabarits (chemins vérifiés)

- **Dialog présentationnel + note** : `ManualReminderDialog.svelte` (feature reminders, `$props.id()`, note, anti-double-submit parent).
- **Récupération 409** : `invoices/[id]/+page.svelte:222-229` (toast + `getInvoice(id)`).
- **Widget fetch dashboard** : `+page.svelte:13-26` (onMount + catch vide + `finally` flag + garde `{#if loaded && …}`).
- **Badge** : `DunningPausedBadge.svelte` (21-6a, contraste AA).
- **RBAC page** : `reminders/+page.svelte:39-41` (`canManage`).
- **Barre d'actions validée** : `invoices/[id]/+page.svelte:444-526`.

### Project Structure Notes

**Nouveaux fichiers** :
- `frontend/src/lib/features/reminders/ReminderHistory.svelte`, `DunningPauseDialog.svelte`

**Modifiés** :
- `frontend/src/lib/features/reminders/reminders.types.ts` (+`DunningPauseResponse`/requests), `reminders.api.ts` (+3 wrappers), `reminders.api.test.ts`, `index.ts`
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (historique + toggle)
- `frontend/src/routes/(app)/+page.svelte` (compteur dashboard)
- `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` + `reminders/+page.svelte` (liens croisés)
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
- `frontend/tests/e2e/invoices.spec.ts` (+ éventuel dashboard spec)
- `CHANGELOG.md`

**Décompte** : frontend pur, ~5 surfaces (feature reminders, fiche facture, dashboard, échéancier, page Rappels). Aucun backend, aucune migration, aucun nouveau crate.

### References

- [Source: `epic-21-echeances-relances.md` — 21-6c (D25 compteur dashboard, liens croisés), L21-8 (pas de cumuls), D10 (suspension)]
- [Source: `21-5a-…md` — endpoints history/pause/resume/manual livrés]
- [Source: `21-6a-…md` — `dunningPausedAt` sur `InvoiceResponse`, `DunningPausedBadge` contraste AA]
- [Source: `21-6b-…md` — feature `reminders/`, patron dialog présentationnel + anti-double-submit, LEÇON fix structurel > incrémental sur bug d'état, disclosure non sélective]
- [Source: `CLAUDE.md#Test Locally First`, `#Review Iteration Rule`, `#Issue Tracking Rule` ; `feedback_no_secure_context_apis_http_lan` (`$props.id()`)]
- [Source: GitHub #231 ; #255/#256/#257 (dettes pré-existantes à ne pas corriger)]

## Change Log — validate

### Pass 1 (Sonnet, 2026-07-17) — 1 MEDIUM → patché

Auteur de la spec : Opus. Reviewer orthogonal : Sonnet. Verdict **GO-ajusté**. Vérification ground-truth **exhaustive** (81 outils) — le finding a été re-vérifié par l'orchestrateur (`grep`) avant patch.

- **M1 (MEDIUM) — `overdueDate` ambigu pour l'AC 16.** L'AC disait « étendre/créer » un test dashboard réutilisant `overdueDate(25)` — or c'est une fonction **locale non exportée** de `reminders.spec.ts:44` (grep confirmé). Un nouveau spec l'important échouerait à la compilation, ou le dev dupliquerait la fonction sans le savoir. **Patch** : AC 16 tranchée — test dans son propre spec `homepage-reminders.spec.ts`, avec **promotion préalable** de `overdueDate` dans `api-fixtures.ts` (export + remplacement de la déclaration locale par un import, DRY). Répercuté en T7 + Dev Notes.

**Vérifications positives (grep/Read, Sonnet) — profondeur inhabituelle** : le **piège n°1** confirmé exact (pause/resume renvoient `DunningPauseResponse` pas `InvoiceResponse` ; `set_dunning_pause` fait `version = version + 1` sur les 2 branches ; mark/unmark envoient `version: invoice.version`) ; les 5 contrats backend (chemins/méthodes/DTO/RBAC **structurel** pas juste commentaire) ; `INVOICE_NOT_PAUSED` = **422** confirmé ; wrappers/types absents confirmés, `ReminderResponse` présent `:62-77` ; point d'insertion `:621-622` (nesting vérifié div par div) ; widget dashboard statique + `authState` absent ; liens croisés + `canManage` ; lint #30 (`features/reminders/` sans entrée `KNOWN_VIOLATIONS` → `reminders-*` seule contrainte ; `homepage-*`/`due-dates-*` hors périmètre) ; aucune clé i18n proposée n'existe déjà. **Aucun défaut de contrat ou d'architecture.**

### Pass 2 (Haiku, 2026-07-17) — 0 finding → **CONVERGÉ**

Contexte frais, prémunie contre le mode d'échec « auditer la spec comme une implémentation ». **0 erreur de catégorie, 0 hallucination.** Verdict **GO**. Absorption du patch P1 confirmée (promotion `overdueDate`). Vérifications indépendantes concordantes : routes pause/resume/historique existent, DTO alignés, `INVOICE_NOT_PAUSED` = 422, la reprise nulle bien **les deux** colonnes (`SET dunning_paused_at = NULL, dunning_paused_note = NULL` — SQL vérifié), points d'insertion `:621-622`/dashboard `:76-93`/échéancier `:299-301` exacts, piège n°1 (AC 9 + Dev Notes alignés, exemple de code correct), RBAC `canManage`, compteur = somme de factures (pas montant).

### Trend & décision — validate

**Passe 1 (Sonnet) : 1 MEDIUM → Passe 2 (Haiku) : 0.** Critère d'arrêt atteint (0 > LOW), budget 2/8. Rotation orthogonale à l'auteur (Opus). Le MEDIUM de P1 re-vérifié ground-truth par grep avant patch. **Spec scellée, prête pour `bmad-dev-story 21-6c`.**

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
