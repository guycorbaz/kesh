# Story 36-7 : les tests ne figent plus 2026 — verts au 1er janvier 2027 (#1125, KF-060)

Status: split

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-7`, branche
`story/36-7-tests-desancres-2026`, partie de `epic/36-0.18.0` (`0bf3c3b4b`), par le workflow `bmad-create-story`. Release
**0.18.0**, phase 2, rang 7 et **première de la phase** (D1 du 2026-11-15). **Aucune migration**, aucune ligne de code de
production, aucune clé i18n, aucun changement d'API. Code lu à la tête du worktree, **`0bf3c3b4b`**. Échéance ferme : **à
terminer d'ici le 2026-12-28** (décision de Guy, Q1 ; C-e360-13). Voie de base **2** (`kesh-mariadb-dev2:3307`), `gate-ci.sh`
en voie 1. Clé courte des bases : `e367`. Choix propres à la story : `e36-choix-autonomes-36-7.md` (C-36-7-1 à C-36-7-6).
Validation P1 remédiée le 2026-11-15 (Change Log). Journaux de la fiche : `/home/gcorbaz/devel/kesh-gate-logs/e36-7/`.*

## Story

En tant que mainteneur de Kesh, je veux que la suite de tests — Rust, Vitest, Playwright — reste verte le 1er janvier 2027
et après, sans qu'aucun test ne fige l'année 2026 contre la date du jour, afin que la CI ne devienne pas rouge au premier
jour de l'année où Guy clôturera son premier exercice, et qu'un rouge de 2027 reste le signe d'un défaut, non d'un calendrier.

## Ce qui se lit au code (`0bf3c3b4b`)

### La plomberie posée par les 35-5a0, 35-5a et 35-5c

- `crates/kesh-core/src/horloge.rs:23` — `pub const NOM_VARIABLE_INSTANT: &str = "KESH_TEST_INSTANT";` ; `:73`
  `pub fn instant_de_test(valeur: Option<&str>) -> Option<DateTime<Utc>>`, qui panique sur une valeur invalide (`:80`
  `Err(e) => panic!("{NOM_VARIABLE_INSTANT} n'est pas un instant RFC 3339 ({valeur:?}) : {e}")`) ; `:58` `pub fn
  jour_suisse(instant: DateTime<Utc>) -> NaiveDate` — le jour de Zurich.
- `crates/kesh-api/tests/common/jour.rs:14-15` — `pub fn aujourd_hui_test() -> NaiveDate {
  kesh_api::horloge::horloge_de_test().aujourd_hui() }` ; `crates/kesh-db/tests/support/jour.rs:17-22` et
  `crates/kesh-report/tests/support/jour.rs` — le même jour, tiré de `KESH_TEST_INSTANT` s'il est posé, de l'heure réelle
  sinon (`instant_de_test(std::env::var(NOM_VARIABLE_INSTANT).ok().as_deref()).unwrap_or_else(Utc::now)`).
- `docs/testing.md:119` — « `KESH_TEST_INSTANT` est **sans effet sur le binaire servi** : `main.rs` construit l'horloge
  réelle et ne la lit jamais » ; et « ⚠️ **Trois exceptions restent** […] (a) les tests à **bornes fixes de 2026** (#1125,
  dont deux d'`admin_full_import_e2e`, bornes `2026-03-31` et `2026-06-30`) rougissent sous un instant antérieur à leur
  borne ». Côté specs : « le jour se prend de `todayInZurich()` (`helpers/api-fixtures.ts`), un jour décalé
  d'`ajouterJoursIso` (`helpers/dates.ts`) ».
- `frontend/tests/e2e/helpers/api-fixtures.ts:392-393` — `export function todayInZurich(): string { return new
  Date().toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' }); }` : le runner Playwright lit **l'heure réelle**, pas
  `KESH_TEST_INSTANT`.
- `crates/kesh-api/tests/date_du_jour_garde.rs:469` (`HORLOGE_REELLE`) et `:473` (`HORLOGE_REELLE_ADMISES`) : deux lectures
  de l'heure réelle seulement dans les tests — `kesh-db/src/repositories/journal_entries.rs` (`setup()`) et
  `kesh-api/src/middleware/auth.rs` (`test_state()`). Ces deux modules **ne lisent pas** `KESH_TEST_INSTANT`.

**Conséquence, qui structure la fiche** : le critère du fichier d'epic (rang 7 : « `scripts/test-fast.sh --ci` et l'E2E
complet verts sous `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, sonde positive jointe ») est **exécutable à la lettre pour
nextest**, mais **sans effet pour l'E2E** : ni le backend servi, ni le runner, ni le navigateur ne lisent la variable. D3
dit comment l'E2E se joue en 2027.

### L'inventaire par exécution — nextest sous l'instant du critère, sans correctif

Détecteur par exécution (`e35-dev-commun.md` § *Inventaires*, 1) : toute la suite, à la tête du worktree, sous l'instant
du critère, voie 2, base `kesh_e367` remise à neuf. Script versionné hors dépôt :
`/home/gcorbaz/devel/kesh-gate-logs/e36-7/inventaire-36-7.sh` ; journal `inventaire-A2027.log`.

```sh
/home/gcorbaz/devel/kesh-gate-logs/e36-7/inventaire-36-7.sh A2027 2027-01-04T10:00:00Z
# → cargo nextest run --workspace --profile ci --no-fail-fast, KESH_TEST_INSTANT=2027-01-04T10:00:00Z, base 3307/kesh_e367
```

Sortie copiée (`/usr/bin/grep -E '^\s+(FAIL|TRY|TIMEOUT|SIG)|Summary' inventaire-A2027.log`, `TRY 1` seuls, motif du
`panicked` ajouté) :

```
# commit 0bf3c3b4b docs(epic-36) : kickoff de la 0.18.0 — décisions de Guy du 2026-11-15
# 2026-11-15T14:01:20+01:00 étiquette=A2027 KESH_TEST_INSTANT=2027-01-04T10:00:00Z
  TRY 1 FAIL kesh-api::admin_full_import_e2e full_import_accepts_an_rc1_backup_with_supplier_invoices   (:4972 FISCAL_YEAR_INVALID)
  TRY 1 FAIL kesh-api::admin_full_import_e2e full_import_accepts_a_0_16_backup_with_supplier_invoices  (:4972 FISCAL_YEAR_INVALID)
  TRY 1 FAIL kesh-api::bank_statements_manual_e2e manual_statement_lines_reconcile_like_imported_ones  (:1149 FISCAL_YEAR_INVALID)
  TRY 1 FAIL kesh-api::dates_bornes_e2e c_facture_echeance          (:573 témoin → 400 « L'échéance ne peut pas être antérieure à la date de la facture »)
  TRY 1 FAIL kesh-api::dates_bornes_e2e c_facture_maj_echeance      (idem)
  TRY 1 FAIL kesh-api::dates_bornes_e2e c_fournisseur_echeance      (idem)
  TRY 1 FAIL kesh-api::dates_bornes_e2e c_fournisseur_maj_echeance  (idem)
  TRY 1 FAIL kesh-db::journal_entries_modification delete_waits_for_a_concurrent_reversal_then_refuses  (:498 contre-passation de B: FiscalYearInvalid)
  TRY 1 FAIL kesh-db::journal_entries_modification update_waits_for_a_concurrent_reversal_then_refuses  (:384 idem)
  TRY 1 FAIL kesh-db::migrations_upgrade_path draft_migration_preserves_a_populated_016_base  (:1521 annulée (contre-passation): FiscalYearInvalid)
  TRY 1 FAIL kesh-db::migrations_upgrade_path draft_migration_preserves_a_populated_rc1_base   (idem)
  TRY 1 FAIL kesh-db::opening_complement_repository un_compte_contre_passe_reste_mouvemente     (:531 FiscalYearInvalid)
  TRY 1 FAIL kesh-report::open_items_invariant open_items_balance_matches_the_ledger_and_the_trial_balance  (:181 lettrage: LetteringAllLinesInClosedPeriods)
     Summary [ 345.592s] 5452 tests run: 5439 passed (1 slow), 13 failed, 13 skipped
rc=100
```

Ce sont **exactement les treize** de la sonde de jour du 2027-01-15 de la 35-5a (`kesh-gate-logs/e355a/nuit-C8.log`,
5357 tests, 13 failed), relevée à un commit antérieur : aucun site neuf depuis, aucun guéri. La présence de ces rouges est
la **sonde positive** du relevé : l'instant est bien lu (à l'heure réelle, les mêmes tests sont verts — gate de la 0.17.8,
nextest 5452 exécutés, 13 ignorés, `kesh-gate-logs/rel0178/`).

**Les sites fautifs, ligne lue** (le montage fige 2026, l'opération se date au jour du test) :

| Test(s) | Site du montage | Ce qui se date au jour |
|---|---|---|
| `admin_full_import_e2e` ×2 | `:4899-4901` `v0174a_decor` : `VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')` ; pièces `"invoiceDate": "2026-06-15"` (`:4990`), `"paymentDate": "2026-06-20"` (`:5039`) | l'annulation `/api/v1/supplier-invoices/3/cancel` (contre-passation au jour) |
| `bank_statements_manual_e2e` | `:218-220` `VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')` | l'écriture passée à `:1149` |
| `dates_bornes_e2e` ×4 | témoin `"2027-01-01"` (`:1010`, `:1050`, `:1510`, `:1556`) pour `dueDate`, face à `"date": aujourd_hui()` (`:1017`) | la facture du jour : au 2027-01-04, l'échéance témoin la précède |
| `journal_entries_modification` ×2 | `:268-275` exercice du seed « ramené à l'année 2026 » (`UPDATE fiscal_years SET name = 'Exercice 2026', start_date = '2026-01-01', end_date = '2026-12-31'`) ; écritures `d(2026, 3, 1)` (`:373`, `:426`, `:487`, …) | la contre-passation `crate::jour::aujourd_hui_test()` (`:381`) |
| `migrations_upgrade_path` ×2 | base 0.16 / rc1 peuplée, `day = … from_ymd_opt(2026, 6, 20)` (`:1498`) | `cancel_in_tx(…, crate::jour::aujourd_hui_test())` (`:1515-1520`) |
| `opening_complement_repository` | `year_span(2026)` (`:47-49`, appelé `:246`, `:277`, `:298`, `:311`, `:331`, `:346`, `:355`, `:383`, …) | `journal_entries::reverse(…, crate::jour::aujourd_hui_test())` (`:523-528`) |
| `open_items_invariant` | exercices 2024 (clos) et 2025-2030 (`:106-116`) ; groupe « à cheval » `2024-06-30` / `2026-03-01` (`:203`) ; écritures à `jours_avant(n)` | le lettrage `:170-181` refusé `LetteringAllLinesInClosedPeriods` — **cause exacte à lire à l'étape 0** (ligne citée au Change Log) |

### Les sites qui rougissent sous un instant antérieur à leur date de 2026 (relevé par exécution)

*Section récrite le 2026-11-15 à la remédiation de la validation P1 (V1-1). Elle ne portait que les trois sites des
commentaires de #1125, pris de journaux **filtrés** (`e355c/nuit-B2.log`, huit binaires ; `e355b/nuit-B3.log`, un).* Ces
tests figent une date de 2026 et rougissent sous un instant **antérieur** à elle, jamais à l'heure réelle, qui ne recule
pas. Détecteur par exécution, suite entière, à la tête, même script et même base remise à neuf :

```sh
/home/gcorbaz/devel/kesh-gate-logs/e36-7/inventaire-36-7.sh B2026 2026-03-10T23:30:00Z N2027 2026-12-31T23:30:00Z
```

Sortie copiée (`inventaire-B2026.log`, `TRY 1` seuls, ligne du `panicked` ajoutée) :

```
# commit d9a61b63f docs(36-7-tests-desancres-2026) : fiche — désancrer de 2026 les tests à date ou exercice fixe (refs #1125)
# 2026-11-15T14:22:50+01:00 étiquette=B2026 KESH_TEST_INSTANT=2026-03-10T23:30:00Z
  TRY 1 FAIL kesh-api::admin_full_import_e2e full_import_letters_a_reopened_historical_piece          (:2009)
  TRY 1 FAIL kesh-api::admin_full_import_e2e validating_a_backdated_invoice_is_refused_by_the_period_lock (:2282)
  TRY 1 FAIL kesh-api::invoice_echeancier_e2e settlement_cancel_blocked_by_closed_lettering           (:2022)
  TRY 1 FAIL kesh-api::project_reports_e2e project_expenses_cumulative_mode                           (:242 left "0", right "120.0000")
  TRY 1 FAIL kesh-api::reconciliation_e2e unreconcile_of_a_closed_lettering_is_refused_in_its_family  (:7314)
  TRY 1 FAIL kesh-api::services_rollout ac11_a_les_services_lisent_en_lecture_seule                  (:1347)
  TRY 1 FAIL kesh-api::supplier_invoice_draft_e2e the_open_items_document_carries_both_numbers       (:1877)
  TRY 1 FAIL kesh-api::supplier_settlement_cancel_e2e cancel_blocked_by_closed_lettering             (:844)
  TRY 1 FAIL kesh-db::supplier_invoices_draft cancelled_invoice_returns_to_draft                     (:1789 « montage : l'achat précède la veille de l'annulation »)
     Summary [ 319.797s] 5452 tests run: 5443 passed (1 slow), 9 failed, 13 skipped
```

Les sites, ligne lue (une date fixe de 2026 face au jour du test) :

| Test | Site | Ce qui se date au jour |
|---|---|---|
| `admin_full_import_e2e::full_import_letters_a_reopened_historical_piece` | `:2007` `let borne = chrono::NaiveDate::from_ymd_opt(2026, 3, 31)` | `assert!(borne < crate::common::jour::aujourd_hui_test(), "montage : borne passée")` (`:2009-2011`) |
| `admin_full_import_e2e::validating_a_backdated_invoice_is_refused_by_the_period_lock` | `:2281` `from_ymd_opt(2026, 6, 30)` | même garde (`:2282-2285`) |
| `invoice_echeancier_e2e::settlement_cancel_blocked_by_closed_lettering` | `:2018` `NaiveDate::from_ymd_opt(2026, 4, 15)`, borne de `lock_books` (`:2014`) | `lock_books(…, crate::common::jour::aujourd_hui_test())` (`:2019`) : `booksLockBoundNotPast` |
| `reconciliation_e2e::unreconcile_of_a_closed_lettering_is_refused_in_its_family` | `:7283` `let paid_on = NaiveDate::from_ymd_opt(2026, 5, 15)` | `lock_books` (`:7306`) |
| `supplier_settlement_cancel_e2e::cancel_blocked_by_closed_lettering` | `:809` `let paid_on = NaiveDate::from_ymd_opt(2026, 6, 20)` | `lock_books` (`:836`) |
| `project_reports_e2e::project_expenses_cumulative_mode` | `:131` `entry_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1)` | rapport cumulatif (`:233` `mode=cumulative`) au jour : l'écriture en sort (`:242`) |
| `services_rollout::ac11_a_les_services_lisent_en_lecture_seule` | `:207` `entry_date: NaiveDate::from_ymd_opt(2026, 6, 1)` (`poser_livres`, `:199`) | postes ouverts à l'`asOf` par défaut (`:1251`, `:1347`) |
| `supplier_invoice_draft_e2e::the_open_items_document_carries_both_numbers` | `:1853` `json!({ "paymentDate": "2026-06-20" })` (et `"requestedExecutionDate": "2026-06-20"`, `:1800`, `:1843`) | `GET /api/v1/accounts/{id}/open-items` sans `asOf` (`:1870`) : la pièce n'y est plus |
| `kesh-db::supplier_invoices_draft::cancelled_invoice_returns_to_draft` | `:1778` `let date = jour(2026, 6, 15);` | `assert!(veille >= date, …)` (`:1788-1791`), `veille = aujourdhui().pred_opt()` |

Sous la nuit du 1er janvier 2027 (`inventaire-N2027.log`, `KESH_TEST_INSTANT=2026-12-31T23:30:00Z`, jour suisse
2027-01-01) : `5452 tests run: 5443 passed (1 slow), 9 failed, 13 skipped` — **neuf des treize** du tableau de 2027, tous
sauf les quatre de `dates_bornes_e2e`, dont le témoin `"2027-01-01"` n'y précède pas la date du jour (il lui est égal).
Aucun rouge hors des treize.

Les deux listes (treize sous 2027, neuf sous l'instant antérieur) sont **disjointes** : **vingt-deux tests** au
périmètre, dans quatorze fichiers (§ *Dev Notes*).

**Rouge de charge exclu** : `mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris` figure aux journaux de la 35-5a
(`e355a/nuit-C2.log`, `nuit-C2b.log`, `nuit-C3.log`, « témoin : sonde servie ») et à **aucun** des trois relevés de cette
fiche : test à marges de temps, sans date de 2026, **hors périmètre**. S'il rougit pendant la story, il se nomme comme
rouge de charge et se relance ; il n'est pas un site de #1125.

### Les angles morts du détecteur par `KESH_TEST_INSTANT`, relus

Un test qui ne lit pas la variable n'est pas mis à 2027 par elle. Détecteur par lecture, parti du symptôme (une lecture de
l'heure réelle dans un test), sur les modules de tests de `src` :

```sh
cd /home/gcorbaz/devel/kesh-36-7 && for f in $(git grep -lE 'Utc::now|SystemTime::now|NOW\(\)|CURRENT_TIMESTAMP' -- 'crates/*/src/*' 'crates/*/src/**'); do t=$(/usr/bin/grep -nE '^\s*#\[cfg\(test\)\]' $f | head -1 | cut -d: -f1); case $f in *tests.rs|*_tests.rs|*/tests/*|*test_fixtures.rs) t=0;; esac; [ -z "$t" ] && continue; /usr/bin/grep -nE 'Utc::now|SystemTime::now|NOW\(\)|CURRENT_TIMESTAMP' $f | while IFS=: read ln rest; do [ "$ln" -gt "$t" ] && echo "$f:$ln:$(echo "$rest" | cut -c1-140)"; done; done | /usr/bin/grep -vE ':\s*(//)'
```

Sortie, triée (les lignes DDL `ON UPDATE CURRENT_TIMESTAMP` de `post_restore.rs` et `sql_lexical.rs`, et l'écriture
`last_boot_at = NOW()` de `version.rs:325`, ne datent rien au jour) :

| Site | Verdict, ligne lue |
|---|---|
| `kesh-db/src/repositories/journal_entries.rs` (`setup()`, `HORLOGE_REELLE_ADMISES`) | jour réel, exercice de la base de dev 2020-2030 (`kesh-db/src/test_fixtures.rs:116-119` « Fiscal year (2020-2030, large pour tolérer dérive d'horloge CI) ») : **juste en 2027** |
| `kesh-api/src/middleware/auth.rs:302` | `HorlogeFixe(chrono::Utc::now())`, tests du middleware qui ne datent rien au jour : **juste** |
| `kesh-db/src/repositories/invoices.rs:5263`, `:5324`, `:5413`, `:5476` | `set_paid_at_for_test(…, Utc::now().naive_utc())` face à `today()` fixe (`:3563-3565`, `2026-06-15`) ; le filtre ne lit que `paid_at IS NOT NULL` / `IS NULL` (`:393`, `:396`, `:406`) : **juste** |
| `kesh-db/src/repositories/reconciliation_link.rs:1264` | `now` d'une structure en mémoire : **juste** |
| `kesh-db/src/repositories/journal_entries.rs:5202` | suffixe d'unicité (`timestamp_nanos_opt`) : **juste** |
| `kesh-api/src/auth/jwt.rs:91`, `:118`, `:192`, `:314` ; `auth/bootstrap.rs:806`, `:809` ; `auto_backup.rs:1087` | instants relatifs (expiration, `+ 1 jour`), sans année : **juste** |

Ce relevé par lecture ne prouve pas l'absence : AC4 rejoue la suite sous une **horloge de processus** décalée (D3), qui
couvre ces modules par exécution. **Sonde** de la lecture : la commande rend bien `middleware/auth.rs:302`, site connu de la
garde.

### Vitest — par exécution, vert en 2027

Vitest ne lit pas `KESH_TEST_INSTANT`. Détecteur par exécution, hors dépôt : un préchargement Node qui décale `Date`
(`/home/gcorbaz/devel/kesh-gate-logs/e36-7/date-decalee-36-7.mjs`, `NODE_OPTIONS="--import …"`, variable
`KESH_TEST_INSTANT_JS`), le temps continuant de s'écouler ; `vi.useFakeTimers()` d'un test le remplace.

```sh
cd /home/gcorbaz/devel/kesh-36-7/frontend && TMPDIR=/tmp/kesh-voie2 KESH_TEST_INSTANT_JS=2027-01-04T10:00:00Z \
  NODE_OPTIONS="--import /home/gcorbaz/devel/kesh-gate-logs/e36-7/date-decalee-36-7.mjs" \
  flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock ../scripts/mem-guard.sh npx vitest run
```

Sortie copiée (`inventaire-vitest-A2027.log`) :

```
 Test Files  238 passed (238)
      Tests  3709 passed (3709)
rc=0
```

**Sonde positive** (`inventaire-vitest-sonde.log`) : un test hors dépôt
(`kesh-gate-logs/e36-7/sonde-vitest/annee.test.js`, `expect(new Date().getUTCFullYear()).toBe(2027)`) lancé par le même
Vitest : sans la variable, `AssertionError: expected 2026 to be 2027` (`rc=1`) ; avec, `Tests 1 passed (1)` (`rc=0`), y
compris sous `--environment jsdom` (celui de `frontend/vite.config.ts:58`). Le préchargement atteint donc les processus de
test. **Aucun site Vitest n'est au périmètre.** Les tests qui lisent l'heure et la comparent à une date de 2026 l'ont
arrêtée eux-mêmes (validation P1, V1-6) : `src/routes/(app)/bank-accounts/bank-accounts-page.test.ts:713-714`
(`vi.useFakeTimers({ toFake: ['Date'] })`, `vi.setSystemTime(new Date('2026-10-13T22:30:00Z'))`, puis `:718`
`toBe('2026-10-13')`) et `src/lib/features/supplier-invoices/SupplierInvoiceForm.test.ts:870` (`vi.setSystemTime(new
Date('2026-11-04T23:30:00Z'))`, lu à `:887`) ; `bank-accounts-page.test.ts:413-416` (`aujourdhui()`) compare le champ au
jour qu'il lit lui-même, sans année fixe. Tous trois sont justes à toute date.

### Playwright — par lecture (l'exécution en 2027 est l'AC5)

```sh
cd /home/gcorbaz/devel/kesh-36-7 && git grep -cE '\b2026\b' -- frontend/tests/e2e
```

Rend 33 fichiers (specs, helpers, fixtures CAMT/CSV). Ce qui borne le risque, ligne lue :

- l'exercice des presets `with-company` est **2020-2030** (`crates/kesh-db/src/test_fixtures.rs:118-119` `VALUES (?,
  'Exercice CI 2020-2030', '2020-01-01', '2030-12-31', 'Open')`, appelé par `seed_accounting_company` depuis
  `crates/kesh-api/src/routes/test_endpoints.rs:201`) : une pièce datée de 2026 reste dans un exercice ouvert en 2027 ;
- les specs qui créent l'exercice de l'année le prennent du jour (`api-fixtures.ts:401` `const year =
  todayInZurich().slice(0, 4);`) ;
- les fenêtres de rapprochement se comptent depuis la date du relevé, non du jour (`reconciliation-amount-due.spec.ts:81`
  « Date de facture au plus 30 jours avant le relevé (2026-05-15) ») ; `fiscal-year-closing-journey.spec.ts:30` « N+1 est
  2026 quelle que soit l'année courante ».
- `frontend/tests/e2e/supplier-internal-number.spec.ts:237` attend `KF-2026-0001` — **il ne rougira pas en 2027** (le
  commentaire de la 35-5c sur #1125 le croyait) : l'aperçu est figé en 2026 **dans le code de production**,
  `frontend/src/routes/(app)/settings/invoicing/+page.svelte:159` `previewInvoiceNumber(format, 2026, '2026', 1)`, et de même
  `:173` et `:180`. C'est un site de **production** : hors périmètre (§ *Hors périmètre*), issue demandée.

Aucune spec n'est identifiée comme fautive par lecture ; ce n'est pas une preuve, d'où AC5. **Premières à lire en cas de
rouge sous AC5** (validation P1, V1-6) : `supplier-invoice-draft-cycle.spec.ts` (dates `2026-05-10` à `2026-07-31`,
`:129` à `:522`, dont `paymentDate: '2026-05-20'` `:478`), `reconciliation-supplier.spec.ts:110-111` (`invoiceDate:
'2026-02-10'`, `dueDate: '2026-03-12'`), `product-revenue-account.spec.ts:190`, `:262` (`date: '2026-06-15'`),
`fiscal-year-closing-journey.spec.ts:177` (`createFiscalYear(ctx, 2026)`).

### Scripts et production

- `scripts/recette-montee.sh:476` `JOUR=$(date +%F)` et preset `with-company` (`:473`) : les relevés de recette d'octobre
  2026 (`scripts/fixtures/recette/releve-A.xml:6`) restent dans l'exercice 2020-2030 : **juste**. `scripts/tests/*.test.sh` :
  dates de 2026 en données, comparaisons au jour lu (`prepare-release.test.sh:103` `DATE_AVANT=$(date +%Y-%m-%d)`) : **juste**.
- Production Rust : `git grep -lE '\b2026\b' -- 'crates/*/src/*' 'crates/*/src/**'` (95 fichiers), lignes avant le premier
  `#[cfg(test)]`, hors fichiers de tests et commentaires : **un seul** site, `crates/kesh-api/src/qr_decode_synthetique.rs:112`,
  texte d'un générateur de test (`:1` « Scans synthétiques de QR-factures — générateur de test »). **Aucun site de
  production Rust ne fige 2026.**
- Production frontend : outre l'aperçu ci-dessus, `frontend/src/routes/(app)/contacts/+page.svelte:1067`
  `placeholder="CLI-2026-00042"` — un exemple de saisie, non une date : **juste** (choix C-36-7-5).

## Décisions

- **D1 — Le montage prend ses années du jour du test (N = année de `aujourd_hui_test()`) ; une date qui doit précéder le
  jour se place dans N-1, jamais dans N.** *(Récrite à la remédiation de la validation P1, V1-2 : la version d'origine
  proposait « le 1er janvier de l'année du jour », qui **est** le jour sous `2026-12-31T23:30:00Z` — jour suisse
  2027-01-01 — et fait alors rougir les gardes « borne passée » et `veille >= date`.)* Règle, juste pour tout jour d'une
  année :
  - l'exercice du montage couvre **N-1 et N** (deux exercices civils, ou un seul qui couvre les deux), N+1 en plus là où le
    test le demande (`journal_entries_modification:268` « pour qu'un exercice 2027 puisse le suivre ») ;
  - pièces, paiements et bornes de verrou qui doivent être **passés** se datent dans **N-1** (par exemple `d(N-1, 6, 15)`,
    borne `d(N-1, 12, 31)`) ; l'opération du jour (contre-passation, annulation, verrou) garde `aujourd_hui_test()` ;
  - aucune date « fixe dans l'année du jour » ni décalage `aujourd_hui - n` qui puisse traverser le 1er janvier.
  **Modèle** déjà au dépôt : `crates/kesh-db/tests/supplier_invoices_draft.rs:2059` `let date = jour(x.y - 2, 6, 15);`,
  avec `scinder` (`:1522-1547`) qui pose les exercices `y - 2`, `y - 1` et `y` depuis `let y = aujourdhui().year();`
  (`:1524`) et garde `assert!(y <= 2029, …)` (`:1525-1528`). Le témoin de `dates_bornes_e2e` devient une échéance
  **postérieure** à la date du jour (`aujourd_hui() + n` jours). Là où un montage « base 0.16 / rc1 peuplée »
  (`migrations_upgrade_path`) ou un exercice existant du seed ne peut pas être déplacé, l'étape 0 le dit, ligne lue.
  *Écartée* : passer à chaque opération une date fixe de 2026 (`cancel(…, 2026-06-20)`) — le test resterait juste, mais
  cesserait d'exercer la date du jour que la 35-5c lui a donnée ; *écartée* aussi : un exercice 2020-2030 dans chaque
  montage — il masquerait le jour et repousse la panne à 2031.
- **D2 — Les neuf tests qui rougissent sous un instant antérieur à leur date de 2026 entrent au périmètre** (C-36-7-3,
  amendé) : ceux du tableau de la section *Les sites qui rougissent sous un instant antérieur*, relevés par la suite entière
  sous `2026-03-10T23:30:00Z` — `admin_full_import_e2e` (2), `invoice_echeancier_e2e`, `reconciliation_e2e`,
  `supplier_settlement_cancel_e2e`, `project_reports_e2e`, `services_rollout`, `supplier_invoice_draft_e2e`, et
  `kesh-db::supplier_invoices_draft`. Leur date suit D1 (dans N-1), ou l'`asOf` du montage se pose explicitement.
  *Écartées* : les trois seuls de #1125 (inventaire à trous, validation P1 V1-1) ; retirer l'instant antérieur de l'AC2
  (l'heure réelle ne recule pas) — la campagne d'instants des 35-5a à 35-5c le joue, et ces neuf rouges y resteraient
  attendus.
- **D3 — L'E2E en 2027 se joue sous une horloge de processus (libfaketime), non sous `KESH_TEST_INSTANT`** (C-36-7-2). Le
  binaire servi ignore la variable (`docs/testing.md:119`, garde `date_du_jour_garde.rs`) et le runner lit l'heure réelle
  (`api-fixtures.ts:393`). Le backend et le runner se lancent sous
  `TZ=Europe/Zurich FAKETIME_DONT_FAKE_MONOTONIC=1 faketime -f '@2027-01-04 11:00:00' <commande>` — `faketime` lit la date
  dans le fuseau du processus : « 11 h » ne vaut 10:00Z qu'avec `TZ=Europe/Zurich` fixé dans la commande (V1-5). Que
  Chromium hérite de `LD_PRELOAD` est une **hypothèse non éprouvée** (le navigateur de Playwright lance ses processus avec
  son bac à sable) : la sonde de l'AC5 la tranche. MariaDB, en conteneur, reste à l'heure réelle (angle mort écrit :
  `NOW(3)` de `refresh_tokens.rs:61`, `:221`, `UTC_TIMESTAMP(6)` d'`invoice_reminders.rs:176`, `invoices.rs:2620`,
  `CURRENT_TIMESTAMP(3)` de `journal_entries.rs:1658`, `payment_batches.rs:640`, `DEFAULT CURRENT_TIMESTAMP` des colonnes
  d'horodatage — lignes relevées par la validation P1 et relues à sa remédiation ; liste non close). **Un rouge ne se rattache à cet angle mort
  que prouvé** : la ligne de SQL qui lit l'heure de la base, la colonne, et un rejeu isolé du test qui montre que le rouge
  tient à l'écart base/processus (par exemple : vert sous `KESH_TEST_INSTANT` seul, rouge sous faketime). Sans cette
  preuve, il compte comme un rouge de la story. **Repli si le navigateur reste en 2026** (étage 3 de la sonde) : aucun
  contournement n'est éprouvé ; l'AC5 se déclare alors **non constatable** pour la partie navigateur — non « vert » —,
  avec les étages serveur et runner joints, et l'orchestrateur arbitre (piste à éprouver, non prescrite : `page.clock`
  posé par un fixture). **Prérequis** : la bibliothèque, absente de la station
  (`ldconfig -p | /usr/bin/grep -i faketime` vide ; `apt-cache policy faketime` : « Installé : (aucun) », candidat
  `0.9.10+2024-06-05+gba9ed5b2-0.6build1`) — son installation est un geste de Guy (paquet système), demandé au rapport. La
  même horloge sert l'AC4 (nextest entier, angles morts compris). *Écartées* : faire lire `KESH_TEST_INSTANT` au binaire en
  `KESH_TEST_MODE` (code de production, contraire à la garde et au périmètre) ; `page.clock` de Playwright (le navigateur
  seul, le serveur resterait en 2026) ; déclarer l'E2E « vert en 2027 » sur la seule lecture.
  **AC4 et AC5 sont des gates de clôture** (C-36-7-6) : le développement **peut passer en `review` sans libfaketime**
  (AC1-AC3, AC6-AC9) ; la story **ne passe pas `done`**, et la PR de release **ne porte pas `closes #1125`**, tant
  qu'AC4 et AC5 n'ont pas tourné, journaux joints. Si la bibliothèque manque à l'étape 0, la **première ligne** du
  rapport de l'agent est la demande à Guy (installer le paquet `faketime`, geste système ; demande déjà transmise par
  l'orchestrateur le 2026-11-15).
- **D4 — Aucun outillage versionné de plus** (C-36-7-4). Les deux recettes (nextest et E2E sous horloge décalée, Vitest
  sous préchargement) s'écrivent dans `docs/testing.md`, à côté du paragraphe de `KESH_TEST_INSTANT` ; le préchargement Vitest
  et les scripts restent hors dépôt (`kesh-gate-logs/e36-7/`). *Écartée* : un `scripts/test-2027.sh` ou un setup Vitest qui
  lit `KESH_TEST_INSTANT` — utile pour la campagne de chaque fin d'année, mais c'est une fonction d'outillage neuve, qui
  élargirait une story que Guy veut courte ; à proposer en issue si l'orchestrateur le juge utile.
- **D5 — La cause du rouge d'`open_items_invariant` se lit avant d'écrire** : le lettrage de la fermeture `lettrer`
  (`:170-181`) est refusé `LetteringAllLinesInClosedPeriods`. Ce qui est établi (V1-8), ligne lue : la règle est celle de
  `crates/kesh-db/src/repositories/letterings.rs:89-94` (« Une ligne est **« en période ouverte »** si son exercice est
  `Open`, si aucun exercice postérieur n'est `Closed`, et si sa date est strictement postérieure à
  `companies.books_locked_through` ») ; la fermeture est appelée deux fois, `:185` (`p1`, `p2` à `jours_avant(60)` et
  `jours_avant(50)`, exercice `fy` 2025-2030) et `:220` (groupe à cheval `2024-06-30` / `2026-03-01`, avant la clôture de
  2024 à `:222`) ; le rouge apparaît sous `2027-01-04` et sous `2026-12-31T23:30:00Z`, non sous `2026-03-10T23:30:00Z`. Ce
  qui ne l'est pas : **lequel** des deux appels rougit, et pourquoi — la panique porte `:181` dans les deux cas, sans trace.
  L'étape 0 rejoue le test seul sous `RUST_BACKTRACE=1` et `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, cite la ligne qui
  produit le refus, et **seulement alors** choisit le correctif ; « il suit D1 » est une hypothèse, non un fait.

## Critères d'acceptation

1. **AC1 — nextest vert sous l'instant du critère.** `scripts/test-fast.sh --ci`, sous
   `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, voie 2, base remise à neuf : 0 échec (journal
   `kesh-gate-logs/e36-7/gate-2027.log`, commit cité). Les treize tests du premier tableau sont verts.
2. **AC2 — et sous trois autres instants, sans régression.** La même suite verte sous `2026-12-31T23:30:00Z` (nuit suisse
   du 1er janvier 2027, jour UTC encore 2026 : neuf rouges à la base, `inventaire-N2027.log`), `2026-03-10T23:30:00Z`
   (antérieur aux dates de D2 : les neuf tests de D2 y rougissent à la base, `inventaire-B2026.log`) et sans la variable
   (heure réelle). **Borne** : l'AC2 vaut pour ces trois réglages ; un instant antérieur au 2026-01-01 ou postérieur au
   2029-12-31 n'est pas visé (exercice de la base de dev 2020-2030, garde `y <= 2029` de `supplier_invoices_draft.rs:1525`).
   Un rouge de charge (§ *Rouge de charge exclu*) se relance et se nomme, sans compter contre l'AC.
3. **AC3 — Sonde positive de la lecture.** (a) Les trois relevés de la base (`inventaire-A2027.log`, 13 rouges ;
   `inventaire-B2026.log`, 9 ; `inventaire-N2027.log`, 9) sont rejoués à l'étape 0 à la tête de la branche et joints ;
   (b) après le correctif, les **seuls binaires corrigés** (`cargo nextest run -E 'binary(admin_full_import_e2e) | …'`, les
   quatorze binaires de la liste des fichiers) rougissent sous `KESH_TEST_INSTANT=pas-un-instant` (panique
   d'`instant_de_test`, message qui nomme la variable) : chaque binaire corrigé lit bien la variable ; (c) les mutations du
   tableau de § *Tests*, une par site, chacune tuée par le test nommé sous l'instant nommé.
4. **AC4 — nextest sous horloge de processus décalée** (D3 ; **gate de clôture**, C-36-7-6) : `cargo nextest run
   --workspace --profile ci` sous `TZ=Europe/Zurich FAKETIME_DONT_FAKE_MONOTONIC=1 faketime -f '@2027-01-04 11:00:00'`,
   **sans** `KESH_TEST_INSTANT`, binaires construits avant à l'heure réelle (`--no-run`) : 0 échec, ou chaque rouge nommé
   et rattaché — à un site de la story, ou à l'angle mort MariaDB **avec la preuve exigée par D3** et une issue. Sonde : un
   test du premier tableau ramené à sa version de la base rougit sous la même commande.
5. **AC5 — E2E complet en 2027** (D3 ; **gate de clôture**, C-36-7-6) : la suite Playwright complète, backend et runner
   sous la même horloge décalée (`TZ=Europe/Zurich` compris), base
   `kesh_e2e_e367` recréée : les échecs se nomment un par un contre `docs/testing.md` § *« Les échecs attendus »* (les sept
   de la KF-029, #97) ; aucun autre. **Sonde positive jointe** : avant la suite, un appel qui rend la date du serveur
   (`GET /api/v1/accounts/{id}/open-items` sans `asOf` : `asOf` vaut `2027-01-04`, `crates/kesh-api/src/routes/letterings.rs:388`),
   `todayInZurich()` du runner qui commence par `2027-`, et `new Date().getFullYear()` du navigateur à 2027
   (`page.evaluate`) — par un script hors dépôt, journal joint. Si l'étage navigateur rend 2026, l'AC5 suit le repli de
   D3 (non constatable pour le navigateur, arbitrage de l'orchestrateur), jamais « vert ».
6. **AC6 — Vitest reste vert en 2027** : le relevé de la fiche (`inventaire-vitest-A2027.log`) se rejoue au dernier commit
   de code, avec sa sonde.
7. **AC7 — Aucune ligne de production ne bouge** : `git diff --stat origin/epic/36-0.18.0...HEAD -- 'crates/*/src/*'
   'crates/*/src/**' frontend/src` vide (hors `#[cfg(test)]`/fichiers de tests, s'il en vient un — à justifier).
8. **AC8 — La documentation dit vrai** : `docs/testing.md:119` n'annonce plus l'exception (a) « tests à bornes fixes de
   2026 » ; il dit ce que `KESH_TEST_INSTANT` ne couvre pas (binaire servi, runner, navigateur, deux modules de tests de
   `src`) et donne les recettes de D3 et D4.
9. **AC9 — Garde intacte** : `date_du_jour_garde.rs` verte sans retouche de ses listes closes (aucune lecture neuve de
   l'heure dans les montages : le jour vient d'`aujourd_hui_test()`).

## Tests à écrire, et la preuve du rouge

La story n'ajoute pas de test : elle **corrige les montages** de vingt-deux tests. Le rouge de chacun est déjà constaté,
sans correctif, par les relevés de la fiche — treize sous `2027-01-04T10:00:00Z` (`inventaire-A2027.log`), neuf sous
`2026-03-10T23:30:00Z` (`inventaire-B2026.log`) — à rejouer à l'étape 0 à la tête de la branche, journal
`kesh-gate-logs/e36-7/rouges.log`.

**Mutations** (script versionné dans `kesh-gate-logs/e36-7/mut/`, code commité avant, restauration
`git restore --worktree -- <fichier>` puis `touch`) : **une par site corrigé** (ou par groupe de sites qui partagent un
même montage, dit comme tel) ; chacune remet la date ou l'exercice du site à sa forme de 2026 et doit être **tuée par le
test nommé, sous l'instant nommé** (journal `mut/M<n>.log`). Les sites de D2 ont leur forme de 2026 **antérieure** au
2027-01-04 : sous l'instant du critère, leur mutation ne rougirait pas — d'où l'instant prescrit ligne par ligne.

| M | Site corrigé | Test qui la tue | Instant |
|---|---|---|---|
| M1 | `admin_full_import_e2e` `v0174a_decor` (`:4899-4901` et ses pièces) — un groupe | `full_import_accepts_an_rc1_backup_with_supplier_invoices` (et `…_0_16_…`) | `2027-01-04T10:00:00Z` |
| M2 | `admin_full_import_e2e:2007` (borne) | `full_import_letters_a_reopened_historical_piece` | `2026-03-10T23:30:00Z` |
| M3 | `admin_full_import_e2e:2281` (borne) | `validating_a_backdated_invoice_is_refused_by_the_period_lock` | `2026-03-10T23:30:00Z` |
| M4 | `bank_statements_manual_e2e:218-220` | `manual_statement_lines_reconcile_like_imported_ones` | `2027-01-04T10:00:00Z` |
| M5 à M8 | `dates_bornes_e2e` témoins `:1010`, `:1050`, `:1510`, `:1556` — une mutation chacun | `c_facture_echeance`, `c_facture_maj_echeance`, `c_fournisseur_echeance`, `c_fournisseur_maj_echeance` (à apparier à l'étape 0, ligne lue) | `2027-01-04T10:00:00Z` |
| M9 | `journal_entries_modification` `societe()` (`:268-275`) — un montage pour deux tests | `update_waits_for_a_concurrent_reversal_then_refuses` (et `delete_…`) | `2027-01-04T10:00:00Z` |
| M10 | `migrations_upgrade_path` (`:1498` et l'exercice de la base peuplée) — un groupe | `draft_migration_preserves_a_populated_016_base` (et `…_rc1_…`) | `2027-01-04T10:00:00Z` |
| M11 | `opening_complement_repository` `year_span(…)` de `un_compte_contre_passe_reste_mouvemente` | ce test ; les autres appels de `year_span(2026)` (`:246` à `:383`) se mutent chacun s'ils sont corrigés, sinon l'étape 0 dit pourquoi ils restent justes | `2027-01-04T10:00:00Z` |
| M12 | `open_items_invariant` (site établi par D5) | `open_items_balance_matches_the_ledger_and_the_trial_balance` | `2027-01-04T10:00:00Z` |
| M13 | `invoice_echeancier_e2e:2018` | `settlement_cancel_blocked_by_closed_lettering` | `2026-03-10T23:30:00Z` |
| M14 | `reconciliation_e2e:7283` | `unreconcile_of_a_closed_lettering_is_refused_in_its_family` | `2026-03-10T23:30:00Z` |
| M15 | `supplier_settlement_cancel_e2e:809` | `cancel_blocked_by_closed_lettering` | `2026-03-10T23:30:00Z` |
| M16 | `project_reports_e2e:131` | `project_expenses_cumulative_mode` | `2026-03-10T23:30:00Z` |
| M17 | `services_rollout:207` | `ac11_a_les_services_lisent_en_lecture_seule` | `2026-03-10T23:30:00Z` |
| M18 | `supplier_invoice_draft_e2e:1853` (et `:1800`, `:1843`) — un groupe | `the_open_items_document_carries_both_numbers` | `2026-03-10T23:30:00Z` |
| M19 | `supplier_invoices_draft:1778` | `cancelled_invoice_returns_to_draft` | `2026-03-10T23:30:00Z` |

Une mutation qui ne rougit pas signale un montage qui ne dépend plus du jour : à dire (juste, ou test muet), jamais à
taire.

## Tâches

1. **Étape 0** : relire `e36-choix-autonomes-e360.md` (C-e360-13) et ce registre ; constater la présence de libfaketime
   (D3) ; rejouer les trois relevés de la fiche à la tête (`rouges.log`) ; lire la cause d'`open_items_invariant` (D5) et
   la citer.
2. Corriger les montages des sept fichiers du premier tableau (D1), puis les sept fichiers de D2.
3. Mettre à jour `docs/testing.md:119` (AC8) ; recettes de D3/D4.
4. Mutations M1 à M19 (AC3 c).
5. Gates (§ *Dev Notes*), AC1, AC2, AC3, AC6 → `Status: review`, clé `review`, Dev Agent Record, prompt de la revue P1.
   **Sans libfaketime, la story s'arrête là** (C-36-7-6).
6. À la clôture : `gate-ci`, E2E complet à l'heure réelle, **puis AC4 et AC5** ; ensuite seulement `done`, et
   `closes #1125` à la PR de release.

## Note de version

*Modifié* — Les tests de Kesh ne figent plus l'année 2026 : la suite reste verte à partir du 1er janvier 2027. Aucun
changement pour l'utilisateur. (refs #1125)

## Dev Notes

### Hors périmètre

- **L'aperçu du format de numérotation montre toujours l'année 2026** — `frontend/src/routes/(app)/settings/invoicing/+page.svelte:159`,
  `:173`, `:180` (`previewInvoiceNumber(format, 2026, '2026', 1)`) : en 2027, *Paramètres → Facturation* affiche
  `FF-2026-001` comme exemple du prochain numéro. Code de production : **issue à ouvrir** (P4, `bug`, comportement visible).
  Si elle est corrigée, `supplier-internal-number.spec.ts:237` (`KF-2026-0001`) devra suivre.
- **L'exercice de test 2020-2030** (`crates/kesh-db/src/test_fixtures.rs:118-119`), qui porte la base de dev, les E2E et la
  recette : il expirera le 2030-12-31. **Issue à ouvrir** (P4, `technical-debt`) — hors de l'horizon de la story.
- Le placeholder `CLI-2026-00042` (`contacts/+page.svelte:1067`) : exemple de saisie, laissé (C-36-7-5).

### Modules touchés (règle de splitting)

`kesh-api` (tests, neuf fichiers : `admin_full_import_e2e.rs`, `bank_statements_manual_e2e.rs`, `dates_bornes_e2e.rs`,
`services_rollout.rs`, `invoice_echeancier_e2e.rs`, `reconciliation_e2e.rs`, `supplier_settlement_cancel_e2e.rs`,
`project_reports_e2e.rs`, `supplier_invoice_draft_e2e.rs`), `kesh-db` (tests, quatre : `journal_entries_modification.rs`,
`migrations_upgrade_path.rs`, `opening_complement_repository.rs`, `supplier_invoices_draft.rs`), `kesh-report` (tests,
un : `open_items_invariant.rs`), `docs` (`testing.md`) : **quatre modules**, quatorze fichiers de tests, aucun fichier
de production. La règle des cinq modules ne découpe pas ; la longueur (vingt-deux tests, dix-neuf mutations, quatre
campagnes de suite entière) fait proposer un découpage à l'orchestrateur, au rapport de la remédiation P1, sans l'écrire
ici.

### Gates (voie 2 ; `e35-dev-commun.md` § *Quels gates, quand*)

- **`kesh-db` est touché** (tests de `migrations_upgrade_path` compris) : gate complet en cours de boucle
  (`scripts/test-fast.sh --ci`, base `kesh_e367` remise à neuf), non ciblé. ⚠️ Ne pas toucher aux sites positionnels de
  `migrations_upgrade_path.rs` (`apply_migrations_up_to`, `migrations.len()`, P6) : seules les dates changent.
- **Passage en `review`** : `test-fast --ci` (heure réelle, puis sous les trois instants d'AC1 et AC2, une campagne par
  prise de verrou) ; `scripts/gate-ci.sh kesh_ci_e367` en voie 1. Frontend non touché : pas de frontend complet, sauf si
  AC5 fait toucher une spec.
- **Clôture** : `gate-ci` et E2E complet à l'heure réelle au dernier commit de code, **puis** AC4 et AC5 sous horloge
  décalée (gates de clôture, C-36-7-6).
- Une commande lourde à la fois (E7). Verrou voie 2 : `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock
  scripts/mem-guard.sh …`, `TMPDIR=/tmp/kesh-voie2`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e367`.
- faketime et cargo : construire **avant**, à l'heure réelle (`cargo nextest run --no-run`), puis lancer sous faketime — une
  horloge décalée en avant fait paraître les sources anciennes, sans reconstruction, mais un `build` sous faketime laisserait
  des dates de fichiers dans le futur.

### Fichiers communs avec les autres stories de la phase 2

`crates/kesh-api/tests/admin_full_import_e2e.rs` et `reconciliation_e2e.rs` : possibles avec les lots 36-11 (0.19.0) ;
`invoice_echeancier_e2e.rs`, `supplier_settlement_cancel_e2e.rs` : à surveiller avec 36-9 (#1044, solde du reste
d'un règlement), qui part après la 36-7 ; `docs/testing.md` : aucun autre
attendu en 0.18.0. 36-10 (#1147, #1053, dates UTC) touche la date du jour côté production : si elle part en parallèle, ses
tests suivent D1 (année du jour, pas 2026). Aucun fichier de manuel : `git grep -nE 'KESH_TEST_INSTANT|KF-060|#1125|faketime'
-- docs/manual` ne rend rien.

### Ce qui doit rester vrai

- Les tests corrigés exercent toujours la date du jour (aucune date fixe substituée à `aujourd_hui_test()`, D1).
- Les montages restent verts **à l'heure réelle** et sous tout instant du 2026-01-01 au 2029-12-31 (exercice de la base
  de dev 2020-2030, pièces dans N-1, garde `y <= 2029` du modèle de D1).
- Le binaire servi ne lit toujours pas `KESH_TEST_INSTANT` (garde inchangée, AC9).
- Aucune donnée réelle dans les fixtures ; rien sous `~/kesh/`.

### Références

`_bmad-output/planning-artifacts/epic-36-premiere-facture-et-suite.md` (rang 7, décisions D1-D7 du 2026-11-15) ;
`e36-choix-autonomes-e360.md` (C-e360-13) ; fiches `35-5a0-horloge-plomberie.md`, `35-5a-date-du-jour-horloge.md`,
`35-5c-date-du-jour-tests.md` ; issue #1125 et ses trois commentaires ; `docs/testing.md:119`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-11-15 — Création de la fiche** (Opus 5.5, `bmad-create-story`, code lu à `0bf3c3b4b`). Inventaire par
  exécution : nextest du workspace sous `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, voie 2 — 5452 exécutés, 5439 passés,
  13 échecs, 13 ignorés (`kesh-gate-logs/e36-7/inventaire-A2027.log`, `rc=100`) ; Vitest complet sous `Date` décalée en
  2027 — 238 fichiers, 3709 tests passés (`inventaire-vitest-A2027.log`), sonde positive rouge sans la variable, verte avec
  (`inventaire-vitest-sonde.log`). Inventaire par lecture des angles morts (modules de tests de `src`, specs, scripts,
  production). Choix C-36-7-1 à C-36-7-5.
- **2026-11-15 — Remédiation de la validation P1** (Sonnet 5.5 : 1 HIGH, 4 MEDIUM, 3 LOW ;
  `kesh-gate-logs/e36-7/validation-p1.md`). Prémisse de chaque finding lue au code ou au journal avant correction ;
  aucun réfuté.
  - **V1-1 (HIGH), corrigé** : suite entière rejouée à la tête (`d9a61b63f`), voie 2, sous `2026-03-10T23:30:00Z`
    (`inventaire-B2026.log` : 5452 exécutés, 5443 passés, 9 échecs, 13 ignorés) et `2026-12-31T23:30:00Z`
    (`inventaire-N2027.log` : 9 échecs, tous parmi les treize de 2027). Aux quatre sites nommés par V1-1 s'ajoutent deux
    rouges que ni la fiche ni V1-1 ne portaient : `project_reports_e2e::project_expenses_cumulative_mode` (`:131`) et
    `supplier_invoice_draft_e2e::the_open_items_document_carries_both_numbers` (`:1853`). D2 passe de trois à neuf tests,
    le périmètre de seize à vingt-deux tests et de huit à quatorze fichiers (quatre modules) ; AC2 gardé, borné.
  - **V1-2, corrigé** : D1 date les pièces passées dans N-1, modèle `supplier_invoices_draft.rs:2059` `let date = jour(x.y
    - 2, 6, 15);` (exercices de `scinder`, `:1522-1547`).
  - **V1-3, V1-4, corrigés** : tableau M1 à M19, une mutation par site (ou groupe de sites d'un même montage, dit), test
    qui la tue et instant prescrits ligne par ligne.
  - **V1-5, corrigé** : AC4 et AC5 gates de clôture (C-36-7-6) ; preuve exigée pour l'angle mort MariaDB (lignes SQL relues :
    `refresh_tokens.rs:61` `expires_at > NOW(3)`, `invoice_reminders.rs:176`, `invoices.rs:2620`, `journal_entries.rs:1658`,
    `payment_batches.rs:640`) ; repli si Chromium ne suit pas `LD_PRELOAD` (non constatable, sans contournement inventé) ;
    `TZ=Europe/Zurich` dans la commande.
  - **V1-6, corrigé** : sites Vitest nommés et justifiés (`bank-accounts-page.test.ts:713-714`, `:718`, `:413-416` ;
    `SupplierInvoiceForm.test.ts:870`, `:887`) ; specs Playwright à lire d'abord sous AC5 nommées.
  - **V1-7, corrigé** : AC3 (b) restreint aux quatorze binaires corrigés.
  - **V1-8, corrigé** : D5 dit ce qui est établi (règle `letterings.rs:89-94`, deux appels `:185` et `:220`, instants
    où le test rougit) et ce qui ne l'est pas ; `mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris` exclu (absent des
    trois relevés de la fiche).
  - Propagation grepée dans la fiche (`seize`, `trois de D2`, `huit mutations`, `C-36-7-5`, `1er janvier de l'année`,
    `2026 à 2030`, nombres de rouges et d'AC) : aucun résidu. Registre : C-36-7-3 amendé, C-36-7-6 ajouté.
- **2026-11-15 — Découpage** (approuvé par l'orchestrateur, règle de Guy du même jour ; C-36-7-7). La fiche passe à
  `Status: split` ; son texte ci-dessus est laissé tel qu'il était à la remédiation P1 (historique, non réécrit). Elle se
  remplace par :
  - `36-7a-tests-desancres-2027.md` — les treize rouges de 2027, sept fichiers, quatre modules, mutations M1 à M10 (M1,
    M4 à M12 d'ici) ; AC1, AC3 à AC9 d'ici, l'heure réelle et la nuit de l'AC2 ; AC4 et AC5 gates de clôture ; urgente,
    d'ici le 2026-12-28 ; porte les décisions communes (D1 en N-1, D3 faketime, D4, D5).
  - `36-7b-tests-bornes-anterieures.md` — les neuf rouges sous instant antérieur, huit fichiers, trois modules, mutations
    M1 à M9 (M2, M3, M13 à M19 d'ici) ; l'instant de mars de l'AC2 ; sans échéance, après la fusion de la 36-7a.
  La PR de release ne porte `closes #1125` qu'avec la 36-7b close et AC4, AC5 de la 36-7a joués.
- **2026-11-15 — Note** : la validation P2 de la 36-7a (`kesh-gate-logs/e36-7/validation-36-7a-p2.md`) a montré que la
  règle D1 ci-dessus (« dans N-1, jamais dans N », « juste pour tout jour d'une année ») est fausse sous le verrou glissant
  d'`open_items_invariant` et invérifiable sous l'instant `2027-01-04`, que la cause de D5 se lit au code (lettrage `:220`,
  verrou jour − 300), que ce test rougit à l'heure réelle dès le 2026-12-26, et que libfaketime est installée. La règle
  récrite et ces faits sont dans `36-7a-tests-desancres-2027.md` (D1, D3, D5, D6), dont le périmètre compte désormais cinq
  modules et seize mutations (M1a à M10) ; le texte de cette fiche n'est pas réécrit.
