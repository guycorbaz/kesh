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
  `stroke-linejoin="round"`, sur les seules parts en `path` ; une part pleine (`circle`) n'en a pas besoin, elle est seule.
  L'attribut `var(…)` suit le patron existant de `MonthlyBarsCard.svelte` (`stroke="var(--color-border)"`).
- **Écartée** : une constante `#ffffff` dans le module (une valeur de plus à tenir égale à un jeton, pour un trait non coloré).
- **Réversible** : oui.

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
