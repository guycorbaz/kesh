# Story 15.5e1 : Socle du rejeu sur interblocage — les enveloppes, le registre, les trois routes des issues

Status: ready-for-dev

<!-- Sous-story de la 15-5e (fiche index `15-5e-ordre-des-verrous-reglements.md`, statut `split`),
     créée le 2026-10-08 par le découpage décidé à la validation P3 de la 15-5e (finding F3-2, choix
     C61) : patron « story-zéro + rollout » de la § Règle de splitting préventif. Cette fiche est la
     story-zéro : elle pose le patron (deux enveloppes nommées, journalisation, registre) et
     l'applique aux trois routes des issues ; la 15-5e2 (`15-5e2-rejeu-des-autres-flux.md`) en fait
     le rollout. Le contenu vient de la 15-5e validée P1 à P3 (commit `94f1365e` pour la version
     d'avant découpage) ; les remédiations de la P3 sont appliquées ici et dans la 15-5e2.
     Choix applicables : C43 (pour le seul AC5 « saisie fournisseur »), C54, C55, C56 (révisé par
     C61 et C63), C57, C58, C59 (révisé par C62), C61, C62, C63, C64, C65.
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation. Le
     développement attend la clôture de la boucle de validation ET le merge des 15-5a et 15-5b. -->

**Issues** : la PR porte **`closes #463`** et **`closes #491`** (mots-clés **dans la PR** : le dépôt
merge en squash), **`refs #536`** et **`refs #429`**.
- **#463** — l'annulation d'un règlement client (`POST /api/v1/invoices/{id}/settlements/{sid}/cancel`)
  rend 500 sur interblocage, sans rejeu : rejouée ici (AC3), test où elle est la victime (AC4).
- **#491** — le règlement manuel (`POST /api/v1/invoices/{id}/settlements`) victime d'un interblocage
  avec un rapprochement n'est pas rejoué : rejoué ici (AC3), test (AC4).
- **#536** (`refs`) — interblocage entre l'acceptation d'un lot de rapprochement et la validation d'une
  facture. L'acceptation **rejoue déjà** (`post_accept`, `reconciliation.rs:850`, Story 25-4-c2, #480 —
  prédicat 1213 **et** `ReconciliationTransactionAborted`) ; la **validation** est rejouée ici (AC3,
  test 4 de l'AC4) : le cas principal de l'issue est couvert. L'issue cite une seconde inversion,
  l'**avoir** d'une facture arrondie : sa route est rejouée par la **15-5e2**, qui porte `closes #536`
  (choix C57).
- **#429** (`refs`) — fermée par la 15-5d, qui s'appuie sur cette story (ci-dessous).

**Dépend de la 15-5a et de la 15-5b.**
- La **15-5a** réécrit la boucle des comptes de charge de `supplier_invoices::create_in_tx` en **deux
  passes**, forme puis comptes (son AC4, choix C22) : l'AC5 place l'appel des réglages entre elles.
- La **15-5b** réécrit des commentaires de `invoice_settlements_write.rs` (choix C23) : l'AC5 réécrit le
  commentaire « 5 bis » du même fichier ; merger après elle évite le conflit.
Ne pas commencer avant le merge de la 15-5b.

**Passe avant la 15-5e2 et avant la 15-5d** (choix C65).
- La **15-5e2** fait le rollout du patron que cette story pose.
- La **15-5d** dépend de **cette story, et pas du rollout** : elle s'appuie sur le rejeu de la
  **validation** (AC3), l'avance des réglages de la saisie fournisseur, le doc-comment canonique de
  `validate_invoice` (numérotation **(2 bis')**) et le commentaire « 5 bis » (AC5) — tous posés ici.
  Le rejeu de la **saisie fournisseur** et de la **complétion d'une facture importée** vient avec la
  15-5e2, dans un ordre de merge libre par rapport à la 15-5d : si la 15-5d merge la première, ces
  deux routes gagnent ses verrous sans être encore rejouées, et une victime y rend `500` comme
  aujourd'hui jusqu'au merge de la 15-5e2 — une fenêtre de même nature que l'état actuel, pas une
  régression de nature.

## Story

En tant que **comptable d'une société tenue dans Kesh**,
je veux qu'**un règlement client, l'annulation d'un règlement client ou la validation d'une facture
victime d'un interblocage avec une autre opération légitime soit rejoué par le serveur, sans que je le
voie**,
afin de **ne plus recevoir d'erreur 500 quand ces opérations croisent un rapprochement bancaire** — et
que le projet dispose du patron (enveloppes, journalisation, registre) que la 15-5e2 étend à toutes les
routes qui écrivent au journal.

### Pourquoi le rejeu, et non un ordre des verrous (choix C54)

Tout flux qui écrit au journal insère des lignes `journal_entry_lines` ; la clé étrangère
**`fk_jel_account`** (`crates/kesh-db/migrations/20260412000001_journal_entries.sql:45`) fait poser par
InnoDB un **verrou partagé sur chaque ligne `accounts` écrite**, à l'insertion — donc **après** le
verrou de l'exercice (`journal_entries::create_in_tx`, exercice `:279`, lignes ensuite). Un flux qui
tient un compte `FOR UPDATE` en attendant l'exercice forme donc un cycle avec **tout** flux qui tient
l'exercice et écrit une ligne sur ce compte (finding **F1-1 HIGH** de la P1 de la 15-5e). Aucun ordre
« comptes avant exercice » ne tient de bout en bout ; les passes de validation des 15-5d, 15-5e, 15-6a
et 15-6b ont chacune trouvé un cycle de plus.

Ce mécanisme n'est pas une hypothèse : le test `accept_replays_the_batch_when_it_is_the_deadlock_victim`
(`crates/kesh-api/tests/reconciliation_e2e.rs:4426`) monte son interblocage déterministe **sur ce verrou
de clé étrangère** (vers `invoices`, à l'insertion du règlement), et InnoDB y annule la victime
aussitôt, sans attendre `innodb_lock_wait_timeout`.

D'où la règle : **l'ordre des verrous réduit la fréquence des interblocages ; le rejeu les rend
invisibles à l'utilisateur.**

## Acceptance Criteria

### Le patron

1. **AC1 — Un inventaire fermé des routes qui écrivent au journal, classé dans le registre.**
   L'inventaire est la table « Les routes qui écrivent au journal » des Dev Notes (relevée sur
   `b80ab8c0`, complétée sur `c734645b`) : **21 routes** de `lib.rs` qui écrivent dans
   `journal_entries` — **4 déjà rejouées**, **3 rejouées par cette story**, **14 rejouées par la
   15-5e2** — et **4 routes exemptées**, raison écrite : 2 de `lib.rs` (restauration d'instance,
   effacement de la démo) et les 2 routes de harnais `/api/v1/_test/seed` et `/api/v1/_test/reset`
   (`test_endpoints.rs:49-50`, montées par `nest()` en mode test seulement, `lib.rs:1056-1065`, qui
   vident tout par `test_fixtures::truncate_all`). Il part du **symptôme**, pas d'une liste de routes :
   des primitives qui écrivent au journal (`journal_entries::create`, `create_in_tx`,
   `create_opening_entry`, `delete_by_id`, `delete_in_tx`, `reverse`, `reverse_in_tx`,
   `reverse_owned_in_tx`, et tout `INSERT INTO journal_entr` / `DELETE FROM journal_entr` hors tests)
   **et** du SQL dont le **nom de table est dynamique** — `TRUNCATE`, `format!` qui construit un
   `DELETE FROM` / `INSERT INTO` / `UPDATE` (`backup.rs:457`, `:509`, `test_fixtures.rs:401`), les
   aides qui vident tout (`truncate_all`, `restore_tables_in_tx`, `reset_demo`, toute boucle sur
   `TABLES_TO_TRUNCATE`, `backup.rs:63`) —, remontés jusqu'aux handlers (commandes en Dev Notes ;
   finding F2-2). T0 refait la remontée sur `HEAD` ; **une route qui écrit au journal absente de la
   table bloque la story**. L'inventaire est **gardé dès cette story** par le registre testé (AC4,
   test 5) : une route mutante ajoutée demain sans examen fait rougir le gate.
2. **AC2 — La forme du rejeu : deux enveloppes nommées, une transaction neuve par tentative**
   (choix C56, C62). Les noms sont **fixés ici** (finding R3-3) — le registre les cherche par texte.
   - **Enveloppe `DbError`** : **`kesh_db::retry::retry_on_deadlock(operation, f)`**
     (`crates/kesh-db/src/retry.rs:105`, qui existe et n'est appelée par aucune route aujourd'hui), et
     sa variante **`retry_on_deadlock_with(operation, max_attempts, f)`** — pour toute route dont
     l'écriture est **une** fonction de dépôt qui ouvre et conclut sa propre transaction
     (`pool.begin()` … `commit()`) et rend `Result<_, DbError>`. La conversion en `AppError` (et toute
     correspondance propre à la route) se fait **après** le rejeu, jamais dans la fermeture.
   - **Enveloppe `AppError`** : module neuf **`crates/kesh-api/src/retry.rs`**, déclaré **`pub mod
     retry;`** dans `lib.rs` (à côté des autres `pub mod`, `lib.rs:9-24`) ; fonction
     **`kesh_api::retry::retry_app_on_deadlock(operation, f)`** ; prédicat **exposé** sous son propre
     nom, **`kesh_api::retry::is_app_deadlock(err: &AppError) -> bool`** (`AppError::Database(db) if
     kesh_db::retry::is_deadlock_error(db)`), que l'enveloppe emploie et que le test 1 appelle depuis
     `crates/kesh-api/tests/`. Doc-comment du module : renvoi à cette story, la règle de l'AC2. Elle
     sert les routes dont la transaction est ouverte **dans le handler** (`complete_import`,
     `post_manual`, `post_split`, `post_cancel_reconciliation`) : **ses appelants de production
     arrivent avec la 15-5e2** — d'ici là, fonction `pub` d'une bibliothèque (aucun avertissement
     `dead_code`), prouvée par le test 1 et par les tests de `retry_with` qu'elle appelle.
   - **Le nom d'opération est porté par l'événement lui-même** (finding F3-3, choix C62, révise C59).
     `retry_with` gagne un **premier paramètre** `operation: &'static str` (p. ex.
     `"invoices::settle"`), et son `tracing::debug!` (`retry.rs:160`) devient un
     **`tracing::warn!(target: "kesh_db::retry", operation, attempt, max_attempts, backoff_ms, …)`** :
     la ligne dit **quelle route** a été rejouée même sous un filtre `RUST_LOG=warn` — un span `info`
     y serait désactivé et l'événement perdrait son nom (le serveur n'a pas de span de requête, aucun
     `TraceLayer`). Les deux enveloppes transmettent le nom. Changement de signature, **et ce qu'il
     touche, en entier** (finding R3-2) : les **six appels de tests** de `retry.rs` (`:189`, `:205`,
     `:225`, `:241`, `:318`, `:348`) gagnent un nom, assertions inchangées ; les **cinq sites directs**
     de `retry_with` gagnent le leur — `post_accept` (`reconciliation.rs:850`,
     `"reconciliation::accept"`), `post_cancel_reconciliation` (`:3843`, `"reconciliation::cancel"`),
     `write_off_invoice_handler` (`invoices.rs:1343`, `"invoices::write_off"`),
     `complete_opening_balances` (`opening_balances.rs:591`, `"opening_balances::complete"`),
     `onboarding::finalize` (`onboarding.rs:614`, `"onboarding::finalize"`) ; **rien d'autre** dans
     ces cinq sites (leur passage aux enveloppes est le rollout de la 15-5e2). Un 1213 épuisé reste
     journalisé comme aujourd'hui, en erreur, par le chemin du 500.
   - **Une tentative = une transaction neuve** : la fermeture rejouée commence par `pool.begin()` (ou
     appelle une fonction qui le fait) et finit par le `commit()`. Restent **hors** de la fermeture les
     contrôles qui **ne dépendent pas** de la transaction (forme du corps, lectures hors transaction,
     préchargement de la société) et ce qui suit le commit (relecture de la réponse). Les contrôles qui
     lisent ce que la transaction verrouille **restent dans la tentative, dans le même ordre** — aucun
     refus ne se déplace (AC7 ; finding R3-4 : `complete_import` contrôle devise, QR-IBAN et montant
     sur le `staging` lu `FOR UPDATE`, `cancel_reconciliation_once` lit `bank_account_id` avant son
     `begin`, `reconciliation.rs:3858-3870` — cas de la 15-5e2, règle posée ici). **Aucun effet de bord
     hors transaction** dans la fermeture — ni e-mail, ni fichier, ni appel réseau : vérifié pour les
     21 routes au relevé de `b80ab8c0` (Dev Notes), **revérifié en T0** pour les trois routes d'ici.
   - **Les entrées consommées se clonent par tentative** (finding F3-8) : les enveloppes prennent une
     fermeture `Fn`. Ce que le handler consomme par déplacement (le corps `Json(req)`, un `New…`
     construit à partir de lui) est cloné **dans** la fermeture, à chaque tentative — patron de
     `onboarding::finalize` (`onboarding.rs:606-624`, « les captures clonées garantissent que la
     closure est `Fn` et non `FnOnce` »). Les types concernés sont `Clone` (vérifié en P3 pour
     `NewSupplierInvoice`, `NewJournalEntry` ; T0 le revérifie pour les corps des trois routes). **Pas
     de `Option::take`** ni de `FnOnce` contourné.
   - **Le 1213 doit atteindre le prédicat** : sur le chemin de chaque route, une erreur sqlx passe par
     `map_db_error` (`crates/kesh-db/src/errors.rs:796`), qui laisse le 1213 en `DbError::Sqlx` ; un
     `map_err` qui convertirait une erreur sqlx en une autre variante rendrait le rejeu **muet** (le
     motif est écrit au doc-comment de `post_cancel_reconciliation`). Relevé sur `b80ab8c0` : aucun sur
     les chemins de l'inventaire (Dev Notes) ; T0 le refait pour les trois routes d'ici.
   - **Ce qui ne change pas** : trois tentatives (`DEFAULT_MAX_DEADLOCK_ATTEMPTS`), backoff 50 puis
     100 ms. Au-delà, le 1213 sort en `500 INTERNAL_ERROR`, comme aujourd'hui — rien n'a été écrit, la
     requête peut être renvoyée telle quelle.
3. **AC3 — Les trois routes des issues, rejouées par l'enveloppe `DbError`** (détail, fonction de dépôt
   et nom d'opération : table des Dev Notes) : **validation d'une facture** (`validate_invoice_handler`,
   `"invoices::validate"` — #536), **règlement client** (`settle_invoice_handler`,
   `"invoices::settle"` — #491) et **son annulation** (`cancel_invoice_settlement_handler`,
   `"invoices::cancel_settlement"` — #463). Chacun de leurs doc-comments dit, en une ligne, que la
   route est rejouée sur interblocage, avec renvoi à l'enveloppe (pas de recopie de la règle).

### Ce qui doit être prouvé

4. **AC4 — Tests.** Fichier neuf `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (tests 1 à 4) ;
   le test 5 **étend le registre existant** `crates/kesh-api/tests/audit_route_registry.rs` (choix
   C58), pas de fichier neuf.
   **Patron des interblocages** : celui de `accept_replays_the_batch_when_it_is_the_deadlock_victim`
   (`reconciliation_e2e.rs:4426`) — une transaction de test **alourdie** (table de lest, 500 lignes
   insérées, `seq_1_to_50` ; InnoDB choisit pour victime la transaction la plus légère), qui tient la
   ressource que la route demandera **en second** ; la route lancée en tâche, vue en attente par
   `kesh_db::test_fixtures::attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs:560`,
   motifs écrits au test) ; puis la transaction de test demande la ressource que la route tient
   **déjà** — cycle, InnoDB annule la route — et **doit l'obtenir** (`.expect`, sinon c'est elle la
   victime et le test ne prouve rien) ; elle annule ; la route rejoue et réussit.
   1. **Le prédicat, sur une vraie 1213** — deux connexions dédiées (`pool.acquire()`, pas de
      `sleep` : le montage ci-dessus, sur une table sentinelle) ; l'erreur de la victime, passée par
      `map_db_error`, est reconnue par `kesh_db::retry::is_deadlock_error`, et, convertie en
      `AppError`, par `kesh_api::retry::is_app_deadlock`. **Négatifs** : un 1205 (`SET SESSION
      innodb_lock_wait_timeout = 1` sur une connexion qui attend une ligne tenue) et une erreur métier
      (`DbError::OptimisticLockConflict`) ne sont **pas** reconnus. Ce test garde aussi `map_db_error` :
      si un jour le 1213 y devenait une variante typée, il rougit.
      **Montage commun aux tests 2 à 4** (findings R2-9, F2-9) : les factures sont créées **par les
      routes** (`POST /api/v1/invoices`, puis `POST /api/v1/invoices/{id}/validate` pour les tests 2
      et 3), par une aide locale au fichier neuf — **pas de troisième copie** de
      `seed_validated_invoice`, déjà recopiée dans `reconciliation_e2e.rs:380` et
      `invoice_pdf_e2e.rs:175` ; la facture est **datée du jour** et son montant est **un multiple de
      5 centimes** (aucun écart d'arrondi : sans cela la validation et le règlement prennent aussi le
      compte d'arrondi, `invoices.rs:2065`, `invoice_settlements_write.rs:184-196`, et le motif
      d'attente change) ; l'exercice tenu par la transaction de test est l'exercice **ouvert qui
      couvre aujourd'hui**, faute de quoi la route refuse avant d'attendre et
      `attendre_une_requete_en_cours` panique.
   2. **Règlement client victime (#491)** — facture validée ; règlement **par compte interne** (le
      plus court : le virement exigerait un compte bancaire configuré, `invoice_settlements_write.rs:119-131`),
      sur un compte actif imputable du plan semé ; la transaction de test tient l'**exercice** ;
      `POST /api/v1/invoices/{id}/settlements` vu en attente sur `fiscal_years` + `FOR UPDATE` (il
      tient la facture, étape (1) de `settle_invoice`, et le compte interne, `:156-167`) ; la
      transaction de test demande la **facture** `FOR UPDATE` → la route est la victime ; après
      annulation : **200**, **un seul** règlement, une seule écriture.
   3. **Annulation d'un règlement victime (#463)** — même montage sur `POST
      /api/v1/invoices/{id}/settlements/{sid}/cancel` (facture, puis exercice **du jour** par la
      contre-passation : le règlement a été posé dans ce même exercice ouvert) : **200**, le
      règlement retiré une fois, **une seule** contre-passation.
   4. **Validation victime (#536)** — facture brouillon créée par `POST /api/v1/invoices`, sans
      écart d'arrondi ; la transaction de test tient l'**exercice** ; `POST
      /api/v1/invoices/{id}/validate` vu en attente sur l'exercice (il tient la facture et la ligne
      des réglages) ; la transaction de test demande la **facture** → victime ; **200**, facture
      `validated`, **une seule** écriture de vente.
   5. **Le registre des routes** — une **seconde colonne de statut** dans le registre existant
      `crates/kesh-api/tests/audit_route_registry.rs` (choix C58, C63) : chaque ligne de `LIB_ROUTES`
      et de `TEST_ENDPOINT_ROUTES` gagne un statut de rejeu **`Rejouee`**, **`ARejouer("15-5e2")`**
      (statut transitoire, retiré par la 15-5e2), **`SansEcritureAuJournal`** ou
      **`Exemptee("raison")`**, à côté de son statut d'audit. Une route ajoutée demain s'examine
      **une fois**, pour les deux propriétés ; l'extracteur (`extract_counted`, `:197`), les deux
      volets `lib.rs` (`:221`) et `test_endpoints.rs` (`:312`) et la garde du troisième fichier
      (`no_third_route_file_escapes_the_registry`, `:404`) servent tels quels — **ni copie de
      l'extracteur, ni second registre**. Les volets (a) route mutante absente du registre et (b)
      entrée absente du code sont **déjà** ceux du registre ; s'ajoutent :
      - **(c) un test neuf qui échoue sur une route `Rejouee` dont le handler n'appelle aucune
        enveloppe** (finding F3-1). Il est textuel, et ses trois précautions sont **exigées** —
        sans elles, la fiche fabriquerait ses propres faux verts : le doc-comment d'un handler précède
        son `pub async fn`, il tombe donc dans la fenêtre de la route **précédente**, et l'AC3 exige
        justement que ce doc-comment nomme l'enveloppe (`validate_invoice_handler`,
        `invoices.rs:869`, est suivi d'une struct puis du doc-comment de
        `unvalidate_invoice_handler`, `:911`) :
        1. les **commentaires sont retirés** du source (`//`, `///`, `//!`, `/* … */`) **avant**
           toute recherche ;
        2. la fenêtre d'un handler va de son `pub async fn <nom>` **au premier attribut, doc-comment
           ou item qui suit son corps** (accolades équilibrées sur le source décommenté), dans
           `crates/kesh-api/src/routes/<module>.rs` ;
        3. on y cherche **`nom(`** ou **`nom::<`** pour `retry_on_deadlock`,
           `retry_on_deadlock_with`, `retry_app_on_deadlock` et `retry_with` — pas le nom nu.
        Ces trois précautions sont elles-mêmes **testées sur un source synthétique** écrit au test :
        un handler sans enveloppe suivi d'un doc-comment et d'un handler qui la nomment, un nom
        d'enveloppe en commentaire dans le corps, un nom d'enveloppe sans parenthèse — la fonction
        d'extraction rend « absente » dans les trois cas. **Dans cette story**, `retry_with` est
        accepté pour toute route `Rejouee` (les quatre sites rejoués aujourd'hui l'appellent
        directement) ; la 15-5e2 restreint cette acceptation à `post_accept`.
      - **(d) la partition de rejeu recomptée depuis la source** (patron
        `the_registry_partition_is_what_the_story_declares`, `:453`), **avec son total** (finding
        F3-6) : **7** `Rejouee` (les 4 rejouées aujourd'hui + les 3 de l'AC3), **14** `ARejouer`,
        **4** `Exemptee` (`admin::full_import`, `onboarding::reset`, `/seed`, `/reset` — ces deux
        dernières avec la raison « mode test, jamais monté en production »), **90**
        `SansEcritureAuJournal` (89 de `LIB_ROUTES`, plus `/password-reset-token` de
        `TEST_ENDPOINT_ROUTES`, `test_endpoints.rs:53`, qui n'écrit pas au journal — nommée pour
        qu'on ne la prenne pas pour un oubli, finding R3-6) ; **total 115** = `LIB_ROUTES` (112) +
        `TEST_ENDPOINT_ROUTES` (3), assertion de somme comprise.
      ⚠️ Le doc-comment du module, qui dit déjà ce que le registre n'établit **pas** pour l'audit, le
      dit pour le rejeu : (i) qu'une route classée `SansEcritureAuJournal` n'écrit pas au journal
      (l'écriture se fait au dépôt, parfois à trois appels du handler) — c'est la classification de
      l'inventaire de l'AC1, recontrôlée en revue ; (ii) que le 1213 **atteint** le prédicat : le
      volet (c) est textuel, une conversion faite dans la fermeture le passerait (finding F2-10) —
      seuls les tests 2 à 4 le prouvent dynamiquement, pour trois routes ; pour les autres, c'est la
      revue fichier par fichier ; (iii) que le volet (c) ne voit pas une enveloppe appelée par une
      fonction auxiliaire du handler (le nom doit figurer dans le corps du handler lui-même : c'est la
      forme exigée) ; (iv) **l'angle mort assumé** (finding F2-5) : une route
      `SansEcritureAuJournal` qui prend un verrou peut être la **victime** d'un cycle avec un flux qui
      écrit au journal, et elle n'est pas rejouée — p. ex. `accept_batch` tient un verrou partagé sur
      la ligne `companies` dès qu'une proposition a inséré son écriture (`fk_journal_entries_company`),
      puis demande la sentinelle `FOR UPDATE` de cette ligne pour une règle à projet
      (`reconciliation.rs:2411`) ; une route qui attend entre-temps un verrou exclusif sur `companies`
      (`companies::lock_books`, `unlock_books`, la sentinelle prise par les routes de `projects`,
      `bank_accounts`, `vat`, `dunning_levels`) ferme le cycle et, plus légère, rend 500. Rare, et hors
      de la promesse (« les routes qui écrivent au journal ») ; le registre établit que **toute route
      mutante a été examinée**. Au passage, l'en-tête du module, qui annonce **111 / 114** routes
      (`:20`, `:24-25`) quand les assertions disent **112 / 115**, est corrigé (finding R3-6).
   - **Aucun test de « non-interblocage »** : un test qui asserte que deux opérations concurrentes
     « réussissent toutes deux » ne prouve rien sous un ordre fautif (finding F1-4 de la P1), et
     l'absence de cycle n'est plus affirmée.
   - Le test existant `accept_replays_the_batch_when_it_is_the_deadlock_victim` reste vert sans
     modification.
   - **Mutations** (consignées au Dev Agent Record, en touchant le fichier après restauration — sans
     quoi cargo garde le binaire muté), **sur plusieurs familles** (finding F3-1) :
     - retirer l'enveloppe de `settle_invoice_handler`, de `cancel_invoice_settlement_handler`, de
       `validate_invoice_handler` → le test 2, 3, 4 rend 500 et rougit, **et** le volet (c) rougit
       (pour `validate_invoice_handler` : suivi du doc-comment de la dévalidation — le retrait des
       commentaires est ce qui est exercé) ;
     - famille `retry_with` direct : retirer le `retry_with` de `post_cancel_reconciliation` → le
       volet (c) rougit ;
     - écrire le nom de l'enveloppe **en commentaire** dans le corps d'un handler `Rejouee` dont on a
       retiré l'appel → le volet (c) rougit toujours ;
     - faire reconnaître le 1205 par le prédicat → le test 1 rougit.

### L'ordre des verrous : ce que la 15-5d attend de cette story (choix C54, C55, C65)

5. **AC5 — Le doc-comment canonique, l'avance des réglages de la saisie fournisseur, « 5 bis » et le
   module `retry.rs` disent la règle vraie ; aucun ne prétend à l'absence de cycle.** Les autres
   commentaires d'ordre, le Pattern 5 et l'inventaire au symptôme sont à la **15-5e2**.
   - **Le doc-comment canonique** « # Ordre des locks » de `validate_invoice`
     (`crates/kesh-db/src/repositories/invoices.rs:1916-1929`) est réécrit. Il dit **l'ordre réel**,
     numéroté **comme les étapes du code** — la numérotation est fixée ici, et la 15-5d y renvoie
     (finding R2-1 : l'actuel « 1 bis. `accounts` » disparaît, « 1 bis » étant dans le code le
     projet) :
     ```
     (1)        invoices            FOR UPDATE sur la facture                       (:1953)
     (1 bis)    projects            lu SANS verrou (finding F1-5)                   (:1975-1990)
     (2)        company_invoice_settings  FOR UPDATE (get_or_create_default_in_tx) (:2004-2006)
     (2 bis')   accounts            compte de différences d'arrondi, FOR UPDATE,
                                    seulement s'il y a un écart                     (:2033, :2065)
     (2 quater) accounts            verrou PARTAGÉ sur le compte de produit par la
                                    clé étrangère de invoice_lines.revenue_account_id,
                                    SEULEMENT si une ligne n'a pas encore de compte
                                    de produit (UPDATE invoice_lines de
                                    matérialisation)                                (:2113, :2141-2148)
     (3)        fiscal_years        find_open_covering_date                          (:2171-2172)
     (4)-(5)    invoice_number_sequences  le compteur                               (:2176)
     (7)        journal_entries     create_in_tx — l'insertion reprend des verrous
                                    PARTAGÉS : companies (fk_journal_entries_company),
                                    chaque compte écrit (fk_jel_account), chaque
                                    projet tagué (fk_jel_project)                   (:2241)
     ```
     (numéros de ligne relevés sur `c734645b`, refaits en T0 ; l'étape (2 ter) ne prend aucun verrou,
     findings R2-6, F2-1 ; la condition de (2 quater) est celle du code, `if lines_before.iter().any(|l|
     l.revenue_account_id.is_none())`, `:2141-2148` — finding F3-7). La **15-5d** ajoute ses comptes
     désignés (créance, TVA due) **à l'étape (2 bis')**, après le compte d'arrondi et avant (3). Il dit
     **la règle** : cet ordre est une **convention qui réduit la fréquence** des interblocages entre
     flux qui la suivent ; il ne peut pas les exclure, puisque l'insertion des lignes reprend
     `accounts` **après** l'exercice ; **la défense est le rejeu des routes** (renvoi à l'enveloppe,
     AC2). Les phrases « Aucun chemin ne verrouille `accounts` avant `invoices` ou `fiscal_years` »
     (`:1921-1922`, fausse : le règlement client et le solde du reste le font) et « **Toute divergence
     de cet ordre = risque de deadlock** » (`:1928`) sont retirées. **Ne pas promettre la
     sérialisation par la ligne des réglages** (finding R3-7, non mesuré) : `get_or_create_default_in_tx`
     fait `INSERT IGNORE` **puis** `SELECT … FOR UPDATE` (`company_invoice_settings.rs:102-109`) ; sur
     une ligne déjà présente, l'`INSERT IGNORE` en doublon peut poser un verrou **partagé**, et deux
     transactions de la même société qui le tiennent puis demandent l'exclusif s'interbloquent au lieu
     de se sérialiser. Le texte dit donc « la ligne des réglages, tenue `FOR UPDATE` jusqu'au commit,
     ordonne la plupart des flux d'une même société ; un interblocage entre eux reste possible et il
     est rejoué » — jamais « sérialise ».
   - **Saisie fournisseur : les réglages avant les comptes de charge** (seul réordonnancement gardé,
     choix C55). `supplier_invoices::create_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs:248`
     — donc aussi la complétion d'une facture importée, `imported_supplier_invoices.rs:243`) charge
     aujourd'hui les réglages (`get_or_create_default_in_tx`, `:357-359`) **après** l'exercice
     (`:352-355`). Désormais l'appel se place **après la passe de forme et avant la passe des comptes**
     de la 15-5a (finding F1-7 : un refus de forme ne prend ainsi aucun verrou) : projet (`:277-284`)
     → fournisseur (lecture, `:286-299`) → forme des lignes → **réglages** → comptes de charge →
     exercice → génération.
     - **Aucun refus ne bouge** : `get_or_create_default_in_tx` ne refuse rien (hors erreur de base) ;
       l'exigence `ConfigurationRequired("default_payable_account_id")` reste **à sa place**, après
       l'exercice — seul l'**appel** est avancé.
     - **Pourquoi celui-là** : il ne coûte qu'un déplacement d'appel ; il rend l'ordre « réglages →
       exercice » commun à la saisie et à la validation (l'inversion F2-8 de la 15-5d) ; et la
       **15-5d** en a besoin pour verrouiller ses candidats avant l'exercice. La ligne des réglages
       ordonne la plupart des saisies d'une même société — sans les sérialiser à coup sûr (R3-7,
       ci-dessus) ; le règlement client et fournisseur n'y passent pas.
     - **Pas de tri des comptes de charge** (finding R1-3, choix C55) : le cycle qu'il viserait, deux
       saisies aux comptes croisés, est rendu rare par la ligne des réglages ; celui qui reste — une
       saisie dont un compte de charge est le compte d'arrondi, contre un règlement par compte interne
       avec écart — ne dépend pas de l'ordre des comptes de la saisie. Le rejeu de la route (15-5e2)
       couvre l'un et l'autre ; c'est écrit au doc-comment.
     - Le doc-comment de `create_in_tx` écrit son ordre de verrous et renvoie à `validate_invoice`
       pour la règle.
   - **Le commentaire « 5 bis »** de `write_off_invoice` (`invoice_settlements_write.rs:463-470`) est
     réécrit **en place, sans changer le code** : il affirme « aucun autre chemin ne prend de verrou
     `FOR UPDATE` sur le compte de TVA due […] si bien qu'aucun cycle n'est connu » — **faux** : le
     règlement client par compte interne verrouille le compte qu'on lui désigne, TVA due comprise
     (`:156-167`). Le texte neuf dit l'ordre que prend le flux (compte de la nature `:433`, puis
     arrondi `:475`, puis TVA due `:489`, puis exercice `:493` — finding R2-7), que l'insertion des
     lignes reprend ces comptes après l'exercice, et que la route est rejouée (elle l'est déjà,
     `invoices.rs:1343`).
   - **Le doc-comment du module `crates/kesh-db/src/retry.rs`** : trois prémisses fausses voisines
     (finding R2-5) sont réécrites — `:7-9` « MariaDB ne détecte pas les deadlocks cross-table avant
     `innodb_lock_wait_timeout` » (InnoDB détecte le cycle à l'attente et annule aussitôt une victime,
     1213, ce que les tests de l'AC4 et celui de `reconciliation_e2e.rs:4426` exercent) ; `:11-13`
     « rollback de la tx la plus jeune » (InnoDB annule la transaction la plus **légère** — lignes
     modifiées et verrous tenus —, ce sur quoi repose le montage des tests de l'AC4) ; `:33-36`
     « Acceptable vs. un 500 sur `innodb_lock_wait_timeout` (50 s) » (un interblocage n'attend pas
     50 s ; l'alternative au rejeu est un 500 immédiat). Ce qui vaut pour `innodb_lock_wait_timeout`
     (1205, non rejoué, `:67-70`) est gardé. Le texte neuf dit aussi **ce qu'InnoDB ne détecte pas**
     (finding F2-7) : un cycle qui passe par un verrou nommé `GET_LOCK`
     (`kesh-reconciliation/src/mutex.rs:89`) et un verrou de ligne n'est pas vu — il finit en 1205 ou
     en `GET_LOCK = 0` (409 `RECONCILIATION_ACCOUNT_LOCKED`), jamais rejoué. Aucun ne se forme
     aujourd'hui parce que **chaque flux de rapprochement prend son `GET_LOCK` avant tout verrou de
     ligne** : invariant écrit, à garder. Et le rejeu suppose `innodb_deadlock_detect` à sa valeur par
     défaut (`ON`) : désactivé, chaque 1213 devient un 1205 que rien ne rejoue (le manuel admin le dira,
     15-5e2). La phrase « Cf. Pattern 5 pour la doc des ordres de locks canoniques » renvoie à la
     convention de fréquence que la 15-5e2 y écrit.

### Documentation, et ce qui ne change pas

6. **AC6 — CHANGELOG** (choix C64 : un correctif qui ferme deux issues porte sa ligne dans la PR qui
   les ferme — règle d'inclusion du `CLAUDE.md`), rubrique **Corrigé** de la section `## [0.13.0] —
   Non publié` (créée par la 15-5a ; **la créer en tête si absente** — motif exact exigé par
   `scripts/prepare-release.sh:189`) : « Un règlement client, l'annulation d'un règlement client et
   la validation d'une facture qui croisent une autre opération (un rapprochement bancaire,
   notamment) ne finissent plus en erreur interne : le serveur rejoue l'opération, comme il le
   faisait déjà pour l'acceptation d'un rapprochement. Seul un interblocage répété trois fois de
   suite rend encore une erreur ; rien n'est alors écrit et l'opération peut être relancée telle
   quelle (#463, #491, #536). » La 15-5e2 étend cette ligne aux autres opérations. **Pas d'autre
   document** dans cette story : `docs/api-external.md` § 10, les manuels (#484) et le Pattern 5
   sont à la 15-5e2 ; aucun texte visible de l'utilisateur ne décrit aujourd'hui le 500 des trois
   routes (`grep -nE "INTERNAL_ERROR|interblocage|deadlock" docs/api-external.md website/*.html
   README.md` sur `94f1365e` : seules `api-external.md:307` et `:313`, propres au rapprochement, qui
   restent vraies ; relevé des manuels de la 15-5e2 : rien sur ces trois routes).
7. **AC7 — Aucune règle métier ne change, et c'est vérifié.** Aucun refus neuf, aucune priorité de
   refus déplacée, aucune variante de `DbError`, aucun code d'erreur, aucune clé i18n, aucun écran ;
   les écritures produites sont identiques ; la suite existante reste verte **sans modification de ses
   assertions** (les six appels de tests de `retry.rs` gagnent un nom, AC2).

## Tasks / Subtasks

- [ ] **T0 — Refaire les relevés** sur `HEAD`, **après le merge de la 15-5b** : la remontée de l'AC1
      (commandes en Dev Notes), comparée à la table — **une route absente bloque la story** ; pour les
      trois routes de l'AC3, le chemin d'erreur (aucun `map_err` qui ne passe pas par
      `map_db_error`), l'absence d'effet de bord hors transaction et les entrées à cloner (AC2) ;
      numéros de ligne de `invoices.rs:1916-1929` et des étapes (1)–(7) de l'AC5,
      `invoice_settlements_write.rs:463-470`, `supplier_invoices.rs` (`create_in_tx`, tel que la 15-5a
      l'a réécrit) ; l'invariant « chaque flux de rapprochement prend son `GET_LOCK` avant tout verrou
      de ligne » (AC5, `retry.rs`) ; consigner au Dev Agent Record.
- [ ] **T1 — Le patron et les trois routes** (AC2, AC3) : le module `kesh_api::retry`
      (`retry_app_on_deadlock`, `is_app_deadlock`, `pub mod retry;`) ; le paramètre `operation` de
      `retry_with`, `retry_on_deadlock` et `retry_on_deadlock_with`, le `warn!` avec son champ ; les
      cinq sites directs et les six appels de tests qui gagnent un nom ; les trois routes ;
      doc-comments.
- [ ] **T2 — L'ordre et les commentaires** (AC5) : l'appel des réglages de la saisie fournisseur ; le
      doc-comment canonique ; le commentaire « 5 bis » ; le doc-comment du module `retry.rs`.
- [ ] **T3 — Les tests** (AC4) : les tests 1 à 4 ; la seconde colonne du registre
      `audit_route_registry.rs`, ses tests (c) — avec le test de l'extraction sur source synthétique —
      et (d), l'en-tête 111/114 corrigé ; mutations consignées.
- [ ] **T4 — Documentation** (AC6) : la ligne du CHANGELOG.
- [ ] **T5 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) —
      **même en cours de boucle de revue**, la story touchant des repositories `kesh-db` ; gate
      frontend complet (rien n'y change : il le confirme) ; **E2E Playwright complet au dernier commit
      de code** (décision D7), jugé fichier par fichier contre `docs/testing.md` § « Les échecs
      attendus ».

*(Décompte : 7 AC, 6 tâches T0–T5.)*

## Dev Notes

### Les routes qui écrivent au journal (relevé sur `b80ab8c0`, complété sur `c734645b`)

C'est **l'inventaire de l'Epic** : cette story l'établit et le grave dans le registre ; la 15-5e2 s'y
réfère pour son rollout (colonne « fiche »).

**Remontée depuis le symptôme** — les primitives, puis leurs appelants, jusqu'aux handlers :

```sh
grep -rnE "\b(create_in_tx|reverse_in_tx|reverse_owned_in_tx|delete_in_tx|create_opening_entry|delete_by_id|journal_entries::create|journal_entries::reverse)\s*\(" crates/*/src --include=*.rs
grep -rnE "INSERT INTO journal_entr|DELETE FROM journal_entr" crates/*/src --include=*.rs
grep -rnE "TRUNCATE|TABLES_TO_TRUNCATE|restore_tables_in_tx|truncate_all|reset_demo|format!\(\"[^\"]*(DELETE FROM|INSERT INTO|REPLACE INTO|UPDATE )" crates/*/src --include=*.rs
grep -nE "post\(|put\(|delete\(|patch\(" crates/kesh-api/src/lib.rs crates/kesh-api/src/routes/test_endpoints.rs
```

La troisième commande (findings F2-2, R2-3) voit le SQL dont la table est nommée par variable, que
les motifs littéraux de la deuxième ne peuvent pas voir : relevé sur `c734645b`, **99 lignes**, dont
quatre sites qui écrivent réellement au journal — `backup.rs:457` (``DELETE FROM `{table}` ``) et
`:509` (``INSERT INTO `{table}` ``) dans `restore_tables_in_tx` → `POST /admin/full-import` ;
`test_fixtures.rs:401` (`TRUNCATE TABLE {table}`) dans `truncate_all` → `POST /api/v1/_test/seed`
et `/reset` (`test_endpoints.rs:172`, `:306`) ; `kesh-seed/src/lib.rs:233` (`reset_demo`) →
`POST /onboarding/reset`. Le reste : l'export (`admin_backup/export.rs`, `exports/global.rs`), la
validation de l'import (`admin_backup/import.rs`), des commentaires et des tests — lecture seule.

**Tri des sorties des deux premières commandes** (finding R2-4), à refaire en T0 :
- **homonymes sans rapport** (la première) : `users::create_in_tx`, `api_keys::create_in_tx`,
  `invoice_settlements::create_in_tx`, `reconciliation_rules::create_in_tx`,
  `imported_supplier_invoices::create_in_tx` (`inbox_import.rs:543` : table de pré-import, aucune
  écriture au journal) ;
- **aides de test et de démontage** (la deuxième) : `crates/kesh-db/src/repositories/invoices.rs:4268`,
  `:4311`, `accounts.rs:1173`, `:1179` ; `journal_entries::delete_all_by_company`
  (`journal_entries.rs:1202`, **`pub`, hors `#[cfg(test)]`**, appelée seulement par des tests —
  `:1784`, `:1839`) ; les sites de tests de `journal_entries.rs` (`:1844`, `:2336`, `:2360`, `:2566`,
  `:3411`) ;
- **faux amis du motif** `journal_entr` : `journal_entry_number_sequences` (`kesh-seed/src/lib.rs:272`)
  et `journal_entry_lines` ; `kesh-seed/src/lib.rs:257-260` est `reset_demo` (exemptée).

Les numéros de ligne de la colonne « route » sont ceux des **handlers** (`crates/kesh-api/src/routes/…`) ;
ceux des dépôts sont préfixés de leur chemin (finding R3-6).

| route (handler, `crates/kesh-api/src/routes/…`) | écriture au journal (dépôt) | aujourd'hui | rejeu | fiche |
|---|---|---|---|---|
| `POST /invoices/{id}/write-off` (`invoices.rs:1328`) | `invoice_settlements_write::write_off_invoice` | **rejouée** (`:1343`, `retry_with`) | enveloppe `DbError`, `"invoices::write_off"` | nom : 15-5e1 ; migration : 15-5e2 |
| `POST /reconciliation/accept` (`reconciliation.rs:692`) | `accept_one_invoice` / `_split` / `_rule` → `create_in_tx` | **rejouée** (`:850`, prédicat 1213 + 1305) | inchangée, `"reconciliation::accept"` | nom : 15-5e1 |
| `POST /reconciliation/transactions/{id}/cancel` (`reconciliation.rs:3834`) | `reconciliation_cancel::cancel_in_tx` → `reverse_in_tx`, `cancel_settlement_in_tx` | **rejouée** (`:3843`, `retry_with`) | enveloppe `AppError`, `"reconciliation::cancel"` | nom : 15-5e1 ; migration : 15-5e2 |
| `POST /opening-balances/complete` (`opening_balances.rs:576`) | `opening_complement::create_opening_complement` → `create_in_tx` | **rejouée** (`:591`, `retry_with`) | enveloppe `DbError`, `"opening_balances::complete"` | nom : 15-5e1 ; migration : 15-5e2 |
| `POST /invoices/{id}/validate` (`invoices.rs:869`) | `invoices::validate_invoice` → `create_in_tx` | 500 | enveloppe `DbError`, `"invoices::validate"` — #536 | **15-5e1** |
| `POST /invoices/{id}/settlements` (`invoices.rs:1261`) | `settle_invoice` → `create_in_tx` | 500 | enveloppe `DbError`, `"invoices::settle"` — #491 | **15-5e1** |
| `POST /invoices/{id}/settlements/{sid}/cancel` (`invoices.rs:1478`) | `cancel_settlement` → `reverse_owned_in_tx` | 500 | enveloppe `DbError`, `"invoices::cancel_settlement"` — #463 | **15-5e1** |
| `POST /invoices/{id}/unvalidate` (`invoices.rs:911`) | `invoices::unvalidate` → `delete_in_tx` | 500 | `DbError` ; la `version` du corps rend une tentative périmée en 409 | 15-5e2 |
| `POST /credit-notes` (`credit_notes.rs:183`) | `credit_notes::create_credit_note` → `create_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /supplier-invoices` (`supplier_invoices.rs:330`) | `supplier_invoices::create` → `create_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /supplier-invoices/{id}/pay` (`supplier_invoices.rs:370`) | `supplier_invoices::pay` → `pay_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /supplier-invoices/{id}/cancel` (`supplier_invoices.rs:398`) | `supplier_invoices::cancel` → `reverse_owned_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /supplier-invoices/{id}/settlement/cancel` (`supplier_invoices.rs:424`) | `supplier_invoices::cancel_settlement` → `reverse_owned_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /imported-supplier-invoices/{id}/complete` (`imported_supplier_invoices.rs:123`) | transaction du handler → `supplier_invoices::create_in_tx` | 500 | `AppError`, fonction « une tentative » | 15-5e2 |
| `POST /payment-batches/{id}/confirm` (`payment_batches.rs:241`) | `payment_batches::confirm_batch` → `pay_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /journal-entries` (`journal_entries.rs:469`) | `journal_entries::create` | 500 | `DbError`, correspondance `FiscalYearClosed` après | 15-5e2 |
| `DELETE /journal-entries/{id}` (`journal_entries.rs:611`) | `journal_entries::delete_by_id` → `delete_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /journal-entries/{id}/reverse` (`journal_entries.rs:452`) | `journal_entries::reverse` → `reverse_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /opening-balances` (`opening_balances.rs:338`) | `journal_entries::create_opening_entry` | 500 | `DbError`, `map_opening_balances_error` après | 15-5e2 |
| `POST /reconciliation/manual` (`reconciliation.rs:2942`) | transaction du handler + verrou nommé → `create_in_tx` | 500 | `AppError`, fonction « une tentative » | 15-5e2 |
| `POST /reconciliation/split` (`reconciliation.rs:3362`) | transaction du handler + verrou nommé → `create_in_tx` | 500 | `AppError`, fonction « une tentative » | 15-5e2 |
| `POST /admin/full-import` (`admin.rs:137`, `full_import`) | `kesh_db::backup::restore_tables_in_tx` (toutes les tables) | 500 | **exemptée** : restauration d'instance, geste d'administration exclusif hors exploitation ; un 1213 y annule la transaction unique, et la relance manuelle est sûre | registre : 15-5e1 |
| `POST /onboarding/reset` (`onboarding.rs:240`) | `kesh_seed::reset_demo` (`DELETE FROM journal_entries`) | 500 | **exemptée** : effacement de la démo, même raison | registre : 15-5e1 |
| `POST /api/v1/_test/seed` (`test_endpoints.rs:49`, `seed_handler`) | `test_fixtures::truncate_all` (`TRUNCATE TABLE`, `test_fixtures.rs:401`) puis préréglages | 500 | **exemptée** : mode test, jamais monté en production (`lib.rs:1056-1065`) ; sérialisée par son propre verrou (`seed_lock`) | registre : 15-5e1 |
| `POST /api/v1/_test/reset` (`test_endpoints.rs:50`, `reset_handler`) | `test_fixtures::truncate_all` | 500 | **exemptée** : même raison | registre : 15-5e1 |

**Décompte** (recompté sur la table) : 4 rejouées aujourd'hui + 3 rejouées ici + 14 à la 15-5e2 =
**21** routes de `lib.rs` qui écrivent au journal ; **4** exemptées (2 de `lib.rs`, 2 de
`test_endpoints.rs`) ; total **25**. Le registre range les **90** autres routes mutantes (89 de
`lib.rs`, `/password-reset-token`) en `SansEcritureAuJournal` : 25 + 90 = **115**. Les cinq sites
`retry_with` du dépôt : les quatre premières lignes et `onboarding::finalize`, qui n'écrit pas au
journal.

**Le chemin d'erreur, relevé** (`grep -nE "map_err\(\|[a-z_]*\|"` sur les dépôts concernés) : les seuls
`map_err` qui ne passent pas par `map_db_error` convertissent des erreurs **non sqlx** (rendu d'un
numéro, `crates/kesh-db/src/repositories/credit_notes.rs:390`, `repositories/invoices.rs:2208` ;
`last_insert_id`, `repositories/journal_entries.rs:408` ; équilibre, `repositories/opening_complement.rs:622`) :
aucun ne peut masquer un 1213. Dans `post_manual` et `post_split`, `ReconciliationError::Db(db)` et
`::Database(e)` ressortent en `AppError::Database` (`routes/reconciliation.rs:3260-3267`, et
`:3704-3711` pour le second) : le prédicat les voit.

**Effets de bord hors transaction** : aucune des routes à rejouer n'envoie d'e-mail, n'écrit de
fichier ni n'appelle de service dans la partie à rejouer (`grep -nE "tokio::fs|std::fs|smtp|mailer"`
sur les dépôts : vide ; les PDF se rendent à la lecture). Les relectures post-commit
(`find_by_id_with_lines`, `amount_settled`, `load_with_settlement_cancellation`) restent hors de la
fermeture.

### Pourquoi rejouer est sûr

Un 1213 **annule toute la transaction** de la victime : rien n'est écrit, ni ligne, ni audit, ni
compteur. La tentative suivante repart d'une transaction neuve, relit tout sous ses verrous et refait
toutes ses gardes — c'est exactement ce qu'aurait fait la même requête envoyée une fraction de seconde
plus tard. Ce qui a changé entre-temps est jugé par les gardes ordinaires : un règlement qui trouve la
facture soldée entre-temps est refusé en trop-perçu ; une validation qui trouve la facture déjà validée
est refusée en 409 (`IllegalStateTransition`) ; une annulation de règlement qui trouve le règlement
déjà annulé, par sa propre garde. Le rejeu n'ajoute aucun cas qui n'existe déjà pour deux requêtes
successives.

### Ce que vit l'utilisateur

- Aujourd'hui : la victime d'un interblocage reçoit `500 INTERNAL_ERROR` (« erreur interne ») et doit
  recommencer ; rien n'est écrit. C'est le cas du règlement (#491), de l'annulation de règlement
  (#463) et de la validation contre un lot de rapprochement (#536) ; l'acceptation du lot, elle,
  rejoue déjà.
- Après : l'opération aboutit, avec au pire ≈ 150 ms de latence ajoutée. Seul un interblocage répété
  trois fois de suite ressort en 500, comme avant.

### Ce que la réécriture C54 a retiré (pour mémoire)

« L'arrondi d'abord » au règlement client et au solde du reste, leurs priorités de refus, les trois
tests à sonde `NOWAIT`, toute affirmation de cycle fermé et l'inventaire des sites qui verrouillent
`accounts` (détail au Change Log de la fiche index `15-5e-ordre-des-verrous-reglements.md`).

### L'ordre des verrous des flux comptables, pour mémoire (relevé sur `b80ab8c0`)

Non normatif : c'est ce que le doc-comment canonique décrit et ce que la 15-5e2 écrit au Pattern 5.
Chaque flux reprend en outre, à l'insertion de l'écriture et de ses lignes, des verrous **partagés**
sur la ligne `companies` (`fk_journal_entries_company`), sur chaque compte écrit (`fk_jel_account`) et
sur chaque projet tagué (`fk_jel_project`) — finding F2-1. Tous les numéros de ligne ci-dessous sont
ceux des **dépôts** (`crates/kesh-db/src/repositories/…`), sauf mention contraire (finding R3-6).

| flux | réglages | comptes `FOR UPDATE` | exercice |
|---|---|---|---|
| validation d'une facture | `repositories/invoices.rs:2006` | arrondi `:2065` si écart (la 15-5d y ajoute la créance et la TVA due) | `:2172` |
| règlement client | — | compte interne `repositories/invoice_settlements_write.rs:156-167` (virement : `bank_accounts` `:119-131`, aucune ligne d'`accounts`), puis arrondi `:184-196` si écart | `:199-202` |
| solde du reste | lus sans verrou (`usable_designated_account`, `repositories/company_invoice_settings.rs:484`) | nature `invoice_settlements_write.rs:430-433`, arrondi `:471-482` si reste hors centime, TVA due `:487-490` | `:492-495` |
| saisie / complétion fournisseur | **avancés** avant les comptes de charge (AC5 ; aujourd'hui `repositories/supplier_invoices.rs:357-359`, après l'exercice) | charges `:300-350`, dans l'ordre des lignes | `:352-355` |
| règlement fournisseur par compte interne | — | compte interne `repositories/supplier_invoices.rs:639-650` | `:662` |
| avoir (`create_credit_note`) | **verrouillés** (`repositories/credit_notes.rs:359-361` → `company_invoice_settings.rs:109`) | arrondi `repositories/credit_notes.rs:525` | `:370` — avant l'arrondi (#536) |
| lot de rapprochement (`accept_batch`) | — | arrondi `routes/reconciliation.rs:1487` par proposition | `:1520` ; règle à projet : sentinelle **après** l'exercice (`:2379` → `:2411`) |
| onboarding (`insert_with_defaults_in_tx`) | insérés **après** les comptes de rôle (`company_invoice_settings.rs:563-586`) | comptes de rôle | créé ensuite |

### Fichiers touchés (prévision)

`crates/kesh-api/src/retry.rs` (neuf) et `lib.rs` (déclaration du module) ;
`crates/kesh-api/src/routes/invoices.rs` (trois routes rejouées ; nom au `retry_with` de
`write_off_invoice_handler`) ; `routes/{reconciliation,opening_balances,onboarding}.rs` (un nom
d'opération à chacun des sites `retry_with`, rien d'autre) ; `crates/kesh-db/src/retry.rs` (paramètre
`operation`, `warn!`, doc du module, six appels de tests) ;
`crates/kesh-db/src/repositories/{invoices,invoice_settlements_write,supplier_invoices}.rs`
(commentaires ; le code ne bouge que dans `supplier_invoices`) ;
`crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (neuf) ; `crates/kesh-api/tests/audit_route_registry.rs`
(seconde colonne, tests (c) et (d), en-tête) ; `CHANGELOG.md`. **Aucune migration** (P1–P8 sans
objet), aucun fichier `kesh-i18n` ni `frontend`, aucun manuel. Modules : `kesh-api` (module neuf
`retry`, quatre modules de routes dont trois à une ligne), `kesh-db` (`retry`, trois dépôts dont deux en
commentaires) — sous le seuil de la règle de découpage pour le code qui se conçoit (les sites à une
ligne sont forcés par le changement de signature).

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C43** — côté achat, les réglages avancés avant les comptes de charge (le reste de C43, révisé par
  C48, est retiré par C55).
- **C54** — la défense contre l'interblocage est le rejeu, pas un ordre parfait des verrous.
- **C55** — ce qui reste de l'ordre des verrous : l'avance des réglages de la saisie fournisseur et
  des commentaires vrais.
- **C56** — la forme du rejeu : deux enveloppes partagées et le registre (sa dérogation de découpage
  est **révisée par C61**, ses « deux routes exemptées » par C58).
- **C57** — #536 : couverte pour la validation ici, fermée par la 15-5e2 (avoir).
- **C58** — le registre est une seconde colonne du registre d'audit existant.
- **C59** — les rejeux journalisés en `warn` avec le nom de l'opération (**révisé par C62** : le nom
  est un champ de l'événement, pas un span).
- **C61** — le découpage 15-5e1 / 15-5e2.
- **C62** — les noms des enveloppes et le nom d'opération porté par l'événement.
- **C63** — le statut transitoire `ARejouer` et le volet (c) robuste.
- **C64** — la ligne du CHANGELOG voyage avec les issues qu'elle ferme ; PDF de la brochure.
- **C65** — la 15-5d dépend de la 15-5e1 seule.

### References

- Issues : #463, #491 (fermées ici) ; #536 (couverte pour la validation, fermée par la 15-5e2) ; #429
  (fermée par la 15-5d) ; #480 (rejeu de l'acceptation, précédent) ; #43 (KF-002-H-002, le module
  `retry`).
- Fiches : `15-5e-ordre-des-verrous-reglements.md` (index, `split`, historique des validations P1–P3),
  `15-5e2-rejeu-des-autres-flux.md` (le rollout), `15-5-gardes-postabilite-serveur.md` (mère,
  `split`), `15-5a-refus-non-imputable.md` (AC4 : les deux passes des comptes de charge),
  `15-5d-garde-usage-comptes-reglage.md` (la garde qui s'appuie sur cette story).
- Code : `crates/kesh-db/src/retry.rs` ; `crates/kesh-api/tests/reconciliation_e2e.rs:4426` (patron
  d'interblocage déterministe) ; `crates/kesh-api/tests/audit_route_registry.rs` (registre étendu).
- `CLAUDE.md` : § *Test Locally First* (exception `kesh-db`), § *Propagation post-patch*,
  § *Règle de splitting préventif*, § *Un gate laisse la base piégée*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-08 — **Créée par le découpage de la 15-5e** (choix **C61**, sur le finding **F3-2 MEDIUM** de
  sa validation P3 : la dérogation de découpage écrite dans la 15-5e ne reposait pas sur l'exception
  que la règle prévoit). Story-zéro du patron « story-zéro + rollout » : reprend de la 15-5e (version
  d'avant découpage au commit `94f1365e`) l'inventaire (AC1), la forme du rejeu (AC2), les trois
  routes des issues (AC3), les tests 1 à 5 (AC4), et de son AC5 ce que la 15-5d attend — doc-comment
  canonique, avance des réglages de la saisie fournisseur, « 5 bis », module `retry.rs` (C65).
  **Remédiations de la P3 appliquées ici** : F3-1 (volet (c) : enveloppes nommées, commentaires
  retirés, fenêtre coupée, `nom(`, extraction testée sur source synthétique, mutations sur plusieurs
  familles — C63) ; F3-3 (nom d'opération en champ du `warn!`, signature de `retry_with` — C62) ;
  F3-6 (partition 7 / 14 / 4 / 90, total 115 ; `/password-reset-token` nommée ; en-tête 111/114 ;
  numéros de ligne préfixés route / dépôt) ; F3-7 (condition de (2 quater)) ; F3-8 (entrées clonées
  par tentative) ; F3-9 (CHANGELOG : « seul un interblocage répété trois fois… ») ; R3-2 (six appels
  de tests de `retry.rs`, cinq sites directs) ; R3-3 (noms, `pub mod retry`, prédicat exposé) ; R3-4
  (contrôles qui dépendent de la transaction restent dans la tentative) ; R3-6 (c), (d) ; R3-7 (pas
  de promesse de sérialisation par la ligne des réglages). Le détail de la P3 et le trend des passes
  sont au Change Log de la fiche index. **Propagation post-patch** : `info_span`, `span`,
  `nom au choix`, `17 routes`, `Deux routes exemptées`, `sérialise`, `111`, `114` grepés sur le corps
  de la fiche (hors Change Log) : les occurrences restantes citent le texte à corriger (`111 / 114`)
  ou disent « jamais sérialise ». Décompte (recompté sur la fiche) : **7 AC, 6 tâches T0–T5** ;
  routes : 7 `Rejouee` (4 + 3), 14 `ARejouer`, 4 `Exemptee`, 90 `SansEcritureAuJournal`, total 115.
