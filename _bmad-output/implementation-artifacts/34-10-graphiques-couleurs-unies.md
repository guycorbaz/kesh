# Story 34-10 : graphiques de l'accueil en couleurs unies, anneau avec le total des dépenses au centre (#970)

Status: review

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** ; frontend seul. Registre :
`e34-choix-autonomes-e342.md`, C-e342-15 à C-e342-18, C-e342-20, C-e342-22, C-e342-24, C-e342-25. Commits en `refs #970` ; à la PR : `closes #970`. Défait l'AC14 de la
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
   plausible (`999'999'999.99`). **Règle tranchée** *(P2-10-3 : jsdom n'a pas de mise en page, ni `getBBox` ni
   `getComputedTextLength`)* : une fonction pure `centerFontSize(label, holeDiameter)` (taille de police décroissante avec la
   longueur du texte, bornée), testée en Vitest pour 4 et 14 caractères ; aucune mesure de rendu exigée. Quand `totalExpenses`
   diffère de `expensePostsBase` **en valeur** (`"30"` et `"30.00"` sont égaux, P7-F2) **et qu'il y a des postes** (P7-F1,
   cf. la condition complète ci-dessous), la note existante des comptes au solde créditeur (`homepage-charts-credit-balance-note-one` /
   `-other`, fr `messages.ftl:3477-3478`, inchangée : elle ne dit que le nombre de comptes écartés) reste, et **une phrase jointe
   après elle**, clé neuve `homepage-charts-pie-base-note` (« Les parts portent sur { $base }. »), dit sur quelle base portent les
   parts (C-e342-17 ; *P4-F5 : la note existante ne le dit pas*). *(P5-5)* Elle est jointe **sous la même condition** que
   `homepage-charts-pie-desc-base` (`totalExpenses ≠ expensePostsBase`), **après** la note dans le DOM si celle-ci est rendue,
   seule sinon ; Vitest : présente, avec 30.00, dans le cas « différents » (fixture ci-dessous), absente dans le cas « égaux ».
   *(Validation P6, F1)* La note des comptes créditeurs est **hors** de `{#if hasPosts}` (`ExpensePieCard.svelte:173-180`) :
   la phrase de la base ne s'y accroche donc pas seule. Sa condition complète est **`hasPosts` et `totalExpenses ≠
   expensePostsBase`** ; **sans poste**, elle est absente (pas d'anneau, pas de parts dont dire la base), même si les totaux
   diffèrent — la carte montre alors « Aucun poste de charges… » et la note seule. Vitest : troisième cas, sans poste et totaux
   différents (fixture `HomepageCharts.test.ts:108-119`, `totalExpenses: '-50'`, base `'0'`) : phrase absente.
   **`<desc>`** *(P2-10-2 : il dit aujourd'hui « pour un total de { $base } »,
   `ExpensePieCard.svelte:74-92`, qui contredirait le centre)*. **Tranché à la validation P3 (P3-10-1)** : le `<desc>` existe
   déjà en **deux clés**, choisies par le nombre de comptes — `homepage-charts-pie-desc-one` (« … un seul compte de charges,
   pour un total de { $base } : { $top }. ») et `homepage-charts-pie-desc` (« … pour un total de { $base }. Le plus gros : … ») ;
   catalogues fr `:3464-3465`, de `:3391-3392`, it `:3392-3393`, en `:3397-3398` ; replis `ExpensePieCard.svelte:77`, `:87`. Les **deux**
   passent de `{ $base }` à « total des dépenses { $total } » (`totalExpenses`) ; une clé neuve `homepage-charts-pie-desc-base`
   (« Parts calculées sur { $base }. »), jointe par le Svelte **seulement** quand `totalExpenses ≠ expensePostsBase`. Quatre
   locales, replis égaux. Vitest des **quatre** cas (un compte / plusieurs × égaux / différents). Après la story,
   `git grep -n -F 'pour un total de { $base }' -- frontend crates` ne rend rien (et ses équivalents de/en/it, `insgesamt { $base }`,
   `totalling { $base }`, `per un totale di { $base }`).
   **Total négatif ou nul** *(F10-3 ; fixture corrigée à la validation P2, P2-10-1 : `HomepageCharts.test.ts:108-119` monte
   `totalExpenses: '-50'` **sans poste**, `expensePostsBase` y vaut `'0'` par défaut, `:36`, et la carte n'y dessine aucun
   anneau)* : fixture **neuve avec postes** — un poste à 30, un compte au solde créditeur à −80 : `totalExpenses: '-50'`,
   `expensePostsBase: '30'`, `creditBalanceExpenseAccounts: 1` ; le centre affiche « -50.00 », signe compris (c'est le total du
   compte de résultat), et la phrase jointe (`homepage-charts-pie-base-note`) dit que les parts portent sur 30.00. **Sans poste** (`expensePosts` vide), pas d'anneau,
   donc pas de total au centre : la carte garde son message `homepage-charts-no-posts` et sa note ; le total reste lisible aux
   barres et au compte de résultat (C-e342-17).
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
   retirés). `ChartPatterns.svelte` supprimé, et
   `git grep -n 'ChartPatterns\|patternId\|chartFill\|<pattern' -- frontend/src ':!*.test.ts'` ne rend rien (tests exclus,
   P2-10-4).
8. **AC-8 — rendu** : `HomepageCharts.test.ts` : aucune balise `<pattern>`, assertée par `document.querySelector('pattern')`
   nul (non par une chaîne `'<pattern'`, P2-10-4) ; chaque barre `fill` = la constante (égale au
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

- [x] **T0 — étape 0** : relire les prémisses ; recalculer les contrastes de l'AC-1 par le test avant de les écrire.
- [x] **T1 — rouge** : tests AC-1, AC-3 (géométrie d'anneau), AC-4, AC-7, AC-8 ; rouges journalisés
  (`kesh-gate-logs/e342/rouges.log`).
- [x] **T2** : jetons dans `app.css` ; `homepage-charts.ts` (styles réduits à une couleur, géométrie d'anneau, sans motif) ;
  `MonthlyBarsCard.svelte`, `ExpensePieCard.svelte` ; suppression de `ChartPatterns.svelte` (par `git rm`, dans le commit).
- [x] **T1-bis** : `homepage-charts.spec.ts`, les deux clics de l'AC-8 (`:105-113`, `:215`).
- [x] **T3** : i18n (« Total des dépenses », phrase `homepage-charts-pie-base-note`, `homepage-charts-pie-desc-base`, `<desc>` avec l'ordre ; quatre locales, replis égaux) ; manuel
  (`.tex` ; PDF à l'intégration) ; CHANGELOG *Changé*
  (`refs #970`).
- [x] **T4** : captures ; contrôle visuel en simulation de daltonisme (DevTools de Chromium, *Rendering → Emulate vision
  deficiencies*, deutéranopie et protanopie), une capture chacune, et ce qu'on y lit dit au Change Log.

### Mutations nommées

- **M1** : dépenses en `--color-success` (mêmes couleurs que les revenus) → rouge AC-7 (revenus ≠ dépenses).
- **M2** : une constante de l'anneau différente de son jeton → rouge AC-1.
- **M2-bis** : jeton écrit en alias (`var(--color-success)`) → rouge AC-1 (égalité nom par nom).
- **M3** : rayon intérieur nul (retour au camembert) → rouge AC-3.
- **M4** : centre sur `expensePostsBase` → rouge AC-4 (fixture neuve avec un poste à 30 et `totalExpenses: '-50'` : le centre
  dirait 30.00).
- **M6** : `{ $base }` encore présent dans l'une des deux clés existantes du `<desc>` (`-one` ou sans suffixe) → rouge AC-4
  (Vitest du `<desc>`, cas « différents » du nombre de comptes concerné).
- **M7** : `centerFontSize` rend une taille constante → rouge AC-4 (la taille pour 14 caractères doit être strictement inférieure
  à celle pour 4, et au moins égale au plancher lisible que la fonction déclare).
- **M8** : `homepage-charts-pie-desc-base` (et la phrase `homepage-charts-pie-base-note`) jointe sans condition, ou sous la
  condition inversée → rouge AC-4 (Vitest, cas « égaux » et cas « différents »).
- **M8-bis** : la phrase `homepage-charts-pie-base-note` conditionnée aux seuls totaux, sans `hasPosts` → rouge AC-4 (Vitest,
  cas sans poste et totaux différents).
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
- 2026-11-10 : Validation P2 (Opus), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p2-e342.md`, pour cette
  fiche 0 C, 0 H, 1 M, 3 LOW, tous retenus après lecture :
  - **P2-10-1** MEDIUM, **né de la remédiation P1** (F10-3 citait une fixture sans poste, et une base de `'100'` qu'elle ne porte
    pas ; lus : `HomepageCharts.test.ts:30-40`, `:108-119`) → fixture neuve avec un poste et un compte créditeur, M4 réécrite ;
    le cas sans poste est dit (pas d'anneau, pas de total) ;
  - LOW : **P2-10-2** (lu : `ExpensePieCard.svelte:74-92`) → `<desc>` à deux clés, M6 ; **P2-10-3** → règle pure de taille de
    police, testée en Vitest ; **P2-10-4** → garde sans les `*.test.ts`, assertion par `querySelector`.
  - CHANGELOG : *Changé* gardé (rendu d'une fonction existante modifié), cohérent avec la 34-3 (C-e342-22).
- 2026-11-10 : Validation P3 (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p3-e342.md`, pour cette
  fiche 0 C, 0 H, 1 M, 1 LOW, tous retenus :
  - **P3-10-1** MEDIUM, **né de la remédiation P2** (P2-10-2 ne disait rien des deux clés existantes du `<desc>`) : lus
    `ExpensePieCard.svelte:74-92` et les catalogues (fr `:3464-3465`, de `:3391-3392`, it `:3392-3393`, en `:3397-3398`) ; les deux
    clés passent au total des dépenses, une clé neuve `homepage-charts-pie-desc-base` est jointe quand la base diffère, Vitest des
    quatre cas, M6 reformulée. Symptôme grepé (`git grep -F 'pour un total de { $base }'`) : 2 lignes de catalogue fr, 2 replis,
    tous nommés à l'AC-4 ;
  - **P3-10-2** LOW → M7 pour `centerFontSize`.
- 2026-11-10 : Validation P4 ciblée (Haiku), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p4-e342.md`, pour
  cette fiche 2 LOW, retenus après lecture : **F5** (lu : fr `messages.ftl:3477-3478`, la note des comptes créditeurs ne dit
  que leur nombre) → la base des parts se dit par une phrase jointe, clé neuve `homepage-charts-pie-base-note`, AC-4 et T3 ;
  C-e342-17 annotée au registre ; **F6** → M8, la condition de jointure.
- 2026-11-10 : Validation P5 ciblée (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p5-e342.md`, pour
  cette fiche 2 LOW, retenus : **P5-4** → la sous-puce CHANGELOG de la P2, déplacée par les insertions P3 et P4, est remise
  sous son entrée, texte inchangé ; **P5-5** → phrase jointe spécifiée à l'AC-4 (place, condition, Vitest présente/absente),
  `homepage-charts-pie-base-note` ajoutée à C-e342-25.
- 2026-11-10 : Validation P6 ciblée (Haiku), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p6-e342.md`, pour
  cette fiche 1 M, retenu, et 1 LOW au registre : **F1** MEDIUM, **né de la remédiation P5** (lu : `ExpensePieCard.svelte:167-180`,
  la note des comptes créditeurs est hors de `{#if hasPosts}`) → condition `hasPosts` et totaux différents, absence sans poste
  dite, troisième cas Vitest, M8-bis ; **F3** LOW → titre de C-e342-25 annoté au registre.

- 2026-11-10 : Développement (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e3410`, base `cdf13639`, commits `ddc1606f`
  (code), `7a2cebcd` (liseré), puis ce compte rendu. Détail au Dev Agent Record.
- 2026-11-11 : **Revue de code P1** (Sonnet, trois lentilles B, E, A ; prompt `34-10-graphiques-couleurs-unies-review-prompt-p1.md`,
  rapports `kesh-gate-logs/e3410-review-p1-{B,E,A}.md`) : **0 C, 0 H, 1 M, 7 L** (B : 1 M, 3 L ; E : 1 M, 2 L ; A : 2 L ; le
  MEDIUM est commun à B et E, B1-1 = E1-1). Remédiation (Opus 5.5), commits `924bc54c`, `11af9624` :
  - **B1-1 = E1-1** MEDIUM, retenu (lu : `ExpensePieCard.svelte:144` à `b2eb42ec`, `stroke="var(--color-surface)"
    stroke-width="1.5"` sur toute part) : une part de moins d'environ 0,3 % disparaissait sous son liseré. Corrigé par un angle
    dessiné minimal (`MIN_SLICE_ANGLE`, pris sur la plus grande part), pas de liseré sous `SEPARATED_MIN_ANGLE`, et la peinture des
    parts étroites en dernier (`drawOrder`) — C-e3410-9. Rouge d'abord : visibilité prouvée par la corde de l'arc intérieur (au moins
    une unité), l'absence de liseré et la place de dernier `path` ; mutations MV1 à MV3, MV6, MV7 tuées.
  - **E1-2** LOW : rendu inatteignable par l'angle minimal (corde intérieure ≥ 1,15 unité) ; une garde sur les points intérieurs,
    ajoutée puis retirée, survivait à sa mutation (MV4, équivalente) — test « aucun arc intérieur aux extrémités confondues ».
  - **E1-3** LOW : plus de part pleine par arrondi à côté d'une part minuscule (même correctif), test réécrit.
  - **B1-2** LOW : test de `centerFontSize` sans recopie de la formule (C-e3410-11). **B1-4** LOW : `centerLines`, écart
    proportionnel au libellé, bloc centré (C-e3410-10), mutation MV5 tuée. **B1-3** LOW : CHANGELOG aligné sur le code (base
    différente du total, « par exemple » un compte créditeur). **A1-1** LOW : M6-bis jouée sur la clé plurielle, tuée. **A1-2**
    LOW : ligne du `.tex` coupée.
  - Le manuel et le CHANGELOG disent la part élargie au dessin. Aucun finding reclassé ni laissé ouvert ; aucune issue à ouvrir.
  - Trend : P1 0 C / 0 H / 1 M / 7 L → P2 (passe complète Opus, prompt `34-10-graphiques-couleurs-unies-review-prompt-p2.md`) à
    lancer : la remédiation touche la production.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5, en autonomie. Registre des choix neufs : `e34-choix-autonomes-e3410.md`, C-e3410-1 à C-e3410-8.

### Completion Notes List

**Étape 0.**
- **P7-F1** (LOW) : la phrase de l'AC-4 qui précède l'ajout P6 est qualifiée (« en valeur… et qu'il y a des postes »). Ligne
  lue : `ExpensePieCard.svelte:62` (à `cdf13639`) `{#if hasPosts}`, et `:173-181`, la note des comptes créditeurs hors de ce bloc.
- **P7-F2** (LOW) : comparaison en valeur par `differentAmounts(a, b)` (`!new Big(a).eq(new Big(b))`, `homepage-charts.ts`),
  fixture « `100.00` / `100` » (`HomepageCharts.test.ts`, « un compte, totaux égaux en valeur ») et test pur (`'30'` / `'30.00'`).
  Lu : `homepage-summary.api.ts:45,48`, les deux champs sont des `string`.
- **T0, contrastes** recalculés (formule WCAG de `contrast.ts`) avant d'être écrits au commentaire d'`app.css` : sur blanc, vert
  3,30, rouge 6,47, bleu 8,72, violet 5,70, cyan 3,68, gris 4,76 ; vert/rouge 1,96 — identiques à la fiche. Le test AC-7 les
  borne (≥ 3, ≥ 1,5).
- Journaux et captures sont sous `kesh-gate-logs/e3410/` (clé du prompt), non `e342/` comme l'écrit la fiche.

**Rouges** (`/home/gcorbaz/devel/kesh-gate-logs/e3410/rouges.log`) : tests écrits sur `cdf13639` avant le code ;
`HomepageCharts.test.ts` : 8 rouges (AC-4, AC-5, AC-8) ; `homepage-charts.test.ts` rouge au chargement du module
(`PIE_GEOMETRY` absent : `Cannot read properties of undefined (reading 'innerR')`) — le rouge test par test de ce fichier n'a
donc pas été relevé avant le code ; ce sont les mutations qui montrent que ses tests mordent.

**Fait.**
- `app.css` : `--color-chart-revenue`, `--color-chart-expense`, `--color-chart-1` à `-5`, en hexadécimal.
- `homepage-charts.ts` : `ChartStyle = { color, token }`, palette de l'AC-1 ; `PIE_GEOMETRY` (trou à 60 %), `fullRing`,
  `centerFontSize` (C-e3410-5), `differentAmounts` ; `pieSlices` rend des secteurs d'anneau ; `ChartPattern`, `patternId`,
  `chartFill` retirés.
- `MonthlyBarsCard.svelte` : barres et légende en couleurs unies. `ExpensePieCard.svelte` : anneau, liseré
  `var(--color-surface)` de 1,5 (C-e3410-4, -8), part pleine en `circle` au rayon médian, centre `homepage-chart-pie-total`
  (libellé et montant, `--color-text`), `<desc>` au total des dépenses avec l'ordre des parts (C-e3410-2, -3),
  `homepage-charts-pie-desc-base` et `homepage-charts-pie-base-note` sous `hasPosts && differentAmounts(…)`.
- `ChartPatterns.svelte` supprimé par `git rm` (prescrit par la fiche, T2).
- i18n : trois clés neuves (`homepage-charts-pie-total-label`, `-pie-desc-base`, `-pie-base-note`), deux récrites, quatre
  locales, replis égaux. `git grep -n -F` des quatre formes « { $base } » de l'AC-4 sur `frontend crates` : vide ;
  `git grep -n 'ChartPatterns\|patternId\|chartFill\|<pattern' -- frontend/src ':!*.test.ts'` : vide.
- Compteurs des gardes i18n recomptés à la source : `i18nMsg(` dans `ExpensePieCard.svelte` 15 (`cdf13639`) → 18, d'où
  `sitesTotal` 2922 → 2925 ; candidate neuve `totalLabel`, d'où 85 → 86 et `conforme` 71 → 72.
- E2E `homepage-charts.spec.ts` : aide `clickRing` (C-e3410-7), tests 3 et 6b.
- Manuel utilisateur `.tex`, § *Revenus, dépenses et principaux postes* (barres, anneau, couleurs, ordre, total du centre égal
  aux barres, base des parts). PDF non régénéré (C-e342-20, à l'intégration).
- CHANGELOG : `## [0.17.6] — Non publié`, *Changé*.

**Tests**, de `cdf13639` à `HEAD` : `homepage-charts.test.ts` 33 → 41 `it(` ; `HomepageCharts.test.ts` 18 → 27.

**Mutations** (`/home/gcorbaz/devel/kesh-gate-logs/e3410/mut/*.log`) : 14 jouées, 14 tuées, aucune équivalente — M1, M2,
M2-bis, M3 (`PIE_GEOMETRY.innerR` nul), M3-b (rayon intérieur nul dans `pieSlices`), M4, M5, M6 (repli Svelte), M6-ftl
(catalogue fr, tuée par les gardes i18n de `src/lib/shared` : variables du repli et du catalogue, jeu de variables des quatre
locales), M7, M8-inverse, M8-sans-condition, M8-bis, M9-chaines (P7-F2). Restauration par `git checkout --` puis `touch`.

**Gates, au dernier commit de code `7a2cebcd`** (journaux `/home/gcorbaz/devel/kesh-gate-logs/e3410/`) :
- `npm run check` : 0 erreur, 29 avertissements (aucun dans `homepage/`) ; `lint-i18n-ownership` : PASS ;
  `npm run test:unit` : **2898 passés, 222 fichiers** ; `npm run build` : vert.
- Premier Vitest complet, sur `ddc1606f` : 1 rouge, `jetons-non-definis.test.ts` (l'attribut SVG `stroke-linejoin` pris pour une
  classe), corrigé par `7a2cebcd` (C-e3410-8).
- Rust : aucun fichier `.rs` touché ; les catalogues `.ftl` le sont, d'où `cargo test -p kesh-i18n` (41 passés) et
  `cargo test -p kesh-api --test textes_coherents` (23 passés), et `cargo build -p kesh-api` pour l'E2E. **Ni `test-fast.sh`, ni
  `cargo build --workspace`, ni `gate-ci.sh`** (consigne de la story d'écran, disque partagé).
- **E2E complet** (`e2e-r1.log`, `kesh_e2e_e3410` recréée vide, port 3350, runner avec `KESH_INBOX_DIR` et
  `KESH_DOCUMENTS_DIR`) : **384 passés, 8 échecs, 18 ignorés**. Les sept KF-029 (#97) de `docs/testing.md` :
  `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`. Le huitième,
  `fiscal-year-closing-journey.spec.ts:169`, est un délai dépassé dans le `beforeEach` (`seedTestState`, 30 s) ; **rejoué seul**
  avec `homepage-charts.spec.ts` (`e2e-r2.log`) : **13 passés, 0 échec** — le huitième, variable, de `docs/testing.md`.
  Les tests de `homepage-charts.spec.ts` passent dans les deux runs.

**Captures** (`/home/gcorbaz/devel/kesh-gate-logs/e3410/captures/`, thème clair, base jetable `kesh_e3410`, données
synthétiques : revenus sur onze mois, sept comptes de charges, mars aux dépenses négatives, un compte au solde créditeur) :
`barres-`, `anneau-`, `accueil-{avant,apres}.png` ; `graphiques-deuteranopia-apres.png`, `graphiques-protanopia-apres.png`
(émulation CDP `Emulation.setEmulatedVisionDeficiency`, celle du panneau *Rendering* des DevTools). **Ce qu'on y lit**
(C-e342-18) : en deutéranopie, revenus et dépenses deviennent deux olives, la plus sombre pour les dépenses — séparées par la
clarté et la position, non par la teinte ; à l'anneau, rouge et vert (rangs 1 et 2) sont deux olives de clarté différente, bleu
et violet deux bleus, cyan un bleu clair ; le liseré blanc marque chaque limite, et le tableau donne le rang. Le centre lit
« Total des dépenses 25’125.00 », et la carte « Les parts portent sur 25’200.00. » après la note du compte créditeur.

**Écarts.**
- Consigne Edit : code, catalogues, manuel, CHANGELOG et fiche modifiés par l'outil Edit seulement. Le script de mutations
  (Python, hors dépôt) écrit puis restaure les fichiers mutés, comme le permet la consigne ; le script E2E de `kesh-gate-logs`
  a été retouché par une ligne Python (hors dépôt, non du code).
- `git rm` de `ChartPatterns.svelte` : prescrit par la fiche ; retrait suivi par git, réversible.

**Remédiation de la revue P1 (2026-11-11).**
- Rouges (`rouges.log`, section « Remédiation P1 ») : 7 tests purs et 1 test de rendu, écrits sur `b2eb42ec` avant le code.
- Tests, de `b2eb42ec` à `HEAD` : `homepage-charts.test.ts` 41 → 45 `it(` ; `HomepageCharts.test.ts` 27 → 28.
- Mutations (`mut/M6-bis.log`, `mut/MV*.log`) : 8 jouées, 7 tuées ; **MV4** (part pleine sur l'arc extérieur seul) a survécu
  contre le code qui portait la garde intérieure : mutation équivalente, la garde a été retirée (`11af9624`). MV1 rejouée ensuite :
  tuée, dont par le test E1-2.
- Incident : MV1 a été rejouée alors que `homepage-charts.ts` portait des retouches non commitées ; le `git checkout --` de
  restauration les a effacées. Elles ont été refaites à l'identique et commitées (`11af9624`) ; aucun effet sur le code livré, mais
  une mutation se joue désormais sur un arbre propre.
- Gates au dernier commit de code `11af9624` : `npm run check` 0 erreur (29 avertissements, aucun dans `homepage/`),
  `lint-i18n-ownership` PASS, `npm run test:unit` **2903 passés, 222 fichiers**, `npm run build` vert (`*-p1.log`).
  **E2E complet** (`e2e-r3.log`, base recréée vide, runner avec `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`) : **385 passés,
  7 échecs, 18 ignorés** — exactement les sept KF-029 (#97) : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`,
  `:92`, `onboarding.spec.ts:57`, `:77`, `:150` ; les onze tests de `homepage-charts.spec.ts` passent. Aucun fichier Rust touché
  par la remédiation : pas de gate backend.
- Captures de la part minuscule (scénario `minuscule` : un cinquième poste et « Autres » à 0,1 %) :
  `anneau-avant-p1-minuscule.png` (build de `b2eb42ec`) — les deux parts ne se voient pas, le violet touche le rouge ;
  `anneau-apres-p1-minuscule.png` — deux traits fins, cyan et gris, à midi, entre le violet et le rouge.

### File List

- `frontend/src/app.css`
- `frontend/src/lib/features/homepage/homepage-charts.ts`
- `frontend/src/lib/features/homepage/ExpensePieCard.svelte`
- `frontend/src/lib/features/homepage/MonthlyBarsCard.svelte`
- `frontend/src/lib/features/homepage/ChartPatterns.svelte` (supprimé)
- `frontend/src/lib/features/homepage/homepage-charts.test.ts`
- `frontend/src/lib/features/homepage/HomepageCharts.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`
- `frontend/tests/e2e/homepage-charts.spec.ts`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `docs/manual/fr/user-manual.tex`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/34-10-graphiques-couleurs-unies.md`
- `_bmad-output/implementation-artifacts/34-10-graphiques-couleurs-unies-review-prompt-p1.md`
- `_bmad-output/implementation-artifacts/34-10-graphiques-couleurs-unies-review-prompt-p2.md`
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e3410.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
