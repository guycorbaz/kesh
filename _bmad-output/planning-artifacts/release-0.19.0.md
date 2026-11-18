# Release 0.19.0 — ce qui sera fait, et ce que cela touche

**Publication : vendredi 2026-11-20.** Coupure le jeudi 19 au soir ; ce qui n'est pas clos à la coupure sort de la release. Décisions de Guy du 2026-11-17 (page de planification, archivée au dossier de projet). Ce document s'adresse aussi aux développeurs qui travaillent en parallèle — en particulier sur le **multi-devise** — pour qu'ils voient d'avance où leurs changements croiseront les nôtres.

> État au 2026-11-17, à la planification. Les fiches de l'Epic 43 ne sont pas encore écrites : ce qui la concerne ci-dessous est une **intention**, à confirmer à sa fiche. Ce document sera mis à jour mercredi 19 au matin, quand la migration sera arrêtée.

## Coupure du 2026-11-18 — ce qui entre, ce qui sort

**Décision de Guy du 2026-11-18 : la coupure est avancée au mercredi 2026-11-18**, au lancement de l'intégration. Le contenu est figé ; rien d'autre n'entre. Les sections suivantes gardent l'intention du 2026-11-17 ; celle-ci fait foi.

**Le multi-devises (Epic 37, contributeur) est dans la 0.19.0** : il a été fusionné dans `main` le 2026-11-17 (PR #1247), avant les épiques de la release, avec ses trois migrations. La 0.19.0 porte donc **quatre migrations** depuis la 0.18.0 (trois de l'Epic 37, `20261114000001` à `…03`, et `20261118000001_grouped_settlements` de l'Epic 43), **95** au total ; aucune ne relève `kesh_version_min_required`, qui reste à 0.17.6. La sauvegarde complète d'avant la mise à jour est obligatoire ; le retour à la 0.18.0 passe par elle.

| Entre dans la 0.19.0 | Issues |
|---|---|
| Epic 37 — comptabilité multi-devises (37-1 à 37-5) | refs #233 |
| Epic 43 — 43-1a (règlement groupé, modèle et clients), 43-1b (fournisseurs), 43-2 (API), 43-3 (écran), 43-7 (plafond de l'écart) | #823 |
| Epic 42 — 42-1 (= 36-16a, changer le compte analytique d'une facture client) | refs #1167 |
| Epic 44 — 44-1a (QR-facture : grille SIX, *Payable by*, largeurs) | refs #944 |
| Correction de la semaine — v0190-correction (client affiché ≠ client enregistré), v0190-nom-societe (nom de la société modifiable) | #1162, #1097 |

| Sort de la 0.19.0 (statut `backlog`) | Motif |
|---|---|
| 43-4 (solde), 43-5 (paiements non rapprochés), 43-6 (raccord avec la réimputation) | coupure du 2026-11-18 (43-5 sortie dès le 2026-11-17 par Guy) |
| 42-2 (facture fournisseur), 42-3 (écriture manuelle), 42-4 (solde) | coupure du 2026-11-18 (42-2 et 42-3 sorties dès le 2026-11-17) |
| 44-1b (cascade des paliers), 44-2 (solde) | coupure du 2026-11-18 |

Les épiques 42, 43 et 44 restent ouvertes (`in-progress`) ; #1167 et #944 restent ouvertes.

## Base de départ

- `main` au tag **v0.18.0** (`8b50fd195`). Production (NAS) en 0.18.0. **91 migrations** ; `kesh_version_min_required` = 0.17.6.
- Branches : une par épique, `epic/42-…`, `epic/43-…`, `epic/44-…`, parties de `main` ; les stories en `story/…`. Rien n'est fusionné dans `main` avant la PR de release du vendredi.

## Contenu, dans l'ordre de priorité

| Rang | Épique / story | Issues | Migration | Si le temps manque |
|---|---|---|---|---|
| 1 | **Epic 43 — Une transaction, plusieurs factures** | #823 (close) | **oui** | reste : priorité de Guy |
| 2 | Epic 42-1 — Changer le compte analytique d'une facture client (ex-36-16a, **terminée**) | #1167 (refs) | non | reste |
| 3 | Correction de la semaine — client affiché ≠ client enregistré ; nom de la société modifiable | #1162, #1097 | non | passe à la 0.20.0 |
| 4 | Epic 44-1 — QR-facture : grille, « Payable by », largeurs (ex-36-12b) | #944 (close) | non | sort |
| 5 | Epic 42-2 — idem 42-1, facture fournisseur (ex-36-16b) | #1167 | non | sort |
| 6 | Epic 42-3 — idem 42-1, écriture manuelle (ex-36-16c) ; #1167 se ferme avec elle | #1167 (close) | non | sort la première |

### Epic 43 — Une transaction, plusieurs factures (#823) — porte la migration

**Besoin** : un mouvement du relevé bancaire règle **plusieurs factures** :
- **fournisseurs** : un débit qui paie plusieurs factures fournisseur validées du même fournisseur (typiquement un lot pain.001 débité en une ligne) ;
- **clients** : un crédit qui paie plusieurs factures clients.

À la réconciliation, l'utilisateur choisit plusieurs factures ouvertes dont la somme des restes dus égale le montant de la transaction ; Kesh passe le règlement et lettre chaque facture. Le virement entre comptes propres est déjà traité (0.17.6).

**Ce qui change dans le modèle (intention, à confirmer à la fiche)** :
- aujourd'hui, une transaction bancaire pointe **une** écriture (`bank_transactions.matched_entry_id`) ;
- un règlement client (`invoice_settlements`) exige **sa propre** écriture : `UNIQUE KEY uq_invoice_settlements_entry (journal_entry_id)` — c'est ce qui rend le cas client impossible sans migration ;
- une facture fournisseur pointe son règlement par `supplier_invoices.settlement_journal_entry_id`, sans contrainte d'unicité ;
- la migration lèvera ou remplacera la contrainte d'unicité des règlements clients, ou introduira une table de liaison transaction → règlements. **Non breaking visé** (pas de `DROP`, pas de changement de type) ; elle sera nommée par horodatage du 2026-11-18 ou 2026-11-19.

**Fichiers probablement touchés** :
- `crates/kesh-db/migrations/` (une nouvelle migration), `docs/migrations-idempotence-audit.md`, `crates/kesh-db/test-schema/`, `crates/kesh-db/src/post_restore.rs` (triage P7 si la migration écrit des données) ;
- `crates/kesh-db/src/repositories/` : `reconciliation.rs`, `reconciliation_link.rs`, `reconciliation_cancel.rs`, `invoice_settlements.rs`, `invoice_settlements_write.rs`, `settlement_cancellation.rs`, `supplier_invoices.rs`, `bank_transactions.rs`, `letterings.rs`, peut-être `payment_batches.rs` ;
- `crates/kesh-reconciliation/src/` (`matching.rs`, `manual.rs`) ;
- `crates/kesh-api/src/routes/reconciliation.rs`, `supplier_invoices.rs` ; audit et erreurs (`audit_labels.rs`, `errors.rs`) ;
- `frontend/src/lib/features/reconciliation/`, `frontend/src/lib/features/open-items/` ;
- `.keshbackup` et export de souveraineté si une table est ajoutée ;
- manuels `docs/manual/fr/user-manual.tex` et `admin-manual.tex` ; `docs/api-external.md` si l'API de réconciliation change.

### Epic 42 — Réimputation analytique (#1167)

Un geste dédié, « Changer le compte analytique », sur une facture client (fait), une facture fournisseur et une écriture manuelle, quel que soit leur statut ; tracé à l'audit ; accusé en période close. Sans migration.

Fichiers : `crates/kesh-db/src/repositories/project_reassignment.rs` (nouveau), `crates/kesh-api/src/routes/invoices.rs`, routes des factures fournisseur et des écritures, `audit_labels.rs`, `errors.rs` (kesh-api et kesh-db), `frontend/src/lib/features/projects/` (`ProjectReassignDialog.svelte`, `projects.api.ts`), pages `invoices/[id]`, `supplier-invoices/[id]`, `journal-entries/[id]`, catalogues i18n, `docs/api-external.md`, manuels.

### Epic 44 — QR-facture conforme (#944)

Mise en page de la partie paiement selon SIX § 3.7 : grille, place de « Payable by », largeurs mesurées. Sans migration, sans changement de données.

Fichiers : `crates/kesh-qrbill/src/pdf.rs`, `crates/kesh-qrbill/src/largeurs.rs`, `crates/kesh-api/src/routes/invoice_pdf_service.rs`, `issued_invoice_pdf.rs`, manuel utilisateur.

### Correction de la semaine (#1162, #1097)

- **#1162** : `frontend/src/lib/components/invoices/ContactPicker.svelte` — au départ du champ, un client tapé mais non choisi ne doit plus rester affiché pendant que l'ancien part au serveur.
- **#1097** : le nom de la société devient modifiable — `crates/kesh-api/src/routes/companies.rs` (nouvelle route ou extension), `frontend/src/routes/(app)/settings/+page.svelte`, i18n, manuel.

## Points de contact avec le multi-devise

Là où les deux chantiers se croiseront, et ce que nous recommandons :

1. **Règlements et réconciliation** — c'est le cœur de l'Epic 43 : sommes de restes dus, écritures de règlement, lettrage, liens transaction → écritures. Toute règle de devise sur ces chemins (montant en devise, conversion, écart de change) devra s'appliquer à des règlements **multiples** après la 0.19.0. **Recommandation : ne pas modifier ces fichiers avant la publication de la 0.19.0**, ou nous prévenir avant, pour décider de l'ordre.
2. **Migrations** — la nôtre sera la 92ᵉ. Une migration du multi-devise doit porter un horodatage **postérieur** et partir de `main` après la 0.19.0 ; son ordre se contrôlera à l'intégration, sur une base peuplée par la 0.19.0.
3. **Saisie des montants** — depuis les 0.17.8 et 0.18.0, un montant ambigu comme « 1.250 » est refusé (et l'apostrophe des milliers admise) dans les écrans qui écrivent dans les livres ; les réglages et filtres suivent en 0.20.0 (#1173, #1175 à #1177). Un champ de montant en devise devrait réemployer la même règle.
4. **QR-facture** (Epic 44) : la partie paiement change de mise en page ; elle ne touche ni la devise ni le montant encodés.
5. **0.20.0 (vendredi 2026-11-27)** — annoncée : une **migration** pour la **note interne** (colonne facultative sur les écritures et sur les factures, #1158, #268), peut-être regroupée avec d'autres ; les montants « 1.250 » des réglages et filtres.

## Règles de travail utiles (extraits du `CLAUDE.md`)

- Un epic = une release ; partir de `main`, jamais d'une branche d'epic non publiée ; reprendre `main` après chaque release publiée par un autre.
- Fichiers communs : `CHANGELOG.md` assemblé à l'intégration ; PDF des manuels jamais commités dans une branche de story ou d'epic ; i18n et `sprint-status.yaml` par blocs, conflits résolus en gardant les deux côtés ; compteurs (`sitesTotal`, totaux de l'audit des migrations, schéma de test) recomptés après fusion.
- Le second arrivé résout dans sa branche, jamais dans `main` ; une contradiction se règle avec l'autre développeur, puis Guy arbitre.
