# Story 25.3 : Annuler un règlement et un rapprochement — par contre-passation

Status: split

⛔ **DÉCOUPÉE le 2026-09-24** (arbitrage de Guy), en **25-3-zero** (`reverse_in_tx`), **25-3-a**
([#414]) et **25-3-b** ([#418]). **Cette fiche reste la SOURCE DES FAITS** — ses corrections
ci-dessous valent pour les trois filles, et rien n'y est réécrit.

⛔ **Pourquoi : le « geste commun » de son AC 1 n'était commun qu'aux deux tiers.** La table
`invoice_settlements` **ne concerne que les factures CLIENT** — aucune référence vers
`supplier_invoices`, vérifié dans la migration. Côté fournisseur, le règlement est porté **par la
ligne de facture elle-même** (`settlement_journal_entry_id`, `paid_at`, `status`). « Retirer la
ligne de règlement » n'y a donc aucun sens : il faut repasser le statut et trancher le sort des
colonnes `settlement_*`. *C'est le critère de split que cette fiche annonçait elle-même.*

## ⛔ Corrections de la passe de validation — ce que cette fiche affirmait et qui était FAUX

*Cinq décomptes ou citations, tous établis depuis la source. Ils valent pour les trois filles.*

| affirmé ici | réel |
|---|---|
| `ReversalBlocker` a **sept** variantes | **huit** (`AccountArchived` manquait) — le code le dit lui-même à `errors.rs:526` |
| **dix** sites de production lisent `invoice_settlements` (7+2+1) | ~**huit** — et ⛔ **deux des « sites de production » d'`invoices.rs` étaient du code de TEST** (après `#[cfg(test)]`, l. 2503) |
| le refus « seule une facture ouverte » est à `supplier_invoices.rs:724` | **:776** — la 724 est dans `pay_in_tx`, sans rapport |
| **six** FK vers `journal_entries(id)`, toutes en `RESTRICT` sauf une | **huit**, dont **une en `CASCADE`** (`journal_entry_lines.entry_id`) et une en `SET NULL` (`matched_entry_id`) |
| `supplier_invoices::cancel` est l'exemple à réutiliser (AC 2) | ⛔ **anti-exemple** : elle ne pose **jamais** `reverses_entry_id` et ne consulte **jamais** `reversal_blocker` |

⚠️ **Le point de fond de chaque affirmation restait juste** — le résiduel diverge en silence, il ne
faut pas compter sur `SET NULL` —, mais *un argument vrai adossé à un chiffre faux se retourne
contre celui qui le relit.*

**Issues : [#414] et [#418]**, qu'elle **ferme toutes deux** : `closes #414, closes #418` dans le
**titre ET le corps** de sa PR.

⛔ **Une seule story, arbitrage de Guy du 2026-09-24.** Les deux issues partagent **le même geste**
— contre-passer l'écriture, défaire le lien, laisser retomber l'état dérivé —, et #418 se dit
elle-même *« le voisin immédiat »* de #414. Les séparer ferait écrire **deux fois la même
transaction**, avec le risque qu'elles divergent.

## Pourquoi maintenant, et pourquoi ce n'est pas une commodité

⛔ **L'Epic 24 a livré le moyen de créer, pas celui de défaire.** Il a ouvert l'encaissement (#371)
et le rapprochement, puis **gelé** l'écriture (24-4b) : une écriture enregistrée n'est plus ni
modifiable ni supprimable. Le gel est juste — il applique l'art. 958f CO. Mais il ferme la porte de
sortie que la réécriture à la main tenait entrouverte.

⚠️ **Le code le dit lui-même, et il nomme ces deux issues.** `ReversalBlocker`
(`crates/kesh-db/src/errors.rs`) refuse la contre-passation dans **sept** cas, et deux renvoient
explicitement ici :

| variante | ce que le code écrit |
|---|---|
| `OwnedBySettlement` | *« le résiduel se calcule depuis `invoice_settlements.amount`, que la contre-passation ne toucherait pas — grand livre et résiduel divergeraient **en silence**. Chemin : #414 »* |
| `MatchedBankTransaction` | *« Aucune route de dé-rapprochement n'existe (#418) : le refus laisse un manque, **assumé** »* |

**Conséquence, et c'est l'état de `main` aujourd'hui** : une écriture de règlement ou de
rapprochement n'est **ni contre-passable ni défaisable**. Elle est **définitivement incorrigible**.
*Une dette écrite dans le code par ceux qui l'ont contractée — il reste à la payer.*

## Les arbitrages rendus

**1. La ligne `invoice_settlements` est RETIRÉE, non marquée annulée** (Guy, 2026-09-24).

⛔ **Le fait qui tranche** : le résiduel se calcule par `INVOICE_SETTLED_SUBQUERY_SQL`
(`invoice_settlements.rs:34`), et **dix sites de production** lisent cette table pour établir un
total — sept dans `invoice_settlements.rs`, deux dans `invoices.rs`, un dans `journal_entries.rs`.
Introduire un état « annulé » obligerait à reprendre **chacun** ; un seul oubli ferait diverger le
résiduel **en silence**, c'est-à-dire exactement le mode d'échec que cette story répare.

⚠️ **Et cela compliquerait une story non commencée** : **[#416]**, qui porte la propagation du
résiduel aux rapports agrégés, est **encore ouverte**.

*La trace n'est pas perdue pour autant, et c'est ce qui rend le retrait acceptable* : l'**écriture
de contre-passation reste au grand livre** — avec l'originale, comme l'art. 958f l'exige —, et
l'**audit nomme le geste**. Ce qui disparaît est une ligne de calcul, pas une preuve.

**2. Une seule story, avec un ordre interne strict** : le geste commun d'abord, ses trois appelants
ensuite.

## Acceptance Criteria

1. **Le geste commun, écrit UNE fois.** Une fonction de dépôt annule un règlement dans **une seule
   transaction** : contre-passer l'écriture, retirer la ligne `invoice_settlements`, laisser
   retomber l'état dérivé, journaliser. ⛔ **Les trois appelants (client, fournisseur,
   rapprochement) l'appellent ; aucun ne réécrit la transaction.**

2. **La contre-passation, jamais la suppression.** L'écriture d'origine **reste**, et une écriture
   inverse est créée — c'est déjà ce que fait `supplier_invoices::cancel` pour l'achat, et le
   mécanisme existe (`reverses_entry_id`, unicité par `uq_journal_entries_reverses`).
   ⛔ **Réutiliser le chemin de contre-passation existant**, ne pas en écrire un second.

3. **La date de la contre-passation est dans un exercice OUVERT**, jamais celle de l'origine si son
   exercice est clos — l'issue #414 l'exige, et c'est la règle de l'art. 958f. ⚠️ **À trancher au
   développement et à écrire au Dev Agent Record** : date du jour, ou date fournie par l'appelant
   avec contrôle d'exercice ouvert ?

4. **`paid_at` retombe à `NULL`** quand plus aucun règlement ne subsiste — et **seulement alors** :
   une facture partiellement réglée dont on annule **un** règlement reste partiellement réglée.
   ⚠️ *C'est le cas que l'écran a déjà mal traité ailleurs : `paid_at` ne dit rien du règlement
   partiel.*

5. **Côté fournisseur**, `supplier_invoices::cancel` refuse aujourd'hui toute facture qui n'est pas
   `open` (`supplier_invoices.rs:724`). Une facture **réglée** doit pouvoir être annulée — l'issue
   #414 le demande en propre.

6. **Le dé-rapprochement** (#418) : contre-passer l'écriture produite, remettre
   `bank_transactions.matched_entry_id` à `NULL` et recalculer le statut de la transaction.
   ⛔ **Ne JAMAIS compter sur `ON DELETE SET NULL`** : la FK est bien en `SET NULL`
   (`20260504000001_bank_imports.sql:83`), donc supprimer l'écriture ferait disparaître le lien **en
   silence** — le mode d'échec que la story répare, reproduit par son correctif.

7. **Quand le rapprochement portait sur une facture**, le dé-rapprochement retire aussi la ligne
   `invoice_settlements` et laisse `paid_at` retomber : **c'est le geste de l'AC 1**, appelé, non
   réécrit.

8. **Les deux `ReversalBlocker` concernés changent de sens.** `OwnedBySettlement` et
   `MatchedBankTransaction` refusent aujourd'hui **en orientant vers un chemin qui n'existe pas**.
   Leurs messages doivent nommer le chemin **réel**, dans les **quatre** locales, replis en dur
   compris. ⚠️ **Ne pas retirer les refus** : contre-passer une écriture de règlement *directement*
   reste faux — c'est le geste d'annulation qui la contre-passe, avec le reste.

9. **L'audit nomme chaque geste** : codes distincts pour l'annulation d'un règlement et le
   dé-rapprochement, inscrits aux **trois registres** (`audit_labels.rs` + les quatre locales,
   `audit_route_registry.rs` avec ses totaux **recomptés** et sa ventilation, `SITES_INDIRECTS` si
   le code n'est pas un littéral au site d'insertion).

10. **Les écrans.** Annuler un règlement depuis la fiche de facture (client) et la fiche
    fournisseur ; annuler un rapprochement depuis l'écran de réconciliation. ⛔ **Chaque refus
    s'affiche en nommant son motif** — et le repli en dur dit **mot pour mot** ce que dit le FTL.
    ⚠️ **Contrôler le texte affiché, pas seulement les clés** : sur les deux écrans de factures, un
    relevé a trouvé **49 libellés en dur** dont 41 avec une clé existante.

11. **Tests** : le geste commun, ses trois appelants, le règlement **partiel** (AC 4), l'exercice
    clos (AC 3), le dé-rapprochement d'une transaction **sans** facture, et celui d'une transaction
    **avec** facture. ⛔ Chaque garde **prouvée par mutation**, vue rouge **sur assertion**.
    ⚠️ **Et un test d'étanchéité multi-tenant** : ces chemins écrivent dans quatre tables.

12. **Manuels, `api-external.md`, `CHANGELOG`** (section `[0.12.1] — Non publié`, **aucune ligne
    d'une section publiée ne se réécrit**). ⛔ **Contrôler le PDF APLATI**
    (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).

13. **Gate complet** — la story touche `kesh-db`, `kesh-api` et le frontend.

## Tasks / Subtasks

- [ ] **T1 — Le geste commun** (AC 1, 2, 3, 4) — la transaction, écrite une fois.
- [ ] **T2 — Les trois appelants** (AC 5, 6, 7) : client, fournisseur, rapprochement.
- [ ] **T3 — Les refus qui changent de sens** (AC 8) et l'audit (AC 9).
- [ ] **T4 — Les écrans et les quatre locales** (AC 10).
- [ ] **T5 — Tests et mutations** (AC 11).
- [ ] **T6 — Documentation** (AC 12), gates (AC 13), PR `closes #414, closes #418`.

## Dev Notes

### Ce que cette story ne fait pas

- Elle ne touche **pas** à l'avoir : annuler une **facture** reste le chemin de l'avoir
  (`OwnedByInvoice`, `OwnedByCreditNote`), et ces deux refus ne bougent pas.
- Elle ne propage pas le résiduel aux **rapports agrégés** : c'est **[#416]**.
- Elle n'impute pas l'**écart** d'un règlement partiel (escompte, frais) : c'est **[#384]**.

### Le piège nommé par le code lui-même

⛔ `bank_transactions.matched_entry_id` est **la seule des six références à une écriture qui soit en
`ON DELETE SET NULL`**. Toutes les autres sont en `RESTRICT`. *Une suppression y passerait sans
bruit et emporterait le lien* — d'où l'AC 6, qui l'interdit explicitement.

### Règle de splitting

Modules touchés : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — **quatre**, sous le seuil de
cinq. ⚠️ **Le signal à surveiller** : si le geste commun de l'AC 1 ne se laisse pas écrire une seule
fois pour les trois appelants, c'est que le découpage est mauvais — sortir alors le geste en
story-zéro plutôt que d'accepter trois transactions jumelles.

### References

- [Source: crates/kesh-db/src/errors.rs] — `ReversalBlocker` et ses sept variantes, dont deux
  nomment ces issues.
- [Source: crates/kesh-db/src/repositories/invoice_settlements.rs:34] —
  `INVOICE_SETTLED_SUBQUERY_SQL`, et les **dix** sites qui lisent la table.
- [Source: crates/kesh-db/src/repositories/supplier_invoices.rs:724] — le refus « seule une facture
  ouverte peut être annulée ».
- [Source: crates/kesh-db/migrations/20260504000001_bank_imports.sql:83] — le `ON DELETE SET NULL`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-24 | spec | Story unique pour **[#414]** et **[#418]** (arbitrage de Guy) : même geste, et #418 se dit « le voisin immédiat » de #414. ⛔ **Fait établi avant rédaction** : le code de l'Epic 24 **nomme déjà ces deux issues** dans `ReversalBlocker` — c'est une dette écrite par ceux qui l'ont contractée, et l'écriture de règlement est aujourd'hui **définitivement incorrigible** (ni contre-passable, ni défaisable). **Arbitrage rendu sur le point que l'issue laissait ouvert** : la ligne `invoice_settlements` est **retirée**, non marquée annulée — **dix sites de production** lisent cette table pour établir un total, un seul oubli ferait diverger le résiduel **en silence**, et **[#416]**, qui porte la propagation du résiduel, n'est pas commencée. *La trace subsiste ailleurs : la contre-passation reste au grand livre, l'audit nomme le geste.* ⚠️ Deux points laissés au développement : la **date** de la contre-passation (jour, ou fournie sous contrôle d'exercice ouvert), et le fait que `paid_at` ne doit retomber que si **plus aucun** règlement ne subsiste. |
