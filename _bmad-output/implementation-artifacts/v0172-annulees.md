# Story v0172-annulees : la 0.17.2 — repasser en brouillon une facture fournisseur annulée (#868)

Status: ready-for-dev

*Fiche rédigée le 2026-11-04, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0172a`, branche
`story/v0172-annulees` partie de `d7ec2ea2` (le merge de la v0.17.1 sur `main`). Release **0.17.2**, **sans migration**
(C-v0172a-1). Registre : `v0172-choix-autonomes.md`, section `v0172-annulees` (C-v0172a-1 à 18). Clé courte : `v0172a`.
Issue : **#868** (enhancement, reformulée par la décision du Project Lead du 2026-11-04), en `refs` seulement dans les
commits ; la fermeture se porte sur la PR de release. Les numéros de ligne sont ceux de `d7ec2ea2`.*

## Story

En tant que Comptable qui a **annulé** une facture fournisseur saisie par erreur (doublon, facture d'essai, mauvaise
saisie faite avant que le cycle brouillon → validée → payée n'existe), je veux pouvoir la **repasser en brouillon** —
ce qui retire du journal son écriture d'achat **et** sa contre-passation, sous les mêmes gardes que la dévalidation
d'une facture validée — pour la corriger et la revalider, ou, administrateur, la supprimer comme tout brouillon ; et,
devant une facture qui ne peut pas encore repasser en brouillon, je veux que Kesh me dise **quoi faire d'abord**.

## Origine et contrainte

- **Décision du Project Lead (2026-11-04)**, à la lettre : « Je propose de ne supprimer que les factures "brouillon" :
  donc, pour supprimer une facture, il faut d'abord la repasser en mode brouillon. Ceci permet aussi de ré-éditer des
  factures. » Il a accepté la proposition en quatre points du mandat (`kesh-gate-logs/prompts/v0172-annulees-fiche.md`) :
  (1) annulée → brouillon ; (2) payée ou rapprochée : le refus reste, le chemin passe par l'annulation du règlement ou du
  rapprochement ; (3) le geste se trouve et ses refus disent quoi faire d'abord ; (4) factures clientes : constater, ne
  rien changer.
- **#868 telle qu'écrite** demandait de « supprimer réellement » une annulée ; la décision la **reformule** : on ne
  supprime qu'un brouillon (`delete_draft`, inchangé) ; l'annulée y arrive par « Repasser en brouillon ». Les quatre
  points de conception de l'issue sont tranchés ici : piste d'audit (§ *Piste d'audit*), sort du document (C-v0172a-12),
  trous de numérotation (deux trous, dits au manuel), sauvegarde (aucune migration, le `.keshbackup` d'avant restaure).
- **Production** : Kesh 0.17.1 tient des livres réels sur le NAS. **Aucune migration, aucune donnée réécrite par la
  mise à jour** : le geste n'agit que sur la facture que l'utilisateur désigne, à sa demande. L'inventaire des tables est
  inchangé ; `kesh_version_min_required` reste à 0.17.0 ; le retour à la 0.17.1 reste possible sans restauration — mais
  une facture repassée en brouillon sous la 0.17.2 **reste brouillon** sous la 0.17.1 (état que la 0.17.1 connaît). Aucun
  essai ne se joue sur le NAS : bases `#[sqlx::test]` jetables, base de démonstration jetable pour les captures.
- **Hors périmètre** (release, faits par l'orchestrateur) : bump de version Cargo, macros des manuels, tag.

## Ce que le code dit (relevé le 2026-11-04, `d7ec2ea2`)

### 1. La dévalidation d'aujourd'hui (`supplier_invoices::unvalidate_in_tx`, `supplier_invoices.rs:909-1035`)

- Verrou de la facture `FOR UPDATE` (`:918`) ; statut (`:935-942`) : `open` passe, `paid` →
  `SupplierInvoiceNotUnvalidatable { blocker: Paid }`, **tout autre statut — brouillon et annulée — →
  `IllegalStateTransition`** (« seule une facture validée se dévalide ») ; version (`:944`) ; lot `generated` en lecture
  verrouillante (`:950-965`, `InPaymentBatch`, `document_id` = lot) ; écriture d'achat rapprochée (`:966-978`,
  `MatchedBankTransaction`, `document_id` = transaction) ; instantané (`fetch_lines`, lecture ordinaire, écart assumé
  `:898-906`) ; **un seul `UPDATE`** `status='draft', purchase_journal_entry_id=NULL` gardé `status='open'` (`:983-998`,
  contrainte `chk_supplier_invoices_entry_iff_not_draft`) ; puis `journal_entries::delete_in_tx(…, enforce_ownership =
  false)` (`:1001`) ; audit `supplier_invoice.unvalidated` `{supplierInvoice, purchaseJournalEntryId}` (`:1002-1015`).
- **Le document rattaché n'est pas touché** : aucune ligne de `unvalidate_in_tx` ne lit ni n'écrit
  `imported_supplier_invoices` ; le document reste joint au brouillon et ne revient « à traiter » qu'à `delete_draft`
  (`:758-842`, `detach_in_tx` `:787`). C'est la règle reprise pour l'annulée (C-v0172a-12).
- Route `POST /api/v1/supplier-invoices/{id}/unvalidate` `{version}` (`routes/supplier_invoices.rs:894-913`), Comptable+,
  rejouée sur interblocage (`retry_on_deadlock("supplier_invoices::unvalidate", …)`).

### 2. Ce que l'annulation a écrit (`supplier_invoices::cancel_in_tx`, `:2045-2210`)

- Contre-passation de l'écriture d'achat par le socle `journal_entries::reverse_owned_in_tx` (autorité
  `SupplierPurchase`) : écriture inverse **datée du jour de l'annulation**, dans l'exercice ouvert de ce jour, liée par
  `reverses_entry_id` (FK `RESTRICT`, migration `20260828000002_journal_entries_reversal.sql:47`).
- **Lettrage** (`journal_entries.rs:2964-3010`) : chaque ligne **libre** de l'achat sur un compte lettrable
  (`Asset`/`Liability` non bancaire, `letterings.rs:363`) — la dette 2000 au crédit, l'impôt préalable au débit — forme
  avec son miroir un groupe d'origine **`reversal`**, en mode `System` tenu par l'exercice de la contre-passation. Une
  ligne déjà lettrée garde son groupe, son miroir reste ouvert. La dissolution **manuelle** d'un tel groupe est refusée
  quand une ligne appartient à une pièce (`letterings.rs:1240-1250`, refus 2, test
  `supplier_invoice_cancel_letters_a_pair_that_cannot_be_dissolved_by_hand`).
- Facture **payée** au moment de l'annulation : le groupe `document` {achat, règlement} est dissous **avant** la
  contre-passation (`:2121-2133`) ; le règlement **reste au grand livre, détaché** (colonnes `settlement_*` et `paid_at`
  à `NULL`, `:2146-2163`) ; seule trace du lien : l'audit `supplier_invoice.cancelled`,
  `details.settlementJournalEntryId` (`:2178-2200`).
- `status = 'cancelled'`, `purchase_journal_entry_id` **conservé** (l'annulée garde son achat).

### 3. La suppression d'écriture (`journal_entries::delete_in_tx`, `journal_entries.rs:1822-1980`) — inchangée

Premier acte : écriture et exercice `FOR UPDATE` (`:1833-1845`) ; 2-bis exercice postérieur clos verrouillé
(`:1858`) ; 3 exercice clos → `FiscalYearClosed` ; 2-bis verdict → `LaterFiscalYearClosed` ; **3-bis écriture
contre-passée → `EntryIsReversed`** (`:1887-1889`, « art. 958f CO ») ; 3-ter (seulement `enforce_ownership`) garde de la
modification ; 3-quater verrou de période → `PeriodLocked` (`:1912-1925`) ; 3-quinquies **ligne lettrée** →
`EntryNotModifiable` (`ENTRY_LETTERED`, `:1932-1934`) ; instantané, audit `journal_entry.deleted`, `DELETE` (lignes en
CASCADE). ⇒ **La contre-passation se supprime d'abord** : elle n'est elle-même contre-passée par rien, et une fois
partie, `reversed_by(achat)` (lecture dans la même transaction) ne rend plus rien et l'étape 3-bis de l'achat passe. Dans
l'ordre inverse, la FK `RESTRICT` et l'étape 3-bis refuseraient. **Et les groupes `reversal` se dissolvent avant**, sinon
l'étape 3-quinquies refuse l'une et l'autre.

### 4. Le paiement détaché et son gel (`journal_entries::modification_guard`, `:1066-1105`)

Le gel du paiement détaché (palliatif de #541) est une **lecture d'audit** jointe à la facture **par son statut** :
`… JOIN audit_log al ON … al.action = 'supplier_invoice.cancelled' … WHERE si.company_id = ? AND si.status = 'cancelled'
AND JSON_VALUE(al.details_json, '$.settlementJournalEntryId') = ?` (`:1086-1092`). ⚠️ **Repasser en brouillon une
annulée qui avait été payée lèverait ce gel en silence** : la facture n'est plus `cancelled`, la jointure ne la trouve
plus, et le paiement détaché redeviendrait modifiable et supprimable par `DELETE /journal-entries/{id}` — alors que
l'argent est réellement sorti. D'où le motif `DetachedPayment` (C-v0172a-5, AC-6).

### 5. Le chemin d'une payée (point 2 du mandat) — vérifié au code, inchangé

- Payée depuis la fiche, règlement **non** rapproché : *Annuler le règlement* (`cancel_settlement_in_tx`, `:2318-2485`)
  contre-passe le règlement, ramène la facture à `open` ; puis dévalidation. Test existant
  `paid_then_settlement_cancelled_then_unvalidated` (`crates/kesh-db/tests/supplier_invoices_draft.rs:986`).
- Payée **par la réconciliation** (acceptation d'une proposition « facture fournisseur », Story 30-1b) ou rattachée à une
  transaction (v017-3a) : l'annulation du règlement depuis la fiche est refusée (`MATCHED_BANK_TRANSACTION`, queue
  commune `settlement_entry_cancel_blocker`) ; *Annuler le rapprochement* (`reconciliation_cancel::cancel_in_tx`, famille
  `SupplierSettlement`, `reconciliation_cancel.rs:38-48`) appelle `supplier_invoices::cancel_settlement_in_tx` et ramène
  la facture à `open`. Test route existant `an_accepted_supplier_invoice_is_undone_by_cancelling_the_reconciliation`
  (`crates/kesh-api/tests/reconciliation_e2e.rs:11369`). **Aucun test ne prolonge ce parcours jusqu'au brouillon puis à la
  suppression** : AC-10 l'ajoute.
- ⚠️ Le refus `Paid` d'aujourd'hui dit « annulez d'abord son règlement » **même quand ce règlement est rapproché**, cas où
  la fiche refusera ensuite l'annulation du règlement : le message envoie l'utilisateur vers un geste refusé. D'où
  `PaidReconciled` (C-v0172a-6).

### 6. L'écran (`frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`)

- Bouton *Dévalider* seulement pour `status === 'open'` et `canManage` (`:650-679`) ; rien pour `paid` ; pour
  `cancelled`, la seule phrase « Facture annulée. » (`:922-932`). Le Project Lead ignorait que la dévalidation existe :
  le libellé ne dit pas ce que fait le geste.
- Refus rendu après le clic, hors des blocs de statut (`:695-699`), par `supplierInvoiceUnvalidateMessage(code, label)`
  (`frontend/src/lib/features/supplier-invoices/supplier-invoice-unvalidate.ts`), qui suffixe `label` (le **numéro de la
  facture**, `details.documentNumber`) aux trois motifs propres à la pièce — il ne nomme ni le lot ni la transaction.
- Confirmation `supplier-invoices-unvalidate-confirm-*` (`:991-1027`) ; succès `supplier-invoices-unvalidated`.

### 7. Factures clientes (point 4 du mandat) — l'écart existe, rien ne change

`invoices::unvalidate` (`invoices.rs:1503-`) n'accepte que `status = 'validated'` (`:1520-1526`) ; une facture cliente
`cancelled` → `IllegalStateTransition`. Mais une facture cliente n'est **annulée que par un avoir**
(`credit_notes.rs:772`, seul `UPDATE … status = 'cancelled'` de production) : document numéroté, avec sa propre
écriture, souvent **envoyé au client**, et refusé d'ailleurs par le motif `Credited` de la dévalidation (`:1556-1569`).
L'analogue de la contre-passation fournisseur n'existe pas côté client. **Écart constaté, non comblé** (C-v0172a-14) ;
une issue est demandée à l'orchestrateur pour le tracer (§ *Pour l'orchestrateur*).

## Inventaire clos — qui écrit `cancelled` ou le lit pour décider (factures fournisseur)

Commande : `git grep -nE "cancelled" -- 'crates/*/src/*' 'frontend/src/*'`, trié à la main (lots de paiement, factures
clientes, règlements et `DocumentPreview` écartés). Chaque site, et ce qu'il devient quand une annulée redevient brouillon.

| # | Site (`d7ec2ea2`) | Rôle | Après « annulée → brouillon » |
|---|---|---|---|
| 1 | `supplier_invoices.rs:2149` `cancel_in_tx` | **seul écrivain** de `cancelled` | inchangé ; un brouillon issu d'une annulée se revalide puis s'annule de nouveau comme un autre |
| 2 | `supplier_invoices.rs:1993` `supplier_invoice_cancel_blocker` | coupe court « déjà annulée » | inchangé ; le brouillon répond `SupplierInvoiceDraft` (rang 1) |
| 3 | `supplier_invoices.rs:935-942` `unvalidate_in_tx` | refuse `cancelled` | **modifié** : `cancelled` passe (AC-1) |
| 4 | `supplier_invoices.rs:771-778` `delete_draft_in_tx` | refuse `cancelled` (`IllegalStateTransition`) | inchangé : l'annulée ne se supprime pas directement — d'abord brouillon (décision du Project Lead) |
| 5 | `journal_entries.rs:1086-1092` `modification_guard` (paiement détaché) | gèle le règlement détaché d'une annulée | **protégé par refus** : une annulée dont le paiement détaché existe non contre-passé ne repasse pas en brouillon (AC-6) ; contre-passé, il est gardé par l'étape 3-bis |
| 6 | `letterings.rs:1847-1900` `discover_supplier_invoice_document` | `cancelled` → aucune pièce | non atteint : le brouillon n'est pas soumis au lettrage de pièce (les trois appelants refusent un brouillon avant) |
| 7 | `contacts.rs:401` `find_suppliers_by_invoice_iban` | exclut les annulées des fournisseurs suggérés | le brouillon y revient — voulu : ce n'est plus une facture écartée |
| 8 | `reconciliation.rs:825, 885, 935` propositions | ne lisent que `open` | inchangé (ni annulée ni brouillon proposées) |
| 9 | `closing_checks.rs:504` contrôle K8 | brouillons datés dans l'exercice | le brouillon y apparaît — voulu (il n'est plus aux livres) |
| 10 | `supplier_invoices.rs:212, 218-279` `list` + filtre de la 0.17.0 (`routes/supplier_invoices.rs:644`, case « Afficher les factures annulées », `frontend/src/routes/(app)/supplier-invoices/+page.svelte:113`) | masque les annulées par défaut | le brouillon apparaît dans la liste par défaut |
| 11 | `imported_supplier_invoices.rs:379-400` `find_source_document_of_entry` | le document de l'achat d'une annulée | l'achat supprimé, plus rien à montrer ; le document reste joint au brouillon (C-v0172a-12) |
| 12 | `crates/kesh-api/src/exports/csv_tables.rs:1233` | exporte le statut | exporte `draft` ; aucune règle |
| 13 | `audit_labels.rs:194, 199` | libellés `supplier_invoice.cancelled` / `.unvalidated` | `.unvalidated` réemployé (C-v0172a-10) ; aucun libellé neuf |
| 14 | `frontend/…/supplier-invoices/[id]/+page.svelte:922-940` | bloc « Facture annulée. », bloc d'annulation masqué | **modifié** : bouton « Repasser en brouillon » (AC-12) |
| 15 | `frontend/…/SupplierInvoiceTotal.svelte:30`, `supplier-invoice-helpers.ts:46`, `invoice-cancel.ts:50` | affichage | inchangés |
| 16 | Sauvegarde / `post_restore.rs` | — | aucun : pas de migration, pas de backfill, inventaire des tables identique ; un `.keshbackup` d'avant le geste restaure l'annulée et ses deux écritures |
| 17 | Rapports (`kesh-report`), grand livre, TVA | lisent les écritures | la paire achat + contre-passation était nette à zéro sur chaque compte : la retirer ne change **aucun solde** ; le verrou de période protège les trimestres déclarés (AC-4) |

`git grep -nE "si\.status|status\s*=\s*'cancelled'" -- 'crates/*/src/*'` ne rend aucun autre site fournisseur (relevé :
`contacts.rs:401`, `closing_checks.rs:504`, `journal_entries.rs:1090`, `reconciliation.rs:825,885,935`, `supplier_invoices.rs:2149`).

## Inventaire clos — `SupplierUnvalidationBlocker` et les refus de « Repasser en brouillon »

Après la story, l'énumération (`crates/kesh-db/src/errors.rs:329-352`) compte **cinq** variantes ; les refus qui ne sont
pas des variantes sont listés à la suite. **Tout refus dit quoi faire d'abord.** Les textes fr-CH ci-dessous sont les
**replis** ; les quatre locales reçoivent leur traduction.

| Rang | Refus | Code | Quand | Message (fr-CH) — ce qu'il faut faire d'abord |
|---|---|---|---|---|
| 0 | `NotFound` | `NOT_FOUND` | — | inchangé |
| 1 | `IllegalStateTransition` | `ILLEGAL_STATE_TRANSITION` | brouillon | inchangé : « … n'est plus validée : rechargez la page… » → reformulé « Cette facture est déjà un brouillon : rechargez la page pour voir son état. » |
| 1 | `Paid` (existant) | `SUPPLIER_INVOICE_PAID` | `paid`, règlement **non** rapproché | « Cette facture fournisseur est payée : annulez d'abord son règlement (bouton *Annuler le règlement* de cette fiche), puis repassez-la en brouillon. » |
| 1 | **`PaidReconciled`** (neuf) | `SUPPLIER_INVOICE_PAID_RECONCILED` | `paid`, écriture de règlement rapprochée | « Cette facture est payée par la transaction bancaire du { $date } : annulez d'abord ce rapprochement (Réconciliation, *Annuler le rapprochement*) — elle redeviendra validée —, puis repassez-la en brouillon. » |
| 2 | `OptimisticLockConflict` | `OPTIMISTIC_LOCK_CONFLICT` | version périmée | inchangé |
| 3 | **`DetachedPayment`** (neuf) | `SUPPLIER_INVOICE_DETACHED_PAYMENT` | `cancelled`, l'audit de l'annulation nomme un règlement qui existe et n'est pas contre-passé | non rapproché : « Cette facture a été annulée alors qu'elle était payée : son paiement (écriture n° { $entry }) reste au grand livre. Pour la repasser en brouillon, contre-passez d'abord ce paiement depuis sa fiche — seulement s'il n'a pas eu lieu. » ; rapproché : « … son paiement (écriture n° { $entry }) est rapproché de la transaction bancaire du { $date } : annulez d'abord ce rapprochement, qui contre-passera le paiement — seulement s'il n'a pas eu lieu. » |
| 4 | `InPaymentBatch` (existant) | `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` | lot `generated` | « Cette facture figure dans le lot de paiement n° { $batch }, en cours : annulez d'abord ce lot (Fournisseurs, *Paiements*). » |
| 5 | `MatchedBankTransaction` (existant) | `MATCHED_BANK_TRANSACTION` | écriture d'achat (ou contre-passation) rapprochée | « L'écriture d'achat de cette facture est rapprochée de la transaction bancaire du { $date } : annulez d'abord ce rapprochement. » |
| 6 | `FiscalYearClosed` | `FISCAL_YEAR_CLOSED` | exercice de l'achat ou de la contre-passation clos | inchangé (« …un administrateur doit rouvrir les exercices clôturés… ») |
| 7 | `LaterFiscalYearClosed` | `LATER_FISCAL_YEAR_CLOSED` | exercice postérieur clos | inchangé (nomme l'exercice) |
| 8 | `EntryIsReversed` | `ENTRY_IS_REVERSED` | **inatteignable** (la contre-passation part d'abord ; une contre-passation ne se contre-passe pas) | inchangé, gardé comme au serveur |
| 9 | `PeriodLocked` | `PERIOD_LOCKED` | date de l'achat ou de la contre-passation ≤ verrou | inchangé (« …faire reculer le verrou… ») |
| 10 | `EntryNotModifiable` (`ENTRY_LETTERED`) | `ENTRY_LETTERED` | une ligne lettrée **hors** des paires `reversal` de l'annulation | inchangé (« …délettrez-la d'abord… ») |
| — | `Invariant` | 500 | annulée sans contre-passation liée (`reversed_by` vide), facture validée sans achat | C-v0172a-16 |

⚠️ Les rangs 1 à 5 sont ceux du dépôt, dans cet ordre ; les rangs 6 à 10 restent ceux de `delete_in_tx` (pré-contrôle du
rang 6 compris, C-v0172a-4). **Limite L1** : les messages 6, 9 et 10 parlent de « l'écriture d'achat » ; pour une annulée,
le refus peut venir de la contre-passation — seulement quand l'achat est daté **après** son annulation (facture saisie
avec une date future), cas où le message reste juste sur le geste à faire et inexact sur l'écriture nommée. Écrit, non
comblé (C-v0172a-15).

## Piste d'audit — pourquoi supprimer une écriture contre-passée reste acceptable ici

*Lecture de Kesh, non avis juridique ; le manuel la présente comme telle et renvoie à la fiduciaire.*

1. **Ce que le droit demande.** CO 957a al. 2 ch. 5 : une comptabilité **vérifiable** ; Olico art. 3 : des livres tenus
   de façon à ne pas pouvoir être modifiés **sans que cela puisse être constaté** ; CO 958f et Olico art. 9 : conservation
   et intégrité des supports modifiables. Le principe n'est pas « rien ne disparaît », c'est « rien ne disparaît sans
   trace » — Kesh l'applique déjà à la suppression d'une écriture (v0.13.0) et à la dévalidation (v0.12.1, fournisseur
   v0.17.0), décrites au manuel d'administration (§ *Conformité OLICo Art. 9*, `admin-manual.tex:3057`).
2. **Pourquoi l'étape 3-bis de `delete_in_tx` (« une écriture contre-passée ne se supprime pas, art. 958f ») ne
   s'applique pas.** Elle protège la **correction** : supprimer l'origine en gardant l'inverse, ou l'inverse en gardant
   l'origine, laisserait aux livres une erreur sans sa correction ou une correction sans objet. Ici les **deux**
   disparaissent ensemble, dans une transaction : le grand livre redevient ce qu'il était avant la saisie fautive, chaque
   compte au même solde (la paire était nette à zéro). `delete_in_tx` n'est pas modifiée ; c'est l'ordre du geste
   (contre-passation d'abord) qui rend l'étape 3-bis sans objet pour l'achat.
3. **Ce qui borne le geste.** Exercice ouvert pour l'une et l'autre écriture, aucun exercice postérieur clos, aucune date
   sous le verrou de période (trimestre TVA déclaré), aucune écriture rapprochée, aucun lettrage hors de la paire, aucun
   paiement réel laissé sans gel (AC-6), un lot pain.001 en cours refusé ; **confirmation explicite** qui dit les deux
   suppressions et les deux trous.
4. **Ce qui reste.** Journal d'audit, insert-only : `lettering.removed` pour chaque paire dissoute,
   `journal_entry.deleted` ×2 avec l'instantané complet de chaque écriture, `supplier_invoice.unvalidated` avec l'état de
   la facture, `previousStatus`, les deux identifiants d'écriture ; l'audit `supplier_invoice.cancelled` d'origine n'est
   pas touché. Les numéros d'écriture ne sont jamais réattribués : deux trous, expliqués par l'audit. La sauvegarde
   `.keshbackup` d'avant le geste restaure l'annulée et ses écritures.
5. **Ce que dit le manuel aujourd'hui, et qui change** : « La 0.17.0 ne supprime aucune facture annulée »
   (`user-manual.tex:1892`) et « Annulez une facture qui n'était pas due : l'achat et sa contre-passation restent aux
   livres » (`:1881-1882`). Les deux restent vrais **par défaut** — annuler laisse la trace ; repasser en brouillon est
   un second geste, délibéré, réservé à ce qui n'aurait jamais dû être saisi. Le texte est récrit en ce sens (§ *Manuel*).

## Acceptance Criteria

**AC-1 — Une annulée repasse en brouillon.** `supplier_invoices::unvalidate_in_tx` accepte `status = 'cancelled'` (en plus
de `open`). Effet, dans **une** transaction : la facture devient `draft`, `purchase_journal_entry_id = NULL`,
`version + 1` ; l'écriture d'achat **et** sa contre-passation (`reversed_by(achat)`) sont supprimées, lignes comprises ;
les groupes de lettrage `reversal` qui les unissaient sont dissous ; les lignes de la facture, son numéro, son
fournisseur, ses dates, son projet sont inchangés. Test `kesh-db` `cancelled_invoice_returns_to_draft` (T1) : facture
validée à deux lignes (dont une avec TVA), annulée, puis repassée en brouillon ; asserte statut, colonnes, absence des
deux écritures **et de leurs lignes**, `lettering_key IS NULL` partout où il était, soldes des comptes 2000, impôt
préalable et charge **identiques** avant annulation et après le geste.

**AC-2 — Le brouillon obtenu vit comme tout brouillon.** Il se modifie (`update_draft`), se revalide (`validate` passe une
écriture d'achat neuve) et, administrateur, se supprime (`delete_draft`, son document rendu « à traiter »). Test T2
`a_cancelled_then_redrafted_invoice_is_edited_validated_then_deleted` (deux variantes : revalidée ; supprimée avec un
document joint → document `to_complete`).

**AC-3 — Ordre et socle.** La contre-passation est supprimée **avant** l'achat, toutes deux par
`journal_entries::delete_in_tx(…, enforce_ownership = false)` — aucun `DELETE FROM journal_entries` écrit dans
`supplier_invoices.rs`. `git grep -nF "DELETE FROM journal_entries" -- crates/kesh-db/src/repositories/supplier_invoices.rs`
rend **zéro ligne** (contrôle déclaré au Dev Agent Record). `delete_in_tx` n'est pas modifiée hors de son doc-comment
(liste des appelants `enforce_ownership = false`).

**AC-4 — Mêmes gardes que la dévalidation, sur les deux écritures.** Refusent, sans rien écrire (facture `cancelled`,
version, deux écritures, lettrage et audit inchangés — asserté) :
- (a) exercice de l'**achat** clos, contre-passation dans un exercice ouvert (le cas courant : facture 2025 annulée en
  2026 après la clôture de 2025) → `FISCAL_YEAR_CLOSED` (T3a) ;
- (b) exercice postérieur clos → `LATER_FISCAL_YEAR_CLOSED` (T3b, nomme l'exercice) ;
- (c) **contre-passation** sous le verrou de période, achat hors verrou (achat daté après son annulation) →
  `PERIOD_LOCKED` (T3c) ; (c') achat sous le verrou → `PERIOD_LOCKED` (T3c') ;
- (d) une ligne de l'achat ou de la contre-passation lettrée **hors** d'une paire `reversal` de l'annulation (état posé
  en SQL dans le test, la route ne le produisant pas — dit au commentaire du test) → `ENTRY_LETTERED` (T3d) ;
- (e) achat ou contre-passation rapprochés d'une transaction (lien posé par `fournisseur_rapproche::debit_rapproche`) →
  `MATCHED_BANK_TRANSACTION`, `document_id` = transaction, date de la transaction au détail (T3e) ; le lien
  `matched_entry_id` reste (la FK `ON DELETE SET NULL` ne l'a pas effacé).

**AC-5 — Précédence, testée paire par paire** (T4, pour une annulée) : statut → version → paiement détaché → lot
`generated` → rapprochement → exercice clos → exercice postérieur clos → verrou de période → lettrage. Au moins les paires
(version périmée + paiement détaché → version), (paiement détaché + exercice clos → paiement détaché),
(rapprochement + exercice clos → rapprochement), (exercice clos + verrou → exercice clos), (verrou + lettrage → verrou).
Les tests de précédence existants de la validée (`unvalidation_precedence_*`, `supplier_invoices_draft.rs:870, 944`)
restent verts sans modification.

**AC-6 — Annulée alors que payée : pas de brouillon tant que le paiement réel reste sans gel.** Si l'audit
`supplier_invoice.cancelled` de la facture porte `settlementJournalEntryId` et que cette écriture **existe** et n'est
**pas contre-passée**, refus `DetachedPayment` (`SUPPLIER_INVOICE_DETACHED_PAYMENT`), détails : `entryId`, `entryNumber`
et, si l'écriture est rapprochée, `transactionDate`. Contre-passée (depuis sa fiche, ou par l'annulation du rapprochement
qui la classe « écriture propre », `reconciliation_cancel.rs:44-48`) ou absente → le geste passe ; l'écriture de paiement
et sa contre-passation **restent** au grand livre (jamais touchées par le geste). La lecture d'audit est **la même** que
celle de `modification_guard` : une fonction `pub(crate)` unique (nom libre, par exemple
`detached_settlement_of_cancelled_invoice`), appelée par les deux sites (DRY ; `git grep -nF "settlementJournalEntryId"
-- crates/kesh-db/src` ne rend, hors doc-comments et l'écriture de `cancel_in_tx`, que cette fonction). Tests T5 :
(a) payée → annulée → refus, détails sans date ; (b) idem, paiement rapproché → refus avec date ; (c) paiement contre-passé
→ brouillon, paiement et sa contre-passation présents, inchangés ; (d) le chemin (b) → *Annuler le rapprochement* → brouillon.

**AC-7 — Payée dont le règlement est rapproché : le refus dit d'annuler le rapprochement.** Une facture `paid` dont
l'écriture de règlement est `matched_entry_id` d'une transaction → `PaidReconciled`
(`SUPPLIER_INVOICE_PAID_RECONCILED`), `document_id` = transaction, `transactionDate` au détail ; règlement non rapproché →
`Paid` comme aujourd'hui. Le statut parle avant la version pour les deux (test existant `unvalidation_status_refusals`
étendu). T6.

**AC-8 — Les messages disent quoi faire d'abord, et nomment l'objet.** Le corps de la réponse 409
`SupplierInvoiceNotUnvalidatable` garde `code`, `message`, `details.documentId`, `details.documentNumber` et ajoute
`details.transactionDate` (`AAAA-MM-JJ`, ou `null`), `details.entryNumber` (ou `null`), `details.batchId` (ou `null`).
Le serveur construit le message par `t_args` (clés FTL plates à arguments `$date`, `$entry`, `$batch` ; la date au format
suisse `JJ.MM.AAAA` par le formateur existant du serveur) ; le frontend (`supplierInvoiceUnvalidateMessage`) rend **le
même texte** à partir de `code` et `details`, mot pour mot le FTL fr-CH dans ses replis. Clés neuves ou modifiées dans
les **quatre** locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`). Le suffixe « (numéro de la
facture) » des trois motifs propres à la pièce disparaît des messages qui nomment désormais leur objet (lot, transaction,
écriture) — le numéro de la facture est déjà sur la fiche. Tests : `kesh-api` T7 (une réponse par variante, `details`
complets, message fr-CH exact) ; Vitest T8 (un texte par code, avec et sans date, repli inconnu ; test de parité
`SUPPLIER_INVOICE_UNVALIDATE_CODES` mis à jour).

**AC-9 — Trace d'audit du geste.** `supplier_invoice.unvalidated` (même action, C-v0172a-10), `details` :
`supplierInvoice` (instantané d'avant), `previousStatus` (`"open"` ou `"cancelled"`), `purchaseJournalEntryId`, et pour une
annulée `reversalJournalEntryId` ; plus, dans la même transaction, `lettering.removed` pour chaque groupe dissous et
`journal_entry.deleted` pour chacune des deux écritures. T9 asserte les quatre familles et leurs champs ; la dévalidation
d'une validée écrit `previousStatus: "open"` (T9b) et rien d'autre de neuf.

**AC-10 — Le chemin d'une payée par la réconciliation, de bout en bout (point 2 du mandat).** Test d'intégration `kesh-db`
T10 `a_reconciled_paid_invoice_goes_back_to_draft_through_unreconciliation` : facture validée, payée, règlement rapproché
→ repasser en brouillon refusé `PaidReconciled` → `reconciliation_cancel::cancel_in_tx` → facture `open` → repasser en
brouillon → `draft` → `delete_draft` (administrateur) → facture absente, écriture d'achat absente, écriture de règlement
et sa contre-passation présentes (le règlement n'est jamais supprimé). Aucun refus n'est levé par la story : le parcours
n'emprunte que des gestes existants.

**AC-11 — Lot de paiement : l'historique se garde toujours.** Une annulée qui a figuré dans un lot (annulé, ou confirmé
puis règlement annulé) repasse en brouillon, mais le brouillon **ne se supprime pas**
(`SUPPLIER_INVOICE_HAS_PAYMENT_BATCH_HISTORY`, inchangé). T11. Le manuel le dit.

**AC-12 — Le geste se trouve.** Sur la fiche d'une facture fournisseur, pour un rôle Comptable ou Administrateur
(`canManage`), un bouton **« Repasser en brouillon »** (`data-testid="supplier-invoice-unvalidate"`, conservé) s'affiche
pour les statuts `open`, `paid` **et** `cancelled` ; jamais pour un brouillon, jamais pour un rôle sans écriture. Pour
`paid`, le clic aboutit au refus du serveur, qui dit quoi faire (C-v0172a-8). La confirmation dit ce qui va s'écrire :
validée — le texte actuel adapté (« Repasser la facture en brouillon ? … ») ; **annulée** — un texte propre (clé neuve
`supplier-invoices-unvalidate-confirm-body-cancelled`) : « Son écriture d'achat **et** sa contre-passation seront
supprimées du journal, et la numérotation des écritures gardera deux trous ; le journal d'audit garde les deux
écritures. La facture redeviendra un brouillon modifiable — à valider de nouveau, ou à supprimer. » Succès : clé neuve
`supplier-invoices-unvalidated-cancelled` pour l'annulée. Libellés renommés : `supplier-invoices-unvalidate` = « Repasser
en brouillon », `…-confirm-title` = « Repasser la facture en brouillon ? », dans les quatre locales (C-v0172a-9). La fiche
**client** garde « Dévalider ». Les refus `SUPPLIER_INVOICE_PAID_RECONCILED` et `SUPPLIER_INVOICE_DETACHED_PAYMENT`
relisent la facture comme `SUPPLIER_INVOICE_PAID` (`isStaleState`). Vitest T12 (page) : bouton par statut × rôle ; corps
de confirmation par statut ; message de refus affiché avec sa date.

**AC-13 — E2E du parcours.** Playwright, dans `frontend/tests/e2e/supplier-invoice-draft-cycle.spec.ts` (ou une spec
voisine) : facture saisie, validée, annulée → fiche annulée → *Repasser en brouillon* → confirmation (texte « deux trous »)
→ badge *Brouillon*, *Modifier* proposé → (administrateur) *Supprimer le brouillon* → retour à la liste, facture absente ;
et une facture payée depuis la fiche → *Repasser en brouillon* → message « annulez d'abord son règlement » affiché ; et
une facture payée par la réconciliation (parcours existant de la spec, `:400-412`) → *Repasser en brouillon* → message
« … transaction bancaire du JJ.MM.AAAA : annulez d'abord ce rapprochement … ». T13.

**AC-14 — Manuels, CHANGELOG, captures** : § *Manuel*, § *CHANGELOG*, § *Captures* ci-dessous, réalisés ; PDF régénérés
et contrôlés aplatis (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, grep des phrases neuves).

## Tests et mutations

| Test | Où | AC |
|---|---|---|
| T1, T2, T3a-e, T4, T5a-d, T6, T9, T9b, T10, T11 | `crates/kesh-db/tests/supplier_invoices_draft.rs` (helpers `decor`, `validee`, `brouillon`, `en_lot`, `rapprochee`, `fournisseur_rapproche::*` existants) | 1-7, 9-11 |
| T7 | `crates/kesh-api/tests/supplier_invoice_draft_e2e.rs` (route `POST …/unvalidate` : 200 sur une annulée, 409 par variante, `details`, message) | 8 |
| T8 | `frontend/src/lib/features/supplier-invoices/supplier-invoice-unvalidate.test.ts` | 8 |
| T12 | `frontend/src/routes/(app)/supplier-invoices/[id]/supplier-invoice-draft-page.test.ts` | 12 |
| T13 | `frontend/tests/e2e/supplier-invoice-draft-cycle.spec.ts` | 13 |

⚠️ **Test existant à réécrire, non à supprimer** : `unvalidation_status_refusals` (`supplier_invoices_draft.rs:761-865`)
asserte `annulée → IllegalStateTransition` (`:827-834`). Ce cas devient le succès d'AC-1 ; le brouillon y reste le seul
`IllegalStateTransition`. Le rouge de ce test, avant le correctif, est la première preuve demandée (tests rouges d'abord).
Le cas « annulée » de `delete_draft` (`:1165-1172`, `IllegalStateTransition`) reste **inchangé** : une annulée ne se
supprime pas directement. Deux autres assertions existantes changent de sens et se réécrivent :
`supplier-invoice-draft-page.test.ts` (les cas par statut et rôle, `:160-235` — le bouton s'affiche désormais pour `paid`
et `cancelled` aux rôles qui écrivent ; le cas Consultation `:490-501` reste « absent ») et l'E2E
`supplier-invoice-draft-cycle.spec.ts:412` (`supplier-invoice-unvalidate` en `toHaveCount(0)` sur une facture payée par la
réconciliation) — qui devient le second parcours d'AC-13 : bouton visible, clic, message « transaction bancaire du … ».

**Rouges d'abord** : T1, T5a, T6, T7, T8, T12 écrits et constatés rouges avant le code (sortie au Dev Agent Record).

**Mutations** (sous le verrou, restauration `git checkout -- <fichier>` puis `touch`), chacune tuée par le test nommé :

| # | Mutation | Tuée par |
|---|---|---|
| M1 | **garde d'exercice / de période sur la contre-passation** : remplacer `delete_in_tx(contre-passation)` par un `DELETE` SQL brut | T3c |
| M2 | **suppression de la contre-passation** retirée (seul l'achat supprimé) | T1 (le geste échoue en `ENTRY_IS_REVERSED`/FK, ou la contre-passation reste) |
| M3 | **trace d'audit** : retirer l'`insert` `supplier_invoice.unvalidated` sur la branche annulée | T9 |
| M3b | retirer `reversalJournalEntryId` (ou `previousStatus`) des `details` | T9 |
| M4 | dissolution des paires retirée | T1 (`ENTRY_LETTERED`) |
| M5 | dissolution élargie à tout groupe des deux écritures (manuel compris) | T3d |
| M6 | motif `DetachedPayment` retiré | T5a |
| M6b | `DetachedPayment` levé même quand le paiement est contre-passé | T5c |
| M7 | détection du règlement rapproché retirée (`Paid` partout) | T6, T10 |
| M8 | `status IN ('open','cancelled')` ramené à `'open'` dans l'`UPDATE` | T1 |
| M9 | contrôle de rapprochement sur la contre-passation retiré | T3e (variante contre-passation rapprochée) |
| M10 | bouton masqué pour `cancelled` | T12 |
| M11 | `transactionDate` absent du message frontend | T8 |
| M12 | pré-contrôle d'exercice clos (C-v0172a-4) retiré | **équivalente, déclarée** : `delete_in_tx` rend le même code et la transaction est annulée ; le pré-contrôle tient le contrat du mode `System` (exercice tenu ouvert), il n'a pas d'effet observable — la mutation s'exécute et son résultat (tests verts) se déclare |

## Tâches

1. **Rouges** : T1, T5a, T6, T7, T8, T12 écrits, rouges constatés ; `unvalidation_status_refusals` réécrit (rouge).
2. **`errors.rs` (kesh-db)** : variantes `PaidReconciled`, `DetachedPayment`, leurs `code()` ; doc-comment de l'énumération
   (cinq variantes, précédence). `DbError::SupplierInvoiceNotUnvalidatable` porte les détails nécessaires (forme libre :
   par exemple un champ `hint` — `transaction_date`, `entry_number`, `batch_id` — sans casser les appelants existants).
3. **Dépôt** — `unvalidate_in_tx`, ordre **à respecter** (« les verrous d'abord », leçon 25-3-b) :
   1. facture `FOR UPDATE` ; statut : `open` | `cancelled` passent, `paid` → lecture verrouillante (`LOCK IN SHARE MODE`)
      de la transaction rapprochée de l'écriture de règlement → `PaidReconciled` ou `Paid` ; brouillon →
      `IllegalStateTransition` ; version ;
   2. lot `generated` (existant) ;
   3. **branche annulée** : `reversal_id = reversed_by(achat)` (vide → `Invariant`, C-v0172a-16) ; écriture d'achat **et**
      son exercice `FOR UPDATE`, puis la contre-passation **et** son exercice `FOR UPDATE` (requête jointe, texte du
      `SELECT fy.id FROM journal_entries je JOIN fiscal_years fy … FOR UPDATE` de `cancel_in_tx:2074-2084`) ; transaction rapprochée de l'achat **ou** de
      la contre-passation (`LOCK IN SHARE MODE`) ; puis, **après** ces verrous, la lecture d'audit du paiement détaché
      (AC-6 ; l'écriture détachée lue par `reversed_by`, sans verrou : une contre-passation ne s'annule pas, une
      contre-passation concurrente ne fait que rendre le refus conservateur) ; refus dans l'ordre d'AC-5 ;
      pré-contrôle : exercice de l'achat ou de la contre-passation `Closed` → `FiscalYearClosed` ;
   4. instantané (`fetch_lines`, écart déjà assumé) ;
   5. branche annulée : dissoudre, par `letterings::dissolve_group_in_tx(…, Mode::System { held_open_fiscal_year_id:
      <exercice de la contre-passation> }, …)`, **chaque** groupe d'origine `reversal` dont **toutes** les lignes
      appartiennent à l'achat ou à sa contre-passation ; aucun autre groupe ;
   6. `UPDATE … status = 'draft', purchase_journal_entry_id = NULL, version = version + 1 WHERE … AND version = ? AND
      status IN ('open','cancelled')` (garde le statut **lu**) ;
   7. `delete_in_tx(contre-passation)` puis `delete_in_tx(achat)`, `enforce_ownership = false` ;
   8. audit (AC-9) ; relecture.
   Fonction partagée du paiement détaché, appelée aussi par `modification_guard` (AC-6). Doc-comments à jour : module
   (`supplier_invoices.rs:1-17`), `unvalidate_in_tx` (précédence, verrous, branche annulée), `cancel_in_tx` (l'annulée
   n'est plus définitive), `delete_in_tx` (§ `enforce_ownership`, deux suppressions par geste), `modification_guard`,
   `SupplierInvoiceNotUnvalidatable`.
4. **API** : `supplier_unvalidation_blocked_text` (`kesh-api/src/errors.rs:4714-4733`) et le bras
   `SupplierInvoiceNotUnvalidatable` (`:4137-4159`) : messages par `t_args`, `details` d'AC-8 ; doc-comment de la route.
5. **i18n** : clés d'AC-8 et AC-12 dans les quatre locales ; `npm run lint-i18n-ownership`, tests i18n du frontend
   (`i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`, `i18n-libelle-en-dur.test.ts`) verts.
6. **Frontend** : `supplier-invoice-unvalidate.ts` (codes, arguments, replis = FTL fr-CH) ; `+page.svelte` (bouton pour
   `open`/`paid`/`cancelled`, confirmation par statut, succès par statut, `isStaleState`) ; commentaires à jour
   (`ENTRY_IS_REVERSED` reste « inatteignable », pour une autre raison).
7. **Manuels, CHANGELOG, captures** (ci-dessous).
8. **Greps de propagation** (CLAUDE.md, « greper la valeur ») : `IllegalStateTransition` sur l'annulée
   (`git grep -nE "annulée.*IllegalState|cancelled.*IllegalState"`), le libellé « Dévalider » côté fournisseur
   (`git grep -nF "Dévalider" -- frontend/src docs/manual/fr crates/kesh-i18n`, tri client/fournisseur), « ne supprime
   aucune facture annulée », « sa fiche ne le propose pas » ; résultats au Dev Agent Record.
9. **Gates** : gate complet (`scripts/test-fast.sh`, base `kesh_v0172a` remise à neuf), `npm run check`,
   `lint-i18n-ownership`, `test:unit`, `build`, **E2E complet au dernier commit de code** (port 3172, `kesh_e2e_v0172a`,
   dossiers `/tmp/kesh-e2e-v0172a-r<n>/…`), échecs jugés contre `docs/testing.md` § *Les échecs attendus*. Exception
   `kesh-db` : patch de dépôt ⇒ gate complet même en cours de boucle.

## Manuel

**Utilisateur** (`docs/manual/fr/user-manual.tex`, puis `make fr` — PDF commité) :
- § *Dévalider une facture validée* (`:1718-1751`) → **« Repasser une facture en brouillon »** : le geste s'appelle
  *Repasser en brouillon* (bouton de la fiche, comptable ou administrateur) ; il vaut pour une facture **validée** (texte
  actuel) et, nouveau, pour une **annulée** (deux écritures supprimées, deux trous, ce que garde l'audit, la lecture CO
  957a / Olico art. 3 du § *Piste d'audit* résumée en trois phrases, renvoi à la fiduciaire) ; une **payée** : la fiche le
  propose, et le refus dit quoi faire — annuler le règlement, ou, payée par la réconciliation, annuler le rapprochement
  (renvoi `sec:annuler-rapprochement`), puis repasser en brouillon ; une annulée **qui avait été payée** : contre-passer
  d'abord le paiement — seulement s'il n'a pas eu lieu — ; la liste des refus mise à jour (lot nommé, date de la
  transaction). Remplacer « Une facture payée ne se dévalide pas, et sa fiche ne le propose pas » (`:1729`).
- § *Le brouillon* (`:1697-1705`) : un brouillon issu d'une annulée qui a figuré dans un lot ne se supprime pas
  davantage ; la phrase « s'il n'est pas dû, validez-le puis annulez-le » reste juste.
- § *Annuler une facture fournisseur* (`:1881-1892`) : « Annuler ou dévalider ? » devient « Annuler ou repasser en
  brouillon ? » ; « La 0.17.0 ne supprime aucune facture annulée » est remplacé par : annuler laisse l'achat et sa
  contre-passation aux livres ; depuis la 0.17.2, une annulée saisie par erreur **repasse en brouillon** (puis se corrige,
  ou se supprime), sous les gardes dites.
- Mentions croisées : `:683-687`, `:765-767` (numérotation : deux trous pour une annulée), `:1843-1860` (corriger au
  centime : « dévalidez-la » → « repassez-la en brouillon »), `:2133`.

**Administration** (`docs/manual/fr/admin-manual.tex:3057`, § *Conformité OLICo Art. 9*) : la seconde voie (« la
dévalidation d'une facture ») inclut désormais, pour une facture fournisseur **annulée**, la suppression de l'écriture
d'achat **et de sa contre-passation**, journalisée (`supplier_invoice.unvalidated` avec `previousStatus`,
`journal_entry.deleted` ×2), refusée sous les mêmes motifs, plus le paiement détaché non contre-passé ; la liste des
motifs fournisseur mise à jour. PDF régénéré.

Manuels DE/IT/EN : vides (`docs/manual/{de,en,it}/README.md` seulement) — rien à traduire.

## CHANGELOG

Sous `## [0.17.2] — Non publié` (créer l'en-tête s'il n'existe pas ; ne pas dater) :

- **Ajouté** — « **Une facture fournisseur annulée repasse en brouillon** ([#868]) : *Repasser en brouillon*, sur sa
  fiche, retire du journal son écriture d'achat et sa contre-passation — deux trous dans la numérotation, expliqués par
  le journal d'audit —, sous les gardes de la dévalidation (exercice ouvert, période non verrouillée, aucune écriture
  rapprochée ni lettrée ailleurs). Le brouillon se corrige et se revalide, ou se supprime. Une facture annulée alors
  qu'elle était payée ne repasse en brouillon qu'une fois son paiement contre-passé. *Refs #868.* »
- **Modifié** — « Le bouton *Dévalider* d'une facture fournisseur s'appelle *Repasser en brouillon* et s'affiche aussi
  sur une facture payée ou annulée ; quand le geste est refusé, le message dit quoi faire d'abord — annuler le règlement,
  le rapprochement de la transaction du …, ou le lot de paiement n° … *Refs #868.* »

## Captures

Thème clair, base de démonstration jetable (`kesh_v0172a`), données synthétiques, dans
`/home/gcorbaz/devel/kesh-gate-logs/v0172a/captures/` : fiche validée (bouton *Repasser en brouillon*) ; fiche annulée
(bouton) ; confirmation sur l'annulée (texte « deux trous ») ; brouillon obtenu ; refus sur une payée ; refus sur une
payée rapprochée (date) ; refus sur une annulée qui avait été payée. Jointes au Dev Agent Record et au prompt de la revue.

## Dev Notes

- **Aucune migration** (C-v0172a-1) : P1–P8 sans objet ; `migrations-idempotence-audit.md`, `test-schema/`,
  `post_restore.rs` intouchés. Contrôle : `git diff --stat d7ec2ea2 -- crates/kesh-db/migrations crates/kesh-db/test-schema`
  vide.
- **Verrous** : facture → (achat, exercice) → (contre-passation, exercice) → transactions bancaires (S) → lignes (par la
  dissolution) → `UPDATE` facture → `delete_in_tx` ×2 (qui reprennent des verrous déjà tenus, puis les exercices
  postérieurs et les lignes). Cycle connu lignes ↔ écriture avec un lettrage concurrent : couvert par le rejeu de la route.
  Aucune lecture ordinaire avant le dernier verrou d'écriture/exercice, hors `fetch_lines` (écart déjà assumé, même
  justification : `reversed_by` de l'achat ne change que par ce geste, sous le verrou de la facture).
- **Ne pas** toucher `delete_in_tx` (hors doc), `cancel_in_tx`, `delete_draft_in_tx`, `reconciliation_cancel` : le
  périmètre est le dépôt de la dévalidation, la lecture partagée du paiement détaché, l'API, l'écran, les textes.
- Le seed (`kesh-seed`) n'est pas modifié.
- **Rôles** : inchangés (Comptable+ pour repasser en brouillon, Administrateur pour supprimer un brouillon), clés d'API
  `read-write` comme aujourd'hui ; le serveur MCP n'écrit pas — rien à faire.

## Pour l'orchestrateur

- **Issue à ouvrir (on trace toujours)** — *Factures clientes annulées : aucun retour au brouillon* (enhancement, P4,
  `triage`) : « `invoices::unvalidate` refuse une facture cliente `cancelled` (`invoices.rs:1520-1526`). Une facture
  cliente n'est annulée que par un avoir — document numéroté, avec son écriture, souvent envoyé —, que la dévalidation
  refuse déjà (`Credited`). L'écart avec les factures fournisseur (0.17.2, #868) est voulu par construction ; l'issue le
  trace pour que le Project Lead décide s'il faut un geste (supprimer un avoir jamais envoyé ?) ou rien. »
- **Commentaire à porter sur #868** : la reformulation (on ne supprime qu'un brouillon ; l'annulée y arrive par
  *Repasser en brouillon*) et le sort des quatre points de conception (§ *Origine*).
- Prémisse du mandat corrigée : « Sort du document rattaché : le même que pour la dévalidation d'une `open` » — vérifié :
  la dévalidation ne touche pas au document ; il reste joint au brouillon et ne revient « à traiter » qu'à la suppression.

## Change Log

| Date | Version | Description | Auteur |
|---|---|---|---|
| 2026-11-04 | 0.1 | Fiche créée en autonomie (C-v0172a-1 à 18) ; validation P1 à lancer (`v0172-annulees-validate-prompt-p1.md`, Sonnet). | Opus 5.5 |

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
