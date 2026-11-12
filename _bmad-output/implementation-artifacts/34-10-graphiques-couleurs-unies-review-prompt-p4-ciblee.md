# Revue de code P4 ciblée — 34-10 (#970), une seule lentille, sur la remédiation P3 (lecture seule)

Tu es la **lentille unique** de la passe 4 ciblée de `bmad-code-review` de la story **34-10** de Kesh (issue #970 : graphiques
de l'accueil en couleurs unies, anneau des postes de charges). Contexte frais. Travaille en français soigné.

**Question unique** : la remédiation de la revue P3 introduit-elle un défaut, une régression ou une contradiction ? Tu ne relis
pas la story entière : la P3 n'a rien rendu au-dessus de LOW, et ce qui reste à relire est la dernière remédiation.

## Le périmètre — le seul diff de la remédiation P3

```sh
git -C /home/gcorbaz/devel/kesh-e3410 show ce42fd4a --stat
git -C /home/gcorbaz/devel/kesh-e3410 show ce42fd4a
```

Fichiers : `frontend/src/lib/features/homepage/homepage-charts.ts` (`PIE_GEOMETRY.innerR` écrit 57.6 ; `pieSlices` : rang et
couleur fixés avant un filtre `amount > 0`, `rank: item.rank`), `ExpensePieCard.svelte` (clé de l'`each` des séparateurs par
l'indice), `homepage-charts.test.ts` (deux tests), `CHANGELOG.md` (`### Modifié`), le registre `e34-choix-autonomes-e3410.md`
(annotation de C-e3410-4, C-e3410-14 à -16) et la fiche (annotation de l'AC-3, entrée « revue de code P3 » du Change Log).

Rapports P3 : `/home/gcorbaz/devel/kesh-gate-logs/e3410-review-p3-{B,E,A}.md`. Journaux : `rouges.log` (section
« Remédiation P3 »), `mut/MV15-parts-non-positives.log`, `mut/MV16-rang-apres-filtre.log`, gates `*-p3.log` dans
`/home/gcorbaz/devel/kesh-gate-logs/e3410/`.

## Axes, à braquer sur ce diff seulement

1. **Le filtre `amount > 0`** : les rangs et les couleurs des parts restantes égalent-ils ceux du tableau (`ExpensePieCard.svelte`,
   `postStyle(i)`, `i + 1`) ? `postOf(slice.accountId)`, `data-rank`, la clé `(slice.rank)` de l'`each` des parts restent-ils
   justes et uniques ? Que se passe-t-il si toutes les parts sont filtrées alors que `hasPosts` est vrai (anneau vide, séparateurs,
   centre) ? Le calcul des angles (`real`, `spans`, plus grande part, dernière part fermée sur le tour) reste-t-il juste quand une
   part du milieu est retirée ?
2. **`innerR: 57.6`** : tout ce qui lisait `96 * 0.6` (tests de géométrie, `fullRing`, `centerFontSize` via `HOLE`, `clickRing` de
   l'E2E qui recopie `96 * 0.8`) reste-t-il juste ?
3. **La clé par l'indice** : un séparateur peut-il être réutilisé à tort quand le nombre de parts change (Svelte réutilise le nœud
   d'indice égal ; les attributs suivent-ils) ?
4. **Les documents** : l'annotation de l'AC-3 et celle de C-e3410-4 disent-elles ce que fait le code ? Le CHANGELOG est-il
   cohérent avec la rubrique choisie ? L'entrée « revue de code P3 » du Change Log affirme-t-elle seulement ce qui a tourné
   (gates, mutations, dispense de l'E2E et sa raison) ?
5. **Les tests neufs** : prouvent-ils ce que leur titre dit, et les mutations MV15 et MV16 sont-elles tuées pour la bonne raison ?

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

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e3410-review-p4-ciblee.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`C4-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
