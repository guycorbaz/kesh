# Epic 40 — Compte Stripe : téléchargement, imputation et versements

**Cadrage : 2026-11-15** (branche `epic/40-stripe`, issue de `epic/39-wise`). Demande de Beat (contributeur),
décisions B1–B28 de `epics-38-40-cadrage.md`, qui **priment** sur ce fichier ; pour Stripe : B3, B4, B6, **B13**
(compte intermédiaire « Débiteurs Stripe » par devise), B10 (rapports joints à la main), B12, B14. Socle : l'**Epic 38**
(`epic-38-paypal.md`, validé, seize fiches `ready-for-dev`), dont les décisions E*, O*, D-P1-*, D-P2-* valent ici sans
être répétées ; l'**Epic 39** (`epic-39-wise.md`, projet) pour ses amendements additifs du socle (C-39-2), repris s'il
est développé avant. Kickoff (décomptes, validation de ce fichier, **recette sur le compte Stripe de Beat**) :
`40-0-kickoff`.

**Aucune migration** : comme pour Wise (F15), tout entre dans des colonnes existantes à domaine ouvert (E4) —
`provider = 'stripe'`, `source_format = 'PROVIDER_API'`, genres et motifs de C-38-4, `event_code` court (G8),
`bank_transactions.value_date` (colonne existante, laissée `NULL` par 38-3b), `bank_account_documents.kind`, `source`
et `mime_type` (38-10). `kesh_version_min_required` inchangé ; P5–P8 sans objet. Une fiche qui découvre un besoin de
DDL s'arrête et le dit (C-40-0).

**Numéro : 40, provisoire** (B25, Q9 de l'Epic 38) — **à confirmer par Guy** (Q-40-1). Une branche par story,
fusionnée localement dans `epic/40-stripe`, puis **une PR** vers le dépôt de Guy (remote `beat`), **après celles des
Epics 37, 38 et 39**. Pas d'issue GitHub à ce stade ; issues et PR à la fin.

**Cible** : la release de l'Epic 38 ou une suivante (aucune migration, aucun relèvement) — à Guy (Q-40-9).

## Intention

La société de Beat encaisse par Stripe en **CHF, EUR et USD** (règlement multi-devises : un solde Stripe par devise).
Kesh télécharge chaque jour (ou à la demande, B9) les **transactions de solde** de chaque devise — paiements,
remboursements, litiges, frais Stripe, versements —, les classe et les **propose** : une vente se passe **à la date de la
charge** au **compte intermédiaire « Débiteurs Stripe »** de la devise (1101 CHF, 1102 EUR, 1103 USD), produit hors TVA
au compte de la règle, TVA séparée (B6), frais au sous-compte de frais Stripe de la devise (6914 CHF, 6912 EUR, 6913
USD, B3). Le **versement** (payout) sort du compte intermédiaire vers la banque réelle (UBS 1020) et se rattache à la
ligne du relevé CAMT de la banque **comme un virement entre ses comptes** (34-11b, 38-6b). À la clôture, le solde du
compte intermédiaire est une **créance** sur Stripe, contrôlée contre le solde que rapporte Stripe. Les rapports Stripe
(CSV ou PDF) se **joignent à la main** (B10, 38-10). Rien n'est écrit d'office.

## Ce qui diffère fondamentalement de PayPal et de Wise

- **Le compte prestataire est une créance, non une liquidité.** Chez PayPal et Wise, le solde se dépense et se vire à
  l'instant : le compte du plan lié (1030–1036) est un avoir disponible. Chez Stripe, le produit d'une vente est d'abord
  `pending`, devient `available` à `available_on` (« *The date that the transaction’s net funds become available in the
  Stripe balance* », 3 jours ouvrables en Suisse au régime standard, 7 jours calendaires au début), puis **part en
  versement** vers la banque, qui arrive à `arrival_date`. Entre la charge et l'arrivée en banque, l'argent est **dû par
  Stripe** : c'est ce que tient « Débiteurs Stripe » (B13).
- **Comment le socle le porte, sans rien y ajouter de structurel** (G1) : le compte bancaire Kesh « Stripe CHF » est un
  compte `bank` sans IBAN portant sa ligne `payment_provider_accounts` (38-2, E11), **lié au compte d'actif 1101**
  — la garde d'E11 (« compte du plan lié actif, de bilan (actif), dans la devise du solde ») l'admet tel quel ; compte de
  frais 6914. La colonne FK « compte intermédiaire » qu'E24 réservait à Stripe **n'est pas créée** : le compte
  intermédiaire **est** le compte du plan lié. Les transactions de solde Stripe sont des `bank_transactions` du compte
  (38-3a) ; leur somme est le solde Stripe `available + pending`, comparé au solde de 1101 (E20).
- **Le versement est le seul flux vers la banque**, et il a **deux dates** : la sortie du solde Stripe (`created` de la
  transaction de solde) et l'arrivée en banque (`arrival_date` du payout). La seconde porte l'écriture (G10), la première
  le mouvement du compte Stripe ; l'écart entre les deux, à une date de contrôle, est un **versement en route** (G12).
- **Pas de conversion interne comme mouvement** : un paiement présenté dans une devise et réglé dans une autre est
  converti à la charge (`exchange_rate` de la transaction de solde) ; Kesh ne voit que le montant dans la devise de
  règlement, frais de conversion compris dans `fee` (G19).
- **Les frais sont dans la transaction** (`fee`, `fee_details`), comme PayPal (E7) ; s'y ajoutent des **frais de
  service** à part (`stripe_fee`, `tax_fee`, `stripe_fx_fee`), passés comme mouvements de frais (38-6a FR-38-6a-2).

## Sources (docs.stripe.com, consultées le 2026-11-15)

- Transaction de solde : <https://docs.stripe.com/api/balance_transactions/object> (et `?query=fee_details`) —
  `amount` (« *Gross amount* », plus petite unité, signé), `fee` (« *Represented as a positive integer when assessed* »),
  `net` (« *`amount` - `fee`* »), `fee_details[] { amount, currency, type ∈ application_fee,
  payment_method_passthrough_fee, stripe_fee, tax, withheld_tax }`, `created`, `available_on`, `status`
  (`available`|`pending`), `exchange_rate`, `balance_type` (`payments`, `issuing`, `refund_and_dispute_prefunding`,
  `risk_reserved`), `reporting_category`, `source` (expansible), `type` (≈ cinquante valeurs : `charge`, `payment`,
  `refund`, `payment_refund`, `adjustment`, `payout`, `payout_cancel`, `payout_failure`, `stripe_fee`, `stripe_fx_fee`,
  `tax_fee`, `reserve_*`, `topup`, `transfer`, …) ; aucun type `dispute` : un litige est un `adjustment` dont la source
  est un litige (`dp_…`).
- Liste : <https://docs.stripe.com/api/balance_transactions/list> — filtres `created`, `currency`, `payout`, `source`,
  `type` ; `limit` 1 à 100 ; curseurs `starting_after` / `ending_before`, `has_more` ; « *most recent transactions
  appearing first* ».
- Solde : <https://docs.stripe.com/api/balance/balance_object> — `available[]`, `pending[]` par devise (`amount`,
  `currency`, `source_types`), `connect_reserved`, `livemode` ; **aucun solde à une date passée**.
- Versement : <https://docs.stripe.com/api/payouts/object> — `arrival_date` (« *Date that you can expect the payout to
  arrive in the bank* »), `automatic`, `balance_transaction`, `failure_balance_transaction`, `status` (`paid`,
  `pending`, `in_transit`, `canceled`, `failed` ; « *Some payouts that fail might initially show as `paid`, then change
  to `failed`* »), `reconciliation_status`, `trace_id`, `destination`.
- Types : <https://docs.stripe.com/reports/balance-transaction-types> ; catégories :
  <https://docs.stripe.com/reports/reporting-categories> (`charge`+`payment` → `charge`, `refund`+`payment_refund` →
  `refund`, `payout_cancel`+`payout_failure` → `payout_reversal`, `stripe_fee` → `fee` ; `adjustment` ventilé en
  litige, reprise de litige, échec de remboursement).
- Soldes et délais : <https://docs.stripe.com/payments/balances> — Suisse : 7 jours calendaires au début, puis 3 jours
  ouvrables ; solde négatif : débits automatiques du compte bancaire externe.
- Multi-devises : <https://docs.stripe.com/payouts/multicurrency-settlement> — disponible en Suisse ; « *Stripe sendet
  Auszahlungen von jedem Multi-Währungs-Guthaben an das Bankkonto mit der entsprechenden Währung* » (Q-40-3).
- Clés restreintes : <https://docs.stripe.com/keys/restricted-api-keys> — préfixe `rk_live_` / `rk_test_`, droits
  *None* / *Read* / *Write* par ressource, `GET` = lecture ; valeur non relisible après création.
- Expansion : <https://docs.stripe.com/api/expanding_objects> — sur une liste, `data.<champ>`, **quatre niveaux** au
  plus.
- Erreurs : <https://docs.stripe.com/api/errors> — 400, 401 (« *No valid API key* »), 402, 403 (« *doesn’t have
  permissions* »), 404, 409, 424, 429, 5xx.
- Rapports : <https://docs.stripe.com/reports/api>, <https://docs.stripe.com/reports/report-types/balance>,
  <https://docs.stripe.com/reports/report-types/payout-reconciliation> — CSV, aussi téléchargeables du tableau de bord ;
  **non retenus par l'API** (G13).
- **Non lus** : droits exacts d'une clé restreinte par ressource (noms à constater, Q-40-8) ; règles de version de l'API
  (`Stripe-Version`) au-delà de l'en-tête de l'exemple (`2026-10-28.endive`) ; table des devises à zéro décimale.

## Réponses réelles observées (compte de test de Beat, 2026-11-15)

Relevées par l'orchestrateur en **mode test** (aucun argent réel ; `livemode: false`), hors dépôt :
`data-not-committed/stripe-test-samples/` (`balance.json`, `balance_transactions.json` avec `expand data.source`,
`payouts.json`, `charges.json`, `refunds.json`, `disputes.json`). Compte suisse, devise par défaut USD ; six paiements,
un remboursement partiel, un litige, un versement USD de 300.00. Constats, champ par champ :

- **Trois soldes réglés sans conversion** : transactions en `usd`, `eur`, `chf`, toutes `exchange_rate: null` ;
  `balance.available[]` et `pending[]` portent les trois devises (plus `refund_and_dispute_prefunding`, nul).
- **Identité du solde vérifiée** (G5) : Σ `net` des transactions = `available + pending`, devise par devise — USD
  `−30000 + 58020 − 1451 + 3840 − 5000 + 2872 + 11580 = 39861 = 36989 + 2872` ; EUR `7715` ; CHF `4807`.
- **Vente** (`type: charge`, `reporting_category: charge`) : `amount 12000`, `fee 420`, `net 11580`, `fee_details
  [{type: stripe_fee, amount: 420, currency: usd}]` ; `status` `available` ou `pending` (`available_on` = `created`
  + 7 jours pour la charge en `pending` — régime initial) ; source `ch_…` avec `description` (« Order WEB-1004 »),
  `metadata.invoice` (« WEB-1004 »), `payment_intent`, `calculated_statement_descriptor` ; `billing_details.email`,
  `name` et `receipt_email` **nuls** dans ces paiements de test.
- **Remboursement partiel** (`type: refund`) : `amount −1500`, **`fee −49`** (Stripe rend une part du frais), `net
  −1451` ; source `re_…` avec `charge` (`ch_…` d'origine) — le lien vers la charge se lit **sans** expansion de second
  niveau, la transaction de solde de la charge se retrouve par `source = ch_…` dans les transactions lues.
- **Litige** (`type: adjustment`, `reporting_category: dispute`) : `amount −3000`, `fee 2000` (`fee_details[].description`
  « Dispute fee », `type: stripe_fee`), `net −5000` ; source **`du_…`** (et non `dp_…`) avec `charge`, `reason`,
  `status: needs_response`, `balance_transactions[]`.
- **Versement** (`type: payout`, `reporting_category: payout`) : `amount −30000`, `fee 0`, `net −30000`, `created` =
  `available_on` ; source `po_…` : `amount 30000` (positif), `arrival_date 1794787200` (**minuit UTC** : une date, non un
  instant), `automatic: false`, `status: paid`, `reconciliation_status: not_applicable`, `destination ba_…`,
  `trace_id { status: supported, value }`.
- `balance_type: payments` sur toutes ; `has_more: false` (une page) ; `charges.json` rendu **vide** par `GET
  /v1/charges` alors que les charges existent (à comprendre à la recette : filtre, droit ou version).

**Non observé** (reste à la recette en mode réel, 40-0) : conversion au règlement (`exchange_rate` non nul) ; versement
**automatique** (`reconciliation_status`, contenu par `payout`) ; versements dans **plusieurs devises** et leurs
destinations ; échec ou annulation de versement ; reprise de litige (`dispute_reversal`) ; réserves ; frais de service
`stripe_fee`/`tax_fee` à part ; `fee_details` de type `tax` ; pagination réelle (`has_more: true`) ; droits d'une clé
restreinte ; version d'API réellement servie.

## Dépendances

L'Epic 40 **ne se développe qu'après l'Epic 38** (développé et fusionné), donc après l'Epic 37.

### Réutilisé tel quel (Epic 38)

| Socle | Story 38 | Usage par Stripe |
|---|---|---|
| Coffre, `seal`/`open`, `KESH_SECRETS_KEY`, secrets effacés à la restauration | 38-1 | clé restreinte (`restrictedKey`) |
| Connexions, `settings JSON` typé (E24), routes C-38-2 | 38-1 | `provider = 'stripe'`, `StripeSettings` |
| Compte prestataire `payment_provider_accounts`, gardes, levée de C-37-9 | 38-2 | trois comptes `bank` liés à 1101–1103, frais 6912–6914 (G1) |
| Schéma des mouvements, passages, soldes vus et instantanés, en-tête `PROVIDER_API` | 38-3a | inchangé ; `value_date` déjà en colonne |
| `Connector`, `validate_page`, `FakeConnector`, `Transport`, fabrique, passage, planification, complétude (E20 a) | 38-3b | `StripeConnector` derrière la même fabrique |
| Critères des règles (`counterparty_email_exact`, `invoice_prefix`, `event_code_prefix`, filtres) | 38-4 | courriel du payeur, boutique (G14), code court (G8) |
| TVA par défaut du compte, impôt sur les acquisitions | 38-5 | ventes des boutiques (TVA due) |
| Propositions vente, achat, frais ; conversions, retraits, remboursements, file | 38-6a, 38-6b | versement = `withdrawal` (G10) |
| Rattachement en devise, en lot, à un groupe ; API des scripts | 38-6c1, 38-6c2 | livres 2024 (B17, B28), scripts WHMCS et CB Paid Subscriptions (B21) |
| Écrans génériques | 38-9a, 38-9b | complétés par 40-6 |
| Pièces de compte, écart au solde, contrôle de clôture K9 | 38-10 | rapports CSV ou PDF (G13) |

### Repris de l'Epic 39 s'il est développé avant (C-39-2)

Secrets nommés dans `ConnectorContext` et **liste des secrets exigés par prestataire** (F2) : Stripe n'a qu'un secret,
qui s'y inscrit (`restricted_key`). Si 39 n'est pas développé, 40-1 porte le seul ajout requis (la liste des secrets
exigés), dans la forme de C-39-2, pour que 39 n'ait rien à refaire. Rien d'autre de 39 n'est requis (ni SCA, ni PDF par
l'API).

### Ajouté (sans migration)

- `kesh-providers::stripe` : client (clé restreinte, version épinglée, contrôle `livemode`), transactions de solde
  (curseur, expansion), soldes, classement, montants.
- `kesh-core::providers::settings` : `Provider::Stripe`, `StripeSettings`.
- Amendements **additifs** du socle (C-40-2) : `ProviderMovement.value_date` écrit dans `bank_transactions.value_date` ;
  date de l'écriture d'un retrait = `value_date` (38-6b) ; « versements en route » dans la ventilation de l'écart (E20 b) ;
  pièce CSV admise par 38-10 (`kind = 'provider_report'`) ; variante `stripeResponses` de la route `_test`.

## Décisions (G1…)

- **G1 — Compte intermédiaire = compte du plan lié du compte prestataire.** Un compte bancaire `bank` sans IBAN par
  solde de devise (B1, E11), lié à « Débiteurs Stripe » de la devise (1101–1103, actif de bilan), compte de frais
  6912–6914. Aucune colonne FK neuve (E24 l'envisageait) : *raison* — le compte intermédiaire porte exactement ce que
  porte le compte du plan d'un compte PayPal (le solde du prestataire) ; une seconde référence dirait deux fois la même
  chose et ouvrirait un désaccord possible. La nature « créance » se lit au plan comptable (classe 11), non dans Kesh.
  ⚠️ Les écrans et rapports qui traitent un compte bancaire lié comme une **liquidité** (graphiques de l'accueil, 33-3 ;
  flux de trésorerie s'il existe) se recensent à la fiche 40-4 : la créance Stripe ne doit pas y être comptée comme
  trésorerie sans le dire.
- **G2 — Vente à la date de la charge, au compte intermédiaire.** `booked_at` = `created` de la transaction de solde
  (date à Zurich, 38-3b) ; la proposition est celle de 38-6a, **inchangée** : compte prestataire (1101) au net,
  contrepartie de la règle au brut hors TVA, TVA du compte (E13), frais au compte de frais de la devise. `pending` et
  `available` ne se distinguent pas en comptabilité (les deux sont la créance) ; `available_on` n'est pas stocké.
  *Raison* : B13 (« la vente est passée à la date de la charge ») ; la date de disponibilité ne change pas le débiteur.
- **G3 — Source : l'API *Balance Transactions*, par fenêtre de `created`, page unique.** `GET /v1/balance_transactions`
  avec `created[gte]`, `created[lt]`, `limit=100`, `expand[]=data.source`, curseur `starting_after` suivi **dans le
  connecteur** jusqu'à `has_more = false` ; le connecteur rend la fenêtre entière en **une** `MovementPage`
  (`total_pages = 1`, patron PayPal 38-7 et Wise F11) ; au-delà de `max_movements_per_page()` (10 000) ou de 100 requêtes
  par fenêtre → `WindowTooLarge`, la fenêtre se redécoupe dans le connecteur (O10). Pas de filtre `currency` : une
  lecture sert les trois soldes (la devise d'une transaction désigne le compte). `max_window()` 31 jours ;
  `history_start` `None` sauf constat (Q-40-7) ; `overlap()` = `overlapDays`, **défaut 3** (1–30). *Raison* : la
  transaction de solde est la seule source qui dit le mouvement **net** du solde, ses frais et sa date ; les objets
  charge, remboursement ou versement ne servent qu'à l'enrichir (expansion).
- **G4 — Clé restreinte en lecture seule, et rien d'autre.** Le secret est une clé restreinte : `rk_live_…` **et**
  `rk_test_…` sont admises (décision P40-1 : le compte de test de Beat sert à la recette) ; `PROVIDER_SECRET_NOT_RESTRICTED`
  (400) ne vise que `sk_…` (elle peut tout faire sur le compte), `pk_…` et les préfixes inconnus. Le **mode**
  (`livemode`) est lu de la clé, montré à l'écran, posé aux réglages par le premier « Tester » réussi ou le premier
  passage, puis **gardé** : une clé de l'autre mode est refusée tant que la connexion a des transactions téléchargées
  (FR-40-1-7, `PROVIDER_MODE_CHANGED`) ; tout objet de réponse dont `livemode` diffère du mode de la clé →
  `InvalidResponse { reason: "livemode" }`. Kesh **ne peut pas** vérifier qu'une clé restreinte n'a pas de droit
  d'écriture (aucune API ne le dit) : le manuel liste les droits **en lecture** à donner (Q-40-8) et dit de n'en donner aucun autre.
  **`Stripe-Version` épinglée** : constante du connecteur, envoyée à chaque requête, celle des réponses enregistrées
  (fixtures) ; changer de version est une story (le classement en dépend). *Raison* : B11, B12, « accès en lecture
  seule » ; une version flottante changerait la forme des réponses sans que rien ne rougisse.
- **G5 — Soldes : `available + pending` par devise, solde passé par sommation.** `balances(None)` = `GET /v1/balance`,
  par devise `available.amount + pending.amount` (`connect_reserved` ignoré : pas de Connect) ; `balances(Some(as_of))`
  = solde courant − Σ `net` des transactions de solde `created ≥ as_of` (lues par le connecteur, au plus 50 requêtes,
  sinon `opening` non disponible comme 38-3b AC-7) ; `as_of` rendu = `as_of` demandé. Une devise du solde sans compte
  Kesh → soldes vus (E12, B2). `refund_and_dispute_prefunding` ignoré (nul, observé). *Raison* : Stripe ne sert aucun
  solde passé ; le rapport `balance.summary` le ferait, mais par une exécution asynchrone et un fichier (G13) ; l'identité
  Σ `net` = `available + pending` est **vérifiée** sur les réponses de test, dans les trois devises (§ *Réponses réelles*).
- **G6 — Identifiants.** `provider_tx_id` = `id` de la transaction de solde (`txn_…`, unique, ASCII, ≤ 64) — non la
  source, qui peut porter plusieurs transactions (un litige et sa reprise). `reference` = identifiant de la source
  (`ch_…`, `py_…`, `re_…`, `po_…`, `du_…` observé, `dp_…` documenté). `related_provider_tx_id` d'un remboursement = la
  transaction de solde de la charge d'origine, retrouvée par la `charge` de la source `re_…` (observée sans expansion
  de second niveau) : parmi les transactions lues, celle dont la source est ce `ch_…`, sinon par
  `expand[]=data.source.charge` (`balance_transaction` de la charge) quand l'origine est hors de la fenêtre ; d'un
  litige, de même par la `charge` de `du_…` ; d'un échec de versement = la transaction du versement d'origine.
- **G7 — Montants.** Entiers en plus petite unité → décimal par l'exposant de la devise (table constante ; CHF, EUR,
  USD : 2 ; devise à zéro décimale admise ; devise à trois décimales ou inconnue de la table → `InvalidResponse`) ;
  `gross = amount`, `fee = −fee`, `net = net` ; **`fee` de Stripe est signé** : positif quand il est prélevé (vente :
  `fee 420` → frais Kesh `−4.20`), **négatif quand Stripe le rend** (remboursement partiel observé : `amount −1500`,
  `fee −49`, `net −1451` → frais Kesh `+0.49`, les « frais rendus » de 38-6b) ; contrôle `net == amount − fee` et
  `Σ fee_details.amount == fee`,
  `fee_details[].currency` = devise de la transaction, sinon `InvalidResponse`. Convention du socle `net = gross + fee`
  tenue (38-3b AC-2). Les frais de conversion de présentation sont dans `fee` (G19).
- **G8 — `event_code` court** (`VARCHAR(16)`, 38-3a ; `payment_failure_refund` en a 22) : table constante, ≤ 16,
  majuscules — `CHARGE`, `PAYMENT`, `REFUND`, `PAY_REFUND`, `REFUND_FAIL`, `PAY_FAIL_REFUND`, `PAY_REVERSAL`, `DISPUTE`,
  `DISPUTE_REV`, `ADJUSTMENT`, `PAYOUT`, `PAYOUT_CANCEL`, `PAYOUT_FAIL`, `STRIPE_FEE`, `FX_FEE`, `TAX_FEE`, `RESERVE`,
  `TOPUP`, `TOPUP_REV`, `CONNECT`, `ISSUING`, `OTHER` ; type inconnu → `OTHER`. Un `adjustment` prend `DISPUTE` ou
  `DISPUTE_REV` selon `reporting_category`. Listés au manuel (aide du critère `event_code_prefix`, 38-4).
- **G9 — Classement** (40-2, table constante, pure) :

  | Type (catégorie) | Genre | Détail |
  |---|---|---|
  | `charge`, `payment` (montant > 0) | `sale` | courriel, nom, boutique de la source (G14) ; montant ≤ 0 → `other` |
  | `refund`, `payment_refund` | `refund` | relié à la charge d'origine (G6) ; origine absente ou non réconciliée → file (38-6b, `refund_origin_missing` calculé) |
  | `adjustment` (`dispute`, `dispute_reversal`) | `dispute` | file « à examiner », aucune proposition (B7) ; frais du litige dans la même transaction (observé : `amount −3000`, `fee 2000` « Dispute fee », `net −5000`) |
  | `payout` | `withdrawal` | `value_date` = date UTC de `arrival_date` (G10) |
  | `stripe_fee`, `stripe_fx_fee`, `tax_fee` | `fee` | compte de frais de la devise (38-6a) |
  | `topup` | `deposit` | rattaché au relevé de la banque (34-11b) |
  | `payout_cancel`, `payout_failure`, `refund_failure`, `payment_failure_refund`, `payment_reversal`, `topup_reversal`, `adjustment` (autre), `reserve_*`, `reserved_funds`, `payout_minimum_balance_*`, Connect, Issuing, Climate, tout type inconnu | `other` | `unknown_event`, file « à examiner » |

  **Aucun type inconnu ne devient une vente** (patron NFR-38-8-1). Une transaction de solde dont `balance_type` n'est
  pas `payments` (ni absent) n'est **pas un mouvement du compte** : comptée `skipped.non_balance`, non écrite (elle ne
  change pas `available + pending`, G5). Surcharges `eventKindOverrides` (patron 38-8 AC-7), clés = codes courts, sur les
  seuls codes classés `other`. La table se confirme type par type sur les réponses de Beat (40-0).
- **G10 — Le versement : un virement vers la banque, daté de son arrivée.** Un `payout` est un `withdrawal` de 38-6b
  (C-38-6b-3), **inchangé dans ses deux voies** — rattacher l'écriture passée côté banque (voie a), ou créer
  `D 1020 / C 1101` et laisser la ligne UBS s'y rattacher (voie b, 34-11b) —, **à une différence** : le mouvement porte
  `value_date` = `arrival_date` du payout, et **l'écriture créée (voie b) est datée de `value_date`** quand elle est
  posée, la fenêtre de suggestion du compte de l'autre côté partant de `value_date` (au lieu de `booking_date`). Le
  mouvement Stripe garde `booked_at` = `created` (il quitte le solde Stripe ce jour-là, ce qui tient la complétude E20 a).
  Observé : transaction `payout` `amount −30000`, `created 1794710640` ; source `po_…` `amount 30000`, `arrival_date
  1794787200` — **minuit UTC**, donc une **date** lue en UTC (non convertie à Zurich comme un instant) ; `status: paid`,
  `automatic: false`. Contrôle : `amount` du versement = `−amount` de sa transaction, même devise, sinon
  `InvalidResponse`.
  *Raison* : la créance sur Stripe (B13) s'éteint quand l'argent est en banque ; dater l'écriture de la sortie Stripe
  gonflerait UBS au 31 décembre d'un montant que son relevé n'a pas (contrôle K3 faux), la dater de l'arrivée laisse le
  montant en route dans « Débiteurs Stripe », ce que B13 appelle créance. Deux écritures par un compte de transit (1090)
  sont écartées, comme en 34-11b (C-e341-22) : double saisie sans gain. *Contestable par Beat* (Q-40-4).
- **G11 — Versement échoué ou annulé : à examiner.** `payout_failure` / `payout_cancel` rend l'argent au solde Stripe ;
  si le versement avait déjà son écriture, la défaire est un geste humain (annulation du rapprochement, 34-11b) : genre
  `other`, motif `unknown_event`, `related_provider_tx_id` = le versement d'origine (G6). Un versement dit `paid` qui
  passe `failed` (doc) se voit à la transaction d'échec.
- **G12 — Contrôle de solde : créance, en route, inexpliqué.** Le contrôle E20 (b) — solde du compte du plan en devise à
  `as_of` contre le solde rapporté (`available + pending`) — gagne une ligne de ventilation **« versements en route »** :
  versement compté au solde rapporté (`booked_at < as_of`) dont l'écriture rapprochée est datée après la date de
  Zurich de `as_of` ; montant = ligne de l'écriture sur le compte lié (40-4 Écart 1) — en plus des « transactions non
  rapprochées » et du « reste inexpliqué ». L'écran du compte montre aussi
  `available` et `pending` (lecture de `balances(None)` à « Tester » et au passage, non stockée : l'instantané garde le
  total, 38-3a). **À la clôture**, le solde de 1101–1103 est la créance (B13) : aucune écriture neuve ; le contrôle
  préalable à la clôture (K9 de 38-10) compare le rapport joint (G13) au solde comptable.
- **G13 — Rapports Stripe joints à la main** (B10). Stripe ne fournit aucun relevé PDF ; ses rapports (solde,
  rapprochement des versements) sont des **CSV** du tableau de bord. 38-10 admet donc, **en plus du PDF**, un CSV
  (`kind = 'provider_report'`, `mime_type = 'text/csv'`, colonnes existantes à domaine ouvert, C-38-10-6) : texte UTF-8,
  première ligne d'en-tête, borne de taille de 38-10 ; aucun contenu lu ni interprété ; solde de clôture **saisi** comme
  pour un PDF. `GET …/documents/{docId}` rend le `mime_type` stocké. L'**API des rapports** (exécution asynchrone, fichier
  sur `files.stripe.com`, droits de plus) **n'est pas retenue** : tracée (C-40-7).
- **G14 — Boutique d'une vente Stripe : un réglage dit d'où la lire.** La transaction de solde ne porte ni courriel ni
  numéro de facture ; la source développée (charge ou paiement) porte `description`, `metadata`,
  `billing_details.email`, `receipt_email`. `StripeSettings.invoiceRefSource` ∈ `description` | `metadata:<clé>`
  (défaut **`metadata:invoice`**) dit où lire `invoice_ref` (critère `invoice_prefix`, B24 transposé) ; clé absente →
  `invoice_ref` `NULL` (aucune règle de boutique ne s'applique, jamais de repli silencieux sur `description`) ;
  `counterparty_email` = `billing_details.email`, sinon `receipt_email` ; `counterparty_name` = `billing_details.name` ;
  `details` = `description` ; `item_names` `NULL`. Observé : `metadata.invoice` « WEB-1004 », « SHOP-2001 » (deux
  boutiques, deux préfixes) quand `description` vaut « Order WEB-1004 » — le préfixe de `description` ne désignerait pas
  la boutique ; courriel et nom **nuls** dans les paiements de test (un critère `counterparty_email_exact` n'y aurait
  prise sur rien). *Raison* : WHMCS et CB Paid Subscriptions (B21) posent leur numéro où ils le posent — les paiements
  de test ne sont pas les leurs ; le réglage évite de deviner (Q-40-2, à constater en 40-0 sur des paiements réels).
- **G15 — Aucune migration** (C-40-0).
- **G16 — Tests sans réseau** (B12, C-38-11) : réponses Stripe enregistrées et **anonymisées** sous
  `crates/kesh-providers/tests/fixtures/stripe/` (courriels, noms, identifiants `cus_`, `ch_`, `po_` remplacés ;
  montants, types et catégories gardés) ; variante additive `stripeResponses` de `POST /api/v1/_test/provider-connector`
  (patron `paypalResponses`, chaîne `"default"` comprise).
- **G17 — Aucun outil MCP neuf** (E23).
- **G18 — Amendements du socle : additifs, non régressifs.** PayPal (et Wise s'il est livré) inchangés : leurs suites
  restent vertes sans retouche de leurs attentes ; `value_date` reste `NULL` pour eux.
- **G19 — Conversions.** Le règlement multi-devises garde un solde par devise (observé : trois soldes, `exchange_rate`
  nul partout ; reste à constater en mode réel, Q-40-3) ; un paiement présenté dans une
  autre devise est converti par Stripe **à la charge** : Kesh passe le montant de la devise de règlement, la TVA tirée de
  ce brut (B20 : tels que rapportés ; l'écart d'arrondi avec la TVA calculée par la boutique dans la devise présentée est
  assumé et dit au manuel). Aucune paire de conversion ; une conversion entre soldes Stripe (si Beat en fait) arrive en
  `other` (Q-40-11).

## Registre des contrats

- **C-40-0 — Aucune migration** (toutes) : G15. Une fiche qui découvre un DDL, une valeur neuve dans une colonne à
  domaine fermé, un `MODIFY`/`CHANGE COLUMN` ou un `DROP` s'arrête et le dit.
- **C-40-1 — Connexion Stripe** (40-1) : `provider = 'stripe'` ; `StripeSettings` `#[serde(deny_unknown_fields,
  rename_all = "camelCase")]` : modifiables par le client — `overlapDays` (1–30, défaut 3), `invoiceRefSource` (G14),
  `eventKindOverrides` (40-2) ; posés par le serveur — `accountId` (`acct_…`, si la clé peut le lire, Q-40-8),
  `apiVersion` (lecture seule, égale à la constante, G4), `livemode` (mode de la connexion, posé par le premier
  « Tester » réussi ou le premier passage, puis gardé, G4). Secret `restrictedKey` (`PUT …/secrets`, corps propre au
  prestataire, 38-1 AC-6) ; préfixes de G4. URL de base constante `https://api.stripe.com`.
- **C-40-2 — Amendements additifs du socle** : (a) `ProviderMovement.value_date: Option<NaiveDate>` (date seule, déjà
  locale), validé par `validate_page` (`value_date ≥ date Zurich de booked_at − 1 jour`, 40-1 Écart 5 ; ≤ + 30 jours), écrit dans
  `bank_transactions.value_date` par le passage (38-3b), `NULL` ailleurs (40-1 pour le type, 40-3 pour l'usage) ;
  (b) 38-6b, retrait : date de l'écriture créée et départ de la fenêtre de suggestion = `value_date` si posée (40-3) ;
  (c) ventilation de l'écart E20 (b) : `inTransitPayouts` (40-4) — versement compté au solde rapporté (`booked_at <
  as_of`) dont l'écriture rapprochée est datée après la date de Zurich de `as_of` ; montant = ligne de l'écriture sur
  le compte lié (40-4 Écart 1) ; (d) 38-10 : CSV admis, `kind = 'provider_report'`
  (40-5) ; (e) `stripeResponses` (40-1) ; (f) secrets exigés par prestataire si 39 n'est pas livré (40-1).
- **C-40-3 — Erreurs** : table de 38-7 AC-7 — 401 → `Auth` ; 403 → `Permission` ; 400, 402, 404 → `Rejected` ; 429 →
  `RateLimited` ; 5xx, 424, délai → `Unavailable` ; `livemode` d'un objet ≠ mode de la clé → `InvalidResponse` ; code
  d'API neuf `PROVIDER_SECRET_NOT_RESTRICTED` (400, dépôt d'une clé `sk_…`, `pk_…` ou de préfixe inconnu ; `rk_live_` et
  `rk_test_` admises, G4) ; `PROVIDER_MODE_CHANGED` (chaîne de `errorCode`/`lastError` : clé de l'autre mode que la
  connexion, FR-40-1-7) ; `PROVIDER_ACCOUNT_CHANGED` (même forme : `accountId` lu par « Tester » ≠ celui des réglages
  sur une connexion déjà téléchargée, D40-P2-1, C-40-7). Aucun texte de
  Stripe à l'écran ni dans `last_error`.
- **C-40-4 — Mouvement Stripe** (40-1 lecture, 40-2 correspondance) : G6 à G9, G14 ; correspondance vers
  `ProviderMovement` en **une** fonction pure.
- **C-40-5 — Versement** (40-3) : G10, G11 ; `transferBankAccountId` suggéré dans `[value_date, value_date + 5 jours]`.
- **C-40-6 — Registres** : `LIB_ROUTES` inchangé (aucune route neuve) ; `TEST_ENDPOINT_ROUTES` inchangé (variante de la
  même route) ; audit : `payment_provider_connection.secrets_updated` (existant) ; pièce CSV par
  `bank_account_document.attached` (38-10), détail `{ kind, mimeType }`.
- **C-40-7 — Tracé pour plus tard** (issues avec la PR) : rapports par l'API (`balance.summary`,
  `payout_reconciliation.by_id`) ; contenu d'un versement (transactions de solde par `payout`) à l'écran ; impôt sur les
  acquisitions des frais Stripe et PayPal (Q-40-5) ; montant et devise de présentation d'un paiement converti (non
  gardés) ; réserves et soldes `risk_reserved` (Q-40-6) ; changement de `Stripe-Version`. **Changement de compte Stripe
  — décidé (D40-P2-1, orchestrateur, 2026-11-15)** : `accountId` lu par « Tester » ≠ celui des réglages sur une
  connexion **déjà téléchargée** → refusé, contrôle `account_unchanged` en échec, code `PROVIDER_ACCOUNT_CHANGED`
  (patron de `PROVIDER_MODE_CHANGED`), réglages inchangés ; sans téléchargement → réécrit et tracé (`accountIdChanged`)
  (40-1 AC-9). Fermé (D40-P3-1) : le passage lit aussi le compte à chaque passage et applique la même règle ; une clé sans droit de lecture du compte est refusée (`PROVIDER_ACCOUNT_UNREADABLE`, Q-40-8).

## Stories

| Story | Objet | Modules (≤ 5) | Dépend de | Migration |
|---|---|---|---|---|
| 40-0 | kickoff : décomptes, validation de ce fichier, **recette sur le compte Stripe de Beat** (clé restreinte, droits, réponses anonymisées) | — | — | — |
| 40-1 | client Stripe : réglages, clé restreinte, version épinglée, « Tester », transactions de solde par curseur, soldes et devises du compte | `kesh-providers` (stripe), `kesh-core` (réglages), `kesh-api` (secrets, fabrique, `_test`), `kesh-i18n` | Epic 38, 40-0 | aucune |
| 40-2 | classement des types en mouvements : genres, codes courts, montants, liens, boutique, surcharges | `kesh-providers` (stripe::classify), `kesh-core` (surcharges) | 40-1 | aucune |
| 40-3 | versements : `value_date`, écriture datée de l'arrivée, rattachement à la ligne UBS, échecs | `kesh-providers` (type), `kesh-api` (passage, reconciliation), `kesh-reconciliation`, `kesh-db` | 40-2, 38-6b, 38-6c1 | aucune |
| 40-4 | contrôle de solde : `available + pending` contre Débiteurs Stripe, versements en route, créance à la clôture ; recensement des lecteurs « liquidité » | `kesh-db` (écart, en route), `kesh-api` (contrôles), `kesh-i18n` | 40-3, 38-3b, 38-10 | aucune |
| 40-5 | rapports joints à la main : CSV admis par 38-10, solde de clôture saisi | `kesh-api` (documents), `kesh-db` (lecture `mime_type`), `frontend` (bank-accounts), `kesh-i18n` | 38-10 | aucune |
| 40-6 | écrans et manuels : connexion Stripe, droits de la clé, versements en route, codes courts ; E2E et captures | `frontend`, `kesh-i18n`, `docs/manual`, E2E | 40-1 à 40-5, 38-9a, 38-9b | aucune |

**Ordre** : 40-0 (recette) → 40-1 → 40-2 → 40-3 → 40-4 → 40-6 ; 40-5 dès 38-10, en parallèle. **Migrations : aucune**
(C-40-0). Chaque fiche porte ses exigences numérotées (B26) ; les gardes communes (P1–P8, *Pattern batch*, aucun appel
réseau en CI, recette visuelle) ne s'y répètent que là où la story les porte en propre.

### 40-0 — Kickoff et recette

- *Intention* : ouvrir l'epic sur des décomptes mesurés, un fichier validé, et les réponses réelles de Stripe qui
  confirment G3 à G14.
- *Périmètre* : décomptes (§ *Priorités des défauts*) ; validation en boucle ; **recette en mode test déjà faite**
  (§ *Réponses réelles observées*) — ses réponses servent de base aux fixtures, identifiants remplacés (G16) ;
  **recette en mode réel par Beat, hors de Kesh et hors du dépôt**, avec une clé `rk_live_` en lecture seule, pour ce
  que le mode test n'a pas montré : droits minimaux d'une clé restreinte (Q-40-8), `GET /v1/balance_transactions` d'un
  mois réel (expansion `data.source`), versements **automatiques** et leurs devises et destinations (Q-40-3), champ de
  la boutique dans les paiements de WHMCS et CB Paid Subscriptions (Q-40-2), historique servi (Q-40-7), conversion au
  règlement s'il y en a ; réponses **anonymisées** ; constats écrits ici.
- *Risques* : droits d'expansion plus larges que prévu (charges, remboursements, versements, litiges) ; aucun appel
  réseau par un agent (B12).
- **FR**
  1. `FR-40-0-1` — Compter issues et bugs, écrire au fichier le compte du tag précédent et l'objectif du tag.
  2. `FR-40-0-2` — Valider ce fichier en boucle jusqu'à une passe sans finding au-dessus de LOW.
  3. `FR-40-0-3` — Consigner la recette de Beat (droits de la clé, transactions de solde, soldes, versement, boutique) et
     ses constats ; aucune fiche ne se développe sans elle.
- **NFR**
  1. `NFR-40-0-1` — Décomptes recomptés à la source, commande et périmètre écrits avec le nombre.
  2. `NFR-40-0-2` — Aucune réponse réelle non anonymisée ni aucune clé dans le dépôt.

### 40-1 — Client Stripe : connexion, clé restreinte, transactions de solde, soldes

- *Intention* : un Administrateur crée une connexion Stripe, dépose sa clé restreinte, et Kesh lit les soldes et les
  transactions de solde de chaque devise.
- *Périmètre* : `Provider::Stripe`, `StripeSettings` (C-40-1) ; corps de `PUT …/secrets` (`restrictedKey`, G4) ;
  `StripeConnector` derrière `ConnectorFactory` : `test()` (soldes par devise, `account_ref` = `accountId` si lisible),
  `balances(None | Some)` (G5), `movements(window, page)` (G3 : curseur suivi dans le connecteur, page unique,
  expansion, validation `livemode`, ordre décroissant reclassé, fenêtre respectée, curseur cohérent) ; `Stripe-Version`
  constante ; erreurs (C-40-3) ; type `value_date` (C-40-2 a, posé à `None` ici) ; `stripeResponses` et jeu
  `"default"` (G16) ; secrets exigés par prestataire (C-40-2 f) si 39 n'est pas livré.
- *Risques* : volume d'une fenêtre (100 par requête : une année de ventes en centaines de requêtes au premier passage —
  borne et redécoupage) ; `has_more` incohérent ou curseur qui boucle (refus bruyant) ; solde passé par sommation sur
  une longue période (borne de G5) ; clé `sk_` collée par erreur (refus, G4).
- **FR**
  1. `FR-40-1-1` — Créer une connexion `stripe` avec ses réglages (C-40-1), validés par `StripeSettings`.
  2. `FR-40-1-2` — Déposer la clé restreinte, jamais réaffichée ; `rk_live_` et `rk_test_` admises ; une clé `sk_…`,
     `pk_…` ou de préfixe inconnu est refusée (`PROVIDER_SECRET_NOT_RESTRICTED`) ; le mode se garde (FR-40-1-7, G4).
  3. `FR-40-1-3` — « Tester la connexion » lit les soldes `available + pending` par devise ; une devise sans compte Kesh
     est listée (B2).
  4. `FR-40-1-4` — Lire les transactions de solde d'une fenêtre de `created`, toutes les pages du curseur, en une
     `MovementPage`.
  5. `FR-40-1-5` — Lire le solde d'une devise à un instant passé, par sommation (G5).
  6. `FR-40-1-6` — Chaque erreur Stripe devient un code lisible (C-40-3) ; la fabrique construit un `StripeConnector`
     pour `stripe`, sur le transport factice `stripeResponses` en mode test.
- **NFR**
  1. `NFR-40-1-1` — URL de base constante, client borné de 37-2, bornes de 38-3b par requête, plafonds de requêtes par
     fenêtre et par lecture de solde.
  2. `NFR-40-1-2` — Clé absente de tout journal, erreur, réponse, audit, export et sauvegarde (test au marqueur).
  3. `NFR-40-1-3` — `Stripe-Version` épinglée sur chaque requête ; `livemode` contrôlé sur chaque objet qui le porte (la transaction
     de solde n'en a pas, 40-1).
  4. `NFR-40-1-4` — Validation stricte ; un test par refus, réponses hostiles comprises (C-38-11) ; PayPal inchangé.

### 40-2 — Classement des types Stripe

- *Intention* : chaque transaction de solde devient un `ProviderMovement` classé, avec brut, frais, net, liens,
  contrepartie et boutique, et ce que Kesh ne sait pas classer va à la file « à examiner ».
- *Périmètre* : G6 à G9, G14, C-40-4 ; table constante types → genres et codes courts ; `adjustment` départagé par
  `reporting_category` ; remboursement relié à la charge, échec de versement au versement ; `balance_type` hors
  `payments` compté `skipped.non_balance` ; `eventKindOverrides` Stripe.
- *Risques* : remboursement d'une charge antérieure à `fetch_from` (origine absente → file) ; frais de litige ;
  `fee_details` de type `tax` (TVA prélevée par Stripe sur ses frais ? Q-40-5) ; paiement présenté dans une autre devise
  (G19) ; source non développée (droit manquant : `InvalidResponse`, non un mouvement sans courriel).
- **FR**
  1. `FR-40-2-1` — Chaque type Stripe se classe selon la table de G9 ; un type inconnu → `other`, `unknown_event`.
  2. `FR-40-2-2` — Brut, frais et net se lisent de `amount`, `fee` et `net` à l'exposant de la devise (G7).
  3. `FR-40-2-3` — Identifiant `txn_…`, référence de la source, remboursement relié à sa charge (G6).
  4. `FR-40-2-4` — Courriel, nom, description et boutique (`invoiceRefSource`) se posent depuis la source (G14).
  5. `FR-40-2-5` — Un litige ou sa reprise va à la file « à examiner », sans proposition.
  6. `FR-40-2-6` — Un réglage de connexion peut classer un code que la table met en `other`.
- **NFR**
  1. `NFR-40-2-1` — Classement pur, sans base, réseau ni horloge.
  2. `NFR-40-2-2` — Un test par type relevé sur les réponses de Beat ; aucun type inconnu ne devient une vente.
  3. `NFR-40-2-3` — Somme des nets d'une fenêtre = variation du solde `available + pending` (test sur fixtures).

### 40-3 — Versements et rattachement à la ligne UBS

- *Intention* : un versement Stripe sort du compte intermédiaire et se rapproche de la ligne du relevé UBS comme un
  virement entre ses comptes, daté de son arrivée en banque.
- *Périmètre* : G10, G11, C-40-2 (a, b), C-40-5 ; `value_date` = `arrival_date` posée par le connecteur, écrite par le
  passage ; 38-6b : écriture du retrait datée de `value_date`, fenêtre de suggestion `[value_date, value_date + 5]` ;
  voies (a) et (b) inchangées sinon ; versements EUR et USD vers le compte de leur devise (UBS en devise, Wise, Q-40-3) ;
  rattachement en lot des versements de 2024 aux écritures existantes (38-6c1, B17).
- *Risques* : versement à cheval sur la clôture (créance au 31 décembre, G12) ; `arrival_date` prévue qui diffère de la
  date réelle du relevé (fenêtre de 5 jours, candidats sans fenêtre de date au rattachement, 34-11b) ; versement échoué
  après avoir été rapproché (G11) ; débit automatique du compte bancaire par Stripe en solde négatif (doc : genre à
  constater) ; non-régression des retraits PayPal (`value_date` nulle).
- **FR**
  1. `FR-40-3-1` — Un versement porte sa date d'arrivée (`value_date`) ; le mouvement garde sa date de sortie.
  2. `FR-40-3-2` — Sa proposition crée l'écriture banque ↔ Débiteurs Stripe datée de l'arrivée, ou rattache l'écriture
     déjà passée côté banque.
  3. `FR-40-3-3` — La ligne du relevé UBS se rattache à la même écriture (34-11b).
  4. `FR-40-3-4` — Un versement échoué ou annulé va à la file « à examiner », relié au versement d'origine.
- **NFR**
  1. `NFR-40-3-1` — *Pattern batch* ; aucune écriture d'office.
  2. `NFR-40-3-2` — Retraits sans `value_date` (PayPal, Wise) inchangés : leurs tests restent verts sans retouche.
  3. `NFR-40-3-3` — Aucune migration (`value_date` existe).

### 40-4 — Contrôle de solde et créance à la clôture

- *Intention* : le solde Stripe (`available + pending`) se compare au compte « Débiteurs Stripe » de la devise, et l'écart
  s'explique par les transactions non rapprochées et les versements en route.
- *Périmètre* : G12, C-40-2 (c) ; ventilation `inTransitPayouts` dans l'écart E20 (b) ; `available` et `pending` montrés
  à « Tester » ; contrôle préalable à la clôture inchangé (K9 de 38-10) ; recensement des lecteurs qui traitent un compte
  bancaire lié comme une liquidité (accueil, 33-3) et décision par lecteur (G1).
- *Risques* : définition unique de « versement en route » (une fonction, lue par l'écart et par l'écran) ; versement
  sans `value_date` (jugé à son écriture comme les autres, 40-4 Écart 1) ; double comptage d'un versement en route non rapproché.
- **FR**
  1. `FR-40-4-1` — L'écart entre solde comptable et solde rapporté se ventile en non rapprochées, versements en route et
     reste inexpliqué.
  2. `FR-40-4-2` — L'écran du compte et « Tester » montrent `available` et `pending`.
  3. `FR-40-4-3` — Un compte prestataire Stripe n'est pas compté comme liquidité là où Kesh additionne la trésorerie
     (ou l'est en le disant), selon le recensement.
- **NFR**
  1. `NFR-40-4-1` — Une seule définition de « versement en route », testée aux bornes (`as_of` = `booked_at`, date
     d'écriture = date de `as_of` ; 40-4, alignement P2).
  2. `NFR-40-4-2` — Contrôle sans écriture ; aucune migration.

### 40-5 — Rapports Stripe joints à la main

- *Intention* : le rapport de solde ou de rapprochement des versements, téléchargé du tableau de bord Stripe en CSV (ou
  imprimé en PDF), se range au compte et à l'exercice, et son solde de clôture sert de somme de contrôle (B20).
- *Périmètre* : G13, C-40-2 (d) ; 38-10 admet `text/csv` (`kind = 'provider_report'`) en plus du PDF ; contrôle d'un CSV
  (UTF-8, en-tête, taille) ; `GET` rend le `mime_type` stocké ; sauvegarde inchangée (le fichier est une pièce comme une
  autre).
- *Risques* : un CSV qui n'en est pas un (refus bruyant, `BANK_DOCUMENT_UNSUPPORTED_TYPE`) ; nom de fichier ; lecteurs de
  38-10 qui supposent `application/pdf` (recensés).
- **FR**
  1. `FR-40-5-1` — Joindre un rapport CSV (ou PDF) à un compte bancaire, pour un exercice ou une période.
  2. `FR-40-5-2` — Saisir son solde de clôture et sa devise ; l'écart et le contrôle de clôture de 38-10 s'appliquent.
  3. `FR-40-5-3` — Télécharger la pièce dans son type d'origine.
- **NFR**
  1. `NFR-40-5-1` — Fichier contrôlé (UTF-8, en-tête, borne de taille) ; aucun contenu interprété.
  2. `NFR-40-5-2` — Pièce auditée et sauvegardée par 38-10 ; aucune migration.

### 40-6 — Écrans et manuels

- *Intention* : ce qui diffère de 38-9a/38-9b pour Stripe se fait à l'écran : clé restreinte, boutique, versements,
  créance, rapports.
- *Périmètre* : fiche de connexion Stripe (clé en saisie seule, refus `sk_` expliqué, `invoiceRefSource`, compte et
  version lus) ; libellés des codes neufs ; codes courts de G8 en aide du critère `event_code_prefix` ; versement :
  dates de sortie et d'arrivée à la proposition ; écart ventilé (versements en route) ; rapports CSV aux *Relevés
  joints* ; manuel d'administration (créer la clé restreinte, droits en lecture à cocher et rien d'autre, restauration
  = clé à ressaisir) et utilisateur (Débiteurs Stripe, date de vente, versement daté de l'arrivée, créance à la
  clôture, rapports à joindre).
- *Risques* : clé jamais au DOM ; textes « proposer », jamais « comptabiliser automatiquement ».
- **FR**
  1. `FR-40-6-1` — Créer et régler une connexion Stripe, clé en saisie seule, refus d'une clé non restreinte lisible.
  2. `FR-40-6-2` — Choisir la source de la boutique (`invoiceRefSource`).
  3. `FR-40-6-3` — Voir, pour un versement, sa sortie et son arrivée, et sa proposition datée de l'arrivée.
  4. `FR-40-6-4` — Voir l'écart ventilé et `available` / `pending` d'un compte Stripe.
  5. `FR-40-6-5` — Joindre un rapport CSV ; codes courts Stripe en aide du critère « code d'événement ».
- **NFR**
  1. `NFR-40-6-1` — Aucun secret au DOM ni dans les réponses lues.
  2. `NFR-40-6-2` — Quatre locales ; manuels d'administration et utilisateur à jour, PDF régénérés et contrôlés aplatis.
  3. `NFR-40-6-3` — E2E contre le transport factice (`stripeResponses: "default"`) : connexion → test → téléchargement →
     vente proposée et acceptée → versement rattaché à la ligne UBS.
  4. `NFR-40-6-4` — Captures d'écran de chaque état touché (*Recette visuelle*).

## Exigences (FR/NFR) par story

Récapitulatif (B26) ; compté aux sections ci-dessus le 2026-11-15 (`grep -oE "\bFR-40-[0-9]+-[0-9]+" epic-40-stripe.md |
sort -u | wc -l`, de même pour `NFR`). **Les fiches font foi** : recompté le 2026-11-15 (alignement P1) aux fiches 40-1 à
40-6, identifiants **propres** à chaque fiche seulement (`grep -oE "\bN?FR-40-<n>-[0-9]+" 40-<n>-*.md | sort -u`, renvois
à une autre fiche exclus) ; 40-0 n'a pas de fiche, ses exigences sont celles de ce fichier. Forme définitive — issues ou
fiches — à Guy (Q12 de l'Epic 38).

| Story | FR | NFR |
|---|---|---|
| 40-0 | 3 — 1 décomptes · 2 validation · 3 recette | 2 — 1 décomptes recomptés · 2 rien de réel au dépôt |
| 40-1 | 7 — 1 connexion `stripe` · 2 clé restreinte · 3 « Tester », soldes · 4 transactions par curseur · 5 solde passé · 6 erreurs, fabrique, factice · `FR-40-1-7` mode gardé *(fiche)* | 4 — 1 bornes · 2 clé nulle part · 3 version, `livemode` · 4 validation, PayPal inchangé |
| 40-2 | 7 — 1 table des types · 2 montants · 3 identifiants, liens · 4 contrepartie, boutique · 5 litiges · 6 surcharges · `FR-40-2-7` date d'arrivée lue *(fiche)* | 3 — 1 pur · 2 test par type · 3 somme des nets |
| 40-3 | 5 — 1 date d'arrivée · 2 écriture datée de l'arrivée · 3 ligne UBS rattachée · 4 échecs · `FR-40-3-5` versements 2024 en lot *(fiche)* | 3 — 1 *Pattern batch* · 2 retraits sans date inchangés · 3 aucune migration |
| 40-4 | 4 — 1 écart ventilé · 2 `available`/`pending` · 3 liquidité · `FR-40-4-4` K9 et versements en route *(fiche)* | 3 — 1 définition unique · 2 sans écriture · `NFR-40-4-3` additif *(fiche)* |
| 40-5 | 4 — 1 joindre un CSV · 2 solde de clôture · 3 télécharger · `FR-40-5-4` type de pièce à la liste *(fiche)* | 3 — 1 fichier contrôlé · 2 audit, sauvegarde · `NFR-40-5-3` additif *(fiche)* |
| 40-6 | 5 — 1 connexion · 2 boutique · 3 versement · 4 écart · 5 CSV, codes | 5 — 1 aucun secret au DOM · 2 i18n, manuels · 3 E2E · 4 captures · `NFR-40-6-5` textes, PayPal inchangé *(fiche)* |

**Total : 35 FR et 23 NFR** sur 7 stories (40-0 comprise) — 30 et 20 au cadrage, plus les cinq FR et trois NFR
ajoutées par les fiches (*(fiche)* ci-dessus).

## Validation P1 (2026-11-15)

Décisions transverses issues de la validation P1 des fiches 40-1 à 40-6, reportées ici et aux fiches concernées :

- **P40-1 — Clés de test admises, mode gardé** (40-1 AC-4, AC-5, AC-9 ; 40-6 AC-1 à AC-3) : `rk_live_` et `rk_test_`
  admises partout ; `PROVIDER_SECRET_NOT_RESTRICTED` ne vise que `sk_`, `pk_` et les préfixes inconnus ; `livemode` lu de
  la clé, montré, posé aux réglages par le premier « Tester » ou le premier passage, puis gardé (FR-40-1-7,
  `PROVIDER_MODE_CHANGED`) ; `livemode` contrôlé sur chaque objet qui le porte. G4, C-40-1, C-40-3, FR-40-1-2 et
  NFR-40-1-3 alignés.
- **DTO des propositions** (40-3 AC-3, lu par 40-6 AC-4) : le DTO de transaction de `GET
  /api/v1/reconciliation/proposals` gagne `valueDate` (`AAAA-MM-JJ` ou `null`, additif) ; la proposition `provider` de
  retrait gagne `entryDate` (date de l'écriture proposée) ; PayPal : `valueDate: null`, `entryDate` = `bookingDate` ;
  test de clés exactes PayPal inchangées. Aucune route neuve (C-40-6 inchangé).
- **Codes surchargeables publics** (40-2 AC-6, lus par 40-6 AC-6) : constante `pub` `OVERRIDABLE_CODES` du module
  `stripe::event_codes`, lue par la validation de `eventKindOverrides` et par la garde de concordance de
  `textes_coherents.rs`.
- **Exigences ajoutées par les fiches** : `FR-40-1-7`, `FR-40-2-7`, `FR-40-3-5`, `FR-40-4-4`, `FR-40-5-4`,
  `NFR-40-4-3`, `NFR-40-5-3`, `NFR-40-6-5` — récapitulatif mis à jour (35 FR, 23 NFR).

## Validation P2 (2026-11-15)

Alignements inter-fiches issus de la validation P2 des fiches 40-1 à 40-6, reportés ici et aux fiches concernées :

- **`value_date`** (C-40-2 a) : `value_date ≥ date Zurich de booked_at − 1 jour` (40-1 Écart 5 : `arrival_date` est une
  date UTC).
- **Versement en route** (G12, C-40-2 c) : la règle `booked_at < as_of ≤ value_date` cède la place à la date de
  l'écriture rapprochée — versement compté au solde rapporté, écriture datée après la date de Zurich de `as_of`, montant
  = ligne de l'écriture sur le compte lié (40-4 Écart 1) ; non rapproché → `unreconciled`, jamais en route.
- **D40-P2-1 — Changement de compte Stripe** (C-40-3, C-40-7 ; 40-1 AC-9, AC-10, AC-14 ; 40-6 AC-3, AC-8) : refusé
  (`PROVIDER_ACCOUNT_CHANGED`, contrôle `account_unchanged`) sur une connexion déjà téléchargée, réécrit et tracé sinon.
- **Documentation de l'API** (40-1 AC-15) : `docs/api-external.md` décrit la connexion Stripe (réglages, corps du
  secret, `PROVIDER_SECRET_NOT_RESTRICTED`, `PROVIDER_MODE_CHANGED`, `PROVIDER_ACCOUNT_CHANGED`).
- **Codes publics** (40-2 AC-2, lus par 40-6 AC-6) : `EVENT_CODES` (`OTHER` compris), constante `pub` de
  `kesh_core::providers::settings::stripe_codes`, réexportée par `kesh_providers::stripe::event_codes`.
- **40-6** : badge du mode « à établir » tant que ni « Tester » ni un passage ne l'a posé ; motif `balance_parts` retiré
  (supprimé par 40-4) ; aide des échecs de versement complétée (écriture manuelle, versement et échec marqués
  examinés) ; limite « inexpliqué transitoire » de 38-9a retirée du manuel au profit des « autres décalages de date »
  (40-4 Écart 6) ; préfixe i18n `provider-connections-stripe-` (règle `keyBelongsToFeature` de `lint-i18n-ownership`,
  correction de 38-9a AC-10 sur la branche 38).
- 40-4 AC-5 renvoyait déjà à 40-1 AC-8 pour `amount = available + pending` (rien à faire). Propagé dans ce fichier :
  NFR-40-4-1 (bornes : date d'écriture = date de `as_of`) et risque de 40-4 (versement sans `value_date` jugé à son
  écriture).

## Questions ouvertes

- **Q-40-1 — Numéro (Guy)** : 40 retenu provisoirement (B25) ; confirmation attendue.
- **Q-40-2 — Boutique (Beat, recette)** : où WHMCS et CB Paid Subscriptions posent-ils le numéro de facture dans
  Stripe — `description` (« Invoice #… »), une clé de `metadata`, autre ? Défaut proposé : `metadata:invoice`, la forme
  des paiements de test (G14).
- **Q-40-3 — Versements EUR et USD (Beat)** : vers quel compte — UBS en devise, Wise EUR/USD (1035, 1036 : virement entre
  deux comptes prestataires), ou pas de versement (solde qui s'accumule) ? Le règlement multi-devises est-il activé, ou
  Stripe convertit-il tout en CHF ?
- **Q-40-4 — Date de l'écriture du versement (Beat)** : G10 la date de l'arrivée en banque (créance jusque-là) ; la
  sortie du solde Stripe est l'autre choix. Comment les livres 2024 l'ont-ils daté ? (Le rattachement en lot n'a pas de
  fenêtre de date : il marche dans les deux cas.)
- **Q-40-5 — TVA sur les frais (Beat, Guy)** : les frais Stripe (Stripe Payments Europe, Irlande) et PayPal
  (Luxembourg) sont des services de l'étranger : impôt sur les acquisitions (B6, B18) ? Le socle passe les frais **sans
  TVA** (38-6a) ; un `fee_details` de type `tax` dit-il une TVA prélevée par Stripe ? Tracé (C-40-7) en attendant.
- **Q-40-6 — Réserves, solde minimum, litiges (Beat)** : en a-t-il ? Classés `other` (G9) ; à reprendre si fréquents.
- **Q-40-7 — Historique et 2024 (recette, Beat)** : jusqu'où remonte la liste ; les frais Stripe de 2024 sont-ils, comme
  ceux de PayPal, des écritures à part (B28 : groupe d'écritures de 38-6c2) ?
- **Q-40-8 — Droits de la clé (recette)** : droits *Read* minimaux pour `balance`, `balance_transactions`, l'expansion
  des sources (charges, paiements, remboursements, versements, litiges) et `GET /v1/account` ; noms exacts au tableau de
  bord.
- **Q-40-9 — Release (Guy)** : sans migration, avec la release de l'Epic 38 ou après.
- **Q-40-10 — Liquidité (Guy)** : un compte bancaire Kesh lié à une créance (1101–1103) doit-il apparaître dans les
  totaux de trésorerie de l'accueil, ou en être écarté ? (40-4 recense, Guy tranche.)
- **Q-40-11 — Conversions entre soldes Stripe (Beat)** : en fait-il ? Sinon G19 suffit.

## Règles de cet epic

- Méthode du dépôt (`CLAUDE.md`) : fiche → validation en boucle → développement → revue en boucle, gate local complet
  avant la PR ; splitting préventif (≤ 5 modules par story).
- **Production** : l'instance de Beat n'est jamais écrite par le travail de cet epic ; la recette sur son compte Stripe
  réel se fait hors de Kesh ou sur une **instance jetable**, avec une clé restreinte en lecture qu'il peut révoquer.
- Aucun appel réseau en CI ni par un agent ; aucune réponse réelle non anonymisée ni aucune clé dans le dépôt.
- Un appariement automatique **propose**, il ne crée jamais : le versement ne se rattache à la ligne UBS que par le geste
  de l'utilisateur.
- On trace toujours : un défaut voisin non traité se note au registre (C-40-7), pour devenir une issue avec la PR.
