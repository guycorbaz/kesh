# Correctif : la destination d'un versement Stripe rejugée à la revue et à l'acceptation

Status: review

*Fiche rédigée le 2026-11-22, branche `fix/40-destination-rejugee`, partie d'`epic/40-stripe` à `15e9f8b13`.
Correctif de l'Epic 40, non urgent, **sans migration**. Relevé par la revue d'intégration de l'Epic 39 (M2, MEDIUM ;
L3, LOW), raisonné au code par la revue, non exécuté par elle.*

## Intention

La règle « la destination d'un versement Stripe est un compte de la société, ni archivé, ni clos, **ni un compte
Stripe ni PayPal**, dont le compte du plan est **dans la devise** du solde » (Story 40-7b, AC-2) ne se jugeait qu'au
`POST`/`PUT` **du compte Stripe** (`check_payout_destination_in_tx`). Or le compte désigné peut changer ensuite : un
`PUT` le fait passer à PayPal ou à Stripe tant qu'il n'a pas de transaction, l'archive, le clôt, ou change son compte
du plan. Ni la revue (`destination_usable`) ni l'acceptation (`accept_payout`) ne relisaient le prestataire ni la
devise : le versement était alors proposé, puis passé **au débit du compte du plan d'un solde PayPal ou Stripe**, sans
aucun signal. Erreur d'imputation muette (P1 au sens de § *Priorités des défauts*, sur une branche non publiée).

**Prémisse vérifiée au code** (`15e9f8b13`) : `destination_usable` (`routes/stripe.rs:98-100`) ne lisait que
`archived`, `is_closed()`, `journal_account_id` ; `accept_payout` ne contrôlait qu'archivé, clos, lien au plan, puis
`check_counterparty(AssetInCurrency)` (type et devise, sous `VALIDATION_ERROR`) — `grep -nF 'provider'
routes/stripe.rs` ne rendait aucun contrôle du prestataire de la destination.

## Décision

1. **Une seule règle** : `PayoutDestination::refusal(currency)` (`kesh-db`, `repositories/bank_accounts.rs`), motifs
   `PAYOUT_DESTINATION_*` (`archived`, `closed`, `stripe`, `paypal`, `currency`, et `not_found`), dans cet ordre.
   `check_payout_destination_in_tx` (réglage), la revue et l'acceptation l'appellent ; `PayoutDestination::of`
   construit les faits d'un compte lu. `routes/stripe.rs::destination_refusal` y ajoute le lien au plan
   (`not_configured`, jugé avant la devise : sans compte du plan, la devise lue ne dit rien).
2. **Revue** : `find_payout_destinations` (qui remplace `find_by_ids`, seul appelant) lit chaque destination avec la
   devise de son compte du plan, dans la transaction de lecture ; une destination refusée n'est plus proposée, le
   versement va à examiner sous `payout_account_missing`, désormais **au motif nommé** (`details: { payoutId, reason
   }`, `reason` ∈ `not_set`, `not_found`, `archived`, `closed`, `stripe`, `paypal`, `currency`, `not_configured`).
3. **Acceptation** : la destination relue est rejugée par la même fonction. Archivé, clos et sans compte du plan
   gardent leur issue (`PROVIDER_PROPOSAL_STALE`/`payout_account_archived`, `BANK_ACCOUNT_CLOSED`,
   `BANK_ACCOUNT_NOT_CONFIGURED`) ; **Stripe, PayPal ou autre devise → `FailedProposal` `STRIPE_PAYOUT_ACCOUNT_INVALID`
   `{ bankAccountId, reason }`**, le code du réglage refusé, dans `failed[]` en `200` (*Pattern batch*), sans écriture.
   ⚠️ Changement de contrat sur une branche non publiée : une destination d'une autre devise rendait
   `VALIDATION_ERROR`/`counterparty_wrong_currency` ; elle rend maintenant `STRIPE_PAYOUT_ACCOUNT_INVALID`/`currency`.
4. **Écrans** : `stripeFailedLabel` nomme `STRIPE_PAYOUT_ACCOUNT_INVALID` par la clé globale
   `error-stripe-payout-account-invalid` (déjà aux quatre locales) ; le texte de `payout_account_missing` dit la règle
   entière (quatre locales). Pas de texte par motif : le motif est dans l'API.
5. **Non retenu** : refuser au `PUT` du compte de destination le changement de prestataire d'un compte désigné. Le
   rejugement à l'usage suffit à fermer l'erreur d'imputation et couvre aussi archivage, clôture et changement de
   compte du plan, que les routes de la destination admettent.

## Critères

- AC-1 : destination devenue PayPal, Stripe, archivée, close, ou d'une autre devise → la revue ne propose pas le
  versement et le met à examiner au motif nommé ; la proposition d'avant est refusée à l'acceptation ; aucune écriture,
  transaction `pending`. Remise en état : acceptée.
- AC-2 : une seule fonction juge la règle (réglage, revue, acceptation).
- AC-3 (L3) : versement vers une banque ordinaire, accepté → écart `payout_without_receipt` ; réception de la banque
  rattachée → l'écart disparaît. Le côté Wise par la route Wise n'est pas jouable sur `epic/40` (la route Wise est de
  l'Epic 39) : il reste à la 39-11d2 ou au solde ; le côté Wise par `link-entry` est déjà joué par
  `revue_puis_acceptation_du_tout`.

## Tests

- `stripe_reconciliation_e2e::destination_devenue_inutilisable_rejugee_a_la_revue_et_a_l_acceptation` (AC-1, cinq cas) ;
- `stripe_reconciliation_e2e::versement_vers_une_banque_puis_reception_l_ecart_disparait` (AC-3) ;
- `stripe_reconciliation_e2e::destination_de_versement_sans_plan_de_mauvais_type_ou_de_mauvaise_devise` : cas de la
  devise passé à `STRIPE_PAYOUT_ACCOUNT_INVALID`/`currency` ;
- `routes::stripe::tests::libelle_et_usage_d_un_compte_de_versement` (unitaire, `destination_refusal` : Stripe, PayPal,
  Wise admis, archivé, clos, devise, sans plan) ;
- `kesh_reconciliation::stripe::tests::versements_proposes_examines_ou_en_ecart` : motif `not_set`, puis motif rendu ;
- `stripe-labels.test.ts` : libellé de `STRIPE_PAYOUT_ACCOUNT_INVALID`.

## Notes de version

- **Fixed** — Stripe : le compte de versement d'un compte Stripe est rejugé à chaque revue et à chaque acceptation ; s'il
  est devenu un compte Stripe ou PayPal, a été archivé, clos ou lié à un compte du plan d'une autre devise, le
  versement va « à examiner » (motif nommé) et son acceptation est refusée (`STRIPE_PAYOUT_ACCOUNT_INVALID`), au lieu
  d'être passé au débit d'un solde de prestataire.

## Change Log

- 2026-11-22 — Développement (voir le Dev Agent Record).

## Dev Agent Record

*(Complété aux gates.)*
