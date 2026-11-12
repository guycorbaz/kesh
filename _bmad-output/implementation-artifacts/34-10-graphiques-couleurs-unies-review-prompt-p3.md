# Revue de code P3 — 34-10 (#970), trois lentilles B, E, A (passe complète, Sonnet, lecture seule)

Tu es **une** des trois lentilles de la passe 3 de `bmad-code-review` de la story **34-10** de Kesh (issue #970 : les
graphiques de l'accueil en couleurs unies, l'anneau des postes de charges avec le total des dépenses au centre). Modèle :
**Sonnet**. Contexte frais. Travaille en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même (géométrie fausse,
  angles dessinés qui ne referment pas le tour, séparateur mal placé, absent ou qui capte un clic, texte qui déborde, test qui
  passe à vide).
- **E — Edge Case Hunter** : chaque branche et chaque frontière — une part, deux, cinq plus « Autres » ; parts minuscules
  multiples (plusieurs sous `MIN_SLICE_ANGLE`, voisines), une part juste sous et juste au-dessus de `MIN_SLICE_ANGLE`, plus
  grande part qui cède plus qu'elle n'a de marge, égalité de deux plus grandes parts, part minuscule en premier, au milieu, en
  dernier ; 99,9 %, 100 % ; base nulle ; `totalExpenses` négatif, nul, égal en valeur et non en chaîne, différent ; sans poste ;
  libellé du centre dans les quatre locales (taille et écart vertical) ; clic dans le trou, sur le texte, sur un séparateur, sur une
  part étroite.
- **A — Acceptance Auditor** : la fiche contre le code, AC-1 à AC-9, mutation par mutation (journaux cités ci-dessous, dont
  ceux des remédiations P1 et P2), manuel `.tex`, CHANGELOG, quatre catalogues, registre ; et les entrées « revue de code P1 » et
  « revue de code P2 » du Change Log contre ce qui a réellement été fait.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e3410 log --oneline cdf13639..HEAD
git -C /home/gcorbaz/devel/kesh-e3410 diff cdf13639 HEAD -- frontend crates docs CHANGELOG.md
git -C /home/gcorbaz/devel/kesh-e3410 diff 111c808b HEAD -- frontend docs CHANGELOG.md   # la remédiation P2 seule
```

Lis le diff **aplati** (un seul diff de `cdf13639` à `HEAD`), puis la remédiation P2 à part. Fichiers principaux :
`frontend/src/lib/features/homepage/homepage-charts.ts` (palette, `PIE_GEOMETRY`, `fullRing`, `centerFontSize`, `centerLines`,
`centerSizes`, `differentAmounts`, `MIN_SLICE_ANGLE`, `SEPARATOR_WIDTH`, `pieSeparators`, `pieSlices`), `ExpensePieCard.svelte`,
`MonthlyBarsCard.svelte`, `frontend/src/app.css`, `homepage-charts.test.ts`, `HomepageCharts.test.ts`,
`frontend/tests/e2e/homepage-charts.spec.ts`, les quatre `crates/kesh-i18n/locales/*/messages.ftl`, `docs/manual/fr/user-manual.tex`,
`CHANGELOG.md`.

Fiche : `/home/gcorbaz/devel/kesh-e3410/_bmad-output/implementation-artifacts/34-10-graphiques-couleurs-unies.md` (lentille A ;
B la lit **après** son examen à l'aveugle, pour classer). Registres : `e34-choix-autonomes-e342.md` (C-e342-15 à -25) et
`e34-choix-autonomes-e3410.md` (C-e3410-1 à -13). Rapports P1 et P2 : `/home/gcorbaz/devel/kesh-gate-logs/e3410-review-p{1,2}-{B,E,A}.md`.
Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e3410/rouges.log`, `mut/*.log` (dont `MV1` et `MV8` à `MV14`, jouées à la levée de la pause
des commandes lourdes), gates `*-p2.log`, E2E `e2e-r4.log` ; captures `/home/gcorbaz/devel/kesh-gate-logs/e3410/captures/`, dont `anneau-avant-p2-minuscule-agrandi.png` et
`anneau-apres-p2-minuscule-agrandi.png` (ouvre-les avec l'outil de lecture d'images).

## Ce qui est décidé, et n'est pas un finding

Les couleurs unies et l'abandon des trames sont une **décision de Guy** (#970). Ne demande ni le retour des motifs, ni des
étiquettes de valeur sur les parts, ni une teinte qui sépare rouge et vert en vision deutéranope (C-e342-18). Le PDF du manuel se
régénère à l'intégration (C-e342-20). L'élargissement au dessin d'une part minuscule (C-e3410-9, angle porté à 0,045 rad par C-e3410-12) est un choix assumé : le
discuter est permis s'il ment (tableau, `<title>`, `<desc>`, manuel), non pour lui préférer une autre option à l'égal.

## Axes obligatoires

1. **Les séparateurs (axe dédié, remédiation P2)** : `pieSeparators` trace-t-il un trait par limite, au bon angle (le début
   dessiné de chaque part, celui de midi fermant la dernière), du trou au bord extérieur ? Deux parts étroites voisines sont-elles
   séparées ? Les traits sont-ils peints après toutes les parts, sans capter le clic (`pointer-events="none"`), sans masquer un
   `<title>` ni un lien ? Une part reste-t-elle visible entre ses deux traits pour tout angle dessiné ≥ `MIN_SLICE_ANGLE`
   (calcul et test) ? Les angles dessinés referment-ils exactement le tour, croissants, et la plus grande part peut-elle céder la
   somme des écarts dans tous les cas (six parts au plus) ? Les angles vrais restent-ils ceux de `amount / base` ? Une part seule
   n'a-t-elle aucun séparateur ? L'ordre DOM des marques est revenu à l'ordre des rangs : un test ou l'E2E en dépend-il encore à tort ?
2. **Géométrie de l'anneau** : secteurs (drapeaux `large-arc` sur les deux arcs), part pleine (`circle`), `full` sur l'arc
   extérieur seul — l'argument qui rend la garde intérieure inutile tient-il ?
3. **Le centre** : `totalExpenses` jamais `expensePostsBase` ; `centerFontSize`, `centerSizes` (libellé ≤ montant) ; `centerLines` ; condition de
   la phrase de la base (`hasPosts` et différence en valeur).
4. **Jetons et couleurs** ; aucune trace des motifs.
5. **i18n** : quatre locales, mêmes variables, replis égaux au fr, anciennes formes absentes.
6. **Le manuel** — `docs/manual/fr/user-manual.tex`, contre le code (couleurs, ordre, total, base, part élargie). PDF non
   régénéré : dis-le, ne le compte pas.
7. **E2E** : journal `e2e-r4.log`, échecs nommés un par un contre `docs/testing.md` § « Les échecs attendus ».
8. **Tests et mutations** : chaque mutation tuée pour la bonne raison ; `MV4` (survivante à la P1, déclarée équivalente) l'est-elle
   vraiment ? `MV8` (`deficit +=` → `=`) et `MV14` (frontière) sont-elles tuées ? Un test passe-t-il à vide ? Un test recopie-t-il la formule qu'il contrôle ?
9. **Captures** : montrent-elles ce que le Dev Agent Record en dit, la part minuscule comprise ?

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C /home/gcorbaz/devel/kesh-e3410 show HEAD:<fichier> | /usr/bin/grep -nF …`),
**dont tu copies la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (montant faux affiché comme total, lien vers un mauvais compte), HIGH (AC faux, test qui ne prouve rien, part
invisible ou mal placée), MEDIUM (incohérence code/fiche/manuel, mutation tuée pour une mauvaise raison, traduction fausse),
LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `node`, `make`, `latexmk`, `docker`, `curl`,
`sqlx`, `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ou de `kesh-gate-logs/` ; pas de
`git checkout`/`switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui
écrit ; pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier
temporaire**. **Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e3410-review-p3-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B3-1`, `E3-1`, `A3-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
