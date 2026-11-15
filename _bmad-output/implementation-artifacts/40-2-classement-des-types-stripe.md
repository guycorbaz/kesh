# Story 40-2 : classement des types Stripe — genres, codes courts, montants, liens, boutique, surcharges

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/40-stripe`, numéros de ligne relevés au commit `a2b1da9c`. Epic 40
(`planning-artifacts/epic-40-stripe.md`) : G6, G7, G8, G9, G11, G14, G19 ; contrats C-40-0, **C-40-4** ; décisions de
Beat B3, B7, B13, B20, B24. **Dépend de** 40-1 (`StripeRecord`, `StripeSource`, `StripePayoutFields`,
`stripe::amounts::minor_to_decimal`, `StripeSettings`, `classify_window` provisoire qu'elle remplace) et, par elle, de
38-3b (`ProviderMovement`, `MovementKind`, `ReviewReason`, `MovementPage`, `SkippedCounts`), 38-3a (colonnes de C-38-4)
et 38-8 (patron : `classify_window`, `Classified`, surcharges). **Lue par** 40-3 (genre `withdrawal`, `value_date`,
lien d'un échec de versement), 38-4 (critères `invoice_prefix`, `event_code_prefix`, `counterparty_email_exact`),
38-6a/38-6b (genres, liens, file), 40-6 (codes courts en aide). **Aucune migration.***

*Modules : `kesh-providers` (`stripe::classify`, `stripe::event_codes`, `stripe::origins`), `kesh-core`
(`StripeSettings.eventKindOverrides`) — deux (≤ 5).*

## Story

En tant que comptable d'une société qui encaisse par Stripe, je veux que chaque transaction de solde devienne un
mouvement classé — vente, remboursement, frais, versement, dépôt, litige ou « autre » — avec son brut, ses frais rendus
ou prélevés, son net, sa date, sa contrepartie, sa boutique (lue où mon logiciel de vente la pose) et le lien vers la
vente qu'un remboursement ou un litige concerne, et que ce que Kesh ne sait pas classer aille à la file « à examiner »
plutôt qu'en vente, afin que les règles et les propositions travaillent sur des champs justes.

## Source (relevée le 2026-11-15)

- Types : <https://docs.stripe.com/reports/balance-transaction-types> ; catégories :
  <https://docs.stripe.com/reports/reporting-categories> ; liste des `type` filtrables relevée sur
  <https://docs.stripe.com/api/balance_transactions/list> (≈ cinquante valeurs : `adjustment`, `charge`, `payment`,
  `payment_failure_refund`, `payment_refund`, `payment_reversal`, `payout`, `payout_cancel`, `payout_failure`,
  `payout_minimum_balance_hold`, `payout_minimum_balance_release`, `refund`, `refund_failure`, `reserve_transaction`,
  `reserved_funds`, `reserve_hold`, `reserve_release`, `payment_network_reserve_hold`, `payment_network_reserve_release`,
  `stripe_fee`, `stripe_fx_fee`, `tax_fee`, `topup`, `topup_reversal`, `transfer*`, `application_fee*`, `issuing_*`,
  `climate_order_*`, `connect_collection_transfer`, …).
- Liste filtrée par **`source`** : « *Only returns transactions associated with the given object* » (même page).
- Versement : <https://docs.stripe.com/api/payouts/object> — `balance_transaction` (« *the impact of this payout on
  your account balance* »), `failure_balance_transaction` (« *If the payout fails or cancels, this is the ID of the
  balance transaction that reverses the initial balance transaction* »), `original_payout`, `arrival_date`.
- Réponses de test (`data-not-committed/stripe-test-samples/balance_transactions.json`) : vente `charge` `amount 12000`,
  `fee 420`, `net 11580`, `fee_details [{type: stripe_fee, amount: 420}]`, source `ch_…` avec `description`
  « Order WEB-1004 », `metadata.invoice` « WEB-1004 », `billing_details.email` et `name` nuls ; remboursement `refund`
  `amount −1500`, `fee −49`, `net −1451`, source `re_…` avec `charge` ; litige `adjustment` /
  `reporting_category: dispute`, `amount −3000`, `fee 2000` (« Dispute fee »), `net −5000`, source `du_…` avec `charge` ;
  versement `payout` `amount −30000`, source `po_…` `amount 30000`, `arrival_date` minuit UTC, `balance_transaction` =
  l'`id` de la transaction elle-même.

## Exigences

- `FR-40-2-1` — Chaque type Stripe se classe selon la table de G9 ; un type inconnu → `other`, `unknown_event` → AC-2.
- `FR-40-2-2` — Brut, frais et net se lisent de `amount`, `fee` et `net` à l'exposant de la devise (G7) → AC-3.
- `FR-40-2-3` — Identifiant `txn_…`, référence de la source, remboursement et litige reliés à leur charge, échec de
  versement relié au versement (G6, G11) → AC-4, AC-5.
- `FR-40-2-4` — Courriel, nom, description et boutique (`invoiceRefSource`) se posent depuis la source (G14) → AC-4.
- `FR-40-2-5` — Un litige ou sa reprise va à la file « à examiner », sans proposition → AC-2.
- `FR-40-2-6` — Un réglage de connexion peut classer un code que la table met en `other` → AC-6.
- `FR-40-2-7` *(ajoutée)* — Un versement porte sa date d'arrivée (`value_date`), lue du versement → AC-2.
- `NFR-40-2-1` — Classement pur, sans base, réseau ni horloge ; la seule lecture réseau (origine hors fenêtre) est hors
  du classement, bornée → AC-1, AC-5.
- `NFR-40-2-2` — Un test par type relevé sur les réponses de Beat ; aucun type inconnu ne devient une vente → AC-7.
- `NFR-40-2-3` — Somme des nets d'une fenêtre = variation du solde `available + pending` (test sur fixtures) → AC-7.

## Écarts à l'epic

1. **Origine hors fenêtre** : G6 la cherche par `expand[]=data.source.charge`. L'expansion d'un chemin qui n'existe pas
   sur toutes les sources d'une liste polymorphe (une source `charge` n'a pas de champ `charge`) n'est pas documentée
   (page *Expanding* relue : rien sur ce cas) — une erreur 400 bloquerait toute fenêtre. Retenu : une lecture
   **documentée**, `GET /v1/balance_transactions?source=<ch_…>` (filtre `source`), seulement pour les origines absentes de
   la fenêtre, au plus 50 par fenêtre (AC-5).
2. **Code `OTHER` non surchargeable** : G9 admet une surcharge « sur les seuls codes classés `other` » ; `OTHER` regroupe
   **tous** les types inconnus de la table, futurs compris — le surcharger classerait d'avance des types que personne n'a
   vus. Refusé (`code_not_overridable`).
3. **`value_date` posée ici** (FR-40-2-7) : la table de G9 la pose au classement (`payout` → `value_date`) ; 40-3 l'écrit en
   base et s'en sert (C-40-2 a, b). Le champ et sa validation viennent de 40-1 (AC-7 de 40-1).
4. **Motif des formes incohérentes** : G9 ne dit pas le motif d'un `charge` à montant ≤ 0 ou d'un remboursement positif ;
   retenu `other` (patron 38-8 AC-3), `unknown_event` restant aux types de la dernière ligne de G9.

## Critères d'acceptation

1. **AC-1 — Contrat (C-40-4, patron 38-8 AC-1).** `kesh_providers::stripe::classify::classify_window(records:
   &[StripeRecord], ctx: &ClassifyCtx) -> Classified { movements, skipped, sums }` (types de 38-8/38-3b) ; `ClassifyCtx {
   overrides: BTreeMap<String, OverrideKind>, invoice_ref_source: InvoiceRefSource (`enum { Description, Metadata(String) }`, défini ici dans `stripe::classify` et
   tiré du réglage déjà validé de 40-1 AC-2), origins: BTreeMap<String, String> }`
   (`origins` : identifiant de charge → `id` de sa transaction de solde, rempli par AC-5) — rien d'autre : ni horloge, ni
   base, ni réseau, ni `tracing` hors `warn!` de troncature (NFR-40-2-1). Appelée **une fois par fenêtre** par
   `StripeConnector::movements` (40-1 AC-7), qui remplace le classement provisoire de 40-1. Ordre de sortie : celui des
   enregistrements reçus (croissant, 40-1). Sort d'un enregistrement :
   - `balance_type` posé et ≠ `payments` → écarté, `skipped.non_balance` (G9 : hors de `available + pending`) ;
   - surcharge `ignore` (AC-6) → écarté, `skipped.non_balance` ;
   - sinon → un `ProviderMovement`. `status` `pending` **n'écarte pas** (G2 : la créance existe dès la charge) ;
     `skipped.denied` et `skipped.pending` restent à 0.
   `sums` = Σ `net` par devise des mouvements rendus.
2. **AC-2 — Table des types (G8, G9 ; `stripe::event_codes`, une constante `&[(type, code, Rule)]` documentée ligne
   par ligne avec la page Stripe).** Code court (`event_code`, ≤ 16, majuscules) et genre. **Ses codes** — la constante
   `EVENT_CODES`, définie dans `kesh_core::providers::settings::stripe_codes` (AC-6) et réexportée par
   `kesh_providers::stripe::event_codes` — sont une constante **`pub`**, lue par la garde de concordance de 40-6 AC-6
   (codes, `OTHER` compris) ; test : `EVENT_CODES` égale l'ensemble des codes de la table (AC-6) :

   | `type` (`reporting_category`) | `event_code` | Genre (`movement_kind`) | `review_reason` | Champs propres |
   |---|---|---|---|---|
   | `charge` | `CHARGE` | `sale` si `amount > 0`, sinon `other` | — ; `other` sinon | contrepartie, boutique (AC-4) |
   | `payment` | `PAYMENT` | idem | idem | idem |
   | `refund` | `REFUND` | `refund` si `amount < 0`, sinon `other` | — ; `other` sinon | lien à la charge (AC-5) |
   | `payment_refund` | `PAY_REFUND` | idem | idem | idem |
   | `adjustment` (`dispute`) | `DISPUTE` | `dispute` | `dispute` | lien à la charge |
   | `adjustment` (`dispute_reversal`) | `DISPUTE_REV` | `dispute` | `dispute` | lien à la charge |
   | `adjustment` (autre ou absente) | `ADJUSTMENT` | `other` | `unknown_event` | — |
   | `payout` | `PAYOUT` | `withdrawal` si `amount < 0`, sinon `other` | — ; `other` sinon | `value_date` (ci-dessous) |
   | `payout_cancel` | `PAYOUT_CANCEL` | `other` | `unknown_event` | lien au versement (AC-5, G11) |
   | `payout_failure` | `PAYOUT_FAIL` | `other` | `unknown_event` | lien au versement (AC-5, G11) |
   | `stripe_fee` | `STRIPE_FEE` | `fee` | — | montants de genre `fee` (AC-3) |
   | `stripe_fx_fee` | `FX_FEE` | `fee` | — | idem |
   | `tax_fee` | `TAX_FEE` | `fee` | — | idem |
   | `topup` | `TOPUP` | `deposit` si `amount > 0`, sinon `other` | — ; `other` sinon | — |
   | `topup_reversal` | `TOPUP_REV` | `other` | `unknown_event` | lien au dépôt (AC-5) |
   | `refund_failure` | `REFUND_FAIL` | `other` | `unknown_event` | lien au remboursement (AC-5) |
   | `payment_failure_refund` | `PAY_FAIL_REFUND` | `other` | `unknown_event` | — |
   | `payment_reversal` | `PAY_REVERSAL` | `other` | `unknown_event` | — |
   | `reserve_transaction`, `reserved_funds`, `reserve_hold`, `reserve_release`, `payment_network_reserve_hold`, `payment_network_reserve_release`, `payout_minimum_balance_hold`, `payout_minimum_balance_release` | `RESERVE` | `other` | `unknown_event` | — |
   | `transfer`, `transfer_cancel`, `transfer_failure`, `transfer_refund`, `application_fee`, `application_fee_refund`, `connect_collection_transfer` | `CONNECT` | `other` | `unknown_event` | — |
   | `issuing_*` (toute valeur à ce préfixe) | `ISSUING` | `other` | `unknown_event` | — |
   | tout autre type (Climate, `advance*`, `contribution`, `tax_fund`, inconnu, futur) | `OTHER` | `other` | `unknown_event` | — |

   - Le type se compare **en entier** (sauf la ligne `issuing_*`, préfixe documenté) : jamais par famille ; un `charge_x`
     futur est inconnu. ⛔ **Aucun type hors des deux premières lignes ne devient une vente**, ni par défaut ni par surcharge
     non admise (AC-6).
   - **`value_date` d'un versement** (`withdrawal`, FR-40-2-7, G10) : `source.payout.arrival_date` (date UTC, 40-1 AC-6) ;
     source absente ou non `payout` → genre `other`, `review_reason` `other` (Écart 4 : forme incohérente) (un versement sans date d'arrivée ne se propose pas).
     Tous les autres genres : `value_date = None`. Un versement à `amount > 0` (débit du compte bancaire par Stripe pour un
     solde négatif, forme à constater, Q-40-2-1) va à la file par la ligne ci-dessus.
   - `dispute` et `other` n'ont **aucune proposition** (38-6a C-38-6a-11, 38-6b C-38-6b-4 : la file).
3. **AC-3 — Montants (G7, C-38-4, FR-40-2-2).** Par `minor_to_decimal` (40-1 AC-8) dans la devise de la transaction
   (majuscules) :
   - genres `sale`, `refund`, `withdrawal`, `deposit`, `dispute`, `other` : `gross = amount`, `fee = −fee` (Stripe signe
     `fee` positif quand il est prélevé : vente `fee 420` → frais Kesh `−4.20` ; négatif quand il est rendu :
     remboursement `fee −49` → `+0.49`, frais rendus de 38-6b), `net = net` ; `net == gross + fee` tient par la validation
     de 40-1 (`net == amount − fee`) ;
   - genre `fee` : `gross = amount`, `fee = 0`, `net = amount` (le mouvement entier va au compte de frais, 38-6a) ; un
     `fee ≠ 0` sur un type de frais → `other` / `other` ;
   - **cohérence** (contrôle, non correction) pour `sale`, `refund`, `withdrawal`, `deposit` : `net` du même signe que
     `gross` ou nul, `|fee| ≤ |gross|` ; sinon `other` / `other`, montants lus gardés ;
   - frais de conversion de présentation : compris dans `fee` (G19) ; `exchange_rate` n'est pas gardé (C-40-7) ;
     `fee_details` n'est pas ventilé (un `fee_details` de type `tax` reste dans les frais, Q-40-5).
4. **AC-4 — Champs (G6, G14, FR-40-2-3, FR-40-2-4).**
   - `provider_tx_id = id` (`txn_…`, jamais tronqué) ; `booked_at = created` (UTC ; la date comptable est de 38-3b) ;
     `event_code` de l'AC-2 ; `reference = source.id` (`ch_`, `py_`, `re_`, `po_`, `du_`, `dp_`, …) ou `NULL`.
   - **Vente** (`charge`, `payment`, source `charge`) : `counterparty_email = source.billing_email` (champ de
     `StripeSource`, 40-1 AC-6), sinon `source.receipt_email`, sinon `NULL` (tel quel, la normalisation est celle de la
     règle, 38-4) ; `counterparty_name = source.billing_name` ; **`invoice_ref`** selon `invoice_ref_source` : `description` → `source.description` ;
     `metadata:<clé>` → `source.metadata[<clé>]` ; absente ou vide → `NULL`, **jamais de repli** sur l'autre source
     (G14) ; > 127 caractères → tronquée à 127 (frontière de caractère, `tracing::warn!` sans la valeur, patron 38-8 AC-4).
     Autres genres : contrepartie et `invoice_ref` `NULL`.
   - `details` = `source.description`, sinon `description` de la transaction, sinon `Stripe <type>` (jamais vide ; aucun
     texte traduit en base) ; `item_names = NULL` ; `counterparty_iban`, `end_to_end_id`, `transaction_id` `NULL` (38-3b).
   - Test sur la fixture : `invoiceRefSource` par défaut → `WEB-1004` et `SHOP-2001` (deux boutiques) ; `description` →
     `Order WEB-1004` ; `metadata:autre` → `NULL` ; courriel et nom `NULL` (paiements de test).
5. **AC-5 — Liens (`related_provider_tx_id`, G6, G11).**
   - **Remboursement et litige** (`refund`, `payment_refund`, `adjustment` de litige) : origine = `source.charge` ; lien =
     `id` de la transaction dont `type ∈ {charge, payment}` et `source.id` = cette charge, **parmi les enregistrements de
     la fenêtre**, sinon `ctx.origins[charge]`, sinon `NULL` (38-6b calcule alors `refund_origin_missing` : rien n'est
     écrit ici, O11 ; `review_reason` reste `NULL` pour un remboursement).
   - **Échec et annulation de versement** (`payout_failure`, `payout_cancel`, source `payout`) : lien =
     `source.balance_transaction` (la transaction du versement d'origine, doc ci-dessus) s'il diffère de `id` ; sinon `NULL`.
     **De même** `refund_failure` (source `refund`) et `topup_reversal` (source `topup`) : `source.balance_transaction`
     s'il diffère de `id` ; la source de `topup_reversal` (objet `topup` ou objet propre) se constate à 40-0 : champ
     absent → `NULL`, jamais une erreur.
   - **Résolution des origines hors fenêtre** (`stripe::origins`, appelé par `StripeConnector::movements` **avant**
     `classify_window`, I/O hors du classement, Écart 1) : pour chaque charge citée par un remboursement ou un litige et
     absente des sources de la fenêtre, `GET {base}/v1/balance_transactions?source=<charge>&limit=10&expand[]=data.source` (en-têtes de 40-1
     AC-1 ; **l'expansion est requise** : sans elle `source` est une chaîne et la règle `"record.source"` de 40-1 AC-6
     rejetterait la lecture ; validation des enregistrements de 40-1 AC-6 hors la règle `created` dans la fenêtre) ; exactement une transaction
     de type `charge` ou `payment` → `origins[charge] = id` ; aucune ou plusieurs → rien ; **au plus 50 lectures par
     fenêtre** (au-delà : rien, `tracing::warn!` du nombre ; ces lectures sont comptées **à part** du plafond global de
     1 000 requêtes de 40-1 AC-7, qui ne borne que le parcours des pages) ; **`Rejected`** (400, 402, 404 : charge
     inconnue, d'un autre compte, filtre refusé) → rien, `tracing::warn!` sans la valeur — la lecture n'est qu'un
     enrichissement, et un refus **durable** ferait échouer la même fenêtre à chaque passage, bloquant tout le
     téléchargement ; `Auth`, `Permission`, `RateLimited`, `Unavailable`, `Transport`, `InvalidResponse` → l'erreur du
     connecteur, la fenêtre échoue (relue au passage suivant, 38-3b). Test : remboursement d'une charge d'une fenêtre
     antérieure → lien posé par une lecture ; 51 origines manquantes → 50 lectures ; 404 et 400 → lien `NULL`, fenêtre
     rendue ; 503 deux fois → fenêtre en échec.
6. **AC-6 — Surcharges `eventKindOverrides` (FR-40-2-6, patron 38-8 AC-7).** Champ de `StripeSettings`
   (`kesh-core`, 40-1 AC-2), modifiable par le client, `#[serde(default)]`, `BTreeMap<String, OverrideKind>` :
   - clé : un code court de l'AC-2 ; **seuls** ceux que la table classe `other` sans condition de signe se surchargent —
     `ADJUSTMENT`, `PAYOUT_CANCEL`, `PAYOUT_FAIL`, `TOPUP_REV`, `REFUND_FAIL`, `PAY_FAIL_REFUND`, `PAY_REVERSAL`,
     `RESERVE`, `CONNECT`, `ISSUING` ; **refusés** : `OTHER` (Écart 2), les codes classés autrement (`CHARGE`, `REFUND`,
     `PAYOUT`, `DISPUTE`, …) → `code_not_overridable` ; un code absent de la table → `malformed` ; les codes
     surchargeables sont une constante `pub` **`OVERRIDABLE_CODES`**, lue par la validation et par la garde de 40-6
     (AC-6, manuel). ⚠️ **Elle vit dans `kesh-core`** (`kesh_core::providers::settings::stripe_codes`, avec
     `EVENT_CODES`, la liste de tous les codes, que `malformed` exige), **non** dans `kesh-providers` : la validation de
     `StripeSettings` est dans `kesh-core` (désérialisation typée de 38-1), et `kesh-providers` dépend de `kesh-core`
     (38-3b AC-1) — l'inverse ferait un cycle. `kesh_providers::stripe::event_codes` les **réexporte** (`pub use`), si
     bien que le chemin `kesh_providers::stripe::event_codes::OVERRIDABLE_CODES` que lit 40-6 reste valable. Tests
     (`kesh-providers`) : `EVENT_CODES` égale l'ensemble des codes de la table ; `OVERRIDABLE_CODES` égale l'ensemble des
     codes que la table classe `other` sans condition de signe, `OTHER` exclu ;
   - valeur : `sale` | `purchase` | `fee` | `withdrawal` | `deposit` | `dispute` | `other` | `ignore` ; `conversion` et
     `refund` refusés (`kind_not_allowed`) ; ≤ 50 entrées (`too_many`) ;
   - effets (38-8 AC-7) : `review_reason` `NULL` pour `sale`, `purchase`, `fee`, `withdrawal`, `deposit` ; `dispute` pour
     `dispute` ; `other` pour `other` ; contraintes de signe gardées (`sale`/`deposit` : > 0 ; `purchase`/`withdrawal` :
     < 0 ; sinon `other`/`other`) ; `fee` suit la règle `fee` de l'AC-3 ; un `withdrawal` surchargé n'a **pas** de
     `value_date` (aucun versement source) et suit 38-6b sans date d'arrivée ; la cohérence des montants (AC-3) s'applique
     après la surcharge ; `ignore` écarte un mouvement qui **change le solde** : le contrôle de complétude (E20 a) verra un
     écart durable (dit par 40-6 sous le champ et au manuel) ; la surcharge vaut pour les mouvements téléchargés après son
     écriture (aucun reclassement, E6) ;
   - violation → `400 PROVIDER_SETTINGS_INVALID`, `details: { field: "eventKindOverrides", code, reason }` (code de 38-1,
     raisons de 38-8 AC-7) ; audit : celui du `PATCH` de 38-1 (avant/après) ; aucun libellé neuf.
7. **AC-7 — Tests (NFR-40-2-1 à -3).**
   - **Table** : un test par ligne de l'AC-2, signes positif, négatif et nul pour les genres signés ; un test qui balaie la
     liste **complète** des `type` de la page de liste (relevée en constante de test, avec sa source) et vérifie qu'aucun ne
     rend `sale` hors `charge`/`payment` ; `charge_x` et `foo` → `OTHER` ; `adjustment` sans catégorie → `ADJUSTMENT` ;
     `climate_order_purchase` à source `climorder_…` → `OTHER` dans une page **acceptée** (règle `source.id` de 40-1
     AC-6).
   - **Réponses de Beat** (fixtures anonymisées de 40-1 et de 40-0) : chaque type relevé a un test qui fige son genre ; le
     test échoue en nommant le type si une fixture en contient un sans test (liste construite depuis les fixtures).
   - **Somme** (NFR-40-2-3) : sur la fixture de test, Σ `net` des mouvements (aucun écarté dans cette fixture,
     toutes `balance_type: payments`) = `available + pending` du `balance.json` par devise (USD 398.61, EUR 77.15, CHF 48.07 — le compte de
     test a commencé à zéro) ; sur deux lectures de solde encadrant une fenêtre (fixtures de 40-0), Σ nets = variation.
   - **`value_date`** (FR-40-2-7) : versement `arrival_date` → date UTC lue ; source absente ou non `payout` → `other` ;
     tout autre genre → `None`.
   - **Montants** : vente, remboursement à frais rendus (`+0.49`), litige à frais (`−20.00`), frais seul, frais à `fee ≠ 0`,
     net de signe contraire, `|fee| > |gross|`, devise à zéro décimale (`JPY 500` → `500`).
   - **Champs** : chaque `invoiceRefSource`, valeur de métadonnée absente ou vide (la clé `metadata:` vide est déjà refusée par 40-1 AC-2), 300 caractères (→ 127), courriel de `receipt_email` seul.
   - **Liens** : remboursement et litige dans la fenêtre, hors fenêtre (origines), sans origine ; échec et annulation de
     versement ; `refund_failure`, `topup_reversal`.
   - **Surcharges** : chaque refus ; `ignore` compté ; `PAYOUT_FAIL` surchargé `deposit` sur un montant positif → `deposit`
     sans `value_date` ; surcharge `sale` sur un débit → `other`.
   - Aucun test de classement ne touche la base ni le réseau ; ceux de l'AC-5 utilisent le serveur local de 40-1.
8. **AC-8 — Ce que la story ne touche pas.** Aucune migration (C-40-0 : `event_code` ≤ 16 tient dans `VARCHAR(16)` de
   38-3a — test : chaque code de la table ≤ 16 ; `movement_kind` et `review_reason` à domaine ouvert, valeurs de C-38-4) ;
   aucune route (C-40-6) ; aucun outil MCP (G17) ; CHANGELOG : une ligne de l'entrée non datée de l'epic (classement des types, surcharges) ; aucun écran ni manuel (40-6 : liste des codes courts en aide du critère
   `event_code_prefix`, champ des surcharges, ce qui va à la file) ; i18n : aucun texte neuf (codes de refus de 38-1/38-8).

## Tâches

- [ ] T1 — `stripe::event_codes` : table, codes, règles ; test de longueur (AC-2, AC-8).
- [ ] T2 — `stripe::classify` : `ClassifyCtx`, `classify_window`, montants, champs, liens dans la fenêtre ; branchement
      dans `StripeConnector::movements` à la place du classement provisoire de 40-1 (AC-1, AC-3, AC-4, AC-5).
- [ ] T3 — `stripe::origins` : lecture bornée par `source`, tests au serveur local (AC-5).
- [ ] T4 — `kesh-core` : constantes `EVENT_CODES` et `OVERRIDABLE_CODES` (réexportées par `stripe::event_codes`),
      `eventKindOverrides` de `StripeSettings`, validation, tests (AC-6).
- [ ] T5 — Tests de l'AC-7 ; gate complet (fmt, clippy, nextest complet, `scripts/gate-ci.sh`).

## Dev Notes

- **Une seule fonction de correspondance** (C-40-4) : `classify_window` rend les `ProviderMovement` ; la correspondance
  vers les colonnes reste celle du passage de 38-3b (et son ajout `value_date` en 40-3).
- **Pourquoi le classement par fenêtre** : un remboursement se relie à sa charge parmi les enregistrements lus ensemble
  (patron 38-8 AC-5) ; la lecture d'origine (AC-5) ne sert qu'aux charges des fenêtres antérieures.
- **Litige et sa reprise** : deux transactions distinctes (`dispute`, `dispute_reversal`) sur la même source `du_…` /
  `dp_…` — d'où `provider_tx_id = txn_…` et non la source (G6). Les frais du litige restent dans la transaction (`fee`),
  à la file avec elle (B7).
- **Paiement converti** (G19) : Kesh voit la devise de règlement ; l'écart d'arrondi avec la TVA calculée par la
  boutique dans la devise présentée est assumé (dit au manuel par 40-6).
- **Points de contact** : 38-4 (préfixes de code : `CHARGE`, `PAY_`, `PAYOUT`… — `event_code_prefix` lit le code court) ;
  38-6b (file : `dispute`, `other`, `refund_origin_missing` calculé) ; 40-3 (`withdrawal`, `value_date`, lien
  d'échec) ; 40-6 (libellés).
- **Lignes relues** : aucune ligne de code de production n'est touchée par une prémisse de cette fiche hors du socle à
  venir (Epic 38 : 38-8 AC-1, AC-3, AC-4, AC-7 ; 38-3b AC-2) ; `crates/kesh-db/migrations/20260504000001_bank_imports.sql:64-65`
  (`reference VARCHAR(255)`, `details TEXT NOT NULL`) lues au commit `a2b1da9c`.
- **Splitting** : deux modules (≤ 5).

### References

- `planning-artifacts/epic-40-stripe.md` § G6–G9, G11, G14, G19, C-40-4, 40-2.
- `implementation-artifacts/38-8-codes-evenement-paypal.md` AC-1 à AC-9 ; `40-1-client-stripe.md` AC-6 à AC-8.
- Documentation Stripe citée en *Source*.

## Questions ouvertes

- **Q-40-2-1** (Beat, recette) : un débit du compte bancaire par Stripe (solde négatif) arrive-t-il en `payout` positif, en
  `topup`, ou autrement ? Aujourd'hui : file (AC-2).
- **Q-40-2** (epic) : où WHMCS et CB Paid Subscriptions posent-ils le numéro de facture — `metadata` de la **charge**, de
  l'intention de paiement (`payment_intent`, non développée ici), ou `description` ? Si c'est l'intention de paiement, une
  source `payment_intent_metadata:<clé>` demanderait `expand[]=data.source.payment_intent` (troisième niveau) : à décider
  sur constat.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — validation P3 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW ; remédiés dans la fiche (aucun
  inter-fiches). Remédiation P2 relue (`EVENT_CODES`/`OVERRIDABLE_CODES` dans `kesh-core` : cohérent avec 38-3b AC-1 et
  38-8 AC-7 ; `Rejected` d'une origine → `NULL` ; dix codes surchargeables recomptés contre la table) ; montants
  recalculés sur les fixtures (vente −4.20/net 115.80, remboursement +0.49/net −14.51, litige −20.00/net −50.00 : `net =
  gross + fee`). LOW : P3-1 AC-7, « clé vide » ambigu (clé refusée par 40-1, valeur vide visée) ; P3-2 AC-8, CHANGELOG
  non mentionné ; P3-3 AC-5, source de `topup_reversal` à constater (champ absent → `NULL`). Axes non exercés : réponses
  réelles de `topup`/`topup_reversal` (absentes des fixtures).
- 2026-11-15 — alignement inter-fiches P2 (Opus) : AC-2, les codes de la table (`EVENT_CODES`, `OTHER` compris, définie
  dans `kesh_core::providers::settings::stripe_codes`, réexportée) sont une constante `pub` lue par la garde de 40-6
  AC-6 (finding 40-6-P2-8).
- 2026-11-15 — validation P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW ; remédiés dans la fiche (aucun
  inter-fiches). MEDIUM : P2-1 AC-6, `OVERRIDABLE_CODES` placée dans `kesh-providers` (alignement P1) mais lue par la
  validation de `StripeSettings`, qui est dans `kesh-core` — `kesh-providers` dépendant de `kesh-core` (38-3b AC-1),
  c'était un cycle → constantes `EVENT_CODES` et `OVERRIDABLE_CODES` dans `kesh-core`, réexportées par
  `stripe::event_codes` (chemin lu par 40-6 inchangé) ; P2-2 AC-5, un 400 à la lecture d'une origine faisait échouer la
  fenêtre à chaque passage (seul 404 était toléré) → `Rejected` = lien `NULL`. LOW : P2-3 AC-5, lectures d'origine
  hors du plafond global de 40-1 dit ; P2-4 AC-7, test d'une source à préfixe long (`climorder_`) après la correction
  40-1-P2-3. Relu : 38-3b AC-1 (l. 56, dépendance à `kesh-core`), 38-8 AC-7, réponses de test
  (`balance_transactions.json`, `balance.json`, `payouts.json`), migration `20260504000001_bank_imports.sql:64-65`.

- 2026-11-15 — alignement inter-fiches P1 (Opus) : AC-6, les codes surchargeables deviennent la constante `pub`
  `OVERRIDABLE_CODES` de `stripe::event_codes`, lue par la validation et par la garde de 40-6 (finding de 40-6 P1).

- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW ; remédiés dans la fiche (aucun
  inter-fiches). P1-1 MEDIUM AC-5, lecture d'origine sans `expand` rejetée par la règle `record.source` de 40-1 ; P1-2
  MEDIUM AC-2, `review_reason` de `payout`/`topup` incohérents (`unknown_event`) contre l'Écart 4 (`other`) ; P1-3 LOW
  AC-4, `billing_details.*` au lieu des champs `billing_email`/`billing_name` de `StripeSource` ; P1-4 LOW AC-1, type
  `InvoiceRefSource` non défini ; P1-5 LOW AC-7, aucun test de `value_date` (FR-40-2-7).

- 2026-11-15 — rédaction (Opus). 8 AC ; 7 FR (dont 1 ajoutée), 3 NFR ; aucune migration ; quatre écarts à l'epic
  (origine hors fenêtre par le filtre `source` au lieu de l'expansion, `OTHER` non surchargeable, `value_date` posée au
  classement, motif des formes incohérentes). Documentation Stripe relue (WebFetch) : liste des transactions et ses
  filtres, expansion, objet versement.
