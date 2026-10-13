# Story 28.5 : Une transaction rejetée redevient atteignable

## Status

**ready-for-dev** (deux clés) — fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur
`5aad8b35`, branche `story/28-5` ; validation **P1** (Sonnet, lentilles R et F : 0 HIGH, 6 MEDIUM, 14 LOW)
**remédiée** le 2026-10-13, avec **découpage** décidé par l'orchestrateur (C-28-5-17, qui remplace la dérogation
C-28-5-11) ; validation **P2** à lancer (`28-5-validate-prompt-p2.md`, Opus, R et F).

<!-- Issue : #526 (bug, P2). Epic : planning-artifacts/epic-28-recette-v0-13-2.md § 28-5.
     Choix : C-28-5-1 à C-28-5-24 au registre epic-28-choix-autonomes.md (C-28-5-11 et C-28-5-12 remplacés). -->

⛔ **Deux stories, une fiche, deux PR successives** (C-28-5-17, sur les précédents C-26-2-15 et C-28-4-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`28-5a-annuler-rejet-serveur`** | route `…/unreject`, champ `autoMatchRejectedAt`, trace d'audit, messages serveur (deux clés, quatre locales), gardes et doc-comments, tests backend, `docs/api-external.md` | AC1 à AC4, AC7 (a), AC9 (a), AC10, AC12 (a), AC13 | **`refs #526`** — ⛔ jamais `closes` |
| **`28-5b-annuler-rejet-ecran`** | détail d'import (mention, bouton), indication de la page *Réconciliation*, libellé `already-rejected` réécrit, i18n de l'interface, manuel `.tex` + PDF, CHANGELOG, E2E, captures | AC5, AC6, AC7 (b), AC8, AC9 (b), AC11, AC12 (b) | **`closes #526`** |

**Ordre** : la 28-5a se développe et se fusionne **d'abord** ; la 28-5b part de `origin/main` **après** cette fusion.
La clé mère `28-5-transaction-rejetee-atteignable` passe au statut `split`. **Aucune release entre les deux PR** :
entre elles, l'annulation n'existe que par l'API, sans écran, sans manuel ni CHANGELOG — à écrire au Dev Agent Record
de la 28-5a. ⚠️ **À l'attention de l'orchestrateur** : la 28-5b ne touche pas `routes/reconciliation.rs` ; la 28-6
peut donc se rebaser sur la 28-5a pour ce fichier, mais elle partage avec la 28-5b `failed-proposal-label.ts` et la
section `reconciliation-*` des catalogues (cf. § *Régions touchées*) — l'ordre 28-5b / 28-6 est un arbitrage de
l'orchestrateur, non tranché ici.

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
- Depuis la 27-4, une rejetée cochée dans un lot (autre onglet) est refusée par proposition en
  `RECONCILIATION_ALREADY_REJECTED` (`classify_batch_transaction`, `:885`), dont le libellé — « Cette transaction a
  déjà été rejetée : elle n’est plus proposée. » (`fr-CH/messages.ftl:1219`) — ne mène nulle part.
- Seule issue aujourd'hui : supprimer l'import (si aucune de ses transactions n'est rapprochée, Story 27-1) et le
  réimporter.

## Périmètre minimal (leçon de l'epic, posée d'entrée)

**Ce qui ferme #526, et rien d'autre :**

1. **Retrouver** — le détail de l'import, qui montre déjà la transaction rejetée, la **marque** « Rejetée le … »
   (tous rôles), à partir d'un champ `autoMatchRejectedAt` ajouté à la transaction exposée (C-28-5-1, C-28-5-8).
   La page *Réconciliation* dit en une phrase où la retrouver (C-28-5-10, C-28-5-22), et le refus
   `already-rejected` d'un lot dit où annuler le rejet (C-28-5-21).
2. **Annuler le rejet** — un bouton *Annuler le rejet* sur cette ligne (Comptable, Administrateur), qui appelle une
   route neuve `POST /api/v1/reconciliation/transactions/{id}/unreject` (C-28-5-3) : le marqueur repasse à `NULL`,
   la transaction **redevient proposable** (de nouveau listée, dans la limite des 100 plus récentes — cf. *Exclu*),
   le geste est **tracé** (`reconciliation.unrejected`).
3. Le **manuel** redevient affirmatif ; l'API et le CHANGELOG le documentent.

**Exclu, et écrit comme tel :**

| Exclu | Pourquoi | Suite |
|---|---|---|
| Filtre ou onglet « Rejetées » sur la page *Réconciliation* (« filtre ou onglet » de l'issue et de l'epic) | une route de lecture, une fonction de dépôt et un composant de plus pour un besoin que le détail de l'import couvre déjà ; écart assumé, C-28-5-1 — **à porter à la PR de la 28-5b comme écart à la lettre de l'issue**, pour arbitrage de Guy | réversible ; à rouvrir en CR si la recette le réclame |
| La page *Réconciliation* ne liste que les **100** transactions en attente les plus récentes (`DEFAULT_PROPOSALS_LIMIT`, `routes/reconciliation.rs:56` ; `reconciliation.api.ts:18`), sans lire `hasMore` : une transaction plus ancienne dont on annule le rejet est proposable mais **non listée** | défaut préexistant, indépendant de #526 ; les textes de la story le disent (toast, manuel) au lieu de promettre un retour inconditionnel (C-28-5-20) | **#649** (P3, ouverte) — ⚠️ sa résolution devra réécrire `reconciliation-unreject-success` et la puce *Rejeter* du manuel, qui citent la limite |
| Rapprocher **directement** depuis la ligne rejetée (*Affecter manuellement*, *Éclater* dans le détail d'import) | annuler le rejet ramène la transaction là où les trois gestes existent déjà (C-28-5-2) | — |
| Annulation de rejet **par lot** (case à cocher, route batch) | un geste par ligne suffit à fermer le défaut ; une route unitaire évite un `{ accepted, failed }` neuf (C-28-5-3) | — |
| Traduire le détail d'import (libellés français en dur, statut brut `pending`) | défaut existant, hors de #526 | **#339** (KF-046) |
| Le rôle Consultation sur la page *Réconciliation* (le menu y mène, `+layout.svelte:109`, mais `GET /proposals` est réservé au Comptable, `lib.rs:647-650`) | existant, **non observé** à l'exécution par cette fiche ; la page de Consultation n'est pas changée ; l'indication de l'AC6 reste vraie pour ce rôle (C-28-5-22) | à observer en recette (T12) ; issue à ouvrir par l'orchestrateur si un `403` brut s'affiche |
| Garde « compte bancaire clos » sur la route neuve | la clôture n'existe pas encore ; un compte **archivé** n'a aucune transaction (archivage refusé en `412 BANK_ACCOUNT_HAS_TRANSACTIONS`), le cas est inatteignable (C-28-5-13) | **déjà inscrite** à la fiche de la 28-4 (`story/28-4`, `7a4adf64` : W30, R8, M33) — à garder au rebase |
| Manuel d'administration | aucune route d'administration, aucune configuration ; l'API se documente dans `docs/api-external.md` | la release écrira, si besoin, son § « Passer à la 0.13.3 » |

### Décompte des modules (règle de découpage préventif)

Quatre **paquets** : `kesh-api`, `kesh-db` (**commentaires seulement**), `kesh-i18n`, `frontend`. À la granularité
de l'exemple du `CLAUDE.md`, **huit** sites de premier niveau. **Plus de cinq : découpage** (C-28-5-17) —
**28-5a** : `kesh-api/routes/reconciliation`, `kesh-api/routes/bank_imports` (un champ), `kesh-api` socle
(`lib.rs`, `errors.rs`, `audit_labels.rs`), `kesh-db` (doc), `kesh-i18n` (deux clés serveur), plus un doc-comment
de `frontend/.../bank-import-deletion.ts` ; **28-5b** : `frontend/features/reconciliation`,
`frontend/features/bank-import` (un champ de type), `frontend/routes/(app)/bank-import/[id]`, `kesh-i18n` (clés
d'interface). Chaque partie se teste seule : la 28-5a par ses tests Rust et l'API, la 28-5b sur la route fusionnée.

### Régions touchées — à l'attention des 28-4, 28-6 et 28-7

| Fichier | Partie | Région | Nature |
|---|---|---|---|
| `crates/kesh-api/src/routes/reconciliation.rs` | a | en-tête du module (l. 1-25, une puce) ; **section neuve entre la fin de `cancel_reconciliation_once` (l. 4586) et `#[cfg(test)] mod period_lock_tests` (l. 4588)** ; dans `cancel_reconciliation_once`, la **seule** lecture du compte (l. 4514-4524) remplacée par l'appel à l'auxiliaire qui la reçoit à l'identique (C-28-5-18) — son `match` de conversion (l. 4558-4583) **intouché** ; `conclude_locked_attempt` (l. 4312-4395) : **signature** rendue générique et doc-comment complété, corps intouché ; un `mod unreject_classification_tests` en fin de fichier | `get_proposals`, `accept_*`, `post_reject`, `reject_batch`, `classify_batch_transaction` **intouchés** |
| `crates/kesh-api/src/lib.rs` | a | `comptable_routes`, après la route `…/transactions/{id}/cancel` (l. 676-679) | une route |
| `crates/kesh-api/src/errors.rs` | a | variante neuve à côté de `ReconciliationTransactionNotPending` (l. 921-930) et son bras (l. 2663-2677) | |
| `crates/kesh-api/src/audit_labels.rs` | a | `ACTIONS` (l. 155-160) | une ligne |
| `crates/kesh-api/src/routes/bank_imports.rs` | a | `TransactionResponse` et son `From` (l. 250-288) | un champ |
| `crates/kesh-api/tests/reconciliation_e2e.rs` | a | **section neuve en fin de fichier** (après l. 7526) | tests B1-B7, B9 |
| `crates/kesh-api/tests/audit_route_registry.rs` | a | une ligne de registre ; compteurs l. 645, 648, 667, 991, 997 ; doc l. 27, 31-32 | |
| `crates/kesh-api/tests/textes_coherents.rs` | a | `TABLE` de G9, bloc `errors.rs` (l. 685-735) | une ligne |
| `crates/kesh-db/src/repositories/reconciliation.rs` | a | doc-comments : en-tête (l. 23-25), `find_by_id_for_account_for_update` (l. 393, « Seul appelant ») | la 28-7 touche `find_unpaid_invoices_for_window` (l. 90-150) : régions disjointes |
| `crates/kesh-db/src/repositories/bank_imports.rs` | a | doc-comment d’`is_reconciled` (l. 346-355, liste des appelants) | |
| `frontend/src/lib/features/bank-import/bank-import-deletion.ts` | a | doc-comment l. 12-16 (appelants serveur d'`is_reconciled`) | commentaire seulement |
| `crates/kesh-i18n/locales/*/messages.ftl` | a, b | section `reconciliation-*` : a — `reconciliation-errors-transaction-not-rejected`, `audit-log-action-reconciliation-unrejected` ; b — quatre clés d'interface et `reconciliation-failed-already-rejected` **réécrite** (fr-CH l. 1219, de/it/en l. 1156) | la 28-6 y ajoute un libellé : union au rebase |
| `docs/api-external.md` | a | § *Accepter des propositions* (l. 552, une phrase) ; sous-section neuve après « Annuler un rapprochement bancaire » (l. 554 et suiv.) ; ligne `GET /api/v1/bank-imports/{id}` (l. 575) | la 28-6 ajoute un refus au § l. 536-552 : contigu, union |
| `frontend/src/lib/features/reconciliation/failed-proposal-label.ts` | b | repli de `RECONCILIATION_ALREADY_REJECTED` (l. 210-214) | la 28-6 y ajoute un code : union |
| `frontend/src/lib/shared/i18n-keys.test.ts` | b | `sitesTotal` (l. 601) | recompté, jamais incrémenté |
| `docs/manual/fr/user-manual.tex` (+ PDF) | b | § *Accepter ou rejeter les propositions*, puce *Rejeter* (l. 1947) | |
| `CHANGELOG.md` | b | `## [0.13.3] — Non publié` (créée si absente, par union) | |

## Acceptance Criteria

### Partie a — `28-5a-annuler-rejet-serveur` (PR `refs #526`)

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
- la transaction **redevient proposable** : elle figure dans `GET /api/v1/reconciliation/proposals` de son compte
  (dans la limite `limit`, 100 par défaut — cf. *Exclu*, #649 ; les tests portent sur des comptes de quelques
  lignes) ;
- **une** ligne d'audit `reconciliation.unrejected`, `entity_type = 'bank_transaction'`, `entity_id` = la
  transaction, `details = { "previouslyRejectedAt": … }` où la valeur est le marqueur lu sous verrou, converti par
  `.and_utc()` en `DateTime<Utc>` — même conversion, donc **même chaîne** que le champ `autoMatchRejectedAt` de
  l'AC1 (C-28-5-6) —, écrite par `NewAuditLogEntry::for_actor(user_id, actor_api_key_id, …)`. Pas de clé
  `wasPreviouslyRejected` : elle serait toujours vraie ici.

**AC3 — Refus, sans rien écrire ni tracer.**

| Cas | Réponse |
|---|---|
| transaction inexistante, ou d'une autre société (lecture **scopée société**, indiscernables) — hors verrou, ou disparue sous verrou (import supprimé entre-temps) | `404 NOT_FOUND` (`AppError::Database(DbError::NotFound)`, comme `…/cancel`) |
| transaction non rejetée (`auto_match_rejected_at` nul) — dont un second appel après un succès | `409 RECONCILIATION_TRANSACTION_NOT_REJECTED`, `details: { bankTransactionId }` |
| transaction **rapprochée** au sens d'`is_reconciled` (`kesh_db::repositories::bank_imports::is_reconciled`), **même si** son marqueur est posé (état forgé, aucun chemin réel ne le produit) | `409 RECONCILIATION_TRANSACTION_NOT_REJECTED` ; marqueur **intact** |
| verrou du compte tenu au-delà de `LOCK_TIMEOUT_SECS` | `409 RECONCILIATION_ACCOUNT_LOCKED` (inchangé) |

Le classement se fait **sous verrou, sur une lecture verrouillante** (`find_by_id_for_account_for_update`), par une
fonction pure `classify_unreject(row: Option<&BankTransaction>) -> UnrejectOutcome` qui appelle `is_reconciled` —
**aucun second test de statut** (C-28-5-5). L'`UPDATE` qui suit est scopé `id`, `company_id`, `bank_account_id` ;
s'il ne touche pas exactement une ligne, la fermeture rend
`Err(ReconciliationError::Db(DbError::OptimisticLockConflict))` — patron de `post_manual_once` (`:3822-3826`),
converti en `409 OPTIMISTIC_LOCK` par `conclude_locked_attempt` ; jamais un `204` muet. Ce bras est **inatteignable**
(la ligne est verrouillée) : sa mutation M13 est déclarée équivalente (C-28-5-19).

**AC4 — Rôles et clés.** Consultation → `403` (RBAC de `comptable_routes`) ; Comptable et Administrateur → `204` ;
clé d'API `read-write` → `204`, la ligne d'audit **nomme la clé** (`actor_type = 'api_key'`, `actor_api_key_id`) ;
clé `read` → `403 API_KEY_READ_ONLY` (intergiciel existant, non retesté). Route **ouverte aux clés**, comme
`/reject` et `…/cancel` (C-28-5-6).

**AC13 — Cohérence avec la 27-4 : une transaction dont le rejet est annulé se rejette et s'accepte de nouveau.**
Après l'annulation, la transaction passe de nouveau `classify_batch_transaction` (`:872-897`) :
- `POST /reconciliation/reject` du même identifiant → elle est dans `rejected[]`, `failed[]` est vide, le marqueur
  est **reposé** (non nul) ; un nouvel `…/unreject` → `204` (cycle rejeter → annuler → rejeter → annuler) ;
- `POST /reconciliation/accept` d'une proposition `split` sur elle (montage `setup_accept_split_ctx`,
  `reconciliation_e2e.rs:5382`) : **avant** l'annulation → `failed[]` en `RECONCILIATION_ALREADY_REJECTED` ;
  **après** → acceptée, rapprochée (`status = 'reconciled'`, marqueur `NULL`).
L'`UPDATE` d'annulation ne laisse donc aucune colonne dans un état que ces gardes refusent (C-28-5-21).

**AC7 (a) — Messages serveur.** Deux clés, **dans les quatre catalogues**
`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (textes au § Dev Notes « Textes ») :
`reconciliation-errors-transaction-not-rejected` (message du `409`, repli Rust **égal mot pour mot** à la valeur
fr-CH, inscrit à la garde G9 — AC10) et `audit-log-action-reconciliation-unrejected` (journal d'audit, exigé par
`audit_label_registry.rs` pour l'entrée d'`ACTIONS`).

**AC9 (a) — API.** `docs/api-external.md` : sous-section neuve « Annuler le rejet d'une transaction bancaire » après
« Annuler un rapprochement bancaire » (l. 554 et suivantes) — route, `204`, effets, tableau des refus de l'AC3 (dont
le `409 OPTIMISTIC_LOCK` défensif, dit inatteignable), clés `read-write` admises et nommées à l'audit, champ
`autoMatchRejectedAt` *(depuis la v0.13.3)*, rappel de la limite de `GET …/proposals` ; une phrase au paragraphe du
rejet en lot (l. 552) qui renvoie à cette sous-section ; et `autoMatchRejectedAt` cité à la ligne
`GET /api/v1/bank-imports/{id}` (l. 575).

**AC10 — Les gardes existantes suivent, et les commentaires disent vrai.**
- `crates/kesh-api/tests/audit_route_registry.rs` : ligne
  `("post", "reconciliation::post_unreject_transaction", Traced, SansEcritureAuJournal)` ; compteurs
  **recomptés** — `LIB_ROUTES` 115 → 116 (l. 645), `traced` 109 → 110 (l. 648, message complété « plus l'annulation
  d'un rejet (28-5a, #526) »), `sans_ecriture` 91 → 92 (l. 991, « 90 routes de lib.rs » → 91, l. 992), total
  118 → 119 (l. 667, l. 997) ; doc du module (l. 27, l. 31-32 : « 115 routes », « (118) ») propagée.
- `crates/kesh-api/src/audit_labels.rs` : `"reconciliation.unrejected"` dans `ACTIONS`, à sa place alphabétique
  (après `reconciliation.split_applied`, avant `reconciliation_rule.applied`).
- `crates/kesh-api/tests/textes_coherents.rs` (G9, `les_replis_rust_suivent_le_catalogue`, l. 679) : ligne
  `("reconciliation-errors-transaction-not-rejected", 1, &[])` au bloc `crates/kesh-api/src/errors.rs`.
- Doc-comments : en-tête de `routes/reconciliation.rs` ; `conclude_locked_attempt` (« Conclut une tentative de
  `post_manual` ou `post_split` » → les trois routes) ; « Seul appelant : `reject_batch` » de
  `find_by_id_for_account_for_update` et l'en-tête de `repositories/reconciliation.rs` ; la liste des appelants
  d'`is_reconciled` (`repositories/bank_imports.rs:346-355`) et son reflet dans `bank-import-deletion.ts:12-16`
  (« écrite deux fois, et seulement deux » reste vrai : `classify_unreject` **appelle** `is_reconciled`).
- Symptôme grepé après chaque patch (§ *Propagation post-patch*) : `Seul appelant`, `post_manual` ou `post_split`
  (doc-comments d'appelants), les valeurs `\b115\b`, `\b118\b`, `\b109\b`, `\b91\b`.

**AC12 (a) — Mutations.** M1-M8 font rougir au moins un test chacune ; M13 et M14 sont déclarées **équivalentes**
avec leur preuve, M15 **survivante assumée** avec sa raison (§ Tests, C-28-5-19, C-28-5-23), au Debug Log.

### Partie b — `28-5b-annuler-rejet-ecran` (PR `closes #526`)

**AC5 — Le détail d'un import montre le rejet et permet de l'annuler.** Dans
`frontend/src/routes/(app)/bank-import/[id]/+page.svelte`, colonne *Rapprochement*, pour toute transaction
**rejetée** — `isRejected(tx)` : `tx.autoMatchRejectedAt != null && !isReconciled(tx)` (`!=` : un champ **absent**,
`undefined`, n'est pas un rejet), `isReconciled` étant celui de `lib/features/bank-import/bank-import-deletion.ts:18`
(pas de seconde définition) — un composant neuf `lib/features/reconciliation/UnrejectTransactionAction.svelte` rend :
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
(`reconciliation.types.ts:140`, `:160`) **ne sont pas** étendus (aucun lecteur). Les fixtures qui construisent un
`BankTransactionDto` sont complétées (au sol : `bank-import-detail-page.test.ts:79`,
`bank-import-deletion.test.ts:9-23`, `DeleteBankImportAction.test.ts:23-37` ; `npm run check` signale les autres).

**AC6 — La page *Réconciliation* dit où retrouver une transaction rejetée.** `ReconciliationProposals.svelte` rend,
**en fin de `<section>`**, après le bloc des échecs partiels (`{#if failed.length > 0}`, l. 432-450) et hors de
toute branche (visible quand la liste est vide, chargée ou en erreur — un lot qui rejette la dernière ligne la vide),
un paragraphe discret `data-testid="reconciliation-rejected-hint"` : `reconciliation-labels-rejected-hint`. Son
texte est vrai pour **tout** rôle qui voit la page (C-28-5-22).

**AC7 (b) — Textes de l'interface.** Dans les quatre catalogues (textes au § Dev Notes « Textes ») :
`reconciliation-unreject-button`, `reconciliation-unreject-rejected-on` (variable `$date`),
`reconciliation-unreject-success`, `reconciliation-labels-rejected-hint` ; et **réécriture** de
`reconciliation-failed-already-rejected` (le refus d'un lot dit désormais où annuler le rejet, C-28-5-21), avec son
repli de `failed-proposal-label.ts:210-214` **égal mot pour mot** à la valeur fr-CH (apostrophe `’` comprise).
Chaque `i18nMsg` neuf a son repli français ; `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts:601` (2075
à `5aad8b35`) est **recompté** depuis la source après ajout, jamais incrémenté de confiance ;
`npm run lint-i18n-ownership` vert (les clés `reconciliation-*` ne sont employées que dans
`lib/features/reconciliation/`) ; les gardes de `frontend/src/lib/shared/i18n-*.test.ts` vertes.

**AC8 — Manuel utilisateur (`.tex` et PDF).** La puce *Rejeter* de `docs/manual/fr/user-manual.tex:1947` est
réécrite (texte au § Dev Notes) : la transaction quitte la liste, **reste** dans le détail de son import avec la
mention « Rejetée le … », et *Annuler le rejet* la rend de nouveau proposable — la liste montrant les 100 plus
récentes, une transaction plus ancienne n'y reparaît qu'une fois les plus récentes traitées (#649) ; rôle
Consultation : la mention, sans le bouton ; geste tracé, sans effet comptable. Les phrases « aucun geste de
l'interface ne la remet ensuite dans la liste », « sans aucune action » et « Ne rejetez donc une transaction que si
vous ne comptez pas la comptabiliser par ce chemin » **disparaissent** ; la parenthèse sur l'API et le renvoi à la
suppression d'import sont retirés ou reformulés pour rester vrais. PDF régénéré (`make fr` dans `docs/manual/`, sous
le verrou de gate) et versionné ; contrôle sur le PDF **aplati**, motifs **insensibles à l'apostrophe** (le PDF porte
`’`, U+2019) :

```sh
pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' > "$GATE_LOGS/28-5b-manuel-plat.txt"
grep -cE "Annuler le rejet" "$GATE_LOGS/28-5b-manuel-plat.txt"                                # ≥ 1
grep -cE "aucun geste de l.interface|sans aucune action" "$GATE_LOGS/28-5b-manuel-plat.txt"   # 0
```

(Au `5aad8b35`, `aucun geste de l.interface` et `sans aucune action` y figurent **une fois chacun**, `Annuler le
rejet` **zéro** fois — vérifié en validation P1 par `grep -oE` sur le PDF aplati : les deux contrôles rougissent bien
avant la correction.)

**AC9 (b) — CHANGELOG.** `CHANGELOG.md` : section `## [0.13.3] — Non publié` (créée si absente, par union —
précédent C-28-2-12) ; entrée *Ajouté* « Annuler le rejet d'une transaction bancaire, depuis le détail de son import
ou par l'API (`POST /api/v1/reconciliation/transactions/{id}/unreject`), et champ `autoMatchRejectedAt` (#526) » ;
entrée *Corrigé* « Une transaction bancaire rejetée se retrouve et redevient proposable (#526) » (C-28-5-24).

**AC11 — E2E Playwright, et recette visuelle.** Spec neuve `frontend/tests/e2e/reconciliation-unreject.spec.ts`
(patron de `reconciliation-deleted-import.spec.ts` : relevé CAMT **synthétique** réécrit à chaque exécution —
références, montants, contrepartie uniques —, import et utilisateur supprimés en `finally`) :
1. import ; rejet d'une transaction **par l'API** (`POST /reconciliation/reject`) ; détail de l'import : la ligne
   porte « Rejetée le » et *Annuler le rejet* ; clic → notification de succès
   (`page.locator('[data-sonner-toast][data-type="success"]')`, **sans** filtre de libellé — KF-043, #326), la
   mention disparaît (assertion décisive) ; page *Réconciliation* : la transaction est de nouveau listée
   (`data-tx-id`) ;
2. rôle Consultation (utilisateur créé par l'API, patron `open-items.spec.ts:251-259`) : la mention est visible,
   le bouton **absent** ;
3. la page *Réconciliation* montre l'indication (`reconciliation-rejected-hint`).
**Captures** (B2 de la rétrospective de l'Epic 27), thème clair **et** sombre (`page.emulateMedia({ colorScheme })`,
un test par schéma), dans `process.env.KESH_E2E_CAPTURES_DIR` s'il est défini, sinon `testInfo.outputPath()`
(convention de la 28-1, C-28-1-29 — reprendre son auxiliaire s'il est sur `main` au moment du développement) :
`detail-rejetee`, `detail-apres-annulation`, `reconciliation-indication`, × `clair|sombre` — **six** captures,
déposées hors dépôt dans `$GATE_LOGS/28-5b-captures/`, jamais versionnées, sans `toHaveScreenshot`. Données
synthétiques uniquement (§ 8 de `docs/agents-orchestration.md`).

**AC12 (b) — Mutations.** M9-M12 et M16 font rougir au moins un test chacune ; une survivante est soit tuée par un
test ajouté, soit déclarée **équivalente** avec sa preuve au Debug Log.

## Tasks / Subtasks

### 28-5a

- [ ] **T0a — Montage** : bases `kesh_285` / `kesh_e2e_285` (migrations du worktree, seed), port **3038**,
      `CARGO_TARGET_DIR=<worktree>/target`, `node_modules` par `cp -al` (§ 4 de `docs/agents-orchestration.md`).
      Rafraîchir la base avant tout gate complet.
- [ ] **T1 — Champ `autoMatchRejectedAt`** (AC1) : `TransactionResponse` + `From` ; test B7.
- [ ] **T2 — Route** (AC2, AC3, AC4, AC13) : variante `AppError::ReconciliationTransactionNotRejected { bank_transaction_id }`
      et son bras (`409`, code, message `t("reconciliation-errors-transaction-not-rejected", "<texte fr-CH exact>")`) ;
      dans `routes/reconciliation.rs`, section « Story 28-5 (#526) — annuler un rejet » : `post_unreject_transaction`,
      `unreject_once`, `enum UnrejectOutcome { Unrejected, NotFound, NotRejected }`, `classify_unreject` ;
      l'auxiliaire de lecture du compte et la signature générique de `conclude_locked_attempt` (C-28-5-18) ; route
      dans `lib.rs`.
- [ ] **T3 — Gardes et commentaires** (AC7 a, AC10) : registre des routes, `ACTIONS`, G9, deux clés serveur × quatre
      locales, doc-comments ; grep du symptôme.
- [ ] **T4 — Tests backend** B1-B9 (§ Tests) ; gate ciblé `binary(reconciliation_e2e)` + `audit_route_registry`
      + `audit_label_registry` + `textes_coherents` + `i18n_e2e` ; **`kesh-db` touché → gate complet** avant tout
      commit de fin de boucle (règle `CLAUDE.md`).
- [ ] **T5a — Documentation** (AC9 a) : `docs/api-external.md`.
- [ ] **T6a — Mutations** (AC12 a) M1-M8, M13-M15, sous verrou ; restauration `git checkout -- <f>` **puis `touch <f>`**.
- [ ] **T7a — Gate et comptes rendus** : gate complet **et** suite E2E complète au dernier commit de code (verrou de
      gate, recette de `docs/testing.md`, échecs jugés fichier par fichier) ; Dev Agent Record (dont la phrase
      « aucune release entre les deux PR »), File List, Change Log ; décomptes recomptés, périmètre déclaré.

### 28-5b

- [ ] **T0b — Montage** : idem T0a, sur `origin/main` après la fusion de la 28-5a.
- [ ] **T8 — Frontend** (AC5, AC6) : type, `unrejectTransaction`, `reconciliation-unreject.ts` (`isRejected`),
      `UnrejectTransactionAction.svelte`, page de détail, indication ; fixtures complétées.
- [ ] **T9 — i18n** (AC7 b) : quatre clés × quatre locales, `already-rejected` réécrite (quatre locales + repli) ;
      `sitesTotal` recompté.
- [ ] **T10 — Tests frontend** V1-V6 ; `npm run check`, `lint-i18n-ownership`, `test:unit`, `build`.
- [ ] **T11 — Documentation** (AC8, AC9 b) : manuel `.tex` + PDF (sous verrou), CHANGELOG.
- [ ] **T12 — E2E et captures** (AC11) : spec neuve ; **suite E2E complète** au dernier commit de code (verrou de
      gate, recette de `docs/testing.md` à la lettre), échecs jugés fichier par fichier contre
      `docs/testing.md` § *Les échecs attendus* ; six captures ; observer la page *Réconciliation* sous le rôle
      Consultation et le noter au Debug Log (sans corriger : exclu). Gate backend complet au même commit (catalogues).
- [ ] **T13 — Mutations** (AC12 b) M9-M12, M16 ; restauration `git checkout -- <f>` puis `touch <f>`.
- [ ] **T14 — Comptes rendus** : comme T7a.

## Tests

**Backend (28-5a) — section neuve en fin de `crates/kesh-api/tests/reconciliation_e2e.rs`** (montages du fichier :
`setup_company`, `seed_bank_transactions`, `make_new_tx`, `unique_hash`, `create_user`, `forge_jwt`,
`setup_accept_split_ctx` l. 5382, patron des clés de `an_api_key_cancels_and_is_named_in_the_audit` l. 4511, patron
du verrou tenu de `post_accept_returns_409_on_account_lock_contention` l. 1435 ; un auxiliaire
`unreject(app, jwt, tx_id) -> (u16, Value)` sur le modèle de `cancel_reco` l. 3578) :

| # | Test | Établit |
|---|---|---|
| B1 | `unreject_puts_a_rejected_transaction_back_in_the_proposals` | rejet par la route ; `204` ; marqueur `NULL`, `version` +1, `status`/`matched_entry_id` inchangés ; la transaction est dans `GET /proposals` (absente avant) ; **une** ligne `reconciliation.unrejected` dont `previouslyRejectedAt` est **la même chaîne** que l'`autoMatchRejectedAt` lu par `GET /bank-imports/{id}` avant l'annulation ; puis **cycle** (AC13) : `POST /reject` du même id → `rejected[]` le contient, `failed[]` vide, marqueur reposé ; `…/unreject` → `204` |
| B2 | `unreject_refuses_a_transaction_that_is_not_rejected` | jamais rejetée → `409` code ; un second appel après succès → `409` ; ni `version` ni `audit_log` ne bougent |
| B3 | `unreject_refuses_a_reconciled_transaction_even_if_marked` | état **forgé, et dit tel** (`status='reconciled'`, `matched_entry_id`, marqueur posé — patron du test l. 4438) → `409` ; marqueur intact |
| B4 | `unreject_roles_and_scope` | Consultation → `403` ; autre société → `404` et rien d'écrit (comptes de lignes avant/après, patron l. 4385) |
| B5 | `unreject_returns_409_when_the_account_is_locked` | verrou nommé du compte tenu par une autre connexion → `409 RECONCILIATION_ACCOUNT_LOCKED`, marqueur intact |
| B6 | `an_api_key_unrejects_and_is_named_in_the_audit` | clé `read-write` → `204` ; `actor_type = 'api_key'`, `actor_api_key_id` = la clé |
| B7 | `the_import_detail_exposes_the_rejection_date` | `GET /bank-imports/{id}` : `autoMatchRejectedAt` chaîne ISO terminée par `Z` pour la rejetée, `null` pour l'autre ; `null` après l'annulation |
| B8 | `mod unreject_classification_tests` (unitaire, `routes/reconciliation.rs`) | `classify_unreject` : `None` → `NotFound` ; rapprochée marquée → `NotRejected` ; non marquée → `NotRejected` ; rejetée → `Unrejected` |
| B9 | `a_transaction_whose_rejection_is_undone_can_be_accepted_again` | AC13 : proposition `split` (`setup_accept_split_ctx`) ; rejet ; `POST /accept` → `failed[]` en `RECONCILIATION_ALREADY_REJECTED` ; `…/unreject` → `204` ; `POST /accept` de nouveau → acceptée, `status = 'reconciled'`, marqueur `NULL` |

**Frontend (28-5b) — Vitest :**

| # | Fichier | Établit |
|---|---|---|
| V1 | `lib/features/reconciliation/reconciliation-unreject.test.ts` | `isRejected` : rejetée `pending` sans lien → vrai ; rapprochée marquée → faux ; non marquée (`null`) → faux ; champ **absent** (`undefined`) → faux |
| V2 | `lib/features/reconciliation/UnrejectTransactionAction.test.ts` | `canManage=false` : mention, aucun bouton ; `canManage=true` : bouton ; clic → **un** appel, bouton désactivé pendant ; succès → `notifySuccess` + `onUnrejected` ; échec → `notifyError(message du serveur)` + `onUnrejected` |
| V3 | `routes/(app)/bank-import/[id]/bank-import-detail-page.test.ts` | ligne rejetée : mention présente ; ligne non rejetée et ligne rapprochée : absente |
| V4 | `ReconciliationProposals.test.ts` | l'indication est rendue liste vide **et** liste non vide, **après** le bloc des échecs partiels quand il est présent |
| V5 | `reconciliation.api.test.ts` | `unrejectTransaction(7)` poste `{}` à `/api/v1/reconciliation/transactions/7/unreject` |
| V6 | `failed-proposal-label.test.ts` | `RECONCILIATION_ALREADY_REJECTED` rend la clé réécrite, et son repli égale la valeur fr-CH |

**Mutations** — motif vérifié unique avant écriture :

| # | Partie | Mutation | Tuée par |
|---|---|---|---|
| M1 | a | `classify_unreject` : retirer le bras `is_reconciled` | B3, B8 |
| M2 | a | `classify_unreject` : retirer le bras « non marquée » | B2, B8 |
| M3 | a | `UPDATE` : retirer `version = version + 1` | B1 |
| M4 | a | supprimer l'insertion d'audit | B1 |
| M5 | a | `for_actor(user_id, None, …)` au lieu de `actor_api_key_id` | B6 |
| M6 | a | appeler `unreject_once` sans `with_account_lock` | B5 |
| M7 | a | monter la route dans `authenticated_routes` | B4 |
| M8 | a | `From` : `auto_match_rejected_at: None` | B7 |
| M13 | a | garde `rows_affected() != 1` retirée | **équivalente** : la ligne est lue `FOR UPDATE` sous verrou, scopée par sa clé primaire ; l'`UPDATE` ne peut toucher qu'elle (C-28-5-19) |
| M14 | a | `UPDATE` scopé par le seul `id` (sans `company_id` ni `bank_account_id`) | **équivalente** : `id` est la clé primaire, et la ligne a été lue scopée société et compte sous verrou — la portée est déjà établie ; les deux prédicats restent comme défense en profondeur (patron du dépôt) |
| M15 | a | lecture **non** verrouillante (`find_pending_by_id_for_account`, l. 351) au lieu de `find_by_id_for_account_for_update` | **survivante assumée** : B1-B9 sont séquentiels ; la différence ne se voit que dans l'intervalle entre la relâche du verrou nommé et le `commit` d'un geste concurrent, que rien ne rend déterministe — même angle mort que celui écrit au doc-comment de la lecture (`repositories/reconciliation.rs:385-400`) (C-28-5-23) |
| M9 | b | `isRejected` sans `!isReconciled(tx)` | V1 |
| M10 | b | bouton rendu sans condition `canManage` | V2 |
| M11 | b | `onUnrejected` non appelé sur échec | V2 |
| M12 | b | indication déplacée dans la branche « liste non vide » | V4 |
| M16 | b | `isRejected` en `!== null` au lieu de `!= null` | V1 (cas `undefined`) |

## Dev Notes

### Séquence de la route (C-28-5-4, C-28-5-5, C-28-5-18, C-28-5-19)

1. Hors verrou : `bank_account_id` de la transaction, lu **scopé société** par un auxiliaire privé
   `bank_account_of_transaction(pool, company_id, id) -> Result<i64, AppError>` qui reçoit **à l'identique** les
   lignes 4514-4524 de `cancel_reconciliation_once` (même SQL, mêmes conversions d'erreur) ; `cancel` l'appelle à
   leur place. Absente → `404`. *Lecture non verrouillante en autocommit : elle ne demande aucun verrou, ne peut être
   victime d'un interblocage, et le rejeu de `…/cancel` n'est pas en jeu.*
2. `pool.begin()`, `with_account_lock(&mut tx, company_id, bank_account_id, LOCK_TIMEOUT_SECS, …)`.
3. Dans la fermeture : `find_by_id_for_account_for_update(&mut **tx_inner, company_id, bank_account_id, id).await?`
   (lecture **courante** et verrouillante — une `SELECT` ordinaire lirait la vue cohérente, cf. son doc-comment) →
   `classify_unreject`. `NotFound` ou `NotRejected` : `Ok(outcome)` sans rien écrire.
4. `Unrejected` : `UPDATE bank_transactions SET auto_match_rejected_at = NULL, updated_at = NOW(3), version = version + 1
   WHERE id = ? AND company_id = ? AND bank_account_id = ?`, erreur rangée par
   `.map_err(|e| ReconciliationError::Db(DbError::Sqlx(e)))?` (patron de `post_manual_once`, `:3800-3820`) ;
   `rows_affected() != 1` → `Err(ReconciliationError::Db(DbError::OptimisticLockConflict))` (`:3822-3826`).
5. Audit : `audit_log::insert_in_tx(tx_inner, NewAuditLogEntry::for_actor(…)).await?` (patron `:3840-3851` : le `?`
   range l'erreur en `ReconciliationError::Db`, `DbError::Sqlx` préservé) ; `details` construit avec le marqueur lu
   au point 3, `.and_utc()`.
6. Après la fermeture : `conclude_locked_attempt(tx, lock_result, "post_unreject").await?` — l'auxiliaire existant
   de `post_manual` et `post_split` (`:4312-4395`), dont seule la **signature** devient générique
   (`lock_result: Result<T, ReconciliationError>` → `Result<T, AppError>`), corps et `match` **inchangés** : il
   `commit` si la fermeture a réussi (pour un refus, rien n'a été écrit : le `commit` vaut un `rollback`), sinon
   `rollback` et conversion qui préserve `DbError::Sqlx`. Puis : `Unrejected` → `204` ; `NotFound` →
   `AppError::Database(DbError::NotFound)` ; `NotRejected` → `AppError::ReconciliationTransactionNotRejected`.
   **Aucune troisième copie** du `match` de conversion (C-28-5-18) ; celui de `cancel_reconciliation_once` reste tel
   quel.

**Pas de rejeu sur interblocage** (`SansEcritureAuJournal`, comme `post_reject`) : la transaction ne verrouille
qu'**une** ligne de `bank_transactions` puis insère une ligne d'audit (`audit_log` n'a aucune clé étrangère) ; la
suppression d'un import (27-1) verrouille les lignes de l'import par identifiant croissant **sans** prendre le verrou
nommé ; la création d'import n'écrit que des lignes neuves ; la restauration d'installation écarte `audit_log` de son
`DELETE` — aucun cycle n'est possible (C-28-5-4). Le verrou nommé est relâché avant le `commit`
(`kesh-reconciliation/src/mutex.rs`, propriété commune à tous les gestes) : un geste concurrent qui obtient le verrou
dans cet intervalle bute sur la lecture `FOR UPDATE` et attend le `commit` ; un lot qui aurait classé la ligne avant
peut la refuser (`RECONCILIATION_ALREADY_REJECTED`) — refus bénin, à relancer, déjà le cas entre tous les gestes.

### Inventaire des sites NON RÉSOLUS (D4-ter)

Classe de défaut : *une transaction rejetée qu'aucun écran ne montre ou ne permet de reprendre.* Ensemble clos,
obtenu par `grep -rn "auto_match_rejected_at" crates --include=*.rs --include=*.sql | grep -v /tests/` et
`grep -rn "autoMatchRejectedAt\|auto_match_rejected" frontend/src` (vide à `5aad8b35`) ; élargi en validation P1
(`rejectedAt|wasRejected|was_previously|rejected_at` sur `frontend/src docs crates/kesh-seed` : seul
`reconciliation.types.ts:104`, `rejectedAt` de `RejectedProposal`, autre objet ; `kesh-seed`, `test_endpoints.rs`,
`post_restore.rs` : aucune occurrence). **Chaque ligne du grep** figure ci-dessous :

| Site | Rôle vis-à-vis du marqueur | Après la story |
|---|---|---|
| `repositories/reconciliation.rs:182` (`find_pending_transactions_for_account`) | exclut les rejetées des propositions | **voulu** (c'est le sens du rejet) ; l'indication de l'AC6 renvoie au détail |
| `routes/reconciliation.rs:885` (`classify_batch_transaction`) | refuse une rejetée à l'acceptation et au rejet | voulu ; annuler le rejet la rend de nouveau acceptable (AC13, B9) ; son libellé dit où annuler (AC7 b) |
| `routes/reconciliation.rs:3357` (`reject_batch`) | **pose** le marqueur | inchangé |
| `:2540`, `:2992`, `:3803`, `:4260`, `reconciliation_cancel.rs:340` | **remettent** le marqueur à `NULL` | inchangés ; aucun n'est atteignable depuis l'écran pour une rejetée (cf. *Le défaut*) |
| **route neuve** `…/unreject` | remet à `NULL` | **le chemin de l'interface** |
| `routes/reconciliation.rs:2565`, `:3033`, `:3750`, `:4224` (`was_previously_rejected`) ; `reconciliation_cancel.rs:400` (`wasPreviouslyRejected`) | lisent le marqueur pour l'**audit** des gestes qui le remettent à `NULL` et de l'annulation | inchangés, sans effet sur la classe ; une transaction dont le rejet a été annulé y lira `false` — exact, puisqu'elle n'est plus rejetée ; l'audit d'`unreject` porte `previouslyRejectedAt` (AC2) |
| `repositories/bank_imports.rs:37` (`COLUMNS`), `bank_transactions.rs:24`, `repositories/reconciliation.rs:82` (`BANK_TX_COLUMNS`) | listes de colonnes lues | inchangées, sans effet sur la classe |
| `repositories/bank_imports.rs:330`, `:373`, `:~480` (`rejected_count` de la suppression d'import, 27-1) | comptent les rejetées supprimées avec leur import | inchangés ; vrais (une rejetée reste supprimable avec son import) |
| `entities/bank_transaction.rs:105` ; `migrations/20260507100001_reconciliation_8_4.sql` ; `test-schema/0001_schema_squash.sql` | définition de la colonne | inchangés |
| `routes/bank_imports.rs:250` (`TransactionResponse`) | n'exposait pas le marqueur | **résolu** (AC1) |
| `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` | seul écran qui liste une rejetée | **résolu** (AC5) |
| `exports/csv_tables.rs:740` | exporte la colonne | déjà résolu (export global) |
| page *Imports précédents* (`BankImportList.svelte`) | ne compte pas les rejetées | **angle mort assumé** : l'utilisateur ouvre l'import d'après la date de la transaction ; un compteur serait du périmètre en plus |
| colonne *Statut* du détail (`pending` brut) | ne dit pas « rejetée » | **angle mort assumé**, la mention de l'AC5 le dit à côté ; traduction suivie par #339 |
| liste de la page *Réconciliation* au-delà de 100 lignes | une rejetée annulée, ancienne, n'y reparaît pas | **angle mort assumé, suivi par #649** ; les textes le disent (C-28-5-20) |

### Textes (quatre locales)

Partie **a** :

| Clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|
| `reconciliation-errors-transaction-not-rejected` | Cette transaction n'est pas rejetée : son rejet a peut-être déjà été annulé, ou elle a été rapprochée. | Diese Transaktion ist nicht abgelehnt: Die Ablehnung wurde möglicherweise bereits aufgehoben, oder sie wurde abgestimmt. | Questa transazione non è rifiutata: il rifiuto potrebbe essere già stato annullato, oppure è stata riconciliata. | This transaction is not rejected: its rejection may already have been undone, or it has been reconciled. |
| `audit-log-action-reconciliation-unrejected` | Rejet de réconciliation annulé | Abstimmungsablehnung aufgehoben | Rifiuto della riconciliazione annullato | Reconciliation rejection undone |

Partie **b** :

| Clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|
| `reconciliation-unreject-button` | Annuler le rejet | Ablehnung aufheben | Annulla il rifiuto | Undo rejection |
| `reconciliation-unreject-rejected-on` | Rejetée le { $date } | Abgelehnt am { $date } | Rifiutata il { $date } | Rejected on { $date } |
| `reconciliation-unreject-success` | Rejet annulé : la transaction est de nouveau proposée à la réconciliation, dont la liste montre les 100 plus récentes en attente. | Ablehnung aufgehoben: Die Transaktion wird wieder zur Abstimmung vorgeschlagen; die Liste zeigt die 100 neuesten offenen Transaktionen. | Rifiuto annullato: la transazione è di nuovo proposta per la riconciliazione, il cui elenco mostra le 100 più recenti in sospeso. | Rejection undone: the transaction is offered for reconciliation again; the list shows the 100 most recent pending ones. |
| `reconciliation-labels-rejected-hint` | Une transaction rejetée quitte cette liste. Elle reste dans le détail de son import (Importer des relevés), où son rejet peut être annulé si votre rôle le permet. | Eine abgelehnte Transaktion verlässt diese Liste. Sie bleibt im Detail ihres Imports (Kontoauszüge importieren), wo ihre Ablehnung aufgehoben werden kann, sofern Ihre Rolle es erlaubt. | Una transazione rifiutata lascia questo elenco. Resta nel dettaglio della sua importazione (Importa estratti conto), dove il rifiuto si può annullare se il suo ruolo lo consente. | A rejected transaction leaves this list. It remains in the detail of its import (Import statements), where its rejection can be undone if your role allows it. |
| `reconciliation-failed-already-rejected` (**réécrite**) | Cette transaction a déjà été rejetée : elle n’est plus proposée. Son rejet s’annule depuis le détail de son import (Importer des relevés). | Diese Transaktion wurde bereits abgelehnt: Sie wird nicht mehr vorgeschlagen. Die Ablehnung lässt sich im Detail ihres Imports aufheben (Kontoauszüge importieren). | Questa transazione è già stata rifiutata: non viene più proposta. Il rifiuto si annulla dal dettaglio della sua importazione (Importa estratti conto). | This transaction has already been rejected: it is no longer proposed. Its rejection can be undone from the detail of its import (Import statements). |

Les libellés de menu cités entre parenthèses sont ceux de `nav-bank-import` (`+layout.svelte:94`), relevés à la
fiche dans les quatre catalogues : « Importer des relevés », « Kontoauszüge importieren », « Importa estratti conto »,
« Import statements ». S'ils changent avant le développement, les textes s'alignent sur eux. ⚠️ Le nombre **100** de
`reconciliation-unreject-success` est celui de `DEFAULT_PROPOSALS_LIMIT` et de `reconciliation.api.ts:18` : la
résolution de #649 le réécrit (cf. *Exclu*). Apostrophes : reprendre la graphie du catalogue à la ligne voisine
(fr-CH : `’` dans la ligne réécrite, comme aujourd'hui).

**Manuel — puce *Rejeter* réécrite (`user-manual.tex:1947`, partie b)** :

> \emph{Rejeter} écarte la proposition~: la transaction est marquée comme revue et \textbf{quitte la liste} de la
> réconciliation. Elle reste dans le \textbf{détail de son import} (\emph{Quotidien} → \emph{Importer des relevés},
> liste \emph{Imports précédents}, puis le nom du fichier), avec le statut «~pending~» et la mention «~Rejetée
> le~\dots~». Le bouton \emph{Annuler le rejet} de sa ligne la rend de nouveau \textbf{proposable}~: elle revient
> dans la liste de la réconciliation --- qui ne montre que les 100 plus récentes en attente~: une transaction plus
> ancienne n'y reparaît qu'une fois les plus récentes traitées ---, où elle peut de nouveau être acceptée, affectée
> manuellement ou éclatée~; le geste ne passe aucune écriture et s'inscrit au journal d'audit (\emph{Rejet de
> réconciliation annulé}). Le Comptable et l'Administrateur voient le bouton~; le rôle Consultation voit la mention
> seule. La page \emph{Réconciliation} le rappelle sous la liste.

Vérifier aussi, au même passage, que § *Supprimer un import* (l. 1739, 1745 : « une transaction rejetée ne compte
pas », « rejetées comprises ») reste vrai — il l'est, la story ne change pas la suppression — et que le paragraphe
*Un refus n'empêche pas les autres* (« rapprochée ou rejetée entre-temps ») reste vrai — il l'est.

### Ce qui doit rester intact

- `POST /reject` et `POST /accept` (27-4) : codes, ordre, `failed[]` — aucune ligne de leurs fonctions ne change.
- `…/cancel` : seule sa lecture du compte se déplace dans l'auxiliaire, **mot pour mot** ; son `match` de
  conversion et tout le reste sont intouchés ; ses tests (l. 3451-4560 de `reconciliation_e2e.rs`) restent verts
  sans modification.
- `post_manual`, `post_split` : `conclude_locked_attempt` ne change que de signature (générique) ; son corps et le
  `DbError::Sqlx` qu'il préserve pour leur rejeu sont intouchés.
- `TransactionResponse` : champs existants inchangés ; le détail d'import de la 27-1 (`DeleteBankImportAction`,
  `reconciledCount`) ne lit pas le champ neuf.
- Aucune migration : `kesh_version_min_required` inchangé, aucune ligne au registre d'idempotence, aucun backfill.

### References

- Issue #526 et ses deux commentaires (lentilles R et F de la 15-5c) ; epic § 28-5 ; #649 (limite de 100).
- `crates/kesh-api/src/routes/reconciliation.rs` : `reject_batch` `:3338-3462`, `classify_batch_transaction`
  `:872-897`, `post_manual_once` `:3699-3866`, `conclude_locked_attempt` `:4312-4395`, annulation `:4400-4586`.
- `crates/kesh-db/src/repositories/reconciliation.rs:330-416` ; `repositories/bank_imports.rs:340-358`.
- `docs/api-external.md:534-575` ; `docs/manual/fr/user-manual.tex:1933-1952`.
- Fiche de la 28-4 (`story/28-4`, `7a4adf64`) : W30, R8, M33 pour la route `unreject`.
- Consignes : `docs/agents-orchestration.md` ; `CLAUDE.md` § *Review Iteration Rule*, § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-13 — Fiche créée en autonomie (bmad-create-story, Opus 5.5) : 12 AC, T0-T11, B1-B8, V1-V5, M1-M12, choix
  C-28-5-1 à C-28-5-16 ; validation P1 à lancer (`28-5-validate-prompt-p1.md`, Sonnet, lentilles R et F).
- 2026-10-13 — **Validation P1** (Sonnet 5.5, lentilles R et F, rapports `$GATE_LOGS/28-5-validate-p1-{R,F}.md`) :
  R — 0 HIGH, 3 MEDIUM, 6 LOW ; F — 0 HIGH, 3 MEDIUM, 8 LOW ; soit 6 MEDIUM, dont F1 et R2 sur le même objet
  (**5 distincts**), et 14 LOW, dont quatre recoupements (F4 = R6, F6 = R8, F8 ⊂ R9, F10 = R5). Remédiation
  (Opus 5.5, décisions de l'orchestrateur) : **F3** — découpage en **28-5a** (serveur, `refs #526`) / **28-5b**
  (écran, `closes #526`), C-28-5-17 remplace la dérogation C-28-5-11 ; **F1 + R2** — aucune extraction du
  convertisseur de `…/cancel` : la route suit `conclude_locked_attempt` (signature générique, corps intact), seule la
  lecture du compte est partagée, mot pour mot (C-28-5-18, remplace C-28-5-12) ; l'`AppError::Internal`
  inatteignable de l'AC3 devient `ReconciliationError::Db(OptimisticLockConflict)` (C-28-5-19) ; **F2** — limite de
  100 exclue avec renvoi à **#649**, textes du toast et du manuel nuancés (C-28-5-20) ; **R1** — AC13 neuf (rejeter →
  annuler → rejeter / accepter), B1 étendu, B9 neuf, libellé `already-rejected` réécrit dans les quatre locales
  (C-28-5-21) ; **R3** — inventaire complété de chaque ligne du grep. LOW : repli Rust inscrit à G9 (F4/R6) ;
  `!= null` + V1 `undefined` + M16 (F5) ; M13/M14 équivalentes, M15 survivante assumée (F6/R8, C-28-5-23) ;
  `.and_utc()` pour l'audit et B1 en égalité de chaîne (F7) ; l. 991 (F8/R9) ; tableau des régions étendu aux
  fichiers partagés (F9) ; indication en fin de section, texte vrai pour tout rôle (F10/R5, C-28-5-22) ; route déjà
  à l'inventaire de la 28-4 (F11) ; motif PDF insensible à l'apostrophe (R4) ; écart « filtre » à porter à la PR
  (R7) ; `$GATE_LOGS`, CHANGELOG en *Ajouté* + *Corrigé* (C-28-5-24), sélecteur du toast (R9). Recompte après
  remédiation : **13 AC** (AC1-AC13 ; AC7, AC9 et AC12 en deux volets), tâches T0a-T7a et T0b-T14, **B1-B9**,
  **V1-V6**, **M1-M16**, choix C-28-5-1 à C-28-5-24 ; **six** clés i18n neuves (deux en a, quatre en b) et une
  réécrite, × quatre locales. Validation P2 à lancer (`28-5-validate-prompt-p2.md`, Opus, R et F).
