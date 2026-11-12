# Revue de code P1 — 34-10 (#970), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **34-10** de Kesh (issue #970 : les
graphiques de l'accueil en couleurs unies, l'anneau des postes de charges avec le total des dépenses au centre). Contexte
frais. Travaille en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même (géométrie fausse,
  arc mal orienté, texte qui déborde, montant mal comparé, test qui passe à vide, rendu qui contredit le `<desc>`).
- **E — Edge Case Hunter** : chaque branche et chaque frontière — une part, deux, cinq plus « Autres » ; part minuscule,
  99,9 %, 100 %, extrémités confondues ; base nulle ; `totalExpenses` négatif, nul, égal à la base en valeur mais non en
  chaîne (`"30"` / `"30.00"`), différent ; sans poste et totaux différents ; avec ou sans compte au solde créditeur ;
  montant de quatre décimales ; libellé du centre dans les quatre locales ; exercice long (barres) ; clic dans le trou, sur
  le texte du centre, sur le liseré.
- **A — Acceptance Auditor** : la fiche contre le code, AC-1 à AC-9 un par un, mutation par mutation (M1 à M9, journaux cités
  ci-dessous), manuel `.tex`, CHANGELOG, quatre catalogues, registre.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e3410 log --oneline cdf13639..HEAD
git -C /home/gcorbaz/devel/kesh-e3410 diff cdf13639 HEAD -- frontend crates docs CHANGELOG.md
```

Lis le diff **aplati** (un seul diff de `cdf13639` à `HEAD`). Fichiers principaux :
`frontend/src/lib/features/homepage/homepage-charts.ts` (palette, `PIE_GEOMETRY`, `fullRing`, `centerFontSize`,
`differentAmounts`, `pieSlices`), `ExpensePieCard.svelte`, `MonthlyBarsCard.svelte`, `ChartPatterns.svelte` (supprimé),
`frontend/src/app.css` (jetons `--color-chart-*`), `homepage-charts.test.ts`, `HomepageCharts.test.ts`,
`frontend/tests/e2e/homepage-charts.spec.ts` (`clickRing`, tests 3 et 6b), les quatre `crates/kesh-i18n/locales/*/messages.ftl`
(clés `homepage-charts-pie-*`), `frontend/src/lib/shared/i18n-keys.test.ts` et `i18n-libelle-en-dur.test.ts` (compteurs),
`docs/manual/fr/user-manual.tex` (§ *Revenus, dépenses et principaux postes*), `CHANGELOG.md` (section 0.17.6).

Fiche : `/home/gcorbaz/devel/kesh-e3410/_bmad-output/implementation-artifacts/34-10-graphiques-couleurs-unies.md` (lentille A ;
B la lit **après** son examen à l'aveugle, pour classer). Registres : `e34-choix-autonomes-e342.md` (C-e342-15 à -25) et
`e34-choix-autonomes-e3410.md` (C-e3410-1 à -8). Journaux : rouges `/home/gcorbaz/devel/kesh-gate-logs/e3410/rouges.log`,
mutations `/home/gcorbaz/devel/kesh-gate-logs/e3410/mut/M*.log`, gates `/home/gcorbaz/devel/kesh-gate-logs/e3410/*.log`,
captures `/home/gcorbaz/devel/kesh-gate-logs/e3410/captures/` (thème clair, avant et après, et deux en simulation de
daltonisme : deutéranopie, protanopie — ouvre-les avec l'outil de lecture d'images).

## Ce qui est décidé, et n'est pas un finding

Les couleurs unies et l'abandon des trames sont une **décision de Guy** (#970), qui défait l'AC14 de la 33-3b. Ne demande ni
le retour des motifs, ni des étiquettes de valeur sur les parts (C-e332-17, inchangé), ni une teinte qui sépare rouge et vert
en vision deutéranope : la fiche dit que la paire reste confondue par la teinte et ce qui la sépare (C-e342-18). Le PDF du
manuel se régénère **à l'intégration** (C-e342-20) : son absence de mise à jour n'est pas un finding ; contrôle le `.tex`.

## Axes obligatoires

1. **Géométrie de l'anneau** : `pieSlices` trace-t-il un secteur d'anneau juste (arc extérieur horaire, arc intérieur à
   rebours, drapeaux `large-arc` cohérents sur les deux arcs) pour toute part, y compris > 180° et minuscule ? La part pleine
   (`circle`, `fill="none"`, trait de l'épaisseur de l'anneau) couvre-t-elle exactement l'anneau ? Le centre (`<g
   data-testid="homepage-chart-pie-total">`) n'est-il ni un `path` ni dans une marque ?
2. **Le centre et la base** : le centre dit-il `totalExpenses` (jamais `expensePostsBase`) ? La phrase
   `homepage-charts-pie-base-note` et le `homepage-charts-pie-desc-base` sont-ils joints exactement quand il y a des postes
   **et** que les deux montants diffèrent **en valeur** ? `centerFontSize` tient-elle sa promesse (décroissante, bornée) et le
   plus long montant plausible tient-il dans le trou ?
3. **Jetons et couleurs** : chaque constante égale-t-elle son jeton d'`app.css`, en hexadécimal ? Reste-t-il une couleur
   écrite ailleurs qui devrait suivre (légende, pastilles, `<title>`) ? Aucune trace des motifs (`git grep -n
   'ChartPatterns\|patternId\|chartFill\|<pattern' -- frontend/src ':!*.test.ts'`) ?
4. **i18n** : quatre locales, mêmes variables, replis égaux au catalogue fr ; plus aucun `pour un total de { $base }`
   (`insgesamt`, `totalling`, `per un totale di`) ; l'ordre des parts écrit au `<desc>` pluriel ; traductions fidèles.
5. **Le manuel** — `docs/manual/fr/user-manual.tex` : contrôle-le **contre le code** (couleurs, ordre des parts, total du centre
   égal à celui des barres, base des parts, « tramées » absent). Le PDF n'est pas régénéré ici (C-e342-20) : dis-le, ne le
   compte pas.
6. **E2E** : `clickRing` vise-t-il bien l'anneau (80 % du rayon, `viewBox` 200, rayon 96) aux deux tests réécrits ? Le
   comptage de six `path` (test 6b) reste-t-il juste ? Lis le journal E2E et la liste nominative des échecs.
7. **Tests et mutations** : chaque mutation est-elle tuée par le test que la fiche nomme, pour la bonne raison (lis les
   `M*.log`) ? Un test passe-t-il à vide (assertion de montage) ? Les compteurs des deux gardes i18n sont-ils recomptés
   depuis la source (`git show cdf13639:<f> | /usr/bin/grep -o 'i18nMsg(' | wc -l` contre l'arbre) ?
8. **Captures** : montrent-elles ce que le Dev Agent Record en dit (barres vertes et rouges, anneau, total au centre, liseré,
   tableau) ?

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C /home/gcorbaz/devel/kesh-e3410 show HEAD:<fichier> | /usr/bin/grep -nF …`),
**dont tu copies la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (montant faux affiché comme total, lien vers un mauvais compte), HIGH (AC faux, test qui ne prouve rien, part
invisible ou mal placée), MEDIUM (incohérence code/fiche/manuel, mutation tuée pour une mauvaise raison, traduction fausse),
LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout`/`switch`,
`git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui écrit ; pas de
`claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier temporaire**.
**Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e3410-review-p1-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B1-1`, `E1-1`, `A1-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
