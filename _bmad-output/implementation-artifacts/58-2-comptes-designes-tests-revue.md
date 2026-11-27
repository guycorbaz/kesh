# Story 58-2 : Comptes désignés et tests Rust de revue — neuf trous de couverture (#592, #732, #736, #1060, #1062, #1063, #1064, #1066, #1433)

Status: ready-for-dev (validation P1 remédiée : 0/0/6/7 ; P2 remédiée : 0/0/1/3 ; P3 ciblée : 0/0/1/2, remédiée par la sortie de #1127 ; P4 ciblée (Sonnet) : 0/0/0/5, remédiée)

*Fiche rédigée le 2026-11-27 (Sonnet 5.5, en autonomie), workflow `bmad-create-story`. Worktree `/home/gcorbaz/devel/kesh-58-2`, branche
`story/58-2-comptes-designes-tests-revue` partie de `epic/58-nettoyage` à `90192c7b3` (qui contient la v0.20.0, la 58-1 et la 58-3). Fichier d'epic :
`_bmad-output/planning-artifacts/epic-58-nettoyage.md` (§ *Story 58-2*). Epic 58 (nettoyage) ; la 58-2 vise la **v0.20.1** (dimanche 2026-11-29 ou lundi 2026-11-30, décision de l'orchestrateur du 2026-11-27 ; numéro attribué au tag) : elle entre dans `main` **avant** l'Epic 53 (v0.21.0, vendredi 2026-12-04). **Aucune migration,
aucun code de production, aucun changement d'API, aucun texte visible** : la story n'ajoute que des tests (et la correction d'un verrou de test) ; aucune
compilation ni aucun gate n'a été lancé pour écrire la fiche, qui ne modifie que ce fichier, le prompt de validation et `sprint-status.yaml`.*

## Story

En tant que mainteneur de Kesh, je veux que neuf gardes que la revue de code avait relevées comme « non tenues par un test » le soient — la clé i18n de chaque motif
d'écran de modification bloquée, le type en mots des refus de comptes désignés, le même compte faux pour deux réglages, l'avoir qui reste possible sur une créance
retypée, l'ordre de la version périmée et du parent d'un compte, les bornes d'une regex de date, un verrou de test oublié —, afin qu'un refactor ou une mutation qui
défait l'une de ces gardes fasse rougir un test, et non qu'on le découvre en production.

## Contrôle de couverture (règle du 2026-11-26 : on vérifie qu'une issue n'est pas déjà couverte avant de la retenir)

Chaque issue a été relue au code de `90192c7b3` (`gh issue view`, lecture seule) ; le test cité comme « manquant » a été cherché (`grep -rn`) dans le fichier nommé,
dans le binaire voisin et dans les tests unitaires du module. **Aucune des neuf issues n'est entièrement réglée** : aucune ne sort de la story. Deux sont **partiellement**
couvertes (#732-1 et #1433-2 : la fiche ne retient que le reste) ; les lignes de plusieurs issues ont bougé (l'écart est écrit).

| # | Constat relu au code | Verdict |
|---|---|---|
| #592 | `blocker-messages.test.ts` mocke `i18nMsg` pour **rendre le repli** (`i18nMsg: (_key, fallback, args) => …`) : aucune assertion ne lit la clé. `i18n-keys.test.ts` (garde B) vérifie qu'une clé demandée **existe** au catalogue, jamais **laquelle** un code emploie. | **gardée** ; le compte a changé (voir § *Ce qui se lit au code*) |
| #732 | (1) `textes_coherents.rs:355-366` : positifs `1171-99-99`, `-13-01`, `-00-10`, `-12-32` ; ni le jour `00`, ni les mois `10`/`11`, ni le garde `(\D\|$)`. (2) Aucun test ne tient la séquence 1171, 2200, 2206, 1100, 3000, 2000, 6940, 3800, 6900, 3805 (aucun `6940`/`3805` dans `crates/*/tests` hors `invoices_validate_vat.rs` et `test_fixtures.rs`, pour d'autres usages). (3) `designate_vat_*` : une seule société, `version` comparée seulement entre t2 et t3. (4) `fiscal-years-page.test.ts:142-189` : « trois clos » et « un clos suivi d'un ouvert », pas « clos, ouvert, clos ». | **gardée**, les quatre points ; le point 1 est **à moitié couvert** (mois `00`/`13`, jour `32` le sont) ; le point 4 est un test **frontend**, non Rust (le fichier d'epic dit « un test frontend d'i18n » : voir D2) |
| #736 | `i18n_e2e.rs:126-128` (`i18n_messages_endpoint_returns_locale_and_messages`) et `:151-153` (`i18n_messages_requires_auth`) appellent `spawn_app` sans `get_i18n_lock()` ; `grep -c 'get_i18n_lock().await'` = **4** pour **6** tests `#[sqlx::test]`. | **gardée** (lignes 126/151, non 128/153 de l'issue) |
| #1060 | `designated_account_type_e2e.rs:527` est le **seul** `type de compte incompatible (attendu : …)` asserté (« actif »). `account_type_in_words` (`routes/company_invoice_settings.rs:220-227`) a quatre bras ; « passif », « charge ou produit » ne sont asserté nulle part dans `crates/kesh-api/tests`. | **gardée** |
| #1062 | Aucun test ne monte un compte faux pour deux réglages ; `WrongTypeDesignatedAccounts::new` (`kesh-db/src/errors.rs:984-1000`) dédoublonne par `(identifiant, réglage)` et trie par numéro, identifiant, **réglage** — la garde existe, seul le test manque. Le commentaire de l'issue renvoie à **#1059** (test **unitaire** de `new` : tri, dédoublonnage), ouverte, sans milestone, **hors de cette story** (une issue par problème) : c'est elle seule qui tient le départage par réglage (AC8, V1-3). | **gardée** |
| #1063 | `ac7_purchase_validation_orders_the_accounts_by_number` (l. 322-387) rend 400 et s'arrête : pas de `GET` de la facture fournisseur ensuite. La vente le fait (`ac7_sale_validation_names_the_retyped_receivable`, l. 252-281 : « la facture reste brouillon »). | **gardée** |
| #1064 | Aucun test de `POST /api/v1/credit-notes` sur une créance retypée (`grep -rn retype` dans les tests d'avoir : vide). Le manuel utilisateur (« Rôles des comptes ») le promet. | **gardée** |
| #1066 | `supplier_invoices_repository.rs:2691-2720` (`ac4_payable_retyped_asset_is_refused`, à la ligne 2536 dans l'issue) : `update_account(Liability, Some(Payable))`, puis `update_account(Asset, Some(Payable))`, **une** assertion du rôle, après le second. | **gardée** |
| #1433 | `accounts.rs` : la version se compare à la ligne **1224** (`Some(a) if a.version != version => … OptimisticLockConflict`), avant `guard_parent_change_in_tx` (ligne **1266**) — non 1126/932 de l'issue. `put_parent` (`accounts_e2e.rs:1185`) relit toujours la version fraîche ; `le_conflit_de_version_precede_la_garde_de_retypage` (`accounts.rs:4065`) tient l'ordre pour le **retypage**, pas pour `parentId`. « Absent » : `accounts_e2e.rs:1249` ne l'asserte qu'**sans autre champ modifié** (no-op). | **gardée**, les deux cas ; le cas (2) est à moitié couvert |

Aucune des neuf ne révèle une migration, un changement d'API ou un défaut de production. Si une mutation révèle un défaut de production, il s'ouvre en issue (§ *Issues à ouvrir*), il ne se corrige pas dans cette story.

## Sortie de #1127

**Décision du Project Lead, 2026-11-27** (signal de recyclage déclaré à la P3 ciblée, règle de splitting préventif) : **#1127** (campagne de mutations M3/M7 sur les sites de la 35-5a qui lisent la date du jour) **sort de la 58-2**.
Raison : l'inventaire de ses sites débordait à chaque passe — la P1 (V1-5) y trouvait 25 lignes de routes et 0 de `kesh-db`, la P2 (V2-1) la couche `services/`, la P3 (F3-1) un décompte faux et 19 sites non classés. L'issue reste **ouverte**, hors de cette story ;
elle ne passe ni en « gardée » ni en « écartée » ici, et aucun commit de la 58-2 ne la porte en `refs`. L'AC12 est **vacante** : les numéros d'AC ne sont pas renumérotés, pour que les renvois des passes P1 à P3 (Change Log) restent lisibles.

Point de départ de sa future story, relevé au commit `c2dd70f5d` (code identique à `90192c7b3`) :
- `grep -rn 'aujourd_hui()' crates/kesh-api/src` rend **54** lignes ; `grep -rn 'horloge.aujourd_hui()' crates/kesh-api/src/routes` en rend **41**, dans **16** fichiers.
- Les tests de la 35-5a à muter sont les sept de `jour_suisse_nuit_e2e.rs` relevés par l'issue ; la 35-5a avait joué M3 sur `routes/journal_entries.rs` et M7 sur `invoice_settlements_write.rs` seulement.
- Les **19 sites que la P3 a relevés sans les classer** sont à reprendre (chacun à l'inventaire, ou « hors AC2/AC3 » avec le chemin qui le lit) : `routes/audit_log.rs:423`, `dunning_reminders.rs:169`, `invoice_pdf.rs:96`, `onboarding.rs:271` et `:1048`, `exports.rs:60`, `admin.rs:101`,
  `invoice_email.rs:446`, `:913` et `:1160`, `companies.rs:711` et `:732`, `exchange_rates.rs:257`, `imported_supplier_invoices.rs:139`, `:599` et `:710` ; `exchange_rates/mod.rs:213` et `:385` ; `lib.rs:116` (un commentaire, non un appel).
- L'Epic 53 (`story/53-6-annulation-rapprochement-exercice`) réécrit `jour_suisse_nuit_e2e.rs` et les dépôts de ces sites : la future story part de `main` **après** la 53-6.

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
   `DefaultRevenue`, `Payable` (dans `complete_default_accounts_in_tx`, l. 1727), puis `chart_designated_accounts` (l. 674). ⚠️ **Celle-ci ne parcourt pas `WriteOffNature::ALL`** (validation P1, V1-1) :
   elle écrit en dur l'arrondi puis `WriteOffNature::Discount`, `BankFees`, `BadDebt` (l. 703-723) ; seul son commentaire (l. 701-702 : « les natures dans l'ordre de `WriteOffNature::ALL` ») le dit,
   et c'est vrai aujourd'hui par coïncidence (`ALL = [Discount, BankFees, BadDebt]`, `kesh-core/src/chart_of_accounts/mod.rs:201`). Le plan PME est **un fichier de production** :
   `crates/kesh-core/assets/charts/pme.json` (`include_str!`, `mod.rs:301`), **hors** de `crates/*/src/*`. Au plan PME : 1171 (`VatRecoverable`), 2200 (`VatPayable`), 2206 (`VatSettlement`), 1100 (`Receivable`),
   3000 (`DefaultRevenue`), 2000 (`Payable`), 6940 (`roundingDifference`), 3800 (`discount`), 6900 (`bankFees`), 3805 (`badDebt`) — relu dans `pme.json` à la validation P1
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
- **Q4 tranchée au code (validation P1)** : `credit_notes::create_credit_note` lit la créance sur l'écriture de vente (`sale_receivable_account`, `credit_notes.rs:486-487`) et la TVA due **dans les réglages**
  (`settings.default_vat_payable_account_id`, l. 545), les verrouille et n'en refuse que l'**archivage** (l. 576-580) ; le commentaire de la 6 ter (l. 661) : « `active` UNIQUEMENT : ni `postable` ni `account_type` ne sont
  re-vérifiés ». Le manuel (`user-manual.tex:619`, PDF aplati compris) : « relit la TVA due dans les réglages au moment de l'avoir, sans la contrôler ». **Code et manuel concordent** : aucune issue à ouvrir,
  la variante « TVA due retypée » de D7 entre au test d'AC10 (attendu 201).

### #1066 (P4) — `ac4_payable_retyped_asset_is_refused`
Le premier `update_account(Liability, Some(AccountRole::Payable))` pose le rôle ; le second retype en `Asset` ; l'assertion du rôle n'est qu'après. La fiche de la 35-3 (AC4) veut le rôle vu **posé avant** le retypage.

### #1433 (P3) — `PUT /accounts/{id}` avec `parentId`
`accounts::update` (l. 1149) : introuvable/archivé → **version** (l. 1224) → no-op → … → `guard_parent_change_in_tx` (l. 1266). Test existant de l'ordre : `le_conflit_de_version_precede_la_garde_de_retypage` (`accounts.rs:4065`, `kesh-db`),
pour le retypage seulement. HTTP : `put_parent` (`accounts_e2e.rs:1185`) lit la version courante et laisse `extra` la surcharger (`body[k] = v`) : une version périmée s'y passe par `extra`.
⚠️ **Deux filets derrière le contrôle de version** (validation P1, V1-4) : l'`UPDATE` porte lui-même `WHERE id = ? AND version = ?` (l. 1324-1327) et rend `OptimisticLockConflict` sur `rows == 0` (l. 1344-1348).
Avec un parent **valide**, retirer ou déplacer le contrôle de la l. 1224 rend donc **toujours 409** : la mutation survit. Seul un parent que `guard_parent_change_in_tx` **refuse** la démasque.
Et « absent » : `is_no_op_change` compare `changes.parent.resolve(before.parent_id)` (l. 770), si bien qu'une mutation de `resolve` est **déjà** tuée par le cas no-op de `accounts_e2e.rs:1249-1253` ; le trou est au
**`.bind(parent_id)`** de l'`UPDATE` (l. 1336), qu'un no-op n'atteint pas.

## Décisions

- **D1 — Aucun code de production, pas même un commentaire.** Les neuf issues sont des tests. La story ne touche pas `company_invoice_settings.rs` (donc pas le doc-comment de la séquence des verrous,
  #732-2), ni `routes/`, ni `accounts.rs`. *Option écartée* : corriger le doc-comment (« au plan PME actuel ») au lieu du test — il toucherait un **repository** de `kesh-db` et déclencherait
  l'exception « gate complet » du `CLAUDE.md` pour une phrase ; et la revue 32-1a demandait un test (« qu'aucun test ne tient »).
- **D2 — Les points frontend restent dans la story.** #592 et #732-4 sont des tests Vitest. Le fichier d'epic parle d'« un test frontend d'i18n » (#592) ; #732-4 en est un second, de la même
  nature (un fichier de test de la page des exercices). Le **module** de la story au sens du `CLAUDE.md` (code de production qui change) reste **zéro** ; au sens des fichiers de test : `kesh-api/tests`,
  `kesh-db/tests`, `frontend/src/lib/features/journal-entries`, `frontend/src/routes/(app)/settings/fiscal-years` (quatre, sous la borne de cinq). *Option écartée* : sortir #732-4 vers la 58-6 (tests
  frontend de la réconciliation) — elle ne touche pas ce domaine.
- **D3 — #592 : la clé s'asserte par un mock qui rend la clé, et l'existence aux quatre catalogues se relit à la source.** Un **nouveau** fichier `blocker-messages-keys.test.ts` (même dossier) mocke
  `i18nMsg: (key) => key` : `modificationBlockerLabel(code, label)` rend alors la **clé** ; une table `code → clé` écrite **en littéraux dans le test** (13 lignes pour la modification, 8 pour la contre-passation)
  est comparée, puis chaque clé est cherchée dans les quatre `messages.ftl` (ligne `^<clé> =`, lecture `readFileSync('../crates/kesh-i18n/locales/<locale>/messages.ftl')` comme `i18n-keys.test.ts`).
  Le site du formulaire (`JournalEntryForm.svelte:687`) se tient dans `JournalEntryForm.edit.test.ts` (cas `FISCAL_YEAR_CLOSED`, l. 202) **sans changer l'assertion d'aucun autre cas**. ⚠️ Le formulaire importe
  `i18nMsg` de **`$lib/features/onboarding/onboarding.svelte`** (`JournalEntryForm.svelte:7`), non de `$lib/shared/utils/i18n.svelte` : c'est le mock de la l. 20-22 du fichier de test qui devient un **espion**
  (`vi.fn((_key, fallback) => fallback)` hissé par `vi.hoisted`, le repli toujours rendu), et le cas asserte `toHaveBeenCalledWith('journal-entries-modify-blocked-fiscal-year-closed', expect.any(String))`
  **en plus** du toast existant. Pas de « dernière clé » : le rendu appelle `i18nMsg` pour d'autres clés après le toast (validation P1, V1-7). *Options écartées* : (a) modifier le mock de `blocker-messages.test.ts` (il tient les phrases par leur repli : deux rôles dans un fichier) ; (b) extraire les clés du source
  par regex (**les deux côtés du même extracteur** : « vrai par construction », `CLAUDE.md`, *Inventorier les sites NON RÉSOLUS*). La table du test est écrite à la main, c'est ce qui la rend indépendante.
  Le périmètre inclut les huit clés de la contre-passation (même fichier, même mutation F2, coût marginal) ; l'issue ne parlait que des motifs de modification : élargissement **déclaré** (Q1, tranchée par l'orchestrateur : oui).
- **D4 — Une mutation se joue dans la copie de travail, se restaure par copie et `touch`, et se garde par `git status`.** Pas de commit de production : chaque mutation se pose sur la copie de travail **du worktree de la story**, se joue en **local**
  (`scripts/mem-guard.sh`, ciblé `binary(<binaire touché>)`, sous `flock .gate.lock`). **Restauration (Q2, tranchée par l'orchestrateur)** : avant chaque mutation, le fichier est **copié** hors du dépôt
  (`$GATE_LOGS/e58/s582-mut/<nom>.orig`, `$GATE_LOGS` désignant `kesh-gate-logs/`, hors dépôt) ; après, il est **recopié** depuis cette sauvegarde **puis `touch`** (cargo garderait le binaire muté : mémoire *Mutation restaurée, binaire périmé*).
  **Garde avant chaque commit et chaque push** (validation P1, V1-2) :
  `git status --porcelain | grep -vE '^.. (crates/kesh-(api|db)/tests/|frontend/src/.*\.test\.ts$|_bmad-output/)'` **vide** — elle voit aussi les mutations du frontend (`blocker-messages.ts`, `JournalEntryForm.svelte`,
  `+page.svelte` des exercices) et du plan (`crates/kesh-core/assets/charts/pme.json`), que la pathspec `crates/*/src/*` ne voyait pas.
- **D5 — #732-2 : le test tient le plan, pas l'ordre SQL ; limite écrite (Q3, tranchée : suffit pour ce point).** Aucun moyen portable de lire l'ordre réel des verrous InnoDB à travers MariaDB (à notre connaissance, MariaDB n'expose pas `performance_schema.data_locks` ;
  un journal général est global ; à confirmer à l'étape 0). Le test monte un plan PME et résout, **dans l'ordre écrit par le code** (relu à l'étape 0 et cité) : par rôle `VatRecoverable`, `VatPayable`, `VatSettlement`, puis
  `Receivable`, `DefaultRevenue`, `Payable` ; puis le compte d'arrondi (`rounding_difference_number`) ; puis les natures **écrites en littéraux** `[Discount, BankFees, BadDebt]` — l'ordre de `chart_designated_accounts` (l. 703-723),
  **non** `WriteOffNature::ALL`, que le code ne parcourt pas (V1-1). Il assere que la suite des **numéros** égale le littéral `["1171","2200","2206","1100","3000","2000","6940","3800","6900","3805"]`, écrit en dur et cité du doc-comment.
  Une **seconde assertion**, `WriteOffNature::ALL == [Discount, BankFees, BadDebt]`, tient la phrase du commentaire de production (l. 701-702) qui les dit égaux. Un plan PME modifié ou un rôle réattribué rougissent la
  première ; un `ALL` réordonné rougit la seconde (et elle seule : l'ordre des verrous, lui, n'aurait pas changé). **L'ordre des appels** dans `lock_vat_role_accounts_in_tx`, `complete_default_accounts_in_tx` et
  `chart_designated_accounts` **reste hors du test** : deux champs permutés dans `chart_designated_accounts` changent l'ordre réel des verrous sans rien rougir — angle mort assumé, écrit dans le doc-comment du test,
  et la fermeture du point 2 de #732 le dit (Q3).
  *Option écartée* : réduire le doc-comment à « l'ordre des rôles, sans numéros » (D1 : production) ; *option écartée* : instrumenter une transaction concurrente pour deviner l'ordre (course, non déterministe).
- **D6 — #1062 : le compte faux pour deux réglages se monte sans retypage.** Un compte de charge **sans rôle** créé par `new_account` (`designated_account_type_e2e.rs:192`) et posé par SQL aux deux réglages que la vente juge
  (`default_receivable_account_id`, `default_vat_payable_account_id`) : un **même** compte aux deux réglages ne se pose ni par la route des réglages (elle refuserait le type) ni par un retypage (la créance
  `1100` et la TVA due `2000` du montage sont deux comptes distincts) ; le test vise la validation. Prémisse **confirmée** à la validation P1 : `check_written` lit l'identifiant **dans les réglages**
  (`role.designated_id(settings)`, `company_invoice_settings.rs:1280`) et la facture de `draft_invoice` est à 8.1 % (TVA due écrite). L'attendu : `rejected` = deux entrées, même numéro, **dans l'ordre des réglages** (`receivable` puis `vatPayable`, l'ordre de `DesignatedSetting`), le message porte deux phrases « Le compte NNNN, désigné comme … » et
  **une seule** phrase d'action. Si l'étape 0 montre que la validation lit le rôle et non le réglage, le montage se réécrit et la décision se corrige au Dev Agent Record.
- **D7 — #1064 : l'avoir sur la créance retypée, puis sur la TVA due retypée.** Le manuel dit que l'avoir « relit la TVA due dans les réglages … sans la contrôler », et le code le confirme (§ *Ce qui se lit au code*,
  Q4) : le second cas (TVA due `2000` retypée en actif, avoir 201) entre dans le test. Si l'étape 0 le démentait à l'exécution, l'écart serait un **défaut de production ou de manuel**, ouvert en issue distincte,
  priorité au Project Lead, et le test garderait la seule créance.
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
- **AC3 (#732-2)** — Un test de `kesh-db` (`company_invoice_settings_repository.rs`) tient la suite de numéros de D5 contre le littéral, et `WriteOffNature::ALL == [Discount, BankFees, BadDebt]` ; mutations jouées dans la copie de travail :
  (i) dans **`crates/kesh-core/assets/charts/pme.json`** (fichier de production, hors `crates/*/src/*` : garde de D4), le rôle `VatPayable` déplacé de `2200` vers un autre passif imputable du plan (le plan doit rester valide pour `load_chart`) ;
  (ii) dans le même fichier, la nature `bankFees` déplacée de `6900` vers une autre charge imputable ; (iii) `WriteOffNature::ALL` réordonné (`kesh-core/src/chart_of_accounts/mod.rs:201`) — (i) et (ii) rougissent la première assertion,
  (iii) la seconde. **Survivante attendue et écrite** : `discount` et `bank_fees` permutés dans `chart_designated_accounts` (l. 705-715) — l'angle mort de D5, écrit au doc-comment du **test** et au Dev Agent Record.
- **AC4 (#732-3)** — Deux sociétés de plan PME (`pme_company_with_chart` deux fois, chacune sa ligne de réglages par `create_settings_row`) ; montage asserté : les trois colonnes de TVA de B à `NULL`. La désignation de A
  laisse les trois colonnes de B à `NULL` et la `version` de B intacte ; la `version` de A est lue **avant** t2 et **après** : **égale** (le doc-comment, `company_invoice_settings.rs:2080` : « N'incrémente pas `version` » —
  l'étape 0 le confirme à l'exécution, sinon l'attendu s'écrit tel que le code le donne et se cite). Mutation : `WHERE company_id = ?` (l. 2096) remplacé par `WHERE company_id = ? OR TRUE` (le `bind` reste valide ; retirer
  la clause sans retirer le `bind` ferait échouer la requête, non le test) survit **avant**, rougit **après**.
- **AC5 (#732-4)** — Un cas `fiscal-years-page.test.ts` « clos, ouvert, clos » : pour A, le bouton Réouvrir est désactivé et l'infobulle (bouton **et** enveloppe) nomme **C**, non B ; B (ouvert) n'a pas de bouton Réouvrir ;
  C est rouvrable sans infobulle. Mutation « ne regarder que l'exercice immédiatement suivant » : tuée **aussi** par ce cas (elle l'était déjà par « trois clos » : l'AC le dit, sans prétendre à un trou neuf).
- **AC6 (#736)** — `grep -c 'get_i18n_lock().await' crates/kesh-api/tests/i18n_e2e.rs` : **4 → 6** ; chacun des six tests `#[sqlx::test]` du fichier prend le verrou avant son premier `spawn_app*`. Pas de preuve du rouge
  déterministe (flake de course) : la preuve est le **comptage** plus la lecture des six têtes de test, citées au Dev Agent Record ; une boucle de rejeu (`--test-threads=8`) n'est pas une preuve.
- **AC7 (#1060)** — Les quatre refus de la boucle de `ac9_…` (créance, créanciers, TVA due, TVA récupérable) assertent le message **entier** `"<libellé> : type de compte incompatible (attendu : actif|passif)"` ; un cinquième cas
  pose un compte d'actif au `defaultRoundingAccountId` : 400 `"Compte de différences d'arrondi : type de compte incompatible (attendu : charge ou produit)"`. Les cinq messages attendus sont écrits **en littéraux entiers**
  dans le test (une table champ → message, non un `format!` qui recomposerait le libellé : l'attendu ne sort pas du code) ; `grep -c 'type de compte incompatible (attendu : ' designated_account_type_e2e.rs` : **1 → ≥ 6**
  (l'existant, plus les cinq littéraux). Mutations `Liability => "actif"`, `Expense => "produit"`, `Revenue => "charge"` (chacune rend « charge ou charge » ou « produit ou produit », ou le mauvais mot d'un passif) :
  toutes rougissent **après**, aucune **avant** ; `Asset => …` est déjà tuée par l'assertion existante (l. 527).
- **AC8 (#1062)** — Un test `ac7_sale_validation_names_the_same_account_wrong_for_two_settings` : `details.rejected` = `[(NNNN, receivable, Asset, Expense), (NNNN, vatPayable, Liability, Expense)]` (D6) ; deux phrases « Le compte NNNN, … » ;
  la phrase d'action **une fois** ; la facture reste brouillon. Mutation : la clé de `dedup_by` réduite au seul `account_id` (`errors.rs:994`) rougit **après** (une entrée au lieu de deux). **Survivante attendue et écrite**
  (validation P1, V1-3) : retirer `.then(a.setting.cmp(&b.setting))` (l. 991) — `check_written` parcourt un `BTreeSet<DesignatedRole>` (`company_invoice_settings.rs:1272`, ordre `Receivable < VatPayable`, l. 941-953) et
  `sort_by` est stable : l'entrée arrive déjà dans l'ordre des réglages. Seul un test **unitaire** de `new` aux entrées inversées la tue : c'est **#1059**, hors de cette story (ni absorbé, ni fermé ici).
- **AC9 (#1063)** — Après le 400 de `ac7_purchase_validation_orders_the_accounts_by_number`, un `GET /api/v1/supplier-invoices/{id}` : `status == "draft"`, `internalNumber` nul, `version` égale à celle d'avant la validation.
  Mutation : à jouer, mais **survivante probable** et à dire telle : l'attribution du numéro (`attribute_internal_number_in_tx`, `supplier_invoices.rs` après `check_written`, l. 2316-2325) et le refus partagent la même
  transaction, si bien que déplacer l'attribution avant le refus est défait par le `rollback`. Le test tient alors le **contrat à travers la route** (rien d'écrit), que l'issue demande ; le Dev Agent Record le dit, sans prétendre au rouge.
- **AC10 (#1064)** — Un test : facture validée, créance `1100` retypée en passif par l'API (`app.retype`), `POST /api/v1/credit-notes` → **201**, l'écriture inverse crédite `1100` (lue sur l'écriture de la facture) ; puis, sur une seconde facture validée,
  la TVA due `2000` retypée en actif → **201** (D7, Q4 : code et manuel concordent). Mutation : un contrôle de type de la créance (puis de la TVA due) ajouté à `create_credit_note` (copie) rend 400 ⇒ rouge **après**.
- **AC11 (#1066)** — Entre les deux `update_account` de `ac4_payable_retyped_asset_is_refused`, l'assertion que le rôle vaut `Some(AccountRole::Payable)` **et** que le type vaut `Liability` ; après le second, l'assertion existante.
  `grep -c 'role, Some(AccountRole::Payable)' supplier_invoices_repository.rs` augmente d'**1** dans ce test.
- **AC13 (#1433)** — Deux tests de `accounts_e2e.rs` : (a) `PUT /accounts/{id}` avec `version` périmée **et** un `parentId` que `guard_parent_change_in_tx` **refuserait** (un compte qui porte des écritures, comme `caisse` de
  `put_range_un_compte_mouvemente_sous_un_groupe` → `ACCOUNT_PARENT_HAS_ENTRIES`, ou le compte lui-même → cycle) → **409** de code **`OPTIMISTIC_LOCK_CONFLICT`** (le code, non le seul statut : `ACCOUNT_PARENT_HAS_ENTRIES` est aussi un 409),
  parent **relu inchangé** en base ; un parent **valide** ne prouve pas l'ordre (V1-4 : l'`UPDATE … AND version = ?` rend 409 de toute façon). (b) `PUT` sans `parentId` mais avec un autre champ modifié (le nom) → 200, nom changé,
  `parentId` **inchangé** (compte rangé sous un groupe au préalable, `version` + 1). Mutations : (a) le bloc du garde de parent (`accounts.rs:1262-1268`, avec la résolution d'`own_fx` de la l. 1256) **déplacé avant** le `match` de la l. 1216, gardé par `if let Some(before) = &before_opt` (`own_fx` ne dépend que de `before` et de `changes` : la mutation compile ; elle s'écrit sur la copie, la l. 1266 d'origine étant retirée) — survit **avant**
  (`le_conflit_de_version_precede_la_garde_de_retypage` ne passe aucun parent), rougit (a) **après**. Le simple retrait du bras `Some(a) if a.version != version` (l. 1224) n'est **pas** la bonne mutation : ce test
  `kesh-db` la tue déjà (le garde de retypage parle alors avant), ce que le Dev Agent Record peut constater sans la compter comme preuve d'(a) ;
  (b) `.bind(parent_id)` (l. 1336) remplacé par `.bind(match changes.parent { <absent> => None, _ => parent_id })` (variante à écrire selon le type de `changes.parent`) — survit **avant** (le cas « absent » existant est un no-op,
  qui ne joint pas l'`UPDATE`), rougit (b) **après**.
- **AC14 (transverse)** — (a) `git diff --name-only 90192c7b3 HEAD` ne nomme que : `crates/kesh-api/tests/{textes_coherents,i18n_e2e,designated_account_type_e2e,accounts_e2e}.rs`,
  `crates/kesh-db/tests/company_invoice_settings_repository.rs`, `crates/kesh-db/tests/supplier_invoices_repository.rs`, `frontend/src/lib/features/journal-entries/{blocker-messages-keys.test.ts,JournalEntryForm.edit.test.ts}`,
  `frontend/src/routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts`, `_bmad-output/implementation-artifacts/58-2-*`, `sprint-status.yaml`, ce fichier (+ prompts). **Aucun** fichier de `crates/*/src/*`, aucun catalogue, aucun manuel.
  (b) `git diff --name-only 90192c7b3 HEAD | grep -vE '^(crates/kesh-(api|db)/tests/|frontend/src/.*\.test\.ts$|_bmad-output/implementation-artifacts/)'` **vide**, et `git status --porcelain` vide (garde de D4)
  (la pathspec `crates/*/src/*` seule ne voyait ni le frontend ni `crates/kesh-core/assets/` : V1-2 ; sonde positive : la même commande sur `90192c7b3 origin/story/53-6-annulation-rapprochement-exercice` rend des lignes). (c) `sitesTotal` de `i18n-keys.test.ts` **inchangé**
  (le nouveau test ne contient pas d'appel `i18nMsg` de production ; il mocke `i18nMsg` : à vérifier au moissonneur, `npm run test:unit -- i18n-keys`).

## Preuve du rouge

Un test de garde est rouge **par mutation**, non par absence : pour chaque AC, le développeur écrit le test, le joue **vert** sur le code actuel, joue la mutation nommée (copie de travail, restauration + `touch`), constate le **rouge**
et relève le nom de l'assertion qui rougit (journal `kesh-gate-logs/e58/s582-mutations.md`, hors dépôt : AC, mutation, test, sortie nextest/Vitest). Les AC sans mutation possible sont dits (AC6 : comptage).
Un test qui reste vert sous sa mutation est **retiré ou récrit** avant commit : « le test passe » ne prouve rien (`tests-qui-prouvent-moins`). Le journal compte les mutations (**recompté** depuis le fichier, avec son périmètre) ;
la fiche n'en annonce pas le nombre d'avance.

## Tâches

0. **Étape 0** : relire les lignes citées **par la chaîne** ; vérifier D6 (la validation lit-elle le réglage ou le rôle ?), D7 (l'avoir sur la TVA due retypée passe-t-il ?), la table de D5 (`WriteOffNature::ALL`, ordre des natures
   au plan PME), l'attendu de version d'AC4 ; relever, pour chaque AC, la **mutation** et le **test qui la tue**. Recontrôler les corrections des validations (ligne lue citée au Dev Agent Record) : l'ordre en dur de
   `chart_designated_accounts` (V1-1), la garde `git status --porcelain` de D4 (V1-2, sonde positive jouée sur une mutation réelle avant la première restauration), l'ordre des entrées de `check_written` (V1-3), les deux filets de version
   de `accounts::update` et le type de `changes.parent` (V1-4), l'état de l'Epic 53 dans `main` (V1-6), l'import d'`i18nMsg` du formulaire (V1-7).
1. AC6, AC11 (une ligne chacun) ; AC7 à AC10 (`designated_account_type_e2e.rs`, `supplier_invoices_repository.rs`) ; AC13 (`accounts_e2e.rs`) ; AC2 (`textes_coherents.rs`) ; AC3, AC4 (`company_invoice_settings_repository.rs`).
2. AC1 et AC5 (frontend).
3. Gates (§ *Gates*), recette (sans objet), Dev Agent Record, Notes de version à l'orchestrateur.

Un commit par étape franchie (`test(58-2): …`, `refs #N` des issues touchées, aucun mot-clé de fermeture) et **push de la seule branche** `story/58-2-comptes-designes-tests-revue` à chaque étape (règle des agents du `CLAUDE.md`).

## Gates

- **Les gates complets se jouent sur le serveur idefix** : `/home/gcorbaz/devel/kesh-gate-logs/serveur/gate-distant.sh story/58-2-comptes-designes-tests-revue <gate>` (`test-fast`, `gate-ci`, `front`, `e2e`, `tout`), **après push**
  de la branche (idefix la lit sur GitHub) ; les journaux reviennent dans `kesh-gate-logs/idefix/<id>/`. Le story file n'affirme que ce qui a tourné (`CLAUDE.md`, *Pendant une boucle de revue*).
- **Entre deux commits** : gate ciblé en local sous `scripts/mem-guard.sh` : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, puis `cargo nextest run -E 'binary(<binaire touché>)'` :
  `textes_coherents`, `i18n_e2e`, `designated_account_type_e2e`, `accounts_e2e` (kesh-api) ; `company_invoice_settings_repository`, `supplier_invoices_repository` (kesh-db) ; Vitest ciblé
  (`npx vitest run blocker-messages-keys JournalEntryForm.edit fiscal-years-page`), `npm run check`, `npm run lint-i18n-ownership`.
- **Exception `kesh-db`** : la story ne touche **ni migration, ni `post_restore.rs`, ni repository** (les fichiers de `kesh-db/tests` ne sont pas un repository) : le gate complet n'est donc pas exigé en cours de boucle ; il l'est **au dernier commit**
  (`tout` sur idefix : `test-fast`, `gate-ci`, `front`, `e2e`) puisque des tests de `kesh-db`, `kesh-api` et du frontend changent.
- **Recette visuelle** : sans objet (aucun contrôle visible, aucun texte d'écran). Captures : aucune.
- **Mutations** (AC1 à AC11 et AC13) : en **local**, jamais sur idefix (gate-distant joue la branche poussée, jamais une copie mutée) ; aucun fichier de production muté n'est commité ni poussé.

## Note de version

*(Pour le CHANGELOG de la v0.20.1 (dimanche 2026-11-29 ou lundi 2026-11-30), rubrique « Interne » ou « Tests », assemblée à l'intégration ; la story n'écrit pas le CHANGELOG.)*

**Tests** — Neuf gardes de la revue de code sont désormais tenues par un test : la clé i18n de chaque motif d'écran de modification bloquée, le type attendu en mots des refus de comptes désignés (actif, passif, charge ou produit),
le même compte faux pour deux réglages, l'avoir possible sur une créance retypée, l'ordre de la version périmée et du déplacement d'un compte, les bornes de la reconnaissance d'une date dans la garde des textes, et le verrou
de locale de deux tests d'i18n. Aucun changement de comportement. (refs #592, refs #732, refs #736, refs #1060, refs #1062, refs #1063, refs #1064, refs #1066, refs #1433)

## Questions (pour l'orchestrateur et le Project Lead)

Toutes tranchées le 2026-11-27 (Q1 à Q3 par l'orchestrateur, choix techniques ; Q4 au code par la validation P1).

- **Q1 — #592 : l'élargissement aux huit clés de la contre-passation est-il admis ?** **Oui** : #592 s'élargit aux huit clés de contre-passation (D3, AC1).
- **Q2 — Une mutation jouée en local sur la copie de travail est-elle admise ?** **Oui** : mutations jouées dans la copie de travail du worktree, restaurées par **copie du fichier sauvegardé puis `touch`**, journal hors dépôt
  (`$GATE_LOGS/e58/`), garde `git status --porcelain` de D4 vide avant chaque commit et chaque push.
- **Q3 — #732-2 : la limite de D5 (le test tient le plan, non l'ordre SQL) suffit-elle à fermer le point ?** **Oui**, l'angle mort (l'ordre des appels, deux champs permutés dans `chart_designated_accounts`) écrit au doc-comment
  du test et au Dev Agent Record ; la fermeture de #732 attend les **quatre** points (une issue, une release).
- **Q4 — L'avoir contrôle-t-il la TVA due retypée, contre le manuel ?** **Non, et le manuel le dit** : le code ne contrôle que l'archivage (`credit_notes.rs:545`, `:576-580`, `:661`), le manuel « relit la TVA due … sans la
  contrôler » (`user-manual.tex:619`, PDF compris). Code et manuel concordent : **aucune issue à ouvrir** ; la variante entre à AC10. Si l'exécution à l'étape 0 le démentait, issue distincte, hors de la story, priorité au Project Lead.

## Dev Notes

### Modules touchés (règle de splitting)
Code de production : **aucun**. Fichiers de test : quatre ensembles (D2), borne de cinq respectée. Pas de dérogation à la règle de splitting. Signal de recyclage : **déclaré** à la P3 ciblée (MEDIUM de même origine aux trois passes), remédié par la sortie de l'issue de campagne de mutations (§ *Sortie de #1127*).

### Fichiers communs
Relevé à la validation P1 par `git diff --name-only 90192c7b3...origin/<branche>` sur les branches `epic/53*`, `story/53-*`, `epic/58*`, `story/58-*` ouvertes (V1-6), **rejoué et complété à la P2** ; l'Epic 53 n'est pas dans `90192c7b3`.
- **Ordre d'entrée (décision de l'orchestrateur, 2026-11-27)** : la 58-2 (v0.20.1) entre dans `main` **la première** ; l'Epic 53 (v0.21.0, 2026-12-04) la reprend. La fiche n'attend donc pas l'Epic 53. **Ce qui se rejoue si l'ordre changeait** (l'Epic 53 d'abord) : la 58-2 reprend `main`, recompte
  `sitesTotal`, les numéros de ligne de `accounts.rs` (AC13 ; l'Epic 53 renomme `find_types_by_ids_in_tx` en `lock_types_by_ids_in_tx`, +19 lignes dès la l. 461) et de `kesh-db/src/errors.rs` (AC8 ; trois hunks, +24 lignes, dès la l. 573,
  le premier (+13) avant `WrongTypeDesignatedAccounts::new`) bougent, les mutations se retrouvent par la chaîne ; **aucun** des deux fichiers n'y change de comportement aux sites mutés (`accounts::update`, `new`).
- `crates/kesh-api/tests/*` : seule la 58-2 de l'Epic 58. **Epic 53** : `designated_account_type_e2e.rs` n'est touché par **aucune** de ses branches ; en revanche `story/53-6-annulation-rapprochement-exercice` touche **`crates/kesh-db/tests/supplier_invoices_repository.rs`** (AC11). Conséquence : dans l'ordre décidé (58-2 d'abord), c'est la 53-6 qui reprend la 58-2 (second arrivé
  résout, deux côtés gardés : `supplier_invoices_repository.rs`, `JournalEntryForm.edit.test.ts`, `i18n-keys.test.ts` ; à signaler au Project Lead au compte rendu de la PR).
- `frontend/src/lib/features/journal-entries/*` : 58-4 (textes d'interface) et 58-5 dans l'Epic 58 ; **Epic 53** (`story/53-1-ecriture-unique`, `53-1b`, `53-6`) modifie `JournalEntryForm.svelte` (le site de la l. 687 peut bouger : le
  retrouver par la chaîne) et `JournalEntryForm.edit.test.ts`, ainsi que `i18n-keys.test.ts`. **Le nouveau fichier de test** ne recoupe aucun fichier existant ; reprendre `epic/58-nettoyage` avant le dernier commit.
- `sprint-status.yaml` : un bloc par story (conflit : garder les deux côtés).

### Ce qui doit rester vrai
Les assertions existantes de `blocker-messages.test.ts`, `ac9_…`, `ac7_…`, `ac4_…`, et `put_range_un_compte_mouvemente_sous_un_groupe` ne changent pas d'attendu (hors les ajouts décrits). `sitesTotal` ne bouge pas. Aucune écriture dans `~/kesh/`.
Les données des tests restent fictives (aucun IBAN, solde ou ligne du relevé réel de Guy) ; les comptes `1190`/`1171` sont ceux du plan de test.

### Issues à ouvrir (par l'orchestrateur ; aucun `gh` qui écrit ici ; une issue par problème, `gh issue list --state all --search` avant)
Aucune d'avance (Q4 : code et manuel concordent). Possibles, selon l'étape 0 : D7 si l'exécution démentait la lecture, une mutation qui survit sans test possible, un trou nouveau de couverture
relevé en jouant une mutation. Les survivantes **attendues** d'AC3 (champs permutés) et d'AC8 (départage par réglage, déjà tracé par **#1059**) ne s'ouvrent pas en issue nouvelle : la première est l'angle mort assumé de Q3,
la seconde a déjà son issue.

### Charge estimée
Moins d'une journée : neuf issues, dont huit courtes (S) et un M (#592 : 14 mutations), le gros du temps en compilations de mutations. Écart à la borne de l'epic : aucun.

### Références
`CLAUDE.md` (*Qualité du code*, *Review Iteration Rule*, *Priorités des défauts*) ; revues `kesh-gate-logs/321a-review-p1-{B,E}.md` ; mémoires *Tests qui prouvent moins*, *Mutation restaurée, binaire périmé*, *Pathspec git muet*.

## Dev Agent Record

### Agent Model Used
*(à remplir)*

### Debug Log References
*(journaux `kesh-gate-logs/e58/s582-*` et `kesh-gate-logs/idefix/<id>/`)*

### Completion Notes List
*(à remplir : par AC, ligne lue citée, mutation jouée, test qui la tue)*

### File List
*(à remplir)*

## Change Log

- 2026-11-27 — Fiche rédigée (Sonnet 5.5), `ready-for-dev` sous réserve de la validation P1 (Opus) ; dix issues retenues, aucune écartée ; deux à moitié couvertes (#732-1, #1433-2) ; les numéros de ligne de #736, #1066 et #1433 ont bougé, relus par la chaîne.
- 2026-11-27 — **Validation P1** (Opus 5.5, passe complète, contexte frais ; rapport `kesh-gate-logs/e58/58-2-validation-p1.md`) : **0 CRITICAL / 0 HIGH / 6 MEDIUM / 7 LOW**, tous remédiés dans la fiche.
  MEDIUM : V1-1 (D5/AC3 : `chart_designated_accounts` écrit les natures en dur, l. 703-723, ne parcourt pas `WriteOffNature::ALL` ; le plan est `crates/kesh-core/assets/charts/pme.json`, non un « preset de test ») ;
  V1-2 (D4/AC14 : la pathspec `crates/*/src/*` ne voyait ni les mutations du frontend ni `pme.json` — garde `git status --porcelain` à liste blanche) ; V1-3 (AC8 : la mutation `.then(setting)` survit, l'entrée de `check_written`
  arrivant déjà triée par réglage — renvoyée à #1059) ; V1-4 (AC13 : un parent valide ne prouve pas l'ordre, l'`UPDATE … AND version = ?` rend 409 ; mutation (b) déplacée au `.bind(parent_id)`) ; V1-5 (AC12 : le `grep` rend 25 lignes
  de routes et 0 de `kesh-db` — ensemble clos relevé depuis les sept tests) ; V1-6 (fichiers communs : l'Epic 53, non fusionné, touche `jour_suisse_nuit_e2e.rs`, `supplier_invoices_repository.rs`, `JournalEntryForm.*` et les
  sites de #1127, non `designated_account_type_e2e.rs`). LOW : V1-7 (espion sur le mock d'`onboarding.svelte`, non « dernière clé ») ; V1-8 (AC7 : messages en littéraux, compte ≥ 6) ; V1-9 (AC4 : ligne de B, `OR TRUE`, version égale) ;
  V1-10 (D6 : motif du montage par SQL) ; V1-11 (AC9 : survivante probable par `rollback`) ; V1-12 (journaux sous `kesh-gate-logs/e58/`) ; V1-13 (`accounts.rs:4065`, `:1280`). Q1-Q3 tranchées par l'orchestrateur
  (oui, oui, oui) ; Q4 tranchée au code : l'avoir ne contrôle pas le type de la TVA due, comme le dit le manuel (PDF compris) — aucune issue. Prémisses confirmées : les dix issues sont ouvertes et non couvertes ;
  D1 (aucun `crates/*/src/*` au diff de la fiche) ; D6 (la validation lit le réglage) ; D7 (l'avoir passe) ; `sitesTotal` inchangé (`dansLePerimetreDeFichier` exclut `.test.`). Passe P2 (Sonnet, complète) à lancer :
  `58-2-comptes-designes-tests-revue-validation-prompt-p2.md`.
- 2026-11-27 — **Validation P2** (Sonnet 5.5, passe complète, contexte frais ; rapport `kesh-gate-logs/e58/58-2-validation-p2.md`) : **0 CRITICAL / 0 HIGH / 1 MEDIUM / 3 LOW**, tous remédiés dans la fiche. Les six remédiations de la P1 (V1-1 à V1-6) tiennent au code ;
  pas de recyclage (le MEDIUM est d'origine neuve : un site non résolu de l'ensemble clos d'AC12, que la P1 avait fermé par un `grep` limité aux routes). MEDIUM : V2-1 (AC12 : la couche `services/` lit la date par `lecture.aujourd_hui()`, `services/factures.rs:582`,
  jambe `GET /settlements` d'`ac3_sans_exercice_…` : contrôle de complétude élargi à `crates/kesh-api/src`). LOW : V2-2 (release visée v0.20.1 et ordre d'entrée avant l'Epic 53, ce qui se rejoue si l'ordre change ; l'Epic 53 touche aussi `kesh-db/src/{repositories/accounts,errors}.rs` : lignes qui bougent, comportement
  inchangé aux sites mutés) ; V2-3 (`accounts.rs:4062` → `4065`) ; V2-4 (AC13(a) : forme compilable de la mutation). Tendance : 0/0/6/7 → 0/0/1/3. Passe P3 (ciblée, Haiku, sur le seul commit de remédiation de la P2, fiche seule) : `58-2-comptes-designes-tests-revue-validation-prompt-p3.md`.
- 2026-11-27 — **Sortie de #1127** (décision du Project Lead, signal de recyclage déclaré à la P3 ciblée ; rapport `$GATE_LOGS/e58/58-2-validation-p3-ciblee.md`, Haiku, 0 CRITICAL / 0 HIGH / 1 MEDIUM / 2 LOW). Tendance : P1 0/0/6/7 → P2 0/0/1/3 → P3 ciblée 0/0/1/2 → P4 ciblée 0/0/0/5 (Sonnet) ; le MEDIUM
  des trois passes porte sur le même ensemble clos d'AC12, dont l'inventaire déborde à chaque passe : **recyclage déclaré**. #1127 sort de la story (§ *Sortie de #1127*) : l'AC12, ses tâches, ses mutations (M3/M7), D4 dans sa part #1127, ses Dev Notes et son tableau de sites sont retirés ;
  la story garde **neuf** issues (#592, #732, #736, #1060, #1062, #1063, #1064, #1066, #1433) et ne touche toujours que des tests (`jour_suisse_nuit_e2e.rs` sort du périmètre). F3-1 (décompte 25/41/54, 19 sites non classés) : repris au point de départ de la future story ;
  F3-2 (Epic 53 : trois hunks, +24 lignes, dans `errors.rs` ; `accounts.rs` dès la l. 461) et F3-3 (`sprint-status.yaml`) : corrigés. Les entrées ci-dessus gardent la numérotation d'AC d'alors (AC12 vacante). Passe P4 (ciblée, Sonnet ; une première P4 Haiku, non conforme à la règle d'alternance, est écartée, `$GATE_LOGS/e58/58-2-validation-p4-ciblee.md`) :
  `58-2-comptes-designes-tests-revue-validation-prompt-p4.md`.
