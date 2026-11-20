# Story 40-8a : client de l'API Stripe — clé restreinte en variable d'environnement, versements et transactions de solde

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`.
Epic 40, **phase 2** (décision de Beat du 2026-11-20, § *Phase 2* du fichier d'epic). **Dépend de** : la phase 1 (40-7a
à 40-7d) ; la phase 2 de l'Epic 38 (client PayPal) s'il est fait avant, pour le patron de configuration par compte.
**Suivie par** : 40-8b (téléchargement).*

## Story

En tant qu'**Administrateur**, je veux poser dans l'environnement de Kesh une clé Stripe **restreinte, en lecture
seule**, rattachée au libellé de mon compte Stripe, afin que Kesh puisse lire mes versements et leurs transactions de
solde sans que la clé soit jamais en base ni dans une sauvegarde.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-8a-1` | Une clé par compte Stripe, en variables d'environnement, rattachée au libellé du compte (réglage de 40-7b) ; rien en base, rien en sauvegarde, rien dans les journaux. | AC-1 |
| `FR-40-8a-2` | Seule une clé restreinte est admise (`rk_live_…`, `rk_test_…`) ; une clé secrète `sk_…` est refusée au démarrage. | AC-1 |
| `FR-40-8a-3` | Lire, pour une période, les versements automatiques payés et, pour chacun, ses transactions de solde ; les rendre comme les lignes du rapport CSV (`StripePayoutRow`, 40-7a). | AC-2, AC-3 |
| `FR-40-8a-4` | Pagination par curseur, 100 par page ; réponses d'erreur traduites (clé invalide, droits manquants, limite atteinte). | AC-2, AC-4 |
| `NFR-40-8a-1` | Aucun appel réseau en CI ni par un agent : une source simulée sert aux tests. | AC-5 |

## Critères d'acceptation

1. **AC-1 — Configuration.** `crates/kesh-api/src/config.rs` lit `KESH_STRIPE_<n>_ACCOUNT` (libellé, égal au
   `provider_account` d'un compte Stripe de 40-7b) et `KESH_STRIPE_<n>_KEY`, `n` de 1 à 9, paires complètes ou absentes
   (une moitié seule → refus de démarrer, message qui nomme la variable). La clé suit le patron de
   `KESH_SMTP_PASSWORD` (`config.rs:329-332` : champ privé, masqué dans `Debug`, accesseur) ; préfixe `rk_live_` ou
   `rk_test_` exigé, `sk_` refusé (« utilisez une clé restreinte en lecture »), valeur de gabarit `<…>` refusée comme
   ailleurs (`is_template_placeholder`). Le mode (`live`/`test`) se lit au préfixe et se montre à l'écran (40-8b).
   Jamais écrite en base, dans l'audit, dans un `.keshbackup`, ni dans un message d'erreur. Manuel d'administration
   (40-8b) : les droits *Read* à donner.
2. **AC-2 — Client.** Module `crates/kesh-api/src/stripe/` sur le patron de la source des cours de l'AFC
   (`crates/kesh-api/src/exchange_rates/source.rs` : trait `RateSource` `:97`, `HttpRateSource` `:110` avec son
   `reqwest::Client` construit `:122`, source simulée `MockRateSource` `:352`) : trait `StripeSource` (`list_payouts`,
   `list_payout_transactions`), `HttpStripeSource` (base `https://api.stripe.com`, authentification `Bearer <clé>`,
   en-tête `Stripe-Version` **épinglé** en constante, délai de réponse borné), `MockStripeSource`. Appels :
   - `GET /v1/payouts?created[gte]=…&created[lt]=…&status=paid&limit=100` puis `starting_after=<dernier id>` tant que
     `has_more` ; seuls les versements `automatic = true` sont gardés (les autres → compte rendu, « versement manuel à
     examiner ») ;
   - pour chaque versement, `GET /v1/balance_transactions?payout=<po_…>&limit=100` (même pagination), dont Stripe dit
     qu'il rend, « *for automatic Stripe payouts only* », les transactions payées par ce versement.
3. **AC-3 — Traduction en lignes du rapport.** Fonction pure : transaction de solde → `StripePayoutRow` (40-7a AC-3) :
   `payout_id`, `payout_date` = `arrival_date` du versement (date UTC), `balance_transaction_id` = `id`, `date` =
   `created` (date UTC), `available_on`, `currency` en majuscules, `gross` = `amount`, `fee`, `net` convertis des **plus
   petites unités** en unités selon l'exposant de la devise (table des devises à zéro décimale de Stripe, en constante,
   citée), `category` = `reporting_category`, `description`, `source_id` = `source` (identifiant), `charge_id` vide.
   La transaction du versement lui-même (`type = payout`) n'est pas rendue comme ligne : son montant sert au contrôle
   « Σ net = montant du versement » (écart `PAYOUT_TOTAL_MISMATCH` de 40-7c, sans attendre l'import). Le même contrôle de
   cohérence que le lecteur (net = brut − frais) s'applique. Les lignes rendues passent ensuite par **le même chemin**
   que le CSV (40-7b AC-7) : CSV et téléchargement se mélangent, doublons par `balance_transaction_id`.
4. **AC-4 — Erreurs.** 401 → `STRIPE_KEY_INVALID` ; 403 (droits) → `STRIPE_KEY_FORBIDDEN` (`details: { resource }`) ;
   429 → rendu à l'appelant avec l'en-tête `Stripe-Rate-Limited-Reason` (la reprise est l'affaire de 40-8b) ; 5xx et
   délai → `STRIPE_UNAVAILABLE` ; réponse illisible → `STRIPE_RESPONSE_INVALID` (sans la réponse au message). Codes aux
   quatre locales ; aucun ne contient la clé.
5. **AC-5 — Tests sans réseau.** Unitaires de la traduction (devise à deux décimales, à zéro décimale, remboursement à
   frais négatif) et de la configuration (paire incomplète, `sk_`, gabarit) ; `MockStripeSource` rejoue des réponses
   **fictives** versionnées (`crates/kesh-api/tests/fixtures/stripe-api/*.json`, identifiants `TEST`), dont une
   pagination à deux pages et un 429. Les lignes obtenues des réponses fictives égalent celles de la fixture CSV de 40-7a
   pour les mêmes versements (test d'égalité). Aucun appel à `api.stripe.com` en CI.

## Tâches

- [ ] T1 — Configuration et masquage (AC-1).
- [ ] T2 — Trait, client HTTP, source simulée, pagination (AC-2).
- [ ] T3 — Traduction en `StripePayoutRow`, exposants de devise (AC-3).
- [ ] T4 — Erreurs, i18n, tests et fixtures fictives (AC-4, AC-5).

## Stratégie de tests

Unitaires et tests d'intégration sans base sur la source simulée ; gate ciblé `binary(stripe_client)` ; gate complet au
push.

## Modules touchés (3)

`kesh-api` (configuration, client), `kesh-import` (lecture seule du type `StripePayoutRow`), `kesh-i18n`.

## Note de version

*Ajouté* — Kesh peut lire les versements Stripe et leurs transactions par l'API, avec une clé restreinte en lecture posée
en variable d'environnement (`KESH_STRIPE_<n>_ACCOUNT`, `KESH_STRIPE_<n>_KEY`).

## Notes de développement

- Doc publique lue le 2026-11-20 : `GET /v1/balance_transactions` (filtres `created`, `currency`, `payout` « *for
  automatic Stripe payouts only* », `source`, `type` ; `limit` 1 à 100 ; curseurs `starting_after`/`ending_before`) ;
  `GET /v1/payouts` (filtres `arrival_date`, `created`, `destination`, `status` ∈ `pending`, `paid`, `failed`,
  `canceled` ; `limit` 1 à 100) ; clés restreintes (`rk_live_`/`rk_test_`, droits *None*/*Read*/*Write* par ressource,
  `GET` = lecture, valeur non relisible) ; montants en plus petite unité (exemple de la doc : `amount: -400` pour 4 USD).
- **Non confirmé** : noms exacts, au tableau de bord, des droits *Read* qui couvrent `balance_transactions` et `payouts`
  (à constater à la recette de Beat sur une instance jetable) ; si la liste par `payout=` inclut la transaction du
  versement lui-même ; la valeur de `Stripe-Version` à épingler ; la liste des devises à zéro décimale (à recopier de
  docs.stripe.com/currencies au développement, avec la date) ; si `arrival_date` du versement égale
  `automatic_payout_effective_at` du rapport (sinon les dates d'écriture différeraient entre CSV et API).
- Remplace l'ancienne 40-1 (`superseded`) : plus de coffre ni de connexion en base (38-1 `deferred`), la clé est en
  variable d'environnement comme le mot de passe SMTP.

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
