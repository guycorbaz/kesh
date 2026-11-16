# Revue de code P1 — 36-7a (#1125, KF-060), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **36-7a — les treize tests qui
rougissent en 2027** de Kesh (issue #1125). Première passe : **Sonnet ou Opus, jamais Haiku**. Contexte frais. Travaille
en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche ce que les montages portent en eux-mêmes — une date qui
  sort de l'exercice du montage sous un instant du 2026-01-01 au 2029-12-31, une opération qui ne se date plus au jour
  (le test cesse d'exercer la date du jour), un décor partagé modifié qui change le sens d'autres tests, une assertion
  affaiblie, un test qui passerait à vide.
- **E — Edge Case Hunter** : chaque frontière d'instant — heure réelle, nuit suisse du 1er janvier (`2026-12-31T23:30Z`,
  jour suisse 2027-01-01, jour UTC 2026-12-31), `2027-01-04`, `2026-12-26` (verrou glissant d'`open_items_invariant`),
  instant antérieur `2026-03-10T23:30Z`, `2029-06-15`, fin `2029-12-31` ; mois et jours de chaque date dérivée (29 février :
  aucune date dérivée n'est un 29-02 ?), `jours_avant(k)` contre la borne `jours_avant(300)` et le début de `fy` ;
  l'échéance témoin `+30 jours` contre l'exercice 2020-2030 ; N+1 contre N-1..N ; la sonde de D6 à minuit.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC (AC1 à AC9), décision par décision (D1 à D6), mutation
  par mutation (M1a à M10 et les trois jouées à la tâche 4, journaux cités ci-dessous), `docs/testing.md`, la garde
  `local-date-guard.test.ts`, le registre.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-36-7 log --oneline origin/epic/36-0.18.0..HEAD
git -C /home/gcorbaz/devel/kesh-36-7 diff origin/epic/36-0.18.0...HEAD -- crates docs frontend
```

Lis le diff **aplati** (un seul diff de la base à `HEAD`). Fichiers touchés — sept fichiers de tests Rust :
`crates/kesh-api/tests/{admin_full_import_e2e,bank_statements_manual_e2e,dates_bornes_e2e}.rs`,
`crates/kesh-db/tests/{journal_entries_modification,migrations_upgrade_path,opening_complement_repository}.rs`,
`crates/kesh-report/tests/open_items_invariant.rs` ; la spec neuve `frontend/tests/e2e/horloge-sonde.spec.ts` ; la garde
`frontend/src/lib/shared/utils/local-date-guard.test.ts` ; `docs/testing.md`. **Aucune ligne de production** (AC7).

Fiche : `/home/gcorbaz/devel/kesh-36-7/_bmad-output/implementation-artifacts/36-7a-tests-desancres-2027.md` (lentille A ;
B la lit **après** son examen à l'aveugle, pour classer). Registre : `e36-choix-autonomes-36-7.md` (même dossier). Sœur,
hors périmètre : `36-7b-tests-bornes-anterieures.md` (les neuf tests sous instant antérieur — ne les demande pas ici).
Journaux (`/home/gcorbaz/devel/kesh-gate-logs/e36-7/`) : `rouges.log` (relevés à la tête d'avant le correctif), `e0-*.log`,
`inventaire-C2029.log`, `cible-*`/`c1-*.log`, mutations `mut/M*.log` et `mut/T4-*.log`, gates `gate-*.log` ; le Dev Agent
Record de la fiche les cite avec leurs commits.

## Ce qui n'a pas encore tourné (dit d'avance)

`scripts/gate-ci.sh`, l'E2E complet à l'heure réelle, **AC4** (nextest sous libfaketime) et **AC5** (E2E sous libfaketime,
sonde à trois étages) sont des **gates de clôture** (C-36-7-6) : leur absence n'est pas un finding ; une prescription
fausse de la fiche à leur sujet en est un.

*(Annoté après la passe 1, finding A1-6 : `scripts/gate-ci.sh kesh_ci_e367` **a tourné** au passage en `review`, au
commit de code `12b9dd90f` — `kesh-gate-logs/e36-7/gate-ci-review.log`, 5459 passés, 0 échec ; il se **rejoue** à la
clôture, au dernier commit de code. Seuls l'E2E complet, AC4 et AC5 n'ont pas encore tourné.)*

## Axes obligatoires

1. **D1 tenu partout** : chaque date de 2026 des treize tests (relevé de l'étape 0 au Change Log) est-elle dérivée de N,
   ou intouchée pour une raison écrite (décors `setup()`, `body()`) ? L'exercice couvre-t-il N-1..N, et l'opération du jour
   garde-t-elle `aujourd_hui_test()` (y compris le `today` de `complete`) ?
2. **Décors partagés** : `setup()` et `body()` de `bank_statements_manual_e2e`, `v0174a_decor`/`v0174a_invoice`
   (troisième consommateur `full_import_round_trip_keeps_a_draft`), `societe()` (cinq tests), `facture_ouverte_rc1` — la
   correction change-t-elle le sens d'un autre test ? Le numéro `'F-2026-0042'` gardé (référence de la ligne de `body()`)
   est-il juste ?
3. **Les mutations** : chacune remet-elle le site à sa forme d'origine (lis le `git diff` en tête de chaque `mut/*.log`),
   et est-elle tuée par le test nommé, pour la bonne raison (lis le motif de la panique) ? Les trois restées vertes
   (`T4-1332`, `T4-533`, `T4-1008`) sont-elles bien équivalentes, la raison écrite au Change Log se vérifie-t-elle au code ?
4. **La sonde de D6** : la spec compare-t-elle vraiment trois horloges (serveur sans `asOf`, runner, navigateur) ? Peut-elle
   passer à vide (compte 1100 absent, `asOf` lu sous un autre nom) ? L'entrée de la garde est-elle au texte exact, et les
   trois mentions du nombre passent-elles à quatre ?
5. **`docs/testing.md`** : l'exception (a) nomme-t-elle les neuf tests de la 36-7b (contre
   `kesh-gate-logs/e36-7/inventaire-B2026.log`) ? Les recettes de D3/D4 disent-elles vrai (options de faketime, binaire
   construit avant, angle mort MariaDB) ? « quatre sites » ?
6. **Le manuel** : la fiche dit qu'aucun manuel n'est touché. Vérifie-le : `git -C /home/gcorbaz/devel/kesh-36-7 diff
   --stat origin/epic/36-0.18.0...HEAD -- docs/manual` vide ; et, forme imposée du contrôle de PDF (E6), sans redirection :
   `pdftotext /home/gcorbaz/devel/kesh-36-7/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c
   'KESH_TEST_INSTANT'` (attendu 0 : le manuel ne décrit pas cette variable de tests). **Captures** : aucune — la story
   n'ajoute aucun contrôle visible (la spec est un test).
7. **AC7** : `git diff --stat origin/epic/36-0.18.0...HEAD -- 'crates/*/src/*' 'crates/*/src/**' frontend/src` ne rend que
   la garde.

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C /home/gcorbaz/devel/kesh-36-7 show HEAD:<fichier> |
/usr/bin/grep -nF …`), **dont tu copies la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` :
utilise **`/usr/bin/grep`**.) Tout chemin cité se vérifie (`ls -l` ou `git cat-file -e HEAD:<chemin>`), ligne lue citée.

## Sévérités

CRITICAL (un test qui ne prouve plus rien ou qui rougira en 2027 malgré la story), HIGH (AC faux, test qui cesse d'exercer
la date du jour, décor partagé dont le sens change), MEDIUM (incohérence code/fiche/doc, mutation tuée pour une mauvaise
raison, équivalence mal fondée), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
`mariadb`, `faketime` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ou de `kesh-gate-logs/` ;
pas de `git checkout`/`switch`, `git restore`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ;
aucune commande `gh` qui écrit ; pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune
redirection `>` ni fichier temporaire**. `pdftotext` n'est permis que dans la forme de l'axe 6. **Tu n'écris qu'un seul
fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e36-7/review-p1-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B1-1`, `E1-1`, `A1-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé. Pour un LOW qui touche du code de test : dis s'il se corrige dans la story ou s'il s'ouvre en issue (étiquette
   `technical-debt` ou `documentation` si aucun comportement faux n'est visible).
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
