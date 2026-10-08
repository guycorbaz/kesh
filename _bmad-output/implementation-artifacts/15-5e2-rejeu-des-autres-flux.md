# Story 15.5e2 : Rejeu sur interblocage des autres flux d'écriture — rollout, commentaires d'ordre, manuels

Status: ready-for-dev

<!-- Sous-story de la 15-5e (fiche index `15-5e-ordre-des-verrous-reglements.md`, statut `split`),
     créée le 2026-10-08 par le découpage décidé à la validation P3 de la 15-5e (finding F3-2, choix
     C61). C'est le ROLLOUT du patron que pose la 15-5e1 (`15-5e1-socle-rejeu.md`) : application
     mécanique des deux enveloppes aux autres routes qui écrivent au journal, revue fichier par
     fichier ; plus les commentaires d'ordre, le Pattern 5, les manuels (#484), le CHANGELOG et
     `docs/api-external.md`. Le contenu vient de la 15-5e validée P1 à P3 (commit `94f1365e` pour la
     version d'avant découpage) ; les remédiations de la P3 sont appliquées ici et dans la 15-5e1.
     Choix applicables : C54, C55 (révisé par C66), C56 (révisé par C61, C63), C57 (révisé par C61,
     cf. C69), C58, C59 (révisé par C62), C60, C61, C62, C63, C64, C65 (révisé par C66 et C68), C66,
     C68, C69, C70.
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation. Le
     développement attend la clôture de la boucle de validation ET le merge de la 15-5e1. Numéros de
     ligne relevés sur `f289414e` (15-5a et 15-5b mergées) sauf mention contraire. -->

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
rejeu de la saisie fournisseur, choix C66 ; les doc-comments canoniques de `validate_invoice` et de
`supplier_invoices::create_in_tx`, « 5 bis », le module `retry.rs` réécrits), donc des 15-5a et
15-5b (mergées, `f289414e`). **Ordre libre par rapport à la 15-5d** (choix C65, révisé par C68) :
- pour le **code**, la 15-5d n'ajoute aucune route et ne change aucun handler ; si elle merge avant
  cette story, la complétion d'import porte ses verrous sans rejeu jusqu'à ce merge (la saisie
  fournisseur, elle, est rejouée par la 15-5e1) ;
- pour la **documentation de l'ordre des verrous**, l'ordre de merge est sûr dans les deux sens
  parce que le Pattern 5 **ne recopie plus** l'ordre des deux flux que la 15-5d modifie : ses lignes
  `POST /supplier-invoices` et `invoices::validate_invoice` **renvoient aux doc-comments
  canoniques** (AC3, choix C68), seuls lieux de vérité, que la 15-5d met elle-même à jour quand elle
  y place ses verrous (son AC9) ;
- **mais pas pour les fichiers que les deux stories touchent** (finding R5-1 de la P5) : la 15-5d
  régénère et commite elle aussi `user-manual.pdf` et `admin-manual.pdf` (son AC9, sa T6) et
  modifie `user-manual.tex` avant la ligne 909 ; la seconde des deux à merger aura un **conflit
  binaire** sur les PDF — qui ne se résout pas, il se **régénère** : rebase, relocalisation des
  passages des manuels **par leur texte** (les numéros de ligne cités ici auront bougé), puis
  `make admin user` et contrôle aplati (AC5) — et des conflits textuels probables dans
  `CHANGELOG.md` (lignes voisines de « Corrigé »), résolus en gardant les deux lignes.

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

1. **AC1 — Les 17 autres routes `Rejouee` de l'inventaire** (inventaire de la 15-5e1, colonne
   « fiche » = 15-5e2 ; 21 − 4 = 17 — la saisie fournisseur est passée à la 15-5e1, choix C66) :
   - **13 routes à rejouer**, chacune par l'enveloppe que la table de la 15-5e1 lui assigne, avec un
     nom d'opération `"<module>::<action>"` : dévalidation d'une facture ; création d'un avoir ;
     règlement fournisseur, annulation d'une facture fournisseur et de son règlement ; confirmation
     d'un lot de paiement ; écriture manuelle (création, suppression, contre-passation) ; bilan
     d'ouverture (`DbError`, `map_opening_balances_error` **après**) — **10** routes de la famille
     `DbError` ; complétion d'une facture importée, rapprochement manuel et ventilé (`AppError`,
     partie transactionnelle extraite en fonction « une tentative », patron `accept_once`,
     `routes/reconciliation.rs:917`, et `cancel_reconciliation_once`, `:3975`, qui ouvre la
     transaction, prend le verrou nommé du compte bancaire le cas échéant, écrit et conclut) — **3**
     de la famille `AppError`.
   - **3 sites existants migrés** vers les enveloppes (DRY) : `write_off_invoice_handler`
     (`routes/invoices.rs:1343`) et `complete_opening_balances` (`routes/opening_balances.rs:594`)
     par l'enveloppe `DbError` ; `post_cancel_reconciliation` (`routes/reconciliation.rs:3960`) par
     l'enveloppe `AppError` ; leurs noms d'opération (posés par la 15-5e1) sont gardés.
   - **`post_accept` inchangée** (`routes/reconciliation.rs:888`) : son prédicat élargi au 1305 est propre à
     la route et le reste ; elle est le seul `retry_with` direct restant parmi les routes `Rejouee`.
     `onboarding::finalize` (`routes/onboarding.rs:614`) n'écrit pas au journal et garde son `retry_with`.
   - **Les règles de la 15-5e1 s'appliquent route par route**, vérifiées en T0 et en revue fichier par
     fichier : une tentative = une transaction neuve ; les contrôles qui ne dépendent pas de la
     transaction restent hors de la fermeture, **ceux qui lisent ce que la transaction verrouille
     restent dans la tentative, dans le même ordre** (finding R3-4 : dans `complete_import`
     (`routes/imported_supplier_invoices.rs:123-290`), les contrôles de devise, QR-IBAN et montant
     lisent le `staging` pris `FOR UPDATE` et restent dans `complete_import_once` après lui ; dans
     `cancel_reconciliation_once`, la lecture de `bank_account_id`
     (`routes/reconciliation.rs:3982-3992`), déjà dans la tentative, y reste) ; les entrées consommées
     sont **clonées par tentative** (finding F3-8 : `create_credit_note` et `create_journal_entry`
     consomment `Json(req)` pour construire un `NewCreditNote` / `NewJournalEntry`, tous deux
     `Clone` — patron `onboarding::finalize`, pas de `Option::take` ; `confirm_payment_batch` ne lit
     que `req.payment_date`, `Copy`, rien à cloner, finding R4-5) ; les trois fonctions « une
     tentative » (`complete_import_once`, et celles de `post_manual` et `post_split`) reçoivent le
     corps **par référence** (`&CompleteImportRequest`, `&ManualMatchBody`, `&SplitBody`) : aucun
     de ces trois types ne dérive `Clone` (`routes/imported_supplier_invoices.rs:99-118`,
     `routes/reconciliation.rs:2981`, `:3423`), et il n'y a pas à le dériver — T0 le vérifie
     (finding F5-2 de la P5) ; aucun effet de bord hors
     transaction ; le 1213 atteint le prédicat — **aucune conversion qui fasse sortir une erreur sqlx
     de `DbError::Sqlx` avant le prédicat** (règle de l'AC2 de la 15-5e1, portée sur l'effet et non
     sur la forme : un `map_err(|e| …DbError::Sqlx(e))` écrit à la main, comme
     `routes/reconciliation.rs:3991`, la respecte — finding R4-5).
   - **Le registre** (`audit_route_registry.rs`) : les 13 statuts `ARejouer` deviennent `Rejouee` et
     la variante **`ARejouer` est retirée** de l'énumération ; la partition (volet (d)) devient **21**
     `Rejouee`, **4** `Exemptee`, **90** `SansEcritureAuJournal`, total **115** ; le volet (c)
     **n'accepte plus `retry_with` que pour `post_accept`** (liste nommée dans le test) — un
     `retry_with` recopié sur une autre route `Rejouee` le fait rougir (le volet (c) n'examine pas
     les routes `SansEcritureAuJournal` : `onboarding::finalize` garde le sien, finding F4-11).
   - Chacun des doc-comments des 13 routes et des 3 sites migrés dit, en une ligne, que la route est
     rejouée sur interblocage, avec renvoi à l'enveloppe (pas de recopie de la règle) ; celui de
     `post_cancel_reconciliation` (`routes/reconciliation.rs:3946`) et celui de
     `complete_opening_balances` (`routes/opening_balances.rs:573-578`, la mention de `retry_with`
     étant `:576`, finding R5-2) cessent de nommer `retry_with`.
     **Les mentions de `retry_with` qui désignent l'appelant d'un site migré**, hors des doc-comments
     de route, nomment l'enveloppe (findings R4-2, F4-6 de la P4 ; trou antérieur au découpage,
     R4-10 de la 15-5e1, attribué ici) : `crates/kesh-db/src/repositories/opening_complement.rs:36`
     (doc du module : « l'appelant enveloppe l'appel dans `retry_with` ») et `:405` (« l'enveloppe
     dans `retry_with` »). Après la migration, `grep -rn "retry_with" crates/*/src docs` ne rend plus
     que `retry.rs`, `post_accept`, `onboarding::finalize` (code et doc-comments) et les phrases qui
     désignent `retry_with` comme la primitive des enveloppes ; chaque autre occurrence est corrigée
     ou justifiée au Dev Agent Record. Le doc-comment
     de `write_off_invoice_handler` (`crates/kesh-api/src/routes/invoices.rs:1324-1327` ; la phrase
     sur l'ordre est `:1324-1326`, celle sur la `version` `:1326-1327` — finding R3-1 ; F1-3) :
     « l'ordre des verrous est celui du règlement manuel (facture → compte → exercice), qui peut
     former un cycle avec l'acceptation d'un rapprochement (#491) » est réécrit : la route est rejouée
     comme toute route d'écriture au journal ; la phrase sur la `version` (« une tentative qui trouve
     un reste changé est refusée en 409 ») est gardée.

### Ce qui doit être prouvé

2. **AC2 — Tests.** Pas de test dynamique neuf (choix C56 : les tests 2 à 5 et 7 de la 15-5e1
   prouvent le patron sur cinq routes de la famille `DbError` — quatre qui écrivent au journal et
   l'enregistrement des réglages, finding R6-2 de la P6 de la 15-5e1 —, chacun avec un témoin du
   rejeu (C74), et son test 1 le prédicat et l'enveloppe
   `AppError` ; pour les autres, c'est la revue fichier par fichier et le registre — le doc-comment du
   registre le dit, finding F2-10). **Angle mort assumé** (finding F4-8) : aucune route de la famille
   `AppError` n'a de preuve dynamique — le chemin propre à `post_manual` et `post_split` (1213 levé
   sous le **verrou nommé**, converti par le `match` du handler, `RELEASE_LOCK` puis `rollback`,
   nouvelle tentative) et celui de `complete_import` (1213 levé sous le verrou du **`staging`**,
   pris `FOR UPDATE`, `routes/imported_supplier_invoices.rs:141` ; `match` local, `rollback`,
   nouvelle tentative — **aucun verrou nommé** sur cette route, finding F5-1 de la P5 ; son risque
   propre est l'ordre des contrôles lus sur le `staging`, R3-4, AC1) ne sont exercés nulle part ; seul
   `accept_replays_the_batch_when_it_is_the_deadlock_victim` (`reconciliation_e2e.rs:4426`) exerce un
   chemin voisin. Ces trois extractions reçoivent donc une lentille adversariale de la revue de code
   (Dev Notes, « Règle de découpage »).
   - Les tests existants des routes touchées restent verts **sans modification de leurs assertions**,
     ainsi que `accept_replays_the_batch_when_it_is_the_deadlock_victim` et les tests 1 à 5 et 7 de
     la 15-5e1 — **hors** le registre `audit_route_registry.rs`, dont l'AC1 change la partition, retire
     la variante `ARejouer` et restreint l'acceptation de `retry_with` (findings R4-6, F4-7).
   - **Mutations** (consignées au Dev Agent Record, en touchant le fichier après restauration — sans
     quoi cargo garde le binaire muté), **une par famille** (finding F3-1) :
     - famille `DbError` à correspondance après le rejeu : retirer l'enveloppe de
       `create_journal_entry` → le volet (c) rougit ;
     - famille `DbError` dont le doc-comment de la route suivante nomme l'enveloppe : retirer
       l'enveloppe de `pay_supplier_invoice` (suivi du doc-comment de `cancel_supplier_invoice`,
       `routes/supplier_invoices.rs:~392-397`) → le volet (c) rougit. Depuis que le volet (c)
       analyse le source avec `syn` (15-5e1, choix C70), un doc-comment est un attribut de l'item
       **qu'il précède**, hors du corps examiné : la mutation éprouve sur le source réel le cas
       qui motivait jadis les précautions textuelles (un doc-comment voisin qui nomme
       l'enveloppe) ;
     - famille `AppError` : retirer l'enveloppe de `post_manual` → le volet (c) rougit ;
     - remplacer l'enveloppe de `write_off_invoice_handler` par un `retry_with` recopié → le volet (c)
       rougit (acceptation de `retry_with` restreinte à `post_accept`).
   - **Aucun test de « non-interblocage »** (finding F1-4 de la P1 de la 15-5e).

### L'ordre des verrous : les commentaires et le Pattern 5 (choix C54, C55)

3. **AC3 — Les commentaires disent la règle vraie, et aucun ne prétend à l'absence de cycle.** Le
   doc-comment canonique de `validate_invoice`, celui de `supplier_invoices::create_in_tx`, « 5 bis »
   et le module `retry.rs` sont réécrits par la 15-5e1 ; le reste est ici.
   - **Les commentaires qui fondent une absence de cycle sur « l'ordre global »** (finding F2-1) :
     - **l'étape 0** de `create_in_tx` (`crates/kesh-db/src/repositories/journal_entries.rs:249-253` :
       « AVANT le lock fiscal_years pour respecter l'ordre de verrouillage global companies →
       projects → fiscal_years […] créerait une inversion ABBA inter-flux ») et
       `create_opening_entry` (`:547-549` et `:561-562` : « pas d'inversion de l'ordre de verrou
       global », « respecte l'ordre de verrou global — pas d'inversion ABBA ») sont **réécrits** :
       prendre la sentinelle et les projets avant l'exercice **réduit la fréquence** des
       interblocages, il ne les exclut pas. L'étape 0 précède l'étape 1 (le re-verrou de l'exercice,
       `:291-294`), **pas l'exercice que l'appelant a souvent déjà pris** : elle ne vient avant
       l'exercice que pour `journal_entries::create` ; les flux qui verrouillent l'exercice avant
       d'appeler `create_in_tx` prennent l'ordre **inverse**, exercice **puis** sentinelle et
       projets (finding F4-1 de la P4) :
       - le lot de rapprochement par **règle** (`accept_one_rule`,
         `crates/kesh-api/src/routes/reconciliation.rs:2274` : exercice `:2461`, puis
         `validate_taggable_in_tx` `:2493` → sentinelle `crates/kesh-db/src/repositories/projects.rs:99`) ;
       - le lot de rapprochement par **ventilation** (`accept_one_split`, `routes/reconciliation.rs:1876` :
         exercice `:2108`, puis l'étape 0 de `create_in_tx` `:2150` sur les projets de chaque ligne,
         `:2128-2131`) ;
       - le rapprochement **manuel** à projet (`post_manual` : exercice `routes/reconciliation.rs:3222`,
         puis `validate_taggable_in_tx` `:3234`) ;
       - le rapprochement **ventilé** (`post_split` : exercice `routes/reconciliation.rs:3719`, puis
         l'étape 0 de `create_in_tx` `:3736` sur les projets des lignes) ;
       contre `journal_entries::create` (étape 0 puis étape 1), `create_opening_entry` (sentinelle
       puis exercice) et la saisie fournisseur à projet. Les deux côtés de chaque cycle sont rejoués
       (renvoi à l'enveloppe). **Cette liste n'est pas close** : l'inventaire au symptôme ci-dessous
       et T0 cherchent les autres (clause de la 15-5e d'avant découpage, perdue au découpage et
       rétablie — F4-1, preuve 3). L'inversion est écrite au commentaire de chacun
       (`routes/reconciliation.rs:2484-2490`, Step 11bis d'`accept_one_rule` ; `:2128-2131` pour
       `accept_one_split` ; et `:3227-3232`, Step 6bis de `post_manual`, qui affirme aujourd'hui
       l'inverse — « cohérent Pattern 5 (avant le lock fiscal_years pris par create_in_tx) », alors
       que l'exercice est déjà verrouillé à l'étape 6 —, réécrit ; le point d'appel de
       `create_in_tx` dans `post_split`, `:3736`, gagne la même phrase).
     - **l'étape 0-bis** (`journal_entries.rs:264-282`) : la puce « Verrous : `companies` vient EN
       PREMIER dans l'ordre global (…) Lire la borne après le lock `fiscal_years` de l'étape 1
       créerait une inversion ABBA » (`:270-273`) est **retirée** — non reformulée (finding F4-2) :
       la lecture de `books_locked_through` n'est **pas verrouillante** (`:283-289`, et le
       commentaire le dit lui-même, `:278-282`), elle ne peut donc participer à aucun interblocage.
       Sa place n'est dictée que par l'**ordre des refus** : la puce « Refus » (`:274-276`) et
       l'avertissement « Lecture NON verrouillante » sont gardés ; la phrase d'en-tête qui oppose
       l'ordre des verrous à celui des refus (`:266-268`) est ajustée en conséquence.
   - **`crates/kesh-db/src/repositories/fiscal_years.rs:499-502`** (finding F3-4) : « Toute
     divergence = risque de deadlock avec `journal_entries::create_in_tx` en cours sur la même
     company », jumelle de la phrase que la 15-5e1 retire de
     `crates/kesh-db/src/repositories/invoices.rs:1928` — **réécrite** : la place de l'exercice est
     une convention de fréquence, la défense est le rejeu. Le bloc qui la porte (`:489-502`) est le
     doc-comment de `find_open_covering_date` **égaré** au-dessus de `find_covering_date_in_tx`
     (`:516`), suivi sans séparation de la doc de celle-ci ; `find_open_covering_date` (`:543`) n'a
     plus de doc-comment. La réécriture le **remet à sa place** (finding F4-5).
   - **Gardés, vrais pour la paire qu'ils nomment** : `repositories/projects.rs:77-80`,
     `repositories/supplier_invoices.rs:278-280` et `repositories/reconciliation_rules.rs:190-191`
     (« anti-ABBA » avec le chemin d'archivage d'un projet : sentinelle puis projet, des deux côtés),
     avec la seule précision qu'ils ne valent que pour cette paire si le texte laisse entendre plus ;
     `repositories/journal_entry_number_sequences.rs:130` (« pas de cycle entre créations », le
     compteur pris en premier par toute création), vrai entre créations.
   - **Inventaire au symptôme des affirmations d'ordre ou d'absence de cycle** (règle *Inventorier les
     sites NON RÉSOLUS* ; findings F2-1, R2-2, F3-4 — périmètre étendu aux tests ; **insensible à la
     casse** depuis la P4, finding F4-5) :
     `grep -rniE "ABBA|inversion|deadlock|interblocage|cycle|ordre global|global lock order|lock order|ordre des (locks|verrous)|follow the documented order|deny list" crates/*/src crates/*/tests docs/*.md --include=*.rs --include=*.md`
     — relevé sur `f289414e` : **205 lignes dans 42 fichiers** (la forme sensible à la casse en
     rendait 181, et manquait p. ex. `onboarding.rs:232`, `:592` « LOCK ORDERING »,
     `repositories/invoices.rs:1064` « **Ordre des locks** », `fiscal_years.rs:499` « Ordre des
     locks », `routes/opening_balances.rs:578` « Cycles », finding R5-2) — **169 lignes / 32 fichiers** sous
     `crates/*/src` et `docs/*.md` (dont 52 dans `retry.rs`, en grande partie ses tests, 20 dans
     `repositories/invoices.rs`, 17 dans `MULTI-TENANT-SCOPING-PATTERNS.md`, 12 dans
     `routes/reconciliation.rs`, 11 dans `routes/onboarding.rs`) et **36 lignes / 10 fichiers sous
     `crates/*/tests`** (12 dans `reconciliation_e2e.rs`, 8 dans `opening_complement_repository.rs`,
     5 dans `invoice_unvalidate_e2e.rs`, 3 dans `supplier_invoices_repository.rs`, 3 dans
     `fiscal_years_repository.rs`, une dans chacun de cinq autres). **Chaque occurrence est triée au
     Dev Agent Record**, une par une, **en lisant le bloc de commentaire entier** et non la seule
     ligne rendue — une phrase coupée en fin de ligne échappe au motif (p. ex.
     `fiscal_years.rs:513-515`, « inverserait l'ordre des / verrous ») : *vraie* (et pourquoi),
     *réécrite* (ici, ou par la 15-5e1 — doc-comments canoniques, « 5 bis », `retry.rs`), ou *hors
     sujet* (autre mécanisme). `crates/kesh-api/src/routes/onboarding.rs:592-595` (« New endpoints
     with multiple locks MUST follow the same order to avoid cross-table deadlocks ») est la jumelle
     de la « Mitigation (v0.1) » du Pattern 5 réécrite ci-dessous : même traitement. Pour les tests,
     le tri est assumé ainsi : une phrase qui **asserte** une absence d'interblocage
     (`crates/kesh-db/tests/opening_complement_repository.rs:746` « aucun interblocage », `:800`
     « l'écriture tenant l'exercice passe : aucun interblocage » ; `fiscal_years_repository.rs:1071`
     « Le FOR UPDATE sérialise → pas de deadlock ») est **vraie si et seulement si elle décrit le
     montage du test** (deux transactions qui ne peuvent pas former de cycle par construction) — elle
     est alors précisée en ce sens, sans toucher aux assertions ; sinon réécrite.
     `crates/kesh-api/tests/reconciliation_e2e.rs:4411-4414` (« Avant la 25-4-c2, […] `retry_with`
     n'existant que sur l'annulation ») est **vraie, historique** — gardée ; le motif rend `:4409`,
     première ligne du même bloc, pas `:4414` (finding R4-3). Une occurrence qui affirme encore une
     absence de cycle sans la limiter à la paire ou au montage qu'elle nomme **bloque la story**.
   - **`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, Pattern 5** (`:285-389`) :
     - « Why Lock Ordering Matters » (`:289-291`, même affirmation fausse sur la détection) est
       réécrit ;
     - « Global Lock Order » (`:293-305`) devient une **convention de fréquence**, qui nomme la
       sentinelle `companies` et les verrous **partagés** que posent à l'insertion les clés
       étrangères des écritures — `fk_journal_entries_company`
       (`20260412000001_journal_entries.sql:27`), `fk_jel_account` (`:45`), `fk_jel_project`
       (`20260702000001_projects_analytics.sql:40-41`) —, les **inversions** relevées ci-dessus
       (règle, ventilation, manuel, ventilé : exercice, puis sentinelle et projets), et la règle
       « toute route qui écrit au journal est rejouée » (renvoi aux enveloppes et au registre) ;
     - la table « Where This Applies » (`:307-318`) : les lignes `POST /supplier-invoices` (`:315`)
       et `invoices::validate_invoice` (`:318`) **ne recopient plus l'ordre** — elles **renvoient au
       doc-comment canonique** de `supplier_invoices::create_in_tx` et de `validate_invoice`, seuls
       lieux de vérité (choix C68, findings F4-3 = R4-1 de la P4) : la 15-5d y ajoute ses verrous
       (son AC9) sans avoir à toucher au Pattern 5, et l'ordre de merge 15-5d / 15-5e2 est sûr dans
       les deux sens. La table **gagne une ligne pour le lot de rapprochement** (`accept_batch`), qui
       dit l'ordre **du lot** et non d'une seule proposition (finding F4-1) : par proposition,
       arrondi puis exercice (`accept_one_invoice`, `routes/reconciliation.rs:1534` → `:1567`), ou
       exercice puis sentinelle et projets (`accept_one_rule`, `accept_one_split`) ; **entre
       propositions**, dans la même transaction (`:1063-1069`, une sauvegarde par proposition),
       l'exercice tenu par la proposition n précède l'arrondi, la sentinelle ou les projets de la
       n+1 — l'ordre **exercice → arrondi** à l'échelle du lot, inverse de la validation (arrondi
       `repositories/invoices.rs:2065`, exercice `:2172`) : c'est le cycle de #536, que la story
       ferme par le rejeu des deux côtés. ⚠️ **Le Pattern 5 nomme les fonctions et les étapes,
       jamais un numéro de ligne** (findings R5-3, F5-3 de la P5) : `accept_one_invoice`,
       `rounding_account_for_write`, `find_open_covering_date`, les sauvegardes par proposition
       d'`accept_batch` ; les numéros ci-dessus servent à la relecture de cette fiche — la 15-5e1
       réécrit les doc-comments de `repositories/invoices.rs` et la 15-5d insère ses verrous entre
       `:2065` et `:2172`, ils bougeront avant ce merge (même logique que C68) ;
     - « Known Risk » (`:320-324`, finding R2-2) : la « Mitigation (v0.1) » de `:324` — « all
       current write endpoints follow the documented order. New endpoints MUST follow it or be added
       to the deny list » — est réécrite (l'ordre est une convention de fréquence, la défense est le
       rejeu, le registre garde les routes) ;
     - « Resolution status » (`:326-330`) : la puce du helper de rejeu (`:328`) nomme les deux
       enveloppes au lieu de « `finalize` via `retry_with` » ; la puce **« CI lint (grep-detect
       `FOR UPDATE` + verify global order) »** (`:329`, ⏳) est **retirée** (finding R3-6 (e)) — sans
       ordre global garanti, il n'y a plus d'ordre à vérifier, et le registre testé de la 15-5e1
       tient le rôle de garde ; la puce « Deny list of divergent endpoints » (`:330`) est retirée ;
       la rubrique « Deny list » (`:332-336`) est retirée ;
     - l'exemple de code « How to use the retry helper » (`:338-351`) nomme les deux enveloppes au
       lieu d'un `retry_with` recopié ;
     - le paragraphe **« Required »** (`:353`) est **réécrit sur la règle de la 15-5e1** (finding
       F4-4) : il demande aujourd'hui que `handler_inner` soit « idempotent » et met en garde contre
       les compteurs et les lignes d'audit ; or un 1213 **annule toute la transaction**, compteurs et
       audit compris, et le rejeu est sûr **si toute écriture vit dans la transaction** — une
       tentative = une transaction neuve, aucun effet de bord hors transaction (e-mail, fichier,
       réseau), contrôles qui lisent la transaction gardés dans la tentative et dans leur ordre,
       entrées clonées par tentative, aucune conversion qui fasse sortir le 1213 de
       `DbError::Sqlx` avant le prédicat. Le texte renvoie à `kesh_db::retry` / `kesh_api::retry` et
       à la 15-5e1, sans recopier la règle ;
     - **« When to Use »** (`:355-362`) est réécrit (finding F4-4) : l'enveloppe s'applique à
       **toute route qui écrit au journal** (le registre le vérifie), quel que soit le nombre de
       tables qu'elle verrouille ; l'ordre reste une **convention de fréquence** pour toute
       transaction qui prend plusieurs verrous ;
     - « Code Reference » (`:364-389`, « WRONG: reverse order will deadlock ») dit « réduit la
       fréquence », pas « évite ».

### Documentation

4. **AC4 — `docs/api-external.md` § 10 « Gestion des erreurs »** (`:401`) gagne une phrase générale :
   toute route qui écrit au journal — hors `POST /admin/full-import` et `POST /onboarding/reset`, qui
   ne rejouent pas (finding F2-6) — rejoue un interblocage transitoire sans le montrer ; s'il persiste
   après trois tentatives, la requête finit en `500 INTERNAL_ERROR`, rien n'a été écrit, et elle peut
   être renvoyée telle quelle. Les deux phrases propres à l'acceptation (`:307`) et à l'annulation
   d'un rapprochement (`:317`) restent vraies ; dans celle de `:307`, « après plusieurs
   tentatives » devient « après trois tentatives », pour que le document dise le même nombre
   partout (`DEFAULT_MAX_DEADLOCK_ATTEMPTS = 3`, `crates/kesh-db/src/retry.rs:39` — finding F5-6 de
   la P5) ; `:317` ne parle pas du nombre et n'est pas touchée.
5. **AC5 — CHANGELOG et manuels.**
   - **CHANGELOG**, rubrique **Corrigé** de la section `## [0.13.0] — Non publié` : la ligne posée par
     la 15-5e1 (choix C64) est **remplacée** par ce texte final complet, sans promettre plus que le
     mécanisme (findings F3-9, R4-9), dans la forme des entrées de la section — titre en **gras**
     terminé par les issues en liens (finding F6-4 de la P6 de la 15-5e1) : « **Une opération qui
     croise un rapprochement bancaire ne finit plus en erreur interne ([#463](https://github.com/guycorbaz/kesh/issues/463), [#491](https://github.com/guycorbaz/kesh/issues/491),
     [#536](https://github.com/guycorbaz/kesh/issues/536)).** Un règlement client, l'annulation d'un règlement client, la
     validation d'une facture, la saisie d'une facture fournisseur et l'enregistrement des réglages
     de facturation qui croisent une autre opération (un rapprochement bancaire, notamment) ne
     finissent plus en erreur interne : le
     serveur rejoue l'opération, comme il le faisait déjà pour l'acceptation d'un rapprochement. Il
     en va de même de toutes les opérations courantes qui écrivent au journal — hors la restauration
     d'une sauvegarde et l'effacement des données de démonstration. Seul un interblocage répété
     trois fois de suite rend encore une erreur ; rien n'est alors écrit et l'opération peut être
     relancée telle quelle. » ; et une seconde ligne : « **Les manuels ne prétendent plus que la
     numérotation des factures se fait dans une transaction `SERIALIZABLE` ([#484](https://github.com/guycorbaz/kesh/issues/484)).** Ils
     décrivent le verrou du compteur, la contrainte d'unicité et le rejeu. » La ligne
     « Modifié » de la 15-5e1 (journalisation des rejeux en avertissement) n'est pas touchée.
   - **Manuels — #484, fermée ici** (choix C60, finding F2-4). Deux passages affirment un mécanisme
     qui n'existe pas :
     - `docs/manual/fr/user-manual.tex:909` (§ « Numérotation des factures ») : « L'attribution du
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
       de ligne (`SELECT … FOR UPDATE`) et des verrous nommés (rapprochement), et **relance
       automatiquement l'opération interrompue** par un interblocage (jusqu'à trois tentatives) — pas
       « rejoue » : le manuel d'administration emploie déjà ce mot pour les reprises de données
       rejouées à l'import (`admin-manual.tex:1637-1654`), un mécanisme sans rapport (finding
       F4-10). Le même fichier
       gagne la consigne de **laisser `innodb_deadlock_detect` à sa valeur par défaut (`ON`)** :
       désactivé, un interblocage n'est plus détecté, il attend `innodb_lock_wait_timeout` et finit en
       erreur, sans rejeu (finding F2-7). **Variable vérifiée** sur la version que le manuel exige
       (« MariaDB 10.11+ », `admin-manual.tex:108`) : `SHOW VARIABLES LIKE 'innodb_deadlock%'` sur
       la base de dev (`10.11.16-MariaDB`, image `mariadb:10.11` de `docker-compose.yml:4`) rend
       `innodb_deadlock_detect = ON` (et `innodb_deadlock_report = full`) — relevé le 2026-10-08 à
       la remédiation P5, finding F5-4 ; la consigne s'écrit donc à l'indicatif, T3 refait la
       requête et la consigne au Dev Agent Record. Le commentaire reste en ASCII, comme le reste du
       listing, et chacune de ses lignes tient en **70 caractères** au plus : un `lstlisting` trop
       large déborde de la marge sans que le texte aplati le montre (finding F5-5) — contrôle :
       `grep -c "Overfull" docs/manual/fr/admin-manual.log` avant et après, le nombre ne croît
       pas.
     **PDF** : `make admin user` dans `docs/manual/` (cibles du `Makefile`, `:46-48`) régénère les
     deux PDF des manuels modifiés, et eux seuls (finding R4-8 : `make fr` ne reconstruit la brochure
     que si l'un de ses prérequis a changé, le geste direct rend la restauration inutile) ; **seuls
     `user-manual.pdf` et `admin-manual.pdf` sont commités** — contrôle : si `marketing-brochure.pdf`
     apparaît modifié, il est restauré (`git checkout -- docs/manual/fr/marketing-brochure.pdf`) ; des
     octets changés sans texte changé ne racontent rien (choix C64, finding F3-9). **Si la 15-5d
     (ou la 15-5c) a mergé avant** : les passages se relocalisent **par leur texte**, et les deux PDF
     se **régénèrent après rebase** au lieu de résoudre leur conflit binaire (finding R5-1 ; en-tête
     « Ordre libre »). Les deux PDF sont
     **contrôlés aplatis** :
     `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g'` —
     `SERIALIZABLE` absent des deux PDF, les phrases neuves présentes.
     **Relevé élargi** (T3, `.tex` **et** PDF aplatis, consigné au Dev Agent Record ; motif étendu
     en P3, finding F3-5) :
     `grep -rniE "SERIALIZABLE|isolation|interblocage|deadlock|1213|erreur interne|erreur 500|concurren|simultan|en même temps|racing|race condition|atomi|verrous? de (ligne|base)|FOR UPDATE" docs/manual/fr/*.tex`
     — relevé sur `94f1365e` et sur `f289414e` : **18 lignes** — les deux passages ci-dessus
     (`user-manual.tex:909`, `admin-manual.tex:885-886`) ; `admin-manual.tex:889` (`transaction-isolation =
     REPEATABLE-READ`, vrai) ; `admin-manual.tex:1395` (« en même temps », sauvegarde des documents,
     hors sujet) ; dix occurrences d'« isolation » au sens **multi-société** (`admin-manual.tex:86`,
     `:932`, `:948`, `:954`, `:2241`, `user-manual.tex:1997`, `:2000`, `marketing-brochure.tex:207`,
     `:317`, `:318` — hors sujet) ; et les trois lignes que le motif étendu ajoute :
     `user-manual.tex:1536` (lot « atomique », renvoi à « la doc CLAUDE.md projet ») — **réécrit par la
     15-5c** (son AC6, #481), **non touché ici** ; `admin-manual.tex:1856` (« Aucun verrou de base de
     données ne s'y oppose », protection de la table d'audit, hors sujet) ; `marketing-brochure.tex:360`
     (« pas de race conditions silencieuses », propriété du langage Rust, hors sujet). Chaque
     occurrence restante au dev est triée. Les manuels DE, IT, EN ne contiennent qu'un `README.md`
     (rien à traduire) ; la brochure ne parle de « transactions » qu'au sens bancaire (`:167`, `:390`).
6. **AC6 — Aucune règle métier ne change, et c'est vérifié.** Aucun refus neuf, aucune priorité de
   refus déplacée (les contrôles qui lisent la transaction restent dans la tentative, dans leur ordre —
   AC1), aucune variante de `DbError`, aucun code d'erreur, aucune clé i18n, aucun écran ; les
   écritures produites sont identiques ; la suite existante reste verte **sans modification de ses
   assertions**, hors le registre `audit_route_registry.rs`, dont l'AC1 change la partition et
   l'acceptation de `retry_with` (findings R4-6, F4-7).

## Tasks / Subtasks

- [ ] **T0 — Refaire les relevés** sur `HEAD`, **après le merge de la 15-5e1** : la remontée de l'AC1
      de la 15-5e1 (une route qui écrit au journal absente de la table **bloque la story**) ; pour
      chacune des 13 routes et des 3 sites migrés, le chemin d'erreur (aucune conversion qui fasse
      sortir une erreur sqlx de `DbError::Sqlx` avant le prédicat), l'absence d'effet de bord hors
      transaction, les entrées à cloner, les contrôles qui dépendent de la transaction (AC1) ; les
      flux qui prennent l'exercice avant la sentinelle et les projets, **au-delà des quatre de
      l'AC3** (F4-1) ; `grep -rn "retry_with"` (AC1) ; que les doc-comments canoniques de
      `validate_invoice` et de `supplier_invoices::create_in_tx`, cibles des renvois du Pattern 5,
      disent l'ordre réel sur `HEAD` (15-5d mergée ou non — C68) ; numéros de ligne des commentaires
      de l'AC3 et du Pattern 5 ; que les corps de `complete_import`, `post_manual` et `post_split`
      peuvent passer par référence (AC1, F5-2) ; si la 15-5d a mergé avant, les passages des
      manuels relocalisés par leur texte (AC5, R5-1) ; consigner au Dev Agent Record.
- [ ] **T1 — Le rollout** (AC1) : les 13 routes, dont les trois fonctions « une tentative » de
      `complete_import`, `post_manual`, `post_split` ; la migration des trois sites ; le registre
      (`ARejouer` retiré, partition 21 / 4 / 90, `retry_with` restreint à `post_accept`) ;
      doc-comments, dont celui de `write_off_invoice_handler`, et les mentions de `retry_with`
      d'`opening_complement.rs` (`:36`, `:405`). **Revue fichier par fichier** (Dev Notes).
- [ ] **T2 — Les commentaires d'ordre et le Pattern 5** (AC3) : `journal_entries.rs` (étapes 0 et
      0-bis, `create_opening_entry`), `accept_one_rule`, `accept_one_split`, `post_manual`,
      `post_split`, `fiscal_years.rs:489-502` (réécrit et remis à sa place) ; le Pattern 5 ;
      l'inventaire au symptôme (205 lignes, motif insensible à la casse), trié bloc par bloc au Dev
      Agent Record.
- [ ] **T3 — Documentation** (AC4, AC5) : `docs/api-external.md` ; CHANGELOG ; les deux passages
      `SERIALIZABLE` des manuels et la consigne `innodb_deadlock_detect` (#484) ; deux PDF régénérés,
      brochure non commitée, contrôlés aplatis ; relevé élargi (18 lignes) trié, consigné.
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
la 15-5e1) et la partition. La preuve dynamique n'est **pas** égale entre les deux familles
(finding F4-8) : pour la famille `DbError`, les tests 2 à 5 et 7 de la 15-5e1 prouvent le mécanisme
sur une vraie 1213, sur cinq routes de même forme, chacun avec un témoin du rejeu (C74) ; pour la famille `AppError`, le test 1 de la 15-5e1 ne
prouve que le prédicat et l'enveloppe — le chemin propre à chacune des trois routes (pour
`post_manual` et `post_split` : verrou nommé, `match` du handler, `RELEASE_LOCK`, `rollback` ; pour
`complete_import` : verrou du `staging`, `match` local, `rollback`, et l'ordre des contrôles lus sur
le `staging` — finding F5-1) est un **angle mort assumé** (AC2), couvert par la revue seule.
Ce que la revue doit voir, route par route : (1) la fermeture commence par la transaction et finit par
le commit ; (2) aucun contrôle qui lit la transaction n'en sort, aucun refus ne change de place ; (3)
les entrées consommées sont clonées dans la fermeture ; (4) la conversion d'erreur propre à la route
vient après le rejeu ; (5) le nom d'opération est unique et suit `"<module>::<action>"`.

### Ce que vit l'utilisateur

Comme pour les cinq routes rejouées par la 15-5e1 : l'opération aboutit, avec au pire ≈ 150 ms de latence
ajoutée ; seul un interblocage répété trois fois de suite ressort en 500, rien n'étant alors écrit.

### Fichiers touchés (prévision)

`crates/kesh-api/src/routes/{invoices,credit_notes,supplier_invoices,imported_supplier_invoices,payment_batches,journal_entries,opening_balances,reconciliation}.rs` ;
`crates/kesh-db/src/repositories/{journal_entries,fiscal_years,opening_complement}.rs` (commentaires)
et, selon le tri de l'inventaire, d'autres commentaires de `crates/*/src` (p. ex.
`routes/onboarding.rs:592-595`) et de `crates/*/tests` ; `crates/kesh-api/tests/audit_route_registry.rs` ;
`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `docs/api-external.md`, `CHANGELOG.md` ;
`docs/manual/fr/{user-manual,admin-manual}.{tex,pdf}` (#484). **Aucune migration**, aucun fichier
`kesh-i18n` ni `frontend`.

**Règle de découpage** : plus de cinq modules de routes, mais c'est le **rollout** que la règle
prescrit après une story-zéro (§ *Règle de splitting préventif*, « Comment splitter »). Il n'est pas
**strictement** mécanique (finding F4-9) : trois des treize routes exigent l'extraction d'une
fonction « une tentative » qui garde l'ordre des contrôles (R3-4), et c'est le point que la preuve
dynamique ne couvre pas (F4-8). D'où le partage de la revue de code (`bmad-code-review`, § *Review
Iteration Rule*) : la **revue fichier par fichier** s'applique aux dix routes `DbError` et aux trois
migrations ; les trois extractions (`complete_import`, `post_manual`, `post_split`) reçoivent **au
moins une lentille adversariale** de la revue de code ; les parties documentaires (Pattern 5, manuels,
inventaire) restent relues par les passes de validation.

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C54**, **C55** — le rejeu comme défense ; ce qui reste de l'ordre des verrous (C55 révisé par
  **C66** : la ligne des réglages ne « sérialise » pas).
- **C56** — deux enveloppes partagées, le registre (dérogation de découpage révisée par **C61**).
- **C57** — #536 fermée ici (l'avoir), la validation l'étant par la 15-5e1 (`closes` porté ici depuis
  C61 ; constat en C69).
- **C58** — seconde colonne du registre d'audit.
- **C59** / **C62** — journalisation `warn`, nom d'opération en champ de l'événement.
- **C60** — #484 fermée ici.
- **C61** — le découpage 15-5e1 / 15-5e2.
- **C63** — statut transitoire `ARejouer`, retiré ici ; volet (c) robuste, `retry_with` restreint.
- **C64** — ligne du CHANGELOG étendue ici ; PDF de la brochure non commité.
- **C65** — ordre de merge libre avec la 15-5d (révisé par **C66** et **C68**).
- **C66** — la saisie fournisseur est rejouée par la 15-5e1 (avec l'avance de ses réglages) ; elle
  sort du périmètre de cette story (17 routes au lieu de 18).
- **C68** — le Pattern 5 renvoie aux doc-comments canoniques au lieu de recopier l'ordre : l'ordre de
  merge 15-5d / 15-5e2 est sûr pour la documentation.
- **C69** — remédiations P4 de moindre portée, dont l'attribution ici des mentions de `retry_with`
  d'`opening_complement.rs`.
- **C70** — remédiation de la P5 de la 15-5e1 (volet (c) analysé par `syn`, réglages de facturation
  rejoués) et LOW de la P5 de celle-ci ; ligne « Corrigé » du CHANGELOG étendue aux réglages.

### References

- Issues : #536, #484 (fermées ici) ; #463, #491 (fermées par la 15-5e1) ; #481 (le passage
  `user-manual.tex:1536`, 15-5c).
- Fiches : `15-5e1-socle-rejeu.md` (le patron, l'inventaire, la règle), `15-5e-ordre-des-verrous-reglements.md`
  (index, historique des validations P1–P3), `15-5c-rapprochement-libelles-et-manuel.md` (AC6 :
  `user-manual.tex:1536`).
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
- 2026-10-08 — **Validation P4** (Opus ×2, contexte frais, lecture seule ; lentilles R et F ; prompt
  `15-5e2-validate-prompt-p4.md` ; rapports `target/gate-logs/15-5e2-p4-R.md` et `-F.md`).
  **Findings** : 0 CRITICAL, 0 HIGH, **5 MEDIUM** (R4-1 = F4-3, F4-1, F4-2, F4-4 — quatre défauts
  distincts), **15 LOW** (R4-2 à R4-9, F4-5 à F4-11). Numéros de ligne **recalés sur `f289414e`**
  (merge des 15-5a/15-5b), `grep -nF` / `sed -n` à l'appui. **Remédiations** :
  - **R4-1 = F4-3 MEDIUM** (choix **C68**) : les lignes `POST /supplier-invoices` et
    `invoices::validate_invoice` du Pattern 5 **renvoient aux doc-comments canoniques** au lieu de
    recopier l'ordre ; vérifié que l'AC9 de la 15-5d met déjà ces doc-comments à jour (elle le dit
    désormais, Change Log de la 15-5d) ; l'ordre de merge 15-5d / 15-5e2 est sûr dans les deux sens.
    **C65** révisé.
  - **F4-1 MEDIUM** : AC3 — les flux qui prennent l'exercice avant la sentinelle et les projets sont
    quatre (`accept_one_rule`, `accept_one_split`, `post_manual`, `post_split`), la clause
    « l'inventaire et T0 cherchent les autres » est rétablie ; la ligne du lot au Pattern 5 dit
    l'ordre du lot, entre propositions compris (cycle de #536).
  - **F4-2 MEDIUM** : la phrase de l'étape 0-bis (« inversion ABBA ») est **retirée**, non
    reformulée — la lecture de `books_locked_through` ne verrouille rien.
  - **F4-4 MEDIUM** : « Required » (`:353`) et « When to Use » (`:355-362`) du Pattern 5 réécrits sur
    la règle de la 15-5e1 (la transaction entière est annulée ; le rejeu est sûr si toute écriture vit
    dans la transaction ; l'enveloppe vaut pour toute route qui écrit au journal).
  - **Choix C66** (P4 de la 15-5e1) : la saisie fournisseur passe à la 15-5e1 — **17** routes ici
    (13 à rejouer : 10 `DbError`, 3 `AppError` ; 3 migrées ; `post_accept`), 21 − 4 = 17.
  - **LOW** : R4-2 = F4-6 (mentions de `retry_with` d'`opening_complement.rs:36`, `:405`, attribuées
    ici, C69 ; contrôle `grep` après migration) ; R4-3 (`reconciliation_e2e.rs:4411-4414`) ; R4-4 et
    F4-11 (lignes et chemins précisés ; `retry_with` recopié « sur une autre route `Rejouee` ») ; R4-5
    (règle du 1213 portée sur l'effet ; `confirm` retiré des corps à cloner) ; R4-6 = F4-7 (le registre
    est hors de « sans modification de leurs assertions », AC2 et AC6) ; R4-7 (C65 aux listes de
    choix et à l'index) ; R4-8 (`make admin user`) ; R4-9 (texte final complet de la ligne du
    CHANGELOG) ; F4-5 (inventaire **insensible à la casse**, recompté sur `f289414e` : **205 lignes /
    42 fichiers** — 169 / 32 sous `src` et `docs`, 36 / 10 sous `tests` —, tri bloc par bloc,
    `onboarding.rs:592-595` nommé, doc-comment de `find_open_covering_date` remis à sa place) ; F4-8
    (angle mort de la famille `AppError` écrit, phrase des Dev Notes corrigée) ; F4-9 (lentille
    adversariale de revue de code pour les trois extractions) ; F4-10 (« relance automatiquement »
    au manuel d'administration, « rejeu » y désignant déjà les reprises de l'import). Aucun LOW
    écarté.
  ⚠️ **Signal de découpage (amendement D5) déclaré au Project Lead** : la P4 rend **4 MEDIUM
  distincts** (5 entrées, R4-1 = F4-3), soit une sévérité égale à celle de la passe précédente.
  Constat : ces défauts sont **d'origine** — ordre des flux mal énuméré, clause perdue au découpage,
  phrase sur une lecture non verrouillante, Pattern 5 à recopie fragile, critère de sûreté du rejeu
  hérité — et **aucun n'est né d'une remédiation** ni ne recycle un défaut antérieur : **pas de
  découpage**, la revue trouve des défauts neufs, elle ne tourne pas en rond.
  **Propagation post-patch** : `18 autres`, `14 routes`, `14 à rejouer`, `21 − 3`, `tests 2 à 4`,
  `tests 1 à 4`, `make fr`, `deux flux prennent`, « `map_err` hors », `réduit la fréquence` (étape
  0-bis), `181`, et les anciens numéros (`850`, `3843`, `3858`, `2411`, `2379`, `3124`, `3136`,
  `:591`, `:905`, `:1532`, `:397`, `:313`) grepés sur cette fiche, la 15-5e1, la 15-5d, l'index et la
  fiche mère (hors Change Log) : restent la mention historique « la forme sensible à la casse en
  rendait 181 » et la phrase « `make fr` ne reconstruit la brochure… » qui explique le changement.
  **Décompte** (recompté sur la fiche) : **6 AC, 6 tâches T0–T5** ; routes 13 + 3 + `post_accept` =
  17 = 21 − 4 ; relevé des manuels 18 lignes (identique sur `f289414e`).

- 2026-10-08 — **Validation P5** (Sonnet ×2, contexte frais, lecture seule ; lentilles R et F ;
  prompt `15-5e2-validate-prompt-p5.md` ; rapports `target/gate-logs/15-5e2-p5-R.md` et `-F.md`).
  **Findings** : 0 CRITICAL, 0 HIGH, **0 MEDIUM**, **9 LOW** (R5-1 à R5-3, F5-1 à F5-6 ; R5-3 et
  F5-3 se recoupent). **Remédiations** (choix **C70**), toutes appliquées, aucune écartée :
  R5-1 (l'ordre de merge 15-5d / 15-5e2 n'est sûr que pour le code et les doc-comments : les PDF et
  le CHANGELOG entrent en conflit — relocaliser les passages des manuels par leur texte et
  régénérer les deux PDF après rebase ; en-tête, AC5, T0) ; R5-2 (`opening_balances.rs:578`
  « Cycles », doc-comment de `complete_opening_balances` `:573-578`) ; R5-3 = F5-3 (le Pattern 5
  nomme fonctions et étapes, jamais un numéro de ligne) ; F5-1 (`complete_import` ne prend aucun
  verrou nommé : angle mort de l'AC2 et Dev Notes réécrits — verrou du `staging`, `match` local) ;
  F5-2 (les trois fonctions « une tentative » reçoivent le corps par référence, aucun `Clone` à
  dériver) ; F5-4 (`innodb_deadlock_detect` **vérifiée** sur `10.11.16-MariaDB` — `ON` —, la
  version que le manuel exige ; consigne à l'indicatif) ; F5-5 (lignes du commentaire `99-kesh.cnf`
  à 70 caractères, contrôle des `Overfull` du `.log`) ; F5-6 (« trois tentatives » aussi à
  `api-external.md:307`). Alignements venus de la P5 de la 15-5e1 (C70) : ligne « Corrigé » du
  CHANGELOG étendue à l'enregistrement des réglages de facturation, mutation 2 de l'AC2 relue avec
  le volet (c) analysé par `syn`.
  **Boucle de validation close** : plus aucun finding au-dessus de LOW. **Trend** (15-5e jusqu'au
  découpage, puis cette fiche) : P1 **1 HIGH, 6 MEDIUM**, 6 LOW → réécriture C54 → P2 **6 MEDIUM**,
  12 LOW → P3 **2 MEDIUM**, 14 LOW → découpage C61 → P4 **5 MEDIUM** (4 distincts, d'origine),
  15 LOW → P5 **0 au-dessus de LOW**, 9 LOW. **Modèles** : Sonnet (P1), Opus (P2), Sonnet (P3),
  Opus (P4), Sonnet (P5), deux lentilles (R « regression hunter », F « full-scope adversary ») par
  passe, contexte frais. **Reclassements** : aucun (aucune dette technique ouverte). Signal D5 de
  la P4 déclaré, sans découpage (défauts d'origine). La fiche reste `ready-for-dev` ; son
  développement attend le merge de la 15-5e1.
  **Propagation post-patch** : `opening_balances.rs:575`, `plusieurs tentatives`, `verrou nommé`,
  `précautions 1 et 2`, `(#463, #491, #536)` et la phrase « ordre de merge est sûr » grepés sur
  cette fiche, la 15-5e1, la 15-5d, l'index et la fiche mère : restent les mentions historiques du
  Change Log, le texte final du CHANGELOG (qui garde « #536 », fermée ici) et la phrase de l'en-tête,
  désormais bornée au code et aux doc-comments. **Décompte** (recompté sur la fiche) : **6 AC,
  6 tâches T0–T5** ; routes 13 + 3 + `post_accept` = 17 = 21 − 4 ; partition finale 21 / 4 / 90 =
  115 (inchangée : la route des réglages, rejouée par la 15-5e1, reste `SansEcritureAuJournal`).
- 2026-10-08 — **Propagation de la validation P6 de la 15-5e1** (choix **C74** ; rapports
  `target/gate-logs/15-5e1-p6-{R,F}.md`), sans passe sur cette fiche : AC2 et « Pourquoi la revue
  fichier par fichier suffit » disent « les tests 2 à 5 **et 7** de la 15-5e1, **cinq** routes de la
  famille `DbError`, chacun avec un témoin du rejeu » (finding R6-2) ; « Ce que vit l'utilisateur »,
  « cinq routes rejouées par la 15-5e1 » ; `routes/onboarding.rs:614` (R6-4) ; le texte final du
  CHANGELOG prend la forme de la section — titre en gras, issues en liens (F6-4). Aucune règle ni
  aucun périmètre de cette fiche ne change ; **décompte inchangé** : 6 AC, 6 tâches T0–T5.
