# Story 28.5 : Une transaction rejetée redevient atteignable

Status: ready-for-dev

<!-- Fiche créée en autonomie le 2026-10-13 (bmad-create-story, Opus 5.5), sur `story/28-5` = `origin/main` 5aad8b35.
     Issue : #526 (bug, P2). Epic : planning-artifacts/epic-28-recette-v0-13-2.md § 28-5.
     Choix : C-28-5-1 à C-28-5-16 au registre epic-28-choix-autonomes.md. -->

## Story

En tant que **Comptable** qui a rejeté une proposition de réconciliation — par erreur, ou pour la rapprocher plus
tard autrement —,
je veux **retrouver la transaction rejetée et annuler son rejet depuis l'interface**,
afin qu'elle **revienne dans la liste de la réconciliation**, où je peux de nouveau l'accepter, l'affecter
manuellement ou l'éclater — au lieu de devoir supprimer et réimporter tout son relevé.

## Le défaut, vérifié au code (2026-10-13)

- *Rejeter* (`POST /api/v1/reconciliation/reject`) pose `auto_match_rejected_at = NOW(3)`
  (`crates/kesh-api/src/routes/reconciliation.rs:3355-3359`, `reject_batch`).
- La liste des propositions l'exclut : `find_pending_transactions_for_account` filtre
  `AND auto_match_rejected_at IS NULL` (`crates/kesh-db/src/repositories/reconciliation.rs:182`).
- **Aucun** fichier du frontend ne lit le marqueur : `grep -rn "autoMatchRejectedAt\|auto_match_rejected" frontend/src`
  ne rend rien. Le détail d'un import (`frontend/src/routes/(app)/bank-import/[id]/+page.svelte:109-148`) affiche la
  transaction avec le statut brut `pending`, **sans aucune action** : le seul bouton de la colonne *Rapprochement*
  est *Annuler le rapprochement*, réservé à `tx.status === 'reconciled'` (l. 140).
- Les seuls gestes qui remettent le marqueur à `NULL` sont le rapprochement manuel (`:3803`), l'éclatement
  (`:4260`), les propositions `split` et `rule` de l'acceptation (`:2540`, `:2992`) et l'annulation d'un
  rapprochement (`crates/kesh-db/src/repositories/reconciliation_cancel.rs:340`). Or une transaction rejetée n'est
  **ni proposée** (donc ni `split` ni `rule` ni *Affecter manuellement* ni *Éclater* à l'écran : ces boutons ne
  vivent que sur les lignes de la liste des propositions, `ReconciliationProposals.svelte:375-397`), **ni
  rapprochée** (donc pas d'annulation). Le manuel le dit lui-même : « aucun geste de l'interface ne la remet
  ensuite dans la liste » (`docs/manual/fr/user-manual.tex:1947`).
- Seule issue aujourd'hui : supprimer l'import (si aucune de ses transactions n'est rapprochée, Story 27-1) et le
  réimporter.

## Périmètre minimal (leçon de l'epic, posée d'entrée)

**Ce qui ferme #526, et rien d'autre :**

1. **Retrouver** — le détail de l'import, qui montre déjà la transaction rejetée, la **marque** « Rejetée le … »
   (tous rôles), à partir d'un champ `autoMatchRejectedAt` ajouté à la transaction exposée (C-28-5-1, C-28-5-8).
   La page *Réconciliation* dit en une phrase où la retrouver (C-28-5-10).
2. **Annuler le rejet** — un bouton *Annuler le rejet* sur cette ligne (Comptable, Administrateur), qui appelle une
   route neuve `POST /api/v1/reconciliation/transactions/{id}/unreject` (C-28-5-3) : le marqueur repasse à `NULL`,
   la transaction **revient dans la liste des propositions**, le geste est **tracé** (`reconciliation.unrejected`).
3. Le **manuel** redevient affirmatif ; l'API et le CHANGELOG le documentent.

**Exclu, et écrit comme tel :**

| Exclu | Pourquoi | Suite |
|---|---|---|
| Filtre ou onglet « Rejetées » sur la page *Réconciliation* (« filtre ou onglet » de l'issue et de l'epic) | une route de lecture, une fonction de dépôt et un composant de plus pour un besoin que le détail de l'import couvre déjà ; écart assumé, C-28-5-1 | réversible ; à rouvrir en CR si la recette le réclame |
| Rapprocher **directement** depuis la ligne rejetée (*Affecter manuellement*, *Éclater* dans le détail d'import) | annuler le rejet ramène la transaction là où les trois gestes existent déjà (C-28-5-2) | — |
| Annulation de rejet **par lot** (case à cocher, route batch) | un geste par ligne suffit à fermer le défaut ; une route unitaire évite un `{ accepted, failed }` neuf (C-28-5-3) | — |
| Traduire le détail d'import (libellés français en dur, statut brut `pending`) | défaut existant, hors de #526 | **#339** (KF-046) |
| Le rôle Consultation sur la page *Réconciliation* (le menu y mène, `+layout.svelte:109`, mais `GET /proposals` est réservé au Comptable, `lib.rs:647-650`) | existant, **non observé** à l'exécution par cette fiche ; la page de Consultation n'est pas changée | à observer en recette (T9) ; issue à ouvrir par l'orchestrateur si un `403` brut s'affiche |
| Garde « compte bancaire clos » sur la route neuve | la clôture n'existe pas encore ; un compte **archivé** n'a aucune transaction (archivage refusé en `412 BANK_ACCOUNT_HAS_TRANSACTIONS`), le cas est inatteignable (C-28-5-13) | **à inscrire à l'inventaire de la 28-4** (chemins d'écriture qui prennent un `bank_account_id`) |
| Manuel d'administration | aucune route d'administration, aucune configuration ; l'API se documente dans `docs/api-external.md` | la release écrira, si besoin, son § « Passer à la 0.13.3 » |

### Décompte des modules (règle de découpage préventif)

Quatre **paquets** : `kesh-api`, `kesh-db` (**commentaires seulement**), `kesh-i18n`, `frontend`. À la granularité
de l'exemple du `CLAUDE.md` (« `kesh-api/routes/invoices`, `frontend/src/features/invoices` »), **huit** sites de
premier niveau : `kesh-api/routes/reconciliation`, `kesh-api/routes/bank_imports` (un champ),
`kesh-api` socle (`lib.rs`, `errors.rs`, `audit_labels.rs` — une ligne, une variante, un code), `kesh-db` (doc),
`kesh-i18n` (quatre catalogues), `frontend/features/reconciliation`, `frontend/features/bank-import` (un champ de
type), `frontend/routes/(app)/bank-import/[id]`. **Plus de cinq : dérogation motivée (C-28-5-11)** — un seul
module porte une logique neuve (la route), les sept autres reçoivent chacun quelques lignes mécaniques ; un
découpage donnerait une story « route sans écran » qui ne ferme pas #526 et une story « écran » qui ne se teste pas
sans elle.

### Régions touchées — à l'attention des 28-6 et 28-7 (développées après, en série)

| Fichier | Région | Nature |
|---|---|---|
| `crates/kesh-api/src/routes/reconciliation.rs` | en-tête du module (l. 1-25, une puce) ; **section neuve entre la fin de `cancel_reconciliation_once` (l. 4586) et `#[cfg(test)] mod period_lock_tests` (l. 4588)** ; `cancel_reconciliation_once` (l. 4507-4586) **refactoré** pour partager deux auxiliaires (C-28-5-12) ; un `mod unreject_classification_tests` en fin de fichier | `get_proposals`, `accept_*`, `post_reject`, `reject_batch`, `classify_batch_transaction` **intouchés** |
| `crates/kesh-api/src/lib.rs` | `comptable_routes`, après la route `…/transactions/{id}/cancel` (l. 676-679) | une route |
| `crates/kesh-api/src/errors.rs` | variante neuve à côté de `ReconciliationTransactionNotPending` (l. 921-930) et son bras (l. 2663-2677) | |
| `crates/kesh-api/src/routes/bank_imports.rs` | `TransactionResponse` et son `From` (l. 250-288) | un champ |
| `crates/kesh-api/tests/reconciliation_e2e.rs` | **section neuve en fin de fichier** (après l. 7526) | tests B1-B7 |
| `crates/kesh-db/src/repositories/reconciliation.rs` | doc-comments : en-tête (l. 23-25), `find_by_id_for_account_for_update` (l. 393, « Seul appelant ») | la 28-7 touche `find_unpaid_invoices_for_window` (l. 90-150) : régions disjointes |
| `crates/kesh-db/src/repositories/bank_imports.rs` | doc-comment d’`is_reconciled` (l. 346-355, liste des appelants) | |

## Acceptance Criteria

**AC1 — La transaction exposée porte sa date de rejet.** `TransactionResponse`
(`crates/kesh-api/src/routes/bank_imports.rs:250`) gagne `auto_match_rejected_at: Option<chrono::DateTime<Utc>>`,
sérialisé `autoMatchRejectedAt` — chaîne ISO 8601 UTC, ou `null` —, rempli dans le `From` par
`t.auto_match_rejected_at.map(|d| d.and_utc())` (convention de `reject_batch`, `:3401-3419`). Le champ est donc
présent partout où ce type sert : détail d'un import (`GET /api/v1/bank-imports/{id}`), `GET
/api/v1/reconciliation/transactions/{id}` (aplati, `:4412`) et réponse de `…/cancel` (`:4471`). Champ **additif** :
aucun champ existant ne change.

**AC2 — Annuler un rejet.** `POST /api/v1/reconciliation/transactions/{id}/unreject`, monté dans `comptable_routes`
(Comptable+), sans corps, répond **`204`** quand la transaction est **rejetée et non rapprochée**. Effets, dans
**une** transaction, sous le **verrou nommé du compte** (`with_account_lock`, C-28-5-4) :
- `auto_match_rejected_at = NULL`, `version = version + 1`, `updated_at = NOW(3)` ; `status`, `matched_entry_id`
  et toute autre colonne **inchangés** ; aucune écriture comptable, aucun règlement ;
- la transaction **réapparaît** dans `GET /api/v1/reconciliation/proposals` de son compte ;
- **une** ligne d'audit `reconciliation.unrejected`, `entity_type = 'bank_transaction'`, `entity_id` = la
  transaction, `details = { "previouslyRejectedAt": "<horodatage ISO UTC lu sous verrou>" }`, écrite par
  `NewAuditLogEntry::for_actor(user_id, actor_api_key_id, …)` (C-28-5-6).

**AC3 — Refus, sans rien écrire ni tracer.**

| Cas | Réponse |
|---|---|
| transaction inexistante, ou d'une autre société (lecture **scopée société**, indiscernables) | `404 NOT_FOUND` (`AppError::Database(DbError::NotFound)`, comme `…/cancel`) |
| transaction non rejetée (`auto_match_rejected_at` nul) — dont un second appel après un succès | `409 RECONCILIATION_TRANSACTION_NOT_REJECTED`, `details: { bankTransactionId }` |
| transaction **rapprochée** au sens d'`is_reconciled` (`kesh_db::repositories::bank_imports::is_reconciled`), **même si** son marqueur est posé (état forgé, aucun chemin réel ne le produit) | `409 RECONCILIATION_TRANSACTION_NOT_REJECTED` ; marqueur **intact** |
| verrou du compte tenu au-delà de `LOCK_TIMEOUT_SECS` | `409 RECONCILIATION_ACCOUNT_LOCKED` (inchangé) |

Le classement se fait **sous verrou, sur une lecture verrouillante** (`find_by_id_for_account_for_update`), par une
fonction pure `classify_unreject(row: Option<&BankTransaction>) -> UnrejectOutcome` qui appelle `is_reconciled` —
**aucun second test de statut** (C-28-5-5). L'`UPDATE` qui suit est scopé `id`, `company_id`, `bank_account_id` ;
s'il ne touche pas exactement une ligne (inatteignable : la ligne est verrouillée), `tracing::error!` et
`AppError::Internal` — jamais un `204` muet.

**AC4 — Rôles et clés.** Consultation → `403` (RBAC de `comptable_routes`) ; Comptable et Administrateur → `204` ;
clé d'API `read-write` → `204`, la ligne d'audit **nomme la clé** (`actor_type = 'api_key'`, `actor_api_key_id`) ;
clé `read` → `403 API_KEY_READ_ONLY` (intergiciel existant, non retesté). Route **ouverte aux clés**, comme
`/reject` et `…/cancel` (C-28-5-6).

**AC5 — Le détail d'un import montre le rejet et permet de l'annuler.** Dans
`frontend/src/routes/(app)/bank-import/[id]/+page.svelte`, colonne *Rapprochement*, pour toute transaction
**rejetée** — `isRejected(tx)` : `tx.autoMatchRejectedAt !== null && !isReconciled(tx)`, `isReconciled` étant celui
de `lib/features/bank-import/bank-import-deletion.ts:18` (pas de seconde définition) — un composant neuf
`lib/features/reconciliation/UnrejectTransactionAction.svelte` rend :
- **pour tous les rôles**, la mention « Rejetée le {date} » (`data-testid="detail-tx-rejected-on"`), date par
  `Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium' })` (C-28-5-15) ;
- **si `canManage`** (Admin, Comptable — prop passée par la page, qui la calcule déjà, l. 23-25), un bouton
  *Annuler le rejet* (`data-testid="detail-tx-unreject"`, style de lien `text-primary underline`, **pas** la variante
  destructive : le geste est réversible et sans effet comptable — C-28-5-9 ; **pas** de dialogue de confirmation) ;
- au clic : bouton **désactivé** pendant l'appel (un double-clic n'envoie qu'une requête) ; succès →
  `notifySuccess(reconciliation-unreject-success)` puis `onUnrejected()` (la page recharge son détail : la mention
  disparaît) ; échec → `notifyError(errorMessageOf(e))` (message du serveur, déjà traduit) **et** `onUnrejected()`
  (l'état a pu changer : on relit).

L'appel passe par `unrejectTransaction(id)` dans `lib/features/reconciliation/reconciliation.api.ts`
(`apiClient.post<void>(…/unreject, {})` — le client gère le `204`, `api-client.ts:437-438`). Le type
`BankTransactionDto` (`lib/features/bank-import/bank-import.types.ts:109`) gagne
`autoMatchRejectedAt: string | null` ; `ReconciliationTransactionResponse` et `CancelReconciliationResponse`
(`reconciliation.types.ts:140`, `:160`) **ne sont pas** étendus (aucun lecteur).

**AC6 — La page *Réconciliation* dit où retrouver une transaction rejetée.** `ReconciliationProposals.svelte` rend,
**hors des branches** de la liste (visible quand la liste est vide, chargée ou en erreur — un lot qui rejette la
dernière ligne la vide), un paragraphe discret `data-testid="reconciliation-rejected-hint"` :
`reconciliation-labels-rejected-hint`.

**AC7 — Traductions.** Six clés, **dans les quatre catalogues** `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
(textes au § Dev Notes « Textes ») :
`reconciliation-errors-transaction-not-rejected` (serveur), `audit-log-action-reconciliation-unrejected` (journal
d'audit), `reconciliation-unreject-button`, `reconciliation-unreject-rejected-on` (variable `$date`),
`reconciliation-unreject-success`, `reconciliation-labels-rejected-hint`. Chaque `i18nMsg` a son repli français ;
`sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts:601` (2075 à `5aad8b35`) est **recompté** depuis la source
après ajout, jamais incrémenté de confiance ; `npm run lint-i18n-ownership` vert (les clés `reconciliation-*` ne sont
employées que dans `lib/features/reconciliation/`).

**AC8 — Manuel utilisateur (`.tex` et PDF).** La puce *Rejeter* de `docs/manual/fr/user-manual.tex:1947` est
réécrite (texte au § Dev Notes) : la transaction quitte la liste, **reste** dans le détail de son import avec la
mention « Rejetée le … », et *Annuler le rejet* l'y remet — rôle Consultation : la mention, sans le bouton ; geste
tracé, sans effet comptable. La phrase « aucun geste de l'interface ne la remet ensuite dans la liste » et
« Ne rejetez donc une transaction que si vous ne comptez pas la comptabiliser par ce chemin » **disparaissent** ; la
parenthèse sur l'API et le renvoi à la suppression d'import sont retirés ou reformulés pour rester vrais. PDF
régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et versionné ; contrôle sur le PDF **aplati**
(`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) : « Annuler le rejet » présent, « aucun geste
de l'interface » absent.

**AC9 — API et CHANGELOG.** `docs/api-external.md` : sous-section neuve « Annuler le rejet d'une transaction
bancaire » après « Annuler un rapprochement bancaire » (l. 554 et suivantes) — route, `204`, effets, tableau des
refus de l'AC3, clés `read-write` admises et nommées à l'audit, champ `autoMatchRejectedAt` *(depuis la v0.13.3)* ;
et une phrase au paragraphe du rejet en lot (l. 552) qui renvoie à cette sous-section. `CHANGELOG.md` : section
`## [0.13.3] — Non publié` (créée si absente, par union — précédent C-28-2-12), entrée *Corrigé* « Une transaction
bancaire rejetée se retrouve et son rejet s'annule (#526) » et entrée *Modifié* pour la route et le champ.

**AC10 — Les gardes existantes suivent, et les commentaires disent vrai.**
- `crates/kesh-api/tests/audit_route_registry.rs` : ligne
  `("post", "reconciliation::post_unreject_transaction", Traced, SansEcritureAuJournal)` ; compteurs
  **recomptés** — `LIB_ROUTES` 115 → 116 (l. 645), `traced` 109 → 110 (l. 648, message complété « plus l'annulation
  d'un rejet (28-5, #526) »), `sans_ecriture` 91 → 92 (l. 990, « 90 routes de lib.rs » → 91), total 118 → 119
  (l. 667, l. 997) ; doc du module (l. 27, l. 31-32 : « 115 routes », « (118) ») propagée.
- `crates/kesh-api/src/audit_labels.rs` : `"reconciliation.unrejected"` dans `ACTIONS`, à sa place alphabétique
  (après `reconciliation.split_applied`, avant `reconciliation_rule.applied`).
- Doc-comments : en-tête de `routes/reconciliation.rs` ; « Seul appelant : `reject_batch` » de
  `find_by_id_for_account_for_update` et l'en-tête de `repositories/reconciliation.rs` ; la liste des appelants
  d'`is_reconciled` (`repositories/bank_imports.rs:346-355`) ; le doc-comment de `bank-import-deletion.ts:12-16`
  (« écrite deux fois, et seulement deux » reste vrai : `isRejected` **appelle** `isReconciled`).
- Symptôme grepé après chaque patch (§ *Propagation post-patch*) : `aucun geste`, `sans aucune action`,
  `Seul appelant`, les valeurs `\b115\b`, `\b118\b`, `\b109\b`, `\b91\b`, `\b2075\b`.

**AC11 — E2E Playwright, et recette visuelle.** Spec neuve `frontend/tests/e2e/reconciliation-unreject.spec.ts`
(patron de `reconciliation-deleted-import.spec.ts` : relevé CAMT **synthétique** réécrit à chaque exécution —
références, montants, contrepartie uniques —, import et utilisateur supprimés en `finally`) :
1. import ; rejet d'une transaction **par l'API** (`POST /reconciliation/reject`) ; détail de l'import : la ligne
   porte « Rejetée le » et *Annuler le rejet* ; clic → notification de succès, la mention disparaît ; page
   *Réconciliation* : la transaction est de nouveau listée (`data-tx-id`) ;
2. rôle Consultation (utilisateur créé par l'API, patron `open-items.spec.ts:251-259`) : la mention est visible,
   le bouton **absent** ;
3. la page *Réconciliation* montre l'indication (`reconciliation-rejected-hint`).
**Captures** (B2 de la rétrospective de l'Epic 27), thème clair **et** sombre (`page.emulateMedia({ colorScheme })`,
un test par schéma), dans `process.env.KESH_E2E_CAPTURES_DIR` s'il est défini, sinon `testInfo.outputPath()`
(convention de la 28-1, C-28-1-29 — reprendre son auxiliaire s'il est sur `main` au moment du développement) :
`detail-rejetee`, `detail-apres-annulation`, `reconciliation-indication`, × `clair|sombre` — **six** captures,
déposées hors dépôt dans `/home/gcorbaz/devel/kesh-gate-logs/28-5-captures/`, jamais versionnées, sans
`toHaveScreenshot`. Données synthétiques uniquement (§ 8 de `docs/agents-orchestration.md`).

**AC12 — Mutations.** Les mutations M1-M12 du § Tests font rougir au moins un test chacune ; une survivante est
soit tuée par un test ajouté, soit déclarée **équivalente** avec sa preuve au Debug Log.

## Tasks / Subtasks

- [ ] **T0 — Montage** : bases `kesh_285` / `kesh_e2e_285` (migrations du worktree, seed), port **3038**,
      `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-28-5/target`, `node_modules` par `cp -al` (§ 4 de
      `docs/agents-orchestration.md`). Rafraîchir la base avant tout gate complet.
- [ ] **T1 — Champ `autoMatchRejectedAt`** (AC1) : `TransactionResponse` + `From` ; test B7.
- [ ] **T2 — Route** (AC2, AC3, AC4) : variante `AppError::ReconciliationTransactionNotRejected { bank_transaction_id }`
      et son bras (`409`, code, message `t("reconciliation-errors-transaction-not-rejected", …)`) ; dans
      `routes/reconciliation.rs`, section « Story 28-5 (#526) — annuler un rejet » : `post_unreject_transaction`,
      `unreject_once`, `enum UnrejectOutcome { Unrejected { previously_rejected_at }, NotFound, NotRejected }`,
      `classify_unreject` ; les deux auxiliaires partagés avec l'annulation (C-28-5-12) ; route dans `lib.rs`.
- [ ] **T3 — Gardes et commentaires** (AC10) : registre des routes, `ACTIONS`, doc-comments ; grep du symptôme.
- [ ] **T4 — Tests backend** B1-B8 (§ Tests) ; gate ciblé `binary(reconciliation_e2e)` + `audit_route_registry`
      + `audit_label_registry` + `i18n_e2e` ; **`kesh-db` touché → gate complet** avant tout commit de fin de
      boucle (règle `CLAUDE.md`).
- [ ] **T5 — Frontend** (AC5, AC6) : type, `unrejectTransaction`, `reconciliation-unreject.ts` (`isRejected`),
      `UnrejectTransactionAction.svelte`, page de détail, indication ; fixtures des tests qui construisent un
      `BankTransactionDto` complétées (`npm run check` les signale).
- [ ] **T6 — i18n** (AC7) : six clés × quatre locales ; `sitesTotal` recompté.
- [ ] **T7 — Tests frontend** V1-V5 ; `npm run check`, `lint-i18n-ownership`, `test:unit`, `build`.
- [ ] **T8 — Documentation** (AC8, AC9) : manuel `.tex` + PDF (sous verrou), `api-external.md`, CHANGELOG.
- [ ] **T9 — E2E et captures** (AC11) : spec neuve ; **suite E2E complète** au dernier commit de code (verrou de
      gate, recette de `docs/testing.md` à la lettre), échecs jugés fichier par fichier contre
      `docs/testing.md` § *Les échecs attendus* ; six captures ; observer la page *Réconciliation* sous le rôle
      Consultation et le noter au Debug Log (sans corriger : exclu).
- [ ] **T10 — Mutations** (AC12) M1-M12, sous verrou ; restauration `git checkout -- <f>` **puis `touch <f>`**.
- [ ] **T11 — Comptes rendus** : Dev Agent Record, File List, Change Log ; décomptes recomptés depuis la source,
      périmètre de mesure déclaré.

## Tests

**Backend — section neuve en fin de `crates/kesh-api/tests/reconciliation_e2e.rs`** (montages du fichier :
`setup_company`, `seed_bank_transactions`, `make_new_tx`, `unique_hash`, `create_user`, `forge_jwt`, patron des
clés de `an_api_key_cancels_and_is_named_in_the_audit` l. 4511, patron du verrou tenu de
`post_accept_returns_409_on_account_lock_contention` l. 1435 ; un auxiliaire `unreject(app, jwt, tx_id) -> (u16, Value)`
sur le modèle de `cancel_reco` l. 3578) :

| # | Test | Établit |
|---|---|---|
| B1 | `unreject_puts_a_rejected_transaction_back_in_the_proposals` | rejet par la route ; `204` ; marqueur `NULL`, `version` +1, `status`/`matched_entry_id` inchangés ; la transaction est dans `GET /proposals` (absente avant) ; **une** ligne `reconciliation.unrejected` avec `previouslyRejectedAt` égal au marqueur lu avant |
| B2 | `unreject_refuses_a_transaction_that_is_not_rejected` | jamais rejetée → `409` code ; un second appel après succès → `409` ; ni `version` ni `audit_log` ne bougent |
| B3 | `unreject_refuses_a_reconciled_transaction_even_if_marked` | état **forgé, et dit tel** (`status='reconciled'`, `matched_entry_id`, marqueur posé — patron du test l. 4438) → `409` ; marqueur intact |
| B4 | `unreject_roles_and_scope` | Consultation → `403` ; autre société → `404` et rien d'écrit (comptes de lignes avant/après, patron l. 4385) |
| B5 | `unreject_returns_409_when_the_account_is_locked` | verrou nommé du compte tenu par une autre connexion → `409 RECONCILIATION_ACCOUNT_LOCKED`, marqueur intact |
| B6 | `an_api_key_unrejects_and_is_named_in_the_audit` | clé `read-write` → `204` ; `actor_type = 'api_key'`, `actor_api_key_id` = la clé |
| B7 | `the_import_detail_exposes_the_rejection_date` | `GET /bank-imports/{id}` : `autoMatchRejectedAt` chaîne ISO pour la rejetée, `null` pour l'autre ; `null` après l'annulation |
| B8 | `mod unreject_classification_tests` (unitaire, `routes/reconciliation.rs`) | `classify_unreject` : `None` → `NotFound` ; rapprochée marquée → `NotRejected` ; non marquée → `NotRejected` ; rejetée → `Unrejected` |

**Frontend — Vitest :**

| # | Fichier | Établit |
|---|---|---|
| V1 | `lib/features/reconciliation/reconciliation-unreject.test.ts` | `isRejected` : rejetée `pending` sans lien → vrai ; rapprochée marquée → faux ; non marquée → faux |
| V2 | `lib/features/reconciliation/UnrejectTransactionAction.test.ts` | `canManage=false` : mention, aucun bouton ; `canManage=true` : bouton ; clic → **un** appel, bouton désactivé pendant ; succès → `notifySuccess` + `onUnrejected` ; échec → `notifyError(message du serveur)` + `onUnrejected` |
| V3 | `routes/(app)/bank-import/[id]/bank-import-detail-page.test.ts` | ligne rejetée : mention présente ; ligne non rejetée et ligne rapprochée : absente |
| V4 | `ReconciliationProposals.test.ts` | l'indication est rendue liste vide **et** liste non vide |
| V5 | `reconciliation.api.test.ts` | `unrejectTransaction(7)` poste `{}` à `/api/v1/reconciliation/transactions/7/unreject` |

**Mutations (T10)** — motif vérifié unique avant écriture :

| # | Mutation | Tuée par |
|---|---|---|
| M1 | `classify_unreject` : retirer le bras `is_reconciled` | B3, B8 |
| M2 | `classify_unreject` : retirer le bras « non marquée » | B2, B8 |
| M3 | `UPDATE` : retirer `version = version + 1` | B1 |
| M4 | supprimer l'insertion d'audit | B1 |
| M5 | `for_actor(user_id, None, …)` au lieu de `actor_api_key_id` | B6 |
| M6 | appeler `unreject_once` sans `with_account_lock` | B5 |
| M7 | monter la route dans `authenticated_routes` | B4 |
| M8 | `From` : `auto_match_rejected_at: None` | B7 |
| M9 | `isRejected` sans `!isReconciled(tx)` | V1 |
| M10 | bouton rendu sans condition `canManage` | V2 |
| M11 | `onUnrejected` non appelé sur échec | V2 |
| M12 | indication déplacée dans la branche « liste non vide » | V4 |

## Dev Notes

### Séquence de la route (C-28-5-4, C-28-5-5)

1. Hors verrou : `bank_account_id` de la transaction, lu **scopé société** — auxiliaire partagé avec
   `cancel_reconciliation_once` (`:4514-4524`), extrait sans changer son comportement. Absente → `404`.
2. `pool.begin()`, `with_account_lock(&mut tx, company_id, bank_account_id, LOCK_TIMEOUT_SECS, …)`.
3. Dans la fermeture : `find_by_id_for_account_for_update(&mut **tx_inner, company_id, bank_account_id, id)`
   (lecture **courante** et verrouillante — une `SELECT` ordinaire lirait la vue cohérente, cf. son doc-comment) →
   `classify_unreject`. `NotFound` ou `NotRejected` : `Ok(outcome)` sans rien écrire.
4. `Unrejected` : `UPDATE bank_transactions SET auto_match_rejected_at = NULL, updated_at = NOW(3), version = version + 1
   WHERE id = ? AND company_id = ? AND bank_account_id = ?` ; `rows_affected() != 1` → erreur défensive tracée.
5. Audit `for_actor`, erreur d'insertion convertie **comme** `reject_batch` (`:3443-3455`, `DbError::Sqlx` préservé).
6. Après la fermeture : `Unrejected` → `commit`, `204` ; refus → `rollback`, `404` / `409` ; erreur de verrou ou de
   base → l'auxiliaire de conversion partagé avec l'annulation (le `match` de `:4558-4583`, extrait), qui préserve
   `DbError::Sqlx`.

**Pas de rejeu sur interblocage** (`SansEcritureAuJournal`, comme `post_reject`) : la transaction ne verrouille
qu'**une** ligne de `bank_transactions` puis insère une ligne d'audit ; la suppression d'un import (27-1) verrouille
les lignes de l'import par identifiant croissant **sans** prendre le verrou nommé ; aucun cycle n'est possible
(C-28-5-4). Le verrou nommé est relâché avant le `commit` (`kesh-reconciliation/src/mutex.rs`, propriété commune à
tous les gestes) : un geste concurrent qui obtient le verrou dans cet intervalle peut lire l'état d'avant et
refuser (`RECONCILIATION_ALREADY_REJECTED`) — refus bénin, à relancer, déjà le cas entre tous les gestes.

### Inventaire des sites NON RÉSOLUS (D4-ter)

Classe de défaut : *une transaction rejetée qu'aucun écran ne montre ou ne permet de reprendre.* Ensemble clos,
obtenu par `grep -rn "auto_match_rejected_at" crates --include=*.rs --include=*.sql | grep -v /tests/` et
`grep -rn "autoMatchRejectedAt\|auto_match_rejected" frontend/src` (vide à `5aad8b35`) :

| Site | Rôle vis-à-vis du marqueur | Après la story |
|---|---|---|
| `repositories/reconciliation.rs:182` (`find_pending_transactions_for_account`) | exclut les rejetées des propositions | **voulu** (c'est le sens du rejet) ; l'indication de l'AC6 renvoie au détail |
| `routes/reconciliation.rs:885` (`classify_batch_transaction`) | refuse une rejetée à l'acceptation et au rejet | voulu ; annuler le rejet la rend de nouveau acceptable |
| `routes/reconciliation.rs:3357` (`reject_batch`) | **pose** le marqueur | inchangé |
| `:2540`, `:2992`, `:3803`, `:4260`, `reconciliation_cancel.rs:340` | **remettent** le marqueur à `NULL` | inchangés ; aucun n'est atteignable depuis l'écran pour une rejetée (cf. *Le défaut*) |
| **route neuve** `…/unreject` | remet à `NULL` | **le chemin de l'interface** |
| `routes/bank_imports.rs:250` (`TransactionResponse`) | n'exposait pas le marqueur | **résolu** (AC1) |
| `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` | seul écran qui liste une rejetée | **résolu** (AC5) |
| `exports/csv_tables.rs:740` | exporte la colonne | déjà résolu (export global) |
| page *Imports précédents* (`BankImportList.svelte`) | ne compte pas les rejetées | **angle mort assumé** : l'utilisateur ouvre l'import d'après la date de la transaction ; un compteur serait du périmètre en plus |
| colonne *Statut* du détail (`pending` brut) | ne dit pas « rejetée » | **angle mort assumé**, la mention de l'AC5 le dit à côté ; traduction suivie par #339 |

### Textes (quatre locales)

| Clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|
| `reconciliation-errors-transaction-not-rejected` | Cette transaction n'est pas rejetée : son rejet a peut-être déjà été annulé, ou elle a été rapprochée. | Diese Transaktion ist nicht abgelehnt: Die Ablehnung wurde möglicherweise bereits aufgehoben, oder sie wurde abgestimmt. | Questa transazione non è rifiutata: il rifiuto potrebbe essere già stato annullato, oppure è stata riconciliata. | This transaction is not rejected: its rejection may already have been undone, or it has been reconciled. |
| `audit-log-action-reconciliation-unrejected` | Rejet de réconciliation annulé | Abstimmungsablehnung aufgehoben | Rifiuto della riconciliazione annullato | Reconciliation rejection undone |
| `reconciliation-unreject-button` | Annuler le rejet | Ablehnung aufheben | Annulla il rifiuto | Undo rejection |
| `reconciliation-unreject-rejected-on` | Rejetée le { $date } | Abgelehnt am { $date } | Rifiutata il { $date } | Rejected on { $date } |
| `reconciliation-unreject-success` | Rejet annulé : la transaction est de nouveau proposée à la réconciliation. | Ablehnung aufgehoben: Die Transaktion wird wieder zur Abstimmung vorgeschlagen. | Rifiuto annullato: la transazione è di nuovo proposta per la riconciliazione. | Rejection undone: the transaction is offered for reconciliation again. |
| `reconciliation-labels-rejected-hint` | Une transaction rejetée quitte cette liste. Elle reste dans le détail de son import (Importer des relevés), d'où son rejet s'annule. | Eine abgelehnte Transaktion verlässt diese Liste. Sie bleibt im Detail ihres Imports (Kontoauszüge importieren), wo ihre Ablehnung aufgehoben werden kann. | Una transazione rifiutata lascia questo elenco. Resta nel dettaglio della sua importazione (Importa estratti conto), dove il rifiuto si può annullare. | A rejected transaction leaves this list. It remains in the detail of its import (Import statements), where its rejection can be undone. |

Les libellés de menu cités entre parenthèses sont ceux de `nav-bank-import` (`+layout.svelte:94`), relevés à la
fiche dans les quatre catalogues : « Importer des relevés », « Kontoauszüge importieren », « Importa estratti conto »,
« Import statements ». S'ils changent avant le développement, l'indication s'aligne sur eux.

**Manuel — puce *Rejeter* réécrite (`user-manual.tex:1947`)** :

> \emph{Rejeter} écarte la proposition~: la transaction est marquée comme revue et \textbf{quitte la liste} de la
> réconciliation. Elle reste dans le \textbf{détail de son import} (\emph{Quotidien} → \emph{Importer des relevés},
> liste \emph{Imports précédents}, puis le nom du fichier), avec le statut «~pending~» et la mention «~Rejetée
> le~\dots~». Le bouton \emph{Annuler le rejet} de sa ligne la \textbf{remet dans la liste} de la réconciliation, où
> elle peut de nouveau être acceptée, affectée manuellement ou éclatée~; le geste ne passe aucune écriture et
> s'inscrit au journal d'audit (\emph{Rejet de réconciliation annulé}). Le Comptable et l'Administrateur voient le
> bouton~; le rôle Consultation voit la mention seule. La page \emph{Réconciliation} le rappelle sous la liste.

Vérifier aussi, au même passage, que § *Supprimer un import* (l. 1739, 1745 : « une transaction rejetée ne compte
pas », « rejetées comprises ») reste vrai — il l'est, la story ne change pas la suppression.

### Ce qui doit rester intact

- `POST /reject` et `POST /accept` (27-4) : codes, ordre, `failed[]` — aucune ligne de leurs fonctions ne change.
- `…/cancel` : comportement **identique** après l'extraction des deux auxiliaires ; ses tests (l. 3451-4560 de
  `reconciliation_e2e.rs`) restent verts sans modification.
- `TransactionResponse` : champs existants inchangés ; le détail d'import de la 27-1 (`DeleteBankImportAction`,
  `reconciledCount`) ne lit pas le champ neuf.
- Aucune migration : `kesh_version_min_required` inchangé, aucune ligne au registre d'idempotence, aucun backfill.

### References

- Issue #526 et ses deux commentaires (lentilles R et F de la 15-5c) ; epic § 28-5.
- `crates/kesh-api/src/routes/reconciliation.rs` : `reject_batch` `:3338-3462`, `classify_batch_transaction`
  `:872-897`, annulation `:4400-4586`.
- `crates/kesh-db/src/repositories/reconciliation.rs:330-386` ; `repositories/bank_imports.rs:349-358`.
- `docs/api-external.md:552-575` ; `docs/manual/fr/user-manual.tex:1933-1952`.
- Consignes : `docs/agents-orchestration.md` ; `CLAUDE.md` § *Review Iteration Rule*, § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-13 — Fiche créée en autonomie (bmad-create-story, Opus 5.5) : 12 AC, T0-T11, B1-B8, V1-V5, M1-M12, choix
  C-28-5-1 à C-28-5-16 ; validation P1 à lancer (`28-5-validate-prompt-p1.md`, Sonnet, lentilles R et F).
