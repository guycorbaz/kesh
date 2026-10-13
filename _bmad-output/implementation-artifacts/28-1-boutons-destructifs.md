# Story 28.1 : Les boutons d'action destructive se voient

## Status

**ready-for-dev** — fiche créée le 2026-10-13 en autonomie (`bmad-create-story`), sur `6f971d76`
(rétrospective de l'Epic 27), branche `story/28-1`, worktree `/home/gcorbaz/devel/kesh-28-1`. Choix consignés
au registre `epic-28-choix-autonomes-28-1.md` (**C-28-1-1 à C-28-1-25**), fusionné plus tard par union dans
`epic-28-choix-autonomes.md`. 13 AC (AC1-AC13), tâches T0-T10, mutations M1-M20. Validation P1 faite (Sonnet,
lentilles R et F) et remédiée — **périmètre réduit à #640** (C-28-1-17) ; validation P2 : prompt
`28-1-validate-prompt-p2.md`. PR : **`closes #640`** (et `refs #645 #646`, angles morts renvoyés).

## Story

**As a** utilisateur de Kesh (le Comptable comme l'Administrateur),
**I want** qu'un bouton qui supprime, archive, révoque, désactive ou annule se reconnaisse au premier coup d'œil
comme un bouton,
**so that** je ne passe plus à côté d'une fonction — comme *Supprimer l'import*, que je n'ai pas vu à la recette
de la v0.13.2 — et que je distingue d'un regard l'action irréversible de son *Annuler*.

Issue : [#640](https://github.com/guycorbaz/kesh/issues/640) (`bug`, **P3**, recette de la v0.13.2 du
2026-10-13). Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` (en cours d'écriture sur
`chore/epic-28-kickoff`). Cible : **v0.13.3**.

⚠️ **La cause écrite dans l'issue et dans l'epic est inexacte, et la fiche la corrige (C-28-1-1).** L'issue dit
« `bg-destructive/10` — un fond rouge à 10 % — sous un texte rouge ». En réalité **le jeton `destructive`
n'existe pas dans le thème** : `frontend/src/app.css:10-28` (`@theme`) définit `primary`, `secondary`, `success`,
`error`, `warning`, `info`, `surface`, `text`, `border`, `popover` — **aucun `--color-destructive`**. Tailwind v4
n'engendre une classe de couleur que pour un jeton défini : `bg-destructive/10`, `text-destructive`,
`hover:bg-destructive/20` ne produisent **aucune règle CSS**. Preuve au sol, sur le build du 2026-10-06 :

```sh
$ grep -c destructive frontend/build/_app/immutable/assets/*.css
…/0.CNEpAahO.css:0      (et 0 dans les six autres feuilles)
$ grep -o '\-\-color-[a-z-]*:' frontend/build/_app/immutable/assets/0.CNEpAahO.css | sort -u
--color-black: --color-border: --color-error: --color-info: --color-popover: … --color-white:
```

Le bouton n'a donc **ni fond, ni bordure, ni couleur propre** : il hérite du texte courant (`#1e293b`) — c'est
littéralement du texte en `font-medium`. Et le défaut ne touche pas que les boutons : **201 occurrences** du
jeton (`bg-`/`text-`/`border-`/`ring-destructive`) dans **42 fichiers** hors tests, dont **32 bandeaux
d'erreur** hors `components/ui/` (lignes portant à la fois `border-destructive`, `bg-destructive/10` et
`text-destructive`, dans un ordre variable), s'affichent aujourd'hui **sans aucun rouge**. `app.css` et
`button.svelte` n'ont pas bougé depuis les Stories 1.9 et 7-1 : le défaut est d'origine.

**Périmètre : #640, et seulement #640** (C-28-1-17). D'autres jetons employés par les composants ne sont pas
définis non plus (`muted`, `foreground`, `background`, `ring`, `input`, `accent`, `card`, `error-soft`,
`warning-soft`, `danger`…) : ils relèvent de **#645**, et la suppression sans confirmation d'une personne de
contact relève de **#646**. La fiche les cite comme angles morts (Dev Notes) et ne les traite pas.

## Critères d'acceptation

**AC1 — Trois jetons neufs dans le thème, mesurés (C-28-1-1, C-28-1-3, C-28-1-4).**

Dans `frontend/src/app.css`, bloc `@theme`, après `--color-error` :

```css
  /* Action destructive (Story 28-1, #640) — distincte de --color-error (#dc2626), trop
     clair pour un texte sur bandeau bg-destructive/10 (4,13:1 < 4,5). Ratios WCAG mesurés :
     blanc sur destructive 6,47:1 ; blanc sur destructive-hover 8,31:1 ; destructive sur
     blanc 6,47:1 ; destructive sur bandeau /10 (#f8e8e8) 5,45:1. */
  --color-destructive: #b91c1c;
  --color-destructive-foreground: #ffffff;
  --color-destructive-hover: #991b1b;
```

Le 5,45:1 du bandeau est calculé sur le mélange **non arrondi** de `#b91c1c` à 10 % sur blanc,
`(248 ; 232,3 ; 232,3)` — c'est ce que rend Chromium pour une couleur à alpha 0,1 composée sur blanc, et ce que
calcule le helper `blend` (AC8) ; arrondi à `#f8e8e8`, le ratio vaut 5,46. Le `/20` n'est plus cité : plus
aucun usage applicatif ne l'emploie (seuls l'ancienne variante et le `dark:` inutilisé de
`dropdown-menu-item.svelte:23`).

Après `npm run build`, `grep -c 'bg-destructive' build/_app/immutable/assets/*.css` rend au moins une
occurrence (la commande et sa sortie au Dev Agent Record). `--color-error` **ne change pas** (73 lignes
`text-error`/`bg-error`/`text-red-*` hors périmètre, cf. Dev Notes § Inventaire).

**AC2 — La variante `destructive` du `Button` partagé a un fond plein (C-28-1-2).**

`frontend/src/lib/components/ui/button/button.svelte:14` devient exactement :

```ts
destructive: "bg-destructive text-destructive-foreground hover:bg-destructive-hover focus-visible:border-destructive focus-visible:ring-destructive/40",
```

- **aucune** classe `dark:` dans la variante (C-28-1-7 : Kesh n'a pas de thème sombre, et le variant `dark` de
  Tailwind v4 est `@media (prefers-color-scheme: dark)` — la `dark:bg-destructive/20` d'aujourd'hui rendrait le
  bouton **à nouveau pâle** chez un utilisateur dont le système est en sombre, dès que le jeton existe). La
  **base**, elle, garde ses `dark:aria-invalid:ring-destructive/40` et `dark:aria-invalid:border-destructive/50`
  (`button.svelte:7`) : ils ne s'appliquent qu'à un bouton `aria-invalid`, ce qu'aucun bouton n'est —
  `grep -rn 'aria-invalid' src --include=*.svelte | grep -v components/ui` rend **11** lignes au 2026-10-13, toutes
  sur des champs (`opening-balances` ×4, `SetupForm` ×4, `ResetPasswordForm` ×2, `AccountAutocomplete` ×1) ; la garde
  AC7 (1) lit la **variante seule** pour cette raison (C-28-1-18) ;
- `focus-visible:border-destructive` (C-28-1-20) : la base porte `focus-visible:border-ring`, et le jeton `ring`
  **n'est pas défini** (#645). Aujourd'hui cette classe n'engendre aucune règle et la bordure reste
  `border-transparent` au focus ; le jour où #645 définit `ring`, un bouton rouge plein prendrait une **bordure
  bleue** au focus sans que rien ne le décide ici. La variante fixe donc sa bordure de focus à sa propre couleur
  (tailwind-merge garde la dernière classe du même groupe : `focus-visible:border-destructive` remplace
  `focus-visible:border-ring`, comme `focus-visible:ring-destructive/40` remplace `focus-visible:ring-ring/50`) :
  le rendu au focus de la variante ne dépend plus d'aucun jeton absent ;
- la classe de base (`button.svelte:7`) **ne change pas** : `disabled:opacity-50`, `outline-none`,
  `focus-visible:ring-3`, `active:…translate-y-px`, `transition-all` restent communs à toutes les variantes ;
- doc-comment `//` au-dessus de la ligne : renvoi à #640, à AC1 pour les ratios, et la phrase « pas de `dark:` :
  voir C-28-1-7 ».

**AC3 — Les quatre états se distinguent, mesurés (C-28-1-4, C-28-1-5, C-28-1-6, C-28-1-7).**

| État | Rendu attendu (style calculé, Chromium) | Contraste mesuré | Exigence |
|---|---|---|---|
| repos | fond `rgb(185, 28, 28)`, texte `rgb(255, 255, 255)` | 6,47:1 (texte) ; 6,47:1 (fond / page blanche) | 1.4.3 ≥ 4,5 ; 1.4.11 ≥ 3 |
| survol | fond `rgb(153, 27, 27)` | 8,31:1 | 1.4.3 ≥ 4,5 |
| focus clavier | `outline-style: solid`, `outline-color: rgb(59, 130, 246)` (règle globale `app.css:62-65`) ; `border-top-color: rgb(185, 28, 28)` (la bordure de focus de la variante, C-28-1-20 — non celle du jeton absent `ring`) | 3,68:1 contre la page (le contour est décalé de 2 px) | 2.4.7 ; 1.4.11 ≥ 3 |
| désactivé | `opacity: 0.5`, fond **toujours** `rgb(185, 28, 28)` (l'opacité s'applique à l'élément) | 2,53:1 effectif | **exempté** (1.4.3 : composant inactif) — distinct du repos (C-28-1-5) |
| repos, `prefers-color-scheme: dark` émulé | **identique** au repos clair | — | C-28-1-7 |

L'anneau `focus-visible:ring-destructive/40` (3 px, rouge à 40 %) n'est **pas** asserté : sa couleur passe par
`color-mix(in oklab, …)`, que `getComputedStyle` rend dans une syntaxe que `parseRgb` refuse (AC8) ; il est
vu sur la capture. Toutes les valeurs du tableau se lisent **après stabilisation** : la base porte
`transition-all` (150 ms par défaut, `tailwindcss/theme.css` `--default-transition-duration`), et une lecture
immédiate après `hover()` ou `focus` rendrait une couleur intermédiaire (AC8, C-28-1-19).

La règle globale `:focus-visible` est hors couche (`app.css:62`) et l'emporte sur `outline-none`, qui vit dans
`@layer utilities` : **à vérifier par l'E2E de l'AC8, non à supposer**. Si le style calculé dément cette
lecture, corriger (par exemple `focus-visible:outline-2 focus-visible:outline-primary-light` dans la variante)
et l'écrire au registre.

**AC4 — Le jeton défini rend leur rouge aux autres usages du jeton, sans édition de site (C-28-1-8).**

Définir `--color-destructive` active d'un coup tous les usages du jeton qui ne sont pas la variante (sur les
201 occurrences dans 42 fichiers, 29 sont dans 4 fichiers de `components/ui/`, dont la variante elle-même) : les 32
bandeaux d'erreur, les textes d'erreur `text-destructive` (`<p>`, `<span>`, `<td>`, `<li>`), les bordures de
champ en erreur (`border-destructive` sur `Input`, `aria-invalid:border-destructive` de `input.svelte`,
`select-trigger.svelte` — 11 sites `aria-invalid` hors `ui/`), la section `border-destructive` de
`settings/vat-rates/+page.svelte:356`, et les trois `<button>` bruts bordés en `text-destructive` (Dev Notes §
Inventaire, classe C ; le seul `text-destructive` de la classe D, *Annuler le rapprochement*, est migré par l'AC5). **Aucun de ces sites n'est édité** pour l'AC4 — c'était leur intention d'origine.

- Contraste garanti par AC1 : texte sur blanc 6,47:1, texte sur bandeau `/10` 5,45:1.
- Les specs E2E qui lancent axe **avec la règle `color-contrast` active** restent vertes : c'est le contrôle
  indépendant que le rouge neuf ne crée aucune violation là où un bandeau est affiché à l'analyse. Décompte au
  2026-10-13, depuis `frontend/` (C-28-1-25) :

  ```sh
  grep -l "\.analyze(" tests/e2e/*.spec.ts | wc -l          # 19 fichiers
  grep -n "disableRules" tests/e2e/*.spec.ts                # users:134, email-templates:203, reminders:298,
                                                            # invoices:457 (+ un commentaire, invoices:217)
  grep -n "\.analyze(" tests/e2e/invoices.spec.ts           # :96, :231 (sans disableRules), :458
  ```

  **16 fichiers** exercent `color-contrast` au moins une fois : `auth`, `bank-account-journal-link`,
  `bank-csv-import`, `bank-import`, `bank-import-confirms`, `contacts`, `dunning`, `homepage-dashboard`,
  `homepage-settings`, `products`, `reconciliation`, `reconciliation-manual`, `reconciliation-rules`,
  `reconciliation-split`, `reports` (15 fichiers sans `disableRules` ; `reconciliation*` ×3 et `reports` passent par
  `withTags(['wcag2a', 'wcag2aa'])`, qui inclut `color-contrast`) **et** `invoices` (deux de ses trois analyses,
  `:96` et `:231`). **3 fichiers** la neutralisent entièrement (`users`, `email-templates`, `reminders`) :
  ils ne contrôlent rien du rouge neuf. La spec neuve de l'AC8 en ajoute une dix-septième.

**AC5 — Cinq boutons stylés à la main passent au `Button` partagé : quatre à `destructive`, un à `outline`
(C-28-1-9, C-28-1-21).**

| Site | Aujourd'hui | Devient |
|---|---|---|
| `routes/(app)/settings/fiscal-years/+page.svelte:603` (`fiscal-year-close-confirm`) | `<Button class="bg-red-600 hover:bg-red-700">` | `<Button variant="destructive">`, `class` retiré |
| `routes/(app)/settings/email-templates/+page.svelte:439` (`email-template-restore-confirm`) | `<Button class="bg-red-600 hover:bg-red-700">` | `<Button variant="destructive">`, `class` retiré |
| `routes/(app)/bank-import/[id]/+page.svelte:141-149` (`detail-tx-cancel-reconciliation`) | `<button class="ml-2 text-destructive underline">` — se lit comme un lien | `<Button variant="outline" size="sm" class="ml-2">` — la forme exacte du même bouton sur la fiche de la facture (C-28-1-21) |
| `lib/features/contacts/ContactPersonsManager.svelte:112-119` (`contact-person-delete-{id}`) | `<button class="text-red-600 hover:underline">` | `<Button variant="destructive" size="xs">` |
| `lib/features/bank-import/BankProfileList.svelte:75-83` (`bank-import-profile-delete-button`) | `<button class="ml-2 text-error">` | `<Button variant="destructive" size="xs" class="ml-2">` |

`data-testid`, `onclick`, `disabled`, libellé `i18nMsg` (clé et repli) **inchangés** ; le `Button` partagé pose
`type="button"` par défaut (`button.svelte:51`, `type = "button"`). Un seul de ces sites est rendu **dans un
`<form>`** : `ContactPersonsManager`, monté à `contacts/+page.svelte:1099` dans le formulaire d'édition
(`:824-1116`) ; son `<button>` actuel est déjà `type="button"`, et le `Button` le reste — aucun envoi du
formulaire n'est introduit. Import `import { Button } from '$lib/components/ui/button';` ajouté là où il manque.

*Annuler le rapprochement* passe à `outline` et non à `destructive` (C-28-1-21) : le manuel promet « le même
bouton » sur le détail de l'import et sur la fiche de la facture (`user-manual.tex:2019-2020`), où il est un
`<Button variant="outline" size="sm">` (`InvoiceSettlements.svelte:106-111`, `invoice-settlement-cancel-reconciliation`) ;
c'est un déclencheur qui ouvre `CancelReconciliationDialog`, dont la **confirmation** est, elle, `destructive`
(`CancelReconciliationDialog.svelte:137-138`, classe A). Le défaut de l'issue — « se lit comme du texte » — est
corrigé par la bordure et la forme de bouton.

**AC6 — Inventaire clos des boutons rouges posés à la main : une garde Vitest (C-28-1-11, C-28-1-22).**

Fichier neuf `frontend/src/lib/components/ui/button/boutons-rouges-inventory.test.ts`, sur le patron de
`frontend/src/lib/features/bank-accounts/bank-name-inventory.test.ts` (Story 26-2b) :

- parcourt **tous** les `.svelte` de `src/` (hors `.test.`), les analyse par `parse(source, { modern: true })` de
  `svelte/compiler` (pas d'expression régulière sur des balises : un `onclick={() => …}` contient un `>`) ;
- pour chaque élément `button` (`RegularElement`) et chaque composant `Button` (`Component`), lit le **texte
  source** de ses attributs `class` (littéral **ou** expression) et de ses directives `class:` ;
- relève tout jeton `(text|bg|border|ring|outline)-(destructive|error|red-\d{2,3})(/\d+)?`, préfixes de variante
  compris (`hover:text-destructive`) ;
- exige que l'ensemble `{ clé → jetons }` **égale** une table fermée, chaque entrée avec son motif. La **clé**
  est non ambiguë (C-28-1-22) : `<chemin relatif à src/>#<data-testid>` quand l'élément porte un `data-testid`
  — texte source de la valeur, gabarit compris (`api-keys-revoke-{k.id}` s'écrit tel quel, `settings/api-keys/+page.svelte:379`) —, sinon
  `<chemin>@<élément>:<rang>`, le rang comptant à partir de 1 les éléments **sans** `data-testid` porteurs
  d'un jeton rouge du même fichier, dans l'ordre du source. Pas de numéro de ligne (une édition sans rapport
  ferait rougir la garde). Les deux `<button>` de `supplier-invoices/[id]/+page.svelte` (`:410`, `:457`) ont
  deux `data-testid` distincts ; les trois icônes ✕, sans `data-testid`, sont chacune seule dans son fichier
  (`@button:1`). La garde **échoue** si deux éléments produisent la même clé. Après la story, la table contient
  **exactement neuf entrées** : les six déclencheurs bordés de la classe C et les trois icônes ✕ de la classe E
  (Dev Notes § Inventaire). Une entrée en trop **ou** en moins fait rougir (les deux sens, M12/M13) ;
- **sites migrés** (C-28-1-22) : une seconde table fermée, des cinq sites de l'AC5, `{ chemin#data-testid →
  { élément: 'Button', variant, class } }` — `variant` **littéral** attendu (`destructive` ×4, `outline` ×1),
  `class` attendu exactement (`ml-2` ou absent). La garde retrouve l'élément par son `data-testid` et exige
  qu'il soit un composant `Button` (non un `<button>`), avec ce `variant` et cette `class`. C'est ce qui fait
  rougir le retour d'un site à un `<button class="ml-2">` nu ou à un `Button` sans `variant` — invisibles à la
  première table, qui ne relève que les jetons rouges (M18) ;
- doc-comment : ce qu'elle voit, ce qu'elle **ne voit pas** (rouge passé par une variable de script, par `cn(…)`
  dans le `<script>`, par un `style=`, ou porté par un `<a>`/`<div role="button">` — aucun aujourd'hui, vérifié),
  et comment ajouter une entrée légitime (« avec son motif : c'est le but — le changement se voit en revue »).

La variante `destructive` du `Button` est la **seule** source du rouge d'un bouton hors table. La première
table ne contrôle pas `variant=`, qui n'est pas une classe ; la seconde le contrôle pour les **cinq sites
migrés seulement** — les 24 sites de la classe A ne sont pas tenus par une table (un `variant="destructive"`
retiré de l'un d'eux ne rougit aucune garde : angle mort 7).

**AC7 — Une garde Vitest qui rougit si la variante redevient pâle (C-28-1-11, C-28-1-12, C-28-1-18, C-28-1-23).**

Fichier neuf `frontend/src/lib/components/ui/button/button-destructive.test.ts` :

Les points (1) et (2) lisent **la variante seule**, `buttonVariants.variants.variant.destructive` — la chaîne
déclarée, que tailwind-variants expose sur le composant (`component.variants = variants`,
`node_modules/tailwind-variants/dist/chunk-RZF76H2U.js:254`) —, **non** la sortie fusionnée
`buttonVariants({ variant: 'destructive' })`, qui contient la base : celle-ci porte des `dark:aria-invalid:…`,
`bg-clip-padding`, `text-sm`, `focus-visible:ring-ring/50`, `focus-visible:border-ring`, et ferait rougir (1) et
(2) dès la première exécution sur des classes que la story ne touche pas (C-28-1-18). Si `.variants` n'est pas
exposé au moment de l'implémentation, lire la ligne `destructive:` de `button.svelte` par `readFileSync` (même
contenu, même garde) et l'écrire au Debug Log. La sortie fusionnée ne sert qu'à l'assertion (1-bis).

1. La variante contient les jetons `bg-destructive`, `text-destructive-foreground`, `hover:bg-destructive-hover`
   et `focus-visible:border-destructive` **comme mots entiers** (`/(^|\s)bg-destructive(\s|$)/` —
   `bg-destructive/10` ne compte pas) ; ne contient **aucun** jeton préfixé `dark:` ; ne contient aucun
   `bg-transparent` ni `bg-destructive/<n>`.
   (1-bis) La sortie fusionnée `buttonVariants({ variant: 'destructive' })` contient `bg-destructive` et
   `focus-visible:border-destructive` comme mots entiers, et **ne contient plus** `focus-visible:border-ring` :
   c'est la preuve que tailwind-merge a remplacé la classe de la base (C-28-1-20).
2. `readFileSync('src/app.css')` : les trois jetons de l'AC1 sont définis dans `@theme` (expression extraite par
   `--color-destructive:\s*(#[0-9a-f]{6})`) ; **tout** jeton de couleur employé par la variante — chaque classe de
   la variante, préfixes de variante (`hover:`, `focus-visible:`) et opacité (`/40`) retirés, de la forme
   `(bg|text|border|ring)-X` → `--color-X` — est défini. C'est la classe de défaut d'origine, bornée à la variante.
3. Contrastes recalculés **depuis les valeurs lues** dans `app.css`, par le helper de l'AC8 :
   `destructive-foreground`/`destructive` ≥ 4,5 ; `destructive-foreground`/`destructive-hover` ≥ 4,5 ;
   `destructive`/`#ffffff` ≥ 4,5 (texte d'erreur) ; `destructive`/mélange 10 % sur blanc ≥ 4,5 (bandeau) ;
   `destructive`/`#ffffff` ≥ 3 (limite du bouton, 1.4.11).
4. **Facultatif — garde d'ensemble clos des classes de couleur sans jeton défini, en table fermée
   (C-28-1-23).** Fichier `frontend/src/lib/components/ui/jetons-non-definis.test.ts`
   (`// @vitest-environment node`). Elle relève dans tous les `.svelte` et `.ts` de `src/` (hors `.test.`) les
   classes candidates `(bg|text|border|ring|outline|fill|stroke|divide|placeholder|decoration|caret|accent)-<nom>`
   (préfixes de variante et opacité retirés), et demande **à Tailwind lui-même** lesquelles n'engendrent aucune
   règle, en compilant `src/app.css` par `compile` de `@tailwindcss/node` (exporté, `dist/index.d.ts:256`) puis
   `build([candidat])`, ou par `__unstable__loadDesignSystem(...).candidatesToCss(...)` (rend `null` pour un
   candidat non résolu). Ce n'est **pas** une liste de formes connues : c'est l'inventaire des non-résolus,
   décidé par le compilateur (CLAUDE.md, « inventorier les sites NON RÉSOLUS »). L'ensemble des **noms** non
   résolus doit **égaler** une table fermée, chaque nom renvoyé à **#645** ; attendu d'après le relevé
   indépendant des lentilles P1 : `muted`, `muted-foreground`, `foreground`, `background`, `ring`, `input`,
   `accent`, `accent-foreground`, `card`, `error-soft`, `warning-soft`, `danger`. La garde rougit (a) si un nom
   **neuf** non défini apparaît, (b) si un nom listé devient défini sans que la table soit mise à jour (#645 la
   videra). Tout écart entre la sortie du compilateur et la liste attendue ci-dessus est **examiné nom par nom**
   et écrit au Debug Log avant d'être recopié dans la table — la table ne se remplit pas depuis la sortie de la
   garde qu'elle contrôle (le défaut de l'AC7 de l'Epic 23, vert par construction). Si aucune des deux API ne se
   charge sous Vitest, le point (4) est abandonné et le Debug Log le dit : la décision de l'orchestrateur le
   rend facultatif, et les points (1) à (3) couvrent seuls #640.

**AC8 — Recette visuelle : une spec Playwright qui mesure le style calculé et laisse des captures (C-28-1-10,
C-28-1-19, C-28-1-22).**

Fichier neuf `frontend/tests/e2e/destructive-buttons.spec.ts`, et helper `frontend/tests/e2e/helpers/contrast.ts`
(fonctions pures `parseRgb`, `relativeLuminance`, `contrastRatio`, `blend` ; tests unitaires
`frontend/tests/e2e/helpers/contrast.test.ts` — Vitest les ramasse par `tests/**/*.test.ts`, Playwright les
ignore par `testMatch`). Chaque scénario tourne **deux fois**, `page.emulateMedia({ colorScheme: 'light' })` puis
`'dark'` ; le passage sombre ne porte d'exigence que sur la **variante destructive** (identique au clair,
C-28-1-7) — les `dark:` d'autres composants (`outline` : `dark:bg-input/30`, `dark:border-input`, inertes tant que
`input` n'est pas défini, #645) ne sont pas assertés, sinon par « fonds différents » en (b)/(d).

**Lecture stabilisée, obligatoire (C-28-1-19).** La base porte `transition-all` (150 ms) : une valeur lue par
`evaluate(el => getComputedStyle(el)…)` juste après `hover()` ou `focus` est une couleur **intermédiaire**. Toute
assertion de valeur passe par `await expect(locator).toHaveCSS('<propriété>', '<valeur>')` (réessai intégré) ;
une valeur qui sert ensuite à un calcul (contraste) se lit par `await expect.poll(() => lire(locator)).toBe(<valeur
attendue>)`, puis le calcul porte sur cette valeur stabilisée. Repos et désactivé n'ont pas de transition en
cours, mais suivent la même forme (un seul idiome dans la spec).

**`parseRgb` (helper).** Accepte `rgb(r, g, b)` et `rgba(r, g, b, a)` et rend `{ r, g, b, a }` (`a = 1` pour
`rgb(`) ; toute autre syntaxe — `color(srgb …)`, `oklab(…)`, `oklch(…)`, que Chromium rend pour une couleur
issue de `color-mix(in oklab, …)` comme `bg-destructive/10` — **lève** une erreur nommant la chaîne reçue, au
lieu de rendre une valeur fausse. `rgba(0, 0, 0, 0)` (fond transparent : le cas de M2) se lit et a `a = 0`.
`contrast.test.ts` couvre : les deux formes acceptées, le refus d'une forme `oklab(` et de `color(`, et trois
ratios connus (blanc/noir 21 ; blanc/blanc 1 ; blanc/`#b91c1c` 6,47 à 0,01 près) — c'est ce qui tue M15 et M16.

Scénarios :

- **(a) `/design-system`** (route publique, hors `(app)`) — section *Boutons* : le bouton destructif au repos,
  survolé (`hover()`), focalisé **au clavier** (`getByTestId('ds-destructive').focus()`, puis
  `keyboard.press('Shift+Tab')` et `keyboard.press('Tab')` pour que le focus vienne du clavier ;
  `evaluate(el => el.matches(':focus-visible'))` doit rendre `true` avant de lire le contour), désactivé (exemple ajouté par
  l'AC9) : valeurs du tableau de l'AC3 — dont, au focus, `outline-color` **et** `border-top-color`
  (`rgb(185, 28, 28)`, C-28-1-20) —, contraste recalculé dans le test depuis la valeur stabilisée ≥ 4,5
  (repos, survol) ;
- **(b) détail d'un import** — *Supprimer l'import* (`bank-import-delete-button`) : fond **non transparent**
  (canal alpha 1) et égal à `rgb(185, 28, 28)`, contraste ≥ 4,5 ; ouverture du dialogue
  (`bank-import-delete-dialog`) : *Annuler* (`outline`) et *Supprimer définitivement*
  (`bank-import-delete-submit`) ont des fonds **différents** ; `new AxeBuilder({ page })
  .include('[data-testid="bank-import-delete-dialog"]').withRules(['color-contrast']).analyze()` → 0 violation ;
- **(c) liste des profils** (`/bank-import/profiles`) — `bank-import-profile-delete-button` (taille `xs`,
  AC5) : fond `rgb(185, 28, 28)` ;
- **(d) dialogue d'archivage d'un compte** (`/accounts`, plan comptable du seed `with-company`) — ouvert par
  `account-row-<numéro>-archive-button` (`accounts/+page.svelte:499`, déclencheur `ghost`, présent seulement si
  `canModify()` et compte actif ; prendre le premier :
  `page.locator('[data-testid^="account-row-"][data-testid$="-archive-button"]').first()`), le bouton
  `account-archive-dialog-confirm` (`:742`) à côté d'`account-archive-dialog-cancel` (`:740`, `autofocus`) : mêmes
  assertions que (b), axe compris ; le dialogue est **refermé par *Annuler*** (rien n'est archivé).
- **(e) *Annuler le rapprochement*, dans la spec existante** `tests/e2e/reconciliation-cancel.spec.ts` (C-28-1-22) :
  juste avant le clic de `:139`, une assertion `await expect(row.getByTestId('detail-tx-cancel-reconciliation'))
  .toHaveCSS('border-top-width', '1px')` **et** `toHaveCSS('border-top-color', 'rgb(226, 232, 240)')`
  (`--color-border`, la bordure de la variante `outline`). Aujourd'hui `border-top-width` vaut `0px` : le
  préflight pose `border: 0 solid` (`tailwindcss/preflight.css:15` — le **style** est donc déjà `solid`, ne pas
  l'asserter) et le `<button>` brut n'a pas de classe `border` ; l'assertion rougit en T0.
  Le montage de rapprochement de cette spec est réutilisé tel quel ; aucune autre ligne n'en change.

Le cinquième site de l'AC5, `contact-person-delete-{id}`, n'a **aucune** spec E2E qui le monte (`grep -rn
"contact-person-delete" frontend/tests` → rien) : il est couvert **par transitivité** — la garde « sites migrés »
de l'AC6 prouve qu'il est un `Button variant="destructive" size="xs"` sans `class`, et le scénario (c) prouve
qu'un tel bouton rend un fond `rgb(185, 28, 28)`. Angle mort assumé (Dev Notes, angle mort 8) : un style propre à
son conteneur (dialogue d'édition du contact) qui l'écraserait ne serait vu que par l'œil.

⚠️ **Aucun sélecteur sur un libellé** : la garde `src/lib/shared/e2e-selecteurs-traduits.test.ts` (KF-043, #326)
l'interdit — `data-testid` seulement. C'est pourquoi (d) vise les comptes et non les contacts : le dialogue
d'archivage d'un contact (`contacts/+page.svelte:1135-1140`) n'a pas de `data-testid`.

**Captures** : chaque scénario prend `locator.screenshot()` du bouton **et** du dialogue (ou de la section),
**avant** ses assertions, sous le nom `<scénario>-<clair|sombre>.png`, dans
`process.env.KESH_E2E_CAPTURES_DIR` s'il est défini, sinon `testInfo.outputPath()`. Les captures **ne sont pas
versionnées** et ne servent **pas** de référence de comparaison (pas de `toHaveScreenshot`, C-28-1-10) : elles
sont la pièce de la recette humaine, jointe au prompt de revue et à la PR par l'orchestrateur. Données des
scénarios **synthétiques** (fixtures du dépôt, contact inventé).

Le montage d'import de (b) et (c) — compte bancaire, profil `E2E suppression`, import par l'écran — est
aujourd'hui écrit dans `tests/e2e/bank-import-delete.spec.ts:35-85` (`login`, `mount`, `importByScreen`) : il est
**extrait** dans `tests/e2e/helpers/bank-import-fixtures.ts` et les deux specs l'importent (C-28-1-16 — une spec
n'importe pas une autre spec : Playwright y enregistrerait ses tests deux fois).

**AC9 — La page `/design-system` montre la variante dans tous ses états (C-28-1-10).**

`frontend/src/routes/design-system/+page.svelte:66-73` (section *Boutons*) : ajouter
`<Button variant="destructive" disabled data-testid="ds-destructive-disabled">Destructive désactivé</Button>` et
`<Button variant="destructive" size="xs" data-testid="ds-destructive-xs">Destructive xs</Button>`, et poser
`data-testid="ds-destructive"` sur l'exemple existant (`:69`). Le pavé de palette (`:39-47`) gagne un carré
`bg-destructive` « Destructive ». Le bouton *Toast erreur* (`:170`) reste tel quel (page de démonstration).

**AC10 — Manuels : « grisé » devient « désactivé », PDF régénérés (C-28-1-5).**

Avec le nouveau style, un bouton destructif désactivé est un rouge estompé, non un gris :

- `docs/manual/fr/user-manual.tex:1739` — « Sinon le bouton est grisé, avec le nombre… » → « Sinon le bouton est
  **désactivé**, avec le nombre… » ;
- `docs/manual/fr/admin-manual.tex:1985` — « le bouton est grisé avec leur nombre » → « le bouton est
  **désactivé** avec leur nombre ».

Le reste du manuel est vrai au code : il ne décrit la couleur d'aucun bouton (`grep -niE "rouge|couleur"
docs/manual/fr/user-manual.tex` ne rend rien) et ses 25 emplacements de capture sont tous au
`_placeholder.png` (aucune capture réelle à refaire). `docs/manual/fr/user-manual.tex:1107` (« Bouton grisé avec
une infobulle », envoi d'e-mail, variante `default`) n'est **pas** touché. Les deux PDF sont régénérés (`make fr`
dans `docs/manual/`, sous `flock`) et contrôlés aplatis :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -c "le bouton est désactivé, avec le nombre"`
→ 1 ; même contrôle, `admin-manual.pdf`, « le bouton est désactivé avec leur nombre » → 1 ; et
`grep -c "le bouton est grisé, avec le nombre"` → 0. Le CHANGELOG de la 0.13.2 écrit « grisé » deux fois pour ce
même bouton (`CHANGELOG.md:19` et `:27`) : historique, non touché. `user-manual.tex:1739` coupe la phrase après
« avec le » (« nombre » à la ligne suivante) : la substitution ne vise que le mot `grisé`. `user-manual.tex:2010`
(« grisée », liste de règles, hors bouton) n'est pas touché non plus.

**AC11 — Spec UX : la palette nomme le rouge d'action (C-28-1-14).**

`_bmad-output/planning-artifacts/ux-design-specification.md` § *Système de Couleurs* : une ligne
`| **Action destructive** | Rouge foncé | `#b91c1c` | Boutons qui suppriment, archivent, révoquent ; survol `#991b1b` |`
après la ligne *Erreur*, et sous *Ratios de contraste WCAG AA* : « Bouton destructif (blanc sur rouge foncé) :
6,47:1 (AA) ; texte destructif sur bandeau d'erreur : 5,45:1 (AA) ».

**AC12 — CHANGELOG.**

`CHANGELOG.md` : section `## [0.13.3] — Non publié` (à créer au-dessus de `## [0.13.2] — 2026-10-13` si
absente — d'autres stories de l'epic la créent aussi, fusion par union), sous-section `### Corrigé` :

> - **Les boutons d'action destructive se voient ([#640](https://github.com/guycorbaz/kesh/issues/640)).**
>   *Supprimer l'import*, les confirmations de suppression, d'archivage, de révocation ou de désactivation
>   (factures, contacts, produits, comptes, écritures, comptes bancaires, utilisateurs, clés d'API, taux de TVA,
>   niveaux de relance, restauration d'une sauvegarde, réinitialisation de la démonstration, déverrouillage des
>   livres) s'affichaient comme du texte : leur couleur n'était pas définie dans le thème. Ils ont désormais un
>   fond rouge plein et un texte blanc (contraste 6,47:1), un survol plus foncé et un état désactivé estompé. Les
>   messages et cadres d'erreur, qui devaient être rouges et ne l'étaient pas, le deviennent. La suppression
>   d'une personne de contact et celle d'un profil d'import prennent la même forme de bouton ; *Annuler le
>   rapprochement*, dans le détail d'un import, qui se lisait comme un lien, prend la forme qu'il a déjà sur la
>   fiche de la facture.

**AC13 — Gates, au dernier commit de code.**

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E
**complet** (recette `docs/testing.md` § *Prérequis Playwright local*, port **3034**, base `kesh_e2e_281`) :
vert au sens de `docs/testing.md` § *Les échecs attendus* (référence 27-4 : 267 passés + 7 KF-029, à juger
fichier par fichier) **plus** la spec neuve ; `scripts/test-fast.sh` complet (le texte des manuels est lu par
`crates/kesh-api/tests/textes_coherents.rs`). Gates lourds sous `flock
/home/gcorbaz/devel/kesh-gate-logs/.gate.lock`. Le Dev Agent Record n'affirme que ce qui a tourné, chiffres
recomptés.

## Tasks / Subtasks

- [ ] **T0 — Captures « avant » et phase rouge** (AC8) : écrire `helpers/contrast.ts` (+ tests),
      `helpers/bank-import-fixtures.ts` (extraction, AC8), `destructive-buttons.spec.ts` et l'assertion (e) de
      `reconciliation-cancel.spec.ts` **avant** de toucher `app.css`, `button.svelte` ou les sites de l'AC5 ; les
      lancer seules avec `KESH_E2E_CAPTURES_DIR=/home/gcorbaz/devel/kesh-gate-logs/28-1-captures/avant`
      — elles **doivent rougir** sur les assertions de fond et de bordure (sortie copiée au Debug Log ; un
      `toHaveCSS` qui expire est le rouge attendu, une exception de `parseRgb` sur une syntaxe inattendue ne l'est
      pas et se documente) et laisser les captures « avant ». ⚠️ Le scénario (a) a besoin des exemples de
      l'AC9 : faire d'abord la T5.
- [ ] **T1 — Jetons** (AC1) dans `app.css`, doc-comment compris.
- [ ] **T2 — Variante** (AC2) dans `button.svelte:14`.
- [ ] **T3 — Gardes Vitest** (AC6, AC7) ; table de l'AC6 à neuf entrées, motifs écrits ; table « sites migrés »
      à cinq entrées ; AC7 (4) **facultatif** — tenté, et son sort (écrite, ou abandonnée avec la raison) écrit au
      Debug Log.
- [ ] **T4 — Cinq sites** (AC5) : quatre `destructive`, un `outline` (*Annuler le rapprochement*).
- [ ] **T5 — Page `/design-system`** (AC9).
- [ ] **T6 — Captures « après »** : la spec neuve verte, captures dans `…/28-1-captures/apres`.
- [ ] **T7 — Manuels** (AC10) + `make fr` sous `flock` + contrôles aplatis copiés au Debug Log.
- [ ] **T8 — Spec UX et CHANGELOG** (AC11, AC12).
- [ ] **T9 — Mutations M1-M20** (§ ci-dessous ; M20 seulement si AC7 (4) est écrite), chacune restaurée par `git checkout -- <fichier>` puis `touch`
      (mémoire *Mutation restaurée, binaire périmé*) ; tableau « mutation → test qui rougit » au Debug Log.
- [ ] **T10 — Gates** (AC13), au dernier commit de code ; grep de propagation du symptôme (§ ci-dessous).

## Mutations à tuer (T9)

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | variante rendue à sa valeur d'origine (`bg-destructive/10 … text-destructive …`) | AC7 (1), AC8 (a)(b) |
| M2 | `--color-destructive` retiré d'`app.css` | AC7 (2), AC8 (fond transparent) |
| M3 | `dark:bg-destructive/20` rajouté à la variante | AC7 (1) ; AC8 passe sombre |
| M4 | `--color-destructive: #dc2626` | AC7 (3) — bandeau 4,13 < 4,5 |
| M5 | `--color-destructive-foreground: #fca5a5` | AC7 (3), AC8 contraste |
| M6 | `hover:bg-destructive-hover` retiré | AC7 (1), AC8 survol |
| M7 | `--color-destructive-hover: #ef4444` | AC7 (3) (3,76 < 4,5) |
| M8 | variante `bg-transparent border-destructive text-destructive` (contour seul) | AC7 (1) |
| M9 | règle `:focus-visible` d'`app.css:62-65` retirée | AC8 focus |
| M10 | `disabled:opacity-50` retiré de la base | AC8 désactivé |
| M11 | `ContactPersonsManager.svelte` rendu à son `<button class="text-red-600 …">` | AC6 |
| M12 | un `<button class="text-destructive">` ajouté dans un fichier quelconque | AC6 (entrée en trop) |
| M13 | une icône ✕ de la table rendue neutre (classe retirée) sans toucher la table | AC6 (entrée en moins) |
| M14 | `fiscal-years/+page.svelte:603` rendu à `class="bg-red-600 hover:bg-red-700"` | AC6 |
| M15 | `contrastRatio` sans le `+ 0.05` | `contrast.test.ts` |
| M16 | `contrastRatio` qui ne trie pas les luminances (L2/L1) | `contrast.test.ts` |
| M17 | la garde AC6 ne lit plus les directives `class:` | AC6 (test de la garde sur un source synthétique `class:text-destructive`) |
| M18 | `variant` retiré d'un site migré (`BankProfileList.svelte`, `Button` sans `variant`), puis, second essai, le site `bank-import/[id]` rendu à un `<button class="ml-2">` nu | AC6 (sites migrés) ; AC8 (c), resp. (e) |
| M19 | `focus-visible:border-destructive` retiré de la variante | AC7 (1) et (1-bis) ; AC8 (a) focus (`border-top-color` ≠ `rgb(185, 28, 28)`) |
| M20 | *(si AC7 (4) est écrite)* un `<p class="text-inexistant">` ajouté dans un `.svelte`, puis, second essai, `--color-card: #ffffff` ajouté à `@theme` sans toucher la table | AC7 (4), dans les deux sens |

Une mutation qui survit est écrite au Debug Log avec sa raison (équivalente, ou test ajouté).

## Dev Notes

### Inventaire clos — tout bouton destructif ou rouge, au sol (2026-10-13, `6f971d76`)

Commandes, depuis `frontend/` :

```sh
grep -rn 'variant="destructive"' src | wc -l                    # 24
grep -rln 'variant="destructive"' src | wc -l                   # 17
grep -rnoE '(bg|text|border|ring)-destructive(/[0-9]+)?\b' src --include=*.svelte --include=*.ts \
  | grep -v '\.test\.' | wc -l                                   # 201 (42 fichiers)
grep -rnE '(text|bg|border)-(error|red-[0-9]+)' src --include=*.svelte | wc -l   # 73
```

**Classe A — `<Button variant="destructive">` : 24 occurrences, 17 fichiers, tous des `<Button>`** (vérifié
élément par élément ; aucun n'emploie la variante pour un **texte** d'erreur, ce que l'epic demandait de
vérifier ; aucun ne passe de `class`). Corrigés par AC2, **sans édition** :

| Fichier:ligne | Action | Dialogue avec *Annuler* `outline` ? |
|---|---|---|
| `lib/features/bank-import/DeleteBankImportAction.svelte:73` | *Supprimer l'import* (déclencheur, page) | — |
| `…/DeleteBankImportAction.svelte:139` | *Supprimer définitivement* | oui (`:135`) |
| `lib/features/reconciliation/CancelReconciliationDialog.svelte:138` | confirmer l'annulation du rapprochement | oui (`:133`) |
| `lib/features/admin-restore/AdminRestorePanel.svelte:182` | *Importer et remplacer l'installation* (déclencheur) | — |
| `…/AdminRestorePanel.svelte:218` | confirmer la restauration | oui (`ghost`) |
| `lib/shared/components/DemoBanner.svelte:68` | confirmer la réinitialisation | oui |
| `routes/(app)/accounts/+page.svelte:717` | archiver (formulaire d'édition) | — |
| `…/accounts/+page.svelte:742` | confirmer l'archivage | oui |
| `routes/(app)/bank-accounts/+page.svelte:586` | confirmer l'archivage | oui |
| `routes/(app)/contacts/+page.svelte:1138` | confirmer l'archivage | oui (`:1136`) |
| `routes/(app)/products/+page.svelte:760` | confirmer l'archivage | oui (`:758`) |
| `routes/(app)/invoices/+page.svelte:519` | confirmer la suppression | oui |
| `routes/(app)/invoices/[id]/+page.svelte:961` | *Supprimer* (déclencheur) | — |
| `…/invoices/[id]/+page.svelte:1088` | *Dévalider* (déclencheur) | — |
| `…/invoices/[id]/+page.svelte:1440` | confirmer la suppression | oui |
| `…/invoices/[id]/+page.svelte:1496` | confirmer la dévalidation | oui |
| `routes/(app)/journal-entries/[id]/+page.svelte:557` | confirmer la suppression | oui (`:553`) |
| `routes/(app)/settings/+page.svelte:568` | *Déverrouiller* les livres (`size="sm"`) | — |
| `routes/(app)/settings/api-keys/+page.svelte:404` | confirmer la révocation | oui |
| `routes/(app)/settings/dunning/+page.svelte:300` | confirmer la suppression d'un niveau | oui |
| `routes/(app)/settings/vat-rates/+page.svelte:366` | confirmer la désactivation | oui |
| `routes/(app)/users/+page.svelte:485` | confirmer la désactivation | oui (`autofocus` sur *Annuler*) |
| `routes/design-system/+page.svelte:69` | exemple | — |
| `routes/design-system/+page.svelte:170` | *Toast erreur* (démonstration) | — |

**Classe B — `<Button>` rouge par `class` : 2 sites**, migrés par AC5 (`fiscal-years:603`,
`email-templates:439`). Déjà visibles (`bg-red-600`, texte blanc 4,83:1) : la migration est une affaire de DRY
et de garde, non de visibilité.

**Classe C — `<button>` brut bordé, texte rouge : 6 sites, laissés tels quels, inscrits à la table de l'AC6**
(C-28-1-9). La bordure les fait lire comme des boutons ; trois sont en `text-destructive` et **deviennent
rouges** par AC4 (aujourd'hui gris foncé).

| Site | Classe | Motif à écrire dans la table |
|---|---|---|
| `routes/(app)/settings/api-keys/+page.svelte:377` (`api-keys-revoke-{id}`) | `border border-border … text-red-600` | déclencheur de ligne bordé, ouvre une confirmation |
| `routes/(app)/bank-accounts/+page.svelte:477` (`archive-button-{id}`) | `border border-border … text-red-600` | idem |
| `lib/features/reconciliation/rules/RulesList.svelte:175` (`delete-button-{id}`) | `border border-error … text-error` | idem (`confirm()` natif) |
| `routes/(app)/payment-batches/[id]/+page.svelte:166` (`payment-batch-cancel`) | `border … text-destructive` | action de page bordée |
| `routes/(app)/supplier-invoices/[id]/+page.svelte:410` (`supplier-invoice-settlement-cancel`) | `border … text-destructive` | idem |
| `routes/(app)/supplier-invoices/[id]/+page.svelte:457` (`supplier-invoice-cancel`) | `border … text-destructive` | idem |

**Classe D — `<button>` brut sans bordure, texte rouge : 3 sites, migrés par AC5** — exactement le symptôme de
l'issue (« se lit comme du texte ») : `ContactPersonsManager.svelte:114` et `BankProfileList.svelte:79` passent à
`destructive` ; `bank-import/[id]/+page.svelte:143` (souligné, à côté du lien *Voir l'écriture*, sur l'écran même
de la recette) passe à `outline size="sm"`, la forme du même bouton sur la fiche de la facture (C-28-1-21).

**Classe E — icônes ✕ / X de retrait d'une ligne de saisie : 3 sites, laissés, inscrits à la table** :
`routes/(app)/supplier-invoices/+page.svelte:403` et `routes/(app)/supplier-invoices/import/+page.svelte:564`
(`text-destructive`, ✕) ; `lib/features/journal-entries/JournalEntryForm.svelte:465`
(`text-muted-foreground hover:text-destructive`, icône `X` avec `aria-label`). Motif : retrait d'une ligne non
encore enregistrée, réversible, contrôle iconique.

**Non-boutons, hors table** : `DropdownMenu.Item class="text-error"` de `design-system/+page.svelte:159` (élément
de menu de démonstration) ; les variantes `data-[variant=destructive]` de `dropdown-menu-item.svelte`,
`dropdown-menu-sub-trigger.svelte`, `select-item.svelte` — **aucun usage** dans l'application (les
`DropdownMenu.Item` de `(app)/+layout.svelte:311-332` n'ont pas de variante). `role="button"` : un seul site,
la zone de dépôt de `BankImportUpload.svelte:391`, non destructive. Aucun `<a>` rouge.

**Déclencheurs neutres d'une action destructive — hors périmètre** : `journal-entries/[id]/+page.svelte:327`
(*Supprimer*, `outline`), `settings/vat-rates/+page.svelte:249` (*Désactiver*, `ghost`),
`settings/projects/+page.svelte:274/298` (*Archiver*, `outline`), `DemoBanner.svelte:51` (*Réinitialiser*,
`outline`), `RulesList.svelte:161` (*Désactiver*, bordé neutre), `InvoiceSettlements.svelte:86-91` (*Annuler le
règlement* / *Annuler le solde*, `invoice-settlement-cancel`, `outline sm`) et `:106-111` (*Annuler le
rapprochement*, `invoice-settlement-cancel-reconciliation`, `outline sm` — la forme que prend désormais le bouton
du détail d'import). Ils **se voient** (bordure ou survol) et ouvrent une confirmation : leur sobriété est une
hiérarchie voulue, non le défaut de l'issue.

**Confirmation non destructive d'une annulation — hors périmètre, signalée** : `invoices/[id]/+page.svelte:1666-1674`
(`invoice-settlement-cancel-confirm`, *Annuler le règlement* / *Annuler le solde*) est un `<Button>` `default`
(bleu plein), alors que les deux autres confirmations de la page (`:1440` suppression, `:1496` dévalidation) sont
`destructive`. Il **se voit** : ce n'est pas le défaut de #640 (un bouton qui ne se voit pas), mais une question
d'harmonisation des variantes de confirmation, qui n'est pas tranchée ici (le règlement annulé se contre-passe et
se ressaisit). Signalé à l'orchestrateur, sans demande d'issue.

### Angles morts assumés (à écrire au Dev Agent Record)

1. **D'autres jetons de couleur ne sont pas définis non plus — renvoi à #645** (P3, ouverte). Les neuf noms
   shadcn `muted`, `muted-foreground`, `foreground`, `background`, `ring`, `input`, `accent`,
   `accent-foreground`, `card` (ordre de grandeur : `muted-foreground` seul ≈ 58 occurrences dans 23 fichiers),
   et trois noms relevés par la lentille R de la validation P1 : `error-soft` (3 sites), `warning-soft` (8 sites)
   et `danger` (`AdminRestorePanel.svelte:173`, `AdminBackupPanel.svelte:71`). Effet visible : la variante
   `outline` (*Annuler*) n'a ni fond ni survol (`bg-background`, `hover:bg-muted` sans règle), `ghost` n'a pas de
   survol, `focus-visible:ring-ring` est sans effet (le contour global bleu compense ; la variante destructive
   fixe sa propre bordure et son propre anneau de focus, C-28-1-20). **Hors périmètre** (C-28-1-13, C-28-1-17) :
   *Annuler* reste lisible (bordure `border-border`, jeton défini). La validation P1 a mesuré que les définir
   n'est pas neutre — `input` sert à la fois de bordure (≥ 3:1 requis) et de fond désactivé, `muted-foreground`
   au ton shadcn `#64748b` tombe à 4,34:1 sur `muted` : ces mesures sont à reprendre dans #645, non ici. La garde
   facultative AC7 (4) tient cette liste en table fermée, renvoyée à #645.
2. **Suppression d'une personne de contact sans confirmation — renvoi à #646** (P3, ouverte)
   (`ContactPersonsManager.svelte:76-83` : `removePerson` appelle `deleteContactPerson` directement), contraire
   au principe UX « confirmations avant actions destructives » (spec UX l. 286). La story rend le bouton
   visible, elle n'ajoute pas de confirmation (C-28-1-17).
3. **Sous `prefers-color-scheme: dark`**, des classes `dark:` d'autres composants s'activent déjà
   (`dark:bg-amber-950/30`, `dark:bg-green-900`… émises dans le build) alors que Kesh n'a pas de thème sombre.
   Préexistant, hors périmètre ; la variante destructive n'en porte plus aucune (AC2).
4. **Taille de cible** : `h-8` (32 px) pour toutes les variantes, sous les 44 px du mode Guidé (spec UX l. 718).
   Préexistant, commun à tous les boutons.
5. **Pas de régression au pixel** : les captures ne sont pas comparées (C-28-1-10). Une régression de style hors
   des propriétés mesurées (taille, police, espacement) n'est vue que par l'œil.
6. **La CI n'exécute aucun E2E** (#577) : en CI, seules les gardes Vitest (AC6, AC7) protègent la variante.
7. **Les 24 sites de la classe A ne sont tenus par aucune table** : un `variant="destructive"` retiré de l'un
   d'eux ne fait rougir aucune garde (la table « sites migrés » de l'AC6 ne couvre que les cinq sites de l'AC5) ;
   seuls les trois exercés par l'AC8 — (b) et (d) — le verraient, hors CI.
8. **`contact-person-delete-{id}` n'est vu à l'écran par aucun test** : couvert par transitivité (AC8, fin).
9. **Bordure d'`outline` à 1,23:1** contre le blanc (`--color-border` `#e2e8f0` : L = 0,8018 ; 1,05 / 0,8518) —
   la bordure de tous les *Annuler* de l'application et du bouton *Annuler le rapprochement* de la fiche
   facture, que l'AC5 étend au détail d'import. Le bouton reste identifiable par sa forme et son libellé
   (texte courant `#1e293b`, 14,6:1) ; le contraste de la bordure n'est pas traité ici.

### État actuel des fichiers modifiés — ce qui doit être préservé

- `button.svelte` : `buttonVariants` est **exporté** (`ui/button/index.ts:5,13`), sans autre consommateur dans
  `src/` au 2026-10-13 (`grep -rn buttonVariants frontend/src`) ; seules la ligne `destructive` change. Le rendu `<a>` (avec `href`) et
  `<button>` reste identique.
- `app.css` : ordre du `@theme`, styles globaux, espacement Guidé/Expert et règle `:focus-visible` inchangés.
- Sites de l'AC5 : comportements préservés — la
  `confirm()` native de `BankProfileList.handleDelete` (`:26`), l'ouverture de `CancelReconciliationDialog` par
  `cancelTxId` (`bank-import/[id]/+page.svelte:147`), l'absence de confirmation de `removePerson` (#646, hors
  périmètre), les `disabled={closeSubmitting}` / `{restoreSubmitting}` des dialogues B. Tests qui ciblent ces
  boutons, par `data-testid` seulement (aucun n'assert une classe) : `fiscal-years-page.test.ts:282`,
  `e2e/fiscal-years.spec.ts`, `e2e/email-templates.spec.ts`, `e2e/reconciliation-cancel.spec.ts:139,149`
  (`:139` reçoit l'assertion (e) de l'AC8).
- Le seul test du dépôt qui lit la classe `text-destructive` : `routes/(app)/contacts/contacts-page.test.ts:729`
  (`document.querySelector('p.text-destructive')`) — la classe ne change pas, il reste vrai.
- `bank-import-delete.spec.ts` : son scénario ne change pas, seul son montage passe par le helper extrait.

### Propagation post-patch (T10)

Greper la **valeur** après chaque correctif : `grep -rnE '#b91c1c|#991b1b|6,47|5,45|8,31|2,53|1,23' frontend/src
_bmad-output/implementation-artifacts/28-1-* _bmad-output/planning-artifacts/ux-design-specification.md
CHANGELOG.md` (les ratios cités doivent tous être ceux d'`app.css`) ; `grep -rnE '4,59|\b33 bandeaux'
_bmad-output/implementation-artifacts/28-1-boutons-destructifs.md` → seulement le Change Log et les rectifications
du registre (valeurs retirées en validation P1) ; `grep -rn 'bg-destructive/10 hover' frontend/src` → 0 ;
`grep -rn 'est grisé' docs/manual/fr/*.tex` → seul `user-manual.tex:1107`.

### Découpage préventif — Dérogation règle de splitting (C-28-1-24, remplace C-28-1-15)

**Recompte, sur le périmètre réduit** (validation P1, finding R6 : l'ancien « cinq modules au plus » était faux
sur sa propre liste). Modules de code de premier niveau édités, dans `frontend/src/` :

| # | Module | Ce qui change |
|---|---|---|
| 1 | `lib/components/ui/button` (et `ui/` pour la garde facultative AC7 (4)) | variante (1 ligne) ; gardes AC6, AC7 |
| 2 | `app.css` (thème) | trois jetons |
| 3 | `lib/features/contacts` | `ContactPersonsManager.svelte` : un `<button>` → `Button` |
| 4 | `lib/features/bank-import` | `BankProfileList.svelte` : idem |
| 5 | `routes/(app)/bank-import` | `[id]/+page.svelte` : idem (`outline`) |
| 6 | `routes/(app)/settings` | `fiscal-years`, `email-templates` : `class` → `variant` |
| 7 | `routes/design-system` | trois exemples, un `data-testid`, un carré de palette |

Hors code de production : `tests/e2e` (spec neuve, deux helpers, `bank-import-delete.spec.ts`,
`reconciliation-cancel.spec.ts`), `docs/manual/fr` (deux phrases, deux PDF), la spec UX, le CHANGELOG. Aucun
crate, aucune clé i18n, aucune migration, aucune règle métier.

**Sept modules : le critère « plus de 5 modules » est franchi, et la story n'est pas découpée.** Motifs :

- **Ce que la règle protège n'est pas exposé ici.** Elle vise une story trop large pour tenir dans un seul
  modèle mental adversarial. Les modules 3 à 7 portent chacun une édition **mécanique d'une à trois lignes**
  (`<button class>` → `<Button variant>`), sans logique ; le seul changement qui demande un raisonnement est
  concentré dans les modules 1 et 2.
- **Le découpage qu'elle prescrit existe déjà dans la fiche.** Le patron « story-zéro qui pose le patron, puis
  rollout mécanique » donnerait 28-1 = modules 1, 2, 7 (jetons, variante, gardes, page de démonstration) et
  28-1b = modules 3 à 6 (cinq sites). Mais la garde AC6 est **l'inventaire même** des sites — ses deux tables
  bougent avec chaque migration ; scindée, elle serait écrite deux fois, et l'état intermédiaire (variante
  pleine, `Annuler le rapprochement` encore souligné) serait livré à part sans que l'issue soit close.
- **Le rollout se revoit fichier par fichier**, comme la règle le demande pour une sous-story de rollout : le
  prompt de revue de code nommera les cinq sites de l'AC5 un à un, contre la table « sites migrés ».

**Risque accepté** : une régression dans un des cinq sites qui ne toucherait ni la classe, ni le `variant`, ni le
`data-testid` (par exemple un `onclick` perdu) ne serait vue que par les tests existants de ces écrans
(`fiscal-years-page.test.ts`, `fiscal-years.spec.ts`, `email-templates.spec.ts`, `reconciliation-cancel.spec.ts`,
`bank-import-delete.spec.ts`) — `ContactPersonsManager` et `BankProfileList` n'ont pas de test de clic.
Le signal de découpage est **déclaré au Project Lead** par l'orchestrateur (CLAUDE.md, amendement D5).

### Pièges connus du dépôt, applicables ici

- **E2E** : `KESH_COOKIE_SECURE=false`, `KESH_HOST=127.0.0.1`, `KESH_TEST_MODE=true`,
  `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, `KESH_BACKEND_URL=http://127.0.0.1:3034`,
  `KESH_STATIC_DIR` vers le `frontend/build` **du worktree** (rebuild après chaque changement de CSS : le
  backend sert le build, non les sources), `KESH_ADMIN_BACKUP_DIR` inscriptible, `CARGO_TARGET_DIR` du worktree.
  Attendre `/health` ; tuer le backend par PID.
- **Base** : `kesh_281` / `kesh_e2e_281`, jamais `restart` du conteneur partagé (consigne 2).
- **Édition** du code par l'outil Edit uniquement (consigne 14) ; mutations restaurées par `git checkout --` +
  `touch`.
- **tailwind-merge** (`cn`) : `class="ml-2"` sur un `Button` ne heurte pas la variante ; ne **pas** passer de
  classe de couleur à un `Button variant="destructive"` (la garde AC6 la relèverait).
- **Données réelles** : aucune ; les captures ne montrent que les fixtures synthétiques.

### Fichiers probablement communs avec les autres stories de l'epic

`CHANGELOG.md` (section `[0.13.3] — Non publié`, créée par plusieurs stories), `sprint-status.yaml` (section
`epic-28`, créée aussi par la 28-0), `docs/manual/fr/user-manual.{tex,pdf}` et `admin-manual.{tex,pdf}` (la 28-4
et la 28-5 les réécrivent : **conflit binaire des PDF**, résolu en régénérant après fusion),
`lib/features/bank-import/BankProfileList.svelte` (la 28-2 y traite un `String(e)` — lignes distinctes),
`routes/(app)/bank-import/[id]/+page.svelte` (la 28-5 peut y toucher les rejetées).

### References

- Issue #640 ; epic `epic-28-recette-v0-13-2.md` § 28-1 et § *Recette visuelle (B2)*.
- `_bmad-output/planning-artifacts/ux-design-specification.md` l. 286, 446, 514, 639-662, 714-718.
- Patron de garde : `frontend/src/lib/features/bank-accounts/bank-name-inventory.test.ts`.
- `frontend/src/lib/components/ui/sonner/sonner.svelte:3` (« Kesh n'a pas de dark mode »).
- WCAG 2.2 : 1.4.3 (exemption des composants inactifs), 1.4.11, 2.4.7.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-13 — Fiche créée en autonomie (Opus 5.5), `ready-for-dev`. Prémisse de l'issue corrigée (C-28-1-1 :
  jeton absent du thème, non « fond à 10 % »). Choix C-28-1-1 à C-28-1-16.
- 2026-10-13 — **Validation P1** (Sonnet selon le prompt versionné, lentilles R et F, prompt `28-1-validate-prompt-p1.md` ; rapports hors
  dépôt `kesh-gate-logs/28-1-validate-p1-{R,F}.md`) : **R 0 CRITICAL / 2 HIGH / 8 MEDIUM / 6 LOW ; F 0 / 1 HIGH /
  4 MEDIUM / 4 LOW** — bilans tels que déclarés ; recompté sur les titres, R porte **2 HIGH / 9 MEDIUM / 5 LOW**
  (R3 à R11 sont titrés MEDIUM), même total de 16 ; les 9 sont traités ci-dessous. Les lentilles avaient reçu la consigne d'examiner un élargissement (neuf jetons +
  confirmation) ; l'orchestrateur l'a **annulé** — périmètre réduit à #640 (C-28-1-17). Remédiation :
  - **Fermés par le retrait du périmètre** (C-28-1-17) : F3, F5, R1, R2, R3, R7, R8 — tous portaient sur la
    définition des autres jetons ou sur la confirmation de suppression d'une personne de contact, renvoyées à
    **#645** et **#646** (angles morts 1 et 2 réécrits avec renvoi, C-28-1-13 précisé). R8 (le commentaire posté
    sur #640 annonçait « la Story 28-1 traite l'ensemble ») : la fiche est cohérente avec le périmètre réduit ; le
    commentaire de l'issue est à rectifier par l'orchestrateur.
  - **F1 (HIGH)** : AC7 (1)(2) lisent `buttonVariants.variants.variant.destructive`, non la sortie fusionnée ;
    assertion (1-bis) sur la fusion (C-28-1-18).
  - **F2 = R9 (MEDIUM)** : lecture stabilisée par `toHaveCSS` / `expect.poll` (`transition-all`, 150 ms)
    (C-28-1-19) ; **F8 (LOW)** : `parseRgb` refuse `oklab(`/`color(` en le disant, lit `rgba(0, 0, 0, 0)`.
  - **R11 (MEDIUM)** : `focus-visible:border-destructive` ajouté à la variante — la bordure de focus ne dépend
    plus du jeton absent `ring` ; AC3, AC7 (1-bis), AC8 (a) et M19 (C-28-1-20).
  - **F4 (MEDIUM), pour la part qui touche la variante** : aucune règle `dark:` dans la variante (déjà AC2), les
    `dark:aria-invalid` de la base expliqués (11 `aria-invalid`, tous des champs) ; le passage sombre de l'AC8
    n'exige rien d'autre que la variante ; le reste relève de #645.
  - **R5 (MEDIUM)** : « dix specs axe » → 19 fichiers à `.analyze(`, **16** exerçant `color-contrast` (nommés à
    l'AC4 ; la lentille R en comptait 15 : `invoices` y garde deux analyses sur trois), 3 qui la neutralisent
    (C-28-1-25).
  - **R6 (MEDIUM)** : modules recomptés sur le périmètre réduit — **sept** ; dérogation motivée écrite
    (C-28-1-24, remplace C-28-1-15).
  - **R10 (MEDIUM) et F9 (LOW)** : clé de table AC6 non ambiguë (`chemin#data-testid` ou `chemin@élément:rang`),
    seconde table « sites migrés » (variant et `class` littéraux), M18, assertion (e) dans
    `reconciliation-cancel.spec.ts`, transitivité assumée pour `contact-person-delete` (C-28-1-22).
  - **R14 (LOW), tranché** : *Annuler le rapprochement* du détail d'import passe à `outline size="sm"`, la forme
    du même bouton sur la fiche de la facture que le manuel promet identique ; la confirmation
    `invoice-settlement-cancel-confirm` (`default`) est triée hors périmètre et signalée (C-28-1-21).
  - **R4 (MEDIUM)** : garde d'ensemble clos facultative, en table fermée renvoyée à #645, décidée par le
    compilateur Tailwind (AC7 (4), M20, C-28-1-23).
  - **LOW** : R12 (`/20` 4,59 → 4,57, retiré : plus aucun usage), R13 (33 → 32 bandeaux), F7 (5,45 exact sur le
    mélange non arrondi, 5,46 arrondi : écrit à l'AC1), R15 (`CHANGELOG.md:19,27` et `user-manual.tex:2010` nommés
    à l'AC10), R16 (texte du CHANGELOG : niveaux de relance, réinitialisation de la démonstration, déverrouillage
    des livres), F6 (`ContactPersonsManager` **est** dans un `<form>` — énoncé corrigé, sans conséquence). Décomptes
    rectifiés au registre (C-28-1-25).
  Choix C-28-1-17 à C-28-1-25. Mutations M1-M20. Validation P2 : `28-1-validate-prompt-p2.md`.
