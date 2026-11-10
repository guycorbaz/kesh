# Story 34-10 : graphiques de l'accueil en couleurs unies, anneau avec le total des dépenses au centre (#970)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** ; frontend seul. Registre :
`e34-choix-autonomes-e342.md`, C-e342-15 à C-e342-18, C-e342-20. Commits en `refs #970` ; à la PR : `closes #970`. Défait l'AC14 de la
Story 33-3b et son choix C-e332-17 (« jamais la couleur seule » par les trames) sur décision de Guy.*

## Story

En tant qu'utilisateur qui ouvre Kesh, je veux des graphiques en couleurs unies — recettes en vert, dépenses en rouge, et un
anneau des postes de charges qui porte le total des dépenses en son centre —, parce que les hachures sont pénibles à lire.

## Décision de Guy (2026-11-10, #970 et fichier d'epic)

Les hachures sont retirées. Barres en couleurs unies : **recettes en vert, dépenses en rouge**. Le camembert devient un
**anneau**, avec **le total des dépenses au centre** ; parts en couleurs unies : **rouge, vert, bleu**, puis deux ou trois
autres si nécessaire (cinq postes plus « Autres »). L'issue demande en outre, « sans revenir sur le choix » : de garder ce
que la couleur ne porte pas (position fixe des barres, légende, valeurs au survol et au focus, étiquettes ou légende de
chaque part, contraste contre le fond de la carte), et des teintes **dans les jetons du thème** (`--color-…`), non en dur.

## Ce que le code dit (prémisses établies)

- **Les trames** : `frontend/src/lib/features/homepage/ChartPatterns.svelte` (40 lignes, `<pattern>` dans un `<defs>`),
  utilisé par `MonthlyBarsCard.svelte:7`, `:98` et `ExpensePieCard.svelte:8`, `:96` ; le style d'une série est
  `{ color, pattern }` (`homepage-charts.ts:14-19`) ; `chartFill` (`:358-361`) et `patternId` (`:353-356`) n'existent que
  pour elles. Sites qui les lisent (`git grep -l`) : ces deux cartes, `homepage-charts.ts`, `homepage-charts.test.ts`, et
  `ChartPatterns.svelte`.
- **Les couleurs sont écrites en dur** dans `homepage-charts.ts:22-41` : revenus `#1e40af` (= `--color-primary`, un test le
  tient égal, `homepage-charts.test.ts:346-349`), dépenses `#c2410c` tramées, palette des postes bleu, orange, sarcelle,
  fuchsia, vert, « Autres » `#64748b` (= `--color-text-muted`).
- **Les jetons existants** (`frontend/src/app.css:12-35`) : `--color-success: #16a34a`, `--color-destructive: #b91c1c`,
  `--color-error: #dc2626`, `--color-primary: #1e40af`, `--color-warning: #d97706`, `--color-text-muted: #64748b`.
- **Ce qui protège déjà la lecture sans la couleur, et reste** :
  - **position** : la barre des revenus est à gauche de celle des dépenses dans chaque mois (`monthlyBars`, `homepage-charts.ts:168-176`) ;
  - **légende** des barres (`MonthlyBarsCard.svelte:63-71`) ; **tableau des postes** à côté de l'anneau, rang, compte, montant,
    part (`ExpensePieCard.svelte:128-162`), qui porte aussi les seuls liens au clavier (C-e332-16) ;
  - **valeurs au survol** : `<title>` sur chaque mois et chaque part (`MonthlyBarsCard.svelte:112-118`, `ExpensePieCard.svelte:104-118`) ;
    au focus, le tableau (le SVG n'a aucun arrêt de tabulation, AC13 de la 33-3b) ;
  - **description** `<desc>` de l'image (`ExpensePieCard.svelte:70-94`).
- **Les montants** : `totalExpenses` (total des dépenses de l'exercice, celui du compte de résultat et de la somme des barres)
  et `expensePostsBase` (somme des soldes positifs, base des parts) sont deux champs distincts
  (`homepage-summary.api.ts:42-50` ; `crates/kesh-report/src/homepage_summary.rs:115-124`) ; ils diffèrent quand un compte de
  charges est au solde créditeur, cas que la carte annonce déjà par une note (`ExpensePieCard.svelte:170-178`).
- **La géométrie** : `pieSlices` (`homepage-charts.ts:316-351`) rend des secteurs pleins (`M centre L … A … Z`) et un cercle
  pour une part qui fait le tour (`full`, `:288-293`) ; ses tests (`homepage-charts.test.ts:207-292`) figent cette forme ;
  `HomepageCharts.test.ts:133` cherche un `circle` dans la part pleine.
- **Contraste mesuré** (luminance relative WCAG, calculée à la rédaction, à refaire par le test) : sur blanc, `#16a34a` 3,30:1,
  `#b91c1c` 6,47:1, `#1e40af` 8,72:1, `#7c3aed` 5,70:1, `#0891b2` 3,68:1, `#64748b` 4,76:1 — tous ≥ 3:1. Entre eux,
  `#16a34a`/`#b91c1c` 1,96:1 : une **différence de clarté** nette, qui les sépare encore en vision deutéranope ou protanope,
  où leur teinte se confond. Simulation (matrices de Machado, sévérité 1) : la paire rouge/vert reste la plus proche de la
  palette proposée ; ambre ou orange en quatrième place s'en rapprochaient davantage (écartés).

## Acceptance Criteria

1. **AC-1 — jetons** : `app.css` porte les jetons des graphiques, et chaque couleur de `homepage-charts.ts` égale son jeton :
   `--color-chart-revenue: #16a34a` (valeur de `--color-success`), `--color-chart-expense: #b91c1c` (valeur de
   `--color-destructive`), et `--color-chart-1` à `--color-chart-5` pour les postes, dans l'ordre rouge `#b91c1c`, vert
   `#16a34a`, bleu `#1e40af`, violet `#7c3aed`, cyan `#0891b2` ; « Autres » reste `--color-text-muted` (C-e342-15).
   **Forme tranchée** *(F10-2)* : les jetons s'écrivent **en hexadécimal**, non en alias `var(--color-success)` ; le module
   TypeScript garde des **constantes hexadécimales** (les tests en ont besoin : `contrastRatio` ne lit que `#rrggbb`, et jsdom ne
   charge pas `app.css`) ; un test les tient égales, nom par nom (`toContain('--color-chart-1: #b91c1c;')`, patron de
   `homepage-charts.test.ts:346`), et vérifie aussi `--color-chart-revenue` = `--color-success`, `--color-chart-expense` =
   `--color-destructive`. Les SVG emploient la constante (valeur en clair dans `fill`). La fiche ne promet donc plus « aucune
   couleur en dur » : elle promet **aucun motif**, et des couleurs **égales à leurs jetons**.
2. **AC-2 — barres** : revenus pleins en vert, dépenses pleines en rouge ; aucune trame ; revenus à gauche, dépenses à droite
   (inchangé). Légende aux mêmes couleurs.
3. **AC-3 — anneau** : les parts sont des **secteurs d'anneau** (rayon intérieur ≈ 60 % du rayon extérieur), séparés par un
   liseré blanc (`stroke` de la couleur de fond de la carte, 1 à 2 unités) qui marque chaque limite quelle que soit la teinte
   (C-e342-16) ; les cas limites de `pieSlices` (part minuscule, 99,9 %, 100 %, « Autres ») gardent leurs tests, réécrits sur
   la forme d'anneau. **Forme de la part pleine, tranchée** *(F10-6)* : un `circle` au rayon médian, sans remplissage, au
   trait de l'épaisseur de l'anneau (`fill="none"`, `stroke-width = r_ext − r_int`) ; une part ordinaire est un `path` (arc
   extérieur, segment, arc intérieur inverse). Le trou et le texte du centre ne sont **ni** des `path` ni dans un groupe
   `homepage-chart-post-mark` (le comptage de `homepage-charts.spec.ts:207`, six `path`, reste juste) ;
   `HomepageCharts.test.ts:133` (`circle` dans la part pleine) reste juste.
4. **AC-4 — le centre** : au centre, sur deux lignes, « Total des dépenses » et le **total des dépenses de l'exercice**
   (`totalExpenses`), au format suisse (`chartAmount`), en `--color-text` ; il tient dans le trou pour le plus long montant
   plausible (`999'999'999.99`, test de rendu : la largeur du texte mesurée ou bornée par la taille de police calculée sur sa
   longueur). Quand `totalExpenses` diffère de `expensePostsBase`, la note existante des comptes au solde créditeur reste, et
   dit en plus que les parts portent sur `{ $base }` (C-e342-17). Le `<desc>` dit le total. **Total négatif ou nul** *(F10-3 ;
   cas monté par `HomepageCharts.test.ts:113`, `totalExpenses: '-50'` pour `expensePostsBase: '100'`)* : le centre l'affiche tel
   quel, signe compris (`chartAmount`, « -50.00 »), puisque c'est le total du compte de résultat ; la note des parts dit alors
   sur quelle base elles portent. Test de ce cas.
5. **AC-5 — couleurs unies au camembert** : rang 1 rouge, 2 vert, 3 bleu, 4 violet, 5 cyan, « Autres » gris ; la pastille de
   chaque ligne du tableau porte la couleur de sa part ; le **rang** reste écrit dans le tableau (il relie la part à son
   compte sans la couleur).
6. **AC-6 — ce que la couleur ne porte pas** (#970, « conséquence à traiter ») : position fixe des barres ; légende ; `<title>`
   au survol de chaque mois et de chaque part ; clavier par les tableaux, sans arrêt de tabulation dans les SVG (AC13 de la
   33-3b, inchangé) ; clic sur une part nommée vers son grand livre conservé (`data-href`, `followMark`) ; **l'ordre des
   parts** — à partir de midi, dans le sens horaire, rang 1 d'abord, « Autres » en dernier —, écrit dans le `<desc>` et au manuel
   *(F10-5)* : c'est le seul repère non coloré de l'anneau, des parts voisines ne se distinguant guère que par le liseré
   (vert/cyan 1,12:1, bleu/rouge 1,35:1, bleu/violet 1,53:1 entre elles). La fiche **dit** que
   la paire rouge/vert reste confondue **par la teinte** pour un daltonien, et ce qui la sépare : clarté (1,96:1), position,
   légende, tableau (C-e342-18).
7. **AC-7 — tests de palette** (remplacent ceux de l'AC14, `homepage-charts.test.ts:344-375`) : contraste ≥ 3:1 sur blanc
   de chaque couleur (`contrastRatio`, `tests/e2e/helpers/contrast.ts`) ; revenus ≠ dépenses et contraste entre eux ≥ 1,5:1 ;
   six couleurs distinctes à l'anneau ; aucune mention de motif dans le module (`ChartPattern`, `chartFill`, `patternId`
   retirés). `ChartPatterns.svelte` supprimé, et `git grep -n 'ChartPatterns\|patternId\|chartFill\|<pattern' frontend/src`
   ne rend rien.
8. **AC-8 — rendu** : `HomepageCharts.test.ts` : aucune balise `<pattern>` ; chaque barre `fill` = la constante (égale au
   jeton, AC-1) ; l'anneau rend son total au centre (`data-testid="homepage-chart-pie-total"`) ; part pleine en `circle`.
   **E2E `homepage-charts.spec.ts` : deux clics à réécrire** *(F10-1)* — le test 3 (`:105-113`) clique « à neuf heures, à
   mi-rayon » (`x = 0,25 × largeur`), qui tombe dans le trou d'un anneau à 60 % : cliquer à **≈ 80 % du rayon** (dans
   l'anneau), et réécrire le message `:108` (« un arc, non un disque ») et le commentaire `:110-111` ; `:215`
   (`marks.first().click()`) clique le centre de la boîte de la première part, qui peut tomber dans le trou ou sur le texte du
   centre : un point de l'anneau, calculé comme au test 3. `:207` (six `path`) reste juste (AC-3). Les autres parcours
   (clavier, axe-core sans violation, AC15 de la 33-3b) inchangés et verts.
9. **AC-9 — manuel et captures** : manuel utilisateur, § des graphiques : `docs/manual/fr/user-manual.tex:263` (« les revenus
   (pleins) et les dépenses (tramées) ») → revenus en vert, dépenses en rouge ; `:288-296` (*Les postes de charges* : « la
   somme des parts peut dépasser le total des dépenses des barres ») → l'anneau, son total au centre, **égal à celui des
   barres**, la base des parts qui peut en différer, l'ordre des parts *(F10-4)*. Le `.tex` change dans la story ; le PDF de
   `user-manual`, touché par quatre stories de l'epic, se **régénère une fois à l'intégration** et s'y contrôle aplati
   (« tramées » absent ; un PDF ne se fusionne pas). Captures des deux graphiques (thème clair, base jetable peuplée : cinq
   postes et « Autres », un mois négatif), dans `kesh-gate-logs/e342/captures/`, avant et après.

## Tâches

- [ ] **T0 — étape 0** : relire les prémisses ; recalculer les contrastes de l'AC-1 par le test avant de les écrire.
- [ ] **T1 — rouge** : tests AC-1, AC-3 (géométrie d'anneau), AC-4, AC-7, AC-8 ; rouges journalisés
  (`kesh-gate-logs/e342/rouges.log`).
- [ ] **T2** : jetons dans `app.css` ; `homepage-charts.ts` (styles réduits à une couleur, géométrie d'anneau, sans motif) ;
  `MonthlyBarsCard.svelte`, `ExpensePieCard.svelte` ; suppression de `ChartPatterns.svelte` (par `git rm`, dans le commit).
- [ ] **T1-bis** : `homepage-charts.spec.ts`, les deux clics de l'AC-8 (`:105-113`, `:215`).
- [ ] **T3** : i18n (« Total des dépenses », note des parts, `<desc>` avec l'ordre ; quatre locales, replis égaux) ; manuel
  (`.tex` ; PDF à l'intégration) ; CHANGELOG *Changé*
  (`refs #970`).
- [ ] **T4** : captures ; contrôle visuel en simulation de daltonisme (DevTools de Chromium, *Rendering → Emulate vision
  deficiencies*, deutéranopie et protanopie), une capture chacune, et ce qu'on y lit dit au Change Log.

### Mutations nommées

- **M1** : dépenses en `--color-success` (mêmes couleurs que les revenus) → rouge AC-7 (revenus ≠ dépenses).
- **M2** : une constante de l'anneau différente de son jeton → rouge AC-1.
- **M2-bis** : jeton écrit en alias (`var(--color-success)`) → rouge AC-1 (égalité nom par nom).
- **M3** : rayon intérieur nul (retour au camembert) → rouge AC-3.
- **M4** : centre sur `expensePostsBase` → rouge AC-4 (cas d'un compte au solde créditeur, et cas `totalExpenses: '-50'`).
- **M5** : `#facc15` (jaune, 1,5:1 sur blanc) en cinquième couleur → rouge AC-7.

## Gates

Frontend seul : `npm run check`, `lint-i18n-ownership`, Vitest, `npm run build`, puis E2E complet au dernier commit de code.
La garde `jetons-non-definis.test.ts` ne voit que des classes ; les jetons neufs ne sont lus par aucune classe, et le test
AC-1 les tient égaux aux constantes.

## Ce qui ne se traite pas

- Les étiquettes de valeur **sur** les parts : écartées par C-e332-17 (illisibles sur une petite part), le tableau les porte ;
  inchangé.
- Aucune issue à ouvrir à la rédaction.

## Recoupements

Aucun fichier de code commun avec les autres stories de l'Epic 34. `user-manual.tex` est partagé avec 34-3, 34-4 et 34-9
(sections distinctes) ; son PDF se régénère à l'intégration (AC-9).

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).
- 2026-11-10 : Validation P1 (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p1-e342.md`, pour cette
  fiche 0 C, 0 H, 2 M, 4 LOW, tous retenus après lecture : **F10-1** (lus : `homepage-charts.spec.ts:105-113`, `:207`, `:215`)
  → deux clics réécrits à ≈ 80 % du rayon, T1-bis ; **F10-2** (lus : `homepage-charts.test.ts:346`, `contrast.ts` ne lit que
  `#rrggbb`) → jetons en hexadécimal, constantes TypeScript égales, M2-bis ; **F10-3** (lu : `HomepageCharts.test.ts:113`) →
  total négatif affiché tel quel, testé ; **F10-4** (lu : `user-manual.tex:288-296`) → AC-9 ; **F10-5** (contrastes entre parts
  recalculés : 1,12, 1,35, 1,53) → ordre des parts à l'AC-6 et au `<desc>` ; **F10-6** → part pleine en `circle` au trait épais.
  Transversal **X-1** : PDF à l'intégration (AC-9).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
