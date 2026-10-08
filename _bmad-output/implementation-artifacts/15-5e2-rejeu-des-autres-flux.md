# Story 15.5e2 : Rejeu sur interblocage des autres flux d'écriture — rollout, commentaires d'ordre, manuels

Status: ready-for-dev

<!-- Sous-story de la 15-5e (fiche index `15-5e-ordre-des-verrous-reglements.md`, statut `split`),
     créée le 2026-10-08 par le découpage décidé à la validation P3 de la 15-5e (finding F3-2, choix
     C61). C'est le ROLLOUT du patron que pose la 15-5e1 (`15-5e1-socle-rejeu.md`) : application
     mécanique des deux enveloppes aux autres routes qui écrivent au journal, revue fichier par
     fichier ; plus les commentaires d'ordre, le Pattern 5, les manuels (#484), le CHANGELOG et
     `docs/api-external.md`. Le contenu vient de la 15-5e validée P1 à P3 (commit `94f1365e` pour la
     version d'avant découpage) ; les remédiations de la P3 sont appliquées ici et dans la 15-5e1.
     Choix applicables : C54, C55, C56 (révisé par C61, C63), C57, C58, C59 (révisé par C62), C60,
     C61, C62, C63, C64.
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation. Le
     développement attend la clôture de la boucle de validation ET le merge de la 15-5e1. -->

**Issues** : la PR porte **`closes #536`** et **`closes #484`** (mots-clés **dans la PR** : le dépôt
merge en squash), et **`refs #463`**, **`refs #491`** (fermées par la 15-5e1).
- **#536** — interblocage entre l'acceptation d'un lot de rapprochement et la validation d'une
  facture : la validation est rejouée par la 15-5e1 ; la seconde inversion que l'issue cite —
  l'**avoir** d'une facture arrondie, exercice puis arrondi — est couverte ici, la création d'avoir
  étant rejouée (AC1). Le rejeu couvre alors l'issue entière (choix C57).
- **#484** — les manuels affirment que Kesh protège la numérotation et les « opérations critiques »
  par des transactions `SERIALIZABLE` ; il n'y en a aucune (`grep -rn SERIALIZABLE crates/*/src` :
  vide ; isolation `REPEATABLE READ` par défaut, `crates/kesh-db/src/pool.rs:17`). Les deux passages
  sont réécrits sur le mécanisme réel (AC5, choix C60).

**Dépend de la 15-5e1** (le patron : `kesh_db::retry::retry_on_deadlock`, `kesh_api::retry::retry_app_on_deadlock`,
`is_app_deadlock`, le paramètre `operation`, le registre et son statut transitoire `ARejouer` ; le
doc-comment canonique de `validate_invoice`, « 5 bis », le module `retry.rs` réécrits), donc des 15-5a
et 15-5b. **Ordre libre par rapport à la 15-5d** (choix C65) : la 15-5d ne dépend que de la 15-5e1 ;
si elle merge avant cette story, la saisie fournisseur et la complétion d'import portent ses verrous
sans rejeu jusqu'à ce merge — T0 relève alors le `create_in_tx` fournisseur tel que la 15-5d l'aura
laissé.

## Story

En tant que **comptable d'une société tenue dans Kesh**,
je veux que **toute opération qui écrit au journal, victime d'un interblocage avec une autre opération
légitime, soit rejouée par le serveur sans que je le voie**,
afin de **ne plus recevoir d'erreur 500 quand deux opérations se croisent** — et que la documentation
dise le mécanisme réel, sans transaction `SERIALIZABLE` imaginaire ni ordre de verrous « sans cycle ».

La règle et sa raison (« l'ordre des verrous réduit la fréquence des interblocages ; le rejeu les rend
invisibles », choix C54) sont écrites dans la 15-5e1, § *Pourquoi le rejeu* ; elles ne sont pas
recopiées ici.

## Acceptance Criteria

### Le rollout

1. **AC1 — Les 18 autres routes `Rejouee` de l'inventaire** (inventaire de la 15-5e1, colonne
   « fiche » = 15-5e2 ; 21 − 3 = 18) :
   - **14 routes à rejouer**, chacune par l'enveloppe que la table de la 15-5e1 lui assigne, avec un
     nom d'opération `"<module>::<action>"` : dévalidation d'une facture ; création d'un avoir ;
     saisie d'une facture fournisseur, règlement fournisseur, annulation d'une facture fournisseur et
     de son règlement ; confirmation d'un lot de paiement ; écriture manuelle (création, suppression,
     contre-passation) ; bilan d'ouverture (`DbError`, `map_opening_balances_error` **après**) ;
     complétion d'une facture importée, rapprochement manuel et ventilé (`AppError`, partie
     transactionnelle extraite en fonction « une tentative », patron `accept_once`,
     `reconciliation.rs:879`, et `cancel_reconciliation_once`, `:3858`, qui ouvre la transaction,
     prend le verrou nommé du compte bancaire le cas échéant, écrit et conclut).
   - **3 sites existants migrés** vers les enveloppes (DRY) : `write_off_invoice_handler`
     (`invoices.rs:1343`) et `complete_opening_balances` (`opening_balances.rs:591`) par l'enveloppe
     `DbError` ; `post_cancel_reconciliation` (`reconciliation.rs:3843`) par l'enveloppe `AppError` ;
     leurs noms d'opération (posés par la 15-5e1) sont gardés.
   - **`post_accept` inchangée** (`reconciliation.rs:850`) : son prédicat élargi au 1305 est propre à
     la route et le reste ; elle est le seul `retry_with` direct restant parmi les routes `Rejouee`.
     `onboarding::finalize` (`onboarding.rs:614`) n'écrit pas au journal et garde son `retry_with`.
   - **Les règles de la 15-5e1 s'appliquent route par route**, vérifiées en T0 et en revue fichier par
     fichier : une tentative = une transaction neuve ; les contrôles qui ne dépendent pas de la
     transaction restent hors de la fermeture, **ceux qui lisent ce que la transaction verrouille
     restent dans la tentative, dans le même ordre** (finding R3-4 : dans `complete_import`
     (`imported_supplier_invoices.rs:123-290`), les contrôles de devise, QR-IBAN et montant lisent le
     `staging` pris `FOR UPDATE` et restent dans `complete_import_once` après lui ; dans
     `cancel_reconciliation_once`, la lecture de `bank_account_id` (`reconciliation.rs:3858-3870`)
     reste dans la tentative) ; les entrées consommées sont **clonées par tentative** (finding F3-8 :
     `create_supplier_invoice` construit un `NewSupplierInvoice` en déplaçant les lignes de `req`,
     `supplier_invoices.rs:~330-352` ; `confirm`, `create_credit_note`, `create_journal_entry`
     consomment `Json(req)` — patron `onboarding::finalize`, pas de `Option::take`) ; aucun effet de
     bord hors transaction ; le 1213 atteint le prédicat (aucun `map_err` hors `map_db_error`).
   - **Le registre** (`audit_route_registry.rs`) : les 14 statuts `ARejouer` deviennent `Rejouee` et
     la variante **`ARejouer` est retirée** de l'énumération ; la partition (volet (d)) devient **21**
     `Rejouee`, **4** `Exemptee`, **90** `SansEcritureAuJournal`, total **115** ; le volet (c)
     **n'accepte plus `retry_with` que pour `post_accept`** (liste nommée dans le test) — un
     `retry_with` recopié sur une autre route le fait rougir.
   - Chacun des doc-comments des 14 routes et des 3 sites migrés dit, en une ligne, que la route est
     rejouée sur interblocage, avec renvoi à l'enveloppe (pas de recopie de la règle). Le doc-comment
     de `write_off_invoice_handler` (`crates/kesh-api/src/routes/invoices.rs:1324-1327` ; la phrase
     sur l'ordre est `:1324-1326`, celle sur la `version` `:1326-1327` — finding R3-1 ; F1-3) :
     « l'ordre des verrous est celui du règlement manuel (facture → compte → exercice), qui peut
     former un cycle avec l'acceptation d'un rapprochement (#491) » est réécrit : la route est rejouée
     comme toute route d'écriture au journal ; la phrase sur la `version` (« une tentative qui trouve
     un reste changé est refusée en 409 ») est gardée.

### Ce qui doit être prouvé

2. **AC2 — Tests.** Pas de test dynamique neuf (choix C56 : les tests 2 à 4 de la 15-5e1 prouvent le
   patron sur trois routes ; pour les autres, c'est la revue fichier par fichier et le registre — le
   doc-comment du registre le dit, finding F2-10).
   - Les tests existants des routes touchées restent verts **sans modification de leurs assertions**,
     ainsi que `accept_replays_the_batch_when_it_is_the_deadlock_victim` et les tests 1 à 4 de la
     15-5e1.
   - **Mutations** (consignées au Dev Agent Record, en touchant le fichier après restauration — sans
     quoi cargo garde le binaire muté), **une par famille** (finding F3-1) :
     - famille `DbError` à correspondance après le rejeu : retirer l'enveloppe de
       `create_journal_entry` → le volet (c) rougit ;
     - famille `DbError` dont le doc-comment de la route suivante nomme l'enveloppe : retirer
       l'enveloppe de `pay_supplier_invoice` (suivi du doc-comment de `cancel_supplier_invoice`,
       `supplier_invoices.rs:~392-397`) → le volet (c) rougit ;
     - famille `AppError` : retirer l'enveloppe de `post_manual` → le volet (c) rougit ;
     - remplacer l'enveloppe de `write_off_invoice_handler` par un `retry_with` recopié → le volet (c)
       rougit (acceptation de `retry_with` restreinte à `post_accept`).
   - **Aucun test de « non-interblocage »** (finding F1-4 de la P1 de la 15-5e).

### L'ordre des verrous : les commentaires et le Pattern 5 (choix C54, C55)

3. **AC3 — Les commentaires disent la règle vraie, et aucun ne prétend à l'absence de cycle.** Le
   doc-comment canonique de `validate_invoice`, « 5 bis » et le module `retry.rs` sont réécrits par la
   15-5e1 ; le reste est ici.
   - **Les commentaires qui fondent une absence de cycle sur « l'ordre global »** (finding F2-1) :
     `crates/kesh-db/src/repositories/journal_entries.rs:236-240` et `:253-260` (`create_in_tx`,
     étapes 0 et 0-bis : « respecter l'ordre de verrouillage global companies → projects →
     fiscal_years […] créerait une inversion ABBA ») et `:547-548` (`create_opening_entry` :
     « respecte l'ordre de verrou global — pas d'inversion ABBA ») sont **réécrits** : prendre la
     sentinelle et les projets avant l'exercice **réduit la fréquence** des interblocages, il ne les
     exclut pas — deux flux prennent l'ordre **inverse**, exercice **puis** sentinelle et projet : le
     lot de rapprochement (`accept_one_rule`, `crates/kesh-api/src/routes/reconciliation.rs:2207` :
     exercice `:2379`, puis `validate_taggable_in_tx` `:2411` → sentinelle
     `crates/kesh-db/src/repositories/projects.rs:99`) et le rapprochement manuel à projet
     (`post_manual` : exercice `routes/reconciliation.rs:3124`, puis `validate_taggable_in_tx`
     `:3136`) ; les deux côtés de chaque cycle sont rejoués (renvoi à l'enveloppe). L'inversion est
     écrite au commentaire de chacun (`routes/reconciliation.rs:2408-2411` ; et `:3128-3134`, qui
     affirme aujourd'hui l'inverse — « cohérent Pattern 5 (avant le lock fiscal_years pris par
     create_in_tx) », alors que l'exercice est déjà verrouillé à l'étape 6 —, réécrit).
   - **`crates/kesh-db/src/repositories/fiscal_years.rs:497-502`** (finding F3-4) : « Toute
     divergence = risque de deadlock avec `journal_entries::create_in_tx` en cours sur la même
     company », jumelle de la phrase que la 15-5e1 retire de `invoices.rs:1928` — **réécrite** : la
     place de l'exercice est une convention de fréquence, la défense est le rejeu.
   - **Gardés, vrais pour la paire qu'ils nomment** : `repositories/projects.rs:77-80`,
     `repositories/supplier_invoices.rs:278-280` et `repositories/reconciliation_rules.rs:184-185`
     (« anti-ABBA » avec le chemin d'archivage d'un projet : sentinelle puis projet, des deux côtés),
     avec la seule précision qu'ils ne valent que pour cette paire si le texte laisse entendre plus ;
     `repositories/journal_entry_number_sequences.rs:130` (« pas de cycle entre créations », le
     compteur pris en premier par toute création), vrai entre créations.
   - **Inventaire au symptôme des affirmations d'ordre ou d'absence de cycle** (règle *Inventorier les
     sites NON RÉSOLUS* ; findings F2-1, R2-2, **F3-4** — périmètre étendu aux tests) :
     `grep -rnE "ABBA|inversion|deadlock|interblocage|cycle|ordre global|global lock order|[Ll]ock [Oo]rder|ordre des (locks|verrous)|follow the documented order|[Dd]eny list" crates/*/src crates/*/tests docs/*.md --include=*.rs --include=*.md`
     — relevé sur `94f1365e` : **181 lignes dans 42 fichiers** — 148 lignes / 32 fichiers sous
     `crates/*/src` et `docs/*.md` (dont 48 dans `retry.rs`, en grande partie ses tests, 18 dans
     `repositories/invoices.rs`, 15 dans `MULTI-TENANT-SCOPING-PATTERNS.md`, 10 dans
     `routes/reconciliation.rs`) et **33 lignes / 10 fichiers sous `crates/*/tests`** (10 dans
     `reconciliation_e2e.rs`, 8 dans `opening_complement_repository.rs`, 4 dans
     `invoice_unvalidate_e2e.rs`, 3 dans `supplier_invoices_repository.rs`, 3 dans
     `fiscal_years_repository.rs`, une dans chacun de cinq autres). **Chaque occurrence est triée au
     Dev Agent Record**, une par une : *vraie* (et pourquoi), *réécrite* (ici, ou par la 15-5e1 —
     doc-comment canonique, « 5 bis », `retry.rs`), ou *hors sujet* (autre mécanisme). Pour les tests,
     le tri est assumé ainsi : une phrase qui **asserte** une absence d'interblocage
     (`crates/kesh-db/tests/opening_complement_repository.rs:727` « aucun interblocage », `:781`
     « l'écriture tenant l'exercice passe : aucun interblocage » ; `fiscal_years_repository.rs:1071`
     « Le FOR UPDATE sérialise → pas de deadlock ») est **vraie si et seulement si elle décrit le
     montage du test** (deux transactions qui ne peuvent pas former de cycle par construction) — elle
     est alors précisée en ce sens, sans toucher aux assertions ; sinon réécrite.
     `crates/kesh-api/tests/reconciliation_e2e.rs:4414` (« `retry_with` n'existant que sur
     l'annulation ») est **vraie, historique** : la phrase est introduite par « Avant la 25-4-c2 » —
     gardée. Une occurrence qui affirme encore une absence de cycle sans la limiter à la paire ou au
     montage qu'elle nomme **bloque la story**.
   - **`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, Pattern 5** (`:285-391`) : « Why Lock Ordering
     Matters » (`:289-291`, même affirmation fausse sur la détection) est réécrit ; « Global Lock
     Order » (`:293-305`) devient une **convention de fréquence**, qui nomme la sentinelle `companies`
     et les verrous **partagés** que posent à l'insertion les clés étrangères des écritures —
     `fk_journal_entries_company` (`20260412000001_journal_entries.sql:27`), `fk_jel_account` (`:45`),
     `fk_jel_project` (`20260702000001_projects_analytics.sql:40-41`) —, l'inversion
     d'`accept_one_rule` et de `post_manual` (exercice, puis sentinelle et projet) et la règle « toute
     route qui écrit au journal est rejouée » (renvoi aux enveloppes et au registre) ; la table
     « Where This Applies » (`:307-318`) corrige ses deux lignes de flux comptables —
     `POST /supplier-invoices` : companies (sentinelle, si projet) → projets → réglages → comptes de
     charge → exercice → lignes (verrous partagés sur la société, les comptes, le projet) — la
     sentinelle de la ligne actuelle est **gardée** (finding F2-9) ; `invoices::validate_invoice` : la
     numérotation de l'étape canonique de la 15-5e1 (finding F1-2) — et gagne une ligne pour le lot de
     rapprochement (`accept_one_rule`). « Known Risk » (`:320-324`, finding R2-2) : la « Mitigation
     (v0.1) » de `:324` — « all current write endpoints follow the documented order. New endpoints
     MUST follow it or be added to the deny list » — est réécrite (l'ordre est une convention de
     fréquence, la défense est le rejeu, le registre garde les routes). « Resolution status »
     (`:326-330`) : la puce du helper de rejeu (`:328`) nomme les deux enveloppes au lieu de
     « `finalize` via `retry_with` » ; la puce **« CI lint (grep-detect `FOR UPDATE` + verify global
     order) »** (`:329`, ⏳) est **retirée** (finding R3-6 (e)) — sans ordre global garanti, il n'y a
     plus d'ordre à vérifier, et le registre testé de la 15-5e1 tient le rôle de garde ; la puce
     « Deny list of divergent endpoints » (`:330`) est retirée. L'exemple de code (`:338-353`) nomme
     les deux enveloppes au lieu d'un `retry_with` recopié. La rubrique « Deny list » (`:332-336`) est
     retirée. « When to Use » et « Code Reference » (`:355-391`, « WRONG: reverse order will
     deadlock ») disent « réduit la fréquence », pas « évite ».

### Documentation

4. **AC4 — `docs/api-external.md` § 10 « Gestion des erreurs »** (`:397`) gagne une phrase générale :
   toute route qui écrit au journal — hors `POST /admin/full-import` et `POST /onboarding/reset`, qui
   ne rejouent pas (finding F2-6) — rejoue un interblocage transitoire sans le montrer ; s'il persiste
   après trois tentatives, la requête finit en `500 INTERNAL_ERROR`, rien n'a été écrit, et elle peut
   être renvoyée telle quelle. Les deux phrases propres à l'acceptation (`:307`) et à l'annulation
   d'un rapprochement (`:313`) restent vraies et ne sont pas touchées.
5. **AC5 — CHANGELOG et manuels.**
   - **CHANGELOG**, rubrique **Corrigé** de la section `## [0.13.0] — Non publié` : la ligne posée par
     la 15-5e1 (choix C64) est **étendue**, sans promettre plus que le mécanisme (finding F3-9) :
     « … Il en va de même de toutes les opérations courantes qui écrivent au journal — hors la
     restauration d'une sauvegarde et l'effacement des données de démonstration. Seul un interblocage
     répété trois fois de suite rend encore une erreur ; rien n'est alors écrit et l'opération peut
     être relancée telle quelle (#463, #491, #536). » ; et une seconde ligne : « Les manuels ne
     prétendent plus que la numérotation des factures se fait dans une transaction `SERIALIZABLE` :
     ils décrivent le verrou du compteur, la contrainte d'unicité et le rejeu (#484). »
   - **Manuels — #484, fermée ici** (choix C60, finding F2-4). Deux passages affirment un mécanisme
     qui n'existe pas :
     - `docs/manual/fr/user-manual.tex:905` (§ « Numérotation des factures ») : « L'attribution du
       numéro se fait dans une transaction DB `SERIALIZABLE` pour garantir l'unicité même en cas de
       validations concurrentes. » → réécrit sur le mécanisme réel, au niveau du lecteur du manuel
       utilisateur : le compteur est **verrouillé** le temps de la validation
       (`invoice_number_sequences.rs:35-38`, `SELECT … FOR UPDATE`) et une contrainte d'unicité
       interdit deux fois le même numéro (`uq_invoices_number`,
       `20260417000001_invoice_validation.sql:57`) : deux validations simultanées reçoivent deux
       numéros distincts ; une validation qui croise une autre opération est **rejouée
       automatiquement** ; si le conflit se répète (trois fois de suite), la validation échoue sans
       rien enregistrer et peut être relancée (finding F3-9 : ne pas écrire « sans que l'utilisateur
       le voie » sans cette réserve) ;
     - `docs/manual/fr/admin-manual.tex:885-889` (§ « Configuration MariaDB », commentaire du fichier
       `99-kesh.cnf`) : « Kesh utilise des transactions explicites SERIALIZABLE pour les operations
       critiques comme la numerotation des factures et la reconciliation bancaire » → réécrit : Kesh
       garde l'isolation par défaut `REPEATABLE READ`, protège ses opérations critiques par des verrous
       de ligne (`SELECT … FOR UPDATE`) et des verrous nommés (rapprochement), et **rejoue
       automatiquement** une opération victime d'un interblocage (trois tentatives). Le même fichier
       gagne la consigne de **laisser `innodb_deadlock_detect` à sa valeur par défaut (`ON`)** :
       désactivé, un interblocage n'est plus détecté, il attend `innodb_lock_wait_timeout` et finit en
       erreur, sans rejeu (finding F2-7). Le commentaire reste en ASCII, comme le reste du listing.
     **PDF** : `make fr` dans `docs/manual/` régénère les trois PDF ; **seuls `user-manual.pdf` et
     `admin-manual.pdf` sont commités** ; `marketing-brochure.pdf`, dont la source ne change pas, est
     **restauré** (`git checkout -- docs/manual/fr/marketing-brochure.pdf`) — des octets changés sans
     texte changé ne racontent rien (choix C64, finding F3-9). Les deux PDF sont **contrôlés aplatis** :
     `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g'` —
     `SERIALIZABLE` absent des deux PDF, les phrases neuves présentes.
     **Relevé élargi** (T3, `.tex` **et** PDF aplatis, consigné au Dev Agent Record ; motif étendu
     en P3, finding F3-5) :
     `grep -rniE "SERIALIZABLE|isolation|interblocage|deadlock|1213|erreur interne|erreur 500|concurren|simultan|en même temps|racing|race condition|atomi|verrous? de (ligne|base)|FOR UPDATE" docs/manual/fr/*.tex`
     — relevé sur `94f1365e` : **18 lignes** — les deux passages ci-dessus (`user-manual.tex:905`,
     `admin-manual.tex:885-886`) ; `admin-manual.tex:889` (`transaction-isolation =
     REPEATABLE-READ`, vrai) ; `admin-manual.tex:1395` (« en même temps », sauvegarde des documents,
     hors sujet) ; dix occurrences d'« isolation » au sens **multi-société** (`admin-manual.tex:86`,
     `:932`, `:948`, `:954`, `:2241`, `user-manual.tex:1993`, `:1996`, `marketing-brochure.tex:207`,
     `:317`, `:318` — hors sujet) ; et les trois lignes que le motif étendu ajoute :
     `user-manual.tex:1532` (lot « atomique », renvoi à « la doc CLAUDE.md projet ») — **réécrit par la
     15-5c** (son AC6, #481), **non touché ici** ; `admin-manual.tex:1856` (« Aucun verrou de base de
     données ne s'y oppose », protection de la table d'audit, hors sujet) ; `marketing-brochure.tex:360`
     (« pas de race conditions silencieuses », propriété du langage Rust, hors sujet). Chaque
     occurrence restante au dev est triée. Les manuels DE, IT, EN ne contiennent qu'un `README.md`
     (rien à traduire) ; la brochure ne parle de « transactions » qu'au sens bancaire (`:167`, `:390`).
6. **AC6 — Aucune règle métier ne change, et c'est vérifié.** Aucun refus neuf, aucune priorité de
   refus déplacée (les contrôles qui lisent la transaction restent dans la tentative, dans leur ordre —
   AC1), aucune variante de `DbError`, aucun code d'erreur, aucune clé i18n, aucun écran ; les
   écritures produites sont identiques ; la suite existante reste verte **sans modification de ses
   assertions**.

## Tasks / Subtasks

- [ ] **T0 — Refaire les relevés** sur `HEAD`, **après le merge de la 15-5e1** : la remontée de l'AC1
      de la 15-5e1 (une route qui écrit au journal absente de la table **bloque la story**) ; pour
      chacune des 14 routes et des 3 sites migrés, le chemin d'erreur (aucun `map_err` hors
      `map_db_error`), l'absence d'effet de bord hors transaction, les entrées à cloner, les contrôles
      qui dépendent de la transaction (AC1) ; numéros de ligne des commentaires de l'AC3 et du
      Pattern 5 ; consigner au Dev Agent Record.
- [ ] **T1 — Le rollout** (AC1) : les 14 routes, dont les trois fonctions « une tentative » de
      `complete_import`, `post_manual`, `post_split` ; la migration des trois sites ; le registre
      (`ARejouer` retiré, partition 21 / 4 / 90, `retry_with` restreint à `post_accept`) ;
      doc-comments, dont celui de `write_off_invoice_handler`. **Revue fichier par fichier.**
- [ ] **T2 — Les commentaires d'ordre et le Pattern 5** (AC3) : `journal_entries.rs`, `accept_one_rule`,
      `post_manual`, `fiscal_years.rs:497-502` ; le Pattern 5 ; l'inventaire au symptôme (181 lignes),
      trié occurrence par occurrence au Dev Agent Record.
- [ ] **T3 — Documentation** (AC4, AC5) : `docs/api-external.md` ; CHANGELOG ; les deux passages
      `SERIALIZABLE` des manuels et la consigne `innodb_deadlock_detect` (#484) ; deux PDF régénérés,
      brochure restaurée, contrôlés aplatis ; relevé élargi (18 lignes) trié, consigné.
- [ ] **T4 — Les mutations** (AC2), consignées.
- [ ] **T5 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) —
      **même en cours de boucle de revue**, la story touchant des repositories `kesh-db` (commentaires)
      et le registre ; gate frontend complet (rien n'y change : il le confirme) ; **E2E Playwright
      complet au dernier commit de code** (décision D7), jugé fichier par fichier contre
      `docs/testing.md` § « Les échecs attendus ».

*(Décompte : 6 AC, 6 tâches T0–T5.)*

## Dev Notes

### L'inventaire

La table « Les routes qui écrivent au journal » est dans la **15-5e1** (Dev Notes), avec sa colonne
« fiche » : les lignes marquées 15-5e2 sont le périmètre de l'AC1 — enveloppe, fonction de dépôt et
correspondance d'erreur à faire **après** le rejeu y sont écrites. Elle n'est pas recopiée ici (une
seule table à tenir à jour).

### Pourquoi la revue fichier par fichier suffit au rollout

Chaque route reçoit la même forme (un appel d'enveloppe autour d'une fonction de dépôt, ou une
fonction « une tentative »), le registre garde la présence de l'enveloppe (volet (c), robuste depuis
la 15-5e1) et la partition, les tests 2 à 4 de la 15-5e1 prouvent le mécanisme sur une vraie 1213.
Ce que la revue doit voir, route par route : (1) la fermeture commence par la transaction et finit par
le commit ; (2) aucun contrôle qui lit la transaction n'en sort, aucun refus ne change de place ; (3)
les entrées consommées sont clonées dans la fermeture ; (4) la conversion d'erreur propre à la route
vient après le rejeu ; (5) le nom d'opération est unique et suit `"<module>::<action>"`.

### Ce que vit l'utilisateur

Comme pour les trois routes de la 15-5e1 : l'opération aboutit, avec au pire ≈ 150 ms de latence
ajoutée ; seul un interblocage répété trois fois de suite ressort en 500, rien n'étant alors écrit.

### Fichiers touchés (prévision)

`crates/kesh-api/src/routes/{invoices,credit_notes,supplier_invoices,imported_supplier_invoices,payment_batches,journal_entries,opening_balances,reconciliation}.rs` ;
`crates/kesh-db/src/repositories/{journal_entries,fiscal_years}.rs` (commentaires) et, selon le tri
de l'inventaire, des commentaires de `crates/*/tests` ; `crates/kesh-api/tests/audit_route_registry.rs` ;
`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `docs/api-external.md`, `CHANGELOG.md` ;
`docs/manual/fr/{user-manual,admin-manual}.{tex,pdf}` (#484). **Aucune migration**, aucun fichier
`kesh-i18n` ni `frontend`.

**Règle de découpage** : plus de cinq modules de routes, mais c'est le **rollout mécanique** que la
règle prescrit après une story-zéro (§ *Règle de splitting préventif*, « Comment splitter ») — revu
fichier par fichier, sans passe adversariale globale sur le rollout lui-même ; les parties
documentaires (Pattern 5, manuels, inventaire) restent relues par les passes de validation.

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C54**, **C55** — le rejeu comme défense ; ce qui reste de l'ordre des verrous.
- **C56** — deux enveloppes partagées, le registre (dérogation de découpage révisée par **C61**).
- **C57** — #536 fermée ici (l'avoir), la validation l'étant par la 15-5e1.
- **C58** — seconde colonne du registre d'audit.
- **C59** / **C62** — journalisation `warn`, nom d'opération en champ de l'événement.
- **C60** — #484 fermée ici.
- **C61** — le découpage 15-5e1 / 15-5e2.
- **C63** — statut transitoire `ARejouer`, retiré ici ; volet (c) robuste, `retry_with` restreint.
- **C64** — ligne du CHANGELOG étendue ici ; PDF de la brochure restauré.

### References

- Issues : #536, #484 (fermées ici) ; #463, #491 (fermées par la 15-5e1) ; #481 (le passage
  `user-manual.tex:1532`, 15-5c).
- Fiches : `15-5e1-socle-rejeu.md` (le patron, l'inventaire, la règle), `15-5e-ordre-des-verrous-reglements.md`
  (index, historique des validations P1–P3), `15-5c-rapprochement-libelles-et-manuel.md` (AC6 :
  `user-manual.tex:1532`).
- `CLAUDE.md` : § *Inventorier les sites NON RÉSOLUS*, § *Le prompt d'une passe doit NOMMER le manuel*
  (PDF aplati), § *Règle de splitting préventif*, § *Propagation post-patch*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-08 — **Créée par le découpage de la 15-5e** (choix **C61**, finding **F3-2 MEDIUM** de sa
  validation P3). Rollout du patron de la 15-5e1 : reprend de la 15-5e (version d'avant découpage au
  commit `94f1365e`) les routes à rejouer hors des trois des issues, la migration des trois sites
  existants, les commentaires d'ordre de son AC5 hors ceux que la 15-5d attend (restés à la 15-5e1,
  C65), le Pattern 5, l'inventaire au symptôme, son AC6 (CHANGELOG étendu, `api-external`, manuels
  #484). **Remédiations de la P3 appliquées ici** : F3-4 (inventaire étendu à `crates/*/tests`,
  181 lignes / 42 fichiers sur `94f1365e` ; `fiscal_years.rs:497-502` réécrit ; tri assumé des
  phrases de tests ; `reconciliation_e2e.rs:4414` gardée, historique) ; F3-5 (motif du relevé des
  manuels étendu, 18 lignes ; `user-manual.tex:1532` à la 15-5c) ; F3-8 et R3-4 (règles de la 15-5e1
  appliquées route par route, cas de `complete_import` et `cancel_reconciliation_once` nommés) ;
  F3-9 (CHANGELOG et manuel avec la réserve des trois tentatives ; brochure restaurée) ; R3-1
  (`invoices.rs:1324-1327`) ; R3-6 (e) (puce « CI lint » retirée). **Recompte** (commandes exécutées
  sur `94f1365e`) : inventaire au symptôme 148 + 33 = 181 lignes, 32 + 10 = 42 fichiers ; relevé des
  manuels 15 + 3 = 18 lignes ; routes : 14 à rejouer + 3 à migrer + `post_accept` = 18 = 21 − 3.
  Décompte de la fiche : **6 AC, 6 tâches T0–T5** (recompté).
