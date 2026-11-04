# Story v0172-annulees : la 0.17.2 — repasser en brouillon une facture fournisseur annulée (#868)

Status: ready-for-dev

*Fiche rédigée le 2026-11-04, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0172a`, branche
`story/v0172-annulees` partie de `d7ec2ea2` (le merge de la v0.17.1 sur `main`). Release **0.17.2**, **sans migration**
(C-v0172a-1). Registre : `v0172-choix-autonomes.md`, section `v0172-annulees` (C-v0172a-1 à 25). Clé courte : `v0172a`.
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

### 1. La dévalidation d'aujourd'hui (`supplier_invoices::unvalidate_in_tx`, `supplier_invoices.rs:909-1016`)

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
  (`:758-842`, `detach_in_tx` `:791`). C'est la règle reprise pour l'annulée (C-v0172a-12).
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
contre-passée → `EntryIsReversed`** (`:1880-1882`, « art. 958f CO ») ; 3-ter (seulement `enforce_ownership`) garde de la
modification ; 3-quater verrou de période → `PeriodLocked` (`:1904-1918`) ; 3-quinquies **ligne lettrée** →
`EntryNotModifiable` (`ENTRY_LETTERED`, `:1925-1927`) ; instantané, audit `journal_entry.deleted`, `DELETE` (lignes en
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
`cancelled` → `IllegalStateTransition`, **par le statut**, avant tout motif. Une facture cliente n'est **annulée que
par un avoir** (`credit_notes.rs:772`, seul `UPDATE … status = 'cancelled'` de production) : document numéroté, avec sa
propre écriture, souvent **envoyé au client** — l'avoir **est** la contre-passation. Le motif `Credited` (`:1556-1569`)
ne concerne qu'une facture encore `validated` qui porte un avoir ; il n'est pas ce qui refuse une annulée (V1-15).
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
| 4 | `supplier_invoices.rs:771-778` `delete_draft_in_tx` | refuse `cancelled` (`IllegalStateTransition`) | inchangé : l'annulée ne se supprime pas directement — d'abord brouillon (décision du Project Lead) ; le refus reste le générique `ILLEGAL_STATE_TRANSITION` (« Transition d'état interdite », `kesh-api/src/errors.rs:4260-4266`), sans motif neuf : la route `DELETE` est réservée à l'Administrateur et **fermée aux clés d'API** (`docs/api-external.md:563`), et l'écran ne propose *Supprimer le brouillon* que sur un brouillon — le refus n'est atteignable que par un appel direct de session (V3-7, C-v0172a-25) |
| 5 | `journal_entries.rs:1086-1092` `modification_guard` (paiement détaché) | gèle le règlement détaché d'une annulée | **protégé par refus** : une annulée dont le paiement détaché existe non contre-passé ne repasse pas en brouillon (AC-6) ; contre-passé, il est gardé par l'étape 3-bis |
| 6 | `letterings.rs:1847-1900` `discover_supplier_invoice_document` | `cancelled` → aucune pièce | non atteint : le brouillon n'est pas soumis au lettrage de pièce (les trois appelants refusent un brouillon avant) |
| 7 | `contacts.rs:401` `find_suppliers_by_invoice_iban` | exclut les annulées des fournisseurs suggérés | le brouillon y revient — voulu : ce n'est plus une facture écartée |
| 8 | `reconciliation.rs:825, 885, 935` propositions | ne lisent que `open` | inchangé (ni annulée ni brouillon proposées) |
| 9 | `closing_checks.rs:504` contrôle K8 | brouillons datés dans l'exercice | le brouillon y apparaît — voulu (il n'est plus aux livres) |
| 10 | `supplier_invoices.rs:212, 218-279` `list` + filtre de la 0.17.0 (`routes/supplier_invoices.rs:644`, case « Afficher les factures annulées », `frontend/src/routes/(app)/supplier-invoices/+page.svelte:113`) | masque les annulées par défaut | le brouillon apparaît dans la liste par défaut |
| 11 | `imported_supplier_invoices.rs:379-400` `find_source_document_of_entry` | le document de l'achat d'une annulée | l'achat supprimé, plus rien à montrer ; le document reste joint au brouillon (C-v0172a-12) |
| 12 | `crates/kesh-api/src/exports/csv_tables.rs:1233` | exporte le statut | exporte `draft` ; aucune règle |
| 13 | `audit_labels.rs:194, 199` | libellés `supplier_invoice.cancelled` / `.unvalidated` | `.unvalidated` réemployé (C-v0172a-10) ; aucune action neuve ; son **libellé affiché** (`audit-log-action-supplier-invoice-unvalidated`, fr-CH `:3053` « Facture fournisseur dévalidée ») devient « Facture fournisseur repassée en brouillon » — 18ᵉ clé de la liste `K` (V3-1, C-v0172a-24) |
| 14 | `frontend/…/supplier-invoices/[id]/+page.svelte:922-940` | bloc « Facture annulée. », bloc d'annulation masqué | **modifié** : bouton « Repasser en brouillon » (AC-12) |
| 15 | `frontend/…/SupplierInvoiceTotal.svelte:30`, `supplier-invoice-helpers.ts:46`, `invoice-cancel.ts:50` | affichage | inchangés |
| 16 | Sauvegarde / `post_restore.rs` | — | aucun : pas de migration, pas de backfill, inventaire des tables identique ; un `.keshbackup` d'avant le geste restaure l'annulée et ses deux écritures |
| 17 | Rapports (`kesh-report`), grand livre, TVA | lisent les écritures | la paire achat + contre-passation est nette à zéro **au cumul** seulement : l'achat est daté de la facture, la contre-passation du jour de l'annulation ; la retirer change, en sens inverse, les totaux de la période de l'achat et de celle de l'annulation (charge, impôt préalable, TVA du trimestre, tout solde arrêté entre les deux dates) ; le verrou de période protège une période déclarée, sur l'une **et** l'autre écriture (AC-4 c, c' ; C-v0172a-23) |
| 18 | `crates/kesh-api/src/mcp/outils/commun.rs:153` | filtre `status` de l'outil MCP (`cancelled` → `StatutFactureFournisseur::Cancelled`) | inchangé : lecture ; le brouillon y apparaît sous `draft` |
| 19 | `crates/kesh-api/src/services/factures_fournisseur.rs:77` | validation du filtre `status` | inchangé |
| 20 | `crates/kesh-api/src/errors.rs:4673` | texte `supplier-invoices-cancel-blocked-cancelled` (annuler une annulée) | inchangé : le refus d'annuler reste ; le brouillon répond `SupplierInvoiceDraft` |

Critère de tri : un site figure s'il **écrit** le statut, s'il le **lit pour décider** d'un refus ou d'un affichage, ou
s'il le **filtre** ; les sites des lots de paiement, des factures clientes, des règlements et de `DocumentPreview` sont
écartés (mot `cancelled` sans rapport). `git grep -nE "si\.status|status\s*=\s*'cancelled'" -- 'crates/*/src/*'` ne rend aucun autre site fournisseur (relevé :
`contacts.rs:401`, `closing_checks.rs:504`, `journal_entries.rs:1090`, `reconciliation.rs:825,885,935`, `supplier_invoices.rs:2149`).

## Inventaire clos — `SupplierUnvalidationBlocker` et les refus de « Repasser en brouillon »

Après la story, l'énumération (`crates/kesh-db/src/errors.rs:329-352`) compte **cinq** variantes ; les refus qui ne sont
pas des variantes sont listés à la suite. **Tout refus dit quoi faire d'abord, avec le nom du geste tel que l'écran le
montre** — « repasser en brouillon », jamais « dévalider » côté fournisseur (C-v0172a-20). Les textes fr-CH ci-dessous
sont les **replis**, mot pour mot le FTL fr-CH ; les quatre locales reçoivent leur traduction. Les chemins de menu sont
ceux de l'écran : le lot se trouve dans *Paiements fournisseurs* (`nav-payment-batches`, `messages.ftl:118`) ; un
rapprochement s'annule depuis le **détail de l'import bancaire** (`user-manual.tex:2828-2830`). Un numéro d'écriture
**repart à 1 à chaque exercice** (`journal_entries.rs:2908-2915`) : il est toujours suivi de la date de l'écriture.

| Ordre | Refus | Code | Quand | Clé — message fr-CH (ce qu'il faut faire d'abord) |
|---|---|---|---|---|
| 0 | `NotFound` | `NOT_FOUND` | — | inchangé |
| 1 | `IllegalStateTransition` | `ILLEGAL_STATE_TRANSITION` | brouillon | `supplier-invoices-unvalidate-blocked-not-validated` — « Cette facture est déjà un brouillon : rechargez la page pour voir son état. » |
| 1 | `Paid` (existant) | `SUPPLIER_INVOICE_PAID` | `paid`, règlement **non** rapproché | `error-supplier-invoice-unvalidate-blocked-paid` — « Cette facture fournisseur est payée : annulez d'abord son règlement (bouton « Annuler le règlement » de cette fiche), puis repassez-la en brouillon. » |
| 1 | **`PaidReconciled`** (neuf) | `SUPPLIER_INVOICE_PAID_RECONCILED` | `paid`, écriture de règlement rapprochée | `error-supplier-invoice-unvalidate-blocked-paid-reconciled` — « Cette facture fournisseur est payée par la transaction bancaire du { $date } : annulez d'abord ce rapprochement, depuis le détail de l'import bancaire — elle redeviendra validée —, puis repassez-la en brouillon. » |
| 2 | `OptimisticLockConflict` | `OPTIMISTIC_LOCK_CONFLICT` | version périmée | inchangé (`supplier-invoices-unvalidate-blocked-stale`, sans le verbe) |
| 3 | `InPaymentBatch` (existant) | `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` | lot `generated` | `error-supplier-invoice-unvalidate-blocked-in-payment-batch` — « Cette facture figure dans le lot de paiement n° { $batch }, en cours : annulez d'abord ce lot, dans Paiements fournisseurs, puis repassez-la en brouillon. » |
| 4 | **`DetachedPayment`** (neuf) | `SUPPLIER_INVOICE_DETACHED_PAYMENT` | `cancelled`, l'audit de l'annulation nomme un règlement qui existe et n'est pas contre-passé, non rapproché | `error-supplier-invoice-unvalidate-blocked-detached-payment` — « Cette facture a été annulée alors qu'elle était payée : son paiement (écriture n° { $entry } du { $entryDate }) reste au grand livre. Si ce paiement n'a pas eu lieu, contre-passez-le d'abord depuis sa fiche, puis repassez la facture en brouillon. » |
| 4 | `DetachedPayment` (même variante) | `SUPPLIER_INVOICE_DETACHED_PAYMENT` | idem, paiement rapproché | `error-supplier-invoice-unvalidate-blocked-detached-payment-reconciled` — « Cette facture a été annulée alors qu'elle était payée : son paiement (écriture n° { $entry } du { $entryDate }) est rapproché de la transaction bancaire du { $date }. Si ce paiement n'a pas eu lieu, annulez d'abord ce rapprochement, depuis le détail de l'import bancaire — il contre-passera le paiement —, puis repassez la facture en brouillon. » |
| 5 | `MatchedBankTransaction` (existant) | `MATCHED_BANK_TRANSACTION` | écriture d'achat, puis contre-passation, rapprochée | `error-supplier-invoice-unvalidate-blocked-matched` — « L'écriture d'achat de cette facture est rapprochée de la transaction bancaire du { $date } : annulez d'abord ce rapprochement, depuis le détail de l'import bancaire, puis repassez la facture en brouillon. » |
| 6 | `FiscalYearClosed` | `FISCAL_YEAR_CLOSED` | **pré-contrôle** : exercice de l'achat ou de la contre-passation clos | `supplier-invoices-unvalidate-blocked-fiscal-year-closed` — « L'écriture d'achat de cette facture appartient à un exercice clôturé : pour pouvoir la repasser en brouillon, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent. » |
| 7a | refus de `delete_in_tx` **pour la contre-passation** (annulée seulement), dans leur ordre : `LaterFiscalYearClosed`, `PeriodLocked`, `ENTRY_LETTERED` | voir 7b | — | mêmes clés que 7b |
| 7b | refus de `delete_in_tx` **pour l'achat**, dans leur ordre : `LaterFiscalYearClosed` | `LATER_FISCAL_YEAR_CLOSED` | exercice postérieur clos | `supplier-invoices-unvalidate-blocked-later-fiscal-year-closed[-named]` — « …pour pouvoir la repasser en brouillon, un administrateur doit rouvrir les exercices postérieurs clôturés, en commençant par le plus récent. » (nomme l'exercice) |
| 7b | `EntryIsReversed` | `ENTRY_IS_REVERSED` | **inatteignable** (la contre-passation part d'abord ; une contre-passation ne se contre-passe pas, `IsAReversal`) | `supplier-invoices-unvalidate-blocked-reversed` — « L'écriture d'achat de cette facture a été contre-passée : elle ne peut plus être supprimée, et la facture ne repasse pas en brouillon. » |
| 7b | `PeriodLocked` | `PERIOD_LOCKED` | date ≤ verrou | `supplier-invoices-unvalidate-blocked-period-locked` — « L'écriture d'achat de cette facture est datée sous le verrou de période : pour pouvoir la repasser en brouillon, un administrateur doit faire reculer le verrou avant sa date. » |
| 7b | `EntryNotModifiable` (`ENTRY_LETTERED`) | `ENTRY_LETTERED` | ligne lettrée **hors** des paires `reversal` de l'annulation | `supplier-invoices-unvalidate-blocked-lettered` — « L'écriture d'achat de cette facture est lettrée : délettrez-la d'abord, puis repassez la facture en brouillon. » |
| — | (repli) | autre | — | `supplier-invoices-unvalidate-error` — « La facture n'a pas pu repasser en brouillon. » |
| — | `Invariant` | 500 | annulée sans contre-passation liée, facture validée sans achat | C-v0172a-16 (T14) |

⚠️ **La précédence est composée par écriture, non en un ordre total** (C-v0172a-19) : rangs 0 à 6 évalués par le dépôt ;
puis `delete_in_tx` sur la **contre-passation** (7a), puis sur l'**achat** (7b), chacune dans l'ordre de `delete_in_tx`
(`journal_entries.rs:1857-1927`). Conséquence assumée, testée (T4, paire *P5*) : achat sous un exercice postérieur clos et
contre-passation sous le verrou de période → `PERIOD_LOCKED` (la contre-passation parle d'abord), non
`LATER_FISCAL_YEAR_CLOSED`. Les deux obstacles doivent être levés de toute façon ; le second refus se lit au clic suivant.

**Limite L1** (C-v0172a-15) : les messages 5, 6 et 7 nomment « l'écriture d'achat » ; pour une annulée, le refus 7a — et
le rang 5 quand c'est la contre-passation qui est rapprochée (cas posé en SQL seulement, T3e, aucune route ne rapproche
une écriture sans ligne bancaire) — vient de la contre-passation. Elle est datée du jour de l'annulation, donc presque toujours **après** l'achat : si elle est sous le
verrou, l'achat l'est aussi, et le geste à faire (faire reculer le verrou) est le bon ; le message n'est inexact sur
l'écriture nommée que pour un achat daté **après** son annulation (facture saisie avec une date future). Écrit, non comblé.

**Limite L2** (V1-8, C-v0172a-5) : la lecture d'audit du paiement détaché hérite de la réserve de `modification_guard`
(`journal_entries.rs:1049-1054`) : après l'import d'une sauvegarde d'une **autre** installation, une trace d'annulation
étrangère peut se joindre par `entity_id` à une facture annulée restaurée. L'effet n'est plus seulement un gel de
modification : c'est aussi un refus `DetachedPayment` dont le message désigne une écriture sans rapport. Dite au manuel
d'administration, à côté de la réserve existante (`admin-manual.tex:3073`).

## Piste d'audit — pourquoi supprimer une écriture contre-passée reste acceptable ici

*Lecture de Kesh, non avis juridique ; le manuel la présente comme telle et renvoie à la fiduciaire.*

1. **Ce que le droit demande.** CO 957a al. 2 ch. 5 : une comptabilité **vérifiable** ; CO 958f et OLICo art. 9 al. 1
   let. b : conservation et intégrité des supports modifiables — les deux références que le dépôt appuie déjà
   (`admin-manual.tex:3046`, `:3049-3057`). *L'art. 3 de l'Olico (livres non modifiables sans que cela puisse être
   constaté) est cité ici de mémoire, non vérifié dans le dépôt : il n'entre pas au manuel tant que le Project Lead ou sa
   fiduciaire ne l'a pas confirmé* (V1-13, C-v0172a-18). Le principe n'est pas « rien ne disparaît », c'est « rien ne disparaît sans
   trace » — Kesh l'applique déjà à la suppression d'une écriture (v0.13.0) et à la dévalidation (v0.12.1, fournisseur
   v0.17.0), décrites au manuel d'administration (§ *Conformité OLICo Art. 9*, `admin-manual.tex:3057`).
2. **Pourquoi l'étape 3-bis de `delete_in_tx` (« une écriture contre-passée ne se supprime pas, art. 958f ») ne
   s'applique pas.** Elle protège la **correction** : supprimer l'origine en gardant l'inverse, ou l'inverse en gardant
   l'origine, laisserait aux livres une erreur sans sa correction ou une correction sans objet. Ici les **deux**
   disparaissent ensemble, dans une transaction : le grand livre redevient ce qu'il était avant la saisie fautive. ⚠️ La
   paire n'est nette à zéro qu'**au cumul** : l'achat et la contre-passation tombent en général dans deux périodes
   différentes (date de la facture, jour de l'annulation), dont les totaux changent en sens inverse — TVA du trimestre
   comprise. C'est pourquoi le verrou de période s'applique à l'une **et** à l'autre (C-v0172a-23). `delete_in_tx` n'est pas modifiée ; c'est l'ordre du geste
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
   (`user-manual.tex:1892`), « Annulez une facture qui n'était pas due : l'achat et sa contre-passation restent aux
   livres » (`:1881-1882`) et « Une facture annulée ne se désannule pas, ni ne se dévalide » (`:1917`, qui devient
   faux). Les deux restent vrais **par défaut** — annuler laisse la trace ; repasser en brouillon est
   un second geste, délibéré, réservé à ce qui n'aurait jamais dû être saisi. Le texte est récrit en ce sens (§ *Manuel*).

## Acceptance Criteria

**AC-1 — Une annulée repasse en brouillon.** `supplier_invoices::unvalidate_in_tx` accepte `status = 'cancelled'` (en plus
de `open`). Effet, dans **une** transaction : la facture devient `draft`, `purchase_journal_entry_id = NULL`,
`version + 1` ; l'écriture d'achat **et** sa contre-passation (l'écriture dont `reverses_entry_id` désigne l'achat) sont supprimées, lignes comprises ;
les groupes de lettrage `reversal` qui les unissaient sont dissous ; les lignes de la facture, son numéro, son
fournisseur, ses dates, son projet sont inchangés. Test `kesh-db` `cancelled_invoice_returns_to_draft` (T1) : facture
validée à deux lignes (dont une avec TVA), annulée, puis repassée en brouillon ; asserte statut, colonnes, absence des
deux écritures **et de leurs lignes**, aucune trace `lettering.removed` manquante (une par paire dissoute) et aucun groupe
`reversal` résiduel sur une autre ligne (les lignes qui portaient la marque sont supprimées par CASCADE : on ne les relit
pas) ; soldes des comptes 2000, impôt préalable et charge : **cumulés** (toutes dates) identiques juste avant et juste
après le geste — la paire était nette à zéro au cumul — ; **par période** : le solde arrêté à la veille de l'annulation
baisse du montant de l'achat sur la charge et l'impôt préalable (V2-5 : la suppression change les totaux des deux
périodes, en sens inverse).

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
- (b) exercice postérieur clos → `LATER_FISCAL_YEAR_CLOSED` (T3b, nomme l'exercice) — état **posé en SQL** (exercice
  postérieur clos inséré après l'annulation, hérité d'avant la v0.13.0), comme le fait le test existant
  `unvalidation_precedence_later_year_then_period_lock` (`supplier_invoices_draft.rs:951-958`) — mais avec l'exercice clos
  **entre** l'achat et la contre-passation (§ *Topologie des exercices*, V3-3) : sous la clôture ordonnée
  (`admin-manual.tex:1479`), une annulée dont les deux exercices sont ouverts n'a pas d'exercice postérieur clos ;
- (c) **contre-passation** sous le verrou de période, achat hors verrou (achat daté après son annulation) →
  `PERIOD_LOCKED` (T3c) ; (c') achat sous le verrou → `PERIOD_LOCKED` (T3c') ;
- (d) une ligne de l'achat ou de la contre-passation lettrée **hors** d'une paire `reversal` de l'annulation (état posé
  en SQL dans le test, la route ne le produisant pas — dit au commentaire du test) → `ENTRY_LETTERED` (T3d) ;
- (e) achat ou contre-passation rapprochés d'une transaction (lien posé par `fournisseur_rapproche::debit_rapproche`) →
  `MATCHED_BANK_TRANSACTION`, `document_id` = transaction, date de la transaction au détail (T3e) ; le lien
  `matched_entry_id` reste (la FK `ON DELETE SET NULL` ne l'a pas effacé).

**AC-5 — Précédence, composée par écriture, testée paire par paire** (T4, pour une annulée ; C-v0172a-19). L'ordre est
celui du tableau des refus : statut → version → lot `generated` → paiement détaché → rapprochement (achat, puis
contre-passation) → pré-contrôle d'exercice clos (achat ou contre-passation) → refus de `delete_in_tx` pour la
contre-passation → refus de `delete_in_tx` pour l'achat. Paires exigées, chacune montée pour que les **deux** refus
jouent :
- *P1* version périmée + paiement détaché → `OPTIMISTIC_LOCK_CONFLICT` ;
- *P2* paiement détaché + exercice de l'achat clos → `SUPPLIER_INVOICE_DETACHED_PAYMENT` ;
- *P3* rapprochement de l'achat + exercice de l'achat clos → `MATCHED_BANK_TRANSACTION` ;
- *P4* exercice de l'achat clos + **contre-passation sous le verrou de période** (verrou posé après la date de
  l'annulation) → `FISCAL_YEAR_CLOSED` — c'est la paire qui tue M12 ;
- *P5* exercice postérieur à celui de l'achat clos, **situé entre l'achat et la contre-passation** (état posé en SQL, hérité
  d'avant la v0.13.0) + contre-passation sous le
  verrou → `PERIOD_LOCKED` (la contre-passation parle d'abord ; conséquence assumée, écrite) ;
- *P6* verrou de période sur l'achat + ligne de l'achat lettrée hors paire → `PERIOD_LOCKED`.
La paire « lot + paiement détaché » n'est pas exigée : une annulée n'entre jamais dans un lot `generated`
(`supplier_invoice_cancel_blocker:2013` refuse l'annulation d'une facture en lot `generated`, et `create_batch` n'accepte
que `open`) ; le rang du lot reste avant, pour la validée. Les tests de précédence existants de la validée
(`unvalidation_precedence_*`, `supplier_invoices_draft.rs:870, 944`) restent verts — leurs messages attendus, s'ils en
comparent, suivent AC-8.

**Topologie des exercices des montages** (V3-3). Le décor (`crates/kesh-db/src/test_fixtures.rs:71, 118`) n'a **qu'un**
exercice, 2020–2030, ouvert ; la contre-passation est datée **du jour** (horloge réelle). Les tests à deux exercices le
**scindent en SQL** avant de valider la facture, relativement à l'année du jour `Y` : `A` = [`Y-2`-01-01, `Y-2`-12-31],
`B` = [`Y-1`-01-01, `Y-1`-12-31], `C` = [`Y`-01-01, 2030-12-31] (l'exercice du décor, raccourci). Achat daté de `A`
(par exemple `Y-2`-06-15), annulation le jour même (contre-passation dans `C`), puis états posés en SQL :
- **T3a** : `A` `Closed`, `B` ouvert → `FISCAL_YEAR_CLOSED` (pré-contrôle) ;
- **T3b** : `B` `Closed` — **strictement entre** l'achat et la contre-passation —, `A` et `C` ouverts, aucun verrou → la
  contre-passation passe, l'achat rend `LATER_FISCAL_YEAR_CLOSED` (nomme `B`) ; ⚠️ le montage du test existant
  (`Exercice 2031` clos, **postérieur aux deux** écritures, `supplier_invoices_draft.rs:951-958`) ne convient **pas** : la
  contre-passation y rendrait `LATER_FISCAL_YEAR_CLOSED` la première, et pour sa propre raison ;
- **T3c** : achat daté **après** l'annulation (dans `C`, par exemple `Y+1`-03-01 si ≤ 2030, sinon une date future du même
  exercice), verrou au jour (`books_locked_through` = aujourd'hui) → la contre-passation rend `PERIOD_LOCKED` ;
  **T3c'** : achat dans `A`, verrou au `Y-2`-12-31 → l'achat rend `PERIOD_LOCKED` ;
- **P4** : `A` `Closed` + verrou au jour → `FISCAL_YEAR_CLOSED` (sans le pré-contrôle : `PERIOD_LOCKED` de la
  contre-passation, d'où M12) ;
- **P5** : `B` `Closed` (entre les deux) + verrou au jour → la contre-passation rend `PERIOD_LOCKED` avant que l'achat
  ne dise `LATER_FISCAL_YEAR_CLOSED` ;
- **P6** : verrou au `Y-2`-12-31 (achat seul) + une ligne de l'achat lettrée hors paire (SQL) → `PERIOD_LOCKED` de
  l'achat (3-quater avant 3-quinquies), la contre-passation ayant passé.
Les tests à un seul exercice (T1, T2, T5, T6, T9, T10, T11, T14) gardent le décor tel quel.

**AC-6 — Annulée alors que payée : pas de brouillon tant que le paiement réel reste sans gel.** Si l'audit
`supplier_invoice.cancelled` de la facture porte `settlementJournalEntryId` et que cette écriture **existe** et n'est
**pas contre-passée**, refus `DetachedPayment` (`SUPPLIER_INVOICE_DETACHED_PAYMENT`), détails : `entryId`,
`entryNumber`, `entryDate` et, si l'écriture est rapprochée, `transactionDate` (sinon `null`). Contre-passée (depuis sa
fiche, ou par l'annulation du rapprochement qui la classe « écriture propre », `reconciliation_cancel.rs:44-48`) ou
absente → le geste passe ; l'écriture de paiement et sa contre-passation **restent** au grand livre (jamais touchées par
le geste). La lecture d'audit **partage son prédicat** avec celle de `modification_guard` (DRY : un fragment SQL unique,
décrit ci-dessous ; aucune copie de la jointure ni du chemin JSON). Contrôle (V2-4) : la clé `settlementJournalEntryId` s'**écrit** aussi
dans quatre autres traces que la story ne touche pas (`supplier_invoices.rs:87`, `:1747`, `:2461`,
`invoice_settlements_write.rs:1015`). Le contrôle porte sur la **lecture** : `git grep -nF "'\$.settlementJournalEntryId'" -- crates/kesh-db/src`
rend **une seule ligne**, dans le fragment partagé ci-dessous ; sonde positive à `d7ec2ea2` : une ligne,
`journal_entries.rs:1091`, dans `modification_guard`. **Ce qui est partagé** (V2-12) : les deux sites ne cherchent pas
dans le même sens — `modification_guard` part de l'**écriture** pour trouver la facture annulée qui la nomme
(`ORDER BY si.id LIMIT 1`), la dévalidation part de la **facture** pour énumérer les écritures que nomment ses traces.
Le partage est un **fragment SQL** (constante) : la jointure `supplier_invoices si JOIN audit_log al ON
al.entity_type = 'supplier_invoice' AND al.entity_id = si.id AND al.action = 'supplier_invoice.cancelled' AND
al.company_id = ?`, le filtre `si.company_id = ?` et l'extraction `JSON_VALUE(al.details_json,
'$.settlementJournalEntryId')` ; deux requêtes l'emploient, chacune avec son `WHERE` propre (l'écriture liée en chaîne
d'un côté, `si.id = ?` de l'autre). Le filtre `si.status = 'cancelled'` reste à `modification_guard` (la dévalidation
lit la facture qu'elle tient déjà sous verrou). Plusieurs annulations successives de la même facture (annulée, repassée, revalidée, repayée, annulée) : la
la requête de la dévalidation examine **chaque** trace `supplier_invoice.cancelled` de la facture, et le refus nomme la première écriture
non contre-passée (ordre des traces). Tests T5 : (a) payée → annulée → refus, `transactionDate` nul ; (b) idem, paiement
rapproché → refus avec date ; (c) paiement contre-passé → brouillon, paiement et sa contre-passation présents,
inchangés ; (d) le chemin (b) → *Annuler le rapprochement* → brouillon ; (e) deux cycles d'annulation, le premier
paiement contre-passé, le second non → refus nommant le second.

**AC-7 — Payée dont le règlement est rapproché : le refus dit d'annuler le rapprochement.** Une facture `paid` dont
l'écriture de règlement est `matched_entry_id` d'une transaction → `PaidReconciled`
(`SUPPLIER_INVOICE_PAID_RECONCILED`), `document_id` = transaction, `transactionDate` au détail ; règlement non rapproché →
`Paid` comme aujourd'hui. Le statut parle avant la version pour les deux (test existant `unvalidation_status_refusals`
étendu). T6.

**AC-8 — Les messages disent quoi faire d'abord, et nomment l'objet.** Le corps de la réponse 409
`SupplierInvoiceNotUnvalidatable` garde `code`, `message`, `details.documentId`, `details.documentNumber` et ajoute
**cinq** champs, toujours présents (`null` quand ils ne s'appliquent pas) : `details.batchId`, `details.transactionDate`
(`AAAA-MM-JJ`), `details.entryId`, `details.entryNumber`, `details.entryDate` (`AAAA-MM-JJ`). Le message serveur suit le
patron de `settlement_counterparty_message` (`kesh-api/src/errors.rs:4741-`) : ⚠️ **`t_args` rend le repli tel quel,
sans substituer ses arguments** quand aucun catalogue n'est chargé (`errors.rs:52-60`, et la note `:4739-4740`) — le repli
est donc **construit par `format!`, arguments déjà en place**, puis passé à `t_args` avec les mêmes `FluentArgs`
(`$batch`, `$date`, `$entry`, `$entryDate`) ; les dates passent par `kesh_i18n::formatting::format_date`
(`formatting.rs:38`, `JJ.MM.AAAA`). Clés FTL **plates** (aucun sélecteur). Le frontend (`supplierInvoiceUnvalidateMessage`)
rend **le même texte** à partir de `code` et `details`, ses replis mot pour mot le FTL fr-CH ; un `details` sans date
(paiement détaché non rapproché) choisit la clé sans date. Le suffixe « (numéro de la facture) » disparaît des trois
motifs propres à la pièce : ils nomment désormais leur objet (lot, transaction, écriture), et le numéro de la facture est
déjà sur la fiche. Clés : liste close au § *Clés i18n* (AC-15). Tests : `kesh-api` T7 (une réponse par variante,
`details` complets, message fr-CH exact **arguments substitués**, sous le catalogue de l'harnais) ; test unitaire
`kesh-api` T7b du repli **sans catalogue** (la fonction de message appelée hors état i18n chargé rend le texte `format!`,
sans `{ $…}` littéral — `assert!(!m.contains("{ $"))`) ; Vitest T8 (un texte par code, avec et sans date, repli inconnu ;
test de parité `SUPPLIER_INVOICE_UNVALIDATE_CODES` mis à jour).

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
pour les statuts `open`, `paid` **et** `cancelled` ; jamais pour un brouillon, jamais pour un rôle sans écriture.
Séquences (C-v0172a-21) :
- **validée** : clic → boîte de confirmation (`supplier-invoice-unvalidate-dialog`), titre « Repasser la facture en
  brouillon ? », corps `supplier-invoices-unvalidate-confirm-body` (texte actuel, inchangé) → bouton de la boîte
  `supplier-invoice-unvalidate-confirm` → appel ;
- **annulée** : idem, corps **propre** `supplier-invoices-unvalidate-confirm-body-cancelled` : « Son écriture d'achat et
  sa contre-passation seront supprimées du journal, et la numérotation des écritures gardera deux trous ; le journal
  d'audit garde les deux écritures. Les totaux de la période de l'achat et de celle de l'annulation changeront, TVA
  comprise : si l'une a déjà été déclarée, faites d'abord verrouiller la période. La facture redeviendra un brouillon
  modifiable — à valider de nouveau, ou à supprimer. » (V2-5) ;
- **payée** : clic → **appel direct, sans boîte** (aucune suppression ne peut suivre : le statut `paid` est refusé au
  rang 1, avant toute écriture ; si la facture est redevenue validée dans une autre session, l'annulation du règlement a
  incrémenté sa version, `supplier_invoices.rs:2427`, et l'appel rend `OPTIMISTIC_LOCK_CONFLICT` au rang 2) → le refus
  s'affiche (`supplier-invoice-unvalidate-error`) et la fiche se relit.
Succès : `supplier-invoices-unvalidated` (validée) ou clé neuve `supplier-invoices-unvalidated-cancelled` (annulée) :
« Facture repassée en brouillon : son écriture d'achat et sa contre-passation ont été supprimées. » La fiche **client**
garde « Dévalider ». Les refus `SUPPLIER_INVOICE_PAID_RECONCILED` et `SUPPLIER_INVOICE_DETACHED_PAYMENT` relisent la
facture comme `SUPPLIER_INVOICE_PAID` (`isStaleState`, `+page.svelte:294-306`). Vitest T12 (page) : bouton par statut ×
rôle ; boîte pour `open`/`cancelled` et corps par statut ; aucun dialogue pour `paid` ; message de refus affiché avec sa
date.

**AC-13 — E2E du parcours.** Playwright, dans `frontend/tests/e2e/supplier-invoice-draft-cycle.spec.ts` (T13) :
1. facture saisie, validée, annulée → fiche annulée → clic `supplier-invoice-unvalidate` → boîte
   (`supplier-invoice-unvalidate-dialog` contient « deux trous ») → clic `supplier-invoice-unvalidate-confirm` → badge
   *Brouillon*, *Modifier* proposé → (administrateur) *Supprimer le brouillon* → confirmation → retour à la liste,
   facture absente ;
2. facture payée depuis la fiche → clic `supplier-invoice-unvalidate` (pas de boîte) → `supplier-invoice-unvalidate-error`
   contient « annulez d'abord son règlement » ;
3. facture payée par la réconciliation (parcours existant de la spec, `:400-412`, dont l'assertion `toHaveCount(0)` est
   réécrite) → clic → message « … transaction bancaire du JJ.MM.AAAA : annulez d'abord ce rapprochement … ».

**AC-14 — Manuels, CHANGELOG, captures** : § *Manuel*, § *CHANGELOG*, § *Captures* ci-dessous, réalisés ; PDF régénérés
et contrôlés aplatis (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, grep des phrases neuves **et** absence des phrases
retirées).

**AC-15 — Clés i18n et replis : listes closes, quatre locales, vocabulaire.** Dans
`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` :
- **neuves (5)** : `error-supplier-invoice-unvalidate-blocked-paid-reconciled`,
  `error-supplier-invoice-unvalidate-blocked-detached-payment`,
  `error-supplier-invoice-unvalidate-blocked-detached-payment-reconciled`,
  `supplier-invoices-unvalidate-confirm-body-cancelled`, `supplier-invoices-unvalidated-cancelled` ; toute clé de plus
  se déclare au Dev Agent Record, avec le recompte ;
- **valeur modifiée (18)** — la liste `K` du contrôle ci-dessous, numéros fr-CH : **`audit-log-action-supplier-invoice-unvalidated`**
  (`:3053`, « Facture fournisseur repassée en brouillon » ; de `:2980`, it `:2981`, en `:2986` — libellé du journal d'audit,
  ajouté en remédiation P3, C-v0172a-24), `error-supplier-invoice-must-be-unvalidated-first`
  (`:50`, « …repassez-la d'abord en brouillon, puis supprimez le brouillon. »), `error-supplier-invoice-unvalidate-blocked-paid`
  (`:52`), `…-in-payment-batch` (`:53`), `…-matched` (`:54`), `supplier-invoices-unvalidate` (`:2659`, « Repasser en
  brouillon »), **`supplier-invoices-validate-confirm-body`** (`:2662`, « …pour la corriger ensuite, il faudra la repasser
  en brouillon. »), `…-unvalidate-confirm-title` (`:2663`), `supplier-invoices-unvalidated` (`:2668`),
  `…-blocked-not-validated` (`:2671`), `…-blocked-fiscal-year-closed` (`:2672`), `…-blocked-later-fiscal-year-closed`
  (`:2673`), `…-blocked-reversed` (`:2674`), `…-blocked-period-locked` (`:2675`), `…-blocked-lettered` (`:2676`),
  `supplier-invoices-unvalidate-error` (`:2677`), **`supplier-invoices-edit-not-draft`** (`:2679`, « Seul un brouillon se
  modifie : repassez d'abord la facture en brouillon. »), `…-blocked-later-fiscal-year-closed-named` (`:2692`) ;
  `:2664` (`…-unvalidate-confirm-body`), `:2670` (`…-blocked-stale`) et `:2689-2691` (avertissement du lot confirmé) ne
  portent pas le verbe et restent ;
- **typographie** (V3-4) : chaque clé garde l'apostrophe de sa valeur actuelle — typographique `’` pour la famille
  `supplier-invoices-*` (`:2662-2692`), droite `'` pour `error-supplier-invoice-*` (`:50-54`) et pour le libellé d'audit ;
  les clés neuves prennent celle de leur famille ; les textes cités dans cette fiche avec `'` se recopient avec
  l'apostrophe de la famille, et le repli suit le FTL caractère pour caractère ;
- **vocabulaire** (C-v0172a-20) : fr « repasser en brouillon », de « in den Entwurf zurücksetzen », it « riportare in
  bozza », en « return to draft ».

**Contrôle des catalogues, décidable par liste de clés** (V2-1). Un motif global ne décide rien : en italien
`convalida`/`convalidata` est aussi la racine de *valider* (`docs/i18n-glossaire.md:95`, `supplier-invoices-validated`,
`-draft-info` doivent le garder), et en allemand le verbe se fléchit et se sépare (« Freigabe aufgehoben », « Heben Sie
zuerst die Freigabe … auf », « entvalidieren »). Le contrôle ne lit donc **que les 18 clés de la liste**, et cherche dans
leur valeur la racine de *dévalider* propre à chaque langue : fr `[Dd][ée]valid`, en `[Uu]nvalidat`, de
`Freigabe|[Ee]ntvalid`, it `[Cc]onvalida\b` (le **nom** ; `convalidata`, l'état validé, n'est pas visé). Commande, à
recopier au Dev Agent Record avec ses sorties :

```sh
K='^(error-supplier-invoice-must-be-unvalidated-first|error-supplier-invoice-unvalidate-blocked-(paid|in-payment-batch|matched)|supplier-invoices-unvalidate|supplier-invoices-unvalidate-confirm-title|supplier-invoices-unvalidated|supplier-invoices-unvalidate-blocked-(not-validated|fiscal-year-closed|later-fiscal-year-closed|reversed|period-locked|lettered|later-fiscal-year-closed-named)|supplier-invoices-unvalidate-error|supplier-invoices-validate-confirm-body|supplier-invoices-edit-not-draft|audit-log-action-supplier-invoice-unvalidated) = '
for x in "fr-CH:[Dd][ée]valid" "en-CH:[Uu]nvalidat" "de-CH:Freigabe|[Ee]ntvalid" "it-CH:[Cc]onvalida\b"; do
  loc=${x%%:*}; pat=${x#*:}; f=crates/kesh-i18n/locales/$loc/messages.ftl
  echo "$loc clés=$(/usr/bin/grep -cE "$K" "$f") verbe=$(/usr/bin/grep -E "$K" "$f" | /usr/bin/grep -cE "^[a-z-]+ = .*($pat)")"
done
```

**Sonde positive, relevée par l'agent de fiche** sur `d7ec2ea2` (même commande, `git show d7ec2ea2:<fichier>`) :
`clés=18 verbe=14` dans **chacune** des quatre locales (les quatre clés sans le verbe : `-blocked-paid`,
`-in-payment-batch`, `-matched`, `-not-validated` ; relevé refait en remédiation P3 avec la 18ᵉ clé — il valait `clés=17
verbe=13` à dix-sept clés). **À la tête : `clés=18 verbe=0` partout.** Une valeur `clés` ≠ 18 dit
qu'une clé a été renommée ou supprimée : le contrôle ne vaut plus, il se corrige avant d'être cru.

**Contrôle des replis, liste close** (V2-2). Les replis en dur qui portent le verbe, relevés à `d7ec2ea2` par
`git grep -niE "d[ée]valid" d7ec2ea2 -- 'frontend/src/routes/(app)/supplier-invoices' 'frontend/src/lib/features/supplier-invoices' crates/kesh-api/src/errors.rs`,
tests et commentaires écartés, **14 lignes** :
- `frontend/src/lib/features/supplier-invoices/supplier-invoice-unvalidate.ts:91, 99, 105, 110, 115, 120, 123` (7) ;
- `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte:392, 676, 977, 994, 1023` (5) ;
- `frontend/src/routes/(app)/supplier-invoices/[id]/edit/+page.svelte:88` (1) ;
- `crates/kesh-api/src/errors.rs:4115` (bras `SupplierInvoiceMustBeUnvalidatedFirst`) (1).
Chacun reprend mot pour mot la nouvelle valeur fr-CH. À la tête, le même grep, sur les mêmes fichiers, ne rend plus
que des **commentaires** (`+page.svelte:654-655`, `edit/+page.svelte:5`, à mettre à jour s'ils décrivent le geste) et
les lignes du code client hors de ces dossiers ; sortie triée au Dev Agent Record.
- `docs/i18n-glossaire.md:96` : la ligne « dévalider » ne dit plus que `supplier-invoices-unvalidate` la reprend ; une ligne
  neuve « repasser en brouillon » porte les quatre termes et les clés fournisseur.

**AC-16 — Le contrat de l'API externe est à jour.** `docs/api-external.md` : `:565` (la ligne `unvalidate` du tableau :
« `status: "draft"` ; l'écriture d'achat est supprimée — et, pour une annulée, sa contre-passation »), `:572` (une annulée
n'est plus en `ILLEGAL_STATE_TRANSITION` ; précédence par écriture ; deux codes neufs ; les cinq champs de `details`),
`:869` (le paragraphe de la dévalidation), `:963` (ligne `SUPPLIER_INVOICE_PAID`, `details` complétés) et deux lignes
neuves au tableau des codes (`SUPPLIER_INVOICE_PAID_RECONCILED`, `SUPPLIER_INVOICE_DETACHED_PAYMENT`), avec la mention
*(depuis la v0.17.2)* ; l'audit `supplier_invoice.unvalidated` y gagne `previousStatus` et `reversalJournalEntryId` s'il
y est décrit. Et deux lignes qui citent le refus de suppression d'une validée (V3-5) : `:574` (« facture validée : dévalidez-la
d'abord ») et `:961` (« la dévalider d'abord ») → « repassez-la d'abord en brouillon ». Contrôle :
`git grep -nF "(brouillon, annulée)" -- docs/api-external.md` vide ; `git grep -niE "d[ée]valid" -- docs/api-external.md`
trié au Dev Agent Record (restent les lignes des factures clientes) ;
`git grep -nE "PAID_RECONCILED|DETACHED_PAYMENT" -- docs/api-external.md` rend au moins deux lignes.

## Tests et mutations

| Test | Où | AC |
|---|---|---|
| T1, T2, T3a-e, T4 (P1-P6), T5a-e, T6, T9, T9b, T10, T11, T14 | `crates/kesh-db/tests/supplier_invoices_draft.rs` (helpers `decor`, `validee`, `brouillon`, `en_lot`, `rapprochee`, `fournisseur_rapproche::*` existants) | 1-7, 9-11, C-v0172a-16 |
| T7 | `crates/kesh-api/tests/supplier_invoice_draft_e2e.rs` (route `POST …/unvalidate` : 200 sur une annulée, 409 par variante, `details`, message) | 8 |
| T7b | test unitaire de `crates/kesh-api/src/errors.rs` (repli sans catalogue) | 8 |
| T8 | `frontend/src/lib/features/supplier-invoices/supplier-invoice-unvalidate.test.ts` | 8 |
| T12 | `frontend/src/routes/(app)/supplier-invoices/[id]/supplier-invoice-draft-page.test.ts` | 12 |
| T13 | `frontend/tests/e2e/supplier-invoice-draft-cycle.spec.ts` | 13 |

**T14** (C-v0172a-16) : facture validée, annulée, puis `UPDATE journal_entries SET reverses_entry_id = NULL` sur sa
contre-passation (SQL de test, dit au commentaire) → repasser en brouillon rend `DbError::Invariant`, rien n'est écrit
(facture `cancelled`, deux écritures présentes, lettrage inchangé, aucune trace d'audit neuve).

⚠️ **Tests existants à réécrire, non à supprimer — liste close** (relevée à `d7ec2ea2` par
`git grep -nE "unvalidat|Dévalider|annulez d'abord|\(Lot 4\)|sitesTotal" -- crates/kesh-db/tests crates/kesh-api/tests frontend/src frontend/tests`,
triée) :

| Site | Ce qu'il fige | Devient |
|---|---|---|
| `crates/kesh-db/tests/supplier_invoices_draft.rs:827-834` (`unvalidation_status_refusals`) | annulée → `IllegalStateTransition` | le succès d'AC-1 ; le brouillon reste le seul `IllegalStateTransition` — **premier rouge** |
| `crates/kesh-db/tests/supplier_invoices_draft.rs:1165-1172` (`delete_draft`) | annulée → `IllegalStateTransition` | **inchangé** (une annulée ne se supprime pas directement) |
| `crates/kesh-api/tests/supplier_invoice_draft_e2e.rs:395-411` | payée depuis la fiche → `SUPPLIER_INVOICE_PAID` | reste vrai (règlement non rapproché) ; le message, s'il est comparé, suit AC-8 ; un cas `PaidReconciled` s'ajoute (T7) |
| `supplier-invoice-draft-page.test.ts:160-235` | bouton par statut et rôle | bouton pour `paid` et `cancelled` aux rôles qui écrivent |
| `supplier-invoice-draft-page.test.ts:296-301` | refus du lot suffixé « (Lot 4) » | texte d'AC-8, lot nommé par `details.batchId` |
| `supplier-invoice-draft-page.test.ts:394-398` | ancien texte de `SUPPLIER_INVOICE_PAID` | texte d'AC-8 ; et la séquence `paid` sans boîte (AC-12) |
| `supplier-invoice-draft-page.test.ts:490-501` | Consultation : pas de bouton | inchangé |
| `supplier-invoice-draft-page.test.ts:574-576` | texte de `LATER_FISCAL_YEAR_CLOSED` nommé (« pour pouvoir dévalider cette facture ») | nouvelle valeur (AC-15) ; ajouté en remédiation P2 |
| `supplier-invoice-unvalidate.test.ts:33` (`REPLI`) | « La facture n’a pas pu être dévalidée. » | nouvelle valeur de `supplier-invoices-unvalidate-error` ; ajouté en remédiation P2 |
| `supplier-invoice-unvalidate.test.ts:47-56` | liste exacte des dix codes | douze codes |
| `supplier-invoice-unvalidate.test.ts:62-75` | « mot pour mot le FTL, suffixé du numéro de la pièce » | mot pour mot le FTL, **arguments** substitués, sans suffixe |
| `frontend/src/lib/shared/i18n-keys.test.ts:837` (`sitesTotal: 2796`) et ses compteurs par fichier (`:810-840`) | nombre d'appels `i18nMsg(` | **recomptés à la source** aux deux bornes (`d7ec2ea2` → tête), la variation écrite dans le commentaire comme les précédentes |
| `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts:204-212` | décompte des clés de la famille | recompté, variation écrite (clés neuves d'AC-15) |
| `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts:267` (`CANDIDATES_ATTENDUES = 77`, asserté `:762`, `:787` ; préambule `:255-265`, dont `withLabel` de `supplier-invoice-unvalidate.ts`) | nombre de fonctions candidates (suffixes `Label`, `Text`, `Display`) | **recompté à la source** aux deux bornes (`d7ec2ea2` → tête) par l'extraction de son préambule : `withLabel` disparaît si le suffixe du numéro de facture est retiré (AC-8), d'autres fonctions en `*Label`/`*Text`/`*Display` peuvent apparaître ; variation et noms écrits dans le commentaire, entrées passées non réécrites ; ajouté en remédiation P3 (V3-2) |
| `frontend/tests/e2e/supplier-invoice-draft-cycle.spec.ts:365-372` | boîte de la dévalidation contient « trou » | reste vrai (« trou » ⊂ « deux trous ») ; le libellé du bouton change |
| `frontend/tests/e2e/supplier-invoice-draft-cycle.spec.ts:412` | payée par la réconciliation : `toHaveCount(0)` | le parcours 3 d'AC-13 |

Tout autre test qui rougit au gate se juge et se déclare au Dev Agent Record (ce relevé est un grep, pas une exécution).

**Rouges d'abord** : T1, T5a, T6, T7, T7b, T8, T12, T14 écrits et constatés rouges avant le code (sortie au Dev Agent Record).

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
| M6c | la lecture d'audit ne retient que la première trace d'annulation | T5e |
| M7 | détection du règlement rapproché retirée (`Paid` partout) | T6, T10 |
| M8 | dans l'`UPDATE` (tâche 3.8, `AND status = ?`), lier `'open'` au lieu du statut lu | T1 |
| M9 | contrôle de rapprochement sur la contre-passation retiré | T3e (variante contre-passation rapprochée) |
| M10 | bouton masqué pour `cancelled` | T12 |
| M11 | `transactionDate` absent du message frontend | T8 |
| M12 | **pré-contrôle d'exercice clos** (C-v0172a-4) retiré | T4 *P4* : sans lui, la contre-passation sous le verrou parle d'abord (`PERIOD_LOCKED`) au lieu de `FISCAL_YEAR_CLOSED` |
| M13 | repli serveur passé à `t_args` sans `format!` (arguments non substitués) | T7b |
| M14 | identifiant de la contre-passation lu par `reversed_by` (lecture ordinaire) avant les verrous | **non tuable par un test séquentiel** — c'est une règle de sérialisation ; vérifiée par lecture en revue (ligne citée au Dev Agent Record), comme `cancel_in_tx` |

## Tâches

1. **Rouges** : T1, T5a, T6, T7, T7b, T8, T12, T14 écrits, rouges constatés ; `unvalidation_status_refusals` réécrit (rouge).
2. **`errors.rs` (kesh-db)** : variantes `PaidReconciled`, `DetachedPayment`, leurs `code()` ; doc-comment de l'énumération
   (cinq variantes, précédence par écriture). `DbError::SupplierInvoiceNotUnvalidatable` porte les détails d'AC-8 (forme
   libre : par exemple un champ `hint` — `batch_id`, `transaction_date`, `entry_id`, `entry_number`, `entry_date` — sans
   casser les appelants existants).
3. **Dépôt** — `unvalidate_in_tx`, ordre **à respecter** (« les verrous d'abord, tous, avant toute lecture non
   verrouillante », leçon 25-3-b, `supplier_invoices.rs:2034-2039`) :
   1. facture `FOR UPDATE` ; statut : `open` | `cancelled` passent ; `paid` → lecture **verrouillante**
      (`LOCK IN SHARE MODE`) de la transaction rapprochée de l'écriture de règlement → `PaidReconciled` (avec sa date) ou
      `Paid` ; brouillon → `IllegalStateTransition` ; version ;
   2. lot `generated` (existant, lecture verrouillante) ;
   3. **branche annulée — verrous** : écriture d'achat **et** son exercice `FOR UPDATE` (requête jointe de
      `cancel_in_tx:2074-2084`, étendue au statut et à la date) ; **puis la contre-passation et son exercice par une
      lecture verrouillante jointe** — `SELECT je.id, je.entry_date, fy.id, fy.status FROM journal_entries je JOIN
      fiscal_years fy ON fy.id = je.fiscal_year_id WHERE je.reverses_entry_id = ? AND je.company_id = ? FOR UPDATE` —, et
      **non** par `reversed_by`, lecture ordinaire (`journal_entries.rs:2455-2471`) qui ouvrirait l'instantané avant les
      verrous (V1-1) ; aucune ligne → `Invariant` (C-v0172a-16) ; transactions rapprochées de l'achat puis de la
      contre-passation (`LOCK IN SHARE MODE`, avec `booking_date`) ;
   4. **branche annulée — lectures ordinaires, après le dernier verrou** : la lecture d'audit du paiement détaché (AC-6),
      puis, pour chaque écriture qu'elle nomme, son existence, son numéro et sa date, `reversed_by` (sans verrou : une
      contre-passation ne s'annule pas ; une contre-passation concurrente ne fait que rendre le refus conservateur) et sa
      transaction rapprochée éventuelle ;
   5. **refus**, dans l'ordre du tableau (rangs 3 à 6) : lot, paiement détaché, rapprochement (achat, puis
      contre-passation), pré-contrôle d'exercice clos (achat ou contre-passation → `FiscalYearClosed`) ;
   6. instantané (`fetch_lines`, écart déjà assumé) ;
   7. branche annulée : dissoudre, par `letterings::dissolve_group_in_tx(…, Mode::System { held_open_fiscal_year_id:
      <exercice de la contre-passation> }, …)`, **chaque** groupe d'origine `reversal` dont **toutes** les lignes
      appartiennent à l'achat ou à sa contre-passation (sélection par `lettering_key` des lignes des deux écritures, puis
      contrôle que le groupe n'a aucune ligne ailleurs) ; aucun autre groupe ;
   8. `UPDATE … status = 'draft', purchase_journal_entry_id = NULL, version = version + 1 WHERE … AND version = ? AND
      status = ?` (le statut **lu** en 1) ;
   9. `delete_in_tx(contre-passation)` puis `delete_in_tx(achat)`, `enforce_ownership = false` (rangs 7a et 7b) ;
   10. audit (AC-9) ; relecture.
   Fragment SQL partagé du paiement détaché, employé aussi par `modification_guard` (AC-6, V2-12). Doc-comments à jour : module
   (`supplier_invoices.rs:1-17`), `unvalidate_in_tx` (précédence par écriture, verrous, branche annulée), `cancel_in_tx`
   (l'annulée n'est plus définitive), `delete_in_tx` (§ `enforce_ownership`, deux suppressions par geste),
   `modification_guard` (réserve L2), `SupplierInvoiceNotUnvalidatable` ; et deux doc-comments qui **deviennent faux**
   (V2-11) : `journal_entries.rs:1775-1777` (3-quinquies « Inatteignable par la dévalidation aujourd'hui ») et
   `kesh-db/src/errors.rs:263-266` (« inatteignable par la dévalidation … n'est pas comptée parmi les refus », C132) — la
   marque est atteignable pour une annulée lettrée hors de ses paires (AC-4 d, P6).
4. **API** : `supplier_unvalidation_blocked_text` (`kesh-api/src/errors.rs:4714-4733`) et le bras
   `SupplierInvoiceNotUnvalidatable` (`:4137-4159`) : repli par `format!`, message par `t_args` (AC-8), `details`
   d'AC-8 ; doc-comment de la route.
5. **i18n** : clés d'AC-15 dans les quatre locales ; `npm run lint-i18n-ownership` ; tests i18n du frontend
   (`i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`, `i18n-libelle-en-dur.test.ts`) verts, **`sitesTotal` et les
   compteurs par fichier de `i18n-keys.test.ts`, le décompte de `i18n-un-repli-par-cle.test.ts` et
   `CANDIDATES_ATTENDUES` de `i18n-libelle-en-dur.test.ts:267`, recomptés à la source** (aux deux bornes, la variation écrite dans le commentaire, entrées passées non réécrites) ;
   `docs/i18n-glossaire.md` (AC-15).
6. **Frontend** : `supplier-invoice-unvalidate.ts` (douze codes, arguments, replis = FTL fr-CH) ; `+page.svelte` (bouton
   pour `open`/`paid`/`cancelled`, boîte pour `open`/`cancelled`, appel direct pour `paid`, succès par statut,
   `isStaleState`) ; commentaires à jour (`ENTRY_IS_REVERSED` reste « inatteignable », pour une autre raison).
7. **Documentation** : manuels (§ *Manuel*), `docs/api-external.md` (AC-16), CHANGELOG, captures.
8. **Greps de propagation** (CLAUDE.md, « greper la valeur »), **insensibles à la casse**, résultats triés
   client/fournisseur au Dev Agent Record :
   - `git grep -niE "d[ée]valid" -- frontend/src crates/kesh-i18n/locales crates/kesh-api/src/errors.rs docs/manual/fr docs/api-external.md docs/i18n-glossaire.md` ;
   - `git grep -niE "freigabe aufheben|convalida|unvalidat" -- crates/kesh-i18n/locales` (valeurs fournisseur seulement) ;
   - `git grep -nE "annulée.*IllegalState|cancelled.*IllegalState"` ; `git grep -nF "ne supprime aucune facture annulée"` ;
     `git grep -nF "sa fiche ne le propose pas"` ; `git grep -nF "ni ne se dévalide"` ;
   le nom de code (`unvalidate`, `supplier_invoice.unvalidated`) reste ; le vocabulaire client (« Dévalider ») aussi.
9. **Gates** : gate complet (`scripts/test-fast.sh`, base `kesh_v0172a` remise à neuf), `npm run check`,
   `lint-i18n-ownership`, `test:unit`, `build`, **E2E complet au dernier commit de code** (port 3172, `kesh_e2e_v0172a`,
   dossiers `/tmp/kesh-e2e-v0172a-r<n>/…`), échecs jugés contre `docs/testing.md` § *Les échecs attendus*. Exception
   `kesh-db` : patch de dépôt ⇒ gate complet même en cours de boucle.

## Manuel

**Utilisateur** (`docs/manual/fr/user-manual.tex`, puis `make fr` — PDF commité). Le vocabulaire fournisseur devient
« repasser en brouillon » (C-v0172a-20) ; « dévalider » reste le mot des factures **clientes** et le nom du concept dans
les passages généraux (`:873`, exercices dans le désordre). Inventaire clos des passages fournisseur, relevé à
`d7ec2ea2` par `/usr/bin/grep -niE "d[ée]valid" docs/manual/fr/user-manual.tex` (46 lignes, triées) :
- § *Dévalider une facture validée* (`:1718-1751`) → **« Repasser une facture en brouillon »** : le geste s'appelle
  *Repasser en brouillon* (bouton de la fiche, comptable ou administrateur) ; il vaut pour une facture **validée** (texte
  actuel) et, nouveau, pour une **annulée** (deux écritures supprimées, deux trous, ce que garde l'audit, la lecture du
  § *Piste d'audit* résumée en trois phrases — CO 957a et 958f, OLICo art. 9, **sans** l'art. 3 de l'Olico (C-v0172a-18)
  —, renvoi à la fiduciaire) ; une **payée** : la fiche le propose, et le refus dit quoi faire — annuler le règlement, ou,
  payée par la réconciliation, annuler le rapprochement depuis le détail de l'import bancaire
  (`sec:annuler-rapprochement`), puis repasser en brouillon ; une annulée **qui avait été payée** : contre-passer d'abord
  le paiement — seulement s'il n'a pas eu lieu — ; la liste des refus mise à jour (lot nommé, date de la transaction).
  Remplacer « Une facture payée ne se dévalide pas, et sa fiche ne le propose pas » (`:1729-1730`), et « au-dessus de
  \emph{Dévalider} … pour qui peut dévalider … la confirmation de la dévalidation » (`:1747-1749`).
- § *Le brouillon* (`:1697-1705`, dont `:1701` « puis dévalidé ») : un brouillon issu d'une annulée qui a figuré dans un
  lot ne se supprime pas davantage ; la phrase « s'il n'est pas dû, validez-le puis annulez-le » reste juste.
- § *Régler une facture fournisseur* (`:1811`) : « annulez le règlement, puis dévalidez la facture pour la corriger » →
  « …puis repassez-la en brouillon pour la corriger ».
- § *La corriger au centime* (`:1844-1859`) : les cinq « dévalid… » → « repasser en brouillon ».
- § *Annuler une facture fournisseur* (`:1881-1884`, `:1892`) : « Annuler ou dévalider ? » devient « Annuler ou repasser
  en brouillon ? » ; « La 0.17.0 ne supprime aucune facture annulée » est remplacé par : annuler laisse l'achat et sa
  contre-passation aux livres ; depuis la 0.17.2, une annulée saisie par erreur **repasse en brouillon** (puis se corrige,
  ou se supprime), sous les gardes dites.
- **`:1917`** : « Une facture annulée ne se désannule pas, ni ne se dévalide : si l'annulation était une erreur,
  saisissez-la de nouveau. » — **devient faux** : « Une facture annulée ne se désannule pas ; si l'annulation était une
  erreur, repassez-la en brouillon (§ …), puis validez-la de nouveau. »
- **`:3762-3766`** (corrections, liste par pièce) : « une facture validée dont la saisie est fausse se dévalide, se corrige
  et se revalide » → « …se repasse en brouillon, se corrige et se revalide » ; et l'annulée y gagne sa phrase (« une
  facture annulée par erreur se repasse en brouillon »). Ligne absente de la première rédaction (V2-3).
- **TVA et périodes** (V2-5), dans le § *Repasser une facture en brouillon* : la suppression de l'achat et de sa
  contre-passation change les totaux de deux périodes — celle de la facture et celle de l'annulation —, TVA du trimestre
  comprise ; **verrouiller une période déclarée avant** de repasser en brouillon une annulée dont l'achat ou
  l'annulation y tombe (renvoi `sec:verrou-periode`) ; le verrou, posé, refuse le geste.
- Mentions croisées : `:683-687`, `:765-767` (numérotation : deux trous pour une annulée), `:2117`, `:2133`.

**Administration** (`docs/manual/fr/admin-manual.tex`). Inventaire clos relevé à `d7ec2ea2` par
`/usr/bin/grep -niE "d[ée]valid" docs/manual/fr/admin-manual.tex` (V2-3) — **dix lignes**, triées :
`:1479` (clôture ordonnée, général), `:2339` (exercices dans le désordre, général), `:2529` (montée depuis la
0.17.0-rc1, historique), `:3054` (verrou de période, « pas même en dévalidant la facture », général) et `:3609` (rappels
clients) **restent** ; changent :
- `:2546` (facture au total sous le centime, « se corrige en la dévalidant ») → « en la repassant en brouillon » ;
- `:2550` (« se dévalide par `POST …/unvalidate` tant qu'elle n'est pas payée ») → elle repasse en brouillon par la même
  route, validée **ou annulée**, et pour une annulée l'écriture d'achat **et** sa contre-passation sont supprimées ;
- **`:3010`**, § des clés `read-write` (avertissement de sécurité : ce qu'une clé d'intégration peut effacer) : depuis la
  v0.17.2, une clé `read-write` peut aussi repasser en brouillon une facture fournisseur **annulée**, donc supprimer
  **deux** écritures, sous les refus dits (dont le paiement détaché non contre-passé), trace `supplier_invoice.unvalidated`
  avec `previousStatus` ; C-v0172a-11 le nomme ;
- `:3058` (liste des traces) : ajouter `supplier_invoice.unvalidated` à côté d'`invoice.unvalidated` ;
- `:3057`, § *Conformité OLICo Art. 9* : la seconde voie (« la dévalidation d'une facture ») inclut désormais, pour une
  facture fournisseur **annulée**, la suppression de l'écriture d'achat **et de sa contre-passation**, journalisée
  (`supplier_invoice.unvalidated` avec `previousStatus`, `journal_entry.deleted` ×2), refusée sous les mêmes motifs, plus
  le paiement détaché non contre-passé ; la liste des motifs fournisseur mise à jour ;
- `:3073`, réserve de la fusion d'audit : ajouter que la même trace étrangère peut aussi **refuser de repasser en
  brouillon** une facture fournisseur annulée restaurée, en désignant un paiement sans rapport (limite L2).
PDF régénérés, contrôlés aplatis : les phrases neuves présentes, et absentes à la tête « se dévalide, se corrige et se
revalide », « dévalider une facture fournisseur », « se dévalide par », « ne supprime aucune facture annulée », « ni ne
se dévalide ».

**Hors des manuels** (V3-5), même vocabulaire :
- `README.md:42` (Fonctionnalités) : « validée et non payée, elle se **dévalide** pour être corrigée (v0.17.0) » → « elle
  se **repasse en brouillon** pour être corrigée — annulée aussi, depuis la v0.17.2 » ;
- `docs/manual/fr/marketing-brochure.tex:449` **et son PDF** (`marketing-brochure.pdf`, régénéré par `make fr`) : « validée
  et non payée, elle se dévalide pour être corrigée, sans annulation ni contre-passation » → même correction ;
- `website/` : relevé propre à `d7ec2ea2` (`git grep -niE "d[ée]valid" -- website` vide, P3) — à rejouer.
Ce sont des éléments de la liste de contrôle pré-release (CLAUDE.md, points 1 et 4) : ils se font **dans cette story**,
avec le code qui les motive, non à la release.

Manuels DE/IT/EN : vides (`docs/manual/{de,en,it}/README.md` seulement) — rien à traduire.

## CHANGELOG

Sous `## [0.17.2] — Non publié` (créer l'en-tête s'il n'existe pas ; ne pas dater) :

- **Ajouté** — « **Une facture fournisseur annulée repasse en brouillon** ([#868]) : *Repasser en brouillon*, sur sa
  fiche, retire du journal son écriture d'achat et sa contre-passation — deux trous dans la numérotation, expliqués par
  le journal d'audit —, sous les gardes de la dévalidation (exercice ouvert, période non verrouillée, aucune écriture
  rapprochée ni lettrée ailleurs). Le brouillon se corrige et se revalide, ou se supprime. Une facture annulée alors
  qu'elle était payée ne repasse en brouillon qu'une fois son paiement contre-passé. ⚠️ Les totaux de la période de
  l'achat et de celle de l'annulation changent (TVA comprise) : verrouillez d'abord une période déjà déclarée. *Refs #868.* »
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
- **Verrous** : facture → (payée : transaction du règlement, S) → articles de lot (S) → (achat, exercice) →
  (contre-passation, exercice), trouvée par une lecture **verrouillante** sur `reverses_entry_id` → transactions
  bancaires de l'achat et de la contre-passation (S) → *lectures ordinaires* (audit du paiement détaché, `reversed_by` de
  ce paiement, `fetch_lines`) → lignes (par la dissolution) → `UPDATE` facture → `delete_in_tx` ×2 (qui reprennent des
  verrous déjà tenus, puis les exercices postérieurs et les lignes). Cycle connu lignes ↔ écriture avec un lettrage
  concurrent : couvert par le rejeu de la route. **Aucune lecture ordinaire avant le dernier verrou d'écriture et
  d'exercice** : l'identifiant de la contre-passation ne se lit pas par `reversed_by` (V1-1, M14) ; `fetch_lines` vient
  après, et son écart d'origine (`:898-906`) ne s'étend pas.
- **Contraintes** (V2-11) : l'unique `UPDATE` suffit parce que `chk_supplier_invoices_entry_iff_not_draft` **et**
  `chk_supplier_invoices_draft_without_settlement` (`20261031000001_supplier_invoices_draft.sql:46-49` : un brouillon n'a
  aucune colonne `settlement_*` ni `paid_at`) sont tenues : `cancel_in_tx` a déjà vidé ces colonnes
  (`supplier_invoices.rs:2148-2152`).
- **Exercices postérieurs** : `delete_in_tx` les verrouille après l'ouverture de l'instantané (lectures ordinaires de la
  tâche 3.4). Sans effet : son verdict 2-bis est une lecture **verrouillante** (lecture courante), et `reversed_by` de
  l'achat ne peut plus changer (déjà contre-passé ; une contre-passation ne se contre-passe pas).
- **Ne pas** toucher `delete_in_tx` (hors doc), `cancel_in_tx`, `delete_draft_in_tx`, `reconciliation_cancel` : le
  périmètre est le dépôt de la dévalidation, la lecture partagée du paiement détaché, l'API, l'écran, les textes.
- Le seed (`kesh-seed`) n'est pas modifié.
- **Rôles** : inchangés (Comptable+ pour repasser en brouillon, Administrateur pour supprimer un brouillon), clés d'API
  `read-write` comme aujourd'hui ; le serveur MCP n'écrit pas — rien à faire.

## Pour l'orchestrateur

- **Issue à ouvrir (on trace toujours)** — *Factures clientes annulées : aucun retour au brouillon* (enhancement, P4,
  `triage`) : « `invoices::unvalidate` refuse une facture cliente `cancelled` (`invoices.rs:1520-1526`). Une facture
  cliente n'est annulée que par un avoir — document numéroté, avec son écriture, souvent envoyé ; l'avoir est la
  contre-passation, et la facture `cancelled` est refusée par son statut. L'écart avec les factures fournisseur (0.17.2, #868) est voulu par construction ; l'issue le
  trace pour que le Project Lead décide s'il faut un geste (supprimer un avoir jamais envoyé ?) ou rien. »
- **Commentaire à porter sur #868** : la reformulation (on ne supprime qu'un brouillon ; l'annulée y arrive par
  *Repasser en brouillon*) et le sort des quatre points de conception (§ *Origine*).
- Prémisse du mandat corrigée : « Sort du document rattaché : le même que pour la dévalidation d'une `open` » — vérifié :
  la dévalidation ne touche pas au document ; il reste joint au brouillon et ne revient « à traiter » qu'à la suppression.

## Change Log

| Date | Version | Description | Auteur |
|---|---|---|---|
| 2026-11-04 | 0.1 | Fiche créée en autonomie (C-v0172a-1 à 18) ; validation P1 à lancer (`v0172-annulees-validate-prompt-p1.md`, Sonnet). | Opus 5.5 |
| 2026-11-04 | 0.2 | **Validation P1 (Sonnet) : 0/0/7/8**, remédiée (Opus 5.5) ; les quinze findings appliqués, aucun réfuté (prémisses relues au code). MEDIUM : V1-1 identifiant de la contre-passation par lecture verrouillante après le verrou de l'achat (tâche 3.3, Dev Notes, M14) ; V1-2 précédence composée par écriture, contre-passation d'abord, paires P1-P6 (AC-5, tableau, C-v0172a-19) ; V1-3 M12 n'est plus déclarée équivalente, tuée par P4 (C-v0172a-4 annoté) ; V1-4 liste close des tests existants à réécrire (13 sites, dont `sitesTotal` et le décompte de `i18n-un-repli-par-cle`) ; V1-5 `docs/api-external.md` et `docs/i18n-glossaire.md` (AC-15, AC-16) ; V1-6 vocabulaire « repasser en brouillon » dans toutes les valeurs fournisseur, grep insensible à la casse, manuel `:1811`, `:1917` (C-v0172a-20) ; V1-7 repli serveur par `format!`, T7b, M13. LOW : V1-8 limite L2 et manuel d'administration `:3073` ; V1-9 cinq numéros de ligne ; V1-10 lot avant paiement détaché ; V1-11 cinq champs de `details`, écriture nommée par sa date, chemins de menu de l'écran (C-v0172a-22) ; V1-12 contextes de C-11 et C-13, liste close des clés (AC-15), T14 pour l'`Invariant` ; V1-13 Olico art. 3 retiré du manuel, marqué non vérifié ; V1-14 payée sans boîte, séquences E2E explicites (C-v0172a-21) ; V1-15 justification côté clientes corrigée. Registre : C-v0172a-19 à 22. Prochaine : validation P2 (Opus, passe complète, `v0172-annulees-validate-prompt-p2.md`). | Opus 5.5 |
| 2026-11-04 | 0.3 | **Validation P2 (Opus) : 0/1/4/8**, remédiée (Opus 5.5) ; les treize findings appliqués, aucun réfuté (prémisses relues : catalogues des quatre locales, replis, manuels, `git grep` à `d7ec2ea2`). HIGH : V2-1 contrôle de vocabulaire par **liste de 17 clés** et racine par langue (it `[Cc]onvalida\b`, de `Freigabe\|[Ee]ntvalid`), sonde positive relevée à `d7ec2ea2` (13 sur 17 dans chaque locale), vide attendu à la tête (AC-15). MEDIUM : V2-2 deux clés ajoutées (`validate-confirm-body`, `edit-not-draft`, 15 → 17) et liste close de 14 replis en dur, page d'édition et repli Rust compris ; V2-3 manuel utilisateur `:3762-3766` et inventaire clos des dix lignes du manuel d'administration (`:2546`, `:2550`, `:3010` clés `read-write`, `:3058` changent) ; V2-4 contrôle DRY sur le chemin JSON `'$.settlementJournalEntryId'` (une ligne, sonde à `d7ec2ea2`) ; V2-5 la paire n'est nette qu'au cumul : totaux de deux périodes, TVA comprise — fiche, confirmation à l'écran, manuel, CHANGELOG, T1 par période ; le verrou de période suffit (C-v0172a-23). LOW : V2-6 registre C-14 annoté ; V2-7 contextes de C-17 et C-18 ; V2-8 L1 étendue au rang 5 ; V2-9 assertion vide de T1 remplacée, M8 sur la forme prescrite ; V2-10 AC-4 (b) posé en SQL ; V2-11 contrainte `chk_…_draft_without_settlement` et deux doc-comments qui deviennent faux ; V2-12 prédicat partagé nommé (fragment SQL, deux requêtes) ; V2-13 trois sites ajoutés à l'inventaire (18 à 20) avec le critère de tri. Deux tests à réécrire ajoutés (`supplier-invoice-draft-page.test.ts:574-576`, `supplier-invoice-unvalidate.test.ts:33`). Prochaine : validation P3 (Sonnet, passe complète, `v0172-annulees-validate-prompt-p3.md`). | Opus 5.5 |
| 2026-11-04 | 0.4 | **Validation P3 (Sonnet) : 0/0/2/5**, remédiée (Opus 5.5) ; les sept findings appliqués, aucun réfuté (prémisses relues à `d7ec2ea2` : FTL des quatre locales `audit-log-action-supplier-invoice-unvalidated`, `i18n-libelle-en-dur.test.ts:255-267`, `test_fixtures.rs:71, 118`, `supplier_invoices_draft.rs:944-960`, `README.md:42`, `marketing-brochure.tex:449`, `api-external.md:563, 574, 961`, `kesh-api/src/errors.rs:4260-4266`). MEDIUM : V3-1 le libellé d'audit devient la **18ᵉ** clé de `K` (« Facture fournisseur repassée en brouillon »), sonde refaite à `d7ec2ea2` : `clés=18 verbe=14` dans chaque locale, `verbe=0` attendu à la tête (C-v0172a-24) ; V3-2 `CANDIDATES_ATTENDUES` (`i18n-libelle-en-dur.test.ts:267`) ajouté au tableau des tests à réécrire et à la tâche 5, recompte à la source. LOW : V3-3 topologie des exercices des montages (scission du décor en SQL relativement à l'année du jour ; T3b et P5 avec l'exercice clos **entre** les deux écritures) ; V3-4 typographie des apostrophes par famille de clés ; V3-5 `README.md:42`, brochure `:449` et son PDF, `api-external.md:574`, `:961` ; V3-6 C-5 et C-20 annotés, choix du parcours de toutes les traces consigné ; V3-7 refus générique de `DELETE` d'une annulée écarté par écrit (C-v0172a-25). Prochaine : validation P4 (Opus, passe ciblée sur le diff `ea098675` → tête, `v0172-annulees-validate-prompt-p4.md`). | Opus 5.5 |

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
