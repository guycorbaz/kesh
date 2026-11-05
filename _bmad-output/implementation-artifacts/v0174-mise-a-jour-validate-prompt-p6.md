# Validation P6 — passe ciblée sur la remédiation P5 de v0174-mise-a-jour-1 (Haiku, une lentille, lecture seule)

Tu es une **passe ciblée** (`CLAUDE.md`, *La passe ciblée*) : une seule lentille, contexte frais, braquée sur **le seul
commit** `ef1287f7` (remédiation de la validation P5 de la fiche **v0174-mise-a-jour-1** de Kesh, issue #847). Modèle :
**Haiku**. Travaille en français soigné.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-0174m show ef1287f7 --stat
git -C /home/gcorbaz/devel/kesh-0174m diff a704585f ef1287f7
```

Lis le diff **aplati** ci-dessus (un seul diff, de `a704585f` à `ef1287f7`), puis les fichiers à leur état final :
`/home/gcorbaz/devel/kesh-0174m/_bmad-output/implementation-artifacts/v0174-mise-a-jour-1.md` et
`v0174-choix-autonomes-v0174m.md` (C-v0174m-33, C-v0174m-34, annotation de C-24). Le rapport P5 que la remédiation traite :
`/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p5.md`.

## Ce que tu cherches — ce que la remédiation a pu casser

1. **Cohérence interne de la fiche après le patch.** La définition d'`Issue` (Conception) et chacun de ses emplois : AC-4,
   AC-5, T1, T7-bis, T11, M17 à M20, § *Messages*, point 8 (*Journal*), point 4. Un emploi resté sur l'ancienne forme
   (`Ecrite(PathBuf)`, « sans avertissement », `Refus::Ecriture` sur la synchronisation) ?
2. **Chaque mutation ajoutée ou récrite (M6, M17, M18, M19, M20) est-elle tuée par le test nommé ?** Déroule le test contre
   le code muté décrit.
3. **V5-4** : `Refus::Plan` « incohérent » sous `Aligned` — l'énoncé du point 5, AC-4, T3, M20 et la `{cause}` du § *Messages*
   disent-ils la même chose ? T3 reste-t-il montable (base vide : `check_downgrade_protection` y rendrait `FreshInstall` ;
   le test passe `Aligned` lui-même au point d'entrée) ?
4. **V5-5** : `sync_dir` à corps par défaut ; `plan_de_migration` et `ecrire_la_sauvegarde` (signature, `de` en argument) ;
   l'extension de la garde lexicale à `#![cfg(test)]` — est-elle prescrite sans contredire `export.rs:489-520`
   (`git show 7bd4d5c0:crates/kesh-api/src/admin_backup/export.rs`) ?
5. **V5-2** : les deux phrases du § *Manuel* point 3 (machine même, `docker compose down` sans `-v`, base existante vidée)
   — « réutilisez la base existante, vidée » réintroduit-il une procédure de restauration neuve que C-v0174m-32 exclut, ou un
   geste destructeur avant le vidage de sécurité ? Le renvoi de `:3758` existe-t-il
   (`git show 7bd4d5c0:docs/manual/fr/admin-manual.tex | grep -n Rollback`) ?
6. **§ *Dérogation règle de splitting*** : le compte des modules est-il exact au regard du diff de conception ?
7. **Registre** : C-33 et C-34 complets (contexte, retenu, écartées, réversibilité) ; liste « en vigueur » à jour.

Ne cherche pas au-delà de ce commit, sauf pour vérifier une affirmation qu'il fait.

## ⛔ Ground-truth obligatoire

Toute affirmation de **présence ou d'absence** d'un texte (dans la fiche, le registre ou le code) s'appuie sur une
commande `grep -nF "<chaîne exacte>" <fichier>` (ou `git show 7bd4d5c0:<fichier> | grep -nF "<chaîne>"` pour le code),
**dont tu copies la sortie** dans le rapport. Pour un bloc de plusieurs lignes, `grep -nF -A5`. Un finding CRITICAL ou HIGH
qui affirme qu'un patch « n'a pas été appliqué » ou qu'un texte « manque » sans sortie `grep -nF` copiée ne compte pas.
(`grep` est ici une fonction vers `ugrep` : utilise **`/usr/bin/grep -nF`**.)

## Sévérités

CRITICAL (migration possible sans sauvegarde ni refus, perte de données), HIGH (AC faux ou intestable), MEDIUM
(incohérence entre AC, tests, mutations et conception ; mutation non tuée ; manuel faux), LOW (rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout`/`switch` (sous
aucune forme), `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui
écrit ; pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, ni de redirection `>` hors de ton rapport.
**Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p6.md`, en français :

1. En-tête : modèle, date, commit lu (`git -C /home/gcorbaz/devel/kesh-0174m rev-parse HEAD`).
2. Décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
3. Findings `V6-n` : sévérité, point (1 à 7), citation, **sortie `grep -nF` copiée**, correctif proposé.
4. **Points exercés et non exercés** (1 à 7, un par un, avec les commandes réellement lancées). Un « 0 finding » sans cette
   liste ne compte pas comme passe.
