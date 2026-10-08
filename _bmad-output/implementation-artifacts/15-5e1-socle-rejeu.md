# Story 15.5e1 : Socle du rejeu sur interblocage — les enveloppes, le registre, les trois routes des issues, la saisie fournisseur et l'enregistrement des réglages de facturation

Status: review

<!-- Sous-story de la 15-5e (fiche index `15-5e-ordre-des-verrous-reglements.md`, statut `split`),
     créée le 2026-10-08 par le découpage décidé à la validation P3 de la 15-5e (finding F3-2, choix
     C61) : patron « story-zéro + rollout » de la § Règle de splitting préventif. Cette fiche est la
     story-zéro : elle pose le patron (deux enveloppes nommées, journalisation, registre) et
     l'applique aux trois routes des issues (et, depuis la P4, à la saisie fournisseur — C66) ; la
     15-5e2 (`15-5e2-rejeu-des-autres-flux.md`) en fait le rollout. Le contenu vient de la 15-5e validée P1 à P3 (commit `94f1365e` pour la version
     d'avant découpage) ; les remédiations de la P3 sont appliquées ici et dans la 15-5e2.
     Choix applicables : C43 (pour le seul AC5 « saisie fournisseur »), C54, C55 (révisé par C66),
     C56 (révisé par C61 et C63), C57 (révisé par C61, cf. C69), C58, C59 (révisé par C62), C61,
     C62, C63, C64, C65 (révisé par C66 et C68), C66, C67 (révisé par C70), C69, C70 (corrigé par
     C74), C74.
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation. Le
     développement attend la clôture de la boucle de validation ; les 15-5a et 15-5b sont mergées
     (`f289414e`). Numéros de ligne relevés sur `f289414e` sauf mention contraire. -->

**Issues** : la PR porte **`closes #463`** et **`closes #491`** (mots-clés **dans la PR** : le dépôt
merge en squash), **`refs #536`** et **`refs #429`**.
- **#463** — l'annulation d'un règlement client (`POST /api/v1/invoices/{id}/settlements/{sid}/cancel`)
  rend 500 sur interblocage, sans rejeu : rejouée ici (AC3), test où elle est la victime (AC4).
- **#491** — le règlement manuel (`POST /api/v1/invoices/{id}/settlements`) victime d'un interblocage
  avec un rapprochement n'est pas rejoué : rejoué ici (AC3), test (AC4).
- **#536** (`refs`) — interblocage entre l'acceptation d'un lot de rapprochement et la validation d'une
  facture. L'acceptation **rejoue déjà** (`post_accept`, `reconciliation.rs:888`, Story 25-4-c2, #480 —
  prédicat 1213 **et** `ReconciliationTransactionAborted`) ; la **validation** est rejouée ici (AC3,
  test 4 de l'AC4, qui monte le cycle de l'issue — compte d'arrondi contre exercice) : le cas
  principal de l'issue est couvert. L'issue cite une seconde inversion,
  l'**avoir** d'une facture arrondie : sa route est rejouée par la **15-5e2**, qui porte `closes #536`
  (choix C57).
- **#429** (`refs`) — fermée par la 15-5d, qui s'appuie sur cette story (ci-dessous).

**Dépend de la 15-5a et de la 15-5b — dépendance remplie** : les deux sont mergées dans `main` et
réintégrées dans la branche de planification (`f289414e`).
- La **15-5a** a réécrit la boucle des comptes de charge de `supplier_invoices::create_in_tx` en
  **deux passes**, forme puis comptes (son AC4, choix C22 ; `supplier_invoices.rs:301-372`, forme
  `:309`, comptes `:326`) : l'AC5 place l'appel des réglages entre elles.
- La **15-5b** a réécrit des commentaires de `invoice_settlements_write.rs` (choix C23) : l'AC5
  réécrit le commentaire « 5 bis » du même fichier, désormais sans conflit.

**Passe avant la 15-5e2 et avant la 15-5d** (choix C65, révisé par C66).
- La **15-5e2** fait le rollout du patron que cette story pose.
- La **15-5d** dépend de **cette story, et pas du rollout** : elle s'appuie sur le rejeu de la
  **validation** et de la **saisie fournisseur** (AC3), l'avance des réglages de la saisie
  fournisseur, le doc-comment canonique de `validate_invoice` (numérotation **(2 bis')**) et le
  commentaire « 5 bis » (AC5) — tous posés ici.
- **La saisie fournisseur est rejouée ici, en même temps que l'avance de ses réglages** (choix C66,
  finding F4-1 de la P4) : l'avance fait de la ligne des réglages le premier verrou disputé entre
  deux saisies d'une même société, avant l'exercice où elles s'attendaient jusqu'ici. Si un
  `INSERT IGNORE` en doublon pose un verrou **partagé** sur l'enregistrement (hypothèse R3-7 —
  **hypothèse forte** depuis la P5 : c'est le comportement documenté d'InnoDB pour un doublon,
  relevé par la lentille F, finding F5-6 ; **mesurée en T0**, choix C70), deux saisies
  concurrentes s'interbloquent là où l'une attendait l'autre. Le rejeu rend cette hypothèse
  **sans conséquence** pour l'utilisateur : la victime est rejouée. La
  défense arrive donc avec le geste qui pourrait la rendre nécessaire, et pas un merge plus tard.
- Le rejeu de la **complétion d'une facture importée** (même `create_in_tx`, mais transaction
  ouverte dans le handler, enveloppe `AppError` et fonction « une tentative ») reste à la 15-5e2,
  dans un ordre de merge libre par rapport à la 15-5d : entre le merge de cette story et celui de la
  15-5e2, une victime d'interblocage y rend `500` comme aujourd'hui — et l'avance des réglages peut,
  si R3-7 se vérifie, l'exposer à un cycle de plus (deux complétions, ou une complétion et une
  saisie, de la même société). Fenêtre assumée : la complétion d'import est un geste interactif,
  rarement concurrent d'un autre sur la même société, et la 15-5e2 la ferme.
- **L'enregistrement des réglages de facturation est rejoué ici aussi** (choix C70, finding F5-6
  de la P5) : `PUT /api/v1/company/invoice-settings` passe par la même ligne des réglages
  (`INSERT IGNORE`, lecture simple, puis `UPDATE` exclusif) ; si R3-7 se vérifie, l'avance la met
  dans le même cycle que les saisies, et, n'ayant rien modifié quand le cycle se ferme, elle en
  serait la victime — un 500. Elle n'écrit pas au journal : elle reste `SansEcritureAuJournal` au
  registre (AC3, test 7 de l'AC4).

## Story

En tant que **comptable d'une société tenue dans Kesh**,
je veux qu'**un règlement client, l'annulation d'un règlement client, la validation d'une facture,
la saisie d'une facture fournisseur ou l'enregistrement des réglages de facturation victime d'un
interblocage avec une autre opération légitime soit
rejoué par le serveur, sans que je le voie**,
afin de **ne plus recevoir d'erreur 500 quand ces opérations croisent un rapprochement bancaire** — et
que le projet dispose du patron (enveloppes, journalisation, registre) que la 15-5e2 étend à toutes les
routes qui écrivent au journal.

### Pourquoi le rejeu, et non un ordre des verrous (choix C54)

Tout flux qui écrit au journal insère des lignes `journal_entry_lines` ; la clé étrangère
**`fk_jel_account`** (`crates/kesh-db/migrations/20260412000001_journal_entries.sql:45`) fait poser par
InnoDB un **verrou partagé sur chaque ligne `accounts` écrite**, à l'insertion — donc **après** le
verrou de l'exercice (`journal_entries::create_in_tx`, exercice
`crates/kesh-db/src/repositories/journal_entries.rs:291-294`, lignes ensuite). Un flux qui
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
   `b80ab8c0`, complétée sur `c734645b`, numéros recalés sur `f289414e`) : **21 routes** de `lib.rs`
   qui écrivent dans `journal_entries` — **4 déjà rejouées**, **4 rejouées par cette story**, **13
   rejouées par la 15-5e2** — et **4 routes exemptées**, raison écrite : 2 de `lib.rs` (restauration d'instance,
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
   test 6) : une route mutante ajoutée demain sans examen fait rougir le gate.
2. **AC2 — La forme du rejeu : deux enveloppes nommées, une transaction neuve par tentative**
   (choix C56, C62). Les noms sont **fixés ici** (finding R3-3) — le registre les cherche par leur nom (dernier segment du chemin appelé, AC4, test 6 (c)).
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
     arrivent avec la 15-5e2** (la saisie fournisseur, rejouée ici, prend l'enveloppe `DbError` :
     `supplier_invoices::create` ouvre et conclut sa transaction) — d'ici là, fonction `pub` d'une
     bibliothèque (aucun avertissement `dead_code`), **exercée par le test 1** sur une vraie 1213
     (finding F4-5 de la P4 : son câblage — prédicat, nom transmis, nombre de tentatives — ne
     rougirait sinon nulle part avant la 15-5e2).
   - **Le nom d'opération est porté par l'événement lui-même** (finding F3-3, choix C62, révise C59).
     `retry_with` gagne un **premier paramètre** `operation: &'static str` (p. ex.
     `"invoices::settle"`), et son `tracing::debug!` (`retry.rs:160`) devient un
     **`tracing::warn!(target: "kesh_db::retry", operation, attempt, max_attempts, backoff_ms, …)`** :
     la ligne dit **quelle route** a été rejouée même sous un filtre `RUST_LOG=warn` — un span `info`
     y serait désactivé et l'événement perdrait son nom (le serveur n'a pas de span de requête, aucun
     `TraceLayer`). Les deux enveloppes transmettent le nom. Changement de signature, **et ce qu'il
     touche, en entier** (findings R3-2, R4-4) : les **six appels de tests** de `retry.rs` (`:189`,
     `:205`, `:225`, `:241`, `:318`, `:348`) gagnent un nom, assertions inchangées ; l'**exemple du
     doc-comment** de `retry_on_deadlock` (`retry.rs:95-104`, bloc `ignore` — aucun gate ne le
     compile, il faut donc le corriger à la main) gagne le sien ; les **cinq sites directs**
     de `retry_with` gagnent le leur — `post_accept` (`routes/reconciliation.rs:888`,
     `"reconciliation::accept"`), `post_cancel_reconciliation` (`:3960`, `"reconciliation::cancel"`),
     `write_off_invoice_handler` (`routes/invoices.rs:1343`, `"invoices::write_off"`),
     `complete_opening_balances` (`routes/opening_balances.rs:594`, `"opening_balances::complete"`),
     `onboarding::finalize` (`routes/onboarding.rs:614`, `"onboarding::finalize"`) ; **rien d'autre** dans
     ces cinq sites (leur passage aux enveloppes est le rollout de la 15-5e2). Un 1213 épuisé reste
     journalisé comme aujourd'hui, en erreur, par le chemin du 500.
   - **Une tentative = une transaction neuve** : la fermeture rejouée commence par `pool.begin()` (ou
     appelle une fonction qui le fait) et finit par le `commit()`. Les contrôles qui **ne dépendent
     pas** de la transaction (forme du corps, préchargement de la société) et ce qui suit le commit
     (relecture de la réponse) **peuvent rester hors** de la fermeture. Les contrôles qui lisent ce que
     la transaction verrouille **ne sortent jamais de la tentative** et y restent dans le même ordre —
     aucun refus ne se déplace (AC7 ; finding R3-4 : `complete_import` contrôle devise, QR-IBAN et
     montant sur le `staging` lu `FOR UPDATE`, cas de la 15-5e2, règle posée ici). Une lecture hors
     transaction **déjà placée** dans la fonction « une tentative » **peut y rester** (finding R4-3 :
     `cancel_reconciliation_once` lit sans verrou, avant son `begin`, le `bank_account_id` qui nomme
     le verrou nommé, `routes/reconciliation.rs:3982-3992` — elle n'a pas à en sortir). **Aucun effet
     de bord hors transaction** dans la fermeture — ni e-mail, ni fichier, ni appel réseau : vérifié
     pour les 21 routes au relevé de `b80ab8c0` (Dev Notes), **revérifié en T0** pour les cinq
     routes rejouées ici (finding R6-5).
   - **Les entrées consommées se clonent par tentative** (finding F3-8) : les enveloppes prennent une
     fermeture `Fn`. Ce que le handler consomme par déplacement (le corps `Json(req)`, un `New…`
     construit à partir de lui) est cloné **dans** la fermeture, à chaque tentative — patron de
     `onboarding::finalize` (`routes/onboarding.rs:606-624`, « les captures clonées garantissent que la
     closure est `Fn` et non `FnOnce` »). Les types concernés sont `Clone` (vérifié en P3 pour
     `NewSupplierInvoice`, `NewJournalEntry` ; T0 le revérifie pour les corps des cinq routes rejouées ici). **Pas
     de `Option::take`** ni de `FnOnce` contourné.
   - **Le 1213 doit atteindre le prédicat** : sur le chemin de chaque route, une erreur sqlx passe par
     `map_db_error` (`crates/kesh-db/src/errors.rs:918`), qui laisse le 1213 en `DbError::Sqlx` ; un
     `map_err` qui convertirait une erreur sqlx en une autre variante rendrait le rejeu **muet** (le
     motif est écrit au doc-comment de `post_cancel_reconciliation`). Relevé sur `b80ab8c0` : aucun sur
     les chemins de l'inventaire (Dev Notes) ; T0 le refait pour les cinq routes rejouées ici.
   - **Ce qui ne change pas** : trois tentatives (`DEFAULT_MAX_DEADLOCK_ATTEMPTS`), backoff 50 puis
     100 ms. Au-delà, le 1213 sort en `500 INTERNAL_ERROR`, comme aujourd'hui — rien n'a été écrit, la
     requête peut être renvoyée telle quelle.
3. **AC3 — Les trois routes des issues, la saisie fournisseur et l'enregistrement des réglages de facturation, rejoués par l'enveloppe `DbError`**
   (détail, fonction de dépôt et nom d'opération : table des Dev Notes) : **validation d'une facture**
   (`validate_invoice_handler`, `"invoices::validate"` — #536), **règlement client**
   (`settle_invoice_handler`, `"invoices::settle"` — #491), **son annulation**
   (`cancel_invoice_settlement_handler`, `"invoices::cancel_settlement"` — #463), et **saisie d'une
   facture fournisseur** (`create_supplier_invoice`, `routes/supplier_invoices.rs:330`,
   `"supplier_invoices::create"` — choix C66 : elle arrive avec l'avance de ses réglages, AC5 ; le
   `NewSupplierInvoice` construit hors de la fermeture, qui déplace les lignes de `req`, est cloné à
   chaque tentative). Chacun de leurs doc-comments dit, en une ligne, que la route est rejouée sur
   interblocage, avec renvoi à l'enveloppe (pas de recopie de la règle).
   **Et l'enregistrement des réglages de facturation** (choix C70, finding F5-6 de la P5) :
   `update_invoice_settings` (`PUT /api/v1/company/invoice-settings`,
   `routes/company_invoice_settings.rs:268`) enveloppe son seul appel d'écriture,
   `company_invoice_settings::update` (`repositories/company_invoice_settings.rs:147`, qui ouvre et
   conclut sa transaction), par l'enveloppe `DbError`, `"company_invoice_settings::update"` ; le
   `CompanyInvoiceSettingsUpdate` (`Clone`, `entities/company_invoice_settings.rs:71`) est cloné à
   chaque tentative ; les lectures et contrôles qui précèdent (`get_or_create_default` sur le pool,
   formats, comptes) restent hors de la fermeture. **Ils lisent pourtant la ligne même que `update`
   verrouille** (findings R6-1 = F6-2) : `current`, lu hors transaction
   (`routes/company_invoice_settings.rs:278`), fournit les champs reconduits
   (`credit_note_number_format`, `default_payable_account_id`, `round_to_5_centimes`,
   `minimum_invoice_amount`) et le critère « imputable si la valeur change ». Ils sont **déjà** hors
   transaction aujourd'hui : ils n'en sortent donc pas, et l'AC2 n'est pas enfreinte ; c'est
   l'exception qu'elle prévoit pour une lecture déjà placée hors de la tentative. Ce qui les rend
   sûrs est le **verrou optimiste** : la `version` du client est rejugée à chaque tentative par
   l'`UPDATE … WHERE version = ?` de `update` (`repositories/company_invoice_settings.rs:205`) ; une
   ligne modifiée entre la lecture et la tentative rend 409 (`OptimisticLockConflict`), comme pour
   deux requêtes successives — aucun refus déplacé. Doc-comment : une ligne, comme les quatre autres.
   **Elle n'écrit pas au journal** : au registre, elle reste **`SansEcritureAuJournal`**, comme
   `onboarding::finalize`, rejouée elle aussi sans écrire au journal — pas de cinquième valeur de
   statut, la colonne dit l'inventaire de l'AC1 et non la présence d'une enveloppe. Le volet (c) ne
   l'examine donc pas ; elle est tenue par le **test 7** (preuve dynamique, et sa mutation) et par
   le point (vi) du doc-comment du registre, qui la nomme.

### Ce qui doit être prouvé

4. **AC4 — Tests.** Fichier neuf `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (tests 1 à 5 et 7) ;
   le test 6 **étend le registre existant** `crates/kesh-api/tests/audit_route_registry.rs` (choix
   C58), pas de fichier neuf.
   **Patron des interblocages** : celui de `accept_replays_the_batch_when_it_is_the_deadlock_victim`
   (`reconciliation_e2e.rs:4426`) — une transaction de test **alourdie** (table de lest, 500 lignes
   insérées, `seq_1_to_50` ; InnoDB choisit pour victime la transaction la plus légère), qui tient la
   ressource que la route demandera **en second** ; la route lancée en tâche, vue en attente par
   `kesh_db::test_fixtures::attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs:560`,
   motifs écrits au test) ; puis la transaction de test demande la ressource que la route tient
   **déjà** — cycle, InnoDB annule la route — et **doit l'obtenir** (`.expect`, sinon c'est elle la
   victime et le test ne prouve rien) ; elle annule ; la route rejoue et réussit.
   **Montage commun** (findings R2-9, F2-9, F4-9 ; sorti de l'item 1 où il était rangé, finding
   R4-11 ; partagé en P5, finding R5-2 ; société de test précisée en P6, finding R6-3).
   - **La société vient de `kesh_db::test_fixtures::seed_accounting_company`**
     (`crates/kesh-db/src/test_fixtures.rs:80`), et d'elle seule : elle crée la **ligne des
     réglages de facturation** (créance 1100, produit 3000, TVA due 2000, TVA récupérable 1000),
     les **quatre taux de TVA** suisses (dont 8,1 % et 0 %, que `POST /invoices` contrôle contre la
     base, `routes/invoices.rs:718`), les cinq comptes (1000, 1100, 2000, 3000, 4000), l'exercice
     2020-2030 et deux utilisateurs `Admin` (`SeededCompany`). **Pas de `create_company`** du
     harnais de `reconciliation_e2e.rs` (`:144`, `companies::create`) : il n'insère que la ligne
     `companies` — ni réglages, ni taux, ni exercice —, et `POST /invoices` refuserait le taux de
     8,1 %.
   - **Le harnais** n'apporte que `test_config`, `spawn_app` et `forge_jwt` (patron
     `reconciliation_e2e.rs:64-138`), plus `create_user` (`:169`) s'il faut un rôle autre qu'`Admin` (le
     JWT se forge sinon sur `admin_user_id`). Il est **recopié**, comme dans chacun des fichiers
     E2E de `crates/kesh-api/tests/` (`forge_jwt` y existe en 23 copies, `tests/common/mod.rs` ne
     porte que `create_test_company` et des aides d'audit) : copie assumée, déclarée au Dev Agent
     Record ; factoriser le harnais de tous les fichiers est une dette antérieure à cette story,
     hors de son périmètre (choix C70, signalée pour une issue).
   - **Aux tests 2 à 4** : les factures clients sont créées **par les routes** (`POST
     /api/v1/invoices`, puis `POST /api/v1/invoices/{id}/validate` pour les tests 2 et 3), par une
     aide locale au fichier neuf — **pas de quatrième copie** de `seed_validated_invoice`, qui en
     compte déjà trois (`invoice_pdf_e2e.rs:175`, `invoice_send_email_e2e.rs:217`,
     `reconciliation_e2e.rs:380` — finding F5-4) ; l'aide locale passe par les routes, ce n'en est
     pas une copie.
   - **Aux tests 2 et 3** : la facture est **datée du jour**, et l'arrondi à 5 centimes est
     **désactivé** par `kesh_db::test_fixtures::disable_rounding_to_5_centimes`
     (`test_fixtures.rs:236` ; finding F6-5) — sans quoi la validation et le règlement prendraient
     aussi le compte d'arrondi (`repositories/invoices.rs:2065`,
     `repositories/invoice_settlements_write.rs:195-207`) dès que le TTC brut n'est pas un multiple
     de 5 centimes (l'arrondi est actif par défaut, `round_to_5_centimes … DEFAULT TRUE`,
     `20261001000001_invoice_rounding_5_centimes.sql:26`, et `seed_accounting_company` ne le
     désactive pas). Le montant devient libre ; le motif d'attente, lui, ne change pas (finding
     F5-5 : la route attend toujours l'exercice). Le test 4 garde au contraire l'arrondi actif et
     prend un écart, exprès.
   - **Aux tests 2 à 5** : l'exercice tenu par la transaction de test est l'exercice **ouvert qui
     couvre aujourd'hui**, obtenu sans date codée en dur — patron `ensure_fiscal_year_today`
     (`reconciliation_e2e.rs:3230`) ; l'exercice 2020-2030 de `seed_accounting_company` couvre
     aujourd'hui mais cessera de le faire en 2031 —, faute de quoi la route refuse avant d'attendre
     et `attendre_une_requete_en_cours` panique.
   - **Le témoin du rejeu, aux tests 2 à 5 et 7** (finding **F6-1 MEDIUM** de la P6, choix
     **C74**). Le code de retour et les comptes (une écriture, une version, une entrée d'audit) ne
     départagent pas « la route a été annulée puis rejouée » de « la route a réussi du premier
     coup » : un montage qui ne forme pas son cycle passe au vert sans qu'aucun rejeu ait eu lieu.
     Chacun de ces tests installe donc, **pour sa durée et avant la première requête**, un
     abonné `tracing` de capture, et exige **au moins un** événement de niveau `WARN`, de cible
     `kesh_db::retry`, dont le champ `operation` vaut le nom attendu (`"invoices::settle"`,
     `"invoices::cancel_settlement"`, `"invoices::validate"`, `"supplier_invoices::create"`,
     `"company_invoice_settings::update"`) — l'événement que `retry_with` émet **avant** chaque
     nouvelle tentative (AC2). Un test qui passe sans rejeu **rougit**.
     - **Où et comment** : un module de test partagé, `crates/kesh-api/tests/common/capture_rejeu.rs`,
       déclaré `pub mod capture_rejeu;` dans `tests/common/mod.rs` (qui autorise déjà `dead_code`) ;
       une couche maison d'une trentaine de lignes sur **`tracing-subscriber`**, déjà dépendance
       normale de `kesh-api` (`crates/kesh-api/Cargo.toml:28`, 0.3.23 au `Cargo.lock`, fonction
       `registry` disponible) — **aucune crate ajoutée** : une `Layer` dont `on_event` retient,
       pour la cible et le niveau voulus, la valeur du champ `operation` (un `Visit` qui lit
       `record_str`) dans un `Arc<Mutex<Vec<String>>>` ; installée par
       `tracing::subscriber::set_default(registry().with(couche))`, dont la garde vit jusqu'à la
       fin du test.
     - **Pourquoi `set_default` suffit** : `#[sqlx::test]` exécute le test sur un runtime tokio
       **à un seul fil** (`sqlx-core-0.8.6/src/rt/mod.rs:119`, `new_current_thread`) ; la tâche du
       serveur (`spawn_app`) et celle qui porte la requête y tournent sur le fil du test, où
       l'abonné par défaut s'applique. Si un test passait un jour à un runtime multi-fil, les
       événements échapperaient à la capture et le témoin **rougirait** — un faux rouge, jamais
       un faux vert. Le serveur de test n'installe aucun abonné global (`AppState::new_for_tests`).
     - Le test 1 n'en a pas besoin : il compte les appels de la fermeture.
   1. **Le prédicat et l'enveloppe `AppError`, sur une vraie 1213** — deux connexions dédiées
      (`pool.acquire()`, pas de `sleep` : le patron ci-dessus, sur une table sentinelle, dans une
      aide locale qui rend l'erreur de la victime) ; l'erreur de la victime, passée par
      `map_db_error`, est reconnue par `kesh_db::retry::is_deadlock_error`, et, convertie en
      `AppError`, par `kesh_api::retry::is_app_deadlock`. **Négatifs** : un 1205 (`SET SESSION
      innodb_lock_wait_timeout = 1` sur une connexion qui attend une ligne tenue) et une erreur métier
      (`DbError::OptimisticLockConflict`) ne sont **pas** reconnus. Ce test garde aussi `map_db_error` :
      si un jour le 1213 y devenait une variante typée, il rougit. **L'enveloppe elle-même** (finding
      F4-5) : `retry_app_on_deadlock` autour d'une fermeture qui, au premier appel, provoque par la
      même aide une vraie 1213 et la rend convertie en `AppError`, puis rend `Ok` au second → `Ok`,
      **deux** appels ; autour d'une fermeture qui rend une erreur métier → l'erreur, **un seul**
      appel.
   2. **Règlement client victime (#491)** — facture validée ; règlement **par compte interne** (le
      plus court : le virement exigerait un compte bancaire configuré,
      `repositories/invoice_settlements_write.rs:119-133`), sur un compte actif imputable du plan
      semé ; la transaction de test tient l'**exercice** ; `POST /api/v1/invoices/{id}/settlements`
      vu en attente sur `fiscal_years` + `FOR UPDATE` (il tient la facture, étape (1) de
      `settle_invoice`, et le compte interne, `:157-164`) ; la transaction de test demande la
      **facture** `FOR UPDATE` → la route est la victime ; après annulation : **200**, **un seul**
      règlement, une seule écriture.
   3. **Annulation d'un règlement victime (#463)** — même montage sur `POST
      /api/v1/invoices/{id}/settlements/{sid}/cancel`. La route verrouille la facture (1), la ligne de
      règlement (2), puis, à l'étape **(2-bis)** de `cancel_settlement_in_tx`, l'**exercice de
      l'écriture de règlement**, joint à cette écriture (`SELECT fy.id FROM journal_entries je JOIN
      fiscal_years fy … FOR UPDATE`, `repositories/invoice_settlements_write.rs:742-745`) — **avant**
      toute contre-passation (findings R4-2, F4-3). C'est là qu'elle attend la transaction de test, et
      le motif d'`attendre_une_requete_en_cours` vise cette requête-là ; l'exercice du règlement est
      celui du jour, le règlement ayant été posé aujourd'hui. **200**, le règlement retiré une fois,
      **une seule** contre-passation.
   4. **Validation victime (#536)** — **le cycle de l'issue lui-même** (finding F5-5 de la P5) :
      facture brouillon créée par `POST /api/v1/invoices`, **avec un écart d'arrondi** (p. ex.
      HT 100.10 à 8,1 % → TTC 108.21, arrondi à 108.20), compte de différences d'arrondi désigné
      par **`kesh_db::test_fixtures::designate_rounding_account`** (`test_fixtures.rs:212`) — et
      non par son homonyme local de `reconciliation_e2e.rs:4681`, qui fait un upsert : celle de
      `kesh_db` pose le réglage par un `UPDATE` simple (`:225`), qui ne touche **aucune ligne, sans
      erreur**, si la ligne des réglages manque ; elle existe ici, semée par
      `seed_accounting_company`, et le test **vérifie** après l'appel que
      `default_rounding_account_id` vaut l'id rendu (finding R6-3) ; la
      transaction de test tient l'**exercice** ; `POST /api/v1/invoices/{id}/validate` vu en attente
      sur l'exercice (il tient la facture, la ligne des réglages et le **compte d'arrondi**, étape
      (2 bis'), `repositories/invoices.rs:2065`) ; la transaction de test demande le **compte
      d'arrondi** `FOR UPDATE` — l'ordre exercice → arrondi du lot de rapprochement — → la route
      est la victime ; **200**, facture `validated` avec son arrondi, **une seule** écriture de
      vente.
   5. **Saisie fournisseur victime** (choix C66, finding F4-1) — `POST /api/v1/supplier-invoices`,
      datée du jour, une ligne à **TVA 0 %** (le compte de TVA récupérable n'est alors pas exigé,
      `repositories/supplier_invoices.rs:129-131`) sur un compte de charge actif imputable du plan
      semé ; **préalables** (finding R5-2) : un contact **fournisseur** (`is_supplier`, contrôle
      (1) de `create_in_tx`, `repositories/supplier_invoices.rs:286-299`), créé par la route des
      contacts ou semé, et le **compte créanciers** désigné (`default_payable_account_id`, exigé
      après l'exercice) — sans l'un ou l'autre la route refuse avant d'attendre et
      `attendre_une_requete_en_cours` panique ; la transaction de test tient l'**exercice** ; la route, vue en
      attente sur `fiscal_years` + `FOR UPDATE`, tient déjà la ligne des réglages et le compte de
      charge (ordre de l'AC5) ; la transaction de test demande le **compte de charge** `FOR UPDATE`
      → victime ; **201**, **une seule** facture fournisseur, une seule écriture d'achat. Le test
      demande le compte de charge et non la ligne des réglages : il reste valable que la 15-5d soit
      mergée ou non (ses comptes désignés se prennent après les comptes de charge), et il ne dépend
      pas de l'hypothèse R3-7, que ce test ne tranche pas.
   6. **Le registre des routes** — une **seconde colonne de statut** dans le registre existant
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
        enveloppe** (finding F3-1). **Réécrit en P5** (findings R5-1 = F5-1, choix **C70**) : le
        scanner textuel des P3-P4 — retrait des commentaires en sautant les littéraux, fenêtre close
        à l'accolade en colonne 0, limite de mot — a eu **deux trous successifs** (les `{`/`}` des
        littéraux en P4, puis les durées de vie `'_`, `&'static str` et les littéraux octets `b';'`
        en P5, présents dans les fichiers visés) ; il est remplacé par l'**analyseur syntaxique de
        Rust**. Le test lit `crates/kesh-api/src/routes/<module>.rs` (le module est le premier
        segment de l'identité du registre, `"invoices::settle_invoice_handler"`), le **parse avec
        `syn` 2** (`syn::parse_file`), retrouve **par son nom** le `fn` du handler (un visiteur
        `syn::visit::Visit`, `visit_item_fn`, sur l'identifiant) et cherche dans **son corps** un
        appel à une enveloppe : un **`ExprCall`** dont la fonction est un chemin (`ExprPath`) dont le
        **dernier segment** est `retry_on_deadlock`, `retry_on_deadlock_with`,
        `retry_app_on_deadlock` ou `retry_with` (`retry_with(…)`, `kesh_db::retry::retry_with(…)`,
        `retry_with::<…>(…)`), ou un **`ExprMethodCall`** de l'un de ces noms. `syn` 2 est **déjà au
        `Cargo.lock`** (2.0.118, dépendance transitive) : il s'ajoute aux `[dev-dependencies]` de
        `kesh-api` (`syn = { version = "2", features = ["full", "visit"] }`), sans rien télécharger.
        Un commentaire, un doc-comment (attribut de l'item **qu'il précède**, hors du corps examiné),
        le contenu d'un littéral (chaîne, chaîne brute, octet, caractère), une durée de vie ne sont
        pas des appels : **aucun ne peut produire de faux vert ni désynchroniser la lecture**, et les
        précautions textuelles des P3-P4 (retrait des commentaires, fenêtre à l'accolade, limite de
        mot — un `mon_retry_with` n'a pas le dernier segment cherché) **disparaissent avec le
        source synthétique qui les éprouvait**. Le visiteur du corps ne descend pas dans les items
        qui y seraient déclarés (`fn`, `impl`, `mod` imbriqués). Restent exigés :
        1. **handler introuvable → échec explicite**, en nommant la route : fichier absent (module
           en répertoire), aucun `fn` de ce nom ou plusieurs, ou fichier que `syn` ne parse pas (son
           erreur est rendue) — une route `Rejouee` n'est jamais sautée ;
        2. **un test du visiteur sur un source synthétique** écrit au test : un appel réel (nu,
           qualifié, en turbofish) → **trouvé** ; le nom dans un commentaire `//` ou `/* */`, dans
           une chaîne `"retry_with("`, dans un doc-comment `/// … retry_with(…)` placé au-dessus du
           handler **suivant**, ou appelé dans le corps d'une **autre** fonction, ou préfixé
           (`mon_retry_with(…)`) → **non trouvé** ; une signature avec `'_`, `'a` et `&'static
           str`, un corps avec `b';'` et `r#"…"#` → le source **parse sans erreur** et l'appel qu'il
           contient est trouvé ; **des appels imbriqués** (finding R6-6) — receveur d'un `.await`
           suivi d'un `.map_err(…)?` et argument d'un `Ok(…)`, p. ex.
           `Ok(retry_on_deadlock("x", || async { … }).await.map_err(f)?)`, un appel dans le corps
           d'une fermeture `async`, un appel dans un bloc `{ … }` imbriqué → **trouvés**. Les
           surcharges `visit_expr_call` et `visit_expr_method_call` du visiteur **rappellent**
           l'implémentation par défaut (`syn::visit::visit_expr_call(self, node)`,
           `syn::visit::visit_expr_method_call(self, node)`) : sans ce rappel, le visiteur ne descend
           plus sous un appel et manque `retry_with(…).await.map_err(…)?` — ce que ces cas
           éprouvent ;
        3. **les mutations qui comptent** (plus bas) : retirer l'enveloppe d'une route `Rejouee` →
           le volet (c) rougit.
        **Limite écrite** : un appel placé **dans une macro** (`tokio::join!(…)`) n'est pas analysé
        par `syn`, ses jetons restant opaques — il rend « absente », c'est-à-dire un **faux rouge**,
        jamais un faux vert ; aucune route `Rejouee` ne place son enveloppe dans une macro.
        **Dans cette story**, `retry_with` est accepté pour toute route `Rejouee` (les quatre sites
        rejoués aujourd'hui l'appellent directement) ; la 15-5e2 restreint cette acceptation à
        `post_accept`.
      - **(d) la partition de rejeu recomptée depuis la source** (patron
        `the_registry_partition_is_what_the_story_declares`, `:453`), **avec son total** (finding
        F3-6) : **8** `Rejouee` (les 4 rejouées aujourd'hui + les 4 de l'AC3), **13** `ARejouer`,
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
      volet (c) est statique (il voit l'appel, pas le chemin de l'erreur), une conversion faite dans
      la fermeture le passerait (finding F2-10) —
      seuls les tests 2 à 5 et 7 le prouvent dynamiquement, pour cinq routes de la famille `DbError`
      (quatre qui écrivent au journal, et l'enregistrement des réglages ; finding R6-2) ;
      pour la famille `AppError` (dont les routes arrivent avec la 15-5e2), le test 1 éprouve le
      prédicat et l'enveloppe, et le chemin propre à chaque route relève de la revue fichier par
      fichier ; (iii) que le volet (c) ne voit pas une enveloppe appelée par une
      fonction auxiliaire du handler (le nom doit figurer dans le corps du handler lui-même : c'est la
      forme exigée) ; (iv) **l'angle mort assumé** (finding F2-5) : une route
      `SansEcritureAuJournal` qui prend un verrou peut être la **victime** d'un cycle avec un flux qui
      écrit au journal, et elle n'est pas rejouée — p. ex. `accept_batch` tient un verrou partagé sur
      la ligne `companies` dès qu'une proposition a inséré son écriture (`fk_journal_entries_company`),
      puis demande la sentinelle `FOR UPDATE` de cette ligne pour une règle à projet
      (`routes/reconciliation.rs:2493`) ; une route qui attend entre-temps un verrou exclusif sur `companies`
      (`companies::lock_books`, `unlock_books`, la sentinelle prise par les routes de `projects`,
      `bank_accounts`, `vat`, `dunning_levels`) ferme le cycle et, plus légère, rend 500. Rare, et hors
      de la promesse (« les routes qui écrivent au journal ») ; le registre établit que **toute route
      mutante a été examinée** — l'enregistrement des réglages de facturation, exposé par l'avance de
      l'AC5, en sort par le rejeu (point (vi)) ; (v) qu'il ne balaie que les verbes `post`, `put`, `delete` et
      `patch` (`extract_counted`, `:199`) : une route `GET` qui écrirait au journal lui échapperait
      — aucune ne le fait aujourd'hui (remontée de l'AC1 : seuls des `POST` et un `DELETE`), et
      l'angle mort est du même ordre que celui que le registre documente déjà pour l'audit
      (`GET /invoices/{id}/pdf`, `:116-121`) (finding F4-10). Au passage, l'en-tête du module, qui annonce **111 / 114** routes
      (`:20`, `:24-25`) quand les assertions disent **112 / 115**, est corrigé (finding R3-6) ;
      (vi) **deux routes `SansEcritureAuJournal` sont rejouées quand même** (choix C70) :
      `onboarding::finalize` (`retry_with`, `routes/onboarding.rs:614`) et `update_invoice_settings`
      (enveloppe `DbError`, AC3) ; la colonne dit l'inventaire de l'AC1, pas la présence d'une
      enveloppe, et le volet (c) ne les examine pas — la seconde est tenue par le **test 7**, la
      première par sa revue.
   7. **Réglages de facturation victimes** (choix C70, finding F5-6 de la P5 ; montage et portée
      précisés en P6, findings R6-3 et F6-1, choix C74) — `PUT /api/v1/company/invoice-settings`,
      avec un corps qui **change** un réglage (p. ex. le format de numéro) et la `version` courante :
      un corps identique est un no-op court-circuité **avant** l'`UPDATE`
      (`repositories/company_invoice_settings.rs:181-184`, KF-004), la route n'attendrait rien.
      Aucun exercice n'est requis. **La ligne des réglages existe** (semée par
      `seed_accounting_company`) : sans elle, le verrou partagé du test serait un verrou de
      **trou**, et la route attendrait sur l'`INSERT IGNORE` de `get_or_create_default`, hors de
      la fermeture (`routes/company_invoice_settings.rs:278`), et non sur l'`UPDATE` — motif jamais
      vu, panique. La **`version` courante** est lue par `GET /api/v1/company/invoice-settings`
      **avant** d'ouvrir la transaction de test. La transaction de test, alourdie, prend un verrou
      **partagé** sur la ligne des réglages (`SELECT … FROM company_invoice_settings WHERE
      company_id = ? LOCK IN SHARE MODE`) ; la route, vue en attente sur `UPDATE
      company_invoice_settings`, demande l'exclusif ; la transaction de test demande alors
      l'exclusif (`SELECT … FOR UPDATE` sur la même ligne). **Sur MariaDB 10.11, la version
      épinglée** (`mariadb:10.11` de `ci.yml:50`, `release.yml:21`, `docker-compose.yml:4` ; base
      de dev `10.11.16`), c'est un **cycle** — l'ancien comportement d'InnoDB : un `S` tenu, un `X`
      en attente devant, le détenteur du `S` qui demande le `X` attend derrière lui. **Ce n'est
      plus vrai partout** (finding F6-1) : MySQL 8.0.18 (bogue #11745929) puis MariaDB 11.4.5 et
      11.7.2 (MDEV-34877) accordent le `X` au détenteur du `S` **sans attendre** le `X` en file ;
      sur ces versions, le cycle ne se forme que si R3-7 est vraie (le `S` posé par l'`INSERT
      IGNORE` de la route, que le `X` du test doit alors attendre). La transaction de test
      **doit** obtenir son verrou (`.expect`), la route est annulée ; la transaction de test
      annule ; **200**, `version` incrémentée **une seule fois**, **une seule** entrée d'audit
      `company_invoice_settings.updated`, **et le témoin du rejeu** (montage commun) : au moins un
      `warn!` de `kesh_db::retry` portant `operation = "company_invoice_settings::update"`. Sans
      lui, sur une version qui ne forme pas le cycle, l'`.expect` réussit d'emblée, la route
      reçoit son `X` après l'annulation du test et réussit **au premier essai** — 200, une
      version, un audit, tout vert, et rien de prouvé. Le témoin rend ce cas **visible** (le test
      rougit) au lieu de le masquer ; un tel rouge, après une montée de version de MariaDB, dit
      que le montage ne forme plus son cycle, non que le rejeu est cassé.
   - **Aucun test de « non-interblocage »** : un test qui asserte que deux opérations concurrentes
     « réussissent toutes deux » ne prouve rien sous un ordre fautif (finding F1-4 de la P1), et
     l'absence de cycle n'est plus affirmée.
   - Le test existant `accept_replays_the_batch_when_it_is_the_deadlock_victim` reste vert sans
     modification.
   - **Mutations** (consignées au Dev Agent Record, en touchant le fichier après restauration — sans
     quoi cargo garde le binaire muté), **sur plusieurs familles** (finding F3-1) :
     - retirer l'enveloppe de `settle_invoice_handler`, de `cancel_invoice_settlement_handler`, de
       `validate_invoice_handler`, de `create_supplier_invoice` → le test 2, 3, 4, 5 rend 500 et
       rougit, **et** le volet (c) rougit ;
     - retirer l'enveloppe de `update_invoice_settings` → le **test 7** rend 500 et rougit (le volet
       (c) ne l'examine pas : route `SansEcritureAuJournal`, point (vi)) ;
     - **le témoin du rejeu** (finding F6-1, choix C74), aux tests **2 et 7** : retirer de la
       transaction de test la demande qui ferme le cycle (elle annule sans demander la ressource
       que la route tient) — la route reçoit son verrou après l'annulation et aboutit **au premier
       essai**, sans `warn!` : code de retour et comptes restent verts, **seul le témoin rougit**.
       C'est le cas d'une version de MariaDB qui ne forme pas le cycle du test 7 ; la même
       mutation, l'enveloppe retirée en plus, rougit pareillement (la requête aboutit, aucun
       `warn!`) ;
     - famille `retry_with` direct : retirer le `retry_with` de `post_cancel_reconciliation` → le
       volet (c) rougit ;
     - écrire le nom de l'enveloppe **en commentaire** dans le corps d'un handler `Rejouee` dont on a
       retiré l'appel → le volet (c) rougit toujours (un commentaire n'est pas un appel ; le source
       synthétique le prouve déjà, la mutation le confirme sur le source réel pour une ligne) ;
     - faire rendre à `retry_app_on_deadlock` le premier résultat sans rejeu (`max_attempts` à 1) →
       la partie « enveloppe » du test 1 rougit (un appel au lieu de deux) ;
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
     (2)        company_invoice_settings  INSERT IGNORE puis FOR UPDATE
                                    (get_or_create_default_in_tx,
                                    company_invoice_settings.rs:102-109)           (:2004-2006)
     (2 bis')   accounts            compte de différences d'arrondi, FOR UPDATE,
                                    seulement s'il y a un écart                     (:2033, :2065)
     (2 quater) accounts            verrou PARTAGÉ sur le compte de produit par la
                                    clé étrangère de invoice_lines.revenue_account_id,
                                    SEULEMENT si une ligne n'a pas encore de compte
                                    de produit (UPDATE invoice_lines de
                                    matérialisation)                                (:2113, :2141-2148)
     (3)        fiscal_years        find_open_covering_date                          (:2171-2172)
     (4)-(5)    invoice_number_sequences  le compteur, FOR UPDATE, SEULEMENT si la
                                    facture n'a pas encore de numéro (une
                                    revalidation reprend le sien sans le toucher)  (:2176, :2202-2206)
     (7)        journal_entries     create_in_tx — après l'exercice, le compteur des
                                    écritures FOR UPDATE
                                    (journal_entry_number_sequences, journal_entries.rs:369) ;
                                    l'insertion reprend des verrous
                                    PARTAGÉS : companies (fk_journal_entries_company),
                                    chaque compte écrit (fk_jel_account), chaque
                                    projet tagué (fk_jel_project)                   (:2241)
     ```
     (numéros de ligne relevés sur `c734645b`, inchangés sur `f289414e`, refaits en T0 ; l'étape
     (2 ter) ne prend aucun verrou, findings R2-6, F2-1 ; la condition de (2 quater) est celle du code,
     `if lines_before.iter().any(|l| l.revenue_account_id.is_none())`, `:2141-2148` — finding F3-7 ;
     celles de (2) et de (4)-(5), l'`INSERT IGNORE` qui précède le verrou des réglages et le compteur
     épargné à la revalidation, finding F4-6). La **15-5d** ajoute ses comptes
     désignés (créance, TVA due) **à l'étape (2 bis')**, après le compte d'arrondi et avant (3). Il dit
     **la règle** : cet ordre est une **convention qui réduit la fréquence** des interblocages entre
     flux qui la suivent ; il ne peut pas les exclure, puisque l'insertion des lignes reprend
     `accounts` **après** l'exercice ; **la défense est le rejeu des routes** (renvoi à l'enveloppe,
     AC2). Les phrases « Aucun chemin ne verrouille `accounts` avant `invoices` ou `fiscal_years` »
     (`:1921-1922`, fausse : le règlement client et le solde du reste le font) et « **Toute divergence
     de cet ordre = risque de deadlock** » (`:1928`) sont retirées. **Ne pas promettre la
     sérialisation par la ligne des réglages** (finding R3-7) : `get_or_create_default_in_tx`
     fait `INSERT IGNORE` **puis** `SELECT … FOR UPDATE` (`company_invoice_settings.rs:102-109`) ; sur
     une ligne déjà présente, l'`INSERT IGNORE` en doublon peut poser un verrou **partagé**, et deux
     transactions de la même société qui le tiennent puis demandent l'exclusif s'interbloquent au lieu
     de se sérialiser. Le texte dit donc « la ligne des réglages, tenue `FOR UPDATE` jusqu'au commit,
     ordonne la plupart des flux d'une même société ; un interblocage entre eux reste possible et il
     est rejoué » — jamais « sérialise ». L'hypothèse R3-7 est **forte** — un `INSERT IGNORE` qui
     rencontre un doublon pose un verrou partagé sur l'enregistrement existant, comportement
     documenté d'InnoDB, relevé par la lentille F de la P5 (finding F5-6) — et **mesurée en T0**
     (deux connexions, une dizaine de lignes de SQL, résultat consigné au Dev Agent Record, choix
     C70). Le texte ne dépend pas du résultat : le rejeu des flux qui passent par cette ligne
     (validation, saisie fournisseur, enregistrement des réglages ici ; avoir, complétion d'import à
     la 15-5e2) la rend sans conséquence pour l'utilisateur (choix C66, C70). Si T0 la confirme,
     réécrire `get_or_create_default_in_tx` (`SELECT … FOR UPDATE` d'abord, `INSERT IGNORE`
     seulement si la ligne manque) reste **hors de cette story** (C66, option (b) écartée) : le
     Dev Agent Record le signale à l'orchestrateur, qui ouvre une issue.
   - **Saisie fournisseur : les réglages avant les comptes de charge** (seul réordonnancement gardé,
     choix C55 ; rejeu de la route dans cette même story, choix C66).
     `supplier_invoices::create_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs:248` —
     donc aussi la complétion d'une facture importée, `crates/kesh-api/src/routes/imported_supplier_invoices.rs:243`)
     charge aujourd'hui les réglages (`get_or_create_default_in_tx`, `:379-381`) **après** l'exercice
     (`:374-377`). Désormais l'appel se place **après la passe de forme et avant la passe des
     comptes** de la 15-5a (finding F1-7 : un refus de forme ne prend ainsi aucun verrou ; passe de
     forme `:309`, passe des comptes `:326`) : projet (`:276-284`) → fournisseur (lecture,
     `:286-299`) → forme des lignes → **réglages** → comptes de charge → exercice → génération.
     - **Aucun refus ne bouge** : `get_or_create_default_in_tx` ne refuse rien (hors erreur de base) ;
       l'exigence `ConfigurationRequired("default_payable_account_id")` reste **à sa place**, après
       l'exercice — seul l'**appel** est avancé.
     - **Pourquoi celui-là** : il ne coûte qu'un déplacement d'appel ; il rend l'ordre « réglages →
       exercice » commun à la saisie et à la validation (l'inversion F2-8 de la 15-5d) ; et la
       **15-5d** en a besoin pour verrouiller ses candidats avant l'exercice. La ligne des réglages
       ordonne la plupart des saisies d'une même société — sans les sérialiser à coup sûr (R3-7,
       ci-dessus) ; le règlement client et fournisseur n'y passent pas. Elle en devient le **premier**
       verrou disputé entre deux saisies, avant l'exercice où elles s'attendaient jusqu'ici : si R3-7
       se vérifie, l'avance crée un cycle qui n'existait pas — d'où le rejeu de la saisie dans cette
       même story (AC3, choix C66), et non au merge suivant.
     - **Pas de tri des comptes de charge** (finding R1-3, choix C55) : le cycle qu'il viserait, deux
       saisies aux comptes croisés, est rendu rare par la ligne des réglages ; celui qui reste — une
       saisie dont un compte de charge est le compte d'arrondi, contre un règlement par compte interne
       avec écart — ne dépend pas de l'ordre des comptes de la saisie. Le rejeu de la route (ici ; de
       la complétion d'import, 15-5e2) couvre l'un et l'autre ; c'est écrit au doc-comment.
     - Le doc-comment de `create_in_tx` écrit son ordre de verrous — il devient, avec celui de
       `validate_invoice`, l'un des **deux doc-comments canoniques** auxquels le Pattern 5 renvoie
       sans recopier l'ordre (choix C68, 15-5e2) ; la 15-5d y ajoute ses comptes désignés (son AC9) —
       et renvoie à `validate_invoice` pour la règle.
   - **Le commentaire « 5 bis »** de `write_off_invoice`
     (`crates/kesh-db/src/repositories/invoice_settlements_write.rs:474-481`) est réécrit **en place,
     sans changer le code** : il affirme « aucun autre chemin ne prend de verrou `FOR UPDATE` sur le
     compte de TVA due […] si bien qu'aucun cycle n'est connu » — **faux** : le règlement client par
     compte interne verrouille le compte qu'on lui désigne, TVA due comprise (`:157-164`). Le texte
     neuf dit l'ordre que prend le flux (compte de la nature `:443-444`, puis arrondi `:486-490`,
     puis TVA due `:500`, puis exercice `:504` — finding R2-7), que l'insertion des lignes reprend
     ces comptes après l'exercice, et que la route est rejouée (elle l'est déjà,
     `routes/invoices.rs:1343`).
   - **Le doc-comment du module `crates/kesh-db/src/retry.rs` et celui de la constante
     `DEFAULT_MAX_DEADLOCK_ATTEMPTS`** (`:28-39`) : trois prémisses fausses voisines (finding R2-5 ;
     la troisième est dans la constante, finding R4-4) sont réécrites — `:7-9` « MariaDB ne détecte pas les deadlocks cross-table avant
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
   les ferme — règle d'inclusion du `CLAUDE.md`), section `## [0.13.0] — Non publié` (elle existe
   depuis le merge des 15-5a et 15-5b : `CHANGELOG.md:11`, rubriques « Modifié » `:13` et « Corrigé »
   `:17` ; motif exact exigé par `scripts/prepare-release.sh:189`).
   - Rubrique **Corrigé**, **dans la forme des entrées de la section** (finding F6-4 : une phrase
     de titre en **gras**, terminée par les issues en liens, `CHANGELOG.md:15`, `:19`, `:21`) :
     « **Une opération qui croise un rapprochement bancaire ne finit plus en erreur interne
     ([#463](https://github.com/guycorbaz/kesh/issues/463), [#491](https://github.com/guycorbaz/kesh/issues/491) ; [#536](https://github.com/guycorbaz/kesh/issues/536) en partie).** Un règlement client, l'annulation d'un règlement
     client, la validation d'une facture, la saisie d'une facture fournisseur et l'enregistrement
     des réglages de facturation qui croisent une autre opération (un rapprochement bancaire,
     notamment) ne finissent plus en erreur interne : le serveur rejoue l'opération, comme il le
     faisait déjà pour l'acceptation d'un rapprochement. Seul un interblocage répété trois fois de
     suite rend encore une erreur ; rien n'est alors écrit et l'opération peut être relancée telle
     quelle. » — « en partie » parce que #536 n'est que `refs` ici, la 15-5e2 la ferme (finding
     F5-7). La 15-5e2 remplace cette ligne par son texte final, étendu aux autres opérations.
   - Rubrique **Modifié** (finding F4-11 : le passage de `debug!` à `warn!` fait apparaître une ligne
     neuve dans les journaux du serveur au réglage par défaut, `RUST_LOG=info`), même forme : « **Les
     opérations rejouées après un interblocage sont journalisées en avertissement.** Chaque
     opération rejouée est désormais journalisée par le serveur au niveau *avertissement*, avec le
     nom de l'opération (`operation`) et le numéro de la tentative. »
   **Pas d'autre document** dans cette story : `docs/api-external.md` § 10, les manuels (#484) et le
   Pattern 5 sont à la 15-5e2. Aucun texte visible de l'utilisateur ne décrit aujourd'hui le 500 des
   cinq routes (`grep -nE "INTERNAL_ERROR|interblocage|deadlock" docs/api-external.md
   website/*.html README.md` sur `f289414e` : seules `api-external.md:307` et `:317`, propres au
   rapprochement, qui restent vraies). **Quatre documents restent faux entre le merge de cette story
   et celui de la 15-5e2** (finding F4-8 ; exception de la règle d'inclusion : plusieurs stories y
   contribuent, la 15-5e2 les réécrit) — à ne pas prendre pour un oubli : l'exemple « How to use
   the retry helper » du Pattern 5 (`docs/MULTI-TENANT-SCOPING-PATTERNS.md:338-351`), qui appelle
   `retry_with` sans nom d'opération et ne compilerait plus ; `docs/api-external.md`, qui ne dit le
   rejeu que du rapprochement (`:307`, `:317`) alors que le règlement, son annulation, la validation
   et la saisie fournisseur, ouverts aux clés API, le sont désormais (§ 10, `:401`, à la 15-5e2) ;
   `docs/manual/fr/user-manual.tex:909` (« transaction DB `SERIALIZABLE` » à la validation, l'une
   des routes d'ici — #484, 15-5e2) ; et `docs/manual/fr/admin-manual.tex:885-886` (« transactions
   explicites SERIALIZABLE pour les operations critiques », commentaire de `99-kesh.cnf` — même
   issue, même fiche, finding F5-7).
7. **AC7 — Aucune règle métier ne change, et c'est vérifié.** Aucun refus neuf, aucune priorité de
   refus déplacée, aucune variante de `DbError`, aucun code d'erreur, aucune clé i18n, aucun écran ;
   les écritures produites sont identiques ; la suite existante reste verte **sans modification de ses
   assertions** (les six appels de tests de `retry.rs` gagnent un nom, AC2 ; les tests existants
   d'`audit_route_registry.rs` adaptent leur déstructuration des tuples à la seconde colonne —
   `the_registry_partition_is_what_the_story_declares`, `:456-464`,
   `every_exemption_names_the_issue_that_follows_it`, `:493`, et les deux projections
   `.map(|(v, h, _)| …)` des volets (a)-(b), `:290` (`lib.rs`) et `:338` (`test_endpoints.rs`) —,
   AC4, findings R4-7, F5-3 ; liste relevée par `grep -nE "\|\(" crates/kesh-api/tests/audit_route_registry.rs`,
   refaite en T0, le compilateur rendant de toute façon l'arité fausse).

## Tasks / Subtasks

- [x] **T0 — Refaire les relevés** sur `HEAD` (les 15-5a et 15-5b sont mergées, `f289414e` ; la
      fiche est recalée sur ce commit) : la remontée de l'AC1 (commandes en Dev Notes), comparée à la
      table — **une route absente bloque la story** ; pour les cinq routes de l'AC3, le chemin
      d'erreur (aucune conversion qui fasse sortir une erreur sqlx de `DbError::Sqlx` avant le
      prédicat), l'absence d'effet de bord hors transaction et les entrées à cloner (AC2) ; numéros de
      ligne de `repositories/invoices.rs:1916-1929` et des étapes (1)–(7) de l'AC5,
      `invoice_settlements_write.rs:474-481`, `supplier_invoices.rs` (`create_in_tx`, tel que la 15-5a
      l'a réécrit) ; l'invariant « chaque flux de rapprochement prend son `GET_LOCK` avant tout verrou
      de ligne » (AC5, `retry.rs`) ; **l'hypothèse R3-7** (choix C70) — deux connexions sur une
      même société dont la ligne des réglages existe : la première exécute `INSERT IGNORE INTO
      company_invoice_settings (company_id) VALUES (?)` sans conclure, la seconde un `SELECT … FOR
      UPDATE` de la même ligne ; si la seconde attend (motif vu par
      `attendre_une_requete_en_cours`, ou `innodb_lock_wait_timeout` court), R3-7 est vraie ;
      `update_invoice_settings` y compris pour les trois relevés de l'AC2 — chemin d'erreur,
      **effets de bord** hors transaction, entrée clonée (finding R6-5) ; **le cycle du test 7,
      formé à la main avant d'écrire le test** (finding F6-1, choix C74), sur la version épinglée
      (`mariadb --version` consigné : 10.11) — trois connexions, la première alourdie qui prend
      `LOCK IN SHARE MODE` sur la ligne des réglages, la deuxième un `UPDATE
      company_invoice_settings … WHERE company_id = ?` qui attend (sans `INSERT IGNORE` préalable :
      c'est le cycle hors R3-7 qu'on éprouve), puis la première un `SELECT …
      FOR UPDATE` de la même ligne : la deuxième doit sortir en 1213 et la première obtenir son
      verrou. Si ce n'est pas le cas sur 10.11, le montage du test 7 est à revoir avant d'écrire
      une ligne de test (et le constat remonte à l'orchestrateur). À noter au Dev Agent Record :
      MySQL ≥ 8.0.18 et MariaDB ≥ 11.4.5 (MDEV-34877) peuvent ne pas former ce cycle, sauf si
      R3-7 est vraie — le témoin du rejeu rend ce cas visible au lieu de le masquer ;
      consigner au Dev Agent Record.
- [x] **T1 — Le patron et les cinq routes** (AC2, AC3) : le module `kesh_api::retry`
      (`retry_app_on_deadlock`, `is_app_deadlock`, `pub mod retry;`) ; le paramètre `operation` de
      `retry_with`, `retry_on_deadlock` et `retry_on_deadlock_with`, le `warn!` avec son champ ; les
      cinq sites directs, les six appels de tests et l'exemple `ignore` qui gagnent un nom ; les
      quatre routes des issues et de la saisie, et l'enregistrement des réglages de facturation
      (C70) ; doc-comments.
- [x] **T2 — L'ordre et les commentaires** (AC5) : l'appel des réglages de la saisie fournisseur ; le
      doc-comment canonique ; le commentaire « 5 bis » ; le doc-comment du module `retry.rs`.
- [x] **T3 — Les tests** (AC4) : les tests 1 à 5 et 7 ; la seconde colonne du registre
      `audit_route_registry.rs`, ses tests (c) — analyse par `syn` 2 (`[dev-dependencies]` de
      `kesh-api`), avec le test du visiteur sur source synthétique — et (d), l'en-tête 111/114
      corrigé ; mutations consignées.
- [x] **T4 — Documentation** (AC6) : les deux lignes du CHANGELOG (« Corrigé », « Modifié »).
- [x] **T5 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) —
      **même en cours de boucle de revue**, la story touchant des repositories `kesh-db` ; gate
      frontend complet (rien n'y change : il le confirme) ; **E2E Playwright complet au dernier commit
      de code** (décision D7), jugé fichier par fichier contre `docs/testing.md` § « Les échecs
      attendus ».

*(Décompte : 7 AC, 6 tâches T0–T5.)*

## Dev Notes

### Les routes qui écrivent au journal (relevé sur `b80ab8c0`, complété sur `c734645b`, recalé sur `f289414e`)

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
les motifs littéraux de la deuxième ne peuvent pas voir : relevé sur `c734645b`, **99 lignes**
(identique sur `f289414e`), dont
quatre sites qui écrivent réellement au journal — `backup.rs:457` (``DELETE FROM `{table}` ``) et
`:509` (``INSERT INTO `{table}` ``) dans `restore_tables_in_tx` → `POST /admin/full-import` ;
`test_fixtures.rs:401` (`TRUNCATE TABLE {table}`) dans `truncate_all` → `POST /api/v1/_test/seed`
et `/reset` (`test_endpoints.rs:172`, `:306`) ; `kesh-seed/src/lib.rs:233` (`reset_demo`) →
`POST /onboarding/reset`. Le reste : l'export (`admin_backup/export.rs`, `exports/global.rs`), la
validation de l'import (`admin_backup/import.rs`), des commentaires et des tests — lecture seule.

**Tri des sorties des deux premières commandes** (finding R2-4), à refaire en T0 :
- **homonymes sans rapport** (la première) : `users::create_in_tx`, `api_keys::create_in_tx`,
  `invoice_settlements::create_in_tx`, `reconciliation_rules::create_in_tx`,
  `imported_supplier_invoices::create_in_tx` (`crates/kesh-api/src/inbox_import.rs:543` : table de pré-import, aucune
  écriture au journal) ;
- **aides de test et de démontage** (la deuxième) : `crates/kesh-db/src/repositories/invoices.rs:4268`,
  `:4311`, `accounts.rs:1216`, `:1222` ; `journal_entries::delete_all_by_company`
  (`journal_entries.rs:1216`, **`pub`, hors `#[cfg(test)]`**, appelée seulement par des tests —
  `:1798`, `:1853`, …) ; les sites de tests de `journal_entries.rs` (`:1858`, `:2350`, `:2374`,
  `:2580`, `:3425`) ;
- **faux amis du motif** `journal_entr` : `journal_entry_number_sequences` (`kesh-seed/src/lib.rs:272`)
  et `journal_entry_lines` ; `kesh-seed/src/lib.rs:257-260` est `reset_demo` (exemptée).

Les numéros de ligne de la colonne « route » sont ceux des **handlers** (`crates/kesh-api/src/routes/…`) ;
ceux des dépôts sont préfixés de leur chemin (finding R3-6).

| route (handler, `crates/kesh-api/src/routes/…`) | écriture au journal (dépôt) | aujourd'hui | rejeu | fiche |
|---|---|---|---|---|
| `POST /invoices/{id}/write-off` (`invoices.rs:1328`) | `invoice_settlements_write::write_off_invoice` | **rejouée** (`:1343`, `retry_with`) | enveloppe `DbError`, `"invoices::write_off"` | nom : 15-5e1 ; migration : 15-5e2 |
| `POST /reconciliation/accept` (`reconciliation.rs:730`) | `accept_one_invoice` / `_split` / `_rule` → `create_in_tx` | **rejouée** (`:888`, prédicat 1213 + 1305) | inchangée, `"reconciliation::accept"` | nom : 15-5e1 |
| `POST /reconciliation/transactions/{id}/cancel` (`reconciliation.rs:3951`) | `reconciliation_cancel::cancel_in_tx` → `reverse_in_tx`, `cancel_settlement_in_tx` | **rejouée** (`:3960`, `retry_with`) | enveloppe `AppError`, `"reconciliation::cancel"` | nom : 15-5e1 ; migration : 15-5e2 |
| `POST /opening-balances/complete` (`opening_balances.rs:579`) | `opening_complement::create_opening_complement` → `create_in_tx` | **rejouée** (`:594`, `retry_with`) | enveloppe `DbError`, `"opening_balances::complete"` | nom : 15-5e1 ; migration : 15-5e2 |
| `POST /invoices/{id}/validate` (`invoices.rs:869`) | `invoices::validate_invoice` → `create_in_tx` | 500 | enveloppe `DbError`, `"invoices::validate"` — #536 | **15-5e1** |
| `POST /invoices/{id}/settlements` (`invoices.rs:1261`) | `settle_invoice` → `create_in_tx` | 500 | enveloppe `DbError`, `"invoices::settle"` — #491 | **15-5e1** |
| `POST /invoices/{id}/settlements/{sid}/cancel` (`invoices.rs:1478`) | `cancel_settlement` → `reverse_owned_in_tx` | 500 | enveloppe `DbError`, `"invoices::cancel_settlement"` — #463 | **15-5e1** |
| `POST /invoices/{id}/unvalidate` (`invoices.rs:911`) | `invoices::unvalidate` → `delete_in_tx` | 500 | `DbError` ; la `version` du corps rend une tentative périmée en 409 | 15-5e2 |
| `POST /credit-notes` (`credit_notes.rs:183`) | `credit_notes::create_credit_note` → `create_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /supplier-invoices` (`supplier_invoices.rs:330`) | `supplier_invoices::create` → `create_in_tx` | 500 | enveloppe `DbError`, `"supplier_invoices::create"` — avance des réglages (AC5), choix C66 | **15-5e1** |
| `POST /supplier-invoices/{id}/pay` (`supplier_invoices.rs:370`) | `supplier_invoices::pay` → `pay_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /supplier-invoices/{id}/cancel` (`supplier_invoices.rs:398`) | `supplier_invoices::cancel` → `reverse_owned_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /supplier-invoices/{id}/settlement/cancel` (`supplier_invoices.rs:424`) | `supplier_invoices::cancel_settlement` → `reverse_owned_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /imported-supplier-invoices/{id}/complete` (`imported_supplier_invoices.rs:123`) | transaction du handler → `supplier_invoices::create_in_tx` | 500 | `AppError`, fonction « une tentative » | 15-5e2 |
| `POST /payment-batches/{id}/confirm` (`payment_batches.rs:241`) | `payment_batches::confirm_batch` → `pay_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /journal-entries` (`journal_entries.rs:469`) | `journal_entries::create` | 500 | `DbError`, correspondance `FiscalYearClosed` après | 15-5e2 |
| `DELETE /journal-entries/{id}` (`journal_entries.rs:611`) | `journal_entries::delete_by_id` → `delete_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /journal-entries/{id}/reverse` (`journal_entries.rs:452`) | `journal_entries::reverse` → `reverse_in_tx` | 500 | `DbError` | 15-5e2 |
| `POST /opening-balances` (`opening_balances.rs:341`) | `journal_entries::create_opening_entry` | 500 | `DbError`, `map_opening_balances_error` après | 15-5e2 |
| `POST /reconciliation/manual` (`reconciliation.rs:3026`) | transaction du handler + verrou nommé → `create_in_tx` | 500 | `AppError`, fonction « une tentative » | 15-5e2 |
| `POST /reconciliation/split` (`reconciliation.rs:3462`) | transaction du handler + verrou nommé → `create_in_tx` | 500 | `AppError`, fonction « une tentative » | 15-5e2 |
| `POST /admin/full-import` (`admin.rs:137`, `full_import`) | `kesh_db::backup::restore_tables_in_tx` (toutes les tables) | 500 | **exemptée** : restauration d'instance, geste d'administration exclusif hors exploitation ; un 1213 y annule la transaction unique, et la relance manuelle est sûre | registre : 15-5e1 |
| `POST /onboarding/reset` (`onboarding.rs:240`) | `kesh_seed::reset_demo` (`DELETE FROM journal_entries`) | 500 | **exemptée** : effacement de la démo, même raison | registre : 15-5e1 |
| `POST /api/v1/_test/seed` (`test_endpoints.rs:49`, `seed_handler`) | `test_fixtures::truncate_all` (`TRUNCATE TABLE`, `test_fixtures.rs:401`) puis préréglages | 500 | **exemptée** : mode test, jamais monté en production (`lib.rs:1056-1065`) ; sérialisée par son propre verrou (`seed_lock`) | registre : 15-5e1 |
| `POST /api/v1/_test/reset` (`test_endpoints.rs:50`, `reset_handler`) | `test_fixtures::truncate_all` | 500 | **exemptée** : même raison | registre : 15-5e1 |

**Décompte** (recompté sur la table) : 4 rejouées aujourd'hui + 4 rejouées ici + 13 à la 15-5e2 =
**21** routes de `lib.rs` qui écrivent au journal ; **4** exemptées (2 de `lib.rs`, 2 de
`test_endpoints.rs`) ; total **25**. Le registre range les **90** autres routes mutantes (89 de
`lib.rs`, `/password-reset-token`) en `SansEcritureAuJournal` : 25 + 90 = **115**. Les cinq sites
`retry_with` du dépôt : les quatre premières lignes et `onboarding::finalize`, qui n'écrit pas au
journal.

**Le chemin d'erreur, relevé** (`grep -nE "map_err\(\|[a-z_]*\|"` sur les dépôts concernés) : les seuls
`map_err` qui ne passent pas par `map_db_error` convertissent des erreurs **non sqlx** (rendu d'un
numéro, `crates/kesh-db/src/repositories/credit_notes.rs:390`, `repositories/invoices.rs:2208` ;
`last_insert_id`, `repositories/journal_entries.rs:422` ; équilibre, `repositories/opening_complement.rs:622`) :
aucun ne peut masquer un 1213. Dans `post_manual` et `post_split`, `ReconciliationError::Db(db)` et
`::Database(e)` ressortent en `AppError::Database` (`routes/reconciliation.rs:3358-3365`, et
`:3821-3828` pour le second) : le prédicat les voit. Les `map_err(|e| …DbError::Sqlx(e))` écrits à
la main (p. ex. `cancel_reconciliation_once`, `:3991`) gardent eux aussi le 1213 en `DbError::Sqlx`
— la règle de l'AC2 porte sur l'effet, pas sur la forme.

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
déjà annulé, par sa propre garde ; un enregistrement des réglages dont la `version` a changé
entre-temps, en 409 (`OptimisticLockConflict`). Le rejeu n'ajoute aucun cas qui n'existe déjà pour deux requêtes
successives.

**Le règlement partiel concurrent** (finding F6-3) est le cas que le rejeu rend précisément
probable : le cycle de #491 exige **la même facture** — le rapprochement la règle pendant que le
règlement manuel la règle —, si bien que la tentative rejouée arrive toujours **après** le
règlement concurrent. Facture soldée : refus pour trop-perçu, ci-dessus. Règlement concurrent
**partiel** : la tentative rejouée enregistre le second règlement **sans rien signaler**, là où le
500 d'aujourd'hui faisait recharger l'écran et montrait le règlement bancaire. Ce n'est pas un cas
neuf — deux requêtes successives donnent le même résultat — et le règlement manuel porte un
montant, que la facture borne ; mais **aucune `version` ne le garde**, à la différence du solde du
reste, qui n'a pas de montant et porte une `version` pour cette raison même
(`routes/invoices.rs:1293-1304`, doc-comment de `WriteOffInvoiceRequest`, le corps de
`write_off_invoice_handler`). Constat
écrit, pas une garde de cette story : en ajouter une serait une règle métier neuve (AC7).

### Ce que vit l'utilisateur

- Aujourd'hui : la victime d'un interblocage reçoit `500 INTERNAL_ERROR` (« erreur interne ») et doit
  recommencer ; rien n'est écrit. C'est le cas du règlement (#491), de l'annulation de règlement
  (#463), de la validation contre un lot de rapprochement (#536) et d'une saisie fournisseur prise
  dans un cycle (comptes de charge croisés avec une autre saisie, compte d'arrondi contre un
  règlement) ; et, si R3-7 se vérifie, de l'enregistrement des réglages de facturation pris dans
  un cycle sur leur ligne ; l'acceptation du lot, elle, rejoue déjà.
- Après : l'opération aboutit, avec au pire ≈ 150 ms de latence ajoutée. Seul un interblocage répété
  trois fois de suite ressort en 500, comme avant.

### Ce que la réécriture C54 a retiré (pour mémoire)

« L'arrondi d'abord » au règlement client et au solde du reste, leurs priorités de refus, les trois
tests à sonde `NOWAIT`, toute affirmation de cycle fermé et l'inventaire des sites qui verrouillent
`accounts` (détail au Change Log de la fiche index `15-5e-ordre-des-verrous-reglements.md`).

### L'ordre des verrous des flux comptables, pour mémoire (relevé sur `b80ab8c0`, recalé sur `f289414e`)

Non normatif : c'est ce que le doc-comment canonique décrit et ce que la 15-5e2 écrit au Pattern 5.
Chaque flux reprend en outre, à l'insertion de l'écriture et de ses lignes, des verrous **partagés**
sur la ligne `companies` (`fk_journal_entries_company`), sur chaque compte écrit (`fk_jel_account`) et
sur chaque projet tagué (`fk_jel_project`) — finding F2-1. Tous les numéros de ligne ci-dessous sont
ceux des **dépôts** (`crates/kesh-db/src/repositories/…`), sauf mention contraire (finding R3-6).

| flux | réglages | comptes `FOR UPDATE` | exercice |
|---|---|---|---|
| validation d'une facture | `repositories/invoices.rs:2006` | arrondi `:2065` si écart (la 15-5d y ajoute la créance et la TVA due) | `:2172` |
| règlement client | — | compte interne `repositories/invoice_settlements_write.rs:157-164` (virement : `bank_accounts` `:119-133`, aucune ligne d'`accounts`), puis arrondi `:195-207` si écart | `:210-213` |
| solde du reste | lus sans verrou (`usable_designated_account`, `repositories/company_invoice_settings.rs:484`) | nature `invoice_settlements_write.rs:441-444`, arrondi `:482-493` si reste hors centime, TVA due `:498-501` | `:503-506` |
| saisie / complétion fournisseur | **avancés** avant les comptes de charge (AC5 ; aujourd'hui `repositories/supplier_invoices.rs:379-381`, après l'exercice) | charges `:301-372` (passe des comptes `:326`, `FOR UPDATE` `:345`), dans l'ordre des lignes | `:374-377` |
| règlement fournisseur par compte interne | — | compte interne `repositories/supplier_invoices.rs:660-668` | `:693` |
| avoir (`create_credit_note`) | **verrouillés** (`repositories/credit_notes.rs:359-361` → `company_invoice_settings.rs:109`) | arrondi `repositories/credit_notes.rs:525` | `:370` — avant l'arrondi (#536) |
| lot de rapprochement (`accept_batch`) | — | arrondi `routes/reconciliation.rs:1534` par proposition | `:1567` ; règle à projet : sentinelle **après** l'exercice (`:2461` → `:2493`) ; ventilation : sentinelle et projets par l'étape 0 de `create_in_tx` après l'exercice (`accept_one_split`, `:2108` → `:2150`) ; entre propositions, exercice de la proposition n avant l'arrondi de la n+1 (#536) |
| onboarding (`insert_with_defaults_in_tx`) | insérés **après** les comptes de rôle (`company_invoice_settings.rs:699-721`) | comptes de rôle | créé ensuite |

### Fichiers touchés (prévision)

`crates/kesh-api/src/retry.rs` (neuf) et `lib.rs` (déclaration du module) ;
`crates/kesh-api/src/routes/invoices.rs` (trois routes rejouées ; nom au `retry_with` de
`write_off_invoice_handler`) ; `routes/supplier_invoices.rs` (la saisie rejouée, choix C66) ;
`routes/company_invoice_settings.rs` (l'enregistrement des réglages rejoué, choix C70) ;
`routes/{reconciliation,opening_balances,onboarding}.rs` (un nom d'opération à chacun des sites
`retry_with`, rien d'autre) ; `crates/kesh-db/src/retry.rs` (paramètre `operation`, `warn!`, doc du
module et de la constante, six appels de tests, exemple `ignore`) ;
`crates/kesh-db/src/repositories/{invoices,invoice_settlements_write,supplier_invoices}.rs`
(commentaires ; le code ne bouge que dans `supplier_invoices`) ;
`crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (neuf) ; `crates/kesh-api/tests/common/capture_rejeu.rs`
(neuf, la couche de capture du témoin du rejeu, choix C74) et `tests/common/mod.rs` (sa
déclaration) ; `crates/kesh-api/tests/audit_route_registry.rs`
(seconde colonne, tests (c) et (d), en-tête) ; `crates/kesh-api/Cargo.toml` (`syn` 2 en
`[dev-dependencies]`, choix C70) et `Cargo.lock` (la ligne de `syn 2.0.118` ajoutée aux dépendances
de `kesh-api`, aucune version neuve) ; `CHANGELOG.md`. **Aucune migration** (P1–P8 sans
objet), aucun fichier `kesh-i18n` ni `frontend`, aucun manuel.

### Dérogation règle de splitting

*(Finding **F4-2 MEDIUM** de la P4 ; choix **C67**. Remplace la phrase qui déclarait la story « sous
le seuil de la règle de découpage pour le code qui se conçoit » : la règle ne connaît pas ce critère.)*

**Décompte, selon la méthode de la règle** (§ *Règle de splitting préventif* du `CLAUDE.md` : modules
distincts au grain « `kesh-api/routes/invoices` », tests et CHANGELOG exclus, sites « forcés »
compris — la règle ne les exclut pas) : **11 modules de production** dans **2 crates**, 12 si l'on
compte la ligne `pub mod retry;` de `kesh-api/lib` (10 jusqu'à la P5 ; le onzième vient de C70) —
- **le patron** (2) : `kesh-api/retry` (neuf), `kesh-db/retry` ;
- **les sites pilotes** (2) : `kesh-api/routes/invoices` (trois routes des issues),
  `kesh-api/routes/supplier_invoices` (la saisie, C66) ;
- **les sites forcés par le changement de signature** (3) : `kesh-api/routes/{reconciliation,
  opening_balances,onboarding}`, une ligne chacun (le quatrième, dans `routes/invoices`, est déjà
  compté) ;
- **l'ordre des verrous que la 15-5d attend** (3) : `kesh-db/repositories/{invoices,
  invoice_settlements_write}` (commentaires), `kesh-db/repositories/supplier_invoices` (un appel
  déplacé) ;
- **la route exposée par l'avance des réglages** (1, choix C70) : `kesh-api/routes/company_invoice_settings`
  (un appel enveloppé).
Le test du registre, son analyseur `syn` (dépendance de test) et le `Cargo.toml` ne sont pas des
modules de production, la règle excluant les tests. Le seuil (« plus de 5 modules ») est **franchi**.

**Pourquoi ne pas redécouper.** Cette story **est déjà** la story-zéro du patron « story-zéro +
rollout » que la règle prescrit (C61) : le patron et ses premiers sites pilotes. Un découpage de plus
séparerait le registre et les enveloppes de leurs premiers utilisateurs — un patron livré sans site
qui l'exerce, dont les tests « route victime » arriveraient dans une autre PR ; ou bien une avance des
réglages livrée sans la défense qui la rend sûre (C66), c'est-à-dire précisément la régression que la
P4 a relevée. Les trois sites forcés **suivent du choix de signature** (C62) : une fois `retry_with`
doté du paramètre `operation`, ils compilent avec lui ou pas du tout. Une variante nommée à côté d'un
`retry_with` inchangé les aurait renvoyés au rollout — huit modules ici au lieu de onze — au prix de
cinq rejeux sans nom jusqu'au merge de la 15-5e2 ; C62 l'a écartée pour que tout rejeu dise son nom :
c'est un choix, pas une impossibilité (finding F5-8). Restent les trois dépôts de l'AC5, que C61 (c) a délibérément gardés ici pour que la 15-5d ne
dépende que du socle ; les déplacer dans la 15-5d rouvrirait C61 et C65 pour deux commentaires et un
appel. **Ce n'est pas une exception de la règle** (ni cycle Cargo, ni merge intermédiaire non
testable) : c'est un **arbitrage**, déclaré comme tel.

**Risque accepté.** Une revue adversariale moins sûre sur un périmètre de onze modules : les passes de
revue de code devront nommer chaque module dans leurs axes exercés. Atténué par la nature des
sites : cinq des onze ne changent qu'un commentaire ou un argument de chaîne
(`routes/{reconciliation,opening_balances,onboarding}`, `repositories/{invoices,invoice_settlements_write}`),
un ne fait que déplacer un appel (`repositories/supplier_invoices`), un n'enveloppe qu'un appel de
dépôt (`routes/company_invoice_settings`), et la conception se concentre dans les quatre autres
(`kesh-api/retry`, `kesh-db/retry`, `routes/invoices`, `routes/supplier_invoices`) — 5 + 1 + 1 + 4
= 11 (finding R5-5 : la phrase d'avant n'en comptait que neuf).

**Signal D5 déclaré au Project Lead** (amendement D5 de la règle) : le seuil de dispersion est
franchi ; la story n'est pas redécoupée ; l'arbitrage est au registre (C67) et au Change Log. Il
l'est **à nouveau en P5** (C70) : le décompte passe à onze modules, et le volet (c) sort d'un
recyclage — deux trous successifs du scanner textuel — par un changement de méthode.

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C43** — côté achat, les réglages avancés avant les comptes de charge (le reste de C43, révisé par
  C48, est retiré par C55).
- **C54** — la défense contre l'interblocage est le rejeu, pas un ordre parfait des verrous.
- **C55** — ce qui reste de l'ordre des verrous : l'avance des réglages de la saisie fournisseur et
  des commentaires vrais (**révisé par C66** : la ligne des réglages ne « sérialise » pas — R3-7).
- **C56** — la forme du rejeu : deux enveloppes partagées et le registre (sa dérogation de découpage
  est **révisée par C61**, ses « deux routes exemptées » par C58).
- **C57** — #536 : couverte pour la validation ici, fermée par la 15-5e2 (avoir) — `closes` porté
  par la 15-5e2 depuis C61 (constat écrit en C69).
- **C58** — le registre est une seconde colonne du registre d'audit existant.
- **C59** — les rejeux journalisés en `warn` avec le nom de l'opération (**révisé par C62** : le nom
  est un champ de l'événement, pas un span).
- **C61** — le découpage 15-5e1 / 15-5e2.
- **C62** — les noms des enveloppes et le nom d'opération porté par l'événement.
- **C63** — le statut transitoire `ARejouer` et le volet (c) robuste.
- **C64** — la ligne du CHANGELOG voyage avec les issues qu'elle ferme ; PDF de la brochure.
- **C65** — la 15-5d dépend de la 15-5e1 seule (**révisé par C66** : la saisie fournisseur est
  rejouée ici ; **et par C68** : l'ordre de merge 15-5d / 15-5e2 est sûr aussi pour le Pattern 5).
- **C66** — la saisie fournisseur rejouée dans cette story, avec l'avance de ses réglages ; R3-7 non
  tranchée, rendue sans conséquence par le rejeu.
- **C67** — dérogation de découpage : dix modules, story-zéro non redécoupée ; signal D5 déclaré
  (**révisé par C70** : onze modules).
- **C68** — le Pattern 5 renvoie aux doc-comments canoniques (`validate_invoice`, `create_in_tx`
  fournisseur) au lieu de recopier l'ordre (15-5e2).
- **C69** — remédiations P4 de moindre portée (volet (c) à l'accolade en colonne 0 et aux littéraux —
  **remplacé par C70**, analyse par `syn` —,
  test de l'enveloppe `AppError`, ligne « Modifié » du CHANGELOG, frontière des mentions de
  `retry_with` d'`opening_complement.rs`).
- **C70** — remédiation de la P5 : le volet (c) analysé par `syn` 2 (sortie d'un recyclage, signal
  D5 déclaré) ; l'enregistrement des réglages de facturation rejoué (test 7), `SansEcritureAuJournal`
  au registre ; R3-7 hypothèse forte, mesurée en T0 ; test 4 sur le cycle de #536 ; harnais E2E
  recopié, copie assumée (**corrigé par C74** : le cycle du test 7 n'est pas « indépendant de
  R3-7 » sur toutes les versions).
- **C74** — remédiation de la P6 : un **témoin du rejeu** (événement `warn!` de `kesh_db::retry`
  capté par un abonné `tracing` de test) aux tests 2 à 5 et 7 ; le cycle du test 7 formé à la main
  en T0 sur MariaDB 10.11, et non garanti sur MariaDB ≥ 11.4.5 ou MySQL ≥ 8.0.18 ; montage par
  `seed_accounting_company` ; arrondi désactivé aux tests 2 et 3.

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

Claude Opus 5.5 (agent de développement, autonomie complète — Epic 15).

### Debug Log References

- Mesures T0 : base dédiée `kesh_155e1` sur `10.11.16-MariaDB-ubu2204`, `innodb_deadlock_detect = 1`.
- Gate backend : `scripts/test-fast.sh` (DATABASE_URL → `kesh_155e1`, remise à zéro DROP/CREATE + migrations + seed juste avant).
- E2E : backend du worktree (binaire copié) sur le port 3002, base `kesh_e2e_155e1` neuve, `KESH_COOKIE_SECURE=false`, SMTP factices, répertoires inbox/documents dédiés.

### Completion Notes List

**T0 — relevés sur `HEAD` (`778513aa`)**
- **Code identique à `f289414e`** : `git diff --stat f289414e HEAD -- crates` est vide ; les numéros de ligne de la fiche valent donc tels quels.
- **Remontée de l'AC1 refaite** (les quatre commandes des Dev Notes) : 57 lignes pour la première (commentaires exclus), 18 pour la deuxième, 99 pour la troisième (identique au relevé de la fiche). Chaque site d'écriture au journal se rattache à une ligne de la table ; aucune route absente. Les homonymes, aides de test et faux amis sont ceux que la fiche trie (`accounts.rs:1302`, `:1351` sont sous `#[cfg(test)]`, `:1089`).
- **Chemin d'erreur des cinq routes** : les seuls `map_err` qui ne passent pas par `map_db_error` dans `invoice_settlements_write`, `supplier_invoices`, `company_invoice_settings`, `invoices` et `journal_entries` convertissent des erreurs non sqlx (`last_insert_id`, rendu du numéro). **Effets de bord** : `grep -nE "tokio::fs|std::fs|smtp|mailer|reqwest"` sur ces dépôts : vide. **Entrées clonées** : `SettlementChoice` est `Copy` (règlement), `NewSupplierInvoice` et `CompanyInvoiceSettingsUpdate` sont `Clone`, la validation et l'annulation ne prennent que des identifiants.
- **Invariant `GET_LOCK`** : les cinq `begin()` de `routes/reconciliation.rs` (`:926`, `:2749`, `:3170`, `:3687`, `:3995`) sont suivis de `with_account_lock` sans requête intermédiaire ; `with_account_lock` lit `DATABASE()` puis `GET_LOCK` avant la fermeture. L'invariant tient.
- **R3-7 : VRAIE** sur 10.11.16. `INSERT IGNORE` en doublon non conclu, puis `SELECT … FOR UPDATE` d'une autre connexion → **1205** ; `INNODB_LOCKS` montre un verrou **`S` RECORD** sur `PRIMARY` tenu par l'`INSERT IGNORE`. ⚠️ **À l'orchestrateur** : ouvrir l'issue prévue par l'AC5 (réécrire `get_or_create_default_in_tx` : `SELECT … FOR UPDATE` d'abord, `INSERT IGNORE` seulement si la ligne manque — hors de cette story, C66 (b)). Choix **C-15-5e1-1**.
- **Cycle du test 7 formé à la main** : connexion lourde (500 lignes de lest) en `LOCK IN SHARE MODE`, `UPDATE company_invoice_settings` d'une autre connexion en attente, puis `SELECT … FOR UPDATE` de la lourde → **l'`UPDATE` sort en 1213, la lourde obtient son verrou**. Le montage tient sur la version épinglée, sans adaptation. Note : MySQL ≥ 8.0.18 et MariaDB ≥ 11.4.5 (MDEV-34877) accordent l'`X` au détenteur du `S` sans attendre ; mais `company_invoice_settings::update` commence par un `INSERT IGNORE` et, R3-7 étant vraie, la route y tient un `S` : le cycle du test devrait s'y former aussi (non mesuré hors 10.11). Le témoin du rejeu reste la garde.

**T1 — le patron et les cinq routes**
- `kesh_db::retry` : `retry_with(operation, max_attempts, should_retry, f)`, `retry_on_deadlock(operation, f)`, `retry_on_deadlock_with(operation, max_attempts, f)` ; `tracing::warn!(target: "kesh_db::retry", operation, attempt, max_attempts, backoff_ms, …)` ; six appels de tests et exemple `ignore` nommés.
- `kesh_api::retry` (neuf, `pub mod retry;`) : `is_app_deadlock`, `retry_app_on_deadlock`.
- Cinq sites directs de `retry_with` nommés (`reconciliation::accept`, `reconciliation::cancel`, `invoices::write_off`, `opening_balances::complete`, `onboarding::finalize`), rien d'autre.
- Rejouées par l'enveloppe `DbError` : `invoices::validate`, `invoices::settle`, `invoices::cancel_settlement`, `supplier_invoices::create` (`NewSupplierInvoice` cloné par tentative), `company_invoice_settings::update` (`CompanyInvoiceSettingsUpdate` cloné ; lectures préalables laissées hors de la fermeture). Une ligne de doc-comment chacune.

**T2 — ordre et commentaires**
- `supplier_invoices::create_in_tx` : `get_or_create_default_in_tx` avancé entre la passe de forme et la passe des comptes ; `ConfigurationRequired("default_payable_account_id")` reste après l'exercice. Doc-comment « # Ordre des verrous » écrit (renvoi à `validate_invoice` pour la règle).
- `validate_invoice` : doc-comment canonique réécrit (étapes (1), (1 bis), (2), (2 bis'), (2 quater), (3), (4)-(5), (7), règle « convention qui réduit la fréquence ; la défense est le rejeu », jamais « sérialise ») ; les deux phrases fausses retirées. **Sans numéros de ligne** (choix **C-15-5e1-2**).
- « 5 bis » de `write_off_invoice` réécrit en place (ordre nature → arrondi → TVA due → exercice, cycle possible, route rejouée), code inchangé.
- Doc du module `kesh_db::retry` et de `DEFAULT_MAX_DEADLOCK_ATTEMPTS` : les trois prémisses fausses remplacées ; `GET_LOCK` et `innodb_deadlock_detect` écrits.

**T3 — tests** (nouveaux, périmètre `HEAD` `778513aa` → arbre de travail)
- `rejeu_interblocage_e2e.rs` : **6** tests (1 à 5 et 7) ; `common/capture_rejeu.rs` : couche de capture du témoin (aucune crate ajoutée).
- `audit_route_registry.rs` : seconde colonne `Rejeu` sur les 115 lignes ; **3** tests neufs — volet (c) `every_replayed_route_calls_an_envelope` (par `syn` 2), `the_envelope_visitor_sees_calls_and_only_calls` (source synthétique), volet (d) `the_replay_partition_is_what_the_story_declares` (8 / 13 / 4 / 90 = 115) ; en-tête 111/114 → 112/115 ; doc-comment du module (points (i) à (vi)). Déstructurations adaptées : la partition d'audit, l'exemption, les deux projections.
- `Cargo.toml` : `syn = { version = "2", features = ["full", "visit"] }` en `[dev-dependencies]` ; `Cargo.lock` : une ligne `"syn 2.0.118"` aux dépendances de `kesh-api`, aucune version neuve.
- **Harnais recopié** (`test_config`, `spawn_app`, `forge_jwt`) : copie assumée (C70) ; la factorisation du harnais E2E de `kesh-api/tests` reste à ouvrir en issue par l'orchestrateur.
- **Mutations** (choix **C-15-5e1-3** ; lancées sur `binary(rejeu_interblocage_e2e) | binary(audit_route_registry)`, fichier restauré puis `touch`) — toutes rouges comme attendu :

  | mutation | rouge observé |
  |---|---|
  | enveloppe retirée de `settle_invoice_handler` | test 2 (500) + volet (c) |
  | … de `cancel_invoice_settlement_handler` | test 3 (500) + volet (c) |
  | … de `validate_invoice_handler` | test 4 (500) + volet (c) |
  | … de `create_supplier_invoice` | test 5 (500) + volet (c) |
  | … de `update_invoice_settings` | test 7 (500) seul (volet (c) ne l'examine pas) |
  | témoin : la demande qui ferme le cycle retirée (aide `victime`) | **seul le témoin** rougit, aux tests 2, 3, 4, 5 et 7 (statuts et comptes verts) |
  | `retry_with` retiré de `post_cancel_reconciliation` | volet (c) |
  | enveloppe de `settle_invoice_handler` retirée **et** nommée en commentaire dans le corps | volet (c) + test 2 |
  | `retry_app_on_deadlock` à une tentative | test 1 (un appel au lieu de deux) |
  | prédicat qui reconnaît le 1205 | test 1 |

**T4** — `CHANGELOG.md`, `[0.13.0] — Non publié` : une entrée « Modifié » (journalisation en avertissement) et une entrée « Corrigé » (#463, #491 ; #536 en partie), dans la forme des entrées de la section.

**T5 — gates réellement exécutés, sur l'arbre de travail final (dernier état du code)**
- Backend complet `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest), base `kesh_155e1` remise à zéro juste avant : **2793 exécutés, 2793 passés, 4 ignorés** (110,8 s). Écart avec la 15-5b (2784) : +9 = 6 + 3, recompté (`grep -c '#\[sqlx::test'` sur le fichier neuf ; `#[test]` du registre 6 → 9).
- Frontend : `npm run check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership` PASS, `test:unit` **979 / 979** (107 fichiers), `build` OK. Rien n'y change.
- **E2E complet** (Playwright, base `kesh_e2e_155e1` neuve, backend sur 3002) : **240 passés, 7 échoués, 19 ignorés** (9,7 min). Les 7 sont exactement les sept KF-029 (#97) de `docs/testing.md` § « Les échecs attendus » (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) ; ni huitième variable, ni KF-045 (run de l'après-midi), ni KF-046.
- Manuels : non touchés (AC6 : manuels et Pattern 5 à la 15-5e2). `docs/MULTI-TENANT-SCOPING-PATTERNS.md:338-351` appelle encore `retry_with` sans nom d'opération : l'un des quatre documents faux entre ce merge et celui de la 15-5e2, nommés à l'AC6.

**Propagation post-patch** : `Toute divergence de cet ordre`, `Aucun chemin ne verrouille`, `deadlocks cross-table`, `la plus jeune`, `aucun cycle n'est connu`, `Acceptable vs. un 500` grepés sur `crates/` et `docs/` : restent la phrase neuve du module `retry.rs` (« non la plus jeune ») ; `retry_with(` sans nom dans `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (attendu, AC6).

**Points à vérifier en revue** : la 15-5e2 restreint `retry_with` à `post_accept` dans le volet (c) ; les numéros de ligne de la fiche (`validate_invoice` `:1916-1929` etc.) sont décalés par les doc-comments réécrits.

### File List

- `CHANGELOG.md`
- `Cargo.lock`
- `crates/kesh-api/Cargo.toml`
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/retry.rs` (neuf)
- `crates/kesh-api/src/routes/company_invoice_settings.rs`
- `crates/kesh-api/src/routes/invoices.rs`
- `crates/kesh-api/src/routes/onboarding.rs`
- `crates/kesh-api/src/routes/opening_balances.rs`
- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/src/routes/supplier_invoices.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-api/tests/common/capture_rejeu.rs` (neuf)
- `crates/kesh-api/tests/common/mod.rs`
- `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (neuf)
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/src/repositories/invoices.rs`
- `crates/kesh-db/src/repositories/supplier_invoices.rs`
- `crates/kesh-db/src/retry.rs`
- `_bmad-output/implementation-artifacts/15-5e1-socle-rejeu.md`
- `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

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
- 2026-10-08 — **Validation P4** (Opus ×2, contexte frais, lecture seule ; lentilles R « regression
  hunter » et F « full-scope adversary » ; prompt `15-5e1-validate-prompt-p4.md` ; rapports
  `target/gate-logs/15-5e1-p4-R.md` et `-F.md`). **Findings** : 0 CRITICAL, 0 HIGH, **3 MEDIUM**
  (R4-1, F4-1, F4-2), **19 LOW** (R4-2 à R4-11 — le bilan du rapport R annonce 9 LOW, il en détaille
  10 —, F4-3 à F4-11). Les passes ont lu des numéros de ligne antérieurs au merge des 15-5a/15-5b
  (`f289414e`) ; **tous les numéros de la fiche sont recalés sur `f289414e`** (`grep -nF` / `sed -n`).
  **Remédiations** :
  - **F4-1 MEDIUM** (choix **C66**, décision de l'orchestrateur) : la **saisie fournisseur**
    (`POST /supplier-invoices`) est rejouée **ici**, enveloppe `DbError`, avec un **test 5** « route
    victime » sur une vraie 1213 — l'avance des réglages et sa défense arrivent ensemble. L'hypothèse
    R3-7 est écrite **non tranchée**, rendue sans conséquence par le rejeu. Partition du registre
    **8 / 13 / 4 / 90 = 115** (au lieu de 7 / 14 / 4 / 90) ; AC1, AC3, AC4, AC6, table des routes,
    Story, dépendances, T0, T1 alignés. **C55** (« sérialise ») et **C65** révisés par C66, sans
    réécrire les entrées.
  - **F4-2 MEDIUM** (choix **C67**) : section **« Dérogation règle de splitting »** — 10 modules de
    production (2 crates), justification (story-zéro ; ne pas séparer le patron de ses premiers sites
    ni l'avance des réglages de sa défense), risque accepté. ⚠️ **Signal D5 déclaré au Project Lead** :
    seuil de dispersion franchi, story non redécoupée, arbitrage consigné en C67.
  - **R4-1 MEDIUM** : le volet (c) dit quelle preuve exerce quelle précaution — sur le source de
    cette story, aucune route `Rejouee` n'est suivie d'un item qui nomme une enveloppe (le cas naît à
    la 15-5e2) ; seule la mutation « nom en commentaire » exerce une précaution (la 1) ; les autres
    ne sont prouvées que par le source synthétique. La parenthèse fausse de la mutation de
    `validate_invoice_handler` est retirée.
  - **LOW** : R4-2 = F4-3 (test 3 : l'attente est à l'étape (2-bis) de `cancel_settlement_in_tx`,
    exercice de l'écriture de règlement) ; R4-3 (une lecture hors transaction déjà dans la tentative
    peut y rester) ; R4-4 (exemple `ignore` de `retry.rs:95-104` ; troisième prémisse dans la
    constante) ; R4-5 = F4-7 (recalage ; préfixe `routes/` de `imported_supplier_invoices.rs:243`) ;
    R4-6 (C55, C57 : révisions écrites en C66, C69) ; R4-7 (déstructurations du registre à l'AC7) ;
    R4-8 (`insert_with_defaults_in_tx`, `:699-721`) ; R4-9 = F4-4 (volet (c) : fenêtre close à
    l'accolade en colonne 0, retrait des commentaires qui saute les littéraux, limite de mot, handler
    introuvable → échec, source synthétique étendu) ; R4-10 (mentions de `retry_with`
    d'`opening_complement.rs:36`, `:405` : **attribuées à la 15-5e2**, C69) ; R4-11 (montage sorti de
    l'item 1) ; F4-5 (test 1 exerce `retry_app_on_deadlock` ; **non fermé par C66**, la saisie
    fournisseur prenant l'enveloppe `DbError`) ; F4-6 (doc-comment canonique : `INSERT IGNORE` de
    (2), condition du compteur de (4)-(5), compteur des écritures de (7)) ; F4-8 (trois documents
    faux entre les deux merges, écrits à l'AC6) ; F4-9 (TTC et non HT ; exercice du jour sans date
    codée en dur — ⚠️ **l'exemple de la lentille est faux** : la TVA s'arrondit au centime par ligne,
    100.05 à 8,1 % donne 108.15, multiple de 5 centimes ; remplacé par 100.10 → 108.21) ; F4-10
    (point (v) : une route `GET` échappe au registre) ; F4-11 (ligne « Modifié » du CHANGELOG pour le
    `warn!`). Aucun LOW écarté.
  **Propagation post-patch** : `sérialise`, `7 / 14`, `14 à rejouer`, `3 rejouées`, `tests 2 à 4`,
  `tests 1 à 4`, `test 5`, `trois routes`, et les anciens numéros (`850`, `3843`, `3858`, `692`,
  `2411`, `2379`, `:591`, `:576`, `:338`, `1487`, `1520`) grepés sur cette fiche, la 15-5e2, la 15-5d,
  l'index et la fiche mère (hors Change Log) : les occurrences restantes citent le texte à corriger
  (« jamais sérialise »), désignent les tests de la 15-5e1 par leur nouveau numéro, ou sont des
  numéros d'autres fichiers. Fiches touchées hors celle-ci : 15-5e2, 15-5d (C66, C68), index, mère,
  registre (C66 à C69), `sprint-status.yaml`. **Décompte** (recompté sur la fiche) : **7 AC, 6 tâches
  T0–T5** ; tests 1 à 5 au fichier neuf, test 6 au registre ; table des routes 4 + 4 + 13 + 4 = 25,
  registre 115 − 25 = 90 `SansEcritureAuJournal`, partition 8 / 13 / 4 / 90.

- 2026-10-08 — **Validation P5** (Sonnet ×2, contexte frais, lecture seule ; lentilles R et F ;
  prompt `15-5e1-validate-prompt-p5.md` ; rapports `target/gate-logs/15-5e1-p5-R.md` et `-F.md`).
  **Findings** : 0 CRITICAL, 0 HIGH, **1 MEDIUM** distinct (R5-1 = F5-1), **11 LOW** (R5-2 à R5-5,
  F5-2 à F5-8). **Remédiations** (choix **C70**, décisions de l'orchestrateur) :
  - **R5-1 = F5-1 MEDIUM — recyclage** : le volet (c) du registre, scanner textuel, avait déjà eu un
    trou en P4 (accolades des littéraux) ; la P5 en trouve un second (durées de vie `'_`,
    `&'static str`, littéral octet `b';'`, présents dans les fichiers visés). **Changement de
    méthode** : le volet (c) parse chaque fichier de routes avec **`syn` 2** (déjà au `Cargo.lock`,
    2.0.118 ; ajouté aux `[dev-dependencies]` de `kesh-api`, features `full` et `visit`), retrouve
    le `fn` du handler par son nom et cherche dans son corps un `ExprCall` (dernier segment du
    chemin) ou un `ExprMethodCall` nommant une enveloppe. Les quatre précautions textuelles et le
    source synthétique qui les éprouvait sont retirés ; restent l'échec explicite sur handler
    introuvable, un test du visiteur sur source synthétique, et les mutations. Limite écrite : un
    appel dans une macro rend un faux rouge. ⚠️ **Signal D5 déclaré au Project Lead** : la P5 rend
    un MEDIUM de même sévérité que la P4 **et** c'est un recyclage (deuxième trou du même scanner) —
    le cas même où l'amendement D5 découpe. Constat : le recyclage tient à la **méthode** du volet
    (c), pas au périmètre de la story ; il est fermé par un changement de méthode, sans découpage ;
    la story passe de dix à onze modules (F5-6 ci-dessous), arbitrage écrit à la section
    « Dérogation » et en C70.
  - **F5-6 LOW** (décision de l'orchestrateur) : `PUT /api/v1/company/invoice-settings`
    (`update_invoice_settings`) est **rejouée ici** (enveloppe `DbError` autour de
    `company_invoice_settings::update`), avec un **test 7** « route victime » (S tenu par le test, X
    demandé par la route, X demandé par le test : cycle documenté d'InnoDB, indépendant de R3-7). Au
    registre elle reste `SansEcritureAuJournal`, comme `onboarding::finalize` ; tenue par le test 7
    et le point (vi) du doc-comment — pas de cinquième valeur, l'option la plus simple. **R3-7** est
    désormais une **hypothèse forte** (verrou partagé de l'`INSERT IGNORE` en doublon), **mesurée en
    T0**.
  - **LOW** : R5-2 (préalables du test 5 : contact fournisseur, compte créanciers ; montage commun
    partagé entre tests 2-4 et 2-5) ; R5-3 (ligne de la 15-5d recalée, `:301-372`, `:374-377`) ;
    R5-4 (`:305-372` → `:301-372`) ; R5-5 et F5-8 (dérogation : 5 + 1 + 1 + 4 = 11 ; les sites
    forcés « suivent du choix de signature », non plus « ne se découpent pas ») ; F5-2 (contenu des
    littéraux : **absorbé** par `syn`, un littéral n'est pas un appel) ; F5-3 (projections `:290`,
    `:338` à l'AC7) ; F5-4 (trois copies de `seed_validated_invoice`, non deux ; harnais recopié,
    copie assumée comme dans les autres fichiers E2E — dette signalée pour une issue) ; F5-5 (le
    test 4 monte désormais le cycle de #536, compte d'arrondi contre exercice ; la justification
    « le motif d'attente change » retirée) ; F5-7 (quatrième document faux entre les deux merges,
    `admin-manual.tex:885-886` ; CHANGELOG « #536 en partie »). Aucun LOW écarté.
  **Propagation post-patch** : `textuel`, `par texte`, `précaution`, `colonne 0`, `non tranch`,
  `ni mesur`, `non mesur`, `tests 1 à 5`, `dix modules`, `10 modules`, `Trois documents`,
  `troisième copie`, `305-372`, `300-350`, `352-355`, `(#463, #491, #536)` grepés sur cette fiche, la
  15-5e2, la 15-5d, l'index, la fiche mère et le registre (hors Change Log et entrées antérieures du
  registre) : restent les mentions historiques du scanner textuel (Change Log, C63, C69) et la
  phrase qui dit pourquoi il a été retiré. Fiches touchées hors celle-ci : 15-5e2 (texte du
  CHANGELOG, mutation 2), 15-5d (`:301-372`), index et mère (C70), registre (C70),
  `sprint-status.yaml`. **Décompte** (recompté sur la fiche) : **7 AC, 6 tâches T0–T5** ; **7
  tests** (1 à 5 et 7 au fichier neuf, 6 au registre) ; table des routes 4 + 4 + 13 + 4 = 25,
  partition **8 / 13 / 4 / 90 = 115** (inchangée : la route des réglages est
  `SansEcritureAuJournal`) ; **11 modules de production**.
  **Prochaine passe** : ciblée (une lentille) sur la remédiation P5 — le volet (c) réécrit, l'AC3
  et le test 7, le test 4, la dérogation.
  ⚠️ **Correction (P6, finding F6-1, choix C74)** : « cycle documenté d'InnoDB, indépendant de
  R3-7 » ci-dessus est **faux hors de la version épinglée** — le cycle se forme sur MariaDB 10.11,
  pas nécessairement sur MariaDB ≥ 11.4.5 ni sur MySQL ≥ 8.0.18 (MDEV-34877), où il dépend de
  R3-7. L'entrée est laissée telle quelle ; le corps de la fiche est corrigé.

- 2026-10-08 — **Validation P6** (Opus ×2, contexte frais, lecture seule ; lentilles R « regression
  hunter », braquée sur la remédiation P5 `53c47bb7`, et F « full-scope adversary » ; prompt
  `15-5e1-validate-prompt-p6.md` ; rapports `target/gate-logs/15-5e1-p6-R.md` et `-F.md`).
  **Findings** : 0 CRITICAL, 0 HIGH, **1 MEDIUM** (F6-1), **11 LOW** (R6-1 à R6-7, F6-2 à F6-5 ;
  R6-1 = F6-2, soit **10 LOW distincts**). **Remédiations** (choix **C74**, décisions de
  l'orchestrateur) :
  - **F6-1 MEDIUM** : le cycle du test 7 n'est pas « indépendant de R3-7 » — MySQL 8.0.18 et
    MariaDB 11.4.5 / 11.7.2 (MDEV-34877) accordent le `X` au détenteur du `S` sans attendre ; sur
    ces versions, si R3-7 est fausse, le test passe au vert sans aucun rejeu, ses trois assertions
    comprises. **Témoin du rejeu** aux tests 2 à 5 et 7 (montage commun) : un abonné `tracing` de
    capture installé pour la durée du test (couche maison sur `tracing-subscriber`, déjà
    dépendance de `kesh-api` — aucune crate ajoutée —, module partagé
    `tests/common/capture_rejeu.rs` ; `set_default` suffit, `#[sqlx::test]` tournant sur un
    runtime à un seul fil) exige au moins un `warn!` de `kesh_db::retry` portant l'`operation`
    attendue ; **mutation** du témoin aux tests 2 et 7 (cycle retiré du montage → la requête
    aboutit sans rejeu → seul le témoin rougit). Test 7 réécrit : cycle **sur MariaDB 10.11, la
    version épinglée**, versions plus récentes nommées. **T0** forme le cycle du test 7 à la main
    avant de l'écrire. « Indépendant de R3-7 » retiré du corps ; corrigé au Change Log P5 par une
    ligne, et dans le registre par C74 (C70 n'est pas réécrite).
    ⚠️ **Signal D5 déclaré au Project Lead** : sévérité **égale** à la P5 (MEDIUM → MEDIUM) **et**
    recyclage — F6-1 naît de la remédiation P5 (le test 7 y a été ajouté), le cas même où
    l'amendement D5 découpe. Constat : le défaut porte sur la **force probante** d'un test neuf,
    pas sur le périmètre de la story ; il est traité par un **renforcement des preuves** (témoin,
    mutation, mesure en T0), sans découpage ni module de production de plus.
  - **LOW** : R6-1 = F6-2 (AC3 : les lectures préalables lisent la ligne même que `update`
    verrouille ; déjà hors transaction, elles n'en sortent pas — exception de l'AC2 —, et le
    verrou optimiste `WHERE version = ?` les rejuge à chaque tentative) ; R6-2 (point (ii) du
    doc-comment du registre : « tests 2 à 5 et 7, cinq routes `DbError` » ; propagé à la 15-5e2,
    AC2 et « Pourquoi la revue fichier par fichier suffit », et à sa phrase « Comme pour les
    quatre routes de la 15-5e1 ») ; R6-3 (montage : la société vient de `seed_accounting_company`
    — réglages, taux, exercice —, pas de `create_company` ; `designate_rounding_account` de
    `kesh_db`, `UPDATE` simple, vérifié après appel, et non l'homonyme local qui upsert ; test 7 :
    ligne des réglages semée, `version` lue par `GET` avant la transaction de test) ; R6-4
    (`routes/onboarding.rs`, trois sites ici, un à la 15-5e2) ; R6-5 (« cinq routes rejouées
    ici » à l'AC2 et à T0, effets de bord de `update_invoice_settings`) ; R6-6 (source
    synthétique : appels imbriqués, fermeture `async`, bloc ; surcharges qui rappellent
    l'implémentation par défaut) ; R6-7 (titre de la story, de l'AC3, de T1) ; F6-3 (« Pourquoi
    rejouer est sûr » : le règlement partiel concurrent, non gardé par une `version`, constat sans
    garde neuve) ; F6-4 (CHANGELOG dans la forme de la section — titre gras, issues en liens —,
    ici et à la 15-5e2) ; F6-5 (tests 2 et 3 : `disable_rounding_to_5_centimes`, le TTC multiple
    de 5 centimes n'est plus exigé). Aucun LOW écarté.
  **Propagation post-patch** : `indépendant de R3-7`, `se vérifie ou non`, `qu'il y ait ou non`,
  `quatre routes`, `tests 2 à 5`, `onboarding.rs:6`, `create_company`, `(#463, #491`,
  `ils ne lisent rien` grepés sur cette fiche, la 15-5e2, la 15-5d, l'index et la fiche mère :
  restent la mention historique du Change Log P5 (corrigée par la ligne ci-dessus), « Aux tests 2
  à 5 » pour l'exercice (le test 7 n'en prend pas), « quatre routes des issues et de la saisie »
  (T1, qui nomme la cinquième à part), « pas de `create_company` », et la liste de propagation
  de la P5 de la 15-5e2. Le registre (C70) n'est pas réécrit : C74 le corrige. **Décompte**
  (recompté sur la fiche) : **7 AC, 6 tâches T0–T5** ; **7 tests** (1 à 5 et 7 au fichier neuf, 6
  au registre) ; partition **8 / 13 / 4 / 90 = 115** inchangée ; **11 modules de production**
  inchangés (le module de capture est un fichier de test).
  **Prochaine passe** : P7, ciblée (Haiku, une lentille) sur la remédiation P6.
- **2026-10-08 — Validation P7 ciblée (Haiku, une lentille, prompt `15-5e1-validate-prompt-p7-ciblee.md`) :
  0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.** Rapport : `target/gate-logs/15-5e1-p7-ciblee.md`. Le « 0 » a été
  vérifié par l'orchestrateur sur l'axe que la lentille n'a qu'effleuré (axe 1) : le `warn!` de rejeu vit dans
  `crates/kesh-db/src/retry.rs` (aujourd'hui `debug!` à `:160`, passé en `warn!` avec `operation` par cette
  story — C62), sa cible par défaut est `kesh_db::retry`, et la création d'un `Dispatch` reconstruit le cache
  d'intérêt des callsites ; la couche de capture, sans filtre, reçoit donc l'événement. **Validation CLOSE.**
  Trend : P1 1 HIGH / 6 MEDIUM (15-5e entière) → P2 6 MEDIUM → P3 2 MEDIUM → découpage → P4 3 MEDIUM → P5
  1 MEDIUM (recyclage, méthode remplacée par `syn`) → P6 1 MEDIUM (recyclage, témoin du rejeu) → P7 0.
  Modèles : Sonnet, Opus, Sonnet, Opus, Sonnet, Opus, Haiku (ciblée). Signaux D5 des P5 et P6 déclarés au
  Project Lead.
- **2026-10-08 — Développement** (`bmad-dev-story`, Opus 5.5, autonomie complète) : T0 à T5 faits. R3-7 mesurée **vraie** sur MariaDB 10.11.16 (issue à ouvrir par l'orchestrateur), cycle du test 7 formé à la main ; patron (`kesh_db::retry` nommé, `kesh_api::retry`), cinq routes rejouées, avance des réglages de la saisie fournisseur, doc-comments canoniques, « 5 bis », registre à deux colonnes (volets (c) par `syn`, (d)), tests 1 à 5 et 7 avec témoin, dix mutations rouges, CHANGELOG. Gates : backend 2793/2793 (4 ignorés), frontend 979/979, E2E 240 passés / 7 KF-029 attendus. Choix **C-15-5e1-1** à **3**. Statut → `review`.
