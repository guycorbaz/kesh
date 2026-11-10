# Revue de code P2 — 34-10 (#970), trois lentilles B, E, A (passe complète, Opus, lecture seule)

Tu es **une** des trois lentilles de la passe 2 de `bmad-code-review` de la story **34-10** de Kesh (issue #970 : les
graphiques de l'accueil en couleurs unies, l'anneau des postes de charges avec le total des dépenses au centre). Modèle :
**Opus**. Contexte frais. Travaille en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même (géométrie fausse,
  angles dessinés qui ne referment pas le tour, ordre de peinture qui casse un clic ou un lien, texte qui déborde, test qui
  passe à vide).
- **E — Edge Case Hunter** : chaque branche et chaque frontière — une part, deux, cinq plus « Autres » ; parts minuscules
  multiples (plusieurs sous `MIN_SLICE_ANGLE`), une part juste sous et juste au-dessus de `SEPARATED_MIN_ANGLE`, plus grande
  part qui cède plus qu'elle n'a de marge, égalité de deux plus grandes parts, part minuscule en premier, au milieu, en
  dernier ; 99,9 %, 100 % ; base nulle ; `totalExpenses` négatif, nul, égal en valeur et non en chaîne, différent ; sans poste ;
  libellé du centre dans les quatre locales (taille et écart vertical) ; clic dans le trou, sur le texte, sur le liseré, sur une
  part étroite.
- **A — Acceptance Auditor** : la fiche contre le code, AC-1 à AC-9, mutation par mutation (journaux cités ci-dessous, dont
  ceux de la remédiation P1), manuel `.tex`, CHANGELOG, quatre catalogues, registre ; et l'entrée « revue de code P1 » du Change
  Log contre ce qui a réellement été fait.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e3410 log --oneline cdf13639..HEAD
git -C /home/gcorbaz/devel/kesh-e3410 diff cdf13639 HEAD -- frontend crates docs CHANGELOG.md
git -C /home/gcorbaz/devel/kesh-e3410 diff b2eb42ec HEAD -- frontend docs CHANGELOG.md   # la remédiation P1 seule
```

Lis le diff **aplati** (un seul diff de `cdf13639` à `HEAD`), puis la remédiation P1 à part. Fichiers principaux :
`frontend/src/lib/features/homepage/homepage-charts.ts` (palette, `PIE_GEOMETRY`, `fullRing`, `centerFontSize`, `centerLines`,
`differentAmounts`, `MIN_SLICE_ANGLE`, `SEPARATED_MIN_ANGLE`, `SEPARATOR_WIDTH`, `drawOrder`, `pieSlices`), `ExpensePieCard.svelte`,
`MonthlyBarsCard.svelte`, `frontend/src/app.css`, `homepage-charts.test.ts`, `HomepageCharts.test.ts`,
`frontend/tests/e2e/homepage-charts.spec.ts`, les quatre `crates/kesh-i18n/locales/*/messages.ftl`, `docs/manual/fr/user-manual.tex`,
`CHANGELOG.md`.

Fiche : `/home/gcorbaz/devel/kesh-e3410/_bmad-output/implementation-artifacts/34-10-graphiques-couleurs-unies.md` (lentille A ;
B la lit **après** son examen à l'aveugle, pour classer). Registres : `e34-choix-autonomes-e342.md` (C-e342-15 à -25) et
`e34-choix-autonomes-e3410.md` (C-e3410-1 à -11). Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/e3410-review-p1-{B,E,A}.md`.
Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e3410/rouges.log`, `mut/*.log` (dont `M6-bis`, `MV1` à `MV7`), gates `*-p1.log`,
E2E `e2e-r3.log` ; captures `/home/gcorbaz/devel/kesh-gate-logs/e3410/captures/`, dont `anneau-avant-p1-minuscule.png` et
`anneau-apres-p1-minuscule.png` (ouvre-les avec l'outil de lecture d'images).

## Ce qui est décidé, et n'est pas un finding

Les couleurs unies et l'abandon des trames sont une **décision de Guy** (#970). Ne demande ni le retour des motifs, ni des
étiquettes de valeur sur les parts, ni une teinte qui sépare rouge et vert en vision deutéranope (C-e342-18). Le PDF du manuel se
régénère à l'intégration (C-e342-20). L'élargissement au dessin d'une part minuscule (C-e3410-9) est un choix assumé : le
discuter est permis s'il ment (tableau, `<title>`, `<desc>`, manuel), non pour lui préférer une autre option à l'égal.

## Axes obligatoires

1. **La séparation des parts (axe dédié, remédiation P1)** : toute part non nulle est-elle visible au rendu ? Les angles
   dessinés (`displayStart`, `displayEnd`) referment-ils exactement le tour, restent-ils croissants, et la plus grande part
   peut-elle céder ce qu'on lui prend dans tous les cas (six parts au plus) ? Une part sans liseré peinte en dernier
   masque-t-elle à tort une voisine, un clic ou un `<title>` ? Le liseré marque-t-il encore chaque limite (demi-liseré resté
   chez la voisine, deux parts étroites côte à côte) ? L'ordre de peinture change l'ordre DOM des marques : un test, l'E2E
   (`.first()`, `clickRing`) ou un lecteur d'écran en dépend-il ? Les angles vrais restent-ils ceux de `amount / base` ?
2. **Géométrie de l'anneau** : secteurs (drapeaux `large-arc` sur les deux arcs), part pleine (`circle`), `full` sur l'arc
   extérieur seul — l'argument qui rend la garde intérieure inutile tient-il ?
3. **Le centre** : `totalExpenses` jamais `expensePostsBase` ; `centerFontSize` ; `centerLines` (écart, centrage) ; condition de
   la phrase de la base (`hasPosts` et différence en valeur).
4. **Jetons et couleurs** ; aucune trace des motifs.
5. **i18n** : quatre locales, mêmes variables, replis égaux au fr, anciennes formes absentes.
6. **Le manuel** — `docs/manual/fr/user-manual.tex`, contre le code (couleurs, ordre, total, base, part élargie). PDF non
   régénéré : dis-le, ne le compte pas.
7. **E2E** : journal `e2e-r3.log`, échecs nommés un par un contre `docs/testing.md` § « Les échecs attendus ».
8. **Tests et mutations** : chaque mutation tuée pour la bonne raison ; `MV4` (survivante, déclarée équivalente) l'est-elle
   vraiment ? Un test passe-t-il à vide ? Un test recopie-t-il la formule qu'il contrôle ?
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

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e3410-review-p2-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B2-1`, `E2-1`, `A2-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
