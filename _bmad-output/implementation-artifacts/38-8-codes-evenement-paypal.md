# Story 38-8 : codes d'événement PayPal → mouvements classés ; `invoice_id` → boutique ; surcharges

Status: deferred

*(2026-11-20 — reportée en phase 2 par la décision du responsable de l'epic (phase 1 KISS, § « Phase 1 KISS » du fichier d'epic) ; validée le 2026-11-15, gardée telle quelle comme base de la phase 2. Ce qu'en reprend la phase 1 est dit dans les fiches 38-11a à 38-11d.)*

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`. Epic 38 (`planning-artifacts/epic-38-paypal.md`),
E4, E6, E7, E8, E16, E20, E24 ; contrats C-38-4, C-38-5, C-38-11. Décisions B3, B4, B7, B20, B24. **Dépend de** 38-7
(`PaypalRecord`, `Balances`) et, par lui, de 38-3b (`ProviderMovement`, `MovementKind`, `MovementPage`), de 38-3a (colonnes de C-38-4) et de 38-1
(`PaypalSettings`, E24). **Lue par** 38-4 (`invoice_prefix`, `event_code_prefix`, `counterparty_email_exact`,
`item_name_contains`), 38-6a et 38-6b (genres, liens, file « à examiner »). **Aucune migration.***

## Story

En tant que comptable d'une société qui encaisse et paie par PayPal, je veux que chaque enregistrement téléchargé
devienne un mouvement classé — vente, achat, frais, conversion, dépôt, retrait, remboursement, litige ou « autre » —
avec son brut, ses frais, son net, sa devise, son payeur, ses articles, sa boutique (préfixe du numéro de facture) et
le lien vers la transaction qu'il concerne, et que ce que Kesh ne sait pas classer aille à la file « à examiner »
plutôt qu'en vente, afin que les règles et les propositions des stories suivantes travaillent sur des champs justes.

## Source (relevée le 2026-11-15)

- Codes d'événement : <https://developer.paypal.com/docs/transaction-search/transaction-event-codes/> — familles
  T00xx (paiements), T01xx (frais, dont des retraits : T0105, T0108–T0110), T02xx (conversions : T0200, T0201,
  T0202), T03xx (dépôts), T04xx (retraits), T05xx (carte de débit PayPal), T06xx / T07xx (carte de crédit),
  T08xx–T10xx (bonus, bons, BillPay), T11xx (inversions et remboursements : T1106, T1107, T1108, T1109, T1110, T1111,
  T1114, T1115, T1120…), T12xx (rétrofacturations et ajustements : T1201, T1202, T1205, T1207, T1208…), T13xx
  (autorisations), T14xx–T20xx, T21xx (blocages, libérations, réserves), T30xx, T97xx, T98xx (« display-only »,
  sans effet sur le solde), T99xx.
- Champs : `transaction_event_code` « A five-digit transaction event code that classifies the transaction type based
  on money movement and debit or credit » ; `transaction_status` `D` refusée, `P` en attente, `S` réussie, `V`
  « fully reversed and funds were refunded » ; `paypal_reference_id` « Defined as a related, pre-existing transaction
  or event », type `ODR` | `TXN` | `SUB` | `PAP` ; `invoice_id` « The invoice ID that is sent by the merchant with the
  transaction » (schéma <https://developer.paypal.com/api/transaction-search/v1/schema.json>).
- ⚠️ La documentation **ne dit pas** le signe de `fee_amount` : il se constate sur les réponses du responsable de l'epic (AC-9) ; la
  règle de l'AC-3 ne dépend pas de ce signe tant que `net = brut + frais` se vérifie (voir *Dev Notes*).

## Exigences

- `FR-38-8-1` — Chaque code `T…` se classe en genre de mouvement selon la table ; un code inconnu → `other` → AC-2.
- `FR-38-8-2` — Brut, frais, net, devise, payeur, articles et liens (`related_provider_tx_id`) se lisent de la
  réponse → AC-3, AC-4, AC-5.
- `FR-38-8-3` — `invoice_id` se pose dans `invoice_ref` et `reference` (critère boutique, B24) → AC-4.
- `FR-38-8-4` — Un réglage de connexion peut classer un code non couvert — **retenu** (`eventKindOverrides`) → AC-7.
- `FR-38-8-5` *(ajoutée)* — Les deux moitiés d'une conversion sont reliées l'une à l'autre → AC-5.
- `FR-38-8-6` *(ajoutée)* — Seuls les enregistrements réussis qui changent le solde total deviennent des mouvements ;
  les autres sont comptés, jamais perdus en silence → AC-1, AC-6.
- `NFR-38-8-1` — Chaque code relevé sur les réponses du responsable de l'epic a son test ; aucun code inconnu ne devient une vente →
  AC-2, AC-9.
- `NFR-38-8-2` — Classement pur, sans base ni réseau → AC-8.
- `NFR-38-8-3` *(ajoutée)* — Le classement est vérifiable : sur les réponses du responsable de l'epic, Σ nets des mouvements d'une
  période = écart de `total_balance` entre ses bornes (contrôle E20 a) → AC-9.

## Critères d'acceptation

1. **AC-1 — Statuts et sort d'un enregistrement.** `classify_window(records: &[PaypalRecord], ctx: &ClassifyCtx) ->
   Classified { movements: Vec<ProviderMovement>, skipped: SkippedCounts, sums: Vec<(String, Decimal)> }`
   (`kesh-providers::paypal::classify` ; `SkippedCounts` et `sums` sont ceux de `MovementPage`, 38-3b AC-2, AC-6
   ci-dessous ; une surcharge `ignore`, AC-7, se compte en `skipped.non_balance`).
   `ClassifyCtx` porte les surcharges (AC-7) et la liste des `payerId` des autres connexions (AC-5), rien d'autre
   (aucune horloge, aucune base). **Où elle s'appelle (contrat avec 38-3b et 38-7 AC-4)** : 38-3b reçoit des
   `ProviderMovement` **déjà classés** (`Connector::movements`), page par page ; le classement exigeant la fenêtre
   entière (paires, liens), c'est **`PaypalConnector::movements` qui lit toutes les pages d'une fenêtre, appelle
   `classify_window` une fois, et rend une page unique** (`total_pages = 1`) avec `skipped` et `sums` (AC-6) — contrat porté par
   38-3b AC-2 (`MovementPage`). Aucun branchement dans le passage lui-même. Par enregistrement :
   - `D` (refusée) → écarté, `skipped.denied` ;
   - `P` (en attente) → écarté, `skipped.pending` : un paiement en attente ne change pas encore le solde total ; il se
     relit réussi à un passage suivant **si** sa date reste dans la fenêtre relue (recouvrement de trois jours,
     C-38-5) — sinon le contrôle de complétude (E20 a) le signale ; le recouvrement se règle (`overlapDays`, AC-7, O11) ;
   - `S` et `V` → classés (AC-2) : `V` est une transaction **réussie puis inversée** ; l'inversion arrive comme un
     enregistrement séparé (T11xx) et l'original a bien changé le solde ;
   - genre « hors solde » (AC-2 : T13xx, T15xx, T21xx, T98xx, T1110, T1111) → écarté, `skipped.non_balance`.
   `SkippedCounts` (denied, pending, non_balance) est rendu au passage (38-3b), qui l'écrit à son état ; rien n'est écarté
   sans être compté.
2. **AC-2 — Table de classement** (`kesh-providers::paypal::event_codes`, **une constante** `&[(code, Rule)]`
   ou correspondance `match` exhaustive par code, documentée ligne par ligne avec la page PayPal). Le signe est celui
   de `transaction_amount.value`.

   | Codes | Genre (`movement_kind`, C-38-4) | `review_reason` |
   |---|---|---|
   | T0000–T0013, T0018, T0019, T0022, T0023 | signe > 0 → `sale` ; < 0 → `purchase` ; nul → `other` | `other` si nul |
   | T0014 (rabais, cash back), T0015 (versement de tiers), T0016 (récupération de tiers), T0017 (paiement **ou remboursement** chaîné), T0020 (taxe d'une place de marché), T0021 (crypto) | `other` | `other` |
   | T0100–T0104, T0106, T0107, T0111–T0118, T0121 | `fee` (débit **ou** crédit : un crédit est un frais rendu) | — |
   | T0105 (retrait par chèque), T0108 (retrait au distributeur), T0110 (retrait vers une carte de crédit internationale) | `other` (ni frais ni retrait vérifiable sur un relevé de Kesh) | `other` |
   | T0109 (« Auto-sweep from account », virement automatique du solde, comme T0401) | `withdrawal` si < 0, sinon `other` | `other` sinon |
   | T0200, T0201, T0202 | `conversion` (AC-5) | — |
   | T0300–T0303 | `deposit` si > 0, sinon `other` | `other` sinon |
   | T0400–T0403 | `withdrawal` si < 0, sinon `other` | `other` sinon |
   | T0700 (dépôt par carte de crédit), T0701 | `deposit` si > 0, sinon `other` | `other` sinon |
   | T0600 | `withdrawal` si < 0, sinon `other` | `other` sinon |
   | T1106, T1107, T1114, T1115, T1120 | `refund` (AC-5) | — |
   | T1108, T1109 | `fee` | — |
   | T1201, T1202, T1205, T1207, T1208 | `dispute` | `dispute` |
   | T1110, T1111, T13xx, T15xx, T21xx, T98xx | **écarté** (« hors solde », AC-1) | — |
   | **tout autre code connu** (T05xx, T08xx–T10xx, T1100–T1105, T1112, T1113, T1116–T1119, T1200, T1203, T1204, T14xx, T16xx–T20xx, T30xx, T97xx, T99xx) | `other` | `other` |
   | **code absent de la page PayPal** | `other` | `unknown_event` |

   *Relevé de la page PayPal le 2026-11-15 (validation P1) : ~~T0108 « ATM withdrawal fee », T0109 « Auto-sweep fee »,
   T0110 « International credit card withdrawal fee » sont des **frais**~~ — **faux, corrigé à la validation P2**
   (page relue le 2026-11-15 : « `T0108` | ATM withdrawal », « `T0109` | Auto-sweep from account », « `T0110` |
   International CC withdrawal », rangés sous l'intitulé *Fees* mais sans le mot « fee », au contraire de T0113
   « Partner fee » ou T0118 « Gross billed fees ») : ce sont des **retraits**. Les classer `fee` aurait passé un
   retrait en charge — une erreur muette ; T0109 suit T0401 (`withdrawal`), T0108 et T0110, sans ligne de relevé
   bancaire à rattacher, vont à la file comme T0105 ; T0014 à T0017 ne sont pas des ventes (rabais, versement, récupération, paiement ou remboursement
   chaîné) ; T0105 est un retrait par chèque, sans ligne de relevé bancaire à rattacher. La page avertit que les codes
   « may change » : la table se rejoue contre les réponses du responsable de l'epic (AC-9), et le contrôle de somme (AC-6) en est le
   filet.*

   ⛔ Aucun code hors des lignes `sale` ne devient une vente, ni par défaut ni par famille (« T00 » ne se lit **jamais**
   par préfixe : un T0024 futur est inconnu). Un mouvement `dispute` ou `other` n'a **aucune proposition** (E16 ; 38-6b
   le met à la file). `refund_origin_missing` n'est **écrit nulle part** : 38-6b le calcule à la lecture (O11,
   C-38-6b-4).
3. **AC-3 — Montants** (C-38-4, E7). Pour un mouvement classé, en devise de l'enregistrement (`currency` =
   `transaction_amount.currency_code`) :
   - `gross_amount` = `transaction_amount.value` ; `fee_amount` = `fee_amount.value` ou 0 si absent ; `amount` (net,
     ce qui change le solde) = `gross_amount + fee_amount` ;
   - **genre `fee`** : `gross_amount` = `transaction_amount.value`, `fee_amount` = 0, `amount` = brut (le mouvement
     entier va au compte de frais, 38-6a) ; un `fee_amount` non nul sur un T01xx → `other`/`other` (forme inattendue) ;
   - **cohérence** (contrôle, non correction) : pour `sale`, `purchase`, `refund`, le net doit être du **même signe**
     que le brut ou nul, et |frais| ≤ |brut| ; sinon le mouvement passe `other`/`other` avec le brut, les frais et le
     net lus — jamais une proposition sur des montants incohérents ;
   - décimaux exacts (`rust_decimal`, déjà en `kesh-core` et `kesh-import`), deux décimales au plus (validé par 38-7).
4. **AC-4 — Champs textuels** (C-38-4, E8) :
   - `provider_tx_id` = `transaction_id` (tel quel ; unicité par devise garantie par 38-7 AC-4) ;
   - `event_code` = `transaction_event_code` ;
   - `booked_at` (`ProviderMovement`, 38-3b) = `transaction_initiation_date` **en UTC** (instant) ; **ce n'est
     pas ce classement qui calcule `booking_date`** : le passage de 38-3b le dérive de `booked_at` à Europe/Zurich
     (« Temps » de 38-3b), `value_date` reste `NULL`. Le test du 31.12 à 23 h 30 UTC (→ du **1.1** à
     Zurich) vit donc en 38-3b ; ici, un test fige `booked_at` à l'instant UTC reçu (décalages `Z`, `+00:00`,
     `+0000` de 38-7 AC-4) ;
   - **`invoice_ref` = `invoice_id`, et `reference` = `invoice_id`**, **sans transformation** (ni casse, ni espaces :
     la normalisation est celle de la règle `invoice_prefix`, une seule fonction, 38-4 NFR-38-4-2) ; `invoice_id`
     absent ou vide → `NULL` aux deux (aucune règle `invoice_prefix` ne s'applique ; B24) ; > **127** caractères (colonne
     `invoice_ref VARCHAR(127)`, 38-3a ; `reference` est `VARCHAR(255)`, `migrations/20260504000001_bank_imports.sql:64`)
     → tronqué à 127 **aux deux colonnes** (le préfixe, seul lu par la règle, est conservé), avec `tracing::warn!`
     sans la valeur ; `provider_tx_id` n'est jamais tronqué (refus, validation commune de 38-3b) ;
   - `counterparty_name` = `payer_name.full_name`, sinon `given_name` + `surname`, sinon `alternate_full_name`, sinon
     `NULL` (≤ 255) ; `counterparty_email` = `payer_info.email_address` tel quel (la normalisation est celle de la
     règle `counterparty_email_exact`, 38-4) ;
   - `item_names` = les `cart_info.item_details[].item_name` non vides, **dans l'ordre**, au format fixé par 38-3a pour
     la colonne (C-38-4), **bornés aux limites de `validate_page` de 38-3b** (50 noms au plus, 255 caractères au plus
     chacun) : les 50 premiers gardés, chaque nom tronqué à 255 caractères (frontière de caractère), avec
     `tracing::warn!` sans la valeur — 38-7 admet 500 articles de 2000 caractères, et `validate_page` **rejette la
     page** au-delà de ses bornes (`item_names` n'est pas un texte libre qu'elle tronque) : sans cette borne, une
     commande de 51 articles bloquerait sa fenêtre à chaque passage (validation P2). Test : 60 articles → 50 ; un nom
     de 300 caractères → 255 ;
   - `details` (`TEXT NOT NULL`, `:65`) = `transaction_subject`, sinon `PayPal <code>` (p. ex. `PayPal T0006`) —
     jamais vide, et aucun texte traduit écrit en base (l'écran traduit le genre) ;
   - `counterparty_iban`, `end_to_end_id` = `NULL` ; `transaction_id` (colonne existante) = `NULL` (le prestataire
     est dans `provider_tx_id`).
5. **AC-5 — Liens** (`related_provider_tx_id`, E7, E16) :
   - **remboursement, inversion, rétrofacturation, frais rendus** (`refund`, `dispute`, T1108, T1109) :
     `related_provider_tx_id` = `paypal_reference_id` si `paypal_reference_id_type = 'TXN'` ; sinon `NULL` (38-6b
     envoie alors le remboursement à la file : le motif `refund_origin_missing` est **calculé à la lecture** par 38-6b,
     jamais écrit ici — O11, 38-6b C-38-6b-4 ; `review_reason` reste `NULL` pour un remboursement) ;
   - **conversion** : les deux moitiés (deux enregistrements T02xx de devises différentes, de signes opposés) se
     **relient l'une à l'autre** (`related_provider_tx_id` de chacune = `provider_tx_id` de l'autre ; 38-6b lit l'un
     **ou** l'autre sens). Appariement (O11 : par `paypal_reference_id`, à constater sur les réponses du responsable de l'epic),
     dans la fenêtre classée : d'abord par `paypal_reference_id` commun ; sinon par **même instant d'initiation**
     (à la seconde), devises différentes, signes opposés, un seul candidat de chaque côté. Plusieurs candidats ou
     aucun → la moitié reste `conversion` sans lien et `review_reason = 'other'` (38-6b la met à la file au lieu de
     proposer une demi-conversion). La règle d'appariement **se confirme sur les réponses du responsable de l'epic** (AC-9) avant d'être
     figée ; si elles montrent un autre lien (p. ex. `paypal_reference_id` vers le paiement qui a déclenché la
     conversion), la fiche est amendée avant le développement de l'appariement ;
   - **paire coupée** : `classify_window` reçoit **tous les enregistrements d'une fenêtre** (AC-1 : le connecteur
     lit toutes les pages avant de classer) ; une moitié à la frontière d'une fenêtre (instants identiques, donc
     même fenêtre en pratique) se traite comme « aucun candidat » — la moitié reste sans lien et, comme toute moitié
     sans candidat, **garde `review_reason = 'other'`** ; 38-6b la met à la file (motif rendu `conversion_half_missing`)
     et **ignore ce motif stocké dès que l'autre moitié s'y relie** (C-38-6b-4) ; si l'autre moitié arrive plus tard, le recouvrement du passage suivant relit
     la première (déjà connue, jamais réécrite, sans lien) : seule la seconde porte alors le lien, et 38-6b le lit
     dans les deux sens (`u.related_provider_tx_id = t.provider_tx_id`).
   - **virement entre les deux comptes PayPal du responsable de l'epic** (E24 ; *suppose que `payer_info.account_id` désigne la
     contrepartie, y compris pour un débit : à constater sur les réponses du responsable de l'epic, Q-38-8-4*) : un T00xx dont `payer_info.account_id` égale le
     `payerId` d'une **autre connexion PayPal de la même société** n'est pas une vente ni un achat : genre `other`,
     `review_reason = 'other'`. `ClassifyCtx` porte la liste des `payerId` des autres connexions (lue par la fabrique de
     38-3b, passée ici ; aucune base dans le classement). Le rattachement des deux côtés se fait à la main ou par 38-6b.
6. **AC-6 — Contrôle de somme par fenêtre** (aide de test et de diagnostic, non bloquante) : `Classified` rend, par
   devise, `Σ amount` des mouvements ; le contrôle des soldes de 38-3b la compare à l'écart de `total_balance` (E20 a). Un écart s'explique soit
   par un enregistrement écarté à tort (AC-1, AC-2), soit par un téléchargement incomplet : c'est le filet qui rend une
   erreur de la table **visible** au lieu de muette.
7. **AC-7 — Surcharges `eventKindOverrides`** (E24, FR-38-8-4, retenu). Champ facultatif de `PaypalSettings`
   (`kesh-core::providers::settings`, 38-1), `#[serde(default)]`, `BTreeMap<String, OverrideKind>` :
   - clé : `^T\d{4}$` ; valeur : `sale` | `purchase` | `fee` | `withdrawal` | `deposit` | `dispute` | `other` |
     `ignore`. **Refusés** : `conversion` et `refund` (ils exigent un lien que la surcharge ne fournit pas) ; ≤ 50
     entrées ;
   - **seuls les codes que la table classe `other` sans condition de signe** (les lignes T0014–T0017, T0020, T0021,
     T0105/T0108/T0110, « tout autre code connu » et « code absent ») se surchargent : une clé visant un code classé
     autrement — y compris `deposit`/`withdrawal` sous condition de signe, et T1110/T1111/T13xx/T15xx/T21xx/T98xx
     « hors solde » — est refusée (`code_not_overridable`) ; un T0006 ne devient pas un achat par réglage ;
   - **effets d'une surcharge** : `review_reason` = `NULL` pour `sale`, `purchase`, `fee`, `withdrawal`, `deposit` ;
     `dispute` pour `dispute` ; `other` pour `other` ; `deposit`/`withdrawal` gardent la contrainte de signe (sinon
     `other`/`other`), comme `sale`/`purchase` ; un surchargé `fee` suit la règle `fee` de l'AC-3 ; la surcharge
     remplace la **ligne de table** seulement : le contrôle de cohérence des montants (AC-3) et le virement entre
     connexions (AC-5 — T0014 à T0017, T0020, T0021 sont des T00xx) s'appliquent **après** elle et la priment (un
     virement entre les deux comptes du responsable de l'epic surchargé `sale` reste `other`/`other`) ; **`ignore` écarte
     un code qui peut changer le solde** : le contrôle de complétude (E20 a, 38-3b AC-7) y verra alors un écart
     durable — c'est dit sous le champ (38-9a) et au manuel ;
   - violation → `PROVIDER_SETTINGS_INVALID` (400, code de 38-1), `details: { field: "eventKindOverrides", code: "T…",
     reason: "kind_not_allowed" | "code_not_overridable" | "too_many" | "malformed" }` — validation dans la
     désérialisation typée de 38-1 (aucun JSON stocké tel quel, E24) ;
   - `sale`/`purchase` surchargés **gardent la contrainte de signe** (une surcharge `sale` sur un débit → `other`) ;
   - une surcharge vaut pour les mouvements classés **après** son écriture ; les transactions déjà téléchargées ne sont
     pas reclassées (l'insertion idempotente de E6 n'écrase rien) : celles de la file se traitent à la main (34-3).
     L'écran de 38-9a le dit sous le champ ; le manuel aussi (38-9a) ;
   - audit : celui de la modification des réglages de 38-1 (avant/après) ; aucun libellé neuf ;
   - **amende 38-1 AC-4** (`PATCH` : « `settings` présent remplace les champs modifiables (`clientId`) et conserve
     ceux que le serveur pose ») : `eventKindOverrides` et `overlapDays` sont **modifiables par le client** et
     remplacés par le `PATCH` ; seul `payerId` est posé par le serveur (O10 : `accountEmail` n'existe plus) ;
   - **`overlapDays`** (O11, « recouvrement réglable » ; `Option<u8>`, 1 à 30, défaut 3, `PROVIDER_SETTINGS_INVALID`
     `reason: "out_of_range"` sinon) : un paiement `P` écarté puis réussi plus de trois jours après son initiation se
     relit en élargissant le recouvrement du passage (`Connector::overlap()`, lu par le passage de 38-3b à la place
     du « 3 jours » en dur ; liste inter-fiches P1). Au-delà, le contrôle de complétude (E20 a) signale l'écart.
8. **AC-8 — Pureté.** `classify_window` et `event_codes` n'ont ni I/O, ni horloge, ni accès base, ni `tracing` autre
   que `warn!` de troncature ; ils vivent dans `kesh-providers`, qui ne dépend pas de `kesh-db` (E9). Dépendances :
   `rust_decimal`, `chrono` (instants UTC seulement : pas de `chrono-tz`, la date comptable est de 38-3b), `kesh-core` pour `PaypalSettings` (si 38-1 l'a
   placé en `kesh-core::providers::settings`, E24).
9. **AC-9 — Tests.**
   - **Table** : un test par ligne de l'AC-2, signe positif, négatif et nul pour les genres signés ; un test « aucun
     code hors des lignes `sale` ne rend `sale` » qui balaie **tous les codes T0000–T9999** (10 000 valeurs, sans
     surcharge) ; T0024 (inexistant) → `unknown_event` ; un code connu non listé (T0500) → `other`/`other`.
   - **Réponses du responsable de l'epic** (fixtures anonymisées de 38-7, C-38-11) : **chaque code relevé** a un test qui fige son
     genre, et le test échoue en nommant le code si une fixture en contient un sans test (liste des codes relevés
     construite depuis les fixtures, comparée à la liste testée) ; sur chaque fenêtre de fixture couverte par deux
     soldes (`as_of_time` aux bornes), Σ nets = écart de `total_balance` par devise (NFR-38-8-3). Le relevé se fait sur
     une **instance jetable**, jamais sur la production du responsable de l'epic.
   - **Montants** : frais négatifs et positifs, frais absents, T01xx à `fee_amount` non nul, net de signe contraire,
     |frais| > |brut|.
   - **Champs** : `booked_at` UTC pour chaque forme de décalage, `invoice_id` absent, vide, de 300 caractères
     (tronqué à 127) ; payeur sans nom, à nom
     composé ; articles vides et multiples.
   - **Liens** : remboursement `TXN`, `ODR` (→ `NULL`) ; conversion par référence, par instant, ambiguë (deux paires au
     même instant), orpheline ; virement entre deux connexions (même `payerId`).
   - **Surcharges** : chaque refus de l'AC-7 (dont un code `other` sous condition de signe, et T0014 admis) ; `ignore`
     → écarté et compté ; surcharge `sale` sur un débit → `other` ; `review_reason` de chaque genre surchargé ;
     T0015 surchargé `sale` dont `payer_info.account_id` est le `payerId` d'une autre connexion → `other`/`other`.
   - Aucun test ne touche la base ni le réseau.
10. **AC-10 — Ce que la story ne touche pas.** Aucune migration ; aucune route (C-38-12 : delta 0) ; aucun outil MCP
    (E23) ; aucun écran (le champ des surcharges est à 38-9a) ; i18n : aucun texte neuf (les codes de refus sont ceux
    de 38-1 ; le libellé des genres de mouvement est à 38-9a/38-9b) ; manuel : à 38-9a (surcharges, classement, ce qui
    va à la file).

## Écart à l'epic

- **T1110 / T1111** (blocage pour enquête de litige et son annulation) : l'epic range T11xx en « inversions,
  remboursements » et les litiges en T12xx ; ces deux codes sont des **blocages** (ils ne changent pas
  `total_balance`) et sont **écartés** comme T15xx/T21xx. Le litige qui déplace de l'argent arrive en T1201 ou en
  remboursement.
- **T01xx** contient des retraits — T0105 (chèque), T0108 (distributeur), T0109 (virement automatique du solde),
  T0110 (carte de crédit internationale) — rangés sous l'intitulé *Fees* de la page PayPal : aucun n'est classé
  `fee` ; T0109 en `withdrawal` (comme T0401), les trois autres en `other` (file). *(La P1 les disait « frais » sur
  une citation fausse de la page ; corrigé en P2.)*
- **`reference`** : l'epic E8 la veut « numéro de facture du prestataire » ; FR-38-8-3 la veut égale à `invoice_id`.
  Retenu : `invoice_id` (seule valeur qui soit un numéro de facture ; le sujet va à `details`).
- **Virement entre deux comptes PayPal du responsable de l'epic** : non classé par l'epic ; retenu `other`, reconnu par `payerId`
  (AC-5), faute de courriel du compte (38-7, *Écart à l'epic*).

## Questions ouvertes

- **Q-38-8-1** (recette, le responsable de l'epic) : comment PayPal relie-t-il les deux moitiés d'une conversion dans ses réponses
  (référence commune, instant identique) ? Bloque l'appariement (AC-5), pas le reste.
- ~~Q-38-8-2~~ close par O11 : recouvrement réglable (`overlapDays`, AC-7). ~~Q-38-8-3~~ close par O11 : `ignore`
  gardé.
- **Q-38-8-4** (recette, le responsable de l'epic) : pour un **débit** (achat, virement vers l'autre compte), `payer_info` désigne-t-il la
  contrepartie (marchand, autre compte) ou le titulaire lui-même ? Décide la source de `counterparty_name`,
  `counterparty_email` (critère `counterparty_email_exact`, B5) et du `payerId` du virement entre comptes (AC-5) ;
  si c'est le titulaire, la contrepartie se lit ailleurs (`shipping_info`, `cart_info`) et la fiche est amendée avant
  le développement des champs.

## Tâches

- [ ] `event_codes` : table de l'AC-2, documentée ligne par ligne (AC-2).
- [ ] `classify_window` : statuts, montants, champs, liens, conversions, virements entre connexions, compteurs
      (AC-1, AC-3 à AC-6).
- [ ] `PaypalSettings.event_kind_overrides` et sa validation dans la désérialisation de 38-1 (AC-7).
- [ ] Appel depuis `PaypalConnector::movements` (38-7 AC-4) : `classify_window` sur la fenêtre entière, `ClassifyCtx`
      (surcharges, `payerId` des autres connexions, passés par la fabrique de 38-3b) ; `skipped` et `sums` dans
      `MovementPage`, écrits à l'état du passage par 38-3b.
- [ ] `PaypalSettings` : `event_kind_overrides` et `overlap_days`, validation (AC-7), refus d'un champ inconnu.
- [ ] Tests de l'AC-9, dont le balayage des 10 000 codes et le test « code relevé sans test ».
- [ ] Gate : `cargo fmt`, `clippy -D warnings`, `cargo nextest` (crates `kesh-providers`, `kesh-core`), gate complet
      avant `done`.

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release.)*

- **Ajouté** : la lecture des codes d'événement PayPal (vente, frais, conversion, remboursement, litige…) et leur
  classement en genres de mouvement, avec surcharges réglables.

**Compteurs** (registres, `Cargo.lock`, tableaux de codes) : recomptés depuis la source après chaque reprise de `main`.

## Dev Notes

- **Pourquoi `net = brut + frais` sans présumer le signe** : si PayPal rend les frais d'une vente négatifs (attendu,
  « montant prélevé »), `100 + (−3.20) = 96.80` ; s'il les rendait positifs, la cohérence de l'AC-3 (|net| ≤ |brut|,
  même signe) passerait les mouvements en `other` et le défaut se verrait à la recette au lieu d'écrire faux. Le
  signe constaté s'écrit ensuite ici et dans le manuel.
- **Pourquoi une table code par code** : la famille ne suffit pas (T01xx contient des retraits, T11xx des blocages) ;
  une erreur de classement est **muette** (une vente comptée en frais fausse produits et TVA). Le balayage des 10 000
  codes et le contrôle de somme (AC-6) sont les deux filets.
- **Colonnes** : noms de C-38-4 (`movement_kind`, `event_code`, `counterparty_email`, `invoice_ref`, `item_names`,
  `related_provider_tx_id`, `review_reason`, `gross_amount`, `fee_amount`, `provider_tx_id`) ; `ProviderMovement` est
  le type de 38-3b qui les porte. Les décodages tolérants (`Other(String)`) sont de 38-3a (E4).
- **`details TEXT NOT NULL`** et **`reference VARCHAR(255)`** : `migrations/20260504000001_bank_imports.sql:64-65`.
- **Points de contact** : 38-7 (`PaypalRecord`, validation, unicité) ; 38-3b (classement par fenêtre entière,
  compteurs, `ClassifyCtx`) ; 38-4 (normalisation unique des courriels et préfixes) ; 38-6a/38-6b (genres, liens,
  file) ; 38-1 (`PaypalSettings`, `PROVIDER_SETTINGS_INVALID`) ; 38-9a (champ des surcharges, textes).
- **Splitting** : deux modules (`kesh-providers`, `kesh-core`).

### References

- `planning-artifacts/epic-38-paypal.md` § E7, E8, E16, E20, E24, C-38-4, 38-8.
- `implementation-artifacts/38-7-client-paypal.md` AC-4, AC-6, *Écart à l'epic*.
- Documentation PayPal citée en *Source* (consultée le 2026-11-15).

## Change Log

- 2026-11-15 — rédaction (Opus).
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 1 HIGH, 5 MEDIUM, 3 LOW ; remédiés dans la fiche (appel par le
  connecteur sur la fenêtre entière, `booked_at` et non `booking_date`, table recoupée à la page PayPal — T0108 à
  T0110 en frais, T0105 et T0014 à T0017 en `other` —, `invoice_ref` à 127, `refund_origin_missing` calculé, `overlapDays`,
  champs modifiables au `PATCH`, Q-38-8-4) sauf les inter-fiches 38-3 (`MovementPage.skipped`/`sums`, `overlap()`,
  recouvrement réglable) et 38-1 (PATCH, `accountEmail`).
- 2026-11-15 — alignement inter-fiches P1 (Opus) : AC-4/AC-5 vérifiés contre O11 (`refund_origin_missing` calculé,
  `overlapDays`, appariement par `paypal_reference_id`) — un résidu corrigé à l'AC-2 (« posé par 38-6b » → écrit nulle
  part, calculé par 38-6b) ; découpage D-P1-1 : renvois à 38-3 portés sur 38-3b (`ProviderMovement`, `MovementPage`,
  passage, fabrique, `overlap()`, `booking_date`, contrôle des soldes) ou 38-3a (colonnes, format d'`item_names`,
  décodage tolérant).
- 2026-11-15 — validation P2 (Opus) : 0 CRITICAL, 1 HIGH, 1 MEDIUM, 2 LOW ; remédiés dans la fiche sauf l'inter-fiche
  38-3b (unicité de `provider_tx_id` par devise, portée par la P2 de 38-7). P2-1 (HIGH) : la table classait T0108,
  T0109, T0110 en `fee` sur une citation inexacte de la page PayPal (relue par WebFetch : « ATM withdrawal »,
  « Auto-sweep from account », « International CC withdrawal ») — un retrait passé en charge → T0109 `withdrawal`,
  T0108 et T0110 `other` ; P2-2 : `item_names` sans borne, alors que `validate_page` de 38-3b rejette la page au-delà
  de 50 noms de 255 caractères → bornés et tronqués avec avertissement ; LOW : `SkipCounts` → `SkippedCounts` et
  `Classified` porte `sums` ; compteur d'une surcharge `ignore` nommé (`non_balance`). Grep du symptôme
  (`T0108`, `T0110`, `SkipCounts`) fait dans la fiche ; l'entrée P1 ci-dessus, relevé daté, n'est pas réécrite.
- 2026-11-15 — alignement inter-fiches P2 (Opus) : AC-5, paire coupée — elle garde `review_reason = 'other'` comme
  toute moitié sans candidat (la phrase « 38-6b la met à la file, calculé » laissait croire à un motif non stocké) ;
  38-6b ignore ce motif dès que l'autre moitié s'y relie (C-38-6b-4).
- 2026-11-15 — validation P3 (Sonnet) : 4 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW) ; tous remédiés dans la
  fiche. MEDIUM : P3-1 AC-7 — « seuls les codes classés `other` se surchargent » ne disait pas le cas des codes
  `other` **sous condition de signe** (T0300 à montant négatif…), classés `other` pour une transaction et `deposit` pour
  une autre → seules les lignes `other` sans condition ; P3-2 AC-7 — `review_reason` d'un mouvement surchargé non dit
  (`unknown_event` aurait pu survivre à une surcharge) → table des effets, `deposit`/`withdrawal` gardent la contrainte
  de signe. LOW : P3-3 `ignore` sur un code qui change le solde laisse un écart durable au contrôle de complétude →
  dit (38-9a, manuel) ; P3-4 AC-8 — `chrono-tz` listé sans usage (la date comptable est de 38-3b) retiré, et renvoi
  périmé « texte à porter à 38-3b » de l'AC-1. Vérifié : aucun renvoi à 38-6c1/38-6c2 ni D-P2-5 dans cette fiche ;
  `PaypalSettings` (`deny_unknown_fields`, `eventKindOverrides`, `overlapDays`) conforme à 38-1 AC-4 ; colonnes
  (`invoice_ref VARCHAR(127)`, `item_names`, `review_reason`) conformes à 38-3a ; `details TEXT NOT NULL` à
  `20260504000001_bank_imports.sql:64`. Axes non exercés : table des codes non reconfrontée à la page PayPal en P3
  (relue en P2) ; aucune exécution.
- 2026-11-15 — validation P4 ciblée (Opus) : 1 finding (0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW), braqué sur la
  remédiation P3 (`git diff f005ba2f a5682b2e`) ; remédié dans la fiche. LOW : P4-1 AC-7 — l'ordre entre une
  surcharge et les contrôles qui suivent la table n'était pas dit : les codes surchargeables T0014–T0017, T0020, T0021
  sont des T00xx, que l'AC-5 range `other` quand la contrepartie est une autre connexion ; une surcharge `sale` aurait
  pu en faire une vente → la surcharge remplace la ligne de table seulement, AC-3 et AC-5 la priment ; test ajouté
  (AC-9). Vérifié conforme : la liste des lignes surchargeables recoupe exactement les lignes `other` sans condition de
  l'AC-2 (T0109, T0300–T0303, T0400–T0403, T0600, T0700/T0701 exclues) ; effets cohérents avec l'AC-2, l'AC-3 et
  l'AC-5 (`dispute` surchargé relié par `paypal_reference_id`) ; « contrat porté par 38-3b AC-2 » exact (`MovementPage`
  à l'AC-2 de 38-3b) ; retrait de `chrono-tz` sans effet (38-8 n'écrit que l'instant UTC ; 38-3b AC-2 déclare la
  dépendance au crate) ; l'avertissement `ignore` est porté par 38-9a (lignes 94-97). Axes non exercés : manuel (écrit
  par 38-9a) ; page PayPal non relue (aucun réseau) ; aucune exécution.
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : section *Note de version* ajoutée ;
  compteurs « recomptés après chaque reprise de `main` ». Aucun PDF ni CHANGELOG commité dans cette fiche.
