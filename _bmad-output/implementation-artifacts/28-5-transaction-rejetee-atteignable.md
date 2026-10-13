# Story 28.5 : Une transaction rejetée redevient atteignable

## Status

**28-5a : review** (2026-10-13, `bmad-dev-story`, Opus 5.5 ; dernier commit de code `822ba587` ; revue de code P1 à
lancer, `28-5a-review-prompt-p1.md`, Sonnet, lentilles B, E et A). **28-5b : ready-for-dev** (part de `origin/main`
après la fusion de la 28-5a).

*Historique :* **ready-for-dev** (deux clés) — fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur
`5aad8b35`, branche `story/28-5` ; validation **P1** (Sonnet, lentilles R et F : 0 HIGH, 6 MEDIUM, 14 LOW)
**remédiée** le 2026-10-13, avec **découpage** décidé par l'orchestrateur (C-28-5-17, qui remplace la dérogation
C-28-5-11) ; validation **P2** (Opus, lentilles R et F : 0 HIGH, 4 MEDIUM soit 2 distincts, 18 LOW) **remédiée** le
2026-10-13 (C-28-5-25 à C-28-5-30) ; passe **ciblée P3** à lancer (`28-5-validate-prompt-p3-ciblee.md`, Haiku, une
lentille sur la seule remédiation P2).

<!-- Issue : #526 (bug, P2). Epic : planning-artifacts/epic-28-recette-v0-13-2.md § 28-5.
     Choix : C-28-5-1 à C-28-5-30 au registre epic-28-choix-autonomes.md (C-28-5-11, C-28-5-12 et C-28-5-23
     remplacés). -->

⛔ **Deux stories, une fiche, deux PR successives** (C-28-5-17, sur les précédents C-26-2-15 et C-28-4-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`28-5a-annuler-rejet-serveur`** | route `…/unreject`, champ `autoMatchRejectedAt`, trace d'audit, messages serveur (deux clés, quatre locales), gardes et doc-comments du serveur, tests backend, `docs/api-external.md` — **aucun fichier frontend** (C-28-5-26) | AC1 à AC4, AC7 (a), AC9 (a), AC10, AC12 (a), AC13 | **`refs #526`** — ⛔ jamais `closes` |
| **`28-5b-annuler-rejet-ecran`** | détail d'import (mention, bouton), indication de la page *Réconciliation*, libellé `already-rejected` réécrit, i18n de l'interface, doc-comment de `bank-import-deletion.ts`, manuel `.tex` + PDF, CHANGELOG, E2E, captures | AC5, AC6, AC7 (b), AC8, AC9 (b), AC11, AC12 (b) | **`closes #526`** |

**Ordre** : la 28-5a se développe et se fusionne **d'abord** ; la 28-5b part de `origin/main` **après** cette fusion.
La clé mère `28-5-transaction-rejetee-atteignable` passe au statut `split`. **Aucune release entre les deux PR** :
entre elles, l'annulation n'existe que par l'API, sans écran, sans manuel ni CHANGELOG — à écrire au Dev Agent Record
de la 28-5a. **Ordre de l'epic, tranché par l'orchestrateur** (C-28-5-29) : **28-5a, puis 28-6 ∥ 28-5b, puis
28-7**. La 28-5b ne touche pas `routes/reconciliation.rs` : la 28-6 se rebase sur la 28-5a pour ce fichier (et pour
`docs/api-external.md`, que la 28-5b ne touche pas) ; elle partage avec la 28-5b `failed-proposal-label.ts`, son
test `failed-proposal-label.test.ts`, `sitesTotal` de `i18n-keys.test.ts` et la section `reconciliation-*` des
catalogues (cf. § *Régions touchées*) — **union au rebase**, `sitesTotal` recompté par la seconde fusionnée.

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
(`lib.rs`, `errors.rs`, `audit_labels.rs`), `kesh-db` (doc), `kesh-i18n` (deux clés serveur) — **cinq**, au seuil
et non au-delà : le doc-comment de `frontend/.../bank-import-deletion.ts`, qui en faisait un sixième, passe à la
28-5b (C-28-5-26) ; **28-5b** : `frontend/features/reconciliation`, `frontend/features/bank-import` (un champ de
type et ce doc-comment), `frontend/routes/(app)/bank-import/[id]`, `kesh-i18n` (clés d'interface). Chaque partie se
teste seule : la 28-5a par ses tests Rust et l'API, la 28-5b sur la route fusionnée. Entre les deux PR, le
doc-comment frontend est **incomplet** (il ne nomme pas `classify_unreject` parmi les appelants serveur) sans être
faux.

### Régions touchées — à l'attention des 28-4, 28-6 et 28-7

| Fichier | Partie | Région | Nature |
|---|---|---|---|
| `crates/kesh-api/src/routes/reconciliation.rs` | a | en-tête du module (l. 1-25, une puce) ; **section neuve entre la fin de `cancel_reconciliation_once` (l. 4586) et `#[cfg(test)] mod period_lock_tests` (l. 4588)** ; dans `cancel_reconciliation_once`, la **seule** lecture du compte (l. 4514-4524) remplacée par l'appel à l'auxiliaire qui la reçoit à l'identique (C-28-5-18) — son `match` de conversion (l. 4558-4583) **intouché** ; `conclude_locked_attempt` (doc-comment l. 4312, fonction l. 4325-4399) : **signature** rendue générique, doc-comment complété, liaison `Ok(journal_entry_id)` renommée `Ok(value)` et les deux commentaires internes qui nomment ses appelants (l. 4366, l. 4384) complétés — logique et `match` intouchés (C-28-5-28) ; un `mod unreject_classification_tests` en fin de fichier | `get_proposals`, `accept_*`, `post_reject`, `reject_batch`, `classify_batch_transaction` **intouchés** |
| `crates/kesh-api/src/lib.rs` | a | `comptable_routes`, après la route `…/transactions/{id}/cancel` (l. 676-679) | une route |
| `crates/kesh-api/src/errors.rs` | a | variante neuve à côté de `ReconciliationTransactionNotPending` (l. 921-930) et son bras (l. 2663-2677) | |
| `crates/kesh-api/src/audit_labels.rs` | a | `ACTIONS` (l. 155-160) | une ligne |
| `crates/kesh-api/src/routes/bank_imports.rs` | a | `TransactionResponse` et son `From` (l. 250-288) | un champ |
| `crates/kesh-api/tests/reconciliation_e2e.rs` | a | **section neuve en fin de fichier** (après l. 7526) | tests B1-B7, B9, B10 |
| `crates/kesh-api/tests/audit_route_registry.rs` | a | une ligne de registre ; compteurs l. 645, 648, 667, 991, 997 ; doc l. 27, 31-32 ; point (iv) du doc du module (l. 88-115, après la phrase sur `post_reject`) : l'angle mort d'interblocage de la route neuve (C-28-5-30) | |
| `crates/kesh-api/tests/textes_coherents.rs` | a | `TABLE` de G9, bloc `errors.rs` (l. 685-735) | une ligne |
| `crates/kesh-db/src/repositories/reconciliation.rs` | a | doc-comments : en-tête (l. 23-25), `find_by_id_for_account_for_update` (l. 393, « Seul appelant ») | la 28-7 touche `find_unpaid_invoices_for_window` (l. 90-150) : régions disjointes |
| `crates/kesh-db/src/repositories/bank_imports.rs` | a | doc-comment d’`is_reconciled` (l. 338-355 : l'inventaire « clos » des écrivains, l. 338-344, et la liste des appelants, l. 350-355) | |
| `frontend/src/lib/features/bank-import/bank-import-deletion.ts` | **b** | doc-comment l. 12-16 (appelants serveur d'`is_reconciled`) | commentaire seulement (C-28-5-26) |
| `crates/kesh-i18n/locales/*/messages.ftl` | a, b | section `reconciliation-*` : a — `reconciliation-errors-transaction-not-rejected`, `audit-log-action-reconciliation-unrejected` ; b — quatre clés d'interface et `reconciliation-failed-already-rejected` **réécrite** (fr-CH l. 1219, de/it/en l. 1156) | la 28-6 y ajoute un libellé : union au rebase |
| `docs/api-external.md` | a | § *Accepter des propositions*, paragraphe **« Transaction disparue, rapprochée ou rejetée »** (l. 540, une phrase de renvoi — C-28-5-27) ; sous-section neuve après « Annuler un rapprochement bancaire » (l. 554 et suiv.) ; ligne `GET /api/v1/bank-imports/{id}` (l. 574) | la 28-6 ajoute un refus au § l. 536-552 : contigu, union |
| `frontend/src/lib/features/reconciliation/failed-proposal-label.ts` | b | repli de `RECONCILIATION_ALREADY_REJECTED` (l. 210-214) | la 28-6 y ajoute un code : union |
| `frontend/src/lib/features/reconciliation/failed-proposal-label.test.ts` | b | V6 | la 28-6 y teste son code : union |
| `frontend/src/lib/shared/i18n-keys.test.ts` | b | `sitesTotal` (l. 601) | recompté, jamais incrémenté ; la 28-6 le modifie aussi si elle ajoute un `i18nMsg` : **recompté par la seconde fusionnée** |
| `frontend/src/lib/features/reconciliation/UnrejectTransactionAction.svelte` (neuf) | b | import d'`errorMessageOf` | la 28-2 le déplace d'`api-client.ts:38` vers `error-message.ts` : importer depuis le chemin en vigueur sur `origin/main` au départ de la 28-5b |
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
| transaction inexistante, ou d'une autre société (lecture **scopée société**, indiscernables) — hors verrou, ou disparue sous verrou (import supprimé entre-temps : B10) | `404 NOT_FOUND` (`AppError::Database(DbError::NotFound)`, comme `…/cancel`) |
| transaction non rejetée (`auto_match_rejected_at` nul) — dont un second appel après un succès | `409 RECONCILIATION_TRANSACTION_NOT_REJECTED`, `details: { bankTransactionId }` |
| transaction **rapprochée** au sens d'`is_reconciled` (`kesh_db::repositories::bank_imports::is_reconciled`), **même si** son marqueur est posé (état forgé, aucun chemin réel ne le produit) | `409 RECONCILIATION_TRANSACTION_NOT_REJECTED` ; marqueur **intact** |
| verrou du compte tenu au-delà de `LOCK_TIMEOUT_SECS` | `409 RECONCILIATION_ACCOUNT_LOCKED` (inchangé) |

Le classement se fait **sous verrou, sur une lecture verrouillante** (`find_by_id_for_account_for_update`), par une
fonction pure `classify_unreject(row: Option<&BankTransaction>) -> UnrejectOutcome` qui appelle `is_reconciled` —
**aucun second test de statut** (C-28-5-5). L'`UPDATE` qui suit est scopé `id`, `company_id`, `bank_account_id` ;
s'il ne touche pas exactement une ligne, la fermeture rend
`Err(ReconciliationError::Db(DbError::OptimisticLockConflict))` — patron de `post_manual_once` (`:3822-3826`) ;
`conclude_locked_attempt` la rend en `AppError::Database(DbError::OptimisticLockConflict)`, et c'est le **mapping
des erreurs de base** (`errors.rs:2922-2925`) qui en fait un `409 OPTIMISTIC_LOCK_CONFLICT` ; jamais un `204` muet.
Ce bras est **inatteignable** tant que la lecture est verrouillante (la ligne est verrouillée) : sa mutation M13 est
déclarée équivalente (C-28-5-19). C'est aussi ce `409` que rendrait une lecture **non** verrouillante sur une
transaction supprimée pendant l'attente — d'où B10, qui exige le `404` (C-28-5-25).

**AC4 — Rôles et clés.** Consultation → `403` (RBAC de `comptable_routes`) ; Comptable (B1) et Administrateur (B4)
→ `204` ;
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
le `409 OPTIMISTIC_LOCK_CONFLICT` défensif, dit inatteignable), clés `read-write` admises et nommées à l'audit, champ
`autoMatchRejectedAt` *(depuis la v0.13.3)*, rappel de la limite de `GET …/proposals` ; une phrase au paragraphe
**« Transaction disparue, rapprochée ou rejetée »** (l. 540, là où est décrit le refus
`RECONCILIATION_ALREADY_REJECTED`) qui renvoie à cette sous-section (C-28-5-27) ; et `autoMatchRejectedAt` cité à la
ligne `GET /api/v1/bank-imports/{id}` (l. 574).

**AC10 — Les gardes existantes suivent, et les commentaires disent vrai.**
- `crates/kesh-api/tests/audit_route_registry.rs` : ligne
  `("post", "reconciliation::post_unreject_transaction", Traced, SansEcritureAuJournal)` ; au point (iv) du doc du
  module, après la phrase sur `post_reject`, l'angle mort de la route neuve (C-28-5-30) ; compteurs
  **recomptés** — `LIB_ROUTES` 115 → 116 (l. 645), `traced` 109 → 110 (l. 648, message complété « plus l'annulation
  d'un rejet (28-5a, #526) »), `sans_ecriture` 91 → 92 (l. 991, « 90 routes de lib.rs » → 91, l. 992), total
  118 → 119 (l. 667, l. 997) ; doc du module (l. 27, l. 31-32 : « 115 routes », « (118) ») propagée.
- `crates/kesh-api/src/audit_labels.rs` : `"reconciliation.unrejected"` dans `ACTIONS`, à sa place alphabétique
  (après `reconciliation.split_applied`, avant `reconciliation_rule.applied`).
- `crates/kesh-api/tests/textes_coherents.rs` (G9, `les_replis_rust_suivent_le_catalogue`, l. 679) : ligne
  `("reconciliation-errors-transaction-not-rejected", 1, &[])` au bloc `crates/kesh-api/src/errors.rs`.
- Doc-comments : en-tête de `routes/reconciliation.rs` ; `conclude_locked_attempt` (« Conclut une tentative de
  `post_manual` ou `post_split` » → les trois routes ; et ses deux commentaires internes, « ni `post_manual` ni
  `post_split` n'ouvrent de savepoint » l. 4366 et « ces routes ne touchent pas aux règles » l. 4384, qui restent
  vrais pour `unreject` et nomment désormais les trois — C-28-5-28) ; « Seul appelant : `reject_batch` » de
  `find_by_id_for_account_for_update` et l'en-tête de `repositories/reconciliation.rs` ; le doc-comment
  d'`is_reconciled` (`repositories/bank_imports.rs:338-355`) — l'inventaire « clos » des écrivains (l. 338-344 :
  « et l'**annulation du rejet** (`unreject_once`) non plus ») **et** la liste des appelants (l. 350-355) ; son
  reflet dans `bank-import-deletion.ts:12-16` va à la **28-5b** (C-28-5-26) (« écrite deux fois, et seulement deux »
  reste vrai : `classify_unreject` **appelle** `is_reconciled`).
- Symptôme grepé après chaque patch (§ *Propagation post-patch*) : `Seul appelant`, `post_manual` ou `post_split`,
  `` post_split` n' `` (doc-comments et commentaires d'appelants), `inventaire clos`, les valeurs `\b115\b`,
  `\b118\b`, `\b109\b`, `\b91\b`.

**AC12 (a) — Mutations.** M1-M8 et **M15** font rougir au moins un test chacune (M15 : B10, C-28-5-25) ;
M13, M14 et **M17** sont déclarées **équivalentes** avec leur preuve (§ Tests, C-28-5-19 ; M17 : `updated_at` porte
`ON UPDATE CURRENT_TIMESTAMP(3)`, équivalence lue sur le DDL **et confirmée par le rejeu de T6a**), au Debug Log. Aucune survivante assumée.

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
grep -oE "Annuler le rejet" "$GATE_LOGS/28-5b-manuel-plat.txt" | wc -l                                       # ≥ 1
grep -oE "aucun geste de l.interface|sans aucune action|Ne rejetez donc" "$GATE_LOGS/28-5b-manuel-plat.txt" | wc -l   # 0
```

Le fichier aplati tient sur **une** ligne : `grep -c` y compte des lignes et rend 0 ou 1 ; `grep -o … | wc -l`
compte les occurrences. (Au `5aad8b35`, `aucun geste de l.interface`, `sans aucune action` et `Ne rejetez donc` y
figurent **une fois chacun**, `Annuler le rejet` **zéro** fois — vérifié en validation P1 pour les deux premiers
motifs, puis en validation P2 sur le PDF aplati par les commandes ci-dessus, qui rendent `0` et `3` : les deux
contrôles rougissent bien avant la correction.)

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

- [x] **T0a — Montage** : bases `kesh_285` / `kesh_e2e_285` (migrations du worktree, seed), port **3038**,
      `CARGO_TARGET_DIR=<worktree>/target`, `node_modules` par `cp -al` (§ 4 de `docs/agents-orchestration.md`).
      Rafraîchir la base avant tout gate complet.
- [x] **T1 — Champ `autoMatchRejectedAt`** (AC1) : `TransactionResponse` + `From` ; test B7.
- [x] **T2 — Route** (AC2, AC3, AC4, AC13) : variante `AppError::ReconciliationTransactionNotRejected { bank_transaction_id }`
      et son bras (`409`, code, message `t("reconciliation-errors-transaction-not-rejected", "<texte fr-CH exact>")`) ;
      dans `routes/reconciliation.rs`, section « Story 28-5 (#526) — annuler un rejet » : `post_unreject_transaction`,
      `unreject_once`, `enum UnrejectOutcome { Unrejected, NotFound, NotRejected }`, `classify_unreject` ;
      l'auxiliaire de lecture du compte et la signature générique de `conclude_locked_attempt` (C-28-5-18) ; route
      dans `lib.rs`.
- [x] **T3 — Gardes et commentaires** (AC7 a, AC10) : registre des routes, `ACTIONS`, G9, deux clés serveur × quatre
      locales, doc-comments ; grep du symptôme.
- [x] **T4 — Tests backend** B1-B10 (§ Tests) ; **`kesh-db` touché (un dépôt, même en doc-comment) → gate complet à
      chaque passe, ciblage interdit** (règle `CLAUDE.md`, § *Pendant une boucle de revue*, exception `kesh-db`) — y
      compris en cours de boucle de revue.
- [x] **T5a — Documentation** (AC9 a) : `docs/api-external.md`.
- [x] **T6a — Mutations** (AC12 a) M1-M8, M13-M15, M17 (M13 et M14 équivalentes : preuve au Debug Log ; **M17 rejouée** — son équivalence, lue sur le DDL, se confirme en l'exécutant : B1 doit rester vert sous M17, résultat consigné au Debug Log ; si B1 rougit, M17 n'est pas équivalente et la fiche se corrige), sous verrou ; restauration `git checkout -- <f>` **puis
      `touch <f>`**.
- [x] **T7a — Gate et comptes rendus** : gate complet **et** suite E2E complète au dernier commit de code (verrou de
      gate, recette de `docs/testing.md`, échecs jugés fichier par fichier) ; la 28-5a ne touche **aucun** fichier
      frontend (C-28-5-26) : aucun gate frontend n'est exigé d'elle — **s'il en touchait un**, ajouter `npm run
      check`, `lint-i18n-ownership`, `test:unit` et `build` au même commit ; Dev Agent Record (dont la phrase
      « aucune release entre les deux PR »), File List, Change Log ; décomptes recomptés, périmètre déclaré.

### 28-5b

- [ ] **T0b — Montage** : idem T0a, sur `origin/main` après la fusion de la 28-5a.
- [ ] **T8 — Frontend** (AC5, AC6) : type, `unrejectTransaction`, `reconciliation-unreject.ts` (`isRejected`),
      `UnrejectTransactionAction.svelte`, page de détail, indication ; fixtures complétées ; doc-comment de
      `bank-import-deletion.ts:12-16` (appelants serveur d'`is_reconciled`, dont `classify_unreject` — C-28-5-26).
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
| B1 | `unreject_puts_a_rejected_transaction_back_in_the_proposals` | rejet par la route ; `updated_at` **forcé** à une valeur passée au montage, par un `UPDATE … SET updated_at = <valeur>` qui l'affecte **explicitement** (sans quoi `ON UPDATE` écraserait la valeur forcée) (un rejet et son annulation peuvent tomber dans la même milliseconde) ; `204` ; marqueur `NULL`, `version` +1, `updated_at` **postérieur** à la valeur forcée (établit que la ligne a bien été écrite par l'annulation ; ne tue **pas** M17, la colonne étant renouvelée par `ON UPDATE`), `status`/`matched_entry_id` inchangés ; nombre de lignes de `journal_entries` et d'`invoice_settlements` de la société **inchangé** (aucune écriture, aucun règlement) ; la transaction est dans `GET /proposals` (absente avant) ; **une** ligne `reconciliation.unrejected` dont `previouslyRejectedAt` est **la même chaîne** que l'`autoMatchRejectedAt` lu par `GET /bank-imports/{id}` avant l'annulation ; puis **cycle** (AC13) : `POST /reject` du même id → `rejected[]` le contient, `failed[]` vide, marqueur reposé ; `…/unreject` → `204` |
| B2 | `unreject_refuses_a_transaction_that_is_not_rejected` | jamais rejetée → `409` code ; un second appel après succès → `409` ; ni `version` ni `audit_log` ne bougent |
| B3 | `unreject_refuses_a_reconciled_transaction_even_if_marked` | état **forgé, et dit tel** (`status='reconciled'`, `matched_entry_id`, marqueur posé — patron du test l. 4438) → `409` ; marqueur intact |
| B4 | `unreject_roles_and_scope` | Consultation → `403` ; autre société → `404` et rien d'écrit (comptes de lignes avant/après, patron l. 4385) ; Administrateur → `204` |
| B5 | `unreject_returns_409_when_the_account_is_locked` | verrou nommé du compte tenu par une autre connexion → `409 RECONCILIATION_ACCOUNT_LOCKED`, marqueur intact |
| B6 | `an_api_key_unrejects_and_is_named_in_the_audit` | clé `read-write` → `204` ; `actor_type = 'api_key'`, `actor_api_key_id` = la clé |
| B7 | `the_import_detail_exposes_the_rejection_date` | `GET /bank-imports/{id}` : `autoMatchRejectedAt` chaîne ISO terminée par `Z` pour la rejetée, `null` pour l'autre ; `null` après l'annulation |
| B8 | `mod unreject_classification_tests` (unitaire, `routes/reconciliation.rs`) | `classify_unreject` : `None` → `NotFound` ; rapprochée marquée → `NotRejected` ; non marquée → `NotRejected` ; rejetée → `Unrejected` |
| B9 | `a_transaction_whose_rejection_is_undone_can_be_accepted_again` | AC13 : proposition `split` (`setup_accept_split_ctx`) ; rejet ; `POST /accept` → `failed[]` en `RECONCILIATION_ALREADY_REJECTED` ; `…/unreject` → `204` ; `POST /accept` de nouveau → acceptée, `status = 'reconciled'`, marqueur `NULL` |
| B10 | `unreject_of_a_transaction_deleted_under_the_lock_is_not_found` | AC3, « disparue sous verrou » — **déterministe**, patron de `post_reject_locking_reread_sees_a_deletion_committed_after_the_snapshot` (`reconciliation_e2e.rs:7225-7330`) : rejet ; `pool.begin()` + `DELETE FROM bank_transactions WHERE id = ?` **non validé** (montage asserté : une ligne) ; `…/unreject` lancé dans une tâche ; `kesh_db::test_fixtures::attendre_une_requete_en_cours` sur les motifs **`["bank_transactions", "bank_account_id = ?"]`** — communs au `SELECT … FOR UPDATE` (`bank_tx_by_id_for_account_sql`) **et** à l'`UPDATE` prescrit, absents de la lecture hors verrou (`WHERE id = ? AND company_id = ?`) —, appelée deux fois à 500 ms d'écart (observation durable, C-27-4-26) ; `commit` de la suppression ; **`404 NOT_FOUND`** attendu, et **zéro** ligne `reconciliation.unrejected`. Sous M15, la lecture ordinaire voit encore la ligne, l'`UPDATE` attend le verrou X puis touche 0 ligne → `409 OPTIMISTIC_LOCK_CONFLICT` : rouge sur l'assertion du statut, non sur le montage (C-28-5-25). Doc-comment de B10 : reprendre du test de la 27-4 l'**angle mort de la sonde** (elle voit une requête **en cours**, non une requête **en attente de verrou** — `INNODB_TRX` exige le privilège `PROCESS`, absent sur le poste de dev ; sous M15, la mutation ne survivrait que si la lecture ordinaire durait plus de 500 ms) et la précaution **MariaDB ≥ 11.6** (`innodb_snapshot_isolation`) |

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
| M15 | a | lecture **non** verrouillante (`find_pending_by_id_for_account`, l. 351) au lieu de `find_by_id_for_account_for_update` | **B10** : la suppression d'import ne prend pas le verrou nommé, sa transaction entière est la fenêtre, et le test la tient ouverte ; code juste → `404`, M15 → `409 OPTIMISTIC_LOCK_CONFLICT` (C-28-5-25, qui remplace C-28-5-23). M15 garde aussi M13 : l'équivalence de M13 suppose la lecture verrouillante |
| M17 | a | `UPDATE` : retirer `updated_at = NOW(3)` | **équivalente** : `bank_transactions.updated_at` est `DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)` (`20260504000001_bank_imports.sql:74` ; même définition dans le squash `crates/kesh-db/test-schema/0001_schema_squash.sql`) : l'`UPDATE` change `auto_match_rejected_at` et `version`, le moteur renouvelle donc la colonne même sans l'affectation explicite. L'affectation reste dans la requête (lisibilité, patron du dépôt) |
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
   (lecture **courante** et verrouillante — une `SELECT` ordinaire lirait la vue cohérente, cf. son doc-comment ; elle
   attend une suppression d'import en cours puis la voit, d'où le `404` de B10) → `classify_unreject`. `NotFound` ou `NotRejected` : `Ok(outcome)` sans rien écrire.
4. `Unrejected` : `UPDATE bank_transactions SET auto_match_rejected_at = NULL, updated_at = NOW(3), version = version + 1
   WHERE id = ? AND company_id = ? AND bank_account_id = ?`, erreur rangée par
   `.map_err(|e| ReconciliationError::Db(DbError::Sqlx(e)))?` (patron de `post_manual_once`, `:3800-3820`) ;
   `rows_affected() != 1` → `Err(ReconciliationError::Db(DbError::OptimisticLockConflict))` (`:3822-3826`).
5. Audit : `audit_log::insert_in_tx(tx_inner, NewAuditLogEntry::for_actor(…)).await?` (patron `:3840-3851` : le `?`
   range l'erreur en `ReconciliationError::Db`, `DbError::Sqlx` préservé) ; `details` construit avec le marqueur lu
   au point 3, `.and_utc()`.
6. Après la fermeture : `conclude_locked_attempt(tx, lock_result, "post_unreject").await?` — l'auxiliaire existant
   de `post_manual` et `post_split` (doc-comment `:4312`, fonction `:4325-4399`), dont la **signature** devient
   générique (`lock_result: Result<T, ReconciliationError>` → `Result<T, AppError>`) ; logique et `match`
   **inchangés**, avec trois retouches de texte seulement (C-28-5-28) : la liaison `Ok(journal_entry_id)` devient
   `Ok(value)` (le nom serait faux pour `UnrejectOutcome`), et les commentaires internes `:4366` et `:4384`, qui
   nomment les appelants, nomment les trois. Il `commit` si la fermeture a réussi (pour un refus, rien n'a été
   écrit : le `commit` vaut un `rollback`), sinon `rollback` et conversion qui préserve `DbError::Sqlx` — un
   `OptimisticLockConflict` sort en `AppError::Database`, que le mapping de `errors.rs:2922-2925` rend
   `409 OPTIMISTIC_LOCK_CONFLICT`. Puis : `Unrejected` → `204` ; `NotFound` →
   `AppError::Database(DbError::NotFound)` ; `NotRejected` → `AppError::ReconciliationTransactionNotRejected`.
   **Aucune troisième copie** du `match` de conversion (C-28-5-18) ; celui de `cancel_reconciliation_once` reste tel
   quel.

**Pas de rejeu sur interblocage** (`SansEcritureAuJournal`, comme `post_reject`) : la transaction ne verrouille
qu'**une** ligne de `bank_transactions` puis insère une ligne d'audit (`audit_log` n'a aucune clé étrangère) ; la
suppression d'un import (27-1) verrouille les lignes de l'import par identifiant croissant **sans** prendre le verrou
nommé ; la création d'import n'écrit que des lignes neuves ; la restauration d'installation écarte `audit_log` de son
`DELETE`. **Un cycle reste possible, un seul, et il est nommé** (C-28-5-30) : la **remise à zéro d'onboarding**
(`kesh_seed::reset_demo`, `POST /onboarding/reset`) prend `audit_log` par une lecture verrouillante **puis**
supprime `bank_transactions` (`docs/MULTI-TENANT-SCOPING-PATTERNS.md:324`, `kesh-seed/src/lib.rs:576`) — l'ordre
inverse de l'annulation. Si InnoDB désigne l'annulation comme victime, elle rend un `500` **sans rien de partiel**
(annulation complète de sa transaction) et la remise à zéro est rejouée ; c'est rare (installation de démonstration
ou d'avant production) et c'est **le cas même de `post_reject`**, préexistant. Écrit au point (iv) du doc
d'`audit_route_registry.rs` (AC10). Hors ce cas, aucun cycle à `5aad8b35`. **La garde de la 28-4 ne prendra aucun
verrou** (C-28-4-21) : la garde « compte clos » qu'elle ajoute à `unreject` (W30) lit la ligne du compte par une
lecture cohérente non verrouillante, et n'ajoute donc aucun verrou à l'ordre ci-dessus ; l'argument reste à relire
sur le code fusionné, au rebase sur la 28-4a. *(Corrigé le 2026-10-13, sur note de l'orchestrateur : la phrase
annonçait un `bank_accounts FOR UPDATE` pris par `unreject` du fait de la 28-4.)* Le verrou nommé est relâché avant le `commit`
(`kesh-reconciliation/src/mutex.rs`, propriété commune à tous les gestes) : un geste concurrent qui obtient le verrou
dans cet intervalle bute sur la lecture `FOR UPDATE` et attend le `commit` ; un lot qui aurait classé la ligne avant
peut la refuser (`RECONCILIATION_ALREADY_REJECTED`) — refus bénin, à relancer, déjà le cas entre tous les gestes.

### Inventaire des sites NON RÉSOLUS (D4-ter)

Classe de défaut : *une transaction rejetée qu'aucun écran ne montre ou ne permet de reprendre.* Ensemble clos,
obtenu par `grep -rn "auto_match_rejected_at" crates --include=*.rs --include=*.sql | grep -v /tests/` et
`grep -rn "autoMatchRejectedAt\|auto_match_rejected" frontend/src` (vide à `5aad8b35`) ; élargi en validation P1
(`rejectedAt|wasRejected|was_previously|rejected_at` sur `frontend/src docs crates/kesh-seed` : seul
`reconciliation.types.ts:104`, `rejectedAt` de `RejectedProposal`, autre objet ; `kesh-seed`, `test_endpoints.rs`,
`post_restore.rs` : aucune occurrence). Le premier grep rend **45 lignes** à `5aad8b35` (recomptées en validation
P2) ; **chacune** figure ci-dessous — les lignes de code de production par site (la migration `…8_4.sql`, l. 3, 6 et 15, en une ligne), et, dans les deux dernières lignes
du tableau, ce que le filtre `grep -v /tests/` laisse passer sans que ce soit du code de production (commentaires et
fixtures `#[cfg(test)]`) :

| Site | Rôle vis-à-vis du marqueur | Après la story |
|---|---|---|
| `repositories/reconciliation.rs:182` (`find_pending_transactions_for_account`) | exclut les rejetées des propositions | **voulu** (c'est le sens du rejet) ; l'indication de l'AC6 renvoie au détail |
| `routes/reconciliation.rs:885` (`classify_batch_transaction`) | refuse une rejetée à l'acceptation et au rejet | voulu ; annuler le rejet la rend de nouveau acceptable (AC13, B9) ; son libellé dit où annuler (AC7 b) |
| `routes/reconciliation.rs:3357`, `:3359` (`reject_batch`) | **pose** le marqueur (`:3357`), sous la garde `auto_match_rejected_at IS NULL` (`:3359`) | inchangé |
| `routes/reconciliation.rs:3409` (`reject_batch`) | relit le marqueur posé pour rendre `rejectedAt` | inchangé, sans effet sur la classe |
| `:2540`, `:2992`, `:3803`, `:4260`, `reconciliation_cancel.rs:340` | **remettent** le marqueur à `NULL` | inchangés ; aucun n'est atteignable depuis l'écran pour une rejetée (cf. *Le défaut*) |
| **route neuve** `…/unreject` | remet à `NULL` | **le chemin de l'interface** |
| `routes/reconciliation.rs:2565`, `:3033`, `:3750`, `:4224` (`was_previously_rejected`) ; `reconciliation_cancel.rs:400` (`wasPreviouslyRejected`) | lisent le marqueur pour l'**audit** des gestes qui le remettent à `NULL` et de l'annulation | inchangés, sans effet sur la classe ; une transaction dont le rejet a été annulé y lira `false` — exact, puisqu'elle n'est plus rejetée ; l'audit d'`unreject` porte `previouslyRejectedAt` (AC2) |
| `repositories/bank_imports.rs:37` (`COLUMNS`), `bank_transactions.rs:24`, `repositories/reconciliation.rs:82` (`BANK_TX_COLUMNS`) | listes de colonnes lues | inchangées, sans effet sur la classe |
| `repositories/bank_imports.rs:330`, `:373`, et le comptage `:488-490` qui ne nomme pas la colonne (`rejected_count` de la suppression d'import, 27-1) | comptent les rejetées supprimées avec leur import | inchangés ; vrais (une rejetée reste supprimable avec son import) |
| `entities/bank_transaction.rs:105` ; `migrations/20260507100001_reconciliation_8_4.sql` ; `test-schema/0001_schema_squash.sql` | définition de la colonne | inchangés |
| `routes/bank_imports.rs:250` (`TransactionResponse`) | n'exposait pas le marqueur | **résolu** (AC1) |
| `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` | seul écran qui liste une rejetée | **résolu** (AC5) |
| `exports/csv_tables.rs:740` (en-tête), `:764` (valeur) | exporte la colonne | déjà résolu (export global) |
| page *Imports précédents* (`BankImportList.svelte`) | ne compte pas les rejetées | **angle mort assumé** : l'utilisateur ouvre l'import d'après la date de la transaction ; un compteur serait du périmètre en plus |
| colonne *Statut* du détail (`pending` brut) | ne dit pas « rejetée » | **angle mort assumé**, la mention de l'AC5 le dit à côté ; traduction suivie par #339 |
| liste de la page *Réconciliation* au-delà de 100 lignes | une rejetée annulée, ancienne, n'y reparaît pas | **angle mort assumé, suivi par #649** ; les textes le disent (C-28-5-20) |
| *Hors code de production* — lignes de **commentaire** : `routes/reconciliation.rs:12`, `:857`, `:2535`, `:3129`, `:3353`, `:3394`, `:3401`, `:3501`, `:4256` ; `repositories/reconciliation.rs:12`, `:163`, `:330`, `:337` | décrivent le marqueur | sans effet sur la classe ; l'en-tête `routes/reconciliation.rs:12` et `repositories/reconciliation.rs:12` reçoivent la puce de la route neuve (AC10) |
| *Hors code de production* — **fixtures `#[cfg(test)]`** : `kesh-reconciliation/src/matching.rs:208`, `manual.rs:152`, `split.rs:217`, `rules.rs:170` (`auto_match_rejected_at: None`, dans les `#[cfg(test)]` ouverts l. 178, 127, 192, 137) ; `routes/reconciliation.rs:4799` (module de tests) | construisent une transaction de test | sans effet |

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
  conversion et tout le reste sont intouchés ; ses tests (l. 3451-4552 de `reconciliation_e2e.rs`) restent verts
  sans modification.
- `post_manual`, `post_split` : `conclude_locked_attempt` ne change que de signature (générique), plus le nom de la
  liaison `Ok(value)` et les deux commentaires d'appelants (C-28-5-28) ; sa logique, son `match` et le
  `DbError::Sqlx` qu'il préserve pour leur rejeu sont intouchés.
- `TransactionResponse` : champs existants inchangés ; le détail d'import de la 27-1 (`DeleteBankImportAction`,
  `reconciledCount`) ne lit pas le champ neuf.
- Aucune migration : `kesh_version_min_required` inchangé, aucune ligne au registre d'idempotence, aucun backfill.

### References

- Issue #526 et ses deux commentaires (lentilles R et F de la 15-5c) ; epic § 28-5 ; #649 (limite de 100).
- `crates/kesh-api/src/routes/reconciliation.rs` : `reject_batch` `:3338-3462`, `classify_batch_transaction`
  `:872-897`, `post_manual_once` `:3699-3866`, `conclude_locked_attempt` `:4312-4399` (doc-comment compris),
  annulation `:4400-4586` ; mapping `DbError::OptimisticLockConflict` → `409 OPTIMISTIC_LOCK_CONFLICT` :
  `errors.rs:2922-2925`.
- Test patron de B10 : `reconciliation_e2e.rs:7225-7330` ; sonde `kesh-db/src/test_fixtures.rs:561`.
- `crates/kesh-db/src/repositories/reconciliation.rs:330-416` ; `repositories/bank_imports.rs:340-358`.
- `docs/api-external.md:534-575` ; `docs/manual/fr/user-manual.tex:1933-1952`.
- Fiche de la 28-4 (`story/28-4`, `7a4adf64`) : W30, R8, M33 pour la route `unreject`.
- Consignes : `docs/agents-orchestration.md` ; `CLAUDE.md` § *Review Iteration Rule*, § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

28-5a : Claude Opus 5.5 (`bmad-dev-story`, autonomie, worktree `kesh-28-5`, port 3038, bases `kesh_285` /
`kesh_e2e_285`).

### Debug Log References

Journaux hors dépôt, sous `/home/gcorbaz/devel/kesh-gate-logs/28-5a/`.

- **Rouge d'abord** (`red.log`, tests écrits, code absent — seul un bouchon `classify_unreject` rendant toujours
  `NotFound` pour que B8 compile) : filtre de 14 tests (`test(unreject) | …`, qui attrape aussi
  `batch_classification_tests::pending_unrejected_row_is_eligible`, préexistant) — **12 rouges, 2 verts** : les neuf
  tests HTTP B1-B7, B9, B10 rougissent (route absente : `405` au lieu du statut attendu ; B7 : champ
  `autoMatchRejectedAt` absent ; B10 : la sonde ne voit aucune attente, montage asserté), et B8 rougit sur 3 de ses
  4 cas (`absent_row_is_not_found` vert par construction du bouchon).
- **Vert** (`green1.log`, gate ciblé) : `fmt` et `clippy --workspace --all-targets -D warnings` verts ; nextest sur
  `reconciliation_e2e`, `audit_route_registry`, `textes_coherents`, `audit_label_registry`, `bank_imports_e2e`, la
  bibliothèque `kesh-api` et `kesh-i18n` : **657 passés, 0 échec, 2 ignorés** ; les 13 tests de la story (B1-B7, B9,
  B10 et les quatre cas de B8) verts.
- **Mutations T6a** (`mutations/`, `mutations-run.txt`, sous le verrou de gate ; script
  `scratchpad/28-5a/mutations-28-5a.py`, motif vérifié unique, restauration `git checkout --` puis `touch`, `git
  status` des crates vide après la campagne). Filtre de 14 tests ci-dessus à chaque mutation :

  | # | Résultat | Tuée par |
  |---|---|---|
  | M1 | tuée | B8 (`reconciled_row_is_not_rejected_even_if_marked`), B3 |
  | M2 | tuée | B8 (`unmarked_row_is_not_rejected`), B2 |
  | M3 | tuée | B1 |
  | M4 | tuée | B1, B2, B6 |
  | M5 | tuée | B6 |
  | M6 | tuée | B5 (fermeture appelée directement, sans `with_account_lock`) |
  | M7 | tuée | B4 (route montée dans `authenticated_routes`) |
  | M8 | tuée | B7, B1 |
  | M13 | **survit — équivalente** (14/14 verts) | la ligne est lue `FOR UPDATE` sous verrou, scopée par sa clé primaire : l'`UPDATE` ne peut toucher qu'elle (C-28-5-19) ; M15 garde la prémisse |
  | M14 | **survit — équivalente** (14/14 verts) | `id` est la clé primaire et la ligne a été lue scopée société et compte sous verrou ; les deux prédicats restent en défense en profondeur |
  | M15 | tuée | B10 (`409 OPTIMISTIC_LOCK_CONFLICT` au lieu de `404`) |
  | M17 | **rejouée — survit, équivalente** (14/14 verts, dont B1) | `bank_transactions.updated_at` porte `ON UPDATE CURRENT_TIMESTAMP(3)` : sans l'affectation explicite, le moteur renouvelle la colonne et B1 voit toujours `updated_at` postérieur à la valeur forcée. **L'équivalence lue sur le DDL est confirmée par l'exécution** ; la fiche n'a pas à être corrigée |

  Aucune survivante assumée : les trois survivantes sont les trois équivalences prescrites (AC12 (a)).
- **Gate final au dernier commit de code `822ba587`** (`final-gate.out`, sous le verrou de gate, bases `kesh_285` et
  `kesh_e2e_285` remises à zéro par `DROP`/`CREATE` + migrations + seed, sans redémarrer le conteneur) :
  - `scripts/test-fast.sh` (fmt + clippy + nextest, `final-testfast.log`) : **3527 passés, 0 échec, 6 ignorés**
    (baseline v0.13.2 : 3514 ; +13 = les tests de la story) ;
  - frontend (C-28-5-33 : aucun fichier frontend touché, mais des gardes Vitest lisent les catalogues) :
    `npm run check` 0 erreur, 29 avertissements préexistants (`BankProfileForm.svelte` notamment) ;
    `lint-i18n-ownership` PASS ; `test:unit` **1395 passés, 134 fichiers** ; `build` vert ;
  - E2E complet (recette de la consigne 6, port 3038, `final-e2e.log`) : **267 passés, 7 échecs, 19 ignorés**. Les
    sept échecs sont nominativement les sept KF-029 (#97) de `docs/testing.md` § *Les échecs attendus* :
    `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`.
    Aucun huitième de pollution, aucun KF-045 (run de l'après-midi), pas de KF-046. Aucune régression.

### Completion Notes List

- **28-5a livrée côté serveur** : `POST /api/v1/reconciliation/transactions/{id}/unreject` (Comptable+, clés
  `read-write`), `204` ; `classify_unreject` pure (appelle `is_reconciled`, aucun second test de statut) ;
  `unreject_once` sous `with_account_lock`, lecture `find_by_id_for_account_for_update`, `UPDATE` scopé
  `id`/`company_id`/`bank_account_id`, garde `rows_affected() != 1` → `OptimisticLockConflict`, audit
  `reconciliation.unrejected` (`previouslyRejectedAt`, `.and_utc()`, `for_actor`) ; `conclude_locked_attempt` rendu
  générique (liaison `Ok(value)`, deux commentaires internes et doc-comment nommant les trois appelants, logique
  intouchée) ; la lecture du compte de `…/cancel` passe dans `bank_account_of_transaction`, mot pour mot, son
  `match` intouché ; `AppError::ReconciliationTransactionNotRejected` (`409`, repli inscrit à G9) ;
  `TransactionResponse.auto_match_rejected_at` (AC1).
- Gardes suivies : registre des routes (`LIB_ROUTES` 116, `traced` 110, `sans_ecriture` 92, total 119, doc du
  module et point (iv)), `ACTIONS`, G9 ; deux clés serveur × quatre locales ; doc-comments de
  `repositories/reconciliation.rs` (en-tête, « Deux appelants ») et d'`is_reconciled` (inventaire des écrivains,
  « trois chemins » appelants). Symptômes grepés après patch : `Seul appelant` (0 dans `crates`, `frontend/src`,
  `docs`), `` post_manual` ou `post_split `` / `` post_split` n' `` (0), `\b115\b`/`\b118\b`/`\b109\b` dans
  `audit_route_registry.rs` (0), `\b91\b` (une ligne, le libellé « 91 routes de lib.rs », juste), `inventaire clos`
  (une ligne, l'en-tête d'`is_reconciled`, juste) ; `bank-import-deletion.ts` non touché (28-5b, C-28-5-26).
- `docs/api-external.md` : sous-section « Annuler le rejet d'une transaction bancaire », renvoi au paragraphe
  « Transaction disparue, rapprochée ou rejetée », `autoMatchRejectedAt` à la ligne `GET /api/v1/bank-imports/{id}`.
- **Aucune release entre les deux PR** : entre la fusion de la 28-5a et celle de la 28-5b, l'annulation n'existe que
  par l'API, sans écran, sans manuel ni CHANGELOG — la 28-5b les apporte.
- **Décomptes** (périmètre : `da25bf1b` — validation close — à `822ba587` — dernier commit de code) : **13 tests
  neufs** — `reconciliation_e2e.rs` 81 → 90 attributs `#[sqlx::test]` (+9 : B1-B7, B9, B10) et
  `routes/reconciliation.rs` 14 → 18 `#[test]` (+4 : B8) ; 15 fichiers de code, de test, de catalogue et de doc
  touchés, plus deux de `_bmad-output` (registre, registre de sprint) — `git diff --stat da25bf1b 822ba587` : 17
  fichiers. **Deux** clés i18n neuves × quatre locales. Aucune migration, aucun `Cargo.toml`, aucun fichier
  frontend.
- Choix consignés : **C-28-5-31** (constructeur de test partagé, `pub(super)`), **C-28-5-32** (place de
  l'auxiliaire), **C-28-5-33** (gates frontend exécutés), **C-28-5-34** (fiche corrigée : la garde de la 28-4 ne
  verrouille pas le compte, C-28-4-21).
- À l'attention de la revue : la sonde de B10 n'observe qu'une requête **en cours** (angle mort repris de la 27-4,
  écrit au doc-comment) ; la 28-4 ajoutera sa garde « compte clos » à `unreject` (W30) — non traitée ici.

### File List

- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/src/routes/bank_imports.rs`
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/audit_labels.rs`
- `crates/kesh-api/tests/reconciliation_e2e.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-api/tests/textes_coherents.rs`
- `crates/kesh-db/src/repositories/reconciliation.rs` (doc-comments)
- `crates/kesh-db/src/repositories/bank_imports.rs` (doc-comment)
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
- `docs/api-external.md`
- `_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md`
- `_bmad-output/implementation-artifacts/28-5a-review-prompt-p1.md` (neuf)
- `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

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
- 2026-10-13 — **Validation P2** (Opus 5.5, lentilles R et F, rapports `$GATE_LOGS/28-5-validate-p2-{R,F}.md`) :
  F — 0 HIGH, 2 MEDIUM, 8 LOW ; R — 0 HIGH, 2 MEDIUM, 10 LOW ; soit **4 MEDIUM, 2 distincts** (F1 = R1, F2 = R2,
  tous deux issus de la remédiation P1) et **18 LOW**, dont cinq recoupements (F4 = R4, F6 = R10, F9 = R3,
  F10 = R9, F8 ⊃ R5). Remédiation (Opus 5.5, consignes de l'orchestrateur) : **F1 = R1** — le code est
  `409 OPTIMISTIC_LOCK_CONFLICT`, produit par le mapping des erreurs de base (`errors.rs:2922-2925`) et non par
  `conclude_locked_attempt` : AC3, AC9 (a), séquence de la route ; au registre, notes à C-28-5-4 et C-28-5-19 ;
  **F2 = R2** — test **B10** déterministe (patron du test de la 27-4, `DELETE` non validé, sonde, `commit`) : `404`
  attendu, `409` sous M15, qui passe de « survivante assumée » à « tuée » ; C-28-5-25 remplace C-28-5-23, dont la
  prémisse d'un angle mort écrit au doc-comment de la lecture était fausse ; AC12 (a), tableau des mutations, AC3.
  LOW : numéros de ligne de `conclude_locked_attempt` (`:4312-4399`) et du comptage (`:488-490`) (F3) ; inventaire —
  45 lignes du grep recomptées, `:3359`, `:3409`, `csv_tables.rs:764`, commentaires et fixtures `#[cfg(test)]` en
  deux lignes (F4 = R4) ; doc-comment de `bank-import-deletion.ts` passé à la 28-5b, la 28-5a à cinq sites sans
  fichier frontend (F5, C-28-5-26) ; ordre 28-5a, puis 28-6 ∥ 28-5b, puis 28-7 au § *Ordre* de l'epic et à la fiche,
  régions `sitesTotal` et `failed-proposal-label.test.ts` (F6 = R10, C-28-5-29) ; commentaires internes et liaison
  de `conclude_locked_attempt` retouchables, motif `` post_split` n' `` au grep (F7, R12, C-28-5-28) ; point (iv)
  d'`audit_route_registry.rs` et inventaire « clos » d'`is_reconciled` (l. 338-344) dans l'AC10 (F8, R5) ; argument
  « aucun cycle » nuancé — remise à zéro d'onboarding, `500` rare sans écriture partielle comme `post_reject`, à
  revérifier au rebase sur la 28-4a (F9 = R3, C-28-5-30) ; contrôle PDF en `grep -o … | wc -l` avec
  `Ne rejetez donc` (F10 = R9 ; rejoué sur le PDF aplati : `0` et `3`) ; renvoi d'API au paragraphe l. 540 (R6,
  C-28-5-27) ; T4 en gate complet à chaque passe, T7a sans gate frontend faute de fichier frontend (R7) ; B1 compte
  `journal_entries` et `invoice_settlements` et vérifie `updated_at` (forcé dans le passé au montage), B4 appelle en
  Administrateur, mutation **M17** (R8) ; chemin d'import d'`errorMessageOf` qui suit la 28-2 (R11). Registre de
  sprint et § *Ordre* de l'epic alignés. Recompte après remédiation : **13 AC**, tâches T0a-T7a et T0b-T14,
  **B1-B10**, **V1-V6**, **M1-M17**, choix C-28-5-1 à C-28-5-30 (C-28-5-11, 12 et 23 remplacés) ; six clés i18n
  neuves et une réécrite, × quatre locales (inchangé). Passe **ciblée P3** à lancer
  (`28-5-validate-prompt-p3-ciblee.md`, Haiku, une lentille sur le seul diff de cette remédiation).
- 2026-10-13 — **Validation P3 ciblée** (Haiku 5.5, une lentille sur le diff de la remédiation P2, rapport
  `$GATE_LOGS/28-5-validate-p3-ciblee.md`) : **0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW**. Le MEDIUM (P3-1) est issu de la
  remédiation P2 (R8) et confirmé par l'orchestrateur. Remédiation : **P3-1** — M17 (retirer `updated_at = NOW(3)`)
  est **équivalente**, non « tuée par B1 » : `bank_transactions.updated_at` porte
  `ON UPDATE CURRENT_TIMESTAMP(3)` (`20260504000001_bank_imports.sql:74` ; squash `test-schema/0001_schema_squash.sql`,
  colonne `updated_at` de `bank_transactions`, même définition), et l'`UPDATE` change `auto_match_rejected_at` et
  `version` : le moteur renouvelle la colonne seul. AC12 (a), tableau des mutations, T6a, `sprint-status.yaml` (28-5a)
  alignés ; l'assertion de B1 sur `updated_at` est **conservée** mais ne prouve que l'écriture de la ligne (elle ne
  tue aucune mutation) ; **P3-2** — la ligne `GET /api/v1/bank-imports/{id}` de `docs/api-external.md` est à l. **574**
  (région du tableau et AC9 (a)) ; **P3-3** — « (nuancé, C-28-5-30) » au *Retenu* de C-28-5-4. L'entrée P2 ci-dessus
  n'est pas réécrite : son « B1 tue M17 » et ses « M1-M17 tuées » sont **annotés** par la présente entrée (règle B4).
  Recompte après remédiation (inchangé) : **13 AC**, tâches T0a-T7a et T0b-T14, **B1-B10**, **V1-V6**, **M1-M17**
  (dont **trois équivalentes** en partie a : M13, M14, M17), choix C-28-5-1 à C-28-5-30. Passe **ciblée P4** à lancer
  (`28-5-validate-prompt-p4-ciblee.md`, une lentille sur le seul diff de cette remédiation).
- 2026-10-13 — **Validation P4 ciblée** (Haiku 5.5, une lentille sur le diff de la remédiation P3, rapport
  `$GATE_LOGS/28-5-validate-p4-ciblee.md`) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW. **F1 reclassé LOW par
  l'orchestrateur** : il ne relève aucun défaut de la conception — l'équivalence de M17 est juste sur le DDL, et
  aucun contre-exemple n'a été trouvé —, mais une formulation (« pas de rejeu ») qui aurait fait déclarer au dev une
  équivalence non exécutée ; une spec ne s'exécute pas, sa campagne de mutations si. Remédiation (orchestrateur) :
  T6a **rejoue M17** et consigne le résultat, AC12 (a) le dit (F1, F2) ; B1 prescrit le forçage par une affectation
  **explicite** de `updated_at`, sans quoi `ON UPDATE` écraserait la valeur forcée (F3). La remédiation ne touche que
  la formulation de deux tâches et d'un test prescrit, aucun AC ni aucune règle métier : **boucle de validation
  close** — trend P1 → P4 : 6 MEDIUM (5 distincts) → 4 MEDIUM (2 distincts) → 1 MEDIUM → 0 MEDIUM après
  reclassement, 0 HIGH à toutes les passes ; modèles Sonnet, Opus, Haiku, Haiku. Recompte inchangé : 13 AC, B1-B10, V1-V6, M1-M17, C-28-5-1 à C-28-5-30.
- 2026-10-13 — **Fiche corrigée** (dev 28-5a, sur note de l'orchestrateur) : § *Séquence de la route*, la garde
  « compte clos » de la 28-4 ne prendra aucun verrou — lecture cohérente non verrouillante (C-28-4-21) ; la phrase
  qui annonçait un `bank_accounts FOR UPDATE` dans `unreject` est réécrite. Aucun code touché. Choix C-28-5-34.
- 2026-10-13 — **Développement de la 28-5a** (`bmad-dev-story`, Opus 5.5) : T0a-T7a. Route `…/unreject`, champ
  `autoMatchRejectedAt`, gardes, deux clés serveur × quatre locales, doc-comments, `docs/api-external.md` (commit de
  code `822ba587`). Tests B1-B10 rouges avant la correction (12 des 14 tests du filtre), verts après. Mutations :
  M1-M8 et M15 tuées ; M13, M14 et **M17 (rejouée)** survivent, équivalentes. Gate final au `822ba587` : nextest
  3527/3527 (6 ignorés), Vitest 1395/1395, `check` 0 erreur, `lint-i18n-ownership` et `build` verts, E2E 267 passés
  / 7 échecs, les sept KF-029. Choix C-28-5-31 à 33. Statut **review** ; revue de code P1 à lancer
  (`28-5a-review-prompt-p1.md`, Sonnet, lentilles B, E et A).
