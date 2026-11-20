# Story 38-12a : téléchargement des transactions par l'API PayPal (identifiants, client, conversion, tâche)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal` (phase 1 au commit `90428594`). Epic 38, **phase 2**
(décisions de Beat du 2026-11-20, `kesh-gate-logs/epic-38-kiss/phase2-decisions.md`). **Dépend de** : 38-11a (modèle
`PaypalRow`), 38-11b (comptes PayPal, ventilation et doublons), 38-11c (le rapprochement lit `bank_transactions`, rien à
changer). **Suivie par** : 38-12b (écran, manuels, E2E). Scindée de 38-12b par la règle de splitting préventif : une seule
fiche aurait porté l'outillage réseau, le travail en tâche de fond, l'écran, deux manuels et un E2E.*

## Story

En tant qu'**Administrateur**, je veux poser dans l'environnement du serveur, comme le mot de passe SMTP, les
identifiants d'une application PayPal en lecture seule par compte PayPal, afin qu'en tant que **Comptable** je puisse
faire télécharger par Kesh les transactions d'une période, sans passer par le CSV, avec les mêmes doublons et le même
rapprochement.

## Ce qui est confirmé de l'API PayPal, et ce qui ne l'est pas

*Relevé du 2026-11-20 sur le schéma OpenAPI publié par PayPal (`developer.paypal.com/api/transaction-search/v1/schema.json`)
et la page des codes d'événement. À rejouer au développement ; ce n'est pas une preuve contre un compte réel.*

| Point | État |
|---|---|
| `GET /v1/reporting/transactions` ; `start_date`, `end_date` au format RFC 3339, secondes obligatoires | **confirmé** |
| plage maximale **31 jours** | **confirmé** (description d'`end_date`) |
| `page_size` 1 à 500, défaut 100 ; `page` à partir de 1 ; `total_pages` dans la réponse | **confirmé** (schéma ; `total_pages` : page de l'opération, non relu champ à champ) |
| `fields=all` (défaut `transaction_info`) ; `balance_affecting_records_only` `Y` par défaut | **confirmé** |
| une transaction met **jusqu'à trois heures** à paraître ; l'appel couvre les **trois dernières années** | **confirmé** (notes de l'opération) |
| `ending_balance` vide dès qu'un filtre facultatif est donné ; l'identifiant n'est pas unique dans le système de rapport (une ligne qui affecte le solde et une qui ne l'affecte pas peuvent le partager) | **confirmé** |
| erreurs : 400 (dont `RESULTSET_TOO_LARGE`), 403 (droits insuffisants), 500, `SERVICE_UNAVAILABLE` | **confirmé** (schéma) ; la **limite** de résultats derrière `RESULTSET_TOO_LARGE` : **non confirmée** |
| **429** et en-tête `Retry-After` | **non confirmé** : absents du schéma ; traités par précaution |
| hôte de test `api-m.sandbox.paypal.com` | **confirmé** ; hôte réel `api-m.paypal.com` : de mémoire, **non relu** |
| OAuth : `POST /v1/oauth2/token`, authentification Basic `client_id:secret`, corps `grant_type=client_credentials`, `access_token`, `expires_in` | **non confirmé** (page de référence introuvable, 404) : de mémoire |
| nom exact de la fonction à cocher à la création de l'application, conditions d'accès du compte (type, activation) | **non confirmés** ; « Transaction Search » est le nom de la décision de Beat |
| signe de `fee_amount` (négatif attendu), libellé du CSV pour le statut `V`, fuseau de `transaction_initiation_date` (« fuseau du compte ») | **non confirmés** |
| correspondance code d'événement ↔ libellé du CSV (`Type`) | **partielle** : la page de PayPal décrit les codes, pas les libellés du CSV ; ceux-ci viennent de la fixture de 38-11a |

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-12a-1` | Les identifiants se posent par variables d'environnement, une paire par compte PayPal, rattachée à son libellé ; rien en base, rien dans la sauvegarde. | AC-1, AC-2 |
| `FR-38-12a-2` | Les transactions d'une période se téléchargent par fenêtres de 31 jours, pages de 500, avec reprise après une limitation de débit. | AC-4, AC-6 |
| `FR-38-12a-3` | Ce qui est téléchargé prend la forme du CSV de 38-11a, puis passe par la ventilation et les doublons de 38-11b. | AC-5, AC-7 |
| `FR-38-12a-4` | Le téléchargement tourne en tâche de fond ; sa progression et son résultat se lisent par une route. | AC-8 |
| `FR-38-12a-5` | Le parallélisme est réglé par `KESH_PAYPAL_PARALLEL` (1 par défaut, 6 au plus). | AC-3, AC-6 |
| `NFR-38-12a-1` | Aucun secret dans un journal, une réponse, une erreur ni une trace d'audit. | AC-2, AC-9 |
| `NFR-38-12a-2` | Aucune donnée réelle ni appel au vrai PayPal dans les tests. | AC-10 |

## Critères d'acceptation

1. **AC-1 — Variables par compte PayPal** (`crates/kesh-api/src/config.rs`, au patron de `KESH_SMTP_*`, `:1078-1081`,
   `env_nonempty` `:1406`). Pour chaque `n` de 1 à 9 : `KESH_PAYPAL_<n>_ACCOUNT` (le **libellé** du compte PayPal,
   exactement celui du réglage `provider_account` de 38-11b), `KESH_PAYPAL_<n>_CLIENT_ID`, `KESH_PAYPAL_<n>_SECRET`.
   Les trois ensemble ou aucune ; deux libellés égaux (sans casse) refusés. Facultatives : `KESH_PAYPAL_BASE_URL`
   (défaut `https://api-m.paypal.com` ; `https://api-m.sandbox.paypal.com` pour un essai ; `http` admis seulement vers
   une adresse de bouclage, pour les tests), `KESH_PAYPAL_PARALLEL` (AC-3). Tout est lu **une fois**, au démarrage, dans
   `Config` ; un compte PayPal de Kesh dont le libellé n'a pas de paire reste sans téléchargement par l'API (le CSV
   continue).
2. **AC-2 — Validation au démarrage et secrets.** Comme `KESH_SMTP_*` : paire incomplète, libellé vide ou numéro hors
   `1..=9` → **refus de démarrer**, message qui nomme les variables, jamais leur valeur. Une valeur de la forme `<…>`
   (`is_template_placeholder`, `:1538`) est refusée comme pour le mot de passe SMTP. Le secret et l'identifiant sont des
   champs **privés** de `Config`, lus par accesseur unique ; `Debug` les masque (test sur le patron de
   `smtp_password_masked_in_debug`, `:3484`). Aucune colonne, aucune table, rien dans `.keshbackup`.
3. **AC-3 — Parallélisme.** `KESH_PAYPAL_PARALLEL` entier strict : absent = **1** ; hors `1..=6` ou illisible → refus
   de démarrer (pas de repli silencieux). Il borne le nombre de fenêtres demandées en même temps pour l'ensemble du
   serveur (un seul lot de permis partagé par tous les téléchargements).
4. **AC-4 — Fenêtres.** Fonction pure `windows(from, to) -> Vec<(start, end)>` : jours de **Zurich** (horloge
   injectable de l'Epic 35), `from` 00:00:00 à `to` 23:59:59, fenêtres contiguës de **31 jours au plus** (fin d'une
   fenêtre = seconde précédant le début de la suivante), décalage horaire de la date réelle (heure d'été comprise).
   Refus 400 `PAYPAL_PERIOD_INVALID` si `from` > `to` ou si `to` est dans le futur, 400 `PAYPAL_PERIOD_TOO_OLD` au-delà
   de **trois ans** en arrière (limite de l'API). Pages de **500**, `page` de 1 à `total_pages`, `fields=all`,
   `balance_affecting_records_only=Y`. Sur 400 `RESULTSET_TOO_LARGE`, la fenêtre se coupe en deux et chaque moitié est
   reprise (récursif, plancher d'un jour : au-delà, échec nommant la fenêtre).
5. **AC-5 — Client et conversion.** Module `crates/kesh-api/src/paypal_api/` (client, jeton, pages) sur `reqwest`
   **déjà en dépendance** (`Cargo.toml`, rustls, comme `exchange_rates/source.rs`) — aucune dépendance neuve. Jeton :
   un par compte, gardé en mémoire jusqu'à une marge avant `expires_in`, renouvelé une fois sur 401, puis échec
   `PAYPAL_AUTH_FAILED` (identifiants refusés : le message n'imprime que le code d'erreur de PayPal). Conversion pure
   `kesh_import::paypal_api::to_paypal_row(&PaypalApiTx, line) -> Result<PaypalRow, PaypalApiError>` (module neuf de
   `kesh-import`, **sans dépendance interne ni réseau** ; `kesh-api` lit le JSON vers `PaypalApiTx`). Correspondance
   champ à champ avec 38-11a :

   | `PaypalRow` (CSV) | Source dans l'API |
   |---|---|
   | `date`, `time`, `time_zone` | `transaction_info.transaction_initiation_date` : date et heure **telles que données**, sans conversion ; `time_zone` = le décalage (`+01:00`) ; l'en-tête `PayPal-Enforce-ISO8601-Format: true` est envoyé |
   | `name` | `payer_info.payer_name` (nom complet, sinon prénom et nom) |
   | `kind` | table code d'événement → libellé du CSV (ci-dessous) ; code inconnu → le code lui-même (`T9999`), la ligne ira « à examiner » (38-11c) |
   | `status` | `S` → `Completed`, `P` → `Pending`, `D` → `Denied`, `V` → `Reversed` ; autre → refus de la ligne |
   | `currency`, `gross` | `transaction_amount.currency_code`, `.value` |
   | `fee` | `fee_amount.value` ; absent → 0 ; positif refusé (`FEE_POSITIVE`, contrôle de 38-11a) |
   | `net` | **calculé** `gross + fee` (l'API n'en donne pas) |
   | `from_email` / `to_email` | `payer_info.email_address`, placé en `from_email` pour un crédit, en `to_email` pour un débit ; l'autre côté `None` |
   | `transaction_id` | `transaction_info.transaction_id` (mêmes contraintes que 38-11a AC-3) |
   | `reference_txn_id` | `transaction_info.paypal_reference_id` |
   | `balance_impact` | `Credit` si `gross > 0`, sinon `Debit` (la demande n'admet que les lignes qui affectent le solde : pas de `Memo`) |
   | `balance` | `None` (`ending_balance` est vide dès qu'un filtre est posé) ; le `closing_balance` du `bank_imports` reste `NULL` |
   | `item_title` | `cart_info.item_details[].item_name` joints par « · » |
   | `subject`, `note`, `invoice_number` | `transaction_subject`, `transaction_note`, `transaction_info.invoice_id` (à défaut `cart_info.paypal_invoice_id`) |
   | `line` | rang de la transaction dans le téléchargement, à partir de 1 |

   Table des types, **fermée**, en constante testée (page de PayPal, codes seulement) : `T0001` mass payment ;
   `T0002` subscription payment ; `T0003` preapproved payment ; `T0006` Express Checkout ; `T0007` standard web
   checkout ; `T0013` donation ; `T0200` general currency conversion ; `T0201` user-initiated currency conversion ;
   `T0400` general withdrawal ; `T1105` reversal of general account hold ; `T1107` payment refund ; `T1110` hold on
   balance for dispute investigation ; `T2105` / `T2106` payment review hold / release. **Le libellé de CHAQUE code est
   celui que porte le CSV de la fixture de 38-11a** (`Subscription Payment`, `Website Payment`, `General Withdrawal`,
   `User Initiated Currency Conversion`, `Payment Review Hold`…), lu sur elle au développement, jamais deviné ; **non
   confirmé** : le libellé exact du CSV pour `T0006`, `T0007`, `T0300` et les autres codes de retenue (`T1500`,
   `T2101` à `T2104`, `T2107`, `T2108`). Un code absent de la table donne une ligne « à examiner », jamais une ligne
   ignorée.
6. **AC-6 — Débit, limitation, reprise, parallélisme.** Un seul fil de fenêtres quand `KESH_PAYPAL_PARALLEL` = 1 ;
   sinon jusqu'à N fenêtres en vol (permis partagés, AC-3), les résultats rangés par fenêtre puis par page. Sur **429**
   (ou 503) : pause **commune** à tous les fils (l'instant de reprise se partage), durée = `Retry-After` en secondes si
   l'en-tête est présent et entier, sinon 5, 10, 20, 40 puis 60 secondes ; **la même requête est reprise** (pas la
   période), six essais au plus par page, puis échec nommant la fenêtre et la page. Pause et attente injectables
   (`PaypalTiming`) : les tests ne dorment pas. Autres statuts : 401 une fois renouvelé (AC-5) ; 403
   `PAYPAL_PERMISSION_DENIED` (l'application n'a pas la fonction, message qui le dit) ; 5xx → mêmes essais que 429 ; le
   reste `PAYPAL_API_ERROR` avec le nom d'erreur de PayPal.
7. **AC-7 — Même ventilation, mêmes doublons.** Toutes les fenêtres sont lues en mémoire, **puis** passent, comme les
   lignes d'un CSV, par la fonction de ventilation et d'import de 38-11b (extraite si besoin en fonction partagée
   `paypal_import::import_rows`, comportement de la route CSV inchangé, ses tests verts) : mêmes filtres (statut,
   paires neutres), mêmes comptes par devise, mêmes verrous, **une seule transaction de base** (tout ou rien). Les
   doublons se jugent par identifiant de transaction sur le compte bancaire (brut ou `:FEE`) : un CSV puis un
   téléchargement, ou l'inverse, ou deux téléchargements qui se chevauchent, ne créent rien en double. L'empreinte
   (`file_hash`) d'un téléchargement est le SHA-256 des identifiants triés ‖ devise ; le refus
   `BANK_IMPORT_DUPLICATE_FILE` **ne s'applique pas** au téléchargement (relancer la même période rend `inserted: 0`).
   Un identifiant présent deux fois dans la même réponse est gardé une fois. Le rapprochement (38-11c) lit ensuite ces
   lignes sans changement.
8. **AC-8 — Tâche et routes.** `POST /api/v1/paypal-downloads` (corps `{ account, from, to }` ; **Comptable et plus** ;
   **clés d'API non admises** : la route fait sortir un appel authentifié par un secret de l'Administrateur ; inscrite
   au registre des routes auditées, `crates/kesh-api/tests/audit_route_registry.rs`, et à `LIB_ROUTES` si le dépôt la
   compte). Refus dans l'ordre, rien lancé : 400 `VALIDATION_ERROR` → 404 `PAYPAL_ACCOUNT_NOT_FOUND` (38-11b) → 422
   `PAYPAL_API_NOT_CONFIGURED` (aucune paire pour ce libellé) → 400 `PAYPAL_PERIOD_INVALID` / `PAYPAL_PERIOD_TOO_OLD` →
   409 `PAYPAL_DOWNLOAD_RUNNING` (une tâche tourne déjà pour ce compte PayPal) ; sinon **202** `{ jobId }`.
   `GET /api/v1/paypal-downloads/{jobId}` (même rôle, **société de l'appelant seulement**, sinon 404) : `{ state:
   "running" | "done" | "failed", windows: { done, total }, pausedUntilSeconds: number | null, result: <réponse 201 de
   38-11b> | null, error: { code, message, window?, page? } | null }`. La tâche vit **en mémoire** du processus (table
   partagée de l'état d'application, purgée une heure après sa fin), **perdue au redémarrage** : relancer est sans risque
   (AC-7). Rien n'est écrit en base avant la fin : une tâche échouée n'a rien changé. Trace d'audit
   `paypal_download.created` (`details` : compte PayPal, période, fenêtres, compte rendu) à la fin réussie ; aucun
   secret, aucun jeton.
9. **AC-9 — Pas de fuite.** Aucun journal (`tracing`), message d'erreur, réponse ni trace n'imprime `client_id`, secret
   ni jeton ; test qui rejoue un échec d'authentification et un échec de page et cherche ces trois chaînes dans la
   réponse et dans les journaux capturés. `KESH_PAYPAL_BASE_URL` n'est jamais renvoyée par l'API.
10. **AC-10 — Serveur PayPal simulé.** `crates/kesh-api/tests/paypal_download_e2e.rs` lance **dans le processus** un
    petit serveur `axum` sur `127.0.0.1:0` (dépendance existante) qui joue : le jeton, des pages à `total_pages`
    réglable, un 429 (avec et sans `Retry-After`), un 400 `RESULTSET_TOO_LARGE`, un 401, un 403, une transaction de
    chaque statut, une paire neutre, un doublon d'identifiant. Données **fictives** (identifiants `0TEST…`, e-mails
    `@example.invalid`). Cas : période de quatre mois → fenêtres attendues et ventilation de 38-11b par devise ; CSV
    (fixture de 38-11a) puis téléchargement des mêmes transactions → rien de doublé ; 429 puis reprise ; parallélisme 3
    → mêmes lignes qu'à 1 ; échec à la fenêtre 3 → rien d'écrit ; deux tâches du même compte → 409 ; secrets absents
    des journaux (AC-9). Tests unitaires : `windows` (31 jours, bascule d'heure d'été, fin d'année), `to_paypal_row`
    (chaque ligne du tableau de l'AC-5), configuration (AC-1 à AC-3, patron des tests de `KESH_SMTP_*`).

## Tâches

- [ ] T1 — `Config` : variables, validation, masque `Debug` (AC-1 à AC-3).
- [ ] T2 — `kesh-import::paypal_api` : types, table des types, conversion (AC-5).
- [ ] T3 — `windows` et client : jeton, pages, limitation, parallélisme (AC-4, AC-5, AC-6).
- [ ] T4 — Fonction d'import partagée avec 38-11b (AC-7).
- [ ] T5 — Tâche en mémoire, routes, audit, erreurs et messages aux quatre locales (AC-8, AC-9).
- [ ] T6 — Serveur simulé et tests (AC-10).

## Stratégie de tests

Gate ciblé : `binary(paypal_download_e2e)`, les tests de `kesh-import` et ceux de `config`. **Gate complet** au dernier
commit (la fonction d'import de 38-11b est touchée : ses tests, ceux des imports CAMT/CSV et `audit_route_registry`
restent verts sans changement d'assertion). Aucune migration : P5 à P8 sans objet.

## Modules touchés (3)

`kesh-import` (conversion pure), `kesh-api` (configuration, client, tâche, routes, erreurs), `kesh-i18n`.

## Fichiers communs

`crates/kesh-api/src/config.rs`, `lib.rs` (routes, état d'application), `errors.rs`, catalogues i18n (leur bloc),
`crates/kesh-api/src/routes/paypal_imports.rs` de 38-11b (fonction partagée) : à relire à la reprise de `main`.

## Note de version

*Ajouté* — Kesh peut télécharger les transactions d'un compte PayPal par l'API de PayPal, par périodes, au lieu du CSV ;
les identifiants se posent dans l'environnement du serveur (`KESH_PAYPAL_<n>_…`), une application PayPal en lecture seule
par compte ; les doublons se reconnaissent à l'identifiant de transaction, le CSV et le téléchargement se mélangent.
*Limite* — une transaction peut mettre jusqu'à trois heures à paraître dans l'API.

## Notes de développement

- Aucun appel au vrai PayPal en test ni en CI ; la première exécution réelle est celle de Beat, avec ses identifiants.
- Si PayPal contredit une des hypothèses « non confirmées » (signe des frais, libellé `V`, fuseau, libellés de type), la
  correction va à la table ou à la conversion, jamais aux règles de 38-11c.

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
