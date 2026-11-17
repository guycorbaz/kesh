# Story 43-3 : un règlement pour plusieurs factures — l'écran de réconciliation, façon bexio (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`, code lu à
`8b50fd195` ; **révisée le soir même** (décisions de Guy : interface comme bexio, tiers mixtes, écart). Release
**0.19.0**. **Commence tout de suite** : Vitest sur le contrat de la fiche 43-2 (maquettes) ; l'**E2E** se joue sur la
branche réunie (43-1a et 43-2 fusionnées). Frontend et manuel utilisateur ; aucune migration. Décisions : C-43-14 à
C-43-16 (fichier d'épique), qui font foi. Validation : à lancer.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant que comptable, je choisis une ligne du relevé, j'y ajoute des factures une à une en les cherchant, je vois à
chaque ajout le total imputé et la différence avec le montant du relevé, et j'enregistre — puis, si je me suis trompé,
j'annule ce rapprochement en voyant les factures qu'il rouvrira.

## Ce qui se lit au code (`8b50fd195`)

- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` : actions par transaction, modales
  `ManualMatchModal`, `TransactionSplitModal`, `LinkSettlementModal` (:78-80) ; patron le plus proche :
  `LinkSettlementModal.svelte`.
- `CancelReconciliationDialog.svelte`, `reconciliation-cancel.ts`, `reconciliation.types.ts:204` (`kind`).
- `reconciliation.api.ts`, `failed-proposal-label.ts`.
- Specs voisines : `frontend/tests/e2e/reconciliation-link.spec.ts`, `reconciliation-supplier.spec.ts`,
  `reconciliation-cancel.spec.ts`.

## Décisions propres à la story

- **D1** — Bouton **« Régler plusieurs factures… »** par transaction en attente, à côté de « Rattacher… », qui ouvre
  `GroupSettlementModal.svelte` (neuf). Le sens de la transaction choisit la famille (C-43-15) ; le titre le dit.
- **D2 — Interface comme bexio (C-43-14)** : en tête, la ligne du relevé (date, contrepartie, montant) ; un champ de
  **recherche** (numéro, tiers, référence ; `GET /reconciliation/group-candidates?q=`) dont chaque résultat s'**ajoute**
  d'un clic ; la liste des factures **imputées**, chacune retirable, avec son reste dû ; en pied, **montant du relevé**,
  **total imputé**, **différence** — en vert à zéro, signalée au-delà de `maxDifference` ; aucune présélection
  automatique, le nom de la contrepartie ne fait que préremplir la recherche.
- **D3** — *Enregistrer* actif si au moins **deux** factures sont imputées et `|différence| ≤ maxDifference` ; une
  différence non nulle s'annonce (« 0.40 seront passés au compte de différences d'arrondi »). Calcul au centime, en
  entiers (aucun flottant). Le serveur reste juge : un refus `failed[]` s'affiche par son libellé, la modale reste ouverte.
- **D4** — Le côté fournisseur (débit) est dans des **commits séparés**, retirables si la 43-1b sort (C-43-18) : sans
  lui, le bouton n'apparaît que sur un crédit.

## Critères d'acceptation

- **AC1** — Le bouton existe sur une transaction en attente, non rejetée, d'un compte ouvert ; absent sinon.
- **AC2** — D2 et D3 : ajout, retrait, recherche, total, différence, bouton ; une facture déjà imputée n'est pas proposée
  deux fois ; `hasMore` → phrase qui invite à préciser la recherche ; aucun résultat → phrase qui le dit.
- **AC3** — *Enregistrer* envoie `POST /reconciliation/accept` (`invoice_group` ou `supplier_invoice_group`) ; succès →
  toast, modale fermée, transaction retirée ; refus → libellé du code (`failed-proposal-label.ts` étendu aux codes neufs
  de la 43-2), facture nommée quand `details` la donne.
- **AC4** — `CancelReconciliationDialog` : `kind = "grouped_settlement"` → les **N** factures listées (`groupedInvoices`)
  redeviendront ouvertes, l'écriture sera contre-passée.
- **AC5** — i18n : quatre locales (`fr-CH` rédigé ; `de-CH`, `it-CH`, `en-CH` traduits) ; `lint-i18n-ownership` ;
  `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` recompté depuis la source.
- **AC6** — Manuel utilisateur (`docs/manual/fr/user-manual.tex`, `.tex` seulement ; § `sec:reconciliation` et
  `sec:annuler-rapprochement`) : « Régler plusieurs factures par une transaction » — gestes, différence et son plafond,
  compte de différences d'arrondi à configurer, un règlement et un lettrage par facture, annulation en entier seulement ;
  limites : la ligne de banque d'un règlement groupé ne porte pas de compte analytique ; le motif qui gèle l'écriture ne
  nomme qu'une facture ; *(avec la 43-5)* le lot pain.001 confirmé.
- **AC7** — Captures (thème clair, base de démonstration jetable) : modale vide, une facture, deux factures avec
  différence, différence nulle, différence trop grande, refus serveur, dialogue d'annulation — dans
  `/home/gcorbaz/devel/kesh-gate-logs/e433/captures/`.

## Tests

- **Vitest** : `GroupSettlementModal.test.ts` (ajout, retrait, doublon refusé, total et différence au centime —
  `0.10 + 0.20` —, bouton à une facture, à différence trop grande, à différence admise ; corps envoyé ; refus affiché) ;
  `CancelReconciliationDialog.test.ts` (`grouped_settlement`) ; `failed-proposal-label.test.ts` (codes neufs) ;
  `reconciliation.api.test.ts` (recherche).
- **E2E** (`frontend/tests/e2e/reconciliation-group.spec.ts`, neuf) : un crédit réglant deux factures de **deux clients**
  avec une différence de 0.40, puis son annulation (factures rouvertes) ; un débit réglant deux factures fournisseur
  (commit D4) ; une différence trop grande laisse *Enregistrer* inactif. Données créées par l'API dans la spec. Sonde
  négative : la transaction rapprochée n'offre plus le bouton.
- **Mutation** : retirer la condition « au moins deux » (un test Vitest rougit).

## Tâches

- [ ] **T0** — Étape 0 : LOW de la validation ; contrat de la 43-2 relu (fiche, puis code s'il existe).
- [ ] **T1** — Types et appels. — [ ] **T2** — `GroupSettlementModal.svelte` et son bouton (AC1 à AC3), côté client.
- [ ] **T3** — Dialogue d'annulation (AC4). — [ ] **T4** — i18n (AC5). — [ ] **T5** — Manuel (AC6).
- [ ] **T6** — Commits D4 (côté fournisseur). — [ ] **T7** — E2E sur la branche réunie, captures (AC7).
- [ ] **T8** — Gates frontend complets ; E2E complet à la clôture ; Dev Agent Record ; prompt de revue P1.

## Note de version

- **Ajouté** — Réconciliation : « Régler plusieurs factures… » — sur une ligne du relevé, ajoutez les factures une à une ;
  le total et la différence s'affichent ; Kesh règle et lettre chacune, une petite différence passant au compte de
  différences d'arrondi (refs #823).

## Dev Notes

Modules : `frontend/src/lib/features/reconciliation`, catalogues i18n, `frontend/tests/e2e`, manuel utilisateur.
`ContactPicker.svelte` n'est plus employé (recherche par texte). Manuel aussi touché par 42 et 44 : `.tex` seulement.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (kickoff de l'Epic 43).
- 2026-11-17 (soir) — Révisée : interface comme bexio (ajout une à une, total, différence), tiers mixtes, écart borné,
  côté fournisseur retirable.
