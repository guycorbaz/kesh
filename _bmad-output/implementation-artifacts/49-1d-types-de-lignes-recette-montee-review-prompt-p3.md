# Revue de code P3 — Story 49-1d « Types de lignes — recette de montée » (#1242)

*Prompt de la passe 3 de `bmad-code-review`, **passe complète**, trois lentilles **Sonnet** en contexte frais : **B** (Blind
Hunter), **E** (Edge Case Hunter), **A** (Acceptance Auditor). P1 (Sonnet) : 0/0/2/16 ; P2 (Opus) : 0/0/1/14
(`/home/gcorbaz/devel/kesh-gate-logs/e491d/review-p{1,2}-{B,E,A}.md`). Les deux MEDIUM étaient de **la même classe** : un
garde-fou sans scénario ni mutation. La remédiation de la P2 ne les corrige pas site par site : elle dresse l'**inventaire clos**
de toutes les sorties d'échec (Dev Agent Record, § *Inventaire clos des sorties d'échec*). **Ta tâche première est de le
vérifier en le recalculant toi-même.** Relis tout le diff, non la seule remédiation.*

## Ce que tu relis

- **Worktree** : `/home/gcorbaz/devel/kesh-49-1d`, branche `story/49-1d-types-de-lignes-recette-montee`.
- **Diff** : `git -C /home/gcorbaz/devel/kesh-49-1d diff origin/epic/49-types-de-lignes-facture...HEAD -- scripts crates docs`.
  Code : `scripts/recette-montee.sh`, `scripts/tests/recette-montee.test.sh`, `crates/kesh-db/tests/recette_montee_script.rs` ;
  documentation : `docs/testing.md` (§ *La recette de montée et de retour*), `docs/manual/fr/admin-manual.tex` (« Passer à la
  0.20.0 », « Compatibilité »).
- **Commits** : `git log --oneline origin/epic/49-types-de-lignes-facture..HEAD` ; code de la remédiation P2 : `d2416c596`.
- **Fiche** et **registre** : `_bmad-output/implementation-artifacts/49-1d-types-de-lignes-recette-montee.md`,
  `e0200-choix-autonomes-e491d.md`.
- **Scripts hors dépôt, relus comme du code** (lecture seule) : `kesh-gate-logs/rel0200/verifs-0200.sh`,
  `verifs-0200.test.sh`, `verifs-0200.mut.sh`, `recette-complement-rel0200.sh`, `perte-assumee.sh` ;
  `kesh-gate-logs/e491d/mut/mutations.py` ; `kesh-gate-logs/e491d/inventaire/inventaire.py` (le classeur de l'inventaire) ;
  `kesh-gate-logs/e491d/dev/recette-p2.sh`, `complement-rouge-p2.sh`.
- **Journaux** sous `/home/gcorbaz/devel/kesh-gate-logs/e491d/` : `dev/test-shell-*.log`, `mut/juge.log`, `mut/resume-4.log`,
  `mut/rouge-*.log`, `crochet-test.log`, `crochet-mutations.log`, `recette-reelle-p2.log`, `recette-reelle-rouge-diff-p2.log`,
  `recette-complement-p2.log`, `recette-complement-rouge-p2.log`, `recette-complement-rouge2-p2.log`, `inventaire/inventaire.md`,
  et les journaux de gate nommés au Dev Agent Record.
- **Manuel compilé** : `/home/gcorbaz/devel/kesh-gate-logs/e491d/manuel/admin-manual.pdf`.
- **Captures** : sans objet (`git diff --stat origin/epic/49-types-de-lignes-facture...HEAD -- frontend` vide).

## Le contexte qui commande la revue

Kesh est **en production** (0.19.1). Cette story est le **gate de la migration** `20261118000002` : la recette doit prouver,
depuis l'image publiée `gcorbaz/kesh:0.19.1`, qu'aucune facture ni aucun avoir existant ne change à la montée ni à l'import,
et que le retour par la sauvegarde fonctionne. **Un garde-fou qu'aucun scénario ni aucune mutation n'éprouve, ou un angle
mort assumé sans raison valable, est le défaut que cette passe doit trouver.**

## Axe premier, toutes lentilles — l'inventaire clos

1. **Recalcule l'inventaire par ta propre commande**, sans reprendre celle du Dev Agent Record : par exemple
   `/usr/bin/grep -nE '…' scripts/recette-montee.sh` sur les formes que **tu** juges être des sorties d'échec (`echec`,
   `return 1`, `exit` non nul, `refus`, appels de fonctions qui échouent, conditions qui lisent un code). Prouve ton extracteur
   par une sonde positive. Compare **ton** décompte au sien (188 + 5 + 24) : tout site que tu trouves et qu'il n'a pas est un
   finding ; tout site qu'il a et que tu n'as pas, dis pourquoi.
2. Pour chaque site de classe **C** (couvert), vérifie que le scénario nommé existe au test et fait rougir **ce** site (son
   message), et que la mutation nommée est **TUÉE** dans `mut/juge.log` **pour la bonne raison** (l'assertion visée, non un
   échec collatéral) — lis `mut/rouge-<nom>.log`. Pour le crochet, `crochet-test.log` et `crochet-mutations.log`.
3. Pour chaque site **A** (angle mort assumé), juge la raison : un site qui garde la **donnée** d'une facture ou d'un avoir
   (lignes, totaux, TVA, écritures, PDF) et que la raison ne justifie pas est un finding MEDIUM au moins.
4. Pour chaque site **S** (scénario sans mutation, Story 33-10), vérifie que le scénario existe.
5. Les factices sont-ils **cohérents** avec une vraie base (P2, B-P2-1 : un factice impossible prouve moins) ? Les nouveaux
   réglages (`TABLE_VIDE`, `ECART_RETOUR`, `ECART_IMPORT`, `REGEL_SHA_ALTERE`, `TEXTE_ECART`, `PAGES_RENDU`,
   `MUTOOL_AVERTISSEMENT`) et le crochet de S16d.

## Lentille B — Blind Hunter

Shell : `set -e`/`pipefail` et leurs exceptions, `| grep -q` sous `pipefail` (dans **tous** les scripts, outillage de test et de
mutation compris), code masqué par une substitution, citation, DRY, `jouer_crochet` (portée de `PHASE_APRES`, journal par phase),
le classeur `inventaire.py` (une règle qui classerait à tort : première règle gagnante, contexte de la ligne précédente).

## Lentille E — Edge Case Hunter

Les bornes laissées ou déplacées : table vide **nommée** et **non nommée**, import qui change une valeur d'une table **non**
nommée et d'`api`, crochet en phase 5 sans `--import`, `mutool` qui écrit l'avis ICC plusieurs fois ou avec une variante,
recette à cheval sur minuit (phases 2, 4, 5), une facture de deux pages, le script complémentaire sur un `$D` incomplet.

## Lentille A — Acceptance Auditor

AC1 à AC16 contre le code et les journaux, ligne lue citée ; sort de chacun des quinze findings de la P2 (dont l'arbitrage A4
de la P1, « sans changement », écrit au Change Log) ; AC11 rejoué au dernier commit de code (`recette-complement-p2.log`) ;
AC16 : `test-fast --ci` et `gate-ci kesh_ci_e491d` au dernier commit de code ; recompte chaque chiffre du Dev Agent Record
(assertions, mutations, sites par classe, tables, paires).

## Le manuel et `docs/testing.md` (obligatoire)

Vérifie le manuel contre le code et les journaux, **PDF compris**, aplati avant de greper, par cette seule forme :
`pdftotext /home/gcorbaz/devel/kesh-gate-logs/e491d/manuel/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
Section « Passer à la 0.20.0 » : sauvegarde en tête, 0.19.1 partout, 96 migrations, `min_required` 0.17.6, nom
`-0.19.1-to-0.20.0` (établi, non constaté), message de sqlx avec ses accents graves, refus d'une sauvegarde 0.20.0 « établi par
le code », aucun `DELETE` ni *dirty*, « même dessin » et ses limites, « constaté » qualifié. `docs/testing.md` : crochet joué en
phases 2 et 5, table de l'empreinte non vide, seul l'avis ICC toléré, minuit (phases 2, 4, 5), scénarios « témoins » et
inventaire, « Ce que la recette ne prouve pas ».

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`/`npx`, ni `make`, ni
`scripts/prepare-release.sh`, `scripts/recette-montee.sh`, `scripts/tests/recette-montee.test.sh`, `scripts/test-fast.sh`,
`scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `sqlx`, ni aucun script de `kesh-gate-logs/rel0200/` ou de `kesh-gate-logs/e491d/`
(`verifs-0200.sh`, `verifs-0200.test.sh`, `verifs-0200.mut.sh`, `recette-complement-rel0200.sh`, `perte-assumee.sh`,
`mut/mutations.py` — même `--juger` —, `inventaire/inventaire.py`, `dev/*.sh`, `gates-review*.sh`, `bases-neuves-e491d.sh` — il
efface les bases —, `test-fast-ci-e491d.sh`, `gate-ci-e491d.sh`), ni `python3` sur un script du dépôt ou de `kesh-gate-logs`,
ni `docker`, `mariadb`, `jq` ou `mutool` vers un fichier, ni `pdftotext` vers un fichier (seulement vers `-`), ni `git commit`,
`git checkout`, `git restore`, `git stash`, `git worktree`, `git merge`, ni `rm`, `mv`, `cp`, `sed -i`, `flock`, ni redirection
vers un fichier, ni fichier temporaire. Seule écriture permise : ton rapport. `grep` est une fonction vers `ugrep` : emploie
`/usr/bin/grep` ou `git grep`.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/e491d/review-p3-{B,E,A}.md` selon ta lentille. Chaque finding : sévérité, fichier et
ligne, la ligne lue citée, le scénario qui échoue, le remède proposé ; toute présence ou absence appuyée sur une sortie
`grep -nF` copiée. **Ta commande d'inventaire, sa sonde et ton décompte figurent en tête du rapport.** Termine par la liste des
**axes exercés et non exercés** (un « 0 finding » sans elle ne compte pas) et le décompte par sévérité.
