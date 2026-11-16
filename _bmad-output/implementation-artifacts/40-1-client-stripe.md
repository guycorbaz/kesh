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
- `FR-40-1-7` *(ajoutée)* — Le mode (test ou réel) de la connexion se fixe au premier « Tester » réussi ou au premier
  passage ; une clé de l'autre mode est refusée tant que la connexion a des transactions téléchargées → AC-5, AC-9 ; de
  même, à « Tester », la clé d'un autre compte Stripe (D40-P2-1, `PROVIDER_ACCOUNT_CHANGED`) → AC-9 ; **le passage lit le compte au début de chaque passage** et applique la
  même règle, une clé qui ne lit pas le compte étant refusée au passage comme à « Tester » (D40-P3-1,
  `PROVIDER_ACCOUNT_UNREADABLE`) → AC-9.
- `NFR-40-1-1` — URL de base constante, client borné de 37-2, bornes de 38-3b par requête, plafonds de requêtes par
  fenêtre et par lecture de solde → AC-1, AC-7, AC-8.
- `NFR-40-1-2` — Clé absente de tout journal, erreur, réponse, audit, export et sauvegarde (test au marqueur) → AC-12.
- `NFR-40-1-3` — `Stripe-Version` épinglée sur chaque requête ; `livemode` contrôlé sur chaque objet qui le porte → AC-1,
  AC-5.
- `NFR-40-1-4` — Validation stricte ; un test par refus, réponses hostiles comprises (C-38-11) ; PayPal inchangé → AC-6,
  AC-13.

## Écarts à l'epic

1. **Clé de test admise hors mode test (P40-1) — écart levé.** À la rédaction, G4, FR-40-1-2 et C-40-3 réservaient
   `rk_test_` à `KESH_TEST_MODE` ; depuis l'alignement inter-fiches P1, l'epic porte P40-1 (G4, C-40-1, C-40-3,
   FR-40-1-2, NFR-40-1-3 et la garde de mode FR-40-1-7 alignés, § *Validation P1* de l'epic) : `rk_live_` et `rk_test_`
   admises partout, `PROVIDER_SECRET_NOT_RESTRICTED` ne visant que `sk_`, `pk_` et les préfixes inconnus. Il ne reste
   aucun écart sur ce point ; la fiche garde le détail de la garde (AC-5) — sans elle, des transactions de test et
   réelles se mêleraient dans les mêmes comptes.
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
6. **`Connector::account_ref` (D40-P3-1)** : méthode additive du trait de 38-3b (défaut `Ok(None)` = aucune garde
   possible ; PayPal et le `FakeConnector` inchangés, leurs suites sans retouche d'attentes), lue **au début de chaque
   passage** et, pour `stripe`, par la route « Tester » **après** `test()` — **un seul** appel, la même méthode, pour les
   deux appelants (AC-9). Raisons : le passage ne dispose que du trait objet, et `test()` lit soldes et droits (deux
   requêtes) là où le passage n'a besoin que du compte ; et `test()` ne rend qu'**une** `ConnectorError` — un 403 du
   compte y serait indiscernable d'un 403 de la liste (`permission_present`), alors qu'un appel distinct dit lequel des
   deux contrôles a échoué sans variante neuve (AC-10). **Amendement de 38-3b AC-4 (Échec)** : `last_error` y porte
   `err.app_code()` ou un code propre du passage ; les deux gardes du passage (mode, AC-5 ; compte, AC-9) y écrivent
   leur constante (`PROVIDER_MODE_CHANGED`, `PROVIDER_ACCOUNT_CHANGED`, `PROVIDER_ACCOUNT_UNREADABLE`) — les mêmes que
   « Tester », libellées une fois (AC-14) ; `error_code` reste un code de `code()` (aucun code de passage neuf).

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
   - tout préfixe `sk_` (`sk_live_`, `sk_test_`, autre : P40-1 refuse `sk_`) → `400 PROVIDER_SECRET_NOT_RESTRICTED`, `details: { field: "restrictedKey", reason:
     "secret_key" }` ; `pk_` → `reason: "publishable_key"` ; tout autre préfixe que `rk_live_` / `rk_test_` →
     `reason: "unknown_prefix"`. Variante d'`AppError` neuve (statut 400), message aux quatre locales ;
   - suite du préfixe : 1 à 247 caractères `[A-Za-z0-9]` (longueur totale ≤ 255) → sinon `400 PROVIDER_SECRETS_INVALID`,
     `details.field` (38-1 AC-6). **Gabarit** : couvert par ces deux contrôles, sans appel distinct — `changeme`,
     `<…>` ou `GENERATE_ME` seuls n'ont pas de préfixe admis (`unknown_prefix`) ; après `rk_live_`/`rk_test_`, `<`, `>`
     et `_` sont hors du jeu `[A-Za-z0-9]` (`PROVIDER_SECRETS_INVALID`). (`is_template_placeholder`,
     `crates/kesh-api/src/config.rs:1538`, ne connaît d'ailleurs que `generate_me` et `<…>`, `config.rs:1522` ;
     `changeme` est un contrôle à côté, 38-1 AC-6.)
   - **Secrets exigés par prestataire (C-40-2 f)** : si 39-1 est livré, `stripe → ["restricted_key"]` s'inscrit dans sa
     liste (C-39-2, F2) ; sinon 40-1 la pose **dans la forme de C-39-2** — fonction `required_secrets(provider) ->
     &'static [&'static str]` (`paypal → ["client_secret"]`, `stripe → ["restricted_key"]`) lue par les contrôles
     `secret_present` / `secret_decryptable` de 38-1 AC-7 et par la fabrique ; et `ConnectorContext` porte les secrets
     **nommés** (`secrets: BTreeMap<&'static str, Zeroizing<String>>`, F2), PayPal lisant `client_secret` sans changement
     de comportement.
   - Tests : `sk_live_…`, `sk_test_…`, `pk_live_…`, `whsec_…` refusés (raison juste, valeur absente du corps de réponse et
     de la sortie `tracing` capturée) ; `rk_live_…` et `rk_test_…` admis ; caractère interdit ; 256 caractères ; gabarits
     `changeme` → `unknown_prefix`, `rk_live_GENERATE_ME` → `PROVIDER_SECRETS_INVALID` ; clé en trop (`clientSecret`) →
     `PROVIDER_SECRETS_INVALID`.
5. **AC-5 — Le mode : lu, montré, contrôlé (P40-1, NFR-40-1-3, FR-40-1-7).**
   - **Mode de la clé** : `rk_live_` → réel, `rk_test_` → test (fonction pure `key_mode(&str) -> Option<Livemode>`,
     testée). Le connecteur le reçoit à sa construction.
   - **Chaque objet qui porte `livemode`** (solde, compte, sources développées) doit l'avoir égal au mode de la clé ;
     sinon `InvalidResponse { reason: "livemode" }`, la page entière rejetée (Écart 2).
   - **Mode de la connexion** : `StripeSettings.livemode`, posé par le premier « Tester » réussi (AC-9) ou par le premier
     passage. **Une seule règle**, fonction pure `mode_decision(stored: Option<bool>, key: bool, has_downloaded: bool) ->
     Keep | Pose | Refuse` (`kesh-api`, module des connexions Stripe, testée sur ses douze combinaisons : trois états du mode stocké × deux modes de clé × transaction téléchargée ou non) : `stored =
     Some(key)` → `Keep` ; `stored = None`, ou `stored ≠ key` sans transaction téléchargée → `Pose` (mode de la clé
     écrit) ; `stored ≠ key` avec transaction téléchargée → `Refuse`. *Transaction téléchargée* : une ligne
     `bank_transactions` dont le compte porte une ligne `payment_provider_accounts` de la connexion. **La comparaison
     n'est faite ni par la fabrique ni par le connecteur** — un connecteur bâti pour refuser rendrait impossible la
     réécriture que « Tester » doit faire quand rien n'est téléchargé ; le connecteur ne contrôle que `livemode` des
     objets contre le mode **de la clé** (puce précédente). Appelants, avant tout appel au connecteur :
     - « Tester » : `Refuse` → contrôle `mode_unchanged` en échec, `errorCode = PROVIDER_MODE_CHANGED`, suivants `ok:
       null`, aucun appel ; `Pose` → le mode s'écrit avec les autres champs du serveur au succès (version + 1, audit
       `tested`, AC-9) ;
     - passage (38-3b) : `Refuse` → passage `failed` **sans appel**, `error_code = invalid_response`, `error_detail =
       "livemode"`, `lastError = PROVIDER_MODE_CHANGED` (aucun code de passage neuf ; même constante qu'à « Tester »,
       Écart 6) ; `Pose` → `livemode` écrit
       dans la transaction qui écrit la première page, version + 1 (rien d'écrit si aucune page ne l'est : le mode reste
       à établir, sans effet puisque rien n'est téléchargé) ; pas d'action d'audit (passage `daily` non audité, C-37-6 :
       le mode se déduit de la clé déposée, auditée par `secrets_updated`), `tracing::info!` sans la clé.
     Tests : passage sans « Tester » → mode posé ; puis clé de l'autre mode → passage refusé sans appel (compteur du
     serveur local à 0) et « Tester » en échec `mode_unchanged` ; clé de l'autre mode sans transaction → « Tester » et
     passage réécrivent le mode.
     Changer de mode après un téléchargement demande une **connexion neuve** (dit au manuel par 40-6).
   - `PROVIDER_MODE_CHANGED` : chaîne constante de `errorCode`/`lastError` (non une `AppError`, comme les codes de 38-7
     AC-7), message `provider-error-*` aux quatre locales.
   - Tests : clé `rk_test_` et solde `livemode: true` → refus ; source `charge` `livemode: true` dans une page de clé
     de test → page refusée ; changement de clé `rk_test_` → `rk_live_` sans transaction → mode réécrit ; avec une
     transaction → `mode_unchanged` en échec, passage refusé sans appel (compteur du serveur local à 0),
     `lastError = PROVIDER_MODE_CHANGED`.
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
     motifs des autres règles : `"record.type"` (`type`, `reporting_category`), `"record.amount"` (bornes),
     `"record.exchange_rate"` ;
     `source` **chaîne** (non développée) → `"record.source"` (droit d'expansion manquant : jamais un mouvement sans ses
     champs ; **sauf** dans les lectures sans expansion de l'AC-8, où la source n'est pas lue) ; `source` nulle admise ;
     `source.id` `^[a-z][a-z0-9]{0,19}_[A-Za-z0-9]{1,200}$` (`"record.source"` ; préfixes réels de longueur variable :
     `ch_`, `pyr_`, `iauth_` — doc *Issuing Authorization*, relue le 2026-11-15 —, `climorder_` ; un préfixe trop
     étroit rejetterait la page entière d'un type que G9 range en `other`) ; `source.livemode` ≠ mode de la clé →
     `"livemode"` (AC-5) ; **versement** (`type = payout`, source `payout`) : `source.amount == −amount` et même devise,
     sinon `"record.payout"` (G10) ; `arrival_date` entier lu comme **date UTC** (minuit UTC observé).
   - Tests hostiles : un par règle ; `has_more` absent ; `data` non tableau ; 101 éléments. Tests admis : sources
     `iauth_…`, `climorder_…`, `pyr_…` (page acceptée).
7. **AC-7 — Une fenêtre entière, curseur suivi dans le connecteur (G3, O10, FR-40-1-4, NFR-40-1-1).**
   `StripeConnector::movements(window, page)` (forme de 38-3b AC-2) :
   - fenêtre refusée **avant tout appel** si `to − from > max_window()` ou `from ≥ to` → `InvalidWindow` ; `page ≥ 2` →
     `InvalidWindow` ;
   - lit **toutes** les pages : `starting_after` = `id` du **dernier** élément de la page précédente, jusqu'à `has_more =
     false` ; `has_more: true` avec `data` vide, un `id` déjà vu, ou un `created` **supérieur** à celui de l'élément précédent, d'une
     page à l'autre comprise (la liste est décroissante ; l'égalité est admise : plusieurs transactions par seconde) → `InvalidResponse { reason: "pagination" }` ; rien n'est rendu tant que la fenêtre n'est pas lue ;
   - au-delà de **100 requêtes** ou de `max_movements_per_page()` = **10 000** enregistrements → la fenêtre se coupe en deux
     moitiés `[from, mid)`, `[mid, to)` relues récursivement (patron 38-7 AC-5) ; sous **une heure** →
     `WindowTooLarge { total_items: None }` ; aucune transaction lue deux fois ni perdue (test à la frontière exacte) ;
     **plafond global** de **1 000 requêtes** par appel de `movements` (toutes sous-fenêtres comprises, relectures du
     découpage comprises), au-delà `WindowTooLarge { total_items: None }` — sans lui, une réponse hostile (`has_more` vrai
     partout) multiplierait les 100 requêtes par le nombre de sous-fenêtres (jusqu'à 744 d'une heure) ; test au serveur
     local ;
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
     décimales de la liste Stripe relevée à 40-0 → 2 (les « cas spéciaux » de Stripe — ISK, HUF, TWD, UGX — gardent
     l'exposant 2, comme la page *Currencies* le dit : à relever à 40-0 et à tester) ; devises à zéro décimale (`jpy`, `krw`, … : liste de Stripe recopiée
     avec sa source) → 0 ; trois décimales ou inconnue → `InvalidResponse { reason: "currency" }` ;
     `minor_to_decimal(i64, devise) -> Decimal`, testée.
   - `balances(None)` : `GET {base}/v1/balance` ; validation : `livemode` (AC-5), `available` et `pending` tableaux
     ≤ 50, devises `^[a-z]{3}$` **uniques dans chaque tableau**, montants entiers bornés ; `connect_reserved`,
     `refund_and_dispute_prefunding`, `issuing`, `instant_available` **ignorés** ; par devise de l'union : `amount =
     available + pending` (absente d'un tableau = 0) ; `as_of` = instant de la lecture (horloge injectée).
   - `balances(Some(as_of))` : `T0` = horloge juste **avant** la lecture du solde ; solde courant (ci-dessus) − Σ `net` des
     transactions `created ∈ [⌈as_of⌉, ⌈T0⌉)` lues par le même parcours de curseur (AC-7, sans expansion :
     `limit=100`, pas d'`expand`), **hors** `balance_type` ≠ `payments` ; validation de l'AC-6 **sauf** les règles de la
     source (chaîne admise, non lue, aucun `livemode` à contrôler sur elle) — test : page sans expansion acceptée,
     somme juste ; au plus **50 requêtes**, sinon
     `WindowTooLarge { total_items: None }` (38-3b AC-7 : motif `balances_unavailable` ou `opening_unavailable`, contrôle
     « non disponible ») ; `as_of` rendu = `as_of` demandé ; une transaction créée **entre `T0` et la lecture du solde** est dans
     le solde mais hors de la somme (bornée à `T0`) : le solde passé rendu est trop fort de son `net` — écart
     **transitoire**, corrigé au passage suivant (l'`opening` se relit, 38-3b AC-7) ; de même une transaction créée juste
     avant `T0` mais encore absente du solde (dit au Change Log du développement si observé).
   - Tests : identité G5 sur la fixture (USD 39861, EUR 7715, CHF 4807) ; solde passé à l'instant de la première
     transaction ; 51ᵉ requête → `WindowTooLarge` ; devise en double ; devise inconnue ; `livemode` faux.
9. **AC-9 — « Tester la connexion » (FR-40-1-3, FR-40-1-7).** `POST /api/v1/provider-connections/{id}/test` (38-1 AC-7,
   38-7 AC-8 : forme `200 { usable, unusableReason, checks, … }`, toujours 200, Administrateur session, connecteur par la
   fabrique, `Connector::test()`). Pour `stripe`, après les contrôles locaux : `mode_unchanged` (AC-5, sans réseau), puis
   `StripeConnector::test()` (deux requêtes) :
   - `provider_reachable` : `balances(None)` ;
   - `permission_present` : `GET /v1/balance_transactions?limit=1&expand[]=data.source` (403 → `Permission`) — le droit
     de lire les transactions et d'en développer la source se constate ici, non au premier passage ; page validée par
     l'AC-6 hors la règle `created` (aucune fenêtre) ;
   - `ConnectorTest { account_ref: None, balances, livemode: Some(mode) }` (Écart 3 ; le compte n'est pas lu par
     `test()`, ci-dessous). Un contrôle en échec porte `errorCode` (table de l'AC-10), les suivants `ok: null`.
   - puis, si `test()` a réussi, contrôle **`account_readable`** : la route appelle **`Connector::account_ref()`**
     (Écart 6 — la **même** méthode que le passage) : `GET {base}/v1/account` → `id` (`^acct_[A-Za-z0-9]{1,60}$`, sinon
     `InvalidResponse { reason: "account" }`) ; **403 et 404 → `Permission`** (D40-P3-1 : une clé qui ne lit pas le
     compte ne peut pas être gardée contre un changement de compte ; elle est **refusée**, comme au passage), 401 →
     `Auth`. Erreur de cet appel → `account_readable` en échec, `usable: false`, aucun champ du serveur écrit,
     `last_error` = son `errorCode` : `Permission` → `PROVIDER_ACCOUNT_UNREADABLE` (constante, patron de
     `PROVIDER_MODE_CHANGED`), toute autre variante → son code de la table de l'AC-10. L'erreur venant d'un appel
     distinct, un 403 du compte ne se confond jamais avec celui de la liste (`permission_present`). Le compte est donc
     toujours connu quand le test réussit (Q-40-8).
   - Réponse : `accountId` (`null` si le compte n'a pas été lu : contrôle antérieur ou `account_readable` en échec), `livemode`, `apiVersion`, `balances: [{ currency, total }]` (lus, non enregistrés),
     `version`. Succès : `accountId`, `apiVersion` (= `API_VERSION`) et `livemode` écrits aux réglages **s'ils
     diffèrent**, version + 1, dans la transaction de l'audit ; `last_error` remis à `NULL`. Échec : `last_error` = code.
   - Audit `payment_provider_connection.tested` (38-7 AC-8, libellé existant) : `details` `{ result, accountIdChanged,
     livemode, currencies }` — ni clé ni montant.
   - **Compte gardé (D40-P2-1, C-40-7 de l'epic)** : contrôle `account_unchanged`, après `account_readable`. **Une
     seule règle**, fonction pure `account_decision(stored: Option<&str>, read: &str, has_downloaded: bool) ->
     Keep | Pose | Refuse` (même module que `mode_decision`, même définition de *transaction téléchargée*, AC-5 ; testée
     sur ses six combinaisons ; **`read` est toujours connu** : un compte illisible est refusé en amont,
     `account_readable`) ; **appelée par « Tester » et au début de chaque passage** (ci-dessous, D40-P3-1) :
     `stored = None` → `Pose` ; `stored = read` → `Keep` ; `stored ≠ read` sans transaction téléchargée → `Pose` (réécrit, tracé
     `accountIdChanged: true`) ; `stored ≠ read` avec transaction téléchargée → `Refuse` : `account_unchanged` en échec,
     `errorCode = PROVIDER_ACCOUNT_CHANGED`, `usable: false`, **aucun champ du serveur écrit** (ni `accountId`, ni
     `livemode`, ni `apiVersion`), `last_error = PROVIDER_ACCOUNT_CHANGED`, audit `tested` `{ result: "failure",
     accountIdChanged: true }`. Changer de compte Stripe après un téléchargement demande une **connexion neuve** (dit
     au manuel par 40-6). `PROVIDER_ACCOUNT_CHANGED` : chaîne constante de `errorCode`/`lastError`, même patron que
     `PROVIDER_MODE_CHANGED` (AC-5), message `provider-error-*` aux quatre locales.
   - **Le passage garde aussi le compte (D40-P3-1, orchestrateur).** Au **début de chaque passage** (après
     `mode_decision`, avant tout téléchargement), le passage lit `GET /v1/account` — **une requête, hors plafond** de
     l'AC-7 — par une méthode neuve `Connector::account_ref(&self) -> Result<Option<String>, ConnectorError>`
     (additive, Écart 6 ; défaut `Ok(None)` : PayPal et le factice inchangés, et `Ok(None)` → **aucune garde**, le
     passage continue ; `StripeConnector` ne rend jamais `Ok(None)` : `Ok(Some(id))` ou une erreur ; c'est l'appel
     que fait aussi « Tester », ci-dessus) et applique la **même** `account_decision` :
     `Pose` → `accountId` écrit dans la transaction qui écrit la première page, avec `livemode` (version + 1, rien
     d'écrit si aucune page ne l'est) ; `Keep` → rien ; `Refuse` → passage `failed` **sans téléchargement**,
     `error_code = invalid_response`, `error_detail = "account"`, `lastError = PROVIDER_ACCOUNT_CHANGED` ;
     compte illisible (403/404 → `Permission`) → passage `failed`, `error_code = permission`, `error_detail =
     "account"`, `lastError = PROVIDER_ACCOUNT_UNREADABLE` ; autres erreurs du connecteur : table de l'AC-10. Pas
     d'audit (passage `daily`, C-37-6). Les deux cas que le registre C-40-7 portait (clé d'un autre compte suivie d'un
     passage sans « Tester » ; clé sans droit sur le compte) sont **fermés**. Tests : passage sans « Tester » → compte
     posé ; clé d'un autre compte après un téléchargement → passage refusé sans appel de transactions (compteur du
     serveur local : `/v1/account` 1, `/v1/balance_transactions` 0) ; sans téléchargement → réécrit ; `/v1/account` en
     403 → passage `failed`, `PROVIDER_ACCOUNT_UNREADABLE`, aucune transaction lue ; la requête de compte ne compte pas
     dans les 1 000 de l'AC-7.
   - Devises lues sans compte Kesh : listées à la réponse ; leur écriture à `provider_balances_seen` reste l'affaire du
     passage (38-3b AC-7, E12, B2).
   - Tests HTTP : succès qui pose les trois champs ; second succès sans changement (rien d'écrit, audit seul) ; 401 →
     `PROVIDER_AUTH_FAILED` ; liste en 403 → `permission_present` faux, `PROVIDER_PERMISSION_MISSING` ; compte en 403 →
     `account_readable` en échec, `PROVIDER_ACCOUNT_UNREADABLE`, `usable: false`, rien d'écrit ; compte en 404 : idem ; compte en 500 (deux fois) → `account_readable` en échec,
     `PROVIDER_UNAVAILABLE` ; Comptable → 403 ; clé d'API → `403 API_KEY_ADMIN_FORBIDDEN` ; `accountId` lu ≠ réglages
     sans transaction → réécrit, `accountIdChanged: true` ; avec une transaction → `account_unchanged` en échec,
     `PROVIDER_ACCOUNT_CHANGED`, réglages et version inchangés ; compte en 403 sur une connexion téléchargée →
     `account_readable` en échec, `accountId` des réglages gardé.
10. **AC-10 — Erreurs (C-40-3, FR-40-1-6).** `ConnectorError` de 38-3b, **aucune variante neuve** ; table : 401 →
    `Auth` ; 403 → `Permission` ; 400, 402, 404, 409 → `Rejected` (409, absent de C-40-3 — conflit d'idempotence,
    sans objet en `GET` —, y est rangé par défaut sûr) ; 429 → `RateLimited { retry_after }` (`Retry-After`
    lu, au journal) ; 5xx, 424, délai → `Unavailable`, **un** second essai après 2 s (patron 38-7 AC-7) ; réseau, TLS,
    redirection → `Transport` ; corps invalide ou trop gros → `InvalidResponse`. `error.message`, `error.param`,
    `request-id` : au `tracing::warn!` seulement, jamais à l'écran ni à `last_error`. Codes d'API : ceux de 38-7 AC-7
    (`app_code()`), plus `PROVIDER_SECRET_NOT_RESTRICTED` (AppError, AC-4), `PROVIDER_MODE_CHANGED` (constante, AC-5),
    `PROVIDER_ACCOUNT_CHANGED` et `PROVIDER_ACCOUNT_UNREADABLE` (constantes, AC-9). `/v1/account` : 403 et 404 → `Permission`
    (exception à la ligne 404 → `Rejected`, propre à cette route).
11. **AC-11 — Fabrique et transport factice (G16, C-40-2 e).**
    - `ConnectorFactory` (38-3b AC-3) : `stripe` → `StripeConnector` (réglages typés, secret `restricted_key` ouvert par la
      voie unique — **aucun appelant d'`open` ajouté**, le site de la fabrique reste le seul —, mode de la clé, transport).
    - **`KESH_TEST_MODE`** : variante additive `stripeResponses` du corps de `POST /api/v1/_test/provider-connector`
      (forme et validation de `paypalResponses`, 38-3b AC-3 : tableau non vide `{ method ∈ GET, path "/…", query?, status
      100–599, body }` servi dans l'ordre, dernière répétée — d'où l'ordre des requêtes Stripe à programmer : « Tester »
     lit `/v1/balance`, `/v1/balance_transactions`, puis `/v1/account` ; un passage lit **d'abord** `/v1/account`
     (AC-9, D40-P3-1), puis ses autres lectures dans l'ordre de 38-3b —, **ou** exactement `"default"` ; sinon 400
      `AppError::Validation` ; **`query?` admet une liste de valeurs par clé** — `expand[]`, ou `created[gte]` unique —,
      amendement additif de la forme « objet de chaînes » de 38-3b AC-3, PayPal inchangé) ; une connexion `stripe` sans variante garde le `FakeConnector` ; avec elle, un vrai
      `StripeConnector` sur le transport factice. `TEST_ENDPOINT_ROUTES` inchangé (même route).
    - **Jeu `"default"`** (`stripe::fake`, constantes et une fonction `default_responses(now)`) : servi **par chemin** et
      non dans l'ordre — `/v1/balance`, `/v1/account`, `/v1/balance_transactions` (le factice applique lui-même
      `created[gte]`, `created[lt]`, `source`, `limit`, `starting_after` à une liste constante, et ne développe les sources
      que si `expand[]=data.source` est demandé — chaîne sinon, comme Stripe, pour exercer la lecture de l'AC-8) — afin qu'un E2E (40-6,
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
      fixture ne contient `@` hors `example.invalid`, ni de clé — motif `\b(sk|rk|pk)_(live|test)_[A-Za-z0-9]` ; **pas**
      la sous-chaîne `rk_` seule, que portent des noms de champs réels des réponses de test (`network_status`,
      `network_token`, `risk_level`) ; les identifiants factices suivent une forme
      reconnaissable (`<préfixe>_kesh<n>`), et le test refuse tout identifiant qui ne l'a pas.
    - Serveur axum local (`with_base_url`, `HttpTransport::for_local_tests()`) : forme exacte de chaque requête (chemin,
      paramètres, `expand[]`, `Stripe-Version`, `Bearer`) ; chaque ligne de la table d'erreurs ; 500 puis 200 ; délai ;
      302 non suivi ; corps de 9 MiB ; `Content-Length` menteur.
    - Suites PayPal (38-7, 38-8, 38-3b) vertes **sans retouche de leurs attentes** (G18) ; si `ConnectorContext` passe aux
      secrets nommés ici (AC-4), seuls leurs constructeurs de test changent, test nommé au Change Log.
14. **AC-14 — Ce que la story ne touche pas, registres, i18n.** Aucune migration (C-40-0 : `provider = 'stripe'` et
    `settings` dans des colonnes existantes à domaine ouvert ; P1–P8 sans objet, ni `TABLES_TO_TRUNCATE`, ni registre de
    rejeu) ; aucune route (`LIB_ROUTES`, `TEST_ENDPOINT_ROUTES` inchangés, C-40-6) ; aucune action d'audit neuve
    (`secrets_updated`, `tested`, `created`, `updated` : posées par 38-1 au registre `ACTIONS` de `crates/kesh-api/src/audit_labels.rs:70`, **absentes du code
d'aujourd'hui** — l'Epic 38 n'étant pas développé) ; aucun outil
    MCP (G17, E23 : connexions non exposées) ; sauvegarde : la ligne de connexion et ses réglages partent comme toute
    connexion (38-1 AC-8), la clé est effacée à la restauration et se ressaisit (dit au manuel par 40-6) ; aucun écran ni
    manuel (40-6) — les noms de contrôles neufs (`mode_unchanged`, `account_unchanged`, `account_readable`, `permission_present` pour
    Stripe) et les codes neufs sont **transmis à 40-6** pour `provider-labels.ts`. **i18n**, quatre locales
    (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`) : message de `PROVIDER_SECRET_NOT_RESTRICTED`
    (AppError), `provider-error-provider-mode-changed`, `provider-error-provider-account-changed` et
    `provider-error-provider-account-unreadable` (D40-P3-1 : « Cette clé ne peut pas lire le compte Stripe : ajoutez le
    droit de lecture du compte », le texte que 40-6 AC-3 reprend à l'écran). Aucune ligne au `CHANGELOG.md` :
    la section **Note de version** de cette fiche porte le texte, assemblé à la release.
15. **AC-15 — Documentation de l'API (`docs/api-external.md`).** Les routes des connexions de prestataire restent
    fermées aux clés (§4, `403 API_KEY_ADMIN_FORBIDDEN`) ; la section qui décrit la connexion PayPal (38-1), ou à défaut
    une sous-section du §7, gagne Stripe : `provider: "stripe"`, réglages `StripeSettings` (modifiables : `overlapDays`,
    `invoiceRefSource`, `eventKindOverrides` ; posés par le serveur : `accountId`, `apiVersion`, `livemode`), corps de
    `PUT …/secrets` `{ "restrictedKey": "…" }` (préfixes `rk_live_`/`rk_test_`) ; au §10, `400
    PROVIDER_SECRET_NOT_RESTRICTED` (`details.reason` ∈ `secret_key`, `publishable_key`, `unknown_prefix`) ; et
    `PROVIDER_MODE_CHANGED`, `PROVIDER_ACCOUNT_CHANGED`, `PROVIDER_ACCOUNT_UNREADABLE` dits comme valeurs de `errorCode` d'un contrôle de « Tester » et
    de `lastError`, **non** comme réponses d'erreur HTTP. Contrôlé contre le code (noms de champs, codes, statut).

## Tâches

- [ ] T0 — Prérequis : Epic 38 fusionné ; 40-0 tenue (version d'API des fixtures, droits de la clé constatés) ; sinon arrêt.
- [ ] T1 — `kesh-core` : `Provider::Stripe`, `StripeSettings`, validation, `parse_settings`, tests (AC-2).
- [ ] T2 — `kesh-providers::stripe` : constantes, requête, `amounts`, `StripeRecord`/`StripeSource` et validation,
      curseur et découpage, `balances`, `test()`, `account_ref()` (méthode additive du trait, Écart 6), `key_mode`, `classify_window` provisoire ; `value_date` dans
      `ProviderMovement` et `validate_page` (AC-1, AC-5 à AC-8).
- [ ] T3 — `kesh-api` : lecture du compte au début du passage (`Connector::account_ref`, `account_decision`, D40-P3-1), corps de secrets Stripe, `PROVIDER_SECRET_NOT_RESTRICTED`, `required_secrets` et secrets nommés si
      39 n'est pas livré, fabrique, route `test` (contrôles, écriture des champs du serveur, audit), garde de mode au passage,
      variante `stripeResponses` et jeu `"default"` (AC-3, AC-4, AC-5, AC-9, AC-11).
- [ ] T4 — i18n quatre locales, Note de version (AC-14) ; `docs/api-external.md` (AC-15).
- [ ] T5 — Fixtures anonymisées et leurs tests, tests hostiles, test au marqueur, non-régression PayPal (AC-12, AC-13).
- [ ] T6 — Gate complet (fmt, clippy `-D warnings`, nextest complet sur base remise à zéro, `scripts/gate-ci.sh`) ;
      `Cargo.lock` commité si une dépendance change (aucune attendue : `zeroize`, `chrono-tz`, `rust_decimal` déjà là) ;
      compteurs (`Cargo.lock`, registres) recomptés depuis la source après chaque reprise de `main`.

## Note de version

*Texte prêt à assembler au `CHANGELOG.md` à la release ; aucune ligne n'y est ajoutée dans la branche.*

- **Ajouté** : compte Stripe — connexion par clé d'API **restreinte en lecture** (`rk_live_` ou `rk_test_` ; une clé
  secrète `sk_` est refusée), version d'API épinglée, bouton « Tester » (mode, compte, droits, soldes `available` et
  `pending`), téléchargement des transactions de solde par curseur, une devise après l'autre. Aucune migration.

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
  (dix crates aujourd'hui, sans `kesh-providers` : il sera le onzième, posé par 38-3b AC-1). Le reste du socle est **à venir** (Epic 38 non développé) : ses noms
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
- **Q-40-8** (epic) : droits *Read* minimaux ; ceux qu'exigent `GET /v1/account` et l'expansion des sources. ⚠️ Depuis
  D40-P3-1, le droit de lire le compte est **exigé** : si la recette (40-0) montre qu'une clé restreinte ne peut pas le
  recevoir en lecture seule, la décision est à reprendre avec Beat avant le développement.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : CHANGELOG remplacé par la section Note de
  version ; compteurs « recomptés depuis la source après chaque reprise de `main` » (T6).

- 2026-11-15 — validation P5 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW ; remédiés dans la fiche. Périmètre :
  `git diff 98bca548 b5d23154` de la fiche (remédiation P4). Confirmés : « Tester » appelle `test()` (deux requêtes :
  `/v1/balance`, `/v1/balance_transactions`) puis `account_ref()`, erreur attribuée à `account_readable` (`Permission` →
  `PROVIDER_ACCOUNT_UNREADABLE`) ; `lastError` du passage = `PROVIDER_MODE_CHANGED` / `_ACCOUNT_CHANGED` /
  `_ACCOUNT_UNREADABLE`, amendement de 38-3b AC-4 (Échec, `:256-260`) déclaré à l'Écart 6, `error_code` restant un
  `code()` ; ordre des programmes du factice (AC-11) conforme à l'AC-9 ; `account_decision` à six combinaisons, `read`
  toujours connu ; plus aucun « trois requêtes » ni `PROVIDER_RESPONSE_INVALID` au passage. LOW : P5-1 AC-14, texte du
  message `unreadable` (« la clé ne peut pas… ») différent de celui que 40-6 AC-3 reprend (« Cette clé ne peut
  pas… ») → aligné ; P5-2 Écart 6, « une seule lecture du compte » ambigu (un appel par appelant) → « un seul appel,
  la même méthode » ; P5-3 AC-10, « et … et » → énumération. Lignes relues : 38-3b `:250-260`, `:372-380` (`lastError`,
  constantes), 38-9a AC-5 (deux vocabulaires), epic C-40-3 (`:326-333`) et périmètre de 40-1 (`:390-392`). Inter-fiches :
  aucun. Axes exercés : chaque phrase changée en P4 contre 38-3b, 38-9a, 40-6, l'epic ; cohérence AC-5/AC-9/AC-10/AC-11/
  AC-14/AC-15 et Tâches. Non exercés : le reste de la fiche, exécution de code.
- 2026-11-15 — validation P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW ; tous remédiés dans la fiche.
  MEDIUM : P4-1 AC-9/Écart 6, `test()` ne rend qu'une `ConnectorError` : un 403 de `/v1/account` y était
  indiscernable d'un 403 de la liste (`permission_present`), et l'AC-10 interdit une variante neuve — le contrôle
  `account_readable` n'était pas implémentable → `StripeConnector::test()` ne lit plus le compte (deux requêtes,
  `account_ref: None`) ; la route appelle `Connector::account_ref()` après `test()`, la même méthode que le passage, et
  attribue son erreur à `account_readable` (`Permission` → `PROVIDER_ACCOUNT_UNREADABLE`, sinon table de l'AC-10) ;
  P4-2 AC-9/AC-5, `lastError = PROVIDER_ACCOUNT_CHANGED`/`_UNREADABLE` au passage contredisait 38-3b AC-4 (Échec :
  `last_error` = `app_code()` ou code propre du passage), amendement non déclaré, et divergeait de la garde de mode
  (`PROVIDER_RESPONSE_INVALID`) → amendement déclaré à l'Écart 6, garde de mode alignée (`PROVIDER_MODE_CHANGED`,
  test). LOW : P4-3 `account_ref() = Ok(None)` au passage sans effet dit (aucune garde) ; P4-4 AC-11, ordre des
  requêtes à programmer en tableau (`/v1/account` d'abord au passage) ; P4-5 AC-9, `accountId` `null` aussi quand
  `account_readable` échoue ; test `/v1/account` en 500 ajouté. Lignes relues : 38-3b AC-2 (trait, `ConnectorTest`,
  `code()`/`app_code()`), 38-3b AC-4 (Échec), 38-7 AC-8 ; `crates/kesh-api/src/routes/reconciliation.rs:170-176, 1478`
  (`valueDate` déjà rendu). Epic (points inter-fiches de la passe) : C-40-3 gagne `PROVIDER_ACCOUNT_UNREADABLE` ; DTO
  des propositions (`valueDate` déjà rendu, `entryDate` seule) ; G12 et FR-40-4-2 ; G18 nuancé ; propagés : C-40-1
  (`accountId` exigé) et périmètre de 40-1 (`account_ref()`).
- 2026-11-15 — validation P3 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW ; remédiés dans la fiche sauf 40-1-P3-5
  (inter-fiches : 40-6 et C-40-3 de l'epic, au rapport : le contrôle `account_readable` et le code neuf). MEDIUM : P3-1 décision D40-P3-1 appliquée (AC-9) — le passage
  lit `/v1/account` au début de chaque passage (`Connector::account_ref`, Écart 6, une requête hors plafond) et applique
  `account_decision` (désormais `read: &str`, six combinaisons) ; une clé qui ne lit pas le compte est refusée au
  passage comme à « Tester » (contrôle `account_readable`, `PROVIDER_ACCOUNT_UNREADABLE`, message
  `provider-error-provider-account-unreadable`) ; les deux cas du registre C-40-7 sont retirés ; tests ajoutés ;
  Q-40-8 durcie. LOW : P3-2 AC-5, « huit combinaisons » de `mode_decision` (trois états × deux modes × deux = douze) ;
  P3-3 AC-4, `sk_` autre que `sk_live_`/`sk_test_` (`sk_org_…`) hors des raisons ; P3-4 AC-9/AC-10, 404 de
  `/v1/account` en `Rejected` contre la table → exception dite. Lignes relues : `config.rs:1522, 1538`,
  `extractors.rs:18`, `audit_labels.rs:24, 70`, 38-3b (trait `Connector`, `ConnectorTest`).
- 2026-11-15 — alignement inter-fiches P2 (Opus) : décision D40-P2-1 de l'orchestrateur reportée (AC-9) — garde
  `account_unchanged` (fonction pure `account_decision`, patron de `mode_decision`) : `accountId` lu ≠ réglages sur une
  connexion déjà téléchargée → refus `PROVIDER_ACCOUNT_CHANGED`, sans téléchargement → réécrit et tracé ; limite
  restante (passage sans « Tester », clé sans droit sur le compte) au registre C-40-7 ; code ajouté à l'AC-10, contrôle
  et message `provider-error-provider-account-changed` à l'AC-14 ; AC-15 neuve, `docs/api-external.md` (réglages, corps
  du secret, trois codes), tâche T4. 15 AC.
- 2026-11-15 — validation P2 (Opus) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 8 LOW ; remédiés dans la fiche sauf 40-1-P2-10
  (registre C-40-7 de l'epic), 40-1-P2-11 (40-6 AC-2) et 40-1-P2-12 (40-4 AC-5), inter-fiches, au rapport. MEDIUM :
  P2-1 AC-8, somme du solde passé lue sans expansion rejetée par la règle `record.source` de l'AC-6 → règles de la
  source levées dans ce parcours, jeu `"default"` qui ne développe que sur demande ; P2-2 AC-5, comparaison du mode
  placée dans la fabrique : « Tester » n'aurait jamais pu réécrire le mode d'une connexion sans téléchargement, et
  passage et « Tester » suivaient deux règles → fonction pure `mode_decision` partagée, avant tout appel ; P2-3 AC-6,
  `source.id` `^[a-z]{2,4}_` rejetait des sources réelles (`iauth_`, doc relue ; `climorder_`) et toute leur page →
  préfixe 1 à 20 ; P2-4 AC-13, test « ni `rk_` » infaisable (`network_status`, `risk_level` dans les réponses de
  test) → motif de clé. LOW : P2-5 Écart 1 périmé (epic aligné sur P40-1) ; P2-6 AC-4, contrôle de gabarit
  inatteignable après préfixe et jeu de caractères, `changeme` inconnu d'`is_template_placeholder` (`config.rs:1522`)
  → tests précisés ; P2-7 AC-6/AC-9, motifs non nommés, règle `created` sans fenêtre ; P2-8 AC-10, 409 hors C-40-3
  dit ; P2-9 « dix crates » ambigu ; P2-10 changement de compte Stripe non gardé (limite dite, à tracer) ; P2-11, P2-12
  inter-fiches. FR-40-1-7 : « ou au premier passage ». Lignes relues : `config.rs:1522, 1538`, `extractors.rs:18`,
  `audit_labels.rs:24, 70`, `Cargo.toml:1-14` ; réponses de test (`balance_transactions.json`, `balance.json`,
  `payouts.json`).

*(Findings de la validation P1 : P1-1 MEDIUM AC-5, passage d'une connexion jamais testée sans mode posé ; P1-2 MEDIUM
AC-7, plafond de requêtes par sous-fenêtre seulement ; P1-3 MEDIUM AC-11, `query?` de `stripeResponses` sans valeurs
répétées pour `expand[]` ; P1-4 LOW AC-8, course de `T0` mal décrite ; P1-5 LOW AC-8, devises spéciales ; P1-6 LOW AC-14,
actions d'audit dites existantes ; P1-7 LOW AC-7, « croissant » ambigu ; P1-8 LOW inter-fiche, voir rapport.)*

- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 5 LOW ; remédiés dans la fiche sauf 40-1-P1-8
  (inter-fiche : fichier d'epic, au rapport).
- 2026-11-15 — rédaction (Opus). 14 AC ; 7 FR (dont 1 ajoutée), 4 NFR ; aucune migration ; cinq écarts à l'epic
  (P40-1 et sa garde de mode, `livemode` absent des transactions de solde, `ConnectorTest.livemode`, classement provisoire,
  tolérance d'un jour sur `value_date`). Documentation Stripe relue (WebFetch) : authentification, versions, liste des
  transactions, expansion, versement, compte.
