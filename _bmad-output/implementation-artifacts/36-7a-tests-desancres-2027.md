# Story 36-7a : les tests ne figent plus 2026 — verts au 1er janvier 2027 (#1125, KF-060)

Status: ready-for-dev

*Fiche issue du découpage de la fiche `36-7-tests-desancres-2026.md` (`Status: split`), approuvé par l'orchestrateur le
2026-11-15 sur la règle de Guy du même jour (« éviter les stories trop longues ou trop complexes ») ; choix C-36-7-7.
Rédigée par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-7`, branche
`story/36-7-tests-desancres-2026`, partie de `epic/36-0.18.0` (`0bf3c3b4b`). Release **0.18.0**, phase 2, rang 7,
**première de la phase** (D1 du 2026-11-15). **Aucune migration**, aucune ligne de code de production, aucune clé i18n,
aucun changement d'API. Code lu à `0bf3c3b4b`. **Échéance ferme : d'ici le 2026-12-28** (décision de Guy, Q1 ;
C-e360-13) — ⚠️ **la fusion doit précéder le 2026-12-26** : `open_items_invariant` rougit **à l'heure réelle** dès ce
jour-là (D5 ; sondes `sonde-p2-S1a.log` vert au 2026-12-24, `sonde-p2-S1b.log` rouge au 2026-12-26 ; validation P2,
V2-3). Voie de base **2** (`kesh-mariadb-dev2:3307`), `gate-ci.sh` en voie 1. Clé courte des bases proposée : `e367a`
(à confirmer par l'orchestrateur). Choix : `e36-choix-autonomes-36-7.md` (C-36-7-1 à C-36-7-8). Relevés de la fiche :
`/home/gcorbaz/devel/kesh-gate-logs/e36-7/` ; journaux de la story : le même dossier (V7-3). Sœur : `36-7b-tests-bornes-anterieures.md`
(les neuf tests qui rougissent sous un instant antérieur à leur date de 2026), développée après la fusion de celle-ci.*

## Story

En tant que mainteneur de Kesh, je veux que la suite de tests — Rust, Vitest, Playwright — reste verte le 1er janvier 2027
et après (au moins jusqu'au 2029-12-31, exercice de la base de dev ; suite entière jouée sous `2029-06-15`, AC2), sans qu'aucun test ne fige l'année 2026 contre la date du jour, afin que la CI ne devienne pas rouge au premier
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
| `admin_full_import_e2e` ×2 | `:4899-4901` `v0174a_decor` : `VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')` ; pièces `"invoiceDate": "2026-06-15"` (`:4990`), `"paymentDate": "2026-06-20"` (`:5039`). Troisième consommateur du décor, hors des treize : `full_import_round_trip_keeps_a_draft` (`:5152-5156`) | l'annulation `/api/v1/supplier-invoices/3/cancel` (contre-passation au jour) |
| `bank_statements_manual_e2e` | `:218-220` `VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')`, dans `setup()`, décor de 21 tests — **non modifié** : le test corrigé replace l'exercice par un `UPDATE` (V4-1, D1) ; pièces en dur `:253` `"bookingDate": "2026-09-15", "valueDate": "2026-09-15"`, `:1008` écriture `'2026-08-20'`, `:1036` facture `'F-2026-0042'` du `'2026-08-20'` | l'annulation du rapprochement (`:1141` « Annulation du rapprochement. », `:1149` `assert_eq!(s, 200, "{r}")`) : contre-passation du règlement au jour |
| `dates_bornes_e2e` ×4 | témoin `"2027-01-01"` (`:1010`, `:1050`, `:1510`, `:1556`) pour `dueDate`, face à `"date": aujourd_hui()` (`:1017`) | la facture du jour : au 2027-01-04, l'échéance témoin la précède |
| `journal_entries_modification` ×2 | `:268-275` `societe()`, exercice du seed « ramené à l'année 2026 » (`UPDATE fiscal_years SET name = 'Exercice 2026', start_date = '2026-01-01', end_date = '2026-12-31'`). **Cinq** tests le montent (`/usr/bin/grep -c 'societe(&pool)'` rend 5 : `:372`, `:424`, `:486`, `:538`, `:592`), dont les deux rouges et trois verts ; deux d'entre eux créent un exercice postérieur par SQL brut, sans contrôle de chevauchement (`exercice()`, `:336-356`) : `:425` et `:539` `exercice(&pool, &s, "Exercice 2027", 2027, "Open")`, nom assertionné `:458`, `:571`. Écritures `d(2026, 3, 1)` (`:373`, `:426`, `:487`, `:540`, `:593`), corps `d(2026, 3, 2)` (`:388`, `:437`), `d(2026, 3, 1)` (`:612`) | la contre-passation `crate::jour::aujourd_hui_test()` (`:381`) |
| `migrations_upgrade_path` ×2 | exercice de la base 0.16 / rc1 peuplée `:1413-1415` (`VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31') RETURNING id`) ; pièces `:1319` (`entry_date` `2026-06-15`), `:1332` (`'2026-06-15', '2026-07-15'`), `:1498` (`day = … from_ymd_opt(2026, 6, 20)`), `:1716` (`invoice_date` `2026-06-15`) | `cancel_in_tx(…, crate::jour::aujourd_hui_test())` (`:1515-1520`), puis les opérations du jour d'après la montée (`:1537`, `:1661`, `:1701`, `:1742`) |
| `opening_complement_repository` | `setup(&pool, &[year_span(2026)])` (`:514`), écriture `d(2026, 2, 1)` (`:519`) ; `complete(…, d(2026, 6, 1))` (`:533`) passe en 4e argument le **jour** (`today`, `:163-175`), non une date de pièce ; `year_span` (`:47-49`) appelé ailleurs (`:246`, `:277`, `:298`, `:311`, `:331`, `:346`, `:355`, `:383`, …) par des tests verts | `journal_entries::reverse(…, crate::jour::aujourd_hui_test())` (`:523-528`) |
| `open_items_invariant` | exercices 2024 (clos) et `fy` à partir du 2025-01-01 (`:106-116`) ; groupe « à cheval » `a1` `ymd(2024, 6, 30)` / `a2` `ymd(2026, 3, 1)` (`:208-218`), lettré `:220` ; verrou des livres de la fixture partagée à **jour − 300** (D5) | le lettrage `:220` refusé `LetteringAllLinesInClosedPeriods` (cause établie, D5) |

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

- **D1 — Le montage prend ses années du jour du test (N = année de `aujourd_hui_test()`), et chaque date passée se place
  selon ce qui la contraint.** *(Récrite à la remédiation de la validation P2, V2-1, V2-2, V2-4 : la version précédente
  disait « dans N-1, jamais dans N » et se donnait pour « juste pour tout jour d'une année » ; c'est faux sous le verrou
  glissant d'`open_items_invariant`, et sous l'instant `2027-01-04` N-1 vaut 2026, si bien que rien ne la prouvait.)* La
  règle doit tenir **pour tout jour du 2026-01-01 au 2029-12-31** (borne : exercice de la base de dev 2020-2030, garde
  `y <= 2029` du modèle), deux formes selon le site :
  - **(a) Dates contraintes par un exercice** (cas général) : l'exercice du montage couvre **N-1 et N** — un seul exercice
    `d(N-1, 1, 1)`–`d(N, 12, 31)`, ou deux civils, au choix du développeur, ligne lue — sauf `societe()`, où l'exercice
    unique est prescrit (ci-dessous, V3-8) ; pièces, paiements et bornes qui
    doivent être passés se datent en **N-1** (`d(N-1, m, j)`) ; l'opération du jour garde `aujourd_hui_test()` — y compris
    le paramètre `today` d'un appel (`opening_complement_repository:533`, quatrième argument de `complete`, `:163-175`,
    qui est le jour et non une date de pièce ; V3-1). **Toutes** les dates de 2026 **du test corrigé** suivent, non
    seulement le site du tableau : l'étape 0 les relève par `/usr/bin/grep -nE '\b2026\b'` dans le test et les cite. Une date
    d'un **décor partagé** par d'autres tests (`bank_statements_manual_e2e::body()`, ci-dessous) ne se change pas : le test
    corrigé en dérive sa propre copie. *Écartée* : garder les dates de 2026 sous un exercice borné `2026-01-01`–`d(N, 12,
    31)` — juste jusqu'en 2029, mais aucune mutation ne la distinguerait de la forme d'origine, et l'année 2026 resterait en
    dur.
  - **`bank_statements_manual_e2e`** (V3-2, V4-1) : **`setup()` ne change pas** non plus. Son exercice
    (`:218-220`) est le décor de **21 tests** (`/usr/bin/grep -c 'setup(&pool)'` rend 21), dont
    `manual_statement_feeds_the_tile_and_k3` (`:1183`) : son montage reste en 2026 (écriture `'2026-09-10'`, `:1198-1199` ;
    relevé de `body()`, `:1225`), et le contrôle K3 n'examine un compte que si une transaction, un relevé ou une écriture
    tombe dans les bornes de l'exercice (`kesh-db/src/repositories/closing_checks.rs:461-474`, `EXAMINED_ACCOUNTS_SQL`).
    Avec un exercice N-1..N partagé, ce test rougirait dès que N-1 ≥ 2027 (assertion de montage `NO_BALANCE_STATEMENT`,
    `:1220-1223`), donc sous l'instant de 2029 de l'AC2. Le test corrigé, et lui seul, replace donc l'exercice
    `c.fiscal_year_id` en `d(N-1, 1, 1)`–`d(N, 12, 31)` par un **`UPDATE fiscal_years … WHERE id = ?`** juste après
    `setup()`, comme `societe()` le fait ; M2a porte sur cet `UPDATE`. *Écartée* : `setup()` en N-1..N et le test K3 dérivé
    en N-1, qui ajouterait un quatorzième test au périmètre. — `body()` (`:241-256`, période `2026-09-01`–`2026-09-30` `:244-245`, numéro
    `9/2026` `:246`, lignes `:250`, `:253`) sert de décor à **37 appels** du fichier (`/usr/bin/grep -c 'body('` rend 43,
    définition et autres fonctions comprises) et ses valeurs sont assertionnées par une quinzaine de tests (nom de fichier
    `saisie-manuelle-2026-09-01-2026-09-30` à `:327`, refus de ligne hors période `:487`, …) que la story ne corrige pas :
    **`body()` ne change pas**. `manual_statement_lines_reconcile_like_imported_ones` (`:1060`, qui l'appelle à `:1064`)
    construit son relevé **dérivé** : période, numéro et dates de lignes décalés en N-1 (même mois, même jour), le reste de
    `body()` intact. Les pièces de ce test, posées par **INSERT SQL brut** (écriture `:1008`, facture `:1036`, au
    `'2026-08-20'`), suivent en N-1. Un INSERT brut ne contrôle pas l'exercice, mais cela ne rend pas sa mutation
    équivalente : ce qui compte est une **lecture ultérieure sous une règle de date** (V4-4). Ici, la proposition de rapprochement
    exige `F ≤ D + 1 jour` et `A ≥ D − 30 jours` (`kesh-db/src/repositories/reconciliation.rs:402-408`, `:116-118`) ; la
    facture de `:1036` n'a pas d'échéance (colonnes de l'INSERT, `:1034-1036`), donc `A = F` — restée en 2026 face à un
    relevé de 2028, elle ne serait plus candidate (assertion `:1098-1100`, « la facture est candidate »). L'écriture de
    `:1008` ne participe pas à cette fenêtre et suit par cohérence ; son équivalence **n'est pas déclarée** : la tâche 4,
    après le correctif commité, **joue sa mutation** sous `2029-06-15T10:00:00Z` (journal) — seule preuve admise, aucune
    équivalence par simple lecture (V6-1, V6-2).
  - **(b) Dates contraintes par un verrou glissant** (`open_items_invariant`, verrou `jours_avant(300)` de la fixture,
    D5) : la date suit le jour, `jours_avant(k)` avec **0 < k < 300**, dans un exercice ouvert qui la couvre (`fy`, à partir du
    2025-01-01 : `jours_avant(k)` y tombe pour tout jour à partir du 2026-01-01 dès que k ≤ 365). Une date en N-1 y serait
    fausse une partie de l'année (`d(N-1, 12, 1)` passe sous le verrou dès la fin septembre de N).
  - Le décalage interdit est celui qui **sort de l'exercice du montage**, non tout décalage qui traverse le 1er janvier.
  - Sites particuliers : le témoin de `dates_bornes_e2e` devient une échéance **postérieure** au jour
    (`aujourd_hui() + n` jours, n > 0) ; `journal_entries_modification::societe()` pose **un exercice unique**
    `d(N-1, 1, 1)`–`d(N, 12, 31)` — non deux civils (V3-8) : ses cinq tests sont des tests d'ordre de verrous, et dans
    l'origine l'écriture et sa contre-passation du jour tombaient dans **un** exercice ; deux exercices feraient verrouiller
    à `reverse` un second exercice (`find_open_covering_date`, `journal_entries.rs:2727-2735`) et changeraient ce que ces
    tests exercent ; aucune contrainte ne borne la durée d'un exercice (`chk_fiscal_years_dates`,
    `20260404000001_initial_schema.sql:51`, ne compare que début et fin) — ses écritures et corps en N-1, et l'exercice **postérieur** des deux tests `…close_of_a_later_year…` (`:425`, `:539`) se crée en **N+1**,
    nommé `format!("Exercice {}", N + 1)` (assertions `:458`, `:571` assorties) — aucun chevauchement avec N-1..N, vérifié
    sous l'instant `2029-06-15` (N+1 = 2030, dans la borne) ; `migrations_upgrade_path` : exercice de la base peuplée
    (`:1413-1415`) en N-1..N, pièces `:1319`, `:1332`, `:1498`, `:1716` en N-1, sites positionnels intouchés (P6) ; `:1332`
    est un INSERT brut (`INSERT INTO supplier_invoices … '2026-06-15', '2026-07-15'`) ; il suit par cohérence. Son équivalence **n'est
    pas déclarée** (V5-1) : `validate_in_tx` lit `invoice_date` (`kesh-db/src/repositories/supplier_invoices.rs:1584`,
    `:1718`), les factures revalidées passent d'abord par `update_draft` (`migrations_upgrade_path.rs:1716`), mais la
    migration `20261106000001` **classe par `invoice_date`** pour poser le numéro interne (`:47`, `:63` : `ORDER BY
    f.start_date, s.invoice_date, s.id`). Comme pour `:1008`, la tâche 4, **après le correctif commité**, joue la mutation sous
    `2029-06-15T10:00:00Z` (V6-1) ; seulement si elle reste verte, elle l'écrit équivalente, avec la raison constatée (par exemple : le test n'asserte aucun
    numéro interne, à vérifier alors, ligne lue).
  **Modèle** au dépôt pour (a) : `crates/kesh-db/tests/supplier_invoices_draft.rs:2059` `let date = jour(x.y - 2, 6, 15);`,
  avec `scinder` (`:1522-1547`), qui pose les exercices `y - 2`, `y - 1`, `y` depuis `let y = aujourdhui().year();`
  (`:1524`) et garde `assert!(y <= 2029, …)` (`:1525-1528`). **Preuve** : l'AC2 joue l'instant `2029-06-15T10:00:00Z`, où
  N-1 = 2028 ≠ 2026, et les mutations des sites (a) y sont tuées (§ *Tests*). *Écartées* : passer à chaque opération une
  date fixe de 2026 (`cancel(…, 2026-06-20)`) — le test cesserait d'exercer la date du jour ; un exercice 2020-2030 dans
  chaque montage — il masquerait le jour et repousse la panne à 2031.
- **D2 — Le périmètre est celui des treize rouges de 2027** (C-36-7-7). Les neuf tests qui ne rougissent que sous un
  instant antérieur à leur date de 2026 sont la 36-7b. *Écartée* : une seule story pour les vingt-deux (validation P1 de
  la fiche d'origine, axe 5 : trop longue, et la dépendance à libfaketime aurait retenu les deux).
- **D3 — L'E2E en 2027 se joue sous une horloge de processus (libfaketime), non sous `KESH_TEST_INSTANT`** (C-36-7-2). Le
  binaire servi ignore la variable (`docs/testing.md:119`, garde `date_du_jour_garde.rs`) et le runner lit l'heure réelle
  (`api-fixtures.ts:393`). *(Commande récrite à la remédiation de la validation P2, V2-5 et V2-9 ; chaque option est tirée
  de `man faketime` ou d'une exécution, citée.)* La forme, pour toute commande sous horloge décalée — le verrou et
  `mem-guard` **dehors**, à l'heure réelle, `faketime` en dernier :

  ```sh
  flock <verrou de voie> scripts/mem-guard.sh env TZ=Europe/Zurich NO_FAKE_STAT=1 \
    faketime --exclude-monotonic -f '@2027-01-04 11:00:00' <commande>
  ```

  - `-f '@AAAA-MM-JJ hh:mm:ss'` : « Start-at timestamps […] The wall clock will start counting at the given timestamp »
    (`man faketime`, *ADVANCED TIMESTAMP FORMAT*) — l'horloge part de l'instant et avance ; elle se lit dans le fuseau du
    processus, d'où `TZ=Europe/Zurich` (11 h à Zurich = 10:00Z, constaté : `sonde-p2-S3.log`, « avec faketime :
    2027-01-04T10:00:00Z »).
  - `--exclude-monotonic` : « Do not fake time when the program makes a call to clock_gettime with a CLOCK_MONOTONIC
    clock » (`man faketime`) — les délais de tokio et des minuteries restent réels.
  - `NO_FAKE_STAT=1` : « Faking of filesystem timestamps may be disabled by setting the NO_FAKE_STAT environment variable
    to a non-empty value » (`man faketime`) — sans lui, faketime décale aussi les dates des fichiers (« Filesystem
    timestamps will also be reported relative to this timestamp ») ; cargo ne doit pas les voir bouger.
  - `-m` (« use the multi-threading variant of libfaketime ») : **non tranché par la documentation** ; la variante simple
    se joue d'abord, et si l'étage serveur de la sonde de l'AC5 échoue ou si le backend ne répond pas, `-m` est rejouée. La
    variante jouée s'écrit en tête du journal. Les deux rendent l'instant sur `date` (`sonde-p2-S3.log`).
  - Un **binaire construit avant**, jamais `cargo run` ni `cargo build` sous faketime : `target/debug/kesh-api` pour le
    backend (comme `kesh-gate-logs/e36-3/e2e-36-3.sh`), `cargo nextest run --no-run` à l'heure réelle puis
    `cargo nextest run` sous faketime pour l'AC4.
  - Chaque processus fils démarre sa propre horloge à l'instant (« a new instance of libfaketime is used for each child
    process, which means that the libfaketime start time […] will also be re-initialized », `man faketime`, *BUGS*) :
    l'écart entre deux processus vaut leur écart de démarrage, sans effet sur un jour.
  - `mem-guard` exécute par `systemd-run --user --scope` (`~/.local/bin/mem-guard:185`) : faketime placé **après** lui ne
    dépend pas de la propagation de l'environnement au scope, et ne fausse ni `flock` ni `systemd-run`.
  - **Détecteur et sonde positive** (V2-5 : `ldconfig -p | /usr/bin/grep -i faketime` rend vide **avec** la bibliothèque
    installée, sous `/usr/lib/x86_64-linux-gnu/faketime/`) : `ls -l /usr/bin/faketime
    /usr/lib/x86_64-linux-gnu/faketime/` ; `TZ=Europe/Zurich faketime --exclude-monotonic -f '@2027-01-04 11:00:00' date -u
    +%FT%TZ` rend `2027-01-04T10:00:00Z` et la même commande sans faketime la date réelle ; `node -e 'console.log(new
    Date().toISOString())'` sous faketime rend `2027-01-04T10:00:00.033Z`, et `stat -c %y /usr/bin/faketime` sous
    `NO_FAKE_STAT=1` rend la date réelle du fichier (`2026-01-20 20:37:26`) — constaté le 2026-11-15 avec la commande
    ci-dessus, `sonde-p2-S3b.log` ; l'étape 0 rejoue la sonde.
  Que Chromium hérite de `LD_PRELOAD` est une **hypothèse non éprouvée** (le navigateur de Playwright lance ses processus
  avec son bac à sable) : l'étage navigateur de la sonde de l'AC5 la tranche (D6). MariaDB, en conteneur, reste à l'heure réelle (angle mort écrit : `NOW(3)` de
  `refresh_tokens.rs:61`, `:221`, `UTC_TIMESTAMP(6)` d'`invoice_reminders.rs:176`, `invoices.rs:2620`,
  `CURRENT_TIMESTAMP(3)` de `journal_entries.rs:1658`, `payment_batches.rs:640`, `DEFAULT CURRENT_TIMESTAMP` des colonnes
  d'horodatage ; liste non close). **Un rouge ne se rattache à cet angle mort que prouvé** : la ligne de SQL qui lit
  l'heure de la base, la colonne, et un rejeu isolé du test qui montre que le rouge tient à l'écart base/processus (par
  exemple : vert sous `KESH_TEST_INSTANT` seul, rouge sous faketime). Sans cette preuve, il compte comme un rouge de la
  story. **Repli si le navigateur reste en 2026** (étage 3 de la sonde) : aucun contournement n'est éprouvé ; l'AC5 se
  déclare **non constatable** pour la partie navigateur — non « vert » —, avec les étages serveur et runner joints, et
  l'orchestrateur arbitre (piste à éprouver, non prescrite : `page.clock` posé par un fixture). **Prérequis** : libfaketime,
  **installée** sur la station depuis le 2026-11-15 (`/usr/bin/faketime`, `/usr/lib/x86_64-linux-gnu/faketime/libfaketime.so.1`
  et `libfaketimeMT.so.1`, `sonde-p2-S3.log`) ; la fiche de la P1 la disait absente sur un détecteur qui ne la voit pas.
  *Écartées* : faire lire `KESH_TEST_INSTANT` au binaire en `KESH_TEST_MODE`
  (code de production, contraire à la garde) ; `page.clock` seul (le serveur resterait en 2026) ; déclarer l'E2E « vert en
  2027 » sur la seule lecture.
  **AC4 et AC5 sont des gates de clôture** (C-36-7-6) : le développement **peut passer en `review` sans libfaketime**
  (AC1-AC3, AC6-AC9) ; la story **ne passe pas `done`** tant qu'AC4 et AC5 n'ont pas tourné, journaux joints. Si la
  sonde du détecteur échoue à l'étape 0, la **première ligne** du rapport de l'agent le dit. La PR de release ne porte
  `closes #1125` qu'avec la 36-7b close **et** AC4, AC5 joués.
- **D4 — Aucun outillage versionné de plus** (C-36-7-4). Les recettes (nextest et E2E sous horloge décalée, Vitest sous
  préchargement) s'écrivent dans `docs/testing.md`, à côté du paragraphe de `KESH_TEST_INSTANT` ; le préchargement Vitest et
  les scripts restent hors dépôt. *Écartée* : un `scripts/test-2027.sh` ou un setup Vitest qui lit `KESH_TEST_INSTANT`,
  fonction d'outillage neuve, à proposer en issue si l'orchestrateur le juge utile.
- **D5 — La cause du rouge d'`open_items_invariant` est établie** *(récrite à la remédiation de la validation P2, V2-2 ;
  la version précédente la laissait à l'étape 0)*. Ligne lue : la fixture partagée (`open_items_invariant.rs:37` `#[path =
  "../../kesh-db/tests/support/lettering_documents.rs"]`) pose le verrou des livres à **jour − 300** —
  `lettering_documents.rs:51` `pub fn jours_avant(n: i64) -> NaiveDate {`, `:552` `let ancienne = jours_avant(300);`,
  `:556-562` `companies::lock_books(pool, seeded.admin_user_id, seeded.company_id, ancienne, jour::aujourd_hui_test())` ; une
  ligne n'est en période ouverte que si sa date est strictement postérieure à la borne (`letterings.rs:89-94`). Le groupe à
  cheval (`:203` « 2024-06-30 et 2026-03-01 (au-dessus de la borne) ») est `a1` `ymd(2024, 6, 30)` et `a2` `ymd(2026, 3,
  1)` (`:216`), lettré à **`:220`** ; dès que le verrou atteint le 2026-03-01, les deux lignes sont closes et `:220` est
  refusé (la panique porte `:181`, l'`.expect("lettrage")` de la fermeture). L'appel `:185` (`p1`, `p2` à `jours_avant(60)`,
  `jours_avant(50)`) n'est pas en cause. Le verrou atteint le 2026-03-01 quand le jour vaut 2026-03-01 + 300 j =
  **2026-12-26** : le test **rougit à l'heure réelle dès le 2026-12-26** — exécuté sur `binary(open_items_invariant)` à la
  tête : vert sous `KESH_TEST_INSTANT=2026-12-24T10:00:00Z` (`sonde-p2-S1a.log`, 1 passé), rouge sous
  `2026-12-26T10:00:00Z` (`sonde-p2-S1b.log`, 1 échoué). Correctif : `a2` suit D1 (b), `jours_avant(k)`, 0 < k < 300
  (dans `fy`) ; M10 le défait. L'étape 0 confirme l'appel fautif par une exécution sous `RUST_BACKTRACE=1` (ligne citée).
- **D6 — L'étage navigateur de la sonde de l'AC5 est une spec versionnée, jouée par le runner de la suite** (C-36-7-8 ;
  V2-9). `frontend/tests/e2e/horloge-sonde.spec.ts` : le jour du navigateur (`page.evaluate`, `toLocaleDateString('sv-SE',
  { timeZone: 'Europe/Zurich' })`), celui du runner (`todayInZurich()`) et celui du serveur (`asOf` de `GET
  /api/v1/accounts/{id}/open-items` sans `asOf` sur la créance 1100, compte lettrable du preset — la route refuse un compte
  non lettrable, `routes/letterings.rs:513-517`) sont égaux. Vraie à toute heure réelle (les trois horloges disent le même
  jour), elle est sous faketime la sonde à trois étages, dans les conditions exactes de la suite (`playwright.config.ts:22`
  `testDir: 'tests/e2e'`, `:59` `timezoneId: 'Europe/Zurich'`). Elle lit l'horloge du navigateur : la garde
  `src/lib/shared/utils/local-date-guard.test.ts` (`ALLOWLIST_SPECS`, liste close de trois sites) reçoit son entrée,
  fragment au texte exact, justifiée, et les trois mentions du nombre passent à quatre (V3-5) : `:28-29` (« `ALLOWLIST_SPECS`
  est close de trois entrées »), `:263` (« Les trois sites admis »), `:318` (titre « les specs ne portent que les trois
  sites de l'allowlist »). Angle mort écrit : à minuit à Zurich, les trois lectures peuvent tomber de part et d'autre
  (fenêtre de quelques millisecondes). **Pourquoi une spec versionnée** (V3-4, C-36-7-8 amendé) : l'alternative — une
  configuration Playwright **hors dépôt** qui reprend celle du dépôt et ne déplace que `testDir` — a été **essayée** le
  2026-11-15 (`kesh-gate-logs/e36-7/sonde-pw/playwright.sonde.config.ts`, `npx playwright test --config … --list` depuis
  `frontend/`, journal `sonde-pw-list.log`) : la spec hors dépôt résout `@playwright/test` hors du `node_modules` du
  frontend, dans le paquet système (`/usr/share/nodejs/playwright/lib/common/testType.js:70`), et Playwright refuse
  (« Playwright Test did not expect test() to be called here […] You have two different versions of @playwright/test »),
  0 test listé, `rc=1`. La faire marcher demanderait un contournement non éprouvé (liens vers `node_modules`) ; la spec
  versionnée, elle, sert aussi les campagnes de fin d'année. Le prix : le module `frontend`, cinquième de la story, à la
  limite de la règle. *Écartées* : la configuration hors dépôt (ci-dessus) ; un `chromium.launch()` autonome (autres
  conditions que la suite) ; aucune sonde navigateur (l'hypothèse de `LD_PRELOAD` resterait non éprouvée).

## Critères d'acceptation

1. **AC1 — nextest vert sous l'instant du critère.** `scripts/test-fast.sh --ci`, sous
   `KESH_TEST_INSTANT=2027-01-04T10:00:00Z`, voie 2, base remise à neuf : 0 échec (journal
   `kesh-gate-logs/e36-7/gate-2027.log`, commit cité). Les treize tests du tableau sont verts.
2. **AC2 — et sous les autres réglages, sans régression.** La même suite verte sous `2026-12-31T23:30:00Z` (nuit suisse du
   1er janvier 2027 : neuf rouges à la base, `inventaire-N2027.log`), sans la variable (heure réelle) et sous
   `2029-06-15T10:00:00Z` (N-1 = 2028 ≠ 2026, V2-1 ; **suite entière**, une prise de verrou, V3-7 — à la base, les sept
   binaires corrigés y rendaient 216 exécutés, 13 échecs, les treize, `sonde-p2-S2.log` ; la suite entière n'y a pas été
   relevée à la base, l'étape 0 la relève, journal `kesh-gate-logs/e36-7/inventaire-C2029.log` (V6-3, V7-3) ; un rouge hors des treize sous cet instant **compte contre l'AC2 s'il vient de
   la story** — tel `manual_statement_feeds_the_tile_and_k3` si `setup()` était modifié, V4-1 — ; sinon il se nomme, ne
   compte pas contre l'AC, et une issue est **demandée à l'orchestrateur**, avec la preuve qu'il rougit déjà à la base sous
   le même instant, journal joint) ; `binary(open_items_invariant)` vert sous `2026-12-26T10:00:00Z`
   (rouge à la base, `sonde-p2-S1b.log`). **Non-régression sous l'instant antérieur** : sous `2026-03-10T23:30:00Z`, les
   rouges sont exactement les neuf de la 36-7b (`inventaire-B2026.log`), aucun autre. Un rouge de charge se relance et se
   nomme, sans compter contre l'AC.
3. **AC3 — Sonde positive de la lecture.** (a) Les relevés `inventaire-A2027.log` (13 rouges) et `inventaire-N2027.log`
   (9) sont rejoués à l'étape 0 à la tête de la branche et joints ; (b) après le correctif, les **sept binaires corrigés**
   (`cargo nextest run -E 'binary(admin_full_import_e2e) | binary(bank_statements_manual_e2e) | binary(dates_bornes_e2e) |
   binary(journal_entries_modification) | binary(migrations_upgrade_path) | binary(opening_complement_repository) |
   binary(open_items_invariant)'`) rougissent sous `KESH_TEST_INSTANT=pas-un-instant` (panique d'`instant_de_test`, message
   qui nomme la variable) ; (c) les mutations M1a à M10 de § *Tests*, chacune tuée par le test nommé sous l'instant nommé.
4. **AC4 — nextest sous horloge de processus décalée** (D3 ; **gate de clôture**, C-36-7-6) : `cargo nextest run
   --workspace --profile ci` sous la commande de D3 (`flock … mem-guard … env TZ=Europe/Zurich NO_FAKE_STAT=1 faketime
   --exclude-monotonic -f '@2027-01-04 11:00:00' cargo nextest run …`, variante `-m` dite au journal), **sans**
   `KESH_TEST_INSTANT`, binaires construits avant à l'heure réelle (`--no-run`) : 0 échec, ou chaque rouge nommé
   et rattaché — à un site de la story, ou à l'angle mort MariaDB **avec la preuve exigée par D3** et une issue. Sonde : un
   test du tableau ramené à sa version de la base rougit sous la même commande.
5. **AC5 — E2E complet en 2027** (D3 ; **gate de clôture**, C-36-7-6) : la suite Playwright complète, backend et runner
   sous la commande de D3 — le backend par son **binaire construit** (`target/debug/kesh-api`), le runner par `npx
   playwright test` —, base `kesh_e2e_e367a` recréée : les échecs se nomment un par un contre `docs/testing.md` § *« Les
   échecs attendus »* (les sept de la KF-029, #97) ; aucun autre. **Sonde positive jointe** : la spec de D6
   (`horloge-sonde.spec.ts`), jouée d'abord seule par le même runner sous la même horloge, rend les trois jours égaux à
   `2027-01-04` — serveur (`asOf` par défaut, `routes/letterings.rs:311` « absent (ou vide) → aujourd'hui », tenu par
   `postes_ouverts::lister`, `:524-530`), runner (`todayInZurich()`) et navigateur (`page.evaluate`) ; journal joint, avec
   la variante (`-m` ou non) jouée. Si l'étage navigateur rend 2026, repli de D3 (non constatable pour le navigateur,
   arbitrage), jamais « vert ».
6. **AC6 — Vitest reste vert en 2027** : le relevé `inventaire-vitest-A2027.log` se rejoue au dernier commit de code, avec
   sa sonde.
7. **AC7 — Aucune ligne de production ne bouge** : `git diff --stat origin/epic/36-0.18.0...HEAD -- 'crates/*/src/*'
   'crates/*/src/**' frontend/src` ne rend que `frontend/src/lib/shared/utils/local-date-guard.test.ts` (entrée de D6),
   fichier de test.
8. **AC8 — La documentation dit vrai** : `docs/testing.md:119` dit ce que `KESH_TEST_INSTANT` ne couvre pas (binaire servi,
   runner, navigateur, deux modules de tests de `src`) et donne les recettes de D3 et D4 ; l'exception (a) nomme les neuf
   tests de la 36-7b (la 36-7b la retire) ; la phrase « `local-date-guard.test.ts` parcourt aussi `tests/e2e` et n'y
   admet que trois sites, nommés » passe à **quatre** sites (entrée de D6 ; V4-3).
9. **AC9 — Garde intacte** : `date_du_jour_garde.rs` verte sans retouche de ses listes closes ; `local-date-guard.test.ts`
   vert avec la seule entrée de D6 ajoutée (quatre sites) et ses trois mentions du nombre mises à jour (`:28-29`, `:263`,
   `:318`).

## Tests à écrire, et la preuve du rouge

La story **corrige les montages** de treize tests, dans sept fichiers, et **ajoute une spec**, la sonde d'horloge de D6.
Le rouge de chacun des treize est constaté, sans correctif, par `inventaire-A2027.log` (et neuf d'entre eux par
`inventaire-N2027.log`), celui d'`open_items_invariant` à l'heure du 2026-12-26 par `sonde-p2-S1b.log` — à rejouer à
l'étape 0 à la tête de la branche, journal `kesh-gate-logs/e36-7/rouges.log`. La spec de D6 est verte à l'heure réelle
par construction ; son rouge se prouve en la jouant sous faketime appliqué au runner **seul** (backend à l'heure réelle) :
le jour du serveur diffère, elle rougit (journal `rouges.log`).

**Mutations** (script versionné dans `kesh-gate-logs/e36-7/mut/`, code commité avant, restauration
`git restore --worktree -- <fichier>` puis `touch`) : **une par site corrigé** (ou par groupe de sites qui partagent un
même montage, dit comme tel) ; chacune remet le site à sa **forme d'origine** (pour les témoins de `dates_bornes_e2e`, le
littéral `"2027-01-01"` ; ailleurs l'année 2026 ; V2-10) et doit être **tuée par le test nommé, sous l'instant nommé**
(journal `mut/M<n>.log`). Les parts « exercice N » se tuent sous `2027-01-04` ; les parts « pièces en N-1 » y seraient des
**mutants équivalents** (N-1 = 2026, V2-1) et se tuent sous `2029-06-15`.

| M | Site corrigé | Test qui la tue | Instant |
|---|---|---|---|
| M1a | `admin_full_import_e2e` `v0174a_decor`, exercice (`:4899-4901`) | `full_import_accepts_an_rc1_backup_with_supplier_invoices` (et `…_0_16_…`) | `2027-01-04T10:00:00Z` |
| M1b | `admin_full_import_e2e` pièces `:4990`, `:5039` remises en 2026 | idem ; `full_import_round_trip_keeps_a_draft` (`:5152-5156`) reste vert | `2029-06-15T10:00:00Z` |
| M2a | `bank_statements_manual_e2e`, l'`UPDATE` de l'exercice dans `manual_statement_lines_reconcile_like_imported_ones` retiré (`setup()`, `:218-220`, intouché ; V4-1) | ce test | `2027-01-04T10:00:00Z` |
| M2b | `bank_statements_manual_e2e` relevé dérivé du test (période, numéro, dates de lignes) remis aux valeurs de `body()` (2026) | idem : la facture en N-1 n'est plus candidate (`F ≤ D + 1`, `reconciliation.rs:402-408` ; assertion `:1098-1100`) | `2029-06-15T10:00:00Z` |
| M2c | `bank_statements_manual_e2e:1036` (facture) remise au `'2026-08-20'` | idem : `A = F` hors de `D − 30` (assertion `:1098-1100`) | `2029-06-15T10:00:00Z` |
| — | `bank_statements_manual_e2e:1008` (écriture, INSERT brut) | **jouée à la tâche 4**, après le correctif commité ; déclarée équivalente seulement si elle reste verte, raison constatée écrite ; aucune équivalence par lecture (V6-1, V6-2) | `2029-06-15T10:00:00Z` |
| M3 à M6 | `dates_bornes_e2e` témoins `:1010`, `:1050`, `:1510`, `:1556` remis à `"2027-01-01"` — une mutation chacun | `c_facture_echeance`, `c_facture_maj_echeance`, `c_fournisseur_echeance`, `c_fournisseur_maj_echeance` (appariés à l'étape 0, ligne lue) | `2027-01-04T10:00:00Z` |
| M7a | `journal_entries_modification` `societe()` (`:268-275`), exercice — un montage pour **cinq** tests | `update_waits_for_a_concurrent_reversal_then_refuses` (et `delete_…`) | `2027-01-04T10:00:00Z` |
| M7b | `journal_entries_modification` écritures et corps (`:373`, `:388`, `:426`, `:437`, `:487`, `:540`, `:593`, `:612`) remis en 2026 — un groupe | l'un des cinq tests, nommé à l'étape 0 | `2029-06-15T10:00:00Z` |
| M7c | exercice postérieur `:425`, `:539` remis à `2027` | `update_waits_for_a_concurrent_close_of_a_later_year_then_refuses` (et `delete_…`) : sous 2029, l'exercice 2027 est **antérieur** à N-1..N (2028-2029), `find_later_closed_in_tx(…, fy_start)` (`journal_entries.rs:1548`, `:1896` ; `fiscal_years.rs:892`, `FIND_LATER_CLOSED_SQL` : `WHERE company_id = ? AND start_date > ? AND status = 'Closed'`) ne le voit plus (V3-3). Premier rouge attendu : **l'attente** — le `PUT` ne bloque plus sur le verrou de B, et l'assertion `:443-448` (« le PUT doit attendre sur le verrou des exercices postérieurs ») échoue avant le `match` `LaterFiscalYearClosed` (`:452-461`) ; à défaut, **le refus** n'arrive pas. Le journal `mut/M7c.log` tranche (V4-5) | `2029-06-15T10:00:00Z` |
| M8a | `migrations_upgrade_path` exercice de la base peuplée (`:1413-1415`) | `draft_migration_preserves_a_populated_016_base` (et `…_rc1_…`) | `2027-01-04T10:00:00Z` |
| M8b | `migrations_upgrade_path` pièces `:1319` (`create_in_tx`), `:1498`, `:1716` remises en 2026 — une mutation chacune ; celles qui restent vertes une fois jouées à la tâche 4 se disent équivalentes, raison constatée ; `:1716` traité comme `:1332` (V7-1) | idem | `2029-06-15T10:00:00Z` |
| — | `migrations_upgrade_path:1332` (INSERT brut de `supplier_invoices`) | **jouée à la tâche 4**, après le correctif commité (V6-1) ; déclarée équivalente seulement si elle reste verte, raison constatée écrite — la migration `20261106000001` classe par `invoice_date` (`:47`, `:63`) (V5-1) | `2029-06-15T10:00:00Z` |
| M9a | `opening_complement_repository` exercice de `un_compte_contre_passe_reste_mouvemente` (`:514`) | ce test | `2027-01-04T10:00:00Z` |
| M9b | l'écriture `:519` remise à `d(2026, 2, 1)` | ce test : `create_in_tx` refuse une date hors de l'exercice N-1..N (`DateOutsideFiscalYear`, `journal_entries.rs:392`), `post` panique sur `.expect("écriture")` (`:144`) (V3-1) | `2029-06-15T10:00:00Z` |
| — | `:533`, le `today` de `complete` (passé à `aujourd_hui_test()`) | **jouée à la tâche 4**, après le correctif commité ; ne se dit équivalente qu'au constat (restée verte), raison constatée écrite — attendu à la lecture : `decide_date` rend `OpeningDay` quel que soit `today` (`opening_complement.rs:256-259`), `AccountMoved` suit (`:663`) (V7-2) | `2029-06-15T10:00:00Z` |
| M10 | `open_items_invariant` `a2` remis à `ymd(2026, 3, 1)` (D5) | `open_items_balance_matches_the_ledger_and_the_trial_balance` | `2027-01-04T10:00:00Z` |

Les autres appels de `year_span(2026)` d'`opening_complement_repository` (`:246` à `:383`), s'ils sont corrigés, reçoivent
chacun leur mutation ; sinon l'étape 0 dit pourquoi ils restent justes (ligne lue). Une mutation qui ne rougit pas sous
l'instant prescrit se dit (juste, test muet, ou mutant équivalent constaté), jamais à taire.

## Tâches

1. **Étape 0** : relire `e36-choix-autonomes-e360.md` (C-e360-13) et `e36-choix-autonomes-36-7.md` ; rejouer la sonde du
   détecteur de libfaketime (D3) ; rejouer les relevés A2027, N2027 et les sondes S1a, S1b, S2 à la tête (`rouges.log`) ;
   confirmer l'appel fautif d'`open_items_invariant` sous `RUST_BACKTRACE=1` (D5) ; relever par `/usr/bin/grep -nE
   '\b2026\b'` toutes les dates de 2026 des treize tests (D1 a) ; **relever la suite entière à la base sous
   `2029-06-15T10:00:00Z`** (`inventaire-36-7.sh`, journal `kesh-gate-logs/e36-7/inventaire-C2029.log`), référence de
   l'AC2 pour juger un rouge hors des treize (V5-2, V6-3).
2. Corriger les montages des sept fichiers du tableau (D1).
3. Écrire la spec de D6 et l'entrée de la garde ; mettre à jour `docs/testing.md:119` (AC8, dont « trois sites » → quatre) ; recettes de D3/D4.
4. Mutations M1a à M10 (AC3 c), le correctif commité ; puis les mutations de `bank_statements_manual_e2e:1008` et
   `migrations_upgrade_path:1332` et de `opening_complement_repository:533` sous `2029-06-15T10:00:00Z` (§ *Tests*, V6-1,
   V7-2).
5. Gates (§ *Dev Notes*), AC1, AC2, AC3, AC6 → `Status: review`, clé `review`, Dev Agent Record, prompt de la revue P1.
   **La fusion doit précéder le 2026-12-26** (D5).
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
`opening_complement_repository.rs`), `kesh-report` (tests, un : `open_items_invariant.rs`), `frontend` (tests : la spec
neuve `tests/e2e/horloge-sonde.spec.ts`, l'entrée de `src/lib/shared/utils/local-date-guard.test.ts` ; D6), `docs`
(`testing.md`) : **cinq modules** — à la limite de la règle, non au-delà —, sept fichiers de tests corrigés, une spec
neuve, une garde, aucun fichier de production.

### Gates (voie 2 ; `e35-dev-commun.md` § *Quels gates, quand*)

- **`kesh-db` est touché** : gate complet en cours de boucle (`scripts/test-fast.sh --ci`, base `kesh_e367a` remise à
  neuf), non ciblé. ⚠️ Ne pas toucher aux sites positionnels de `migrations_upgrade_path.rs` (`apply_migrations_up_to`,
  `migrations.len()`, P6) : seules les dates changent.
- **Passage en `review`** : `test-fast --ci` à l'heure réelle, puis sous `2027-01-04T10:00:00Z`, `2026-12-31T23:30:00Z` et
  `2026-03-10T23:30:00Z` (une campagne par prise de verrou), et la **suite entière** sous `2029-06-15T10:00:00Z` (AC2 ;
  conduite à tenir devant un rouge hors des treize : AC2) ;
  `scripts/gate-ci.sh kesh_ci_e367a` en voie 1 ; frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`,
  `build`), la garde des specs étant touchée (D6).
- **Clôture** : `gate-ci` et E2E complet à l'heure réelle au dernier commit de code, **puis** AC4 et AC5.
- Une commande lourde à la fois (E7). Verrou voie 2 : `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock
  scripts/mem-guard.sh …`, `TMPDIR=/tmp/kesh-voie2`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e367a`.
- faketime et cargo : construire **avant**, à l'heure réelle (`cargo nextest run --no-run`), puis lancer sous faketime.

### Fichiers communs

- `frontend/src/lib/shared/utils/local-date-guard.test.ts` : touché par toute story qui ajoute une lecture de l'horloge
  dans les specs (liste close).
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
- **2026-11-15 — Remédiation de la validation P2** (Opus 5.5 : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 5 LOW ;
  `kesh-gate-logs/e36-7/validation-36-7a-p2.md`). Prémisse de chaque finding lue au code ou exécutée ; aucun réfuté.
  Sondes neuves, à la tête, voie 2 (`kesh-gate-logs/e36-7/sondes-p2-36-7a.sh`) : S1a `binary(open_items_invariant)` sous
  `2026-12-24T10:00:00Z` 1 passé ; S1b sous `2026-12-26T10:00:00Z` 1 échoué ; S2 les sept binaires sous
  `2029-06-15T10:00:00Z` 216 exécutés, 13 échecs (les treize, aucun autre) ; S3, S3b détecteur de libfaketime.
  - **V2-1, corrigé** : instant `2029-06-15T10:00:00Z` ajouté à l'AC2 sur les sept binaires ; mutations scindées en parts
    « exercice » (tuées sous 2027-01-04) et « pièces en N-1 » (tuées sous 2029-06-15) ; sites de 2026 en dur nommés
    (`bank_statements_manual_e2e:253`, `:1008`, `:1036` ; `opening_complement_repository:519`, `:533` ;
    `migrations_upgrade_path:1319`, `:1332`, `:1498`, `:1716` ; `journal_entries_modification:373` … `:612`) et D1 (a) qui
    étend la règle à toutes les dates de 2026 des tests corrigés ; exercice borné 2026→N écarté.
  - **V2-2, corrigé** : D1 récrite en deux formes, (a) exercice N-1..N et dates en N-1, (b) verrou glissant, `jours_avant(k)`,
    0 < k < 300 ; l'interdit ne vise plus que le décalage qui sort de l'exercice ; D5 établie au code
    (`lettering_documents.rs:51`, `:552`, `:556-562` ; `open_items_invariant.rs:216`, `:220`) — la suggestion de la P1
    (V1-8, « un autre lettrage, à `jours_avant(n)` ») était fausse.
  - **V2-3, corrigé** : `open_items_invariant` rougit à l'heure réelle dès le 2026-12-26 (S1a, S1b) ; en-tête et tâche 5 :
    fusion avant le 2026-12-26. Le fichier d'epic et `sprint-status.yaml` sont corrigés par l'orchestrateur.
  - **V2-4, corrigé** : `societe()` monte cinq tests (`:372`, `:424`, `:486`, `:538`, `:592`) ; exercice N-1..N, exercice
    postérieur en N+1 nommé `format!("Exercice {}", N + 1)` (`:425`, `:539`, assertions `:458`, `:571`) ; M7a, M7b, M7c.
  - **V2-5, corrigé** : libfaketime est installée (`/usr/bin/faketime`, `/usr/lib/x86_64-linux-gnu/faketime/`) ; détecteur
    `ldconfig` remplacé par `ls -l` et une sonde positive (`sonde-p2-S3b.log` : `2027-01-04T10:00:00Z` sous faketime, date
    réelle sans).
  - **V2-6, corrigé** : troisième consommateur de `v0174a_decor` (`:5152-5156`) et sites de `migrations_upgrade_path`
    (`:1413-1415`, `:1537`, `:1661`, `:1701`, `:1742`) nommés.
  - **V2-7, corrigé** : `bank_statements_manual_e2e:1149` est l'annulation du rapprochement (`:1141`).
  - **V2-8, corrigé** : étage serveur cité à `routes/letterings.rs:311` et `:524-530`, sur la créance 1100 (compte
    lettrable ; refus `:513-517`).
  - **V2-9, corrigé** : commande écrite (`flock` et `mem-guard` dehors, `env TZ=… NO_FAKE_STAT=1 faketime
    --exclude-monotonic -f '@…'` dedans), chaque option citée de `man faketime` ; binaire construit ; `-m` non tranché par la
    documentation, variante dite au journal ; étage navigateur par une spec versionnée, D6 (C-36-7-8), qui ajoute le module
    `frontend` (cinq modules).
  - **V2-10, corrigé** : mutations « à la forme d'origine », `"2027-01-01"` pour les témoins.
  - Propagation grepée (`absente`, `ldconfig`, `FAKETIME_DONT`, `M1 à M10`, `quatre modules`, `un montage pour deux`,
    `forme de 2026`, `Non établi`, `cause à établir`, `Sans libfaketime`) : résidus traités ; les mentions restantes sont
    citées comme historique.
- **2026-11-15 — Remédiation de la validation P3** (Sonnet 5.5 : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW ;
  `kesh-gate-logs/e36-7/validation-36-7a-p3.md`). Remédiation commencée avant un arrêt de session et terminée après
  (`git diff` relu à la reprise). Prémisses lues au code ; aucun finding réfuté.
  - **V3-1, corrigé** : `opening_complement_repository:533` est le paramètre `today` de `complete` (`:163-175`), non une date
    de pièce ; D1 (a) le range parmi les opérations du jour (`aujourd_hui_test()`) ; M9b ne porte plus que `:519`, tuée par la
    panique de `post` (`:144`, `DateOutsideFiscalYear`, `journal_entries.rs:392`) ; la remise de `:533` est un mutant
    équivalent démontré (`opening_complement.rs:256-259`, `:663`).
  - **V3-2, corrigé** : `body()` (`:241-256`, 43 occurrences de `body(`) ne change pas ; le test construit un relevé dérivé
    en N-1 ; `:1036` (facture sans échéance, `A = F`) est tuée par la fenêtre de proposition (`reconciliation.rs:116-118`,
    `:402-408` ; assertion `:1098-1100`) ; `:1008` est équivalente ; l'affirmation « rougirait en 2028 » de la P2 est retirée.
  - **V3-3, corrigé** : M7c tuée parce que l'exercice 2027 devient **antérieur** (`find_later_closed_in_tx`,
    `journal_entries.rs:1548`, `:1896`) ; M8b scindée par site, `:1332` (INSERT brut) équivalente.
  - **V3-4, corrigé** : l'alternative d'une configuration Playwright hors dépôt a été **essayée** (`sonde-pw-list.log` :
    `@playwright/test` résolu dans `/usr/share/nodejs/playwright`, « two different versions », 0 test) ; spec versionnée
    gardée, raison écrite (D6, C-36-7-8 amendé).
  - **V3-5, corrigé** : les trois mentions du nombre de la garde (`local-date-guard.test.ts:28-29`, `:263`, `:318`) sont à
    mettre à jour (D6, AC9).
  - **V3-6, corrigé** : registre mis à jour (C-36-7-2 « Retenu », C-36-7-6, C-36-7-7) ; ligne de `sprint-status.yaml` de la
    branche : échéance de fusion au 2026-12-26 ; le fichier d'epic et la branche d'epic restent à l'orchestrateur.
  - **V3-7, corrigé** : l'AC2 joue la **suite entière** sous `2029-06-15T10:00:00Z` ; la Story borne « et après » au
    2029-12-31.
  - **V3-8, corrigé** : `societe()` pose un **exercice unique** N-1..N (`find_open_covering_date`, `journal_entries.rs:2727-2735` ;
    `chk_fiscal_years_dates`, `20260404000001_initial_schema.sql:51`).
  - Périmètre : treize tests, sept fichiers corrigés, une spec neuve et la garde, cinq modules ; **dix-neuf mutations**
    (M1a, M1b, M2a à M2c, M3 à M6, M7a à M7c, M8a, M8b ×3, M9a, M9b, M10) et trois mutants équivalents déclarés (`:1008`,
    `:1332`, `:533`).
- **2026-11-15 — Remédiation de la validation P4 ciblée** (Opus 5.5 : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW ;
  `kesh-gate-logs/e36-7/validation-36-7a-p4-ciblee.md`). Prémisses lues au code ; aucun finding réfuté.
  - **V4-1, corrigé** : `setup()` de `bank_statements_manual_e2e` (`:218-220`, 21 appels) ne change pas ;
    `manual_statement_feeds_the_tile_and_k3` (`:1183`) aurait rougi sous 2029 (K3, `closing_checks.rs:461-474` ; assertion
    `:1220-1223`). Le test corrigé replace l'exercice par un `UPDATE` ; M2a porte sur cet `UPDATE`. **Inventaire des décors
    partagés du périmètre** (callers comptés par `/usr/bin/grep -n '<fonction>('`) : `admin_full_import_e2e::v0174a_decor`
    (`:4898`) — appelée à `:5031` (dans `v0174a_backup_without_link_origin`, `:5028`, elle-même appelée par les deux tests
    du tableau, `:5140`, `:5146`) et `:5156` (`full_import_round_trip_keeps_a_draft`, troisième consommateur, déjà nommé :
    il compare un avant et un après l'import et ne lit aucune année, `:5153-5178` lus ; ses pièces suivent par
    `v0174a_invoice`, donc restent dans l'exercice) ; `journal_entries_modification::societe()` (`:270`) — cinq appels
    (`:372`, `:424`, `:486`, `:538`, `:592`), les cinq traités par D1 (V3-8) ;
    `migrations_upgrade_path::draft_migration_preserves_a_populated_base` (`:1362`) — deux appels (`:1760`, `:1767`), les
    deux tests du tableau ; `opening_complement_repository::setup`/`year_span` (`:47`, `:51`) — non modifiés, seul
    l'argument de l'appel du test change ; `dates_bornes_e2e` — témoins propres à chaque cas de la macro `champ_corps!`
    (`:1007-1010`, …), aucun décor ; `open_items_invariant` — un seul test dans le fichier (`:97`) ; `bank_statements_manual_e2e`
    — `setup()` et `body()` non modifiés. Aucun autre décor partagé n'est modifié.
  - **V4-2, corrigé** : Dev Notes § *Gates* alignées sur l'AC2 (suite entière sous 2029) ; l'AC2 dit qu'un rouge hors des
    treize compte contre lui s'il vient de la story, sinon se nomme et donne lieu à une issue demandée à l'orchestrateur,
    avec la preuve qu'il rougit déjà à la base.
  - **V4-3, corrigé** : la mention « trois sites » de `docs/testing.md:119` entre à l'AC8 et à la tâche 3.
  - **V4-4, corrigé** : « INSERT brut, donc équivalent » retiré ; `:1332` équivalente par la raison lue
    (`supplier_invoices.rs:1584`, `:1718` ; `migrations_upgrade_path.rs:1716` ; migration `20261106000001`, `:50`, `:66`) ;
    l'équivalence de `:1008` n'est plus déclarée : l'étape 0 la démontre ou joue la mutation. Les mutants équivalents
    déclarés passent donc de trois à **deux** (`:1332`, `:533`), plus `:1008` à trancher à l'étape 0 (l'entrée P3 ci-dessus,
    datée, n'est pas réécrite).
  - **V4-5, corrigé** : M7c nomme le premier rouge attendu, l'attente (`:443-448`), avant le refus (`:452-461`) ; le journal
    tranche.
- **2026-11-15 — Remédiation de la validation P5 ciblée** (`kesh-gate-logs/e36-7/validation-36-7a-p5-ciblee.md` :
  0/0/1/3). Prémisses lues ; aucun finding réfuté.
  - **V5-1, corrigé** : la migration `20261106000001` classe par `invoice_date` (`:47`, `:63`, lus) ; l'équivalence de
    `migrations_upgrade_path:1332` déclarée à la P4 était fausse dans sa raison. Elle n'est plus déclarée : l'étape 0 joue la
    mutation, comme pour `:1008`, et ne l'écrit équivalente que si elle reste verte, raison constatée. Mutants équivalents
    déclarés : **un** (`opening_complement_repository:533`). Les entrées P3 et P4 ci-dessus, datées, ne sont pas réécrites.
  - **V5-2, corrigé** : la tâche 1 relève la suite entière à la base sous `2029-06-15T10:00:00Z`.
  - **V5-3, corrigé** : M7c cite `FIND_LATER_CLOSED_SQL`, `fiscal_years.rs:892` (lu).
  - **V5-4, corrigé** : mutations : **dix-neuf, plus jusqu'à deux jouées à l'étape 0** (`:1008`, `:1332`).
- **2026-11-15 — Remédiation de la validation P6 ciblée** (`kesh-gate-logs/e36-7/validation-36-7a-p6-ciblee.md` :
  0/0/1/3), par l'outil Edit. Aucun finding réfuté.
  - **V6-1, corrigé** : une mutation remet un site **corrigé** à sa forme d'origine ; à l'étape 0 rien n'est corrigé. Les
    mutations de `bank_statements_manual_e2e:1008` et `migrations_upgrade_path:1332` passent à la **tâche 4**, après le
    correctif commité (D1, tableau, tâches 1 et 4, registre). Décompte : **dix-neuf, plus jusqu'à deux jouées à la tâche 4**.
    L'entrée P5 ci-dessus (« à l'étape 0 »), datée, n'est pas réécrite.
  - **V6-2, corrigé** : `:1008` — la voie « équivalence par lecture » est retirée ; seule la mutation jouée fait preuve.
  - **V6-3, corrigé** : relevé de base sous 2029 nommé `kesh-gate-logs/e36-7a/inventaire-C2029.log` (tâche 1, AC2).
  - **V6-4, corrigé** : propagé à la 36-7b (D1 et Change Log).
- **2026-11-15 — Remédiation de la validation P7 ciblée** (`kesh-gate-logs/e36-7/validation-36-7a-p7-ciblee.md` :
  0/0/1/2), par l'outil Edit. Aucun finding réfuté.
  - **V7-1, corrigé** : M8b — « celles qui restent vertes une fois jouées à la tâche 4 se disent équivalentes, raison
    constatée » ; `:1716` traité comme `:1332`. Aucune autre formule d'équivalence par lecture hors des entrées datées.
  - **V7-2, corrigé** : `:533` jouée à la tâche 4, équivalente seulement au constat. Plus aucun mutant équivalent déclaré
    d'avance. Décompte : **dix-neuf, plus jusqu'à trois jouées à la tâche 4** (`:1008`, `:1332`, `:533`).
  - **V7-3, corrigé** : tous les journaux de la story dans `/home/gcorbaz/devel/kesh-gate-logs/e36-7/` (en-tête, AC1, AC2,
    tâche 1, § *Tests*). L'entrée V6-3 ci-dessus, datée, n'est pas réécrite.
