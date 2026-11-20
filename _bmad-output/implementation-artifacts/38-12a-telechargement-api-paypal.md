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
| `FR-38-12a-2` | Les transactions d'une période se téléchargent par fenêtres (30 jours de Zurich au plus, sous la limite de 31 jours de PayPal), pages de 500, avec reprise après une limitation de débit. | AC-4, AC-6 |
| `FR-38-12a-3` | Ce qui est téléchargé prend la forme du CSV de 38-11a, puis passe par la ventilation et les doublons de 38-11b. | AC-5, AC-7 |
| `FR-38-12a-4` | Le téléchargement tourne en tâche de fond ; sa progression et son résultat se lisent par une route. | AC-8 |
| `FR-38-12a-5` | Le parallélisme est réglé par `KESH_PAYPAL_PARALLEL` (1 par défaut, 6 au plus). | AC-3, AC-6 |
| `NFR-38-12a-1` | Aucun secret dans un journal, une réponse, une erreur ni une trace d'audit. | AC-2, AC-9 |
| `NFR-38-12a-2` | Aucune donnée réelle ni appel au vrai PayPal dans les tests ni en CI. | AC-10 |

## Critères d'acceptation

*Socle commun S1 à S9* (texte identique de `epic-39-wise.md` et `epic-40-stripe.md`, § « Socle commun de la phase 2 »,
C-40-K6) : cette fiche, développée la première, **écrit le socle générique** (`provider_env`, `provider_download`,
constructeur de client, routes et codes `PROVIDER_*`, cœur d'import `ProviderLine`, test S9) ; Wise (39-12a1, 39-12a2) et
Stripe (40-8a, 40-8b) l'appellent. Restent propres à PayPal : le client (jeton, pages, `RESULTSET_TOO_LARGE`), la conversion
vers `PaypalRow`, la table des types et le passage `PaypalRow` → `ProviderLine` de 38-11b.

1. **AC-1 — Configuration, lecteur unique `provider_env`** (socle S1 ; module neuf de `kesh-api`, appelé par
   `crates/kesh-api/src/config.rs`, lecture par `env_nonempty`, `:1406`). Générique par prestataire `<P>` : `KESH_<P>_<n>_ACCOUNT`
   (le **libellé** = `provider_account` de 38-11b), les champs propres au prestataire, `KESH_<P>_BASE_URL`,
   `KESH_<P>_PARALLEL` ; `n` de 1 à 9 ; un contrôle propre au prestataire s'y branche. Pour PayPal :
   `KESH_PAYPAL_<n>_ACCOUNT`, `KESH_PAYPAL_<n>_CLIENT_ID`, `KESH_PAYPAL_<n>_SECRET` ; `KESH_PAYPAL_BASE_URL` (défaut
   `https://api-m.paypal.com` ; `https://api-m.sandbox.paypal.com` pour un essai ; `https` exigé, `http` admis vers le
   seul bouclage, `127.0.0.1` ou `localhost`, pour le serveur simulé, S5) ; `KESH_PAYPAL_PARALLEL` (AC-3). Une valeur **vide ou blanche vaut absente**, comme partout dans `config.rs` (`env_nonempty`) : les fichiers
   `docker-compose.yml` et `docker-compose.prod.yml` transmettent chaque variable au conteneur en `${NOM:-}`, donc vide quand
   `.env` ne la pose pas — **ces deux fichiers et `.env.example` portent les séries 1 à 3**, `KESH_PAYPAL_BASE_URL` et
   `KESH_PAYPAL_PARALLEL` (au-delà de trois comptes, on ajoute les lignes : le manuel le dit, 38-12b AC-5) ; sans cela, des
   identifiants posés au `.env` n'atteindraient jamais le serveur. ⚠️ **La garde `crates/kesh-api/tests/configuration_transmise.rs`
   (#550) s'étend dans cette story** : elle calcule l'ensemble des variables lues depuis les **littéraux** passés aux jetons
   surveillés (`JETONS_SURVEILLES`, sites de la liste fermée `EMPLACEMENTS_AUTORISES`), exige que chaque variable lue soit
   transmise par les deux compose (familles (T), sauf `EXCEPTIONS`) et ait sa ligne dans `.env.example` (E), que toute clé
   transmise soit lue (T), et que tout jeton `KESH_…` du gabarit, des compose, du manuel français et des littéraux de la
   production nomme une variable connue (F). Un nom construit (`format!("KESH_PAYPAL_{n}_SECRET")`) et le parcours des
   variables du préfixe (numéro hors `1..=9`) lui sont invisibles ou interdits : sans extension, les lignes
   `KESH_PAYPAL_1_…` des compose rougissent en (T) (« transmise, non lue ») et les séries 4 à 9 en (E). L'extension —
   une **famille indexée** déclarée une fois par prestataire (préfixe, champs, `n` de 1 à 9 ; séries 1 à 3 exigées aux deux
   compose et à `.env.example`, séries 4 à 9 admises absentes par une raison nommée, comme `EXCEPTIONS` ; les nouveaux
   sites de lecture de `provider_env` inscrits à `EMPLACEMENTS_AUTORISES` en forme exacte ; la forme `KESH\_PAYPAL\_<n>\_…`
   du manuel admise en (F)) — est **générique** (Wise et Stripe y ajoutent leur famille), testée par l'auto-test (S) de
   la garde, et aucune liste fermée existante n'est relâchée pour les autres variables. Tout est lu
   **une fois**, au démarrage ; un compte PayPal de Kesh dont le libellé n'a pas de série reste sans téléchargement
   (le CSV continue). Le libellé est **propre à l'installation**, non à la société (une installation porte une société,
   manuel d'administration `:1079`) : limite écrite au manuel par 38-12b.
2. **AC-2 — Validation au démarrage et secrets** (S1). Contrairement à `KESH_SMTP_*` — qui ne refuse de démarrer que si
   la récupération de mot de passe est activée (`:1120`) et retombe en silence sur le port par défaut —, la validation
   est **inconditionnelle** : série **partielle** (au moins une variable non vide, pas toutes ; une série entièrement vide est
   absente, jamais refusée : c'est l'état de toute installation qui ne l'utilise pas), libellé en double (sans casse), numéro hors `1..=9` (p. ex.
   `KESH_PAYPAL_10_…`, repéré en parcourant les variables du préfixe), URL de base hors de la règle de S5 → **refus de
   démarrer** (`ConfigError` neuve), message qui nomme la variable, jamais sa valeur. Une valeur de la forme `<…>`
   (`is_template_placeholder`, `:1538`) est refusée, comme le mot de passe administrateur (`:650`) et le secret JWT
   (`:683`). Champs **privés**, lus par accesseur unique ; `Debug` les masque (test sur le patron de
   `smtp_password_masked_in_debug`, `:3484`). Aucune colonne, aucune table, rien dans `.keshbackup`.
3. **AC-3 — Parallélisme** (S1, S2). `KESH_<P>_PARALLEL` entier strict : absent = **1** ; hors `1..=6` ou illisible →
   refus de démarrer (pas de repli silencieux). Un sémaphore par prestataire, partagé par tous ses téléchargements,
   borne les requêtes en vol.
4. **AC-4 — Fenêtres** (S2). Fonction pure générique `windows(from, to, max_days, now) -> Vec<(start, end)>` de
   `provider_download`, sur les jours de **Zurich** : bornes par `kesh_core::horloge::debut_du_jour_suisse` et
   `fin_du_jour_suisse` (Story 36-10 ; aucun décalage recalculé), fenêtres contiguës de `max_days` jours de Zurich au
   plus, fin d'une fenêtre = seconde précédant le début de la suivante, fin de la dernière **ramenée à `now`** si elle
   est plus tard. PayPal l'appelle avec **`max_days = 30`** : une fenêtre de 31 jours de Zurich qui contient le passage
   à l'heure d'hiver dure 31 × 24 h + 1 h, au-delà de la limite de 31 jours de PayPal (unité non dite) ; 30 jours en
   font au plus 30 × 24 h + 1 h. Bornes envoyées en UTC (`…Z`), **tronquées à la seconde** (`fin_du_jour_suisse` rend la dernière milliseconde du jour : sa
   borne envoyée est `23:59:59` de Zurich, les horodatages de PayPal étant à la seconde). Règle de `max_days` du socle : la limite
   du prestataire moins un jour quand elle ne se dit pas en jours entiers de Zurich (PayPal : 31 jours, unité non dite → 30). `now` et le jour du jour viennent de
   `state.horloge` (Epic 35 ; garde `date_du_jour_garde.rs`) ; le délai d'un jeton se mesure en `Instant`. Refus 400
   `PROVIDER_PERIOD_INVALID` (S4), `details: { provider: "paypal", reason }` avec `reason` ∈ {`from_after_to`,
   `future` (`from` après le jour de Zurich du jour ; une `to` future n'est **pas** refusée, elle est ramenée à `now`),
   `too_old`} — `too_old` si `from` est avant le même jour trois ans plus tôt (limite de l'API PayPal).
   Propre à PayPal : pages de **500**, `page` de 1 à `total_pages`, `fields=all`, `balance_affecting_records_only=Y` ;
   sur 400 `RESULTSET_TOO_LARGE`, la fenêtre se coupe en deux et chaque moitié est reprise (récursif, plancher d'un
   jour : au-delà, échec `PROVIDER_API_ERROR` nommant la fenêtre ; `steps.total` grandit alors des moitiés).
5. **AC-5 — Client et conversion.** Constructeur de client HTTP **commun** (S5), **extrait** de
   `exchange_rates/source.rs:116-130` (rustls, aucune redirection suivie — le jeton ne part jamais ailleurs —, délais
   de connexion et total, HTTPS seul hors bouclage, `User-Agent` sans donnée personnelle), que la source des cours
   appelle ensuite ; corps de réponse **borné** — par une constante **propre à PayPal** (16 Mio : `MAX_BODY_BYTES` des cours est de 1 Mio, trop
   petit pour 500 transactions en `fields=all`), la boucle de lecture par morceaux de `source.rs:174-190` **extraite avec le
   constructeur** (une fonction `lire_corps_borne(reponse, max)` commune, non recopiée), délai total de 60 s par requête
   (constante PayPal, celui des cours — 20 s — est trop court pour une page de 500), délais réglables pour les tests
   (patron `with_timeouts`, `source.rs:155`) — sans délai, une requête pendue laisserait la
   tâche « en cours » et le compte refusé en 409 jusqu'au redémarrage. `reqwest` est **déjà en dépendance**
   (`crates/kesh-api/Cargo.toml:112`, sans la fonctionnalité `json`) : `.bytes()` puis `serde_json` ; aucune
   dépendance neuve. Client PayPal (`crates/kesh-api/src/paypal_api/`) : jeton **un par tâche**, demandé au début,
   renouvelé à l'approche de `expires_in` et une fois sur 401, puis échec `PROVIDER_AUTH_FAILED` ; aucun cache partagé.
   **Lecture et conversion pures dans `kesh-import`** (module neuf `crates/kesh-import/src/paypal_api.rs`, sans
   dépendance interne ni réseau ; `serde` et `serde_json` y sont déjà) : `parse_transactions_page(&[u8]) ->
   Result<PaypalApiPage, PaypalApiError>` (transactions, `total_pages`, `last_refreshed_datetime`) et
   `to_paypal_row(&PaypalApiTx, line) -> Result<PaypalRow, PaypalLineError>`. Les **contrôles de 38-11a** (identifiant
   `[0-9A-Z]` après mise en majuscules — un `_` admis par le schéma est refusé comme au CSV —, devise de trois lettres,
   montant par `parse_dot_decimal_amount`, signe du brut selon le sens, `net = gross + fee`) sont **ceux de
   `read_row`** (`paypal.rs`), extraits en fonctions partagées, non recopiés ; les erreurs ont la forme de
   `PaypalLineError` (champ = nom du champ de l'API). Correspondance champ à champ avec `PaypalRow` :

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
   (`User Initiated Currency Conversion`), `T0400` (`General Withdrawal`), `T0403`, `T1105` (page : « reversal of general account hold »), `T1107` (`Payment
   Refund`), `T1110` (page : « hold on balance for dispute investigation »), `T1500` à `T1503` (`T1501` : « account hold
   for open authorization »), `T2105` / `T2106` (`Payment Review Hold` / `Payment Review Release`). La page ne donne aucun
   code évident pour `Reversal of Account Hold` (liste des paires neutres) : le test de couverture l'impose, le code se
   trouve par l'appel réel. Le
   libellé attribué à un code **se vérifie au développement**, par appel réel joint au CSV de la même période sur
   l'identifiant de transaction (Notes de développement) ; un libellé non vérifié s'écrit tel en commentaire de la
   constante, jamais comme acquis. La fixture fictive de 38-11a (`paypal-balance-affecting-fictif.csv`) n'est **pas**
   une source de libellés : elle a été écrite pour la phase 1. Un code absent de la table donne une ligne « à
   examiner », jamais une ligne ignorée.
6. **AC-6 — Reprise et parallélisme** (S2, politique **unique** des trois prestataires, dans `provider_download`). Un
   seul fil quand `KESH_PAYPAL_PARALLEL` = 1 ; sinon jusqu'à N requêtes en vol (sémaphore de l'AC-3), les résultats
   rangés par fenêtre puis par page. Sur **429** et **5xx** (délai dépassé compris) : pause **commune** à tous les fils
   du prestataire (l'instant de reprise se partage), sans réduire le parallélisme ; durée = `Retry-After` en secondes
   s'il est présent et entier, **borné à 120 s**, sinon 5, 10, 20, 40 puis 60 secondes ; **la même requête est
   reprise** (pas la période), **six essais** au plus par requête, puis échec `PROVIDER_RATE_LIMITED` (429) ou
   `PROVIDER_UNAVAILABLE` (5xx, délai) nommant la fenêtre et la page ; **aucune pause fixe** entre requêtes. Horloge et
   attente injectables : les tests ne dorment pas. Autres statuts : 401 une fois renouvelé (AC-5) ; 403
   `PROVIDER_PERMISSION_DENIED` (l'application n'a pas la fonction, message qui le dit) ; corps illisible ou trop grand
   `PROVIDER_RESPONSE_INVALID` ; le reste `PROVIDER_API_ERROR`. `details.provider` nomme le prestataire ; **jamais le
   texte du prestataire** ni l'URL (le texte d'une `reqwest::Error`, qui porte l'hôte, ne sort pas tel quel) : le nom
   d'erreur de PayPal (`NOT_AUTHORIZED`…) va au journal du serveur, sans secret.
7. **AC-7 — Même ventilation, mêmes doublons** (S6). Toutes les fenêtres sont lues et converties **avant** toute
   écriture ; les `PaypalRow` passent par le **traitement PayPal de 38-11b** (statut ≠ `Completed`, `Memo`, paires
   neutres, séparation brut / `:FEE`) puis par le **cœur commun d'import** qui reçoit des lignes neutres
   (`ProviderLine` : devise, dates, montant, identifiants, référence, contrepartie, e-mail, type, détails ; recherche
   des comptes par devise, refus indépendants du format, `with_account_locks`, empreinte, doublons, compte rendu, **une**
   transaction de base, tout ou rien). **Ce cœur, la fiche 38-11b ne le factorise pas** (ses AC-5 et AC-6 décrivent la ventilation et l'import sur `PaypalRow`) ;
   39-11b (NFR-39-11b-2) ou 40-7b peut l'avoir fait avant : à la reprise (T0), si un cœur existe, 38-12a l'**appelle** et ne
   crée aucun second chemin ; sinon, **c'est 38-12a qui l'extrait** de la route PayPal en deux pièces : une fonction **pure**
   `PaypalRow` → `ProviderLine` (écarts, paires neutres, séparation brut / `:FEE` de 38-11b AC-5, testée sans base) et le
   cœur d'écriture (`crates/kesh-api/src/routes/provider_imports.rs`), que la route CSV appelle à son tour, ses tests
   **inchangés en assertions**. ⚠️ **Le déplacement change le couple (fichier, fonction)** que
   `bank_account_closed_inventory.rs` (`REGISTRE`) classe pour `create_with_transactions` et ses appelants : l'entrée
   suit, avec sa raison, faute de quoi le test rougit. État trouvé et décision au Change Log (T0). Ce qui diffère d'un import de fichier, et cela seul :
   `filename` = « PayPal API <from>…<to> », `file_hash` = SHA-256 des identifiants retenus triés ‖ `":"` ‖ devise, **pas
   de refus `BANK_IMPORT_DUPLICATE_FILE`** (relancer la même période rend « 0 insérée »), trace `paypal_download.created`
   à la place de `paypal_import.created` (jamais les deux), écrite **dans la même transaction** que l'import ; `source_format` reste `'CSV'` (valeur existante, aucun
   comportement n'en dépend, aucune migration) ; traces `bank_import.created` par devise comme à l'import. Les doublons
   se jugent par identifiant de transaction sur le compte bancaire (brut ou `:FEE`) : un CSV puis un téléchargement, ou
   l'inverse, ou deux téléchargements qui se chevauchent, ne créent rien en double. Un identifiant présent deux fois
   dans le même téléchargement (pages décalées par une transaction arrivée pendant la lecture) est gardé une fois s'il
   porte les mêmes devise, brut et frais ; sinon échec `PROVIDER_RESPONSE_INVALID` (l'identifiant nommé), rien
   d'écrit. Limite écrite : une page décalée peut aussi faire **manquer** une transaction de la fenêtre du jour ;
   relancer la période plus tard la rattrape, sans doublon. Le rapprochement (38-11c1, 38-11c2) lit ces lignes sans
   changement.
8. **AC-8 — Tâche et routes** (S2, S3, S4). `POST /api/v1/paypal-downloads` (corps `{ account, from, to }` ; **Comptable
   et plus**, dans `comptable_routes`) et `GET /api/v1/paypal-downloads/{jobId}` (même rôle). **Sessions seules** : une
   clé d'API est refusée sur les deux, dans le gestionnaire, avant tout, en 403 `API_KEY_OPERATION_FORBIDDEN`
   (`AppError::ApiKeyOperationForbidden`, patron de `bank_imports::delete`, `routes/bank_imports.rs:1688`) — un secret de
   l'Administrateur sort ; une clé `read` est arrêtée avant par le middleware. Le `POST` est inscrit à `LIB_ROUTES`
   (`crates/kesh-api/tests/audit_route_registry.rs`) en `Traced, SansEcritureAuJournal` ; le compte de
   `the_registry_partition_is_what_the_story_declares` est relevé de un avec sa raison, et la phrase « N des M routes à
   verbe mutant » du manuel d'administration (`docs/manual/fr/admin-manual.tex`, lue par
   `le_manuel_dit_le_compte_du_registre`) mise à jour **dans cette story** (son gate la lit) — **`.tex` seul**, comptes
   **recomptés** après ceux de 38-11b et 38-11c2. Refus du `POST` dans l'ordre, rien lancé : 403 (clé d'API) → 400
   `VALIDATION_ERROR` → 404 `PROVIDER_ACCOUNT_NOT_FOUND` (38-11b) → 422 `PROVIDER_API_NOT_CONFIGURED` (aucune série pour
   ce libellé) → 400 `PROVIDER_PERIOD_INVALID` (AC-4) → 409 `PROVIDER_DOWNLOAD_RUNNING` (une tâche tourne déjà pour ce
   prestataire, cette société et ce libellé ; `details: { provider, jobId }`, pour que l'écran reprenne le suivi) ;
   sinon **202** `{ jobId }` (entier croissant du processus). `GET` : **société de l'appelant seulement**, sinon 404 ;
   réponse, forme commune de S3 : `{ state: "running" | "done" | "failed", steps: { done, total: number | null },
   pausedUntilSeconds: number | null, result, error: { code, message, details? } | null }` — pour PayPal une étape est
   une fenêtre (`total` connu d'avance), `result` = la réponse 201 de 38-11b plus `lastRefreshed` (le plus ancien
   `last_refreshed_datetime` des pages, « données de PayPal à jour jusqu'à »), `details` = `{ provider, window?, page?,
   lines? }`. Les refus qui ne se jugent qu'**après** la lecture, une fois les devises connues, passent en `failed`
   avec leur code et le contenu des `details` de 38-11b, rien d'écrit : `PROVIDER_RESPONSE_INVALID` (erreurs de ligne
   de l'AC-5, au plus `MAX_CSV_LINE_ERRORS`, forme de `PaypalLineError`, dans `details.lines`),
   `PROVIDER_CURRENCY_ACCOUNT_MISSING`, `PROVIDER_CURRENCY_DUPLICATE`, `BANK_ACCOUNT_CLOSED`,
   `RECONCILIATION_ACCOUNT_LOCKED`. La tâche est lancée par `tokio::spawn` ; **une panique ne laisse jamais une tâche « en cours »** (le compte resterait refusé
   en 409 jusqu'au redémarrage) : le `JoinHandle` est surveillé et la tâche passe en `failed` `PROVIDER_API_ERROR`, un test
   la fait paniquer. Le 409 ne vaut que **tant que la tâche tourne** : une tâche `done` ou `failed` ne bloque pas la
   suivante. L'acteur (la session de la demande, jamais une clé) est capturé au `POST` et porté par la tâche. Les tâches vivent **en mémoire** du processus (table de `provider_download` dans
   l'état d'application, une tâche par prestataire, société et libellé ; `AppState::new_for_tests`, `lib.rs:148`, et les
   constructions de test reçoivent leur valeur), purgées une heure après leur fin (à chaque accès), **perdues au
   redémarrage** : relancer est sans risque (AC-7). Trace d'audit `paypal_download.created` (`details` : compte PayPal,
   période, fenêtres, compte rendu) à la fin réussie, sous l'acteur de la demande ; aucun secret, aucun jeton. Codes
   `PROVIDER_*` neufs dans `errors.rs` et messages aux quatre locales, **sans nom de prestataire** (Wise et Stripe les
   reprennent).
9. **AC-9 — Pas de fuite** (S9). Aucun journal (`tracing`), message d'erreur, réponse ni trace n'imprime `client_id`,
   secret ni jeton ; **test au marqueur commun** (réutilisé par Wise et Stripe) qui rejoue un échec d'authentification
   et un échec de page et cherche ces chaînes dans la réponse et dans les journaux capturés. `KESH_PAYPAL_BASE_URL`
   n'est jamais renvoyée par l'API.
10. **AC-10 — Serveur PayPal simulé** (S5 : un seul schéma de test, pas de trait de source simulée).
    `crates/kesh-api/tests/paypal_download_e2e.rs` lance **dans le processus** un petit serveur `axum` sur
    `127.0.0.1:0` (patron de `exchange_rates_source_http.rs`) qui joue : le jeton, des pages à `total_pages` réglable,
    un 429 (avec et sans `Retry-After`, et un `Retry-After` de 3600 borné à 120), un 400 `RESULTSET_TOO_LARGE`, un 401,
    un 403, une réponse qui ne vient pas (délai), une transaction de chaque statut, des frais positifs, une paire
    neutre, un doublon d'identifiant (identique, puis différent). Données **fictives** (identifiants `0TEST…`, e-mails
    `@example.invalid`). Cas : période de quatre mois → fenêtres attendues et ventilation de 38-11b par devise ; CSV
    (fixture de 38-11a) puis téléchargement des mêmes transactions → rien de doublé ; 429 puis reprise ; parallélisme
    3 → mêmes lignes qu'à 1 ; échec à la fenêtre 3 → rien d'écrit ; devise sans compte → `failed`
    `PROVIDER_CURRENCY_ACCOUNT_MISSING`, rien d'écrit ; deux tâches du même compte → 409 avec `jobId` ; clé d'API → 403
    ; secrets absents des journaux (AC-9). Tests unitaires : `windows` (`max_days` 30 et 31, bascule d'heure d'été et
    d'hiver — pour 30, aucune fenêtre au-delà de 31 × 24 h —, fin d'année, fin ramenée à `now`), politique de reprise
    (suite des pauses, borne de 120 s, six essais, pause commune), `parse_transactions_page` et `to_paypal_row` (chaque
    ligne du tableau de l'AC-5), couverture des libellés de la phase 1 par la table (AC-5), `provider_env` (AC-1 à
    AC-3, patron des tests de `KESH_SMTP_*`).

## Tâches

- [ ] T0 — Reprendre `main` et les branches des Epics 39, 40 si fusionnées : socle déjà écrit ? cœur `ProviderLine` déjà factorisé ? État au Change Log.
- [ ] T1 — `provider_env` générique, série PayPal, validation, masque `Debug` (AC-1 à AC-3) ; séries 1 à 3 aux deux fichiers `docker-compose` et à `.env.example` ; famille indexée dans `configuration_transmise.rs` (AC-1).
- [ ] T2 — `kesh-import::paypal_api` : lecture de page, table des types, conversion, contrôles partagés avec `read_row` (AC-5).
- [ ] T3 — `provider_download` (`windows`, reprise, tâches, sémaphore), constructeur de client extrait, client PayPal (AC-4 à AC-6).
- [ ] T4 — Appel du cœur commun `ProviderLine` (ou son extraction hors de la route PayPal) (AC-7).
- [ ] T5 — Routes, registre et phrase du manuel, audit, codes `PROVIDER_*` et messages aux quatre locales, test S9 (AC-8, AC-9).
- [ ] T6 — Serveur simulé et tests (AC-10).

## Stratégie de tests

Gate ciblé : `binary(paypal_download_e2e)`, les tests de `kesh-import` et ceux de `config`. **Gate complet** au dernier
commit (la fonction d'import de 38-11b est touchée : ses tests, ceux des imports CAMT/CSV, `audit_route_registry` et
`date_du_jour_garde` restent verts sans changement d'assertion autre que les comptes relevés de l'AC-8). Aucune
migration : P5 à P8 sans objet.

## Modules touchés (3)

`kesh-import` (lecture et conversion pures), `kesh-api` (socle `provider_env` et `provider_download`, constructeur de
client, client PayPal, routes, erreurs ; le `.tex` du manuel d'administration pour la seule phrase du registre),
`kesh-i18n`.

## Fichiers communs

`crates/kesh-api/src/config.rs`, `lib.rs` (routes, état d'application), `errors.rs`, `docker-compose.yml`,
`docker-compose.prod.yml`, `.env.example` et la garde `crates/kesh-api/tests/configuration_transmise.rs` (familles des trois
prestataires), catalogues i18n (leur bloc),
`crates/kesh-api/src/routes/paypal_imports.rs` de 38-11b et le cœur commun `ProviderLine` (38-11b, 39-11b, 40-7b),
`crates/kesh-api/src/exchange_rates/source.rs` (constructeur de client extrait), modules du socle (appelés par 39-12a1,
39-12a2, 40-8a, 40-8b), `crates/kesh-import/src/paypal.rs` de
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
- 2026-11-20 — validation P1 de la phase 2, complément (Claude Opus 5.5, même passe, sur message du coordinateur) :
  alignement sur le **socle commun S1 à S9** des fichiers `epic-39-wise.md` et `epic-40-stripe.md` (texte identique,
  relevé par `diff` ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-P1.md`, § « Ce qui doit changer côté
  38-12 »). Lecteur unique `provider_env` ; `provider_download` (`windows(from, to, max_days)`, PayPal à 30 jours —
  constat DST de la P1 —, reprise unique : `Retry-After` borné à 120 s au lieu de 300, six essais, pause commune) ;
  routes sessions seules, état `steps: { done, total | null }`, `lastRefreshed` déplacé dans `result` ; codes
  `PROVIDER_*` (`PROVIDER_API_NOT_CONFIGURED` 422 comme au socle — le 412 de la P1 est retiré —,
  `PROVIDER_PERIOD_INVALID` avec `details.reason`, `PROVIDER_DOWNLOAD_RUNNING`, `PROVIDER_AUTH_FAILED`,
  `PROVIDER_PERMISSION_DENIED`, `PROVIDER_RATE_LIMITED`, `PROVIDER_UNAVAILABLE`, `PROVIDER_RESPONSE_INVALID` —
  `PAYPAL_API_ROW_INVALID` de la P1 retiré —, `PROVIDER_API_ERROR`) ; cœur d'import commun `ProviderLine` appelé, ou
  extrait hors de la route PayPal, jamais un second chemin ; constructeur de client extrait de `source.rs:116-130` ;
  `KESH_PAYPAL_BASE_URL` selon S5 (`https`, `http` vers le bouclage) — l'ensemble fermé de la P1 est retiré ; test S9
  au marqueur commun ; T0 de reprise.
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 2 HIGH, 5 MEDIUM, 3 LOW pour cette fiche
  (3 H, 8 M, 5 L avec 38-12b), remédiés (rapport `kesh-gate-logs/epic-38-kiss/validation/38-12-P2.md`). Principaux :
  une valeur vide vaut absente (les fichiers `docker-compose` transmettent `${NOM:-}`) et **les séries doivent être
  transmises par ces fichiers**, sans quoi aucun identifiant n'atteint le conteneur ; `detail` → `details` (décision 3) ;
  cœur `ProviderLine` que 38-11b ne factorise pas (extraction ici, `REGISTRE` de `bank_account_closed_inventory`) ;
  plafond de corps propre à PayPal (1 Mio des cours trop petit), délai de 60 s, lecture bornée extraite ; panique de la
  tâche ; bornes tronquées à la seconde ; `max_days` expliqué ; trace dans la transaction d'import.
- 2026-11-20 — validation P3 ciblée sur le commit `dc83cdf6` (Claude Opus 5.5, contexte frais, une lentille) : 0 CRITICAL,
  1 HIGH, 1 MEDIUM, 3 LOW pour les trois fichiers, remédiés (rapport `kesh-gate-logs/epic-38-kiss/validation/38-12-P3-ciblee.md`).
  HIGH : la transmission par les compose (P2) heurtait la garde `configuration_transmise.rs` (#550), qui ne connaît que des
  noms littéraux et exige « lue ⇔ transmise » et une ligne `.env.example` par variable lue — famille indexée à ajouter
  (AC-1, T1, fichiers communs). Avec le message du coordinateur (P2 Wise/Stripe) : `FR-38-12a-2` à 30 jours (31 ne reste
  qu'un cas de test de `windows`) ; aucune occurrence vivante de `detail` au singulier.
