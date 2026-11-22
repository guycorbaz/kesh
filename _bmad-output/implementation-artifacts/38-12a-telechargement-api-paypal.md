# Story 38-12a : téléchargement des transactions par l'API PayPal (identifiants, client, conversion, tâche)

Status: done

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal` (phase 1 au commit `90428594`). Epic 38, **phase 2**
(décisions du responsable de l'epic du 2026-11-20, `kesh-gate-logs/epic-38-kiss/phase2-decisions.md`, autorisations du soir comprises).
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
| nom exact de la fonction à cocher à la création de l'application, conditions d'accès du compte (type, activation) | **non confirmés** ; « Transaction Search » est le nom de la décision du responsable de l'epic |
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
constructeur de client, routes et codes `PROVIDER_*`, cœur d'import `ProviderLine`, test S9) ; Wise (39-12a1, 39-12a2, 39-12a3, 39-12b) et
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
   une **famille indexée** déclarée une fois par prestataire (préfixe, champs, `n` de 1 à 9, **et ses variables sans numéro, `KESH_<P>_BASE_URL` et `KESH_<P>_PARALLEL`**, construites elles aussi par `format!` et donc invisibles de l'ensemble lu : sans elles, `KESH_PAYPAL_BASE_URL` et `KESH_PAYPAL_PARALLEL` rougiraient en (T), (E) et (F) ; l'ensemble déclaré rejoint les noms **connus** de (T) « transmise, lue », (E) et (F), sans entrer dans l'ensemble lu que (L) calcule ; séries 1 à 3 exigées aux deux
   compose et à `.env.example`, **de même `KESH_<P>_BASE_URL` et `KESH_<P>_PARALLEL`** (exigence portée par une liste dédiée de la garde, distincte de l'ensemble lu : sans elle, la phrase « portent » de l'AC-1 n'est contrôlée par rien), séries 4 à 9 admises absentes par une raison nommée, comme `EXCEPTIONS` ; les nouveaux
   sites de lecture de `provider_env` (appel de `env_nonempty` sur un nom construit **et** parcours `std::env::vars()` du préfixe, qui porte le jeton `env`) inscrits à `EMPLACEMENTS_AUTORISES` en forme exacte, avec leur nombre ; la forme `KESH\_PAYPAL\_<n>\_…`
   du manuel admise en (F)) — est **générique** (Wise et Stripe y ajoutent leur famille), testée par l'auto-test (S) de
   la garde, et aucune liste fermée existante n'est relâchée pour les autres variables. La garde (V) admet déjà les formes prévues (`${NOM:-}` d'une variable qui n'est dans aucune de ses listes fermées) ; les lignes `.env.example` des séries sont **commentées ou vides**, jamais au gabarit `<…>` (refusé au démarrage, AC-2). Tout est lu
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
   *(Amendé le 2026-11-21, remédiation de la revue de code P1 : (F3) la fin d'une fenêtre est désormais le **début de
   la suivante** (`00:00:00` de Zurich), recouvrement d'une seconde, l'inclusivité d'`end_date` n'étant pas établie —
   la transaction de cette seconde, lue deux fois, est une répétition identique écartée ; (F1) une `to` future est
   ramenée au jour du jour **avant** le découpage (`effective_to`, `period_windows`), et `windows` n'émet aucune
   fenêtre qui commence après `now` ; le nom de l'import et la trace disent la période **lue** ; (F11) `too_old` se
   juge au jour de Zurich du **plancher** `now − 3 ans + 1 h` (`history_floor`), jour admis, la première fenêtre
   commençant au plancher lui-même.)*
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
   que lit la phase 1**, relevés sur le CSV réel du responsable de l'epic (`kesh-gate-logs/epic-38-kiss/paypal-2024-comprendre.md`) :
   les listes de `classify` (38-11c2 AC-4 : ventes, conversions, retraits) et `PAYPAL_NEUTRAL_PAIR_TYPES` (38-11b AC-5).
   **Test obligatoire** : chaque libellé de ces listes est l'image d'au moins un code de la table — sans quoi une vente
   téléchargée tomberait en `Other` et une paire de retenue ne serait plus neutralisée, en silence (rien de faux n'est
   écrit, Kesh ne fait que proposer, mais le CSV et l'API ne se rapprocheraient plus de la même façon). Codes de départ
   (page de PayPal) : `T0000`, `T0001`, `T0002` (`Subscription Payment`), `T0003`, `T0006` (`Express Checkout Payment`),
   `T0007` (`Website Payment`), `T0011` (`Mobile Payment`), `T0013`, `T0200` (`General Currency Conversion`), `T0201`
   (`User Initiated Currency Conversion`), `T0400` (`General Withdrawal`), `T0403`, `T1105` (page : « reversal of general account hold »), `T1107` (`Payment
   Refund`), `T1110` (page : « hold on balance for dispute investigation »), `T1111` (page : « cancellation of hold for dispute resolution », levée de `T1110`), `T1500` à `T1503` (`T1501` : « account hold
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
   (`crates/kesh-api/tests/audit_route_registry.rs`) en `Traced, SansEcritureAuJournal`. **Toutes les assertions de
   `the_registry_partition_is_what_the_story_declares` sont recomptées depuis la source** (six `assert_eq!` à `origin/main`, `audit_route_registry.rs` :1098, :1108, :1135, :1170, :1175, :1181 : cinq à valeur littérale — 157, 149, 4, 4, 161 — et l'égalité de partition `traced + exempt + no_matter = len`) : `LIB_ROUTES.len()` (+ 1, la raison
   ajoutée à son message), `traced` (+ 1, ajouté à son énumération), `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` (+ 1,
   « 161 » à l'origine) ; `exempt` (4) et `no_matter` (4) inchangées ; la somme `traced + exempt + no_matter = len` reste
   vraie. **Les deux phrases** que `le_manuel_dit_le_compte_du_registre` lit dans le manuel d'administration
   (`docs/manual/fr/admin-manual.tex`, paragraphe de l'audit, `:3302` à `origin/main`), **chacune exactement une fois**, sont
   mises à jour **dans cette story** (son gate les lit) : « N des M routes à verbe mutant » (N = `traced`, M = `len`) et
   « N + E + R = M » (`149 + 4 + 4 = 157` à l'origine) — **`.tex` seul**, comptes **recomptés** après ceux de 38-11b et
   38-11c2 et des autres epics fusionnés (même règle que le socle S3 de `epic-39-wise.md`). Refus du `POST` dans l'ordre, rien lancé : 403 (clé d'API) → 400
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
   en 409 jusqu'au redémarrage) : le `JoinHandle` est surveillé et la tâche passe en `failed` `INTERNAL_ERROR` (faute de Kesh, non du prestataire : `PROVIDER_API_ERROR` laisserait croire à une panne de PayPal ; décision de l'orchestrateur, 2026-11-20), un test
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

- [x] T0 — Reprendre `main` et les branches des Epics 39, 40 si fusionnées : socle déjà écrit ? cœur `ProviderLine` déjà factorisé ? État au Change Log.
- [x] T1 — `provider_env` générique, série PayPal, validation, masque `Debug` (AC-1 à AC-3) ; séries 1 à 3 aux deux fichiers `docker-compose` et à `.env.example` ; famille indexée dans `configuration_transmise.rs` (AC-1).
- [x] T2 — `kesh-import::paypal_api` : lecture de page, table des types, conversion, contrôles partagés avec `read_row` (AC-5).
- [x] T3 — `provider_download` (`windows`, reprise, tâches, sémaphore), constructeur de client extrait, client PayPal (AC-4 à AC-6).
- [x] T4 — Appel du cœur commun `ProviderLine` (ou son extraction hors de la route PayPal) (AC-7).
- [x] T5 — Routes, registre et phrase du manuel, audit, codes `PROVIDER_*` et messages aux quatre locales, test S9 (AC-8, AC-9).
- [x] T6 — Serveur simulé et tests (AC-10).

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
39-12a2, 39-12a3, 39-12b, 40-8a, 40-8b), `crates/kesh-import/src/paypal.rs` de
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
  (autorisation du responsable de l'epic du 2026-11-20) : chaque requête passe d'abord par `kesh-gate-logs/appel-reel.sh paypal
  "<motif>"` (une par minute, quarante au plus pour PayPal ; code 3 = quota épuisé, s'arrêter), avec
  `data-not-committed/paypal-api.env`, jamais affiché, commité ni envoyé ; aucune réponse réelle n'entre dans le dépôt.
  Budget conseillé : un jeton, puis une ou deux pages d'une période dont le responsable de l'epic a le CSV, pour vérifier les points « non
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
  réels de développement selon les autorisations du responsable de l'epic.
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
- 2026-11-20 — validation P4 ciblée sur le commit `e6da3774` (Claude Sonnet 5.5, contexte frais, une lentille ; rapport
  `kesh-gate-logs/epic-38-kiss/validation/38-12-P4-ciblee.md`) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW, remédiés. MEDIUM : la
  famille indexée ne couvrait pas `KESH_PAYPAL_BASE_URL` et `KESH_PAYPAL_PARALLEL` (noms sans numéro, construits eux aussi) —
  ajoutés à la famille, avec le parcours `env::vars()` à inscrire en forme exacte. LOW : lignes `.env.example` commentées ou
  vides (pas de gabarit `<…>`) ; décisions de l'orchestrateur appliquées — panique de tâche en `INTERNAL_ERROR`, `T1111`
  (« Cancellation of hold for dispute resolution », relevé sur la page publique des codes) aux codes de départ. Ajout du coordinateur (P3 Wise/Stripe, M2) :
  les deux phrases du manuel et les assertions du registre écrites en toutes lettres à l'AC-8.
- 2026-11-20 — validation P5 ciblée sur les commits `977bf9ec` et `452606a3` (modèle : claude-haiku-5-5, contexte frais, une lentille ; rapport `kesh-gate-logs/epic-38-kiss/validation/38-12-P5-ciblee.md`) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW, remédié. LOW : l'AC-1 disait que les compose et `.env.example` « portent » `KESH_<P>_BASE_URL` et `KESH_<P>_PARALLEL` (ligne 76), mais ne les exigeait nulle part : la garde ne l'aurait pas contrôlé — exigence ajoutée à l'AC-1, par une liste dédiée distincte de l'ensemble lu. Vérifié au code de `origin/main` : `JETONS_SURVEILLES`, `EMPLACEMENTS_AUTORISES` (`Forme::Exacte`, `env :: vars ( )` sans nom lu), (V) admet `${NOM:-}`, `AppError::Internal` rend `INTERNAL_ERROR` (`errors.rs`), `is_template_placeholder` aux lignes 650 et 683 ; registre 157 / 149 / 4 / 4 / 161 et manuel « 149 des 157 », « 149 + 4 + 4 = 157 » (une fois chacune, `admin-manual.tex:3302`) ; `T1111` confirmé sur la page publique PayPal. Le test de partition porte six `assert_eq!` (et non cinq), dont la somme : l'AC-8 les couvre toutes.
- 2026-11-20 — alignement demandé par le coordinateur : l'AC-8 écrit le décompte du test de partition (six `assert_eq!` à `origin/main`, cinq à valeur littérale et une égalité de partition). Les fiches 39-12a2 et 40-8b, citées comme modèle, n'existent pas encore à HEAD : la formulation est la même, sans renvoi à elles.
- 2026-11-20 — développement (`bmad-dev-story`, Claude Opus 5.5), branche `story/38-12a` partie de `epic/38-paypal` à
  `ca34a3f4` ; commits de code `d64688fc` (socle, client, routes, tests), `04436455` (montant à 17 chiffres). Détail au
  Dev Agent Record. Renvois Wise corrigés sur demande du coordinateur (l. 64, l. 300 de cette fiche, `epic-38-paypal.md`
  l. 190 : « 39-12a1, 39-12a2, 39-12a3, 39-12b »).
- 2026-11-21 — reprise par un second développeur (Claude Opus 5.5, le premier ne répondant plus ; 14 fichiers non
  commités trouvés, aucun processus resté sur ce worktree, le dernier gate ciblé `cible-6.log` vide, interrompu). Le gate
  complet à `04436455` (`gate-backend.log`, 2026-11-21 00:58) rendait 6268 tests, 11 échecs, tous dans des gardes
  (`audit_label_registry` 2, `date_du_jour_garde` 7, `dates_recues_garde` 2) : les corrections non commitées les
  visaient, relues et commitées en `1b791613` ; le gate ciblé suivant a rendu deux échecs de plus dans
  `audit_label_registry` (site indirect sans littéral, compte de l'inventaire), corrigés en `9bbf1388` à l'identique de
  la 40-7b (`c760dc49`). Détail au Dev Agent Record, § Reprise.
- 2026-11-21 — revue de code P1 (Claude Sonnet 5.5, passe complète, une lentille ; rapport
  `kesh-gate-logs/s38-12a/revue-p1-rapport.md`) : 0 CRITICAL, 1 HIGH, 0 MEDIUM, 12 LOW. Remédiation (Claude Opus 5.5)
  au commit `3e22ea0e` : F1 (HIGH), F2, F3, F4, F5, F6, F9, F10, F11, F13 corrigés dans la story (le socle sert Wise et
  Stripe) ; F7, F8, F12 en dette, issues 17 à 19 de `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`. AC-4 amendé
  (recouvrement d'une seconde, fin future ramenée au jour du jour, plancher de l'historique). **12 tests ajoutés**
  (recomptés de `9bbf1388` à `3e22ea0e` par `#[test]`/`#[tokio::test]`/`#[sqlx::test]` : `provider_download.rs` 11 → 15,
  `http_client.rs` 0 → 2, `provider_env.rs` 7 → 8, `routes/provider_imports.rs` 2 → 3, `paypal_download_e2e.rs` 9 → 11,
  `kesh-import` `csv/amount.rs` 2 → 3, `paypal_api.rs` 11 → 12) ; deux tests existants étendus (`suite_des_pauses_et_borne`,
  `nom_d_erreur_de_paypal`) et les bornes de fin attendues décalées d'une seconde (F3). Onze sondes négatives, **11
  rouges sur 11** (`mut/mutations-p1.log`). Détail au Dev Agent Record, § Remédiation de la revue de code P1. Passe P2
  complète (Opus) à lancer : `kesh-gate-logs/s38-12a/revue-p2-prompt.md`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), worktree `kesh-wt38p2`.

### T0 — état trouvé

`main`, `origin/main`, `epic/39-wise`, `epic/40-stripe` relus le 2026-11-20 : **aucun** socle (`provider_env`,
`provider_download`, `ProviderLine`, `paypal_api`) n'existe sur aucune branche. Décision : 38-12a écrit le socle et
**extrait** le cœur d'import de la route PayPal de 38-11b (AC-7, deuxième branche).

### Ce que les appels réels ont confirmé (métadonnées seules, aucune donnée personnelle conservée)

33 appels sur 40 (`appel-reel.sh paypal`, journal `kesh-gate-logs/appels-reels/paypal.log`), lecture seule, deux
comptes du responsable de l'epic ; réponses brutes gardées hors dépôt (`data-not-committed/paypal-api-samples/`, mode 600) ; analyse par
décomptes seulement.

| Point « non confirmé » de la fiche | Constat |
|---|---|
| hôte réel | `https://api-m.paypal.com` répond pour un compte réel (OAuth et Transaction Search) |
| OAuth | `POST /v1/oauth2/token`, Basic, `grant_type=client_credentials` : `access_token`, `token_type` `Bearer`, `expires_in` **32 400**, portée `…/services/reporting/search/read` ; même jeton rendu aux demandes répétées (échéance décroissante) |
| fuseau des dates | ⚠️ **contredit** : avec `PayPal-Enforce-ISO8601-Format: true`, les 341 dates de 2024 sont en **`+00:00`** (UTC), non dans le fuseau du compte. Correction à la conversion (Notes de dev.) : l'instant est ramené à l'heure de Zurich (`kesh_core::horloge::heure_suisse`, fonction neuve), date et heure du CSV |
| signe des frais | négatif sur les encaissements (`T0007`, `T0002`) et sur les paiements `T0000` qui en portent ; **absent** ailleurs ; aucun frais positif observé (aucun remboursement avec frais sur la période) — admis de tout signe |
| `payer_info` d'un débit | la **contrepartie** : 6 adresses distinctes sur 26 débits qui en portent, de codes différents (un titulaire en donnerait une seule) ; conversions et retenues de contrôle sans adresse |
| codes et libellés | 2024, compte 1 : 341 transactions, codes `T0000` (13), `T0002` (66), `T0003` (1), `T0006` (10), `T0007` (210), `T0201` (36), `T1107` (1), `T1110` (1), `T1111` (1), `T2105` (1), `T2106` (1) ; statuts S 339, P 1 (`T1110`), D 1 (`T1111`) — **même décompte et même ventilation des statuts** que le CSV de 2024 (`paypal-2024-comprendre.md`) ; libellés attribués par concordance du sens et des types du CSV. **Jointure par identifiant impossible** (CSV de 2024 supprimé avant la phase 2) : écrit tel au commentaire de `PAYPAL_EVENT_TYPES` (✔ concordance / doc) |
| `paypal_reference_id_type` | `TXN`, `PAP` et **`RP`** (abonnements, 66), absent de la liste du schéma ; référence reprise telle quelle |
| fenêtre vide | `total_pages = 0`, pas de `transaction_details` (compte 2 sur 2025) — testé |
| limite de 31 jours | une fenêtre de 31 jours **plus une heure** est acceptée : les fenêtres de 30 jours de Zurich gardent leur marge |
| `ending_balance` | **présent** sur les 341 lignes malgré `balance_affecting_records_only=Y` ; gardé `None` comme l'écrit l'AC-5 (le solde de clôture reste `NULL`) — piste pour plus tard |
| montants | chaînes à deux décimales (`"value": "12.34"`), devise des frais = devise du montant |
| `Retry-After`, limitation, `RESULTSET_TOO_LARGE` | non rencontrés (33 requêtes) : non confirmés, joués au serveur simulé |
| nom de la fonction à cocher | non vérifiable par l'API (la portée du jeton dit « reporting/search/read ») |

### Implementation Plan / décisions

- **Socle** (générique, pour 39-12a1/a2/a3/b et 40-8a/b) : `src/provider_env.rs` (S1 : `ProviderSpec` — clé,
  champs requis, **champs facultatifs** `optional_fields` demandés par le coordinateur, vides pour PayPal —, `read` pure
  testée sans environnement, `ProvidersConfig::from_env` seule lecture, `ConfigError::InvalidProviderVar` qui nomme la
  variable, gabarit refusé par `is_template_placeholder` rendu `pub(crate)`) ; `src/provider_download.rs` (S2/S3 :
  `windows`, `Window::split`, `check_period`, `pause_for`, `with_retry`, `ProviderGate` — sémaphore et pause commune —,
  `Wait` injectable, `Downloads` — tâches, purge à 1 h, `JobView` —, `spawn_job` qui conclut une panique en
  `INTERNAL_ERROR`, `error_body` qui rend le corps d'une `AppError` avec `details.provider`) ; `src/http_client.rs`
  (S5 : `build_client`, `read_body_bounded`, extraits de `exchange_rates/source.rs`, qui les appelle) ;
  `routes/provider_imports.rs` (S6 : `ProviderLine`, `ProviderGroup`, `load_balances`, `import_lines`, `dedupe_groups`,
  empreinte d'un fichier ou d'un téléchargement) ; codes `PROVIDER_*` (S4) dans `errors.rs` (502/503 pour les erreurs
  du prestataire, rendues dans le corps d'une tâche), messages aux quatre locales sans nom de prestataire ; marqueur S9
  `tests/common/secrets_prestataire.rs`.
- **Cœur extrait** : la route CSV appelle `import_lines` ; ses tests unitaires et `paypal_import_e2e` passent
  **inchangés en assertions** (le test `doublon_dans_le_fichier_et_en_base` appelle `dedupe_groups` au lieu de
  `dedupe_rows`, retiré). L'unité des doublons devient le **groupe** (une ligne de la source) : un premier essai par
  `group_id` contigu fusionnait deux lignes répétées à la suite — corrigé avant commit. Opération du rejeu gardée
  (`paypal_imports::create`, témoin de `paypal_import_e2e`). Écart assumé : une ligne à brut **et** frais nuls ne
  produit aucun groupe et n'entre plus dans l'ensemble « déjà vu » (cas dégénéré, sans transaction).
- **PayPal** : `src/paypal_api/mod.rs` (jeton par tâche renouvelé à 60 s de l'échéance et une fois sur 401 ; pages de
  500 ; coupe sur `RESULTSET_TOO_LARGE` jusqu'au jour ; 16 Mio ; 60 s ; parallélisme par `JoinSet`, résultats rangés par
  fenêtre) ; `kesh-import/src/paypal_api.rs` (lecture et conversion pures ; contrôles **extraits** de `read_row` :
  `normalize_currency`, `normalize_transaction_id`, `sign_mismatch`, `PaypalLineError::new` `pub(crate)`) ; code de ligne
  neuf `STATUS` ; `to_paypal_row` rend **toutes** les erreurs d'une ligne (`Vec`), comme le lecteur du CSV (la fiche
  écrivait une seule) ; `routes/paypal_downloads.rs` (POST/GET, corps lu en octets pour que le 403 de la clé précède le
  400).
- **Montants** (constat de la revue du client Wise, transmis par le coordinateur) : PayPal envoie ses montants en
  **chaînes** (`PaypalApiMoney::value: Option<String>`, `kesh-import/src/paypal_api.rs`), lues par
  `parse_dot_decimal_amount` — aucun flottant ; un montant en nombre JSON fait refuser la page. Test au corps brut à 17
  chiffres (`un_montant_a_dix_sept_chiffres_…`). Rien à propager au socle : il ne lit aucun montant.
- **Libellés** : test de couverture `la_table_couvre_les_libelles_de_la_phase_1` — `PAYPAL_NEUTRAL_PAIR_TYPES` et les
  listes de `classify` de 38-11c2, **recopiées de sa fiche** (38-11c2 se développe en parallèle, ses constantes ne sont
  pas sur cette branche) : ⚠️ **à remplacer par ses constantes à la fusion**. Quatre libellés sans code, inventaire fermé
  avec raison (`PAYPAL_CSV_LABELS_WITHOUT_CODE`) : `Shopping Cart Payment Received`, `Payment Received`, `Withdraw Funds
  to Bank Account`, `Reversal of Account Hold`.
- **Garde `configuration_transmise.rs`** : familles indexées **dérivées** de `kesh_api::provider_env::ALL_PROVIDERS`
  (aucune liste recopiée) ; connues en (T), (E), (F), hors ensemble lu ; séries 1 à 3 et variables sans numéro exigées
  aux deux compose et à `.env.example` ; séries 4 à 9 admises absentes, raison contrôlée par le marqueur `_4_…` des trois
  fichiers ; deux entrées (L) exactes (`env :: vars_os ( )`, `env_nonempty`) ; auto-test `s_familles_indexees`.
- **Registre** : `paypal_downloads::create` `Traced, SansEcritureAuJournal` ; recomptés depuis la source : 158 → 159,
  `traced` 150 → 151, exempt 4, no_matter 4, total avec les routes de test 162 → 163, `SansEcritureAuJournal` 129 → 130
  (partition de rejeu), `tout.len()` 162 → 163 ; manuel « 151 des 159 », « 151 + 4 + 4 = 159 » (`.tex` seul, PDF à la
  release). Le `GET` n'a pas de verbe mutant.
- Modules touchés : `kesh-import`, `kesh-api`, `kesh-i18n` **et `kesh-core`** (`heure_suisse`, pure, testée) — quatrième
  module, dû au constat UTC.

### Reprise (2026-11-21)

- **Montants** (question de l'orchestrateur) : vérifié au code, aucun montant lu comme nombre JSON —
  `PaypalApiMoney::value: Option<String>` (`crates/kesh-import/src/paypal_api.rs:142`), lu par `parse_dot_decimal_amount`
  (`:350`, `:362`) ; la page se lit par `serde_json::from_slice` dans une structure typée, sans `f64`, `Number` ni
  `Value` (`:252`) ; côté `kesh-api` (`paypal_api/`, `provider_download.rs`, routes), aucun `as_f64` ni `Number` (le seul
  `f64`, `provider_download.rs:230`, est une durée de pause). Prouvé par
  `un_montant_a_dix_sept_chiffres_se_lit_exactement_et_un_nombre_json_est_refuse` (`:700`), au corps JSON brut :
  `"999999999999999.99"` en chaîne lu au chiffre près (net `999999999999999.98`), le même en nombre JSON refusé
  (`PaypalApiError::Malformed`) ; vert au gate ciblé de la reprise.
- **`1b791613`** (corrections trouvées non commitées, relues) : `from`/`to` de `DownloadRequest` lus par
  `dates_recues::date` (bornés à la plage des années ; `COMPTE_CHAMPS` 70 → 72) ; action `paypal_download.created` à
  `ACTIONS` et ses libellés aux quatre locales ; site indirect `provider_imports.rs` à `SITES_INDIRECTS` ; exemption
  `date_du_jour_garde` d'une lecture de `date_naive` à `paypal_api.rs` (la conversion n'en porte plus qu'une, la date
  locale calculée une fois avec l'instant) ; test de `heure_suisse` sans `date_naive`.
- **`9bbf1388`** : `AUDIT_ACTIONS` (`provider_imports.rs`) déclare en toutes lettres les deux actions que les
  appelants passent au cœur, contrôlées par `debug_assert!` à l'écriture de la trace ; mentions « neuf » → « dix » et
  compte de `SITES_INDIRECTS` à 10 dans `audit_label_registry.rs`. **Même texte que la 40-7b** (`c760dc49`, worktree
  `kesh-wt40b`) au tableau près : à la fusion, la seule ligne en conflit est celle d'`AUDIT_ACTIONS`, qui prend
  l'union des deux (`[&str; 3]`) ; le compte « dix » des deux côtés devient **onze** si la 40-7b inventorie un site
  de plus que celui-ci — à recompter à la fusion, non à fusionner.
- **Doublons voisins** (constat transmis par le coordinateur, relevé par la 40-7b sur sa copie `dedupe_lines`) :
  **prémisse fausse pour cette branche**. Ici `dedupe_groups` reçoit des **groupes explicites** (`&[ProviderGroup]`, un
  groupe par ligne de la source, `provider_imports.rs:223`) et ne regroupe rien par `group_id` contigu : deux lignes
  identiques côte à côte sont deux groupes, la seconde est vue (`seen.insert` faux) et comptée en doublon. C'est le
  défaut que le premier essai de la story avait eu et qui a été corrigé avant le commit `d64688fc` (§ Implementation
  Plan, « Cœur extrait ») ; la 40-7b en a recopié une version antérieure. Le test `doublon_dans_le_fichier_et_en_base`
  (`paypal_imports.rs:714`) porte exactement ce cas (`a`, `b = a.clone()` voisines) et passe : `gate-backend.log`
  (00:58) et `cible-7.log` de la reprise ; la mutation M5 (« doublon de groupe non vu ») l'avait fait rougir. Aucun
  correctif ici ; ⚠️ **à la fusion des Epics 38 et 40**, le cœur garde une seule forme : `dedupe_groups` (groupes
  explicites), la 40-7b passant ses lignes groupées par ligne de la source — sinon deux fonctions pour un même cœur.
- Aucun appel réel à PayPal pendant la reprise ; `data-not-committed/` non lu.

### Remédiation de la revue de code P1 (2026-11-21)

Rapport : `kesh-gate-logs/s38-12a/revue-p1-rapport.md` (Sonnet 5.5, passe complète) — 0 CRITICAL, 1 HIGH, 0 MEDIUM,
12 LOW. Prémisse de F1 vérifiée au code avant correction : `windows` découpait jusqu'à `to` sans la ramener au jour du
jour, `Window::new` ne ramenait que la fin à `now`, et le test existant `fin_d_annee_et_fin_ramenee_a_l_instant`
portait **lui-même** une fenêtre inversée (2025-01-31, début après `now` = fin) sans la voir ; `download` lançait une
tâche par fenêtre en parallélisme > 1. La chaîne de gates sur `9bbf1388` (`cible-puis-complet.sh`), encore en file
d'attente, a été arrêtée avant de tourner.

| Finding | Traitement | Où |
|---|---|---|
| F1 HIGH — `to` future : fenêtres inversées, nombre non borné | **corrigé** : `to` ramenée au jour de Zurich avant le découpage (`effective_to`, `period_windows`), `windows` s'arrête à la fenêtre qui contient `now` (défense en profondeur), `run_bounded` borne les tâches en vol par le parallélisme ; nom de l'import et trace d'audit = période **lue** (choix écrit au module des routes) | `provider_download.rs`, `paypal_api/mod.rs` (`download`), `routes/paypal_downloads.rs` |
| F2 — transport tout passager | **corrigé** : passager = délai, ou erreur d'entrée-sortie de connexion (refusée, coupée, injoignable…) dans la chaîne des causes ; certificat, hôte inconnu, schéma → `PROVIDER_API_ERROR` sans reprise. Limite écrite : une panne passagère du résolveur se lit comme un hôte inconnu | `http_client.rs` (`is_transient_transport`), `provider_download.rs` (`classify_transport`) |
| F3 — inclusivité d'`end_date` | **corrigé** par recouvrement d'une seconde : la fin d'une fenêtre est le début de la suivante (AC-4 amendé) | `Window::new` |
| F4 — `AUDIT_ACTIONS` `[&str; 2]`, `debug_assert!` | **corrigé** : tranche `&[&str]`, contrôle `check_audit_action` en tête d'`import_lines`, `AppError::Internal` en version publiée | `provider_imports.rs` |
| F5 — colle PayPal non partagée | **corrigé** : `retry_after` (`http_client`), `classify_status`/`StatusClass`, `classify_transport` (`provider_download`), appelés par PayPal | |
| F6 — champs lus par chaîne, erreur dans la tâche | **corrigé** : constantes `PAYPAL_CLIENT_ID`, `PAYPAL_SECRET` (spécification et client), `ProviderSpec::require_field` (nom non déclaré → `Internal`), client construit **avant** la tâche (refus synchrone) | `provider_env.rs`, `paypal_downloads.rs` |
| F7 — identifiant répété à montants différents bloque la période | **dette** (issue 17 de `epic-38-kiss/issues-a-ouvrir.md`) | |
| F8 — séries globales, libellés par société | **dette** (issue 18) | |
| F9 — `Retry-After: 0` | **corrigé** : pause entre 1 et 120 s (`RETRY_AFTER_FLOOR`) | `pause_for` |
| F10 — nom d'erreur non borné au journal | **corrigé** : 64 caractères (`ERROR_NAME_MAX_CHARS`) | `paypal_api/mod.rs` |
| F11 — borne des trois ans | **corrigé sans perdre de jour** : plancher `history_floor` = `now − 3 ans + 1 h` ; `too_old` au jour de Zurich du plancher (jour admis) ; la première fenêtre commence au plancher, la première moitié d'une fenêtre coupée garde ce début | `provider_download.rs` |
| F12 — lignes sans transaction hors « déjà vu » | **dette** (issue 19) : le correctif change la forme de `ProviderGroup`, partagée avec la 40-7b | |
| F13 — virgule des milliers sur une valeur d'API | **corrigé** : `parse_api_amount` (forme fermée sans virgule), commun pour Wise et Stripe | `kesh-import/src/csv/amount.rs`, `paypal_api.rs` |

Tests ajoutés : 12, décompte au Change Log. Sondes négatives de la remédiation (`kesh-gate-logs/s38-12a/mut/mutations-p1.log`,
script `mutations-p1.py`) : N1 `windows` sans arrêt à `now`, N2 route sans fin ramenée au jour, N3 parallélisme non borné,
N4 transport toujours passager, N5 sans recouvrement, N6 action de trace non contrôlée, N7 champ non déclaré admis,
N8 `Retry-After: 0` sans pause, N9 nom d'erreur non borné, N10 plancher ignoré, N11 virgule des milliers admise —
**11 rouges sur 11**, fichiers restaurés (vérifié par `git status`).

⚠️ **Raccords** : la 40-7b (Stripe, worktree `kesh-wt40b`) et la 39-12a1 (Wise) devront **reprendre la nouvelle tête**
de `story/38-12a` : la signature de `check_period` change (`now` au lieu du jour du jour), les routes appellent
`period_windows` et non `windows`, `download` passe par `run_bounded`, les clients classent par `classify_status` /
`classify_transport` et lisent `retry_after` dans `http_client`, les champs de série par `require_field` ;
`AUDIT_ACTIONS` est une tranche (l'union avec `stripe_import.created` à la fusion n'est plus qu'une ligne ajoutée) ; les
montants d'une API se lisent par `parse_api_amount`.

### Debug Log

- La cible partagée `kesh-target` a été abandonnée en cours de story (règle de l'orchestrateur) : **tous les gates
  déclarés ci-dessous ont tourné sur `kesh-targets/kesh-wt38p2`, `CARGO_INCREMENTAL=0`** ; les essais antérieurs sur la
  cible partagée (vérifications de compilation) ne comptent pas.
- La base `kesh_s3812a` (dev2, 3307) a disparu deux fois (recréation de dev2) : remise à zéro avant chaque gate.

### Completion Notes

- Mutations (sonde négative, `kesh-gate-logs/s38-12a/mut/mutations.log`) : M1 `Retry-After` non borné, M2 clé d'API
  admise au POST, M3 e-mail d'un débit en émetteur, M4 série partielle admise, M5 doublon de groupe non vu — **5 rouges
  sur 5**.
- Gates : voir « Gates » ci-dessous (complétés au dernier commit de code).

### Points relevés hors périmètre (à ouvrir en issue par l'orchestrateur ; `gh` interdit ici)

1. `PAYPAL_NEUTRAL_PAIR_TYPES` (38-11b) ne porte pas `Cancellation of Hold for Dispute Resolution` (`T1111`, levée de
   `T1110`) : une paire de litige `Completed` ne serait pas neutralisée, ni au CSV ni à l'API. Sur 2024 elle est en
   statuts P/D, donc écartée de toute façon.
2. `ending_balance` est rendu par l'API : le solde de clôture d'un téléchargement pourrait être renseigné.

### File List

Ajoutés : `crates/kesh-api/src/http_client.rs`, `crates/kesh-api/src/paypal_api/mod.rs`,
`crates/kesh-api/src/provider_download.rs`, `crates/kesh-api/src/provider_env.rs`,
`crates/kesh-api/src/routes/paypal_downloads.rs`, `crates/kesh-api/src/routes/provider_imports.rs`,
`crates/kesh-api/tests/common/secrets_prestataire.rs`, `crates/kesh-api/tests/paypal_download_e2e.rs`,
`crates/kesh-import/src/paypal_api.rs`.
Modifiés : `.env.example`, `docker-compose.yml`, `docker-compose.prod.yml`, `docs/manual/fr/admin-manual.tex`,
`crates/kesh-api/src/{config.rs, errors.rs, lib.rs, main.rs, middleware/auth.rs, exchange_rates/source.rs,
routes/mod.rs, routes/paypal_imports.rs, audit_labels.rs}`, `crates/kesh-api/tests/{audit_route_registry.rs,
audit_label_registry.rs, date_du_jour_garde.rs, dates_recues_garde.rs, common/mod.rs,
configuration_transmise.rs, invoice_frozen_pdf_e2e.rs, invoice_send_email_e2e.rs, password_recovery_e2e.rs,
setup_admin_e2e.rs}`, `crates/kesh-core/src/horloge.rs`, `crates/kesh-i18n/locales/{de-CH,en-CH,fr-CH,it-CH}/messages.ftl`,
`crates/kesh-import/src/{lib.rs, paypal.rs}`, `_bmad-output/implementation-artifacts/{38-12a-telechargement-api-paypal.md,
sprint-status.yaml}`, `_bmad-output/planning-artifacts/epic-38-paypal.md`.

### Gates

- Frontend à `1b791613` (`front-*-2.log`) : `check`, `lint-i18n-ownership`, `test:unit` (4612 tests, 278 fichiers),
  `build` verts ; `9bbf1388` ne touche ni le frontend ni les catalogues.
- Backend : gate ciblé à `1b791613` (`cible-7.log`, dev4) : 1089 exécutés, 1087 passés, 2 échecs
  (`audit_label_registry`, corrigés en `9bbf1388`). **Gate ciblé, gate complet et `gate-ci.sh` au dernier commit de code
  `9bbf1388` : lancés (`cible-puis-complet.sh`, journaux `cible-8.log`, `gate-backend-8.log`, `gate-ci.log`), résultats
  non encore relevés** — à reporter ici avant de passer la fiche en `review`.
- E2E : aucun écran touché (l'écran est la 38-12b) ; non lancé.
- *(Annoté le 2026-11-21.)* La chaîne `cible-puis-complet.sh` sur `9bbf1388` a été **arrêtée en file d'attente** avant
  de tourner (remédiation de la P1) : `cible-8.log`, `gate-backend-8.log` n'existent pas, aucun résultat à reporter.
- **Remédiation de la P1** (code au commit `3e22ea0e`) :
  - gate ciblé sur l'arbre de la remédiation avant commit (`cible-9.log`, dev4) : fmt, clippy workspace verts ;
    nextest 1101 exécutés, 1100 passés, 1 échec (`une_fin_future_est_ramenee_au_jour_du_jour` : la période d'avril
    choisie n'écrivait aucun import, test corrigé sur mars) ;
  - puis `paypal_download_e2e` seul (`cible-10.log`, dev5) : **11 sur 11**, fmt vert ; sondes : 11 rouges sur 11 ;
  - **gate backend complet et `gate-ci.sh` au commit `3e22ea0e` : NON EXÉCUTÉS** — lancés (`complet-p1.sh`), arrêtés en
    file d'attente par la station (mémoire basse) avant de tourner ; à relancer avant la passe P2 ou avec elle
    (`heavy.sh kesh-gate-logs/s38-12a/complet-p1.sh`). Frontend non touché par la remédiation.


### Remédiation de la revue de code P2 (2026-11-21)

Rapport : `kesh-gate-logs/s38-12a/revue-p2-rapport.md` (Opus 5.5, passe complète) — 0 CRITICAL, 0 HIGH, 1 MEDIUM, 6 LOW.

| Finding | Traitement | Où |
|---|---|---|
| G1 MEDIUM — connexion fermée après l'envoi de la requête classée définitive | **prémisse vérifiée par exécution** : un serveur du bouclage qui lit la requête puis ferme sans répondre fait rougir, sur le code d'avant, le test du client (`http_client::tests::connexion_fermee_apres_la_requete_est_passagere`) et l'E2E (`connexion_fermee_apres_la_requete_est_reprise`) — `p2-rouge.log`, 2 rouges sur 3, le test de contrôle `erreurs_reelles_de_reqwest_classees` passant. **Corrigé** : la chaîne des causes est aussi lue pour une `hyper::Error` (`is_incomplete_message`, `is_closed`, `is_canceled`) ; `hyper = "1"` en dépendance directe de `kesh-api`, version déjà au graphe (1.12.0) | `http_client.rs`, `Cargo.toml` |
| G2 — fenêtre `start == end` | **corrigé** : `windows` s'arrête à `start >= now`, `period_windows` écarte `start >= end` | `provider_download.rs` |
| G3 — `Window::split` inversée hors des routes | **corrigé** : `windows` ramène `to` au jour du jour (`effective_to`) | `provider_download.rs` |
| G4 — marge du plancher | **doc** : la première heure du jour du plancher n'est pas lue, le nom et la trace disent le jour demandé (`check_period`) ; à reprendre au manuel de la 38-12b | `provider_download.rs` |
| G5 — `AUDIT_ACTIONS` lié à la main | **corrigé** : test `les_actions_du_coeur_d_import_sont_inventoriees_et_libellees` (égalité avec l'entrée de `SITES_INDIRECTS`, libellé présent) | `tests/audit_label_registry.rs` |
| G6 — 408 définitif | **corrigé** : 408 passager (`classify_status`) | `provider_download.rs` |
| G7 — clippy non rejoué après le dernier test, gate complet non lancé | **constat exact, aucun correctif de code** : gate ciblé de cette remédiation (fmt, clippy workspace, nextest `kesh-api` lib, `paypal_download_e2e`, `audit_label_registry`, `textes_coherents`) ; le gate complet, `gate-ci.sh` et l'E2E complet restent à lancer par l'orchestrateur avant `done` | |

Tests ajoutés : 3 de plus (client, E2E, `audit_label_registry`) et des assertions aux tests `une_fin_future_s_arrete_a_l_instant` et `statuts_classes`. Gate backend complet, `gate-ci.sh` et E2E complet : **non exécutés** par cette remédiation.

### Remédiation de la revue de code P3 (2026-11-21)

Rapport : `kesh-gate-logs/s38-12a/revue-p3-rapport.md` (passe ciblée, une lentille) — 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW.

| Finding | Traitement | Où |
|---|---|---|
| P3-1 — seconde moitié de `Window::split` de durée nulle | **corrigé** : `Window::is_empty` (`start >= end`), la moitié vide n'est pas envoyée à PayPal par `fetch_window` ; test `une_fin_future_s_arrete_a_l_instant` étendu (`now` à minuit + 0,5 s, `split` d'une fenêtre 19–20 novembre) | `provider_download.rs`, `paypal_api/mod.rs` |
| P3-2 — idempotence de la reprise non écrite | **doc** : `is_transient_io_chain` et `is_transient_transport` disent que la reprise suppose une requête idempotente (`is_incomplete_message`/`is_closed` : requête possiblement reçue) ; `token_once` déclare le `POST` du jeton rejouable sans danger ; à trancher (clé d'idempotence) avant tout `POST` créateur de Wise ou de Stripe | `http_client.rs`, `paypal_api/mod.rs` |
| P3-3 — couverture de la reprise limitée au jeton | **corrigé** : E2E `get_coupe_apres_la_requete_est_repris` (le `POST` du jeton est servi en 200, seul le `GET` est coupé après la requête : six `GET`, `PROVIDER_UNAVAILABLE`, rien d'écrit) ; le cas « connexion du pool réutilisée » reste non joué (hyper ne rejoue pas, `retry_canceled_requests`, lu non exécuté) | `tests/paypal_download_e2e.rs` |

**Défaut trouvé au gate complet, hérité des remédiations P1/P2 (non vu des gates ciblés)** : la garde `mcp_gardes_lexicales::un_seul_ecouteur` rougissait (un `TcpListener` par test de `http_client.rs`, F2 de la P1 et G1 de la P2, dans `src/`). Correctif : les deux tests asynchrones liant un écouteur (`erreurs_reelles_de_reqwest_classees`, `connexion_fermee_apres_la_requete_est_passagere`) sont déplacés dans `tests/http_client_transport.rs`, hors du périmètre de la garde (commit `b64c00bd`). Premier gate complet (`gate-backend-12.log`, commit `020de8e2`) : 6284 exécutés, 6283 passés, 1 échec = cette garde.

Dernier commit de code `205cbf25` : une assertion de plus au test `une_fin_future_s_arrete_a_l_instant` (une dernière fenêtre qui commence à `now`, `windows` en fenêtres de 19 jours), parce que la sonde N1 (suppression de la garde `start >= now`) était **muette** depuis la P2 ; rouge après l'assertion. Test seul, aucune ligne de production.

Tests (périmètre `74979d0f` → `205cbf25`) : 1 test de plus (`get_coupe_apres_la_requete_est_repris`) ; 2 déplacés de `http_client.rs` vers `http_client_transport.rs` ; 1 test existant étendu (`une_fin_future_s_arrete_a_l_instant`). *(Annoté le 2026-11-21, revue P4, P4-4 : cette ligne disait « 2 tests existants étendus » ; le diff n'en étend qu'un.)*

Gates (journaux dans `kesh-gate-logs/s38-12a/`) :
- ciblé `020de8e2` (`cible-12.log`, dev5) : fmt, clippy workspace, nextest 1105 exécutés, 1105 passés ;
- complet `b64c00bd` (`gate-backend-13.log`, `test-fast.sh --ci`, dev6) : fmt, clippy, **6284 exécutés, 6284 passés**, 14 ignorés ;
- `gate-ci.sh` : **au dernier commit de code `205cbf25`** (`gate-ci.log`, `gate-ci-14.out`) : **6303 passés, 0 échoué, 18 ignorés**, code 0 ;
- ciblé `205cbf25` (`p4-cible.log`) : fmt, clippy workspace, nextest `kesh-api` lib + `http_client_transport` + `mcp_gardes_lexicales` : 789 exécutés, 789 passés, 5 ignorés ;
- sondes (`mut/mutations-p3.log` et `mut/mutations-n1.sh`, copie hors dépôt) : 13 mutations, N1 à N13 (dont N12 `is_empty` toujours faux, N13 branche hyper retirée) : **13 rouges sur 13** (N1 rejouée après l'assertion, N1 muette avant) ;
- E2E (Playwright) : aucun écran, non lancé. Le gate-ci du premier passage (`gate-ci-13-interrompu.log`), à `b64c00bd`, a été tué par le délai de l'enveloppe à 5415 tests passés, sans échec : il est rejoué en entier à `205cbf25`. Le gate backend complet (`b64c00bd`) précède le dernier commit, qui ne touche qu'un test unitaire : couvert par le ciblé `p4-cible.log`.

### Remédiation de la revue de code P4 (2026-11-21)

Rapport : `kesh-gate-logs/s38-12a/revue-p4-rapport.md` (passe ciblée, une lentille) — 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW. Dernier commit de code : `6584f643`.

| Finding | Traitement | Où |
|---|---|---|
| P4-1 (MEDIUM) — `add_steps(1)` avant de savoir si la moitié `b` part : `b` vide (première seconde du jour), feuille jamais comptée, travail fini `{done: N, total: N+1}` | **corrigé** : `add_steps(1)` n'est appelé que si `!b.is_empty()`, avant la lecture de `a` | `paypal_api/mod.rs` (`fetch_window`) |
| P4-2 (LOW) — garde `!b.is_empty()` sans test | **corrigé** : E2E `moitie_vide_a_minuit_non_demandee_et_non_comptee` (horloge à 2020-06-30 00:00:00,5 à Zurich, fenêtre 29–30 juin coupée par `RESULTSET_TOO_LARGE` : `done == total`, deux demandes exactement, aucune de durée nulle) ; aides `spawn_app_a`/`monter_a` (instant au choix, `spawn_app`/`monter` inchangés) | `tests/paypal_download_e2e.rs` |
| P4-3 (LOW) — N13 sans nom de test rouge | **consigné** : N13 rejouée avec la sortie complète (`mut/mutations-p5.log`) : rouges `http_client_transport::connexion_fermee_apres_la_requete_est_passagere` et `paypal_download_e2e::get_coupe_apres_la_requete_est_repris` | journal |
| P4-4 (LOW) — « 2 tests existants étendus » | **corrigé** : « 1 test existant étendu » (annotation datée à la remédiation P3) | ce fichier |

Tests (périmètre `205cbf25` → `6584f643`) : 1 test de plus (`moitie_vide_a_minuit_non_demandee_et_non_comptee`), aucun étendu.

Sondes (`mut/mutations-p5.py`, copie hors dépôt `mut-src`) : N14 (`add_steps` remis avant la garde) rouge ; N15 (garde `!b.is_empty()` retirée) rouge, toutes deux par le nouvel E2E seul ; N13 rouge (deux tests). Total des sondes : 15 (N1 à N15), toutes rouges.

Gates (journaux dans `kesh-gate-logs/s38-12a/`) : ciblé (`p5-cible.log`) : fmt, clippy workspace, nextest `kesh-api` lib + `paypal_download_e2e` + `paypal_import_e2e` + `http_client_transport` + `mcp_gardes_lexicales` : 814 exécutés, 814 passés, 5 ignorés. Complet `6584f643` (`gate-backend-15.log`, `test-fast.sh --ci`) : fmt, clippy, **6285 exécutés, 6285 passés**, 14 ignorés ; `gate-ci.sh` (`complet-p5-15.out`, `gate-ci.log`) : **6304 passés, 0 échoué, 18 ignorés**, code 0. E2E Playwright : aucun écran touché, non lancé.

### Clôture (2026-11-21)

Revue de code : P1 (Sonnet, complète) et P2 (Opus, complète), rapports `kesh-gate-logs/s38-12a/revue-p1-rapport.md` et
`revue-p2-rapport.md` ; P3 : trois LOW, corrigés (`020de8e2`, avec `b64c00bd` et `205cbf25`) ; **P4** (Haiku, ciblée)
0 CRITICAL · 0 HIGH · 1 MEDIUM · 3 LOW — le MEDIUM (étape comptée pour une moitié vide à minuit) venait de la remédiation
P3 et a été corrigé par `6584f643` ; **P5** (Haiku, ciblée sur `6584f643`) : **0 finding**. Boucle close sur P5.
Sondes P5 : référence `paypal_download_e2e` 14/14 ; coupe imbriquée à `days = 2` à minuit, jouée dans une copie : verte,
`done == total` ; mutation `Window::is_empty → false` : rouge sur les deux tests de minuit ; N14 et N15 rouges
(`mut/mutations-p5.log`). Gates du dernier commit de code `6584f643` : ciblé 814 passés ; backend complet 6285 passés,
14 ignorés ; `gate-ci.sh` 6304 passés, 0 échec, 18 ignorés ; pas d'E2E (aucun écran).
