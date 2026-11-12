# Story 35-5c : les tests ne dépendent plus de l'heure à laquelle on les lance — tests Rust et specs E2E au jour suisse, gardes fermées (#929)

Status: ready-for-dev

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
  binaire (`lib` de `kesh-db`) ne lit pas : `kesh-db` ne lit plus l'horloge (ses fonctions reçoivent la date, 35-5a0), et
  chaque test fabrique lui-même son exercice et ses pièces depuis le même jour — il est cohérent avec lui-même à toute heure.
  **Reste une course**, non une faute de nuit : `setup()` lit l'horloge (`:3219`, exercice « couvrant aujourd'hui ») puis des
  tests la relisent (`lettrer_dans` `:4266`, `:4384`, `:4457`) ; une exécution qui chevauche minuit le 31 décembre crée
  l'exercice d'une année et une pièce de la suivante. **Fermée ici** (D1) : `setup()` rend le jour qu'elle a lu, et les tests
  le réemploient au lieu de relire l'horloge. Les 6 de `invoices.rs` sont à la 35-5b. Aucun autre module de tests de
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
   **tous** les binaires avec `KESH_TEST_INSTANT="$(date -u +%F)T23:30:00Z"`, de jour (la nuit réelle reproduite) : **plus
   aucun rouge**, l'epic étant close sur ce point ; journal `kesh-gate-logs/e355c/nuit.log`. Le binaire `lib` de `kesh-db`
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
de montage qui mémorise le jour dans un `OnceLock` — tuée par la **garde** de la 35-5a0 (aucun `static`/`OnceLock`/`LazyLock`
dans les fichiers de l'horloge et des helpers), **non** par AC5 : chaque exécution est un processus neuf, où la mémoire rend
la même valeur que la version sans mémoire (validation P2, V2-2) ; M6 un test de `journal_entries.rs` qui relit l'horloge au
lieu du jour de `setup()` — non attrapable par un test (la course ne se produit qu'à minuit) : contrôle par lecture, au Change
Log, `/usr/bin/grep -n 'Utc::now' ` sur le `mod tests` ne rendant que la ligne de `setup()`.

## Tâches

- [ ] Étape 0 : partir de la 35-5b fusionnée ; relever la liste d'exemptions restante de la garde Rust et la liste des specs
      restantes (sortie de la garde étendue sur la 35-5b fusionnée : compte d'entrée) ; confirmer les tableaux ci-dessus ;
      Change Log.
- [ ] Conversion des tests Rust (D1) ; garde Rust fermée (D2) ; preuve de nuit (AC5).
- [ ] Specs E2E (D3), `ajouterJoursIso` et son test, garde frontend étendue (D4).
- [ ] Gates : `test-fast.sh --ci` complet à chaque commit (tests de `kesh-db`), `scripts/gate-ci.sh` au dernier, frontend
      complet, E2E complet au dernier commit de code.

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

### Debug Log References

### Completion Notes List

- Fiche rédigée à la remédiation de la validation P1 de la 35-5b (C-e350-22) ; validation à lancer.

### File List

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
