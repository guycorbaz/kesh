# Revue de code P1 — Story 49-1d « Types de lignes — recette de montée » (#1242)

*Prompt de la passe 1 de `bmad-code-review`, trois lentilles **Opus** en contexte frais : **B** (Blind Hunter), **E** (Edge
Case Hunter), **A** (Acceptance Auditor). Une première passe n'est jamais ciblée et ne va pas à Haiku.*

## Ce que tu relis

- **Worktree** : `/home/gcorbaz/devel/kesh-49-1d`, branche `story/49-1d-types-de-lignes-recette-montee`.
- **Diff** : `git -C /home/gcorbaz/devel/kesh-49-1d diff origin/epic/49-types-de-lignes-facture...HEAD -- scripts crates docs`
  (trois points : depuis la base de fusion). Code de la story : `scripts/recette-montee.sh`,
  `scripts/tests/recette-montee.test.sh`, `crates/kesh-db/tests/recette_montee_script.rs` (une ligne et la doc du module) ;
  documentation : `docs/testing.md` (§ *La recette de montée et de retour*), `docs/manual/fr/admin-manual.tex` (section
  « Passer à la 0.20.0 », paragraphe « Compatibilité » du § *Exporter (sauvegarde complète)*).
- **Fiche** : `_bmad-output/implementation-artifacts/49-1d-types-de-lignes-recette-montee.md` (D1 à D11, AC1 à AC16, S1 à
  S14, T0 à T12, section *Allègement*, Dev Agent Record) ; registre :
  `_bmad-output/implementation-artifacts/e0200-choix-autonomes-e491d.md` (C-e491d-1 à C-e491d-5).
- **Commits** : `git log --oneline origin/epic/49-types-de-lignes-facture..HEAD`. Code : `e137ae1e9` (pause du 2026-11-20),
  `a5cf95297`, `a02b79bec`, `bd7e72ce8` ; le reste est documentation (fiche, registre, `docs/testing.md`, manuel). Le Dev Agent
  Record (§ *Reprise du 2026-11-24*) donne pour chaque jeu son commit, son journal et ses chiffres.
- **Journaux de gate** : `test-fast-ci.log`, `gate-ci.log` (+ `.entete`, `.out`), `mesure-nextest-1.log`, `-2.log`,
  `gates-review.log` ; mutations : `mut/juge.log` (verdict final), `mut/resume.log`, `mut/resume-2.log`,
  `mut/juge-tirage-parallele.log` (tirage parallèle, onze journaux tronqués, rejoués).
- **Scripts hors dépôt, relus comme du code** (AC12, D7, D11 ; lecture seule) :
  `/home/gcorbaz/devel/kesh-gate-logs/rel0200/verifs-0200.sh` (crochet `--apres`),
  `/home/gcorbaz/devel/kesh-gate-logs/rel0200/recette-complement-rel0200.sh` (retour par `pre-upgrade/`),
  `/home/gcorbaz/devel/kesh-gate-logs/rel0200/perte-assumee.sh` (contrôle de la perte assumée),
  `/home/gcorbaz/devel/kesh-gate-logs/e491d/mut/mutations.py` (mutations du test shell),
  `/home/gcorbaz/devel/kesh-gate-logs/e491d/t0/sonde-t0.sh` (sonde T0).
- **Journaux** (lecture) : `/home/gcorbaz/devel/kesh-gate-logs/e491d/` — `t0/sonde-t0.log` (premier tirage : QR-IBAN) et
  `t0/sonde-t0-2.log` ; `dev/test-shell-*.log` ; `mut/resume.log` et `mut/rouge-*.log` ; `recette-reelle.log`,
  `recette-reelle-rouge-diff.log`, `recette-reelle-rouge-projection.log`, `trame-hors-script.log` ;
  `recette-complement.log`, `recette-complement-rouge.log` ; `dev/perte-assumee-*.log` ; `dev/make-fr.log` ; les journaux
  des gates nommés au Dev Agent Record (chacun avec son commit et ses chiffres). Les dossiers `$D` des recettes (sous
  `/tmp/kesh-recette.*`) sont nommés à la dernière ligne de chaque journal.
- **Manuel compilé** : le PDF du dépôt n'est pas commité dans une story ; une copie compilée depuis le `.tex` de la branche
  est dans `/home/gcorbaz/devel/kesh-gate-logs/e491d/manuel/admin-manual.pdf`.
- **Captures** : sans objet — la story n'ajoute ni ne change aucun contrôle visible (AC16). Vérifie-le au diff
  (`-- frontend` vide).

## Le contexte qui commande la revue

Kesh est **en production** : Guy tient ses livres réels en **0.19.1** sur le NAS et a déjà remis sur papier de vraies
factures à un locataire. Cette story est **le gate de la migration** de la 49-1a (`20261118000002_invoice_line_types`) : la
recette doit **prouver**, depuis l'image publiée `gcorbaz/kesh:0.19.1`, qu'aucune facture ni aucun avoir existant ne change à
la montée — lignes, totaux, TVA, écritures, numéros, règlements, rapport TVA, PDF archivés à l'octet, PDF rendus de nouveau
au même dessin — et que le retour par la sauvegarde d'avant la mise à jour fonctionne. **Une preuve qui peut passer à vide
est le défaut le plus grave de cette revue** : cherche-la partout.

## Axes communs (toutes les lentilles)

1. **Comparaisons qui passent à vide** : la projection de `--diff-attendu` (tables et `api`) peut-elle rendre « identique »
   sans rien comparer (fichier vide des deux côtés, colonnes relevées vides, `jq` qui sort sans document, `diff` de deux
   fichiers absents) ? La phase 6 peut-elle compter une paire sans avoir comparé ses trames (sortie vide de `mutool` des deux
   côtés : `md5sum` de rien égale `md5sum` de rien) ? Le crochet peut-il passer sur une table vide ? Le script
   complémentaire peut-il passer sans avoir comparé une table ?
2. **Le périmètre fermé du script** (Dev Notes, *Ce qui doit rester vrai*) : relance `env -i`, `HOME` neutralisé, aucune
   écriture hors de `$D`, aucun montage, conteneurs du jeton seuls, `docker cp` du seul conteneur du jeton, jamais
   `… | grep -q` sous `pipefail`. `KESH_MUTOOL` est-elle la seule variable neuve qui traverse ?
3. **L'ordre des phases et des PDF** : F2, F4, F6 téléchargés en phase 1 les figeraient ; les références de la phase 4
   sont-elles prises après son empreinte et avant l'arrêt de l'image ? Les PDF de la phase 2 après l'empreinte et le crochet ?
4. **Les factures réelles** : le peuplement ressemble-t-il à l'usage réel (factures payées, impayées, groupées, annulées,
   figées, jamais téléchargées) ? Les deux comptes bancaires (C-e491d-2) changent-ils ce que la recette prouve ?
5. **La version de départ 0.19.1** (C-e491d-1) : tous les sites opératoires (scripts, fiche, `docs/testing.md`, manuel)
   disent-ils 0.19.1, et les faits « établis par le code » sont-ils vrais au tag `v0.19.1` ?

## Lentille B — Blind Hunter

Diff et scripts hors dépôt, sans la fiche. Défauts de shell : citation, `set -e`/`pipefail` et leurs exceptions, code de
retour masqué par une substitution (`x=$(…) || …`, `local x=$(…)`), `awk`/`jq`/`sed` fragiles, globbing, fichiers relus
avant d'être écrits, condition de course entre un téléchargement et un arrêt, DRY (fonctions en double), doc-comments faux,
factices qui prouvent moins qu'ils ne disent (une assertion dont les deux côtés sortent du même factice), mutation
équivalente non déclarée.

## Lentille E — Edge Case Hunter

Chaque branche et chaque borne : une désignation qui contient une tabulation ou un saut de ligne (mode batch de `mariadb`),
une table d'avant vide, une colonne ajoutée **au milieu** d'une table, une colonne retirée, un document `api` de longueur
différente, `null` devenu objet, une facture à deux pages (la trame page à page), `mutool` qui écrit sur la sortie d'erreur
mais rend 0, un PDF tronqué qui commence par `%PDF-`, un `410` d'un autre code, un regel qui rend 200 sans PDF, une recette
qui passe minuit (dates du jour, `isOverdue`), un import qui laisse des pièces manquantes, `--import` absent (le décompte des
paires), le crochet sur une base où la colonne n'existe pas, le script complémentaire sur un `$D` sans `pre-upgrade/` ou
avec deux sauvegardes.

## Lentille A — Acceptance Auditor

AC1 à AC16 contre le code et les journaux, un par un, avec la ligne lue citée ; S1 à S14 présents et chacun avec sa mutation
tuée (`mut/resume.log`, `mut/rouge-*.log`) ou déclarée équivalente ; D1 à D11 respectées, et les écarts au registre justes ;
T0 : chaque surprise écrite au Dev Agent Record avant le code qui en dépend ; AC10 : la recette réelle verte, R1 et R2
rouges, la trame F2 contre F2 égale et F2 contre F4 différente ; AC11 : le script complémentaire vert et son rouge ;
AC12 et AC13 : le rejeu décrit à `docs/testing.md` ; AC15 : `git diff --stat origin/epic/49-types-de-lignes-facture...HEAD --
crates frontend` ne nomme que `crates/kesh-db/tests/recette_montee_script.rs`, et `ls crates/kesh-db/migrations/*.sql | wc -l`
rend 96 ; AC16 : chaque gate du Dev Agent Record a son journal et son commit. Recompte chaque décompte écrit (assertions,
mutations, paires, tables) depuis sa source.

## Le manuel (obligatoire)

Vérifie le manuel contre le code et contre les journaux de la recette, **PDF compris**, aplati avant de greper, par cette
seule forme :
`pdftotext /home/gcorbaz/devel/kesh-gate-logs/e491d/manuel/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`
(l'apostrophe du PDF est typographique). Points à vérifier, section « Passer à la 0.20.0 » et « Compatibilité » :
la sauvegarde complète **en tête** ; « La 0.20.0 se pose sur la 0.19.1 » ; le total des migrations (96) ; aucune valeur
existante réécrite (`ADD COLUMN … DEFAULT 'priced'`, `crates/kesh-db/migrations/20261118000002_invoice_line_types.sql`) ;
`kesh_version_min_required` à `0.17.6` ; le nom `-0.19.1-to-0.20.0.keshbackup` (établi par `crates/kesh-api/src/main.rs` et
`pre_upgrade_backup.rs`, non constaté à la tête de la story) ; `KESH_SKIP_PRE_UPGRADE_BACKUP=0.20.0` ; le message de sqlx du
refus de la 0.19.1 (constaté par la phase 3, journal `image-refus.log`) ; le refus d'une sauvegarde 0.20.0 dit **établi par le
code** (`git show v0.19.1:crates/kesh-api/src/admin_backup/import.rs`, `:615-633`), code `IMPORT_SCHEMA_MISMATCH`, texte de
l'écran (`git show v0.19.1:frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte`, `:189-193`, et la clé fr-CH) ;
**aucun** `DELETE` prescrit, ni le mot *dirty*, dans la section 0.20.0 ; le message « partially applied » cité exact
(`sqlx-core-0.8.6/src/migrate/error.rs:39`) ; « même dessin » et ce qu'il ne couvre pas ; ce qui est dit « constaté »
l'a-t-il été par un journal de cette story ? La 49-4b n'est pas dans la release (Epic 52) : la phrase sur les désignations
longues le reflète-t-elle ? Une phrase devenue fausse ailleurs au manuel (sections 0.19.1, 0.19.0, *La sauvegarde avant mise
à jour*, *Revenir à la version précédente*) ?

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo` (build, test, run, fmt, nextest), ni `npm`/`npx`,
ni `make`, ni `scripts/prepare-release.sh`, `scripts/recette-montee.sh`, `scripts/tests/recette-montee.test.sh`,
`scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `sqlx`, ni aucun script de `kesh-gate-logs/rel0200/`
ou de `kesh-gate-logs/e491d/` (`verifs-0200.sh`, `recette-complement-rel0200.sh`, `perte-assumee.sh`, `mut/mutations.py`,
`t0/sonde-t0.sh`), ni `docker`, ni `mariadb`, ni `mutool` vers un fichier, ni `pdftotext` vers un fichier (seulement vers
`-`), ni `git commit`, `git checkout`, `git restore`, `git stash`, `git worktree`, `git merge`, ni `rm`, `mv`, `cp`,
`sed -i`, ni redirection vers un fichier, ni fichier temporaire. Seule écriture permise : ton rapport. `grep` est une
fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/e491d/review-p1-{B,E,A}.md` selon ta lentille. Chaque finding : sévérité
(CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, la ligne lue citée, le scénario qui échoue, le remède proposé. Toute
affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée. Termine par la liste des **axes exercés et
non exercés** (un « 0 finding » sans elle ne compte pas) et le décompte par sévérité.
