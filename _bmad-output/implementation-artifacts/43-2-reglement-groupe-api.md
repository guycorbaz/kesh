# Story 43-2 : un règlement pour plusieurs factures — l'API de la réconciliation (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 par l'agent de rédaction de l'Epic 43 (Opus 5.5), par le workflow `bmad-create-story`, code lu
à `8b50fd195`. Release **0.19.0**. **Part de la branche de la 43-1 dès son commit T3** (signatures publiques de
`grouped_settlements.rs`) ; se fusionne après elle. Aucune migration. Décisions de l'epic : C-43-1 à C-43-13
(`_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md`) ; elles font foi. Validation : à lancer.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant qu'intégration (clé d'API) ou écran de réconciliation, je veux accepter **une** transaction contre **plusieurs**
factures d'un même tiers, lire les factures candidates d'un tiers pour cette transaction, et voir puis annuler le
rapprochement qui en résulte — avec les refus par proposition du *Pattern batch*.

## Ce qui se lit au code (`8b50fd195`)

- `AcceptProposalInput` (`routes/reconciliation.rs:855`), enum étiqueté sur `type` : `invoice`, `split`, `rule`,
  `supplier_invoice` ; message de l'extracteur qui énumère les types (:961) ; `bank_transaction_id()` (:903).
- Pré-vol du lot, verrou du compte, aiguillage `accept_one` (:2057), `FailedProposal` (:571), `AcceptedProposal` (:557).
- Acceptation unitaire client `accept_one_invoice` (:2375), fournisseur `accept_one_supplier_invoice` (:3312), comparaison
  exacte du montant fournisseur (:3406).
- Vue d'un rapprochement `ReconciliationTransactionResponse` (:5821) et annulation `CancelReconciliationResponse`
  (:5896), routes déclarées à `crates/kesh-api/src/lib.rs:886-932`.

## Décisions propres à la story

- **D1** — Le code neuf vit dans **`crates/kesh-api/src/routes/reconciliation_group.rs`** (handlers d'acceptation groupée
  appelés par `accept_one`, route des candidates) ; `reconciliation.rs` ne gagne que les deux variantes, l'aiguillage, le
  texte de l'extracteur et les champs de la vue. Raison : ce fichier fait 7 508 lignes, et le multi-devise y travaillera.
- **D2** — Les refus d'un groupe sont **par proposition** (*Pattern batch*, identifiant `bank_transaction_id`) ; un corps
  illisible reste un `400` de tout le lot. Un groupe vide ou d'une seule facture est un refus de la proposition, non un
  `400` : il dépend de la proposition.

## Contrat

### `POST /api/v1/reconciliation/accept` — deux types neufs

```json
{ "bankAccountId": 3, "proposals": [
  { "type": "invoice_group", "bankTransactionId": 41, "invoiceIds": [12, 15, 19] },
  { "type": "supplier_invoice_group", "bankTransactionId": 42, "supplierInvoiceIds": [7, 8] }
] }
```

Succès : `accepted[]` porte, pour la proposition, `bankTransactionId`, `journalEntryId` (l'écriture commune) et
`settledInvoices: [{ "invoiceId" | "supplierInvoiceId", "amount" }]` (champs additifs de `AcceptedProposal`, nuls ou absents
pour les autres types — à trancher en un sens à l'étape 0 et à écrire dans `docs/api-external.md`).

Refus (`failed[]`, `200`) — codes neufs, `details` en objet, jamais un message interpolé :

| Code | Quand | `details` |
|---|---|---|
| `RECONCILIATION_GROUP_INVALID` | moins de 2 ou plus de 50 factures, doublon | `{ "reason": "too_few" \| "too_many" \| "duplicate", "invoiceId"? }` |
| `RECONCILIATION_GROUP_MIXED_CONTACTS` | tiers différents | `{ "invoiceId", "contactId", "expectedContactId" }` |
| `RECONCILIATION_GROUP_AMOUNT_MISMATCH` | somme ≠ montant | `{ "transactionAmount", "sumDue" }` |
| codes existants | facture introuvable, non éligible, déjà payée, lot `generated`, compte de créance = banque, sens, compte bancaire, période | ceux de l'acceptation unitaire, avec `invoiceId` ou `supplierInvoiceId` ajouté |

Les codes existants se **réemploient** (`INVOICE_NOT_FOUND`, `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, ceux de
`supplier_settlement_refusal_failed_proposal`, `PERIOD_LOCKED`, `BANK_ACCOUNT_CLOSED`…) : l'inventaire exact se fait au
code de la 43-1 (`GroupRefusal`) et s'écrit au Change Log, un code par variante, sans `_` fourre-tout.

### `GET /api/v1/reconciliation/group-candidates?bankTransactionId=…&contactId=…`

Comptable+. La transaction doit être `pending`, non rejetée, d'un compte ouvert (sinon `404`, `409 BANK_ACCOUNT_CLOSED`,
`409 RECONCILIATION_ALREADY_RECONCILED` selon le cas, comme `link-candidates`). Crédit → factures clients `validated` non
payées du contact ; débit → factures fournisseur `open` du contact, hors lot `generated`. Réponse :

```json
{ "direction": "client" | "supplier", "transactionAmount": "1250.00",
  "candidates": [ { "id": 12, "number": "F-2026-0012", "date": "2026-10-03", "dueDate": "2026-11-02",
                    "totalTtc": "800.00", "amountDue": "500.00" } ],
  "hasMore": false }
```

Ordre : l'ordre d'imputation (exigibilité, date, identifiant — `imputation_order`, `crates/kesh-db/src/repositories/reconciliation.rs:416`), 100 au
plus, `hasMore` par sentinelle. `amountDue` au centime (`amount_due_to_centime`). Lecture seule, sans verrou.

### Vue et annulation d'un rapprochement

`GET /reconciliation/transactions/{id}` et la réponse de `POST …/cancel` : `kind` gagne `"grouped_settlement"` ; champ
additif `groupedInvoices: [{ "kind": "invoice" | "supplierInvoice", "id", "number", "amount" }]` (vide hors groupe) ;
`cancellable` et `cancelBlockedBy` suivent `entry_cancel_outlook` de la 43-1. Contrat additif : les champs existants
gardent leur sens.

## Critères d'acceptation

- **AC1** — Les deux types sont acceptés par l'extracteur ; son message d'erreur énumère les six types ; un type inconnu
  reste un `400 VALIDATION_ERROR`.
- **AC2** — `invoice_group` : sous le verrou du compte, la transaction rechargée, appel de
  `grouped_settlements::settle_invoices_in_tx` ; succès → `accepted[]` d'AC du contrat, audit `reconciliation.accepted`
  (`details` : `proposalType: "invoice_group"`, `journalEntryId`, `invoices: [{ id, amount }]`, `transactionAmount`) avec
  l'**acteur** (clé d'API comprise) ; chaque refus d'un `GroupRefusal` → son code, rien d'écrit pour cette proposition, les
  autres propositions du lot passent (point de sauvegarde par proposition, comme `accept_one`).
- **AC3** — `supplier_invoice_group` : symétrique, `pay_supplier_invoices_in_tx`.
- **AC4** — Un lot mêlant une proposition `invoice`, une `invoice_group` valide et une `invoice_group` refusée rend deux
  `accepted` et un `failed`, dans l'ordre du lot.
- **AC5** — La route des candidates répond selon le contrat, pour un crédit et pour un débit ; une facture déjà dans un
  autre groupe (payée) n'y figure plus ; un tiers d'une autre société → liste vide (aucune fuite).
- **AC6** — Vue et annulation : un groupe client et un groupe fournisseur s'affichent `grouped_settlement` avec leurs
  factures ; l'annulation les défait (43-1, AC9) et rend la vue mise à jour ; un refus d'annulation rend le code du
  premier motif, comme aujourd'hui.
- **AC7** — Libellés d'audit : les actions et champs neufs ont leur libellé dans `crates/kesh-api/src/audit_labels.rs` (et
  les catalogues si la garde l'exige) ; le journal d'audit de l'écran les affiche.
- **AC8** — `docs/api-external.md` : section « Régler plusieurs factures par une transaction » — les deux types, la route
  des candidates, les codes, les champs additifs de la vue, *depuis la v0.19.0*, et la phrase sur le *Pattern batch*.
- **AC9** — Rôles : Comptable+ partout, comme l'acceptation unitaire ; une clé d'API à la portée d'écriture de la
  réconciliation peut accepter un groupe ; sans cette portée, `403`.

## Tests

`crates/kesh-api/tests/reconciliation_group_e2e.rs` (neuf) : AC1 à AC6 et AC9, chacun avec sa sonde négative (le refus
qui prouve que le succès ne vient pas d'une route ouverte) ; un test par code de refus ; lot mixte d'AC4 ; acteur clé d'API
dans `reconciliation.accepted`, `invoice.settled`, `supplier_invoice.paid` et à l'annulation. Mutation : retirer la
comparaison de tiers (le test `MIXED_CONTACTS` rougit) ; renvoyer le refus en `AppError` (le test du lot mixte rougit).

## Tâches

- [ ] **T0** — Étape 0 : LOW de la validation ; inventaire des refus de `GroupRefusal` (43-1) → codes, au Change Log.
- [ ] **T1** — Variantes, extracteur, aiguillage (AC1).
- [ ] **T2** — `routes/reconciliation_group.rs` : acceptations (AC2 à AC4), audit.
- [ ] **T3** — Route des candidates, déclarée dans `lib.rs` (AC5).
- [ ] **T4** — Vue et annulation (AC6), libellés d'audit (AC7).
- [ ] **T5** — `docs/api-external.md` (AC8).
- [ ] **T6** — Gates (backend ; `kesh-db` non touché sauf exception déclarée), Dev Agent Record, prompt de revue P1.

## Note de version

- **Ajouté (API)** — `POST /reconciliation/accept` accepte `invoice_group` et `supplier_invoice_group` ;
  `GET /reconciliation/group-candidates` ; la vue d'un rapprochement rend `grouped_settlement` et `groupedInvoices`
  (refs #823).

## Dev Notes

### Modules touchés

`kesh-api/routes/reconciliation` (variantes, aiguillage, vue), `routes/reconciliation_group` (neuf), `lib.rs` (route),
`audit_labels`, `errors` si un code l'exige, `docs/api-external.md`. Quatre modules.

### Fichiers communs

`audit_labels.rs` et `errors.rs` sont aussi touchés par 42-2/42-3 : blocs distincts, conflit résolu en gardant les deux.

### Hors périmètre

Le modèle et l'annulation (43-1) ; l'écran (43-3) ; le MCP (lecture seule : rien n'y change, à vérifier qu'aucun outil ne
sérialise `kind` en énumération fermée — ligne lue citée).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (kickoff de l'Epic 43).
