# Story 40-8a : client de l'API Stripe — clé restreinte en variable d'environnement, versements et transactions de solde

Status: review

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`.
Epic 40, **phase 2** (décision de Beat du 2026-11-20, § *Phase 2* du fichier d'epic). **Dépend de** : la phase 1 (40-7a à
40-7d) et du **socle commun S1 à S9** de la phase 2 (fichier d'epic), écrit par 38-12a ou, à défaut, par la première fiche de
phase 2 développée (C-40-K6). **Suivie par** : 40-8b (téléchargement).*

## Amendement du 22.11 (développement, avant la première ligne de code)

Relu contre le socle de la 38-12a (`provider_env`, `provider_download`, `http_client`) et contre les réponses réelles de
2024 (lecture seule, relevées le 2026-11-21, jamais versées au dépôt). Ce qui change, et pourquoi :

1. **AC-4, `details.resource` et `error.code`/`error.param`** : les erreurs du socle (`AppError::ProviderPermissionDenied {
   provider }`, `ProviderApiError { provider, window, page }`) ne portent ni la ressource ni le code du prestataire, et le
   client PayPal écrit le **nom** d'erreur de PayPal au **journal du serveur** seulement. Pour ne pas élargir le socle
   (PayPal, Wise), Stripe fait de même : la ressource (`payouts`, `balance_transactions`), `error.code` et `error.param`
   (bornés à 64 caractères chacun), et l'en-tête `Stripe-Rate-Limited-Reason` vont au journal, **jamais** `error.message`
   ni la clé ; la réponse porte `details: { provider, window, page }`, `window` = la période demandée, `page` = le rang de
   la page dans la liste (1 pour la première).
2. **AC-3, transaction du versement** : constaté sur les données réelles de 2024, `?payout=` **rend** la transaction du
   versement lui-même (`type = payout`, 104 versements payés sur 104) ; elle est écartée (type `payout` **et** `source` =
   l'identifiant du versement). Σ net des autres = `amount` du versement pour les 104 (aucun écart) : le contrôle
   bloquant d'AC-3 tient tel qu'écrit.
3. **AC-2, `Stripe-Version`** : `2026-11-18.endive`, relue le 2026-11-22 sur la page « Versioning » de la documentation
   (« The current version is 2026-11-18.endive »).
4. **AC-3, exposants** : la page des devises, relue le 2026-11-22, ne rend pas sa liste des devises à zéro décimale
   (liens) ; la constante reprend la liste publiée par Stripe (seize codes : BIF, CLP, DJF, GNF, JPY, KMF, KRW, MGA, PYG,
   RWF, VND, VUV, XAF, XOF, XPF, et non UGX), datée. **ISK et UGX** sont lus à **deux** décimales : la page dit que, pour
   la compatibilité, leurs montants s'écrivent en centièmes « dont la partie décimale vaut toujours 00 » ; HUF et TWD
   aussi (deux décimales hors versements manuels). Trois décimales (BHD, JOD, KWD, OMR, TND) : erreur de ligne `AMOUNT`.
5. **AC-1, contrôle propre à Stripe** : le lecteur du socle reçoit un crochet `ProviderSpec::check` (fonction pure champ,
   valeur → raison), appelé pour chaque champ présent après le contrôle du gabarit ; `None` pour PayPal. Aucun site de
   lecture de l'environnement n'est ajouté : `KESH_STRIPE_*` est lu par le parcours du socle, déjà inscrit à
   `EMPLACEMENTS_AUTORISES` ; la garde `configuration_transmise.rs` connaît la famille par `ALL_PROVIDERS` (noms
   dérivés), sans retouche de ses listes.
6. **Modules** : `kesh-import` n'est plus seulement lu : `stripe::valid_id` (contrôle d'identifiant du lecteur CSV)
   devient public, pour que l'API et le CSV refusent les mêmes identifiants (DRY). Les années hors plage se contrôlent par
   `kesh_core::dates::annee_stockable` (même plage, égalité déjà testée).
7. **AC-3, forme des lignes** : versements rangés par (`arrival_date`, `id`), transactions par (`created`, `id`) ; `line`
   = rang à partir de 2 (comme une ligne de fichier sous son en-tête) ; `created` = l'heure de Zurich
   `AAAA-MM-JJ HH:MM:SS` (la forme du rapport du tableau de bord, constatée identique sur 336 transactions de 2024).
8. **Orchestration** : le client rend les lignes d'une période (`fetch_rows` : liste des versements, puis leurs
   transactions, au plus `parallel` versements à la fois par `run_bounded`) ; la tâche, sa progression et l'import sont
   ceux de 40-8b.

## Story

En tant qu'**Administrateur**, je veux poser dans l'environnement de Kesh une clé Stripe **restreinte, en lecture
seule**, rattachée au libellé de mon compte Stripe, afin que Kesh puisse lire mes versements et leurs transactions de
solde sans que la clé soit jamais en base ni dans une sauvegarde.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-8a-1` | Une clé par compte Stripe, en variables d'environnement, rattachée au libellé du compte (réglage de 40-7b) ; rien en base, rien en sauvegarde, rien dans les journaux. | AC-1 |
| `FR-40-8a-2` | Seule une clé restreinte est admise (`rk_live_…`, `rk_test_…`) ; une clé secrète `sk_…` est refusée au démarrage. | AC-1 |
| `FR-40-8a-3` | Lire, pour une période, les versements automatiques payés et, pour chacun, ses transactions de solde ; les rendre comme les lignes du rapport CSV (`StripePayoutRow`, 40-7a), **aux dates de Zurich**. | AC-2, AC-3 |
| `FR-40-8a-4` | Pagination par curseur, 100 par page ; erreurs rendues par les codes génériques du socle (S4), sans texte de Stripe. | AC-2, AC-4 |
| `NFR-40-8a-1` | Aucun appel réseau en CI ni par un agent : serveur Stripe simulé dans le processus (socle S5). | AC-5 |

## Critères d'acceptation

1. **AC-1 — Configuration**, par le **lecteur unique du socle S1** (`provider_env`, patron de `KESH_SMTP_PASSWORD`,
   `crates/kesh-api/src/config.rs:329-332` : champ privé, masqué dans `Debug`, accesseur ; `is_template_placeholder` `:1538`) :
   `KESH_STRIPE_<n>_ACCOUNT` (libellé, égal au `provider_account` d'un compte Stripe de 40-7b) et `KESH_STRIPE_<n>_KEY`,
   `n` de 1 à 9, paires complètes ou absentes (une moitié seule → refus de démarrer, message qui nomme la variable ; une valeur **vide vaut
   absente**, `env_nonempty`, `config.rs:1406` : sans variable Stripe, Kesh démarre) ;
   facultatives `KESH_STRIPE_BASE_URL` (défaut `https://api.stripe.com` ; `http` admis vers le seul bouclage, pour les tests,
   S5) et `KESH_STRIPE_PARALLEL` (S1 : 1 à 6, défaut 1 ; employée par 40-8b). **Contrôle propre à Stripe**, branché sur le
   lecteur : préfixe `rk_live_` ou `rk_test_` exigé, `sk_` refusé (« utilisez une clé restreinte en lecture »), valeur de gabarit
   `<…>` refusée comme ailleurs. Le mode (`live`/`test`) se lit au préfixe et se montre à l'écran par `providerApi.mode`
   (40-8b). Jamais écrite en base, dans l'audit, dans un `.keshbackup`, ni dans un message d'erreur. **Transmission au conteneur** (S1) : `KESH_STRIPE_1_*` à
   `KESH_STRIPE_3_*`, `KESH_STRIPE_BASE_URL`, `KESH_STRIPE_PARALLEL` ajoutées à `docker-compose.yml` et
   `docker-compose.prod.yml` en `${NOM:-}` (patron `KESH_SMTP_*`, `docker-compose.yml:117-122`) et à `.env.example`
   (patron `:302-340`). **Garde `configuration_transmise.rs` (#550)** : cette story **inscrit la famille `KESH_STRIPE_<n>_*`** (ACCOUNT, KEY ; séries 1 à 3 exigées, 4 à 9 admises absentes avec raison) à la famille indexée générique de 38-12a, sans relâcher aucune liste. Elle inscrit aussi `KESH_STRIPE_BASE_URL` et `KESH_STRIPE_PARALLEL` (noms **sans numéro**, construits par `format!`, lus par le parcours `std::env::vars()` : sites inscrits en forme exacte à `EMPLACEMENTS_AUTORISES`) ; leurs lignes de `.env.example` sont commentées ou vides, jamais `<…>`. Le libellé est **propre à
   l'installation** (S1) : manuel d'administration (40-8b) : les droits *Read* à donner et ce point.
2. **AC-2 — Client** (`crates/kesh-api/src/stripe/`, client HTTP du socle S5 : rustls, aucune redirection, délais bornés ;
   **`reqwest` n'a pas la fonctionnalité `json` en dépendance normale** — `Cargo.toml:112` —, lire par `.bytes()` puis
   `serde_json::from_slice`, comme la source des cours lit son XML). **Pas de trait de source simulée** (la version initiale
   de cette fiche copiait `RateSource`/`MockRateSource`, `exchange_rates/source.rs:97`, `:352`) : un seul schéma de test pour les
   trois prestataires, un serveur HTTP simulé sur `KESH_STRIPE_BASE_URL` (AC-5). Authentification `Authorization: Bearer
   <clé>` ; en-tête `Stripe-Version` **épinglé** en constante (valeur lue sur la page de référence du 2026-11-20 :
   `2026-11-18.endive`, **à relire au développement** ; une version trop neuve n'a jamais servi ici, les champs lus —
   `amount`, `fee`, `net`, `currency`, `created`, `available_on`, `reporting_category`, `source`, `description`, `type` —
   sont anciens). Appels :
   - `GET /v1/payouts?arrival_date[gte]=…&arrival_date[lt]=…&status=paid&limit=100` puis `starting_after=<dernier id>` tant
     que `has_more`, **filtré par `arrival_date`** (et non `created`) : c'est la date que la banque voit et que prend
     `payout_date` (AC-3), si bien qu'un versement créé fin décembre et arrivé début janvier appartient à **un seul** exercice,
     jamais manqué ni compté deux fois entre deux téléchargements ; bornes = `kesh_core::horloge::debut_du_jour_suisse(from)`
     et `debut_du_jour_suisse(to + 1 jour)` en secondes d'époque (`crates/kesh-core/src/horloge.rs:95`). Seuls les versements
     `automatic = true` sont gardés (les autres → compte rendu, « versement manuel à examiner ») ;
   - pour chaque versement, `GET /v1/balance_transactions?payout=<po_…>&limit=100` (même pagination), dont Stripe dit qu'il
     rend, « *for automatic Stripe payouts only* », les transactions payées par ce versement.
   - **Aucune fenêtre** : Stripe ne borne pas la période d'une liste, les curseurs suffisent ; le socle S2 n'est employé que
     pour la reprise et le parallélisme, sans `windows` (Stripe n'a pas de `max_days`, décision de l'orchestrateur sur le
     socle).
3. **AC-3 — Traduction en lignes du rapport.** Fonction pure : transaction de solde → `StripePayoutRow` (40-7a AC-3) :
   `payout_id`, `payout_date` = **jour de Zurich** de l'`arrival_date` du versement, `balance_transaction_id` = `id`, `date`
   = **jour de Zurich** de `created` (`kesh_core::horloge::jour_suisse`, `:64`), `available_on` idem, `currency` en
   majuscules, `gross` = `amount`, `fee`, `net` convertis des **plus petites unités** en unités selon l'exposant de la devise
   (table des devises à zéro décimale de Stripe, en constante, **datée** et copiée de `docs.stripe.com/currencies` au
   développement ; une devise à **trois décimales** est **refusée** comme le lecteur CSV la refuse, 40-7a AC-5 : erreur de
   ligne `AMOUNT`), `category` = `reporting_category`, `description`, `source_id` = `source` (identifiant), `charge_id` vide.
   ⚠️ **Zurich et non UTC** : un instant entre 00:00 et 02:00 à Zurich a, en UTC, le jour d'avant — le 31 décembre à 23:30
   UTC est le 1er janvier à Zurich —, et mettrait la transaction dans le mauvais exercice. La transaction du versement
   lui-même (`type = payout`), **si `?payout=` la rend** (non confirmé), n'est pas une ligne. Le contrôle « Σ net des lignes
   = montant du versement » (la même grandeur que l'écart `payout_total_mismatch` de 40-7c (liste des écarts, sans proposition) pour un CSV ; ici il est **bloquant**, ci-dessous) prend le montant du versement dans
   **l'objet versement** de `GET /v1/payouts` (`amount`), non dans une transaction de solde que Stripe peut ne pas rendre. **Issue d'un écart** (validation P2 : elle n'était pas
   dite) : Σ net ≠ `amount` → la tâche échoue en `PROVIDER_RESPONSE_INVALID`, `details: { provider: "stripe", payoutId,
   expected, found }`, **rien d'écrit** (socle S6) — l'import calcule la transaction du versement sur Σ net (40-7b AC-5) et
   40-7c ne verrait plus l'écart ; une liste incomplète écrirait un versement faux. Relancer plus tard est sans risque. Le
   même contrôle de cohérence que le lecteur (net = brut − frais) s'applique. Les lignes rendues passent ensuite par **le
   même chemin** que le CSV (40-7b AC-7) : CSV et téléchargement se mélangent, doublons par `balance_transaction_id`.
4. **AC-4 — Erreurs**, codes **génériques** du socle S4 (`details: { provider: "stripe" }`), **jamais** le texte de Stripe
   (celui d'une clé invalide cite un fragment de la clé) ni la clé : 401 → `PROVIDER_AUTH_FAILED` ; 403 →
   `PROVIDER_PERMISSION_DENIED` (`details.resource` : la ressource demandée, `payouts` ou `balance_transactions`) ; **400** →
   `PROVIDER_API_ERROR` avec le seul `error.code` et `error.param` de Stripe, jamais `error.message` — la doc de Stripe dit
   qu'un droit manquant rend « une erreur de requête invalide » **sans dire le statut** (le journal des requêtes de Stripe
   parle de « 403 ») : **le statut réel d'un droit manquant n'est pas confirmé** et se constate à la recette de Beat sur une
   instance jetable (le manuel d'administration, 40-8b, dit de vérifier d'abord les droits de la clé sur un
   `PROVIDER_API_ERROR`) ; 429 et 5xx → la reprise du socle S2 (`Stripe-Rate-Limited-Reason` est relevée au journal, sans
   effet sur la politique), puis `PROVIDER_RATE_LIMITED` ou `PROVIDER_UNAVAILABLE` ; délai dépassé →
   `PROVIDER_UNAVAILABLE` ; réponse illisible → `PROVIDER_RESPONSE_INVALID` (sans la réponse au message). Messages aux
   quatre locales posés avec le socle S4 (38-12a) ; aucun ne contient la clé.
5. **AC-5 — Tests sans réseau** (socle S5 et S9). Unitaires de la traduction (devise à deux décimales, à zéro décimale,
   refus de trois décimales, remboursement à frais négatif, **instant du 31 décembre à 23:30 UTC rendu au 1er janvier**, versement
   dont `arrival_date` franchit l'année, Σ net ≠ `amount` → `PROVIDER_RESPONSE_INVALID` avec `payoutId`), de la configuration (paire incomplète, `sk_`, gabarit) ; un serveur `axum` **dans le
   processus** (`127.0.0.1:0` = `KESH_STRIPE_BASE_URL`) sert des réponses **fictives** versionnées
   (`crates/kesh-api/tests/fixtures/stripe-api/*.json`, identifiants `TEST`), dont une pagination à deux pages, un 429, un 401 et
   un 400 ; il vérifie l'en-tête `Stripe-Version` et l'authentification reçues. Les lignes obtenues des réponses fictives
   égalent celles de la fixture CSV de 40-7a pour les mêmes versements (test d'égalité). Test au marqueur (S9) : la clé
   `rk_test_MARQUEUR…` n'apparaît dans aucun journal capturé ni réponse. Aucun appel à `api.stripe.com` en CI.

## Tâches

- [x] T1 — Contrôle de clé branché sur le lecteur du socle (AC-1).
- [x] T2 — Client HTTP, pagination, `Stripe-Version`, filtre `arrival_date` (AC-2).
- [x] T3 — Traduction en `StripePayoutRow`, dates de Zurich, exposants de devise (AC-3).
- [x] T4 — Erreurs, tests, serveur simulé et fixtures fictives (AC-4, AC-5).

## Stratégie de tests

Unitaires et tests d'intégration sans base sur le serveur simulé ; gate ciblé `binary(stripe_client)` ; gate complet au
push.

## Modules touchés (2)

`kesh-api` (configuration, client), `kesh-import` (type `StripePayoutRow` ; `stripe::valid_id` rendu public, amendement du 22.11, point 6) ; plus `docker-compose.yml`,
`docker-compose.prod.yml`, `.env.example`. Aucun message propre : les
`PROVIDER_*` et leurs messages aux quatre locales sont posés par le socle S4 (38-12a) ; `kesh-i18n` n'est pas touché
(validation P2).

## Note de version

*Ajouté* — Kesh peut lire les versements Stripe et leurs transactions par l'API, avec une clé restreinte en lecture posée
en variable d'environnement (`KESH_STRIPE_<n>_ACCOUNT`, `KESH_STRIPE_<n>_KEY`).

## Notes de développement

- Doc publique lue le 2026-11-20 : `GET /v1/balance_transactions` (filtres `created`, `currency`, `payout` « *for automatic
  Stripe payouts only* », `source`, `type` ; `limit` 1 à 100, défaut 10 ; curseurs `starting_after`/`ending_before`) ;
  `GET /v1/payouts` (filtres `arrival_date`, `created`, `destination`, `status` ∈ `pending`, `paid`, `failed`, `canceled` ;
  `limit` 1 à 100) ; clés restreintes (`rk_live_`/`rk_test_`, droits *None*/*Read*/*Write* par ressource, `GET` = lecture,
  valeur **non relisible** après création, 2FA à la création) ; montants en plus petite unité (exemple de la doc : `amount:
  -400` pour 4 USD).
- **Non confirmé** : noms exacts, au tableau de bord, des droits *Read* qui couvrent `balance_transactions` et `payouts` ; le
  statut HTTP d'un droit manquant (AC-4) ; si la liste par `payout=` inclut la transaction du versement lui-même ; la valeur
  de `Stripe-Version` à épingler ; la liste des devises à zéro décimale ; si l'`arrival_date` du versement donne le même jour de
  Zurich que `automatic_payout_effective_at` du rapport (sinon les dates d'écriture différeraient entre CSV et API : à
  constater à la recette sur deux versements).
- `charge_id` reste vide : 40-7c rattache les ventes par `source_id` (l'objet de la transaction), et laisse les remboursements
  « à examiner » (FR-40-7c-5) ; la phase 2 n'y change rien.
- Remplace l'ancienne 40-1 (`superseded`) : plus de coffre ni de connexion en base (38-1 `deferred`), la clé est en
  variable d'environnement comme le mot de passe SMTP.

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais ; commune à 40-8a et 40-8b) : **3 HIGH, 11 MEDIUM, 8 LOW,
  remédiés** pour l'ensemble des phases 2 de Wise et de Stripe (rapport `kesh-gate-logs/epic-39-40-kiss/validation/40-8-P1.md`
  pour l'Epic 40). Pour 40-8a : HIGH — dates des transactions en **UTC** (une transaction entre minuit et 2 h à Zurich, le 31
  décembre notamment, tombait dans le mauvais exercice) : jours de Zurich ; HIGH — versements filtrés par `created` alors que
  `payout_date` est l'`arrival_date` : filtre `arrival_date` ; MEDIUM — contrôle du total du versement lu sur une transaction
  `payout` que `?payout=` peut ne pas rendre : montant de l'objet versement ; MEDIUM — erreurs `STRIPE_*` redondantes avec les
  `PROVIDER_*` génériques, statut d'un droit manquant (400 ou 403) non établi et message de Stripe qui peut citer la clé ;
  MEDIUM — source simulée par trait au lieu du serveur HTTP commun, `reqwest` sans `json` ; MEDIUM — lecteur de
  configuration propre à Stripe au lieu du lecteur unique ; LOW — devises à trois décimales non traitées, `Stripe-Version`
  sans valeur ni relecture, `Bearer` et `charge_id` non justifiés.
- 2026-11-20 — validation P2 (Claude Opus 5.5, contexte frais ; commune à 40-8a et 40-8b ; rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/40-8-P2.md`) : 0 C, 0 H, 2 M, 3 L, remédiés. Ici : issue d'un écart entre Σ net
  et le montant du versement définie (échec `PROVIDER_RESPONSE_INVALID`, rien d'écrit) ; aucune fenêtre (Stripe sans
  `max_days`, décision de l'orchestrateur) ; `kesh-i18n` retiré des modules (aucun message propre).
- 2026-11-20 — validation P3 ciblée (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P3-ciblee.md`) : famille Stripe inscrite à la garde `configuration_transmise.rs` ; écart de versement distingué de l'écart non bloquant du CSV.
- 2026-11-20 — validation P4 ciblée (Claude Haiku 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P4-ciblee.md`) : registre : six `assert_eq!` dont cinq littéraux, les deux phrases du manuel ; garde : noms sans numéro `KESH_STRIPE_BASE_URL` et `KESH_STRIPE_PARALLEL` inscrits ; `payout_total_mismatch` (code exact de 40-7c).
- 2026-11-20 — validation P5 ciblée (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-40-phase2-P5-ciblee.md`) : 0 C, 0 H, 0 M, 1 L, remédié : entrée P4 limitée à ce qui concerne Stripe (renvoi à la Q-39-16 de Wise retiré), casse des noms `KESH_STRIPE_BASE_URL` et `KESH_STRIPE_PARALLEL`.
- 2026-11-20 — validation P2, complément (Claude Opus 5.5)
 : valeur vide = absente, séries 1 à 3 aux deux compose et à
  `.env.example` (corrections du socle relevées par la P2 de la 38-12).
- 2026-11-22 — amendement du 22.11 (développement), puis développement : statut `review`. Voir le Dev Agent Record.

## Dev Agent Record

### Ce qui a été fait

- **Crochet du lecteur (AC-1, T1)** : `ProviderSpec::check: Option<FieldCheck>` (`crates/kesh-api/src/provider_env.rs`),
  appelé par `read` pour chaque champ présent, après le contrôle du gabarit ; `None` pour PayPal. L'égalité de
  `ProviderSpec` compare désormais le seul `key` (`PartialEq` manuel : un pointeur de fonction ne se compare pas, refus de
  clippy) ; aucun site du dépôt ne comparait deux `ProviderSpec` de `kesh-api` (relevé par `grep`). `ALL_PROVIDERS` =
  PayPal, Stripe.
- **Spécification et clé (AC-1)** : `crates/kesh-api/src/stripe/mod.rs` — `STRIPE` (`KESH_STRIPE_<n>_ACCOUNT`, `_KEY`,
  hôte `https://api.stripe.com`), `key_mode` (préfixe `rk_live_` ou `rk_test_` suivi de lettres, chiffres ou `_`),
  `check_field` (`sk_…` : « clé secrète refusée… », autre : « clé restreinte attendue… », jamais la valeur),
  `KeyMode::as_str` (`live`/`test`, pour 40-8b). Séries 1 à 3, `KESH_STRIPE_BASE_URL` et `KESH_STRIPE_PARALLEL` aux deux
  compose (`${NOM:-}`) et à `.env.example` (lignes commentées). La garde `configuration_transmise.rs` connaît la famille
  par `ALL_PROVIDERS`, sans retouche (verte, `s_familles_indexees` comprise).
- **Client (AC-2, T2)** : `StripeClient` — `GET` seulement, `Bearer`, `Stripe-Version: 2026-11-18.endive`, client du socle
  (rustls, sans redirection, délais 10 s / 60 s, corps borné à 8 Mio), lecture par `.bytes()` puis `serde_json` ;
  `payouts(&Window)` (`arrival_date[gte]` = début de la fenêtre, `arrival_date[lt]` = sa fin, `status=paid`, `limit=100`),
  `balance_transactions(payout, &Window)` ; pagination `starting_after` tant que `has_more`, garde d'une page vide ou d'un
  curseur immobile (`reason: "cursor"`) et de 1000 pages (`too_many_pages`). `fetch_rows` : versements, puis transactions
  des seuls versements automatiques par `run_bounded` (au plus `parallel`), versements manuels rendus par identifiant
  (`manual_payouts`).
- **Traduction (AC-3, T3)** : `crates/kesh-api/src/stripe/rows.rs`, fonctions pures — jours de Zurich
  (`jour_suisse`), `created` en heure de Zurich `AAAA-MM-JJ HH:MM:SS`, montants entiers → `Decimal` à l'exposant
  (quinze devises à zéro décimale, cinq à trois décimales refusées en `AMOUNT`), contrôles du lecteur CSV (identifiant par
  `valid_id`, devise, `NET_MISMATCH`, `PAYOUT_CURRENCY_MIXED`, `DATE` hors années 1000 à 9999, catégorie vide `ID`),
  transaction du versement écartée (`is_own_payout_transaction` : type **et** source), Σ net = `amount` du versement
  (`PROVIDER_RESPONSE_INVALID` `{ payoutId, expected, found }`), doublon d'une transaction entre deux versements refusé
  (`duplicate_transaction`) ; versements rangés par (`arrival_date`, `id`), lignes numérotées à partir de 2.
- **Erreurs (AC-4, T4)** : classement du socle (`classify_status`, `classify_transport`, `with_retry`) ; 401
  `PROVIDER_AUTH_FAILED` sans reprise, 403 `PROVIDER_PERMISSION_DENIED`, autre statut `PROVIDER_API_ERROR` avec
  `window` (période) et `page` ; ressource, `error.code`, `error.param` (64 caractères au plus) et
  `Stripe-Rate-Limited-Reason` au journal seulement ; jamais `error.message` ni la clé.
- **Tests et fixtures (AC-5, T4)** : serveur `axum` dans le processus (`tests/stripe_client.rs`), réponses fictives
  `tests/fixtures/stripe-api/` (README), tirées de la fixture CSV de 40-7a ; les lignes obtenues égalent celles du CSV
  pour les trois mêmes versements (`charge_id` mis à `None` côté CSV, l'API ne le rend pas). Clé de test porteuse du
  marqueur S9 (`common::secrets_prestataire`), absence vérifiée dans les corps d'erreur et le journal capté (témoins
  exigés). Aucun appel à `api.stripe.com`.

### Constat sur les réponses réelles (lecture seule, données jamais versées)

`?payout=` rend la transaction du versement lui-même pour 104 versements payés sur 104 ; Σ net des autres = `amount`
pour les 104. Les identifiants et les montants des fixtures sont inventés.

### Tests ajoutés (recomptés par `grep -cE '#\[(tokio::)?test\]'`, de `73fa1573` à `HEAD`)

26 : `stripe/rows.rs` 10, `stripe/mod.rs` 3, `tests/stripe_client.rs` 13 ; `provider_env.rs` inchangé (8).

### Mutations (copie `git archive` hors dépôt, cible cargo propre, supprimées ensuite ; journal `kesh-gate-logs/s40-8a/mutations.log`)

Copie non mutée verte d'abord (30 tests du filtre `binary(stripe_client) | test(stripe::)`). Onze mutations, **onze
tuées**, chacune par une assertion d'un test nommé (`panicked at` et ligne `FAIL` du test attendu) : garde de la
transaction du versement retirée (`lit_deux_pages_et_rend_les_lignes_du_rapport_csv`) ; source ignorée
(`seule_la_transaction_du_versement_lui_meme_est_ecartee`) ; jour UTC au lieu de Zurich
(`trente_et_un_decembre_utc_est_le_premier_janvier_a_zurich`) ; clé non contrôlée (`configuration_de_la_cle`) ; Σ net
non contrôlée (`transaction_manquante_echoue`) ; `Stripe-Version` retiré, filtre `created` au lieu d'`arrival_date`
(`lit_deux_pages_et_rend_les_lignes_du_rapport_csv`) ; zéro décimale ignorée
(`zero_decimale_et_trois_decimales_refusees`) ; 401 rendu `PROVIDER_API_ERROR` (`cle_refusee`) ; garde du curseur
immobile retirée (`curseur_immobile`) ; crochet `check` débranché (`configuration_de_la_cle`). Une première forme de la
mutation du 401 ne compilait pas (`match` non exhaustif) : non comptée, remplacée.

### Gates (au dernier commit de code `f203c135`)

- `scripts/test-fast.sh --ci` sur base remise à zéro (`kesh_s408a`) : `cargo fmt --check`, `clippy --workspace
  --all-targets -D warnings` verts ; nextest **6526 exécutés, 6526 passés**, 15 ignorés
  (`kesh-gate-logs/s40-8a/gate-backend.log`) ; `cargo publish --dry-run -p kesh-import` vert (`valid_id` public).
- Frontend : non touché, aucun gate frontend. E2E : aucune spec ajoutée, aucun E2E (règle de la nuit du 22.11).
  `gate-ci.sh` et E2E complet à la branche d'intégration, en fin de nuit.

### Hors périmètre, relevé

- Le statut réel d'un droit manquant (400 ou 403) et la concordance de `arrival_date` avec
  `automatic_payout_effective_at` du rapport restent à constater à la recette (40-8b), comme la fiche le dit.
- `fetch_rows` ne rend pas de progression : la tâche et ses étapes sont celles de 40-8b.
- Une branche Wise qui ajoutera sa `ProviderSpec` devra porter le champ `check: None` (conflit de compilation attendu à
  la fusion, sans conflit de texte).

### File List

- `crates/kesh-api/src/stripe/mod.rs` (nouveau), `crates/kesh-api/src/stripe/rows.rs` (nouveau),
  `crates/kesh-api/src/lib.rs`, `crates/kesh-api/src/provider_env.rs`, `crates/kesh-import/src/stripe.rs`
- `crates/kesh-api/tests/stripe_client.rs` (nouveau), `crates/kesh-api/tests/fixtures/stripe-api/` (neuf fichiers JSON et
  un README, nouveaux)
- `docker-compose.yml`, `docker-compose.prod.yml`, `.env.example`
- `_bmad-output/implementation-artifacts/40-8a-client-api-stripe.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`

