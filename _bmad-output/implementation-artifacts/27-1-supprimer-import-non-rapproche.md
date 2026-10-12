# Story 27.1 : Supprimer un import bancaire dont aucune transaction n'est rapprochée

## Status

**review** — développement terminé le 2026-10-12 (`bmad-dev-story`, dernier commit de code `6e305767`) ; revue de code **P1** (Sonnet, B/E/A : 0 HIGH, 1 MEDIUM, 14 LOW) **remédiée** — dernier commit hors fiche `5881450b`, gate complet et E2E complet verts à ce commit ; revue de code **P2** (Opus, B/E/A : 0 HIGH, 4 MEDIUM, 10 LOW) **remédiée** — dernier commit de code `1c9e4ea0`, gate complet et E2E complet verts à ce commit (E2E : les sept échecs de la KF-029) ; revue de code **P3** à lancer (`27-1-review-prompt-p3.md`, Sonnet, complète). Validation close (P4 ciblée : 0 MEDIUM, 2 LOW appliqués). Fiche créée le 2026-10-12 en autonomie (`bmad-create-story`), sur `aadace40` (kickoff de
l'Epic 27), branche `story/27-1-supprimer-import`. Validation **P1** (Sonnet, lentilles R et F : 0 HIGH,
10 MEDIUM dont 7 distincts, 14 LOW) **remédiée** ; validation **P2** (Opus, R et F : 0 HIGH, 4 MEDIUM distincts,
16 LOW) **remédiée** le 2026-10-12, avec la **décision du Project Lead** de fermer la suppression aux clés d'API
(C-27-1-16) ; validation **P3** (Sonnet, complète, R et F : 0 HIGH, 2 MEDIUM distincts, 13 LOW) **remédiée** — dont
une sortie par **retrait** sur un angle mort recyclé (C-27-1-20) ; passe **P4 ciblée** (Haiku, une lentille,
prompt `27-1-validate-prompt-p4-ciblee.md`) **rendue : 0 MEDIUM, 2 LOW appliqués — validation close**. Choix consignés au registre `epic-27-choix-autonomes.md`, **C-27-1-1 à
C-27-1-31** (C-27-1-7 remplacé par C-27-1-12 ; C-27-1-3 et C-27-1-13 révisés ; C-27-1-16 complété). 17 AC (AC1-AC16 et AC4-bis). PR : **`closes #632`**.

## Story

**As a** utilisateur qui importe ses relevés (Comptable ou Admin),
**I want** supprimer un import bancaire entier, avec ses transactions, tant qu'aucune d'elles n'est rapprochée,
**so that** je puisse réimporter le même fichier après un import fautif, sans requête SQL ni restauration de
sauvegarde.

Issue : [#632](https://github.com/guycorbaz/kesh/issues/632) (P2, recette de la v0.13.1). Epic :
`_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md`. Décision du Project Lead du 2026-10-12 :
« ajouter une fonction pour **supprimer les imports non rapprochés** » — l'objet est l'**import entier** avec ses
transactions, permis seulement si **aucune** n'est rapprochée ; Guy veut ensuite réimporter le même fichier.
Cible : **v0.13.2**.

⚠️ **Une prémisse de l'epic et de l'issue est périmée, et la fiche la corrige (C-27-1-8).** L'unicité
`uq_bank_imports_company_hash (company_id, file_hash)` **n'existe plus** : la migration
`crates/kesh-db/migrations/20260507000001_bank_imports_relax_hash_unique.sql` (Story 8-3) l'a remplacée par un index
non unique `idx_bank_imports_company_hash`. Le blocage du réimport est **applicatif** : `create` et `create_csv`
cherchent l'empreinte par `bank_imports::find_by_company_and_hash` et refusent en `422 BANK_IMPORT_DUPLICATE_FILE`
sans `confirmDuplicateFile=true` (`crates/kesh-api/src/routes/bank_imports.rs:1108-1119`, et `:1781-1792` pour le
CSV). Supprimer l'entête lève donc le blocage — et ce que Guy refuse aujourd'hui, ce n'est pas une contrainte de
base, c'est d'avoir **deux** imports du même fichier, dont un fautif, l'un à côté de l'autre.

## Critères d'acceptation

**AC1 — « Rapprochée » : une définition fermée, la même au serveur et à l'écran (C-27-1-1).**

Une transaction bancaire est **rapprochée** si et seulement si `status <> 'pending'` **ou**
`matched_entry_id IS NOT NULL`. Un import est **supprimable** si aucune de ses transactions n'est rapprochée — un
import sans transaction l'est donc toujours.

- Une transaction **rejetée** à la réconciliation (`auto_match_rejected_at` posé, `status = 'pending'`,
  `matched_entry_id` nul — `reject_batch`, `reconciliation.rs:3209-3212`) **n'est pas rapprochée** : elle part avec
  son import.
- Une transaction dont le rapprochement a été **annulé** (`reconciliation_cancel::cancel_in_tx`,
  `reconciliation_cancel.rs:338-342` : retour à `pending`, lien et rejet remis à `NULL`) n'est plus rapprochée.
- La définition s'appuie sur l'inventaire clos des écrivains de `bank_transactions` (Dev Notes § Inventaire (b)) ;
  elle est écrite **une fois** côté Rust (dans la requête de garde de l'AC2) et **une fois** côté TypeScript
  (helper de l'AC9), chacune avec un doc-comment qui renvoie à l'autre.

**AC2 — Le dépôt : `bank_imports::delete_unreconciled_in_tx`, verrous dans l'ordre import → transactions (C-27-1-2, C-27-1-4).**

Fonction neuve dans `crates/kesh-db/src/repositories/bank_imports.rs` :

```rust
pub async fn delete_unreconciled_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    import_id: i64,
) -> Result<DeleteBankImportOutcome, DbError>
```

avec, dans le même module (ou `entities/bank_import.rs`, au choix du dev, réexporté) :

```rust
pub enum DeleteBankImportOutcome {
    /// L'import et ses transactions sont supprimés.
    Deleted(DeletedBankImport),
    /// Import inconnu, ou d'une autre société.
    NotFound,
    /// Au moins une transaction est rapprochée : rien n'est supprimé.
    HasReconciled { reconciled_count: i64, transaction_count: i64 },
}
pub struct DeletedBankImport {
    pub import: BankImport,          // l'entête tel qu'il était, lu sous verrou
    pub transaction_count: i64,      // lignes supprimées, comptées sous verrou
    pub rejected_count: i64,         // dont rejetées (auto_match_rejected_at non nul)
}
```

Séquence, **dans cet ordre, sans aucune lecture ordinaire avant le premier verrou** (règle de la 15-8a : une
lecture ordinaire ouvrirait la vue cohérente avant l'attente et ne verrait pas le commit d'une acceptation
concurrente) :

1. `SELECT {COLUMNS} FROM bank_imports WHERE id = ? AND company_id = ? FOR UPDATE` → aucune ligne ⇒
   `Ok(NotFound)`.
2. `SELECT id, status, matched_entry_id, auto_match_rejected_at FROM bank_transactions WHERE company_id = ? AND
   import_id = ? ORDER BY id FOR UPDATE`. Le balayage doit passer par `idx_bank_transactions_import (import_id)` :
   sous REPEATABLE READ, un plan qui parcourrait l'index `company_id` verrouillerait toutes les lignes de la société
   examinées. Le dev **constate** le plan (`EXPLAIN` de la requête sur sa base de dev, sortie copiée au Dev Agent
   Record : `key = idx_bank_transactions_import`) ; si l'optimiseur en choisit un autre, `FORCE INDEX
   (idx_bank_transactions_import)`. Conséquence connue, sans cycle : quand l'import supprimé est **le plus récent
   de l'installation**, le verrou de trou sur `import_id` va jusqu'au supremum d'un index **non scopé par
   société** — toute création d'import, **toutes sociétés confondues**, attend alors la fin de la suppression ;
   sinon aucune n'attend (revue P2, F2-5).
3. Compter, **sur le résultat de cette lecture verrouillante** (pas par une seconde requête), les lignes
   rapprochées au sens de l'AC1. Si > 0 ⇒ `Ok(HasReconciled { reconciled_count, transaction_count })`, **sans
   rien écrire**.
4. `DELETE FROM bank_imports WHERE id = ? AND company_id = ?` ; `rows_affected() != 1` ⇒ `Ok(NotFound)`. Les
   transactions partent par `fk_bank_transactions_import … ON DELETE CASCADE` (pas de `DELETE` explicite,
   C-27-1-4).
5. `Ok(Deleted(..))`.

La fonction **ne commit pas** (comme `create_with_transactions`) : l'appelant y ajoute la trace d'audit et
conclut. Toutes les erreurs sqlx passent par `map_db_error`, qui garde l'interblocage 1213 en `DbError::Sqlx`
(`crates/kesh-db/src/errors.rs:1481`), ce que reconnaît `kesh_db::retry::is_deadlock_error`. Doc-comment : la
définition de l'AC1, l'ordre des verrous et sa raison, la cascade, et ce que la garde **ne** couvre **pas** (§
Angles morts) ; et une phrase : le raisonnement sur les lectures verrouillantes vaut sous **MariaDB 10.11**
(`docker-compose.yml`, CI) ; avec `innodb_snapshot_isolation` (actif par défaut depuis 11.6), une lecture
verrouillante qui rencontre une version postérieure à une vue ouverte rend `ER_CHECKREAD` (1020) — sans effet sur
le code nominal (aucune vue ouverte avant les verrous), à revoir à la montée de version (revue P2, F2-7).

**AC3 — La course avec une acceptation est fermée, et un test le prouve dans les deux sens (C-27-1-2).**

Trois tests de dépôt à deux connexions, dans un fichier neuf `crates/kesh-db/tests/bank_import_delete.rs`
(`#[sqlx::test(migrations = "./test-schema")]`), synchronisés par
`kesh_db::test_fixtures::attendre_une_requete_en_cours` (`test_fixtures.rs:561`, jamais par un `sleep`) :

- **(a) l'acceptation d'abord** : une transaction A exécute, sans commit, l'`UPDATE bank_transactions SET
  status = 'reconciled', matched_entry_id = ?, version = version + 1 WHERE id = ? AND company_id = ? AND status =
  'pending' AND version = ?` (le texte de `accept_one_invoice`, `reconciliation.rs:1917-1921`, avec une écriture
  réelle créée par `journal_entries::create`, patron de `reconciliation_cancel.rs:29-60`) ; la suppression est
  lancée dans une tâche ; on attend de la **voir en cours** sur le motif `["bank_transactions", "FOR UPDATE"]`
  — en code nominal, seule la lecture verrouillante de l'étape 2 le porte (l'étape 1 lit `bank_imports`) ; A
  commit ; la suppression rend `HasReconciled { reconciled_count: 1, .. }`, et l'import comme ses transactions
  sont **intacts**. ⚠️ Sous la mutation M3 (lecture ordinaire à l'étape 2), c'est le `DELETE FROM bank_imports`
  de l'étape 4 qui attend, et son texte ne porte pas le motif : la synchronisation **panique à 10 s**
  (`attendre_une_requete_en_cours`, `test_fixtures.rs:585-589`) — détection **déterministe** par absence du
  motif, non par l'assertion, et c'est ainsi qu'elle s'écrit au Dev Agent Record. Un motif plus large
  (`["bank_transactions"]`) serait pire : il pourrait attraper en vol le `SELECT` ordinaire muté, A commiterait
  trop tôt, et M3 survivrait (revue P1, F1).
- **(b) la suppression d'abord** : une transaction B appelle `delete_unreconciled_in_tx` et ne commit pas ; une
  tâche exécute le même `UPDATE` (version lue avant) ; on attend de la voir en cours (motif
  `["UPDATE bank_transactions"]`) ; B commit ; l'`UPDATE` rend `rows_affected() == 0`. ⚠️ Ce que les flux en font
  dépend du **moment** (revue P2, F2-1, R2-5 ; issue [#634](https://github.com/guycorbaz/kesh/issues/634)) :
  - suppression validée **après** le pré-vol des lots (`find_pending_by_ids`, sur le pool, hors verrou :
    `post_accept`, `reconciliation.rs:950-963` ; `post_reject`, `:3049-3064`) — fenêtre qui comprend l'attente du
    verrou nommé du compte, jusqu'à 5 s (`LOCK_TIMEOUT_SECS`, l. 65) — la proposition sort en `failed[]` :
    pour l'**acceptation**, sous `BANK_TRANSACTION_NOT_FOUND` (code exact) si la suppression précède la recharge de
    la transaction sous verrou (`accept_one_invoice`, `find_pending_by_id_for_account`, l. 1442-1455, et ses
    sœurs), sous `RECONCILIATION_ALREADY_RECONCILED` (`:1935-1941`, trompeur) seulement entre cette recharge et
    l'`UPDATE` ; pour le **rejet**, qui ne recharge pas (`reject_batch` lit le `tx_map` du pré-vol, l. 3188), sous
    `RECONCILIATION_ALREADY_RECONCILED` (`:3218-3225`) ; `OptimisticLockConflict` (`409`) pour le rapprochement
    manuel (`:3656-3660`) (revue P3, R3-4, F3-3) ;
  - suppression validée **avant** le pré-vol — le cas courant, une page *Réconciliation* restée ouverte : le lot
    **entier** rend `400 VALIDATION_ERROR` (« bankTransactions […] n'appartiennent pas au bankAccountId ») et
    **aucune** proposition du lot n'est traitée. Préexistant pour une transaction rapprochée entre-temps
    (`reconciliation_e2e.rs:1686-1693`), la story en ajoute une cause. **Non corrigé ici** (#634) ; dit à l'AC8,
    aux angles morts et au manuel.
  Le test (b) n'établit que le refus au niveau du dépôt (`rows_affected() == 0`).
- **(c) deux suppressions** — l'import du montage porte **au moins une transaction** (revue P2, R2-2/F2-2) : sous
  M14, la seconde suppression doit attendre sur la lecture verrouillante de ses **lignes**, tenues en X par la
  cascade de B ; avec un import vide, elle ne bloquerait qu'à l'étape 4 (`DELETE FROM bank_imports …`, sans
  `FOR UPDATE`), la sonde paniquerait au bout de 10 s, et M14 comme M14+M15 rougiraient par délai, non comme
  annoncé. B supprime sans commit ; une seconde suppression est lancée, vue en cours (motif
  **`["FOR UPDATE"]`** : il voit la seconde suppression bloquée, qu'elle attende sur l'entête — code nominal — ou,
  sous M14, sur les transactions ; B ne fait tourner aucune requête pendant l'attente) ; B commit ; la seconde
  rend `NotFound`. Ce que ce test établit : le **résultat** `NotFound`, tenu par **l'un ou l'autre** des deux
  contrôles — le `FOR UPDATE` de l'entête (étape 1) ou `rows_affected() != 1` (étape 4). Le verrou de l'entête
  n'est donc pas nécessaire à la correction ; il fixe l'ordre des verrous et réduit la fréquence des
  interblocages. D'où M14 et M15 chacune **équivalente**, et **M14 + M15 ensemble rouge** par l'assertion (la
  seconde suppression rendrait `Deleted { transaction_count: 0 }`).

**AC4 — La route `DELETE /api/v1/bank-imports/{id}` : montée avec le `POST`, fermée aux clés d'API, rejouée, inscrite au registre (C-27-1-2, C-27-1-3, C-27-1-16).**

- Handler `pub async fn delete` dans `crates/kesh-api/src/routes/bank_imports.rs` (nom aligné sur
  `bank_profiles::delete`), `Path(id): Path<i64>`, rend `Result<StatusCode, AppError>` — **`204 No Content`** en
  succès. Corps : `crate::retry::retry_app_on_deadlock("bank_imports::delete", || delete_once(&state.pool,
  &current_user, id))` — patron `complete_import` / `complete_import_once`
  (`routes/imported_supplier_invoices.rs:127-175`). `delete_once` ouvre la transaction, appelle
  `delete_unreconciled_in_tx`, mappe le résultat (AC5), écrit la trace (AC6) **dans la même transaction**, commit.
  Sur `HasReconciled` et `NotFound`, `tx.rollback().await` **avant** de rendre l'erreur (patron
  `imported_supplier_invoices.rs:315`, dans `complete_import_once`, la fonction « une tentative » dont la route
  reprend la forme) : la transaction tient alors l'entête et toutes les lignes sous `FOR UPDATE`, et un
  abandon implicite ne les rendrait qu'au retour de la connexion au pool (revue P3, F3-4).
  Un `id <= 0` rend `404 NOT_FOUND` comme un identifiant inconnu (pas de `400` : le détail `GET` ne distingue pas
  non plus).
- Montage dans **`comptable_routes`** de `crates/kesh-api/src/lib.rs`, **juste après** le `POST
  /api/v1/bank-imports` (`lib.rs:629-633`) :
  `.route("/api/v1/bank-imports/{id}", delete(routes::bank_imports::delete))`. Le `GET` du même chemin reste dans
  `authenticated_routes` (`lib.rs:922-925`) — même partage que `/api/v1/bank-profiles/{id}`. Rôle : Comptable et
  Admin (`require_comptable_role`, `middleware/rbac.rs:36-38`) ; Consultation ⇒ `403`. Le commentaire du bloc
  d'import (`lib.rs:618-621`) est complété : « Story 27-1 : suppression d'un import non rapproché (Comptable+) ».
- **Clés d'API : refusées, toutes** (décision du Project Lead du 2026-10-12, C-27-1-16 — « les api n'ont pas
  besoin de supprimer un import, du moins pour le moment »). Première instruction du handler, **avant** l'enveloppe
  de rejeu : `if current_user.api_key_id.is_some() { return Err(AppError::ApiKeyOperationForbidden); }` — contrôle
  du précédent `ensure_not_pat` (`routes/api_keys.rs:95-100`), mais avec une **variante neuve**
  `AppError::ApiKeyOperationForbidden` ⇒ `403`, code **`API_KEY_OPERATION_FORBIDDEN`**, message
  `error-api-key-operation-forbidden` (« Cette opération n'est pas accessible via une clé API. Utilisez
  l'interface web. »), déclarée à côté d'`ApiKeyAdminForbidden` (`errors.rs:204-217`) et rendue à côté
  (`errors.rs:1478-1497`). Pourquoi pas un code existant : la doctrine D2 de la 22-4a (`errors.rs:208-210`)
  interdit de réutiliser `API_KEY_MANAGEMENT_FORBIDDEN`, dont le message parle de gestion de clés, et
  `API_KEY_ADMIN_FORBIDDEN` dirait « route d'administration » pour une route Comptable. Pas de `require_not_pat`
  (la garde d'`admin_pat_denied_e2e` en interdit un second consommateur). Une clé `read` est arrêtée **avant**, par
  le middleware (`403 API_KEY_READ_ONLY`, `middleware/auth.rs:141-142`) ; une clé `read-write` atteint le handler et
  reçoit `403 API_KEY_OPERATION_FORBIDDEN` — **aucune suppression, aucune trace**. La trace d'un succès est donc
  toujours d'une session ; `for_actor(user_id, current_user.api_key_id, …)` reste la forme d'appel (l'extracteur
  de `audit_label_registry.rs` la lit), son `api_key_id` vaut toujours `None`.
- **Registre des routes** `crates/kesh-api/tests/audit_route_registry.rs` : ligne
  `("delete", "bank_imports::delete", Traced, SansEcritureAuJournal),` après `bank_imports::create` (l. 304).
  Recomptes **à recopier depuis les assertions qui rougissent**, et toutes les mentions de la valeur grepées
  (§ Propagation post-patch) : `LIB_ROUTES.len()` **114 → 115** (l. 636 et prose l. 27, 31) ; `traced` **108 →
  109** (l. 639) et sa prose d'historique (l. 640 et suivantes) prolongée de « plus la suppression d'un import
  non rapproché (27-1, #632) » ; registre total
  **117 → 118** (l. 32, l. 657, l. 986) ; `sans_ecriture` **90 → 91** et sa prose « 89 routes de lib.rs » → **90**
  (l. 981-982) ; point (vi) du doc-comment « **six** routes `SansEcritureAuJournal` sont rejouées quand même »
  (l. 126) → **sept**, avec une phrase qui nomme `bank_imports::delete` (enveloppe `AppError`, opération
  `bank_imports::delete`, raison : cycle possible avec un lot d'acceptation qui verrouille plusieurs transactions
  du même import dans l'ordre des propositions — C-27-1-2 ; tenue par sa revue, aucune preuve dynamique). Commande
  de contrôle : `grep -nE "\b(108|109|114|115|117|118|89|90|91|six|sept)\b" crates/kesh-api/tests/audit_route_registry.rs`.
  Le point (iv) du même doc-comment gagne une phrase : le lot de **rejet** (`post_reject`, non rejoué) verrouille
  désormais ses transactions par identifiant croissant (AC4-bis), ce qui ferme son cycle avec la suppression.
- **`docs/api-external.md:662`** (paragraphe *Interblocages* du § 10) : **inchangé**. Il énumère les routes rejouées
  « ouvertes aux clés `read-write` » ; la suppression ne l'est plus (C-27-1-16). Le correctif P1 (F7) est donc
  **retiré**, au motif écrit au Change Log.

**AC4-bis — Le lot de rejet verrouille par identifiant croissant (C-27-1-14).**

`post_reject` (`crates/kesh-api/src/routes/reconciliation.rs:3010-3170`) pose ses `UPDATE bank_transactions`
**dans l'ordre reçu** (`reject_batch`, boucle `for &id in ids`, l. 3187) et n'est pas rejouée (registre l. 309,
`SansEcritureAuJournal`, sans enveloppe). Un lot `[7, 3]` contre une suppression qui tient 3 et attend 7 forme un
cycle ; la victime probable est le rejet (le plus léger), qui rendrait `500`. Correctif, une ligne : à
`let ids = body.bank_transaction_ids.clone();` (l. 3079), **trier** : `let mut ids = …; ids.sort_unstable();`
(les doublons sont déjà refusés en `400` plus haut, l. 3023-3030, donc le tri ne change pas l'ensemble traité).
Avec la suppression, qui verrouille aussi ses transactions par identifiant croissant, et le rejet qui ne touche pas
`bank_imports`, plus de cycle entre les deux. Effets visibles, à écrire : `rejected[]` et `failed[]` de la réponse
sortent par identifiant croissant ; la trace `reconciliation.rejected` prend pour `entity_id` le plus petit
identifiant rejeté (aujourd'hui : le premier du lot), et son `details_json.bank_transaction_ids` sort lui aussi
trié (`success_ids`, `reconciliation.rs:3259`). Commentaires à corriger dans le même patch (revue P2, R2-4/F2-6) :
`reconciliation.rs:3257` (« entity_id = first id » → « plus petit identifiant, le lot étant trié ») et
`reconciliation_e2e.rs:1625` (« entity_id = tx_ids[0] »). L'écran n'en dépend pas
(`ReconciliationProposals.svelte:246` ne lit que `r.rejected.length`). Le lot de rejet et l'acceptation restent sérialisés entre eux
par le verrou nommé du compte (inchangé). Test **H7** (`crates/kesh-api/tests/reconciliation_e2e.rs`, à côté du test
du rejet en lot, `post_reject_marks_transactions_as_manually_reviewed`, l. 1558-1640) : rejet de `[hi, lo]` ⇒ `rejected[].bankTransactionId == [lo, hi]` et
`entity_id == lo`. Le test existant (l. 1635, « `entity_id = 1er ID du batch` », lot déjà croissant) reste vert ;
son message est corrigé en « plus petit ID du lot ».

**AC5 — Les refus : `404 NOT_FOUND`, `409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` (C-27-1-3).**

- `NotFound` ⇒ `AppError::Database(DbError::NotFound)` ⇒ `404`, code `NOT_FOUND`, message `error-not-found`
  (`errors.rs:2800-2804`) — exactement ce que rend le `GET` du détail pour un import d'une autre société
  (`bank_imports.rs:1325-1327`). Jamais `403`.
- `HasReconciled` ⇒ variante neuve **`AppError::BankImportHasReconciledTransactions { reconciled_count: i64,
  transaction_count: i64 }`** dans `crates/kesh-api/src/errors.rs`, à côté de `BankImportDuplicateFile`
  (`errors.rs:683-694`), réponse construite sur le patron de `:2163-2181` :
  ```json
  { "error": { "code": "BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS",
               "message": "<localisé, nomme reconciledCount>",
               "details": { "reconciledCount": 2, "transactionCount": 14 } } }
  ```
  Statut **`409 CONFLICT`**. Message par `t_args` (`errors.rs:51-59`) sur la clé serveur
  `bank-import-errors-has-reconciled-transactions` (AC10), argument `count`. Repli français :
  « { $count } transaction(s) de cet import sont rapprochées : annulez d'abord leur rapprochement pour pouvoir le
  supprimer. »
- Ordre : le rôle et la portée de clé (`403`, middlewares) précèdent tout ; puis le refus des clés
  `read-write` (`403 API_KEY_OPERATION_FORBIDDEN`, handler) ; puis `404` ; puis `409`. Sur chacun de ces refus :
  **aucune écriture**, ni suppression ni trace.

**AC6 — La trace `bank_import.deleted`, dans la transaction de la suppression (C-27-1-5).**

- `audit_log::insert_in_tx(&mut tx, NewAuditLogEntry::for_actor(user_id, api_key_id, "bank_import.deleted",
  "bank_imports", import.id, Some(details)))` — l'action **en littéral** à cette position (l'extracteur de
  `tests/audit_label_registry.rs` lit l'argument d'index 2 de `for_actor`), écrite **avant** le `commit`, après le
  `DELETE`.
- `details_json` (clés snake_case, comme `bank_import.created`, `bank_imports.rs:1488-1494`) :
  `filename`, `file_hash`, `source_format` (chaîne de base de `BankImportSourceFormat`), `bank_account_id`,
  `period_from`, `period_to` (`AAAA-MM-JJ`), `transaction_count` (lignes supprimées, comptées sous verrou),
  `rejected_count`.
- `crates/kesh-api/src/audit_labels.rs` : `"bank_import.deleted"` inséré dans `ACTIONS` **à sa place triée**,
  juste après `"bank_import.created"` (l. 83). `bank_imports` est déjà dans `ENTITY_TYPES` (l. 28).
- Libellé `audit-log-action-bank-import-deleted` dans les quatre catalogues (AC10). La garde
  `audit_label_registry.rs` le vérifie toute seule (diff ensembliste bilatéral) ; `audit_log_e2e.rs:531`
  (`actions.len() == ACTIONS.len()`) suit sans retouche.

**AC7 — Le même fichier se réimporte, sans avertissement de doublon (C-27-1-8).**

Après une suppression réussie, un `POST /api/v1/bank-imports` du **même fichier** (même empreinte), **sans**
`confirmDuplicateFile`, rend `201` ; son `transactionCount` égale celui du premier import ; l'aperçu `POST
…/preview` du même fichier ne porte plus `warnings.duplicateFile`. Les lignes ne sont pas non plus vues en
doublon (`find_in_dedup_window` ne trouve plus les transactions supprimées) — sauf si un **autre** import du même
compte chevauche la période, ce qui est le comportement existant et reste inchangé.

**AC8 — Ce que la suppression change ailleurs, et ce qu'elle ne change pas : vérifié et écrit.**

La fiche a vérifié au sol (Dev Notes § Inventaire (a) et § Effets) et le dev **recontrôle** par `grep` avant de
fermer la story :

- ⚠️ **La garde d'archivage d'un compte bancaire se lève** (C-27-1-12, arbitrage de l'orchestrateur) :
  `DELETE /api/v1/bank-accounts/{id}` refuse en `412 BANK_ACCOUNT_HAS_TRANSACTIONS` dès qu'une ligne de
  `bank_transactions` existe pour le compte (`bank_accounts.rs:729-751`, « comptage inconditionnel (auditabilité
  CO Art. 958f) » ; `routes/bank_accounts.rs:860-868`). Supprimer les imports non rapprochés d'un compte peut
  donc le rendre archivable — **seulement s'il ne lui reste plus aucune transaction** (une transaction rapprochée
  garde son import, et la garde compte toutes les lignes), et **sous les autres gardes de l'archivage**, dont celle
  du compte principal (`412`, `BankAccountCannotArchivePrimary`, `routes/bank_accounts.rs:870-876`). **Accepté** : des transactions jamais rapprochées ne sont pas des écritures (aucune
  pièce au journal), et la suppression laisse sa propre trace `bank_import.deleted` (fichier, empreinte, période,
  nombres) — l'auditabilité passe par la trace, non par la conservation des lignes. Test **H6** (AC11), une
  phrase au manuel (AC13), une ligne à `api-external.md` (AC14). Le doc-comment de
  `count_transactions_for_bank_account` gagne une phrase qui le dit (la garde compte les lignes **présentes** ;
  une suppression d'import non rapproché en retire, tracée). Le message du refus (`bank-accounts-errors-has-transactions`,
  4 catalogues, et son repli `errors.rs:2467-2470`) gagne **une phrase** qui nomme le remède, sans toucher au code
  d'erreur (AC10, revue P2, R2-10).

- aucune table ne référence `bank_imports` ou `bank_transactions` hors `fk_bank_transactions_import` ; aucune
  colonne d'une autre table n'y pointe sans clé (`audit_log.entity_id` est un pointeur logique documenté
  « survit aux DELETE », squash l. 97) ;
- l'**export global** (`exports/global.rs:294`, `csv_tables.rs:692-772`) et la **sauvegarde** `.keshbackup` : un
  import supprimé **avant** l'export n'y figure plus, sa trace d'audit si ; aucun code à changer, aucun rejeu
  `post_restore` concerné (pas de migration). ⚠️ **Angle mort préexistant, non corrigé ici** (C-27-1-13) :
  l'export lit ses tables une à une, en autocommit, sans instantané commun (`admin_backup/export.rs:59-62`, ordre
  `TABLES_TO_TRUNCATE` où `bank_transactions` précède `bank_imports`, `backup.rs:67-68`) — une suppression validée
  **pendant** un export peut produire une sauvegarde portant les transactions d'un import absent, que la
  restauration (sous `FOREIGN_KEY_CHECKS = 0`) recréerait orphelines. La fiche ne promet donc rien d'une
  sauvegarde prise **pendant** une suppression ; défaut suivi par
  [#633](https://github.com/guycorbaz/kesh/issues/633) (sauvegarde `.keshbackup` sans instantané commun). L'export
  global CSV (`exports/global.rs`) a le même défaut, suivi par
  [#465](https://github.com/guycorbaz/kesh/issues/465) (« L'export de souveraineté lit ses trente tables sans
  instantané commun ») — un export pris pendant une suppression peut lister des transactions d'un import absent ;
- la tuile **Relevé du …** de *Comptes bancaires* (`bank_accounts.rs:1011-1017` : parmi les imports du compte
  portant un **solde de clôture** — un CAMT.053 ; un CSV n'en porte pas —, celui de plus grande `period_to`) se
  recalcule sur **le relevé restant de plus grande période portant un solde de clôture**, ou disparaît s'il n'en
  reste aucun, même quand d'autres imports (CSV) existent — dit au manuel (AC13) ;
- les propositions de réconciliation ne montrent plus les transactions supprimées (elles lisent
  `bank_transactions`, `repositories/reconciliation.rs:178-381`) — **une fois la page rechargée** ;
- ⚠️ **effet non corrigé** (revue P2, F2-1 ; [#634](https://github.com/guycorbaz/kesh/issues/634)) : un lot
  d'acceptation ou de rejet préparé **avant** une suppression, et qui coche encore une transaction supprimée, est
  refusé **en entier** (`400 VALIDATION_ERROR`, message « … n'appartiennent pas au bankAccountId ») — aucune autre
  proposition du lot n'est traitée, contrairement à la règle « un refus n'empêche pas les autres »
  (`user-manual.tex:1878`). Remède à l'écran : recharger la page *Réconciliation*. Dit au manuel (AC13) ; le
  correctif (ranger les absents en `failed[]`) appartient à #634.

**AC9 — L'écran : l'action dans le détail d'un import (C-27-1-6).**

- Helper pur neuf **`frontend/src/lib/features/bank-import/bank-import-deletion.ts`** :
  `export function isReconciled(tx: BankTransactionDto): boolean` (= `tx.status !== 'pending' ||
  tx.matchedEntryId !== null`, doc-comment qui renvoie à `delete_unreconciled_in_tx`) et
  `export function reconciledCount(txs: BankTransactionDto[]): number`.
- Client API : `export async function deleteBankImport(id: number): Promise<void>` dans
  `bank-import.api.ts`, sur `apiClient.delete` (`shared/utils/api-client.ts:578`), chemin
  `/api/v1/bank-imports/${id}`.
- Composant neuf **`frontend/src/lib/features/bank-import/DeleteBankImportAction.svelte`**, props
  `{ detail: BankImportDetailResponse; canManage: boolean; onDeleted: (filename: string) => void }` :
  - `canManage` faux ⇒ le composant **ne rend rien** (ni bouton ni motif) — c'est ce qui rend le masquage pour
    le rôle Consultation testable (revue P1, R5) ;
  - **zéro** transaction rapprochée : bouton `data-testid="bank-import-delete-button"`, variante destructive,
    libellé `bank-import-delete-button` (« Supprimer l'import ») ;
  - **au moins une** : le même bouton, **désactivé** (`disabled`), suivi d'un texte
    `data-testid="bank-import-delete-blocked"` : `bank-import-delete-blocked`, « { $count } transaction(s) de
    cet import sont rapprochées. Pour supprimer l'import, annulez d'abord leur rapprochement. » (sans renvoyer à
    un bouton : la page ne montre *Annuler le rapprochement* que pour une ligne `reconciled`, et l'annulation
    peut elle-même être refusée — le manuel le dit, AC13) ;
  - clic ⇒ dialogue (`$lib/components/ui/dialog`, patron `CancelReconciliationDialog.svelte`)
    `data-testid="bank-import-delete-dialog"` : titre `bank-import-delete-dialog-title` (« Supprimer cet
    import ? ») ; corps `bank-import-delete-dialog-body` `data-testid="bank-import-delete-dialog-body"` :
    « Le fichier { $filename } (période du { $from } au { $to }) et ses { $count } transaction(s) seront
    supprimés. Vous pourrez ensuite réimporter le même fichier. Cette suppression ne peut pas être annulée. »
    (`count` = `detail.transactions.length`, la valeur que le serveur supprimera) ; bouton *Annuler*
    (`bank-import-labels-cancel`, clé existante) ; bouton de confirmation `data-testid="bank-import-delete-submit"`,
    `bank-import-delete-dialog-confirm` (« Supprimer définitivement »), désactivé pendant l'envoi (garde
    `if (submitting) return`) ;
  - erreurs, dans le dialogue, `data-testid="bank-import-delete-error"`, `role="alert"` : `409` de code
    `BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` ⇒ `bank-import-delete-error-reconciled` (« Une transaction de cet
    import a été rapprochée entre-temps ({ $count }) : rechargez la page. », `count` =
    `details.reconciledCount`) ; `404` ⇒ `bank-import-delete-error-not-found` (« Cet import n'existe plus : il a
    peut-être déjà été supprimé. ») ; autre ⇒ `err.message` (message serveur localisé) ;
  - succès ⇒ fermeture du dialogue, `onDeleted(detail.filename)`.
- Page `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` : sous la liste de description de l'import
  (`<dl>`, l. 79-94), `<DeleteBankImportAction {detail} {canManage} onDeleted={…} />` — `canManage` est la
  dérivée existante (l. 20-22 : Admin ou Comptable), passée au composant. `onDeleted` : `notifySuccess(i18nMsg('bank-import-delete-success',
  "Import { $filename } supprimé.", { filename }))` (`shared/utils/notify.ts:33`) puis `goto('/bank-import')`
  (`$app/navigation`). La liste *Imports précédents* se recharge à son montage.
- Rien d'autre de la page ne change (ses textes en dur restent hors périmètre).

**AC10 — Les textes, dans les quatre langues, et les gardes qui les comptent.**

**Onze** clés neuves et **une** clé modifiée, dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` :

| Clé | Lieu | Français |
|---|---|---|
| `bank-import-errors-has-reconciled-transactions` | serveur, bloc `bank-import-errors-*` (fr l. 1050-1061) | { $count } transaction(s) de cet import sont rapprochées : annulez d'abord leur rapprochement pour pouvoir le supprimer. |
| `audit-log-action-bank-import-deleted` | après `audit-log-action-bank-import-created` (fr 2364, de 2310, it 2311, en 2316) | Import bancaire supprimé |
| `error-api-key-operation-forbidden` | serveur, après `error-api-key-admin-forbidden` (l. 13 des quatre catalogues) | Cette opération n'est pas accessible via une clé API. Utilisez l'interface web. |
| `bank-import-delete-button` | frontend | Supprimer l'import |
| `bank-import-delete-blocked` | frontend | (texte de l'AC9) |
| `bank-import-delete-dialog-title` | frontend | Supprimer cet import ? |
| `bank-import-delete-dialog-body` | frontend | (texte de l'AC9) |
| `bank-import-delete-dialog-confirm` | frontend | Supprimer définitivement |
| `bank-import-delete-error-reconciled` | frontend | (texte de l'AC9) |
| `bank-import-delete-error-not-found` | frontend | (texte de l'AC9) |
| `bank-import-delete-success` | frontend | Import { $filename } supprimé. |
| `bank-accounts-errors-has-transactions` (**modifiée**, fr l. 1251) | serveur | Le compte bancaire contient des transactions — archivage refusé pour préserver l'audit comptable. Les imports dont aucune transaction n'est rapprochée peuvent être supprimés (Quotidien → Importer des relevés). |

- Le chemin de menu cité par la phrase ajoutée à `bank-accounts-errors-has-transactions` reprend, dans chaque
  langue, les libellés **réels** de `nav-quotidien` et `nav-bank-import` (fr l. 107 et 2106, de 107 et 2065, it 107
  et 2066, en 107 et 2071), jamais une traduction littérale du français (revue P3, R3-8).
- Traductions DE/IT/EN écrites par le dev, terminologie de `docs/i18n-glossaire.md` (l. 151 : *Bankimport* /
  *importazione bancaria* / *bank import* ; l. 105 : relevé = *Kontoauszug* / *estratto conto* / *statement*).
  Parité des catalogues : `parity_between_locales` (`kesh-i18n/src/loader.rs:733`).
- **Propriété des clés** : toutes les clés du composant sont préfixées `bank-import-` et le composant vit dans
  `features/bank-import/` ⇒ `npm run lint-i18n-ownership` vert sans entrée nouvelle à `KNOWN_VIOLATIONS`.
- **`frontend/src/lib/shared/i18n-keys.test.ts`, `sitesTotal`** : **2032 → 2041 attendus** (+9 sites d'appel
  `i18nMsg(`, commentaires masqués) : `DeleteBankImportAction.svelte` **+8** (bouton, motif, titre, corps,
  *Annuler*, confirmation, deux erreurs) et `routes/(app)/bank-import/[id]/+page.svelte` **+1** (succès) ;
  `sitesNonResolus` (31), `relais` (6), `sitesGabarit` (11) **inchangés** (que des littéraux). Le dev **recompte**
  aux deux bornes (`grep -o "i18nMsg(" <fichier> | wc -l`, sur `aadace40` et sur la branche) et écrit au
  commentaire de `ATTENDU` (patron des entrées 26-x, l. 541-575) le delta ventilé par fichier. Si la base a bougé
  (rebase), les valeurs qui font foi sont celles que la garde rend.

**AC11 — Les tests Rust et Vitest.**

Rust, en plus des trois tests de course de l'AC3, dans `crates/kesh-db/tests/bank_import_delete.rs` (dépôt) :

- **D1** import de trois transactions dont une rejetée ⇒ `Deleted { transaction_count: 3, rejected_count: 1 }`,
  plus de ligne `bank_imports` ni `bank_transactions` pour cet import ; l'import d'une **autre** société et un
  autre import de la même société sont intacts.
- **D2** une transaction `status = 'reconciled'` avec lien ⇒ `HasReconciled { reconciled_count: 1,
  transaction_count: 3 }`, tout intact. (D2 et D3 exigent une écriture réelle pour la clé
  `fk_bank_transactions_matched_entry` : montage de `reconciliation_cancel.rs:29-60`, `seed_accounting_company` +
  `journal_entries::create`.)
- **D3** une transaction `status = 'pending'` **avec** `matched_entry_id` posé (état qu'aucun chemin ne produit,
  posé par `UPDATE` de test : c'est la seconde branche de la disjonction) ⇒ `HasReconciled`.
- **D4** une transaction `status = 'reconciled'` **sans** lien (état laissé par un `ON DELETE SET NULL`) ⇒
  `HasReconciled` (première branche seule).
- **D5** identifiant d'une autre société ⇒ `NotFound`, l'import de l'autre société intact ; identifiant inconnu ⇒
  `NotFound`.
- **D6** import sans transaction ⇒ `Deleted { transaction_count: 0, .. }`.

Intégration HTTP, dans `crates/kesh-api/tests/bank_imports_e2e.rs` (helpers existants `setup`, `build_multipart`,
`read_fixture("v04_minimal.xml")`, `create_user`, `forge_jwt`) :

- **H1** `POST` puis `DELETE` ⇒ `204` ; `GET` du détail ⇒ `404` ; aucune transaction restante (`SELECT COUNT(*)`) ;
  une ligne `audit_log` `bank_import.deleted` / `bank_imports` / l'identifiant / l'utilisateur, `details_json`
  portant les huit clés de l'AC6 avec `transaction_count` égal au nombre importé ; **puis** `POST` du même
  fichier **sans** `confirmDuplicateFile` ⇒ `201`, même `transactionCount` (AC7).
- **H2** une transaction passée `status = 'reconciled'` **sans lien** par `UPDATE` de test (pas
  d'écriture à créer côté HTTP ; c'est aussi ce qui fait de H2 un témoin de M1) ⇒ `409`, `error.code =
  BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`, `details.reconciledCount = 1`, `details.transactionCount` = le
  nombre importé ; l'import et ses transactions intacts ; **aucune** trace `bank_import.deleted`.
- **H3** l'import de la société A supprimé par un Comptable de B ⇒ `404` `NOT_FOUND`, l'import de A intact ;
  identifiant inconnu ⇒ `404` ; identifiant `0` ⇒ `404` (revue P2, R2-12).
- **H4** rôle Consultation ⇒ `403`, import intact.
- **H5** second `DELETE` du même identifiant ⇒ `404`.
- **H6** (C-27-1-12, dans `crates/kesh-api/tests/bank_accounts_e2e.rs`, helpers de ce fichier : onboarding
  complet, `version` du compte) : un compte portant un import non vide ⇒ archivage `412
  BANK_ACCOUNT_HAS_TRANSACTIONS` ; suppression de l'import ⇒ `204` ; archivage ⇒ `200`. Le test fige le
  comportement accepté, dans les deux sens.
- **H7** (AC4-bis, `reconciliation_e2e.rs`) : le lot de rejet sort et trace par identifiant croissant.
- **H6** asserte aussi le **message** du `412` : il contient la phrase de remède (installation en `fr-CH` dans le
  montage de test) (revue P3, R3-7).
- **H9** (`bank_accounts_e2e.rs`, montage de `list_bank_accounts_exposes_the_last_statement`, l. 1206) : deux relevés
  CAMT.053 du même compte ; suppression du plus récent ⇒ la tuile (`GET /bank-accounts`) expose le solde et la date
  du **restant** ; suppression du dernier ⇒ plus de relevé exposé (revue P3, R3-7).
- **Test de message** (`kesh-i18n/src/loader.rs`, à côté de
  `admin_forbidden_message_is_translated_and_distinct_from_key_management`, l. 663 et suivantes) : `error-api-key-operation-forbidden`
  présent dans les quatre locales et **distinct** de `error-api-key-management-forbidden` et
  `error-api-key-admin-forbidden` (revue P3, R3-9/F3-6) — le patron dit pourquoi : un calque qui copierait le
  message voisin satisferait toutes les autres clauses.
- **H8** (C-27-1-16) : une clé `read-write` créée par l'Admin (`POST /api/v1/settings/api-keys`, patron
  `fiscal_years_e2e.rs:2011-2030`) ⇒ `DELETE` en `403`, `error.code = API_KEY_OPERATION_FORBIDDEN` ; l'import et ses
  transactions intacts ; **aucune** trace `bank_import.deleted`. Une clé `read` ⇒ `403 API_KEY_READ_ONLY` (même
  vérification d'intégrité).
- Registre des routes (AC4) et garde des libellés d'audit (AC6) verts.

Vitest :

- `bank-import-deletion.test.ts` : `isReconciled` vrai pour `reconciled` sans lien, vrai pour `pending` avec lien,
  faux pour `pending` sans lien (rejetée ou non) ; `reconciledCount` sur une liste mixte.
- `bank-import.api.test.ts` (existant) : `deleteBankImport(42)` appelle `DELETE /api/v1/bank-imports/42`.
- `DeleteBankImportAction.test.ts` (patron `CancelReconciliationDialog.test.ts` : `vi.mock` de l'API et
  d'`i18nMsg` avec interpolation) : `canManage: false` ⇒ ni bouton ni motif ; bouton actif sans rapprochement ; bouton désactivé et motif `{ count }` avec
  deux rapprochées ; le corps du dialogue nomme fichier, période et nombre ; *Supprimer définitivement* appelle
  `deleteBankImport` avec **l'identifiant de l'import** puis `onDeleted(filename)` ; double clic ⇒ un seul appel ;
  `409` ⇒ message `error-reconciled` avec `details.reconciledCount`, `onDeleted` non appelé ; `404` ⇒ message
  `error-not-found`.

**AC12 — Un E2E Playwright : importer, supprimer, réimporter (C-27-1-10).**

Spec neuve `frontend/tests/e2e/bank-import-delete.spec.ts` (patron `bank-csv-import.spec.ts` : `seedTestState
('with-company')` en `beforeAll`, `login`, `ensureBankAccount`, `authedApiContext`), fixture neuve
`frontend/tests/e2e/fixtures/csv_e2e_delete_import.csv` — **synthétique**, rien d'un relevé réel :

```
date;amount;reference;details
2026-03-02;120.00;E2E-DEL-1;Versement test suppression
2026-03-03;-45.50;E2E-DEL-2;Achat test suppression
2026-03-04;10.00;E2E-DEL-3;Remboursement test suppression
```

Un scénario :

1. profil CSV créé **par l'API** (`POST /api/v1/bank-profiles`, nom `E2E suppression`, motif
   `^csv_e2e_delete_import\.csv$`, `%Y-%m-%d`, `;`, `.`, une ligne d'en-tête, colonnes date 0 / montant 1 /
   référence 2 / détails 3, `debitSign: "positive"`) — corps à recopier du formulaire réel, `debitSign` est exigé
   depuis la v0.13.1 ;
2. import par l'écran (`bank-account-select`, `bank-import-file-input`, `bank-import-confirm`) ; la ligne apparaît
   dans `bank-import-list-row` ; clic sur le nom du fichier ⇒ détail ;
3. `bank-import-delete-button` ⇒ dialogue ; `bank-import-delete-dialog-body` contient `csv_e2e_delete_import.csv`
   et `3` ; `bank-import-delete-submit` ⇒ URL `/bank-import`, aucune ligne de la liste ne porte ce nom de
   fichier ;
4. même fichier redéposé ⇒ l'aperçu **n'affiche pas** `warning-duplicate-file` ; `bank-import-confirm` ⇒ la ligne
   réapparaît, et le détail compte trois transactions.

Gate : la suite E2E **complète** au dernier commit de code, jugée fichier par fichier contre les échecs attendus
(`docs/testing.md` § *Les échecs attendus*).

**AC13 — Les manuels (utilisateur et administration) disent ce que le code garantit, PDF compris.**

`docs/manual/fr/user-manual.tex` :

- **Sous-section neuve** `\subsubsection{Supprimer un import}\label{sec:supprimer-import}`, placée **après**
  `\subsubsection{Import dans Kesh}` (l. 1690-1719), avant `\subsection{Import CSV multi-encodage}` (l. 1721).
  Elle dit, et ne dit que : où (*Quotidien* → *Importer des relevés* → *Imports précédents* → nom du fichier,
  bouton *Supprimer l'import*) ; qui (Comptable et Admin ; le rôle Consultation ne voit pas le bouton) ; quand
  (aucune transaction de l'import n'est rapprochée — une transaction **rejetée** ne compte pas ; sinon le bouton
  est grisé, avec le nombre, et il faut d'abord annuler leur rapprochement, renvoi
  `\S\ref{sec:annuler-rapprochement}`) ; ce qui part (l'import et **toutes** ses transactions, rejetées
  comprises ; rien au grand livre, puisqu'aucune n'est rapprochée) ; ce qui reste (une trace *Import bancaire
  supprimé* au journal d'audit, avec le fichier, la période et le nombre de transactions) ; ce qui suit (le même
  fichier se réimporte sans l'avertissement « fichier déjà importé » ; la tuile *Relevé du …* de *Comptes
  bancaires* reprend le relevé restant de plus grande période **portant un solde de clôture** (un CAMT.053), ou
  disparaît s'il n'en reste aucun, renvoi `\S\ref{sec:solde-comptable}`) ; l'irréversibilité ; **en une phrase**,
  qu'un compte bancaire qui porte des transactions importées ne s'archive pas, et que supprimer ses imports peut le
  rendre archivable **s'il ne lui reste aucune transaction**, sous les autres conditions de l'archivage (dont celle
  du compte principal) (C-27-1-12) ; et que l'annulation d'un rapprochement peut elle-même être refusée : **tant
  qu'elle l'est, l'import ne peut pas être supprimé** — les remèdes sont ceux que le
  `\S\ref{sec:annuler-rapprochement}` donne pour chaque motif (l. 1964-1983 : rouvrir des exercices, faire reculer
  le verrou de période, réactiver un compte, créer l'exercice du jour), et la phrase y renvoie sans les recopier ;
  un seul motif n'en a pas, la facture créditée par un avoir (cas hérité d'avant la 0.12.1) ; et qu'un lot de
  réconciliation préparé **avant** la suppression est refusé en entier : recharger la page *Réconciliation*
  (#634).
- **Renvoi** dans le § *Comptes bancaires* (l. 1643, « on y crée, modifie et archive un compte ») vers
  `\S\ref{sec:supprimer-import}` pour l'archivage d'un compte qui porte des transactions (revue P2, R2-11).
- **Correction** du paragraphe *Rejeter* (l. 1873) : « aucun geste de l'interface ne permet ensuite de la passer
  en comptabilité » est faux après cette story. Le remplacer par : aucun geste **de l'interface** ne la remet
  dans la liste de la réconciliation (par l'API, `POST /reconciliation/manual` et `/split` l'acceptent encore,
  `find_strictly_pending_by_id_for_account` ne regardant que `status`, `reconciliation.rs:2300, 2701, 3477` ;
  `POST /reconciliation/accept` la refuse au pré-vol, qui filtre le rejet — ne pas écrire « aucun geste » tout
  court ; revue P3, R3-3) ; **sauf** à supprimer son import — si aucune transaction de cet import n'est rapprochée — et à
  le réimporter (`\S\ref{sec:supprimer-import}`). Greper le symptôme (`aucun geste`, `sans aucune action`) dans
  tout `docs/` et `website/`.
- **PDF régénéré** : `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock make -C docs/manual user` puis commit du
  `.pdf` ; contrôle sur le PDF **aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '
  | grep -o "Supprimer un import.\{0,200\}"`.
- Manuels DE/EN/IT : **non tenus** (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`, et le `Makefile` n'a
  que `LANGS_AVAILABLE := fr`) — rien à écrire. Brochure : non concernée.
- **Manuel d'administration** (revue P3, R3-2/F3-2 — « non concerné » était faux depuis C-27-1-16) :
  `docs/manual/fr/admin-manual.tex`, § *Clés API (PAT)* (l. 2262-2285) : une puce neuve après *Journal d'audit*
  (l. 2269) — la **suppression d'un import bancaire** (`DELETE /api/v1/bank-imports/\{id\}`) est fermée aux clés,
  `403 API\_KEY\_OPERATION\_FORBIDDEN` pour une clé `read-write` (une clé `read` est arrêtée avant, en
  `API\_KEY\_READ\_ONLY`) ; elle se fait dans l'interface web, par un Comptable ou un Administrateur ; et la puce
  *Portée* (l. 2272) complétée : « …et l'administration en est exclue dans tous les cas, de même que le journal
  d'audit **et la suppression d'un import bancaire** (points précédents) ». La phrase d'ouverture (l. 2264, « Une
  clé s'utilise sur les routes `/api/v1/*` existantes ») reste vraie avec ces exceptions énumérées juste après.
  PDF régénéré (`flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock make -C docs/manual admin`) et contrôlé
  aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "suppression d'un
  import bancaire.\{0,120\}"`).
- Manuel utilisateur, § *Supprimer un import* : une phrase — la suppression se fait à l'écran ; une clé d'API ne
  peut pas supprimer un import (renvoi au manuel d'administration, § *Clés API*).
- Manuel utilisateur, § *Solde comptable, relevé et écart* (l. 250-251) : « le solde de clôture du **dernier
  relevé** importé » devient « du relevé CAMT.053 **de plus grande période** » — le code choisit par `period_to`,
  pas par date d'import (`bank_accounts.rs:1011-1017`), et la section neuve y renvoie (revue P3, R3-11).

**AC14 — API externe, CHANGELOG, README (C-27-1-11).**

- `docs/api-external.md` (C-27-1-16 : la route **n'est pas ouverte aux clés**) :
  - § 4 : un **paragraphe à part** (encadré `> ⚠️`, sur le modèle de celui du journal d'audit, l. 83), **pas** un
    ajout à la liste de l'encadré d'administration (l. 79-81, « ces opérations [d'administration] ») : la route est
    Comptable, non d'administration. Il dit : la **suppression d'un import bancaire** (`DELETE /bank-imports/{id}`)
    refuse toute clé — `403 API_KEY_OPERATION_FORBIDDEN` pour une clé `read-write`, `403 API_KEY_READ_ONLY` pour une
    clé `read` (arrêtée avant) — et se fait dans l'interface web ; *depuis la v0.13.2* ;
  - § 7, phrase d'introduction (l. 205) : « **à deux exceptions près, toutes deux fermées aux clés API** : les
    routes d'administration, et la consultation du journal d'audit […] » devient **trois** exceptions — les routes
    d'administration, la consultation du journal d'audit et la suppression d'un import bancaire —, avec leurs trois
    codes — § 4 pour la fermeture (administration, journal d'audit et suppression), § 10 pour le code d'erreur
    (revue P3, R3-2/F3-2 ; revue P4, P4-2 : la fermeture du journal s'énonce au § 4, l. 83, le § 10 n'en porte
    que le code, l. 647) ;
  - § 7, ligne neuve du tableau (l. 207-218) — *Imports bancaires* | `GET /bank-imports`, `GET /bank-imports/{id}`
    | `POST /bank-imports` ⁸ ; note ⁸ : la suppression d'un import existe à l'interface web et n'est **pas**
    ouverte aux clés (§ 4) ; un import supprimé disparaît des lectures, et une fois **tous** ses imports supprimés
    — s'il ne lui reste aucune transaction —, un compte bancaire peut s'archiver par `DELETE /bank-accounts/{id}`,
    sous ses autres conditions (C-27-1-12) ;
  - § 10, tableau (l. 643 et suivantes) : ligne `403 API_KEY_OPERATION_FORBIDDEN` (« opération réservée à
    l'interface web ; aujourd'hui, la suppression d'un import bancaire »). Pas de ligne pour le `409`, que les clés
    n'atteignent pas ;
  - le paragraphe *Interblocages* (l. 662) reste inchangé (AC4) ; la ligne du rejet (AC4-bis) va sous *Accepter
    des propositions de rapprochement* (l. 529) ou à côté : `POST /reconciliation/reject` traite et rend ses
    transactions par identifiant croissant, *depuis la v0.13.2*.
  Mentions « *depuis la v0.13.2* ». ⛔ Ni « Unreleased » (garde G6, `textes_coherents.rs:619-640`), ni « à venir »
  (règle de documentation du `CLAUDE.md` : ne décrire que ce qui est livré — aucune garde ne tient ce second
  interdit, revue P2, R2-6).
- `CHANGELOG.md` : section **`## [0.13.2] — Non publié`** créée au-dessus de `## [0.13.1] — 2026-10-11`
  (l. 11), forme exacte attendue par `scripts/prepare-release.sh:237` ; `### Ajouté` : une entrée « **Supprimer un
  import bancaire dont aucune transaction n'est rapprochée
  ([#632](https://github.com/guycorbaz/kesh/issues/632)).** … » qui dit où, la condition, le réimport, la trace,
  et la route (`DELETE /api/v1/bank-imports/{id}`, réservée à l'interface web : refusée aux clés d'API en `403
  API_KEY_OPERATION_FORBIDDEN`) ; sous
  `### Modifié`, une ligne : `POST /api/v1/reconciliation/reject` traite et rend ses transactions par identifiant
  croissant (AC4-bis).
- `README.md` § *Fonctionnalités*, ligne *Import bancaire* (l. 41) : ajouter « **suppression d'un import non
  rapproché**, pour le réimporter ✓ ». La ligne v0.13.2 de la feuille de route se pose à la release
  (orchestrateur). Toute référence `#N` du README est un lien (garde G7).
- `website/` : non concerné (aucune page ne détaille l'import bancaire à ce grain — à recontrôler par `grep -rn
  "import" website/*.html`).

**AC15 — Aucune migration (C-27-1-8).**

Le diff ne touche ni `crates/kesh-db/migrations/` ni `crates/kesh-db/test-schema/`. Une migration au diff serait
une dérive à justifier contre P1-P8 du `CLAUDE.md`. Pas de bump `min_required`, pas de bump de version Cargo par
la story (la version 0.13.2 se pose à la release).

**AC16 — Chaque garde a un test qui rougit : la campagne de mutations.**

Jouées au dev, chacune **restaurée** puis le fichier **touché** (`touch`, cf. mémoire « mutation restaurée,
binaire périmé »), résultat consigné au Dev Agent Record (test qui rougit, ou « équivalente » justifiée) :

| # | Mutation | Test attendu rouge |
|---|---|---|
| M1 | prédicat Rust sans la branche `status <> 'pending'` | D4, H2 (D2 reste vert : son lien suffit) |
| M2 | prédicat Rust sans la branche `matched_entry_id IS NOT NULL` | D3 |
| M3 | lecture des transactions **sans** `FOR UPDATE` | AC3 (a) — par **absence du motif** (panique à 10 s, déterministe), non par l'assertion |
| M4 | `SELECT` de l'entête sans `AND company_id = ?` (et `DELETE` idem) | D5, H3 |
| M5 | trace d'audit retirée | H1 |
| M6 | ~~trace écrite avant la garde~~ — **retirée** : sur refus, la transaction n'est pas validée et la trace part au `rollback` ; la mutation est équivalente par construction (revue P1, R2/F9) | — |
| M7 | `409` remplacé par `422` | H2 |
| M8 | route montée dans `authenticated_routes` | H4 |
| M9 | `"bank_import.deleted"` retiré d'`ACTIONS` | `audit_label_registry` |
| M10 | ligne du registre des routes retirée | `audit_route_registry` |
| M11 | `isReconciled` sans la branche `matchedEntryId` | `bank-import-deletion.test.ts` |
| M12 | le dialogue appelle `deleteBankImport` avec un autre identifiant | `DeleteBankImportAction.test.ts` |
| M13 | garde `submitting` retirée | `DeleteBankImportAction.test.ts` (double clic) |
| M14 | `FOR UPDATE` de l'entête retiré | **équivalente** (AC3 (c) vert : `rows_affected() != 1` rend `NotFound`) — à constater |
| M15 | contrôle `rows_affected() != 1` retiré | **équivalente** (AC3 (c) vert : l'étape 1 sous verrou rend `NotFound`) — à constater |
| M14+M15 | les deux ensemble | AC3 (c), par l'assertion (`Deleted { transaction_count: 0 }` au lieu de `NotFound`) |
| M16 | `canManage` ignoré par le composant | `DeleteBankImportAction.test.ts` (`canManage: false`) |
| M17 | tri de `post_reject` retiré | H7 |
| M18 | refus des clés d'API retiré du handler | H8 |
| M19 | message `error-api-key-operation-forbidden` copié de `error-api-key-admin-forbidden` (une locale) | test de message de `loader.rs` |
| M20 | phrase de remède retirée du message du `412` (`fr-CH`) | H6 |

## Tasks / Subtasks

- [x] **T0 — Montage** (consignes communes des agents) : base `kesh_27_1` et `kesh_e2e_27_1` créées par l'agent
  (migrations du worktree, seed `scripts/seed-dev-db.sql`), `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-27-1/target`,
  `node_modules` par `cp -al` depuis le dépôt principal, port attribué par l'orchestrateur ; gates lourds sous
  `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`.
- [x] **T1 — Dépôt** (AC1, AC2)
  - [x] T1.1 `DeleteBankImportOutcome`, `DeletedBankImport`, `delete_unreconciled_in_tx` dans
    `crates/kesh-db/src/repositories/bank_imports.rs`, doc-comments (définition, ordre des verrous, cascade, angles
    morts) ; corriger au passage le doc-comment de module (l. 10-12), faux deux fois : il annonce la contrainte
    UNIQUE retirée en 8-3, et un refus `409` qui est un `422`.
  - [x] T1.2 Tests D1-D6 et AC3 (a)(b)(c) dans `crates/kesh-db/tests/bank_import_delete.rs`.
- [x] **T2 — Route** (AC4, AC5, AC6)
  - [x] T2.1 `AppError::BankImportHasReconciledTransactions` et sa réponse dans `errors.rs`.
  - [x] T2.2 `delete` (refus des clés d'API en tête, C-27-1-16) + `delete_once` dans `routes/bank_imports.rs`
    (doc-comment de module l. 1-11 complété) ; `AppError::ApiKeyOperationForbidden` et sa réponse dans `errors.rs`.
  - [x] T2.3 Montage `lib.rs` (comptable_routes, après l. 633).
  - [x] T2.4 `audit_labels.rs` (`ACTIONS`).
  - [x] T2.5 `audit_route_registry.rs` : ligne et recomptes (AC4).
  - [x] T2.6 Tests H1-H6 et H8 (H6 dans `bank_accounts_e2e.rs`) ; doc-comment de `count_transactions_for_bank_account`
    et phrase du message `bank-accounts-errors-has-transactions` (repli `errors.rs:2467-2470`).
  - [x] T2.7 AC4-bis : tri de `post_reject`, test H7, message du test existant, point (iv) du registre,
    `api-external.md:662`.
- [x] **T3 — i18n serveur et frontend** (AC10) : onze clés neuves et une modifiée × quatre catalogues.
- [x] **T4 — Écran** (AC9)
  - [x] T4.1 `bank-import-deletion.ts` + tests.
  - [x] T4.2 `deleteBankImport` + test.
  - [x] T4.3 `DeleteBankImportAction.svelte` + tests.
  - [x] T4.4 Page détail : insertion, notification, retour à la liste.
  - [x] T4.5 `sitesTotal` recompté (AC10), `lint-i18n-ownership`.
- [x] **T5 — E2E** (AC12) : fixture, spec.
- [x] **T6 — Documentation** (AC13, AC14) : manuels utilisateur **et d'administration**, `.tex` + PDF (sous
  `flock`), `api-external.md` (§ 4, § 7 l. 205, § 10), `CHANGELOG.md`, `README.md` ; symptômes grepés dans tout le
  dépôt (paragraphe *Rejeter* ; `grep -rn "fermées aux clés\|exceptions près\|en est exclue\|de même que le journal"
  docs/ crates/kesh-api/src frontend/src` — chaque phrase qui énumère les fermetures aux clés).
- [x] **T7 — Mutations** (AC16), sous `flock`.
- [x] **T8 — Gates au dernier commit de code** : base de gate remise à zéro **sans toucher au conteneur**
  (⛔ pas de `docker restart` de `kesh-mariadb-dev` : recréer `kesh_27_1` par `DROP/CREATE DATABASE`, migrations,
  seed) ; `scripts/test-fast.sh` complet ; frontend `npm run check`, `npm run lint-i18n-ownership`,
  `npm run test:unit`, `npm run build` ; E2E complet. N'écrire au Dev Agent Record **que ce qui a tourné**, avec
  les chiffres. AC8 et AC15 recontrôlés par `git diff --stat aadace40` et les `grep` de l'AC8.

## Dev Notes

### Inventaire (a) — ce qui référence `bank_imports` et `bank_transactions`, au sol

Clés étrangères, migrations **et** squash de test :

```
$ grep -rn -iE "references\s+\`?bank_(transactions|imports)" crates/kesh-db/
crates/kesh-db/migrations/20260504000001_bank_imports.sql:79:        FOREIGN KEY (import_id) REFERENCES bank_imports(id) ON DELETE CASCADE,
crates/kesh-db/test-schema/0001_schema_squash.sql:232:  CONSTRAINT `fk_bank_transactions_import` FOREIGN KEY (`import_id`) REFERENCES `bank_imports` (`id`) ON DELETE CASCADE,
```

**Une seule** clé étrangère vers ces deux tables, et c'est celle des transactions vers leur import. Aucune table ne
référence `bank_transactions`. Colonnes sans clé : recherche large des noms de colonnes dans les migrations et le
squash —

```
$ grep -rln -i "bank_transaction_id\|bank_import_id\|import_id" crates/kesh-db/migrations/ crates/kesh-db/test-schema/
crates/kesh-db/migrations/20260504000001_bank_imports.sql
crates/kesh-db/test-schema/0001_schema_squash.sql
$ grep -rln -i "bank_transaction\|bank_import" crates/kesh-db/migrations/
crates/kesh-db/migrations/20260505000001_bank_profiles.sql
crates/kesh-db/migrations/20260504000001_bank_imports.sql
crates/kesh-db/migrations/20260507000001_bank_imports_relax_hash_unique.sql
crates/kesh-db/migrations/20260507100001_reconciliation_8_4.sql
```

`import_id` n'existe que dans `bank_transactions`. `bank_profiles.sql` ne nomme les tables bancaires qu'en
commentaire ; `relax_hash_unique` retire l'unicité (C-27-1-8) ; `reconciliation_8_4` ajoute
`auto_match_rejected_at`. Les références **logiques** (sans clé) passent par :

- `audit_log.entity_id` avec `entity_type = 'bank_imports'` (création, et désormais suppression) et
  `'bank_transaction'` (réconciliation) — pointeur logique, « survit aux DELETE » (squash l. 97) ;
- `bank_transactions.matched_entry_id → journal_entries` (`ON DELETE SET NULL`, squash l. 233) — la seule
  référence **sortante** qui compte pour la garde.

Lecteurs de `bank_transactions` hors import (non touchés, et sans effet puisqu'on ne supprime que des
transactions non rapprochées) : `settlement_cancellation.rs:141` et `invoices.rs:1608` (cherchent une transaction
qui pointe une écriture — seules les rapprochées ont un lien) ; `journal_entries.rs:2274` (propriétaires de
documents, idem) ; `bank_accounts.rs:983,995` (filtrent `status = 'reconciled'`) ; `bank_accounts.rs:1011-1017`
(relevé de plus grande période portant un solde de clôture, lit `bank_imports` — AC8) ; `exports/global.rs:294` et `exports/csv_tables.rs:692-772` (export) ;
`repositories/reconciliation.rs:178,301,347,379` (propositions, `pending`).

⚠️ **Un lecteur manquait à la première rédaction (revue P1, R3/F6)** : la garde d'archivage d'un compte bancaire,
`bank_accounts::count_transactions_for_bank_account` (`repositories/bank_accounts.rs:729-751`), appelée par
`archive_bank_account` (`routes/bank_accounts.rs:860-868`) — elle compte **toutes** les lignes du compte, et la
suppression d'un import en retire. Effet accepté et testé (AC8, C-27-1-12, H6). L'inventaire d'origine ne
cherchait que les clés et les colonnes ; un lecteur qui **compte** n'en a pas.

### Inventaire (b) — tout ce qui écrit `bank_transactions` (et donc définit « rapprochée »), au sol

```
$ grep -rn -iE "(UPDATE|INSERT INTO|DELETE FROM|REPLACE INTO)\s+\`?bank_(transactions|imports)" crates/ --include=*.rs | grep -v "/tests/"
crates/kesh-api/src/routes/reconciliation.rs:1918:        "UPDATE bank_transactions \
crates/kesh-api/src/routes/reconciliation.rs:2417:        "UPDATE bank_transactions \
crates/kesh-api/src/routes/reconciliation.rs:2869:        "UPDATE bank_transactions \
crates/kesh-api/src/routes/reconciliation.rs:3210:            "UPDATE bank_transactions \
crates/kesh-api/src/routes/reconciliation.rs:3634:                "UPDATE bank_transactions \
crates/kesh-api/src/routes/reconciliation.rs:4091:                "UPDATE bank_transactions \
crates/kesh-db/src/repositories/reconciliation_cancel.rs:339:        "UPDATE bank_transactions \
crates/kesh-db/src/repositories/invoices.rs:4362:            "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
crates/kesh-db/src/repositories/invoices.rs:4376:            "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, \
crates/kesh-db/src/repositories/invoices.rs:4852:        sqlx::query("DELETE FROM bank_transactions WHERE id = ?")
crates/kesh-db/src/repositories/invoices.rs:4857:        sqlx::query("DELETE FROM bank_imports WHERE id = ?")
crates/kesh-db/src/repositories/bank_imports.rs:56:        "INSERT INTO bank_imports \
crates/kesh-db/src/repositories/bank_imports.rs:100:            "INSERT INTO bank_transactions \
```

(Six lignes de **commentaire** de `reconciliation.rs` que la commande rend aussi — l. 1615, 2414, 2867, 3351, 3630,
4088, « Step … — UPDATE bank_transactions … » — sont retirées de la transcription ; revue P2, R2-8. Les quatre lignes
de `invoices.rs` au-delà de la l. 2984 sont dans son `mod tests` — `#[cfg(test)]` l. 2984-2985.)
Et le relevé des affectations de colonnes, qui confirme la table :

| Site (fonction) | Écrit | Condition |
|---|---|---|
| `reconciliation.rs:1918` — `accept_one_invoice` (accepter une facture, lot `accept`) | `status='reconciled'`, `matched_entry_id` | `status='pending' AND version=?` |
| `reconciliation.rs:2417` — `accept_one_split` (accepter un éclatement) | idem + `auto_match_rejected_at=NULL` | idem |
| `reconciliation.rs:2869` — `accept_one_rule` (accepter une règle) | idem | idem |
| `reconciliation.rs:3210` — `reject_batch` (rejeter) | `auto_match_rejected_at=NOW(3)` **seulement** | `status='pending'` |
| `reconciliation.rs:3634` — `post_manual_once` (rapprochement manuel) | `status='reconciled'`, `matched_entry_id`, rejet remis à `NULL` | `status='pending' AND version=?` |
| `reconciliation.rs:4091` — `post_split_once` (éclatement direct) | idem | idem |
| `reconciliation_cancel.rs:339` — `cancel_in_tx` (annuler un rapprochement) | `status='pending'`, `matched_entry_id=NULL`, rejet `NULL` | `status='reconciled' AND version=?` |
| `bank_imports.rs:56,100` — `create_with_transactions` | insertion, valeurs par défaut (`pending`, `NULL`) | — |

Les gestes « lettrage », « règlement de facture » et « paiement fournisseur » **n'écrivent pas**
`bank_transactions` : ils lisent `matched_entry_id` pour refuser (`settlement_cancellation.rs:141`,
`invoices.rs:1608`) — vérifié par le `grep` ci-dessus, qui ne rend aucun autre écrivain. **Effacements totaux**,
hors de la définition de « rapprochée » mais écrivains tout de même (revue P1, F11) : la restauration
d'installation (`restore_tables_in_tx`, `kesh-db/src/backup.rs:444-568`, qui vide `bank_transactions` puis
`bank_imports` sous `FOREIGN_KEY_CHECKS = 0`), la remise à zéro de la démonstration (`kesh-seed::reset_demo`, même
liste `TABLES_TO_TRUNCATE`, `backup.rs:111-120`) et `test_fixtures::truncate_all` (`test_endpoints.rs:172, 306`).
⚠️ La restauration n'est **pas** « exclusive des autres écrivains » : la Story 15-12b a restreint la formule à
« exclusive des **autres imports** seulement » (verrou `_kesh_version … FOR UPDATE`, `routes/admin.rs:239`) —
elle peut donc croiser une suppression (angle mort ci-dessous, revue P1, F4). Aucun déclencheur SQL dans les migrations
(`grep -rln -i "CREATE TRIGGER" crates/kesh-db/migrations/` : vide).

**Ce que fait réellement le rejet** — au manuel (`user-manual.tex:1873`) : « la transaction est marquée comme
revue et **quitte la liste** de la réconciliation. Elle reste visible dans le **détail de son import** […] avec le
statut « pending », mais **sans aucune action** » ; au code : `reject_batch` ne pose que `auto_match_rejected_at`
(`reconciliation.rs:3209-3212`), et `find_pending_transactions_for_account` exclut ces lignes
(`repositories/reconciliation.rs:301-303`). Le manuel et le code concordent ; la phrase « aucun geste de
l'interface ne permet ensuite de la passer en comptabilité » devient fausse avec cette story (AC13).

**D'où la définition fermée** de l'AC1 : les cinq chemins qui rapprochent posent les deux colonnes ensemble, le
seul qui les retire les retire ensemble, le rejet ne touche aucune des deux. La disjonction couvre en plus deux
états qu'aucun chemin vivant ne produit (`reconciled` sans lien, après un `ON DELETE SET NULL` antérieur à la garde
de `journal_entries::delete`, `journal_entries.rs:1775-1780` ; `pending` avec lien), testés par D3 et D4.

### Concurrence — la doctrine du dépôt, appliquée ici

- **Ce qui protège réellement les acceptations**, ce n'est pas le verrou nommé : `with_account_lock`
  (`kesh-reconciliation/src/mutex.rs:62-140`) relâche `GET_LOCK` **avant** le commit de l'appelant
  (`accept_once`, `reconciliation.rs:1037-1070`). C'est l'`UPDATE … WHERE status = 'pending' AND version = ?`,
  dont le verrou de ligne vit jusqu'au commit et dont `rows_affected()` est contrôlé. Les lectures des flux
  (`find_pending_by_id_for_account`, `repositories/reconciliation.rs:337-349`) sont **ordinaires** (pas de
  `FOR UPDATE`).
- Donc : si l'acceptation a écrit (verrou X tenu) avant la suppression, la lecture verrouillante de la
  suppression **attend** son commit, puis lit la version **validée la plus récente** (propriété d'une lecture
  verrouillante InnoDB, même en REPEATABLE READ) et refuse — AC3 (a). Si la suppression a verrouillé d'abord,
  l'`UPDATE` de l'acceptation attend, puis ne trouve plus la ligne : `rows_affected() == 0` → refus déjà codé —
  AC3 (b).
- **Pourquoi un rejeu** (C54 de l'Epic 15 ; `kesh-api/src/retry.rs`) : `accept_batch` traite les propositions
  **dans l'ordre reçu**, chacune dans un savepoint, en gardant les verrous des précédentes ; la suppression
  balaie les transactions **par identifiant**. Un lot qui tient la transaction 7 puis demande la 3, contre une
  suppression qui tient la 3 et attend la 7, forme un cycle ; InnoDB en tue un (1213). L'enveloppe rejoue la
  suppression victime (l'acceptation victime l'est déjà, `post_accept`). ⚠️ **Non prouvé dynamiquement** — angle
  mort (iii bis) du registre ; la fiche n'exige pas de test de 1213 pour cette route.
- La suppression ne prend **aucun** autre verrou que ses lignes et l'insertion d'audit : pas de `companies`, pas
  d'exercice, pas de compte. Mais elle n'est pas seule à verrouiller **plusieurs** lignes d'un même import (revue
  P1, F3) : inventaire des verrouilleurs multi-lignes de `bank_transactions` — `accept_batch` (ordre des
  propositions, **rejoué**), `reject_batch` (ordre reçu, **non rejoué** : une 1213 y rendrait `500`) ; `manual`,
  `split`, `cancel` sont unitaires ; la restauration est un effacement total. D'où l'AC4-bis : `reject_batch`
  verrouille désormais par identifiant croissant, comme la suppression, ce qui ferme leur cycle ; le cycle avec
  `accept_batch` reste, défendu par le rejeu des deux côtés.

### Angles morts assumés (à écrire au doc-comment de `delete_unreconciled_in_tx`)

- **Un import créé pendant la suppression** n'est pas concerné (autre identifiant) ; **deux imports du même
  fichier** (créés avec `confirmDuplicateFile`) se suppriment un par un — la story ne supprime pas « le fichier »,
  elle supprime **un** import.
- **Un lot d'acceptation ou de rejet** contre la suppression (plus de cycle avec le rejet depuis l'AC4-bis) —
  deux trajectoires (revue P2, F2-1, R2-5, [#634](https://github.com/guycorbaz/kesh/issues/634)) : suppression
  validée **avant** le pré-vol `find_pending_by_ids` ⇒ le lot **entier** en `400 VALIDATION_ERROR` (« …
  n'appartiennent pas au bankAccountId »), aucune proposition traitée ; validée **après** le pré-vol ⇒ la ligne
  sort en `failed[]` — pour l'acceptation sous `BANK_TRANSACTION_NOT_FOUND` puis, dans la fenêtre plus étroite qui
  suit la recharge sous verrou, sous `RECONCILIATION_ALREADY_RECONCILED` ; pour le rejet sous
  `RECONCILIATION_ALREADY_RECONCILED` (code trompeur pour une ligne supprimée). Le détail est à l'AC3 (b). Rien
  n'est corrigé ici.
- **Une création concurrente du même compte** (`create` / `create_csv`, avec `confirmDuplicateFile=true` pour le
  même fichier, ou un relevé qui chevauche la période) fixe sa vue cohérente à `find_by_company_and_hash` puis lit
  `find_in_dedup_window` sur cette vue (`routes/bank_imports.rs:1108-1119, 1173`) : une suppression validée entre
  les deux lui fait juger « doublons » — et **ignorer** par défaut — des lignes qui n'existent plus. L'import
  persisté compte alors moins de lignes que le fichier. Deux gestes simultanés du même utilisateur, fenêtre de
  quelques millisecondes ; la story ne le ferme pas (revue P1, R11/F13).
- **La restauration d'installation contre la suppression** — phrase **minimale**, à recopier telle quelle au
  doc-comment (C-27-1-20 : sortie par retrait après trois réécritures) : la restauration supprime les lignes de
  `bank_transactions` puis celles de `bank_imports` (`backup.rs:492-500`, boucle sur `TABLES_TO_TRUNCATE` dans
  l'ordre des l. 67-68), la suppression verrouille l'entête puis les lignes : un interblocage est possible, rejoué
  côté suppression par `retry_app_on_deadlock`. La restauration n'étant exclusive que des autres imports
  (Story 15-12b), une suppression concurrente d'une restauration n'est pas gardée au-delà.
- **Une sauvegarde prise pendant une suppression** peut porter des transactions orphelines (AC8,
  [#633](https://github.com/guycorbaz/kesh/issues/633) ; l'export global CSV, #465).
- **Imports que l'écran ne peut pas supprimer** — énumération exacte, au lieu d'« indélébile » (revue P2, R2-3,
  F2-4) : (i) une transaction rapprochée dont l'annulation est **refusée** : le remède est celui du motif (cinq
  motifs au manuel, `\S\ref{sec:annuler-rapprochement}`, l. 1964-1983 — rouvrir des exercices, faire reculer le
  verrou de période, réactiver un compte, créer l'exercice du jour) ; **aucun** pour une facture créditée par un
  avoir (cas hérité d'avant la 0.12.1) ; (ii) l'état
  `pending` **avec** lien (D3), qu'aucun chemin ne produit : la page n'offre *Annuler le rapprochement* qu'aux lignes
  `reconciled` ; (iii) l'état `reconciled` **sans** lien (D4) : le bouton s'affiche, mais `cancel_in_tx` exige le
  lien (`reconciliation_cancel.rs:98-104`, `DbError::Invariant`) et l'annulation échoue. (ii) et (iii) n'ont aucun
  remède à l'écran ; ils ne naissent que de données anciennes ou d'une intervention en base.
- **Le prédicat est dupliqué** Rust/TypeScript (une requête SQL et un helper d'écran) : l'écran n'est qu'une
  commodité, le serveur seul décide (`409`). Chaque site renvoie à l'autre dans son doc-comment.

### État actuel des fichiers modifiés — ce qui doit être préservé

- `crates/kesh-api/src/routes/bank_imports.rs` (1999 lignes) : `preview`, `create` (CAMT, l. 1048-1279),
  `create_csv` (l. 1758…), `list` (l. 1287-1317), `detail` (l. 1320-1340), `insert_canonical_audit_log` (l. 1462-1517,
  action `bank_import.created`, à ne **pas** généraliser : la suppression a sa trace propre). Rien de l'existant ne
  change ; seul le doc-comment de module (l. 1-11) gagne la route.
- `crates/kesh-db/src/repositories/bank_imports.rs` (294 lignes) : `create_with_transactions`,
  `find_by_company_id`, `list_all_by_company`, `count_by_company_id`, `find_by_company_and_hash` (exécuteur
  générique), `find_by_id_for_company` (pool). La constante `COLUMNS` (l. 21-23) sert le `SELECT … FOR UPDATE` de
  l'entête ; `bank_transactions::COLUMNS` est `pub(crate)` (`bank_transactions.rs:21-24`).
- `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` (156 lignes) : `load()` (l. 31-50), `canManage`
  (l. 20-22), le bouton *Annuler le rapprochement* par ligne (l. 129-138) et son dialogue (l. 146-156). Préserver ;
  le composant neuf s'insère sans déplacer l'existant.
- `frontend/src/lib/features/bank-import/bank-import.api.ts` : ajouter une fonction, ne rien changer à
  `appendFlags`.
- `crates/kesh-api/tests/audit_route_registry.rs` : ses compteurs (AC4) et son doc-comment ; aucune autre ligne.

### Effets vérifiés hors de la story (AC8)

- `.keshbackup` : `parse_and_verify` compare l'inventaire des **tables**, pas leurs lignes ; aucune table ne
  change → une sauvegarde prise **avant** ou **après** une suppression s'importe comme toute autre. Rien n'est
  promis d'une sauvegarde prise **pendant** (sauvegarde sans instantané commun,
  [#633](https://github.com/guycorbaz/kesh/issues/633) ; export global CSV, #465 ; AC8).
- Archivage d'un compte bancaire : la garde `412 BANK_ACCOUNT_HAS_TRANSACTIONS` se lève une fois les imports
  supprimés (AC8, H6).
- Export global : `exports/global.rs:131-132` liste `bank_imports` et `bank_transactions` ; les lignes supprimées
  n'y sont plus, comme toute ligne supprimée.
- Réconciliation : `GET /reconciliation/proposals` ne lit que les transactions existantes et `pending`.

### Pièges connus du dépôt, applicables ici

- **Lecture ordinaire avant le verrou** (15-8a) : la première instruction de la transaction est le `FOR UPDATE` de
  l'entête. Pas de `find_by_id_for_company` sur le pool « pour vérifier d'abord ».
- **Test muet** : les tests de course ne se synchronisent **jamais** par `sleep` ; `attendre_une_requete_en_cours`
  échoue si la requête n'est jamais vue (10 s) et s'interrompt si la tâche a fini sans attendre.
- **Mutation restaurée, binaire périmé** : `touch` après chaque restauration.
- **Base partagée** : ⛔ jamais `docker compose restart mariadb` ni `docker restart kesh-mariadb-dev` (tmpfs commun
  aux autres agents) ; remettre **sa** base à zéro par `DROP DATABASE`/`CREATE DATABASE` + migrations + seed.
- **E2E local** : `KESH_COOKIE_SECURE=false`, `KESH_HOST=127.0.0.1`, `KESH_TEST_MODE=true`,
  `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, `KESH_BACKEND_URL` sur le port attribué,
  `KESH_ADMIN_BACKUP_DIR` inscriptible. Attendre `/health` ; tuer le backend **par PID**.
- **Données réelles** : rien de `~/kesh-test/` (relevé réel) dans une fixture, un test, un commit ; la fixture E2E
  est inventée.
- **Manuel** : contrôler le PDF aplati, pas seulement le `.tex`.

### Découpage préventif — dérogation (C-27-1-9)

Modules touchés : `kesh-db` (un dépôt, un fichier de test), `kesh-api` (une route, `errors.rs`, `lib.rs`,
`audit_labels.rs`, deux fichiers de test), `kesh-i18n` (quatre catalogues), frontend (`features/bank-import`, une
page, une spec E2E), documentation — plus, depuis la remédiation P1, **une ligne** dans
`routes/reconciliation.rs` (AC4-bis, tri du lot de rejet) et un test dans `reconciliation_e2e.rs` et
`bank_accounts_e2e.rs`. **Deux décomptes** (revue P2, R2-7, F2-8) : au grain des **crates et ensembles**, cinq ; au
grain des « modules métier de premier niveau » de la règle (`kesh-db/repositories/bank_imports`,
`…/bank_accounts` pour un doc-comment, `kesh-api/routes/bank_imports`, `kesh-api/routes/reconciliation`, socle
`kesh-api` — `errors`, `lib`, `audit_labels` —, `kesh-i18n`, `frontend/features/bank-import`, la page
`frontend/routes/bank-import`) : **plus de cinq** — le seuil est franchi à ce grain. Un seul domaine (l'import
bancaire et sa réconciliation), sans changement de règle transverse, et la story est une tranche verticale qu'un
découpage laisserait sans écran entre deux fusions.

**Après la P3 (2026-10-12) — signal de recyclage, sortie par retrait.** L'angle mort « restauration contre
suppression » a été écrit en P1, réécrit en P1 (F4) puis en P2 (F2-3/R2-9), et il était **faux** en P3 (R3-1/F3-1 :
la restauration ne vide pas `audit_log`, `backup.rs:493`) — un défaut né du correctif précédent, sur le même
texte : c'est le critère de **recyclage** de l'amendement D5. Il ne touche que du texte (aucune ligne de code
prescrit). L'orchestrateur en sort par un **retrait**, non par une quatrième réécriture : l'angle mort est réduit
à la phrase minimale vérifiable (C-27-1-20). Pas de découpage ; signal déclaré au Project Lead par l'orchestrateur.

**Décision de l'orchestrateur après la P2 (2026-10-12) : pas de découpage.** Constat : les défauts de la P2 sont
**distincts** de ceux de la P1 et aucun n'est un recyclage (amendement D5 du `CLAUDE.md`, rétrospective de
l'Epic 25) ; la sévérité baisse (P1 : 7 MEDIUM distincts, P2 : 4, 0 HIGH aux deux) ; le domaine reste unique. Le
signal est **déclaré au Project Lead** par l'orchestrateur. **Repli** si une passe ultérieure recycle un défaut :
**27-1a** (T1, T2, i18n serveur, `api-external.md` — `refs #632`) puis **27-1b** (T4, T5, T6 hors
`api-external.md`, i18n frontend — `closes #632`).

### Project Structure Notes

- Fichiers neufs : `crates/kesh-db/tests/bank_import_delete.rs` ; `frontend/src/lib/features/bank-import/
  {bank-import-deletion.ts, bank-import-deletion.test.ts, DeleteBankImportAction.svelte,
  DeleteBankImportAction.test.ts}` ; `frontend/tests/e2e/bank-import-delete.spec.ts` ;
  `frontend/tests/e2e/fixtures/csv_e2e_delete_import.csv`.
- Fichiers modifiés : `crates/kesh-db/src/repositories/{bank_imports.rs, bank_accounts.rs (doc-comment seul)}` ;
  `crates/kesh-api/src/{errors.rs, lib.rs, audit_labels.rs, routes/bank_imports.rs, routes/reconciliation.rs}` ;
  `crates/kesh-api/tests/{bank_imports_e2e.rs, bank_accounts_e2e.rs, reconciliation_e2e.rs, audit_route_registry.rs}` ;
  `crates/kesh-i18n/locales/*/messages.ftl` ; `frontend/src/lib/features/bank-import/{bank-import.api.ts,
  bank-import.api.test.ts}` ; `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` ;
  `frontend/src/lib/shared/i18n-keys.test.ts` ; `docs/manual/fr/{user-manual,admin-manual}.{tex,pdf}` ; `docs/api-external.md` ; `kesh-i18n/src/loader.rs` (test de message) ;
  `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md` (note rectificative, déjà posée par la fiche) ;
  `CHANGELOG.md` ; `README.md`.
- Ajoutés par les remédiations de revue de code (revue de code P2, A2-1) : neuf —
  `frontend/src/routes/(app)/bank-import/[id]/bank-import-detail-page.test.ts` (P2) ; modifiés —
  `crates/kesh-db/src/entities/bank_import.rs` (P1), `crates/kesh-api/src/routes/bank_accounts.rs` (P1, P2),
  `crates/kesh-api/tests/textes_coherents.rs` (P1, P2), `frontend/src/lib/features/bank-accounts/{bank-accounts.api.ts,
  bank-accounts.api.test.ts}` (P1, P2), `frontend/src/lib/features/homepage/BankAccountsCard.svelte` (P1, P2),
  `frontend/src/lib/features/bank-import/BankImportList.svelte` (P2), `docs/manual/Makefile` (P2),
  `docs/manual/fr/marketing-brochure.pdf` (P2, régénéré par le Makefile).
- Aucun fichier de `crates/kesh-db/migrations/`, `test-schema/`, `post_restore.rs` (AC15).

### References

- Issue #632 ; epic `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md`.
- `CLAUDE.md` : § *Test Locally First*, § *Review Iteration Rule* (et ses sous-sections : manuel, axes déclarés,
  sites non résolus, propagation post-patch, recompte), § *Pattern batch* (non applicable : route unitaire),
  § *Migration breaking policy* (non applicable, AC15), § *Issue Tracking Rule*.
- Registre `epic-15-choix-autonomes.md` : C54-C59 (rejeu) ; registre `epic-27-choix-autonomes.md` : C-27-1-1 à 31.
- Code : `kesh-reconciliation/src/mutex.rs` ; `kesh-api/src/retry.rs` ; `kesh-db/src/retry.rs:100-160` ;
  `kesh-db/src/test_fixtures.rs:561-605` ; `kesh-db/tests/reconciliation_cancel.rs:127-185` (patron de course).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), en autonomie, worktree `/home/gcorbaz/devel/kesh-27-1`.

### Debug Log References

Journaux hors dépôt, `/home/gcorbaz/devel/kesh-gate-logs/` : `27-1-explain.log` (F10), `27-1-mutations.log` et
`27-1-mut/*.out` (campagne), `27-1-gate-summary.log`, `27-1-gate-backend.log`, `27-1-gate-front-*.log`,
`27-1-gate-e2e.log` (+ `.backend`), `27-1-make-manuals.log`. Bases `kesh_271` (gate) et `kesh_e2e_271` (E2E), port 3034.

**F10 — plan constaté** (`EXPLAIN` sur `kesh_271`, MariaDB 10.11, 500 transactions dont 450 pour un import,
`ANALYZE TABLE` fait) :

```
sans FORCE INDEX, import minoritaire : type=ref   key=idx_bank_transactions_import rows=50  Using where
sans FORCE INDEX, import majoritaire : type=index key=PRIMARY                      rows=500 Using where
avec FORCE INDEX, import majoritaire : type=ref   key=idx_bank_transactions_import rows=450 Using where
```

L'optimiseur bascule sur l'index primaire dès qu'un import porte l'essentiel des lignes : `FORCE INDEX
(idx_bank_transactions_import)` posé (C-27-1-21).

### Completion Notes List

**Ce qui a tourné, au dernier commit de code `6e305767`** (base `kesh_271` recréée — `DROP`/`CREATE`, migrations,
seed — et `kesh_e2e_271` recréée, sans toucher au conteneur ; tout sous `flock`) :

- `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest, profil par défaut) : **3438 exécutés, 3438 passés,
  6 ignorés**. Delta depuis la v0.13.1 (3419 au `CLAUDE.md`) : **+19**, tous neufs — 9 dans `bank_import_delete.rs`
  (D1-D6, AC3 (a)(b)(c)), 6 dans `bank_imports_e2e.rs` (H1-H5, H8), 2 dans `bank_accounts_e2e.rs` (H6, H9), 1 dans
  `reconciliation_e2e.rs` (H7), 1 dans `loader.rs` (test de message). `kesh-import` non touché : pas de
  `cargo publish --dry-run`.
- En plus, `bank_accounts_e2e` en mode CI (`cargo test -p kesh-api --test bank_accounts_e2e -- --test-threads=1`,
  un seul processus) : **42 passés** — H6 pose la langue des erreurs, globale au processus, et les autres tests du
  binaire n'en sont pas affectés.
- Frontend : `npm run check` 0 erreur, 28 avertissements (préexistants, aucun sur un fichier de la story) ;
  `npm run lint-i18n-ownership` PASS ; `npm run test:unit` **131 fichiers, 1347 tests passés** (1334 + 13 neufs :
  `bank-import-deletion.test.ts` 4, `bank-import.api.test.ts` +1, `DeleteBankImportAction.test.ts` 8) ;
  `npm run build` vert.
- **E2E complet** (backend sur 3034, `kesh_e2e_271`, recette de `docs/testing.md`, `/health` `smtpConfigured:true`,
  run à 16:47 UTC) : **263 passés, 7 échecs, 19 ignorés**. Les sept échecs sont exactement la KF-029 (#97) :
  `mode-expert.spec.ts:26` et `:41`, `onboarding-path-b.spec.ts:65` et `:92`, `onboarding.spec.ts:57`, `:77`, `:150`.
  Aucun échec hors liste ; la KF-045 (rappels datés avant 12:00 UTC) n'a pas joué. La spec neuve
  `bank-import-delete.spec.ts:87` passe (elle était aussi passée seule, avant le gate).
- Manuels : `make -C docs/manual user admin` sous `flock` (0 référence indéfinie au passage final), PDF contrôlés
  aplatis (`pdftotext … | tr '\n' ' ' | tr -s ' '`) : « Supprimer un import », « de plus grande période » (deux
  sites), « sauf à supprimer son import », « une clé d'API ne peut pas supprimer un import » (utilisateur) ; puce
  « Suppression d'un import bancaire » et puce *Portée* (administration) — toutes présentes.

**Campagne de mutations (AC16)** — 20 jouées une par une sur arbre propre, chacune restaurée par `git checkout --`
puis `touch` (journal `27-1-mutations.log`, HEAD `36de9980`, puis M20 rejouée sur `6e305767`) :

| # | Résultat observé |
|---|---|
| M1 | rouge : D4 (D2 vert) et H2 |
| M2 | rouge : D3 |
| M3 | rouge : AC3 (a), **panique de la sonde à 10,155 s** — détection par absence du motif, déterministe |
| M4 | rouge : D5 et H3 |
| M5 | rouge : H1 |
| M7 | rouge : H2 |
| M8 | rouge : H4 |
| M9 | rouge : `audit_label_registry` (`les_actions_de_la_liste_sont_exactement_celles_que_le_code_ecrit`) |
| M10 | rouge : `audit_route_registry` (3 tests) |
| M11 | rouge : `bank-import-deletion.test.ts` (2 tests) |
| M12 | rouge : `DeleteBankImportAction.test.ts` (identifiant) |
| M13 | rouge : `DeleteBankImportAction.test.ts` (double clic, deux clics sans attendre le DOM) |
| M14 | **équivalente constatée** : AC3 (c) vert |
| M15 | **équivalente constatée** : AC3 (c) vert |
| M14+M15 | rouge : AC3 (c), par l'assertion |
| M16 | rouge : `DeleteBankImportAction.test.ts` (`canManage: false`) |
| M17 | rouge : H7 |
| M18 | rouge : H8 |
| M19 | rouge : test de message de `loader.rs` |
| M20 | **survivante au premier passage** — H6 lisait le **repli** Rust (le montage de `bank_accounts_e2e.rs` n'initialise pas la langue des erreurs), qui porte la même phrase. Test renforcé (`init_error_i18n` fr-CH dans H6, commit `6e305767`) ; rejouée : **rouge** (H6) |

Bilan : **18 rouges sur 18 mutations non équivalentes** (dont M20 après renforcement), **2 équivalentes constatées**
(M14, M15), comme la fiche l'annonçait.

**Recontrôles de fermeture** : AC15 — `git diff --stat aadace40` vide sur `migrations/`, `test-schema/`,
`post_restore.rs`, les `Cargo.toml` et `kesh-import`. AC8 — le `grep` de l'inventaire (b) rend 14 sites hors
commentaires, les 13 de la fiche plus le `DELETE FROM bank_imports` de `delete_unreconciled_in_tx`. Registre des
routes : 115 / 109 / 118 / 91, « sept » au point (vi), confirmés par la garde. `sitesTotal` 2041 confirmé par la garde
(+8 `DeleteBankImportAction.svelte`, +1 page détail, comptés `grep -o "i18nMsg("` aux deux bornes). Clés i18n : 11
neuves par locale (`grep -c`), une modifiée.

**Écarts à la fiche, et pourquoi** :

- La fiche nomme les bases `kesh_27_1` / `kesh_e2e_27_1` (T0) ; l'orchestrateur a attribué `kesh_271` /
  `kesh_e2e_271` — ce sont elles qui ont servi.
- H6 pose la langue des erreurs (`init_error_i18n` fr-CH) : sans cela, il ne prouvait que le repli (M20 survivante).
- Second site « dernier relevé » du manuel (l. 274) corrigé au même titre que l. 250-251 (C-27-1-22) ; quatre
  commentaires frontend qui disent « dernier relevé » laissés hors périmètre.
- AC1 (revue de code P2, A2-2) : la définition Rust de « rapprochée » est la **fonction** `is_reconciled`
  (`repositories/bank_imports.rs`), appliquée aux lignes de la lecture verrouillante de l'AC2, et non une clause SQL
  de cette requête comme l'écrit la lettre de l'AC1 ; un seul site Rust, comme prescrit, et l'intention de l'AC2
  (compter sur le résultat verrouillé, pas par une seconde requête) est tenue.
- AC9 (revue de code P2, B2-4) : le message du `409` de course n'est plus « Une transaction de cet import a été
  rapprochée entre-temps ({ $count }) » mais « { $count } transaction(s) de cet import ont été rapprochées
  entre-temps : rechargez la page. » (C-27-1-29).
- Le paragraphe *Rejeter* nomme l'API sans les routes (« le rapprochement manuel et l'éclatement l'acceptent
  encore ; l'acceptation d'une proposition la refuse »), le manuel utilisateur ne citant pas de chemins d'API à cet
  endroit.

### File List

Neufs :

- `crates/kesh-db/tests/bank_import_delete.rs`
- `frontend/src/lib/features/bank-import/bank-import-deletion.ts`
- `frontend/src/lib/features/bank-import/bank-import-deletion.test.ts`
- `frontend/src/lib/features/bank-import/DeleteBankImportAction.svelte`
- `frontend/src/lib/features/bank-import/DeleteBankImportAction.test.ts`
- `frontend/tests/e2e/bank-import-delete.spec.ts`
- `frontend/tests/e2e/fixtures/csv_e2e_delete_import.csv`

Modifiés :

- `crates/kesh-db/src/repositories/bank_imports.rs`, `crates/kesh-db/src/repositories/bank_accounts.rs` (doc-comment)
- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/lib.rs`, `crates/kesh-api/src/audit_labels.rs`,
  `crates/kesh-api/src/routes/bank_imports.rs`, `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`, `crates/kesh-api/tests/bank_imports_e2e.rs`,
  `crates/kesh-api/tests/bank_accounts_e2e.rs`, `crates/kesh-api/tests/reconciliation_e2e.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, `crates/kesh-i18n/src/loader.rs`
- `frontend/src/lib/features/bank-import/bank-import.api.ts`, `frontend/src/lib/features/bank-import/bank-import.api.test.ts`,
  `frontend/src/routes/(app)/bank-import/[id]/+page.svelte`, `frontend/src/lib/shared/i18n-keys.test.ts`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`, `docs/manual/fr/admin-manual.tex`,
  `docs/manual/fr/admin-manual.pdf`, `docs/api-external.md`, `CHANGELOG.md`, `README.md`
- `_bmad-output/implementation-artifacts/{27-1-supprimer-import-non-rapproche.md, epic-27-choix-autonomes.md,
  sprint-status.yaml, 27-1-review-prompt-p1.md}`

Ajoutés par les remédiations de revue de code (revue de code P2, A2-1 — la liste ci-dessus s'arrêtait au
développement ; recompte : `git diff --name-only fc45574c 1c9e4ea0 -- . ':(exclude)_bmad-output' | wc -l` → **44**,
soit les 34 ci-dessus et les 10 suivants) :

- Neuf : `frontend/src/routes/(app)/bank-import/[id]/bank-import-detail-page.test.ts` (P2, E-P2-5)
- Modifiés en P1 (et retouchés en P2 pour certains) : `crates/kesh-db/src/entities/bank_import.rs`,
  `crates/kesh-api/src/routes/bank_accounts.rs`, `crates/kesh-api/tests/textes_coherents.rs`,
  `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts`,
  `frontend/src/lib/features/bank-accounts/bank-accounts.api.test.ts`,
  `frontend/src/lib/features/homepage/BankAccountsCard.svelte`
- Modifiés en P2 : `frontend/src/lib/features/bank-import/BankImportList.svelte` (E-P2-6), `docs/manual/Makefile`
  (C-27-1-30), `docs/manual/fr/marketing-brochure.pdf` (régénéré)
- Artefacts : `27-1-review-prompt-p2.md`, `27-1-review-prompt-p3.md`

## Change Log

- **2026-10-12** — Fiche créée (`bmad-create-story`, Opus 5.5, en autonomie) sur `aadace40`. 16 AC, 9 tâches
  (T0-T8). Choix C-27-1-1 à C-27-1-11 au registre. Prémisse « unicité du hash » de l'epic et de l'issue corrigée
  (C-27-1-8). Validation P1 à lancer : `27-1-validate-prompt-p1.md` (lentilles R et F).

### Validation P1 — bilan finding par finding

Passe P1 (Sonnet, contexte frais, lecture seule) : lentille **R** 0 HIGH / 3 MEDIUM / 8 LOW, lentille **F** 0 HIGH
/ 7 MEDIUM / 6 LOW (`kesh-gate-logs/27-1-validate-p1-{R,F}.md`). Doublons : R1 = F1 + F2, R2 = F9, R3 = F6,
R4 = F8 — **7 MEDIUM distincts** (R1/F1/F2, R2, R3/F6, F3, F4, F5, F7 ; le message de l'orchestrateur annonçait « 9 » en en listant 7 — recompté ici). Remédiation (Opus), arbitrages de l'orchestrateur pour R3/F6, F5, F3 :

| Finding | Sév. | Traitement |
|---|---|---|
| R1 / F1 / F2 — motifs de synchronisation de l'AC3 | MEDIUM | (a) motif `["bank_transactions","FOR UPDATE"]`, M3 rouge par absence du motif (déterministe) ; (c) motif `["FOR UPDATE"]`, M14 et M15 équivalentes chacune, M14+M15 rouge par l'assertion (C-27-1-15) |
| R2 / F9 — M6 équivalente, M1 mal attribuée | MEDIUM / LOW | M6 retirée avec sa raison (rollback) ; M1 → D4, H2 ; H2 précisé (`reconciled` sans lien) |
| R3 / F6 — garde d'archivage `412` non inventoriée, C-27-1-7 quasi vide | MEDIUM | **accepté** (orchestrateur) : AC8 réécrit, test H6, phrase au manuel et à `api-external.md`, doc-comment de la garde ; C-27-1-7 remplacé par C-27-1-12 |
| F3 — `post_reject` non rejouée, cycle avec la suppression | MEDIUM | **traité** : AC4-bis, tri des identifiants (une ligne), test H7, M17, CHANGELOG `### Modifié` (C-27-1-14) |
| F4 — restauration « exclusive des autres écrivains » | MEDIUM | phrase corrigée (« des autres imports seulement », 15-12b) ; cycle restauration/suppression en angle mort |
| F5 — sauvegarde sans instantané | MEDIUM | **non corrigé** (orchestrateur) : promesse retirée de l'AC8, angle mort renvoyé à #465 (C-27-1-13) |
| F7 — paragraphe *Interblocages* d'`api-external.md:662` | MEDIUM | ajouté à l'AC4 et à l'AC14 |
| R4 / F8 — `traced 108 → 109` | LOW | ajouté à l'AC4 et au `grep` de contrôle |
| R5 — masquage Consultation sans test | LOW | `canManage` devient une prop du composant ; test Vitest ; M16 |
| R6 / F12 (tuile) — « l'import précédent » | LOW | « le relevé restant de plus grande période portant un solde de clôture » (AC8, AC13) |
| R7 — montage D2/D3 | LOW | renvoi au montage de `reconciliation_cancel.rs:29-60` |
| R8 / F12 (message) — bouton absent ou annulation refusée | LOW | texte du motif sans renvoi au bouton ; manuel et angles morts disent l'import indélébile dans ces cas |
| R9 — « aucun geste » à borner | LOW | « aucun geste **de l'interface** » (AC13) |
| R10 — phrase corrompue de T1.1 | LOW | réécrite |
| R11 / F13 — création concurrente et détection de doublons | LOW | angle mort écrit |
| F10 — plan d'exécution | LOW | `EXPLAIN` constaté au dev, `FORCE INDEX` sinon ; verrou de clé suivante écrit |
| F11 — effacements totaux | LOW | ajoutés à l'inventaire (b) |

Recomptes après remédiation : 17 AC ; tâches T0-T8 (T2.7 neuve) ; mutations M1-M17, M6 retirée, plus la combinaison
M14+M15 : **17 à jouer** ; dix clés i18n, `sitesTotal` +9 inchangé (aucun site ajouté par la remédiation) ; registre des routes
114→115, 117→118, 90→91, 108→109, « six » → « sept ». Symptômes grepés dans la fiche et le registre (`["bank_transactions"]`,
« équivalente probable », « import précédent », « à l'identique », « exclusive des autres écrivains », « de la
ligne », `108`, `six`) : seules restent les mentions historiques voulues.

### Validation P2 — bilan finding par finding

Passe P2 (Opus, contexte frais, lecture seule) : lentille **R** 0 HIGH / 3 MEDIUM / 9 LOW, lentille **F** 0 HIGH /
1 MEDIUM / 7 LOW (`kesh-gate-logs/27-1-validate-p2-{R,F}.md`). Doublons : R2-2 = F2-2 (MEDIUM chez R, LOW chez F),
R2-4 = F2-6, R2-7 = F2-8, R2-5 ⊂ F2-1. **4 MEDIUM distincts** (F2-1, R2-1, R2-2/F2-2, R2-3), recomptés ; tous nés de
phrases écrites par la remédiation P1 sauf F2-1 (d'origine, non vu en P1). **Trend : P1 7 MEDIUM distincts → P2 4,
0 HIGH aux deux.** S'y ajoute la **décision du Project Lead** du 2026-10-12 (C-27-1-16).

| Finding | Sév. | Traitement |
|---|---|---|
| Décision PL — clés d'API | — | route **fermée à toute clé** : `403 API_KEY_OPERATION_FORBIDDEN` (code neuf, doctrine D2) en tête du handler ; H8, M18 ; `api-external.md` § 4/§ 7/§ 10 réécrits ; paragraphe *Interblocages* laissé inchangé (le correctif P1 F7 est **retiré** : il n'énumère que les routes ouvertes aux clés) ; C-27-1-3 révisé |
| F2-1 / R2-5 — lot refusé en entier (`400`) quand une transaction a disparu | MEDIUM / LOW | **non corrigé** (orchestrateur, #634) : AC3 (b) réécrit en deux trajectoires, puce AC8, angle mort, phrase au manuel (recharger la page) ; C-27-1-17 |
| R2-1 — #465 ne suit pas la sauvegarde | MEDIUM | renvoi à **#633** (ouverte par l'orchestrateur) pour la sauvegarde, #465 gardée pour l'export global CSV (AC8, § Effets, angles morts) ; C-27-1-13 révisé |
| R2-2 / F2-2 — montage de l'AC3 (c) | MEDIUM | l'import porte au moins une transaction, raison écrite |
| R2-3 — « rend archivable », « indélébile » | MEDIUM | AC13 : archivable **s'il ne reste aucune transaction**, sous les autres gardes (compte principal) ; « indélébile » remplacé par « tant que l'annulation est refusée, l'import ne peut pas être supprimé » et renvoi aux remèdes du § *Annuler un rapprochement* (cinq motifs, un sans remède) ; même retouche à l'AC8, aux angles morts, à `api-external.md` |
| R2-4 / F2-6 — effets du tri non écrits | LOW | `details_json.bank_transaction_ids`, commentaires `reconciliation.rs:3257` et `reconciliation_e2e.rs:1625` ajoutés à l'AC4-bis |
| R2-6 — G6 n'interdit pas « à venir » | LOW | attribution corrigée (règle de documentation, sans garde) |
| R2-7 / F2-8 — décompte du découpage | LOW | deux décomptes écrits ; seuil franchi au grain fin ; **pas de découpage**, décision de l'orchestrateur (C-27-1-18), signal déclaré au Project Lead |
| R2-8 — sortie de commande retouchée | LOW | six lignes de commentaire nommées |
| R2-9 / F2-3 — restauration | LOW | angle mort réécrit — ⚠️ **corrigé en P3** : la mécanique écrite ici (`audit_log` vidée avant) était fausse (`backup.rs:493` la saute) ; angle mort réduit à une phrase minimale (C-27-1-20) |
| R2-10 — message du `412` | LOW | une phrase de remède ajoutée au message (quatre catalogues et repli), code inchangé (C-27-1-19) ; arbitrage CO 958f signalé à Guy par l'orchestrateur |
| R2-11 — § *Comptes bancaires* muet | LOW | renvoi vers `sec:supprimer-import` |
| R2-12 — promesses sans test | LOW | `id = 0` dans H3 ; clés : H8 (décision PL) |
| F2-4 — état D4 sans remède | LOW | ajouté à l'énumération des imports que l'écran ne peut pas supprimer |
| F2-5 — verrou de trou toutes sociétés | LOW | AC2 reformulé |
| F2-7 — `innodb_snapshot_isolation` | LOW | phrase au doc-comment prescrit (MariaDB 10.11) |

Recomptes après remédiation : 17 AC ; T0-T8 ; mutations M1-M18, M6 retirée, plus M14+M15 : **18 à jouer** ;
i18n : **onze clés neuves et une modifiée** (`sitesTotal` +9 inchangé : aucun site frontend ajouté) ; tests H1-H8,
D1-D6, AC3 (a)(b)(c). Symptômes grepés (valeurs et formules) dans la fiche et le registre : « ouverte aux clés »,
« nomme la clé », `#465`, « dix clés », « indélébile », « rend archivable », « Signal déclaré à l'orchestrateur » —
ne restent que les mentions voulues (historique du bilan P1, négations).

### Validation P3 — bilan finding par finding

Passe P3 (Sonnet, complète, contexte frais, lecture seule) : lentille **R** 0 HIGH / 2 MEDIUM / 9 LOW, lentille **F**
0 HIGH / 2 MEDIUM / 4 LOW (`kesh-gate-logs/27-1-validate-p3-{R,F}.md`). Doublons : R3-1 = F3-1, R3-2 = F3-2, R3-10 =
F3-5 ; R3-9 et F3-6 se recouvrent en partie. **2 MEDIUM distincts**, 13 LOW bruts. **Trend des MEDIUM distincts :
P1 7 → P2 4 → P3 2, 0 HIGH partout.**

⚠️ **Signal de recyclage** (amendement D5) : R3-1/F3-1 est né du correctif P2, sur un angle mort déjà retouché en
P1 et en P2. Sortie par **retrait** (C-27-1-20), décidée par l'orchestrateur, déclarée au Project Lead ; pas de
découpage (dérogation mise à jour).

| Finding | Sév. | Traitement |
|---|---|---|
| R3-1 / F3-1 — angle mort restauration : `audit_log` n'est pas vidée | MEDIUM | **retrait** : phrase minimale vérifiable (`backup.rs:492-500`, ordre l. 67-68, rejeu, exclusivité 15-12b) ; ligne du bilan P2 annotée « corrigé en P3 » ; C-27-1-20 |
| R3-2 / F3-2 — propagation de C-27-1-16 | MEDIUM | `api-external.md` l. 205 (trois exceptions) et § 4 en paragraphe à part ; manuel **d'administration** § *Clés API* (puce neuve, puce *Portée*) et son PDF ; AC13 corrigé (« non concerné » était faux) ; phrase au manuel utilisateur ; grep des phrases qui énumèrent les fermetures prescrit en T6 (sites relevés : `api-external.md:77, 205, 565, 582, 603`, `admin-manual.tex:2269, 2272, 2280` — seuls 205 et 2272 énumèrent les exceptions hors administration, les autres parlent de l'administration seule et restent vrais) |
| R3-3 — mauvais helper cité | LOW | `find_strictly_pending_by_id_for_account` (`manual`, `split`) ; `accept` refuse au pré-vol |
| R3-4 / F3-3 — codes de la trajectoire « après pré-vol » | LOW | acceptation : `BANK_TRANSACTION_NOT_FOUND` puis `ALREADY_RECONCILED` ; rejet : `ALREADY_RECONCILED` ; fenêtre jusqu'à 5 s (verrou nommé) ; « quelques millisecondes » retiré de l'AC3 (b) |
| R3-5 — `sprint-status.yaml` périmé | LOW | ligne et `last_updated` mis à jour |
| R3-6 — prémisse du hash dans l'epic | LOW | note rectificative datée dans l'epic |
| R3-7 — promesses sans test | LOW | H9 (tuile, deux relevés) ; H6 asserte le message ; M20 |
| R3-8 — chemin de menu par locale | LOW | libellés réels `nav-quotidien` / `nav-bank-import` prescrits |
| R3-9 / F3-6 — test de message, précédent `audit_log.rs` | LOW | test de message dans `loader.rs` (M19) ; précédent contraire cité au registre (complément à C-27-1-16) |
| R3-10 / F3-5 — titre de C-27-1-3 | LOW | titre annoté « révisé », contexte corrigé |
| R3-11 — « dernier relevé importé » au manuel | LOW | phrase alignée (plus grande période) ; inventaire (a) aligné |
| F3-4 — `rollback` explicite | LOW | prescrit à l'AC4 (patron `imported_supplier_invoices.rs:315`) |

Recomptes après remédiation : 17 AC ; T0-T8 ; tests H1-H9, D1-D6, AC3 (a)(b)(c), test de message ; mutations M1-M20,
M6 retirée, plus M14+M15 : **20 à jouer** ; i18n inchangé (onze neuves, une modifiée ; `sitesTotal` +9). Symptômes
grepés : « DELETE FROM audit_log », « `audit_log` avant », « deux exceptions » (reste la citation de la phrase à
réécrire), « non concernés », « ne filtre pas le rejet », « dernier relevé », « C-27-1-1 à 19 » — ne restent que les
mentions voulues (citations, bilans historiques). « quelques millisecondes » subsiste pour la **création**
concurrente (autre fenêtre, juste).

### Validation P4 ciblée — validation close

Passe P4 (Haiku, **ciblée** : une lentille R, contexte frais, lecture seule, sur le seul commit de remédiation P3
`269bb704` ; prompt `27-1-validate-prompt-p4-ciblee.md`, rapport `kesh-gate-logs/27-1-validate-p4-R.md`) :
0 CRITICAL / 0 HIGH / **0 MEDIUM** / 2 LOW, axes exercés et non exercés déclarés (six exercés, quatre non exercés
dont toute exécution, interdite par le prompt).

| Finding | Sév. | Traitement |
|---|---|---|
| P4-1 — qualification du « recyclage » de R3-1/F3-1 | LOW | **deux lectures, nommées ici, arbitrage au Project Lead** : celle de l'orchestrateur (fiche, § Découpage préventif, et bilan P3) — R3-1/F3-1 est un **recyclage** au sens de l'amendement D5, puisqu'il « naît du correctif précédent » sur le même texte ; celle de la lentille R de la P3 (`27-1-validate-p3-R.md` l. 17-18) — « aucun des deux ne recycle un défaut antérieur au sens de la règle de découpage ». La fiche suit la lettre de D5, la lentille la conteste. La décision de sortie par **retrait** (C-27-1-20) ne dépend pas de l'arbitrage et reste inchangée |
| P4-2 — pointeur « § 10 pour le journal » (`api-external.md:205`) | LOW | phrase prescrite à l'AC14 corrigée : « § 4 pour la fermeture, § 10 pour le code d'erreur » (la fermeture du journal s'énonce au § 4, l. 83 ; le § 10 n'en porte que le code, l. 647) |

**Validation close.** Trend des MEDIUM distincts : **P1 7 → P2 4 → P3 2 → P4 ciblée 0**, 0 HIGH partout. Modèles :
Sonnet (P1, complète), Opus (P2, complète), Sonnet (P3, complète), Haiku (P4, ciblée sur la remédiation P3).
Reclassements : aucun. Décision du Project Lead intégrée en P2 (C-27-1-16). La remédiation P4 ne touche aucune
ligne de code (fiche seule) : la boucle se clôt.

### Développement — 2026-10-12

`bmad-dev-story` (Opus 5.5, autonomie), T0 à T8 dans l'ordre. Commits : `0af5b5ec` (dépôt et tests de dépôt),
`4e47602e` (route, erreurs, trace, tri du rejet, i18n, tests HTTP), `b32b05f6` (écran, Vitest), `e27cce8a` (E2E),
`c4511d2f` (manuels, API externe, CHANGELOG, README), `36de9980` (choix C-27-1-21 et 22), `6e305767` (H6 renforcé
après M20 survivante — dernier commit de code). Gates et campagne : Dev Agent Record. Statut → `review`.

### Revue de code P1 — bilan finding par finding

Passe P1 (Sonnet, **complète**, trois lentilles B, E, A en contexte frais, lecture seule, sur
`git diff fc45574c 6e305767 -- . ':(exclude)_bmad-output'` ; prompt `27-1-review-prompt-p1.md`, rapports
`kesh-gate-logs/27-1-review-p1-{B,E,A}.md`, axes exercés et non exercés déclarés par chacune) :
**0 CRITICAL / 0 HIGH / 1 MEDIUM / 14 LOW** (B : 3 LOW ; E : 1 MEDIUM, 6 LOW, 1 INFO ; A : 5 LOW, plus une
observation hors périmètre non comptée).

| Finding | Sév. | Traitement |
|---|---|---|
| E1 — retirer le `FORCE INDEX` ne fait rougir aucun test | **MEDIUM** | **corrigé** (C-27-1-23) : SQL de l'étape 2 extrait en `LOCK_IMPORT_TRANSACTIONS_SQL`, test `lock_query_uses_the_import_index` (`EXPLAIN` de la constante exécutée sur 500 lignes / 450 / 50 après `ANALYZE` : `key = idx_bank_transactions_import`, `type = ref`, plus un témoin sans l'indication). **M21** (indication retirée) : rouge, `PRIMARY` / `index` |
| E2 — H1 ne tient pas toutes les valeurs de la trace | LOW | **corrigé** : une transaction rejetée dans l'import supprimé (`rejected_count = 1`), `file_hash`, `source_format`, `period_from`, `period_to` comparés à l'entête en base, format `AAAA-MM-JJ`. **M22** (`rejected_count` à 0 dans le handler) et **M23** (`%d.%m.%Y`) : rouges |
| E3 + A-2 — nombre du dialogue : Vitest et E2E trop larges | LOW | **corrigé** : `detail(transactions, transactionCount)` et un Vitest où l'entête dit 9 pour trois lignes (« ses 3 transaction(s) », jamais « ses 9 ») ; E2E `toContainText(/ses 3 transaction\(s\)/)`. **M27** (le composant compte `detail.transactionCount`) : rouge |
| E4 — le 409 ne passe jamais par le catalogue ; G9 ne tient pas les replis neufs ; doc-comment de H6 faux | LOW | **corrigé** (C-27-1-25) : H2 pose `init_error_i18n(fr-CH)` et asserte le texte exact ; G9 étendue aux trois replis (`error-api-key-operation-forbidden`, `bank-import-errors-has-reconciled-transactions` avec `{ $count }` → `{reconciled_count}`, `bank-accounts-errors-has-transactions`) ; doc-comment de H6 corrigé. **M24** (clé mal orthographiée) et **M25** (repli divergent) : rouges. Binaire `bank_imports_e2e` rejoué en un seul processus : 39/39 |
| E5 — H7 observe les sorties, pas l'ordre des verrous ; `failed[]` non asserté | LOW | **corrigé** (C-27-1-24) : H7-bis, sonde `NOWAIT` sur `hi` pendant que le lot `[hi, lo]` attend `lo`. **M26** (ordre reçu, sorties triées après coup) : H7-bis rouge, H7 vert. `failed[]` : inatteignable hors course (pré-vol en `400`), écrit au doc-comment de H7-bis |
| E6 — rejeu sur interblocage et `rollback` sans mutation rouge | LOW | **laissé, angle mort assumé** (C-27-1-26) — déjà écrit au doc-comment de la route |
| E7 + B-3 — commentaires « dernier relevé », § 9 de l'API, renvoi au remède | LOW | **corrigé** : quatre commentaires frontend et le doc-comment de `routes/bank_accounts.rs` disent « de plus grande période » (C-27-1-22 révisé : ils ne sont plus laissés) ; § 9 de `api-external.md` : lignes « journal d'audit » et « suppression d'un import bancaire » ; `admin-manual.tex` (« Transactions déjà importées ») renvoie à *Supprimer un import* |
| E8 — campagne M1-M20 | INFO | rien à faire |
| B-1 — conservation dite sans réserve | LOW | **corrigé** : les deux puces (journal d'audit, CO 958f al. 1) disent qu'un import non rapproché peut être supprimé avec ses transactions et que `bank_import.deleted` reste — sans « seule exception » (la suppression d'une écriture existe aussi) |
| B-2 — « (0) » si `details` manque au 409 de course | LOW | **laissé** (C-27-1-26) : inatteignable, le serveur envoie toujours `reconciledCount` |
| A-1 — `entities/bank_import.rs` annonce l'UNIQUE et un 409 | LOW | **corrigé**, symptôme grepé (`UNIQUE…file_hash`, `uq_bank_imports_company_hash`, `409…DUPLICATE_FILE`) hors migrations et artefacts BMAD : trois copies fausses corrigées — l'entité, le doc-comment d'erreurs de `create_with_transactions` (`repositories/bank_imports.rs`), un commentaire de `bank_imports_e2e.rs` ; restent justes `inbox_import.rs:551` et `imported_supplier_invoices_repository.rs:5` (autre table) et `migrations-idempotence-audit.md:45` (historique) |
| A-3 — « C-27-1-1 à 20 » périmé | LOW | **corrigé** : « C-27-1-1 à C-27-1-27 » (Status, References, `sprint-status.yaml`) ; le registre en porte 27 (`grep -oE "^### C-27-1-[0-9]+"` : 27, aucun doublon) |
| A-4 — CHANGELOG : archivage sans condition | LOW | **corrigé** : « il ne lui reste alors aucune transaction — peut s'archiver, sous les autres conditions de l'archivage (un compte principal ne s'archive toujours pas) » |
| A-5 — titre du Vitest d'`isReconciled` | LOW | **corrigé** : « faux pour `pending` sans lien … une rejetée a le même état à l'écran, le DTO ne porte pas le rejet » |
| Observation — TOC des PDF décalée | — | **corrigé** (C-27-1-27) : PDF régénérés jusqu'à disparition de « Rerun » ; chaque entrée de TOC contrôlée contre son pied de page (95 entrées utilisateur, 122 administration, 0 écart). Makefile non touché |

**Commits de remédiation** : `1bee08b0` (E1), `e75e3d79` (E2), `d74cb9d8` (E4), `4f03693e` (E5), `25f17318` (E3,
A-2, A-5), `94326a24` (A-1, E7 côté code), `5881450b` (B-1, B-3, E7 § 9, A-4, PDF — dernier commit hors fiche).

**Mutations M21 à M27** (journal `27-1-mutations.log`, une par une sur arbre propre, `git checkout --` + `touch`) :
**7 jouées, 7 rouges**, chacune pour la raison attendue (motif d'échec relu dans `27-1-mut/M2x-0.out`). Campagne
complète de la story : M1 à M27 moins M6 (retirée par la fiche), soit 26 lignes jouées + M14+M15 ; 25 rouges non
équivalentes, M14 et M15 seules équivalentes.

**Gate complet au HEAD `5881450b`** (bases `kesh_271` et `kesh_e2e_271` recréées, sans toucher au conteneur, tout
sous `flock`) : `scripts/test-fast.sh` **3440 exécutés, 3440 passés, 6 ignorés** (+2 depuis `6e305767` :
`lock_query_uses_the_import_index`, `post_reject_locks_the_batch_by_increasing_id`) ; `npm run check` 0 erreur,
28 avertissements préexistants ; `lint-i18n-ownership` PASS ; `npm run test:unit` **131 fichiers, 1348 tests**
(+1) ; `npm run build` vert ; **E2E complet** : **263 passés, 7 échecs, 19 ignorés** — les sept de la KF-029 (#97),
`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` ;
`bank-import-delete.spec.ts:87` passe.

**Trend** : P1 **1 MEDIUM** (0 HIGH). La remédiation touche du code de production (`bank_imports.rs`, doc-comments)
et ajoute des tests : la boucle **n'est pas close** — passe **P2 complète (Opus)**, prompt versionné
`27-1-review-prompt-p2.md`, axe 9 braqué sur la remédiation P1. Reclassements : aucun.

### Revue de code P2 — bilan finding par finding

Passe P2 (Opus, **complète**, trois lentilles B, E, A en contexte frais, lecture seule, sur
`git diff fc45574c 5881450b -- . ':(exclude)_bmad-output'`, axe 9 sur la remédiation P1 ; prompt
`27-1-review-prompt-p2.md`, rapports `kesh-gate-logs/27-1-review-p2-{B,E,A}.md`, axes exercés et non exercés
déclarés par chacune) : **0 CRITICAL / 0 HIGH / 4 MEDIUM / 10 LOW** (B : 2 MEDIUM, 3 LOW ; E : 2 MEDIUM, 4 LOW,
4 INFO non comptées ; A : 3 LOW). Trois des quatre MEDIUM portent sur la **remédiation P1** (E-P2-2, B2-2) ou sur
un test qu'elle a laissé faible (E-P2-1) : le motif « la sévérité se déplace vers ce qu'on vient d'écrire ».

| Finding | Sév. | Traitement |
|---|---|---|
| E-P2-1 — H2 ne distingue pas `reconciledCount` de `transactionCount` (une seule ligne) | **MEDIUM** | **corrigé** : helper `add_lines` (copies de la première ligne, entête inchangé) ; H2 sur deux lignes dont une rapprochée — `reconciledCount == 1`, `transactionCount == 2`, message « 1 transaction(s)… », import intact à 2 lignes. **M28** (champs inversés dans `details`), **M29** (`count` = `transaction_count`), **M30** (champs inversés au mappage de `delete_once`) : rouges |
| E-P2-2 — H1 : `transaction_count` et `rejected_count` valent 1 tous deux (régression de preuve née de la remédiation E2) | **MEDIUM** | **corrigé** : H1 sur `n + 2 = 3` lignes, l'entête disant `n = 1`, une rejetée ; la trace doit dire 3 (lignes sous verrou) et 1. **M31** (`rejected_count` = `transaction_count`), **M32** (inverse), **M33** (`transaction_count` pris à l'entête) : rouges |
| B2-1 — dialogue fermable pendant l'envoi ; `open` non lié, le bouton ne le rouvre plus | **MEDIUM** | **corrigé** (C-27-1-28) : garde dans `Dialog.Content` (Échap, clic extérieur, croix) pendant `submitting`, patron de `WriteOffDialog`, et `bind:open`. Deux Vitest. **M36** (Échap), **M37** (croix), **M38** (`{open}` à sens unique) : rouges. `interactOutsideBehavior` non tenu par un test (angle mort écrit au registre) |
| B2-2 — « le relevé de plus grande période » est faux (le code retient la période qui **se termine le plus tard**) ; introduit par la remédiation P1 | **MEDIUM** | **corrigé** : « dont la période se termine le plus tard » aux trois sites du manuel utilisateur (PDF régénéré), au doc-comment de `routes/bank_accounts.rs`, au commentaire de H9, aux deux commentaires de `bank-accounts.api.ts`, à `BankAccountsCard.svelte`, au titre du `describe` de `bank-accounts.api.test.ts`. Valeur grepée sur tout le dépôt (« plus grande période » et variantes `plus longue période`, `greatest/largest period`, `längste`, `periodo più`) : ne restent que les artefacts BMAD (historique) ; PDF aplati : 0 occurrence |
| E-P2-3 — langue globale de H2 : les trois tests `ubs_*` lisent le catalogue ou le repli selon le lanceur, clés hors G9 | LOW | **corrigé** : les trois clés de la 26-1 inscrites à G9 (paramètres `{ $count }` → `{total_errors}`, `{ $line }` → `{line}`), doc-comment de H2 complété. **M35** (repli ubs divergent) : rouge |
| E-P2-4 — l'ordre « clé avant `404` » n'est tenu par aucun test | LOW | **corrigé** : H8 frappe aussi `id + 10_000` avec chaque clé ⇒ `403`. **M34** (refus déplacé après la recherche) : rouge (`404` sur l'id inconnu) |
| E-P2-5 — masquage Consultation prouvé au composant seulement | LOW | **corrigé** (C-27-1-31) : Vitest de page `bank-import-detail-page.test.ts` (Admin, Comptable : bouton ; Consultation : aucun, détail chargé). **M39** (`canManage={true}` passé par la page) : rouge |
| E-P2-6 — l'E2E clique le lien par son rôle | LOW | **corrigé** : `data-testid="bank-import-list-link"` sur le lien de `BankImportList.svelte`, E2E sur ce testid (les deux sites). Les lignes restent filtrées par `hasText: FILENAME` sur le testid de ligne |
| A2-1 — File List et *Project Structure Notes* sans les fichiers de la remédiation | LOW | **corrigé** : dix fichiers ajoutés (34 + 10 = 44, recompté par `git diff --name-only`) |
| A2-2 — AC1 : définition Rust dans une fonction, non dans la requête ; écart non déclaré | LOW | **corrigé** : ligne aux *Écarts à la fiche* |
| A2-3 — H7-bis : la sonde voit une requête en cours, non bloquée | LOW | **corrigé** comme angle mort écrit au doc-comment de H7-bis (fenêtre de l'ordre de la microseconde, aucun faux rouge possible sur le code juste) |
| B2-3 — « se réimporte sans l'avertissement » absolu | LOW | **corrigé** : « s'il n'en reste aucun autre import » au manuel (avec la raison) et au CHANGELOG |
| B2-4 — « Une transaction … ({ $count }) » | LOW | **corrigé** (C-27-1-29) : « { $count } transaction(s) de cet import ont été rapprochées entre-temps : rechargez la page. » dans les quatre locales et au repli Svelte ; Vitest du `409` mis à jour |
| B2-5 — `POST /bank-imports/preview` absent du § 7 | LOW | **corrigé** |
| Makefile (C-27-1-27, demande de l'orchestrateur) | — | **corrigé** (C-27-1-30) : passes jusqu'à stabilisation de `.aux`/`.toc`/`.out` et absence d'avertissement « Rerun to get / Rerun LaTeX / Please rerun / Label(s) may have changed », 2 à 6 passes. `make -B fr` sous `flock` : 2 passes stables chacun ; depuis zéro (copie au scratchpad) : 3 passes, `.toc` identiques octet pour octet ; aucun avertissement de nouvelle passe ni référence indéfinie aux journaux. Sondage de toutes les entrées de TOC contre les pieds de page : utilisateur 116/121, administration 93/94 — les six écarts sont des titres orphelins en bas de page (la TOC pointe vers la page où commence leur texte), identiques dans les PDF de `5881450b` : préexistants, non traités. Le « 0 écart » sur 95 et 122 entrées déclaré en P1 (C-27-1-27) ne se retrouve pas par ce sondage, qui lit 121 et 94 entrées dans les `.toc` (sections, sous-sections, chapitres) : le périmètre de la P1 n'est pas écrit, l'écart n'est pas élucidé ici |

**Commits de remédiation** : `a5387405` (E-P2-1, -2, -3, -4, A2-3), `7ab16821` (B2-1, B2-4, E-P2-5, E-P2-6),
`1c9e4ea0` (B2-2, B2-3, B2-5, Makefile, PDF — **dernier commit de code**).

**Mutations M28 à M39** (journal `27-1-mutations.log`, campagne à `1c9e4ea0`, une par une sur arbre propre,
`git checkout --` + `touch` ; motif d'échec relu dans `27-1-mut/M28-0.out` à `M39-0.out`) : **12 jouées, 12
rouges**, chacune pour la raison attendue — M28 et M30 sur `reconciledCount` (2 ≠ 1), M29 sur le message
(« 2 transaction(s) »), M31 sur « la rejetée est comptée » (3 ≠ 1), M32 et M33 sur « les lignes comptées sous verrou,
non l'entête » (1 ≠ 3), M34 sur « clé read-write, cible 10001 » (404 ≠ 403), M35 sur G9
(`…amount-sign-refused-profile`), M36 et M37 sur le Vitest « pendant l'envoi », M38 sur « se rouvre », M39 sur
« Consultation ». Le script de campagne vérifie désormais tous les motifs avant la première écriture (un premier
lancement de M28, motif ambigu, avait sali `errors.rs` : restauré par `git checkout --` puis `touch` avant toute
compilation). Campagne complète de la story : M1 à M39 moins M6, soit 38 lignes jouées + M14+M15 ; **37 rouges non
équivalentes**, M14 et M15 seules équivalentes.

**Gate complet au HEAD `1c9e4ea0`** (dernier commit de code ; bases `kesh_271` et `kesh_e2e_271` recréées — `DROP`/
`CREATE`, migrations, seed — sans toucher au conteneur, tout sous `flock`, 20:09-20:25) : `scripts/test-fast.sh`
(fmt + clippy `-D warnings` + nextest) **3440 exécutés, 3440 passés, 6 ignorés** (inchangé : la P2 modifie H1, H2, H8
et G9 sans ajouter de test Rust) ; `npm run check` 0 erreur, 28 avertissements préexistants (aucun sur un fichier de
la story) ; `lint-i18n-ownership` PASS ; `npm run test:unit` **132 fichiers, 1353 tests** (+1 fichier, +5 tests depuis
`5881450b` : 2 dans `DeleteBankImportAction.test.ts`, 3 dans `bank-import-detail-page.test.ts`) ; `npm run build`
vert ; **E2E complet** (`/health` `smtpConfigured:true`, aucun `Permission denied` au journal du backend) : **263
passés, 7 échecs, 19 ignorés** — les sept de la KF-029 (#97), `mode-expert.spec.ts:26`, `:41`,
`onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` ; aucun échec hors liste ;
`bank-import-delete.spec.ts:87` passe (sur le nouveau `data-testid`). Avant le gate, gate ciblé
`binary(bank_imports_e2e) or binary(textes_coherents) or binary(bank_accounts_e2e)` : 96/96.

**Trend** : P1 **1 MEDIUM** → P2 **4 MEDIUM** (0 HIGH aux deux). La hausse n'est pas un recyclage au sens du
§ *Règle de splitting* : trois MEDIUM sur quatre viennent de la remédiation P1 (régression de preuve E-P2-2, texte
B2-2) ou d'un test qu'elle laissait faible (E-P2-1), le quatrième (B2-1) est un défaut d'origine neuf. Signal
déclaré au Project Lead, sans découpage (décision D5). La remédiation P2 touche du code de production
(`DeleteBankImportAction.svelte`, `BankImportList.svelte`, quatre catalogues) : la boucle **n'est pas close** —
passe **P3 complète (Sonnet)**, prompt versionné `27-1-review-prompt-p3.md`, axe 9 braqué sur la remédiation P2.
Reclassements : aucun.
