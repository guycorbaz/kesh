# Epic 38 — findings inter-fiches de la validation P2 (2026-11-15), à appliquer

Chaque fiche a ses correctifs propres (P2, Opus). 38-4 et 38-5 sont closes (LOW seulement).

## Décisions de l'orchestrateur

- **D-P2-1 — Code du refus sur un compte prestataire** : un seul code, **`PROVIDER_PROPOSAL_REQUIRED`** (celui de
  C-38-8), posé par **38-3a** ; 38-6a le reprend. `PROVIDER_TX_REQUIRES_PROVIDER_PROPOSAL` disparaît partout.
- **D-P2-2 — Q-38-6a-1, option (b)** : sur un compte prestataire, `rule` et `split` restent refusés ; une facture
  (client ou fournisseur) n'est proposée **que si `fee_amount = 0`** ; sinon la proposition `provider`. Le règlement
  d'une facture au brut avec ligne de frais est **à tracer en issue** (C-38-13).
- **D-P2-3 — Découpage de 38-6c** en **38-6c1** (AC-1 à AC-11 : base de mouvement, suggestions, lot, API, exercice clos)
  et **38-6c2** (AC-12 à AC-15 : groupe d'écritures B28, table `bank_transaction_extra_entries`, paires, annulation de
  groupe). 38-6c2 se développe avant 38-6a. Registre : `38-6c-…: split`, deux clés `backlog`.
- **D-P2-4 — O8 étendu au CHF** : le compte du plan d'un compte prestataire, en devise **ou en CHF**, n'est lié à aucun
  autre compte bancaire (`PROVIDER_ACCOUNT_LEDGER_SHARED`, deux sens).

## Epic (`epic-38-paypal.md`) et registre

1. C-38-8 : D-P2-1 ; C-38-3 : `PROVIDER_ACCOUNT_HAS_TRANSACTIONS` (pose de réglages refusée sur un compte qui porte des
   transactions) ; O8/E11 : D-P2-4 ; C-38-5 : méthodes `max_movements_per_page()` et `test()` de `Connector` ;
   C-38-13 : issue « règlement de facture au brut avec frais » (D-P2-2).
2. D-P2-3 : tableau des stories, dépendances, récapitulatif FR/NFR (recompté aux fiches), `sprint-status.yaml`.
3. § 38-9b et récapitulatif : FR-38-9b-3 (paires, groupe composé à la main) et FR-38-9b-5 (annulation d'une paire ou
   d'un groupe) repris.

## Fiches

- **38-3a** : D-P2-1 (code) ; point de contact « 38-6a lève ce saut pour sa variante `provider` ; `rule` et `split`
  restent refusés ; factures selon D-P2-2 » ; `BankTransaction::item_names() -> Result<Vec<String>, DbError>`.
- **38-3b** : rejouer la course de 38-2 AC-6 (c) contre l'insertion d'un passage ; `validate_page` : unicité du couple
  `(provider_tx_id, currency)` dans une page ; AC-3 : variante additive `paypalResponses: [{ method, path, query?,
  status, body }]` du corps de `POST /api/v1/_test/provider-connector` (servies dans l'ordre, la dernière se répète ;
  avec elle `PaypalConnector` tourne sur le transport factice ; `TEST_ENDPOINT_ROUTES` inchangé).
- **38-6a** : D-P2-1 et D-P2-2 ; « déjà comptabilisée » non nul dès que `pairCount` l'est, `TRANSACTION_ALREADY_BOOKED`
  porte `pairCount` ; dépendance à 38-6c2.
- **38-6b** : renvois à 38-6c1/38-6c2.
- **38-6c → 38-6c1/38-6c2** : D-P2-3 ; relevé (f) de l'AC-12 : les écritures de `matched_entry_id` de 38-6b (paire,
  paire à moitié CHF, virement) « par construction » ; `CancelReconciliationResponse` gagne
  `groupReversalJournalEntryIds` (additif, `null` hors groupe) — aligner avec `reversalJournalEntryIds` déjà ajouté
  (un seul nom).
- **38-7** : AC-1 : délai de connexion 5 s sur le client, délai total 60 s et corps ≤ 8 MiB par requête, tests par
  `HttpTransport::for_local_tests()`, dépassement → `InvalidResponse` ; AC-3/4/6 : `InvalidResponse { reason:
  &'static str }` ; AC-8 : `PaypalConnector` implémente `Connector::test()` → `ConnectorTest { account_ref =
  account_id, balances }`, la route l'appelle sur le connecteur de la fabrique.
- **38-8** : AC-5 : « une paire coupée ou sans candidat garde `review_reason = 'other'` ; 38-6b l'ignore dès que l'autre
  moitié s'y relie (C-38-6b-4) ».
- **38-9a** : Q-38-9a-1 close (seul `ignoredCount`) ; AC-6 : `PATCH …/balances-seen` de 38-3b AC-7 ; AC-11 : faits de
  38-3b AC-8 (contrôle chaque heure, passage 24 h après le début du dernier, asynchrone, « interrompu », blocage 6 h au
  plus après une sauvegarde importée, `overlapDays`, `truncated_from`, devise d'un compte archivé ou clos) ; message
  de `PROVIDER_ACCOUNT_HAS_TRANSACTIONS` ; manuel : retrait d'un compte créé par erreur (délier, archiver, retirer les
  réglages, supprimer la connexion) ; `payerId` remis à `null` quand `clientId` change (« Tester » le repose).
- **38-9b** : afficher `pairCount` ; renvois à 38-6c1/38-6c2 ; nom du champ de contre-passations d'un groupe aligné sur
  38-6c2.
