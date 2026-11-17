# Story 43-2 : un règlement pour plusieurs factures — l'API de la réconciliation (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`, code lu à
`8b50fd195` ; **révisée le soir même** (décisions de Guy : tiers mixtes, écart, recherche par texte ; validation P1 de la
43-1 : H1, M4). Release **0.19.0**. **Part de la 43-1a à son commit T3** (signatures de `grouped_settlements.rs`) ; se
fusionne après elle. Aucune migration. Décisions : C-43-1 à C-43-20 (fichier d'épique), qui font foi. Validation : à
lancer.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant qu'intégration (clé d'API) ou écran de réconciliation, je veux accepter **une** transaction contre **plusieurs**
factures, chercher les factures candidates par texte, et voir puis annuler le rapprochement qui en résulte — avec les
refus par proposition du *Pattern batch*.

## Ce qui se lit au code (`8b50fd195`)

- `AcceptProposalInput` (`routes/reconciliation.rs:855`) : `invoice`, `split`, `rule`, `supplier_invoice` ; message de
  l'extracteur (:961) ; `bank_transaction_id()` (:903) ; `accept_one` (:2057) ; `FailedProposal` (:571) ;
  `AcceptedProposal` (:557).
- `accept_one_invoice` (:2375) et `accept_one_supplier_invoice` (:3312) écrivent leurs traces dans la route ; le groupe,
  lui, les écrit dans le dépôt (C-43-9).
- Vue `ReconciliationTransactionResponse` (:5821), annulation `CancelReconciliationResponse` (:5896), routes déclarées
  `crates/kesh-api/src/lib.rs:886-932`.

## Décisions propres à la story

- **D1** — Code neuf dans `crates/kesh-api/src/routes/reconciliation_group.rs` ; `reconciliation.rs` ne gagne que les
  variantes, l'aiguillage, le texte de l'extracteur et les champs de la vue.
- **D2** — Refus **par proposition** (*Pattern batch*, identifiant `bank_transaction_id`) : chaque variante de
  `GroupRefusal` (43-1a T3) a **son** code, `details` en objet ; un `DbError` hors `GroupRefusal` reste une erreur de la
  transaction, comme pour l'unitaire. Un corps illisible reste un `400` du lot.
- **D3** — **La route n'écrit aucune trace d'audit** pour un groupe : le dépôt écrit `reconciliation.accepted` puis les
  traces par facture (H1).
- **D4** — Le type fournisseur et ses tests vivent dans des **commits séparés**, retirables si la 43-1b sort (C-43-18).

## Contrat

### `POST /api/v1/reconciliation/accept` — deux types neufs

```json
{ "bankAccountId": 3, "proposals": [
  { "type": "invoice_group", "bankTransactionId": 41, "invoiceIds": [12, 15, 19] },
  { "type": "supplier_invoice_group", "bankTransactionId": 42, "supplierInvoiceIds": [7, 8] }
] }
```

`paymentEntryIds` (paiements existants) s'ajoute avec la 43-5, champ facultatif. Succès : `accepted[]` gagne, pour ces
types, `journalEntryId`, `difference` (chaîne décimale, `"0.00"` sans écart) et `settledInvoices: [{ "invoiceId" |
"supplierInvoiceId", "amount" }]` — champs additifs, absents pour les autres types.

Refus (`failed[]`, `200`) :

| Code | Quand | `details` |
|---|---|---|
| `RECONCILIATION_GROUP_INVALID` | moins de 2 ou plus de 50, doublon | `{ "reason": "too_few" \| "too_many" \| "duplicate", "invoiceId"? }` |
| `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE` | `|d| > 5.00` | `{ "transactionAmount", "sumApplied", "difference", "maxDifference" }` |
| `ROUNDING_ACCOUNT_NOT_CONFIGURED` | écart sans compte d'arrondi | `{}` |
| `PROJECT_NOT_TAGGABLE` (ou le code de `project_error_to_failed_proposal`) | projet d'une facture archivé ou inexistant | `{ "invoiceId", "projectId" }` |
| codes de l'unitaire | facture introuvable, non éligible, payée, lot `generated`, compte de créance = banque ou = compte d'arrondi, sens, compte bancaire, période | ceux de l'unitaire, avec `invoiceId` ou `supplierInvoiceId` |

L'inventaire exact (variante → code) s'écrit au Change Log à l'étape 0, sans code fourre-tout ; il s'aligne sur
`failed-proposal-label.ts` (43-3).

### `GET /api/v1/reconciliation/group-candidates?bankTransactionId=…&q=…&limit=…`

Comptable+. Transaction `pending`, non rejetée, compte ouvert (sinon `404`, `409 BANK_ACCOUNT_CLOSED`, `409
RECONCILIATION_ALREADY_RECONCILED`, comme `link-candidates`). **Tous tiers.** Crédit → factures clients `validated`, reste
dû > 0 ; débit → factures fournisseur `open`, hors lot `generated`. `q` (facultatif) filtre sur le numéro, le numéro
interne, le nom du tiers et la référence de paiement, insensible à la casse ; sans `q`, les plus anciennes d'abord.

```json
{ "direction": "client" | "supplier", "transactionAmount": "1250.00", "maxDifference": "5.00",
  "candidates": [ { "kind": "invoice", "id": 12, "number": "F-2026-0012", "contactName": "Exemple SA",
                    "date": "2026-10-03", "dueDate": "2026-11-02", "totalTtc": "800.00", "amountDue": "500.00" } ],
  "hasMore": false }
```

Ordre d'imputation (`imputation_order`, `crates/kesh-db/src/repositories/reconciliation.rs:416`), `limit` 50 par défaut,
100 au plus, `hasMore` par sentinelle ; montants au centime (`amount_due_to_centime`). Lecture seule. Avec la 43-5 :
`kind: "payment"`, `entryId`.

### Vue et annulation

`GET /reconciliation/transactions/{id}` et `POST …/cancel` : `kind` gagne `"grouped_settlement"` ; champ additif
`groupedInvoices: [{ "kind": "invoice" | "supplierInvoice", "id", "number", "amount" }]` (vide hors groupe) ;
`cancellable`, `cancelBlockedBy` suivent `entry_cancel_outlook` (43-1a). Les champs existants gardent leur sens.

## Critères d'acceptation

- **AC1** — Extracteur : les deux types acceptés, message à six types ; type inconnu → `400 VALIDATION_ERROR`.
- **AC2** — `invoice_group` : sous le verrou du compte, appel de `settle_invoices_in_tx` ; succès → `accepted[]` du
  contrat ; chaque `GroupRefusal` → son code, rien d'écrit pour cette proposition, les autres passent (point de sauvegarde
  par proposition, comme `accept_one`). La route n'écrit aucune trace (D3) : le test lit `reconciliation.accepted`, puis
  `invoice.paid` par facture, avec l'acteur.
- **AC3** — `supplier_invoice_group` : symétrique, `pay_supplier_invoices_in_tx` (commits D4).
- **AC4** — Lot mixte (`invoice` + `invoice_group` valide + `invoice_group` refusé) : deux `accepted`, un `failed`, dans
  l'ordre.
- **AC5** — Recherche des candidates selon le contrat : crédit et débit ; `q` ; tiers différents dans la même liste ; une
  facture d'une autre société jamais rendue.
- **AC6** — Vue et annulation d'un groupe client (et fournisseur, D4) ; refus d'annulation par le code du premier motif.
- **AC7** — `docs/api-external.md` : « Régler plusieurs factures par une transaction » (*depuis la v0.19.0*), types,
  recherche, codes, champs additifs, différence et son plafond, *Pattern batch*.
- **AC8** — Rôles : Comptable+ ; clé d'API à la portée d'écriture de la réconciliation ; sans elle, `403`.

## Tests

`crates/kesh-api/tests/reconciliation_group_e2e.rs` (neuf) : AC1 à AC6 et AC8, chacun avec sa sonde négative ; un test
par code ; acteur clé d'API dans `reconciliation.accepted` et `invoice.paid`. Mutations : renvoyer un refus en `AppError`
(AC4 rougit) ; écrire `reconciliation.accepted` aussi dans la route (le test qui compte les traces rougit).

## Tâches

- [ ] **T0** — Étape 0 : LOW de la validation ; inventaire `GroupRefusal` → codes.
- [ ] **T1** — Variantes, extracteur, aiguillage (AC1). — [ ] **T2** — `invoice_group` (AC2, AC4).
- [ ] **T3** — Recherche (AC5). — [ ] **T4** — Vue et annulation (AC6). — [ ] **T5** — `docs/api-external.md` (AC7).
- [ ] **T6** — Commits D4 : `supplier_invoice_group` (AC3) et son côté d'AC5, AC6.
- [ ] **T7** — Gates ; Dev Agent Record ; prompt de revue P1.

## Note de version

- **Ajouté (API)** — `POST /reconciliation/accept` accepte `invoice_group` et `supplier_invoice_group` ;
  `GET /reconciliation/group-candidates` ; la vue d'un rapprochement rend `grouped_settlement` et `groupedInvoices`
  (refs #823).

## Dev Notes

Modules : `routes/reconciliation`, `routes/reconciliation_group` (neuf), `lib.rs`, `errors` si un code l'exige,
`docs/api-external.md`. `audit_labels.rs` inchangé (aucune action neuve). Le MCP est en lecture seule : vérifier qu'aucun
outil ne sérialise `kind` en énumération fermée (ligne lue citée).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (kickoff de l'Epic 43).
- 2026-11-17 (soir) — Révisée : tiers mixtes, recherche par texte, écart et son code, audit écrit par le dépôt (H1),
  `GroupRefusal` complet (M4), type fournisseur en commits retirables.
