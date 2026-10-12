# Story 27.1 : Supprimer un import bancaire dont aucune transaction n'est rapprochée

## Status

**ready-for-dev** — fiche créée le 2026-10-12 en autonomie (`bmad-create-story`), sur `aadace40` (kickoff de
l'Epic 27), branche `story/27-1-supprimer-import`. Validation P1 à lancer (prompt
`27-1-validate-prompt-p1.md`). Choix consignés au registre `epic-27-choix-autonomes.md`, **C-27-1-1 à
C-27-1-11**. PR : **`closes #632`**.

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
   import_id = ? ORDER BY id FOR UPDATE` (l'index `idx_bank_transactions_import (import_id)` sert le balayage).
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
Angles morts).

**AC3 — La course avec une acceptation est fermée, et un test le prouve dans les deux sens (C-27-1-2).**

Trois tests de dépôt à deux connexions, dans un fichier neuf `crates/kesh-db/tests/bank_import_delete.rs`
(`#[sqlx::test(migrations = "./test-schema")]`), synchronisés par
`kesh_db::test_fixtures::attendre_une_requete_en_cours` (`test_fixtures.rs:561`, jamais par un `sleep`) :

- **(a) l'acceptation d'abord** : une transaction A exécute, sans commit, l'`UPDATE bank_transactions SET
  status = 'reconciled', matched_entry_id = ?, version = version + 1 WHERE id = ? AND company_id = ? AND status =
  'pending' AND version = ?` (le texte de `accept_one_invoice`, `reconciliation.rs:1917-1921`, avec une écriture
  réelle créée par `journal_entries::create`, patron de `reconciliation_cancel.rs:29-60`) ; la suppression est
  lancée dans une tâche ; on attend de la **voir en cours** sur le motif `["bank_transactions"]` (et **non**
  `["bank_transactions", "FOR UPDATE"]` : la mutation M3 remplace la lecture verrouillante par une lecture
  ordinaire, et c'est alors le `DELETE` qui attend — le motif doit la voir aussi) ; A commit ; la suppression rend
  `HasReconciled { reconciled_count: 1, .. }`, et l'import comme ses transactions sont **intacts**.
- **(b) la suppression d'abord** : une transaction B appelle `delete_unreconciled_in_tx` et ne commit pas ; une
  tâche exécute le même `UPDATE` (version lue avant) ; on attend de la voir en cours (motif
  `["UPDATE bank_transactions"]`) ; B commit ; l'`UPDATE` rend `rows_affected() == 0` — c'est ce que les flux
  d'acceptation traduisent déjà en refus (`RECONCILIATION_ALREADY_RECONCILED` dans `failed[]`,
  `reconciliation.rs:1935-1941` ; `OptimisticLockConflict` pour le rapprochement manuel, `:3656-3660`).
- **(c) deux suppressions** : B supprime sans commit ; une seconde suppression est lancée, vue en cours (motif
  `["bank_imports", "FOR UPDATE"]`) ; B commit ; la seconde rend `NotFound`.

**AC4 — La route `DELETE /api/v1/bank-imports/{id}` : montée avec le `POST`, rejouée, inscrite au registre (C-27-1-2, C-27-1-3).**

- Handler `pub async fn delete` dans `crates/kesh-api/src/routes/bank_imports.rs` (nom aligné sur
  `bank_profiles::delete`), `Path(id): Path<i64>`, rend `Result<StatusCode, AppError>` — **`204 No Content`** en
  succès. Corps : `crate::retry::retry_app_on_deadlock("bank_imports::delete", || delete_once(&state.pool,
  &current_user, id))` — patron `complete_import` / `complete_import_once`
  (`routes/imported_supplier_invoices.rs:127-175`). `delete_once` ouvre la transaction, appelle
  `delete_unreconciled_in_tx`, mappe le résultat (AC5), écrit la trace (AC6) **dans la même transaction**, commit.
  Un `id <= 0` rend `404 NOT_FOUND` comme un identifiant inconnu (pas de `400` : le détail `GET` ne distingue pas
  non plus).
- Montage dans **`comptable_routes`** de `crates/kesh-api/src/lib.rs`, **juste après** le `POST
  /api/v1/bank-imports` (`lib.rs:629-633`) :
  `.route("/api/v1/bank-imports/{id}", delete(routes::bank_imports::delete))`. Le `GET` du même chemin reste dans
  `authenticated_routes` (`lib.rs:922-925`) — même partage que `/api/v1/bank-profiles/{id}`. Rôle : Comptable et
  Admin (`require_comptable_role`, `middleware/rbac.rs:36-38`) ; Consultation ⇒ `403`. Le commentaire du bloc
  d'import (`lib.rs:618-621`) est complété : « Story 27-1 : suppression d'un import non rapproché (Comptable+) ».
- **Clés d'API** : ouverte aux clés `read-write` comme le `POST` ; une clé `read` reçoit `403
  API_KEY_READ_ONLY` par `middleware/auth.rs:141-142` (aucun code à écrire). La trace nomme la clé (`for_actor`).
- **Registre des routes** `crates/kesh-api/tests/audit_route_registry.rs` : ligne
  `("delete", "bank_imports::delete", Traced, SansEcritureAuJournal),` après `bank_imports::create` (l. 304).
  Recomptes **à recopier depuis les assertions qui rougissent**, et toutes les mentions de la valeur grepées
  (§ Propagation post-patch) : `LIB_ROUTES.len()` **114 → 115** (l. 636 et prose l. 27, 31) ; registre total
  **117 → 118** (l. 32, l. 657, l. 986) ; `sans_ecriture` **90 → 91** et sa prose « 89 routes de lib.rs » → **90**
  (l. 981-982) ; point (vi) du doc-comment « **six** routes `SansEcritureAuJournal` sont rejouées quand même »
  (l. 126) → **sept**, avec une phrase qui nomme `bank_imports::delete` (enveloppe `AppError`, opération
  `bank_imports::delete`, raison : cycle possible avec un lot d'acceptation qui verrouille plusieurs transactions
  du même import dans l'ordre des propositions — C-27-1-2 ; tenue par sa revue, aucune preuve dynamique). Commande
  de contrôle : `grep -nE "\b(114|115|117|118|89|90|91|six|sept)\b" crates/kesh-api/tests/audit_route_registry.rs`.

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
- Ordre : le rôle (`403`) précède tout ; puis `404` ; puis `409`. Sur `404` et `409` : **aucune écriture**, ni
  suppression ni trace.

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

**AC8 — Rien d'autre ne bouge : vérifié et écrit, pas supposé.**

La fiche a vérifié au sol (Dev Notes § Inventaire (a) et § Effets) et le dev **recontrôle** par `grep` avant de
fermer la story :

- aucune table ne référence `bank_imports` ou `bank_transactions` hors `fk_bank_transactions_import` ; aucune
  colonne d'une autre table n'y pointe sans clé (`audit_log.entity_id` est un pointeur logique documenté
  « survit aux DELETE », squash l. 97) ;
- l'**export global** (`exports/global.rs:294`, `csv_tables.rs:692-772`) et la **sauvegarde** `.keshbackup` lisent
  l'état au moment de l'export : un import supprimé n'y figure plus, sa trace d'audit si ; aucun code à changer,
  aucun rejeu `post_restore` concerné (pas de migration) ;
- la tuile **Relevé du …** de *Comptes bancaires* (`bank_accounts.rs:1011-1017`, dernier import du compte portant
  un solde de clôture) se recalcule sur l'import précédent, ou disparaît s'il n'y en a plus — dit au manuel
  (AC13) ;
- les propositions de réconciliation ne montrent plus les transactions supprimées (elles lisent
  `bank_transactions`, `repositories/reconciliation.rs:178-381`).

**AC9 — L'écran : l'action dans le détail d'un import (C-27-1-6).**

- Helper pur neuf **`frontend/src/lib/features/bank-import/bank-import-deletion.ts`** :
  `export function isReconciled(tx: BankTransactionDto): boolean` (= `tx.status !== 'pending' ||
  tx.matchedEntryId !== null`, doc-comment qui renvoie à `delete_unreconciled_in_tx`) et
  `export function reconciledCount(txs: BankTransactionDto[]): number`.
- Client API : `export async function deleteBankImport(id: number): Promise<void>` dans
  `bank-import.api.ts`, sur `apiClient.delete` (`shared/utils/api-client.ts:578`), chemin
  `/api/v1/bank-imports/${id}`.
- Composant neuf **`frontend/src/lib/features/bank-import/DeleteBankImportAction.svelte`**, props
  `{ detail: BankImportDetailResponse; onDeleted: (filename: string) => void }` :
  - **zéro** transaction rapprochée : bouton `data-testid="bank-import-delete-button"`, variante destructive,
    libellé `bank-import-delete-button` (« Supprimer l'import ») ;
  - **au moins une** : le même bouton, **désactivé** (`disabled`), suivi d'un texte
    `data-testid="bank-import-delete-blocked"` : `bank-import-delete-blocked`, « { $count } transaction(s) de
    cet import sont rapprochées. Pour supprimer l'import, annulez d'abord leur rapprochement (bouton
    « Annuler le rapprochement » de la ligne). » ;
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
  (`<dl>`, l. 79-94), `{#if canManage}<DeleteBankImportAction {detail} onDeleted={…} />{/if}` — `canManage` est
  la dérivée existante (l. 20-22 : Admin ou Comptable). `onDeleted` : `notifySuccess(i18nMsg('bank-import-delete-success',
  "Import { $filename } supprimé.", { filename }))` (`shared/utils/notify.ts:33`) puis `goto('/bank-import')`
  (`$app/navigation`). La liste *Imports précédents* se recharge à son montage.
- Rien d'autre de la page ne change (ses textes en dur restent hors périmètre).

**AC10 — Les textes, dans les quatre langues, et les gardes qui les comptent.**

Dix clés neuves, dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` :

| Clé | Lieu | Français |
|---|---|---|
| `bank-import-errors-has-reconciled-transactions` | serveur, bloc `bank-import-errors-*` (fr l. 1050-1061) | { $count } transaction(s) de cet import sont rapprochées : annulez d'abord leur rapprochement pour pouvoir le supprimer. |
| `audit-log-action-bank-import-deleted` | après `audit-log-action-bank-import-created` (fr 2364, de 2310, it 2311, en 2316) | Import bancaire supprimé |
| `bank-import-delete-button` | frontend | Supprimer l'import |
| `bank-import-delete-blocked` | frontend | (texte de l'AC9) |
| `bank-import-delete-dialog-title` | frontend | Supprimer cet import ? |
| `bank-import-delete-dialog-body` | frontend | (texte de l'AC9) |
| `bank-import-delete-dialog-confirm` | frontend | Supprimer définitivement |
| `bank-import-delete-error-reconciled` | frontend | (texte de l'AC9) |
| `bank-import-delete-error-not-found` | frontend | (texte de l'AC9) |
| `bank-import-delete-success` | frontend | Import { $filename } supprimé. |

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
  transaction_count: 3 }`, tout intact.
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
- **H2** une transaction passée `reconciled` par `UPDATE` de test ⇒ `409`, `error.code =
  BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`, `details.reconciledCount = 1`, `details.transactionCount` = le
  nombre importé ; l'import et ses transactions intacts ; **aucune** trace `bank_import.deleted`.
- **H3** l'import de la société A supprimé par un Comptable de B ⇒ `404` `NOT_FOUND`, l'import de A intact ;
  identifiant inconnu ⇒ `404`.
- **H4** rôle Consultation ⇒ `403`, import intact.
- **H5** second `DELETE` du même identifiant ⇒ `404`.
- Registre des routes (AC4) et garde des libellés d'audit (AC6) verts.

Vitest :

- `bank-import-deletion.test.ts` : `isReconciled` vrai pour `reconciled` sans lien, vrai pour `pending` avec lien,
  faux pour `pending` sans lien (rejetée ou non) ; `reconciledCount` sur une liste mixte.
- `bank-import.api.test.ts` (existant) : `deleteBankImport(42)` appelle `DELETE /api/v1/bank-imports/42`.
- `DeleteBankImportAction.test.ts` (patron `CancelReconciliationDialog.test.ts` : `vi.mock` de l'API et
  d'`i18nMsg` avec interpolation) : bouton actif sans rapprochement ; bouton désactivé et motif `{ count }` avec
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

**AC13 — Le manuel utilisateur dit ce que le code garantit, PDF compris.**

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
  bancaires* reprend l'import précédent du compte, ou disparaît, renvoi `\S\ref{sec:solde-comptable}`) ;
  l'irréversibilité ; et qu'un import d'un compte bancaire **archivé** se supprime mais ne se réimporte plus
  (C-27-1-7).
- **Correction** du paragraphe *Rejeter* (l. 1873) : « aucun geste de l'interface ne permet ensuite de la passer
  en comptabilité » est faux après cette story. Le remplacer par : aucun geste ne la remet dans la liste de la
  réconciliation ; **sauf** à supprimer son import — si aucune transaction de cet import n'est rapprochée — et à
  le réimporter (`\S\ref{sec:supprimer-import}`). Greper le symptôme (`aucun geste`, `sans aucune action`) dans
  tout `docs/` et `website/`.
- **PDF régénéré** : `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock make -C docs/manual user` puis commit du
  `.pdf` ; contrôle sur le PDF **aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '
  | grep -o "Supprimer un import.\{0,200\}"`.
- Manuels DE/EN/IT : **non tenus** (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`, et le `Makefile` n'a
  que `LANGS_AVAILABLE := fr`) — rien à écrire. Manuel d'administration et brochure : non concernés.

**AC14 — API externe, CHANGELOG, README (C-27-1-11).**

- `docs/api-external.md` : ligne neuve du tableau § 7 (l. 207-218) — *Imports bancaires* | `GET /bank-imports`,
  `GET /bank-imports/{id}` | `POST /bank-imports`, `DELETE /bank-imports/{id}` ⁸ ; note ⁸ et sous-section
  `### Supprimer un import bancaire — DELETE /api/v1/bank-imports/{id}` (près de *Annuler un rapprochement
  bancaire*, l. 543) : écriture (`read-write`), rôle Comptable, `204`, définition de « rapprochée » (AC1), `409
  BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` et ses `details`, `404 NOT_FOUND` pour un import inconnu ou d'une autre
  société, trace `bank_import.deleted` nommant la clé, rejeu d'un interblocage transitoire, réimport du même
  fichier sans `confirmDuplicateFile`. Ligne neuve au tableau § 10 (l. 643 et suivantes) pour le `409`. Mentions
  « *depuis la v0.13.2* ». ⛔ Ni « Unreleased » ni « à venir » (garde G6, `textes_coherents.rs:619-640`).
- `CHANGELOG.md` : section **`## [0.13.2] — Non publié`** créée au-dessus de `## [0.13.1] — 2026-10-11`
  (l. 11), forme exacte attendue par `scripts/prepare-release.sh:237` ; `### Ajouté` : une entrée « **Supprimer un
  import bancaire dont aucune transaction n'est rapprochée
  ([#632](https://github.com/guycorbaz/kesh/issues/632)).** … » qui dit où, la condition, le réimport, la trace,
  et la route d'API (`DELETE /api/v1/bank-imports/{id}`, `409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`).
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
| M1 | prédicat Rust sans la branche `status <> 'pending'` | D2, D4, H2 |
| M2 | prédicat Rust sans la branche `matched_entry_id IS NOT NULL` | D3 |
| M3 | lecture des transactions **sans** `FOR UPDATE` | AC3 (a) |
| M4 | `SELECT` de l'entête sans `AND company_id = ?` (et `DELETE` idem) | D5, H3 |
| M5 | trace d'audit retirée | H1 |
| M6 | trace écrite avant la garde (donc aussi sur refus) | H2 |
| M7 | `409` remplacé par `422` | H2 |
| M8 | route montée dans `authenticated_routes` | H4 |
| M9 | `"bank_import.deleted"` retiré d'`ACTIONS` | `audit_label_registry` |
| M10 | ligne du registre des routes retirée | `audit_route_registry` |
| M11 | `isReconciled` sans la branche `matchedEntryId` | `bank-import-deletion.test.ts` |
| M12 | le dialogue appelle `deleteBankImport` avec un autre identifiant | `DeleteBankImportAction.test.ts` |
| M13 | garde `submitting` retirée | `DeleteBankImportAction.test.ts` (double clic) |
| M14 | `FOR UPDATE` de l'entête retiré | AC3 (c) — **équivalente probable** : le contrôle `rows_affected() != 1` rend `NotFound` à la seconde suppression ; à constater et écrire, non à supposer |
| M15 | contrôle `rows_affected() != 1` retiré | AC3 (c) — équivalente si M14 ne l'est pas ; les deux ensemble doivent rougir |

## Tasks / Subtasks

- [ ] **T0 — Montage** (consignes communes des agents) : base `kesh_27_1` et `kesh_e2e_27_1` créées par l'agent
  (migrations du worktree, seed `scripts/seed-dev-db.sql`), `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-27-1/target`,
  `node_modules` par `cp -al` depuis le dépôt principal, port attribué par l'orchestrateur ; gates lourds sous
  `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`.
- [ ] **T1 — Dépôt** (AC1, AC2)
  - [ ] T1.1 `DeleteBankImportOutcome`, `DeletedBankImport`, `delete_unreconciled_in_tx` dans
    `crates/kesh-db/src/repositories/bank_imports.rs`, doc-comments (définition, ordre des verrous, cascade, angles
    morts) ; corriger au passage le doc-comment de module (l. 10-12, qui annonce encore un `409`), qui parle encore de la contrainte UNIQUE
    retirée en 8-3.
  - [ ] T1.2 Tests D1-D6 et AC3 (a)(b)(c) dans `crates/kesh-db/tests/bank_import_delete.rs`.
- [ ] **T2 — Route** (AC4, AC5, AC6)
  - [ ] T2.1 `AppError::BankImportHasReconciledTransactions` et sa réponse dans `errors.rs`.
  - [ ] T2.2 `delete` + `delete_once` dans `routes/bank_imports.rs` (doc-comment de module l. 1-11 complété).
  - [ ] T2.3 Montage `lib.rs` (comptable_routes, après l. 633).
  - [ ] T2.4 `audit_labels.rs` (`ACTIONS`).
  - [ ] T2.5 `audit_route_registry.rs` : ligne et recomptes (AC4).
  - [ ] T2.6 Tests H1-H5.
- [ ] **T3 — i18n serveur et frontend** (AC10) : dix clés × quatre catalogues.
- [ ] **T4 — Écran** (AC9)
  - [ ] T4.1 `bank-import-deletion.ts` + tests.
  - [ ] T4.2 `deleteBankImport` + test.
  - [ ] T4.3 `DeleteBankImportAction.svelte` + tests.
  - [ ] T4.4 Page détail : insertion, notification, retour à la liste.
  - [ ] T4.5 `sitesTotal` recompté (AC10), `lint-i18n-ownership`.
- [ ] **T5 — E2E** (AC12) : fixture, spec.
- [ ] **T6 — Documentation** (AC13, AC14) : manuel `.tex` + PDF (sous `flock`), `api-external.md`, `CHANGELOG.md`,
  `README.md` ; symptôme du paragraphe *Rejeter* grepé dans tout le dépôt.
- [ ] **T7 — Mutations** (AC16), sous `flock`.
- [ ] **T8 — Gates au dernier commit de code** : base de gate remise à zéro **sans toucher au conteneur**
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
(dernier relevé, lit `bank_imports` — AC8) ; `exports/global.rs:294` et `exports/csv_tables.rs:692-772` (export) ;
`repositories/reconciliation.rs:178,301,347,379` (propositions, `pending`).

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

(Les quatre lignes de `invoices.rs` au-delà de la l. 2984 sont dans son `mod tests` — `#[cfg(test)]` l. 2984-2985.)
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
`invoices.rs:1608`) — vérifié par le `grep` ci-dessus, qui ne rend aucun autre écrivain. La restauration
d'installation (`admin_backup`, `kesh-db/src/backup.rs`) réécrit les tables en bloc : elle n'est pas un chemin de
rapprochement et elle est exclusive des autres écrivains (Story 15-12b). Aucun déclencheur SQL dans les migrations
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
  d'exercice, pas de compte. L'angle mort (iv) du registre (victime d'un cycle avec un flux qui écrit au journal)
  ne s'y présente pas par ce qu'elle verrouille.

### Angles morts assumés (à écrire au doc-comment de `delete_unreconciled_in_tx`)

- **Un import créé pendant la suppression** n'est pas concerné (autre identifiant) ; **deux imports du même
  fichier** (créés avec `confirmDuplicateFile`) se suppriment un par un — la story ne supprime pas « le fichier »,
  elle supprime **un** import.
- **Le rejet concurrent** d'une transaction (`reject_batch`) contre la suppression : l'un attend l'autre ; si la
  suppression passe d'abord, l'`UPDATE` du rejet ne trouve plus la ligne. Son traitement de `rows_affected() == 0`
  est celui du code existant (`reconciliation.rs:3218-3225`), non modifié par la story.
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
  change → une sauvegarde prise avant ou après une suppression s'importe à l'identique.
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
page, une spec E2E), documentation. **Cinq ensembles, un seul domaine** (l'import bancaire), sans changement de
règle transverse : pas au-delà du seuil « plus de 5 modules », et la story est une tranche verticale qu'un
découpage laisserait sans écran entre deux fusions. **Repli** si la validation recycle un défaut : **27-1a**
(T1, T2, i18n serveur, `api-external.md` — `refs #632`) puis **27-1b** (T4, T5, T6 hors `api-external.md`, i18n
frontend — `closes #632`). Signal déclaré à l'orchestrateur.

### Project Structure Notes

- Fichiers neufs : `crates/kesh-db/tests/bank_import_delete.rs` ; `frontend/src/lib/features/bank-import/
  {bank-import-deletion.ts, bank-import-deletion.test.ts, DeleteBankImportAction.svelte,
  DeleteBankImportAction.test.ts}` ; `frontend/tests/e2e/bank-import-delete.spec.ts` ;
  `frontend/tests/e2e/fixtures/csv_e2e_delete_import.csv`.
- Fichiers modifiés : `crates/kesh-db/src/repositories/bank_imports.rs` ; `crates/kesh-api/src/{errors.rs, lib.rs,
  audit_labels.rs, routes/bank_imports.rs}` ; `crates/kesh-api/tests/{bank_imports_e2e.rs, audit_route_registry.rs}` ;
  `crates/kesh-i18n/locales/*/messages.ftl` ; `frontend/src/lib/features/bank-import/{bank-import.api.ts,
  bank-import.api.test.ts}` ; `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` ;
  `frontend/src/lib/shared/i18n-keys.test.ts` ; `docs/manual/fr/user-manual.{tex,pdf}` ; `docs/api-external.md` ;
  `CHANGELOG.md` ; `README.md`.
- Aucun fichier de `crates/kesh-db/migrations/`, `test-schema/`, `post_restore.rs` (AC15).

### References

- Issue #632 ; epic `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md`.
- `CLAUDE.md` : § *Test Locally First*, § *Review Iteration Rule* (et ses sous-sections : manuel, axes déclarés,
  sites non résolus, propagation post-patch, recompte), § *Pattern batch* (non applicable : route unitaire),
  § *Migration breaking policy* (non applicable, AC15), § *Issue Tracking Rule*.
- Registre `epic-15-choix-autonomes.md` : C54-C59 (rejeu) ; registre `epic-27-choix-autonomes.md` : C-27-1-1 à 11.
- Code : `kesh-reconciliation/src/mutex.rs` ; `kesh-api/src/retry.rs` ; `kesh-db/src/retry.rs:100-160` ;
  `kesh-db/src/test_fixtures.rs:561-605` ; `kesh-db/tests/reconciliation_cancel.rs:127-185` (patron de course).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-12** — Fiche créée (`bmad-create-story`, Opus 5.5, en autonomie) sur `aadace40`. 16 AC, 9 tâches
  (T0-T8). Choix C-27-1-1 à C-27-1-11 au registre. Prémisse « unicité du hash » de l'epic et de l'issue corrigée
  (C-27-1-8). Validation P1 à lancer : `27-1-validate-prompt-p1.md` (lentilles R et F).
