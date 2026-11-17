# Story 43-3 : un règlement pour plusieurs factures — l'écran de réconciliation (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 par l'agent de rédaction de l'Epic 43 (Opus 5.5), par le workflow `bmad-create-story`, code lu
à `8b50fd195`. Release **0.19.0**. **Commence tout de suite**, en parallèle de 43-1 et 43-2 : Vitest sur le contrat de la
fiche 43-2 (maquettes) ; l'**E2E** se joue sur la branche qui porte 43-1 et 43-2 fusionnées. Frontend et manuel
utilisateur seulement ; aucune migration. Décisions de l'epic : C-43-1 à C-43-13 ; elles font foi. Validation : à lancer.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant que comptable, devant un crédit (ou un débit) du relevé qui règle plusieurs factures, je veux choisir le client (ou
le fournisseur), cocher ses factures ouvertes en voyant le total se rapprocher du montant, et valider quand il est égal —
puis, si je me suis trompé, annuler ce rapprochement en voyant les factures qu'il rouvrira.

## Ce qui se lit au code (`8b50fd195`)

- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` : actions par transaction et modales
  `ManualMatchModal`, `TransactionSplitModal`, `LinkSettlementModal` (:78-80) ; patron le plus proche :
  `LinkSettlementModal.svelte` (choix dans une liste lue d'une route de candidates).
- `CancelReconciliationDialog.svelte` et `reconciliation-cancel.ts` : textes par `kind` (`reconciliation.types.ts:204`,
  `'invoice_settlement' | 'supplier_settlement' | 'entry' | null`).
- `reconciliation.api.ts` : appels de `accept`, `link-candidates`.
- Sélecteur de tiers : `frontend/src/lib/components/invoices/ContactPicker.svelte` — **employé, non modifié** (la
  correction #1162 de la même release le modifie).
- Specs voisines : `frontend/tests/e2e/reconciliation-link.spec.ts`, `reconciliation-supplier.spec.ts`,
  `reconciliation-cancel.spec.ts`.

## Décisions propres à la story

- **D1** — Un bouton **« Régler plusieurs factures… »** par transaction en attente, à côté de « Rattacher… », ouvre
  `GroupSettlementModal.svelte` (neuf). Le sens de la transaction choisit la famille : crédit → clients, débit →
  fournisseurs ; le titre le dit.
- **D2** — Aucune présélection automatique d'un tiers : le nom de la contrepartie du relevé peut **préremplir la
  recherche** du sélecteur, jamais choisir (règle « un appariement propose, il ne crée jamais »).
- **D3** — Le bouton *Régler* n'est actif que si **au moins deux** factures sont cochées et que leur total au centime égale
  le montant ; sinon l'écart s'affiche (« Il manque 120.00 » / « 120.00 de trop »). Le serveur reste juge : un refus
  `failed[]` s'affiche par son libellé traduit, la modale reste ouverte.

## Critères d'acceptation

- **AC1** — Le bouton existe sur une transaction en attente non rejetée, d'un compte ouvert ; absent sur une transaction
  rapprochée ou rejetée.
- **AC2** — Modale : choix du tiers (`ContactPicker`, filtré client ou fournisseur selon le sens) ; puis la liste de
  `GET /reconciliation/group-candidates` (numéro, date, échéance, reste dû), cases à cocher ; total coché, montant de la
  transaction, écart ; *Régler* actif selon D3 ; liste vide → phrase qui le dit ; `hasMore` → phrase qui le dit.
- **AC3** — *Régler* envoie `POST /reconciliation/accept` avec `invoice_group` ou `supplier_invoice_group` ; succès →
  toast, modale fermée, transaction retirée des propositions ; refus → libellé du code (`failed-proposal-label.ts` étendu
  aux trois codes neufs de la 43-2), facture en cause nommée quand `details` la donne.
- **AC4** — `CancelReconciliationDialog` : `kind = "grouped_settlement"` → texte qui annonce que les **N** factures
  (numéros listés, `groupedInvoices`) redeviendront ouvertes et que l'écriture sera contre-passée.
- **AC5** — i18n : toutes les chaînes dans les quatre locales (`fr-CH` rédigé ; `de-CH`, `it-CH`, `en-CH` traduits), garde
  `lint-i18n-ownership` et `sitesTotal` de `i18n-keys.test.ts` recomptés depuis la source après l'ajout.
- **AC6** — Manuel utilisateur (`docs/manual/fr/user-manual.tex`, `.tex` seulement ; § *Réconciliation bancaire*,
  `\label{sec:reconciliation}`, et § *Annuler un rapprochement*, `\label{sec:annuler-rapprochement}`) : « Régler plusieurs
  factures par une transaction » — quand l'employer, même tiers, somme exacte, un règlement et un lettrage par facture,
  annulation en entier seulement ; limites : le débit d'un lot pain.001 **confirmé** ne s'y rattache pas (C-43-10) ; la
  ligne de banque d'un règlement groupé ne reçoit pas de compte analytique (43-1, *Limites*).
- **AC7** — Captures (thème clair, base de démonstration jetable) : modale vide, tiers choisi, total inférieur, total égal,
  refus serveur, dialogue d'annulation d'un groupe — dans `/home/gcorbaz/devel/kesh-gate-logs/e433/captures/`.

## Tests

- **Vitest** : `GroupSettlementModal.test.ts` (D3 : bouton inactif à une facture, à total différent ; actif à total égal ;
  calcul au centime sans flottant — `0.10 + 0.20` ; corps envoyé pour chaque famille ; refus affiché) ;
  `CancelReconciliationDialog.test.ts` (cas `grouped_settlement`) ; `failed-proposal-label.test.ts` (trois codes) ;
  `reconciliation.api.test.ts` (route des candidates).
- **E2E** (`frontend/tests/e2e/reconciliation-group.spec.ts`, neuf) : un crédit réglant deux factures clients puis son
  annulation (factures rouvertes) ; un débit réglant deux factures fournisseur ; un total différent qui laisse *Régler*
  inactif. Données créées par l'API dans la spec. Sonde négative : la même transaction, une fois rapprochée, n'offre plus le
  bouton.
- **Mutation** : retirer la condition « au moins deux » de D3 (un test Vitest rougit).

## Tâches

- [ ] **T0** — Étape 0 : LOW de la validation ; contrat de la 43-2 relu à sa dernière version (fiche, puis code s'il existe).
- [ ] **T1** — Types et appels (`reconciliation.types.ts`, `reconciliation.api.ts`) selon le contrat.
- [ ] **T2** — `GroupSettlementModal.svelte` et son bouton (AC1 à AC3).
- [ ] **T3** — Dialogue d'annulation (AC4), libellés de refus.
- [ ] **T4** — i18n (AC5).
- [ ] **T5** — Manuel utilisateur (AC6).
- [ ] **T6** — E2E sur la branche réunie (43-1 + 43-2) ; captures (AC7).
- [ ] **T7** — Gates frontend complets ; E2E complet à la clôture, au dernier commit de code ; Dev Agent Record ; prompt
  de revue P1.

## Note de version

- **Ajouté** — Réconciliation : « Régler plusieurs factures… » sur une transaction en attente — choisir le client ou le
  fournisseur, cocher ses factures ; Kesh règle et lettre chacune quand leur total égale le montant (refs #823).

## Dev Notes

### Modules touchés

`frontend/src/lib/features/reconciliation`, catalogues i18n, `frontend/tests/e2e`, `docs/manual/fr/user-manual.tex`.

### Fichiers communs

Catalogues i18n (bloc de l'epic) ; `ContactPicker.svelte` employé tel quel — si #1162 change son interface, la 43-3 suit
au raccord ; manuel utilisateur aussi touché par 42 et 44 (`.tex` seulement, PDF à la release).

### Hors périmètre

Règlement partiel dans un groupe ; groupe de plusieurs tiers ; lot pain.001 confirmé.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (kickoff de l'Epic 43).
