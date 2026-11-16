# Prompt — revue de code P1 de la Story 35-5c (refs #929), trois lentilles, première passe complète

Tu relis la **Story 35-5c** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **les tests ne dépendent plus de l'heure à laquelle on les lance**. Les tests Rust prennent le jour de
l'horloge de test (`aujourd_hui_test()`), une date fixe, ou — dans le module de tests de `kesh-db journal_entries.rs` — le jour que
`setup()` lit **une fois** ; les specs E2E prennent `todayInZurich()` et calculent leurs décalages par un helper neuf,
`ajouterJoursIso` ; les deux gardes se ferment (Rust : plus que les deux entrées permanentes ; frontend : étendue aux specs, ensemble
clos de trois entrées). **Aucun code de production. Aucune migration.** La 35-5a0, la 35-5a et la 35-5b (closes) ont posé l'horloge,
`jour_suisse`, `aujourd_hui_test()` et la garde `date_du_jour_garde.rs`. Kesh tient des livres réels : un test qui rougit selon
l'heure masque une vraie régression, ou fait douter d'une release.

Worktree `/home/gcorbaz/devel/kesh-e355c`, branche `story/35-5c-date-du-jour-tests`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e355c diff ebd5b654c 40090cdd3` (`ebd5b654c` = pointe de la 35-5b, base ; `40090cdd3` = dernier
commit de code ; le commit suivant ne porte que la fiche, `sprint-status.yaml` et ce prompt). Lis tout le diff : il est mécanique,
c'est là que se cachent les écarts. Langue du rapport : français. Modèle : **Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-5c-date-du-jour-tests.md` en entier (*Ce qui se lit au code*, D1 à D4, AC1 à AC8,
mutations, **Change Log** — entrée « Développement » — et **Dev Agent Record**) ; les registres `e35-choix-autonomes-e355c.md`
(C-e355c-1 à -7) et `e35-choix-autonomes-e355b.md` (C-e355b-5). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le
avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e355c/` : `rouges.log` (garde frontend, compte
d'entrée), `rouges-rust.log` (garde Rust), `cible-1.log`, `verts-front-cible.log`, `nuit-B1.log`, `nuit-B2.log`, `nuit-B3.log`,
`nuit-J1.log`, `nuit-N1.log`, `nuit-S1.log`, `nuit-S2.log` et la synthèse **`nuit.log`**, `mut/*.log`, `ac6.log`, `gate-test-fast.log`,
`gate-ci.log`, `front-check.log`, `front-lint-i18n-ownership.log`, `front-test-unit.log`, `front-build.log` (commit et environnement
en tête de chacun). Les scripts `*.sh` sont à **lire**, jamais à exécuter.

**Livrables visibles** : **aucun**. La story ne touche aucun écran, aucune route, aucun texte : ni capture, ni clé i18n, ni point au
`CHANGELOG.md` (AC8). **Le manuel** `docs/manual/fr/user-manual.tex` **et son PDF** `docs/manual/fr/user-manual.pdf` ne changent pas :
vérifie-le (`git diff --stat ebd5b654c 40090cdd3 -- docs CHANGELOG.md` vide), puis contrôle que la note *La date du jour est celle de
Zurich, même la nuit* (`\label{sec:date-du-jour}`) ne promet rien que cette story aurait dû livrer côté tests — PDF lu aplati :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (lecture seule, sortie à l'écran, sans redirection).
`docs/testing.md` : une phrase sur les tests « la nuit », les dates du jour des specs ou `todayInZurich` qui serait devenue fausse ?

⚠️ **Ce qui n'a pas tourné** : l'**E2E (Playwright)** — il se joue à la clôture, avec un second `gate-ci` ; les specs converties
n'ont donc été vérifiées qu'au typage (`npm run check`) et par la garde. Le compte rendu le dit.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part.
- **Aucune redirection, sous aucune forme** : ni `>`, `>>`, `<`, `tee`, ni `2>`, `&>`, `2>&1`, **ni `2>/dev/null`**, ni `> /dev/null`.
  **Aucune substitution de processus** (`<(…)`, `>(…)`) ni de commande qui écrive. Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`commit`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ;
  ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`,
  ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**,
  `sed -i`, `playwright`, `vitest`. `pdftotext` est permis, sortie à l'écran.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext … -`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Le module de tests de `journal_entries.rs` (D1, M6)** : `setup()` lit-il le jour **une fois** (`jour_suisse(Utc::now())`,
   C-e355c-1) et le rend-il ? Les 51 appels de `setup(&pool)` : chacun lie-t-il le jour ou l'écarte-t-il explicitement (`_`), et
   **aucun** test ne relit-il l'horloge — `lettrer_dans` (paramètre `jour`), la borne de
   `delete_in_tx_of_a_lettered_entry_in_a_locked_period_says_period_locked`, l'année de `test_create_rejects_closed_fiscal_year`,
   les trois helpers à `jour` ? Un test où le jour lié n'est **pas** celui de l'exercice que `setup()` a garanti (un second `setup()`
   au milieu, comme `update_no_op_in_closed_fy_returns_fiscal_year_closed`) ? Le `timestamp_nanos_opt` de `:5129` est-il bien un
   suffixe d'unicité ?
2. **Les sept fichiers `kesh-api/tests` (D1, AC2)** : chaque site devient-il `common::jour::aujourd_hui_test()` ? Un de ces tests
   asserte-t-il une date **produite par le serveur** (auquel cas D1 exige une `HorlogeFixe`, non `aujourd_hui_test()`) ? Le `mod
   common;` ajouté à quatre fichiers ne heurte-t-il rien (helpers homonymes, `dead_code`) ? Deux horloges dans un même test
   (`Utc::now()` restant pour autre chose qu'un horodatage ou un suffixe) ?
3. **La garde Rust (D2, AC1, AC4)** : la liste d'exemptions ne porte-t-elle plus que `kesh-core/src/horloge.rs` (1) et
   `kesh-api/src/routes/admin.rs` (1) ? Les sondes ajoutées rougissent-elles **par la bonne garde** (marque attendue) : un module de
   tests de `src`, un fichier de `tests`, une forme coupée sur plusieurs lignes ? La sonde « + 1 dans un fichier exempté » vise-t-elle
   encore une entrée existante ? Rouge prouvé sur la base (`rouges-rust.log`, entrées et comptes) ?
4. **Les specs E2E (D3, AC3)** : chaque site du compte d'entrée (47 relevés sans entrée, `rouges.log`) est-il converti à
   `todayInZurich()` ou `ajouterJoursIso` **sans changer ce que la spec prouve** ? En particulier : `invoices.spec.ts` (échéance
   = jour du poste + 30 : le navigateur est-il bien à Zurich, `playwright.config.ts`, `timezoneId`) ;
   `supplier-invoice-draft-cycle.spec.ts` (la lecture dans le navigateur remplacée par la lecture Node à Zurich : même valeur ?
   l'horloge du navigateur y est-elle simulée, `page.clock` ?) ; `bank-manual-statement.spec.ts` (`dayBefore` garde son cas du
   1er janvier) ; `reconciliation-show-older.spec.ts` (le 501ᵉ jour depuis 2025-01-01 inchangé) ; `overdueDate`, `daysFromToday`,
   `reports.spec.ts` (−40 jours). `audit-log.spec.ts` garde son `todayUtc()` (motivé) ? Un import circulaire
   (`helpers/inbox.ts` → `api-fixtures.ts`) ?
5. **`ajouterJoursIso` et son test (D3)** : écrit en `Date.UTC` et `getUTC*`, sans lecture de l'horloge ; les quatre cas de la fiche
   et le cas « part du jour reçu » ; le refus d'un jour mal formé (C-e355c-5). Une année < 1000, un `n` non entier, un 31 + 1 sur un
   mois de 30 jours ?
6. **La garde frontend étendue (D4)** : ensemble clos de trois entrées au **texte exact** de la ligne ; un seul relevé par ligne
   (sonde) ; forme coupée vue au numéro de la ligne où elle commence ; `new Date(` suivi d'un retour à la ligne puis `)` ; `new
   Date(Date.now` ; les commentaires lus (C-e355c-3) ; les `*.test.ts` des helpers balayés. Une forme qui lui échappe encore et
   qu'une spec pourrait employer (`Date.prototype`, `new Date(undefined)`, `new Date(Date.now())` écrit avec des espaces, `new
   globalThis.Date()`) : est-elle dans l'angle mort écrit, ou est-ce un trou ?
7. **La nuit, chiffrée (AC5, `nuit.log`)** : B1 et B2 (les deux instants de la fiche, binaires touchés) rendent **deux rouges**,
   tous deux dans `admin_full_import_e2e` (`full_import_letters_a_reopened_historical_piece`,
   `validating_a_backdated_invoice_is_refused_by_the_period_lock`) : le tri « classe (b), instant antérieur à une borne fixe de 2026
   que le montage suppose passée » est-il juste, ligne lue (B3 de jour, J1 après juillet) ? Est-ce un **écart à la lettre d'AC5**
   (« verts ») correctement déclaré (C-e355c-6) — ou faudrait-il le corriger ici ? N1 (tous les binaires, nuit réelle reproduite) :
   vert ? La **sonde positive** S2 (un montage converti de la 35-5b ramené au jour UTC réel) rougit-elle bien par le
   mélange de deux jours ? S1 (les sept binaires convertis rendus à la base) **ne rougit pas** : la conclusion du Change Log (« la
   conversion rend ces sites cohérents, elle ne corrigeait pas un rouge observable ») est-elle juste, ou un de ces tests aurait-il
   dû rougir ? Le binaire `lib` de
   `kesh-db` ne lit pas la variable : l'argument de sûreté écrit au Change Log tient-il (ligne lue) ?
8. **AC6 — rien hors des tests** (`ac6.log`) : un seul fichier de `crates/*/src`, tous ses hunks au-delà de la ligne `3198` (`mod
   tests` sur la base) ; `frontend/src/**` touché par la seule garde ; aucune migration.
9. **Mutations** (`mut/`) : M1 à M6 tuées par la garde ou le test **nommés** ? M5 (mémoire dans un helper de jour) : tuée dans les
   fichiers que la garde nomme ; l'angle mort (un `OnceLock` dans un autre helper de montage) est-il écrit ? Propose des mutations
   plausibles non jouées (un `setup()` qui relit l'horloge pour son exercice après avoir lu `today` ; `todayInZurich` qui lit le fuseau
   du processus ; un relevé qui ignore la ligne de début d'une forme coupée).
10. **Le compte rendu** : chaque chiffre (sites convertis par fichier, 47 relevés, tests ajoutés, gates) recompté depuis la source,
    avec son périmètre et son journal ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné (E2E : **non lancé**) ?

## Lentille B — Blind (le diff seul, sans la fiche)

Relis `crates/kesh-db/src/repositories/journal_entries.rs` (module de tests), les sept fichiers `crates/kesh-api/tests` touchés,
`date_du_jour_garde.rs`, puis `frontend/tests/e2e/**` et `local-date-guard.test.ts`. Cherche : un jour lu deux fois dans un test ; une
date de pièce remplacée par le jour (ce qui changerait l'objet du test) ; un test devenu vide de sens ; un import inutile ou manquant ;
une forme de date du jour qui échappe aux deux gardes.

## Lentille E — Edge (chaque branche, chaque borne)

Minuit à Zurich (22:59:59Z / 23:00:00Z en hiver, 21:59:59Z / 22:00:00Z en été) entre la lecture du jour et la requête ; le
31 décembre à 23:59:59 heure suisse pour `setup()` puis la pièce ; le 1er janvier pour `dayBefore` et `overdueDate(25)` ; un
`KESH_TEST_INSTANT` vide, blanc, à décalage ; une spec lancée sous un fuseau Node `TZ=Pacific/Kiritimati` ou `TZ=America/Los_Angeles`
(ce que la conversion prétend neutraliser) ; un fichier `.ts` de `tests/e2e` vide ou sans retour à la ligne final pour la garde.
Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC8 : livré, testé, au lieu prescrit ? Les écarts déclarés (C-e355c-1 : `jour_suisse` au lieu d'`HorlogeFixe`
dans `setup()` ; C-e355c-6 : deux rouges de classe (b) sous les instants d'AC5 ; C-e355c-7 : C-e355b-5 hors périmètre) sont-ils exacts
et sans effet sur ce que l'AC veut prouver ? L'étape 0 (liste d'exemptions et compte d'entrée relevés sur la base, tableaux de la fiche
confirmés) citée ligne lue ? Les gates du passage en `review` (`gate-test-fast.log`, `gate-ci.log`, frontend) : chaque chiffre au
journal, au commit `40090cdd3`, horodaté après lui ? AC7 exige l'E2E : est-il bien déclaré « à la clôture », conformément aux
consignes de l'Epic 35 ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e355c-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque
rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue
copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la
liste**.
