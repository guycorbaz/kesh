# Story 28.1 : Les boutons d'action destructive se voient

## Status

**done** — revue de code close le 2026-10-13 : P1 (Sonnet, B/E/A) remédiée, passe ciblée P2 (Haiku, 3 LOW)
remédiée en tests seuls (`f3fadb5a`). AVANT : **review** — développement fait le 2026-10-13 (`bmad-dev-story`, Opus 5.5), gates au dernier commit ; revue de code
P1 : prompt `28-1-review-prompt-p1.md`. Validation close après la passe ciblée P4. Fiche créée le 2026-10-13 en autonomie (`bmad-create-story`), sur `6f971d76`
(rétrospective de l'Epic 27), branche `story/28-1`, worktree `/home/gcorbaz/devel/kesh-28-1`, rebasée en
validation P2 sur `5aad8b35` (kickoff de l'Epic 28, #643). Choix consignés au registre
`epic-28-choix-autonomes.md` (**C-28-1-1 à C-28-1-44** — recompté le 2026-10-13 à la remédiation de la revue de
code P1, `grep -c '^### C-28-1-'` ; le registre séparé de la story y a été versé par union en validation P2). 13 AC
(AC1-AC13), tâches T0-T9, mutations M1-M22 (M18 retirée), plus P1-M1 à P1-M11 de la remédiation de la revue de
code P1. Revue de code P1 (Sonnet, lentilles B, E, A) remédiée ; passe ciblée P2 : prompt
`28-1-review-prompt-p2-ciblee.md`. Validations P1 (Sonnet),
P2 (Opus) et P3 (Sonnet), lentilles R et F, remédiées — **périmètre réduit à #640** en P1 (C-28-1-17), puis **au
strict nécessaire pour fermer #640** en P2 (C-28-1-26) ; passe ciblée P4 (Haiku, prompt
`28-1-validate-prompt-p4-ciblee.md`), remédiée — validation close. PR : **`closes #640`** (et `refs #645 #646 #648`, angles morts renvoyés).

## Story

**As a** utilisateur de Kesh (le Comptable comme l'Administrateur),
**I want** qu'un bouton qui supprime, archive, révoque, désactive ou annule se reconnaisse au premier coup d'œil
comme un bouton,
**so that** je ne passe plus à côté d'une fonction — comme *Supprimer l'import*, que je n'ai pas vu à la recette
de la v0.13.2 — et que je distingue d'un regard l'action irréversible de son *Annuler*.

Issue : [#640](https://github.com/guycorbaz/kesh/issues/640) (`bug`, **P3**, recette de la v0.13.2 du
2026-10-13). Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` (sur `main` depuis
`5aad8b35`, #643). Cible : **v0.13.3**.

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
     clair pour un texte sur bandeau bg-destructive/10 (4,14:1 < 4,5). Ratios WCAG, calculés
     comme le helper contrast.ts (mélanges non arrondis, arrondi au centième) : blanc sur
     destructive 6,47:1 ; blanc sur destructive-hover 8,31:1 ; destructive sur blanc 6,47:1 ;
     destructive sur bandeau /10 5,46:1. */
  --color-destructive: #b91c1c;
  --color-destructive-foreground: #ffffff;
  --color-destructive-hover: #991b1b;
```

**Une seule méthode de calcul, celle du helper (C-28-1-27).** Tout ratio de la fiche, du registre, d'`app.css`,
de la spec UX et du CHANGELOG est calculé comme le fait `blend` + `contrastRatio` du helper de l'AC7 (3) : un
mélange à alpha est pris **non arrondi** (le bandeau `/10` vaut `(248 ; 232,3 ; 232,3)`), puis le ratio est
**arrondi au centième** (non tronqué). Sur le pixel 8 bits `#f8e8e8`, le même bandeau donnerait 5,45 : ce n'est
pas la valeur citée. Le `/20` n'est plus cité : plus aucun **fond sous un texte** ne l'emploie (l'ancienne
variante, et le `dark:` inutilisé de `dropdown-menu-item.svelte:23`) ; les `aria-invalid:ring-destructive/20` de
`input.svelte:28,41`, `select-trigger.svelte:22` et de la base de `button.svelte:7` restent actifs, mais ce sont
des anneaux, sans texte dessus.

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
| désactivé | `opacity: 0.5`, fond **toujours** `rgb(185, 28, 28)` (l'opacité s'applique à l'élément) | 2,54:1 effectif (mélange non arrondi `(220 ; 141,5 ; 141,5)`) | **exempté** (1.4.3 : composant inactif) — distinct du repos (C-28-1-5) |
| repos, `prefers-color-scheme: dark` émulé | **identique** au repos clair | — | C-28-1-7 |

L'anneau `focus-visible:ring-destructive/40` (3 px, rouge à 40 %) n'est **pas** asserté : sa couleur passe par
`color-mix(in oklab, …)`, que `getComputedStyle` rend en `oklab(…)`/`color(…)` et non en `rgb(…)` ; il est
vu sur la capture **de l'état focalisé** (`a-focus-<clair|sombre>.png`, AC8, C-28-1-29), cadrée 8 px **au-delà**
de la boîte de bordure — l'anneau (3 px) et le contour (2 px décalés de 2 px) en sont hors, et
`locator.screenshot()` les rognerait (C-28-1-33) — œil seul, angle mort 10. Toutes les valeurs du tableau se lisent **après stabilisation** : la base porte
`transition-all` (150 ms par défaut, `tailwindcss/theme.css` `--default-transition-duration`), et une lecture
immédiate après `hover()` ou `focus` rendrait une couleur intermédiaire (AC8, C-28-1-19).

La règle globale `:focus-visible` est hors couche (`app.css:62`) et l'emporte sur `outline-none`, qui vit dans
`@layer utilities` : **à vérifier par l'E2E de l'AC8, non à supposer**. Si le style calculé dément cette
lecture, corriger (par exemple `focus-visible:outline-2 focus-visible:outline-primary-light` dans la variante)
et l'écrire au registre.

**AC4 — Le jeton défini rend leur rouge aux autres usages du jeton, sans édition de site (C-28-1-8, C-28-1-36).**

Définir `--color-destructive` active d'un coup tous les usages du jeton qui ne sont pas la variante (sur les
201 occurrences dans 42 fichiers, 29 sont dans 4 fichiers de `components/ui/`, dont la variante elle-même) : les 32
bandeaux d'erreur, les textes d'erreur `text-destructive` (`<p>`, `<span>`, `<td>`, `<li>`), les bordures de
champ en erreur (`border-destructive` sur `Input`, `aria-invalid:border-destructive` de `input.svelte`,
`select-trigger.svelte` — 11 sites `aria-invalid` hors `ui/`), la section `border-destructive` de
`settings/vat-rates/+page.svelte:356`, et les **six** `<button>` bruts en `text-destructive` (C-28-1-36) — les
trois bordés de la classe C (*Annuler le lot*, *Annuler le règlement*, *Annuler la facture* fournisseur :
`payment-batches/[id]:166`, `supplier-invoices/[id]:410`, `:457` ; leur `border` sans couleur suit `currentcolor`
et devient rouge avec le texte), *Annuler le rapprochement* du détail d'import (classe D, `bank-import/[id]:143`),
qui devient un texte **rouge** souligné à côté du lien *Voir l'écriture*, et les deux icônes ✕ de retrait de ligne
(classe E, `supplier-invoices:403`, `supplier-invoices/import:564`) ; la troisième icône de la classe E
(`JournalEntryForm:465`) ne devient rouge qu'au survol (`hover:text-destructive`) (Dev Notes § Inventaire).
**Aucun de ces sites n'est édité** pour l'AC4 — c'était leur intention d'origine.

- Contraste garanti par AC1 : texte sur blanc 6,47:1, texte sur bandeau `/10` 5,46:1.
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

**AC5 — Aucun site applicatif n'est édité (C-28-1-26).**

La story ne touche, dans `frontend/src/`, que trois modules : `app.css` (AC1), `lib/components/ui/` (variante,
AC2 ; gardes, AC6, AC7) et `routes/design-system` (AC9). `git diff --stat main -- frontend/src` au dernier
commit de code ne nomme aucun autre fichier (sortie au Dev Agent Record). En particulier, les cinq boutons rouges
stylés à la main — classe B (`fiscal-years:603`, `email-templates:439`) et classe D (`ContactPersonsManager:114`,
`BankProfileList:79`, `bank-import/[id]:143`) — **restent tels quels** et sont inscrits à la table de l'AC6 ;
leur forme est un angle mort (Dev Notes, angle mort 11), non un livrable de la 28-1. Le retrait est écrit à
C-28-1-26 : la migration de ces cinq sites, et le passage d'*Annuler le rapprochement* en `outline` (C-28-1-21,
**annulé** — la variante `outline` le rendait moins visible qu'aujourd'hui : bordure à 1,23:1, ni fond ni survol
tant que #645 n'est pas fait), étaient nés de la remédiation P1 et non de #640, dont le symptôme est
*Supprimer l'import* (classe A, corrigé par l'AC2 sans édition).

**AC6 — Inventaire clos des boutons rouges posés à la main : une garde Vitest (C-28-1-11, C-28-1-22,
C-28-1-28).**

Fichier neuf `frontend/src/lib/components/ui/button/boutons-rouges-inventory.test.ts`, sur le patron de la
**table fermée** de `frontend/src/lib/features/bank-accounts/bank-name-inventory.test.ts` (Story 26-2b) — non de
son analyse : aucune garde du dépôt n'emploie encore `svelte/compiler`, celle-ci sera la première :

- parcourt **tous** les `.svelte` de `src/` (hors `.test.`), les analyse par `parse(source, { modern: true })` de
  `svelte/compiler` (pas d'expression régulière sur des balises : un `onclick={() => …}` contient un `>`) ;
- pour chaque élément `button` (`RegularElement`) et chaque composant `Button` (`Component`), lit le **texte
  source** de ses attributs `class` (littéral **ou** expression) et de ses directives `class:` ;
- relève tout jeton `(text|bg|border|ring|outline)-(destructive|error|red-\d{2,3})(/\d+)?`, préfixes de variante
  compris (`hover:text-destructive`) ;
- exige que l'ensemble `{ clé → jetons }` **égale** une table fermée, chaque entrée avec son motif. La **clé**
  est non ambiguë (C-28-1-22) : `<chemin relatif à src/>#<data-testid>` quand l'élément porte un `data-testid`,
  sinon `<chemin>@<élément>:<rang>`, le rang comptant à partir de 1 les éléments **sans** `data-testid` porteurs
  d'un jeton rouge du même fichier, dans l'ordre du source. Le `data-testid` s'écrit par son **texte source**
  (C-28-1-28) : `source.slice(start, end)` de la **valeur** de l'attribut — pour une valeur entre guillemets
  (tableau `Text`/`ExpressionTag`), du début du premier nœud à la fin du dernier, guillemets exclus
  (`api-keys-revoke-{k.id}`, `settings/api-keys/+page.svelte:379`) ; pour une valeur `ExpressionTag` seule,
  accolades comprises (``{`contact-person-delete-${p.id}`}``, `ContactPersonsManager.svelte:116`). Aucune
  normalisation des gabarits. Pas de numéro de ligne (une édition sans rapport ferait rougir la garde). Les deux
  `<button>` de `supplier-invoices/[id]/+page.svelte` (`:410`, `:457`) ont deux `data-testid` distincts ; les
  trois icônes ✕, sans `data-testid`, sont chacune seule dans son fichier (`@button:1`). La garde **échoue** si
  deux éléments produisent la même clé. La table contient **exactement quatorze entrées** (Dev Notes §
  Inventaire) : les deux `Button` rouges par `class` de la classe B, les six déclencheurs bordés de la classe C,
  les trois `<button>` sans bordure de la classe D et les trois icônes ✕ de la classe E. Une entrée en trop **ou**
  en moins, ou des jetons changés, fait rougir (M11-M14). Clés attendues des classes B et D :
  `routes/(app)/settings/fiscal-years/+page.svelte#fiscal-year-close-confirm`,
  `routes/(app)/settings/email-templates/+page.svelte#email-template-restore-confirm`,
  ``lib/features/contacts/ContactPersonsManager.svelte#{`contact-person-delete-${p.id}`}``,
  `lib/features/bank-import/BankProfileList.svelte#bank-import-profile-delete-button`,
  `routes/(app)/bank-import/[id]/+page.svelte#detail-tx-cancel-reconciliation` ;
- **(f) — aucune `class` sur la variante destructive** (C-28-1-28) : tout composant `Button` dont l'attribut
  `variant` est le littéral `destructive` ne porte **aucun** attribut `class` (24 usages au 2026-10-13, aucun avec
  `class`). C'est ce qui rend vraie *par construction* la phrase « les 24 usages de la variante rendent le style de
  l'AC2 » (M21) ;
- **auto-tests sur source synthétique** (R13) : la fonction de relevé est exportée du fichier de test (ou d'un
  module voisin `boutons-rouges-releve.ts`) et appliquée à trois sources écrites dans le test : un `<button
  class:text-destructive={x}>` → jeton relevé (tue M17) ; deux `<button class="text-error">` sans `data-testid`
  → clés `@button:1` et `@button:2` ; deux éléments au même `data-testid` → la garde lève ;
- doc-comment : ce qu'elle voit, ce qu'elle **ne voit pas** (rouge passé par une variable de script, par `cn(…)`
  dans le `<script>`, par un `style=`, ou porté par un `<a>`/`<div role="button">` — aucun aujourd'hui, vérifié),
  et comment ajouter une entrée légitime (« avec son motif : c'est le but — le changement se voit en revue »).

La variante `destructive` du `Button` est la **seule** source du rouge d'un bouton hors table. La table ne
contrôle pas `variant=`, qui n'est pas une classe : un `variant="destructive"` retiré d'un des 24 usages de la
classe A ne rougit aucune garde (angle mort 7) ; une `class` ajoutée à l'un d'eux rougit (f).

**AC7 — Des gardes Vitest qui rougissent si la variante redevient pâle (C-28-1-11, C-28-1-12, C-28-1-18, C-28-1-23,
C-28-1-30, C-28-1-34, C-28-1-35).**

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
3. Contrastes recalculés **depuis les valeurs lues** dans `app.css`, par le helper
   `frontend/tests/e2e/helpers/contrast.ts` (§ *Helper de contraste* ci-dessous) :
   `destructive-foreground`/`destructive` ≥ 4,5 ; `destructive-foreground`/`destructive-hover` ≥ 4,5 ;
   `destructive`/`#ffffff` ≥ 4,5 (texte d'erreur) ; `destructive`/mélange 10 % sur blanc ≥ 4,5 (bandeau) ;
   `destructive`/`#ffffff` ≥ 3 (limite du bouton, 1.4.11). C'est le seul contrôle des contrastes de la story ;
   l'E2E ne recalcule aucun ratio (C-28-1-31).
4. **Garde d'ensemble clos des classes de couleur sans jeton défini, en table fermée (C-28-1-23, C-28-1-30).**
   Fichier `frontend/src/lib/components/ui/jetons-non-definis.test.ts` (`// @vitest-environment node`).
   - **Relevé** : dans tous les `.svelte` et `.ts` de `src/` (hors `.test.`), le texte est découpé en jetons sur
     les séparateurs `` [\s"'`{}()=] `` — le `=` compris (C-28-1-34) : sans lui, la directive
     `class:bg-accent={…}` livrerait le jeton `bg-accent=`, que l'ancre `$` rejette, et un nom porté par une
     directive seule échapperait à la garde ; un jeton est retenu s'il s'écrit, une fois retirés ses préfixes de
     variante (tout ce qui précède le dernier `:`, donc aussi le `class:` d'une directive), un `!` initial et une
     opacité finale `/<n>`,
     `^(bg|text|border|ring|outline|fill|stroke|divide|placeholder|decoration|caret|accent)-[a-z][a-z0-9-]*$`.
     La frontière est le **début du jeton** : `context-menu` ne livre pas `text-menu`. Le dépôt compte, au
     2026-10-13, **16** directives `class:` à préfixe de couleur (`grep -rnE 'class:[a-z-]*(bg|text|border|ring|outline)-'
     src --include=*.svelte | grep -v '\.test\.' | wc -l`), dont **15** de couleur — `class:text-right` de
     `GeneralLedgerView.svelte:173` est un alignement, que Tailwind résout (le repli doit l'écarter de même) ;
     aucun des treize noms de la table n'y est porté **seulement** par une directive (`bg-accent` figure aussi en
     `class="…"`), et le `=` ajouté ne fait naître **aucun** jeton neuf hors directives (jetons de la forme
     `<x>=<préfixe>-<nom>`, préfixes de la regex ci-dessus, cherchés par `grep -rhoE` dans les `.svelte` et `.ts`
     de `src/`, `class:` exclu → aucune sortie au 2026-10-13).
   - **Auto-tests sur source synthétique** (C-28-1-34) : la fonction de relevé est exportée (du test ou d'un module
     voisin) et appliquée à trois sources écrites dans le test : `<span class:text-inexistant={x}>` → nom
     `inexistant` relevé (tue M22) ; `<div class="context-menu">` → aucun nom ; `<p class="hover:!bg-foo/40">` →
     nom `foo`.
   - **Décision par Tailwind lui-même** : `__unstable__loadDesignSystem(css, { base })` de `@tailwindcss/node`
     (exporté, `dist/index.d.ts:256` ; il fournit lui-même `loadModule`/`loadStylesheet`, si bien que les
     `@import` d'`app.css` se résolvent depuis `base` = `frontend/src`), puis `candidatesToCss(candidats)`, qui
     rend `null` pour un candidat non résolu. Pas `compile(...).build(...)` : son `build` est cumulatif et rend
     la feuille entière, il ne décide pas candidat par candidat. **`@tailwindcss/node` est déclaré** en
     `devDependencies` de `frontend/package.json` **avec le même spécificateur que `tailwindcss` et
     `@tailwindcss/vite`** (`^4.2.2` au 2026-10-13, C-28-1-35) — il n'est aujourd'hui qu'une dépendance
     transitive de `@tailwindcss/vite`, qu'un test ne doit pas supposer. Un épinglage exact divergerait au premier
     `tailwindcss` 4.3 : `@tailwindcss/node` 4.2.2 dépend de `tailwindcss` **exactement** 4.2.2
     (`node_modules/@tailwindcss/node/package.json`), et embarquerait sa propre copie — la garde déciderait alors
     avec un moteur autre que celui du build, sans signal. D'où une **assertion de version** en tête du fichier :
     le champ `version` de `node_modules/@tailwindcss/node/package.json` (lu par `readFileSync`, le paquet
     n'exporte pas son `package.json`) **égale** celui de `node_modules/tailwindcss/package.json`, et
     `node_modules/@tailwindcss/node/node_modules/tailwindcss` **n'existe pas** ; sinon la garde échoue en disant
     « monter `tailwindcss`, `@tailwindcss/vite` et `@tailwindcss/node` ensemble ».
   - **Repli**, si l'API ne se charge pas sous Vitest (écrit au Debug Log avec l'erreur) : un nom est « défini »
     s'il figure parmi les `--color-<nom>` du `@theme` d'`app.css` **ou** de `node_modules/tailwindcss/theme.css`
     (palette), ou parmi les mots-clés `transparent`, `current`, `inherit`, `black`, `white` ; les candidats dont
     le nom n'est pas une couleur (`text-sm`, `border-b`…) sont écartés par la même règle que Tailwind (le
     candidat est résolu par une autre utilité) — cette dernière règle étant elle-même une liste, le repli est
     moins sûr que l'API, et le Debug Log le dit.
   - **Table** : ce n'est **pas** une liste de formes connues, c'est l'inventaire des non-résolus (CLAUDE.md,
     « inventorier les sites NON RÉSOLUS »). L'ensemble des **noms** non résolus doit **égaler** une table
     fermée de **treize** noms, chacun renvoyé à **#645** : `muted`, `muted-foreground`, `foreground`,
     `background`, `ring`, `input`, `accent`, `accent-foreground`, `card`, `error-soft`, `warning-soft`, `danger`,
     `surface-hover` (relevé indépendant des lentilles P1 et P2 ; `surface-hover` : `credit-notes:59`,
     `payment-batches:229`, `supplier-invoices:452`). La garde rougit (a) si un nom **neuf** non défini
     apparaît, (b) si un nom listé devient défini sans que la table soit mise à jour (#645 la videra). Tout écart
     entre la sortie du compilateur et cette liste est **examiné nom par nom** et écrit au Debug Log avant
     d'être recopié dans la table — la table ne se remplit pas depuis la sortie de la garde qu'elle contrôle (le
     défaut de l'AC7 de l'Epic 23, vert par construction).

**AC8 — Recette visuelle : une spec Playwright qui mesure le style calculé et laisse des captures, centrée sur
*Supprimer l'import* (C-28-1-10, C-28-1-19, C-28-1-26, C-28-1-29, C-28-1-31, C-28-1-32, C-28-1-33).**

Fichier neuf `frontend/tests/e2e/destructive-buttons.spec.ts`. Deux scénarios, chacun écrit comme **deux tests
distincts**, un par schéma de couleurs (`for (const scheme of ['light', 'dark'] as const) test(…)`, avec
`page.emulateMedia({ colorScheme: scheme })` en tête) : un échec en clair n'empêche pas la passe sombre de tourner
et de laisser ses captures. Le **titre porte le schéma** — `` test(`a — états du bouton destructif (${scheme})`, …) ``
et `` test(`b — Supprimer l'import et son dialogue (${scheme})`, …) `` — : Playwright refuse deux tests de même
titre dans un fichier (C-28-1-36).

**Délais (C-28-1-32).** `playwright.config.ts` ne fixe ni `timeout` ni `expect.timeout` : défauts de Playwright,
30 s par test (`beforeEach` compris) et 5 s par assertion. En phase rouge (T0), cinq assertions de (a) expirent
(≈ 25 s) et trois de (b) (≈ 15 s, plus le montage) : le test serait interrompu avant ses derniers steps, et leurs
captures ne partiraient pas. La spec fixe donc **en tête de fichier** `test.describe.configure({ timeout: 120_000 })`
et fait passer **toutes** les assertions de valeur par
`const softExpect = expect.configure({ soft: true, timeout: 2_000 })` — 2 s couvrent largement la transition de
150 ms et bornent la phase rouge à ≈ 10 s pour (a), ≈ 6 s pour (b). Les assertions de **montage**
(`toBeVisible` d'un écran, `toHaveCount(1)` du sélecteur structurel) gardent le délai par défaut et ne sont pas
`soft` : elles ne doivent pas rougir en phase rouge. En T0, un test terminé en **timeout** n'est **pas** le rouge
attendu : il se documente au Debug Log comme toute autre exception. Le passage sombre ne porte d'exigence que sur la **variante destructive** (identique
au clair, C-28-1-7) — les `dark:` d'autres composants (`outline` : `dark:bg-input/30`, `dark:border-input`, inertes
tant que `input` n'est pas défini, #645) ne sont pas assertés, sinon par « fonds différents » en (b).

**Lecture stabilisée, obligatoire (C-28-1-19).** La base porte `transition-all` (150 ms) : une valeur lue par
`evaluate(el => getComputedStyle(el)…)` juste après `hover()` ou `focus` est une couleur **intermédiaire**. Toute
assertion de valeur passe par `softExpect(locator).toHaveCSS('<propriété>', '<valeur>')` — `expect` configuré
`soft`, délai 2 s, ci-dessus — (réessai intégré).
L'E2E **ne recalcule aucun contraste** (C-28-1-31) : la valeur stabilisée y est, par l'assertion même, égale à la
valeur attendue, et un ratio calculé dessus serait une constante ; les contrastes sont contrôlés par l'AC7 (3),
depuis `app.css`, et dans les dialogues par axe.

**États, assertions et captures (C-28-1-29).** Chaque état est un `test.step` ; ses assertions de valeur sont
`softExpect`, soit `expect.soft` à 2 s (elles rendent le test rouge sans l'arrêter) ; sa capture est prise **dans le `finally`** du step,
donc aussi quand une action du step lève (clic ou survol qui expire). La capture vient **après** les assertions
de l'état — c'est-à-dire après stabilisation, ou après expiration du délai en phase rouge (T0) — sous le nom
`<scénario>-<état>-<clair|sombre>.png`, dans `process.env.KESH_E2E_CAPTURES_DIR` s'il est défini, sinon
`testInfo.outputPath()`. **Cadrage (C-28-1-33)** : `locator.screenshot()` rogne à la **boîte de bordure** de
l'élément (`playwright-core/lib/server/screenshotter.js`, `screenshotElement` : clip = `boundingBox()`) — l'anneau
`ring-3` et le contour `:focus-visible` (2 px, décalé de 2 px, `app.css:62-65`) en sont **hors**. Les quatre
captures de (a) et `b-bouton` passent donc par un helper de la spec : `scrollIntoViewIfNeeded()`, puis
`page.screenshot({ clip })` sur la `boundingBox()` du bouton élargie de **8 px** de chaque côté (bornée à 0 en
haut et à gauche) ; `b-dialogue` reste un `locator.screenshot()` du dialogue entier (aucun effet hors de sa boîte
n'y est attendu). Douze captures par
exécution complète : `a-repos`, `a-survol`, `a-focus`, `a-desactive`, `b-bouton`, `b-dialogue`, × clair/sombre.
Les captures **ne sont pas versionnées** et ne servent **pas** de référence de comparaison (pas de
`toHaveScreenshot`, C-28-1-10) : elles sont la pièce de la recette humaine, jointe au prompt de revue et à la PR
par l'orchestrateur. Données **synthétiques** (fixture `csv_e2e_delete_import.csv` du dépôt).

Scénarios :

- **(a) `/design-system`** (route publique, hors `(app)`) — section *Boutons* : le bouton destructif
  (`ds-destructive`) au **repos**, **survolé** (`hover()`), **focalisé au clavier** — le pointeur d'abord
  **éloigné** du bouton (`page.mouse.move(0, 0)`, C-28-1-33), sans quoi le survol resterait actif et `a-focus`
  montrerait le fond de survol — (`getByTestId('ds-destructive').focus()`, puis `keyboard.press('Shift+Tab')` et `keyboard.press('Tab')` pour
  que le focus vienne du clavier ; `evaluate(el => el.matches(':focus-visible'))` doit rendre `true` avant de lire
  le contour), et l'exemple **désactivé** (`ds-destructive-disabled`, AC9) : valeurs du tableau de l'AC3 — dont,
  au focus, `outline-color` **et** `border-top-color` (`rgb(185, 28, 28)`, C-28-1-20) ; au focus encore, le fond
  est asserté **au repos** (`rgb(185, 28, 28)`), ce qui prouve que le pointeur a bien quitté le bouton. Une capture
  par état, cadrée au-delà de la boîte de bordure ; celle du focus montre l'anneau et le contour, que l'œil seul
  juge pour l'anneau (angle mort 10).
- **(b) détail d'un import — *Supprimer l'import*, le symptôme de la recette.** Montage **par test** :
  `test.beforeEach` appelle `seedTestState('with-company')`, puis le test se connecte, monte compte et profil et
  importe le fichier **une fois** par l'écran (helper extrait, ci-dessous), ouvre le détail. Chaque passe part donc
  d'une base remise à zéro : aucun second import du même fichier, donc jamais l'avertissement de doublon qui
  désactive *Confirmer* (`BankImportUpload.svelte:345`, C-28-1-29, ligne rectifiée en validations P3 et P4). Assertions : *Supprimer l'import*
  (`bank-import-delete-button`) a `background-color: rgb(185, 28, 28)` et `color: rgb(255, 255, 255)` ;
  capture `b-bouton`. Ouverture du dialogue (`bank-import-delete-dialog`) : *Supprimer définitivement*
  (`bank-import-delete-submit`) a le même fond ; *Annuler* — sans `data-testid` (`DeleteBankImportAction.svelte:135`),
  visé par le sélecteur **structurel**
  `getByTestId('bank-import-delete-dialog').locator('[data-slot="dialog-footer"] [data-slot="button"]:not([data-testid])')`,
  dont le test exige d'abord `toHaveCount(1)` — n'a **pas** ce fond (`not.toHaveCSS('background-color',
  'rgb(185, 28, 28)')`) ; `new AxeBuilder({ page }).include('[data-testid="bank-import-delete-dialog"]')
  .withRules(['color-contrast']).analyze()` → 0 violation ; capture `b-dialogue`. Le dialogue est **refermé par
  *Annuler*** et `bank-import-delete-button` reste visible (rien n'est supprimé).

Le dialogue de (b) est le **dialogue de confirmation destructif représentatif** de la story : *Supprimer
définitivement* à côté d'un *Annuler* `outline`, la configuration de 13 des 24 usages de la variante (dix-sept
sont une confirmation, à côté d'un *Annuler* `outline` pour treize et `ghost` pour quatre ; douze de ces
confirmations sont dans un `Dialog.Root`, cinq hors — dont l'overlay `role="dialog"` maison de
`journal-entries/[id]` — Dev Notes § Classe A, recompté en validations P3 et P4, C-28-1-36). Les 17 fichiers et 24 usages de la classe A ne sont **pas** montrés un à un à l'écran : ils sont
vérifiés **par construction** — l'AC7 (1)-(2) garde la variante, seule source de leur style, et l'AC6 (f) garde
qu'aucun ne lui passe de `class` qui la contredirait ; l'E2E en montre un échantillon, (a) et (b)
(C-28-1-28) — trois usages sur 24 (`design-system:69`, `DeleteBankImportAction:73` et `:139`). Angle mort
assumé : un style propre au conteneur d'un des 21 autres usages (angle mort 7).

⚠️ **Aucun sélecteur sur un libellé** : la garde `src/lib/shared/e2e-selecteurs-traduits.test.ts` (KF-043, #326)
l'interdit — `data-testid`, ou sélecteur structurel par `data-slot`, seulement.

Le montage d'import de (b) — compte bancaire, profil `E2E suppression`, import par l'écran — est aujourd'hui
écrit dans `tests/e2e/bank-import-delete.spec.ts:35-85` (`login`, `mount`, `importByScreen`) : il est **extrait**
dans `tests/e2e/helpers/bank-import-fixtures.ts` et les deux specs l'importent (C-28-1-16 — une spec n'importe pas
une autre spec : Playwright y enregistrerait ses tests deux fois). Le scénario de `bank-import-delete.spec.ts`
ne change pas.

**Helper de contraste** (pour l'AC7 (3), C-28-1-31) : `frontend/tests/e2e/helpers/contrast.ts` — fonctions pures
`parseHex` (`#rrggbb` seulement ; toute autre chaîne **lève** une erreur qui la nomme), `relativeLuminance`,
`contrastRatio` et `blend(couleur, fond, alpha)` (mélange **non arrondi**, C-28-1-27) ; tests unitaires
`frontend/tests/e2e/helpers/contrast.test.ts` — Vitest les ramasse par `tests/**/*.test.ts`, Playwright les ignore
par `testMatch`, précédent `helpers/test-state.test.ts`. Les tests couvrent : le refus d'une chaîne `rgb(…)` et
d'un `#fff` court ; `blend('#b91c1c', '#ffffff', 0.1)` = `(248 ; 232,3 ; 232,3)` à 0,01 près ; et cinq ratios
connus, **dans les deux ordres** — `#ffffff`/`#000000` 21 **et** `#000000`/`#ffffff` 21, `#ffffff`/`#ffffff` 1,
`#b91c1c`/`#ffffff` 6,47 (la couleur foncée en premier) et le bandeau `/10` 5,46, à 0,01 près. Le premier cas
tue M15 (sans `+ 0,05`, 1/0) ; les cas « foncé en premier » tuent M16 (luminances non triées), que des appels
« clair en premier » laisseraient vivre.

**AC9 — La page `/design-system` montre la variante dans tous ses états (C-28-1-10).**

`frontend/src/routes/design-system/+page.svelte:66-73` (section *Boutons*) : ajouter **un**
exemple, `<Button variant="destructive" disabled data-testid="ds-destructive-disabled">Destructive
désactivé</Button>`, et poser `data-testid="ds-destructive"` sur l'exemple existant (`:69`). (L'exemple `size="xs"`
des passes précédentes est retiré avec la migration des sites `xs`, C-28-1-26.) Le pavé de palette (`:39-47`) gagne un carré
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
une infobulle », envoi d'e-mail, variante `default`) n'est **pas** touché, ni `admin-manual.tex:803` et `:1172`
(« est simplement grisé », bouton d'envoi, variante `default`). Les deux PDF sont régénérés (`make fr` dans
`docs/manual/`, sous `flock`) et contrôlés aplatis — par `grep -o … | wc -l`, qui compte, et non `grep -c`, qui
sur un texte aplati en une ligne ne rend que 0 ou 1 :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "le bouton est désactivé, avec le nombre" | wc -l`
→ 1 ; même contrôle, `admin-manual.pdf`, « le bouton est désactivé avec leur nombre » → 1 ; et, sur les deux,
« le bouton est grisé » → 0. Le CHANGELOG de la 0.13.2 écrit « grisé » deux fois pour ce
même bouton (`CHANGELOG.md:19` et `:27`) : historique, non touché. `user-manual.tex:1739` coupe la phrase après
« avec le » (« nombre » à la ligne suivante) : la substitution ne vise que le mot `grisé`. `user-manual.tex:2010`
(« grisée », liste de règles, hors bouton) n'est pas touché non plus.

**AC11 — Spec UX : la palette nomme le rouge d'action (C-28-1-14).**

`_bmad-output/planning-artifacts/ux-design-specification.md` § *Système de Couleurs* : une ligne
`| **Action destructive** | Rouge foncé | `#b91c1c` | Boutons qui suppriment, archivent, révoquent ; survol `#991b1b` |`
après la ligne *Erreur*, et sous *Ratios de contraste WCAG AA* : « Bouton destructif (blanc sur rouge foncé) :
6.47:1 (AA) ; texte destructif sur bandeau d'erreur : 5.46:1 (AA) » — avec le **point** décimal de la spec
(`ux-design-specification.md:660-662` écrit `13.5:1`, `4.6:1`, `7.2:1`). Hors périmètre, signalé seulement :
`:660` « 13.5:1 » vaut 14,63:1 recalculé, et `:514` « Axe-core intégré dans la CI » est démenti par #577.

**AC12 — CHANGELOG.**

`CHANGELOG.md` : section `## [0.13.3] — Non publié` (à créer au-dessus de `## [0.13.2] — 2026-10-13` si
absente — d'autres stories de l'epic la créent aussi, fusion par union), sous-section `### Corrigé` :

> - **Les boutons d'action destructive se voient ([#640](https://github.com/guycorbaz/kesh/issues/640)).**
>   *Supprimer l'import*, les confirmations de suppression, d'archivage, de révocation ou de désactivation
>   (factures, contacts, produits, comptes, écritures, comptes bancaires, utilisateurs, clés d'API, taux de TVA,
>   niveaux de relance, restauration d'une sauvegarde, réinitialisation de la démonstration, déverrouillage des
>   livres) s'affichaient comme du texte : leur couleur n'était pas définie dans le thème. Ils ont désormais un
>   fond rouge plein et un texte blanc (contraste 6,47:1), un survol plus foncé et un état désactivé estompé.
>   Les messages, cadres et champs d'erreur qui emploient cette même couleur, censés être rouges, le deviennent
>   aussi — de même que le lien *Annuler le rapprochement* du détail d'un import, les boutons bordés *Annuler le
>   lot*, *Annuler le règlement* et *Annuler la facture* (factures fournisseurs, lots de paiement) et les croix de
>   retrait d'une ligne de saisie, qui passent du gris foncé au rouge. Deux familles de cadres
>   emploient d'autres couleurs, elles aussi non définies, et ne changent pas encore : les cadres d'erreur de la
>   sauvegarde et de la restauration restent sans couleur, trois cadres d'erreur de l'import bancaire gardent leur
>   bordure rouge sans fond ([#645](https://github.com/guycorbaz/kesh/issues/645)).

*[Amendement de la revue de code P1, 2026-10-13 (B1 = A-L1) : le texte ci-dessus promettait trop pour la
troisième croix de la classe E (`JournalEntryForm.svelte`, `hover:text-destructive`), rouge au survol seulement.
Le CHANGELOG écrit : « … les croix de retrait d'une ligne de saisie des factures fournisseurs, qui passent du gris
foncé au rouge ; la croix de retrait d'une ligne d'écriture, elle, ne rougit qu'au survol ».]*

**AC13 — Gates, au dernier commit de code.**

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E
**complet** (recette `docs/testing.md` § *Prérequis Playwright local*, port **3034**, base `kesh_e2e_281`) :
vert au sens de `docs/testing.md` § *Les échecs attendus* (référence 27-4 : 267 passés + 7 KF-029, à juger
fichier par fichier) **plus** la spec neuve ; `scripts/test-fast.sh` complet (le texte des manuels est lu par
`crates/kesh-api/tests/textes_coherents.rs`). Gates lourds sous `flock
/home/gcorbaz/devel/kesh-gate-logs/.gate.lock`. Le Dev Agent Record n'affirme que ce qui a tourné, chiffres
recomptés.

### Amendements de la revue de code P1 (2026-10-13)

Portés par les **tests** seuls (aucune ligne de production) ; le texte des AC ci-dessus reste celui de la
validation, ces amendements le complètent.

- **AC6 (f)** (B2, E3 = A-L3 ; C-28-1-41) : la garde refuse aussi une **directive `class:`** sur un
  `<Button variant="destructive">` ; trois auto-tests synthétiques de `destructifsAvecClasse` ; le montage compte
  par l'**analyseur** (`boutonsDestructifs`), exige l'égalité avec le compte textuel et `>= 25`.
- **AC7 (1) et (1-bis)** (E2) : `focus-visible:ring-destructive/40` est le cinquième mot entier exigé de la
  variante ; la sortie fusionnée le contient et ne contient plus `focus-visible:ring-ring/50`.
- **AC7 (2) et (3)** (E6) : le `@theme` est lu **sans ses commentaires** ; auto-test synthétique.
- **AC7 (4)** (E4 ; C-28-1-43) : `!` final retiré ; préfixes `shadow-`, `from-`, `via-`, `to-`, `ring-offset-`,
  `inset-ring-`, `inset-shadow-` relevés ; deux auto-tests. Angles morts écrits : `.js` et `app.html` non lus,
  table au niveau du nom.
- **AC8 (b)** (E1 ; C-28-1-44) : axe ne court qu'une fois les animations du document finies (infinies exclues), et
  *Supprimer définitivement* doit être dans les `passes` de `color-contrast`, non dans ses `incomplete`.
- **AC12** (B1 = A-L1) : voir l'amendement sous l'AC12.

## Tasks / Subtasks

- [x] **T0 — Captures « avant » et phase rouge** (AC8) : écrire `helpers/contrast.ts` (+ tests),
      `helpers/bank-import-fixtures.ts` (extraction, AC8) et `destructive-buttons.spec.ts` **avant** de toucher
      `app.css` ou `button.svelte` ; les lancer seules avec
      `KESH_E2E_CAPTURES_DIR=/home/gcorbaz/devel/kesh-gate-logs/28-1-captures/avant` — elles **doivent rougir**
      sur les assertions de fond et de couleur (sortie copiée au Debug Log ; un `toHaveCSS` qui expire est le
      rouge attendu ; un test terminé en timeout ne l'est pas, et toute autre exception se documente) et laisser
      les **douze** captures « avant » (assertions `softExpect` à 2 s, délai de test 120 s — C-28-1-32 —,
      captures dans le `finally` de chaque step, passes clair et sombre en tests distincts, C-28-1-29) ; le nombre
      de fichiers produits est copié au Debug Log. ⚠️ Le scénario (a) a besoin de
      l'exemple de l'AC9 : faire d'abord la T4.
- [x] **T1 — Jetons** (AC1) dans `app.css`, doc-comment compris.
- [x] **T2 — Variante** (AC2) dans `button.svelte:14`.
- [x] **T3 — Gardes Vitest** (AC6, AC7) : table de l'AC6 à **quatorze** entrées, motifs écrits, assertion (f) et
      auto-tests sur source synthétique ; AC7 (1) à (4), avec `@tailwindcss/node` déclaré en `devDependencies`
      (même spécificateur que `tailwindcss`, `npm install`, lockfile commité, assertion de version — C-28-1-35)
      et les auto-tests du relevé (C-28-1-34) ; si l'API de l'AC7 (4) ne se charge pas,
      le repli est écrit au Debug Log avec l'erreur.
- [x] **T4 — Page `/design-system`** (AC9).
- [x] **T5 — Captures « après »** : la spec neuve verte, douze captures dans `…/28-1-captures/apres`.
- [x] **T6 — Manuels** (AC10) + `make fr` sous `flock` + contrôles aplatis copiés au Debug Log.
- [x] **T7 — Spec UX et CHANGELOG** (AC11, AC12).
- [x] **T8 — Mutations M1-M22** (§ ci-dessous ; M18 retirée), chacune restaurée par `git checkout -- <fichier>`
      puis `touch` (mémoire *Mutation restaurée, binaire périmé*) ; tableau « mutation → test qui rougit » au
      Debug Log.
- [x] **T9 — Gates** (AC13), au dernier commit de code ; `git diff --stat main -- frontend/src` (AC5) ; grep de
      propagation du symptôme (§ ci-dessous).

## Mutations à tuer (T8)

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | variante rendue à sa valeur d'origine (`bg-destructive/10 … text-destructive …`) | AC7 (1), AC8 (a)(b) |
| M2 | `--color-destructive` retiré d'`app.css` | AC7 (2), AC8 (fond ≠ `rgb(185, 28, 28)`) |
| M3 | `dark:bg-destructive/20` rajouté à la variante | AC7 (1) ; AC8 passe sombre |
| M4 | `--color-destructive: #dc2626` | AC7 (3) — bandeau 4,14 < 4,5 |
| M5 | `--color-destructive-foreground: #fca5a5` | AC7 (3) (3,41 < 4,5), AC8 (`color` ≠ blanc) |
| M6 | `hover:bg-destructive-hover` retiré | AC7 (1), AC8 survol |
| M7 | `--color-destructive-hover: #ef4444` | AC7 (3) (3,76 < 4,5) |
| M8 | variante `bg-transparent border-destructive text-destructive` (contour seul) | AC7 (1) |
| M9 | règle `:focus-visible` d'`app.css:62-65` retirée | AC8 focus |
| M10 | `disabled:opacity-50` retiré de la base | AC8 désactivé |
| M11 | `ContactPersonsManager.svelte:114` : `text-red-600` remplacé par `text-error`, table inchangée | AC6 (jetons changés) |
| M12 | un `<button class="text-destructive">` ajouté dans un fichier quelconque | AC6 (entrée en trop) |
| M13 | une icône ✕ de la table rendue neutre (classe retirée) sans toucher la table | AC6 (entrée en moins) |
| M14 | `fiscal-years/+page.svelte:603` : `bg-red-600 hover:bg-red-700` → `bg-red-700 hover:bg-red-800`, table inchangée | AC6 (jetons changés) |
| M15 | `contrastRatio` sans le `+ 0.05` | `contrast.test.ts` (blanc/noir : 1/0) |
| M16 | `contrastRatio` qui ne trie pas les luminances (`(L(a)+0,05)/(L(b)+0,05)`) | `contrast.test.ts` (cas « foncé en premier ») |
| M17 | la garde AC6 ne lit plus les directives `class:` | AC6 (auto-test sur source synthétique `class:text-destructive`) |
| M18 | *retirée en validation P2* (elle visait la table « sites migrés », retirée avec l'AC5 d'origine, C-28-1-26) | — |
| M19 | `focus-visible:border-destructive` retiré de la variante | AC7 (1) et (1-bis) ; AC8 (a) focus (`border-top-color` ≠ `rgb(185, 28, 28)`) |
| M20 | un `<p class="text-inexistant">` ajouté dans un `.svelte`, puis, second essai, `--color-card: #ffffff` ajouté à `@theme` sans toucher la table | AC7 (4), dans les deux sens |
| M21 | `class="bg-transparent"` ajouté au `<Button variant="destructive">` de `DeleteBankImportAction.svelte:73` | AC6 (f) ; AC8 (b) |
| M22 | le `=` retiré des séparateurs du relevé de l'AC7 (4) | AC7 (4) (auto-test `class:text-inexistant={x}`, C-28-1-34) |

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

| Fichier:ligne | Action | Dialogue avec *Annuler* ? (« oui » : *Annuler* `outline`) |
|---|---|---|
| `lib/features/bank-import/DeleteBankImportAction.svelte:73` | *Supprimer l'import* (déclencheur, page) | — |
| `…/DeleteBankImportAction.svelte:139` | *Supprimer définitivement* | oui (`:135`) |
| `lib/features/reconciliation/CancelReconciliationDialog.svelte:138` | confirmer l'annulation du rapprochement | oui (`:133`) |
| `lib/features/admin-restore/AdminRestorePanel.svelte:182` | *Importer et remplacer l'installation* (déclencheur) | — |
| `…/AdminRestorePanel.svelte:218` | confirmer la restauration | *Annuler* `ghost` |
| `lib/shared/components/DemoBanner.svelte:68` | confirmer la réinitialisation | oui |
| `routes/(app)/accounts/+page.svelte:717` | confirmer le changement de type du compte (*Changer le type quand même*, `account-retype-dialog-confirm`) | oui (`:712`, `autofocus`) |
| `…/accounts/+page.svelte:742` | confirmer l'archivage | oui |
| `routes/(app)/bank-accounts/+page.svelte:586` | confirmer l'archivage | *Annuler* `ghost` (`:590`) |
| `routes/(app)/contacts/+page.svelte:1138` | confirmer l'archivage | oui (`:1136`) |
| `routes/(app)/products/+page.svelte:760` | confirmer l'archivage | oui (`:758`) |
| `routes/(app)/invoices/+page.svelte:519` | confirmer la suppression | oui |
| `routes/(app)/invoices/[id]/+page.svelte:961` | *Supprimer* (déclencheur) | — |
| `…/invoices/[id]/+page.svelte:1088` | *Dévalider* (déclencheur) | — |
| `…/invoices/[id]/+page.svelte:1440` | confirmer la suppression | oui |
| `…/invoices/[id]/+page.svelte:1496` | confirmer la dévalidation | oui |
| `routes/(app)/journal-entries/[id]/+page.svelte:557` | confirmer la suppression | oui (`:553`) |
| `routes/(app)/settings/+page.svelte:568` | *Déverrouiller* les livres (`size="sm"`) | — |
| `routes/(app)/settings/api-keys/+page.svelte:404` | confirmer la révocation | *Annuler* `ghost` (`:409-413`) |
| `routes/(app)/settings/dunning/+page.svelte:300` | confirmer la suppression d'un niveau | oui |
| `routes/(app)/settings/vat-rates/+page.svelte:366` | confirmer la désactivation | *Annuler* `ghost` (`:373`) |
| `routes/(app)/users/+page.svelte:485` | confirmer la désactivation | oui (`autofocus` sur *Annuler*) |
| `routes/design-system/+page.svelte:69` | exemple | — |
| `routes/design-system/+page.svelte:170` | *Toast erreur* (démonstration) | — |

Dix-sept de ces 24 usages sont une confirmation : treize à côté d'un *Annuler* `outline`, quatre à côté d'un
*Annuler* `ghost` (recompté en validation P3, variante du bouton voisin lue à ±10 lignes, C-28-1-36 — la ligne
`:717` décrivait à tort un archivage). Douze sont dans un `Dialog.Root` ; **cinq sont hors** `Dialog.Root`
(recompté en validation P4 ciblée, P4-1) : quatre en ligne dans la page (`<section>` ou `<div>`) — `dunning:300`
(*Annuler* `outline`), `vat-rates:366`, `api-keys:404`, `bank-accounts:586` (*Annuler* `ghost`) — et
`journal-entries/[id]:557`, dans un overlay `role="dialog"` maison (`{#if showDeleteConfirm && entry}`,
`:526-530`), à côté d'un *Annuler* `outline` (`:553`). Un *Annuler* `ghost`
se lit comme du texte tant que #645 n'est pas fait — hors périmètre (angle mort 1).

**Classe B — `<Button>` rouge par `class` : 2 sites, laissés tels quels, inscrits à la table de l'AC6**
(C-28-1-26) : `fiscal-years:603`, `email-templates:439`. Déjà visibles : `bg-red-600` — le `red-600` de
Tailwind **v4**, `oklch(57.7% 0.245 27.325)` (`node_modules/tailwindcss/theme.css:16`), ≈ `#e7000b` —, texte blanc
≈ 4,8:1 (valeur approchée, conversion oklch non rendue). Leur passage à la variante serait une affaire de DRY et
de garde, non de visibilité (angle mort 11).

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

**Classe D — `<button>` brut sans bordure, texte rouge : 3 sites, laissés tels quels, inscrits à la table de
l'AC6** (C-28-1-26) : `ContactPersonsManager.svelte:114` (`text-red-600 hover:underline`, déjà rouge),
`BankProfileList.svelte:79` (`text-error`, déjà rouge) et `bank-import/[id]/+page.svelte:143` (`text-destructive
underline`, à côté du lien *Voir l'écriture* ; **devient rouge** par l'AC4, aujourd'hui gris foncé). Ils se lisent
comme du texte ou comme un lien — la forme du symptôme de #640, mais non le bouton que nomme l'issue, qui est
*Supprimer l'import* (classe A). Angle mort 11, porté par **#648**.

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

**Déclencheurs neutres bordés d'une action destructive — hors périmètre** : `journal-entries/[id]/+page.svelte:327`
(*Supprimer*, `outline`), `settings/projects/+page.svelte:274/298` (*Archiver*, `outline`), `DemoBanner.svelte:51`
(*Réinitialiser*, `outline`), `settings/+page.svelte:544` (*Déverrouiller*, `outline`, ouvre la confirmation
`:568`), `RulesList.svelte:161` (*Désactiver*, bordé neutre), `InvoiceSettlements.svelte:86-95` (*Annuler le
règlement* / *Annuler le solde*, `invoice-settlement-cancel`, `outline sm`) et `:106-117` (*Annuler le
rapprochement*, `invoice-settlement-cancel-reconciliation`, `outline sm`). Ils portent une bordure et ouvrent une
confirmation : leur sobriété est une hiérarchie voulue, non le défaut de l'issue — sous réserve d'une bordure
`outline` à 1,23:1 (angle mort 9, #645).

**Déclencheurs `ghost` d'une action destructive — hors périmètre, renvoyés à #645** : la variante `ghost`
(`button.svelte:13`) n'a ni fond ni bordure, et son survol `hover:bg-muted` est sans règle (jeton absent) : ils
**se lisent comme du texte** tant que #645 n'a pas défini `muted`. Avec libellé : `settings/dunning/+page.svelte:241`
(*Supprimer* un niveau), `settings/vat-rates/+page.svelte:245-250` (*Désactiver* un taux). À icône, **sept** :
`accounts/+page.svelte:499` (*Archiver*), `contacts/+page.svelte:784` (*Archiver*), `products/+page.svelte:581`
(*Archiver*), `users/+page.svelte:344` (*Réinitialiser le mot de passe*) et `:348` (*Désactiver*),
`invoices/+page.svelte:440` (*Supprimer la facture*), `settings/fiscal-years/+page.svelte:417`
(*Clôturer* un exercice, `fiscal-year-close-{fy.id}`, `size="icon-xs"`, ouvre la confirmation de la classe B
`:603` — ajouté en validation P3, C-28-1-36 : le commentaire de #645 annonce « sept `ghost` à icône » sans les
nommer, la fiche en listait six ; le relevé des `Button variant="ghost"` à gestionnaire `onclick` donne ce
septième (`grep -rn -A6 'variant="ghost"' src --include=*.svelte | grep onclick`, gestionnaires relus ;
*Rouvrir* un exercice, `fiscal-years:446`, n'est pas compté destructif)) ;
retrait de ligne, à icône : `InvoiceForm.svelte:850`, `TransactionSplitModal.svelte:313`. Relevés par la lentille
R de la validation P2 (gestionnaire `onclick` destructif sur un `Button` non destructif) ; #645 les porte.

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
   trois noms relevés par la lentille R de la validation P1 — `error-soft` (3 sites), `warning-soft` (8 sites),
   `danger` (`AdminRestorePanel.svelte:173`, `AdminBackupPanel.svelte:71`) — et un treizième relevé par les deux
   lentilles de la validation P2, `surface-hover` (`credit-notes:59`, `payment-batches:229`,
   `supplier-invoices:452`, survol des lignes de trois listes). Effet visible : la variante `outline` (*Annuler*)
   n'a ni fond ni survol (`bg-background`, `hover:bg-muted` sans règle), `ghost` n'a ni fond ni survol — d'où
   les *Annuler* `ghost` de quatre dialogues et les déclencheurs `ghost` destructifs (Dev Notes § Inventaire) qui
   se lisent comme du texte —, `focus-visible:ring-ring` est sans effet (le contour global bleu compense ; la
   variante destructive fixe sa propre bordure et son propre anneau de focus, C-28-1-20). **Hors périmètre**
   (C-28-1-13, C-28-1-17, C-28-1-26). La validation P1 a mesuré que les définir n'est pas neutre — `input` sert à
   la fois de bordure (≥ 3:1 requis) et de fond désactivé, `muted-foreground` au ton shadcn `#64748b` tombe à
   4,34:1 sur `muted` : ces mesures sont à reprendre dans #645, non ici. La garde AC7 (4) tient ces treize noms
   en table fermée, renvoyée à #645 ; #645 porte les treize noms et les déclencheurs `ghost`.
2. **Suppression d'une personne de contact sans confirmation — renvoi à #646** (P3, ouverte)
   (`ContactPersonsManager.svelte:76-83` : `removePerson` appelle `deleteContactPerson` directement), contraire
   au principe UX « confirmations avant actions destructives » (spec UX l. 286). La story ne touche ni ce bouton
   (angle mort 11) ni sa confirmation (C-28-1-17).
3. **Sous `prefers-color-scheme: dark`**, des classes `dark:` d'autres composants s'activent déjà
   (`dark:bg-amber-950/30`, `dark:bg-green-900`… émises dans le build) alors que Kesh n'a pas de thème sombre.
   Préexistant, hors périmètre ; la variante destructive n'en porte plus aucune (AC2).
4. **Taille de cible** : `h-8` (32 px) pour toutes les variantes, sous les 44 px du mode Guidé (spec UX l. 718).
   Préexistant, commun à tous les boutons.
5. **Pas de régression au pixel** : les captures ne sont pas comparées (C-28-1-10). Une régression de style hors
   des propriétés mesurées (taille, police, espacement) n'est vue que par l'œil.
6. **La CI n'exécute aucun E2E** (#577) : en CI, seules les gardes Vitest (AC6, AC7) protègent la variante.
7. **Les 24 usages de la classe A ne sont montrés à l'écran que pour trois d'entre eux** (`design-system:69`,
   `DeleteBankImportAction:73` et `:139`, AC8) : les autres sont tenus par construction (AC7 (1)-(2) sur la
   variante, AC6 (f) contre toute `class`, C-28-1-28). Restent invisibles à toute garde : un `variant="destructive"`
   **retiré** d'un usage, et un style propre au conteneur d'un des 21 autres usages qui écraserait la variante.
8. **Les cinq boutons rouges stylés à la main ne sont montrés à l'écran par aucun test de la story** : l'AC6 tient
   leurs jetons, non leur rendu (angle mort 11).
9. **Bordure d'`outline` à 1,23:1** contre le blanc (`--color-border` `#e2e8f0` : L = 0,8017 ; 1,05 / 0,8517) —
   la bordure de tous les *Annuler* `outline` de l'application, dont celui du dialogue de (b), et des déclencheurs
   neutres bordés. Le bouton reste identifiable par sa forme et son libellé (texte courant `#1e293b`, 14,63:1) ;
   le contraste de la bordure n'est pas traité ici. **Renvoyé à #645**, qui le porte depuis la validation P2.
10. *[Revue de code P1, E2 : la PRÉSENCE de la classe est désormais assertée par l'AC7 (1) et (1-bis) — retirer
    `focus-visible:ring-destructive/40` rougit (P1-M1). Reste vrai ce qui suit pour sa COULEUR rendue.]*
    **L'anneau de focus `focus-visible:ring-destructive/40` n'est asserté par rien** : sa couleur passe par
    `color-mix(in oklab, …)`, que `getComputedStyle` ne rend pas en `rgb(…)`. Il est vu sur la capture
    `a-focus-<clair|sombre>.png` (AC8), cadrée 8 px au-delà de la boîte de bordure pour qu'il y figure
    (C-28-1-33), par l'œil seulement ; la bordure et le contour de focus, eux, sont assertés (AC3).
11. **Cinq boutons rouges stylés à la main restent tels quels** (C-28-1-26) : classe B (`fiscal-years:603`,
    `email-templates:439`, déjà visibles, rouge de palette au lieu de la variante) et classe D
    (`ContactPersonsManager:114`, `BankProfileList:79`, `bank-import/[id]:143` — trois `<button>` sans bordure
    qui se lisent comme du texte ou comme un lien ; le troisième devient un lien **rouge** par l'AC4, à côté de
    *Supprimer l'import*). Ce n'est pas du thème : **#648** les porte (ouverte par l'orchestrateur), et reprend
    aussi la question de la forme d'*Annuler le rapprochement* (C-28-1-21 annulé : `outline` le rendait moins
    visible tant que #645 n'est pas fait). Des cinq, **un seul dépend du jeton** : `bank-import/[id]:143`
    (`text-destructive underline`), qui devient rouge par l'AC4 ; les quatre autres sont déjà rouges par la
    palette ou par `error` (`bg-red-600`, `text-red-600`, `text-error`) — C-28-1-36.
12. **Relevé de l'AC7 (4) : un jeton suivi d'une ponctuation** (`text-foo.` ou `text-foo,` dans un commentaire ou
    une chaîne de prose) est écarté par l'ancre `$` : un nom non défini écrit **seulement** ainsi échapperait à la
    garde. Aucune classe Tailwind ne s'écrit ainsi dans un attribut ; le trou est assumé (C-28-1-34).
13. *[Ajouté en revue de code P1, E4, C-28-1-43.]* **Relevé de l'AC7 (4) : les `.js` de `src/` et `app.html` ne
    sont pas lus** (aucune classe de couleur au 2026-10-13), et **la table tient des noms** : un `bg-danger` neuf
    passe, `danger` y figurant déjà (voulu, #645 traite le nom). Le `!` final et les préfixes `shadow-`, `from-`,
    `via-`, `to-`, `ring-offset-`, `inset-ring-`, `inset-shadow-`, eux, sont relevés depuis la remédiation P1.
    *[Revue de code P2, P2-2 :]* le relevé lit **tout** le texte des `.svelte` et `.ts`, non les seules valeurs de
    `class` (des classes passent par des chaînes de `cn(…)` en script) : un mot de prose ou un identifiant commençant
    par `to-`, `from-` ou `via-` (`to-do`) rougirait la garde au premier ajout. Faux rouge **bruyant**, jamais
    silencieux ; aucun tel mot au 2026-10-13 ; à examiner puis traiter par la table, non en retirant le préfixe.
    *[P2-1 :]* `bg-foo!/40` est relevé comme `bg-foo/40!` depuis la remédiation P2.

### État actuel des fichiers modifiés — ce qui doit être préservé

- `button.svelte` : `buttonVariants` est **exporté** (`ui/button/index.ts:5,13`), sans autre consommateur dans
  `src/` au 2026-10-13 (`grep -rn buttonVariants frontend/src`) ; seule la ligne `destructive` change. Le rendu
  `<a>` (avec `href`) et `<button>` reste identique.
- `app.css` : ordre du `@theme`, styles globaux, espacement Guidé/Expert et règle `:focus-visible` inchangés.
- Sites applicatifs : **aucun n'est édité** (AC5). Les tests qui ciblent les boutons de la classe A et des
  classes B à E, par `data-testid` seulement (aucun n'asserte une classe), ne changent pas.
- Le seul test du dépôt qui lit la classe `text-destructive` : `routes/(app)/contacts/contacts-page.test.ts:729`
  (`document.querySelector('p.text-destructive')`) — la classe ne change pas, il reste vrai.
- `bank-import-delete.spec.ts` : son scénario ne change pas, seul son montage passe par le helper extrait.
- `frontend/package.json` : seule une ligne `devDependencies` s'ajoute (`@tailwindcss/node`, AC7 (4)) ; le
  lockfile ne doit changer que pour cette entrée (`git diff package-lock.json` relu au Debug Log).

### Propagation post-patch (T9)

Greper la **valeur** après chaque correctif : `grep -rnE '#b91c1c|#991b1b|6,47|5,46|8,31|2,54|4,14|1,23'
frontend/src _bmad-output/implementation-artifacts/28-1-* _bmad-output/planning-artifacts/ux-design-specification.md
CHANGELOG.md` (les ratios cités doivent tous être ceux d'`app.css`, calculés par la méthode du helper,
C-28-1-27 ; la spec UX les écrit avec un point) ; `grep -rnE '5,45|2,53|4,13|4,83|5,58|4,59|\b33 bandeaux'
_bmad-output/implementation-artifacts/28-1-boutons-destructifs.md` → seulement le Change Log et les rectifications
du registre (valeurs retirées en validations P1 et P2), plus la mention de 5,45 comme valeur **non citée** de
l'AC1 ; `grep -rn 'bg-destructive/10 hover' frontend/src` → 0 ; `grep -rn 'est grisé' docs/manual/fr/*.tex` → 0
après l'AC10. (Le contrôle `variant="outline"` sur `bank-import/[id]/+page.svelte` est retiré en validation P3 :
le fichier ne porte aucun `variant=`, le « 0 » était vrai avant comme après la story — C-28-1-36.)

### Découpage préventif — trois modules, pas de dérogation (C-28-1-26, remplace C-28-1-24)

**Recompte, sur le périmètre réduit de la validation P2.** Modules de code de premier niveau édités, dans
`frontend/src/` :

| # | Module | Ce qui change |
|---|---|---|
| 1 | `lib/components/ui` (`button/` et, pour la garde AC7 (4), `ui/` lui-même) | variante (1 ligne) ; gardes AC6, AC7 |
| 2 | `app.css` (thème) | trois jetons |
| 3 | `routes/design-system` | un exemple, un `data-testid`, un carré de palette |

Hors code de production : `tests/e2e` (spec neuve, deux helpers, montage de `bank-import-delete.spec.ts`),
`frontend/package.json` et son lockfile (une `devDependency`), `docs/manual/fr` (deux phrases, deux PDF), la spec
UX, le CHANGELOG. Aucun crate, aucune clé i18n, aucune migration, aucune règle métier.

**Trois modules : le critère « plus de 5 modules » n'est pas franchi** ; la dérogation de C-28-1-24 (sept modules,
dont la conformité à l'exception codifiée était contestée par les deux lentilles P2, R6 et F9) est **sans objet**,
et son « risque accepté » (R12 = F11) avec elle. Le second critère, lui, a été levé : la validation P2 a rendu une
sévérité égale à la P1 (MEDIUM), et la plupart de ses défauts étaient nés de la remédiation P1 — c'est le
**recyclage** que l'amendement D5 désigne. L'orchestrateur, sur l'autonomie déléguée par Guy le 2026-10-13
(`epic-28-recette-v0-13-2.md`), a décidé de ne pas découper formellement et de **réduire le périmètre au strict
nécessaire pour fermer #640** (C-28-1-26) ; le signal est déclaré au Project Lead (Change Log, validation P2).

### Pièges connus du dépôt, applicables ici

- **E2E** : `KESH_COOKIE_SECURE=false`, `KESH_HOST=127.0.0.1`, `KESH_TEST_MODE=true`,
  `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, `KESH_BACKEND_URL=http://127.0.0.1:3034`,
  `KESH_STATIC_DIR` vers le `frontend/build` **du worktree** (rebuild après chaque changement de CSS : le
  backend sert le build, non les sources), `KESH_ADMIN_BACKUP_DIR` inscriptible, `CARGO_TARGET_DIR` du worktree.
  Attendre `/health` ; tuer le backend par PID.
- **Base** : `kesh_281` / `kesh_e2e_281`, jamais `restart` du conteneur partagé (consigne 2).
- **Édition** du code par l'outil Edit uniquement (consigne 14) ; mutations restaurées par `git checkout --` +
  `touch`.
- **tailwind-merge** (`cn`) : ne passer **aucune** `class` à un `Button variant="destructive"` (l'assertion (f)
  de l'AC6 la refuse).
- **Données réelles** : aucune ; les captures ne montrent que les fixtures synthétiques.

### Fichiers probablement communs avec les autres stories de l'epic

`CHANGELOG.md` (section `[0.13.3] — Non publié`, créée par plusieurs stories), `sprint-status.yaml` (section
`epic-28`, créée aussi par la 28-0), `docs/manual/fr/user-manual.{tex,pdf}` et `admin-manual.{tex,pdf}` (la 28-4
et la 28-5 les réécrivent : **conflit binaire des PDF**, résolu en régénérant après fusion),
`frontend/package.json` et son lockfile (une `devDependency` ajoutée). La 28-1 n'édite plus
`BankProfileList.svelte` ni `bank-import/[id]/+page.svelte` (C-28-1-26) : plus de conflit avec la 28-2 ni la 28-5
sur ces fichiers.

### References

- Issue #640 ; epic `epic-28-recette-v0-13-2.md` § 28-1 et § *Recette visuelle (B2)*.
- `_bmad-output/planning-artifacts/ux-design-specification.md` l. 286, 446, 514, 639-662, 714-718.
- Patron de table fermée : `frontend/src/lib/features/bank-accounts/bank-name-inventory.test.ts`.
- `frontend/src/lib/components/ui/sonner/sonner.svelte:3` (« Kesh n'a pas de dark mode »).
- WCAG 2.2 : 1.4.3 (exemption des composants inactifs), 1.4.11, 2.4.7.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), `bmad-dev-story` en autonomie, 2026-10-13. Worktree `kesh-28-1`, port 3034,
bases `kesh_281` / `kesh_e2e_281`, `CARGO_TARGET_DIR` du worktree ; gates lourds, campagne de mutations, E2E et
`make` des manuels sous `flock kesh-gate-logs/.gate.lock`.

### Debug Log References

Journaux hors dépôt, `/home/gcorbaz/devel/kesh-gate-logs/` : `28-1-t0-rouge.log` (phase rouge),
`28-1-t5-vert.log`, `28-1-make-fr.log`, `28-1-mutations.log` et `28-1-mut/` (sorties par mutation),
`28-1-gate-testfast.log`, `28-1-gate-front-*.log`, `28-1-gate-e2e.log`, `28-1-gate-e2e-rejeu-invoices.log`.
Captures : `28-1-captures/{avant,apres,gate}/`.

**Ordre suivi** : T4 (exemple de l'AC9, prérequis du scénario (a)), puis helpers, spec E2E et gardes Vitest, puis
**T0 avant toute édition d'`app.css` et de `button.svelte`** (le build de la phase rouge a été fait, sous verrou, sur
un arbre où seule la T4 touchait `frontend/src`), puis T1/T2, T5, T6, T7, T8, T9.

**T0 — phase rouge** (`28-1-t0-rouge.log`, commit de base `bbd0a9a1` + T4 et tests, non commités) :
`destructive-buttons.spec.ts` **4 tests rouges / 4**, `bank-import-delete.spec.ts` (montage extrait) **vert**.
Toutes les erreurs sont des `toHaveCSS` expirés à 2 s — **22**, aucune autre exception, **aucun timeout de test**
(durées 15,4 s, 15,4 s, 10,7 s, 10,8 s) :

```
Error: expect(locator).toHaveCSS(expected) failed     ×22
Expected: "rgb(185, 28, 28)"   Received: "rgba(0, 0, 0, 0)"   (fond, repos/focus/désactivé/Supprimer…)
Expected: "rgb(255, 255, 255)" Received: "rgb(30, 41, 59)"    (texte : le texte courant #1e293b)
Expected: "rgb(153, 27, 27)"   Received: "rgba(0, 0, 0, 0)"   (survol)
4 failed, 1 passed (58.5s)
```

Ventilation : (a) 7 par schéma (repos fond + texte, survol fond + texte, focus bordure + fond, désactivé fond), (b) 4
par schéma (*Supprimer l'import* fond + texte, *Supprimer définitivement* fond + texte) : 2 × 7 + 2 × 4 = 22. Restés
**verts** dès la phase rouge, et c'est une information : `:focus-visible` vrai après Shift+Tab/Tab, `outline-style:
solid` et `outline-color: rgb(59, 130, 246)` — la règle globale hors couche l'emporte bien sur `outline-none`
(lecture de l'AC3 vérifiée, aucune correction de la variante nécessaire) — ; `opacity: 0.5` du désactivé ; *Annuler*
sans fond rouge ; axe `color-contrast` 0 violation (un texte `#1e293b` sans fond passe). **12 fichiers** produits
dans `28-1-captures/avant/` (les douze noms attendus). Les gardes Vitest, écrites avant T1/T2, étaient rouges elles
aussi : `button-destructive.test.ts` **16 / 17** (seul « est lue sur la variante seule » vert : `.variants` est
exposé, la lecture par `readFileSync` n'a pas servi) ; `jetons-non-definis.test.ts` 1 / 7, sur le seul nom
`destructive` ; `boutons-rouges-inventory.test.ts` vert (il ne dépend pas du thème).

**T5** (`28-1-t5-vert.log`) : 5 / 5 verts (4 neufs + `bank-import-delete`), 12 captures dans `…/apres/`, regardées :
fond rouge plein et texte blanc au repos, plus foncé au survol, contour bleu et anneau rouge au focus, rouge estompé
désactivé, *Supprimer définitivement* rouge plein à côté d'un *Annuler* bordé, sombre identique au clair.

**AC1** — après `npm run build` :

```
$ grep -c 'bg-destructive' build/_app/immutable/assets/*.css
…/0.BjaBLe4V.css:1        (0 dans les six autres feuilles)
```

Ratios recalculés par la méthode du helper (mélanges non arrondis, arrondi au centième) : blanc/destructive 6,47 ;
blanc/destructive-hover 8,31 ; destructive/bandeau `/10` 5,46 ; désactivé effectif 2,54 ; `#dc2626`/son bandeau 4,14 ;
contour `#3b82f6`/blanc 3,68 — conformes à la fiche.

**AC5** — `git diff --stat bbd0a9a1 3b0f7707 -- frontend/src` :

```
 frontend/src/app.css                               |   8 +
 .../ui/button/boutons-rouges-inventory.test.ts     | 314 +++++
 .../ui/button/button-destructive.test.ts           | 135 +++++
 .../src/lib/components/ui/button/button.svelte     |   8 +-
 .../lib/components/ui/jetons-non-definis.test.ts   | 184 +++++
 frontend/src/routes/design-system/+page.svelte     |   4 +-
 6 files changed, 651 insertions(+), 2 deletions(-)
```

Trois modules (`app.css`, `lib/components/ui`, `routes/design-system`), aucun site applicatif.

**AC7 (4) — l'API s'est chargée** (C-28-1-40) : pas de repli. **AC6 — table écrite depuis la fiche** ; la première
exécution n'a rougi que sur deux clés dont la fiche ne donnait que la forme (C-28-1-39), lues au sol.

**Lockfile** (`git diff package-lock.json`) : l'entrée racine gagne `"@tailwindcss/node": "^4.2.2"`, l'entrée
`node_modules/@tailwindcss/node` perd `"license": "MIT"` (normalisation de npm 9.2.0) ; rien d'autre (C-28-1-37).

**T6 — manuels** (`make fr`, sous verrou) : `admin-manual.pdf` et `user-manual.pdf` stables en 3 passes ;
`marketing-brochure.pdf`, régénéré par la même cible sans changement de source, a été **remis** à sa version
commitée (aucune modification à verser). Contrôles aplatis :

```
pdftotext user-manual.pdf  | … | grep -o "le bouton est désactivé, avec le nombre" | wc -l   → 1
pdftotext admin-manual.pdf | … | grep -o "le bouton est désactivé avec leur nombre"  | wc -l   → 1
les deux PDF | grep -o "le bouton est grisé" | wc -l                                        → 0
grep -rn 'est grisé' docs/manual/fr/*.tex | wc -l                                           → 0
```

**T8 — mutations** (`28-1-mutations.log`, `28-1-campagne` à `HEAD=3b0f7707`, arbre propre exigé avant chacune, motif
unique vérifié, restauration `git checkout --` puis `touch`). Gardes Vitest à chaque mutation (les quatre fichiers
neufs, 41 tests) ; E2E `destructive-buttons.spec.ts` (frontend rebâti) pour celles dont la fiche nomme l'AC8.
**21 / 21 actives tuées** (M20 jouée en ses deux essais) :

| # | Vitest rouge | E2E rouge |
|---|---|---|
| M1 | 7 — (1) quatre mots entiers, aucun `dark:`, ni `bg-destructive/<n>`, (1-bis) | 4/4 — fonds, textes, bordure de focus |
| M2 | 7 — (2) `--color-destructive` défini et jetons de la variante, (3) quatre ratios, AC7 (4) (a) | 4/4 — fonds |
| M3 | 2 — (1) aucun `dark:`, ni `bg-destructive/<n>` | 2/4 — les deux passes **sombres** seules (fonds ; axe en (b)) |
| M4 | 1 — (3) bandeau | — |
| M5 | 2 — (3) texte/fond et texte/survol | 4/4 — `color`, et axe en (b) |
| M6 | 1 — (1) `hover:bg-destructive-hover` | 2/4 — (a) survol |
| M7 | 1 — (3) texte/survol | — |
| M8 | 7 — (1) quatre mots entiers, `bg-transparent`, (1-bis), (2) jetons de la variante | — |
| M9 | 0 (attendu : AC8 seule) | 2/4 — (a) `outline-style`, `outline-color` |
| M10 | 0 (attendu : AC8 seule) | 2/4 — (a) `opacity` |
| M11 | 1 — AC6 table (jetons changés) | — |
| M12 | 1 — AC6 table (entrée en trop) | — |
| M13 | 1 — AC6 table (entrée en moins) | — |
| M14 | 1 — AC6 table (jetons changés) | — |
| M15 | 4 — blanc/noir dans les deux ordres, 6,47, bandeau | — |
| M16 | 6 — les cas « foncé en premier » de `contrast.test.ts` et trois ratios de (3) | — |
| M17 | 1 — auto-test « une directive class: est relevée » | — |
| M19 | 2 — (1) `focus-visible:border-destructive`, (1-bis) | 2/4 — (a) `border-top-color` au focus |
| M20 (a) `<p class="text-inexistant">` | 1 — AC7 (4) (a) | — |
| M20 (b) `--color-card` | 1 — AC7 (4) (b) | — |
| M21 | 1 — AC6 (f) | 2/4 — (b) fond de *Supprimer l'import* |
| M22 | 1 — auto-test « une directive class: livre son nom » | — |

Constat de M8 : la garde AC7 (2) rougit aussi sur `bg-transparent` (`--color-transparent` n'est pas un jeton du
`@theme`) — lecture littérale de la règle « `(bg|text|border|ring)-X` → `--color-X` » de l'AC7 (2). C'est sans
conséquence aujourd'hui (l'AC7 (1) interdit `bg-transparent` dans la variante), mais un mot-clé Tailwind
(`transparent`, `current`, `white`…) ajouté un jour à la variante ferait rougir (2) à tort : le message le dirait.

**T9 — gates au dernier commit** (`HEAD=3b0f7707`, dernier commit de code `37b74298`, `3b0f7707` ne portant que les
PDF ; bases `kesh_281` et `kesh_e2e_281` recréées avant le gate backend **et** avant l'E2E) :

- `scripts/test-fast.sh` complet (fmt + clippy `-D warnings` + nextest) : **3514 exécutés, 3514 passés, 6 ignorés**
  (153,4 s).
- Frontend : `npm run check` 0 erreur (29 avertissements préexistants, 5 fichiers) ; `lint-i18n-ownership` PASS ;
  `npm run test:unit` **1436 / 1436**, 138 fichiers ; `npm run build` vert.
- **E2E complet** (port 3034, `kesh_e2e_281`, recette de `docs/testing.md`, `smtpConfigured: true`) :
  **269 passés, 9 échoués, 19 ignorés** (11,0 min). Les 9, fichier par fichier contre `docs/testing.md` § *Les
  échecs attendus* : les **7 de la KF-029** (#97) — `mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`,
  `onboarding:57`, `:77`, `:150` — et les **2 de la KF-045** (#421) — `invoices.spec.ts:415` « historique des
  rappels affiché » et `:439` « axe-core sans violations sur la fiche », `manual reminder failed: 422`, run lancé
  avant 12:00 UTC (11:3x UTC). Aucun échec de pollution sur ce run. La spec neuve, 4 / 4 verte ; les 16 fichiers à
  `color-contrast` actif, verts — `invoices.spec.ts` compris, dont l'analyse `:439` (KF-045) rougissait au montage du rappel et passe au rejeu.
  **Rejeu** de `invoices.spec.ts` seul, base `kesh_e2e_281` recréée, lancé à 12:01 UTC (même `HEAD`, frontend
  rebâti) : **23 / 23 passés**, dont `:415` et `:439` — confirmé KF-045, sans rapport avec la story
  (`28-1-gate-e2e-rejeu-invoices.log`). Bilan E2E, les deux runs réunis : 271 passés (269 + 2), 7 échoués, tous KF-029, 19 ignorés.
- Propagation (T9) : `grep -rn 'bg-destructive/10 hover' frontend/src` → 0 ; ratios cités = ceux d'`app.css` dans
  `app.css`, la spec UX (point décimal) et le CHANGELOG ; `est grisé` → 0 dans les `.tex`.

### Completion Notes List

- **AC1-AC3** : trois jetons dans `@theme`, variante pleine sans `dark:`, quatre états mesurés au style calculé
  (E2E), identiques en sombre émulé. La règle globale `:focus-visible` l'emporte sur `outline-none` (vérifié dès T0).
- **AC4** : aucun site édité ; le jeton défini rend leur rouge aux bandeaux, textes et champs d'erreur et aux six
  `<button>` bruts. Les 16 specs à `color-contrast` restent vertes.
- **AC6-AC7** : trois gardes Vitest neuves (32 tests) et les tests du helper (9) — **41 tests Vitest neufs**, de
  `bbd0a9a1` à `HEAD`. Quatorze entrées en table fermée (AC6), treize noms non définis renvoyés à #645 (AC7 (4)).
- **AC8** : 4 tests E2E neufs (2 scénarios × 2 schémas) ; montage d'import extrait dans un helper partagé, scénario
  de `bank-import-delete.spec.ts` inchangé et vert.
- **AC9-AC12** : `/design-system` (exemple désactivé, `data-testid`, palette) ; manuels et PDF ; spec UX ; CHANGELOG.
- **Écarts à la fiche** : 25 usages de la variante au `HEAD` au lieu de 24 (l'exemple désactivé de l'AC9 est un
  usage), `design-system:69` devenu `:70` (C-28-1-38) ; deux `data-testid` lus au sol (C-28-1-39) ; pas de repli de
  l'AC7 (4) (C-28-1-40) ; `@tailwindcss/node` installé à `4.2.2` explicitement (C-28-1-37) ; constat de M8 sur l'AC7
  (2) (ci-dessus). Aucun autre écart.
- **Angles morts** : ceux de la fiche, 1 à 12, inchangés (Dev Notes) ; renvois #645, #646, #648.
- **Demandes à l'orchestrateur** : push de `story/28-1`, PR `closes #640` (et `refs #645 #646 #648`) ; aucune issue
  neuve.

### File List

- `frontend/src/app.css` (modifié)
- `frontend/src/lib/components/ui/button/button.svelte` (modifié)
- `frontend/src/lib/components/ui/button/button-destructive.test.ts` (nouveau)
- `frontend/src/lib/components/ui/button/boutons-rouges-inventory.test.ts` (nouveau)
- `frontend/src/lib/components/ui/jetons-non-definis.test.ts` (nouveau)
- `frontend/src/routes/design-system/+page.svelte` (modifié)
- `frontend/tests/e2e/destructive-buttons.spec.ts` (nouveau)
- `frontend/tests/e2e/helpers/bank-import-fixtures.ts` (nouveau)
- `frontend/tests/e2e/helpers/contrast.ts` (nouveau)
- `frontend/tests/e2e/helpers/contrast.test.ts` (nouveau)
- `frontend/tests/e2e/bank-import-delete.spec.ts` (modifié — montage importé du helper)
- `frontend/package.json`, `frontend/package-lock.json` (modifiés — `@tailwindcss/node`)
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf` (modifiés)
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf` (modifiés)
- `_bmad-output/planning-artifacts/ux-design-specification.md` (modifié)
- `CHANGELOG.md` (modifié — `## [0.13.3] — Non publié`)
- `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` (C-28-1-37 à C-28-1-40)
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/28-1-review-prompt-p1.md` (nouveau)

Remédiation de la revue de code P1 (commit de code `99682720`) : les trois gardes Vitest, la spec
`destructive-buttons.spec.ts`, `CHANGELOG.md`, `frontend/package-lock.json` (modifiés) ; registre (C-28-1-41 à
C-28-1-44) ; `28-1-review-prompt-p2-ciblee.md` (nouveau). **Aucun fichier de production** :
`git diff --stat 37b74298..HEAD -- frontend/src ':!*.test.ts' ':!*.spec.ts'` ne rend rien.

### Remédiation de la revue de code P1 (2026-10-13, Opus 5.5)

**Mutations de la remédiation** — journal `kesh-gate-logs/28-1-mutations-p1.log`, chacune à motif vérifié unique,
restaurée par `git checkout --` puis `touch`, au `HEAD` `99682720`. **11 / 11 rouges** :

| # | Mutation | Rougit |
|---|---|---|
| P1-M1 | `focus-visible:ring-destructive/40` retiré de la variante (`button.svelte`) | 2 — (1) le mot entier, (1-bis) |
| P1-M2 | `blocTheme` ne retire plus les commentaires | 1 — auto-test du jeton commenté |
| P1-M3 | `--color-destructive-hover` mis en commentaire dans `app.css` | 3 — (2) défini, (2) tout jeton, (3) survol |
| P1-M4 | (f) ne refuse plus la directive `class:` | 1 — auto-test de la directive |
| P1-M5 | `class:bg-transparent={false}` ajouté à *Supprimer l'import* | 1 — (f) sur le dépôt |
| P1-M6 | `boutonsDestructifs` : `===` → `!==` | 5 — trois auto-tests, (f), montage |
| P1-M7 | `variant="destructive"` retiré de l'exemple désactivé de `/design-system` | 1 — montage (`>= 25`) |
| P1-M8 | le `!` final n'est plus retiré | 1 — auto-test du `!` final |
| P1-M9 | `shadow`, `from`, `via`, `to` retirés du motif | 1 — auto-test des préfixes |
| P1-M10 | E2E : `withRules(['button-name'])` au lieu de `color-contrast` | 2/2 tests (b) — « absent de passes » |
| P1-M11 | E2E : `class="bg-linear-to-r from-destructive to-destructive-hover"` sur *Supprimer définitivement* | 2/2 tests (b) — absent de `passes` ET présent dans `incomplete` ; `violations` vide |

P1-M11 est la preuve d'E1 : l'ancien test (`violations == []` seul) serait resté vert. L'attente des animations,
elle, n'a pas de mutation qui rougisse de façon déterministe (le fondu dure 100 ms) : c'est une mesure de
robustesse, non une assertion.

**Premier essai d'E1 rouge à tort**, consigné : l'identification par `node.html` rendait « absent de passes »
sans mutation — axe tronque le `html` d'un élément long avant le `data-testid`. Remplacée par la résolution de
`target[0]` dans la page (C-28-1-44) ; E2E ciblé `destructive-buttons.spec.ts` ensuite vert, 4/4
(`kesh-gate-logs/28-1-p1rem-e2e-cible.log`).

**Gates au dernier commit de code `99682720`**, sous `flock`, bases `kesh_281` et `kesh_e2e_281` remises à zéro
(drop, create, migrations, seed) avant le backend et avant l'E2E — journaux `kesh-gate-logs/28-1-p1rem-*` :

- `scripts/test-fast.sh` : `3514 tests run: 3514 passed, 6 skipped` (aucun Rust touché) ;
- `npm run check` : 0 erreur, 29 avertissements (5 fichiers), comme au développement ;
  `lint-i18n-ownership` : PASS ; `npm run build` : vert ;
- `npm run test:unit` : **1443 / 1443**, 138 fichiers. Périmètre : de `5ded5663` à `99682720`, +7 tests exécutés
  (`button-destructive` 17 → 19, `boutons-rouges-inventory` 8 → 11, `jetons-non-definis` 7 → 9) ; 1436 + 7 = 1443 ;
- **E2E complet**, port 3034 : **271 passés, 7 échoués, 19 ignorés** (11,0 min, fin 13:45 UTC). Les 7 sont les
  sept KF-029 (#97) de `docs/testing.md` § *Les échecs attendus* : `mode-expert:26`, `:41`,
  `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`. Aucun huitième ; KF-045 (`invoices:415`, `:439`)
  verte, l'heure passée midi UTC. 271 = 267 (référence 27-4) + 4 de la spec neuve.

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
- 2026-10-13 — **Validation P2** (Opus 5.5 selon le prompt versionné, lentilles R et F, prompt
  `28-1-validate-prompt-p2.md` ; rapports hors dépôt `kesh-gate-logs/28-1-validate-p2-{R,F}.md`) :
  **R 0 CRITICAL / 0 HIGH / 6 MEDIUM / 10 LOW ; F 0 / 0 / 9 MEDIUM / 7 LOW** (bilans recomptés sur les titres par
  les lentilles, conformes). Après rapprochement (R1 = F2, R3 = F1, R4 ≈ F8, R6 = F9, R11 = F3 — titré LOW chez R,
  MEDIUM chez F —, R12 = F11) : **26 sujets distincts, dont 11 MEDIUM** ; sept des onze MEDIUM sont nés de la
  remédiation P1 (R1, R3, R4, R6, F3, F4, F5), quatre de la conception d'origine (R2, R5, F6, F7). Branche rebasée
  au préalable sur `origin/main` (`5aad8b35`, kickoff de l'Epic 28) ; registre de la story versé par union dans
  `epic-28-choix-autonomes.md`, fichier séparé supprimé ; `sprint-status.yaml` réuni par union.
  - **Constat D5 et décision.** La P1 avait rendu 3 HIGH et une douzaine de MEDIUM, la P2 rend une douzaine de
    MEDIUM distincts : sévérité égale ou supérieure au sens du second critère de découpage, et la majorité des
    défauts naissent du correctif précédent (C-28-1-19 à 25) — c'est le **recyclage** que l'amendement D5
    désigne, non une revue qui découvre des défauts d'origine. Décision de l'**orchestrateur**, sur l'autonomie
    déléguée par Guy le 2026-10-13 : pas de découpage formel, **réduction du périmètre au strict nécessaire pour
    fermer #640** (C-28-1-26). Signal déclaré au Project Lead par cette entrée.
  - **Retiré** (C-28-1-26) : la migration des cinq boutons stylés à la main (ancien AC5 ; l'AC5 devient « aucun
    site applicatif n'est édité ») ; le passage d'*Annuler le rapprochement* en `outline` (C-28-1-21 **annulé** :
    il rendait le bouton moins visible, bordure à 1,23:1 sans fond ni survol — F8, R4) ; la table « sites
    migrés » de l'AC6 et M18 ; les scénarios E2E (c), (d), (e) ; l'exemple `xs` de l'AC9 ; l'exigence de captures
    des 17 fichiers (R5 : remplacée par la vérification par construction, C-28-1-28) ; les déclencheurs `ghost`
    destructifs (R2) et la bordure `outline` (R4) sont renvoyés à **#645**, qui les porte déjà. Les cinq sites
    restants sont inscrits à la table de l'AC6 (quatorze entrées) et forment l'angle mort 11, **issue demandée**.
  - **Modules recomptés** : trois (`lib/components/ui`, `app.css`, `routes/design-system`) — la règle n'est plus
    franchie, la dérogation C-28-1-24 est sans objet (R6, F9), son risque accepté aussi (R12 = F11).
  - **Corrigé dans le périmètre** : R1 = F2 (`surface-hover`, treizième nom, AC7 (4) et angle mort 1) ; F1 = R3
    et R10 (une seule méthode de calcul, celle du helper : 5,46, 2,54, 4,14, 5,59 — C-28-1-27) ; F4 (M16 tuée par
    des cas « foncé en premier ») ; F5 + F6 (captures par état, dans le `finally` de chaque `test.step`, assertions
    `expect.soft`, passes clair et sombre en tests distincts — C-28-1-29) ; F7 (base remise à zéro avant chaque
    test de (b), un seul import par passe) ; F12 (*Annuler* du dialogue visé par `data-slot`, sans libellé) ; F13
    (`candidatesToCss` de `@tailwindcss/node`, déclaré en `devDependencies`, frontière de jeton, repli écrit —
    C-28-1-30 ; l'AC7 (4) n'est plus facultative) ; F15 (l'E2E ne recalcule plus de contraste, `parseRgb` retiré,
    `parseHex` pour l'AC7 (3) — C-28-1-31) ; F10 (palette Tailwind v4) ; F11 = R12 (sans objet) ; F14 (un seul
    exemple) ; F16 (CHANGELOG : `danger` et `error-soft` restent sans couleur, #645) ; R5 (AC6 (f), C-28-1-28) ;
    R7 (quatre *Annuler* `ghost` dans la classe A) ; R8 (`est grisé` → 0 après l'AC10, `admin-manual.tex:803`,
    `:1172` nommés, `grep -o | wc -l`) ; R9 (`InvoiceSettlements.svelte:86-95`, `:106-117`, relus au sol —
    la lentille donnait `:108`) ; R11 = F3 (clé de l'AC6 par `source.slice` de la valeur, gabarit JS compris) ;
    R13 (auto-tests sur source synthétique) ; R14 (epic sur `main`, « contact inventé » retiré) ; R15 (« aucun
    fond sous un texte ») ; R16 (point décimal dans la spec UX).
  - Recomptes : 13 AC, tâches T0-T9, mutations M1-M21 (M18 retirée : 20 actives), choix C-28-1-26 à C-28-1-31.
    Demandes à l'orchestrateur : issue des cinq boutons stylés à la main (angle mort 11) ; rectifier à nouveau le
    commentaire de #640 (« boutons stylés à la main » ne sont plus traités). Validation P3 :
    `28-1-validate-prompt-p3.md`.
- 2026-10-13 — **Validation P3** (Sonnet 5.5 selon le prompt versionné, lentilles R et F, prompt
  `28-1-validate-prompt-p3.md` ; rapports hors dépôt `kesh-gate-logs/28-1-validate-p3-{R,F}.md`) :
  **R 0 CRITICAL / 0 HIGH / 3 MEDIUM / 7 LOW ; F 0 / 0 / 3 MEDIUM / 3 LOW** (bilans recomptés sur les titres,
  conformes). Après rapprochement (F1 = R2, F3 = R3) : **14 sujets distincts, dont 4 MEDIUM** (F1 = R2, F2, F3 =
  R3, R1) et 10 LOW. Trend des MEDIUM distincts : P1 3 HIGH + 12 MEDIUM → P2 11 MEDIUM → **P3 4 MEDIUM**, 0 HIGH.
  Signal D5 : la sévérité reste MEDIUM, mais les défauts sont **distincts** de ceux de la P2 ; la lentille R n'y
  voit **aucun recyclage** d'un MEDIUM (deux LOW, R6 et R7, sont des résidus du retrait de périmètre), la lentille F
  en voit **deux** (F1 et F2 : le correctif de F6-P2 et de F5-P2 laisse le défaut sous une autre forme). Aucun ne
  touche de code de production : tous se corrigent dans la fiche. Pas de découpage ; signal déclaré au Project Lead
  par cette entrée.
  - **F1 = R2 (MEDIUM)** : délais de la spec fixés — `test.describe.configure({ timeout: 120_000 })`,
    `softExpect = expect.configure({ soft: true, timeout: 2_000 })` ; un timeout en T0 n'est pas le rouge attendu
    (AC8, T0, C-28-1-32).
  - **F2 (MEDIUM)** : captures de (a) et `b-bouton` par `page.screenshot({ clip })` élargi de 8 px — l'anneau et
    le contour, hors de la boîte de bordure, y figurent ; l'affirmation de l'AC3 et de l'angle mort 10 est gardée
    (C-28-1-33). **F5 (LOW)** : `page.mouse.move(0, 0)` avant le step de focus, fond asserté au repos (C-28-1-33).
  - **F3 = R3 (MEDIUM)** : `=` ajouté aux séparateurs du relevé de l'AC7 (4) ; 16 directives `class:` à préfixe
    de couleur dont 15 de couleur, relevées ; auto-tests sur source synthétique ; M22 ; angle mort 12 (jeton suivi
    d'une ponctuation) (C-28-1-34).
  - **R1 (MEDIUM)** : `accounts:717` décrit (changement de type, *Annuler* `outline`) ; 17 confirmations, 13
    `outline` + 4 `ghost`, dont 4 en ligne hors `Dialog.Root` ; AC8 « 13 des 24 » (C-28-1-36).
    *[Annotation P4 : la répartition était fausse à sa date — 12 dans un `Dialog.Root`, 5 hors, cf. entrée P4.]*
  - **LOW** : F4 (titres de tests portant le schéma) ; F6 (`@tailwindcss/node` au spécificateur de `tailwindcss`,
    assertion de version, C-28-1-35) ; R4 (six `<button>` bruts en `text-destructive`, nommés à l'AC4) ; R5
    (CHANGELOG : boutons bordés et ✕ qui deviennent rouges) ; R6 (C-28-1-17 rectifiée) ; R7 (contrôle muet
    `variant="outline"` retiré de la T9) ; R8 (**#648** cité au Status, à la classe D et à l'angle mort 11 ; un
    seul des cinq sites dépend du jeton ; la rectification du texte de #648 et du commentaire de #640 revient à
    l'orchestrateur) ; R9 (`BankImportUpload.svelte:344` — *[annotation P4 : fausse à sa date, `:345`]*) ; R10 (septième `ghost` à icône, `fiscal-years:417`).
  - Recomptes : 13 AC, tâches T0-T9, mutations M1-M22 (M18 retirée : 21 actives), angles morts 1-12, choix
    C-28-1-32 à C-28-1-36. Passe ciblée P4 (Haiku, une lentille, diff aplati de cette remédiation) :
    `28-1-validate-prompt-p4-ciblee.md`.
- 2026-10-13 — **Validation P4 ciblée** (Haiku, une lentille sur le seul commit de remédiation P3 `2d794013`,
  prompt `28-1-validate-prompt-p4-ciblee.md` ; rapport hors dépôt `kesh-gate-logs/28-1-validate-p4-ciblee.md`) :
  **0 CRITICAL / 0 HIGH / 1 MEDIUM / 1 LOW**, tous deux nés de la remédiation P3, vérifiés par l'orchestrateur au
  sol. **P4-1 (MEDIUM, décompte)** : 12 confirmations dans un `Dialog.Root` et **5** hors — `journal-entries/[id]`
  est un overlay `role="dialog"` maison (`:526-530`) ; corrigé à l'AC8, en Dev Notes § Classe A et au registre
  (C-28-1-36, annotation). **P4-2 (LOW)** : le contrôle du doublon est `BankImportUpload.svelte:345` ; corrigé à
  l'AC8 et au registre. Les entrées P3 portent une annotation datée, sans réécriture. Aucun ne touche de code.
  **Validation close.** Trend : P1 (Sonnet) 3 HIGH + ~12 MEDIUM → P2 (Opus) ~11 MEDIUM (constat D5 : périmètre
  réduit à #640, trois modules) → P3 (Sonnet) 4 MEDIUM → P4 ciblée (Haiku) 1 MEDIUM de décompte, fiche seule →
  close (la remédiation P4 ne touche aucune ligne de code). Statut `in-progress`.
- 2026-10-13 — **Développement** (`bmad-dev-story`, Opus 5.5, autonomie) : T0-T9 faites. Phase rouge T0 consignée
  (22 `toHaveCSS` rouges, aucun timeout, 12 captures « avant ») avant toute édition d'`app.css` et de
  `button.svelte` ; jetons (AC1), variante (AC2), trois gardes Vitest et le helper de contraste (41 tests neufs),
  E2E `destructive-buttons.spec.ts` (4 tests) et montage extrait, `/design-system`, manuels et PDF, spec UX,
  CHANGELOG `## [0.13.3] — Non publié`. Mutations **21 / 21** actives tuées (M20 en deux essais). Gates au `HEAD`
  `3b0f7707` (dernier commit de code `37b74298`) : `test-fast.sh` 3514/3514 (6 ignorés) ; Vitest 1436/1436 ;
  `check`, `lint-i18n-ownership`, `build` verts ; E2E complet 269 passés, 9 échoués = 7 KF-029 + 2 KF-045 (avant
  12:00 UTC), voir le Dev Agent Record pour le rejeu. Choix C-28-1-37 à C-28-1-40. Statut `review`.
- 2026-10-13 — **Revue de code P1** (Sonnet selon le prompt versionné `28-1-review-prompt-p1.md`, lentilles B, E et
  A ; rapports hors dépôt `kesh-gate-logs/28-1-review-p1-{B,E,A}.md`, diff `bbd0a9a1..3b0f7707`) : **B 0 / 0 / 0 /
  3 LOW ; E 0 / 0 / 2 MEDIUM / 4 LOW ; A 0 / 0 / 0 / 3 LOW** (+ 3 observations sans sévérité), bilans recomptés sur
  les titres des rapports, soit **0 CRITICAL / 0 HIGH / 2 MEDIUM / 10 LOW** bruts. Après rapprochement (B1 = A-L1,
  B3 = E5, E3 = A-L3) : **9 sujets distincts — 2 MEDIUM (E1, E2), 7 LOW** ; aucun ne touche de code de production. Remédiés par
  `99682720` (Opus 5.5), tests, CHANGELOG et lockfile seuls :
  - **E1 (MEDIUM)** : l'E2E (b) attend la fin des animations du document et exige que `color-contrast` ait
    **évalué** *Supprimer définitivement* (`passes`, non `incomplete`) ; P1-M10 et P1-M11 rougissent (C-28-1-44).
  - **E2 (MEDIUM)** : `focus-visible:ring-destructive/40` exigé en (1), `focus-visible:ring-ring/50` absent en
    (1-bis) ; P1-M1 rougit ; angle mort 10 annoté.
  - **LOW** : B1 = A-L1 (CHANGELOG : la croix d'écriture ne rougit qu'au survol ; amendement sous l'AC12) ; B2
    (directive `class:` refusée par (f), auto-test) ; B3 = E5 (`"license": "MIT"` rétabli à la main, C-28-1-42) ;
    E3 = A-L3 (montage `>= 25` par l'analyseur, égal au compte textuel, trois auto-tests, C-28-1-41) ; E4 (`!`
    final et préfixes `shadow-`/`from-`/`via-`/`to-`/`ring-offset-` relevés, `.js`/`app.html` en angle mort 13,
    C-28-1-43) ; E6 (`@theme` lu sans commentaires, auto-test) ; A-L2 (Status : C-28-1-1 à C-28-1-44, recompté).
  - Recomptes : choix **C-28-1-41 à C-28-1-44** (`grep -c '^### C-28-1-'` → 44) ; mutations P1-M1 à P1-M11,
    **11 / 11 rouges** ; tests Vitest **+7** exécutés de `5ded5663` à `99682720` (1436 → 1443) ; angles morts 1-13.
  - Gates au dernier commit de code `99682720`, sous `flock`, bases remises à zéro : `test-fast.sh` 3514/3514
    (6 ignorés) ; Vitest 1443/1443 ; `check`, `lint-i18n-ownership`, `build` verts ; E2E complet **271 passés,
    7 échoués = les 7 KF-029**, 19 ignorés (KF-045 verte, après midi UTC).
  - Passe ciblée P2 (Haiku, une lentille, diff aplati de cette remédiation) : `28-1-review-prompt-p2-ciblee.md`,
    rapport attendu `kesh-gate-logs/28-1-review-p2-ciblee.md`.
- 2026-10-13 — **Revue de code P2 ciblée** (Haiku, une lentille sur le diff aplati `5ded5663..b577b8da`, prompt
  `28-1-review-prompt-p2-ciblee.md` ; rapport hors dépôt `kesh-gate-logs/28-1-review-p2-ciblee.md`) :
  **0 CRITICAL / 0 HIGH / 0 MEDIUM / 3 LOW**, tous nés de la remédiation P1, aucun dans le code de production.
  Remédiés par `f3fadb5a` (Opus 5.5), deux fichiers de test seuls :
  - **P2-1** : le relevé AC7 (4) retire l'opacité et le `!` dans les deux ordres (`bg-foo!/40` comme
    `bg-foo/40!`) ; auto-test neuf ; mutation P2-M1 (retour à l'ordre `!` puis opacité) **rouge** sur cet auto-test
    (`kesh-gate-logs/28-1-mut-p1/P2-M1-vitest.log`).
  - **P2-2** : restreindre `to-`/`from-`/`via-` aux valeurs de `class` n'est pas simple (des classes passent par
    `cn(…)` en script) ; écrit en angle mort, au préambule de la garde et à l'angle mort 13.
  - **P2-3** : commentaire du montage de (f) corrigé (un usage ajouté sans `class` n'est pas « tenu par (f) », il
    n'est pas fautif) ; C-28-1-41 annoté au registre, sans réécriture.
  - Gates au commit `f3fadb5a` : gate ciblé Vitest sur les trois gardes 40/40 ; `npm run test:unit` complet sous
    `flock` **1444 / 1444**, 138 fichiers (+1 depuis `99682720`) ; `npm run check` 0 erreur, 29 avertissements.
    E2E non rejoué : ni production ni spec E2E touchées depuis `99682720` (E2E complet vert à ce commit).
  - **Trend de la revue de code** : P1 (Sonnet B/E/A) 0/0/2/10 bruts, 9 sujets → P2 ciblée (Haiku) 0/0/0/3, toutes
    remédiations en tests seuls → **boucle close** (seulement des LOW, et la remédiation finale ne touche aucune
    ligne de production). Statut **`done`**.
