# Story 25.3-a-2 : Annuler un règlement fournisseur — par contre-passation

Status: ready-for-dev

**Issue : [#414]**, qu'elle **ferme** : `closes #414` dans le **titre ET le corps** de la PR (squash).
⛔ **Derrière la 25-3-a-1**, qui porte le côté client et **étend le socle** (autorité, motifs en
liste ordonnée) dont cette story a besoin : ne pas commencer avant son merge.

**Mère : `25-3-a-annuler-reglement.md`** (`split`) — **source des faits**, avec ses trois passes de
validation. Sœur : **25-3-a-1** — ses AC 1 à 3 (socle, datation, motifs) sont le **gabarit** de
celle-ci, qui les **réutilise** et ne les réécrit pas. Voisine : **25-3-c** (annuler une facture
fournisseur dans tous les cas, sauf exercice clos ; absorbe #454).

## Story

En tant que comptable,
je veux annuler le règlement d'une facture fournisseur enregistré par erreur, pour la ramener à
« ouverte »,
afin de la régler à nouveau correctement — ou de l'annuler ensuite (25-3-c).

## Ce qui distingue le fournisseur du client

Le règlement fournisseur n'a **pas** de ligne `invoice_settlements` : il est porté par la **ligne
de facture** (`status`, `settlement_type`, `settlement_bank_account_id`, `settlement_account_id`,
`settlement_journal_entry_id`, `paid_at`), un seul règlement, pour le TTC
(`supplier_invoices.rs::pay_in_tx`, `:542`). Il n'est jamais rapproché d'une transaction bancaire :
aucun des cinq sites de `reconciliation.rs` qui posent `matched_entry_id` ne vise une écriture
fournisseur (vérifié en passes 1 et 3 de la mère).

## Arbitrages de Guy qui s'appliquent ici (2026-09-24)

- **Annuler le règlement ramène la facture à `open`.** L'annulation d'une facture **payée** — son
  règlement redevenant **à lettrer** — est la **25-3-c**, pas celle-ci.
- **Lot pain.001 confirmé** : l'annulation du règlement est autorisée, le lot est **laissé tel
  quel** (historique de l'ordre transmis ; cas d'usage : le rejet bancaire).
- **Règlement d'un exercice CLOS : refusé ; rouvrir l'exercice reste le chemin** (Q5).

⚠️ **Ne pas contester ces arbitrages en revue** : en contester la mise en œuvre.

## Acceptance Criteria

1. **L'autorité fournisseur, et elle ne couvre QUE l'écriture de règlement.** Nouvelle variante de
   l'enum d'autorité posé par la 25-3-a-1 : `(OwnedBySupplierInvoice, supplier_invoices.id)`.
   ⛔ `reversal_blocker` attribue `OwnedBySupplierInvoice` à l'écriture d'achat **comme** à celle de
   règlement (`journal_entries.rs:1271-1276`, `purchase_journal_entry_id = je.id OR
   settlement_journal_entry_id = je.id`) : **l'autorité ne doit JAMAIS permettre de contre-passer
   l'écriture d'achat**. La variante vérifie que l'écriture est `settlement_journal_entry_id` de cette
   facture, et refuse sinon. Test dédié, **prouvé par mutation**.

2. **Les motifs** — le type et la fonction de la 25-3-a-1, **étendus** ou **jumeaux** (au choix du
   développeur, à justifier au Dev Agent Record ; ⛔ jamais une seconde précédence divergente) :

   | rang | motif | code | à l'écriture |
   |---|---|---|---|
   | 1 | facture non `paid` | `ILLEGAL_STATE_TRANSITION` n'est **pas** acceptable — code dédié, p. ex. `SUPPLIER_INVOICE_NOT_PAID` | 409 par le geste |
   | 2 | exercice de l'écriture de règlement **clos** | `FISCAL_YEAR_CLOSED` | 409 par le geste |
   | 3 | compte archivé | `ACCOUNT_ARCHIVED` | laissé au socle (400 qui **nomme**) |
   | 4 | aucun exercice ouvert ne couvre le jour | `FISCAL_YEAR_INVALID` | laissé au socle (400) |

   ⛔ Rangs 3 et 4 dans l'ordre **réel** du socle (archivés à l'étape 3 de `reverse_in_tx`, exercice
   du jour à l'étape 4) — même correction que la 25-3-a-1, passe 4.

   ⚠️ Le rang 1 **précède** le rang 2 par nécessité : une facture non `paid` n'a pas d'écriture de
   règlement, l'exercice ne s'y évalue même pas. Pas de rang « rapprochement » : il n'est pas
   atteignable (§ ci-dessus) — mais le socle le **garde** quand même, et c'est lui qui refuserait.

3. **Le geste.** Wrapper + cœur `_in_tx`, sur le modèle de `pay` / `pay_in_tx` :
   1. verrou facture `FOR UPDATE` ;
   2. motifs rangs 1-2 → refus ;
   3. contre-passation de `settlement_journal_entry_id` au titre de l'autorité (AC 1) ;
   4. `UPDATE` → `status = 'open'`, `settlement_type`, `settlement_bank_account_id`,
      `settlement_account_id`, `settlement_journal_entry_id` et `paid_at` **à `NULL`**, `version + 1`,
      garde `AND version = ? AND status = 'paid'` (défense en profondeur ; la version est lue sous
      verrou, 0 ligne n'arrive donc pas en usage normal) ;
   5. audit (AC 4), qui **conserve** l'identifiant de l'écriture de règlement annulée et celui de sa
      contre-passation — seule trace du lien, la colonne étant remise à `NULL`.

   ⚠️ **Pourquoi vider les colonnes** : il n'y en a qu'un jeu, et un nouveau règlement l'écraserait.
   `chk_supplier_invoices_paid_has_settlement` n'impose rien à une facture `open`. Le lien survit
   dans l'audit **et** au grand livre (l'écriture d'origine porte `reversed_by`).

   **Lot confirmé** : aucune écriture sur `payment_batches` ni `payment_batch_items`. Une facture
   revenue à `open` peut repartir dans un lot (`payment_batches.rs:273-282` n'arrête que les lots
   `generated`).

4. **L'audit** : `supplier_invoice.settlement_cancelled`, littéral, inscrit à `ACTIONS`, libellé ×4.
   Le socle écrit en plus `journal_entry.reversed` — voulu.

5. **La route** : `POST /api/v1/supplier-invoices/{id}/settlement/cancel` — **Comptable+**, dans
   `comptable_routes`, à côté de `pay` et `cancel`. Registre de routes : `Traced`, totaux
   **recomptés** (ils auront bougé avec la 25-3-a-1 — partir de la source, pas de la fiche).

6. **Les champs de lecture, et où ils vivent.** `SupplierInvoiceResponse` gagne
   `settlementCancellable`, `settlementCancelBlockedBy`, `settlementCancelBlockedLabel` (numéro du
   compte au rang 3). ⚠️ `SupplierInvoiceResponse::from_parts` (`routes/supplier_invoices.rs:90`) est
   **synchrone** et a **cinq** appelants (GET, création, `pay`, `cancel`,
   `imported_supplier_invoices.rs:282`) : suivre le patron
   `InvoiceResponse::from_parts(..).with_settlement(..)` (`routes/invoices.rs:640`) — un constructeur
   additionnel, appelé par le **GET**, par la réponse de **`pay`** et par celle de l'**annulation** ;
   ailleurs, champs **absents** (`Option` + `skip_serializing_if`), jamais une valeur par défaut
   qui mentirait. `SupplierInvoiceListItemResponse` n'est pas touché.

7. **L'écran** (`supplier-invoices/[id]/+page.svelte`) : bouton « Annuler le règlement » si
   **`settlementCancellable === true`** — ⛔ **pas** sur `status === 'paid'` : le bouton afficherait
   un geste qui échoue au clic (exercice clos, compte archivé). `pay()` remplace l'état par la
   réponse (`:87`) : c'est pourquoi l'AC 6 fait porter les champs à la réponse de `pay`. Motif à la
   place du bouton sinon, par le module de mapping de la 25-3-a-1 (ou son jumeau fournisseur, même
   discipline de préfixe pour `lint-i18n-ownership` : `supplier-invoices-…` dans
   `features/supplier-invoices/`). Lien vers l'écriture de règlement, absent aujourd'hui
   (`:280-284`).

8. **Le motif de contre-passation directe `OWNED_BY_SUPPLIER_INVOICE`** : son texte (« annulez la
   facture ») est **faux pour l'écriture de règlement** — et il couvre **aussi** l'écriture d'achat,
   pour laquelle il est juste. Nouveau texte **qui couvre les deux rôles** sans changer
   `reversal_blocker` : « cette écriture appartient à une facture fournisseur : annulez la facture ou
   son règlement depuis sa fiche ». Sites, ensemble : FTL `journal-entries-reverse-blocked-supplier-invoice`
   ×4 (fr-CH `:342`, autres `:348`), repli serveur `errors.rs:2466-2478`, repli Svelte
   `journal-entries/[id]/+page.svelte:151-165`, doc-comment `kesh-db/src/errors.rs:67-68`, manuel
   (`:502-507`, `:1769`).

9. **Tests** (mutations vues rouges sur assertion) : autorité refusée sur l'écriture d'achat (AC 1) ;
   chaque motif et la paire rangs 1-2 par les vrais chemins ; `paid → open`, colonnes vidées,
   **re-règlement** ensuite (le cycle complet) ; facture réglée **par lot confirmé** (AC 3) ;
   **composition et rollback** (appel `_in_tx`, lecture positive dans la transaction, abandon,
   rien n'a changé) ; étanchéité multi-tenant table par table (`journal_entries`,
   `journal_entry_lines`, `supplier_invoices`, `audit_log`) ; API (200, 403 Consultation, 404, 409 avec
   `code`) ; vitest (bouton selon `settlementCancellable`, motif, état après `pay`) ; Playwright
   (payer puis annuler le règlement). Gardes i18n **recomptées**.

10. **Documentation** : manuel, « Régler une facture fournisseur » (`:1098-1108`) — le geste, ses
    refus, le lot laissé tel quel ; ⛔ **`:1771-1774`** (écriture d'exercice clos « reste
    contre-passable ») : la 25-3-a-1 l'a corrigé pour le règlement **client** — **l'étendre au
    règlement fournisseur** (Q5 vaut des deux côtés) ; `api-external.md` ; `CHANGELOG` `[0.12.1] — Non publié` ; PDF
    contrôlé **aplati**. **Gate complet** (`kesh-db`).

## Tasks / Subtasks

- [ ] **T1 — Autorité fournisseur** (AC 1) et motifs (AC 2).
- [ ] **T2 — Le geste** (AC 3), l'audit (AC 4), la route (AC 5).
- [ ] **T3 — Champs de lecture** (AC 6) et écran (AC 7).
- [ ] **T4 — Motif `OWNED_BY_SUPPLIER_INVOICE`** (AC 8).
- [ ] **T5 — Tests** (AC 9), documentation et gates (AC 10), PR `closes #414`.

## Dev Notes

- **Anti-modèle** : `supplier_invoices::cancel` réécrit sa contre-passation à la main — ne pas
  l'imiter ; sa correction est la 25-3-c (#454).
- Le projet analytique est posé au niveau document puis **recopié par ligne** à l'insertion
  (`journal_entries.rs:426`) : la contre-passation du socle, qui reprend `project_id` par ligne, le
  **préserve**.
- ⚠️ Cette fiche est **plus courte** que sa sœur parce qu'elle **réutilise** : si une revue y trouve
  un trou, vérifier d'abord que la 25-3-a-1 ne le couvre pas.

### References

- [Source: crates/kesh-db/src/repositories/supplier_invoices.rs:470-898] — garde de lot, `pay`, `pay_in_tx`, `cancel`.
- [Source: crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:65-73] — contraintes de statut.
- [Source: crates/kesh-api/src/routes/supplier_invoices.rs:62-90,295-331] — réponse, `pay`, `cancel`.
- [Source: frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte:75-109,215-284] — l'écran.
- [Source: _bmad-output/implementation-artifacts/25-3-a-1-annuler-reglement-client.md] — gabarit.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-24 | split | Née du découpage de la 25-3-a. Intègre les corrections de la passe 3 côté fournisseur : bouton conditionné à `settlementCancellable` et non au statut ; champs portés aussi par la réponse de `pay` (qui remplace l'état de l'écran) ; **cinq** appelants de `from_parts`, non quatre ; texte d'`OWNED_BY_SUPPLIER_INVOICE` qui couvre l'écriture d'achat **et** celle de règlement ; motif « non `paid` » **codé** ; test de rollback par composition. Ferme #414, **après** la 25-3-a-1. |
