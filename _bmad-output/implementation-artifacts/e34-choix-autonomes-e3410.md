# Epic 34 — registre des choix autonomes, story 34-10 (`e3410`, #970)

*Ouvert le 2026-11-10 au développement (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e3410`, base `cdf13639`. Les choix de la
rédaction et de la validation sont au registre du groupe, `e34-choix-autonomes-e342.md` (C-e342-15 à C-e342-25). Mise à jour par
union, jamais par écrasement.*

### C-e3410-1 — Le nom de la clé du libellé du centre

- **Contexte** : l'AC-4 veut « Total des dépenses » au centre, et la tâche T3 le cite parmi les textes, sans nommer sa clé.
- **Retenu** : `homepage-charts-pie-total-label`, à côté des autres clés `homepage-charts-pie-*` ; quatre locales, repli égal au
  catalogue fr (de « Gesamtaufwand », it « Totale delle uscite », en « Total expenses »).
- **Écartée** : réutiliser `homepage-charts-legend-expenses` (« Dépenses ») : ne dit pas qu'il s'agit d'un total.
- **Réversible** : oui.

### C-e3410-2 — L'ordre des parts dans le `<desc>` : une phrase dans la clé plurielle seulement

- **Contexte** : l'AC-6 veut l'ordre des parts écrit dans le `<desc>` ; la fiche ne dit pas dans quelle clé.
- **Retenu** : une phrase à la fin de `homepage-charts-pie-desc` (« Les parts suivent l'ordre du tableau, dans le sens horaire à
  partir de midi. »). La clé `-one` ne la porte pas : avec un seul compte, il n'y a qu'une part et pas d'ordre à dire.
- **Écartées** : une clé à part jointe par le Svelte (une jointure de plus pour une phrase toujours présente au pluriel) ; la phrase
  dans les deux clés (fausse ou vide au singulier).
- **Réversible** : oui.

### C-e3410-3 — Forme des textes du `<desc>` après le passage au total des dépenses

- **Contexte** : les deux clés existantes passent de `{ $base }` à `{ $total }` (C-e342-25).
- **Retenu** : `-one` : « { $year } : un seul compte de charges, { $top } ; total des dépenses { $total }. » — le nom du compte
  d'abord, le total ensuite : avec un compte au solde créditeur, le total n'est plus le montant de ce compte, et la phrase ne doit
  pas les lier par « : ». Pluriel : « …, total des dépenses { $total }. Le plus gros : … ».
- **Réversible** : oui, textes.

### C-e3410-4 — Le liseré entre les parts : `var(--color-surface)`, 1,5 unité

- **Contexte** : l'AC-3 veut un trait de la couleur de fond de la carte, de 1 à 2 unités (C-e342-16).
- **Retenu** : `stroke="var(--color-surface)"` (`#ffffff`, le fond `bg-white` de la carte), `stroke-width="1.5"`,
  sur les seules parts en `path` ; une part pleine (`circle`) n'en a pas besoin, elle est seule.
  L'attribut `var(…)` suit le patron existant de `MonthlyBarsCard.svelte` (`stroke="var(--color-border)"`).
- **Écartée** : une constante `#ffffff` dans le module (une valeur de plus à tenir égale à un jeton, pour un trait non coloré).
- **Réversible** : oui.
- *(Annoté le 2026-11-11, revue de code P3, A3-1.)* **Support remplacé par C-e3410-12** (`e982f7e8`) : le liseré n'est plus le
  contour (`stroke`) des parts, qui n'en ont plus aucun, mais un `<line>` radial par limite, dans le groupe
  `homepage-chart-pie-separators`, peint au-dessus des parts. La valeur reste : `var(--color-surface)`, 1,5 unité
  (`SEPARATOR_WIDTH`).

### C-e3410-5 — `centerFontSize` : 0,6 em par caractère, 85 % du trou, bornes 8 et 22

- **Contexte** : la règle pure de C-e342-24 doit être fixée en nombres.
- **Retenu** : taille = min(22, max(8, 0,85 × diamètre du trou ÷ (0,6 × longueur))). Avec le trou de l'accueil (115,2 unités) :
  « 1.00 » → 22 ; « 999’999’999.99 » → 11,7 ; « Total des dépenses » → 9,1. Le libellé et le montant ont chacun leur taille.
  Mesure en unités du `viewBox` (200 pour 240 px au plus : 8 unités ≈ 9,6 px).
- **Écartée** : une seule taille pour les deux lignes (le libellé, plus long, imposerait sa petite taille au montant).
- **Réversible** : oui.

### C-e3410-6 — Le centre en `--color-text`, le montant en gras

- **Contexte** : l'AC-4 dit « en `--color-text` ».
- **Retenu** : les deux lignes en `var(--color-text)` ; le montant en `font-weight="600"`, le libellé en poids normal ; `pointer-events="none"`
  sur le groupe du centre, qui n'est pas une marque.
- **Réversible** : oui.

### C-e3410-7 — Les clics E2E sur l'anneau calculés depuis la boîte du `<svg>`

- **Contexte** : F10-1 : deux clics à porter dans l'anneau.
- **Retenu** : une aide `clickRing(page, angle)` dans `homepage-charts.spec.ts`, qui calcule le point à 80 % du rayon depuis la
  boîte du `<svg>` (`viewBox` 200, rayon 96, recopiés de `PIE_GEOMETRY` en commentaire) et clique par `page.mouse.click`. Test 3 :
  à neuf heures ; test 6b : au milieu de la première part (31,9 % du tour, angle 0,319 π).
- **Écartée** : `mark.click({ position })` sur la boîte de la part (la boîte d'un secteur d'anneau ne contient pas toujours son
  milieu au même endroit relatif) ; importer `PIE_GEOMETRY` dans la spec (aucune spec E2E n'importe de `src/`).
- **Réversible** : oui.

### C-e3410-8 — Pas de `stroke-linejoin` sur le liseré

- **Contexte** : le premier Vitest complet a rougi sur `jetons-non-definis.test.ts` : la garde lit tout le texte des `.svelte`, et
  prend l'attribut SVG `stroke-linejoin` pour une classe de couleur `stroke-<nom>` au nom non défini (« faux rouge bruyant »,
  angle mort documenté en tête de la garde).
- **Retenu** : retirer l'attribut, dont le rendu (angles arrondis du trait blanc) n'apporte rien à 1,5 unité.
- **Écartées** : inscrire `linejoin` à la table fermée `NON_DEFINIS` (elle tient des jetons à définir sous #645, non des attributs) ;
  modifier la garde (hors de la zone de la story).
- **Réversible** : oui.

## Ajouts de la revue de code P1 (2026-11-10)

### C-e3410-9 — Une part étroite reste visible : angle dessiné minimal, sans liseré, peinte en dernier

- **Contexte** : B1-1 = E1-1 (MEDIUM). Le liseré de 1,5 unité, centré sur le contour, mangeait 0,75 unité de chaque côté de
  chaque part : une part de moins d'environ 0,3 % disparaissait sous lui, cliquable mais invisible, et ses voisines étaient
  rognées.
- **Retenu**, deux gestes : (1) **angle dessiné minimal** `MIN_SLICE_ANGLE = 0,02` rad (≈ 1,15°, 0,32 % du tour, 1,15 unité à
  l'arc intérieur), l'écart étant pris sur la plus grande part ; les angles vrais (`startAngle`, `endAngle`) restent ceux du
  montant, le tableau porte les montants exacts ; (2) une part dessinée sous `SEPARATED_MIN_ANGLE = 0,06` rad (≈ 3,4°) ne porte
  **pas** de liseré et se peint **après** les autres (`drawOrder`) : le demi-liseré de la voisine qui mordait sur elle est
  recouvert, celui qui reste chez la voisine marque encore la limite. Une part au-dessus du seuil garde au moins 1,96 unité à
  l'arc intérieur une fois ses deux demi-liserés retirés. Effet de bord voulu : plus de part « pleine » par arrondi à côté
  d'une part minuscule (E1-3), et `full` teste aussi les extrémités intérieures (E1-2).
- **Écartées** : liseré en rayons tracés à part (même recouvrement d'une part plus étroite que le rayon) ; liseré seulement
  entre parts assez larges, sans angle minimal (une part de 0,0001 % resterait invisible, faute de largeur) ; `paint-order`
  (ne change rien au recouvrement par la voisine) ; renoncer au liseré (contraire à C-e342-16).
- **Coût** : le dessin d'une part minuscule n'est plus proportionnel (au plus 0,32 % du tour ajoutés par part, pris sur la
  plus grande) ; dit au manuel et au CHANGELOG.
- **Réversible** : oui, deux constantes.
- *(Annoté le 2026-11-11, revue de code P2.)* **Faux à sa date sur deux points.** (a) « `full` teste aussi les extrémités
  intérieures » : la garde, posée par `924bc54c`, a été retirée par `11af9624` (mutation MV4 équivalente) ; `full` ne lit que
  l'arc extérieur (B2-3, E2-4, A2-2). (b) « celui qui reste chez la voisine marque encore la limite » ne vaut que si la voisine
  porte un liseré : deux parts étroites voisines n'étaient séparées par rien (B2-1 = E2-1 = A2-1). Le geste (2) — pas de liseré
  sous `SEPARATED_MIN_ANGLE`, `drawOrder` — est **remplacé** par C-e3410-12 ; le geste (1) reste, son angle porté à 0,045 rad.

### C-e3410-10 — Écart vertical du centre proportionnel au libellé

- **Contexte** : B1-4 (LOW), écart fixe de 8 unités entre les deux lignes du centre.
- **Retenu** : `centerLines(cy, labelSize, amountSize)` : écart = 0,3 × taille du libellé + 1, bloc centré sur `cy` (hauteurs
  prises à 0,75 em).
- **Réversible** : oui.

### C-e3410-11 — Le test de `centerFontSize` ne recopie plus la formule

- **Contexte** : B1-2 (LOW), test circulaire.
- **Retenu** : on vérifie ce qui se décide sans mise en page — le montant plausible le plus long (`-999’999’999.9999`) n'est pas
  écrasé au plancher, un texte de trente caractères l'est (débordement assumé au-delà, non plausible), un montant court atteint
  le plafond. La largeur réelle des chiffres d'Inter n'est pas mesurée (jsdom) ; la capture la montre.
- **Écartée** : une mesure `getBBox` en E2E (C-e342-24 l'avait écartée pour un montant que le harnais n'atteint pas).
- **Réversible** : oui.

## Ajouts de la revue de code P2 (2026-11-11)

### C-e3410-12 — Le liseré en traits radiaux peints au-dessus des parts, un par limite

- **Contexte** : B2-1 = E2-1 = A2-1 (MEDIUM, trois lentilles). Avec C-e3410-9, deux parts étroites voisines n'avaient aucun
  liseré entre elles ; cyan et gris ne s'y séparaient qu'à 1,29:1 de clarté. B2-4 : une part sans contour dépassait ses
  voisines de 0,75 unité au bord.
- **Retenu** : les parts n'ont **plus de contour** ; le liseré est un `<line>` radial par limite, du trou au bord extérieur,
  de la couleur de la carte, `SEPARATOR_WIDTH` = 1,5, peint **après** toutes les parts, dans un groupe
  `homepage-chart-pie-separators` en `pointer-events="none"` (`pieSeparators`). Chaque limite est marquée quelle que soit la
  largeur des parts ; aucune part ne dépasse au bord. Pour qu'une part reste visible entre ses deux traits, `MIN_SLICE_ANGLE`
  passe de 0,02 à **0,045 rad** (≈ 2,6°, 0,72 % du tour) : à l'arc intérieur, 57,6 × 0,045 − 1,5 ≈ 1,09 unité visible. Une part
  seule n'a pas de séparateur.
- **Écartées** : un liseré sur un seul côté des parts étroites (règles par cas, et la dent du bord reste) ; un trait plus fin
  entre deux étroites seulement (deux largeurs de liseré) ; dire l'exception au lieu de la corriger (contraire à l'AC-3,
  « chaque limite »).
- **Coût** : l'élargissement au dessin monte à 0,72 % du tour par part minuscule (au plus 0,225 rad pris sur la plus grande,
  qui en a au moins 1,05) ; dit au manuel et au CHANGELOG.
- **Réversible** : oui.

### C-e3410-13 — Le libellé du centre jamais plus grand que le montant

- **Contexte** : E2-5 (LOW) : « Gesamtaufwand » (12,6) dépassait un montant de quatorze caractères (11,7).
- **Retenu** : `centerSizes` ramène la taille du libellé à celle du montant quand elle la dépasse.
- **Réversible** : oui.

## Ajouts de la revue de code P3 (2026-11-11)

### C-e3410-14 — Seule une part de montant strictement positif est dessinée

- **Contexte** : E3-1 (LOW) : une part nulle ou négative était élargie à `MIN_SLICE_ANGLE`, et une négative gonflait le déficit.
  Inatteignable aujourd'hui (le serveur ne range que des soldes strictement positifs).
- **Retenu** : `pieSlices` filtre `amount > 0` après avoir fixé rang et couleur sur l'indice du tableau : les parts restantes
  gardent le rang et la couleur de leur ligne.
- **Écartée** : le dire dans la doc seulement (préférence de l'orchestrateur pour le filtre, testé).
- **Réversible** : oui.

### C-e3410-15 — Rubrique « Modifié » au CHANGELOG

- **Contexte** : B3-3 (LOW) ; la 34-5 nomme la rubrique `### Modifié`, et les sections s'unissent à l'intégration.
- **Retenu** : `### Modifié`. *(Formule corrigée le 2026-11-11, passe ciblée P4, C4-1 : elle disait « C-e342-22 disait *Changé* :
  même intention, autre mot », ce qui taisait un changement de décision.)* C-e342-22 **retenait *Changé*** pour la 34-3 et la
  34-10 ; ce choix est **remplacé** par la rubrique *Modifié*, celle du CHANGELOG existant (aucun `### Changé`), sur décision de
  l'orchestrateur du 2026-11-11 ; C-e342-22 est annotée en ce sens, et la 34-3 suivra à l'intégration.
- **Réversible** : oui.

### C-e3410-16 — Le libellé du centre au plancher sur un écran étroit : accepté, sans issue

- **Contexte** : E3-2 (LOW, simple constat) : « Totale delle uscite » tombe à ≈ 8,6 unités, ≈ 10 px à la largeur ordinaire de
  la carte (240 px), ≈ 7 px si la carte se réduit à 160 px. Le texte tient dans le trou à toute échelle.
- **Retenu** : accepté. La page ne descend pas sous 1280 px (`min-w-[1280px]`, `(app)/+layout.svelte`, rappelé dans
  `MonthlyBarsCard.svelte`) : la carte garde ses 240 px, et le montant, la ligne principale, est plus grand ; le libellé dit ce
  que le tableau et le `<desc>` disent aussi.
- **Réversible** : oui.
