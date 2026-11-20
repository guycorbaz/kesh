# Prompt — revue de code P1 de la Story 36-10 (#1147, #1053), trois lentilles, première passe complète

Tu relis la **Story 36-10** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), release **0.18.0** (sans
migration), Epic 36, phase 2 : **les dernières dates en UTC**. (a) Les filtres de date du journal d'audit — liste et export
CSV — bornent désormais le **jour de Zurich** (`bornes_du_filtre`, `kesh-db/src/repositories/audit_log.rs`, sur
`debut_du_jour_suisse` / `fin_du_jour_suisse`, deux fonctions neuves de `kesh-core/src/horloge.rs`) ; (b) la fenêtre de
30 jours de la réconciliation part du **jour de Zurich** du dernier rappel : la proposition (`unpaid_invoices_in_window_sql`)
lit `X >= e OR rmd.last_sent_at >= début du jour e`, l'acceptation (`invoice_payment_anchor_date`) rend
`max(X, jour_suisse(r))` en Rust ; (c) une famille de plus dans la garde `kesh-api/tests/date_du_jour_garde.rs`
(« jour d'un instant enregistré », `DATE(` et `.date()` sur `crates/*/src`) ; (d) les libellés des filtres perdent
« jour UTC » (quatre catalogues, replis Svelte), la spec `audit-log.spec.ts` prend `todayInZurich()`, son entrée de
`local-date-guard.test.ts` est retirée. **Aucune migration, aucun code d'erreur ni champ de réponse changé, aucune clé i18n
ajoutée ni retirée.**

Worktree `/home/gcorbaz/devel/kesh-36-10`, branche `story/36-10-dernieres-dates-utc`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-10 diff f1ffed694 HEAD` (`f1ffed694` = la fiche validée). Les commits de la story :
`git -C /home/gcorbaz/devel/kesh-36-10 log --oneline f1ffed694..HEAD` — `3f272a5bc` = D1 (`debut_du_jour_suisse`,
`fin_du_jour_suisse`, AC1) et l'étape 0 de la fiche ; `ec807b913` = les tests AC2 à AC8, **rouges** contre le code d'avant
(`rouges.log`) ; `ee35a8076` = le correctif (D2 à D7, docs), **dernier commit de code** ; le dernier commit = la fiche, le
registre, le `sprint-status.yaml` et ce prompt. Le code de production seul :
`git -C /home/gcorbaz/devel/kesh-36-10 diff f1ffed694 ee35a8076 -- 'crates/*/src/*' frontend/src ':!*.test.ts'`. Langue du rapport : français.
Modèle : **Sonnet ou Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-10-dernieres-dates-utc.md` en entier (*Ce qui se lit au code*, inventaire I1 à I5,
D1 à D7, AC1 à AC11, mutations M1 à M8, *Dev Agent Record*, *Change Log*) ; les registres `e36-choix-autonomes-e360.md`
(C-e360-11) et `e36-choix-autonomes-36-10.md` (C-36-10-1 et suivants). Ne prends pas le Dev Agent Record pour argent
comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-10/` :
`etape0-inventaire.log`, `etape0-garde.log`, `rouges.log`, `rouges-run.log`, `mut/*.log` et `mut/*.sh` (à **lire**),
`test-fast-ci.log`, `gate-ci.log`, `front-*.log`, `captures/`.

⚠️ **L'E2E complet n'a pas encore tourné** (il se joue à la clôture de la story ; consigne de l'Epic 36). `test-fast --ci`,
`gate-ci` et le frontend complet ont tourné au passage en `review`. Le dire n'est pas un finding ; affirmer le contraire au Dev
Agent Record en serait un.

**Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e36-10/captures/` — les filtres du journal d'audit ; regarde-les
si ton outil lit les images. **Manuels** : `docs/manual/fr/user-manual.tex` (note « date du jour » vers `:754`, *Algorithme de
matching* vers `:3063`, journal d'audit vers `:4004` et `:4040`), `docs/manual/fr/admin-manual.tex` (`:2558`, `:3633`),
`docs/api-external.md` (`:681`), `docs/testing.md` (`:119`). ⚠️ **Les PDF ne sont pas régénérés dans la story** (à
l'intégration) : contrôle `docs/manual/fr/user-manual.pdf` sous la seule forme permise et attends **`1`** pour
« Les dates filtrent sur des jours UTC » (le PDF publié, encore d'avant) et **`0`** pour « Les dates filtrent sur des jours de
Zurich » — un PDF modifié sur la branche serait un écart :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`,
  `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport. **Tout
  chemin cité** se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`), ligne lue citée.

## Les axes obligatoires, pour chaque lentille

1. **Les bornes d'un jour (D1)** : `debut_du_jour_suisse`, `fin_du_jour_suisse`, `premier_instant_du_jour`
   (`crates/kesh-core/src/horloge.rs`). La dichotomie est-elle exacte à la milliseconde (invariant, terminaison, pas sur des
   millisecondes entières) ? Le cas `Ambiguous` ; `NaiveDate::MAX`/`MIN` (aucune panique hors précondition ?) ; les attendus
   d'AC1 (hiver, été, 23 h, 25 h, 1894) sont-ils justes, recalculés à la main ? Le balayage tient-il le contrat de `jour_suisse`
   à ± 1 ms ?
2. **Le filtre du journal (D2, D3)** : `bornes_du_filtre` et `push_where_clauses` (`crates/kesh-db/src/repositories/audit_log.rs`).
   Liste et export partagent-ils encore une seule clause ? Le plafond de la borne basse (`1000-01-01 00:00:00.000`) est-il
   appliqué toujours, et seulement à elle ? La borne haute de `9999-12-31` (`22:59:59.999`) ? La précondition de
   `push_where_clauses` et son doc-comment (« JOURS de Zurich ») disent-ils vrai ? Le commentaire de
   `routes/audit_log.rs` (nom du fichier, lignes, filtres) ?
3. **L'ancre de la réconciliation (D4)** : `unpaid_invoices_in_window_sql`, `find_unpaid_invoices_for_window`,
   `find_invoices_slightly_overpaid_for_window`, `invoice_payment_anchor_date` (`crates/kesh-db/src/repositories/reconciliation.rs`).
   L'équivalence `max(X, R) >= e ⟺ X >= e OR r >= début(e)` est-elle exacte (`R` absent, `NULL`, microsecondes de
   `DATETIME(6)`) ? L'ordre des `?` liés est-il juste dans les **deux** lecteurs, et le doc-comment qui les énumère ?
   L'acceptation rend-elle exactement l'ancre d'avant de jour ? Les macros et constantes retirées ou gardées : un lecteur
   orphelin, un doc-comment (`[`…`]` intra-doc) qui renvoie à un nom disparu (le module `:1-30`, `:116-125`, `:559-569`,
   `:593-597`) ? Un autre lecteur de l'ancre ailleurs (`git grep -n "anchor" -- 'crates/*/src/*'`) ?
4. **La garde (D5)** : la famille « jour d'un instant enregistré » de `crates/kesh-api/tests/date_du_jour_garde.rs`. Le motif
   compte-t-il ce qu'il dit (et rien de légitime à tort : `DATE_FORMAT(`, `CAST(… AS DATE)`, identifiants Rust en `…_DATE`) ?
   La liste close de trois est-elle juste (relis chaque ligne admise) ? Les sondes rougissent-elles **par cette garde** ? Les
   angles morts écrits sont-ils complets ? Le squelette rougissait-il bien sur `reconciliation.rs` (`rouges.log`) ?
5. **Les tests** (AC2 à AC8) : `audit_log_e2e.rs` (filtre de nuit, export, bornes extrêmes), le test réécrit de
   `audit_log.rs`, `reconciliation_repository.rs` (deux cas de nuit, accord des deux écritures, AC7), `reconciliation_e2e.rs`
   (AC5). Chaque test prouve-t-il ce que son nom dit, ou passe-t-il à vide (liste vide, filtre qui rend tout) ? Les attendus
   sont-ils écrits en clair, non recalculés par la fonction testée ? Les rouges consignés (`rouges.log`) sont-ils ceux de la
   fiche ? Les mutations (`mut/*.log`) : chacune tuée par le test nommé ?
6. **L'écran et la spec (D6, D7)** : quatre catalogues, replis de `+page.svelte` égaux à fr-CH, commentaires ; `audit-log.spec.ts`
   (import, pas 6) ; `local-date-guard.test.ts` (entrée retirée, deux sondes réécrites sur une entrée synthétique — rougissent-
   elles encore sur leur cas ?, mentions « deux »).
7. **Les docs (AC10)** : chaque site du manuel, d'`api-external.md` et de `docs/testing.md` dit-il ce que fait le code ? Reste-t-il
   un « jour UTC » faux (`git grep -n -i "jour UTC\|jours UTC\|UTC-Tag\|UTC day\|giorno UTC" -- docs/ crates/kesh-i18n frontend`) ?
   Le CSV dit-il toujours ses heures en UTC ?
8. **Rien d'autre ne bouge (AC11)** : aucune migration (`git diff --stat f1ffed694 HEAD -- crates/kesh-db/migrations` vide),
   aucun code d'erreur ni champ de réponse changé, aucun test existant dont l'attendu change hors de ceux que la fiche nomme.
9. **Le compte rendu** : chaque chiffre du Dev Agent Record (tests, rouges, mutations, gates) se recompte-t-il depuis la source,
   avec son périmètre et son journal ? `git log origin/epic/36-0.18.0..HEAD --format=%B | /usr/bin/grep -niE
   '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` est-il vide ?

## Les trois lentilles

- **B — Blind Hunter** : le diff aplati seul, sans la fiche d'abord ; ce qui est faux, fragile ou incohérent dans le code tel qu'il est.
- **E — Edge Case Hunter** : chaque branche et chaque borne (minuit exact, changement d'heure, 1894, années 1000 et 9999, `NULL`
  du dernier rappel, rappel annulé, microsecondes, deux rappels), contre le code.
- **A — Acceptance Auditor** : chaque AC (AC1 à AC11) et chaque mutation de la fiche contre ce qui a été livré et exécuté ; chaque
  écart entre la fiche, le registre, le Dev Agent Record et les journaux.

## Rapport

Un rapport par lentille, et **seulement** là :

- B : `/home/gcorbaz/devel/kesh-gate-logs/e36-10/review-p1-B.md`
- E : `/home/gcorbaz/devel/kesh-gate-logs/e36-10/review-p1-E.md`
- A : `/home/gcorbaz/devel/kesh-gate-logs/e36-10/review-p1-A.md`

Chacun porte : l'en-tête (lentille, modèle, commit relu, date) ; les findings numérotés (`B1-1`, `E1-1`, `A1-1`…), chacun avec sa
**sévérité** (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), son lieu (`fichier:ligne`), la **ligne lue citée**, la preuve (sortie copiée), le
remède proposé ; la liste des **axes exercés** et celle des **axes non exercés** (un « 0 finding » sans elle ne compte pas) ; le
décompte des findings par sévérité, recompté depuis la liste.
