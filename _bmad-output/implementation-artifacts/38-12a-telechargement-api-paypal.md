# Story 38-12a : téléchargement des transactions par l'API PayPal (identifiants, client, conversion, tâche)

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal` (phase 1 au commit `90428594`). Epic 38, **phase 2**
(décisions de Beat du 2026-11-20, `kesh-gate-logs/epic-38-kiss/phase2-decisions.md`, autorisations du soir comprises).
**Dépend de** : 38-11a (modèle `PaypalRow`, `crates/kesh-import/src/paypal.rs`, développé au commit `1608bb62`), 38-11b
(comptes PayPal, ventilation et doublons), 38-11c1 et 38-11c2 (le rapprochement lit `bank_transactions`, rien à
changer). **Suivie par** : 38-12b (bouton, manuels, E2E). Scindée de 38-12b par la règle de splitting préventif : une
seule fiche aurait porté l'outillage réseau, le travail en tâche de fond, l'écran, deux manuels et un E2E.*

## Story

En tant qu'**Administrateur**, je veux poser dans l'environnement du serveur, comme le mot de passe SMTP, les
identifiants d'une application PayPal en lecture seule par compte PayPal, afin qu'en tant que **Comptable** je puisse
faire télécharger par Kesh les transactions d'une période, sans passer par le CSV, avec les mêmes doublons et le même
rapprochement.

## Ce qui est confirmé de l'API PayPal, et ce qui ne l'est pas

*Relevé du 2026-11-20 sur le schéma OpenAPI publié par PayPal (`developer.paypal.com/api/transaction-search/v1/schema.json`),
la page d'authentification (`developer.paypal.com/api/rest/authentication/`), la page de limitation de débit
(`developer.paypal.com/api/rest/reference/rate-limiting/`) et la page des codes d'événement ; complété à la validation P1
le même jour. Ce n'est pas une preuve contre un compte réel : les points « non confirmés » se vérifient au développement
par des appels réels en lecture seule (Notes de développement).*

| Point | État |
|---|---|
| `GET /v1/reporting/transactions` ; `start_date`, `end_date` au format RFC 3339, secondes obligatoires | **confirmé** |
| plage maximale **31 jours** | **confirmé** (description d'`end_date`) ; mesurée en jours ou en heures : **non dit** (d'où les fenêtres de 30 jours, AC-4) |
| `page_size` 1 à 500, défaut 100 ; `page` à partir de 1 ; `total_pages`, `total_items`, `last_refreshed_datetime` dans la réponse | **confirmé** (schéma `search_response`) |
| `fields=all` (défaut `transaction_info`) ; `balance_affecting_records_only` `Y` par défaut | **confirmé** |
| une transaction met **jusqu'à trois heures** à paraître ; l'appel couvre les **trois dernières années** | **confirmé** (notes de l'opération) |
| `ending_balance` vide dès qu'un filtre facultatif est donné ; l'identifiant n'est pas unique dans le système de rapport (une ligne qui affecte le solde et une qui ne l'affecte pas peuvent le partager) | **confirmé** |
| `transaction_initiation_date` est exprimée **dans le fuseau du compte** de ce côté du paiement | **confirmé** (schéma) |
| `fee_amount` n'a de valeur que si des frais s'appliquent | **confirmé** (schéma) ; son **signe** (négatif attendu, positif sur un remboursement comme au CSV) : **non confirmé** |
| `transaction_id` : `[a-zA-Z0-9_]{1,24}` au schéma ; `paypal_reference_id` : `[a-zA-Z0-9_-]{1,24}`, de type `TXN`, `ODR`, `SUB` ou `PAP` (`paypal_reference_id_type`) | **confirmé** (schéma) |
| `payer_info.payer_name` : `full_name`, `alternate_full_name`, `given_name`, `surname`… | **confirmé** (schéma `name`) |
| `payer_info` d'un **débit** (paiement envoyé) : la contrepartie ou le titulaire du compte | **non confirmé** (le schéma dit « the payer ») |
| erreurs : 400 (dont `RESULTSET_TOO_LARGE`), 403 (droits insuffisants), 500 | **confirmé** (schéma) ; la **limite** de résultats derrière `RESULTSET_TOO_LARGE` : **non confirmée** |
| **429** `RATE_LIMIT_REACHED` | **confirmé** (page de limitation de débit) ; en-tête **`Retry-After`** : **non documenté**, lu par précaution |
| hôte de test `api-m.sandbox.paypal.com` | **confirmé** ; hôte réel `api-m.paypal.com` : cité par les portées de l'exemple d'authentification, **non dit** comme hôte de production |
| OAuth : `POST /v1/oauth2/token`, authentification Basic `client_id:secret`, corps `grant_type=client_credentials` en `application/x-www-form-urlencoded`, réponse `access_token`, `token_type`, `expires_in` (secondes) | **confirmé** (page d'authentification) |
| nom exact de la fonction à cocher à la création de l'application, conditions d'accès du compte (type, activation) | **non confirmés** ; « Transaction Search » est le nom de la décision de Beat |
| libellé du CSV pour le statut `V` | **non confirmé** (sans effet : tout statut autre que `Completed` est écarté par 38-11b) |
| correspondance code d'événement ↔ libellé du CSV (`Type`) | **partielle** : la page de PayPal décrit les codes, pas les libellés du CSV (AC-5) |

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-12a-1` | Les identifiants se posent par variables d'environnement, une paire par compte PayPal, rattachée à son libellé ; rien en base, rien dans la sauvegarde. | AC-1, AC-2 |
| `FR-38-12a-2` | Les transactions d'une période se téléchargent par fenêtres (31 jours au plus, limite de PayPal), pages de 500, avec reprise après une limitation de débit. | AC-4, AC-6 |
| `FR-38-12a-3` | Ce qui est téléchargé prend la forme du CSV de 38-11a, puis passe par la ventilation et les doublons de 38-11b. | AC-5, AC-7 |
| `FR-38-12a-4` | Le téléchargement tourne en tâche de fond ; sa progression et son résultat se lisent par une route. | AC-8 |
| `FR-38-12a-5` | Le parallélisme est réglé par `KESH_PAYPAL_PARALLEL` (1 par défaut, 6 au plus). | AC-3, AC-6 |
| `NFR-38-12a-1` | Aucun secret dans un journal, une réponse, une erreur ni une trace d'audit. | AC-2, AC-9 |
| `NFR-38-12a-2` | Aucune donnée réelle ni appel au vrai PayPal dans les tests ni en CI. | AC-10 |

## Critères d'acceptation

1. **AC-1 — Variables par compte PayPal** (`crates/kesh-api/src/config.rs`, lues par `env_nonempty`, `:1406`, comme
   `KESH_SMTP_*`, `:1078-1081`). Pour chaque `n` de 1 à 9 : `KESH_PAYPAL_<n>_ACCOUNT` (le **libellé** du compte PayPal,
   exactement celui du réglage `provider_account` de 38-11b), `KESH_PAYPAL_<n>_CLIENT_ID`, `KESH_PAYPAL_<n>_SECRET`.
   Les trois ensemble ou aucune ; deux libellés égaux (sans casse) refusés. Facultatives : `KESH_PAYPAL_BASE_URL`,
   **ensemble fermé** — `https://api-m.paypal.com` (défaut) ou `https://api-m.sandbox.paypal.com` ; une URL `http` vers
   `127.0.0.1` ou `localhost` n'est admise que sous `KESH_TEST_MODE=true` (serveur simulé des tests et de l'E2E) ; toute
   autre valeur refusée —, et `KESH_PAYPAL_PARALLEL` (AC-3). Tout est lu **une fois**, au démarrage, dans `Config` ; un
   compte PayPal de Kesh dont le libellé n'a pas de paire reste sans téléchargement par l'API (le CSV continue). Les
   identifiants appartiennent à l'**installation** (une installation porte une société, manuel d'administration
   `:1079`) : limite écrite au manuel par 38-12b.
2. **AC-2 — Validation au démarrage et secrets.** Contrairement à `KESH_SMTP_*` — qui ne refuse de démarrer que si la
   récupération de mot de passe est activée (`:1120`) et retombe en silence sur le port par défaut —, la validation
   PayPal est **inconditionnelle** : paire incomplète, libellé vide, numéro hors `1..=9` (p. ex. `KESH_PAYPAL_10_…` ou
   `KESH_PAYPAL_0_…`, repérés en parcourant les variables de préfixe `KESH_PAYPAL_`), libellés en double, URL de base
   hors de l'ensemble → **refus de démarrer** (`ConfigError` neuve), message qui nomme les variables, jamais leur
   valeur. Une valeur de la forme `<…>` (`is_template_placeholder`, `:1538`) est refusée, comme le mot de passe
   administrateur (`:650`) et le secret JWT (`:683`). Le secret et l'identifiant sont des champs **privés** de
   `Config`, lus par accesseur unique ; `Debug` les masque (test sur le patron de `smtp_password_masked_in_debug`,
   `:3484`). Aucune colonne, aucune table, rien dans `.keshbackup`.
3. **AC-3 — Parallélisme.** `KESH_PAYPAL_PARALLEL` entier strict : absent = **1** ; hors `1..=6` ou illisible → refus
   de démarrer (pas de repli silencieux). Il borne le nombre de requêtes de fenêtres en vol pour l'ensemble du serveur
   (un seul sémaphore de l'état d'application, partagé par tous les téléchargements).
4. **AC-4 — Fenêtres.** Fonction pure `windows(from, to, now) -> Vec<(start, end)>` sur les jours de **Zurich** :
   bornes par `kesh_core::horloge::debut_du_jour_suisse` et `fin_du_jour_suisse` (Story 36-10 ; aucun décalage
   recalculé), fenêtres contiguës de **30 jours de Zurich** au plus — soit au plus 30 × 24 h + 1 h, **sous** la limite
   de 31 jours de PayPal même quand la fenêtre contient le passage à l'heure d'hiver, ce qu'une fenêtre de 31 jours de
   Zurich dépasserait d'une heure —, fin d'une fenêtre = seconde précédant le début de la suivante, envoyées en UTC
   (`…Z`), à la seconde. La fin de la dernière fenêtre est **ramenée à `now`** si elle est plus tard (période qui finit
   aujourd'hui). `now` et le jour du jour viennent de l'horloge de l'état d'application (`state.horloge`, Epic 35 ;
   garde `date_du_jour_garde.rs`) ; un délai de jeton (AC-5) se mesure en `Instant`, non en date. Refus 400
   `PAYPAL_PERIOD_INVALID` si `from` > `to` ou si `to` est après aujourd'hui, 400 `PAYPAL_PERIOD_TOO_OLD` si `from` est
   avant le même jour, trois ans plus tôt (limite de l'API). Pages de **500**, `page` de 1 à `total_pages`,
   `fields=all`, `balance_affecting_records_only=Y`. Sur 400 `RESULTSET_TOO_LARGE`, la fenêtre se coupe en deux et
   chaque moitié est reprise (récursif, plancher d'un jour : au-delà, échec nommant la fenêtre ; la progression compte
   alors les moitiés).
5. **AC-5 — Client et conversion.** Module `crates/kesh-api/src/paypal_api/` (client, jeton, pages) sur `reqwest`
   **déjà en dépendance** (`crates/kesh-api/Cargo.toml:112`, rustls, sans la fonction `json` : les corps se lisent en
   octets) — aucune dépendance neuve. Client construit **comme celui des cours** (`exchange_rates/source.rs:117-131`) :
   aucune redirection suivie (le jeton ne part jamais ailleurs), délais de connexion et total, HTTPS seul hors tests,
   `User-Agent` sans donnée personnelle, **corps de réponse borné** (une page de 500 transactions, plafond écrit en
   constante) — sans délai, une requête pendue laisserait la tâche « en cours » et le compte refusé en 409 jusqu'au
   redémarrage. Jeton : **un par tâche**, demandé au début, renouvelé à l'approche de `expires_in` et une fois sur 401,
   puis échec `PAYPAL_AUTH_FAILED` (identifiants refusés : le message n'imprime que le code d'erreur de PayPal) ; aucun
   cache partagé entre tâches. **Lecture et conversion pures dans `kesh-import`** (module neuf
   `crates/kesh-import/src/paypal_api.rs`, sans dépendance interne ni réseau ; `serde` et `serde_json` y sont déjà) :
   `parse_transactions_page(&[u8]) -> Result<PaypalApiPage, PaypalApiError>` (transactions, `total_pages`,
   `last_refreshed_datetime`) et `to_paypal_row(&PaypalApiTx, line) -> Result<PaypalRow, PaypalLineError>`. Les
   **contrôles de 38-11a** (identifiant `[0-9A-Z]` après mise en majuscules — un `_` admis par le schéma est refusé comme
   au CSV —, devise de trois lettres, montant par `parse_dot_decimal_amount`, signe du brut selon le sens, `net = gross +
   fee`) sont **ceux de `read_row`** (`paypal.rs`), extraits en fonctions partagées, non recopiés ; les erreurs ont la
   forme de `PaypalLineError` (champ = nom du champ de l'API). Correspondance champ à champ avec `PaypalRow` :

   | `PaypalRow` (CSV) | Source dans l'API |
   |---|---|
   | `date`, `time`, `time_zone` | `transaction_info.transaction_initiation_date` (fuseau du compte, confirmé) : date et heure **telles que données**, sans conversion, comme au CSV ; `time_zone` = le décalage (`+01:00`) là où le CSV écrit un nom (`CET`) — affichage seulement ; l'en-tête `PayPal-Enforce-ISO8601-Format: true` est envoyé |
   | `name` | `payer_info.payer_name` : `full_name`, sinon `alternate_full_name`, sinon `given_name` et `surname` joints ; aucun → `None` |
   | `kind` | table code d'événement → libellé du CSV (ci-dessous) ; code inconnu → le code lui-même (`T9999`), la ligne ira en `Other` puis « à examiner » (38-11c2 AC-4) |
   | `status` | `S` → `Completed`, `P` → `Pending`, `D` → `Denied`, `V` → `Reversed` ; autre lettre → erreur de ligne (champ `transaction_status`) |
   | `currency`, `gross` | `transaction_amount.currency_code`, `.value` |
   | `fee` | `fee_amount.value` ; absent → 0 ; **quel que soit son signe** (positif sur un remboursement, admis par 38-11a et ventilé par 38-11b AC-5) |
   | `net` | **calculé** `gross + fee` (l'API n'en donne pas) |
   | `from_email` / `to_email` | `payer_info.email_address`, placé en `from_email` pour un crédit, en `to_email` pour un débit ; l'autre côté `None`. **Non confirmé** pour un débit (titulaire ou contrepartie) : vérifié au premier appel réel ; si c'est le titulaire, `to_email` reste `None` pour un débit (jamais l'e-mail du titulaire, qui ferait mentir le filtre `counterparty_email` des règles, 38-11c1) |
   | `transaction_id` | `transaction_info.transaction_id` (contrôle de 38-11a) |
   | `reference_txn_id` | `transaction_info.paypal_reference_id`, **tel quel** quel que soit son type (`TXN`, `SUB`…), en majuscules, comme la colonne du CSV |
   | `balance_impact` | `Credit` si `gross ≥ 0`, sinon `Debit` (la demande n'admet que les lignes qui affectent le solde : pas de `Memo`) |
   | `balance` | `None` (`ending_balance` est vide dès qu'un filtre est posé) ; le `closing_balance` du `bank_imports` reste `NULL` |
   | `item_title` | `cart_info.item_details[].item_name` joints par « · » |
   | `subject`, `note`, `invoice_number` | `transaction_info.transaction_subject`, `.transaction_note`, `.invoice_id` (à défaut `cart_info.paypal_invoice_id`) |
   | `line` | rang de la transaction dans le téléchargement, à partir de 1 (le CSV compte l'en-tête comme ligne 1 : le rang ne sert qu'aux messages) |

   **Table des types**, fermée, en constante `PAYPAL_EVENT_TYPES` (code → libellé du CSV). Les libellés sont **ceux
   que lit la phase 1**, relevés sur le CSV réel de Beat (`kesh-gate-logs/epic-38-kiss/paypal-2024-comprendre.md`) :
   les listes de `classify` (38-11c2 AC-4 : ventes, conversions, retraits) et `PAYPAL_NEUTRAL_PAIR_TYPES` (38-11b AC-5).
   **Test obligatoire** : chaque libellé de ces listes est l'image d'au moins un code de la table — sans quoi une vente
   téléchargée tomberait en `Other` et une paire de retenue ne serait plus neutralisée, en silence (rien de faux n'est
   écrit, Kesh ne fait que proposer, mais le CSV et l'API ne se rapprocheraient plus de la même façon). Codes de départ
   (page de PayPal) : `T0000`, `T0001`, `T0002` (`Subscription Payment`), `T0003`, `T0006` (`Express Checkout Payment`),
   `T0007` (`Website Payment`), `T0011` (`Mobile Payment`), `T0013`, `T0200` (`General Currency Conversion`), `T0201`
   (`User Initiated Currency Conversion`), `T0400` (`General Withdrawal`), `T0403`, `T1105`, `T1107` (`Payment
   Refund`), `T1110`, `T1500` à `T1503`, `T2105` / `T2106` (`Payment Review Hold` / `Payment Review Release`). Le
   libellé attribué à un code **se vérifie au développement**, par appel réel joint au CSV de la même période sur
   l'identifiant de transaction (Notes de développement) ; un libellé non vérifié s'écrit tel en commentaire de la
   constante, jamais comme acquis. La fixture fictive de 38-11a (`paypal-balance-affecting-fictif.csv`) n'est **pas**
   une source de libellés : elle a été écrite pour la phase 1. Un code absent de la table donne une ligne « à
   examiner », jamais une ligne ignorée.
6. **AC-6 — Débit, limitation, reprise, parallélisme.** Un seul fil de fenêtres quand `KESH_PAYPAL_PARALLEL` = 1 ;
   sinon jusqu'à N requêtes en vol (sémaphore partagé, AC-3), les résultats rangés par fenêtre puis par page. Sur
   **429** (ou 503) : pause **commune** à tous les fils (l'instant de reprise se partage), durée = `Retry-After` en
   secondes s'il est présent et entier (borné à 300 s), sinon 5, 10, 20, 40 puis 60 secondes ; **la même requête est
   reprise** (pas la période), six essais au plus par requête, puis échec nommant la fenêtre et la page. Pause et
   attente injectables (`PaypalTiming`) : les tests ne dorment pas. Autres statuts : 401 une fois renouvelé (AC-5) ;
   403 `PAYPAL_PERMISSION_DENIED` (l'application n'a pas la fonction, message qui le dit) ; 5xx et délai dépassé →
   mêmes essais que 429 ; le reste `PAYPAL_API_ERROR` avec le nom d'erreur de PayPal (jamais l'URL, qui porte l'hôte
   configuré : le texte d'une `reqwest::Error` ne sort pas tel quel).
7. **AC-7 — Même ventilation, mêmes doublons.** Toutes les fenêtres sont lues et converties en mémoire, **puis**
   passent par la ventilation et l'import de 38-11b, extraits en fonction partagée `paypal_import::import_rows(…,
   source)` (comportement de la route CSV inchangé, ses tests verts sans changement d'assertion). `source` décide de ce
   qui diffère, et de cela seul : **CSV** — `filename` = nom du fichier, empreinte de 38-11b AC-6, refus
   `BANK_IMPORT_DUPLICATE_FILE`, trace `paypal_import.created` ; **API** — `filename` = « PayPal API
   <from>…<to> », `file_hash` = SHA-256 des identifiants retenus triés ‖ `":"` ‖ devise, pas de refus de fichier en
   double (relancer la même période rend `inserted: 0`), trace `paypal_download.created` à la place de
   `paypal_import.created` (jamais les deux). `source_format` reste `'CSV'` (valeur existante, aucun comportement n'en
   dépend, aucune migration). Le reste est commun : mêmes filtres (statut, paires neutres), mêmes comptes par devise,
   mêmes verrous (`with_account_locks`), **une seule transaction de base** (tout ou rien), traces `bank_import.created`
   par devise. Les doublons se jugent par identifiant de transaction sur le compte bancaire (brut ou `:FEE`) : un CSV
   puis un téléchargement, ou l'inverse, ou deux téléchargements qui se chevauchent, ne créent rien en double. Un
   identifiant présent deux fois dans le même téléchargement (pages décalées par une transaction arrivée pendant la
   lecture) est gardé une fois s'il porte les mêmes devise, brut et frais ; sinon la tâche échoue
   (`PAYPAL_API_ROW_INVALID`, l'identifiant nommé), rien d'écrit. Limite écrite : une page décalée peut aussi faire
   **manquer** une transaction de la fenêtre du jour ; relancer la période plus tard la rattrape, sans doublon. Le
   rapprochement (38-11c1, 38-11c2) lit ensuite ces lignes sans changement.
8. **AC-8 — Tâche et routes.** `POST /api/v1/paypal-downloads` (corps `{ account, from, to }` ; **Comptable et plus**,
   dans `comptable_routes`) et `GET /api/v1/paypal-downloads/{jobId}` (même rôle). **Clés d'API refusées** sur les deux,
   dans le gestionnaire, avant tout : 403 `API_KEY_OPERATION_FORBIDDEN` (`AppError::ApiKeyOperationForbidden`, patron
   de `bank_imports::delete`, `routes/bank_imports.rs:1688`) — la route fait sortir un appel authentifié par un secret
   de l'Administrateur ; une clé `read` est arrêtée avant par le middleware. Le `POST` est inscrit à `LIB_ROUTES`
   (`crates/kesh-api/tests/audit_route_registry.rs`) en `Traced, SansEcritureAuJournal` ; le compte de
   `the_registry_partition_is_what_the_story_declares` est relevé de un avec sa raison, et la phrase « N des M routes à
   verbe mutant » du manuel d'administration (`docs/manual/fr/admin-manual.tex`, lue par
   `le_manuel_dit_le_compte_du_registre`) mise à jour **dans cette story** (son gate la lit) — **`.tex` seul**, comptes
   **recomptés** après ceux de 38-11b et 38-11c2. Refus du `POST` dans l'ordre, rien lancé : 403 (clé d'API) → 400
   `VALIDATION_ERROR` → 404 `PROVIDER_ACCOUNT_NOT_FOUND` (38-11b) → 412 `PAYPAL_API_NOT_CONFIGURED` (aucune paire pour ce
   libellé ; 412 comme `SMTP_NOT_CONFIGURED`, `errors.rs:2652`) → 400 `PAYPAL_PERIOD_INVALID` / `PAYPAL_PERIOD_TOO_OLD`
   → 409 `PAYPAL_DOWNLOAD_RUNNING` (une tâche tourne déjà pour ce compte PayPal de la société ; `details: { jobId }`,
   pour que l'écran reprenne le suivi) ; sinon **202** `{ jobId }` (entier croissant du processus). `GET` : **société
   de l'appelant seulement**, sinon 404 ; réponse `{ state: "running" | "done" | "failed", windows: { done, total },
   pausedUntilSeconds: number | null, lastRefreshed: string | null, result: <réponse 201 de 38-11b> | null, error: {
   code, message, window?, page?, details? } | null }` (`lastRefreshed` : le plus ancien `last_refreshed_datetime` des
   pages, « données de PayPal à jour jusqu'à »). Les refus qui ne se jugent qu'**après** le téléchargement, une fois
   les devises connues, passent en `failed` avec le code et les `details` de 38-11b, rien d'écrit :
   `PAYPAL_API_ROW_INVALID` (erreurs de ligne de l'AC-5, au plus `MAX_CSV_LINE_ERRORS`, forme de `PaypalLineError`),
   `PROVIDER_CURRENCY_ACCOUNT_MISSING`, `PROVIDER_CURRENCY_DUPLICATE`, `BANK_ACCOUNT_CLOSED`,
   `RECONCILIATION_ACCOUNT_LOCKED`. La tâche vit **en mémoire** du processus (table de l'état d'application, à côté du
   sémaphore de l'AC-3 ; `AppState::new_for_tests`, `lib.rs:148`, et les constructions de test reçoivent leur valeur),
   purgée à chaque accès des tâches finies depuis plus d'une heure, **perdue au redémarrage** : relancer est sans
   risque (AC-7). Rien n'est écrit en base avant la fin : une tâche échouée n'a rien changé. Trace d'audit
   `paypal_download.created` (`details` : compte PayPal, période, fenêtres, compte rendu) à la fin réussie, sous
   l'acteur de la demande ; aucun secret, aucun jeton.
9. **AC-9 — Pas de fuite.** Aucun journal (`tracing`), message d'erreur, réponse ni trace n'imprime `client_id`, secret
   ni jeton ; test qui rejoue un échec d'authentification et un échec de page et cherche ces trois chaînes dans la
   réponse et dans les journaux capturés. `KESH_PAYPAL_BASE_URL` n'est jamais renvoyée par l'API.
10. **AC-10 — Serveur PayPal simulé.** `crates/kesh-api/tests/paypal_download_e2e.rs` lance **dans le processus** un
    petit serveur `axum` sur `127.0.0.1:0` (dépendance existante, patron des tests de `exchange_rates`) qui joue : le
    jeton, des pages à `total_pages` réglable, un 429 (avec et sans `Retry-After`), un 400 `RESULTSET_TOO_LARGE`, un
    401, un 403, une réponse qui ne vient pas (délai), une transaction de chaque statut, des frais positifs, une paire
    neutre, un doublon d'identifiant (identique, puis différent). Données **fictives** (identifiants `0TEST…`, e-mails
    `@example.invalid`). Cas : période de quatre mois → fenêtres attendues et ventilation de 38-11b par devise ; CSV
    (fixture de 38-11a) puis téléchargement des mêmes transactions → rien de doublé ; 429 puis reprise ; parallélisme
    3 → mêmes lignes qu'à 1 ; échec à la fenêtre 3 → rien d'écrit ; devise sans compte → `failed`
    `PROVIDER_CURRENCY_ACCOUNT_MISSING`, rien d'écrit ; deux tâches du même compte → 409 avec `jobId` ; clé d'API → 403
    ; secrets absents des journaux (AC-9). Tests unitaires : `windows` (30 jours, bascule d'heure d'été et d'hiver —
    aucune fenêtre au-delà de 31 × 24 h —, fin d'année, fin ramenée à `now`), `parse_transactions_page` et
    `to_paypal_row` (chaque ligne du tableau de l'AC-5), la couverture des libellés de la phase 1 par la table (AC-5),
    configuration (AC-1 à AC-3, patron des tests de `KESH_SMTP_*`).

## Tâches

- [ ] T1 — `Config` : variables, validation, masque `Debug` (AC-1 à AC-3).
- [ ] T2 — `kesh-import::paypal_api` : lecture de page, table des types, conversion, contrôles partagés avec `read_row` (AC-5).
- [ ] T3 — `windows` et client : jeton, pages, délais, limitation, parallélisme (AC-4, AC-5, AC-6).
- [ ] T4 — Fonction d'import partagée avec 38-11b, paramétrée par la source (AC-7).
- [ ] T5 — Tâche en mémoire, routes, registre et phrase du manuel, audit, erreurs et messages aux quatre locales (AC-8, AC-9).
- [ ] T6 — Serveur simulé et tests (AC-10).

## Stratégie de tests

Gate ciblé : `binary(paypal_download_e2e)`, les tests de `kesh-import` et ceux de `config`. **Gate complet** au dernier
commit (la fonction d'import de 38-11b est touchée : ses tests, ceux des imports CAMT/CSV, `audit_route_registry` et
`date_du_jour_garde` restent verts sans changement d'assertion autre que les comptes relevés de l'AC-8). Aucune
migration : P5 à P8 sans objet.

## Modules touchés (3)

`kesh-import` (lecture et conversion pures), `kesh-api` (configuration, client, tâche, routes, erreurs ; le `.tex` du
manuel d'administration pour la seule phrase du registre), `kesh-i18n`.

## Fichiers communs

`crates/kesh-api/src/config.rs`, `lib.rs` (routes, état d'application), `errors.rs`, catalogues i18n (leur bloc),
`crates/kesh-api/src/routes/paypal_imports.rs` de 38-11b (fonction partagée), `crates/kesh-import/src/paypal.rs` de
38-11a (contrôles extraits), `audit_route_registry.rs` et `admin-manual.tex` (comptes recomptés) : à relire à la reprise
de `main`.

## Note de version

*Ajouté* — Kesh peut télécharger les transactions d'un compte PayPal par l'API de PayPal, par périodes, au lieu du CSV ;
les identifiants se posent dans l'environnement du serveur (`KESH_PAYPAL_<n>_…`), une application PayPal en lecture seule
par compte ; les doublons se reconnaissent à l'identifiant de transaction, le CSV et le téléchargement se mélangent.
*Limite* — une transaction peut mettre jusqu'à trois heures à paraître dans l'API ; relancer la période plus tard la
rattrape, sans doublon.

## Notes de développement

- **Aucun appel au vrai PayPal en test ni en CI.** Au développement, des **appels réels en lecture seule** sont permis
  (autorisation de Beat du 2026-11-20) : chaque requête passe d'abord par `kesh-gate-logs/appel-reel.sh paypal
  "<motif>"` (une par minute, quarante au plus pour PayPal ; code 3 = quota épuisé, s'arrêter), avec
  `data-not-committed/paypal-api.env`, jamais affiché, commité ni envoyé ; aucune réponse réelle n'entre dans le dépôt.
  Budget conseillé : un jeton, puis une ou deux pages d'une période dont Beat a le CSV, pour vérifier les points « non
  confirmés » (hôte réel, signe des frais, `payer_info` d'un débit, libellés des codes par jointure sur l'identifiant
  de transaction) ; le résultat s'écrit au Dev Agent Record **sans donnée personnelle** (codes, libellés, signes).
  L'essai complet (tout 2024, d'abord par CSV, puis en ligne) se joue sur l'instance de répétition (8889), jamais sur
  celle des livres (8888).
- Si PayPal contredit une des hypothèses « non confirmées » (signe des frais, libellé `V`, `payer_info`, libellés de
  type), la correction va à la table ou à la conversion, jamais aux règles de 38-11c1 et 38-11c2.

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
- 2026-11-20 — validation P4 (Claude Sonnet 5.5, contexte frais) : renvois « 38-11c » nus désignés 38-11c1 ou 38-11c2 après le découpage ; aucun autre changement.
- 2026-11-20 — validation P1 de la phase 2 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 1 HIGH, 7 MEDIUM, 8 LOW
  (pour cette fiche ; 2 H, 10 M, 8 L avec 38-12b), remédiés (rapport `kesh-gate-logs/epic-38-kiss/validation/38-12-P1.md`).
  Principaux : `FEE_POSITIVE` inexistant, frais de tout signe comme 38-11a et 38-11b ; libellés des types tenus par
  la phase 1 (listes de `classify` et des paires neutres, test de couverture), non par la fixture fictive ; refus
  postérieurs au téléchargement nommés (`failed`) et ligne invalide en tout ou rien (`PAYPAL_API_ROW_INVALID`) ; client
  à délais, sans redirection, corps borné ; fenêtres de 30 jours de Zurich (une fenêtre de 31 jours dépasse la limite
  au passage à l'heure d'hiver), bornes par `debut_du_jour_suisse` / `fin_du_jour_suisse`, fin ramenée à l'instant ;
  registre et phrase du manuel relevés dans cette story ; clés d'API refusées en `API_KEY_OPERATION_FORBIDDEN` ;
  `payer_info` d'un débit non confirmé ; OAuth, 429 et fuseau confirmés à la doc ; 412 pour la paire absente ;
  `KESH_PAYPAL_BASE_URL` fermée ; jeton par tâche, lecture JSON dans `kesh-import`, trace unique par source ; appels
  réels de développement selon les autorisations de Beat.
