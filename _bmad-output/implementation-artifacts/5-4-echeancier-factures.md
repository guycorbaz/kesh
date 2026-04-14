# Story 5.4: Échéancier factures

Status: backlog

## Contexte

Créée en backlog le 2026-04-14 par Guy Corbaz lors du démarrage de la Story 5.2. La colonne `invoices.value_date` (ajoutée en 5.2) reste inutilisée tant qu'aucune vue ne l'exploite. Cette story crée l'échéancier — vue liste des factures validées triées par `value_date`, avec indicateur impayé/en retard.

## Story

As a **utilisateur (comptable ou indépendant)**,
I want **voir un échéancier des factures validées avec leur date de valeur et leur statut de paiement (payé / impayé / en retard)**,
so that **je puisse suivre mes créances clients, prioriser les relances, et anticiper ma trésorerie**.

## Scope envisagé (à affiner via `bmad-create-story`)

- **Page `/invoices/due-dates`** (ou onglet dans `/invoices`) : liste triée par `value_date ASC`, filtres (impayées / payées / toutes / en retard).
- **Statut de paiement** : dérivé de la réconciliation bancaire (Epic 6) ou, en attendant, champ manuel `invoices.paid_at DATETIME NULL` avec bouton « Marquer payée » (Admin/Comptable).
- **Mise en évidence visuelle** : lignes en retard (today > value_date ET non payée) surlignées.
- **Export CSV** de l'échéancier (pertinent pour transmission comptable).
- **API** : `GET /api/v1/invoices/due-dates?from=&to=&status=` avec pagination.

## Dépendances

- Story 5.2 (colonne `value_date` + `invoice_number`) : **prérequis DOIT être done**.
- Story 5.3 (PDF QR Bill) : non bloquant mais utile pour visualiser.
- Idéalement avant Epic 6 (la réconciliation automatique suppose un échéancier existant).

## Acceptance Criteria

À rédiger via `bmad-create-story`.

## Change Log

| Date       | Version | Description                                                   | Auteur          |
| ---------- | ------- | ------------------------------------------------------------- | --------------- |
| 2026-04-14 | 0.1     | Stub créé en backlog pendant démarrage Story 5.2 (scope v0.6) | Claude Opus 4.6 |
