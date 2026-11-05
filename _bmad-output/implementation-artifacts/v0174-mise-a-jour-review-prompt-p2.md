# Revue de code P2 — passe ciblée sur la remédiation P1 de v0174-mise-a-jour-1 (#847) (Sonnet, une lentille, lecture seule)

Tu es une **passe ciblée** (`CLAUDE.md`, *La passe ciblée*) : une seule lentille, contexte frais, braquée sur la
**remédiation de la revue de code P1** de la story **v0174-mise-a-jour-1** de Kesh (sauvegarde avant migration, version
minimale). Modèle : **Sonnet**. Français soigné.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-0174m log --oneline c66c126f..HEAD
git -C /home/gcorbaz/devel/kesh-0174m diff c66c126f HEAD
```

Lis le diff **aplati** (un seul diff, de `c66c126f` à `HEAD`). Il ne touche **aucune ligne de code de production** :
`crates/kesh-api/src/pre_upgrade_backup/tests.rs` (fichier `#![cfg(test)]`), `docs/manual/fr/admin-manual.tex` et son
PDF, la fiche `_bmad-output/implementation-artifacts/v0174-mise-a-jour-1.md` (Change Log, Dev Notes, Dev Agent Record,
§ *Recette*). Rapports traités : `/home/gcorbaz/devel/kesh-gate-logs/v0174m-review-p1-{B,E,A}.md`. Journaux :
`/home/gcorbaz/devel/kesh-gate-logs/v0174m/mut/{M5-bis,M9-bis,M23}.log`, `mut/campagne-C.log`, `p1rem-cible.log`.

## Ce que tu cherches — ce que la remédiation a pu casser

1. **T6-bis** (`t6_bis_frontiere_du_seuil`) : le seuil attendu est-il bien celui que calcule le module sur ces dossiers
   (vides : `seuil_d_espace(0, 0)`) ? Le second appel (espace égal) peut-il passer pour une mauvaise raison (dossier
   `pre-upgrade/` laissé par le premier appel, plus grande sauvegarde changée) ? M9-bis est-elle tuée par lui
   (`mut/M9-bis.log`) ?
2. **`BaseJetable`** (E1-6) : le `Drop` supprime-t-il la seconde base sur un échec d'assertion (panique) comme en fin de
   test ? Risque d'interblocage (`join` d'un fil qui bloque un autre exécuteur, connexion à une base déjà supprimée),
   de suppression d'une base qui n'est pas la sienne, de fuite si la création échoue après le `CREATE DATABASE` ?
   L'assertion de fin (`SCHEMATA`) prouve-t-elle la suppression ?
3. **`messages_de_refus`** (E1-5) : les trois assertions neuves (`Lecture`, `Ecriture`, `Tache`) correspondent-elles au
   `Display` réel de `DbError::NotFound` et de `BackupWriteError` (lis-les) ?
4. **M5-bis, M23** : tuées par l'assertion que la fiche nomme (lis les journaux) ? M23 a-t-elle bien porté (squash
   recompilé) ?
5. **Le manuel** — `.tex` **et PDF** aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`,
   puis `/usr/bin/grep -oF`) : les deux phrases ajoutées (table ou vue ajoutée à la main ; plan illisible / incohérent)
   disent-elles vrai contre le code (`kesh-db/src/backup.rs` `tables_du_schema_present`,
   `admin_backup/import.rs` « hors inventaire applicatif », `pre_upgrade_backup.rs` `message_de_refus`) ? Le renvoi
   `\S\ref{sec:revenir-version-precedente}` mène-t-il au vidage de sécurité ?
6. **La fiche** : le Dev Agent Record et le § *Recette* récrits disent-ils seulement ce qui a tourné (A1-3 : AC-8 jouée en
   partie) ? Les décomptes (campagne C 3/3, tests 19, gate ciblé 140/140) se recomptent-ils depuis les journaux et le
   source ? Une entrée passée du Change Log a-t-elle été réécrite (interdit) ? L'angle mort B1-3/E1-4 est-il exact
   (`git ls-tree` des tags) ?
7. Rien d'autre n'a changé de sens : relis le diff ligne à ligne.

⛔ **Ne demande pas ce que la version minimale a retiré** (écran, rétention, relecture, procédure de restauration neuve,
etc., fiche § *Retiré*) ; un défaut **dans** la remédiation est un finding, l'absence d'un élément retiré n'en est pas un.

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence s'appuie sur `/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou
`git -C … show HEAD:<fichier> | /usr/bin/grep -nF …`), **sortie copiée**. (`grep` est une fonction vers `ugrep`.)

## Sévérités

CRITICAL (migration possible sans sauvegarde ni refus, perte de données), HIGH (test qui ne prouve rien, manuel qui
promet faux sur la restauration), MEDIUM (incohérence code/manuel/fiche, mutation tuée pour une mauvaise raison,
décompte faux), LOW (rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout`/`switch`,
`git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui écrit ; pas de
`claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier temporaire**.
**Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/v0174m-review-p2.md` (jamais sous `target/`) : (1) décompte CRITICAL / HIGH /
MEDIUM / LOW recompté ; (2) findings (`R2-1`…, sévérité, fichier:ligne, sortie `grep -nF`, correctif) ; (3) **axes
exercés et non exercés** — un « 0 finding » sans cette liste ne compte pas.
