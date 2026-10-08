# Story 15.5e2 : Rejeu sur interblocage des autres flux d'écriture — rollout, commentaires d'ordre, manuels

Status: done

<!-- Sous-story de la 15-5e (fiche index `15-5e-ordre-des-verrous-reglements.md`, statut `split`),
     créée le 2026-10-08 par le découpage décidé à la validation P3 de la 15-5e (finding F3-2, choix
     C61). C'est le ROLLOUT du patron que pose la 15-5e1 (`15-5e1-socle-rejeu.md`) : application
     mécanique des deux enveloppes aux autres routes qui écrivent au journal, revue fichier par
     fichier ; plus les commentaires d'ordre, le Pattern 5, les manuels (#484), le CHANGELOG et
     `docs/api-external.md`. Le contenu vient de la 15-5e validée P1 à P3 (commit `94f1365e` pour la
     version d'avant découpage) ; les remédiations de la P3 sont appliquées ici et dans la 15-5e1.
     Choix applicables : C54, C55 (révisé par C66), C56 (révisé par C61, C63, C86), C57 (révisé par C61,
     cf. C69), C58, C59 (révisé par C62), C60, C61, C62, C63, C64, C65 (révisé par C66 et C68), C66,
     C68, C69, C70, C85, C86.
     Statut `ready-for-dev` : la 15-5e1 est mergée (PR #559, `de1e1c26`). La boucle de validation,
     close en P5, a été **rouverte** par l'alignement C85 : la P6 (Opus ×2) rend 1 MEDIUM, remédié
     (C86) ; une passe P7 sur cette remédiation précède le développement. **Numéros de ligne relevés sur
     `cecd5d1d`** (branche de planification qui intègre `origin/main` `de1e1c26` : 15-5c, 15-5e1 et
     15-8a mergées), par leur texte, à l'alignement sur le livré (C85) — sauf mention contraire. La
     15-8b (`DELETE /journal-entries/{id}`, worktree `kesh-15-8`) n'est pas mergée : si elle l'est
     au T0, les numéros de `routes/journal_entries.rs`, `repositories/journal_entries.rs`, du
     Pattern 5, d'`api-external.md` et des deux manuels se relocalisent par leur texte. -->

**Issues** : la PR porte **`closes #536`** et **`closes #484`** (mots-clés **dans la PR** : le dépôt
merge en squash), et **`refs #463`**, **`refs #491`** (fermées par la 15-5e1, mergée), **`refs #555`**
(R3-7, mesurée vraie par la 15-5e1 : l'`INSERT IGNORE` en doublon de `get_or_create_default_in_tx`
pose un verrou partagé — hors de cette story, citée parce que le Pattern 5 et les commentaires
d'ordre la mentionnent).
- **#536** — interblocage entre l'acceptation d'un lot de rapprochement et la validation d'une
  facture : la validation est rejouée par la 15-5e1 ; la seconde inversion que l'issue cite —
  l'**avoir** d'une facture arrondie, exercice puis arrondi — est couverte ici, la création d'avoir
  étant rejouée (AC1). Le rejeu couvre alors l'issue entière (choix C57).
- **#484** — les manuels affirment que Kesh protège la numérotation et les « opérations critiques »
  par des transactions `SERIALIZABLE` ; il n'y en a aucune (`grep -rn SERIALIZABLE crates/*/src` :
  vide ; isolation `REPEATABLE READ` par défaut, `crates/kesh-db/src/pool.rs:17`). Les deux passages
  sont réécrits sur le mécanisme réel (AC5, choix C60).

**Dépend de la 15-5e1 — satisfaite** (mergée, PR #559, `de1e1c26`). Ce qu'elle a livré et que cette
story suppose, relu sur `cecd5d1d` :
- les enveloppes **`kesh_db::retry::retry_on_deadlock(operation, f)`** et
  **`retry_on_deadlock_with(operation, max_attempts, f)`** (famille `DbError`, prédicat
  `is_deadlock_error`), **`kesh_api::retry::retry_app_on_deadlock(operation, f)`** et le prédicat
  **`is_app_deadlock(&AppError)`** (famille `AppError`) ; la primitive
  **`retry_with(operation, max_attempts, should_retry, f)`** (`crates/kesh-db/src/retry.rs:196`), qui
  émet un `warn!` (cible `kesh_db::retry`, champs `operation`, `attempt`, `max_attempts`,
  `backoff_ms`) avant chaque nouvelle tentative et un `error!` (`operation`, `attempts`) à
  l'épuisement ; `DEFAULT_MAX_DEADLOCK_ATTEMPTS = 3` (`:71`) ;
- le registre `audit_route_registry.rs` à **deux colonnes**, le statut transitoire `ARejouer`, les
  volets (c) (analyse `syn` du corps du handler) et (d) (partition), les limites (i) à (vi) et
  (iii bis) de son doc-comment ; partition sur `cecd5d1d` : **9 `Rejouee` / 13 `ARejouer` /
  4 `Exemptee` / 89 `SansEcritureAuJournal` = 115** (recomptée sur le fichier) ;
- cinq routes rejouées par les enveloppes (`invoices::validate`, `invoices::settle`,
  `invoices::cancel_settlement`, `supplier_invoices::create`, `company_invoice_settings::update`) ;
  les noms d'opération posés sur les sites `retry_with` existants, dont le `PUT` des écritures de la
  15-8a (`"journal_entries::update"`, classé `Rejouee` à l'intégration, choix C-15-5e1-5) ;
- les doc-comments canoniques de `validate_invoice` et de `supplier_invoices::create_in_tx`
  (étiquettes `(0)`, `(1)`, `(2)`, `(2 bis)`, `(2, suite)`, `(3)`, `(4)`, `(5)` côté achat), « 5 bis »,
  le module `retry.rs` ; le témoin du rejeu `crates/kesh-api/tests/common/capture_rejeu.rs` ;
- **deux passages du Pattern 5 déjà corrigés** (revue de code P1 de la 15-5e1, B-2 = E-1) : le
  paragraphe « Why Lock Ordering Matters » (`MULTI-TENANT-SCOPING-PATTERNS.md:291`) et l'exemple
  « How to use the retry helper » (`:338-363`). Ils **sortent** du périmètre de l'AC3 ;
- **laissés à cette story** par la 15-5e1, et écrits ici : `routes/onboarding.rs:595` (« to avoid
  cross-table deadlocks ») et « Used on `finalize` » du Pattern 5 (`:328`) ; la migration des
  **prédicats d'interblocage écrits en ligne** (finding B-3 de sa revue de code : `onboarding.rs`,
  `reconciliation.rs` ×2, `invoices.rs`, `opening_balances.rs` — et, apparu depuis, le `PUT` de la
  15-8a) ; et les **trois documents qu'elle laisse faux** entre les deux merges (son AC6) :
  `docs/api-external.md` (§ 10, AC4 ici), `user-manual.tex` (« transaction DB `SERIALIZABLE` »,
  AC5) et `admin-manual.tex` (commentaire de `99-kesh.cnf`, AC5).

**Et des 15-8a (mergée) et 15-8b (non mergée)**, qui touchent les mêmes fichiers. La 15-8a a rouvert
le `PUT /journal-entries/{id}`, rejoué par un `retry_with` direct (C-15-8-19), et ajouté au Pattern 5
une rubrique « Deny list » qui le décrit. La **15-8b** rouvre le `DELETE /journal-entries/{id}`,
**aujourd'hui `ARejouer` au registre et dans le périmètre de cette story** : elle le rejoue elle-même
par un `retry_with` direct (`"journal_entries::delete"`, par uniformité avec le `PUT`, choix
C-15-8b-6), le passe `Rejouee`, ajoute une seconde ligne à la « Deny list » et une phrase de rejeu à
`api-external.md`. **L'ordre de merge 15-8b / 15-5e2 est libre**, et la partition **d'arrivée est la
même dans les deux ordres** (choix C85) :

| état de départ | `Rejouee` | `ARejouer` | `Exemptee` | `SansEcriture…` | ce que fait cette story |
|---|---|---|---|---|---|
| 15-8b non mergée (`cecd5d1d`) | 9 | 13 | 4 | 89 | rejoue 13 routes, dont le `DELETE` |
| 15-8b mergée avant (mesuré sur son worktree en cours de rebase) | 10 | 12 | 4 | 89 | rejoue 12 routes ; migre le `retry_with` du `DELETE` |
| **après cette story** | **22** | **0** | **4** | **89** | total **115** |

Si la 15-8b merge **après** cette story, c'est elle qui rebase : son `DELETE`, déjà rejoué ici par
l'enveloppe, garde l'enveloppe (son `retry_with` ne revient pas — le volet (c) le refuserait) ; son
assertion de partition se recale sur 22 / 0 / 4 / 89 ; et ce qu'elle réintroduirait **en texte**
— rien ne le garde, le contrôle `grep` de l'AC1 ayant déjà tourné (finding F6-10 de la P6) — se
range à la forme d'arrivée de cette story : la ligne du `DELETE` de sa « Deny list » va dans
« Where This Applies » et sa note sous la table (AC3), et les mentions de `retry_with` de son
doc-comment de route (« `retry_with`, C-15-8-19 ») et de `delete_in_tx` nomment l'enveloppe
(AC1). La consigne est à porter aussi dans la fiche de la 15-8b (hors de cette fiche :
l'orchestrateur la propage). Conflits textuels attendus dans les deux
ordres : `routes/journal_entries.rs` (handler du `DELETE`), `audit_route_registry.rs` (statut et
assertions), Pattern 5 (« Deny list »), `api-external.md`, `CHANGELOG.md`, les deux manuels et leurs
PDF — les PDF se **régénèrent** après rebase (AC5).

**Ordre libre par rapport à la 15-5d** (choix C65, révisé par C68) :
- pour le **code**, la 15-5d n'ajoute aucune route et ne change aucun handler ; si elle merge avant
  cette story, la complétion d'import porte ses verrous sans rejeu jusqu'à ce merge (la saisie
  fournisseur, elle, est rejouée par la 15-5e1) ;
- pour la **documentation de l'ordre des verrous**, l'ordre de merge est sûr dans les deux sens
  parce que le Pattern 5 **ne recopie plus** l'ordre des deux flux que la 15-5d modifie : ses lignes
  `POST /supplier-invoices` et `invoices::validate_invoice` **renvoient aux doc-comments
  canoniques** (AC3, choix C68), seuls lieux de vérité, que la 15-5d met elle-même à jour quand elle
  y place ses verrous (son AC9) ;
- **mais pas pour les fichiers que les deux stories touchent** (finding R5-1 de la P5) : la 15-5d
  régénère et commite elle aussi `user-manual.pdf` et `admin-manual.pdf` (son AC8, sa T6) et
  modifie `user-manual.tex` avant le passage `SERIALIZABLE` (`:969` sur `cecd5d1d`) ; elle ajoute
  aussi une phrase au commentaire « 5 bis » (`invoice_settlements_write.rs:474-486`) dont cette
  story remplace la mention de `retry_with` (AC1) — conflit textuel, résolu en gardant les deux
  changements ; elle modifie la table du § 10 de `docs/api-external.md` (ligne
  `ACCOUNT_NOT_POSTABLE`, son AC9), où cette story ajoute ses phrases **sous** la table, hors de
  ses lignes (AC4) — conflit textuel possible, résolu en gardant les deux (finding R6-6 de la P6) ;
  la seconde des deux à merger aura un **conflit
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

1. **AC1 — Toutes les routes qui écrivent au journal passent par une enveloppe, et `retry_with` ne
   reste direct qu'à `post_accept`** (inventaire de la 15-5e1, colonne « fiche » = 15-5e2 ; choix
   C66, C85). Sur `cecd5d1d`, le registre porte **13 routes `ARejouer`** et les routes portent **six
   sites `retry_with` directs** ; la 15-8b, si elle merge avant, en déplace une d'une colonne à
   l'autre (en-tête, tableau des partitions) :
   - **les routes `ARejouer` à rejouer**, chacune par l'enveloppe que la table de la 15-5e1 lui
     assigne, avec un nom d'opération `"<module>::<action>"` :
     - famille **`DbError`** — `kesh_db::retry::retry_on_deadlock(operation, f)` — : dévalidation
       d'une facture ; création d'un avoir ; règlement fournisseur, annulation d'une facture
       fournisseur et de son règlement ; confirmation d'un lot de paiement ; écriture manuelle
       (création, contre-passation **et suppression** — `DELETE /journal-entries/{id}`, nom
       `"journal_entries::delete"`) ; bilan d'ouverture (`map_opening_balances_error` **après**) —
       **10** routes sur `cecd5d1d`, **9** si la 15-8b a mergé avant (le `DELETE` est alors un site
       à migrer, ci-dessous) ;
     - famille **`AppError`** — `kesh_api::retry::retry_app_on_deadlock(operation, f)` — :
       complétion d'une facture importée, rapprochement manuel et ventilé, partie transactionnelle
       extraite en fonction « une tentative », patron `accept_once` (`routes/reconciliation.rs:918`)
       et `cancel_reconciliation_once` (`:3977`), qui ouvre la transaction, prend le verrou nommé du
       compte bancaire le cas échéant, écrit et conclut — **3** routes, de noms d'opération
       **`"imported_supplier_invoices::complete"`**, **`"reconciliation::manual"`** et
       **`"reconciliation::split"`** (fixés ici : le test 8 de l'AC2 exige le second par son
       témoin) ;
     soit **13** routes (**12** si la 15-8b a mergé avant).
   - **Les sites `retry_with` directs des routes** — relevé `grep -rn "retry_with(" crates/kesh-api/src/routes`
     sur `cecd5d1d` : **six**, plus le `DELETE` de la 15-8b quand elle a mergé (**sept**) — dont
     **cinq migrent** vers une enveloppe (**six** avec le `DELETE`), `post_accept` gardant
     `retry_with` (finding R6-5 de la P6). Chacun garde son nom d'opération ; son sort, et la
     raison :

     | site (`crates/kesh-api/src/routes/…`) | nom | aujourd'hui | après cette story |
     |---|---|---|---|
     | `write_off_invoice_handler` (`invoices.rs:1353`) | `invoices::write_off` | `retry_with`, `DEFAULT_MAX_DEADLOCK_ATTEMPTS`, `\|err: &DbError\| is_deadlock_error(err)` | `retry_on_deadlock` |
     | `complete_opening_balances` (`opening_balances.rs:594`) | `opening_balances::complete` | idem | `retry_on_deadlock` |
     | `update_journal_entry` (`journal_entries.rs:675`, 15-8a) | `journal_entries::update` | idem | `retry_on_deadlock` |
     | `delete_journal_entry` (15-8b, si mergée) | `journal_entries::delete` | idem | `retry_on_deadlock` |
     | `post_cancel_reconciliation` (`reconciliation.rs:3961`) | `reconciliation::cancel` | `retry_with`, prédicat `matches!(err, AppError::Database(db) if is_deadlock_error(db))` écrit en ligne | `retry_app_on_deadlock` |
     | `onboarding::finalize` (`onboarding.rs:614`) | `onboarding::finalize` | idem (prédicat en ligne) | `retry_app_on_deadlock` |
     | `post_accept` (`reconciliation.rs:888`) | `reconciliation::accept` | prédicat en ligne **élargi au 1305** (`ReconciliationTransactionAborted`) | **reste `retry_with`** ; son prédicat devient `is_app_deadlock(err) \|\| matches!(err, AppError::ReconciliationTransactionAborted)` |

     Chaque migration est une **équivalence exacte** — même nombre de tentatives (les enveloppes
     passent `DEFAULT_MAX_DEADLOCK_ATTEMPTS`), même prédicat (`retry_on_deadlock` porte
     `is_deadlock_error`, `retry_app_on_deadlock` porte `is_app_deadlock`, que le prédicat écrit en
     ligne recopie à la lettre), même nom d'opération —, vérifiée à la lecture en T0 et consignée
     au Dev Agent Record. C'est la migration des **prédicats en ligne** que la revue de code de la
     15-5e1 a laissée ici (finding B-3) : après elle, **aucun prédicat d'interblocage n'est écrit en
     ligne** dans `crates/kesh-api/src` hors du `|| matches!(…TransactionAborted)` propre à
     `post_accept`. Les deux commentaires de `post_accept` qui nomment `is_deadlock_error`
     (`routes/reconciliation.rs:873`, « que `is_deadlock_error` reconnaît », et `:880`,
     « `is_deadlock_error` reste 1213 seul ») nomment désormais **`is_app_deadlock`**, le prédicat
     qu'ils décrivent après la migration (findings R6-2 = F6-2 de la P6) ; contrôle :
     `grep -rn "is_deadlock_error" crates/kesh-api/src` ne rend plus que `retry.rs` ;
     `grep -rn "retry_with(" crates/kesh-api/src/routes` ne rend plus que `post_accept`.
     `onboarding::finalize` est `SansEcritureAuJournal` au registre : le volet (c) ne l'examine pas
     — c'est le **volet (c bis)** ci-dessous qui garde sa migration dans la durée (finding F6-5 de
     la P6 ; choix C85 : la laisser en `retry_with` aurait gardé un prédicat en ligne pour un gain
     nul).
   - **`post_accept` reste le seul `retry_with` direct** : son prédicat élargi au 1305 est propre à la
     route (un 1305 ailleurs dans le crate n'a pas ce sens, commentaire `reconciliation.rs:880-881`).
   - **Les règles de la 15-5e1 s'appliquent route par route**, vérifiées en T0 et en revue fichier par
     fichier : une tentative = une transaction neuve ; les contrôles qui ne dépendent pas de la
     transaction restent hors de la fermeture, **ceux qui lisent ce que la transaction verrouille
     restent dans la tentative, dans le même ordre** (finding R3-4 : dans `complete_import`
     (`routes/imported_supplier_invoices.rs:123-292`), les contrôles de devise, QR-IBAN et montant
     lisent le `staging` pris `FOR UPDATE` et restent dans `complete_import_once` après lui ; dans
     `cancel_reconciliation_once`, la lecture de `bank_account_id`
     (`routes/reconciliation.rs:3984-3994`), déjà dans la tentative, y reste) ; les entrées consommées
     sont **clonées par tentative** (finding F3-8 : `create_credit_note` et `create_journal_entry`
     consomment `Json(req)` pour construire un `NewCreditNote` / `NewJournalEntry`, tous deux
     `Clone` — patron `onboarding::finalize`, pas de `Option::take` ; de même
     `generate_opening_balances`, dont `create_opening_entry` consomme le `NewJournalEntry` `new`
     (`routes/opening_balances.rs:468-473`, finding F6-8 de la P6) ; la liste n'est pas close, la
     règle vaut pour toute entrée consommée et T0 la relève route par route ; `confirm_payment_batch` ne lit
     que `req.payment_date`, `Copy`, rien à cloner, finding R4-5 ; le `PUT` clone déjà son
     `NewJournalEntry` par tentative, le `DELETE` ne prend que des identifiants) ; les trois fonctions
     « une tentative » (`complete_import_once`, et celles de `post_manual` et `post_split`) reçoivent
     le corps **par référence** (`&CompleteImportRequest`, `&ManualMatchBody`, `&SplitBody`) : aucun
     de ces trois types ne dérive `Clone` (`routes/imported_supplier_invoices.rs:109-117`, et
     `CompleteImportLineRequest` `:99-107` ; `routes/reconciliation.rs:2982`, `:3424`), et il n'y a
     pas à le dériver — T0 le vérifie (finding F5-2 de la P5) ; aucun effet de bord hors
     transaction ; le 1213 atteint le prédicat — **aucune conversion qui fasse sortir une erreur sqlx
     de `DbError::Sqlx` avant le prédicat** (règle de l'AC2 de la 15-5e1, portée sur l'effet et non
     sur la forme : un `map_err(|e| …DbError::Sqlx(e))` écrit à la main, comme
     `routes/reconciliation.rs:3993`, la respecte — finding R4-5).
   - **Le registre** (`audit_route_registry.rs`) : les statuts `ARejouer` restants deviennent
     `Rejouee` et la variante **`ARejouer` est retirée** de l'énumération ; la partition (volet (d),
     `the_replay_partition_is_what_the_story_declares`) devient **22** `Rejouee`, **4** `Exemptee`,
     **89** `SansEcritureAuJournal`, total **115** — dans les deux ordres de merge avec la 15-8b
     (en-tête) ; le volet (c) (`every_replayed_route_calls_an_envelope`) examine **22** routes
     (`assert_eq!(examinees, …)`, 9 sur `cecd5d1d`) et **n'accepte plus `retry_with` que pour
     `post_accept`** — un `retry_with` recopié sur une autre route `Rejouee` le fait rougir. **Le
     mécanisme est fixé** (finding F6-4 de la P6) : `retry_with` sort de la liste `ENVELOPPES`
     (`audit_route_registry.rs:602-607`), qui ne garde que `retry_on_deadlock`,
     `retry_on_deadlock_with` et `retry_app_on_deadlock` ; une seconde liste nommée,
     `RETRY_WITH_AUTORISE = &["post_accept"]`, fait accepter au volet (c) un appel `retry_with` dans
     le corps de ces seuls handlers. Le banc du visiteur
     (`the_envelope_visitor_sees_calls_and_only_calls`, `:725-822`) suit : ses quatre cas positifs
     `qualifie`, `turbofish`, `dans_un_bloc` et `aide`, écrits aujourd'hui sur `retry_with`
     (`:745-778`), sont **transposés** sur `retry_on_deadlock` ou `retry_app_on_deadlock` — sans
     quoi ils basculeraient et le banc ne couvrirait plus ces formes pour une vraie enveloppe — et il
     gagne un cas **négatif** : un `retry_with` seul n'est pas une enveloppe pour un handler hors de
     `RETRY_WITH_AUTORISE`.
   - **Le volet (c bis), garde durable des migrations** (finding F6-5 de la P6) : le volet (c)
     n'examine que les routes `Rejouee` (finding F4-11), si bien qu'une route
     `SansEcritureAuJournal` — `onboarding::finalize` aujourd'hui — pourrait réintroduire un
     `retry_with` à prédicat en ligne sans que rien ne rougisse. Un test neuf du registre,
     `no_route_calls_retry_with_except_post_accept`, parse avec `syn` (même outillage que le
     volet (c)) **chaque fichier** de `crates/kesh-api/src/routes/` et échoue, en nommant le
     fichier et la fonction, sur tout appel `retry_with` (`ExprCall` dont le dernier segment est
     `retry_with`, ou `ExprMethodCall` de ce nom) hors du corps d'une fonction de
     `RETRY_WITH_AUTORISE`, **quel que soit le statut de la route** ; il échoue aussi si
     `post_accept` n'appelle plus `retry_with` (la liste ne doit pas survivre à son objet).
   - Les messages des deux assertions et le doc-comment du registre sont réécrits sur ce décompte :
     la limite **(iii bis)** (quelles routes ont une preuve dynamique — tests 2 à 5, 7 **et 8**
     de `rejeu_interblocage_e2e.rs`, `accept_replays_the_batch_when_it_is_the_deadlock_victim`,
     `the_put_replays_a_deadlock_it_lost` de la 15-8a — et lesquelles la seule revue) et la
     limite **(vi)** (`audit_route_registry.rs:89`, « `onboarding::finalize` (`retry_with`) » :
     `finalize` passe à `retry_app_on_deadlock`, et la limite renvoie au volet (c bis) — finding
     R6-1 de la P6).
   - Chacun des doc-comments des routes rejouées ici et des sites migrés dit, en une ligne, que la
     route est rejouée sur interblocage, avec renvoi à l'enveloppe (pas de recopie de la règle) ; ceux
     qui nomment `retry_with` cessent de le nommer : `post_cancel_reconciliation`
     (`routes/reconciliation.rs:3947`), `complete_opening_balances` (`routes/opening_balances.rs:570-579`,
     la mention de `retry_with` étant `:576`, finding R5-2), `update_journal_entry`
     (`routes/journal_entries.rs:652`, « Rejoué sur interblocage (`retry_with`, C-15-8-19) », et
     `:653`, « (exception de Pattern 5) », qui perd son référent avec la rubrique « Deny list » et
     renvoie à la ligne du `PUT` dans « Where This Applies » — finding R6-3 de la P6), celui du
     `DELETE` si la 15-8b a mergé, et `onboarding::finalize` (`routes/onboarding.rs:597-601`,
     « la fonction est wrappée dans `retry_with` », et le doc-comment de `finalize_inner`, `:628-631`).
     **Les mentions de `retry_with` qui désignent l'appelant d'un site migré**, hors des doc-comments
     de route, nomment l'enveloppe (findings R4-2, F4-6 de la P4 ; trou antérieur au découpage,
     R4-10 de la 15-5e1, attribué ici ; liste complétée sur `cecd5d1d`, C85) :
     `crates/kesh-db/src/repositories/opening_complement.rs:36` (doc du module : « l'appelant
     enveloppe l'appel dans `retry_with` ») et `:405` (« l'enveloppe dans `retry_with` ») ;
     `crates/kesh-db/src/repositories/invoice_settlements_write.rs:485` (commentaire « 5 bis », réécrit
     par la 15-5e1 : « la route est rejouée (`write_off_invoice_handler`, `retry_with`) ») ;
     `crates/kesh-db/src/repositories/journal_entries.rs:1225-1228` (doc de `journal_entries::update`,
     15-8a : « que le handler **rejoue** (`retry_with`) » et « Le `PUT` figure à la liste des
     exceptions de Pattern 5 », rubrique que l'AC3 retire — la phrase renvoie à la ligne du `PUT`
     dans « Where This Applies ») et les mentions jumelles de `delete_in_tx` si la 15-8b a mergé ;
     dans le Pattern 5, les puces et lignes que l'AC3 réécrit. **Sous `crates/*/tests`** (finding
     R6-1 = F6-3 de la P6 : le contrôle ne les voyait pas), six mentions deviennent fausses et sont
     réécrites — numéros sur `cecd5d1d`, relocalisés par leur texte au T0 :
     `crates/kesh-api/tests/journal_entry_reversal_e2e.rs:1609` (« ⛔ **Tue** « retirer
     `retry_with` du handler » » : la mutation retire désormais l'enveloppe) ;
     `crates/kesh-db/tests/journal_entries_modification.rs:441` (« le handler `PUT` rejoue
     (`retry_with`) ») ; et dans `crates/kesh-api/tests/audit_route_registry.rs`, `:89` (limite
     **(vi)**, ci-dessus), `:119` (« ou, jusqu'à la 15-5e2, `retry_with` »), `:165` (« Rejouée par
     la 15-8a (`retry_with`, C-15-8-19) ») et `:600` (doc d'`ENVELOPPES`, « `retry_with` n'est
     accepté que jusqu'à la 15-5e2 »). Après la migration,
     `grep -rn "retry_with" crates/*/src crates/*/tests docs --include=*.rs --include=*.md` ne rend
     plus que : les deux `retry.rs` ; `post_accept` (code, commentaire `reconciliation.rs:868`, doc
     d'`accept_once` `:917`) ; la forme générique de l'exemple du Pattern 5 (`:355-362`, qui désigne
     `retry_with` comme la primitive des enveloppes) ; **sous `tests`**, `capture_rejeu.rs:6` (vrai :
     les enveloppes émettent par `retry_with`), `reconciliation_e2e.rs:4414` (historique, AC3), le
     registre où `retry_with` est l'objet du contrôle (`RETRY_WITH_AUTORISE`, volet (c bis) et leurs
     doc-comments) et les sources synthétiques du banc du visiteur (`:729-781` sur `cecd5d1d` :
     cas « nom en commentaire, chaîne, doc-comment, nom préfixé », et le cas négatif neuf), légitimes
     parce qu'ils **sont** la matière du test ; chaque autre occurrence est corrigée ou justifiée au
     Dev Agent Record. Le doc-comment de `write_off_invoice_handler`
     (`crates/kesh-api/src/routes/invoices.rs:1334-1337` ; la phrase sur l'ordre est `:1334-1336`,
     celle sur la `version` `:1336-1337` — finding R3-1 ; F1-3) : « l'ordre des verrous est celui du
     règlement manuel (facture → compte → exercice), qui peut former un cycle avec l'acceptation d'un
     rapprochement (#491) » est réécrit : la route est rejouée comme toute route d'écriture au
     journal ; la phrase sur la `version` (« une tentative qui trouve un reste changé est refusée en
     409 ») est gardée.

### Ce qui doit être prouvé

2. **AC2 — Tests.** **Un seul test dynamique neuf**, le test « route victime » de `post_manual`
   (choix C56 **révisé par C86**, finding F6-1 de la P6) ; pour le reste, les tests 2 à 5 et 7 de
   la 15-5e1 prouvent le patron sur cinq routes de la famille `DbError` — quatre qui écrivent au
   journal et l'enregistrement des réglages, finding R6-2 de la P6 de la 15-5e1 —, chacun avec un
   témoin du rejeu (C74), et son test 1 le prédicat et l'enveloppe `AppError` ; les autres routes
   reposent sur la revue fichier par fichier et le registre — le doc-comment du registre le dit,
   finding F2-10.
   - **Test 8 — rapprochement manuel victime** (`crates/kesh-api/tests/rejeu_interblocage_e2e.rs`,
     à la suite du test 7, nom `manual_match_is_replayed_when_it_is_the_deadlock_victim`), écrit
     avec le harnais livré par la 15-5e1 — `monter`, `exercice_du_jour`, `transaction_lourde`,
     `requete_en_tache`, `victime` et `CaptureRejeu` (`tests/common/capture_rejeu.rs`) —, montage
     **avec projet** : `post_manual` prend l'exercice `FOR UPDATE` (étape 6,
     `find_open_covering_date`, `routes/reconciliation.rs:3223`) **puis** la sentinelle
     `companies` (étape 6bis, `validate_taggable_in_tx` `:3235` →
     `acquire_company_sentinel_lock`, `repositories/projects.rs:99`). (1) La transaction de test,
     **lourde**, tient la sentinelle (`SELECT id FROM companies WHERE id = ? FOR UPDATE`) ;
     (2) la route est lancée en tâche (`POST /api/v1/reconciliation/manual`, transaction bancaire
     en attente, compte de contrepartie, `projectId` d'un projet actif — données reprises du
     montage de `reconciliation_manual_e2e.rs:512-570`) et `victime` attend qu'elle bloque sur
     `["companies", "FOR UPDATE"]` ; (3) la transaction de test demande l'exercice
     (`SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE`) : le cycle se ferme, la route, plus
     légère, est la victime ; le verrou nommé du compte bancaire, hors du graphe d'InnoDB, est
     relâché par `with_account_lock` après l'erreur de la fermeture et repris à la tentative
     suivante. **Assertions** : `200` ; une seule écriture créée et la transaction bancaire
     rapprochée une seule fois (comptes avant / après) ; **témoin**
     `capture.exiger_un_rejeu("reconciliation::manual")` — le `warn!` de `kesh_db::retry` porte le
     champ `operation`, sans quoi le test passerait au vert sans rejeu. Il exerce le chemin propre à
     la famille `AppError` avec verrou nommé — 1213 levé sous le verrou nommé, converti par le
     `match` du handler, `RELEASE_LOCK`, `rollback`, nouvelle tentative —, que l'AC2 déclarait
     jusqu'ici non exercé. **T0 forme d'abord ce cycle à la main** sur MariaDB 10.11 (deux sessions
     `mariadb`, ordre ci-dessus, la seconde reçoit le 1213) et le consigne au Dev Agent Record. **Si
     le cycle ne se forme pas** (la route ne bloque pas sur la sentinelle, ou c'est la transaction
     de test qui est choisie victime malgré son lest), le test n'est pas écrit en vert de
     complaisance : il est **écrit comme angle mort** — la fiche, le doc-comment du registre (limite
     (iii bis)) et le Dev Agent Record disent pourquoi le cycle ne se forme pas, et `post_manual`
     rejoint les deux extractions ci-dessous.
   - **Angle mort restant** (finding F4-8, réduit par F6-1) : `post_split` et `complete_import`
     n'ont pas de preuve dynamique — le premier suit le chemin de `post_manual` (verrou nommé,
     `match`, `RELEASE_LOCK`, `rollback`), que le test 8 exerce sur la route sœur ; le second lève
     son 1213 sous le verrou du **`staging`**, pris `FOR UPDATE`
     (`routes/imported_supplier_invoices.rs:141` ; `match` local, `rollback`, nouvelle tentative —
     **aucun verrou nommé** sur cette route, finding F5-1 de la P5), et son risque propre est
     l'ordre des contrôles lus sur le `staging` (R3-4, AC1). Ces deux extractions, et celle de
     `post_manual`, reçoivent une lentille adversariale de la revue de code (Dev Notes, « Règle de
     découpage »). Pour les **sites migrés** (AC1), la migration est une équivalence exacte : deux
     en ont une preuve dynamique qui reste valable après elle — `the_put_replays_a_deadlock_it_lost`
     (`crates/kesh-api/tests/journal_entry_reversal_e2e.rs:1611`, 15-8a, sans témoin `tracing`)
     pour le `PUT`, `accept_replays_the_batch_when_it_is_the_deadlock_victim`
     (`reconciliation_e2e.rs:4426`) pour `post_accept` (dont seul le prédicat change de forme) ; les
     autres (`invoices::write_off`, `opening_balances::complete`, `reconciliation::cancel`,
     `onboarding::finalize`, le `DELETE` de la 15-8b) reposent sur la lecture et, pour les quatre
     `Rejouee`, sur le volet (c), pour tous sur le volet (c bis).
   - **Test neuf du registre** : `no_route_calls_retry_with_except_post_accept`, le volet (c bis)
     de l'AC1 ; le banc du visiteur est transposé (AC1, F6-4).
   - Les tests existants des routes touchées restent verts **sans modification de leurs assertions**,
     ainsi que `accept_replays_the_batch_when_it_is_the_deadlock_victim`,
     `the_put_replays_a_deadlock_it_lost` (15-8a), les tests de la 15-8b si elle a mergé, et les
     tests 1 à 5 et 7 de la 15-5e1 (`rejeu_interblocage_e2e.rs`, témoins compris : les noms
     d'opération ne changent pas) — **hors** le registre `audit_route_registry.rs`, dont l'AC1 change la partition, retire
     la variante `ARejouer`, restreint l'acceptation de `retry_with` et transpose le banc du
     visiteur (findings R4-6, F4-7, F6-4).
   - **Mutations** (consignées au Dev Agent Record, en touchant le fichier après restauration — sans
     quoi cargo garde le binaire muté), **une par famille** (finding F3-1) :
     - famille `DbError` à correspondance après le rejeu : retirer l'enveloppe de
       `create_journal_entry` → le volet (c) rougit ;
     - famille `DbError` dont le doc-comment de la route suivante nomme l'enveloppe : retirer
       l'enveloppe de `pay_supplier_invoice` (suivi du doc-comment de `cancel_supplier_invoice`,
       `routes/supplier_invoices.rs:400-404`) → le volet (c) rougit. Depuis que le volet (c)
       analyse le source avec `syn` (15-5e1, choix C70), un doc-comment est un attribut de l'item
       **qu'il précède**, hors du corps examiné : la mutation éprouve sur le source réel le cas
       qui motivait jadis les précautions textuelles (un doc-comment voisin qui nomme
       l'enveloppe) ;
     - famille `AppError` : retirer l'enveloppe de `post_manual` → le volet (c) rougit **et le
       test 8 rougit** (la 1213 remonte en 500) — c'est la mutation qui prouve que le test 8 voit
       le rejeu (si le test 8 est écrit comme angle mort, seul le volet (c) est attendu rouge, et
       c'est consigné) ;
     - remplacer l'enveloppe de `write_off_invoice_handler` par un `retry_with` recopié → le volet (c)
       rougit (acceptation de `retry_with` restreinte à `post_accept`) ; idem sur
       `update_journal_entry`, site migré venu de la 15-8a (C85) ;
     - remettre `onboarding::finalize` en `retry_with` à prédicat en ligne (route
       `SansEcritureAuJournal`, que le volet (c) n'examine pas) → le volet (c bis) rougit en nommant
       `onboarding.rs` et `finalize` (finding F6-5) ;
     - remettre à la main un `ARejouer` dans le registre → ne compile pas (variante retirée) :
       consigné, sans mutation à lancer.
   - **Aucun test de « non-interblocage »** (finding F1-4 de la P1 de la 15-5e).

### L'ordre des verrous : les commentaires et le Pattern 5 (choix C54, C55)

3. **AC3 — Les commentaires disent la règle vraie, et aucun ne prétend à l'absence de cycle.** Le
   doc-comment canonique de `validate_invoice`, celui de `supplier_invoices::create_in_tx`, « 5 bis »
   et le module `retry.rs` ont été réécrits par la 15-5e1 (mergée), ainsi que deux passages du
   Pattern 5 (en-tête) ; le reste est ici.
   - **Les commentaires qui fondent une absence de cycle sur « l'ordre global »** (finding F2-1) :
     - **l'étape 0** de `create_in_tx` (`crates/kesh-db/src/repositories/journal_entries.rs:261-265` :
       « AVANT le lock fiscal_years pour respecter l'ordre de verrouillage global companies →
       projects → fiscal_years […] créerait une inversion ABBA inter-flux ») et
       `create_opening_entry` (`:559-561` et `:573-574` : « pas d'inversion de l'ordre de verrou
       global », « respecte l'ordre de verrou global — pas d'inversion ABBA ») sont **réécrits** :
       prendre la sentinelle et les projets avant l'exercice **réduit la fréquence** des
       interblocages, il ne les exclut pas. L'étape 0 précède l'étape 1 (le re-verrou de l'exercice,
       `:303-306`), **pas l'exercice que l'appelant a souvent déjà pris** : elle ne vient avant
       l'exercice que pour `journal_entries::create` ; les flux qui verrouillent l'exercice avant
       d'appeler `create_in_tx` prennent l'ordre **inverse**, exercice **puis** sentinelle et
       projets (finding F4-1 de la P4) :
       - le lot de rapprochement par **règle** (`accept_one_rule`,
         `crates/kesh-api/src/routes/reconciliation.rs:2275` : exercice `:2462`, puis
         `validate_taggable_in_tx` `:2494` → sentinelle `crates/kesh-db/src/repositories/projects.rs:99`) ;
       - le lot de rapprochement par **ventilation** (`accept_one_split`, `routes/reconciliation.rs:1877` :
         exercice `:2109`, puis l'étape 0 de `create_in_tx` `:2151` sur les projets de chaque ligne,
         `:2128-2131`) ;
       - le rapprochement **manuel** à projet (`post_manual` : exercice `routes/reconciliation.rs:3223`,
         puis `validate_taggable_in_tx` `:3235`) ;
       - le rapprochement **ventilé** (`post_split` : exercice `routes/reconciliation.rs:3720`, puis
         l'étape 0 de `create_in_tx` `:3737` sur les projets des lignes) ;
       contre `journal_entries::create` (étape 0 puis étape 1), `create_opening_entry` (sentinelle
       puis exercice) et la saisie fournisseur à projet. Les deux côtés de chaque cycle sont rejoués
       (renvoi à l'enveloppe). **Cette liste n'est pas close** : l'inventaire au symptôme ci-dessous
       et T0 cherchent les autres (clause de la 15-5e d'avant découpage, perdue au découpage et
       rétablie — F4-1, preuve 3). **Le `PUT` des écritures (15-8a)**, qui verrouille l'écriture
       **avant** la sentinelle et les projets, puis l'exercice (« PREMIER ACTE »,
       `journal_entries.rs:1291` et suivantes, doc-comment « # Ordre des verrous » `:1194-1228`, qui nomme trois cycles hérités),
       est déjà écrit juste — ses cycles sont nommés et la route rejouée — ; seule sa mention de
       `retry_with` et de la « liste des exceptions de Pattern 5 » change (AC1) ; de même pour le
       `DELETE` si la 15-8b a mergé. L'inversion est écrite au commentaire de chacun
       (`routes/reconciliation.rs:2485-2491`, Step 11bis d'`accept_one_rule` ; `:2128-2131` pour
       `accept_one_split` ; et `:3228-3233`, Step 6bis de `post_manual`, qui affirme aujourd'hui
       l'inverse — « cohérent Pattern 5 (avant le lock fiscal_years pris par create_in_tx) », alors
       que l'exercice est déjà verrouillé à l'étape 6 —, réécrit ; le point d'appel de
       `create_in_tx` dans `post_split`, `:3737`, gagne la même phrase).
     - **l'étape 0-bis** (`journal_entries.rs:276-294`) : la puce « Verrous : `companies` vient EN
       PREMIER dans l'ordre global (…) Lire la borne après le lock `fiscal_years` de l'étape 1
       créerait une inversion ABBA » (`:282-285`) est **retirée** — non reformulée (finding F4-2) :
       la lecture de `books_locked_through` n'est **pas verrouillante** (`:295-301`, et le
       commentaire le dit lui-même, `:290-294`), elle ne peut donc participer à aucun interblocage.
       Sa place n'est dictée que par l'**ordre des refus** : la puce « Refus » (`:286-288`) et
       l'avertissement « Lecture NON verrouillante » sont gardés ; la phrase d'en-tête qui oppose
       l'ordre des verrous à celui des refus (`:278-280`) est ajustée en conséquence. Le renvoi que
       la 15-8a y fait (`journal_entries.rs:1682`, « pour la raison écrite à l'étape 0-bis ») vise
       l'avertissement gardé : **le renvoi** reste vrai — mais non son bloc : la suite
       (`:1683-1684`, « ici elle évite en plus de prendre un verrou sur `companies` APRÈS ceux de
       l'étape 2, ce qui inverserait l'ordre global ») fonde un « ordre global » que la story abolit
       et prête un verrou à une lecture qui n'en prend aucun ; elle se trie comme l'étape 0-bis :
       **retirée**, le renvoi et « Seuil INCLUSIF » gardés (finding R6-7 de la P6).
   - **`crates/kesh-db/src/repositories/fiscal_years.rs:499-502`** (finding F3-4) : « Toute
     divergence = risque de deadlock avec `journal_entries::create_in_tx` en cours sur la même
     company », jumelle de la phrase que la 15-5e1 a retirée du doc-comment de `validate_invoice` —
     **réécrite** : la place de l'exercice est une convention de fréquence, la défense est le rejeu.
     Le bloc qui la porte (`:489-502`) est le doc-comment de `find_open_covering_date` **égaré**
     au-dessus de `find_covering_date_in_tx` (`:516`), suivi sans séparation de la doc de celle-ci ;
     `find_open_covering_date` (`:543`) n'a plus de doc-comment. La réécriture le **remet à sa place**
     (finding F4-5).
   - **Gardés, vrais pour la paire qu'ils nomment** : `repositories/projects.rs:77-81`,
     `repositories/supplier_invoices.rs:320-325` (étape (0) de `create_in_tx`) et
     `repositories/reconciliation_rules.rs:190-191` (« anti-ABBA » avec le chemin d'archivage d'un
     projet : sentinelle puis projet, des deux côtés), avec la seule précision qu'ils ne valent que
     pour cette paire si le texte laisse entendre plus ; `repositories/journal_entry_number_sequences.rs:128-131`
     (« pas de cycle entre créations », le compteur pris en premier par toute création), vrai entre
     créations.
   - **Inventaire au symptôme des affirmations d'ordre ou d'absence de cycle** (règle *Inventorier les
     sites NON RÉSOLUS* ; findings F2-1, R2-2, F3-4 — périmètre étendu aux tests ; **insensible à la
     casse** depuis la P4, finding F4-5) :
     `grep -rniE "ABBA|inversion|deadlock|interblocage|cycle|ordre global|global lock order|lock order|ordre des (locks|verrous)|follow the documented order|deny list|lock-order|ordre de verrou|pattern 5|même ordre" crates/*/src crates/*/tests docs/*.md --include=*.rs --include=*.md`
     — motif **étendu en P6** (finding F6-7 : `lock-order` avec trait d'union, « ordre de
     verrou(illage) », « Pattern 5 », « même ordre » affirmaient un ordre hors motif, parfois dans
     des blocs sans aucun mot de l'ancien motif) ; relevé sur **`cecd5d1d`** : **385 lignes dans
     62 fichiers** — **264 lignes / 42 fichiers** sous `crates/*/src` et `docs/*.md` (dont 63 dans
     `kesh-db/src/retry.rs`, 28 dans `repositories/invoices.rs`, 22 dans
     `repositories/journal_entries.rs`, 20 dans `MULTI-TENANT-SCOPING-PATTERNS.md`, 14 dans
     `routes/onboarding.rs`, 14 dans `routes/invoices.rs`, 13 dans `routes/reconciliation.rs`, 12
     dans `kesh-api/src/retry.rs`) et **121 lignes / 20 fichiers sous `crates/*/tests`** (37 dans
     `rejeu_interblocage_e2e.rs`, 21 dans `audit_route_registry.rs`, 12 dans
     `reconciliation_e2e.rs`, 11 dans `journal_entries_modification.rs`, 9 dans
     `opening_complement_repository.rs`, 8 dans `journal_entry_reversal_e2e.rs`). L'ancien motif
     rendait sur le même arbre 347 lignes / 52 fichiers (232 / 36 sous `src` et `docs`, 115 / 16
     sous `tests`). Sites que l'ancien motif ne rendait pas et qui sont nommés pour le tri :
     `crates/kesh-db/src/repositories/fiscal_years.rs:12-18` (doc du module : « Aucune chaîne de
     locks cross-table : `fiscal_years` est isolé » et « `validate_invoice` … acquiert d'abord le
     lock sur `invoices` puis sur `fiscal_years` (Pattern 5) » — affirmation d'ordre **incomplète**,
     l'arrondi précédant l'exercice dans `validate_invoice` : **réécrite** ou renvoyée au
     doc-comment canonique) ; `routes/onboarding.rs:686`, `:853`, `:910` (« Pattern 5 ») ;
     `repositories/company_invoice_settings.rs:312` (« Ordre de verrouillage fixe ») ;
     `repositories/invoice_settlements_write.rs:709` (« même ordre que `settle_invoice` ») ;
     `repositories/invoices.rs:19` (« lock-ordering »). Sur `f289414e` (avant la 15-5e1 et la
     15-8a), l'ancien motif rendait 205 lignes / 42 fichiers : l'écart vient surtout des fichiers **écrits** par la 15-5e1
     (`retry.rs` ×2, `rejeu_interblocage_e2e.rs`, le registre) et par la 15-8a ; la 15-8b en
     ajoutera — T0 recompte sur `HEAD`. **Chaque occurrence est triée au Dev Agent Record**, **bloc
     par bloc**, **en lisant le bloc de commentaire entier** et non la seule ligne rendue — une
     phrase coupée en fin de ligne échappe au motif (p. ex. `fiscal_years.rs:513-515`, « inverserait
     l'ordre des / verrous ») : *vraie* (et pourquoi), *réécrite* (ici, ou par la 15-5e1 —
     doc-comments canoniques, « 5 bis », `retry.rs`), ou *hors sujet* (autre mécanisme). Les blocs
     écrits par la 15-5e1 et la 15-8a, validés et revus dans leur story, se trient comme les autres
     — leur verdict est le plus souvent *vraie* (ils disent déjà la règle de C54), et une phrase de
     leur cru qui prétendrait à une absence de cycle n'en est pas moins bloquante.
     `crates/kesh-api/src/routes/onboarding.rs:592-595` (« New endpoints with multiple locks MUST
     follow the same order to avoid cross-table deadlocks », laissée ici par la 15-5e1) est la
     jumelle de la « Mitigation (v0.1) » du Pattern 5 réécrite ci-dessous : même traitement ;
     `onboarding.rs:232` (« LOCK ORDERING (P3 — partial protection only) », `reset`) est trié de même.
     Pour les tests, le tri est assumé ainsi : une phrase qui **asserte** une absence d'interblocage
     (`crates/kesh-db/tests/opening_complement_repository.rs:746` « aucun interblocage », `:800`
     « l'écriture tenant l'exercice passe : aucun interblocage » ; `fiscal_years_repository.rs:1071`
     « Le FOR UPDATE sérialise → pas de deadlock ») est **vraie si et seulement si elle décrit le
     montage du test** (deux transactions qui ne peuvent pas former de cycle par construction) — elle
     est alors précisée en ce sens, sans toucher aux assertions ; sinon réécrite.
     `crates/kesh-api/tests/reconciliation_e2e.rs:4411-4414` (« Avant la 25-4-c2, […] `retry_with`
     n'existant que sur l'annulation ») est **vraie, historique** — gardée ; le motif rend `:4409`,
     première ligne du même bloc, pas `:4414` (finding R4-3). Une occurrence qui affirme encore une
     absence de cycle sans la limiter à la paire ou au montage qu'elle nomme **bloque la story**.
   - **`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, Pattern 5** (`:285-401` sur `cecd5d1d`). **Déjà faits
     par la 15-5e1, hors périmètre** : « Why Lock Ordering Matters » (`:291`) et l'exemple « How to
     use the retry helper » (`:338-363`, deux enveloppes et la forme générique de `retry_with`, nom
     d'opération compris) — T0 vérifie qu'ils disent toujours vrai, sans les réécrire. **Restent
     ici** :
     - « Global Lock Order » (`:293-305`) devient une **convention de fréquence**, qui nomme la
       sentinelle `companies` et les verrous **partagés** que posent à l'insertion les clés
       étrangères des écritures — `fk_journal_entries_company`
       (`20260412000001_journal_entries.sql:27`), `fk_jel_account` (`:45`), `fk_jel_project`
       (`20260702000001_projects_analytics.sql:40-41`) —, les **inversions** relevées ci-dessus
       (règle, ventilation, manuel, ventilé : exercice, puis sentinelle et projets ; le `PUT` : écriture
       d'abord), et la règle « toute route qui écrit au journal est rejouée » (renvoi aux enveloppes
       et au registre) ;
     - la table « Where This Applies » (`:307-318`) : les lignes `POST /supplier-invoices` (`:315`,
       qui recopie un ordre devenu faux depuis l'avance des réglages de la 15-5e1 : « companies →
       projects → accounts → company_invoice_settings ») et `invoices::validate_invoice` (`:318`)
       **ne recopient plus l'ordre** — elles **renvoient au doc-comment canonique** de
       `supplier_invoices::create_in_tx` et de `validate_invoice`, seuls lieux de vérité (choix C68,
       findings F4-3 = R4-1 de la P4) : la 15-5d y ajoute ses verrous (son AC9) sans avoir à toucher
       au Pattern 5, et l'ordre de merge 15-5d / 15-5e2 est sûr dans les deux sens. La table **gagne
       une ligne pour le lot de rapprochement** (`accept_batch`), qui dit l'ordre **du lot** et non
       d'une seule proposition (finding F4-1) : par proposition, arrondi puis exercice
       (`accept_one_invoice`, `routes/reconciliation.rs:1535` → `:1568`), ou exercice puis
       sentinelle et projets (`accept_one_rule`, `accept_one_split`) ; **entre propositions**, dans
       la même transaction (`:1064-1070`, une sauvegarde par proposition), l'exercice tenu par la
       proposition n précède l'arrondi, la sentinelle ou les projets de la n+1 — l'ordre
       **exercice → arrondi** à l'échelle du lot, inverse de la validation (arrondi
       `repositories/invoices.rs:2093`, exercice `:2200`) : c'est le cycle de #536, que la story ferme
       par le rejeu des deux côtés. Elle **gagne aussi la ligne du `PUT /journal-entries/{id}`** — et
       celle du `DELETE` si la 15-8b a mergé —, **reprise de la rubrique « Deny list »** que la 15-8a
       a posée et que cette story retire (ci-dessous ; choix C85) : son ordre (écriture d'abord), sa
       raison (le `FOR UPDATE` de l'écriture est le premier acte de la transaction), ses trois cycles
       hérités et son test (`update_and_a_reversal_of_the_same_year_can_deadlock`) sont **gardés**.
       **Forme fixée** (findings R6-4 = F6-6 de la P6 : la table d'arrivée n'a que trois colonnes,
       `Endpoint | Lock sequence | File`, et ni « Reason » ni « Mitigation ») : la table garde ses
       trois colonnes ; l'**ordre** du `PUT` (et du `DELETE`) va dans « Lock sequence » ; sa
       **raison**, ses **cycles**, sa **mitigation** et son **test** vont dans un paragraphe
       **« Notes »** placé sous la table, **une puce par endpoint** — la mitigation y devient
       « rejoué (`retry_on_deadlock`, `"journal_entries::update"`) » au lieu de `retry_with(…)`, et
       la phrase « Locking the entry before `companies → projects` diverges from creation » devient
       « inverse l'ordre de la création (convention de fréquence, cf. Global Lock Order) » : sans
       ordre global, « diverge » n'a plus de référent. ⚠️ **Le Pattern 5 nomme les fonctions et les étapes, jamais un
       numéro de ligne** (findings R5-3, F5-3 de la P5) : `accept_one_invoice`,
       `rounding_account_for_write`, `find_open_covering_date`, les sauvegardes par proposition
       d'`accept_batch` ; les numéros ci-dessus servent à la relecture de cette fiche — la 15-5d
       insère ses verrous entre l'arrondi (`:2093`) et l'exercice (`:2200`) de `validate_invoice`,
       ils bougeront si elle merge avant (même logique que C68) ;
     - « Known Risk » (`:320-324`, finding R2-2) : la « Mitigation (v0.1) » de `:324` — aujourd'hui
       « write endpoints follow the documented order, **except those listed in the deny list
       below** (one since Story 15-8a). New endpoints **MUST** follow it or be added to the deny
       list » (« two since Story 15-8b » si elle a mergé) — est réécrite (l'ordre est une convention
       de fréquence, la défense est le rejeu, le registre garde les routes) ;
     - « Resolution status » (`:326-330`) : la puce du helper de rejeu (`:328`, « Used on `finalize`
       via `retry_with(...)` wrapper. Closure must be idempotent », laissée ici par la 15-5e1) nomme
       les deux enveloppes et renvoie au registre ; la puce **« CI lint (grep-detect `FOR UPDATE` +
       verify global order) »** (`:329`, ⏳) est **retirée** (finding R3-6 (e)) — sans ordre global
       garanti, il n'y a plus d'ordre à vérifier, et le registre testé de la 15-5e1 tient le rôle de
       garde ; la puce « Deny list of divergent endpoints » (`:330`, « one entry (Story 15-8a…) ») est
       retirée ; la rubrique « Deny list » (`:332-336`) est retirée, **son contenu passant à « Where
       This Applies »** (ci-dessus) — sans ordre global, « divergent » n'a plus de référence ;
     - le paragraphe **« Required »** (`:365`) est **réécrit sur la règle de la 15-5e1** (finding
       F4-4) : il demande aujourd'hui que `handler_inner` soit « idempotent » et met en garde contre
       les compteurs et les lignes d'audit ; or un 1213 **annule toute la transaction**, compteurs et
       audit compris, et le rejeu est sûr **si toute écriture vit dans la transaction** — une
       tentative = une transaction neuve, aucun effet de bord hors transaction (e-mail, fichier,
       réseau), contrôles qui lisent la transaction gardés dans la tentative et dans leur ordre,
       entrées clonées par tentative, aucune conversion qui fasse sortir le 1213 de
       `DbError::Sqlx` avant le prédicat. Le texte renvoie à `kesh_db::retry` / `kesh_api::retry` et
       à la 15-5e1, sans recopier la règle ;
     - **« When to Use »** (`:367-374`) est réécrit (finding F4-4) : l'enveloppe s'applique à
       **toute route qui écrit au journal** (le registre le vérifie), quel que soit le nombre de
       tables qu'elle verrouille ; l'ordre reste une **convention de fréquence** pour toute
       transaction qui prend plusieurs verrous ;
     - « Code Reference » (`:376-401`, « WRONG: reverse order will deadlock ») dit « réduit la
       fréquence », pas « évite ».

### Documentation

4. **AC4 — `docs/api-external.md` § 10 « Gestion des erreurs »** (`:441` sur `cecd5d1d`) gagne une
   phrase générale — c'est le premier des **trois documents que la 15-5e1 laisse faux** (son AC6 :
   le règlement, son annulation, la validation et la saisie fournisseur, ouverts aux clés API, sont
   rejoués sans que le document le dise) :
   toute route qui écrit au journal — hors `POST /admin/full-import` et `POST /onboarding/reset`, qui
   ne rejouent pas (finding F2-6) — rejoue un interblocage transitoire sans le montrer ; s'il persiste
   après trois tentatives, la requête finit en `500 INTERNAL_ERROR`, rien n'a été écrit, et elle peut
   être renvoyée telle quelle. Comme le lecteur d'`api-external.md` ne sait pas quelles routes
   écrivent au journal (finding F6-9 de la P6), la phrase **énumère** les routes rejouées ouvertes
   aux clés API (relevées en T0 sur le registre, colonne `Rejouee`, croisée avec les routes
   accessibles à une clé `read-write`), et elle est suivie de sa **converse** : « Une autre route
   peut, rarement, rendre `500` sur un conflit transitoire d'accès concurrent ; rien n'est alors
   écrit et la requête peut être renvoyée. » (limite (iv) du registre : une route qui n'écrit pas
   au journal, `POST /reconciliation/reject` ou une route à sentinelle `companies`, peut encore
   perdre un interblocage). Les phrases sont placées **sous** la table du § 10, hors de ses lignes
   (en-tête : conflit avec la 15-5d). Les phrases propres à l'acceptation (`:347`) et à l'annulation
   d'un rapprochement (`:357`) restent vraies ; dans celle de `:347`, « après plusieurs
   tentatives » devient « après trois tentatives », pour que le document dise le même nombre
   partout (`DEFAULT_MAX_DEADLOCK_ATTEMPTS = 3`, `crates/kesh-db/src/retry.rs:71` — finding F5-6 de
   la P5) ; `:357` ne parle pas du nombre et n'est pas touchée. Celles que les 15-8a et 15-8b ont
   posées pour le `PUT` (`:255`, « Un interblocage avec une écriture concurrente est rejoué par le
   serveur ; s'il persiste, la réponse est un `500` — réessayez ») et pour le `DELETE` (même forme,
   si la 15-8b a mergé) restent vraies et ne disent pas de nombre : non touchées (C85).
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
     qui n'existe pas — ce sont les deux autres documents que la 15-5e1 laisse faux (son AC6 ; le
     premier est `api-external.md`, AC4) :
     - `docs/manual/fr/user-manual.tex:969` (§ « Numérotation des factures » ; `:909` sur
       `f289414e`) : « L'attribution du
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
     octets changés sans texte changé ne racontent rien (choix C64, finding F3-9). **Si la 15-5d ou
     la 15-8b a mergé avant** (la 15-5c l'est déjà, et la 15-8b régénère elle aussi les deux PDF) :
     les passages se relocalisent **par leur texte**, et les deux PDF se **régénèrent après rebase**
     au lieu de résoudre leur conflit binaire (finding R5-1 ; en-tête). Les deux PDF sont
     **contrôlés aplatis** :
     `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g'` —
     `SERIALIZABLE` absent des deux PDF, les phrases neuves présentes.
     **Relevé élargi** (T3, `.tex` **et** PDF aplatis, consigné au Dev Agent Record ; motif étendu
     en P3, finding F3-5) :
     `grep -rniE "SERIALIZABLE|isolation|interblocage|deadlock|1213|erreur interne|erreur 500|concurren|simultan|en même temps|racing|race condition|atomi|verrous? de (ligne|base)|FOR UPDATE" docs/manual/fr/*.tex`
     — relevé sur `94f1365e` et sur `f289414e` : 18 lignes ; **relevé sur `cecd5d1d` : 17 lignes**
     (C85) — la ligne `user-manual.tex:1536` d'alors (lot « atomique », renvoi à « la doc CLAUDE.md
     projet ») a été **réécrite par la 15-5c** (mergée, son AC6, #481) et ne sort plus. Restent : les
     deux passages ci-dessus (`user-manual.tex:969`, `admin-manual.tex:885-886`) ;
     `admin-manual.tex:889` (`transaction-isolation = REPEATABLE-READ`, vrai) ; `admin-manual.tex:1395`
     (« en même temps », sauvegarde des documents, hors sujet) ; dix occurrences d'« isolation » au
     sens **multi-société** (`admin-manual.tex:86`, `:932`, `:948`, `:954`, `:2242`,
     `user-manual.tex:2062`, `:2065`, `marketing-brochure.tex:207`, `:317`, `:318` — hors sujet) ;
     `admin-manual.tex:1857` (« Aucun verrou de base de données ne s'y oppose », protection de la
     table d'audit, hors sujet) ; `marketing-brochure.tex:360` (« pas de race conditions
     silencieuses », propriété du langage Rust, hors sujet) — 1 + 2 + 1 + 1 + 10 + 1 + 1 = 17.
     Chaque occurrence restante au dev est triée (la 15-8b, si elle a mergé, peut en ajouter). Les manuels DE, IT, EN ne contiennent qu'un `README.md`
     (rien à traduire) ; la brochure ne parle de « transactions » qu'au sens bancaire (`:167`, `:390`).
6. **AC6 — Aucune règle métier ne change, et c'est vérifié.** Aucun refus neuf, aucune priorité de
   refus déplacée (les contrôles qui lisent la transaction restent dans la tentative, dans leur ordre —
   AC1), aucune variante de `DbError`, aucun code d'erreur, aucune clé i18n, aucun écran ; les
   écritures produites sont identiques ; la suite existante reste verte **sans modification de ses
   assertions**, hors le registre `audit_route_registry.rs`, dont l'AC1 change la partition et
   l'acceptation de `retry_with` (findings R4-6, F4-7).

## Tasks / Subtasks

- [x] **T0 — Refaire les relevés** sur `HEAD` (la 15-5e1 est mergée ; **relever si la 15-8b l'est**
      et le consigner — l'en-tête dit ce qui en dépend) : la remontée de l'AC1 de la 15-5e1 (une
      route qui écrit au journal absente de la table **bloque la story**) ; la partition du registre
      (9 / 13 / 4 / 89 sur `cecd5d1d`, 10 / 12 / 4 / 89 après la 15-8b) ; pour chacune des routes
      `ARejouer` et des sites `retry_with` à migrer (AC1, cinq, ou six avec le `DELETE`), le chemin d'erreur (aucune
      conversion qui fasse sortir une erreur sqlx de `DbError::Sqlx` avant le prédicat), l'absence
      d'effet de bord hors transaction, les entrées à cloner, les contrôles qui dépendent de la
      transaction ; pour chaque site migré, l'**équivalence exacte** (nombre de tentatives,
      prédicat, nom) ; les flux qui prennent l'exercice avant la sentinelle et les projets, **au-delà
      des quatre de l'AC3** (F4-1) ; `grep -rn "retry_with"` (sur `crates/*/src`,
      `crates/*/tests` et `docs`) et `grep -rn "is_deadlock_error"` (AC1) ; les entrées consommées
      de chaque route, `generate_opening_balances` comprise (F6-8) ; **le cycle du test 8 formé à
      la main** sur MariaDB 10.11 (deux sessions : sentinelle `companies` puis exercice d'un côté,
      exercice puis sentinelle de l'autre ; AC2, F6-1) ; les routes rejouées ouvertes aux clés API
      (AC4, F6-9) ; que les doc-comments canoniques de `validate_invoice` et de
      `supplier_invoices::create_in_tx`, cibles des renvois du Pattern 5, disent l'ordre réel sur
      `HEAD` (15-5d mergée ou non — C68) ; que les deux passages du Pattern 5 corrigés par la 15-5e1
      disent toujours vrai ; numéros de ligne des commentaires de l'AC3 et du Pattern 5 ; que les
      corps de `complete_import`, `post_manual` et `post_split` peuvent passer par référence (AC1,
      F5-2) ; si la 15-5d ou la 15-8b a mergé avant, les passages des manuels relocalisés par leur
      texte (AC5, R5-1) ; consigner au Dev Agent Record.
- [x] **T1 — Le rollout** (AC1) : les routes `ARejouer` (13, ou 12 après la 15-8b), dont les trois
      fonctions « une tentative » de `complete_import`, `post_manual`, `post_split` ; la migration
      des sites `retry_with` (cinq, ou six avec le `DELETE`, dont `onboarding::finalize`) **plus**
      le prédicat de `post_accept` et ses deux commentaires `:873`, `:880` (B-3, R6-5, R6-2) ; le
      registre (`ARejouer` retiré, partition 22 / 4 / 89, volet (c) sur 22 routes, `ENVELOPPES`
      sans `retry_with` et `RETRY_WITH_AUTORISE`, banc du visiteur transposé, volet (c bis)
      `no_route_calls_retry_with_except_post_accept`, limites (iii bis) et (vi)) ; doc-comments,
      dont ceux de `write_off_invoice_handler`, `update_journal_entry` (`:652`, `:653`) et
      `finalize`, et les mentions de `retry_with` d'`opening_complement.rs` (`:36`, `:405`),
      d'`invoice_settlements_write.rs:485`, de `journal_entries.rs:1225-1228` et des six sites de
      `crates/*/tests` (AC1, R6-1) ; le **test 8** (AC2, F6-1) — ou son angle mort écrit si T0
      n'a pas formé le cycle. **Revue fichier par fichier** (Dev Notes).
- [x] **T2 — Les commentaires d'ordre et le Pattern 5** (AC3) : `journal_entries.rs` (étapes 0 et
      0-bis, `create_opening_entry`), `accept_one_rule`, `accept_one_split`, `post_manual`,
      `post_split`, `fiscal_years.rs:489-502` (réécrit et remis à sa place) et `:12-18` (doc du
      module), `journal_entries.rs:1683-1684`, `onboarding.rs:592-595` ; le Pattern 5 hors les deux
      passages faits par la 15-5e1 (« Deny list » versée à « Where This Applies » : ordre dans
      « Lock sequence », paragraphe « Notes » sous la table) ; l'inventaire au symptôme (385 lignes
      sur `cecd5d1d`, motif étendu en P6, insensible à la casse, recompté en T0), trié bloc par bloc
      au Dev Agent Record.
- [x] **T3 — Documentation** (AC4, AC5) : `docs/api-external.md` (phrase générale avec
      l'énumération des routes rejouées ouvertes aux clés, et sa converse) ; CHANGELOG ; les deux passages
      `SERIALIZABLE` des manuels et la consigne `innodb_deadlock_detect` (#484) ; deux PDF régénérés,
      brochure non commitée, contrôlés aplatis ; relevé élargi (17 lignes sur `cecd5d1d`) trié,
      consigné ; les trois documents laissés faux par la 15-5e1 contrôlés justes.
- [x] **T4 — Les mutations** (AC2) : six lancées (dont celle de `post_manual`, qui doit faire
      rougir le volet (c) **et** le test 8, et celle de `finalize`, qui doit faire rougir le volet
      (c bis)), une consignée sans exécution.
- [x] **T5 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) —
      **même en cours de boucle de revue**, la story touchant des repositories `kesh-db` (commentaires)
      et le registre ; gate frontend complet (rien n'y change : il le confirme) ; **E2E Playwright
      complet au dernier commit de code** (décision D7), jugé fichier par fichier contre
      `docs/testing.md` § « Les échecs attendus ».

*(Décompte : 6 AC, 6 tâches T0–T5.)*

## Dev Notes

### L'inventaire

La table « Les routes qui écrivent au journal » est dans la **15-5e1** (Dev Notes), avec sa colonne
« fiche » (relevée avant les 15-8a et 15-8b : depuis, le `PUT` est rejoué par la 15-8a, et le
`DELETE`, marqué 15-5e2, l'est par la 15-8b si elle merge avant — AC1, en-tête) : les lignes marquées
15-5e2 sont le périmètre de l'AC1 — enveloppe, fonction de dépôt et
correspondance d'erreur à faire **après** le rejeu y sont écrites. Elle n'est pas recopiée ici (une
seule table à tenir à jour).

### Pourquoi la revue fichier par fichier suffit au rollout

Chaque route reçoit la même forme (un appel d'enveloppe autour d'une fonction de dépôt, ou une
fonction « une tentative »), le registre garde la présence de l'enveloppe (volet (c), robuste depuis
la 15-5e1) et la partition. La preuve dynamique n'est **pas** égale entre les deux familles
(finding F4-8) : pour la famille `DbError`, les tests 2 à 5 et 7 de la 15-5e1 prouvent le mécanisme
sur une vraie 1213, sur cinq routes de même forme, chacun avec un témoin du rejeu (C74) ; pour la famille `AppError`, le test 1 de la 15-5e1 ne
prouve que le prédicat et l'enveloppe ; le chemin propre aux routes à verrou nommé (verrou nommé,
`match` du handler, `RELEASE_LOCK`, `rollback`) est prouvé par le **test 8** sur `post_manual`
(AC2, choix C86, finding F6-1 de la P6 : le harnais de la 15-5e1 le rend désormais abordable) ;
restent un **angle mort assumé** (AC2), couvert par la revue seule, `post_split` (même chemin que
`post_manual`, sur la route sœur) et `complete_import` (verrou du `staging`, `match` local,
`rollback`, et l'ordre des contrôles lus sur le `staging` — finding F5-1) — et `post_manual`
aussi si T0 ne forme pas le cycle du test 8.
**Angles morts du volet (c bis)** (revue P1, L-3 = B-4 = A5 ; écrits au point (vii) du doc-comment
du registre) : il reconnaît `retry_with` au **dernier segment** du chemin appelé, dans un arbre
`syn`, et ne voit donc pas (a) un alias `use kesh_db::retry::retry_with as r;` puis `r(…)`, (b) un
appel dans une macro (`syn` ne descend pas dans un `TokenStream` : **faux vert** pour (c bis), là où
c'est un faux rouge pour (c)), (c) un appel hors de `src/routes/`, seul répertoire balayé, (d)
l'exemption `RETRY_WITH_AUTORISE` désigne `post_accept` par son **seul nom**, quel que soit le
fichier. La méthode qui fermerait (a) et (b) à moindre coût est le relevé lexical par jetons de la
15-11b (C78) ; elle n'est pas introduite ici.

Ce que la revue doit voir, route par route : (1) la fermeture commence par la transaction et finit par
le commit ; (2) aucun contrôle qui lit la transaction n'en sort, aucun refus ne change de place ; (3)
les entrées consommées sont clonées dans la fermeture ; (4) la conversion d'erreur propre à la route
vient après le rejeu ; (5) le nom d'opération est unique et suit `"<module>::<action>"`.

### Ce que vit l'utilisateur

Comme pour les cinq routes rejouées par la 15-5e1 : l'opération aboutit, avec au pire ≈ 150 ms de latence
ajoutée ; seul un interblocage répété trois fois de suite ressort en 500, rien n'étant alors écrit.

### Fichiers touchés (prévision)

`crates/kesh-api/src/routes/{invoices,credit_notes,supplier_invoices,imported_supplier_invoices,payment_batches,journal_entries,opening_balances,reconciliation,onboarding}.rs`
(`onboarding.rs` : migration de `finalize` et commentaire `:592-595`, C85) ;
`crates/kesh-db/src/repositories/{journal_entries,fiscal_years,opening_complement,invoice_settlements_write}.rs`
(commentaires ; « 5 bis » pour le dernier) et, selon le tri de l'inventaire, d'autres commentaires de
`crates/*/src` et de `crates/*/tests` ; `crates/kesh-api/tests/audit_route_registry.rs` (partition,
`ENVELOPPES`, `RETRY_WITH_AUTORISE`, banc du visiteur, volet (c bis), limites (iii bis) et (vi)) ;
`crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (test 8) ; les doc-comments de
`crates/kesh-api/tests/journal_entry_reversal_e2e.rs` et `crates/kesh-db/tests/journal_entries_modification.rs`
(mentions de `retry_with`, AC1) ;
`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `docs/api-external.md`, `CHANGELOG.md` ;
`docs/manual/fr/{user-manual,admin-manual}.{tex,pdf}` (#484). **Aucune migration**, aucun fichier
`kesh-i18n` ni `frontend`.

**Règle de découpage** : plus de cinq modules de routes, mais c'est le **rollout** que la règle
prescrit après une story-zéro (§ *Règle de splitting préventif*, « Comment splitter »). Il n'est pas
**strictement** mécanique (finding F4-9) : trois des routes à rejouer (treize, ou douze après la 15-8b) exigent l'extraction d'une
fonction « une tentative » qui garde l'ordre des contrôles (R3-4), et c'est le point que la preuve
dynamique couvre le moins (F4-8 ; le test 8 n'en exerce qu'une, `post_manual`, F6-1). D'où le partage de la revue de code (`bmad-code-review`, § *Review
Iteration Rule*) : la **revue fichier par fichier** s'applique aux routes `DbError` (dix, ou neuf
après la 15-8b) et aux migrations des sites `retry_with` (cinq, ou six avec le `DELETE`,
équivalences exactes) ;
`onboarding.rs` ajoute un module de routes à la liste, sans changer la nature du rollout (C85) ; les trois extractions (`complete_import`, `post_manual`, `post_split`) reçoivent **au
moins une lentille adversariale** de la revue de code ; les parties documentaires (Pattern 5, manuels,
inventaire) restent relues par les passes de validation.

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C54**, **C55** — le rejeu comme défense ; ce qui reste de l'ordre des verrous (C55 révisé par
  **C66** : la ligne des réglages ne « sérialise » pas).
- **C56** — deux enveloppes partagées, le registre (dérogation de découpage révisée par **C61** ;
  « aucun test dynamique au rollout » révisé par **C86**).
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
- **C85** — alignement sur le livré (15-5e1, 15-8a, 15-8b) : partition d'arrivée 22 / 4 / 89 dans les
  deux ordres de merge avec la 15-8b ; tous les sites `retry_with` des routes migrés vers les
  enveloppes sauf `post_accept` (dont `onboarding::finalize` et le `PUT`, prédicats en ligne B-3) ;
  « Deny list » du Pattern 5 versée à « Where This Applies » ; inventaire recompté.
- **C86** — remédiation de la P6 : test 8 « route victime » sur `post_manual` (révise **C56** et
  l'écart de C69), volet (c bis), mécanisme de restriction de `retry_with`, contrôle `grep` étendu
  aux tests, motif de l'inventaire étendu.

### References

- Issues : #536, #484 (fermées ici) ; #463, #491 (fermées par la 15-5e1) ; #555 (R3-7) ; #481 (le
  passage « atomique » du manuel, réécrit par la 15-5c, mergée) ; #532 (15-8a, 15-8b).
- Code livré : PR #559 (`de1e1c26`, 15-5e1 — `kesh_db::retry`, `kesh_api::retry`, registre à deux
  colonnes, `tests/common/capture_rejeu.rs`) ; PR #553 (15-8a, `PUT` et « Deny list »).
- Fiches : `15-5e1-socle-rejeu.md` (le patron, l'inventaire, la règle), `15-5e-ordre-des-verrous-reglements.md`
  (index, historique des validations P1–P3), `15-5c-rapprochement-libelles-et-manuel.md` (AC6 :
  passage « atomique » du manuel), `15-8a-modifier-une-ecriture.md`, `15-8b-supprimer-une-ecriture.md`
  (worktree `kesh-15-8` tant qu'elle n'est pas mergée).
- `CLAUDE.md` : § *Inventorier les sites NON RÉSOLUS*, § *Le prompt d'une passe doit NOMMER le manuel*
  (PDF aplati), § *Règle de splitting préventif*, § *Propagation post-patch*.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), `bmad-dev-story` en autonomie (consignes de l'Epic 15). Worktree
`/home/gcorbaz/devel/kesh-15-5e2`, branche `story/15-5e2-rejeu-des-autres-flux`, cible cargo
`CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-15-5e2/target`, bases dédiées `kesh_155e2` (gate) et
`kesh_e2e_155e2` (E2E). Commit de code : `6125d50b`.

### Debug Log References

- Gate backend : `target/gate-logs/15-5e2-gate-backend.log` ; gate frontend :
  `target/gate-logs/15-5e2-gate-frontend.log` ; E2E : `target/gate-logs/15-5e2-e2e.log` (worktree,
  non versionnés).

### Completion Notes List

**T0 — relevés sur `HEAD` (`688fed25`, qui porte la 15-8b ; choix C-15-5e2-1).**

- **La 15-8b est mergée** (`ec675288`) : partition de départ **10 `Rejouee` / 12 `ARejouer` / 4 / 89
  = 115** (recomptée sur le registre). Aucune route qui écrit au journal hors de la table.
- **Cycle du test 8 formé à la main** sur `10.11.16-MariaDB-ubu2204` (`innodb_deadlock_detect = ON`,
  `innodb_deadlock_report = full`, relevé `SHOW VARIABLES` — T3 aussi) : session A alourdie (500
  lignes) tient `companies` `FOR UPDATE` ; session B prend l'exercice par la requête réelle de
  `find_open_covering_date` puis attend `companies` ; A demande l'exercice → **B reçoit 1213**, A
  obtient son verrou (`SHOW ENGINE INNODB STATUS`). **Le test 8 est écrit en vert**, non comme angle
  mort.
- **Sept sites `retry_with`** dans `crates/kesh-api/src/routes` (`invoices.rs` write_off,
  `opening_balances.rs` complete, `journal_entries.rs` PUT et DELETE, `reconciliation.rs` accept et
  cancel, `onboarding.rs` finalize). **Équivalences** vérifiées à la lecture : les six migrés
  passaient `DEFAULT_MAX_DEADLOCK_ATTEMPTS` — l'enveloppe aussi — ; prédicat `|err: &DbError|
  is_deadlock_error(err)` = celui de `retry_on_deadlock` (via `retry_on_deadlock_with`) ; prédicat
  en ligne `matches!(err, AppError::Database(db) if is_deadlock_error(db))` = `is_app_deadlock` à la
  lettre ; noms d'opération inchangés. `post_accept` : seule la forme du prédicat change
  (`crate::retry::is_app_deadlock(err) || matches!(err, AppError::ReconciliationTransactionAborted)`).
- **Routes à rejouer** — chemin d'erreur, effets de bord, entrées, contrôles, route par route :
  `unvalidate`, `create_credit_note`, `pay`, `cancel`, `cancel_settlement` (fournisseur),
  `confirm_batch`, `reverse` — une fonction de dépôt qui ouvre et conclut sa transaction, rend
  `DbError` par `map_db_error` (le 1213 reste `DbError::Sqlx` : `errors.rs:1048`, aucun cas 1213),
  aucun effet de bord hors transaction ; entrées `Copy` (`SettlementChoice` est `Copy`,
  `NewCreditNote` construit dans la fermeture, `req.payment_date` `Copy`). `create_journal_entry` et
  `generate_opening_balances` : `NewJournalEntry` **cloné par tentative**, pré-contrôles sur le pool
  (lecture de l'exercice, comptage, types de comptes en transaction de lecture annulée) gardés
  **avant** la fermeture et dans leur ordre ; `map_err` / `map_opening_balances_error` **après** le
  rejeu. Les trois extractions : les corps `CompleteImportRequest`, `ManualMatchBody`, `SplitBody`
  passent **par référence** (aucun `Clone` à dériver) ; `complete_import_once` garde verrou du
  `staging`, statut, devise, IBAN/QRR, montant **dans cet ordre** après le verrou ; dans
  `post_manual_once` et `post_split_once`, les erreurs sqlx passent par
  `ReconciliationError::Db(DbError::Sqlx)` ou `?` (`From<DbError>`), le `match` les rend en
  `AppError::Database(db_err)` après `rollback` — forme reconnue par `is_app_deadlock`. Choix de
  frontière : C-15-5e2-3.
- **Flux qui prennent l'exercice avant la sentinelle et les projets** : les quatre de l'AC3
  (`accept_one_rule`, `accept_one_split`, `post_manual`, `post_split`) ; aucun autre trouvé par
  l'inventaire (les autres appelants de `validate_taggable_in_tx` — saisie fournisseur, factures,
  règles, `create_in_tx` étape 0 — la prennent avant l'exercice ; le `PUT` verrouille d'abord
  l'écriture, déjà écrit).
- **Routes rejouées ouvertes aux clés API** : les 22 routes `Rejouee` sont montées dans
  `comptable_routes` (`crates/kesh-api/src/lib.rs:347-727`), ouvertes à une clé `read-write` ;
  `update_invoice_settings` (rejouée, `SansEcritureAuJournal`) est dans `admin_routes`, fermée aux
  clés.
- **Doc-comments canoniques** de `validate_invoice` et `supplier_invoices::create_in_tx` : disent
  l'ordre réel sur `HEAD` (la 15-5d n'est pas mergée) ; les deux passages du Pattern 5 de la 15-5e1
  (« Why Lock Ordering Matters », exemple « How to use ») disent toujours vrai — à la réserve près,
  écrite dans « Required », que la forme générique `retry_with` est réservée à `post_accept` dans
  `src/routes/`.
- **Inventaire au symptôme** (motif de l'AC3, insensible à la casse) : sur `HEAD`, **394 lignes / 62
  fichiers** (272 / 42 sous `crates/*/src` et `docs/*.md`, 122 / 20 sous `crates/*/tests`) — 385 sur
  `cecd5d1d` + la 15-8b ; après cette story, **462 lignes / 64 fichiers** (324 / 44 ; 138 / 20), la
  hausse venant des doc-comments « rejouée sur interblocage » et des tests neufs.

**T1 — rollout (AC1).** Douze routes `ARejouer` passées par une enveloppe : neuf `DbError`
(`invoices::unvalidate`, `credit_notes::create`, `supplier_invoices::pay`, `supplier_invoices::cancel`,
`supplier_invoices::cancel_settlement`, `payment_batches::confirm`, `journal_entries::create`,
`journal_entries::reverse`, `opening_balances::generate` — noms : C-15-5e2-2) et trois `AppError`
(`imported_supplier_invoices::complete`, `reconciliation::manual`, `reconciliation::split`), chacune
avec un doc-comment d'une ligne renvoyant à l'enveloppe — ⚠️ **faux au commit `6125d50b` pour
`post_manual` et `post_split`** (seuls leur corps et leurs fonctions « une tentative » le disaient),
corrigé en remédiation de la revue P1 (finding A1). Six sites `retry_with` migrés (`write_off`,
`opening_balances::complete`, `PUT`, `DELETE`, `reconciliation::cancel`, `onboarding::finalize`) ;
`post_accept` garde `retry_with`, prédicat réécrit, commentaires `:873`, `:880` → `is_app_deadlock`.
Contrôles : `grep -rn "is_deadlock_error" crates/kesh-api/src` ne rend que `retry.rs` ;
`grep -rn "retry_with(" crates/kesh-api/src/routes` ne rend que `post_accept` (`reconciliation.rs:890`) ;
`grep -rn "retry_with" crates/*/src crates/*/tests docs --include=*.rs --include=*.md` ne rend plus
que les occurrences légitimes de l'AC1 (les deux `retry.rs`, `post_accept` et la doc d'`accept_once`,
l'exemple générique et la phrase « Required » du Pattern 5, `capture_rejeu.rs:6`,
`reconciliation_e2e.rs:4414`, le registre — `RETRY_WITH_AUTORISE`, `PRIMITIVE`, volet (c bis), bancs
synthétiques —) et `onboarding.rs:600`, réécrit en « la primitive » pour ne plus nommer
`retry_with`. Registre : `ARejouer` retiré, partition **22 / 4 / 89 = 115**, volet (c) sur 22
routes, `ENVELOPPES` sans `retry_with`, `RETRY_WITH_AUTORISE = &["post_accept"]`, banc du visiteur
transposé (`qualifie`, `turbofish` → `retry_on_deadlock` / `retry_app_on_deadlock` ; `dans_un_bloc`,
`aide` idem) avec un cas négatif `primitive_seule` et un cas autorisé `post_accept`, volet (c bis)
`no_route_calls_retry_with_except_post_accept` et son banc
`the_primitive_visitor_names_the_enclosing_function` (C-15-5e2-4), limites (ii), (iii bis), (vi)
réécrites. Mentions de `retry_with` réécrites : `opening_complement.rs` (doc du module, doc de
`create_opening_complement`), « 5 bis » (`invoice_settlements_write.rs`), `journal_entries.rs`
(doc d'`update`, « Where This Applies »), `journal_entry_reversal_e2e.rs` (mutation tuée),
`journal_entries_modification.rs`, et dans le registre `:89`, `:119`, `:165`, `:600` d'alors. Le doc
de `delete_in_tx` ne nommait pas `retry_with` (rien à changer).

**Test 8** (`manual_match_is_replayed_when_it_is_the_deadlock_victim`, `rejeu_interblocage_e2e.rs`) :
montage avec projet (compte bancaire lié à `1100`, contrepartie `4000`, transaction bancaire du jour
de −150, projet actif) ; la transaction de test tient la sentinelle, la route attend sur
`["companies", "FOR UPDATE"]`, la transaction de test demande l'exercice ; assertions : `200`, une
seule écriture créée, transaction bancaire `reconciled` par l'écriture rendue, une seule entrée
d'audit `reconciliation.manual_matched`, témoin `exiger_un_rejeu("reconciliation::manual")`.

**T2 — commentaires d'ordre et Pattern 5 (AC3).** Réécrits : étape 0 de `create_in_tx_inner`,
étape 0-bis (puce « Verrous » retirée, en-tête ajusté, « Refus » et « Lecture NON verrouillante »
gardés), `create_opening_entry` (deux passages), la suite du renvoi à l'étape 0-bis dans
`delete_in_tx` (« inverserait l'ordre global » retiré, renvoi et « Seuil INCLUSIF » gardés),
Step 11bis d'`accept_one_rule`, Step j d'`accept_one_split`, Step 6bis de `post_manual` (qui
affirmait l'inverse), Step 11 de `post_split` ; `fiscal_years.rs` : doc-comment de
`find_open_covering_date` **remis à sa place** et réécrit, doc du module (`:12-18`) réécrite ;
`onboarding.rs` (ex-`:592-595`, « MUST follow the same order to avoid cross-table deadlocks »).
Précisions « pour cette paire » et tri des autres blocs : C-15-5e2-5. Pattern 5 : « Global Lock
Order » en convention de fréquence (verrous partagés `fk_journal_entries_company`, `fk_jel_account`,
`fk_jel_project` ; les quatre flux inversés ; le rejeu et le registre) ; « Where This Applies » :
lignes `POST /supplier-invoices` et `invoices::validate_invoice` par renvoi aux doc-comments
canoniques, ligne du lot de rapprochement (`accept_batch`, ordre par proposition et entre
propositions, cycle de #536), lignes du `PUT` et du `DELETE` reprises de la « Deny list »,
paragraphe « Notes » ; « Mitigation » réécrite ; puces « CI lint » et « Deny list » et rubrique
« Deny list » retirées ; « Required » et « When to Use » réécrits ; « Code Reference » (C-15-5e2-6).
Aucun numéro de ligne au Pattern 5.

Tri de l'inventaire, bloc par bloc (lu entier) — verdicts :
- *réécrites ici* : les blocs ci-dessus ;
- *réécrites par la 15-5e1 et vraies* : `kesh-db/src/retry.rs` (63 lignes), `kesh-api/src/retry.rs`,
  doc-comments canoniques (`invoices.rs:1921-1961`, `supplier_invoices.rs:249-286`), « 5 bis »,
  doc-comments des routes rejouées par la 15-5e1 ;
- *vraies pour la paire ou le montage qu'elles nomment* : `projects.rs:77-81`,
  `supplier_invoices.rs:320-325`, `reconciliation_rules.rs:190-191` (précisés),
  `journal_entry_number_sequences.rs:128-131`, `invoices.rs:1074-1078`, `:1099-1101` (précisé),
  `:2014`, `:2124`, `opening_complement.rs:26-37` (« Le dépôt n'a pas d'ordre de verrouillage
  unique ») ; tests `opening_complement_repository.rs:746`/`:800`, `fiscal_years_repository.rs:1051`/
  `:1071` (précisés, assertions intactes), `journal_entries_modification.rs`,
  `journal_entry_reversal_e2e.rs`, `reconciliation_e2e.rs:4409-4523` (historique gardé),
  `rejeu_interblocage_e2e.rs`, `audit_route_registry.rs`, `capture_rejeu.rs` ;
- *hors sujet* (autre mécanisme) : « cycle » de vie / de crates / CPU (`audit.rs:5`,
  `dunning_reminders.rs`, `password.rs:90`, `chart_of_accounts/mod.rs:48`, `account.rs:84`,
  `users.rs:41`, `invoices.rs:1483`, `:2213`, `:4448`, `dunning_eligibility.rs:35`,
  `imported_supplier_invoices.rs:287`, `docs/testing.md:46` et les tests homologues), « même ordre »
  de lignes ou de refus (`credit_notes.rs:225`, `audit_log.rs`, `bank_imports.rs:1170`,
  `journal_entries.rs:664`, `opening_balances.rs:389`, `kesh-qrbill/src/types.rs:347` et les tests
  homologues), auto-interblocages de connexion (`onboarding.rs:282`, `admin.rs:257`,
  `kesh-seed/src/lib.rs:117`), interblocages de lacunes d'`INSERT` (`email_templates.rs:20-30`,
  `:245`), test de concurrence de `invoices.rs:5430-5488`, `errors.rs` (kesh-api, kesh-reconciliation),
  `inbox_import.rs:126` (verrou nommé de connexion), `company_invoice_settings.rs:312` (ordre entre
  comptes désignés), `invoice_settlements_write.rs:709`, `onboarding.rs:232`, `:683`, `:850`,
  `:907`, `reconciliation_cancel.rs:256` (mis à jour, #463), `docs/api-external.md` (phrases de rejeu,
  vraies).
Aucune occurrence n'affirme encore une absence de cycle sans la borner à une paire ou à un montage.

**T3 — documentation (AC4, AC5).** `docs/api-external.md` § 10 : phrase générale sous la table,
énumérant les 22 routes rejouées ouvertes aux clés, hors restauration et effacement de la démo, et
sa converse ; « après plusieurs tentatives » → « après trois tentatives » (acceptation d'un
rapprochement) ; phrases du `PUT` et du `DELETE` non touchées. CHANGELOG `[0.13.0]` « Corrigé » :
ligne de la 15-5e1 remplacée par le texte final, #536 en entier, et ligne #484 ajoutée ; « Modifié »
non touché. Manuels (#484) : `user-manual.tex:982` (numérotation : compteur verrouillé, contrainte
d'unicité, rejeu, réserve des trois tentatives), `admin-manual.tex:885-896` (commentaire de
`99-kesh.cnf` : `REPEATABLE READ`, verrous de ligne et nommés, relance automatique jusqu'à trois
tentatives, consigne `innodb_deadlock_detect` à sa valeur `ON` — lignes ASCII ≤ 70 caractères,
vérifié par script). `make admin user` : `Overfull` **69 → 69** (admin) et **26 → 26** (utilisateur),
compte relevé sur un build des `.tex` d'avant la modification puis après ; brochure ni régénérée ni
modifiée. PDF aplatis (`pdftotext -nopgbrk | tr | tr -s | sed` des ligatures) : `SERIALIZABLE`
absent des deux, phrases neuves présentes (le commentaire du listing traverse un saut de page, texte
complet). Relevé élargi (motif de l'AC5) sur les `.tex` après modification : **21 lignes**
(recompté en revue P1, finding A3 : « 23 » était faux, et la ventilation comptait deux fois `:891`) —
les sept lignes du commentaire `99-kesh.cnf` (`:885`, `:887`, `:888`, `:890`, `:891` =
`transaction-isolation = REPEATABLE-READ`, `:893`, `:894`) et `user-manual.tex:982`, vraies ; `:1402` (« en
même temps », sauvegarde), `:1864` (verrou de base, table d'audit), dix « isolation » multi-société
(`admin-manual.tex:86`, `:939`, `:955`, `:961`, `:2249`, `user-manual.tex:2075`, `:2078`,
`marketing-brochure.tex:207`, `:317`, `:318`), `marketing-brochure.tex:360` (Rust) — hors sujet. Les
trois documents laissés faux par la 15-5e1 (api-external, manuel utilisateur, manuel
d'administration) sont justes.

**T4 — mutations (AC2)**, chacune : copie, mutation, gate ciblé, restauration, `touch`, et contrôle
`git diff` identique avant / après (script `mut.py` hors dépôt) :
1. enveloppe retirée de `create_journal_entry` → **volet (c) rouge** ;
2. enveloppe retirée de `pay_supplier_invoice` (doc-comment de `cancel_supplier_invoice` nommant
   l'enveloppe juste après) → **volet (c) rouge** sur `pay_supplier_invoice` ;
3. enveloppe retirée de `post_manual` → **volet (c) rouge et test 8 rouge** (`500 INTERNAL_ERROR`) ;
4. `retry_with` recopié dans `write_off_invoice_handler` → **volet (c) et volet (c bis) rouges** ;
   idem dans `update_journal_entry` → **les deux rouges** ;
5. `onboarding::finalize` remis en `retry_with` à prédicat en ligne → **volet (c bis) rouge**, en
   nommant `onboarding.rs : fn finalize` ;
6. `ARejouer` remis à la main au registre → ne compile pas (variante retirée) : consigné, non lancé.
Après restauration : `binary(rejeu_interblocage_e2e) | binary(audit_route_registry)` 18/18 vert.

**T5 — gates**, au commit de code `6125d50b`, cible cargo du worktree, base `kesh_155e2` remise à
zéro (DROP/CREATE, migrations, seed) avant :
- **backend** : `scripts/test-fast.sh` (fmt, clippy `-D warnings`, nextest) — **2837 / 2837
  passés, 4 ignorés** (107,8 s) ; vert ;
- **frontend** : `npm run check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership`
  vert, `test:unit` **1086 / 1086** (111 fichiers), `build` vert ;
- **E2E Playwright complet** (backend `./target/debug/kesh-api` du worktree sur le port **3005**,
  base `kesh_e2e_155e2` remise à zéro et migrée, secrets générés par `openssl rand`,
  `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés, SMTP factices, inbox et
  documents en scratchpad, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, run de 19:40 UTC) :
  **246 passés / 8 échoués / 19 ignorés**. Jugés contre `docs/testing.md` § « Les échecs attendus » :
  les **7 KF-029** (#97 : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150`) et `sidebar-navigation.spec.ts:75`, **rejoué seul : vert
  (1,4 s)** — pollution d'état, non KF-046 ni régression. Aucune spec ne touche les routes modifiées
  hors du chemin nominal ; aucun rouge hors liste. Backend E2E arrêté après le run.

**Revue de code P1 — remédiation** (commit `ee77f450`, cible cargo
`CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-15-5e2/target`). Mutation L-1 (lecture de
`was_previously_rejected` neutralisée à `false` dans `post_manual_once`, ce qu'écrivait la
pré-lecture) : test 8 **rouge** (`Some("false")` contre `Some("true")`) ; restaurée, `touch`, test 8
vert. Gates au commit `ee77f450`, bases `kesh_155e2` et `kesh_e2e_155e2` remises à zéro
(DROP/CREATE, migrations, seed pour la première) :
- **backend** : `scripts/test-fast.sh` (fmt, clippy `-D warnings`, nextest) — **2837 / 2837 passés,
  4 ignorés** (117,4 s) ; vert (aucun test neuf : le test 8 est étendu) ;
- **frontend** : `npm run check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership`
  vert, `test:unit` **1086 / 1086** (111 fichiers), `build` vert ;
- **E2E Playwright complet** (backend du worktree, port 3005, secrets `openssl rand`, SMTP factices,
  inbox et documents en scratchpad, `KESH_TEST_MODE` des deux côtés) : **249 passés / 7 échoués /
  17 ignorés** (10,3 min). Les 7 sont les **KF-029** (#97 : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) ; aucun rouge hors
  liste. Backend arrêté après le run.

### File List

- `crates/kesh-api/src/routes/credit_notes.rs`
- `crates/kesh-api/src/routes/imported_supplier_invoices.rs`
- `crates/kesh-api/src/routes/invoices.rs`
- `crates/kesh-api/src/routes/journal_entries.rs`
- `crates/kesh-api/src/routes/onboarding.rs`
- `crates/kesh-api/src/routes/opening_balances.rs`
- `crates/kesh-api/src/routes/payment_batches.rs`
- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/src/routes/supplier_invoices.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`
- `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`
- `crates/kesh-db/src/repositories/fiscal_years.rs`
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/src/repositories/invoices.rs`
- `crates/kesh-db/src/repositories/journal_entries.rs`
- `crates/kesh-db/src/repositories/opening_complement.rs`
- `crates/kesh-db/src/repositories/projects.rs`
- `crates/kesh-db/src/repositories/reconciliation_cancel.rs`
- `crates/kesh-db/src/repositories/reconciliation_rules.rs`
- `crates/kesh-db/src/repositories/supplier_invoices.rs`
- `crates/kesh-db/tests/fiscal_years_repository.rs`
- `crates/kesh-db/tests/journal_entries_modification.rs`
- `crates/kesh-db/tests/opening_complement_repository.rs`
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md`
- `docs/api-external.md`
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/15-5e2-rejeu-des-autres-flux.md`,
  `_bmad-output/implementation-artifacts/sprint-status.yaml`,
  `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`

Décompte (recompté en revue P1, finding A2, `git diff --shortstat 688fed25 6125d50b`) : **31
fichiers** au commit de code (dont 2 PDF), **959 insertions**, 369 suppressions (« 32 fichiers,
998 insertions » était faux) ; tests neufs sur ce périmètre : **3** (`#[test]` du
registre 9 → 11, `#[sqlx::test]` de `rejeu_interblocage_e2e.rs` 6 → 7). Aucune migration, aucune clé
i18n, aucun fichier `frontend`.

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
- 2026-10-08 — **Alignement sur le livré (15-5e1, 15-8a, 15-8b)** (choix **C85**, sans passe de
  validation ; code relu sur `cecd5d1d`, qui intègre `origin/main` `de1e1c26` ; worktree de la 15-8b
  `kesh-15-8` lu en lecture seule, rebase en cours). Sections modifiées :
  - **en-tête** (commentaire) : statut — la 15-5e1 est mergée, le développement peut commencer ;
    numéros relevés sur `cecd5d1d` ; C85 ajouté aux choix applicables ;
  - **Issues** : `refs #555` (R3-7, mesurée vraie par la 15-5e1) ;
  - **Dépend** : réécrit — ce que la 15-5e1 a livré (signatures `retry_on_deadlock(operation, f)`,
    `retry_on_deadlock_with(operation, max_attempts, f)`, `retry_app_on_deadlock(operation, f)`,
    `is_app_deadlock`, primitive `retry_with(operation, max_attempts, should_retry, f)`, `warn!` /
    `error!`, registre à deux colonnes, partition 9 / 13 / 4 / 89, doc-comments canoniques et leurs
    étiquettes, témoin `capture_rejeu.rs`), les deux passages du Pattern 5 qu'elle a déjà corrigés
    (sortis du périmètre), ce qu'elle laisse ici (`onboarding.rs:595`, « Used on `finalize` »,
    prédicats en ligne B-3, trois documents faux) ; **nouveau paragraphe 15-8a / 15-8b** avec le
    tableau des partitions (départ 9 / 13 / 4 / 89 ou 10 / 12 / 4 / 89, arrivée **22 / 0 / 4 / 89**
    dans les deux ordres) et les conflits attendus ; puce 15-5d complétée (« 5 bis » commun, `:969`,
    AC8 et non AC9 pour les PDF de la 15-5d) ;
  - **AC1** : réécrit — titre ; routes à rejouer 13 (10 `DbError` dont le `DELETE`, 3 `AppError`),
    12 si la 15-8b a mergé ; **table des sites `retry_with` directs** (six sur `cecd5d1d`, sept avec
    le `DELETE`) et leur sort — tous migrés vers une enveloppe par équivalence exacte, dont le `PUT`
    de la 15-8a et `onboarding::finalize` (B-3), sauf `post_accept`, dont seul le prédicat change
    (`is_app_deadlock(err) || …TransactionAborted`) ; contrôles `grep` ; registre 22 / 4 / 89,
    volet (c) sur 22 routes ; doc-comments et mentions de `retry_with` à réécrire, liste complétée
    (`invoice_settlements_write.rs:485`, `journal_entries.rs:1225-1228`, `update_journal_entry`,
    `finalize`) ; numéros relocalisés ;
  - **AC2** : preuves dynamiques des sites migrés (`the_put_replays_a_deadlock_it_lost`), témoin
    `capture_rejeu.rs` pour un éventuel test neuf, tests à garder verts complétés ; mutation sur
    `update_journal_entry` ; `cancel_supplier_invoice` `:400-404` ;
  - **AC3** : numéros relocalisés sur `cecd5d1d` ; le `PUT` (15-8a) situé parmi les inversions ;
    renvoi de `journal_entries.rs:1682` à l'étape 0-bis ; inventaire au symptôme **recompté :
    347 lignes / 52 fichiers** (232 / 36 sous `src` et `docs`, 115 / 16 sous `tests`) au lieu de
    205 / 42 ; Pattern 5 : les deux passages faits par la 15-5e1 retirés du périmètre, la rubrique
    « Deny list » de la 15-8a **versée à « Where This Applies »** (ordre, raison, cycles, test
    gardés), « Mitigation » et puces de `:328`, `:330` relues sur leur texte actuel, numéros
    `:285-401` ;
  - **AC4** : § 10 `:441`, phrases `:347`, `:357`, `retry.rs:71` ; phrases du `PUT` (`:255`) et du
    `DELETE` non touchées ; lien avec les trois documents laissés faux par la 15-5e1 ;
  - **AC5** : `user-manual.tex:969` ; relevé élargi **recompté : 17 lignes** (la ligne « atomique »
    réécrite par la 15-5c ne sort plus) avec numéros relocalisés ; 15-8b ajoutée aux cas de
    régénération des PDF ;
  - **T0 à T3** : relevés (15-8b, équivalences, `is_deadlock_error`), rollout (sites, partition
    22 / 4 / 89), Pattern 5 et inventaire (347), relevé élargi (17) ;
  - **Dev Notes** : « L'inventaire » (le `DELETE`), « Fichiers touchés » (`onboarding.rs`,
    `invoice_settlements_write.rs`), « Règle de découpage », « Décisions » (C85), « References »
    (#555, #532, PR #559, PR #553, fiches 15-8a / 15-8b).
  **Propagation post-patch** : `21`, `90`, `17 autres`, `3 sites`, `trois sites`, `205`, `18 lignes`,
  `:909`, `:401`, `:307`, `:317`, `retry.rs:39`, `1343`, `3960`, `3946`, `1324`, `2065`, `2172`,
  `treize` grepés sur le corps de la fiche (hors Change Log) : restent les mentions historiques
  (relevé de `f289414e` pour comparaison, C66 « 17 routes au lieu de 18 », numéros de
  `f289414e` cités comme tels). **Décompte** (recompté, `grep -c`) : **6 AC, 6 tâches T0–T5** ;
  aucun test dynamique neuf ; mutations 5 lancées + 1 consignée sans exécution ; partition
  d'arrivée 22 + 4 + 89 = 115.
- 2026-10-08 — **Validation P6** (Opus ×2, contexte frais, lecture seule ; lentilles R et F ; prompt
  `15-5e2-validate-prompt-p6.md` ; rapports `target/gate-logs/15-5e2-p6-R.md` et `-F.md`), première
  passe après l'alignement C85. **Findings** : 0 CRITICAL, 0 HIGH, **2 MEDIUM** (R6-1, F6-1 —
  distincts), **15 LOW** (R6-2 à R6-7, F6-2 à F6-10 ; R6-2 = F6-2, R6-4 = F6-6, R6-1 recoupe F6-3).
  **Remédiations** (choix **C86**), toutes appliquées, aucune écartée :
  - **R6-1 MEDIUM** (= F6-3) : le contrôle `grep retry_with` de l'AC1 couvre aussi `crates/*/tests` ;
    six sites qui deviennent faux nommés (`journal_entry_reversal_e2e.rs:1609`,
    `journal_entries_modification.rs:441`, `audit_route_registry.rs:89` — limite (vi), nommée à
    côté de (iii bis) —, `:119`, `:165`, `:600`) ; occurrences légitimes sous `tests` listées
    (`capture_rejeu.rs:6`, `reconciliation_e2e.rs:4414`, sources synthétiques du banc).
  - **F6-1 MEDIUM** : **C56 révisé** — test 8 « route victime » sur `post_manual` avec projet
    (harnais de la 15-5e1 : la transaction de test tient la sentinelle `companies` puis demande
    l'exercice, la route tient l'exercice et attend la sentinelle), témoin
    `exiger_un_rejeu("reconciliation::manual")`, mutation « enveloppe retirée » → rouge ; T0 forme
    le cycle à la main sur MariaDB 10.11, et s'il ne se forme pas le test est écrit comme angle
    mort ; l'angle mort « aucune route `AppError` n'a de preuve dynamique » est retiré de l'AC2 et
    des Dev Notes, ramené à `post_split` et `complete_import`. Noms d'opération des trois routes
    `AppError` fixés.
  - **LOW** : R6-2 = F6-2 (commentaires `reconciliation.rs:873`, `:880` → `is_app_deadlock`) ; R6-3
    (`routes/journal_entries.rs:653`) ; R6-4 = F6-6 (forme fixée : ordre dans « Lock sequence »,
    paragraphe « Notes » sous la table, « diverges » reformulé) ; R6-5 (cinq sites migrés, six avec
    le `DELETE`, plus le prédicat de `post_accept` — T0, T1, Dev Notes) ; R6-6 (§ 10
    d'`api-external.md` aux conflits avec la 15-5d, phrases sous la table) ; R6-7
    (`journal_entries.rs:1683-1684` triée comme l'étape 0-bis) ; F6-4 (`ENVELOPPES` sans
    `retry_with`, `RETRY_WITH_AUTORISE`, banc du visiteur transposé et cas négatif) ; F6-5 (volet
    (c bis) `no_route_calls_retry_with_except_post_accept`, garde durable de la migration de
    `finalize`, et sa mutation) ; F6-7 (motif de l'inventaire étendu, sites hors motif nommés dont
    `fiscal_years.rs:12-18`) ; F6-8 (`generate_opening_balances` aux entrées clonées) ; F6-9 (AC4 :
    routes rejouées énumérées, phrase converse) ; F6-10 (rebase de la 15-8b après cette story,
    en-tête ; la consigne est à porter dans la fiche 15-8b par l'orchestrateur).
  **Signal de découpage (amendement D5)** : sévérité MEDIUM après une passe P5 à 0 au-dessus de LOW —
  déclaré au Project Lead. Constat : les deux MEDIUM ne recyclent aucun défaut antérieur ; R6-1 est
  **né de la remédiation C85** (les migrations qu'elle décide rendent fausses des mentions hors du
  périmètre de son contrôle), F6-1 conteste un choix (C56) dont la prémisse a changé avec la
  livraison du harnais. Pas de découpage : le périmètre ne change pas de nature.
  **Recompte** (commandes exécutées sur `HEAD` `96d7eccb`, code identique à `cecd5d1d`) : inventaire
  au symptôme, motif étendu, **385 lignes / 62 fichiers** (264 / 42 sous `src` et `docs`, 121 / 20
  sous `tests`) ; ancien motif 347 / 52 (inchangé) ; `grep -rn "retry_with" crates/*/tests` hors
  `rejeu_interblocage_e2e.rs` (qui n'en contient pas) : 21 lignes, dont les six sites nommés.
  **Décompte** (recompté sur la fiche, `grep -c`) : **6 AC, 6 tâches T0–T5** ; **2 tests neufs**
  (test 8 de `rejeu_interblocage_e2e.rs`, `no_route_calls_retry_with_except_post_accept`) et le banc
  du visiteur transposé ; mutations **6 lancées + 1 consignée** sans exécution ; partition
  d'arrivée 22 + 4 + 89 = 115 (inchangée).
  **Trend** : P1 **1 HIGH, 6 MEDIUM** → réécriture C54 → P2 **6 MEDIUM** → P3 **2 MEDIUM** →
  découpage C61 → P4 **5 MEDIUM** (4 distincts) → P5 **0** → alignement C85 → P6 **2 MEDIUM**,
  15 LOW. Modèles : Sonnet, Opus, Sonnet, Opus, Sonnet, **Opus** (P6). La boucle est rouverte ;
  **P7** sur la remédiation C86.
  **Propagation post-patch** : `six ou sept`, `347`, `232`, `115 lignes`, `Pas de test dynamique`,
  `aucun n'est exigé`, `angle mort`, `liste nommée`, `n'a pas d'autre garde`, `colonne « Mitigation »`,
  `is_deadlock_error` reste, `crates/*/src docs` grepés sur le corps de la fiche (hors Change Log) :
  restent la comparaison « l'ancien motif rendait 347 / 52 » et l'angle mort conditionnel du test 8.
- **2026-10-08 — Validation P7 ciblée (Haiku, une lentille, prompt `15-5e2-validate-prompt-p7-ciblee.md`) sur
  `b5ac76b4` : 0 finding**, axes exercés et non exercés déclarés. Rapport : `target/gate-logs/15-5e2-p7-ciblee.md`.
  Vérifié par l'orchestrateur sur le point le plus exposé (test 8) : la transaction concurrente du test est du SQL
  brut qui ne prend pas le verrou nommé `GET_LOCK` ; ce verrou ne sérialise que deux ROUTES entre elles et
  n'empêche donc pas le cycle route / test décrit. **Validation CLOSE.** Trend : P1 1 HIGH / 6 MEDIUM → … → P5 0
  → alignement sur le livré (C85) → P6 2 MEDIUM (Opus ×2) → P7 ciblée 0 (Haiku).
- 2026-10-08 — **Développement** (`bmad-dev-story`, Claude Opus 5.5 ; commit de code `6125d50b`,
  sur `688fed25` — `origin/main` `ec675288`, qui porte les 15-5e1, 15-8a et 15-8b). La 15-8b étant
  mergée : départ **10 / 12 / 4 / 89**, arrivée **22 / 0 / 4 / 89 = 115** ; **douze** routes rejouées
  (neuf `DbError`, trois `AppError` extraites en fonctions « une tentative ») et **six** sites
  `retry_with` migrés à équivalence exacte, `post_accept` seul à le garder. **T0** : cycle du test 8
  formé à la main sur MariaDB 10.11.16 (la route est la victime) — le test 8 est écrit en vert.
  Volet (c bis) et son banc ajoutés au registre ; commentaires d'ordre, Pattern 5, `api-external.md`
  § 10, CHANGELOG et manuels (#484) réécrits ; deux PDF régénérés, `Overfull` inchangés. Mutations :
  six lancées, toutes rouges comme attendu, une consignée sans exécution. Choix **C-15-5e2-1** à
  **C-15-5e2-6**. Gates au commit de code : voir T5 du Dev Agent Record. Statut → `review`.
  **Décompte** (recompté) : 6 AC, 6 tâches T0–T5 cochées ; 3 tests neufs (`main` → `6125d50b`) ;
  inventaire au symptôme 394 / 62 sur `HEAD` d'origine, 462 / 64 après.
- 2026-10-08 — **Revue de code P1** (Sonnet ×3 : B 6 LOW, E 5 LOW, A 1 MEDIUM / 4 LOW ; 15 findings,
  12 distincts après doublons L-1 = B-2 et L-3 = B-4 = A5). Remédiation au commit `ee77f450` :
  **A1 (MEDIUM)** doc-comments de `post_manual` et `post_split` « Rejouée sur interblocage » ;
  **L-1 = B-2** audit (`was_previously_rejected`, montant) lu dans la tentative, test 8 étendu,
  mutation rouge ; **B-3** `conclude_locked_attempt` factorise le `match` des deux tentatives ;
  **B-5** clone du pool retiré des cinq fermetures migrées ; **L-3 = B-4 = A5** angles morts du
  volet (c bis) écrits (registre, point (vii) ; Dev Notes) ; **A2, A3** décomptes recomptés (31
  fichiers, 959 insertions ; relevé 21 lignes) ; **A4** Pattern 5 renvoie à la règle ; **B-6**
  cosmétiques (`{settlementId}` gardé, convention du document). **LOW acceptés** : **B-1 = L-4**
  (pas de test dynamique pour `split` et `complete_import` — angle mort (iii bis) déjà écrit, forme
  identique à `manual` relue par deux lentilles) ; **L-2** (postabilité de la contrepartie non
  revérifiée sous verrou — préexistante, D-A0, le rejeu n'allonge la fenêtre que du backoff) ;
  **L-5** (écrivains hors journal victimes possibles : `PUT /invoices/{id}` avec changement de
  projet, archivage de projet, clôture et réouverture d'exercice — sans cycle démontré, nommés au
  point (iv) du registre ; issue d'amélioration à ouvrir). Choix **C-15-5e2-7**. Gates au commit
  `ee77f450` : backend 2837 / 2837, frontend 1086 / 1086, E2E 249 / 7 KF-029 / 17. Statut reste
  `review` jusqu'à la passe ciblée.
- **2026-10-08 — Revue de code P2 ciblée (Haiku, une lentille, prompt `15-5e2-review-prompt-p2-ciblee.md`) sur
  `ee77f450` : 0 finding.** Rapport : `target/gate-logs/15-5e2-review-p2-ciblee.md`. La lentille déclarant « aucun axe
  non exercé » sans rien avoir exécuté, l'orchestrateur a repris le point le plus fragile : dans
  `conclude_locked_attempt`, chaque branche d'erreur fait `rollback`, sauf `TransactionAborted` qui fait
  `drop(tx_outer)` — rollback implicite, identique aux trois anciennes branches (`git show ee77f450^`). **Boucle de
  revue CLOSE** (P1 Sonnet ×3 : 1 MEDIUM, 15 LOW → P2 ciblée Haiku : 0). Statut `done`.
