# Story 24.3 : Le règlement hors banque — espèces, compensation, poste

## Status

draft

## Story

**As a** personne qui encaisse aussi en espèces,
**I want** enregistrer un règlement quel qu'en soit le mode, et qu'il produise son écriture,
**so that** ma caisse soit tenue et que « payé » veuille dire la même chose partout.

Ferme l'issue **#372**.

## Le défaut

Une facture client réglée hors virement — espèces, compensation, poste — n'a **aucun chemin
qui produise son écriture**. Seul `mark_as_paid` existe, et il ne comptabilise rien : depuis la
24-2, c'est **le dernier endroit de Kesh où « payé » ne veut rien dire comptablement**.

⚠️ **Le mode de règlement est indifférent au traitement comptable** — seule change la
contrepartie : 1020 banque, 1000 caisse, 1010 poste. La distinction « virement → écriture /
espèces → simple marquage » n'est **pas comptablement fondée**. Ce que l'import CAMT automatise,
c'est la **détection** du paiement, pas son **enregistrement**.

⚠️ **Et s'il fallait plus de rigueur d'un côté, ce serait le cash.** Les espèces n'ont aucune
trace externe, ce qui en fait la zone la plus scrutée par l'AFC. Une caisse ne peut **jamais**
être créditrice — un solde négatif au 1000 est la **preuve arithmétique** d'un défaut de tenue.
Marquer « payé en espèces » sans écriture est exactement ce qui empêche de tenir un livre de
caisse.

## Le gabarit, encore une fois, est le fournisseur

`supplier_invoices` porte déjà le contrat, **vérifié au sol**
(`20260628000001_supplier_invoices.sql:67`) :

```sql
CHECK (settlement_type IS NULL OR settlement_type IN ('bank_transfer', 'internal_account'))
```

et `SettlementChoice` (`entities/supplier_invoice.rs:104`) en est la forme Rust :
`BankTransfer { bank_account_id }` — contrepartie = `bank_account.journal_account_id` — ou
`InternalAccount { account_id }` — contrepartie = **n'importe quel compte du plan**, caisse
comprise.

⛔ **Reprendre ce contrat mot pour mot.** Deux vocabulaires pour la même notion coûteraient à
chaque lecture, et la 24-2 a déjà montré ce que vaut le miroir fournisseur.

## D1 — Le mode vit sur le RÈGLEMENT, pas sur la facture

Chez le fournisseur, `settlement_type` est une colonne de `supplier_invoices` : le règlement y
est unique par construction. **Côté client il ne l'est pas** (24-2) — une facture peut être
réglée moitié en espèces, moitié par virement.

Les trois colonnes vont donc sur **`invoice_settlements`**, une par règlement :

```sql
ALTER TABLE invoice_settlements
    ADD COLUMN settlement_type VARCHAR(20) NOT NULL DEFAULT 'bank_transfer',
    ADD COLUMN settlement_bank_account_id BIGINT NULL,
    ADD COLUMN settlement_account_id BIGINT NULL,
    ADD CONSTRAINT chk_invoice_settlements_type
        CHECK (settlement_type IN ('bank_transfer', 'internal_account')),
    ADD CONSTRAINT chk_invoice_settlements_counterparty
        CHECK ((settlement_type = 'bank_transfer'
                AND settlement_bank_account_id IS NOT NULL AND settlement_account_id IS NULL)
            OR (settlement_type = 'internal_account'
                AND settlement_account_id IS NOT NULL AND settlement_bank_account_id IS NULL));
```

⚠️ **`DEFAULT 'bank_transfer'` et la migration reste NON-BREAKING.** Les lignes déjà écrites par
la 24-2 viennent toutes de la réconciliation bancaire : le défaut dit d'elles la vérité, il ne
la fabrique pas. ⛔ Mais la contrainte de contrepartie exige alors
`settlement_bank_account_id IS NOT NULL` — **il faut donc le renseigner sur l'existant dans la
même migration**, ce qui en fait un **backfill** : triage P7 obligatoire, et la question de la
fenêtre d'importabilité se repose (cf. Dev Notes).

## D2 — `mark_as_paid` disparaît, et il faut le dire

L'issue est nette : *« retirer `mark_as_paid` sans écriture — l'expert-comptable ne lui voit
aucun usage légitime survivant. »* Il est remplacé par **« Enregistrer un règlement »** :
mode, compte, date, montant.

⛔ **Et `unmark-paid` disparaît avec lui.** Un marquage qui n'écrivait rien s'annulait
gratuitement ; un règlement qui produit une écriture ne s'annule pas, il se **contre-passe**.

⚠️ **Conséquence à assumer, pas à masquer : il n'y aura plus AUCUN moyen d'annuler un règlement
depuis l'interface.** C'est exactement la situation du côté fournisseur — vérifié au sol :
`supplier_invoices::cancel` (`:724`) contre-passe bien, mais **refuse toute facture qui n'est
pas `open`**, donc une facture réglée y est déjà figée. Cette story **aligne** les deux côtés au
lieu d'aggraver un écart, mais elle rend le manque visible des deux.

⛔ **Issue #414 ouverte** — « annuler un règlement par contre-passation », couvrant les **deux**
côtés. Ce n'est pas un détail reporté en silence : c'est le pendant nécessaire de ce qu'on
retire, et il est tracé à ce titre.

## D3 — Le montant, et ce qu'on ne demande pas

Le formulaire demande **un montant**, pré-rempli au résiduel. C'est ce qui rend le règlement
partiel saisissable à la main — la 24-2 ne l'ouvrait qu'à la réconciliation bancaire.

⛔ Le trop-perçu reste **refusé**, même garde qu'en 24-2 : un règlement supérieur au résiduel
rendrait le compte de créance créditeur.

## Critères d'acceptation

**AC1** — `POST /api/v1/invoices/{id}/settlements` enregistre un règlement : `settlementType`,
la contrepartie correspondante, `settledOn`, `amount`. Il crée l'écriture
`D <contrepartie> / C <compte de créance>` et sa ligne `invoice_settlements`.

**AC2** — Le compte de créance est lu **sur l'écriture de vente**, jamais sur les réglages —
même invariante qu'en 24-2, et c'est elle qui fait que le compte se solde.

**AC3** — `internal_account` accepte **n'importe quel compte actif du plan**, caisse comprise.
⛔ Le compte doit être **actif** ; un compte archivé est refusé.

**AC4** — `paid_at` reste la **projection** du résiduel à zéro. Un règlement partiel en espèces
ne solde pas plus qu'un virement partiel.

**AC5** — Le trop-perçu est refusé (`400`), avec résiduel et montant dans le détail.

**AC6** — Aucun exercice **ouvert** ne couvre `settledOn` ⇒ refus. Jamais d'écriture dans un
exercice clos.

**AC7** — ⛔ **`POST /invoices/{id}/mark-paid` et `/unmark-paid` sont SUPPRIMÉS**, routes,
handlers, `invoices::mark_as_paid`, DTO, et leurs surfaces frontend.

**AC8** — L'écran remplace « Marquer payée » par « Enregistrer un règlement » : mode, compte,
date, montant pré-rempli au résiduel. Sur la fiche **et** sur l'échéancier.

**AC9** — Migration : `ADD COLUMN` + `CHECK` + **backfill** de `settlement_bank_account_id` sur
l'existant. Non-breaking ⇒ pas de bump `min_required` ; **triage P7 obligatoire** (elle écrit des
données) ; ligne d'audit d'idempotence, cinq compteurs recomptés.

**AC10** — i18n : quatre locales, y compris les libellés de mode de règlement.

## Invariants testables

1. **La caisse se meut.** Facture 100 réglée en espèces sur le 1000 ⇒ le 1000 est débité de 100,
   la créance créditée de 100, solde de créance nul.
2. **Deux modes sur une même facture.** 60 en espèces puis 40 par virement ⇒ soldée, créance à
   zéro, **deux** lignes `invoice_settlements` de types différents.
3. **Le compte archivé est refusé**, et rien n'est écrit.
4. **Le trop-perçu est refusé**, et rien n'est écrit.
5. **Les routes supprimées rendent 404/405** — l'assertion qui empêche un retrait à moitié fait.
6. **Concordance grand livre** (24-1) : le compte de caisse porte une ligne par règlement
   espèces.
7. ⛔ **Le backfill est vérifié sur des lignes PRÉEXISTANTES** — un test qui n'écrirait ses
   lignes qu'après la migration ne mesurerait rien.

## Hors périmètre

L'**annulation d'un règlement** par contre-passation — **issue #414**, les deux côtés. Le
**livre de caisse** comme rapport dédié. La **garde « caisse jamais créditrice »** : elle mérite
sa propre story, et elle porte sur la saisie d'écriture en général, pas sur ce chemin. Le
**lettrage** (epic 15). La propagation du résiduel aux rapports agrégés.

## Dev Notes

⛔ **Gate ciblé INTERDIT** (P6/P7) : migration + repository. La 24-2 a déclenché **sept**
garde-fous sur des fichiers hors périmètre ; celle-ci écrit des données, donc au moins le triage
P7 et l'audit d'idempotence s'y ajoutent.

⚠️ **La fenêtre d'importabilité se reposera.** La 24-2 a vidé `POST_RESTORE_BACKFILLS` en créant
une table. Cette migration **écrit des données** : il faudra décider si son backfill entre au
registre (elle est postérieure à la dernière table applicative, donc **dans** la fenêtre — c'est
le premier cas depuis longtemps) ou s'il est exempté. **Ne pas trancher par réflexe** : le test
`registry_entries_are_within_import_window` recalcule la fenêtre et tranchera.

⚠️ **La base de gate se remet à zéro AVANT chaque gate**, inconditionnellement (KF-039).

⚠️ **Surface de régression à relever, pas à supposer** : `mark-paid` / `unmark-paid` sont
consommés par `invoices/[id]/+page.svelte`, `invoices/due-dates/+page.svelte`,
`MarkPaidDialog.svelte`, et par les specs E2E `invoices.spec.ts` et
`invoices_echeancier.spec.ts` — dont le **golden path de l'échéancier**, qui fait
« marquer payée → dé-marquer ». Ce scénario **disparaît avec les routes** : le remplacer par le
parcours de règlement, et non le supprimer.
