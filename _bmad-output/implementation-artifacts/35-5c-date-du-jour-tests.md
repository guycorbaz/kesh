# Story 35-5c : les tests ne dépendent plus de l'heure à laquelle on les lance — tests Rust et specs E2E au jour suisse, gardes fermées (#929)

Status: review

*Fiche rédigée le 2026-11-12 par l'agent de rédaction des fiches 35-5a et 35-5b (Opus 5.5), worktree `kesh-e355`, à la
remédiation de la validation P1 de la 35-5b (V1-8 : la story ne tenait pas en une boucle). Troisième des trois stories du
découpage de la 35-5 (C-e350-14, C-e350-22, C-e350-28) : elle **part de la 35-5b fusionnée** (donc de la 35-5a0 et de la
35-5a), dont elle emploie `jour_suisse`, `HorlogeFixe`, `horloge_de_test()`, `AppState.horloge`, `Lecture` à horloge, le helper
`aujourd_hui_test()` (C-e350-29) et la garde `date_du_jour_garde.rs`. Release **0.17.7**. **Aucune migration**. **Aucun code
de production** : tests, specs E2E, gardes. Boucle de revue : les fichiers de tests de `kesh-db` sont touchés, donc **gate
complet à chaque commit** (`CLAUDE.md`, § *Exception `kesh-db`*). Code lu sur **`release/0.17.6`** (`4e1c768a`, identique à
`8424e36e` pour `crates/` et `frontend/`). Registres : `e35-choix-autonomes-e350.md` (C-e350-14, -21, -22, -24, -26, -28, -29) ;
`e35-choix-autonomes-e355c.md` pour la story, à créer au premier choix.*

## Story

En tant que mainteneur, je veux que les tests Rust et les specs E2E prennent la date du jour comme le serveur — le jour
suisse — ou une date fixée, afin qu'aucun ne rougisse selon l'heure à laquelle on lance le gate ou la CI (entre minuit et
1 h, 2 h en été, à Zurich), et que rien ne réintroduise une date du jour UTC dans un test.

## Ce qui se lit au code (`release/0.17.6`, à recompter sur la 35-5b fusionnée)

- **Modules de tests de `src`** : `kesh-db/src/repositories/journal_entries.rs`, `mod tests` à partir de `:3178` — 50 sites
  (`:3219` à `:6430`), moins ceux que la 35-5a a convertis (dates de contre-passation assertées). Les restants sont des
  **jours de montage** (sous plusieurs formes : `let today`, `let jour` `:3849`, `:3971`, `:4203`, `:4266`, le `.bind(…)` d'une
  borne de verrou `:4384`, un `.year()` `:4457`) : aucun verdict à lire un par un, sauf pour un test qui asserte une date
  produite par le code. **Pourquoi ils sont sûrs la nuit** (validation P2, V2-1), et non par `KESH_TEST_INSTANT`, que ce
  binaire (`lib` de `kesh-db`) ne lit pas : `kesh-db` ne lit plus l'horloge (ses fonctions reçoivent la date, de la 35-5a0 à
  la 35-5b), et, **une fois la course ci-dessous fermée**, chaque test fabrique son exercice et ses pièces depuis un seul jour —
  il est alors cohérent avec lui-même à toute heure. **La course**, non une faute de nuit : `setup()` lit l'horloge (`:3219`,
  `ensure_open_fiscal_year(pool, company_id, today)`, exercice « couvrant aujourd'hui »), puis **chaque** test relit sa propre
  date (ex. `test_create_writes_audit_log`, `:3324`) ; les **51** appels de `setup(&pool)` sont concernés, et `lettrer_dans`
  aussi (`:4266`) ; une exécution qui chevauche minuit le 31 décembre crée l'exercice d'une année et une pièce de la suivante.
  **Fermée ici** (D1) : `setup()` rend le jour qu'elle a lu (sa signature `(i64, i64, i64)` s'étend), et les 51 tests et
  `lettrer_dans` le réemploient au lieu de relire l'horloge (V3-2 de la validation P3). Les 6 de `invoices.rs` sont à la 35-5b. Aucun autre module de tests de
  `crates/*/src` ne porte de forme.
- **`crates/*/tests`** : les fichiers marqués **35-5c** au verdict de la fiche 35-5a (après sa validation P2) —
  `kesh-api/tests/accounts_e2e.rs` (1 + 1 coupé, `:243-245`), `admin_full_import_e2e.rs` (4 ; son appel de `:1809` est
  changé par la 35-5a), `credit_card_e2e.rs` (3), `invoice_delete_e2e.rs` (2), `invoice_unvalidate_e2e.rs` (6),
  `mcp_enveloppe_e2e.rs` (1), `reconciliation_link_e2e.rs` (1) — et tout reste qu'une story précédente aurait laissé, à son
  compte exact dans la liste de la garde.
- **Specs E2E, l'ensemble clos** (comptes sur `release/0.17.6` ; le compte d'entrée se relève sur la 35-5b fusionnée, la
  35-5a ayant converti ses specs — au moins `invoices-settlement-cancel.spec.ts`, `journal-entries.spec.ts`,
  `invoice-write-off.spec.ts`, `supplier-invoice-draft-cycle.spec.ts`, liste au Change Log de la 35-5a, dont la 35-5c part) :
  `new Date()` **sans argument** paraît **41** fois dans `frontend/tests/e2e/**` (`git grep -nE 'new Date\(\s*\)' --
  frontend/tests/e2e`), toutes pour un jour du calendrier : **41 = 27** `new Date().toISOString().slice(0, 10)` **+ 7**
  `toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' })` (dont `todayInZurich`, `api-fixtures.ts:391`) **+ 7** formes Node
  (tableau ci-dessous). `toISOString().slice(0, 10)` paraît **34** fois dans 26 fichiers, **34 = 27 + 4** (récepteur : une
  variable née d'un `new Date()`, `api-fixtures.ts:64`, `invoices_echeancier.spec.ts:97`, `reports.spec.ts:262`,
  `v0174-correctifs.spec.ts:53`) **+ 3** (un jour donné, `bank-manual-statement.spec.ts:74`, `:314`,
  `reconciliation-show-older.spec.ts:80`), plus **1** coupé sur trois lignes (`invoices.spec.ts:526-528`), soit **35**. Les
  formes Node locales ou mixtes (fuseau du processus Node, non le `timezoneId: 'Europe/Zurich'` du navigateur,
  `playwright.config.ts:59`) :

  | Site | Forme | Horloge lue | Source retenue |
  |---|---|---|---|
  | `bank-accounts-closed.spec.ts:144-146` | `getFullYear`/`getMonth`/`getDate` sur `new Date()` (Node) | oui | `todayInZurich()` |
  | `helpers/api-fixtures.ts:62-64` (`overdueDate`) | `setDate` local puis `toISOString` | oui | `ajouterJoursIso(todayInZurich(), −n)` |
  | `invoices.spec.ts:523-524` | `getFullYear`/`getMonth`/`getDate` sur `now` (Node, « date du poste ») | oui | `todayInZurich()` |
  | `invoices_echeancier.spec.ts:95-97` (`daysFromToday`) | `setDate` local puis `toISOString` | oui | `ajouterJoursIso(todayInZurich(), n)` |
  | `reports.spec.ts:260-262` | `setDate(getDate() − 40)` local puis `toISOString` | oui | `ajouterJoursIso(todayInZurich(), −40)` |
  | `v0174-correctifs.spec.ts:51-53` (`daysFromToday`) | `setUTCDate` puis `toISOString` (UTC) | oui | `ajouterJoursIso(todayInZurich(), n)` |
  | `supplier-invoice-draft-cycle.spec.ts:685-688` | `getFullYear`… dans `page.evaluate` (navigateur, Zurich par `timezoneId` : juste) | oui | `todayInZurich()` (même valeur, sans exception à la garde) |

  **Arithmétique sur un jour déjà donné** (horloge non lue) : `reconciliation-show-older.spec.ts:80` (`Date.UTC(2025, 0,
  1 + i)`), `bank-manual-statement.spec.ts:72-74`, `:312-314` (sur `T12:00:00Z`), `invoices.spec.ts:526-528` (depuis `today`
  déjà pris) → `ajouterJoursIso`. Hors motif et hors sujet : les `Date.UTC(y, m, 0).getUTCDate()` (`credit-card-statement.spec.ts:138`,
  `api-fixtures.ts:358`, `homepage-charts.spec.ts:115`), qui ne produisent pas de jour du calendrier, et `invoices.spec.ts:265`
  (`Date.now()` − 1 h, un instant).
  **Autres lectures déjà justes du jour suisse**, à ramener à `todayInZurich()` (une ligne chacune) : `bank-manual-statement.spec.ts:66`
  (`todayZurich`), `open-items.spec.ts:39`, et `toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' })` en ligne dans
  `bank-account-opening-balance.spec.ts:71`, `bank-accounts-closed.spec.ts:220`, `api-fixtures.ts:199`, `journal-entries.spec.ts:362`.
  **Le jour UTC voulu** : `audit-log.spec.ts:56-59` (`todayUtc()`, « la route filtre sur des jours UTC ») — le filtre du
  journal d'audit compare `created_at` (UTC, #669) à la date à minuit (`kesh-db audit_log.rs:224-238`), comme le dit le manuel
  (`user-manual.tex:3955-3958`) ; converti en Zurich, ce test rougirait la nuit (V2-6 de la validation P2 de la 35-5b). Il
  **reste** au jour UTC.
- **Les helpers existants** : `todayInZurich()`, `frontend/tests/e2e/helpers/api-fixtures.ts:390-392` ; `addDaysIso` existe
  dans `src` (`frontend/src/lib/features/invoices/invoice-helpers.ts:120-124`, corps `toISOString().slice(0, 10)`, entrée
  « site 4 » de l'allowlist `local-date-guard.test.ts:45-47`), mais aucune spec n'importe `src/` ni `$lib`.
  `frontend/vite.config.ts:59` (`include: ['src/**/*.test.ts', 'tests/**/*.test.ts']`) fait tourner les tests de
  `tests/e2e/helpers/*.test.ts` (convention : `contrast.test.ts`, `test-state.test.ts`).
- **La garde frontend** : `frontend/src/lib/shared/utils/local-date-guard.test.ts` relève `toISOString().slice(0, 10)` dans
  `src/**` hors tests, contre une allowlist `{ fichier, fragment, motif }` ; elle ne masque pas les commentaires.

## Décisions

- **D1 — Une date du jour, dans un test Rust, vient de trois sources** : une `HorlogeFixe` (tout test qui asserte une date
  produite par le serveur) ; une date écrite en dur ; ou `aujourd_hui_test()` (montage qui veut « un exercice qui couvre
  aujourd'hui ») — le helper de sa crate de tests (`crates/kesh-api/tests/common/jour.rs`, `crates/kesh-db/tests/support/jour.rs`,
  posés par la 35-5a0), ou, dans un `mod tests` de `src` qui ne peut pas les importer, `HorlogeFixe(Utc::now()).aujourd_hui()`
  en ligne (`kesh-db` dépend de `kesh-core`, `Cargo.toml:8`), **lu une fois par test** — `setup()` de `journal_entries.rs`
  rend le jour qu'elle a lu, et ses tests le réemploient (V2-1). Jamais `Utc::now().date_naive()` ni `Local::now()`. **Aucun
  `static`, `OnceLock`, `LazyLock`** qui mémorise un jour dans un helper de montage : la garde de la 35-5a0 (D5 de sa fiche)
  le refuse dans les fichiers de l'horloge et des helpers.
- **D2 — La garde Rust se ferme** : la liste d'exemptions de `date_du_jour_garde.rs` ne porte plus que les entrées
  permanentes (`kesh-core/src/horloge.rs` 1, `admin.rs` 1 pour `:722`) ; tous les autres fichiers de `src` et de
  `crates/*/tests` comptent 0 (égalité stricte, C-e350-24). Le fichier de la garde est exclu nommément (35-5a0) : ses sondes
  contiennent les formes.
- **D3 — Les specs E2E** prennent le jour par `todayInZurich()` (et rien d'autre : les lectures déjà justes s'y ramènent) ;
  l'arithmétique sur un jour passe par **un** helper neuf, **`ajouterJoursIso(iso, n)`**, dans
  `frontend/tests/e2e/helpers/dates.ts` — nommé autrement que l'`addDaysIso` de `src` (aucun homonyme), écrit en `Date.UTC`
  et accesseurs `getUTC*`, **sans `toISOString` ni `new Date()` sans argument**, testé dans `helpers/dates.test.ts`
  (`2026-02-28` + 1 → `2026-03-01`, `2024-02-28` + 1 → `2024-02-29`, `2026-03-29` + 1 → `2026-03-30`, `2026-01-01` − 1 →
  `2025-12-31`). `audit-log.spec.ts` garde son `todayUtc()`, motivé.
- **D4 — La garde frontend s'étend aux specs, par l'ensemble clos** : `local-date-guard.test.ts` parcourt aussi
  `tests/e2e/**` et relève **tout `new Date()` sans argument** (forme : `new Date\(\s*\)`), et **tout `new Date(Date.now`**
  (forme : `new Date\(\s*Date\.now`, V2-6), contre une allowlist `{ fichier, fragment, motif }` **close de trois entrées**,
  dont le **fragment est le texte exact de la ligne relevée** (l'auditeur apparie par `texte.includes(fragment)` sur la ligne,
  V2-3) :
  - `helpers/api-fixtures.ts`, `return new Date().toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' });` (`:391`,
    `todayInZurich` : « le jour suisse, seule lecture de l'horloge des specs ») ;
  - `audit-log.spec.ts`, `return new Date().toISOString().slice(0, 10);` (`:58`, `todayUtc` : « le filtre du journal compare des
    horodatages UTC, #669 ») ;
  - `invoices.spec.ts`, la ligne de `:265` (`new Date(Date.now() − 1 h)`, `.slice(0, 19)` : « un instant, non un jour »).

  Elle relève aussi `toISOString()` suivi de `.slice(0, 10)`, **même coupé sur plusieurs lignes**, sur tout récepteur (une date
  fabriquée depuis un instant), avec la même allowlist. **Un seul relevé par ligne**, quel que soit le nombre de motifs qu'elle
  porte (`audit-log.spec.ts:58` en porte deux) : sinon l'entrée en recevrait deux et l'auditeur rougirait ; une sonde le
  prouve. `Date.now()` seul n'est pas relevé (identifiants d'unicité, centaines d'emplois). **Angle mort assumé**, écrit au
  Change Log : un jour tiré de `Date.now()` sans `new Date(` (`Intl.DateTimeFormat('sv-SE').format(Date.now())`) — aucun
  aujourd'hui.

## Critères d'acceptation

1. **AC1 — Tests Rust** : les tests de la liste ci-dessus n'emploient plus que D1 ; la garde (D2) est verte avec les seules
   entrées permanentes ; rouge sur la 35-5b fusionnée (entrées 35-5c encore présentes, comptes différents).
2. **AC2 — Verdict par fichier** au Change Log : pour chaque fichier converti, la source retenue (D1) ligne lue citée ; un
   test qui asserte une date produite par le serveur fixe l'horloge, il ne prend pas `aujourd_hui_test()`.
3. **AC3 — Specs E2E** (D3) : les `new Date()` sans argument restants sur la 35-5b fusionnée (compte d'entrée de l'étape 0 ;
   41 sur `release/0.17.6`) tombent à **deux** (les deux premières entrées de l'allowlist ; la troisième est la ligne
   `new Date(Date.now` de `invoices.spec.ts:265`) ; plus aucun `toISOString().slice(0, 10)` hors elles ; la garde étendue (D4) rougit sur la 35-5b
   fusionnée et passe à la fin.
4. **AC4 — Sondes** au Change Log : un `Utc::now().date_naive()` ajouté à un test Rust rougit ; une forme coupée sur deux
   lignes aussi ; dans une spec, `new Date().toISOString().slice(0, 10)` rougit, coupé ou non ; `const d = new Date();
   d.getDate()` (jour par une variable) rougit ; `new Date(Date.now() + 1)` rougit ; une ligne à deux motifs ne donne qu'un
   relevé ; `ajouterJoursIso('2026-03-10', 1)` ne rougit pas.
5. **AC5 — La nuit, chiffrée** (C-e350-26) : les **binaires d'intégration** (`crates/*/tests`) que la story touche, lancés une
   fois avec `KESH_TEST_INSTANT=2025-12-31T23:30:00Z` puis `KESH_TEST_INSTANT=2026-03-10T23:30:00Z`, sont **verts** ; puis
   **tous** les binaires avec `KESH_TEST_INSTANT="$(date -u +%F)T23:30:00Z"`, de jour (la nuit réelle reproduite) : aucun rouge
   **dû au jour** ne doit rester, l'epic étant close sur ce point ; chaque rouge est nommé avec son motif (un horodatage MariaDB
   `NOW(3)`, heure réelle, n'est pas un défaut de nuit : V3-3) ; journal `kesh-gate-logs/e355c/nuit.log`. Le binaire `lib` de `kesh-db`
   (`mod tests` de `journal_entries.rs`) ne lit pas la variable : son vert n'y prouve rien, et sa sûreté de nuit tient à
   l'argument écrit plus haut (ligne lue citée au Change Log). Côté specs, la conversion à `todayInZurich()` / `ajouterJoursIso`
   est la preuve (le jour suisse ne dépend plus du fuseau du processus Node).
6. **AC6 — Rien ne change hors des tests** : `git diff -U0 <base> -- 'crates/*/src/*'` ne rend que des hunks commençant à une
   ligne au moins égale au `mod tests` de leur fichier (sa ligne **sur la base**, relevée à l'étape 0 : la 35-5a et la 35-5b
   déplacent celle de `journal_entries.rs`, `:3178` sur `release/0.17.6`) — `git diff --stat` ne distinguerait pas
   la production des tests d'un même fichier ; `frontend/src/**` n'est touché que par `local-date-guard.test.ts` ;
   `crates/kesh-db/migrations/` intact.
7. **AC7 — Gates et E2E** : gate complet, `scripts/gate-ci.sh`, frontend complet et E2E complet au dernier commit de code ;
   les échecs E2E nommés contre `docs/testing.md` § « Les échecs attendus ».
8. **AC8 — CHANGELOG, PR** : aucun point au CHANGELOG (aucun changement visible) ; la PR porte `refs #929`.

## Tests à écrire, et la preuve du rouge

AC1, AC3 : les gardes, rouges sur la 35-5b fusionnée ; `helpers/dates.test.ts` rouge avant `ajouterJoursIso`. Journal
`kesh-gate-logs/e355c/rouges.log`.

**Mutations** : M1 un test remis à `Utc::now().date_naive()` (garde Rust) ; M2 la même forme coupée sur deux lignes ; M3
`todayInZurich` remplacé par `toISOString().slice(0, 10)` dans une spec (garde frontend) ; M4 `ajouterJoursIso` qui part de
`new Date()` au lieu du jour reçu (garde frontend : relevé de `new Date()` sans argument ; et `dates.test.ts`) ; M5 un helper
de montage qui mémorise le jour dans un `OnceLock` — tuée par la **garde** de la 35-5a0 **dans les fichiers qu'elle nomme**
(`kesh-core/src/horloge.rs`, `kesh-api/src/horloge.rs`, `tests/common/jour.rs`, les deux `tests/support/jour.rs` ; un
`OnceLock` dans un autre helper de montage lui échappe : la règle de D1 s'y contrôle en revue), **non** par AC5 : chaque
exécution est un processus neuf, où la mémoire rend la même valeur que la version sans mémoire (validation P2, V2-2) ; M6 un
test de `journal_entries.rs` qui relit l'horloge au lieu du jour de `setup()` — non attrapable par un test (la course ne se
produit qu'à minuit) : contrôle par lecture, au Change Log, `/usr/bin/grep -nE 'Utc::now\(\)\s*\.(date_naive|naive_utc)'` sur le
`mod tests` ne rendant que la ligne de `setup()` (le motif nu `Utc::now` rend aussi `:5125`,
`timestamp_nanos_opt`, un suffixe d'unicité : V3-1).

## Tâches

- [x] Étape 0 : partir de la 35-5b fusionnée ; relever la liste d'exemptions restante de la garde Rust et la liste des specs
      restantes (sortie de la garde étendue sur la 35-5b fusionnée : compte d'entrée) ; confirmer les tableaux ci-dessus ;
      Change Log.
- [x] Conversion des tests Rust (D1) ; garde Rust fermée (D2) ; preuve de nuit (AC5).
- [x] Specs E2E (D3), `ajouterJoursIso` et son test, garde frontend étendue (D4).
- [x] Gates du passage en `review` (consignes de l'Epic 35, qui priment sur cette ligne) : `test-fast.sh --ci` et
      `scripts/gate-ci.sh` au dernier commit de code, frontend complet. **E2E complet : à la clôture** (non lancé ici).

## Dev Notes

### Fichiers communs — règle d'union C-e350-8

`kesh-db/src/repositories/journal_entries.rs` (`mod tests`) avec la **35-8**, qui part de la 35-5c fusionnée ;
`frontend/tests/e2e/helpers/api-fixtures.ts` avec la **35-2a** (paramètre `city`) : hunks distincts ;
`frontend/src/lib/shared/utils/local-date-guard.test.ts` avec la 35-5a (entrée du verrou retirée) : la 35-5c part d'elle ;
`crates/kesh-api/tests/date_du_jour_garde.rs` (liste d'exemptions : 35-5a0, 35-5a, 35-5b, puis ici, dans cet ordre).

### Signal de découpage

Rust (19 sites dans 7 fichiers, plus le reste de `journal_entries.rs`, mécanique) et specs (**29** fichiers au plus — ceux
des 41, plus `reconciliation-show-older.spec.ts`, moins `audit-log.spec.ts`, avant soustraction de la 35-5a —, plus `dates.ts`
et `dates.test.ts`, deux gardes) : quatre zones, toutes mécaniques ; tenable en une boucle (validations P1, P2). Un
découpage Rust / E2E reste possible s'il le fallait, sans dépendance entre les deux.

### Ce qui doit rester vrai

- Les tests gardent leur objet : une conversion ne change pas ce qu'un test prouve, seulement d'où vient sa date. Un test
  qui devient vide de sens après conversion se signale au Change Log.
- Fixtures synthétiques seulement.

### Références

- Issue #929 ; fiches 35-5a et 35-5b ; #889 et `local-date-guard.test.ts`.
- `CLAUDE.md` : § *Exception `kesh-db`* ; § *Inventorier les sites NON RÉSOLUS* ; § *Test Locally First* (`cargo test`
  contre `nextest`).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement, worktree `kesh-e355c`, branche `story/35-5c-date-du-jour-tests`, voie 2), le 2026-11-14.

### Debug Log References

Journaux hors dépôt, `/home/gcorbaz/devel/kesh-gate-logs/e355c/` (commit et environnement en tête de chacun) :
`rouges.log` (garde frontend), `rouges-rust.log` (garde Rust), `cible-1.log`, `verts-front-cible.log`, `nuit-B1.log`,
`nuit-B2.log`, `nuit-B3.log`, `nuit-J1.log`, `nuit-N1.log`, `nuit-S1.log`, `nuit-S2.log` et la synthèse `nuit.log`,
`mut/M1-M2-M5.log`, `mut/M3.log`, `mut/M4.log`, `mut/M6-lecture.log`, `mut/resume.log`, `ac6.log`, `gate-test-fast.log`,
`gate-ci.log` (+ `gate-ci.journal`), `front-check.log`, `front-lint-i18n-ownership.log`, `front-test-unit.log`, `front-build.log`.
Scripts : `reset-db-e355c.sh`, `test-fast-e355c.sh`, `nuit-e355c.sh`, `cible-e355c.sh`.

### Completion Notes List

- Fiche rédigée à la remédiation de la validation P1 de la 35-5b (C-e350-22) ; validation terminée en P3 ciblée.
- **Commits** : `40090cdd3` (code et tests, registre) — dernier commit de code ; le commit suivant ne porte que la fiche, le
  registre annoté, `sprint-status.yaml` et le prompt de la revue P1.
- **Rust (D1)** : 68 occurrences converties (périmètre `ebd5b654c..40090cdd3`, comptes de la garde sur la base,
  `rouges-rust.log`) — `journal_entries.rs` 50 (module de tests ; `setup()` lit le jour suisse une fois et le rend), et
  `kesh-api/tests` : `accounts_e2e.rs` 2, `admin_full_import_e2e.rs` 3, `credit_card_e2e.rs` 3, `invoice_delete_e2e.rs` 2,
  `invoice_unvalidate_e2e.rs` 6, `mcp_enveloppe_e2e.rs` 1, `reconciliation_link_e2e.rs` 1.
- **Garde Rust (D2)** : liste d'exemptions ramenée aux deux entrées permanentes ; sondes AC4 ajoutées dans le test existant
  `sonde_une_forme_ajoutee_rougit_entiere_ou_coupee` (7 tests dans le fichier, avant comme après).
- **Specs (D3)** : 47 relevés sans entrée sur la base (`rouges.log`) → 0 ; 29 fichiers de `tests/e2e` touchés (27 specs,
  `helpers/api-fixtures.ts`, `helpers/inbox.ts`) plus deux neufs (`helpers/dates.ts`, `helpers/dates.test.ts`, 6 tests).
- **Garde frontend (D4)** : 7 → 13 tests dans `local-date-guard.test.ts` (1 audit des specs, 5 sondes).
- **AC5** : nuit réelle reproduite sur tous les binaires **5372/5372** ; sous les deux instants de la fiche, **deux rouges** de
  classe (b) dans `admin_full_import_e2e` (bornes fixes de 2026 supposées passées) — écart déclaré, C-e355c-6.
  *(Annoté le 2026-11-14, revue P1, A3 : l'un des deux, `validating_a_backdated_invoice…`, naît de la conversion faite par
  cette story ; tous deux sont commentés sur #1125.)*
- **Non lancé** : l'E2E (Playwright), à la clôture selon les consignes de l'Epic 35 ; les specs converties ne sont donc vérifiées
  qu'au typage (`npm run check`) et par la garde.
- Aucun changement visible : ni capture, ni CHANGELOG (AC8), ni manuel.

### File List

- `crates/kesh-db/src/repositories/journal_entries.rs` (module de tests seulement)
- `crates/kesh-api/tests/accounts_e2e.rs`, `admin_full_import_e2e.rs`, `credit_card_e2e.rs`, `date_du_jour_garde.rs`,
  `invoice_delete_e2e.rs`, `invoice_unvalidate_e2e.rs`, `mcp_enveloppe_e2e.rs`, `reconciliation_link_e2e.rs`
- `frontend/src/lib/shared/utils/local-date-guard.test.ts`
- `frontend/tests/e2e/helpers/dates.ts` (neuf), `frontend/tests/e2e/helpers/dates.test.ts` (neuf),
  `frontend/tests/e2e/helpers/api-fixtures.ts`, `frontend/tests/e2e/helpers/inbox.ts`
- `frontend/tests/e2e/` : `bank-account-opening-balance`, `bank-accounts-closed`, `bank-manual-statement`, `credit-notes`,
  `date-filtre-frappe`, `invoice-frozen-pdf`, `invoice-minimum-amount`, `invoice-revenue-account`, `invoice-rounding-5-centimes`,
  `invoice-write-off`, `invoices-settle-unknown-outcome`, `invoices-settle-version`, `invoices-settlement-cancel`, `invoices`,
  `invoices_echeancier`, `journal-entries`, `open-items`, `payment-batches`, `reconciliation-show-older`, `reminders`, `reports`,
  `supplier-documents`, `supplier-internal-number`, `supplier-invoice-draft-cycle`, `supplier-invoice-unknown-outcome`,
  `supplier-invoices`, `v0174-correctifs` (`.spec.ts`, 27 fichiers)
- `_bmad-output/implementation-artifacts/e35-choix-autonomes-e355c.md` (neuf),
  `35-5c-date-du-jour-tests-review-prompt-p1.md` (neuf), cette fiche, `sprint-status.yaml`

## Change Log

- 2026-11-12 — Fiche rédigée (découpage de la 35-5b, V1-8 de sa validation P1 ; C-e350-22). Validation non lancée.
- 2026-11-12 — **Validation P1** (Sonnet, `kesh-gate-logs/e35-5c-validation-p1.md` : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 5 LOW)
  remédiée par l'agent de rédaction (worktree `kesh-e355`), avec V2-6 de la validation P2 de la 35-5b. Prémisses lues au code
  de `release/0.17.6` (formes des specs relues à leurs lignes, `invoice-helpers.ts:120-124`, `vite.config.ts:59`).
  - **V1-1** : `addDaysIso` existe dans `src` ; le helper des specs s'appelle `ajouterJoursIso`, écrit en `getUTC*` sans
    `toISOString` ni `new Date()`, testé dans `helpers/dates.test.ts` ; M4 redéfinie contre le relevé de `new Date()`.
  - **V1-2, V1-3** : tableau fermé des formes Node (sept sites, dont `invoices.spec.ts:523-524` et
    `supplier-invoice-draft-cycle.spec.ts:685-688`, absents de la rédaction), avec « horloge lue » et la source retenue ; liste
    des arithmétiques sur un jour donné (quatre), distinguée de celles qui partent de `new Date()`.
  - **V1-4** : la garde relève **tout `new Date()` sans argument** dans `tests/e2e/**` (41 aujourd'hui) contre une allowlist
    close de deux entrées (`todayInZurich`, `audit-log.spec.ts` `todayUtc`) ; sonde « jour par une variable ».
  - **V1-5** : la garde Rust exclut son propre fichier (décidé à la 35-5a, D4, validation P2 V2-5) ; D2 le dit.
  - **V1-7** : AC5 chiffré par `KESH_TEST_INSTANT` (C-e350-26 ; `faketime` absent de la station) ; interdiction des `static`
    / `OnceLock` (D1), M5.
  - **LOW** : V1-6 (formes `Utc::now().year(`… et `DATE(UTC_TIMESTAMP` ajoutées à la garde de la 35-5a ; angle mort de la
    variable d'instant écrit là) ; V1-8 (AC6 par `git diff -U0` ; `jour_suisse(Utc::now())` en ligne dans les `mod tests` de
    `src` ; jours de montage de `journal_entries.rs` sans verdict un par un) ; V1-9 (quatre lectures justes du jour suisse
    ramenées à `todayInZurich()`) ; V1-10 (liste des specs restantes après la 35-5b au Change Log de l'étape 0).
  - **35-5b V2-6** : `audit-log.spec.ts` reste au jour UTC, entrée motivée ; l'allowlist n'est donc pas vide.
  - Liste des fichiers Rust mise à jour par la validation P2 de la 35-5a : `invoice_delete_e2e.rs`, `invoice_unvalidate_e2e.rs`
    rejoignent la 35-5c ; l'appel de `admin_full_import_e2e.rs:1809` part à la 35-5a.
  - Aucun finding réfuté. Signal de découpage : non levé (validation : tenable en une boucle). Aucune migration. Statut
    `ready-for-dev`.
- 2026-11-12 — **Validation P2** (Opus, `kesh-gate-logs/e35-5c-validation-p2.md` : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; série
  5 → 2 MEDIUM) remédiée par l'agent de rédaction (worktree `kesh-e355`). Prémisses lues au code de `release/0.17.6`.
  - **V2-1** : la sûreté de nuit des 50 sites de `journal_entries.rs` tient à ce que `kesh-db` reçoit la date et que chaque
    test fabrique ses données depuis le même jour (écrit) ; AC5 dit que ce binaire ne lit pas `KESH_TEST_INSTANT` ; la course
    de `setup()` est fermée (le jour lu une fois, réemployé), M6.
  - **V2-2** : M5 est tuée par la garde de la 35-5a0 (aucun `static`/`OnceLock`/`LazyLock` dans l'horloge et les helpers),
    non par AC5 (un processus neuf par exécution) — dit.
  - **LOW** : V2-3 (fragments = texte exact des lignes `:391` et `:58`, un relevé par ligne, sonde) ; V2-4 (specs de la 35-5a
    nommées, compte d'entrée sur la 35-5b fusionnée) ; V2-5 (41 = 27 + 7 + 7, 34 = 27 + 4 + 3, 29 fichiers, « jours de
    montage » sans la forme, ligne du `mod tests` relevée sur la base) ; V2-6 (`new Date(Date.now` relevé, `invoices.spec.ts:265`
    à l'allowlist, angle mort écrit).
  - Découpage de la 35-5a répercuté (35-5a0, C-e350-28) ; helper `aujourd_hui_test()` (C-e350-29) ; AC5 ajoute l'exécution
    « nuit réelle » sur tous les binaires, sans plus aucun rouge à la fin de l'epic.
  - Aucun finding réfuté. Signal de découpage : non levé. Aucune migration. Statut `ready-for-dev`.
- 2026-11-12 — **Validation P3 ciblée** (`kesh-gate-logs/e35-5c-validation-p3-ciblee.md`, lentille unique sur `a1923051` :
  0 HIGH, 0 MEDIUM, 3 LOW). LOW appliqués : V3-1 (contrôle de M6 par le motif `Utc::now\(\)\s*\.(date_naive|naive_utc)`, le motif
  nu rendant aussi `:5125`, `timestamp_nanos_opt`, lu) ; V3-2 (la course touche les 51 appels de `setup(&pool)` et
  `lettrer_dans`, non trois sites ; l'argument de cohérence vaut après la fermeture) ; V3-3 (M5 tuée par la garde dans les seuls
  fichiers qu'elle nomme ; AC5 : aucun rouge dû au jour, chaque rouge nommé ; « `kesh-db` ne lit plus l'horloge » de la 35-5a0 à
  la 35-5b). Aucun finding réfuté.
  **Validation terminée.** Série : P1 5 MEDIUM → P2 2 MEDIUM → P3 ciblée 0 au-dessus de LOW. Modèles : P1 Sonnet, P2 Opus, P3
  ciblée. Les remédiations n'ont touché que la fiche, le registre et le `sprint-status.yaml`. Aucune migration. Statut
  `ready-for-dev`.
- 2026-11-14 — **Développement** (Opus 5.5, worktree `kesh-e355c`, voie 2 ; base `ebd5b654c`, pointe de la 35-5b au code final ;
  dernier commit de code `40090cdd3`). Journaux sous `kesh-gate-logs/e355c/`. Aucune migration, aucun code de production.
  - **Étape 0, relevée sur la base** : la liste d'exemptions de la garde portait encore, hors les deux permanentes,
    `journal_entries.rs` 50, `accounts_e2e.rs` 2, `admin_full_import_e2e.rs` 3 (et non 4 : la remédiation P2 de la 35-5a en avait
    retiré un), `credit_card_e2e.rs` 3, `invoice_delete_e2e.rs` 2, `invoice_unvalidate_e2e.rs` 6, `mcp_enveloppe_e2e.rs` 1,
    `reconciliation_link_e2e.rs` 1 (lu à `date_du_jour_garde.rs:188-241` de la base). `mod tests` de `journal_entries.rs` à la ligne
    **3198** de la base (non `:3178`) ; 51 appels de `setup(&pool)`, `setup()` à `:3239`. Specs : `git grep -nE 'new Date\(\s*\)'`
    rend toujours **41** sur la base — la 35-5a n'avait converti aucune ligne de ce motif ; la garde étendue y relève **50** lignes,
    dont **47 sans entrée** (le compte d'entrée, `rouges.log`) et les 3 de l'allowlist : 41 + 1 (`new Date(Date.now`) + 8 lignes à
    `toISOString` seul (dont `invoices.spec.ts:527`, forme coupée vue à la ligne de `.toISOString()`). Les deux tableaux de la fiche
    (formes Node, arithmétiques sur un jour donné, lectures déjà justes, `audit-log.spec.ts`) se confirment ligne à ligne.
    *(Annoté le 2026-11-14, revue P1, A2 : « `setup()` à `:3239` » désigne la ligne de lecture de l'horloge ; la fonction
    commence à `:3218` sur la base (`async fn setup(pool: &MySqlPool) -> (i64, i64, i64) {`) et à `:3225` au commit
    `40090cdd3`, où la lecture est à `:3246`.)*
  - **Rouges d'abord** : garde Rust fermée sur la base, 5 tests sur 7 rouges, les huit fichiers nommés à leur compte
    (`rouges-rust.log`) ; garde frontend étendue, 47 relevés sans entrée ; `dates.test.ts` rouge faute de `dates.ts` (`rouges.log`).
  - **Verdict par fichier (AC2)** — aucun de ces tests n'asserte une date **produite par le serveur** à partir de son horloge : tous
    datent des pièces de montage, donc `aujourd_hui_test()` (D1) :
    - `journal_entries.rs` (`mod tests`) : `setup()` lit `kesh_core::horloge::jour_suisse(chrono::Utc::now())` une fois et le rend
      (C-e355c-1 : `HorlogeFixe` est interdite en `src` hors trois fichiers par la garde de durcissement) ; les 50 destructurations
      le lient (`today`, `jour`) ou l'écartent (`_`, `test_find_covering_date_none`) ; `lettrer_dans` reçoit `jour` ; la borne du
      verrou de `delete_in_tx_of_a_lettered_entry_in_a_locked_period_says_period_locked` et l'année recréée de
      `test_create_rejects_closed_fiscal_year` (`today.year()`) le réemploient (C-e355c-2) ; le `setup(&pool).await;` nu de
      `update_no_op_in_closed_fy_returns_fiscal_year_closed` (nettoyage) reste tel quel.
    - `accounts_e2e.rs` : `current_year()` et `mk_entry` (date de l'écriture) ; `mod common;` ajouté.
    - `admin_full_import_e2e.rs` : les deux bornes `today - 30` et la précondition « borne passée » de `:2283` (C-e355c-6).
    - `credit_card_e2e.rs` : un `jour` lu une fois pour la date de la facture et les deux `settledOn` ; `mod common;` ajouté.
    - `invoice_delete_e2e.rs` (date de facture, borne) et `invoice_unvalidate_e2e.rs` (six dates de brouillon) ; `mod common;`.
    - `mcp_enveloppe_e2e.rs:1237` (date et échéance saisies) ; `reconciliation_link_e2e.rs:757` (jour de facture et de transaction).
    - *(Annoté le 2026-11-14, revue P1, A2 — les lignes lues qui manquaient, au commit `40090cdd3`, par
      `git show 40090cdd3:<fichier> | grep -nF 'aujourd_hui_test()'` : `accounts_e2e.rs:247` (`current_year()`) et `:287`
      (`entry_date` de `mk_entry`) ; `credit_card_e2e.rs:1063` (`let jour = common::jour::aujourd_hui_test().to_string();`) ;
      `invoice_delete_e2e.rs:165` (date de facture) et `:302` (`let borne = … - TimeDelta::days(30);`) ;
      `invoice_unvalidate_e2e.rs:215`, `:328`, `:605` (dates de brouillon) et `:372`, `:453`, `:537`
      (`let aujourd_hui = common::jour::aujourd_hui_test();`) ; `admin_full_import_e2e.rs:2177` et `:2235`
      (`let borne = … - chrono::Duration::days(30);`), et `:2283` (précondition). Toutes datent une pièce ou une borne de
      montage ; aucune n'asserte une date produite par le serveur.)*
  - **Specs (AC3)** : 27 specs et deux helpers convertis à `todayInZurich()` et à `ajouterJoursIso` (neuf, `helpers/dates.ts`,
    `Date.UTC` et `getUTC*`, sans lecture de l'horloge ; refus d'un jour mal formé, C-e355c-5) ; `invoices.spec.ts:523-528` et
    `supplier-invoice-draft-cycle.spec.ts:685-688` (jour du poste) deviennent `todayInZurich()`, le navigateur des specs étant à
    Zurich (`playwright.config.ts`, `timezoneId`) ; `reconciliation-show-older.spec.ts:80` part de `'2025-01-01'` ; les
    `todayZurich()` locales délèguent (C-e355c-4) ; `audit-log.spec.ts` garde `todayUtc()`. Restent les trois lignes de l'allowlist.
  - **Gardes (AC1, AC3, AC4)** : garde Rust à deux entrées (sondes : un module de tests de `src`, un fichier de `tests`, une forme
    coupée sur trois lignes, et le « + 1 » ramené sur l'entrée permanente d'`admin.rs`) ; garde frontend étendue à `tests/e2e`
    (motifs sur le texte entier, un relevé par ligne, ensemble clos de trois entrées au texte exact ; sondes : entière, coupée,
    jour par une variable, `new Date(Date.now() + 1)`, `new Date(\n)`, ligne à deux motifs → un relevé apparié, `ajouterJoursIso`,
    `Date.UTC`, `Date.now()` seul, relevé hors allowlist nommé). **Angle mort assumé** (écrit dans la garde) : un jour tiré de
    `Date.now()` sans construction de date ; la garde lit les commentaires (C-e355c-3).
    *(Annoté le 2026-11-14, revue P1, B3/B5/E4/E5 — angles morts que cette liste taisait : l'appariement par `includes`
    avec un relevé par ligne laissait une seconde lecture passer sous une ligne admise (fermé : appariement exact) ;
    `new Date;`, `new globalThis.Date()`, `Date()` sans `new`, `Intl.DateTimeFormat().format()` sans argument (fermés :
    motifs étendus) ; restent assumés un instant passé par une variable, `new Date(...[])`, `new Date(+Date.now())`, un
    alias de `Date`, `Reflect.construct`, et le jour relu plusieurs fois dans une même spec. Côté Rust, la garde ne voyait
    pas `jour_suisse(Utc::now())` ni `HorlogeFixe(Utc::now())` dans `tests/` (B2/E6, fermé : garde « horloge réelle d'un
    test »). C-e355c-8, C-e355c-9.)*
  - **AC5, la nuit** (`nuit.log`) : nuit réelle reproduite (`2026-11-14T23:30:00Z`) sur **tous** les binaires, **5372/5372** ;
    les huit binaires touchés sous `2025-12-31T23:30:00Z` et `2026-03-10T23:30:00Z` : **170/172**, deux rouges de classe (b) dans
    `admin_full_import_e2e` (`full_import_letters_a_reopened_historical_piece`, borne `2026-03-31`, montage de la 35-5a0 ;
    `validating_a_backdated_invoice_is_refused_by_the_period_lock`, borne `2026-06-30`), les mêmes **de jour** au 11 mars 2026,
    absents au 15 juillet 2026 : un instant antérieur à une borne fixe supposée passée, non la nuit. **Écart à la lettre d'AC5**
    (« verts »), déclaré (C-e355c-6), à relier à #1125. **Sonde positive** : S2, un montage de `bank_accounts_closed_e2e` ramené au
    jour UTC réel, rougit sous l'instant de nuit ; S1, les sept binaires convertis rendus à leur forme d'avant, **ne rougit pas**
    (165/165) : ces sites datent des pièces que le serveur ne compare pas à son jour dans ces tests — la conversion les rend
    cohérents, elle ne corrigeait pas un rouge observable. Le binaire `lib` de `kesh-db` ne lit pas la variable : sa sûreté tient à
    la lecture unique de `setup()` (ligne lue : `journal_entries.rs:3246`, `let today = kesh_core::horloge::jour_suisse(chrono::Utc::now());`).
    *(Annoté le 2026-11-14, revue P1 — A4 : la conclusion de S1 repose sur **un seul instant**, N1
    (`2026-11-14T23:30:00Z`), qui ne franchit ni une année ni un exercice ; elle vaut sous N1, non en général (S1 n'a pas
    été rejoué sous B1, où les formes d'avant, à l'heure réelle, auraient daté les pièces après le jour du serveur). A3 :
    des deux rouges de classe (b), `validating_a_backdated_invoice_is_refused_by_the_period_lock` est **créé par cette
    story** (sa précondition lisait l'heure réelle sur `ebd5b654c`, verte sous tout instant) ; l'autre vient de la 35-5a0.
    AC5 était inatteignable à la lettre sans #1125. Les deux tests sont commentés sur #1125 (fait par l'orchestrateur le
    2026-11-14). C-e355c-6 annoté.)*
  - **M6 par lecture** (`mut/M6-lecture.log`) : le motif `Utc::now\(\)\s*\.(date_naive|naive_utc)` ne rend **rien** sur le module
    (50 sur la base : sonde positive) ; le motif nu `Utc::now` n'y rend que `:3246` (`setup()`) et `:5129`
    (`timestamp_nanos_opt`, suffixe d'unicité).
  - **Mutations** (`mut/resume.log`, arbre `40090cdd3`, chaque fichier restauré par `git checkout --` puis `touch`) : M1
    (`accounts_e2e.rs:287`), M2 (forme coupée, `invoice_delete_e2e.rs:302`) et M5 (`OnceLock` dans `common/jour.rs`) tuées par la
    garde Rust, chacune nommée par sa ligne (jouées ensemble, une exécution) ; M3 (`credit-notes.spec.ts:89`) et M4
    (`ajouterJoursIso` depuis l'horloge) tuées par la garde frontend, M4 aussi par cinq tests de `dates.test.ts`. 6/6.
  - **AC6** (`ac6.log`) : un seul fichier de `crates/*/src`, 107 hunks commençant tous à la ligne 3213 ou au-delà (`mod tests` à
    `:3198` sur la base) ; `frontend/src` : `local-date-guard.test.ts` seul ; migrations intactes.
  - **Gates au dernier commit de code `40090cdd3`** : `scripts/test-fast.sh --ci` sur la voie 2, bases remises à neuf, fmt +
    clippy + nextest **5372/5372**, 13 ignorés (`gate-test-fast.log`) ; `scripts/gate-ci.sh kesh_ci_e355c` (voie 1)
    **5379 passés, 0 échoué, 17 ignorés** (`gate-ci.log`, `gate-ci.journal`) ; frontend : `npm run check` 0 erreur (29 avertissements, aucun sous `tests/`), `lint-i18n-ownership`
    vert, `test:unit` **3229** tests / 233 fichiers, `build` vert (`front-*.log`). **E2E : non lancé** (clôture).
  - **Hors périmètre, dit** : C-e355b-5 (`services_rollout::ac11_a…`, fichier non touché ici, vert sous N1 ; C-e355c-7) ;
    `supplier-internal-number.spec.ts:237` attend l'aperçu `KF-2026-0001`, une année fixe qui rougira en 2027 (classe de #1125).
  - Consigne Edit : tenue — code et tests par l'outil Edit seulement (`replace_all` pour les lignes identiques de
    `journal_entries.rs`, `payment-batches.spec.ts`, `reports.spec.ts`, `invoice_unvalidate_e2e.rs`, `credit_card_e2e.rs`,
    `admin_full_import_e2e.rs`), `cargo fmt` ; la sonde S1 ramène `crates/` à la base par `git checkout ebd5b654c -- crates`, puis
    `git checkout HEAD -- crates` et `touch` (comme le S1 de la 35-5b).
  - Statut `review`. Revue P1 : `35-5c-date-du-jour-tests-review-prompt-p1.md` (trois lentilles, Opus).
- 2026-11-14 — **Revue de code P1** (Opus, trois lentilles ; `kesh-gate-logs/e355c-review-p1-{A,B,E}.md`) : A 1 MEDIUM + 4 LOW,
  B 2 MEDIUM + 4 LOW, E 1 MEDIUM + 6 LOW ; après fusion des doublons, **2 MEDIUM** (A1 = B1 = E1 ; B2, = E6 classé LOW par sa
  lentille) et 9 LOW distincts (A2 ; A3 = E7 ; A4 ; A5 = B4 = E3 ; B3 = E5 ; B5 ; B6 ; E2 ; E4). Aucun finding réfuté.
- 2026-11-14 — **Remédiation P1** (Opus 5.5, worktree `kesh-e355c`, voie 2 ; commit de code `7fcede7e8`, 11 fichiers). Journaux
  `kesh-gate-logs/e355c/p1-*.log`. Aucun code de production, aucune migration.
  - **A1 = B1 = E1 (MEDIUM)** — `docs/testing.md:119` récrit au présent : la suite suit `KESH_TEST_INSTANT` depuis la 35-5c ;
    deux exceptions nommées, (a) les tests à bornes fixes de 2026 (#1125, deux d'`admin_full_import_e2e`), (b) le binaire `lib`
    de `kesh-db` (module de tests de `journal_entries.rs`, jour réel lu une fois par `setup()`) ; la règle des specs
    (`todayInZurich()`, `ajouterJoursIso`, garde étendue à `tests/e2e`) ; la parenthèse de fin ne parle plus de « montages non
    convertis ». Symptôme grepé (`git grep -nE "35-5c|non convertis|pas convertis|ne la suit pas|heure réelle"` sur `docs`,
    `CLAUDE.md`, `crates`, `frontend`) : plus aucune phrase vivante qui annonce la 35-5c au futur ; les mentions des fiches
    35-5a0, 35-5a, 35-5b, de l'epic et du registre C-e350 sont des relevés datés de leur planification, laissés tels quels.
    Le manuel utilisateur ne parle que du produit (lu par A et B) : inchangé.
  - **B2 (MEDIUM) = E6** — garde « horloge réelle d'un test » dans `date_du_jour_garde.rs` (troisième famille, C-e355c-8) :
    `Utc::now()` passé directement à `jour_suisse` ou à `HorlogeFixe`, compté sur le texte entier sans ses lignes `//`
    (forme coupée vue), sur tout `src` et `tests`, liste close à égalité stricte de **deux** entrées — `journal_entries.rs` 1
    et `middleware/auth.rs` 1 (le détecteur, joué à liste vide, a trouvé ce second site, que l'orchestrateur n'attendait pas ;
    ligne lue `:302`, `test_state()`, choix de la 35-5a0). Rouge d'abord (`p1-rouge-garde-rust.log` : les deux sites nommés) ;
    sonde `sonde_l_horloge_reelle_d_un_test_rougit` (qualifiée et coupée dans `tests/`, `HorlogeFixe(Utc::now())` dans
    `tests/`, seconde lecture dans `journal_entries.rs` ; verts : instant donné, `Utc::now().timestamp()`, commentaire). Angle
    mort écrit dans l'en-tête : variable intermédiaire, alias, enveloppe, `SystemTime`, `impl Horloge` ad hoc.
  - **A5 = B4 = E3** — `ajouterJoursIso` refuse une année hors de 1000 à 9999 (entrée et résultat), un jour impossible (contrôle
    aller-retour), un `n` non entier ; trois tests neufs (`dates.test.ts` 6 → 9), rouges sur le `dates.ts` de `40090cdd3`
    (`p1-rouge-dates.log`, obtenu par `git stash push -- dates.ts` puis `git stash pop`, fichier relu identique par `cmp`).
    C-e355c-5 annoté.
  - **E4, E5 = B3 ; B5** — garde des specs : appariement **exact** de la ligne (sans indentation) pour `ALLOWLIST_SPECS`
    (`auditer(…, exact)`), la moitié `src` gardant l'extrait ; quatre motifs de plus (`new Date;`, `new globalThis.Date()`,
    `Date()` sans `new`, `.format()` sans argument), aucun site actuel relevé de plus ; deux sondes neuves, rouges sans
    l'extension (`p1-rouge-garde-front.log`) ; angles morts restants écrits dans l'en-tête de la garde et annotés ci-dessus
    (dont B5, le jour relu plusieurs fois dans une spec : non fermé, risque inhérent). C-e355c-9.
  - **E2** — `accounts_e2e.rs` : `current_year()` retiré ; chaque test lit `let jour = common::jour::aujourd_hui_test();` une
    fois et le passe à `mk_fiscal_year`, à `mk_entry` et à l'assertion `Exercice {}` (`jour.year()`).
  - **B6** — alias retirés : `todayZurich()` (`bank-manual-statement.spec.ts`, `open-items.spec.ts`) et `today()` (trois
    `invoices-settle*.spec.ts`), appels remplacés par `todayInZurich()`. C-e355c-4 annoté.
  - **A2, A3 = E7, A4** — annotations datées dans l'entrée « Développement » ci-dessus (lignes lues des cinq fichiers, ligne de
    `setup()` ; rouge créé par la story et commentaire de #1125 ; S1 borné à N1) et au registre (C-e355c-6). Déclaration
    C-e355c-6 gardée, borne non rendue relative.
  - **Mutations sur l'arbre commité `7fcede7e8`**, chaque fichier restauré par `git checkout --` puis `touch`, `git status` vide
    après : MP1 (`credit_card_e2e.rs:1063` → `jour_suisse(chrono::Utc::now())`) tuée par `horloge_reelle_d_un_test`
    (« credit_card_e2e.rs : 1 occurrence(s), 0 écrite(s) ») ; MP2 (motif cassé, `nowX`) tuée (« journal_entries.rs : 0
    occurrence, 1 écrite(s) », sondes muettes) ; MP3 (aller-retour retiré), MP4 (`Number.isInteger` retiré), MP5 (bornes
    d'année élargies à 0-99999) tuées chacune par son test de `dates.test.ts` ; MP6 (appariement exact retiré) tuée par la
    sonde E4. 6/6 (`p1-mut-MP1.log` à `p1-mut-MP6.log`).
  - **Gates au commit `7fcede7e8`** : `scripts/test-fast.sh --ci` sur la voie 2, base `kesh_e355c` remise à neuf, fmt + clippy +
    nextest **5374/5374**, 13 ignorés (`p1-test-fast.log` ; 5372 + les deux tests neufs de la garde) ; ciblés
    `p1-cible-verts.log` (`date_du_jour_garde`, `accounts_e2e` : 28/28) ; frontend : `npm run check` 0 erreur
    (29 avertissements), `lint-i18n-ownership` vert, `test:unit` **3234** tests / 233 fichiers (3229 + 5), `build` vert
    (`p1-front-*.log`). **Non lancés** : `gate-ci` et E2E, à la clôture (consignes de l'Epic 35).
  - Consigne Edit : tenue pour le code, les tests et les docs (`replace_all` pour les lignes identiques d'`accounts_e2e.rs` et
    des specs), `cargo fmt`. Le prompt de la P2 ciblée, fichier neuf, est créé par l'outil Write. Les rouges temporaires de la
    garde Rust et de la garde frontend ont été posés et retirés par Edit avant le commit ; le rouge de `dates.ts` par
    `git stash`.
  - Prompt de la passe suivante : `35-5c-date-du-jour-tests-review-prompt-p2-ciblee.md` (une lentille, Sonnet, le seul commit
    `7fcede7e8`). Statut `review`.
