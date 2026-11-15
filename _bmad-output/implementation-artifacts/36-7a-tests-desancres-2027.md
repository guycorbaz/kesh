# Story 36-7a : les tests ne figent plus 2026 — verts au 1er janvier 2027 (#1125, KF-060)

Status: ready-for-dev

*Fiche issue du découpage de la fiche `36-7-tests-desancres-2026.md` (`Status: split`), approuvé par l'orchestrateur le
2026-11-15 sur la règle de Guy du même jour (« éviter les stories trop longues ou trop complexes ») ; choix C-36-7-7.
Rédigée par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-7`, branche
`story/36-7-tests-desancres-2026`, partie de `epic/36-0.18.0` (`0bf3c3b4b`). Release **0.18.0**, phase 2, rang 7,
**première de la phase** (D1 du 2026-11-15). **Aucune migration**, aucune ligne de code de production, aucune clé i18n,
aucun changement d'API. Code lu à `0bf3c3b4b`. **Échéance ferme : d'ici le 2026-12-28** (décision de Guy, Q1 ;
C-e360-13). Voie de base **2** (`kesh-mariadb-dev2:3307`), `gate-ci.sh` en voie 1. Clé courte des bases proposée : `e367a`
(à confirmer par l'orchestrateur). Choix : `e36-choix-autonomes-36-7.md` (C-36-7-1 à C-36-7-7). Relevés de la fiche :
`/home/gcorbaz/devel/kesh-gate-logs/e36-7/` ; journaux de la story : `kesh-gate-logs/e36-7a/`. Sœur : `36-7b-tests-bornes-anterieures.md`
(les neuf tests qui rougissent sous un instant antérieur à leur date de 2026), développée après la fusion de celle-ci.*

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

**Conséquence** : le critère du fichier d'epic (rang 7 : « `scripts/test-fast.sh --ci` et l'E2E complet verts sous
`KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, sonde positive jointe ») est **exécutable à la lettre pour nextest**, mais
**sans effet pour l'E2E** : ni le backend servi, ni le runner, ni le navigateur ne lisent la variable. D3 dit comment
l'E2E se joue en 2027.

### Les treize rouges de 2027 — inventaire par exécution, sans correctif

Détecteur par exécution (`e35-dev-commun.md` § *Inventaires*, 1) : toute la suite, à la tête, voie 2, base remise à neuf.
Script hors dépôt : `/home/gcorbaz/devel/kesh-gate-logs/e36-7/inventaire-36-7.sh` ; journal `inventaire-A2027.log`.

```sh
/home/gcorbaz/devel/kesh-gate-logs/e36-7/inventaire-36-7.sh A2027 2027-01-04T10:00:00Z
# → cargo nextest run --workspace --profile ci --no-fail-fast, KESH_TEST_INSTANT=2027-01-04T10:00:00Z, base 3307/kesh_e367
```

Sortie copiée (`TRY 1` seuls, motif du `panicked` ajouté) :

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
5357 tests, 13 failed), relevée à un commit antérieur. Leur présence est la **sonde positive** du relevé : l'instant est
bien lu (à l'heure réelle, les mêmes tests sont verts — gate de la 0.17.8, `kesh-gate-logs/rel0178/`).

Sous la nuit du 1er janvier 2027 (`inventaire-N2027.log`, `KESH_TEST_INSTANT=2026-12-31T23:30:00Z`, jour suisse
2027-01-01, jour UTC 2026-12-31) : `5452 tests run: 5443 passed (1 slow), 9 failed, 13 skipped` — **neuf des treize**,
tous sauf les quatre de `dates_bornes_e2e`, dont le témoin `"2027-01-01"` n'y précède pas la date du jour (il lui est
égal). Aucun rouge hors des treize.

**Les sites fautifs, ligne lue** (le montage fige 2026, l'opération se date au jour du test) :

| Test(s) | Site du montage | Ce qui se date au jour |
|---|---|---|
| `admin_full_import_e2e` ×2 | `:4899-4901` `v0174a_decor` : `VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')` ; pièces `"invoiceDate": "2026-06-15"` (`:4990`), `"paymentDate": "2026-06-20"` (`:5039`) | l'annulation `/api/v1/supplier-invoices/3/cancel` (contre-passation au jour) |
| `bank_statements_manual_e2e` | `:218-220` `VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')` | l'écriture passée à `:1149` |
| `dates_bornes_e2e` ×4 | témoin `"2027-01-01"` (`:1010`, `:1050`, `:1510`, `:1556`) pour `dueDate`, face à `"date": aujourd_hui()` (`:1017`) | la facture du jour : au 2027-01-04, l'échéance témoin la précède |
| `journal_entries_modification` ×2 | `:268-275` exercice du seed « ramené à l'année 2026 » (`UPDATE fiscal_years SET name = 'Exercice 2026', start_date = '2026-01-01', end_date = '2026-12-31'`) ; écritures `d(2026, 3, 1)` (`:373`, `:426`, `:487`, …) | la contre-passation `crate::jour::aujourd_hui_test()` (`:381`) |
| `migrations_upgrade_path` ×2 | base 0.16 / rc1 peuplée, `day = … from_ymd_opt(2026, 6, 20)` (`:1498`) | `cancel_in_tx(…, crate::jour::aujourd_hui_test())` (`:1515-1520`) |
| `opening_complement_repository` | `year_span(2026)` (`:47-49`, appelé `:246`, `:277`, `:298`, `:311`, `:331`, `:346`, `:355`, `:383`, …) | `journal_entries::reverse(…, crate::jour::aujourd_hui_test())` (`:523-528`) |
| `open_items_invariant` | exercices 2024 (clos) et 2025-2030 (`:106-116`) ; groupe « à cheval » `2024-06-30` / `2026-03-01` (`:203`) ; écritures à `jours_avant(n)` | le lettrage `:170-181` refusé `LetteringAllLinesInClosedPeriods` — cause à établir (D5) |

**Hors de cette story** : les neuf tests qui rougissent sous un instant **antérieur** à leur date de 2026 (relevé
`inventaire-B2026.log`) sont la story **36-7b**. **Rouge de charge exclu** :
`mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris` figure aux journaux de la 35-5a (`e355a/nuit-C2.log`,
`nuit-C2b.log`, `nuit-C3.log`) et à aucun des trois relevés de la fiche d'origine : test à marges de temps, sans date de
2026 ; s'il rougit, il se nomme comme rouge de charge et se relance.

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

Ce relevé par lecture ne prouve pas l'absence : AC4 rejoue la suite sous une **horloge de processus** décalée (D3).
**Sonde** de la lecture : la commande rend bien `middleware/auth.rs:302`, site connu de la garde.

### Vitest — par exécution, vert en 2027

Vitest ne lit pas `KESH_TEST_INSTANT`. Détecteur par exécution, hors dépôt : un préchargement Node qui décale `Date`
(`/home/gcorbaz/devel/kesh-gate-logs/e36-7/date-decalee-36-7.mjs`, `NODE_OPTIONS="--import …"`, variable
`KESH_TEST_INSTANT_JS`), le temps continuant de s'écouler ; `vi.useFakeTimers()` d'un test le remplace.

```sh
cd /home/gcorbaz/devel/kesh-36-7/frontend && TMPDIR=/tmp/kesh-voie2 KESH_TEST_INSTANT_JS=2027-01-04T10:00:00Z \
  NODE_OPTIONS="--import /home/gcorbaz/devel/kesh-gate-logs/e36-7/date-decalee-36-7.mjs" \
  flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock ../scripts/mem-guard.sh npx vitest run
```

Sortie copiée (`inventaire-vitest-A2027.log`) : `Test Files  238 passed (238)`, `Tests  3709 passed (3709)`, `rc=0`.

**Sonde positive** (`inventaire-vitest-sonde.log`) : un test hors dépôt (`kesh-gate-logs/e36-7/sonde-vitest/annee.test.js`,
`expect(new Date().getUTCFullYear()).toBe(2027)`) lancé par le même Vitest : sans la variable, `AssertionError: expected
2026 to be 2027` (`rc=1`) ; avec, `Tests 1 passed (1)` (`rc=0`), y compris sous `--environment jsdom`
(`frontend/vite.config.ts:58`). **Aucun site Vitest n'est au périmètre.** Les tests qui lisent l'heure et la comparent à une
date de 2026 l'ont arrêtée eux-mêmes : `src/routes/(app)/bank-accounts/bank-accounts-page.test.ts:713-714`
(`vi.useFakeTimers({ toFake: ['Date'] })`, `vi.setSystemTime(new Date('2026-10-13T22:30:00Z'))`, puis `:718`
`toBe('2026-10-13')`) et `src/lib/features/supplier-invoices/SupplierInvoiceForm.test.ts:870` (`vi.setSystemTime(new
Date('2026-11-04T23:30:00Z'))`, lu à `:887`) ; `bank-accounts-page.test.ts:413-416` (`aujourdhui()`) compare le champ au
jour qu'il lit lui-même, sans année fixe.

### Playwright — par lecture (l'exécution en 2027 est l'AC5)

`git grep -cE '\b2026\b' -- frontend/tests/e2e` rend 33 fichiers (specs, helpers, fixtures CAMT/CSV). Ce qui borne le
risque, ligne lue :

- l'exercice des presets `with-company` est **2020-2030** (`crates/kesh-db/src/test_fixtures.rs:118-119` `VALUES (?,
  'Exercice CI 2020-2030', '2020-01-01', '2030-12-31', 'Open')`, appelé par `seed_accounting_company` depuis
  `crates/kesh-api/src/routes/test_endpoints.rs:201`) : une pièce datée de 2026 reste dans un exercice ouvert en 2027 ;
- les specs qui créent l'exercice de l'année le prennent du jour (`api-fixtures.ts:401` `const year =
  todayInZurich().slice(0, 4);`) ;
- les fenêtres de rapprochement se comptent depuis la date du relevé (`reconciliation-amount-due.spec.ts:81`) ;
  `fiscal-year-closing-journey.spec.ts:30` « N+1 est 2026 quelle que soit l'année courante » ;
- `frontend/tests/e2e/supplier-internal-number.spec.ts:237` attend `KF-2026-0001` — **il ne rougira pas en 2027** :
  l'aperçu est figé en 2026 **dans le code de production**, `frontend/src/routes/(app)/settings/invoicing/+page.svelte:159`
  `previewInvoiceNumber(format, 2026, '2026', 1)`, et de même `:173`, `:180` (hors périmètre, issue demandée).

Aucune spec n'est identifiée comme fautive par lecture ; ce n'est pas une preuve, d'où AC5. **Premières à lire en cas de
rouge sous AC5** : `supplier-invoice-draft-cycle.spec.ts` (dates `2026-05-10` à `2026-07-31`, `:129` à `:522`, dont
`paymentDate: '2026-05-20'` `:478`), `reconciliation-supplier.spec.ts:110-111`, `product-revenue-account.spec.ts:190`,
`:262`, `fiscal-year-closing-journey.spec.ts:177` (`createFiscalYear(ctx, 2026)`).

### Scripts et production

- `scripts/recette-montee.sh:476` `JOUR=$(date +%F)` et preset `with-company` (`:473`) : les relevés de recette d'octobre
  2026 (`scripts/fixtures/recette/releve-A.xml:6`) restent dans l'exercice 2020-2030 : **juste**. `scripts/tests/*.test.sh` :
  dates de 2026 en données, comparaisons au jour lu (`prepare-release.test.sh:103`) : **juste**.
- Production Rust : `git grep -lE '\b2026\b' -- 'crates/*/src/*' 'crates/*/src/**'` (95 fichiers), lignes avant le premier
  `#[cfg(test)]`, hors fichiers de tests et commentaires : **un seul** site, `crates/kesh-api/src/qr_decode_synthetique.rs:112`,
  texte d'un générateur de test. **Aucun site de production Rust ne fige 2026.**
- Production frontend : outre l'aperçu ci-dessus, `contacts/+page.svelte:1067` `placeholder="CLI-2026-00042"` — exemple de
  saisie, **juste** (C-36-7-5).

## Décisions

Ces décisions valent aussi pour la 36-7b, qui y renvoie.

- **D1 — Le montage prend ses années du jour du test (N = année de `aujourd_hui_test()`) ; une date qui doit précéder le
  jour se place dans N-1, jamais dans N.** Le « 1er janvier de l'année du jour » **est** le jour sous
  `2026-12-31T23:30:00Z` (jour suisse 2027-01-01) et fait alors rougir les gardes « borne passée » et `veille >= date`
  (validation P1 de la fiche d'origine, V1-2). Règle, juste pour tout jour d'une année :
  - l'exercice du montage couvre **N-1 et N** (deux exercices civils, ou un seul qui couvre les deux), N+1 en plus là où le
    test le demande (`journal_entries_modification:268` « pour qu'un exercice 2027 puisse le suivre ») ;
  - pièces, paiements et bornes de verrou qui doivent être **passés** se datent dans **N-1** (par exemple `d(N-1, 6, 15)`,
    borne `d(N-1, 12, 31)`) ; l'opération du jour (contre-passation, annulation, verrou) garde `aujourd_hui_test()` ;
  - aucune date « fixe dans l'année du jour » ni décalage `aujourd_hui - n` qui puisse traverser le 1er janvier.
  **Modèle** au dépôt : `crates/kesh-db/tests/supplier_invoices_draft.rs:2059` `let date = jour(x.y - 2, 6, 15);`, avec
  `scinder` (`:1522-1547`) qui pose les exercices `y - 2`, `y - 1` et `y` depuis `let y = aujourdhui().year();` (`:1524`)
  et garde `assert!(y <= 2029, …)` (`:1525-1528`). Le témoin de `dates_bornes_e2e` devient une échéance **postérieure** à
  la date du jour (`aujourd_hui() + n` jours). Là où un montage « base 0.16 / rc1 peuplée » (`migrations_upgrade_path`) ou
  un exercice existant du seed ne peut pas être déplacé, l'étape 0 le dit, ligne lue. *Écartée* : passer à chaque opération
  une date fixe de 2026 (`cancel(…, 2026-06-20)`) — le test cesserait d'exercer la date du jour ; *écartée* aussi : un
  exercice 2020-2030 dans chaque montage — il masquerait le jour et repousse la panne à 2031.
- **D2 — Le périmètre est celui des treize rouges de 2027** (C-36-7-7). Les neuf tests qui ne rougissent que sous un
  instant antérieur à leur date de 2026 sont la 36-7b. *Écartée* : une seule story pour les vingt-deux (validation P1 de
  la fiche d'origine, axe 5 : trop longue, et la dépendance à libfaketime aurait retenu les deux).
- **D3 — L'E2E en 2027 se joue sous une horloge de processus (libfaketime), non sous `KESH_TEST_INSTANT`** (C-36-7-2). Le
  binaire servi ignore la variable (`docs/testing.md:119`, garde `date_du_jour_garde.rs`) et le runner lit l'heure réelle
  (`api-fixtures.ts:393`). Le backend et le runner se lancent sous
  `TZ=Europe/Zurich FAKETIME_DONT_FAKE_MONOTONIC=1 faketime -f '@2027-01-04 11:00:00' <commande>` — `faketime` lit la date
  dans le fuseau du processus : « 11 h » ne vaut 10:00Z qu'avec `TZ=Europe/Zurich` fixé dans la commande. Que Chromium
  hérite de `LD_PRELOAD` est une **hypothèse non éprouvée** (le navigateur de Playwright lance ses processus avec son bac à
  sable) : la sonde de l'AC5 la tranche. MariaDB, en conteneur, reste à l'heure réelle (angle mort écrit : `NOW(3)` de
  `refresh_tokens.rs:61`, `:221`, `UTC_TIMESTAMP(6)` d'`invoice_reminders.rs:176`, `invoices.rs:2620`,
  `CURRENT_TIMESTAMP(3)` de `journal_entries.rs:1658`, `payment_batches.rs:640`, `DEFAULT CURRENT_TIMESTAMP` des colonnes
  d'horodatage ; liste non close). **Un rouge ne se rattache à cet angle mort que prouvé** : la ligne de SQL qui lit
  l'heure de la base, la colonne, et un rejeu isolé du test qui montre que le rouge tient à l'écart base/processus (par
  exemple : vert sous `KESH_TEST_INSTANT` seul, rouge sous faketime). Sans cette preuve, il compte comme un rouge de la
  story. **Repli si le navigateur reste en 2026** (étage 3 de la sonde) : aucun contournement n'est éprouvé ; l'AC5 se
  déclare **non constatable** pour la partie navigateur — non « vert » —, avec les étages serveur et runner joints, et
  l'orchestrateur arbitre (piste à éprouver, non prescrite : `page.clock` posé par un fixture). **Prérequis** : la
  bibliothèque, absente de la station (`ldconfig -p | /usr/bin/grep -i faketime` vide ; `apt-cache policy faketime` :
  « Installé : (aucun) », candidat `0.9.10+2024-06-05+gba9ed5b2-0.6build1`) ; son installation est un geste de Guy,
  demandé par l'orchestrateur le 2026-11-15. *Écartées* : faire lire `KESH_TEST_INSTANT` au binaire en `KESH_TEST_MODE`
  (code de production, contraire à la garde) ; `page.clock` seul (le serveur resterait en 2026) ; déclarer l'E2E « vert en
  2027 » sur la seule lecture.
  **AC4 et AC5 sont des gates de clôture** (C-36-7-6) : le développement **peut passer en `review` sans libfaketime**
  (AC1-AC3, AC6-AC9) ; la story **ne passe pas `done`** tant qu'AC4 et AC5 n'ont pas tourné, journaux joints. Si la
  bibliothèque manque à l'étape 0, la **première ligne** du rapport de l'agent est la demande à Guy. La PR de release ne
  porte `closes #1125` qu'avec la 36-7b close **et** AC4, AC5 joués.
- **D4 — Aucun outillage versionné de plus** (C-36-7-4). Les recettes (nextest et E2E sous horloge décalée, Vitest sous
  préchargement) s'écrivent dans `docs/testing.md`, à côté du paragraphe de `KESH_TEST_INSTANT` ; le préchargement Vitest et
  les scripts restent hors dépôt. *Écartée* : un `scripts/test-2027.sh` ou un setup Vitest qui lit `KESH_TEST_INSTANT`,
  fonction d'outillage neuve, à proposer en issue si l'orchestrateur le juge utile.
- **D5 — La cause du rouge d'`open_items_invariant` se lit avant d'écrire** : le lettrage de la fermeture `lettrer`
  (`:170-181`) est refusé `LetteringAllLinesInClosedPeriods`. Établi, ligne lue : la règle est celle de
  `crates/kesh-db/src/repositories/letterings.rs:89-94` (« Une ligne est **« en période ouverte »** si son exercice est
  `Open`, si aucun exercice postérieur n'est `Closed`, et si sa date est strictement postérieure à
  `companies.books_locked_through` ») ; la fermeture est appelée deux fois, `:185` (`p1`, `p2` à `jours_avant(60)` et
  `jours_avant(50)`, exercice `fy` 2025-2030) et `:220` (groupe à cheval `2024-06-30` / `2026-03-01`, avant la clôture de
  2024 à `:222`) ; le rouge apparaît sous `2027-01-04` et sous `2026-12-31T23:30:00Z`, non sous `2026-03-10T23:30:00Z`.
  Non établi : **lequel** des deux appels rougit, et pourquoi (la panique porte `:181` dans les deux cas). L'étape 0 rejoue
  le test seul sous `RUST_BACKTRACE=1` et `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, cite la ligne qui produit le refus, et
  **seulement alors** choisit le correctif.

## Critères d'acceptation

1. **AC1 — nextest vert sous l'instant du critère.** `scripts/test-fast.sh --ci`, sous
   `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, voie 2, base remise à neuf : 0 échec (journal
   `kesh-gate-logs/e36-7a/gate-2027.log`, commit cité). Les treize tests du tableau sont verts.
2. **AC2 — et sous deux autres réglages, sans régression.** La même suite verte sous `2026-12-31T23:30:00Z` (nuit suisse du
   1er janvier 2027 : neuf rouges à la base, `inventaire-N2027.log`) et sans la variable (heure réelle). **Non-régression
   sous l'instant antérieur** : sous `2026-03-10T23:30:00Z`, les rouges sont exactement les neuf de la 36-7b
   (`inventaire-B2026.log`), aucun autre. Un rouge de charge se relance et se nomme, sans compter contre l'AC.
3. **AC3 — Sonde positive de la lecture.** (a) Les relevés `inventaire-A2027.log` (13 rouges) et `inventaire-N2027.log`
   (9) sont rejoués à l'étape 0 à la tête de la branche et joints ; (b) après le correctif, les **sept binaires corrigés**
   (`cargo nextest run -E 'binary(admin_full_import_e2e) | binary(bank_statements_manual_e2e) | binary(dates_bornes_e2e) |
   binary(journal_entries_modification) | binary(migrations_upgrade_path) | binary(opening_complement_repository) |
   binary(open_items_invariant)'`) rougissent sous `KESH_TEST_INSTANT=pas-un-instant` (panique d'`instant_de_test`, message
   qui nomme la variable) ; (c) les mutations M1 à M10 de § *Tests*, chacune tuée par le test nommé.
4. **AC4 — nextest sous horloge de processus décalée** (D3 ; **gate de clôture**, C-36-7-6) : `cargo nextest run
   --workspace --profile ci` sous `TZ=Europe/Zurich FAKETIME_DONT_FAKE_MONOTONIC=1 faketime -f '@2027-01-04 11:00:00'`,
   **sans** `KESH_TEST_INSTANT`, binaires construits avant à l'heure réelle (`--no-run`) : 0 échec, ou chaque rouge nommé
   et rattaché — à un site de la story, ou à l'angle mort MariaDB **avec la preuve exigée par D3** et une issue. Sonde : un
   test du tableau ramené à sa version de la base rougit sous la même commande.
5. **AC5 — E2E complet en 2027** (D3 ; **gate de clôture**, C-36-7-6) : la suite Playwright complète, backend et runner
   sous la même horloge décalée (`TZ=Europe/Zurich` compris), base `kesh_e2e_e367a` recréée : les échecs se nomment un par
   un contre `docs/testing.md` § *« Les échecs attendus »* (les sept de la KF-029, #97) ; aucun autre. **Sonde positive
   jointe** : avant la suite, un appel qui rend la date du serveur (`GET /api/v1/accounts/{id}/open-items` sans `asOf` :
   `asOf` vaut `2027-01-04`, `crates/kesh-api/src/routes/letterings.rs:388`), `todayInZurich()` du runner qui commence par
   `2027-`, et `new Date().getFullYear()` du navigateur à 2027 (`page.evaluate`) — par un script hors dépôt, journal joint.
   Si l'étage navigateur rend 2026, repli de D3 (non constatable pour le navigateur, arbitrage), jamais « vert ».
6. **AC6 — Vitest reste vert en 2027** : le relevé `inventaire-vitest-A2027.log` se rejoue au dernier commit de code, avec
   sa sonde.
7. **AC7 — Aucune ligne de production ne bouge** : `git diff --stat origin/epic/36-0.18.0...HEAD -- 'crates/*/src/*'
   'crates/*/src/**' frontend/src` vide.
8. **AC8 — La documentation dit vrai** : `docs/testing.md:119` dit ce que `KESH_TEST_INSTANT` ne couvre pas (binaire servi,
   runner, navigateur, deux modules de tests de `src`) et donne les recettes de D3 et D4 ; l'exception (a) nomme les neuf
   tests de la 36-7b (la 36-7b la retire).
9. **AC9 — Garde intacte** : `date_du_jour_garde.rs` verte sans retouche de ses listes closes.

## Tests à écrire, et la preuve du rouge

La story n'ajoute pas de test : elle **corrige les montages** de treize tests, dans sept fichiers. Le rouge de chacun est
constaté, sans correctif, par `inventaire-A2027.log` (et neuf d'entre eux par `inventaire-N2027.log`) — à rejouer à
l'étape 0 à la tête de la branche, journal `kesh-gate-logs/e36-7a/rouges.log`.

**Mutations** (script versionné dans `kesh-gate-logs/e36-7a/mut/`, code commité avant, restauration
`git restore --worktree -- <fichier>` puis `touch`) : **une par site corrigé** (ou par groupe de sites qui partagent un
même montage, dit comme tel), qui remet la date ou l'exercice du site à sa forme de 2026 ; chacune **tuée par le test
nommé sous `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`** (journal `mut/M<n>.log`).

| M | Site corrigé | Test qui la tue |
|---|---|---|
| M1 | `admin_full_import_e2e` `v0174a_decor` (`:4899-4901` et ses pièces) — un groupe | `full_import_accepts_an_rc1_backup_with_supplier_invoices` (et `…_0_16_…`) |
| M2 | `bank_statements_manual_e2e:218-220` | `manual_statement_lines_reconcile_like_imported_ones` |
| M3 à M6 | `dates_bornes_e2e` témoins `:1010`, `:1050`, `:1510`, `:1556` — une mutation chacun | `c_facture_echeance`, `c_facture_maj_echeance`, `c_fournisseur_echeance`, `c_fournisseur_maj_echeance` (appariés à l'étape 0, ligne lue) |
| M7 | `journal_entries_modification` `societe()` (`:268-275`) — un montage pour deux tests | `update_waits_for_a_concurrent_reversal_then_refuses` (et `delete_…`) |
| M8 | `migrations_upgrade_path` (`:1498` et l'exercice de la base peuplée) — un groupe | `draft_migration_preserves_a_populated_016_base` (et `…_rc1_…`) |
| M9 | `opening_complement_repository` `year_span(…)` de `un_compte_contre_passe_reste_mouvemente` | ce test ; les autres appels de `year_span(2026)` (`:246` à `:383`) se mutent chacun s'ils sont corrigés, sinon l'étape 0 dit pourquoi ils restent justes |
| M10 | `open_items_invariant` (site établi par D5) | `open_items_balance_matches_the_ledger_and_the_trial_balance` |

Une mutation qui ne rougit pas signale un montage qui ne dépend plus du jour : à dire (juste, ou test muet).

## Tâches

1. **Étape 0** : relire `e36-choix-autonomes-e360.md` (C-e360-13) et `e36-choix-autonomes-36-7.md` ; constater la présence
   de libfaketime (D3) ; rejouer les relevés A2027 et N2027 à la tête (`rouges.log`) ; lire la cause
   d'`open_items_invariant` (D5) et la citer.
2. Corriger les montages des sept fichiers du tableau (D1).
3. Mettre à jour `docs/testing.md:119` (AC8) ; recettes de D3/D4.
4. Mutations M1 à M10 (AC3 c).
5. Gates (§ *Dev Notes*), AC1, AC2, AC3, AC6 → `Status: review`, clé `review`, Dev Agent Record, prompt de la revue P1.
   **Sans libfaketime, la story s'arrête là** (C-36-7-6).
6. À la clôture : `gate-ci`, E2E complet à l'heure réelle, **puis AC4 et AC5** ; ensuite seulement `done`.

## Note de version

*Modifié* — Les tests de Kesh ne figent plus l'année 2026 : la suite reste verte à partir du 1er janvier 2027. Aucun
changement pour l'utilisateur. (refs #1125)

## Dev Notes

### Hors périmètre

- Les neuf tests à date de 2026 qui rougissent sous un instant antérieur : **36-7b**.
- **L'aperçu du format de numérotation montre toujours l'année 2026** — `frontend/src/routes/(app)/settings/invoicing/+page.svelte:159`,
  `:173`, `:180` (`previewInvoiceNumber(format, 2026, '2026', 1)`) : en 2027, *Paramètres → Facturation* affiche
  `FF-2026-001` comme exemple du prochain numéro. Code de production : **issue à ouvrir** (P4, `bug`). Si elle est
  corrigée, `supplier-internal-number.spec.ts:237` (`KF-2026-0001`) devra suivre.
- **L'exercice de test 2020-2030** (`crates/kesh-db/src/test_fixtures.rs:118-119`) expirera le 2030-12-31 : **issue à
  ouvrir** (P4, `technical-debt`).
- Le placeholder `CLI-2026-00042` (`contacts/+page.svelte:1067`) : laissé (C-36-7-5).

### Modules touchés (règle de splitting)

`kesh-api` (tests, trois fichiers : `admin_full_import_e2e.rs`, `bank_statements_manual_e2e.rs`, `dates_bornes_e2e.rs`),
`kesh-db` (tests, trois : `journal_entries_modification.rs`, `migrations_upgrade_path.rs`,
`opening_complement_repository.rs`), `kesh-report` (tests, un : `open_items_invariant.rs`), `docs` (`testing.md`) :
**quatre modules**, sept fichiers de tests, aucun fichier de production.

### Gates (voie 2 ; `e35-dev-commun.md` § *Quels gates, quand*)

- **`kesh-db` est touché** : gate complet en cours de boucle (`scripts/test-fast.sh --ci`, base `kesh_e367a` remise à
  neuf), non ciblé. ⚠️ Ne pas toucher aux sites positionnels de `migrations_upgrade_path.rs` (`apply_migrations_up_to`,
  `migrations.len()`, P6) : seules les dates changent.
- **Passage en `review`** : `test-fast --ci` à l'heure réelle, puis sous `2027-01-04T10:00:00Z`, `2026-12-31T23:30:00Z` et
  `2026-03-10T23:30:00Z` (une campagne par prise de verrou) ; `scripts/gate-ci.sh kesh_ci_e367a` en voie 1. Frontend non
  touché : pas de frontend complet, sauf si AC5 fait toucher une spec.
- **Clôture** : `gate-ci` et E2E complet à l'heure réelle au dernier commit de code, **puis** AC4 et AC5.
- Une commande lourde à la fois (E7). Verrou voie 2 : `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock
  scripts/mem-guard.sh …`, `TMPDIR=/tmp/kesh-voie2`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e367a`.
- faketime et cargo : construire **avant**, à l'heure réelle (`cargo nextest run --no-run`), puis lancer sous faketime.

### Fichiers communs

- `crates/kesh-api/tests/admin_full_import_e2e.rs` et `docs/testing.md` : **communs avec la 36-7b**, qui part de la 36-7a
  fusionnée (`v0174a_decor` ici ; `:2007`, `:2281` là-bas — paragraphes distincts).
- `admin_full_import_e2e.rs` : possible avec les lots 36-11 (0.19.0). 36-10 (#1147, #1053) : ses tests suivent D1.
- Aucun manuel : `git grep -nE 'KESH_TEST_INSTANT|KF-060|#1125|faketime' -- docs/manual` ne rend rien.

### Ce qui doit rester vrai

- Les tests corrigés exercent toujours la date du jour (D1).
- Les montages restent verts à l'heure réelle et sous tout instant du 2026-01-01 au 2029-12-31.
- Le binaire servi ne lit toujours pas `KESH_TEST_INSTANT` (AC9).
- Aucune donnée réelle dans les fixtures ; rien sous `~/kesh/`.

### Références

Fiche d'origine `36-7-tests-desancres-2026.md` (`split`) et sa validation P1 (`kesh-gate-logs/e36-7/validation-p1.md`) ;
`_bmad-output/planning-artifacts/epic-36-premiere-facture-et-suite.md` (rang 7) ; `e36-choix-autonomes-e360.md`
(C-e360-13) ; fiches `35-5a0-horloge-plomberie.md`, `35-5a-date-du-jour-horloge.md`, `35-5c-date-du-jour-tests.md` ;
issue #1125 ; `docs/testing.md:119`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-11-15 — Création par découpage** (Opus 5.5) de la fiche `36-7-tests-desancres-2026.md`, après sa validation P1
  (1 HIGH, 4 MEDIUM, 3 LOW, `kesh-gate-logs/e36-7/validation-p1.md`) et sa remédiation (commit `afe0a4f2e`), découpage
  approuvé par l'orchestrateur le même jour (C-36-7-7). Reprend les treize rouges de 2027, les AC1, AC3 à AC9 d'origine,
  l'heure réelle et la nuit de l'AC2 (l'instant de mars ne sert ici qu'à la non-régression), les mutations M1, M4 à M12
  d'origine renumérotées M1 à M10, et les décisions communes (D1 en N-1, D3 faketime, D4, D5), auxquelles la 36-7b renvoie.
  Aucune passe de validation propre encore.
