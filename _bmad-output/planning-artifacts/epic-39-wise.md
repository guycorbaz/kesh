# Epic 39 — Compte Wise : import des relevés CSV (le téléchargement par l'API est sorti de l'epic le 22.11)

**Refonte du 2026-11-20** (décision du responsable de l'epic, sur le modèle de l'Epic 38 PayPal, page approuvée « Epic 38 PayPal,
version minimale ») : l'epic se fait en **deux phases**. La **phase 1 KISS** importe à la demande le relevé CSV que
l'on exporte de Wise, par un bouton et une route d'API, sans aucun automatisme. La phase 2 (téléchargement par l'API de Wise) **est sortie de l'epic le 2026-11-22** (§ *Téléchargement par l'API : sorti de l'epic*, branche `futur/wise-api`). L'automatisme (téléchargement périodique) vient plus tard encore, hors de cet epic. Le
cadrage du 2026-11-15 (connexions, coffre des secrets, SCA par clé RSA, relevé JSON, relevé PDF) reste plus bas comme
historique, **remplacé (2026-11-20)** ; ce qui en survit est cité nommément.

Branche `epic/39-wise` (fusion de `origin/main` au commit `9c8bc452`). Socle : la **phase 1 KISS de l'Epic 38**
(fiches 38-11a à 38-11d sur `epic/38-paypal` au 2026-11-20 : 38-11c scindée en 38-11c1 et 38-11c2 ; phase 1 de l'Epic 38 validée P3, `28623871` — lues dans le worktree de l'Epic 38 à la validation P1 des fiches 39). L'Epic 39 se développe **après** la phase 1
de l'Epic 38. Stories `drafted` ; validation P1 des fiches de la phase 1 le 2026-11-20 (décisions (1) à (4) de
l'orchestrateur appliquées, § *Décisions*), la boucle (`CLAUDE.md` § *Review Iteration Rule*) se poursuit.

## Phase 1 KISS

### Ce que fait la phase 1

Un Comptable exporte de Wise le relevé CSV de chaque solde (CHF, EUR, USD) pour une période, et l'importe dans Kesh
d'un geste. Kesh range chaque ligne au compte bancaire Kesh de ce solde, ignore les doublons, puis **propose** :

| Ligne Wise | Reconnue par | Proposition |
|---|---|---|
| Frais (`FEE-…`, « Wise Charges for : … ») | préfixe de l'identifiant | débit compte de frais du compte Wise, crédit compte Wise de la devise |
| Conversion (`BALANCE-…`, `CONVERSION_ORDER-…`) | même identifiant sur deux soldes du même compte Wise | un côté en CHF : **une** écriture, ligne CHF au montant de Wise, ligne en devise (jamais un cours de l'AFC) ; EUR↔USD : deux écritures par le compte de conversion de la société, au même CHF (une écriture ne porte qu'une devise étrangère dans Kesh, voir G6) |
| Ligne dont l'autre côté est déjà passé : virement depuis ou vers un compte bancaire de la société, « Argent reçu de Stripe » | écriture déjà rapprochée d'une transaction d'un autre compte bancaire, de mouvement égal (`find_matched_elsewhere`) | **rattachée** à cette écriture (virement entre vos comptes, Story 34-11b ; versement de l'Epic 40), jamais comptée deux fois ; versement Stripe sans écriture de versement : « à examiner » |
| Achat par carte, paiement, salaire, virement vers sa banque dont l'autre côté n'est pas encore passé | règles d'affectation (bénéficiaire ou payeur, marchand, IBAN, filtre de type ou préfixe d'identifiant, sens) | écriture de la règle, sans code TVA |
| Tout le reste | — | « à examiner » |

Les **écarts** sont listés : moitié de conversion manquante, frais sans sa ligne d'origine, trou dans le solde
courant entre deux imports ; « à examiner » : compte de frais absent, rattachement ambigu, versement Stripe sans
versement, aucune règle.

### Décisions (G1…)

- **G1 — Entrée : le CSV « relevé » de Wise, un fichier par solde.** 23 colonnes nommées en anglais (`TransferWise
  ID`, `Date`, `Date Time`, `Amount`, `Currency`, `Description`, `Payment Reference`, `Running Balance`, `Exchange
  From`, `Exchange To`, `Exchange Rate`, `Payer Name`, `Payee Name`, `Payee Account Number`, `Merchant`, `Card Last
  Four Digits`, `Card Holder Full Name`, `Attachment`, `Note`, `Total fees`, `Exchange To Amount`, `Transaction
  Type`, `Transaction Details Type`) ; une seule devise par fichier ; lignes de la plus récente à la plus ancienne ;
  dates `JJ-MM-AAAA` ; montants signés à point décimal ; **les frais sont des lignes à part** (identifiant `FEE-`
  suivi de celui de la ligne d'origine), si bien que `Amount` d'une ligne n'inclut pas ses frais et que `Total fees`
  n'est qu'une information (relevé sur les relevés 2024 du responsable de l'epic : solde courant continu ligne à ligne, frais à part
  égaux au `Total fees` de leur ligne, sans exception ; notes sans donnée personnelle dans `kesh-gate-logs/`).
  Plusieurs fichiers dans une même requête (les trois devises d'un coup) ; le zip que propose Wise n'est **pas** lu
  en phase 1 (Q-39-11). *(Amendé le 2026-11-22 : le zip est lu tel que téléchargé, et un relevé exporté sans les
  frais séparés est refusé — story 39-11e, Q-39-11 tranchée.)*
- **G2 — Un compte bancaire Kesh par compte Wise et par devise** (Epic 37, devise du compte du plan), sans IBAN
  obligatoire, réglé comme un compte PayPal de 38-11b : **libellé du compte Wise** et **compte de frais**. Réutilise
  les colonnes `provider`, `provider_account`, `fee_account_id` de 38-11b, `provider = 'wise'` ; objet `wise` du
  corps de l'API, de même forme que `paypal`, servi par la même fonction de gardes (Comptable et plus).
- **G3 — Une ligne = une transaction.** Les frais étant déjà à part, le découpage brut / frais de 38-11b (suffixe
  `:FEE`) ne sert pas à Wise. `transaction_id` = `TransferWise ID` ; doublon = même identifiant sur le même compte
  bancaire, **ignoré** et compté (une conversion porte le même identifiant sur deux soldes : deux comptes, deux
  transactions, aucun doublon).
- **G4 — Le solde courant est une somme de contrôle.** Chaque fichier doit être continu (`Running Balance` =
  précédent + `Amount`, dans l'ordre chronologique), sinon il est refusé ; l'import pose solde d'ouverture et de
  clôture (premier solde − premier montant ; dernier solde). Un import qui ne prolonge pas le précédent du même
  compte est **accepté** et signalé en écart (`gap`), par la fonction existante `previous_statement_closing`
  (`crates/kesh-db/src/repositories/bank_imports.rs:358`) ; un fichier dont le **premier jour** est couvert par un relevé du compte
  (`overlapping_statements(…, first_date, first_date)`) ne l'est pas — les doublons ne sont pas stockés ; le premier jour, et
  non toute la période, pour qu'un relevé postérieur ne masque pas l'écart du début ; l'import juge à son instant, la revue (39-11c AC-6) relit la même règle sur l'état présent des relevés : un relevé importé ou saisi depuis peut changer sa réponse, rien n'est stocké.
- **G5 — Règles : critères existants, filtres de 38-11b, rien de neuf pour Wise** (décision (3) de l'orchestrateur).
  Contrepartie = `Merchant` pour une carte, `Payee Name` pour un débit, `Payer Name` pour un crédit
  (`counterparty_contains`, `counterparty_exact`) ; IBAN du bénéficiaire (`iban_exact`, colonne `counterparty_iban`
  existante) ; **préfixe de l'identifiant** (`TRANSFER`, `CARD`, `BALANCE`, `CONVERSION_ORDER`, `FEE-TRANSFER`…)
  dans `bank_transactions.provider_type`, lu par le **filtre facultatif** `provider_type` d'une règle (colonnes de
  38-11b, servies par 38-11c), avec le filtre `direction` ; `counterparty_email` sans objet pour Wise. Aucun type de
  critère neuf, aucun `match_type` neuf.
- **G6 — Conversions : règle commune aux trois prestataires** (décision (1) de l'orchestrateur, Q-39-12 tranchée ;
  code partagé avec 38-11c). **Un côté en CHF** → **une** écriture : ligne CHF au montant réel (celui de Wise), ligne
  en devise au montant en devise, cours déduit du CHF (`rate_from_chf`, `crates/kesh-core/src/accounting/fx.rs:266`),
  les deux transactions rattachées à cette écriture — ce qui étend « l'autre côté d'un virement »
  (`other_side_predicate`, `reconciliation_link.rs:698-709`) aux deux transactions de devises différentes d'une
  conversion, pour que l'annulation d'un côté ne retire que son lien (39-11c AC-3). **Deux devises étrangères** :
  Kesh refuse deux devises étrangères dans une écriture (`FX_MULTIPLE_CURRENCIES`,
  `crates/kesh-db/src/repositories/journal_entry_conversions.rs:1-9`), d'où **deux écritures par le compte de
  conversion** de la société (`companies.fx_transit_account_id`, `crates/kesh-db/src/entities/company.rs:205`), par
  `create_conversion_in_tx` (`journal_entry_conversions.rs:261`, publique par 38-11c2) — même CHF des deux côtés, CHF =
  côté source au cours habituel de Kesh (cours mensuel retenu de la société). Les frais de conversion sont leur propre
  ligne `FEE-BALANCE-…` (G3), au compte de frais.
- **G7 — Virements entre ses comptes : fonction existante, sans heuristique propre.** Une écriture de virement porte
  deux transactions, une de chaque compte bancaire (`reconciliation_link.rs:21-35`, `transfer_other_side` `:773`). Si
  l'autre côté est déjà passé, la transaction Wise se **propose** au rattachement à cette écriture
  (`find_matched_elsewhere` `:896`, candidate unique à 5 jours au plus ; `link_entry_in_tx` `:1156`, route `POST
  …/link-entry`, `lib.rs:1046`) ; sinon elle s'affecte par une **règle** (IBAN ou nom → compte du plan de la banque)
  et l'autre côté s'y rattachera à son import. Une **proposition**, jamais une écriture d'office (§ *Un appariement
  automatique propose*). Q-39-15 tranchée ainsi.
- **G8 — Versements Stripe : rattachés comme un virement entre deux comptes propres, jamais comptés** (décision (4)
  de l'orchestrateur). L'écriture du versement de l'Epic 40 porte la transaction du compte Stripe ; le crédit Wise dont
  le payeur contient « stripe » (sans casse) en est **l'autre côté** (`transfer_other_side`), par le même chemin que
  G7 ; aucune écriture de vente, **jamais de règle** sur ces lignes ; sans écriture de versement, « à examiner »
  (motif `stripe_payout_unmatched`). Seule la revue Wise (39-11c) propose ce rattachement ; 40-7c ne propose aucune
  réception et liste l'écart `payout_without_receipt` tant qu'il n'est pas accepté.
- **G9 — Aucun code TVA en phase 1**, comme PayPal (38-11c).
- **G10 — Factorisation avec l'Epic 38 (DRY).** Ce qui est commun aux deux prestataires vit **une fois** : lecture
  d'un en-tête CSV par noms et forme fermée des montants (`kesh-import`, `csv/by_name.rs`), réglages prestataire d'un
  compte bancaire et leurs gardes, cœur de l'import ventilé par devise (`with_account_locks`, doublons par
  identifiant, empreinte, compte rendu), moteur de revue et d'acceptation de 38-11c (écritures de frais, de règle et
  de conversion, rattachement, *Pattern batch*), écrans de 38-11d. Les fiches 39 nomment chaque point de partage ; si la fiche 38 correspondante est
  développée sous une forme propre à PayPal (`paypal` dans les noms), la fiche 39 la **généralise** au lieu de la
  recopier, sans changer le comportement de PayPal (ses tests verts sans retouche de leurs attentes).
- **G11 — Aucune migration** (décision (2) de l'orchestrateur, Q-39-10 tranchée) : la migration de 38-11b pose
  `chk_bank_accounts_provider` avec `BINARY provider IN ('paypal', 'wise', 'stripe')` (fiche 38-11b validée P1). Le
  repli (migration propre à Wise) est **retiré** : si 38-11b n'admettait pas `'wise'`, l'Epic 38 corrige sa migration
  avant toute publication (P8).

### Ce que la phase 1 attend de l'Epic 38 (contrat)

Vérifié au code de `origin/main` (commit `fd04099f`, revérifié à la validation P1) : **la réconciliation de Kesh
n'écrit aujourd'hui qu'en CHF.**
Règles : refus `RECONCILIATION_CURRENCY_MISMATCH` si la transaction n'est pas en CHF
(`crates/kesh-api/src/routes/reconciliation.rs:4385-4396`) ; écriture manuelle à lignes sans devise
(`crates/kesh-reconciliation/src/manual.rs:116-118`, « la réconciliation n'écrit qu'en CHF ») ; rattachement à une
écriture réservé au CHF (`reconciliation_link.rs:535-537`, `transaction_can_be_linked_to_entry`) ; un compte bancaire
ne se lie pas à un compte du plan en devise (C-37-9, `crates/kesh-api/src/routes/bank_accounts.rs:435`). PayPal
(EUR, USD) a besoin de lever tout cela ; **38-11b** lève C-37-9 pour le compte prestataire ; **38-11c1** rend le
rattachement à une écriture valable en devise (sa AC-1) et porte le prédicat de l'autre côté (AC-1b) ; **38-11c2** et écrit en devise **par ses propres routes** (`/paypal/review`,
`/paypal/accept`), en laissant `get_proposals` et `accept_one_rule` en CHF. La phase 1 de l'Epic 39 **ne développe
pas sa propre version** de ces levées : elle branche des routes `wise` minces sur le moteur de 38-11c2. Servi générique ; 39-11c ne le généralise (G10) que si le développement de 38-11c2 s'en écarte.

**Demandes à l'Epic 38** (validation P1 du 2026-11-20 ; **servies** par la phase 1 de l'Epic 38, validée P3 le 2026-11-20) : codes d'erreur génériques
(`PROVIDER_*`, et non `PAYPAL_*`) pour les gardes, l'import et la proposition périmée ; conversion à **une** écriture
quand un côté est en CHF, avec l'alternative du prédicat de l'autre côté (décision (1) ; servi par 38-11c1 AC-1b et 38-11c2 AC-8) ; proposition de rattachement générique (`link_entry`, qui remplace le `link_sale`) ; moteur de revue et
d'acceptation paramétré par le prestataire ; gardes qui lisent `provider IS NOT NULL`, jamais `= 'paypal'`.

### Stories de la phase 1

| Story | Objet | Modules (≤ 5) | Dépend de | Migration |
|---|---|---|---|---|
| 39-11a | lecteur du CSV Wise | `kesh-import` | 38-11a | aucune |
| 39-11b | réglages du compte Wise et import ventilé par devise | `kesh-api`, `kesh-reconciliation` (appel), `kesh-import` (lecture), `kesh-i18n` | 39-11a, 38-11b | aucune |
| 39-11c | rapprochement et propositions Wise, écarts | `kesh-reconciliation`, `kesh-api`, `kesh-db` (prédicat de l'autre côté, si 38-11c1 ne le porte pas), `kesh-i18n` | 39-11b, 38-11c1, 38-11c2 | aucune |
| 39-11d | écrans, manuel, E2E | `frontend`, `kesh-i18n`, `docs/manual` (`.tex`), E2E | 39-11a à 39-11c, 38-11d | aucune |
| 39-11e | zip de Wise tel que téléchargé, refus d'un relevé sans les frais séparés | `kesh-import`, `kesh-api`, `kesh-i18n`, docs (`api-external.md`, `.tex`) | 39-11a2, 39-11b | aucune |

**Ordre** : 39-11a → 39-11b → 39-11c → 39-11d. 39-11a peut s'écrire dès que 38-11a est fixée. 39-11e (2026-11-22)
se développe dès sa validation, avant 39-11d, qui branche le zip à l'écran.

## Téléchargement par l'API : sorti de l'epic le 22.11

Le téléchargement des mouvements Wise par l'API (ex-phase 2 : client, activités en lignes, route et import,
doublons CSV, conversions partielles, écran et manuels) **sort de l'Epic 39** sur décision du responsable de
l'epic, le 2026-11-22 : l'Epic 39 se limite à l'import du CSV. Raison : il attend le support Wise ; aucune voie
API exacte n'est établie pour ce compte (recherche hors dépôt). Tout le travail est conservé sur la **branche
locale `futur/wise-api`** (partie de `7c2af959`, avant le retrait), pour un epic futur : client Wise (la story
39-12a1, `done`, fusionnée dans l'epic puis retirée par un `revert`) et fiches 39-12a2, 39-12a2b, 39-12a3, 39-12a3b,
39-12a4a1, 39-12a4a2, 39-12a4b et 39-12b, telles quelles. Les fiches ne sont plus dans `epic/39-wise`.

Le socle commun ci-dessous (S1 à S9) reste : il est partagé avec les Epics 38 et 40.

## Socle commun de la phase 2 (Epics 38, 39, 40) — contrat S1 à S9

*Texte identique au fichier de l'Epic 40 ; décidé à la validation P1 des phases 2 de Wise et de Stripe (2026-11-20, rapport
`39-12-P1.md`), amendé à la validation P2 du même jour (S1 : vide = absente, séries 1 à 3 au compose ; S2 à S4 : `max_days` par prestataire, 409 avec `jobId`, un
seul nom `details`, clé d'API en 403 `API_KEY_OPERATION_FORBIDDEN`, registre ; S6 : `ProviderLine` extrait par 38-12a ; rapports `39-12-P2.md`, `40-8-P2.md` ; S1, S2, S9 amendés le 2026-11-20 par la voie activités de Wise : clé SCA retirée, Wise 366) pour que
les trois prestataires partagent **un** code. **Écrit par la première fiche de phase 2 développée**
(38-12a, normalement), **générique dès le départ** (C-40-K6) ; les deux autres l'appellent. Ce que chaque fiche garde
en propre : sa conversion, son jeu d'étapes, ses codes `<PRESTATAIRE>_*` propres au format.*

- **S1 — Configuration.** Un lecteur unique `provider_env` (kesh-api) lit `KESH_<P>_<n>_ACCOUNT` (libellé = `provider_account`
  de 38-11b), les champs du prestataire (`_CLIENT_ID`/`_SECRET`, `_TOKEN`, `_KEY`…), des champs facultatifs, `KESH_<P>_BASE_URL`
  et `KESH_<P>_PARALLEL` (1 à 6 strict, défaut 1) ; `n` de 1 à 9 ; une valeur **vide vaut absente** (`env_nonempty`, `config.rs` ; les
  fichiers compose transmettent `${NOM:-}`, vide quand le `.env` ne la pose pas) : une installation sans le prestataire démarre ;
  seules une série **partielle**, un libellé en double (sans casse) ou une valeur `<…>` font refuser le démarrage, message qui
  nomme la variable ; **les séries 1 à 3 de chaque prestataire, `KESH_<P>_BASE_URL` et `KESH_<P>_PARALLEL` sont transmises par
  `docker-compose.yml`, `docker-compose.prod.yml` et décrites dans `.env.example`** (patron des `KESH_SMTP_*`,
  `docker-compose.yml:117-122`, `.env.example:302-340`) — sans quoi le `.env` n'atteint pas le conteneur —, par la story de
  phase 2 de chaque prestataire ; **la garde `crates/kesh-api/tests/configuration_transmise.rs` (#550)** exige qu'une variable soit lue
  si et seulement si elle est transmise, une ligne `.env.example` par variable lue et des noms `KESH_…` connus, et les noms construits
  par `format!` lui échappent : **38-12a y ajoute une famille indexée générique par prestataire** (séries 1 à 3 exigées, 4 à 9 admises
  absentes avec raison, lectures de `provider_env` inscrites en forme exacte), Wise et Stripe y **inscrivent leur famille** ; les noms **sans numéro** `KESH_<P>_BASE_URL` et `KESH_<P>_PARALLEL` (construits par `format!`, et lus par le parcours `std::env::vars()`) sont inscrits en forme exacte à `EMPLACEMENTS_AUTORISES`, leurs lignes de `.env.example` sont commentées ou vides, jamais `<…>`, sans relâcher aucune liste ; le manuel d'administration les porte dans **un tableau neuf du §
  `sec:env-vars`** (modèle `:840-862`) **et** dans le tableau de la section du prestataire — **jamais** le tableau SMTP (`:1155`) —,
  et dit comment aller au-delà de trois comptes (lignes à ajouter au compose) ; champs privés, `Debug` masqué ; un contrôle propre au prestataire
  s'y branche (clé Stripe `rk_` ; Wise n'en a pas). Le libellé est **propre à l'installation**, non à la société.
- **S2 — Mécanique de téléchargement** (`provider_download`) : table des tâches en mémoire (une par prestataire, société et
  libellé, purgée une heure après la fin), `windows(from, to, max_days, now)` pure (jours de Zurich ; **`max_days` par prestataire** — PayPal 30, Wise 366 (aucune borne de période à l'API des activités de Wise : la fenêtre ne découpe que la progression), Stripe
  sans fenêtre (curseurs, aucune borne de période) — : quand la limite du prestataire se compte en heures, sur des instants
  UTC, ou n'a pas d'unité dite, on prend **un jour de moins** qu'elle, pour couvrir le jour de 25 heures du passage à
  l'heure d'hiver ; fin ramenée à `now`), **politique de reprise unique**
  (429 et 5xx : `Retry-After` entier borné à 120 s, sinon 5, 10, 20, 40, 60 s ; six essais par requête ; **pause commune** à
  tous les fils, sans réduire le parallélisme ; aucune pause fixe entre requêtes), permis de parallélisme, horloge et attente
  injectables. Chaque prestataire écrit sa propre boucle d'étapes.
- **S3 — Routes** `POST /api/v1/<prestataire>-downloads` (`paypal`, `wise`, `stripe`) : Comptable et plus, **sessions seules (clés
  d'API non admises : un secret de l'Administrateur sort)** — refusées sur le `POST` et le `GET`, dans le gestionnaire, avant
  tout, en 403 `API_KEY_OPERATION_FORBIDDEN` (patron de `bank_imports::delete`) ; le `POST` est inscrit à `LIB_ROUTES`
  (`audit_route_registry.rs`) en `Traced, SansEcritureAuJournal`, et la story qui l'ajoute relève **dans le même commit** le
  compte de `the_registry_partition_is_what_the_story_declares` (**six** `assert_eq!` : cinq valeurs littérales — `LIB_ROUTES.len()` = 157, `traced` = 149, `exempt` et `no_matter` = 4, `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` = 161 — et l'égalité de la partition) et **les deux phrases** du manuel d'administration lues par `le_manuel_dit_le_compte_du_registre` —
  « N des M routes à verbe mutant » **et** « N + E + R = M », même paragraphe (`.tex`) ; 202 `{ jobId }` ; `GET …/{jobId}` (société de
  l'appelant, sinon 404) → `{ state: running | done | failed, steps: { done, total: number | null }, pausedUntilSeconds,
  result, error: { code, message, details? } }` — **un seul nom, `details`**, dans l'état d'une tâche comme dans un refus
  HTTP.
- **S4 — Codes génériques** (38-11b a déjà posé `PROVIDER_ACCOUNT_NOT_FOUND`…) : `PROVIDER_API_NOT_CONFIGURED` (422),
  `PROVIDER_DOWNLOAD_RUNNING` (409, `details: { provider, jobId }`, pour que l'écran reprenne le suivi), `PROVIDER_PERIOD_INVALID` (400, `details.reason`), et, dans `error.code` d'une tâche,
  `PROVIDER_AUTH_FAILED`, `PROVIDER_PERMISSION_DENIED`, `PROVIDER_RATE_LIMITED`, `PROVIDER_UNAVAILABLE`,
  `PROVIDER_RESPONSE_INVALID`, `PROVIDER_API_ERROR` ; `details.provider` nomme le prestataire ; jamais le texte du prestataire.
  **La panique d'une tâche** donne `INTERNAL_ERROR`, non `PROVIDER_API_ERROR` (qui accuserait le prestataire).
- **S5 — Transport.** Un constructeur de client HTTP commun (rustls, aucune redirection, délais, `User-Agent`), extrait de
  `exchange_rates/source.rs:116-130` ; **`reqwest` n'a pas la fonctionnalité `json` en dépendance normale** (`Cargo.toml:112`) :
  `.bytes()` puis `serde_json`. Une adresse de base par prestataire (`https` ; `http` admis vers le seul bouclage), et **un seul
  schéma de test** : un petit serveur `axum` dans le processus (`127.0.0.1:0`) joue le prestataire sur cette adresse — pas de
  trait de source simulée.
- **S6 — Import.** Tout est lu **avant** toute écriture ; les lignes passent par le **cœur commun** d'import (lignes neutres
  `ProviderLine` : fonction pure de conversion + cœur d'écriture `provider_imports.rs`), **une** transaction de base ; **aucune
  fiche de phase 1 ne le factorise** (38-11b travaille sur `PaypalRow`) : **38-12a l'extrait** — déplacement suivi au `REGISTRE`
  de `bank_account_closed_inventory.rs` (couple fichier, fonction) — ; 39-11b et 40-7b l'appellent s'il existe, sinon
  l'extraient eux-mêmes de la même façon, et la phase 2 de Wise et de Stripe ne fait que l'appeler ; l'empreinte d'un téléchargement porte sur les identifiants triés et le
  refus `BANK_IMPORT_DUPLICATE_FILE` **ne s'applique pas** (relancer rend « 0 insérée »).
- **S7 — Écran.** Un dialogue générique (38-12b), paramétré par prestataire ; `providerApi: { configured, mode?, … }` sur le
  compte bancaire.
- **S8 — E2E.** Un seul serveur de prestataire simulé pour Playwright, port `KESH_E2E_PROVIDER_MOCK_PORT`.
- **S9 — Secrets.** Un test au marqueur commun : aucun secret (jeton, clé, secret client) dans un journal, une réponse, une
  erreur ou une trace d'audit.

## Registre (stories de l'epic)

| Clé | Statut | Note |
|---|---|---|
| `39-0-kickoff` | done | cadrage du 2026-11-15 (historique) |
| `39-1-client-wise` | superseded | remplacée par le téléchargement par l'API, sorti de l'epic le 22.11 (branche `futur/wise-api`) |
| `39-2-authentification-forte` | superseded | le défi SCA constaté à la recette 1 (2026-11-20) ; reprise un temps par le client Wise, puis **abandonnée** avec la voie relevé + SCA (H5, 2026-11-20) |
| `39-3-releve-json-mouvements` | superseded | remplacée par 39-11a (lecteur) ; le téléchargement par les activités est sorti de l'epic (branche `futur/wise-api`) |
| `39-4-classement-des-types-wise` | superseded | remplacée par 39-11c (préfixes d'identifiant, règles) |
| `39-5-releve-pdf` | deferred | hors de l'epic |
| `39-6-ecrans-et-manuels` | superseded | remplacée par 39-11d (la part API est sortie de l'epic : branche `futur/wise-api`) |
| `39-13-solde-dette`, `39-14-raccord-epic-38` | backlog | story de solde et raccord avec l'Epic 38 |
| `39-12*` (téléchargement par l'API) | sorties | retirées de l'epic le 22.11 ; la 39-12a1 était `done` ; branche `futur/wise-api` |
| `39-11a` à `39-11d` | ready-for-dev | fiches du 2026-11-20, phase 1 validée |
| `39-11e-wise-zip-et-format-des-frais` | drafted | 2026-11-22, besoin du responsable de l'epic (comptes 2025) ; validation P1 à lancer |

## Questions ouvertes (2026-11-20)

- **Q-39-10 — Contrat avec l'Epic 38** : `'wise'` et `'stripe'` admis dès la contrainte de 38-11b — **tranché** (décision
  (2), fiche 38-11b validée P1). Reste demandé (§ *Ce que la phase 1 attend de l'Epic 38*) : codes `PROVIDER_*`,
  conversion à une écriture, rattachement et moteur génériques. L'API porte un objet `wise` de même forme que `paypal`
  (plus simple qu'un objet `provider` générique, PayPal inchangé).
- **Q-39-11 — Zip** : lire aussi le zip des trois relevés qu'exporte Wise ? (non en phase 1 ; trois fichiers d'un
  coup suffisent.) **Tranchée le 2026-11-22** par le responsable de l'epic : oui, tel que téléchargé, par la route
  et l'écran — story 39-11e (API), 39-11d (écran).
- **Q-39-12 — CHF d'une conversion EUR↔USD** : **tranchée** (décision (1)) — CHF = côté source au cours habituel de
  Kesh (cours mensuel retenu de la société), même CHF aux deux écritures.
- **Q-39-13 — CSV de l'API** : **sans objet** (2026-11-20) — le relevé de l'API n'est pas servi (H5) ; les lignes viennent des activités (H2).
- **Q-39-14 — Relevés au jeton personnel suisse et SCA** : **close** (2026-11-20) — refusés par règle, signature non évaluée (H4, H5) ; voie abandonnée.
- **Q-39-15 — Virement vers son propre compte reconnu par un numéro qui n'est pas un IBAN** (relevés 2024 : des
  numéros de compte non IBAN) : faut-il un réglage « numéros de mes comptes » ou une règle suffit-elle ? **Tranchée
  par G7** : une règle d'affectation vers le compte du plan de la banque ; l'autre côté s'y rattache à son import.
- **Q-39-16 — Délai du rejeu SCA** : **close** (2026-11-20) — sans objet, plus aucun rejeu signé (H5). (Le délai de 120 s du rejeu dans `appel-reel.sh` reste sans emploi pour Wise.)
- **Q-39-17 — Recette sur 2024 (orchestrateur, 2026-11-20 ; elle n'attend pas le responsable de l'epic)** : avant toute ligne de la phase 2, comparer 2024 de l'API (≈ 45 appels, réponses brutes sauvées hors du dépôt) au CSV 2024 du site, en local, et trancher l'horodatage de la date, l'identifiant d'une conversion automatique, le champ de solde (`amount` ou `cashAmount`), la règle du sens, le type de détail par type d'activité, la `reference` des transferts et le devis d'un virement sans frais (39-12a1 AC-0 a) ; puis, après 39-12a2, rejouer la conversion de Kesh sur 2024 comme au 2024-12-31, ligne à ligne, solde courant compris (AC-0 b). **Un mois 2025** se compare plus tard, quand le responsable de l'epic aura son CSV 2025, avec les soldes réels (AC-0 c) : non bloquant. Constats (métadonnées) à écrire ici. **Close (2026-11-20)** : AC-0 (a) faite, décisions au paragraphe « Décisions de la recette réelle AC-0 (a) » ; restent AC-0 (b), AC-0 (c) (2025), les activités non terminées sur `amount` et le décalage UTC+1 à revérifier avec un CSV 2025.

## Change Log (refonte)

- 2026-11-22 — **Fiche 39-11e** (besoin du responsable de l'epic pour les comptes 2025) : le zip de Wise s'importe
  tel quel par `POST /api/v1/wise-imports` (entrées `.csv` à la racine, bornes, tout ou rien), et un relevé exporté
  sans « frais indiqués séparément » est refusé (`WISE_CSV_FEES_NOT_SEPARATED`, critère par fichier mesuré sur les
  relevés réels) ; G1 amendé, Q-39-11 tranchée ; 39-11d amendée (bouton et manuel). `drafted`, validation P1 à lancer.

- 2026-11-22 — **Téléchargement par l'API sorti de l'epic** (décision du responsable de l'epic) : code de la 39-12a1 retiré par un `revert` de sa fusion, fiches 39-12* retirées, tout conservé sur la branche locale `futur/wise-api` ; l'epic garde 39-0, 39-11a, 39-11a2, 39-11b, 39-11c, 39-11d, 39-13, 39-14.

- 2026-11-22 — **Téléchargement par l'API mis en attente** par le responsable de l'epic (inutile pour la comptabilité 2025 ; le support Wise est à contacter d'abord) : 39-12a2, 39-12a2b, 39-12a3, 39-12a3b, 39-12a4a1, 39-12a4a2, 39-12a4b et 39-12b passent `deferred` ; validation P3 (1 H, 5 M, 9 L) non remédiée ; la décision du 22.11 (règle du centime) est écrite dans 39-12a4a2, sans propagation.
- 2026-11-21 — Remédiation de la validation P1 des fiches 39-12a2 à 39-12a4b et 39-12b (Claude Sonnet 5.5, rapport `kesh-gate-logs/s39-12a-specs/validation-p1-rapport.md` : 0 C, 2 H, 15 M, 18 L) : points de contrôle placés par identifiant d'import (H1) ; virement de même devise dans 39-12a2 (H2) ; fiche neuve 39-12a2b, approvisionnement et prélèvement de 2026 (M1) ; 39-12a4a scindée en 39-12a4a1 (continuité) et 39-12a4a2 (estimation au cours recalibrée, M3 à M6) ; **décision ouverte** de 39-12a4a2, « le cours confirme ou fixe une part » (M10), à soumettre au responsable de l'epic ; dépendances à 39-14 et 39-11a2, lecture réutilisée (M13) ; 39-12b amendée (M14). Fiches 39-12a2, 39-12a3 et 39-12b repassées `drafted`. Ordre de développement : 39-12a2, 39-12a2b, 39-12a3, 39-12a3b, 39-12a4a1, 39-12a4a2, 39-12a4b, 39-12b. Validation P2 à lancer.
- 2026-11-21 — Amendements du 21.11 de 39-12a2 et 39-12a3 (recherche `kesh-gate-logs/epic-39-40-kiss/wise-conversion-partielle-recherche.md`, décisions (a) et (b) du 21.11) : partielle refusée d'abord (décision (a)), puis répartie par continuité des soldes (suivi « essayer encore ») dans les stories neuves 39-12a4a et 39-12a4b ; doublons R1 à R3 sortis de 39-12a3 en 39-12a3b (règle des petites stories). Validation P1 à lancer.
- 2026-11-20 — **décisions de la recette AC-0 (a), 2026-11-20** (source `kesh-gate-logs/epic-39-40-kiss/wise-ac0-decisions.md`) : Q-39-17 close pour 2024 ; le paragraphe « Ce qui reste à confirmer » devient « Décisions de la recette réelle AC-0 (a) » avec les restes ouverts (2025, activités non terminées sur `amount`, décalage UTC+1) ; l'identifiant `AUTO_CONVERSION-<id>` passe de provisoire à définitif ; fiches 39-12a1, 39-12a2, 39-12a3 et 39-12b alignées.

- 2026-11-20 — Validation P5 ciblée sur le commit `06a8f336` (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P5-ciblee.md`) : 0 C, 0 H, 0 M, 1 L, remédié dans 39-12a1, 39-12a2 et 39-12b (virement remboursé : recette et suffixe d'identifiant). Socle S1 à S9 inchangé.
- 2026-11-20 — Validation P4 ciblée sur le commit `a98da37a` (Claude Opus 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P4-ciblee.md`) : 0 C, 0 H, 1 M, 2 L, remédiés dans 39-12a1, 39-12a2 et 39-12b. Socle S1 à S9 inchangé.
- 2026-11-20 — Validation P3 de la voie activités (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P3.md`) : 0 C, 0 H, 3 M, 4 L, remédiés dans les fiches 39-12a1 à 39-12b. Le renvoi de la P2 (« deux extractions du cœur d'import divergent ») est **périmé** : le cœur de 40-7b reprend celui de 38-12a (relevé aux worktrees). Socle S1 à S9 inchangé.
- 2026-11-20 — Validation P2 de la voie activités (Claude Opus 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P2.md`) : 0 C, 1 H, 2 M, 9 L, remédiés dans les fiches 39-12a1 à 39-12b ; ici, « deux lectures » de `kesh-db` au tableau (résidu de la P1). Socle S1 à S9 inchangé. Renvoi : deux extractions du cœur d'import divergent (38-12a, 40-7b), à réunir à l'intégration.
- 2026-11-20 — Validation P1 de la voie activités (Claude Sonnet 5.5, contexte frais ; rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-v2-P1.md`) : 0 C, 2 H, 10 M, 7 L, remédiés dans les fiches 39-12a1 à 39-12b ; ici, Q-39-17 et le paragraphe « À confirmer » passent sur 2024 (mois 2025 plus tard, non bloquant). Socle S1 à S9 inchangé et toujours identique à `epic-40-stripe.md`.


- 2026-11-20 — **Phase 2 réécrite sur la voie activités** (recherche du 2026-11-20, `data-not-committed/wise-recherche.md`,
  métadonnées) : jeton personnel lecture seule, sans SCA — activités paginées, transferts, devis des virements sortants,
  soldes actuels ; identifiants du CSV, montants en texte par forme fermée, frais de conversion inclus, solde reconstruit et
  contrôlé. H1, H2, H3, H5 récrites, H6 (Enable Banking, option future hors périmètre) ajoutée ; H4 gardée comme historique ;
  `KESH_WISE_<n>_SCA_KEY_FILE` retirée ; Q-39-13, Q-39-14, Q-39-16 closes, Q-39-17 ouverte (recette d'un mois 2025) ; fiches
  39-12a1 (renommée), 39-12a2 (neuve), 39-12a3 (ex-39-12a2), 39-12b `drafted` ; socle S1, S2 (`max_days` Wise = 366 : aucune
  borne de période documentée pour les activités), S9 amendés à l'identique dans `epic-40-stripe.md`.

- 2026-11-20 — Refonte en phase 1 KISS et phase 2 (décision du responsable de l'epic du 2026-11-20) : six fiches `drafted` (39-11a à
  39-11d, 39-12a, 39-12b) ; anciennes clés 39-1, 39-3, 39-4, 39-6 `superseded`, 39-2 et 39-5 `deferred` ; fusion de
  `origin/main` (arbre de `main` repris, fiches et blocs des Epics 38 et 39 gardés). Cadrage du 2026-11-15 conservé
  ci-dessous, remplacé.
- 2026-11-20 — Validation P1 des fiches 39-11a à 39-11d (Claude Opus 5.5, contexte frais ; rapports
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11x-…-P1.md`) : décisions (1) à (4) de l'orchestrateur appliquées
  (G5, G6, G7, G8, G11 ; Q-39-10, Q-39-12, Q-39-15) ; migration de repli retirée ; propositions par les routes de
  38-11c, non par le rapprochement CHF ; virements internes et versements Stripe par l'autre côté déjà passé
  (`find_matched_elsewhere`) puis les règles ; continuité par `previous_statement_closing` ; demandes à l'Epic 38.
- 2026-11-20 — Validation P3 ciblée du commit `02ed58c6` (Claude Opus 5.5, contexte frais ; rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11-P3-ciblee.md`) : G4, continuité jugée sur le **premier jour** du fichier
  (`overlapping_statements(…, first_date, first_date)`), non sur toute sa période ; à la revue, l'état présent des relevés (un relevé importé après coup peut changer la réponse, comme en 39-11c AC-6). Validation P4 ciblée (Claude Haiku 5.5, passe ciblée, contexte frais) : « réponde comme l'import » retiré, il était faux pour un relevé importé après coup.
  Rectificatif du même jour : G8, la 39-11c seule propose le rattachement de la réception Wise au versement (40-7c ne
  propose plus de réception depuis sa P2, écart `payout_without_receipt`).

- 2026-11-20 — Validation P2 de la phase 2 (Claude Opus 5.5, contexte frais ; rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-12-P2.md`) : 0 C, 0 H, 3 M, 7 L, remédiés, plus trois décisions de l'orchestrateur sur le socle. H1 (clé SCA exigée), H2
  (`max_days` = 468), H5 (signature par `jsonwebtoken` RS256, déjà dépendance directe ; RUSTSEC-2023-0071 relu ; recette 2
  **préalable** au développement) ; socle S2 à S4 amendés **à l'identique** dans le fichier de l'Epic 40 sur décision de
  l'orchestrateur (`max_days` par prestataire, un jour de moins quand la limite se compte en heures ; 409
  `details: { provider, jobId }` ; un seul nom `details`) et sur la P2 (clé d'API en 403 `API_KEY_OPERATION_FORBIDDEN` sur
  les deux routes ; registre et phrase du manuel relevés par la story qui ajoute le `POST`, comme 38-12a).

---
- 2026-11-20 — alignement sur la phase 1 de l'Epic 38 (Claude Sonnet 5.5, contexte frais ; fiches 38-11c1, 38-11c2, 38-11d et rapport de validation P3 de `28623871` lus) : 38-11c scindée en 38-11c1 et 38-11c2 (dépendances, tableau des stories, repli du prédicat, `link_sale` → `link_entry`, demandes à l'Epic 38 déclarées servies). Rapport `kesh-gate-logs/epic-39-40-kiss/validation/alignement-38-P3.md`.

## Historique — cadrage du 2026-11-15, remplacé (2026-11-20)

*Tout ce qui suit est caduc, sauf ce que les sections ci-dessus citent. Titres abaissés d'un niveau, texte inchangé.*

⏸ **Les fiches de l'Epic 39 attendent le test du responsable de l'epic sur l'API des relevés Wise (Q-39-1, recette de 39-0)** : aucune
fiche ne s'écrit ni ne se développe avant ce constat ; les règles de travail à plusieurs (§ *Alignement*, plus bas)
s'appliqueront alors à chaque fiche.

**Cadrage : 2026-11-15** (branche `epic/39-wise`). Demande du responsable de l'epic (contributeur), décisions B1–B28 de
`epics-38-40-cadrage.md`, qui **priment** sur ce fichier ; pour Wise : B1, B3, B4, B9, **B10** (relevé PDF par l'API,
SCA), B12, B14. Socle : l'**Epic 38** (`epic-38-paypal.md`, validé, seize fiches `ready-for-dev`), dont les décisions
E*, O*, D-P1-*, D-P2-* valent ici sans être répétées. Kickoff (décomptes, validation de ce fichier, **recette de
faisabilité**) : `39-0-kickoff`.

**Aucune migration** : l'Epic 38 a promis que 39 n'ajoute que son prestataire, sans migration breaking, grâce au
décodage tolérant (E4) — `provider` au format seul, `source_format = 'PROVIDER_API'`, `movement_kind`,
`review_reason`, `match_type` et `bank_account_documents.source` à domaine ouvert. Ce fichier le confirme story par
story (F15) ; une fiche qui découvre un besoin de DDL s'arrête et le dit (C-39-0).

**Numéro : 39** (35 et 36 réservés ailleurs ; 40 Stripe, provisoire, B25). Une branche par story, fusionnée
localement dans `epic/39-wise`, puis **une PR** vers le dépôt de Guy, ouverte depuis le dépôt du contributeur, **après celles des Epics 37 et
38**. Pas d'issue GitHub à ce stade (le responsable de l'epic n'a qu'un accès en lecture) ; issues et PR à la fin.

**Release : un epic = une release ; 7 stories** (39-0 à 39-6, kickoff et story de solde comprises). ⚠️ **Cela dépasse la
borne de six** des epics de correction ou d'amélioration ; l'Epic 39 ajoute une **fonctionnalité nouvelle** (un
prestataire), ce qui permet de dépasser **à condition de le dire au kickoff** — c'est dit ici, à confirmer par Guy à
39-0. Repli si Guy préfère la borne : fusionner 39-3 et 39-4 (relevé JSON et classement, tous deux dans
`kesh-providers`), au prix d'une story plus longue, contraire à « pas de story trop longue » ; ou 39-1 et 39-2
(client et SCA), au prix d'une dépendance cryptographique neuve dans la même fiche. Aucune des deux n'est proposée
d'office.

**Cible** : la release qui suit celle de l'Epic 38, ou la même (aucune migration, aucun relèvement de
`kesh_version_min_required`) — à Guy (Q-39-9).

⛔ **Faisabilité non établie** : la documentation Wise consultée restreint les **jetons personnels** — les relevés de
solde ne seraient pas servis par l'API hors des comptes établis aux États-Unis, au Canada, en Australie, en
Nouvelle-Zélande, à Singapour et en Malaisie (§ *Sources*, Q-39-1). Le compte du responsable de l'epic est suisse. **Aucune fiche ne se
développe avant que la recette de 39-0 l'ait tranché** sur le compte réel du responsable de l'epic.

### Intention

Le compte Wise du responsable de l'epic (B1 : **un** compte, **trois soldes** CHF, EUR, USD, et ses **cartes**, qui débitent ces
soldes) se tient dans Kesh **sans saisie**, comme les deux comptes PayPal de l'Epic 38 : Kesh télécharge chaque jour
(ou à la demande, B9) les mouvements de chaque solde — achats par carte, frais, conversions, virements envoyés et
reçus, dépôts, retraits —, les classe, et les **propose** à l'imputation par les règles d'affectation (frais au
sous-compte de frais de Wise et de la devise, B3 ; conversions par le compte de transit ou en une écriture, B4) pour
une acceptation en lot. Rien n'est écrit d'office. **Les relevés de compte** (B10) se **génèrent et se téléchargent
par l'API** — PDF rangé comme document du compte et de l'exercice (38-10), solde de clôture rempli depuis le relevé,
qui sert de somme de contrôle (B20). L'authentification forte (SCA) de Wise se fait par une **clé privée RSA** gardée
au coffre de 38-1.

### Sources (docs.wise.com, consultées le 2026-11-15)

- Relevé de solde : <https://docs.wise.com/api-reference/balance-statement> et
  <https://docs.wise.com/api-reference/balance-statement/balancestatementget.md> —
  `GET /profiles/{profileId}/balance-statements/{balanceId}/statement.{json|csv|pdf|xlsx|xml|mt940|qif}` ; requête
  `currency` (exigé), `intervalStart`, `intervalEnd` (UTC, exigés, **469 jours au plus**), `type` `COMPACT` (une
  ligne par transaction) | `FLAT` (frais sur une ligne à part, « pour les relevés comptables »), `statementLocale` ;
  « *This endpoint is SCA protected* » — hors US, AU, NZ, SG, CA, MY, **une fois tous les 90 jours** ; réponse JSON :
  `transactions[]` (`type` `DEBIT`|`CREDIT`, `date`, `amount {value, currency}` — négatif au débit —, `totalFees`,
  `details { type, description, amount, senderName, senderAccount, paymentReference, category, merchant {name, …,
  category}, sourceAmount, targetAmount, fee, rate }`, `exchangeDetails {forAmount, rate}`, `runningBalance`,
  `referenceNumber` — « référence Wise unique, qui permet de rattacher un remboursement au transfert d'origine »),
  `endOfStatementBalance`, `query {intervalStart, intervalEnd, currency, accountId}` ; **aucun
  `startOfStatementBalance` documenté**. Types de `details.type` : `CARD`, `CONVERSION`, `DEPOSIT`, `TRANSFER`,
  `MONEY_ADDED`, `INCOMING_CROSS_BALANCE`, `OUTGOING_CROSS_BALANCE`, `DIRECT_DEBIT`, `BALANCE_INTEREST`,
  `BALANCE_ADJUSTMENT`, `UNKNOWN`, `ACCRUAL_CHARGE`, `INVESTMENT_TRADE_ORDER`, `ACQUIRING_PAYMENT`, `CARD_CASHBACK`,
  `CARD_ORDER_CHECKOUT`.
- Jetons personnels : <https://docs.wise.com/api-docs/features/authentication-access/personal-tokens>,
  <https://docs.wise.com/guides/developer/auth-and-security/personal-api-token> — base `https://api.wise.com`,
  `Authorization: Bearer <jeton>`, actif « jusqu'à révocation » ; ⚠️ « *Funding transfers and retrieving balance
  statements via API are not supported, except for accounts based in the US, Canada, Australia, New Zealand,
  Singapore, and Malaysia* » (Q-39-1).
- Dépréciation : <https://docs.wise.com/guides/developer/auth-and-security/personal-token-migration.md> — « *the
  personal token logic for first party partner accounts will be deprecated by the end of next year* », remplacée par
  OAuth 2.0 et mTLS ; périmètre (« first party », comptes *Enterprise*) sans mention des PME (Q-39-3).
- SCA : <https://docs.wise.com/guides/developer/auth-and-security/sca-over-api>,
  <https://docs.wise.com/guides/developer/auth-and-security/one-time-token>,
  <https://docs.wise.com/guides/developer/auth-and-security/sca-and-2fa.md>,
  <https://docs.wise.com/guides/developer/errors/authentication.md> — `403` avec en-têtes `x-2fa-approval` (jeton à
  usage unique, OTT) et `x-2fa-approval-result: REJECTED` ; la requête se rejoue avec l'OTT dans `x-2fa-approval` une
  fois le défi levé ; une SCA réussie vaut 5 minutes pour les actions « à faible risque ». ⚠️ **La signature du défi
  par une clé privée RSA (en-tête `X-Signature`, clé publique déposée dans les réglages Wise), décrite par B10, ne
  figure dans aucune des pages consultées** : ces pages décrivent les défis d'une application partenaire (PIN,
  empreinte d'appareil, code par téléphone). Le mécanisme est à **constater** à la recette de 39-0 (Q-39-2).
- Soldes et profils : <https://docs.wise.com/api-reference/legacy/balance/tag.md> (un solde `STANDARD` par devise,
  des soldes `SAVINGS` — « Jars » — plusieurs par devise ; seuls les soldes `NOT_INVESTED` se manipulent par l'API),
  <https://docs.wise.com/api-reference/profile> (`GET /profiles`, profils personnel et entreprise). Les chemins
  exacts de la liste des soldes (`/v4/profiles/{profileId}/balances?types=STANDARD`) et la forme de leur réponse ne
  sont **pas** dans les extraits lus : à constater (39-1).

### Dépendance à l'Epic 38 (et, par lui, à l'Epic 37)

L'Epic 39 **ne se développe qu'après l'Epic 38** (développé et fusionné dans sa branche). Le partage est celui que
l'Epic 38 a promis (§ *Ce que réutiliseront les Epics 39 et 40*).

#### Réutilisé tel quel

| Socle | Story 38 | Usage par Wise |
|---|---|---|
| Coffre `_kesh_provider_secrets`, `seal`/`open`, voie unique d'`open`, `KESH_SECRETS_KEY`, secrets effacés à la restauration | 38-1 | jeton personnel et clé privée SCA (`ciphertext VARBINARY(16384)` : « une clé privée PEM de Wise y tient », 38-1 AC-2) |
| Connexions `payment_provider_connections`, `settings JSON` typé (E24), routes C-38-2 | 38-1 | `provider = 'wise'` (format seul, sans DDL), struct `WiseSettings` |
| Compte prestataire `payment_provider_accounts` (un compte `bank` par devise, compte du plan en devise, compte de frais de la devise), gardes, levée de C-37-9 | 38-2 | trois soldes CHF, EUR, USD (B1), frais Wise par devise (B3) |
| Schéma des mouvements, passages, soldes vus et instantanés, en-tête `PROVIDER_API`, gardes d'import | 38-3a | inchangé |
| `kesh-providers` : `Connector`, `MovementPage`, `ConnectorError` (`code()`, `app_code()`), `validate_page`, `FakeConnector` ; `HttpTransport`, `ConnectorFactory`, `ConnectorContext` ; passage, planification, verrou, complétude (E20 a) ; route `_test/provider-connector` | 38-3b | `WiseConnector` derrière la même fabrique |
| Critères neufs des règles (`invoice_prefix`, `event_code_prefix`, filtres sens, devise, compte) | 38-4 | référence de paiement → `invoice_ref`, code court du type Wise → `event_code` |
| TVA par défaut d'un compte, impôt sur les acquisitions (B18) | 38-5 | achats par carte de services étrangers |
| Propositions (vente, achat, frais ; conversions, retraits, remboursements, file « à examiner ») et acceptation en lot | 38-6a, 38-6b | genres de mouvement de C-38-4, sans valeur neuve |
| Rattachement en devise, en lot, à un groupe (2024, B17, B28) ; API des scripts (B21) | 38-6c1, 38-6c2 | inchangé |
| Écrans génériques des connexions, comptes, soldes, contrôles ; règles, propositions, file | 38-9a, 38-9b | complétés par 39-6 |
| Pièces de compte `bank_account_documents` (`source` à domaine ouvert, C-38-10-6), écart au solde comptable, contrôle de clôture K9 | 38-10 | PDF Wise rangé en `source = 'provider_api'` |

#### Ajouté (sans migration)

- `kesh-providers::wise` : client (jeton, profils, soldes), **SCA** (signature du défi), relevé JSON → mouvements,
  classement des types Wise, relevé PDF.
- `kesh-core::providers::settings` : `Provider::Wise`, `WiseSettings` (E24 l'annonçait : `profileId`, empreinte de la
  clé publique SCA et sa date, type et langue du relevé PDF ; `balanceIds` **non retenu**, F6).
- Amendements **additifs** de contrats de l'Epic 38, posés par les stories de 39 (C-39-2) : secrets nommés dans
  `ConnectorContext`, secrets exigés par prestataire, en-têtes de réponse du `Transport`, variante `wiseResponses` de
  la route `_test`, fonction de rangement de 38-10 appelable par le serveur.
- Deux routes : génération de la clé SCA (39-2), relevé PDF à la demande (39-5).

### Décisions (F1…)

- **F1 — Source des mouvements : le relevé JSON, `type=COMPACT`, par solde.** `statement.json` porte tout ce qu'E7 et
  E8 demandent : montant net (`amount`), frais (`totalFees`), conversion (`exchangeDetails`, `details.sourceAmount`,
  `targetAmount`, `rate`), contrepartie (`merchant.name`, `senderName`), référence de paiement, et **`runningBalance`**,
  qui contrôle la complétude ligne à ligne. `COMPACT` garde les frais **dans** la transaction (E7 : une transaction =
  un mouvement du solde, brut par la règle, frais au sous-compte, B3). *Raison* : une seule source pour les mouvements,
  les soldes et le relevé PDF (F14) ; même authentification forte. L'API *activities* n'est pas retenue (pas de
  frais ni de solde courant documentés dans les pages lues).
- **F2 — Deux secrets au coffre** : `api_token` (jeton personnel) et `sca_private_key` (clé privée RSA, PEM PKCS#8).
  `ConnectorContext` passe d'**un** secret ouvert à des **secrets nommés** (`secrets: BTreeMap<&'static str,
  Zeroizing<String>>`, ouverts par la voie unique de 38-1 AC-1 au même site) ; PayPal lit `client_secret`, sans
  changement de comportement. Les contrôles « secret présent / déchiffrable » de 38-1 AC-7 passent par une liste des
  **secrets exigés par prestataire** (PayPal : `client_secret` ; Wise : `api_token` ; `sca_private_key` exigé
  seulement pour les relevés, F4). *Raison* : un prestataire à deux secrets est le cas que 38-1 avait prévu
  (`PRIMARY KEY (connection_id, name)`), non la fabrique de 38-3b.
- **F3 — La clé SCA est générée par Kesh** (RSA 2048, par défaut) : `POST /provider-connections/{id}/sca-key`
  (Administrateur, session) scelle la clé privée au coffre et pose aux réglages, **champs posés par le serveur**, la
  clé publique PEM (non secrète, à copier chez Wise), son empreinte SHA-256 et sa date. Une clé privée **existante**
  (déjà déposée chez Wise) se dépose par `PUT …/secrets` (`scaPrivateKey`) ; le serveur en dérive la clé publique et
  l'empreinte. La clé privée n'est **jamais** réaffichée. *Raison* : générée dans Kesh, la clé privée ne transite
  jamais par un fichier ni par le presse-papiers de l'Administrateur.
- **F4 — Défi SCA traité par le connecteur, sans humain.** Réponse `403` portant `x-2fa-approval` → le connecteur
  signe l'OTT par la clé privée (forme exacte à constater, Q-39-2 : SHA-256 avec RSA, signature base64 dans
  `X-Signature`) et **rejoue une seule fois** la requête avec `x-2fa-approval` et la signature ; second refus →
  `ConnectorError::ScaRejected` (`code()` `sca_rejected`, `app_code()` `PROVIDER_SCA_FAILED`) ; clé absente →
  `ScaKeyMissing` (`sca_key_missing`, `PROVIDER_SCA_KEY_MISSING`). Les « 90 jours » de Wise (B10) sont la durée de
  validité **de l'approbation côté Wise** : Kesh ne la stocke pas et ne la compte pas — il signe chaque fois qu'un
  défi revient. La clé elle-même n'expire pas (à constater, Q-39-2). *Raison* : aucune donnée de plus à tenir ; un
  défi qui revient tous les 90 jours ne demande aucune action de l'Administrateur tant que la clé est déposée.
- **F5 — Profil choisi par l'Administrateur.** Un jeton personnel voit plusieurs profils (personnel, entreprise) ;
  « Tester la connexion » les liste (`ConnectorTest` gagne `profiles: [{ id, type, name }]`, champ facultatif,
  additif) ; `profileId` (réglage du client) doit être l'un d'eux. **Immuable** dès qu'un compte prestataire est
  rattaché à la connexion (`PROVIDER_SETTINGS_LOCKED`, `details.field = "profileId"`) : changer de profil changerait
  le sens des comptes liés.
- **F6 — Soldes résolus à chaque passage, non stockés.** Le connecteur lit la liste des soldes `STANDARD` du profil
  (devise → identifiant) à chaque passage ; aucun `balanceIds` aux réglages. `balances(None)` = cette liste ;
  `balances(Some(as_of))` = `endOfStatementBalance` d'un relevé JSON de `[as_of − 1 jour, as_of)` par devise,
  `as_of` rendu = `intervalEnd` (38-3b AC-7 : l'`as_of` écrit est celui que rend le connecteur). Soldes `SAVINGS`
  (« Jars ») non téléchargés ; un solde `STANDARD` sans compte → soldes vus (E12, B2).
- **F7 — Identifiant d'un mouvement : `<referenceNumber>:<D|C>`.** `referenceNumber` est « unique » mais rattache un
  remboursement à son origine (doc) : un remboursement de carte risque de porter la référence de l'achat **sur le
  même solde**, que l'unicité `(bank_account_id, provider_tx_id)` (38-3a) traiterait comme « connu » — **perte
  muette**. Le suffixe du sens (`DEBIT` → `D`, `CREDIT` → `C`) les distingue ; `related_provider_tx_id` d'un `C` de
  carte = le `D` de même référence quand il existe. Deux lignes de même référence et même sens dans une page →
  `InvalidResponse` (refus bruyant, `validate_page`). ≤ 64 caractères ASCII (38-3a). À constater sur les réponses du
  responsable de l'epic (39-4).
- **F8 — `event_code` court.** La colonne est `VARCHAR(16)` (38-3a) et quatre types Wise la dépassent
  (`INCOMING_CROSS_BALANCE`, 22). Une **table constante** de codes courts (≤ 16, majuscules, p. ex. `CARD`,
  `CONVERSION`, `TRANSFER`, `DEPOSIT`, `MONEY_ADDED`, `DIRECT_DEBIT`, `INTEREST`, `ADJUSTMENT`, `IN_CROSS_BAL`,
  `OUT_CROSS_BAL`, `ACCRUAL_CHARGE`, `INVEST_TRADE`, `ACQUIRING`, `CASHBACK`, `CARD_ORDER`, `UNKNOWN`) ; type inconnu
  → `UNKNOWN`. *Raison* : aucune migration ; le critère `event_code_prefix` (38-4) s'écrit sur ces codes, listés au
  manuel.
- **F9 — Classement** (39-4, table constante, pure) :

  | `details.type` | `DEBIT` | `CREDIT` |
  |---|---|---|
  | `CARD` | `purchase` (marchand) | `refund`, relié au `D` de même référence (F7) ; origine absente ou non réconciliée → file, motif `refund_origin_missing` **calculé** par 38-6b (O11) |
  | `CONVERSION` | `conversion` (moitié qui paie) | `conversion` (moitié qui reçoit) — paire par `referenceNumber` |
  | `TRANSFER` | `purchase` (paiement envoyé ; virement à soi : Q-39-4) | `sale` (paiement reçu) ; `deposit` si `senderAccount` est l'IBAN d'un compte bancaire de la société (F12) |
  | `DEPOSIT` | `other` | `sale` ; `deposit` si `senderAccount` = IBAN propre (F12) |
  | `MONEY_ADDED` | `other` | `deposit` |
  | `DIRECT_DEBIT` | `purchase` | `refund`, comme `CARD` |
  | `ACCRUAL_CHARGE`, `CARD_ORDER_CHECKOUT` | `fee` | `other` |
  | `ACQUIRING_PAYMENT` | `other` | `sale` |
  | `BALANCE_INTEREST` | `fee` | `sale` (règle `event_code_prefix` `INTEREST` vers le compte d'intérêts) |
  | `INCOMING_CROSS_BALANCE`, `OUTGOING_CROSS_BALANCE`, `BALANCE_ADJUSTMENT`, `INVESTMENT_TRADE_ORDER`, `CARD_CASHBACK`, `UNKNOWN`, tout type inconnu | `other` | `other` |

  `other` → `review_reason = unknown_event` (file « à examiner », B7) ; **aucun type inconnu ne devient une vente**
  (patron NFR-38-8-1). Surcharges `eventKindOverrides` (patron 38-8 AC-7) sur les seuls codes classés `other` sans
  condition, clé = code court de F8. La table se confirme à la fiche, type par type, sur les réponses du responsable de l'epic.
- **F10 — Montants** (E7, `net = gross + fee`) : `net = amount.value` (signé) ; `fee = −totalFees.value` (≤ 0 ;
  `totalFees.currency` ≠ devise → `InvalidResponse`) ; `gross = net − fee`. Le coût de conversion porté par
  `totalFees` de la moitié qui paie va au compte de frais de sa devise (B4, B27 : jamais dans la différence de change).
  **Contrôle de continuité** : dans un relevé, chaque `runningBalance` = précédent + `amount` (ordre chronologique) ;
  rupture → `InvalidResponse { reason: "running_balance" }`. À constater : signe et inclusion des frais dans `amount`.
- **F11 — Fenêtres** : `max_window()` = 31 jours (Wise admet 469 ; la borne garde des réponses sous 8 MiB et une page
  unique, `total_pages = 1`, comme PayPal) ; `history_start` = `None` tant que la recette n'a pas constaté de limite ;
  `overlap()` = `overlapDays` des réglages, **défaut 7** (1 à 30) — un achat par carte peut paraître au relevé
  plusieurs jours après sa date ; fin de fenêtre « maintenant − 3 h » de 38-3b **gardée** : la méthode
  `Connector::settle_delay()` évoquée par 38-3b (*Limite non traitée*) **n'est pas ajoutée**, le recouvrement suffit.
- **F12 — Virements entre ses comptes.** Un crédit dont `senderAccount` est l'IBAN d'un compte bancaire **de la
  société** (lu par la fabrique, `ConnectorContext.own_ibans`, patron de `other_payer_ids` de 38-3b AC-3) est un
  `deposit`, rattaché des deux côtés au relevé CAMT de la banque (34-11b, E16). `MONEY_ADDED` (« ajout d'argent »)
  est un `deposit`. Le débit vers son propre compte bancaire n'a pas de compte destinataire au relevé (champs lus) :
  Q-39-4.
- **F13 — Les cartes Wise débitent les soldes.** Aucun compte `credit_card` (34-11a1) : un achat par carte est un
  mouvement du solde de sa devise (conversion automatique comprise, portée par `exchangeDetails` et `totalFees`).
- **F14 — Relevé PDF par l'API, rangé par 38-10.** `statement.pdf` d'un solde et d'une période (mois ou exercice),
  `type` des réglages (`statementType`, défaut **`FLAT`**, recommandé par Wise pour la comptabilité), langue des
  réglages (`statementLocale`, défaut la langue de la société) ; rangé dans `bank_account_documents` par la fonction de
  service de 38-10 (aucune requête HTTP interne), `source = 'provider_api'`, `created_by_user_id` = le demandeur ou
  l'auteur de la connexion ; **solde de clôture** = `endOfStatementBalance` du relevé JSON du même intervalle (somme de
  contrôle, B20 ; écart et K9 de 38-10 sans code neuf). À la demande (`POST /provider-connections/{id}/statements`),
  et, si le réglage `monthlyStatementPdf` est vrai, le mois écoulé par la tâche quotidienne de 38-3b (à partir du 3 du
  mois). Une pièce `provider_api` déjà rangée pour le même compte et la même période **n'est pas redemandée** (un PDF
  régénéré change d'octets : l'unicité par empreinte de 38-10 ne suffit pas).
- **F15 — Aucune migration.** `provider = 'wise'`, genres et motifs de C-38-4, `event_code` court (F8),
  `source = 'provider_api'` : tout entre dans des colonnes existantes à domaine ouvert ; réglages dans `settings`
  (E24) ; secrets dans le coffre. `kesh_version_min_required` inchangé ; P5–P8 sans objet.
- **F16 — Tests sans réseau** (B12, C-38-11) : réponses Wise enregistrées et **anonymisées** sous
  `crates/kesh-providers/tests/fixtures/wise/` ; transport factice au format Wise par une variante additive
  `wiseResponses` de `POST /api/v1/_test/provider-connector` (patron `paypalResponses`, 38-3b AC-3, chaîne
  `"default"` comprise) ; clé RSA de test générée par les tests, jamais réelle ; défi SCA simulé.
- **F17 — Aucun outil MCP neuf** (E23).

### Registre des contrats

- **C-39-0 — Aucune migration** (toutes) : F15. Une fiche qui découvre un DDL, une valeur neuve dans une colonne à
  domaine fermé, un `MODIFY`/`CHANGE COLUMN` ou un `DROP` s'arrête et le dit.
- **C-39-1 — Connexion Wise** (39-1) : `provider = 'wise'` ; `WiseSettings` `#[serde(deny_unknown_fields,
  rename_all = "camelCase")]` : modifiables par le client — `profileId`, `overlapDays` (1–30, défaut 7),
  `eventKindOverrides` (39-4), `statementType` (`COMPACT`|`FLAT`, défaut `FLAT`), `statementLocale`,
  `monthlyStatementPdf` (défaut faux) ; posés par le serveur — `scaPublicKeyPem`, `scaKeyFingerprint`,
  `scaKeyCreatedAt` (39-2) ; secrets `apiToken`, `scaPrivateKey` (`PUT …/secrets`, corps propre au prestataire,
  38-1 AC-6). URL de base constante `https://api.wise.com`.
- **C-39-2 — Amendements additifs du socle 38** (39-1, sauf mention) : secrets nommés de `ConnectorContext` et liste
  des secrets exigés par prestataire (F2) ; `ConnectorTest.profiles` facultatif (F5) ; en-têtes de requête et de
  réponse dans `Transport` s'ils n'y sont pas (`x-2fa-approval`, `X-Signature`, 39-2) ; `own_ibans` dans
  `ConnectorContext` (39-4) ; `wiseResponses` (F16) ; rangement de 38-10 appelable par le serveur (39-5). PayPal
  inchangé : ses tests restent verts sans retouche de leurs attentes.
- **C-39-3 — Erreurs** : variantes neuves de `ConnectorError` `ScaRejected`, `ScaKeyMissing` (F4) ; codes d'API
  `PROVIDER_SCA_FAILED`, `PROVIDER_SCA_KEY_MISSING` ; erreurs Wise sur la table de 38-7 AC-7 (401 → `Auth` ; 403 sans
  `x-2fa-approval` → `Permission` ; 429 → `RateLimited` ; 5xx, délai → `Unavailable` ; 400 → `Rejected`) ;
  `PROVIDER_SETTINGS_LOCKED` (F5) ; `PROVIDER_STATEMENT_FAILED` (39-5, 409, `details.errorCode`). Aucun texte de Wise à
  l'écran ni dans `last_error`.
- **C-39-4 — Mouvement Wise** (39-3, 39-4) : F7 à F12 ; correspondance vers `ProviderMovement` en **une** fonction ;
  `invoice_ref` = `details.paymentReference` (critère « boutique » de 38-4 pour un virement reçu) ; `reference` =
  `referenceNumber` ; `counterparty_name` = `merchant.name` ou `senderName` ; `details` = `details.description` ;
  `counterparty_email`, `item_names` `NULL`.
- **C-39-5 — Relevé PDF** (39-5) : F14 ; route `POST /api/v1/provider-connections/{id}/statements` `{ bankAccountId,
  periodFrom, periodTo }` (Comptable ou Administrateur, session, clés d'API refusées dans le handler) → `201` la pièce
  (forme de 38-10 AC-3) ; `409 BANK_DOCUMENT_ALREADY_FETCHED` (F14) ; exercice absent → `400
  BANK_DOCUMENT_NO_FISCAL_YEAR` (code de 38-10).
- **C-39-6 — Registres** : `LIB_ROUTES` +2 (`sca-key`, `statements`), `TEST_ENDPOINT_ROUTES` inchangé (variante de la
  même route) ; ces compteurs (`LIB_ROUTES`, `sitesTotal`, `Cargo.lock`) **se recomptent depuis la source après chaque
  reprise de `main`**, sans se fusionner ; audit : `payment_provider_connection.sca_key_generated` ; dépôt de la clé par
  `payment_provider_connection.secrets_updated` (`names`) ; pièce rangée par `bank_account_document.attached`
  (38-10), détail `{ source: "provider_api", trigger }`.
- **C-39-7 — Tracé pour plus tard** (issues avec la PR) : migration des jetons personnels vers OAuth 2.0 + mTLS si
  Wise l'impose aux PME (Q-39-3) ; rotation de la clé SCA (plusieurs clés déposées chez Wise ?) ; soldes `SAVINGS` et
  mouvements entre soldes.

### Stories

| Story | Objet | Modules (≤ 5) | Dépend de | Migration |
|---|---|---|---|---|
| 39-0 | kickoff : décomptes, validation de ce fichier, **recette de faisabilité** sur le compte réel du responsable de l'epic (Q-39-1, Q-39-2) | — | — | — |
| 39-1 | client Wise : réglages, jeton personnel, profils, soldes, « Tester la connexion », branchement, amendements du socle (C-39-2) | `kesh-providers` (wise), `kesh-core` (réglages), `kesh-api` (secrets, fabrique, test, transport), `kesh-i18n` | Epic 38, 39-0 | aucune |
| 39-2 | authentification forte : clé RSA générée ou déposée, signature du défi, rejeu, erreurs SCA | `kesh-providers` (wise::sca), `kesh-api` (route `sca-key`, secrets), `kesh-core` (champs posés par le serveur), `kesh-i18n` | 39-1 | aucune |
| 39-3 | relevé JSON → mouvements et soldes : `movements`, `balances(as_of)`, fenêtres, validation et continuité | `kesh-providers` (wise::statement) | 39-1, 39-2 | aucune |
| 39-4 | classement des types Wise, montants, identifiants et liens, virements entre ses comptes, surcharges | `kesh-providers` (wise::classify), `kesh-core` (surcharges), `kesh-api` (`own_ibans` de la fabrique) | 39-3 | aucune |
| 39-5 | relevé PDF par l'API, rangé au compte et à l'exercice (38-10), solde de clôture, à la demande et mensuel | `kesh-providers` (wise::pdf), `kesh-api` (route, tâche, rangement), `kesh-db` (pièce par période), `kesh-i18n` | 39-2, 38-10 | aucune |
| 39-6 | écrans et manuels : connexion Wise, profil, clé SCA, relevés PDF ; E2E et captures | `frontend`, `kesh-i18n`, `docs/manual`, E2E | 39-1 à 39-5, 38-9a, 38-9b | aucune |

**Ordre** : 39-0 (recette) → 39-1 → 39-2 → 39-3 → 39-4 → 39-5 → 39-6 ; 39-5 peut suivre 39-2 en parallèle de 39-3.
**Migrations : aucune** (C-39-0). Chaque fiche porte ses exigences numérotées (B26) ; les gardes communes (P1–P8,
*Pattern batch*, aucun appel réseau en CI, recette visuelle) ne s'y répètent que là où la story les porte en propre.

#### 39-0 — Kickoff et recette de faisabilité

- *Intention* : ouvrir l'epic sur des décomptes mesurés, un fichier validé, et la preuve que l'API sert ce que l'epic
  suppose sur le compte suisse du responsable de l'epic.
- *Périmètre* : décomptes (§ *Priorités des défauts*) ; validation en boucle ; **recette par le responsable de l'epic, hors de Kesh et hors
  du dépôt**, avec son jeton (lecture seule si Wise en propose) : `GET /profiles`, liste des soldes, `statement.json`
  d'un mois avec le défi SCA signé par sa clé, `statement.pdf` ; réponses **anonymisées** remises pour les fixtures
  (F16) ; constats écrits ici (Q-39-1, Q-39-2, Q-39-7, F7, F10, F11).
- *Risques* : refus des relevés au jeton personnel (Q-39-1) — l'epic se redécoupe alors (repli possible : relevé
  CAMT.053 téléchargé à la main, ce que 38-3a refuse aujourd'hui sur un compte prestataire) ; aucun appel réseau par un
  agent (B12).
- **FR**
  1. `FR-39-0-1` — Compter issues et bugs, écrire au fichier le compte du tag précédent et l'objectif du tag.
  2. `FR-39-0-2` — Valider ce fichier en boucle jusqu'à une passe sans finding au-dessus de LOW.
  3. `FR-39-0-3` — Consigner la recette de faisabilité du responsable de l'epic (jeton, profils, soldes, relevé JSON avec SCA, PDF) et
     ses constats ; aucune fiche ne se développe sans elle.
- **NFR**
  1. `NFR-39-0-1` — Décomptes recomptés à la source, commande et périmètre écrits avec le nombre.
  2. `NFR-39-0-2` — Aucune réponse réelle non anonymisée ni aucun jeton dans le dépôt.

#### 39-1 — Client Wise : connexion, jeton, profils, soldes

- *Intention* : un Administrateur crée une connexion Wise, dépose son jeton personnel, choisit le profil d'entreprise,
  et « Tester la connexion » lit profils et soldes.
- *Périmètre* : `Provider::Wise`, `WiseSettings` (C-39-1) ; corps de `PUT …/secrets` pour Wise (`apiToken`) ; secrets
  nommés dans `ConnectorContext` et secrets exigés par prestataire (F2, C-39-2) ; `WiseConnector` (client, `test()`,
  `balances(None)`) derrière `ConnectorFactory` ; `ConnectorTest.profiles` (F5) ; `profileId` immuable dès qu'un
  compte est rattaché ; erreurs (C-39-3) ; en-têtes du `Transport` vérifiés (C-39-2) ; `wiseResponses` et jeu de
  réponses synthétiques `"default"` (F16) ; `earliest_fetch_from("wise")` = `None` (F11).
- *Risques* : refactor de `ConnectorContext` sans changer PayPal (non-régression de 38-7) ; profil personnel choisi
  par erreur (avertissement à l'écran, 39-6) ; jeton à portée plus large que la lecture (dit au manuel).
- **FR**
  1. `FR-39-1-1` — Créer une connexion `wise` avec ses réglages (C-39-1), validés par `WiseSettings`.
  2. `FR-39-1-2` — Déposer le jeton personnel (`apiToken`), jamais réaffiché.
  3. `FR-39-1-3` — « Tester la connexion » lit les profils et les soldes `STANDARD` ; `profileId` doit être un profil
     listé.
  4. `FR-39-1-4` — `profileId` ne change plus dès qu'un compte prestataire est rattaché (`PROVIDER_SETTINGS_LOCKED`).
  5. `FR-39-1-5` — Chaque erreur Wise devient un code lisible (C-39-3).
  6. `FR-39-1-6` — La fabrique construit un `WiseConnector` pour `wise` ; en mode test, sur le transport factice
     `wiseResponses`.
- **NFR**
  1. `NFR-39-1-1` — URL de base constante, client borné de 37-2 et bornes de 38-3b par requête.
  2. `NFR-39-1-2` — Jeton absent de tout journal, erreur, réponse, audit, export et sauvegarde (test au marqueur).
  3. `NFR-39-1-3` — PayPal inchangé : suites de 38-1, 38-3b et 38-7 vertes sans retouche de leurs attentes.
  4. `NFR-39-1-4` — Aucune migration (C-39-0).

#### 39-2 — Authentification forte (SCA)

- *Intention* : Kesh lève seul le défi SCA de Wise en le signant par une clé privée gardée au coffre, dont la clé
  publique est déposée chez Wise.
- *Périmètre* : F3, F4 ; route `POST …/sca-key` (Administrateur, session) ; dépôt d'une clé privée existante
  (`scaPrivateKey`, PEM PKCS#8 ou PKCS#1, RSA ≥ 2048) ; signature pure dans `kesh-providers::wise::sca` ; rejeu
  unique ; contrôle `sca_ok` de « Tester » (un relevé JSON d'un jour) ; audit `sca_key_generated`.
- *Risques* : mécanisme exact à constater (Q-39-2) ; **dépendance cryptographique neuve** (génération et signature
  RSA : crate `rsa` de RustCrypto — avis RUSTSEC-2023-0071, canal temporel « Marvin » sur les opérations à clé
  privée, à peser à la fiche contre une autre bibliothèque ; licence compatible EUPL-1.2) ; clé régénérée sans
  dépôt chez Wise → passages en échec jusqu'au dépôt (dit à l'écran et au manuel) ; boucle de défis (un seul rejeu).
- **FR**
  1. `FR-39-2-1` — Générer une paire RSA : privée scellée au coffre, publique et empreinte aux réglages, publique
     rendue pour être copiée chez Wise.
  2. `FR-39-2-2` — Déposer une clé privée existante ; le serveur en dérive la clé publique et l'empreinte.
  3. `FR-39-2-3` — Sur un `403` portant `x-2fa-approval`, signer le défi et rejouer la requête une seule fois.
  4. `FR-39-2-4` — Second refus → `PROVIDER_SCA_FAILED` ; clé absente → `PROVIDER_SCA_KEY_MISSING`, à l'état du
     passage et à « Tester ».
  5. `FR-39-2-5` — « Tester la connexion » vérifie un appel protégé (`sca_ok`).
- **NFR**
  1. `NFR-39-2-1` — Clé privée jamais réaffichée ni journalisée ; signature pure, testée avec une clé de test et un
     vecteur vérifié par la clé publique.
  2. `NFR-39-2-2` — Dépendance cryptographique déclarée avec sa licence et ses avis de sécurité, jugés à la fiche.
  3. `NFR-39-2-3` — Au plus une signature et un rejeu par défi ; aucun texte de Wise en clair.
  4. `NFR-39-2-4` — Route réservée à une session d'Administrateur (`require_not_pat`), auditée, `LIB_ROUTES` +1 (recompté depuis la source après chaque reprise de `main`) ; `Cargo.lock` régénéré après la
     fusion, non fusionné.

#### 39-3 — Relevé JSON : mouvements et soldes

- *Intention* : le passage de 38-3b lit les mouvements et les soldes de chaque solde Wise lié.
- *Périmètre* : F1, F6, F11 ; `movements(window, page)` : un `statement.json` `COMPACT` par solde lié et par
  sous-fenêtre, page unique ; `balances(Some(as_of))` par `endOfStatementBalance` ; validation (devise de la réponse =
  devise demandée, `query` conforme à la requête, dates dans la fenêtre, continuité de `runningBalance`) ; soldes
  sans compte → soldes vus (E12).
- *Risques* : statut tardif d'un achat par carte (recouvrement de 7 jours, F11) ; relevé vide d'une devise sans
  mouvement ; fuseau des dates (UTC → date à Zurich, 38-3b) ; nombre d'appels (trois soldes × sous-fenêtres + soldes).
- **FR**
  1. `FR-39-3-1` — Lire les mouvements d'une fenêtre par le relevé JSON de chaque solde lié, en une page.
  2. `FR-39-3-2` — Lire le solde d'une devise à un instant (`endOfStatementBalance`), et les soldes courants.
  3. `FR-39-3-3` — `max_window` 31 jours, `overlap` `overlapDays` (défaut 7), `history_start` non borné sauf constat.
  4. `FR-39-3-4` — Un solde `STANDARD` sans compte Kesh est listé (B2), ses mouvements non téléchargés.
  5. `FR-39-3-5` — Une rupture de `runningBalance` ou une réponse non conforme à la requête fait échouer la page.
- **NFR**
  1. `NFR-39-3-1` — Validation stricte ; un test par refus, réponses hostiles comprises (C-38-11).
  2. `NFR-39-3-2` — Fixtures anonymisées issues de la recette de 39-0.
  3. `NFR-39-3-3` — Un appel par solde et par sous-fenêtre, sous les bornes de 38-3b.

#### 39-4 — Classement des types Wise

- *Intention* : chaque ligne de relevé devient un `ProviderMovement` classé, avec brut, frais, net, liens et
  contrepartie, et ce que Kesh ne sait pas classer va à la file « à examiner ».
- *Périmètre* : F7 à F10, F12, C-39-4 ; table constante types → genres et codes courts ; liens des moitiés de
  conversion et des remboursements de carte ; `own_ibans` dans `ConnectorContext` ; `eventKindOverrides` Wise
  (patron 38-8 AC-7, clés = codes courts).
- *Risques* : signe et inclusion des frais (F10, à constater) ; remboursement de carte sans achat connu ; conversion
  automatique d'un achat par carte ; virement à soi sortant non reconnu (Q-39-4) ; « Jars » et mouvements entre soldes
  (`other`).
- **FR**
  1. `FR-39-4-1` — Chaque type Wise se classe selon la table de F9 ; un type inconnu → `other`, `unknown_event`.
  2. `FR-39-4-2` — Brut, frais et net se lisent de `amount` et `totalFees` (F10).
  3. `FR-39-4-3` — Identifiant `<referenceNumber>:<D|C>` ; moitiés de conversion et remboursements reliés.
  4. `FR-39-4-4` — Contrepartie, référence de paiement (`invoice_ref`), description et code court se posent (C-39-4).
  5. `FR-39-4-5` — Un crédit venu d'un IBAN de la société est un `deposit` (F12).
  6. `FR-39-4-6` — Un réglage de connexion peut classer un code que la table met en `other`.
- **NFR**
  1. `NFR-39-4-1` — Classement pur, sans base, réseau ni horloge.
  2. `NFR-39-4-2` — Un test par type relevé sur les réponses du responsable de l'epic ; aucun type inconnu ne devient une vente.
  3. `NFR-39-4-3` — Somme des nets d'un relevé = variation de `runningBalance` (test).

#### 39-5 — Relevé PDF par l'API

- *Intention* : Kesh demande à Wise le relevé PDF d'un solde pour un mois ou un exercice, le range au compte et à
  l'exercice, et en remplit le solde de clôture, qui devient la somme de contrôle de 38-10.
- *Périmètre* : F14, C-39-5 ; `statement.pdf` (SCA de 39-2) et `endOfStatementBalance` du même intervalle ; rangement
  par la fonction de service de 38-10 (`source = 'provider_api'`) ; route à la demande ; mois écoulé par la tâche
  quotidienne si `monthlyStatementPdf` ; pièce déjà rangée pour la période → non redemandée.
- *Risques* : PDF régénéré à octets différents (F14) ; intervalle à cheval sur deux exercices (rangé à l'exercice de
  sa fin, C-38-10-2) ; exercice absent (motif à l'état, pas d'échec du passage) ; taille (borne de 38-10) ; erreur Wise
  sur une route synchrone (409, jamais 5xx : bandeau du frontend, 38-7 AC-7).
- **FR**
  1. `FR-39-5-1` — Obtenir à la demande le relevé PDF d'un compte Wise pour une période.
  2. `FR-39-5-2` — Le ranger comme pièce du compte et de l'exercice (38-10), solde de clôture rempli.
  3. `FR-39-5-3` — Si le réglage le demande, obtenir chaque mois le relevé du mois écoulé.
  4. `FR-39-5-4` — Ne pas redemander une période déjà rangée (`BANK_DOCUMENT_ALREADY_FETCHED`).
  5. `FR-39-5-5` — Une erreur de Wise se lit sans statut 5xx (`PROVIDER_STATEMENT_FAILED`).
- **NFR**
  1. `NFR-39-5-1` — Fichier contrôlé (`%PDF-`, borne de taille de 38-10) avant rangement.
  2. `NFR-39-5-2` — Pièce auditée (`bank_account_document.attached`, `source`, déclencheur), sauvegardée par 38-10.
  3. `NFR-39-5-3` — Aucune migration ; `LIB_ROUTES` +1 (recompté depuis la source après chaque reprise de `main`) ; clés d'API refusées.
  4. `NFR-39-5-4` — Aucun appel réseau en CI : PDF factice servi par `wiseResponses`.

#### 39-6 — Écrans et manuels

- *Intention* : ce qui diffère de 38-9a pour Wise se fait à l'écran : jeton, profil, clé SCA, relevés PDF.
- *Périmètre* : fiche de connexion Wise (jeton en saisie seule ; profil choisi dans la liste de « Tester » ; champs de
  C-39-1) ; **clé SCA** : générer, afficher et copier la clé publique et son empreinte, déposer une clé existante,
  avertir qu'une clé neuve doit être déposée chez Wise ; libellés des codes neufs (`PROVIDER_SCA_*`,
  `PROVIDER_SETTINGS_LOCKED`, `PROVIDER_STATEMENT_FAILED`) ; *Relevés joints* d'un compte Wise : « Obtenir le relevé de
  Wise » (période) ; codes courts de F8 en aide du critère `event_code_prefix` ; manuel d'administration (jeton, clé
  publique chez Wise, SCA, 90 jours, restauration = secrets à ressaisir) et utilisateur (classement Wise, relevés).
- *Risques* : clé privée jamais au DOM ; profil personnel choisi (avertissement) ; textes « proposer », jamais
  « comptabiliser automatiquement ».
- **FR**
  1. `FR-39-6-1` — Créer et régler une connexion Wise, jeton en saisie seule, profil choisi parmi ceux lus.
  2. `FR-39-6-2` — Générer, afficher, copier ou déposer la clé SCA, avec son empreinte et sa date.
  3. `FR-39-6-3` — Les erreurs Wise et SCA s'affichent en clair (quatre locales).
  4. `FR-39-6-4` — Obtenir un relevé PDF depuis *Relevés joints* ; régler le relevé mensuel.
  5. `FR-39-6-5` — Les codes courts Wise s'affichent en aide du critère « code d'événement ».
- **NFR**
  1. `NFR-39-6-1` — Aucun secret (jeton, clé privée) au DOM ni dans les réponses lues.
  2. `NFR-39-6-2` — Quatre locales ; manuels d'administration et utilisateur à jour : modifier le `.tex` seulement, le PDF se régénère à la release
     (contrôle local du rendu permis, aplati, PDF non commité) ; chaque fiche porte sa *Note de version*.
  3. `NFR-39-6-3` — E2E contre le transport factice (`wiseResponses: "default"`) : connexion → test → téléchargement →
     proposition → acceptation ; relevé PDF rangé.
  4. `NFR-39-6-4` — Captures d'écran de chaque état touché (*Recette visuelle*).

### Exigences (FR/NFR) par story

Récapitulatif (B26) ; compté aux sections ci-dessus le 2026-11-15 (`grep -oE "\bFR-39-[0-9]+-[0-9]+" … | sort -u |
wc -l`, de même pour `NFR`). Les fiches feront foi. Forme définitive — issues ou fiches — à Guy (Q12 de l'Epic 38).

| Story | FR | NFR |
|---|---|---|
| 39-0 | 3 — 1 décomptes · 2 validation · 3 recette de faisabilité | 2 — 1 décomptes recomptés · 2 rien de réel au dépôt |
| 39-1 | 6 — 1 connexion `wise` · 2 jeton · 3 « Tester », profils, soldes · 4 profil verrouillé · 5 erreurs lisibles · 6 fabrique et factice | 4 — 1 URL constante, bornes · 2 jeton nulle part · 3 PayPal inchangé · 4 aucune migration |
| 39-2 | 5 — 1 générer la clé · 2 déposer une clé · 3 signer et rejouer · 4 erreurs SCA · 5 « Tester » protégé | 4 — 1 clé privée nulle part, signature testée · 2 dépendance jugée · 3 un rejeu · 4 route d'Administrateur auditée |
| 39-3 | 5 — 1 mouvements par relevé · 2 soldes à un instant · 3 fenêtres · 4 solde sans compte · 5 continuité | 3 — 1 validation stricte · 2 fixtures anonymisées · 3 appels bornés |
| 39-4 | 6 — 1 table des types · 2 montants · 3 identifiants et liens · 4 champs · 5 virements à soi · 6 surcharges | 3 — 1 pur · 2 test par type · 3 somme des nets |
| 39-5 | 5 — 1 PDF à la demande · 2 rangement et solde · 3 mensuel · 4 pas de doublon · 5 erreur sans 5xx | 4 — 1 PDF contrôlé · 2 audit, sauvegarde · 3 aucune migration, registre · 4 aucun réseau en CI |
| 39-6 | 5 — 1 connexion et profil · 2 clé SCA · 3 erreurs · 4 relevés PDF · 5 codes courts | 4 — 1 aucun secret au DOM · 2 i18n, manuels · 3 E2E · 4 captures |

**Total : 35 FR et 24 NFR** sur 7 stories (39-0 comprise).

### Modules touchés (fichiers communs, règles de travail à plusieurs)

Liste **tirée des fiches** (ici, de leur tableau en attendant qu'elles existent) ; à refaire à partir des fiches
quand elles seront écrites. Aucune migration : ni `crates/kesh-db/migrations/`, ni audit des migrations.

| Fiche | Crates | Frontend | Manuels |
|---|---|---|---|
| 39-1 | `kesh-providers` (wise), `kesh-core` (réglages), `kesh-api` (secrets, fabrique, test, transport), `kesh-i18n` | — | — |
| 39-2 | `kesh-providers` (wise::sca), `kesh-api` (route `sca-key`, secrets), `kesh-core`, `kesh-i18n` ; **Cargo.lock** (dépendance RSA) | — | — |
| 39-3 | `kesh-providers` (wise::statement) | — | — |
| 39-4 | `kesh-providers` (wise::classify), `kesh-core` (surcharges), `kesh-api` (`own_ibans`) | — | — |
| 39-5 | `kesh-providers` (wise::pdf), `kesh-api` (route, tâche, rangement), `kesh-db` (pièce par période), `kesh-i18n` | — | — |
| 39-6 | `kesh-i18n` | écrans connexion, clé SCA, *Relevés joints* ; E2E | `docs/manual` (`.tex` seulement, PDF à la release) |

**Partagés avec l'Epic 38** (socle : `kesh-providers`, secrets, fabrique, `ConnectorContext`, route `_test`,
38-10) : amendements additifs de C-39-2 — à voir avec l'Epic 38 avant la première ligne. **Fichiers communs** :
catalogues i18n des quatre locales (bloc propre à l'epic), `sprint-status.yaml` (bloc propre), `LIB_ROUTES`,
`sitesTotal`, `Cargo.lock` (compteurs recomptés après chaque reprise de `main`) ; `CHANGELOG.md` non écrit (notes de
version des fiches).

### Questions ouvertes

- **Q-39-1 — Relevés au jeton personnel (responsable, bloquant)** : la doc dit les relevés de solde **non servis** par l'API
  au jeton personnel hors US, CA, AU, NZ, SG, MY. Le jeton du responsable de l'epic les obtient-il (avec SCA) sur son compte suisse ?
  Sinon : accord de partenaire (OAuth), ou relevé CAMT.053 / CSV téléchargé à la main et importé — ce qui demande de
  lever le refus des imports de fichier sur un compte prestataire (38-3a, O9) et redécoupe l'epic.
- **Q-39-2 — Mécanisme de signature (responsable, recette)** : B10 décrit un défi signé par une clé privée dont la clé
  publique est déposée chez Wise ; aucune page consultée ne le documente (en-tête `X-Signature`, algorithme, format et
  taille de clé, nombre de clés déposables, expiration). À constater en 39-0 ; F3 et F4 en dépendent.
- **Q-39-3 — Dépréciation des jetons personnels (responsable de l'epic, Guy)** : annoncée « d'ici la fin de l'an prochain » pour les
  comptes partenaires « first party » ; les PME sont-elles visées ? Si oui, l'epic a une durée de vie bornée
  (C-39-7).
- **Q-39-4 — Virement sortant vers son propre compte (responsable)** : le relevé ne donne pas le compte destinataire d'un
  `TRANSFER` débit (champs lus) ; défaut `purchase` imputé par une règle (p. ex. `counterparty_contains` sur le nom),
  ou reconnaissance par un réglage (noms ou références de ses comptes) en `withdrawal` rattachable (34-11b) ?
- **Q-39-5 — Relevé PDF (responsable)** : mensuel automatique par défaut ou à la demande seulement ? `FLAT` ou `COMPACT` ?
  Langue ?
- **Q-39-6 — Jars, intérêts, cashback (responsable)** : utilise-t-il des soldes `SAVINGS`, des intérêts ou du cashback ?
  (classés `other` aujourd'hui, F9).
- **Q-39-7 — Historique (recette)** : jusqu'où Wise sert-il les relevés (`history_start`) ? Les livres 2024 se
  rattachent en lot (B17) si 2024 est servi.
- **Q-39-8 — IBAN des soldes Wise (responsable)** : les soldes CHF/EUR ont des coordonnées bancaires ; les garder sur le
  compte bancaire Kesh (facultatif, 38-2) sert-il (virements reçus à rapprocher) ?
- **Q-39-9 — Release (Guy)** : sans migration, l'Epic 39 peut partir avec la release de l'Epic 38 ou après ; numéro à
  Guy. Confirmation du numéro 39 (et 40 pour Stripe, Q9 de l'Epic 38).

### Règles de cet epic

- Méthode du dépôt (`CLAUDE.md`) : fiche → validation en boucle → développement → revue en boucle, gate local complet
  avant la PR ; splitting préventif (≤ 5 modules par story).
- **Production** : l'instance du responsable de l'epic n'est jamais écrite par le travail de cet epic ; la recette sur son compte Wise
  réel se fait sur une **instance jetable** ou hors de Kesh (39-0), avec un jeton qu'il peut révoquer.
- Aucun appel réseau en CI ni par un agent ; aucune réponse réelle non anonymisée, aucun jeton, aucune clé privée
  réelle dans le dépôt.
- On trace toujours : un défaut voisin non traité se note au registre (C-39-7), pour devenir une issue avec la PR.
- Travail à plusieurs (`CLAUDE.md` § *Travailler à plusieurs développeurs*) : chaque fiche porte une section **Note de
  version** (texte prêt à assembler à la release, juste avant les notes de développement), aucune ligne au
  `CHANGELOG.md` ; manuels : `.tex` seulement ; compteurs recomptés depuis la source après chaque reprise de `main`.

### Change Log

#### Alignement 2026-11-16

Alignement sur les règles de travail à plusieurs (Sonnet), fichier d'epic seul — les fiches n'existent pas encore et
attendent le test du responsable de l'epic sur l'API des relevés Wise (Q-39-1) :

- **PDF des manuels** : NFR-39-6-2 passe à « `.tex` seulement, PDF régénéré à la release » (1 remplacement).
- **CHANGELOG** : aucune ligne n'y était prévue ; règle « Note de version » des fiches ajoutée (§ *Règles de cet epic*,
  NFR-39-6-2).
- **Compteurs** : « recompté depuis la source après chaque reprise de `main` » ajouté à C-39-6, NFR-39-2-4, NFR-39-5-3
  (`LIB_ROUTES`, `sitesTotal`, `Cargo.lock`).
- **Fichier d'epic** : section « Modules touchés » ajoutée (tirée du tableau des stories, à refaire depuis les
  fiches) ; ligne « Release : un epic = une release ; 7 stories » ajoutée, qui **dépasse la borne de six** — admis
  pour une fonctionnalité nouvelle si on le dit au kickoff, repli de fusion décrit.
- Rappel en tête : les fiches attendent la recette de 39-0 (Q-39-1).
- 2026-11-20 — Validation P5 ciblée (Claude Sonnet 5.5, passe ciblée, contexte frais, rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11-P5-ciblee.md`) : 0 C/H/M, 2 LOW remédiés — G4, 39-11b AC-5 et 39-11c AC-6 disent la même chose (l'import juge à son instant, la revue relit la même règle sur l'état présent) ; 39-11c AC-3 : 40-7c n'écrit le texte qu'en dernier recours.
- 2026-11-20 — Validation P1 de la phase 2 (Claude Sonnet 5.5, contexte frais, rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-12-P1.md`) : **3 HIGH, 11 MEDIUM, 8 LOW, remédiés**. Recette 1 de Wise : défi SCA constaté, signature intégrée (H4, H5, `KESH_WISE_<n>_SCA_KEY_FILE`), 39-12a scindée en 39-12a1 (client, SCA) et 39-12a2 (route, import), 39-2 `superseded` ; socle commun S1 à S9 avec les Epics 38 et 40 ; lecture complète avant écriture, aucune pause fixe, politique de reprise unique, codes `PROVIDER_*`, sessions seules, `providerApi`.
