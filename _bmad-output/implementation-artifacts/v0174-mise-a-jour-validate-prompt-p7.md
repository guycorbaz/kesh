# Validation P7 — passe ciblée sur la remédiation P6 de v0174-mise-a-jour-1 (Sonnet, une lentille, lecture seule)

Tu es une **passe ciblée** (`CLAUDE.md`, *La passe ciblée*) : une seule lentille, contexte frais, braquée sur **le seul
commit** `a271e226` (remédiation de la validation P6 de la fiche **v0174-mise-a-jour-1** de Kesh, issue #847). Modèle :
**Sonnet**. Travaille en français soigné.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-0174m show a271e226 --stat
git -C /home/gcorbaz/devel/kesh-0174m diff 79577a69 a271e226
```

Lis le diff **aplati** ci-dessus (un seul diff, de `79577a69` à `a271e226`), puis les fichiers à leur état final :
`/home/gcorbaz/devel/kesh-0174m/_bmad-output/implementation-artifacts/v0174-mise-a-jour-1.md` et
`v0174-choix-autonomes-v0174m.md` (annotations de C-19 et C-31). Le rapport P6 que la remédiation traite :
`/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p6.md`.

## Ce que tu cherches — ce que la remédiation a pu casser

La sixième passe s'est faite par **retraits et qualifications seulement** (consigne de l'orchestrateur). Vérifie :

1. **V6-1** : « autre valeur » de l'échappatoire, qualifiée par le plan — point 6, AC-5, T11 (quatre cas) et M19 disent-ils
   la même chose ? Une base *à jour* avec `0.0.1` rend-elle `AJour` partout (AC-1, AC-5, point 6, T11) ?
2. **V6-2** : le plan *incohérent* sous échappatoire égale — point 6, § *Messages* (refus « sauf … incohérent »), phrase
   finale du § *Messages*, T3 (quatrième cas), M21, M22, registre C-31 annoté : cohérents ? `message_de_refus(&Refus,
   vers)` est-elle compatible avec la forme des variantes de `Refus` (V6-6) ? Le cas plan *illisible* sous échappatoire
   (`Contournee`, T11-bis) est-il resté intact ?
3. **V6-3** : la phrase « réutilisez la base existante, vidée » est-elle absente du § *Manuel* point 3, sans qu'aucune
   autre phrase de procédure n'ait été ajoutée ? Le point 3 garde-t-il le vidage de sécurité avant tout geste, et dit-il
   que l'application sur la machine même n'est pas éprouvée ?
4. **V6-6** : les variantes de `Refus` portent-elles toutes les données que les `{cause}` du § *Messages* citent
   (`dir`, `libre`, `seuil`, `chemin`, erreurs) ? `libre: Option<u64>` couvre-t-il « espace libre illisible » ?
5. **V6-4, V6-5, V6-7** : sept modules (liste et compte), annotation de C-19, coquille de M17.
6. Rien d'autre n'a changé de sens : relis le diff ligne à ligne.

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

`/home/gcorbaz/devel/kesh-gate-logs/v0174m-validate-p7.md`, en français :

1. En-tête : modèle, date, commit lu (`git -C /home/gcorbaz/devel/kesh-0174m rev-parse HEAD`).
2. Décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
3. Findings `V7-n` : sévérité, point (1 à 6), citation, **sortie `grep -nF` copiée**, correctif proposé.
4. **Points exercés et non exercés** (1 à 6, un par un, avec les commandes réellement lancées). Un « 0 finding » sans cette
   liste ne compte pas comme passe.
