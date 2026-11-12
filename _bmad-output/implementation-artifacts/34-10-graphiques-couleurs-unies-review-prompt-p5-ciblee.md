# Revue de code P5 ciblée — 34-10 (#970), une seule lentille, sur la remédiation P4 (Sonnet, lecture seule)

Tu es la **lentille unique** de la passe 5 ciblée de `bmad-code-review` de la story **34-10** de Kesh (issue #970 : graphiques
de l'accueil en couleurs unies, anneau des postes de charges). Modèle : **Sonnet**. Contexte frais. Travaille en français soigné.

**Question unique** : la remédiation de la passe ciblée P4 introduit-elle un défaut, une régression ou une contradiction ? Tu ne
relis pas la story entière : ce qui reste à relire est la dernière remédiation.

## Le périmètre — le seul diff de la remédiation P4

```sh
git -C /home/gcorbaz/devel/kesh-e3410 show 625eb797 --stat
git -C /home/gcorbaz/devel/kesh-e3410 show 625eb797
git -C /home/gcorbaz/devel/kesh-e3410 log --oneline 625eb797..HEAD   # comptes rendus qui suivent
```

Fichiers : `frontend/src/lib/features/homepage/ExpensePieCard.svelte` (`hasPosts` exige au moins une part dessinable),
`HomepageCharts.test.ts` (un test), les registres `e34-choix-autonomes-e342.md` (annotation de C-e342-22) et
`e34-choix-autonomes-e3410.md` (C-e3410-15 reformulée), la fiche `34-10-graphiques-couleurs-unies.md` (annotations de la ligne
du CHANGELOG de la validation P2 et de la dispense d'E2E de la P3, entrée « passe ciblée P4 » du Change Log).

Rapport P4 : `/home/gcorbaz/devel/kesh-gate-logs/e3410-review-p4-ciblee.md`. Journaux dans
`/home/gcorbaz/devel/kesh-gate-logs/e3410/` : `rouges.log` (section « Remédiation P4 ciblée »), `mut/MV17-anneau-vide.log`,
gates `*-p4.log`, E2E `e2e-r5.log`.

## Axes, à braquer sur ce diff seulement

1. **`hasPosts`** : il gouverne désormais l'anneau, le tableau des postes, le `<desc>` **et** la phrase de la base
   (`baseDiffers`). Le changement masque-t-il à tort le tableau ou une note dans un cas atteignable (postes positifs, « Autres »
   seul positif, base nulle) ? `slices` étant dérivé avant `hasPosts`, l'ordre des `$derived` est-il sain ? Le test neuf
   prouve-t-il l'état vide et l'absence d'anneau, et la mutation MV17 est-elle tuée pour la bonne raison ?
2. **Les annotations** : C-e342-22, C-e3410-15 et la fiche disent-elles la même chose, datée, sans réécrire une entrée passée ?
   Le décompte cité (« dix-sept `### Modifié` jusqu'à la 0.17.5 ») est-il juste (`git show cdf13639:CHANGELOG.md`) ? La fiche 34-3
   n'est pas touchée (consigne de l'orchestrateur) : vérifie-le.
3. **L'E2E r5** : joué au dernier commit de code, recette entière (runner avec `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`) ? Échecs
   nommés un par un contre `docs/testing.md` § « Les échecs attendus » ? La fiche affirme-t-elle seulement ce qui a tourné ?

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C /home/gcorbaz/devel/kesh-e3410 show HEAD:<fichier> | /usr/bin/grep -nF …`),
**dont tu copies la sortie**. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (montant faux affiché comme total, lien vers un mauvais compte), HIGH (part mal placée ou de mauvaise couleur, test qui
ne prouve rien), MEDIUM (incohérence code/fiche/registre, mutation tuée pour une mauvaise raison), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `node`, `make`, `latexmk`, `docker`, `curl`,
`sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ou de `kesh-gate-logs/` ; pas de
`git checkout`/`switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui
écrit ; pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier
temporaire**. **Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e3410-review-p5-ciblee.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`C5-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
