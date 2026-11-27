# Story 58-2 — prompt de la revue de code P2 (Sonnet, passe complète, contexte frais)

Tu conduis la **revue de code P2 de la story 58-2 de Kesh** (logiciel de comptabilité suisse en production ; Epic 58, nettoyage : neuf issues de tests — #592, #732, #736, #1060, #1062, #1063, #1064, #1066, #1433 ; **aucun code de production**), avec le workflow BMAD **`bmad-code-review`** (invoque la compétence et suis-la, sans attendre de confirmation humaine aux points d'arrêt). Contexte frais : tu n'as pas vu la P1 ; ne lis son rapport qu'**après** ta propre lecture du diff, pour vérifier ses remédiations.

- Worktree `/home/gcorbaz/devel/kesh-58-2`, branche `story/58-2-comptes-designes-tests-revue` ; base de comparaison `90192c7b3` (branche d'epic). Diff : `git diff 90192c7b3 HEAD`.
- Fiche : `_bmad-output/implementation-artifacts/58-2-comptes-designes-tests-revue.md` (AC, Décisions, Dev Agent Record, Change Log).
- Journal des mutations : `$GATE_LOGS/e58/s582-mutations.md` (`$GATE_LOGS` = `/home/gcorbaz/devel/kesh-gate-logs`) ; scripts du développeur : `/tmp/s582/mut.py`, `/tmp/s582/fmut.py`.
- Rapport P1 (Opus, 0/0/1/4) : `$GATE_LOGS/e58/58-2-revue-p1.md`.
- Lis le `CLAUDE.md` du worktree : *Review Iteration Rule*, *Recompter*, *Propagation post-patch*, *Inventorier les sites NON RÉSOLUS*, la règle des LOW (§ *Priorités des défauts*), et le principe « un test qui compare deux sorties d'un même extracteur ne prouve rien ».

## Axes obligatoires — chacun déclaré exercé ou non exercé

1. **Remédiations de la P1** : R1-1 (MEDIUM : AC5 et AC14 sans preuve au Dev Agent Record, phrase « aucune … omise » fausse) et les LOW R1-2 à R1-5, s'ils ont été traités. Vérifie au journal et à la fiche ; recompte depuis la source tout nombre écrit (mutations distinctes contre exécutions, `grep -c` et son périmètre).
2. **Chaque AC** (AC1 à AC11, AC13, AC14 ; AC12 sortie avec #1127) contre le test écrit : le test prouve-t-il ce que l'issue demande (`gh issue view <N>`, lecture seule), et rougirait-il **pour la bonne raison** (l'assertion visée, non une panique de montage) ?
3. **Mutations** : rejoues-en au moins **trois** que la P1 n'a pas rejouées (la P1 a rejoué ac3-i, ac3-ii, ac13a, ac8 et la mutation d'AC5) — par exemple ac4, ac7-rev, ac10-vat, ac13b, une F2 frontend. Lis la **raison** du rouge dans la sortie complète. Sous `flock /tmp/station-heavy.lock flock $GATE_LOGS/.gate.lock` (Rust, `scripts/mem-guard.sh`, base `kesh_e582`) ou `flock $GATE_LOGS/.gate-front.lock` (vitest d'un fichier). Sauvegarde hors dépôt (`/tmp/…`), **restauration par copie puis `touch`**, `git status --porcelain` vide après chacune et à la fin.
4. **Survivantes déclarées** (AC3 permutation dans `chart_designated_accounts`, AC8 `.then(setting)` renvoyée à #1059, AC9 rollback) : vraiment inatteignables par un test de la story, ou seulement hors périmètre ?
5. **AC14** : rejoue `git diff --name-only 90192c7b3 HEAD` et le filtre à liste blanche de la fiche ; aucun `crates/*/src/*`, catalogue (`.ftl`) ni manuel.
6. **Tests instables** (horloge, ordre, concurrence), **noms et doc-comments**.
7. **Manuel** : aucun fichier du manuel n'est touché ; vérifie seulement qu'aucune phrase de `docs/manual/` ne devient fausse, **PDF compris**, aplati avant de greper : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`.

Pour tout finding CRITICAL ou HIGH qui affirme une absence ou un anti-pattern : preuve `grep -nF` ou extrait lu, cité.

## Interdits

- N'applique **aucun correctif** ; n'écris rien hors de `/tmp` et de `$GATE_LOGS/e58/` (sauf le prompt de passe suivante ci-dessous).
- Ni `gh` en écriture, ni `scripts/prepare-release.sh`, ni aucun script mutant du dépôt, ni `docker`, ni `GRANT`/`REVOKE`, ni redémarrage du conteneur MariaDB ; rien sous `~/kesh/` ; jamais `pkill`, `killall` ni `kill` par motif.

## Livrables

- Rapport : `$GATE_LOGS/e58/58-2-revue-p2.md` — findings classés (CRITICAL/HIGH/MEDIUM/LOW) avec preuve, axes **exercés et non exercés** (un « 0 finding » sans cette liste ne compte pas).
- S'il reste un finding au-dessus de LOW : prompt de la passe suivante (`58-2-comptes-designes-tests-revue-revue-prompt-p3.md`, même dossier, modèle **Opus** pour une passe complète), commité seul (`git commit -- <chemin>`, message en français, `refs` des neuf issues, aucun mot-clé de fermeture), et poussé sur la seule branche de la story.
- Réponse finale en français : décompte, une ligne par finding MEDIUM ou plus, axes exercés et non exercés, chemin du rapport.
