# Revue de code P2 — Story 49-1d « Types de lignes — recette de montée » (#1242)

*Prompt de la passe 2 de `bmad-code-review`, **passe complète**, trois lentilles **Opus** en contexte frais : **B** (Blind
Hunter), **E** (Edge Case Hunter), **A** (Acceptance Auditor). La passe 1 (Sonnet) a rendu 0 CRITICAL, 0 HIGH, 2 MEDIUM,
16 LOW (`/home/gcorbaz/devel/kesh-gate-logs/e491d/review-p1-{B,E,A}.md`) ; sa remédiation est décrite au Change Log de la fiche
(entrée « revue P1 remédiée ») et au Dev Agent Record (§ *Remédiation de la revue P1*). Relis **tout** le diff, non la seule
remédiation : la sévérité se déplace vers ce qu'on vient d'écrire, mais une passe complète ne s'y borne pas.*

## Ce que tu relis

- **Worktree** : `/home/gcorbaz/devel/kesh-49-1d`, branche `story/49-1d-types-de-lignes-recette-montee`.
- **Diff** : `git -C /home/gcorbaz/devel/kesh-49-1d diff origin/epic/49-types-de-lignes-facture...HEAD -- scripts crates docs`
  (trois points). Code : `scripts/recette-montee.sh`, `scripts/tests/recette-montee.test.sh`,
  `crates/kesh-db/tests/recette_montee_script.rs` ; documentation : `docs/testing.md` (§ *La recette de montée et de retour*),
  `docs/manual/fr/admin-manual.tex` (section « Passer à la 0.20.0 » ; paragraphe « Compatibilité »).
- **Commits** : `git log --oneline origin/epic/49-types-de-lignes-facture..HEAD`. Code : `e137ae1e9`, `a5cf95297`,
  `a02b79bec`, `bd7e72ce8`, `a4fc72d82` (remédiation P1) ; le reste est documentation.
- **Fiche** : `_bmad-output/implementation-artifacts/49-1d-types-de-lignes-recette-montee.md` ; registre
  `_bmad-output/implementation-artifacts/e0200-choix-autonomes-e491d.md`.
- **Scripts hors dépôt, relus comme du code** (lecture seule) : `/home/gcorbaz/devel/kesh-gate-logs/rel0200/verifs-0200.sh`,
  `rel0200/recette-complement-rel0200.sh`, `rel0200/perte-assumee.sh`, `e491d/mut/mutations.py`, `e491d/t0/sonde-t0.sh`,
  `e491d/dev/chaine-reelle.sh`, `e491d/dev/recette-p1.sh`, `e491d/gates-review.sh` et les scripts de gate de `e491d/`.
- **Journaux** (lecture), sous `/home/gcorbaz/devel/kesh-gate-logs/e491d/` : ceux que nomme le Dev Agent Record — sondes
  (`t0/`), test shell (`dev/test-shell-*.log`), mutations (`mut/juge.log`, `mut/resume-3.log`, `mut/rouge-*.log`), recettes
  (`recette-reelle.log`, `recette-reelle-p1.log`, `recette-reelle-rouge-*.log`), script complémentaire
  (`recette-complement*.log`), gates (`test-fast-ci*.log`, `gate-ci*.log`, `gates-review*.log`). Les dossiers `$D` des recettes
  (`/tmp/kesh-recette.*`) sont nommés à la dernière ligne de chaque journal de recette.
- **Manuel compilé** (la story ne commite pas de PDF) : `/home/gcorbaz/devel/kesh-gate-logs/e491d/manuel/admin-manual.pdf`,
  compilé depuis le `.tex` de la tête de la branche (le Dev Agent Record dit à quel commit).
- **Captures** : sans objet (aucun contrôle visible ; `git diff --stat origin/epic/49-types-de-lignes-facture...HEAD -- frontend`
  doit être vide).

## Le contexte qui commande la revue

Kesh est **en production** : Guy tient ses livres réels en **0.19.1** et a remis sur papier de vraies factures. Cette story est
**le gate de la migration** `20261118000002_invoice_line_types` : la recette doit prouver, depuis l'image publiée
`gcorbaz/kesh:0.19.1`, qu'aucune facture ni aucun avoir existant ne change — lignes, totaux, TVA, écritures, numéros,
règlements, rapport TVA, PDF archivés à l'octet, PDF rendus au même dessin — et que le retour par la sauvegarde
`pre-upgrade/` fonctionne. **Une preuve qui peut passer à vide, ou un garde-fou qu'aucun scénario ni aucune mutation n'éprouve,
est le défaut le plus grave de cette revue.** La passe 1 en a trouvé deux familles (garde-fous de la projection `jq`, garde
« avant vide ») ; cherche la troisième.

## Axes communs (toutes les lentilles)

1. **Inventaire des garde-fous** : liste chaque contrôle du script qui fait échouer une phase (`echec`, `|| return 1`) et, pour
   chacun, le scénario du test qui le fait rougir et la mutation de `mut/mutations.py` qui le retire (`mut/juge.log`). Un
   contrôle sans les deux est un finding ; dis-le par ligne. Les contrôles qui se couvrent l'un l'autre doivent être éprouvés
   séparément (S5/S5b, S10 principal/lien).
2. **Le juge des mutations** : `mutations.py --juger` exclut l'en-tête et exige le résumé « assertion(s) en échec » ; tient-il
   pour tuée une mutation qui casse le test en bloc ? Recompte tuées et survivantes depuis `mut/juge.log`.
3. **Comparaisons à vide** : projection des tables et de `api`, phase 6 (trame, texte, pages, erreurs de `mutool`), crochet,
   script complémentaire.
4. **Le périmètre fermé du script** : relance `env -i`, `HOME` neutralisé, aucune écriture hors de `$D`, aucun montage,
   conteneurs du jeton seuls, jamais `… | grep -q` sous `pipefail`.
5. **La version de départ 0.19.1** : tous les sites opératoires (scripts, fiche — y compris AC9 à AC14, T0, T9 —,
   `docs/testing.md`, manuel) disent 0.19.1 ; les faits « établis par le code » sont vrais au tag `v0.19.1`.

## Lentille B — Blind Hunter

Diff et scripts hors dépôt, sans la fiche. Shell : citation, `set -e`/`pipefail` et leurs exceptions, code masqué par une
substitution, `awk`/`jq`/`sed` fragiles, fichiers relus avant d'être écrits, DRY, doc-comments faux, factices qui prouvent moins
qu'ils ne disent (une assertion dont les deux côtés sortent du même factice), mutation équivalente non déclarée ou déclarée à
tort.

## Lentille E — Edge Case Hunter

Chaque branche et chaque borne, notamment celles que la passe 1 a laissées ouvertes ou déplacées : `mutool.log` qui porte une
ligne ni `warning:` ni erreur ; `410` sans corps JSON ; tableau d'avant non vide devenu `[]` ; objet `{}` devenu `[]` ; clé
d'avant à valeur `false` ou `0` qui disparaît ; un document `api` de moins ; table d'avant vide **non** nommée ;
recette à cheval sur minuit (ce que `docs/testing.md` en dit) ; facture à deux pages ; QR-IBAN avec QRR ; binaire de
débogage contre binaire de release.

## Lentille A — Acceptance Auditor

AC1 à AC16 contre le code et les journaux, un par un, ligne lue citée ; S1 à S14 et leurs compléments (S3d à S3h, S5b, S6c,
S6d, S8b, S10 lien) chacun avec sa mutation tuée (`mut/juge.log`) ; sort de chacun des dix-huit findings de la passe 1 (corrigé,
ou issue demandée avec son texte au Dev Agent Record) — vérifie chaque correctif au code ; Dev Agent Record honnête : chaque
chiffre a son journal et son commit, recompte-le (assertions, mutations, paires, tables, tests des gates). AC16 : gate complet
(`test-fast --ci`, `gate-ci kesh_ci_e491d`) au dernier commit de code ; E2E non joué (clôture).

## Le manuel et `docs/testing.md` (obligatoire)

Vérifie le manuel contre le code et les journaux, **PDF compris**, aplati avant de greper, par cette seule forme :
`pdftotext /home/gcorbaz/devel/kesh-gate-logs/e491d/manuel/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`
(apostrophe typographique). Section « Passer à la 0.20.0 » et « Compatibilité » : sauvegarde en tête ; « se pose sur la
0.19.1 » ; 96 migrations ; aucune valeur réécrite ; `kesh_version_min_required` à 0.17.6 ; `-0.19.1-to-0.20.0.keshbackup`
(établi par le code, non constaté) ; `KESH_SKIP_PRE_UPGRADE_BACKUP=0.20.0` ; message de sqlx du refus (constaté, phase 3) ;
refus d'une sauvegarde 0.20.0 « établi par le code » (`IMPORT_SCHEMA_MISMATCH`, texte de l'écran) ; aucun `DELETE` ni *dirty*
dans la section ; message « partially applied » cité **avec** ses accents graves (`sqlx-core-0.8.6/src/migrate/error.rs:39`) ;
« même dessin », « bloc de paiement compris », et les limites dites (une page, IBAN ordinaire, binaire de développement) ;
« constaté » ne vaut que pour ce qu'un journal établit, et le rejeu de la release est annoncé, non affirmé. `docs/testing.md` :
outils (dont `cmp`), `410` qualifié, erreurs de `mutool`, scénarios et mutations (aucune « équivalente » ne doit plus y être
dite), durées mesurées, minuit, et « Ce que la recette ne prouve pas » (binaire de débogage, plusieurs pages, QR-IBAN + QRR).

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo` (build, test, run, fmt, clippy, nextest), ni
`npm`/`npx`, ni `make`, ni `scripts/prepare-release.sh`, `scripts/recette-montee.sh`, `scripts/tests/recette-montee.test.sh`,
`scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `sqlx`, ni aucun script de `kesh-gate-logs/rel0200/` ou
de `kesh-gate-logs/e491d/` (`verifs-0200.sh`, `recette-complement-rel0200.sh`, `perte-assumee.sh`, `mut/mutations.py` — même
`--juger` —, `t0/sonde-t0.sh`, `dev/chaine-reelle.sh`, `dev/recette-p1.sh`, `gates-review*.sh`, `bases-neuves-e491d.sh` — il
efface les bases de la story —, `test-fast-ci-e491d.sh`, `gate-ci-e491d.sh`), ni `docker`, ni `mariadb`, ni `jq` ou `mutool`
vers un fichier, ni `pdftotext` vers un fichier (seulement vers `-`), ni `git commit`, `git checkout`, `git restore`,
`git stash`, `git worktree`, `git merge`, ni `rm`, `mv`, `cp`, `sed -i`, `flock`, ni redirection vers un fichier, ni fichier
temporaire. Seule écriture permise : ton rapport. `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/e491d/review-p2-{B,E,A}.md` selon ta lentille. Chaque finding : sévérité (CRITICAL,
HIGH, MEDIUM, LOW), fichier et ligne, la ligne lue citée, le scénario qui échoue, le remède proposé. Toute affirmation de
présence ou d'absence s'appuie sur une sortie `grep -nF` copiée. Termine par la liste des **axes exercés et non exercés** (un
« 0 finding » sans elle ne compte pas) et le décompte par sévérité.
