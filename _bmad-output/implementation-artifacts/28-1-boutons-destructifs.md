# Story 28.1 : Les boutons d'action destructive se voient

## Status

**ready-for-dev** — fiche créée le 2026-10-13 en autonomie (`bmad-create-story`), sur `6f971d76`
(rétrospective de l'Epic 27), branche `story/28-1`, worktree `/home/gcorbaz/devel/kesh-28-1`. Choix consignés
au registre `epic-28-choix-autonomes-28-1.md` (**C-28-1-1 à C-28-1-16**), fusionné plus tard par union dans
`epic-28-choix-autonomes.md`. 13 AC (AC1-AC13), tâches T0-T10, mutations M1-M17. Validation P1 à lancer :
prompt `28-1-validate-prompt-p1.md`. PR : **`closes #640`**.

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
jeton (`bg-`/`text-`/`border-`/`ring-destructive`) dans **42 fichiers** hors tests, dont **33 bandeaux
d'erreur** `border-destructive bg-destructive/10 text-destructive`, s'affichent aujourd'hui **sans aucun
rouge**. `app.css` et `button.svelte` n'ont pas bougé depuis les Stories 1.9 et 7-1 : le défaut est d'origine.

## Critères d'acceptation

**AC1 — Trois jetons neufs dans le thème, mesurés (C-28-1-1, C-28-1-3, C-28-1-4).**

Dans `frontend/src/app.css`, bloc `@theme`, après `--color-error` :

```css
  /* Action destructive (Story 28-1, #640) — distincte de --color-error (#dc2626), trop
     clair pour un texte sur bandeau bg-destructive/10 (4,13:1 < 4,5). Ratios WCAG mesurés :
     blanc sur destructive 6,47:1 ; blanc sur destructive-hover 8,31:1 ; destructive sur
     blanc 6,47:1 ; destructive sur bandeau /10 (#f8e8e8) 5,45:1 ; sur /20 4,59:1. */
  --color-destructive: #b91c1c;
  --color-destructive-foreground: #ffffff;
  --color-destructive-hover: #991b1b;
```

Après `npm run build`, `grep -c 'bg-destructive' build/_app/immutable/assets/*.css` rend au moins une
occurrence (la commande et sa sortie au Dev Agent Record). `--color-error` **ne change pas** (73 lignes
`text-error`/`bg-error`/`text-red-*` hors périmètre, cf. Dev Notes § Inventaire).

**AC2 — La variante `destructive` du `Button` partagé a un fond plein (C-28-1-2).**

`frontend/src/lib/components/ui/button/button.svelte:14` devient exactement :

```ts
destructive: "bg-destructive text-destructive-foreground hover:bg-destructive-hover focus-visible:ring-destructive/40",
```

- **aucune** classe `dark:` dans la variante (C-28-1-7 : Kesh n'a pas de thème sombre, et le variant `dark` de
  Tailwind v4 est `@media (prefers-color-scheme: dark)` — la `dark:bg-destructive/20` d'aujourd'hui rendrait le
  bouton **à nouveau pâle** chez un utilisateur dont le système est en sombre, dès que le jeton existe) ;
- la classe de base (`button.svelte:7`) **ne change pas** : `disabled:opacity-50`, `outline-none`,
  `focus-visible:ring-3`, `active:…translate-y-px` restent communs à toutes les variantes ;
- doc-comment `//` au-dessus de la ligne : renvoi à #640, à AC1 pour les ratios, et la phrase « pas de `dark:` :
  voir C-28-1-7 ».

**AC3 — Les quatre états se distinguent, mesurés (C-28-1-4, C-28-1-5, C-28-1-6, C-28-1-7).**

| État | Rendu attendu (style calculé, Chromium) | Contraste mesuré | Exigence |
|---|---|---|---|
| repos | fond `rgb(185, 28, 28)`, texte `rgb(255, 255, 255)` | 6,47:1 (texte) ; 6,47:1 (fond / page blanche) | 1.4.3 ≥ 4,5 ; 1.4.11 ≥ 3 |
| survol | fond `rgb(153, 27, 27)` | 8,31:1 | 1.4.3 ≥ 4,5 |
| focus clavier | `outline-style: solid`, `outline-color: rgb(59, 130, 246)` (règle globale `app.css:62-65`) | 3,68:1 contre la page (le contour est décalé de 2 px) | 2.4.7 ; 1.4.11 ≥ 3 |
| désactivé | `opacity: 0.5`, fond **toujours** `rgb(185, 28, 28)` (l'opacité s'applique à l'élément) | 2,53:1 effectif | **exempté** (1.4.3 : composant inactif) — distinct du repos (C-28-1-5) |
| repos, `prefers-color-scheme: dark` émulé | **identique** au repos clair | — | C-28-1-7 |

La règle globale `:focus-visible` est hors couche (`app.css:62`) et l'emporte sur `outline-none`, qui vit dans
`@layer utilities` : **à vérifier par l'E2E de l'AC8, non à supposer**. Si le style calculé dément cette
lecture, corriger (par exemple `focus-visible:outline-2 focus-visible:outline-primary-light` dans la variante)
et l'écrire au registre.

**AC4 — Le jeton défini rend leur rouge aux autres usages du jeton, sans édition de site (C-28-1-8).**

Définir `--color-destructive` active d'un coup tous les usages du jeton qui ne sont pas la variante (sur les
201 occurrences dans 42 fichiers, 29 sont dans 4 fichiers de `components/ui/`, dont la variante elle-même) : les 33
bandeaux d'erreur, les textes d'erreur `text-destructive` (`<p>`, `<span>`, `<td>`, `<li>`), les bordures de
champ en erreur (`border-destructive` sur `Input`, `aria-invalid:border-destructive` de `input.svelte`,
`select-trigger.svelte` — 11 sites `aria-invalid` hors `ui/`), la section `border-destructive` de
`settings/vat-rates/+page.svelte:356`, et les trois `<button>` bruts en `text-destructive` (Dev Notes §
Inventaire, classe C et D). **Aucun de ces sites n'est édité** pour l'AC4 — c'était leur intention d'origine.

- Contraste garanti par AC1 : texte sur blanc 6,47:1, texte sur bandeau `/10` 5,45:1.
- Les **dix specs E2E** qui lancent `AxeBuilder` (`contacts`, `bank-account-journal-link`, `dunning`,
  `reconciliation-rules`, `reconciliation`, `bank-import`, `bank-import-confirms`, `homepage-dashboard`,
  `bank-csv-import`, `homepage-settings`) restent vertes : c'est le contrôle indépendant que le rouge neuf ne
  crée aucune violation `color-contrast` là où un bandeau est affiché à l'analyse.

**AC5 — Cinq boutons stylés à la main passent à la variante (C-28-1-9).**

| Site | Aujourd'hui | Devient |
|---|---|---|
| `routes/(app)/settings/fiscal-years/+page.svelte:603` (`fiscal-year-close-confirm`) | `<Button class="bg-red-600 hover:bg-red-700">` | `<Button variant="destructive">`, `class` retiré |
| `routes/(app)/settings/email-templates/+page.svelte:439` (`email-template-restore-confirm`) | `<Button class="bg-red-600 hover:bg-red-700">` | `<Button variant="destructive">`, `class` retiré |
| `routes/(app)/bank-import/[id]/+page.svelte:141-149` (`detail-tx-cancel-reconciliation`) | `<button class="ml-2 text-destructive underline">` — se lit comme un lien | `<Button variant="destructive" size="xs" class="ml-2">` |
| `lib/features/contacts/ContactPersonsManager.svelte:112-119` (`contact-person-delete-{id}`) | `<button class="text-red-600 hover:underline">` | `<Button variant="destructive" size="xs">` |
| `lib/features/bank-import/BankProfileList.svelte:75-83` (`bank-import-profile-delete-button`) | `<button class="ml-2 text-error">` | `<Button variant="destructive" size="xs" class="ml-2">` |

`data-testid`, `onclick`, `disabled`, libellé `i18nMsg` (clé et repli) **inchangés** ; le `Button` partagé pose
`type="button"` par défaut — aucun de ces sites n'est dans un `<form>` (vérifié : Dev Notes § Inventaire).
Import `import { Button } from '$lib/components/ui/button';` ajouté là où il manque.

**AC6 — Inventaire clos des boutons rouges posés à la main : une garde Vitest (C-28-1-11).**

Fichier neuf `frontend/src/lib/components/ui/button/boutons-rouges-inventory.test.ts`, sur le patron de
`frontend/src/lib/features/bank-accounts/bank-name-inventory.test.ts` (Story 26-2b) :

- parcourt **tous** les `.svelte` de `src/` (hors `.test.`), les analyse par `parse(source, { modern: true })` de
  `svelte/compiler` (pas d'expression régulière sur des balises : un `onclick={() => …}` contient un `>`) ;
- pour chaque élément `button` (`RegularElement`) et chaque composant `Button` (`Component`), lit le **texte
  source** de ses attributs `class` (littéral **ou** expression) et de ses directives `class:` ;
- relève tout jeton `(text|bg|border|ring|outline)-(destructive|error|red-\d{2,3})(/\d+)?`, préfixes de variante
  compris (`hover:text-destructive`) ;
- exige que l'ensemble `{ fichier:élément → jetons }` **égale** une table fermée, chaque entrée avec son motif.
  Après la story, la table contient **exactement neuf entrées** : les six déclencheurs bordés de la classe C et
  les trois icônes ✕ de la classe E (Dev Notes § Inventaire). Une entrée en trop **ou** en moins fait rougir
  (les deux sens, M12/M13) ;
- doc-comment : ce qu'elle voit, ce qu'elle **ne voit pas** (rouge passé par une variable de script, par `cn(…)`
  dans le `<script>`, par un `style=`, ou porté par un `<a>`/`<div role="button">` — aucun aujourd'hui, vérifié),
  et comment ajouter une entrée légitime (« avec son motif : c'est le but — le changement se voit en revue »).

La variante `destructive` du `Button` est la **seule** source du rouge d'un bouton hors table : la garde ne
contrôle pas `variant=`, qui n'est pas une classe.

**AC7 — Une garde Vitest qui rougit si la variante redevient pâle (C-28-1-11, C-28-1-12).**

Fichier neuf `frontend/src/lib/components/ui/button/button-destructive.test.ts` :

1. `buttonVariants({ variant: 'destructive' })` contient les jetons `bg-destructive`, `text-destructive-foreground`
   et `hover:bg-destructive-hover` **comme mots entiers** (`/(^|\s)bg-destructive(\s|$)/` — `bg-destructive/10`
   ne compte pas) ; ne contient **aucun** jeton préfixé `dark:` ; ne contient aucun `bg-transparent` ni
   `bg-destructive/<n>` hors préfixe `focus-visible:`.
2. `readFileSync('src/app.css')` : les trois jetons de l'AC1 sont définis dans `@theme` (expression extraite par
   `--color-destructive:\s*(#[0-9a-f]{6})`) ; **tout** jeton de couleur employé par la variante
   (`bg-X`, `text-X`, `hover:bg-X`, `ring-X` → `--color-X`) est défini — c'est la classe de défaut d'origine.
3. Contrastes recalculés **depuis les valeurs lues** dans `app.css`, par le helper de l'AC8 :
   `destructive-foreground`/`destructive` ≥ 4,5 ; `destructive-foreground`/`destructive-hover` ≥ 4,5 ;
   `destructive`/`#ffffff` ≥ 4,5 (texte d'erreur) ; `destructive`/mélange 10 % sur blanc ≥ 4,5 (bandeau) ;
   `destructive`/`#ffffff` ≥ 3 (limite du bouton, 1.4.11).

**AC8 — Recette visuelle : une spec Playwright qui mesure le style calculé et laisse des captures (C-28-1-10).**

Fichier neuf `frontend/tests/e2e/destructive-buttons.spec.ts`, et helper `frontend/tests/e2e/helpers/contrast.ts`
(fonctions pures `parseRgb`, `relativeLuminance`, `contrastRatio`, `blend` ; tests unitaires
`frontend/tests/e2e/helpers/contrast.test.ts` — Vitest les ramasse par `tests/**/*.test.ts`, Playwright les
ignore par `testMatch`). Chaque scénario tourne **deux fois**, `page.emulateMedia({ colorScheme: 'light' })` puis
`'dark'` :

- **(a) `/design-system`** (route publique, hors `(app)`) — section *Boutons* : le bouton destructif au repos,
  survolé (`hover()`), focalisé **au clavier** (`getByTestId('ds-destructive').focus()`, puis
  `keyboard.press('Shift+Tab')` et `keyboard.press('Tab')` pour que le focus vienne du clavier ;
  `evaluate(el => el.matches(':focus-visible'))` doit rendre `true` avant de lire le contour), désactivé (exemple ajouté par
  l'AC9) : valeurs du tableau de l'AC3, contraste recalculé dans le test depuis `getComputedStyle` ≥ 4,5
  (repos, survol) ;
- **(b) détail d'un import** — *Supprimer l'import* (`bank-import-delete-button`) : fond **non transparent**
  (canal alpha 1) et égal à `rgb(185, 28, 28)`, contraste ≥ 4,5 ; ouverture du dialogue
  (`bank-import-delete-dialog`) : *Annuler* (`outline`) et *Supprimer définitivement*
  (`bank-import-delete-submit`) ont des fonds **différents** ; `new AxeBuilder({ page })
  .include('[data-testid="bank-import-delete-dialog"]').withRules(['color-contrast']).analyze()` → 0 violation ;
- **(c) liste des profils** (`/bank-import/profiles`) — `bank-import-profile-delete-button` (taille `xs`,
  AC5) : fond `rgb(185, 28, 28)` ;
- **(d) dialogue d'archivage d'un compte** (`/accounts`, plan comptable du seed `with-company`) — ouvert par
  `account-row-<numéro>-archive-button` (`accounts/+page.svelte:499`, déclencheur `ghost`), le bouton
  `account-archive-dialog-confirm` (`:742`) à côté d'`account-archive-dialog-cancel` (`:740`, `autofocus`) : mêmes
  assertions que (b), axe compris ; le dialogue est **refermé par *Annuler*** (rien n'est archivé).

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
`grep -c "le bouton est grisé, avec le nombre"` → 0. Le CHANGELOG de la 0.13.2 (« grisé ») est historique : non
touché.

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
>   rappels, restauration) s'affichaient comme du texte : leur couleur n'était pas définie dans le thème. Ils
>   ont désormais un fond rouge plein et un texte blanc (contraste 6,47:1), un survol plus foncé et un état
>   désactivé estompé. Les messages et cadres d'erreur, qui devaient être rouges et ne l'étaient pas, le
>   deviennent. *Annuler le rapprochement* (détail d'un import), la suppression d'une personne de contact et
>   celle d'un profil d'import prennent la même forme de bouton.

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
      `helpers/bank-import-fixtures.ts` (extraction, AC8) et `destructive-buttons.spec.ts` **avant** de toucher
      `app.css` ou `button.svelte` ; la lancer seule avec `KESH_E2E_CAPTURES_DIR=/home/gcorbaz/devel/kesh-gate-logs/28-1-captures/avant`
      — elle **doit rougir** sur les assertions de fond (sortie copiée au Debug Log) et laisser les captures
      « avant ». ⚠️ Le scénario (a) a besoin des exemples de l'AC9 : faire d'abord la T5.
- [ ] **T1 — Jetons** (AC1) dans `app.css`, doc-comment compris.
- [ ] **T2 — Variante** (AC2) dans `button.svelte:14`.
- [ ] **T3 — Gardes Vitest** (AC6, AC7) ; table de l'AC6 à neuf entrées, motifs écrits.
- [ ] **T4 — Cinq sites** (AC5).
- [ ] **T5 — Page `/design-system`** (AC9).
- [ ] **T6 — Captures « après »** : la spec neuve verte, captures dans `…/28-1-captures/apres`.
- [ ] **T7 — Manuels** (AC10) + `make fr` sous `flock` + contrôles aplatis copiés au Debug Log.
- [ ] **T8 — Spec UX et CHANGELOG** (AC11, AC12).
- [ ] **T9 — Mutations M1-M17** (§ ci-dessous), chacune restaurée par `git checkout -- <fichier>` puis `touch`
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
l'issue (« se lit comme du texte ») : `bank-import/[id]/+page.svelte:143` (souligné, à côté du lien *Voir
l'écriture*, sur l'écran même de la recette), `ContactPersonsManager.svelte:114`, `BankProfileList.svelte:79`.

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
`outline`), `RulesList.svelte:161` (*Désactiver*, bordé neutre). Ils **se voient** (bordure ou survol) et
ouvrent une confirmation destructive : leur sobriété est une hiérarchie voulue, non le défaut de l'issue.

### Angles morts assumés (à écrire au Dev Agent Record)

1. **Neuf autres jetons shadcn ne sont pas définis non plus** — `muted`, `muted-foreground`, `foreground`,
   `background`, `ring`, `input`, `accent`, `accent-foreground`, `card` (ordre de grandeur : `muted-foreground`
   seul ≈ 58 occurrences dans 23 fichiers). Effet visible : la variante `outline` (*Annuler*) n'a ni fond ni
   survol (`bg-background`, `hover:bg-muted` sans règle), `ghost` n'a pas de survol, `focus-visible:ring-ring`
   est sans effet (le contour global bleu compense). **Hors périmètre** (C-28-1-13) : *Annuler* reste lisible
   (bordure `border-border`, jeton défini). **Issue à ouvrir par l'orchestrateur** (P3, `bug`) — elle fera
   remonter le décompte, ce que le kickoff déclare recevable.
2. **Suppression d'une personne de contact sans confirmation** (`ContactPersonsManager.svelte:76-83` :
   `removePerson` appelle `deleteContactPerson` directement) — contraire au principe UX « confirmations avant
   actions destructives » (spec UX l. 286). La story rend le bouton visible, elle n'ajoute pas de confirmation.
   **Issue à ouvrir** (P3).
3. **Sous `prefers-color-scheme: dark`**, des classes `dark:` d'autres composants s'activent déjà
   (`dark:bg-amber-950/30`, `dark:bg-green-900`… émises dans le build) alors que Kesh n'a pas de thème sombre.
   Préexistant, hors périmètre ; la variante destructive n'en porte plus aucune (AC2).
4. **Taille de cible** : `h-8` (32 px) pour toutes les variantes, sous les 44 px du mode Guidé (spec UX l. 718).
   Préexistant, commun à tous les boutons.
5. **Pas de régression au pixel** : les captures ne sont pas comparées (C-28-1-10). Une régression de style hors
   des propriétés mesurées (taille, police, espacement) n'est vue que par l'œil.
6. **La CI n'exécute aucun E2E** (#577) : en CI, seules les gardes Vitest (AC6, AC7) protègent la variante.

### État actuel des fichiers modifiés — ce qui doit être préservé

- `button.svelte` : `buttonVariants` est **exporté** (`ui/button/index.ts:5,13`), sans autre consommateur dans
  `src/` au 2026-10-13 (`grep -rn buttonVariants frontend/src`) ; seules la ligne `destructive` change. Le rendu `<a>` (avec `href`) et
  `<button>` reste identique.
- `app.css` : ordre du `@theme`, styles globaux, espacement Guidé/Expert et règle `:focus-visible` inchangés.
- Sites de l'AC5 : comportements préservés — la
  `confirm()` native de `BankProfileList.handleDelete` (`:26`), l'ouverture de `CancelReconciliationDialog` par
  `cancelTxId` (`bank-import/[id]/+page.svelte:147`), l'absence de confirmation de `removePerson`, les
  `disabled={closeSubmitting}` / `{restoreSubmitting}` des dialogues B. Tests qui ciblent ces boutons, par
  `data-testid` seulement (aucun n'assert une classe) : `fiscal-years-page.test.ts:282`,
  `e2e/fiscal-years.spec.ts`, `e2e/email-templates.spec.ts`, `e2e/reconciliation-cancel.spec.ts:149`.
- `bank-import-delete.spec.ts` : son scénario ne change pas, seul son montage passe par le helper extrait.

### Propagation post-patch (T10)

Greper la **valeur** après chaque correctif : `grep -rnE '#b91c1c|#991b1b|6,47|5,45|8,31|4,59|2,53' frontend/src
_bmad-output/implementation-artifacts/28-1-* _bmad-output/planning-artifacts/ux-design-specification.md
CHANGELOG.md` (les ratios cités doivent tous être ceux d'`app.css`) ; `grep -rn 'bg-destructive/10 hover'
frontend/src` → 0 ; `grep -rn 'est grisé' docs/manual/fr/*.tex` → seul `user-manual.tex:1107`.

### Découpage préventif (C-28-1-15)

Modules touchés : `components/ui` (button), le thème (`app.css`), `features/contacts`, `features/bank-import`
(+ sa route), `routes/(app)/settings` (deux pages), `routes/design-system`, et la documentation. Cinq modules
de premier niveau au plus, aucune règle métier, aucune migration : **pas de découpage**. Les 24 sites de la
classe A ne sont **pas** édités.

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
