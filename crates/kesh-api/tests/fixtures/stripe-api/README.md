# Réponses fictives de l'API Stripe (Story 40-8a)

Servies par le serveur Stripe simulé de `tests/stripe_client.rs`. **Inventées** : aucune réponse réelle, aucun
identifiant réel. Elles ont la **forme** des réponses de `GET /v1/payouts` et `GET /v1/balance_transactions` relevées
en lecture seule sur un compte réel (champs, types, ordre « le plus récent d'abord », transaction du versement
lui-même dans `?payout=`), et les **valeurs** de la fixture CSV de 40-7a
(`crates/kesh-import/tests/fixtures/stripe-itemized-payout-fictif.csv`) : le test exige que les lignes lues par l'API
égalent celles du CSV pour les mêmes versements.

| Fichier | Contenu |
|---|---|
| `payouts-p1.json` | Première page des versements (`has_more: true`) : `po_TEST0003`, `po_TEST0005` (**manuel**, `automatic: false`), `po_TEST0002`. |
| `payouts-p2.json` | Seconde page (après `po_TEST0002`) : `po_TEST0001`. |
| `bt-po_TEST0001.json` à `bt-po_TEST0003.json` | Transactions de solde de chaque versement automatique, dont la transaction du versement lui-même (`type: payout`, `txn_TESTPO…`), à écarter ; montants en centimes, instants en secondes d'époque, heures de Zurich du CSV. |
| `erreur-400.json`, `erreur-401.json`, `erreur-403.json`, `erreur-429.json` | Corps d'erreur de la forme de Stripe ; leur `message` porte le mot `FUITE`, qui ne doit sortir ni dans une réponse de Kesh ni au journal. |

Le versement `po_TEST0004` du CSV (un seul `payout_reversal`, net négatif) n'a pas d'équivalent ici : un versement
automatique payé à montant négatif n'existe pas chez Stripe.
