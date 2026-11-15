# Story 40-1 : client Stripe — connexion, clé restreinte, version épinglée, « Tester », transactions de solde, soldes

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/40-stripe`, numéros de ligne relevés au commit `a2b1da9c`. Epic 40
(`planning-artifacts/epic-40-stripe.md`) : G3, G4, G5, G7 (contrôles de forme), G15, G16, G17 ; contrats **C-40-0**,
**C-40-1**, **C-40-2** (a, e, f), **C-40-3**, C-40-4 (lecture), C-40-6 ; décision **P40-1** de l'orchestrateur (clés
`rk_live_` **et** `rk_test_` admises, `sk_` refusée, mode montré et contrôlé). Décisions de Beat B10, B11, B12, B13, B14.
**Dépend de** l'Epic 38 développé et fusionné — 38-1 (coffre, connexions, `settings` typé, `PUT …/secrets`, `POST
…/test`), 38-3b (`kesh-providers`, `Connector`, `Transport`, `HttpTransport`, `ConnectorFactory`, `ConnectorContext`,
`validate_page`, passage, route `_test/provider-connector`), 38-7 (patron du connecteur : `with_base_url`, découpage
O10, table d'erreurs AC-7, route `POST …/test`, jeu `"default"`) — et de **40-0** (recette en mode réel de Beat,
constats écrits au fichier d'epic). **Suivie de** 40-2 (classement), 40-3 (versements). **Aucune migration**.*

*Modules : `kesh-providers` (module `stripe`), `kesh-core` (`providers::settings`), `kesh-api` (secrets, fabrique,
route `test`, variante `_test`), `kesh-i18n` — quatre (≤ 5).*

## Story

En tant qu'**Administrateur** d'une société qui encaisse par Stripe en CHF, EUR et USD, je veux créer une connexion
Stripe, y déposer une **clé restreinte en lecture seule** (jamais réaffichée, une clé secrète `sk_…` refusée), voir si
elle est **de test ou réelle**, et que « Tester la connexion » me dise si Kesh lit bien les soldes et les transactions
de solde — et sinon pourquoi —, afin que le passage de téléchargement (38-3b) dispose d'une source Stripe bornée,
paginée sans perte, à version d'API fixe et sans fuite de la clé.

## Source (relevée le 2026-11-15)

- **Authentification** (<https://docs.stripe.com/api/authentication>) : « *The Stripe API authenticates requests using
  HTTP Basic Auth. Provide your API key as the basic auth username value* » ; « *If you need to authenticate using bearer
  auth […], use `-H "Authorization: Bearer <<YOUR_SECRET_KEY>>"`* » ; « *Calls made over plain HTTP fail* ». Préfixes :
  `sk_test_`, `sk_live_` (accès complet), `rk_live_` (clé restreinte) ; des clés restreintes se créent aussi en mode test
  (`rk_test_`, <https://docs.stripe.com/keys/restricted-api-keys>).
- **Version** (<https://docs.stripe.com/api/versioning>) : sans en-tête `Stripe-Version`, une requête prend « *your
  Stripe account’s default API version* » ; version courante relevée : `2026-10-28.endive`.
- **Liste** (<https://docs.stripe.com/api/balance_transactions/list>) : « *The transactions return in sorted order, with
  the most recent transactions appearing first* » ; `created` (intervalle), `currency`, `payout`, **`source`** (« *Only
  returns transactions associated with the given object* »), `type` ; `limit` « *between 1 and 100* » ;
  `starting_after` / `ending_before` (curseurs = identifiants d'objet) ; réponse `{ object: "list", data, has_more, url }`.
- **Expansion** (<https://docs.stripe.com/api/expanding_objects>) : sur une liste, chemin `data.<champ>`, « *up to a
  maximum depth of four levels* ».
- **Soldes** (<https://docs.stripe.com/api/balance/balance_object>) : `available[]`, `pending[]` (`amount`, `currency`,
  `source_types`), `livemode` ; aucun solde à une date passée.
- **Compte** (<https://docs.stripe.com/api/accounts/retrieve>) : objet `account`, `id` `acct_…`.
- **Erreurs** (<https://docs.stripe.com/api/errors>) : 400, 401, 402, 403, 404, 409, 424, 429, 5xx ; corps `{ error: {
  type, code, message, param } }`.

## Ce que les réponses de test montrent (`data-not-committed/stripe-test-samples/`, lu le 2026-11-15)

- `balance_transactions.json` : `{ object, data, has_more: false, url }` ; chaque transaction porte `id`, `amount`,
  `available_on`, `balance_type` (`payments`), `created`, `currency` (minuscules), `description`, `exchange_rate`
  (`null`), `fee`, `fee_details[] { amount, application, currency, description, type }`, `net`, `object`,
  `reporting_category`, `source` (objet développé), `status` (`available` | `pending`), `type`. ⚠️ **La transaction de
  solde n'a pas de champ `livemode`** ; la liste non plus.
- Sources développées : `charge` (`livemode`, `description`, `metadata`, `billing_details { email, name }`,
  `receipt_email`, `balance_transaction`, `payment_intent`), `refund` (**sans** `livemode` ; `charge`,
  `balance_transaction`), `dispute` (`du_…`, `livemode`, `charge`, `balance_transactions[]`), `payout` (`livemode`,
  `amount` positif, `arrival_date`, `balance_transaction`, `failure_balance_transaction`, `original_payout`, `status`).
- `balance.json` : `available[]` et `pending[]` en trois devises, `livemode: false`,
  `refund_and_dispute_prefunding` (nul) ; Σ `net` des transactions = `available + pending` par devise (G5).

## Exigences

- `FR-40-1-1` — Créer une connexion `stripe` avec ses réglages (C-40-1), validés par `StripeSettings` → AC-2, AC-3.
- `FR-40-1-2` — Déposer la clé restreinte, jamais réaffichée ; une clé qui n'est pas restreinte (`sk_…`, `pk_…`, autre)
  est refusée (`PROVIDER_SECRET_NOT_RESTRICTED`) ; `rk_live_` **et** `rk_test_` admises (P40-1) → AC-4.
- `FR-40-1-3` — « Tester la connexion » lit les soldes `available + pending` par devise, le mode, le compte et le droit de
  lecture des transactions ; une devise sans compte Kesh est listée (B2) → AC-9.
- `FR-40-1-4` — Lire les transactions de solde d'une fenêtre de `created`, toutes les pages du curseur, en une
  `MovementPage` → AC-6, AC-7.
- `FR-40-1-5` — Lire le solde d'une devise à un instant passé, par sommation (G5) → AC-8.
- `FR-40-1-6` — Chaque erreur Stripe devient un code lisible (C-40-3) ; la fabrique construit un `StripeConnector` pour
  `stripe`, sur le transport factice `stripeResponses` en mode test → AC-10, AC-11.
- `FR-40-1-7` *(ajoutée)* — Le mode (test ou réel) de la connexion se fixe au premier « Tester » réussi ; une clé de
  l'autre mode est refusée tant que la connexion a des transactions téléchargées → AC-5, AC-9.
- `NFR-40-1-1` — URL de base constante, client borné de 37-2, bornes de 38-3b par requête, plafonds de requêtes par
  fenêtre et par lecture de solde → AC-1, AC-7, AC-8.
- `NFR-40-1-2` — Clé absente de tout journal, erreur, réponse, audit, export et sauvegarde (test au marqueur) → AC-12.
- `NFR-40-1-3` — `Stripe-Version` épinglée sur chaque requête ; `livemode` contrôlé sur chaque objet qui le porte → AC-1,
  AC-5.
- `NFR-40-1-4` — Validation stricte ; un test par refus, réponses hostiles comprises (C-38-11) ; PayPal inchangé → AC-6,
  AC-13.

## Écarts à l'epic

1. **Clé de test admise hors mode test (P40-1).** G4, FR-40-1-2 et C-40-3 réservent `rk_test_` à `KESH_TEST_MODE` ;
   l'orchestrateur a décidé (P40-1) que `rk_test_` est admise partout — le compte de test de Beat sert à la recette —,
   le mode étant montré et contrôlé. `PROVIDER_SECRET_NOT_RESTRICTED` ne vise donc plus que les clés non restreintes.
   *Garde ajoutée en contrepartie* (FR-40-1-7, AC-5) : une connexion ne change pas de mode après son premier
   téléchargement — sans elle, des transactions de test et réelles se mêleraient dans les mêmes comptes.
2. **`livemode` « sur chaque réponse » (NFR-40-1-3)** : la transaction de solde et la liste n'ont **pas** de champ
   `livemode` (constat ci-dessus) ; le contrôle porte sur chaque objet qui en porte un — solde, compte, sources
   développées (`charge`, `payout`, `dispute`, …) — et sur le préfixe de la clé (AC-5). Un remboursement (`refund`, sans
   `livemode`) n'est contrôlé que par sa transaction voisine et le solde lu au même passage.
3. **`ConnectorTest.livemode`** : champ additif `Option<bool>` (PayPal : `None`) — amendement du socle non listé à C-40-2,
   nécessaire pour poser le mode aux réglages sans transtypage du connecteur (même raison que `test()` en 38-3b AC-2).
4. **Classement** : en 40-1, `classify_window` (40-2) n'existe pas ; `movements` rend chaque transaction classée
   `other` / `unknown_event` (AC-7) — défaut sûr (aucune proposition), remplacé par 40-2. La table des exposants de
   devise (G7) est posée **ici**, parce que les soldes (AC-8) en ont besoin ; 40-2 la réutilise.
5. **`value_date` (C-40-2 a)** : la règle de `validate_page` admet `value_date ≥ date Zurich de booked_at − 1 jour`
   (et non `≥` strict) : `arrival_date` est une date UTC (G10) ; un versement créé entre minuit et 1 h (2 h en été) à
   Zurich a une date UTC antérieure d'un jour, et une égalité stricte rejetterait toute la page à chaque passage.

## Critères d'acceptation

1. **AC-1 — Requête Stripe et bornes.** Module `kesh_providers::stripe` :
   - `LIVE_BASE_URL = "https://api.stripe.com"` (aucune variable d'environnement, C-37-6) ;
     `StripeConnector::with_base_url(…)` `#[doc(hidden)] pub` pour un serveur axum local (patron 38-7 AC-2).
   - `API_VERSION: &str = "2026-10-28.endive"` (constante, G4 ; **à confirmer à 40-0** : les fixtures se relèvent avec
     cet en-tête, sinon la constante prend la version des fixtures) ; envoyée en `Stripe-Version` sur **chaque** requête ;
     un test vérifie l'en-tête sur chaque chemin (`/v1/balance`, `/v1/account`, `/v1/balance_transactions`).
   - Authentification `Authorization: Bearer <clé>` (doc ci-dessus), construite à partir du `Zeroizing<String>` remis par
     la fabrique, jamais gardée au-delà du connecteur ; aucune autre en-tête d'identité.
   - Requêtes `GET` seulement (le connecteur n'a aucune méthode d'écriture : un test de source vérifie qu'aucun `POST`,
     `DELETE` ni chemin hors des trois ci-dessus n'est construit dans `stripe/`).
   - Bornes par requête (`TransportLimits`, 38-3b AC-3) : délai total **30 s**, corps ≤ **8 MiB** (100 transactions à
     sources développées ≈ 1 Mo relevé sur les fixtures) ; délai de connexion du client (5 s, 37-2).
   - Paramètres de requête en **liste de paires** (clé répétable : `expand[]`). Si le `Transport` de 38-3b les porte en
     table à clés uniques, 40-1 l'amende en liste de paires (additif ; PayPal inchangé, NFR-40-1-4).
2. **AC-2 — `Provider::Stripe` et `StripeSettings`** (`kesh-core::providers::settings`, E24, C-40-1).
   - `Provider` gagne `Stripe` (`as_str` `"stripe"`, `FromStr` tolérant inchangé) ; `ProviderSettings::Stripe(StripeSettings)` ;
     `parse_settings` le lit ; `stripe` entre aux prestataires connus de `POST /provider-connections` (38-1 AC-4 : sinon
     `PROVIDER_UNKNOWN`).
   - `StripeSettings` `#[serde(deny_unknown_fields, rename_all = "camelCase")]`, champs tous `Option` +
     `#[serde(default)]` :
     - modifiables par le client : `overlap_days: Option<u8>` (1 à 30, défaut 3 ; sinon `PROVIDER_SETTINGS_INVALID`,
       `reason: "out_of_range"`, forme de 38-8 AC-7) ; `invoice_ref_source: Option<String>` (G14 : exactement
       `description`, ou `metadata:<clé>` avec `<clé>` `^[A-Za-z0-9_.-]{1,40}$` ; défaut **`metadata:invoice`** ; sinon
       `PROVIDER_SETTINGS_INVALID`, `field: "invoiceRefSource"`, `reason: "invalid_format"`). `eventKindOverrides` est
       ajouté par **40-2** (même forme que PayPal, codes courts) ;
     - **posés par le serveur** (refusés du client comme `payerId`, 38-1 AC-4 : `PROVIDER_SETTINGS_INVALID`,
       `details.field`) : `account_id: Option<String>` (`acct_…`, AC-9), `api_version: Option<String>` (égale à
       `API_VERSION`, écrite par « Tester »), `livemode: Option<bool>` (AC-5, AC-9).
   - `PATCH` (38-1 AC-4) : remplace les champs du client, **conserve** les trois champs du serveur ; aucune règle de remise
     à `null` (au contraire de `clientId` PayPal : la clé n'est pas un réglage ; le mode se garde par l'AC-5).
   - `history_start` : `None` (Q-40-7) ; `kesh_providers::earliest_fetch_from("stripe", now)` → `None` (38-7 AC-9) ;
     `earliestFetchFrom` rendu `null`.
   - Tests : chaque refus (clé inconnue, `accountId`/`apiVersion`/`livemode` fournis par le client, `overlapDays` 0 et
     31, `invoiceRefSource` `metadata:` vide, `metadata:a b`, `autre`), défauts, aller-retour de sérialisation, PATCH qui
     garde les champs du serveur.
3. **AC-3 — Créer une connexion Stripe.** `POST /api/v1/provider-connections` `{ provider: "stripe", label, fetchFrom,
   schedule?, settings? }` (route, rôle Administrateur session, audit `payment_provider_connection.created`, codes :
   38-1 AC-3/AC-4, inchangés) ; réponse `201` avec `settings` résérialisé (défauts écrits). Aucune route neuve
   (C-40-6 : `LIB_ROUTES` inchangé). Test HTTP : création, refus `PROVIDER_SETTINGS_INVALID`, connexion relue.
4. **AC-4 — Déposer la clé restreinte (FR-40-1-2, G4, P40-1).** `PUT /api/v1/provider-connections/{id}/secrets`
   (38-1 AC-6 : extracteur `impl_validation_extractor`, `crates/kesh-api/src/extractors.rs:18` ; `seal`, AAD, verrou,
   audit, réponse `200 { secretSet, secretUpdatedAt, version }`, version inchangée), corps Stripe
   `#[serde(deny_unknown_fields)]` `{ "restrictedKey": "<chaîne>" }`, nom au coffre **`restricted_key`**, audit
   `secrets_updated` `{ names: ["restricted_key"] }`. Contrôles, **dans cet ordre**, jamais la valeur dans `details` ni au
   journal :
   - préfixe `sk_live_`, `sk_test_` → `400 PROVIDER_SECRET_NOT_RESTRICTED`, `details: { field: "restrictedKey", reason:
     "secret_key" }` ; `pk_` → `reason: "publishable_key"` ; tout autre préfixe que `rk_live_` / `rk_test_` →
     `reason: "unknown_prefix"`. Variante d'`AppError` neuve (statut 400), message aux quatre locales ;
   - suite du préfixe : 1 à 247 caractères `[A-Za-z0-9]` (longueur totale ≤ 255) ; gabarit (`is_template_placeholder`,
     `crates/kesh-api/src/config.rs:1538`, `changeme`) → `400 PROVIDER_SECRETS_INVALID`, `details.field` (38-1 AC-6).
   - **Secrets exigés par prestataire (C-40-2 f)** : si 39-1 est livré, `stripe → ["restricted_key"]` s'inscrit dans sa
     liste (C-39-2, F2) ; sinon 40-1 la pose **dans la forme de C-39-2** — fonction `required_secrets(provider) ->
     &'static [&'static str]` (`paypal → ["client_secret"]`, `stripe → ["restricted_key"]`) lue par les contrôles
     `secret_present` / `secret_decryptable` de 38-1 AC-7 et par la fabrique ; et `ConnectorContext` porte les secrets
     **nommés** (`secrets: BTreeMap<&'static str, Zeroizing<String>>`, F2), PayPal lisant `client_secret` sans changement
     de comportement.
   - Tests : `sk_live_…`, `sk_test_…`, `pk_live_…`, `whsec_…` refusés (raison juste, valeur absente du corps de réponse et
     de la sortie `tracing` capturée) ; `rk_live_…` et `rk_test_…` admis ; caractère interdit ; 256 caractères ; gabarit ;
     clé en trop (`clientSecret`) → `PROVIDER_SECRETS_INVALID`.
5. **AC-5 — Le mode : lu, montré, contrôlé (P40-1, NFR-40-1-3, FR-40-1-7).**
   - **Mode de la clé** : `rk_live_` → réel, `rk_test_` → test (fonction pure `key_mode(&str) -> Option<Livemode>`,
     testée). Le connecteur le reçoit à sa construction.
   - **Chaque objet qui porte `livemode`** (solde, compte, sources développées) doit l'avoir égal au mode de la clé ;
     sinon `InvalidResponse { reason: "livemode" }`, la page entière rejetée (Écart 2).
   - **Mode de la connexion** : `StripeSettings.livemode` est posé par le premier « Tester » réussi (AC-9). Ensuite, si le
     mode de la clé déposée diffère de `livemode` :
     - « Tester » : contrôle `mode_unchanged` en échec, `errorCode = PROVIDER_MODE_CHANGED`, **sauf** si aucune
       transaction n'a encore été téléchargée pour la connexion (aucune ligne `bank_transactions` dont le compte porte
       une ligne `payment_provider_accounts` de la connexion) : le mode est alors réécrit (version + 1, audit `tested`) ;
     - passage (38-3b) : la fabrique compare avant tout appel ; différence → le connecteur rend `InvalidResponse {
       reason: "livemode" }` sans appel réseau (passage `failed`, `error_code = invalid_response`, `error_detail =
       "livemode"`, `lastError = PROVIDER_RESPONSE_INVALID`).
     Changer de mode après un téléchargement demande une **connexion neuve** (dit au manuel par 40-6).
   - `PROVIDER_MODE_CHANGED` : chaîne constante de `errorCode`/`lastError` (non une `AppError`, comme les codes de 38-7
     AC-7), message `provider-error-*` aux quatre locales.
   - Tests : clé `rk_test_` et solde `livemode: true` → refus ; source `charge` `livemode: true` dans une page de clé
     de test → page refusée ; changement de clé `rk_test_` → `rk_live_` sans transaction → mode réécrit ; avec une
     transaction → `mode_unchanged` en échec, passage refusé sans appel (compteur du serveur local à 0).
6. **AC-6 — Lecture d'une page de transactions de solde et validation (G3, G7, C-38-11).**
   - Requête : `GET {base}/v1/balance_transactions?created[gte]=<s0>&created[lt]=<s1>&limit=100&expand[]=data.source`
     (+ `starting_after=<id>` aux pages suivantes) ; `s0 = ⌈from⌉`, `s1 = ⌈to⌉` en secondes Unix (`created` est entier :
     `[from, to)` ⇔ `[⌈from⌉, ⌈to⌉)`), sans filtre `currency` (une lecture sert les trois soldes, G3).
   - Réponse : objet, `object = "list"`, `data` tableau ≤ 100, `has_more` booléen ; sinon `InvalidResponse { reason:
     "page" }`. Champs inconnus ignorés (pas de `deny_unknown_fields` sur les réponses, patron 38-7).
   - **Forme validée `StripeRecord`** (noms fixés ici, consommés par 40-2) : `id`, `type_`, `reporting_category:
     Option`, `amount: i64`, `fee: i64`, `net: i64`, `currency` (passée en **majuscules**), `created: DateTime<Utc>`,
     `status` (`available` | `pending`), `balance_type: Option<String>`, `exchange_rate: Option<Decimal>`,
     `description: Option`, `fee_details: Vec<StripeFeeDetail { amount: i64, currency, type_ }>`, `source:
     Option<StripeSource>` ; `StripeSource { id, object, livemode: Option<bool>, description, metadata:
     BTreeMap<String, String>, billing_email, billing_name, receipt_email, charge: Option<String>,
     balance_transaction: Option<String>, payout: Option<StripePayoutFields { amount: i64, currency, arrival_date:
     NaiveDate, status, automatic }> }` (les champs d'un objet `payout` ; `charge` d'un `refund` ou d'un `dispute`, gardé
     comme identifiant, non développé). La réponse brute n'est **jamais** gardée.
   - **Règles** (une violation rejette la page : `InvalidResponse { reason }`, motif constant, jamais la valeur ; rang et
     page au `tracing::warn!` seulement) : `id` `^txn_[A-Za-z0-9]{1,60}$` (`"record.id"`) ; `type` et `reporting_category`
     `^[a-z_]{1,64}$` ; `currency` `^[a-z]{3}$` et connue de la table des exposants (AC-8 ; inconnue ou à trois décimales →
     `"record.currency"`) ; `amount`, `fee`, `net` entiers, `|x| < 10^15` ; **`net == amount − fee`** (`"record.net"`) ;
     `fee_details` ≤ 20 éléments, chaque `currency` = celle de la transaction, **`Σ fee_details.amount == fee`**
     (`"record.fee_details"`) ; `created` entier dans `[s0, s1)` (`"record.created"` — Stripe filtre exactement : hors
     intervalle = réponse hostile) ; `status` hors `available|pending` → `"record.status"` ; `exchange_rate` nul ou
     décimal > 0 ; textes > 2000 caractères, courriel > 254, `metadata` > 50 clés ou valeur > 500 → `"record.text"` ;
     `source` **chaîne** (non développée) → `"record.source"` (droit d'expansion manquant : jamais un mouvement sans ses
     champs) ; `source` nulle admise ; `source.id` `^[a-z]{2,4}_[A-Za-z0-9]{1,60}$` ; `source.livemode` ≠ mode de la clé →
     `"livemode"` (AC-5) ; **versement** (`type = payout`, source `payout`) : `source.amount == −amount` et même devise,
     sinon `"record.payout"` (G10) ; `arrival_date` entier lu comme **date UTC** (minuit UTC observé).
   - Tests hostiles : un par règle ; `has_more` absent ; `data` non tableau ; 101 éléments.
7. **AC-7 — Une fenêtre entière, curseur suivi dans le connecteur (G3, O10, FR-40-1-4, NFR-40-1-1).**
   `StripeConnector::movements(window, page)` (forme de 38-3b AC-2) :
   - fenêtre refusée **avant tout appel** si `to − from > max_window()` ou `from ≥ to` → `InvalidWindow` ; `page ≥ 2` →
     `InvalidWindow` ;
   - lit **toutes** les pages : `starting_after` = `id` du **dernier** élément de la page précédente, jusqu'à `has_more =
     false` ; `has_more: true` avec `data` vide, un `id` déjà vu, ou un `created` croissant d'un élément au suivant (la liste
     est décroissante) → `InvalidResponse { reason: "pagination" }` ; rien n'est rendu tant que la fenêtre n'est pas lue ;
   - au-delà de **100 requêtes** ou de `max_movements_per_page()` = **10 000** enregistrements → la fenêtre se coupe en deux
     moitiés `[from, mid)`, `[mid, to)` relues récursivement (patron 38-7 AC-5) ; sous **une heure** →
     `WindowTooLarge { total_items: None }` ; aucune transaction lue deux fois ni perdue (test à la frontière exacte) ;
   - enregistrements reclassés en ordre **croissant** `(created, id)`, puis remis au classement : en 40-1,
     `classify_window` provisoire (Écart 4) qui rend chaque enregistrement en `ProviderMovement { provider_tx_id = id,
     booked_at = created, currency, gross = amount, fee = −fee, net = net` (exposant, AC-8), `movement_kind` = genre `other` (C-38-4, valeur de 38-3b telle que 38-8 la rend),
     `review_reason = UnknownEvent`, `event_code = "OTHER"`, `value_date = None`, textes `NULL`, `details = "Stripe <type>"` }` ;
     une transaction dont `balance_type` est posé et ≠ `payments` est comptée `skipped.non_balance` et non rendue (G9) ;
     `MovementPage { movements, page: 1, total_pages: 1, skipped, sums }` (`sums` = Σ `net` par devise des mouvements
     rendus) ;
   - `max_window()` 31 jours ; `history_start(now)` `None` ; `overlap()` = `overlapDays` (défaut 3 jours) ;
     `max_movements_per_page()` 10 000.
   - **C-40-2 (a), type** : `ProviderMovement` gagne `value_date: Option<NaiveDate>` (`None` pour PayPal et ici) ;
     `validate_page` (38-3b AC-2) le contrôle s'il est posé : `≥ date Zurich de booked_at − 1 jour` (Écart 5) et
     `≤ date Zurich de booked_at + 30 jours`, sinon `InvalidResponse { reason: "value_date" }` ; tests au `FakeConnector`
     (bornes incluses, un jour de moins, trente et un de plus). L'écriture en base est de **40-3**.
   - Tests (serveur local) : trois pages (`has_more` vrai, vrai, faux) → une page rendue, ordre croissant, curseurs
     exacts ; 101ᵉ requête → découpage ; fenêtre d'une heure trop pleine → `WindowTooLarge` ; `created` de la frontière
     (`= s1`) refusé, (`= s0`) admis ; transaction `balance_type: issuing` → `skipped.non_balance = 1`.
8. **AC-8 — Soldes (G5, FR-40-1-5).**
   - **Exposants** (`stripe::amounts`, table constante, réutilisée par 40-2) : CHF, EUR, USD et les devises à deux
     décimales de la liste Stripe relevée à 40-0 → 2 ; devises à zéro décimale (`jpy`, `krw`, … : liste de Stripe recopiée
     avec sa source) → 0 ; trois décimales ou inconnue → `InvalidResponse { reason: "currency" }` ;
     `minor_to_decimal(i64, devise) -> Decimal`, testée.
   - `balances(None)` : `GET {base}/v1/balance` ; validation : `livemode` (AC-5), `available` et `pending` tableaux
     ≤ 50, devises `^[a-z]{3}$` **uniques dans chaque tableau**, montants entiers bornés ; `connect_reserved`,
     `refund_and_dispute_prefunding`, `issuing`, `instant_available` **ignorés** ; par devise de l'union : `amount =
     available + pending` (absente d'un tableau = 0) ; `as_of` = instant de la lecture (horloge injectée).
   - `balances(Some(as_of))` : `T0` = horloge juste **avant** la lecture du solde ; solde courant (ci-dessus) − Σ `net` des
     transactions `created ∈ [⌈as_of⌉, ⌈T0⌉)` lues par le même parcours de curseur (AC-7, sans expansion :
     `limit=100`, pas d'`expand`), **hors** `balance_type` ≠ `payments` ; au plus **50 requêtes**, sinon
     `WindowTooLarge { total_items: None }` (38-3b AC-7 : motif `balances_unavailable` ou `opening_unavailable`, contrôle
     « non disponible ») ; `as_of` rendu = `as_of` demandé ; une transaction créée entre `T0` et la lecture de la liste est
     hors de la somme (bornée à `T0`), une transaction créée juste avant `T0` mais encore absente du solde fait un écart
     **transitoire**, corrigé au passage suivant (dit au Change Log du développement si observé).
   - Tests : identité G5 sur la fixture (USD 39861, EUR 7715, CHF 4807) ; solde passé à l'instant de la première
     transaction ; 51ᵉ requête → `WindowTooLarge` ; devise en double ; devise inconnue ; `livemode` faux.
9. **AC-9 — « Tester la connexion » (FR-40-1-3, FR-40-1-7).** `POST /api/v1/provider-connections/{id}/test` (38-1 AC-7,
   38-7 AC-8 : forme `200 { usable, unusableReason, checks, … }`, toujours 200, Administrateur session, connecteur par la
   fabrique, `Connector::test()`). Pour `stripe`, après les contrôles locaux : `mode_unchanged` (AC-5, sans réseau), puis
   `StripeConnector::test()` :
   - `provider_reachable` : `balances(None)` ;
   - `permission_present` : `GET /v1/balance_transactions?limit=1&expand[]=data.source` (403 → `Permission`) — le droit
     de lire les transactions et d'en développer la source se constate ici, non au premier passage ;
   - compte : `GET {base}/v1/account` → `account_ref = id` (`^acct_[A-Za-z0-9]{1,60}$`) ; **403 ou 404 n'échouent pas**
     le test (`account_ref = None` : la clé n'a pas ce droit, Q-40-8) ; 401 → `Auth`.
   - `ConnectorTest { account_ref, balances, livemode: Some(mode) }` (Écart 3). Un contrôle en échec porte `errorCode`
     (table de l'AC-10), les suivants `ok: null`.
   - Réponse : `accountId` (ou `null`), `livemode`, `apiVersion`, `balances: [{ currency, total }]` (lus, non enregistrés),
     `version`. Succès : `accountId`, `apiVersion` (= `API_VERSION`) et `livemode` écrits aux réglages **s'ils
     diffèrent**, version + 1, dans la transaction de l'audit ; `last_error` remis à `NULL`. Échec : `last_error` = code.
   - Audit `payment_provider_connection.tested` (38-7 AC-8, libellé existant) : `details` `{ result, accountIdChanged,
     livemode, currencies }` — ni clé ni montant.
   - Devises lues sans compte Kesh : listées à la réponse ; leur écriture à `provider_balances_seen` reste l'affaire du
     passage (38-3b AC-7, E12, B2).
   - Tests HTTP : succès qui pose les trois champs ; second succès sans changement (rien d'écrit, audit seul) ; 401 →
     `PROVIDER_AUTH_FAILED` ; liste en 403 → `permission_present` faux, `PROVIDER_PERMISSION_MISSING` ; compte en 403 →
     succès, `accountId: null` ; Comptable → 403 ; clé d'API → `403 API_KEY_ADMIN_FORBIDDEN`.
10. **AC-10 — Erreurs (C-40-3, FR-40-1-6).** `ConnectorError` de 38-3b, **aucune variante neuve** ; table : 401 →
    `Auth` ; 403 → `Permission` ; 400, 402, 404, 409 → `Rejected` ; 429 → `RateLimited { retry_after }` (`Retry-After`
    lu, au journal) ; 5xx, 424, délai → `Unavailable`, **un** second essai après 2 s (patron 38-7 AC-7) ; réseau, TLS,
    redirection → `Transport` ; corps invalide ou trop gros → `InvalidResponse`. `error.message`, `error.param`,
    `request-id` : au `tracing::warn!` seulement, jamais à l'écran ni à `last_error`. Codes d'API : ceux de 38-7 AC-7
    (`app_code()`), plus `PROVIDER_SECRET_NOT_RESTRICTED` (AppError, AC-4) et `PROVIDER_MODE_CHANGED` (constante, AC-5).
11. **AC-11 — Fabrique et transport factice (G16, C-40-2 e).**
    - `ConnectorFactory` (38-3b AC-3) : `stripe` → `StripeConnector` (réglages typés, secret `restricted_key` ouvert par la
      voie unique — **aucun appelant d'`open` ajouté**, le site de la fabrique reste le seul —, mode de la clé, transport).
    - **`KESH_TEST_MODE`** : variante additive `stripeResponses` du corps de `POST /api/v1/_test/provider-connector`
      (forme et validation de `paypalResponses`, 38-3b AC-3 : tableau non vide `{ method ∈ GET, path "/…", query?, status
      100–599, body }` servi dans l'ordre, dernière répétée, **ou** exactement `"default"` ; sinon 400
      `AppError::Validation`) ; une connexion `stripe` sans variante garde le `FakeConnector` ; avec elle, un vrai
      `StripeConnector` sur le transport factice. `TEST_ENDPOINT_ROUTES` inchangé (même route).
    - **Jeu `"default"`** (`stripe::fake`, constantes et une fonction `default_responses(now)`) : servi **par chemin** et
      non dans l'ordre — `/v1/balance`, `/v1/account`, `/v1/balance_transactions` (le factice applique lui-même
      `created[gte]`, `created[lt]`, `source`, `limit`, `starting_after` à une liste constante) — afin qu'un E2E (40-6,
      NFR-40-6-3) puisse tester puis télécharger à n'importe quelle date : transactions datées de `now − 20 jours` à
      `now − 4 jours` (hors du délai de 3 h du passage), couvrant vente CHF, EUR et USD (dont une `pending`), remboursement
      partiel à frais rendus, litige, frais `stripe_fee`, versement USD ; solde = Σ `net` par devise ; `livemode: false` ;
      courriels `@example.invalid`.
    - Tests : `"default"` admis, `"autre"` → 400, tableau vide → 400 ; une connexion à variante tourne le vrai connecteur
      (« Tester » puis passage factice → mouvements insérés, `other` en 40-1).
12. **AC-12 — Aucune fuite (NFR-40-1-2).** Test au marqueur (patron 38-1 AC-9, 38-7 AC-11) : clé
    `rk_test_keshsecretmarqueur…` déposée, « Tester » réussi puis échoué (401, 500), passage factice ; ni la clé ni son
    en-tête `Authorization` dans la sortie `tracing` capturée, les corps de réponse, `audit_log`,
    `payment_provider_connections`, l'export CSV, la sauvegarde (`.keshbackup` : le coffre est exclu, 38-1 AC-8) ;
    `Debug` des requêtes du connecteur masque l'en-tête (`"***"`). Une clé `sk_` refusée n'apparaît pas non plus.
13. **AC-13 — Tests sans réseau et non-régression (G16, B12, NFR-40-1-4).**
    - Fixtures **anonymisées** `crates/kesh-providers/tests/fixtures/stripe/` : `balance.json`,
      `balance_transactions.json` (une page, et un jeu de trois pages construit), `account.json`, erreurs 400, 401, 403,
      404, 429, 500 au format `{ error: { type, code, message } }`. Base : les réponses de test de 40-0 ; identifiants
      (`txn_`, `ch_`, `py_`, `re_`, `du_`, `po_`, `pi_`, `ba_`, `acct_`, `cus_`) remplacés de façon **cohérente** (une même
      valeur → une même valeur factice, liens gardés), courriels et noms remplacés, `calculated_statement_descriptor` et
      `metadata` hors `invoice` remplacés ; montants, devises, types, catégories, statuts, dates gardés. Test : aucune
      fixture ne contient `@` hors `example.invalid`, ni `sk_`, ni `rk_` ; les identifiants factices suivent une forme
      reconnaissable (`<préfixe>_kesh<n>`), et le test refuse tout identifiant qui ne l'a pas.
    - Serveur axum local (`with_base_url`, `HttpTransport::for_local_tests()`) : forme exacte de chaque requête (chemin,
      paramètres, `expand[]`, `Stripe-Version`, `Bearer`) ; chaque ligne de la table d'erreurs ; 500 puis 200 ; délai ;
      302 non suivi ; corps de 9 MiB ; `Content-Length` menteur.
    - Suites PayPal (38-7, 38-8, 38-3b) vertes **sans retouche de leurs attentes** (G18) ; si `ConnectorContext` passe aux
      secrets nommés ici (AC-4), seuls leurs constructeurs de test changent, test nommé au Change Log.
14. **AC-14 — Ce que la story ne touche pas, registres, i18n.** Aucune migration (C-40-0 : `provider = 'stripe'` et
    `settings` dans des colonnes existantes à domaine ouvert ; P1–P8 sans objet, ni `TABLES_TO_TRUNCATE`, ni registre de
    rejeu) ; aucune route (`LIB_ROUTES`, `TEST_ENDPOINT_ROUTES` inchangés, C-40-6) ; aucune action d'audit neuve
    (`secrets_updated`, `tested`, `created`, `updated` existent : `crates/kesh-api/src/audit_labels.rs:70`) ; aucun outil
    MCP (G17, E23 : connexions non exposées) ; sauvegarde : la ligne de connexion et ses réglages partent comme toute
    connexion (38-1 AC-8), la clé est effacée à la restauration et se ressaisit (dit au manuel par 40-6) ; aucun écran ni
    manuel (40-6) — les noms de contrôles neufs (`mode_unchanged`, `permission_present` pour Stripe) et les codes
    neufs sont **transmis à 40-6** pour `provider-labels.ts`. **i18n**, quatre locales
    (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`) : message de `PROVIDER_SECRET_NOT_RESTRICTED`
    (AppError) et `provider-error-provider-mode-changed`. CHANGELOG : une ligne de l'entrée non datée de l'epic.

## Tâches

- [ ] T0 — Prérequis : Epic 38 fusionné ; 40-0 tenue (version d'API des fixtures, droits de la clé constatés) ; sinon arrêt.
- [ ] T1 — `kesh-core` : `Provider::Stripe`, `StripeSettings`, validation, `parse_settings`, tests (AC-2).
- [ ] T2 — `kesh-providers::stripe` : constantes, requête, `amounts`, `StripeRecord`/`StripeSource` et validation,
      curseur et découpage, `balances`, `test()`, `key_mode`, `classify_window` provisoire ; `value_date` dans
      `ProviderMovement` et `validate_page` (AC-1, AC-5 à AC-8).
- [ ] T3 — `kesh-api` : corps de secrets Stripe, `PROVIDER_SECRET_NOT_RESTRICTED`, `required_secrets` et secrets nommés si
      39 n'est pas livré, fabrique, route `test` (contrôles, écriture des champs du serveur, audit), garde de mode au passage,
      variante `stripeResponses` et jeu `"default"` (AC-3, AC-4, AC-5, AC-9, AC-11).
- [ ] T4 — i18n quatre locales, CHANGELOG (AC-14).
- [ ] T5 — Fixtures anonymisées et leurs tests, tests hostiles, test au marqueur, non-régression PayPal (AC-12, AC-13).
- [ ] T6 — Gate complet (fmt, clippy `-D warnings`, nextest complet sur base remise à zéro, `scripts/gate-ci.sh`) ;
      `Cargo.lock` commité si une dépendance change (aucune attendue : `zeroize`, `chrono-tz`, `rust_decimal` déjà là).

## Dev Notes

- **Pourquoi la transaction de solde** (G3) : seule source qui dit le net du solde, les frais et la date ; la source
  développée n'apporte que contrepartie, boutique et liens (40-2).
- **Ordre décroissant et curseur** : `starting_after` = dernier `id` de la page, documenté ; le contrôle de monotonie de
  `created` attrape un curseur qui boucle ou une liste réordonnée par une version d'API future.
- **`⌈from⌉`** : le passage peut produire une borne à fraction de seconde (*maintenant − 3 h*) ; arrondir les deux bornes
  vers le haut garde `[from, to)` exact sur des secondes entières, sans trou ni recouvrement entre sous-fenêtres.
- **Délai de 3 h du socle** (38-3b AC-4, « limite non traitée ») : propre à PayPal, sans effet néfaste pour Stripe (les
  transactions y sont immédiates) ; rien n'est changé ici.
- **Solde passé** : 50 requêtes = 5 000 transactions entre `as_of` et maintenant. L'`opening` d'un compte dont
  `fetch_from` est ancien (1.1.2024) et le volume élevé peut rester « non disponible » durablement (38-3b AC-7, relu à
  chaque passage, au plus 50 requêtes par jour en `daily`) — Q-40-1-1.
- **Points de contact** : C-38-5 (`Connector`, fabrique), C-38-2 (routes de 38-1), C-38-1 (`open`) ; 40-2 consomme
  `StripeRecord`, `StripeSource`, `amounts::minor_to_decimal` et remplace `classify_window` ; 40-3 consomme
  `ProviderMovement.value_date` ; 40-6 affiche `livemode`, `accountId`, `apiVersion`, les contrôles et les codes neufs.
- **Lignes relues** (commit `a2b1da9c`) : `crates/kesh-api/src/config.rs:1522, 1538` ; `crates/kesh-api/src/extractors.rs:18` ;
  `crates/kesh-api/src/routes/test_endpoints.rs:349` ; `crates/kesh-api/src/audit_labels.rs:24, 70` ; `Cargo.toml:2-13`
  (dix crates : `kesh-providers` vient de 38-3b). Le reste du socle est **à venir** (Epic 38 non développé) : ses noms
  sont ceux des fiches 38-1, 38-3b, 38-7, citées par AC.
- **Splitting** : quatre modules (≤ 5).

### References

- `planning-artifacts/epic-40-stripe.md` § G3–G7, G15–G17, C-40-0 à C-40-6, 40-1 ; `epics-38-40-cadrage.md` B10–B14.
- `implementation-artifacts/38-1-…` AC-4, AC-6, AC-7, AC-8, AC-9 ; `38-3b-…` AC-2, AC-3, AC-7 ; `38-7-…` AC-1 à AC-12.
- `planning-artifacts/epic-39-wise.md` F2, C-39-2.
- Documentation Stripe citée en *Source*.

## Questions ouvertes

- **Q-40-1-1** (Beat, recette) : combien de transactions de solde par an ? Au-delà de ~5 000 entre `fetch_from` et le jour
  du premier passage, l'`opening` (et donc le contrôle de complétude) reste non disponible ; relever la borne (100
  requêtes ?) est à décider sur ce chiffre.
- **Q-40-1-2** (40-0) : version d'API servie par défaut au compte de Beat, et celle des fixtures ; `API_VERSION` en dépend.
- **Q-40-8** (epic) : droits *Read* minimaux ; ceux qu'exigent `GET /v1/account` et l'expansion des sources.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — rédaction (Opus). 14 AC ; 7 FR (dont 1 ajoutée), 4 NFR ; aucune migration ; cinq écarts à l'epic
  (P40-1 et sa garde de mode, `livemode` absent des transactions de solde, `ConnectorTest.livemode`, classement provisoire,
  tolérance d'un jour sur `value_date`). Documentation Stripe relue (WebFetch) : authentification, versions, liste des
  transactions, expansion, versement, compte.
