# Story 58-2 : Comptes désignés et tests Rust de revue — dix trous de couverture (#592, #732, #736, #1060, #1062, #1063, #1064, #1066, #1127, #1433)

Status: ready-for-dev (sous réserve de la validation P1)

*Fiche rédigée le 2026-11-27 (Sonnet 5.5, en autonomie), workflow `bmad-create-story`. Worktree `/home/gcorbaz/devel/kesh-58-2`, branche
`story/58-2-comptes-designes-tests-revue` partie de `epic/58-nettoyage` à `90192c7b3` (qui contient la v0.20.0, la 58-1 et la 58-3). Fichier d'epic :
`_bmad-output/planning-artifacts/epic-58-nettoyage.md` (§ *Story 58-2*). Epic 58 (nettoyage), release visée v0.21.0 (numéro attribué au tag). **Aucune migration,
aucun code de production, aucun changement d'API, aucun texte visible** : la story n'ajoute que des tests (et la correction d'un verrou de test) ; aucune
compilation ni aucun gate n'a été lancé pour écrire la fiche, qui ne modifie que ce fichier, le prompt de validation et `sprint-status.yaml`.*

## Story

En tant que mainteneur de Kesh, je veux que dix gardes que la revue de code avait relevées comme « non tenues par un test » le soient — la clé i18n de chaque motif
d'écran de modification bloquée, le type en mots des refus de comptes désignés, le même compte faux pour deux réglages, l'avoir qui reste possible sur une créance
retypée, l'ordre de la version périmée et du parent d'un compte, les bornes d'une regex de date, un verrou de test oublié —, afin qu'un refactor ou une mutation qui
défait l'une de ces gardes fasse rougir un test, et non qu'on le découvre en production.

## Contrôle de couverture (règle du 2026-11-26 : on vérifie qu'une issue n'est pas déjà couverte avant de la retenir)

Chaque issue a été relue au code de `90192c7b3` (`gh issue view`, lecture seule) ; le test cité comme « manquant » a été cherché (`grep -rn`) dans le fichier nommé,
dans le binaire voisin et dans les tests unitaires du module. **Aucune des dix issues n'est entièrement réglée** : aucune ne sort de la story. Deux sont **partiellement**
couvertes (#732-1 et #1433-2 : la fiche ne retient que le reste) ; les lignes de plusieurs issues ont bougé (l'écart est écrit).

| # | Constat relu au code | Verdict |
|---|---|---|
| #592 | `blocker-messages.test.ts` mocke `i18nMsg` pour **rendre le repli** (`i18nMsg: (_key, fallback, args) => …`) : aucune assertion ne lit la clé. `i18n-keys.test.ts` (garde B) vérifie qu'une clé demandée **existe** au catalogue, jamais **laquelle** un code emploie. | **gardée** ; le compte a changé (voir § *Ce qui se lit au code*) |
| #732 | (1) `textes_coherents.rs:355-366` : positifs `1171-99-99`, `-13-01`, `-00-10`, `-12-32` ; ni le jour `00`, ni les mois `10`/`11`, ni le garde `(\D\|$)`. (2) Aucun test ne tient la séquence 1171, 2200, 2206, 1100, 3000, 2000, 6940, 3800, 6900, 3805 (aucun `6940`/`3805` dans `crates/*/tests` hors `invoices_validate_vat.rs` et `test_fixtures.rs`, pour d'autres usages). (3) `designate_vat_*` : une seule société, `version` comparée seulement entre t2 et t3. (4) `fiscal-years-page.test.ts:142-189` : « trois clos » et « un clos suivi d'un ouvert », pas « clos, ouvert, clos ». | **gardée**, les quatre points ; le point 1 est **à moitié couvert** (mois `00`/`13`, jour `32` le sont) ; le point 4 est un test **frontend**, non Rust (le fichier d'epic dit « un test frontend d'i18n » : voir D2) |
| #736 | `i18n_e2e.rs:126-128` (`i18n_messages_endpoint_returns_locale_and_messages`) et `:151-153` (`i18n_messages_requires_auth`) appellent `spawn_app` sans `get_i18n_lock()` ; `grep -c 'get_i18n_lock().await'` = **4** pour **6** tests `#[sqlx::test]`. | **gardée** (lignes 126/151, non 128/153 de l'issue) |
| #1060 | `designated_account_type_e2e.rs:527` est le **seul** `type de compte incompatible (attendu : …)` asserté (« actif »). `account_type_in_words` (`routes/company_invoice_settings.rs:220-227`) a quatre bras ; « passif », « charge ou produit » ne sont asserté nulle part dans `crates/kesh-api/tests`. | **gardée** |
| #1062 | Aucun test ne monte un compte faux pour deux réglages ; `WrongTypeDesignatedAccounts::new` (`kesh-db/src/errors.rs:984-1000`) dédoublonne par `(identifiant, réglage)` et trie par numéro, identifiant, **réglage** — la garde existe, seul le test manque. | **gardée** |
| #1063 | `ac7_purchase_validation_orders_the_accounts_by_number` (l. 322-387) rend 400 et s'arrête : pas de `GET` de la facture fournisseur ensuite. La vente le fait (`ac7_sale_validation_names_the_retyped_receivable`, l. 252-281 : « la facture reste brouillon »). | **gardée** |
| #1064 | Aucun test de `POST /api/v1/credit-notes` sur une créance retypée (`grep -rn retype` dans les tests d'avoir : vide). Le manuel utilisateur (« Rôles des comptes ») le promet. | **gardée** |
| #1066 | `supplier_invoices_repository.rs:2691-2720` (`ac4_payable_retyped_asset_is_refused`, à la ligne 2536 dans l'issue) : `update_account(Liability, Some(Payable))`, puis `update_account(Asset, Some(Payable))`, **une** assertion du rôle, après le second. | **gardée** |
| #1127 | `jour_suisse_nuit_e2e.rs` : `ac2_solde_d_ouverture_la_nuit_du_nouvel_an` (l. 539), `ac3_*` (l. 615-735) ; la 35-5a a relevé M3 sur `journal_entries.rs` seulement et M7 sur `invoice_settlements_write.rs` seulement (fiche 35-5a, Change Log, **A4**). | **gardée** ; c'est une campagne de **mutations à jouer**, non un test à écrire (D4) |
| #1433 | `accounts.rs` : la version se compare à la ligne **1224** (`Some(a) if a.version != version => … OptimisticLockConflict`), avant `guard_parent_change_in_tx` (ligne **1266**) — non 1126/932 de l'issue. `put_parent` (`accounts_e2e.rs:1185`) relit toujours la version fraîche ; `le_conflit_de_version_precede_la_garde_de_retypage` (`accounts.rs:4062`) tient l'ordre pour le **retypage**, pas pour `parentId`. « Absent » : `accounts_e2e.rs:1249` ne l'asserte qu'**sans autre champ modifié** (no-op). | **gardée**, les deux cas ; le cas (2) est à moitié couvert |

Aucune des dix ne révèle une migration, un changement d'API ou un défaut de production. Si l'une des mutations de #1127 survit à tout test existant, le test manquant s'écrit
ici ; si elle révèle un défaut de production, il s'ouvre en issue (§ *Issues à ouvrir*), il ne se corrige pas dans cette story.

## Ce qui se lit au code (`90192c7b3`)

### #592 (P3) — l'identité de la clé i18n des motifs d'écran
- `frontend/src/lib/features/journal-entries/blocker-messages.ts` : `modificationBlockerLabel` (l. 85-130) a **13 codes** (`ATTENDU`, `blocker-messages.test.ts:27`), dont **6 clés propres**
  `journal-entries-modify-blocked-{fiscal-year-closed, later-fiscal-year-closed, period-locked, detached-settlement, fx-revaluation, lettered}` et 7 renvois à `reversalBlockerLabel`
  (8 clés `journal-entries-reverse-blocked-*` : `is-a-reversal`, `already-reversed`, `invoice`, `credit-note`, `supplier-invoice`, `settlement`, `bank-match`, `account-archived`,
  la dernière propre à la contre-passation). L'issue dit « douze branches » : le compte était juste à la 15-1a-ii ; **à ce jour** : 13 codes de modification, 14 clés distinctes
  atteintes par les deux `switch` (6 + 8), plus un **site de plus** : `JournalEntryForm.svelte:687` répète la clé `…-fiscal-year-closed` pour le toast de l'édition périmée.
- Le mock de `blocker-messages.test.ts:17` rend `fallback` : changer la clé d'un bras ne rougit rien (mutation F2 de la 15-1a-ii, survivante).
- Les quatre catalogues : `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (fr-CH : l. 460-467 et 480-485, et l. 4442 pour `…-fx-revaluation`). La garde B
  (`i18n-keys.test.ts`) tient « la clé existe au catalogue » ; elle ne dit pas « le bras `LATER_FISCAL_YEAR_CLOSED` emploie `…-later-fiscal-year-closed` ».

### #732 (P3) — quatre trous
1. `textes_coherents.rs:285` `DATE_ISO = ^-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])(\D|$)` ; test `nombres_isoles_ne_prend_pour_date_que_des_mois_et_jours_bornes` (l. 340-366).
   Mutations survivantes : `0[1-9]` → `0\d` (jour), `1[0-2]` → `1[02]` ou `1[12]` (mois), retrait de `(\D|$)`.
2. `lock_vat_role_accounts_in_tx` (`company_invoice_settings.rs:2028`) verrouille `VatRecoverable`, `VatPayable`, `VatSettlement` ; `insert_with_defaults_in_tx` verrouille `Receivable`,
   `DefaultRevenue`, `Payable`, puis `chart_designated_accounts` (l. 674 : arrondi, puis `WriteOffNature::ALL`). Au plan PME : 1171, 2200, 2206, 1100, 3000, 2000, 6940, 3800, 6900, 3805
   (doc-comment, juste avant la l. 2028 ; recoupée par la revue 32-1a, `321a-review-p1-B.md`, B2). Aucun test ne la tient.
3. `designate_vat_accounts_in_tx` (l. 2086) fait `UPDATE … WHERE company_id = ?` ; `designate_vat_never_overwrites_a_set_value` et `designate_vat_fills_nulls_and_is_idempotent`
   (`company_invoice_settings_repository.rs:1865`, `:1916`) ne montent **qu'une** société (`pme_company_with_chart`, l. 1649) ; `version` n'est lue qu'**après** t2 (`apres_t2`) et t3.
4. `fiscal-years-page.test.ts` : le cas « A clos, B ouvert, C clos » n'existe pas ; l'issue relève (revue 32-1a, E3) qu'il est **étroit** (la mutation « ne regarder que le suivant » est
   déjà tuée par « trois clos »).

### #736 (P3) — verrou i18n oublié
`i18n_e2e.rs:27` `get_i18n_lock()` ; le commentaire l. 83 l'exige de tout appelant de `spawn_app_with_locale` (qui appelle `init_error_i18n`, global). Sans lui, sous `cargo test` parallèle,
les deux tests peuvent remettre `fr-CH` pendant qu'un test allemand attend un message : `error_messages_in_german_when_locale_de` (l. 186), `backup_schedule_out_of_domain_refusal_in_german`
(l. 212) et `mcp_settings_host_bits_refusal_in_german` (l. 265) peuvent rougir sans cause. Aucun effet sous nextest ni en CI (`--test-threads=1`).

### #1060, #1062, #1063, #1064 (P3) — la 35-3, au fichier `designated_account_type_e2e.rs`
- `validate_account_of` (`routes/company_invoice_settings.rs:241-282`) formule `"{field_label} : type de compte incompatible (attendu : {attendus.join(" ou ")})"` ; libellés (l. 410-506) :
  `Compte créance`, `Compte produit`, `Compte TVA due`, `Compte TVA récupérable`, `Compte décompte TVA`, `Compte créanciers`, `Compte de différences d'arrondi`, `Compte d'escompte`,
  `Compte de frais bancaires`, `Compte de pertes sur créances`. Les quatre champs de la boucle de `ac9_…` (l. 461-529) attendent : créance → actif, créanciers → passif, TVA due → passif,
  TVA récupérable → actif ; le compte d'arrondi (`resolve_designated_account`, l. 291-311, `[Expense, Revenue]`) → « charge ou produit ». Mutations survivantes : `Liability => "actif"`, `Expense => "produit"`.
- `WrongTypeDesignatedAccounts::new` : tri `(account_number, account_id, setting)`, dédoublonnage `(account_id, setting)` ; `DesignatedSetting` (`errors.rs:918-927`) est ordonné
  `Receivable < VatPayable < Payable < VatRecoverable`. La vente juge créance et TVA due ; l'achat, créanciers et TVA récupérable.
- La facture fournisseur : `SupplierInvoiceResponse` porte `status` et `internal_number: Option<String>` (`routes/supplier_invoices.rs:78-79`, `camelCase`) ; `GET /api/v1/supplier-invoices/{id}`.
- L'avoir : `POST /api/v1/credit-notes` `{ "invoiceId", "date" }` (`invoice_echeancier_e2e.rs:751-758`). Le manuel utilisateur (« Rôles des comptes ») : « l'avoir n'est pas soumis à ce contrôle, pour qu'une facture émise reste toujours annulable ».

### #1066 (P4) — `ac4_payable_retyped_asset_is_refused`
Le premier `update_account(Liability, Some(AccountRole::Payable))` pose le rôle ; le second retype en `Asset` ; l'assertion du rôle n'est qu'après. La fiche de la 35-3 (AC4) veut le rôle vu **posé avant** le retypage.

### #1127 (P4) — mutations de la 35-5a
Fiche `35-5a-date-du-jour-horloge.md` : AC2 (solde d'ouverture, 3 sites : `bank_accounts.rs`, `opening_balances.rs` complément et statut), AC3 (contre-passation et quatre annulations : règlement client,
facture fournisseur, règlement fournisseur, rapprochement). Les sites d'AC2 et d'AC3 se relèvent à l'**étape 0** par `grep -n 'horloge.aujourd_hui()'` sur `kesh-api/src/routes/{journal_entries,bank_accounts,opening_balances,
supplier_invoices,reconciliation,invoices}.rs` et `kesh-db/src/repositories/{invoice_settlements_write,supplier_invoices,reconciliation_cancel,settlement_cancellation}.rs` ; ils ne se recopient pas ici (ils ont bougé).

### #1433 (P3) — `PUT /accounts/{id}` avec `parentId`
`accounts::update` (l. 1149) : introuvable/archivé → **version** (l. 1224) → no-op → … → `guard_parent_change_in_tx` (l. 1266). Test existant de l'ordre : `le_conflit_de_version_precede_la_garde_de_retypage` (`accounts.rs:4062`, `kesh-db`),
pour le retypage seulement. HTTP : `put_parent` (`accounts_e2e.rs:1185`) lit la version courante et laisse `extra` la surcharger (`body[k] = v`) : une version périmée s'y passe par `extra`.

## Décisions

- **D1 — Aucun code de production, pas même un commentaire.** Les dix issues sont des tests. La story ne touche pas `company_invoice_settings.rs` (donc pas le doc-comment de la séquence des verrous,
  #732-2), ni `routes/`, ni `accounts.rs`. *Option écartée* : corriger le doc-comment (« au plan PME actuel ») au lieu du test — il toucherait un **repository** de `kesh-db` et déclencherait
  l'exception « gate complet » du `CLAUDE.md` pour une phrase ; et la revue 32-1a demandait un test (« qu'aucun test ne tient »).
- **D2 — Les points frontend restent dans la story.** #592 et #732-4 sont des tests Vitest. Le fichier d'epic parle d'« un test frontend d'i18n » (#592) ; #732-4 en est un second, de la même
  nature (un fichier de test de la page des exercices). Le **module** de la story au sens du `CLAUDE.md` (code de production qui change) reste **zéro** ; au sens des fichiers de test : `kesh-api/tests`,
  `kesh-db/tests`, `frontend/src/lib/features/journal-entries`, `frontend/src/routes/(app)/settings/fiscal-years` (quatre, sous la borne de cinq). *Option écartée* : sortir #732-4 vers la 58-6 (tests
  frontend de la réconciliation) — elle ne touche pas ce domaine.
- **D3 — #592 : la clé s'asserte par un mock qui rend la clé, et l'existence aux quatre catalogues se relit à la source.** Un **nouveau** fichier `blocker-messages-keys.test.ts` (même dossier) mocke
  `i18nMsg: (key) => key` : `modificationBlockerLabel(code, label)` rend alors la **clé** ; une table `code → clé` écrite **en littéraux dans le test** (13 lignes pour la modification, 8 pour la contre-passation)
  est comparée, puis chaque clé est cherchée dans les quatre `messages.ftl` (ligne `^<clé> =`, lecture `readFileSync('../crates/kesh-i18n/locales/<locale>/messages.ftl')` comme `i18n-keys.test.ts`).
  Le site du formulaire (`JournalEntryForm.svelte:687`) se tient dans `JournalEntryForm.edit.test.ts` (cas `FISCAL_YEAR_CLOSED`, l. 202) **sans changer l'assertion d'aucun autre cas** : le mock du fichier
  devient un espion qui garde la dernière clé. *Options écartées* : (a) modifier le mock de `blocker-messages.test.ts` (il tient les phrases par leur repli : deux rôles dans un fichier) ; (b) extraire les clés du source
  par regex (**les deux côtés du même extracteur** : « vrai par construction », `CLAUDE.md`, *Inventorier les sites NON RÉSOLUS*). La table du test est écrite à la main, c'est ce qui la rend indépendante.
  Le périmètre inclut les huit clés de la contre-passation (même fichier, même mutation F2, coût marginal) ; l'issue ne parlait que des motifs de modification : élargissement **déclaré** (Q1).
- **D4 — #1127 : une mutation par site, jouée, journalisée ; pas de test écrit d'avance.** Le « correctif » de l'issue est une **exécution** (jouer M3 et M7 sur chaque site restant et nommer l'assertion
  qui la tue). Pas de commit de production : chaque mutation se pose sur la copie de travail, se joue en **local** (`scripts/mem-guard.sh`, ciblé `binary(jour_suisse_nuit_e2e)`, sous `flock .gate.lock`), puis
  se restaure par `git checkout -- <fichier>` **et `touch`** (cargo garderait le binaire muté), `git diff --stat -- crates/*/src/*` vide avant chaque commit (pathspec : `crates/*/src/*`, non `crates/*/src`). Le journal
  (`kesh-gate-logs/e582/mutations-1127.md`, hors dépôt) a une ligne par site : fichier:ligne, mutation, test qui la tue (nom). **Si une mutation survit** : un test s'ajoute à `jour_suisse_nuit_e2e.rs`
  (le site devient assertable à l'horloge fixée), rejoué rouge sous la mutation, vert ensuite ; si le site est **mort** (aucun chemin n'y mène, ou la mutation est sans effet observable à `2025-12-31T23:30:00Z`),
  il s'écrit **angle mort assumé** avec sa raison (jamais « raisonné »). *Option écartée* : écrire un test par site sans jouer la mutation (c'est le défaut même que l'issue décrit : couvert « par lecture »).
- **D5 — #732-2 : le test tient le plan, pas l'ordre SQL ; limite écrite.** Aucun moyen portable de lire l'ordre réel des verrous InnoDB à travers MariaDB (à notre connaissance, MariaDB n'expose pas `performance_schema.data_locks` ;
  un journal général est global ; à confirmer à l'étape 0). Le test monte un plan PME, résout **par rôle** les trois premiers, puis les trois suivants, puis le compte d'arrondi, puis les natures **dans l'ordre de `WriteOffNature::ALL`** (la constante,
  non recopiée), et assere que la suite des **numéros** égale le littéral `["1171","2200","2206","1100","3000","2000","6940","3800","6900","3805"]`, écrit en dur dans le test et cité du doc-comment. Un plan PME
  modifié, un rôle réattribué, un `WriteOffNature::ALL` réordonné, rougissent ; **l'ordre des appels dans `lock_usable_role_account` / `insert_with_defaults_in_tx` reste hors du test** (angle mort assumé, écrit dans le doc-comment du test).
  *Option écartée* : réduire le doc-comment à « l'ordre des rôles, sans numéros » (D1 : production) ; *option écartée* : instrumenter une transaction concurrente pour deviner l'ordre (course, non déterministe).
- **D6 — #1062 : le compte faux pour deux réglages se monte sans retypage.** Un compte de charge **sans rôle** créé par `new_account` (`designated_account_type_e2e.rs:192`) et posé par SQL aux deux réglages que la vente juge
  (`default_receivable_account_id`, `default_vat_payable_account_id`) : le retypage d'un compte à rôle vers `Expense` est refusé par `accounts::update` (le rôle ne l'admet pas), et le test vise la validation, non le
  retypage. L'attendu : `rejected` = deux entrées, même numéro, **dans l'ordre des réglages** (`receivable` puis `vatPayable`, l'ordre de `DesignatedSetting`), le message porte deux phrases « Le compte NNNN, désigné comme … » et
  **une seule** phrase d'action. Si l'étape 0 montre que la validation lit le rôle et non le réglage, le montage se réécrit et la décision se corrige au Dev Agent Record.
- **D7 — #1064 : l'avoir sur la créance retypée seule ; la TVA due retypée est une variante, retenue seulement si le code la laisse passer.** Le manuel dit que l'avoir « relit la TVA due dans les réglages … sans la contrôler » :
  le second assert (TVA due retypée en actif, avoir 201) entre dans le même test **s'il passe** ; sinon l'écart est un **défaut de production ou de manuel**, ouvert en issue (§ *Issues à ouvrir*), et le test garde la seule créance.
- **D8 — #736 : le verrou se prend en tête des deux tests**, comme les quatre autres (`let _guard = get_i18n_lock().await;`). *Option écartée* : le prendre dans `spawn_app` (il faudrait rendre le verrou avec `TestApp` :
  la durée de vie du garde et le `TestApp` sont liés, changement de l'API des helpers pour un oubli de deux lignes).

## Critères d'acceptation

Chaque AC se prouve par un **avant** (le trou existe : une mutation survit, un compte, une ligne) et un **après** (la mutation rougit, le compte change). Les lignes sont celles de `90192c7b3` ; le développeur les
retrouve **par la chaîne**.

- **AC1 (#592)** — Un test Vitest tient, pour chacun des 13 codes de `modificationBlockerLabel` et des 8 de `reversalBlockerLabel` (21 lignes de table, 14 clés distinctes), la clé que le bras emploie (table littérale code → clé) et, pour chaque clé, sa présence
  dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`. Mutation **F2** (la clé d'un bras remplacée par celle d'un autre bras) : survit **avant**, rougit **après** ; jouée sur **chaque** bras distinct (14 : 6 clés propres + 8 de la contre-passation ; un journal), et sur
  le site de `JournalEntryForm.svelte:687`. Les assertions existantes (phrases) ne changent pas.
- **AC2 (#732-1)** — `nombres_isoles_ne_prend_pour_date_que_des_mois_et_jours_bornes` porte en plus : négatifs `2026-10-15`, `2026-11-30`, `2026-01-01T10:00` (aucun nombre rendu) ; positifs `1171-12-00`, `1171-01-011` (le nombre
  `1171` est rendu). Les trois mutations (`0[1-9]`→`0\d` au jour, `1[0-2]`→`1[02]` et `→1[12]` au mois, retrait de `(\D|$)`) survivent **avant**, rougissent **après**.
- **AC3 (#732-2)** — Un test de `kesh-db` (`company_invoice_settings_repository.rs`) tient la suite de numéros de D5 contre le littéral ; trois mutations jouées dans la copie de travail (le plan : un numéro de rôle changé dans le preset de
  test ; `WriteOffNature::ALL` réordonné ; le rôle `DefaultRevenue` remplacé) rougissent ; l'angle mort (l'ordre des appels) est écrit au doc-comment du **test**.
- **AC4 (#732-3)** — Deux sociétés de plan PME : la désignation de A laisse les trois colonnes de B à `NULL` et sa `version` intacte ; la `version` de A est lue **avant** t2 et **après** (+1 attendu seulement si le code la
  fait bouger : à lire à l'étape 0, l'attendu **s'écrit tel que le code le donne** et se cite). Mutation : retrait de `WHERE company_id = ?` (`company_invoice_settings.rs:2086`) survit **avant**, rougit **après**.
- **AC5 (#732-4)** — Un cas `fiscal-years-page.test.ts` « clos, ouvert, clos » : pour A, le bouton Réouvrir est désactivé et l'infobulle (bouton **et** enveloppe) nomme **C**, non B ; B (ouvert) n'a pas de bouton Réouvrir ;
  C est rouvrable sans infobulle. Mutation « ne regarder que l'exercice immédiatement suivant » : tuée **aussi** par ce cas (elle l'était déjà par « trois clos » : l'AC le dit, sans prétendre à un trou neuf).
- **AC6 (#736)** — `grep -c 'get_i18n_lock().await' crates/kesh-api/tests/i18n_e2e.rs` : **4 → 6** ; chacun des six tests `#[sqlx::test]` du fichier prend le verrou avant son premier `spawn_app*`. Pas de preuve du rouge
  déterministe (flake de course) : la preuve est le **comptage** plus la lecture des six têtes de test, citées au Dev Agent Record ; une boucle de rejeu (`--test-threads=8`) n'est pas une preuve.
- **AC7 (#1060)** — Les quatre refus de la boucle de `ac9_…` (créance, créanciers, TVA due, TVA récupérable) assertent le message **entier** `"<libellé> : type de compte incompatible (attendu : actif|passif)"` ; un cinquième cas
  pose un compte d'actif au `defaultRoundingAccountId` : 400 `"Compte de différences d'arrondi : type de compte incompatible (attendu : charge ou produit)"`. `grep -c 'type de compte incompatible (attendu : ' designated_account_type_e2e.rs` :
  **1 → ≥ 5**. Mutations `Liability => "actif"`, `Expense => "produit"`, `Revenue => "charge"` (la jointure `" ou "` change l'ordre) : toutes rougissent **après**, aucune **avant** sauf celle de `Asset`.
- **AC8 (#1062)** — Un test `ac7_sale_validation_names_the_same_account_wrong_for_two_settings` : `details.rejected` = `[(NNNN, receivable, Asset, Expense), (NNNN, vatPayable, Liability, Expense)]` (D6) ; deux phrases « Le compte NNNN, … » ;
  la phrase d'action **une fois** ; la facture reste brouillon. Mutation : retirer `.then(a.setting.cmp(&b.setting))` ou changer la clé de `dedup_by` pour le seul `account_id` (`errors.rs:987-995`) rougit **après**.
- **AC9 (#1063)** — Après le 400 de `ac7_purchase_validation_orders_the_accounts_by_number`, un `GET /api/v1/supplier-invoices/{id}` : `status == "draft"`, `internalNumber` nul, `version` égale à celle d'avant la validation.
  Mutation : la validation fournisseur qui attribuerait le numéro interne avant de refuser rougit **après** (jouée sur la copie de travail ; si aucune mutation vraisemblable n'existe, l'AC le dit au Dev Agent Record, sans prétendre au rouge).
- **AC10 (#1064)** — Un test : facture validée, créance `1100` retypée en passif par l'API, `POST /api/v1/credit-notes` → **201**, l'écriture inverse crédite `1100` (lue sur l'écriture de la facture) ; la TVA due retypée (D7) si le code le permet.
  Mutation : un contrôle de type de la créance ajouté à `create_credit_note` (copie) rend 400 ⇒ rouge **après**.
- **AC11 (#1066)** — Entre les deux `update_account` de `ac4_payable_retyped_asset_is_refused`, l'assertion que le rôle vaut `Some(AccountRole::Payable)` **et** que le type vaut `Liability` ; après le second, l'assertion existante.
  `grep -c 'role, Some(AccountRole::Payable)' supplier_invoices_repository.rs` augmente d'**1** dans ce test.
- **AC12 (#1127)** — Le journal `kesh-gate-logs/e582/mutations-1127.md` liste **chaque** site restant d'AC2 et d'AC3 de la 35-5a, relevé à l'étape 0 par le `grep` de § *Ce qui se lit au code*, avec M3 (AC2/AC3 : `Utc::now()` pour
  `state.horloge.aujourd_hui()`) **et** M7 (annulations : horloge réelle au lieu de la date du bloqueur), la sortie de nextest citée (le **nom** du test rouge), la restauration (`git diff --stat` vide, `touch`). Le **compte** des sites
  du journal égale celui du `grep` (recomptage depuis la source). Une survivante → un test (D4) ; un site mort → angle mort écrit.
- **AC13 (#1433)** — Deux tests de `accounts_e2e.rs` : (a) `PUT /accounts/{id}` avec `parentId` valide **et** `version` périmée → **409 `OPTIMISTIC_LOCK_CONFLICT`**, parent **relu inchangé** (en base) ; (b) `PUT` sans `parentId` mais avec un autre champ
  modifié (le nom) → 200, nom changé, `parentId` **inchangé** (compte rangé sous un groupe au préalable, `version` + 1). Mutation : `guard_parent_change_in_tx` avant le contrôle de version (`accounts.rs:1224`/`1266`) rougit (a) **après** ; mutation :
  `parent` traité comme `Some(None)` quand absent rougit (b) **après**.
- **AC14 (transverse)** — (a) `git diff --name-only 90192c7b3 HEAD` ne nomme que : `crates/kesh-api/tests/{textes_coherents,i18n_e2e,designated_account_type_e2e,accounts_e2e,jour_suisse_nuit_e2e}.rs` (le dernier **seulement** si D4 l'impose),
  `crates/kesh-db/tests/company_invoice_settings_repository.rs`, `crates/kesh-db/tests/supplier_invoices_repository.rs`, `frontend/src/lib/features/journal-entries/{blocker-messages-keys.test.ts,JournalEntryForm.edit.test.ts}`,
  `frontend/src/routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts`, `_bmad-output/implementation-artifacts/58-2-*`, `sprint-status.yaml`, ce fichier (+ prompts). **Aucun** fichier de `crates/*/src/*`, aucun catalogue, aucun manuel.
  (b) `git diff --stat 90192c7b3 HEAD -- 'crates/*/src/*' docs/ frontend/src/lib/i18n` **vide** (sonde positive : la même commande sur `HEAD~0` d'une branche qui touche `crates/kesh-db/src/` rend des lignes). (c) `sitesTotal` de `i18n-keys.test.ts` **inchangé**
  (le nouveau test ne contient pas d'appel `i18nMsg` de production ; il mocke `i18nMsg` : à vérifier au moissonneur, `npm run test:unit -- i18n-keys`).

## Preuve du rouge

Un test de garde est rouge **par mutation**, non par absence : pour chaque AC, le développeur écrit le test, le joue **vert** sur le code actuel, joue la mutation nommée (copie de travail, restauration + `touch`), constate le **rouge**
et relève le nom de l'assertion qui rougit (journal `kesh-gate-logs/e582/mutations.md`, hors dépôt : AC, mutation, test, sortie nextest/Vitest). Les AC sans mutation possible sont dits (AC6 : comptage ; AC12 : le journal **est** la preuve).
Un test qui reste vert sous sa mutation est **retiré ou récrit** avant commit : « le test passe » ne prouve rien (`tests-qui-prouvent-moins`). Le journal compte les mutations (**recompté** depuis le fichier, avec son périmètre) ;
la fiche n'en annonce pas le nombre d'avance.

## Tâches

0. **Étape 0** : relire les lignes citées **par la chaîne** ; relever les sites d'AC12 (`grep`) ; vérifier D6 (la validation lit-elle le réglage ou le rôle ?), D7 (l'avoir sur la TVA due retypée passe-t-il ?), la table de D5 (`WriteOffNature::ALL`, ordre des natures
   au plan PME), l'attendu de version d'AC4 ; relever, pour chaque AC, la **mutation** et le **test qui la tue**. Recontrôler les corrections des validations (ligne lue citée au Dev Agent Record).
1. AC6, AC11 (une ligne chacun) ; AC7 à AC10 (`designated_account_type_e2e.rs`, `supplier_invoices_repository.rs`) ; AC13 (`accounts_e2e.rs`) ; AC2 (`textes_coherents.rs`) ; AC3, AC4 (`company_invoice_settings_repository.rs`).
2. AC1 et AC5 (frontend) ; AC12 (campagne de mutations, **en dernier**, sur un arbre propre).
3. Gates (§ *Gates*), recette (sans objet), Dev Agent Record, Notes de version à l'orchestrateur.

Un commit par étape franchie (`test(58-2): …`, `refs #N` des issues touchées, aucun mot-clé de fermeture) et **push de la seule branche** `story/58-2-comptes-designes-tests-revue` à chaque étape (règle des agents du `CLAUDE.md`).

## Gates

- **Les gates complets se jouent sur le serveur idefix** : `/home/gcorbaz/devel/kesh-gate-logs/serveur/gate-distant.sh story/58-2-comptes-designes-tests-revue <gate>` (`test-fast`, `gate-ci`, `front`, `e2e`, `tout`), **après push**
  de la branche (idefix la lit sur GitHub) ; les journaux reviennent dans `kesh-gate-logs/idefix/<id>/`. Le story file n'affirme que ce qui a tourné (`CLAUDE.md`, *Pendant une boucle de revue*).
- **Entre deux commits** : gate ciblé en local sous `scripts/mem-guard.sh` : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, puis `cargo nextest run -E 'binary(<binaire touché>)'` :
  `textes_coherents`, `i18n_e2e`, `designated_account_type_e2e`, `accounts_e2e`, `jour_suisse_nuit_e2e` (kesh-api) ; `company_invoice_settings_repository`, `supplier_invoices_repository` (kesh-db) ; Vitest ciblé
  (`npx vitest run blocker-messages-keys JournalEntryForm.edit fiscal-years-page`), `npm run check`, `npm run lint-i18n-ownership`.
- **Exception `kesh-db`** : la story ne touche **ni migration, ni `post_restore.rs`, ni repository** (les fichiers de `kesh-db/tests` ne sont pas un repository) : le gate complet n'est donc pas exigé en cours de boucle ; il l'est **au dernier commit**
  (`tout` sur idefix : `test-fast`, `gate-ci`, `front`, `e2e`) puisque des tests de `kesh-db`, `kesh-api` et du frontend changent.
- **Recette visuelle** : sans objet (aucun contrôle visible, aucun texte d'écran). Captures : aucune.
- **Mutations** (AC12 et AC1 à AC13) : en **local**, jamais sur idefix (gate-distant joue la branche poussée, jamais une copie mutée) ; aucun fichier de production muté n'est commité ni poussé.

## Note de version

*(Pour le CHANGELOG de la release du vendredi 2026-12-04, rubrique « Interne » ou « Tests », assemblée à l'intégration ; la story n'écrit pas le CHANGELOG.)*

**Tests** — Dix gardes de la revue de code sont désormais tenues par un test : la clé i18n de chaque motif d'écran de modification bloquée, le type attendu en mots des refus de comptes désignés (actif, passif, charge ou produit),
le même compte faux pour deux réglages, l'avoir possible sur une créance retypée, l'ordre de la version périmée et du déplacement d'un compte, les bornes de la reconnaissance d'une date dans la garde des textes, et le verrou
de locale de deux tests d'i18n. Aucun changement de comportement. (refs #592, refs #732, refs #736, refs #1060, refs #1062, refs #1063, refs #1064, refs #1066, refs #1127, refs #1433)

## Questions (pour l'orchestrateur et le Project Lead)

- **Q1 — #592 : l'élargissement aux huit clés de la contre-passation est-il admis ?** L'issue ne parle que des motifs de modification ; les deux `switch` partagent un fichier, une mutation (F2) et sept bras. *Proposition* : oui (D3).
- **Q2 — #1127 : une mutation jouée en local sur la copie de travail est-elle admise ?** Le `CLAUDE.md` demande un gate ciblé local ; il interdit à un agent d'écrire hors de son worktree, non d'y muter, avec restauration. *Proposition* : oui, journal hors dépôt.
- **Q3 — #732-2 : la limite de D5 (le test tient le plan, non l'ordre SQL) suffit-elle à fermer le point ?** *Proposition* : oui, et la fermeture de #732 attend les **quatre** points (une issue, une release).
- **Q4 — Si D7 révèle que l'avoir contrôle la TVA due retypée** (contre le manuel), qui tranche : corriger le code (P1/P2 d'un avoir impossible sur une facture émise) ou le manuel ? *Proposition* : issue distincte, priorité au Project Lead.

## Dev Notes

### Modules touchés (règle de splitting)
Code de production : **aucun**. Fichiers de test : quatre ensembles (D2), borne de cinq respectée. Pas de dérogation à la règle de splitting. Signal de recyclage : sans objet (première passe).

### Fichiers communs
- `crates/kesh-api/tests/*` : seule la 58-2 de l'Epic 58 (fichier d'epic § *Fichiers et modules touchés*). Epic 53 (écritures justes, TVA) : `designated_account_type_e2e.rs` côtoie la 53-2 ; second arrivé résout, deux côtés gardés.
- `frontend/src/lib/features/journal-entries/*` : 58-4 (textes d'interface, `JournalEntryForm.svelte`) et 58-5 : **le nouveau fichier de test** ne recoupe aucun fichier existant ; `JournalEntryForm.edit.test.ts` peut être touché par une autre story : reprendre `epic/58-nettoyage` avant le dernier commit.
- `sprint-status.yaml` : un bloc par story (conflit : garder les deux côtés).

### Ce qui doit rester vrai
Les assertions existantes de `blocker-messages.test.ts`, `ac9_…`, `ac7_…`, `ac4_…`, `put_range_un_compte_mouvemente_sous_un_groupe` et `jour_suisse_nuit_e2e.rs` ne changent pas d'attendu (hors les ajouts décrits). `sitesTotal` ne bouge pas. Aucune écriture dans `~/kesh/`.
Les données des tests restent fictives (aucun IBAN, solde ou ligne du relevé réel de Guy) ; les comptes `1190`/`1171` sont ceux du plan de test.

### Issues à ouvrir (par l'orchestrateur ; aucun `gh` qui écrit ici ; une issue par problème, `gh issue list --state all --search` avant)
Aucune d'avance. Possibles, selon l'étape 0 : D7 (avoir qui contrôle la TVA due), un site mort ou une mutation qui survit sans test possible (AC12), un trou nouveau de couverture relevé en jouant une mutation.

### Charge estimée
Environ une journée : dix tests courts (S), deux M (#592 : 14 mutations ; #1127 : la campagne), le gros du temps en compilations de mutations. Écart à la borne de l'epic : aucun.

### Références
`CLAUDE.md` (*Qualité du code*, *Review Iteration Rule*, *Priorités des défauts*) ; `crates/kesh-api/tests/jour_suisse_nuit_e2e.rs` ; fiche `35-5a-date-du-jour-horloge.md` (Change Log, A4) ; revues `kesh-gate-logs/321a-review-p1-{B,E}.md` ; mémoires *Tests qui prouvent moins*, *Mutation restaurée, binaire périmé*, *Pathspec git muet*.

## Dev Agent Record

### Agent Model Used
*(à remplir)*

### Debug Log References
*(journaux `kesh-gate-logs/e582/` et `kesh-gate-logs/idefix/<id>/`)*

### Completion Notes List
*(à remplir : par AC, ligne lue citée, mutation jouée, test qui la tue)*

### File List
*(à remplir)*

## Change Log

- 2026-11-27 — Fiche rédigée (Sonnet 5.5), `ready-for-dev` sous réserve de la validation P1 (Opus) ; dix issues retenues, aucune écartée ; deux à moitié couvertes (#732-1, #1433-2) ; les numéros de ligne de #736, #1066 et #1433 ont bougé, relus par la chaîne.
