# Story 36-7 : les tests ne figent plus 2026 — verts au 1er janvier 2027 (#1125, KF-060)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-7`, branche
`story/36-7-tests-desancres-2026`, partie de `epic/36-0.18.0` (`0bf3c3b4b`), par le workflow `bmad-create-story`. Release
**0.18.0**, phase 2, rang 7 et **première de la phase** (D1 du 2026-11-15). **Aucune migration**, aucune ligne de code de
production, aucune clé i18n, aucun changement d'API. Code lu à la tête du worktree, **`0bf3c3b4b`**. Échéance ferme : **à
terminer d'ici le 2026-12-28** (décision de Guy, Q1 ; C-e360-13). Voie de base **2** (`kesh-mariadb-dev2:3307`), `gate-ci.sh`
en voie 1. Clé courte des bases : `e367`. Choix propres à la story : `e36-choix-autonomes-36-7.md` (C-36-7-1 à C-36-7-5).
Journaux de la fiche : `/home/gcorbaz/devel/kesh-gate-logs/e36-7/`.*

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

### Les sites du même genre qui ne rougissent pas en 2027 (commentaires de #1125)

Relevés aux 35-5b et 35-5c, portés par #1125 : ils figent une borne de 2026 et rougissent sous un instant **antérieur** à
elle, jamais à l'heure réelle, qui ne recule pas.

- `crates/kesh-api/tests/admin_full_import_e2e.rs:2007` `let borne = chrono::NaiveDate::from_ymd_opt(2026, 3, 31)`
  (`full_import_letters_a_reopened_historical_piece`, `:2002`) et `:2281` `from_ymd_opt(2026, 6, 30)`
  (`validating_a_backdated_invoice_is_refused_by_the_period_lock`, `:2275`).
- `crates/kesh-api/tests/services_rollout.rs:207` `entry_date: NaiveDate::from_ymd_opt(2026, 6, 1)` (`poser_livres`,
  `:199`), lu à l'`asOf` par défaut par `ac11_a_les_services_lisent_en_lecture_seule` (`:1251`).

Ils entrent au périmètre (D2) : ce sont les seuls que #1125 nomme encore, et le premier fichier est déjà touché.

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
test. **Aucun site Vitest n'est au périmètre.**

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

Aucune spec n'est identifiée comme fautive par lecture ; ce n'est pas une preuve, d'où AC5.

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

- **D1 — Le montage prend son année du jour du test ; une date qui doit précéder le jour se construit pour rester dans
  l'exercice.** Chaque site du tableau ci-dessus remplace l'année 2026 par celle de `aujourd_hui_test()` (`common::jour`,
  `support/jour.rs`) — exercice, `year_span`, dates de pièces —, et une pièce qui doit précéder l'opération du jour se date
  soit au 1er janvier de l'année du jour, soit par décalage depuis le jour **à condition que l'exercice du montage la
  couvre** : sous l'instant du critère (le 4 janvier), `aujourd_hui - 30` tombe l'année d'avant. Là où un test a besoin de
  deux années (`journal_entries_modification:268` « pour qu'un exercice 2027 puisse le suivre »), ce sont N et N+1 du jour.
  Le témoin de `dates_bornes_e2e` devient une échéance **postérieure** à la date du jour (`aujourd_hui() + n` jours).
  *Écartée* : passer à chaque opération une date fixe de 2026 (`cancel(…, 2026-06-20)`) — le test resterait juste, mais
  cesserait d'exercer la date du jour que la 35-5c lui a donnée ; *écartée* aussi : un exercice 2020-2030 dans chaque
  montage — il masquerait le jour et repousse la panne à 2031.
- **D2 — Les trois sites à borne de 2026 qui rougissent sous un instant antérieur entrent au périmètre** (C-36-7-3) :
  `admin_full_import_e2e:2007`, `:2281`, `services_rollout:207`. Leur borne se tire du jour (une borne passée : le jour
  moins n, dans un exercice qui la couvre), ou l'`asOf` du montage se pose explicitement. *Écartée* : les laisser à une
  issue — #1125 resterait ouverte pour trois tests, et la campagne d'instants de test garderait trois rouges attendus.
- **D3 — L'E2E en 2027 se joue sous une horloge de processus (libfaketime), non sous `KESH_TEST_INSTANT`** (C-36-7-2). Le
  binaire servi ignore la variable (`docs/testing.md:119`, garde `date_du_jour_garde.rs`) et le runner lit l'heure réelle
  (`api-fixtures.ts:393`). Le backend et le runner (donc Chromium, qui hérite de `LD_PRELOAD`) se lancent sous
  `faketime -f '@2027-01-04 11:00:00'` (11 h à Zurich = 10:00Z), `FAKETIME_DONT_FAKE_MONOTONIC=1` ; MariaDB, en conteneur,
  reste à l'heure réelle (angle mort écrit : `NOW()`/`CURRENT_TIMESTAMP` de la base — horodatages, `refresh_tokens` —, un rouge
  qui en viendrait se nomme comme tel). **Prérequis** : la bibliothèque, absente de la station
  (`ldconfig -p | /usr/bin/grep -i faketime` vide ; `apt-cache policy faketime` : « Installé : (aucun) », candidat
  `0.9.10+2024-06-05+gba9ed5b2-0.6build1`) — son installation est un geste de Guy (paquet système), demandé au rapport. La
  même horloge sert l'AC4 (nextest entier, angles morts compris). *Écartées* : faire lire `KESH_TEST_INSTANT` au binaire en
  `KESH_TEST_MODE` (code de production, contraire à la garde et au périmètre) ; `page.clock` de Playwright (le navigateur
  seul, le serveur resterait en 2026) ; déclarer l'E2E « vert en 2027 » sur la seule lecture.
  **Si la bibliothèque n'est pas installée à l'étape 0**, l'agent s'arrête sur ce point et le dit à l'orchestrateur ; AC4 et
  AC5 restent ouverts, le reste se poursuit.
- **D4 — Aucun outillage versionné de plus** (C-36-7-4). Les deux recettes (nextest et E2E sous horloge décalée, Vitest
  sous préchargement) s'écrivent dans `docs/testing.md`, à côté du paragraphe de `KESH_TEST_INSTANT` ; le préchargement Vitest
  et les scripts restent hors dépôt (`kesh-gate-logs/e36-7/`). *Écartée* : un `scripts/test-2027.sh` ou un setup Vitest qui
  lit `KESH_TEST_INSTANT` — utile pour la campagne de chaque fin d'année, mais c'est une fonction d'outillage neuve, qui
  élargirait une story que Guy veut courte ; à proposer en issue si l'orchestrateur le juge utile.
- **D5 — La cause du rouge d'`open_items_invariant` se lit avant d'écrire** : le lettrage `:170-181` est refusé
  `LetteringAllLinesInClosedPeriods` ; la ligne qui le produit (verrou de période, exercice clos, date du jour) se cite au
  Change Log à l'étape 0, et le correctif suit D1.

## Critères d'acceptation

1. **AC1 — nextest vert sous l'instant du critère.** `scripts/test-fast.sh --ci`, sous
   `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, voie 2, base remise à neuf : 0 échec (journal
   `kesh-gate-logs/e36-7/gate-2027.log`, commit cité). Les treize tests du tableau sont verts.
2. **AC2 — et sous trois autres instants, sans régression.** La même suite verte sous `2026-12-31T23:30:00Z` (nuit suisse
   du 1er janvier 2027, jour UTC encore 2026), `2026-03-10T23:30:00Z` (antérieur aux bornes de D2 : les trois tests de D2 y
   sont verts, ils y rougissent à la base) et sans la variable (heure réelle).
3. **AC3 — Sonde positive de la lecture.** (a) Le relevé de la base (`inventaire-A2027.log`, 13 rouges) est rejoué à
   l'étape 0 et joint ; (b) après le correctif, la même suite sous `KESH_TEST_INSTANT=pas-un-instant` rougit (panique de
   `instant_de_test`, message qui nomme la variable), preuve que la variable est lue par les montages corrigés ; (c) une
   mutation qui rend sa date fixe de 2026 à un site corrigé rougit sous l'instant du critère (§ *Tests*).
4. **AC4 — nextest sous horloge de processus décalée** (D3) : `cargo nextest run --workspace --profile ci` sous
   `faketime -f '@2027-01-04 11:00:00'`, **sans** `KESH_TEST_INSTANT`, binaires construits avant à l'heure réelle
   (`--no-run`) : 0 échec, ou chaque rouge nommé et rattaché (à un site de la story, ou à l'angle mort MariaDB, avec une
   issue). Sonde : un test du tableau ramené à sa version de la base rougit sous la même commande.
5. **AC5 — E2E complet en 2027** (D3) : la suite Playwright complète, backend et runner sous la même horloge décalée, base
   `kesh_e2e_e367` recréée : les échecs se nomment un par un contre `docs/testing.md` § *« Les échecs attendus »* (les sept
   de la KF-029, #97) ; aucun autre. **Sonde positive jointe** : avant la suite, un appel qui rend la date du serveur
   (`GET /api/v1/accounts/{id}/open-items` sans `asOf` : `asOf` vaut `2027-01-04`, `crates/kesh-api/src/routes/letterings.rs:388`),
   `todayInZurich()` du runner qui commence par `2027-`, et `new Date().getFullYear()` du navigateur à 2027
   (`page.evaluate`) — par un script hors dépôt, journal joint.
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

La story n'ajoute pas de test : elle **corrige les montages** de seize tests. Le rouge de chacun est déjà constaté, sans
correctif, par le relevé de la fiche (treize, `inventaire-A2027.log`) et, pour les trois de D2, par celui de la 35-5c
(`kesh-gate-logs/e355c/nuit-B2.log`, instant `2026-03-10T23:30:00Z` : 2 rouges sur les binaires ciblés ; `services_rollout`
aux sondes `kesh-gate-logs/e355b/nuit-B3.log`) — à rejouer à l'étape 0 à la tête de la branche, journal
`kesh-gate-logs/e36-7/rouges.log`.

**Mutations** (script versionné dans `kesh-gate-logs/e36-7/mut/`, code commité avant, restauration
`git restore --worktree -- <fichier>` puis `touch`) : pour chaque fichier corrigé, ramener **une** date ou un exercice à
sa forme de 2026 ; la suite ciblée doit rougir sous `KESH_TEST_INSTANT=2027-01-04T10:00:00Z` (journal
`mut/M<n>.log`). Sept fichiers Rust, plus `services_rollout.rs` : huit mutations au moins. Une mutation qui ne rougit pas
signale un montage qui ne dépend plus du jour (à dire : soit juste, soit test muet).

## Tâches

1. **Étape 0** : relire `e36-choix-autonomes-e360.md` (C-e360-13) et ce registre ; constater la présence de libfaketime
   (D3) ; rejouer le relevé de la fiche à la tête (`rouges.log`) ; lire la cause d'`open_items_invariant` (D5) et la citer.
2. Corriger les montages des sept fichiers du tableau (D1), puis les trois sites de D2.
3. Mettre à jour `docs/testing.md:119` (AC8) ; recettes de D3/D4.
4. Mutations (AC3 c).
5. Gates (§ *Dev Notes*), AC1, AC2, AC4, AC6 ; AC5 à la clôture.
6. `Status: review`, clé `review`, Dev Agent Record, prompt de la revue P1.

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

`kesh-api` (tests : `admin_full_import_e2e.rs`, `bank_statements_manual_e2e.rs`, `dates_bornes_e2e.rs`,
`services_rollout.rs`), `kesh-db` (tests : `journal_entries_modification.rs`, `migrations_upgrade_path.rs`,
`opening_complement_repository.rs`), `kesh-report` (tests : `open_items_invariant.rs`), `docs` (`testing.md`) : **quatre
modules**, aucun fichier de production. Pas de découpage.

### Gates (voie 2 ; `e35-dev-commun.md` § *Quels gates, quand*)

- **`kesh-db` est touché** (tests de `migrations_upgrade_path` compris) : gate complet en cours de boucle
  (`scripts/test-fast.sh --ci`, base `kesh_e367` remise à neuf), non ciblé. ⚠️ Ne pas toucher aux sites positionnels de
  `migrations_upgrade_path.rs` (`apply_migrations_up_to`, `migrations.len()`, P6) : seules les dates changent.
- **Passage en `review`** : `test-fast --ci` (heure réelle **et** sous l'instant du critère) ; `scripts/gate-ci.sh
  kesh_ci_e367` en voie 1. Frontend non touché : pas de frontend complet, sauf si AC5 fait toucher une spec.
- **Clôture** : `gate-ci` et E2E complet à l'heure réelle au dernier commit de code, **puis** AC5 sous horloge décalée.
- Une commande lourde à la fois (E7). Verrou voie 2 : `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock
  scripts/mem-guard.sh …`, `TMPDIR=/tmp/kesh-voie2`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e367`.
- faketime et cargo : construire **avant**, à l'heure réelle (`cargo nextest run --no-run`), puis lancer sous faketime — une
  horloge décalée en avant fait paraître les sources anciennes, sans reconstruction, mais un `build` sous faketime laisserait
  des dates de fichiers dans le futur.

### Fichiers communs avec les autres stories de la phase 2

`crates/kesh-api/tests/admin_full_import_e2e.rs` : possible avec les lots 36-11 (0.19.0) ; `docs/testing.md` : aucun autre
attendu en 0.18.0. 36-10 (#1147, #1053, dates UTC) touche la date du jour côté production : si elle part en parallèle, ses
tests suivent D1 (année du jour, pas 2026). Aucun fichier de manuel : `git grep -nE 'KESH_TEST_INSTANT|KF-060|#1125|faketime'
-- docs/manual` ne rend rien.

### Ce qui doit rester vrai

- Les tests corrigés exercent toujours la date du jour (aucune date fixe substituée à `aujourd_hui_test()`, D1).
- Les montages restent verts **à l'heure réelle** et sous tout instant de 2026 à 2030 (exercice de la base de dev).
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
