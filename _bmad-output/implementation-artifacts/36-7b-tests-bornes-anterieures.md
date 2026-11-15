# Story 36-7b : les tests à date de 2026 restent verts sous un instant antérieur (#1125, KF-060)

Status: ready-for-dev

*Fiche issue du découpage de la fiche `36-7-tests-desancres-2026.md` (`Status: split`), approuvé par l'orchestrateur le
2026-11-15 sur la règle de Guy du même jour (« éviter les stories trop longues ou trop complexes ») ; choix C-36-7-7.
Rédigée par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-7`, code lu à `0bf3c3b4b`. Release
**0.18.0**, phase 2. **Sans échéance de calendrier** : ces tests ne rougissent jamais à l'heure réelle. **Développée après
la fusion de la 36-7a** (`36-7a-tests-desancres-2027.md`), dont elle reprend les décisions communes. **Aucune migration**,
aucune ligne de code de production. Voie de base **2**, `gate-ci.sh` en voie 1. Clé courte des bases proposée : `e367b` (à
confirmer par l'orchestrateur). Choix : `e36-choix-autonomes-36-7.md` (C-36-7-3 amendé, C-36-7-7). Relevés de la fiche :
`/home/gcorbaz/devel/kesh-gate-logs/e36-7/` ; journaux de la story : `kesh-gate-logs/e36-7b/`.*

## Story

En tant que mainteneur de Kesh, je veux que les tests qui datent leurs pièces en 2026 restent verts sous tout instant de
test de 2026 à 2029, y compris antérieur à ces dates, afin que les campagnes d'instants (`KESH_TEST_INSTANT`) n'aient plus
de rouge attendu et que #1125 puisse se fermer.

## Ce qui se lit au code (`0bf3c3b4b`)

### La plomberie

Celle de la 36-7a, § *La plomberie posée par les 35-5a0, 35-5a et 35-5c* : `KESH_TEST_INSTANT`
(`crates/kesh-core/src/horloge.rs:23`), `instant_de_test` (`:73`, panique `:80`), `aujourd_hui_test()`
(`crates/kesh-api/tests/common/jour.rs:14-15`, `crates/kesh-db/tests/support/jour.rs:17-22`). `docs/testing.md:119` porte
l'exception (a) : « les tests à **bornes fixes de 2026** (#1125, dont deux d'`admin_full_import_e2e`, bornes `2026-03-31`
et `2026-06-30`) rougissent sous un instant antérieur à leur borne ».

### Les neuf rouges sous un instant antérieur — inventaire par exécution, sans correctif

Relevé de la remédiation de la validation P1 de la fiche d'origine (V1-1) : la fiche d'origine ne portait que les trois
sites des commentaires de #1125, pris de journaux **filtrés** (`e355c/nuit-B2.log`, huit binaires ; `e355b/nuit-B3.log`,
un). Suite entière, à la tête, voie 2, base remise à neuf :

```sh
/home/gcorbaz/devel/kesh-gate-logs/e36-7/inventaire-36-7.sh B2026 2026-03-10T23:30:00Z
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

Aucun de ces neuf ne rougit sous `2027-01-04T10:00:00Z` ni sous `2026-12-31T23:30:00Z` (`inventaire-A2027.log`,
`inventaire-N2027.log`), ni à l'heure réelle : leur date de 2026 est passée dès qu'on y est. **Sonde positive** du relevé :
les neuf rougissent sous l'instant antérieur et sont verts à l'heure réelle (gate de la 0.17.8, `kesh-gate-logs/rel0178/`).

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

**Rouge de charge exclu** : `mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris` (journaux de la 35-5a
`e355a/nuit-C2.log`, `nuit-C2b.log`, `nuit-C3.log` ; absent des trois relevés de la fiche d'origine) : test à marges de
temps, sans date de 2026, hors périmètre ; s'il rougit, il se nomme comme rouge de charge et se relance.

## Décisions

- **D1 — Les dates passées se placent selon la règle D1 de la 36-7a** (récrite à sa validation P2 : forme (a), exercice
  couvrant N-1 et N et dates passées en N-1, pour les sites contraints par un exercice ou par une borne de verrou **fixe**
  posée par le test — `invoice_echeancier_e2e:2018`, `reconciliation_e2e:7283`, `supplier_settlement_cancel_e2e:809`,
  `admin_full_import_e2e:2007`, `:2281`, `supplier_invoices_draft:1778` ; forme (b), `jours_avant(k)`, réservée à un verrou
  **glissant** de la fixture, qu'aucun des neuf ne relève à la lecture de son site — l'étape 0 le vérifie, ligne lue).
  Toutes les dates de 2026 des neuf tests suivent, relevées par `/usr/bin/grep -nE '\b2026\b'`. Sous l'instant de mars,
  N-1 = 2025 : une mutation qui remet 2026 y est distinguable. Pour les tests dont le seul défaut est
  de lire à l'`asOf` du jour une pièce de date fixe (`project_reports_e2e`, `services_rollout`,
  `supplier_invoice_draft_e2e`), l'alternative est un `asOf` (ou une borne de rapport) **explicite** dans le montage ; le
  développeur choisit, ligne lue, celle qui garde ce que le test vérifie, et le dit au Change Log.
- **D2 — Périmètre : les neuf tests du tableau** (C-36-7-3 amendé, C-36-7-7). *Écartées* : les trois seuls de #1125
  (inventaire à trous, V1-1) ; retirer l'instant antérieur des critères (l'heure réelle ne recule pas) — les campagnes
  d'instants des 35-5a à 35-5c le jouent, et ces neuf rouges y resteraient attendus.
- **D3 — Pas d'E2E** : la story ne touche ni code visible ni route, et aucune spec ne lit `KESH_TEST_INSTANT`
  (`api-fixtures.ts:393`) ; l'E2E en 2027 est l'AC5 de la 36-7a. L'E2E complet à l'heure réelle se joue à la clôture selon
  la règle commune.
- Les décisions D3 (faketime), D4 (aucun outillage versionné) et D5 de la 36-7a ne s'appliquent pas ici.

## Critères d'acceptation

1. **AC1 — nextest vert sous l'instant antérieur.** `scripts/test-fast.sh --ci`, sous `KESH_TEST_INSTANT=2026-03-10T23:30:00Z`,
   voie 2, base remise à neuf : 0 échec (journal `kesh-gate-logs/e36-7b/gate-B2026.log`, commit cité). Les neuf tests du
   tableau sont verts.
2. **AC2 — Sans régression** (la 36-7a fusionnée) : la même suite verte sous `2027-01-04T10:00:00Z`, sous
   `2026-12-31T23:30:00Z` et sans la variable (heure réelle) ; les **huit binaires corrigés** (filtre de l'AC3 b) verts sous
   `2029-06-15T10:00:00Z` (N-1 = 2028, instant ajouté à la validation P2 de la 36-7a, V2-1). **Borne** : un instant antérieur au 2026-01-01 ou postérieur
   au 2029-12-31 n'est pas visé (exercice de la base de dev 2020-2030, garde `y <= 2029` de `supplier_invoices_draft.rs:1525`).
   Un rouge de charge se relance et se nomme, sans compter contre l'AC.
3. **AC3 — Sonde positive de la lecture.** (a) Le relevé `inventaire-B2026.log` (9 rouges) est rejoué à l'étape 0 à la tête
   de la branche et joint ; (b) après le correctif, les **huit binaires corrigés** (`cargo nextest run -E
   'binary(admin_full_import_e2e) | binary(invoice_echeancier_e2e) | binary(reconciliation_e2e) |
   binary(supplier_settlement_cancel_e2e) | binary(project_reports_e2e) | binary(services_rollout) |
   binary(supplier_invoice_draft_e2e) | binary(supplier_invoices_draft)'`) rougissent sous `KESH_TEST_INSTANT=pas-un-instant`
   (panique d'`instant_de_test`) ; (c) les mutations M1 à M9 de § *Tests*, chacune tuée par le test nommé.
4. **AC4 — Aucune ligne de production ne bouge** : `git diff --stat origin/epic/36-0.18.0...HEAD -- 'crates/*/src/*'
   'crates/*/src/**' frontend/src` vide.
5. **AC5 — La documentation dit vrai** : `docs/testing.md:119` n'annonce plus l'exception (a) « tests à bornes fixes de
   2026 » (que la 36-7a aura réécrite pour nommer ces neuf tests).
6. **AC6 — Garde intacte** : `date_du_jour_garde.rs` verte sans retouche de ses listes closes.

## Tests à écrire, et la preuve du rouge

La story n'ajoute pas de test : elle **corrige les montages** de neuf tests, dans huit fichiers. Le rouge de chacun est
constaté, sans correctif, par `inventaire-B2026.log` — à rejouer à l'étape 0 à la tête de la branche, journal
`kesh-gate-logs/e36-7b/rouges.log`.

**Mutations** (script versionné dans `kesh-gate-logs/e36-7b/mut/`, code commité avant, restauration
`git restore --worktree -- <fichier>` puis `touch`) : une par site corrigé (ou par groupe de sites d'un même montage, dit
comme tel), qui remet la date du site à sa forme de 2026 ; chacune **tuée par le test nommé sous
`KESH_TEST_INSTANT=2026-03-10T23:30:00Z`** — et non sous l'instant de 2027, où la forme de 2026 est passée et ne rougit pas
(validation P1 de la fiche d'origine, V1-3). Journal `mut/M<n>.log`.

| M | Site corrigé | Test qui la tue |
|---|---|---|
| M1 | `admin_full_import_e2e:2007` (borne) | `full_import_letters_a_reopened_historical_piece` |
| M2 | `admin_full_import_e2e:2281` (borne) | `validating_a_backdated_invoice_is_refused_by_the_period_lock` |
| M3 | `invoice_echeancier_e2e:2018` | `settlement_cancel_blocked_by_closed_lettering` |
| M4 | `reconciliation_e2e:7283` | `unreconcile_of_a_closed_lettering_is_refused_in_its_family` |
| M5 | `supplier_settlement_cancel_e2e:809` | `cancel_blocked_by_closed_lettering` |
| M6 | `project_reports_e2e:131` | `project_expenses_cumulative_mode` |
| M7 | `services_rollout:207` | `ac11_a_les_services_lisent_en_lecture_seule` |
| M8 | `supplier_invoice_draft_e2e:1853` (et `:1800`, `:1843`) — un groupe | `the_open_items_document_carries_both_numbers` |
| M9 | `supplier_invoices_draft:1778` | `cancelled_invoice_returns_to_draft` |

Si D1 retient un `asOf` explicite plutôt qu'une date déplacée, la mutation porte sur cet `asOf` (retiré) : elle doit rougir
de même. Une mutation qui ne rougit pas se dit (juste, ou test muet).

## Tâches

1. **Étape 0** : partir de la 36-7a fusionnée ; relire `e36-choix-autonomes-36-7.md` ; rejouer le relevé B2026 à la tête
   (`rouges.log`) ; pour chaque site, lire ce que le test vérifie et choisir date déplacée ou `asOf` explicite (D1), ligne
   citée au Change Log.
2. Corriger les montages des huit fichiers (D1).
3. Retirer l'exception (a) de `docs/testing.md:119` (AC5).
4. Mutations M1 à M9 (AC3 c).
5. Gates (§ *Dev Notes*), AC1 à AC3 → `Status: review`, clé `review`, Dev Agent Record, prompt de la revue P1.
6. À la clôture : `gate-ci` et E2E complet à l'heure réelle au dernier commit de code ; puis `done`.

## Note de version

*Modifié* — Les tests de Kesh qui datent leurs pièces en 2026 restent verts sous toute date de test, y compris antérieure.
Aucun changement pour l'utilisateur. (refs #1125)

## Dev Notes

### Hors périmètre

- Les treize rouges de 2027, l'E2E et nextest sous horloge de processus décalée : **36-7a**.
- Les issues demandées par la fiche d'origine (aperçu de numérotation figé en 2026, exercice de test 2020-2030) : portées
  par la 36-7a.

### Modules touchés (règle de splitting)

`kesh-api` (tests, sept fichiers : `admin_full_import_e2e.rs`, `invoice_echeancier_e2e.rs`, `reconciliation_e2e.rs`,
`supplier_settlement_cancel_e2e.rs`, `project_reports_e2e.rs`, `services_rollout.rs`, `supplier_invoice_draft_e2e.rs`),
`kesh-db` (tests, un : `supplier_invoices_draft.rs`), `docs` (`testing.md`) : **trois modules**, huit fichiers de tests,
aucun fichier de production.

### Gates (voie 2 ; `e35-dev-commun.md` § *Quels gates, quand*)

- **`kesh-db` est touché** (`supplier_invoices_draft.rs`) : gate complet en cours de boucle (`scripts/test-fast.sh --ci`,
  base `kesh_e367b` remise à neuf), non ciblé.
- **Passage en `review`** : `test-fast --ci` à l'heure réelle, puis sous `2026-03-10T23:30:00Z`, `2027-01-04T10:00:00Z`
  et `2026-12-31T23:30:00Z` (une campagne par prise de verrou) ; `scripts/gate-ci.sh kesh_ci_e367b` en voie 1.
- **Clôture** : `gate-ci` et E2E complet à l'heure réelle au dernier commit de code.
- Une commande lourde à la fois (E7). Verrou voie 2 : `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock
  scripts/mem-guard.sh …`, `TMPDIR=/tmp/kesh-voie2`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e367b`.

### Fichiers communs

- `admin_full_import_e2e.rs` et `docs/testing.md` : **communs avec la 36-7a**, d'où le départ de la 36-7a fusionnée.
- `invoice_echeancier_e2e.rs`, `supplier_settlement_cancel_e2e.rs` : à surveiller avec 36-9 (#1044, solde du reste d'un
  règlement) ; `reconciliation_e2e.rs`, `admin_full_import_e2e.rs` : possibles avec les lots 36-11 (0.19.0).
- Aucun manuel.

### Ce qui doit rester vrai

- Les tests corrigés vérifient toujours ce qu'ils vérifiaient (borne passée, verrou, lecture du jour).
- Verts à l'heure réelle et sous tout instant du 2026-01-01 au 2029-12-31.
- Aucune donnée réelle dans les fixtures.

### Références

Fiche d'origine `36-7-tests-desancres-2026.md` (`split`) et sa validation P1 (`kesh-gate-logs/e36-7/validation-p1.md`) ;
fiche sœur `36-7a-tests-desancres-2027.md` ; issue #1125 et ses commentaires des 35-5b et 35-5c ; `docs/testing.md:119`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-11-15 — Création par découpage** (Opus 5.5) de la fiche `36-7-tests-desancres-2026.md`, après sa validation P1
  (`kesh-gate-logs/e36-7/validation-p1.md`) et sa remédiation (commit `afe0a4f2e`), découpage approuvé par l'orchestrateur
  le même jour (C-36-7-7). Reprend les neuf tests de D2 d'origine, l'instant de mars de l'AC2 d'origine (devenu AC1), les
  mutations M2, M3, M13 à M19 d'origine renumérotées M1 à M9 ; renvoie à la 36-7a pour la règle D1. Aucune passe de
  validation propre encore.
- **2026-11-15 — Propagation de la remédiation P2 de la 36-7a** (`kesh-gate-logs/e36-7/validation-36-7a-p2.md`) : D1
  renvoie à la règle récrite (formes a et b) et nomme la forme de chacun des neuf sites ; AC2 gagne l'instant
  `2029-06-15T10:00:00Z` sur les huit binaires. Le reste inchangé.
